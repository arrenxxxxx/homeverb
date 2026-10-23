import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/appliance_panels.dart';
import 'package:homeverb_mobile/compartment_controls.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/localization.dart';
import 'component_test.dart' show CompartmentFixture, example;

class ModeFixture extends CompartmentFixture {
  bool omitModeReadings = false;
  ModeFixture() {
    capabilities.add(example('capability.meiju-refrigerator'));
    device['components'][0]['capabilities'].add('vendor.midea.refrigerator');
  }
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    final value = await super.call(method, fields);
    if (method == 'read_state' &&
        fields['component'] == 'fridge' &&
        !omitModeReadings) {
      value['observations'].addAll({
        'vendor.midea.refrigerator.storage_mode': {
          'value': true,
          'stale': true,
        },
        'vendor.midea.refrigerator.freezing_mode': {
          'value': false,
          'stale': true,
        },
        'vendor.midea.refrigerator.left_flexzone_temperature': {
          'value': -2,
          'stale': true,
        },
      });
    }
    return value;
  }
}

class ReferenceFridgeFixture extends ModeFixture {
  ReferenceFridgeFixture() {
    device['components'].add({
      'id': 'variable',
      'capabilities': ['climate'],
    });
    final climate = example('capability.fridge');
    climate['component_id'] = 'variable';
    climate['actions'] = <String, dynamic>{};
    for (final property in (climate['properties'] as Json).values) {
      property['writable'] = false;
    }
    capabilities.add(climate);
    targets['variable'] = 6;
    device['components'][0]['capabilities'].add(
      'vendor.midea.refrigerator.details',
    );
    capabilities.add({
      'schema_version': '1.0',
      'id': 'vendor.midea.refrigerator.details',
      'component_id': 'fridge',
      'revision': 1,
      'properties': {
        'deep_cold_mode': {
          'type': 'boolean',
          'readable': true,
          'writable': false,
        },
        'e_cleaning_percentage': {
          'type': 'number',
          'unit': 'percent',
          'readable': true,
          'writable': false,
        },
        'storage_door_state': {
          'type': 'string',
          'readable': true,
          'writable': false,
        },
      },
      'actions': <String, dynamic>{},
      'events': <String, dynamic>{},
    });
  }
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    final result = await super.call(method, fields);
    if (method == 'read_state') {
      for (final observation in (result['observations'] as Json).values) {
        observation['stale'] = true;
      }
    }
    if (method == 'read_state' && fields['component'] == 'fridge') {
      result['observations'].addAll({
        'vendor.midea.refrigerator.details.deep_cold_mode': {
          'value': true,
          'stale': true,
        },
        'vendor.midea.refrigerator.details.e_cleaning_percentage': {
          'value': 35,
          'stale': true,
        },
        'vendor.midea.refrigerator.details.storage_door_state': {
          'value': 'closed',
          'stale': true,
        },
      });
    }
    return result;
  }
}

class GroupedFridgeFixture extends ReferenceFridgeFixture {
  GroupedFridgeFixture() {
    final properties = capabilities.last['properties'] as Json;
    for (final name in [
      'plasma_aseptic_mode_power',
      'electronic_smell',
      'rapid_ice_making',
      'open_door_tips_switch',
      'telstar_saving_mode',
    ]) {
      properties[name] = {
        'type': 'boolean',
        'readable': true,
        'writable': false,
      };
    }
    properties['future_status'] = {
      'type': 'string',
      'readable': true,
      'writable': false,
    };
    properties['hidden_status'] = {
      'type': 'string',
      'readable': false,
      'writable': false,
    };
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    final result = await super.call(method, fields);
    if (method == 'read_state' && fields['component'] == 'fridge') {
      for (final cap in capabilities.where(
        (cap) => (cap['id'] as String).startsWith('vendor.midea.refrigerator'),
      )) {
        for (final entry in (cap['properties'] as Json).entries) {
          result['observations']['${cap['id']}.${entry.key}'] = {
            'value': switch (entry.value['type']) {
              'boolean' => false,
              'number' => 0,
              _ => 'unrecognized-code',
            },
            'stale': true,
          };
        }
      }
      // Observation presence alone must not create a feature without a descriptor.
      result['observations']['vendor.midea.refrigerator.undeclared'] = {
        'value': true,
        'stale': true,
      };
    }
    return result;
  }
}

