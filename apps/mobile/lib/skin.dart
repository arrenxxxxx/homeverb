import 'dart:convert';
import 'dart:math' as math;
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';

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
ui.FragmentProgram? _surfaceProgram;
Future<void>? _skinAssetsLoad;
final _buttonFaces = <String, ui.Image>{};
ui.Image? _glyphAtlas;
final _glyphRegions = <(String, double), Rect>{};
final _surfaceFaces = <double, List<ui.Image>>{};
final _textInkBounds = <(int, double), Map<int, (double, double)>>{};
final _editableInkBounds = <int, (double, double)>{};
Future<void> loadWindowSkin() => _skinAssetsLoad ??= () async {
  _surfaceProgram = await ui.FragmentProgram.fromAsset(
    'shaders/window_surface.frag',
  );
  final metrics =
      jsonDecode(await rootBundle.loadString('assets/text-ink-metrics.json'))
          as Map<String, dynamic>;
  for (final variant in metrics['variants'] as List) {
    _textInkBounds[(
      variant['weight'] as int,
      (variant['size'] as num).toDouble(),
    )] = {
      for (final entry in (variant['bounds'] as Map<String, dynamic>).entries)
        entry.key.runes.single: (
          (entry.value[0] as num).toDouble(),
          (entry.value[1] as num).toDouble(),
        ),
    };
    final editable = variant['editable_bounds'] as Map<String, dynamic>?;
    if (editable != null) {
      _editableInkBounds.addAll({
        for (final entry in editable.entries)
          entry.key.runes.single: (
            (entry.value[0] as num).toDouble(),
            (entry.value[1] as num).toDouble(),
          ),
      });
    }
  }
  final glyphs = jsonDecode(await rootBundle.loadString('assets/glyphs.json'));
  for (final entry in (glyphs['regions'] as Map<String, dynamic>).entries) {
    final key = entry.key.split(':');
    final region = (entry.value as List).cast<num>();
    _glyphRegions[(key[0], double.parse(key[1]))] = Rect.fromLTWH(
      region[0].toDouble(),
      region[1].toDouble(),
      region[2].toDouble(),
      region[3].toDouble(),
    );
  }
  final glyphBytes = await rootBundle.load('assets/glyphs.png');
  final glyphCodec = await ui.instantiateImageCodec(
    glyphBytes.buffer.asUint8List(),
  );
  _glyphAtlas = (await glyphCodec.getNextFrame()).image;
  glyphCodec.dispose();
  for (final radius in [12, 14, 16, 18, 20, 22, 25, 28]) {
    final faces = <ui.Image>[];
    for (final kind in ['mask', 'border-1', 'border-2']) {
      final bytes = await rootBundle.load('assets/surface-$radius-$kind.png');
      final codec = await ui.instantiateImageCodec(bytes.buffer.asUint8List());
      faces.add((await codec.getNextFrame()).image);
      codec.dispose();
    }
    _surfaceFaces[radius.toDouble()] = faces;
  }
  for (final style in ['primary', 'secondary', 'danger', 'disabled']) {
    final bytes = await rootBundle.load('assets/button-$style.png');
    final codec = await ui.instantiateImageCodec(bytes.buffer.asUint8List());
    _buttonFaces[style] = (await codec.getNextFrame()).image;
    codec.dispose();
  }
}();
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

const windowFieldTextStyle = TextStyle(
  fontFamily: 'Inter',
  fontSize: 15,
  fontWeight: FontWeight.w400,
  height: 4 / 3,
  letterSpacing: 0,
  wordSpacing: 0,
);

