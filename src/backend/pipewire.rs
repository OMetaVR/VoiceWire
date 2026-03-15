use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::backend::{AudioBackend, BackendCommand, BackendEvent};
use crate::model::{AppRouteState, DeviceOption, EndpointKind, MeterSnapshot, SessionState, StereoLevels};

const VIRTUAL_INPUT_ENDPOINTS: [VirtualEndpointSpec; 3] = [
    VirtualEndpointSpec::new("voicewire.in1", "VoiceWire VAIO", "Audio/Sink"),
    VirtualEndpointSpec::new("voicewire.in2", "VoiceWire AUX", "Audio/Sink"),
    VirtualEndpointSpec::new("voicewire.in3", "VoiceWire SYS", "Audio/Sink"),
];

const VIRTUAL_BUS_ENDPOINTS: [VirtualBusEndpointSpec; 3] = [
    VirtualBusEndpointSpec::new(
        "voicewire.internal.b1",
        "VoiceWire-B1-Internal",
        "voicewire.b1",
        "VoiceWire-B1",
    ),
    VirtualBusEndpointSpec::new(
        "voicewire.internal.b2",
        "VoiceWire-B2-Internal",
        "voicewire.b2",
        "VoiceWire-B2",
    ),
    VirtualBusEndpointSpec::new(
        "voicewire.internal.b3",
        "VoiceWire-B3-Internal",
        "voicewire.b3",
        "VoiceWire-B3",
    ),
];

#[derive(Clone, Copy)]
struct VirtualEndpointSpec {
    node_name: &'static str,
    description: &'static str,
    media_class: &'static str,
}

impl VirtualEndpointSpec {
    const fn new(
        node_name: &'static str,
        description: &'static str,
        media_class: &'static str,
    ) -> Self {
        Self {
            node_name,
            description,
            media_class,
        }
    }
}

#[derive(Clone, Copy)]
struct VirtualBusEndpointSpec {
    sink_node_name: &'static str,
    sink_description: &'static str,
    source_node_name: &'static str,
    source_description: &'static str,
}

impl VirtualBusEndpointSpec {
    const fn new(
        sink_node_name: &'static str,
        sink_description: &'static str,
        source_node_name: &'static str,
        source_description: &'static str,
    ) -> Self {
        Self {
            sink_node_name,
            sink_description,
            source_node_name,
            source_description,
        }
    }
}

#[derive(Clone)]
struct VirtualEndpointNode {
    id: u32,
    name: String,
    media_class: String,
}

#[derive(Clone)]
struct PulseModuleRecord {
    id: u32,
}

#[derive(Clone, PartialEq, Eq)]
struct PipeWireDeviceNode {
    id: u32,
    name: String,
    stable_id: String,
    node_name: String,
    device_class: String,
    kind: EndpointKind,
}

impl PipeWireDeviceNode {
    fn to_option(&self) -> DeviceOption {
        DeviceOption {
            name: self.name.clone(),
            kind: self.kind,
            stable_id: self.stable_id.clone(),
            device_class: self.device_class.clone(),
            connected: true,
        }
    }
}

#[derive(Clone)]
struct BusBinding {
    bus_index: usize,
    stable_id: String,
    node_name: String,
    node_id: u32,
}

#[derive(Clone)]
struct ActiveRoute {
    strip_index: usize,
    bus_index: usize,
    links: Vec<RouteLink>,
}

#[derive(Clone)]
struct RouteLink {
    source_port: String,
    dest_port: String,
}

struct StripMeterTap {
    strip_index: usize,
    target_name: String,
    latest_levels: Arc<Mutex<StereoLevels>>,
    stop: Arc<AtomicBool>,
    child_slot: Arc<Mutex<Option<Child>>>,
    handle: Option<JoinHandle<()>>,
}

pub struct PipeWireDiscoveryBackend {
    known_input_nodes: Vec<PipeWireDeviceNode>,
    known_input_devices: Vec<DeviceOption>,
    known_output_devices: Vec<DeviceOption>,
    known_output_nodes: Vec<PipeWireDeviceNode>,
    connected: bool,
    virtual_endpoint_ids: Vec<u32>,
    virtual_bus_module_ids: Vec<u32>,
    bus_bindings: Vec<BusBinding>,
    active_routes: Vec<ActiveRoute>,
    strip_meter_taps: Vec<StripMeterTap>,
    last_device_refresh: Option<Instant>,
}

impl Default for PipeWireDiscoveryBackend {
    fn default() -> Self {
        Self {
            known_input_nodes: Vec::new(),
            known_input_devices: Vec::new(),
            known_output_devices: Vec::new(),
            known_output_nodes: Vec::new(),
            connected: false,
            virtual_endpoint_ids: Vec::new(),
            virtual_bus_module_ids: Vec::new(),
            bus_bindings: Vec::new(),
            active_routes: Vec::new(),
            strip_meter_taps: Vec::new(),
            last_device_refresh: None,
        }
    }
}

