import 'dart:convert';
import 'package:flutter/material.dart';
import 'blocks.dart';
import 'controller.dart';
import 'controls.dart';
import 'localization.dart';
import 'native_client.dart';
import 'receipts.dart';
import 'skin.dart';

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
    super.key,
  });
  final Object label, value;
  final IconData icon;
  final bool active;
  @override
  Widget build(BuildContext context) => Surface(
    radius: 20,
    padding: const EdgeInsets.all(16),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Icon(icon, color: active ? accent : muted, size: 24),
        const SizedBox(height: 12),
        Copy(label, size: 13, color: muted),
        const SizedBox(height: 6),
        Copy(value, size: 18, bold: true, color: active ? accent : ink),
      ],
    ),
  );
}

class RefrigeratorModes extends StatelessWidget {
  const RefrigeratorModes({required this.controller, super.key});
  final HomeController controller;
  @override
  Widget build(BuildContext context) {
    const id = 'vendor.midea.refrigerator';
    final cap = controller.componentCapability('fridge', id);
    if (cap == null) return const SizedBox.shrink();
    final readings =
        controller.componentSnapshot('fridge')?['observations'] as Json? ?? {};
    final entries = (cap['properties'] as Json).entries
        .where((e) => readings['$id.${e.key}']?['value'] != null)
        .toList();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const Padding(
          padding: EdgeInsets.symmetric(vertical: 16),
          child: Copy('Modes and freshness', bold: true, size: 20),
        ),
        if (entries.isEmpty)
          const Copy('No mode readings have been returned yet.', color: muted)
        else ...[
          ApplianceGrid(
            children: [
              for (final e in entries)
                ApplianceReading(
                  controlLabel('$id.${e.key}'),
                  propertyValue(
                    readings['$id.${e.key}']['value'],
                    e.value as Json,
                    propertyId: '$id.${e.key}',
                  ),
                  active: readings['$id.${e.key}']['value'] == true,
                  icon: e.value['unit'] == 'celsius'
                      ? Icons.thermostat_outlined
                      : Icons.kitchen_outlined,
                ),
            ],
          ),
          const Padding(
            padding: EdgeInsets.only(top: 12),
            child: Copy(
              'Reported modes are read-only on this connection.',
              size: 12,
              color: muted,
            ),
          ),
        ],
        const SizedBox(height: 16),
      ],
    );
  }
}

