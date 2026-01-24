import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yaru/yaru.dart';

import 'providers/providers.dart';
import 'screens/screens.dart';
import 'windows_ui/windows_app.dart';
import 'widgets/widgets.dart';

/// Main application widget.
class GoshAuthenticatorApp extends ConsumerWidget {
  const GoshAuthenticatorApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeMode = ref.watch(themeModeProvider);
    final isAuthRequired = ref.watch(isAuthRequiredProvider);
    final isTouchRequired = ref.watch(isTouchRequiredProvider);

    final isLinux = !kIsWeb && Platform.isLinux;
    final isWindows = !kIsWeb && Platform.isWindows;

    if (isWindows) {
      return const WindowsApp();
    }

    if (isLinux) {
      return YaruTheme(
        builder: (context, yaru, child) => _buildApp(
          ref,
          themeMode: themeMode,
          isAuthRequired: isAuthRequired,
          isTouchRequired: isTouchRequired,
          theme: yaru.theme,
          darkTheme: yaru.darkTheme,
        ),
      );
    }

    return _buildApp(
      ref,
      themeMode: themeMode,
      isAuthRequired: isAuthRequired,
      isTouchRequired: isTouchRequired,
      theme: ThemeData.light(useMaterial3: true),
      darkTheme: ThemeData.dark(useMaterial3: true),
    );
  }

  Widget _buildApp(
    WidgetRef ref, {
    required ThemeMode themeMode,
    required bool isAuthRequired,
    required bool isTouchRequired,
    required ThemeData theme,
    required ThemeData darkTheme,
  }) {
    return MaterialApp(
      title: 'Gosh Yubikey Manager',
      debugShowCheckedModeBanner: false,
      theme: theme,
      darkTheme: darkTheme,
      themeMode: themeMode,
      scrollBehavior: const MaterialScrollBehavior().copyWith(
        dragDevices: {
          PointerDeviceKind.mouse,
          PointerDeviceKind.touch,
          PointerDeviceKind.stylus,
          PointerDeviceKind.unknown,
          PointerDeviceKind.trackpad,
        },
      ),
      home: Stack(
        children: [
          // Main navigator
          Navigator(
            onGenerateRoute: _onGenerateRoute,
          ),

          // Touch prompt overlay
          if (isTouchRequired)
            TouchPrompt(
              onCancel: () => ref.read(touchProvider.notifier).hidePrompt(),
            ),

          // PIN entry overlay
          if (isAuthRequired) const PinEntryDialog(),
        ],
      ),
      routes: {
        '/home': (context) => const HomeScreen(),
        '/add': (context) => const AddCredentialScreen(),
        '/settings': (context) => const SettingsScreen(),
        '/about': (context) => const AboutScreen(),
      },
    );
  }

  Route<dynamic>? _onGenerateRoute(RouteSettings settings) {
    switch (settings.name) {
      case '/':
      case '/home':
        return MaterialPageRoute(
          builder: (context) => const HomeScreen(),
          settings: settings,
        );
      case '/add':
        return MaterialPageRoute(
          builder: (context) => const AddCredentialScreen(),
          settings: settings,
        );
      case '/settings':
        return MaterialPageRoute(
          builder: (context) => const SettingsScreen(),
          settings: settings,
        );
      case '/about':
        return MaterialPageRoute(
          builder: (context) => const AboutScreen(),
          settings: settings,
        );
      default:
        return MaterialPageRoute(
          builder: (context) => const HomeScreen(),
          settings: settings,
        );
    }
  }
}
