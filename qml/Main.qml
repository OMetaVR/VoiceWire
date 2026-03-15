import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import VoiceWire
import "components"

ApplicationWindow {
    id: window

    width: 1360
    height: 720
    minimumWidth: 1360
    minimumHeight: 720
    maximumWidth: 1360
    maximumHeight: 720
    visible: true
    color: "#161616"
    title: "VoiceWire"

    onClosing: mixer.shutdown()

    Component.onDestruction: mixer.shutdown()

    MixerController {
        id: mixer
    }

    SettingsPanel {
        id: settingsPanel
        controller: mixer
        appRoutes: window.sessionState.appRoutes || []
    }

    Timer {
        interval: 40
        running: window.visible
        repeat: true
        onTriggered: mixer.refresh_demo_levels()
    }

    property var sessionState: {
        try {
            return JSON.parse(mixer.state_json)
        } catch (error) {
            return { strips: [], buses: [] }
        }
    }

    property var meterState: {
        try {
            return JSON.parse(mixer.meter_json)
        } catch (error) {
            return { stripMeters: [], busMeters: [], appMeters: [] }
        }
    }

    readonly property var busLabels: ["A1", "A2", "A3", "B1", "B2", "B3"]
    readonly property int laneGap: 6

    QtObject {
        id: palette
        readonly property color background: "#161616"
        readonly property color surface: "#212121"
        readonly property color surfaceRaised: "#262626"
        readonly property color panelEdge: "#353535"
        readonly property color text: "#ece7e2"
        readonly property color mutedText: "#a49d95"
        readonly property color accent: "#7a9b86"
        readonly property color accentStrong: "#90b59c"
        readonly property color warning: "#b98e52"
        readonly property color danger: "#a15a52"
        readonly property color meterTrack: "#101010"
    }

    Rectangle {
        anchors.fill: parent
        color: palette.background

        RowLayout {
            anchors.fill: parent
            anchors.margins: 8
            spacing: 8

            Rectangle {
                id: stripsPanel
                Layout.fillWidth: true
                Layout.fillHeight: true
                radius: 4
                color: palette.surface
                border.color: palette.panelEdge

                property int stripCount: (window.sessionState.strips || []).length
                property real columnWidth: Math.floor((stripViewport.width - Math.max(0, stripCount - 1) * window.laneGap) / Math.max(1, stripCount))

                ColumnLayout {
                    id: stripViewport
                    anchors.fill: parent
                    anchors.margins: 8
                    spacing: 8

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 12

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 28
                            color: "#1c1c1c"
                            border.color: palette.panelEdge
                            radius: 4

                            Label {
                                anchors.centerIn: parent
                                text: "Hardware Inputs"
                                color: palette.text
                                font.family: "Noto Sans"
                                font.pixelSize: 11
                                font.weight: Font.DemiBold
                            }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 28
                            color: "#1c1c1c"
                            border.color: palette.panelEdge
                            radius: 4

                            Label {
                                anchors.centerIn: parent
                                text: "Software Inputs"
                                color: palette.text
                                font.family: "Noto Sans"
                                font.pixelSize: 11
                                font.weight: Font.DemiBold
                            }
                        }
                    }

                    Row {
                        id: stripsRow
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        spacing: window.laneGap

                        Repeater {
                            model: window.sessionState.strips || []

                            Strip {
                                required property int index
                                required property var modelData

                                width: stripsPanel.columnWidth
                                height: stripsRow.height
                                stripIndex: index
                                stripData: modelData
                                meterData: (window.meterState.stripMeters || [])[index] || { left: 0, right: 0 }
                                appMeterData: (window.meterState.appMeters || [])[index] || []
                                controller: mixer
                                busLabels: window.busLabels
                            }
                        }
                    }
                }
            }

            Rectangle {
                id: busesPanel
                Layout.preferredWidth: 620
                Layout.fillHeight: true
                radius: 4
                color: palette.surface
                border.color: palette.panelEdge

                property int busCount: (window.sessionState.buses || []).length
                property real columnWidth: Math.floor((busViewport.width - Math.max(0, busCount - 1) * window.laneGap) / Math.max(1, busCount))

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 8
                    spacing: 8

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 0

                        Label {
                            text: "Master Section"
                            color: palette.text
                            opacity: 0.7
                            font.family: "Noto Sans"
                            font.pixelSize: 16
                            font.weight: Font.DemiBold
                            Layout.alignment: Qt.AlignVCenter
                        }

                        Item {
                            Layout.fillWidth: true
                        }

                        ToolButton {
                            id: settingsButton

                            Layout.preferredWidth: 28
                            Layout.preferredHeight: 28
                            padding: 0
                            hoverEnabled: true
                            onClicked: {
                                mixer.refresh_app_routes()
                                settingsPanel.open()
                            }

                            background: Rectangle {
                                radius: 4
                                color: settingsButton.down ? "#1c1c1c" : (settingsButton.hovered ? "#1a1a1a" : "transparent")
                                border.color: settingsButton.hovered ? palette.panelEdge : "transparent"
                            }

                            contentItem: Item {
                                Image {
                                    anchors.centerIn: parent
                                    width: 18
                                    height: 18
                                    source: "qrc:/qt/qml/VoiceWire/qml/assets/settings-configure-symbolic.svg"
                                    fillMode: Image.PreserveAspectFit
                                    smooth: true
                                }
                            }

                            ToolTip.visible: hovered
                            ToolTip.text: "Settings"
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 12

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 28
                            color: "#1c1c1c"
                            border.color: palette.panelEdge
                            radius: 4

                            Label {
                                anchors.centerIn: parent
                                text: "Hardware Outs"
                                color: palette.text
                                font.family: "Noto Sans"
                                font.pixelSize: 11
                                font.weight: Font.DemiBold
                            }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 28
                            color: "#1c1c1c"
                            border.color: palette.panelEdge
                            radius: 4

                            Label {
                                anchors.centerIn: parent
                                text: "Software Outs"
                                color: palette.text
                                font.family: "Noto Sans"
                                font.pixelSize: 11
                                font.weight: Font.DemiBold
                            }
                        }
                    }

                    Item {
                        id: busViewport
                        Layout.fillWidth: true
                        Layout.fillHeight: true

                        Row {
                            id: busRow
                            anchors.fill: parent
                            spacing: window.laneGap

                            Repeater {
                                model: window.sessionState.buses || []

                                Bus {
                                    required property int index
                                    required property var modelData

                                    width: busesPanel.columnWidth
                                    height: busViewport.height
                                    busIndex: index
                                    busData: modelData
                                    meterData: (window.meterState.busMeters || [])[index] || { left: 0, right: 0 }
                                    controller: mixer
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
