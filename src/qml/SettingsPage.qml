// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.goshapps.yubicoauthenticator

Kirigami.ScrollablePage {
    id: page

    title: qsTr("Settings")

    readonly property bool connected: AppController.connectionState === AppController.Connected

    ChangePasswordDialog {
        id: changePasswordDialog
    }

    Kirigami.FormLayout {

        Kirigami.Separator {
            Kirigami.FormData.isSection: true
            Kirigami.FormData.label: qsTr("Appearance")
        }

        Controls.ComboBox {
            Kirigami.FormData.label: qsTr("Theme:")
            model: [qsTr("Follow System"), qsTr("Light"), qsTr("Dark")]
            currentIndex: AppController.settings.themeMode
            onActivated: index => AppController.settings.themeMode = index
        }

        Kirigami.Separator {
            Kirigami.FormData.isSection: true
            Kirigami.FormData.label: qsTr("Security")
        }

        Controls.ComboBox {
            Kirigami.FormData.label: qsTr("Clear clipboard after:")
            model: AppController.settings.clipboardTimeoutChoices.map(s => qsTr("%1 seconds").arg(s))
            currentIndex: AppController.settings.clipboardTimeoutIndex
            onActivated: index => AppController.settings.clipboardTimeoutIndex = index
        }

        Controls.Button {
            Kirigami.FormData.label: qsTr("YubiKey password:")
            text: qsTr("Change YubiKey Password…")
            icon.name: "dialog-password"
            enabled: page.connected
            onClicked: changePasswordDialog.openAndReset()
        }

        Controls.Label {
            visible: !page.connected
            text: qsTr("Connect a YubiKey to change its password")
            opacity: 0.7
            font: Kirigami.Theme.smallFont
        }

        Kirigami.Separator {
            Kirigami.FormData.isSection: true
        }

        Controls.Label {
            text: qsTr("%1 · Version %2").arg(AppController.appName).arg(AppController.appVersion)
            opacity: 0.6
        }
    }
}
