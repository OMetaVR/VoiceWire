import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "."

Rectangle {
    id: root

    required property int stripIndex
    required property var stripData
    required property var meterData
    required property var appMeterData
    required property var controller
    required property var busLabels
    required property bool quickRouteEnabled
    property var quickRouteState: null

    readonly property color surfaceColor: "#252525"
    readonly property color edgeColor: "#343434"
    readonly property color textColor: "#ece7e2"
    readonly property color mutedTextColor: "#a49d95"
    readonly property color meterColor: "#6fa67e"
    readonly property color fillColor: "#6fa67e"
    readonly property color dangerColor: "#a15a52"
    readonly property color warningColor: "#b98e52"
    readonly property bool isVirtual: root.stripData.endpointKind === "virtualInput"
    readonly property string routeTargetLabel: {
        const headerName = String(root.stripData && root.stripData.headerName ? root.stripData.headerName : "")
        if (headerName.indexOf("VAIO") !== -1)
            return "VAIO"
        if (headerName.indexOf("AUX") !== -1)
            return "AUX"
        if (headerName.indexOf("SYS") !== -1)
            return "SYS"
        return ""
    }
    property real liveGainValue: Number(root.stripData && root.stripData.gainDb !== undefined ? root.stripData.gainDb : 0)

    color: root.surfaceColor
    radius: 4
    border.color: root.edgeColor
    clip: true

    onStripDataChanged: {
        if (!gainSlider.pressed)
            liveGainValue = Number(root.stripData && root.stripData.gainDb !== undefined ? root.stripData.gainDb : 0)
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 6
        spacing: 5

        Loader {
            id: headerLoader

            Layout.fillWidth: true
            Layout.preferredHeight: headerLoader.item
                ? headerLoader.item.implicitHeight
                : (root.isVirtual ? 269 : 238)
            sourceComponent: root.isVirtual ? virtualHeader : hardwareHeader
        }

        RowLayout {
            id: mixerRow

            Layout.fillWidth: true
            spacing: 5
            Layout.minimumHeight: 340
            Layout.maximumHeight: 340
            Layout.preferredHeight: 340

            Item {
                Layout.preferredWidth: 18
                Layout.fillHeight: true

                StereoMeter {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: 340
                    leftLevel: Number(root.meterData && root.meterData.left !== undefined ? root.meterData.left : 0)
                    rightLevel: Number(root.meterData && root.meterData.right !== undefined ? root.meterData.right : 0)
                    fillColor: root.meterColor
                }
            }

            Item {
                Layout.preferredWidth: 44
                Layout.fillHeight: true

                Fader {
                    id: gainSlider

                    anchors.horizontalCenter: parent.horizontalCenter
                    anchors.bottom: parent.bottom
                    width: 44
                    height: 340
                    from: -60
                    to: 12
                    stepSize: 0.5
                    value: root.liveGainValue
                    fillColor: root.fillColor
                    trackColor: "#101010"
                    borderColor: root.edgeColor
                    onMoved: function(nextValue) {
                        root.liveGainValue = nextValue
                        root.controller.preview_strip_gain(root.stripIndex, nextValue)
                    }
                    onReleased: function(nextValue) {
                        root.liveGainValue = nextValue
                        root.controller.set_strip_gain(root.stripIndex, nextValue)
                    }
                }
            }

            ColumnLayout {
                Layout.preferredWidth: 38
                Layout.fillHeight: true
                spacing: 4

                Repeater {
                    model: root.stripData.routes || []

                    RouteButton {
                        required property int index
                        required property var modelData

                        Layout.fillWidth: true
                        text: modelData.label
                        active: modelData.enabled
                        onClicked: root.controller.toggle_strip_route(root.stripIndex, index)
                    }
                }

                Button {
                    Layout.fillWidth: true
                    text: "Mono"
                    onClicked: root.controller.toggle_strip_mono(root.stripIndex)

                    contentItem: Label {
                        text: parent.text
                        color: root.stripData.mono ? "#171717" : root.textColor
                        font.family: "Noto Sans"
                        font.pixelSize: 8
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }

                    background: Rectangle {
                        color: root.stripData.mono ? "#90b59c" : "#202020"
                        border.color: root.edgeColor
                        radius: 3
                    }
                }

                Item {
                    Layout.fillHeight: true
                }
            }
        }

        Item {
            Layout.fillHeight: true
            visible: true
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 5

            Button {
                Layout.fillWidth: true
                text: "Solo"
                onClicked: root.controller.toggle_strip_solo(root.stripIndex)

                contentItem: Label {
                    text: parent.text
                    color: root.stripData.solo ? "#171717" : root.textColor
                    font.family: "Noto Sans"
                    font.pixelSize: 9
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                background: Rectangle {
                    radius: 3
                    color: root.stripData.solo ? root.warningColor : "#202020"
                    border.color: root.edgeColor
                }
            }

            Button {
                Layout.fillWidth: true
                text: "Mute"
                onClicked: root.controller.toggle_strip_muted(root.stripIndex)

                contentItem: Label {
                    text: parent.text
                    color: root.stripData.muted ? "#171717" : root.textColor
                    font.family: "Noto Sans"
                    font.pixelSize: 9
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                background: Rectangle {
                    radius: 3
                    color: root.stripData.muted ? root.dangerColor : "#202020"
                    border.color: root.edgeColor
                }
            }
        }
    }

    Component {
        id: hardwareHeader

        ColumnLayout {
            implicitHeight: 238
            spacing: 5

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 1

                Label {
                    text: root.stripData.title
                    color: root.textColor
                    font.family: "Noto Sans"
                    font.pixelSize: 11
                    font.weight: Font.DemiBold
                }

                Label {
                    text: root.stripData.subtitle
                    color: root.mutedTextColor
                    font.family: "Noto Sans"
                    font.pixelSize: 8
                }
            }

            DeviceSelector {
                Layout.fillWidth: true
                options: root.stripData.deviceOptions || []
                currentValue: root.stripData.deviceName
                onPicked: function(value) {
                    root.controller.set_strip_device(root.stripIndex, value)
                }
            }

            PadControl {
                Layout.fillWidth: true
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 4

                MiniKnob {
                    Layout.fillWidth: true
                    label: "Comp."
                }

                MiniKnob {
                    Layout.fillWidth: true
                    label: "Gate"
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 4

                MiniKnob {
                    Layout.fillWidth: true
                    label: "Reverb"
                    accentColor: "#ef8f7b"
                }

                MiniKnob {
                    Layout.fillWidth: true
                    label: "Width"
                    accentColor: "#90b59c"
                    bipolar: true
                    value: 0.0
                }

                MiniKnob {
                    Layout.fillWidth: true
                    label: "Delay"
                    accentColor: "#ef8f7b"
                }
            }
        }
    }

    Component {
        id: virtualHeader

        VirtualInputPanel {
            stripIndex: root.stripIndex
            stripData: root.stripData
            controller: root.controller
            quickRouteEnabled: root.quickRouteEnabled
            quickRouteState: root.quickRouteState
            routeTargetLabel: root.routeTargetLabel
        }
    }
}
