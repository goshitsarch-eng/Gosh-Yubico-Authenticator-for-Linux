import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../theme/colors.dart';

/// Circular countdown timer widget for TOTP credentials.
class CircularCountdown extends StatelessWidget {
  final double progress;
  final double size;
  final Color? color;
  final double strokeWidth;

  const CircularCountdown({
    super.key,
    required this.progress,
    this.size = 32,
    this.color,
    this.strokeWidth = 3,
  });

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final progressColor = color ?? _getProgressColor(progress);
    final backgroundColor = isDark
        ? Colors.white.withValues(alpha: 0.1)
        : Colors.grey.shade200;

    return SizedBox(
      width: size,
      height: size,
      child: CustomPaint(
        painter: _CircularProgressPainter(
          progress: progress,
          progressColor: progressColor,
          backgroundColor: backgroundColor,
          strokeWidth: strokeWidth,
        ),
      ),
    );
  }

  Color _getProgressColor(double progress) {
    if (progress <= 0.25) {
      return AppColors.warning;
    }
    return AppColors.primary;
  }
}

class _CircularProgressPainter extends CustomPainter {
  final double progress;
  final Color progressColor;
  final Color backgroundColor;
  final double strokeWidth;

  _CircularProgressPainter({
    required this.progress,
    required this.progressColor,
    required this.backgroundColor,
    required this.strokeWidth,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = (size.width - strokeWidth) / 2;

    // Background circle
    final backgroundPaint = Paint()
      ..color = backgroundColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = strokeWidth;

    canvas.drawCircle(center, radius, backgroundPaint);

    // Progress arc
    final progressPaint = Paint()
      ..color = progressColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = strokeWidth
      ..strokeCap = StrokeCap.round;

    final sweepAngle = 2 * math.pi * progress;
    canvas.drawArc(
      Rect.fromCircle(center: center, radius: radius),
      -math.pi / 2, // Start from top
      sweepAngle,
      false,
      progressPaint,
    );
  }

  @override
  bool shouldRepaint(_CircularProgressPainter oldDelegate) {
    return oldDelegate.progress != progress ||
        oldDelegate.progressColor != progressColor;
  }
}
