import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../ffi/ffi.dart';
import 'gosh_client_provider.dart';

/// Connection state.
enum ConnectionState {
  disconnected,
  connecting,
  connected,
}

/// Information about the connected YubiKey.
class YubiKeyInfo {
  final GoshVersion version;
  final Uint8List deviceId;

  const YubiKeyInfo({
    required this.version,
    required this.deviceId,
  });

  String get deviceName => 'YubiKey ${version.major}';
}

/// State for connection management.
class ConnectionNotifierState {
  final ConnectionState status;
  final YubiKeyInfo? yubiKeyInfo;

  const ConnectionNotifierState({
    this.status = ConnectionState.disconnected,
    this.yubiKeyInfo,
  });

  ConnectionNotifierState copyWith({
    ConnectionState? status,
    YubiKeyInfo? yubiKeyInfo,
  }) {
    return ConnectionNotifierState(
      status: status ?? this.status,
      yubiKeyInfo: yubiKeyInfo ?? this.yubiKeyInfo,
    );
  }
}

/// Notifier for connection state.
class ConnectionNotifier extends StateNotifier<ConnectionNotifierState> {
  final GoshClient _client;

  ConnectionNotifier(this._client)
      : super(const ConnectionNotifierState(status: ConnectionState.connecting));

  /// Handle connected event.
  void onConnected(GoshVersion version, Uint8List deviceId) {
    state = state.copyWith(
      status: ConnectionState.connected,
      yubiKeyInfo: YubiKeyInfo(version: version, deviceId: deviceId),
    );
  }

  /// Handle disconnected event.
  void onDisconnected() {
    state = const ConnectionNotifierState(status: ConnectionState.disconnected);
  }

  /// Reconnect to the YubiKey.
  void reconnect() {
    state = state.copyWith(status: ConnectionState.connecting);
    _client.connect();
  }
}

/// Provider for connection state.
final connectionProvider =
    StateNotifierProvider<ConnectionNotifier, ConnectionNotifierState>((ref) {
  final client = ref.watch(goshClientProvider);
  final notifier = ConnectionNotifier(client);

  // Listen to events and update state
  ref.listen(goshEventStreamProvider, (previous, next) {
    next.whenData((event) {
      switch (event.type) {
        case GoshEventType.connected:
          if (event.version != null && event.deviceId != null) {
            notifier.onConnected(event.version!, event.deviceId!);
          }
          break;
        case GoshEventType.disconnected:
          notifier.onDisconnected();
          break;
        default:
          break;
      }
    });
  });

  return notifier;
});

/// Provider for checking if YubiKey is connected.
final isConnectedProvider = Provider<bool>((ref) {
  final connection = ref.watch(connectionProvider);
  return connection.status == ConnectionState.connected;
});

/// Provider for YubiKey info.
final yubiKeyInfoProvider = Provider<YubiKeyInfo?>((ref) {
  final connection = ref.watch(connectionProvider);
  return connection.yubiKeyInfo;
});
