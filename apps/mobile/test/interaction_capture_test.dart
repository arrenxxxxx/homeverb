// Opt-in captures of actual NativeApp routes with fictional client fixtures.
// These are production widgets, not HTML or a screenshot-only page wrapper.
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/client_contract.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'accounts_test.dart' show AccountFixture, firstAccount;
import 'climate_controls_test.dart' show ClimateFixture;
import 'device_detail_controls_test.dart' show DishwasherFixture, dishwasher;
import 'diagnostics_view_test.dart' show reportFixture;
import 'interaction_fixture.dart';
import 'refrigerator_modes_test.dart' show ReferenceFridgeFixture;

class _CaptureHome implements CoreClient {
  _CaptureHome() {
    ac.device['id'] = 'fictional-ac';
    ac.device['alias'] = 'Bedroom AC';
    ac.capabilities.first['properties']['target_temperature'] = {
      'type': 'number',
      'readable': true,
      'writable': false,
      'unit': 'celsius',
    };
    ac.observations['climate.target_temperature'] = 24;
    fridge.device['id'] = 'fictional-fridge';
    fridge.device['alias'] = 'Kitchen refrigerator';
    washer.device['id'] = 'fictional-dishwasher';
    washer.device['alias'] = 'Kitchen dishwasher';
  }

  final ac = ClimateFixture();
  final fridge = ReferenceFridgeFixture();
  final washer = DishwasherFixture();

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'list_rooms') return <Json>[];
    if (method == 'list_device_cards') {
      return [
        ...(await ac.call(method) as List),
        ...(await fridge.call(method) as List),
        ...(await washer.call(method) as List),
      ];
    }
    final device = fields['device'];
    if (device == ac.device['id']) return ac.call(method, fields);
    if (device == fridge.device['id']) return fridge.call(method, fields);
    if (device == washer.device['id']) return washer.call(method, fields);
    throw StateError('Unexpected fictional home request: $method');
  }

  @override
  void cancelAll() {}
  @override
  void foreground(bool value) {}
  @override
  Future<void> close() async {}
}

