import 'package:flutter/material.dart';
import 'blocks.dart';
import 'controller.dart';
import 'controls.dart';
import 'device_presentation.dart';
import 'localization.dart';
import 'native_client.dart';
import 'skin.dart';

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
    super.key,
  });
  final HomeController controller;
  final ActionHandler onAction;

  @override
  Widget build(BuildContext context) {
    final c = controller;
    final controls = <Widget>[];
    final unavailableReasons = <String>{};
    for (var i = 0; i < c.capabilities.length; i++) {
      final capability = c.capabilities[i], id = capability['id'] as String;
      for (final entry in (capability['actions'] as Json).entries) {
        final action = entry.value as Json;
        if (!c.hasControlAction(id, entry.key)) continue;
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
              c.observed('$id.$name') == null,
        ))
          continue;
        final label =
            sourceForKey('$id.${entry.key}') ?? controlLabel(entry.key);
        controls.add(
          ActionButton(
            label,
            key: ValueKey('control:$i:${Uri.encodeComponent(entry.key)}'),
            style: 'secondary',
            onPressed: c.busy || !c.isForeground
                ? null
                : () =>
                      onAction('control:$i:${Uri.encodeComponent(entry.key)}'),
          ),
        );
      }
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        DeviceDetailHeader(controller: c),
        if (c.components.length > 1)
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              for (final part in c.components)
                ChoiceChip(
                  label: Copy(controlLabel(part['id'] as String)),
                  selected: c.component == part['id'],
                  onSelected: c.busy || !c.isForeground
                      ? null
                      : (_) => onAction('component:${part['id']}'),
                ),
            ],
          ),
        const Padding(
          padding: EdgeInsets.fromLTRB(2, 4, 2, 12),
          child: Copy('Available controls', bold: true),
        ),
        if (controls.isEmpty)
          const Padding(
            padding: EdgeInsets.only(bottom: 16),
            child: Copy(
              'No controls are allowed for this device on this connection.',
              color: muted,
            ),
          ),
        if (controls.isNotEmpty)
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
        for (final reason in unavailableReasons)
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Copy(reason, color: muted),
          ),
        for (final capability in c.capabilities)
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
                      UiMessage('{value}{stale}', {
                        'value': UiMessage(
                          propertyValue(
                            c.observed('${capability['id']}.${entry.key}'),
                            entry.value as Json,
                            propertyId: '${capability['id']}.${entry.key}',
                          ),
                        ),
                        'stale': UiMessage(
                          c.snapshot?['observations']?['${capability['id']}.${entry.key}']?['stale'] ==
                                  true
                              ? ' · Stale'
                              : '',
                        ),
                      }),
                      null,
                    ],
              ],
              title: c.capabilities.length == 1
                  ? 'Observed readings'
                  : controlLabel(capability['id'] as String),
              onAction: onAction,
            ),
        ActionButton(
          'Refresh State',
          style: 'secondary',
          onPressed: c.busy || !c.isForeground
              ? null
              : () => onAction('Refresh State'),
        ),
        ActionButton(
          'Device information',
          style: 'secondary',
          onPressed: c.busy || !c.isForeground
              ? null
              : () => onAction('Device information'),
        ),
        ActionButton(
          'Recent Operations',
          style: 'secondary',
          onPressed: c.busy || !c.isForeground
              ? null
              : () => onAction('Recent Operations'),
        ),
      ],
    );
  }
}
