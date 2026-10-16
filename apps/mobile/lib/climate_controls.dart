import 'package:flutter/material.dart';
import 'blocks.dart';
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
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    selectionEpoch++;
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) return;
    selectionEpoch++;
    final sheet = sheetContext;
    if (sheet != null && sheet.mounted) Navigator.of(sheet).pop();
  }

  Object choiceValue(ClimateChoice? choice) => choice == null
      ? 'Not supported yet'
      : propertyValue(
          c.observed(choice.property),
          choice.capability['properties']?[choice.parameter] as Json? ?? {},
          propertyId: choice.property,
        );

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
                ActionButton(
                  'Apply',
                  onPressed: selected == null
                      ? null
                      : () => Navigator.of(context).pop(selected),
                ),
                ActionButton(
                  'Cancel',
                  style: 'secondary',
                  onPressed: () => Navigator.of(context).pop(),
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

  @override
  Widget build(BuildContext context) {
    final mode = climateChoice(c, {'mode'});
    final direction = climateChoice(c, {
      'air_direction',
      'swing',
      'oscillation',
    });
    final fan = climateChoice(c, {'wind_speed', 'fan_speed'});
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
    final targetAction = c.capability('climate')?['actions']?['set'] as Json?;
    final canTarget =
        targetAction != null &&
        actionUnavailable(c.capability('climate')!, targetAction) == null &&
        targetAction['parameters']['properties']['target_temperature'] != null;
    final receipt = c.receipt;
    final observations = (c.snapshot?['observations'] as Json? ?? {}).values
        .whereType<Map>();
    final times =
        observations.map((v) => v['received_at']).whereType<String>().toList()
          ..sort();
    final stale = observations.any((v) => v['stale'] == true);

    Widget button(
      String title,
      IconData icon,
      ClimateChoice? choice,
    ) => OutlinedButton(
      key: ValueKey('climate-button:$title'),
      style: OutlinedButton.styleFrom(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 16),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(18)),
        backgroundColor: const Color(0xfff8f5ee),
      ),
      onPressed: c.busy || choice == null ? null : () => choose(title, choice),
      child: Column(
        children: [
          Icon(icon, size: 26, color: choice == null ? muted : accent),
          const SizedBox(height: 8),
          Copy(title, bold: true, center: true),
          const SizedBox(height: 5),
          Copy(choiceValue(choice), size: 12, color: muted, center: true),
        ],
      ),
    );
    return Column(
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
        Surface(
          dark: true,
          padding: const EdgeInsets.all(20),
          child: Column(
            children: [
              Copy(
                power is bool
                    ? (power ? 'Power on' : 'Power off')
                    : 'Power unknown',
                color: Colors.white,
                bold: true,
              ),
              const SizedBox(height: 12),
              InkWell(
                onTap: !canTarget || c.busy
                    ? null
                    : () => widget.onAction('Target temperature'),
                child: Copy(
                  target is num
                      ? '${target}°C'
                      : propertyValue(
                          c.observed('climate.current_temperature'),
                          {'unit': 'celsius'},
                        ),
                  size: 44,
                  color: Colors.white,
                  center: true,
                ),
              ),
              Copy(
                target is num ? 'Target temperature' : 'Current temperature',
                color: Colors.white70,
                size: 12,
              ),
              if (canTarget)
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    IconButton(
                      tooltip: localize(context, 'Decrease target'),
                      onPressed: c.busy
                          ? null
                          : () => widget.onAction('Decrease target'),
                      icon: const Icon(Icons.remove, color: Colors.white),
                    ),
                    IconButton(
                      tooltip: localize(context, 'Increase target'),
                      onPressed: c.busy
                          ? null
                          : () => widget.onAction('Increase target'),
                      icon: const Icon(Icons.add, color: Colors.white),
                    ),
                  ],
                ),
              const SizedBox(height: 12),
              FilledButton.icon(
                key: const ValueKey('climate-power'),
                onPressed: c.busy || !canPower
                    ? null
                    : () => widget.onAction('Power'),
                icon: const Icon(Icons.power_settings_new),
                label: Copy(
                  power == true ? 'Turn off' : 'Turn on',
                  color: Colors.white,
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        LayoutBuilder(
          builder: (context, constraints) {
            final columns =
                constraints.maxWidth >= 300 &&
                    MediaQuery.textScalerOf(context).scale(14) <= 21
                ? 3
                : 1;
            return Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                for (final item in [
                  ('Mode', Icons.ac_unit, mode),
                  ('Air direction', Icons.air, direction),
                  ('Fan speed', Icons.toys, fan),
                ])
                  SizedBox(
                    width: (constraints.maxWidth - (columns - 1) * 8) / columns,
                    child: button(item.$1, item.$2, item.$3),
                  ),
              ],
            );
          },
        ),
        const SizedBox(height: 16),
        if (c.snapshot != null)
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Copy(
              UiMessage('Read: {time}{stale}', {
                'time': receiptTime(times.lastOrNull, compact: true),
                'stale': UiMessage(stale ? ' · Stale' : ''),
              }),
              size: 12,
              color: muted,
            ),
          ),
        if (c.busy)
          Semantics(liveRegion: true, child: const Copy('Updating device…')),
        if (receipt != null && receipt['device_id'] == c.selectedDevice) ...[
          Semantics(
            liveRegion: true,
            child: Surface(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Copy(outcomeLabel(receipt['status']), bold: true),
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
          const SizedBox(height: 12),
        ],
        Blocks(
          describedControls(
            c,
            excludeActions: {
              for (final choice in [
                mode,
                direction,
                fan,
              ].whereType<ClimateChoice>())
                '${choice.capability['id']}.${choice.action}',
            },
            excludeProperties: {
              for (final choice in [
                mode,
                direction,
                fan,
              ].whereType<ClimateChoice>())
                choice.property,
            },
          ),
          onAction: widget.onAction,
          fields: {},
          choices: {},
          values: {},
        ),
        ActionButton(
          'Refresh State',
          style: 'secondary',
          onPressed: c.busy ? null : () => widget.onAction('Refresh State'),
        ),
        ActionButton(
          'Device information',
          style: 'secondary',
          onPressed: () => widget.onAction('Device information'),
        ),
      ],
    );
  }
}
