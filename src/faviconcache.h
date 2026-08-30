// SPDX-License-Identifier: GPL-3.0-or-later
//
// Small on-disk favicon cache with background fetching. Fetching is
// best-effort: without network access (e.g. the Flatpak sandbox) the app
// silently keeps the colored letter avatars.

#pragma once

#include <QNetworkAccessManager>
#include <QObject>
#include <QSet>
#include <QString>
#include <QUrl>

class FaviconCache : public QObject {
    Q_OBJECT

public:
    explicit FaviconCache(QObject *parent = nullptr);

    /// Cached favicon for a domain, or an empty URL. Schedules a background
    /// fetch on a miss; faviconReady() fires when it lands.
    QUrl cachedFavicon(const QString &domain);

Q_SIGNALS:
    void faviconReady(const QString &domain);

private:
    QString cachePath(const QString &domain) const;
    void fetch(const QString &domain);

    QNetworkAccessManager m_network;
    QSet<QString> m_inFlight;
    QSet<QString> m_failed;
};
