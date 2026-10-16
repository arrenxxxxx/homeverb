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
  testWidgets('body glyph tops align to physical pixels across font styles', (
    tester,
  ) async {
    await tester.runAsync(loadWindowSkin);
    tester.view.devicePixelRatio = 2;
    tester.view.physicalSize = const Size(780, 1600);
    addTearDown(() {
      tester.view.resetDevicePixelRatio();
      tester.view.resetPhysicalSize();
    });
    const cases = [
      (10.0, false, 'Updated just now'),
      (11.0, false, 'HomeVerb'),
      (12.0, false, 'Waiting for an updated reading'),
      (13.0, false, 'observation.'),
      (14.0, true, 'Accepted; not yet confirmed'),
      (16.0, false, '⌄'),
      (23.0, true, 'Select mode'),
      (25.0, true, 'Bedroom AC'),
      (68.0, false, '24°'),
      (
        13.0,
        false,
        'The pending request has not replaced the last observation.',
      ),
    ];
    final boundary = GlobalKey();
    // The independent renderer rounds the requested ink-top coordinate at 2x.
    for (final origin in [
      (0.0, 16, 52),
      (.2, 16, 53),
      (.5, 17, 53),
      (.7, 17, 54),
    ]) {
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
                      child: Stack(
                        children: [
                          Positioned(
                            top: 8 + origin.$1,
                            left: 20,
                            right: 20,
                            child: Copy(
                              sample.$3,
                              size: sample.$1,
                              bold: sample.$2,
                            ),
                          ),
                        ],
                      ),
                    ),
                ],
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
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
          for (var y = i * 160; y < (i + 1) * 160; y++) {
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
          expect(
            rows.first,
            origin.$2,
            reason: '${cases[i].$3} at ${origin.$1}',
          );
          if (i == cases.length - 1) {
            final starts = [
              rows.first,
              for (var j = 1; j < rows.length; j++)
                if (rows[j] > rows[j - 1] + 1) rows[j],
            ];
            expect(starts, [origin.$2, origin.$3]);
          }
        }
        image.dispose();
      });
      expect(tester.takeException(), isNull);
    }
  });
}