EdgeInsets fieldTextPadding(BuildContext context, String text) {
  const fallback = EdgeInsets.symmetric(horizontal: 15, vertical: 15);
  final metrics = _editableInkBounds;
  if (metrics.isEmpty ||
      MediaQuery.boldTextOf(context) ||
      MediaQuery.maybeLineHeightScaleFactorOverrideOf(context) != null ||
      MediaQuery.maybeLetterSpacingOverrideOf(context) != null ||
      MediaQuery.maybeWordSpacingOverrideOf(context) != null)
    return fallback;
  double? top, bottom;
  for (final rune in text.runes) {
    final glyph = metrics[rune];
    if (glyph == null) return fallback;
    if (glyph.$1 == glyph.$2) continue;
    top = top == null ? glyph.$1 : math.min(top, glyph.$1);
    bottom = bottom == null ? glyph.$2 : math.max(bottom, glyph.$2);
  }
  if (top == null || bottom == null) return fallback;
  final scaler = MediaQuery.textScalerOf(context);
  final scale = scaler.scale(15) / 30;
  final painter = TextPainter(
    text: TextSpan(
      text: text,
      style: Theme.of(context).textTheme.bodyLarge!.merge(windowFieldTextStyle),
    ),
    textDirection: Directionality.of(context),
    textScaler: scaler,
    maxLines: 1,
  )..layout();
  final inkTop =
      painter
          .computeDistanceToActualBaseline(TextBaseline.alphabetic)
          .roundToDouble() +
      top * scale;
  final shift = (painter.height - (bottom - top) * scale) / 2 - inkTop;
  painter.dispose();
  // Move the native editable, caret and selection together; retain total height.
  return EdgeInsets.fromLTRB(15, 15 + shift, 15, 15 - shift);
}

class Copy extends StatelessWidget {
  const Copy(
    this.text, {
    this.size = 13,
    this.color = ink,
    this.bold = false,
    this.center = false,
    this.control = false,
    this.paragraph = false,
    this.maxLines,
    super.key,
  });
  final String text;
  final double size;
  final Color color;
  final bool bold, center, control, paragraph;
  final int? maxLines;
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
    if (control) {
      return _InkAlignedText(
        text,
        textAlign: center ? TextAlign.center : TextAlign.start,
        maxLines: maxLines,
        style: style,
      );
    }
    return LayoutBuilder(
      builder: (context, constraints) {
        const heightBehavior = TextHeightBehavior(
          applyHeightToFirstAscent: false,
          applyHeightToLastDescent: false,
        );
        final scaler = MediaQuery.textScalerOf(context);
        final measuredStyle = DefaultTextStyle.of(context).style.merge(style);
        final measure = TextPainter(
          text: TextSpan(text: text, style: paragraph ? measuredStyle : style),
          textDirection: Directionality.of(context),
          textScaler: scaler,
          maxLines: maxLines,
          ellipsis: maxLines == null ? null : '…',
          textHeightBehavior: heightBehavior,
        )..layout(maxWidth: paragraph ? constraints.maxWidth : double.infinity);
        final glyphHeight = measure.height;
        final double leading;
        if (paragraph) {
          // Preserve the specified line box despite native paragraph rounding.
          final lineHeight = scaler.scale(textSize) * 1.4;
          leading = math.max(
            0.0,
            measure.computeLineMetrics().length * lineHeight - glyphHeight,
          );
        } else {
          measure.textHeightBehavior = const TextHeightBehavior();
          measure.layout();
          leading = math.max(0.0, measure.height - glyphHeight);
        }
        measure.dispose();
        return Padding(
          padding: EdgeInsets.only(bottom: leading),
          child: _InkAlignedText(
            text,
            textAlign: center ? TextAlign.center : TextAlign.start,
            maxLines: maxLines,
            textHeightBehavior: heightBehavior,
            topAligned: true,
            fallbackOffset: Offset(0, -size * .20),
            style: style,
          ),
        );
      },
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
  Widget build(BuildContext context) {
    final highContrast = MediaQuery.highContrastOf(context);
    final faces = highContrast ? null : _surfaceFaces[radius];
    return DecoratedBox(
      decoration: BoxDecoration(
        color: MediaQuery.highContrastOf(context)
            ? (dark ? Colors.black : Colors.white)
            : null,
        gradient: highContrast || faces != null
            ? null
            : WindowSurfaceGradient(
                pixelRatio: MediaQuery.devicePixelRatioOf(context),
                bottomBleed: bottomBleed,
                colors: dark
                    ? const [Color(0xff42413e), Color(0xff2b2e2f)]
                    : const [Color(0xfffdfaf3), Color(0xffeeefeb)],
              ),
        borderRadius: BorderRadius.vertical(
          top: Radius.circular(radius),
          bottom: Radius.circular(bottomBleed > 0 ? 0 : radius),
        ),
        border: faces != null
            ? null
            : Border.all(
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
                  color: Color(0x1a2b2c2e),
                  // The artwork uses sigma 10 at 2x scale (5 logical pixels).
                  blurRadius: (5 - .5) / .57735,
                  offset: Offset(0, 3),
                ),
              ],
      ),
      child: CustomPaint(
        painter: faces == null
            ? null
            : _SurfaceFace(
                faces: faces,
                radius: radius,
                dark: dark,
                border:
                    border ?? (dark ? const Color(0xff5e605c) : Colors.white),
                borderWidth: borderWidth,
                bottomBleed: bottomBleed,
                pixelRatio: MediaQuery.devicePixelRatioOf(context),
              ),
        child: Padding(padding: padding, child: child),
      ),
    );
  }
}

