import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "."

Item {
    id: root

    required property int stripIndex
    required property int appIndex
    required property var appData
    required property var controller
    required property bool quickRouteEnabled
    property var quickRouteState: null
    required property string routeTargetLabel

    readonly property var safeAppData: root.appData || ({
        id: -1,
        iconColor: "#444444",
        iconText: "--",
        name: "",
        detail: "",
        muted: false,
        level: 0
    })
    readonly property bool canDragRoute: root.quickRouteEnabled && root.safeAppData.id >= 0 && root.routeTargetLabel.length > 0
    readonly property string displayText: {
        const name = String(root.safeAppData.name || "")
        const detail = String(root.safeAppData.detail || "")
        if (name.length > 0 && detail.length > 0)
            return name + " - " + detail
        return name.length > 0 ? name : detail
    }
    readonly property bool dragInProgress: Boolean(root.quickRouteState
        && root.quickRouteState.active
        && root.quickRouteState.appId === root.safeAppData.id
        && root.quickRouteState.sourceTarget === root.routeTargetLabel)
    readonly property bool marqueeRunning: dragHover.hovered
        && !root.dragInProgress
        && titleTextMetrics.tightBoundingRect.width > titleViewport.width
    property real marqueeOffset: 0
    property real liveLevel: Number(root.appData && root.appData.level !== undefined ? root.appData.level : 0)
    property bool levelCommitPending: false
    property real pendingLevelValue: liveLevel
    property bool previewDirty: false
    property real previewLevelValue: liveLevel

    implicitHeight: 22

    onAppDataChanged: {
        const modelValue = Number(root.appData && root.appData.level !== undefined ? root.appData.level : 0)
        if (levelCommitPending) {
            if (Math.abs(modelValue - pendingLevelValue) <= 0.01) {
                levelCommitPending = false
                liveLevel = modelValue
            }
            return
        }

        if (!appSlider.pressed)
            liveLevel = modelValue
    }

    onMarqueeRunningChanged: {
        if (!marqueeRunning)
            marqueeOffset = 0
    }

    Timer {
        interval: 16
        repeat: true
        running: appSlider.pressed
        onTriggered: {
            if (!root.previewDirty)
                return

            root.previewDirty = false
            root.controller.preview_app_level(root.safeAppData.id, root.previewLevelValue)
        }
    }

    Column {
        anchors.fill: parent
        spacing: 1

        Rectangle {
            id: dragHandle

            width: parent.width
            height: 10
            radius: 2
            color: root.canDragRoute ? (root.dragInProgress ? "#2a332d" : (dragHover.hovered ? "#202020" : "transparent")) : "transparent"
            border.color: root.dragInProgress ? "#7a9b86" : "transparent"

            HoverHandler {
                id: dragHover

                cursorShape: root.canDragRoute ? (root.dragInProgress ? Qt.ClosedHandCursor : Qt.OpenHandCursor) : Qt.ArrowCursor
            }

            DragHandler {
                id: routeDrag

                target: null
                enabled: root.canDragRoute && !!root.quickRouteState

                onActiveChanged: {
                    if (!root.quickRouteState)
                        return

                    if (active) {
                        const scenePoint = centroid.scenePosition
                        root.quickRouteState.beginDrag(
                            root.safeAppData.id,
                            root.routeTargetLabel,
                            root.safeAppData.iconText,
                            root.safeAppData.iconColor,
                            root.displayText,
                            scenePoint.x,
                            scenePoint.y
                        )
                        return
                    }

                    if (root.dragInProgress) {
                        const scenePoint = centroid.scenePosition
                        root.quickRouteState.updateDrag(scenePoint.x, scenePoint.y)
                        root.quickRouteState.finishDrag()
                    }
                }

                onTranslationChanged: {
                    if (!active || !root.quickRouteState)
                        return

                    const scenePoint = centroid.scenePosition
                    root.quickRouteState.updateDrag(scenePoint.x, scenePoint.y)
                }

                onCanceled: {
                    if (root.dragInProgress && root.quickRouteState)
                        root.quickRouteState.cancelDrag()
                }
            }

            Row {
                anchors.fill: parent
                spacing: 4

                Rectangle {
                    width: 10
                    height: 10
                    radius: 2
                    color: root.safeAppData.iconColor
                    border.color: "#424242"

                    Label {
                        anchors.centerIn: parent
                        text: root.safeAppData.iconText
                        color: "#ece7e2"
                        font.family: "Noto Sans"
                        font.pixelSize: 5
                        font.weight: Font.DemiBold
                    }
                }

                Item {
                    id: titleViewport

                    width: parent.width - 14
                    height: parent.height
                    clip: true

                    SequentialAnimation {
                        id: marqueeAnimation

                        running: root.marqueeRunning
                        loops: Animation.Infinite

                        PauseAnimation {
                            duration: 350
                        }

                        NumberAnimation {
                            target: root
                            property: "marqueeOffset"
                            from: 0
                            to: Math.max(0, titleTextMetrics.tightBoundingRect.width - titleViewport.width)
                            duration: Math.max(1400, (titleTextMetrics.tightBoundingRect.width - titleViewport.width) * 40)
                            easing.type: Easing.Linear
                        }

                        PauseAnimation {
                            duration: 450
                        }

                        ScriptAction {
                            script: root.marqueeOffset = 0
                        }
                    }

                    Label {
                        id: titleText

                        x: root.marqueeRunning ? -root.marqueeOffset : 0
                        width: titleTextMetrics.tightBoundingRect.width
                        text: root.displayText
                        color: "#d9d3ce"
                        font.family: "Noto Sans"
                        font.pixelSize: 8
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
        }

        Row {
            width: parent.width
            height: 11
            spacing: 3

            InlineSlider {
                id: appSlider
                width: parent.width - muteButton.width - parent.spacing
                anchors.verticalCenter: parent.verticalCenter
                defaultValue: 1.0
                value: root.liveLevel
                fillColor: "#7f9d88"
                onMoved: function(nextValue) {
                    root.levelCommitPending = false
                    root.liveLevel = nextValue
                    root.previewLevelValue = nextValue
                    root.previewDirty = true
                }
                onReleased: function(nextValue) {
                    root.liveLevel = nextValue
                    root.pendingLevelValue = nextValue
                    root.levelCommitPending = true
                    root.previewDirty = false
                    root.controller.set_app_level(root.safeAppData.id, nextValue)
                }
            }

            Button {
                id: muteButton

                width: 14
                height: 11
                leftPadding: 0
                rightPadding: 0
                topPadding: 0
                bottomPadding: 0
                text: "M"
                onClicked: root.controller.toggle_app_muted(root.safeAppData.id)

                contentItem: Label {
                    anchors.fill: parent
                    text: muteButton.text
                    color: root.safeAppData.muted ? "#171717" : "#d0ccc7"
                    font.family: "Noto Sans"
                    font.pixelSize: 7
                    font.weight: Font.DemiBold
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                background: Rectangle {
                    color: root.safeAppData.muted ? "#a15a52" : "#232323"
                    border.color: "#3a3a3a"
                    radius: 2
                }
            }
        }
    }

    TextMetrics {
        id: titleTextMetrics

        text: root.displayText
        font.family: "Noto Sans"
        font.pixelSize: 8
    }
}
