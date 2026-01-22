import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/providers.dart';
import '../theme/colors.dart';

/// PIN entry dialog for YubiKey authentication.
class PinEntryDialog extends ConsumerStatefulWidget {
  const PinEntryDialog({super.key});

  @override
  ConsumerState<PinEntryDialog> createState() => _PinEntryDialogState();
}

class _PinEntryDialogState extends ConsumerState<PinEntryDialog> {
  final _pinController = TextEditingController();
  bool _showPin = false;

  @override
  void dispose() {
    _pinController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final authState = ref.watch(authProvider);
    final connection = ref.watch(connectionProvider);
    final deviceName = connection.yubiKeyInfo?.deviceName ?? 'YubiKey';
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Stack(
      children: [
        // Blurred background
        BackdropFilter(
          filter: ImageFilter.blur(sigmaX: 6, sigmaY: 6),
          child: Container(
            color: Colors.black.withValues(alpha: 0.5),
          ),
        ),

        // Dialog
        Center(
          child: Container(
            margin: const EdgeInsets.all(24),
            constraints: const BoxConstraints(maxWidth: 400),
            decoration: BoxDecoration(
              color: isDark ? AppColors.modalDark : Colors.white,
              borderRadius: BorderRadius.circular(16),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withValues(alpha: 0.3),
                  blurRadius: 30,
                  offset: const Offset(0, 10),
                ),
              ],
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                // Top highlight
                Container(
                  height: 4,
                  decoration: BoxDecoration(
                    gradient: LinearGradient(
                      colors: [
                        Colors.transparent,
                        AppColors.primary.withValues(alpha: 0.5),
                        Colors.transparent,
                      ],
                    ),
                    borderRadius: const BorderRadius.only(
                      topLeft: Radius.circular(16),
                      topRight: Radius.circular(16),
                    ),
                  ),
                ),

                Padding(
                  padding: const EdgeInsets.all(32),
                  child: Column(
                    children: [
                      // Lock icon
                      _buildIcon(isDark),
                      const SizedBox(height: 20),

                      // Title
                      Text(
                        'Unlock YubiKey',
                        style:
                            Theme.of(context).textTheme.headlineSmall?.copyWith(
                                  fontWeight: FontWeight.bold,
                                ),
                      ),
                      const SizedBox(height: 8),

                      // Description
                      Text(
                        'Please enter the PIN for your $deviceName to access credentials.',
                        style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                              color: AppColors.textSecondary,
                            ),
                        textAlign: TextAlign.center,
                      ),
                      const SizedBox(height: 32),

                      // PIN input
                      _buildPinInput(isDark),

                      // Error message
                      if (authState.status == AuthState.failed) ...[
                        const SizedBox(height: 12),
                        Row(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            Icon(
                              Icons.error,
                              size: 14,
                              color: AppColors.error,
                            ),
                            const SizedBox(width: 4),
                            Text(
                              authState.errorMessage ?? 'Incorrect PIN',
                              style: Theme.of(context)
                                  .textTheme
                                  .labelSmall
                                  ?.copyWith(
                                    color: AppColors.error,
                                    fontWeight: FontWeight.bold,
                                  ),
                            ),
                          ],
                        ),
                      ],
                      const SizedBox(height: 24),

                      // Unlock button
                      ElevatedButton.icon(
                        onPressed: authState.status == AuthState.authenticating
                            ? null
                            : _submit,
                        icon: authState.status == AuthState.authenticating
                            ? const SizedBox(
                                width: 20,
                                height: 20,
                                child: CircularProgressIndicator(
                                  strokeWidth: 2,
                                  color: Colors.white,
                                ),
                              )
                            : const Icon(Icons.key),
                        label: Text(
                          authState.status == AuthState.authenticating
                              ? 'Unlocking...'
                              : 'Unlock',
                        ),
                        style: ElevatedButton.styleFrom(
                          minimumSize: const Size.fromHeight(48),
                        ),
                      ),
                      const SizedBox(height: 16),

                      // Cancel button
                      TextButton(
                        onPressed: () =>
                            ref.read(authProvider.notifier).cancel(),
                        child: Text(
                          'Cancel',
                          style:
                              Theme.of(context).textTheme.labelLarge?.copyWith(
                                    color: AppColors.textSecondary,
                                  ),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildIcon(bool isDark) {
    return Stack(
      alignment: Alignment.center,
      children: [
        // Glow
        Container(
          width: 80,
          height: 80,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            boxShadow: [
              BoxShadow(
                color: AppColors.primary.withValues(alpha: 0.3),
                blurRadius: 30,
              ),
            ],
          ),
        ),
        // Icon container
        Container(
          width: 64,
          height: 64,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: isDark ? const Color(0xFF363636) : AppColors.backgroundLight,
            border: Border.all(
              color: isDark ? const Color(0xFF444444) : Colors.grey.shade200,
            ),
          ),
          child: const Icon(
            Icons.lock,
            size: 32,
            color: AppColors.primary,
          ),
        ),
      ],
    );
  }

  Widget _buildPinInput(bool isDark) {
    return Container(
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(12),
        boxShadow: [
          BoxShadow(
            color: AppColors.primary.withValues(alpha: 0.1),
            blurRadius: 10,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      child: TextField(
        controller: _pinController,
        obscureText: !_showPin,
        autofocus: true,
        textAlign: TextAlign.center,
        style: const TextStyle(
          fontSize: 18,
          letterSpacing: 2,
        ),
        decoration: InputDecoration(
          hintText: 'Enter PIN',
          filled: true,
          fillColor: isDark ? const Color(0xFF222222) : Colors.grey.shade50,
          border: OutlineInputBorder(
            borderRadius: BorderRadius.circular(12),
            borderSide: BorderSide(
              color: isDark ? const Color(0xFF444444) : Colors.grey.shade300,
            ),
          ),
          enabledBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(12),
            borderSide: BorderSide(
              color: isDark ? const Color(0xFF444444) : Colors.grey.shade300,
            ),
          ),
          focusedBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(12),
            borderSide: const BorderSide(
              color: AppColors.primary,
              width: 2,
            ),
          ),
          suffixIcon: IconButton(
            icon: Icon(
              _showPin ? Icons.visibility_off : Icons.visibility,
              color: AppColors.textSecondary,
            ),
            onPressed: () => setState(() => _showPin = !_showPin),
          ),
        ),
        onSubmitted: (_) => _submit(),
      ),
    );
  }

  void _submit() {
    final pin = _pinController.text;
    if (pin.isNotEmpty) {
      ref.read(authProvider.notifier).authenticate(pin);
    }
  }
}
