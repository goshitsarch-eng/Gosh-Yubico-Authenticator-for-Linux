// SPDX-License-Identifier: GPL-3.0-or-later

#include "appcontroller.h"

#include "appsettings.h"
#include "clipboardservice.h"
#include "credentialmodel.h"
#include "faviconcache.h"
#include "otpauth.h"
#include "serviceicons.h"
#include "thememanager.h"
#include "version.h"
#include "yubikeyworker.h"

#include <QDateTime>
#include <QMetaObject>

#include <cmath>

using oath::Credential;

AppController::AppController(QObject *parent)
    : QObject(parent)
{
    m_settings = new AppSettings(this);
    m_favicons = new FaviconCache(this);
    m_model = new CredentialModel(m_settings, m_favicons, this);
    m_clipboard = new ClipboardService(this);

    thememanager::applyTheme(m_settings->themeMode());
    connect(m_settings, &AppSettings::themeModeChanged, this, [this]() {
        thememanager::applyTheme(m_settings->themeMode());
    });

    m_worker = new YubiKeyWorker;
    m_worker->moveToThread(&m_workerThread);
    connect(&m_workerThread, &QThread::finished, m_worker, &QObject::deleteLater);

    connect(m_worker, &YubiKeyWorker::connected, this,
            [this](int major, int minor, int patch, bool hasPassword) {
                m_versionMajor = major;
                m_versionMinor = minor;
                m_versionPatch = patch;
                m_hasPassword = hasPassword;
                Q_EMIT deviceInfoChanged();
                if (m_connectionState != Connected) {
                    setConnectionState(Connected);
                    Q_EMIT showMessage(tr("YubiKey connected"));
                }
            });
    connect(m_worker, &YubiKeyWorker::disconnected, this, [this]() {
        m_model->clear();
        m_hasPassword = false;
        Q_EMIT deviceInfoChanged();
        setConnectionState(Disconnected);
    });
    connect(m_worker, &YubiKeyWorker::authenticationRequired, this, [this]() {
        setConnectionState(Connected);
        Q_EMIT authenticationRequired();
    });
    connect(m_worker, &YubiKeyWorker::authenticationSucceeded, this, [this]() {
        Q_EMIT authenticationSucceeded();
        Q_EMIT showMessage(tr("YubiKey unlocked"));
    });
    connect(m_worker, &YubiKeyWorker::authenticationFailed, this,
            &AppController::authenticationFailed);
    connect(m_worker, &YubiKeyWorker::credentialsUpdated, this,
            [this](const QList<Credential> &credentials) {
                m_model->setCredentials(credentials);
            });
    connect(m_worker, &YubiKeyWorker::credentialCalculated, this,
            [this](const QByteArray &id, const QString &code, int digits) {
                m_model->setCode(id, code, digits);
                Q_EMIT touchResolved();
            });
    connect(m_worker, &YubiKeyWorker::touchRequired, this, [this](const QByteArray &) {
        Q_EMIT touchRequired();
    });
    connect(m_worker, &YubiKeyWorker::credentialAdded, this, [this]() {
        Q_EMIT showMessage(tr("Credential added"));
    });
    connect(m_worker, &YubiKeyWorker::credentialDeleted, this, [this]() {
        Q_EMIT showMessage(tr("Credential deleted"));
    });
    connect(m_worker, &YubiKeyWorker::passwordChanged, this, [this](bool removed) {
        m_hasPassword = !removed;
        Q_EMIT deviceInfoChanged();
        Q_EMIT showMessage(removed ? tr("Password protection removed")
                                   : tr("YubiKey password changed"));
    });
    connect(m_worker, &YubiKeyWorker::errorOccurred, this, &AppController::showMessage);

    m_workerThread.setObjectName(QStringLiteral("yubikey-worker"));
    m_workerThread.start();

    m_timer.setInterval(500);
    connect(&m_timer, &QTimer::timeout, this, &AppController::tick);
    m_timer.start();

    retryConnect();
}

AppController::~AppController()
{
    m_workerThread.quit();
    m_workerThread.wait(2000);
}

QString AppController::deviceName() const
{
    return m_versionMajor > 0 ? tr("YubiKey %1").arg(m_versionMajor) : QString();
}

QString AppController::firmwareVersion() const
{
    if (m_versionMajor == 0 && m_versionMinor == 0 && m_versionPatch == 0) {
        return {};
    }
    return QStringLiteral("%1.%2.%3").arg(m_versionMajor).arg(m_versionMinor).arg(m_versionPatch);
}

QString AppController::appName() const
{
    return QStringLiteral(APP_DISPLAY_NAME);
}

QString AppController::appVersion() const
{
    return QStringLiteral(APP_VERSION_STRING);
}

