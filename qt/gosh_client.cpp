#include "gosh_client.h"

#include <QByteArray>
#include <QMetaObject>
#include <QString>

#include "credential_types.h"

extern "C" {
#include "gosh_ffi.h"
}

namespace {
constexpr uint32_t EVENT_READY = 0;
constexpr uint32_t EVENT_CONNECTED = 1;
constexpr uint32_t EVENT_DISCONNECTED = 2;
constexpr uint32_t EVENT_AUTH_REQUIRED = 3;
constexpr uint32_t EVENT_AUTH_SUCCESS = 4;
constexpr uint32_t EVENT_AUTH_FAILED = 5;
constexpr uint32_t EVENT_CREDENTIALS_UPDATED = 6;
constexpr uint32_t EVENT_CREDENTIAL_CALCULATED = 7;
constexpr uint32_t EVENT_TOUCH_REQUIRED = 8;
constexpr uint32_t EVENT_CREDENTIAL_ADDED = 9;
constexpr uint32_t EVENT_CREDENTIAL_DELETED = 10;
constexpr uint32_t EVENT_ERROR = 11;
}  // namespace

GoshClientWrapper::GoshClientWrapper(QObject* parent)
    : QObject(parent) {
    gosh_init_logging();
    handle = gosh_client_new();
    gosh_client_set_callback(handle, &GoshClientWrapper::eventCallback, this);
}

GoshClientWrapper::~GoshClientWrapper() {
    if (handle) {
        gosh_client_set_callback(handle, nullptr, nullptr);
        gosh_client_free(handle);
        handle = nullptr;
    }
}

void GoshClientWrapper::connectDevice() {
    if (handle) {
        gosh_client_connect(handle);
    }
}

void GoshClientWrapper::refresh() {
    if (handle) {
        gosh_client_refresh(handle);
    }
}

void GoshClientWrapper::authenticate(const QString& password) {
    if (!handle) {
        return;
    }
    const QByteArray utf8 = password.toUtf8();
    gosh_client_authenticate(handle, utf8.constData());
}

void GoshClientWrapper::calculate(const QByteArray& id) {
    if (!handle || id.isEmpty()) {
        return;
    }
    gosh_client_calculate(handle,
                          reinterpret_cast<const uint8_t*>(id.constData()),
                          static_cast<size_t>(id.size()));
}

bool GoshClientWrapper::addCredential(const QString& issuer,
                                      const QString& account,
                                      const QString& secret,
                                      uint8_t oathType,
                                      uint8_t algorithm,
                                      uint8_t digits,
                                      bool requireTouch,
                                      bool hasInitialCounter,
                                      uint32_t initialCounter) {
    if (!handle) {
        return false;
    }
    const QByteArray issuerUtf8 = issuer.toUtf8();
    const QByteArray accountUtf8 = account.toUtf8();
    const QByteArray secretUtf8 = secret.toUtf8();

    return gosh_client_add_credential(
        handle,
        issuerUtf8.isEmpty() ? nullptr : issuerUtf8.constData(),
        accountUtf8.constData(),
        secretUtf8.constData(),
        oathType,
        algorithm,
        digits,
        requireTouch ? 1 : 0,
        initialCounter,
        hasInitialCounter ? 1 : 0);
}

void GoshClientWrapper::deleteCredential(const QByteArray& id) {
    if (!handle || id.isEmpty()) {
        return;
    }
    gosh_client_delete_credential(handle,
                                  reinterpret_cast<const uint8_t*>(id.constData()),
                                  static_cast<size_t>(id.size()));
}

QString GoshClientWrapper::takeLastError() {
    if (!handle) {
        return QString();
    }
    char* err = gosh_client_last_error_take(handle);
    if (!err) {
        return QString();
    }
    QString message = QString::fromUtf8(err);
    gosh_string_free(err);
    return message;
}

