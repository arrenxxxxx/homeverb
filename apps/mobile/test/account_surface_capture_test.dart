import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'accounts_test.dart'
    show AccountFixture, AccountPreferences, firstAccount, secondAccount;

void main() {
  const output = String.fromEnvironment('HOMEVERB_ACCOUNT_CAPTURE');
  const font = String.fromEnvironment('HOMEVERB_CJK_FONT');
  TestWidgetsFlutterBinding.ensureInitialized();
  late ScreenCatalog catalog;
  setUpAll(() async {
    if (output.isEmpty) return;
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    if (font.isNotEmpty) {
      const explicitBold = String.fromEnvironment('HOMEVERB_CJK_BOLD_FONT');
      final bold = explicitBold.isNotEmpty
          ? explicitBold
          : '${File(font).parent.path}/NotoSansCJK-Bold.ttc';
      if (!File(font).existsSync() || !File(bold).existsSync()) {
        throw StateError('Provide regular and bold Noto CJK font paths.');
      }
      await (FontLoader('Noto Sans CJK SC')
            ..addFont(File(font).readAsBytes().then(ByteData.sublistView))
            ..addFont(File(bold).readAsBytes().then(ByteData.sublistView)))
          .load();
    }
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
    catalog = await ScreenCatalog.load();
  });
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  for (final skin in ['window_light', 'sky_blue']) {
    testWidgets('capture approved account surfaces $skin', (tester) async {
      LocaleSettings.setLocale(AppLocale.zhCn);
      tester.view.physicalSize = const Size(860, 1864);
      tester.view.devicePixelRatio = 2;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final core = AccountFixture();
      core.record(firstAccount)['label'] = 'Home';
      core.record(secondAccount)['label'] = 'Parents';
      core.cards.first['device']['alias'] = 'Living room AC';
      final prefs = AccountPreferences();
      prefs.values['skin_id'] = skin;
      final c = HomeController(core, prefs);
      final app = GlobalKey<NativeAppState>();
      final boundary = GlobalKey();
      await tester.pumpWidget(
        TranslationProvider(
          child: RepaintBoundary(
            key: boundary,
            child: NativeApp(key: app, catalog: catalog, controller: c),
          ),
        ),
      );
      await tester.pumpAndSettle();
      await tester.runAsync(
        () => precacheImage(
          const AssetImage('assets/window-light-background.png'),
          app.currentContext!,
        ),
      );
      for (final entry in {
        'P01': 'accounts',
        'P02': 'platform-picker',
        'P09': 'account-detail',
      }.entries) {
        if (entry.key == 'P09') await c.selectAccount(firstAccount);
        await app.currentState!.go(entry.key);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull, reason: entry.key);
        await tester.runAsync(() async {
          final directory = Directory('$output/$skin')
            ..createSync(recursive: true);
          final image =
              await (boundary.currentContext!.findRenderObject()
                      as RenderRepaintBoundary)
                  .toImage(pixelRatio: 2);
          final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
          await File(
            '${directory.path}/${entry.value}.png',
          ).writeAsBytes(bytes!.buffer.asUint8List());
          image.dispose();
        });
      }
      expect(core.connectionTargets, isEmpty);
      expect(
        core.requests.any(
          (r) => [
            'read_state',
            'refresh_account',
            'execute',
            'remove_account',
          ].contains(r['method']),
        ),
        false,
      );
      await tester.pumpWidget(const SizedBox());
    }, skip: output.isEmpty);
  }
}
