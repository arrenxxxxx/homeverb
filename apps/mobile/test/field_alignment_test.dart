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

  testWidgets('editable ink stays centered as values and hint change', (
    tester,
  ) async {
    await tester.runAsync(loadWindowSkin);
    tester.view.devicePixelRatio = 2;
    tester.view.physicalSize = const Size(780, 800);
    addTearDown(() {
      tester.view.resetDevicePixelRatio();
      tester.view.resetPhysicalSize();
    });
    final fields = [
      for (final value in ['23.5°C', 'Living Room', 'Kitchen', '', '••••'])
        TextEditingController(text: value),
    ];
    addTearDown(() {
      for (final field in fields) {
        field.dispose();
      }
    });
    final boundary = GlobalKey();
    await tester.pumpWidget(
      RepaintBoundary(
        key: boundary,
        child: MaterialApp(
          theme: windowTheme(),
          debugShowCheckedModeBanner: false,
          home: Scaffold(
            backgroundColor: Colors.white,
            body: Column(
              children: [
                for (final field in fields)
                  SizedBox(
                    height: 80,
                    child: Padding(
                      padding: const EdgeInsets.fromLTRB(20, 8, 20, 22),
                      child: ValueListenableBuilder<TextEditingValue>(
                        valueListenable: field,
                        builder: (context, value, _) => TextField(
                          controller: field,
                          style: windowFieldTextStyle,
                          textAlignVertical: TextAlignVertical.center,
                          decoration: InputDecoration(
                            border: InputBorder.none,
                            hintText: 'Search by name',
                            contentPadding: fieldTextPadding(
                              context,
                              value.text.isEmpty
                                  ? 'Search by name'
                                  : value.text,
                            ),
                          ),
                        ),
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
    Future<void> checkInk() async {
      await tester.runAsync(() async {
        final image =
            await (boundary.currentContext!.findRenderObject()
                    as RenderRepaintBoundary)
                .toImage(pixelRatio: 2);
        final bytes = (await image.toByteData(
          format: ui.ImageByteFormat.rawRgba,
        ))!;
        for (var row = 0; row < fields.length; row++) {
          final rows = <int>[];
          for (var y = row * 160 + 16; y < row * 160 + 116; y++) {
            for (var x = 40; x < 740; x++) {
              final i = (y * image.width + x) * 4;
              if (bytes.getUint8(i) != 255 ||
                  bytes.getUint8(i + 1) != 255 ||
                  bytes.getUint8(i + 2) != 255) {
                rows.add(y - row * 160);
                break;
              }
            }
          }
          expect(rows, isNotEmpty);
          // The 100 px field begins at 16: its visible ink center is y=66.
          expect(
            (rows.first + rows.last + 1) / 2,
            closeTo(66, .5),
            reason: 'Ink center for ${fields[row].text}',
          );
        }
        image.dispose();
      });
    }

    await checkInk();
    await tester.tap(find.byType(TextField).first);
    await tester.enterText(find.byType(TextField).first, 'Living Room');
    expect(fields.first.selection.baseOffset, 'Living Room'.length);
    FocusManager.instance.primaryFocus?.unfocus();
    await tester.pumpAndSettle();
    await checkInk();
    fields.first.clear();
    await tester.pumpAndSettle();
    await checkInk();
    for (final field in find.byType(TextField).evaluate()) {
      expect(tester.getSize(find.byWidget(field.widget)).height, 50);
    }
    expect(tester.takeException(), isNull);
  });
}
