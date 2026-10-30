import 'package:flutter/material.dart';
import 'skin.dart';
import 'device_typography.dart';

/// Read-only evidence has no elevation, gesture handler or control semantics.
class DeviceReadingRow extends StatelessWidget {
  const DeviceReadingRow(
    this.label,
    this.value, {
    this.valueColor,
    this.progress,
    super.key,
  });
  final Color? valueColor;
  final double? progress;
  final Object label, value;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 10),
    child: LayoutBuilder(
      builder: (context, constraints) {
        final compact =
            constraints.maxWidth < 260 ||
            MediaQuery.textScalerOf(context).scale(14) > 22;
        return compact
            ? Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Copy(
                    label,
                    size: DeviceTypography.reading,
                    lineHeight: DeviceTypography.lineHeight,
                    color: muted,
                  ),
                  const SizedBox(height: 6),
                  Copy(
                    value,
                    size: DeviceTypography.reading,
                    lineHeight: DeviceTypography.lineHeight,
                    bold: true,
                    color: valueColor ?? ink,
                  ),
                ],
              )
            : Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Expanded(
                    flex: 3,
                    child: Copy(
                      label,
                      size: DeviceTypography.reading,
                      lineHeight: DeviceTypography.lineHeight,
                      color: muted,
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    flex: 2,
                    child: Align(
                      alignment: AlignmentDirectional.centerEnd,
                      child: progress == null
                          ? Copy(
                              value,
                              size: DeviceTypography.reading,
                              lineHeight: DeviceTypography.lineHeight,
                              bold: true,
                              color: valueColor ?? ink,
                            )
                          : Row(
                              children: [
                                Copy(
                                  value,
                                  size: DeviceTypography.reading,
                                  lineHeight: DeviceTypography.lineHeight,
                                  bold: true,
                                ),
                                const SizedBox(width: 8),
                                Expanded(
                                  child: LinearProgressIndicator(
                                    value: progress,
                                    minHeight: 7,
                                    borderRadius: BorderRadius.circular(6),
                                    color: accent,
                                    backgroundColor: const Color(0xffddddda),
                                  ),
                                ),
                              ],
                            ),
                    ),
                  ),
                ],
              );
      },
    ),
  );
}

class DeviceReadings extends StatelessWidget {
  const DeviceReadings({required this.children, super.key});
  final List<Widget> children;

  @override
  Widget build(BuildContext context) => DecoratedBox(
    decoration: BoxDecoration(
      color: Colors.white,
      borderRadius: BorderRadius.circular(14),
      border: Border.all(
        color: MediaQuery.highContrastOf(context)
            ? Colors.black
            : const Color(0xfff0eee9),
      ),
    ),
    child: Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          for (var i = 0; i < children.length; i++) ...[
            if (i > 0) const Divider(height: 1),
            children[i],
          ],
        ],
      ),
    ),
  );
}
