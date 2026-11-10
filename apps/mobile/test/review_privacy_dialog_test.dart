import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart';

import 'accounts_test.dart' show AccountFixture, AccountPreferences;

void main() {
  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    testWidgets('privacy dialog scrolls and closes at large text: $locale', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(320, 844);
      tester.view.devicePixelRatio = 1;
      tester.platformDispatcher.textScaleFactorTestValue = 2;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
      await LocaleSettings.setLocale(locale);
      addTearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
      final c = HomeController(AccountFixture(), AccountPreferences());
      final app = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        TranslationProvider(
          child: NativeApp(key: app, catalog: catalog, controller: c),
        ),
      );
      await tester.pumpAndSettle();
      await app.currentState!.go('S01');
      await tester.pumpAndSettle();
      app.currentState!.action('Privacy');
      await tester.pumpAndSettle();
      final dialog = find.byType(AlertDialog);
      final scroll = find.descendant(
        of: dialog,
        matching: find.byType(Scrollable),
      );
      expect(scroll, findsOneWidget);
      final state = tester.state<ScrollableState>(scroll);
      if (locale == AppLocale.enUs) {
        expect(state.position.maxScrollExtent, greaterThan(0));
      }
      state.position.jumpTo(state.position.maxScrollExtent);
      await tester.pumpAndSettle();
      final body = find
          .descendant(of: dialog, matching: find.byType(Text))
          .at(1);
      final close = find.descendant(
        of: dialog,
        matching: find.byType(TextButton),
      );
      expect(
        tester.getBottomLeft(body).dy,
        lessThanOrEqualTo(tester.getTopLeft(close).dy),
      );
      await tester.tap(close);
      await tester.pumpAndSettle();
      expect(dialog, findsNothing);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    });
  }
}
