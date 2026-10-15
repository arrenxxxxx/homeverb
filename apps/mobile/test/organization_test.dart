import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

// Isolated multi-device organization fixture; no physical identity is used.
class OrganizationFixture implements CoreClient {
  final calls = <Json>[];
  final rooms = <Json>[
    {'id': 'room', 'name': 'Study'},
    {'id': 'other', 'name': 'Kitchen'},
  ];
  List<Json> cards = [
    for (final id in ['a', 'x', 'b', 'y', 'c']) card(id),
  ];
  static Json card(String id) => {
    'device': {
      'id': id,
      'alias': 'Same name',
      'model': 'Fixture',
      'components': [
        {'id': 'main', 'capabilities': []},
      ],
    },
    'preferences': {
      'alias': 'Same name',
      'room_id': ['x', 'y'].contains(id) ? 'other' : 'room',
      'favorite': id == 'a',
    },
  };
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    calls.add({'method': method, ...fields});
    switch (method) {
      case 'list_rooms':
        return jsonDecode(jsonEncode(rooms));
      case 'list_device_cards':
        return jsonDecode(jsonEncode(cards));
      case 'reorder_devices':
      case 'reorder_rooms':
        final current = method == 'reorder_devices' ? cards : rooms;
        String id(Json row) =>
            method == 'reorder_devices' ? row['device']['id'] : row['id'];
        final ids =
            (fields[method == 'reorder_devices' ? 'device_ids' : 'room_ids']
                    as List)
                .cast<String>();
        if (ids.length != current.length ||
            ids.toSet().length != ids.length ||
            !ids.toSet().containsAll(current.map(id))) {
          throw NativeFailure(
            'INVALID_ARGUMENT',
            'Ordering must contain every current ID exactly once.',
          );
        }
        final next = ids
            .map((key) => current.singleWhere((r) => id(r) == key))
            .toList();
        current
          ..clear()
          ..addAll(next);
        return [];
      default:
        throw StateError('Unexpected organization fixture method: $method');
    }
  }

  @override
  void cancelAll() {}
  @override
  void foreground(bool value) {}
  @override
  Future<void> close() async {}
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
  });
  test(
    'filtered reorder preserves hidden slots and stale lists reload without replay',
    () async {
      final core = OrganizationFixture();
      final c = HomeController(
        core,
        LocalPreferences(File('/unused/preferences.json')),
      );
      await c.initialize();
      c.filter = 'room';
      final metadata = {
        for (final row in core.cards) row['device']['id']: jsonEncode(row),
      };
      expect(await c.reorderDevices(['c', 'a', 'b']), true);
      expect(core.cards.map((row) => row['device']['id']), [
        'c',
        'x',
        'a',
        'y',
        'b',
      ]);
      for (final row in core.cards) {
        expect(jsonEncode(row), metadata[row['device']['id']]);
      }
      final before = core.calls
          .where((call) => call['method'] == 'reorder_devices')
          .length;
      expect(await c.reorderDevices(['a', 'a', 'b']), false);
      expect(await c.reorderDevices(['a', 'b']), false);
      expect(
        core.calls.where((call) => call['method'] == 'reorder_devices').length,
        before,
      );
      core.cards.insert(1, OrganizationFixture.card('new'));
      final staleOrder = core.cards.map((r) => r['device']['id']).toList();
      expect(await c.reorderDevices(['b', 'c', 'a']), false);
      expect((c.error as NativeFailure).message, contains('refreshed order'));
      expect(
        core.calls.where((call) => call['method'] == 'reorder_devices').length,
        before + 1,
      );
      expect(c.cards.map((r) => r['device']['id']), staleOrder);
      expect(core.cards.map((r) => r['device']['id']), staleOrder);
      core.rooms.add({'id': 'new-room', 'name': 'Bedroom'});
      expect(await c.reorderRooms(['other', 'room']), false);
      expect(c.rooms.map((r) => r['id']), ['room', 'other', 'new-room']);
      expect(
        core.calls.where((call) => call['method'] == 'reorder_rooms').length,
        1,
      );
    },
  );
  testWidgets(
    'device drag and exact-ID menus preserve hidden cards and caller navigation',
    (tester) async {
      tester.view.physicalSize = const Size(780, 1688);
      tester.view.devicePixelRatio = 2;
      tester.view.padding = const FakeViewPadding(top: 88, bottom: 28);
      debugDisableShadows = false;
      WidgetController.hitTestWarningShouldBeFatal = true;
      final core = OrganizationFixture();
      final c = HomeController(
        core,
        LocalPreferences(File('/unused/preferences.json')),
      );
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      final app = GlobalKey<NativeAppState>(), boundary = GlobalKey();
      Future<void> tap(Finder f) async {
        await tester.ensureVisible(f);
        await tester.tap(f);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
      }

      try {
        await tester.pumpWidget(
          RepaintBoundary(
            key: boundary,
            child: NativeApp(key: app, catalog: catalog, controller: c),
          ),
        );
        await tester.runAsync(
          () => precacheImage(
            const AssetImage('assets/window-light-background.png'),
            tester.element(find.byType(NativeApp)),
          ),
        );
        await tester.pumpAndSettle();
        c.filter = 'room';
        c.selectedDevice = 'a';
        await app.currentState!.go('D05');
        await tester.pumpAndSettle();
        await tap(find.text('Reorder Devices'));
        expect(app.currentState!.id, 'D10');
        expect(find.text('Same name'), findsNWidgets(3));
        await tap(find.byKey(const ValueKey('order-menu:a')));
        expect(
          tester
              .widget<PopupMenuItem<int>>(
                find.widgetWithText(PopupMenuItem<int>, 'Move up'),
              )
              .enabled,
          false,
        );
        await tap(find.text('Move down'));
        expect(core.cards.map((r) => r['device']['id']), [
          'b',
          'x',
          'a',
          'y',
          'c',
        ]);
        final handle = find.byKey(const ValueKey('order-menu:c'));
        final drag = await tester.startGesture(tester.getCenter(handle));
        await tester.pump();
        await drag.moveBy(const Offset(0, -30));
        await tester.pump(const Duration(milliseconds: 300));
        await drag.moveBy(const Offset(0, -40));
        await tester.pump(const Duration(milliseconds: 300));
        await drag.up();
        await tester.pumpAndSettle();
        expect(c.visibleCards.map((r) => r['device']['id']), ['b', 'c', 'a']);
        expect(core.cards.map((r) => r['device']['id']), [
          'b',
          'x',
          'c',
          'y',
          'a',
        ]);
        await tester.runAsync(() async {
          final image =
              await (boundary.currentContext!.findRenderObject()
                      as RenderRepaintBoundary)
                  .toImage(pixelRatio: 2);
          final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
          final dir = Directory('build/verification/organization')
            ..createSync(recursive: true);
          File(
            '${dir.path}/D10.png',
          ).writeAsBytesSync(bytes!.buffer.asUint8List());
          image.dispose();
        });
        await tap(find.text('Done'));
        expect(app.currentState!.id, 'D05');
        expect(c.selectedDevice, 'a');
        expect(c.filter, 'room');
      } finally {
        await tester.pumpWidget(const SizedBox());
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
        tester.view.resetPadding();
        debugDisableShadows = true;
        WidgetController.hitTestWarningShouldBeFatal = false;
      }
    },
  );
}
