import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../ffi/ffi.dart';
import 'gosh_client_provider.dart';

/// Authentication state.
enum AuthState {
  none,
  required,
  authenticating,
  authenticated,
  failed,
}

/// State for authentication management.
class AuthNotifierState {
  final AuthState status;
  final String? errorMessage;

  const AuthNotifierState({
    this.status = AuthState.none,
    this.errorMessage,
  });

  AuthNotifierState copyWith({
    AuthState? status,
    String? errorMessage,
  }) {
    return AuthNotifierState(
      status: status ?? this.status,
      errorMessage: errorMessage,
    );
  }
}

/// Notifier for authentication state.
class AuthNotifier extends StateNotifier<AuthNotifierState> {
  final GoshClient _client;

  AuthNotifier(this._client) : super(const AuthNotifierState());

  /// Handle authentication required event.
  void onAuthRequired() {
    state = state.copyWith(status: AuthState.required, errorMessage: null);
  }

  /// Handle authentication success event.
  void onAuthSuccess() {
    state = state.copyWith(status: AuthState.authenticated, errorMessage: null);
  }

  /// Handle authentication failed event.
  void onAuthFailed(String? message) {
    state = state.copyWith(
      status: AuthState.failed,
      errorMessage: message ?? 'Authentication failed',
    );
  }

  /// Submit password for authentication.
  void authenticate(String password) {
    state = state.copyWith(status: AuthState.authenticating, errorMessage: null);
    _client.authenticate(password);
  }

  /// Cancel authentication.
  void cancel() {
    state = state.copyWith(status: AuthState.none, errorMessage: null);
  }

  /// Reset state after disconnect.
  void reset() {
    state = const AuthNotifierState();
  }
}

/// Provider for authentication state.
final authProvider =
    StateNotifierProvider<AuthNotifier, AuthNotifierState>((ref) {
  final client = ref.watch(goshClientProvider);
  final notifier = AuthNotifier(client);

  // Listen to events and update state
  ref.listen(goshEventStreamProvider, (previous, next) {
    next.whenData((event) {
      switch (event.type) {
        case GoshEventType.authRequired:
          notifier.onAuthRequired();
          break;
        case GoshEventType.authSuccess:
          notifier.onAuthSuccess();
          break;
        case GoshEventType.authFailed:
          notifier.onAuthFailed(event.message);
          break;
        case GoshEventType.disconnected:
          notifier.reset();
          break;
        default:
          break;
      }
    });
  });

  return notifier;
});

/// Provider for checking if authentication is required.
final isAuthRequiredProvider = Provider<bool>((ref) {
  final auth = ref.watch(authProvider);
  return auth.status == AuthState.required || auth.status == AuthState.failed;
});
