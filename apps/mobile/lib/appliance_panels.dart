import 'dart:convert';
import 'package:flutter/material.dart';
import 'blocks.dart';
import 'controller.dart';
import 'controls.dart';
import 'localization.dart';
import 'client_contract.dart';
import 'receipts.dart';
import 'skin.dart';
import 'device_readings.dart';
import 'device_typography.dart';

/// Local card metadata only; protected provider identities and endpoints stay
/// outside the ordinary device view.
class InlineDeviceInformation extends StatelessWidget {
  const InlineDeviceInformation({
    required this.controller,
    required this.onAction,
    super.key,
  });

  final HomeController controller;
  final ActionHandler onAction;

  @override
  Widget build(BuildContext context) {
    final device = controller.currentCard?['device'] as Json?;
    final preferences = controller.currentCard?['preferences'] as Json?;
    if (device == null) return const SizedBox.shrink();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Rows(
          [
            ['Name', LiteralText(device['alias'] ?? '—'), null],
            [
              'Room',
              preferences?['room_id'] == null
                  ? 'Unassigned'
                  : LiteralText(controller.roomName(preferences!['room_id'])),
              null,
            ],
            ['Model', LiteralText(device['model'] ?? '—'), null],
            [
              'Connection',
              switch (controller.preferences.route(controller.selectedDevice)) {
                'lan' => 'Local network',
                'cloud' => 'Provider cloud',
                _ => 'Automatic',
              },
              null,
            ],
          ],
          title: 'Device information',
          onAction: onAction,
        ),
        ActionButton(
          'Recent Operations',
          style: 'secondary',
          onPressed: controller.busy || !controller.isForeground
              ? null
              : () => onAction('Recent Operations'),
        ),
      ],
    );
  }
}

/// Presentation for explicitly named capabilities, never a brand or model guess.
class ApplianceGrid extends StatelessWidget {
  const ApplianceGrid({required this.children, super.key});
  final List<Widget> children;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      final columns =
          box.maxWidth >= 300 &&
              MediaQuery.textScalerOf(context).scale(14) <= 22
          ? 2
          : 1;
      return Wrap(
        spacing: 12,
        runSpacing: 12,
        children: [
          for (final child in children)
            SizedBox(
              width: (box.maxWidth - (columns - 1) * 12) / columns,
              child: child,
            ),
        ],
      );
    },
  );
}

class ApplianceReading extends StatelessWidget {
  const ApplianceReading(
    this.label,
    this.value, {
    this.icon = Icons.sensors_outlined,
    this.active = false,
    this.stale = false,
    super.key,
  });
  final Object label, value;
  final IconData icon;
  final bool active, stale;
  @override
  Widget build(BuildContext context) => Surface(
    radius: 20,
    padding: const EdgeInsets.all(16),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Icon(icon, color: active ? skinAccent(context) : muted, size: 24),
        const SizedBox(height: 12),
        Copy(label, size: 13, color: muted),
        const SizedBox(height: 6),
        Copy(
          value,
          size: 18,
          bold: true,
          color: active ? skinAccent(context) : ink,
        ),
        if (stale) ...[
          const SizedBox(height: 6),
          const Copy('Stale', size: 12, color: muted),
        ],
      ],
    ),
  );
}

class RefrigeratorModes extends StatelessWidget {
  const RefrigeratorModes({
    required this.controller,
    this.compact = false,
    super.key,
  });
  final bool compact;
  final HomeController controller;

