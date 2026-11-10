// Opt-in captures of actual NativeApp routes with fictional Core clients.
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart';

import 'accounts_test.dart' show AccountPreferences;
import 'climate_controls_test.dart' show ClimateFixture;
import 'device_opening_test.dart'
    show OpeningClient, OrderedPermissionCore, deviceA;
import 'interaction_fixture.dart';
import 'review_compartment_mode_test.dart' show ModeCompartment;
import 'review2_compartment_test.dart' show CombinedGrant;
import 'review4_fallback_test.dart' show PatternFixture;
import 'review5_root_fallback_test.dart' show RootConstraintFixture;
import 'component_test.dart' show CompartmentFixture;
import 'review_lifecycle_recovery_test.dart' show RecoveryCore;
import 'package:homeverb_mobile/native_client.dart' show NativeFailure;
import 'review_refrigerator_failure_test.dart' show FailedRefrigerator;
import 'secondary_journeys_test.dart' show SecondaryFixture;
import 'session_permissions_test.dart' show PermissionCore, inspection;

class JsonOrderedPermission extends OrderedPermissionCore {
  JsonOrderedPermission(super.inspected);
  @override
  Future<dynamic> call(
    String method, [
    Map<String, dynamic> fields = const {},
  ]) async {
    final result = await super.call(method, fields);
    return method == 'describe_device'
        ? jsonDecode(jsonEncode(result))
        : result;
  }
}

