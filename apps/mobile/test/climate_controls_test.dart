import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/climate_controls.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/device_typography.dart';
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
    'refresh reads without executing and rejects background callbacks',
    (tester) async {
      final (core, c, _) = await open(tester);
      core.calls.clear();
      final refresh = find.byKey(const ValueKey('climate-refresh'));
      await tap(tester, refresh);
      expect(
        core.calls.where((v) => v['method'] == 'read_state'),
        hasLength(1),
      );
      expect(core.calls.where((v) => v['method'] == 'execute'), isEmpty);
      final callback = tester.widget<ActionButton>(refresh).onPressed!;
      core.calls.clear();
      c.setForeground(false);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      await tester.pumpAndSettle();
      expect(tester.widget<ActionButton>(refresh).onPressed, isNull);
      callback();
      await tester.pumpAndSettle();
      expect(core.calls, isEmpty);
      c.setForeground(true);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpAndSettle();
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets('power is a switch and selected mode has a checkmark', (
    tester,
  ) async {
    final (core, _, _) = await open(tester);
    final power = tester.widget<Switch>(
      find.byKey(const ValueKey('climate-power')),
    );
    expect(power.value, true);
    final mode = tester.widget<ChoiceChip>(
      find.byKey(const ValueKey('climate-option:mode:cool')),
    );
    expect(mode.selected, true);
    expect(mode.showCheckmark, false);
    expect(
      find.descendant(
        of: find.byKey(const ValueKey('climate-option:mode:cool')),
        matching: find.byIcon(Icons.check),
      ),
      findsOneWidget,
    );
    expect(mode.labelStyle!.fontSize, DeviceTypography.body);
    expect(mode.labelStyle!.height, DeviceTypography.lineHeight);
    core.calls.clear();
    await tap(tester, find.byKey(const ValueKey('climate-option:mode:cool')));
    expect(core.calls.where((v) => v['method'] == 'execute'), isEmpty);
    await tester.pumpWidget(const SizedBox());
  });

  for (final layout in [(430.0, 1.0), (320.0, 1.0), (320.0, 2.0)]) {
    testWidgets(
      'fan presets use a compact equal-width accessible grid at ${layout.$1}px and ${layout.$2}x text',
      (tester) async {
        LocaleSettings.setLocale(AppLocale.zhCn);
        final (core, c, _) = await open(tester);
        tester.view.physicalSize = Size(layout.$1 * 2, 1688);
        tester.platformDispatcher.textScaleFactorTestValue = layout.$2;
        addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
        await tester.pumpAndSettle();
        final fan = find.byKey(const ValueKey('climate-row:Fan speed'));
        final chips = find.descendant(
          of: fan,
          matching: find.byType(ChoiceChip),
        );
        expect(chips, findsNWidgets(6));
        final rows = <int>{};
        for (var i = 0; i < 6; i++) {
          rows.add(tester.getTopLeft(chips.at(i)).dy.round());
          expect(
            tester.widget<ChoiceChip>(chips.at(i)).labelStyle!.fontSize,
            DeviceTypography.body,
          );
          expect(tester.getSize(chips.at(i)).height, greaterThanOrEqualTo(48));
        }
        expect(rows, hasLength(layout.$2 == 1 ? 2 : 3));
        expect(find.byKey(const ValueKey('climate-fan-scroll')), findsNothing);
        final width = tester.getSize(chips.first).width;
        for (var i = 0; i < 6; i++) {
          expect(tester.getSize(chips.at(i)).width, closeTo(width, .01));
          final face = find.descendant(
            of: chips.at(i),
            matching: find.byType(Material),
          );
          expect(tester.getSize(face).width, closeTo(width, .01));
        }
        if (layout.$2 == 1) {
          Rect faceRect(int index) => tester.getRect(
            find.descendant(
              of: chips.at(index),
              matching: find.byType(Material),
            ),
          );
          expect(faceRect(3).top - faceRect(0).bottom, closeTo(4, .01));
        }
        final grid = tester.widget<Wrap>(
          find.descendant(
            of: find.byKey(const ValueKey('climate-options-grid:wind_speed')),
            matching: find.byType(Wrap),
          ),
        );
        expect(grid.runSpacing, 0);
        final modes = find.descendant(
          of: find.byKey(const ValueKey('climate-row:Mode')),
          matching: find.byType(ChoiceChip),
        );
        expect(modes, findsNWidgets(5));
        final modeRows = <int, int>{};
        final modeWidth = tester.getSize(modes.first).width;
        for (var i = 0; i < 5; i++) {
          final y = tester.getTopLeft(modes.at(i)).dy.round();
          modeRows[y] = (modeRows[y] ?? 0) + 1;
          expect(tester.getSize(modes.at(i)).width, closeTo(modeWidth, .01));
          expect(tester.getSize(modes.at(i)).height, greaterThanOrEqualTo(48));
        }
        if (layout.$2 == 1) expect(modeRows.values, [3, 2]);
        final choice = climateChoice(c, {'wind_speed'})!;
        core.calls.clear();
        for (final option in choice.available(c)) {
          core.observations[choice.property] = option;
          await c.refresh();
          await tester.pumpAndSettle();
          final selected = find.byKey(
            ValueKey('climate-option:${choice.parameter}:$option'),
          );
          expect(tester.widget<ChoiceChip>(selected).selected, true);
          final viewport = tester.getRect(fan);
          final bounds = tester.getRect(selected);
          expect(bounds.left, greaterThanOrEqualTo(viewport.left - 1));
          expect(bounds.right, lessThanOrEqualTo(viewport.right + 1));
        }
        expect(
          core.calls.where((call) => call['method'] == 'execute'),
          isEmpty,
        );
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
      },
    );
  }

  testWidgets(
    'power feedback stays local and observed until the post-command read',
    (tester) async {
      final (core, c, _) = await open(tester);
      final power = find.byKey(const ValueKey('climate-power'));
      expect(tester.widget<Switch>(power).value, true);
      expect(find.text('Powered on'), findsOneWidget);
      core.executionGate = Completer<void>();
      core.readGate = Completer<void>();
      core.calls.clear();
      await tester.tap(power);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));
      expect(c.observed('power.state'), true);
      expect(find.text('Turning off…'), findsOneWidget);
      expect(find.byType(OperationPulse), findsOneWidget);
      expect(find.byType(OperationProgress), findsNothing);
      expect(find.byType(LinearProgressIndicator), findsNothing);
      expect(tester.widget<Switch>(power).onChanged, isNull);
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
      expect(tester.widget<Switch>(power).value, false);
      expect(c.receipt!['status'], 'accepted');
      expect(find.text('Accepted'), findsOneWidget);
      expect(find.text('View Details'), findsOneWidget);
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

  testWidgets(
    'target temperature has one primary control with bounded half-degree steps',
    (tester) async {
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
      expect(
        find.byKey(const ValueKey('climate-temperature-slider')),
        findsNothing,
      );
      expect(find.text('Target temperature'), findsOneWidget);
      expect(find.byKey(const ValueKey('climate-more-settings')), findsNothing);
      expect(find.text('Other settings'), findsNothing);
      final increase = find.byKey(const ValueKey('climate-increase'));
      final decrease = find.byKey(const ValueKey('climate-decrease'));
      await tap(tester, increase);
      expect(app.currentState!.id, 'C01');
      expect(
        core.calls
            .where((v) => v['method'] == 'execute')
            .single['command']['arguments'],
        {'target_temperature': 24.5},
      );
      expect(c.observed('climate.target_temperature'), 24.0);
      expect(find.text('24°C'), findsOneWidget);
      for (final edge in [18.0, 27.0]) {
        core.observations['climate.target_temperature'] = edge;
        await c.refresh();
        await tester.pumpAndSettle();
        expect(
          tester.widget<IconButton>(edge == 18 ? decrease : increase).onPressed,
          isNull,
        );
      }
      core.observations['climate.target_temperature'] = 24.0;
      await c.refresh();
      await tester.pumpAndSettle();
      final oldCallback = tester.widget<IconButton>(increase).onPressed!;
      core.calls.clear();
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      oldCallback();
      await tester.pumpAndSettle();
      expect(core.calls, isEmpty);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      oldCallback();
      await tester.pumpAndSettle();
      expect(core.calls, isEmpty);
      climate['actions']['set']['parameters']['required'] = [
        'target_temperature',
        'mode',
      ];
      await c.refresh();
      await tester.pumpAndSettle();
      expect(find.byType(Slider), findsNothing);
      expect(increase, findsNothing);
      expect(decrease, findsNothing);
      climate['actions']['set']['parameters']['required'] = [
        'target_temperature',
      ];
      climate['actions']['set']['parameters']['properties']['target_temperature'] =
          {
            'type': 'number',
            'enum': [23.0, 24.0, 25.0],
          };
      await c.refresh();
      await tester.pumpAndSettle();
      expect(find.text('Target temperature'), findsOneWidget);
      expect(increase, findsNothing);
      expect(
        find.byKey(const ValueKey('climate-option:target_temperature:25.0')),
        findsOneWidget,
      );

      await tester.pumpWidget(const SizedBox());
    },
  );

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
