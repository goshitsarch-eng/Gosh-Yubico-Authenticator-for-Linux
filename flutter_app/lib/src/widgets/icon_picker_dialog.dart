import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_symbols_icons/symbols.dart';

import '../models/icon_preference.dart';
import '../utils/service_icons.dart';

/// Dialog for selecting a custom icon for a credential.
class IconPickerDialog extends ConsumerStatefulWidget {
  final String credentialName;
  final IconPreference? currentPreference;

  const IconPickerDialog({
    super.key,
    required this.credentialName,
    this.currentPreference,
  });

  @override
  ConsumerState<IconPickerDialog> createState() => _IconPickerDialogState();
}

class _IconPickerDialogState extends ConsumerState<IconPickerDialog>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  final _domainController = TextEditingController();
  String? _selectedIconKey;
  String? _customDomain;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);

    // Initialize from current preference
    if (widget.currentPreference != null) {
      _selectedIconKey = widget.currentPreference!.customIconKey;
      _customDomain = widget.currentPreference!.faviconDomain;
      if (_customDomain != null) {
        _domainController.text = _customDomain!;
        _tabController.index = 1;
      }
    }
  }

  @override
  void dispose() {
    _tabController.dispose();
    _domainController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;

    return AlertDialog(
      title: Text(
        'Choose Icon',
        style: Theme.of(context).textTheme.titleLarge,
      ),
      content: SizedBox(
        width: 400,
        height: 400,
        child: Column(
          children: [
            Text(
              'for ${widget.credentialName}',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: colorScheme.onSurfaceVariant,
                  ),
            ),
            const SizedBox(height: 16),
            TabBar(
              controller: _tabController,
              tabs: const [
                Tab(text: 'Service Icons'),
                Tab(text: 'Custom URL'),
              ],
              labelColor: colorScheme.primary,
              unselectedLabelColor: colorScheme.onSurfaceVariant,
              indicatorColor: colorScheme.primary,
            ),
            const SizedBox(height: 16),
            Expanded(
              child: TabBarView(
                controller: _tabController,
                children: [
                  _buildServiceIconsGrid(isDark),
                  _buildCustomUrlInput(isDark),
                ],
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(null),
          child: const Text('Cancel'),
        ),
        TextButton(
          onPressed: () {
            // Reset to auto-detection
            Navigator.of(context).pop(IconPreference.auto());
          },
          child: Text(
            'Reset to Auto',
            style: TextStyle(color: colorScheme.onSurfaceVariant),
          ),
        ),
        FilledButton(
          onPressed: _canConfirm() ? _confirm : null,
          child: const Text('Confirm'),
        ),
      ],
    );
  }

  Widget _buildServiceIconsGrid(bool isDark) {
    final colorScheme = Theme.of(context).colorScheme;
    final iconKeys = getServiceIconKeys();

    return GridView.builder(
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 5,
        crossAxisSpacing: 8,
        mainAxisSpacing: 8,
      ),
      itemCount: iconKeys.length,
      itemBuilder: (context, index) {
        final key = iconKeys[index];
        final serviceIcon = getServiceIconByKey(key);
        final isSelected = _selectedIconKey == key;

        return Tooltip(
          message: getServiceDisplayName(key),
          child: InkWell(
            onTap: () {
              setState(() {
                _selectedIconKey = key;
                _customDomain = null;
                _domainController.clear();
              });
            },
            borderRadius: BorderRadius.circular(8),
            child: Container(
              decoration: BoxDecoration(
                color: isSelected
                    ? colorScheme.primary.withValues(alpha: 0.15)
                    : isDark
                        ? colorScheme.surfaceContainerHighest
                        : colorScheme.surfaceContainerHighest,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: isSelected
                      ? colorScheme.primary
                      : isDark
                          ? colorScheme.outline.withValues(alpha: 0.4)
                          : colorScheme.outline.withValues(alpha: 0.4),
                  width: isSelected ? 2 : 1,
                ),
              ),
              child: Center(
                child: serviceIcon != null
                    ? Icon(
                        serviceIcon.icon,
                        color: serviceIcon.color,
                        size: 24,
                      )
                    : Text(
                        key[0].toUpperCase(),
                        style: TextStyle(
                          color: colorScheme.primary,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
              ),
            ),
          ),
        );
      },
    );
  }

  Widget _buildCustomUrlInput(bool isDark) {
    final colorScheme = Theme.of(context).colorScheme;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Enter a domain to fetch its favicon:',
          style: Theme.of(context).textTheme.bodyMedium,
        ),
        const SizedBox(height: 12),
        TextField(
          controller: _domainController,
          decoration: InputDecoration(
            hintText: 'e.g., namecheap.com',
            prefixIcon: const Icon(Symbols.language),
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8),
            ),
            helperText: 'Favicons are fetched via Google\'s favicon service',
          ),
          onChanged: (value) {
            setState(() {
              _customDomain = value.trim().isNotEmpty ? value.trim() : null;
              if (_customDomain != null) {
                _selectedIconKey = null;
              }
            });
          },
        ),
        const SizedBox(height: 24),
        if (_customDomain != null && _customDomain!.isNotEmpty) ...[
          Text(
            'Preview:',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: colorScheme.onSurfaceVariant,
                ),
          ),
          const SizedBox(height: 8),
          Container(
            width: 64,
            height: 64,
            decoration: BoxDecoration(
              color: colorScheme.surface,
              borderRadius: BorderRadius.circular(12),
              border: Border.all(
                color: colorScheme.outline.withValues(alpha: 0.25),
              ),
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(11),
              child: Image.network(
                'https://www.google.com/s2/favicons?domain=$_customDomain&sz=64',
                width: 64,
                height: 64,
                fit: BoxFit.cover,
                errorBuilder: (context, error, stackTrace) {
                  return Center(
                    child: Icon(
                      Symbols.broken_image,
                      color: colorScheme.onSurfaceVariant,
                    ),
                  );
                },
                loadingBuilder: (context, child, loadingProgress) {
                  if (loadingProgress == null) return child;
                  return Center(
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      color: colorScheme.primary,
                    ),
                  );
                },
              ),
            ),
          ),
        ],
      ],
    );
  }

  bool _canConfirm() {
    return _selectedIconKey != null ||
        (_customDomain != null && _customDomain!.isNotEmpty);
  }

  void _confirm() {
    IconPreference pref;
    if (_selectedIconKey != null) {
      pref = IconPreference.withServiceIcon(_selectedIconKey!);
    } else if (_customDomain != null && _customDomain!.isNotEmpty) {
      pref = IconPreference.withFavicon(_customDomain!);
    } else {
      pref = IconPreference.auto();
    }
    Navigator.of(context).pop(pref);
  }
}

/// Shows the icon picker dialog and returns the selected preference.
/// Returns null if cancelled, or an IconPreference (possibly with no custom icon
/// if reset to auto).
Future<IconPreference?> showIconPickerDialog(
  BuildContext context, {
  required String credentialName,
  IconPreference? currentPreference,
}) {
  return showDialog<IconPreference>(
    context: context,
    builder: (context) => IconPickerDialog(
      credentialName: credentialName,
      currentPreference: currentPreference,
    ),
  );
}
