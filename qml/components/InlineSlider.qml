import QtQuick

Item {
    id: root

    required property real value
    property alias pressed: dragArea.pressed
    property real from: 0.0
    property real to: 1.0
    property color fillColor: "#8bc39f"
    property color trackColor: "#1b1b1b"
    property color borderColor: "#353535"
    property real stepSize: 0.01
    signal moved(real value)
    signal released(real value)

    implicitWidth: 96
    implicitHeight: 4

    readonly property real normalizedValue: {
        const span = root.to - root.from
        if (Math.abs(span) < 0.00001)
            return 0
        return Math.max(0, Math.min(1, (root.value - root.from) / span))
    }

    function clamp(nextValue) {
        return Math.max(root.from, Math.min(root.to, nextValue))
    }

    function snapped(nextValue) {
        if (root.stepSize <= 0)
            return clamp(nextValue)
        const steps = Math.round((nextValue - root.from) / root.stepSize)
        return clamp(root.from + steps * root.stepSize)
    }

    function valueFromX(xPos) {
        const ratio = Math.max(0, Math.min(1, xPos / width))
        return snapped(root.from + ratio * (root.to - root.from))
    }

    Rectangle {
        anchors.fill: parent
        radius: 2
        color: root.trackColor
        border.color: root.borderColor

        Rectangle {
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            width: Math.max(2, parent.width * root.normalizedValue)
            color: root.fillColor
            radius: 0
        }
    }

    Rectangle {
        width: 3
        height: root.height + 4
        color: "#d8ddd9"
        border.color: "#8f9a93"
        radius: 1
        x: Math.round(root.normalizedValue * (root.width - width))
        y: -2
    }

    MouseArea {
        id: dragArea
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor

        function applyAt(mouseX) {
            root.moved(root.valueFromX(mouseX))
        }

        onPressed: function(mouse) {
            applyAt(mouse.x)
        }
        onPositionChanged: function(mouse) {
            if (pressed)
                applyAt(mouse.x)
        }
        onReleased: root.released(root.value)
    }
}
