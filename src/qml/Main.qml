// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls as Controls
import org.kde.kirigami as Kirigami
import com.goshapps.yubicoauthenticator

Kirigami.ApplicationWindow {
    id: root

    title: AppController.appName
    width: 960
    height: 680
    minimumWidth: 400
    minimumHeight: 400

    pageStack.initialPage: credentialsPageComponent

    function switchToPage(component, name) {
        if (pageStack.currentItem && pageStack.currentItem.objectName === name) {
            return
        }
        pageStack.clear()
        pageStack.push(component)
    }

    globalDrawer: Kirigami.GlobalDrawer {
        id: drawer
        title: AppController.appName
        titleIcon: "com.goshapps.YubicoAuthenticator"
        isMenu: !root.wideScreen
        modal: !root.wideScreen
        collapsible: root.wideScreen
        showHeaderWhenCollapsed: true

        actions: [
            Kirigami.Action {
                text: qsTr("Credentials")
                icon.name: "dialog-password"
                checked: pageStack.currentItem && pageStack.currentItem.objectName === "credentialsPage"
                onTriggered: root.switchToPage(credentialsPageComponent, "credentialsPage")
            },
            Kirigami.Action {
                text: qsTr("Key Info")
                icon.name: "dialog-information"
                checked: pageStack.currentItem && pageStack.currentItem.objectName === "keyInfoPage"
                onTriggered: root.switchToPage(keyInfoPageComponent, "keyInfoPage")
            },
            Kirigami.Action {
                text: qsTr("Settings")
                icon.name: "settings-configure"
                checked: pageStack.currentItem && pageStack.currentItem.objectName === "settingsPage"
                onTriggered: root.switchToPage(settingsPageComponent, "settingsPage")
            },
            Kirigami.Action {
                text: qsTr("About Gosh Yubico Authenticator")
                icon.name: "help-about"
                onTriggered: root.pageStack.pushDialogLayer(aboutPageComponent)
            },
            Kirigami.Action {
                text: qsTr("Quit")
                icon.name: "application-exit"
                onTriggered: Qt.quit()
            }
        ]
    }

    Component {
        id: credentialsPageComponent
        CredentialsPage { objectName: "credentialsPage" }
    }
    Component {
        id: keyInfoPageComponent
        KeyInfoPage { objectName: "keyInfoPage" }
    }
    Component {
        id: settingsPageComponent
        SettingsPage { objectName: "settingsPage" }
    }
    Component {
        id: aboutPageComponent
        Kirigami.AboutPage {
            aboutData: AppController.aboutData
        }
    }

    UnlockDialog {
        id: unlockDialog
    }

    TouchDialog {
        id: touchDialog
    }

    Connections {
        target: AppController

        function onShowMessage(message) {
            root.showPassiveNotification(message)
        }

        function onAuthenticationRequired() {
            unlockDialog.errorMessage = ""
            unlockDialog.openAndFocus()
        }

        function onAuthenticationFailed(message) {
            unlockDialog.errorMessage = message
            unlockDialog.openAndFocus()
        }

        function onTouchRequired() {
            touchDialog.open()
        }

        function onTouchResolved() {
            touchDialog.close()
        }
    }
}
