import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_symbols_icons/symbols.dart';

import '../ffi/gosh_event.dart';
import '../models/icon_preference.dart';
import '../providers/favicon_provider.dart';
import '../providers/icon_preferences_provider.dart';
import '../providers/providers.dart';
import '../services/icon_preferences_service.dart';
import '../theme/colors.dart';
import '../utils/service_icons.dart';
import 'circular_countdown.dart';
import 'icon_picker_dialog.dart';

/// Card widget displaying a single credential.
class CredentialCard extends ConsumerStatefulWidget {
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
  ConsumerState<CredentialCard> createState() => _CredentialCardState();
}

class _CredentialCardState extends ConsumerState<CredentialCard> {
  bool _showCopied = false;

  /// Get a stable credential ID for storing preferences.
  String get _credentialId =>
      IconPreferencesService.credentialIdFromBytes(widget.credential.id);

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final needsTouch = widget.credential.touchRequired && widget.credential.code == null;

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
              onTap: () => _copyCode(context),
              onLongPress: () => _showIconPicker(context),
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
                            widget.credential.issuer ?? widget.credential.account,
                            style:
                                Theme.of(context).textTheme.titleSmall?.copyWith(
                                      fontWeight: FontWeight.w600,
                                    ),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                          if (widget.credential.issuer != null) ...[
                            const SizedBox(height: 2),
                            Text(
                              widget.credential.account,
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
                        if (_showCopied)
                          Text(
                            'COPIED',
                            style: TextStyle(
                              color: AppColors.primaryGlow,
                              fontSize: 10,
                              fontWeight: FontWeight.bold,
                              letterSpacing: 0.5,
                            ),
                          )
                        else
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
    final boxDecoration = BoxDecoration(
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
    );

    // 1. Check for custom icon preference first
    final iconPreference = ref.watch(iconPreferenceProvider(_credentialId));
    if (iconPreference != null && iconPreference.hasCustomIcon) {
      // Custom service icon
      if (iconPreference.customIconKey != null) {
        final customIcon = getServiceIconByKey(iconPreference.customIconKey!);
        if (customIcon != null) {
          return Container(
            width: 48,
            height: 48,
            decoration: boxDecoration,
            child: Center(
              child: Icon(
                customIcon.icon,
                size: 28,
                color: customIcon.color,
              ),
            ),
          );
        }
      }

      // Custom favicon domain
      if (iconPreference.faviconDomain != null) {
        return _buildFaviconIcon(
          iconPreference.faviconDomain!,
          boxDecoration,
          isDark,
        );
      }
    }

    // 2. Check for service icon from keyword matching
    final serviceIcon = getServiceIcon(
      widget.credential.issuer,
      widget.credential.account,
    );
    if (serviceIcon != null) {
      return Container(
        width: 48,
        height: 48,
        decoration: boxDecoration,
        child: Center(
          child: Icon(
            serviceIcon.icon,
            size: 28,
            color: serviceIcon.color,
          ),
        ),
      );
    }

    // 3. Try to fetch favicon from domain
    final extractDomain = ref.watch(domainExtractorProvider);
    final domain = extractDomain(
      widget.credential.issuer,
      widget.credential.account,
    );

    if (domain != null) {
      return _buildFaviconIcon(domain, boxDecoration, isDark);
    }

    // 4. Fallback: first letter avatar
    return _buildLetterAvatar(boxDecoration);
  }

  Widget _buildFaviconIcon(
    String domain,
    BoxDecoration boxDecoration,
    bool isDark,
  ) {
    final faviconAsync = ref.watch(faviconProvider(domain));

    return faviconAsync.when(
      data: (imageData) {
        if (imageData != null) {
          return _buildFaviconContainer(imageData, boxDecoration);
        }
        return _buildLetterAvatar(boxDecoration);
      },
      loading: () => Container(
        width: 48,
        height: 48,
        decoration: boxDecoration,
        child: Center(
          child: SizedBox(
            width: 20,
            height: 20,
            child: CircularProgressIndicator(
              strokeWidth: 2,
              color: AppColors.primary,
            ),
          ),
        ),
      ),
      error: (_, __) => _buildLetterAvatar(boxDecoration),
    );
  }

  Widget _buildFaviconContainer(Uint8List imageData, BoxDecoration boxDecoration) {
    return Container(
      width: 48,
      height: 48,
      decoration: boxDecoration,
      child: ClipRRect(
        borderRadius: BorderRadius.circular(9),
        child: Image.memory(
          imageData,
          width: 32,
          height: 32,
          fit: BoxFit.contain,
          errorBuilder: (context, error, stackTrace) {
            return _buildLetterAvatarContent();
          },
        ),
      ),
    );
  }

  Widget _buildLetterAvatar(BoxDecoration boxDecoration) {
    return Container(
      width: 48,
      height: 48,
      decoration: boxDecoration,
      child: _buildLetterAvatarContent(),
    );
  }

  Widget _buildLetterAvatarContent() {
    final letter =
        (widget.credential.issuer ?? widget.credential.account).substring(0, 1).toUpperCase();

    return Center(
      child: Text(
        letter,
        style: TextStyle(
          fontSize: 20,
          fontWeight: FontWeight.bold,
          color: AppColors.primary,
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
    if (widget.credential.touchRequired && widget.credential.code == null) {
      // Show pulsing dots for touch-required credentials without code
      return _buildPulsingDots(isDark);
    }

    final code = widget.credential.code ?? '------';
    final isExpiring = widget.progress <= 0.25;

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
    if (widget.credential.touchRequired && widget.credential.code == null) {
      // Show calculate/refresh button for touch-required credentials
      return GestureDetector(
        onTap: widget.onCalculate,
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

    if (!widget.credential.isTotp) {
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

    return CircularCountdown(progress: widget.progress, size: 28, strokeWidth: 3);
  }

  String _formatCode(String code) {
    if (code.length >= 6) {
      return '${code.substring(0, 3)} ${code.substring(3)}';
    }
    return code;
  }

  void _copyCode(BuildContext context) {
    if (widget.credential.code == null) {
      // Trigger calculate for touch-required credentials
      widget.onCalculate?.call();
      return;
    }

    final copyToClipboard = ref.read(copyToClipboardProvider);
    final settings = ref.read(settingsProvider);
    copyToClipboard(widget.credential.code!);

    setState(() => _showCopied = true);
    Future.delayed(const Duration(seconds: 2), () {
      if (mounted) setState(() => _showCopied = false);
    });

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

  Future<void> _showIconPicker(BuildContext context) async {
    final currentPreference = ref.read(iconPreferenceProvider(_credentialId));
    final credentialName = widget.credential.issuer ?? widget.credential.account;

    final result = await showIconPickerDialog(
      context,
      credentialName: credentialName,
      currentPreference: currentPreference,
    );

    if (result != null) {
      final notifier = ref.read(iconPreferencesProvider.notifier);
      await notifier.setPreference(_credentialId, result);
    }
  }
}
