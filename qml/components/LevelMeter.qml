import QtQuick

Rectangle {
    id: root

    required property real level
    required property color fillColor
    required property color trackColor
    property color warningColor: "#b35a54"
    property real warningThreshold: 0.78
    property real displayLevel: Math.max(0, Math.min(1, root.level))
    readonly property real warningHeight: height * warningThreshold

    radius: 4
    color: root.trackColor
    border.color: "#303030"
    border.width: 1
    clip: true

    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: Math.max(0, Math.min(root.warningHeight, parent.height * root.displayLevel))
        color: root.fillColor
        radius: 3

        Behavior on height {
            NumberAnimation {
                duration: 90
                easing.type: Easing.OutCubic
            }
        }
    }

    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.bottomMargin: Math.min(root.warningHeight, parent.height * root.displayLevel)
        height: Math.max(0, parent.height * root.displayLevel - root.warningHeight)
        color: root.warningColor
        radius: 0

        Behavior on height {
            NumberAnimation {
                duration: 45
                easing.type: Easing.OutCubic
            }
        }
    }

    onLevelChanged: root.displayLevel = Math.max(0, Math.min(1, root.level))
}
