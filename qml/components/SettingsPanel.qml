import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "."

Popup {
    id: root

    required property var controller
    required property var appRoutes

    parent: Overlay.overlay
    anchors.centerIn: parent
    width: 1216
    height: 664
    padding: 0
    modal: true
    focus: true
    closePolicy: Popup.CloseOnEscape

    onOpened: root.controller.refresh_app_routes()

    Timer {
        interval: 1000
        running: root.opened
        repeat: true
        onTriggered: root.controller.refresh_app_routes()
    }

    background: Rectangle {
        color: "#151515"
        border.color: "#343434"
        radius: 6
    }

    Overlay.modal: Rectangle {
        color: "#80000000"
    }

    contentItem: RowLayout {
        spacing: 0

        Rectangle {
            Layout.preferredWidth: 228
            Layout.fillHeight: true
            color: "#111111"
            border.color: "#2d2d2d"

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 10

                Label {
                    text: "Settings"
                    color: "#ece7e2"
                    font.family: "Noto Sans"
                    font.pixelSize: 16
                    font.weight: Font.DemiBold
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 32
                    radius: 4
                    color: "#252525"
                    border.color: "#353535"

                    Label {
                        anchors.fill: parent
                        anchors.leftMargin: 10
                        anchors.rightMargin: 10
                        text: "App Routing"
                        color: "#ece7e2"
                        font.family: "Noto Sans"
                        font.pixelSize: 11
                        verticalAlignment: Text.AlignVCenter
                    }
                }

                Item {
                    Layout.fillHeight: true
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: "#181818"
            border.color: "#2d2d2d"

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 18
                spacing: 14

                RowLayout {
                    Layout.fillWidth: true

                    Label {
                        text: "App Routing"
                        color: "#ece7e2"
                        font.family: "Noto Sans"
                        font.pixelSize: 15
                        font.weight: Font.DemiBold
                    }

                    Item {
                        Layout.fillWidth: true
                    }

                    ToolButton {
                        id: closeButton

                        Layout.preferredWidth: 28
                        Layout.preferredHeight: 28
                        padding: 0
                        onClicked: root.close()

                        background: Rectangle {
                            radius: 4
                            color: closeButton.down ? "#1d1d1d" : (closeButton.hovered ? "#1a1a1a" : "transparent")
                            border.color: closeButton.hovered ? "#353535" : "transparent"
                        }

                        contentItem: Label {
                            text: "\u00d7"
                            color: "#d4cec8"
                            font.family: "Noto Sans"
                            font.pixelSize: 16
                            horizontalAlignment: Text.AlignHCenter
                            verticalAlignment: Text.AlignVCenter
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: "#2b2b2b"
                }

                ScrollView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true

                    ColumnLayout {
                        width: parent ? parent.width : 0
                        spacing: 8

                        Repeater {
                            model: root.appRoutes || []

                            AppRoutingRow {
                                required property var modelData

                                Layout.fillWidth: true
                                routeData: modelData
                                controller: root.controller
                            }
                        }

                        Label {
                            Layout.fillWidth: true
                            visible: (root.appRoutes || []).length === 0
                            text: "No active app playback streams found."
                            color: "#a49d95"
                            font.family: "Noto Sans"
                            font.pixelSize: 11
                            horizontalAlignment: Text.AlignHCenter
                            topPadding: 16
                        }
                    }
                }
            }
        }
    }
}
