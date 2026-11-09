import 'package:flutter/material.dart';
import 'localization.dart';
import 'skin.dart';
import 'device_typography.dart';

/// Keeps each unresolved reading local to its value position.
class ReadingValue extends StatelessWidget {
  const ReadingValue(
    this.value, {
    this.loading = false,
    this.size = DeviceTypography.reading,
    this.color,
    this.bold = false,
    this.center = false,
    super.key,
  });
  final Object value;
  final bool loading, bold, center;
  final double size;
  final Color? color;

  @override
  Widget build(BuildContext context) => loading
      ? Semantics(
          label: localize(context, 'Loading'),
          child: SizedBox(
            width: 18,
            height: 18,
            child: CircularProgressIndicator(
              strokeWidth: 2,
              color: color ?? skinAccent(context),
            ),
          ),
        )
      : Copy(
          value,
          size: size,
          color: color ?? ink,
          bold: bold,
          center: center,
          lineHeight: DeviceTypography.lineHeight,
        );
}

/// Read-only evidence has no elevation, gesture handler or control semantics.
class DeviceReadingRow extends StatelessWidget {
  const DeviceReadingRow(
    this.label,
    this.value, {
    this.valueColor,
    this.progress,
    this.loading = false,
    super.key,
  });
  final Color? valueColor;
  final double? progress;
  final bool loading;
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
                  ReadingValue(
                    value,
                    size: DeviceTypography.reading,
                    loading: loading,
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
                      child: progress == null || loading
                          ? ReadingValue(
                              value,
                              size: DeviceTypography.reading,
                              loading: loading,
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
                                    color: skinAccent(context),
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
