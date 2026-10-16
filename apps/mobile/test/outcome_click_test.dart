import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/receipts.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
  });
  for (final scenario in {
    'lost-ack': 'unknown',
    'accept-only': 'accepted',
    'reject': 'failed',
  }.entries) {
    testWidgets('click ${scenario.value} receipt and refresh without replay', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(780, 1688);
      tester.view.devicePixelRatio = 2;
      final directory = Directory.systemTemp.createTempSync('homeverb-click-');
      late NativeClient core;
      late HomeController controller;
      late ScreenCatalog catalog;
      await tester.runAsync(() async {
        core = await NativeClient.open(
          dataDir: directory.path,
          libraryPath: File(
            '../../target/debug/${NativeClient.defaultLibrary()}',
          ).absolute.path,
          allowMockWrites: true,
          scenario: scenario.key,
        );
        controller = HomeController(
          core,
          LocalPreferences(File('${directory.path}/ui.json')),
        );
        catalog = await ScreenCatalog.load();
      });
      final app = GlobalKey<NativeAppState>();
      Future<void> settle() async {
        for (var i = 0; i < 200; i++) {
          await tester.pump(const Duration(milliseconds: 100));
          await tester.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 10)),
          );
          if (i >= 5 && !controller.busy) break;
        }
        expect(controller.busy, false);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
      }

      Future<void> tap(String label) async {
        final target = find.text(label).last;
        await tester.ensureVisible(target);
        await tester.runAsync(() => tester.tap(target));
        await settle();
      }

      try {
        await tester.pumpWidget(
          NativeApp(key: app, catalog: catalog, controller: controller),
        );
        await settle();
        await tester.tap(find.byTooltip('Add device'));
        await settle();
        await tap('Discover Mock device');
        await tap('mock-ac');
        await tap('Refresh State');
        final before = controller.observed('power.state');
        await tap(before == true ? 'Turn off' : 'Turn on');
        expect(app.currentState!.id, 'C01');
        expect(find.text(outcomeLabel(scenario.value)), findsOneWidget);
        expect(controller.receipt!['status'], scenario.value);
        expect(
          controller.observed('power.state'),
          scenario.key == 'lost-ack' ? !before : before,
        );
        final request = controller.receipt!['request_id'];
        await tester.runAsync(() => controller.loadHistory());
        final count = controller.history.length;
        expect(count, 1);
        await tap('Refresh State');
        expect(controller.error, isNull);
        expect(controller.receipt!['request_id'], request);
        expect(controller.receipt!['status'], scenario.value);
        await tap('View Details');
        expect(app.currentState!.id, 'S06');
        await tap('View Operations');
        expect(controller.history.length, count);
        expect(
          find.textContaining(outcomeLabel(scenario.value)),
          findsOneWidget,
        );
        expect(controller.history.single['request_id'], request);
      } finally {
        await tester.pumpWidget(const SizedBox());
        await tester.runAsync(() => core.close());
        directory.deleteSync(recursive: true);
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      }
    });
  }
}
