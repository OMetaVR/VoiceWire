import QtQuick

Rectangle {
    id: root

    required property real level
    required property color fillColor
    required property color trackColor
    property bool animated: true
    property real displayLevel: Math.max(0, Math.min(1, root.level))
    property real phase: 0

    radius: 4
    color: root.trackColor
    border.color: "#303030"
    border.width: 1
    clip: true

    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: Math.max(0, parent.height * root.displayLevel)
        color: root.fillColor
        radius: 3

        Behavior on height {
            NumberAnimation {
                duration: 90
                easing.type: Easing.OutCubic
            }
        }
    }

    Timer {
        interval: 90
        repeat: true
        running: root.animated
        onTriggered: {
            root.phase += 0.34
            const base = Math.max(0, Math.min(1, root.level))
            if (base <= 0.001) {
                root.displayLevel = 0
                return
            }
            const wobble = 0.86 + Math.sin(root.phase) * 0.14
            root.displayLevel = Math.max(0, Math.min(1, base * wobble))
        }
    }

    onLevelChanged: {
        if (!animated)
            root.displayLevel = Math.max(0, Math.min(1, root.level))
    }
}