impl AudioBackend for PipeWireDiscoveryBackend {
    fn handle(&mut self, session: &SessionState, command: BackendCommand) -> Vec<BackendEvent> {
        let is_start = matches!(&command, BackendCommand::Start);
        let is_shutdown = matches!(&command, BackendCommand::Shutdown);
        let mut events = match &command {
            BackendCommand::Start
            | BackendCommand::SetStripDevice { .. }
            | BackendCommand::SetBusDevice { .. } => self.refresh_device_events(),
            BackendCommand::RefreshMeters => {
                self.refresh_device_events_if_stale(Duration::from_millis(1000))
            }
            _ => Vec::new(),
        };
        let devices_changed = events
            .iter()
            .any(|event| matches!(event, BackendEvent::DevicesUpdated { .. }));

        match command {
            BackendCommand::Start => {
                if self.connected {
                    events.push(BackendEvent::ConnectionStateChanged { connected: true });
                }
                events.extend(self.ensure_virtual_endpoints());
                events.extend(self.ensure_session_bus_bindings(session));
                self.sync_all_runtime_mix_state(session);
                events.extend(self.sync_all_routes(session));
                events.extend(self.refresh_app_routes());
                events.push(self.next_meter_event(session));
            }
            BackendCommand::Shutdown => {
                self.clear_all_routes();
                self.stop_all_meter_taps();
                self.destroy_virtual_endpoints();
                events.push(BackendEvent::ConnectionStateChanged { connected: false });
                events.push(BackendEvent::VirtualEndpointsReady { ready: false });
            }
            BackendCommand::PreviewStripGain {
                strip_index,
                gain_db,
            } => {
                let clamped = gain_db.clamp(-60.0, 12.0);
                let mut temp = session.clone();
                temp.set_strip_gain(strip_index, clamped);
                self.sync_strip_runtime_state(strip_index, &temp);
            }
            BackendCommand::PreviewBusGain { bus_index, gain_db } => {
                let clamped = gain_db.clamp(-60.0, 12.0);
                let mut temp = session.clone();
                temp.set_bus_gain(bus_index, clamped);
                self.sync_bus_runtime_state(bus_index, &temp);
            }
            BackendCommand::RefreshMeters => {
                events.push(self.next_meter_event(session));
            }
            BackendCommand::RefreshAppRoutes => {
                events.extend(self.refresh_app_routes());
            }
            BackendCommand::ToggleRoute {
                strip_index,
                bus_index,
            } => {
                let enabled = session
                    .strips
                    .get(strip_index)
                    .and_then(|strip| strip.routes.get(bus_index))
                    .map(|route| !route.enabled)
                    .unwrap_or(false);
                events.push(BackendEvent::RouteChanged {
                    strip_index,
                    bus_index,
                    enabled,
                });
                let mut temp = session.clone();
                temp.set_route_enabled(strip_index, bus_index, enabled);
                events.extend(self.sync_route(strip_index, bus_index, &temp));
                events.push(self.next_meter_event_with_update(session, |temp| {
                    temp.set_route_enabled(strip_index, bus_index, enabled);
                }));
            }
            BackendCommand::SetStripGain {
                strip_index,
                gain_db,
            } => {
                let clamped = gain_db.clamp(-60.0, 12.0);
                events.push(BackendEvent::StripGainChanged {
                    strip_index,
                    gain_db: clamped,
                });
                let mut temp = session.clone();
                temp.set_strip_gain(strip_index, clamped);
                self.sync_strip_runtime_state(strip_index, &temp);
                events.push(self.next_meter_event_with_update(session, |temp| {
                    temp.set_strip_gain(strip_index, clamped);
                }));
            }
            BackendCommand::SetBusGain { bus_index, gain_db } => {
                let clamped = gain_db.clamp(-60.0, 12.0);
                events.push(BackendEvent::BusGainChanged {
                    bus_index,
                    gain_db: clamped,
                });
                let mut temp = session.clone();
                temp.set_bus_gain(bus_index, clamped);
                self.sync_bus_runtime_state(bus_index, &temp);
                events.push(self.next_meter_event_with_update(session, |temp| {
                    temp.set_bus_gain(bus_index, clamped);
                }));
            }
            BackendCommand::ToggleStripMuted { strip_index } => {
                let muted = session
                    .strips
                    .get(strip_index)
                    .map(|strip| !strip.muted)
                    .unwrap_or(false);
                events.push(BackendEvent::StripMutedChanged { strip_index, muted });
                let mut temp = session.clone();
                temp.set_strip_muted(strip_index, muted);
                self.sync_all_strip_runtime_states(&temp);
                events.push(self.next_meter_event_with_update(session, |temp| {
                    temp.set_strip_muted(strip_index, muted);
                }));
            }
            BackendCommand::ToggleStripSolo { strip_index } => {
                let solo = session
                    .strips
                    .get(strip_index)
                    .map(|strip| !strip.solo)
                    .unwrap_or(false);
                events.push(BackendEvent::StripSoloChanged { strip_index, solo });
                let mut temp = session.clone();
                temp.set_strip_solo(strip_index, solo);
                self.sync_all_strip_runtime_states(&temp);
                events.push(self.next_meter_event_with_update(session, |temp| {
                    temp.set_strip_solo(strip_index, solo);
                }));
            }
            BackendCommand::ToggleStripMono { strip_index } => {
                let mono = session
                    .strips
                    .get(strip_index)
                    .map(|strip| !strip.mono)
                    .unwrap_or(false);
                events.push(BackendEvent::StripMonoChanged { strip_index, mono });
                let mut temp = session.clone();
                temp.set_strip_mono(strip_index, mono);
                events.extend(self.sync_strip_routes(strip_index, &temp));
                events.push(self.next_meter_event_with_update(session, |temp| {
                    temp.set_strip_mono(strip_index, mono);
                }));
            }
            BackendCommand::ToggleBusMuted { bus_index } => {
                let muted = session
                    .buses
                    .get(bus_index)
                    .map(|bus| !bus.muted)
                    .unwrap_or(false);
                events.push(BackendEvent::BusMutedChanged { bus_index, muted });
                let mut temp = session.clone();
                temp.set_bus_muted(bus_index, muted);
                self.sync_bus_runtime_state(bus_index, &temp);
                events.push(self.next_meter_event_with_update(session, |temp| {
                    temp.set_bus_muted(bus_index, muted);
                }));
            }
            BackendCommand::SetStripDevice {
                strip_index,
                device_name,
            } => {
                events.push(BackendEvent::StripDeviceChanged {
                    strip_index,
                    device_name: device_name.clone(),
                });
                let mut temp = session.clone();
                temp.set_strip_device(strip_index, &device_name);
                self.sync_strip_runtime_state(strip_index, &temp);
                events.extend(self.sync_strip_routes(strip_index, &temp));
                events.push(self.next_meter_event(session));
            }
            BackendCommand::SetBusDevice { bus_index, device_name } => {
                let binding_events = self.bind_hardware_bus(bus_index, &device_name);
                let resolved_name = binding_events.iter().find_map(|event| match event {
                    BackendEvent::BusBindingChanged {
                        bus_index: changed_index,
                        device_name,
                    } if *changed_index == bus_index => Some(device_name.clone()),
                    _ => None,
                });
                events.extend(binding_events);
                match resolved_name {
                    Some(device_name) => {
                        let mut temp = session.clone();
                        temp.set_bus_device(bus_index, &device_name);
                        self.sync_bus_runtime_state(bus_index, &temp);
                        events.extend(self.sync_bus_routes(bus_index, &temp));
                    }
                    None if bus_index < 3 && device_name.is_empty() => {
                        let mut temp = session.clone();
                        temp.set_bus_device(bus_index, "");
                        events.extend(self.sync_bus_routes(bus_index, &temp));
                    }
                    _ => {}
                }
                events.push(self.next_meter_event(session));
            }
            BackendCommand::SetVirtualAppLevel {
                strip_index,
                app_index,
                level,
            } => {
                let clamped = level.clamp(0.0, 1.0);
                events.push(BackendEvent::VirtualAppLevelChanged {
                    strip_index,
                    app_index,
                    level: clamped,
                });
                events.push(self.next_meter_event_with_update(session, |temp| {
                    temp.set_virtual_app_level(strip_index, app_index, clamped);
                }));
            }
            BackendCommand::ToggleVirtualAppMuted {
                strip_index,
                app_index,
            } => {
                let muted = session
                    .strips
                    .get(strip_index)
                    .and_then(|strip| strip.apps.get(app_index))
                    .map(|app| !app.muted)
                    .unwrap_or(false);
                events.push(BackendEvent::VirtualAppMutedChanged {
                    strip_index,
                    app_index,
                    muted,
                });
                events.push(self.next_meter_event_with_update(session, |temp| {
                    temp.set_virtual_app_muted(strip_index, app_index, muted);
                }));
            }
            BackendCommand::SetAppRoute { app_id, target } => {
                if let Err(message) = move_sink_input_to_target(app_id, target.as_str()) {
                    events.push(BackendEvent::BackendError { message });
                }
                events.extend(self.refresh_app_routes());
            }
            BackendCommand::SetAppLevel { app_id, level } => {
                if let Err(message) = set_sink_input_volume(app_id, level.clamp(0.0, 1.0)) {
                    events.push(BackendEvent::BackendError { message });
                }
                events.extend(self.refresh_app_routes());
            }
            BackendCommand::ToggleAppMuted { app_id } => {
                match find_sink_input_muted(app_id) {
                    Ok(muted) => {
                        if let Err(message) = set_sink_input_mute(app_id, !muted) {
                            events.push(BackendEvent::BackendError { message });
                        }
                    }
                    Err(message) => events.push(BackendEvent::BackendError { message }),
                }
                events.extend(self.refresh_app_routes());
            }
        }

        if devices_changed && !is_start && !is_shutdown {
            events.extend(self.ensure_session_bus_bindings(session));
            self.sync_all_runtime_mix_state(session);
            events.extend(self.sync_all_routes(session));
        }

        events
    }
}

impl PipeWireDiscoveryBackend {
    fn refresh_device_events_if_stale(&mut self, max_age: Duration) -> Vec<BackendEvent> {
        if self
            .last_device_refresh
            .is_some_and(|instant| instant.elapsed() < max_age)
        {
            return Vec::new();
        }
        self.refresh_device_events()
    }

    fn ensure_virtual_endpoints(&mut self) -> Vec<BackendEvent> {
        self.destroy_virtual_endpoints();

        for spec in VIRTUAL_INPUT_ENDPOINTS {
            if let Err(message) = create_virtual_endpoint(spec) {
                return vec![
                    BackendEvent::VirtualEndpointsReady { ready: false },
                    BackendEvent::BackendError { message },
                ];
            }
        }

        let mut module_ids = Vec::new();
        for spec in VIRTUAL_BUS_ENDPOINTS {
            match create_virtual_bus_endpoint(spec) {
                Ok(ids) => module_ids.extend(ids),
                Err(message) => {
                    unload_modules(module_ids.iter().copied());
                    self.virtual_bus_module_ids.clear();
                    return vec![
                        BackendEvent::VirtualEndpointsReady { ready: false },
                        BackendEvent::BackendError { message },
                    ];
                }
            }
        }
        self.virtual_bus_module_ids = module_ids;

        match list_virtual_endpoint_nodes() {
            Ok(nodes) if has_complete_virtual_layout(&nodes) => {
                self.virtual_endpoint_ids = nodes.iter().map(|node| node.id).collect();
                vec![BackendEvent::VirtualEndpointsReady { ready: true }]
            }
            Ok(nodes) => {
                self.virtual_endpoint_ids = nodes.iter().map(|node| node.id).collect();
                vec![
                    BackendEvent::VirtualEndpointsReady { ready: false },
                    BackendEvent::BackendError {
                        message: "virtual endpoints were created incompletely".into(),
                    },
                ]
            }
            Err(message) => vec![
                BackendEvent::VirtualEndpointsReady { ready: false },
                BackendEvent::BackendError { message },
            ],
        }
    }

    fn destroy_virtual_endpoints(&mut self) {
        unload_modules(self.virtual_bus_module_ids.iter().copied());
        self.virtual_bus_module_ids.clear();

        if let Ok(modules) = list_virtual_bus_modules() {
            unload_modules(modules.iter().map(|module| module.id));
        }

        if !self.virtual_endpoint_ids.is_empty() {
            destroy_nodes(self.virtual_endpoint_ids.iter().copied());
            self.virtual_endpoint_ids.clear();
        }

        if let Ok(nodes) = list_virtual_endpoint_nodes() {
            destroy_nodes(nodes.iter().map(|node| node.id));
        }
    }

    fn next_meter_event(&mut self, session: &SessionState) -> BackendEvent {
        self.sync_strip_meter_taps(session);
        BackendEvent::MetersUpdated {
            snapshot: self.build_meter_snapshot(session),
        }
    }

    fn next_meter_event_with_update<F>(&mut self, session: &SessionState, update: F) -> BackendEvent
    where
        F: FnOnce(&mut SessionState),
    {
        let mut temp = session.clone();
        update(&mut temp);
        self.next_meter_event(&temp)
    }

