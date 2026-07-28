use serde::Serialize;

#[derive(Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EndpointKind {
    HardwareInput,
    VirtualInput,
    HardwareOutput,
    VirtualOutput,
}

#[derive(Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceOption {
    pub name: String,
    pub kind: EndpointKind,
    pub stable_id: String,
    pub device_class: String,
    pub connected: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteState {
    pub label: String,
    pub enabled: bool,
}

#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppFeedState {
    pub id: u32,
    pub icon_text: String,
    pub icon_color: String,
    pub name: String,
    pub detail: String,
    pub level: f32,
    pub meter: f32,
    pub muted: bool,
}

#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppRouteState {
    pub id: u32,
    pub name: String,
    pub detail: String,
    pub target: String,
    pub icon_text: String,
    pub icon_color: String,
    pub level: f32,
    pub muted: bool,
}

#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RememberedAppRouteState {
    pub name: String,
    pub detail: String,
    pub target: String,
    pub icon_text: String,
    pub icon_color: String,
    pub level: f32,
    pub muted: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StripState {
    pub title: String,
    pub subtitle: String,
    pub device_name: String,
    pub device_options: Vec<DeviceOption>,
    pub header_name: String,
    pub header_detail: String,
    pub gain_db: f32,
    pub gate: f32,
    pub muted: bool,
    pub solo: bool,
    pub mono: bool,
    pub meter_left: f32,
    pub meter_right: f32,
    pub routes: Vec<RouteState>,
    pub endpoint_kind: EndpointKind,
    pub apps: Vec<AppFeedState>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BusState {
    pub title: String,
    pub subtitle: String,
    pub destination_name: String,
    pub device_options: Vec<DeviceOption>,
    pub gain_db: f32,
    pub muted: bool,
    pub meter_left: f32,
    pub meter_right: f32,
    pub endpoint_kind: EndpointKind,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    pub strips: Vec<StripState>,
    pub buses: Vec<BusState>,
    pub app_routes: Vec<AppRouteState>,
    #[serde(skip_serializing)]
    pub remembered_app_routes: Vec<RememberedAppRouteState>,
    pub quick_route_enabled: bool,
    pub backend_connected: bool,
    pub virtual_endpoints_ready: bool,
    pub last_backend_error: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StereoLevels {
    pub left: f32,
    pub right: f32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeterSnapshot {
    pub strip_meters: Vec<StereoLevels>,
    pub bus_meters: Vec<StereoLevels>,
    pub app_meters: Vec<Vec<f32>>,
}

impl SessionState {
    pub fn demo() -> Self {
        let mic_devices = vec![
            demo_device(
                "Scarlett 2i2 Mic 1",
                EndpointKind::HardwareInput,
                "scarlett-2i2-mic-1",
                "capture",
            ),
            demo_device(
                "USB Podcast Mic",
                EndpointKind::HardwareInput,
                "usb-podcast-mic",
                "capture",
            ),
            demo_device(
                "HD Webcam Mic",
                EndpointKind::HardwareInput,
                "hd-webcam-mic",
                "capture",
            ),
        ];
        let virtual_inputs = vec![
            demo_device(
                "VoiceWire VAIO",
                EndpointKind::VirtualInput,
                "voicewire-in-1",
                "virtual-playback",
            ),
            demo_device(
                "VoiceWire AUX",
                EndpointKind::VirtualInput,
                "voicewire-in-2",
                "virtual-playback",
            ),
            demo_device(
                "VoiceWire SYS",
                EndpointKind::VirtualInput,
                "voicewire-in-3",
                "virtual-playback",
            ),
        ];
        let hardware_outputs = vec![
            demo_device(
                "Desk Speakers",
                EndpointKind::HardwareOutput,
                "desk-speakers",
                "playback",
            ),
            demo_device(
                "Studio Headphones",
                EndpointKind::HardwareOutput,
                "studio-headphones",
                "playback",
            ),
            demo_device(
                "USB Monitor",
                EndpointKind::HardwareOutput,
                "usb-monitor",
                "playback",
            ),
        ];

        let strip_specs = [
            (
                "MICROPHONE 1",
                "Hardware Input 1",
                "Scarlett 2i2 Mic 1",
                "Scarlett 2i2",
                "48000 Hz - 512",
                true,
            ),
            (
                "MICROPHONE 2",
                "Hardware Input 2",
                "USB Podcast Mic",
                "USB Podcast Mic",
                "48000 Hz - 512",
                true,
            ),
            (
                "MICROPHONE 3",
                "Hardware Input 3",
                "HD Webcam Mic",
                "HD Webcam Mic",
                "48000 Hz - 512",
                true,
            ),
            (
                "MUSIC",
                "Virtual Input 1",
                "VoiceWire VAIO",
                "VoiceWire VAIO",
                "44100 Hz - 7168",
                false,
            ),
            (
                "DESKTOP",
                "Virtual Input 2",
                "VoiceWire AUX",
                "VoiceWire AUX",
                "44100 Hz - 7168",
                false,
            ),
            (
                "CHAT",
                "Virtual Input 3",
                "VoiceWire SYS",
                "VoiceWire SYS",
                "44100 Hz - 7168",
                false,
            ),
        ];
        let bus_labels = ["A1", "A2", "A3", "B1", "B2", "B3"];

        let strips = strip_specs
            .into_iter()
            .enumerate()
            .map(
                |(
                    index,
                    (title, subtitle, device_name, header_name, header_detail, is_hardware),
                )| {
                    let apps = if is_hardware {
                        Vec::new()
                    } else {
                        match index {
                            3 => vec![
                                demo_app("SP", "#335c44", "Spotify"),
                                demo_app("YT", "#6d463f", "YouTube"),
                                demo_app("VL", "#886b32", "VLC"),
                                demo_app("FF", "#584136", "Firefox"),
                                demo_app("TM", "#42466a", "Tidal"),
                            ],
                            4 => vec![
                                demo_app("FF", "#584136", "Firefox"),
                                demo_app("OB", "#444444", "OBS 23.0.2"),
                                demo_app("ST", "#2d5460", "Steam"),
                                demo_app("VL", "#886b32", "Vivaldi"),
                                demo_app("DC", "#42466a", "Discord Overlay"),
                            ],
                            _ => vec![
                                demo_app("DC", "#42466a", "Discord"),
                                demo_app("TS", "#4b5a3f", "Teamspeak"),
                                demo_app("SL", "#5f433e", "Slack Huddle"),
                            ],
                        }
                    };

                    StripState {
                        title: title.into(),
                        subtitle: subtitle.into(),
                        device_name: device_name.into(),
                        device_options: if is_hardware {
                            mic_devices.clone()
                        } else {
                            virtual_inputs.clone()
                        },
                        header_name: header_name.into(),
                        header_detail: header_detail.into(),
                        gain_db: if index == 0 { -2.0 } else { 0.0 },
                        gate: 0.0,
                        muted: false,
                        solo: false,
                        mono: false,
                        meter_left: 0.0,
                        meter_right: 0.0,
                        routes: bus_labels
                            .iter()
                            .enumerate()
                            .map(|(bus_index, label)| RouteState {
                                label: (*label).into(),
                                enabled: matches!(
                                    (index, bus_index),
                                    (0, 0) | (0, 3) | (3, 0) | (4, 1) | (4, 4) | (5, 0) | (5, 3)
                                ),
                            })
                            .collect(),
                        endpoint_kind: if is_hardware {
                            EndpointKind::HardwareInput
                        } else {
                            EndpointKind::VirtualInput
                        },
                        apps,
                    }
                },
            )
            .collect();

        let bus_specs = [
            ("A1", "Hardware Out", "Desk Speakers", true),
            ("A2", "Hardware Out", "Studio Headphones", true),
            ("A3", "Hardware Out", "USB Monitor", true),
            ("B1", "Virtual Out", "VoiceWire B1", false),
            ("B2", "Virtual Out", "VoiceWire B2", false),
            ("B3", "Virtual Out", "VoiceWire B3", false),
        ];

        let buses = bus_specs
            .into_iter()
            .map(
                |(title, subtitle, destination_name, can_assign_device)| BusState {
                    title: title.into(),
                    subtitle: subtitle.into(),
                    destination_name: destination_name.into(),
                    device_options: if can_assign_device {
                        hardware_outputs.clone()
                    } else {
                        vec![demo_device(
                            destination_name,
                            EndpointKind::VirtualOutput,
                            destination_name.to_lowercase().replace(' ', "-").as_str(),
                            "virtual-capture",
                        )]
                    },
                    gain_db: 0.0,
                    muted: false,
                    meter_left: 0.0,
                    meter_right: 0.0,
                    endpoint_kind: if can_assign_device {
                        EndpointKind::HardwareOutput
                    } else {
                        EndpointKind::VirtualOutput
                    },
                },
            )
            .collect();

        let app_routes = vec![
            AppRouteState {
                id: 1,
                name: "Spotify".into(),
                detail: "spotify".into(),
                target: "VAIO".into(),
                icon_text: "SP".into(),
                icon_color: "#335c44".into(),
                level: 1.0,
                muted: false,
            },
            AppRouteState {
                id: 2,
                name: "Discord".into(),
                detail: "WEBRTC VoiceEngine".into(),
                target: "AUX".into(),
                icon_text: "DC".into(),
                icon_color: "#42466a".into(),
                level: 1.0,
                muted: false,
            },
            AppRouteState {
                id: 3,
                name: "Zen".into(),
                detail: "YouTube".into(),
                target: "SYS".into(),
                icon_text: "ZE".into(),
                icon_color: "#584136".into(),
                level: 1.0,
                muted: false,
            },
        ];

        Self {
            strips,
            buses,
            remembered_app_routes: app_routes
                .iter()
                .map(|route| RememberedAppRouteState {
                    name: route.name.clone(),
                    detail: route.detail.clone(),
                    target: route.target.clone(),
                    icon_text: route.icon_text.clone(),
                    icon_color: route.icon_color.clone(),
                    level: route.level,
                    muted: route.muted,
                })
                .collect(),
            app_routes,
            quick_route_enabled: false,
            backend_connected: false,
            virtual_endpoints_ready: false,
            last_backend_error: None,
        }
    }

    pub fn set_connection_state(&mut self, connected: bool) {
        self.backend_connected = connected;
    }

    pub fn set_virtual_endpoints_ready(&mut self, ready: bool) {
        self.virtual_endpoints_ready = ready;
    }

    pub fn set_backend_error(&mut self, message: impl Into<String>) {
        self.last_backend_error = Some(message.into());
    }

    pub fn clear_backend_error(&mut self) {
        self.last_backend_error = None;
    }

    pub fn set_strip_device_options(&mut self, devices: &[DeviceOption]) {
        for strip in &mut self.strips {
            if strip.endpoint_kind == EndpointKind::HardwareInput {
                strip.device_options = devices.to_vec();
                if !strip.device_name.is_empty()
                    && !strip
                        .device_options
                        .iter()
                        .any(|device| device.name == strip.device_name)
                {
                    strip.device_options.push(DeviceOption {
                        name: strip.device_name.clone(),
                        kind: EndpointKind::HardwareInput,
                        stable_id: strip.device_name.clone(),
                        device_class: "capture".into(),
                        connected: false,
                    });
                }
            }
        }
    }

    pub fn set_bus_device_options(&mut self, devices: &[DeviceOption]) {
        for bus in &mut self.buses {
            if bus.endpoint_kind == EndpointKind::HardwareOutput {
                bus.device_options = devices.to_vec();
                if !bus.destination_name.is_empty()
                    && !bus
                        .device_options
                        .iter()
                        .any(|device| device.name == bus.destination_name)
                {
                    bus.device_options.push(DeviceOption {
                        name: bus.destination_name.clone(),
                        kind: EndpointKind::HardwareOutput,
                        stable_id: bus.destination_name.clone(),
                        device_class: "playback".into(),
                        connected: false,
                    });
                }
            }
        }
    }

    pub fn set_route_enabled(&mut self, strip_index: usize, bus_index: usize, enabled: bool) {
        if let Some(route) = self
            .strips
            .get_mut(strip_index)
            .and_then(|strip| strip.routes.get_mut(bus_index))
        {
            route.enabled = enabled;
        }
    }

    pub fn set_strip_gain(&mut self, strip_index: usize, gain_db: f32) {
        if let Some(strip) = self.strips.get_mut(strip_index) {
            strip.gain_db = gain_db.clamp(-60.0, 12.0);
        }
    }

    pub fn set_strip_gate(&mut self, strip_index: usize, gate: f32) {
        if let Some(strip) = self.strips.get_mut(strip_index) {
            strip.gate = gate.clamp(0.0, 10.0);
        }
    }

    pub fn set_bus_gain(&mut self, bus_index: usize, gain_db: f32) {
        if let Some(bus) = self.buses.get_mut(bus_index) {
            bus.gain_db = gain_db.clamp(-60.0, 12.0);
        }
    }

    pub fn set_strip_muted(&mut self, strip_index: usize, muted: bool) {
        if let Some(strip) = self.strips.get_mut(strip_index) {
            strip.muted = muted;
        }
    }

    pub fn set_strip_solo(&mut self, strip_index: usize, solo: bool) {
        if let Some(strip) = self.strips.get_mut(strip_index) {
            strip.solo = solo;
        }
    }

    pub fn set_strip_mono(&mut self, strip_index: usize, mono: bool) {
        if let Some(strip) = self.strips.get_mut(strip_index) {
            strip.mono = mono;
        }
    }

    pub fn set_bus_muted(&mut self, bus_index: usize, muted: bool) {
        if let Some(bus) = self.buses.get_mut(bus_index) {
            bus.muted = muted;
        }
    }

    pub fn set_strip_device(&mut self, strip_index: usize, device_name: &str) {
        if let Some(strip) = self.strips.get_mut(strip_index) {
            strip.device_name = device_name.into();
        }
    }

    pub fn set_bus_device(&mut self, bus_index: usize, device_name: &str) {
        if let Some(bus) = self.buses.get_mut(bus_index) {
            bus.destination_name = device_name.into();
        }
    }

    pub fn set_app_routes(&mut self, routes: Vec<AppRouteState>) -> bool {
        let previous_routes = self.app_routes.clone();
        let mut normalized_routes = routes;
        for route in &mut normalized_routes {
            if let Some(previous) = previous_routes
                .iter()
                .find(|candidate| candidate.id == route.id)
            {
                if route.name.is_empty() {
                    route.name = previous.name.clone();
                }
                if route.detail.is_empty() {
                    route.detail = previous.detail.clone();
                }
                if route.icon_text.is_empty() {
                    route.icon_text = previous.icon_text.clone();
                }
                if route.icon_color.is_empty() {
                    route.icon_color = previous.icon_color.clone();
                }
            }
        }

        let remembered_changed = self.remember_app_routes(&normalized_routes);
        if self.app_routes == normalized_routes {
            return remembered_changed;
        }

        self.app_routes = normalized_routes.clone();

        for strip in self
            .strips
            .iter_mut()
            .filter(|strip| strip.endpoint_kind == EndpointKind::VirtualInput)
        {
            strip.apps.clear();
        }

        for route in normalized_routes {
            let strip_index = match route.target.as_str() {
                "VAIO" => Some(3),
                "AUX" => Some(4),
                "SYS" => Some(5),
                _ => None,
            };
            let Some(strip_index) = strip_index else {
                continue;
            };
            let Some(strip) = self.strips.get_mut(strip_index) else {
                continue;
            };
            strip.apps.push(AppFeedState {
                id: route.id,
                icon_text: route.icon_text,
                icon_color: route.icon_color,
                name: route.name,
                detail: route.detail,
                level: route.level.clamp(0.0, 1.0),
                meter: 0.0,
                muted: route.muted,
            });
        }

        true
    }

    pub fn set_remembered_app_routes(&mut self, routes: Vec<RememberedAppRouteState>) {
        self.remembered_app_routes = routes;
    }

    pub fn set_quick_route_enabled(&mut self, enabled: bool) {
        self.quick_route_enabled = enabled;
    }

    fn remember_app_routes(&mut self, routes: &[AppRouteState]) -> bool {
        let mut changed = false;

        for route in routes {
            let next = RememberedAppRouteState {
                name: route.name.clone(),
                detail: route.detail.clone(),
                target: route.target.clone(),
                icon_text: route.icon_text.clone(),
                icon_color: route.icon_color.clone(),
                level: route.level,
                muted: route.muted,
            };

            let Some(existing) = self.remembered_app_routes.iter_mut().find(|candidate| {
                app_route_identity_key(&candidate.name, &candidate.detail)
                    == app_route_identity_key(&route.name, &route.detail)
            }) else {
                self.remembered_app_routes.push(next);
                changed = true;
                continue;
            };

            if *existing != next {
                *existing = next;
                changed = true;
            }
        }

        changed
    }

    pub fn set_virtual_app_level(&mut self, strip_index: usize, app_index: usize, level: f32) {
        if let Some(app) = self
            .strips
            .get_mut(strip_index)
            .and_then(|strip| strip.apps.get_mut(app_index))
        {
            app.level = level.clamp(0.0, 1.0);
        }
    }

    pub fn set_virtual_app_muted(&mut self, strip_index: usize, app_index: usize, muted: bool) {
        if let Some(app) = self
            .strips
            .get_mut(strip_index)
            .and_then(|strip| strip.apps.get_mut(app_index))
        {
            app.muted = muted;
        }
    }

    pub fn set_meters(&mut self, snapshot: &MeterSnapshot) {
        for (strip, levels) in self.strips.iter_mut().zip(snapshot.strip_meters.iter()) {
            strip.meter_left = levels.left;
            strip.meter_right = levels.right;
        }

        for (bus, levels) in self.buses.iter_mut().zip(snapshot.bus_meters.iter()) {
            bus.meter_left = levels.left;
            bus.meter_right = levels.right;
        }

        for (strip, app_meters) in self.strips.iter_mut().zip(snapshot.app_meters.iter()) {
            for (app, meter) in strip.apps.iter_mut().zip(app_meters.iter()) {
                app.meter = *meter;
            }
        }
    }

    pub fn hardware_input_devices(&self) -> Vec<DeviceOption> {
        self.strips
            .iter()
            .find(|strip| strip.endpoint_kind == EndpointKind::HardwareInput)
            .map(|strip| strip.device_options.clone())
            .unwrap_or_default()
    }

    pub fn hardware_output_devices(&self) -> Vec<DeviceOption> {
        self.buses
            .iter()
            .find(|bus| bus.endpoint_kind == EndpointKind::HardwareOutput)
            .map(|bus| bus.device_options.clone())
            .unwrap_or_default()
    }

    pub fn meter_snapshot(&self, phase: f32) -> MeterSnapshot {
        let solo_active = self.strips.iter().any(|strip| strip.solo && !strip.muted);

        let mut strip_meters = Vec::with_capacity(self.strips.len());
        let mut app_meters = Vec::with_capacity(self.strips.len());

        for (strip_index, strip) in self.strips.iter().enumerate() {
            let route_count = strip.routes.iter().filter(|route| route.enabled).count() as f32;
            let base = if strip.muted || route_count == 0.0 {
                0.0
            } else if solo_active && !strip.solo {
                0.0
            } else {
                let trim = ((strip.gain_db + 60.0) / 72.0).clamp(0.12, 1.0);
                let pulse = (phase + strip_index as f32 * 0.6).sin() * 0.18 + 0.42;
                (trim * pulse * (0.65 + route_count * 0.08)).clamp(0.0, 1.0)
            };

            let left =
                (base * ((phase + strip_index as f32 * 0.8).sin() * 0.12 + 0.88)).clamp(0.0, 1.0);
            let right = (base * ((phase + strip_index as f32 * 0.8 + 0.9).sin() * 0.12 + 0.86))
                .clamp(0.0, 1.0);

            let levels = if strip.mono {
                let mono = ((left + right) * 0.5).clamp(0.0, 1.0);
                StereoLevels {
                    left: mono,
                    right: mono,
                }
            } else {
                StereoLevels { left, right }
            };
            strip_meters.push(levels);

            let strip_app_meters = strip
                .apps
                .iter()
                .enumerate()
                .map(|(app_index, app)| {
                    if strip.muted || app.muted {
                        0.0
                    } else {
                        let flutter = (phase + strip_index as f32 * 0.7 + app_index as f32 * 0.52)
                            .sin()
                            * 0.12
                            + 0.28;
                        (app.level * flutter).clamp(0.0, 1.0)
                    }
                })
                .collect();
            app_meters.push(strip_app_meters);
        }

        let mut bus_meters = Vec::with_capacity(self.buses.len());
        for (bus_index, bus) in self.buses.iter().enumerate() {
            let input_left: f32 = self
                .strips
                .iter()
                .zip(strip_meters.iter())
                .filter(|(strip, _)| {
                    strip
                        .routes
                        .get(bus_index)
                        .is_some_and(|route| route.enabled)
                })
                .map(|(strip, levels)| {
                    levels.left * ((strip.gain_db + 60.0) / 72.0).clamp(0.2, 1.0)
                })
                .sum();
            let input_right: f32 = self
                .strips
                .iter()
                .zip(strip_meters.iter())
                .filter(|(strip, _)| {
                    strip
                        .routes
                        .get(bus_index)
                        .is_some_and(|route| route.enabled)
                })
                .map(|(strip, levels)| {
                    levels.right * ((strip.gain_db + 60.0) / 72.0).clamp(0.2, 1.0)
                })
                .sum();

            if bus.muted {
                bus_meters.push(StereoLevels {
                    left: 0.0,
                    right: 0.0,
                });
            } else {
                let master = ((bus.gain_db + 60.0) / 72.0).clamp(0.18, 1.0);
                bus_meters.push(StereoLevels {
                    left: (input_left * 0.48 * master).clamp(0.0, 1.0),
                    right: (input_right * 0.48 * master).clamp(0.0, 1.0),
                });
            }
        }

        MeterSnapshot {
            strip_meters,
            bus_meters,
            app_meters,
        }
    }

    pub fn current_meter_snapshot(&self) -> MeterSnapshot {
        MeterSnapshot {
            strip_meters: self
                .strips
                .iter()
                .map(|strip| StereoLevels {
                    left: strip.meter_left,
                    right: strip.meter_right,
                })
                .collect(),
            bus_meters: self
                .buses
                .iter()
                .map(|bus| StereoLevels {
                    left: bus.meter_left,
                    right: bus.meter_right,
                })
                .collect(),
            app_meters: self
                .strips
                .iter()
                .map(|strip| strip.apps.iter().map(|app| app.meter).collect())
                .collect(),
        }
    }
}

fn app_route_identity_key(name: &str, detail: &str) -> String {
    format!(
        "{}\u{1f}{}",
        name.trim().to_lowercase(),
        detail.trim().to_lowercase()
    )
}

fn demo_device(
    name: &str,
    kind: EndpointKind,
    stable_id: &str,
    device_class: &str,
) -> DeviceOption {
    DeviceOption {
        name: name.into(),
        kind,
        stable_id: stable_id.into(),
        device_class: device_class.into(),
        connected: true,
    }
}

fn demo_app(icon_text: &str, icon_color: &str, name: &str) -> AppFeedState {
    AppFeedState {
        id: 0,
        icon_text: icon_text.into(),
        icon_color: icon_color.into(),
        name: name.into(),
        detail: String::new(),
        level: 1.0,
        meter: 0.0,
        muted: false,
    }
}
