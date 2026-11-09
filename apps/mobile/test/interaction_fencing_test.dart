import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'accounts_test.dart' show AccountFixture, AccountPreferences;
import 'interaction_fixture.dart';

class HeldLocalFixture extends AccountFixture {
  final gates = <String, Completer<void>>{};
  final entered = <String, Completer<void>>{};

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    final result = await super.call(method, fields);
    final gate = gates[method];
    if (gate != null) {
      entered[method]!.complete();
      await gate.future;
    }
    return result;
  }
}

class HeldRefreshFixture extends AccountFixture {
  HeldRefreshFixture() {
    cards.clear();
    for (final id in ['first-device', 'second-device', 'third-device']) {
      cards.add({
        'device': {
          'id': id,
          'alias': id,
          'model': 'Fixture',
          'support_status': 'experimental',
          'components': [
            {
              'id': 'main',
              'capabilities': ['sensor'],
            },
          ],
          'connections': [],
        },
        'preferences': {'room_id': null, 'favorite': false},
      });
    }
  }
  final gate = Completer<void>();
  final secondStarted = Completer<void>();
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'read_state') {
      requests.add({'method': method, ...fields});
      if (fields['cached'] == true) {
        throw NativeFailure('CACHE_MISS', 'No fictional snapshot.');
      }
      if (fields['device'] == 'second-device') {
        secondStarted.complete();
        await gate.future;
        throw NativeFailure('NETWORK_UNAVAILABLE', 'Fictional late failure.');
      }
      return {
        'device_id': fields['device'],
        'component_id': fields['component'],
        'observations': {
          'sensor.value': {'value': 1},
        },
      };
    }
    return super.call(method, fields);
  }
}

