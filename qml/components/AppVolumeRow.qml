import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "."

Item {
    id: root

    required property int stripIndex
    required property int appIndex
    required property var appData
    required property var controller

    readonly property var safeAppData: root.appData || ({
        iconColor: "#444444",
        iconText: "--",
        name: "",
        muted: false,
        level: 0
    })
    property real liveLevel: Number(root.appData && root.appData.level !== undefined ? root.appData.level : 0)

    implicitHeight: 22

    onAppDataChanged: liveLevel = Number(root.appData && root.appData.level !== undefined ? root.appData.level : 0)

    ColumnLayout {
        anchors.fill: parent
        spacing: 2

        RowLayout {
            Layout.fillWidth: true
            spacing: 4

            Rectangle {
                Layout.preferredWidth: 12
                Layout.preferredHeight: 12
                radius: 2
                color: root.safeAppData.iconColor
                border.color: "#424242"

                Label {
                    anchors.centerIn: parent
                    text: root.safeAppData.iconText
                    color: "#ece7e2"
                    font.family: "Noto Sans"
                    font.pixelSize: 6
                    font.weight: Font.DemiBold
                }
            }

            Label {
                Layout.fillWidth: true
                text: root.safeAppData.name
                color: "#d9d3ce"
                font.family: "Noto Sans"
                font.pixelSize: 8
                clip: true
                elide: Text.ElideNone
                verticalAlignment: Text.AlignVCenter
                maximumLineCount: 1
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 3

            InlineSlider {
                Layout.fillWidth: true
                value: root.liveLevel
                fillColor: "#7f9d88"
                onMoved: function(nextValue) {
                    root.liveLevel = nextValue
                }
                onReleased: function(nextValue) {
                    root.liveLevel = nextValue
                    root.controller.set_virtual_app_level(root.stripIndex, root.appIndex, nextValue)
                }
            }

            Button {
                id: muteButton

                Layout.preferredWidth: 14
                Layout.preferredHeight: 12
                leftPadding: 0
                rightPadding: 0
                topPadding: 0
                bottomPadding: 0
                text: "M"
                onClicked: root.controller.toggle_virtual_app_muted(root.stripIndex, root.appIndex)

                contentItem: Label {
                    anchors.fill: parent
                    text: muteButton.text
                    color: root.safeAppData.muted ? "#171717" : "#d0ccc7"
                    font.family: "Noto Sans"
                    font.pixelSize: 7
                    font.weight: Font.DemiBold
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                background: Rectangle {
                    color: root.safeAppData.muted ? "#a15a52" : "#232323"
                    border.color: "#3a3a3a"
                    radius: 2
                }
            }
        }
    }
}
