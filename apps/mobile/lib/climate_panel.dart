import 'package:flutter/material.dart';
import 'controls.dart';
import 'localization.dart';
import 'skin.dart';

const climateBlue = Color(0xff087fa9);

class ClimateSurface extends StatelessWidget {
  const ClimateSurface({required this.child, super.key});
  final Widget child;
  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.all(20),
    decoration: BoxDecoration(
      color: Colors.white,
      borderRadius: BorderRadius.circular(24),
      border: MediaQuery.highContrastOf(context)
          ? Border.all(color: Colors.black)
          : null,
      boxShadow: const [
        BoxShadow(
          color: Color(0x060c3c52),
          blurRadius: 16,
          offset: Offset(0, 5),
        ),
      ],
    ),
    child: child,
  );
}

class ClimateHero extends StatelessWidget {
  const ClimateHero({required this.power, required this.indoor, super.key});
  final Object? power, indoor;
  @override
  Widget build(BuildContext context) => Column(
    children: [
      const SizedBox(height: 6),
      ExcludeSemantics(
        child: SizedBox(
          height: 100,
          width: 240,
          child: CustomPaint(painter: _AppliancePainter(power == true)),
        ),
      ),
      Wrap(
        alignment: WrapAlignment.center,
        crossAxisAlignment: WrapCrossAlignment.center,
        spacing: 18,
        runSpacing: 8,
        children: [
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                Icons.circle,
                size: 6,
                color: power == true ? climateBlue : muted,
              ),
              const SizedBox(width: 7),
              Copy(
                power is bool
                    ? (power == true ? 'Power on' : 'Power off')
                    : 'Power unknown',
                size: 12,
                color: muted,
              ),
            ],
          ),
          Copy(
            UiMessage('Indoor {temperature}', {
              'temperature': UiMessage(
                propertyValue(indoor, {'unit': 'celsius'}),
              ),
            }),
            size: 12,
            color: muted,
          ),
        ],
      ),
      const SizedBox(height: 24),
    ],
  );
}

/// Original vector illustration, independent of vendor artwork or device models.
class _AppliancePainter extends CustomPainter {
  const _AppliancePainter(this.on);
  final bool on;
  @override
  void paint(Canvas canvas, Size size) {
    final body = RRect.fromRectAndRadius(
      const Rect.fromLTWH(15, 6, 210, 58),
      const Radius.circular(12),
    );
    canvas.drawShadow(
      Path()..addRRect(body),
      const Color(0xff416f82),
      10,
      false,
    );
    if (on) {
      final air = Path()
        ..moveTo(28, 61)
        ..lineTo(212, 61)
        ..lineTo(234, 100)
        ..lineTo(6, 100)
        ..close();
      canvas.drawPath(
        air,
        Paint()
          ..shader = const LinearGradient(
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
            colors: [Color(0x3611a8dc), Color(0x0011a8dc)],
          ).createShader(const Rect.fromLTWH(0, 61, 240, 39)),
      );
    }
    canvas.drawRRect(
      body,
      Paint()
        ..shader = const LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [Colors.white, Color(0xffe3eaf0)],
        ).createShader(body.outerRect),
    );
    canvas.drawRRect(
      body,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1
        ..color = const Color(0xffd2dce3),
    );
    final vent = RRect.fromRectAndRadius(
      Rect.fromLTWH(26, 51, 188, on ? 9 : 3),
      const Radius.circular(3),
    );
    canvas.drawRRect(vent, Paint()..color = const Color(0xff9baebc));
    canvas.drawLine(
      const Offset(30, 56),
      const Offset(210, 56),
      Paint()
        ..color = const Color(0xffdae3e9)
        ..strokeWidth = 2,
    );
    canvas.drawCircle(
      const Offset(200, 37),
      2,
      Paint()..color = on ? climateBlue : const Color(0xffb4c1c9),
    );
  }

  @override
  bool shouldRepaint(_AppliancePainter oldDelegate) => oldDelegate.on != on;
}

class ClimateTile extends StatelessWidget {
  const ClimateTile({
    required this.title,
    required this.value,
    required this.icon,
    required this.onPressed,
    this.active = false,
    super.key,
  });
  final String title;
  final Object value;
  final Widget icon;
  final VoidCallback? onPressed;
  final bool active;
  @override
  Widget build(BuildContext context) => OutlinedButton(
    style: OutlinedButton.styleFrom(
      padding: const EdgeInsets.all(16),
      minimumSize: const Size(0, 100),
      backgroundColor: active ? const Color(0xffe3f3fa) : Colors.white,
      side: BorderSide(
        color: MediaQuery.highContrastOf(context)
            ? Colors.black
            : active
            ? const Color(0xffa9dae9)
            : Colors.transparent,
      ),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(22)),
    ),
    onPressed: onPressed,
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            Expanded(child: Copy(title, bold: true, color: ink)),
            const SizedBox(width: 6),
            icon,
          ],
        ),
        const SizedBox(height: 12),
        Copy(value, size: 12, color: muted),
      ],
    ),
  );
}

class ClimateGrid extends StatelessWidget {
  const ClimateGrid({required this.children, super.key});
  final List<Widget> children;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) {
      final columns =
          constraints.maxWidth >= 300 &&
              MediaQuery.textScalerOf(context).scale(14) <= 24
          ? 2
          : 1;
      return Wrap(
        spacing: 12,
        runSpacing: 12,
        children: [
          for (final child in children)
            SizedBox(
              width: (constraints.maxWidth - (columns - 1) * 12) / columns,
              child: child,
            ),
        ],
      );
    },
  );
}