void GoshClientWrapper::eventCallback(const GoshEvent* event, void* userData) {
    auto* self = static_cast<GoshClientWrapper*>(userData);
    if (!self || !event) {
        if (event) {
            gosh_event_free(const_cast<GoshEvent*>(event));
        }
        return;
    }

    // Copy event data to Qt types before freeing.
    const int eventType = static_cast<int>(event->event_type);
    QString message;
    QString code;
    QByteArray id;
    QVector<Credential> credentials;
    QString version;
    QByteArray deviceId;
    uint8_t digits = event->digits;

    if (event->message) {
        message = QString::fromUtf8(event->message);
    }
    if (event->code) {
        code = QString::fromUtf8(event->code);
    }
    if (event->id_ptr && event->id_len > 0) {
        id = QByteArray(reinterpret_cast<const char*>(event->id_ptr),
                        static_cast<int>(event->id_len));
    }

    if (event->event_type == EVENT_CONNECTED) {
        version = QString("%1.%2.%3")
                      .arg(event->version.major)
                      .arg(event->version.minor)
                      .arg(event->version.patch);
        deviceId = QByteArray(reinterpret_cast<const char*>(event->device_id), 8);
    }

    if (event->credentials && event->credentials_len > 0) {
        credentials.reserve(static_cast<int>(event->credentials_len));
        for (size_t i = 0; i < event->credentials_len; ++i) {
            const GoshCredential& c = event->credentials[i];
            Credential cred;
            if (c.id_ptr && c.id_len > 0) {
                cred.id = QByteArray(reinterpret_cast<const char*>(c.id_ptr),
                                     static_cast<int>(c.id_len));
            }
            if (c.issuer) {
                cred.issuer = QString::fromUtf8(c.issuer);
            }
            if (c.account) {
                cred.account = QString::fromUtf8(c.account);
            }
            if (c.code) {
                cred.code = QString::fromUtf8(c.code);
            }
            cred.oathType = c.oath_type;
            cred.algorithm = c.algorithm;
            cred.digits = c.digits;
            cred.touchRequired = c.touch_required != 0;
            cred.period = c.period;
            credentials.push_back(cred);
        }
    }

    gosh_event_free(const_cast<GoshEvent*>(event));

    QMetaObject::invokeMethod(
        self,
        [self,
         eventType,
         message,
         credentials,
         id,
         code,
         digits,
         version,
         deviceId]() {
            self->dispatchEvent(eventType, message, credentials, id, code, digits, version, deviceId);
        },
        Qt::QueuedConnection);
}

void GoshClientWrapper::dispatchEvent(int eventType,
                                      const QString& message,
                                      const QVector<Credential>& credentials,
                                      const QByteArray& id,
                                      const QString& code,
                                      uint8_t digits,
                                      const QString& version,
                                      const QByteArray& deviceId) {
    switch (eventType) {
        case EVENT_READY:
            emit ready();
            break;
        case EVENT_CONNECTED:
            emit connected(version, deviceId);
            break;
        case EVENT_DISCONNECTED:
            emit disconnected();
            break;
        case EVENT_AUTH_REQUIRED:
            emit authenticationRequired();
            break;
        case EVENT_AUTH_SUCCESS:
            emit authenticationSuccessful();
            break;
        case EVENT_AUTH_FAILED:
            emit authenticationFailed(message);
            break;
        case EVENT_CREDENTIALS_UPDATED:
            emit credentialsUpdated(credentials);
            break;
        case EVENT_CREDENTIAL_CALCULATED:
            emit credentialCalculated(id, code, digits);
            break;
        case EVENT_TOUCH_REQUIRED:
            emit touchRequired(id);
            break;
        case EVENT_CREDENTIAL_ADDED:
            if (!credentials.isEmpty()) {
                emit credentialAdded(credentials.first());
            }
            break;
        case EVENT_CREDENTIAL_DELETED:
            emit credentialDeleted(id);
            break;
        case EVENT_ERROR:
            emit error(message);
            break;
        default:
            break;
    }
}
