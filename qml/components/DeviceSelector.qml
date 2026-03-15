import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: selectorRoot

    required property var options
    required property string currentValue
    property string placeholderText: "Select Device"
    signal picked(string value)
    property bool hovered: false
    readonly property string displayValue: selectorRoot.currentValue.length > 0 ? selectorRoot.currentValue : selectorRoot.placeholderText
    property real popupWidth: 220

    implicitHeight: 28
    radius: 4
    color: "#1d1d1d"
    border.color: "#353535"

    function recomputePopupWidth() {
        let widest = Math.max(220, metrics.tightBoundingRect.width + 28)
        const list = selectorRoot.options || []
        for (let i = 0; i < list.length; i += 1) {
            metrics.text = list[i].name
            widest = Math.max(widest, metrics.tightBoundingRect.width + 28)
        }
        selectorRoot.popupWidth = widest
    }

    function togglePopup() {
        if (selectorPopup.opened)
            selectorPopup.close()
        else
            selectorPopup.open()
    }

    function closePopup() {
        selectorPopup.close()
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
            text: selectorRoot.displayValue
            color: selectorRoot.currentValue.length > 0 ? "#ece7e2" : "#9d978f"
            font.family: "Noto Sans"
            font.pixelSize: 10
            clip: true
            elide: Text.ElideNone
            verticalAlignment: Text.AlignVCenter
        }
    }

    TapHandler {
        onTapped: selectorRoot.togglePopup()
    }

    HoverHandler {
        onHoveredChanged: selectorRoot.hovered = hovered
    }

    TextMetrics {
        id: metrics
        text: selectorRoot.displayValue
        font.family: "Noto Sans"
        font.pixelSize: 11
    }

    Popup {
        id: selectorPopup

        y: selectorRoot.height + 4
        width: selectorRoot.popupWidth
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
                model: selectorRoot.options || []

                Rectangle {
                    required property var modelData

                    width: selectorPopup.availableWidth
                    height: 30
                    radius: 3
                    color: modelData.name === selectorRoot.currentValue ? "#2a2a2a" : "#202020"
                    border.color: modelData.name === selectorRoot.currentValue ? "#7a9b86" : "#353535"

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
                            selectorRoot.picked(modelData.name)
                            selectorRoot.closePopup()
                        }
                    }
                }
            }
        }
    }
}
