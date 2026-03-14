import QtQuick
import QtQuick.Controls

Rectangle {
    id: root

    property var modes: [
        { title: "VOICE", subtitle: "Color Panel", hint: "echo\nbrightness" },
        { title: "PAN", subtitle: "Stereo", hint: "left\nright" },
        { title: "TONE", subtitle: "Shape", hint: "dark\nbright" }
    ]
    property int modeIndex: 0
    property real pointX: 0.5
    property real pointY: 1.0
    property color accentColor: "#ef8f7b"
    property color textColor: "#ece7e2"
    property color mutedTextColor: "#8d979f"
    property color borderColor: "#353535"
    property color faceColor: "#1f262d"

    implicitHeight: 94
    radius: 4
    color: root.faceColor
    border.color: root.borderColor

    readonly property var mode: root.modes[root.modeIndex]

    function clamp01(value) {
        return Math.max(0, Math.min(1, value))
    }

    Column {
        anchors.fill: parent
        anchors.margins: 6
        spacing: 3

        Row {
            spacing: 6

            Label {
                text: root.mode.title
                color: root.mutedTextColor
                font.family: "Noto Sans"
                font.pixelSize: 7
            }

            Label {
                text: root.mode.subtitle
                color: root.textColor
                font.family: "Noto Sans"
                font.pixelSize: 7
            }
        }

        Item {
            width: parent.width
            height: 66

            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                height: 1
                color: "#33404a"
            }

            Rectangle {
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                anchors.horizontalCenter: parent.horizontalCenter
                width: 1
                color: "#33404a"
            }

            Label {
                anchors.horizontalCenter: parent.horizontalCenter
                anchors.top: parent.top
                anchors.topMargin: 12
                horizontalAlignment: Text.AlignHCenter
                text: root.mode.hint
                color: "#5f7686"
                font.family: "Noto Sans"
                font.pixelSize: 7
            }

            Rectangle {
                x: root.pointX * (parent.width - width)
                y: root.pointY * (parent.height - height)
                width: 12
                height: 12
                radius: 2
                color: root.accentColor
                border.color: "#f4b09f"
            }

            MouseArea {
                anchors.fill: parent
                acceptedButtons: Qt.LeftButton | Qt.RightButton
                cursorShape: Qt.CrossCursor

                function updatePoint(mouseX, mouseY) {
                    root.pointX = root.clamp01(mouseX / width)
                    root.pointY = root.clamp01(mouseY / height)
                }

                onPressed: function(mouse) {
                    if (mouse.button === Qt.RightButton) {
                        root.modeIndex = (root.modeIndex + 1) % root.modes.length
                    } else {
                        updatePoint(mouse.x, mouse.y)
                    }
                }

                onPositionChanged: function(mouse) {
                    if (pressedButtons & Qt.LeftButton)
                        updatePoint(mouse.x, mouse.y)
                }
            }
        }

        Row {
            width: parent.width

            Label {
                text: "Lo"
                color: "#5f7686"
                font.family: "Noto Sans"
                font.pixelSize: 7
            }

            Item {
                width: parent.width - 24
                height: 1
            }

            Label {
                text: "Hi"
                color: "#5f7686"
                font.family: "Noto Sans"
                font.pixelSize: 7
            }
        }
    }
}