class _SurfaceFace extends CustomPainter {
  const _SurfaceFace({
    required this.faces,
    required this.radius,
    required this.dark,
    required this.border,
    required this.borderWidth,
    required this.bottomBleed,
    required this.pixelRatio,
  });
  final List<ui.Image> faces;
  final double radius, borderWidth, bottomBleed, pixelRatio;
  final bool dark;
  final Color border;

  @override
  void paint(Canvas canvas, Size size) {
    if (size.isEmpty) return;
    double pixel(double value) {
      final scaled = value * pixelRatio;
      final lower = scaled.floor();
      return (scaled - lower == .5
              ? (lower.isEven ? lower : lower + 1).toDouble()
              : scaled.roundToDouble()) /
          pixelRatio;
    }

    final transform = canvas.getTransform();
    canvas.save();
    if (transform[0] == 1 &&
        transform[5] == 1 &&
        transform[1] == 0 &&
        transform[4] == 0) {
      canvas.translate(
        pixel(transform[12]) - transform[12],
        pixel(transform[13]) - transform[13],
      );
    }
    final rect = Rect.fromLTWH(
      0,
      0,
      pixel(size.width),
      pixel(size.height + bottomBleed),
    );
    final center = Rect.fromLTWH(radius * 2, radius * 2, 1, 1);
    void drawFace(ui.Image image, Rect target, Paint paint) {
      canvas.save();
      canvas.scale(.5);
      canvas.drawImageNine(
        image,
        center,
        Rect.fromLTWH(
          target.left * 2,
          target.top * 2,
          target.width * 2,
          target.height * 2,
        ),
        paint,
      );
      canvas.restore();
    }

    canvas.saveLayer(rect, Paint());
    canvas.drawRect(
      rect,
      Paint()
        ..shader = WindowSurfaceGradient(
          pixelRatio: pixelRatio,
          colors: dark
              ? const [Color(0xff42413e), Color(0xff2b2e2f)]
              : const [Color(0xfffdfaf3), Color(0xffeeefeb)],
        ).createShader(rect),
    );
    drawFace(
      faces[0],
      rect,
      Paint()
        ..blendMode = BlendMode.dstIn
        ..filterQuality = FilterQuality.low,
    );
    canvas.restore();
    final outline = Rect.fromLTWH(
      0,
      0,
      rect.width + 1 / pixelRatio,
      rect.height + 1 / pixelRatio,
    );
    drawFace(
      faces[(borderWidth * 2).round().clamp(1, 2)],
      outline,
      Paint()
        ..filterQuality = FilterQuality.low
        ..colorFilter = ColorFilter.mode(border, BlendMode.srcIn),
    );
    canvas.restore();
  }

