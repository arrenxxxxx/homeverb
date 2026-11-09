import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/room_row.dart';
import 'package:homeverb_mobile/skin.dart';

void main() {
  testWidgets(
    'room drag grip and visible menu are independent and accessible',
    (tester) async {
      tester.view.physicalSize = const Size(320, 800);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final semantics = tester.ensureSemantics();
      final rooms = ['Study', 'Kitchen'];
      final actions = <String>[];
      var reorders = 0;
      await tester.pumpWidget(
        MaterialApp(
          theme: windowTheme(),
          home: MediaQuery(
            data: const MediaQueryData(
              size: Size(320, 800),
              textScaler: TextScaler.linear(2),
            ),
            child: Scaffold(
              body: StatefulBuilder(
                builder: (context, update) => ReorderableListView.builder(
                  buildDefaultDragHandles: false,
                  itemCount: rooms.length,
                  onReorderItem: (from, to) => update(() {
                    reorders++;
                    rooms.insert(to, rooms.removeAt(from));
                  }),
                  itemBuilder: (context, index) {
                    final room = rooms[index];
                    return RoomRow(
                      key: ValueKey(room),
                      id: room,
                      name: room,
                      deviceCount: '2 devices',
                      index: index,
                      canMoveUp: index > 0,
                      canMoveDown: index + 1 < rooms.length,
                      onAction: (action) => actions.add('$room:$action'),
                    );
                  },
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final drag = find.byKey(const ValueKey('room-drag:Study'));
      final menu = find.byKey(const ValueKey('room-actions:Study'));
      expect(tester.getSize(drag).width, greaterThanOrEqualTo(48));
      expect(tester.getSize(drag).height, greaterThanOrEqualTo(48));
      expect(tester.getSize(menu).width, greaterThanOrEqualTo(48));
      expect(tester.getSize(menu).height, greaterThanOrEqualTo(48));
      expect(
        tester.getSemantics(drag).getSemanticsData().label,
        contains('Reorder Study'),
      );
      expect(find.byTooltip('Room actions Study'), findsOneWidget);
      expect(tester.getRect(drag).overlaps(tester.getRect(menu)), isFalse);

      await tester.tap(menu);
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<PopupMenuItem<String>>(
              find.widgetWithText(PopupMenuItem<String>, 'Move up'),
            )
            .enabled,
        isFalse,
      );
      expect(
        tester
            .widget<PopupMenuItem<String>>(
              find.widgetWithText(PopupMenuItem<String>, 'Move down'),
            )
            .enabled,
        isTrue,
      );
      await tester.tap(find.text('Move down'));
      await tester.pumpAndSettle();
      expect(actions, ['Study:Move down']);
      expect(reorders, 0);
      expect(tester.takeException(), isNull);

      final gesture = await tester.startGesture(tester.getCenter(drag));
      await tester.pump();
      await gesture.moveBy(const Offset(0, 30));
      await tester.pump(const Duration(milliseconds: 300));
      await gesture.moveBy(const Offset(0, 80));
      await tester.pump(const Duration(milliseconds: 300));
      await gesture.moveBy(const Offset(0, 60));
      await tester.pump(const Duration(milliseconds: 300));
      await gesture.up();
      await tester.pumpAndSettle();
      expect(reorders, 1);
      expect(rooms, ['Kitchen', 'Study']);
      expect(actions, ['Study:Move down']);
      expect(tester.takeException(), isNull);
      semantics.dispose();
    },
  );
}
