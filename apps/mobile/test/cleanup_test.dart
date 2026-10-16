import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter/material.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';

class InterruptedCleanup implements CoreClient {
  final requests = <String>[];
  bool background = false;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    requests.add(method);
    if (background) throw NativeFailure('HOST_BACKGROUND', 'Paused');
    switch (method) {
      case 'clear_cached_state':
        background = true;
        throw NativeFailure('CANCELLED', 'No completed cleanup report.');
      case 'list_rooms':
      case 'list_device_cards':
      case 'recent_executions':
        return <Json>[];
      default:
        throw StateError('Unexpected call: $method');
    }
  }

  @override
  void cancelAll() {}
  @override
  void foreground(bool value) {
    background = !value;
  }

  @override
  Future<void> close() async {}
}

class BusyHistory extends InterruptedCleanup {
  bool journalActive = true;
  bool cleared = false;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    requests.add(method);
    switch (method) {
      case 'clear_execution_history':
        if (journalActive)
          throw NativeFailure('RESOURCE_BUSY', 'An operation is still active.');
        cleared = true;
        return {'removed_receipts': 1};
      case 'list_rooms':
      case 'list_device_cards':
        return <Json>[];
      case 'recent_executions':
        return cleared
            ? <Json>[]
            : <Json>[
                {'request_id': 'retained-fictional', 'status': 'unknown'},
              ];
      default:
        throw StateError('Unexpected call: $method');
    }
  }
}

void main() {
  testWidgets(
    'busy history cleanup requires local reload and explicit reconfirmation',
    (tester) async {
      final core = BusyHistory();
      final state = HomeController(
        core,
        LocalPreferences(File('/unused-fixture.json')),
      );
      final key = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      tester.view.physicalSize = const Size(390, 844);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await tester.pumpWidget(
        NativeApp(key: key, catalog: catalog, controller: state),
      );
      await tester.pumpAndSettle();
      await key.currentState!.go('S04');
      await tester.pumpAndSettle();
      Future<void> tap(String text) async {
        final button = find.text(text).last;
        await tester.ensureVisible(button);
        await tester.tap(button);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
      }

      await tap('Clear Recent Operations');
      await tap('Confirm Clear');
      expect((state.error as NativeFailure).code, 'RESOURCE_BUSY');
      expect(key.currentState!.id, 'S09');
      expect(find.text('Confirm Clear'), findsNothing);
      expect(
        core.requests.where((m) => m == 'clear_execution_history'),
        hasLength(1),
      );
      await tap('Reload local data');
      expect(state.history, hasLength(1));
      expect(state.cleanupNeedsReload, false);
      expect(
        core.requests.where((m) => m == 'clear_execution_history'),
        hasLength(1),
      );
      core.journalActive = false;
      await tap('Confirm Clear');
      expect(key.currentState!.id, 'S04');
      expect(state.cleanupCount, 1);
      expect(state.history, isEmpty);
      expect(
        core.requests.where((m) => m == 'clear_execution_history'),
        hasLength(2),
      );
      expect(
        core.requests.any(
          (m) => ['execute', 'read_state', 'discover'].contains(m),
        ),
        false,
      );
      await tester.pumpWidget(const SizedBox());
    },
  );

  test(
    'native cache and history cleanup preserve organization and retire IDs',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'homeverb-cleanup-',
      );
      final core = await NativeClient.open(
        dataDir: directory.path,
        libraryPath: File(
          '../../target/debug/${NativeClient.defaultLibrary()}',
        ).absolute.path,
        allowMockWrites: true,
        scenario: 'lost-ack',
      );
      final state = HomeController(
        core,
        LocalPreferences(File('${directory.path}/ui.json')),
      );
      try {
        expect(state.supports('clear_cached_state'), true);
        expect(state.supports('clear_execution_history'), true);
        await state.discover();
        await state.saveRoom('Study');
        final device = state.cards.single['device']['id'];
        await state.selectDevice(device);
        await state.saveDevice({'alias': 'Desk', 'favorite': true});
        await state.assign(state.rooms.single['id']);
        await state.refresh();
        await state.execute('power', 'set', {'state': true});
        expect(state.receipt!['status'], 'unknown');
        final request = state.receipt!['request_id'];
        final snapshot = state.snapshot;
        expect(await state.clearLocalData(LocalCleanup.executionHistory), true);
        expect(state.cleanupCount, 1);
        expect(state.history, isEmpty);
        expect(state.receipt, isNull);
        expect(state.snapshot!['observations'], snapshot!['observations']);
        expect(state.snapshot!['from_cache'], true);
        await expectLater(
          core.call('get_execution', {'request_id': request}),
          throwsA(
            isA<NativeFailure>().having(
              (e) => e.code,
              'code',
              'REQUEST_RETIRED',
            ),
          ),
        );
        final replay =
            await core.call('execute', {
                  'command': {
                    'schema_version': '1.0',
                    'request_id': request,
                    'device_id': device,
                    'component_id': 'main',
                    'capability': 'power',
                    'action': 'set',
                    'arguments': {'state': true},
                    'connection': 'auto',
                    'timeout_ms': 10000,
                    'wait_for': 'verified',
                  },
                })
                as Json;
        expect(replay['status'], 'failed');
        expect(replay['error']['code'], 'REQUEST_RETIRED');
        expect(replay['attempts'], isEmpty);
        expect(await state.clearLocalData(LocalCleanup.cachedState), true);
        expect(state.cleanupCount, 1);
        expect(state.snapshot, isNull);
        expect(state.snapshots, isEmpty);
        expect(state.componentSnapshot('main'), isNull);
        expect(await state.selectDevice(device), true);
        expect(state.snapshot, isNull);
        expect(state.cards.single['preferences']['alias'], 'Desk');
        expect(state.cards.single['preferences']['favorite'], true);
        expect(
          state.cards.single['preferences']['room_id'],
          state.rooms.single['id'],
        );
        expect(state.cleanupNeedsReload, false);
        expect(await state.clearLocalData(LocalCleanup.cachedState), true);
        expect(state.cleanupCount, 0);
        expect(await state.refresh(), true);
        expect(state.snapshot, isNotNull);
        expect(state.history, isEmpty);
      } finally {
        await core.close();
        await directory.delete(recursive: true);
      }
    },
  );

  test(
    'lost cleanup reply invalidates UI and resumes with local queries only',
    () async {
      final core = InterruptedCleanup();
      final state = HomeController(
        core,
        LocalPreferences(File('/unused-fixture.json')),
      );
      state.snapshot = {'observations': {}};
      state.snapshots['fixture'] = state.snapshot!;
      expect(await state.clearLocalData(LocalCleanup.cachedState), false);
      expect(state.cleanupCount, isNull);
      expect(state.cleanupNeedsReload, true);
      expect(state.snapshot, isNull);
      expect(state.snapshots, isEmpty);
      expect(core.requests, ['clear_cached_state']);
      state.setForeground(true);
      expect(core.requests, ['clear_cached_state']);
      expect(await state.initialize(), true);
      expect(state.cleanupNeedsReload, false);
      expect(core.requests, [
        'clear_cached_state',
        'list_rooms',
        'list_device_cards',
        'recent_executions',
      ]);
    },
  );
}
