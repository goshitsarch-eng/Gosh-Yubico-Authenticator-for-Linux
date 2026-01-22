import 'dart:convert';
import 'dart:typed_data';

import 'package:shared_preferences/shared_preferences.dart';

/// Service for caching favicons in SharedPreferences.
class FaviconCacheService {
  static const _keyPrefix = 'favicon_';
  static const _timestampSuffix = '_ts';
  static const _cacheExpirationDays = 7;

  final SharedPreferences _prefs;

  FaviconCacheService(this._prefs);

  /// Get cached favicon for a domain.
  Uint8List? getCached(String domain) {
    final key = _getKey(domain);
    final timestampKey = _getTimestampKey(domain);

    // Check if cache exists
    final base64Data = _prefs.getString(key);
    if (base64Data == null) return null;

    // Check if cache is expired
    final timestamp = _prefs.getInt(timestampKey);
    if (timestamp != null) {
      final cachedTime = DateTime.fromMillisecondsSinceEpoch(timestamp);
      final now = DateTime.now();
      if (now.difference(cachedTime).inDays > _cacheExpirationDays) {
        // Cache expired, remove it
        _prefs.remove(key);
        _prefs.remove(timestampKey);
        return null;
      }
    }

    try {
      return base64Decode(base64Data);
    } catch (e) {
      // Invalid base64 data, remove it
      _prefs.remove(key);
      _prefs.remove(timestampKey);
      return null;
    }
  }

  /// Cache a favicon for a domain.
  Future<void> cache(String domain, Uint8List imageData) async {
    final key = _getKey(domain);
    final timestampKey = _getTimestampKey(domain);

    final base64Data = base64Encode(imageData);
    await _prefs.setString(key, base64Data);
    await _prefs.setInt(timestampKey, DateTime.now().millisecondsSinceEpoch);
  }

  /// Clear all expired favicon caches.
  Future<void> clearExpired() async {
    final keys = _prefs.getKeys();
    final now = DateTime.now();

    for (final key in keys) {
      if (key.startsWith(_keyPrefix) && !key.endsWith(_timestampSuffix)) {
        final domain = key.substring(_keyPrefix.length);
        final timestampKey = _getTimestampKey(domain);
        final timestamp = _prefs.getInt(timestampKey);

        if (timestamp != null) {
          final cachedTime = DateTime.fromMillisecondsSinceEpoch(timestamp);
          if (now.difference(cachedTime).inDays > _cacheExpirationDays) {
            await _prefs.remove(key);
            await _prefs.remove(timestampKey);
          }
        }
      }
    }
  }

  /// Clear favicon cache for a specific domain.
  Future<void> clearDomain(String domain) async {
    final key = _getKey(domain);
    final timestampKey = _getTimestampKey(domain);
    await _prefs.remove(key);
    await _prefs.remove(timestampKey);
  }

  /// Clear all favicon caches.
  Future<void> clearAll() async {
    final keys = _prefs.getKeys();
    for (final key in keys) {
      if (key.startsWith(_keyPrefix)) {
        await _prefs.remove(key);
      }
    }
  }

  String _getKey(String domain) => '$_keyPrefix$domain';
  String _getTimestampKey(String domain) => '$_keyPrefix${domain}$_timestampSuffix';
}
