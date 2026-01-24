import 'package:fluent_ui/fluent_ui.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../ffi/gosh_event.dart';
import '../../providers/providers.dart';
import '../../utils/qr_scanner.dart';

class WindowsAddCredentialScreen extends ConsumerStatefulWidget {
  const WindowsAddCredentialScreen({super.key});

  @override
  ConsumerState<WindowsAddCredentialScreen> createState() =>
      _WindowsAddCredentialScreenState();
}

class _WindowsAddCredentialScreenState
    extends ConsumerState<WindowsAddCredentialScreen> {
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
    final isConnected = ref.watch(isConnectedProvider);

    return NavigationView(
      content: ScaffoldPage.scrollable(
        header: PageHeader(
          title: const Text('Add Credential'),
          commandBar: Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              if (isConnected)
                Icon(
                  FluentIcons.plug_connected,
                  color: FluentTheme.of(context).accentColor,
                  size: 18,
                )
              else
                const Icon(FluentIcons.plug_disconnected, size: 18),
              const SizedBox(width: 8),
              Text(isConnected ? 'Connected' : 'No YubiKey'),
            ],
          ),
        ),
        children: [
          Form(
            key: _formKey,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _buildBasics(context),
                const SizedBox(height: 16),
                _buildSecurity(context),
                const SizedBox(height: 20),
                Row(
                  children: [
                    Expanded(
                      child: FilledButton(
                        onPressed: _isSubmitting ? null : _submit,
                        child: Text(
                            _isSubmitting ? 'Saving...' : 'Save Credential'),
                      ),
                    ),
                    const SizedBox(width: 8),
                    Button(
                      onPressed: _isSubmitting
                          ? null
                          : () => Navigator.of(context).pop(),
                      child: const Text('Cancel'),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildBasics(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('Basics'),
            const SizedBox(height: 12),
            InfoLabel(
              label: 'Issuer (optional)',
              child: TextFormBox(
                controller: _issuerController,
                placeholder: 'e.g. Google, AWS',
              ),
            ),
            const SizedBox(height: 12),
            InfoLabel(
              label: 'Account Name',
              child: TextFormBox(
                controller: _accountController,
                placeholder: 'user@example.com',
                validator: (value) {
                  if (value == null || value.trim().isEmpty) {
                    return 'Account name is required';
                  }
                  return null;
                },
              ),
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: InfoLabel(
                    label: 'Type',
                    child: ComboBox<OathType>(
                      value: _oathType,
                      items: OathType.values
                          .map(
                            (t) => ComboBoxItem(
                              value: t,
                              child: Text(t == OathType.totp ? 'TOTP' : 'HOTP'),
                            ),
                          )
                          .toList(),
                      onChanged: (value) {
                        if (value == null) return;
                        setState(() => _oathType = value);
                      },
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: InfoLabel(
                    label: 'Algorithm',
                    child: ComboBox<Algorithm>(
                      value: _algorithm,
                      items: Algorithm.values
                          .map(
                            (a) => ComboBoxItem(
                              value: a,
                              child: Text(a.name.toUpperCase()),
                            ),
                          )
                          .toList(),
                      onChanged: (value) {
                        if (value == null) return;
                        setState(() => _algorithm = value);
                      },
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: InfoLabel(
                    label: 'Digits',
                    child: ComboBox<int>(
                      value: _digits,
                      items: const [6, 8]
                          .map(
                            (d) => ComboBoxItem(
                              value: d,
                              child: Text('$d'),
                            ),
                          )
                          .toList(),
                      onChanged: (value) {
                        if (value == null) return;
                        setState(() => _digits = value);
                      },
                    ),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSecurity(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text('Security'),
                Button(
                  onPressed: _scanQrCode,
                  child: const Text('Scan QR'),
                ),
              ],
            ),
            const SizedBox(height: 12),
            InfoLabel(
              label: 'Secret (Base32)',
              child: TextFormBox(
                controller: _secretController,
                placeholder: 'JBSWY3DPEHPK3PXP',
                obscureText: !_showSecret,
                suffix: IconButton(
                  icon: Icon(
                    _showSecret ? FluentIcons.hide3 : FluentIcons.red_eye,
                    size: 14,
                  ),
                  onPressed: () => setState(() => _showSecret = !_showSecret),
                ),
                validator: (value) {
                  if (value == null || value.trim().isEmpty) {
                    return 'Secret key is required';
                  }
                  final cleaned = value.replaceAll(' ', '').toUpperCase();
                  if (!RegExp(r'^[A-Z2-7]+=*$').hasMatch(cleaned)) {
                    return 'Invalid Base32 secret';
                  }
                  return null;
                },
              ),
            ),
            const SizedBox(height: 12),
            InfoLabel(
              label: 'Require touch',
              child: ToggleSwitch(
                checked: _requireTouch,
                onChanged: (value) => setState(() => _requireTouch = value),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _scanQrCode() async {
    try {
      final credential = await QrScanner.scanFromFile();
      if (credential == null) return;

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
    } on QrScanException catch (e) {
      if (!mounted) return;
      _showError(context, e.message);
    } catch (e) {
      if (!mounted) return;
      _showError(context, 'Failed to scan QR code: $e');
    }
  }

  void _showError(BuildContext context, String message) {
    displayInfoBar(
      context,
      builder: (context, close) => InfoBar(
        title: const Text('Error'),
        content: Text(message),
        severity: InfoBarSeverity.error,
        isLong: true,
        action: IconButton(
          icon: const Icon(FluentIcons.clear),
          onPressed: close,
        ),
      ),
    );
  }

  Future<void> _submit() async {
    if (!_formKey.currentState!.validate()) return;

    setState(() => _isSubmitting = true);

    final client = ref.read(goshClientProvider);
    final success = client.addCredential(
      issuer: _issuerController.text.trim().isEmpty
          ? null
          : _issuerController.text.trim(),
      account: _accountController.text.trim(),
      secret: _secretController.text.replaceAll(' ', ''),
      oathType: _oathType,
      algorithm: _algorithm,
      digits: _digits,
      requireTouch: _requireTouch,
    );

    setState(() => _isSubmitting = false);

    if (!mounted) return;

    if (success) {
      Navigator.of(context).pop();
      return;
    }

    _showError(context, client.takeLastError() ?? 'Failed to add credential');
  }
}
