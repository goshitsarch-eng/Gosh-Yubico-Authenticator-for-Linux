import 'dart:async';

import 'package:fluent_ui/fluent_ui.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../ffi/gosh_event.dart';
import '../../providers/connection_provider.dart' as conn;
import '../../providers/providers.dart';
import '../widgets/windows_credential_tile.dart';
import 'windows_add_credential_screen.dart';

class WindowsCredentialsScreen extends ConsumerStatefulWidget {
  const WindowsCredentialsScreen({super.key});

  static Future<void> openAddCredential(BuildContext context) {
    return Navigator.of(context).push(
      FluentPageRoute(
        builder: (context) => const WindowsAddCredentialScreen(),
      ),
    );
  }

  @override
  ConsumerState<WindowsCredentialsScreen> createState() =>
      _WindowsCredentialsScreenState();
}

class _WindowsCredentialsScreenState
    extends ConsumerState<WindowsCredentialsScreen> {
  Timer? _timer;
  String _search = '';

  @override
  void initState() {
    super.initState();
    _timer = Timer.periodic(const Duration(milliseconds: 250), (_) {
      if (mounted) setState(() {});
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final connectionState = ref.watch(connectionProvider);
    final credsState = ref.watch(credentialsProvider);

    final isConnected =
        connectionState.status == conn.ConnectionState.connected;

    final deviceName = connectionState.yubiKeyInfo?.deviceName ??
        (isConnected ? 'YubiKey' : 'No YubiKey');

    final filtered = _search.trim().isEmpty
        ? credsState.credentials
        : credsState.credentials.where((c) {
            final q = _search.toLowerCase();
            final display = (c.issuer ?? c.account).toLowerCase();
            return display.contains(q) || c.account.toLowerCase().contains(q);
          }).toList();

    return Mica(
      child: ScaffoldPage(
        header: PageHeader(
          title: Text(deviceName),
          commandBar: Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              if (isConnected)
                FilledButton(
                  onPressed: () =>
                      WindowsCredentialsScreen.openAddCredential(context),
                  child: const Row(
                    children: [
                      Icon(FluentIcons.add, size: 16),
                      SizedBox(width: 8),
                      Text('Add'),
                    ],
                  ),
                ),
              const SizedBox(width: 8),
              Button(
                onPressed: () =>
                    ref.read(credentialsProvider.notifier).refresh(),
                child: const Icon(FluentIcons.refresh),
              ),
            ],
          ),
        ),
        content: Column(
          children: [
            InfoLabel(
              label: 'Search',
              child: TextBox(
                placeholder: 'Search accounts...',
                prefix: const Icon(FluentIcons.search),
                onChanged: (value) => setState(() => _search = value),
              ),
            ),
            const SizedBox(height: 12),
            Expanded(
              child: _buildBody(context, isConnected, credsState, filtered),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildBody(
    BuildContext context,
    bool isConnected,
    CredentialsState credsState,
    List<Credential> filtered,
  ) {
    if (!isConnected) {
      return Center(
        child: InfoBar(
          title: const Text('No YubiKey Connected'),
          content: const Text('Insert your YubiKey to view credentials.'),
          severity: InfoBarSeverity.warning,
          action: Button(
            onPressed: () => ref.read(connectionProvider.notifier).reconnect(),
            child: const Text('Retry'),
          ),
        ),
      );
    }

    if (credsState.isLoading) {
      return const Center(child: ProgressRing());
    }

    if (credsState.error != null) {
      return Center(
        child: InfoBar(
          title: const Text('Error'),
          content: Text(credsState.error!),
          severity: InfoBarSeverity.error,
        ),
      );
    }

    if (filtered.isEmpty) {
      return Center(
        child: InfoBar(
          title: const Text('No Credentials'),
          content: const Text('Add your first credential to get started.'),
          severity: InfoBarSeverity.info,
          action: FilledButton(
            onPressed: () =>
                WindowsCredentialsScreen.openAddCredential(context),
            child: const Text('Add Credential'),
          ),
        ),
      );
    }

    return ListView.separated(
      itemCount: filtered.length,
      separatorBuilder: (_, __) => const SizedBox(height: 8),
      itemBuilder: (context, index) {
        final credential = filtered[index];
        final progress = _progress(credential);
        return WindowsCredentialTile(
          credential: credential,
          progress: progress,
          onCalculate: () =>
              ref.read(credentialsProvider.notifier).calculate(credential.id),
          onDelete: () =>
              ref.read(credentialsProvider.notifier).delete(credential.id),
        );
      },
    );
  }

  double _progress(Credential credential) {
    if (!credential.isTotp) return 1.0;
    final period = credential.period > 0 ? credential.period : 30;
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final elapsed = now % period;
    return (period - elapsed) / period;
  }
}
