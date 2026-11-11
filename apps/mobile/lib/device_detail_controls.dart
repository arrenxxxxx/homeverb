import 'package:flutter/material.dart';
import 'blocks.dart';
import 'controller.dart';
import 'controls.dart';
import 'device_presentation.dart';
import 'device_readings.dart';
import 'localization.dart';
import 'client_contract.dart';
import 'receipts.dart';
import 'skin.dart';
import 'appliance_panels.dart';

/// Shared detail material; artwork never selects command semantics.
class DeviceDetailHeader extends StatelessWidget {
  const DeviceDetailHeader({required this.controller, super.key});
  final HomeController controller;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: 16),
    child: Surface(
      radius: 18,
      padding: const EdgeInsets.all(16),
      child: Column(
        children: [
          ExcludeSemantics(
            child: SizedBox(
              height: 136,
              width: double.infinity,
              child: product(
                deviceArtwork(controller.description!['device'] as Json),
              ),
            ),
          ),
          const SizedBox(height: 8),
          Copy(
            controller.currentCard?['preferences']['room_id'] == null
                ? 'Unassigned'
                : LiteralText(
                    controller.roomName(
                      controller.currentCard!['preferences']['room_id'],
                    ),
                  ),
            size: 13,
            color: muted,
          ),
        ],
      ),
    ),
  );
}

/// Descriptor-driven fallback for appliances without a specialized panel.
/// Buttons open the existing argument/approval flow; they never send a command.
class DeviceDetailControls extends StatelessWidget {
  const DeviceDetailControls({
    required this.controller,
    required this.onAction,
    this.outcomeComponent,
    super.key,
  });
  final HomeController controller;
  final ActionHandler onAction;
  final String? outcomeComponent;

  @override
  Widget build(BuildContext context) {
    final c = controller;
    if (c.components.length == 1 &&
        c.capability('vendor.midea.dishwasher') != null) {
      return DishwasherPanel(controller: c, onAction: onAction);
    }
    final groups = <Widget>[];
    final receipt = c.receipt;
    final showReceipt =
        receipt != null &&
        !c.isCommandError &&
        receipt['device_id'] == c.selectedDevice &&
        (['unknown', 'failed', 'partial'].contains(receipt['status']) ||
            (receipt['status'] == 'accepted' && c.error != null));
    var anyControl = false;
    for (final part in c.components) {
      final component = part['id'] as String;
      final capabilities = c.capabilitiesFor(component);
      final dishwasher = capabilities.any(
        (capability) => capability['id'] == DishwasherPanel.id,
      );
      final genericCapabilities = capabilities
          .where(
            (capability) =>
                capability['id'] != DishwasherPanel.id &&
                (!dishwasher ||
                    capability['id'] != '${DishwasherPanel.id}.details'),
          )
          .toList();
      final snapshot = c.componentSnapshot(component);
      dynamic observed(String property) =>
          snapshot?['observations']?[property]?['value'];
      final controls = <Widget>[];
      final unavailableReasons = <String>{};
      for (final capability in genericCapabilities) {
        final i = capabilities.indexOf(capability),
            id = capability['id'] as String;
        for (final entry in (capability['actions'] as Json).entries) {
          final action = entry.value as Json;
          if (!c.hasControlAction(id, entry.key, componentId: component))
            continue;
          final reason = actionUnavailable(
            capability,
            action,
            trustedApproval: c.supportsTrustedApproval,
          );
          if (reason != null) {
            unavailableReasons.add(reason);
            continue;
          }
          if (((action['parameters']['properties'] as Map?) ?? {}).keys.any(
            (name) =>
                capability['properties'][name]?['control_requires_observation'] ==
                    true &&
                observed('$id.$name') == null,
          ))
            continue;
          final label =
              sourceForKey('$id.${entry.key}') ?? controlLabel(entry.key);
          final token = component == c.component
              ? 'control:$i:${Uri.encodeComponent(entry.key)}'
              : 'control-component:${Uri.encodeComponent(component)}:${Uri.encodeComponent(id)}:${Uri.encodeComponent(entry.key)}';
          controls.add(
            ActionButton(
              label,
              key: ValueKey(token),
              style: 'secondary',
              onPressed: c.busy || !c.isForeground
                  ? null
                  : () => onAction(token),
            ),
          );
        }
      }
      anyControl = anyControl || controls.isNotEmpty || dishwasher;
      if (capabilities.isEmpty) continue;
      groups.addAll([
        if (c.components.length > 1)
          Padding(
            key: ValueKey('component-group:$component'),
            padding: const EdgeInsets.fromLTRB(2, 12, 2, 12),
            child: Copy(controlLabel(component), bold: true, size: 18),
          ),
        if (dishwasher)
          DishwasherPanel(
            key: ValueKey('component-dishwasher:$component'),
            controller: c,
            onAction: onAction,
            componentId: component,
            showInformation: false,
          ),
        if (controls.isNotEmpty) ...[
          const Padding(
            padding: EdgeInsets.fromLTRB(2, 4, 2, 12),
            child: Copy('Available controls', bold: true),
          ),
          LayoutBuilder(
            builder: (context, constraints) {
              final columns =
                  constraints.maxWidth >= 320 &&
                      MediaQuery.textScalerOf(context).scale(14) <= 20 &&
                      controls.length > 1
                  ? 2
                  : 1;
              final width =
                  (constraints.maxWidth - 12 * (columns - 1)) / columns;
              return Wrap(
                spacing: 12,
                children: [
                  for (final control in controls)
                    SizedBox(width: width, child: control),
                ],
              );
            },
          ),
        ],
        if (showReceipt && component == (outcomeComponent ?? c.component))
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Copy(outcomeSummary(receipt['status']), color: muted),
                TextButton(
                  onPressed: c.busy ? null : () => onAction('View Details'),
                  child: const Copy('View Details'),
                ),
              ],
            ),
          ),
        for (final reason in unavailableReasons)
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Copy(reason, color: muted),
          ),
        for (final capability in genericCapabilities)
          if ((capability['properties'] as Json).values.any(
            (p) => p['readable'] == true,
          ))
            Rows(
              [
                for (final entry in (capability['properties'] as Json).entries)
                  if (entry.value['readable'] == true)
                    [
                      sourceForKey('${capability['id']}.${entry.key}') ??
                          controlLabel(entry.key),
                      UiMessage(
                        propertyValue(
                          observed('${capability['id']}.${entry.key}'),
                          entry.value as Json,
                          propertyId: '${capability['id']}.${entry.key}',
                        ),
                      ),
                      null,
                      if (c.readingLoading(component) &&
                          observed('${capability['id']}.${entry.key}') == null)
                        const ReadingValue('—', loading: true),
                    ],
              ],
              title: capabilities.length == 1
                  ? 'Observed readings'
                  : controlLabel(capability['id'] as String),
              onAction: onAction,
            ),
      ]);
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        DeviceDetailHeader(controller: c),
        if (!anyControl &&
            !c.deviceControlsPreparing &&
            !c.deviceControlsRecoveryNeeded)
          const Padding(
            padding: EdgeInsets.only(bottom: 16),
            child: Copy(
              'No controls are allowed for this device on this connection.',
              color: muted,
            ),
          ),
        ...groups,
        InlineDeviceInformation(controller: c, onAction: onAction),
      ],
    );
  }
}
