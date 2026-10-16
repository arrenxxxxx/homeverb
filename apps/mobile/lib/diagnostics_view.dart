import 'localization.dart';
import 'native_client.dart';
import 'receipts.dart';

const diagnosticReviewNote =
    'Review before saving. Account and device identifiers are excluded.';

List<dynamic> diagnosticsBlocks(Json? report, {required bool exporting}) {
  if (report == null) {
    return [
      ['note', 'No report loaded.'],
      ['button', 'Export Redacted Report', 'disabled'],
      ['button', 'Cancel', 'secondary'],
    ];
  }
  final connections = report['connections'] as Json;
  final latest = (report['recent_receipts'] as List).firstOrNull as Json?;
  final attempt = (latest?['attempts'] as List?)?.lastOrNull as Json?;
  final phase = const {
    'not_sent': 'Not sent',
    'possibly_sent': 'Possibly sent',
    'accepted': 'Accepted',
    'rejected': 'Rejected',
  }[attempt?['phase']];
  final transport = const {
    'lan': 'LAN',
    'cloud': 'Cloud',
  }[attempt?['transport']];
  return [
    [
      'rows',
      [
        ['App version', report['application_version'], null],
        ['Generated', receiptTime(report['generated_at'], compact: true), null],
        ['Local scope', 'This profile', null],
        [
          'Devices / rooms',
          '${report['device_count']} / ${report['room_count']}',
          null,
        ],
        [
          'LAN / cloud routes',
          '${connections['lan']} / ${connections['cloud']}',
          null,
        ],
        [
          'Receipt / attempt limits',
          '${report['receipt_limit']} / ${report['attempt_limit']}',
          null,
        ],
      ],
    ],
    [
      'card',
      'Latest receipt',
      [
        [
          'Outcome',
          latest == null
              ? 'No recent operations'
              : outcomeLabel(latest['status']),
          null,
        ],
        ['Error', latest == null ? '—' : latest['error_code'] ?? 'None', null],
        [
          'Phase',
          attempt == null
              ? 'No attempts'
              : UiMessage('{phase} · {transport}{suffix}', {
                  'phase': UiMessage(phase ?? 'Unknown'),
                  'transport': UiMessage(transport ?? 'Not provided'),
                  'suffix': UiMessage(
                    latest!['attempts_omitted'] > 0 ? ' (retained)' : '',
                  ),
                }),
          null,
        ],
        [
          'Attempts omitted',
          latest == null ? '—' : '${latest['attempts_omitted']}',
          null,
        ],
      ],
    ],
    ['note', diagnosticReviewNote, 'Review Full Report'],
    ['button', 'Export Redacted Report', exporting ? 'disabled' : ''],
    ['button', 'Cancel', 'secondary'],
  ];
}
