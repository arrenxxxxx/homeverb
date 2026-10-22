import 'localization.dart';
import 'package:flutter/material.dart';
import 'blocks.dart';
import 'controller.dart';
import 'controls.dart';
import 'appliance_panels.dart';
import 'native_client.dart';
import 'receipts.dart';
import 'skin.dart';

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
    super.key,
  });
  final HomeController controller;
  final ActionHandler onAction;
  final void Function(String component, num target) onTarget;

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      const Padding(
        padding: EdgeInsets.only(bottom: 16),
        child: Copy('Temperature zones', size: 22, bold: true),
      ),
      for (final part in controller.components) compartment(context, part),
      RefrigeratorModes(controller: controller),
      ActionButton(
        'Refresh State',
        style: 'secondary',
        onPressed: controller.busy ? null : () => onAction('Refresh State'),
      ),
      for (final label in ['Device information', 'Recent Operations'])
        ActionButton(
          label,
          style: 'secondary',
          onPressed: controller.busy ? null : () => onAction(label),
        ),
    ],
  );

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
        ]);
        if (capability['id'] != 'climate' && capability['id'] != 'power') {
          extraRows.add(rows.last);
        }
      }
    }
    final extraActions = controller
        .capabilitiesFor(id)
        .any(
          (capability) => (capability['actions'] as Json).entries.any(
            (entry) =>
                controller.hasControlAction(
                  capability['id'] as String,
                  entry.key,
                  componentId: id,
                ) &&
                actionUnavailable(
                      capability,
                      entry.value as Json,
                      trustedApproval: controller.supportsTrustedApproval,
                    ) ==
                    null &&
                (capability['id'] != 'climate' ||
                    entry.key != 'set' ||
                    (entry.value['parameters']['properties'] as Map).keys.any(
                      (name) => name != 'target_temperature',
                    )),
          ),
        );
    final readError = controller.componentReadError(id);
    final observations = (snapshot?['observations'] as Json? ?? {}).values
        .whereType<Map>();
    final stale = observations.any((o) => o['stale'] == true);
    final times =
        observations.map((o) => o['received_at']).whereType<String>().toList()
          ..sort((a, b) => DateTime.parse(a).compareTo(DateTime.parse(b)));
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
                      color: accent,
                    ),
                    const SizedBox(width: 10),
                    Expanded(child: Copy(label, bold: true, size: 18)),
                    if (controller.componentCapability(id, 'power') != null)
                      Copy(
                        propertyValue(observation('power.state')?['value'], {}),
                        size: 13,
                        color: muted,
                      ),
                  ],
                ),
                const SizedBox(height: 18),
                const Copy('Target', size: 13, color: muted),
                Copy(
                  propertyValue(target, targetProperty ?? {}),
                  size: 42,
                  bold: true,
                ),
                const SizedBox(height: 8),
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
        if (extraRows.isNotEmpty) Rows(extraRows, onAction: onAction),
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
        if (extraActions)
          ActionButton(
            'More controls',
            key: ValueKey('open-component:$id'),
            style: 'secondary',
            onPressed: controller.busy
                ? null
                : () => onAction('open-component:${Uri.encodeComponent(id)}'),
          ),
        if (readError != null)
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Copy(
              UiMessage('{component} could not be read: {error}', {
                'component': UiMessage(label),
                'error': localizedError(context, readError),
              }),
              color: const Color(0xff9e3421),
            ),
          ),
        Padding(
          padding: const EdgeInsets.only(left: 2, bottom: 12),
          child: Copy(
            times.isEmpty
                ? UiMessage('{component} · No saved reading', {
                    'component': UiMessage(label),
                  })
                : UiMessage('{component} · {source} {time}{stale}', {
                    'component': UiMessage(label),
                    'source': UiMessage(
                      snapshot?['from_cache'] == true ? 'Cached' : 'Read',
                    ),
                    'time': receiptTime(times.last, compact: true),
                    'stale': UiMessage(stale ? ' · Stale' : ''),
                  }),
            size: 12,
            color: muted,
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
