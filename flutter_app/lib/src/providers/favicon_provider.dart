import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../services/favicon_cache_service.dart';
import '../services/favicon_service.dart';
import 'settings_provider.dart';

/// Provider for the favicon service.
final faviconServiceProvider = Provider<FaviconService>((ref) {
  final service = FaviconService();
  ref.onDispose(() => service.dispose());
  return service;
});

/// Provider for the favicon cache service.
final faviconCacheServiceProvider = Provider<FaviconCacheService>((ref) {
  final prefs = ref.watch(sharedPreferencesProvider);
  return FaviconCacheService(prefs);
});

/// Provider that fetches and caches favicons by domain.
/// Uses family modifier to support different domains.
final faviconProvider = FutureProvider.family<Uint8List?, String>((ref, domain) async {
  final cacheService = ref.watch(faviconCacheServiceProvider);
  final faviconService = ref.watch(faviconServiceProvider);

  // 1. Check cache first
  final cached = cacheService.getCached(domain);
  if (cached != null) {
    return cached;
  }

  // 2. Fetch from network
  final imageData = await faviconService.fetchFavicon(domain);

  // 3. Cache result if successful
  if (imageData != null) {
    await cacheService.cache(domain, imageData);
  }

  return imageData;
});

/// Provider to extract domain from credential info.
final domainExtractorProvider = Provider<String? Function(String?, String)>((ref) {
  final faviconService = ref.watch(faviconServiceProvider);
  return faviconService.extractDomain;
});
