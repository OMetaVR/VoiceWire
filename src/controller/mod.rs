use crate::backend::{AudioBackend, BackendCommand, BackendEvent};
use crate::model::SessionState;
use crate::persistence;

pub struct AppController {
    session: SessionState,
    backend: Box<dyn AudioBackend>,
}

impl AppController {
    pub fn new(backend: Box<dyn AudioBackend>) -> Self {
        let mut controller = Self {
            session: persistence::load_session().unwrap_or_else(SessionState::demo),
            backend,
        };
        controller.dispatch(BackendCommand::Start);
        controller
    }

    pub fn session(&self) -> &SessionState {
        &self.session
    }

    pub fn dispatch(&mut self, command: BackendCommand) {
        let should_persist = should_persist_command(&command);
        let events = self.backend.handle(&self.session, command);
        for event in events {
            self.apply_event(event);
        }
        if should_persist {
            persistence::save_session_async(self.session.clone());
        }
    }

    fn apply_event(&mut self, event: BackendEvent) {
        match event {
            BackendEvent::DevicesUpdated {
                input_devices,
                output_devices,
            } => {
                self.session.set_strip_device_options(&input_devices);
                self.session.set_bus_device_options(&output_devices);
                self.session.clear_backend_error();
            }
            BackendEvent::VirtualEndpointsReady { ready } => {
                self.session.set_virtual_endpoints_ready(ready);
            }
            BackendEvent::RouteChanged {
                strip_index,
                bus_index,
                enabled,
            } => {
                self.session
                    .set_route_enabled(strip_index, bus_index, enabled);
            }
            BackendEvent::StripGainChanged {
                strip_index,
                gain_db,
            } => {
                self.session.set_strip_gain(strip_index, gain_db);
            }
            BackendEvent::BusGainChanged { bus_index, gain_db } => {
                self.session.set_bus_gain(bus_index, gain_db);
            }
            BackendEvent::StripMutedChanged { strip_index, muted } => {
                self.session.set_strip_muted(strip_index, muted);
            }
            BackendEvent::StripSoloChanged { strip_index, solo } => {
                self.session.set_strip_solo(strip_index, solo);
            }
            BackendEvent::StripMonoChanged { strip_index, mono } => {
                self.session.set_strip_mono(strip_index, mono);
            }
            BackendEvent::BusMutedChanged { bus_index, muted } => {
                self.session.set_bus_muted(bus_index, muted);
            }
            BackendEvent::StripDeviceChanged {
                strip_index,
                device_name,
            } => {
                self.session.set_strip_device(strip_index, &device_name);
            }
            BackendEvent::BusBindingChanged { bus_index, device_name } => {
                self.session.set_bus_device(bus_index, &device_name);
            }
            BackendEvent::VirtualAppLevelChanged {
                strip_index,
                app_index,
                level,
            } => {
                self.session
                    .set_virtual_app_level(strip_index, app_index, level);
            }
            BackendEvent::VirtualAppMutedChanged {
                strip_index,
                app_index,
                muted,
            } => {
                self.session
                    .set_virtual_app_muted(strip_index, app_index, muted);
            }
            BackendEvent::AppRoutesUpdated { routes } => {
                self.session.set_app_routes(routes);
            }
            BackendEvent::MetersUpdated { snapshot } => {
                self.session.set_meters(&snapshot);
            }
            BackendEvent::BackendError { message } => {
                self.session.set_backend_error(message);
            }
            BackendEvent::ConnectionStateChanged { connected } => {
                self.session.set_connection_state(connected);
                if connected {
                    self.session.clear_backend_error();
                }
            }
        }
    }
}

fn should_persist_command(command: &BackendCommand) -> bool {
    !matches!(
        command,
        BackendCommand::Start
            | BackendCommand::RefreshMeters
            | BackendCommand::RefreshAppRoutes
            | BackendCommand::SetAppRoute { .. }
            | BackendCommand::SetAppLevel { .. }
            | BackendCommand::ToggleAppMuted { .. }
            | BackendCommand::SetVirtualAppLevel { .. }
            | BackendCommand::ToggleVirtualAppMuted { .. }
            | BackendCommand::PreviewStripGain { .. }
            | BackendCommand::PreviewBusGain { .. }
    )
}
