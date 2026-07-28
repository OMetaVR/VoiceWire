use crate::backend::{AudioBackend, BackendCommand, BackendEvent};
use crate::model::SessionState;

#[allow(dead_code)]
#[derive(Default)]
pub struct MockBackend {
    phase: f32,
}

impl AudioBackend for MockBackend {
    fn handle(&mut self, session: &SessionState, command: BackendCommand) -> Vec<BackendEvent> {
        match command {
            BackendCommand::Start => vec![
                BackendEvent::ConnectionStateChanged { connected: true },
                BackendEvent::VirtualEndpointsReady { ready: true },
                BackendEvent::DevicesUpdated {
                    input_devices: session.hardware_input_devices(),
                    output_devices: session.hardware_output_devices(),
                },
                BackendEvent::AppRoutesUpdated {
                    routes: session.app_routes.clone(),
                },
                self.next_meter_event(session),
            ],
            BackendCommand::PreviewStripGain { .. }
            | BackendCommand::PreviewBusGain { .. }
            | BackendCommand::PreviewAppLevel { .. } => Vec::new(),
            BackendCommand::Shutdown => vec![
                BackendEvent::ConnectionStateChanged { connected: false },
                BackendEvent::VirtualEndpointsReady { ready: false },
            ],
            BackendCommand::RefreshMeters => vec![self.next_meter_event(session)],
            BackendCommand::RefreshAppRoutes => vec![BackendEvent::AppRoutesUpdated {
                routes: session.app_routes.clone(),
            }],
            BackendCommand::ToggleRoute {
                strip_index,
                bus_index,
            } => self.with_temp_session(session, |temp| {
                let enabled = temp
                    .strips
                    .get(strip_index)
                    .and_then(|strip| strip.routes.get(bus_index))
                    .map(|route| !route.enabled)
                    .unwrap_or(false);
                temp.set_route_enabled(strip_index, bus_index, enabled);
                vec![BackendEvent::RouteChanged {
                    strip_index,
                    bus_index,
                    enabled,
                }]
            }),
            BackendCommand::SetStripGain {
                strip_index,
                gain_db,
            } => self.with_temp_session(session, |temp| {
                temp.set_strip_gain(strip_index, gain_db);
                vec![BackendEvent::StripGainChanged {
                    strip_index,
                    gain_db: temp
                        .strips
                        .get(strip_index)
                        .map(|strip| strip.gain_db)
                        .unwrap_or(gain_db),
                }]
            }),
            BackendCommand::SetStripGate { strip_index, gate } => {
                self.with_temp_session(session, |temp| {
                    temp.set_strip_gate(strip_index, gate);
                    vec![BackendEvent::StripGateChanged {
                        strip_index,
                        gate: temp
                            .strips
                            .get(strip_index)
                            .map(|strip| strip.gate)
                            .unwrap_or(gate),
                    }]
                })
            }
            BackendCommand::SetBusGain { bus_index, gain_db } => {
                self.with_temp_session(session, |temp| {
                    temp.set_bus_gain(bus_index, gain_db);
                    vec![BackendEvent::BusGainChanged {
                        bus_index,
                        gain_db: temp
                            .buses
                            .get(bus_index)
                            .map(|bus| bus.gain_db)
                            .unwrap_or(gain_db),
                    }]
                })
            }
            BackendCommand::ToggleStripMuted { strip_index } => {
                self.with_temp_session(session, |temp| {
                    let muted = temp
                        .strips
                        .get(strip_index)
                        .map(|strip| !strip.muted)
                        .unwrap_or(false);
                    temp.set_strip_muted(strip_index, muted);
                    vec![BackendEvent::StripMutedChanged { strip_index, muted }]
                })
            }
            BackendCommand::ToggleStripSolo { strip_index } => {
                self.with_temp_session(session, |temp| {
                    let solo = temp
                        .strips
                        .get(strip_index)
                        .map(|strip| !strip.solo)
                        .unwrap_or(false);
                    temp.set_strip_solo(strip_index, solo);
                    vec![BackendEvent::StripSoloChanged { strip_index, solo }]
                })
            }
            BackendCommand::ToggleStripMono { strip_index } => {
                self.with_temp_session(session, |temp| {
                    let mono = temp
                        .strips
                        .get(strip_index)
                        .map(|strip| !strip.mono)
                        .unwrap_or(false);
                    temp.set_strip_mono(strip_index, mono);
                    vec![BackendEvent::StripMonoChanged { strip_index, mono }]
                })
            }
            BackendCommand::ToggleBusMuted { bus_index } => {
                self.with_temp_session(session, |temp| {
                    let muted = temp
                        .buses
                        .get(bus_index)
                        .map(|bus| !bus.muted)
                        .unwrap_or(false);
                    temp.set_bus_muted(bus_index, muted);
                    vec![BackendEvent::BusMutedChanged { bus_index, muted }]
                })
            }
            BackendCommand::SetStripDevice {
                strip_index,
                device_name,
            } => self.with_temp_session(session, |temp| {
                temp.set_strip_device(strip_index, &device_name);
                vec![BackendEvent::StripDeviceChanged {
                    strip_index,
                    device_name,
                }]
            }),
            BackendCommand::SetBusDevice {
                bus_index,
                device_name,
            } => self.with_temp_session(session, |temp| {
                temp.set_bus_device(bus_index, &device_name);
                vec![BackendEvent::BusBindingChanged {
                    bus_index,
                    device_name,
                }]
            }),
            BackendCommand::SetVirtualAppLevel {
                strip_index,
                app_index,
                level,
            } => self.with_temp_session(session, |temp| {
                temp.set_virtual_app_level(strip_index, app_index, level);
                let level = temp
                    .strips
                    .get(strip_index)
                    .and_then(|strip| strip.apps.get(app_index))
                    .map(|app| app.level)
                    .unwrap_or(level);
                vec![BackendEvent::VirtualAppLevelChanged {
                    strip_index,
                    app_index,
                    level,
                }]
            }),
            BackendCommand::ToggleVirtualAppMuted {
                strip_index,
                app_index,
            } => self.with_temp_session(session, |temp| {
                let muted = temp
                    .strips
                    .get(strip_index)
                    .and_then(|strip| strip.apps.get(app_index))
                    .map(|app| !app.muted)
                    .unwrap_or(false);
                temp.set_virtual_app_muted(strip_index, app_index, muted);
                vec![BackendEvent::VirtualAppMutedChanged {
                    strip_index,
                    app_index,
                    muted,
                }]
            }),
            BackendCommand::SetAppRoute { app_id, target } => {
                self.with_temp_session(session, |temp| {
                    let mut routes = temp.app_routes.clone();
                    if let Some(route) = routes.iter_mut().find(|route| route.id == app_id) {
                        route.target = target;
                    }
                    temp.set_app_routes(routes.clone());
                    vec![BackendEvent::AppRoutesUpdated { routes }]
                })
            }
            BackendCommand::SetAppLevel { app_id, level } => {
                self.with_temp_session(session, |temp| {
                    let mut routes = temp.app_routes.clone();
                    if let Some(route) = routes.iter_mut().find(|route| route.id == app_id) {
                        route.level = level.clamp(0.0, 1.0);
                    }
                    temp.set_app_routes(routes.clone());
                    vec![BackendEvent::AppRoutesUpdated { routes }]
                })
            }
            BackendCommand::ToggleAppMuted { app_id } => self.with_temp_session(session, |temp| {
                let mut routes = temp.app_routes.clone();
                if let Some(route) = routes.iter_mut().find(|route| route.id == app_id) {
                    route.muted = !route.muted;
                }
                temp.set_app_routes(routes.clone());
                vec![BackendEvent::AppRoutesUpdated { routes }]
            }),
        }
    }
}

#[allow(dead_code)]
impl MockBackend {
    fn with_temp_session<F>(&mut self, session: &SessionState, f: F) -> Vec<BackendEvent>
    where
        F: FnOnce(&mut SessionState) -> Vec<BackendEvent>,
    {
        let mut temp = session.clone();
        let mut events = f(&mut temp);
        events.push(self.next_meter_event(&temp));
        events
    }

    fn next_meter_event(&mut self, session: &SessionState) -> BackendEvent {
        self.phase += 0.19;
        BackendEvent::MetersUpdated {
            snapshot: session.meter_snapshot(self.phase),
        }
    }
}
