import 'dart:convert';
import 'dart:ffi' hide Size;
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/approval_review.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/controls.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/session_permissions.dart';
import 'package:homeverb_mobile/skin.dart';
import 'accounts_test.dart' show AccountPreferences;
import 'session_permissions_test.dart' show inspection;
import 'mijia_settings_test.dart' show SettingCore, settingsInspection;

Json outletInspection() {
  final result = inspection();
  final rules = jsonDecode(
    File('../../schemas/native-open.schema.json').readAsStringSync(),
  )['properties']['mijia_outlet_grants']['items']['oneOf'];
  result['device_id'] = '40000000-0000-4000-8000-000000000001';
  result['options'] = [
    for (final rule in rules)
      (() {
        final component = rule['properties']['component_id']['const'] as String;
        final property = rule['properties']['property']['const'] as String;
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
            'risk': component == 'indicator_light-13' ? 'low' : 'elevated',
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

class OutletCore extends SettingCore
    implements ApprovalClient, NativeSessionClient {
  OutletCore(super.inspected, {this.minor = 34});
  final int minor;
  @override
  bool apiAtLeast(int version) => version <= minor;
  @override
  Set<String> get methods => {'inspect_control_options'};
  @override
  bool get supportsTrustedApproval => true;
  @override
  Future<ApprovalPreview> prepareApproval(Json command) async =>
      throw StateError('Device opening must not prepare a command.');
  @override
  Future<Json> executeApproved(ApprovalPreview preview) async =>
      throw StateError('Device opening must not execute a command.');
  @override
  Future<void> discardApproval(ApprovalPreview preview) async {}
}

// Display-only fixture, never accepted by NativeClient for execution.
class OutletPreview implements ApprovalPreview {
  OutletPreview(this.option, this.value);
  final Json option;
  final Object value;
  @override
  Json get command => {
    'request_id': 'fictional-outlet-request',
    'device_id': 'fictional-outlet',
    'component_id': option['component_id'],
    'capability': option['capability'],
    'action': option['action'],
    'arguments': <String, dynamic>{
      option['definition']['parameters']['required'][0]: value,
    },
  };
  @override
  Json get descriptor => {
    'device': {'alias': 'Fictional outlet'},
  };
  @override
  Json get connection => {'id': 'fictional-cloud', 'transport': 'cloud'};
  @override
  DateTime get expiresAt => DateTime.utc(2026, 10, 29);
  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnsupportedError('Display-only preview');
}

void main() {
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  test('all nine targets use independent outlet grants and exact values', () {
    final choices = SessionPermission.choices(
      outletInspection(),
      trustedApproval: true,
    );
    expect(choices, hasLength(9));
    expect(choices.every((p) => p.requiresOutletApi), true);
    final expected = jsonDecode(
      File('../../examples/native-open.mijia-outlet.json').readAsStringSync(),
    )['mijia_outlet_grants'];
    expect(SessionPermission.startup(choices), {
      'mijia_outlet_grants': expected,
    });
    expect(SessionPermission.startup([...choices, ...choices]), {
      'mijia_outlet_grants': expected,
    });
    final low = SessionPermission.choices(outletInspection());
    expect(low, hasLength(2));
    expect(low.every((p) => p.component == 'indicator_light-13'), true);
    final purifier = SessionPermission.choices(
      settingsInspection(false),
      trustedApproval: true,
    );
    expect(
      purifier
          .where((p) => p.component == 'physical_controls_locked-7')
          .every((p) => p.startupField == 'mijia_setting_grants'),
      true,
    );
  });
  test(
    'native negotiation sends no outlet field to an API 1.33 library',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'homeverb-old-outlet-api-',
      );
      try {
        final path = '${directory.path}/${NativeClient.defaultLibrary()}';
        final built = await Process.run('cc', [
          Platform.isMacOS ? '-dynamiclib' : '-shared',
          '-fPIC',
          '-std=c11',
          '-D_DEFAULT_SOURCE',
          'test/fixtures/outlet_startup_bridge.c',
          '-o',
          path,
        ]);
        expect(built.exitCode, 0, reason: '${built.stderr}');
        final library = DynamicLibrary.open(path);
        await expectLater(
          NativeClient.open(
            dataDir: directory.path,
            libraryPath: path,
            adapter: 'xiaomi.mijia',
            permissions: SessionPermission.choices(
              outletInspection(),
              trustedApproval: true,
            ),
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

  test('unexpected outlet risk, routes, scopes and domains are refused', () {
    final mutations = <void Function(Json)>[
      (v) => v['component_id'] = 'switch-3',
      (v) => v['definition']['risk'] = 'low',
      (v) => v['definition']['risk'] = 'high',
      (v) => v['definition']['transports'] = ['cloud', 'lan'],
      (v) => v['definition']['atomic'] = true,
      (v) => v['definition']['verification'] = 'read_back',
      (v) => v['definition']['parameters']['additionalProperties'] = true,
      (v) => v['definition']['parameters']['properties']['state'] = {
        'type': 'integer',
        'enum': [1],
      },
      (v) => v['definition']['parameters']['required'] = [],
      (v) => v['definition']['parameters']['properties']['confirmed'] = {
        'type': 'boolean',
      },
    ];
    for (final mutate in mutations) {
      final data = outletInspection();
      data['options'] = [data['options'][0]];
      mutate(data['options'][0]);
      expect(SessionPermission.choices(data, trustedApproval: true), isEmpty);
    }
    final data = outletInspection();
    data['options'] = [data['options'][1]];
    data['options'][0]['definition']['parameters']['properties']['default_power_on_state_2']['enum'] =
        [-1, 0, 1, 2, 3];
    expect(
      SessionPermission.choices(
        data,
        trustedApproval: true,
      ).map((p) => p.value),
      [0, 1, 2],
    );
  });
  test('API 1.33 refuses outlet setup before reopening or refresh', () async {
    final data = outletInspection(),
        core = OutletCore(outletInspection(), minor: 33);
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
    'opening an outlet captures only its grants and sends no command',
    () async {
      final data = outletInspection(),
          core = OutletCore(outletInspection()),
          next = OutletCore(outletInspection());
      List<SessionPermission>? configured;
      final c = HomeController(
        core,
        AccountPreferences(),
        reopen: (permissions) async {
          configured = permissions;
          return next;
        },
      );
      expect(await c.initialize(), true);
      expect(await c.selectDevice(data['device_id']), true);
      expect(configured, hasLength(9));
      expect(SessionPermission.startup(configured!).keys, [
        'mijia_outlet_grants',
      ]);
      expect(
        next.requests.where((r) => r['method'] == 'refresh_account'),
        hasLength(1),
      );
      expect(
        next.requests.where((r) => ['plan', 'execute'].contains(r['method'])),
        isEmpty,
      );
      c.dispose();
    },
  );
  test(
    'both native modes admit all exact outlet targets without networking',
    () async {
      for (final adapter in ['xiaomi.mijia', 'homeverb.platforms']) {
        final directory = await Directory.systemTemp.createTemp(
          'homeverb-outlet-',
        );
        NativeClient? client;
        try {
          client = await NativeClient.open(
            dataDir: directory.path,
            adapter: adapter,
            libraryPath: '../../target/debug/${NativeClient.defaultLibrary()}',
            permissions: SessionPermission.choices(
              outletInspection(),
              trustedApproval: true,
            ),
          );
          expect(client.apiAtLeast(34), true);
          expect(await client.call('list_devices'), isEmpty);
        } finally {
          await client?.close();
          await directory.delete(recursive: true);
        }
      }
    },
  );
  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    for (final index in [0, 1, 2]) {
      testWidgets(
        'outlet confirmation effects fit narrow enlarged text $locale/$index',
        (tester) async {
          LocaleSettings.setLocale(locale);
          tester.view.physicalSize = const Size(640, 1600);
          tester.view.devicePixelRatio = 2;
          tester.platformDispatcher.textScaleFactorTestValue = 2;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
          final labels = Map<String, String>.from(
            jsonDecode(
              File(
                '../../packages/presentation/locales/en-US.json',
              ).readAsStringSync(),
            ),
          );
          final localized = Map<String, String>.from(
            jsonDecode(
              File(
                '../../packages/presentation/locales/${locale == AppLocale.enUs ? 'en-US' : 'zh-CN'}.json',
              ).readAsStringSync(),
            ),
          );
          var confirmed = 0;
          final option = outletInspection()['options'][index] as Json;
          final preview = OutletPreview(option, index == 1 ? 2 : true);
          await tester.pumpWidget(
            TranslationProvider(
              child: MaterialApp(
                theme: windowTheme(),
                home: Scaffold(
                  body: SingleChildScrollView(
                    child: ApprovalReview(
                      preview: preview,
                      labels: labels,
                      onConfirm: () => confirmed++,
                      onCancel: () {},
                    ),
                  ),
                ),
              ),
            ),
          );
          await tester.pumpAndSettle();
          final key = index == 0
              ? 'component.switch_2.power.set.confirm'
              : '${option['capability']}.${option['action']}.confirm';
          final template = localized[key]!;
          final rendered = tester
              .widgetList<Text>(find.byType(Text))
              .map((w) => w.data ?? '')
              .join(' ');
          expect(
            rendered,
            contains(template.substring(template.lastIndexOf('}') + 1)),
          );
          expect(rendered, contains('Fictional outlet'));
          expect(rendered, isNot(contains('{target}')));
          expect(confirmed, 0);
          expect(tester.takeException(), isNull);
          await tester.pumpWidget(const SizedBox());
        },
      );
    }
    testWidgets(
      'elevated outlet form prepares only after explicit cloud selection $locale',
      (tester) async {
        LocaleSettings.setLocale(locale);
        final o = outletInspection()['options'][1] as Json;
        var sent = 0, prepared = 0;
        Json? target;
        await tester.pumpWidget(
          TranslationProvider(
            child: MaterialApp(
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
                    onSubmit: (_) async {
                      sent++;
                      return true;
                    },
                    onPrepare: (args, route) async {
                      expect(route, 'cloud');
                      target = args;
                      prepared++;
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
        final review = find.byWidgetPredicate(
          (w) => w is ActionButton && w.label == 'Review Command',
        );
        await tester.tap(review);
        await tester.pumpAndSettle();
        expect(prepared, 0);
        await tester.tap(
          find.byKey(const ValueKey('parameter:default_power_on_state_2:2')),
        );
        await tester.pumpAndSettle();
        await tester.tap(review);
        await tester.pumpAndSettle();
        expect(prepared, 0);
        await tester.tap(find.byType(ChoiceChip).last);
        await tester.pumpAndSettle();
        await tester.tap(review);
        await tester.pumpAndSettle();
        expect(prepared, 1);
        expect(target, {'default_power_on_state_2': 2});
        expect(sent, 0);
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
      },
    );
  }
}
