import 'package:flutter/material.dart';
import 'localization.dart';
import 'skin.dart';

/// Shared Window Light material, also used by the other application pages.
class ClimateSurface extends StatelessWidget {
  const ClimateSurface({required this.child, this.dark = false, super.key});
  final Widget child;
  final bool dark;
  @override
  Widget build(BuildContext context) =>
      Surface(dark: dark, padding: const EdgeInsets.all(20), child: child);
}

class ClimateRow extends StatelessWidget {
  const ClimateRow({
    required this.title,
    required this.icon,
    this.value,
    this.child,
    this.trailing,
    this.updating = false,
    super.key,
  });
  final String title;
  final IconData icon;
  final Object? value;
  final Widget? child, trailing;
  final bool updating;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 14),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Row(
                    children: [
                      if (updating)
                        Semantics(
                          liveRegion: true,
                          label: localize(context, 'Updating device…'),
                          child: const OperationPulse(size: 22),
                        )
                      else
                        Icon(icon, color: ink, size: 22),
                      const SizedBox(width: 10),
                      Expanded(child: Copy(title, bold: true)),
                    ],
                  ),
                  if (value case final value?) ...[
                    const SizedBox(height: 8),
                    Copy(value, size: 12, color: muted),
                  ],
                ],
              ),
            ),
            if (trailing != null) ...[const SizedBox(width: 8), trailing!],
          ],
        ),
        if (child != null) ...[const SizedBox(height: 8), child!],
      ],
    ),
  );
}

class ClimateGroup extends StatelessWidget {
  const ClimateGroup({required this.children, super.key});
  final List<Widget> children;
  @override
  Widget build(BuildContext context) => ClimateSurface(
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        for (var i = 0; i < children.length; i++) ...[
          if (i > 0) const Divider(height: 1),
          children[i],
        ],
      ],
    ),
  );
}
