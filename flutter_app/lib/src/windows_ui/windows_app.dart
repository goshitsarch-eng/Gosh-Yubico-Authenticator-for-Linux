import 'package:fluent_ui/fluent_ui.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:system_theme/system_theme.dart';

import '../providers/providers.dart';
import 'screens/windows_home_shell.dart';
import 'widgets/windows_pin_entry_dialog.dart';
import 'widgets/windows_touch_prompt.dart';

class WindowsApp extends ConsumerWidget {
  const WindowsApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeMode = ref.watch(themeModeProvider);
    final isAuthRequired = ref.watch(isAuthRequiredProvider);
    final isTouchRequired = ref.watch(isTouchRequiredProvider);

    final accentColor = SystemTheme.accentColor.accent.toAccentColor();

    return FluentApp(
      debugShowCheckedModeBanner: false,
      title: 'Gosh Yubikey Manager',
      themeMode: themeMode,
      theme: FluentThemeData(
        accentColor: accentColor,
        visualDensity: VisualDensity.standard,
      ),
      darkTheme: FluentThemeData(
        brightness: Brightness.dark,
        accentColor: accentColor,
        visualDensity: VisualDensity.standard,
      ),
      home: Stack(
        children: [
          const WindowsHomeShell(),
          if (isTouchRequired)
            WindowsTouchPrompt(
              onCancel: () => ref.read(touchProvider.notifier).hidePrompt(),
            ),
          if (isAuthRequired) const WindowsPinEntryDialog(),
        ],
      ),
    );
  }
}
