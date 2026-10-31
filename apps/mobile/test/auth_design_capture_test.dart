// Actual login widgets and screen chrome with fictional, empty credentials only.
// Opt in with HOMEVERB_CJK_FONT; never contacts a provider.
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/account_views.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/screens.dart';
import 'package:homeverb_mobile/skin.dart';

import 'credential_access_test.dart' show FixtureAccess;

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const path = String.fromEnvironment('HOMEVERB_CJK_FONT');
  const skinId = String.fromEnvironment(
    'HOMEVERB_CAPTURE_SKIN',
    defaultValue: 'sky_blue',
  );
  setUpAll(() async {
    if (path.isEmpty) return;
    const explicitBoldPath = String.fromEnvironment('HOMEVERB_CJK_BOLD_FONT');
    final boldPath = explicitBoldPath.isNotEmpty
        ? explicitBoldPath
        : '${File(path).parent.path}/NotoSansCJK-Bold.ttc';
    if (!File(path).existsSync() || !File(boldPath).existsSync()) {
      throw StateError('Provide regular and bold Noto CJK font paths.');
    }
    // Keep the production Latin/numeral face, with CJK in its own fallback
    // family. Never register Noto as Inter: that changes every numeral's shape.
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    await (FontLoader('Noto Sans CJK SC')
          ..addFont(File(path).readAsBytes().then(ByteData.sublistView))
          ..addFont(File(boldPath).readAsBytes().then(ByteData.sublistView)))
        .load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
  });

  for (final scenario in [
    'login',
    'reauthorize',
    'login-loading',
    'login-error',
  ]) {
    testWidgets('capture approved auth surface: $scenario', (tester) async {
      tester.view.physicalSize = const Size(430, 932);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      LocaleSettings.setLocale(AppLocale.zhCn);
      addTearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      final boundary = GlobalKey();
      await tester.pumpWidget(
        RepaintBoundary(
          key: boundary,
          child: TranslationProvider(
            child: MaterialApp(
              theme: windowTheme(skinId: skinId),
              debugShowCheckedModeBanner: false,
              home: Scaffold(
                body: ScreenView(
                  id: scenario == 'reauthorize' ? 'P09' : 'P04',
                  catalog: catalog,
                  screenOverride: {
                    ...catalog.ui(scenario == 'reauthorize' ? 'P09' : 'P04'),
                    'header': scenario == 'reauthorize'
                        ? 'Account details'
                        : 'Meiju sign-in',
                  },
                  fields: {},
                  choices: {},
                  values: {},
                  onAction: (_) {},
                  home: AccountSignIn(
                    credentialAccess: FixtureAccess(),
                    busy: scenario == 'login-loading',
                    reauthorizing: scenario == 'reauthorize',
                    accountName: const LiteralText('Home account'),
                    externalError: scenario == 'login-error'
                        ? const UiError('NETWORK_ERROR')
                        : null,
                    needsReload: false,
                    onSubmit: (_, _) async {},
                    onCancel: () {},
                    onReload: () {},
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.runAsync(
        () => precacheImage(
          const AssetImage('assets/window-light-background.png'),
          tester.element(find.byType(ScreenView)),
        ),
      );
      // The loading indicator intentionally remains active.
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 120));
      expect(tester.takeException(), isNull);
      await tester.runAsync(() async {
        final image =
            await (boundary.currentContext!.findRenderObject()!
                    as RenderRepaintBoundary)
                .toImage(pixelRatio: 2);
        final data = await image.toByteData(format: ui.ImageByteFormat.png);
        final directory = Directory('build/verification/auth-design/$skinId')
          ..createSync(recursive: true);
        File('${directory.path}/$scenario.png').writeAsBytesSync(
          data!.buffer.asUint8List(data.offsetInBytes, data.lengthInBytes),
        );
        image.dispose();
      });
    }, skip: path.isEmpty);
  }
}
