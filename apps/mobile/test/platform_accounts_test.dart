import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/account_views.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/skin.dart';
import 'accounts_test.dart'
    show AccountFixture, AccountPreferences, firstAccount, secondAccount;

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  late ScreenCatalog catalog;
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
    catalog = await ScreenCatalog.load();
  });
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));

  testWidgets(
    'visible detail and remark actions select the exact local account',
    (tester) async {
      final core = AccountFixture();
      final prefs = AccountPreferences();
      await prefs.saveAccountLoginLabel(secondAccount, '138****5678');
      final c = HomeController(core, prefs);
      final app = GlobalKey<NativeAppState>();
      await tester.pumpWidget(
        NativeApp(key: app, catalog: catalog, controller: c),
      );
      await tester.pumpAndSettle();
      await app.currentState!.go('P01');
      await tester.pumpAndSettle();
      expect(find.text('Saved authorization · not checked'), findsNothing);
      expect(find.textContaining('Opening this page only reads'), findsNothing);
      final rename = find.byKey(
        const ValueKey('rename-account:$secondAccount'),
      );
      await tester.ensureVisible(rename);
      await tester.tap(rename);
      await tester.pumpAndSettle();
      expect(app.currentState!.id, 'P03');
      expect(c.selectedAccount, secondAccount);
      expect(
        tester
            .widget<TextField>(find.byKey(const ValueKey('field:Account name')))
            .decoration!
            .hintText,
        'Account name',
      );
      await tester.enterText(
        find.byKey(const ValueKey('field:Account name')),
        'Upstairs',
      );
      await tester.tap(find.text('Save Account Name'));
      await tester.pumpAndSettle();
      expect(core.record(secondAccount)['label'], 'Upstairs');
      expect(core.record(firstAccount)['label'], 'Home account');
      await app.currentState!.go('P01');
      await tester.pumpAndSettle();
      expect(find.text('Upstairs'), findsOneWidget);
      await tester.ensureVisible(rename);
      await tester.tap(rename);
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const ValueKey('field:Account name')),
        '',
      );
      await tester.tap(find.text('Save Account Name'));
      await tester.pumpAndSettle();
      expect(c.accountDisplayName(c.account!), '138****5678');
      expect(
        core.requests.any(
          (r) => [
            'read_state',
            'refresh_account',
            'recover_account',
            'execute',
          ].contains(r['method']),
        ),
        false,
      );
      await tester.pumpWidget(const SizedBox());
    },
  );

  for (final (width, scale, highContrast) in [
    (390.0, 1.0, false),
    (320.0, 2.0, true),
  ]) {
    testWidgets(
      'account actions fit $width with $scale text and literal names',
      (tester) async {
        LocaleSettings.setLocale(scale == 1 ? AppLocale.enUs : AppLocale.zhCn);
        tester.view.physicalSize = Size(width * 2, 1688);
        tester.view.devicePixelRatio = 2;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final core = AccountFixture();
        core.records.removeLast();
        core.record(firstAccount)['label'] = 'Settings';
        final c = HomeController(core, AccountPreferences());
        await c.initialize();
        final capture = GlobalKey();
        final actions = <String>[];
        Widget page() => TranslationProvider(
          child: MaterialApp(
            theme: windowTheme(highContrast: highContrast),
            home: MediaQuery(
              data: MediaQueryData(
                size: Size(width, 844),
                devicePixelRatio: 2,
                textScaler: TextScaler.linear(scale),
                highContrast: highContrast,
              ),
              child: Scaffold(
                body: LightBackground(
                  child: SingleChildScrollView(
                    child: RepaintBoundary(
                      key: capture,
                      child: Padding(
                        padding: const EdgeInsets.all(20),
                        child: PlatformAccounts(
                          controller: c,
                          onAction: actions.add,
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
        await tester.pumpWidget(page());
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        expect(find.text('Settings'), findsOneWidget);
        final open = find.byKey(const ValueKey('account:$firstAccount'));
        await tester.ensureVisible(open);
        await tester.tap(open);
        final rename = find.byKey(
          const ValueKey('rename-account:$firstAccount'),
        );
        await tester.ensureVisible(rename);
        await tester.tap(rename);
        expect(actions, [
          'account:$firstAccount',
          'rename-account:$firstAccount',
        ]);
        await tester.pumpAndSettle();
        const path = String.fromEnvironment('HOMEVERB_PLATFORM_CAPTURE');
        if (path.isNotEmpty && scale == 1) {
          await tester.runAsync(() async {
            final render =
                capture.currentContext!.findRenderObject()
                    as RenderRepaintBoundary;
            final image = await render.toImage(pixelRatio: 2);
            final bytes = (await image.toByteData(
              format: ui.ImageByteFormat.png,
            ))!.buffer.asUint8List();
            await File(path).writeAsBytes(bytes);
            image.dispose();
          });
        }
        c.busy = true;
        await tester.pumpWidget(page());
        expect(tester.widget<OutlinedButton>(open).onPressed, isNull);
        expect(tester.widget<TextButton>(rename).onPressed, isNull);
        await tester.pumpWidget(const SizedBox());
        c.dispose();
      },
    );
  }
}
