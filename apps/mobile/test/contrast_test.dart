import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/screens.dart';
import 'package:homeverb_mobile/skin.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  late ScreenCatalog catalog;
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
    catalog = await ScreenCatalog.load();
  });
  testWidgets('system high contrast renders every state with readable text', (
    tester,
  ) async {
    tester.platformDispatcher.accessibilityFeaturesTestValue =
        const FakeAccessibilityFeatures(highContrast: true);
    final semantics = tester.ensureSemantics();
    final boundary = GlobalKey();
    final output = Directory('build/verification/accessibility')
      ..createSync(recursive: true);
    final failures = <String>[];
    try {
      for (final id in catalog.screens.keys) {
        tester.view.devicePixelRatio = 2;
        tester.view.physicalSize = id.startsWith('W')
            ? const Size(2048, 1536)
            : const Size(780, 1688);
        await tester.pumpWidget(
          RepaintBoundary(
            key: boundary,
            child: PreviewApp(
              key: ValueKey(id),
              catalog: catalog,
              initial: id,
              testChrome: true,
            ),
          ),
        );
        await tester.runAsync(() async {
          for (final asset in [
            'window-light-background',
            'ac',
            'lamp',
            'tv',
            'sensor',
          ]) {
            await precacheImage(
              AssetImage('assets/$asset.png'),
              tester.element(find.byType(PreviewApp)),
            );
          }
        });
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull, reason: id);
        if (['D01', 'C01', 'C14', 'S01', 'R05', 'P03'].contains(id)) {
          await tester.runAsync(() async {
            final image =
                await (boundary.currentContext!.findRenderObject()
                        as RenderRepaintBoundary)
                    .toImage(pixelRatio: 2);
            final bytes = await image.toByteData(
              format: ui.ImageByteFormat.png,
            );
            File(
              '${output.path}/$id-high-contrast.png',
            ).writeAsBytesSync(bytes!.buffer.asUint8List());
            image.dispose();
          });
        }
        final evaluation = await textContrastGuideline.evaluate(tester);
        if (!evaluation.passed) failures.add('$id: ${evaluation.reason}');
      }
      File(
        '${output.path}/contrast-findings.txt',
      ).writeAsStringSync(failures.join('\n'));
      expect(failures, isEmpty);
    } finally {
      semantics.dispose();
      tester.platformDispatcher.clearAccessibilityFeaturesTestValue();
      tester.view.resetPhysicalSize();
      tester.view.resetDevicePixelRatio();
    }
  });
  testWidgets('high contrast supports narrow layouts with 200 percent text', (
    tester,
  ) async {
    tester.platformDispatcher.accessibilityFeaturesTestValue =
        const FakeAccessibilityFeatures(highContrast: true);
    tester.platformDispatcher.textScaleFactorTestValue = 2;
    tester.view.devicePixelRatio = 2;
    tester.view.physicalSize = const Size(640, 1688);
    final boundary = GlobalKey();
    try {
      for (final id in catalog.screens.keys) {
        await tester.pumpWidget(
          RepaintBoundary(
            key: boundary,
            child: PreviewApp(
              key: ValueKey(id),
              catalog: catalog,
              initial: id,
              testChrome: true,
            ),
          ),
        );
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull, reason: id);
        if (['D01', 'C14'].contains(id)) {
          await tester.runAsync(() async {
            final image =
                await (boundary.currentContext!.findRenderObject()
                        as RenderRepaintBoundary)
                    .toImage(pixelRatio: 2);
            final bytes = await image.toByteData(
              format: ui.ImageByteFormat.png,
            );
            final file = File(
              'build/verification/accessibility/$id-high-contrast-large.png',
            );
            file.parent.createSync(recursive: true);
            file.writeAsBytesSync(bytes!.buffer.asUint8List());
            image.dispose();
          });
        }
      }
    } finally {
      tester.platformDispatcher.clearAccessibilityFeaturesTestValue();
      tester.platformDispatcher.clearTextScaleFactorTestValue();
      tester.view.resetPhysicalSize();
      tester.view.resetDevicePixelRatio();
    }
  });

  testWidgets(
    'changing the system contrast preference restores the original skin',
    (tester) async {
      await tester.pumpWidget(PreviewApp(catalog: catalog, testChrome: true));
      await tester.pumpAndSettle();
      final original = tester.widget<Text>(find.text('Platforms')).style!;
      expect(original.color, muted);
      tester.platformDispatcher.accessibilityFeaturesTestValue =
          const FakeAccessibilityFeatures(highContrast: true);
      try {
        await tester.pumpAndSettle();
        final contrast = tester.widget<Text>(find.text('Platforms')).style!;
        expect(contrast.color, Colors.black);
        expect(contrast.fontSize, greaterThanOrEqualTo(12));
      } finally {
        tester.platformDispatcher.clearAccessibilityFeaturesTestValue();
      }
      await tester.pumpAndSettle();
      expect(tester.widget<Text>(find.text('Platforms')).style, original);
    },
  );
}
