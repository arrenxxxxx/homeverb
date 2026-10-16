import 'package:flutter/material.dart';
import 'blocks.dart';
import 'climate_panel.dart';
import 'controller.dart';
import 'controls.dart';
import 'localization.dart';
import 'native_client.dart';
import 'receipts.dart';
import 'skin.dart';

/// A single independently settable descriptor field, never a protocol command.
class ClimateChoice {
  const ClimateChoice(
    this.capability,
    this.action,
    this.parameter,
    this.schema,
  );
  final Json capability, schema;
  final String action, parameter;
  String get property => '${capability['id']}.$parameter';
  List<dynamic> get values => schema['enum'] as List? ?? [true, false];
}

ClimateChoice? climateChoice(HomeController controller, Set<String> names) {
  final candidates = <ClimateChoice>[];
  for (final capability in controller.capabilities) {
    for (final entry in (capability['actions'] as Json).entries) {
      final action = entry.value as Json;
      final parameters = action['parameters'] as Json;
      if (action['risk'] != 'low' ||
          action['idempotency'] != 'set_value' ||
          actionUnavailable(capability, action) != null ||
          (parameters['minProperties'] as num? ?? 0) > 1)
        continue;
      for (final field in (parameters['properties'] as Json).entries) {
        final schema = field.value as Json;
        if (!names.contains(field.key) ||
            (parameters['required'] as List? ?? []).any(
              (key) => key != field.key,
            ) ||
            (schema['enum'] is! List && schema['type'] != 'boolean'))
          continue;
        final values = schema['enum'] as List? ?? [true, false];
        if (values.isEmpty ||
            values.any((v) => validateValue(v, schema) != null))
          continue;
        candidates.add(ClimateChoice(capability, entry.key, field.key, schema));
      }
    }
  }
  // Ambiguous or coupled controls remain in the full descriptor form.
  return candidates.length == 1 ? candidates.single : null;
}

class ClimateControls extends StatefulWidget {
  const ClimateControls({
    super.key,
    required this.controller,
    required this.onAction,
    required this.onExecute,
  });
  final HomeController controller;
  final ActionHandler onAction;
  final Future<bool> Function(String, String, Json) onExecute;
  @override
  State<ClimateControls> createState() => _ClimateControlsState();
}

