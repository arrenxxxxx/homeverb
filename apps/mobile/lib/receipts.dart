import 'localization.dart';
import 'dart:convert';
import 'client_contract.dart';

String outcomeLabel(dynamic status) =>
    const {
      'accepted': 'Accepted',
      'verified': 'Verified',
      'failed': 'Failed',
      'unknown': 'Unknown',
      'partial': 'Partially completed',
    }[status] ??
    'Unknown';

String receiptTime(dynamic value, {bool compact = false}) {
  final time = value is String ? DateTime.tryParse(value)?.toUtc() : null;
  if (time == null) return 'Not provided';
  final iso = time.toIso8601String();
  return compact
      ? '${iso.substring(0, 16).replaceFirst('T', ' ')} UTC'
      : iso
            .replaceFirst('T', ' ')
            .replaceFirst('.000Z', 'Z')
            .replaceFirst('Z', ' UTC');
}

String outcomeSummary(dynamic status) =>
    const {
      'accepted': 'Request accepted; completion has not been verified.',
      'verified': 'Verification evidence was recorded for this operation.',
      'failed': 'The request failed. Inspect its recorded attempts.',
      'unknown':
          'This request may have executed. Refresh before considering another write.',
      'partial': 'Outcomes differ. Completed actions are not rolled back.',
    }[status] ??
    'No recognized outcome is available.';

String outcomeColor(dynamic status) => status == 'verified'
    ? 'success'
    : status == 'failed'
    ? 'error'
    : 'warning';

String routeLabel(dynamic value) =>
    const {'lan': 'Local network', 'cloud': 'Provider cloud'}[value] ??
    'Not provided';

List<dynamic> receiptBlocks(Json receipt, String deviceName) {
  final verification = receipt['verification'] as Json?;
  final error = receipt['error'] as Json?;
  final data = receipt['data'] as Json?;
  final attempts = receipt['attempts'] as List? ?? [];
  final items = receipt['items'] as List? ?? [];
  return [
    [
      'status',
      outcomeLabel(receipt['status']),
      outcomeSummary(receipt['status']),
      outcomeColor(receipt['status']),
    ],
    [
      'rows',
      [
        ['Device', LiteralText(deviceName), null],
        ['Started', receiptTime(receipt['started_at']), null],
        ['Recorded', receiptTime(receipt['completed_at']), null],
      ],
    ],
    if (error != null) ...[
      [
        'rows',
        [
          ['Error', '${error['code']}', null],
        ],
      ],
      ['note', UiError(error['code'])],
    ],
    if (verification != null) ...[
      [
        'card',
        'Verification evidence',
        [
          [
            'Method',
            verification['kind'] == 'correlated_completion'
                ? 'Correlated completion'
                : 'State match',
            null,
          ],
          ['Source', routeLabel(verification['source']), null],
          ['Observed', receiptTime(verification['observed_at']), null],
          ['Received', receiptTime(verification['received_at']), null],
        ],
      ],
      if (verification['note'] != null) ['note', verification['note']],
      [
        'note',
        verification['kind'] == 'correlated_completion'
            ? 'Completion evidence does not establish the current state or exactly-once execution.'
            : 'Matching state alone does not establish physical causality.',
      ],
    ] else
      ['note', 'No verification evidence is recorded for this outcome.'],
    if (data != null && data.isNotEmpty)
      [
        'card',
        'Returned values',
        [
          for (final entry in data.entries)
            [
              entry.key.replaceAll('_', ' '),
              entry.value is String ? entry.value : jsonEncode(entry.value),
              null,
            ],
        ],
      ],
    if (attempts.isNotEmpty)
      [
        'card',
        'Recorded attempts',
        [
          for (var i = 0; i < attempts.length; i++)
            [
              UiMessage('Attempt {count}', {'count': i + 1}),
              UiMessage('{route} · {phase} · {duration} ms', {
                'route': UiMessage(routeLabel(attempts[i]['transport'])),
                'phase': UiMessage(
                  const {
                        'not_sent': 'Not sent',
                        'possibly_sent': 'May have been sent',
                        'accepted': 'Accepted',
                        'rejected': 'Rejected',
                      }[attempts[i]['phase']] ??
                      'Unknown phase',
                ),
                'duration': attempts[i]['duration_ms'],
              }),
              null,
            ],
        ],
      ]
    else
      ['note', 'No attempts are recorded.'],
    if (items.isNotEmpty)
      [
        'card',
        'Sub-operations',
        [
          for (var i = 0; i < items.length; i++)
            [
              UiMessage('Step {count}', {'count': i + 1}),
              outcomeLabel(items[i]['status']),
              'receipt-item:$i',
            ],
        ],
      ],
    for (final warning in receipt['warnings'] as List? ?? [])
      [
        'note',
        UiMessage('Warning: {warning}', {'warning': UiMessage(warning)}),
      ],
    [
      'note',
      'Refresh reads device state; it never repeats this write or changes the retained outcome.',
    ],
    ['button', 'Refresh State'],
    ['button', 'View Operations', 'secondary'],
  ];
}
