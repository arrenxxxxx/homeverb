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
import 'mijia_outlet_test.dart' show outletInspection;
import 'mijia_power_actions_test.dart' show PowerCore;

const cases = [
  ('switch-3', 'state'),
  ('switch-11', 'state'),
  ('switch-12', 'state'),
  ('switch-2', 'state'),
  ('switch-2', 'default_power_on_state_2'),
  ('indicator_light-4', 'on_1'),
  ('custom-7', 'child_lock_7'),
];
Json scopeInspection((String, String) scope) {
  final data = outletInspection(extended: true);
  final entries =
      jsonDecode(
            File(
              '../../examples/native-open.mijia-switch-outlet.json',
            ).readAsStringSync(),
          )['mijia_outlet_grants']
          as List;
  data['device_id'] = entries.firstWhere(
    (e) => e['component_id'] == scope.$1 && e['property'] == scope.$2,
  )['device_id'];
  data['options'] = (data['options'] as List)
      .where(
        (o) =>
            o['component_id'] == scope.$1 &&
            o['definition']['parameters']['properties'].containsKey(scope.$2),
      )
      .toList();
  return data;
}

List<SessionPermission> choices(Json data) =>
    SessionPermission.choices(data, trustedApproval: true);

class TargetPreview implements ApprovalPreview {
  TargetPreview(Json input, this.action)
    : command = Map.unmodifiable({
        ...input,
        'arguments': Map<String, dynamic>.unmodifiable(
          input['arguments'] as Map,
        ),
      });
  @override
  final Json command;
  @override
  final Json action;
  @override
  Json get descriptor => {
    'device': {'id': command['device_id'], 'alias': 'Fictional outlet'},
  };
  @override
  Json get connection => {'id': 'fictional-cloud', 'transport': 'cloud'};
  @override
  Duration get remaining => const Duration(seconds: 60);
  @override
  DateTime get expiresAt => DateTime.now().add(remaining);
}

class TargetCore extends PowerCore {
  TargetCore(super.inspected, {super.minor = 43});
  @override
  Future<ApprovalPreview> prepareApproval(Json command) async {
    final p = TargetPreview(command, inspected['options'][0]['definition']);
    prepared.add(p);
    return p;
  }
}