Future<void> frames(WidgetTester tester, [int count = 8]) async {
  for (var i = 0; i < count; i++) {
    await tester.pump();
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  for (final interruption in ['navigation', 'background-resume']) {
    testWidgets('manual multi-card refresh stops after $interruption', (
      tester,
    ) async {
      final core = HeldRefreshFixture();
      final c = HomeController(core, AccountPreferences());
      expect(await c.initialize(), true);
      core.requests.clear();
      final work = c.refreshCards();
      await core.secondStarted.future;
      expect(
        c.snapshots['first-device']?['observations']?['sensor.value']?['value'],
        1,
      );
      expect(
        core.requests
            .where((call) => call['method'] == 'read_state')
            .map((call) => call['device']),
        ['first-device', 'second-device'],
      );
      if (interruption == 'navigation') {
        c.cancelPending();
      } else {
        c.setForeground(false);
        c.setForeground(true);
      }
      core.gate.complete();
      expect(await work, false);
      expect(
        core.requests
            .where((call) => call['method'] == 'read_state')
            .map((call) => call['device']),
        ['first-device', 'second-device'],
      );
      expect(
        c.snapshots['first-device']?['observations']?['sensor.value']?['value'],
        1,
      );
      expect(c.snapshots['second-device'], isNull);
      expect(c.snapshots['third-device'], isNull);
      expect(c.error, isNull);
      c.dispose();
    });

    testWidgets('manual multi-component refresh stops after $interruption', (
      tester,
    ) async {
      final core = await InteractionFixture.open();
      core.device['components'].add({
        'id': 'third',
        'capabilities': ['climate'],
      });
      core.targets['third'] = 6;
      final c = HomeController(core, AccountPreferences());
      expect(await c.initialize(), true);
      expect(await c.selectDevice(core.device['id']), true);
      final device = core.device['id'] as String;
      final previous = c.componentSnapshot('freezer');
      core.values[(device, 'fridge')] = 7;
      core.failures.add((device, 'freezer'));
      final gate = core.hold(device, 'freezer');
      core.calls.clear();
      final work = c.refreshComponents();
      await frames(tester);
      expect(core.callsFor('read_state').map((call) => call['component']), [
        'fridge',
        'freezer',
      ]);
      expect(
        c.componentSnapshot(
          'fridge',
        )?['observations']?['climate.target_temperature']?['value'],
        7,
      );
      if (interruption == 'navigation') {
        c.cancelPending();
      } else {
        c.setForeground(false);
        c.setForeground(true);
      }
      gate.complete();
      expect(await work, false);
      expect(core.callsFor('read_state').map((call) => call['component']), [
        'fridge',
        'freezer',
      ]);
      expect(c.componentSnapshot('freezer'), same(previous));
      expect(c.componentReadError('freezer'), isNull);
      expect(c.error, isNull);
      c.dispose();
    });
  }

  testWidgets('reopening clears the previous component failure while loading', (
    tester,
  ) async {
    final core = await InteractionFixture.open();
    final device = core.device['id'] as String;
    core.failures.add((device, 'fridge'));
    final c = HomeController(core, AccountPreferences());
    expect(await c.initialize(), true);
    expect(await c.selectDevice(device), true);
    expect(c.componentReadError('fridge'), isA<NativeFailure>());
    expect(c.componentSnapshot('fridge'), isNull);
    c.cancelPending();
    core.failures.remove((device, 'fridge'));
    final gate = core.hold(device, 'fridge');
    final opening = c.selectDevice(device);
    await frames(tester);
    expect(c.readingLoading('fridge'), true);
    expect(c.componentReadError('fridge'), isNull);
    gate.complete();
    expect(await opening, true);
    expect(c.componentReadError('fridge'), isNull);
    expect(c.componentSnapshot('fridge'), isNotNull);
    c.dispose();
  });

  for (final method in ['list_device_cards', 'list_accounts']) {
    for (final interruption in ['navigation', 'background']) {
      testWidgets('$method does not publish after $interruption', (
        tester,
      ) async {
        final core = HeldLocalFixture();
        final c = HomeController(core, AccountPreferences());
        expect(await c.initialize(), isTrue);
        final priorCards = c.cards;
        final priorAccounts = c.accounts;
        core.cards.add({
          'device': {
            'id': 'fictional-late-device',
            'alias': 'Late device',
            'model': 'Fixture',
            'components': <Json>[],
            'connections': <Json>[],
          },
          'preferences': {'room_id': null, 'favorite': false},
        });
        core.records.add({
          'id': 'fictional-late-account',
          'label': 'Late account',
          'provider': 'midea.meiju',
          'region': 'CN',
          'credential_state': 'stored',
          'authorization': 'not_checked',
        });
        final gate = core.gates[method] = Completer<void>();
        final started = core.entered[method] = Completer<void>();
        final work = method == 'list_accounts'
            ? c.loadAccounts()
            : c.initialize();
        await started.future;
        if (interruption == 'navigation') {
          c.cancelPending();
        } else {
          c.setForeground(false);
        }
        gate.complete();
        expect(await work, isFalse);
        expect(c.cards, same(priorCards));
        expect(c.accounts, same(priorAccounts));
        expect(
          c.cards.any(
            (card) => card['device']['id'] == 'fictional-late-device',
          ),
          isFalse,
        );
        expect(
          c.accounts.any(
            (account) => account['id'] == 'fictional-late-account',
          ),
          isFalse,
        );
        expect(
          core.requests.any(
            (call) => ['plan', 'execute'].contains(call['method']),
          ),
          isFalse,
        );
        c.dispose();
      });
    }
  }

  for (final interruption in ['back', 'background', 'removal']) {
    testWidgets('held receipt read is fenced after $interruption', (
      tester,
    ) async {
      final core = await InteractionFixture.open();
      final device = core.device['id'] as String;
      final gate = core.hold(device, 'fridge');
      final c = HomeController(core, AccountPreferences());
      final app = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: app, catalog: catalog, controller: c),
      );
      await tester.pumpAndSettle();
      final receipt = <String, dynamic>{
        'device_id': device,
        'request_id': 'fictional-uncertain-request',
        'status': 'unknown',
      };
      c.selectedDevice = device;
      c.component = 'fridge';
      c.receipt = receipt;
      await app.currentState!.go('S06');
      await tester.pump();
      core.calls.clear();
      final work = app.currentState!.action('Refresh State');
      await frames(tester);
      expect(
        core.callsFor('read_state').where((call) => call['cached'] == false),
        hasLength(1),
      );
      expect(gate.isCompleted, isFalse);
      switch (interruption) {
        case 'back':
          await app.currentState!.go('D01', back: true);
        case 'background':
          app.currentState!.didChangeAppLifecycleState(
            AppLifecycleState.paused,
          );
        case 'removal':
          // Model local publication of removal independently of network timing.
          c.cards.removeWhere((card) => card['device']['id'] == device);
      }
      gate.complete();
      await work;
      await frames(tester);
      expect(c.snapshots.containsKey(device), isFalse);
      expect(c.componentSnapshot('fridge'), isNull);
      expect(c.receipt, same(receipt));
      expect(c.error, isNull);
      expect(core.callsFor('execute'), isEmpty);
      expect(core.callsFor('plan'), isEmpty);
      expect(find.byType(SnackBar), findsNothing);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    });
  }

  for (final filter in ['room', 'search']) {
    testWidgets('held automatic read is fenced after changing $filter', (
      tester,
    ) async {
      final core = await InteractionFixture.open();
      final device = core.device['id'] as String;
      final gate = core.hold(device, 'fridge');
      final c = HomeController(core, AccountPreferences());
      final app = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: app, catalog: catalog, controller: c),
      );
      await tester.pumpAndSettle();
      if (filter == 'room') {
        c.rooms = [
          {'id': 'empty-room', 'name': 'Office'},
        ];
      } else {
        await app.currentState!.go('D03');
        await tester.pump();
      }
      core.calls.clear();
      c.startAutomaticRefresh();
      await tester.pump(const Duration(seconds: 3));
      await frames(tester);
      expect(
        core.callsFor('read_state').where((call) => call['cached'] == false),
        hasLength(1),
      );
      expect(gate.isCompleted, isFalse);
      if (filter == 'room') {
        await app.currentState!.action('room:Office');
      } else {
        await tester.enterText(
          find.byKey(const ValueKey('field:Search devices')),
          'no matching device',
        );
      }
      await frames(tester);
      expect(c.visibleCards, isEmpty);
      gate.complete();
      await frames(tester);
      expect(c.snapshots.containsKey(device), isFalse);
      expect(c.error, isNull);
      expect(core.callsFor('execute'), isEmpty);
      expect(find.byType(SnackBar), findsNothing);
      await tester.pump(const Duration(seconds: 3));
      await frames(tester);
      expect(
        core.callsFor('read_state').where((call) => call['cached'] == false),
        hasLength(1),
      );
      c.stopAutomaticRefresh();
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    });
  }
}
