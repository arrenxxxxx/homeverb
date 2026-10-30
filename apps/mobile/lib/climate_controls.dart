import 'package:flutter/material.dart';

import 'blocks.dart';
import 'climate_capabilities.dart';
import 'climate_panel.dart';
import 'controller.dart';
import 'controls.dart';
import 'device_typography.dart';
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

  Object? choiceValue(ClimateChoice choice) {
    final observed = c.observed(choice.property);
    if (observed != null && choice.values.contains(observed)) return null;
    final value = propertyValue(
      observed,
      choice.capability['properties']?[choice.parameter] as Json? ?? {},
      propertyId: choice.property,
    );
    return observed == null
        ? UiMessage(value)
        : UiMessage('Unrecognized ({value})', {'value': value});
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
    (
      'Cool energy saving',
      {'cool_energy_saving'},
      Icons.energy_savings_leaf_outlined,
    ),
    ('Strong cooling', {'strong_cooling'}, Icons.ac_unit),
    ('Light sensing', {'light_sensing'}, Icons.light_mode_outlined),
    (
      'Comfort energy saving',
      {'comfort_energy_saving'},
      Icons.energy_savings_leaf_outlined,
    ),
    ('Breezeless', {'breezeless'}, Icons.air),
    ('Boost', {'boost'}, Icons.bolt_outlined),
    ('Avoid direct airflow', {'avoid_direct_airflow'}, Icons.air),
    ('Prevent overcooling', {'prevent_overcooling'}, Icons.ac_unit),
    ('Follow airflow', {'follow_airflow'}, Icons.person_outline),
    ('Dry after shutdown', {'dry_after_shutdown'}, Icons.dry_outlined),
    ('Auxiliary heat', {'auxiliary_heat'}, Icons.waves),
    ('Lower horizontal swing', {'lower_horizontal_swing'}, Icons.swap_horiz),
    (
      'Odor removal',
      {'fresh_air_remove_odor', 'fengguan_remove_odor'},
      Icons.air,
    ),
    (
      'Electrostatic dust removal',
      {'elec_dust_remove'},
      Icons.filter_alt_outlined,
    ),
    (
      'Automatic fresh air',
      {'auto_comfort_fresh_air'},
      Icons.auto_awesome_outlined,
    ),
    ('Fresh air CO2 control', {'auto_fresh_off_co2'}, Icons.co2),
    ('Fresh air', {'comfort_fresh_air'}, Icons.air),
    ('Fresh air fan speed', {'fresh_air_fan_speed'}, Icons.speed),
    ('Manual humidity', {'manul_humi'}, Icons.water_drop_outlined),
    ('Target humidity', {'manul_humi_value'}, Icons.water_drop_outlined),
    (
      'Purifier PM threshold',
      {'auto_purifier_on_pm'},
      Icons.filter_alt_outlined,
    ),
  ];

  bool isSwitch(ClimateChoice choice) {
    final options = choice.available(c);
    return options.length == 2 &&
        options.contains(true) &&
        options.contains(false) &&
        c.observed(choice.property) is bool;
  }

  Widget scalar(ClimateChoice choice, {bool Function()? canInteract}) {
    final interactive = enabled && (canInteract?.call() ?? true);
    final epoch = c.controlEpoch, lifecycle = selectionEpoch;
    void choose(Object value) {
      if (canInteract?.call() == false) return;
      send(choice, value, epoch, lifecycle);
    }

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
        child: Semantics(
          label: localize(context, controlLabel(choice.parameter)),
          child: Switch(
            key: ValueKey('climate-switch:' + choice.parameter),
            value: observed,
            onChanged: interactive ? (value) => choose(value) : null,
          ),
        ),
      );
    }
    if (choice.numeric &&
        options.length == choice.values.length &&
        !(const {
              'wind_speed',
              'fan_speed',
              'speed',
            }.contains(choice.parameter) &&
            options.length <= 12)) {
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
            onPressed: interactive && valid && options.contains(observed - step)
                ? () => choose(observed - step)
                : null,
            icon: const Icon(Icons.remove),
          ),
          Expanded(
            child: Copy(
              choice.numeric
                  ? UiMessage('Value {value}', {
                      'value': propertyValue(
                        observed,
                        property,
                        propertyId: choice.property,
                      ),
                    })
                  : propertyValue(
                      observed,
                      property,
                      propertyId: choice.property,
                    ),
              center: true,
              bold: true,
              size: DeviceTypography.body,
              lineHeight: DeviceTypography.lineHeight,
            ),
          ),
          IconButton.filled(
            key: ValueKey('climate-increase:' + choice.parameter),
            tooltip:
                localize(context, 'Increase') +
                ' ' +
                localize(context, controlLabel(choice.parameter)),
            onPressed: interactive && valid && options.contains(observed + step)
                ? () => choose(observed + step)
                : null,
            icon: const Icon(Icons.add),
          ),
        ],
      );
    }
    // An enum is a set of presets, never an inferred percentage or speed order.
    String optionLabel(Object option) => localize(
      context,
      choice.numeric &&
              const {
                'wind_speed',
                'fan_speed',
                'speed',
              }.contains(choice.parameter)
          ? UiMessage('Level {value}', {'value': '$option'})
          : propertyValue(option, {}, propertyId: choice.property),
    );
    final isFanChoice = const {
      'wind_speed',
      'fan_speed',
      'speed',
    }.contains(choice.parameter);
    Widget chip(Object option) => ChoiceChip(
      key: ValueKey('climate-option:' + choice.parameter + ':' + '$option'),
      label: Text(optionLabel(option)),
      selected: observed == option,
      selectedColor: MediaQuery.highContrastOf(context) ? Colors.black : accent,
      labelStyle: TextStyle(
        color: observed == option ? Colors.white : ink,
        fontSize: DeviceTypography.body,
        height: DeviceTypography.lineHeight,
      ),
      padding: EdgeInsets.symmetric(
        horizontal: isFanChoice ? 0 : 4,
        vertical: 8,
      ),
      labelPadding: EdgeInsets.symmetric(horizontal: isFanChoice ? 4 : 6),
      side: MediaQuery.highContrastOf(context)
          ? const BorderSide(color: Colors.black)
          : BorderSide.none,
      backgroundColor: const Color(0xffe9e7e0),
      // Keep measured row width stable while revealing an observed selection.
      chipAnimationStyle: isFanChoice
          ? ChipAnimationStyle(
              avatarDrawerAnimation: AnimationStyle.noAnimation,
            )
          : null,
      showCheckmark: true,
      checkmarkColor: Colors.white,
      materialTapTargetSize: MaterialTapTargetSize.padded,
      onSelected: interactive
          ? (_) {
              if (option != c.observed(choice.property)) {
                choose(option);
              }
            }
          : null,
    );
    return LayoutBuilder(
      builder: (context, constraints) {
        final chips = [
          for (final option in options)
            if (isFanChoice)
              ConstrainedBox(
                constraints: const BoxConstraints(minWidth: 44, minHeight: 44),
                child: chip(option),
              )
            else
              chip(option),
        ];
        final isFan = const {
          'wind_speed',
          'fan_speed',
          'speed',
        }.contains(choice.parameter);
        if (!isFan || !constraints.hasBoundedWidth) {
          return Wrap(spacing: 8, runSpacing: 8, children: chips);
        }
        var requiredWidth = 0.0;
        for (final option in options) {
          final painter = TextPainter(
            text: TextSpan(
              text: optionLabel(option),
              style: DefaultTextStyle.of(context).style.copyWith(
                fontSize: DeviceTypography.body,
                height: DeviceTypography.lineHeight,
              ),
            ),
            textDirection: Directionality.of(context),
            textScaler: MediaQuery.textScalerOf(context),
          )..layout();
          // Compact padding; retain readable labels and the selected checkmark.
          final naturalWidth =
              painter.width + 16 + (observed == option ? 24 : 0);
          final width = naturalWidth < 44 ? 44.0 : naturalWidth;
          requiredWidth += width;
          painter.dispose();
        }
        return _FanPresetRow(
          children: chips,
          selectedIndex: options.indexOf(observed),
          scrollable:
              requiredWidth + (options.length - 1) * 4 > constraints.maxWidth,
        );
      },
    );
  }

  Widget card(Widget child, {EdgeInsets padding = const EdgeInsets.all(12)}) =>
      Material(
        color: Colors.white,
        borderRadius: BorderRadius.circular(16),
        child: Container(
          padding: padding,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(16),
            border: Border.all(
              color: MediaQuery.highContrastOf(context)
                  ? Colors.black
                  : const Color(0xffefede8),
            ),
          ),
          child: child,
        ),
      );

  Widget row(String title, IconData icon, ClimateChoice choice) => Column(
    key: ValueKey('climate-row:$title'),
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      Row(
        children: [
          if (updating(choice))
            const OperationPulse(size: 20)
          else if (title != 'Horizontal swing' && title != 'Vertical swing')
            Icon(icon, color: ink, size: 20),
          if (updating(choice) ||
              (title != 'Horizontal swing' && title != 'Vertical swing'))
            const SizedBox(width: 8),
          Expanded(
            child: Wrap(
              crossAxisAlignment: WrapCrossAlignment.center,
              spacing: 6,
              children: [
                Copy(
                  title,
                  bold: true,
                  size: title == 'Horizontal swing' || title == 'Vertical swing'
                      ? DeviceTypography.body
                      : DeviceTypography.section,
                  lineHeight: DeviceTypography.lineHeight,
                ),
                if (!isSwitch(choice))
                  const Copy(
                    'Choose one',
                    size: DeviceTypography.secondary,
                    color: muted,
                    lineHeight: DeviceTypography.lineHeight,
                  ),
              ],
            ),
          ),
          if (isSwitch(choice)) scalar(choice),
        ],
      ),
      if (choiceValue(choice) case final value?) ...[
        const SizedBox(height: 4),
        Copy(
          value,
          size: DeviceTypography.secondary,
          color: muted,
          lineHeight: DeviceTypography.lineHeight,
        ),
      ],
      if (!isSwitch(choice)) ...[
        if (choice.action == 'activate_mode') ...[
          const SizedBox(height: 4),
          const Copy(
            'Selecting a mode also turns this device on.',
            size: DeviceTypography.secondary,
            lineHeight: DeviceTypography.lineHeight,
          ),
        ],
        const SizedBox(height: 4),
        scalar(choice),
      ],
    ],
  );

  Widget airflow(
    String title,
    IconData icon,
    ClimateChoice? swing,
    ClimateChoice? direction,
  ) {
    final choice = direction!;
    final label = title == 'Horizontal swing'
        ? 'Horizontal direction'
        : 'Vertical direction';
    return Column(
      key: ValueKey('climate-row:$label'),
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            Expanded(
              child: Copy(
                label,
                bold: true,
                size: DeviceTypography.section,
                lineHeight: DeviceTypography.lineHeight,
              ),
            ),
            if (updating(choice)) const OperationPulse(size: 20),
          ],
        ),
        if (choiceValue(choice) case final value?)
          Copy(
            value,
            size: DeviceTypography.secondary,
            color: muted,
            lineHeight: DeviceTypography.lineHeight,
          ),
        if (swing != null && c.observed(swing.property) != false)
          Copy(
            c.observed(swing.property) == true
                ? 'Turn off swing before choosing a fixed direction.'
                : 'Refresh swing state before choosing a fixed direction.',
            size: DeviceTypography.secondary,
            color: muted,
            lineHeight: DeviceTypography.lineHeight,
          ),
        scalar(
          choice,
          canInteract: () =>
              swing == null || c.observed(swing.property) == false,
        ),
      ],
    );
  }

  Widget directionSummary(String label, ClimateChoice? swing) {
    final observed = swing == null ? false : c.observed(swing.property);
    final status = observed == true
        ? 'Unavailable while swing is on.'
        : observed == false
        ? 'Adjust in Other settings.'
        : 'Swing state is unknown.';
    return Padding(
      key: ValueKey('climate-direction-summary:$label'),
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Wrap(
        spacing: 6,
        runSpacing: 2,
        children: [
          Copy(
            label,
            size: DeviceTypography.secondary,
            lineHeight: DeviceTypography.lineHeight,
          ),
          Copy(
            status,
            size: DeviceTypography.secondary,
            color: muted,
            lineHeight: DeviceTypography.lineHeight,
          ),
        ],
      ),
    );
  }

  Widget fineTemperature(ClimateChoice targetChoice) {
    final target = c.observed('climate.target_temperature');
    final options = targetChoice.available(c);
    if (!targetChoice.numeric ||
        target is! num ||
        options.length != targetChoice.values.length ||
        !options.contains(target))
      return const SizedBox.shrink();
    final schema = targetChoice.schema;
    final step = schema['multipleOf'] as num;
    final epoch = c.controlEpoch, lifecycle = selectionEpoch;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const Copy(
          'Target temperature',
          bold: true,
          size: DeviceTypography.body,
          lineHeight: DeviceTypography.lineHeight,
        ),
        Slider(
          key: const ValueKey('climate-temperature-slider'),
          value: draftTarget ?? target.toDouble(),
          min: (schema['minimum'] as num).toDouble(),
          max: (schema['maximum'] as num).toDouble(),
          divisions: ((schema['maximum'] - schema['minimum']) / step).round(),
          label: (draftTarget ?? target).toString() + '°C',
          semanticFormatterCallback: (v) => '$v' + '°C',
          onChangeStart: !enabled ? null : (_) => targetGestureEpoch = epoch,
          onChanged: !enabled
              ? null
              : (v) {
                  if (epoch == c.controlEpoch && lifecycle == selectionEpoch)
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
        ),
      ],
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
    final white = readableInk(context, ink);
    final soft = readableInk(context, muted);
    Widget minus() => IconButton.filledTonal(
      key: const ValueKey('climate-decrease'),
      tooltip: localize(context, 'Decrease target'),
      style: IconButton.styleFrom(
        backgroundColor: const Color(0xffedeae4),
        foregroundColor: white,
        minimumSize: const Size(56, 56),
      ),
      onPressed: enabled && bounded && options.contains(target - step!)
          ? () => send(targetChoice, target - step, epoch, lifecycle)
          : null,
      icon: const Icon(Icons.remove),
    );
    Widget plus() => IconButton.filled(
      key: const ValueKey('climate-increase'),
      tooltip: localize(context, 'Increase target'),
      style: IconButton.styleFrom(minimumSize: const Size(56, 56)),
      onPressed: enabled && bounded && options.contains(target + step!)
          ? () => send(targetChoice, target + step, epoch, lifecycle)
          : null,
      icon: const Icon(Icons.add),
    );
    final number = FittedBox(
      fit: BoxFit.scaleDown,
      child: Text(
        (target is num
                ? (draftTarget ?? target).toString().replaceFirst(
                    RegExp(r'\.0$'),
                    '',
                  )
                : '—') +
            '°C',
        textAlign: TextAlign.center,
        style: TextStyle(
          fontSize: 64,
          fontWeight: FontWeight.w600,
          height: DeviceTypography.lineHeight,
          color: white,
        ),
      ),
    );
    return card(
      Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              const Icon(Icons.power_settings_new, size: 20),
              const SizedBox(width: 8),
              const Copy(
                'Power',
                bold: true,
                size: DeviceTypography.body,
                lineHeight: DeviceTypography.lineHeight,
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Copy(
                  updating(powerChoice)
                      ? (pending!.$2 == true ? 'Turning on…' : 'Turning off…')
                      : power == true
                      ? 'Powered on'
                      : power == false
                      ? 'Powered off'
                      : 'Power unknown',
                  size: DeviceTypography.secondary,
                  color: soft,
                  lineHeight: DeviceTypography.lineHeight,
                ),
              ),
              if (updating(powerChoice)) const OperationPulse(size: 20),
              if (power is bool &&
                  powerChoice != null &&
                  powerChoice.permits(c, !power))
                Semantics(
                  label: localize(context, 'Power'),
                  child: Switch(
                    key: const ValueKey('climate-power'),
                    value: power,
                    onChanged: enabled
                        ? (value) => send(powerChoice, value, epoch, lifecycle)
                        : null,
                  ),
                ),
            ],
          ),
          const SizedBox(height: 4),
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
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Copy(
                'Target temperature',
                color: soft,
                size: 14,
                lineHeight: DeviceTypography.lineHeight,
              ),
              if (updating(targetChoice)) ...[
                const SizedBox(width: 6),
                const OperationPulse(size: 16),
              ],
            ],
          ),
          if (!bounded && targetChoice != null) ...[
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
            size: 14,
            center: true,
            lineHeight: DeviceTypography.lineHeight,
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final epoch = c.controlEpoch, lifecycle = selectionEpoch;
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
    final swingRows = [
      if (horizontal != null)
        row('Horizontal swing', Icons.swap_horiz, horizontal),
      if (vertical != null) row('Vertical swing', Icons.swap_vert, vertical),
    ];
    final advanced = <Widget>[
      if (target != null) fineTemperature(target),
      if (horizontalDirection != null)
        airflow(
          'Horizontal swing',
          Icons.swap_horiz,
          horizontal,
          horizontalDirection,
        ),
      if (verticalDirection != null)
        airflow('Vertical swing', Icons.swap_vert, vertical, verticalDirection),
      if (direction != null) row('Air direction', Icons.air, direction),
      for (final f in extras) row(f.$1, f.$2, f.$3),
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
        if (mode != null) ...[
          const SizedBox(height: 8),
          card(row('Mode', Icons.ac_unit, mode)),
        ],
        if (fan != null) ...[
          const SizedBox(height: 8),
          card(row('Fan speed', Icons.air, fan)),
        ],
        if (swingRows.isNotEmpty ||
            horizontalDirection != null ||
            verticalDirection != null) ...[
          const SizedBox(height: 8),
          card(
            Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                const Copy(
                  'Swing',
                  bold: true,
                  size: DeviceTypography.section,
                  lineHeight: DeviceTypography.lineHeight,
                ),
                const SizedBox(height: 4),
                LayoutBuilder(
                  builder: (context, constraints) {
                    if (constraints.maxWidth < 320 ||
                        MediaQuery.textScalerOf(context).scale(14) > 20)
                      return Column(children: swingRows);
                    return Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        for (var i = 0; i < swingRows.length; i++) ...[
                          if (i > 0) const SizedBox(width: 12),
                          Expanded(child: swingRows[i]),
                        ],
                      ],
                    );
                  },
                ),
                if (horizontalDirection != null ||
                    verticalDirection != null) ...[
                  const Divider(height: 16),
                  if (horizontalDirection != null)
                    directionSummary('Horizontal direction', horizontal),
                  if (verticalDirection != null)
                    directionSummary('Vertical direction', vertical),
                ],
              ],
            ),
          ),
        ],
        if (advanced.isNotEmpty) ...[
          const SizedBox(height: 8),
          card(
            ExpansionTile(
              key: const ValueKey('climate-more-settings'),
              tilePadding: EdgeInsets.zero,
              childrenPadding: EdgeInsets.zero,
              dense: true,
              visualDensity: VisualDensity.compact,
              title: const Copy(
                'Other settings',
                bold: true,
                size: DeviceTypography.section,
                lineHeight: DeviceTypography.lineHeight,
              ),
              leading: const Icon(Icons.tune, size: 20),
              children: [
                for (var i = 0; i < advanced.length; i++) ...[
                  if (i > 0) const Divider(height: 20),
                  advanced[i],
                ],
              ],
            ),
            padding: const EdgeInsets.symmetric(horizontal: 12),
          ),
        ],
        if (receipt != null &&
            receipt['device_id'] == c.selectedDevice &&
            pending == null) ...[
          const SizedBox(height: 16),
          Semantics(
            liveRegion: true,
            child: ClimateSurface(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Copy(
                    outcomeLabel(receipt['status']),
                    bold: true,
                    size: DeviceTypography.section,
                    lineHeight: DeviceTypography.lineHeight,
                  ),
                  const SizedBox(height: 6),
                  Copy(
                    outcomeSummary(receipt['status']),
                    size: DeviceTypography.secondary,
                    lineHeight: DeviceTypography.lineHeight,
                  ),
                  TextButton(
                    onPressed: c.busy
                        ? null
                        : () => widget.onAction('View Details'),
                    child: const Copy(
                      'View Details',
                      size: DeviceTypography.body,
                      lineHeight: DeviceTypography.lineHeight,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
        const SizedBox(height: 16),
        ActionButton(
          'Refresh State',
          key: const ValueKey('climate-refresh'),
          style: 'secondary',
          onPressed: !enabled
              ? null
              : () {
                  if (!mounted ||
                      !enabled ||
                      epoch != c.controlEpoch ||
                      lifecycle != selectionEpoch)
                    return;
                  widget.onAction('Refresh State');
                },
        ),
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

/// Keeps every admitted fan preset on one line without shrinking device labels.
class _FanPresetRow extends StatefulWidget {
  const _FanPresetRow({
    required this.children,
    required this.selectedIndex,
    required this.scrollable,
  });
  final List<Widget> children;
  final int selectedIndex;
  final bool scrollable;
  @override
  State<_FanPresetRow> createState() => _FanPresetRowState();
}

class _FanPresetRowState extends State<_FanPresetRow> {
  final controller = ScrollController();
  final itemKeys = <GlobalKey>[];
  bool canBack = false, canForward = true;
  double? lastWidth, lastScale;

  @override
  void initState() {
    super.initState();
    controller.addListener(updateEdges);
    revealSelected();
  }

  @override
  void didUpdateWidget(_FanPresetRow oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.selectedIndex != oldWidget.selectedIndex ||
        widget.children.length != oldWidget.children.length ||
        widget.scrollable != oldWidget.scrollable)
      revealSelected();
  }

  void updateEdges() {
    if (!mounted || !controller.hasClients) return;
    final back = controller.offset > .5;
    final forward =
        controller.offset < controller.position.maxScrollExtent - .5;
    if (back != canBack || forward != canForward) {
      setState(() {
        canBack = back;
        canForward = forward;
      });
    }
  }

  void revealSelected() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || !controller.hasClients) return;
      final index = widget.selectedIndex;
      if (index >= 0 && index < itemKeys.length) {
        final object = itemKeys[index].currentContext?.findRenderObject();
        if (object != null)
          controller.position.ensureVisible(object, alignment: .5);
      }
      updateEdges();
    });
  }

  void scroll(double direction) {
    if (!controller.hasClients) return;
    controller.animateTo(
      (controller.offset +
              direction * controller.position.viewportDimension * .75)
          .clamp(0.0, controller.position.maxScrollExtent),
      duration: const Duration(milliseconds: 180),
      curve: Curves.easeOut,
    );
  }

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) {
      while (itemKeys.length < widget.children.length) {
        itemKeys.add(GlobalKey());
      }
      final scale = MediaQuery.textScalerOf(
        context,
      ).scale(DeviceTypography.body);
      if (lastWidth != constraints.maxWidth || lastScale != scale) {
        lastWidth = constraints.maxWidth;
        lastScale = scale;
        revealSelected();
      }
      final content = Row(
        children: [
          for (var i = 0; i < widget.children.length; i++) ...[
            if (i > 0) const SizedBox(width: 8),
            KeyedSubtree(key: itemKeys[i], child: widget.children[i]),
          ],
        ],
      );
      if (!widget.scrollable) return content;
      return Row(
        children: [
          IconButton(
            key: const ValueKey('climate-fan-scroll-back'),
            tooltip: localize(context, 'Previous fan speeds'),
            constraints: const BoxConstraints(minWidth: 44, minHeight: 44),
            onPressed: canBack ? () => scroll(-1) : null,
            icon: const Icon(Icons.chevron_left),
          ),
          Expanded(
            child: SingleChildScrollView(
              key: const ValueKey('climate-fan-scroll'),
              controller: controller,
              scrollDirection: Axis.horizontal,
              child: content,
            ),
          ),
          IconButton(
            key: const ValueKey('climate-fan-scroll-forward'),
            tooltip: localize(context, 'More fan speeds'),
            constraints: const BoxConstraints(minWidth: 44, minHeight: 44),
            onPressed: canForward ? () => scroll(1) : null,
            icon: const Icon(Icons.chevron_right),
          ),
        ],
      );
    },
  );
}
