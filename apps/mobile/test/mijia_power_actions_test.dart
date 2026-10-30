import 'dart:async';
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
import 'mijia_actions_test.dart' show ActionCore, actionInspection;

const components = ['switch-2', 'switch-3', 'switch-11', 'switch-12'];
const capability = 'vendor.xiaomi.mijia.switch';
Json powerInspection(String component) {
  final data = inspection();
  final entries =
      jsonDecode(
            File(
              '../../examples/native-open.mijia-power-actions.json',
            ).readAsStringSync(),
          )['mijia_action_grants']
          as List;
  final entry = entries.singleWhere((e) => e['component_id'] == component);
  data['device_id'] = entry['device_id'];
  data['options'] = [
    {
      'component_id': component,
      'capability': capability,
      'action': 'toggle',
      'definition': jsonDecode(
        File(
          '../../examples/capability.mijia-power-action.json',
        ).readAsStringSync(),
      )['actions']['toggle'],
    },
  ];
  return data;
}

List<SessionPermission> choices(Json data) => SessionPermission.choices(
  data,
  trustedApproval: true,
  includePowerToggleActions: true,
);

// Fictional UI fixture. Real native handles are exercised by approval_test.dart
// and immutable protected power scope is covered by the Host account suite.
class PowerPreview implements ApprovalPreview {
  PowerPreview(Json input, this.action)
    : command = Map.unmodifiable({
        ...input,
        'arguments': const <String, dynamic>{},
      });
  @override
  final Json command;
  @override
  final Json action;
  @override
  Json get descriptor => {
    'device': {'alias': 'Fictional switch', 'id': command['device_id']},
  };
  @override
  Json get connection => {'id': 'fictional-cloud', 'transport': 'cloud'};
  @override
  DateTime get expiresAt => DateTime.now().add(remaining);
  @override
  Duration get remaining => const Duration(seconds: 60);
}

class PowerCore extends ActionCore implements ApprovalClient {
  PowerCore(super.inspected, {super.minor = 41, this.trusted = true});
  final bool trusted;
  @override
  bool get supportsTrustedApproval => trusted;
  final prepared = <ApprovalPreview>[],
      sent = <ApprovalPreview>[],
      discarded = <ApprovalPreview>[];
  @override
  Future<ApprovalPreview> prepareApproval(Json command) async {
    final preview = PowerPreview(
      command,
      inspected['options'][0]['definition'],
    );
    prepared.add(preview);
    return preview;
  }

  @override
  Future<Json> executeApproved(ApprovalPreview preview) async {
    sent.add(preview);
    return executionGate == null
        ? {'status': 'unknown', 'request_id': preview.command['request_id']}
        : await executionGate!.future;
  }

  @override
  Future<void> discardApproval(ApprovalPreview preview) async {
    discarded.add(preview);
  }
}

