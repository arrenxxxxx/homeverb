import 'dart:math' as math;
import 'dart:ui' as ui;
import 'package:flutter/material.dart';

const ink = Color(0xff292b2d),
    muted = Color(0xff626566),
    accent = Color(0xffb94e29);

@immutable
class HomeVerbSkin extends ThemeExtension<HomeVerbSkin> {
  const HomeVerbSkin({this.accentColor = accent});
  final Color accentColor;
  @override
  HomeVerbSkin copyWith({Color? accentColor}) =>
      HomeVerbSkin(accentColor: accentColor ?? this.accentColor);
  @override
  HomeVerbSkin lerp(HomeVerbSkin? other, double t) => HomeVerbSkin(
    accentColor: Color.lerp(accentColor, other?.accentColor, t)!,
  );
}

const skinRegistry = {'window_light': HomeVerbSkin()};
ThemeData windowTheme() => ThemeData(
  fontFamily: 'Inter',
  useMaterial3: true,
  scaffoldBackgroundColor: Colors.transparent,
  extensions: const [HomeVerbSkin()],
  colorScheme: ColorScheme.fromSeed(
    seedColor: accent,
  ).copyWith(primary: accent, surface: const Color(0xfff6f2ea), onSurface: ink),
  splashFactory: InkRipple.splashFactory,
  textSelectionTheme: const TextSelectionThemeData(cursorColor: accent),
);

class Copy extends StatelessWidget {
  const Copy(
    this.text, {
    this.size = 13,
    this.color = ink,
    this.bold = false,
    this.center = false,
    super.key,
  });
  final String text;
  final double size;
  final Color color;
  final bool bold, center;
  @override
  Widget build(BuildContext context) {
    final style = TextStyle(
      fontFamily: 'Inter',
      fontSize: size,
      height: 1.4,
      letterSpacing: 0,
      wordSpacing: 0,
      color: color,
      fontWeight: bold ? FontWeight.w600 : FontWeight.w400,
    );
    final measure = TextPainter(
      text: TextSpan(text: text, style: style),
      textDirection: Directionality.of(context),
      textScaler: MediaQuery.textScalerOf(context),
      textHeightBehavior: const TextHeightBehavior(
        applyHeightToFirstAscent: false,
        applyHeightToLastDescent: false,
      ),
    )..layout();
    final glyphHeight = measure.height;
    measure.textHeightBehavior = const TextHeightBehavior();
    measure.layout();
    final leading = math.max(0.0, measure.height - glyphHeight);
    measure.dispose();
    return Padding(
      padding: EdgeInsets.only(bottom: leading),
      child: Transform.translate(
        offset: Offset(0, -size * .20),
        child: Text(
          text,
          textAlign: center ? TextAlign.center : TextAlign.start,
          textHeightBehavior: const TextHeightBehavior(
            applyHeightToFirstAscent: false,
            applyHeightToLastDescent: false,
          ),
          style: style,
        ),
      ),
    );
  }
}

class LightBackground extends StatelessWidget {
  const LightBackground({required this.child, super.key});
  final Widget child;
  @override
  Widget build(BuildContext context) => Stack(
    fit: StackFit.expand,
    children: [
      Positioned.fill(
        child: ExcludeSemantics(
          child: Image.asset(
            'assets/window-light-background.png',
            fit: BoxFit.fill,
            filterQuality: FilterQuality.high,
          ),
        ),
      ),
      child,
    ],
  );
}

class Surface extends StatelessWidget {
  const Surface({
    required this.child,
    this.dark = false,
    this.radius = 22,
    this.border,
    this.borderWidth = .6,
    this.bottomBleed = 0,
    this.padding = EdgeInsets.zero,
    super.key,
  });
  final Widget child;
  final bool dark;
  final double radius;
  final Color? border;
  final double borderWidth;
  final double bottomBleed;
  final EdgeInsetsGeometry padding;
  @override
  Widget build(BuildContext context) => DecoratedBox(
    decoration: BoxDecoration(
      gradient: WindowSurfaceGradient(
        bottomBleed: bottomBleed,
        colors: dark
            ? const [Color(0xff42413e), Color(0xff2b2e2f)]
            : const [Color(0xfffdfaf3), Color(0xffeeefeb)],
      ),
      borderRadius: BorderRadius.vertical(
        top: Radius.circular(radius),
        bottom: Radius.circular(bottomBleed > 0 ? 0 : radius),
      ),
      border: Border.all(
        color: border ?? (dark ? const Color(0xff5e605c) : Colors.white),
        width: borderWidth,
      ),
      boxShadow: const [
        BoxShadow(
          color: Color(0x192b2c2e),
          blurRadius: 10,
          offset: Offset(0, 3),
        ),
      ],
    ),
    child: Padding(padding: padding, child: child),
  );
}

