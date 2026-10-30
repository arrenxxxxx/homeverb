import 'dart:async';
import 'dart:convert';
import 'dart:ffi' hide Size;
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/approval_review.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/controls.dart';
import 'package:homeverb_mobile/device_detail_controls.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/session_permissions.dart';
import 'package:homeverb_mobile/skin.dart';
import 'accounts_test.dart' show AccountPreferences;
import 'session_permissions_test.dart' show inspection;
import 'mijia_actions_test.dart' show actionInspection;
import 'mijia_power_actions_test.dart' show PowerCore, powerInspection;

const actions = [
  'ptz_calibrate',
  'restart_device',
  'pop_up',
  'stop_dry_cleaning',
];
Json maintenanceInspection(String action) {
  final data = inspection();
  final entries =
      jsonDecode(
            File(
              '../../examples/native-open.mijia-maintenance-actions.json',
            ).readAsStringSync(),
          )['mijia_action_grants']
          as List;
  final entry = entries.singleWhere((e) => e['action'] == action);
  data['device_id'] = entry['device_id'];
  data['options'] = [
    {
      'component_id': entry['component_id'],
      'capability': entry['capability'],
      'action': action,
      'definition': jsonDecode(
        File(
          '../../examples/capability.mijia-maintenance-action.json',
        ).readAsStringSync(),
      )['actions']['ptz_calibrate'],
    },
  ];
  return data;
}

List<SessionPermission> choices(Json data) => SessionPermission.choices(
  data,
  trustedApproval: true,
  includeMaintenanceActions: true,
);

class MaintenanceCore extends PowerCore {
  MaintenanceCore(super.inspected, {super.minor = 42, super.trusted});
}

