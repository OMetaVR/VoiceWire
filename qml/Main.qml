import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import VoiceWire
import "components"

ApplicationWindow {
    id: window

    width: 1360
    height: 720
    minimumWidth: 1360
    minimumHeight: 720
    maximumWidth: 1360
    maximumHeight: 720
    visible: true
    color: "#161616"
    title: "VoiceWire"

    onClosing: mixer.shutdown()

    Component.onDestruction: mixer.shutdown()

    MixerController {
        id: mixer
    }

    SettingsPanel {
        id: settingsPanel
        controller: mixer
        appRoutes: window.sessionState.appRoutes || []
        quickRouteEnabled: window.quickRouteEnabled
        onQuickRouteToggled: function(enabled) {
            mixer.set_quick_route_enabled(enabled)
        }
    }

    Timer {
        interval: 40
        running: window.visible
        repeat: true
        onTriggered: mixer.refresh_demo_levels()
    }

    Timer {
        interval: 600
        running: window.visible
        repeat: false
        onTriggered: mixer.refresh_app_routes()
    }

    property var sessionState: {
        try {
            return JSON.parse(mixer.state_json)
        } catch (error) {
            return { strips: [], buses: [] }
        }
    }

    property var meterState: {
        try {
            return JSON.parse(mixer.meter_json)
        } catch (error) {
            return { stripMeters: [], busMeters: [], appMeters: [] }
        }
    }
    readonly property bool quickRouteEnabled: Boolean(window.sessionState.quickRouteEnabled)

    readonly property var busLabels: ["A1", "A2", "A3", "B1", "B2", "B3"]
    readonly property int laneGap: 6

    QtObject {
        id: palette
        readonly property color background: "#161616"
        readonly property color surface: "#212121"
        readonly property color surfaceRaised: "#262626"
        readonly property color panelEdge: "#353535"
        readonly property color text: "#ece7e2"
        readonly property color mutedText: "#a49d95"
        readonly property color accent: "#7a9b86"
        readonly property color accentStrong: "#90b59c"
        readonly property color warning: "#b98e52"
        readonly property color danger: "#a15a52"
        readonly property color meterTrack: "#101010"
    }

    QtObject {
        id: quickRouteTracker

        property bool active: false
        property real dragX: 0
        property real dragY: 0
        property int appId: -1
        property string sourceTarget: ""
        property string hoveredTarget: ""
        property string iconText: "--"
        property string iconColor: "#444444"
        property string displayText: ""
        property var targets: ({})

        function beginDrag(appId, sourceTarget, iconText, iconColor, displayText, x, y) {
            active = true
            dragX = x
            dragY = y
            hoveredTarget = ""
            quickRouteTracker.appId = appId
            quickRouteTracker.sourceTarget = sourceTarget
            quickRouteTracker.iconText = iconText
            quickRouteTracker.iconColor = iconColor
            quickRouteTracker.displayText = displayText
            refreshHoveredTarget()
        }

        function updateDrag(x, y) {
            dragX = x
            dragY = y
            refreshHoveredTarget()
        }

        function setHoveredTarget(target) {
            hoveredTarget = target
        }

        function registerTarget(target, item) {
            if (!target || !item)
                return

            const next = Object.assign({}, targets)
            next[target] = { item: item }
            targets = next
            refreshHoveredTarget()
        }

        function unregisterTarget(target) {
            if (!targets[target])
                return

            const next = Object.assign({}, targets)
            delete next[target]
            targets = next
            refreshHoveredTarget()
        }

        function refreshHoveredTarget() {
            if (!active) {
                hoveredTarget = ""
                return
            }

            let nextTarget = ""
            for (const key in targets) {
                const rect = targets[key]
                if (!rect || !rect.item || key === sourceTarget)
                    continue

                const topLeft = rect.item.mapToItem(null, 0, 0)
                const width = rect.item.width
                const height = rect.item.height
                if (width <= 0 || height <= 0)
                    continue

                if (dragX >= topLeft.x && dragX <= topLeft.x + width
                    && dragY >= topLeft.y && dragY <= topLeft.y + height) {
                    nextTarget = key
                    break
                }
            }
            hoveredTarget = nextTarget
        }

        function finishDrag() {
            const nextTarget = hoveredTarget
            const nextAppId = appId
            const shouldRoute = active && nextAppId >= 0 && nextTarget.length > 0 && nextTarget !== sourceTarget
            cancelDrag()
            if (shouldRoute)
                mixer.set_app_route(nextAppId, nextTarget)
        }

        function cancelDrag() {
            active = false
            dragX = 0
            dragY = 0
            appId = -1
            sourceTarget = ""
            hoveredTarget = ""
            iconText = "--"
            iconColor = "#444444"
            displayText = ""
        }
    }

    Rectangle {
        anchors.fill: parent
        color: palette.background

        RowLayout {
            anchors.fill: parent
            anchors.margins: 8
            spacing: 8

            Rectangle {
                id: stripsPanel
                Layout.fillWidth: true
                Layout.fillHeight: true
                radius: 4
                color: palette.surface
                border.color: palette.panelEdge

                property int stripCount: (window.sessionState.strips || []).length
                property real columnWidth: Math.floor((stripViewport.width - Math.max(0, stripCount - 1) * window.laneGap) / Math.max(1, stripCount))

                ColumnLayout {
                    id: stripViewport
                    anchors.fill: parent
                    anchors.margins: 8
                    spacing: 8

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 12

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 28
                            color: "#1c1c1c"
                            border.color: palette.panelEdge
                            radius: 4

                            Label {
                                anchors.centerIn: parent
                                text: "Hardware Inputs"
                                color: palette.text
                                font.family: "Noto Sans"
                                font.pixelSize: 11
                                font.weight: Font.DemiBold
                            }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 28
                            color: "#1c1c1c"
                            border.color: palette.panelEdge
                            radius: 4

                            Label {
                                anchors.centerIn: parent
                                text: "Software Inputs"
                                color: palette.text
                                font.family: "Noto Sans"
                                font.pixelSize: 11
                                font.weight: Font.DemiBold
                            }
                        }
                    }

                    Row {
                        id: stripsRow
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        spacing: window.laneGap

                        Repeater {
                            model: window.sessionState.strips || []

                            Strip {
                                required property int index
                                required property var modelData

                                width: stripsPanel.columnWidth
                                height: stripsRow.height
                                stripIndex: index
                                stripData: modelData
                                meterData: (window.meterState.stripMeters || [])[index] || { left: 0, right: 0 }
                                appMeterData: (window.meterState.appMeters || [])[index] || []
                                controller: mixer
                                busLabels: window.busLabels
                                quickRouteEnabled: window.quickRouteEnabled
                                quickRouteState: quickRouteTracker
                            }
                        }
                    }
                }
            }

            Rectangle {
                id: busesPanel
                Layout.preferredWidth: 620
                Layout.fillHeight: true
                radius: 4
                color: palette.surface
                border.color: palette.panelEdge

                property int busCount: (window.sessionState.buses || []).length
                property real columnWidth: Math.floor((busViewport.width - Math.max(0, busCount - 1) * window.laneGap) / Math.max(1, busCount))

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 8
                    spacing: 8

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 0

                        Label {
                            text: "Master Section"
                            color: palette.text
                            opacity: 0.7
                            font.family: "Noto Sans"
                            font.pixelSize: 16
                            font.weight: Font.DemiBold
                            Layout.alignment: Qt.AlignVCenter
                        }

                        Item {
                            Layout.fillWidth: true
                        }

                        ToolButton {
                            id: settingsButton

                            Layout.preferredWidth: 28
                            Layout.preferredHeight: 28
                            padding: 0
                            hoverEnabled: true
                            onClicked: {
                                mixer.refresh_app_routes()
                                settingsPanel.open()
                            }

                            background: Rectangle {
                                radius: 4
                                color: settingsButton.down ? "#1c1c1c" : (settingsButton.hovered ? "#1a1a1a" : "transparent")
                                border.color: settingsButton.hovered ? palette.panelEdge : "transparent"
                            }

                            contentItem: Item {
                                Image {
                                    anchors.centerIn: parent
                                    width: 18
                                    height: 18
                                    source: "qrc:/qt/qml/VoiceWire/qml/assets/settings-configure-symbolic.svg"
                                    fillMode: Image.PreserveAspectFit
                                    smooth: true
                                }
                            }

                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 12

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 28
                            color: "#1c1c1c"
                            border.color: palette.panelEdge
                            radius: 4

                            Label {
                                anchors.centerIn: parent
                                text: "Hardware Outs"
                                color: palette.text
                                font.family: "Noto Sans"
                                font.pixelSize: 11
                                font.weight: Font.DemiBold
                            }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 28
                            color: "#1c1c1c"
                            border.color: palette.panelEdge
                            radius: 4

                            Label {
                                anchors.centerIn: parent
                                text: "Software Outs"
                                color: palette.text
                                font.family: "Noto Sans"
                                font.pixelSize: 11
                                font.weight: Font.DemiBold
                            }
                        }
                    }

                    Item {
                        id: busViewport
                        Layout.fillWidth: true
                        Layout.fillHeight: true

                        Row {
                            id: busRow
                            anchors.fill: parent
                            spacing: window.laneGap

                            Repeater {
                                model: window.sessionState.buses || []

                                Bus {
                                    required property int index
                                    required property var modelData

                                    width: busesPanel.columnWidth
                                    height: busViewport.height
                                    busIndex: index
                                    busData: modelData
                                    meterData: (window.meterState.busMeters || [])[index] || { left: 0, right: 0 }
                                    controller: mixer
                                }
                            }
                        }
                    }
                }
            }
        }

        Rectangle {
            visible: quickRouteTracker.active
            x: quickRouteTracker.dragX + 12
            y: quickRouteTracker.dragY + 12
            z: 100
            width: Math.min(196, previewLabel.implicitWidth + 30)
            height: 24
            radius: 4
            color: "#202020"
            border.color: "#3a3a3a"
            clip: true

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 8
                anchors.rightMargin: 8
                spacing: 6

                Rectangle {
                    Layout.preferredWidth: 12
                    Layout.preferredHeight: 12
                    radius: 2
                    color: quickRouteTracker.iconColor
                    border.color: "#424242"

                    Label {
                        anchors.centerIn: parent
                        text: quickRouteTracker.iconText
                        color: "#ece7e2"
                        font.family: "Noto Sans"
                        font.pixelSize: 6
                        font.weight: Font.DemiBold
                    }
                }

                Label {
                    id: previewLabel

                    Layout.fillWidth: true
                    text: quickRouteTracker.displayText
                    color: "#ece7e2"
                    font.family: "Noto Sans"
                    font.pixelSize: 9
                    elide: Text.ElideRight
                    verticalAlignment: Text.AlignVCenter
                }
            }
        }

    }
}
