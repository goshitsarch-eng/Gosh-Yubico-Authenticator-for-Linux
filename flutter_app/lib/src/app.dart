import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'providers/providers.dart';
import 'screens/screens.dart';
import 'theme/theme.dart';
import 'widgets/widgets.dart';

/// Main application widget.
class GoshAuthenticatorApp extends ConsumerWidget {
  const GoshAuthenticatorApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeMode = ref.watch(themeModeProvider);
    final isAuthRequired = ref.watch(isAuthRequiredProvider);
    final isTouchRequired = ref.watch(isTouchRequiredProvider);

    return MaterialApp(
      title: 'Gosh Yubikey Manager',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.light(),
      darkTheme: AppTheme.dark(),
      themeMode: themeMode,
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
