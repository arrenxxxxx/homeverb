import 'package:flutter/material.dart';

import 'client_contract.dart';
import 'controls.dart';
import 'localization.dart';

/// Card copy is derived from saved observations, never from requested values.
/// It neither infers connectivity nor changes a device's control permissions.
({Object value, Object caption}) deviceSummary(
  BuildContext context,
  Json device,
  Json? Function(String component) snapshotFor,
) {
  final parts = (device['components'] as List? ?? []).cast<Json>();
  dynamic observed(String component, String property) =>
      snapshotFor(component)?['observations']?[property]?['value'];
  String temperature(dynamic value) => value is num && value.isFinite
      ? '${value.toString().replaceFirst(RegExp(r'\.0$'), '')}°C'
      : '—';
  String label(Object value) => localize(context, value);
  final climateParts = parts
      .where(
        (part) => (part['capabilities'] as List? ?? []).contains('climate'),
      )
      .toList();
  final hasZones =
      climateParts.length > 1 ||
      climateParts.any((part) => ['fridge', 'freezer'].contains(part['id']));
  if (hasZones) {
    final zones = <String>[];
    String firstValue = '—';
    for (final part in climateParts) {
      final id = part['id'] as String;
      final target = observed(id, 'climate.target_temperature');
      final current = observed(id, 'climate.current_temperature');
      final missing = target == null && current == null;
      final useCurrent = target == null && current != null;
      final formatted = temperature(useCurrent ? current : target);
      if (zones.isEmpty) firstValue = formatted;
      if (missing) {
        final name = label(UiMessage(controlLabel(id)));
        zones.add(zones.isEmpty ? name : '$name —');
        continue;
      }
      zones.add(
        label(
          UiMessage(
            zones.isEmpty
                ? (useCurrent ? '{component} current' : '{component} target')
                : (useCurrent
                      ? '{component} current {value}'
                      : '{component} target {value}'),
            {
              'component': UiMessage(controlLabel(id)),
              'value': temperature(useCurrent ? current : target),
            },
          ),
        ),
      );
    }
    return (value: firstValue, caption: LiteralText(zones.join(' · ')));
  }
  for (final part in parts) {
    final id = part['id'] as String;
    final capabilities = (part['capabilities'] as List? ?? []).cast<String>();
    final dishwasher = capabilities
        .where((cap) => cap.endsWith('.dishwasher'))
        .firstOrNull;
    if (dishwasher != null) {
      final program = observed(id, '$dishwasher.mode');
      final phase = observed(id, '$dishwasher.work_status');
      final minutes = observed(id, '$dishwasher.remaining_minutes');
      String known(String property, dynamic value) {
        if (value == null) return '—';
        return label(
          UiMessage(sourceForKey('$dishwasher.$property.$value') ?? 'Unknown'),
        );
      }

      final details = <String>[
        if (phase != null) known('work_status', phase),
        if (minutes is num && minutes.isFinite && minutes >= 0)
          label(UiMessage('{value} min', {'value': minutes})),
      ];
      return (
        value: LiteralText(known('program', program)),
        caption: details.isEmpty ? '—' : LiteralText(details.join(' · ')),
      );
    }
  }
  if (climateParts.length == 1) {
    final id = climateParts.single['id'] as String;
    final mode = observed(id, 'climate.mode');
    return (
      value: temperature(observed(id, 'climate.target_temperature')),
      caption: mode == null
          ? '—'
          : UiMessage(sourceForKey('climate.mode.$mode') ?? 'Unknown'),
    );
  }
  return (value: '—', caption: '');
}