void main() {
  setUpAll(() async {
    const cjkFont = String.fromEnvironment('HOMEVERB_CJK_FONT');
    final font = FontLoader('Inter');
    if (cjkFont.isNotEmpty) {
      font.addFont(File(cjkFont).readAsBytes().then(ByteData.sublistView));
    } else {
      font
        ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
        ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf'));
    }
    await font.load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
  });

  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    for (final narrow in [false, true]) {
      testWidgets(
        'refrigerator groups preserve readings in $locale, narrow=$narrow',
        (tester) async {
          tester.view.physicalSize = Size(narrow ? 320 : 400, 900);
          tester.view.devicePixelRatio = 1;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          LocaleSettings.setLocale(locale);
          addTearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
          final core = GroupedFridgeFixture();
          final c = HomeController(
            core,
            LocalPreferences(File('/unused/grouped-fridge.json')),
          );
          addTearDown(c.dispose);
          await c.initialize();
          await c.selectDevice(core.device['id']);
          final calls = core.calls.length;
          final saved = jsonEncode(c.componentSnapshot('fridge'));
          final boundary = GlobalKey();
          await tester.pumpWidget(
            TranslationProvider(
              child: MaterialApp(
                home: MediaQuery(
                  data: MediaQueryData(
                    textScaler: TextScaler.linear(narrow ? 2 : 1),
                  ),
                  child: Scaffold(
                    body: SingleChildScrollView(
                      child: RepaintBoundary(
                        key: boundary,
                        child: ColoredBox(
                          color: const Color(0xfff5f5f2),
                          child: Padding(
                            padding: const EdgeInsets.all(20),
                            child: RefrigeratorModes(controller: c),
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          );
          await tester.pumpAndSettle();
          void inGroup(String group, String property) => expect(
            find.descendant(
              of: find.byKey(ValueKey('refrigerator-group:$group')),
              matching: find.byKey(ValueKey('refrigerator-reading:$property')),
            ),
            findsOneWidget,
          );
          inGroup('variable', 'vendor.midea.refrigerator.variable_mode');
          inGroup('smart', 'vendor.midea.refrigerator.intelligent_mode');
          inGroup(
            'smart',
            'vendor.midea.refrigerator.details.telstar_saving_mode',
          );
          inGroup('functions', 'vendor.midea.refrigerator.storage_mode');
          inGroup(
            'functions',
            'vendor.midea.refrigerator.details.e_cleaning_percentage',
          );
          inGroup(
            'status',
            'vendor.midea.refrigerator.details.storage_door_state',
          );
          inGroup(
            'settings',
            'vendor.midea.refrigerator.details.open_door_tips_switch',
          );
          inGroup('other', 'vendor.midea.refrigerator.details.future_status');
          expect(find.byType(ApplianceReading), findsNWidgets(24));
          expect(find.text('0%'), findsOneWidget);
          expect(find.text('unrecognized-code'), findsNWidgets(3));
          expect(find.byType(Switch), findsNothing);
          expect(find.text(locale.translations.mobileStale), findsNothing);
          expect(find.text('Modes and freshness'), findsNothing);
          final headings = find.byWidgetPredicate(
            (widget) => widget is Semantics && widget.properties.header == true,
          );
          expect(headings, findsNWidgets(6));
          for (final title in [
            locale.translations.mobileRefrigeratorVariableDetails,
            locale.translations.mobileRefrigeratorSmartSaving,
            locale.translations.mobileRefrigeratorFunctions,
            locale.translations.mobileRefrigeratorStatus,
            locale.translations.mobileClimateOtherSettings,
            locale.translations.mobileRefrigeratorOtherReadings,
          ]) {
            expect(find.text(title), findsOneWidget);
          }
          expect(core.calls.length, calls);
          expect(jsonEncode(c.componentSnapshot('fridge')), saved);
          expect(tester.takeException(), isNull);
          if (locale == AppLocale.zhCn && !narrow) {
            await tester.runAsync(() async {
              final image =
                  await (boundary.currentContext!.findRenderObject()
                          as RenderRepaintBoundary)
                      .toImage();
              final bytes = await image.toByteData(
                format: ui.ImageByteFormat.png,
              );
              final directory = Directory('build/verification/refrigerator')
                ..createSync(recursive: true);
              File(
                '${directory.path}/grouped-zh-CN.png',
              ).writeAsBytesSync(bytes!.buffer.asUint8List());
              image.dispose();
            });
          }
          await tester.pumpWidget(const SizedBox());
        },
      );
    }
  }

  testWidgets('unreported refrigerator groups remain absent', (tester) async {
    final core = ModeFixture()..omitModeReadings = true;
    final c = HomeController(
      core,
      LocalPreferences(File('/unused/empty-fridge.json')),
    );
    addTearDown(c.dispose);
    await c.initialize();
    await c.selectDevice(core.device['id']);
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: RefrigeratorModes(controller: c)),
      ),
    );
    expect(
      find.text('No mode readings have been returned yet.'),
      findsOneWidget,
    );
    expect(find.byType(ApplianceReading), findsNothing);
    expect(
      find.byWidgetPredicate(
        (widget) =>
            widget.key?.toString().contains('refrigerator-group:') == true,
      ),
      findsNothing,
    );
  });
  testWidgets(
    'reference refrigerator has a separate read-only third zone and detail readings',
    (tester) async {
      final core = ReferenceFridgeFixture();
      final c = HomeController(
        core,
        LocalPreferences(File('/unused/reference-fridge.json')),
      );
      await c.initialize();
      await c.selectDevice(core.device['id']);
      await tester.pumpWidget(
        TranslationProvider(
          child: MaterialApp(
            home: Scaffold(
              body: SingleChildScrollView(
                child: CompartmentControls(
                  controller: c,
                  onAction: (_) {},
                  onTarget: (_, __) {
                    throw StateError('No reference target write.');
                  },
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Variable zone'), findsOneWidget);
      expect(find.text('6°C'), findsOneWidget);
      expect(find.text('Deep cooling'), findsOneWidget);
      expect(find.text('35%'), findsOneWidget);
      expect(find.text('Fridge door state'), findsOneWidget);
      expect(find.textContaining('Stale'), findsNothing);
      expect(
        (c.componentSnapshot('fridge')!['observations'] as Json).values.every(
          (observation) => observation['stale'] == true,
        ),
        true,
      );
      expect(
        find.byKey(const ValueKey('edit-component:variable')),
        findsNothing,
      );
      expect(
        find.byKey(const ValueKey('component-stepper:variable')),
        findsNothing,
      );
      LocaleSettings.setLocale(AppLocale.zhCn);
      await tester.pumpAndSettle();
      expect(find.text('\u53d8\u6e29\u533a'), findsOneWidget);
      expect(find.text('\u6df1\u51b7'), findsOneWidget);
      expect(find.textContaining('\u5df2\u8fc7\u671f'), findsNothing);
      expect(
        core.calls.where((v) => ['plan', 'execute'].contains(v['method'])),
        isEmpty,
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
      LocaleSettings.setLocale(AppLocale.enUs);
    },
  );
  testWidgets(
    'reported refrigerator modes never invent missing features or writes',
    (tester) async {
      final core = ModeFixture();
      final c = HomeController(
        core,
        LocalPreferences(File('/unused/modes.json')),
      );
      await c.initialize();
      await c.selectDevice(core.device['id']);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: RefrigeratorModes(controller: c),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Rapid cooling'), findsOneWidget);
      expect(find.text('Rapid freezing'), findsOneWidget);
      expect(find.text('On'), findsOneWidget);
      expect(find.text('Off'), findsOneWidget);
      expect(find.text('-2°C'), findsOneWidget);
      expect(find.text('Stale'), findsNothing);
      expect(find.text('Holiday mode'), findsNothing);
      expect(
        find.byKey(const ValueKey('refrigerator-group:smart')),
        findsNothing,
      );
      expect(
        find.byKey(const ValueKey('refrigerator-group:status')),
        findsNothing,
      );
      expect(
        find.byKey(const ValueKey('refrigerator-group:settings')),
        findsNothing,
      );
      expect(
        core.calls.where(
          (call) => ['execute', 'plan'].contains(call['method']),
        ),
        isEmpty,
      );
      expect(tester.takeException(), isNull);
      c.dispose();
    },
  );
}
