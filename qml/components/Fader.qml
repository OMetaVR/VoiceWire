import QtQuick
import QtQuick.Controls

Item {
    id: root

    required property real from
    required property real to
    required property real value
    required property color fillColor
    property color trackColor: "#111111"
    property color borderColor: "#353535"
    property color handleColor: "#d7cdb9"
    property color handleBorderColor: "#8f8575"
    property real stepSize: 0.5
    property real defaultValue: 0.0
    property bool pressed: dragArea.pressed
    signal moved(real value)
    signal released(real value)

    implicitWidth: 44
    implicitHeight: 208

    readonly property real usableHeight: Math.max(1, height - bubble.height)
    readonly property color activeFillColor: root.value > 0 ? "#b35a54" : root.fillColor
    readonly property real normalizedValue: {
        const span = root.to - root.from
        if (Math.abs(span) < 0.00001)
            return 0
        return Math.max(0, Math.min(1, (root.value - root.from) / span))
    }
    readonly property string valueLabel: {
        const rounded = Number(root.value).toFixed(1)
        return (root.value > 0 ? "+" : "") + rounded + " dB"
    }

    function clampValue(nextValue) {
        return Math.max(root.from, Math.min(root.to, nextValue))
    }

    function snappedValue(nextValue) {
        if (root.stepSize <= 0)
            return clampValue(nextValue)
        const steps = Math.round((nextValue - root.from) / root.stepSize)
        return clampValue(root.from + steps * root.stepSize)
    }

    function valueFromY(yPos) {
        const ratio = 1.0 - Math.max(0, Math.min(1, yPos / height))
        return snappedValue(root.from + ratio * (root.to - root.from))
    }

    Rectangle {
        id: track

        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: 10
        radius: 5
        color: root.trackColor
        border.color: root.borderColor
        border.width: 1

        Rectangle {
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: parent.bottom
            width: 10
            height: Math.max(10, parent.height - (bubble.y + bubble.height / 2))
            radius: 5
            color: root.activeFillColor
        }
    }

    Rectangle {
        id: bubble

        width: 36
        height: 36
        radius: 18
        color: root.activeFillColor
        border.color: root.activeFillColor
        border.width: 2
        x: Math.round((root.width - width) / 2)
        y: (1.0 - root.normalizedValue) * root.usableHeight

        Label {
            anchors.fill: parent
            anchors.topMargin: 1
            text: root.valueLabel
            color: "#243227"
            font.family: "Noto Sans"
            font.pixelSize: 9
            font.weight: Font.DemiBold
            opacity: 0.4
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }

        Label {
            anchors.fill: parent
            text: root.valueLabel
            color: "#f3f7f3"
            font.family: "Noto Sans"
            font.pixelSize: 9
            font.weight: Font.DemiBold
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }
    }

    MouseArea {
        id: dragArea

        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor

        function applyPosition(mouseY) {
            const nextValue = root.valueFromY(mouseY)
            root.moved(nextValue)
        }

        onPressed: function(mouse) {
            applyPosition(mouse.y)
        }
        onPositionChanged: function(mouse) {
            if (pressed)
                applyPosition(mouse.y)
        }
        onReleased: function(mouse) {
            const nextValue = root.valueFromY(mouse.y)
            root.moved(nextValue)
            root.released(nextValue)
        }
        onDoubleClicked: {
            const nextValue = root.snappedValue(root.defaultValue)
            root.moved(nextValue)
            root.released(nextValue)
        }
    }
}
