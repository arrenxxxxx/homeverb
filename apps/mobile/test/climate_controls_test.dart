import 'dart:async';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/climate_controls.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'controls_test.dart' show ControlsFixture, capability, setter, example;

class ClimateFixture extends ControlsFixture {
  ClimateFixture() {
    final vendor = example('capability.meiju-mode');
    final fan = example('capability.meiju-fan');
    vendor['properties']['wind_speed'] = fan['properties']['wind_speed'];
    vendor['actions'].addAll(fan['actions']);
    capabilities.clear();
    capabilities.addAll([
      capability('climate', {
        'current_temperature': {
          'type': 'number',
          'readable': true,
          'writable': false,
          'unit': 'celsius',
        },
      }, {}),
      capability(
        'power',
        {
          'state': {'type': 'boolean', 'readable': true, 'writable': true},
        },
        {
          'set': setter(
            {
              'state': {'type': 'boolean'},
            },
            required: ['state'],
          ),
        },
      ),
      vendor,
    ]);
    observations.clear();
    observations.addAll({
      'power.state': true,
      'climate.current_temperature': 26,
      'vendor.midea.ac.mode': 'cool',
      'vendor.midea.ac.wind_speed': 40,
    });
  }
  String outcome = 'accepted';
  bool failRead = false;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'read_state' && failRead) {
      calls.add({'method': method, ...fields});
      throw NativeFailure('STATE_UNAVAILABLE', 'Fixture read failure.');
    }
    final result = await super.call(method, fields);
    if (method == 'execute') {
      if (fields['command']['capability'] == 'power')
        observations['power.state'] = fields['command']['arguments']['state'];
      return {...result as Json, 'status': outcome};
    }
    return result;
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUp(() => LocaleSettings.setLocale(AppLocale.enUs));
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));

  Future<(ClimateFixture, HomeController, GlobalKey<NativeAppState>)> open(
    WidgetTester tester,
  ) async {
    tester.view.physicalSize = const Size(780, 1688);
    tester.view.devicePixelRatio = 2;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final core = ClimateFixture();
    final c = HomeController(
      core,
      LocalPreferences(File('/unused/climate.json')),
    );
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    final key = GlobalKey<NativeAppState>();
    await tester.pumpWidget(
      TranslationProvider(
        child: NativeApp(key: key, catalog: catalog, controller: c),
      ),
    );
    await tester.pumpAndSettle();
    await c.selectDevice(core.device['id']);
    await key.currentState!.go('C01');
    await tester.pumpAndSettle();
    return (core, c, key);
  }

  Future<void> tap(WidgetTester tester, Finder finder) async {
    await tester.ensureVisible(finder);
    await tester.tap(finder);
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
  }

  testWidgets(
    'power stays on device, refreshes once and retains unknown receipt',
    (tester) async {
      final (core, c, app) = await open(tester);
      core.outcome = 'unknown';
      core.calls.clear();
      await tap(tester, find.byKey(const ValueKey('climate-power')));
      expect(app.currentState!.id, 'C01');
      expect(core.calls.where((v) => v['method'] == 'execute').length, 1);
      expect(core.calls.where((v) => v['method'] == 'read_state').length, 1);
      expect(c.observed('power.state'), false);
      expect(c.receipt!['status'], 'unknown');
      expect(
        find.text(
          'This request may have executed. Refresh before considering another write.',
        ),
        findsOneWidget,
      );
      await tap(tester, find.text('View Details'));
      expect(app.currentState!.id, 'S06');
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets(
    'three buttons expose bounded mode and fan choices without extra navigation',
    (tester) async {
      final (core, c, app) = await open(tester);
      final direction = tester.widget<OutlinedButton>(
        find.byKey(const ValueKey('climate-button:Air direction')),
      );
      expect(direction.onPressed, isNull);
      expect(find.text('Not supported yet'), findsOneWidget);
      core.observations['vendor.midea.ac.wind_speed'] = 101;
      await c.refresh();
      await tester.pumpAndSettle();
      expect(find.text('Unrecognized (101)'), findsOneWidget);
      core.observations['vendor.midea.ac.wind_speed'] = 40;
      core.calls.clear();
      await tap(tester, find.byKey(const ValueKey('climate-button:Mode')));
      expect(
        find.text('Selecting a mode also turns this device on.'),
        findsOneWidget,
      );
      await tap(tester, find.byKey(const ValueKey('climate-option:mode:dry')));
      expect(core.calls, isEmpty);
      await tap(tester, find.text('Apply'));
      expect(app.currentState!.id, 'C01');
      expect(
        core.calls.firstWhere(
          (v) => v['method'] == 'execute',
        )['command']['arguments'],
        {'mode': 'dry'},
      );
      core.calls.clear();
      await tap(tester, find.byKey(const ValueKey('climate-button:Fan speed')));
      await tap(
        tester,
        find.byKey(const ValueKey('climate-option:wind_speed:80')),
      );
      await tap(tester, find.text('Apply'));
      expect(app.currentState!.id, 'C01');
      expect(
        core.calls.firstWhere(
          (v) => v['method'] == 'execute',
        )['command']['arguments'],
        {'wind_speed': 80},
      );
      expect(
        c.observed('vendor.midea.ac.wind_speed'),
        40,
        reason: 'Only a read changes observed values.',
      );
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets(
    'dismissal and background invalidate a selector without sending',
    (tester) async {
      final (core, _, app) = await open(tester);
      core.calls.clear();
      await tap(tester, find.byKey(const ValueKey('climate-button:Mode')));
      await tap(tester, find.byKey(const ValueKey('climate-option:mode:heat')));
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      await tester.pumpAndSettle();
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpAndSettle();
      expect(find.text('Apply'), findsNothing);
      expect(core.calls, isEmpty);
      expect(app.currentState!.id, 'C01');
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets(
    'late writes do not refresh after navigation and failed reads retain outcomes',
    (tester) async {
      final (core, c, app) = await open(tester);
      core.executionGate = Completer<void>();
      core.calls.clear();
      await tester.tap(find.byKey(const ValueKey('climate-power')));
      await tester.pump();
      await app.currentState!.go('D06');
      core.executionGate!.complete();
      await tester.pumpAndSettle();
      expect(app.currentState!.id, 'D06');
      expect(core.calls.where((v) => v['method'] == 'read_state'), isEmpty);
      await app.currentState!.go('C01');
      await tester.pumpAndSettle();
      core.failRead = true;
      core.outcome = 'unknown';
      await tap(tester, find.byKey(const ValueKey('climate-power')));
      expect(c.receipt!['status'], 'unknown');
      expect(c.error, isA<NativeFailure>());
      expect(app.currentState!.id, 'C01');
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets('background and resume do not start a late verification read', (
    tester,
  ) async {
    final (core, _, app) = await open(tester);
    core.executionGate = Completer<void>();
    core.calls.clear();
    await tester.tap(find.byKey(const ValueKey('climate-power')));
    await tester.pump();
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    core.executionGate!.complete();
    await tester.pumpAndSettle();
    expect(app.currentState!.id, 'C01');
    expect(core.calls.where((v) => v['method'] == 'execute').length, 1);
    expect(core.calls.where((v) => v['method'] == 'read_state'), isEmpty);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets(
    'Chinese buttons reflow at large text without inventing unsupported controls',
    (tester) async {
      final (core, c, _) = await open(tester);
      await LocaleSettings.setLocale(AppLocale.zhCn);
      tester.platformDispatcher.textScaleFactorTestValue = 2;
      addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
      await tester.pumpAndSettle();
      expect(
        find.text(AppLocale.zhCn.translations.mobileAirDirection),
        findsOneWidget,
      );
      expect(tester.takeException(), isNull);
      core.capabilities.last['actions']['activate_mode']['risk'] = 'elevated';
      expect(climateChoice(c, {'mode'}), isNull);
      core
          .capabilities
          .last['actions']['set_fan_speed']['parameters']['required']
          .add('mode');
      expect(climateChoice(c, {'wind_speed'}), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );
}
