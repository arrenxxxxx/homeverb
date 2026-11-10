import 'localization.dart';
import 'package:flutter/material.dart';
import 'blocks.dart';
import 'controller.dart';
import 'controls.dart';
import 'appliance_panels.dart';
import 'client_contract.dart';
import 'receipts.dart';
import 'skin.dart';
import 'device_readings.dart';
import 'device_typography.dart';

bool hasDirectTarget(Json action) {
  final parameters = action['parameters'] as Json;
  return action['idempotency'] == 'set_value' &&
      (parameters['required'] as List? ?? []).every(
        (name) => name == 'target_temperature',
      ) &&
      (parameters['minProperties'] as num? ?? 0) <= 1 &&
      parameters.keys.every(
        (key) => const {
          'type',
          'properties',
          'required',
          'minProperties',
          'maxProperties',
          'additionalProperties',
          'description',
          'title',
        }.contains(key),
      );
}

class CompartmentControls extends StatelessWidget {
  const CompartmentControls({
    required this.controller,
    required this.onAction,
    required this.onTarget,
    this.onComponentAction,
    this.pendingComponent,
    this.outcomeComponent,
    super.key,
  });
  final HomeController controller;
  final ActionHandler onAction;
  final void Function(String component, num target) onTarget;
  final void Function(
    String component,
    String capability,
    String action,
    Json arguments,
  )?
  onComponentAction;
  final String? pendingComponent, outcomeComponent;

  bool get _showReceipt {
    final receipt = controller.receipt;
    return pendingComponent == null &&
        !controller.isCommandError &&
        receipt != null &&
        receipt['device_id'] == controller.selectedDevice &&
        (['unknown', 'failed', 'partial'].contains(receipt['status']) ||
            (receipt['status'] == 'accepted' && controller.error != null));
  }

