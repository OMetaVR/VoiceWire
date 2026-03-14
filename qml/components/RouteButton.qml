import QtQuick
import QtQuick.Controls

Button {
    id: root

    required property bool active

    implicitWidth: 36
    implicitHeight: 22

    font.family: "Noto Sans"
    font.pixelSize: 10
    font.weight: Font.DemiBold

    contentItem: Label {
        text: root.text
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        color: root.active ? "#171717" : "#a49d95"
        font: root.font
    }

    background: Rectangle {
        radius: 4
        color: root.active ? "#90b59c" : "#202020"
        border.color: root.active ? "#7a9b86" : "#353535"
    }
}
