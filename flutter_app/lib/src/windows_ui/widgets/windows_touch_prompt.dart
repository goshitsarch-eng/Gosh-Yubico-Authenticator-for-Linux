import 'package:fluent_ui/fluent_ui.dart';

class WindowsTouchPrompt extends StatelessWidget {
  final VoidCallback? onCancel;

  const WindowsTouchPrompt({super.key, this.onCancel});

  @override
  Widget build(BuildContext context) {
    final theme = FluentTheme.of(context);
    return Positioned.fill(
      child: Mica(
        child: Container(
          color: Colors.black.withValues(alpha: 0.15),
          child: Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 520),
              child: Card(
                padding: const EdgeInsets.all(24),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      FluentIcons.touch,
                      size: 52,
                      color: theme.accentColor,
                    ),
                    const SizedBox(height: 16),
                    Text(
                      'Touch your YubiKey',
                      style: theme.typography.title,
                      textAlign: TextAlign.center,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      'Touch the contact point on your security key to generate the code.',
                      style: theme.typography.body,
                      textAlign: TextAlign.center,
                    ),
                    const SizedBox(height: 18),
                    FilledButton(
                      onPressed: onCancel,
                      child: const Text('Cancel Request'),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
