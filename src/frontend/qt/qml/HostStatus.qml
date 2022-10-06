import QtQuick 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Layouts 1.15

Item {
    id: root
    required property string status
    property var colors: {}
    property bool showIcon: true
    anchors.fill: parent

    FontLoader { id: font_status; source: "qrc:/main/fonts/pressstart2p" }

    RowLayout {
        anchors.fill: parent

        Image {
            id: status_image
            antialiasing: true
            source: "qrc:/main/images/status/" + root.status
            visible: showIcon

            Layout.leftMargin: showIcon ? 0.4 * parent.height : 0
            Layout.rightMargin: showIcon ? 0.4 * parent.height : 0
            Layout.preferredWidth: showIcon ? 0.7 * parent.height : 0
            Layout.preferredHeight: showIcon ? 0.7 * parent.height : 0
            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter

            ColorOverlay {
                anchors.fill: parent
                source: parent
                color: getColor(root.status)
                antialiasing: true
                visible: showIcon
            }
        }

        NormalText {
            text: status.toUpperCase()
            font.family: font_status.name
            color: getColor(root.status)

            Layout.fillWidth: true
            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
        }
    }


    Component.onCompleted: function() {
        colors = {
            up: "forestgreen",
            down: "firebrick",
            _: "orange",
        }
    }

    function getColor(status) {
        if (typeof status === "undefined") {
            return colors["_"]
        }
        return colors[status]
    }
}