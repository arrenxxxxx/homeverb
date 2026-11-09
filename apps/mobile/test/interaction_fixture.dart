import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:homeverb_mobile/native_client.dart';

import 'component_test.dart' show CompartmentFixture, example;

/// Fictional, inspectable state reads for interaction and lifecycle tests.
/// A new client can reopen the same isolated profile file to model a process
/// restart; the file contains only fixture snapshots, never credentials.
class InteractionFixture extends CompartmentFixture {
  InteractionFixture._(this.profileFile, this.profile, this._saved);

  static Future<InteractionFixture> open({
    File? profileFile,
    String profile = 'fictional-profile',
  }) async {
    final saved = profileFile != null && await profileFile.exists()
        ? jsonDecode(await profileFile.readAsString()) as Json
        : <String, dynamic>{};
    return InteractionFixture._(profileFile, profile, saved);
  }

  final File? profileFile;
  final String profile;
  final Json _saved;
  final gates = <(String, String), Completer<void>>{};
  final cachedGates = <(String, String), Completer<void>>{};
  final failures = <(String, String)>{};
  final values = <(String, String), num>{};
  Duration? deadline;
  bool closed = false;
  int closeCount = 0;
  int activeReads = 0;
  int maxActiveReads = 0;

  (String, String) key(Json fields) =>
      (fields['device'] as String, fields['component'] as String);

  List<Json> callsFor(String method) =>
      calls.where((call) => call['method'] == method).toList();

  Completer<void> hold(String device, String component) =>
      gates[(device, component)] = Completer<void>();

  Json get _profileCache =>
      _saved.putIfAbsent(profile, () => <String, dynamic>{}) as Json;

  String _cacheKey((String, String) target) =>
      jsonEncode([target.$1, target.$2]);

  Future<void> _save() async {
    final file = profileFile;
    if (file != null) {
      await file.writeAsString(jsonEncode(_saved), flush: true);
    }
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (closed) throw StateError('The fictional session is closed.');
    if (method != 'read_state') return super.call(method, fields);
    calls.add({'method': method, ...fields});
    final target = key(fields);
    if (fields['cached'] == true) {
      await cachedGates[target]?.future;
      final cached = _profileCache[_cacheKey(target)];
      if (cached == null) {
        throw NativeFailure('STATE_UNAVAILABLE', 'No fictional snapshot.');
      }
      return {...jsonDecode(jsonEncode(cached)) as Json, 'from_cache': true};
    }
    activeReads++;
    if (activeReads > maxActiveReads) maxActiveReads = activeReads;
    try {
      final gate = gates[target];
      if (gate != null) {
        final timeout = deadline;
        if (timeout == null) {
          await gate.future;
        } else {
          await gate.future.timeout(timeout);
        }
      }
      if (failures.contains(target)) {
        throw NativeFailure('NETWORK_UNAVAILABLE', 'Fictional read failure.');
      }
      final snapshot = <String, dynamic>{
        ...example('state'),
        'device_id': target.$1,
        'component_id': target.$2,
        'from_cache': false,
        'observations': {
          'climate.target_temperature': {
            ...example('state')['observations']['climate.target_temperature']
                as Json,
            'value':
                values[target] ??
                targets[target.$2] ??
                (throw StateError('No fictional value for $target.')),
          },
        },
      };
      _profileCache[_cacheKey(target)] = jsonDecode(jsonEncode(snapshot));
      await _save();
      return snapshot;
    } finally {
      activeReads--;
    }
  }

  @override
  Future<void> close() async {
    closed = true;
    closeCount++;
  }
}
