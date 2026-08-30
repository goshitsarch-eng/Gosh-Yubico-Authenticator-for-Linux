// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.PromptDialog {
    id: dialog

    title: qsTr("Touch Required")
    standardButtons: Kirigami.Dialog.Cancel

    RowLayout {
        spacing: Kirigami.Units.largeSpacing

        Kirigami.Icon {
            source: "input-touchpad"
            Layout.preferredWidth: Kirigami.Units.iconSizes.large
            Layout.preferredHeight: Kirigami.Units.iconSizes.large
        }

        Controls.Label {
            Layout.fillWidth: true
            text: qsTr("Touch your YubiKey to generate the code.")
            wrapMode: Text.WordWrap
        }
    }
}