class _ClimateControlsState extends State<ClimateControls>
    with WidgetsBindingObserver {
  HomeController get c => widget.controller;
  BuildContext? sheetContext;
  int selectionEpoch = 0;
  double? draftTarget;
  int? targetGestureEpoch;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    selectionEpoch++;
    targetGestureEpoch = null;
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) return;
    selectionEpoch++;
    if (mounted) setState(() => draftTarget = null);
    final sheet = sheetContext;
    if (sheet != null && sheet.mounted) Navigator.of(sheet).pop();
  }

  Object choiceValue(ClimateChoice? choice) {
    if (choice == null) return 'Not connected in HomeVerb';
    final observed = c.observed(choice.property);
    final value = propertyValue(
      observed,
      choice.capability['properties']?[choice.parameter] as Json? ?? {},
      propertyId: choice.property,
    );
    if (observed != null && !choice.values.contains(observed)) {
      return UiMessage('Unrecognized ({value})', {'value': value});
    }
    return value;
  }

  Future<void> choose(String title, ClimateChoice choice) async {
    if (c.busy || !c.isForeground) return;
    final epoch = ++selectionEpoch;
    final device = c.selectedDevice;
    final component = c.component;
    Object? selected;
    final value = await showModalBottomSheet<Object>(
      context: context,
      isScrollControlled: true,
      useSafeArea: true,
      builder: (context) {
        sheetContext = context;
        return StatefulBuilder(
          builder: (context, update) => SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Copy(title, size: 24, bold: true),
                const SizedBox(height: 12),
                if (choice.action == 'activate_mode') ...[
                  const Copy('Selecting a mode also turns this device on.'),
                  const SizedBox(height: 12),
                ],
                Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  children: [
                    for (final option in choice.values)
                      ChoiceChip(
                        key: ValueKey(
                          'climate-option:${choice.parameter}:$option',
                        ),
                        label: Text(
                          localize(
                            context,
                            propertyValue(
                              option,
                              {},
                              propertyId: choice.property,
                            ),
                          ),
                        ),
                        selected: selected == option,
                        onSelected: (_) => update(() => selected = option),
                      ),
                  ],
                ),
                const SizedBox(height: 20),
                FilledButton(
                  onPressed: selected == null
                      ? null
                      : () => Navigator.of(context).pop(selected),
                  style: FilledButton.styleFrom(backgroundColor: climateBlue),
                  child: Text(localize(context, 'Apply')),
                ),
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: Text(localize(context, 'Cancel')),
                ),
              ],
            ),
          ),
        );
      },
    );
    sheetContext = null;
    if (value == null ||
        !mounted ||
        epoch != selectionEpoch ||
        !c.isForeground ||
        device != c.selectedDevice ||
        component != c.component)
      return;
    await widget.onExecute(choice.capability['id'], choice.action, {
      choice.parameter: value,
    });
  }

  Json? get targetSchema {
    final capability = c.capability('climate');
    final action = capability?['actions']?['set'] as Json?;
    if (capability == null ||
        action == null ||
        action['risk'] != 'low' ||
        action['idempotency'] != 'set_value' ||
        actionUnavailable(capability, action) != null)
      return null;
    final parameters = action['parameters'] as Json;
    if ((parameters['minProperties'] as num? ?? 0) > 1 ||
        (parameters['required'] as List? ?? []).any(
          (key) => key != 'target_temperature',
        ))
      return null;
    final schema = parameters['properties']?['target_temperature'] as Json?;
    if (schema == null) return null;
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
      return null;
    return schema;
  }

  Future<void> setTarget(double value) async {
    final schema = targetSchema;
    setState(() => draftTarget = null);
    if (c.busy ||
        !c.isForeground ||
        schema == null ||
        validateValue(value, schema) != null ||
        value == c.observed('climate.target_temperature'))
      return;
    await widget.onExecute('climate', 'set', {'target_temperature': value});
  }

  static const features = <(String, Set<String>, IconData)>[
    ('Vertical swing', {'vertical_swing', 'swing_vertical'}, Icons.swap_vert),
    (
      'Horizontal swing',
      {'horizontal_swing', 'swing_horizontal'},
      Icons.swap_horiz,
    ),
    ('Avoid direct airflow', {'avoid_direct_airflow'}, Icons.air),
    ('Sleep', {'sleep'}, Icons.bedtime_outlined),
    ('Energy saving', {'energy_saving', 'eco'}, Icons.eco_outlined),
    ('Boost', {'boost'}, Icons.bolt_outlined),
    ('Display', {'display'}, Icons.light_mode_outlined),
    ('Auxiliary heat', {'auxiliary_heat'}, Icons.waves),
  ];

  Future<void> showSupport() async {
    await showModalBottomSheet<void>(
      context: context,
      isScrollControlled: true,
      useSafeArea: true,
      showDragHandle: true,
      builder: (context) => DraggableScrollableSheet(
        expand: false,
        initialChildSize: .7,
        maxChildSize: .9,
        builder: (context, scroll) => ListView(
          controller: scroll,
          padding: const EdgeInsets.fromLTRB(24, 0, 24, 24),
          children: [
            const Copy('Function support', size: 24, bold: true),
            const SizedBox(height: 12),
            const Copy(
              'This list describes HomeVerb integration, not the appliance hardware. Unconnected functions cannot send commands.',
            ),
            const SizedBox(height: 12),
            for (final item in <(String, bool)>[
              ('Target temperature', targetSchema != null),
              ('Mode', climateChoice(c, {'mode'}) != null),
              (
                'Fan speed',
                climateChoice(c, {'fan_speed', 'wind_speed'}) != null,
              ),
              (
                'Air direction',
                climateChoice(c, {'air_direction', 'swing', 'oscillation'}) !=
                    null,
              ),
              for (final f in features) (f.$1, climateChoice(c, f.$2) != null),
              ('Timer', false),
              ('Self-clean', false),
            ])
              ListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(localize(context, item.$1)),
                subtitle: Text(
                  localize(
                    context,
                    item.$2
                        ? 'Available controls'
                        : 'Not connected in HomeVerb',
                  ),
                ),
                trailing: Icon(
                  item.$2 ? Icons.check_circle_outline : Icons.link_off,
                  color: item.$2 ? climateBlue : muted,
                ),
              ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final mode = climateChoice(c, {'mode'});
    final direction = climateChoice(c, {
      'air_direction',
      'swing',
      'oscillation',
    });
    final fan = climateChoice(c, {'wind_speed', 'fan_speed'});
    final extras = [
      for (final f in features)
        if (climateChoice(c, f.$2) case final ClimateChoice choice)
          (f.$1, f.$3, choice),
    ];
    final primary = [
      mode,
      direction,
      fan,
      ...extras.map((f) => f.$3),
    ].whereType<ClimateChoice>().toList();
    final power = c.observed('power.state');
    final powerCapability = c.capability('power');
    final powerAction = powerCapability?['actions']?['set'] as Json?;
    final canPower =
        power is bool &&
        powerAction != null &&
        actionUnavailable(powerCapability!, powerAction) == null &&
        validateValue(
              !power,
              powerAction['parameters']['properties']['state'] as Json,
            ) ==
            null;
    final target = c.observed('climate.target_temperature');
    final schema = targetSchema;
    final bounded =
        schema != null &&
        target is num &&
        validateValue(target, schema) == null;
    final receipt = c.receipt;
    final observations = (c.snapshot?['observations'] as Json? ?? {}).values
        .whereType<Map>();
    final times =
        observations.map((v) => v['received_at']).whereType<String>().toList()
          ..sort();
    final stale = observations.any((v) => v['stale'] == true);
    final remaining = describedControls(
      c,
      excludeActions: {
        for (final choice in primary)
          '${choice.capability['id']}.${choice.action}',
      },
      excludeProperties: {for (final choice in primary) choice.property},
    );
    Widget selector(String title, Widget icon, ClimateChoice? choice) =>
        ClimateTile(
          key: ValueKey('climate-button:$title'),
          title: title,
          value: choiceValue(choice),
          icon: icon,
          onPressed: c.busy || choice == null
              ? null
              : () => choose(title, choice),
        );
    return Theme(
      data: Theme.of(context).copyWith(
        colorScheme: Theme.of(
          context,
        ).colorScheme.copyWith(primary: climateBlue),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (c.components.length > 1)
            Wrap(
              spacing: 8,
              children: [
                for (final part in c.components)
                  ChoiceChip(
                    label: Text(part['id']),
                    selected: c.component == part['id'],
                    onSelected: c.busy
                        ? null
                        : (_) => widget.onAction('component:${part['id']}'),
                  ),
              ],
            ),
          ClimateHero(
            power: power,
            indoor: c.observed('climate.current_temperature'),
          ),
          ClimateSurface(
            child: Column(
              children: [
                const Copy(
                  'Target temperature',
                  color: muted,
                  size: 13,
                  center: true,
                ),
                const SizedBox(height: 10),
                Row(
                  children: [
                    IconButton.filledTonal(
                      key: const ValueKey('climate-decrease'),
                      tooltip: localize(context, 'Decrease target'),
                      onPressed:
                          !bounded || c.busy || target <= schema['minimum']
                          ? null
                          : () => setTarget(
                              (target - schema['multipleOf']).toDouble(),
                            ),
                      icon: const Icon(Icons.remove),
                    ),
                    Expanded(
                      child: Semantics(
                        liveRegion: true,
                        child: Text.rich(
                          TextSpan(
                            text: target is num
                                ? '${draftTarget ?? target}'.replaceFirst(
                                    RegExp(r'\.0$'),
                                    '',
                                  )
                                : '—',
                            children: const [
                              TextSpan(
                                text: ' °C',
                                style: TextStyle(fontSize: 18, color: muted),
                              ),
                            ],
                          ),
                          textAlign: TextAlign.center,
                          style: TextStyle(
                            fontSize: 58,
                            fontWeight: FontWeight.w300,
                            color: readableInk(context, ink),
                            height: 1.2,
                          ),
                        ),
                      ),
                    ),
                    IconButton.filledTonal(
                      key: const ValueKey('climate-increase'),
                      tooltip: localize(context, 'Increase target'),
                      onPressed:
                          !bounded || c.busy || target >= schema['maximum']
                          ? null
                          : () => setTarget(
                              (target + schema['multipleOf']).toDouble(),
                            ),
                      icon: const Icon(Icons.add),
                    ),
                  ],
                ),
                if (bounded)
                  Slider(
                    key: const ValueKey('climate-temperature-slider'),
                    value: draftTarget ?? target.toDouble(),
                    min: (schema['minimum'] as num).toDouble(),
                    max: (schema['maximum'] as num).toDouble(),
                    divisions:
                        ((schema['maximum'] - schema['minimum']) /
                                schema['multipleOf'])
                            .round(),
                    label: '${draftTarget ?? target}°C',
                    semanticFormatterCallback: (v) => '${v}°C',
                    onChangeStart: c.busy
                        ? null
                        : (_) => targetGestureEpoch = selectionEpoch,
                    onChanged: c.busy
                        ? null
                        : (v) => setState(() => draftTarget = v),
                    onChangeEnd: c.busy
                        ? null
                        : (value) {
                            if (targetGestureEpoch != selectionEpoch) return;
                            targetGestureEpoch = null;
                            setTarget(value);
                          },
                  )
                else
                  Padding(
                    padding: const EdgeInsets.only(top: 12),
                    child: Copy(
                      schema == null
                          ? 'Temperature control is not connected yet.'
                          : 'No reading',
                      color: muted,
                      size: 12,
                      center: true,
                    ),
                  ),
              ],
            ),
          ),
          const SizedBox(height: 14),
          ClimateGrid(
            children: [
              ClimateTile(
                key: const ValueKey('climate-power'),
                title: 'Power',
                value: power is bool
                    ? (power ? 'Turn off' : 'Turn on')
                    : 'Power unknown',
                icon: Icon(
                  Icons.power_settings_new,
                  size: 28,
                  color: power == true ? climateBlue : muted,
                ),
                active: power == true,
                onPressed: c.busy || !canPower
                    ? null
                    : () => widget.onAction('Power'),
              ),
              selector(
                'Mode',
                Icon(
                  switch (mode == null ? null : c.observed(mode.property)) {
                    'cool' => Icons.ac_unit,
                    'heat' => Icons.wb_sunny_outlined,
                    'dry' => Icons.water_drop_outlined,
                    'fan' => Icons.air,
                    _ => Icons.tune,
                  },
                  size: 28,
                  color: mode == null ? muted : climateBlue,
                ),
                mode,
              ),
              selector(
                'Fan speed',
                Glyph(
                  'fan',
                  size: 28,
                  color: fan == null ? muted : climateBlue,
                ),
                fan,
              ),
              selector(
                'Air direction',
                Icon(
                  Icons.swap_vert,
                  size: 28,
                  color: direction == null ? muted : climateBlue,
                ),
                direction,
              ),
            ],
          ),
          if (extras.isNotEmpty) ...[
            const SizedBox(height: 22),
            const Copy('More functions', size: 18, bold: true),
            const SizedBox(height: 12),
            ClimateGrid(
              children: [
                for (final f in extras)
                  selector(f.$1, Icon(f.$2, color: climateBlue), f.$3),
              ],
            ),
          ],
          if (c.busy) ...[
            const SizedBox(height: 12),
            Semantics(
              liveRegion: true,
              child: const Copy(
                'Updating device…',
                color: climateBlue,
                center: true,
              ),
            ),
          ],
          if (receipt != null && receipt['device_id'] == c.selectedDevice) ...[
            const SizedBox(height: 16),
            Semantics(
              liveRegion: true,
              child: ClimateSurface(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Copy(outcomeLabel(receipt['status']), bold: true),
                    const SizedBox(height: 6),
                    Copy(outcomeSummary(receipt['status']), size: 12),
                    TextButton(
                      onPressed: c.busy
                          ? null
                          : () => widget.onAction('View Details'),
                      child: const Copy('View Details'),
                    ),
                  ],
                ),
              ),
            ),
          ],
          if (remaining.isNotEmpty) ...[
            const SizedBox(height: 16),
            Blocks(
              remaining,
              onAction: widget.onAction,
              fields: {},
              choices: {},
              values: {},
            ),
          ],
          const SizedBox(height: 16),
          Row(
            children: [
              Expanded(
                child: c.snapshot == null
                    ? const Copy('No reading', size: 11, color: muted)
                    : Copy(
                        UiMessage('Read: {time}{stale}', {
                          'time': receiptTime(times.lastOrNull, compact: true),
                          'stale': UiMessage(stale ? ' · Stale' : ''),
                        }),
                        size: 11,
                        color: muted,
                      ),
              ),
              TextButton.icon(
                label: Text(localize(context, 'Refresh State')),
                onPressed: c.busy
                    ? null
                    : () => widget.onAction('Refresh State'),
                icon: const Icon(Icons.refresh),
              ),
            ],
          ),
          Wrap(
            alignment: WrapAlignment.spaceBetween,
            children: [
              TextButton(
                onPressed: showSupport,
                child: const Copy('Function support', size: 13),
              ),
              TextButton(
                onPressed: () => widget.onAction('Device information'),
                child: const Copy('Device information', size: 13),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
