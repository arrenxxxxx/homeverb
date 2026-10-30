import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/account_views.dart';
import 'package:homeverb_mobile/client_contract.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'account_qr_test.dart' show ModernFixture;
import 'accounts_test.dart'
    show AccountPreferences, firstAccount, secondAccount;
import 'controls_test.dart' show capability;

const qrDeviceId = '40000000-0000-4000-8000-000000000001';

class DevicePublicationFixture extends ModernFixture {
  bool empty = false, failReload = false, resultSeen = false;
  Completer<void>? reloadGate;
  final device = <String, dynamic>{
    'id': qrDeviceId,
    'alias': 'QR Fan',
    'model': 'Fictional Fan',
    'support_status': 'experimental',
    'components': [
      {
        'id': 'fan-2',
        'name': 'Fan',
        'capabilities': ['power', 'vendor.xiaomi.mijia.fan'],
      },
    ],
    'connections': [
      {
        'id': '40000000-0000-4000-8000-000000000002',
        'adapter': 'xiaomi.mijia',
        'transport': 'cloud',
        'account_id': firstAccount,
      },
    ],
  };
  @override
  ModernAccountResult result(ModernAccountResultKind kind, String id) {
    if (kind == ModernAccountResultKind.renewed) return super.result(kind, id);
    if (!empty && !failed && !links[id]!.contains(qrDeviceId)) {
      links[id]!.add(qrDeviceId);
      cards.add({
        'device': device,
        'preferences': {'alias': null, 'room_id': 'study', 'favorite': false},
      });
    }
    resultSeen = true;
    final account = {...record(id)};
    return ModernAccountResult(
      kind: kind == ModernAccountResultKind.connected
          ? ModernAccountResultKind.connectedDevices
          : ModernAccountResultKind.devices,
      account: account,
      deviceReport: {
        'account': account,
        'authentication': kind == ModernAccountResultKind.connected
            ? 'succeeded'
            : 'not_attempted',
        'retrieval': {
          'status': failed ? 'failed' : 'partial',
          'device_ids': failed || empty ? <String>[] : [qrDeviceId],
          'error': {
            'code': failed ? 'MIJIA_REQUEST_FAILED' : 'MIJIA_INVENTORY_PARTIAL',
            'message': 'Fictional coverage.',
            'retryable': false,
          },
        },
      },
    );
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'list_accounts' && resultSeen) {
      if (reloadGate != null) await reloadGate!.future;
      if (failReload)
        throw NativeFailure('STORAGE_ERROR', 'Fictional local failure.');
    }
    if (method == 'describe_device' && fields['device'] == qrDeviceId) {
      requests.add({'method': method, ...fields});
      return {
        'device': device,
        'capabilities': [
          capability('power', {
            'state': {'type': 'boolean', 'readable': true, 'writable': false},
          }, {}),
          capability('vendor.xiaomi.mijia.fan', {
            'level': {'type': 'integer', 'readable': true, 'writable': false},
          }, {}),
        ],
      };
    }
    if (method == 'read_state' && fields['device'] == qrDeviceId) {
      requests.add({'method': method, ...fields});
      if (fields['cached'] == true)
        throw NativeFailure('CACHE_MISS', 'No saved reading.');
      return {
        'schema_version': '1.0',
        'device_id': qrDeviceId,
        'component_id': 'fan-2',
        'observations': {
          for (final entry in {
            'power.state': false,
            'vendor.xiaomi.mijia.fan.level': 3,
          }.entries)
            entry.key: {
              'value': entry.value,
              'source': 'cloud',
              'observed_at': null,
              'received_at': '2026-10-30T00:00:00Z',
              'stale': true,
            },
        },
      };
    }
    return super.call(method, fields);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  testWidgets(
    'QR continuation imports cards and actual card taps use the Core state interface',
    (tester) async {
      final core = DevicePublicationFixture();
      final c = HomeController(core, AccountPreferences());
      final key = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: key, catalog: catalog, controller: c),
      );
      await tester.pumpAndSettle();
      await c.selectAccount(firstAccount);
      await key.currentState!.go('P04');
      await tester.pumpAndSettle();
      final start = find.text('Sign In with QR Code');
      await tester.ensureVisible(start);
      await tester.tap(start);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));
      await tester.tap(find.byKey(const ValueKey('qr-continue')));
      await tester.pumpAndSettle();
      expect(c.error, isNull);
      expect(key.currentState!.id, 'P09');
      expect(c.modernAccountReport, isNull);
      expect(c.accountReport!['retrieval']['device_ids'], [qrDeviceId]);
      expect(c.cards.length, 3);
      expect(find.textContaining('Retrieved devices: 1.'), findsOneWidget);
      expect(find.textContaining('not yet imported'), findsNothing);
      final card = find.text('QR Fan');
      await tester.ensureVisible(card);
      await tester.tap(card);
      await tester.pumpAndSettle();
      expect(c.error, isNull);
      expect(key.currentState!.id, 'C01');
      expect(c.selectedDevice, qrDeviceId);
      final refresh = find.text('Refresh State');
      await tester.ensureVisible(refresh);
      await tester.tap(refresh);
      await tester.pumpAndSettle();
      expect(c.error, isNull);
      expect(c.observed('power.state'), false);
      expect(c.snapshot!['observations']['power.state']['stale'], true);
      expect(
        core.requests.where(
          (r) => r['method'] == 'read_state' && r['cached'] == false,
        ),
        isNotEmpty,
      );
      expect(
        core.requests.where(
          (r) => [
            'execute',
            'plan',
            'refresh_account',
            'recover_account',
          ].contains(r['method']),
        ),
        isEmpty,
      );
      expect(core.begins, 1);
      expect(core.continuations, 1);
      expect(core.operations, isEmpty);
      expect(core.qr.scanData.every((v) => v == 0), true);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );

  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    for (final failed in [false, true]) {
      testWidgets(
        'empty/failed device report remains accurate at narrow enlarged $locale failed=$failed',
        (tester) async {
          LocaleSettings.setLocale(locale);
          tester.view.physicalSize = const Size(320, 1100);
          tester.view.devicePixelRatio = 1;
          tester.platformDispatcher.textScaleFactorTestValue = 2;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
          final core = DevicePublicationFixture()
            ..empty = true
            ..failed = failed;
          final c = HomeController(core, AccountPreferences());
          final key = GlobalKey<NativeAppState>();
          final catalog = (await tester.runAsync(ScreenCatalog.load))!;
          await tester.pumpWidget(
            TranslationProvider(
              child: NativeApp(key: key, catalog: catalog, controller: c),
            ),
          );
          await tester.pumpAndSettle();
          await c.selectAccount(firstAccount);
          expect(await c.connectModernAccount('', (_) async => true), true);
          await key.currentState!.go('P09');
          await tester.pumpAndSettle();
          expect(
            c.accountReport!['retrieval']['status'],
            failed ? 'failed' : 'partial',
          );
          expect(c.accountReport!['retrieval']['device_ids'], isEmpty);
          expect(c.modernAccountReport, isNull);
          expect(c.cards.length, 2);
          expect(
            accountBlocks(c, 'P09').toString(),
            isNot(contains('QR directory')),
          );
          expect(core.operations, isEmpty);
          expect(tester.takeException(), isNull);
          await tester.pumpWidget(const SizedBox());
        },
      );
    }
  }

  test(
    'explicit device refresh/recovery publish once and renewal keeps device state unchanged',
    () async {
      final core = DevicePublicationFixture();
      final c = HomeController(core, AccountPreferences());
      await c.selectAccount(firstAccount);
      for (final action in [
        ModernAccountOperation.refresh,
        ModernAccountOperation.recover,
      ]) {
        expect(await c.modernAccountSession(action), true);
        expect(c.accountReport!['retrieval']['device_ids'], [qrDeviceId]);
        expect(c.modernAccountReport, isNull);
        expect(c.cards.length, 3);
      }
      expect(await c.modernAccountSession(ModernAccountOperation.renew), true);
      expect(c.modernAccountReport!.kind, ModernAccountResultKind.renewed);
      expect(c.accountReport, isNull);
      expect(c.cards.length, 3);
      expect(core.operations, ModernAccountOperation.values);
      expect(core.begins, 0);
      expect(
        core.requests.where(
          (r) => ['refresh_account', 'recover_account'].contains(r['method']),
        ),
        isEmpty,
      );
      c.dispose();
    },
  );

  for (final point in ['continue', 'session', 'reload']) {
    for (final ending in ['background', 'navigation', 'selection']) {
      test(
        'device publication after $point $ending cannot restore reports',
        () async {
          final core = DevicePublicationFixture();
          final gate = Completer<void>();
          if (point == 'continue') core.continueGate = gate;
          if (point == 'session') core.sessionGate = gate;
          if (point == 'reload') core.reloadGate = gate;
          final c = HomeController(core, AccountPreferences());
          await c.selectAccount(firstAccount);
          final work = point == 'session'
              ? c.modernAccountSession(ModernAccountOperation.refresh)
              : c.connectModernAccount('', (_) async => true);
          await Future<void>.delayed(Duration.zero);
          if (ending == 'background') c.setForeground(false);
          if (ending == 'navigation') c.cancelPending();
          if (ending == 'selection') c.selectedAccount = secondAccount;
          gate.complete();
          expect(await work, false);
          expect(c.accountReport, isNull);
          expect(c.modernAccountReport, isNull);
          expect(c.accountNeedsReload, true);
          if (ending == 'selection') expect(c.selectedAccount, secondAccount);
          c.dispose();
        },
      );
    }
  }

  test(
    'local reload failure retains reload requirement without retrying completed login',
    () async {
      final core = DevicePublicationFixture()..failReload = true;
      final c = HomeController(core, AccountPreferences());
      await c.selectAccount(firstAccount);
      expect(await c.connectModernAccount('', (_) async => true), false);
      expect(c.accountReport, isNull);
      expect(c.modernAccountReport, isNull);
      expect(c.accountNeedsReload, true);
      core.failReload = false;
      expect(await c.loadAccounts(), true);
      expect(c.cards.length, 3);
      expect(c.accountNeedsReload, false);
      expect(core.begins, 1);
      expect(core.continuations, 1);
      expect(core.operations, isEmpty);
      c.dispose();
    },
  );
}
