import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/account_captcha.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'accounts_test.dart'
    show AccountFixture, AccountPreferences, firstAccount;

class CaptchaFixture extends AccountLoginCaptcha {
  CaptchaFixture({this.remaining = const Duration(minutes: 3)});
  @override
  final image = base64Decode(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jP1sAAAAASUVORK5CYII=',
  );
  @override
  Duration remaining;
  @override
  void clearImage() => image.fillRange(0, image.length, 0);
}

class InteractiveFixture extends AccountFixture
    implements InteractiveAccountClient {
  InteractiveFixture() {
    adapter = 'xiaomi.mijia';
    accountProviders = {'xiaomi.mijia'};
    for (final record in records) {
      record['provider'] = 'xiaomi.mijia';
    }
  }
  @override
  bool supportsInteractiveLogin = true;
  final challenges = <CaptchaFixture>[];
  final discarded = <AccountLoginCaptcha>[];
  final answers = <String>[];
  final pending = <AccountLoginCaptcha>{};
  final buffers = <Uint8List>[];
  int wrongAnswers = 0;
  String? target;
  Completer<void>? answerGate;
  CaptchaFixture next() {
    final result = CaptchaFixture();
    challenges.add(result);
    pending.add(result);
    return result;
  }

  @override
  Future<AccountLoginProgress> beginAccountLogin({
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
    return next();
  }

  @override
  Future<AccountLoginProgress> continueAccountLogin(
    AccountLoginCaptcha challenge,
    Uint8List answer,
  ) async {
    expect(pending.remove(challenge), true);
    answers.add(String.fromCharCodes(answer));
    buffers.add(answer);
    if (answerGate != null) await answerGate!.future;
    if (wrongAnswers-- > 0) return next();
    return AccountLoginConnected(report(target ?? firstAccount, 'succeeded'));
  }

  @override
  Future<void> discardAccountLogin(AccountLoginCaptcha challenge) async {
    discarded.add(challenge);
    pending.remove(challenge);
    challenge.clearImage();
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));

  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
  });

  for (final end in ['success', 'navigation']) {
    testWidgets('App sign-in reaches captcha then $end', (tester) async {
      final core = InteractiveFixture()..wrongAnswers = 1;
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
      expect(c.error, isNull);
      expect(core.challenges, hasLength(1));
      expect(find.byKey(const ValueKey('captcha-answer')), findsOneWidget);
      expect(core.ownedPassword!.every((b) => b == 0), true);
      if (end == 'success') {
        for (final answer in ['Wrong', 'Good']) {
          await tester.enterText(
            find.byKey(const ValueKey('captcha-answer')),
            answer,
          );
          await tester.tap(find.text('Verify and Continue'));
          await tester.pump();
          await tester.pump(const Duration(milliseconds: 400));
          await tester.pump();
          expect(tester.takeException(), isNull);
        }
        expect(key.currentState!.id, 'P09');
        expect(c.selectedAccount, firstAccount);
        expect(core.answers, ['Wrong', 'Good']);
      } else {
        await key.currentState!.go('P01');
        await tester.pumpAndSettle();
        expect(key.currentState!.id, 'P01');
        expect(core.answers, isEmpty);
        expect(c.accountReport, isNull);
      }
      expect(find.byKey(const ValueKey('captcha-answer')), findsNothing);
      expect(core.pending, isEmpty);
      expect(core.challenges.every((c) => c.image.every((b) => b == 0)), true);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    });
  }

  test(
    'replacement preserves exact reauthorization and partial inventory',
    () async {
      final core = InteractiveFixture()
        ..wrongAnswers = 1
        ..retrieval = 'partial'
        ..retrievalError = 'MIJIA_INVENTORY_PARTIAL';
      final c = HomeController(core, AccountPreferences());
      await c.selectAccount(firstAccount);
      var prompts = 0;
      expect(
        await c.connectAccount(
          Uint8List.fromList('fixture@example.invalid'.codeUnits),
          Uint8List.fromList('Fictional'.codeUnits),
          'Home',
          onChallenge: (challenge) async {
            expect(c.accountReport, isNull);
            expect(core.ownedLogin!.every((b) => b == 0), true);
            expect(core.ownedPassword!.every((b) => b == 0), true);
            return Uint8List.fromList(
              (++prompts == 1 ? 'Wrong' : 'Good').codeUnits,
            );
          },
        ),
        true,
      );
      expect(core.target, firstAccount);
      expect(core.answers, ['Wrong', 'Good']);
      expect(core.discarded, core.challenges);
      expect(core.pending, isEmpty);
      expect(core.buffers.every((b) => b.every((v) => v == 0)), true);
      expect(core.challenges.every((c) => c.image.every((b) => b == 0)), true);
      expect(c.accountReport!['retrieval']['status'], 'partial');
      expect(c.selectedAccount, firstAccount);
      c.dispose();
    },
  );

  for (final cancellation in [
    'dismiss',
    'background',
    'navigation',
    'expired',
  ]) {
    test(
      '$cancellation prevents answer dispatch and releases challenge',
      () async {
        final core = InteractiveFixture();
        final c = HomeController(core, AccountPreferences());
        final bytes = Uint8List.fromList('Ab12'.codeUnits);
        expect(
          await c.connectAccount(
            Uint8List.fromList('fixture'.codeUnits),
            Uint8List.fromList('Fictional'.codeUnits),
            'Home',
            onChallenge: (challenge) async {
              if (cancellation == 'background') c.setForeground(false);
              if (cancellation == 'navigation') c.cancelPending();
              if (cancellation == 'expired')
                (challenge as CaptchaFixture).remaining = Duration.zero;
              return cancellation == 'dismiss' ? null : bytes;
            },
          ),
          false,
        );
        expect(core.answers, isEmpty);
        expect(core.pending, isEmpty);
        expect(core.challenges.single.image.every((b) => b == 0), true);
        if (cancellation != 'dismiss') expect(bytes.every((b) => b == 0), true);
        expect(c.accountReport, isNull);
        c.dispose();
      },
    );
  }

  test('late completion cannot publish success after cancellation', () async {
    final core = InteractiveFixture()..answerGate = Completer<void>();
    final c = HomeController(core, AccountPreferences());
    final answering = Completer<void>();
    final work = c.connectAccount(
      Uint8List.fromList('fixture'.codeUnits),
      Uint8List.fromList('Fictional'.codeUnits),
      'Home',
      onChallenge: (_) async {
        answering.complete();
        return Uint8List.fromList('Good'.codeUnits);
      },
    );
    await answering.future;
    await Future<void>.delayed(Duration.zero);
    c.cancelPending();
    core.answerGate!.complete();
    expect(await work, false);
    expect(c.accountReport, isNull);
    expect(core.pending, isEmpty);
    expect(core.buffers.single.every((b) => b == 0), true);
    c.dispose();
  });

  test('older clients retain one-shot login', () async {
    final core = InteractiveFixture()..supportsInteractiveLogin = false;
    final c = HomeController(core, AccountPreferences());
    expect(
      await c.connectAccount(
        Uint8List.fromList('fixture'.codeUnits),
        Uint8List.fromList('Fictional'.codeUnits),
        'Home',
        onChallenge: (_) async {
          fail('An older native API must not prompt.');
        },
      ),
      true,
    );
    expect(core.connectionProviders, ['xiaomi.mijia']);
    expect(core.challenges, isEmpty);
    c.dispose();
  });

  test(
    'native version and lifecycle refusals erase buffers without traffic',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'homeverb-captcha-ffi-',
      );
      final library = File(
        '../../target/debug/${NativeClient.defaultLibrary()}',
      ).absolute.path;
      final client = await NativeClient.open(
        dataDir: directory.path,
        libraryPath: library,
        adapter: 'xiaomi.mijia',
      );
      try {
        expect(client.supportsInteractiveLogin, true);
        for (final state in ['old', 'background', 'closed']) {
          client.apiVersion = state == 'old' ? '1.27.0' : '1.28.0';
          if (state == 'background') client.foreground(false);
          if (state == 'closed') await client.close();
          final login = Uint8List.fromList('fictional'.codeUnits),
              password = Uint8List.fromList('fictional'.codeUnits);
          await expectLater(
            client.beginAccountLogin(
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

  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    testWidgets('320px double-text captcha in ${locale.languageCode}', (
      tester,
    ) async {
      LocaleSettings.setLocale(locale);
      tester.view.physicalSize = const Size(640, 1600);
      tester.view.devicePixelRatio = 2;
      tester.platformDispatcher.textScaleFactorTestValue = 2;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
      final challenge = CaptchaFixture();
      Uint8List? result;
      await tester.pumpWidget(
        TranslationProvider(
          child: MaterialApp(
            home: Builder(
              builder: (context) => TextButton(
                onPressed: () async {
                  result = await showDialog<Uint8List>(
                    context: context,
                    builder: (_) => AccountCaptchaPrompt(challenge: challenge),
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
      expect(tester.takeException(), isNull);
      final submit = find.text(locale.translations.mobileCaptchaContinue);
      await tester.ensureVisible(submit);
      await tester.tap(submit);
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('captcha-answer')), findsOneWidget);
      await tester.enterText(
        find.byKey(const ValueKey('captcha-answer')),
        'Ab12',
      );
      await tester.ensureVisible(submit);
      await tester.tap(submit);
      await tester.pumpAndSettle();
      expect(result, 'Ab12'.codeUnits);
      expect(challenge.image.every((b) => b == 0), true);
      expect(tester.takeException(), isNull);
    });
  }

  for (final ending in ['background', 'expiry', 'back', 'navigation']) {
    testWidgets('$ending dismisses without an answer', (tester) async {
      final challenge = CaptchaFixture(remaining: const Duration(seconds: 2));
      Uint8List? result;
      var closed = false;
      final cancellation = ChangeNotifier();
      addTearDown(cancellation.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => TextButton(
              onPressed: () async {
                result = await showDialog<Uint8List>(
                  context: context,
                  builder: (_) => AccountCaptchaPrompt(
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
      await tester.enterText(
        find.byKey(const ValueKey('captcha-answer')),
        'Ab12',
      );
      if (ending == 'background') {
        tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      } else if (ending == 'navigation') {
        cancellation.notifyListeners();
      } else if (ending == 'back') {
        await tester.binding.handlePopRoute();
      } else {
        await tester.pump(const Duration(seconds: 2));
      }
      await tester.pumpAndSettle();
      if (ending == 'background') {
        expect(challenge.image.every((b) => b == 0), true);
        tester.binding.handleAppLifecycleStateChanged(
          AppLifecycleState.resumed,
        );
        await tester.pumpAndSettle();
      }
      expect(result, isNull);
      expect(closed, true);
      expect(find.byKey(const ValueKey('captcha-answer')), findsNothing);
      expect(challenge.image.every((b) => b == 0), true);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pump(const Duration(seconds: 3));
      expect(tester.takeException(), isNull);
    });
  }
}
