use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
enum EndpointKind {
    HardwareInput,
    VirtualInput,
    HardwareOutput,
    VirtualOutput,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceOption {
    name: String,
    kind: EndpointKind,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RouteState {
    label: String,
    enabled: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppFeedState {
    icon_text: String,
    icon_color: String,
    name: String,
    level: f32,
    meter: f32,
    muted: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StripState {
    title: String,
    subtitle: String,
    device_name: String,
    device_options: Vec<DeviceOption>,
    header_name: String,
    header_detail: String,
    gain_db: f32,
    muted: bool,
    solo: bool,
    mono: bool,
    meter_left: f32,
    meter_right: f32,
    routes: Vec<RouteState>,
    endpoint_kind: EndpointKind,
    apps: Vec<AppFeedState>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BusState {
    title: String,
    subtitle: String,
    destination_name: String,
    device_options: Vec<DeviceOption>,
    gain_db: f32,
    muted: bool,
    meter_left: f32,
    meter_right: f32,
    endpoint_kind: EndpointKind,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionState {
    strips: Vec<StripState>,
    buses: Vec<BusState>,
}

impl SessionState {
    fn demo() -> Self {
        let mic_devices = vec![
            DeviceOption {
                name: "Scarlett 2i2 Mic 1".into(),
                kind: EndpointKind::HardwareInput,
            },
            DeviceOption {
                name: "USB Podcast Mic".into(),
                kind: EndpointKind::HardwareInput,
            },
            DeviceOption {
                name: "HD Webcam Mic".into(),
                kind: EndpointKind::HardwareInput,
            },
        ];
        let virtual_inputs = vec![
            DeviceOption {
                name: "VoiceWire In 1".into(),
                kind: EndpointKind::VirtualInput,
            },
            DeviceOption {
                name: "VoiceWire In 2".into(),
                kind: EndpointKind::VirtualInput,
            },
            DeviceOption {
                name: "VoiceWire In 3".into(),
                kind: EndpointKind::VirtualInput,
            },
        ];
        let hardware_outputs = vec![
            DeviceOption {
                name: "Desk Speakers".into(),
                kind: EndpointKind::HardwareOutput,
            },
            DeviceOption {
                name: "Studio Headphones".into(),
                kind: EndpointKind::HardwareOutput,
            },
            DeviceOption {
                name: "USB Monitor".into(),
                kind: EndpointKind::HardwareOutput,
            },
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
                "VoiceWire In 1",
                "VoiceWire VAIO",
                "44100 Hz - 7168",
                false,
            ),
            (
                "DESKTOP",
                "Virtual Input 2",
                "VoiceWire In 2",
                "VoiceWire AUX",
                "44100 Hz - 7168",
                false,
            ),
            (
                "CHAT",
                "Virtual Input 3",
                "VoiceWire In 3",
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
                |(index, (title, subtitle, device_name, header_name, header_detail, is_hardware))| {
                    let apps = if is_hardware {
                        Vec::new()
                    } else {
                        match index {
                            3 => vec![
                                AppFeedState {
                                    icon_text: "SP".into(),
                                    icon_color: "#335c44".into(),
                                    name: "Spotify".into(),
                                    level: 1.0,
                                    meter: 0.0,
                                    muted: false,
                                },
                                AppFeedState {
                                    icon_text: "YT".into(),
                                    icon_color: "#6d463f".into(),
                                    name: "YouTube".into(),
                                    level: 1.0,
                                    meter: 0.0,
                                    muted: false,
                                },
                                AppFeedState {
                                    icon_text: "VL".into(),
                                    icon_color: "#886b32".into(),
                                    name: "VLC".into(),
                                    level: 1.0,
                                    meter: 0.0,
                                    muted: false,
                                },
                                AppFeedState {
                                    icon_text: "FF".into(),
                                    icon_color: "#584136".into(),
                                    name: "Firefox".into(),
                                    level: 1.0,
                                    meter: 0.0,
                                    muted: false,
                                },
                                AppFeedState {
                                    icon_text: "TM".into(),
                                    icon_color: "#42466a".into(),
                                    name: "Tidal".into(),
                                    level: 1.0,
                                    meter: 0.0,
                                    muted: false,
                                },
                            ],
                            4 => vec![
                                AppFeedState {
                                    icon_text: "FF".into(),
                                    icon_color: "#584136".into(),
                                    name: "Firefox".into(),
                                    level: 1.0,
                                    meter: 0.0,
                                    muted: false,
                                },
                                AppFeedState {
                                    icon_text: "OB".into(),
                                    icon_color: "#444444".into(),
                                    name: "OBS 23.0.2".into(),
                                    level: 1.0,
                                    meter: 0.0,
                                    muted: false,
                                },
                                AppFeedState {
                                    icon_text: "ST".into(),
                                    icon_color: "#2d5460".into(),
                                    name: "Steam".into(),
                                    level: 1.0,
                                    meter: 0.0,
                                    muted: false,
                                },
                                AppFeedState {
                                    icon_text: "VL".into(),
                                    icon_color: "#886b32".into(),
                                    name: "Vivaldi".into(),
                                    level: 1.0,
                                    meter: 0.0,
                                    muted: false,
                                },
                                AppFeedState {
                                    icon_text: "DC".into(),
                                    icon_color: "#42466a".into(),
                                    name: "Discord Overlay".into(),
                                    level: 1.0,
                                    meter: 0.0,
                                    muted: false,
                                },
                            ],
                            _ => vec![
                                AppFeedState {
                                    icon_text: "DC".into(),
                                    icon_color: "#42466a".into(),
                                    name: "Discord".into(),
                                    level: 1.0,
                                    meter: 0.0,
                                    muted: false,
                                },
                                AppFeedState {
                                    icon_text: "TS".into(),
                                    icon_color: "#4b5a3f".into(),
                                    name: "Teamspeak".into(),
                                    level: 1.0,
                                    meter: 0.0,
                                    muted: false,
                                },
                                AppFeedState {
                                    icon_text: "SL".into(),
                                    icon_color: "#5f433e".into(),
                                    name: "Slack Huddle".into(),
                                    level: 1.0,
                                    meter: 0.0,
                                    muted: false,
                                },
                            ]
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
            .map(|(title, subtitle, destination_name, can_assign_device)| BusState {
                title: title.into(),
                subtitle: subtitle.into(),
                destination_name: destination_name.into(),
                device_options: if can_assign_device {
                    hardware_outputs.clone()
                } else {
                    vec![DeviceOption {
                        name: destination_name.into(),
                        kind: EndpointKind::VirtualOutput,
                    }]
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
            })
            .collect();

        let mut session = Self { strips, buses };
        session.refresh_meters(0.0);
        session
    }

    fn toggle_route(&mut self, strip_index: usize, bus_index: usize) {
        if let Some(route) = self
            .strips
            .get_mut(strip_index)
            .and_then(|strip| strip.routes.get_mut(bus_index))
        {
            route.enabled = !route.enabled;
        }
    }

    fn set_strip_gain(&mut self, strip_index: usize, gain_db: f32) {
        if let Some(strip) = self.strips.get_mut(strip_index) {
            strip.gain_db = gain_db.clamp(-60.0, 12.0);
        }
    }

    fn set_bus_gain(&mut self, bus_index: usize, gain_db: f32) {
        if let Some(bus) = self.buses.get_mut(bus_index) {
            bus.gain_db = gain_db.clamp(-60.0, 12.0);
        }
    }

    fn toggle_strip_muted(&mut self, strip_index: usize) {
        if let Some(strip) = self.strips.get_mut(strip_index) {
            strip.muted = !strip.muted;
        }
    }

    fn toggle_strip_solo(&mut self, strip_index: usize) {
        if let Some(strip) = self.strips.get_mut(strip_index) {
            strip.solo = !strip.solo;
        }
    }

    fn toggle_strip_mono(&mut self, strip_index: usize) {
        if let Some(strip) = self.strips.get_mut(strip_index) {
            strip.mono = !strip.mono;
        }
    }

    fn toggle_bus_muted(&mut self, bus_index: usize) {
        if let Some(bus) = self.buses.get_mut(bus_index) {
            bus.muted = !bus.muted;
        }
    }

    fn set_strip_device(&mut self, strip_index: usize, device_name: &str) {
        if let Some(strip) = self.strips.get_mut(strip_index) {
            strip.device_name = device_name.into();
        }
    }

    fn set_bus_device(&mut self, bus_index: usize, device_name: &str) {
        if let Some(bus) = self.buses.get_mut(bus_index) {
            bus.destination_name = device_name.into();
        }
    }

    fn set_virtual_app_level(&mut self, strip_index: usize, app_index: usize, level: f32) {
        if let Some(app) = self
            .strips
            .get_mut(strip_index)
            .and_then(|strip| strip.apps.get_mut(app_index))
        {
            app.level = level.clamp(0.0, 1.0);
        }
    }

    fn toggle_virtual_app_muted(&mut self, strip_index: usize, app_index: usize) {
        if let Some(app) = self
            .strips
            .get_mut(strip_index)
            .and_then(|strip| strip.apps.get_mut(app_index))
        {
            app.muted = !app.muted;
        }
    }

    fn refresh_meters(&mut self, phase: f32) {
        let solo_active = self.strips.iter().any(|strip| strip.solo && !strip.muted);

        for (strip_index, strip) in self.strips.iter_mut().enumerate() {
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
            let left = (base * ((phase + strip_index as f32 * 0.8).sin() * 0.12 + 0.88))
                .clamp(0.0, 1.0);
            let right = (base * ((phase + strip_index as f32 * 0.8 + 0.9).sin() * 0.12 + 0.86))
                .clamp(0.0, 1.0);

            if strip.mono {
                let mono = ((left + right) * 0.5).clamp(0.0, 1.0);
                strip.meter_left = mono;
                strip.meter_right = mono;
            } else {
                strip.meter_left = left;
                strip.meter_right = right;
            }

            for (app_index, app) in strip.apps.iter_mut().enumerate() {
                let app_base = if strip.muted || app.muted {
                    0.0
                } else {
                    let flutter = (phase + strip_index as f32 * 0.7 + app_index as f32 * 0.52).sin()
                        * 0.12
                        + 0.28;
                    (app.level * flutter).clamp(0.0, 1.0)
                };
                app.meter = app_base;
            }
        }

        for (bus_index, bus) in self.buses.iter_mut().enumerate() {
            let input_left: f32 = self
                .strips
                .iter()
                .filter(|strip| strip.routes.get(bus_index).is_some_and(|route| route.enabled))
                .map(|strip| strip.meter_left * ((strip.gain_db + 60.0) / 72.0).clamp(0.2, 1.0))
                .sum();
            let input_right: f32 = self
                .strips
                .iter()
                .filter(|strip| strip.routes.get(bus_index).is_some_and(|route| route.enabled))
                .map(|strip| strip.meter_right * ((strip.gain_db + 60.0) / 72.0).clamp(0.2, 1.0))
                .sum();

            if bus.muted {
                bus.meter_left = 0.0;
                bus.meter_right = 0.0;
            } else {
                let master = ((bus.gain_db + 60.0) / 72.0).clamp(0.18, 1.0);
                bus.meter_left = (input_left * 0.48 * master).clamp(0.0, 1.0);
                bus.meter_right = (input_right * 0.48 * master).clamp(0.0, 1.0);
            }
        }
    }
}

fn to_qstring_json(session: &SessionState) -> QString {
    QString::from(
        serde_json::to_string(session).expect("session state should always serialize to JSON"),
    )
}

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, state_json)]
        #[namespace = "voicewire"]
        type MixerController = super::MixerControllerRust;

        #[qinvokable]
        fn toggle_strip_route(self: Pin<&mut MixerController>, strip_index: i32, bus_index: i32);

        #[qinvokable]
        fn set_strip_gain(self: Pin<&mut MixerController>, strip_index: i32, gain_db: f32);

        #[qinvokable]
        fn set_bus_gain(self: Pin<&mut MixerController>, bus_index: i32, gain_db: f32);

        #[qinvokable]
        fn toggle_strip_muted(self: Pin<&mut MixerController>, strip_index: i32);

        #[qinvokable]
        fn toggle_strip_solo(self: Pin<&mut MixerController>, strip_index: i32);

        #[qinvokable]
        fn toggle_strip_mono(self: Pin<&mut MixerController>, strip_index: i32);

        #[qinvokable]
        fn toggle_bus_muted(self: Pin<&mut MixerController>, bus_index: i32);

        #[qinvokable]
        fn set_strip_device(
            self: Pin<&mut MixerController>,
            strip_index: i32,
            device_name: &QString,
        );

        #[qinvokable]
        fn set_bus_device(self: Pin<&mut MixerController>, bus_index: i32, device_name: &QString);

        #[qinvokable]
        fn set_virtual_app_level(
            self: Pin<&mut MixerController>,
            strip_index: i32,
            app_index: i32,
            level: f32,
        );

        #[qinvokable]
        fn toggle_virtual_app_muted(
            self: Pin<&mut MixerController>,
            strip_index: i32,
            app_index: i32,
        );

        #[qinvokable]
        fn refresh_demo_levels(self: Pin<&mut MixerController>);
    }
}

pub struct MixerControllerRust {
    state_json: QString,
    session: SessionState,
    phase: f32,
}

impl Default for MixerControllerRust {
    fn default() -> Self {
        let session = SessionState::demo();
        let state_json = to_qstring_json(&session);

        Self {
            state_json,
            session,
            phase: 0.0,
        }
    }
}

impl qobject::MixerController {
    fn sync_state_json(mut self: core::pin::Pin<&mut Self>) {
        let json = {
            let borrowed = self.as_ref();
            let rust = borrowed.rust();
            to_qstring_json(&rust.session)
        };
        self.as_mut().set_state_json(json);
    }

    fn with_session_mut<F>(mut self: core::pin::Pin<&mut Self>, f: F)
    where
        F: FnOnce(&mut MixerControllerRust),
    {
        {
            let mut rust = self.as_mut().rust_mut();
            f(rust.as_mut().get_mut());
            rust.as_mut().get_mut().phase += 0.19;
            let phase = rust.as_ref().get_ref().phase;
            rust.as_mut().get_mut().session.refresh_meters(phase);
        }
        self.sync_state_json();
    }

    fn toggle_strip_route(self: core::pin::Pin<&mut Self>, strip_index: i32, bus_index: i32) {
        if strip_index < 0 || bus_index < 0 {
            return;
        }
        self.with_session_mut(move |rust| {
            rust.session
                .toggle_route(strip_index as usize, bus_index as usize);
        });
    }

    fn set_strip_gain(self: core::pin::Pin<&mut Self>, strip_index: i32, gain_db: f32) {
        if strip_index < 0 {
            return;
        }
        self.with_session_mut(move |rust| {
            rust.session.set_strip_gain(strip_index as usize, gain_db);
        });
    }

    fn set_bus_gain(self: core::pin::Pin<&mut Self>, bus_index: i32, gain_db: f32) {
        if bus_index < 0 {
            return;
        }
        self.with_session_mut(move |rust| {
            rust.session.set_bus_gain(bus_index as usize, gain_db);
        });
    }

    fn toggle_strip_muted(self: core::pin::Pin<&mut Self>, strip_index: i32) {
        if strip_index < 0 {
            return;
        }
        self.with_session_mut(move |rust| {
            rust.session.toggle_strip_muted(strip_index as usize);
        });
    }

    fn toggle_strip_solo(self: core::pin::Pin<&mut Self>, strip_index: i32) {
        if strip_index < 0 {
            return;
        }
        self.with_session_mut(move |rust| {
            rust.session.toggle_strip_solo(strip_index as usize);
        });
    }

    fn toggle_strip_mono(self: core::pin::Pin<&mut Self>, strip_index: i32) {
        if strip_index < 0 {
            return;
        }
        self.with_session_mut(move |rust| {
            rust.session.toggle_strip_mono(strip_index as usize);
        });
    }

    fn toggle_bus_muted(self: core::pin::Pin<&mut Self>, bus_index: i32) {
        if bus_index < 0 {
            return;
        }
        self.with_session_mut(move |rust| {
            rust.session.toggle_bus_muted(bus_index as usize);
        });
    }

    fn set_strip_device(
        self: core::pin::Pin<&mut Self>,
        strip_index: i32,
        device_name: &QString,
    ) {
        if strip_index < 0 {
            return;
        }
        let device_name = device_name.to_string();
        self.with_session_mut(move |rust| {
            rust.session
                .set_strip_device(strip_index as usize, &device_name);
        });
    }

    fn set_bus_device(self: core::pin::Pin<&mut Self>, bus_index: i32, device_name: &QString) {
        if bus_index < 0 {
            return;
        }
        let device_name = device_name.to_string();
        self.with_session_mut(move |rust| {
            rust.session.set_bus_device(bus_index as usize, &device_name);
        });
    }

    fn set_virtual_app_level(
        self: core::pin::Pin<&mut Self>,
        strip_index: i32,
        app_index: i32,
        level: f32,
    ) {
        if strip_index < 0 || app_index < 0 {
            return;
        }
        self.with_session_mut(move |rust| {
            rust.session
                .set_virtual_app_level(strip_index as usize, app_index as usize, level);
        });
    }

    fn toggle_virtual_app_muted(
        self: core::pin::Pin<&mut Self>,
        strip_index: i32,
        app_index: i32,
    ) {
        if strip_index < 0 || app_index < 0 {
            return;
        }
        self.with_session_mut(move |rust| {
            rust.session
                .toggle_virtual_app_muted(strip_index as usize, app_index as usize);
        });
    }

    fn refresh_demo_levels(self: core::pin::Pin<&mut Self>) {
        self.with_session_mut(|rust| {
            rust.phase += 0.19;
            let phase = rust.phase;
            rust.session.refresh_meters(phase);
        });
    }
}
