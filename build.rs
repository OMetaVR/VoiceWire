use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("VoiceWire").qml_files([
            "qml/components/AppVolumeRow.qml",
            "qml/components/AppRoutingRow.qml",
            "qml/Main.qml",
            "qml/components/Bus.qml",
            "qml/components/DeviceSelector.qml",
            "qml/components/Fader.qml",
            "qml/components/InlineSlider.qml",
            "qml/components/LevelMeter.qml",
            "qml/components/MiniKnob.qml",
            "qml/components/PadControl.qml",
            "qml/components/RouteButton.qml",
            "qml/components/SettingsPanel.qml",
            "qml/components/StereoMeter.qml",
            "qml/components/Strip.qml",
            "qml/components/VirtualInputPanel.qml",
        ]),
    )
    .qrc_resources(["qml/assets/settings-configure-symbolic.svg"])
    .qt_module("Gui")
    .qt_module("Quick")
    .qt_module("QuickControls2")
    .file("src/mixer_controller.rs")
    .build();
}
