import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/skin.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
  });

  testWidgets('control glyphs center across bundled font sizes and weights', (
    tester,
  ) async {
    await tester.runAsync(loadWindowSkin);
    tester.view.devicePixelRatio = 2;
    tester.view.physicalSize = const Size(780, 1600);
    addTearDown(() {
      tester.view.resetDevicePixelRatio();
      tester.view.resetPhysicalSize();
    });
    // Independent Pillow bounds from the bundled fonts in 160 px rows.
    const cases = [
      (10.0, false, 'Updated just now', 70, 89),
      (12.0, false, 'On', 71, 88),
      (12.0, false, 'Cooling', 68, 90),
      (13.0, false, 'Power', 70, 88),
      (13.0, false, 'Primary bedroom', 66, 92),
      (12.0, true, 'Current', 71, 88),
      (13.0, true, 'Living Room', 66, 92),
      (14.0, true, 'Account settings', 66, 93),
      (17.0, true, 'Window Light', 63, 96),
      (32.0, true, '123 456', 56, 104),
    ];
    final boundary = GlobalKey();
    await tester.pumpWidget(
      RepaintBoundary(
        key: boundary,
        child: MaterialApp(
          debugShowCheckedModeBanner: false,
          theme: windowTheme(),
          home: Scaffold(
            backgroundColor: Colors.white,
            body: Column(
              children: [
                for (final sample in cases)
                  SizedBox(
                    height: 80,
                    child: Center(
                      child: Copy(
                        sample.$3,
                        size: sample.$1,
                        bold: sample.$2,
                        control: true,
                      ),
                    ),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final actual = <String, (int, int)>{};
    await tester.runAsync(() async {
      final image =
          await (boundary.currentContext!.findRenderObject()
                  as RenderRepaintBoundary)
              .toImage(pixelRatio: 2);
      final bytes = (await image.toByteData(
        format: ui.ImageByteFormat.rawRgba,
      ))!;
      for (var i = 0; i < cases.length; i++) {
        final rows = <int>[];
        for (var y = i * 160 + 16; y < i * 160 + 144; y++) {
          for (var x = 40; x < 740; x++) {
            final offset = (y * image.width + x) * 4;
            if (bytes.getUint8(offset) != 255 ||
                bytes.getUint8(offset + 1) != 255 ||
                bytes.getUint8(offset + 2) != 255) {
              rows.add(y - i * 160);
              break;
            }
          }
        }
        expect(rows, isNotEmpty, reason: cases[i].$3);
        actual[cases[i].$3] = (rows.first, rows.last);
      }
      image.dispose();
    });
    expect(actual, {for (final c in cases) c.$3: (c.$4, c.$5)});
    expect(tester.takeException(), isNull);
  });
}
