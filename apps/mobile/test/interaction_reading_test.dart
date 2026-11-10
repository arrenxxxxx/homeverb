import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/device_page.dart';
import 'package:homeverb_mobile/feedback.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'accounts_test.dart' show AccountPreferences;
import 'interaction_fixture.dart';

List<dynamic> liveReads(InteractionFixture core) => core
    .callsFor('read_state')
    .where((call) => call['cached'] == false)
    .toList();

Object? reading(HomeController c, String component) => c.componentSnapshot(
  component,
)?['observations']?['climate.target_temperature']?['value'];

Future<void> frames(WidgetTester tester, [int count = 5]) async {
  for (var i = 0; i < count; i++) {
    await tester.pump();
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('first component failures end loading with one opening Toast', (
    tester,
  ) async {
    final core = await InteractionFixture.open();
    final device = core.device['id'] as String;
    final fridge = core.hold(device, 'fridge');
    final freezer = core.hold(device, 'freezer');
    core.failures.addAll({(device, 'fridge'), (device, 'freezer')});
    final c = HomeController(core, AccountPreferences());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: c),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Example refrigerator').first);
    await frames(tester);
    expect(app.currentState!.id, 'C08');
    expect(c.readingLoading('fridge'), true);
    expect(c.readingLoading('freezer'), true);
    expect(find.byType(DeviceLoading), findsOneWidget);
    expect(c.openingReadFailureSerial, 0);
    await tester.pump(const Duration(seconds: 5));
    expect(c.readingLoading('fridge'), true);
    fridge.complete();
    await frames(tester);
    expect(c.readingLoading('fridge'), false);
    expect(c.readingLoading('freezer'), true);
    expect(c.openingReadFailureSerial, 0);
    freezer.complete();
    await frames(tester, 10);
    expect(c.readingLoading('freezer'), false);
    expect(c.openingReadFailureSerial, 1);
    expect(find.byType(FeedbackToast), findsOneWidget);
    expect(find.text('Could not load readings.'), findsOneWidget);
    expect(find.text('Unknown'), findsNothing);
    expect(find.text('—'), findsWidgets);
    expect(core.callsFor('execute'), isEmpty);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('partial opening publishes success while another read is held', (
    tester,
  ) async {
    final core = await InteractionFixture.open();
    final device = core.device['id'] as String;
    final freezer = core.hold(device, 'freezer');
    core.failures.add((device, 'freezer'));
    core.values[(device, 'fridge')] = 6;
    final c = HomeController(core, AccountPreferences());
    addTearDown(c.dispose);
    expect(await c.initialize(), true);
    final opening = c.selectDevice(device);
    await frames(tester);
    expect(reading(c, 'fridge'), 6);
    expect(c.readingLoading('fridge'), false);
    expect(c.readingLoading('freezer'), true);
    expect(liveReads(core), hasLength(2));
    freezer.complete();
    await frames(tester);
    expect(await opening, true);
    expect(reading(c, 'fridge'), 6);
    expect(reading(c, 'freezer'), isNull);
    expect(c.openingReadFailureSerial, 1);
    expect(core.callsFor('execute'), isEmpty);
  });

  testWidgets('cached component survives failure while another is held', (
    tester,
  ) async {
    final core = await InteractionFixture.open();
    final device = core.device['id'] as String;
    await core.call('read_state', {
      'device': device,
      'component': 'fridge',
      'cached': false,
    });
    core.calls.clear();
    core.failures.add((device, 'fridge'));
    final freezer = core.hold(device, 'freezer');
    final c = HomeController(core, AccountPreferences());
    addTearDown(c.dispose);
    expect(await c.initialize(), true);
    final opening = c.selectDevice(device);
    await frames(tester);
    expect(reading(c, 'fridge'), 4);
    expect(c.readingLoading('fridge'), false);
    expect(c.readingLoading('freezer'), true);
    expect(c.openingReadFailureSerial, 0);
    freezer.complete();
    await frames(tester);
    expect(await opening, true);
    expect(reading(c, 'fridge'), 4);
    expect(reading(c, 'freezer'), -18);
    expect(c.openingReadFailureSerial, 0);
    expect(core.callsFor('execute'), isEmpty);
  });

  testWidgets(
    'manual refresh publishes completed components and combines failure',
    (tester) async {
      final core = await InteractionFixture.open();
      final device = core.device['id'] as String;
      core.values[(device, 'fridge')] = 8;
      final freezer = core.hold(device, 'freezer');
      core.failures.add((device, 'freezer'));
      final c = HomeController(core, AccountPreferences());
      addTearDown(c.dispose);
      expect(await c.initialize(), true);
      core.calls.clear();
      var publications = 0;
      c.addListener(() => publications++);
      final refresh = c.refreshCards();
      await frames(tester);
      expect(freezer.isCompleted, false);
      expect(
        c.snapshots[device]?['observations']?['climate.target_temperature']?['value'],
        8,
      );
      expect(publications, greaterThan(1));
      expect(liveReads(core), hasLength(2));
      freezer.complete();
      await frames(tester);
      expect(await refresh, false);
      expect(c.error, isNotNull);
      expect(
        c.snapshots[device]?['observations']?['climate.target_temperature']?['value'],
        8,
      );
      core.failures
        ..clear()
        ..add((device, 'fridge'));
      core.gates.remove((device, 'freezer'));
      core.values[(device, 'freezer')] = -20;
      c.selectedDevice = device;
      c.component = 'freezer';
      expect(await c.refreshCards(), false);
      expect(reading(c, 'freezer'), -20);
      expect(core.callsFor('execute'), isEmpty);
    },
  );

  testWidgets(
    'cached readings survive ten quiet failed cycles and background',
    (tester) async {
      final core = await InteractionFixture.open();
      final device = core.device['id'] as String;
      final c = HomeController(core, AccountPreferences());
      addTearDown(c.dispose);
      expect(await c.initialize(), true);
      c.setAutomaticRefreshView(StateRefreshView.device);
      c.startAutomaticRefresh();
      final opening = c.selectDevice(device);
      await frames(tester);
      expect(await opening, true);
      expect(reading(c, 'fridge'), 4);
      expect(reading(c, 'freezer'), -18);
      final prior = c.componentSnapshot('fridge');
      c.receipt = {'status': 'unknown', 'request_id': 'fictional-request'};
      final receipt = c.receipt;
      final openingCount = liveReads(core).length;
      expect(openingCount, 2);
      core.failures.addAll({(device, 'fridge'), (device, 'freezer')});
      await tester.pump(const Duration(milliseconds: 2999));
      expect(liveReads(core), hasLength(openingCount));
      await tester.pump(const Duration(milliseconds: 1));
      await frames(tester);
      expect(liveReads(core), hasLength(openingCount + 2));
      for (var cycle = 0; cycle < 9; cycle++) {
        await tester.pump(const Duration(seconds: 3));
        await frames(tester);
      }
      expect(liveReads(core), hasLength(openingCount + 20));
      expect(c.componentSnapshot('fridge'), same(prior));
      expect(c.receipt, same(receipt));
      expect(c.error, isNull);
      expect(c.openingReadFailureSerial, 0);
      expect(core.callsFor('execute'), isEmpty);
      final gate = core.hold(device, 'fridge');
      await tester.pump(const Duration(seconds: 3));
      await frames(tester);
      final beforeBackground = liveReads(core).length;
      c.setForeground(false);
      gate.complete();
      await tester.pump(const Duration(seconds: 9));
      await frames(tester);
      expect(liveReads(core), hasLength(beforeBackground));
      expect(c.componentSnapshot('fridge'), same(prior));
      c.setForeground(true);
      await tester.pump(const Duration(milliseconds: 2999));
      expect(liveReads(core), hasLength(beforeBackground));
      await tester.pump(const Duration(milliseconds: 1));
      await frames(tester);
      expect(liveReads(core).length, greaterThan(beforeBackground));
      c.stopAutomaticRefresh();
    },
  );
}
