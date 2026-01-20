import 'dart:async';
import 'dart:ffi';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';

import 'gosh_event.dart' as events;
import 'gosh_ffi_bindings.dart' as ffi_bindings;
import 'native_library.dart';

/// Dart wrapper for the Gosh Authenticator FFI client.
///
/// This class provides a Dart-friendly API for interacting with the
/// Rust backend, converting C callbacks into Dart streams.
class GoshClient {
  late final Pointer<ffi_bindings.GoshClient> _handle;
  late final NativeCallable<ffi_bindings.GoshEventCallback> _nativeCallback;
  final _eventController = StreamController<events.GoshEvent>.broadcast();
  bool _disposed = false;

  GoshClient() {
    // Initialize logging
    goshFfi.gosh_init_logging();

    // Create the client
    _handle = goshFfi.gosh_client_new();

    // Set up the callback
    _nativeCallback = NativeCallable<ffi_bindings.GoshEventCallback>.listener(
      _onEventNative,
    );

    goshFfi.gosh_client_set_callback(
      _handle,
      _nativeCallback.nativeFunction,
      nullptr,
    );

    // Register this client for callback routing
    _globalClient = this;
  }

  /// Stream of events from the Rust backend.
  Stream<events.GoshEvent> get eventStream => _eventController.stream;

  /// Connect to the YubiKey.
  void connect() {
    _checkDisposed();
    goshFfi.gosh_client_connect(_handle);
  }

  /// Refresh the credentials list.
  void refresh() {
    _checkDisposed();
    goshFfi.gosh_client_refresh(_handle);
  }

  /// Authenticate with the YubiKey using the provided password.
  void authenticate(String password) {
    _checkDisposed();
    final passwordPtr = password.toNativeUtf8();
    try {
      goshFfi.gosh_client_authenticate(_handle, passwordPtr.cast());
    } finally {
      calloc.free(passwordPtr);
    }
  }

  /// Calculate a credential's code (for touch-required or HOTP).
  void calculate(Uint8List credentialId) {
    _checkDisposed();
    final idPtr = calloc<Uint8>(credentialId.length);
    try {
      for (int i = 0; i < credentialId.length; i++) {
        idPtr[i] = credentialId[i];
      }
      goshFfi.gosh_client_calculate(_handle, idPtr, credentialId.length);
    } finally {
      calloc.free(idPtr);
    }
  }

  /// Delete a credential from the YubiKey.
  void deleteCredential(Uint8List credentialId) {
    _checkDisposed();
    final idPtr = calloc<Uint8>(credentialId.length);
    try {
      for (int i = 0; i < credentialId.length; i++) {
        idPtr[i] = credentialId[i];
      }
      goshFfi.gosh_client_delete_credential(_handle, idPtr, credentialId.length);
    } finally {
      calloc.free(idPtr);
    }
  }

  /// Add a new credential to the YubiKey.
  bool addCredential({
    String? issuer,
    required String account,
    required String secret,
    events.OathType oathType = events.OathType.totp,
    events.Algorithm algorithm = events.Algorithm.sha1,
    int digits = 6,
    bool requireTouch = false,
    int? initialCounter,
  }) {
    _checkDisposed();

    final issuerPtr = issuer != null ? issuer.toNativeUtf8() : nullptr;
    final accountPtr = account.toNativeUtf8();
    final secretPtr = secret.toNativeUtf8();

    try {
      return goshFfi.gosh_client_add_credential(
        _handle,
        issuerPtr.cast(),
        accountPtr.cast(),
        secretPtr.cast(),
        oathType.value,
        algorithm.value,
        digits,
        requireTouch ? 1 : 0,
        initialCounter ?? 0,
        initialCounter != null ? 1 : 0,
      );
    } finally {
      if (issuer != null) calloc.free(issuerPtr);
      calloc.free(accountPtr);
      calloc.free(secretPtr);
    }
  }

  /// Set or change the OATH password.
  ///
  /// Pass an empty string to remove password protection.
  void setPassword(String password) {
    _checkDisposed();
    final passwordPtr = password.toNativeUtf8();
    try {
      goshFfi.gosh_client_set_password(_handle, passwordPtr.cast());
    } finally {
      calloc.free(passwordPtr);
    }
  }

