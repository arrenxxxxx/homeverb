import 'dart:async';
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
import 'mijia_auxiliary_test.dart' show AuxiliaryCore;

Json actionInspection(String component) {
  final data = inspection();
  final startup = jsonDecode(
    File('../../examples/native-open.mijia-actions.json').readAsStringSync(),
  );
  final entries = (startup['mijia_action_grants'] as List)
      .where((g) => g['component_id'] == component)
      .toList();
  data['device_id'] = entries.first['device_id'];
  data['options'] = [
    for (final entry in entries)
      {
        'component_id': component,
        'capability': entry['capability'],
        'action': entry['action'],
        'definition': jsonDecode(
          File(
            '../../examples/capability.mijia-action.json',
          ).readAsStringSync(),
        )['actions']['toggle'],
      },
  ];
  return data;
}

List<SessionPermission> choices(Json data) =>
    SessionPermission.choices(data, includeNoArgumentActions: true);

class ActionCore extends AuxiliaryCore {
  ActionCore(super.inspected, {super.minor = 40});
  Completer<Json>? executionGate;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'plan' || method == 'execute') {
      requests.add({'method': method, ...fields});
      if (method == 'plan')
        return {'command': fields['command'], 'warnings': []};
      if (executionGate != null) return executionGate!.future;
      return {
        'status': 'unknown',
        'request_id': fields['command']['request_id'],
      };
    }
    return super.call(method, fields);
  }
}

