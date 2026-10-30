import 'dart:convert';
import 'dart:ffi' hide Size;
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/controls.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/session_permissions.dart';
import 'package:homeverb_mobile/skin.dart';
import 'accounts_test.dart' show AccountPreferences;
import 'session_permissions_test.dart' show inspection;
import 'mijia_auxiliary_test.dart' show AuxiliaryCore;

const preferenceProperties = {
  'power_on_with_light_9',
  'power_off_with_light_10',
  'clean_remind_on_11',
  'clean_remind_time_12',
  'gestures_18',
};
Json hoodInspection({bool preferences = false}) {
  final result = inspection();
  result['device_id'] = '40000000-0000-4000-8000-000000000001';
  final rules = jsonDecode(
    File('../../schemas/native-open.schema.json').readAsStringSync(),
  )['properties']['mijia_setting_grants']['items']['oneOf'];
  result['options'] = [
    for (final rule in rules)
      if ({
            'hood-2',
            'fan_control-3',
            'screen-13',
          }.contains(rule['properties']['component_id']['const']) &&
          preferenceProperties.contains(
                rule['properties']['property']['const'],
              ) ==
              preferences)
        (() {
          final component =
              rule['properties']['component_id']['const'] as String;
          final property = rule['properties']['property']['const'] as String;
          return <String, dynamic>{
            'component_id': component,
            'capability': 'vendor.xiaomi.mijia.${component.split('-').first}',
            'action': 'set_$property',
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

void main() {
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  test(
    'all ten exact hood targets preserve vendor codes and separate authority',
    () {
      final choices = SessionPermission.choices(hoodInspection());
      final expected = jsonDecode(
        File('../../examples/native-open.mijia-hood.json').readAsStringSync(),
      );
      expect(choices, hasLength(10));
      expect(choices.every((p) => p.requiresHoodManualApi), isTrue);
      expect(
        choices.every(
          (p) => !p.requiresReferenceFanApi && !p.requiresOutletApi,
        ),
        isTrue,
      );
      expect(SessionPermission.startup(choices), {
        'mijia_setting_grants': expected['mijia_setting_grants'],
      });
      expect(
        choices.where((p) => p.parameter == 'fan_level_1').map((p) => p.value),
        [1, 2, 3, 4],
      );
      expect(choices.where((p) => p.parameter == 'on_1').map((p) => p.value), [
        false,
        true,
      ]);
    },
  );
  for (final fault in [
    'broadened levels',
    'partial levels',
    'fractional levels',
    'wrong component',
    'wrong capability',
    'schedule',
    'wrong type',
    'boolean enum',
    'extra schema',
    'risk',
  ]) {
    test('refuse altered hood options: $fault', () {
      final data = hoodInspection();
      final option = data['options'][fault == 'boolean enum' ? 0 : 1] as Json;
      data['options'] = [option];
      final properties =
          option['definition']['parameters']['properties'] as Json;
      final schema = properties.values.single as Json;
      switch (fault) {
        case 'broadened levels':
          schema['enum'] = [0, 1, 2, 3, 4];
        case 'partial levels':
          schema['enum'] = [1, 2, 3];
        case 'fractional levels':
          schema['enum'] = [1, 2, 3, 3.5];
        case 'wrong component':
          option['component_id'] = 'fan_control-4';
        case 'wrong capability':
          option['capability'] = 'vendor.xiaomi.mijia.fan';
        case 'schedule':
          option['action'] = 'set_auto_screen_off_time_5';
        case 'wrong type':
          schema['type'] = 'number';
        case 'boolean enum':
          schema['enum'] = [true];
        case 'extra schema':
          schema['minimum'] = 0;
        case 'risk':
          option['definition']['risk'] = 'elevated';
      }
      expect(SessionPermission.choices(data), isEmpty);
    });
  }
  test(
    'API 1.38 refuses hood permissions before reopen, refresh or write',
    () async {
      final core = AuxiliaryCore(hoodInspection(), minor: 38);
      var opened = false;
      final c = HomeController(
        core,
        AccountPreferences(),
        reopen: (_) async {
          opened = true;
          return core;
        },
      );
      expect(await c.initialize(), isTrue);
      expect(await c.selectDevice(hoodInspection()['device_id']), isFalse);
      expect(
        c.error,
        isA<NativeFailure>().having(
          (e) => e.code,
          'code',
          'API_UPGRADE_REQUIRED',
        ),
      );
      expect(opened, isFalse);
      expect(
        core.requests.where(
          (r) => ['execute', 'refresh_account'].contains(r['method']),
        ),
        isEmpty,
      );
      c.dispose();
    },
  );
  test(
    'actual API 1.38 peer receives only an unprivileged version probe',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'homeverb-hood-old-',
      );
      try {
        final path = '${directory.path}/${NativeClient.defaultLibrary()}';
        final built = await Process.run('cc', [
          Platform.isMacOS ? '-dynamiclib' : '-shared',
          '-fPIC',
          '-std=c11',
          '-D_DEFAULT_SOURCE',
          '-DFIXTURE_API_VERSION="1.38.0"',
          'test/fixtures/outlet_startup_bridge.c',
          '-o',
          path,
        ]);
        expect(built.exitCode, 0, reason: '${built.stderr}');
        final library = DynamicLibrary.open(path);
        await expectLater(
          NativeClient.open(
            dataDir: directory.path,
            adapter: 'xiaomi.mijia',
            libraryPath: path,
            permissions: SessionPermission.choices(hoodInspection()),
          ),
          throwsA(
            isA<NativeFailure>().having(
              (e) => e.code,
              'code',
              'API_UPGRADE_REQUIRED',
            ),
          ),
        );
        int count(String name) =>
            library.lookupFunction<Uint32 Function(), int Function()>(name)();
        expect(count('fixture_opened'), 1);
        expect(count('fixture_granted'), 0);
        expect(count('fixture_closed'), 1);
      } finally {
        await directory.delete(recursive: true);
      }
    },
  );
  test(
    'current native Mijia and combined sessions accept every exact hood grant',
    () async {
      for (final adapter in ['xiaomi.mijia', 'homeverb.platforms']) {
        final directory = await Directory.systemTemp.createTemp(
          'homeverb-hood-native-',
        );
        NativeClient? client;
        try {
          client = await NativeClient.open(
            dataDir: directory.path,
            adapter: adapter,
            libraryPath: '../../target/debug/${NativeClient.defaultLibrary()}',
            permissions: SessionPermission.choices(hoodInspection()),
          );
          expect(client.apiAtLeast(39), isTrue);
          expect(await client.call('list_devices'), isEmpty);
        } finally {
          await client?.close();
          await directory.delete(recursive: true);
        }
      }
    },
  );
  test('API 1.39 prepares all targets without executing any control', () async {
    final next = AuxiliaryCore(hoodInspection(), minor: 39);
    List<SessionPermission>? grants;
    final c = HomeController(
      AuxiliaryCore(hoodInspection(), minor: 39),
      AccountPreferences(),
      reopen: (permissions) async {
        grants = permissions;
        next.granted = permissions;
        return next;
      },
    );
    expect(await c.initialize(), isTrue);
    expect(await c.selectDevice(hoodInspection()['device_id']), isTrue);
    expect(grants, hasLength(10));
    expect(
      next.requests.where((r) => ['plan', 'execute'].contains(r['method'])),
      isEmpty,
    );
    c.dispose();
  });
  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    for (var index = 0; index < 4; index++) {
      testWidgets(
        'hood form submits exact single property at narrow enlarged $locale/$index',
        (tester) async {
          await LocaleSettings.setLocale(locale);
          tester.view.physicalSize = const Size(640, 1600);
          tester.view.devicePixelRatio = 2;
          tester.platformDispatcher.textScaleFactorTestValue = 2;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
          final option = hoodInspection()['options'][index] as Json;
          final schema =
              option['definition']['parameters']['properties'] as Json;
          final property = schema.keys.single;
          final values = schema[property]['type'] == 'boolean'
              ? [false, true]
              : schema[property]['enum'] as List;
          Json? sent;
          await tester.pumpWidget(
            TranslationProvider(
              child: MaterialApp(
                theme: windowTheme(),
                home: Scaffold(
                  body: SingleChildScrollView(
                    child: CapabilityActionForm(
                      capability: {
                        'id': option['capability'],
                        'properties': {},
                        'actions': {option['action']: option['definition']},
                      },
                      action: option['action'],
                      observation: (_) => null,
                      onSubmit: (args) async {
                        sent = args;
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
          final apply = find.byWidgetPredicate(
            (w) => w is ActionButton && w.label == 'Apply',
          );
          for (final value in values) {
            final choice = find.byKey(ValueKey('parameter:$property:$value'));
            await tester.ensureVisible(choice);
            await tester.tap(choice);
            await tester.ensureVisible(apply);
            await tester.tap(apply);
            await tester.pumpAndSettle();
            expect(sent, {property: value});
          }
          expect(tester.takeException(), isNull);
          await tester.pumpWidget(const SizedBox());
        },
      );
    }
  }
}
