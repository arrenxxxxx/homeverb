import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/controls.dart';
import 'package:homeverb_mobile/device_detail_controls.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/session_permissions.dart';
import 'package:homeverb_mobile/skin.dart';
import 'accounts_test.dart' show AccountPreferences;
import 'session_permissions_test.dart' show inspection;
import 'mijia_controls_test.dart' show FanPermissionCore;

Json settingsInspection(bool light) {
  final result = inspection();
  final schema = jsonDecode(
    File('../../schemas/native-open.schema.json').readAsStringSync(),
  );
  result['options'] = [
    for (final rule
        in schema['properties']['mijia_setting_grants']['items']['oneOf'])
      if ((rule['properties']['component_id']['const'] == 'light-2') == light &&
          !{
            'light-6',
            'function-3',
            'hood-2',
            'fan_control-3',
            'screen-13',
          }.contains(rule['properties']['component_id']['const']))
        (() {
          final component =
                  rule['properties']['component_id']['const'] as String,
              property = rule['properties']['property']['const'] as String;
          return {
            'component_id': component,
            'capability': property == 'state'
                ? 'power'
                : 'vendor.xiaomi.mijia.${component.split('-').first}',
            'action': property == 'state' ? 'set' : 'set_$property',
            'definition': {
              'parameters': {
                'type': 'object',
                'properties': {property: rule['properties']['value']},
                'required': [property],
                'additionalProperties': false,
              },
              'transports': ['cloud'],
              'risk': 'low',
              'idempotency': 'set_value',
              'atomic': false,
              'independent_fields': false,
              'verification': 'unavailable',
            },
          };
        })(),
  ];
  return result;
}

class SettingCore extends FanPermissionCore {
  SettingCore(super.inspected) {
    cards.single['device']['alias'] = 'Fictional reference appliance';
    final components = <String, Set<String>>{};
    for (final o in inspected['options']) {
      components.putIfAbsent(o['component_id'], () => {}).add(o['capability']);
    }
    cards.single['device']['components'] = [
      for (final e in components.entries)
        {'id': e.key, 'capabilities': e.value.toList()},
    ];
  }
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    final result = await super.call(method, fields);
    if (method == 'describe_device') {
      for (final cap in result['capabilities']) {
        cap['component_id'] = (inspected['options'] as List).firstWhere(
          (o) => o['capability'] == cap['id'],
        )['component_id'];
      }
    }
    return result;
  }
}

class OldSettingCore extends SettingCore implements NativeSessionClient {
  OldSettingCore(super.inspected);
  @override
  Set<String> get methods => {'inspect_control_options'};
  @override
  bool apiAtLeast(int minor) => minor <= 29;
}

