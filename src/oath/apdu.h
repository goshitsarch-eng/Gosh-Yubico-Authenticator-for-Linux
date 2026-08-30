// SPDX-License-Identifier: GPL-3.0-or-later
//
// APDU (Application Protocol Data Unit) building and parsing for YubiKey
// OATH communication.
//
// Based on the YKOATH protocol:
// https://developers.yubico.com/OATH/YKOATH_Protocol.html

#pragma once

#include <QByteArray>
#include <QList>
#include <QString>

#include <optional>

namespace oath {

/// OATH applet AID (Application Identifier)
inline const QByteArray OATH_AID = QByteArray::fromHex("a0000005272101");

enum class Instruction : quint8 {
    Put = 0x01,
    Delete = 0x02,
    SetCode = 0x03,
    Reset = 0x04,
    List = 0xA1,
    Calculate = 0xA2,
    Validate = 0xA3,
    CalculateAll = 0xA4,
    SendRemaining = 0xA5,
};

/// TLV (Tag-Length-Value) tags used in the OATH protocol
enum class Tag : quint8 {
    Name = 0x71,
    NameList = 0x72,
    Key = 0x73,
    Challenge = 0x74,
    Response = 0x75,
    TruncatedResponse = 0x76,
    Hotp = 0x77,
    Property = 0x78,
    Version = 0x79,
    InitialMovingFactor = 0x7A,
    Algorithm = 0x7B,
    Touch = 0x7C,
};

enum class Algorithm : quint8 {
    Sha1 = 0x01,
    Sha256 = 0x02,
    Sha512 = 0x03,
};

enum class OathType : quint8 {
    Hotp = 0x10,
    Totp = 0x20,
};

std::optional<Algorithm> algorithmFromByte(quint8 byte);
std::optional<OathType> oathTypeFromByte(quint8 byte);
int hmacKeySize(Algorithm algorithm);

/// Status words
namespace sw {
inline constexpr quint16 Success = 0x9000;
inline constexpr quint8 MoreData = 0x61;
inline constexpr quint16 AuthRequired = 0x6982;
inline constexpr quint16 WrongSyntax = 0x6A80;
inline constexpr quint16 NoSuchObject = 0x6984;
inline constexpr quint16 AuthNotInitialized = 0x6986;
inline constexpr quint16 NoSpace = 0x6A84;
} // namespace sw

QByteArray buildSelectApdu();
QByteArray buildListApdu();
QByteArray buildCalculateApdu(const QByteArray &name, const QByteArray &challenge);
QByteArray buildCalculateAllApdu(const QByteArray &challenge);
QByteArray buildPutApdu(const QByteArray &name,
                        const QByteArray &secret,
                        OathType type,
                        Algorithm algorithm,
                        int digits,
                        bool requireTouch,
                        std::optional<quint32> initialCounter);
QByteArray buildDeleteApdu(const QByteArray &name);
QByteArray buildValidateApdu(const QByteArray &response, const QByteArray &challenge);
QByteArray buildSendRemainingApdu();
QByteArray buildSetCodeApdu(const QByteArray &key,
                            const QByteArray &challenge,
                            const QByteArray &response);

/// A single decoded TLV entry
struct Tlv {
    quint8 tag = 0;
    QByteArray value;
};

/// Parse a buffer of TLV entries. Trailing malformed data is reported via
/// *malformed (when non-null) and not returned as an entry.
QList<Tlv> parseTlv(const QByteArray &data, bool *malformed = nullptr);

/// Parse a truncated response TLV value into (code, digits).
std::optional<QPair<quint32, int>> parseTruncatedResponse(const QByteArray &value);

/// Format an OTP code with a space in the middle for readability.
QString formatCode(quint32 code, int digits);

/// Split a raw credential name into issuer (may be empty) and account.
QPair<QString, QString> parseCredentialName(const QByteArray &name);

/// Build the raw credential name from issuer and account.
QByteArray buildCredentialName(const QString &issuer, const QString &account);

/// TOTP challenge (big-endian time counter) for the given period.
QByteArray totpChallenge(qint64 unixTime, int period);

/// Decode a base32 secret, ignoring whitespace and dashes and fixing padding.
/// Returns std::nullopt when the input is not valid base32.
std::optional<QByteArray> decodeBase32Secret(const QString &secret);

} // namespace oath
