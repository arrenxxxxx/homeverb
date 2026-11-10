import 'localization.dart';
import 'package:flutter/material.dart';
import 'blocks.dart';
import 'appliance_panels.dart' show InlineDeviceInformation;
import 'controller.dart';
import 'controls.dart';
import 'device_readings.dart';
import 'device_option_grid.dart';
import 'device_typography.dart';
import 'client_contract.dart';
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
  String? pendingAction, pendingProperty;
  bool pendingFor(String? action, String property) =>
      action != null &&
      pendingAction == action &&
      (pendingProperty == null || pendingProperty == property);
  HomeController get c => widget.controller;
  bool get busy => pendingAction != null || c.busy || !c.isForeground;
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

  void open(String action) {
    if (!mounted ||
        busy ||
        !c.isForeground ||
        !c.hasControlAction(widget.family, action))
      return;
    final definition = actions[action] as Json?;
    final fields = definition?['parameters']?['properties'] as Map? ?? {};
    if (definition == null ||
        actionUnavailable(
              capability,
              definition,
              trustedApproval: c.supportsTrustedApproval,
            ) !=
            null ||
        fields.keys.any(
          (name) =>
              properties[name]?['control_requires_observation'] == true &&
              observed(name) == null,
        ))
      return;
    widget.onAction(
      'control:${c.capabilities.indexOf(capability)}:${Uri.encodeComponent(action)}',
    );
  }

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
      if (!c.hasControlAction(widget.family, entry.key) ||
          (properties[property]?['control_requires_observation'] == true &&
              observed(property) == null))
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
    if (pendingAction != null ||
        busy ||
        !c.isForeground ||
        !c.hasControlAction(widget.family, action))
      return;
    final definition = actions[action] as Json?;
    if (definition == null || actionUnavailable(capability, definition) != null)
      return;
    if (arguments.length == 1) {
      final entry = arguments.entries.single;
      if (entry.value is! Object ||
          !c.canControlValue(
            widget.family,
            action,
            entry.key,
            entry.value as Object,
          ))
        return;
    }
    setState(() {
      drafts.clear();
      pendingAction = action;
      pendingProperty = arguments.length == 1 ? arguments.keys.single : null;
    });
    try {
      await widget.onExecute(widget.family, action, arguments);
    } finally {
      if (mounted)
        setState(() {
          pendingAction = null;
          pendingProperty = null;
        });
    }
  }

  Future<void> sendPower() async {
    final value = c.observed('power.state');
    final power = c.capability('power');
    final action = power?['actions']?['set'] as Json?;
    final schema = action?['parameters']?['properties']?['state'] as Json?;
    if (pendingAction != null ||
        busy ||
        !c.isForeground ||
        value is! bool ||
        action == null ||
        schema == null ||
        actionUnavailable(power!, action) != null ||
        validateValue(!value, schema) != null ||
        (action['parameters']['required'] as List? ?? []).any(
          (name) => name != 'state',
        ) ||
        !c.canControlValue('power', 'set', 'state', !value))
      return;
    setState(() => pendingAction = 'power.set');
    try {
      await widget.onExecute('power', 'set', {'state': !value});
    } finally {
      if (mounted) setState(() => pendingAction = null);
    }
  }

  Object display(String property) => UiMessage(
    propertyValue(observed(property), properties[property] as Json),
  );

  Widget propertyControl(String name) {
    final property = properties[name] as Json;
    final action = widget.family == 'sensor' ? null : setter(name);
    final schema = action == null
        ? <String, dynamic>{}
        : actions[action]['parameters']['properties'][name] as Json;
    final value = observed(name);
    if (value == null) return Rows([propertyRow(name)], onAction: rowAction);
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

      if (!usable) return Rows([propertyRow(name)], onAction: rowAction);
      final numericFan =
          widget.family == 'fan' && name == 'speed' && step != null;
      Widget speedButtons() => InlineStepper(
        label: 'Speed',
        value: propertyValue(current, property),
        onIncrease:
            busy ||
                !usable ||
                validateValue(current + step!, schema) != null ||
                !c.canControlValue(widget.family, action, name, current + step)
            ? null
            : () => send(action, {name: current + step}),
        onDecrease:
            busy ||
                !usable ||
                validateValue(current - step!, schema) != null ||
                !c.canControlValue(widget.family, action, name, current - step)
            ? null
            : () => send(action, {name: current - step}),
      );
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (pendingFor(action, name))
            const Align(
              alignment: Alignment.centerRight,
              child: OperationPulse(size: 20),
            ),
          if (numericFan)
            Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: Surface(
                radius: 18,
                padding: const EdgeInsets.only(left: 16, right: 8),
                child: MediaQuery.textScalerOf(context).scale(13) <= 13
                    ? Row(
                        children: [
                          const Expanded(child: Copy('Speed', control: true)),
                          speedButtons(),
                        ],
                      )
                    : Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          const Copy('Speed', control: true),
                          speedButtons(),
                        ],
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
              onChanged: busy || !usable
                  ? null
                  : (v) {
                      final next = snap(v);
                      if (next != null &&
                          c.canControlValue(
                            widget.family,
                            action,
                            name,
                            next,
                          )) {
                        setState(() => drafts[name] = next);
                      }
                    },
              onChangeEnd: busy || !usable
                  ? null
                  : (_) {
                      final next = drafts[name];
                      if (next != null) send(action, {name: next});
                    },
            ),
          if (draft != null)
            Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: Copy(
                UiMessage('Observed: {value}', {'value': display(name)}),
                size: 12,
                color: muted,
              ),
            ),
          if (!usable)
            ActionButton(
              localize(
                context,
                UiMessage('Set {name}', {
                  'name': localize(context, controlLabel(name)).toLowerCase(),
                }),
              ),
              style: 'secondary',
              onPressed: busy ? null : () => open(action),
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
            if (pendingFor(action, name))
              const Align(
                alignment: Alignment.centerRight,
                child: OperationPulse(size: 20),
              ),
            Rows([
              [controlLabel(name), display(name), null],
            ], onAction: widget.onAction),
            DeviceOptionGrid(
              key: ValueKey('appliance-options-grid:$name'),
              labels: [
                for (final option in schema['enum'] as List)
                  localize(context, propertyValue(option, {})),
              ],
              children: [
                for (final option in schema['enum'] as List)
                  ChoiceChip(
                    key: ValueKey('appliance-option:$name:$option'),
                    label: DeviceOptionLabel(
                      text: localize(context, propertyValue(option, {})),
                      selected: value == option,
                    ),
                    labelStyle: TextStyle(
                      fontSize: DeviceTypography.body,
                      height: DeviceTypography.lineHeight,
                      color: value == option ? Colors.white : muted,
                    ),
                    padding: const EdgeInsets.symmetric(vertical: 12),
                    labelPadding: const EdgeInsets.symmetric(horizontal: 4),
                    materialTapTargetSize: MaterialTapTargetSize.padded,
                    chipAnimationStyle: ChipAnimationStyle(
                      avatarDrawerAnimation: AnimationStyle.noAnimation,
                    ),
                    selected: value == option,
                    showCheckmark: false,
                    checkmarkColor: Colors.white,
                    side: BorderSide.none,
                    backgroundColor: skinOf(context).isSky
                        ? skinOf(context).secondaryColor
                        : const Color(0xfff8f5ee),
                    selectedColor: skinOf(context).isSky
                        ? skinAccent(context)
                        : ink,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(16),
                    ),
                    onSelected:
                        busy ||
                            !c.isForeground ||
                            !c.canControlValue(
                              widget.family,
                              action,
                              name,
                              option,
                            )
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
      action == null ||
              busy ||
              !c.isForeground ||
              (value is bool &&
                  !c.canControlValue(widget.family, action, name, !value))
          ? null
          : 'property:$name',
      if (pendingFor(action, name))
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Copy(display(name)),
            const SizedBox(width: 8),
            const OperationPulse(size: 20),
          ],
        )
      else if (value == null && c.readingLoading(c.component))
        const ReadingValue('—', loading: true),
    ];
  }

  void rowAction(String label) {
    if (busy) return;
    if (label == 'Power') {
      sendPower();
      return;
    }
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
        !busy &&
        c.isForeground &&
        power is bool &&
        powerSchema != null &&
        powerAction != null &&
        actionUnavailable(powerCapability!, powerAction) == null &&
        c.canControlValue('power', 'set', 'state', !power) &&
        validateValue(!power, powerSchema) == null &&
        (powerAction['parameters']['minProperties'] as num? ?? 0) <= 1 &&
        (powerAction['parameters']['required'] as List? ?? []).every(
          (name) => name == 'state',
        );

    final readings = properties.keys.any(
      (name) => properties[name]['readable'] == true && observed(name) != null,
    );
    final state = family == 'sensor'
        ? (readings ? 'Observed readings' : '—')
        : family == 'media'
        ? propertyValue(observed('playback_state'), {})
        : family == 'appliance_cycle'
        ? (observed('running') is bool
              ? (observed('running') == true ? 'Running' : 'Not running')
              : '—')
        : (power is bool
              ? (power ? 'Power on' : 'Power off')
              : (c.capability('power') == null ? controlLabel(family) : '—'));
    final heroLoading =
        c.readingLoading(c.component) &&
        (family == 'sensor'
            ? !readings
            : family == 'media'
            ? observed('playback_state') == null
            : family == 'appliance_cycle'
            ? observed('running') == null
            : powerCapability != null && power == null);
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
          power is bool ? (power ? 'On' : 'Off') : '—',
          canSetPower ? 'Power' : null,
          if (pendingAction == 'power.set')
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Copy(
                  power == true
                      ? 'On'
                      : power == false
                      ? 'Off'
                      : '—',
                ),
                const SizedBox(width: 8),
                const OperationPulse(size: 20),
              ],
            )
          else if (power == null && c.readingLoading(c.component))
            const ReadingValue('—', loading: true),
        ],
    ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.only(bottom: 14),
          child: Surface(
            child: Padding(
              padding: const EdgeInsets.fromLTRB(20, 13, 20, 20),
              child: Column(
                children: [
                  SizedBox(
                    height: 87,
                    child: product(
                      const {
                        'light': 'lamp',
                        'fan': 'fan',
                        'media': 'tv',
                        'sensor': 'sensor',
                        'appliance_cycle': 'washer',
                      }[family]!,
                      size: 87,
                    ),
                  ),
                  const SizedBox(height: 19),
                  ReadingValue(
                    state,
                    loading: heroLoading,
                    size: 23,
                    bold: true,
                    center: true,
                  ),
                  const SizedBox(height: 13),
                  Align(
                    alignment: Alignment.centerLeft,
                    child: Copy(
                      c.currentCard?['preferences']['room_id'] == null
                          ? 'Unassigned'
                          : LiteralText(
                              c.roomName(
                                c.currentCard!['preferences']['room_id'],
                              ),
                            ),
                      size: 12,
                      color: muted,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
        if (family != 'media') ...controls,
        if (rows.isNotEmpty) Rows(rows, onAction: rowAction),
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
              Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  ActionButton(
                    family == 'appliance_cycle' && entry.key == 'start'
                        ? 'Start Cycle'
                        : controlLabel(entry.key),
                    style: 'secondary',
                    onPressed:
                        busy ||
                            !c.isForeground ||
                            !c.hasControlAction(family, entry.key) ||
                            ((entry.value['parameters']['properties']
                                        as Map?) ??
                                    {})
                                .keys
                                .any(
                                  (name) =>
                                      properties[name]?['control_requires_observation'] ==
                                          true &&
                                      observed(name) == null,
                                ) ||
                            actionUnavailable(
                                  capability,
                                  entry.value as Json,
                                  trustedApproval: c.supportsTrustedApproval,
                                ) !=
                                null
                        ? null
                        : () {
                            if (entry.value['risk'] == 'elevated') {
                              open(entry.key);
                            } else if ((entry.value['parameters']['properties']
                                        as Map?)
                                    ?.isEmpty ==
                                true) {
                              send(entry.key, {});
                            } else {
                              open(entry.key);
                            }
                          },
                  ),
                  if (pendingAction == entry.key && pendingProperty == null)
                    const Align(
                      alignment: Alignment.centerRight,
                      child: OperationPulse(size: 20),
                    ),
                ],
              ),
        if (family == 'media') ...controls,
        for (final entry in actions.entries)
          if (actionUnavailable(
                capability,
                entry.value as Json,
                trustedApproval: c.supportsTrustedApproval,
              )
              case final String reason)
            Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: Copy(
                UiMessage('{label}: {message}', {
                  'label': UiMessage(controlLabel(entry.key)),
                  'message': UiMessage(reason),
                }),
                color: muted,
              ),
            ),
        blocks(describedControls(c, exclude: {family})),
        if (c.receipt case final Json receipt)
          if (receipt['device_id'] == c.selectedDevice &&
              pendingAction == null &&
              !c.isCommandError &&
              (['unknown', 'failed', 'partial'].contains(receipt['status']) ||
                  (receipt['status'] == 'accepted' && c.error != null)))
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Copy(outcomeSummary(receipt['status']), color: muted),
                TextButton(
                  onPressed: busy
                      ? null
                      : () => widget.onAction('View Details'),
                  child: const Copy('View Details'),
                ),
              ],
            ),
        InlineDeviceInformation(controller: c, onAction: widget.onAction),
      ],
    );
  }
}
