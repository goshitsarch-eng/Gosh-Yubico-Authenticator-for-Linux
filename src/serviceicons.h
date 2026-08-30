// SPDX-License-Identifier: GPL-3.0-or-later
//
// Brand colors and label guessing for credential avatars.

#pragma once

#include <QColor>
#include <QString>
#include <QVariantList>

namespace serviceicons {

struct ServiceIcon {
    QString key;
    QString label;
    QColor color;
};

/// All known services, for the icon picker.
const QList<ServiceIcon> &allServices();

/// Look up a service by its stable key.
const ServiceIcon *serviceByKey(const QString &key);

/// Guess the service from issuer/account keywords.
const ServiceIcon *guessService(const QString &issuer, const QString &account);

/// Best-effort domain for a credential, used for favicon lookup.
QString guessDomain(const QString &issuer, const QString &account);

/// Extract a domain from arbitrary user text ("https://foo.com/" -> "foo.com").
QString extractDomainFromText(const QString &text);

/// The picker list as QML-consumable maps: {key, label, color}.
QVariantList allServicesVariant();

} // namespace serviceicons
