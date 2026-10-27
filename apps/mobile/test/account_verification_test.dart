import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/account_verification.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'account_captcha_test.dart' show InteractiveFixture;
import 'accounts_test.dart' show AccountPreferences, firstAccount;

class VerificationFixture extends AccountVerificationPending {
  VerificationFixture({
    this.kind = AccountVerificationKind.verification,
    this.methods = const [
      AccountVerificationMethod.phone,
      AccountVerificationMethod.email,
    ],
    this.selectedMethod,
    this.codeRequests = 0,
    this.cooldown = Duration.zero,
    this.remaining = const Duration(minutes: 3),
  });
  @override
  final AccountVerificationKind kind;
  @override
  final List<AccountVerificationMethod> methods;
  @override
  final AccountVerificationMethod? selectedMethod;
  @override
  int codeRequests;
  @override
  Duration cooldown, remaining;
  @override
  final image = base64Decode(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jP1sAAAAASUVORK5CYII=',
  );
  @override
  void clearImage() => image.fillRange(0, image.length, 0);
}

class VerificationClientFixture extends InteractiveFixture
    implements AccountVerificationClient {
  @override
  bool supportsAccountVerification = true;
  final verificationPending = <VerificationFixture>{};
  final verificationPrompts = <VerificationFixture>[];
  final sentMethods = <AccountVerificationMethod>[];
  Completer<void>? sendGate;
  bool startWithCaptcha = true;
  VerificationFixture prompt({
    bool captcha = false,
    AccountVerificationMethod? selected,
  }) {
    final result = VerificationFixture(
      kind: captcha
          ? AccountVerificationKind.captcha
          : AccountVerificationKind.verification,
      methods: captcha ? [] : AccountVerificationMethod.values,
      selectedMethod: selected,
      codeRequests: sentMethods.length,
      cooldown: sentMethods.isEmpty
          ? Duration.zero
          : const Duration(seconds: 60),
    );
    verificationPrompts.add(result);
    verificationPending.add(result);
    return result;
  }

  @override
  Future<AccountLoginProgress> beginAccountVerification({
    required Uint8List login,
    required Uint8List password,
    required String label,
    String? accountId,
  }) async {
    target = accountId;
    ownedLogin = login;
    ownedPassword = password;
    login.fillRange(0, login.length, 0);
    password.fillRange(0, password.length, 0);
    return prompt(captcha: startWithCaptcha);
  }

  @override
  Future<AccountLoginProgress> answerAccountVerification(
    AccountVerificationPending challenge,
    Uint8List answer, {
    AccountVerificationMethod? method,
  }) async {
    expect(verificationPending.remove(challenge), true);
    buffers.add(answer);
    answers.add(utf8.decode(answer));
    if (challenge.kind == AccountVerificationKind.captcha) {
      expect(method, isNull);
      return prompt();
    }
    expect(method, sentMethods.last);
    return AccountLoginConnected(report(target ?? firstAccount, 'succeeded'));
  }

  @override
  Future<AccountLoginProgress> requestAccountVerificationCode(
    AccountVerificationPending challenge,
    AccountVerificationMethod method,
  ) async {
    expect(verificationPending.remove(challenge), true);
    sentMethods.add(method);
    if (sendGate != null) await sendGate!.future;
    return prompt(selected: method);
  }

  @override
  Future<void> discardAccountVerification(
    AccountVerificationPending challenge,
  ) async {
    verificationPending.remove(challenge);
    challenge.clearImage();
  }
}