void main() {
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  test(
    'six exact actions use separate empty-argument startup grants only after opt-in',
    () {
      final permissions = <SessionPermission>[];
      for (final component in ['light-2', 'fan-2', 'light-6']) {
        final data = actionInspection(component);
        expect(SessionPermission.choices(data), isEmpty);
        permissions.addAll(choices(data));
      }
      expect(permissions, hasLength(6));
      expect(
        permissions.every((p) => p.requiresActionApi && p.arguments.isEmpty),
        true,
      );
      expect(SessionPermission.startup([...permissions, ...permissions]), {
        'mijia_action_grants': jsonDecode(
          File(
            '../../examples/native-open.mijia-actions.json',
          ).readAsStringSync(),
        )['mijia_action_grants'],
      });
      for (final mutate in <void Function(Json)>[
        (o) => o['component_id'] = 'light-3',
        (o) => o['capability'] = 'power',
        (o) => o['action'] = 'set',
        (o) => o['definition']['risk'] = 'elevated',
        (o) => o['definition']['idempotency'] = 'set_value',
        (o) => o['definition']['transports'] = ['cloud', 'lan'],
        (o) => o['definition']['atomic'] = true,
        (o) => o['definition']['independent_fields'] = true,
        (o) => o['definition']['verification'] = 'state_match',
        (o) => o['definition']['parameters']['additionalProperties'] = true,
        (o) => o['definition']['parameters']['properties']['state'] = {
          'type': 'boolean',
        },
        (o) => o['definition']['parameters']['required'] = ['state'],
        (o) => o['definition']['parameters']['oneOf'] = [],
      ]) {
        final data = actionInspection('fan-2');
        mutate(data['options'][0]);
        expect(choices(data), isEmpty);
      }
      final bad = actionInspection('fan-2')..['device_id'] = '*';
      expect(choices(bad), isEmpty);
    },
  );
  test('old native version receives no action opt-in or authority', () async {
    final data = actionInspection('fan-2');
    final core = ActionCore(data, minor: 39);
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
    expect(await c.selectDevice(data['device_id']), true);
    expect(reopened, false);
    expect(
      core.requests
          .where((r) => r['method'] == 'inspect_control_options')
          .every((r) => !r.containsKey('include_no_argument_actions')),
      true,
    );
    expect(
      core.requests.where(
        (r) => ['refresh_account', 'plan', 'execute'].contains(r['method']),
      ),
      isEmpty,
    );
    c.dispose();
  });
  test('actual old ABI refuses before any granted open', () async {
    final directory = await Directory.systemTemp.createTemp(
      'homeverb-actions-old-',
    );
    try {
      final path = '${directory.path}/${NativeClient.defaultLibrary()}';
      final built = await Process.run('cc', [
        Platform.isMacOS ? '-dynamiclib' : '-shared',
        '-fPIC',
        '-std=c11',
        '-D_DEFAULT_SOURCE',
        '-DFIXTURE_API_VERSION="1.39.0"',
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
          permissions: choices(actionInspection('fan-2')),
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
    'both actual native modes accept all six grants after negotiation',
    () async {
      for (final adapter in ['xiaomi.mijia', 'homeverb.platforms']) {
        final directory = await Directory.systemTemp.createTemp(
          'homeverb-actions-',
        );
        NativeClient? client;
        try {
          client = await NativeClient.open(
            dataDir: directory.path,
            adapter: adapter,
            libraryPath: '../../target/debug/${NativeClient.defaultLibrary()}',
            permissions: [
              for (final component in ['light-2', 'fan-2', 'light-6'])
                ...choices(actionInspection(component)),
            ],
          );
          expect(client.apiAtLeast(40), true);
          expect(await client.call('list_devices'), isEmpty);
        } finally {
          await client?.close();
          await directory.delete(recursive: true);
        }
      }
    },
  );
  test(
    'late action inspection cannot reopen or refresh a backgrounded device',
    () async {
      final data = actionInspection('fan-2');
      final core = ActionCore(data)..inspectionGate = Completer<Json>();
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
      final selecting = c.selectDevice(data['device_id']);
      await Future<void>.delayed(Duration.zero);
      c.setForeground(false);
      core.inspectionGate!.complete(data);
      expect(await selecting, false);
      expect(reopened, false);
      expect(
        core.requests.where(
          (r) => ['refresh_account', 'plan', 'execute'].contains(r['method']),
        ),
        isEmpty,
      );
      c.dispose();
    },
  );
  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    for (final component in ['light-2', 'fan-2', 'light-6']) {
      testWidgets(
        'exact actions prepare locally and submit once at 320px double text $locale/$component',
        (tester) async {
          LocaleSettings.setLocale(locale);
          tester.view.physicalSize = const Size(640, 1800);
          tester.view.devicePixelRatio = 2;
          tester.platformDispatcher.textScaleFactorTestValue = 2;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
          final data = actionInspection(component),
              initial = ActionCore(actionInspection(component)),
              next = ActionCore(actionInspection(component));
          final c = HomeController(
            initial,
            AccountPreferences(),
            reopen: (permissions) async {
              next.granted = permissions;
              return next;
            },
          );
          expect(await c.initialize(), true);
          expect(await c.selectDevice(data['device_id']), true);
          expect(
            initial.requests
                .where((r) => r['method'] == 'inspect_control_options')
                .single['include_no_argument_actions'],
            true,
          );
          expect(next.granted.length, component == 'light-2' ? 4 : 1);
          expect(
            next.requests.where((r) => r['method'] == 'refresh_account'),
            hasLength(1),
          );
          String? selected;
          Widget wrap(Widget child) => TranslationProvider(
            child: MaterialApp(
              theme: windowTheme(),
              home: Scaffold(body: SingleChildScrollView(child: child)),
            ),
          );
          await tester.pumpWidget(
            wrap(
              DeviceDetailControls(
                controller: c,
                onAction: (a) => selected = a,
              ),
            ),
          );
          await tester.pumpAndSettle();
          for (final option in data['options']) {
            final key = 'control:0:${option['action']}';
            final button = find.byKey(ValueKey(key));
            await tester.ensureVisible(button);
            await tester.tap(button);
            expect(selected, key);
          }
          expect(
            next.requests.where(
              (r) => ['plan', 'execute'].contains(r['method']),
            ),
            isEmpty,
          );
          final option = data['options'][0];
          next.executionGate = Completer<Json>();
          await tester.pumpWidget(
            wrap(
              CapabilityActionForm(
                capability: c.capability(option['capability'])!,
                action: option['action'],
                observation: (_) => null,
                onSubmit: (args) =>
                    c.execute(option['capability'], option['action'], args),
                onCancel: () {},
                error: () => c.error,
              ),
            ),
          );
          await tester.pumpAndSettle();
          final apply = find.widgetWithText(
            ActionButton,
            locale.translations.actionApply,
          );
          await tester.ensureVisible(apply);
          await tester.tap(apply);
          await tester.pump();
          await tester.tap(apply);
          await tester.pump();
          final sent = next.requests
              .where((r) => ['plan', 'execute'].contains(r['method']))
              .toList();
          expect(sent.map((r) => r['method']), ['plan', 'execute']);
          expect(sent[0]['command'], sent[1]['command']);
          expect(sent[1]['command']['arguments'], isEmpty);
          expect(sent[1]['command']['component_id'], component);
          next.executionGate!.complete({
            'status': 'unknown',
            'request_id': sent[1]['command']['request_id'],
          });
          await tester.pumpAndSettle();
          expect(c.receipt?['status'], 'unknown');
          expect(
            next.requests.where((r) => r['method'] == 'execute'),
            hasLength(1),
          );
          expect(tester.takeException(), isNull);
          await tester.pumpWidget(const SizedBox());
          c.dispose();
        },
      );
    }
  }
}
