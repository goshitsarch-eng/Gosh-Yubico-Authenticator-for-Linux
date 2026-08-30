// SPDX-License-Identifier: GPL-3.0-or-later
//
// Unit tests for the OATH protocol core and otpauth parsing.

#include "oath/apdu.h"
#include "oath/oathsession.h"
#include "otpauth.h"
#include "serviceicons.h"

#include <QtTest>

using namespace oath;

class TestOath : public QObject {
    Q_OBJECT

private Q_SLOTS:
    void selectApdu()
    {
        const QByteArray apdu = buildSelectApdu();
        QCOMPARE(static_cast<quint8>(apdu.at(0)), quint8(0x00)); // CLA
        QCOMPARE(static_cast<quint8>(apdu.at(1)), quint8(0xA4)); // INS
        QCOMPARE(static_cast<quint8>(apdu.at(4)), quint8(7)); // AID length
        QCOMPARE(apdu.mid(5), OATH_AID);
    }

    void listApdu()
    {
        QCOMPARE(buildListApdu(), QByteArray::fromHex("00a10000"));
    }

    void tlvParser()
    {
        // Name TLV "test" followed by a version TLV
        const QByteArray data = QByteArray::fromHex("710474657374") + QByteArray::fromHex("7903050300");
        const QList<Tlv> tlvs = parseTlv(data);
        QCOMPARE(tlvs.size(), 2);
        QCOMPARE(tlvs.at(0).tag, quint8(0x71));
        QCOMPARE(tlvs.at(0).value, QByteArray("test"));
        QCOMPARE(tlvs.at(1).tag, quint8(0x79));
        QCOMPARE(tlvs.at(1).value, QByteArray::fromHex("050300"));
    }

    void truncatedResponse()
    {
        // 6 digits, code = 123456 = 0x1E240
        const auto result = parseTruncatedResponse(QByteArray::fromHex("060001e240"));
        QVERIFY(result.has_value());
        QCOMPARE(result->first, quint32(123456));
        QCOMPARE(result->second, 6);

        QVERIFY(!parseTruncatedResponse(QByteArray::fromHex("0600")).has_value());
        QVERIFY(!parseTruncatedResponse(QByteArray::fromHex("050001e240")).has_value());
    }

    void codeFormatting()
    {
        QCOMPARE(formatCode(123456, 6), QStringLiteral("123 456"));
        QCOMPARE(formatCode(12345678, 8), QStringLiteral("1234 5678"));
        QCOMPARE(formatCode(1, 6), QStringLiteral("000 001"));
    }

    void credentialNames()
    {
        auto [issuer, account] = parseCredentialName("Google:user@example.com");
        QCOMPARE(issuer, QStringLiteral("Google"));
        QCOMPARE(account, QStringLiteral("user@example.com"));

        auto [noIssuer, bareAccount] = parseCredentialName("user@example.com");
        QVERIFY(noIssuer.isEmpty());
        QCOMPARE(bareAccount, QStringLiteral("user@example.com"));

        QCOMPARE(buildCredentialName(QStringLiteral("Google"), QStringLiteral("user@example.com")),
                 QByteArray("Google:user@example.com"));
        QCOMPARE(buildCredentialName(QString(), QStringLiteral("user@example.com")),
                 QByteArray("user@example.com"));
    }

    void base32Decoding()
    {
        // RFC 4226 test secret "12345678901234567890"
        const QByteArray expected("12345678901234567890");
        QCOMPARE(decodeBase32Secret(QStringLiteral("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")).value(), expected);
        QCOMPARE(decodeBase32Secret(QStringLiteral("GEZD GNBV GY3T QOJQ GEZD GNBV GY3T QOJQ")).value(),
                 expected);
        QCOMPARE(decodeBase32Secret(QStringLiteral("gezdgnbvgy3tqojqgezdgnbvgy3tqojq")).value(), expected);
        QCOMPARE(decodeBase32Secret(QStringLiteral("JBSWY3DPEHPK3PXP")).value(),
                 QByteArray("Hello!\xDE\xAD\xBE\xEF"));
        QVERIFY(!decodeBase32Secret(QStringLiteral("not base32 !!")).has_value());
        QVERIFY(!decodeBase32Secret(QString()).has_value());
    }

    void totpChallengeBytes()
    {
        // 59s with period 30 -> counter 1 (RFC 6238 test vector timing)
        QCOMPARE(totpChallenge(59, 30), QByteArray::fromHex("0000000000000001"));
        QCOMPARE(totpChallenge(0, 0), QByteArray::fromHex("0000000000000000"));
    }

    void selectResponseRequiresDeviceId()
    {
        // Version TLV only, success status
        const QByteArray response = QByteArray::fromHex("7903050701") + QByteArray::fromHex("9000");
        try {
            OathSession::parseSelectResponse(response);
            QFAIL("Expected an exception for a missing device ID");
        } catch (const OathException &e) {
            QCOMPARE(e.code(), ErrorCode::InvalidResponse);
            QVERIFY(e.message().contains(QStringLiteral("missing the device ID")));
        }
    }

    void selectResponseRejectsInvalidDeviceIdLength()
    {
        const QByteArray response = QByteArray::fromHex("710701020304050607") + QByteArray::fromHex("9000");
        try {
            OathSession::parseSelectResponse(response);
            QFAIL("Expected an exception for an invalid device ID length");
        } catch (const OathException &e) {
            QVERIFY(e.message().contains(QStringLiteral("invalid device ID length")));
        }
    }

    void selectResponseRejectsMalformedTlv()
    {
        const QByteArray response = QByteArray::fromHex("7108010203") + QByteArray::fromHex("9000");
        try {
            OathSession::parseSelectResponse(response);
            QFAIL("Expected an exception for malformed TLV data");
        } catch (const OathException &e) {
            QVERIFY(e.message().contains(QStringLiteral("malformed TLV")));
        }
    }

