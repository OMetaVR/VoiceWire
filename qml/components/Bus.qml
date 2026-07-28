import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "."

Rectangle {
    id: root

    required property int busIndex
    required property var busData
    required property var meterData
    required property var controller

    readonly property color surfaceColor: "#252525"
    readonly property color edgeColor: "#343434"
    readonly property color textColor: "#ece7e2"
    readonly property color mutedTextColor: "#a49d95"
    readonly property color meterColor: "#6fa67e"
    readonly property color fillColor: "#6fa67e"
    readonly property color dangerColor: "#a15a52"
    readonly property bool isHardwareBus: root.busIndex < 3
    property real liveGainValue: Number(root.busData && root.busData.gainDb !== undefined ? root.busData.gainDb : 0)
    property bool gainCommitPending: false
    property real pendingGainValue: liveGainValue
    property bool previewDirty: false
    property real previewGainValue: liveGainValue

    color: root.surfaceColor
    radius: 4
    border.color: root.edgeColor
    clip: true

    onBusDataChanged: {
        const modelValue = Number(root.busData && root.busData.gainDb !== undefined ? root.busData.gainDb : 0)
        if (gainCommitPending) {
            if (Math.abs(modelValue - pendingGainValue) <= 0.01) {
                gainCommitPending = false
                liveGainValue = modelValue
            }
            return
        }

        if (!gainSlider.pressed)
            liveGainValue = modelValue
    }

    Timer {
        id: gainPreviewTimer

        interval: 16
        repeat: true
        running: gainSlider.pressed
        onTriggered: {
            if (!root.previewDirty)
                return

            root.previewDirty = false
            root.controller.preview_bus_gain(root.busIndex, root.previewGainValue)
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 5
        spacing: 4

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
                    height: parent.height
                    from: -60
                    to: 12
                    stepSize: 0.5
                    defaultValue: 0
                    value: root.liveGainValue
                    fillColor: root.fillColor
                    trackColor: "#101010"
                    borderColor: root.edgeColor
                    onMoved: function(nextValue) {
                        root.gainCommitPending = false
                        root.liveGainValue = nextValue
                        root.previewGainValue = nextValue
                        root.previewDirty = true
                    }
                    onReleased: function(nextValue) {
                        root.liveGainValue = nextValue
                        root.pendingGainValue = nextValue
                        root.gainCommitPending = true
                        root.previewDirty = false
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