  @override
  bool shouldRepaint(_SurfaceFace old) =>
      faces != old.faces ||
      radius != old.radius ||
      dark != old.dark ||
      border != old.border ||
      borderWidth != old.borderWidth ||
      bottomBleed != old.bottomBleed ||
      pixelRatio != old.pixelRatio;
}

class WindowSurfaceGradient extends Gradient {
  const WindowSurfaceGradient({
    required super.colors,
    required this.pixelRatio,
    this.bottomBleed = 0,
  });
  final double bottomBleed, pixelRatio;
  @override
  Shader createShader(Rect rect, {TextDirection? textDirection}) {
    rect = Rect.fromLTWH(
      rect.left,
      rect.top,
      rect.width,
      rect.height + bottomBleed,
    );
    final program = _surfaceProgram;
    if (program != null && !rect.isEmpty) {
      final shader = program.fragmentShader();
      final first = colors.first, last = colors.last;
      final uniforms = [
        rect.left,
        rect.top,
        rect.width,
        rect.height,
        pixelRatio,
        first.r,
        first.g,
        first.b,
        first.a,
        last.r,
        last.g,
        last.b,
        last.a,
      ];
      for (var i = 0; i < uniforms.length; i++) {
        shader.setFloat(i, uniforms[i]);
      }
      return shader;
    }
    // Standalone widgets can render before the application skin assets load.
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
    pixelRatio: pixelRatio,
    bottomBleed: bottomBleed,
    colors: colors.map((c) => c.withValues(alpha: c.a * factor)).toList(),
  );
  @override
  Gradient withOpacity(double opacity) => WindowSurfaceGradient(
    pixelRatio: pixelRatio,
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
  Widget build(BuildContext context) {
    final face = MediaQuery.highContrastOf(context)
        ? null
        : _buttonFaces[onPressed == null
              ? 'disabled'
              : style == ''
              ? 'primary'
              : style];
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: CustomPaint(
        painter: face == null
            ? null
            : _ButtonFace(face, MediaQuery.devicePixelRatioOf(context)),
        child: FilledButton(
          style: FilledButton.styleFrom(
            minimumSize: const Size(double.infinity, 48),
            disabledBackgroundColor: face == null
                ? const Color(0xffe4e2dc)
                : Colors.transparent,
            disabledForegroundColor: MediaQuery.highContrastOf(context)
                ? Colors.black
                : const Color(0xff888781),
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 14),
            backgroundColor: face != null
                ? Colors.transparent
                : MediaQuery.highContrastOf(context)
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
              side: face != null
                  ? BorderSide.none
                  : MediaQuery.highContrastOf(context)
                  ? const BorderSide(color: Colors.black, width: 1.5)
                  : style == 'secondary'
                  ? const BorderSide(color: Color(0xffd7d6cf))
                  : BorderSide.none,
            ),
          ),
          onPressed: onPressed,
          child: _InkAlignedText(
            label,
            textAlign: TextAlign.center,
            style: const TextStyle(
              fontSize: 14,
              fontWeight: FontWeight.w600,
              height: 1.4,
              letterSpacing: 0,
            ),
          ),
        ),
      ),
    );
  }
}

class _InkAlignedText extends StatelessWidget {
  const _InkAlignedText(
    this.label, {
    required this.style,
    required this.textAlign,
    this.maxLines,
    this.textHeightBehavior,
    this.topAligned = false,
    this.fallbackOffset = Offset.zero,
  });
  final TextHeightBehavior? textHeightBehavior;
  final bool topAligned;
  final Offset fallbackOffset;
  final String label;
  final TextStyle style;
  final TextAlign textAlign;
  final int? maxLines;

