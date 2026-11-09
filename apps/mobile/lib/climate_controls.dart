import 'package:flutter/material.dart';

import 'blocks.dart';
import 'appliance_panels.dart' show InlineDeviceInformation;
import 'climate_capabilities.dart';
import 'climate_panel.dart';
import 'controller.dart';
import 'controls.dart';
import 'device_typography.dart';
import 'device_readings.dart';
import 'device_option_grid.dart';
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
    ('Lower horizontal swing', {'lower_horizontal_swing'}, Icons.swap_horiz),
    ('Sleep', {'sleep'}, Icons.bedtime_outlined),
    ('Energy saving', {'energy_saving', 'eco'}, Icons.eco_outlined),
    (
      'Cool energy saving',
      {'cool_energy_saving'},
      Icons.energy_savings_leaf_outlined,
    ),
    ('Strong cooling', {'strong_cooling'}, Icons.ac_unit),
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
    ('Light sensing', {'light_sensing'}, Icons.light_mode_outlined),
    ('Display light', {'display', 'display_light'}, Icons.light_mode_outlined),
    (
      'Button sound',
      {'button_sound', 'sound', 'buzzer'},
      Icons.volume_up_outlined,
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
            child: Center(
              child: ReadingValue(
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
                loading: observed == null && c.readingLoading(c.component),
              ),
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
    Widget chip(Object option) => ChoiceChip(
      key: ValueKey('climate-option:' + choice.parameter + ':' + '$option'),
      label: DeviceOptionLabel(
        text: optionLabel(option),
        selected: observed == option,
      ),
      selected: observed == option,
      selectedColor: MediaQuery.highContrastOf(context)
          ? Colors.black
          : skinAccent(context),
      labelStyle: TextStyle(
        color: observed == option ? Colors.white : ink,
        fontSize: DeviceTypography.body,
        height: DeviceTypography.lineHeight,
      ),
      padding: const EdgeInsets.symmetric(vertical: 12),
      labelPadding: const EdgeInsets.symmetric(horizontal: 4),
      side: MediaQuery.highContrastOf(context)
          ? const BorderSide(color: Colors.black)
          : BorderSide.none,
      backgroundColor: skinOf(context).isSky
          ? skinOf(context).secondaryColor
          : const Color(0xffe9e7e0),
      chipAnimationStyle: ChipAnimationStyle(
        avatarDrawerAnimation: AnimationStyle.noAnimation,
      ),
      showCheckmark: false,
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
    return DeviceOptionGrid(
      key: ValueKey('climate-options-grid:${choice.parameter}'),
      labels: [for (final option in options) optionLabel(option)],
      children: [for (final option in options) chip(option)],
    );
  }

  Widget card(Widget child, {EdgeInsets padding = const EdgeInsets.all(12)}) =>
      Material(
        color: skinOf(context).cardColor,
        borderRadius: BorderRadius.circular(16),
        child: Container(
          padding: padding,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(16),
            border: Border.all(
              color: MediaQuery.highContrastOf(context)
                  ? Colors.black
                  : skinOf(context).isSky
                  ? const Color(0xffd6e8f3)
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
        ReadingValue(
          value,
          loading:
              c.observed(choice.property) == null &&
              c.readingLoading(c.component),
          size: DeviceTypography.secondary,
          color: muted,
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
          ReadingValue(
            value,
            loading:
                c.observed(choice.property) == null &&
                c.readingLoading(c.component),
            size: DeviceTypography.secondary,
            color: muted,
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
        backgroundColor: skinOf(context).isSky
            ? skinOf(context).secondaryColor
            : const Color(0xffedeae4),
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
    final number = target == null && c.readingLoading(c.component)
        ? Center(child: ReadingValue('—', loading: true, color: white))
        : FittedBox(
            fit: BoxFit.scaleDown,
            child: Text(
              target is num && target.isFinite
                  ? '${target.toString().replaceFirst(RegExp(r'\.0$'), '')}°C'
                  : '—',
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
                child: ReadingValue(
                  updating(powerChoice)
                      ? (pending!.$2 == true ? 'Turning on…' : 'Turning off…')
                      : power == true
                      ? 'Powered on'
                      : power == false
                      ? 'Powered off'
                      : '—',
                  loading: power == null && c.readingLoading(c.component),
                  size: DeviceTypography.secondary,
                  color: soft,
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
          if (c.observed('climate.current_temperature') == null &&
              c.readingLoading(c.component))
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Copy('Indoor', color: soft, size: 14),
                const SizedBox(width: 6),
                ReadingValue('—', loading: true, color: soft),
              ],
            )
          else
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
    final additional = <Widget>[
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
    ];
    const lightAndSound = {
      'light_sensing',
      'display',
      'display_light',
      'button_sound',
      'sound',
      'buzzer',
    };
    final comfort = extras
        .where((feature) => !lightAndSound.contains(feature.$3.parameter))
        .toList();
    final utilities = extras
        .where((feature) => lightAndSound.contains(feature.$3.parameter))
        .toList();
    Widget featureGroup(
      String title,
      List<(String, IconData, ClimateChoice)> features,
    ) => card(
      Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Copy(title, bold: true, size: DeviceTypography.section),
          for (var i = 0; i < features.length; i++) ...[
            if (i > 0) const Divider(height: 24),
            row(features[i].$1, features[i].$2, features[i].$3),
          ],
        ],
      ),
    );
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
        temperature(target, power),
        if (mode != null) ...[
          const SizedBox(height: 8),
          card(row('Mode', Icons.ac_unit, mode)),
        ],
        if (fan != null) ...[
          const SizedBox(height: 8),
          card(row('Fan speed', Icons.air, fan)),
        ],
        if (swingRows.isNotEmpty) ...[
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
              ],
            ),
          ),
        ],
        for (final control in additional) ...[
          const SizedBox(height: 8),
          card(control),
        ],
        if (comfort.isNotEmpty) ...[
          const SizedBox(height: 8),
          featureGroup('Comfort and energy', comfort),
        ],
        if (utilities.isNotEmpty) ...[
          const SizedBox(height: 8),
          featureGroup('Light and sound', utilities),
        ],
        if (receipt != null &&
            receipt['device_id'] == c.selectedDevice &&
            (['unknown', 'failed', 'partial'].contains(receipt['status']) ||
                (receipt['status'] == 'accepted' && c.error != null)) &&
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
        InlineDeviceInformation(controller: c, onAction: widget.onAction),
      ],
    );
  }
}
