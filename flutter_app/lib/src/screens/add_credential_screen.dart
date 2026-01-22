import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_symbols_icons/symbols.dart';

import '../ffi/gosh_event.dart';
import '../providers/providers.dart';
import '../theme/colors.dart';
import '../utils/qr_scanner.dart';

/// Screen for adding a new credential.
class AddCredentialScreen extends ConsumerStatefulWidget {
  const AddCredentialScreen({super.key});

  @override
  ConsumerState<AddCredentialScreen> createState() =>
      _AddCredentialScreenState();
}

class _AddCredentialScreenState extends ConsumerState<AddCredentialScreen> {
  final _formKey = GlobalKey<FormState>();
  final _issuerController = TextEditingController();
  final _accountController = TextEditingController();
  final _secretController = TextEditingController();

  OathType _oathType = OathType.totp;
  Algorithm _algorithm = Algorithm.sha1;
  int _digits = 6;
  bool _requireTouch = false;
  bool _showSecret = false;
  bool _isSubmitting = false;

  @override
  void dispose() {
    _issuerController.dispose();
    _accountController.dispose();
    _secretController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final isConnected = ref.watch(isConnectedProvider);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: Icon(Symbols.arrow_back),
          onPressed: () => Navigator.of(context).pop(),
        ),
        title: const Text('Add Credential'),
        actions: [
          Padding(
            padding: const EdgeInsets.only(right: 16),
            child: Icon(
              Symbols.usb,
              color: isConnected ? AppColors.primary : AppColors.textSecondary,
            ),
          ),
        ],
      ),
      body: Form(
        key: _formKey,
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            // Type selector
            _buildTypeSelector(isDark),
            const SizedBox(height: 24),

            // Identity card
            _buildIdentityCard(context, isDark),
            const SizedBox(height: 16),

            // Security card
            _buildSecurityCard(context, isDark),
            const SizedBox(height: 16),

            // Info banner
            _buildInfoBanner(context),
            const SizedBox(height: 100), // Space for button
          ],
        ),
      ),
      bottomSheet: _buildSubmitButton(context),
    );
  }

  Widget _buildTypeSelector(bool isDark) {
    return Container(
      padding: const EdgeInsets.all(4),
      decoration: BoxDecoration(
        color: isDark ? AppColors.surfaceDark : Colors.grey.shade200,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        children: [
          Expanded(
            child: _buildTypeButton('TOTP', OathType.totp, isDark),
          ),
          Expanded(
            child: _buildTypeButton('HOTP', OathType.hotp, isDark),
          ),
        ],
      ),
    );
  }

  Widget _buildTypeButton(String label, OathType type, bool isDark) {
    final isSelected = _oathType == type;

    return GestureDetector(
      onTap: () => setState(() => _oathType = type),
      child: Container(
        padding: const EdgeInsets.symmetric(vertical: 12),
        decoration: BoxDecoration(
          color: isSelected
              ? (isDark ? const Color(0xFF3D3D3D) : Colors.white)
              : Colors.transparent,
          borderRadius: BorderRadius.circular(8),
          boxShadow: isSelected
              ? [
                  BoxShadow(
                    color: Colors.black.withValues(alpha: 0.1),
                    blurRadius: 4,
                    offset: const Offset(0, 2),
                  ),
                ]
              : null,
        ),
        child: Center(
          child: Text(
            label,
            style: TextStyle(
              fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
              color: isSelected
                  ? (isDark ? Colors.white : Colors.black)
                  : AppColors.textSecondary,
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildIdentityCard(BuildContext context, bool isDark) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Issuer field
            _buildLabel(context, 'Issuer'),
            const SizedBox(height: 8),
            TextFormField(
              controller: _issuerController,
              decoration: InputDecoration(
                hintText: 'e.g. Google, AWS',
                prefixIcon: Icon(Symbols.grid_view, color: AppColors.textSecondary),
              ),
            ),
            const SizedBox(height: 20),

            // Account field
            _buildLabel(context, 'Account Name'),
            const SizedBox(height: 8),
            TextFormField(
              controller: _accountController,
              decoration: InputDecoration(
                hintText: 'user@example.com',
                prefixIcon: Icon(Symbols.person, color: AppColors.textSecondary),
              ),
              validator: (value) {
                if (value == null || value.isEmpty) {
                  return 'Account name is required';
                }
                return null;
              },
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSecurityCard(BuildContext context, bool isDark) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Secret key field
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                _buildLabel(context, 'Secret Key'),
                TextButton.icon(
                  onPressed: _scanQrCode,
                  icon: Icon(Symbols.qr_code_scanner, size: 16),
                  label: const Text('Scan QR'),
                  style: TextButton.styleFrom(
                    foregroundColor: AppColors.primary,
                    textStyle: const TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            TextFormField(
              controller: _secretController,
              obscureText: !_showSecret,
              style: const TextStyle(
                fontFamily: 'monospace',
                letterSpacing: 1.5,
              ),
              decoration: InputDecoration(
                hintText: 'JBSWY3DPEHPK3PXP',
                prefixIcon: Icon(Symbols.key, color: AppColors.textSecondary),
                suffixIcon: IconButton(
                  icon: Icon(
                    _showSecret ? Symbols.visibility_off : Symbols.visibility,
                    color: AppColors.textSecondary,
                  ),
                  onPressed: () => setState(() => _showSecret = !_showSecret),
                ),
              ),
              validator: (value) {
                if (value == null || value.isEmpty) {
                  return 'Secret key is required';
                }
                // Basic Base32 validation
                final cleaned = value.replaceAll(' ', '').toUpperCase();
                if (!RegExp(r'^[A-Z2-7]+=*$').hasMatch(cleaned)) {
                  return 'Invalid Base32 secret';
                }
                return null;
              },
            ),
            const SizedBox(height: 4),
            Text(
              'Base32 encoded key provided by the service. Spaces are ignored.',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: AppColors.textSecondary,
                  ),
            ),
            const SizedBox(height: 16),
            const Divider(),
            const SizedBox(height: 16),

            // Require touch toggle
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Require Touch',
                      style: Theme.of(context).textTheme.titleSmall?.copyWith(
                            fontWeight: FontWeight.bold,
                          ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      'Physical touch needed to generate code',
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            color: AppColors.textSecondary,
                          ),
                    ),
                  ],
                ),
                Switch(
                  value: _requireTouch,
                  onChanged: (value) => setState(() => _requireTouch = value),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildLabel(BuildContext context, String text) {
    return Text(
      text.toUpperCase(),
      style: Theme.of(context).textTheme.labelSmall?.copyWith(
            color: AppColors.textSecondary,
            fontWeight: FontWeight.bold,
            letterSpacing: 1,
          ),
    );
  }

  Widget _buildInfoBanner(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          colors: [
            AppColors.primaryGlow.withValues(alpha: 0.15),
            AppColors.primary.withValues(alpha: 0.08),
          ],
        ),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: AppColors.primary.withValues(alpha: 0.25),
        ),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            Symbols.info,
            size: 20,
            color: AppColors.primary,
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              'Credentials are stored securely on your YubiKey. Removing the app will not delete credentials from the hardware key.',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    height: 1.5,
                  ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSubmitButton(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [
            Theme.of(context).scaffoldBackgroundColor.withValues(alpha: 0),
            Theme.of(context).scaffoldBackgroundColor,
          ],
        ),
      ),
      child: SafeArea(
        child: ElevatedButton.icon(
          onPressed: _isSubmitting ? null : _submit,
          icon: _isSubmitting
              ? const SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(
                    strokeWidth: 2,
                    color: Colors.white,
                  ),
                )
              : Icon(Symbols.save),
          label: Text(_isSubmitting ? 'Saving...' : 'Save Credential'),
          style: ElevatedButton.styleFrom(
            minimumSize: const Size.fromHeight(56),
          ),
        ),
      ),
    );
  }

  Future<void> _scanQrCode() async {
    try {
      final credential = await QrScanner.scanFromFile();
      if (credential == null) {
        // User cancelled
        return;
      }

      // Fill in the form with scanned data
      setState(() {
        _oathType = credential.type;
        _algorithm = credential.algorithm;
        _digits = credential.digits;
        if (credential.issuer != null) {
          _issuerController.text = credential.issuer!;
        }
        _accountController.text = credential.account;
        _secretController.text = credential.secret;
      });

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('QR code scanned successfully'),
            backgroundColor: AppColors.success,
          ),
        );
      }
    } on QrScanException catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(e.message),
            backgroundColor: AppColors.error,
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to scan QR code: $e'),
            backgroundColor: AppColors.error,
          ),
        );
      }
    }
  }

  Future<void> _submit() async {
    if (!_formKey.currentState!.validate()) return;

    setState(() => _isSubmitting = true);

    final client = ref.read(goshClientProvider);
    final success = client.addCredential(
      issuer: _issuerController.text.isEmpty ? null : _issuerController.text,
      account: _accountController.text,
      secret: _secretController.text.replaceAll(' ', ''),
      oathType: _oathType,
      algorithm: _algorithm,
      digits: _digits,
      requireTouch: _requireTouch,
    );

    setState(() => _isSubmitting = false);

    if (success && mounted) {
      Navigator.of(context).pop();
    } else if (mounted) {
      final error = client.takeLastError() ?? 'Failed to add credential';
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(error), backgroundColor: AppColors.error),
      );
    }
  }
}
