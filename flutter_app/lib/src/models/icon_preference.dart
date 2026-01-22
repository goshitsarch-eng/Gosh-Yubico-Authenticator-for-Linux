import 'dart:convert';

/// Represents a user's custom icon preference for a credential.
class IconPreference {
  /// Key from the service icons map (e.g., 'github', 'gmail').
  /// If set, this takes priority over automatic icon detection.
  final String? customIconKey;

  /// Custom favicon URL domain (e.g., 'namecheap.com').
  /// Used to fetch and display a favicon for services not in the icon map.
  final String? faviconDomain;

  const IconPreference({
    this.customIconKey,
    this.faviconDomain,
  });

  /// Whether this preference has any custom icon set.
  bool get hasCustomIcon => customIconKey != null || faviconDomain != null;

  /// Create a preference with a custom service icon key.
  factory IconPreference.withServiceIcon(String key) {
    return IconPreference(customIconKey: key);
  }

  /// Create a preference with a custom favicon domain.
  factory IconPreference.withFavicon(String domain) {
    return IconPreference(faviconDomain: domain);
  }

  /// Create an empty preference (uses automatic icon detection).
  factory IconPreference.auto() {
    return const IconPreference();
  }

  /// Serialize to JSON string for SharedPreferences storage.
  String toJson() {
    return jsonEncode({
      if (customIconKey != null) 'customIconKey': customIconKey,
      if (faviconDomain != null) 'faviconDomain': faviconDomain,
    });
  }

  /// Deserialize from JSON string.
  factory IconPreference.fromJson(String jsonStr) {
    try {
      final map = jsonDecode(jsonStr) as Map<String, dynamic>;
      return IconPreference(
        customIconKey: map['customIconKey'] as String?,
        faviconDomain: map['faviconDomain'] as String?,
      );
    } catch (e) {
      return const IconPreference();
    }
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is IconPreference &&
          runtimeType == other.runtimeType &&
          customIconKey == other.customIconKey &&
          faviconDomain == other.faviconDomain;

  @override
  int get hashCode => Object.hash(customIconKey, faviconDomain);

  @override
  String toString() =>
      'IconPreference(customIconKey: $customIconKey, faviconDomain: $faviconDomain)';
}
