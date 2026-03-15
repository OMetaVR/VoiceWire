import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "."

Rectangle {
    id: root

    required property var routeData
    required property var controller

    readonly property var routeOptions: [
        { "name": "VAIO" },
        { "name": "AUX" },
        { "name": "SYS" }
    ]

    color: "transparent"
    border.color: "#2d2d2d"
    radius: 4
    implicitHeight: 58

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 12
        anchors.rightMargin: 12
        spacing: 12

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2

            Label {
                Layout.fillWidth: true
                text: root.routeData.name || ""
                color: "#ece7e2"
                font.family: "Noto Sans"
                font.pixelSize: 12
                font.weight: Font.DemiBold
                elide: Text.ElideRight
            }

            Label {
                Layout.fillWidth: true
                visible: text.length > 0
                text: root.routeData.detail || ""
                color: "#a49d95"
                font.family: "Noto Sans"
                font.pixelSize: 10
                elide: Text.ElideRight
            }
        }

        DeviceSelector {
            Layout.preferredWidth: 156
            options: root.routeOptions
            currentValue: root.routeData.target || ""
            placeholderText: "Select Route"
            onPicked: function(value) {
                root.controller.set_app_route(root.routeData.id, value)
            }
        }
    }
}
