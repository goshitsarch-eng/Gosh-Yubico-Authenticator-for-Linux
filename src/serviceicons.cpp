// SPDX-License-Identifier: GPL-3.0-or-later

#include "serviceicons.h"

#include <QVariantMap>

namespace serviceicons {

const QList<ServiceIcon> &allServices()
{
    static const QList<ServiceIcon> services = {
        {QStringLiteral("gmail"), QStringLiteral("Gmail"), QColor(0xEA, 0x43, 0x35)},
        {QStringLiteral("aws"), QStringLiteral("AWS"), QColor(0xFF, 0x99, 0x00)},
        {QStringLiteral("google"), QStringLiteral("Google"), QColor(0x42, 0x85, 0xF4)},
        {QStringLiteral("microsoft"), QStringLiteral("Microsoft"), QColor(0x00, 0xA4, 0xEF)},
        {QStringLiteral("cloudflare"), QStringLiteral("Cloudflare"), QColor(0xF3, 0x80, 0x20)},
        {QStringLiteral("digitalocean"), QStringLiteral("DigitalOcean"), QColor(0x00, 0x84, 0xFF)},
        {QStringLiteral("github"), QStringLiteral("GitHub"), QColor(0x18, 0x17, 0x17)},
        {QStringLiteral("gitlab"), QStringLiteral("GitLab"), QColor(0xFC, 0x6D, 0x26)},
        {QStringLiteral("bitbucket"), QStringLiteral("Bitbucket"), QColor(0x00, 0x52, 0xCC)},
        {QStringLiteral("docker"), QStringLiteral("Docker"), QColor(0x24, 0x96, 0xED)},
        {QStringLiteral("npm"), QStringLiteral("npm"), QColor(0xCB, 0x38, 0x37)},
        {QStringLiteral("openai"), QStringLiteral("OpenAI"), QColor(0x41, 0x2A, 0x4C)},
        {QStringLiteral("discord"), QStringLiteral("Discord"), QColor(0x58, 0x65, 0xF2)},
        {QStringLiteral("slack"), QStringLiteral("Slack"), QColor(0x4A, 0x15, 0x4B)},
        {QStringLiteral("twitter"), QStringLiteral("X"), QColor(0x00, 0x00, 0x00)},
        {QStringLiteral("facebook"), QStringLiteral("Facebook"), QColor(0x18, 0x77, 0xF2)},
        {QStringLiteral("linkedin"), QStringLiteral("LinkedIn"), QColor(0x0A, 0x66, 0xC2)},
        {QStringLiteral("reddit"), QStringLiteral("Reddit"), QColor(0xFF, 0x45, 0x00)},
        {QStringLiteral("zapier"), QStringLiteral("Zapier"), QColor(0xFF, 0x4A, 0x00)},
        {QStringLiteral("paypal"), QStringLiteral("PayPal"), QColor(0x00, 0x30, 0x87)},
        {QStringLiteral("stripe"), QStringLiteral("Stripe"), QColor(0x63, 0x5B, 0xFF)},
        {QStringLiteral("steam"), QStringLiteral("Steam"), QColor(0x17, 0x1A, 0x21)},
        {QStringLiteral("twitch"), QStringLiteral("Twitch"), QColor(0x91, 0x46, 0xFF)},
        {QStringLiteral("epic"), QStringLiteral("Epic Games"), QColor(0x31, 0x31, 0x31)},
        {QStringLiteral("1password"), QStringLiteral("1Password"), QColor(0x00, 0x9B, 0xDE)},
        {QStringLiteral("bitwarden"), QStringLiteral("Bitwarden"), QColor(0x17, 0x50, 0xDD)},
        {QStringLiteral("dropbox"), QStringLiteral("Dropbox"), QColor(0x00, 0x61, 0xFF)},
        {QStringLiteral("apple"), QStringLiteral("Apple"), QColor(0x55, 0x55, 0x55)},
        {QStringLiteral("spotify"), QStringLiteral("Spotify"), QColor(0x1D, 0xB9, 0x54)},
        {QStringLiteral("netflix"), QStringLiteral("Netflix"), QColor(0xE5, 0x09, 0x14)},
        {QStringLiteral("proton"), QStringLiteral("Proton"), QColor(0x6D, 0x4A, 0xFF)},
    };
    return services;
}

namespace {

struct Keyword {
    QStringList words;
    QString key;
};

const QList<Keyword> &keywords()
{
    static const QList<Keyword> list = {
        {{QStringLiteral("gmail")}, QStringLiteral("gmail")},
        {{QStringLiteral("aws"), QStringLiteral("amazon web services")}, QStringLiteral("aws")},
        {{QStringLiteral("google"), QStringLiteral("gcp")}, QStringLiteral("google")},
        {{QStringLiteral("microsoft"), QStringLiteral("azure"), QStringLiteral("outlook"), QStringLiteral("office365")},
         QStringLiteral("microsoft")},
        {{QStringLiteral("cloudflare")}, QStringLiteral("cloudflare")},
        {{QStringLiteral("digitalocean")}, QStringLiteral("digitalocean")},
        {{QStringLiteral("github")}, QStringLiteral("github")},
        {{QStringLiteral("gitlab")}, QStringLiteral("gitlab")},
        {{QStringLiteral("bitbucket")}, QStringLiteral("bitbucket")},
        {{QStringLiteral("docker")}, QStringLiteral("docker")},
        {{QStringLiteral("npm")}, QStringLiteral("npm")},
        {{QStringLiteral("openai"), QStringLiteral("chatgpt")}, QStringLiteral("openai")},
        {{QStringLiteral("discord")}, QStringLiteral("discord")},
        {{QStringLiteral("slack")}, QStringLiteral("slack")},
        {{QStringLiteral("twitter"), QStringLiteral("x.com")}, QStringLiteral("twitter")},
        {{QStringLiteral("facebook"), QStringLiteral("meta")}, QStringLiteral("facebook")},
        {{QStringLiteral("linkedin")}, QStringLiteral("linkedin")},
        {{QStringLiteral("reddit")}, QStringLiteral("reddit")},
        {{QStringLiteral("zapier")}, QStringLiteral("zapier")},
        {{QStringLiteral("paypal")}, QStringLiteral("paypal")},
        {{QStringLiteral("stripe")}, QStringLiteral("stripe")},
        {{QStringLiteral("steam")}, QStringLiteral("steam")},
        {{QStringLiteral("twitch")}, QStringLiteral("twitch")},
        {{QStringLiteral("epic"), QStringLiteral("epicgames")}, QStringLiteral("epic")},
        {{QStringLiteral("1password"), QStringLiteral("onepassword")}, QStringLiteral("1password")},
        {{QStringLiteral("bitwarden")}, QStringLiteral("bitwarden")},
        {{QStringLiteral("dropbox")}, QStringLiteral("dropbox")},
        {{QStringLiteral("apple"), QStringLiteral("icloud")}, QStringLiteral("apple")},
        {{QStringLiteral("spotify")}, QStringLiteral("spotify")},
        {{QStringLiteral("netflix")}, QStringLiteral("netflix")},
        {{QStringLiteral("proton"), QStringLiteral("protonmail")}, QStringLiteral("proton")},
    };
    return list;
}

} // namespace

const ServiceIcon *serviceByKey(const QString &key)
{
    for (const ServiceIcon &service : allServices()) {
        if (service.key.compare(key, Qt::CaseInsensitive) == 0) {
            return &service;
        }
    }
    return nullptr;
}

const ServiceIcon *guessService(const QString &issuer, const QString &account)
{
    const QString name = (issuer.isEmpty() ? account : issuer).toLower();
    for (const Keyword &keyword : keywords()) {
        for (const QString &word : keyword.words) {
            if (name.contains(word)) {
                return serviceByKey(keyword.key);
            }
        }
    }
    return nullptr;
}

QString guessDomain(const QString &issuer, const QString &account)
{
    const int at = account.indexOf(QLatin1Char('@'));
    if (at >= 0) {
        const QString domain = account.mid(at + 1).trimmed().toLower();
        if (!domain.isEmpty() && domain.contains(QLatin1Char('.'))) {
            return domain;
        }
    }
    const QString cleaned = issuer.trimmed().toLower();
    if (cleaned.contains(QLatin1Char('.'))) {
        return cleaned;
    }
    return {};
}

QString extractDomainFromText(const QString &text)
{
    QString cleaned = text.trimmed().toLower();
    if (!cleaned.contains(QLatin1Char('.')) || cleaned.contains(QLatin1Char(' '))) {
        return {};
    }
    if (cleaned.startsWith(QStringLiteral("https://"))) {
        cleaned.remove(0, 8);
    } else if (cleaned.startsWith(QStringLiteral("http://"))) {
        cleaned.remove(0, 7);
    }
    while (cleaned.endsWith(QLatin1Char('/'))) {
        cleaned.chop(1);
    }
    return cleaned;
}

QVariantList allServicesVariant()
{
    QVariantList list;
    for (const ServiceIcon &service : allServices()) {
        QVariantMap entry;
        entry.insert(QStringLiteral("key"), service.key);
        entry.insert(QStringLiteral("label"), service.label);
        entry.insert(QStringLiteral("color"), service.color);
        list.append(entry);
    }
    return list;
}

} // namespace serviceicons
