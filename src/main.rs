mod mixer_controller;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

fn main() {
    unsafe {
        std::env::set_var("QT_QUICK_CONTROLS_STYLE", "Basic");
    }
    cxx_qt::init_qml_module!("VoiceWire");

    let mut app = QGuiApplication::new();
    app.as_mut()
        .expect("QGuiApplication should initialize")
        .set_application_name(&QString::from("VoiceWire"));
    app.as_mut()
        .expect("QGuiApplication should initialize")
        .set_application_version(&QString::from("0.1.0"));

    let mut engine = QQmlApplicationEngine::new();
    engine
        .as_mut()
        .expect("QQmlApplicationEngine should initialize")
        .load(&QUrl::from("qrc:/qt/qml/VoiceWire/qml/Main.qml"));

    app.as_mut()
        .expect("QGuiApplication should initialize")
        .exec();
}
