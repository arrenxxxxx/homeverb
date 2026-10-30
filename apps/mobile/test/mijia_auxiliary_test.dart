import 'dart:convert';
import 'dart:ffi' hide Size;
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
import 'mijia_settings_test.dart' show SettingCore;

Json auxiliaryInspection(bool hood) {
  final result = inspection();
  result['device_id'] = hood
      ? '40000000-0000-4000-8000-000000000001'
      : '40000000-0000-4000-8000-000000000002';
  final rules = jsonDecode(
    File('../../schemas/native-open.schema.json').readAsStringSync(),
  )['properties']['mijia_setting_grants']['items']['oneOf'];
  result['options'] = [
    for (final rule in rules)
      if (rule['properties']['component_id']['const'] ==
          (hood ? 'light-6' : 'function-3'))
        (() {
          final component =
              rule['properties']['component_id']['const'] as String;
          final property = rule['properties']['property']['const'] as String;
          return <String, dynamic>{
            'component_id': component,
            'capability': property == 'state'
                ? 'power'
                : 'vendor.xiaomi.mijia.${component.split('-').first}',
            'action': property == 'state' ? 'set' : 'set_$property',
            'definition': {
              'parameters': {
                'type': 'object',
                'properties': <String, dynamic>{
                  property: rule['properties']['value'],
                },
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

class AuxiliaryCore extends SettingCore implements NativeSessionClient {
  AuxiliaryCore(super.inspected, {this.minor = 35});
  final int minor;
  List<SessionPermission> granted = const [];
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'inspect_session_grants') {
      return {
        'device_id': fields['device_id'],
        'connection_id': fields['connection_id'],
        'transport': 'cloud',
        'source': 'session_policy',
        'provider_authorization': 'not_checked',
        'grants': [
          for (final grant in granted)
            {
              'component_id': grant.component,
              'capability': grant.capability,
              'action': grant.action,
              'argument_schema': {'const': grant.arguments},
            },
        ],
      };
    }
    return super.call(method, fields);
  }

  @override
  Set<String> get methods => {
    'inspect_control_options',
    'inspect_session_grants',
  };
  @override
  bool apiAtLeast(int version) => version <= minor;
}

void main() {
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  test(
    'all seven targets retain exact codes without percentages or fan grants',
    () {
      final choices = [
        ...SessionPermission.choices(auxiliaryInspection(true)),
        ...SessionPermission.choices(auxiliaryInspection(false)),
      ];
      expect(choices, hasLength(7));
      expect(choices.every((p) => p.requiresAuxiliarySettingApi), true);
      final expected = jsonDecode(
        File(
          '../../examples/native-open.mijia-auxiliary.json',
        ).readAsStringSync(),
      )['mijia_setting_grants'];
      expect(SessionPermission.startup(choices), {
        'mijia_setting_grants': expected,
      });
      final wrong = auxiliaryInspection(true);
      wrong['options'] = [wrong['options'][1]];
      wrong['options'][0]['definition']['parameters']['properties']['brightness_2']['enum'] =
          [-1, 0, 1, 2, 50, 100];
      expect(SessionPermission.choices(wrong).map((p) => p.value), [0, 1, 2]);
      for (final mutation in <void Function(Json)>[
        (v) => v['component_id'] = 'light-7',
        (v) => v['action'] = 'set_timing_1',
        (v) => v['definition']['risk'] = 'elevated',
        (v) => v['definition']['parameters']['properties']['brightness_2'] = {
          'type': 'number',
          'enum': [1.5],
        },
        (v) => v['definition']['parameters']['additionalProperties'] = true,
      ]) {
        final data = auxiliaryInspection(true);
        data['options'] = [data['options'][1]];
        mutation(data['options'][0]);
        expect(SessionPermission.choices(data), isEmpty);
      }
    },
  );
  test(
    'API 1.34 refuses auxiliary grants before reopening or refresh',
    () async {
      for (final hood in [false, true]) {
        final data = auxiliaryInspection(hood),
            core = AuxiliaryCore(auxiliaryInspection(hood), minor: 34);
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
      }
    },
  );
  test('actual old ABI sees only the read-only probe before refusal', () async {
    final directory = await Directory.systemTemp.createTemp(
      'homeverb-auxiliary-old-',
    );
    try {
      final path = '${directory.path}/${NativeClient.defaultLibrary()}';
      final built = await Process.run('cc', [
        Platform.isMacOS ? '-dynamiclib' : '-shared',
        '-fPIC',
        '-std=c11',
        '-D_DEFAULT_SOURCE',
        '-DFIXTURE_API_VERSION="1.34.0"',
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
          permissions: SessionPermission.choices(auxiliaryInspection(true)),
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
  });
  test('both native modes accept all seven exact auxiliary grants', () async {
    for (final adapter in ['xiaomi.mijia', 'homeverb.platforms']) {
      final directory = await Directory.systemTemp.createTemp(
        'homeverb-auxiliary-',
      );
      NativeClient? client;
      try {
        client = await NativeClient.open(
          dataDir: directory.path,
          adapter: adapter,
          libraryPath: '../../target/debug/${NativeClient.defaultLibrary()}',
          permissions: [
            ...SessionPermission.choices(auxiliaryInspection(true)),
            ...SessionPermission.choices(auxiliaryInspection(false)),
          ],
        );
        expect(client.apiAtLeast(35), true);
        expect(await client.call('list_devices'), isEmpty);
      } finally {
        await client?.close();
        await directory.delete(recursive: true);
      }
    }
  });
  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    for (final hood in [true, false]) {
      testWidgets(
        'auxiliary panel fits narrow enlarged text without writes $locale/$hood',
        (tester) async {
          LocaleSettings.setLocale(locale);
          tester.view.physicalSize = const Size(640, 1600);
          tester.view.devicePixelRatio = 2;
          tester.platformDispatcher.textScaleFactorTestValue = 2;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
          final data = auxiliaryInspection(hood),
              next = AuxiliaryCore(auxiliaryInspection(hood));
          List<SessionPermission>? grants;
          final c = HomeController(
            AuxiliaryCore(data),
            AccountPreferences(),
            reopen: (permissions) async {
              grants = permissions;
              next.granted = permissions;
              return next;
            },
          );
          expect(await c.initialize(), true);
          expect(await c.selectDevice(data['device_id']), true);
          expect(grants, hasLength(hood ? 5 : 2));
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
          final action = hood ? 'set_brightness_2' : 'set_quiet_5';
          final button = find.byKey(
            ValueKey('control:${hood ? 1 : 0}:$action'),
          );
          await tester.ensureVisible(button);
          await tester.tap(button);
          await tester.pumpAndSettle();
          expect(selected, 'control:${hood ? 1 : 0}:$action');
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
    testWidgets('hood brightness form keeps exact code zero and two $locale', (
      tester,
    ) async {
      LocaleSettings.setLocale(locale);
      final option = auxiliaryInspection(true)['options'][1] as Json;
      Json? sent;
      await tester.pumpWidget(
        TranslationProvider(
          child: MaterialApp(
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
      expect(
        find.byKey(const ValueKey('parameter:brightness_2:0')),
        findsOneWidget,
      );
      expect(
        find.byKey(const ValueKey('parameter:brightness_2:1')),
        findsOneWidget,
      );
      expect(
        find.byKey(const ValueKey('parameter:brightness_2:2')),
        findsOneWidget,
      );
      final apply = find.byWidgetPredicate(
        (w) => w is ActionButton && w.label == 'Apply',
      );
      await tester.tap(apply);
      await tester.pumpAndSettle();
      expect(sent, isNull);
      await tester.tap(find.byKey(const ValueKey('parameter:brightness_2:0')));
      await tester.tap(apply);
      await tester.pumpAndSettle();
      expect(sent, {'brightness_2': 0});
      await tester.tap(find.byKey(const ValueKey('parameter:brightness_2:2')));
      await tester.tap(apply);
      await tester.pumpAndSettle();
      expect(sent, {'brightness_2': 2});
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    });
  }
}
