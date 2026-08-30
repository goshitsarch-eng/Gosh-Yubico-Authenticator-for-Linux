// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.goshapps.yubicoauthenticator

Kirigami.ScrollablePage {
    id: page

    title: qsTr("Credentials")

    readonly property bool connected: AppController.connectionState === AppController.Connected

    titleDelegate: Kirigami.SearchField {
        Layout.fillWidth: true
        placeholderText: qsTr("Search accounts…")
        onTextChanged: AppController.credentials.filterString = text
    }

    actions: [
        Kirigami.Action {
            text: qsTr("Add Credential")
            icon.name: "list-add"
            visible: page.connected
            onTriggered: applicationWindow().pageStack.layers.push(addPageComponent)
        },
        Kirigami.Action {
            text: qsTr("Refresh")
            icon.name: "view-refresh"
            visible: page.connected
            onTriggered: AppController.refresh()
        }
    ]

    Component {
        id: addPageComponent
        AddCredentialPage {}
    }

    DeleteDialog {
        id: deleteDialog
    }

    IconPickerDialog {
        id: iconPickerDialog
    }

    ListView {
        id: listView

        model: AppController.credentials
        currentIndex: -1
        reuseItems: false

        delegate: CredentialDelegate {
            width: listView.width
            onDeleteRequested: (credId, displayName) => {
                deleteDialog.credId = credId
                deleteDialog.displayName = displayName
                deleteDialog.open()
            }
            onIconPickerRequested: (credId, domain) => {
                iconPickerDialog.credId = credId
                iconPickerDialog.currentDomain = domain
                iconPickerDialog.open()
            }
        }

        Kirigami.PlaceholderMessage {
            anchors.centerIn: parent
            width: parent.width - Kirigami.Units.gridUnit * 4
            visible: listView.count === 0 && !page.connected
            icon.name: "media-removable"
            text: qsTr("No YubiKey Connected")
            explanation: qsTr("Insert your YubiKey to view credentials")

            helpfulAction: Kirigami.Action {
                icon.name: "view-refresh"
                text: qsTr("Retry")
                onTriggered: AppController.retryConnect()
            }
        }

        Kirigami.PlaceholderMessage {
            anchors.centerIn: parent
            width: parent.width - Kirigami.Units.gridUnit * 4
            visible: listView.count === 0 && page.connected
                     && AppController.credentials.filterString.length === 0
            icon.name: "dialog-password"
            text: qsTr("No Credentials")
            explanation: qsTr("Add your first credential to get started")

            helpfulAction: Kirigami.Action {
                icon.name: "list-add"
                text: qsTr("Add Credential")
                onTriggered: applicationWindow().pageStack.layers.push(addPageComponent)
            }
        }

        Kirigami.PlaceholderMessage {
            anchors.centerIn: parent
            width: parent.width - Kirigami.Units.gridUnit * 4
            visible: listView.count === 0 && page.connected
                     && AppController.credentials.filterString.length > 0
            icon.name: "edit-find"
            text: qsTr("No Matches")
            explanation: qsTr("No credentials match your search")
        }
    }
}
