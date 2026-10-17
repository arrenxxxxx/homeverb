import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/skin.dart';
import 'accounts_test.dart' show AccountPreferences;
import 'component_test.dart' show CompartmentFixture, example;

class RefreshFixture extends CompartmentFixture {
  Completer<void>? liveGate;
  final failures = <(String, String)>{};
  int cancellations = 0;
  final foregrounds = <bool>[];
  List<Json> get liveReads => calls
      .where(
        (call) => call['method'] == 'read_state' && call['cached'] == false,
      )
      .toList();
  @override
  void cancelAll() => cancellations++;
  @override
  void foreground(bool value) => foregrounds.add(value);
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'plan' || method == 'execute') {
      calls.add({'method': method, ...fields});
      final command = fields['command'] as Json;
      if (method == 'plan') return command;
      targets[command['component_id']] =
          command['arguments']['target_temperature'];
      final receipt = {
        ...example('command-result.verified'),
        'device_id': device['id'],
        'request_id': command['request_id'],
        'data': command['arguments'],
      };
      receipts.add(receipt);
      return receipt;
    }
    if (method == 'list_device_cards') {
      final first = (await super.call(method, fields) as List).single as Json;
      return [
        {
          ...first,
          'preferences': {'room_id': 'first-room', 'favorite': false},
        },
        {
          'device': {
            ...device,
            'id': 'second-fictional-device',
            'components': [
              {
                'id': 'main',
                'capabilities': ['climate'],
              },
            ],
          },
          'preferences': {'room_id': 'second-room', 'favorite': false},
        },
        {
          'device': {
            ...device,
            'id': 'unsupported-fictional-device',
            'support_status': 'unsupported',
          },
          'preferences': {'room_id': null, 'favorite': false},
        },
      ];
    }
    if (method == 'read_state' && fields['cached'] == false) {
      calls.add({'method': method, ...fields});
      final value = targets[fields['component']] ?? 22;
      await liveGate?.future;
      if (failures.contains((fields['device'], fields['component']))) {
        throw NativeFailure('NETWORK_UNAVAILABLE', 'Fictional read failure.');
      }
      return {
        ...example('state'),
        'device_id': fields['device'],
        'component_id': fields['component'],
        'from_cache': false,
        'observations': {
          'climate.target_temperature': {
            ...example('state')['observations']['climate.target_temperature']
                as Json,
            'value': value,
          },
        },
      };
    }
    return super.call(method, fields);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  Future<(RefreshFixture, HomeController)> setup() async {
    final core = RefreshFixture();
    final c = HomeController(core, AccountPreferences());
    expect(await c.initialize(), true);
    c.setAutomaticRefreshView(StateRefreshView.cards);
    c.startAutomaticRefresh();
    addTearDown(() {
      c.stopAutomaticRefresh();
      c.dispose();
    });
    core.calls.clear();
    return (core, c);
  }

  testWidgets(
    'three-second cycles read supported filtered components without setup or writes',
    (tester) async {
      final (core, c) = await setup();
      c.filter = 'first-room';
      await tester.pump(const Duration(milliseconds: 2999));
      expect(core.liveReads, isEmpty);
      await tester.pump(const Duration(milliseconds: 1));
      expect(core.liveReads.map((read) => read['component']), [
        'fridge',
        'freezer',
      ]);
      expect(c.busy, false);
      expect(c.error, isNull);
      expect(
        c.snapshots[core
            .device['id']]!['observations']['climate.target_temperature']['stale'],
        true,
      );
      c.filter = 'second-room';
      await tester.pump(const Duration(seconds: 3));
      expect(core.liveReads.last['device'], 'second-fictional-device');
      expect(core.liveReads, hasLength(3));
      expect(core.calls.every((call) => call['method'] == 'read_state'), true);
      c.stopAutomaticRefresh();
    },
  );

  testWidgets(
    'failures are silent and independent and preserve old snapshots and receipts',
    (tester) async {
      final (core, c) = await setup();
      await tester.pump(const Duration(seconds: 3));
      final prior = c.snapshots[core.device['id']];
      c.selectedDevice = core.device['id'];
      final manualError = NativeFailure('POLICY_DENIED', 'Prior manual error.');
      c.error = manualError;
      c.receipt = {'status': 'unknown', 'request_id': 'fictional-request'};
      final receipt = c.receipt;
      core.failures.add((core.device['id'], 'fridge'));
      core.targets['freezer'] = -20;
      await tester.pump(const Duration(seconds: 3));
      expect(c.snapshots[core.device['id']], same(prior));
      expect(c.error, same(manualError));
      expect(c.receipt, same(receipt));
      expect(c.componentReadError('fridge'), isNull);
      expect(
        c.componentSnapshot(
          'freezer',
        )!['observations']['climate.target_temperature']['value'],
        -20,
      );
      expect(core.liveReads, hasLength(6));
      expect(c.busy, false);
      await tester.pump(const Duration(seconds: 2));
      expect(core.liveReads, hasLength(6));
      c.stopAutomaticRefresh();
    },
  );

  testWidgets('slow cycles skip ticks without overlap or catch-up bursts', (
    tester,
  ) async {
    final (core, c) = await setup();
    core.liveGate = Completer<void>();
    await tester.pump(const Duration(seconds: 3));
    await tester.pump(const Duration(seconds: 12));
    expect(core.liveReads, hasLength(1));
    core.liveGate!.complete();
    await tester.pump();
    expect(core.liveReads, hasLength(3));
    await tester.pump(const Duration(seconds: 2));
    expect(core.liveReads, hasLength(3));
    await tester.pump(const Duration(seconds: 1));
    expect(core.liveReads, hasLength(6));
    c.stopAutomaticRefresh();
  });

  testWidgets(
    'background cancels and fences old replies and resume starts a new interval',
    (tester) async {
      final (core, c) = await setup();
      core.liveGate = Completer<void>();
      await tester.pump(const Duration(seconds: 3));
      c.setForeground(false);
      core.liveGate!.complete();
      await tester.pump(const Duration(seconds: 12));
      expect(core.foregrounds, [false]);
      expect(core.liveReads, hasLength(1));
      expect(c.snapshots, isEmpty);
      c.setForeground(true);
      await tester.pump(const Duration(seconds: 2));
      expect(core.liveReads, hasLength(1));
      await tester.pump(const Duration(seconds: 1));
      expect(core.liveReads, hasLength(4));
      expect(c.snapshots, isNotEmpty);
      c.stopAutomaticRefresh();
    },
  );

  testWidgets(
    'a user command cancels and drains reads before planning without replay',
    (tester) async {
      final (core, c) = await setup();
      expect(await c.selectDevice(core.device['id']), true);
      c.setAutomaticRefreshView(StateRefreshView.device);
      core.calls.clear();
      core.liveGate = Completer<void>();
      final prior = c.snapshot;
      await tester.pump(const Duration(seconds: 3));
      final command = c.execute('climate', 'set', {'target_temperature': 5});
      await tester.pump();
      expect(core.cancellations, greaterThan(0));
      expect(core.calls.any((call) => call['method'] == 'plan'), false);
      core.liveGate!.complete();
      await tester.pump();
      expect(await command, true, reason: '${c.error}');
      expect(c.snapshot, same(prior));
      expect(
        core.calls.where((call) => call['method'] == 'execute'),
        hasLength(1),
      );
      final receipt = c.receipt;
      await tester.pump(const Duration(seconds: 3));
      expect(c.observed('climate.target_temperature'), 5);
      expect(
        c.componentSnapshot(
          'freezer',
        )!['observations']['climate.target_temperature']['value'],
        -18,
      );
      expect(c.receipt, same(receipt));
      expect(
        core.calls.where((call) => call['method'] == 'execute'),
        hasLength(1),
      );
      c.stopAutomaticRefresh();
    },
  );

  testWidgets(
    'a user command cancelled while draining cannot start after resume',
    (tester) async {
      final (core, c) = await setup();
      expect(await c.selectDevice(core.device['id']), true);
      core.calls.clear();
      core.liveGate = Completer<void>();
      await tester.pump(const Duration(seconds: 3));
      final command = c.execute('climate', 'set', {'target_temperature': 5});
      c.setForeground(false);
      c.setForeground(true);
      core.liveGate!.complete();
      await tester.pump();
      expect(await command, false);
      expect((c.error as NativeFailure).code, 'CANCELLED');
      expect(
        core.calls.any((call) => ['plan', 'execute'].contains(call['method'])),
        false,
      );
      c.stopAutomaticRefresh();
    },
  );

  testWidgets(
    'leaving a device view cancels late reads and disposal stops scheduling',
    (tester) async {
      final (core, c) = await setup();
      core.liveGate = Completer<void>();
      await tester.pump(const Duration(seconds: 3));
      c.cancelPending();
      c.setAutomaticRefreshView(StateRefreshView.none);
      core.liveGate!.complete();
      await tester.pump(const Duration(seconds: 30));
      expect(core.liveReads, hasLength(1));
      expect(c.snapshots, isEmpty);
      c.setAutomaticRefreshView(StateRefreshView.cards);
      c.stopAutomaticRefresh();
      await tester.pump(const Duration(seconds: 6));
      expect(core.liveReads, hasLength(1));
    },
  );

  testWidgets(
    'native device views refresh silently and non-device navigation suspends reads',
    (tester) async {
      final core = RefreshFixture();
      final c = HomeController(core, AccountPreferences())
        ..startAutomaticRefresh();
      final key = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: key, catalog: catalog, controller: c),
      );
      await tester.pumpAndSettle();
      core.calls.clear();
      await tester.pump(const Duration(seconds: 3));
      expect(core.liveReads, hasLength(3));
      expect(find.byType(OperationProgress), findsNothing);
      expect(find.byType(SnackBar), findsNothing);
      core.failures.add((core.device['id'], 'fridge'));
      await tester.pump(const Duration(seconds: 3));
      expect(c.error, isNull);
      expect(find.byType(OperationProgress), findsNothing);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      final beforeBackground = core.liveReads.length;
      await tester.pump(const Duration(seconds: 6));
      expect(core.liveReads, hasLength(beforeBackground));
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pump(const Duration(seconds: 3));
      expect(core.liveReads.length, greaterThan(beforeBackground));
      await key.currentState!.go('S01');
      await tester.pumpAndSettle();
      final reads = core.liveReads.length;
      await tester.pump(const Duration(seconds: 6));
      expect(core.liveReads, hasLength(reads));
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
