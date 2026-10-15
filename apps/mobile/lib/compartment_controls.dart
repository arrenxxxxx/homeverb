import 'package:flutter/material.dart';
import 'blocks.dart';
import 'controller.dart';
import 'controls.dart';
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
      for (final part in controller.components) compartment(part),
      const Padding(
        padding: EdgeInsets.fromLTRB(2, 0, 2, 12),
        child: Copy(
          'One appliance, separate components. Each uses its own supported limits.',
          color: muted,
        ),
      ),
      ActionButton(
        'Refresh State',
        style: 'secondary',
        onPressed: controller.busy ? null : () => onAction('Refresh State'),
      ),
    ],
  );

  Widget compartment(Json part) {
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
        actionUnavailable(climate!, action!) == null;
    final direct = editable && hasDirectTarget(action);
    final step =
        schema?['multipleOf'] as num? ??
        (schema?['type'] == 'integer' ? 1 : null);
    final rows = <List<dynamic>>[];
    final trailing = <int, Widget>{};
    for (final capability in controller.capabilitiesFor(id)) {
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
              validateValue(target + direction * step, schema) == null;
          trailing[rows.length] = InlineStepper(
            key: ValueKey('component-stepper:$id'),
            label: '$label target',
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
          isTarget && editable && !controller.busy
              ? 'edit-component:${Uri.encodeComponent(id)}'
              : null,
        ]);
      }
    }
    final extraActions = controller
        .capabilitiesFor(id)
        .any(
          (capability) => (capability['actions'] as Json).entries.any(
            (entry) =>
                capability['id'] != 'climate' ||
                entry.key != 'set' ||
                (entry.value['parameters']['properties'] as Map).keys.any(
                  (name) => name != 'target_temperature',
                ),
          ),
        );
    if (extraActions)
      rows.add([
        'More controls',
        '›',
        controller.busy ? null : 'open-component:${Uri.encodeComponent(id)}',
      ]);
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
        Rows(rows, title: label, trailing: trailing, onAction: onAction),
        if (readError != null)
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Copy(
              '$label could not be read: $readError',
              color: const Color(0xff9e3421),
            ),
          ),
        Padding(
          padding: const EdgeInsets.only(left: 2, bottom: 12),
          child: Copy(
            times.isEmpty
                ? '$label · No saved reading'
                : '$label · ${snapshot?['from_cache'] == true ? 'Cached' : 'Read'} ${receiptTime(times.last, compact: true)}${stale ? ' · Stale' : ''}',
            size: 12,
            color: muted,
          ),
        ),
        if (targetProperty != null && !editable)
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Copy(
              targetProperty['writable'] != true
                  ? '$label target is read-only.'
                  : '$label: ${action == null ? 'Target controls are unavailable.' : actionUnavailable(climate!, action) ?? 'Target controls are unavailable.'}',
              size: 12,
              color: muted,
            ),
          ),
      ],
    );
  }
}
