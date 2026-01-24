import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_symbols_icons/symbols.dart';

import '../providers/connection_provider.dart' as conn;
import '../providers/providers.dart';
import '../widgets/change_password_dialog.dart';

/// Settings screen.
class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final settings = ref.watch(settingsProvider);
    final connection = ref.watch(connectionProvider);
    final colorScheme = Theme.of(context).colorScheme;

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        // General section
        _buildSectionHeader(context, 'General'),
        const SizedBox(height: 12),
        _buildSettingsCard(
          context,
          children: [
            _buildSettingsTile(
              context,
              icon: Symbols.palette,
              title: 'Theme',
              subtitle: 'App appearance',
              trailing: Text(
                _themeModeLabel(settings.themeMode),
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      color: colorScheme.onSurfaceVariant,
                    ),
              ),
              onTap: () => _showThemeDialog(context, ref, settings.themeMode),
            ),
          ],
        ),
        const SizedBox(height: 24),

        // Security section
        _buildSectionHeader(context, 'Security'),
        const SizedBox(height: 12),
        _buildSettingsCard(
          context,
          children: [
            _buildSettingsTile(
              context,
              icon: Symbols.key,
              title: 'Change YubiKey PIN',
              subtitle: connection.status == conn.ConnectionState.connected
                  ? null
                  : 'Connect YubiKey first',
              onTap: connection.status == conn.ConnectionState.connected
                  ? () => showChangePasswordDialog(context)
                  : null,
            ),
            const Divider(height: 1),
            _buildSettingsTile(
              context,
              icon: Symbols.lock,
              title: 'Require PIN',
              subtitle: 'On application launch',
              trailing: Switch(
                value: settings.requirePinOnLaunch,
                onChanged: (value) {
                  ref
                      .read(settingsProvider.notifier)
                      .setRequirePinOnLaunch(value);
                },
              ),
            ),
          ],
        ),
        const SizedBox(height: 24),

        // Preferences section
        _buildSectionHeader(context, 'App Preferences'),
        const SizedBox(height: 12),
        _buildSettingsCard(
          context,
          children: [
            _buildSettingsTile(
              context,
              icon: Symbols.content_paste_off,
              title: 'Clear Clipboard',
              subtitle: 'Sensitive data timeout',
              trailing: Text(
                '${settings.clipboardTimeoutSeconds}s',
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      color: colorScheme.onSurfaceVariant,
                    ),
              ),
              onTap: () => _showClipboardDialog(
                  context, ref, settings.clipboardTimeoutSeconds),
            ),
          ],
        ),
        const SizedBox(height: 48),

        // Footer
        _buildFooter(context, connection),
      ],
    );
  }

  Widget _buildSectionHeader(BuildContext context, String title) {
    final colorScheme = Theme.of(context).colorScheme;
    return Padding(
      padding: const EdgeInsets.fromLTRB(8, 8, 8, 8),
      child: Text(
        title.toUpperCase(),
        style: Theme.of(context).textTheme.labelLarge?.copyWith(
              color: colorScheme.primary,
              fontWeight: FontWeight.bold,
              letterSpacing: 1.2,
              fontSize: 13,
            ),
      ),
    );
  }

  Widget _buildSettingsCard(BuildContext context,
      {required List<Widget> children}) {
    return Card(
      margin: EdgeInsets.zero,
      child: Column(children: children),
    );
  }

  Widget _buildSettingsTile(
    BuildContext context, {
    required IconData icon,
    required String title,
    String? subtitle,
    Widget? trailing,
    VoidCallback? onTap,
  }) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final colorScheme = Theme.of(context).colorScheme;

    return ListTile(
      leading: Container(
        width: 44,
        height: 44,
        decoration: BoxDecoration(
          color: colorScheme.primary.withValues(alpha: isDark ? 0.15 : 0.08),
          borderRadius: BorderRadius.circular(14),
        ),
        child: Icon(
          icon,
          color: colorScheme.primary,
          size: 24,
        ),
      ),
      title: Text(
        title,
        style: Theme.of(context).textTheme.titleSmall?.copyWith(
              fontWeight: FontWeight.w600,
            ),
      ),
      subtitle: subtitle != null
          ? Text(
              subtitle,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: colorScheme.onSurfaceVariant,
                  ),
            )
          : null,
      trailing: trailing ??
          Icon(
            Symbols.chevron_right,
            color: colorScheme.onSurfaceVariant,
          ),
      onTap: onTap,
    );
  }

  Widget _buildFooter(
      BuildContext context, conn.ConnectionNotifierState connection) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final colorScheme = Theme.of(context).colorScheme;

    return Column(
      children: [
        Container(
          width: 48,
          height: 48,
          decoration: BoxDecoration(
            gradient: LinearGradient(
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
              colors: [colorScheme.primary, colorScheme.secondary],
            ),
            borderRadius: BorderRadius.circular(12),
            boxShadow: [
              BoxShadow(
                color: colorScheme.primary.withValues(alpha: 0.3),
                blurRadius: 15,
              ),
            ],
          ),
          child: Icon(
            Symbols.security_key,
            color: Colors.white,
            size: 28,
          ),
        ),
        const SizedBox(height: 16),
        Text(
          'Gosh Yubikey Manager',
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: colorScheme.onSurfaceVariant,
              ),
        ),
        const SizedBox(height: 4),
        Text(
          'Version 1.0.0',
          style: Theme.of(context).textTheme.labelSmall?.copyWith(
                color: isDark ? Colors.grey.shade700 : Colors.grey.shade400,
              ),
        ),
        const SizedBox(height: 24),
        if (connection.status == conn.ConnectionState.connected) ...[
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            decoration: BoxDecoration(
              color: isDark
                  ? Colors.white.withValues(alpha: 0.1)
                  : colorScheme.primary.withValues(alpha: 0.9),
              borderRadius: BorderRadius.circular(20),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  width: 8,
                  height: 8,
                  decoration: BoxDecoration(
                    color: colorScheme.primary,
                    shape: BoxShape.circle,
                    boxShadow: [
                      BoxShadow(
                        color: colorScheme.primary.withValues(alpha: 0.5),
                        blurRadius: 8,
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: 8),
                Text(
                  '${connection.yubiKeyInfo?.deviceName ?? "YubiKey"} Connected',
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                        color: Colors.white,
                        fontWeight: FontWeight.w600,
                      ),
                ),
              ],
            ),
          ),
        ],
      ],
    );
  }

  String _themeModeLabel(ThemeMode mode) {
    switch (mode) {
      case ThemeMode.system:
        return 'System';
      case ThemeMode.light:
        return 'Light';
      case ThemeMode.dark:
        return 'Dark';
    }
  }

  void _showThemeDialog(
      BuildContext context, WidgetRef ref, ThemeMode current) {
    showDialog(
      context: context,
      builder: (dialogContext) => RadioGroup<ThemeMode>(
        groupValue: current,
        onChanged: (value) {
          if (value == null) return;
          ref.read(settingsProvider.notifier).setThemeMode(value);
          Navigator.of(dialogContext).pop();
        },
        child: SimpleDialog(
          title: const Text('Theme'),
          children: ThemeMode.values
              .map((mode) => RadioListTile<ThemeMode>(
                    value: mode,
                    title: Text(_themeModeLabel(mode)),
                  ))
              .toList(),
        ),
      ),
    );
  }

  void _showClipboardDialog(BuildContext context, WidgetRef ref, int current) {
    final options = [10, 20, 30, 60, 120];

    showDialog(
      context: context,
      builder: (dialogContext) => RadioGroup<int>(
        groupValue: current,
        onChanged: (value) {
          if (value == null) return;
          ref.read(settingsProvider.notifier).setClipboardTimeout(value);
          Navigator.of(dialogContext).pop();
        },
        child: SimpleDialog(
          title: const Text('Clear Clipboard After'),
          children: options
              .map((seconds) => RadioListTile<int>(
                    value: seconds,
                    title: Text('$seconds seconds'),
                  ))
              .toList(),
        ),
      ),
    );
  }
}
