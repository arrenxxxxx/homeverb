import 'package:flutter/material.dart';
import 'controller.dart';
import 'native_client.dart';
import 'receipts.dart';
import 'skin.dart';

String controlLabel(String id) {
  final words = id.replaceAll('_', ' ');
  return words.isEmpty ? id : '${words[0].toUpperCase()}${words.substring(1)}';
}

String controlUnit(dynamic unit) =>
    const {'celsius': '°C', 'percent': '%', 'kelvin': 'K'}[unit] ??
    (unit?.toString() ?? '');

String propertyValue(dynamic value, Json property) {
  if (value == null) return 'Unknown';
  final text = value is bool ? (value ? 'On' : 'Off') : '$value';
  final unit = controlUnit(property['unit']);
  return '$text${unit.isEmpty
      ? ''
      : ['°C', '%'].contains(unit)
      ? unit
      : ' $unit'}';
}

String? actionUnavailable(Json capability, Json action) {
  if (action['risk'] != 'low') return 'Trusted approval is unavailable.';
  final parameters = action['parameters'] as Json;
  final properties = parameters['properties'] as Json? ?? {};
  if (parameters['type'] != 'object') return 'This form is not supported.';
  for (final entry in properties.entries) {
    final schema = entry.value;
    if (schema is! Map ||
        (schema['enum'] is! List &&
            ![
              'number',
              'integer',
              'string',
              'boolean',
            ].contains(schema['type']))) {
      return 'This form is not supported.';
    }
    if (schema['enum'] is List &&
        (schema['enum'] as List).any(
          (v) => v is! num && v is! String && v is! bool,
        )) {
      return 'This form is not supported.';
    }
    if (capability['properties']?[entry.key]?['writable'] == false) {
      return 'This property is read-only.';
    }
  }
  return null;
}

List<dynamic> describedControls(HomeController c) {
  final result = <dynamic>[];
  for (var i = 0; i < c.capabilities.length; i++) {
    final capability = c.capabilities[i], id = capability['id'] as String;
    // The climate panel and power row already expose these common controls.
    if (id == 'power') continue;
    final properties = capability['properties'] as Json;
    final actions = capability['actions'] as Json;
    final extraClimate =
        id == 'climate' &&
        (actions.keys.any((a) => a != 'set') ||
            ((actions['set']?['parameters']?['properties'] as Map?) ?? {}).keys
                .any((p) => !['target_temperature', 'mode'].contains(p)));
    if (id == 'climate' && !extraClimate) continue;
    result.add([
      'card',
      controlLabel(id),
      [
        for (final entry in properties.entries)
          if (entry.value['readable'] == true)
            [
              controlLabel(entry.key),
              "${propertyValue(c.observed('$id.${entry.key}'), entry.value as Json)}${c.snapshot?['observations']?['$id.${entry.key}']?['stale'] == true ? ' · Stale' : ''}",
              null,
            ],
        for (final entry in actions.entries)
          [
            controlLabel(entry.key),
            actionUnavailable(capability, entry.value as Json) ??
                'Choose values  ›',
            actionUnavailable(capability, entry.value as Json) == null
                ? 'control:$i:${Uri.encodeComponent(entry.key)}'
                : null,
          ],
      ],
    ]);
  }
  return result;
}

class CapabilityActionForm extends StatefulWidget {
  const CapabilityActionForm({
    required this.capability,
    required this.action,
    required this.observation,
    required this.onSubmit,
    required this.onCancel,
    required this.error,
    super.key,
  });
  final Json capability;
  final String action;
  final Json? Function(String) observation;
  final Future<bool> Function(Json) onSubmit;
  final VoidCallback onCancel;
  final Object? Function() error;
  @override
  State<CapabilityActionForm> createState() => _CapabilityActionFormState();
}

class _CapabilityActionFormState extends State<CapabilityActionForm> {
  final fields = <String, TextEditingController>{};
  final selected = <String, dynamic>{};
  String? message;
  bool saving = false;
  Json get descriptor => widget.capability['actions'][widget.action] as Json;
  Json get parameters => descriptor['parameters'] as Json;
  Json get properties => parameters['properties'] as Json? ?? {};
  @override
  void dispose() {
    for (final field in fields.values) {
      field.dispose();
    }
    super.dispose();
  }