QVariantMap AppController::aboutData() const
{
    QVariantMap author;
    author.insert(QStringLiteral("name"), QStringLiteral("Goshitsarch"));

    QVariantMap license;
    license.insert(QStringLiteral("name"), QStringLiteral("GPL-3.0-or-later"));
    license.insert(QStringLiteral("spdx"), QStringLiteral("GPL-3.0-or-later"));

    QVariantMap about;
    about.insert(QStringLiteral("displayName"), appName());
    about.insert(QStringLiteral("productName"), QStringLiteral("gosh-authenticator"));
    about.insert(QStringLiteral("componentName"), QStringLiteral("gosh-authenticator"));
    about.insert(QStringLiteral("version"), appVersion());
    about.insert(QStringLiteral("shortDescription"),
                 tr("Manage OATH (TOTP/HOTP) credentials on YubiKey devices"));
    about.insert(QStringLiteral("homepage"),
                 QStringLiteral("https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux"));
    about.insert(QStringLiteral("bugAddress"),
                 QStringLiteral("https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux/issues"));
    about.insert(QStringLiteral("copyrightStatement"), QStringLiteral("© Goshitsarch"));
    about.insert(QStringLiteral("otherText"),
                 tr("This independent application is not affiliated with or endorsed by Yubico. "
                    "Yubico and YubiKey are trademarks of Yubico AB."));
    about.insert(QStringLiteral("desktopFileName"), QStringLiteral("com.goshapps.YubicoAuthenticator"));
    about.insert(QStringLiteral("authors"), QVariantList{author});
    about.insert(QStringLiteral("licenses"), QVariantList{license});
    return about;
}

QVariantList AppController::serviceChoices() const
{
    return serviceicons::allServicesVariant();
}

double AppController::nowSeconds() const
{
    return static_cast<double>(QDateTime::currentMSecsSinceEpoch()) / 1000.0;
}

void AppController::setConnectionState(int state)
{
    if (m_connectionState == state) {
        return;
    }
    m_connectionState = state;
    if (state == Disconnected) {
        m_reconnectCountdown = 6; // retry roughly every 3 seconds
    }
    Q_EMIT connectionStateChanged();
}

void AppController::tick()
{
    Q_EMIT ticked();

    if (m_connectionState == Connected) {
        const qint64 slot = QDateTime::currentSecsSinceEpoch() / 30;
        if (m_lastPeriodSlot == 0) {
            m_lastPeriodSlot = slot;
        } else if (slot != m_lastPeriodSlot) {
            m_lastPeriodSlot = slot;
            refresh();
        }
    } else if (m_connectionState == Disconnected) {
        if (--m_reconnectCountdown <= 0) {
            m_reconnectCountdown = 6;
            QMetaObject::invokeMethod(m_worker, "connectToKey", Qt::QueuedConnection);
        }
    }
}

void AppController::retryConnect()
{
    if (m_connectionState != Connected) {
        setConnectionState(Connecting);
    }
    QMetaObject::invokeMethod(m_worker, "connectToKey", Qt::QueuedConnection);
}

void AppController::refresh()
{
    QMetaObject::invokeMethod(m_worker, "refresh", Qt::QueuedConnection);
}

void AppController::authenticate(const QString &password)
{
    QMetaObject::invokeMethod(m_worker, "authenticate", Qt::QueuedConnection,
                              Q_ARG(QString, password));
}

void AppController::calculate(const QString &credId)
{
    QMetaObject::invokeMethod(m_worker, "calculateCredential", Qt::QueuedConnection,
                              Q_ARG(QByteArray, QByteArray::fromBase64(credId.toLatin1())));
}

void AppController::copyCode(const QString &credId)
{
    const QByteArray id = QByteArray::fromBase64(credId.toLatin1());
    const QString code = m_model->codeFor(id);
    if (code.isEmpty()) {
        calculate(credId);
        return;
    }
    const int timeout = m_settings->clipboardTimeoutSeconds();
    // Codes are displayed with a space for readability; copy the raw digits.
    QString raw = code;
    raw.remove(QLatin1Char(' '));
    m_clipboard->copy(raw, timeout);
    Q_EMIT credentialCopied(credId, timeout);
    Q_EMIT showMessage(tr("Code copied (clears in %1s)").arg(timeout));
}

void AppController::addCredential(const QString &issuer,
                                  const QString &account,
                                  const QString &secret,
                                  bool isHotp,
                                  int algorithmIndex,
                                  int digits,
                                  bool requireTouch)
{
    QMetaObject::invokeMethod(m_worker, "addCredential", Qt::QueuedConnection,
                              Q_ARG(QString, issuer), Q_ARG(QString, account),
                              Q_ARG(QString, secret), Q_ARG(bool, isHotp),
                              Q_ARG(int, algorithmIndex), Q_ARG(int, digits),
                              Q_ARG(bool, requireTouch));
}

void AppController::deleteCredential(const QString &credId)
{
    QMetaObject::invokeMethod(m_worker, "deleteCredential", Qt::QueuedConnection,
                              Q_ARG(QByteArray, QByteArray::fromBase64(credId.toLatin1())));
}

void AppController::setPassword(const QString &password)
{
    QMetaObject::invokeMethod(m_worker, "setPassword", Qt::QueuedConnection,
                              Q_ARG(QString, password));
}

bool AppController::isSecretValid(const QString &secret) const
{
    const auto decoded = oath::decodeBase32Secret(secret);
    return decoded.has_value() && !decoded->isEmpty();
}

QVariantMap AppController::parseOtpauth(const QString &uri) const
{
    return otpauth::parseUri(uri);
}

QVariantMap AppController::importQrFile(const QUrl &url) const
{
    return otpauth::scanQrUrl(url);
}

double AppController::totpProgress(int period) const
{
    if (period <= 0) {
        period = 30;
    }
    const double now = static_cast<double>(QDateTime::currentMSecsSinceEpoch()) / 1000.0;
    const double elapsed = std::fmod(now, static_cast<double>(period));
    return (static_cast<double>(period) - elapsed) / static_cast<double>(period);
}