  /// Get the last error message.
  String? takeLastError() {
    _checkDisposed();
    final errorPtr = goshFfi.gosh_client_last_error_take(_handle);
    if (errorPtr == nullptr) return null;

    final error = errorPtr.cast<Utf8>().toDartString();
    goshFfi.gosh_string_free(errorPtr);
    return error;
  }

  /// Dispose of the client and release resources.
  void dispose() {
    if (_disposed) return;
    _disposed = true;

    _nativeCallback.close();
    goshFfi.gosh_client_free(_handle);
    _eventController.close();
    _globalClient = null;
  }

  void _checkDisposed() {
    if (_disposed) {
      throw StateError('GoshClient has been disposed');
    }
  }

  /// Native callback handler - called from C.
  static void _onEventNative(
    Pointer<ffi_bindings.GoshEvent> eventPtr,
    Pointer<Void> userData,
  ) {
    _globalClient?._handleEvent(eventPtr);
  }

  void _handleEvent(Pointer<ffi_bindings.GoshEvent> eventPtr) {
    if (_disposed) return;

    final event = _parseEvent(eventPtr);
    _eventController.add(event);

    // Free the event
    goshFfi.gosh_event_free(eventPtr);
  }

  events.GoshEvent _parseEvent(Pointer<ffi_bindings.GoshEvent> eventPtr) {
    final eventStruct = eventPtr.ref;
    final eventType = events.GoshEventType.values[eventStruct.event_type];

    // Parse version
    events.GoshVersion? version;
    if (eventType == events.GoshEventType.connected) {
      version = events.GoshVersion(
        major: eventStruct.version.major,
        minor: eventStruct.version.minor,
        patch: eventStruct.version.patch,
      );
    }

    // Parse device ID
    Uint8List? deviceId;
    if (eventType == events.GoshEventType.connected) {
      deviceId = Uint8List(8);
      for (int i = 0; i < 8; i++) {
        deviceId[i] = eventStruct.device_id[i];
      }
    }

    // Parse message
    String? message;
    if (eventStruct.message != nullptr) {
      message = eventStruct.message.cast<Utf8>().toDartString();
    }

    // Parse credentials
    List<events.Credential>? credentials;
    if (eventType == events.GoshEventType.credentialsUpdated &&
        eventStruct.credentials != nullptr &&
        eventStruct.credentials_len > 0) {
      credentials = [];
      for (int i = 0; i < eventStruct.credentials_len; i++) {
        final credPtr = eventStruct.credentials + i;
        credentials.add(_parseCredential(credPtr.ref));
      }
    }

    // Parse credential ID
    Uint8List? credentialId;
    if (eventStruct.id_ptr != nullptr && eventStruct.id_len > 0) {
      credentialId = Uint8List(eventStruct.id_len);
      for (int i = 0; i < eventStruct.id_len; i++) {
        credentialId[i] = eventStruct.id_ptr[i];
      }
    }

    // Parse code
    String? code;
    if (eventStruct.code != nullptr) {
      code = eventStruct.code.cast<Utf8>().toDartString();
    }

    return events.GoshEvent(
      type: eventType,
      version: version,
      deviceId: deviceId,
      message: message,
      credentials: credentials,
      credentialId: credentialId,
      code: code,
      digits: eventStruct.digits > 0 ? eventStruct.digits : null,
    );
  }

  events.Credential _parseCredential(ffi_bindings.GoshCredential cred) {
    // Parse ID
    final id = Uint8List(cred.id_len);
    for (int i = 0; i < cred.id_len; i++) {
      id[i] = cred.id_ptr[i];
    }

    // Parse issuer
    String? issuer;
    if (cred.issuer != nullptr) {
      issuer = cred.issuer.cast<Utf8>().toDartString();
    }

    // Parse account
    final account = cred.account != nullptr
        ? cred.account.cast<Utf8>().toDartString()
        : '';

    // Parse code
    String? code;
    if (cred.code != nullptr) {
      code = cred.code.cast<Utf8>().toDartString();
    }

    return events.Credential(
      id: id,
      issuer: issuer,
      account: account,
      code: code,
      oathType: events.OathType.fromValue(cred.oath_type),
      algorithm: events.Algorithm.fromValue(cred.algorithm),
      digits: cred.digits,
      touchRequired: cred.touch_required != 0,
      period: cred.period,
    );
  }
}

/// Global client instance for callback routing.
GoshClient? _globalClient;
