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
ThemeData windowTheme({bool highContrast = false}) => ThemeData(
  fontFamily: 'Inter',
  useMaterial3: true,
  scaffoldBackgroundColor: Colors.transparent,
  extensions: const [HomeVerbSkin()],
  colorScheme: highContrast
      ? const ColorScheme.highContrastLight(
          primary: Colors.black,
          onPrimary: Colors.white,
          surface: Colors.white,
          onSurface: Colors.black,
        )
      : ColorScheme.fromSeed(seedColor: accent).copyWith(
          primary: accent,
          surface: const Color(0xfff6f2ea),
          onSurface: ink,
        ),
  splashFactory: InkRipple.splashFactory,
  textSelectionTheme: const TextSelectionThemeData(cursorColor: accent),
  switchTheme: SwitchThemeData(
    padding: EdgeInsets.zero,
    thumbColor: highContrast
        ? WidgetStateProperty.resolveWith(
            (states) => states.contains(WidgetState.selected)
                ? Colors.white
                : Colors.black,
          )
        : const WidgetStatePropertyAll(Color(0xfffffcf5)),
    thumbIcon: const WidgetStatePropertyAll(Icon(null)),
    trackColor: WidgetStateProperty.resolveWith(
      (states) => states.contains(WidgetState.selected)
          ? (highContrast ? Colors.black : accent)
          : (highContrast ? Colors.white : const Color(0xffc2c4bc)),
    ),
    trackOutlineWidth: WidgetStatePropertyAll(highContrast ? 1.5 : 0),
    trackOutlineColor: WidgetStatePropertyAll(
      highContrast ? Colors.black : Colors.transparent,
    ),
  ),
);

Color readableInk(BuildContext context, Color color) =>
    MediaQuery.highContrastOf(context)
    ? (color.computeLuminance() >= .5 ? Colors.white : Colors.black)
    : color;

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
    final textSize = MediaQuery.highContrastOf(context)
        ? math.max(size, 12.0)
        : size;
    final style = TextStyle(
      fontFamily: 'Inter',
      fontSize: textSize,
      height: 1.4,
      letterSpacing: 0,
      wordSpacing: 0,
      color: readableInk(context, color),
      fontWeight: bold || MediaQuery.highContrastOf(context)
          ? FontWeight.w600
          : FontWeight.w400,
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
          child: MediaQuery.highContrastOf(context)
              ? const ColoredBox(color: Colors.white)
              : Image.asset(
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
      color: MediaQuery.highContrastOf(context)
          ? (dark ? Colors.black : Colors.white)
          : null,
      gradient: MediaQuery.highContrastOf(context)
          ? null
          : WindowSurfaceGradient(
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
        color: MediaQuery.highContrastOf(context)
            ? (dark ? Colors.white : Colors.black)
            : border ?? (dark ? const Color(0xff5e605c) : Colors.white),
        width: MediaQuery.highContrastOf(context)
            ? math.max(borderWidth, 1.5)
            : borderWidth,
      ),
      boxShadow: MediaQuery.highContrastOf(context)
          ? null
          : const [
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
        disabledForegroundColor: MediaQuery.highContrastOf(context)
            ? Colors.black
            : const Color(0xff888781),
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 14),
        backgroundColor: MediaQuery.highContrastOf(context)
            ? (style == '' ? Colors.black : Colors.white)
            : style == 'danger'
            ? const Color(0xfffae7df)
            : style == ''
            ? accent
            : const Color(0xffeeebe3),
        foregroundColor: MediaQuery.highContrastOf(context)
            ? (style == '' ? Colors.white : Colors.black)
            : style == 'danger'
            ? const Color(0xff9e3421)
            : style == ''
            ? const Color(0xfffffaf5)
            : ink,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(15),
          side: MediaQuery.highContrastOf(context)
              ? const BorderSide(color: Colors.black, width: 1.5)
              : style == 'secondary'
              ? const BorderSide(color: Color(0xffd7d6cf))
              : BorderSide.none,
        ),
      ),
      onPressed: onPressed,
      child: Transform.translate(
        offset: const Offset(0, -3),
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
    ),
  );
}

class OperationProgress extends StatelessWidget {
  const OperationProgress({super.key});
  @override
  Widget build(BuildContext context) => Semantics(
    label: 'Operation in progress',
    liveRegion: true,
    child: ExcludeSemantics(
      child: LinearProgressIndicator(
        // A static activity marker avoids an indefinite animation when requested.
        value: MediaQuery.disableAnimationsOf(context) ? .35 : null,
      ),
    ),
  );
}

