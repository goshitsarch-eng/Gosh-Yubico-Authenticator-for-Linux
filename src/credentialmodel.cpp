// SPDX-License-Identifier: GPL-3.0-or-later

#include "credentialmodel.h"

#include "appsettings.h"
#include "faviconcache.h"
#include "serviceicons.h"

using oath::Credential;

CredentialModel::CredentialModel(AppSettings *settings, FaviconCache *favicons, QObject *parent)
    : QAbstractListModel(parent)
    , m_settings(settings)
    , m_favicons(favicons)
{
    connect(m_settings, &AppSettings::iconPreferencesChanged, this, &CredentialModel::refreshVisibleRows);
    connect(m_favicons, &FaviconCache::faviconReady, this, [this](const QString &) {
        refreshVisibleRows();
    });
}

int CredentialModel::rowCount(const QModelIndex &parent) const
{
    return parent.isValid() ? 0 : m_visible.size();
}

QHash<int, QByteArray> CredentialModel::roleNames() const
{
    return {
        {CredIdRole, "credId"},
        {TitleRole, "title"},
        {SubtitleRole, "subtitle"},
        {DisplayNameRole, "displayName"},
        {IssuerRole, "issuer"},
        {AccountRole, "account"},
        {CodeRole, "code"},
        {HasCodeRole, "hasCode"},
        {TouchRequiredRole, "touchRequired"},
        {IsHotpRole, "isHotp"},
        {PeriodRole, "period"},
        {IconLabelRole, "iconLabel"},
        {IconColorRole, "iconColor"},
        {FaviconRole, "favicon"},
        {GuessedDomainRole, "guessedDomain"},
    };
}

QVariant CredentialModel::data(const QModelIndex &index, int role) const
{
    if (!index.isValid() || index.row() < 0 || index.row() >= m_visible.size()) {
        return {};
    }
    const Credential &cred = m_all.at(m_visible.at(index.row()));
    const QString credId = QString::fromLatin1(cred.id.toBase64());

    switch (role) {
    case CredIdRole:
        return credId;
    case TitleRole:
        return cred.issuer.isEmpty() ? cred.account : cred.issuer;
    case SubtitleRole:
        return cred.issuer.isEmpty() ? QString() : cred.account;
    case DisplayNameRole:
        return cred.displayName();
    case IssuerRole:
        return cred.issuer;
    case AccountRole:
        return cred.account;
    case CodeRole:
        return cred.code;
    case HasCodeRole:
        return !cred.code.isEmpty();
    case TouchRequiredRole:
        return cred.touchRequired;
    case IsHotpRole:
        return cred.type == oath::OathType::Hotp;
    case PeriodRole:
        return cred.period;
    case GuessedDomainRole:
        return serviceicons::guessDomain(cred.issuer, cred.account);
    case IconLabelRole:
    case IconColorRole: {
        const QString customKey = m_settings->customIconKey(credId);
        const serviceicons::ServiceIcon *service = customKey.isEmpty()
            ? serviceicons::guessService(cred.issuer, cred.account)
            : serviceicons::serviceByKey(customKey);
        if (service) {
            return role == IconLabelRole ? QVariant(service->label.left(1).toUpper())
                                         : QVariant(service->color);
        }
        if (role == IconColorRole) {
            return QColor(0x35, 0x84, 0xE4);
        }
        const QString source = cred.issuer.isEmpty() ? cred.account : cred.issuer;
        for (const QChar &c : source) {
            if (c.isLetterOrNumber()) {
                return QString(c.toUpper());
            }
        }
        return QStringLiteral("?");
    }
    case FaviconRole: {
        QString domain = m_settings->faviconDomain(credId);
        if (domain.isEmpty()) {
            domain = serviceicons::guessDomain(cred.issuer, cred.account);
        }
        if (domain.isEmpty()) {
            return QUrl();
        }
        return m_favicons->cachedFavicon(domain);
    }
    default:
        return {};
    }
}

void CredentialModel::setFilterString(const QString &filter)
{
    if (m_filter == filter) {
        return;
    }
    m_filter = filter;
    Q_EMIT filterStringChanged();
    refilter();
}

int CredentialModel::totpCount() const
{
    int count = 0;
    for (const Credential &cred : m_all) {
        if (cred.type == oath::OathType::Totp) {
            ++count;
        }
    }
    return count;
}

int CredentialModel::hotpCount() const
{
    return m_all.size() - totpCount();
}

void CredentialModel::setCredentials(const QList<Credential> &credentials)
{
    beginResetModel();
    m_all = credentials;
    refilterInternal();
    endResetModel();
    Q_EMIT countsChanged();
}

void CredentialModel::setCode(const QByteArray &id, const QString &code, int digits)
{
    for (int i = 0; i < m_all.size(); ++i) {
        if (m_all.at(i).id != id) {
            continue;
        }
        m_all[i].code = code;
        m_all[i].digits = digits;
        const int row = m_visible.indexOf(i);
        if (row >= 0) {
            const QModelIndex idx = index(row);
            Q_EMIT dataChanged(idx, idx, {CodeRole, HasCodeRole});
        }
        return;
    }
}

void CredentialModel::clear()
{
    setCredentials({});
}

QString CredentialModel::codeFor(const QByteArray &id) const
{
    for (const Credential &cred : m_all) {
        if (cred.id == id) {
            return cred.code;
        }
    }
    return {};
}

void CredentialModel::refilter()
{
    beginResetModel();
    refilterInternal();
    endResetModel();
}

void CredentialModel::refilterInternal()
{
    m_visible.clear();
    const QString query = m_filter.trimmed().toLower();
    for (int i = 0; i < m_all.size(); ++i) {
        const Credential &cred = m_all.at(i);
        if (query.isEmpty() || cred.displayName().toLower().contains(query)
            || cred.account.toLower().contains(query)) {
            m_visible.append(i);
        }
    }
}

void CredentialModel::refreshVisibleRows()
{
    if (!m_visible.isEmpty()) {
        Q_EMIT dataChanged(index(0), index(m_visible.size() - 1), {IconLabelRole, IconColorRole, FaviconRole});
    }
}
