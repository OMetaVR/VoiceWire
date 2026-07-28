import QtQuick

Rectangle {
    id: root

    required property real leftLevel
    required property real rightLevel
    property color fillColor: "#6fa67e"
    property color trackColor: "#0f0f0f"
    property color warningColor: "#b35a54"
    property real warningThreshold: 0.78
    property real displayLeft: Math.max(0, Math.min(1, leftLevel))
    property real displayRight: Math.max(0, Math.min(1, rightLevel))
    readonly property real contentHeight: Math.max(0, height - 2)
    readonly property real warningHeight: contentHeight * warningThreshold

    Behavior on displayLeft {
        NumberAnimation {
            duration: 45
            easing.type: Easing.OutQuad
        }
    }

    Behavior on displayRight {
        NumberAnimation {
            duration: 45
            easing.type: Easing.OutQuad
        }
    }

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
        height: Math.max(0, Math.min(root.warningHeight, root.contentHeight * root.displayLeft))
        color: root.fillColor
    }

    Rectangle {
        anchors.left: parent.left
        anchors.leftMargin: 1
        anchors.bottom: parent.bottom
        anchors.bottomMargin: Math.min(root.warningHeight, root.contentHeight * root.displayLeft)
        width: Math.floor((parent.width - 3) / 2)
        height: Math.max(0, root.contentHeight * root.displayLeft - root.warningHeight)
        color: root.warningColor
    }

    Rectangle {
        anchors.right: parent.right
        anchors.rightMargin: 1
        anchors.bottom: parent.bottom
        width: Math.floor((parent.width - 3) / 2)
        height: Math.max(0, Math.min(root.warningHeight, root.contentHeight * root.displayRight))
        color: root.fillColor
    }

    Rectangle {
        anchors.right: parent.right
        anchors.rightMargin: 1
        anchors.bottom: parent.bottom
        anchors.bottomMargin: Math.min(root.warningHeight, root.contentHeight * root.displayRight)
        width: Math.floor((parent.width - 3) / 2)
        height: Math.max(0, root.contentHeight * root.displayRight - root.warningHeight)
        color: root.warningColor
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

    onLeftLevelChanged: root.displayLeft = Math.max(0, Math.min(1, root.leftLevel))

    onRightLevelChanged: root.displayRight = Math.max(0, Math.min(1, root.rightLevel))
}
