// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.goshapps.yubicoauthenticator

Controls.ItemDelegate {
    id: delegate

    required property int index
    required property string credId
    required property string title
    required property string subtitle
    required property string displayName
    required property string code
    required property bool hasCode
    required property bool touchRequired
    required property bool isHotp
    required property int period
    required property string iconLabel
    required property color iconColor
    required property url favicon
    required property string guessedDomain

    signal deleteRequested(string credId, string displayName)
    signal iconPickerRequested(string credId, string domain)

    property bool showCopied: false
    // Re-evaluates on every tick through the nowSeconds dependency.
    readonly property real progress: AppController.nowSeconds > 0
        ? AppController.totpProgress(period) : 1
    readonly property bool expiring: !isHotp && hasCode && progress <= 0.25

    hoverEnabled: true

    onClicked: {
        if (hasCode) {
            AppController.copyCode(credId)
        } else {
            AppController.calculate(credId)
        }
    }

    Connections {
        target: AppController
        function onCredentialCopied(copiedId, clearSeconds) {
            if (copiedId === delegate.credId) {
                delegate.showCopied = true
                copiedTimer.restart()
            }
        }
    }

    Timer {
        id: copiedTimer
        interval: 2000
        onTriggered: delegate.showCopied = false
    }

    TapHandler {
        acceptedButtons: Qt.RightButton
        onTapped: contextMenu.popup()
    }
    TapHandler {
        acceptedButtons: Qt.LeftButton
        onLongPressed: contextMenu.popup()
    }

    Controls.Menu {
        id: contextMenu

        Controls.MenuItem {
            text: qsTr("Copy Code")
            icon.name: "edit-copy"
            enabled: delegate.hasCode
            onTriggered: AppController.copyCode(delegate.credId)
        }
        Controls.MenuItem {
            text: qsTr("Calculate")
            icon.name: "view-refresh"
            onTriggered: AppController.calculate(delegate.credId)
        }
        Controls.MenuItem {
            text: qsTr("Choose Icon…")
            icon.name: "preferences-desktop-icons"
            onTriggered: delegate.iconPickerRequested(delegate.credId, delegate.guessedDomain)
        }
        Controls.MenuSeparator {}
        Controls.MenuItem {
            text: qsTr("Delete…")
            icon.name: "edit-delete"
            onTriggered: delegate.deleteRequested(delegate.credId, delegate.displayName)
        }
    }

    contentItem: RowLayout {
        spacing: Kirigami.Units.largeSpacing

        // Avatar: cached favicon when available, colored letter otherwise
        Item {
            Layout.preferredWidth: 36
            Layout.preferredHeight: 36
            Layout.alignment: Qt.AlignVCenter

            Rectangle {
                anchors.fill: parent
                radius: width / 2
                color: delegate.iconColor
                visible: faviconImage.status !== Image.Ready

                Controls.Label {
                    anchors.centerIn: parent
                    text: delegate.iconLabel
                    color: "white"
                    font.bold: true
                    font.pixelSize: 16
                }
            }

            Image {
                id: faviconImage
                anchors.centerIn: parent
                width: 28
                height: 28
                source: delegate.favicon
                visible: status === Image.Ready
                fillMode: Image.PreserveAspectFit
                asynchronous: true
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignVCenter
            spacing: Kirigami.Units.smallSpacing / 2

            Controls.Label {
                Layout.fillWidth: true
                text: delegate.title
                elide: Text.ElideRight
                font.weight: Font.Medium
            }
            Controls.Label {
                Layout.fillWidth: true
                text: delegate.subtitle
                visible: delegate.subtitle.length > 0
                elide: Text.ElideRight
                opacity: 0.7
                font: Kirigami.Theme.smallFont
            }
            Controls.Label {
                text: qsTr("Touch required")
                visible: delegate.touchRequired && !delegate.hasCode
                color: Kirigami.Theme.neutralTextColor
                font: Kirigami.Theme.smallFont
            }
        }

        Controls.Label {
            Layout.alignment: Qt.AlignVCenter
            visible: delegate.showCopied
            text: qsTr("Copied")
            color: Kirigami.Theme.positiveTextColor
            font.weight: Font.Medium
        }

        Controls.Label {
            Layout.alignment: Qt.AlignVCenter
            visible: !delegate.showCopied
            text: delegate.hasCode ? delegate.code : "•••  •••"
            opacity: delegate.hasCode ? 1.0 : 0.5
            color: delegate.expiring ? Kirigami.Theme.neutralTextColor : Kirigami.Theme.textColor
            font.family: "monospace"
            font.pixelSize: Kirigami.Theme.defaultFont.pixelSize + 4
            font.weight: Font.DemiBold
        }

        // Trailing widget: HOTP chip, TOTP countdown ring, or calculate button
        Controls.Label {
            Layout.alignment: Qt.AlignVCenter
            visible: delegate.isHotp
            text: "HOTP"
            opacity: 0.6
            font: Kirigami.Theme.smallFont
        }

        Canvas {
            id: ring
            Layout.preferredWidth: 26
            Layout.preferredHeight: 26
            Layout.alignment: Qt.AlignVCenter
            visible: !delegate.isHotp && delegate.hasCode

            readonly property real value: delegate.progress
            readonly property color ringColor: delegate.expiring
                ? Kirigami.Theme.neutralTextColor
                : Kirigami.Theme.highlightColor
            readonly property color trackColor: Qt.alpha(Kirigami.Theme.textColor, 0.2)

            onValueChanged: requestPaint()
            onRingColorChanged: requestPaint()
            onVisibleChanged: requestPaint()

            onPaint: {
                const ctx = getContext("2d")
                ctx.reset()
                const cx = width / 2
                const cy = height / 2
                const radius = Math.min(width, height) / 2 - 2.5
                ctx.lineWidth = 3
                ctx.strokeStyle = trackColor
                ctx.beginPath()
                ctx.arc(cx, cy, radius, 0, 2 * Math.PI)
                ctx.stroke()
                ctx.strokeStyle = ringColor
                ctx.beginPath()
                ctx.arc(cx, cy, radius, -Math.PI / 2, -Math.PI / 2 + 2 * Math.PI * Math.max(0, Math.min(1, value)))
                ctx.stroke()
            }
        }

        Controls.ToolButton {
            Layout.alignment: Qt.AlignVCenter
            visible: !delegate.isHotp && !delegate.hasCode
            icon.name: "media-playback-start"
            display: Controls.AbstractButton.IconOnly
            text: qsTr("Calculate")
            onClicked: AppController.calculate(delegate.credId)

            Controls.ToolTip.text: text
            Controls.ToolTip.visible: hovered
            Controls.ToolTip.delay: Kirigami.Units.toolTipDelay
        }
    }
}
