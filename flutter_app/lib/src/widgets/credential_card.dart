import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_symbols_icons/symbols.dart';

import '../ffi/gosh_event.dart';
import '../providers/providers.dart';
import '../theme/colors.dart';
import 'circular_countdown.dart';

/// Card widget displaying a single credential.
class CredentialCard extends ConsumerWidget {
  final Credential credential;
  final double progress;
  final VoidCallback? onTap;
  final VoidCallback? onCalculate;
  final VoidCallback? onDelete;

  const CredentialCard({
    super.key,
    required this.credential,
    required this.progress,
    this.onTap,
    this.onCalculate,
    this.onDelete,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final needsTouch = credential.touchRequired && credential.code == null;

    return Stack(
      children: [
        // Main card
        Container(
          decoration: BoxDecoration(
            color: isDark ? AppColors.surfaceDark : AppColors.surfaceLight,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: isDark
                  ? Colors.white.withValues(alpha: 0.05)
                  : Colors.grey.shade100,
            ),
            boxShadow: [
              BoxShadow(
                color: Colors.black.withValues(alpha: 0.08),
                blurRadius: 8,
                offset: const Offset(0, 2),
              ),
            ],
          ),
          child: Material(
            color: Colors.transparent,
            child: InkWell(
              onTap: () => _copyCode(context, ref),
              borderRadius: BorderRadius.circular(12),
              child: Padding(
                padding: EdgeInsets.only(
                  left: needsTouch ? 20 : 16,
                  right: 16,
                  top: 16,
                  bottom: 16,
                ),
                child: Row(
                  children: [
                    // Icon
                    _buildIcon(isDark),
                    const SizedBox(width: 16),

                    // Text content
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            credential.issuer ?? credential.account,
                            style:
                                Theme.of(context).textTheme.titleSmall?.copyWith(
                                      fontWeight: FontWeight.w600,
                                    ),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                          if (credential.issuer != null) ...[
                            const SizedBox(height: 2),
                            Text(
                              credential.account,
                              style: Theme.of(context)
                                  .textTheme
                                  .bodySmall
                                  ?.copyWith(
                                    color: AppColors.textSecondary,
                                  ),
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                            ),
                          ],
                          if (needsTouch) ...[
                            const SizedBox(height: 6),
                            _buildTouchBadge(context),
                          ],
                        ],
                      ),
                    ),
                    const SizedBox(width: 16),

                    // Code and timer
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.end,
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        _buildCode(context, isDark),
                        const SizedBox(height: 4),
                        _buildTimer(context, isDark),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),

        // Amber left strip for touch-required credentials
        if (needsTouch)
          Positioned(
            left: 0,
            top: 0,
            bottom: 0,
            child: Container(
              width: 4,
              decoration: BoxDecoration(
                color: AppColors.warning,
                borderRadius: const BorderRadius.only(
                  topLeft: Radius.circular(12),
                  bottomLeft: Radius.circular(12),
                ),
              ),
            ),
          ),
      ],
    );
  }

  Widget _buildIcon(bool isDark) {
    // Get first letter of issuer or account for icon
    final letter =
        (credential.issuer ?? credential.account).substring(0, 1).toUpperCase();

    return Container(
      width: 48,
      height: 48,
      decoration: BoxDecoration(
        color: isDark ? Colors.white : AppColors.surfaceLight,
        borderRadius: BorderRadius.circular(10),
        border: Border.all(
          color: isDark
              ? Colors.white.withValues(alpha: 0.1)
              : Colors.grey.shade200,
        ),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.05),
            blurRadius: 4,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      child: Center(
        child: Text(
          letter,
          style: TextStyle(
            fontSize: 20,
            fontWeight: FontWeight.bold,
            color: AppColors.primary,
          ),
        ),
      ),
    );
  }

  Widget _buildTouchBadge(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(
          Symbols.touch_app,
          size: 12,
          color: AppColors.warning,
        ),
        const SizedBox(width: 4),
        Text(
          'TOUCH REQUIRED',
          style: Theme.of(context).textTheme.labelSmall?.copyWith(
                color: AppColors.warning,
                fontWeight: FontWeight.bold,
                letterSpacing: 0.5,
                fontSize: 10,
              ),
        ),
      ],
    );
  }

  Widget _buildCode(BuildContext context, bool isDark) {
    if (credential.touchRequired && credential.code == null) {
      // Show pulsing dots for touch-required credentials without code
      return _buildPulsingDots(isDark);
    }

    final code = credential.code ?? '------';
    final isExpiring = progress <= 0.25;

    return Text(
      _formatCode(code),
      style: TextStyle(
        fontFamily: 'monospace',
        fontSize: 22,
        fontWeight: FontWeight.bold,
        letterSpacing: 2,
        color: isExpiring ? AppColors.warning : AppColors.primary,
      ),
    );
  }

  Widget _buildPulsingDots(bool isDark) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: List.generate(6, (index) {
        // Add space in middle (after 3 dots)
        final marginLeft = index == 3 ? 8.0 : 3.0;
        return AnimatedContainer(
          duration: Duration(milliseconds: 300 + (index * 75)),
          curve: Curves.easeInOut,
          margin: EdgeInsets.only(left: index == 0 ? 0 : marginLeft),
          width: 8,
          height: 8,
          decoration: BoxDecoration(
            color: isDark ? Colors.grey.shade600 : Colors.grey.shade300,
            shape: BoxShape.circle,
          ),
        );
      }),
    );
  }

  Widget _buildTimer(BuildContext context, bool isDark) {
    if (credential.touchRequired && credential.code == null) {
      // Show calculate/refresh button for touch-required credentials
      return GestureDetector(
        onTap: onCalculate,
        child: Container(
          width: 32,
          height: 32,
          decoration: BoxDecoration(
            color: AppColors.warning.withValues(alpha: 0.15),
            borderRadius: BorderRadius.circular(16),
          ),
          child: Icon(
            Symbols.touch_app,
            size: 18,
            color: AppColors.warning,
          ),
        ),
      );
    }

    if (!credential.isTotp) {
      // HOTP doesn't have a timer
      return Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
        decoration: BoxDecoration(
          color: AppColors.textSecondary.withValues(alpha: 0.1),
          borderRadius: BorderRadius.circular(4),
        ),
        child: Text(
          'HOTP',
          style: Theme.of(context).textTheme.labelSmall?.copyWith(
                color: AppColors.textSecondary,
                fontWeight: FontWeight.w500,
              ),
        ),
      );
    }

    return CircularCountdown(progress: progress, size: 28, strokeWidth: 3);
  }

  String _formatCode(String code) {
    if (code.length >= 6) {
      return '${code.substring(0, 3)} ${code.substring(3)}';
    }
    return code;
  }

  void _copyCode(BuildContext context, WidgetRef ref) {
    if (credential.code == null) {
      // Trigger calculate for touch-required credentials
      onCalculate?.call();
      return;
    }

    final copyToClipboard = ref.read(copyToClipboardProvider);
    final settings = ref.read(settingsProvider);
    copyToClipboard(credential.code!);

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(
          'Code copied (clears in ${settings.clipboardTimeoutSeconds}s)',
        ),
        duration: const Duration(seconds: 2),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }
}
