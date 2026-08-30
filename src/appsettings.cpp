// SPDX-License-Identifier: GPL-3.0-or-later

#include "appsettings.h"

namespace {

const int kClipboardTimeouts[] = {10, 20, 30, 60, 120};
constexpr int kDefaultTimeoutIndex = 2; // 30 seconds

QString iconGroup(const QString &credId)
{
    // QSettings treats '/' as a group separator; base64 ids may contain it.
    QString safe = credId;
    safe.replace(QLatin1Char('/'), QLatin1Char('_'));
    return QStringLiteral("icons/") + safe;
}

} // namespace

AppSettings::AppSettings(QObject *parent)
    : QObject(parent)
{
}

int AppSettings::themeMode() const
{
    const int mode = m_settings.value(QStringLiteral("appearance/themeMode"), 0).toInt();
    return (mode >= 0 && mode <= 2) ? mode : 0;
}

void AppSettings::setThemeMode(int mode)
{
    if (mode < 0 || mode > 2 || mode == themeMode()) {
        return;
    }
    m_settings.setValue(QStringLiteral("appearance/themeMode"), mode);
    Q_EMIT themeModeChanged();
}

int AppSettings::clipboardTimeoutIndex() const
{
    const int index =
        m_settings.value(QStringLiteral("clipboard/timeoutIndex"), kDefaultTimeoutIndex).toInt();
    return (index >= 0 && index < static_cast<int>(std::size(kClipboardTimeouts)))
        ? index
        : kDefaultTimeoutIndex;
}

void AppSettings::setClipboardTimeoutIndex(int index)
{
    if (index < 0 || index >= static_cast<int>(std::size(kClipboardTimeouts))
        || index == clipboardTimeoutIndex()) {
        return;
    }
    m_settings.setValue(QStringLiteral("clipboard/timeoutIndex"), index);
    Q_EMIT clipboardTimeoutIndexChanged();
}

int AppSettings::clipboardTimeoutSeconds() const
{
    return kClipboardTimeouts[clipboardTimeoutIndex()];
}

QVariantList AppSettings::clipboardTimeoutChoices() const
{
    QVariantList choices;
    for (const int seconds : kClipboardTimeouts) {
        choices.append(seconds);
    }
    return choices;
}

QString AppSettings::customIconKey(const QString &credId) const
{
    return m_settings.value(iconGroup(credId) + QStringLiteral("/customIconKey")).toString();
}

QString AppSettings::faviconDomain(const QString &credId) const
{
    return m_settings.value(iconGroup(credId) + QStringLiteral("/faviconDomain")).toString();
}

void AppSettings::setIconPreference(const QString &credId,
                                    const QString &customIconKey,
                                    const QString &faviconDomain)
{
    if (customIconKey.isEmpty() && faviconDomain.isEmpty()) {
        clearIconPreference(credId);
        return;
    }
    const QString group = iconGroup(credId);
    m_settings.setValue(group + QStringLiteral("/customIconKey"), customIconKey);
    m_settings.setValue(group + QStringLiteral("/faviconDomain"), faviconDomain);
    Q_EMIT iconPreferencesChanged();
}

void AppSettings::clearIconPreference(const QString &credId)
{
    m_settings.remove(iconGroup(credId));
    Q_EMIT iconPreferencesChanged();
}
