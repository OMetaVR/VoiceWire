pub mod mock;
pub mod pipewire;

use crate::model::{AppRouteState, DeviceOption, MeterSnapshot};

pub trait AudioBackend {
    fn handle(
        &mut self,
        session: &crate::model::SessionState,
        command: BackendCommand,
    ) -> Vec<BackendEvent>;
}

#[allow(dead_code)]
pub enum BackendCommand {
    Start,
    Shutdown,
    RefreshMeters,
    PreviewStripGain { strip_index: usize, gain_db: f32 },
    PreviewBusGain { bus_index: usize, gain_db: f32 },
    ToggleRoute { strip_index: usize, bus_index: usize },
    SetStripGain { strip_index: usize, gain_db: f32 },
    SetBusGain { bus_index: usize, gain_db: f32 },
    ToggleStripMuted { strip_index: usize },
    ToggleStripSolo { strip_index: usize },
    ToggleStripMono { strip_index: usize },
    ToggleBusMuted { bus_index: usize },
    SetStripDevice { strip_index: usize, device_name: String },
    SetBusDevice { bus_index: usize, device_name: String },
    SetVirtualAppLevel { strip_index: usize, app_index: usize, level: f32 },
    ToggleVirtualAppMuted { strip_index: usize, app_index: usize },
    RefreshAppRoutes,
    SetAppRoute { app_id: u32, target: String },
    SetAppLevel { app_id: u32, level: f32 },
    ToggleAppMuted { app_id: u32 },
}

#[allow(dead_code)]
pub enum BackendEvent {
    DevicesUpdated {
        input_devices: Vec<DeviceOption>,
        output_devices: Vec<DeviceOption>,
    },
    VirtualEndpointsReady {
        ready: bool,
    },
    RouteChanged {
        strip_index: usize,
        bus_index: usize,
        enabled: bool,
    },
    StripGainChanged {
        strip_index: usize,
        gain_db: f32,
    },
    BusGainChanged {
        bus_index: usize,
        gain_db: f32,
    },
    StripMutedChanged {
        strip_index: usize,
        muted: bool,
    },
    StripSoloChanged {
        strip_index: usize,
        solo: bool,
    },
    StripMonoChanged {
        strip_index: usize,
        mono: bool,
    },
    BusMutedChanged {
        bus_index: usize,
        muted: bool,
    },
    StripDeviceChanged {
        strip_index: usize,
        device_name: String,
    },
    BusBindingChanged {
        bus_index: usize,
        device_name: String,
    },
    VirtualAppLevelChanged {
        strip_index: usize,
        app_index: usize,
        level: f32,
    },
    VirtualAppMutedChanged {
        strip_index: usize,
        app_index: usize,
        muted: bool,
    },
    AppRoutesUpdated {
        routes: Vec<AppRouteState>,
    },
    MetersUpdated {
        snapshot: MeterSnapshot,
    },
    BackendError {
        message: String,
    },
    ConnectionStateChanged {
        connected: bool,
    },
}
