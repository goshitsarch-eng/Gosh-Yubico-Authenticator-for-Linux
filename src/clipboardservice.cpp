// SPDX-License-Identifier: GPL-3.0-or-later

#include "clipboardservice.h"

#include <QClipboard>
#include <QGuiApplication>

ClipboardService::ClipboardService(QObject *parent)
    : QObject(parent)
{
    m_timer.setSingleShot(true);
    connect(&m_timer, &QTimer::timeout, this, &ClipboardService::clearIfUnchanged);
}

void ClipboardService::copy(const QString &text, int timeoutSeconds)
{
    QClipboard *clipboard = QGuiApplication::clipboard();
    if (!clipboard) {
        return;
    }
    clipboard->setText(text);
    m_lastCopied = text;
    m_timer.start(timeoutSeconds * 1000);
}

void ClipboardService::clearIfUnchanged()
{
    QClipboard *clipboard = QGuiApplication::clipboard();
    if (!clipboard || m_lastCopied.isEmpty()) {
        return;
    }
    if (clipboard->text() == m_lastCopied) {
        clipboard->clear();
    }
    m_lastCopied.clear();
}
