import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../ffi/gosh_event.dart';
import '../providers/providers.dart';
import '../theme/colors.dart';

/// Dialog for changing or removing the YubiKey OATH password.
class ChangePasswordDialog extends ConsumerStatefulWidget {
  const ChangePasswordDialog({super.key});

  @override
  ConsumerState<ChangePasswordDialog> createState() =>
      _ChangePasswordDialogState();
}

class _ChangePasswordDialogState extends ConsumerState<ChangePasswordDialog> {
  final _formKey = GlobalKey<FormState>();
  final _newPasswordController = TextEditingController();
  final _confirmPasswordController = TextEditingController();

  bool _isSubmitting = false;
  bool _showPassword = false;
  bool _removePassword = false;
  StreamSubscription<GoshEvent>? _eventSubscription;

  @override
  void initState() {
    super.initState();
    _setupEventListener();
  }

  void _setupEventListener() {
    final client = ref.read(goshClientProvider);
    _eventSubscription = client.eventStream.listen((event) {
      if (!mounted) return;

      switch (event.type) {
        case GoshEventType.passwordChanged:
          setState(() => _isSubmitting = false);
          Navigator.of(context).pop(true);
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('Password changed successfully'),
              backgroundColor: AppColors.success,
            ),
          );
          break;
        case GoshEventType.passwordRemoved:
          setState(() => _isSubmitting = false);
          Navigator.of(context).pop(true);
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('Password protection removed'),
              backgroundColor: AppColors.success,
            ),
          );
          break;
        case GoshEventType.error:
          setState(() => _isSubmitting = false);
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text(event.message ?? 'Failed to change password'),
              backgroundColor: AppColors.error,
            ),
          );
          break;
        case GoshEventType.authRequired:
          setState(() => _isSubmitting = false);
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('Please authenticate first'),
              backgroundColor: AppColors.warning,
            ),
          );
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
    return AlertDialog(
      title: const Text('Change Password'),
      content: Form(
        key: _formKey,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Set a new password for your YubiKey OATH credentials.',
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      color: AppColors.textSecondary,
                    ),
              ),
              const SizedBox(height: 16),

              // Remove password option
              CheckboxListTile(
                value: _removePassword,
                onChanged: (value) {
                  setState(() {
                    _removePassword = value ?? false;
                    if (_removePassword) {
                      _newPasswordController.clear();
                      _confirmPasswordController.clear();
                    }
                  });
                },
                title: const Text('Remove password protection'),
                controlAffinity: ListTileControlAffinity.leading,
                contentPadding: EdgeInsets.zero,
              ),
              const SizedBox(height: 16),

              // New password field
              if (!_removePassword) ...[
                TextFormField(
                  controller: _newPasswordController,
                  obscureText: !_showPassword,
                  enabled: !_isSubmitting,
                  decoration: InputDecoration(
                    labelText: 'New Password',
                    prefixIcon: const Icon(Icons.lock),
                    suffixIcon: IconButton(
                      icon: Icon(
                        _showPassword ? Icons.visibility_off : Icons.visibility,
                      ),
                      onPressed: () =>
                          setState(() => _showPassword = !_showPassword),
                    ),
                  ),
                  validator: (value) {
                    if (!_removePassword && (value == null || value.isEmpty)) {
                      return 'Please enter a password';
                    }
                    if (value != null && value.length < 4) {
                      return 'Password must be at least 4 characters';
                    }
                    return null;
                  },
                ),
                const SizedBox(height: 16),

                // Confirm password field
                TextFormField(
                  controller: _confirmPasswordController,
                  obscureText: !_showPassword,
                  enabled: !_isSubmitting,
                  decoration: const InputDecoration(
                    labelText: 'Confirm Password',
                    prefixIcon: Icon(Icons.lock_outline),
                  ),
                  validator: (value) {
                    if (!_removePassword) {
                      if (value != _newPasswordController.text) {
                        return 'Passwords do not match';
                      }
                    }
                    return null;
                  },
                ),
              ],

              if (_removePassword) ...[
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: AppColors.warning.withValues(alpha: 0.1),
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(
                      color: AppColors.warning.withValues(alpha: 0.3),
                    ),
                  ),
                  child: Row(
                    children: [
                      Icon(
                        Icons.warning_amber,
                        color: AppColors.warning,
                        size: 20,
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Text(
                          'This will allow anyone with physical access to view your credentials.',
                          style: Theme.of(context).textTheme.bodySmall?.copyWith(
                                color: AppColors.warning,
                              ),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: _isSubmitting ? null : () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        ElevatedButton(
          onPressed: _isSubmitting ? null : _submit,
          child: _isSubmitting
              ? const SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : Text(_removePassword ? 'Remove Password' : 'Set Password'),
        ),
      ],
    );
  }

  void _submit() {
    if (!_formKey.currentState!.validate()) return;

    setState(() => _isSubmitting = true);

    final client = ref.read(goshClientProvider);
    final password = _removePassword ? '' : _newPasswordController.text;
    client.setPassword(password);
  }
}

/// Show the change password dialog.
Future<bool?> showChangePasswordDialog(BuildContext context) {
  return showDialog<bool>(
    context: context,
    barrierDismissible: false,
    builder: (context) => const ChangePasswordDialog(),
  );
}
