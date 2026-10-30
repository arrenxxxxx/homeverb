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
}
