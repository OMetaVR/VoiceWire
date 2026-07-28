use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

use crate::backend::BackendCommand;
use crate::backend::pipewire::PipeWireDiscoveryBackend;
use crate::controller::AppController;

fn to_qstring_json(session: &crate::model::SessionState) -> QString {
    QString::from(
        serde_json::to_string(session).expect("session state should always serialize to JSON"),
    )
}

fn to_qstring_meter_json(session: &crate::model::SessionState) -> QString {
    QString::from(
        serde_json::to_string(&session.current_meter_snapshot())
            .expect("meter snapshot should always serialize to JSON"),
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
        #[qproperty(QString, meter_json)]
        #[namespace = "voicewire"]
        type MixerController = super::MixerControllerRust;

        #[qinvokable]
        fn toggle_strip_route(self: Pin<&mut MixerController>, strip_index: i32, bus_index: i32);

        #[qinvokable]
        fn set_strip_gain(self: Pin<&mut MixerController>, strip_index: i32, gain_db: f32);

        #[qinvokable]
        fn set_strip_gate(self: Pin<&mut MixerController>, strip_index: i32, gate: f32);

        #[qinvokable]
        fn preview_strip_gain(self: Pin<&mut MixerController>, strip_index: i32, gain_db: f32);

        #[qinvokable]
        fn set_bus_gain(self: Pin<&mut MixerController>, bus_index: i32, gain_db: f32);

        #[qinvokable]
        fn preview_bus_gain(self: Pin<&mut MixerController>, bus_index: i32, gain_db: f32);

        #[qinvokable]
        fn preview_app_level(self: Pin<&mut MixerController>, app_id: i32, level: f32);

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

        #[qinvokable]
        fn refresh_app_routes(self: Pin<&mut MixerController>);

        #[qinvokable]
        fn set_quick_route_enabled(self: Pin<&mut MixerController>, enabled: bool);

        #[qinvokable]
        fn set_app_route(self: Pin<&mut MixerController>, app_id: i32, target: &QString);

        #[qinvokable]
        fn set_app_level(self: Pin<&mut MixerController>, app_id: i32, level: f32);

        #[qinvokable]
        fn toggle_app_muted(self: Pin<&mut MixerController>, app_id: i32);

        #[qinvokable]
        fn shutdown(self: Pin<&mut MixerController>);
    }
}

pub struct MixerControllerRust {
    state_json: QString,
    meter_json: QString,
    app: AppController,
    shutdown_sent: bool,
}

impl Default for MixerControllerRust {
    fn default() -> Self {
        let app = AppController::new(Box::new(PipeWireDiscoveryBackend::default()));
        let state_json = to_qstring_json(app.session());
        let meter_json = to_qstring_meter_json(app.session());
        Self {
            state_json,
            meter_json,
            app,
            shutdown_sent: false,
        }
    }
}

impl MixerControllerRust {
    fn dispatch_shutdown_once(&mut self) {
        if self.shutdown_sent {
            return;
        }
        self.shutdown_sent = true;
        self.app.dispatch(BackendCommand::Shutdown);
    }
}

impl Drop for MixerControllerRust {
    fn drop(&mut self) {
        self.dispatch_shutdown_once();
    }
}

impl qobject::MixerController {
    fn sync_state_json(mut self: core::pin::Pin<&mut Self>) {
        let (json, unchanged) = {
            let borrowed = self.as_ref();
            let rust = borrowed.rust();
            let next = to_qstring_json(rust.app.session());
            let unchanged = rust.state_json == next;
            (next, unchanged)
        };
        if unchanged {
            return;
        }
        self.as_mut().set_state_json(json);
    }

    fn sync_meter_json(mut self: core::pin::Pin<&mut Self>) {
        let (json, unchanged) = {
            let borrowed = self.as_ref();
            let rust = borrowed.rust();
            let next = to_qstring_meter_json(rust.app.session());
            let unchanged = rust.meter_json == next;
            (next, unchanged)
        };
        if unchanged {
            return;
        }
        self.as_mut().set_meter_json(json);
    }

    fn dispatch_command(mut self: core::pin::Pin<&mut Self>, command: BackendCommand) {
        {
            let mut rust = self.as_mut().rust_mut();
            rust.as_mut().get_mut().app.dispatch(command);
        }
        self.as_mut().sync_state_json();
        self.sync_meter_json();
    }

    fn toggle_strip_route(self: core::pin::Pin<&mut Self>, strip_index: i32, bus_index: i32) {
        if strip_index < 0 || bus_index < 0 {
            return;
        }
        self.dispatch_command(BackendCommand::ToggleRoute {
            strip_index: strip_index as usize,
            bus_index: bus_index as usize,
        });
    }

    fn set_strip_gain(self: core::pin::Pin<&mut Self>, strip_index: i32, gain_db: f32) {
        if strip_index < 0 {
            return;
        }
        self.dispatch_command(BackendCommand::SetStripGain {
            strip_index: strip_index as usize,
            gain_db,
        });
    }

    fn set_strip_gate(self: core::pin::Pin<&mut Self>, strip_index: i32, gate: f32) {
        if strip_index < 0 {
            return;
        }
        self.dispatch_command(BackendCommand::SetStripGate {
            strip_index: strip_index as usize,
            gate,
        });
    }

