import 'package:flutter/material.dart';
import 'package:material_symbols_icons/symbols.dart';

import '../theme/colors.dart';

/// Full-screen prompt for YubiKey touch.
class TouchPrompt extends StatelessWidget {
  final VoidCallback? onCancel;

  const TouchPrompt({super.key, this.onCancel});

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Material(
      color: isDark ? AppColors.backgroundDark : AppColors.backgroundLight,
      child: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            children: [
              const Spacer(),

              // Technical background accents
              Stack(
                alignment: Alignment.center,
                children: [
                  // Outer ring
                  Container(
                    width: 300,
                    height: 300,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      border: Border.all(
                        color: AppColors.primary.withValues(alpha: 0.1),
                        width: 1,
                      ),
                    ),
                  ),
                  // Middle ring
                  _PulsingRing(
                    size: 250,
                    color: AppColors.primary.withValues(alpha: 0.2),
                  ),
                  // YubiKey illustration
                  _buildYubiKeyIllustration(isDark),
                ],
              ),
              const SizedBox(height: 48),

              // Text content
              Text(
                'Touch your YubiKey',
                style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                      fontWeight: FontWeight.bold,
                    ),
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 12),
              Text(
                'Touch the gold contact point on your security key to generate your code.',
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      color: AppColors.textSecondary,
                    ),
                textAlign: TextAlign.center,
              ),

              const Spacer(),

              // Cancel button
              OutlinedButton(
                onPressed: onCancel,
                style: OutlinedButton.styleFrom(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 32,
                    vertical: 16,
                  ),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(24),
                  ),
                  side: BorderSide(
                    color: isDark ? Colors.grey.shade700 : Colors.grey.shade300,
                  ),
                ),
                child: const Text('Cancel Request'),
              ),
              const SizedBox(height: 32),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildYubiKeyIllustration(bool isDark) {
    return Container(
      width: 128,
      height: 224,
      decoration: BoxDecoration(
        color: const Color(0xFF1E2124),
        borderRadius: const BorderRadius.only(
          topLeft: Radius.circular(16),
          topRight: Radius.circular(16),
          bottomLeft: Radius.circular(64),
          bottomRight: Radius.circular(64),
        ),
        border: Border.all(color: Colors.grey.shade700),
        boxShadow: [
          BoxShadow(
            color: AppColors.primary.withValues(alpha: 0.2),
            blurRadius: 40,
            spreadRadius: 10,
          ),
        ],
      ),
      child: Column(
        children: [
          // USB connector
          Container(
            width: 80,
            height: 40,
            margin: const EdgeInsets.only(top: 4),
            decoration: BoxDecoration(
              color: const Color(0xFF2A2E33),
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(12),
                topRight: Radius.circular(12),
              ),
              border: Border(
                bottom: BorderSide(color: Colors.grey.shade700),
              ),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: List.generate(
                4,
                (index) => Container(
                  width: 2,
                  height: 16,
                  margin: const EdgeInsets.symmetric(horizontal: 2),
                  decoration: BoxDecoration(
                    color: Colors.grey.shade600,
                    borderRadius: BorderRadius.circular(1),
                  ),
                ),
              ),
            ),
          ),
          const SizedBox(height: 16),

          // Security icon
          Container(
            width: 32,
            height: 32,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              border: Border.all(
                color: AppColors.primary.withValues(alpha: 0.3),
                width: 2,
              ),
            ),
            child: Icon(
              Symbols.security_key,
              size: 16,
              color: AppColors.primary.withValues(alpha: 0.5),
            ),
          ),
          const SizedBox(height: 24),

          // Gold touch sensor
          _GoldSensor(),
          const Spacer(),

          // Keyring hole
          Container(
            width: 16,
            height: 16,
            margin: const EdgeInsets.only(bottom: 16),
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: const Color(0xFF121416),
              border: Border.all(color: Colors.grey.shade700),
            ),
          ),
        ],
      ),
    );
  }
}

class _PulsingRing extends StatefulWidget {
  final double size;
  final Color color;

  const _PulsingRing({required this.size, required this.color});

  @override
  State<_PulsingRing> createState() => _PulsingRingState();
}

class _PulsingRingState extends State<_PulsingRing>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _animation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 2000),
      vsync: this,
    )..repeat();

    _animation = Tween<double>(begin: 0.6, end: 1.0).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeInOut),
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _animation,
      builder: (context, child) {
        return Container(
          width: widget.size,
          height: widget.size,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            border: Border.all(
              color: widget.color.withValues(alpha: _animation.value * 0.3),
              width: 1,
            ),
          ),
        );
      },
    );
  }
}

class _GoldSensor extends StatefulWidget {
  @override
  State<_GoldSensor> createState() => _GoldSensorState();
}

class _GoldSensorState extends State<_GoldSensor>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _animation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 2000),
      vsync: this,
    )..repeat();

    _animation = Tween<double>(begin: 0.5, end: 1.0).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeInOut),
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _animation,
      builder: (context, child) {
        return Stack(
          alignment: Alignment.center,
          children: [
            // Glow
            Container(
              width: 70,
              height: 70,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                boxShadow: [
                  BoxShadow(
                    color:
                        AppColors.primaryGlow.withValues(alpha: _animation.value * 0.4),
                    blurRadius: 20,
                    spreadRadius: 5,
                  ),
                ],
              ),
            ),
            // Outer ring
            Container(
              width: 56,
              height: 56,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                border: Border.all(
                  color: const Color(0xFFC0A062).withValues(alpha: 0.3),
                  width: 2,
                ),
              ),
            ),
            // Sensor
            Container(
              width: 40,
              height: 40,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                gradient: const LinearGradient(
                  begin: Alignment.topLeft,
                  end: Alignment.bottomRight,
                  colors: [
                    Color(0xFFE6C680),
                    Color(0xFFC0A062),
                  ],
                ),
                boxShadow: [
                  BoxShadow(
                    color: Colors.white.withValues(alpha: 0.3),
                    offset: const Offset(0, -2),
                    blurRadius: 4,
                  ),
                ],
              ),
              child: Container(
                margin: const EdgeInsets.all(4),
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  border: Border.all(
                    color: const Color(0xFF8B7345).withValues(alpha: 0.5),
                  ),
                ),
              ),
            ),
          ],
        );
      },
    );
  }
}
