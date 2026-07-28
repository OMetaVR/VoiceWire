import QtQuick
import QtQuick.Controls

Item {
    id: root

    required property string label
    signal moved(real nextValue)
    signal released(real nextValue)
    property real value: 0.0
    property real from: -1.0
    property real to: 1.0
    property real defaultValue: 0.0
    property color accentColor: "#90b59c"
    property color textColor: "#ece7e2"
    property color mutedTextColor: "#a49d95"
    property color borderColor: "#353535"
    property color faceColor: "#232323"
    property bool bipolar: false
    readonly property bool pressed: knobMouseArea.pressed

    implicitWidth: 42
    implicitHeight: 50

    function clamp(nextValue) {
        return Math.max(root.from, Math.min(root.to, nextValue))
    }

    function applyDelta(delta) {
        root.value = root.clamp(root.value + delta * (root.to - root.from))
    }

    Label {
        anchors.top: parent.top
        anchors.horizontalCenter: parent.horizontalCenter
        text: root.label
        color: root.mutedTextColor
        font.family: "Noto Sans"
        font.pixelSize: 8
    }

    Rectangle {
        id: dial

        anchors.top: parent.top
        anchors.topMargin: 10
        anchors.horizontalCenter: parent.horizontalCenter
        width: 30
        height: 30
        radius: 15
        color: root.faceColor
        border.color: root.borderColor

        Rectangle {
            id: indicator
            width: 2
            height: 10
            radius: 1
            color: root.accentColor
            x: Math.round((parent.width - width) / 2)
            y: Math.round(parent.height / 2 - height + 1)
            antialiasing: true

            transform: Rotation {
                origin.x: indicator.width / 2
                origin.y: indicator.height - 1
                angle: -130 + ((root.value - root.from) / (root.to - root.from)) * 260
            }
        }

        MouseArea {
            id: knobMouseArea
            anchors.fill: parent
            cursorShape: Qt.SizeVerCursor
            property real lastY: 0
            onPressed: lastY = mouse.y
            onPositionChanged: {
                const delta = (lastY - mouse.y) / 140
                root.applyDelta(delta)
                lastY = mouse.y
                root.moved(root.value)
            }
            onReleased: root.released(root.value)
            onDoubleClicked: {
                root.value = root.defaultValue
                root.moved(root.value)
                root.released(root.value)
            }
        }

        WheelHandler {
            onWheel: function(event) {
                root.applyDelta(event.angleDelta.y / 2400)
                root.moved(root.value)
                root.released(root.value)
            }
        }
    }

    Label {
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        text: Number(root.value).toFixed(1)
        color: root.textColor
        font.family: "Noto Sans"
        font.pixelSize: 8
    }
}
