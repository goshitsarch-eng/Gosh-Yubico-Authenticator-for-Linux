import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/icon_preference.dart';
import '../services/icon_preferences_service.dart';
import 'settings_provider.dart';

/// Provider for the icon preferences service.
final iconPreferencesServiceProvider = Provider<IconPreferencesService>((ref) {
  final prefs = ref.watch(sharedPreferencesProvider);
  return IconPreferencesService(prefs);
});

/// State notifier for managing icon preferences with reactive updates.
class IconPreferencesNotifier extends StateNotifier<Map<String, IconPreference>> {
  final IconPreferencesService _service;

  IconPreferencesNotifier(this._service) : super({}) {
    _loadAll();
  }

  void _loadAll() {
    final credentialIds = _service.getCredentialsWithPreferences();
    final prefs = <String, IconPreference>{};
    for (final id in credentialIds) {
      final pref = _service.getPreference(id);
      if (pref != null) {
        prefs[id] = pref;
      }
    }
    state = prefs;
  }

  /// Get preference for a specific credential.
  IconPreference? getPreference(String credentialId) {
    return state[credentialId];
  }

  /// Set preference for a credential.
  Future<void> setPreference(String credentialId, IconPreference pref) async {
    await _service.setPreference(credentialId, pref);
    if (pref.hasCustomIcon) {
      state = {...state, credentialId: pref};
    } else {
      final newState = Map<String, IconPreference>.from(state);
      newState.remove(credentialId);
      state = newState;
    }
  }

  /// Clear preference for a credential (reset to auto-detection).
  Future<void> clearPreference(String credentialId) async {
    await _service.clearPreference(credentialId);
    final newState = Map<String, IconPreference>.from(state);
    newState.remove(credentialId);
    state = newState;
  }

  /// Clear all preferences.
  Future<void> clearAll() async {
    await _service.clearAll();
    state = {};
  }
}

/// Provider for icon preferences state notifier.
final iconPreferencesProvider =
    StateNotifierProvider<IconPreferencesNotifier, Map<String, IconPreference>>((ref) {
  final service = ref.watch(iconPreferencesServiceProvider);
  return IconPreferencesNotifier(service);
});

/// Provider to get a specific credential's icon preference.
/// Uses family modifier to support different credential IDs.
final iconPreferenceProvider = Provider.family<IconPreference?, String>((ref, credentialId) {
  final preferences = ref.watch(iconPreferencesProvider);
  return preferences[credentialId];
});
