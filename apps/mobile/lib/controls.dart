import 'localization.dart';
import 'package:flutter/material.dart';
import 'controller.dart';
import 'client_contract.dart';
import 'receipts.dart';
import 'skin.dart';

String controlLabel(String id) {
  final shared = sourceForKey(id);
  if (shared != null) return shared;
  final words = id.replaceAll('_', ' ');
  return words.isEmpty ? id : '${words[0].toUpperCase()}${words.substring(1)}';
}

String controlUnit(dynamic unit) =>
    const {'celsius': '°C', 'percent': '%', 'kelvin': 'K'}[unit] ??
    (unit?.toString() ?? '');

String propertyValue(dynamic value, Json property, {String? propertyId}) {
  final label = propertyId == null ? null : sourceForKey('$propertyId.$value');
  if (label != null) return label;
  if (value == null) return 'Unknown';
  final text = value is bool ? (value ? 'On' : 'Off') : '$value';
  final unit = controlUnit(property['unit']);
  return '$text${unit.isEmpty
      ? ''
      : ['°C', '%'].contains(unit)
      ? unit
      : ' $unit'}';
}

String? actionUnavailable(
  Json capability,
  Json action, {
  bool trustedApproval = false,
}) {
  if (action['risk'] != 'low' &&
      !(action['risk'] == 'elevated' && trustedApproval))
    return 'Trusted approval is unavailable.';
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

List<dynamic> describedControls(
  HomeController c, {
  Set<String> exclude = const {},
  Set<String> excludeActions = const {},
  Set<String> excludeProperties = const {},
  bool hideUnavailable = false,
  bool includeClimate = false,
}) {
  final result = <dynamic>[];
  for (var i = 0; i < c.capabilities.length; i++) {
    final capability = c.capabilities[i], id = capability['id'] as String;
    // The climate panel and power row already expose these common controls.
    if (id == 'power' || exclude.contains(id)) continue;
    final properties = capability['properties'] as Json;
    final actions = capability['actions'] as Json;
    final extraClimate =
        id == 'climate' &&
        (actions.keys.any((a) => a != 'set') ||
            ((actions['set']?['parameters']?['properties'] as Map?) ?? {}).keys
                .any((p) => !['target_temperature', 'mode'].contains(p)));
    if (id == 'climate' && !extraClimate && !includeClimate) continue;
    final visibleProperties = properties.entries.where(
      (entry) =>
          entry.value['readable'] == true &&
          (entry.value['control_requires_observation'] != true ||
              c.observed('$id.${entry.key}') != null) &&
          !excludeProperties.contains('$id.${entry.key}'),
    );
    final visibleActions = actions.entries.where(
      (entry) =>
          !excludeActions.contains('$id.${entry.key}') &&
          !((entry.value['parameters']?['properties'] as Map? ?? {}).keys.any(
            (p) =>
                properties[p]?['control_requires_observation'] == true &&
                c.observed('$id.$p') == null,
          )) &&
          (!hideUnavailable ||
              (c.hasControlAction(id, entry.key) &&
                  actionUnavailable(
                        capability,
                        entry.value as Json,
                        trustedApproval: c.supportsTrustedApproval,
                      ) ==
                      null)),
    );
    if (visibleProperties.isEmpty && visibleActions.isEmpty) continue;
    result.add([
      'card',
      controlLabel(id),
      [
        for (final entry in visibleProperties)
          if (entry.value['readable'] == true)
            [
              controlLabel(entry.key),
              UiMessage('{value}{stale}', {
                'value': UiMessage(
                  propertyValue(
                    c.observed('$id.${entry.key}'),
                    entry.value as Json,
                    propertyId: '$id.${entry.key}',
                  ),
                ),
                'stale': UiMessage(
                  c.snapshot?['observations']?['$id.${entry.key}']?['stale'] ==
                          true
                      ? ' · Stale'
                      : '',
                ),
              }),
              null,
            ],
        for (final entry in visibleActions)
          [
            controlLabel(entry.key),
            actionUnavailable(
                  capability,
                  entry.value as Json,
                  trustedApproval: c.supportsTrustedApproval,
                ) ??
                'Choose values  ›',
            actionUnavailable(
                      capability,
                      entry.value as Json,
                      trustedApproval: c.supportsTrustedApproval,
                    ) ==
                    null
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
    this.onPrepare,
    super.key,
  });
  final Json capability;
  final String action;
  final Json? Function(String) observation;
  final Future<bool> Function(Json) onSubmit;
  final Future<bool> Function(Json, String)? onPrepare;
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
  String? route;
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
    String? error = actionUnavailable(
      widget.capability,
      descriptor,
      trustedApproval: widget.onPrepare != null,
    );
    if (descriptor['risk'] == 'elevated' && route == null)
      error ??= 'Choose one explicit connection.';
    for (final name in parameters['required'] as List? ?? []) {
      if (!args.containsKey(name)) {
        error ??= localize(
          context,
          UiMessage('Choose {name}.', {'name': UiMessage(controlLabel(name))}),
        );
      }
    }
    if (args.length < (parameters['minProperties'] as num? ?? 0)) {
      error ??= localize(
        context,
        UiMessage('Choose at least {count} value(s).', {
          'count': parameters['minProperties'],
        }),
      );
    }
    for (final entry in args.entries) {
      final problem = validateValue(entry.value, properties[entry.key] as Json);
      if (problem != null)
        error ??=
            '${localize(context, controlLabel(entry.key))}: ${localize(context, problem)}';
    }
    if (error != null) {
      setState(() => message = error);
      return;
    }
    setState(() {
      saving = true;
      message = null;
    });
    final success = descriptor['risk'] == 'elevated' && widget.onPrepare != null
        ? await widget.onPrepare!(args, route!)
        : await widget.onSubmit(args);
    if (!mounted) return;
    setState(() {
      saving = false;
      if (!success)
        message = widget.error() == null
            ? localize(context, 'Operation was not completed.')
            : localizedError(context, widget.error()!);
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
          child: Semantics(
            liveRegion: true,
            child: Copy(message!, color: const Color(0xff9e3421)),
          ),
        ),
      if (descriptor['risk'] == 'elevated' && widget.onPrepare != null) ...[
        const Copy('Connection · Required', bold: true),
        Wrap(
          spacing: 8,
          children: [
            for (final transport in descriptor['transports'] as List)
              ChoiceChip(
                label: Text(
                  localize(context, controlLabel(transport as String)),
                ),
                selected: route == transport,
                onSelected: saving
                    ? null
                    : (selected) =>
                          setState(() => route = selected ? transport : null),
              ),
          ],
        ),
        const SizedBox(height: 16),
      ],
      ActionButton(
        descriptor['risk'] == 'elevated' ? 'Review Command' : 'Apply',
        onPressed: saving ? null : submit,
      ),
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
              UiMessage('{name} · {requirement}', {
                'name': UiMessage(controlLabel(name)),
                'requirement': UiMessage(required ? 'Required' : 'Optional'),
              }),
              bold: true,
            ),
            const SizedBox(height: 8),
            Copy(
              UiMessage('Observed: {value}', {
                'value': UiMessage(propertyValue(observed, property)),
              }),
              size: 12,
              color: muted,
            ),
            if (observation != null)
              Copy(
                UiMessage('Read: {time}{stale}', {
                  'time': receiptTime(
                    observation['received_at'],
                    compact: true,
                  ),
                  'stale': UiMessage(
                    observation['stale'] == true ? ' · Stale' : '',
                  ),
                }),
                size: 12,
                color: muted,
              ),
            if (numeric)
              Copy(
                [
                  if (lower != null)
                    localize(
                      context,
                      UiMessage('Minimum {value}', {'value': lower}),
                    ),
                  if (upper != null)
                    localize(
                      context,
                      UiMessage('Maximum {value}', {'value': upper}),
                    ),
                  if (schema['multipleOf'] != null)
                    localize(
                      context,
                      UiMessage('Step {value}', {
                        'value': schema['multipleOf'],
                      }),
                    ),
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
                        control: true,
                        propertyValue(
                          value,
                          {},
                          propertyId: '${widget.capability['id']}.$name',
                        ),
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
              Semantics(
                label: localize(context, controlLabel(name)),
                child: ValueListenableBuilder<TextEditingValue>(
                  valueListenable: controller!,
                  builder: (context, value, _) => TextField(
                    textAlignVertical: TextAlignVertical.center,
                    key: ValueKey('parameter:$name'),
                    controller: controller,
                    enabled: !saving,
                    keyboardType: numeric
                        ? const TextInputType.numberWithOptions(
                            decimal: true,
                            signed: true,
                          )
                        : TextInputType.text,
                    style: windowFieldTextStyle.copyWith(color: ink),
                    decoration: InputDecoration(
                      filled: true,
                      fillColor: const Color(0xfff8f5ee),
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(14),
                        borderSide: BorderSide.none,
                      ),
                      contentPadding: fieldTextPadding(
                        context,
                        value.text.isEmpty
                            ? localize(
                                context,
                                UiMessage('Enter {name}', {
                                  'name': UiMessage(controlLabel(name)),
                                }),
                              )
                            : value.text,
                      ),
                      hintText: localize(
                        context,
                        UiMessage('Enter {name}', {
                          'name': UiMessage(controlLabel(name)),
                        }),
                      ),
                      suffixText: controlUnit(property['unit']),
                    ),
                    onChanged: (_) => setState(() => message = null),
                  ),
                ),
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
                      '${localize(context, controlLabel(name))} $v ${controlUnit(property['unit'])}',
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
