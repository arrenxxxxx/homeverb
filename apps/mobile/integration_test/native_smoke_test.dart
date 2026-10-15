import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('native Linux process discovers and controls through FFI', (
    tester,
  ) async {
    final dir = await Directory.systemTemp.createTemp('homeverb-linux-smoke-');
    const path = String.fromEnvironment('HOMEVERB_LIBRARY');
    final core = await NativeClient.open(
      dataDir: dir.path,
      libraryPath: path.isEmpty ? null : path,
      allowMockWrites: true,
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

    try {
      await tester.pumpWidget(
        RepaintBoundary(
          key: boundary,
          child: NativeApp(key: app, catalog: catalog, controller: controller),
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
      await tap('Mode');
      await tap('heat');
      await tap('Save Mode');
      expect(controller.receipt!['status'], 'verified');
      await app.currentState!.go('D01');
      await settle();
      final image =
          await (boundary.currentContext!.findRenderObject()
                  as RenderRepaintBoundary)
              .toImage(pixelRatio: 1);
      final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
      const output = String.fromEnvironment('HOMEVERB_CAPTURE');
      if (output.isNotEmpty)
        await File(output).writeAsBytes(bytes!.buffer.asUint8List());
      image.dispose();
      expect(tester.takeException(), isNull);
    } finally {
      await tester.pumpWidget(const SizedBox());
      await core.close();
      await dir.delete(recursive: true);
    }
  });
}