class _CaptureAccounts extends AccountFixture {
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'recent_executions' || method == 'device_executions') {
      return <Json>[];
    }
    if (method == 'diagnostics') return reportFixture();
    return super.call(method, fields);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const enabled = bool.fromEnvironment('HOMEVERB_INTERACTION_CAPTURE');
  const width = int.fromEnvironment(
    'HOMEVERB_CAPTURE_WIDTH',
    defaultValue: 390,
  );
  const scale = int.fromEnvironment(
    'HOMEVERB_CAPTURE_TEXT_PERCENT',
    defaultValue: 100,
  );
  const skin = String.fromEnvironment(
    'HOMEVERB_CAPTURE_SKIN',
    defaultValue: 'window_light',
  );
  const locale = String.fromEnvironment(
    'HOMEVERB_CAPTURE_LOCALE',
    defaultValue: 'en-US',
  );
  const cjk = String.fromEnvironment('HOMEVERB_CJK_FONT');
  const cjkBold = String.fromEnvironment('HOMEVERB_CJK_BOLD_FONT');
  late ScreenCatalog catalog;
  setUpAll(() async {
    if (!enabled) return;
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
    if (locale == 'zh-CN') {
      if (cjk.isEmpty || cjkBold.isEmpty) {
        throw StateError('Set HOMEVERB_CJK_FONT and HOMEVERB_CJK_BOLD_FONT.');
      }
      await (FontLoader('Noto Sans CJK SC')
            ..addFont(File(cjk).readAsBytes().then(ByteData.sublistView))
            ..addFont(File(cjkBold).readAsBytes().then(ByteData.sublistView)))
          .load();
    }
    catalog = await ScreenCatalog.load();
  });
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  for (final scene in [
    'home',
    'ac',
    'refrigerator',
    'dishwasher-idle',
    'dishwasher-running',
    'accounts',
    'account-detail',
    'signin',
    'settings',
    'history',
    'diagnostics',
    'first-loading',
    'first-failure',
  ]) {
    testWidgets('capture actual NativeApp $scene', (tester) async {
      tester.view.physicalSize = Size(width.toDouble(), 844);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await LocaleSettings.setLocale(
        locale == 'zh-CN' ? AppLocale.zhCn : AppLocale.enUs,
      );
      final prefs = LocalPreferences.memory()
        ..values['skin_id'] = skin
        ..values['locale'] = locale;
      late CoreClient client;
      String? device;
      InteractionFixture? delayed;
      if (scene == 'home') {
        client = _CaptureHome();
      } else if (scene == 'ac') {
        final fixture = ClimateFixture();
        client = fixture;
        device = fixture.device['id'] as String;
      } else if (scene == 'refrigerator') {
        final fixture = ReferenceFridgeFixture();
        client = fixture;
        device = fixture.device['id'] as String;
      } else if (scene.startsWith('dishwasher')) {
        final fixture = DishwasherFixture();
        fixture.observations['$dishwasher.work_status'] =
            scene == 'dishwasher-running' ? 'work' : 'power_on';
        client = fixture;
        device = fixture.device['id'] as String;
      } else if (scene.startsWith('first-')) {
        delayed = await InteractionFixture.open();
        client = delayed;
        device = delayed.device['id'] as String;
        if (scene == 'first-loading') delayed.hold(device, 'fridge');
        if (scene == 'first-failure') {
          delayed.failures.addAll({(device, 'fridge'), (device, 'freezer')});
        }
      } else {
        client = _CaptureAccounts();
      }
      final c = HomeController(client, prefs);
      final app = GlobalKey<NativeAppState>();
      final boundary = GlobalKey();
      await tester.pumpWidget(
        TranslationProvider(
          child: RepaintBoundary(
            key: boundary,
            child: MediaQuery(
              data: MediaQueryData.fromView(
                tester.view,
              ).copyWith(textScaler: TextScaler.linear(scale / 100)),
              child: NativeApp(key: app, catalog: catalog, controller: c),
            ),
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
      await tester.pumpAndSettle();
      if (scene == 'home') {
        // Populate through the real controller using fictional client snapshots.
        // This is a synthetic foreground refresh, never appliance traffic.
        expect(await c.refreshCards(), true, reason: '${c.error}');
        await tester.pumpAndSettle();
      }
      if (scene == 'ac' ||
          scene.startsWith('dishwasher') ||
          scene == 'refrigerator') {
        await c.selectDevice(device!);
        await app.currentState!.go(scene == 'refrigerator' ? 'C08' : 'C01');
        await tester.pumpAndSettle();
      } else if (scene.startsWith('first-')) {
        await tester.tap(find.text('Example refrigerator').first);
        for (var i = 0; i < 8; i++) {
          await tester.pump();
        }
      } else if (scene != 'home') {
        if (scene == 'account-detail') {
          await c.selectAccount(firstAccount);
        }
        await app.currentState!.go(switch (scene) {
          'accounts' => 'P01',
          'account-detail' => 'P09',
          'signin' => 'P04',
          'settings' => 'S01',
          'history' => 'S05',
          'diagnostics' => 'S07',
          _ => throw StateError('Unsupported scene'),
        });
        await tester.pumpAndSettle();
      }
      expect(tester.takeException(), isNull);
      Future<void> save(String suffix) async {
        await tester.runAsync(() async {
          final target = Directory(
            'build/verification/interaction/$skin/$locale/${width}px-${scale}pct',
          )..createSync(recursive: true);
          final image =
              await (boundary.currentContext!.findRenderObject()!
                      as RenderRepaintBoundary)
                  .toImage(pixelRatio: 2);
          final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
          await File(
            '${target.path}/$scene$suffix.png',
          ).writeAsBytes(bytes!.buffer.asUint8List());
          image.dispose();
        });
      }

      await save('');
      if (!scene.startsWith('first-')) {
        final scrolls = tester.stateList<ScrollableState>(
          find.byType(Scrollable),
        );
        final extents = scrolls
            .where((state) => state.position.hasContentDimensions)
            .map((state) => state.position.maxScrollExtent)
            .where((extent) => extent > 0)
            .toList();
        if (extents.isNotEmpty) {
          final extra = extents.reduce((a, b) => a > b ? a : b);
          expect(extra + 844, lessThan(12000));
          tester.view.physicalSize = Size(width.toDouble(), 844 + extra);
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull);
          await save('-full');
        }
      }
      if (scene == 'first-loading') {
        delayed!.gates[(device!, 'fridge')]!.complete();
        await tester.pumpAndSettle();
      }
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    }, skip: !enabled);
  }
}
