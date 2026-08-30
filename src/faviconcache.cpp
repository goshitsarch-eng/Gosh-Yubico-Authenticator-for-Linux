// SPDX-License-Identifier: GPL-3.0-or-later

#include "faviconcache.h"

#include <QDateTime>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QRegularExpression>
#include <QSaveFile>
#include <QStandardPaths>

namespace {
constexpr qint64 kMaxAgeSecs = 7 * 24 * 60 * 60;
}

FaviconCache::FaviconCache(QObject *parent)
    : QObject(parent)
{
}

QString FaviconCache::cachePath(const QString &domain) const
{
    // Domains come from untrusted input; keep only safe filename characters.
    static const QRegularExpression unsafe(QStringLiteral("[^a-z0-9.-]"));
    QString safe = domain.toLower();
    safe.remove(unsafe);
    if (safe.isEmpty()) {
        return {};
    }
    const QString dir = QStandardPaths::writableLocation(QStandardPaths::CacheLocation)
        + QStringLiteral("/favicons");
    return dir + QLatin1Char('/') + safe + QStringLiteral(".png");
}

QUrl FaviconCache::cachedFavicon(const QString &domain)
{
    const QString path = cachePath(domain);
    if (path.isEmpty()) {
        return {};
    }

    const QFileInfo info(path);
    if (info.exists()) {
        const qint64 age = info.lastModified().secsTo(QDateTime::currentDateTime());
        if (age <= kMaxAgeSecs) {
            return QUrl::fromLocalFile(path);
        }
    }
    fetch(domain);
    return {};
}

void FaviconCache::fetch(const QString &domain)
{
    if (m_inFlight.contains(domain) || m_failed.contains(domain)) {
        return;
    }
    m_inFlight.insert(domain);

    const QUrl url(QStringLiteral("https://www.google.com/s2/favicons?domain=%1&sz=64").arg(domain));
    QNetworkRequest request(url);
    request.setTransferTimeout(10000);
    QNetworkReply *reply = m_network.get(request);
    connect(reply, &QNetworkReply::finished, this, [this, reply, domain]() {
        reply->deleteLater();
        m_inFlight.remove(domain);

        if (reply->error() != QNetworkReply::NoError) {
            m_failed.insert(domain);
            return;
        }
        const QByteArray data = reply->readAll();
        if (data.size() <= 100) {
            m_failed.insert(domain);
            return;
        }

        const QString path = cachePath(domain);
        QDir().mkpath(QFileInfo(path).absolutePath());
        QSaveFile file(path);
        if (!file.open(QIODevice::WriteOnly)) {
            return;
        }
        file.write(data);
        if (file.commit()) {
            Q_EMIT faviconReady(domain);
        }
    });
}
