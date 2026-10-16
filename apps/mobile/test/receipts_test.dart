import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/receipts.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

Json fixture(String name) =>
    jsonDecode(File('../../examples/$name.json').readAsStringSync()) as Json;

// Isolated receipt fixtures; no native adapter or physical device is registered.
class ReceiptFixture implements CoreClient {
  final calls = <Json>[];
  final parent = fixture('command-result.storage-failure');
  late final second = {
    ...fixture('command-result.completion'),
    'request_id': 'second-request',
    'device_id': 'second-device',
  };
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    calls.add({'method': method, ...fields});
    switch (method) {
      case 'list_rooms':
        return <Json>[];
      case 'list_device_cards':
        return [
          for (final id in [parent['device_id'], second['device_id']])
            {
              'device': {'id': id, 'alias': 'Same name', 'model': 'Fixture'},
              'preferences': {'alias': null, 'room_id': null},
            },
        ];
      case 'recent_executions':
        return [parent, second];
      case 'get_execution':
        return [
          parent,
          second,
        ].singleWhere((r) => r['request_id'] == fields['request_id']);
      case 'describe_device':
        return {
          'device': {
            'components': [
              {'id': 'main'},
            ],
          },
        };
      case 'read_state':
        return {...fixture('state'), 'device_id': fields['device']};
      default:
        throw StateError('Unexpected receipt fixture call: $method');
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
  test('receipt copy preserves evidence limits and UTC timestamps', () {
    final accepted = receiptBlocks(
      fixture('command-result.accepted'),
      'Device',
    );
    final completion = receiptBlocks(
      fixture('command-result.completion'),
      'Device',
    );
    expect(jsonEncode(accepted), contains('No verification evidence'));
    expect(jsonEncode(completion), contains('Correlated completion'));
    expect(jsonEncode(completion), contains('exactly-once'));
    expect(jsonEncode(completion), contains('Returned values'));
    expect(jsonEncode(completion), isNot(contains('Requested values')));
    expect(
      receiptTime('2026-10-14T13:00:00+08:00', compact: true),
      '2026-10-14 05:00 UTC',
    );
    expect(receiptTime(null), 'Not provided');
  });
  testWidgets(
    'duplicate names open exact history and nested batch evidence without replay',
    (tester) async {
      tester.view.physicalSize = const Size(780, 1688);
      tester.view.devicePixelRatio = 2;
      WidgetController.hitTestWarningShouldBeFatal = true;
      final dir = Directory.systemTemp.createTempSync('homeverb-receipts-');
      final core = ReceiptFixture();
      final controller = HomeController(
        core,
        LocalPreferences(File('${dir.path}/ui.json')),
      );
      final catalog = await tester.runAsync(ScreenCatalog.load);
      final app = GlobalKey<NativeAppState>();
      Future<void> tap(Finder target) async {
        await tester.ensureVisible(target);
        await tester.tap(target);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
      }

      try {
        await tester.pumpWidget(
          NativeApp(key: app, catalog: catalog!, controller: controller),
        );
        await tester.pumpAndSettle();
        await app.currentState!.go('S05');
        await tester.pumpAndSettle();
        expect(find.text('Same name'), findsNWidgets(2));
        await tap(find.byKey(const ValueKey('receipt:second-request')));
        expect(controller.receipt, same(core.second));
        expect(find.text('Correlated completion'), findsOneWidget);
        await tap(find.byTooltip('Back'));
        await tap(find.byKey(ValueKey('receipt:${core.parent['request_id']}')));
        expect(find.text('Partially completed'), findsOneWidget);
        await tap(find.byKey(const ValueKey('receipt-item:0')));
        expect(controller.receipt, same(core.parent['items'][0]));
        expect(find.textContaining('Warning:'), findsOneWidget);
        final child = controller.receipt;
        await tap(find.text('Refresh State'));
        expect(controller.receipt, same(child));
        expect(core.calls.last['method'], 'read_state');
        expect(core.calls.last['device'], child!['device_id']);
        await tap(find.byTooltip('Back'));
        expect(controller.receipt, same(core.parent));
        await tap(find.byKey(const ValueKey('receipt-item:1')));
        expect(find.text('NOT_EXECUTED'), findsOneWidget);
        await tap(find.byTooltip('Back'));
        await tap(find.byTooltip('Back'));
        expect(app.currentState!.id, 'S05');
        controller.batchResult = {
          'status': 'partial',
          'items': [core.parent, core.second],
        };
        await app.currentState!.go('C12');
        await tester.pumpAndSettle();
        expect(find.text('Same name'), findsNWidgets(2));
        await tap(find.byKey(const ValueKey('batch-item:1')));
        expect(controller.receipt, same(core.second));
        await tap(find.byTooltip('Back'));
        expect(app.currentState!.id, 'C12');
        await tap(find.byKey(const ValueKey('batch-item:0')));
        await tap(find.byKey(const ValueKey('receipt-item:0')));
        await tap(find.byTooltip('Back'));
        expect(controller.receipt, same(core.parent));
        await tap(find.byTooltip('Back'));
        expect(app.currentState!.id, 'C12');
        tester.view.physicalSize = const Size(640, 1688);
        tester.platformDispatcher.textScaleFactorTestValue = 2;
        for (final screen in ['S05', 'C12', 'S06']) {
          await app.currentState!.go(screen);
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull, reason: screen);
        }
        expect(
          core.calls.where((c) => c['method'] == 'get_execution').length,
          2,
        );
        expect(
          core.calls.where((c) => c['method'].toString().startsWith('execute')),
          isEmpty,
        );
      } finally {
        await tester.pumpWidget(const SizedBox());
        dir.deleteSync(recursive: true);
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
        tester.platformDispatcher.clearTextScaleFactorTestValue();
        WidgetController.hitTestWarningShouldBeFatal = false;
      }
    },
  );
}