class DishwasherPanel extends StatelessWidget {
  const DishwasherPanel({
    required this.controller,
    required this.onAction,
    super.key,
  });
  final HomeController controller;
  final ActionHandler onAction;
  static const id = 'vendor.midea.dishwasher';
  @override
  Widget build(BuildContext context) {
    final c = controller, cap = c.capability(id)!;
    final properties = cap['properties'] as Json,
        actions = cap['actions'] as Json;
    final index = c.capabilities.indexOf(cap);
    Json? observation(String key) =>
        c.snapshot?['observations']?['$id.$key'] as Json?;
    Object display(String key) {
      final value = observation(key)?['value'];
      final label = key == 'mode' && value != null
          ? sourceForKey('$id.program.$value')
          : null;
      return UiMessage('{value}{stale}', {
        'value': UiMessage(
          label ??
              propertyValue(
                value,
                properties[key] as Json? ?? {},
                propertyId: '$id.$key',
              ),
        ),
        'stale': UiMessage(
          observation(key)?['stale'] == true ? ' · Stale' : '',
        ),
      });
    }

    bool allowed(String action) =>
        !c.busy &&
        c.isForeground &&
        c.hasControlAction(id, action) &&
        actions[action] != null &&
        actionUnavailable(
              cap,
              actions[action] as Json,
              trustedApproval: c.supportsTrustedApproval,
            ) ==
            null;
    void open(String action, [Json? selection]) => onAction(
      'control:$index:${Uri.encodeComponent(action)}${selection == null ? '' : ':${Uri.encodeComponent(jsonEncode(selection))}'}',
    );
    final programs =
        actions['start_cycle']?['parameters']?['properties']?['program']?['enum']
            as List? ??
        [];
    final timestamps =
        (c.snapshot?['observations'] as Json? ?? {}).values
            .whereType<Map>()
            .map((o) => o['received_at'])
            .whereType<String>()
            .toList()
          ..sort();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Container(
          padding: const EdgeInsets.all(24),
          decoration: BoxDecoration(
            color: ink,
            borderRadius: BorderRadius.circular(28),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Icon(
                Icons.local_dining_outlined,
                color: Colors.white70,
                size: 32,
              ),
              const SizedBox(height: 20),
              Copy(
                display('work_status'),
                size: 26,
                bold: true,
                color: Colors.white,
              ),
              const SizedBox(height: 8),
              Copy(display('mode'), color: Colors.white70),
              const SizedBox(height: 24),
              const Copy('Remaining time', size: 13, color: Colors.white70),
              Copy(
                display('remaining_minutes'),
                size: 32,
                bold: true,
                color: Colors.white,
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        ApplianceGrid(
          children: [
            for (final key in [
              'temperature',
              'doorswitch',
              'additional',
              'operator',
            ])
              if (properties.containsKey(key) &&
                  (key != 'operator' || observation(key) != null))
                ApplianceReading(
                  controlLabel('$id.$key'),
                  display(key),
                  icon: key == 'temperature'
                      ? Icons.thermostat_outlined
                      : Icons.tune,
                ),
          ],
        ),
        const Padding(
          padding: EdgeInsets.symmetric(vertical: 16),
          child: Copy('Wash controls', size: 20, bold: true),
        ),
        if (actions.isEmpty)
          const Copy(
            'Controls are not connected for this model yet. Available readings are shown below.',
            color: muted,
          )
        else if (!actions.keys.any((action) => c.hasControlAction(id, action)))
          const Copy(
            'No controls are allowed for this device on this connection.',
            color: muted,
          )
        else
          ApplianceGrid(
            children: [
              for (final action in actions.keys)
                if (c.hasControlAction(id, action))
                  ActionButton(
                    controlLabel('$id.$action'),
                    key: ValueKey(
                      'control:$index:${Uri.encodeComponent(action)}',
                    ),
                    style: 'secondary',
                    onPressed: allowed(action) ? () => open(action) : null,
                  ),
            ],
          ),
        if (programs.isNotEmpty && c.hasControlAction(id, 'start_cycle')) ...[
          const Padding(
            padding: EdgeInsets.symmetric(vertical: 16),
            child: Copy('Wash programs', size: 20, bold: true),
          ),
          ApplianceGrid(
            children: [
              for (final program in programs)
                ActionButton(
                  controlLabel('$id.program.$program'),
                  key: ValueKey('wash-program:$program'),
                  style: 'secondary',
                  onPressed: allowed('start_cycle')
                      ? () => open('start_cycle', {'program': program})
                      : null,
                ),
            ],
          ),
        ],
        const Padding(
          padding: EdgeInsets.symmetric(vertical: 16),
          child: Copy('Supplies and reminders', size: 20, bold: true),
        ),
        ApplianceGrid(
          children: [
            for (final key in ['water_lack', 'softwater_lack', 'bright_lack'])
              if (properties.containsKey(key))
                ApplianceReading(
                  controlLabel('$id.$key'),
                  display(key),
                  icon: Icons.water_drop_outlined,
                ),
          ],
        ),
        const Padding(
          padding: EdgeInsets.symmetric(vertical: 12),
          child: Copy(
            'Unreported readings stay unknown. Supply codes are not remaining percentages.',
            size: 12,
            color: muted,
          ),
        ),
        if (timestamps.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(bottom: 16),
            child: Copy(
              UiMessage('Latest read: {time}', {
                'time': receiptTime(timestamps.last, compact: true),
              }),
              size: 12,
              color: muted,
            ),
          ),
        for (final label in [
          'Refresh State',
          'Device information',
          'Recent Operations',
        ])
          ActionButton(
            label,
            style: 'secondary',
            onPressed: c.busy || !c.isForeground ? null : () => onAction(label),
          ),
      ],
    );
  }
}
