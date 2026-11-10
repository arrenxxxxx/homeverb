import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/device_page.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'accounts_test.dart'
    show AccountFixture, AccountPreferences, firstAccount;
import 'component_test.dart' show example;
import 'interaction_fixture.dart';
import 'session_permissions_test.dart' show PermissionCore, inspection;

const deviceA = 'fictional-opening-a';
const deviceB = 'fictional-opening-b';

/// Adds two independent cards to the restartable fictional observation store.
class OpeningClient implements CoreClient {
  OpeningClient(this.inner) {
    for (final entry in [
      (deviceA, 'Opening fridge A'),
      (deviceB, 'Opening fridge B'),
    ]) {
      final device = jsonDecode(jsonEncode(inner.device)) as Json;
      device['id'] = entry.$1;
      device['alias'] = entry.$2;
      devices[entry.$1] = device;
    }
  }

  final InteractionFixture inner;
  final devices = <String, Json>{};
  final descriptions = <String, Completer<void>>{};
  final calls = <Json>[];

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    calls.add({'method': method, ...fields});
    if (method == 'list_accounts') return <Json>[];
    if (method == 'list_device_cards') {
      return [
        for (final device in devices.values)
          {
            'device': device,
            'preferences': {'alias': null, 'room_id': null, 'favorite': false},
          },
      ];
    }
    if (method == 'describe_device') {
      final id = fields['device'] as String;
      await descriptions[id]?.future;
      return {'device': devices[id]!, 'capabilities': inner.capabilities};
    }
    return inner.call(method, fields);
  }

  @override
  void cancelAll() => inner.cancelAll();
  @override
  void foreground(bool value) => inner.foreground(value);
  @override
  Future<void> close() => inner.close();
}

class HeldAccountClient extends AccountFixture {
  final detailsGate = Completer<void>();
  bool holdDetails = false;

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'get_account' && holdDetails) await detailsGate.future;
    return super.call(method, fields);
  }
}

class HeldRootPreferences extends AccountPreferences {
  final gate = Completer<void>();
  final roots = <String>[];
  Future<void> tail = Future.value();

  @override
  Future<void> save(Json changes) {
    final root = changes['root'] as String?;
    if (root != null) roots.add(root);
    final pending = tail.then((_) async {
      if (root == 'P01') await gate.future;
      await super.save(changes);
    });
    tail = pending;
    return pending;
  }
}

class OrderedPermissionCore extends PermissionCore {
  OrderedPermissionCore(super.inspected);