  Widget _receiptFeedback() => Padding(
    padding: const EdgeInsets.symmetric(vertical: 8),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Copy(outcomeSummary(controller.receipt!['status']), color: muted),
        TextButton(
          onPressed: controller.busy ? null : () => onAction('View Details'),
          child: const Copy('View Details'),
        ),
      ],
    ),
  );

  @override
  Widget build(BuildContext context) => _readOnly
      ? _readOnlyOverview(context)
      : Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Padding(
              padding: EdgeInsets.only(bottom: 16),
              child: Copy('Temperature zones', size: 22, bold: true),
            ),
            for (final part in controller.components)
              compartment(context, part),
            RefrigeratorModes(controller: controller),
            InlineDeviceInformation(controller: controller, onAction: onAction),
          ],
        );

  bool get _readOnly => !controller.components.any(
    (part) => controller
        .capabilitiesFor(part['id'] as String)
        .any(
          (capability) => (capability['actions'] as Json).entries.any(
            (entry) =>
                controller.hasControlAction(
                  capability['id'] as String,
                  entry.key,
                  componentId: part['id'] as String,
                ) &&
                actionUnavailable(
                      capability,
                      entry.value as Json,
                      trustedApproval: controller.supportsTrustedApproval,
                    ) ==
                    null,
          ),
        ),
  );

  Widget _readOnlyOverview(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      Container(
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
        decoration: BoxDecoration(
          color: const Color(0xfffff0ce),
          borderRadius: BorderRadius.circular(10),
        ),
        child: const Row(
          children: [
            Icon(Icons.lock_outline, size: 20),
            SizedBox(width: 10),
            Expanded(
              child: Copy(
                'View only. Controls are unavailable on this connection.',
                size: DeviceTypography.body,
                lineHeight: DeviceTypography.lineHeight,
              ),
            ),
          ],
        ),
      ),
      const Padding(
        padding: EdgeInsets.only(top: 18, bottom: 10),
        child: Copy('Temperature zones', size: 20, bold: true),
      ),
      LayoutBuilder(
        builder: (context, constraints) {
          final expandedText = MediaQuery.textScalerOf(context).scale(14) > 21;
          return DeviceReadings(
            children: expandedText || constraints.maxWidth < 280
                ? [
                    for (final part in controller.components)
                      _readOnlyZone(context, part),
                  ]
                : [
                    const Padding(
                      padding: EdgeInsets.symmetric(vertical: 10),
                      child: Row(
                        children: [
                          Expanded(
                            child: Copy(
                              'Zone',
                              size: DeviceTypography.secondary,
                              lineHeight: DeviceTypography.lineHeight,
                              color: muted,
                            ),
                          ),
                          Expanded(
                            child: Copy(
                              'Set temperature',
                              size: DeviceTypography.secondary,
                              lineHeight: DeviceTypography.lineHeight,
                              color: muted,
                              center: true,
                            ),
                          ),
                          Expanded(
                            child: Copy(
                              'Current temperature',
                              size: DeviceTypography.secondary,
                              lineHeight: DeviceTypography.lineHeight,
                              color: muted,
                              center: true,
                            ),
                          ),
                        ],
                      ),
                    ),
                    for (final part in controller.components)
                      _temperatureTableRow(context, part),
                  ],
          );
        },
      ),
      RefrigeratorModes(controller: controller, compact: true),
      if (_showReceipt) _receiptFeedback(),
      InlineDeviceInformation(controller: controller, onAction: onAction),
    ],
  );

  Widget _temperatureTableRow(BuildContext context, Json part) {
    final id = part['id'] as String;
    final climate = controller.componentCapability(id, 'climate');
    final observations =
        controller.componentSnapshot(id)?['observations'] as Json? ?? {};
    Object value(String property) {
      final observed = observations['climate.$property']?['value'];
      return UiMessage(
        observed == null
            ? '—'
            : propertyValue(
                observed,
                climate?['properties']?[property] as Json? ?? {},
              ),
      );
    }

    return Padding(
      key: ValueKey('readonly-zone:$id'),
      padding: const EdgeInsets.symmetric(vertical: 10),
      child: Column(
        children: [
          Row(
            children: [
              Expanded(
                child: Copy(
                  controlLabel(id),
                  size: DeviceTypography.reading,
                  lineHeight: DeviceTypography.lineHeight,
                  bold: true,
                ),
              ),
              Expanded(
                child: Center(
                  child: ReadingValue(
                    value('target_temperature'),
                    size: 24,
                    bold: true,
                    center: true,
                    loading:
                        controller.readingLoading(id) &&
                        observations['climate.target_temperature']?['value'] ==
                            null,
                  ),
                ),
              ),
              Expanded(
                child: Center(
                  child: ReadingValue(
                    value('current_temperature'),
                    size: DeviceTypography.reading,
                    color: muted,
                    center: true,
                    loading:
                        controller.readingLoading(id) &&
                        observations['climate.current_temperature']?['value'] ==
                            null,
                  ),
                ),
              ),
            ],
          ),
          for (final cap in controller.capabilitiesFor(id))
            if (cap['id'] != 'climate' &&
                cap['id'] != 'vendor.midea.refrigerator' &&
                cap['id'] != 'vendor.midea.refrigerator.details')
              for (final entry in (cap['properties'] as Json).entries)
                if (entry.value['readable'] == true)
                  DeviceReadingRow(
                    controlLabel(entry.key),
                    UiMessage(
                      propertyValue(
                        observations['${cap['id']}.${entry.key}']?['value'],
                        entry.value as Json,
                        propertyId: '${cap['id']}.${entry.key}',
                      ),
                    ),
                    loading:
                        controller.readingLoading(id) &&
                        observations['${cap['id']}.${entry.key}']?['value'] ==
                            null,
                  ),
        ],
      ),
    );
  }

  Widget _readOnlyZone(BuildContext context, Json part) {
    final id = part['id'] as String;
    final climate = controller.componentCapability(id, 'climate');
    final snapshot = controller.componentSnapshot(id);
    final observations = snapshot?['observations'] as Json? ?? {};
    Object value(String property) {
      final observed = observations['climate.$property']?['value'];
      return UiMessage(
        observed == null
            ? '—'
            : propertyValue(
                observed,
                climate?['properties']?[property] as Json? ?? {},
              ),
      );
    }

    return Padding(
      key: ValueKey('readonly-zone:$id'),
      padding: const EdgeInsets.symmetric(vertical: 14),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Copy(controlLabel(id), bold: true, size: 18),
          DeviceReadingRow(
            'Set temperature',
            value('target_temperature'),
            loading:
                controller.readingLoading(id) &&
                observations['climate.target_temperature']?['value'] == null,
          ),
          DeviceReadingRow(
            'Current temperature',
            value('current_temperature'),
            loading:
                controller.readingLoading(id) &&
                observations['climate.current_temperature']?['value'] == null,
          ),
          for (final cap in controller.capabilitiesFor(id))
            if (cap['id'] != 'climate' &&
                cap['id'] != 'vendor.midea.refrigerator' &&
                cap['id'] != 'vendor.midea.refrigerator.details')
              for (final entry in (cap['properties'] as Json).entries)
                if (entry.value['readable'] == true)
                  DeviceReadingRow(
                    controlLabel(entry.key),
                    UiMessage(
                      propertyValue(
                        observations['${cap['id']}.${entry.key}']?['value'],
                        entry.value as Json,
                        propertyId: '${cap['id']}.${entry.key}',
                      ),
                    ),
                    loading:
                        controller.readingLoading(id) &&
                        observations['${cap['id']}.${entry.key}']?['value'] ==
                            null,
                  ),
        ],
      ),
    );
  }

  Widget compartment(BuildContext context, Json part) {
    final id = part['id'] as String, label = controlLabel(id);
    final climate = controller.componentCapability(id, 'climate');
    final snapshot = controller.componentSnapshot(id);
    Json? observation(String property) =>
        snapshot?['observations']?[property] as Json?;
    final targetProperty =
        climate?['properties']?['target_temperature'] as Json?;
    final target = observation('climate.target_temperature')?['value'];
    final action = climate?['actions']?['set'] as Json?;
    final parameters = action?['parameters'] as Json?;
    final schema = parameters?['properties']?['target_temperature'] as Json?;
    final editable =
        schema != null &&
        targetProperty?['writable'] == true &&
        controller.hasControlAction('climate', 'set', componentId: id) &&
        actionUnavailable(climate!, action!) == null;
    final direct = editable && hasDirectTarget(action);
    final standalone =
        direct &&
        (parameters?['required'] as List? ?? []).isEmpty &&
        (parameters?['minProperties'] as num? ?? 0) <= 1;
    final inlineOptions = <String, Json>{};
    if (standalone) {
      for (final entry in (parameters?['properties'] as Json? ?? {}).entries) {
        final property = climate?['properties']?[entry.key] as Json?;
        if (entry.key != 'target_temperature' &&
            entry.value is Json &&
            (entry.value as Json)['enum'] is List &&
            property?['writable'] == true &&
            (property?['control_requires_observation'] != true ||
                observation('climate.${entry.key}') != null)) {
          inlineOptions[entry.key] = entry.value as Json;
        }
      }
    }
    final step =
        schema?['multipleOf'] as num? ??
        (schema?['type'] == 'integer' ? 1 : null);
    final rows = <List<dynamic>>[];
    final extraRows = <List<dynamic>>[];
    final trailing = <int, Widget>{};
    for (final capability in controller.capabilitiesFor(id)) {
      if (capability['id'] == 'vendor.midea.refrigerator' ||
          capability['id'] == 'vendor.midea.refrigerator.details')
        continue;
      for (final entry in (capability['properties'] as Json).entries) {
        if (entry.value['readable'] != true) continue;
        final propertyId = '${capability['id']}.${entry.key}',
            observed = observation(propertyId);
        final isTarget =
            capability['id'] == 'climate' && entry.key == 'target_temperature';
        if (isTarget && direct && step != null) {
          bool canStep(int direction) =>
              !controller.busy &&
              target is num &&
              validateValue(target, schema) == null &&
              validateValue(target + direction * step, schema) == null &&
              controller.canControlValue(
                'climate',
                'set',
                'target_temperature',
                target + direction * step,
                componentId: id,
              );
          trailing[rows.length] = InlineStepper(
            key: ValueKey('component-stepper:$id'),
            label: localize(
              context,
              UiMessage('{component} target', {'component': UiMessage(label)}),
            ),
            value: propertyValue(target, entry.value as Json),
            onIncrease: canStep(1)
                ? () => onTarget(id, (target as num) + step)
                : null,
            onDecrease: canStep(-1)
                ? () => onTarget(id, (target as num) - step)
                : null,
          );
        }
        rows.add([
          capability['id'] == 'climate' && entry.key == 'current_temperature'
              ? 'Current'
              : isTarget
              ? 'Target'
              : controlLabel(entry.key),
          propertyValue(observed?['value'], entry.value as Json),
          null,
          if (observed == null && controller.readingLoading(id))
            const ReadingValue('—', loading: true),
        ]);
        if ((capability['id'] != 'climate' && capability['id'] != 'power') ||
            (capability['id'] == 'climate' &&
                ![
                  'target_temperature',
                  'current_temperature',
                ].contains(entry.key))) {
          extraRows.add(rows.last);
        }
      }
    }
    final extraActions = <(Json, String)>[];
    for (final capability in controller.capabilitiesFor(id)) {
      for (final entry in (capability['actions'] as Json).entries) {
        final action = entry.value as Json;
        if (capability['id'] == 'climate' &&
            entry.key == 'set' &&
            editable &&
            ((action['parameters']?['properties'] as Map?) ?? {}).keys.every(
              (name) =>
                  name == 'target_temperature' ||
                  inlineOptions.containsKey(name),
            ))
          continue;
        if (!controller.hasControlAction(
              capability['id'] as String,
              entry.key,
              componentId: id,
            ) ||
            actionUnavailable(
                  capability,
                  action,
                  trustedApproval: controller.supportsTrustedApproval,
                ) !=
                null ||
            ((action['parameters']?['properties'] as Map?) ?? {}).keys.any(
              (name) =>
                  capability['properties']?[name]?['control_requires_observation'] ==
                      true &&
                  observation('${capability['id']}.$name') == null,
            ))
          continue;
        extraActions.add((capability, entry.key));
      }
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.only(bottom: 12),
          child: Surface(
            radius: 24,
            padding: const EdgeInsets.all(20),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Icon(
                      id == 'freezer' ? Icons.ac_unit : Icons.kitchen_outlined,
                      color: skinAccent(context),
                    ),
                    const SizedBox(width: 10),
                    Expanded(child: Copy(label, bold: true, size: 18)),
                    if (pendingComponent == id)
                      OperationPulse(
                        key: ValueKey('component-pending:$id'),
                        size: 20,
                      ),
                    if (controller.componentCapability(id, 'power') != null)
                      ReadingValue(
                        propertyValue(observation('power.state')?['value'], {}),
                        size: 13,
                        color: muted,
                        loading:
                            observation('power.state') == null &&
                            controller.readingLoading(id),
                      ),
                  ],
                ),
                const SizedBox(height: 18),
                const Copy('Target', size: 13, color: muted),
                ReadingValue(
                  propertyValue(target, targetProperty ?? {}),
                  size: 42,
                  bold: true,
                  loading: target == null && controller.readingLoading(id),
                ),
                const SizedBox(height: 8),
                if (observation('climate.current_temperature') == null &&
                    controller.readingLoading(id))
                  Row(
                    children: [
                      const Copy('Current', color: muted),
                      const SizedBox(width: 6),
                      const ReadingValue('—', loading: true),
                    ],
                  )
                else
                  Copy(
                    UiMessage('Current: {value}', {
                      'value': UiMessage(
                        propertyValue(
                          observation('climate.current_temperature')?['value'],
                          climate?['properties']?['current_temperature']
                                  as Json? ??
                              {},
                        ),
                      ),
                    }),
                    color: muted,
                  ),
                for (final stepper in trailing.values)
                  Padding(
                    padding: const EdgeInsets.only(top: 12),
                    child: stepper,
                  ),
              ],
            ),
          ),
        ),
        if (_showReceipt && (outcomeComponent ?? controller.component) == id)
          _receiptFeedback(),
        if (extraRows.isNotEmpty) Rows(extraRows, onAction: onAction),
        for (final entry in inlineOptions.entries) ...[
          Copy(controlLabel(entry.key), bold: true),
          Wrap(
            spacing: 8,
            children: [
              for (final value in entry.value['enum'] as List)
                ChoiceChip(
                  key: ValueKey('component-option:$id:${entry.key}:$value'),
                  label: Text(localize(context, controlLabel('$value'))),
                  selected:
                      observation('climate.${entry.key}')?['value'] == value,
                  onSelected:
                      controller.busy ||
                          !controller.isForeground ||
                          pendingComponent != null ||
                          onComponentAction == null ||
                          observation('climate.${entry.key}')?['value'] ==
                              value ||
                          !controller.canControlValue(
                            'climate',
                            'set',
                            entry.key,
                            value,
                            componentId: id,
                          )
                      ? null
                      : (_) => onComponentAction!(id, 'climate', 'set', {
                          entry.key: value,
                        }),
                ),
            ],
          ),
        ],
        if (editable)
          ActionButton(
            localize(
              context,
              UiMessage('{component} target', {'component': UiMessage(label)}),
            ),
            key: ValueKey('edit-component:$id'),
            style: 'secondary',
            onPressed: controller.busy
                ? null
                : () => onAction('edit-component:${Uri.encodeComponent(id)}'),
          ),
        for (final entry in extraActions)
          ActionButton(
            sourceForKey('${entry.$1['id']}.${entry.$2}') ??
                controlLabel(entry.$2),
            key: ValueKey(
              'component-control:$id:${entry.$1['id']}:${entry.$2}',
            ),
            style: 'secondary',
            onPressed: controller.busy || !controller.isForeground
                ? null
                : () => onAction(
                    'control-component:${Uri.encodeComponent(id)}:${Uri.encodeComponent(entry.$1['id'] as String)}:${Uri.encodeComponent(entry.$2)}',
                  ),
          ),
        if (targetProperty != null && !editable)
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Copy(
              targetProperty['writable'] != true
                  ? UiMessage('{component} target is read-only.', {
                      'component': UiMessage(label),
                    })
                  : UiMessage('{label}: {message}', {
                      'label': UiMessage(label),
                      'message': UiMessage(
                        action == null
                            ? 'Target controls are unavailable.'
                            : actionUnavailable(climate!, action) ??
                                  'Target controls are unavailable.',
                      ),
                    }),
              size: 12,
              color: muted,
            ),
          ),
      ],
    );
  }
}