void main() {
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  test(
    'power choices require explicit opt-in and trusted approval with exact elevated metadata',
    () {
      final permissions = <SessionPermission>[];
      for (final component in components) {
        final data = powerInspection(component);
        expect(
          SessionPermission.choices(
            data,
            trustedApproval: true,
            includeNoArgumentActions: true,
          ),
          isEmpty,
        );
        expect(
          SessionPermission.choices(data, includePowerToggleActions: true),
          isEmpty,
        );
        permissions.addAll(choices(data));
        for (final mutate in <void Function(Json)>[
          (o) => o['component_id'] = 'switch-4',
          (o) => o['capability'] = 'power',
          (o) => o['action'] = 'set',
          (o) => o['definition']['risk'] = 'low',
          (o) => o['definition']['risk'] = 'high',
          (o) => o['definition']['idempotency'] = 'set_value',
          (o) => o['definition']['verification'] = 'state_match',
          (o) => o['definition']['transports'] = ['cloud', 'lan'],
          (o) => o['definition']['parameters']['additionalProperties'] = true,
          (o) => o['definition']['parameters']['properties']['confirmed'] = {
            'const': true,
          },
        ]) {
          final bad = powerInspection(component);
          mutate(bad['options'][0]);
          expect(choices(bad), isEmpty);
        }
      }
      expect(permissions, hasLength(4));
      expect(
        permissions.every(
          (p) => p.requiresPowerToggleApi && p.arguments.isEmpty,
        ),
        true,
      );
      expect(SessionPermission.startup([...permissions, ...permissions]), {
        'mijia_action_grants': jsonDecode(
          File(
            '../../examples/native-open.mijia-power-actions.json',
          ).readAsStringSync(),
        )['mijia_action_grants'],
      });
      expect(choices(actionInspection('fan-2')), isEmpty);
    },
  );
  test(
    'old versions or absent approval support never opt in or reopen with power authority',
    () async {
      for (final core in [
        PowerCore(powerInspection('switch-2'), minor: 40),
        PowerCore(powerInspection('switch-2'), trusted: false),
      ]) {
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
        expect(await c.selectDevice(core.inspected['device_id']), true);
        expect(reopened, false);
        expect(
          core.requests
              .where((r) => r['method'] == 'inspect_control_options')
              .every((r) => !r.containsKey('include_power_toggle_actions')),
          true,
        );
        expect(core.sent, isEmpty);
        c.dispose();
      }
    },
  );
  test(
    'actual API 1.40 peer is closed before any power-granted opening',
    () async {
      final dir = await Directory.systemTemp.createTemp('homeverb-power-old-');
      try {
        final path = '${dir.path}/${NativeClient.defaultLibrary()}';
        final built = await Process.run('cc', [
          Platform.isMacOS ? '-dynamiclib' : '-shared',
          '-fPIC',
          '-std=c11',
          '-D_DEFAULT_SOURCE',
          '-DFIXTURE_API_VERSION="1.40.0"',
          'test/fixtures/outlet_startup_bridge.c',
          '-o',
          path,
        ]);
        expect(built.exitCode, 0, reason: '${built.stderr}');
        final lib = DynamicLibrary.open(path);
        await expectLater(
          NativeClient.open(
            dataDir: dir.path,
            adapter: 'xiaomi.mijia',
            libraryPath: path,
            permissions: choices(powerInspection('switch-2')),
          ),
          throwsA(
            isA<NativeFailure>().having(
              (e) => e.code,
              'code',
              'API_UPGRADE_REQUIRED',
            ),
          ),
        );
        int count(String key) =>
            lib.lookupFunction<Uint32 Function(), int Function()>(key)();
        expect(count('fixture_opened'), 1);
        expect(count('fixture_closed'), 1);
        expect(count('fixture_granted'), 0);
      } finally {
        await dir.delete(recursive: true);
      }
    },
  );
  test(
    'both rebuilt native modes accept power grants after API 1.41 negotiation',
    () async {
      for (final adapter in ['xiaomi.mijia', 'homeverb.platforms']) {
        final dir = await Directory.systemTemp.createTemp(
          'homeverb-power-new-',
        );
        NativeClient? core;
        try {
          core = await NativeClient.open(
            dataDir: dir.path,
            adapter: adapter,
            libraryPath: '../../target/debug/${NativeClient.defaultLibrary()}',
            permissions: [
              for (final component in components)
                ...choices(powerInspection(component)),
            ],
          );
          expect(core.apiAtLeast(41), true);
          expect(core.supportsTrustedApproval, true);
          expect(await core.call('list_devices'), isEmpty);
        } finally {
          await core?.close();
          await dir.delete(recursive: true);
        }
      }
    },
  );
  test(
    'late power inspection cannot grant or refresh after background',
    () async {
      final core = PowerCore(powerInspection('switch-3'))
        ..inspectionGate = Completer<Json>();
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
      final selecting = c.selectDevice(core.inspected['device_id']);
      await Future<void>.delayed(Duration.zero);
      c.setForeground(false);
      core.inspectionGate!.complete(core.inspected);
      expect(await selecting, false);
      expect(reopened, false);
      expect(core.sent, isEmpty);
      c.dispose();
    },
  );
  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    for (final component in components) {
      testWidgets(
        'power review sends once after explicit confirmation $locale/$component',
        (tester) async {
          LocaleSettings.setLocale(locale);
          tester.view.physicalSize = const Size(640, 1800);
          tester.view.devicePixelRatio = 2;
          tester.platformDispatcher.textScaleFactorTestValue = 2;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
          final initial = PowerCore(powerInspection(component)),
              next = PowerCore(powerInspection(component));
          final c = HomeController(
            initial,
            AccountPreferences(),
            reopen: (p) async {
              next.granted = p;
              return next;
            },
          );
          expect(await c.initialize(), true);
          expect(await c.selectDevice(initial.inspected['device_id']), true);
          expect(
            initial.requests
                .where((r) => r['method'] == 'inspect_control_options')
                .single['include_power_toggle_actions'],
            true,
          );
          expect(next.granted.single.requiresPowerToggleApi, true);
          expect(
            next.requests.where((r) => r['method'] == 'refresh_account'),
            hasLength(1),
          );
          expect(next.prepared, isEmpty);
          expect(next.sent, isEmpty);
          expect(await c.execute(capability, 'toggle', {}), false);
          Widget wrap(Widget child) => TranslationProvider(
            child: MaterialApp(
              theme: windowTheme(),
              home: Scaffold(body: SingleChildScrollView(child: child)),
            ),
          );
          await tester.pumpWidget(
            wrap(
              CapabilityActionForm(
                capability: c.capability(capability)!,
                action: 'toggle',
                observation: (_) => null,
                onSubmit: (a) => c.execute(capability, 'toggle', a),
                onPrepare: (a, r) =>
                    c.prepareApproval(capability, 'toggle', a, r),
                onCancel: c.discardApproval,
                error: () => c.error,
              ),
            ),
          );
          await tester.pumpAndSettle();
          final chip = find.byType(ChoiceChip);
          await tester.ensureVisible(chip);
          await tester.tap(chip);
          await tester.pump();
          final review = find.widgetWithText(
            ActionButton,
            locale.translations.mobileReviewCommand,
          );
          await tester.ensureVisible(review);
          await tester.tap(review);
          await tester.pumpAndSettle();
          expect(next.prepared, hasLength(1));
          expect(next.sent, isEmpty);
          final preview = c.approvalPreview!;
          expect(preview.command['arguments'], isEmpty);
          expect(preview.command['component_id'], component);
          expect(preview.command['connection'], 'cloud');
          next.executionGate = Completer<Json>();
          await tester.pumpWidget(
            wrap(
              ApprovalReview(
                preview: preview,
                labels: Map<String, String>.from(
                  jsonDecode(
                    File(
                      '../../packages/presentation/locales/en-US.json',
                    ).readAsStringSync(),
                  ),
                ),
                onConfirm: c.confirmApproval,
                onCancel: c.discardApproval,
              ),
            ),
          );
          await tester.pumpAndSettle();
          final localized = jsonDecode(
            File(
              '../../packages/presentation/locales/${locale == AppLocale.enUs ? 'en-US' : 'zh-CN'}.json',
            ).readAsStringSync(),
          );
          expect(
            find.text(
              localized['vendor.xiaomi.mijia.switch.toggle.confirm'].replaceAll(
                '{device}',
                'Fictional switch',
              ),
            ),
            findsOneWidget,
          );
          final confirm = find.widgetWithText(
            ActionButton,
            locale.translations.mobileConfirmCommand,
          );
          await tester.ensureVisible(confirm);
          await tester.tap(confirm);
          await tester.pump();
          await tester.tap(confirm);
          await tester.pump();
          expect(next.sent, [same(preview)]);
          next.executionGate!.complete({
            'status': 'unknown',
            'request_id': preview.command['request_id'],
          });
          await tester.pumpAndSettle();
          expect(c.receipt?['status'], 'unknown');
          expect(c.approvalPreview, isNull);
          expect(
            next.requests.where(
              (r) => ['plan', 'execute'].contains(r['method']),
            ),
            isEmpty,
          );
          expect(tester.takeException(), isNull);
          c.dispose();
        },
      );
    }
  }
  test(
    'background discards a prepared power toggle without executing it',
    () async {
      final initial = PowerCore(powerInspection('switch-12')),
          next = PowerCore(powerInspection('switch-12'));
      final c = HomeController(
        initial,
        AccountPreferences(),
        reopen: (p) async {
          next.granted = p;
          return next;
        },
      );
      expect(await c.initialize(), true);
      expect(await c.selectDevice(initial.inspected['device_id']), true);
      expect(await c.prepareApproval(capability, 'toggle', {}, 'cloud'), true);
      final p = c.approvalPreview!;
      c.setForeground(false);
      await Future<void>.delayed(Duration.zero);
      expect(c.approvalPreview, isNull);
      expect(next.discarded, [same(p)]);
      expect(await c.confirmApproval(), false);
      expect(next.sent, isEmpty);
      c.dispose();
    },
  );
}