void main() {
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  test(
    'maintenance choices require explicit opt-in and trusted approval with exact elevated metadata',
    () {
      final permissions = <SessionPermission>[];
      for (final action in actions) {
        final data = maintenanceInspection(action);
        expect(
          SessionPermission.choices(
            data,
            trustedApproval: true,
            includeNoArgumentActions: true,
            includePowerToggleActions: true,
          ),
          isEmpty,
        );
        expect(
          SessionPermission.choices(data, includeMaintenanceActions: true),
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
          (o) => o['definition']['atomic'] = true,
          (o) => o['definition']['independent_fields'] = true,
          (o) => o['definition']['parameters']['required'] = ['confirmed'],
          (o) => o['definition']['parameters']['additionalProperties'] = true,
          (o) => o['definition']['parameters']['properties']['confirmed'] = {
            'const': true,
          },
        ]) {
          final bad = maintenanceInspection(action);
          mutate(bad['options'][0]);
          expect(choices(bad), isEmpty);
        }
      }
      expect(permissions, hasLength(4));
      expect(
        permissions.every(
          (p) => p.requiresMaintenanceApi && p.arguments.isEmpty,
        ),
        true,
      );
      expect(SessionPermission.startup([...permissions, ...permissions]), {
        'mijia_action_grants': jsonDecode(
          File(
            '../../examples/native-open.mijia-maintenance-actions.json',
          ).readAsStringSync(),
        )['mijia_action_grants'],
      });
      expect(choices(actionInspection('fan-2')), isEmpty);
      expect(choices(powerInspection('switch-2')), isEmpty);
    },
  );
  test(
    'inspection flags remain independent and unreviewed maintenance is absent',
    () {
      final data = maintenanceInspection('ptz_calibrate');
      data['options'] = [
        ...actionInspection('fan-2')['options'],
        ...powerInspection('switch-2')['options'],
        for (final action in actions)
          ...maintenanceInspection(action)['options'],
      ];
      for (final low in [false, true]) {
        for (final power in [false, true]) {
          for (final maintenance in [false, true]) {
            final selected = SessionPermission.choices(
              data,
              trustedApproval: true,
              includeNoArgumentActions: low,
              includePowerToggleActions: power,
              includeMaintenanceActions: maintenance,
            );
            expect(
              selected.length,
              (low ? 1 : 0) + (power ? 1 : 0) + (maintenance ? 4 : 0),
            );
            expect(
              selected.map((p) => p.identity).toSet().length,
              selected.length,
            );
          }
        }
      }
      for (final action in [
        'format',
        'start_p2p_stream',
        'execute_dry_cleaning',
        'reset',
      ]) {
        for (final admitted in actions) {
          final bad = maintenanceInspection(admitted);
          bad['options'][0]['action'] = action;
          expect(choices(bad), isEmpty);
        }
      }
    },
  );
  test(
    'old versions or absent approval support never opt in or reopen with maintenance authority',
    () async {
      for (final core in [
        MaintenanceCore(maintenanceInspection('ptz_calibrate'), minor: 41),
        MaintenanceCore(maintenanceInspection('ptz_calibrate'), trusted: false),
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
              .every((r) => !r.containsKey('include_maintenance_actions')),
          true,
        );
        expect(core.sent, isEmpty);
        c.dispose();
      }
    },
  );
  test(
    'actual API 1.41 peer is closed before any maintenance-granted opening',
    () async {
      final dir = await Directory.systemTemp.createTemp(
        'homeverb-maintenance-old-',
      );
      try {
        final path = '${dir.path}/${NativeClient.defaultLibrary()}';
        final built = await Process.run('cc', [
          Platform.isMacOS ? '-dynamiclib' : '-shared',
          '-fPIC',
          '-std=c11',
          '-D_DEFAULT_SOURCE',
          '-DFIXTURE_API_VERSION="1.41.0"',
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
            permissions: choices(maintenanceInspection('ptz_calibrate')),
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
    'both rebuilt native modes accept maintenance grants after API 1.42 negotiation',
    () async {
      for (final adapter in ['xiaomi.mijia', 'homeverb.platforms']) {
        final dir = await Directory.systemTemp.createTemp(
          'homeverb-maintenance-new-',
        );
        NativeClient? core;
        try {
          core = await NativeClient.open(
            dataDir: dir.path,
            adapter: adapter,
            libraryPath: '../../target/debug/${NativeClient.defaultLibrary()}',
            permissions: [
              for (final action in actions)
                ...choices(maintenanceInspection(action)),
            ],
          );
          expect(core.apiAtLeast(42), true);
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
    'late maintenance inspection cannot grant or refresh after background',
    () async {
      final core = MaintenanceCore(maintenanceInspection('pop_up'))
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
    for (final action in actions) {
      testWidgets(
        'maintenance review sends once after explicit confirmation $locale/$action',
        (tester) async {
          LocaleSettings.setLocale(locale);
          tester.view.physicalSize = const Size(640, 1800);
          tester.view.devicePixelRatio = 2;
          tester.platformDispatcher.textScaleFactorTestValue = 2;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
          final option = maintenanceInspection(action)['options'][0];
          final component = option['component_id'] as String;
          final capability = option['capability'] as String;
          final initial = MaintenanceCore(maintenanceInspection(action)),
              next = MaintenanceCore(maintenanceInspection(action));
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
                .single['include_maintenance_actions'],
            true,
          );
          expect(next.granted.single.requiresMaintenanceApi, true);
          expect(
            next.requests.where((r) => r['method'] == 'refresh_account'),
            hasLength(1),
          );
          expect(next.prepared, isEmpty);
          expect(next.sent, isEmpty);
          expect(await c.execute(capability, action, {}), false);
          Widget wrap(Widget child) => TranslationProvider(
            child: MaterialApp(
              theme: windowTheme(),
              home: Scaffold(body: SingleChildScrollView(child: child)),
            ),
          );
          String? selected;
          await tester.pumpWidget(
            wrap(
              DeviceDetailControls(
                controller: c,
                onAction: (key) => selected = key,
              ),
            ),
          );
          await tester.pumpAndSettle();
          final button = find.byKey(ValueKey('control:0:$action'));
          await tester.ensureVisible(button);
          await tester.tap(button);
          expect(selected, 'control:0:$action');
          expect(next.prepared, isEmpty);
          expect(next.sent, isEmpty);
          await tester.pumpWidget(
            wrap(
              CapabilityActionForm(
                capability: c.capability(capability)!,
                action: action,
                observation: (_) => null,
                onSubmit: (a) => c.execute(capability, action, a),
                onPrepare: (a, r) =>
                    c.prepareApproval(capability, action, a, r),
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
              localized['$capability.$action.confirm'].replaceAll(
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
    'background discards a prepared maintenance action without executing it',
    () async {
      final initial = MaintenanceCore(maintenanceInspection('restart_device')),
          next = MaintenanceCore(maintenanceInspection('restart_device'));
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
        await c.prepareApproval(
          'vendor.xiaomi.mijia.camera_control',
          'restart_device',
          {},
          'cloud',
        ),
        true,
      );
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
