import 'dart:async';

import 'package:fluent_ui/fluent_ui.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../ffi/gosh_event.dart';
import '../../providers/providers.dart';

Future<bool?> showWindowsChangePasswordDialog(BuildContext context) {
  return showDialog<bool>(
    context: context,
    barrierDismissible: false,
    builder: (context) => const _WindowsChangePasswordDialog(),
  );
}

class _WindowsChangePasswordDialog extends ConsumerStatefulWidget {
  const _WindowsChangePasswordDialog();

  @override
  ConsumerState<_WindowsChangePasswordDialog> createState() =>
      _WindowsChangePasswordDialogState();
}

class _WindowsChangePasswordDialogState
    extends ConsumerState<_WindowsChangePasswordDialog> {
  final _formKey = GlobalKey<FormState>();
  final _newPasswordController = TextEditingController();
  final _confirmPasswordController = TextEditingController();

  bool _isSubmitting = false;
  bool _showPassword = false;
  bool _removePassword = false;
  StreamSubscription<GoshEvent>? _eventSubscription;
  String? _error;

  @override
  void initState() {
    super.initState();
    final client = ref.read(goshClientProvider);
    _eventSubscription = client.eventStream.listen((event) {
      if (!mounted) return;
      switch (event.type) {
        case GoshEventType.passwordChanged:
        case GoshEventType.passwordRemoved:
          setState(() {
            _isSubmitting = false;
            _error = null;
          });
          Navigator.of(context).pop(true);
          break;
        case GoshEventType.error:
          setState(() {
            _isSubmitting = false;
            _error = event.message ?? 'Failed to change password';
          });
          break;
        case GoshEventType.authRequired:
          setState(() {
            _isSubmitting = false;
            _error = 'Please authenticate first';
          });
          break;
        default:
          break;
      }
    });
  }

  @override
  void dispose() {
    _eventSubscription?.cancel();
    _newPasswordController.dispose();
    _confirmPasswordController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ContentDialog(
      title: const Text('Change Password'),
      content: Form(
        key: _formKey,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Set a new password for your YubiKey OATH credentials.',
              style: FluentTheme.of(context).typography.body,
            ),
            const SizedBox(height: 12),
            Checkbox(
              checked: _removePassword,
              onChanged: _isSubmitting
                  ? null
                  : (value) {
                      setState(() {
                        _removePassword = value ?? false;
                        _error = null;
                        if (_removePassword) {
                          _newPasswordController.clear();
                          _confirmPasswordController.clear();
                        }
                      });
                    },
              content: const Text('Remove password protection'),
            ),
            if (!_removePassword) ...[
              const SizedBox(height: 12),
              InfoLabel(
                label: 'New password',
                child: TextFormBox(
                  controller: _newPasswordController,
                  enabled: !_isSubmitting,
                  obscureText: !_showPassword,
                  suffix: IconButton(
                    icon: Icon(
                      _showPassword ? FluentIcons.hide3 : FluentIcons.red_eye,
                      size: 14,
                    ),
                    onPressed: _isSubmitting
                        ? null
                        : () => setState(() => _showPassword = !_showPassword),
                  ),
                  validator: (value) {
                    if (value == null || value.isEmpty) {
                      return 'Please enter a password';
                    }
                    if (value.length < 4) {
                      return 'Password must be at least 4 characters';
                    }
                    return null;
                  },
                ),
              ),
              const SizedBox(height: 12),
              InfoLabel(
                label: 'Confirm password',
                child: TextFormBox(
                  controller: _confirmPasswordController,
                  enabled: !_isSubmitting,
                  obscureText: !_showPassword,
                  validator: (value) {
                    if (value != _newPasswordController.text) {
                      return 'Passwords do not match';
                    }
                    return null;
                  },
                ),
              ),
            ],
            if (_error != null) ...[
              const SizedBox(height: 12),
              InfoBar(
                title: const Text('Error'),
                content: Text(_error!),
                severity: InfoBarSeverity.error,
                isLong: true,
              ),
            ],
          ],
        ),
      ),
      actions: [
        Button(
          onPressed: _isSubmitting ? null : () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: _isSubmitting ? null : _submit,
          child: Text(_removePassword ? 'Remove Password' : 'Set Password'),
        ),
      ],
    );
  }

  void _submit() {
    setState(() => _error = null);

    if (!_removePassword && !_formKey.currentState!.validate()) return;

    setState(() => _isSubmitting = true);

    final client = ref.read(goshClientProvider);
    final password = _removePassword ? '' : _newPasswordController.text;
    client.setPassword(password);
  }
}
