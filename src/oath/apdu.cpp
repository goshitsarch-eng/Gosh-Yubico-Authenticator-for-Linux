// SPDX-License-Identifier: GPL-3.0-or-later

#include "apdu.h"

namespace oath {

std::optional<Algorithm> algorithmFromByte(quint8 byte)
{
    switch (byte & 0x0F) {
    case 0x01:
        return Algorithm::Sha1;
    case 0x02:
        return Algorithm::Sha256;
    case 0x03:
        return Algorithm::Sha512;
    default:
        return std::nullopt;
    }
}

std::optional<OathType> oathTypeFromByte(quint8 byte)
{
    switch (byte & 0xF0) {
    case 0x10:
        return OathType::Hotp;
    case 0x20:
        return OathType::Totp;
    default:
        return std::nullopt;
    }
}

int hmacKeySize(Algorithm algorithm)
{
    switch (algorithm) {
    case Algorithm::Sha1:
        return 20;
    case Algorithm::Sha256:
        return 32;
    case Algorithm::Sha512:
        return 64;
    }
    return 20;
}

static void appendTlv(QByteArray &data, Tag tag, const QByteArray &value)
{
    data.append(static_cast<char>(tag));
    data.append(static_cast<char>(value.size()));
    data.append(value);
}

static QByteArray buildApdu(Instruction ins, quint8 p1, quint8 p2, const QByteArray &data)
{
    QByteArray apdu;
    apdu.append('\x00'); // CLA
    apdu.append(static_cast<char>(ins));
    apdu.append(static_cast<char>(p1));
    apdu.append(static_cast<char>(p2));
    if (!data.isEmpty()) {
        apdu.append(static_cast<char>(data.size()));
        apdu.append(data);
    }
    return apdu;
}

QByteArray buildSelectApdu()
{
    QByteArray apdu;
    apdu.append('\x00'); // CLA
    apdu.append('\xA4'); // INS: SELECT
    apdu.append('\x04'); // P1: select by DF name
    apdu.append('\x00'); // P2
    apdu.append(static_cast<char>(OATH_AID.size()));
    apdu.append(OATH_AID);
    return apdu;
}

QByteArray buildListApdu()
{
    return buildApdu(Instruction::List, 0x00, 0x00, {});
}

QByteArray buildCalculateApdu(const QByteArray &name, const QByteArray &challenge)
{
    QByteArray data;
    appendTlv(data, Tag::Name, name);
    appendTlv(data, Tag::Challenge, challenge);
    return buildApdu(Instruction::Calculate, 0x00, 0x01 /* truncate */, data);
}

QByteArray buildCalculateAllApdu(const QByteArray &challenge)
{
    QByteArray data;
    appendTlv(data, Tag::Challenge, challenge);
    return buildApdu(Instruction::CalculateAll, 0x00, 0x01 /* truncate */, data);
}

QByteArray buildPutApdu(const QByteArray &name,
                        const QByteArray &secret,
                        OathType type,
                        Algorithm algorithm,
                        int digits,
                        bool requireTouch,
                        std::optional<quint32> initialCounter)
{
    QByteArray data;
    appendTlv(data, Tag::Name, name);

    // Key TLV: type/algorithm byte + digits byte + secret
    QByteArray key;
    key.append(static_cast<char>(static_cast<quint8>(type) | static_cast<quint8>(algorithm)));
    key.append(static_cast<char>(digits));
    key.append(secret);
    appendTlv(data, Tag::Key, key);

    if (requireTouch) {
        data.append(static_cast<char>(Tag::Property));
        data.append('\x01');
        data.append('\x02'); // REQUIRE_TOUCH
    }

    if (initialCounter) {
        QByteArray counter;
        counter.append(static_cast<char>((*initialCounter >> 24) & 0xFF));
        counter.append(static_cast<char>((*initialCounter >> 16) & 0xFF));
        counter.append(static_cast<char>((*initialCounter >> 8) & 0xFF));
        counter.append(static_cast<char>(*initialCounter & 0xFF));
        appendTlv(data, Tag::InitialMovingFactor, counter);
    }

    return buildApdu(Instruction::Put, 0x00, 0x00, data);
}

QByteArray buildDeleteApdu(const QByteArray &name)
{
    QByteArray data;
    appendTlv(data, Tag::Name, name);
    return buildApdu(Instruction::Delete, 0x00, 0x00, data);
}

QByteArray buildValidateApdu(const QByteArray &response, const QByteArray &challenge)
{
    QByteArray data;
    appendTlv(data, Tag::Response, response);
    appendTlv(data, Tag::Challenge, challenge);
    return buildApdu(Instruction::Validate, 0x00, 0x00, data);
}

QByteArray buildSendRemainingApdu()
{
    return buildApdu(Instruction::SendRemaining, 0x00, 0x00, {});
}

QByteArray buildSetCodeApdu(const QByteArray &key,
                            const QByteArray &challenge,
                            const QByteArray &response)
{
    QByteArray data;
    data.append(static_cast<char>(Tag::Key));
    if (key.isEmpty()) {
        data.append('\x00'); // zero length removes password protection
    } else {
        data.append(static_cast<char>(1 + key.size()));
        data.append(static_cast<char>(Algorithm::Sha1)); // auth always uses SHA-1
        data.append(key);
    }
    if (!challenge.isEmpty()) {
        appendTlv(data, Tag::Challenge, challenge);
    }
    if (!response.isEmpty()) {
        appendTlv(data, Tag::Response, response);
    }
    return buildApdu(Instruction::SetCode, 0x00, 0x00, data);
}

QList<Tlv> parseTlv(const QByteArray &data, bool *malformed)
{
    QList<Tlv> entries;
    int pos = 0;
    while (pos + 2 <= data.size()) {
        const quint8 tag = static_cast<quint8>(data.at(pos));
        const int len = static_cast<quint8>(data.at(pos + 1));
        if (pos + 2 + len > data.size()) {
            break;
        }
        entries.append(Tlv{tag, data.mid(pos + 2, len)});
        pos += 2 + len;
    }
    if (malformed) {
        *malformed = pos != data.size();
    }
    return entries;
}

std::optional<QPair<quint32, int>> parseTruncatedResponse(const QByteArray &value)
{
    if (value.size() < 5) {
        return std::nullopt;
    }
    const int digits = static_cast<quint8>(value.at(0));
    if (digits < 6 || digits > 8) {
        return std::nullopt;
    }
    const quint32 code = (static_cast<quint32>(static_cast<quint8>(value.at(1))) << 24)
        | (static_cast<quint32>(static_cast<quint8>(value.at(2))) << 16)
        | (static_cast<quint32>(static_cast<quint8>(value.at(3))) << 8)
        | static_cast<quint32>(static_cast<quint8>(value.at(4)));

    quint32 divisor = 1;
    for (int i = 0; i < digits; ++i) {
        divisor *= 10;
    }
    return QPair<quint32, int>{code % divisor, digits};
}

QString formatCode(quint32 code, int digits)
{
    QString text = QStringLiteral("%1").arg(code, digits, 10, QLatin1Char('0'));
    if (digits == 6 || digits == 7) {
        return text.left(3) + QLatin1Char(' ') + text.mid(3);
    }
    if (digits == 8) {
        return text.left(4) + QLatin1Char(' ') + text.mid(4);
    }
    return text;
}

QPair<QString, QString> parseCredentialName(const QByteArray &name)
{
    const QString text = QString::fromUtf8(name);
    const int colon = text.indexOf(QLatin1Char(':'));
    if (colon >= 0) {
        return {text.left(colon), text.mid(colon + 1)};
    }
    return {QString(), text};
}

QByteArray buildCredentialName(const QString &issuer, const QString &account)
{
    if (!issuer.isEmpty()) {
        return (issuer + QLatin1Char(':') + account).toUtf8();
    }
    return account.toUtf8();
}

QByteArray totpChallenge(qint64 unixTime, int period)
{
    if (period <= 0) {
        period = 30;
    }
    const quint64 counter = static_cast<quint64>(unixTime) / static_cast<quint64>(period);
    QByteArray challenge;
    for (int shift = 56; shift >= 0; shift -= 8) {
        challenge.append(static_cast<char>((counter >> shift) & 0xFF));
    }
    return challenge;
}

std::optional<QByteArray> decodeBase32Secret(const QString &secret)
{
    QString cleaned;
    cleaned.reserve(secret.size());
    for (const QChar &c : secret) {
        if (c.isSpace() || c == QLatin1Char('-')) {
            continue;
        }
        cleaned.append(c.toUpper());
    }
    while (cleaned.endsWith(QLatin1Char('='))) {
        cleaned.chop(1);
    }
    if (cleaned.isEmpty()) {
        return std::nullopt;
    }

    QByteArray out;
    out.reserve(cleaned.size() * 5 / 8 + 1);
    quint32 buffer = 0;
    int bits = 0;
    for (const QChar &c : cleaned) {
        int value;
        const char16_t u = c.unicode();
        if (u >= u'A' && u <= u'Z') {
            value = u - u'A';
        } else if (u >= u'2' && u <= u'7') {
            value = u - u'2' + 26;
        } else {
            return std::nullopt;
        }
        buffer = (buffer << 5) | static_cast<quint32>(value);
        bits += 5;
        if (bits >= 8) {
            bits -= 8;
            out.append(static_cast<char>((buffer >> bits) & 0xFF));
        }
    }
    // Leftover bits must be zero padding for valid base32.
    if (bits > 0 && (buffer & ((1u << bits) - 1)) != 0) {
        return std::nullopt;
    }
    return out;
}

} // namespace oath
