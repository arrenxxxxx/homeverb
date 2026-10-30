import 'dart:async';
import 'dart:io';
import 'dart:typed_data';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/account_qr.dart';
import 'package:homeverb_mobile/account_views.dart';
import 'package:homeverb_mobile/client_contract.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/credential_access.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'accounts_test.dart'
    show AccountFixture, AccountPreferences, firstAccount, secondAccount;

const scanFixture = 'https://example.invalid/fictional-homeverb-qr';

class QrFixture extends ModernAccountQr {
  QrFixture({this.remaining = const Duration(minutes: 3), int? length})
    : scanData = length == null
          ? Uint8List.fromList(scanFixture.codeUnits)
          : Uint8List(length);
  @override
  final Uint8List scanData;
  @override
  Duration remaining;
  @override
  void clearScan() => scanData.fillRange(0, scanData.length, 0);
}

class ModernFixture extends AccountFixture implements ModernAccountClient {
  ModernFixture() {
    adapter = 'xiaomi.mijia';
    accountProviders = {'xiaomi.mijia'};
    for (final record in records) {
      record['provider'] = 'xiaomi.mijia';
    }
  }
  @override
  bool supportsModernAccountLogin = true;
  final qr = QrFixture();
  final operations = <ModernAccountOperation>[];
  int begins = 0, continuations = 0, discards = 0;
  bool failed = false;
  String? target;
  Completer<void>? beginGate, continueGate, sessionGate;
  ModernAccountResult result(ModernAccountResultKind kind, String id) =>
      ModernAccountResult(
        kind: kind,
        account: {...record(id)},
        directory: kind == ModernAccountResultKind.renewed
            ? null
            : ModernDirectoryResult(
                count: 0,
                failed: failed,
                issues: [],
                error: failed
                    ? {
                        'code': 'AUTH_EXPIRED',
                        'message': 'Fictional failure.',
                        'retryable': false,
                      }
                    : null,
              ),
      );
  @override
  Future<ModernAccountQr> beginModernAccountLogin({
    required String label,
    String? accountId,
  }) async {
    begins++;
    target = accountId;
    if (beginGate != null) await beginGate!.future;
    return qr;
  }

  @override
  Future<ModernAccountResult> continueModernAccountLogin(
    ModernAccountQr pending,
  ) async {
    continuations++;
    if (continueGate != null) await continueGate!.future;
    return result(ModernAccountResultKind.connected, target ?? firstAccount);
  }

  @override
  Future<void> discardModernAccountLogin(ModernAccountQr pending) async {
    discards++;
    pending.clearScan();
  }

  @override
  Future<ModernAccountResult> modernAccountSession(
    String accountId,
    ModernAccountOperation operation,
  ) async {
    operations.add(operation);
    if (sessionGate != null) await sessionGate!.future;
    return result(
      operation == ModernAccountOperation.renew
          ? ModernAccountResultKind.renewed
          : ModernAccountResultKind.directory,
      accountId,
    );
  }
}

class ReadinessFixture implements CredentialAccess {
  int checks = 0;
  Completer<CredentialReadiness>? gate;
  @override
  Future<CredentialReadiness> check() async {
    checks++;
    return gate == null ? CredentialReadiness.ready : await gate!.future;
  }

