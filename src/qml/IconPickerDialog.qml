// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.goshapps.yubicoauthenticator

Kirigami.PromptDialog {
    id: dialog

    property string credId: ""
    property string currentDomain: ""
    property string chosenKey: ""

    title: qsTr("Choose Icon")
    standardButtons: Kirigami.Dialog.Cancel
    preferredWidth: Kirigami.Units.gridUnit * 24

    onOpened: {
        chosenKey = ""
        domainField.text = currentDomain
    }

    customFooterActions: [
        Kirigami.Action {
            text: qsTr("Reset")
            icon.name: "edit-undo"
            onTriggered: {
                AppController.settings.clearIconPreference(dialog.credId)
                dialog.close()
            }
        },
        Kirigami.Action {
            text: qsTr("Save")
            icon.name: "dialog-ok-apply"
            onTriggered: {
                AppController.settings.setIconPreference(dialog.credId,
                                                         dialog.chosenKey,
                                                         domainField.text.trim())
                dialog.close()
            }
        }
    ]

    ColumnLayout {
        spacing: Kirigami.Units.largeSpacing

        Controls.Label {
            Layout.fillWidth: true
            text: qsTr("Pick a service for the avatar color, or set a domain to fetch its favicon.")
            wrapMode: Text.WordWrap
            opacity: 0.7
        }

        Controls.TextField {
            id: domainField
            Layout.fillWidth: true
            placeholderText: qsTr("Favicon domain, e.g. example.com")
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 3
            rowSpacing: Kirigami.Units.smallSpacing
            columnSpacing: Kirigami.Units.smallSpacing

            Repeater {
                model: AppController.serviceChoices

                delegate: Controls.Button {
                    required property var modelData

                    Layout.fillWidth: true
                    text: modelData.label
                    checkable: true
                    checked: dialog.chosenKey === modelData.key
                    onClicked: dialog.chosenKey = checked ? modelData.key : ""

                    Rectangle {
                        anchors.left: parent.left
                        anchors.leftMargin: Kirigami.Units.smallSpacing
                        anchors.verticalCenter: parent.verticalCenter
                        width: 10
                        height: 10
                        radius: 5
                        color: modelData.color
                    }
                }
            }
        }
    }
}
