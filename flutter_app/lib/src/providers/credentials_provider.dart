import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../ffi/ffi.dart';
import 'gosh_client_provider.dart';

/// State for credentials management.
class CredentialsState {
  final List<Credential> credentials;
  final bool isLoading;
  final String? error;

  const CredentialsState({
    this.credentials = const [],
    this.isLoading = false,
    this.error,
  });

  CredentialsState copyWith({
    List<Credential>? credentials,
    bool? isLoading,
    String? error,
  }) {
    return CredentialsState(
      credentials: credentials ?? this.credentials,
      isLoading: isLoading ?? this.isLoading,
      error: error,
    );
  }
}

/// Notifier for credentials state.
class CredentialsNotifier extends StateNotifier<CredentialsState> {
  final GoshClient _client;

  CredentialsNotifier(this._client) : super(const CredentialsState());

  /// Update credentials from an event.
  void updateCredentials(List<Credential> credentials) {
    state = state.copyWith(
      credentials: credentials,
      isLoading: false,
      error: null,
    );
  }

  /// Update a single credential's code.
  void updateCredentialCode(Uint8List credentialId, String code) {
    final updatedCredentials = state.credentials.map((cred) {
      if (_listEquals(cred.id, credentialId)) {
        return cred.copyWith(code: code);
      }
      return cred;
    }).toList();

    state = state.copyWith(credentials: updatedCredentials);
  }

  /// Remove a credential from the list.
  void removeCredential(Uint8List credentialId) {
    final updatedCredentials = state.credentials
        .where((cred) => !_listEquals(cred.id, credentialId))
        .toList();

    state = state.copyWith(credentials: updatedCredentials);
  }

  /// Set loading state.
  void setLoading(bool loading) {
    state = state.copyWith(isLoading: loading);
  }

  /// Set error state.
  void setError(String? error) {
    state = state.copyWith(error: error, isLoading: false);
  }

  /// Refresh credentials.
  void refresh() {
    state = state.copyWith(isLoading: true);
    _client.refresh();
  }

  /// Calculate a credential (for touch-required or HOTP).
  void calculate(Uint8List credentialId) {
    _client.calculate(credentialId);
  }

  /// Delete a credential.
  void delete(Uint8List credentialId) {
    _client.deleteCredential(credentialId);
  }
}

bool _listEquals(List<int> a, List<int> b) {
  if (a.length != b.length) return false;
  for (int i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}

/// Provider for credentials state.
final credentialsProvider =
    StateNotifierProvider<CredentialsNotifier, CredentialsState>((ref) {
  final client = ref.watch(goshClientProvider);
  final notifier = CredentialsNotifier(client);

  // Listen to events and update state
  ref.listen(goshEventStreamProvider, (previous, next) {
    next.whenData((event) {
      switch (event.type) {
        case GoshEventType.credentialsUpdated:
          if (event.credentials != null) {
            notifier.updateCredentials(event.credentials!);
          }
          break;
        case GoshEventType.credentialCalculated:
          if (event.credentialId != null && event.code != null) {
            notifier.updateCredentialCode(event.credentialId!, event.code!);
          }
          break;
        case GoshEventType.credentialDeleted:
          if (event.credentialId != null) {
            notifier.removeCredential(event.credentialId!);
          }
          break;
        case GoshEventType.error:
          notifier.setError(event.message);
          break;
        default:
          break;
      }
    });
  });

  return notifier;
});
