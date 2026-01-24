import 'package:fluent_ui/fluent_ui.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../providers/connection_provider.dart' as conn;
import '../../providers/providers.dart';
import '../windows_icons.dart';
import 'windows_add_credential_screen.dart';
import 'windows_key_info_screen.dart';
import 'windows_settings_screen.dart';
import 'windows_credentials_screen.dart';

class WindowsHomeShell extends ConsumerStatefulWidget {
  const WindowsHomeShell({super.key});

  @override
  ConsumerState<WindowsHomeShell> createState() => _WindowsHomeShellState();
}

class _WindowsHomeShellState extends ConsumerState<WindowsHomeShell> {
  int _index = 0;

  @override
  Widget build(BuildContext context) {
    final connectionState = ref.watch(connectionProvider);
    final isConnected =
        connectionState.status == conn.ConnectionState.connected;

    final titles = ['Credentials', 'Key Info', 'Settings'];
    const bodies = [
      WindowsCredentialsScreen(),
      WindowsKeyInfoScreen(),
      WindowsSettingsScreen(),
    ];

    return NavigationView(
      pane: NavigationPane(
        selected: _index,
        onChanged: (value) => setState(() => _index = value),
        displayMode: PaneDisplayMode.auto,
        items: [
          PaneItem(
            icon: const Icon(AppWindowsIcons.key),
            title: const Text('Credentials'),
            body: bodies[0],
          ),
          PaneItem(
            icon: const Icon(AppWindowsIcons.info),
            title: const Text('Key Info'),
            body: bodies[1],
          ),
          PaneItem(
            icon: const Icon(AppWindowsIcons.settings),
            title: const Text('Settings'),
            body: bodies[2],
          ),
        ],
      ),
      appBar: NavigationAppBar(
        title: Text(titles[_index]),
        actions: Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            if (_index == 0 && isConnected)
              Padding(
                padding: const EdgeInsets.only(right: 8),
                child: FilledButton(
                  onPressed: () {
                    Navigator.of(context).push(
                      FluentPageRoute(
                        builder: (context) =>
                            const WindowsAddCredentialScreen(),
                      ),
                    );
                  },
                  child: const Row(
                    children: [
                      Icon(AppWindowsIcons.add, size: 16),
                      SizedBox(width: 8),
                      Text('Add Account'),
                    ],
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}
