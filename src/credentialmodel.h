// SPDX-License-Identifier: GPL-3.0-or-later
//
// List model of the credentials on the connected key, with search
// filtering and avatar/icon resolution.

#pragma once

#include "oath/oathsession.h"

#include <QAbstractListModel>
#include <QtQml/qqmlregistration.h>

class AppSettings;
class FaviconCache;

class CredentialModel : public QAbstractListModel {
    Q_OBJECT
    QML_ELEMENT
    QML_UNCREATABLE("Access through AppController.credentials")

    Q_PROPERTY(QString filterString READ filterString WRITE setFilterString NOTIFY filterStringChanged)
    Q_PROPERTY(int totalCount READ totalCount NOTIFY countsChanged)
    Q_PROPERTY(int totpCount READ totpCount NOTIFY countsChanged)
    Q_PROPERTY(int hotpCount READ hotpCount NOTIFY countsChanged)

public:
    enum Roles {
        CredIdRole = Qt::UserRole + 1,
        TitleRole,
        SubtitleRole,
        DisplayNameRole,
        IssuerRole,
        AccountRole,
        CodeRole,
        HasCodeRole,
        TouchRequiredRole,
        IsHotpRole,
        PeriodRole,
        IconLabelRole,
        IconColorRole,
        FaviconRole,
        GuessedDomainRole,
    };

    CredentialModel(AppSettings *settings, FaviconCache *favicons, QObject *parent = nullptr);

    int rowCount(const QModelIndex &parent = QModelIndex()) const override;
    QVariant data(const QModelIndex &index, int role) const override;
    QHash<int, QByteArray> roleNames() const override;

    QString filterString() const { return m_filter; }
    void setFilterString(const QString &filter);

    int totalCount() const { return m_all.size(); }
    int totpCount() const;
    int hotpCount() const;

    void setCredentials(const QList<oath::Credential> &credentials);
    void setCode(const QByteArray &id, const QString &code, int digits);
    void clear();

    /// The formatted code for a credential, empty when not calculated.
    QString codeFor(const QByteArray &id) const;

Q_SIGNALS:
    void filterStringChanged();
    void countsChanged();

private:
    void refilter();
    void refilterInternal();
    void refreshVisibleRows();

    AppSettings *m_settings;
    FaviconCache *m_favicons;
    QList<oath::Credential> m_all;
    QList<int> m_visible; // indexes into m_all, filtered
    QString m_filter;
};
