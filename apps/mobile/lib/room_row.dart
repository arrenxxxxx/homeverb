import 'package:flutter/material.dart';

import 'localization.dart';
import 'skin.dart';

/// A room's reorder grip and action menu have separate touch targets.
class RoomRow extends StatelessWidget {
  const RoomRow({
    required this.id,
    required this.name,
    required this.deviceCount,
    required this.index,
    required this.onAction,
    required this.canMoveUp,
    required this.canMoveDown,
    this.enabled = true,
    super.key,
  });

  final String id, name;
  final Object deviceCount;
  final int index;
  final ValueChanged<String> onAction;
  final bool canMoveUp, canMoveDown, enabled;

  @override
  Widget build(BuildContext context) {
    final dragLabel = localize(
      context,
      UiMessage('Reorder {name}', {'name': name}),
    );
    final menuLabel = localize(
      context,
      UiMessage('Room actions {name}', {'name': name}),
    );
    return Padding(
      padding: const EdgeInsets.only(left: 16, right: 4, top: 4, bottom: 4),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Copy(LiteralText(name), control: true),
                Copy(deviceCount, control: true, size: 12, color: muted),
              ],
            ),
          ),
          ReorderableDragStartListener(
            key: ValueKey('room-drag:$id'),
            index: index,
            enabled: enabled,
            child: Semantics(
              label: dragLabel,
              button: true,
              enabled: enabled,
              child: Tooltip(
                message: dragLabel,
                child: const SizedBox.square(
                  dimension: 48,
                  child: ExcludeSemantics(
                    child: Icon(Icons.drag_handle, color: muted, size: 21),
                  ),
                ),
              ),
            ),
          ),
          SizedBox.square(
            dimension: 48,
            child: PopupMenuButton<String>(
              key: ValueKey('room-actions:$id'),
              tooltip: menuLabel,
              enabled: enabled,
              padding: EdgeInsets.zero,
              icon: const Icon(Icons.more_vert, color: muted),
              onSelected: onAction,
              itemBuilder: (_) => [
                for (final action in [
                  'Rename',
                  'Delete',
                  'Move devices',
                  'Move up',
                  'Move down',
                ])
                  PopupMenuItem<String>(
                    value: action,
                    enabled: switch (action) {
                      'Move up' => canMoveUp,
                      'Move down' => canMoveDown,
                      _ => true,
                    },
                    child: Text(localize(context, action)),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
