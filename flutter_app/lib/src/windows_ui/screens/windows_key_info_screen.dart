import 'package:fluent_ui/fluent_ui.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../providers/connection_provider.dart' as conn;
import '../../providers/credentials_provider.dart';
import '../../providers/providers.dart'
    show connectionProvider, credentialsProvider;

class WindowsKeyInfoScreen extends ConsumerWidget {
  const WindowsKeyInfoScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final connection = ref.watch(connectionProvider);
    final credentials = ref.watch(credentialsProvider);
    final isConnected = connection.status == conn.ConnectionState.connected;

    if (!isConnected) {
      return Center(
        child: InfoBar(
          title: const Text('No YubiKey Connected'),
          content:
              const Text('Insert your YubiKey to view device information.'),
          severity: InfoBarSeverity.warning,
          action: Button(
            onPressed: () => ref.read(connectionProvider.notifier).reconnect(),
            child: const Text('Retry'),
          ),
        ),
      );
    }

    final info = connection.yubiKeyInfo;
    final version = info?.version;

    return ScaffoldPage.scrollable(
      header: PageHeader(
        title: const Text('Key Info'),
        commandBar: Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            Icon(
              FluentIcons.plug_connected,
              color: FluentTheme.of(context).accentColor,
              size: 18,
            ),
            const SizedBox(width: 8),
            Text(info?.deviceName ?? 'YubiKey'),
          ],
        ),
      ),
      children: [
        InfoLabel(
          label: 'Device',
          child: Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    info?.deviceName ?? 'YubiKey',
                    style: FluentTheme.of(context).typography.subtitle,
                  ),
                  const SizedBox(height: 6),
                  Text(
                    version != null
                        ? 'Firmware ${version.major}.${version.minor}.${version.patch}'
                        : 'Firmware unknown',
                    style: FluentTheme.of(context).typography.body,
                  ),
                ],
              ),
            ),
          ),
        ),
        const SizedBox(height: 16),
        InfoLabel(
          label: 'Credentials',
          child: Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: _CredentialStats(credentials: credentials),
            ),
          ),
        ),
      ],
    );
  }
}

class _CredentialStats extends StatelessWidget {
  final CredentialsState credentials;

  const _CredentialStats({required this.credentials});

  @override
  Widget build(BuildContext context) {
    final all = credentials.credentials;
    final totp = all.where((c) => c.isTotp).length;
    final hotp = all.where((c) => !c.isTotp).length;

    return Row(
      children: [
        Expanded(
            child: _StatTile(label: 'Total', value: all.length.toString())),
        const SizedBox(width: 12),
        Expanded(child: _StatTile(label: 'TOTP', value: totp.toString())),
        const SizedBox(width: 12),
        Expanded(child: _StatTile(label: 'HOTP', value: hotp.toString())),
      ],
    );
  }
}

class _StatTile extends StatelessWidget {
  final String label;
  final String value;

  const _StatTile({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    final theme = FluentTheme.of(context);
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: theme.resources.controlFillColorDefault,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: theme.resources.controlStrokeColorDefault),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(value, style: theme.typography.subtitle),
          const SizedBox(height: 4),
          Text(label, style: theme.typography.caption),
        ],
      ),
    );
  }
}
