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

  testWidgets('native button ink matches independent centered font bounds', (
    tester,
  ) async {
    await tester.runAsync(loadWindowSkin);
    tester.view.devicePixelRatio = 2;
    tester.view.physicalSize = const Size(780, 1040);
    addTearDown(() {
      tester.view.resetDevicePixelRatio();
      tester.view.resetPhysicalSize();
    });
    // Pillow's bundled Inter SemiBold at 28 px, centered in a 96 px face.
    // These inclusive bounds come from font metrics, never Flutter captures.
    const expected = {
      'Retrieve Devices': (37, 58),
      'Done': (38, 58),
      'Copy Code': (34, 60),
      'Start Pairing': (34, 61),
      'Apply': (34, 60),
      'Back to Accounts': (38, 58),
      'Start Cycle': (34, 60),
      'Turn Off 3 Devices': (37, 58),
    };
    for (final origin in [0.0, 0.2, 0.5, 0.7]) {
      final boundary = GlobalKey();
      await tester.pumpWidget(
        RepaintBoundary(
          key: boundary,
          child: MaterialApp(
            debugShowCheckedModeBanner: false,
            theme: windowTheme(),
            home: Scaffold(
              body: Padding(
                padding: EdgeInsets.only(top: origin),
                child: SizedBox(
                  width: 350,
                  child: Column(
                    children: [
                      for (final label in expected.keys)
                        ActionButton(label, onPressed: () {}),
                    ],
                  ),
                ),
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
        var i = 0;
        for (final entry in expected.entries) {
          final rows = <int>[];
          for (var y = i * 120 + 20; y < i * 120 + 84; y++) {
            for (var x = 50; x < 650; x++) {
              final offset = (y * image.width + x) * 4;
              if (bytes.getUint8(offset) != 185 ||
                  bytes.getUint8(offset + 1) != 78 ||
                  bytes.getUint8(offset + 2) != 41) {
                rows.add(y - i * 120);
                break;
              }
            }
          }
          expect(rows, isNotEmpty, reason: entry.key);
          final inkHeight = entry.value.$2 - entry.value.$1 + 1;
          final ideal = origin * 2 + (96 - inkHeight) / 2;
          final lower = ideal.floor();
          final top = ideal - lower == .5
              ? (lower.isEven ? lower : lower + 1)
              : ideal.round();
          expect(
            (rows.first, rows.last),
            (top, top + inkHeight - 1),
            reason: '${entry.key} at $origin logical pixels',
          );
          i++;
        }
        image.dispose();
      });
    }
  });

  testWidgets('scaled, wrapped and fallback labels remain usable native text', (
    tester,
  ) async {
    await tester.runAsync(loadWindowSkin);
    tester.view.devicePixelRatio = 2;
    tester.view.physicalSize = const Size(780, 1688);
    addTearDown(() {
      tester.view.resetDevicePixelRatio();
      tester.view.resetPhysicalSize();
    });
    final semantics = tester.ensureSemantics();
    try {
      for (final scale in [1.0, 2.0]) {
        for (final label in [
          'Copy Code and Retrieve Devices',
          'Copy Code\nRetrieve Devices',
          'Continue →',
        ]) {
          var tapped = false;
          await tester.pumpWidget(
            MaterialApp(
              theme: windowTheme(),
              home: MediaQuery(
                data: MediaQueryData(
                  devicePixelRatio: 2,
                  textScaler: TextScaler.linear(scale),
                ),
                child: Scaffold(
                  body: SizedBox(
                    width: 180,
                    child: ActionButton(label, onPressed: () => tapped = true),
                  ),
                ),
              ),
            ),
          );
          await tester.pumpAndSettle();
          final text = find.text(label);
          final paragraph = tester.renderObject<RenderParagraph>(text);
          final buttonRect = tester.getRect(find.byType(FilledButton));
          final textRect = tester.getRect(text);
          expect(textRect.top, greaterThanOrEqualTo(buttonRect.top));
          expect(textRect.bottom, lessThanOrEqualTo(buttonRect.bottom));
          expect(paragraph.didExceedMaxLines, false);
          expect(
            tester
                .getSemantics(text)
                .getSemanticsData()
                .hasAction(ui.SemanticsAction.tap),
            true,
          );
          await tester.tap(text);
          expect(tapped, true);
          expect(tester.takeException(), isNull);
        }
      }
    } finally {
      semantics.dispose();
    }
  });
}
