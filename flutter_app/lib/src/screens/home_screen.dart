import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_symbols_icons/symbols.dart';

import '../ffi/gosh_event.dart';
import '../providers/connection_provider.dart' as conn;
import '../providers/providers.dart';
import '../theme/colors.dart';
import '../widgets/credential_card.dart';
import 'key_info_screen.dart';
import 'settings_screen.dart';

/// Home screen showing the list of credentials with bottom navigation.
class HomeScreen extends ConsumerStatefulWidget {
  const HomeScreen({super.key});

  @override
  ConsumerState<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends ConsumerState<HomeScreen> {
  Timer? _timer;
  String _searchQuery = '';
  int _currentIndex = 0;

  @override
  void initState() {
    super.initState();
    // Update every 100ms for smooth countdown animation
    _timer = Timer.periodic(const Duration(milliseconds: 100), (_) {
      if (mounted) setState(() {});
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final connectionState = ref.watch(connectionProvider);
    final isConnected = connectionState.status == conn.ConnectionState.connected;

    return Scaffold(
      body: IndexedStack(
        index: _currentIndex,
        children: [
          _buildCredentialsTab(),
          const KeyInfoScreen(),
          const SettingsScreen(),
        ],
      ),
      bottomNavigationBar: _buildBottomNav(context),
    );
  }

  Widget _buildBottomNav(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Container(
      decoration: BoxDecoration(
        color: isDark ? AppColors.surfaceDark : AppColors.surfaceLight,
        border: Border(
          top: BorderSide(
            color:
                isDark ? Colors.white.withValues(alpha: 0.05) : Colors.grey.shade200,
          ),
        ),
      ),
      child: SafeArea(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceAround,
            children: [
              _buildNavItem(
                context,
                index: 0,
                icon: Symbols.key,
                label: 'Credentials',
                isDark: isDark,
              ),
              _buildNavItem(
                context,
                index: 1,
                icon: Symbols.info,
                label: 'Key Info',
                isDark: isDark,
              ),
              _buildNavItem(
                context,
                index: 2,
                icon: Symbols.settings,
                label: 'Settings',
                isDark: isDark,
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildNavItem(
    BuildContext context, {
    required int index,
    required IconData icon,
    required String label,
    required bool isDark,
  }) {
    final isSelected = _currentIndex == index;

    return GestureDetector(
      onTap: () => setState(() => _currentIndex = index),
      behavior: HitTestBehavior.opaque,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 8),
        decoration: BoxDecoration(
          color: isSelected
              ? AppColors.primary.withValues(alpha: isDark ? 0.2 : 0.1)
              : Colors.transparent,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              icon,
              size: 24,
              color: isSelected ? AppColors.primary : AppColors.textSecondary,
            ),
            const SizedBox(height: 4),
            Text(
              label,
              style: Theme.of(context).textTheme.labelSmall?.copyWith(
                    color:
                        isSelected ? AppColors.primary : AppColors.textSecondary,
                    fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                  ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildCredentialsTab() {
    final connectionState = ref.watch(connectionProvider);
    final credentialsState = ref.watch(credentialsProvider);
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return CustomScrollView(
      slivers: [
        // Header Section
        SliverToBoxAdapter(
          child: _buildHeader(context, connectionState, isDark),
        ),

        // Search Bar
        SliverToBoxAdapter(child: _buildSearchBar(context, isDark)),

        // Content
        if (connectionState.status == conn.ConnectionState.disconnected)
          _buildNoDevice(context)
        else if (credentialsState.credentials.isEmpty)
          _buildNoCredentials(context)
        else
          _buildCredentialList(context, credentialsState.credentials),

        // Bottom padding for nav bar
        const SliverToBoxAdapter(child: SizedBox(height: 80)),
      ],
    );
  }

  Widget _buildHeader(
      BuildContext context, conn.ConnectionNotifierState connection, bool isDark) {
    final isConnected = connection.status == conn.ConnectionState.connected;

    return Container(
      padding: const EdgeInsets.fromLTRB(16, 48, 16, 16),
      decoration: BoxDecoration(
        color: isDark
            ? AppColors.backgroundDark.withValues(alpha: 0.95)
            : AppColors.backgroundLight.withValues(alpha: 0.95),
        border: Border(
          bottom: BorderSide(
            color:
                isDark ? Colors.white.withValues(alpha: 0.05) : Colors.grey.shade200,
          ),
        ),
      ),
      child: Row(
        children: [
          // USB Icon
          Icon(
            Symbols.usb,
            size: 28,
            color: isConnected ? AppColors.primary : AppColors.textSecondary,
          ),
          const SizedBox(width: 8),

          // Add button
          if (isConnected)
            GestureDetector(
              onTap: () => Navigator.of(context).pushNamed('/add'),
              child: Container(
                width: 32,
                height: 32,
                decoration: BoxDecoration(
                  color: AppColors.primary,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: const Icon(
                  Symbols.add,
                  size: 20,
                  color: Colors.white,
                ),
              ),
            ),
          const SizedBox(width: 12),

          // Center: Status and Device Name
          Expanded(
            child: Column(
              children: [
                Text(
                  isConnected
                      ? 'CONNECTED'
                      : connection.status == conn.ConnectionState.connecting
                          ? 'CONNECTING...'
                          : 'NO YUBIKEY',
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                        color: AppColors.textSecondary,
                        fontWeight: FontWeight.bold,
                        letterSpacing: 1.5,
                      ),
                ),
                const SizedBox(height: 2),
                Text(
                  connection.yubiKeyInfo?.deviceName ?? 'Authenticator',
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                        letterSpacing: -0.5,
                      ),
                ),
              ],
            ),
          ),

          // Connection status indicator
          if (isConnected)
            Container(
              width: 10,
              height: 10,
              decoration: BoxDecoration(
                color: AppColors.success,
                shape: BoxShape.circle,
                boxShadow: [
                  BoxShadow(
                    color: AppColors.success.withValues(alpha: 0.5),
                    blurRadius: 8,
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildSearchBar(BuildContext context, bool isDark) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Container(
        decoration: BoxDecoration(
          color: isDark ? AppColors.surfaceDark : AppColors.surfaceLight,
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: isDark
                ? Colors.white.withValues(alpha: 0.05)
                : Colors.grey.shade200,
          ),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withValues(alpha: 0.05),
              blurRadius: 8,
              offset: const Offset(0, 2),
            ),
          ],
        ),
        child: TextField(
          onChanged: (value) => setState(() => _searchQuery = value),
          decoration: InputDecoration(
            hintText: 'Search accounts...',
            hintStyle: TextStyle(color: AppColors.textSecondary),
            prefixIcon: Icon(
              Symbols.search,
              color: AppColors.textSecondary,
            ),
            filled: false,
            border: InputBorder.none,
            contentPadding:
                const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
          ),
        ),
      ),
    );
  }

  Widget _buildNoDevice(BuildContext context) {
    return SliverFillRemaining(
      hasScrollBody: false,
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Symbols.usb_off,
              size: 64,
              color: AppColors.textSecondary,
            ),
            const SizedBox(height: 16),
            Text(
              'No YubiKey Connected',
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            const SizedBox(height: 8),
            Text(
              'Insert your YubiKey to view credentials',
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: AppColors.textSecondary,
                  ),
            ),
            const SizedBox(height: 24),
            ElevatedButton.icon(
              onPressed: () => ref.read(connectionProvider.notifier).reconnect(),
              icon: Icon(Symbols.refresh),
              label: const Text('Retry'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildNoCredentials(BuildContext context) {
    return SliverFillRemaining(
      hasScrollBody: false,
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Symbols.vpn_key_off,
              size: 64,
              color: AppColors.textSecondary,
            ),
            const SizedBox(height: 16),
            Text(
              'No Credentials',
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            const SizedBox(height: 8),
            Text(
              'Add your first credential to get started',
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: AppColors.textSecondary,
                  ),
            ),
            const SizedBox(height: 24),
            ElevatedButton.icon(
              onPressed: () => Navigator.of(context).pushNamed('/add'),
              icon: Icon(Symbols.add),
              label: const Text('Add Credential'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildCredentialList(
      BuildContext context, List<Credential> credentials) {
    // Filter by search query
    final filteredCredentials = _searchQuery.isEmpty
        ? credentials
        : credentials.where((c) {
            final query = _searchQuery.toLowerCase();
            return c.displayName.toLowerCase().contains(query) ||
                c.account.toLowerCase().contains(query);
          }).toList();

    return SliverPadding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      sliver: SliverList(
        delegate: SliverChildBuilderDelegate(
          (context, index) {
            final credential = filteredCredentials[index];
            final progress = _calculateProgress(credential);

            return Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: CredentialCard(
                credential: credential,
                progress: progress,
                onCalculate: () =>
                    ref.read(credentialsProvider.notifier).calculate(credential.id),
                onDelete: () => _confirmDelete(context, credential),
              ),
            );
          },
          childCount: filteredCredentials.length,
        ),
      ),
    );
  }

  double _calculateProgress(Credential credential) {
    if (!credential.isTotp) return 1.0;

    final period = credential.period > 0 ? credential.period : 30;
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final elapsed = now % period;
    return (period - elapsed) / period;
  }

  void _confirmDelete(BuildContext context, Credential credential) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete Credential'),
        content: Text(
            'Are you sure you want to delete ${credential.displayName}? This cannot be undone.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () {
              ref.read(credentialsProvider.notifier).delete(credential.id);
              Navigator.of(context).pop();
            },
            style: TextButton.styleFrom(foregroundColor: AppColors.error),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
  }
}
