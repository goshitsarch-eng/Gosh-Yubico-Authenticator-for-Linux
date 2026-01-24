import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:system_theme/system_theme.dart';
import 'package:yaru/yaru.dart';

import 'src/app.dart';
import 'src/providers/providers.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  if (!kIsWeb && Platform.isWindows) {
    await SystemTheme.accentColor.load();
  }

  if (!kIsWeb && Platform.isLinux) {
    await YaruWindowTitleBar.ensureInitialized();
  }

  // Initialize SharedPreferences
  final prefs = await SharedPreferences.getInstance();

  runApp(
    ProviderScope(
      overrides: [
        sharedPreferencesProvider.overrideWithValue(prefs),
      ],
      child: const GoshAuthenticatorApp(),
    ),
  );
}
