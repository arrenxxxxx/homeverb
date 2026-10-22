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
import 'package:homeverb_mobile/skin.dart';
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
  Completer<void>? readGate;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'read_state') await readGate?.future;
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
    'power feedback stays local and observed until the post-command read',
    (tester) async {
      final (core, c, _) = await open(tester);
      final power = find.byKey(const ValueKey('climate-power'));
      Color? background() => tester
          .widget<FilledButton>(power)
          .style!
          .backgroundColor!
          .resolve({});
      expect(find.text('Powered on'), findsOneWidget);
      final onColor = background();
      core.executionGate = Completer<void>();
      core.readGate = Completer<void>();
      core.calls.clear();
      await tester.tap(power);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));
      expect(c.observed('power.state'), true);
      expect(find.text('Turning off…'), findsOneWidget);
      expect(
        find.descendant(of: power, matching: find.byType(OperationPulse)),
        findsOneWidget,
      );
      expect(find.byType(OperationProgress), findsNothing);
      expect(find.byType(LinearProgressIndicator), findsNothing);
      expect(tester.widget<FilledButton>(power).onPressed, isNull);
      expect(
        tester
            .widget<ChoiceChip>(
              find.byKey(const ValueKey('climate-option:mode:dry')),
            )
            .onSelected,
        isNull,
      );
      core.executionGate!.complete();
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 100));
      expect(c.observed('power.state'), true);
      expect(find.text('Turning off…'), findsOneWidget);
      core.readGate!.complete();
      await tester.pumpAndSettle();
      expect(find.byType(OperationPulse), findsNothing);
      expect(find.text('Powered off'), findsOneWidget);
      expect(background(), isNot(onColor));
      expect(c.receipt!['status'], 'accepted');
      expect(find.text('Accepted'), findsNothing);
      expect(find.text('View Details'), findsNothing);
      expect(core.calls.where((v) => v['method'] == 'execute').length, 1);
      expect(core.calls.where((v) => v['method'] == 'read_state').length, 1);
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets('mode activity marks only its row without changing selection', (
    tester,
  ) async {
    final (core, c, _) = await open(tester);
    core.executionGate = Completer<void>();
    await tester.tap(find.byKey(const ValueKey('climate-option:mode:dry')));
    await tester.pump();
    expect(
      find.descendant(
        of: find.byKey(const ValueKey('climate-row:Mode')),
        matching: find.byType(OperationPulse),
      ),
      findsOneWidget,
    );
    expect(
      find.descendant(
        of: find.byKey(const ValueKey('climate-row:Fan speed')),
        matching: find.byType(OperationPulse),
      ),
      findsNothing,
    );
    expect(c.observed('vendor.midea.ac.mode'), 'cool');
    expect(
      tester
          .widget<ChoiceChip>(
            find.byKey(const ValueKey('climate-option:mode:cool')),
          )
          .selected,
      true,
    );
    core.executionGate!.complete();
    await tester.pumpAndSettle();
    expect(find.byType(OperationPulse), findsNothing);
    await tester.pumpWidget(const SizedBox());
  });

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
    'expanded rows execute exact mode and fan choices without extra navigation',
    (tester) async {
      final (core, c, app) = await open(tester);
      expect(
        find.byKey(const ValueKey('climate-row:Air direction')),
        findsNothing,
      );
      expect(find.text('Not connected in HomeVerb'), findsNothing);
      core.observations['vendor.midea.ac.wind_speed'] = 101;
      await c.refresh();
      await tester.pumpAndSettle();
      expect(find.textContaining('Unrecognized (101)'), findsOneWidget);
      core.observations['vendor.midea.ac.wind_speed'] = 40;
      core.calls.clear();
      expect(
        find.text('Selecting a mode also turns this device on.'),
        findsOneWidget,
      );
      await tap(tester, find.byKey(const ValueKey('climate-option:mode:dry')));
      expect(app.currentState!.id, 'C01');
      expect(
        core.calls.firstWhere(
          (v) => v['method'] == 'execute',
        )['command']['arguments'],
        {'mode': 'dry'},
      );
      core.calls.clear();
      await tap(
        tester,
        find.byKey(const ValueKey('climate-option:wind_speed:80')),
      );
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

  testWidgets('background invalidates an old inline callback without sending', (
    tester,
  ) async {
    final (core, _, app) = await open(tester);
    core.calls.clear();
    final option = tester.widget<ChoiceChip>(
      find.byKey(const ValueKey('climate-option:mode:heat')),
    );
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    await tester.pumpAndSettle();
    option.onSelected!(true);
    await tester.pumpAndSettle();
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pumpAndSettle();
    expect(find.text('Apply'), findsNothing);
    expect(core.calls, isEmpty);
    expect(app.currentState!.id, 'C01');
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets(
    'exact reference settings stay independent and use observed values',
    (tester) async {
      final (core, c, app) = await open(tester);
      final cap = core.capabilities.last;
      for (final name in [
        'cool_energy_saving',
        'strong_cooling',
        'light_sensing',
      ]) {
        cap['properties'][name] = {
          'type': 'boolean',
          'readable': true,
          'writable': true,
          'control_requires_observation': true,
        };
        cap['actions']['set_$name'] = setter(
          {
            name: {'type': 'boolean'},
          },
          required: [name],
        );
        core.observations['vendor.midea.ac.$name'] = false;
      }
      await c.refresh();
      await tester.pumpAndSettle();
      expect(find.text('Cool energy saving'), findsOneWidget);
      expect(find.text('Strong cooling'), findsOneWidget);
      expect(find.text('Light sensing'), findsOneWidget);
      for (final name in [
        'cool_energy_saving',
        'strong_cooling',
        'light_sensing',
      ]) {
        core.calls.clear();
        await tap(tester, find.byKey(ValueKey('climate-switch:$name')));
        final command = core.calls
            .where((v) => v['method'] == 'execute')
            .single['command'];
        expect(command['capability'], 'vendor.midea.ac');
        expect(command['action'], 'set_$name');
        expect(command['arguments'], {name: true});
        expect(c.observed('vendor.midea.ac.$name'), false);
        expect(app.currentState!.id, 'C01');
      }
      LocaleSettings.setLocale(AppLocale.zhCn);
      await tester.pumpAndSettle();
      expect(find.text('\u9177\u7701\u7535'), findsOneWidget);
      expect(find.text('\u5f3a\u51b7'), findsOneWidget);
      expect(find.text('\u667a\u80fd\u5149\u611f'), findsOneWidget);
      final old = tester
          .widget<Switch>(
            find.byKey(const ValueKey('climate-switch:light_sensing')),
          )
          .onChanged!;
      core.calls.clear();
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      await tester.pumpAndSettle();
      old(true);
      await tester.pumpAndSettle();
      expect(core.calls.where((v) => v['method'] == 'execute'), isEmpty);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets('temperature slider sends once on release within device bounds', (
    tester,
  ) async {
    final (core, c, app) = await open(tester);
    final climate = core.capabilities.first;
    climate['properties']['target_temperature'] = {
      'type': 'number',
      'readable': true,
      'writable': true,
    };
    climate['actions']['set'] = setter(
      {
        'target_temperature': {
          'type': 'number',
          'minimum': 18,
          'maximum': 27,
          'multipleOf': .5,
        },
      },
      required: ['target_temperature'],
    );
    core.observations['climate.target_temperature'] = 24.0;
    await c.refresh();
    await tester.pumpAndSettle();
    core.calls.clear();
    var slider = tester.widget<Slider>(
      find.byKey(const ValueKey('climate-temperature-slider')),
    );
    expect((slider.min, slider.max, slider.divisions), (18.0, 27.0, 18));
    slider.onChangeStart!(24);
    slider.onChanged!(25.5);
    await tester.pump();
    expect(core.calls, isEmpty);
    slider.onChangeEnd!(25.5);
    await tester.pumpAndSettle();
    expect(app.currentState!.id, 'C01');
    expect(
      core.calls
          .where((v) => v['method'] == 'execute')
          .single['command']['arguments'],
      {'target_temperature': 25.5},
    );
    expect(core.calls.where((v) => v['method'] == 'read_state').length, 1);
    expect(c.observed('climate.target_temperature'), 24.0);
    slider = tester.widget<Slider>(
      find.byKey(const ValueKey('climate-temperature-slider')),
    );
    expect(
      slider.value,
      24.0,
      reason: 'A requested target is not an observed state.',
    );
    core.calls.clear();
    slider.onChangeStart!(24);
    slider.onChangeEnd!(27.5);
    slider.onChangeStart!(24);
    slider.onChangeEnd!(25.25);
    await tester.pumpAndSettle();
    expect(core.calls, isEmpty);
    slider.onChangeStart!(24);
    slider.onChanged!(26);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    await tester.pumpAndSettle();
    slider.onChangeEnd!(26);
    await tester.pumpAndSettle();
    expect(core.calls, isEmpty);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pumpAndSettle();
    slider.onChangeEnd!(26);
    await tester.pumpAndSettle();
    expect(core.calls, isEmpty);
    expect(
      tester
          .widget<Slider>(
            find.byKey(const ValueKey('climate-temperature-slider')),
          )
          .value,
      24,
    );
    climate['actions']['set']['parameters']['required'] = [
      'target_temperature',
      'mode',
    ];
    await c.refresh();
    await tester.pumpAndSettle();
    expect(find.byType(Slider), findsNothing);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('additional functions require actual independent capabilities', (
    tester,
  ) async {
    final (core, c, app) = await open(tester);
    expect(find.byKey(const ValueKey('climate-row:Sleep')), findsNothing);
    final vendor = core.capabilities.last;
    vendor['properties']['sleep'] = {
      'type': 'boolean',
      'readable': true,
      'writable': true,
    };
    vendor['actions']['set_sleep'] = setter(
      {
        'sleep': {'type': 'boolean'},
      },
      required: ['sleep'],
    );
    core.observations['${vendor['id']}.sleep'] = false;
    await c.refresh();
    await tester.pumpAndSettle();
    core.calls.clear();
    await tap(tester, find.byKey(const ValueKey('climate-switch:sleep')));
    expect(
      core.calls
          .where((v) => v['method'] == 'execute')
          .single['command']['arguments'],
      {'sleep': true},
    );
    expect(app.currentState!.id, 'C01');
    expect(find.text('Function support'), findsNothing);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
  });

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
        findsNothing,
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
