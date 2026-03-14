import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "."

Rectangle {
    id: root

    required property int busIndex
    required property var busData
    required property var controller

    readonly property color surfaceColor: "#252525"
    readonly property color edgeColor: "#343434"
    readonly property color textColor: "#ece7e2"
    readonly property color mutedTextColor: "#a49d95"
    readonly property color meterColor: "#d6e7de"
    readonly property color fillColor: "#6fa67e"
    readonly property color dangerColor: "#a15a52"
    readonly property bool isHardwareBus: root.busIndex < 3
    property real liveGainValue: Number(root.busData && root.busData.gainDb !== undefined ? root.busData.gainDb : 0)

    color: root.surfaceColor
    radius: 4
    border.color: root.edgeColor
    clip: true

    onBusDataChanged: {
        if (!gainSlider.pressed)
            liveGainValue = Number(root.busData && root.busData.gainDb !== undefined ? root.busData.gainDb : 0)
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 6
        spacing: 5

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 1

            Label {
                text: root.busData.title
                color: root.textColor
                font.family: "Noto Sans"
                font.pixelSize: 11
                font.weight: Font.DemiBold
            }

            Label {
                text: root.busData.subtitle
                color: root.mutedTextColor
                font.family: "Noto Sans"
                font.pixelSize: 8
            }
        }

        Loader {
            Layout.fillWidth: true
            sourceComponent: root.isHardwareBus ? deviceSelector : virtualBadge
        }

        RowLayout {
            id: mixerRow

            Layout.fillWidth: true
            spacing: 4
            Layout.minimumHeight: 520
            Layout.maximumHeight: 520
            Layout.preferredHeight: 520

            Item {
                Layout.preferredWidth: 18
                Layout.fillHeight: true

                StereoMeter {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: parent.height
                    leftLevel: root.busData.meterLeft
                    rightLevel: root.busData.meterRight
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
                    height: parent.height
                    from: -60
                    to: 12
                    stepSize: 0.5
                    value: root.liveGainValue
                    fillColor: root.fillColor
                    trackColor: "#101010"
                    borderColor: root.edgeColor
                    onMoved: function(nextValue) {
                        root.liveGainValue = nextValue
                    }
                    onReleased: function(nextValue) {
                        root.liveGainValue = nextValue
                        root.controller.set_bus_gain(root.busIndex, nextValue)
                    }
                }
            }
        }

        Item {
            Layout.fillHeight: true
            visible: true
        }

        Button {
            Layout.fillWidth: true
            text: "Mute"
            onClicked: root.controller.toggle_bus_muted(root.busIndex)

            contentItem: Label {
                text: parent.text
                color: root.busData.muted ? "#171717" : root.textColor
                font.family: "Noto Sans"
                font.pixelSize: 9
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }

            background: Rectangle {
                radius: 3
                color: root.busData.muted ? root.dangerColor : "#202020"
                border.color: root.edgeColor
            }
        }
    }

    Component {
        id: deviceSelector

        DeviceSelector {
            implicitHeight: 26
            options: root.busData.deviceOptions || []
            currentValue: root.busData.destinationName
            onPicked: function(value) {
                root.controller.set_bus_device(root.busIndex, value)
            }
        }
    }

    Component {
        id: virtualBadge

        Rectangle {
            implicitHeight: 26
            radius: 3
            color: "#1d1d1d"
            border.color: root.edgeColor

            Label {
                anchors.fill: parent
                anchors.leftMargin: 8
                anchors.rightMargin: 8
                text: root.busData.destinationName
                color: root.textColor
                font.family: "Noto Sans"
                font.pixelSize: 9
                verticalAlignment: Text.AlignVCenter
                clip: true
                elide: Text.ElideNone
            }
        }
    }
}
