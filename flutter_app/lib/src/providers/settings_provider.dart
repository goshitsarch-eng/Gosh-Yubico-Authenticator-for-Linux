import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Settings keys.
class SettingsKeys {
  static const String themeMode = 'theme_mode';
  static const String clipboardTimeout = 'clipboard_timeout';
  static const String requirePinOnLaunch = 'require_pin_on_launch';
}

/// Settings state.
class SettingsState {
  final ThemeMode themeMode;
  final int clipboardTimeoutSeconds;
  final bool requirePinOnLaunch;

  const SettingsState({
    this.themeMode = ThemeMode.system,
    this.clipboardTimeoutSeconds = 30,
    this.requirePinOnLaunch = false,
  });

  SettingsState copyWith({
    ThemeMode? themeMode,
    int? clipboardTimeoutSeconds,
    bool? requirePinOnLaunch,
  }) {
    return SettingsState(
      themeMode: themeMode ?? this.themeMode,
      clipboardTimeoutSeconds:
          clipboardTimeoutSeconds ?? this.clipboardTimeoutSeconds,
      requirePinOnLaunch: requirePinOnLaunch ?? this.requirePinOnLaunch,
    );
  }
}

/// Notifier for settings state.
class SettingsNotifier extends StateNotifier<SettingsState> {
  final SharedPreferences _prefs;

  SettingsNotifier(this._prefs) : super(const SettingsState()) {
    _loadSettings();
  }

  void _loadSettings() {
    final themeModeIndex = _prefs.getInt(SettingsKeys.themeMode) ?? 0;
    final clipboardTimeout =
        _prefs.getInt(SettingsKeys.clipboardTimeout) ?? 30;
    final requirePinOnLaunch =
        _prefs.getBool(SettingsKeys.requirePinOnLaunch) ?? false;

    state = SettingsState(
      themeMode: ThemeMode.values[themeModeIndex],
      clipboardTimeoutSeconds: clipboardTimeout,
      requirePinOnLaunch: requirePinOnLaunch,
    );
  }

  Future<void> setThemeMode(ThemeMode mode) async {
    await _prefs.setInt(SettingsKeys.themeMode, mode.index);
    state = state.copyWith(themeMode: mode);
  }

  Future<void> setClipboardTimeout(int seconds) async {
    await _prefs.setInt(SettingsKeys.clipboardTimeout, seconds);
    state = state.copyWith(clipboardTimeoutSeconds: seconds);
  }

  Future<void> setRequirePinOnLaunch(bool require) async {
    await _prefs.setBool(SettingsKeys.requirePinOnLaunch, require);
    state = state.copyWith(requirePinOnLaunch: require);
  }
}

/// Provider for SharedPreferences.
final sharedPreferencesProvider = Provider<SharedPreferences>((ref) {
  throw UnimplementedError('Initialize with override in main');
});

/// Provider for settings.
final settingsProvider =
    StateNotifierProvider<SettingsNotifier, SettingsState>((ref) {
  final prefs = ref.watch(sharedPreferencesProvider);
  return SettingsNotifier(prefs);
});

/// Provider for theme mode.
final themeModeProvider = Provider<ThemeMode>((ref) {
  final settings = ref.watch(settingsProvider);
  return settings.themeMode;
});