  @override
  Widget build(BuildContext context) {
    final text = Text(
      label,
      textAlign: textAlign,
      maxLines: maxLines,
      overflow: maxLines == null ? null : TextOverflow.ellipsis,
      textHeightBehavior: textHeightBehavior,
      style: style,
    );
    final metrics = _textInkBounds[(style.fontWeight!.value, style.fontSize!)];
    if (metrics == null ||
        MediaQuery.boldTextOf(context) ||
        MediaQuery.maybeLineHeightScaleFactorOverrideOf(context) != null ||
        MediaQuery.maybeLetterSpacingOverrideOf(context) != null ||
        MediaQuery.maybeWordSpacingOverrideOf(context) != null) {
      return Transform.translate(offset: fallbackOffset, child: text);
    }
    final defaults = DefaultTextStyle.of(context);
    final scaler = MediaQuery.textScalerOf(context);
    final ratio = MediaQuery.devicePixelRatioOf(context);
    return _AlignedInk(
      painter: TextPainter(
        text: TextSpan(text: label, style: defaults.style.merge(style)),
        textAlign: textAlign,
        maxLines: maxLines ?? defaults.maxLines,
        ellipsis: maxLines != null || defaults.overflow == TextOverflow.ellipsis
            ? '…'
            : null,
        textDirection: Directionality.of(context),
        textScaler: scaler,
        locale: Localizations.maybeLocaleOf(context),
        textWidthBasis: defaults.textWidthBasis,
        textHeightBehavior:
            textHeightBehavior ??
            defaults.textHeightBehavior ??
            DefaultTextHeightBehavior.maybeOf(context),
      ),
      metrics: metrics,
      scale: scaler.scale(style.fontSize!) * ratio / (style.fontSize! * 2),
      pixelRatio: ratio,
      topAligned: topAligned,
      lineAdvance: scaler.scale(style.fontSize!) * 1.4,
      fallbackOffset: fallbackOffset,
      child: text,
    );
  }
}

// Preserve native intrinsic/dry layout for Chip and other baseline consumers.
class _AlignedInk extends SingleChildRenderObjectWidget {
  const _AlignedInk({
    required this.painter,
    required this.metrics,
    required this.scale,
    required this.pixelRatio,
    required this.topAligned,
    required this.lineAdvance,
    required this.fallbackOffset,
    required super.child,
  });
  final TextPainter painter;
  final Map<int, (double, double)> metrics;
  final double scale, pixelRatio, lineAdvance;
  final bool topAligned;
  final Offset fallbackOffset;
  @override
  RenderObject createRenderObject(BuildContext context) => _RenderAlignedInk(
    painter,
    metrics,
    scale,
    pixelRatio,
    topAligned,
    lineAdvance,
    fallbackOffset,
  );
  @override
  void updateRenderObject(
    BuildContext context,
    _RenderAlignedInk renderObject,
  ) => renderObject.configure(
    painter,
    metrics,
    scale,
    pixelRatio,
    topAligned,
    lineAdvance,
    fallbackOffset,
  );
}

class _RenderAlignedInk extends RenderProxyBox {
  _RenderAlignedInk(
    this._painter,
    this._metrics,
    this._scale,
    this._pixelRatio,
    this._topAligned,
    this._lineAdvance,
    this._fallbackOffset,
  );
  TextPainter _painter;
  Map<int, (double, double)> _metrics;
  double _scale, _pixelRatio;
  double _top = 0, _height = 0;
  bool _centerInk = false;
  bool _topAligned;
  Offset _fallbackOffset;
  double _lineAdvance;
  final _lineInk = <(double, double)>[];