  final liveGate = Completer<void>();
  int active = 0;
  int? activeAtClose;
  int callsAfterClose = 0;

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (closed) {
      callsAfterClose++;
      throw StateError('A closed fictional session was used.');
    }
    active++;
    try {
      if (method == 'read_state' && fields['cached'] == false) {
        requests.add({'method': method, ...fields});
        await liveGate.future;
        return {
          ...example('state'),
          'device_id': fields['device'],
          'component_id': fields['component'],
          'from_cache': false,
          'observations': {
            'power.state': {'value': true},
          },
        };
      }
      return await super.call(method, fields);
    } finally {
      active--;
    }
  }

  @override
  Future<void> close() async {
    activeAtClose = active;
    await super.close();
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  Future<void> pumps(WidgetTester tester, [int count = 5]) async {
    for (var i = 0; i < count; i++) {
      await tester.pump();
    }
  }

  testWidgets('root tabs remain usable during a held preference save', (
    tester,
  ) async {
    final core = AccountFixture();
    final preferences = HeldRootPreferences();
    final controller = HomeController(core, preferences);
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: controller),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Platforms'));
    await pumps(tester);
    expect(app.currentState!.id, 'P01');
    expect(preferences.gate.isCompleted, false);
    await tester.tap(find.text('Settings'));
    await pumps(tester);
    expect(app.currentState!.id, 'S01');
    await tester.tap(find.text('Devices'));
    await pumps(tester);
    expect(app.currentState!.id, 'D01');
    expect(controller.busy, false);
    expect(preferences.roots, ['P01', 'S01', 'D01']);
    preferences.gate.complete();
    await pumps(tester);
    expect(preferences.root, 'D01');
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('background and resume fence a late opening read', (
    tester,
  ) async {
    final inner = await InteractionFixture.open();
    final core = OpeningClient(inner);
    final controller = HomeController(core, AccountPreferences());
    expect(await controller.initialize(), true);
    final gate = inner.hold(deviceA, 'fridge');
    final opening = controller.selectDevice(deviceA);
    await pumps(tester);
    expect(inner.activeReads, 1);
    controller.setForeground(false);
    controller.setForeground(true);
    gate.complete();
    await pumps(tester);
    expect(await opening, false);
    expect(controller.componentSnapshot('fridge'), isNull);
    expect(controller.snapshots[deviceA], isNull);
    expect(controller.componentReadError('fridge'), isNull);
    expect(controller.error, isNull);
    expect(
      inner.callsFor('read_state').where((r) => r['cached'] == false),
      hasLength(1),
    );
    controller.dispose();
  });

  testWidgets('account reload removal fences a late opening read', (
    tester,
  ) async {
    final inner = await InteractionFixture.open();
    final core = OpeningClient(inner);
    final controller = HomeController(core, AccountPreferences());
    expect(await controller.initialize(), true);
    final gate = inner.hold(deviceA, 'fridge');
    final opening = controller.selectDevice(deviceA);
    await pumps(tester);
    expect(inner.activeReads, 1);
    core.devices.remove(deviceA);
    await controller.reloadAccountViews();
    expect(controller.selectedDevice, isNull);
    expect(controller.description, isNull);
    expect(
      controller.cards.any((card) => card['device']['id'] == deviceA),
      false,
    );
    gate.complete();
    await pumps(tester);
    expect(await opening, false);
    expect(controller.selectedDevice, isNull);
    expect(controller.description, isNull);
    expect(controller.snapshots[deviceA], isNull);
    expect(controller.componentSnapshot('fridge'), isNull);
    expect(controller.componentReadError('fridge'), isNull);
    expect(controller.error, isNull);
    expect(
      controller.cards.any((card) => card['device']['id'] == deviceA),
      false,
    );
    controller.dispose();
  });

  testWidgets('opening reads drain before control inspection and replacement', (
    tester,
  ) async {
    final options = inspection();
    final old = OrderedPermissionCore(options);
    final replacement = PermissionCore(options);
    final replacementGate = Completer<void>();
    var replacementStarted = false;
    int? activeAtReopen;
    bool? closedAtReopen;
    final controller = HomeController(
      old,
      AccountPreferences(),
      reopen: (_) async {
        replacementStarted = true;
        activeAtReopen = old.active;
        closedAtReopen = old.closed;
        await replacementGate.future;
        return replacement;
      },
    );
    expect(await controller.initialize(), true);
    old.requests.clear();
    old.inspectionGate = Completer<Json>();
    final opening = controller.selectDevice(options['device_id'] as String);
    await pumps(tester);
    expect(
      old.requests.where(
        (request) =>
            request['method'] == 'read_state' && request['cached'] == false,
      ),
      hasLength(1),
    );
    expect(
      old.requests.where((r) => r['method'] == 'inspect_control_options'),
      isEmpty,
    );
    expect(old.closed, false);
    old.liveGate.complete();
    await pumps(tester);
    expect(controller.observed('power.state'), true);
    expect(
      old.requests.where((r) => r['method'] == 'inspect_control_options'),
      hasLength(1),
    );
    await tester.pump(const Duration(seconds: 5));
    expect(old.inspectionGate!.isCompleted, false);
    expect(old.closed, false);
    old.inspectionGate!.complete(options);
    await pumps(tester);
    expect(replacementStarted, true);
    expect(activeAtReopen, 0);
    expect(closedAtReopen, true);
    expect(old.activeAtClose, 0);
    final callCount = old.requests.length;
    expect(await controller.refresh(), false);
    expect(old.requests.length, callCount);
    expect(old.callsAfterClose, 0);
    expect(replacement.requests, isEmpty);
    replacementGate.complete();
    await pumps(tester, 15);
    expect(await opening, true, reason: '${controller.error}');
    expect(old.callsAfterClose, 0);
    expect(old.activeAtClose, 0);
    controller.dispose();
  });

  testWidgets(
    'device shell and Back are available while local metadata waits',
    (tester) async {
      final inner = await InteractionFixture.open();
      final core = OpeningClient(inner);
      final controller = HomeController(core, AccountPreferences());
      final app = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: app, catalog: catalog, controller: controller),
      );
      await tester.pumpAndSettle();
      final gate = core.descriptions[deviceA] = Completer<void>();
      await tester.tap(find.text('Opening fridge A').first);
      await tester.pump();
      expect(['C01', 'C08'], contains(app.currentState!.id));
      expect(find.byTooltip('Back'), findsOneWidget);
      await tester.pump(const Duration(seconds: 5));
      expect(gate.isCompleted, false);
      expect(tester.takeException(), isNull);
      await tester.tap(find.byTooltip('Back'));
      await tester.pump();
      expect(app.currentState!.id, 'D01');
      gate.complete();
      await pumps(tester);
      expect(app.currentState!.id, 'D01');
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets('late A metadata cannot replace a newly opened B', (
    tester,
  ) async {
    final inner = await InteractionFixture.open();
    final core = OpeningClient(inner);
    final controller = HomeController(core, AccountPreferences());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: controller),
    );
    await tester.pumpAndSettle();
    final gate = core.descriptions[deviceA] = Completer<void>();
    await tester.tap(find.text('Opening fridge A').first);
    await tester.pump();
    await tester.tap(find.byTooltip('Back'));
    await tester.pump();
    await tester.tap(find.text('Opening fridge B').first);
    await pumps(tester);
    expect(['C01', 'C08'], contains(app.currentState!.id));
    expect(controller.selectedDevice, deviceB);
    gate.complete();
    await pumps(tester);
    expect(controller.selectedDevice, deviceB);
    expect(controller.description?['device']?['id'], deviceB);
    expect(find.text('Opening fridge A'), findsNothing);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets(
    'first opening keeps restored readings behind a stable loading view',
    (tester) async {
      final directory = await tester.runAsync(
        () => Directory.systemTemp.createTemp('opening-cache-'),
      );
      addTearDown(() => directory!.deleteSync(recursive: true));
      final file = File('${directory!.path}/fictional.json');
      final seed = (await tester.runAsync(
        () => InteractionFixture.open(profileFile: file),
      ))!;
      await tester.runAsync(() async {
        for (final component in ['fridge', 'freezer']) {
          await seed.call('read_state', {
            'device': deviceA,
            'component': component,
            'cached': false,
          });
        }
        await seed.close();
      });
      final inner = (await tester.runAsync(
        () => InteractionFixture.open(profileFile: file),
      ))!;
      final gate = inner.hold(deviceA, 'fridge');
      final core = OpeningClient(inner);
      final controller = HomeController(core, AccountPreferences());
      final app = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: app, catalog: catalog, controller: controller),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Opening fridge A').first);
      await pumps(tester, 12);
      expect(['C01', 'C08'], contains(app.currentState!.id));
      expect(gate.isCompleted, false);
      expect(
        inner
            .callsFor('read_state')
            .any(
              (call) => call['device'] == deviceA && call['cached'] == false,
            ),
        true,
      );
      expect(
        controller.componentSnapshot(
          'fridge',
        )?['observations']?['climate.target_temperature']?['value'],
        4,
      );
      expect(
        controller.componentSnapshot(
          'freezer',
        )?['observations']?['climate.target_temperature']?['value'],
        -18,
      );
      expect(find.byType(DeviceLoading), findsOneWidget);
      expect(find.text('4°C'), findsNothing);
      expect(tester.takeException(), isNull);
      gate.complete();
      for (var i = 0; i < 100 && controller.openingDevice; i++) {
        await tester.pump();
        await tester.runAsync(
          () => Future<void>.delayed(const Duration(milliseconds: 10)),
        );
      }
      await pumps(tester, 12);
      expect(find.byType(DeviceLoading), findsNothing);
      expect(find.text('4°C'), findsWidgets);
      await tester.tap(find.byTooltip('Back'));
      await pumps(tester);
      final repeatGate = inner.hold(deviceA, 'fridge');
      await tester.tap(find.text('Opening fridge A').first);
      await pumps(tester, 12);
      expect(find.byType(DeviceLoading), findsNothing);
      expect(find.text('4°C'), findsWidgets);
      repeatGate.complete();
      for (var i = 0; i < 100 && controller.openingDevice; i++) {
        await tester.pump();
        await tester.runAsync(
          () => Future<void>.delayed(const Duration(milliseconds: 10)),
        );
      }
      await pumps(tester, 12);
      final otherGate = inner.hold(deviceB, 'fridge');
      await tester.tap(find.byTooltip('Back'));
      await pumps(tester);
      await tester.tap(find.text('Opening fridge B').first);
      await pumps(tester, 12);
      expect(controller.selectedDevice, deviceB);
      expect(controller.componentSnapshot('fridge'), isNull);
      expect(controller.componentSnapshot('freezer'), isNull);
      otherGate.complete();
      await pumps(tester);
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets('account destination appears before account details complete', (
    tester,
  ) async {
    final core = HeldAccountClient();
    final controller = HomeController(core, AccountPreferences());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: controller),
    );
    await tester.pumpAndSettle();
    await app.currentState!.go('P01');
    await tester.pumpAndSettle();
    core.holdDetails = true;
    await tester.tap(find.byKey(const ValueKey('account:$firstAccount')));
    await tester.pump();
    expect(app.currentState!.id, 'P09');
    expect(find.byTooltip('Back'), findsOneWidget);
    await tester.pump(const Duration(seconds: 5));
    expect(core.detailsGate.isCompleted, false);
    await tester.tap(find.byTooltip('Back'));
    await tester.pump();
    expect(app.currentState!.id, 'P01');
    core.detailsGate.complete();
    await pumps(tester);
    expect(app.currentState!.id, 'P01');
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
  });
}