void main() {
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  test(
    'seven fields retain all fifteen exact targets and independent risk',
    () {
      final all = [
        for (final scope in cases) ...choices(scopeInspection(scope)),
      ];
      expect(all, hasLength(15));
      expect(
        SessionPermission.startup(all)['mijia_outlet_grants'],
        jsonDecode(
          File(
            '../../examples/native-open.mijia-switch-outlet.json',
          ).readAsStringSync(),
        )['mijia_outlet_grants'],
      );
      for (final scope in cases) {
        expect(
          SessionPermission.choices(scopeInspection(scope)).length,
          scope.$1 == 'indicator_light-4' ? 2 : 0,
        );
        expect(
          choices(
            scopeInspection(scope),
          ).every((p) => p.requiresExtendedOutletApi),
          scope.$1 != 'switch-2',
        );
        for (final mutate in <void Function(Json)>[
          (o) => o['component_id'] = 'switch-99',
          (o) => o['definition']['risk'] = 'high',
          (o) => o['definition']['idempotency'] = 'non_idempotent',
          (o) => o['definition']['verification'] = 'state_match',
          (o) => o['definition']['transports'] = ['cloud', 'lan'],
          (o) => o['definition']['parameters']['additionalProperties'] = true,
        ]) {
          final bad = scopeInspection(scope);
          mutate(bad['options'][0]);
          expect(choices(bad), isEmpty);
        }
      }
    },
  );
  test('old inspected tuples cannot cause a granted reopen', () async {
    final core = TargetCore(scopeInspection(cases.first), minor: 42);
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
    expect(await c.selectDevice(core.inspected['device_id']), false);
    expect(
      c.error,
      isA<NativeFailure>().having(
        (e) => e.code,
        'code',
        'API_UPGRADE_REQUIRED',
      ),
    );
    expect(opened, false);
    expect(core.sent, isEmpty);
    c.dispose();
  });
  test('actual API 1.42 peer receives no new startup authority', () async {
    final dir = await Directory.systemTemp.createTemp('homeverb-outlet-old-');
    try {
      final path = '${dir.path}/${NativeClient.defaultLibrary()}';
      final built = await Process.run('cc', [
        Platform.isMacOS ? '-dynamiclib' : '-shared',
        '-fPIC',
        '-std=c11',
        '-D_DEFAULT_SOURCE',
        '-DFIXTURE_API_VERSION="1.42.0"',
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
          permissions: choices(scopeInspection(cases.first)),
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
  });
  test(
    'both current native modes admit all targets after negotiation',
    () async {
      for (final adapter in ['xiaomi.mijia', 'homeverb.platforms']) {
        final dir = await Directory.systemTemp.createTemp(
          'homeverb-outlet-new-',
        );
        NativeClient? client;
        try {
          client = await NativeClient.open(
            dataDir: dir.path,
            adapter: adapter,
            libraryPath: '../../target/debug/${NativeClient.defaultLibrary()}',
            permissions: [
              for (final scope in cases) ...choices(scopeInspection(scope)),
            ],
          );
          expect(client.apiAtLeast(43), true);
          expect(await client.call('list_devices'), isEmpty);
        } finally {
          await client?.close();
          await dir.delete(recursive: true);
        }
      }
    },
  );
  test('late inspection after background cannot reopen or refresh', () async {
    final core = TargetCore(scopeInspection(cases.last))
      ..inspectionGate = Completer<Json>();
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
    final work = c.selectDevice(core.inspected['device_id']);
    await Future<void>.delayed(Duration.zero);
    c.setForeground(false);
    core.inspectionGate!.complete(core.inspected);
    expect(await work, false);
    expect(opened, false);
    expect(core.sent, isEmpty);
    c.dispose();
  });
  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    for (final scope in cases) {
      testWidgets(
        'exact target submits once with required consent $locale/$scope',
        (tester) async {
          LocaleSettings.setLocale(locale);
          tester.view.physicalSize = const Size(640, 1800);
          tester.view.devicePixelRatio = 2;
          tester.platformDispatcher.textScaleFactorTestValue = 2;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
          final initial = TargetCore(scopeInspection(scope)),
              next = TargetCore(scopeInspection(scope));
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
          final option = initial.inspected['options'][0],
              capability = option['capability'] as String,
              action = option['action'] as String;
          final elevated = option['definition']['risk'] == 'elevated';
          final Object value = scope.$2 == 'default_power_on_state_2'
              ? 2
              : true;
          expect(
            next.granted.length,
            scope.$2 == 'default_power_on_state_2' ? 3 : 2,
          );
          expect(next.prepared, isEmpty);
          expect(next.sent, isEmpty);
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
          next.executionGate = Completer<Json>();
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
          final target = find.byKey(ValueKey('parameter:${scope.$2}:$value'));
          await tester.ensureVisible(target);
          await tester.tap(target);
          await tester.pump();
          String request;
          if (elevated) {
            final route = find.byType(ChoiceChip).last;
            await tester.ensureVisible(route);
            await tester.tap(route);
            await tester.pump();
            final review = find.widgetWithText(
              ActionButton,
              locale.translations.mobileReviewCommand,
            );
            await tester.ensureVisible(review);
            await tester.tap(review);
            await tester.pumpAndSettle();
            expect(next.sent, isEmpty);
            final p = c.approvalPreview!;
            expect(p.command['arguments'], {scope.$2: value});
            expect(p.command['component_id'], scope.$1);
            request = p.command['request_id'];
            final labels = Map<String, String>.from(
              jsonDecode(
                File(
                  '../../packages/presentation/locales/en-US.json',
                ).readAsStringSync(),
              ),
            );
            await tester.pumpWidget(
              wrap(
                ApprovalReview(
                  preview: p,
                  labels: labels,
                  onConfirm: c.confirmApproval,
                  onCancel: c.discardApproval,
                ),
              ),
            );
            await tester.pumpAndSettle();
            final localized = Map<String, String>.from(
              jsonDecode(
                File(
                  '../../packages/presentation/locales/${locale == AppLocale.enUs ? 'en-US' : 'zh-CN'}.json',
                ).readAsStringSync(),
              ),
            );
            final key = scope.$2 == 'state'
                ? 'component.${scope.$1.replaceAll('-', '_')}.power.set.confirm'
                : '$capability.$action.confirm';
            final rendered = tester
                .widgetList<Text>(find.byType(Text))
                .map((w) => w.data ?? '')
                .join(' ');
            expect(
              rendered,
              contains(
                localized[key]!.substring(localized[key]!.lastIndexOf('}') + 1),
              ),
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
            expect(next.sent, [same(p)]);
          } else {
            final apply = find.widgetWithText(
              ActionButton,
              locale.translations.actionApply,
            );
            await tester.ensureVisible(apply);
            await tester.tap(apply);
            await tester.pump();
            await tester.tap(apply);
            await tester.pump();
            final requests = next.requests
                .where((r) => r['method'] == 'execute')
                .toList();
            expect(requests, hasLength(1));
            expect(requests.single['command']['arguments'], {scope.$2: value});
            request = requests.single['command']['request_id'];
            expect(next.prepared, isEmpty);
          }
          next.executionGate!.complete({
            'status': 'unknown',
            'request_id': request,
          });
          await tester.pumpAndSettle();
          expect(c.receipt?['status'], 'unknown');
          expect(tester.takeException(), isNull);
          await tester.pumpWidget(const SizedBox());
          c.dispose();
        },
      );
    }
  }
  test('background discards new outlet preview without a send', () async {
    final initial = TargetCore(scopeInspection(cases.last)),
        next = TargetCore(scopeInspection(cases.last));
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
        'vendor.xiaomi.mijia.custom',
        'set_child_lock_7',
        {'child_lock_7': true},
        'cloud',
      ),
      true,
    );
    final p = c.approvalPreview!;
    c.setForeground(false);
    await Future<void>.delayed(Duration.zero);
    expect(next.discarded, [same(p)]);
    expect(await c.confirmApproval(), false);
    expect(next.sent, isEmpty);
    c.dispose();
  });
}
