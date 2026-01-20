import 'dart:typed_data';

/// Event types from the Rust backend.
enum GoshEventType {
  ready,             // 0
  connected,         // 1
  disconnected,      // 2
  authRequired,      // 3
  authSuccess,       // 4
  authFailed,        // 5
  credentialsUpdated, // 6
  credentialCalculated, // 7
  touchRequired,     // 8
  credentialAdded,   // 9
  credentialDeleted, // 10
  error,             // 11
  passwordChanged,   // 12
  passwordRemoved,   // 13
}

/// OATH type enumeration.
enum OathType {
  hotp(0x10),
  totp(0x20);

  final int value;
  const OathType(this.value);

  static OathType fromValue(int value) {
    return OathType.values.firstWhere(
      (e) => e.value == value,
      orElse: () => OathType.totp,
    );
  }
}

/// Algorithm enumeration.
enum Algorithm {
  sha1(0x01),
  sha256(0x02),
  sha512(0x03);

  final int value;
  const Algorithm(this.value);

  static Algorithm fromValue(int value) {
    return Algorithm.values.firstWhere(
      (e) => e.value == value,
      orElse: () => Algorithm.sha1,
    );
  }
}

/// YubiKey version information.
class GoshVersion {
  final int major;
  final int minor;
  final int patch;

  const GoshVersion({
    required this.major,
    required this.minor,
    required this.patch,
  });

  @override
  String toString() => '$major.$minor.$patch';
}

/// Credential stored on YubiKey.
class Credential {
  final Uint8List id;
  final String? issuer;
  final String account;
  final String? code;
  final OathType oathType;
  final Algorithm algorithm;
  final int digits;
  final bool touchRequired;
  final int period;

  const Credential({
    required this.id,
    this.issuer,
    required this.account,
    this.code,
    required this.oathType,
    required this.algorithm,
    required this.digits,
    required this.touchRequired,
    required this.period,
  });

  /// Display name for the credential.
  String get displayName =>
      issuer != null && issuer!.isNotEmpty ? '$issuer: $account' : account;

  /// Whether this is a TOTP credential.
  bool get isTotp => oathType == OathType.totp;

  /// Whether this is a HOTP credential.
  bool get isHotp => oathType == OathType.hotp;

  /// Create a copy with updated code.
  Credential copyWith({String? code}) {
    return Credential(
      id: id,
      issuer: issuer,
      account: account,
      code: code ?? this.code,
      oathType: oathType,
      algorithm: algorithm,
      digits: digits,
      touchRequired: touchRequired,
      period: period,
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Credential &&
          runtimeType == other.runtimeType &&
          _listEquals(id, other.id);

  @override
  int get hashCode => Object.hashAll(id);
}

bool _listEquals(List<int> a, List<int> b) {
  if (a.length != b.length) return false;
  for (int i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}

/// Event received from the Rust backend.
class GoshEvent {
  final GoshEventType type;
  final GoshVersion? version;
  final Uint8List? deviceId;
  final String? message;
  final List<Credential>? credentials;
  final Uint8List? credentialId;
  final String? code;
  final int? digits;

  const GoshEvent({
    required this.type,
    this.version,
    this.deviceId,
    this.message,
    this.credentials,
    this.credentialId,
    this.code,
    this.digits,
  });

  @override
  String toString() => 'GoshEvent(type: $type, message: $message)';
}
