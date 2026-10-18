import 'localization.dart';
import 'receipts.dart';
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
    final template = arguments.containsKey('additional')
        ? labels['$action.confirmAdditional'] ?? labels['$action.confirm']
        : labels['$action.confirm'];
    final effects = template == null
        ? null
        : UiMessage(template, {
            for (final entry in values.entries)
              entry.key: entry.key == 'device'
                  ? LiteralText(entry.value)
                  : UiMessage(entry.value),
          });
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
              Copy(
                UiMessage('Device: {device}', {'device': values['device']!}),
              ),
              Copy(
                UiMessage('Local ID: {value}', {'value': command['device_id']}),
              ),
              Copy(
                UiMessage('Component: {value}', {
                  'value': command['component_id'],
                }),
              ),
              Copy(
                UiMessage('Action: {value}', {
                  'value': UiMessage(labels[action] ?? action),
                }),
              ),
              Copy(
                UiMessage('Arguments: {value}', {
                  'value': jsonEncode(arguments),
                }),
              ),
              Copy(
                UiMessage('Connection: {transport} · {connection}', {
                  'transport': UiMessage(
                    routeLabel(preview.connection['transport']),
                  ),
                  'connection': preview.connection['id'],
                }),
              ),
              Copy(
                UiMessage('Expires: {value}', {
                  'value': preview.expiresAt.toUtc().toIso8601String(),
                }),
              ),
              Copy(
                UiMessage('Request: {value}', {'value': command['request_id']}),
              ),
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