  void configure(
    TextPainter painter,
    Map<int, (double, double)> metrics,
    double scale,
    double pixelRatio,
    bool topAligned,
    double lineAdvance,
    Offset fallbackOffset,
  ) {
    _painter.dispose();
    _painter = painter;
    _metrics = metrics;
    _scale = scale;
    _pixelRatio = pixelRatio;
    _topAligned = topAligned;
    _lineAdvance = lineAdvance;
    _fallbackOffset = fallbackOffset;
    markNeedsLayout();
    markNeedsSemanticsUpdate();
  }

  (double, double)? bounds(String line) {
    double? top, bottom;
    for (final rune in line.runes) {
      final glyph = _metrics[rune];
      if (glyph == null) return null;
      if (glyph.$1 == glyph.$2) continue;
      top = top == null ? glyph.$1 : math.min(top, glyph.$1);
      bottom = bottom == null ? glyph.$2 : math.max(bottom, glyph.$2);
    }
    return top == null || bottom == null ? null : (top, bottom);
  }

  @override
  void performLayout() {
    super.performLayout();
    _centerInk = false;
    _lineInk.clear();
    _painter.layout(maxWidth: size.width);
    if (_painter.didExceedMaxLines) return;
    final label = _painter.text!.toPlainText();
    final lines = _painter.computeLineMetrics();
    if (lines.isEmpty) return;
    final firstRange = _painter.getLineBoundary(const TextPosition(offset: 0));
    final lastRange = _painter.getLineBoundary(
      TextPosition(offset: label.length, affinity: TextAffinity.upstream),
    );
    final first = bounds(label.substring(firstRange.start, firstRange.end));
    final last = bounds(label.substring(lastRange.start, lastRange.end));
    if (first == null || last == null) return;
    final baseline = _painter.computeDistanceToActualBaseline(
      TextBaseline.alphabetic,
    );
    // Native paragraph paint snaps each baseline before canvas scaling.
    final top = baseline.roundToDouble() * _pixelRatio + first.$1 * _scale;
    final bottom =
        (baseline + lines.last.baseline - lines.first.baseline)
                .roundToDouble() *
            _pixelRatio +
        last.$2 * _scale;
    _top = top / _pixelRatio;
    _height = (bottom - top) / _pixelRatio;
    if (_topAligned && lines.length > 1) {
      for (final line in lines) {
        final range = _painter.getLineBoundary(
          _painter.getPositionForOffset(
            Offset(line.left + line.width / 2, line.baseline - line.ascent / 2),
          ),
        );
        final ink = bounds(label.substring(range.start, range.end).trimRight());
        if (ink == null) {
          _lineInk.clear();
          return;
        }
        final nativeBaseline = (baseline + line.baseline - lines.first.baseline)
            .roundToDouble();
        _lineInk.add((
          nativeBaseline + ink.$1 * _scale / _pixelRatio,
          nativeBaseline + ink.$2 * _scale / _pixelRatio,
        ));
      }
    }
    _centerInk = true;
  }

  // The independent renderer rounds exact half-pixels to the nearest even pixel.
  double roundPixel(double value) {
    final lower = value.floor();
    return value - lower == .5
        ? (lower.isEven ? lower : lower + 1).toDouble()
        : value.roundToDouble();
  }

  Offset get _offset {
    if (!_centerInk) return _fallbackOffset;
    // Fractional paragraph and sheet origins must be rounded in screen space.
    final origin = localToGlobal(Offset.zero).dy;
    final target =
        roundPixel(
          (origin + (_topAligned ? 0 : (size.height - _height) / 2)) *
              _pixelRatio,
        ) /
        _pixelRatio;
    return Offset(0, target - origin - _top);
  }

