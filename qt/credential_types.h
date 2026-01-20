#pragma once

#include <cstdint>

#include <QByteArray>
#include <QString>
#include <QVector>

struct Credential {
    QByteArray id;
    QString issuer;
    QString account;
    QString code;
    uint8_t oathType = 0x20;
    uint8_t algorithm = 0x01;
    uint8_t digits = 6;
    bool touchRequired = false;
    uint32_t period = 30;

    QString displayName() const {
        if (!issuer.isEmpty()) {
            return issuer + ": " + account;
        }
        return account;
    }

    bool isTotp() const { return oathType == 0x20; }
    bool isHotp() const { return oathType == 0x10; }
};

Q_DECLARE_METATYPE(Credential)
Q_DECLARE_METATYPE(QVector<Credential>)
