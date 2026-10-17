import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets(
    'native process discovers, controls and clears local data through FFI',
    (tester) async {
      await LocaleSettings.setLocale(AppLocale.enUs);
      final dir = await Directory.systemTemp.createTemp(
        'homeverb-native-smoke-',
      );
      const path = String.fromEnvironment('HOMEVERB_LIBRARY');
      final core = await NativeClient.open(
        dataDir: dir.path,
        libraryPath: path.isEmpty ? null : path,
        allowMockWrites: true,
      );
      expect(core.apiAtLeast(22), true);
      expect(
        core.methods,
        containsAll([
          'inspect_session_grants',
          'inspect_control_options',
          'inspect_device_conflicts',
        ]),
      );
      final controller = HomeController(
        core,
        LocalPreferences(File('${dir.path}/ui.json')),
      );
      final boundary = GlobalKey();
      final app = GlobalKey<NativeAppState>();
      final catalog = await ScreenCatalog.load();
      Future<void> settle() async {
        await tester.pumpAndSettle();
        for (var i = 0; i < 200 && controller.busy; i++) {
          await tester.pump(const Duration(milliseconds: 20));
        }
        await tester.pumpAndSettle();
      }

      Future<void> tap(String label) async {
        final finder = find.text(label).last;
        await tester.ensureVisible(finder);
        await tester.tap(finder);
        await settle();
      }

      Future<void> capture(String output) async {
        if (output.isEmpty) return;
        final image =
            await (boundary.currentContext!.findRenderObject()
                    as RenderRepaintBoundary)
                .toImage(pixelRatio: 1);
        final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
        await File(output).writeAsBytes(bytes!.buffer.asUint8List());
        image.dispose();
      }

      try {
        await tester.pumpWidget(
          RepaintBoundary(
            key: boundary,
            child: TranslationProvider(
              child: NativeApp(
                key: app,
                catalog: catalog,
                controller: controller,
              ),
            ),
          ),
        );
        await settle();
        await tester.tap(find.byTooltip('Add device'));
        await settle();
        await tap('Discover Mock device');
        expect(controller.cards.length, 1);
        await tap('mock-ac');
        await tap('Refresh State');
        expect(controller.snapshot, isNotNull);
        await tap('Heat');
        expect(app.currentState!.id, 'C01');
        expect(controller.receipt!['status'], 'verified');
        expect(controller.observed('climate.mode'), 'heat');
        final increase = find.byKey(const ValueKey('climate-increase'));
        await tester.ensureVisible(increase);
        await tester.tap(increase);
        await settle();
        expect(controller.receipt!['status'], 'verified');
        expect(controller.observed('climate.target_temperature'), 26.5);
        for (final row in [
          'Fan speed',
          'Horizontal swing',
          'Horizontal direction',
          'Vertical swing',
          'Vertical direction',
          'Display light',
          'Button sound',
        ]) {
          expect(find.byKey(ValueKey('climate-row:$row')), findsNothing);
        }
        await LocaleSettings.setLocale(AppLocale.zhCn);
        await settle();
        final context = tester.element(find.byType(NativeApp));
        expect(find.text(localize(context, 'Mode')), findsOneWidget);
        expect(find.text(localize(context, 'Heat')), findsOneWidget);
        await tester.ensureVisible(
          find.text(localize(context, 'Target temperature')),
        );
        await settle();
        await capture(const String.fromEnvironment('HOMEVERB_CLIMATE_CAPTURE'));
        await LocaleSettings.setLocale(AppLocale.enUs);
        await settle();
        await app.currentState!.go('S04');
        await settle();
        await tap('Clear Cached State');
        await tap('Cancel');
        expect(controller.snapshot, isNotNull);
        await tap('Clear Cached State');
        await tap('Confirm Clear');
        expect(controller.cleanupCount, greaterThan(0));
        expect(controller.snapshot, isNull);
        expect(controller.history, isNotEmpty);
        await tap('Clear Recent Operations');
        await tap('Confirm Clear');
        expect(controller.cleanupCount, greaterThan(0));
        expect(controller.history, isEmpty);
        expect(controller.receipt, isNull);
        expect(controller.cards, hasLength(1));

        await capture(const String.fromEnvironment('HOMEVERB_CAPTURE'));
        expect(tester.takeException(), isNull);
      } finally {
        await tester.pumpWidget(const SizedBox());
        await core.close();
        await dir.delete(recursive: true);
        await LocaleSettings.setLocale(AppLocale.enUs);
      }
    },
  );
}
