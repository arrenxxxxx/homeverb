import 'dart:io';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/skin.dart';
import 'package:homeverb_mobile/blocks.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/receipts.dart';
import 'accounts_test.dart' show AccountFixture, AccountPreferences;

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUp(() => LocaleSettings.setLocale(AppLocale.enUs));
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));

  test(
    'language preference persists and missing preferences follow the system',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'homeverb-language-',
      );
      try {
        final file = File('${directory.path}/presentation.json');
        final first = LocalPreferences(file);
        await first.load();
        expect(first.locale, isNull);
        await first.save({'locale': 'zh-CN'});
        final reopened = LocalPreferences(file);
        await reopened.load();
        expect(reopened.locale, 'zh-CN');
        await reopened.save({'locale': null});
        await first.load();
        expect(first.locale, isNull);
        expect(canonicalLocale('ZhCN'), 'zh-CN');
        expect(canonicalLocale('en_US'), 'en-US');
        expect(canonicalLocale('fr-FR'), isNull);
        expect(jsonDecode(await file.readAsString())['locale'], isNull);
      } finally {
        await directory.delete(recursive: true);
      }
    },
  );

  testWidgets(
    'settings switches both directions without Core writes or translating user names',
    (tester) async {
      final core = AccountFixture();
      core.cards.first['device']['alias'] = 'Settings';
      final preferences = AccountPreferences();
      final controller = HomeController(core, preferences);
      late ScreenCatalog catalog;
      await tester.runAsync(() async {
        catalog = await ScreenCatalog.load();
      });
      final key = GlobalKey<NativeAppState>();
      await tester.pumpWidget(
        TranslationProvider(
          child: NativeApp(key: key, catalog: catalog, controller: controller),
        ),
      );
      await tester.pumpAndSettle();
      await key.currentState!.go('S01');
      await tester.pumpAndSettle();
      await tester.tap(find.text('Language'));
      await tester.pumpAndSettle();
      final before = core.requests.length;
      await tester.tap(find.byKey(const ValueKey('locale:zh-CN')));
      await tester.pumpAndSettle();
      expect(preferences.locale, 'zh-CN');
      expect(
        find.text(AppLocale.zhCn.translations.settingsLanguage),
        findsOneWidget,
      );
      expect(
        find.text(AppLocale.zhCn.translations.mobileSystemDefault),
        findsOneWidget,
      );
      final material = tester.widget<MaterialApp>(find.byType(MaterialApp));
      expect(material.locale, const Locale('zh', 'CN'));
      expect(core.requests.length, before);
      await key.currentState!.go('D01');
      await tester.pumpAndSettle();
      expect(
        find.text('Settings'),
        findsOneWidget,
        reason: 'Device alias is literal user data.',
      );
      expect(find.text(AppLocale.zhCn.translations.deviceList), findsWidgets);
      await key.currentState!.go('L01');
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('locale:en-US')));
      await tester.pumpAndSettle();
      expect(find.text('Language'), findsOneWidget);
      expect(preferences.locale, 'en-US');
      expect(
        core.requests.where(
          (r) =>
              ['execute', 'discover', 'refresh_account'].contains(r['method']),
        ),
        isEmpty,
      );
      await tester.pumpWidget(const SizedBox());
      controller.dispose();
    },
  );

  testWidgets('Chinese account and settings pages fit narrow enlarged text', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(640, 1688);
    tester.view.devicePixelRatio = 2;
    tester.platformDispatcher.textScaleFactorTestValue = 2;
    addTearDown(() {
      tester.view.resetPhysicalSize();
      tester.view.resetDevicePixelRatio();
      tester.platformDispatcher.clearTextScaleFactorTestValue();
    });
    await LocaleSettings.setLocale(AppLocale.zhCn);
    final core = AccountFixture();
    final controller = HomeController(core, AccountPreferences());
    final key = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      TranslationProvider(
        child: NativeApp(key: key, catalog: catalog, controller: controller),
      ),
    );
    await tester.pumpAndSettle();
    for (final id in ['S01', 'L01', 'P02', 'P03', 'P04', 'S08', 'S12']) {
      await key.currentState!.go(id);
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull, reason: id);
    }
    await tester.pumpWidget(const SizedBox());
    controller.dispose();
  });

  testWidgets('system locale follows changes until explicitly overridden', (
    tester,
  ) async {
    tester.platformDispatcher.localesTestValue = const [Locale('zh', 'CN')];
    addTearDown(tester.platformDispatcher.clearLocalesTestValue);
    await applyLocalePreference(null);
    await tester.pumpWidget(
      TranslationProvider(
        child: Builder(
          builder: (context) => MaterialApp(
            locale: translationsOf(context).$meta.locale.flutterLocale,
            supportedLocales: AppLocaleUtils.supportedLocales,
            localizationsDelegates: GlobalMaterialLocalizations.delegates,
            home: const Scaffold(body: Copy('Settings')),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text(AppLocale.zhCn.translations.mobileSettings),
      findsOneWidget,
    );
    tester.platformDispatcher.localesTestValue = const [Locale('en', 'US')];
    await tester.pumpAndSettle();
    expect(find.text('Settings'), findsOneWidget);
    await applyLocalePreference('zh-CN');
    await tester.pumpAndSettle();
    tester.platformDispatcher.localesTestValue = const [Locale('fr', 'FR')];
    await tester.pumpAndSettle();
    expect(
      find.text(AppLocale.zhCn.translations.mobileSettings),
      findsOneWidget,
    );
    await applyLocalePreference(null);
    await tester.pumpAndSettle();
    expect(find.text('Settings'), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets(
    'Chinese receipt and error text preserve outcomes, IDs, parameters and actions',
    (tester) async {
      await LocaleSettings.setLocale(AppLocale.zhCn);
      final receipt =
          jsonDecode(
                File(
                  '../../examples/command-result.unknown.json',
                ).readAsStringSync(),
              )
              as Json;
      final before = jsonEncode(receipt);
      String? action;
      await tester.pumpWidget(
        TranslationProvider(
          child: MaterialApp(
            home: Scaffold(
              body: SingleChildScrollView(
                child: Blocks(
                  receiptBlocks(receipt, 'Settings'),
                  onAction: (value) => action = value,
                  fields: {},
                  choices: {},
                  values: {},
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Settings'), findsOneWidget);
      expect(
        find.text(
          AppLocale
              .zhCn
              .translations
              .mobileThisRequestMayHaveExecutedRefreshBeforeConsidering,
        ),
        findsOneWidget,
      );
      expect(jsonEncode(receipt), before);
      final button = find.text(AppLocale.zhCn.translations.mobileRefreshState);
      await tester.ensureVisible(button);
      await tester.tap(button);
      expect(action, 'Refresh State');
      await tester.pumpWidget(
        TranslationProvider(
          child: MaterialApp(
            home: Builder(
              builder: (context) => Scaffold(
                body: Copy(
                  localizedError(
                    context,
                    NativeFailure(
                      'SECRET_STORE_UNAVAILABLE',
                      'Do not expose raw provider detail.',
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(
        find.text(
          'SECRET_STORE_UNAVAILABLE: ${AppLocale.zhCn.translations.errorSECRETSTOREUNAVAILABLE}',
        ),
        findsOneWidget,
      );
      expect(find.textContaining('Do not expose raw'), findsNothing);
      await tester.pumpWidget(const SizedBox());
    },
  );
}
