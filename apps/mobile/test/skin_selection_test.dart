import 'dart:io';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/skin.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/screens.dart';
import 'accounts_test.dart' show AccountFixture;

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  test(
    'skin persists across restart and unrelated saves; invalid values fall back',
    () async {
      final directory = await Directory.systemTemp.createTemp('homeverb-skin-');
      try {
        final file = File('${directory.path}/presentation.json');
        final first = LocalPreferences(file);
        await first.load();
        expect(first.skin, 'window_light');
        await first.save({'skin_id': 'sky_blue'});
        await first.save({'locale': 'zh-CN'});
        final reopened = LocalPreferences(file);
        await reopened.load();
        expect(reopened.skin, 'sky_blue');
        expect(reopened.locale, 'zh-CN');
        await reopened.save({'skin_id': 'unknown'});
        expect(reopened.skin, 'window_light');
        for (final value in [
          null,
          5,
          {'invalid': true},
          'future_skin',
        ]) {
          await file.writeAsString(
            jsonEncode({'schema_version': 1, 'skin_id': value}),
          );
          await reopened.load();
          expect(reopened.skin, 'window_light');
        }
        await file.writeAsString('corrupt');
        final corrupt = LocalPreferences(file);
        await corrupt.load();
        expect(corrupt.skin, 'window_light');
      } finally {
        await directory.delete(recursive: true);
      }
    },
  );
  test('registry keeps warm default and readable blue state roles', () {
    expect(windowTheme().colorScheme.primary, accent);
    expect(windowTheme(skinId: 'unknown').colorScheme.primary, accent);
    final blue = windowTheme(skinId: 'sky_blue');
    final skin = blue.extension<HomeVerbSkin>()!;
    expect(skin.isSky, isTrue);
    expect(blue.colorScheme.primary, skin.accentColor);
    double contrast(Color a, Color b) {
      final x = a.computeLuminance(), y = b.computeLuminance();
      return ((x > y ? x : y) + .05) / ((x < y ? x : y) + .05);
    }

    expect(contrast(skin.accentColor, Colors.white), greaterThanOrEqualTo(4.5));
    expect(contrast(skin.powerColor, Colors.white), greaterThanOrEqualTo(4.5));
    expect(contrast(ink, skin.cardColor), greaterThanOrEqualTo(4.5));
    expect(
      windowTheme(skinId: 'sky_blue', highContrast: true).colorScheme.primary,
      Colors.black,
    );
  });
  testWidgets(
    'actual Appearance switches both skins without any Core operation',
    (tester) async {
      final core = AccountFixture();
      final prefs = LocalPreferences.memory();
      final controller = HomeController(core, prefs);
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      final key = GlobalKey<NativeAppState>();
      await tester.pumpWidget(
        TranslationProvider(
          child: NativeApp(key: key, catalog: catalog, controller: controller),
        ),
      );
      await tester.pumpAndSettle();
      await key.currentState!.go('S02');
      await tester.pumpAndSettle();
      final before = core.requests.length;
      await tester.tap(find.byKey(const ValueKey('skin:sky_blue')));
      await tester.pumpAndSettle();
      expect(prefs.skin, 'sky_blue');
      expect(
        tester
            .widget<MaterialApp>(find.byType(MaterialApp))
            .theme!
            .extension<HomeVerbSkin>()!
            .isSky,
        isTrue,
      );
      expect(core.requests.length, before);
      await tester.tap(find.byKey(const ValueKey('skin:window_light')));
      await tester.pumpAndSettle();
      expect(prefs.skin, 'window_light');
      expect(core.requests.length, before);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );
  testWidgets(
    'blue home and Appearance reflow at 320 pixels with doubled text',
    (tester) async {
      tester.view.physicalSize = const Size(320, 900);
      tester.view.devicePixelRatio = 1;
      tester.platformDispatcher.textScaleFactorTestValue = 2;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
      final core = AccountFixture();
      final prefs = LocalPreferences.memory()..values['skin_id'] = 'sky_blue';
      final controller = HomeController(core, prefs);
      addTearDown(controller.dispose);
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      final key = GlobalKey<NativeAppState>();
      await tester.pumpWidget(
        TranslationProvider(
          child: NativeApp(key: key, catalog: catalog, controller: controller),
        ),
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      await key.currentState!.go('S02');
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      for (final id in [
        'window_light',
        'sky_blue',
        'window_light',
        'sky_blue',
      ]) {
        final choice = find.byKey(ValueKey('skin:$id'));
        await tester.ensureVisible(choice);
        await tester.tap(choice);
        await tester.pumpAndSettle();
        expect(prefs.skin, id);
        expect(tester.takeException(), isNull);
      }
      await tester.pumpWidget(const SizedBox());
    },
  );
}
