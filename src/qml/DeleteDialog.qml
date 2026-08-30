// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls as Controls
import org.kde.kirigami as Kirigami
import com.goshapps.yubicoauthenticator

Kirigami.PromptDialog {
    id: dialog

    property string credId: ""
    property string displayName: ""

    title: qsTr("Delete Credential?")
    subtitle: qsTr("Are you sure you want to delete “%1”? This cannot be undone.").arg(displayName)
    standardButtons: Kirigami.Dialog.Cancel
    showCloseButton: false

    customFooterActions: [
        Kirigami.Action {
            text: qsTr("Delete")
            icon.name: "edit-delete"
            onTriggered: {
                AppController.deleteCredential(dialog.credId)
                dialog.close()
            }
        }
    ]
}
