import 'dart:async';
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
import 'mijia_auxiliary_test.dart' show AuxiliaryCore;
import 'mijia_hood_test.dart' show hoodInspection, preferenceProperties;

Json preferences() => hoodInspection(preferences: true);
List<SessionPermission> choices() => SessionPermission.choices(preferences());

class PreferenceCore extends AuxiliaryCore {
  PreferenceCore({int minor = 45}) : super(preferences(), minor: minor);
  Completer<void>? planGate, executionGate;
  final planEntered = Completer<void>();
  final executionEntered = Completer<void>();
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'plan' || method == 'execute') {
      requests.add({'method': method, ...fields});
      if (method == 'plan') {
        if (!planEntered.isCompleted) planEntered.complete();
        await planGate?.future;
      } else {
        if (!executionEntered.isCompleted) executionEntered.complete();
        await executionGate?.future;
      }
      return method == 'plan'
          ? <String, dynamic>{}
          : <String, dynamic>{
              'request_id': fields['command']['request_id'],
              'device_id': fields['command']['device_id'],
              'status': 'unknown',
            };
    }
    return super.call(method, fields);
  }
}

void main() {
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  test(
    'twelve preference targets retain exact values and independent startup authority',
    () {
      final offered = choices();
      final fixture = jsonDecode(
        File(
          '../../examples/native-open.mijia-hood-preferences.json',
        ).readAsStringSync(),
      );
      expect(offered, hasLength(12));
      expect(
        offered.every(
          (p) =>
              p.requiresHoodPreferenceApi &&
              !p.requiresOutletApi &&
              !p.requiresReferenceFanApi,
        ),
        true,
      );
      expect(SessionPermission.startup(offered), {
        'mijia_setting_grants': fixture['mijia_setting_grants'],
      });
      expect(
        offered
            .where((p) => p.parameter == 'clean_remind_time_12')
            .map((p) => p.value),
        [20, 30, 40, 50],
      );
      expect(
        SessionPermission.choices(
          hoodInspection(),
        ).every((p) => !p.requiresHoodPreferenceApi),
        true,
      );
    },
  );
  for (final fault in [
    'lower',
    'upper',
    'step',
    'fraction',
    'enum',
    'type',
    'missing',
    'unknown schema',
    'foreign component',
    'foreign action',
    'foreign capability',
    'risk',
    'transport',
    'verification',
    'atomic',
    'independent',
    'extra argument',
    'optional',
  ]) {
    test('refuse altered reminder admission: $fault', () {
      final data = preferences();
      final option =
          (data['options'] as List).firstWhere(
                (o) => o['action'] == 'set_clean_remind_time_12',
              )
              as Json;
      data['options'] = [option];
      final definition = option['definition'] as Json;
      final parameters = definition['parameters'] as Json;
      final schema = parameters['properties']['clean_remind_time_12'] as Json;
      switch (fault) {
        case 'lower':
          schema['minimum'] = 10;
        case 'upper':
          schema['maximum'] = 60;
        case 'step':
          schema['multipleOf'] = 5;
        case 'fraction':
          schema['minimum'] = 20.5;
        case 'enum':
          schema['enum'] = [20, 30, 40, 50];
        case 'type':
          schema['type'] = 'number';
        case 'missing':
          schema.remove('multipleOf');
        case 'unknown schema':
          schema['unit'] = 'hours';
        case 'foreign component':
          option['component_id'] = 'hood-3';
        case 'foreign action':
          option['action'] = 'set_clean_remind_time_29';
        case 'foreign capability':
          option['capability'] = 'vendor.xiaomi.mijia.function';
        case 'risk':
          definition['risk'] = 'elevated';
        case 'transport':
          definition['transports'] = ['cloud', 'lan'];
        case 'verification':
          definition['verification'] = 'state_match';
        case 'atomic':
          definition['atomic'] = true;
        case 'independent':
          definition['independent_fields'] = true;
        case 'extra argument':
          parameters['properties']['clean_remind_on_11'] = {'type': 'boolean'};
        case 'optional':
          parameters['required'] = [];
      }
      expect(SessionPermission.choices(data), isEmpty);
    });
  }
  test(
    'Boolean preference domains cannot become numeric or partial targets',
    () {
      for (final property in preferenceProperties.where(
        (p) => p != 'clean_remind_time_12',
      )) {
        for (final schema in [
          {
            'type': 'integer',
            'enum': [0, 1],
          },
          {
            'type': 'boolean',
            'enum': [true],
          },
          {'type': 'boolean', 'minimum': 0},
        ]) {
          final data = preferences();
          final option = (data['options'] as List).firstWhere(
            (o) => o['action'] == 'set_$property',
          );
          data['options'] = [option];
          option['definition']['parameters']['properties'][property] = schema;
          expect(SessionPermission.choices(data), isEmpty);
        }
      }
    },
  );
  test(
    'API 1.44 controller refuses before granted reopen, refresh or execution',
    () async {
      final core = PreferenceCore(minor: 44);
      var opened = false;
      final c = HomeController(
        core,
        AccountPreferences(),
        reopen: (_) async {
          opened = true;
          return core;
        },
      );
      expect(await c.initialize(), true);
      expect(await c.selectDevice(preferences()['device_id']), false);
      expect(
        c.error,
        isA<NativeFailure>().having(
          (e) => e.code,
          'code',
          'API_UPGRADE_REQUIRED',
        ),
      );
      expect(opened, false);
      expect(
        core.requests.where(
          (r) => ['plan', 'execute', 'refresh_account'].contains(r['method']),
        ),
        isEmpty,
      );
      c.dispose();
    },
  );
  test('actual API 1.44 receives only an unprivileged version probe', () async {
    final directory = await Directory.systemTemp.createTemp(
      'homeverb-hood-preferences-old-',
    );
    try {
      final path = '${directory.path}/${NativeClient.defaultLibrary()}';
      final build = await Process.run('cc', [
        Platform.isMacOS ? '-dynamiclib' : '-shared',
        '-fPIC',
        '-std=c11',
        '-D_DEFAULT_SOURCE',
        '-DFIXTURE_API_VERSION="1.44.0"',
        'test/fixtures/outlet_startup_bridge.c',
        '-o',
        path,
      ]);
      expect(build.exitCode, 0, reason: '${build.stderr}');
      final library = DynamicLibrary.open(path);
      await expectLater(
        NativeClient.open(
          dataDir: directory.path,
          adapter: 'xiaomi.mijia',
          libraryPath: path,
          permissions: choices(),
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
  test(
    'actual API 1.45 admits exact preferences in both native modes without networking',
    () async {
      for (final adapter in ['xiaomi.mijia', 'homeverb.platforms']) {
        final directory = await Directory.systemTemp.createTemp(
          'homeverb-hood-preferences-native-',
        );
        NativeClient? client;
        try {
          client = await NativeClient.open(
            dataDir: directory.path,
            adapter: adapter,
            libraryPath: '../../target/debug/${NativeClient.defaultLibrary()}',
            permissions: choices(),
          );
          expect(client.apiAtLeast(45), true);
          expect(await client.call('list_devices'), isEmpty);
        } finally {
          await client?.close();
          await directory.delete(recursive: true);
        }
      }
    },
  );
  test(
    'preparing preferences sends nothing; each execution is exact and background refuses',
    () async {
      final next = PreferenceCore();
      final c = HomeController(
        PreferenceCore(),
        AccountPreferences(),
        reopen: (permissions) async {
          next.granted = permissions;
          return next;
        },
      );
      expect(await c.initialize(), true);
      expect(await c.selectDevice(preferences()['device_id']), true);
      expect(next.granted, hasLength(12));
      expect(
        next.requests.where((r) => ['plan', 'execute'].contains(r['method'])),
        isEmpty,
      );
      final before = c.snapshot;
      for (final grant in next.granted) {
        final count = next.requests
            .where((r) => r['method'] == 'execute')
            .length;
        expect(
          await c.execute(
            grant.capability,
            grant.action,
            grant.arguments,
            componentId: grant.component,
          ),
          true,
        );
        final calls = next.requests
            .where((r) => r['method'] == 'execute')
            .toList();
        expect(calls, hasLength(count + 1));
        final command = calls.last['command'];
        expect(command['device_id'], grant.device);
        expect(command['component_id'], 'hood-2');
        expect(command['capability'], grant.capability);
        expect(command['action'], grant.action);
        expect(command['arguments'], grant.arguments);
        expect(c.receipt!['status'], 'unknown');
        expect(c.snapshot, same(before));
      }
      final count = next.requests.length;
      c.setForeground(false);
      final grant = next.granted.first;
      expect(
        await c.execute(
          grant.capability,
          grant.action,
          grant.arguments,
          componentId: grant.component,
        ),
        false,
      );
      expect(next.requests, hasLength(count));
      c.dispose();
    },
  );
  for (final ending in [
    'background',
    'resume',
    'cancel',
    'device',
    'descriptor',
    'session',
  ]) {
    test('pending control plan cannot send after $ending', () async {
      final next = PreferenceCore();
      final c = HomeController(
        PreferenceCore(),
        AccountPreferences(),
        reopen: (permissions) async {
          next.granted = permissions;
          return next;
        },
      );
      expect(await c.initialize(), true);
      expect(await c.selectDevice(preferences()['device_id']), true);
      next.planGate = Completer<void>();
      final previous = <String, dynamic>{
        'status': 'unknown',
        'request_id': 'previous',
      };
      c.receipt = previous;
      final grant = next.granted.first;
      final work = c.execute(
        grant.capability,
        grant.action,
        grant.arguments,
        componentId: grant.component,
      );
      await next.planEntered.future;
      switch (ending) {
        case 'background':
          c.setForeground(false);
        case 'resume':
          c.setForeground(false);
          c.setForeground(true);
        case 'cancel':
          c.cancelPending();
        case 'device':
          c.selectedDevice = 'different-device';
        case 'descriptor':
          c.description = {...c.description!};
        case 'session':
          c.core = PreferenceCore();
      }
      next.planGate!.complete();
      expect(await work, false);
      expect(
        c.error,
        isA<NativeFailure>().having((e) => e.code, 'code', 'CANCELLED'),
      );
      expect(next.requests.where((r) => r['method'] == 'execute'), isEmpty);
      expect(c.receipt, same(previous));
      c.dispose();
    });
  }
  test(
    'background after send preserves the unknown receipt without replay',
    () async {
      final next = PreferenceCore();
      final c = HomeController(
        PreferenceCore(),
        AccountPreferences(),
        reopen: (permissions) async {
          next.granted = permissions;
          return next;
        },
      );
      expect(await c.initialize(), true);
      expect(await c.selectDevice(preferences()['device_id']), true);
      next.executionGate = Completer<void>();
      final grant = next.granted.first;
      final work = c.execute(
        grant.capability,
        grant.action,
        grant.arguments,
        componentId: grant.component,
      );
      await next.executionEntered.future;
      c.setForeground(false);
      next.executionGate!.complete();
      expect(await work, true);
      expect(c.receipt!['status'], 'unknown');
      expect(
        next.requests.where((r) => r['method'] == 'execute'),
        hasLength(1),
      );
      c.dispose();
    },
  );
  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    for (final property in preferenceProperties) {
      testWidgets(
        'preference form sends one exact property at 320px double text $locale/$property',
        (tester) async {
          await LocaleSettings.setLocale(locale);
          tester.view.physicalSize = const Size(640, 1600);
          tester.view.devicePixelRatio = 2;
          tester.platformDispatcher.textScaleFactorTestValue = 2;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
          final option = (preferences()['options'] as List).firstWhere(
            (o) => o['action'] == 'set_$property',
          );
          final submitted = <Json>[];
          await tester.pumpWidget(
            TranslationProvider(
              child: MaterialApp(
                theme: windowTheme(),
                home: Scaffold(
                  body: SingleChildScrollView(
                    child: CapabilityActionForm(
                      capability: {
                        'id': option['capability'],
                        'properties': {
                          property: {
                            if (property == 'clean_remind_time_12')
                              'unit': 'hours',
                          },
                        },
                        'actions': {option['action']: option['definition']},
                      },
                      action: option['action'],
                      observation: (_) => null,
                      onSubmit: (args) async {
                        submitted.add(args);
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
          expect(submitted, isEmpty);
          expect(controlLabel(property), isNot(contains('_')));
          final apply = find.byWidgetPredicate(
            (w) => w is ActionButton && w.label == 'Apply',
          );
          final threshold = property == 'clean_remind_time_12';
          if (threshold) {
            await tester.enterText(
              find.byKey(ValueKey('parameter:$property')),
              '25',
            );
            await tester.ensureVisible(apply);
            await tester.tap(apply);
            await tester.pumpAndSettle();
            expect(submitted, isEmpty);
          }
          for (final value
              in threshold ? <Object>[20, 30, 40, 50] : <Object>[false, true]) {
            if (threshold) {
              await tester.enterText(
                find.byKey(ValueKey('parameter:$property')),
                '$value',
              );
            } else {
              final target = find.byKey(ValueKey('parameter:$property:$value'));
              await tester.ensureVisible(target);
              await tester.tap(target);
            }
            await tester.ensureVisible(apply);
            await tester.tap(apply);
            await tester.pumpAndSettle();
            expect(submitted.last, {property: value});
          }
          expect(submitted, hasLength(threshold ? 4 : 2));
          expect(tester.takeException(), isNull);
          await tester.pumpWidget(const SizedBox());
        },
      );
    }
  }
}
