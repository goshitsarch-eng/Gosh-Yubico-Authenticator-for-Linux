import 'dart:async';

import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/settings_provider.dart';

/// Service that manages clipboard operations with auto-clear functionality.
class ClipboardService {
  Timer? _clearTimer;
  String? _copiedValue;

  /// Copy text to clipboard and schedule auto-clear.
  void copyWithAutoClear(String text, int timeoutSeconds) {
    // Cancel any existing timer
    _clearTimer?.cancel();

    // Copy to clipboard
    Clipboard.setData(ClipboardData(text: text));
    _copiedValue = text;

    // Schedule clear
    _clearTimer = Timer(Duration(seconds: timeoutSeconds), () {
      _clearClipboard();
    });
  }

  /// Clear the clipboard if it still contains the copied value.
  Future<void> _clearClipboard() async {
    // Check if clipboard still contains our value
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    if (data?.text == _copiedValue) {
      // Clear by setting empty data
      await Clipboard.setData(const ClipboardData(text: ''));
    }
    _copiedValue = null;
  }

  /// Cancel any pending clear operation.
  void cancelClear() {
    _clearTimer?.cancel();
    _clearTimer = null;
    _copiedValue = null;
  }

  /// Dispose the service.
  void dispose() {
    cancelClear();
  }
}

/// Provider for the clipboard service.
final clipboardServiceProvider = Provider<ClipboardService>((ref) {
  final service = ClipboardService();
  ref.onDispose(service.dispose);
  return service;
});

/// Helper provider that copies text with auto-clear using current settings.
final copyToClipboardProvider = Provider<void Function(String)>((ref) {
  final service = ref.watch(clipboardServiceProvider);
  final settings = ref.watch(settingsProvider);

  return (String text) {
    service.copyWithAutoClear(text, settings.clipboardTimeoutSeconds);
  };
});
