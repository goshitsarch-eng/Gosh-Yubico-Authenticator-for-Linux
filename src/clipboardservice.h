// SPDX-License-Identifier: GPL-3.0-or-later
//
// Copies text to the clipboard and clears it after a timeout, but only if
// the clipboard still holds the copied value.

#pragma once

#include <QObject>
#include <QString>
#include <QTimer>

class ClipboardService : public QObject {
    Q_OBJECT

public:
    explicit ClipboardService(QObject *parent = nullptr);

    void copy(const QString &text, int timeoutSeconds);

private:
    void clearIfUnchanged();

    QString m_lastCopied;
    QTimer m_timer;
};
