import 'client_contract.dart';
import 'numeric_history.dart' show historyTime, historyUtc;

Never _invalid() => throw NativeFailure(
  'INVALID_HISTORY',
  'The historical response does not match the selected event.',
);
Map _object(dynamic value, Set<String> keys) {
  if (value is! Map ||
      value.length != keys.length ||
      !value.keys.every(keys.contains))
    _invalid();
  return value;
}

String _text(dynamic value) {
  if (value is! String ||
      value.isEmpty ||
      value.length > 256 ||
      value.runes.any((v) => v < 32 || v >= 127 && v <= 159))
    _invalid();
  return value;
}

int _integer(dynamic value, int min, int max) {
  if (value is! int || value < min || value > max) _invalid();
  return value;
}

/// Exact declared event selectors; event-only services require no scalar property.
class EventHistoryOption {
  EventHistoryOption._(
    this.device,
    this.connection,
    this.source,
    this.target,
    this.maxWindowSeconds,
    this.maxRecords,
  );
  final String device, connection, source;
  final Map<String, String> target;
  final int maxWindowSeconds, maxRecords;

  static List<EventHistoryOption> parse(
    dynamic value,
    Json descriptor,
    String connection,
  ) {
    final envelope = _object(value, {
      'schema_version',
      'device_id',
      'connection_id',
      'options',
    });
    final device = descriptor['device'];
    final routes = (device['connections'] as List)
        .where((v) => v['id'] == connection)
        .toList();
    if (envelope['schema_version'] != '1.0' ||
        envelope['device_id'] != device['id'] ||
        envelope['connection_id'] != connection ||
        routes.length != 1 ||
        !['lan', 'cloud'].contains(routes.single['transport']))
      _invalid();
    final options = envelope['options'];
    if (options is! List || options.length > 64) _invalid();
    final seen = <String>{};
    return List.unmodifiable(
      options.map((raw) {
        final option = _object(raw, {
          'target',
          'projection',
          'max_window_seconds',
          'max_records',
        });
        final target = _object(option['target'], {
          'component_id',
          'capability',
          'event',
        });
        final owned = Map<String, String>.unmodifiable({
          for (final key in target.keys) key as String: _text(target[key]),
        });
        final parts = (device['components'] as List).where(
          (v) => v['id'] == owned['component_id'],
        );
        final caps = (descriptor['capabilities'] as List).where(
          (v) =>
              v['id'] == owned['capability'] &&
              (v['component_id'] == null ||
                  v['component_id'] == owned['component_id']),
        );
        if (parts.length != 1 ||
            !(parts.single['capabilities'] as List).contains(
              owned['capability'],
            ) ||
            caps.length != 1)
          _invalid();
        final event = caps.single['events']?[owned['event']];
        if (event is! Map ||
            event['history'] is! Map ||
            event['history']['projection'] != 'occurrence_times' ||
            option['projection'] != 'occurrence_times' ||
            !seen.add(
              [
                'component_id',
                'capability',
                'event',
              ].map((key) => Uri.encodeComponent(owned[key]!)).join('/'),
            ))
          _invalid();
        return EventHistoryOption._(
          _text(envelope['device_id']),
          _text(connection),
          routes.single['transport'],
          owned,
          _integer(option['max_window_seconds'], 1, 2764800),
          _integer(option['max_records'], 1, 31),
        );
      }),
    );
  }

  EventHistoryQuery query(String start, String end, DateTime now) {
    final from = historyTime(start), until = historyTime(end);
    if (from.millisecondsSinceEpoch <= 0 ||
        !from.isBefore(until) ||
        until.isAfter(now.toUtc()) ||
        until.difference(from).inSeconds > maxWindowSeconds) {
      throw NativeFailure(
        'INVALID_ARGUMENT',
        'Select a past range within the offered limit.',
      );
    }
    return EventHistoryQuery._(this, from, until);
  }
}

class EventHistoryQuery {
  EventHistoryQuery._(this.option, this.start, this.end);
  final EventHistoryOption option;
  final DateTime start, end;
  Json get json => {
    'schema_version': '1.0',
    'device_id': option.device,
    'connection_id': option.connection,
    'target': option.target,
    'projection': 'occurrence_times',
    'start': historyUtc(start),
    'end': historyUtc(end),
  };
}

class EventHistoryResult {
  EventHistoryResult._(
    this.query,
    this.receivedAt,
    this.records,
    this.rejected,
    this.limitReached,
  );
  final EventHistoryQuery query;
  final DateTime receivedAt;
  final List<DateTime> records;
  final int rejected;
  final bool limitReached;

  static EventHistoryResult parse(
    dynamic value,
    EventHistoryQuery query,
    DateTime started,
    DateTime finished,
  ) {
    final raw = _object(value, {
      ...query.json.keys,
      'source',
      'received_at',
      'records',
      'rejected_records',
      'limit_reached',
      'completeness',
    });
    for (final entry in query.json.entries.where((v) => v.key != 'target')) {
      if (raw[entry.key] != entry.value) _invalid();
    }
    final target = _object(raw['target'], query.option.target.keys.toSet());
    if (!query.option.target.entries.every((v) => target[v.key] == v.value) ||
        raw['source'] != query.option.source ||
        raw['completeness'] != 'unproven')
      _invalid();
    final receipt = raw['received_at'];
    final receivedAt = receipt is String ? DateTime.tryParse(receipt) : null;
    if (receivedAt == null ||
        !receivedAt.isUtc ||
        receivedAt.isBefore(started) ||
        receivedAt.isAfter(finished))
      _invalid();
    final records = raw['records'];
    final rejected = _integer(
      raw['rejected_records'],
      0,
      query.option.maxRecords,
    );
    if (records is! List ||
        records.length + rejected > query.option.maxRecords ||
        raw['limit_reached'] !=
            (records.length + rejected == query.option.maxRecords))
      _invalid();
    DateTime? previous;
    final owned = <DateTime>[];
    for (final record in records) {
      final row = _object(record, {'reported_at'});
      DateTime time;
      try {
        time = historyTime(_text(row['reported_at']));
      } catch (_) {
        _invalid();
      }
      if (time.isBefore(query.start) ||
          !time.isBefore(query.end) ||
          previous != null && time.isBefore(previous))
        _invalid();
      owned.add(time);
      previous = time;
    }
    return EventHistoryResult._(
      query,
      receivedAt,
      List.unmodifiable(owned),
      rejected,
      raw['limit_reached'],
    );
  }
}
