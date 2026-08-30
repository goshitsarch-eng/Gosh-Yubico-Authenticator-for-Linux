// SPDX-License-Identifier: GPL-3.0-or-later

#include "otpauth.h"

#include <QFile>
#include <QImage>
#include <QUrlQuery>

#if defined(HAVE_KIO)
#include <KIO/StoredTransferJob>
#endif

#if defined(HAVE_ZXING)
#include <ZXing/ReadBarcode.h>
#endif

namespace otpauth {

namespace {

QVariantMap makeError(const QString &message)
{
    QVariantMap map;
    map.insert(QStringLiteral("ok"), false);
    map.insert(QStringLiteral("error"), message);
    return map;
}

} // namespace

QVariantMap parseUri(const QString &uri)
{
    const QUrl url(uri.trimmed());
    if (!url.isValid()) {
        return makeError(QStringLiteral("Invalid URI format"));
    }
    if (url.scheme() != QStringLiteral("otpauth")) {
        return makeError(QStringLiteral("Not an otpauth URI (got %1://)").arg(url.scheme()));
    }

    const QString host = url.host().toLower();
    bool isHotp = false;
    if (host == QStringLiteral("hotp")) {
        isHotp = true;
    } else if (host != QStringLiteral("totp")) {
        return makeError(QStringLiteral("Unknown OTP type: %1").arg(host));
    }

    QString label = url.path(QUrl::FullyDecoded);
    while (label.startsWith(QLatin1Char('/'))) {
        label.remove(0, 1);
    }

    QString issuer;
    QString account = label.trimmed();
    const int colon = label.indexOf(QLatin1Char(':'));
    if (colon >= 0) {
        issuer = label.left(colon).trimmed();
        account = label.mid(colon + 1).trimmed();
    }
    if (account.isEmpty()) {
        return makeError(QStringLiteral("Account name is required"));
    }

    const QUrlQuery query(url);
    const QString secret = query.queryItemValue(QStringLiteral("secret"), QUrl::FullyDecoded);
    if (secret.isEmpty()) {
        return makeError(QStringLiteral("Secret key is required"));
    }

    const QString issuerParam = query.queryItemValue(QStringLiteral("issuer"), QUrl::FullyDecoded);
    if (!issuerParam.isEmpty()) {
        issuer = issuerParam;
    }

    int algorithmIndex = 0;
    const QString algorithm = query.queryItemValue(QStringLiteral("algorithm")).toUpper();
    if (algorithm.isEmpty() || algorithm == QStringLiteral("SHA1")) {
        algorithmIndex = 0;
    } else if (algorithm == QStringLiteral("SHA256")) {
        algorithmIndex = 1;
    } else if (algorithm == QStringLiteral("SHA512")) {
        algorithmIndex = 2;
    } else {
        return makeError(QStringLiteral("Unsupported algorithm: %1").arg(algorithm));
    }

    int digits = 6;
    const QString digitsParam = query.queryItemValue(QStringLiteral("digits"));
    if (!digitsParam.isEmpty()) {
        bool ok = false;
        digits = digitsParam.toInt(&ok);
        if (!ok || digits < 6 || digits > 8) {
            return makeError(QStringLiteral("Digits must be 6, 7, or 8"));
        }
    }

    int period = 30;
    const QString periodParam = query.queryItemValue(QStringLiteral("period"));
    if (!isHotp && !periodParam.isEmpty()) {
        bool ok = false;
        const int value = periodParam.toInt(&ok);
        if (ok && value > 0) {
            period = value;
        }
    }

    int counter = 0;
    const QString counterParam = query.queryItemValue(QStringLiteral("counter"));
    if (isHotp && !counterParam.isEmpty()) {
        bool ok = false;
        const int value = counterParam.toInt(&ok);
        if (ok && value >= 0) {
            counter = value;
        }
    }

    QVariantMap map;
    map.insert(QStringLiteral("ok"), true);
    map.insert(QStringLiteral("isHotp"), isHotp);
    map.insert(QStringLiteral("issuer"), issuer);
    map.insert(QStringLiteral("account"), account);
    map.insert(QStringLiteral("secret"), secret);
    map.insert(QStringLiteral("algorithmIndex"), algorithmIndex);
    map.insert(QStringLiteral("digits"), digits);
    map.insert(QStringLiteral("period"), period);
    map.insert(QStringLiteral("counter"), counter);
    return map;
}

namespace {

/// Read a picked file wherever it lives. Local paths are read directly;
/// anything else (smb://, sftp://, ...) goes through KIO so network shares
/// picked in the file dialog work without a handoff to another application.
bool readUrl(const QUrl &url, QByteArray *data, QString *error)
{
    if (url.isLocalFile()) {
        QFile file(url.toLocalFile());
        if (!file.open(QIODevice::ReadOnly)) {
            *error = QStringLiteral("Could not open file: %1").arg(file.errorString());
            return false;
        }
        *data = file.readAll();
        return true;
    }

#if defined(HAVE_KIO)
    KIO::StoredTransferJob *job = KIO::storedGet(url, KIO::NoReload, KIO::HideProgressInfo);
    if (!job->exec()) {
        *error = job->errorString();
        return false;
    }
    *data = job->data();
    return true;
#else
    Q_UNUSED(data)
    *error = QStringLiteral("Remote locations are not supported in this build");
    return false;
#endif
}

} // namespace

QVariantMap scanQrUrl(const QUrl &url)
{
#if !defined(HAVE_ZXING)
    Q_UNUSED(url)
    return makeError(QStringLiteral("QR scanning is not available in this build"));
#else
    QByteArray data;
    QString error;
    if (!readUrl(url, &data, &error)) {
        return makeError(error);
    }

    QImage image = QImage::fromData(data);
    if (image.isNull()) {
        return makeError(QStringLiteral("Could not decode image"));
    }
    image = image.convertToFormat(QImage::Format_Grayscale8);

    const ZXing::ImageView view(image.constBits(),
                                image.width(),
                                image.height(),
                                ZXing::ImageFormat::Lum,
                                static_cast<int>(image.bytesPerLine()));
    const ZXing::Result result = ZXing::ReadBarcode(view);
    if (!result.isValid()) {
        return makeError(QStringLiteral("No QR code found in image"));
    }

    const QString content = QString::fromStdString(result.text());
    if (content.isEmpty()) {
        return makeError(QStringLiteral("QR code is empty"));
    }
    return parseUri(content);
#endif
}

} // namespace otpauth