    void selectResponseAcceptsExactDeviceId()
    {
        const QByteArray response =
            QByteArray::fromHex("7903050701") + QByteArray::fromHex("71080102030405060708")
            + QByteArray::fromHex("9000");
        const SelectInfo info = OathSession::parseSelectResponse(response);
        QCOMPARE(info.version.major, 5);
        QCOMPARE(info.version.minor, 7);
        QCOMPARE(info.version.patch, 1);
        QCOMPARE(info.deviceId, QByteArray::fromHex("0102030405060708"));
        QVERIFY(info.challenge.isEmpty());
        QVERIFY(!info.challengeAlgorithm.has_value());
    }

    void putApduLayout()
    {
        const QByteArray apdu =
            buildPutApdu("a:b", QByteArray(20, 'k'), OathType::Totp, Algorithm::Sha1, 6, true, std::nullopt);
        QCOMPARE(static_cast<quint8>(apdu.at(1)), quint8(0x01)); // INS: PUT
        // Data: name TLV (2+3) + key TLV (2+22) + property (3) = 32
        QCOMPARE(static_cast<quint8>(apdu.at(4)), quint8(32));
        QCOMPARE(apdu.mid(5, 5), QByteArray("\x71\x03") + QByteArray("a:b"));
        QCOMPARE(static_cast<quint8>(apdu.at(10)), quint8(0x73)); // key tag
        QCOMPARE(static_cast<quint8>(apdu.at(12)), quint8(0x21)); // TOTP | SHA1
        QCOMPARE(static_cast<quint8>(apdu.at(13)), quint8(6)); // digits
        QCOMPARE(apdu.right(3), QByteArray::fromHex("780102")); // touch property
    }

    void parseTotpUri()
    {
        const QVariantMap result = otpauth::parseUri(QStringLiteral(
            "otpauth://totp/Example:alice@google.com?secret=JBSWY3DPEHPK3PXP&issuer=Example"
            "&algorithm=SHA256&digits=8&period=30"));
        QVERIFY(result.value(QStringLiteral("ok")).toBool());
        QCOMPARE(result.value(QStringLiteral("isHotp")).toBool(), false);
        QCOMPARE(result.value(QStringLiteral("issuer")).toString(), QStringLiteral("Example"));
        QCOMPARE(result.value(QStringLiteral("account")).toString(), QStringLiteral("alice@google.com"));
        QCOMPARE(result.value(QStringLiteral("secret")).toString(), QStringLiteral("JBSWY3DPEHPK3PXP"));
        QCOMPARE(result.value(QStringLiteral("algorithmIndex")).toInt(), 1);
        QCOMPARE(result.value(QStringLiteral("digits")).toInt(), 8);
        QCOMPARE(result.value(QStringLiteral("period")).toInt(), 30);
    }

    void parseHotpUri()
    {
        const QVariantMap result =
            otpauth::parseUri(QStringLiteral("otpauth://hotp/alice?secret=MFRGGZDF&counter=5"));
        QVERIFY(result.value(QStringLiteral("ok")).toBool());
        QCOMPARE(result.value(QStringLiteral("isHotp")).toBool(), true);
        QCOMPARE(result.value(QStringLiteral("account")).toString(), QStringLiteral("alice"));
        QCOMPARE(result.value(QStringLiteral("counter")).toInt(), 5);
    }

    void rejectNonOtpauthUri()
    {
        const QVariantMap result = otpauth::parseUri(QStringLiteral("https://example.com"));
        QVERIFY(!result.value(QStringLiteral("ok")).toBool());
        QVERIFY(result.value(QStringLiteral("error")).toString().contains(QStringLiteral("otpauth")));

        QVERIFY(!otpauth::parseUri(QStringLiteral("otpauth://totp/alice")).value(QStringLiteral("ok")).toBool());
        QVERIFY(!otpauth::parseUri(QStringLiteral("otpauth://weird/alice?secret=MFRGGZDF"))
                     .value(QStringLiteral("ok"))
                     .toBool());
    }

    void serviceGuessing()
    {
        const auto *gmail = serviceicons::guessService(QStringLiteral("Gmail"), QStringLiteral("user@example.com"));
        QVERIFY(gmail);
        QCOMPARE(gmail->key, QStringLiteral("gmail"));

        const auto *github = serviceicons::guessService(QStringLiteral("GitHub"), QStringLiteral("octocat"));
        QVERIFY(github);
        QCOMPARE(github->key, QStringLiteral("github"));

        QCOMPARE(serviceicons::guessDomain(QString(), QStringLiteral("user@example.com")),
                 QStringLiteral("example.com"));
        QCOMPARE(serviceicons::extractDomainFromText(QStringLiteral("https://Foo.Com/")),
                 QStringLiteral("foo.com"));
    }

    void pbkdf2Derivation()
    {
        // The derived key must be deterministic for password+device id and
        // 20 bytes for SHA-1.
        const QByteArray deviceId = QByteArray::fromHex("0102030405060708");
        const QByteArray key1 = OathSession::deriveKey(QStringLiteral("secret"), deviceId, Algorithm::Sha1);
        const QByteArray key2 = OathSession::deriveKey(QStringLiteral("secret"), deviceId, Algorithm::Sha1);
        QCOMPARE(key1.size(), 20);
        QCOMPARE(key1, key2);
        QVERIFY(key1 != OathSession::deriveKey(QStringLiteral("other"), deviceId, Algorithm::Sha1));
    }
};

QTEST_GUILESS_MAIN(TestOath)
#include "tst_oath.moc"
