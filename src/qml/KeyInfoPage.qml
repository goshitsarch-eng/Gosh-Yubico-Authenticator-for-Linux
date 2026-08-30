// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.goshapps.yubicoauthenticator

Kirigami.ScrollablePage {
    id: page

    title: qsTr("Key Info")

    readonly property bool connected: AppController.connectionState === AppController.Connected

    Kirigami.PlaceholderMessage {
        anchors.centerIn: parent
        width: parent.width - Kirigami.Units.gridUnit * 4
        visible: !page.connected
        icon.name: "media-removable"
        text: qsTr("No YubiKey Connected")
        explanation: qsTr("Insert your YubiKey to view device info")

        helpfulAction: Kirigami.Action {
            icon.name: "view-refresh"
            text: qsTr("Retry")
            onTriggered: AppController.retryConnect()
        }
    }

    Kirigami.FormLayout {
        visible: page.connected

        Kirigami.Separator {
            Kirigami.FormData.isSection: true
            Kirigami.FormData.label: qsTr("Device Information")
        }

        Controls.Label {
            Kirigami.FormData.label: qsTr("Device name:")
            text: AppController.deviceName
        }

        Controls.Label {
            Kirigami.FormData.label: qsTr("Firmware version:")
            text: AppController.firmwareVersion
        }

        Controls.Label {
            Kirigami.FormData.label: qsTr("Password protection:")
            text: AppController.hasPassword ? qsTr("Enabled") : qsTr("Disabled")
        }

        Kirigami.Separator {
            Kirigami.FormData.isSection: true
            Kirigami.FormData.label: qsTr("OATH Credentials")
        }

        Controls.Label {
            Kirigami.FormData.label: qsTr("Total:")
            text: AppController.credentials.totalCount
        }

        Controls.Label {
            Kirigami.FormData.label: qsTr("TOTP:")
            text: AppController.credentials.totpCount
        }

        Controls.Label {
            Kirigami.FormData.label: qsTr("HOTP:")
            text: AppController.credentials.hotpCount
        }

        Kirigami.Separator {
            Kirigami.FormData.isSection: true
            Kirigami.FormData.label: qsTr("Connection")
        }

        RowLayout {
            Kirigami.FormData.label: qsTr("Status:")
            spacing: Kirigami.Units.smallSpacing

            Kirigami.Icon {
                source: "emblem-ok-symbolic"
                Layout.preferredWidth: Kirigami.Units.iconSizes.small
                Layout.preferredHeight: Kirigami.Units.iconSizes.small
            }
            Controls.Label {
                text: qsTr("Connected via USB (PC/SC smart card reader)")
            }
        }
    }
}