  static const _base = 'vendor.midea.refrigerator';
  static const _details = '$_base.details';
  // Exact property IDs classify presentation only; descriptors and observations
  // still determine which readings exist. Unknown fields remain visible below.
  static const _groups = [
    (
      id: 'variable',
      title: 'Variable-zone details',
      icon: Icons.thermostat_outlined,
      properties: [
        '$_base.variable_mode',
        '$_base.left_flexzone_temperature',
        '$_base.left_variable_real_temperature',
        '$_base.right_flexzone_temperature',
        '$_base.right_variable_real_temperature',
      ],
    ),
    (
      id: 'smart',
      title: 'Smart and energy saving',
      icon: Icons.eco_outlined,
      properties: [
        '$_base.intelligent_mode',
        '$_base.energy_saving_mode',
        '$_details.telstar_saving_mode',
      ],
    ),
    (
      id: 'functions',
      title: 'Refrigerator functions',
      icon: Icons.kitchen_outlined,
      properties: [
        '$_details.e_cleaning_percentage',
        '$_details.electronic_smell',
        '$_details.plasma_aseptic_mode_power',
        '$_base.storage_mode',
        '$_base.freezing_mode',
        '$_details.deep_cold_mode',
        '$_base.acme_freezing_mode',
        '$_base.moisturize_mode',
        '$_base.preservation_mode',
        '$_base.icea_mode_power',
        '$_base.freezing_ice_machine_power',
        '$_details.rapid_ice_making',
        '$_base.holiday_mode',
      ],
    ),
    (
      id: 'status',
      title: 'Refrigerator status',
      icon: Icons.door_front_door_outlined,
      properties: [
        '$_details.storage_door_state',
        '$_details.freezer_door_state',
        '$_details.flexzone_door_state',
      ],
    ),
    (
      id: 'settings',
      title: 'Other settings',
      icon: Icons.settings_outlined,
      properties: ['$_details.open_door_tips_switch'],
    ),
  ];

  @override
  Widget build(BuildContext context) {
    final caps = controller
        .capabilitiesFor('fridge')
        .where(
          (cap) =>
              cap['id'] == 'vendor.midea.refrigerator' ||
              cap['id'] == 'vendor.midea.refrigerator.details',
        )
        .toList();
    if (caps.isEmpty) return const SizedBox.shrink();
    final readings =
        controller.componentSnapshot('fridge')?['observations'] as Json? ?? {};
    final entries = <String, Json>{
      for (final cap in caps)
        for (final entry in (cap['properties'] as Json).entries)
          if (entry.value['readable'] == true)
            '${cap['id']}.${entry.key}': entry.value as Json,
    };
    Widget section(
      String id,
      String title,
      IconData icon,
      Iterable<String> properties,
    ) => Column(
      key: ValueKey('refrigerator-group:$id'),
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 18, bottom: 10),
          child: Semantics(
            header: true,
            child: Copy(title, bold: true, size: 20),
          ),
        ),
        DeviceReadings(
          children: [
            for (final property in properties)
              DeviceReadingRow(
                controlLabel(property),
                propertyValue(
                  readings[property]?['value'],
                  entries[property]!,
                  propertyId: property,
                ),
                key: ValueKey('refrigerator-reading:$property'),
                loading:
                    controller.readingLoading('fridge') &&
                    readings[property]?['value'] == null,
                valueColor: readings[property]?['value'] == true
                    ? skinAccent(context)
                    : null,
                progress:
                    property == '$_details.e_cleaning_percentage' &&
                        readings[property]?['value'] is num &&
                        (readings[property]['value'] as num) >= 0 &&
                        (readings[property]['value'] as num) <= 100
                    ? (readings[property]['value'] as num).toDouble() / 100
                    : null,
              ),
          ],
        ),
      ],
    );
    final classified = _groups.expand((group) => group.properties).toSet();
    final other = entries.keys.where(
      (property) => !classified.contains(property),
    );
    if (compact && entries.isNotEmpty) {
      Iterable<String> inGroups(Set<String> ids) => _groups
          .where((g) => ids.contains(g.id))
          .expand((g) => g.properties)
          .where(entries.containsKey);
      final features = inGroups({'smart', 'functions'}).toList();
      final doors = inGroups({'status', 'settings'}).toList();
      final secondary = [
        ...inGroups({'variable'}),
        ...other,
      ];
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (features.isNotEmpty)
            section(
              'functions',
              'Function status',
              Icons.kitchen_outlined,
              features,
            ),
          if (doors.isNotEmpty)
            section(
              'status',
              'Doors and reminders',
              Icons.door_front_door_outlined,
              doors,
            ),
          if (secondary.isNotEmpty)
            section(
              'other',
              'Other readings',
              Icons.sensors_outlined,
              secondary,
            ),
          const SizedBox(height: 8),
        ],
      );
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (entries.isEmpty)
          const Padding(
            padding: EdgeInsets.symmetric(vertical: 16),
            child: Copy(
              'No mode readings have been returned yet.',
              color: muted,
            ),
          )
        else ...[
          for (final group in _groups)
            if (group.properties.any(entries.containsKey))
              section(
                group.id,
                group.title,
                group.icon,
                group.properties.where(entries.containsKey),
              ),
          if (other.isNotEmpty)
            section('other', 'Other readings', Icons.sensors_outlined, other),
        ],
        const SizedBox(height: 16),
      ],
    );
  }
}

