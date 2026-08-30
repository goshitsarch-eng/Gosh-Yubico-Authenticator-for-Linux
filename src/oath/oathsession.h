// SPDX-License-Identifier: GPL-3.0-or-later
//
// PC/SC connection and OATH session for YubiKey devices.

#pragma once

#include "apdu.h"

#include <QByteArray>
#include <QList>
#include <QString>

#include <memory>
#include <optional>
#include <stdexcept>

#include <winscard.h>

namespace oath {

enum class ErrorCode {
    NoDevice,
    SelectFailed,
    AuthenticationRequired,
    WrongPassword,
    TouchRequired,
    CredentialNotFound,
    NoSpace,
    InvalidName,
    InvalidResponse,
    PcscError,
    Disconnected,
    Generic,
};

class OathException : public std::runtime_error {
public:
    OathException(ErrorCode code, const QString &message)
        : std::runtime_error(message.toStdString())
        , m_code(code)
        , m_message(message)
    {
    }

    ErrorCode code() const { return m_code; }
    QString message() const { return m_message; }

private:
    ErrorCode m_code;
    QString m_message;
};

struct Version {
    int major = 0;
    int minor = 0;
    int patch = 0;
};

/// Parsed data from the OATH applet SELECT response.
struct SelectInfo {
    Version version;
    QByteArray deviceId; // exactly 8 bytes, fail-closed when missing/invalid
    QByteArray challenge; // non-empty when the applet is password protected
    std::optional<Algorithm> challengeAlgorithm;
};

struct Credential {
    QByteArray id; // raw name bytes as stored on the key
    QString issuer;
    QString account;
    OathType type = OathType::Totp;
    Algorithm algorithm = Algorithm::Sha1;
    int digits = 6;
    bool touchRequired = false;
    QString code; // formatted code, empty when not calculated
    int period = 30;

    QString displayName() const
    {
        return issuer.isEmpty() ? account : issuer + QStringLiteral(": ") + account;
    }
};

/// A live OATH session over PC/SC. All methods are blocking and throw
/// OathException on failure; use from a worker thread.
class OathSession {
public:
    /// Connect to the first reader exposing the OATH applet.
    static std::unique_ptr<OathSession> connect();

    ~OathSession();
    OathSession(const OathSession &) = delete;
    OathSession &operator=(const OathSession &) = delete;

    bool requiresAuth() const { return !m_selectInfo.challenge.isEmpty() && !m_authenticated; }
    bool hasPassword() const { return !m_selectInfo.challenge.isEmpty(); }
    Version version() const { return m_selectInfo.version; }
    QByteArray deviceId() const { return m_selectInfo.deviceId; }

    /// Authenticate with the OATH password. Throws WrongPassword on mismatch.
    void validate(const QString &password);

    /// List all credentials stored on the key (without codes).
    QList<Credential> listCredentials();

    /// Calculate all TOTP codes at once. Credentials whose code cannot be
    /// produced in bulk (touch required, HOTP) come back with an empty code.
    struct CalculatedCode {
        QByteArray id;
        std::optional<QPair<quint32, int>> code; // (code, digits)
    };
    QList<CalculatedCode> calculateAll(qint64 unixTime);

    /// Calculate a single credential. Throws TouchRequired when the key
    /// wants a touch confirmation.
    QPair<quint32, int> calculate(const Credential &credential, qint64 unixTime);

    void putCredential(const QString &issuer,
                       const QString &account,
                       const QByteArray &secret,
                       OathType type,
                       Algorithm algorithm,
                       int digits,
                       bool requireTouch,
                       std::optional<quint32> initialCounter);

    void deleteCredential(const QByteArray &id);

    /// Set (non-empty) or remove (empty) the OATH password.
    void setCode(const QString &newPassword);

    /// Parse the SELECT response; exposed for unit tests. Throws on
    /// malformed data, including a missing or invalid device ID.
    static SelectInfo parseSelectResponse(const QByteArray &response);

    /// PBKDF2-SHA1 key derivation used for OATH authentication.
    static QByteArray deriveKey(const QString &password, const QByteArray &deviceId, Algorithm algorithm);

private:
    OathSession(SCARDCONTEXT context, SCARDHANDLE card, DWORD protocol, SelectInfo info);

    QByteArray transmit(const QByteArray &apdu);
    QByteArray checkStatus(const QByteArray &response);

    SCARDCONTEXT m_context = 0;
    SCARDHANDLE m_card = 0;
    DWORD m_protocol = 0;
    SelectInfo m_selectInfo;
    bool m_authenticated = false;
    // connect() probes several readers with one context; a probe session
    // must not tear the shared context down when it fails mid-loop.
    bool m_ownsContext = false;
};

} // namespace oath
