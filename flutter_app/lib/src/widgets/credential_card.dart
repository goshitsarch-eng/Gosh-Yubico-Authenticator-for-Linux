import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_symbols_icons/symbols.dart';

import '../ffi/gosh_event.dart';
import '../providers/providers.dart';
import '../services/icon_preferences_service.dart';
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
    final colorScheme = Theme.of(context).colorScheme;
    final needsTouch =
        widget.credential.touchRequired && widget.credential.code == null;

    return Card(
      clipBehavior: Clip.antiAlias,
      shape: needsTouch
          ? RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(16),
              side: BorderSide(
                  color: colorScheme.tertiary.withValues(alpha: 0.55),
                  width: 2),
            )
          : null, // Fallback to theme default
      child: InkWell(
        onTap: () => _copyCode(context),
        onLongPress: () => _showIconPicker(context),
        onSecondaryTapDown: (details) => _showContextMenu(context, details),
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Row(
            children: [
              // Icon
              _buildIcon(isDark),
              const SizedBox(width: 20),

              // Text content
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      widget.credential.issuer ?? widget.credential.account,
                      style: Theme.of(context).textTheme.titleMedium?.copyWith(
                            fontWeight: FontWeight.w700,
                          ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    if (widget.credential.issuer != null) ...[
                      const SizedBox(height: 4),
                      Text(
                        widget.credential.account,
                        style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                              color: colorScheme.onSurfaceVariant,
                            ),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ],
                    if (needsTouch) ...[
                      const SizedBox(height: 8),
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
                  const SizedBox(height: 6),
                  if (_showCopied)
                    Text(
                      'COPIED',
                      style: TextStyle(
                        color: colorScheme.primary,
                        fontSize: 11,
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
    );
  }

  Widget _buildIcon(bool isDark) {
    final colorScheme = Theme.of(context).colorScheme;
    final boxDecoration = BoxDecoration(
      color: colorScheme.surface,
      borderRadius: BorderRadius.circular(10),
      border: Border.all(
        color:
            isDark ? Colors.white.withValues(alpha: 0.1) : Colors.grey.shade200,
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
              color: Theme.of(context).colorScheme.primary,
            ),
          ),
        ),
      ),
      error: (_, __) => _buildLetterAvatar(boxDecoration),
    );
  }

  Widget _buildFaviconContainer(
      Uint8List imageData, BoxDecoration boxDecoration) {
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
    final letter = (widget.credential.issuer ?? widget.credential.account)
        .substring(0, 1)
        .toUpperCase();

    return Center(
      child: Text(
        letter,
        style: TextStyle(
          fontSize: 20,
          fontWeight: FontWeight.bold,
          color: Theme.of(context).colorScheme.primary,
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
          color: Theme.of(context).colorScheme.tertiary,
        ),
        const SizedBox(width: 4),
        Text(
          'TOUCH REQUIRED',
          style: Theme.of(context).textTheme.labelSmall?.copyWith(
                color: Theme.of(context).colorScheme.tertiary,
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
        color: isExpiring
            ? Theme.of(context).colorScheme.tertiary
            : Theme.of(context).colorScheme.primary,
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
            color:
                Theme.of(context).colorScheme.tertiary.withValues(alpha: 0.18),
            borderRadius: BorderRadius.circular(16),
          ),
          child: Icon(
            Symbols.touch_app,
            size: 18,
            color: Theme.of(context).colorScheme.tertiary,
          ),
        ),
      );
    }

    if (!widget.credential.isTotp) {
      // HOTP doesn't have a timer
      return Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
        decoration: BoxDecoration(
          color: Theme.of(context)
              .colorScheme
              .onSurfaceVariant
              .withValues(alpha: 0.12),
          borderRadius: BorderRadius.circular(4),
        ),
        child: Text(
          'HOTP',
          style: Theme.of(context).textTheme.labelSmall?.copyWith(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
                fontWeight: FontWeight.w500,
              ),
        ),
      );
    }

    return CircularCountdown(
        progress: widget.progress, size: 28, strokeWidth: 3);
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
    final credentialName =
        widget.credential.issuer ?? widget.credential.account;

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

  Future<void> _showContextMenu(
    BuildContext context,
    TapDownDetails details,
  ) async {
    final overlay = Overlay.of(context).context.findRenderObject() as RenderBox;
    final position = RelativeRect.fromRect(
      Rect.fromPoints(details.globalPosition, details.globalPosition),
      Offset.zero & overlay.size,
    );

    final hasCode = widget.credential.code != null;
    final needsCalculate = widget.credential.touchRequired && !hasCode;

    final selected = await showMenu<String>(
      context: context,
      position: position,
      items: [
        PopupMenuItem<String>(
          value: 'copy',
          enabled: hasCode,
          child: const Text('Copy code'),
        ),
        PopupMenuItem<String>(
          value: 'calculate',
          enabled: needsCalculate && widget.onCalculate != null,
          child: const Text('Calculate'),
        ),
        const PopupMenuDivider(),
        const PopupMenuItem<String>(
          value: 'icon',
          child: Text('Choose icon...'),
        ),
        PopupMenuItem<String>(
          value: 'delete',
          enabled: widget.onDelete != null,
          child: const Text('Delete'),
        ),
      ],
    );

    if (!context.mounted || selected == null) return;

    switch (selected) {
      case 'copy':
        _copyCode(context);
        break;
      case 'calculate':
        widget.onCalculate?.call();
        break;
      case 'icon':
        await _showIconPicker(context);
        break;
      case 'delete':
        widget.onDelete?.call();
        break;
    }
  }
}
