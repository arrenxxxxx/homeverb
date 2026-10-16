import 'dart:convert';
import 'package:flutter/material.dart';
import 'native_client.dart';
import 'skin.dart';

/// Only the immutable native preview supplies the confirmation scope.
class ApprovalReview extends StatelessWidget {
  const ApprovalReview({
    required this.preview,
    required this.labels,
    required this.onConfirm,
    required this.onCancel,
    super.key,
  });
  final ApprovalPreview preview;
  final Map<String, String> labels;
  final VoidCallback onConfirm, onCancel;

  @override
  Widget build(BuildContext context) {
    final command = preview.command;
    final device = preview.descriptor['device'] as Json;
    final arguments = command['arguments'] as Json;
    final capability = command['capability'];
    final action = '$capability.${command['action']}';
    final values = <String, String>{
      'device': '${device['alias'] ?? device['id']}',
      for (final entry in arguments.entries)
        entry.key:
            labels['$capability.${entry.key}.${entry.value}'] ??
            '${entry.value}',
    };
    final template = labels['$action.confirm'];
    final effects = template?.replaceAllMapped(
      RegExp(r'\{([^}]+)\}'),
      (match) => values[match[1]] ?? match[0]!,
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const Copy('Review Command', size: 23, bold: true),
        const SizedBox(height: 16),
        Copy(
          effects ??
              'Confirm the exact action below. It may have lasting effects.',
          bold: true,
        ),
        const SizedBox(height: 16),
        Surface(
          radius: 18,
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Copy('Device: ${values['device']}'),
              Copy('Local ID: ${command['device_id']}'),
              Copy('Component: ${command['component_id']}'),
              Copy('Action: ${labels[action] ?? action}'),
              Copy('Arguments: ${jsonEncode(arguments)}'),
              Copy(
                'Connection: ${preview.connection['transport']} · ${preview.connection['id']}',
              ),
              Copy('Expires: ${preview.expiresAt.toUtc().toIso8601String()}'),
              Copy('Request: ${command['request_id']}'),
            ],
          ),
        ),
        const SizedBox(height: 16),
        const Copy(
          'All listed arguments are explicitly set. Preparation sends nothing and does not check live preconditions. Confirmation executes once; an uncertain result must not be repeated automatically.',
        ),
        const SizedBox(height: 16),
        ActionButton('Confirm Command', onPressed: onConfirm),
        ActionButton('Cancel', style: 'secondary', onPressed: onCancel),
      ],
    );
  }
}