void main() {
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  test(
    'full exact domains retain all 3801 color temperatures and separate startup grants',
    () {
      for (final light in [true, false]) {
        final choices = SessionPermission.choices(settingsInspection(light));
        expect(choices.length, light ? 3930 : 31);
        expect(choices.every((p) => p.requiresReferenceSettingApi), true);
        expect(SessionPermission.startup(choices).keys, [
          'mijia_setting_grants',
        ]);
        if (light)
          expect(
            choices
                .where((p) => p.parameter == 'color_temperature_3')
                .map((p) => p.value)
                .toList(),
            List.generate(3801, (i) => 2700 + i),
          );
      }
      final bad = settingsInspection(true);
      bad['options'][0]['component_id'] = 'switch-2';
      expect(
        SessionPermission.choices(bad).where((p) => p.parameter == 'state'),
        isEmpty,
      );
    },
  );
  test('API 1.29 refuses new settings before reopening', () async {
    final data = settingsInspection(true),
        core = OldSettingCore(settingsInspection(true));
    var reopened = false;
    final c = HomeController(
      core,
      AccountPreferences(),
      reopen: (_) async {
        reopened = true;
        return core;
      },
    );
    expect(await c.initialize(), true);
    expect(await c.selectDevice(data['device_id']), false);
    expect(
      c.error,
      isA<NativeFailure>().having(
        (e) => e.code,
        'code',
        'API_UPGRADE_REQUIRED',
      ),
    );
    expect(reopened, false);
    expect(
      core.requests.where(
        (r) => ['execute', 'refresh_account'].contains(r['method']),
      ),
      isEmpty,
    );
    c.dispose();
  });
  test(
    'full domains open both native modes within startup buffer bounds',
    () async {
      final choices = [
        ...SessionPermission.choices(settingsInspection(true)),
        ...SessionPermission.choices(settingsInspection(false)),
      ];
      expect(
        utf8.encode(jsonEncode(SessionPermission.startup(choices))).length,
        lessThan(1048000),
      );
      for (final adapter in ['xiaomi.mijia', 'homeverb.platforms']) {
        final directory = await Directory.systemTemp.createTemp(
          'homeverb-settings-',
        );
        NativeClient? client;
        try {
          client = await NativeClient.open(
            dataDir: directory.path,
            adapter: adapter,
            libraryPath: '../../target/debug/${NativeClient.defaultLibrary()}',
            permissions: choices,
          );
          expect(client.apiAtLeast(30), true);
          expect(await client.call('list_devices'), isEmpty);
        } finally {
          await client?.close();
          await directory.delete(recursive: true);
        }
      }
    },
  );
  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    for (final light in [true, false]) {
      testWidgets(
        'reference controls fit 320px double text with no send: $light $locale',
        (tester) async {
          LocaleSettings.setLocale(locale);
          tester.view.physicalSize = const Size(640, 1600);
          tester.view.devicePixelRatio = 2;
          tester.platformDispatcher.textScaleFactorTestValue = 2;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
          final data = settingsInspection(light),
              next = SettingCore(settingsInspection(light));
          final c = HomeController(
            SettingCore(data),
            AccountPreferences(),
            reopen: (_) async => next,
          );
          expect(await c.initialize(), true);
          expect(await c.selectDevice(data['device_id']), true);
          String? selected;
          await tester.pumpWidget(
            TranslationProvider(
              child: MaterialApp(
                theme: windowTheme(),
                home: Scaffold(
                  body: SingleChildScrollView(
                    child: DeviceDetailControls(
                      controller: c,
                      onAction: (a) => selected = a,
                    ),
                  ),
                ),
              ),
            ),
          );
          await tester.pumpAndSettle();
          final action = light ? 'set_color_temperature_3' : 'set_fan_level_2';
          final button = find.byKey(ValueKey('control:1:$action'));
          await tester.ensureVisible(button);
          await tester.tap(button);
          await tester.pumpAndSettle();
          expect(selected, 'control:1:$action');
          expect(
            next.requests.where(
              (r) => ['plan', 'execute'].contains(r['method']),
            ),
            isEmpty,
          );
          expect(tester.takeException(), isNull);
          await tester.pumpWidget(const SizedBox());
          c.dispose();
        },
      );
    }
    testWidgets('color temperature form retains 1 K input: $locale', (
      tester,
    ) async {
      LocaleSettings.setLocale(locale);
      final o = (settingsInspection(true)['options'] as List).firstWhere(
        (o) => o['action'] == 'set_color_temperature_3',
      );
      Json? submitted;
      await tester.pumpWidget(
        TranslationProvider(
          child: MaterialApp(
            theme: windowTheme(),
            home: Scaffold(
              body: SingleChildScrollView(
                child: CapabilityActionForm(
                  capability: {
                    'id': o['capability'],
                    'properties': {},
                    'actions': {o['action']: o['definition']},
                  },
                  action: o['action'],
                  observation: (_) => null,
                  onSubmit: (arguments) async {
                    submitted = arguments;
                    return true;
                  },
                  onCancel: () {},
                  error: () => null,
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final field = find.byKey(const ValueKey('parameter:color_temperature_3'));
      final apply = find.byWidgetPredicate(
        (w) => w is ActionButton && w.label == 'Apply',
      );
      await tester.enterText(field, '6501');
      await tester.tap(apply);
      await tester.pumpAndSettle();
      expect(submitted, isNull);
      await tester.enterText(field, '4321');
      await tester.tap(apply);
      await tester.pumpAndSettle();
      expect(submitted, {'color_temperature_3': 4321});
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    });
  }
}
