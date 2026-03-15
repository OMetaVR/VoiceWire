import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "."

Item {
    id: root

    required property int stripIndex
    required property var stripData
    required property var controller
    required property bool quickRouteEnabled
    property var quickRouteState: null
    required property string routeTargetLabel

    property bool expandedApps: false
    readonly property int compactCount: 3
    readonly property bool canExpand: (root.stripData.apps || []).length > root.compactCount
    readonly property bool canAcceptQuickRoute: root.quickRouteEnabled
        && root.quickRouteState
        && root.quickRouteState.active
        && root.quickRouteState.appId >= 0
        && root.quickRouteState.sourceTarget !== root.routeTargetLabel
    readonly property bool routeDropActive: root.canAcceptQuickRoute
        && root.quickRouteState
        && root.quickRouteState.hoveredTarget === root.routeTargetLabel
    readonly property bool routeDropAvailable: root.canAcceptQuickRoute
    readonly property var visibleApps: {
        const apps = root.stripData.apps || []
        return root.expandedApps ? apps : apps.slice(0, root.compactCount)
    }

    property real panX: 0.5
    property real panY: 0.5

    implicitHeight: 269

    function syncQuickRouteTarget() {
        if (!root.quickRouteState || root.routeTargetLabel.length === 0)
            return

        root.quickRouteState.registerTarget(root.routeTargetLabel, appArea)
    }

    Component.onCompleted: syncQuickRouteTarget()
    onQuickRouteStateChanged: syncQuickRouteTarget()
    onRouteTargetLabelChanged: syncQuickRouteTarget()
    Component.onDestruction: {
        if (root.quickRouteState)
            root.quickRouteState.unregisterTarget(root.routeTargetLabel)
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 5

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 0
            visible: !root.expandedApps

            Label {
                Layout.fillWidth: true
                horizontalAlignment: Text.AlignHCenter
                text: root.stripData.headerName
                color: "#d8d3cf"
                font.family: "Noto Sans"
                font.pixelSize: 10
            }

            Label {
                Layout.fillWidth: true
                horizontalAlignment: Text.AlignHCenter
                text: root.stripData.headerDetail
                color: "#8c98a2"
                font.family: "Noto Sans"
                font.pixelSize: 8
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.preferredHeight: 126
            visible: !root.expandedApps

            ColumnLayout {
                anchors.fill: parent
                spacing: 4

                Label {
                    Layout.alignment: Qt.AlignHCenter
                    text: "EQUALIZER"
                    color: "#8c98a2"
                    font.family: "Noto Sans"
                    font.pixelSize: 8
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 4

                    MiniKnob {
                        Layout.fillWidth: true
                        label: ""
                        from: -12.0
                        to: 12.0
                        defaultValue: 0.0
                        value: 0.0
                    }

                    MiniKnob {
                        Layout.fillWidth: true
                        label: "Treble"
                        from: -12.0
                        to: 12.0
                        defaultValue: 0.0
                        value: 0.0
                    }

                    MiniKnob {
                        Layout.fillWidth: true
                        label: "Bass"
                        from: -12.0
                        to: 12.0
                        defaultValue: 0.0
                        value: 0.0
                    }
                }

                Rectangle {
                    Layout.alignment: Qt.AlignHCenter
                    Layout.preferredWidth: 68
                    Layout.preferredHeight: 68
                    color: "#31414b"
                    border.color: "#5a6871"
                    radius: 2

                    function clamp01(value) {
                        return Math.max(0, Math.min(1, value))
                    }

                    Rectangle {
                        anchors.horizontalCenter: parent.horizontalCenter
                        anchors.top: parent.top
                        anchors.bottom: parent.bottom
                        width: 1
                        color: "#72828c"
                    }

                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.right: parent.right
                        height: 1
                        color: "#72828c"
                    }

                    Label {
                        anchors.left: parent.left
                        anchors.leftMargin: 4
                        anchors.verticalCenter: parent.verticalCenter
                        text: "L"
                        color: "#c0cbd1"
                        font.family: "Noto Sans"
                        font.pixelSize: 7
                    }

                    Label {
                        anchors.right: parent.right
                        anchors.rightMargin: 4
                        anchors.verticalCenter: parent.verticalCenter
                        text: "R"
                        color: "#c0cbd1"
                        font.family: "Noto Sans"
                        font.pixelSize: 7
                    }

                    Label {
                        anchors.horizontalCenter: parent.horizontalCenter
                        anchors.top: parent.top
                        anchors.topMargin: 3
                        text: "Front"
                        color: "#c0cbd1"
                        font.family: "Noto Sans"
                        font.pixelSize: 6
                    }

                    Label {
                        anchors.horizontalCenter: parent.horizontalCenter
                        anchors.bottom: parent.bottom
                        anchors.bottomMargin: 3
                        text: "Rear"
                        color: "#c0cbd1"
                        font.family: "Noto Sans"
                        font.pixelSize: 6
                    }

                    Rectangle {
                        width: 8
                        height: 8
                        radius: 4
                        x: root.panX * (parent.width - width)
                        y: root.panY * (parent.height - height)
                        color: "#d66758"
                        border.color: "#f5b1a0"
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.CrossCursor

                        function updatePoint(mouseX, mouseY) {
                            root.panX = parent.clamp01(mouseX / width)
                            root.panY = parent.clamp01(mouseY / height)
                        }

                        onPressed: function(mouse) {
                            updatePoint(mouse.x, mouse.y)
                        }

                        onPositionChanged: function(mouse) {
                            if (pressed)
                                updatePoint(mouse.x, mouse.y)
                        }
                    }
                }
            }
        }

        Rectangle {
            id: appArea

            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.topMargin: 14
            color: root.routeDropActive ? "#1f2521" : "transparent"
            border.color: root.routeDropAvailable ? (root.routeDropActive ? "#90b59c" : "#444444") : "transparent"
            radius: 4
            clip: true

            Flickable {
                anchors.fill: parent
                contentWidth: width
                contentHeight: appColumn.height
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                interactive: root.expandedApps && contentHeight > height

                Column {
                    id: appColumn
                    width: parent.width
                    spacing: 4

                    Repeater {
                        model: root.visibleApps || []

                        AppVolumeRow {
                            required property var modelData

                            width: appColumn.width
                            stripIndex: root.stripIndex
                            appIndex: index
                            appData: modelData
                            controller: root.controller
                            quickRouteEnabled: root.quickRouteEnabled
                            quickRouteState: root.quickRouteState
                            routeTargetLabel: root.routeTargetLabel
                        }
                    }
                }
            }

            Label {
                anchors.centerIn: parent
                visible: root.routeDropActive
                text: "Move to " + root.routeTargetLabel
                color: "#ece7e2"
                font.family: "Noto Sans"
                font.pixelSize: 10
                font.weight: Font.DemiBold
            }

            TapHandler {
                acceptedButtons: Qt.RightButton
                onTapped: {
                    if (root.canExpand)
                        root.expandedApps = !root.expandedApps
                }
            }
        }
    }
}
