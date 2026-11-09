import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'accounts_test.dart' show AccountPreferences;
import 'component_test.dart' show CompartmentFixture, example;
import 'controls_test.dart' show capability, setter;

const extra = 'vendor.example.extra';
const action = 'set_indicator';
const route = 'control-component:freezer:vendor.example.extra:set_indicator';

class RoutingFixture extends CompartmentFixture {
  RoutingFixture() {
    device['components'][1]['capabilities'].add(extra);
    final definition = capability(
      extra,
      {
        'indicator': {
          'type': 'boolean',
          'readable': true,
          'writable': true,
          'control_requires_observation': true,
        },
      },
      {
        action: setter(
          {
            'indicator': {'type': 'boolean'},
          },
          required: ['indicator'],
        ),
      },
    );
    definition['component_id'] = 'freezer';
    capabilities.add(definition);
  }

  bool indicatorReported = false;
  bool failLive = false;
  String outcome = 'accepted';
  Completer<void>? cachedFreezerGate;
  Completer<void>? executionGate;

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'read_state' &&
        fields['cached'] == true &&
        fields['component'] == 'freezer') {
      await cachedFreezerGate?.future;
    }
    if (method == 'read_state' && fields['cached'] == false && failLive) {
      calls.add({'method': method, ...fields});
      throw NativeFailure('NETWORK_UNAVAILABLE', 'Fictional read failure.');
    }
    if (method == 'execute') {
      calls.add({'method': method, ...fields});
      await executionGate?.future;
      final command = fields['command'] as Json;
      return {
        ...example('command-result.$outcome'),
        'device_id': command['device_id'],
        'request_id': command['request_id'],
        'data': command['arguments'],
      };
    }
    final result = await super.call(method, fields);
    if (method == 'read_state' &&
        fields['component'] == 'freezer' &&
        indicatorReported) {
      result['observations']['$extra.indicator'] = {
        'value': false,
        'source': 'lan',
        'observed_at': null,
        'received_at': '2026-10-14T05:00:01Z',
        'stale': true,
      };
    }
    return result;
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  Future<(RoutingFixture, HomeController, GlobalKey<NativeAppState>)> open(
    WidgetTester tester,
  ) async {
    tester.view.physicalSize = const Size(780, 1688);
    tester.view.devicePixelRatio = 2;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final core = RoutingFixture();
    final c = HomeController(core, AccountPreferences());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: c),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Example refrigerator').first);
    await tester.pumpAndSettle();
    expect(app.currentState!.id, 'C08');
    expect(c.component, 'fridge');
    return (core, c, app);
  }

  testWidgets('guarded non-current component form requires an observation', (
    tester,
  ) async {
    final (core, c, app) = await open(tester);
    final button = find.byKey(
      const ValueKey('component-control:freezer:$extra:$action'),
    );
    expect(button, findsNothing);
    await app.currentState!.action(route);
    await tester.pumpAndSettle();
    expect(app.currentState!.id, 'C08');
    expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
    core.indicatorReported = true;
    expect(await c.refreshComponents(), true);
    expect(await c.selectComponent('fridge'), true);
    await tester.pumpAndSettle();
    expect(button, findsOneWidget);
    await tester.ensureVisible(button);
    await tester.tap(button);
    await tester.pumpAndSettle();
    expect(c.component, 'freezer');
    expect(app.currentState!.id, 'C10');
    expect(app.currentState!.actionName, action);
    expect(app.currentState!.actionCapability!['id'], extra);
    expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  for (final background in [false, true]) {
    testWidgets(
      'late component form is rejected after ${background ? 'background' : 'navigation'}',
      (tester) async {
        final (core, c, app) = await open(tester);
        core.indicatorReported = true;
        expect(await c.refreshComponents(), true);
        core.cachedFreezerGate = Completer<void>();
        final request = app.currentState!.action(route);
        await tester.pump();
        expect(c.busy, true);
        if (background) {
          tester.binding.handleAppLifecycleStateChanged(
            AppLifecycleState.paused,
          );
          tester.binding.handleAppLifecycleStateChanged(
            AppLifecycleState.resumed,
          );
        } else {
          await app.currentState!.go('S01');
        }
        core.cachedFreezerGate!.complete();
        await request;
        await tester.pumpAndSettle();
        expect(app.currentState!.id, background ? 'C08' : 'S01');
        expect(app.currentState!.actionName, isNull);
        expect(
          core.calls.where((call) => call['method'] == 'execute'),
          isEmpty,
        );
        await tester.pumpWidget(const SizedBox());
        c.dispose();
      },
    );
  }

  for (final outcome in ['accepted', 'unknown']) {
    testWidgets('component $outcome and failed readback do not replay', (
      tester,
    ) async {
      final (core, c, app) = await open(tester);
      core.outcome = outcome;
      core.failLive = true;
      core.executionGate = Completer<void>();
      core.calls.clear();
      final stepper = find.byKey(const ValueKey('component-stepper:freezer'));
      await tester.ensureVisible(stepper);
      final increase = find
          .descendant(of: stepper, matching: find.byType(InkWell))
          .last;
      await tester.tap(increase);
      await tester.pump();
      expect(
        find.byKey(const ValueKey('component-pending:freezer')),
        findsOneWidget,
      );
      expect(
        find.byKey(const ValueKey('component-pending:fridge')),
        findsNothing,
      );
      expect(
        c.componentSnapshot(
          'freezer',
        )!['observations']['climate.target_temperature']['value'],
        -18,
      );
      await tester.tap(increase);
      await tester.pump();
      expect(
        core.calls.where((call) => call['method'] == 'execute'),
        hasLength(1),
      );
      core.executionGate!.complete();
      await tester.pumpAndSettle();
      expect(app.currentState!.id, 'C08');
      expect(c.receipt!['status'], outcome);
      expect(c.error, isA<NativeFailure>());
      expect(
        c.componentSnapshot(
          'freezer',
        )!['observations']['climate.target_temperature']['value'],
        -18,
      );
      expect(
        core.calls.where((call) => call['method'] == 'execute'),
        hasLength(1),
      );
      await app.currentState!.go('D01');
      await tester.pumpAndSettle();
      expect(
        core.calls.where((call) => call['method'] == 'execute'),
        hasLength(1),
      );
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    });
  }
}
