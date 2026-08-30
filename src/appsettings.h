// SPDX-License-Identifier: GPL-3.0-or-later
//
// Persistent application preferences, exposed to QML.

#pragma once

#include <QObject>
#include <QSettings>
#include <QVariantList>
#include <QtQml/qqmlregistration.h>

class AppSettings : public QObject {
    Q_OBJECT
    QML_ELEMENT
    QML_UNCREATABLE("Access through AppController.settings")

    /// 0 = follow system, 1 = light, 2 = dark
    Q_PROPERTY(int themeMode READ themeMode WRITE setThemeMode NOTIFY themeModeChanged)
    Q_PROPERTY(int clipboardTimeoutIndex READ clipboardTimeoutIndex WRITE setClipboardTimeoutIndex
                   NOTIFY clipboardTimeoutIndexChanged)
    Q_PROPERTY(int clipboardTimeoutSeconds READ clipboardTimeoutSeconds NOTIFY clipboardTimeoutIndexChanged)
    Q_PROPERTY(QVariantList clipboardTimeoutChoices READ clipboardTimeoutChoices CONSTANT)

public:
    explicit AppSettings(QObject *parent = nullptr);

    int themeMode() const;
    void setThemeMode(int mode);

    int clipboardTimeoutIndex() const;
    void setClipboardTimeoutIndex(int index);
    int clipboardTimeoutSeconds() const;
    QVariantList clipboardTimeoutChoices() const;

    // Per-credential icon preferences, keyed by the base64 credential id.
    QString customIconKey(const QString &credId) const;
    QString faviconDomain(const QString &credId) const;
    Q_INVOKABLE void setIconPreference(const QString &credId,
                                       const QString &customIconKey,
                                       const QString &faviconDomain);
    Q_INVOKABLE void clearIconPreference(const QString &credId);

Q_SIGNALS:
    void themeModeChanged();
    void clipboardTimeoutIndexChanged();
    void iconPreferencesChanged();

private:
    mutable QSettings m_settings;
};
