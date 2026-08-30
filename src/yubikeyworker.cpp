// SPDX-License-Identifier: GPL-3.0-or-later

#include "yubikeyworker.h"

#include <QDateTime>

using oath::Credential;
using oath::ErrorCode;
using oath::OathException;
using oath::OathSession;

YubiKeyWorker::YubiKeyWorker(QObject *parent)
    : QObject(parent)
{
}

YubiKeyWorker::~YubiKeyWorker() = default;

void YubiKeyWorker::handleFailure(const OathException &error)
{
    switch (error.code()) {
    case ErrorCode::NoDevice:
    case ErrorCode::Disconnected:
        m_session.reset();
        Q_EMIT disconnected();
        break;
    case ErrorCode::AuthenticationRequired:
        Q_EMIT authenticationRequired();
        break;
    default:
        Q_EMIT errorOccurred(error.message());
        break;
    }
}

void YubiKeyWorker::connectToKey()
{
    try {
        m_session = OathSession::connect();
    } catch (const OathException &e) {
        m_session.reset();
        if (e.code() == ErrorCode::NoDevice) {
            Q_EMIT disconnected();
        } else {
            Q_EMIT errorOccurred(e.message());
        }
        return;
    }

    const oath::Version version = m_session->version();
    Q_EMIT connected(version.major, version.minor, version.patch, m_session->hasPassword());

    if (m_session->requiresAuth()) {
        Q_EMIT authenticationRequired();
    } else {
        refresh();
    }
}

void YubiKeyWorker::authenticate(const QString &password)
{
    if (!m_session) {
        Q_EMIT disconnected();
        return;
    }
    try {
        m_session->validate(password);
    } catch (const OathException &e) {
        if (e.code() == ErrorCode::WrongPassword) {
            Q_EMIT authenticationFailed(QStringLiteral("Incorrect password"));
        } else {
            handleFailure(e);
        }
        return;
    }
    Q_EMIT authenticationSucceeded();
    refresh();
}

void YubiKeyWorker::refresh()
{
    if (!m_session) {
        Q_EMIT disconnected();
        return;
    }

    QList<Credential> credentials;
    try {
        credentials = m_session->listCredentials();

        const auto results = m_session->calculateAll(QDateTime::currentSecsSinceEpoch());
        for (const auto &result : results) {
            for (Credential &credential : credentials) {
                if (credential.id != result.id) {
                    continue;
                }
                if (result.code) {
                    credential.code = oath::formatCode(result.code->first, result.code->second);
                    credential.digits = result.code->second;
                } else {
                    // Touch-required TOTP or HOTP: needs individual calculation.
                    credential.touchRequired = credential.type == oath::OathType::Totp;
                }
                break;
            }
        }
    } catch (const OathException &e) {
        handleFailure(e);
        return;
    }

    Q_EMIT credentialsUpdated(credentials);
}

void YubiKeyWorker::calculateCredential(const QByteArray &id)
{
    if (!m_session) {
        Q_EMIT disconnected();
        return;
    }
    try {
        const QList<Credential> credentials = m_session->listCredentials();
        const Credential *match = nullptr;
        for (const Credential &credential : credentials) {
            if (credential.id == id) {
                match = &credential;
                break;
            }
        }
        if (!match) {
            Q_EMIT errorOccurred(QStringLiteral("Credential not found"));
            return;
        }
        const auto [code, digits] = m_session->calculate(*match, QDateTime::currentSecsSinceEpoch());
        Q_EMIT credentialCalculated(id, oath::formatCode(code, digits), digits);
    } catch (const OathException &e) {
        if (e.code() == ErrorCode::TouchRequired) {
            Q_EMIT touchRequired(id);
        } else {
            handleFailure(e);
        }
    }
}

void YubiKeyWorker::addCredential(const QString &issuer,
                                  const QString &account,
                                  const QString &secretBase32,
                                  bool isHotp,
                                  int algorithmIndex,
                                  int digits,
                                  bool requireTouch)
{
    if (!m_session) {
        Q_EMIT disconnected();
        return;
    }

    const auto secret = oath::decodeBase32Secret(secretBase32);
    if (!secret || secret->isEmpty()) {
        Q_EMIT errorOccurred(QStringLiteral("Invalid base32 secret"));
        return;
    }

    const oath::Algorithm algorithm = algorithmIndex == 1 ? oath::Algorithm::Sha256
        : algorithmIndex == 2                             ? oath::Algorithm::Sha512
                                                          : oath::Algorithm::Sha1;
    try {
        m_session->putCredential(issuer.trimmed(),
                                 account.trimmed(),
                                 *secret,
                                 isHotp ? oath::OathType::Hotp : oath::OathType::Totp,
                                 algorithm,
                                 digits,
                                 requireTouch,
                                 isHotp ? std::optional<quint32>(0) : std::nullopt);
    } catch (const OathException &e) {
        handleFailure(e);
        return;
    }
    Q_EMIT credentialAdded();
    refresh();
}

void YubiKeyWorker::deleteCredential(const QByteArray &id)
{
    if (!m_session) {
        Q_EMIT disconnected();
        return;
    }
    try {
        m_session->deleteCredential(id);
    } catch (const OathException &e) {
        handleFailure(e);
        return;
    }
    Q_EMIT credentialDeleted();
    refresh();
}

void YubiKeyWorker::setPassword(const QString &password)
{
    if (!m_session) {
        Q_EMIT disconnected();
        return;
    }
    try {
        m_session->setCode(password);
    } catch (const OathException &e) {
        handleFailure(e);
        return;
    }
    Q_EMIT passwordChanged(password.isEmpty());
}

void YubiKeyWorker::disconnectFromKey()
{
    m_session.reset();
    Q_EMIT disconnected();
}