  @override
  void paint(PaintingContext context, Offset offset) {
    if (child == null) return;
    if (_lineInk.isEmpty) {
      context.paintChild(child!, offset + _offset);
      return;
    }
    // Keep one native paragraph and its semantics. Translate each painted line
    // to the artwork's independent ink top, clipping only between ink bands.
    final origin = localToGlobal(Offset.zero).dy;
    for (var i = 0; i < _lineInk.length; i++) {
      final top = i == 0
          ? math.min(0.0, _lineInk[i].$1) - _lineAdvance
          : (_lineInk[i - 1].$2 + _lineInk[i].$1) / 2;
      final bottom = i == _lineInk.length - 1
          ? math.max(size.height, _lineInk[i].$2) + _lineAdvance
          : (_lineInk[i].$2 + _lineInk[i + 1].$1) / 2;
      final target =
          roundPixel((origin + i * _lineAdvance) * _pixelRatio) / _pixelRatio;
      final shift = Offset(0, target - origin - _lineInk[i].$1);
      context.pushClipRect(
        needsCompositing,
        offset + shift,
        Rect.fromLTRB(-_lineAdvance, top, size.width + _lineAdvance, bottom),
        (context, offset) => context.paintChild(child!, offset),
      );
    }
  }

  @override
  void applyPaintTransform(RenderBox child, Matrix4 transform) {
    transform.translateByDouble(0, _offset.dy, 0, 1);
  }

  @override
  bool hitTestChildren(BoxHitTestResult result, {required Offset position}) =>
      child != null &&
      result.addWithPaintOffset(
        offset: _offset,
        position: position,
        hitTest: (result, position) =>
            child!.hitTest(result, position: position),
      );

  @override
  void dispose() {
    _painter.dispose();
    super.dispose();
  }
}

class _ButtonFace extends CustomPainter {
  const _ButtonFace(this.image, this.pixelRatio);
  final ui.Image image;
  final double pixelRatio;
  @override
  void paint(Canvas canvas, Size size) {
    canvas.save();
    canvas.scale(.5);
    canvas.drawImageNine(
      image,
      const Rect.fromLTRB(30, 30, 31, 31),
      Rect.fromLTWH(
        0,
        0,
        (size.width + 1 / pixelRatio) * 2,
        (size.height + 1 / pixelRatio) * 2,
      ),
      Paint()..filterQuality = FilterQuality.low,
    );
    canvas.restore();
  }

