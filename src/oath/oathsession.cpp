// SPDX-License-Identifier: GPL-3.0-or-later

#include "oathsession.h"

#include <QMessageAuthenticationCode>
#include <QRandomGenerator>
#include <QtNetwork/QPasswordDigestor>

#include <cstring>

namespace oath {

namespace {

constexpr int kReceiveBufferSize = 4096;
constexpr int kPbkdf2Iterations = 1000;

QByteArray hmacSha1(const QByteArray &key, const QByteArray &data)
{
    // The YubiKey always uses HMAC-SHA1 for authentication, regardless of
    // the credential algorithms in use.
    return QMessageAuthenticationCode::hash(data, key, QCryptographicHash::Sha1);
}

QByteArray randomBytes(int count)
{
    QByteArray bytes(count, Qt::Uninitialized);
    QRandomGenerator::system()->fillRange(reinterpret_cast<quint32 *>(bytes.data()),
                                          count / static_cast<int>(sizeof(quint32)));
    return bytes;
}

QString pcscErrorText(LONG rv)
{
    return QStringLiteral("PC/SC error 0x%1").arg(static_cast<qulonglong>(rv), 0, 16);
}

} // namespace

QByteArray OathSession::deriveKey(const QString &password, const QByteArray &deviceId, Algorithm algorithm)
{
    return QPasswordDigestor::deriveKeyPbkdf2(QCryptographicHash::Sha1,
                                              password.toUtf8(),
                                              deviceId,
                                              kPbkdf2Iterations,
                                              static_cast<quint64>(hmacKeySize(algorithm)));
}

SelectInfo OathSession::parseSelectResponse(const QByteArray &response)
{
    if (response.size() < 2) {
        throw OathException(ErrorCode::InvalidResponse, QStringLiteral("Response too short"));
    }
    const quint16 status = (static_cast<quint16>(static_cast<quint8>(response.at(response.size() - 2))) << 8)
        | static_cast<quint8>(response.at(response.size() - 1));
    if (status != sw::Success) {
        throw OathException(ErrorCode::SelectFailed, QStringLiteral("Failed to select OATH applet"));
    }

    const QByteArray data = response.left(response.size() - 2);
    SelectInfo info;
    bool haveDeviceId = false;

    bool malformed = false;
    const QList<Tlv> tlvs = parseTlv(data, &malformed);
    if (malformed) {
        throw OathException(ErrorCode::InvalidResponse,
                            QStringLiteral("SELECT response contains malformed TLV data"));
    }

    for (const Tlv &tlv : tlvs) {
        if (tlv.tag == static_cast<quint8>(Tag::Version) && tlv.value.size() >= 3) {
            info.version.major = static_cast<quint8>(tlv.value.at(0));
            info.version.minor = static_cast<quint8>(tlv.value.at(1));
            info.version.patch = static_cast<quint8>(tlv.value.at(2));
        } else if (tlv.tag == static_cast<quint8>(Tag::Name)) {
            if (tlv.value.size() != 8) {
                throw OathException(ErrorCode::InvalidResponse,
                                    QStringLiteral("SELECT response contains an invalid device ID length"));
            }
            if (haveDeviceId) {
                throw OathException(ErrorCode::InvalidResponse,
                                    QStringLiteral("SELECT response contains duplicate device IDs"));
            }
            info.deviceId = tlv.value;
            haveDeviceId = true;
        } else if (tlv.tag == static_cast<quint8>(Tag::Challenge)) {
            info.challenge = tlv.value;
        } else if (tlv.tag == static_cast<quint8>(Tag::Algorithm) && !tlv.value.isEmpty()) {
            info.challengeAlgorithm = algorithmFromByte(static_cast<quint8>(tlv.value.at(0)));
        }
    }

    if (!haveDeviceId) {
        throw OathException(ErrorCode::InvalidResponse,
                            QStringLiteral("SELECT response is missing the device ID"));
    }
    return info;
}

std::unique_ptr<OathSession> OathSession::connect()
{
    SCARDCONTEXT context = 0;
    LONG rv = SCardEstablishContext(SCARD_SCOPE_USER, nullptr, nullptr, &context);
    if (rv != SCARD_S_SUCCESS) {
        throw OathException(ErrorCode::PcscError,
                            QStringLiteral("Failed to establish PC/SC context: %1").arg(pcscErrorText(rv)));
    }

    DWORD readersLen = 0;
    rv = SCardListReaders(context, nullptr, nullptr, &readersLen);
    if (rv != SCARD_S_SUCCESS || readersLen == 0) {
        SCardReleaseContext(context);
        throw OathException(ErrorCode::NoDevice, QStringLiteral("No YubiKey found"));
    }

    QByteArray readerBuffer(static_cast<int>(readersLen), '\0');
    rv = SCardListReaders(context, nullptr, readerBuffer.data(), &readersLen);
    if (rv != SCARD_S_SUCCESS) {
        SCardReleaseContext(context);
        throw OathException(ErrorCode::NoDevice, QStringLiteral("No YubiKey found"));
    }

    // The buffer is a multi-string: NUL-separated names, double NUL at the end.
    const char *ptr = readerBuffer.constData();
    while (*ptr != '\0') {
        const QByteArray readerName(ptr);
        ptr += std::strlen(ptr) + 1;

        SCARDHANDLE card = 0;
        DWORD protocol = 0;
        rv = SCardConnect(context,
                          readerName.constData(),
                          SCARD_SHARE_SHARED,
                          SCARD_PROTOCOL_T0 | SCARD_PROTOCOL_T1,
                          &card,
                          &protocol);
        if (rv != SCARD_S_SUCCESS) {
            continue;
        }

        try {
            std::unique_ptr<OathSession> session(new OathSession(context, card, protocol, SelectInfo{}));
            const QByteArray response = session->transmit(buildSelectApdu());
            session->m_selectInfo = parseSelectResponse(response);
            session->m_authenticated = session->m_selectInfo.challenge.isEmpty();
            session->m_ownsContext = true;
            return session;
        } catch (const OathException &) {
            // Not an OATH-capable card in this reader; the failed session
            // released the card handle but left the shared context alive.
            continue;
        }
    }

    SCardReleaseContext(context);
    throw OathException(ErrorCode::NoDevice, QStringLiteral("No YubiKey found"));
}

OathSession::OathSession(SCARDCONTEXT context, SCARDHANDLE card, DWORD protocol, SelectInfo info)
    : m_context(context)
    , m_card(card)
    , m_protocol(protocol)
    , m_selectInfo(std::move(info))
{
}

OathSession::~OathSession()
{
    if (m_card) {
        SCardDisconnect(m_card, SCARD_LEAVE_CARD);
        m_card = 0;
    }
    if (m_context && m_ownsContext) {
        SCardReleaseContext(m_context);
        m_context = 0;
    }
}

QByteArray OathSession::transmit(const QByteArray &apdu)
{
    const SCARD_IO_REQUEST *sendPci =
        (m_protocol == SCARD_PROTOCOL_T1) ? SCARD_PCI_T1 : SCARD_PCI_T0;

    QByteArray full;
    QByteArray sendBuffer = apdu;

    for (;;) {
        QByteArray recvBuffer(kReceiveBufferSize, '\0');
        DWORD recvLen = static_cast<DWORD>(recvBuffer.size());
        const LONG rv = SCardTransmit(m_card,
                                      sendPci,
                                      reinterpret_cast<const BYTE *>(sendBuffer.constData()),
                                      static_cast<DWORD>(sendBuffer.size()),
                                      nullptr,
                                      reinterpret_cast<BYTE *>(recvBuffer.data()),
                                      &recvLen);
        if (rv != SCARD_S_SUCCESS) {
            const ErrorCode code = (rv == SCARD_W_REMOVED_CARD || rv == SCARD_E_READER_UNAVAILABLE)
                ? ErrorCode::Disconnected
                : ErrorCode::PcscError;
            throw OathException(code, QStringLiteral("YubiKey communication failed: %1").arg(pcscErrorText(rv)));
        }
        if (recvLen < 2) {
            throw OathException(ErrorCode::InvalidResponse, QStringLiteral("Response too short"));
        }

        recvBuffer.truncate(static_cast<int>(recvLen));
        const quint8 sw1 = static_cast<quint8>(recvBuffer.at(recvBuffer.size() - 2));
        if (sw1 == sw::MoreData) {
            full.append(recvBuffer.left(recvBuffer.size() - 2));
            sendBuffer = buildSendRemainingApdu();
            continue;
        }
        full.append(recvBuffer);
        return full;
    }
}

QByteArray OathSession::checkStatus(const QByteArray &response)
{
    if (response.size() < 2) {
        throw OathException(ErrorCode::InvalidResponse, QStringLiteral("Response too short"));
    }
    const quint16 status = (static_cast<quint16>(static_cast<quint8>(response.at(response.size() - 2))) << 8)
        | static_cast<quint8>(response.at(response.size() - 1));

    switch (status) {
    case sw::Success:
        return response.left(response.size() - 2);
    case sw::AuthRequired:
        throw OathException(ErrorCode::AuthenticationRequired,
                            QStringLiteral("Authentication required (YubiKey is password protected)"));
    case sw::NoSuchObject:
        throw OathException(ErrorCode::CredentialNotFound, QStringLiteral("Credential not found"));
    case sw::NoSpace:
        throw OathException(ErrorCode::NoSpace, QStringLiteral("No space left on device"));
    case sw::WrongSyntax:
        throw OathException(ErrorCode::InvalidName, QStringLiteral("Invalid credential name"));
    case sw::AuthNotInitialized:
        throw OathException(ErrorCode::WrongPassword, QStringLiteral("Wrong password"));
    default:
        throw OathException(ErrorCode::Generic,
                            QStringLiteral("APDU error: SW=%1").arg(status, 4, 16, QLatin1Char('0')));
    }
}

void OathSession::validate(const QString &password)
{
    if (m_selectInfo.challenge.isEmpty()) {
        m_authenticated = true;
        return;
    }

    const Algorithm algorithm = m_selectInfo.challengeAlgorithm.value_or(Algorithm::Sha1);
    const QByteArray key = deriveKey(password, m_selectInfo.deviceId, algorithm);
    const QByteArray response = hmacSha1(key, m_selectInfo.challenge);
    const QByteArray ourChallenge = randomBytes(8);

    const QByteArray reply = transmit(buildValidateApdu(response, ourChallenge));
    const QByteArray data = checkStatus(reply);

    const QList<Tlv> tlvs = parseTlv(data);
    for (const Tlv &tlv : tlvs) {
        if (tlv.tag == static_cast<quint8>(Tag::Response)) {
            if (tlv.value == hmacSha1(key, ourChallenge)) {
                m_authenticated = true;
                return;
            }
        }
    }
    throw OathException(ErrorCode::WrongPassword, QStringLiteral("Wrong password"));
}

QList<Credential> OathSession::listCredentials()
{
    if (requiresAuth()) {
        throw OathException(ErrorCode::AuthenticationRequired,
                            QStringLiteral("Authentication required"));
    }

    const QByteArray data = checkStatus(transmit(buildListApdu()));

    QList<Credential> credentials;
    const QList<Tlv> tlvs = parseTlv(data);
    for (const Tlv &tlv : tlvs) {
        if (tlv.tag != static_cast<quint8>(Tag::NameList) || tlv.value.isEmpty()) {
            continue;
        }
        const quint8 typeAlgo = static_cast<quint8>(tlv.value.at(0));
        const QByteArray name = tlv.value.mid(1);
        const auto [issuer, account] = parseCredentialName(name);

        Credential credential;
        credential.id = name;
        credential.issuer = issuer;
        credential.account = account;
        credential.type = oathTypeFromByte(typeAlgo).value_or(OathType::Totp);
        credential.algorithm = algorithmFromByte(typeAlgo).value_or(Algorithm::Sha1);
        credentials.append(credential);
    }
    return credentials;
}

QList<OathSession::CalculatedCode> OathSession::calculateAll(qint64 unixTime)
{
    if (requiresAuth()) {
        throw OathException(ErrorCode::AuthenticationRequired,
                            QStringLiteral("Authentication required"));
    }

    const QByteArray data = checkStatus(transmit(buildCalculateAllApdu(totpChallenge(unixTime, 30))));

    QList<CalculatedCode> results;
    const QList<Tlv> tlvs = parseTlv(data);
    for (int i = 0; i + 1 < tlvs.size(); ++i) {
        if (tlvs.at(i).tag != static_cast<quint8>(Tag::Name)) {
            continue;
        }
        const Tlv &responseTlv = tlvs.at(i + 1);
        CalculatedCode result;
        result.id = tlvs.at(i).value;
        if (responseTlv.tag == static_cast<quint8>(Tag::TruncatedResponse)) {
            result.code = parseTruncatedResponse(responseTlv.value);
        }
        // Tag::Touch and Tag::Hotp responses carry no code; the credential
        // needs an individual CALCULATE.
        results.append(result);
        ++i;
    }
    return results;
}

QPair<quint32, int> OathSession::calculate(const Credential &credential, qint64 unixTime)
{
    if (requiresAuth()) {
        throw OathException(ErrorCode::AuthenticationRequired,
                            QStringLiteral("Authentication required"));
    }

    const QByteArray challenge = credential.type == OathType::Totp
        ? totpChallenge(unixTime, credential.period)
        : QByteArray();

    QByteArray data;
    try {
        data = checkStatus(transmit(buildCalculateApdu(credential.id, challenge)));
    } catch (const OathException &e) {
        if (e.code() == ErrorCode::CredentialNotFound) {
            // SW 0x6984 doubles as "conditions not satisfied" while the key
            // waits for a touch that never came.
            throw OathException(ErrorCode::TouchRequired, QStringLiteral("Touch required on YubiKey"));
        }
        throw;
    }

    const QList<Tlv> tlvs = parseTlv(data);
    for (const Tlv &tlv : tlvs) {
        if (tlv.tag == static_cast<quint8>(Tag::TruncatedResponse)
            || tlv.tag == static_cast<quint8>(Tag::Response)) {
            if (const auto code = parseTruncatedResponse(tlv.value)) {
                return *code;
            }
        }
        if (tlv.tag == static_cast<quint8>(Tag::Touch)) {
            throw OathException(ErrorCode::TouchRequired, QStringLiteral("Touch required on YubiKey"));
        }
    }
    throw OathException(ErrorCode::InvalidResponse, QStringLiteral("No code in response"));
}

void OathSession::putCredential(const QString &issuer,
                                const QString &account,
                                const QByteArray &secret,
                                OathType type,
                                Algorithm algorithm,
                                int digits,
                                bool requireTouch,
                                std::optional<quint32> initialCounter)
{
    if (requiresAuth()) {
        throw OathException(ErrorCode::AuthenticationRequired,
                            QStringLiteral("Authentication required"));
    }

    const QByteArray name = buildCredentialName(issuer, account);
    if (name.isEmpty() || name.size() > 64) {
        throw OathException(ErrorCode::InvalidName, QStringLiteral("Invalid credential name"));
    }

    // Zero-padding a short HMAC key is a no-op for code generation, so the
    // key stores identical codes for the same secret.
    QByteArray paddedSecret = secret;
    const int requiredLen = hmacKeySize(algorithm);
    if (paddedSecret.size() < requiredLen) {
        paddedSecret.append(QByteArray(requiredLen - paddedSecret.size(), '\0'));
    }

    checkStatus(transmit(buildPutApdu(name, paddedSecret, type, algorithm, digits, requireTouch, initialCounter)));
}

void OathSession::deleteCredential(const QByteArray &id)
{
    if (requiresAuth()) {
        throw OathException(ErrorCode::AuthenticationRequired,
                            QStringLiteral("Authentication required"));
    }
    checkStatus(transmit(buildDeleteApdu(id)));
}

void OathSession::setCode(const QString &newPassword)
{
    if (requiresAuth()) {
        throw OathException(ErrorCode::AuthenticationRequired,
                            QStringLiteral("Authentication required"));
    }

    if (newPassword.isEmpty()) {
        checkStatus(transmit(buildSetCodeApdu({}, {}, {})));
    } else {
        const QByteArray key = deriveKey(newPassword, m_selectInfo.deviceId, Algorithm::Sha1);
        const QByteArray challenge = randomBytes(8);
        const QByteArray response = hmacSha1(key, challenge);
        checkStatus(transmit(buildSetCodeApdu(key, challenge, response)));
    }
    m_authenticated = true;
}

} // namespace oath
