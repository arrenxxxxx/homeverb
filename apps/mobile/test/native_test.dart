import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/preferences.dart';

void main() {
  final library = File('../../target/debug/libhomeverb_ffi.so').absolute.path;
  test(
    'native organization, validation, observations, receipt and reopen',
    () async {
      final dir = await Directory.systemTemp.createTemp('homeverb-dart-');
      final core = await NativeClient.open(
        dataDir: dir.path,
        libraryPath: library,
        allowMockWrites: true,
      );
      final prefs = LocalPreferences(File('${dir.path}/ui.json'));
      final state = HomeController(core, prefs);
      try {
        expect(await state.initialize(), true);
        expect(state.cards, isEmpty);
        expect(await state.discover(), true);
        expect(state.cards.length, 1);
        expect(await state.saveRoom('Study'), true);
        expect(await state.saveRoom('Study'), false);
        final id = state.cards.single['device']['id'] as String;
        await state.selectDevice(id);
        expect(await state.refresh(), true);
        final before = state.observed('climate.target_temperature');
        expect(
          await state.execute('climate', 'set', {'target_temperature': 25}),
          true,
        );
        expect(state.receipt!['status'], 'verified');
        expect(state.observed('climate.target_temperature'), before);
        expect(await state.refresh(), true);
        expect(state.observed('climate.target_temperature'), 25);
        expect(
          await state.execute('climate', 'set', {'target_temperature': 100}),
          false,
        );
        expect(await state.assign(state.rooms.single['id']), true);
        expect(
          await state.saveDevice({'alias': 'Quiet AC', 'favorite': true}),
          true,
        );
        expect(
          state.currentCard!['preferences']['room_id'],
          state.rooms.single['id'],
        );
        expect(await state.deleteRoom(state.rooms.single['id']), true);
        expect(state.currentCard!['preferences']['room_id'], null);
        expect(await state.loadHistory(), true);
        expect(state.history, isNotEmpty);
        final request = state.receipt!['request_id'] as String;
        expect(await state.loadReceipt(request), true);
        expect(await state.loadDiagnostics(), true);
        core.foreground(false);
        await expectLater(
          core.call('list_rooms'),
          throwsA(isA<NativeFailure>()),
        );
        core.foreground(true);
        await core.close();
        final reopened = await NativeClient.open(
          dataDir: dir.path,
          libraryPath: library,
        );
        try {
          expect(
            (await reopened.call('list_device_cards') as List)
                .single['device']['alias'],
            'Quiet AC',
          );
          expect(
            (await reopened.call('get_execution', {
              'request_id': request,
            }))['status'],
            'verified',
          );
        } finally {
          await reopened.close();
        }
      } finally {
        await core.close();
        await dir.delete(recursive: true);
      }
    },
  );
  for (final pair in {
    'lost-ack': 'unknown',
    'accept-only': 'accepted',
    'reject': 'failed',
  }.entries) {
    test('preserve ${pair.value} and do not replay on refresh', () async {
      final dir = await Directory.systemTemp.createTemp('homeverb-outcome-');
      final core = await NativeClient.open(
        dataDir: dir.path,
        libraryPath: library,
        allowMockWrites: true,
        scenario: pair.key,
      );
      try {
        final state = HomeController(
          core,
          LocalPreferences(File('${dir.path}/ui.json')),
        );
        await state.discover();
        await state.selectDevice(state.cards.single['device']['id']);
        expect(await state.execute('power', 'set', {'state': true}), true);
        expect(state.receipt!['status'], pair.value);
        expect(state.receipt!['attempts'], isNotEmpty);
        await state.loadHistory();
        final count = state.history.length;
        await state.refresh();
        await state.loadHistory();
        expect(state.history.length, count);
      } finally {
        await core.close();
        await dir.delete(recursive: true);
      }
    });
  }
  test('unknown outcomes fail closed and null query success is valid', () {
    expect(
      () => decodeResponse({'status': 'future_success'}),
      throwsA(isA<NativeFailure>()),
    );
    expect(decodeResponse({'data': null, 'error': null}), null);
    expect(
      decodeResponse({
        'status': 'unknown',
        'error': {'code': 'WRITE_OUTCOME_UNKNOWN'},
      })['status'],
      'unknown',
    );
  });
  test('component numeric schemas retain their own bounds and steps', () {
    expect(
      validateValue(-18, {
        'type': 'number',
        'minimum': -24,
        'maximum': -16,
        'multipleOf': 1,
      }),
      null,
    );
    expect(
      validateValue(-18, {
        'type': 'number',
        'minimum': 2,
        'maximum': 8,
        'multipleOf': 1,
      }),
      isNotNull,
    );
    expect(validateValue(23.5, {'type': 'number', 'multipleOf': 1}), isNotNull);
    expect(validateValue(23.5, {'type': 'number', 'multipleOf': .5}), null);
  });
  test('preferences survive restart and corrupt skin falls back', () async {
    final dir = await Directory.systemTemp.createTemp('homeverb-prefs-');
    try {
      final file = File('${dir.path}/ui.json'),
          prefs = LocalPreferences(File('${dir.path}/ui.json'));
      await prefs.save({
        'skin_id': 'unknown',
        'sharing_name': 'Study phone',
        'route': 'lan',
      });
      final again = LocalPreferences(file);
      await again.load();
      expect(again.skin, 'window_light');
      expect(again.sharingName, 'Study phone');
      expect(again.route(), 'lan');
      await file.writeAsString('{broken');
      final broken = LocalPreferences(file);
      await broken.load();
      expect(broken.skin, 'window_light');
    } finally {
      await dir.delete(recursive: true);
    }
  });
  test(
    'rapid background and resume drains queued calls before reopening admission',
    () async {
      final dir = await Directory.systemTemp.createTemp('homeverb-lifecycle-');
      final core = await NativeClient.open(
        dataDir: dir.path,
        libraryPath: library,
        allowMockWrites: true,
        scenario: 'slow-before-send',
      );
      try {
        final devices = await core.call('discover') as List;
        final command = {
          'schema_version': '1.0',
          'request_id': uniqueId(),
          'device_id': devices.single['id'],
          'component_id': 'main',
          'capability': 'power',
          'action': 'set',
          'arguments': {'state': true},
          'connection': 'auto',
          'timeout_ms': 10000,
          'wait_for': 'verified',
        };
        final call = core.call('execute', {'command': command});
        core.foreground(false);
        core.foreground(true);
        try {
          final result = await call as Json;
          expect(result['status'], 'failed');
        } on NativeFailure catch (e) {
          expect(['HOST_BACKGROUND', 'CANCELLED'].contains(e.code), true);
        }
        expect(core.active, isEmpty);
        expect(await core.call('list_devices'), isNotEmpty);
        final receipts = await core.call('recent_executions') as List;
        expect(
          receipts.every(
            (r) =>
                (r['attempts'] as List).every((a) => a['phase'] == 'not_sent'),
          ),
          true,
        );
      } finally {
        await core.close();
        await dir.delete(recursive: true);
      }
    },
  );
  test(
    'batch envelopes and nested partial results survive Dart decoding',
    () async {
      final dir = await Directory.systemTemp.createTemp('homeverb-batch-');
      final core = await NativeClient.open(
        dataDir: dir.path,
        libraryPath: library,
        allowMockWrites: true,
        scenario: 'partial',
      );
      try {
        final device = (await core.call('discover') as List).single;
        final command = {
          'schema_version': '1.0',
          'request_id': uniqueId(),
          'device_id': device['id'],
          'component_id': 'main',
          'capability': 'climate',
          'action': 'set',
          'arguments': {'mode': 'heat', 'target_temperature': 25},
          'connection': 'auto',
          'timeout_ms': 10000,
          'wait_for': 'verified',
        };
        final batch = {
          'schema_version': '1.0',
          'commands': [command],
        };
        expect(await core.call('plan_batch', {'batch': batch}), isNotNull);
        final receipt =
            await core.call('execute_batch', {'batch': batch}) as Json;
        expect(receipt['status'], 'partial');
        expect(receipt['items'].single['status'], 'partial');
        expect(receipt['items'].single['items'].length, 2);
      } finally {
        await core.close();
        await dir.delete(recursive: true);
      }
    },
  );
}
