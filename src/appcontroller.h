// SPDX-License-Identifier: GPL-3.0-or-later
//
// The QML-facing application controller: owns the worker thread, the
// credential model, settings, clipboard, and connection state.

#pragma once

#include "appsettings.h"
#include "credentialmodel.h"

#include <QObject>
#include <QThread>
#include <QTimer>
#include <QUrl>
#include <QVariantList>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>

class ClipboardService;
class FaviconCache;
class YubiKeyWorker;

class AppController : public QObject {
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    Q_PROPERTY(CredentialModel *credentials READ credentials CONSTANT)
    Q_PROPERTY(AppSettings *settings READ settings CONSTANT)
    Q_PROPERTY(int connectionState READ connectionState NOTIFY connectionStateChanged)
    Q_PROPERTY(QString deviceName READ deviceName NOTIFY deviceInfoChanged)
    Q_PROPERTY(QString firmwareVersion READ firmwareVersion NOTIFY deviceInfoChanged)
    Q_PROPERTY(bool hasPassword READ hasPassword NOTIFY deviceInfoChanged)
    Q_PROPERTY(QString appName READ appName CONSTANT)
    Q_PROPERTY(QString appVersion READ appVersion CONSTANT)
    Q_PROPERTY(QVariantMap aboutData READ aboutData CONSTANT)
    Q_PROPERTY(QVariantList serviceChoices READ serviceChoices CONSTANT)
    /// Monotonic-ish wall clock (seconds, fractional) driving TOTP rings.
    Q_PROPERTY(double nowSeconds READ nowSeconds NOTIFY ticked)

public:
    enum ConnectionState {
        Disconnected,
        Connecting,
        Connected,
    };
    Q_ENUM(ConnectionState)

    explicit AppController(QObject *parent = nullptr);
    ~AppController() override;

    CredentialModel *credentials() const { return m_model; }
    AppSettings *settings() const { return m_settings; }
    int connectionState() const { return m_connectionState; }
    QString deviceName() const;
    QString firmwareVersion() const;
    bool hasPassword() const { return m_hasPassword; }
    QString appName() const;
    QString appVersion() const;
    QVariantMap aboutData() const;
    QVariantList serviceChoices() const;
    double nowSeconds() const;

    Q_INVOKABLE void retryConnect();
    Q_INVOKABLE void refresh();
    Q_INVOKABLE void authenticate(const QString &password);
    Q_INVOKABLE void calculate(const QString &credId);
    Q_INVOKABLE void copyCode(const QString &credId);
    Q_INVOKABLE void addCredential(const QString &issuer,
                                   const QString &account,
                                   const QString &secret,
                                   bool isHotp,
                                   int algorithmIndex,
                                   int digits,
                                   bool requireTouch);
    Q_INVOKABLE void deleteCredential(const QString &credId);
    Q_INVOKABLE void setPassword(const QString &password);
    Q_INVOKABLE bool isSecretValid(const QString &secret) const;
    Q_INVOKABLE QVariantMap parseOtpauth(const QString &uri) const;
    Q_INVOKABLE QVariantMap importQrFile(const QUrl &url) const;
    /// Fraction of the TOTP period remaining, in [0, 1].
    Q_INVOKABLE double totpProgress(int period) const;

Q_SIGNALS:
    void connectionStateChanged();
    void deviceInfoChanged();
    void ticked();
    void showMessage(const QString &message);
    void authenticationRequired();
    void authenticationFailed(const QString &message);
    void authenticationSucceeded();
    void touchRequired();
    void touchResolved();
    void credentialCopied(const QString &credId, int clearSeconds);

private:
    void setConnectionState(int state);
    void tick();

    QThread m_workerThread;
    YubiKeyWorker *m_worker = nullptr;
    AppSettings *m_settings = nullptr;
    FaviconCache *m_favicons = nullptr;
    CredentialModel *m_model = nullptr;
    ClipboardService *m_clipboard = nullptr;
    QTimer m_timer;

    int m_connectionState = Connecting;
    int m_versionMajor = 0;
    int m_versionMinor = 0;
    int m_versionPatch = 0;
    bool m_hasPassword = false;
    qint64 m_lastPeriodSlot = 0;
    int m_reconnectCountdown = 0;
};