Future<bool> connect(
  HomeController c,
  Future<AccountVerificationReply?> Function(AccountVerificationPending)
  callback,
) => c.connectAccount(
  Uint8List.fromList('fictional'.codeUnits),
  Uint8List.fromList('Fictional'.codeUnits),
  'Home',
  onVerification: callback,
);

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
  });
  for (final method in AccountVerificationMethod.values) {
    test(
      'protected captcha and explicit ${method.name} delivery preserve partial inventory',
      () async {
        final core = VerificationClientFixture()
          ..retrieval = 'partial'
          ..retrievalError = 'MIJIA_INVENTORY_PARTIAL';
        final c = HomeController(core, AccountPreferences());
        await c.selectAccount(firstAccount);
        expect(
          await connect(c, (challenge) async {
            expect(core.ownedPassword!.every((b) => b == 0), true);
            if (challenge.kind == AccountVerificationKind.captcha) {
              expect(core.sentMethods, isEmpty);
              return AccountVerificationReply.answer(
                Uint8List.fromList('Ab12'.codeUnits),
              );
            }
            if (challenge.codeRequests == 0)
              return AccountVerificationReply.send(method);
            expect(challenge.selectedMethod, method);
            return AccountVerificationReply.answer(
              Uint8List.fromList('123456'.codeUnits),
              method: method,
            );
          }),
          true,
        );
        expect(core.target, firstAccount);
        expect(core.sentMethods, [method]);
        expect(core.answers, ['Ab12', '123456']);
        expect(core.verificationPending, isEmpty);
        expect(
          core.verificationPrompts.every((p) => p.image.every((b) => b == 0)),
          true,
        );
        expect(core.buffers.every((p) => p.every((b) => b == 0)), true);
        expect(c.accountReport!['retrieval']['status'], 'partial');
        c.dispose();
      },
    );
  }
  for (final reason in [
    'cancel',
    'navigation',
    'background',
    'expired',
    'cooldown',
    'limit',
  ]) {
    test('$reason prevents code dispatch', () async {
      final core = VerificationClientFixture()..startWithCaptcha = false;
      final c = HomeController(core, AccountPreferences());
      expect(
        await connect(c, (challenge) async {
          if (reason == 'cancel') return null;
          if (reason == 'navigation') c.cancelPending();
          if (reason == 'background') c.setForeground(false);
          if (reason == 'expired')
            (challenge as VerificationFixture).remaining = Duration.zero;
          if (reason == 'cooldown')
            (challenge as VerificationFixture).cooldown = const Duration(
              seconds: 1,
            );
          if (reason == 'limit')
            (challenge as VerificationFixture).codeRequests = 3;
          return AccountVerificationReply.send(AccountVerificationMethod.phone);
        }),
        false,
      );
      expect(core.sentMethods, isEmpty);
      expect(core.verificationPending, isEmpty);
      expect(c.accountReport, isNull);
      c.dispose();
    });
  }
  test('late code response is discarded after cancellation', () async {
    final core = VerificationClientFixture()
      ..startWithCaptcha = false
      ..sendGate = Completer<void>();
    final c = HomeController(core, AccountPreferences());
    final work = connect(
      c,
      (_) async =>
          AccountVerificationReply.send(AccountVerificationMethod.email),
    );
    while (core.sentMethods.isEmpty) {
      await Future<void>.delayed(Duration.zero);
    }
    c.cancelPending();
    core.sendGate!.complete();
    expect(await work, false);
    expect(core.sentMethods, [AccountVerificationMethod.email]);
    expect(core.verificationPending, isEmpty);
    expect(c.accountReport, isNull);
    c.dispose();
  });
  test('older APIs preserve captcha and one-shot fallbacks', () async {
    for (final interactive in [true, false]) {
      final core = VerificationClientFixture()
        ..supportsAccountVerification = false
        ..supportsInteractiveLogin = interactive;
      final c = HomeController(core, AccountPreferences());
      expect(
        await c.connectAccount(
          Uint8List.fromList('fictional'.codeUnits),
          Uint8List.fromList('Fictional'.codeUnits),
          'Home',
          onVerification: (_) async =>
              fail('Old APIs must not resolve new symbols.'),
          onChallenge: (_) async => Uint8List.fromList('Good'.codeUnits),
        ),
        true,
      );
      expect(core.verificationPrompts, isEmpty);
      expect(core.challenges.length, interactive ? 1 : 0);
      c.dispose();
    }
  });
  test(
    'actual FFI refuses old API, background and closed sessions without traffic',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'homeverb-verification-refusal-',
      );
      final client = await NativeClient.open(
        dataDir: directory.path,
        libraryPath: File(
          '../../target/debug/${NativeClient.defaultLibrary()}',
        ).absolute.path,
        adapter: 'xiaomi.mijia',
      );
      try {
        expect(client.supportsAccountVerification, true);
        for (final state in ['old', 'background', 'closed']) {
          client.apiVersion = state == 'old' ? '1.31.0' : '1.32.0';
          if (state == 'background') client.foreground(false);
          if (state == 'closed') await client.close();
          final login = Uint8List.fromList('fictional'.codeUnits),
              password = Uint8List.fromList('Fictional'.codeUnits);
          await expectLater(
            client.beginAccountVerification(
              login: login,
              password: password,
              label: 'Home',
            ),
            throwsA(
              isA<NativeFailure>().having(
                (e) => e.code,
                'code',
                {
                  'old': 'API_UPGRADE_REQUIRED',
                  'background': 'HOST_BACKGROUND',
                  'closed': 'HOST_CLOSED',
                }[state],
              ),
            ),
          );
          expect(login.every((b) => b == 0), true);
          expect(password.every((b) => b == 0), true);
        }
      } finally {
        await client.close();
        await directory.delete(recursive: true);
      }
    },
  );
  testWidgets(
    'App completes captcha, explicit email request and code submission',
    (tester) async {
      final core = VerificationClientFixture();
      final c = HomeController(core, AccountPreferences());
      final key = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: key, catalog: catalog, controller: c),
      );
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));
      await c.selectAccount(firstAccount);
      await key.currentState!.go('P04');
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));
      await tester.enterText(
        find.byKey(const ValueKey('credential:Mijia account')),
        'fixture@example.invalid',
      );
      await tester.enterText(
        find.byKey(const ValueKey('credential:Password')),
        'Fictional',
      );
      final submit = find.text('Sign In and Retrieve Devices');
      await tester.ensureVisible(submit);
      await tester.tap(submit);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));
      await tester.enterText(
        find.byKey(const ValueKey('captcha-answer')),
        'Ab12',
      );
      await tester.tap(find.text('Verify and Continue'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));
      expect(core.sentMethods, isEmpty);
      final send = find.byKey(const ValueKey('verification-send'));
      expect(tester.widget<TextButton>(send).onPressed, isNull);
      await tester.tap(find.byKey(const ValueKey('verification-method:email')));
      await tester.pump();
      await tester.tap(send);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));
      expect(core.sentMethods, [AccountVerificationMethod.email]);
      expect(tester.widget<TextButton>(send).onPressed, isNull);
      await tester.enterText(
        find.byKey(const ValueKey('verification-answer')),
        '123456',
      );
      await tester.tap(find.text('Verify and Continue'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));
      expect(key.currentState!.id, 'P09');
      expect(c.error, isNull);
      expect(core.verificationPending, isEmpty);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );
  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    testWidgets(
      'narrow enlarged ${locale.languageCode} prompt validates input and honors limits',
      (tester) async {
        LocaleSettings.setLocale(locale);
        tester.view.physicalSize = const Size(640, 1600);
        tester.view.devicePixelRatio = 2;
        tester.platformDispatcher.textScaleFactorTestValue = 2;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
        final challenge = VerificationFixture(
          methods: [AccountVerificationMethod.phone],
          codeRequests: 3,
        );
        AccountVerificationReply? result;
        await tester.pumpWidget(
          TranslationProvider(
            child: MaterialApp(
              home: Builder(
                builder: (context) => TextButton(
                  onPressed: () async {
                    result = await showDialog<AccountVerificationReply>(
                      context: context,
                      builder: (_) =>
                          AccountVerificationPrompt(challenge: challenge),
                    );
                  },
                  child: const Text('Open'),
                ),
              ),
            ),
          ),
        );
        await tester.tap(find.text('Open'));
        await tester.pumpAndSettle();
        expect(
          tester
              .widget<TextButton>(
                find.byKey(const ValueKey('verification-send')),
              )
              .onPressed,
          isNull,
        );
        final submit = find.text(locale.translations.mobileCaptchaContinue);
        for (final value in [
          '',
          '12 34',
          '12${String.fromCharCode(133)}34',
          String.fromCharCode(233) * 1025,
        ]) {
          await tester.enterText(
            find.byKey(const ValueKey('verification-answer')),
            value,
          );
          await tester.ensureVisible(submit);
          await tester.tap(submit);
          await tester.pumpAndSettle();
          expect(
            find.byKey(const ValueKey('verification-answer')),
            findsOneWidget,
          );
        }
        await tester.enterText(
          find.byKey(const ValueKey('verification-answer')),
          '123456',
        );
        await tester.ensureVisible(submit);
        await tester.tap(submit);
        await tester.pumpAndSettle();
        expect(utf8.decode(result!.answer!), '123456');
        expect(result!.method, AccountVerificationMethod.phone);
        result!.clear();
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
      },
    );
  }
  testWidgets(
    'cooldown only enables an explicit send and never sends automatically',
    (tester) async {
      final challenge = VerificationFixture(
        methods: [AccountVerificationMethod.email],
        selectedMethod: AccountVerificationMethod.email,
        codeRequests: 1,
        cooldown: const Duration(seconds: 1),
      );
      AccountVerificationReply? result;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => TextButton(
              onPressed: () async {
                result = await showDialog<AccountVerificationReply>(
                  context: context,
                  builder: (_) =>
                      AccountVerificationPrompt(challenge: challenge),
                );
              },
              child: const Text('Open'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();
      final send = find.byKey(const ValueKey('verification-send'));
      expect(tester.widget<TextButton>(send).onPressed, isNull);
      challenge.cooldown = Duration.zero;
      await tester.pump(const Duration(seconds: 1));
      expect(tester.widget<TextButton>(send).onPressed, isNotNull);
      expect(result, isNull);
      await tester.tap(send);
      await tester.pumpAndSettle();
      expect(result!.requestsCode, true);
      expect(result!.method, AccountVerificationMethod.email);
      await tester.pumpWidget(const SizedBox());
    },
  );
  for (final event in ['background', 'navigation', 'expiry', 'back']) {
    testWidgets('$event closes prompt and clears image and input', (
      tester,
    ) async {
      final challenge = VerificationFixture(
        remaining: const Duration(seconds: 10),
      );
      final cancellation = ChangeNotifier();
      var completed = false;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => TextButton(
              onPressed: () async {
                final result = await showDialog<AccountVerificationReply>(
                  context: context,
                  builder: (_) => AccountVerificationPrompt(
                    challenge: challenge,
                    cancellation: cancellation,
                  ),
                );
                expect(result, isNull);
                completed = true;
              },
              child: const Text('Open'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const ValueKey('verification-answer')),
        '123456',
      );
      if (event == 'background')
        tester.binding.handleAppLifecycleStateChanged(
          AppLifecycleState.inactive,
        );
      if (event == 'navigation') cancellation.notifyListeners();
      if (event == 'expiry') await tester.pump(const Duration(seconds: 11));
      if (event == 'back') await tester.binding.handlePopRoute();
      await tester.pumpAndSettle();
      expect(completed, true);
      expect(find.byKey(const ValueKey('verification-answer')), findsNothing);
      expect(challenge.image.every((b) => b == 0), true);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpWidget(const SizedBox());
      cancellation.dispose();
    });
  }
}
