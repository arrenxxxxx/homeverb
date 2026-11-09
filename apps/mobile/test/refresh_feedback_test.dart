import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/refresh_feedback.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/skin.dart';
import 'accounts_test.dart' show AccountFixture, AccountPreferences;
import 'climate_controls_test.dart' show ClimateFixture;

class RefreshFailureFixture extends ClimateFixture {
  Completer<void>? gate;
  bool fail = false;
  int refreshes = 0;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'read_state' && fields['cached'] == false) {
      refreshes++;
      await gate?.future;
      if (fail)
        throw NativeFailure('NETWORK_UNAVAILABLE', 'PRIVATE provider response');
    }
    return super.call(method, fields);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  late ScreenCatalog catalog;
  const font = String.fromEnvironment('HOMEVERB_CJK_FONT');
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
    if (font.isNotEmpty) {
      await (FontLoader('Noto Sans CJK SC')
            ..addFont(File(font).readAsBytes().then(ByteData.sublistView))
            ..addFont(
              File(
                '${File(font).parent.path}/NotoSansCJK-Bold.ttc',
              ).readAsBytes().then(ByteData.sublistView),
            ))
          .load();
    }
    catalog = await ScreenCatalog.load();
  });
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));

  Future<(RefreshFailureFixture, HomeController, GlobalKey<NativeAppState>)>
  start(WidgetTester tester) async {
    final core = RefreshFailureFixture();
    final c = HomeController(core, AccountPreferences());
    final key = GlobalKey<NativeAppState>();
    await tester.pumpWidget(
      TranslationProvider(
        child: NativeApp(key: key, catalog: catalog, controller: c),
      ),
    );
    await tester.pumpAndSettle();
    c.stopAutomaticRefresh();
    return (core, c, key);
  }

  Future<void> pullHome(WidgetTester tester) async {
    final scroll = find.byKey(const PageStorageKey('scroll:D01'));
    final gesture = await tester.startGesture(tester.getCenter(scroll));
    await gesture.moveBy(const Offset(0, 25));
    await tester.pump();
    await gesture.moveBy(const Offset(0, 360));
    await tester.pump();
    await gesture.up();
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 250));
  }

  testWidgets(
    'refresh keeps cards, disables repeated requests and offers safe retry',
    (tester) async {
      tester.view.physicalSize = const Size(390, 844);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final (core, c, _) = await start(tester);
      final cards = c.cards;
      core.fail = true;
      core.gate = Completer<void>();
      await pullHome(tester);
      expect(find.text('Refreshing devices…'), findsOneWidget);
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      await pullHome(tester);
      expect(core.refreshes, 1);
      core.gate!.complete();
      await tester.pumpAndSettle();
      expect(c.cards, same(cards));
      expect(find.byType(RefreshFeedback), findsOneWidget);
      expect(
        tester
            .widget<SnackBar>(find.byType(SnackBar))
            .margin!
            .resolve(TextDirection.ltr)
            .bottom,
        greaterThanOrEqualTo(74),
      );
      expect(
        tester
            .widgetList<Semantics>(
              find.descendant(
                of: find.byType(RefreshFeedback),
                matching: find.byType(Semantics),
              ),
            )
            .any((widget) => widget.properties.liveRegion == true),
        isTrue,
      );
      expect(find.textContaining('PRIVATE'), findsNothing);
      expect(find.byType(CircularProgressIndicator), findsNothing);
      core.fail = false;
      core.gate = null;
      await tester.tap(find.text('Retry'));
      await tester.pumpAndSettle();
      expect(find.byType(RefreshFeedback), findsNothing);
      expect(core.refreshes, 2);
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets('unsupported-only home never claims readings were updated', (
    tester,
  ) async {
    final core = AccountFixture();
    final c = HomeController(core, AccountPreferences());
    final app = GlobalKey<NativeAppState>();
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: c),
    );
    await tester.pumpAndSettle();
    c.stopAutomaticRefresh();
    await app.currentState!.action('Refresh State');
    await tester.pumpAndSettle();
    expect(find.text('Readings updated'), findsNothing);
    expect(core.requests.any((r) => r['method'] == 'read_state'), isFalse);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('late refresh failure cannot appear on another page', (
    tester,
  ) async {
    final (core, c, key) = await start(tester);
    core.fail = true;
    core.gate = Completer<void>();
    final pending = key.currentState!.action('Refresh State');
    await tester.pump();
    await key.currentState!.go('S01');
    core.gate!.complete();
    await pending;
    await tester.pumpAndSettle();
    expect(find.byType(RefreshFeedback), findsNothing);
    expect(find.textContaining('PRIVATE'), findsNothing);
    await key.currentState!.go('D01');
    await tester.pumpAndSettle();
    expect(find.byType(RefreshFeedback), findsNothing);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('late refresh failure stays hidden on a device-detail shell', (
    tester,
  ) async {
    final (core, c, key) = await start(tester);
    await c.selectDevice(c.cards.first['device']['id'] as String);
    core.fail = true;
    core.gate = Completer<void>();
    final work = key.currentState!.action('Refresh State');
    await tester.pump();
    await key.currentState!.go('C01');
    core.gate!.complete();
    await work;
    await tester.pumpAndSettle();
    expect(find.text('Could not complete this operation'), findsNothing);
    expect(find.byType(RefreshFeedback), findsNothing);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('background fences a late successful manual reading', (
    tester,
  ) async {
    final (core, c, key) = await start(tester);
    final prior = Map<String, Json>.from(c.snapshots);
    core.gate = Completer<void>();
    final pending = key.currentState!.action('Refresh State');
    await tester.pump();
    key.currentState!.didChangeAppLifecycleState(AppLifecycleState.paused);
    core.gate!.complete();
    await pending;
    await tester.pumpAndSettle();
    expect(c.snapshots, prior);
    expect(find.byType(RefreshFeedback), findsNothing);
    await tester.pumpWidget(const SizedBox());
  });

  for (final skin in ['window_light', 'sky_blue']) {
    testWidgets('refresh feedback fits $skin at 320 pixels and double text', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(320, 900);
      tester.view.devicePixelRatio = 1;
      tester.platformDispatcher.textScaleFactorTestValue = 2;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
      await tester.pumpWidget(
        MaterialApp(
          theme: windowTheme(skinId: skin),
          home: Scaffold(
            body: SingleChildScrollView(
              child: RefreshFeedback(onRetry: () {}, onDismiss: () {}),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
    });
    testWidgets('capture $skin homepage refresh failure', (tester) async {
      debugDisableShadows = false;
      addTearDown(() => debugDisableShadows = true);
      LocaleSettings.setLocale(AppLocale.zhCn);
      tester.view.physicalSize = const Size(860, 1864);
      tester.view.devicePixelRatio = 2;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final core = RefreshFailureFixture();
      final c = HomeController(core, AccountPreferences());
      await c.savePreferences({'skin_id': skin});
      final key = GlobalKey<NativeAppState>(), boundary = GlobalKey();
      await tester.pumpWidget(
        TranslationProvider(
          child: RepaintBoundary(
            key: boundary,
            child: NativeApp(key: key, catalog: catalog, controller: c),
          ),
        ),
      );
      await tester.pumpAndSettle();
      c.stopAutomaticRefresh();
      core.fail = true;
      await key.currentState!.action('Refresh State');
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      await tester.runAsync(() async {
        final image =
            await (boundary.currentContext!.findRenderObject()
                    as RenderRepaintBoundary)
                .toImage(pixelRatio: 2);
        final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
        final file = File(
          'build/verification/auth-refresh/$skin/home-refresh-error.png',
        );
        file.parent.createSync(recursive: true);
        file.writeAsBytesSync(bytes!.buffer.asUint8List());
        image.dispose();
      });
      await tester.pumpWidget(const SizedBox());
      debugDisableShadows = true;
    }, skip: font.isEmpty);
  }
}
