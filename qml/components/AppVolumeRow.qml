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
        id: -1,
        iconColor: "#444444",
        iconText: "--",
        name: "",
        detail: "",
        muted: false,
        level: 0
    })
    property real liveLevel: Number(root.appData && root.appData.level !== undefined ? root.appData.level : 0)

    implicitHeight: 22

    onAppDataChanged: {
        if (!appSlider.pressed)
            liveLevel = Number(root.appData && root.appData.level !== undefined ? root.appData.level : 0)
    }

    Column {
        anchors.fill: parent
        spacing: 1

        Row {
            width: parent.width
            height: 10
            spacing: 4

            Rectangle {
                width: 10
                height: 10
                radius: 2
                color: root.safeAppData.iconColor
                border.color: "#424242"

                Label {
                    anchors.centerIn: parent
                    text: root.safeAppData.iconText
                    color: "#ece7e2"
                    font.family: "Noto Sans"
                    font.pixelSize: 5
                    font.weight: Font.DemiBold
                }
            }

            Label {
                width: parent.width - 14
                text: root.safeAppData.name.length > 0 ? root.safeAppData.name : root.safeAppData.detail
                color: "#d9d3ce"
                font.family: "Noto Sans"
                font.pixelSize: 8
                elide: Text.ElideRight
                verticalAlignment: Text.AlignVCenter
            }
        }

        Row {
            width: parent.width
            height: 11
            spacing: 3

            InlineSlider {
                id: appSlider
                width: parent.width - muteButton.width - parent.spacing
                anchors.verticalCenter: parent.verticalCenter
                value: root.liveLevel
                fillColor: "#7f9d88"
                onMoved: function(nextValue) {
                    root.liveLevel = nextValue
                }
                onReleased: function(nextValue) {
                    root.liveLevel = nextValue
                    root.controller.set_app_level(root.safeAppData.id, nextValue)
                }
            }

            Button {
                id: muteButton

                width: 14
                height: 11
                leftPadding: 0
                rightPadding: 0
                topPadding: 0
                bottomPadding: 0
                text: "M"
                onClicked: root.controller.toggle_app_muted(root.safeAppData.id)

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
