import 'dart:convert';
import 'controller.dart';
import 'controls.dart';
import 'client_contract.dart';

bool _validChoiceValue(Object? value, Json schema) {
  if (schema['type'] != null &&
      !(switch (schema['type']) {
        'string' => value is String,
        'boolean' => value is bool,
        'number' => value is num && value.isFinite,
        'integer' =>
          value is num && value.isFinite && value == value.roundToDouble(),
        _ => false,
      }))
    return false;
  if (schema['multipleOf'] case final num step) {
    if (!step.isFinite || step <= 0) return false;
  }
  return validateValue(value, schema) == null;
}

/// One independently settable scalar. Descriptors supply all values and bounds.
class ClimateChoice {
  ClimateChoice(this.capability, this.action, this.parameter, this.schema)
    : signature = jsonEncode([capability, action, parameter]);
  final Json capability, schema;
  final String action, parameter, signature;
  String get property => '${capability['id']}.$parameter';
  bool get numeric => schema['enum'] == null && schema['type'] != 'boolean';
  late final List<Object> values = List.unmodifiable(_values());
  List<Object> _values() {
    if (schema['enum'] case final List values) return values.cast<Object>();
    if (schema['type'] == 'boolean') return [false, true];
    final min = schema['minimum'] as num, step = schema['multipleOf'] as num;
    return [
      for (var i = 0; i <= ((schema['maximum'] - min) / step).round(); i++)
        min + i * step,
    ];
  }

  List<Object> available(HomeController c) =>
      values.where((v) => permits(c, v)).toList();
  bool permits(HomeController c, Object value) =>
      values.contains(value) &&
      c.canControlValue(capability['id'], action, parameter, value);
}

ClimateChoice? climateChoice(
  HomeController controller,
  Set<String> names, {
  Set<String>? capabilityIds,
}) {
  final candidates = <ClimateChoice>[];
  for (final capability in controller.capabilities) {
    if (capabilityIds != null && !capabilityIds.contains(capability['id']))
      continue;
    for (final entry in (capability['actions'] as Json).entries) {
      final action = entry.value as Json;
      final parameters = action['parameters'] as Json;
      if (action['risk'] != 'low' ||
          action['idempotency'] != 'set_value' ||
          actionUnavailable(capability, action) != null ||
          parameters['type'] != 'object' ||
          parameters.keys.any(
            (key) => !{
              'type',
              'properties',
              'required',
              'minProperties',
              'maxProperties',
              'additionalProperties',
              'description',
              'title',
            }.contains(key),
          ) ||
          (parameters['minProperties'] as num? ?? 0) > 1 ||
          (parameters['maxProperties'] as num? ?? 1) < 1)
        continue;
      for (final field in (parameters['properties'] as Json).entries) {
        final schema = field.value as Json;
        if (!names.contains(field.key) ||
            (parameters['required'] as List? ?? []).any(
              (key) => key != field.key,
            ) ||
            schema.keys.any(
              (key) => !{
                'type',
                'enum',
                'minimum',
                'maximum',
                'multipleOf',
                'description',
                'title',
              }.contains(key),
            ))
          continue;
        if (schema['enum'] case final List values) {
          if (values.isEmpty ||
              values.length > 64 ||
              values.any((v) => v is! String && v is! bool && v is! num) ||
              values.any((v) => !_validChoiceValue(v, schema)))
            continue;
        } else if (schema['type'] != 'boolean') {
          if (!['integer', 'number'].contains(schema['type'])) continue;
          final min = schema['minimum'],
              max = schema['maximum'],
              step = schema['multipleOf'];
          if (min is! num ||
              max is! num ||
              step is! num ||
              !min.isFinite ||
              !max.isFinite ||
              !step.isFinite ||
              step <= 0 ||
              max <= min ||
              (max - min) / step > 1000 ||
              (max - min) / step != ((max - min) / step).roundToDouble() ||
              validateValue(min, schema) != null ||
              validateValue(max, schema) != null)
            continue;
        }
        candidates.add(ClimateChoice(capability, entry.key, field.key, schema));
      }
    }
  }
  // Ambiguity is never resolved using brand, list order or current authority.
  if (candidates.length != 1) return null;
  final choice = candidates.single;
  return choice.available(controller).isEmpty ? null : choice;
}
