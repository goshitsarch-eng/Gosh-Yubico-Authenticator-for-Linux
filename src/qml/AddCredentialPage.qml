// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Dialogs as Dialogs
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.goshapps.yubicoauthenticator

Kirigami.ScrollablePage {
    id: page

    title: qsTr("Add Credential")

    readonly property bool formValid: accountField.text.trim().length > 0
                                      && AppController.isSecretValid(secretField.text)

    function applyOtpauth(result) {
        issuerField.text = result.issuer
        accountField.text = result.account
        secretField.text = result.secret
        typeCombo.currentIndex = result.isHotp ? 1 : 0
        algorithmCombo.currentIndex = result.algorithmIndex
        digitsCombo.currentIndex = Math.max(0, Math.min(2, result.digits - 6))
    }

    function submit() {
        if (!formValid) {
            return
        }
        AppController.addCredential(issuerField.text.trim(),
                                    accountField.text.trim(),
                                    secretField.text,
                                    typeCombo.currentIndex === 1,
                                    algorithmCombo.currentIndex,
                                    digitsCombo.currentIndex + 6,
                                    touchSwitch.checked)
        applicationWindow().pageStack.layers.pop()
    }

    actions: [
        Kirigami.Action {
            text: qsTr("Scan QR from File…")
            icon.name: "view-barcode-qr"
            onTriggered: qrFileDialog.open()
        },
        Kirigami.Action {
            text: qsTr("Add Credential")
            icon.name: "list-add"
            enabled: page.formValid
            onTriggered: page.submit()
        }
    ]

    Dialogs.FileDialog {
        id: qrFileDialog
        title: qsTr("Select QR Code Image")
        fileMode: Dialogs.FileDialog.OpenFile
        nameFilters: [qsTr("Images (*.png *.jpg *.jpeg *.webp *.bmp)"), qsTr("All files (*)")]
        onAccepted: {
            const result = AppController.importQrFile(selectedFile)
            if (result.ok) {
                page.applyOtpauth(result)
                applicationWindow().showPassiveNotification(qsTr("QR code imported"))
            } else {
                applicationWindow().showPassiveNotification(result.error)
            }
        }
    }

    Kirigami.FormLayout {
        id: form

        Kirigami.Separator {
            Kirigami.FormData.isSection: true
            Kirigami.FormData.label: qsTr("Identity")
        }

        Controls.TextField {
            id: issuerField
            Kirigami.FormData.label: qsTr("Issuer:")
            placeholderText: qsTr("e.g. GitHub")
        }

        Controls.TextField {
            id: accountField
            Kirigami.FormData.label: qsTr("Account:")
            placeholderText: qsTr("e.g. user@example.com")
        }

        Kirigami.Separator {
            Kirigami.FormData.isSection: true
            Kirigami.FormData.label: qsTr("Secret")
        }

        Kirigami.PasswordField {
            id: secretField
            Kirigami.FormData.label: qsTr("Secret (Base32):")
            // Pasting a full otpauth:// URI fills in the whole form.
            onTextChanged: {
                if (text.startsWith("otpauth://")) {
                    const result = AppController.parseOtpauth(text)
                    if (result.ok) {
                        page.applyOtpauth(result)
                    }
                }
            }
        }

        Controls.ComboBox {
            id: typeCombo
            Kirigami.FormData.label: qsTr("Type:")
            model: ["TOTP", "HOTP"]
        }

        Controls.ComboBox {
            id: algorithmCombo
            Kirigami.FormData.label: qsTr("Algorithm:")
            model: ["SHA1", "SHA256", "SHA512"]
        }

        Controls.ComboBox {
            id: digitsCombo
            Kirigami.FormData.label: qsTr("Digits:")
            model: ["6", "7", "8"]
        }

        Controls.Switch {
            id: touchSwitch
            Kirigami.FormData.label: qsTr("Require touch:")
            text: qsTr("Physical touch is required to generate a code")
        }

        Kirigami.Separator {
            Kirigami.FormData.isSection: true
        }

        Controls.Label {
            Layout.fillWidth: true
            text: qsTr("Credentials are stored securely on your YubiKey. Removing the app will not delete credentials from the hardware key.")
            wrapMode: Text.WordWrap
            opacity: 0.7
        }

        Controls.Button {
            text: qsTr("Add Credential")
            icon.name: "list-add"
            enabled: page.formValid
            onClicked: page.submit()
        }
    }
}