    fn refresh_device_events(&mut self) -> Vec<BackendEvent> {
        self.last_device_refresh = Some(Instant::now());
        match discover_device_nodes() {
            Ok((input_nodes, output_nodes)) => {
                let input_devices: Vec<_> = input_nodes
                    .iter()
                    .map(PipeWireDeviceNode::to_option)
                    .collect();
                let output_devices: Vec<_> = output_nodes
                    .iter()
                    .map(PipeWireDeviceNode::to_option)
                    .collect();
                let changed = input_devices != self.known_input_devices
                    || output_devices != self.known_output_devices
                    || !self.connected;

                self.known_input_nodes = input_nodes;
                self.known_input_devices = input_devices.clone();
                self.known_output_devices = output_devices.clone();
                self.known_output_nodes = output_nodes;
                self.reconcile_bus_bindings();
                self.reconcile_active_routes();
                self.connected = true;

                if changed {
                    vec![
                        BackendEvent::ConnectionStateChanged { connected: true },
                        BackendEvent::DevicesUpdated {
                            input_devices,
                            output_devices,
                        },
                    ]
                } else {
                    Vec::new()
                }
            }
            Err(message) => {
                let mut events = Vec::new();
                if self.connected {
                    events.push(BackendEvent::ConnectionStateChanged { connected: false });
                }
                self.connected = false;
                events.push(BackendEvent::BackendError { message });
                events
            }
        }
    }

    fn sync_strip_meter_taps(&mut self, session: &SessionState) {
        let desired_targets = session
            .strips
            .iter()
            .enumerate()
            .map(|(strip_index, _)| (strip_index, self.resolve_strip_meter_target(strip_index, session)))
            .collect::<Vec<_>>();

        let mut retained = Vec::with_capacity(self.strip_meter_taps.len());
        for mut tap in std::mem::take(&mut self.strip_meter_taps) {
            let desired = desired_targets
                .iter()
                .find(|(strip_index, _)| *strip_index == tap.strip_index)
                .and_then(|(_, target)| target.as_deref());
            if desired == Some(tap.target_name.as_str()) {
                retained.push(tap);
            } else {
                tap.stop();
            }
        }

        for (strip_index, target_name) in desired_targets {
            let Some(target_name) = target_name else {
                continue;
            };
            if retained
                .iter()
                .any(|tap| tap.strip_index == strip_index && tap.target_name == target_name)
            {
                continue;
            }
            retained.push(StripMeterTap::spawn(strip_index, target_name));
        }

        retained.sort_by_key(|tap| tap.strip_index);
        self.strip_meter_taps = retained;
    }

    fn stop_all_meter_taps(&mut self) {
        for mut tap in std::mem::take(&mut self.strip_meter_taps) {
            tap.stop();
        }
    }

    fn resolve_strip_meter_target(
        &self,
        strip_index: usize,
        session: &SessionState,
    ) -> Option<String> {
        let strip = session.strips.get(strip_index)?;
        match strip.endpoint_kind {
            EndpointKind::HardwareInput => self
                .resolve_input_node(&strip.device_name)
                .map(|node| node.node_name.clone()),
            EndpointKind::VirtualInput => match strip_index {
                3 => Some("voicewire.in1.monitor".to_string()),
                4 => Some("voicewire.in2.monitor".to_string()),
                5 => Some("voicewire.in3.monitor".to_string()),
                _ => None,
            },
            _ => None,
        }
    }