/// Cycle controls keep reported state separate from a draft for the next wash.
/// Selections never dispatch commands: every action still opens trusted review.
class DishwasherPanel extends StatefulWidget {
  const DishwasherPanel({
    required this.controller,
    required this.onAction,
    this.componentId,
    this.showInformation = true,
    super.key,
  });
  final HomeController controller;
  final ActionHandler onAction;
  final String? componentId;
  final bool showInformation;
  static const id = 'vendor.midea.dishwasher';
  @override
  State<DishwasherPanel> createState() => _DishwasherPanelState();
}

class _DishwasherPanelState extends State<DishwasherPanel> {
  Object? _program, _additional;
  (String?, String)? _view;
  static const id = DishwasherPanel.id;
  @override
  Widget build(BuildContext context) {
    final c = widget.controller;
    final component = widget.componentId ?? c.component;
    final cap = c.componentCapability(component, id)!;
    final currentIdentity = (c.selectedDevice, component);
    if (_view != currentIdentity) {
      _view = currentIdentity;
      _program = _additional = null;
    }
    final properties = cap['properties'] as Json,
        actions = cap['actions'] as Json;
    final index = c.capabilitiesFor(component).indexOf(cap);
    final renderedDevice = c.selectedDevice, renderedSelection = c.component;
    final renderedEpoch = c.controlEpoch;
    bool sameView() =>
        c.controlEpoch == renderedEpoch &&
        c.selectedDevice == renderedDevice &&
        c.component == renderedSelection &&
        identical(c.componentCapability(component, id), cap);
    bool currentStateAllows(String action) {
      final work = c.componentSnapshot(
        component,
      )?['observations']?['$id.work_status']?['value'];
      final operation = c.componentSnapshot(
        component,
      )?['observations']?['$id.operator']?['value'];
      final isPaused = work == 'work' && operation == 'pause';
      final isRunning =
          work == 'work' &&
          (operation == null || operation == '' || operation == 'start');
      final isIdle = ['power_on', 'power_off', 'cancel'].contains(work);
      return switch (action) {
        'start_cycle' => isIdle,
        'pause_cycle' => isRunning,
        'resume_cycle' => isPaused,
        'cancel_cycle' => isRunning || isPaused,
        'power_on' => work == 'power_off',
        'power_off' => isIdle && work != 'power_off',
        _ => false,
      };
    }

    Json? observation(String key) =>
        c.componentSnapshot(component)?['observations']?['$id.$key'] as Json?;
    Object display(String key) {
      final value = observation(key)?['value'];
      if (value == null) return '—';
      if (key == 'remaining_minutes' && value is num)
        return UiMessage('{value} min', {'value': value});
      final label = sourceForKey(
        '$id.${key == 'mode' ? 'program' : key}.$value',
      );
      // Raw door and supply integers have no reviewed semantic mapping. Do not
      // guess that 0 means safe/normal or 1 means closed/needs replenishment.
      if ([
        'doorswitch',
        'water_lack',
        'softwater_lack',
        'bright_lack',
      ].contains(key))
        return label ?? 'Unknown';
      return label ??
          propertyValue(
            value,
            properties[key] as Json? ?? {},
            propertyId: '$id.$key',
          );
    }

    bool admitted(String action) =>
        actions[action] != null &&
        c.hasControlAction(id, action, componentId: component);
    bool allowed(String action) =>
        sameView() &&
        !c.busy &&
        c.isForeground &&
        admitted(action) &&
        actionUnavailable(
              cap,
              actions[action] as Json,
              trustedApproval: c.supportsTrustedApproval,
            ) ==
            null;
    void open(String action, [Json? selection]) {
      // Recheck foreground, exact view, permission and state at tap time.
      if (!allowed(action) || !currentStateAllows(action)) return;
      final suffix = selection == null
          ? ''
          : ':${Uri.encodeComponent(jsonEncode(selection))}';
      widget.onAction(
        component == c.component
            ? 'control:$index:${Uri.encodeComponent(action)}$suffix'
            : 'control-component:${Uri.encodeComponent(component)}:${Uri.encodeComponent(id)}:${Uri.encodeComponent(action)}$suffix',
      );
    }

    final status = observation('work_status')?['value'];
    final operator = observation('operator')?['value'];
    final paused = status == 'work' && operator == 'pause';
    final running =
        status == 'work' &&
        (operator == null || operator == '' || operator == 'start');
    final idle = ['power_on', 'power_off', 'cancel'].contains(status);
    final known = idle || running || paused;
    final schema = actions['start_cycle']?['parameters'] as Json? ?? {};
    final programSchema = schema['properties']?['program'] as Json? ?? {};
    final additionalSchema = schema['properties']?['additional'] as Json?;
    final programs = programSchema['enum'] as List? ?? [];
    final additional = additionalSchema?['enum'] as List? ?? [];
    if (!programs.contains(_program)) _program = null;
    if (!additional.contains(_additional)) _additional = null;
    final draft = <String, dynamic>{
      if (_program != null) 'program': _program,
      if (_additional != null) 'additional': _additional,
    };
    final ready =
        _program != null &&
        (schema['required'] as List? ?? []).every(draft.containsKey) &&
        draft.entries.every(
          (e) =>
              validateValue(e.value, schema['properties'][e.key] as Json) ==
              null,
        );
    Widget heading(String label) => Padding(
      padding: const EdgeInsets.only(bottom: 6),
      child: Semantics(
        header: true,
        child: Copy(
          label,
          size: DeviceTypography.section,
          lineHeight: DeviceTypography.lineHeight,
          bold: true,
        ),
      ),
    );
    Widget reading(String key, {bool icon = false}) => Padding(
      padding: const EdgeInsets.symmetric(vertical: 10),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (icon) ...[
            Icon(
              switch (key) {
                'temperature' => Icons.thermostat_outlined,
                'doorswitch' => Icons.door_front_door_outlined,
                _ => Icons.tune,
              },
              size: 20,
              color: muted,
            ),
            const SizedBox(width: 10),
          ],
          Expanded(
            child: Copy(
              controlLabel('$id.$key'),
              size: DeviceTypography.body,
              lineHeight: DeviceTypography.lineHeight,
              color: muted,
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Align(
              alignment: Alignment.centerRight,
              child: ReadingValue(
                display(key),
                size: DeviceTypography.body,
                bold: true,
                loading:
                    observation(key) == null && c.readingLoading(component),
              ),
            ),
          ),
        ],
      ),
    );
    Widget action(
      String name, {
      bool secondary = false,
      Json? selection,
      Object? label,
      bool enabled = true,
    }) => Padding(
      padding: const EdgeInsets.only(top: 10),
      child: _CycleActionButton(
        localize(context, label ?? controlLabel('$id.$name')),
        key: ValueKey('control:$index:$name'),
        style: secondary ? 'secondary' : '',
        onPressed: allowed(name) && enabled
            ? () => open(name, selection)
            : null,
      ),
    );
    Widget card(List<Widget> children, {bool subdued = false}) => Padding(
      padding: const EdgeInsets.only(top: 12),
      child: Material(
        color: subdued
            ? skinOf(context).secondaryColor
            : skinOf(context).cardColor,
        borderRadius: BorderRadius.circular(20),
        clipBehavior: Clip.antiAlias,
        child: Padding(
          padding: const EdgeInsets.all(14),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: children,
          ),
        ),
      ),
    );
    Widget choices(String parameter, List options, Object? selected) {
      Widget tile(Object value) => RadioListTile<Object>(
        key: ValueKey(
          parameter == 'program'
              ? 'wash-program:$value'
              : 'wash-additional:$value',
        ),
        value: value,
        enabled: allowed('start_cycle') && idle,
        activeColor: skinAccent(context),
        selected: selected == value,
        selectedTileColor: skinAccent(context).withValues(alpha: 0.08),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
        contentPadding: EdgeInsets.zero,
        dense: true,
        title: Copy(
          parameter == 'program'
              ? controlLabel('$id.program.$value')
              : propertyValue(
                  value,
                  additionalSchema!,
                  propertyId: '$id.additional',
                ),
          size: DeviceTypography.option,
          lineHeight: DeviceTypography.lineHeight,
          color: selected == value ? skinAccent(context) : ink,
          bold: selected == value,
        ),
      );
      return RadioGroup<Object>(
        groupValue: selected,
        onChanged: (value) {
          if (!allowed('start_cycle') || !currentStateAllows('start_cycle'))
            return;
          setState(() {
            if (parameter == 'program') {
              _program = value;
            } else {
              _additional = value;
            }
          });
        },
        child: Column(children: [for (final value in options) tile(value)]),
      );
    }

    final receipt = c.receipt;
    final mainReadings = [
      'temperature',
      'doorswitch',
      'additional',
    ].where(properties.containsKey).toList();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (running || paused)
          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              color: ink,
              borderRadius: BorderRadius.circular(20),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Copy(
                  paused ? 'Paused' : display('work_status'),
                  size: 24,
                  lineHeight: DeviceTypography.lineHeight,
                  bold: true,
                  color: Colors.white,
                ),
                const SizedBox(height: 6),
                Copy(
                  UiMessage('Current program: {program}', {
                    'program': localize(context, display('mode')),
                  }),
                  size: DeviceTypography.body,
                  lineHeight: DeviceTypography.lineHeight,
                  color: Colors.white70,
                ),
                const SizedBox(height: 12),
                Wrap(
                  spacing: 12,
                  crossAxisAlignment: WrapCrossAlignment.center,
                  children: [
                    const Copy(
                      'Remaining time',
                      size: DeviceTypography.secondary,
                      color: Colors.white70,
                    ),
                    Copy(
                      display('remaining_minutes'),
                      size: 36,
                      lineHeight: DeviceTypography.lineHeight,
                      bold: true,
                      color: Colors.white,
                    ),
                  ],
                ),
              ],
            ),
          )
        else
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8),
            child: Row(
              children: [
                Expanded(
                  child: Copy(
                    'Reported cycle state',
                    color:
                        skinOf(context).isSky &&
                            !MediaQuery.highContrastOf(context)
                        ? Colors.white
                        : muted,
                    size: DeviceTypography.body,
                    lineHeight: DeviceTypography.lineHeight,
                  ),
                ),
                const SizedBox(width: 12),
                Flexible(
                  child: ReadingValue(
                    status == null
                        ? '—'
                        : known
                        ? display('work_status')
                        : 'Unknown',
                    loading: status == null && c.readingLoading(component),
                    color:
                        skinOf(context).isSky &&
                            !MediaQuery.highContrastOf(context)
                        ? Colors.white
                        : ink,
                    size: DeviceTypography.body,
                    bold: true,
                  ),
                ),
              ],
            ),
          ),
        if (running || paused)
          card([
            for (var i = 0; i < mainReadings.length; i++) ...[
              if (i > 0)
                const Divider(
                  height: 1,
                  thickness: 1,
                  color: Color(0xffefefec),
                ),
              reading(mainReadings[i], icon: true),
            ],
          ]),
        if (actions.isEmpty)
          const Copy(
            'Controls are not connected for this model yet. Available readings are shown below.',
            color: muted,
          )
        else if (!actions.keys.any(admitted))
          const Copy(
            'No controls are allowed for this device on this connection.',
            color: muted,
          )
        else if (!known && !c.readingLoading(component))
          const Copy('Control unavailable.', color: muted),
        if (actions['start_cycle'] != null && programs.isNotEmpty) ...[
          card([
            heading('Next wash program (choose one)'),
            choices('program', programs, _program),
          ]),
          if (additional.isNotEmpty)
            card([
              heading('Additional option (choose one)'),
              const Copy(
                'This device accepts one additional option, not a combination.',
                size: DeviceTypography.secondary,
                color: muted,
              ),
              choices('additional', additional, _additional),
            ]),
          if (idle && admitted('start_cycle')) ...[
            const SizedBox(height: 10),
            const Copy(
              'Selecting options does not start the appliance.',
              size: DeviceTypography.secondary,
              color: muted,
            ),
            action(
              'start_cycle',
              enabled: ready,
              selection: draft,
              label: _program == null
                  ? 'Select a wash program'
                  : UiMessage('Start {program}', {
                      'program': UiMessage(
                        controlLabel('$id.program.$_program'),
                      ),
                    }),
            ),
          ],
        ],
        if (running && admitted('pause_cycle')) action('pause_cycle'),
        if (paused && admitted('resume_cycle')) action('resume_cycle'),
        if ((running || paused) && admitted('cancel_cycle')) ...[
          action('cancel_cycle', secondary: true),
          const Padding(
            padding: EdgeInsets.only(top: 6),
            child: Copy(
              'Review and confirmation are required. Cancellation is not a physical rollback.',
              size: DeviceTypography.secondary,
              color: muted,
            ),
          ),
        ],
        if (running || paused) ...[
          const SizedBox(height: 16),
          card([
            const Row(
              children: [
                Icon(Icons.lock_outline, color: muted),
                SizedBox(width: 8),
                Expanded(
                  child: Copy(
                    'Program and options are locked during a cycle.',
                    color: muted,
                  ),
                ),
              ],
            ),
          ], subdued: true),
        ],
        if (idle && admitted(status == 'power_off' ? 'power_on' : 'power_off'))
          action(
            status == 'power_off' ? 'power_on' : 'power_off',
            secondary: true,
          ),
        if (receipt != null &&
            receipt['device_id'] == c.selectedDevice &&
            (['unknown', 'failed', 'partial'].contains(receipt['status']) ||
                (receipt['status'] == 'accepted' && c.error != null)))
          Padding(
            padding: const EdgeInsets.only(top: 12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Copy(outcomeSummary(receipt['status']), color: muted),
                TextButton(
                  onPressed: c.busy
                      ? null
                      : () => widget.onAction('View Details'),
                  child: const Copy('View Details'),
                ),
              ],
            ),
          ),
        Column(
          key: const ValueKey('cycle-readings'),
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            heading('Supplies and reminders'),
            for (final key in [
              if (!running && !paused) ...[
                'mode',
                'temperature',
                'doorswitch',
                'additional',
                'remaining_minutes',
              ],
              'operator',
              'water_lack',
              'softwater_lack',
              'bright_lack',
            ])
              if (properties.containsKey(key)) reading(key),
            for (final entry in properties.entries)
              if (entry.value['readable'] == true &&
                  !{
                    'work_status',
                    'mode',
                    'temperature',
                    'doorswitch',
                    'additional',
                    'remaining_minutes',
                    'operator',
                    'water_lack',
                    'softwater_lack',
                    'bright_lack',
                  }.contains(entry.key))
                reading(entry.key),
          ],
        ),
        if (widget.showInformation)
          InlineDeviceInformation(controller: c, onAction: widget.onAction),
      ],
    );
  }
}

