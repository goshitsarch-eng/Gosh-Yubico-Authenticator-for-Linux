import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

import '../models/icon_preference.dart';

/// Service for storing and retrieving icon preferences per credential.
/// Since credentials are stored on YubiKey hardware with no custom metadata,
/// we store icon preferences locally using SharedPreferences, keyed by credential ID.
class IconPreferencesService {
  static const _keyPrefix = 'icon_pref_';

  final SharedPreferences _prefs;

  IconPreferencesService(this._prefs);

  /// Get the icon preference for a credential.
  /// Returns null if no custom preference is set.
  IconPreference? getPreference(String credentialId) {
    final key = _getKey(credentialId);
    final jsonStr = _prefs.getString(key);
    if (jsonStr == null) return null;
    return IconPreference.fromJson(jsonStr);
  }

  /// Set the icon preference for a credential.
  Future<void> setPreference(String credentialId, IconPreference pref) async {
    final key = _getKey(credentialId);
    if (pref.hasCustomIcon) {
      await _prefs.setString(key, pref.toJson());
    } else {
      // If no custom icon, remove the preference to use auto-detection
      await _prefs.remove(key);
    }
  }

  /// Clear the icon preference for a credential (reset to auto-detection).
  Future<void> clearPreference(String credentialId) async {
    final key = _getKey(credentialId);
    await _prefs.remove(key);
  }

  /// Get all credential IDs that have custom icon preferences.
  List<String> getCredentialsWithPreferences() {
    final keys = _prefs.getKeys();
    return keys
        .where((key) => key.startsWith(_keyPrefix))
        .map((key) => key.substring(_keyPrefix.length))
        .toList();
  }

  /// Clear all icon preferences.
  Future<void> clearAll() async {
    final keys = _prefs.getKeys();
    for (final key in keys) {
      if (key.startsWith(_keyPrefix)) {
        await _prefs.remove(key);
      }
    }
  }

  /// Generate a credential ID from the credential's unique identifier bytes.
  /// This creates a stable string key from the credential's ID.
  static String credentialIdFromBytes(List<int> idBytes) {
    return base64Encode(idBytes);
  }

  String _getKey(String credentialId) => '$_keyPrefix$credentialId';
}
