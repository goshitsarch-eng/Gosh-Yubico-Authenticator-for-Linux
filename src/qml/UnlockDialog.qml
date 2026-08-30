// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.goshapps.yubicoauthenticator

Kirigami.PromptDialog {
    id: dialog

    property string errorMessage: ""

    title: qsTr("Unlock YubiKey")
    standardButtons: Kirigami.Dialog.Cancel
    closePolicy: Controls.Popup.CloseOnEscape

    customFooterActions: [
        Kirigami.Action {
            text: qsTr("Unlock")
            icon.name: "unlock"
            enabled: passwordField.text.length > 0
            onTriggered: dialog.unlock()
        }
    ]

    function openAndFocus() {
        passwordField.text = ""
        open()
        passwordField.forceActiveFocus()
    }

    function unlock() {
        if (passwordField.text.length === 0) {
            return
        }
        AppController.authenticate(passwordField.text)
        close()
    }

    ColumnLayout {
        spacing: Kirigami.Units.largeSpacing

        Controls.Label {
            Layout.fillWidth: true
            text: qsTr("This YubiKey is password protected. Enter the OATH password to continue.")
            wrapMode: Text.WordWrap
        }

        Kirigami.InlineMessage {
            Layout.fillWidth: true
            visible: dialog.errorMessage.length > 0
            type: Kirigami.MessageType.Error
            text: dialog.errorMessage
        }

        Kirigami.PasswordField {
            id: passwordField
            Layout.fillWidth: true
            placeholderText: qsTr("YubiKey password")
            onAccepted: dialog.unlock()
        }
    }
}