class WindowSurfaceGradient extends Gradient {
  const WindowSurfaceGradient({required super.colors, this.bottomBleed = 0});
  final double bottomBleed;
  @override
  Shader createShader(Rect rect, {TextDirection? textDirection}) {
    rect = Rect.fromLTWH(
      rect.left,
      rect.top,
      rect.width,
      rect.height + bottomBleed,
    );
    // Preserve the prototype's 35% horizontal / 65% vertical light blend at any aspect ratio.
    if (rect.isEmpty)
      return ui.Gradient.linear(Offset.zero, const Offset(1, 1), colors);
    final x = .35 / rect.width, y = .65 / rect.height;
    final denominator = x * x + y * y;
    final delta = Offset(x / denominator, y / denominator);
    return ui.Gradient.linear(
      rect.center - delta / 2,
      rect.center + delta / 2,
      colors,
    );
  }

  @override
  Gradient scale(double factor) => WindowSurfaceGradient(
    bottomBleed: bottomBleed,
    colors: colors.map((c) => c.withValues(alpha: c.a * factor)).toList(),
  );
  @override
  Gradient withOpacity(double opacity) => WindowSurfaceGradient(
    bottomBleed: bottomBleed,
    colors: colors.map((c) => c.withValues(alpha: opacity)).toList(),
  );
}

class ActionButton extends StatelessWidget {
  const ActionButton(
    this.label, {
    required this.onPressed,
    this.style = '',
    super.key,
  });
  final String label, style;
  final VoidCallback? onPressed;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: 12),
    child: FilledButton(
      style: FilledButton.styleFrom(
        minimumSize: const Size(double.infinity, 48),
        disabledBackgroundColor: const Color(0xffe4e2dc),
        disabledForegroundColor: const Color(0xff888781),
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 14),
        backgroundColor: style == 'danger'
            ? const Color(0xfffae7df)
            : style == ''
            ? accent
            : const Color(0xffeeebe3),
        foregroundColor: style == 'danger'
            ? const Color(0xff9e3421)
            : style == ''
            ? const Color(0xfffffaf5)
            : ink,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(15),
          side: style == 'secondary'
              ? const BorderSide(color: Color(0xffd7d6cf))
              : BorderSide.none,
        ),
      ),
      onPressed: onPressed,
      child: Text(
        label,
        textAlign: TextAlign.center,
        style: const TextStyle(
          fontSize: 14,
          fontWeight: FontWeight.w600,
          height: 1.1,
          letterSpacing: 0,
        ),
      ),
    ),
  );
}

class Glyph extends StatelessWidget {
  const Glyph(this.name, {this.size = 24, this.color = ink, super.key});
  final String name;
  final double size;
  final Color color;
  @override
  Widget build(BuildContext context) => SizedBox.square(
    dimension: size,
    child: CustomPaint(painter: _GlyphPainter(name, color)),
  );
}

class WindowSliderThumb extends SliderComponentShape {
  const WindowSliderThumb();
  @override
  Size getPreferredSize(bool isEnabled, bool isDiscrete) => const Size(20, 20);
  @override
  void paint(
    PaintingContext context,
    Offset center, {
    required Animation<double> activationAnimation,
    required Animation<double> enableAnimation,
    required bool isDiscrete,
    required TextPainter labelPainter,
    required RenderBox parentBox,
    required SliderThemeData sliderTheme,
    required TextDirection textDirection,
    required double value,
    required double textScaleFactor,
    required Size sizeWithOverflow,
  }) {
    context.canvas.drawCircle(center, 10, Paint()..color = accent);
    context.canvas.drawCircle(
      center,
      8,
      Paint()..color = const Color(0xfffffaf3),
    );
  }
}

