import 'dart:typed_data';

import 'package:http/http.dart' as http;

/// Service for fetching favicons from service URLs.
class FaviconService {
  static const _faviconBaseUrl = 'https://www.google.com/s2/favicons';
  static const _defaultSize = 64;
  static const _timeoutDuration = Duration(seconds: 10);

  final http.Client _client;

  FaviconService({http.Client? client}) : _client = client ?? http.Client();

  /// Fetch favicon for a domain using Google's S2 favicon service.
  Future<Uint8List?> fetchFavicon(String domain, {int size = _defaultSize}) async {
    try {
      final url = Uri.parse('$_faviconBaseUrl?domain=$domain&sz=$size');
      final response = await _client.get(url).timeout(_timeoutDuration);

      if (response.statusCode == 200 && response.bodyBytes.isNotEmpty) {
        // Check if we got an actual image (not the default blank favicon)
        // Google returns a 16x16 default icon if none is found
        if (response.bodyBytes.length > 100) {
          return response.bodyBytes;
        }
      }
      return null;
    } catch (e) {
      // Network error, timeout, etc.
      return null;
    }
  }

  /// Extract domain from issuer or account.
  /// Examples:
  /// - "user@github.com" → "github.com"
  /// - "GitHub" → "github.com"
  /// - "namecheap.com" → "namecheap.com"
  String? extractDomain(String? issuer, String account) {
    // First try extracting from email in account
    if (account.contains('@')) {
      final parts = account.split('@');
      if (parts.length == 2 && parts[1].contains('.')) {
        return parts[1].toLowerCase();
      }
    }

    // Try the issuer
    final source = issuer ?? account;
    final sourceLower = source.toLowerCase().trim();

    // If it already looks like a domain
    if (sourceLower.contains('.') && !sourceLower.contains(' ')) {
      // Remove protocol if present
      var domain = sourceLower
          .replaceAll('https://', '')
          .replaceAll('http://', '')
          .split('/')[0];
      return domain;
    }

    // Try to construct a domain from the service name
    // Remove special characters and spaces
    final cleanName = sourceLower
        .replaceAll(RegExp(r'[^a-z0-9]'), '')
        .trim();

    if (cleanName.isNotEmpty) {
      return '$cleanName.com';
    }

    return null;
  }

  /// Dispose the HTTP client.
  void dispose() {
    _client.close();
  }
}