    fn preview_strip_gain(self: core::pin::Pin<&mut Self>, strip_index: i32, gain_db: f32) {
        if strip_index < 0 {
            return;
        }
        let mut rust = self.rust_mut();
        rust.as_mut()
            .get_mut()
            .app
            .dispatch(BackendCommand::PreviewStripGain {
                strip_index: strip_index as usize,
                gain_db,
            });
    }

    fn set_bus_gain(self: core::pin::Pin<&mut Self>, bus_index: i32, gain_db: f32) {
        if bus_index < 0 {
            return;
        }
        self.dispatch_command(BackendCommand::SetBusGain {
            bus_index: bus_index as usize,
            gain_db,
        });
    }

    fn preview_bus_gain(self: core::pin::Pin<&mut Self>, bus_index: i32, gain_db: f32) {
        if bus_index < 0 {
            return;
        }
        let mut rust = self.rust_mut();
        rust.as_mut()
            .get_mut()
            .app
            .dispatch(BackendCommand::PreviewBusGain {
                bus_index: bus_index as usize,
                gain_db,
            });
    }

    fn preview_app_level(self: core::pin::Pin<&mut Self>, app_id: i32, level: f32) {
        if app_id < 0 {
            return;
        }
        let mut rust = self.rust_mut();
        rust.as_mut()
            .get_mut()
            .app
            .dispatch(BackendCommand::PreviewAppLevel {
                app_id: app_id as u32,
                level,
            });
    }

    fn toggle_strip_muted(self: core::pin::Pin<&mut Self>, strip_index: i32) {
        if strip_index < 0 {
            return;
        }
        self.dispatch_command(BackendCommand::ToggleStripMuted {
            strip_index: strip_index as usize,
        });
    }

    fn toggle_strip_solo(self: core::pin::Pin<&mut Self>, strip_index: i32) {
        if strip_index < 0 {
            return;
        }
        self.dispatch_command(BackendCommand::ToggleStripSolo {
            strip_index: strip_index as usize,
        });
    }

    fn toggle_strip_mono(self: core::pin::Pin<&mut Self>, strip_index: i32) {
        if strip_index < 0 {
            return;
        }
        self.dispatch_command(BackendCommand::ToggleStripMono {
            strip_index: strip_index as usize,
        });
    }

    fn toggle_bus_muted(self: core::pin::Pin<&mut Self>, bus_index: i32) {
        if bus_index < 0 {
            return;
        }
        self.dispatch_command(BackendCommand::ToggleBusMuted {
            bus_index: bus_index as usize,
        });
    }

    fn set_strip_device(self: core::pin::Pin<&mut Self>, strip_index: i32, device_name: &QString) {
        if strip_index < 0 {
            return;
        }
        self.dispatch_command(BackendCommand::SetStripDevice {
            strip_index: strip_index as usize,
            device_name: device_name.to_string(),
        });
    }

    fn set_bus_device(self: core::pin::Pin<&mut Self>, bus_index: i32, device_name: &QString) {
        if bus_index < 0 {
            return;
        }
        self.dispatch_command(BackendCommand::SetBusDevice {
            bus_index: bus_index as usize,
            device_name: device_name.to_string(),
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
        self.dispatch_command(BackendCommand::SetVirtualAppLevel {
            strip_index: strip_index as usize,
            app_index: app_index as usize,
            level,
        });
    }

    fn toggle_virtual_app_muted(self: core::pin::Pin<&mut Self>, strip_index: i32, app_index: i32) {
        if strip_index < 0 || app_index < 0 {
            return;
        }
        self.dispatch_command(BackendCommand::ToggleVirtualAppMuted {
            strip_index: strip_index as usize,
            app_index: app_index as usize,
        });
    }

    fn refresh_demo_levels(mut self: core::pin::Pin<&mut Self>) {
        {
            let mut rust = self.as_mut().rust_mut();
            rust.as_mut()
                .get_mut()
                .app
                .dispatch(BackendCommand::RefreshMeters);
        }
        self.sync_meter_json();
    }

    fn refresh_app_routes(self: core::pin::Pin<&mut Self>) {
        self.dispatch_command(BackendCommand::RefreshAppRoutes);
    }

    fn set_quick_route_enabled(mut self: core::pin::Pin<&mut Self>, enabled: bool) {
        {
            let mut rust = self.as_mut().rust_mut();
            rust.as_mut().get_mut().app.set_quick_route_enabled(enabled);
        }
        self.as_mut().sync_state_json();
    }

    fn set_app_route(self: core::pin::Pin<&mut Self>, app_id: i32, target: &QString) {
        if app_id < 0 {
            return;
        }
        self.dispatch_command(BackendCommand::SetAppRoute {
            app_id: app_id as u32,
            target: target.to_string(),
        });
    }

    fn set_app_level(self: core::pin::Pin<&mut Self>, app_id: i32, level: f32) {
        if app_id < 0 {
            return;
        }
        self.dispatch_command(BackendCommand::SetAppLevel {
            app_id: app_id as u32,
            level,
        });
    }

    fn toggle_app_muted(self: core::pin::Pin<&mut Self>, app_id: i32) {
        if app_id < 0 {
            return;
        }
        self.dispatch_command(BackendCommand::ToggleAppMuted {
            app_id: app_id as u32,
        });
    }

    fn shutdown(mut self: core::pin::Pin<&mut Self>) {
        {
            let mut rust = self.as_mut().rust_mut();
            rust.as_mut().get_mut().dispatch_shutdown_once();
        }
        self.as_mut().sync_state_json();
        self.sync_meter_json();
    }
}
