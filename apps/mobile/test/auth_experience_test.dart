import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/account_views.dart';
import 'package:homeverb_mobile/credential_access.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/skin.dart';

import 'credential_access_test.dart' show FixtureAccess, enter;

void main() {
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  Widget page({
    bool busy = false,
    bool reauthorizing = false,
    Object? externalError,
    double scale = 1,
    String skin = 'sky_blue',
    FixtureAccess? access,
    VoidCallback? onCancel,
    Future<void> Function(dynamic, dynamic)? onSubmit,
  }) => MaterialApp(
    theme: windowTheme(skinId: skin),
    home: MediaQuery(
      data: MediaQueryData(textScaler: TextScaler.linear(scale)),
      child: Scaffold(
        body: SingleChildScrollView(
          padding: const EdgeInsets.all(20),
          child: AccountSignIn(
            credentialAccess: access ?? FixtureAccess(),
            busy: busy,
            needsReload: false,
            reauthorizing: reauthorizing,
            accountName: reauthorizing
                ? const LiteralText('Fictional home')
                : null,
            externalError: externalError,
            onSubmit: onSubmit ?? (_, _) async {},
            onCancel: onCancel ?? () {},
            onReload: () {},
          ),
        ),
      ),
    ),
  );

  testWidgets(
    'login and reauthorization have distinct purpose and safe errors',
    (tester) async {
      await tester.pumpWidget(page());
      await tester.pumpAndSettle();
      expect(find.text('Connect your appliances'), findsOneWidget);
      expect(find.text('Reauthorize your account'), findsNothing);
      await tester.pumpWidget(
        page(
          reauthorizing: true,
          externalError: NativeFailure('UNKNOWN', 'private-session-value'),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Reauthorize your account'), findsOneWidget);
      expect(find.text('Fictional home'), findsOneWidget);
      expect(find.textContaining('private-session-value'), findsNothing);
      expect(find.text('Unable to connect'), findsOneWidget);
    },
  );

  testWidgets('busy login has inline progress and cancel remains actionable', (
    tester,
  ) async {
    var cancelled = 0;
    await tester.pumpWidget(page(busy: true, onCancel: () => cancelled++));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
    expect(find.text('Connecting and retrieving devices…'), findsOneWidget);
    for (final field in tester.widgetList<TextField>(find.byType(TextField))) {
      expect(field.enabled, false);
    }
    await tester.ensureVisible(find.text('Cancel'));
    await tester.tap(find.text('Cancel'));
    expect(cancelled, 1);
  });

  testWidgets('cancel fences a late credential preflight without unmounting', (
    tester,
  ) async {
    final access = FixtureAccess();
    var sent = 0;
    await tester.pumpWidget(
      page(access: access, onSubmit: (_, _) async => sent++),
    );
    await tester.pumpAndSettle();
    await enter(tester);
    final pending = Completer<CredentialReadiness>();
    access.pending = pending;
    await tester.ensureVisible(find.text('Sign In and Retrieve Devices'));
    await tester.tap(find.text('Sign In and Retrieve Devices'));
    await tester.pump();
    await tester.ensureVisible(find.text('Cancel'));
    await tester.tap(find.text('Cancel'));
    pending.complete(CredentialReadiness.ready);
    await tester.pumpAndSettle();
    expect(sent, 0);
    for (final field in tester.widgetList<TextField>(find.byType(TextField))) {
      expect(field.controller!.text, isEmpty);
    }
  });

  for (final skin in ['window_light', 'sky_blue']) {
    testWidgets('$skin login supports narrow screens and large text', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(320, 900);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await tester.pumpWidget(page(skin: skin, scale: 2, reauthorizing: true));
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      await tester.ensureVisible(find.text('Cancel'));
      expect(tester.takeException(), isNull);
    });
  }

  for (final width in [320.0, 390.0, 430.0]) {
    for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
      testWidgets('cancel is centered at $width in ${locale.name}', (
        tester,
      ) async {
        tester.view.physicalSize = Size(width, 1000);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        LocaleSettings.setLocale(locale);
        await tester.pumpWidget(TranslationProvider(child: page()));
        await tester.pumpAndSettle();
        final cancel = find.byKey(const ValueKey('account-signin-cancel'));
        await tester.ensureVisible(cancel);
        await tester.pumpAndSettle();
        final form = find.byKey(const ValueKey('account-signin-card'));
        final text = find.descendant(of: cancel, matching: find.byType(Text));
        final center = tester.getCenter(form).dx;
        expect(tester.getCenter(cancel).dx, closeTo(center, 1));
        expect(tester.getCenter(text).dx, closeTo(center, 1));
        expect(tester.getSize(cancel).height, greaterThanOrEqualTo(48));
        expect(tester.takeException(), isNull);
      });
    }
  }
}
