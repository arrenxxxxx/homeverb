import 'package:flutter/material.dart';
import 'blocks.dart';
import 'climate_capabilities.dart';
import 'climate_panel.dart';
import 'controller.dart';
import 'controls.dart';
import 'localization.dart';
import 'client_contract.dart';
import 'receipts.dart';
import 'skin.dart';

export 'climate_capabilities.dart';

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
  int selectionEpoch = 0;
  double? draftTarget;
  int? targetGestureEpoch;
  (String, Object)? pending;
  bool get enabled => pending == null && !c.busy && c.isForeground;
  bool updating(ClimateChoice? choice) =>
      choice != null && pending?.$1 == choice.signature;
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
    targetGestureEpoch = null;
    if (mounted) setState(() => draftTarget = null);
  }

  Object choiceValue(ClimateChoice choice) {
    final observed = c.observed(choice.property);
    final value = propertyValue(
      observed,
      choice.capability['properties']?[choice.parameter] as Json? ?? {},
      propertyId: choice.property,
    );
    return UiMessage('Observed: {value}', {
      'value': observed != null && !choice.values.contains(observed)
          ? UiMessage('Unrecognized ({value})', {'value': value})
          : UiMessage(value),
    });
  }

  Future<void> send(
    ClimateChoice choice,
    Object value,
    int epoch,
    int lifecycle,
  ) async {
    if (!mounted ||
        !enabled ||
        !c.isForeground ||
        epoch != c.controlEpoch ||
        lifecycle != selectionEpoch)
      return;
    final current = climateChoice(
      c,
      {choice.parameter},
      capabilityIds: {choice.capability['id']},
    );
    if (current == null ||
        current.signature != choice.signature ||
        !current.permits(c, value))
      return;
    setState(() => pending = (choice.signature, value));
    try {
      await widget.onExecute(choice.capability['id'], choice.action, {
        choice.parameter: value,
      });
    } finally {
      if (mounted) setState(() => pending = null);
    }
  }

  static const features = <(String, Set<String>, IconData)>[
    ('Display light', {'display', 'display_light'}, Icons.light_mode_outlined),
    (
      'Button sound',
      {'button_sound', 'sound', 'buzzer'},
      Icons.volume_up_outlined,
    ),
    ('Sleep', {'sleep'}, Icons.bedtime_outlined),
    ('Energy saving', {'energy_saving', 'eco'}, Icons.eco_outlined),
    ('Boost', {'boost'}, Icons.bolt_outlined),
    ('Avoid direct airflow', {'avoid_direct_airflow'}, Icons.air),
    ('Auxiliary heat', {'auxiliary_heat'}, Icons.waves),
  ];

  bool isSwitch(ClimateChoice choice) {
    final options = choice.available(c);
    return options.length == 2 &&
        options.contains(true) &&
        options.contains(false) &&
        c.observed(choice.property) is bool;
  }

  Widget scalar(ClimateChoice choice) {
    final epoch = c.controlEpoch, lifecycle = selectionEpoch;
    final available = choice.available(c);
    const modeOrder = ['auto', 'cool', 'heat', 'dry', 'fan'];
    final options = choice.parameter == 'mode'
        ? <Object>[
            ...modeOrder.where(available.contains),
            ...available.where((v) => !modeOrder.contains(v)),
          ]
        : available;
    final observed = c.observed(choice.property);
    if (options.length == 2 &&
        options.contains(true) &&
        options.contains(false) &&
        observed is bool) {
      return Align(
        alignment: Alignment.centerRight,
        widthFactor: 1,
        child: Switch(
          key: ValueKey('climate-switch:' + choice.parameter),
          value: observed,
          onChanged: enabled
              ? (value) => send(choice, value, epoch, lifecycle)
              : null,
        ),
      );
    }
    if (choice.numeric && options.length == choice.values.length) {
      final valid = observed is num && options.contains(observed);
      final step = choice.schema['multipleOf'] as num;
      final property =
          choice.capability['properties']?[choice.parameter] as Json? ?? {};
      return Row(
        key: ValueKey('climate-stepper:' + choice.parameter),
        children: [
          IconButton.filledTonal(
            key: ValueKey('climate-decrease:' + choice.parameter),
            tooltip:
                localize(context, 'Decrease') +
                ' ' +
                localize(context, controlLabel(choice.parameter)),
            onPressed: enabled && valid && options.contains(observed - step)
                ? () => send(choice, observed - step, epoch, lifecycle)
                : null,
            icon: const Icon(Icons.remove),
          ),
          Expanded(
            child: Copy(
              propertyValue(observed, property, propertyId: choice.property),
              center: true,
              bold: true,
            ),
          ),
          IconButton.filled(
            key: ValueKey('climate-increase:' + choice.parameter),
            tooltip:
                localize(context, 'Increase') +
                ' ' +
                localize(context, controlLabel(choice.parameter)),
            onPressed: enabled && valid && options.contains(observed + step)
                ? () => send(choice, observed + step, epoch, lifecycle)
                : null,
            icon: const Icon(Icons.add),
          ),
        ],
      );
    }
    // An enum is a set of presets, never an inferred percentage or speed order.
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: [
        for (final option in options)
          ChoiceChip(
            key: ValueKey(
              'climate-option:' + choice.parameter + ':' + '$option',
            ),
            label: Text(
              localize(
                context,
                propertyValue(option, {}, propertyId: choice.property),
              ),
            ),
            selected: observed == option,
            selectedColor: MediaQuery.highContrastOf(context)
                ? Colors.black
                : accent,
            labelStyle: TextStyle(
              color: observed == option ? Colors.white : ink,
              fontSize: 13,
            ),
            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 8),
            labelPadding: const EdgeInsets.symmetric(horizontal: 6),
            side: MediaQuery.highContrastOf(context)
                ? const BorderSide(color: Colors.black)
                : BorderSide.none,
            backgroundColor: const Color(0xffe9e7e0),
            showCheckmark: false,
            materialTapTargetSize: MaterialTapTargetSize.padded,
            onSelected: enabled
                ? (_) => send(choice, option, epoch, lifecycle)
                : null,
          ),
      ],
    );
  }

  Widget row(String title, IconData icon, ClimateChoice choice) => ClimateRow(
    key: ValueKey('climate-row:$title'),
    title: title,
    icon: icon,
    value: choiceValue(choice),
    updating: updating(choice),
    trailing: isSwitch(choice) ? scalar(choice) : null,
    child: isSwitch(choice)
        ? null
        : Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              if (choice.action == 'activate_mode') ...[
                const Copy(
                  'Selecting a mode also turns this device on.',
                  size: 12,
                ),
                const SizedBox(height: 10),
              ],
              scalar(choice),
            ],
          ),
  );

  Widget airflow(
    String title,
    IconData icon,
    ClimateChoice? swing,
    ClimateChoice? direction,
  ) {
    if (swing == null)
      return row(title.replaceFirst('swing', 'direction'), icon, direction!);
    return ClimateRow(
      key: ValueKey('climate-row:$title'),
      title: title,
      icon: icon,
      value: choiceValue(swing),
      updating: updating(swing) || updating(direction),
      trailing: isSwitch(swing) ? scalar(swing) : null,
      child: isSwitch(swing) && direction == null
          ? null
          : Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                if (!isSwitch(swing)) scalar(swing),
                if (direction != null) ...[
                  const SizedBox(height: 12),
                  Copy(
                    title == 'Horizontal swing'
                        ? 'Horizontal direction'
                        : 'Vertical direction',
                    bold: true,
                  ),
                  const SizedBox(height: 8),
                  Copy(choiceValue(direction), size: 12, color: muted),
                  const SizedBox(height: 8),
                  scalar(direction),
                ],
              ],
            ),
    );
  }

  Widget temperature(ClimateChoice? targetChoice, ClimateChoice? powerChoice) {
    final target = c.observed('climate.target_temperature');
    final power = c.observed('power.state');
    final epoch = c.controlEpoch, lifecycle = selectionEpoch;
    final options = targetChoice?.available(c) ?? [];
    final schema = targetChoice?.schema;
    final bounded =
        targetChoice != null &&
        targetChoice.numeric &&
        options.length == targetChoice.values.length &&
        target is num &&
        options.contains(target);
    final step = schema?['multipleOf'] as num?;
    final white = readableInk(context, const Color(0xfffffaf1));
    final soft = readableInk(context, const Color(0xffd8d6ce));
    Widget minus() => IconButton.filledTonal(
      key: const ValueKey('climate-decrease'),
      tooltip: localize(context, 'Decrease target'),
      style: IconButton.styleFrom(
        backgroundColor: const Color(0xff545653),
        foregroundColor: white,
      ),
      onPressed: enabled && bounded && options.contains(target - step!)
          ? () => send(targetChoice, target - step, epoch, lifecycle)
          : null,
      icon: const Icon(Icons.remove),
    );
    Widget plus() => IconButton.filled(
      key: const ValueKey('climate-increase'),
      tooltip: localize(context, 'Increase target'),
      onPressed: enabled && bounded && options.contains(target + step!)
          ? () => send(targetChoice, target + step, epoch, lifecycle)
          : null,
      icon: const Icon(Icons.add),
    );
    final number = Text(
      (target is num
              ? (draftTarget ?? target).toString().replaceFirst(
                  RegExp(r'\.0$'),
                  '',
                )
              : '—') +
          '°C',
      textAlign: TextAlign.center,
      style: TextStyle(fontSize: 52, fontWeight: FontWeight.w300, color: white),
    );
    return ClimateSurface(
      dark: true,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Wrap(
            alignment: WrapAlignment.spaceBetween,
            crossAxisAlignment: WrapCrossAlignment.center,
            spacing: 12,
            runSpacing: 8,
            children: [
              Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Copy('Target temperature', color: soft, size: 13),
                  if (updating(targetChoice)) ...[
                    const SizedBox(width: 8),
                    Semantics(
                      label: localize(context, 'Updating device…'),
                      liveRegion: true,
                      child: OperationPulse(color: white),
                    ),
                  ],
                ],
              ),
              if (power is bool &&
                  powerChoice != null &&
                  powerChoice.permits(c, !power))
                Tooltip(
                  message: localize(context, power ? 'Turn off' : 'Turn on'),
                  child: Semantics(
                    toggled: power,
                    liveRegion: updating(powerChoice),
                    child: FilledButton.icon(
                      key: const ValueKey('climate-power'),
                      style: FilledButton.styleFrom(
                        minimumSize: const Size(124, 40),
                        backgroundColor: powerColor(context, power),
                        foregroundColor:
                            MediaQuery.highContrastOf(context) && power
                            ? Colors.black
                            : white,
                        disabledBackgroundColor: powerColor(context, power),
                        disabledForegroundColor:
                            MediaQuery.highContrastOf(context) && power
                            ? Colors.black
                            : white,
                        side: MediaQuery.highContrastOf(context)
                            ? const BorderSide(color: Colors.white)
                            : BorderSide.none,
                      ),
                      onPressed: enabled
                          ? () => send(powerChoice, !power, epoch, lifecycle)
                          : null,
                      icon: updating(powerChoice)
                          ? OperationPulse(
                              color: MediaQuery.highContrastOf(context) && power
                                  ? Colors.black
                                  : white,
                            )
                          : Icon(
                              power
                                  ? Icons.power_settings_new
                                  : Icons.power_off_outlined,
                              size: 20,
                            ),
                      label: Text(
                        localize(
                          context,
                          updating(powerChoice)
                              ? (pending!.$2 == true
                                    ? 'Turning on…'
                                    : 'Turning off…')
                              : (power ? 'Powered on' : 'Powered off'),
                        ),
                      ),
                    ),
                  ),
                )
              else
                Copy(
                  power is bool
                      ? (power ? 'Power on' : 'Power off')
                      : 'Power unknown',
                  color: soft,
                  size: 13,
                ),
            ],
          ),
          const SizedBox(height: 12),
          if (MediaQuery.textScalerOf(context).scale(14) > 20) ...[
            number,
            if (bounded)
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [minus(), plus()],
              ),
          ] else
            Row(
              children: [
                if (bounded) minus(),
                Expanded(child: number),
                if (bounded) plus(),
              ],
            ),
          if (bounded)
            Slider(
              key: const ValueKey('climate-temperature-slider'),
              value: draftTarget ?? target.toDouble(),
              min: (schema!['minimum'] as num).toDouble(),
              max: (schema['maximum'] as num).toDouble(),
              divisions: ((schema['maximum'] - schema['minimum']) / step)
                  .round(),
              label: (draftTarget ?? target).toString() + '°C',
              semanticFormatterCallback: (v) => '$v' + '°C',
              onChangeStart: !enabled
                  ? null
                  : (_) => targetGestureEpoch = epoch,
              onChanged: !enabled
                  ? null
                  : (v) {
                      if (epoch == c.controlEpoch &&
                          lifecycle == selectionEpoch)
                        setState(() => draftTarget = v);
                    },
              onChangeEnd: !enabled
                  ? null
                  : (value) {
                      if (targetGestureEpoch != epoch ||
                          epoch != c.controlEpoch ||
                          lifecycle != selectionEpoch)
                        return;
                      targetGestureEpoch = null;
                      setState(() => draftTarget = null);
                      if (value != target)
                        send(targetChoice, value, epoch, lifecycle);
                    },
            )
          else if (targetChoice != null) ...[
            const SizedBox(height: 12),
            scalar(targetChoice),
          ],
          const SizedBox(height: 8),
          Copy(
            UiMessage('Indoor {temperature}', {
              'temperature': UiMessage(
                propertyValue(c.observed('climate.current_temperature'), {
                  'unit': 'celsius',
                }),
              ),
            }),
            color: soft,
            size: 12,
            center: true,
          ),
        ],
      ),
    );
  }

  Color powerColor(BuildContext context, bool power) =>
      MediaQuery.highContrastOf(context)
      ? (power ? Colors.white : Colors.black)
      : (power ? accent : const Color(0xff595d5a));

  @override
  Widget build(BuildContext context) {
    final mode = climateChoice(c, {'mode'});
    final fan = climateChoice(c, {'wind_speed', 'fan_speed', 'speed'});
    final horizontal = climateChoice(c, {
      'horizontal_swing',
      'swing_horizontal',
    });
    final horizontalDirection = climateChoice(c, {
      'horizontal_direction',
      'horizontal_angle',
      'horizontal_position',
    });
    final vertical = climateChoice(c, {'vertical_swing', 'swing_vertical'});
    final verticalDirection = climateChoice(c, {
      'vertical_direction',
      'vertical_angle',
      'vertical_position',
    });
    final direction = climateChoice(c, {
      'air_direction',
      'swing',
      'oscillation',
    });
    final target = climateChoice(
      c,
      {'target_temperature'},
      capabilityIds: {'climate'},
    );
    final power = climateChoice(c, {'state'}, capabilityIds: {'power'});
    final extras = [
      for (final f in features)
        if (climateChoice(c, f.$2) case final ClimateChoice choice)
          (f.$1, f.$3, choice),
    ];
    final primary = [
      mode,
      fan,
      horizontal,
      horizontalDirection,
      vertical,
      verticalDirection,
      direction,
      target,
      power,
      ...extras.map((f) => f.$3),
    ].whereType<ClimateChoice>().toList();
    final handledProperties = {for (final choice in primary) choice.property};
    final remaining = describedControls(
      c,
      excludeActions: {
        for (final choice in primary)
          if ((choice.capability['actions'][choice
                      .action]['parameters']['properties']
                  as Map)
              .keys
              .every(
                (p) => handledProperties.contains(
                  choice.capability['id'] + '.' + p,
                ),
              ))
            choice.capability['id'] + '.' + choice.action,
      },
      excludeProperties: {
        ...handledProperties,
        'climate.target_temperature',
        'climate.current_temperature',
      },
      hideUnavailable: true,
      includeClimate: true,
    );
    final rows = [
      if (mode != null) row('Mode', Icons.thermostat_outlined, mode),
      if (fan != null) row('Fan speed', Icons.air, fan),
      if (horizontal != null || horizontalDirection != null)
        airflow(
          'Horizontal swing',
          Icons.swap_horiz,
          horizontal,
          horizontalDirection,
        ),
      if (vertical != null || verticalDirection != null)
        airflow('Vertical swing', Icons.swap_vert, vertical, verticalDirection),
      if (direction != null) row('Air direction', Icons.air, direction),
    ];
    final receipt = c.receipt;
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
                  onSelected: !enabled
                      ? null
                      : (_) => widget.onAction('component:' + part['id']),
                ),
            ],
          ),
        if (c.busy && pending == null)
          const Padding(
            padding: EdgeInsets.only(bottom: 12),
            child: OperationProgress(label: 'Updating device…'),
          ),
        temperature(target, power),
        if (rows.isNotEmpty) ...[
          const SizedBox(height: 12),
          ClimateGroup(children: rows),
        ],
        if (extras.isNotEmpty) ...[
          const SizedBox(height: 20),
          const Copy('Other settings', size: 18, bold: true),
          const SizedBox(height: 10),
          ClimateGroup(
            children: [for (final f in extras) row(f.$1, f.$2, f.$3)],
          ),
        ],
        if (receipt != null &&
            receipt['device_id'] == c.selectedDevice &&
            !{'accepted', 'verified'}.contains(receipt['status'])) ...[
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
      ],
    );
  }
}