class JsonPermission extends PermissionCore {
  JsonPermission(super.inspected);
  @override
  Future<dynamic> call(
    String method, [
    Map<String, dynamic> fields = const {},
  ]) async {
    final result = await super.call(method, fields);
    return method == 'describe_device'
        ? jsonDecode(jsonEncode(result))
        : result;
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const enabled = bool.fromEnvironment('HOMEVERB_REVIEW_CAPTURE');
  const cjk = String.fromEnvironment('HOMEVERB_CJK_FONT');
  setUpAll(() async {
    if (!enabled) return;
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
    if (cjk.isNotEmpty) {
      await (FontLoader(
        'Noto Sans CJK SC',
      )..addFont(File(cjk).readAsBytes().then(ByteData.sublistView))).load();
    }
  });

  for (final scene in [
    'review5-root-form',
    'review4-decimal-step',
    'review4-pattern-form',
    'review2-reopen-feedback',
    'review2-quick-opening',
    'review2-combined-form',
    'review2-enum-bounds',
    'command-error',
    'refresh-opening',
    'opening-resume',
    'opening-retry',
    'zone-mode',
    'zone-mode-zh',
    'fridge-failure',
    'history-retry',
    'diagnostics-retry',
    'privacy-en',
    'privacy-zh',
  ]) {
    testWidgets('capture actual NativeApp $scene', (tester) async {
      tester.view.physicalSize = const Size(320, 844);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final privacy = scene.startsWith('privacy');
      tester.platformDispatcher.textScaleFactorTestValue = privacy ? 2 : 1;
      addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
      await LocaleSettings.setLocale(
        scene.endsWith('-zh') ? AppLocale.zhCn : AppLocale.enUs,
      );
      addTearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
      final boundary = GlobalKey();
      final app = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      final climate = ClimateFixture();
      final mode = ModeCompartment();
      final fridge = FailedRefrigerator();
      final combined = CombinedGrant();
      final pattern = PatternFixture();
      final root = RootConstraintFixture(
        'additionalProperties',
        <String, dynamic>{},
      );
      final decimal = CompartmentFixture();
      decimal.targets['fridge'] = 4.1;
      decimal
              .capabilities
              .first['properties']['target_temperature']['multipleOf'] =
          0.1;
      decimal
              .capabilities
              .first['actions']['set']['parameters']['properties']['target_temperature']['multipleOf'] =
          0.1;
      if (scene == 'review2-enum-bounds') {
        final capability = mode.capabilities.first;
        capability['properties']['humidity'] = {
          'type': 'integer',
          'enum': [40, 50, 60],
          'readable': true,
          'writable': true,
        };
        capability['actions']['set']['parameters']['properties']['humidity'] = {
          'type': 'integer',
          'enum': [40, 50, 60],
          'minimum': 50,
        };
      }
      final secondary = SecondaryFixture();
      final inner = await InteractionFixture.open();
      final opening = OpeningClient(inner);
      final options = inspection();
      final recovery = RecoveryCore(options);
      final interrupted = JsonOrderedPermission(options);
      final recovered = JsonPermission(options);
      final core = switch (scene) {
        'review5-root-form' => root,
        'review4-pattern-form' => pattern,
        'review4-decimal-step' => decimal,
        'command-error' || 'review2-quick-opening' => climate,
        'review2-reopen-feedback' => recovery,
        'review2-combined-form' => combined,
        'review2-enum-bounds' => mode,
        'zone-mode' || 'zone-mode-zh' => mode,
        'fridge-failure' => fridge,
        'refresh-opening' => opening,
        'opening-resume' || 'opening-retry' => interrupted,
        _ => secondary,
      };
      var reopenCount = 0;
      final c = HomeController(
        core,
        AccountPreferences(),
        reopen: scene == 'review2-reopen-feedback'
            ? (_) async => throw NativeFailure(
                'CREDENTIAL_STORE_UNAVAILABLE',
                'Fictional vault failure.',
              )
            : scene == 'opening-resume' || scene == 'opening-retry'
            ? (_) async {
                if (scene == 'opening-retry' && ++reopenCount == 1) {
                  throw StateError('Fictional session reopen failed.');
                }
                return recovered;
              }
            : null,
      );
      await tester.pumpWidget(
        RepaintBoundary(
          key: boundary,
          child: TranslationProvider(
            child: ColoredBox(
              color: const Color(0xfff8f5ee),
              child: NativeApp(key: app, catalog: catalog, controller: c),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      Future<void> save(String name) async {
        await tester.runAsync(() async {
          final render =
              boundary.currentContext!.findRenderObject()!
                  as RenderRepaintBoundary;
          final image = await render.toImage(pixelRatio: 2);
          final bytes = (await image.toByteData(
            format: ui.ImageByteFormat.png,
          ))!;
          final directory = Directory('build/verification/interaction-review')
            ..createSync(recursive: true);
          await File(
            '${directory.path}/$name.png',
          ).writeAsBytes(bytes.buffer.asUint8List());
          image.dispose();
        });
      }

      Completer<void>? gate;
      Future<void>? pending;
      switch (scene) {
        case 'review5-root-form':
          await tester.tap(find.text('Example refrigerator').first);
          await tester.pumpAndSettle();
          final form = find.byKey(
            const ValueKey('component-control:fridge:climate:set'),
          );
          await tester.ensureVisible(form);
          await tester.tap(form);
          await tester.pumpAndSettle();
          expect(app.currentState!.id, 'C10');
        case 'review4-decimal-step':
          await tester.tap(find.text('Example refrigerator').first);
          await tester.pumpAndSettle();
          expect(
            find.byKey(const ValueKey('component-stepper:fridge')),
            findsOneWidget,
          );
        case 'review4-pattern-form':
          await tester.tap(find.text('Example refrigerator').first);
          await tester.pumpAndSettle();
          final form = find.byKey(
            const ValueKey('component-control:fridge:climate:set'),
          );
          await tester.ensureVisible(form);
          await tester.tap(form);
          await tester.pumpAndSettle();
          expect(app.currentState!.id, 'C10');
        case 'review2-reopen-feedback':
          expect(await c.selectDevice(options['device_id']), false);
          await app.currentState!.go('D01');
          await app.currentState!.action('Refresh State');
          await tester.pump();
          await tester.pump(const Duration(milliseconds: 300));
          expect(c.error, isA<NativeFailure>());
        case 'review2-quick-opening':
          gate = Completer<void>();
          climate.executionGate = gate;
          pending = app.currentState!.action('power:${climate.device['id']}');
          for (var i = 0; i < 12; i++) await tester.pump();
          await tester.tap(find.text('Controls fixture').first);
          await tester.pump();
          await tester.pump(const Duration(milliseconds: 300));
          expect(app.currentState!.id, 'C01');
        case 'review2-combined-form':
          await tester.tap(find.text('Example refrigerator').first);
          await tester.pumpAndSettle();
          final form = find.byKey(
            const ValueKey('component-control:fridge:climate:set'),
          );
          await tester.ensureVisible(form);
          await tester.tap(form);
          await tester.pumpAndSettle();
          expect(app.currentState!.id, 'C10');
        case 'review2-enum-bounds':
          await tester.tap(find.text('Example refrigerator').first);
          await tester.pumpAndSettle();
          final option = find.byKey(
            const ValueKey('component-option:fridge:humidity:40'),
          );
          await tester.ensureVisible(option);
          await tester.pumpAndSettle();
          expect(tester.widget<ChoiceChip>(option).onSelected, isNull);
        case 'command-error':
          await c.selectDevice(climate.device['id']);
          await app.currentState!.go('C01');
          await tester.pumpAndSettle();
          await app.currentState!.executeOnDevice('power', 'set', {
            'state': false,
          });
          climate.denyPlan = true;
          await app.currentState!.executeOnDevice('power', 'set', {
            'state': true,
          });
          await tester.pumpAndSettle();
          expect(c.isCommandError, true);
        case 'refresh-opening':
          gate = inner.hold(deviceA, 'fridge');
          pending = app.currentState!.action('Refresh State');
          await tester.pump();
          await tester.tap(find.text('Opening fridge A').first);
          await tester.pump();
          await tester.pump(const Duration(milliseconds: 300));
          expect(app.currentState!.id, anyOf('C01', 'C08'));
        case 'opening-resume':
        case 'opening-retry':
          await app.currentState!.go('C01');
          final initial = c.selectDevice(options['device_id']);
          for (var i = 0; i < 10; i++) await tester.pump();
          tester.binding.handleAppLifecycleStateChanged(
            AppLifecycleState.paused,
          );
          tester.binding.handleAppLifecycleStateChanged(
            AppLifecycleState.resumed,
          );
          interrupted.liveGate.complete();
          await initial;
          for (var i = 0; i < 40; i++) await tester.pump();
          if (scene == 'opening-retry') {
            expect(
              find.byKey(const ValueKey('device-control-retry')),
              findsOneWidget,
            );
          } else {
            expect(c.capability('power')?['actions'], contains('set'));
          }
        case 'zone-mode':
        case 'zone-mode-zh':
          await tester.tap(find.text('Example refrigerator').first);
          await tester.pumpAndSettle();
          await tester.ensureVisible(
            find.byKey(const ValueKey('component-option:fridge:mode:eco')),
          );
          await tester.pumpAndSettle();
        case 'fridge-failure':
          await tester.tap(find.text('Example refrigerator').first);
          for (var i = 0; i < 10; i++) await tester.pump();
          fridge.gate.complete();
          await tester.pumpAndSettle();
          final row = find.byKey(
            const ValueKey(
              'refrigerator-reading:vendor.midea.refrigerator.storage_mode',
            ),
          );
          await tester.ensureVisible(row);
          await tester.pumpAndSettle();
          expect(
            find.descendant(of: row, matching: find.text('—')),
            findsOneWidget,
          );
        case 'history-retry':
        case 'diagnostics-retry':
          final history = scene == 'history-retry';
          gate = Completer<void>();
          if (history) {
            secondary.historyGate = gate;
          } else {
            secondary.diagnosticsGate = gate;
          }
          pending = app.currentState!.go(history ? 'S05' : 'S07');
          await tester.pump();
          tester.binding.handleAppLifecycleStateChanged(
            AppLifecycleState.paused,
          );
          tester.binding.handleAppLifecycleStateChanged(
            AppLifecycleState.resumed,
          );
          gate.complete();
          await pending;
          pending = null;
          gate = null;
          await tester.pumpAndSettle();
          expect(
            find.text(history ? 'Retry History' : 'Retry Diagnostics'),
            findsOneWidget,
          );
        case 'privacy-en':
        case 'privacy-zh':
          await app.currentState!.go('S01');
          await tester.pumpAndSettle();
          app.currentState!.action('Privacy');
          await tester.pumpAndSettle();
          await save('$scene-top');
          final scroll = find.descendant(
            of: find.byType(AlertDialog),
            matching: find.byType(Scrollable),
          );
          final state = tester.state<ScrollableState>(scroll);
          state.position.jumpTo(state.position.maxScrollExtent);
          await tester.pumpAndSettle();
      }
      expect(tester.takeException(), isNull);
      await save(scene);
      if (gate != null && !gate.isCompleted) gate.complete();
      if (pending != null) await pending;
      await tester.pumpWidget(const SizedBox());
    }, skip: !enabled);
  }
}
