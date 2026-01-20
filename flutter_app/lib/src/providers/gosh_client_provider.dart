import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../ffi/ffi.dart';

/// Provider for the GoshClient singleton.
final goshClientProvider = Provider<GoshClient>((ref) {
  final client = GoshClient();

  // Start connecting automatically
  client.connect();

  // Dispose when the provider is disposed
  ref.onDispose(() {
    client.dispose();
  });

  return client;
});

/// Provider for the event stream.
final goshEventStreamProvider = StreamProvider<GoshEvent>((ref) {
  final client = ref.watch(goshClientProvider);
  return client.eventStream;
});
