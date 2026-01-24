import 'package:fluent_ui/fluent_ui.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../providers/providers.dart';

class WindowsPinEntryDialog extends ConsumerStatefulWidget {
  const WindowsPinEntryDialog({super.key});

  @override
  ConsumerState<WindowsPinEntryDialog> createState() =>
      _WindowsPinEntryDialogState();
}

class _WindowsPinEntryDialogState extends ConsumerState<WindowsPinEntryDialog> {
  final _pinController = TextEditingController();
  bool _showPin = false;

  @override
  void dispose() {
    _pinController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final auth = ref.watch(authProvider);
    final connection = ref.watch(connectionProvider);
    final deviceName = connection.yubiKeyInfo?.deviceName ?? 'YubiKey';
    final isBusy = auth.status == AuthState.authenticating;

    return Positioned.fill(
      child: Container(
        color: Colors.black.withValues(alpha: 0.55),
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 420),
            child: Card(
              padding: const EdgeInsets.all(20),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Unlock YubiKey',
                    style: FluentTheme.of(context).typography.subtitle,
                  ),
                  const SizedBox(height: 6),
                  Text(
                    'Enter the PIN for your $deviceName to access credentials.',
                    style: FluentTheme.of(context).typography.body,
                  ),
                  const SizedBox(height: 14),
                  InfoLabel(
                    label: 'PIN',
                    child: TextBox(
                      controller: _pinController,
                      autofocus: true,
                      obscureText: !_showPin,
                      enabled: !isBusy,
                      placeholder: 'Enter PIN',
                      suffix: IconButton(
                        icon: Icon(
                          _showPin ? FluentIcons.hide3 : FluentIcons.red_eye,
                          size: 14,
                        ),
                        onPressed: isBusy
                            ? null
                            : () => setState(() => _showPin = !_showPin),
                      ),
                      onSubmitted: (_) => _submit(),
                    ),
                  ),
                  if (auth.status == AuthState.failed)
                    Padding(
                      padding: const EdgeInsets.only(top: 10),
                      child: InfoBar(
                        title: const Text('Authentication failed'),
                        content: Text(auth.errorMessage ?? 'Incorrect PIN'),
                        severity: InfoBarSeverity.error,
                        isLong: true,
                      ),
                    ),
                  const SizedBox(height: 14),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      Button(
                        onPressed: isBusy
                            ? null
                            : () => ref.read(authProvider.notifier).cancel(),
                        child: const Text('Cancel'),
                      ),
                      const SizedBox(width: 8),
                      FilledButton(
                        onPressed: isBusy ? null : _submit,
                        child: Text(isBusy ? 'Unlocking...' : 'Unlock'),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  void _submit() {
    final pin = _pinController.text;
    if (pin.trim().isEmpty) return;
    ref.read(authProvider.notifier).authenticate(pin);
  }
}
