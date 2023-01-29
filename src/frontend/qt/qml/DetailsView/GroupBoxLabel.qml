import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

import ".."

Rectangle {
    id: root
    property string text: ""
    property string icon: ""
    color: "#00000000"

    implicitWidth: label.implicitWidth
    implicitHeight: label.implicitHeight + 10

    layer.enabled: true
    layer.effect: DropShadow {
        color: "#40000000"
        radius: 8
        horizontalOffset: 0
        verticalOffset: 2
    }

    signal refreshClicked()

    Row {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        spacing: 8

        Image {
            source: root.icon
            height: label.implicitHeight * 0.9
            width: height
            // height: 32
            // width: 32
            // So that small images look good. Has a performance cost.
            mipmap: true

            ColorOverlay {
                anchors.fill: parent
                source: parent
                color: label.color
            }
        }

        Label {
            id: label
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            text: cleanupLabel(root.text)
            // So there's a more even padding between top and bottom
            lineHeight: 0.9
            bottomPadding: 4
        }
    }

    ImageButton {
        anchors.verticalCenter: parent.verticalCenter
        anchors.right: parent.right
        imageSource: "qrc:/main/images/button/refresh"
        imageRelativeWidth: 0.8
        imageRelativeHeight: 0.8
        tooltip: "Refresh"
        onClicked: root.refreshClicked()
    }

    function cleanupLabel(text) {
        return text.replace(/-/g, " ")
    }
}