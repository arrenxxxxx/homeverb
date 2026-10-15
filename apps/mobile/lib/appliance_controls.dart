import 'package:flutter/material.dart';
import 'blocks.dart';
import 'controller.dart';
import 'controls.dart';
import 'native_client.dart';
import 'receipts.dart';
import 'skin.dart';

String? applianceFamily(HomeController c) {
  if (c.capability('climate') != null) return null;
  final families = c.capabilities
      .map((capability) => capability['id'] as String)
      .where(['light', 'fan', 'media', 'sensor', 'appliance_cycle'].contains)
      .toList();
  return families.length == 1 ? families.single : null;
}

/// Presentation uses capability semantics; it never infers support from a brand.
class ApplianceControls extends StatefulWidget {
  const ApplianceControls({
    required this.controller,
    required this.family,
    required this.onAction,
    required this.onExecute,
    super.key,
  });
  final HomeController controller;
  final String family;
  final ActionHandler onAction;
  final Future<bool> Function(String capability, String action, Json arguments)
  onExecute;
  @override
  State<ApplianceControls> createState() => _ApplianceControlsState();
}

class _ApplianceControlsState extends State<ApplianceControls> {
  final drafts = <String, num>{};
  final handled = <String>{};
  HomeController get c => widget.controller;
  Json get capability => c.capability(widget.family)!;
  Json get properties => capability['properties'] as Json;
  Json get actions => capability['actions'] as Json;
  Json? observation(String name) =>
      c.snapshot?['observations']?['${widget.family}.$name'] as Json?;
  dynamic observed(String name) => observation(name)?['value'];

  Widget blocks(List<dynamic> items) => Blocks(
    items,
    onAction: widget.onAction,
    fields: {},
    choices: {},
    values: {},
  );

  void open(String action) => widget.onAction(
    'control:${c.capabilities.indexOf(capability)}:${Uri.encodeComponent(action)}',
  );

