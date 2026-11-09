// Opt-in captures of actual NativeApp routes with fictional client fixtures.
// These are production widgets, not HTML or a screenshot-only page wrapper.
import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/client_contract.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/device_readings.dart';
import 'package:homeverb_mobile/feedback.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/skin.dart';

import 'accounts_test.dart' show AccountFixture, firstAccount, firstDevice;
import 'climate_controls_test.dart' show ClimateFixture;
import 'device_detail_controls_test.dart' show DishwasherFixture, dishwasher;
import 'diagnostics_view_test.dart' show reportFixture;
import 'interaction_fixture.dart';
import 'refrigerator_modes_test.dart' show ReferenceFridgeFixture;

void _declareTarget(ClimateFixture fixture) {
  fixture.capabilities.first['properties']['target_temperature'] = {
    'type': 'number',
    'readable': true,
    'writable': false,
    'unit': 'celsius',
  };
  fixture.observations['climate.target_temperature'] = 24;
}

class _CaptureHome implements CoreClient {
  _CaptureHome() {
    ac.device['id'] = 'fictional-ac';
    ac.device['alias'] = 'Bedroom AC';
    _declareTarget(ac);
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
  _CaptureAccounts({this.withHistory = false});
  final bool withHistory;

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'recent_executions' || method == 'device_executions') {
      return [
        if (withHistory)
          {
            'schema_version': '1.0',
            'request_id': 'fictional-history-request',
            'device_id': firstDevice,
            'status': 'unknown',
            'started_at': '2026-10-14T05:00:00Z',
            'completed_at': '2026-10-14T05:00:01Z',
            'data': null,
            'error': null,
            'verification': null,
            'attempts': <Json>[],
          },
      ];
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
  const highContrast = bool.fromEnvironment('HOMEVERB_CAPTURE_HIGH_CONTRAST');
  const safeTop = int.fromEnvironment('HOMEVERB_CAPTURE_SAFE_TOP');
  const safeBottom = int.fromEnvironment('HOMEVERB_CAPTURE_SAFE_BOTTOM');
  const keyboard = int.fromEnvironment('HOMEVERB_CAPTURE_KEYBOARD_INSET');
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
    'history-content',
    'diagnostics',
    'first-loading',
    'first-failure',
    'cached-reopen',
    'partial-refresh',
    'command-pending',
    'command-unknown',
    'toast-short',
    'toast-long',
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
      ClimateFixture? climate;
      if (scene == 'home' || scene.startsWith('toast-')) {
        client = _CaptureHome();
      } else if (scene == 'ac' || scene.startsWith('command-')) {
        final fixture = ClimateFixture();
        _declareTarget(fixture);
        climate = fixture;
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
      } else if (scene.startsWith('first-') ||
          scene == 'cached-reopen' ||
          scene == 'partial-refresh') {
        delayed = await InteractionFixture.open();
        client = delayed;
        device = delayed.device['id'] as String;
        if (scene == 'first-loading') delayed.hold(device, 'fridge');
        if (scene == 'first-failure') {
          delayed.failures.addAll({(device, 'fridge'), (device, 'freezer')});
        }
        if (scene == 'cached-reopen') {
          for (final component in ['fridge', 'freezer']) {
            await delayed.call('read_state', {
              'device': device,
              'component': component,
              'cached': false,
            });
          }
          delayed.hold(device, 'fridge');
        }
        if (scene == 'partial-refresh') {
          delayed.values[(device, 'fridge')] = 8;
          delayed.failures.add((device, 'freezer'));
          delayed.hold(device, 'freezer');
        }
      } else {
        client = _CaptureAccounts(withHistory: scene == 'history-content');
      }
      final c = HomeController(client, prefs);
      final app = GlobalKey<NativeAppState>();
      final boundary = GlobalKey();
      await tester.pumpWidget(
        TranslationProvider(
          child: RepaintBoundary(
            key: boundary,
            child: MediaQuery(
              data: MediaQueryData.fromView(tester.view).copyWith(
                textScaler: TextScaler.linear(scale / 100),
                highContrast: highContrast,
                padding: EdgeInsets.only(
                  top: safeTop.toDouble(),
                  bottom: safeBottom.toDouble(),
                ),
                viewPadding: EdgeInsets.only(
                  top: safeTop.toDouble(),
                  bottom: safeBottom.toDouble(),
                ),
                viewInsets: EdgeInsets.only(bottom: keyboard.toDouble()),
              ),
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
          scene.startsWith('command-') ||
          scene.startsWith('dishwasher') ||
          scene == 'refrigerator') {
        await c.selectDevice(device!);
        await app.currentState!.go(scene == 'refrigerator' ? 'C08' : 'C01');
        await tester.pumpAndSettle();
      } else if (scene.startsWith('first-') || scene == 'cached-reopen') {
        await tester.tap(find.text('Example refrigerator').first);
        for (var i = 0; i < 8; i++) {
          await tester.pump();
        }
        expect(app.currentState!.id, 'C08');
        if (scene == 'first-loading') {
          expect(
            tester
                .widgetList<ReadingValue>(find.byType(ReadingValue))
                .any((value) => value.loading),
            isTrue,
          );
        }
        if (scene == 'cached-reopen') {
          expect(c.componentSnapshot('fridge'), isNotNull);
          expect(c.readingLoading('fridge'), isFalse);
        }
        if (scene == 'first-failure') {
          for (
            var i = 0;
            i < 20 && find.byType(FeedbackToast).evaluate().isEmpty;
            i++
          ) {
            await tester.pump(const Duration(milliseconds: 20));
          }
          expect(c.openingReadFailureSerial, 1);
          expect(find.byType(FeedbackToast), findsOneWidget);
          expect(find.text('—'), findsWidgets);
          await tester.pump(const Duration(milliseconds: 300));
          expect(find.byType(FeedbackToast), findsOneWidget);
        }
      } else if (scene != 'home' &&
          scene != 'partial-refresh' &&
          !scene.startsWith('toast-')) {
        if (scene == 'account-detail') {
          await c.selectAccount(firstAccount);
        }
        await app.currentState!.go(switch (scene) {
          'accounts' => 'P01',
          'account-detail' => 'P09',
          'signin' => 'P04',
          'settings' => 'S01',
          'history' => 'S05',
          'history-content' => 'S05',
          'diagnostics' => 'S07',
          _ => throw StateError('Unsupported scene'),
        });
        await tester.pumpAndSettle();
        if (scene == 'history-content') {
          expect(c.history, hasLength(1));
          expect(
            find.byKey(const ValueKey('receipt:fictional-history-request')),
            findsOneWidget,
          );
        }
      }
      Future<void>? refreshWork;
      if (scene == 'ac' || scene.startsWith('command-')) {
        expect(c.observed('climate.target_temperature'), 24);
        expect(find.text('24°C'), findsOneWidget);
      }
      if (scene == 'partial-refresh') {
        refreshWork = app.currentState!.action('Refresh State');
        for (var i = 0; i < 8; i++) {
          await tester.pump();
        }
        expect(delayed!.gates[(device!, 'freezer')]!.isCompleted, isFalse);
        expect(
          c.snapshots[device]?['observations']?['climate.target_temperature']?['value'],
          8,
        );
      }
      if (scene == 'command-pending' || scene == 'command-unknown') {
        if (scene == 'command-pending')
          climate!.executionGate = Completer<void>();
        if (scene == 'command-unknown') climate!.outcome = 'unknown';
        await tester.tap(find.byKey(const ValueKey('climate-power')));
        if (scene == 'command-pending') {
          for (var i = 0; i < 8; i++) {
            await tester.pump();
          }
          expect(climate!.executionGate!.isCompleted, isFalse);
          expect(find.byType(OperationPulse), findsOneWidget);
          expect(c.observed('power.state'), isTrue);
        } else {
          await tester.pumpAndSettle();
          expect(c.receipt?['status'], 'unknown');
          final details = find.text(localize(app.currentContext!, 'View Details'));
          expect(details, findsOneWidget);
          expect(find.byKey(const ValueKey('climate-power')), findsOneWidget);
          await tester.ensureVisible(details);
          await tester.pumpAndSettle();
        }
      }
      if (scene.startsWith('toast-')) {
        app.currentState!.feedback.show(
          app.currentState!.navigator.currentContext!,
          key: scene,
          aboveNavigation: true,
          hasAction: scene == 'toast-long',
          content: FeedbackToast(
            message: scene == 'toast-short'
                ? 'Could not load readings.'
                : 'Could not refresh. Your last readings are kept.',
            actionLabel: scene == 'toast-long' ? 'Retry' : null,
            onAction: scene == 'toast-long' ? () {} : null,
          ),
        );
        await tester.pumpAndSettle();
        expect(find.byType(FeedbackToast), findsOneWidget);
      }
      expect(tester.takeException(), isNull);
      Future<void> save(String suffix) async {
        await tester.runAsync(() async {
          final variant = [
            if (highContrast) 'contrast',
            if (safeTop != 0 || safeBottom != 0) 'safe-$safeTop-$safeBottom',
            if (keyboard != 0) 'keyboard-$keyboard',
          ].map((value) => '-$value').join();
          final target = Directory(
            'build/verification/interaction/$skin/$locale/${width}px-${scale}pct$variant',
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
      if (scene == 'signin') {
        await tester.ensureVisible(
          find.byKey(const ValueKey('account-signin-cancel')),
        );
        await tester.pumpAndSettle();
        await save('-cancel');
      }
      if (scene == 'partial-refresh') {
        delayed!.gates[(device!, 'freezer')]!.complete();
        await refreshWork;
        for (
          var i = 0;
          i < 20 && find.byType(FeedbackToast).evaluate().isEmpty;
          i++
        ) {
          await tester.pump(const Duration(milliseconds: 20));
        }
        expect(c.error, isNotNull);
        expect(find.byType(FeedbackToast), findsOneWidget);
        await tester.pump(const Duration(milliseconds: 300));
        await save('-failure');
      }
      if (!scene.startsWith('first-') && scene != 'command-pending') {
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
      if (scene == 'first-loading' || scene == 'cached-reopen') {
        delayed!.gates[(device!, 'fridge')]!.complete();
        await tester.pumpAndSettle();
      }
      if (scene == 'command-pending') {
        climate!.executionGate!.complete();
        await tester.pumpAndSettle();
      }
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    }, skip: !enabled);
  }
}
