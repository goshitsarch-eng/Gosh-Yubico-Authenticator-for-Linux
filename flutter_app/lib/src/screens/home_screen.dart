import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_symbols_icons/symbols.dart';
import 'package:yaru/yaru.dart';

import '../ffi/gosh_event.dart';
import '../providers/connection_provider.dart' as conn;
import '../providers/providers.dart';
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

  static const double _desktopNavBreakpoint = 760;

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
    final isLinux = !kIsWeb && Platform.isLinux;
    final connectionState = ref.watch(connectionProvider);
    final isConnected =
        connectionState.status == conn.ConnectionState.connected;

    final titles = ['Credentials', 'Key Info', 'Settings'];

    return Scaffold(
      appBar: isLinux
          ? YaruWindowTitleBar(
              title: Text(titles[_currentIndex]),
              centerTitle: true,
              actions: _currentIndex == 0 && isConnected
                  ? [
                      IconButton(
                        tooltip: 'Add Account',
                        icon: const Icon(Symbols.add),
                        onPressed: () =>
                            Navigator.of(context).pushNamed('/add'),
                      ),
                      const SizedBox(width: 8),
                    ]
                  : null,
            )
          : AppBar(
              automaticallyImplyLeading: false,
              title: Text(titles[_currentIndex]),
            ),
      body: LayoutBuilder(
        builder: (context, constraints) {
          final useRail = constraints.maxWidth >= _desktopNavBreakpoint;

          final content = IndexedStack(
            index: _currentIndex,
            children: [
              _buildCredentialsTab(),
              const KeyInfoScreen(),
              const SettingsScreen(),
            ],
          );

          if (!useRail) return content;

          return Row(
            children: [
              NavigationRail(
                selectedIndex: _currentIndex,
                onDestinationSelected: (index) =>
                    setState(() => _currentIndex = index),
                labelType: NavigationRailLabelType.all,
                leading: const SizedBox(height: 8),
                useIndicator: true,
                destinations: const [
                  NavigationRailDestination(
                    icon: Icon(Symbols.key),
                    label: Text('Credentials'),
                  ),
                  NavigationRailDestination(
                    icon: Icon(Symbols.info),
                    label: Text('Key Info'),
                  ),
                  NavigationRailDestination(
                    icon: Icon(Symbols.settings),
                    label: Text('Settings'),
                  ),
                ],
              ),
              const VerticalDivider(width: 1),
              Expanded(child: content),
            ],
          );
        },
      ),
      floatingActionButton: !isLinux && _currentIndex == 0 && isConnected
          ? FloatingActionButton.extended(
              onPressed: () => Navigator.of(context).pushNamed('/add'),
              icon: const Icon(Symbols.add),
              label: const Text('Add Account'),
            )
          : null,
      bottomNavigationBar: LayoutBuilder(
        builder: (context, constraints) {
          final useRail = constraints.maxWidth >= _desktopNavBreakpoint;
          if (useRail) return const SizedBox.shrink();

          return NavigationBar(
            selectedIndex: _currentIndex,
            onDestinationSelected: (index) =>
                setState(() => _currentIndex = index),
            labelBehavior: NavigationDestinationLabelBehavior.alwaysShow,
            destinations: const [
              NavigationDestination(
                icon: Icon(Symbols.key),
                label: 'Credentials',
              ),
              NavigationDestination(
                icon: Icon(Symbols.info),
                label: 'Key Info',
              ),
              NavigationDestination(
                icon: Icon(Symbols.settings),
                label: 'Settings',
              ),
            ],
          );
        },
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

  Widget _buildHeader(BuildContext context,
      conn.ConnectionNotifierState connection, bool isDark) {
    final colorScheme = Theme.of(context).colorScheme;
    final isConnected = connection.status == conn.ConnectionState.connected;

    return Container(
      padding: const EdgeInsets.fromLTRB(20, 48, 20, 20),
      decoration: BoxDecoration(
        color: colorScheme.surface.withValues(alpha: 0.95),
      ),
      child: Column(
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(
                isConnected ? Symbols.usb : Symbols.usb_off,
                size: 20,
                color: isConnected
                    ? colorScheme.primary
                    : colorScheme.onSurfaceVariant,
              ),
              const SizedBox(width: 8),
              Text(
                connection.yubiKeyInfo?.deviceName ?? 'No YubiKey',
                style: Theme.of(context).textTheme.titleMedium?.copyWith(
                      fontWeight: FontWeight.bold,
                    ),
              ),
              if (isConnected) ...[
                const SizedBox(width: 8),
                Container(
                  width: 8,
                  height: 8,
                  decoration: BoxDecoration(
                    color: colorScheme.primary,
                    shape: BoxShape.circle,
                    boxShadow: [
                      BoxShadow(
                        color: colorScheme.primary.withValues(alpha: 0.35),
                        blurRadius: 4,
                      ),
                    ],
                  ),
                ),
              ],
            ],
          ),
          if (isConnected)
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Text(
                'Connected',
                style: Theme.of(context).textTheme.labelSmall?.copyWith(
                      color: colorScheme.primary,
                      fontWeight: FontWeight.w600,
                      letterSpacing: 0.5,
                    ),
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildSearchBar(BuildContext context, bool isDark) {
    final colorScheme = Theme.of(context).colorScheme;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Container(
        decoration: BoxDecoration(
          color: colorScheme.surface,
          borderRadius: BorderRadius.circular(16),
          border: Border.all(
            color: isDark
                ? Colors.white.withValues(alpha: 0.08)
                : Colors.grey.shade200,
          ),
        ),
        child: TextField(
          onChanged: (value) => setState(() => _searchQuery = value),
          decoration: InputDecoration(
            hintText: 'Search accounts...',
            hintStyle: TextStyle(color: colorScheme.onSurfaceVariant),
            prefixIcon: Icon(
              Symbols.search,
              color: colorScheme.onSurfaceVariant,
            ),
            filled: false,
            border: InputBorder.none,
            focusedBorder: InputBorder.none,
            contentPadding:
                const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
          ),
        ),
      ),
    );
  }

  Widget _buildNoDevice(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    return SliverFillRemaining(
      hasScrollBody: false,
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Symbols.usb_off,
              size: 64,
              color: colorScheme.onSurfaceVariant,
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
                    color: colorScheme.onSurfaceVariant,
                  ),
            ),
            const SizedBox(height: 24),
            ElevatedButton.icon(
              onPressed: () =>
                  ref.read(connectionProvider.notifier).reconnect(),
              icon: Icon(Symbols.refresh),
              label: const Text('Retry'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildNoCredentials(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    return SliverFillRemaining(
      hasScrollBody: false,
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Symbols.vpn_key_off,
              size: 64,
              color: colorScheme.onSurfaceVariant,
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
                    color: colorScheme.onSurfaceVariant,
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
                onCalculate: () => ref
                    .read(credentialsProvider.notifier)
                    .calculate(credential.id),
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
    final colorScheme = Theme.of(context).colorScheme;
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
            style: TextButton.styleFrom(foregroundColor: colorScheme.error),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
  }
}