class InlineStepper extends StatelessWidget {
  const InlineStepper({
    required this.label,
    required this.value,
    required this.onIncrease,
    required this.onDecrease,
    super.key,
  });
  final String label, value;
  final VoidCallback? onIncrease, onDecrease;

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) => SizedBox(
      width: math.min(
        155 * math.max(1, MediaQuery.textScalerOf(context).scale(12) / 12),
        constraints.maxWidth,
      ),
      height: 44 * MediaQuery.textScalerOf(context).scale(12) / 12,
      child: Stack(
        alignment: Alignment.center,
        children: [
          Positioned(
            left: 60,
            right: 45,
            child: Copy(value, size: 12, bold: true, center: true),
          ),
          for (final increase in [false, true])
            Positioned(
              left: increase ? null : 16,
              right: increase ? 0 : null,
              top: 0,
              bottom: 0,
              width: 44,
              child: Semantics(
                button: true,
                label: '${increase ? 'Increase' : 'Decrease'} $label',
                enabled: (increase ? onIncrease : onDecrease) != null,
                child: InkWell(
                  onTap: increase ? onIncrease : onDecrease,
                  child: Center(
                    child: Container(
                      width: 30,
                      height: 30,
                      alignment: Alignment.center,
                      decoration: const BoxDecoration(
                        color: Color(0xffe6e2d8),
                        shape: BoxShape.circle,
                      ),
                      child: ExcludeSemantics(
                        child: Copy(
                          increase ? '+' : '−',
                          size: 16,
                          center: true,
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
        ],
      ),
    ),
  );
}

class ValueSlider extends StatelessWidget {
  const ValueSlider({
    required this.label,
    required this.displayValue,
    required this.value,
    required this.onChanged,
    required this.onChangeEnd,
    required this.formatValue,
    this.min = 0,
    this.max = 1,
    super.key,
  });
  final String label, displayValue;
  final double value, min, max;
  final ValueChanged<double>? onChanged, onChangeEnd;
  final String Function(double) formatValue;

  @override
  Widget build(BuildContext context) {
    Widget slider() => MergeSemantics(
      child: Semantics(
        label: label,
        child: SliderTheme(
          data: SliderTheme.of(context).copyWith(
            trackHeight: 5,
            activeTrackColor: readableInk(context, accent),
            inactiveTrackColor: MediaQuery.highContrastOf(context)
                ? const Color(0xff666666)
                : const Color(0xffd5d6cf),
            thumbColor: readableInk(context, accent),
            thumbShape: const WindowSliderThumb(),
            padding: const EdgeInsets.symmetric(horizontal: 20),
            trackShape: const RoundedRectSliderTrackShape(),
          ),
          child: Slider(
            value: value.clamp(min, max),
            min: min,
            max: max,
            semanticFormatterCallback: formatValue,
            onChanged: onChanged,
            onChangeEnd: onChangeEnd,
          ),
        ),
      ),
    );
    Widget heading() => Row(
      children: [
        Expanded(child: Copy(label)),
        Copy(displayValue, bold: true),
      ],
    );
    return Padding(
      padding: const EdgeInsets.only(bottom: 13),
      child: Surface(
        radius: 18,
        child: MediaQuery.textScalerOf(context).scale(13) <= 13
            ? SizedBox(
                height: 94,
                child: Stack(
                  children: [
                    Positioned(top: 16, left: 16, right: 16, child: heading()),
                    Positioned(
                      top: 39.5,
                      left: 0,
                      right: 0,
                      height: 48,
                      child: slider(),
                    ),
                  ],
                ),
              )
            : Column(
                children: [
                  Padding(
                    padding: const EdgeInsets.fromLTRB(16, 16, 16, 0),
                    child: heading(),
                  ),
                  slider(),
                ],
              ),
      ),
    );
  }
}

class ChoiceTile extends StatelessWidget {
  const ChoiceTile({
    required this.label,
    required this.selected,
    required this.onTap,
    this.multiple = false,
    super.key,
  });
  final String label;
  final bool selected, multiple;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: 9),
    child: Semantics(
      selected: selected,
      checked: multiple ? selected : null,
      inMutuallyExclusiveGroup: !multiple,
      enabled: onTap != null,
      child: Surface(
        radius: 16,
        border: selected ? accent : null,
        borderWidth: selected ? 1.2 : .6,
        child: InkWell(
          borderRadius: BorderRadius.circular(16),
          onTap: onTap,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 15, vertical: 17),
            child: Row(
              children: [
                Expanded(child: Copy(label, bold: selected)),
                const SizedBox(width: 12),
                Container(
                  width: 18,
                  height: 18,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: selected ? accent : null,
                    border: Border.all(
                      color: selected ? accent : const Color(0xffb3b3ac),
                    ),
                  ),
                  child: selected
                      ? const Glyph('check', size: 14, color: Colors.white)
                      : null,
                ),
              ],
            ),
          ),
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
    child: CustomPaint(
      painter: _GlyphPainter(name, readableInk(context, color)),
    ),
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
    context.canvas.drawCircle(
      center,
      10,
      Paint()..color = sliderTheme.thumbColor ?? accent,
    );
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