  String? setter(String property) {
    if (properties[property]?['writable'] != true) return null;
    final expected = switch (widget.family) {
      'light' || 'fan' => 'set',
      'media' => const {
        'volume': 'set_volume',
        'muted': 'mute',
        'source': 'select_source',
      }[property],
      _ => null,
    };

    for (final entry in actions.entries) {
      final action = entry.value as Json;
      final schema = action['parameters'] as Json;
      if (entry.key != expected || action['idempotency'] != 'set_value')
        continue;
      if (schema.keys.any(
            (key) => !const {
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
          (schema['properties'] as Map?)?.containsKey(property) != true ||
          actionUnavailable(capability, action) != null ||
          (schema['minProperties'] as num? ?? 0) > 1 ||
          (schema['required'] as List? ?? []).any((key) => key != property)) {
        continue;
      }
      return entry.key;
    }
    return null;
  }

  Future<void> send(String action, Json arguments) async {
    if (c.busy) return;
    await widget.onExecute(widget.family, action, arguments);
  }

  String display(String property) =>
      '${propertyValue(observed(property), properties[property] as Json)}${observation(property)?['stale'] == true ? ' · Stale' : ''}';

  Widget propertyControl(String name) {
    final property = properties[name] as Json;
    final action = widget.family == 'sensor' ? null : setter(name);
    final schema = action == null
        ? <String, dynamic>{}
        : actions[action]['parameters']['properties'][name] as Json;
    final value = observed(name);
    final lower = schema['minimum'], upper = schema['maximum'];
    if (action != null &&
        ['number', 'integer'].contains(schema['type']) &&
        schema['enum'] == null &&
        lower is num &&
        upper is num &&
        lower < upper) {
      handled.add(action);
      final draft = drafts[name];
      final current = draft ?? (value is num ? value : null);
      final usable = current != null && validateValue(current, schema) == null;
      final step =
          schema['multipleOf'] as num? ??
          (schema['type'] == 'integer' ? 1 : null);
      num? snap(double input) {
        final next = step == null ? input : (input / step).round() * step;
        return validateValue(next, schema) == null ? next : null;
      }

      final numericFan =
          widget.family == 'fan' && name == 'speed' && step != null;
      Widget speedButtons() => InlineStepper(
        label: 'Speed',
        value: propertyValue(current, property),
        onIncrease:
            c.busy || !usable || validateValue(current + step!, schema) != null
            ? null
            : () => send(action, {name: current + step}),
        onDecrease:
            c.busy || !usable || validateValue(current - step!, schema) != null
            ? null
            : () => send(action, {name: current - step}),
      );
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (numericFan)
            Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: Surface(
                radius: 18,
                padding: const EdgeInsets.only(left: 16, right: 8),
                child: MediaQuery.textScalerOf(context).scale(13) <= 13
                    ? Row(
                        children: [
                          const Expanded(child: Copy('Speed')),
                          speedButtons(),
                        ],
                      )
                    : Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [const Copy('Speed'), speedButtons()],
                      ),
              ),
            )
          else
            ValueSlider(
              key: ValueKey('appliance-slider:${widget.family}:$name'),
              label: controlLabel(name),
              displayValue: propertyValue(current, property),
              value: usable ? current.toDouble() : lower.toDouble(),
              min: lower.toDouble(),
              max: upper.toDouble(),
              formatValue: (v) =>
                  '${controlLabel(name)} ${propertyValue(usable ? snap(v) ?? v : current, property)}',
              onChanged: c.busy || !usable
                  ? null
                  : (v) {
                      final next = snap(v);
                      if (next != null) setState(() => drafts[name] = next);
                    },
              onChangeEnd: c.busy || !usable
                  ? null
                  : (_) {
                      final next = drafts[name];
                      if (next != null) send(action, {name: next});
                    },
            ),
          if (draft != null || observation(name)?['stale'] == true)
            Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: Copy('Observed: ${display(name)}', size: 12, color: muted),
            ),
          if (!usable)
            ActionButton(
              'Set ${controlLabel(name).toLowerCase()}',
              style: 'secondary',
              onPressed: c.busy ? null : () => open(action),
            ),
        ],
      );
    }
    if (widget.family == 'fan' &&
        name == 'speed' &&
        action != null &&
        schema['enum'] is List) {
      handled.add(action);
      return Padding(
        padding: const EdgeInsets.only(bottom: 12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Rows([
              [controlLabel(name), display(name), null],
            ], onAction: widget.onAction),
            Wrap(
              spacing: 7,
              runSpacing: 8,
              children: [
                for (final option in schema['enum'] as List)
                  ChoiceChip(
                    key: ValueKey('appliance-option:$name:$option'),
                    label: Copy(
                      propertyValue(option, {}),
                      size: 12,
                      color: value == option ? Colors.white : muted,
                    ),
                    selected: value == option,
                    showCheckmark: false,
                    side: BorderSide.none,
                    backgroundColor: const Color(0xfff8f5ee),
                    selectedColor: ink,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(16),
                    ),
                    onSelected: c.busy
                        ? null
                        : (_) => send(action, {name: option}),
                  ),
              ],
            ),
          ],
        ),
      );
    }
    return Rows([propertyRow(name)], onAction: rowAction);
  }

  List<dynamic> propertyRow(String name) {
    final action = widget.family == 'sensor' ? null : setter(name);
    if (action != null) handled.add(action);
    final value = observed(name);
    return [
      controlLabel(name),
      value is bool ? propertyValue(value, properties[name]) : display(name),
      action == null || c.busy ? null : 'property:$name',
    ];
  }

  void rowAction(String label) {
    if (c.busy) return;
    if (!label.startsWith('property:')) {
      widget.onAction(label);
      return;
    }
    final name = label.substring('property:'.length), action = setter(name);
    if (action == null) return;
    final value = observed(name),
        schema = actions[action]['parameters']['properties'][name] as Json;
    if (value is bool && validateValue(!value, schema) == null) {
      send(action, {name: !value});
    } else {
      open(action);
    }
  }

  @override
  Widget build(BuildContext context) {
    handled.clear();
    final family = widget.family;
    final power = c.observed('power.state');
    final powerCapability = c.capability('power');
    final powerAction = powerCapability?['actions']?['set'] as Json?;
    final powerSchema =
        powerAction?['parameters']?['properties']?['state'] as Json?;
    final canSetPower =
        !c.busy &&
        power is bool &&
        powerSchema != null &&
        powerAction != null &&
        actionUnavailable(powerCapability!, powerAction) == null &&
        validateValue(!power, powerSchema) == null &&
        (powerAction['parameters']['minProperties'] as num? ?? 0) <= 1 &&
        (powerAction['parameters']['required'] as List? ?? []).every(
          (name) => name == 'state',
        );

    final readings = properties.keys.any(
      (name) => properties[name]['readable'] == true && observed(name) != null,
    );
    final state = family == 'sensor'
        ? (readings ? 'Observed readings' : 'No current reading')
        : family == 'media'
        ? propertyValue(observed('playback_state'), {})
        : family == 'appliance_cycle'
        ? (observed('running') is bool
              ? (observed('running') == true ? 'Running' : 'Not running')
              : 'Unknown')
        : (power is bool
              ? (power ? 'Power on' : 'Power off')
              : (c.capability('power') == null
                    ? controlLabel(family)
                    : 'Power unknown'));
    final specialNames = switch (family) {
      'light' => ['brightness', 'color_temperature'],
      'fan' => ['speed'],
      'media' => ['volume'],
      _ => <String>[],
    };
    final controls = <Widget>[
      for (final name in specialNames)
        if (properties[name]?['readable'] == true) propertyControl(name),
    ];
    final rows = <List<dynamic>>[
      for (final name in properties.keys)
        if (properties[name]['readable'] == true &&
            !specialNames.contains(name))
          propertyRow(name),
      if (c.capability('power') != null)
        [
          'Power',
          power is bool ? (power ? 'On' : 'Off') : 'Unknown',
          canSetPower ? 'Power' : null,
        ],
    ];
    final times = [
      for (final name in properties.keys)
        if (observation(name)?['received_at'] is String)
          observation(name)!['received_at'] as String,
    ]..sort();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (c.components.length > 1)
          blocks([
            [
              'tabs',
              c.components.map((p) => '${p['id']}').toList(),
              c.components.indexWhere((p) => p['id'] == c.component),
              c.components.map((p) => 'component:${p['id']}').toList(),
            ],
          ]),
        blocks([
          [
            'hero',
            const {
              'light': 'lamp',
              'fan': 'fan',
              'media': 'tv',
              'sensor': 'sensor',
              'appliance_cycle': 'washer',
            }[family],
            state,
            c.roomName(c.currentCard?['preferences']['room_id']),
          ],
        ]),
        if (family != 'media') ...controls,
        if (rows.isNotEmpty) Rows(rows, onAction: rowAction),
        if (properties.keys.any(
          (name) =>
              observed(name) is bool && observation(name)?['stale'] == true,
        ))
          const Padding(
            padding: EdgeInsets.only(bottom: 12),
            child: Copy(
              'Saved states may be out of date.',
              size: 12,
              color: muted,
            ),
          ),
        if (family != 'sensor')
          for (final entry in actions.entries)
            if (!handled.contains(entry.key) ||
                ((entry.value['parameters']['properties'] as Map?)?.keys ??
                        const [])
                    .any(
                      (name) =>
                          properties[name]?['readable'] != true ||
                          setter(name) != entry.key,
                    ))
              ActionButton(
                family == 'appliance_cycle' && entry.key == 'start'
                    ? 'Start Cycle'
                    : controlLabel(entry.key),
                style: 'secondary',
                onPressed:
                    c.busy ||
                        actionUnavailable(capability, entry.value as Json) !=
                            null
                    ? null
                    : () {
                        if ((entry.value['parameters']['properties'] as Map?)
                                ?.isEmpty ==
                            true) {
                          send(entry.key, {});
                        } else {
                          open(entry.key);
                        }
                      },
              ),
        if (family == 'media') ...controls,
        for (final entry in actions.entries)
          if (actionUnavailable(capability, entry.value as Json)
              case final String reason)
            Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: Copy('${controlLabel(entry.key)}: $reason', color: muted),
            ),
        if (times.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Copy(
              'Latest read: ${receiptTime(times.last, compact: true)}',
              size: 12,
              color: muted,
            ),
          ),
        if (family == 'sensor')
          const Padding(
            padding: EdgeInsets.only(bottom: 12),
            child: Copy(
              'This device is read-only. Missing readings are not zero.',
              color: muted,
            ),
          ),
        blocks(describedControls(c, exclude: {family})),
        ActionButton(
          'Refresh State',
          style: 'secondary',
          onPressed: c.busy
              ? null
              : () {
                  drafts.clear();
                  widget.onAction('Refresh State');
                },
        ),
        ActionButton(
          'Device information',
          style: 'secondary',
          onPressed: () => widget.onAction('Device information'),
        ),
        ActionButton(
          'Recent Operations',
          style: 'secondary',
          onPressed: () => widget.onAction('Recent Operations'),
        ),
      ],
    );
  }
}
