import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/climate_controls.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'climate_controls_test.dart' show ClimateFixture;
import 'controls_test.dart' show setter;

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async {
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
  });
  setUp(() => LocaleSettings.setLocale(AppLocale.enUs));
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));

  Future<(ClimateFixture, HomeController)> open(WidgetTester tester) async {
    tester.view.physicalSize = const Size(780, 1688);
    tester.view.devicePixelRatio = 2;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final core = ClimateFixture();
    final c = HomeController(core, LocalPreferences(File('/unused/rows.json')));
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    final key = GlobalKey<NativeAppState>();
    await tester.pumpWidget(
      RepaintBoundary(
        key: const ValueKey('climate-test-capture'),
        child: TranslationProvider(
          child: NativeApp(key: key, catalog: catalog, controller: c),
        ),
      ),
    );
    await tester.pumpAndSettle();
    await c.selectDevice(core.device['id']);
    await key.currentState!.go('C01');
    await tester.pumpAndSettle();
    return (core, c);
  }

  void field(ClimateFixture core, String name, Json schema, Object? value) {
    final vendor = core.capabilities.last;
    vendor['properties'][name] = {
      ...schema,
      'readable': true,
      'writable': true,
    };
    vendor['actions']['set_$name'] = setter({name: schema}, required: [name]);
    if (value != null) core.observations['${vendor['id']}.$name'] = value;
  }

  Future<void> tap(WidgetTester tester, String key) async {
    final target = find.byKey(ValueKey(key));
    await tester.ensureVisible(target);
    await tester.tap(target);
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
  }

  testWidgets(
    'reported optional ECO and breezeless stay separate and hide absent fields',
    (tester) async {
      final (core, c) = await open(tester);
      field(core, 'eco', {'type': 'boolean'}, null);
      field(core, 'breezeless', {
        'type': 'integer',
        'enum': [0, 1, 2, 3],
      }, null);
      for (final name in ['eco', 'breezeless']) {
        core
                .capabilities
                .last['properties'][name]['control_requires_observation'] =
            true;
      }
      await c.refresh();
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('climate-switch:eco')), findsNothing);
      expect(
        find.byKey(const ValueKey('climate-option:breezeless:2')),
        findsNothing,
      );
      expect(find.text('Energy saving'), findsNothing);
      core.observations['vendor.midea.ac.eco'] = false;
      core.observations['vendor.midea.ac.breezeless'] = 0;
      await c.refresh();
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('climate-switch:eco')), findsOneWidget);
      expect(
        find.byKey(const ValueKey('climate-option:breezeless:2')),
        findsOneWidget,
      );
      expect(find.textContaining('Observed:'), findsNothing);
      core.calls.clear();
      await tap(tester, 'climate-switch:eco');
      expect(
        core.calls
            .where((v) => v['method'] == 'execute')
            .single['command']['arguments'],
        {'eco': true},
      );
      expect(c.observed('vendor.midea.ac.eco'), false);
      core.calls.clear();
      await tap(tester, 'climate-option:breezeless:2');
      expect(
        core.calls
            .where((v) => v['method'] == 'execute')
            .single['command']['arguments'],
        {'breezeless': 2},
      );
      expect(c.observed('vendor.midea.ac.breezeless'), 0);
      LocaleSettings.setLocale(AppLocale.zhCn);
      await tester.pumpAndSettle();
      final label =
          tester
                  .widget<ChoiceChip>(
                    find.byKey(const ValueKey('climate-option:breezeless:2')),
                  )
                  .label
              as Text;
      final chinese = jsonDecode(
        File(
          '../../packages/presentation/locales/zh-CN.json',
        ).readAsStringSync(),
      );
      expect(label.data, chinese['vendor.midea.ac.breezeless.2']);
      core.observations.remove('vendor.midea.ac.breezeless');
      await c.refresh();
      await tester.pumpAndSettle();
      expect(
        find.byKey(const ValueKey('climate-option:breezeless:2')),
        findsNothing,
      );
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets(
    'airflow axes degrade independently to switches or exact positions',
    (tester) async {
      final (core, c) = await open(tester);
      expect(
        find.byKey(const ValueKey('climate-row:Horizontal swing')),
        findsNothing,
      );
      expect(
        find.byKey(const ValueKey('climate-row:Vertical swing')),
        findsNothing,
      );
      field(core, 'horizontal_swing', {'type': 'boolean'}, false);
      field(core, 'vertical_direction', {
        'type': 'string',
        'enum': ['upper', 'lower'],
      }, 'upper');
      await c.refresh();
      await tester.pumpAndSettle();
      expect(
        find.byKey(const ValueKey('climate-switch:horizontal_swing')),
        findsOneWidget,
      );
      expect(
        find.byKey(const ValueKey('climate-row:Horizontal direction')),
        findsNothing,
      );
      expect(
        find.byKey(const ValueKey('climate-switch:vertical_swing')),
        findsNothing,
      );
      expect(
        find.byKey(const ValueKey('climate-option:vertical_direction:middle')),
        findsNothing,
      );
      core.calls.clear();
      await tap(tester, 'climate-switch:horizontal_swing');
      expect(
        core.calls
            .where((v) => v['method'] == 'execute')
            .single['command']['arguments'],
        {'horizontal_swing': true},
      );
      expect(c.observed('vendor.midea.ac.horizontal_swing'), false);
      core.calls.clear();
      await tap(tester, 'climate-option:vertical_direction:lower');
      expect(
        core.calls
            .where((v) => v['method'] == 'execute')
            .single['command']['arguments'],
        {'vertical_direction': 'lower'},
      );
      expect(core.calls.where((v) => v['method'] == 'read_state').length, 1);
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets('numeric speed uses bounded steps and enums remain presets', (
    tester,
  ) async {
    final (core, c) = await open(tester);
    expect(
      find.byKey(const ValueKey('climate-stepper:wind_speed')),
      findsNothing,
    );
    core.capabilities.last['actions'].remove('set_fan_speed');
    field(core, 'fan_speed', {
      'type': 'integer',
      'minimum': 2,
      'maximum': 8,
      'multipleOf': 2,
    }, 6);
    await c.refresh();
    await tester.pumpAndSettle();
    core.calls.clear();
    await tap(tester, 'climate-increase:fan_speed');
    expect(
      core.calls
          .where((v) => v['method'] == 'execute')
          .single['command']['arguments'],
      {'fan_speed': 8},
    );
    core.observations['vendor.midea.ac.fan_speed'] = 8;
    await c.refresh();
    await tester.pumpAndSettle();
    final plus = tester.widget<IconButton>(
      find.byKey(const ValueKey('climate-increase:fan_speed')),
    );
    expect(plus.onPressed, isNull);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets(
    'settings require independent writable actions and never invent unknown false',
    (tester) async {
      final (core, c) = await open(tester);
      field(core, 'display', {'type': 'boolean'}, false);
      field(core, 'button_sound', {'type': 'boolean'}, null);
      field(core, 'sleep', {'type': 'boolean'}, false);
      core.capabilities.last['properties']['sleep']['writable'] = false;
      await c.refresh();
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('climate-row:Sleep')), findsNothing);
      expect(
        find.byKey(const ValueKey('climate-switch:button_sound')),
        findsNothing,
      );
      expect(
        find.byKey(const ValueKey('climate-option:button_sound:true')),
        findsOneWidget,
      );
      core.calls.clear();
      core.denyPlan = true;
      await tap(tester, 'climate-switch:display');
      expect(core.calls.where((v) => v['method'] == 'plan').length, 1);
      expect(core.calls.where((v) => v['method'] == 'execute'), isEmpty);
      expect(c.observed('vendor.midea.ac.display'), false);
      core.denyPlan = false;
      core.calls.clear();
      await tap(tester, 'climate-option:button_sound:true');
      expect(
        core.calls
            .where((v) => v['method'] == 'execute')
            .single['command']['arguments'],
        {'button_sound': true},
      );
      expect(c.observed('vendor.midea.ac.button_sound'), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets(
    'coupled, ambiguous and conditional actions never become quick controls',
    (tester) async {
      final (core, c) = await open(tester);
      field(core, 'display', {'type': 'boolean'}, true);
      core
          .capabilities
          .last['actions']['set_display']['parameters']['dependentRequired'] = {
        'display': ['mode'],
      };
      field(core, 'sleep', {'type': 'boolean'}, false);
      core.capabilities.last['actions']['sleep_again'] = setter(
        {
          'sleep': {'type': 'boolean'},
        },
        required: ['sleep'],
      );
      field(core, 'vertical_swing', {'type': 'boolean'}, false);
      core
          .capabilities
          .last['actions']['set_vertical_swing']['parameters']['required'] = [
        'vertical_swing',
        'mode',
      ];
      await c.refresh();
      await tester.pumpAndSettle();
      expect(climateChoice(c, {'display'}), isNull);
      expect(climateChoice(c, {'sleep'}), isNull);
      expect(climateChoice(c, {'vertical_swing'}), isNull);
      expect(
        find.byKey(const ValueKey('climate-switch:display')),
        findsNothing,
      );
      await tester.pumpWidget(const SizedBox());
    },
  );

  test(
    'saved native actions do not imply permission after reopening read-only',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'homeverb-climate-policy-',
      );
      NativeClient? client;
      try {
        final library = File(
          '../../target/debug/' + NativeClient.defaultLibrary(),
        ).absolute.path;
        client = await NativeClient.open(
          dataDir: directory.path,
          libraryPath: library,
          allowMockWrites: true,
        );
        var c = HomeController(
          client,
          LocalPreferences(File(directory.path + '/ui.json')),
        );
        expect(await c.discover(), true);
        final device = c.cards.single['device']['id'] as String;
        expect(await c.selectDevice(device), true);
        expect(climateChoice(c, {'target_temperature'}), isNotNull);
        expect(
          climateChoice(c, {'state'}, capabilityIds: {'power'}),
          isNotNull,
        );
        await client.close();
        c.dispose();
        client = await NativeClient.open(
          dataDir: directory.path,
          libraryPath: library,
        );
        c = HomeController(
          client,
          LocalPreferences(File(directory.path + '/ui.json')),
        );
        expect(await c.selectDevice(device), true);
        expect(c.capability('climate')!['actions']['set'], isNotNull);
        expect(climateChoice(c, {'target_temperature'}), isNull);
        expect(climateChoice(c, {'mode'}), isNull);
        expect(climateChoice(c, {'state'}, capabilityIds: {'power'}), isNull);
        expect(c.hasControlAction('climate', 'set'), false);
        expect(await client.call('recent_executions'), isEmpty);
        c.dispose();
      } finally {
        await client?.close();
        await directory.delete(recursive: true);
      }
    },
  );

  testWidgets(
    'row order, large Chinese text and high contrast retain native controls',
    (tester) async {
      final (core, c) = await open(tester);
      core.capabilities.first['properties']['target_temperature'] = {
        'type': 'number',
        'readable': true,
        'writable': true,
      };
      core.capabilities.first['actions']['set'] = setter(
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
      core.observations['climate.target_temperature'] = 24;
      field(core, 'horizontal_swing', {'type': 'boolean'}, false);
      field(core, 'horizontal_direction', {
        'type': 'string',
        'enum': ['left', 'center', 'right'],
      }, 'center');
      field(core, 'vertical_swing', {'type': 'boolean'}, true);
      field(core, 'display', {'type': 'boolean'}, false);
      field(core, 'button_sound', {'type': 'boolean'}, false);
      field(core, 'sleep', {'type': 'boolean'}, false);
      await c.refresh();
      await tester.pumpAndSettle();
      final mode = find.byKey(const ValueKey('climate-row:Mode'));
      final fan = find.byKey(const ValueKey('climate-row:Fan speed'));
      final horizontal = find.byKey(
        const ValueKey('climate-row:Horizontal swing'),
      );
      final vertical = find.byKey(const ValueKey('climate-row:Vertical swing'));
      expect(tester.getTopLeft(mode).dy, lessThan(tester.getTopLeft(fan).dy));
      expect(
        tester.getTopLeft(fan).dy,
        lessThan(tester.getTopLeft(horizontal).dy),
      );
      expect(
        tester.getTopLeft(horizontal).dy,
        lessThan(tester.getTopLeft(vertical).dy),
      );
      Future<void> capture(String name) async {
        await tester.runAsync(() async {
          final boundary = tester.renderObject<RenderRepaintBoundary>(
            find.byKey(const ValueKey('climate-test-capture')),
          );
          final raster = await boundary.toImage(pixelRatio: 2);
          final bytes = await raster.toByteData(format: ui.ImageByteFormat.png);
          final output = File(
            'build/verification/climate-rows/' + name + '.png',
          );
          await output.parent.create(recursive: true);
          await output.writeAsBytes(bytes!.buffer.asUint8List());
          raster.dispose();
        });
      }

      await capture('main-controls');
      await tester.ensureVisible(
        find.byKey(const ValueKey('climate-switch:button_sound')),
      );
      await tester.pumpAndSettle();
      await capture('settings');
      await LocaleSettings.setLocale(AppLocale.zhCn);
      tester.view.physicalSize = const Size(640, 1688);
      tester.platformDispatcher.textScaleFactorTestValue = 2;
      addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      expect(
        find.text(AppLocale.zhCn.translations.mobileClimateButtonSound),
        findsOneWidget,
      );
      await tester.ensureVisible(
        find.byKey(const ValueKey('climate-switch:display')),
      );
      await tester.pumpAndSettle();
      await capture('zh-large-text');
      tester.platformDispatcher.accessibilityFeaturesTestValue =
          const FakeAccessibilityFeatures(highContrast: true);
      addTearDown(
        tester.platformDispatcher.clearAccessibilityFeaturesTestValue,
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      await capture('high-contrast');
      await tester.pumpWidget(const SizedBox());
    },
  );
}
