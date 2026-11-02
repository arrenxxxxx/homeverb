import 'dart:async';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/account_surfaces.dart';
import 'package:homeverb_mobile/account_views.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/feedback.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/skin.dart';

import 'accounts_test.dart'
    show AccountFixture, AccountPreferences, firstAccount, secondAccount;
import 'credential_access_test.dart' show FixtureAccess;

class LateAccountFailure extends AccountFixture {
  final gate = Completer<void>();
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'refresh_account') {
      await gate.future;
      throw NativeFailure('NETWORK_UNAVAILABLE', 'PRIVATE account response');
    }
    return super.call(method, fields);
  }
}

void main() {
  Widget page(Widget child, {String skin = 'sky_blue', double scale = 1}) =>
      MaterialApp(
        theme: windowTheme(skinId: skin),
        home: MediaQuery(
          data: MediaQueryData(textScaler: TextScaler.linear(scale)),
          child: Scaffold(
            body: SingleChildScrollView(
              padding: const EdgeInsets.all(16),
              child: child,
            ),
          ),
        ),
      );

  Future<HomeController> failedAccount(
    String code, {
    bool partial = false,
  }) async {
    final core = AccountFixture()
      ..retrieval = partial ? 'partial' : 'failed'
      ..retrievalError = code;
    final controller = HomeController(core, AccountPreferences());
    await controller.initialize();
    await controller.selectAccount(firstAccount);
    await controller.retrieveAccount();
    return controller;
  }

  testWidgets('expired account has one notice and only its sign-in action', (
    tester,
  ) async {
    final c = await failedAccount('AUTH_REQUIRED');
    addTearDown(c.dispose);
    final actions = <String>[];
    await tester.pumpWidget(
      page(AccountDetails(controller: c, onAction: actions.add)),
    );
    expect(find.byType(FeedbackNotice), findsOneWidget);
    expect(find.text('Sign in again'), findsOneWidget);
    expect(find.text('Go to sign in'), findsOneWidget);
    expect(find.text('Refresh Devices'), findsNothing);
    expect(find.textContaining('AUTH_REQUIRED'), findsNothing);
    expect(find.textContaining('Fictional retrieval failure'), findsNothing);
    await tester.tap(find.text('Sign in again'));
    expect(actions, isEmpty);
    await tester.ensureVisible(find.text('Go to sign in'));
    await tester.tap(find.text('Go to sign in'));
    expect(actions, ['Reauthorize']);
    expect((c.core as AccountFixture).connectionTargets, isEmpty);
  });

  testWidgets('partial and network results preserve one explicit refresh', (
    tester,
  ) async {
    for (final partial in [false, true]) {
      final c = await failedAccount('NETWORK_UNAVAILABLE', partial: partial);
      final actions = <String>[];
      await tester.pumpWidget(
        page(AccountDetails(controller: c, onAction: actions.add)),
      );
      expect(find.byType(FeedbackNotice), findsNothing);
      expect(find.text('Refresh Devices'), findsOneWidget);
      expect(find.text('Go to sign in'), findsNothing);
      expect(find.textContaining('NETWORK_UNAVAILABLE'), findsNothing);
      await tester.ensureVisible(find.text('Refresh Devices'));
      await tester.tap(find.text('Refresh Devices'));
      expect(actions, ['Refresh Devices']);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    }
  });

  testWidgets('busy expired account disables sign-in without losing notice', (
    tester,
  ) async {
    final c = await failedAccount('AUTH_REQUIRED');
    addTearDown(c.dispose);
    c.busy = true;
    final actions = <String>[];
    await tester.pumpWidget(
      page(AccountDetails(controller: c, onAction: actions.add)),
    );
    expect(
      tester.widget<FeedbackNotice>(find.byType(FeedbackNotice)).onAction,
      isNull,
    );
    expect(actions, isEmpty);
  });

  test(
    'a previous account report does not expire the selected account',
    () async {
      final c = await failedAccount('AUTH_REQUIRED');
      addTearDown(c.dispose);
      c.selectedAccount = secondAccount;
      expect(accountRequiresSignIn(c), isFalse);
    },
  );

  for (final skin in ['sky_blue', 'window_light']) {
    testWidgets('expired notice fits 320px double text in $skin', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(320, 900);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final c = await failedAccount('AUTH_REQUIRED');
      addTearDown(c.dispose);
      await tester.pumpWidget(
        page(
          AccountDetails(controller: c, onAction: (_) {}),
          skin: skin,
          scale: 2,
        ),
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      expect(find.text('Go to sign in'), findsOneWidget);
    });
  }

  testWidgets('uncertain sign-in shows reload once, never unsafe retry', (
    tester,
  ) async {
    var reloads = 0;
    await tester.pumpWidget(
      page(
        AccountSignIn(
          credentialAccess: FixtureAccess(),
          busy: false,
          needsReload: true,
          externalError: NativeFailure('UNKNOWN', 'PRIVATE secret'),
          onSubmit: (_, _) async => fail('Must not repeat uncertain sign-in'),
          onCancel: () {},
          onReload: () => reloads++,
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.byType(FeedbackNotice), findsOneWidget);
    expect(find.text('Unable to connect'), findsNothing);
    expect(find.text('Sign In and Retrieve Devices'), findsNothing);
    expect(find.textContaining('PRIVATE'), findsNothing);
    await tester.ensureVisible(find.text('Reload Accounts'));
    await tester.tap(find.text('Reload Accounts'));
    expect(reloads, 1);
  });
  testWidgets('late account failure cannot leak onto another screen', (
    tester,
  ) async {
    final core = LateAccountFailure();
    final c = HomeController(core, AccountPreferences());
    final app = GlobalKey<NativeAppState>();
    await tester.pumpWidget(
      NativeApp(
        key: app,
        catalog: (await tester.runAsync(ScreenCatalog.load))!,
        controller: c,
      ),
    );
    await tester.pumpAndSettle();
    await c.selectAccount(firstAccount);
    await app.currentState!.go('P09');
    final work = app.currentState!.action('Refresh Devices');
    await tester.pump();
    await app.currentState!.go('S01');
    core.gate.complete();
    await work;
    await tester.pumpAndSettle();
    expect(find.byType(FeedbackNotice), findsNothing);
    expect(find.byType(FeedbackToast), findsNothing);
    expect(find.textContaining('PRIVATE'), findsNothing);
    await tester.pumpWidget(const SizedBox());
  });
}
