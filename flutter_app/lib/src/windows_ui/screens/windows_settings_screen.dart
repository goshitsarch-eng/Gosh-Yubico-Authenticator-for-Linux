import 'package:fluent_ui/fluent_ui.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../providers/connection_provider.dart' as conn;
import '../../providers/providers.dart';
import '../widgets/windows_change_password_dialog.dart';

class WindowsSettingsScreen extends ConsumerWidget {
  const WindowsSettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final settings = ref.watch(settingsProvider);
    final connection = ref.watch(connectionProvider);
    final isConnected = connection.status == conn.ConnectionState.connected;

    return ScaffoldPage.scrollable(
      header: const PageHeader(title: Text('Settings')),
      children: [
        _SectionHeader(title: 'General'),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                InfoLabel(
                  label: 'Theme',
                  child: ComboBox<ThemeMode>(
                    value: settings.themeMode,
                    items: ThemeMode.values
                        .map(
                          (m) => ComboBoxItem(
                            value: m,
                            child: Text(_themeModeLabel(m)),
                          ),
                        )
                        .toList(),
                    onChanged: (value) {
                      if (value == null) return;
                      ref.read(settingsProvider.notifier).setThemeMode(value);
                    },
                  ),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 16),
        _SectionHeader(title: 'Security'),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                InfoLabel(
                  label: 'Require PIN on launch',
                  child: ToggleSwitch(
                    checked: settings.requirePinOnLaunch,
                    onChanged: (value) => ref
                        .read(settingsProvider.notifier)
                        .setRequirePinOnLaunch(value),
                  ),
                ),
                const SizedBox(height: 12),
                FilledButton(
                  onPressed: isConnected
                      ? () => showWindowsChangePasswordDialog(context)
                      : null,
                  child: Text(
                    isConnected
                        ? 'Change YubiKey PIN'
                        : 'Connect YubiKey to change PIN',
                  ),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 16),
        _SectionHeader(title: 'App Preferences'),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                InfoLabel(
                  label: 'Clear clipboard after',
                  child: ComboBox<int>(
                    value: settings.clipboardTimeoutSeconds,
                    items: const [10, 20, 30, 60, 120]
                        .map(
                          (seconds) => ComboBoxItem(
                            value: seconds,
                            child: Text('$seconds seconds'),
                          ),
                        )
                        .toList(),
                    onChanged: (value) {
                      if (value == null) return;
                      ref
                          .read(settingsProvider.notifier)
                          .setClipboardTimeout(value);
                    },
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }

  static String _themeModeLabel(ThemeMode mode) {
    switch (mode) {
      case ThemeMode.system:
        return 'System';
      case ThemeMode.light:
        return 'Light';
      case ThemeMode.dark:
        return 'Dark';
    }
  }
}

class _SectionHeader extends StatelessWidget {
  final String title;

  const _SectionHeader({required this.title});

  @override
  Widget build(BuildContext context) {
    final theme = FluentTheme.of(context);
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Text(
        title.toUpperCase(),
        style: theme.typography.caption?.copyWith(
          fontWeight: FontWeight.w600,
          color: theme.accentColor,
          letterSpacing: 0.6,
        ),
      ),
    );
  }
}