  @override
  bool shouldRepaint(_ButtonFace old) =>
      image != old.image || pixelRatio != old.pixelRatio;
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
            child: Copy(
              value,
              size: 12,
              bold: true,
              center: true,
              control: true,
            ),
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
                        child: Glyph(
                          increase ? 'step-plus' : 'step-minus',
                          size: 16,
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
        Expanded(child: Copy(label, control: true)),
        Copy(displayValue, bold: true, control: true),
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
                    Positioned(
                      top: 0,
                      left: 16,
                      right: 16,
                      height: 44,
                      child: Center(child: heading()),
                    ),
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
          child: LayoutBuilder(
            builder: (context, constraints) {
              final scaler = MediaQuery.textScalerOf(context);
              final text = TextPainter(
                text: TextSpan(
                  text: label,
                  style: TextStyle(
                    fontFamily: 'Inter',
                    fontSize: 13,
                    height: 1.4,
                    letterSpacing: 0,
                    wordSpacing: 0,
                    fontWeight: selected || MediaQuery.highContrastOf(context)
                        ? FontWeight.w600
                        : FontWeight.w400,
                  ),
                ),
                textDirection: Directionality.of(context),
                textScaler: scaler,
              )..layout(maxWidth: math.max(0, constraints.maxWidth - 60));
              // Keep fractional prototype line spacing despite paragraph rounding.
              final height =
                  34 +
                  text.computeLineMetrics().length * scaler.scale(13) * 1.4;
              text.dispose();
              return ConstrainedBox(
                constraints: BoxConstraints(minHeight: height),
                child: Padding(
                  padding: const EdgeInsets.fromLTRB(15, 17, 16, 17),
                  child: Row(
                    children: [
                      Expanded(
                        child: Copy(label, bold: selected, control: true),
                      ),
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
                            ? const Center(
                                child: Glyph(
                                  'check',
                                  size: 14,
                                  color: Colors.white,
                                ),
                              )
                            : null,
                      ),
                    ],
                  ),
                ),
              );
            },
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
      painter: _GlyphPainter(
        name,
        readableInk(context, color),
        MediaQuery.devicePixelRatioOf(context),
      ),
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
  _GlyphPainter(this.name, this.color, this.pixelRatio);
  final String name;
  final Color color;
  final double pixelRatio;
  @override
  void paint(Canvas canvas, Size size) {
    final atlas = _glyphAtlas;
    final region = _glyphRegions[(name, size.width)];
    if (atlas != null && region != null) {
      final transform = canvas.getTransform();
      double snap(double value) {
        final scaled = value * pixelRatio;
        final floor = scaled.floor();
        final rounded = scaled - floor == .5
            ? (floor.isEven ? floor : floor + 1).toDouble()
            : scaled.roundToDouble();
        return rounded / pixelRatio - value;
      }

      canvas.save();
      if (transform[0] == 1 &&
          transform[5] == 1 &&
          transform[1] == 0 &&
          transform[4] == 0) {
        canvas.translate(snap(transform[12]), snap(transform[13]));
      }
      canvas.drawImageRect(
        atlas,
        region,
        Rect.fromLTWH(-2, -2, region.width / 2, region.height / 2),
        Paint()
          ..filterQuality = FilterQuality.low
          ..colorFilter = ColorFilter.mode(color, BlendMode.srcIn),
      );
      canvas.restore();
      return;
    }
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

    void rect(double x, double y, double w, double h, double r) {
      // The reference outlines sit inside their supplied bounds.
      final outline = Paint()
        ..color = color
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.6;
      canvas.drawRRect(
        RRect.fromRectAndRadius(
          Rect.fromLTWH(x, y, w, h).deflate(.8),
          Radius.circular(math.max(0, r - .8)),
        ),
        outline,
      );
    }

    void arc(double width, double start, double sweep) {
      canvas.drawArc(
        const Rect.fromLTWH(3, 3, 18, 18).deflate(width / 2),
        start * math.pi / 180,
        sweep * math.pi / 180,
        false,
        Paint()
          ..color = color
          ..style = PaintingStyle.stroke
          ..strokeWidth = width,
      );
    }

    switch (name) {
      case 'drag':
        p.style = PaintingStyle.fill;
        // The three reference strokes include both horizontal endpoints.
        for (final y in [3.0, 11.0, 19.0]) {
          canvas.drawRect(
            Rect.fromLTWH(0, y * 12 / 11, 24 + 12 / 11, 24 / 11),
            p,
          );
        }
      case 'back':
        line(const [Offset(15, 4), Offset(7, 12), Offset(15, 20)]);
      case 'step-plus':
      case 'step-minus':
        final pen = Paint()
          ..color = color
          ..strokeWidth = 2;
        canvas.drawLine(const Offset(4, 12), const Offset(20, 12), pen);
        if (name == 'step-plus') {
          canvas.drawLine(const Offset(12, 4), const Offset(12, 20), pen);
        }
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
        arc(1.7, 40, 280);
        line(const [Offset(19, 1), Offset(21, 7), Offset(15, 7)]);
      case 'power':
        arc(1.9, 310, 280);
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
      name != old.name || color != old.color || pixelRatio != old.pixelRatio;
}

class IconAction extends StatelessWidget {
  const IconAction(
    this.icon,
    this.label,
    this.onTap, {
    this.color = ink,
    this.size = 24,
    super.key,
  });
  final String icon, label;
  final VoidCallback onTap;
  final Color color;
  final double size;
  @override
  Widget build(BuildContext context) => IconButton(
    tooltip: label,
    padding: EdgeInsets.zero,
    onPressed: onTap,
    icon: Glyph(icon, color: color, size: size),
    constraints: const BoxConstraints(minWidth: 48, minHeight: 48),
  );
}
