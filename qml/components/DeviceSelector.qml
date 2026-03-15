import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root

    required property var options
    required property string currentValue
    property string placeholderText: "Select Device"
    signal picked(string value)
    property bool hovered: false
    readonly property string displayValue: root.currentValue.length > 0 ? root.currentValue : root.placeholderText
    property real popupWidth: 220

    implicitHeight: 28
    radius: 4
    color: "#1d1d1d"
    border.color: "#353535"

    function recomputePopupWidth() {
        let widest = Math.max(220, metrics.tightBoundingRect.width + 28)
        const list = root.options || []
        for (let i = 0; i < list.length; i += 1) {
            metrics.text = list[i].name
            widest = Math.max(widest, metrics.tightBoundingRect.width + 28)
        }
        root.popupWidth = widest
    }

    Component.onCompleted: recomputePopupWidth()
    onOptionsChanged: recomputePopupWidth()
    onCurrentValueChanged: recomputePopupWidth()

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 10
        anchors.rightMargin: 10
        spacing: 0

        Label {
            Layout.fillWidth: true
            text: root.displayValue
            color: root.currentValue.length > 0 ? "#ece7e2" : "#9d978f"
            font.family: "Noto Sans"
            font.pixelSize: 10
            clip: true
            elide: Text.ElideNone
            verticalAlignment: Text.AlignVCenter
        }
    }

    TapHandler {
        onTapped: {
            if (popup.opened)
                popup.close()
            else
                popup.open()
        }
    }

    HoverHandler {
        onHoveredChanged: root.hovered = hovered
    }

    ToolTip.visible: root.hovered && root.currentValue.length > 0
    ToolTip.text: root.currentValue
    ToolTip.delay: 300

    TextMetrics {
        id: metrics
        text: root.displayValue
        font.family: "Noto Sans"
        font.pixelSize: 11
    }

    Popup {
        id: popup

        y: root.height + 4
        width: root.popupWidth
        padding: 4
        modal: false
        focus: true
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
        background: Rectangle {
            radius: 4
            color: "#202020"
            border.color: "#353535"
        }

        contentItem: Column {
            spacing: 4

            Repeater {
                model: root.options || []

                Rectangle {
                    required property var modelData

                    width: popup.availableWidth
                    height: 30
                    radius: 3
                    color: modelData.name === root.currentValue ? "#2a2a2a" : "#202020"
                    border.color: modelData.name === root.currentValue ? "#7a9b86" : "#353535"

                    Label {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: 10
                        anchors.right: parent.right
                        anchors.rightMargin: 10
                        text: modelData.name
                        color: "#ece7e2"
                        font.family: "Noto Sans"
                        font.pixelSize: 10
                        clip: true
                        elide: Text.ElideNone
                    }

                    TapHandler {
                        onTapped: {
                            root.picked(modelData.name)
                            popup.close()
                        }
                    }
                }
            }
        }
    }
}
