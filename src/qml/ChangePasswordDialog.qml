// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.goshapps.yubicoauthenticator

Kirigami.PromptDialog {
    id: dialog

    title: qsTr("Change YubiKey Password")
    standardButtons: Kirigami.Dialog.Cancel

    readonly property bool formValid: removeSwitch.checked
        || (passwordField.text.length >= 4 && passwordField.text === confirmField.text)

    function openAndReset() {
        passwordField.text = ""
        confirmField.text = ""
        removeSwitch.checked = false
        open()
        passwordField.forceActiveFocus()
    }

    customFooterActions: [
        Kirigami.Action {
            text: qsTr("Save")
            icon.name: "dialog-ok-apply"
            enabled: dialog.formValid
            onTriggered: {
                AppController.setPassword(removeSwitch.checked ? "" : passwordField.text)
                dialog.close()
            }
        }
    ]

    ColumnLayout {
        spacing: Kirigami.Units.largeSpacing

        Controls.Label {
            Layout.fillWidth: true
            text: qsTr("Set a new OATH password, or remove password protection.")
            wrapMode: Text.WordWrap
        }

        Kirigami.PasswordField {
            id: passwordField
            Layout.fillWidth: true
            enabled: !removeSwitch.checked
            placeholderText: qsTr("New password (at least 4 characters)")
        }

        Kirigami.PasswordField {
            id: confirmField
            Layout.fillWidth: true
            enabled: !removeSwitch.checked
            placeholderText: qsTr("Confirm password")
        }

        Controls.Switch {
            id: removeSwitch
            text: qsTr("Remove password protection")
        }

        Controls.Label {
            Layout.fillWidth: true
            visible: removeSwitch.checked
            text: qsTr("Anyone with physical access will be able to use this key.")
            color: Kirigami.Theme.neutralTextColor
            wrapMode: Text.WordWrap
            font: Kirigami.Theme.smallFont
        }
    }
}
