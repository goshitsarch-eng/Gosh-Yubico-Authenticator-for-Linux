import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../ffi/ffi.dart';
import 'gosh_client_provider.dart';

/// State for touch required prompt.
class TouchState {
  final bool isRequired;
  final Uint8List? credentialId;

  const TouchState({
    this.isRequired = false,
    this.credentialId,
  });

  TouchState copyWith({
    bool? isRequired,
    Uint8List? credentialId,
  }) {
    return TouchState(
      isRequired: isRequired ?? this.isRequired,
      credentialId: credentialId ?? this.credentialId,
    );
  }
}

/// Notifier for touch state.
class TouchNotifier extends StateNotifier<TouchState> {
  TouchNotifier() : super(const TouchState());

  /// Show touch prompt.
  void showPrompt(Uint8List? credentialId) {
    state = TouchState(isRequired: true, credentialId: credentialId);
  }

  /// Hide touch prompt.
  void hidePrompt() {
    state = const TouchState();
  }
}

/// Provider for touch state.
final touchProvider = StateNotifierProvider<TouchNotifier, TouchState>((ref) {
  final notifier = TouchNotifier();

  // Listen to events
  ref.listen(goshEventStreamProvider, (previous, next) {
    next.whenData((event) {
      switch (event.type) {
        case GoshEventType.touchRequired:
          notifier.showPrompt(event.credentialId);
          break;
        case GoshEventType.credentialCalculated:
        case GoshEventType.error:
        case GoshEventType.disconnected:
          notifier.hidePrompt();
          break;
        default:
          break;
      }
    });
  });

  return notifier;
});

/// Provider for checking if touch is required.
final isTouchRequiredProvider = Provider<bool>((ref) {
  final touch = ref.watch(touchProvider);
  return touch.isRequired;
});
