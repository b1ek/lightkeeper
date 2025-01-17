import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15

import ".."
import "../Text"
import "../js/TextTransform.js" as TextTransform
import "../js/Parse.js" as Parse
import "../js/ValueUnit.js" as ValueUnit

Item {
    id: root
    required property string hostId
    property var _tabContents: {}
    property var _tabStacks: {}


    signal closeClicked()
    signal maximizeClicked()
    signal minimizeClicked()


    onHostIdChanged: {
        if (!(root.hostId in root._tabContents)) {
            root._tabContents[root.hostId] = []
            root._tabStacks[root.hostId] = tabStack.createObject(tabStackContainer, {
                parentStackIndex: Object.keys(root._tabStacks).length,
            })

            let tabData = {
                "title": hostId,
                "component": detailsMainView.createObject(root._tabStacks[root.hostId], {
                    hostId: hostId,
                })
            }

            createNewTab(tabData)
        }

        root.refresh()
    }

    Component.onCompleted: {
        root._tabContents = {}
        root._tabStacks = {}
    }

    Connections {
        target: CommandHandler

        function onTextViewOpened(title, invocationId) {
            let tabData = {
                "title": title,
                "component": textView.createObject(root._tabStacks[root.hostId], {
                    pendingInvocation: invocationId,
                })
            }

            createNewTab(tabData)
        }

        function onLogsViewOpened(showTimeControls, title, commandId, commandParams, invocationId) {
            let tabData = {
                "title": title,
                "component": logView.createObject(root._tabStacks[root.hostId], {
                    hostId: root.hostId,
                    pendingInvocation: invocationId,
                    commandId: commandId,
                    commandParams: commandParams,
                    showTimeControls: showTimeControls,
                })
            }

            createNewTab(tabData)
        }

        // For integrated text editor (not external).
        function onTextEditorViewOpened(commandId, invocationId, localFilePath) {
            let tabData = {
                "title": commandId,
                "component": textEditorView.createObject(root._tabStacks[root.hostId], {
                    commandId: commandId,
                    localFilePath: localFilePath,
                    pendingInvocation: invocationId,
                })
            }

            createNewTab(tabData)
        }

        // For integrated terminal.
        function onTerminalViewOpened(title, command) {
            let tabData = {
                "title": title,
                "component": terminalView.createObject(root._tabStacks[root.hostId], {})
            }

            createNewTab(tabData)
            tabData.component.open(command)
        }
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.backgroundColor
    }

    Header {
        id: mainViewHeader
        tabs: getTabTitles()
        showMinimizeButton: true
        showMaximizeButton: true
        showCloseButton: true
        showRefreshButton: getCurrentTabContent() instanceof HostDetailsMainView
        showSaveButton: getCurrentTabContent() instanceof HostDetailsTextEditorView
        disableSaveButton: true

        onRefreshClicked: CommandHandler.force_initialize_host(hostId)
        onMaximizeClicked: root.maximizeClicked()
        onMinimizeClicked: root.minimizeClicked()
        onCloseClicked: root.closeClicked()
        onSaveClicked: getCurrentTabContent().save()

        onTabClosed: function(tabIndex) {
            root.closeTab(tabIndex)
        }

        onTabIndexChanged: {
            if (root._tabContents[root.hostId][tabIndex] === undefined) {
                return
            }

            root._tabStacks[root.hostId].currentIndex = tabIndex
            root._tabContents[root.hostId][tabIndex].component.focus()
        }
    }

    StackLayout {
        id: tabStackContainer
        anchors.top: mainViewHeader.bottom
        anchors.bottom: root.bottom
        anchors.left: root.left
        anchors.right: root.right
        anchors.margins: Theme.spacingNormal
    }

    Component {
        id: tabStack

        StackLayout {
            property int parentStackIndex: -1
        }
    }

    Component {
        id: detailsMainView

        HostDetailsMainView {
        }
    }

    Component {
        id: textView

        HostDetailsTextView {
        }
    }

    Component {
        id: logView

        HostDetailsLogView {
        }
    }

    Component {
        id: textEditorView

        HostDetailsTextEditorView {
            onSaved: function(commandId, localFilePath, content) {
                pendingInvocation = CommandHandler.saveAndUploadFile(root.hostId, commandId, localFilePath, content)
            }
            onClosed: function(localFilePath) {
                CommandHandler.removeFile(localFilePath)
            }
            onContentChanged: function(localFilePath, content) {
                mainViewHeader.disableSaveButton = !CommandHandler.hasFileChanged(localFilePath, content)
            }
        }
    }

    Component {
        id: terminalView

        HostDetailsTerminalView {
        }
    }

    Shortcut {
        sequence: StandardKey.Cancel
        onActivated: root.closeClicked()
    }

    Shortcut {
        sequence: StandardKey.Close
        // Close current tab.
        onActivated: root.closeTab(mainViewHeader.tabIndex)
    }

    Shortcut {
        sequence: "Alt+1"
        onActivated: mainViewHeader.selectTab(0)
    }

    Shortcut {
        sequence: "Alt+2"
        onActivated: mainViewHeader.selectTab(1)
    }

    Shortcut {
        sequence: "Alt+3"
        onActivated: mainViewHeader.selectTab(2)
    }

    Shortcut {
        sequence: "Alt+4"
        onActivated: mainViewHeader.selectTab(3)
    }

    Shortcut {
        sequence: "Alt+5"
        onActivated: mainViewHeader.selectTab(4)
    }

    Shortcut {
        sequence: "Alt+6"
        onActivated: mainViewHeader.selectTab(5)
    }

    Shortcut {
        sequence: "Alt+7"
        onActivated: mainViewHeader.selectTab(6)
    }

    Shortcut {
        sequence: "Alt+8"
        onActivated: mainViewHeader.selectTab(7)
    }

    Shortcut {
        sequence: "Alt+9"
        onActivated: mainViewHeader.selectTab(8)
    }


    function refresh() {
        root._tabContents[root.hostId][0].component.refresh()

        mainViewHeader.tabs = getTabTitles()
        tabStackContainer.currentIndex = root._tabStacks[root.hostId].parentStackIndex
    }

    function createNewTab(tabData) {
        let similarTabs = root._tabContents[root.hostId].filter(tab => tab.title.startsWith(tabData.title)).length
        if (similarTabs > 0) {
            tabData.title = `${tabData.title} (${similarTabs + 1})`
        }

        // Extra padding for the title text before close button.
        tabData.title += "  "

        root._tabContents[root.hostId].push(tabData)

        let lastTabIndex = root._tabContents[root.hostId].length - 1
        mainViewHeader.tabs = getTabTitles()
        mainViewHeader.selectTab(lastTabIndex)
    }

    function closeTab(tabIndex) {
        let tabData = root._tabContents[root.hostId][tabIndex]
        tabData.component.close()
        tabData.component.destroy()
        root._tabContents[root.hostId].splice(tabIndex, 1)
        mainViewHeader.tabs = getTabTitles()
    }

    function getTabTitles() {
        if (!(root.hostId in root._tabContents)) {
            return []
        }
        return root._tabContents[root.hostId].map(tabData => tabData.title)
    }

    function getCurrentTabContent() {
        if (!(root.hostId in root._tabContents)) {
            return undefined
        }
        let content = root._tabContents[root.hostId][mainViewHeader.tabIndex]
        if (content === undefined) {
            return undefined
        }
        else {
            return content.component
        }
    }
}