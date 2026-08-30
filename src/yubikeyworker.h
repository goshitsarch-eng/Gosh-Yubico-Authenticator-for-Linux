// SPDX-License-Identifier: GPL-3.0-or-later
//
// Background worker owning the blocking PC/SC session. Lives on a worker
// thread; the controller talks to it exclusively through queued
// signal/slot connections.

#pragma once

#include "oath/oathsession.h"

#include <QObject>

#include <memory>

class YubiKeyWorker : public QObject {
    Q_OBJECT

public:
    explicit YubiKeyWorker(QObject *parent = nullptr);
    ~YubiKeyWorker() override;

public Q_SLOTS:
    void connectToKey();
    void authenticate(const QString &password);
    void refresh();
    void calculateCredential(const QByteArray &id);
    void addCredential(const QString &issuer,
                       const QString &account,
                       const QString &secretBase32,
                       bool isHotp,
                       int algorithmIndex, // 0 = SHA1, 1 = SHA256, 2 = SHA512
                       int digits,
                       bool requireTouch);
    void deleteCredential(const QByteArray &id);
    void setPassword(const QString &password);
    void disconnectFromKey();

Q_SIGNALS:
    void connected(int versionMajor, int versionMinor, int versionPatch, bool hasPassword);
    void disconnected();
    void authenticationRequired();
    void authenticationSucceeded();
    void authenticationFailed(const QString &message);
    void credentialsUpdated(const QList<oath::Credential> &credentials);
    void credentialCalculated(const QByteArray &id, const QString &code, int digits);
    void touchRequired(const QByteArray &id);
    void credentialAdded();
    void credentialDeleted();
    void passwordChanged(bool removed);
    void errorOccurred(const QString &message);

private:
    void handleFailure(const oath::OathException &error);

    std::unique_ptr<oath::OathSession> m_session;
};