/// Local cycle actions follow the approved contrast hierarchy without changing
/// the shared skin used by unrelated screens.
class _CycleActionButton extends ActionButton {
  const _CycleActionButton(
    super.label, {
    required super.onPressed,
    super.style,
    super.key,
  });
  @override
  Widget build(BuildContext context) {
    final highContrast = MediaQuery.highContrastOf(context);
    final color = highContrast ? Colors.black : skinAccent(context);
    final shape = RoundedRectangleBorder(
      borderRadius: BorderRadius.circular(14),
    );
    final text = Text(
      localize(context, label),
      textAlign: TextAlign.center,
      style: TextStyle(
        fontFamily: 'Inter',
        fontFamilyFallback: windowFontFallback,
        fontSize: style == 'secondary'
            ? DeviceTypography.option
            : DeviceTypography.primaryAction,
        height: DeviceTypography.lineHeight,
        fontWeight: FontWeight.w600,
      ),
    );
    final buttonStyle = FilledButton.styleFrom(
      minimumSize: const Size(double.infinity, 52),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
      backgroundColor: color,
      foregroundColor: Colors.white,
      shape: shape,
    );
    return style == 'secondary'
        ? OutlinedButton(
            onPressed: onPressed,
            style: OutlinedButton.styleFrom(
              minimumSize: const Size(double.infinity, 48),
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              foregroundColor: color,
              side: BorderSide(color: onPressed == null ? muted : color),
              shape: shape,
            ),
            child: text,
          )
        : FilledButton(onPressed: onPressed, style: buttonStyle, child: text);
  }
}