  @override
  Future<bool> openSecuritySettings() async => false;
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  for (final ending in [
    'continue',
    'cancel',
    'back',
    'expiry',
    'background',
    'navigation',
  ]) {
    testWidgets('QR $ending clears bytes and rendered modules', (tester) async {
      final challenge = QrFixture(remaining: const Duration(seconds: 3));
      final cancellation = ChangeNotifier();
      addTearDown(cancellation.dispose);
      bool? result;
      var closed = false;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => TextButton(
              onPressed: () async {
                result = await showDialog<bool>(
                  context: context,
                  builder: (_) => AccountQrPrompt(
                    challenge: challenge,
                    cancellation: cancellation,
                  ),
                );
                closed = true;
              },
              child: const Text('Open'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();
      final paint =
          tester
                  .widget<CustomPaint>(find.byKey(const ValueKey('account-qr')))
                  .painter!
              as AccountQrPainter;
      expect(challenge.scanData.every((b) => b == 0), true);
      expect(paint.bitmap.modules.contains(1), true);
      expect(result, isNull);
      if (ending == 'continue') {
        await tester.tap(find.byKey(const ValueKey('qr-continue')));
      } else if (ending == 'cancel') {
        await tester.tap(find.text('Cancel'));
      } else if (ending == 'back') {
        await tester.binding.handlePopRoute();
      } else if (ending == 'expiry') {
        await tester.pump(const Duration(seconds: 3));
      } else if (ending == 'background') {
        tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      } else {
        cancellation.notifyListeners();
      }
      await tester.pumpAndSettle();
      expect(closed, true);
      expect(result == true, ending == 'continue');
      expect(paint.bitmap.modules.every((b) => b == 0), true);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('account-qr')), findsNothing);
      expect(tester.takeException(), isNull);
    });
  }
  for (final locale in AppLocale.values) {
    testWidgets('QR narrow enlarged ${locale.languageCode} layout', (
      tester,
    ) async {
      LocaleSettings.setLocale(locale);
      tester.view.physicalSize = const Size(640, 1600);
      tester.view.devicePixelRatio = 2;
      tester.platformDispatcher.textScaleFactorTestValue = 2;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
      await tester.pumpWidget(
        TranslationProvider(
          child: MaterialApp(home: AccountQrPrompt(challenge: QrFixture())),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('account-qr')), findsOneWidget);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    });
  }
  testWidgets('oversized QR fails closed without protected error text', (
    tester,
  ) async {
    final challenge = QrFixture(length: 8192);
    await tester.pumpWidget(
      MaterialApp(home: AccountQrPrompt(challenge: challenge)),
    );
    await tester.pumpAndSettle();
    expect(
      tester
          .widget<TextButton>(find.byKey(const ValueKey('qr-continue')))
          .onPressed,
      isNull,
    );
    expect(find.byKey(const ValueKey('account-qr')), findsNothing);
    expect(challenge.scanData.every((b) => b == 0), true);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
  });
  testWidgets('paint fictional QR artifact for independent decoder', (
    tester,
  ) async {
    final bitmap = AccountQrBitmap(Uint8List.fromList(scanFixture.codeUnits));
    final key = GlobalKey();
    await tester.pumpWidget(
      MaterialApp(
        home: Center(
          child: RepaintBoundary(
            key: key,
            child: SizedBox.square(
              dimension: 280,
              child: CustomPaint(painter: AccountQrPainter(bitmap)),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    if (Platform.environment['HOMEVERB_QR_FIXTURE_PNG']
        case final String path) {
      final boundary =
          key.currentContext!.findRenderObject()! as RenderRepaintBoundary;
      await tester.runAsync(() async {
        final image = await boundary.toImage(pixelRatio: 2);
        final bytes = (await image.toByteData(format: ui.ImageByteFormat.png))!;
        await File(path).writeAsBytes(bytes.buffer.asUint8List());
        image.dispose();
      });
    }
    bitmap.clear();
  });
  for (final ending in ['success', 'navigation']) {
    testWidgets('App QR entry $ending preserves existing cards', (
      tester,
    ) async {
      final core = ModernFixture();
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
      expect(c.error, isNull);
      expect(core.begins, 1);
      expect(core.continuations, 0);
      expect(core.operations, isEmpty);
      expect(find.byKey(const ValueKey('account-qr')), findsOneWidget);
      if (ending == 'success') {
        await tester.tap(find.byKey(const ValueKey('qr-continue')));
      } else {
        await key.currentState!.go('P01');
      }
      await tester.pumpAndSettle();
      expect(core.continuations, ending == 'success' ? 1 : 0);
      expect(core.discards, 1);
      expect(core.qr.scanData.every((b) => b == 0), true);
      expect(c.cards.length, 2);
      expect(c.accountReport, isNull);
      if (ending == 'success') {
        expect(key.currentState!.id, 'P09');
        expect(find.textContaining('QR directory entries: 0.'), findsOneWidget);
        final section = find.text('QR connection');
        await tester.ensureVisible(section);
        await tester.tap(section);
        await tester.pumpAndSettle();
        for (final label in [
          'Refresh QR Connection',
          'Recover QR Authorization',
          'Renew QR Authorization',
        ]) {
          final action = find.text(label);
          await tester.ensureVisible(action);
          await tester.tap(action);
          await tester.pumpAndSettle();
          expect(c.error, isNull);
        }
        expect(core.operations, [
          ModernAccountOperation.refresh,
          ModernAccountOperation.recover,
          ModernAccountOperation.renew,
        ]);
        expect(key.currentState!.id, 'P09');
      } else {
        expect(c.modernAccountReport, isNull);
        expect(c.accountNeedsReload, true);
      }
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    });
  }
  for (final outcome in ['locked', 'background', 'ready']) {
    testWidgets('QR repeats credential readiness before dispatch: $outcome', (
      tester,
    ) async {
      final access = ReadinessFixture();
      var calls = 0;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: AccountSignIn(
                onSubmit: (_, _) async {},
                onCancel: () {},
                onReload: () {},
                needsReload: false,
                busy: false,
                onQr: () async {
                  calls++;
                },
                credentialAccess: access,
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      access.gate = Completer<CredentialReadiness>();
      final start = find.text('Sign In with QR Code');
      await tester.ensureVisible(start);
      await tester.tap(start);
      await tester.pump();
      expect(access.checks, 2);
      if (outcome == 'background')
        tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      access.gate!.complete(
        outcome == 'locked'
            ? CredentialReadiness.locked
            : CredentialReadiness.ready,
      );
      await tester.pumpAndSettle();
      expect(calls, outcome == 'ready' ? 1 : 0);
      await tester.pumpWidget(const SizedBox());
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    });
  }
  test('old native client does not offer QR', () {
    final core = ModernFixture()..supportsModernAccountLogin = false;
    final c = HomeController(core, AccountPreferences());
    expect(c.canQrSignIn, false);
    c.dispose();
  });
  for (final point in ['begin', 'continue', 'session']) {
    for (final ending in ['background', 'navigation', 'selection']) {
      test('late $point result after $ending is not adopted', () async {
        final core = ModernFixture();
        final gate = Completer<void>();
        if (point == 'begin') core.beginGate = gate;
        if (point == 'continue') core.continueGate = gate;
        if (point == 'session') core.sessionGate = gate;
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
        expect(c.modernAccountReport, isNull);
        expect(c.accountNeedsReload, true);
        if (point == 'begin') expect(core.continuations, 0);
        if (point != 'session') expect(core.discards, 1);
        c.dispose();
      });
    }
  }
  test(
    'directory failures and zero counts do not cause renewal or legacy refresh',
    () async {
      final core = ModernFixture();
      final c = HomeController(core, AccountPreferences());
      await c.selectAccount(firstAccount);
      for (final failed in [false, true]) {
        core.failed = failed;
        expect(
          await c.modernAccountSession(ModernAccountOperation.refresh),
          true,
        );
        expect(c.modernAccountReport!.directory!.failed, failed);
        expect(
          accountBlocks(c, 'P09').toString(),
          contains(
            failed ? 'could not be retrieved' : 'QR directory entries: 0.',
          ),
        );
      }
      for (final op in [
        ModernAccountOperation.recover,
        ModernAccountOperation.renew,
      ]) {
        expect(await c.modernAccountSession(op), true);
      }
      expect(core.operations, [
        ModernAccountOperation.refresh,
        ModernAccountOperation.refresh,
        ModernAccountOperation.recover,
        ModernAccountOperation.renew,
      ]);
      expect(
        core.requests.where(
          (r) => ['refresh_account', 'recover_account'].contains(r['method']),
        ),
        isEmpty,
      );
      expect(c.cards, hasLength(2));
      await c.selectAccount(secondAccount);
      expect(c.modernAccountReport, isNull);
      c.dispose();
    },
  );
}
