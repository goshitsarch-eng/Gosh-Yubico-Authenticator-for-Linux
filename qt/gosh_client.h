#pragma once

#include <cstdint>

#include <QObject>
#include <QVector>

#include "credential_types.h"

struct GoshClient;
struct GoshEvent;

class GoshClientWrapper : public QObject {
    Q_OBJECT

public:
    explicit GoshClientWrapper(QObject* parent = nullptr);
    ~GoshClientWrapper() override;

    void connectDevice();
    void refresh();
    void authenticate(const QString& password);
    void calculate(const QByteArray& id);
    bool addCredential(const QString& issuer,
                       const QString& account,
                       const QString& secret,
                       uint8_t oathType,
                       uint8_t algorithm,
                       uint8_t digits,
                       bool requireTouch,
                       bool hasInitialCounter,
                       uint32_t initialCounter);
    void deleteCredential(const QByteArray& id);
    QString takeLastError();

signals:
    void ready();
    void connected(const QString& version, const QByteArray& deviceId);
    void disconnected();
    void authenticationRequired();
    void authenticationSuccessful();
    void authenticationFailed(const QString& message);
    void credentialsUpdated(const QVector<Credential>& credentials);
    void credentialCalculated(const QByteArray& id, const QString& code, uint8_t digits);
    void touchRequired(const QByteArray& id);
    void credentialAdded(const Credential& credential);
    void credentialDeleted(const QByteArray& id);
    void error(const QString& message);

private:
    static void eventCallback(const GoshEvent* event, void* userData);
    void handleEvent(const GoshEvent* event);
    void dispatchEvent(int eventType,
                       const QString& message,
                       const QVector<Credential>& credentials,
                       const QByteArray& id,
                       const QString& code,
                       uint8_t digits,
                       const QString& version,
                       const QByteArray& deviceId);

    GoshClient* handle = nullptr;
};
