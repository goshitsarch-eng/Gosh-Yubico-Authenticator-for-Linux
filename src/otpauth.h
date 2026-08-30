// SPDX-License-Identifier: GPL-3.0-or-later
//
// otpauth:// URI parsing and QR code import.

#pragma once

#include <QString>
#include <QUrl>
#include <QVariantMap>

namespace otpauth {

/// Parse an otpauth://TYPE/LABEL?PARAMETERS URI.
///
/// Returns a map for QML consumption:
///   ok (bool), error (QString, when !ok),
///   isHotp (bool), issuer, account, secret (QString),
///   algorithmIndex (0=SHA1 1=SHA256 2=SHA512), digits (int),
///   period (int, TOTP), counter (int, HOTP)
QVariantMap parseUri(const QString &uri);

/// Decode the first QR code found in an image and parse it as an otpauth
/// URI. Accepts any URL the platform can read: local files directly, and
/// remote locations (smb://, sftp://, ...) through KIO when built with it,
/// so files picked from network shares open in-app instead of failing.
QVariantMap scanQrUrl(const QUrl &url);

} // namespace otpauth
