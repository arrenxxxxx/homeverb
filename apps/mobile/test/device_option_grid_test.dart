import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/device_option_grid.dart';

void main() {
  for (final count in [2, 3, 4, 5, 6]) {
    testWidgets('$count options share equal widths and balanced compact rows', (
      tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: 398,
              child: DeviceOptionGrid(
                labels: List.generate(count, (i) => 'M$i'),
                children: List.generate(
                  count,
                  (i) => ChoiceChip(
                    key: ValueKey(i),
                    label: Text('M$i'),
                    selected: i == 0,
                    onSelected: (_) {},
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      final rowCounts = <double, int>{};
      final width = tester.getSize(find.byKey(const ValueKey(0))).width;
      for (var i = 0; i < count; i++) {
        final item = find.byKey(ValueKey(i));
        final size = tester.getSize(item);
        expect(size.width, closeTo(width, .01));
        expect(size.height, greaterThanOrEqualTo(48));
        final y = tester.getTopLeft(item).dy;
        rowCounts[y] = (rowCounts[y] ?? 0) + 1;
      }
      expect(rowCounts.length, count <= 3 ? 1 : 2);
      if (count == 4) expect(rowCounts.values, [2, 2]);
      if (count == 5) expect(rowCounts.values, [3, 2]);
      if (count == 6) expect(rowCounts.values, [3, 3]);
      expect(tester.widget<Wrap>(find.byType(Wrap)).runSpacing, 0);
      expect(tester.takeException(), isNull);
    });
  }
  for (final layout in [(320.0, 1.0), (430.0, 1.0), (320.0, 2.0)]) {
    testWidgets('painted columns and centered selection group at $layout', (
      tester,
    ) async {
      final semantics = tester.ensureSemantics();
      await tester.pumpWidget(
        MaterialApp(
          home: MediaQuery(
            data: MediaQueryData(textScaler: TextScaler.linear(layout.$2)),
            child: Scaffold(
              body: SizedBox(
                width: layout.$1,
                child: DeviceOptionGrid(
                  labels: List.generate(5, (i) => 'M$i'),
                  children: List.generate(
                    5,
                    (i) => ChoiceChip(
                      key: ValueKey('choice:$i'),
                      label: DeviceOptionLabel(text: 'M$i', selected: i == 1),
                      labelStyle: const TextStyle(fontSize: 16, height: 1.25),
                      padding: const EdgeInsets.symmetric(vertical: 12),
                      labelPadding: const EdgeInsets.symmetric(horizontal: 4),
                      selected: i == 1,
                      showCheckmark: false,
                      onSelected: (_) {},
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      final faces = [
        for (var i = 0; i < 5; i++)
          tester.getRect(
            find.descendant(
              of: find.byKey(ValueKey('choice:$i')),
              matching: find.byType(Material),
            ),
          ),
      ];
      final columns = faces.where((r) => r.top == faces.first.top).length;
      for (var i = 0; i < faces.length; i++) {
        expect(faces[i].width, closeTo(faces.first.width, .01));
        expect(faces[i].left, closeTo(faces[i % columns].left, .01));
        expect(
          tester.getSize(find.byKey(ValueKey('choice:$i'))).height,
          greaterThanOrEqualTo(48),
        );
      }
      final icon = tester.getRect(find.byIcon(Icons.check));
      final text = tester.getRect(find.text('M1'));
      expect(text.left - icon.right, closeTo(6, .01));
      expect((icon.left + text.right) / 2, closeTo(faces[1].center.dx, .01));
      expect(
        tester
            .getSemantics(find.byKey(const ValueKey('choice:1')))
            .getSemanticsData()
            .flagsCollection
            .isSelected,
        ui.Tristate.isTrue,
      );
      expect(find.byIcon(Icons.check), findsOneWidget);
      semantics.dispose();
      expect(tester.takeException(), isNull);
    });
  }
}