  Future<void> submit() async {
    if (saving) return;
    final args = <String, dynamic>{...selected};
    for (final entry in fields.entries) {
      if (entry.value.text.isEmpty) continue;
      final schema = properties[entry.key] as Json;
      args[entry.key] = ['number', 'integer'].contains(schema['type'])
          ? num.tryParse(entry.value.text)
          : entry.value.text;
    }
    String? error = actionUnavailable(widget.capability, descriptor);
    for (final name in parameters['required'] as List? ?? []) {
      if (!args.containsKey(name)) {
        error ??= 'Choose ${controlLabel(name)}.';
      }
    }
    if (args.length < (parameters['minProperties'] as num? ?? 0)) {
      error ??= 'Choose at least ${parameters['minProperties']} value(s).';
    }
    for (final entry in args.entries) {
      final problem = validateValue(entry.value, properties[entry.key] as Json);
      if (problem != null) error ??= '${controlLabel(entry.key)}: $problem';
    }
    if (error != null) {
      setState(() => message = error);
      return;
    }
    setState(() {
      saving = true;
      message = null;
    });
    final success = await widget.onSubmit(args);
    if (!mounted) return;
    setState(() {
      saving = false;
      if (!success)
        message = '${widget.error() ?? 'Operation was not completed.'}';
    });
  }

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      const Copy(
        'Choose only the values to send. Observations remain unchanged until a read.',
      ),
      const SizedBox(height: 16),
      for (final entry in properties.entries)
        parameter(entry.key, entry.value as Json),
      if (properties.isEmpty) const Copy('This action has no parameters.'),
      if (descriptor['idempotency'] == 'non_idempotent')
        const Copy(
          'This action may have lasting effects. An uncertain result must not be blindly repeated.',
        ),
      if (message != null)
        Padding(
          padding: const EdgeInsets.only(bottom: 12),
          child: Copy(message!, color: const Color(0xff9e3421)),
        ),
      ActionButton('Apply', onPressed: saving ? null : submit),
      ActionButton('Cancel', style: 'secondary', onPressed: widget.onCancel),
    ],
  );

  Widget parameter(String name, Json schema) {
    final property = widget.capability['properties']?[name] as Json? ?? {};
    final observation = widget.observation('${widget.capability['id']}.$name');
    final observed = observation?['value'];
    final options =
        schema['enum'] as List? ??
        (schema['type'] == 'boolean' ? [true, false] : null);
    final required = (parameters['required'] as List? ?? []).contains(name);
    final controller = options == null
        ? fields.putIfAbsent(name, () => TextEditingController())
        : null;
    final numeric = ['number', 'integer'].contains(schema['type']);
    final lower = schema['minimum'], upper = schema['maximum'];
    final draft = controller == null ? null : num.tryParse(controller.text);
    return Padding(
      padding: const EdgeInsets.only(bottom: 16),
      child: Surface(
        radius: 18,
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Copy(
              '${controlLabel(name)}${required ? ' · Required' : ' · Optional'}',
              bold: true,
            ),
            const SizedBox(height: 8),
            Copy(
              'Observed: ${propertyValue(observed, property)}',
              size: 12,
              color: muted,
            ),
            if (observation != null)
              Copy(
                'Read: ${receiptTime(observation['received_at'], compact: true)}${observation['stale'] == true ? ' · Stale' : ''}',
                size: 12,
                color: muted,
              ),
            if (numeric)
              Copy(
                [
                  if (lower != null) 'Minimum $lower',
                  if (upper != null) 'Maximum $upper',
                  if (schema['multipleOf'] != null)
                    'Step ${schema['multipleOf']}',
                ].join(' · '),
                size: 12,
                color: muted,
              ),
            const SizedBox(height: 8),
            if (options != null)
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  for (final value in options)
                    ChoiceChip(
                      key: ValueKey('parameter:$name:$value'),
                      backgroundColor: const Color(0xfff8f5ee),
                      selectedColor: ink,
                      side: BorderSide.none,
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(16),
                      ),
                      showCheckmark: false,
                      label: Copy(
                        propertyValue(value, {}),
                        size: 12,
                        color:
                            selected.containsKey(name) &&
                                selected[name] == value
                            ? Colors.white
                            : muted,
                      ),
                      selected:
                          selected.containsKey(name) && selected[name] == value,
                      onSelected: saving
                          ? null
                          : (on) => setState(() {
                              if (on) {
                                selected[name] = value;
                              } else {
                                selected.remove(name);
                              }
                              message = null;
                            }),
                    ),
                ],
              )
            else
              TextField(
                key: ValueKey('parameter:$name'),
                controller: controller,
                enabled: !saving,
                keyboardType: numeric
                    ? const TextInputType.numberWithOptions(
                        decimal: true,
                        signed: true,
                      )
                    : TextInputType.text,
                style: const TextStyle(fontSize: 15, height: 4 / 3, color: ink),
                decoration: InputDecoration(
                  filled: true,
                  fillColor: const Color(0xfff8f5ee),
                  border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(14),
                    borderSide: BorderSide.none,
                  ),
                  contentPadding: const EdgeInsets.all(15),
                  hintText: 'Enter ${controlLabel(name).toLowerCase()}',
                  suffixText: controlUnit(property['unit']),
                ),
                onChanged: (_) => setState(() => message = null),
              ),
            if (numeric &&
                lower is num &&
                upper is num &&
                upper > lower &&
                draft != null &&
                validateValue(draft, schema) == null)
              SliderTheme(
                data: SliderTheme.of(context).copyWith(
                  trackHeight: 5,
                  activeTrackColor: accent,
                  inactiveTrackColor: const Color(0xffd5d6cf),
                  thumbShape: const WindowSliderThumb(),
                  padding: const EdgeInsets.symmetric(horizontal: 20),
                  trackShape: const RoundedRectSliderTrackShape(),
                ),
                child: Slider(
                  key: ValueKey('parameter-slider:$name'),
                  value: draft.toDouble(),
                  min: lower.toDouble(),
                  max: upper.toDouble(),
                  semanticFormatterCallback: (v) =>
                      '${controlLabel(name)} $v ${property['unit'] ?? ''}',
                  onChanged: saving
                      ? null
                      : (value) => setState(() {
                          final step =
                              schema['multipleOf'] as num? ??
                              (schema['type'] == 'integer' ? 1 : null);
                          final next = step == null
                              ? value
                              : (value / step).round() * step;
                          if (next < lower || next > upper) return;
                          controller!.text = next.toString();
                          message = null;
                        }),
                ),
              ),
          ],
        ),
      ),
    );
  }
}
