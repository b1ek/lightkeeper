import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

import "js/TextTransform.js" as TextTransform
import "js/Parse.js" as Parse
import "js/ValueUnit.js" as ValueUnit

Item {
    id: root
    required property var commandHandler
    required property var hostDataManager
    property string hostId: ""
    property real _subviewSize: 0.0

    signal closeClicked()
    signal maximizeClicked()
    signal minimizeClicked()

    Rectangle {
        anchors.fill: parent
        color: Material.background
    }

    Header {
        id: mainViewHeader
        text: root.hostId
        onMaximizeClicked: root.maximizeClicked()
        onMinimizeClicked: root.minimizeClicked()
        onCloseClicked: root.closeClicked()
    }

    HostDetailsMainView {
        id: detailsMainView
        anchors.top: mainViewHeader.bottom
        anchors.bottom: root.bottom
        anchors.left: root.left
        anchors.right: root.right
        anchors.margins: 10

        commandHandler: root.commandHandler
        hostDataManager: root.hostDataManager
        hostId: root.hostId
    }

    Item {
        id: detailsSubview

        implicitHeight: (detailsMainView.height - mainViewHeader.height - 3) * root._subviewSize
        anchors.bottom: root.bottom
        anchors.left: root.left
        anchors.right: root.right

        Header {
            id: subviewHeader

            showOpenInWindowButton: true
            showMaximizeButton: false
            // TODO:
            // onOpenInWindowClicked: root.maximizeClicked()
            onCloseClicked: animateHideSubview.start()
        }

        HostDetailsSubview {
            id: subviewContent
            anchors.top: subviewHeader.bottom
            anchors.bottom: parent.bottom
        }

    }

    NumberAnimation {
        id: animateShowSubview
        target: root
        property: "_subviewSize"
        to: 1.0
        duration: 150
    }

    NumberAnimation {
        id: animateHideSubview
        target: root
        property: "_subviewSize"
        to: 0.0
        duration: 150
    }

    states: [
        State {
            name: "subviewShownVisibility"
            when: root._subviewSize > 0.01

            PropertyChanges {
                target: detailsSubview
                visible: true
            }
        },
        State {
            name: "subviewHiddenVisibility"
            when: root._subviewSize < 0.01

            PropertyChanges {
                target: detailsSubview
                visible: false
            }
        }
    ]

    function refresh() {
        detailsMainView.refresh()
    }

    function openSubview(headerText) {
        subviewHeader.text = headerText
        animateShowSubview.start()
    }

    function refreshSubview(commandResult) {
        subviewContent.text = commandResult.message
        subviewContent.errorText = commandResult.error
        subviewContent.criticality = commandResult.criticality
    }

}