class _GlyphPainter extends CustomPainter {
  _GlyphPainter(this.name, this.color);
  final String name;
  final Color color;
  @override
  void paint(Canvas canvas, Size size) {
    canvas.scale(size.width / 24);
    final p = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.7
      ..strokeJoin = StrokeJoin.round;
    void line(List<Offset> points) {
      final path = Path()..moveTo(points.first.dx, points.first.dy);
      for (final point in points.skip(1)) {
        path.lineTo(point.dx, point.dy);
      }
      canvas.drawPath(path, p);
    }

    void rect(double x, double y, double w, double h, double r) =>
        canvas.drawRRect(
          RRect.fromRectAndRadius(
            Rect.fromLTWH(x, y, w, h),
            Radius.circular(r),
          ),
          p,
        );
    switch (name) {
      case 'drag':
        for (final y in [6.0, 12.0, 18.0]) {
          line([Offset(7.5, y), Offset(24, y)]);
        }
      case 'back':
        line(const [Offset(15, 4), Offset(7, 12), Offset(15, 20)]);
      case 'plus':
        line(const [Offset(12, 2), Offset(12, 22)]);
        line(const [Offset(2, 12), Offset(22, 12)]);
      case 'check':
        line(const [Offset(4, 12), Offset(10, 18), Offset(21, 5)]);
      case 'home':
        line(const [
          Offset(3, 10),
          Offset(12, 2),
          Offset(21, 10),
          Offset(21, 21),
          Offset(15, 21),
          Offset(15, 14),
          Offset(9, 14),
          Offset(9, 21),
          Offset(3, 21),
          Offset(3, 10),
        ]);
      case 'platform':
        line(const [
          Offset(2, 8),
          Offset(12, 2),
          Offset(22, 8),
          Offset(12, 14),
          Offset(2, 8),
        ]);
        line(const [Offset(2, 13), Offset(12, 19), Offset(22, 13)]);
        line(const [Offset(2, 18), Offset(12, 24), Offset(22, 18)]);
      case 'settings':
        rect(3, 3, 18, 18, 6);
        rect(9, 9, 6, 6, 3);
        line(const [Offset(12, 0), Offset(12, 4)]);
        line(const [Offset(12, 20), Offset(12, 24)]);
        line(const [Offset(0, 12), Offset(4, 12)]);
        line(const [Offset(20, 12), Offset(24, 12)]);
      case 'search':
        rect(2, 2, 15, 15, 8);
        line(const [Offset(16, 16), Offset(23, 23)]);
      case 'refresh':
        canvas.drawArc(const Rect.fromLTWH(3, 3, 18, 18), .7, 4.9, false, p);
        line(const [Offset(19, 1), Offset(21, 7), Offset(15, 7)]);
      case 'power':
        canvas.drawArc(const Rect.fromLTWH(3, 3, 18, 18), -.87, 4.88, false, p);
        line(const [Offset(12, 1), Offset(12, 12)]);
      case 'phone':
        rect(6, 1, 13, 22, 3);
        line(const [Offset(10, 19), Offset(15, 19)]);
      case 'washer':
        rect(3, 1, 18, 22, 3);
        rect(6, 8, 12, 12, 6);
        line(const [Offset(6, 5), Offset(10, 5)]);
      case 'fan':
        rect(3, 1, 18, 18, 9);
        line(const [
          Offset(12, 19),
          Offset(12, 24),
          Offset(5, 24),
          Offset(19, 24),
        ]);
        for (final a in [0, 120, 240]) {
          line([
            const Offset(12, 10),
            Offset(
              12 + 7 * math.cos(a * math.pi / 180),
              10 + 7 * math.sin(a * math.pi / 180),
            ),
          ]);
        }
      default:
        rect(3, 3, 18, 18, 6);
    }
  }

  @override
  bool shouldRepaint(covariant _GlyphPainter old) =>
      name != old.name || color != old.color;
}

class IconAction extends StatelessWidget {
  const IconAction(
    this.icon,
    this.label,
    this.onTap, {
    this.color = ink,
    super.key,
  });
  final String icon, label;
  final VoidCallback onTap;
  final Color color;
  @override
  Widget build(BuildContext context) => IconButton(
    tooltip: label,
    onPressed: onTap,
    icon: Glyph(icon, color: color),
    constraints: const BoxConstraints(minWidth: 48, minHeight: 48),
  );
}
