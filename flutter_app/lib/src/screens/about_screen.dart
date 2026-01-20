import 'package:flutter/material.dart';
import 'package:material_symbols_icons/symbols.dart';
import 'package:url_launcher/url_launcher.dart';

import '../theme/colors.dart';

/// About screen showing app information.
class AboutScreen extends StatelessWidget {
  const AboutScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back_ios_new),
          onPressed: () => Navigator.of(context).pop(),
        ),
        title: const Opacity(opacity: 0, child: Text('About')),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Column(
          children: [
            const SizedBox(height: 32),

            // Logo with glow
            _buildLogo(isDark),
            const SizedBox(height: 24),

            // App name and version
            Text(
              'Yubico Authenticator',
              style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            const SizedBox(height: 8),
            Text(
              'Version 1.0.0',
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: AppColors.textSecondary,
                  ),
            ),
            const SizedBox(height: 4),
            Text(
              'Author: Goshitsarch',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: isDark ? Colors.grey.shade500 : Colors.grey.shade600,
                  ),
            ),
            const SizedBox(height: 48),

            // Action list
            _buildActionList(context, isDark),
            const SizedBox(height: 48),

            // Footer
            _buildFooter(context, isDark),
          ],
        ),
      ),
    );
  }

  Widget _buildLogo(bool isDark) {
    return Stack(
      alignment: Alignment.center,
      children: [
        // Cyan glow effect
        Container(
          width: 140,
          height: 140,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            boxShadow: [
              BoxShadow(
                color: AppColors.primaryGlow.withValues(alpha: 0.4),
                blurRadius: 50,
                spreadRadius: 15,
              ),
            ],
          ),
        ),
        // Logo container
        Container(
          width: 96,
          height: 96,
          decoration: BoxDecoration(
            color: isDark ? AppColors.surfaceDark : AppColors.surfaceLight,
            borderRadius: BorderRadius.circular(24),
            boxShadow: [
              BoxShadow(
                color: Colors.black.withValues(alpha: 0.15),
                blurRadius: 24,
                offset: const Offset(0, 8),
              ),
            ],
            border: Border.all(
              color: isDark
                  ? Colors.white.withValues(alpha: 0.1)
                  : Colors.black.withValues(alpha: 0.05),
              width: 1.5,
            ),
          ),
          child: Icon(
            Symbols.security_key,
            size: 48,
            color: AppColors.primaryGlow,
          ),
        ),
      ],
    );
  }

  Widget _buildActionList(BuildContext context, bool isDark) {
    return Container(
      decoration: BoxDecoration(
        color: isDark ? AppColors.surfaceDark : AppColors.surfaceLight,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(
          color: isDark
              ? Colors.white.withValues(alpha: 0.05)
              : Colors.grey.shade200,
        ),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.08),
            blurRadius: 12,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Column(
        children: [
          _buildActionTile(
            context,
            isDark: isDark,
            icon: Symbols.description,
            title: 'Terms of Service',
            onTap: () => _launchUrl('https://example.com/terms'),
          ),
          Divider(
            height: 1,
            indent: 60,
            color: isDark
                ? Colors.white.withValues(alpha: 0.05)
                : Colors.grey.shade200,
          ),
          _buildActionTile(
            context,
            isDark: isDark,
            icon: Symbols.verified_user,
            title: 'Privacy Policy',
            onTap: () => _launchUrl('https://example.com/privacy'),
          ),
          Divider(
            height: 1,
            indent: 60,
            color: isDark
                ? Colors.white.withValues(alpha: 0.05)
                : Colors.grey.shade200,
          ),
          _buildActionTile(
            context,
            isDark: isDark,
            icon: Symbols.gavel,
            title: 'Licenses',
            onTap: () => showLicensePage(
              context: context,
              applicationName: 'Yubico Authenticator',
              applicationVersion: '1.0.0',
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildActionTile(
    BuildContext context, {
    required bool isDark,
    required IconData icon,
    required String title,
    required VoidCallback onTap,
  }) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
          child: Row(
            children: [
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: AppColors.primaryGlow.withValues(alpha: isDark ? 0.15 : 0.1),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(
                  icon,
                  size: 20,
                  color: AppColors.primaryGlow,
                ),
              ),
              const SizedBox(width: 14),
              Expanded(
                child: Text(
                  title,
                  style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                        fontWeight: FontWeight.w500,
                      ),
                ),
              ),
              Icon(
                Symbols.chevron_right,
                color: isDark ? Colors.grey.shade500 : Colors.grey.shade400,
                size: 22,
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildFooter(BuildContext context, bool isDark) {
    return Column(
      children: [
        // Gradient divider
        Container(
          width: double.infinity,
          height: 1,
          decoration: BoxDecoration(
            gradient: LinearGradient(
              colors: [
                Colors.transparent,
                AppColors.primaryGlow.withValues(alpha: 0.3),
                Colors.transparent,
              ],
            ),
          ),
        ),
        const SizedBox(height: 20),
        // Report issue link
        InkWell(
          onTap: () => _launchUrl(
              'https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux/issues'),
          borderRadius: BorderRadius.circular(8),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  Symbols.bug_report,
                  size: 18,
                  color: AppColors.primaryGlow,
                ),
                const SizedBox(width: 8),
                Text(
                  'Report an Issue',
                  style: Theme.of(context).textTheme.labelMedium?.copyWith(
                        color: AppColors.textSecondary,
                      ),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 20),
        // Tagline
        Text(
          'SECURE HARDWARE, SECURE LOGIN',
          style: Theme.of(context).textTheme.labelSmall?.copyWith(
                color: isDark ? Colors.grey.shade600 : Colors.grey.shade400,
                fontWeight: FontWeight.bold,
                letterSpacing: 2,
                fontSize: 10,
              ),
        ),
      ],
    );
  }

  Future<void> _launchUrl(String url) async {
    final uri = Uri.parse(url);
    if (await canLaunchUrl(uri)) {
      await launchUrl(uri);
    }
  }
}
