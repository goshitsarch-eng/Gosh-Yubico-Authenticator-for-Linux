import 'package:fluent_ui/fluent_ui.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../ffi/gosh_event.dart';
import '../../providers/providers.dart';

class WindowsCredentialTile extends ConsumerWidget {
  final Credential credential;
  final double progress;
  final VoidCallback onCalculate;
  final VoidCallback onDelete;

  const WindowsCredentialTile({
    super.key,
    required this.credential,
    required this.progress,
    required this.onCalculate,
    required this.onDelete,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final settings = ref.watch(settingsProvider);
    final copyToClipboard = ref.read(copyToClipboardProvider);

    return Card(
      padding: const EdgeInsets.all(14),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      credential.issuer?.isNotEmpty == true
                          ? credential.issuer!
                          : credential.account,
                      style: FluentTheme.of(context).typography.subtitle,
                      overflow: TextOverflow.ellipsis,
                    ),
                    if (credential.issuer?.isNotEmpty == true)
                      Text(
                        credential.account,
                        style: FluentTheme.of(context).typography.caption,
                        overflow: TextOverflow.ellipsis,
                      ),
                  ],
                ),
              ),
              if (credential.touchRequired)
                const Padding(
                  padding: EdgeInsets.only(left: 8),
                  child: Icon(FluentIcons.touch, size: 14),
                ),
            ],
          ),
          const SizedBox(height: 10),
          _CodeRow(
            code: credential.code,
            digits: credential.digits,
            progress: progress,
          ),
          const SizedBox(height: 12),
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              Button(
                onPressed: credential.code == null
                    ? null
                    : () {
                        copyToClipboard(credential.code!);
                        displayInfoBar(
                          context,
                          builder: (context, close) => InfoBar(
                            title: const Text('Copied'),
                            content: Text(
                              'Code copied (clears in ${settings.clipboardTimeoutSeconds}s)',
                            ),
                            severity: InfoBarSeverity.success,
                            action: IconButton(
                              icon: const Icon(FluentIcons.clear),
                              onPressed: close,
                            ),
                          ),
                        );
                      },
                child: const Text('Copy'),
              ),
              const SizedBox(width: 8),
              FilledButton(
                onPressed: onCalculate,
                child: Text(credential.isTotp ? 'Refresh' : 'Calculate'),
              ),
              const SizedBox(width: 8),
              Button(
                onPressed: () => _confirmDelete(context),
                child: const Text('Delete'),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Future<void> _confirmDelete(BuildContext context) async {
    final result = await showDialog<bool>(
      context: context,
      builder: (context) => ContentDialog(
        title: const Text('Delete Credential'),
        content: Text(
          'Delete ${credential.displayName}? This cannot be undone.',
        ),
        actions: [
          Button(
            child: const Text('Cancel'),
            onPressed: () => Navigator.pop(context, false),
          ),
          FilledButton(
            child: const Text('Delete'),
            onPressed: () => Navigator.pop(context, true),
          ),
        ],
      ),
    );

    if (result == true) onDelete();
  }
}

class _CodeRow extends StatelessWidget {
  final String? code;
  final int digits;
  final double progress;

  const _CodeRow({
    required this.code,
    required this.digits,
    required this.progress,
  });

  @override
  Widget build(BuildContext context) {
    final theme = FluentTheme.of(context);
    final display =
        (code == null || code!.isEmpty) ? '—'.padLeft(digits, '—') : code!;

    return Row(
      children: [
        Expanded(
          child: Text(
            display,
            style: theme.typography.title?.copyWith(
              fontFamily: 'monospace',
              letterSpacing: 2,
            ),
          ),
        ),
        SizedBox(
          width: 44,
          child: ProgressRing(
            value: progress.clamp(0.0, 1.0),
            strokeWidth: 3,
          ),
        ),
      ],
    );
  }
}
