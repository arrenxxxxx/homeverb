import 'package:flutter/material.dart';

import 'device_typography.dart';

/// Balanced, compact single-choice rows without shrinking labels or hit areas.
/// Chip callers should expand their label to the available width so that the
/// painted face, not only the padded hit area, fills each cell.
class DeviceOptionGrid extends StatelessWidget {
  const DeviceOptionGrid({
    super.key,
    required this.labels,
    required this.children,
    this.fontSize = DeviceTypography.body,
  }) : assert(labels.length == children.length);

  final List<String> labels;
  final List<Widget> children;
  final double fontSize;

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) {
      if (children.isEmpty) return const SizedBox.shrink();
      if (!constraints.hasBoundedWidth) {
        return Wrap(spacing: 8, runSpacing: 0, children: children);
      }
      var minimumWidth = 48.0;
      for (final label in labels) {
        final painter = TextPainter(
          text: TextSpan(
            text: label,
            style: DefaultTextStyle.of(context).style.copyWith(
              fontSize: fontSize,
              height: DeviceTypography.lineHeight,
            ),
          ),
          textDirection: Directionality.of(context),
          textScaler: MediaQuery.textScalerOf(context),
        )..layout();
        // Reserve the selection mark and compact padding for every value.
        final width = painter.width + 40;
        if (width > minimumWidth) minimumWidth = width;
        painter.dispose();
      }
      final capacity = ((constraints.maxWidth + 8) / (minimumWidth + 8))
          .floor()
          .clamp(1, 3);
      final rows = (children.length / capacity).ceil();
      final columns = (children.length / rows).ceil();
      final width = (constraints.maxWidth - (columns - 1) * 8) / columns;
      return Wrap(
        spacing: 8,
        runSpacing: 0,
        children: [
          for (final child in children)
            SizedBox(
              width: width,
              child: ConstrainedBox(
                constraints: const BoxConstraints(minHeight: 48),
                child: child,
              ),
            ),
        ],
      );
    },
  );
}
