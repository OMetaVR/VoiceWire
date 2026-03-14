import QtQuick

Rectangle {
    id: root

    required property real leftLevel
    required property real rightLevel
    property color fillColor: "#c7e9e9"
    property color trackColor: "#0f0f0f"
    property real displayLeft: Math.max(0, Math.min(1, leftLevel))
    property real displayRight: Math.max(0, Math.min(1, rightLevel))
    property bool animated: true
    property real phase: 0

    implicitWidth: 20
    implicitHeight: 320
    color: root.trackColor
    border.color: "#2f3132"
    border.width: 1
    radius: 2
    clip: true

    Rectangle {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: 1
        color: "#32373a"
    }

    Rectangle {
        anchors.left: parent.left
        anchors.leftMargin: 1
        anchors.bottom: parent.bottom
        width: Math.floor((parent.width - 3) / 2)
        height: Math.max(2, (parent.height - 2) * root.displayLeft)
        color: root.fillColor
    }

    Rectangle {
        anchors.right: parent.right
        anchors.rightMargin: 1
        anchors.bottom: parent.bottom
        width: Math.floor((parent.width - 3) / 2)
        height: Math.max(2, (parent.height - 2) * root.displayRight)
        color: root.fillColor
    }

    Repeater {
        model: 26

        Rectangle {
            x: 1
            y: 2 + index * ((root.height - 4) / 26)
            width: root.width - 2
            height: 1
            color: index % 2 === 0 ? "#201f1f" : "#2a2929"
            opacity: 0.85
        }
    }

    Timer {
        interval: 80
        repeat: true
        running: root.animated
        onTriggered: {
            root.phase += 0.33
            const wobbleA = 0.9 + Math.sin(root.phase) * 0.08
            const wobbleB = 0.9 + Math.sin(root.phase + 0.55) * 0.08
            root.displayLeft = Math.max(0, Math.min(1, root.leftLevel * wobbleA))
            root.displayRight = Math.max(0, Math.min(1, root.rightLevel * wobbleB))
        }
    }

    onLeftLevelChanged: {
        if (!animated)
            root.displayLeft = Math.max(0, Math.min(1, root.leftLevel))
    }

    onRightLevelChanged: {
        if (!animated)
            root.displayRight = Math.max(0, Math.min(1, root.rightLevel))
    }
}