    fn build_meter_snapshot(&self, session: &SessionState) -> MeterSnapshot {
        let strip_meters = session
            .strips
            .iter()
            .enumerate()
            .map(|(strip_index, strip)| {
                let mut levels = self.read_strip_levels(strip_index);

                if !self.strip_is_audible(strip_index, session) {
                    levels = StereoLevels { left: 0.0, right: 0.0 };
                }

                if strip.mono {
                    let mono = ((levels.left + levels.right) * 0.5).clamp(0.0, 1.0);
                    levels.left = mono;
                    levels.right = mono;
                }

                levels.left = meter_curve(levels.left);
                levels.right = meter_curve(levels.right);
                levels
            })
            .collect::<Vec<_>>();

        let app_meters = session
            .strips
            .iter()
            .enumerate()
            .map(|(strip_index, strip)| {
                let strip_peak = strip_meters
                    .get(strip_index)
                    .map(|levels| levels.left.max(levels.right))
                    .unwrap_or(0.0);
                strip.apps
                    .iter()
                    .map(|app| {
                        if strip.muted || app.muted {
                            0.0
                        } else {
                            (strip_peak * app.level).clamp(0.0, 1.0)
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let bus_meters = session
            .buses
            .iter()
            .enumerate()
            .map(|(bus_index, bus)| {
                if bus.muted {
                    return StereoLevels {
                        left: 0.0,
                        right: 0.0,
                    };
                }

                let mut left = 0.0;
                let mut right = 0.0;
                for (strip, levels) in session.strips.iter().zip(strip_meters.iter()) {
                    if strip
                        .routes
                        .get(bus_index)
                        .is_some_and(|route| route.enabled)
                    {
                        let strip_gain = db_to_meter_scale(strip.gain_db, 0.18);
                        left += levels.left * strip_gain;
                        right += levels.right * strip_gain;
                    }
                }

                let master_scale = db_to_meter_scale(bus.gain_db, 0.2);
                StereoLevels {
                    left: meter_curve((left * 0.58 * master_scale).clamp(0.0, 1.0)),
                    right: meter_curve((right * 0.58 * master_scale).clamp(0.0, 1.0)),
                }
            })
            .collect::<Vec<_>>();

        MeterSnapshot {
            strip_meters,
            bus_meters,
            app_meters,
        }
    }

    fn read_strip_levels(&self, strip_index: usize) -> StereoLevels {
        self.strip_meter_taps
            .iter()
            .find(|tap| tap.strip_index == strip_index)
            .map(StripMeterTap::levels)
            .unwrap_or(StereoLevels {
                left: 0.0,
                right: 0.0,
            })
    }

    fn sync_all_runtime_mix_state(&self, session: &SessionState) {
        self.sync_all_strip_runtime_states(session);
        self.sync_all_bus_runtime_states(session);
    }

    fn sync_all_strip_runtime_states(&self, session: &SessionState) {
        for strip_index in 0..session.strips.len() {
            self.sync_strip_runtime_state(strip_index, session);
        }
    }

    fn sync_all_bus_runtime_states(&self, session: &SessionState) {
        for bus_index in 0..session.buses.len() {
            self.sync_bus_runtime_state(bus_index, session);
        }
    }

    fn sync_strip_runtime_state(&self, strip_index: usize, session: &SessionState) {
        let Some(strip) = session.strips.get(strip_index) else {
            return;
        };
        let targets = self.resolve_strip_volume_targets(strip_index, session);
        if targets.is_empty() {
            return;
        }
        let volume = db_to_pulse_volume(&strip.gain_db);
        let audible = self.strip_is_audible(strip_index, session);
        for (kind, target_name) in targets {
            set_pulse_node_volume(kind, target_name.as_str(), volume.as_str());
            set_pulse_node_mute(kind, target_name.as_str(), !audible);
        }
    }

    fn sync_bus_runtime_state(&self, bus_index: usize, session: &SessionState) {
        let Some(bus) = session.buses.get(bus_index) else {
            return;
        };
        let Some(target_name) = self.resolve_bus_volume_target(bus_index) else {
            return;
        };
        set_pulse_node_volume("sink", target_name, db_to_pulse_volume(&bus.gain_db).as_str());
        set_pulse_node_mute("sink", target_name, bus.muted);
    }

    fn resolve_strip_volume_targets(
        &self,
        strip_index: usize,
        session: &SessionState,
    ) -> Vec<(&'static str, String)> {
        let Some(strip) = session.strips.get(strip_index) else {
            return Vec::new();
        };
        match strip.endpoint_kind {
            EndpointKind::HardwareInput => self
                .resolve_input_node(&strip.device_name)
                .map(|node| vec![("source", node.node_name.clone())])
                .unwrap_or_default(),
            EndpointKind::VirtualInput => match strip_index {
                3 => vec![
                    ("sink", "voicewire.in1".to_string()),
                    ("source", "voicewire.in1.monitor".to_string()),
                ],
                4 => vec![
                    ("sink", "voicewire.in2".to_string()),
                    ("source", "voicewire.in2.monitor".to_string()),
                ],
                5 => vec![
                    ("sink", "voicewire.in3".to_string()),
                    ("source", "voicewire.in3.monitor".to_string()),
                ],
                _ => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

    fn resolve_bus_volume_target(&self, bus_index: usize) -> Option<&str> {
        if bus_index < 3 {
            self.bus_bindings
                .iter()
                .find(|binding| binding.bus_index == bus_index)
                .map(|binding| binding.node_name.as_str())
        } else {
            match bus_index {
                3 => Some("voicewire.internal.b1"),
                4 => Some("voicewire.internal.b2"),
                5 => Some("voicewire.internal.b3"),
                _ => None,
            }
        }
    }

    fn strip_is_audible(&self, strip_index: usize, session: &SessionState) -> bool {
        let Some(strip) = session.strips.get(strip_index) else {
            return false;
        };
        if strip.muted {
            return false;
        }

        let section_solo_active = session
            .strips
            .iter()
            .any(|candidate| {
                candidate.endpoint_kind == strip.endpoint_kind
                    && candidate.solo
                    && !candidate.muted
            });
        if section_solo_active && !strip.solo {
            return false;
        }

        true
    }

    fn ensure_session_bus_bindings(&mut self, session: &SessionState) -> Vec<BackendEvent> {
        let mut events = Vec::new();
        for (bus_index, bus) in session.buses.iter().enumerate().take(3) {
            if bus.destination_name.is_empty() {
                continue;
            }
            let already_bound = self
                .bus_bindings
                .iter()
                .any(|binding| binding.bus_index == bus_index);
            if already_bound {
                continue;
            }
            if self.resolve_output_node(&bus.destination_name).is_some() {
                events.extend(self.bind_hardware_bus(bus_index, &bus.destination_name));
            }
        }
        events
    }

    fn resolve_output_node(&self, device_name: &str) -> Option<&PipeWireDeviceNode> {
        self.known_output_nodes
            .iter()
            .find(|node| node.name == device_name)
    }

    fn resolve_input_node(&self, device_name: &str) -> Option<&PipeWireDeviceNode> {
        self.known_input_nodes
            .iter()
            .find(|node| node.name == device_name)
    }

    fn bind_hardware_bus(&mut self, bus_index: usize, device_name: &str) -> Vec<BackendEvent> {
        if bus_index >= 3 {
            return vec![BackendEvent::BusBindingChanged {
                bus_index,
                device_name: device_name.to_string(),
            }];
        }

        if device_name.is_empty() {
            self.bus_bindings.retain(|binding| binding.bus_index != bus_index);
            return vec![BackendEvent::BusBindingChanged {
                bus_index,
                device_name: String::new(),
            }];
        }

        let Some(node) = self.resolve_output_node(device_name).cloned() else {
            return vec![BackendEvent::BackendError {
                message: format!("could not bind {} to unavailable device \"{}\"", bus_label(bus_index), device_name),
            }];
        };

        if self
            .bus_bindings
            .iter()
            .any(|binding| binding.bus_index == bus_index && binding.node_id == node.id)
        {
            return vec![BackendEvent::BusBindingChanged {
                bus_index,
                device_name: node.name,
            }];
        }

        self.bus_bindings.retain(|binding| binding.bus_index != bus_index);
        self.bus_bindings.push(BusBinding {
            bus_index,
            stable_id: node.stable_id.clone(),
            node_name: node.node_name.clone(),
            node_id: node.id,
        });

        vec![BackendEvent::BusBindingChanged {
            bus_index,
            device_name: node.name,
        }]
    }

    fn reconcile_bus_bindings(&mut self) {
        let mut rebound = Vec::with_capacity(self.bus_bindings.len());
        for binding in self.bus_bindings.drain(..) {
            if let Some(node) = self
                .known_output_nodes
                .iter()
                .find(|node| node.stable_id == binding.stable_id || node.node_name == binding.node_name)
            {
                rebound.push(BusBinding {
                    bus_index: binding.bus_index,
                    stable_id: node.stable_id.clone(),
                    node_name: node.node_name.clone(),
                    node_id: node.id,
                });
            }
        }
        self.bus_bindings = rebound;
    }

    fn reconcile_active_routes(&mut self) {
        let bound_buses = self
            .bus_bindings
            .iter()
            .map(|binding| binding.bus_index)
            .collect::<Vec<_>>();
        self.active_routes.retain(|route| {
            if route.bus_index < 3 {
                bound_buses.contains(&route.bus_index)
            } else {
                true
            }
        });
    }

    fn sync_all_routes(&mut self, session: &SessionState) -> Vec<BackendEvent> {
        let mut events = Vec::new();
        for bus_index in 0..session.buses.len() {
            events.extend(self.sync_bus_routes(bus_index, session));
        }
        events
    }

    fn sync_bus_routes(&mut self, bus_index: usize, session: &SessionState) -> Vec<BackendEvent> {
        let mut events = Vec::new();
        for strip_index in 0..session.strips.len() {
            events.extend(self.sync_route(strip_index, bus_index, session));
        }
        events
    }

    fn sync_strip_routes(
        &mut self,
        strip_index: usize,
        session: &SessionState,
    ) -> Vec<BackendEvent> {
        let mut events = Vec::new();
        for bus_index in 0..session.buses.len() {
            events.extend(self.sync_route(strip_index, bus_index, session));
        }
        events
    }

    fn sync_route(
        &mut self,
        strip_index: usize,
        bus_index: usize,
        session: &SessionState,
    ) -> Vec<BackendEvent> {
        self.remove_route_links(strip_index, bus_index);

        let Some(strip) = session.strips.get(strip_index) else {
            return Vec::new();
        };
        let Some(route) = strip.routes.get(bus_index) else {
            return Vec::new();
        };
        if !route.enabled {
            return Vec::new();
        }
        if !self.strip_is_audible(strip_index, session) {
            return Vec::new();
        }

        let Some(bus) = session.buses.get(bus_index) else {
            return Vec::new();
        };
        if bus.destination_name.is_empty() || bus.muted {
            return Vec::new();
        }

        let target_node_name = match self.resolve_bus_target_node_name(bus_index) {
            Some(node_name) => node_name,
            None => return Vec::new(),
        };
        let dest_ports = match resolve_input_ports(target_node_name) {
            Ok(ports) => ports,
            Err(message) => return vec![BackendEvent::BackendError { message }],
        };

        let Some((dest_left, dest_right)) = pick_stereo_ports(
            &dest_ports,
            &[("playback_FL", "playback_FR"), ("playback_AUX0", "playback_AUX1")],
        ) else {
            return vec![BackendEvent::BackendError {
                message: format!("{} has no usable playback ports", bus.destination_name),
            }];
        };

        let links = match self.resolve_strip_route_links(
            strip_index,
            session,
            dest_left.as_str(),
            dest_right.as_str(),
        ) {
            Ok(links) => links,
            Err(message) => return vec![BackendEvent::BackendError { message }],
        };

        let mut created_links = Vec::new();
        for link in &links {
            if let Err(message) = create_link(link) {
                self.remove_links(&created_links);
                return vec![BackendEvent::BackendError { message }];
            }
            created_links.push(link.clone());
        }

        self.active_routes.push(ActiveRoute {
            strip_index,
            bus_index,
            links: created_links,
        });

        Vec::new()
    }

    fn remove_route_links(&mut self, strip_index: usize, bus_index: usize) {
        let routes = std::mem::take(&mut self.active_routes);
        let mut retained = Vec::with_capacity(routes.len());
        for route in routes {
            if route.strip_index == strip_index && route.bus_index == bus_index {
                self.remove_links(&route.links);
            } else {
                retained.push(route);
            }
        }
        self.active_routes = retained;
    }

    fn remove_links(&self, links: &[RouteLink]) {
        for link in links {
            remove_all_matching_links(&link.source_port, &link.dest_port);
        }
    }

    fn clear_all_routes(&mut self) {
        let routes = std::mem::take(&mut self.active_routes);
        for route in routes {
            self.remove_links(&route.links);
        }
    }

    fn resolve_strip_route_links(
        &self,
        strip_index: usize,
        session: &SessionState,
        dest_left: &str,
        dest_right: &str,
    ) -> Result<Vec<RouteLink>, String> {
        let strip = session
            .strips
            .get(strip_index)
            .ok_or_else(|| format!("strip {} was missing", strip_index))?;

        match strip.endpoint_kind {
            EndpointKind::HardwareInput => {
                let node = self
                    .resolve_input_node(&strip.device_name)
                    .ok_or_else(|| format!("{} is not assigned to an available input device", strip.title))?;
                let ports = resolve_output_ports(&node.node_name)?;
                let links = if strip.mono {
                    build_hardware_input_mono_links(&ports, dest_left, dest_right)
                } else {
                    build_hardware_input_links(&ports, dest_left, dest_right)
                };
                links
                    .ok_or_else(|| format!("{} has no usable capture ports", strip.device_name))
            }
            EndpointKind::VirtualInput => {
                let node_name = match strip_index {
                    3 => "voicewire.in1",
                    4 => "voicewire.in2",
                    5 => "voicewire.in3",
                    _ => return Err(format!("unsupported virtual strip index {}", strip_index)),
                };
                let ports = resolve_output_ports(node_name)?;
                let (left, right) = pick_stereo_ports(&ports, &[("monitor_FL", "monitor_FR")])
                    .ok_or_else(|| format!("{} has no usable monitor ports", strip.device_name))?;
                if strip.mono {
                    Ok(vec![
                        RouteLink {
                            source_port: left.clone(),
                            dest_port: dest_left.to_string(),
                        },
                        RouteLink {
                            source_port: left,
                            dest_port: dest_right.to_string(),
                        },
                    ])
                } else {
                    Ok(vec![
                        RouteLink {
                            source_port: left,
                            dest_port: dest_left.to_string(),
                        },
                        RouteLink {
                            source_port: right,
                            dest_port: dest_right.to_string(),
                        },
                    ])
                }
            }
            _ => Err(format!("{} is not a routable strip source", strip.title)),
        }
    }

    fn resolve_bus_target_node_name(&self, bus_index: usize) -> Option<&str> {
        if bus_index < 3 {
            return self
                .bus_bindings
                .iter()
                .find(|binding| binding.bus_index == bus_index)
                .map(|binding| binding.node_name.as_str());
        }

        match bus_index {
            3 => Some("voicewire.internal.b1"),
            4 => Some("voicewire.internal.b2"),
            5 => Some("voicewire.internal.b3"),
            _ => None,
        }
    }

    fn refresh_app_routes(&self) -> Vec<BackendEvent> {
        match list_app_routes() {
            Ok(routes) => vec![BackendEvent::AppRoutesUpdated { routes }],
            Err(message) => vec![BackendEvent::BackendError { message }],
        }
    }
}

impl Drop for PipeWireDiscoveryBackend {
    fn drop(&mut self) {
        self.clear_all_routes();
        self.stop_all_meter_taps();
        self.destroy_virtual_endpoints();
    }
}

impl StripMeterTap {
    fn spawn(strip_index: usize, target_name: String) -> Self {
        let latest_levels = Arc::new(Mutex::new(StereoLevels {
            left: 0.0,
            right: 0.0,
        }));
        let stop = Arc::new(AtomicBool::new(false));
        let child_slot = Arc::new(Mutex::new(None));

        let thread_levels = Arc::clone(&latest_levels);
        let thread_stop = Arc::clone(&stop);
        let thread_child_slot = Arc::clone(&child_slot);
        let thread_target_name = target_name.clone();

        let handle = thread::spawn(move || {
            while !thread_stop.load(Ordering::Relaxed) {
                let mut command = Command::new("parec");
                command.args([
                    "--device",
                    thread_target_name.as_str(),
                    "--raw",
                    "--format",
                    "float32le",
                    "--rate",
                    "48000",
                    "--channels",
                    "2",
                    "--latency-msec",
                    "15",
                    "--process-time-msec",
                    "10",
                ]);
                command.stdout(Stdio::piped());
                command.stderr(Stdio::null());

                let mut child = match command.spawn() {
                    Ok(child) => child,
                    Err(_) => {
                        thread::sleep(Duration::from_millis(250));
                        continue;
                    }
                };

                let Some(mut stdout) = child.stdout.take() else {
                    let _ = child.kill();
                    let _ = child.wait();
                    thread::sleep(Duration::from_millis(100));
                    continue;
                };

                {
                    let mut slot = thread_child_slot
                        .lock()
                        .expect("meter child slot mutex should not be poisoned");
                    *slot = Some(child);
                }

                let mut bytes = [0_u8; 8192];
                while !thread_stop.load(Ordering::Relaxed) {
                    match stdout.read(&mut bytes) {
                        Ok(0) => break,
                        Ok(count) => {
                            let levels = decode_peak_levels(&bytes[..count]);
                            let mut current = thread_levels
                                .lock()
                                .expect("meter level mutex should not be poisoned");
                            current.left = smooth_meter(current.left, levels.left);
                            current.right = smooth_meter(current.right, levels.right);
                        }
                        Err(_) => break,
                    }
                }

                if let Some(mut child) = thread_child_slot
                    .lock()
                    .expect("meter child slot mutex should not be poisoned")
                    .take()
                {
                    let _ = child.kill();
                    let _ = child.wait();
                }

                if let Ok(mut levels) = thread_levels.lock() {
                    levels.left = 0.0;
                    levels.right = 0.0;
                }

                if !thread_stop.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        });

        Self {
            strip_index,
            target_name,
            latest_levels,
            stop,
            child_slot,
            handle: Some(handle),
        }
    }

    fn levels(&self) -> StereoLevels {
        self.latest_levels
            .lock()
            .expect("meter level mutex should not be poisoned")
            .clone()
    }

    fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(mut child) = self
            .child_slot
            .lock()
            .expect("meter child slot mutex should not be poisoned")
            .take()
        {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn discover_device_nodes() -> Result<(Vec<PipeWireDeviceNode>, Vec<PipeWireDeviceNode>), String> {
    let output = Command::new("pw-dump")
        .output()
        .map_err(|error| format!("failed to execute pw-dump: {error}"))?;

    if !output.status.success() {
        return Err(format!("pw-dump exited with status {}", output.status));
    }

    let dump: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("failed to parse pw-dump json: {error}"))?;
    parse_dump_device_nodes(&dump)
}

fn parse_dump_device_nodes(
    dump: &Value,
) -> Result<(Vec<PipeWireDeviceNode>, Vec<PipeWireDeviceNode>), String> {
    let entries = dump
        .as_array()
        .ok_or_else(|| "pw-dump output was not a json array".to_string())?;

    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    for entry in entries {
        if prop(entry, "type") != Some("PipeWire:Interface:Node") {
            continue;
        }

        let media_class = prop_path(entry, &["info", "props", "media.class"]);
        let device_api = prop_path(entry, &["info", "props", "device.api"]);
        let factory = prop_path(entry, &["info", "props", "factory.name"]);
        let node_name = prop_path(entry, &["info", "props", "node.name"]).unwrap_or_default();
        let Some(id) = entry.get("id").and_then(Value::as_u64).map(|id| id as u32) else {
            continue;
        };

        if !matches!(media_class, Some("Audio/Source") | Some("Audio/Sink")) {
            continue;
        }

        if device_api.is_none() && factory.is_none() {
            continue;
        }

        if node_name.ends_with(".monitor") || node_name.contains("monitor") {
            continue;
        }

        if is_voicewire_virtual_node(node_name) {
            continue;
        }

        let description = prop_path(entry, &["info", "props", "node.description"])
            .or_else(|| prop_path(entry, &["info", "props", "node.nick"]))
            .or_else(|| prop_path(entry, &["info", "props", "node.name"]))
            .unwrap_or("Unknown Device");
        let stable_id = prop_path(entry, &["info", "props", "device.serial"])
            .or_else(|| prop_path(entry, &["info", "props", "node.name"]))
            .unwrap_or(description);
        let device_class = match media_class {
            Some("Audio/Source") => "capture",
            Some("Audio/Sink") => "playback",
            _ => "unknown",
        };

        let node = PipeWireDeviceNode {
            id,
            name: description.to_string(),
            stable_id: stable_id.to_string(),
            node_name: node_name.to_string(),
            device_class: device_class.to_string(),
            kind: match media_class {
                Some("Audio/Source") => EndpointKind::HardwareInput,
                Some("Audio/Sink") => EndpointKind::HardwareOutput,
                _ => continue,
            },
        };

        match media_class {
            Some("Audio/Source") => push_unique_node(&mut inputs, node),
            Some("Audio/Sink") => push_unique_node(&mut outputs, node),
            _ => {}
        }
    }

    inputs.sort_by(|a, b| a.name.cmp(&b.name));
    outputs.sort_by(|a, b| a.name.cmp(&b.name));

    Ok((inputs, outputs))
}

fn create_virtual_endpoint(spec: VirtualEndpointSpec) -> Result<(), String> {
    let properties = format!(
        "{{ factory.name = support.null-audio-sink node.name = \"{}\" node.description = \"{}\" media.class = \"{}\" object.linger = true audio.position = [ FL FR ] }}",
        spec.node_name, spec.description, spec.media_class
    );

    let output = Command::new("pw-cli")
        .args(["create-node", "adapter", properties.as_str()])
        .output()
        .map_err(|error| format!("failed to create {}: {error}", spec.description))?;

    if !output.status.success() {
        return Err(format!(
            "pw-cli could not create {}: {}",
            spec.description,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn create_virtual_bus_endpoint(spec: VirtualBusEndpointSpec) -> Result<Vec<u32>, String> {
    let sink_module_id = load_pulse_module(&[
        "load-module",
        "module-null-sink",
        &format!("sink_name={}", spec.sink_node_name),
        &format!(
            "sink_properties=device.description={} media.name={} node.description={}",
            spec.sink_description, spec.sink_description, spec.sink_description
        ),
    ])?;

    let source_module_id = match load_pulse_module(&[
        "load-module",
        "module-remap-source",
        &format!("master={}.monitor", spec.sink_node_name),
        &format!("source_name={}", spec.source_node_name),
        &format!(
            "source_properties=device.description={} media.name={} node.description={}",
            spec.source_description, spec.source_description, spec.source_description
        ),
    ]) {
        Ok(module_id) => module_id,
        Err(message) => {
            unload_modules([sink_module_id]);
            return Err(message);
        }
    };

    set_pulse_node_volume("sink", spec.sink_node_name, "100%");
    set_pulse_node_mute("sink", spec.sink_node_name, false);
    set_pulse_node_volume("source", spec.source_node_name, "100%");
    set_pulse_node_mute("source", spec.source_node_name, false);

    Ok(vec![sink_module_id, source_module_id])
}

fn list_virtual_endpoint_nodes() -> Result<Vec<VirtualEndpointNode>, String> {
    let output = Command::new("pw-dump")
        .output()
        .map_err(|error| format!("failed to execute pw-dump: {error}"))?;

    if !output.status.success() {
        return Err(format!("pw-dump exited with status {}", output.status));
    }

    let dump: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("failed to parse pw-dump json: {error}"))?;
    let entries = dump
        .as_array()
        .ok_or_else(|| "pw-dump output was not a json array".to_string())?;

    let mut nodes = Vec::new();
    for entry in entries {
        if prop(entry, "type") != Some("PipeWire:Interface:Node") {
            continue;
        }

        let node_name = match prop_path(entry, &["info", "props", "node.name"]) {
            Some(name) if is_managed_virtual_node(name) => name,
            _ => continue,
        };

        let media_class = prop_path(entry, &["info", "props", "media.class"]).unwrap_or_default();
        let id = entry
            .get("id")
            .and_then(Value::as_u64)
            .ok_or_else(|| "virtual endpoint node missing id".to_string())? as u32;

        nodes.push(VirtualEndpointNode {
            id,
            name: node_name.to_string(),
            media_class: media_class.to_string(),
        });
    }

    Ok(nodes)
}

fn destroy_nodes<I>(ids: I)
where
    I: IntoIterator<Item = u32>,
{
    for id in ids {
        let _ = Command::new("pw-cli")
            .args(["destroy", &id.to_string()])
            .output();
    }
}

fn has_complete_virtual_layout(nodes: &[VirtualEndpointNode]) -> bool {
    VIRTUAL_INPUT_ENDPOINTS.iter().all(|spec| {
        nodes.iter().any(|node| {
            node.name == spec.node_name && node.media_class == spec.media_class
        })
    }) && VIRTUAL_BUS_ENDPOINTS.iter().all(|spec| {
        nodes.iter().any(|node| node.name == spec.sink_node_name && node.media_class == "Audio/Sink")
            && nodes
                .iter()
                .any(|node| node.name == spec.source_node_name && node.media_class == "Audio/Source")
    })
}

fn is_voicewire_virtual_node(node_name: &str) -> bool {
    node_name.starts_with("voicewire.")
}

fn is_managed_virtual_node(node_name: &str) -> bool {
    VIRTUAL_INPUT_ENDPOINTS
        .iter()
        .any(|spec| spec.node_name == node_name)
        || VIRTUAL_BUS_ENDPOINTS.iter().any(|spec| {
            spec.sink_node_name == node_name || spec.source_node_name == node_name
        })
        || matches!(node_name, "voicewire.b1" | "voicewire.b2" | "voicewire.b3")
}

fn list_virtual_bus_modules() -> Result<Vec<PulseModuleRecord>, String> {
    let output = Command::new("pactl")
        .args(["list", "short", "modules"])
        .output()
        .map_err(|error| format!("failed to list Pulse modules: {error}"))?;

    if !output.status.success() {
        return Err(format!("pactl exited with status {}", output.status));
    }

    let mut modules = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut parts = line.splitn(3, '\t');
        let Some(id) = parts.next().and_then(|value| value.parse().ok()) else {
            continue;
        };
        let Some(_name) = parts.next() else {
            continue;
        };
        let args = parts.next().unwrap_or_default();

        if VIRTUAL_BUS_ENDPOINTS.iter().any(|spec| {
            args.contains(&format!("sink_name={}", spec.sink_node_name))
                || args.contains(&format!("source_name={}", spec.source_node_name))
        }) {
            modules.push(PulseModuleRecord {
                id,
            });
        }
    }

    Ok(modules)
}

fn load_pulse_module(args: &[&str]) -> Result<u32, String> {
    let output = Command::new("pactl")
        .args(args)
        .output()
        .map_err(|error| format!("failed to execute pactl {}: {error}", args.join(" ")))?;

    if !output.status.success() {
        return Err(format!(
            "pactl {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .map_err(|error| format!("could not parse pactl module id: {error}"))
}

fn unload_modules<I>(ids: I)
where
    I: IntoIterator<Item = u32>,
{
    for id in ids {
        let _ = Command::new("pactl")
            .args(["unload-module", &id.to_string()])
            .output();
    }
}

fn set_pulse_node_volume(kind: &str, name: &str, volume: &str) {
    let command = match kind {
        "sink" => "set-sink-volume",
        "source" => "set-source-volume",
        _ => return,
    };

    let _ = Command::new("pactl")
        .args([command, name, volume])
        .output();
}

fn set_pulse_node_mute(kind: &str, name: &str, muted: bool) {
    let command = match kind {
        "sink" => "set-sink-mute",
        "source" => "set-source-mute",
        _ => return,
    };

    let value = if muted { "1" } else { "0" };
    let _ = Command::new("pactl")
        .args([command, name, value])
        .output();
}

fn list_app_routes() -> Result<Vec<AppRouteState>, String> {
    let sink_index_by_name = list_sink_index_by_name()?;
    let output = Command::new("pactl")
        .args(["--format=json", "list", "sink-inputs"])
        .output()
        .map_err(|error| format!("failed to list Pulse sink inputs: {error}"))?;

    if !output.status.success() {
        return Err(format!("pactl sink-input listing failed with status {}", output.status));
    }

    let inputs: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("failed to parse Pulse sink-input json: {error}"))?;
    let entries = inputs
        .as_array()
        .ok_or_else(|| "Pulse sink-input output was not a json array".to_string())?;

    let mut routes = Vec::new();
    for entry in entries {
        let Some(id) = entry.get("index").and_then(Value::as_u64).map(|value| value as u32) else {
            continue;
        };

        let sink_name = entry
            .get("sink")
            .and_then(Value::as_u64)
            .and_then(|sink_index| sink_index_by_name.iter().find_map(|(name, index)| {
                (*index == sink_index as u32).then(|| name.clone())
            }))
            .unwrap_or_default();

        let props = entry
            .get("properties")
            .and_then(Value::as_object);
        let level = sink_input_volume_level(entry);
        let muted = entry
            .get("mute")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let app_name = props
            .and_then(|props| props.get("application.name"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let media_name = props
            .and_then(|props| props.get("media.name"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let binary_name = props
            .and_then(|props| props.get("application.process.binary"))
            .and_then(Value::as_str)
            .unwrap_or("");

        if matches!(binary_name, "voicewire" | "pw-cli" | "pactl" | "parec") {
            continue;
        }

        let name = display_app_name(app_name, media_name, binary_name, id);
        let detail = display_app_detail(app_name, media_name, binary_name, &name);
        let (icon_text, icon_color) = app_badge(name.as_str(), app_name, binary_name);

        routes.push(AppRouteState {
            id,
            name,
            detail,
            target: sink_name_to_route_label(sink_name.as_str()).to_string(),
            icon_text,
            icon_color,
            level,
            muted,
        });
    }

    routes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(routes)
}

fn list_sink_index_by_name() -> Result<Vec<(String, u32)>, String> {
    let output = Command::new("pactl")
        .args(["--format=json", "list", "sinks"])
        .output()
        .map_err(|error| format!("failed to list Pulse sinks: {error}"))?;

    if !output.status.success() {
        return Err(format!("pactl sink listing failed with status {}", output.status));
    }

    let sinks: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("failed to parse Pulse sink json: {error}"))?;
    let entries = sinks
        .as_array()
        .ok_or_else(|| "Pulse sink output was not a json array".to_string())?;

    let mut sink_indexes = Vec::new();
    for entry in entries {
        let Some(index) = entry.get("index").and_then(Value::as_u64).map(|value| value as u32) else {
            continue;
        };
        let Some(name) = entry.get("name").and_then(Value::as_str) else {
            continue;
        };
        sink_indexes.push((name.to_string(), index));
    }

    Ok(sink_indexes)
}

fn move_sink_input_to_target(app_id: u32, target: &str) -> Result<(), String> {
    let sink_name = match target {
        "VAIO" => "voicewire.in1",
        "AUX" => "voicewire.in2",
        "SYS" => "voicewire.in3",
        _ => {
            return Err(format!("unsupported app route target \"{target}\""));
        }
    };

    let output = Command::new("pactl")
        .args(["move-sink-input", &app_id.to_string(), sink_name])
        .output()
        .map_err(|error| format!("failed to move app route: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "pactl move-sink-input failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn sink_name_to_route_label(name: &str) -> &'static str {
    match name {
        "voicewire.in1" => "VAIO",
        "voicewire.in2" => "AUX",
        "voicewire.in3" => "SYS",
        _ => "",
    }
}

fn display_app_name(app_name: &str, media_name: &str, binary_name: &str, id: u32) -> String {
    if let Some(name) = normalized_app_name(app_name, binary_name) {
        return name;
    }

    if let Some(name) = normalized_app_name(media_name, binary_name) {
        return name;
    }

    if !binary_name.is_empty() {
        return humanize_binary_name(binary_name);
    }

    format!("App {id}")
}

fn display_app_detail(app_name: &str, media_name: &str, binary_name: &str, display_name: &str) -> String {
    for candidate in [media_name, app_name, binary_name] {
        let normalized = if candidate == binary_name {
            humanize_binary_name(candidate)
        } else {
            candidate.trim().to_string()
        };
        if !normalized.is_empty() && normalized != display_name && !is_technical_stream_name(candidate) {
            return normalized;
        }
    }
    String::new()
}

fn normalized_app_name(primary: &str, binary_name: &str) -> Option<String> {
    let trimmed = primary.trim();
    if trimmed.is_empty() || is_technical_stream_name(trimmed) {
        return (!binary_name.is_empty()).then(|| humanize_binary_name(binary_name));
    }
    Some(if trimmed.chars().all(|character| !character.is_lowercase()) {
        trimmed.to_string()
    } else if trimmed == trimmed.to_lowercase() {
        humanize_binary_name(trimmed)
    } else {
        trimmed.to_string()
    })
}

fn humanize_binary_name(name: &str) -> String {
    name
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_technical_stream_name(name: &str) -> bool {
    matches!(name, "playStream" | "WEBRTC VoiceEngine")
}

fn sink_input_volume_level(entry: &Value) -> f32 {
    let Some(volume) = entry.get("volume").and_then(Value::as_object) else {
        return 1.0;
    };

    let mut total = 0.0_f32;
    let mut count = 0.0_f32;
    for channel in volume.values() {
        let Some(percent) = channel
            .get("value_percent")
            .and_then(Value::as_str)
            .and_then(|value| value.trim_end_matches('%').parse::<f32>().ok())
        else {
            continue;
        };
        total += (percent / 100.0).clamp(0.0, 2.0);
        count += 1.0;
    }

    if count <= 0.0 {
        1.0
    } else {
        (total / count).clamp(0.0, 1.5)
    }
}

fn app_badge(name: &str, app_name: &str, binary_name: &str) -> (String, String) {
    let base = if !app_name.is_empty() {
        app_name
    } else if !binary_name.is_empty() {
        binary_name
    } else {
        name
    };

    let mut letters = base
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    if letters.is_empty() {
        letters = "--".into();
    }

    let palette = ["#5b463b", "#4c5f79", "#5a6b45", "#6b4b66", "#4f5b4b", "#6a5b3f"];
    let hash = base
        .bytes()
        .fold(0_u32, |accumulator, value| accumulator.wrapping_mul(31).wrapping_add(value as u32));
    let color = palette[(hash as usize) % palette.len()].to_string();

    (letters, color)
}

fn set_sink_input_volume(app_id: u32, level: f32) -> Result<(), String> {
    let output = Command::new("pactl")
        .args([
            "set-sink-input-volume",
            &app_id.to_string(),
            &format!("{:.0}%", (level.clamp(0.0, 1.5) * 100.0)),
        ])
        .output()
        .map_err(|error| format!("failed to set app volume: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "pactl set-sink-input-volume failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn set_sink_input_mute(app_id: u32, muted: bool) -> Result<(), String> {
    let output = Command::new("pactl")
        .args([
            "set-sink-input-mute",
            &app_id.to_string(),
            if muted { "1" } else { "0" },
        ])
        .output()
        .map_err(|error| format!("failed to set app mute: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "pactl set-sink-input-mute failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn find_sink_input_muted(app_id: u32) -> Result<bool, String> {
    let output = Command::new("pactl")
        .args(["--format=json", "list", "sink-inputs"])
        .output()
        .map_err(|error| format!("failed to list Pulse sink inputs: {error}"))?;

    if !output.status.success() {
        return Err(format!("pactl sink-input listing failed with status {}", output.status));
    }

    let inputs: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("failed to parse Pulse sink-input json: {error}"))?;
    let entries = inputs
        .as_array()
        .ok_or_else(|| "Pulse sink-input output was not a json array".to_string())?;

    entries
        .iter()
        .find(|entry| entry.get("index").and_then(Value::as_u64) == Some(app_id as u64))
        .and_then(|entry| entry.get("mute").and_then(Value::as_bool))
        .ok_or_else(|| format!("could not find sink input {app_id}"))
}

fn push_unique_node(list: &mut Vec<PipeWireDeviceNode>, item: PipeWireDeviceNode) {
    if !list.iter().any(|existing| existing.stable_id == item.stable_id) {
        list.push(item);
    }
}

fn bus_label(bus_index: usize) -> &'static str {
    match bus_index {
        0 => "A1",
        1 => "A2",
        2 => "A3",
        3 => "B1",
        4 => "B2",
        5 => "B3",
        _ => "bus",
    }
}

fn smooth_meter(previous: f32, next: f32) -> f32 {
    if next > previous {
        next
    } else {
        (previous * 0.55).max(next)
    }
}

fn meter_curve(level: f32) -> f32 {
    (level.clamp(0.0, 1.0).sqrt() * 1.35).clamp(0.0, 1.0)
}

fn decode_peak_levels(bytes: &[u8]) -> StereoLevels {
    let mut left = 0.0_f32;
    let mut right = 0.0_f32;

    for frame in bytes.chunks_exact(8) {
        let left_sample = f32::from_le_bytes([frame[0], frame[1], frame[2], frame[3]]).abs();
        let right_sample = f32::from_le_bytes([frame[4], frame[5], frame[6], frame[7]]).abs();
        left = left.max(left_sample);
        right = right.max(right_sample);
    }

    StereoLevels {
        left: left.clamp(0.0, 1.0),
        right: right.clamp(0.0, 1.0),
    }
}

fn db_to_meter_scale(gain_db: f32, floor: f32) -> f32 {
    ((gain_db + 60.0) / 72.0).clamp(floor, 1.0)
}

fn db_to_pulse_volume(gain_db: &f32) -> String {
    let linear = 10_f32.powf(gain_db.clamp(-60.0, 12.0) / 20.0);
    format!("{:.1}%", (linear * 100.0).clamp(0.0, 400.0))
}

fn resolve_output_ports(node_name: &str) -> Result<Vec<String>, String> {
    resolve_ports(node_name, true)
}

fn resolve_input_ports(node_name: &str) -> Result<Vec<String>, String> {
    resolve_ports(node_name, false)
}

fn resolve_ports(node_name: &str, output_ports: bool) -> Result<Vec<String>, String> {
    let mut command = Command::new("pw-link");
    command.arg(if output_ports { "-o" } else { "-i" });
    let listing = command
        .output()
        .map_err(|error| format!("failed to list PipeWire ports: {error}"))?;

    if !listing.status.success() {
        return Err(format!("pw-link exited with status {}", listing.status));
    }

    let prefix = format!("{node_name}:");
    let mut ports = String::from_utf8_lossy(&listing.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with(&prefix))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    ports.sort();
    Ok(ports)
}

fn pick_stereo_ports(
    ports: &[String],
    preferred_pairs: &[(&str, &str)],
) -> Option<(String, String)> {
    for (left_suffix, right_suffix) in preferred_pairs {
        let left = ports.iter().find(|port| port.ends_with(left_suffix));
        let right = ports.iter().find(|port| port.ends_with(right_suffix));
        if let (Some(left), Some(right)) = (left, right) {
            return Some(((*left).clone(), (*right).clone()));
        }
    }

    match ports {
        [single] => Some((single.clone(), single.clone())),
        [left, right, ..] => Some((left.clone(), right.clone())),
        _ => None,
    }
}

fn build_hardware_input_links(
    ports: &[String],
    dest_left: &str,
    dest_right: &str,
) -> Option<Vec<RouteLink>> {
    if let Some((left, right)) = pick_stereo_ports(ports, &[("capture_FL", "capture_FR")]) {
        return Some(vec![
            RouteLink {
                source_port: left,
                dest_port: dest_left.to_string(),
            },
            RouteLink {
                source_port: right,
                dest_port: dest_right.to_string(),
            },
        ]);
    }

    let aux_ports = ports
        .iter()
        .filter_map(|port| {
            let suffix = port.rsplit('_').next()?;
            let index = suffix.strip_prefix("AUX")?.parse::<usize>().ok()?;
            Some((index, port.clone()))
        })
        .collect::<Vec<_>>();

    if !aux_ports.is_empty() {
        let mut links = Vec::new();
        let mut even = 0;
        let mut odd = 0;
        for (index, port) in aux_ports {
            if index % 2 == 0 {
                links.push(RouteLink {
                    source_port: port,
                    dest_port: dest_left.to_string(),
                });
                even += 1;
            } else {
                links.push(RouteLink {
                    source_port: port,
                    dest_port: dest_right.to_string(),
                });
                odd += 1;
            }
        }
        if even == 0 {
            links.extend(links_for_all_ports(ports, dest_left));
        }
        if odd == 0 {
            links.extend(links_for_all_ports(ports, dest_right));
        }
        return Some(links);
    }

    match ports {
        [] => None,
        [single] => Some(vec![
            RouteLink {
                source_port: single.clone(),
                dest_port: dest_left.to_string(),
            },
            RouteLink {
                source_port: single.clone(),
                dest_port: dest_right.to_string(),
            },
        ]),
        _ => Some(vec![
            RouteLink {
                source_port: ports[0].clone(),
                dest_port: dest_left.to_string(),
            },
            RouteLink {
                source_port: ports[1].clone(),
                dest_port: dest_right.to_string(),
            },
        ]),
    }
}

fn build_hardware_input_mono_links(
    ports: &[String],
    dest_left: &str,
    dest_right: &str,
) -> Option<Vec<RouteLink>> {
    if let Some((left, _right)) = pick_stereo_ports(ports, &[("capture_FL", "capture_FR")]) {
        return Some(vec![
            RouteLink {
                source_port: left.clone(),
                dest_port: dest_left.to_string(),
            },
            RouteLink {
                source_port: left,
                dest_port: dest_right.to_string(),
            },
        ]);
    }

    let aux_source = ports
        .iter()
        .find(|port| port.ends_with("AUX0") || port.ends_with("AUX1"))
        .cloned()
        .or_else(|| ports.first().cloned())?;

    Some(vec![
        RouteLink {
            source_port: aux_source.clone(),
            dest_port: dest_left.to_string(),
        },
        RouteLink {
            source_port: aux_source,
            dest_port: dest_right.to_string(),
        },
    ])
}

fn links_for_all_ports(ports: &[String], dest: &str) -> Vec<RouteLink> {
    ports.iter()
        .cloned()
        .map(|source_port| RouteLink {
            source_port,
            dest_port: dest.to_string(),
        })
        .collect()
}

fn create_link(link: &RouteLink) -> Result<(), String> {
    remove_all_matching_links(&link.source_port, &link.dest_port);

    let output = Command::new("pw-link")
        .args([link.source_port.as_str(), link.dest_port.as_str()])
        .output()
        .map_err(|error| {
            format!(
                "failed to link {} -> {}: {error}",
                link.source_port, link.dest_port
            )
        })?;

    if !output.status.success() {
        return Err(format!(
            "could not link {} -> {}: {}",
            link.source_port,
            link.dest_port,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn remove_all_matching_links(source_port: &str, dest_port: &str) {
    if let Ok(link_ids) = find_link_ids(source_port, dest_port) {
        for link_id in link_ids {
            let _ = Command::new("pw-link")
                .args(["-d", &link_id.to_string()])
                .output();
        }
    }
}

fn find_link_ids(source_port: &str, dest_port: &str) -> Result<Vec<u32>, String> {
    let output = Command::new("pw-link")
        .args(["-I", "-l"])
        .output()
        .map_err(|error| format!("failed to inspect PipeWire links: {error}"))?;

    if !output.status.success() {
        return Err(format!("pw-link -I -l exited with status {}", output.status));
    }

    let mut current_port = String::new();
    let mut link_ids = Vec::new();

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let trimmed = line.trim_end();
        if let Some((port_id, port_name)) = parse_port_header(trimmed) {
            let _ = port_id;
            current_port = port_name.to_string();
            continue;
        }

        if let Some((link_id, direction, linked_port)) = parse_link_line(trimmed) {
            let matches = match direction {
                LinkDirection::Output => current_port == source_port && linked_port == dest_port,
                LinkDirection::Input => current_port == dest_port && linked_port == source_port,
            };
            if matches {
                link_ids.push(link_id);
            }
        }
    }

    Ok(link_ids)
}

fn parse_port_header(line: &str) -> Option<(u32, &str)> {
    if line.contains("|->") || line.contains("|<-") {
        return None;
    }

    let mut parts = line.trim().splitn(2, char::is_whitespace);
    let id = parts.next()?.parse().ok()?;
    let name = parts.next()?.trim();
    if name.is_empty() {
        return None;
    }
    Some((id, name))
}

fn parse_link_line(line: &str) -> Option<(u32, LinkDirection, String)> {
    let trimmed = line.trim();
    let mut parts = trimmed.split_whitespace();
    let link_id = parts.next()?.parse().ok()?;
    let arrow = parts.next()?;
    if arrow != "|->" && arrow != "|<-" {
        return None;
    }
    let _linked_port_id = parts.next()?;
    let linked_port = parts.collect::<Vec<_>>().join(" ");
    if linked_port.is_empty() {
        return None;
    }
    Some((
        link_id,
        if arrow == "|->" {
            LinkDirection::Output
        } else {
            LinkDirection::Input
        },
        linked_port,
    ))
}

enum LinkDirection {
    Output,
    Input,
}

fn prop<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key)?.as_str()
}

fn prop_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    current.as_str()
}

#[cfg(test)]
mod tests {
    use super::{parse_dump_device_nodes, PipeWireDeviceNode, PipeWireDiscoveryBackend};
    use crate::backend::BackendEvent;
    use crate::model::EndpointKind;

    #[test]
    fn parses_hardware_input_and_output_nodes_only() {
        let dump = serde_json::json!([
            {
                "id": 10,
                "type": "PipeWire:Interface:Node",
                "info": {
                    "props": {
                        "media.class": "Audio/Sink",
                        "node.name": "alsa_output.usb-real.sink",
                        "node.description": "Real Speakers",
                        "device.api": "alsa",
                        "factory.name": "api.alsa.pcm.sink"
                    }
                }
            },
            {
                "id": 11,
                "type": "PipeWire:Interface:Node",
                "info": {
                    "props": {
                        "media.class": "Audio/Source",
                        "node.name": "alsa_input.usb-real.source",
                        "node.description": "Real Mic",
                        "device.api": "alsa",
                        "factory.name": "api.alsa.pcm.source"
                    }
                }
            },
            {
                "id": 12,
                "type": "PipeWire:Interface:Node",
                "info": {
                    "props": {
                        "media.class": "Stream/Output/Audio",
                        "node.name": "spotify"
                    }
                }
            },
            {
                "id": 13,
                "type": "PipeWire:Interface:Node",
                "info": {
                    "props": {
                        "media.class": "Audio/Source",
                        "node.name": "alsa_output.usb-real.monitor",
                        "node.description": "Monitor of Real Speakers",
                        "device.api": "alsa",
                        "factory.name": "api.alsa.pcm.source"
                    }
                }
            },
            {
                "id": 14,
                "type": "PipeWire:Interface:Node",
                "info": {
                    "props": {
                        "media.class": "Audio/Sink",
                        "node.name": "voicewire.in1",
                        "node.description": "VoiceWire In 1",
                        "factory.name": "support.null-audio-sink"
                    }
                }
            },
            {
                "id": 15,
                "type": "PipeWire:Interface:Node",
                "info": {
                    "props": {
                        "media.class": "Audio/Source",
                        "node.name": "voicewire.b1",
                        "node.description": "VoiceWire B1",
                        "factory.name": "support.null-audio-sink"
                    }
                }
            }
        ]);

        let (inputs, outputs) = parse_dump_device_nodes(&dump).expect("valid dump should parse");

        assert_eq!(inputs.len(), 1);
        assert_eq!(outputs.len(), 1);
        assert_eq!(inputs[0].name, "Real Mic");
        assert_eq!(outputs[0].name, "Real Speakers");
    }

    #[test]
    fn parse_dump_device_nodes_keeps_node_identity_for_output_binding() {
        let dump = serde_json::json!([
            {
                "id": 41,
                "type": "PipeWire:Interface:Node",
                "info": {
                    "props": {
                        "media.class": "Audio/Sink",
                        "node.name": "alsa_output.pci-0000_00_1f.3.analog-stereo",
                        "node.description": "Desk Speakers",
                        "device.serial": "desk-speakers-serial",
                        "device.api": "alsa",
                        "factory.name": "api.alsa.pcm.sink"
                    }
                }
            }
        ]);

        let (_inputs, outputs) =
            super::parse_dump_device_nodes(&dump).expect("valid dump should parse");

        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].id, 41);
        assert_eq!(outputs[0].name, "Desk Speakers");
        assert_eq!(outputs[0].stable_id, "desk-speakers-serial");
        assert_eq!(
            outputs[0].node_name,
            "alsa_output.pci-0000_00_1f.3.analog-stereo"
        );
    }

    #[test]
    fn bind_hardware_bus_resolves_real_output_node() {
        let mut backend = PipeWireDiscoveryBackend::default();
        backend.known_output_nodes = vec![PipeWireDeviceNode {
            id: 41,
            name: "Desk Speakers".into(),
            stable_id: "desk-speakers-serial".into(),
            node_name: "alsa_output.pci-0000_00_1f.3.analog-stereo".into(),
            device_class: "playback".into(),
            kind: EndpointKind::HardwareOutput,
        }];

        let events = backend.bind_hardware_bus(0, "Desk Speakers");

        assert_eq!(events.len(), 1);
        match &events[0] {
            BackendEvent::BusBindingChanged {
                bus_index,
                device_name,
            } => {
                assert_eq!(*bus_index, 0);
                assert_eq!(device_name, "Desk Speakers");
            }
            other => panic!("unexpected event: {:?}", event_name(other)),
        }
        assert_eq!(backend.bus_bindings.len(), 1);
        assert_eq!(backend.bus_bindings[0].node_id, 41);
        assert_eq!(
            backend.bus_bindings[0].node_name,
            "alsa_output.pci-0000_00_1f.3.analog-stereo"
        );
    }

    #[test]
    fn bind_hardware_bus_rejects_unknown_output() {
        let mut backend = PipeWireDiscoveryBackend::default();

        let events = backend.bind_hardware_bus(1, "Missing Device");

        assert_eq!(events.len(), 1);
        match &events[0] {
            BackendEvent::BackendError { message } => {
                assert!(message.contains("A2"));
                assert!(message.contains("Missing Device"));
            }
            other => panic!("unexpected event: {:?}", event_name(other)),
        }
        assert!(backend.bus_bindings.is_empty());
    }

    fn event_name(event: &BackendEvent) -> &'static str {
        match event {
            BackendEvent::DevicesUpdated { .. } => "DevicesUpdated",
            BackendEvent::VirtualEndpointsReady { .. } => "VirtualEndpointsReady",
            BackendEvent::RouteChanged { .. } => "RouteChanged",
            BackendEvent::StripGainChanged { .. } => "StripGainChanged",
            BackendEvent::BusGainChanged { .. } => "BusGainChanged",
            BackendEvent::StripMutedChanged { .. } => "StripMutedChanged",
            BackendEvent::StripSoloChanged { .. } => "StripSoloChanged",
            BackendEvent::StripMonoChanged { .. } => "StripMonoChanged",
            BackendEvent::BusMutedChanged { .. } => "BusMutedChanged",
            BackendEvent::StripDeviceChanged { .. } => "StripDeviceChanged",
            BackendEvent::BusBindingChanged { .. } => "BusBindingChanged",
            BackendEvent::VirtualAppLevelChanged { .. } => "VirtualAppLevelChanged",
            BackendEvent::VirtualAppMutedChanged { .. } => "VirtualAppMutedChanged",
            BackendEvent::MetersUpdated { .. } => "MetersUpdated",
            BackendEvent::BackendError { .. } => "BackendError",
            BackendEvent::ConnectionStateChanged { .. } => "ConnectionStateChanged",
        }
    }
}
