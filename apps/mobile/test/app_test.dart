import 'dart:io';
import 'dart:convert';
import 'package:file_selector_platform_interface/file_selector_platform_interface.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

class SavePicker extends FileSelectorPlatform {
  String? path;
  bool fail = false;
  int calls = 0;
  @override
  Future<FileSaveLocation?> getSaveLocation({
    List<XTypeGroup>? acceptedTypeGroups,
    SaveDialogOptions options = const SaveDialogOptions(),
  }) async {
    calls++;
    if (fail) throw const FileSystemException('Test save failure');
    return path == null ? null : FileSaveLocation(path!);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  testWidgets(
    'click real native discovery, control, organization, preferences and unavailable gates',
    (tester) async {
      tester.view.physicalSize = const Size(780, 1688);
      tester.view.devicePixelRatio = 2;
      final directory = Directory.systemTemp.createTempSync(
        'homeverb-ui-click-',
      );
      late NativeClient core;
      late HomeController controller;
      late ScreenCatalog catalog;
      await tester.runAsync(() async {
        catalog = await ScreenCatalog.load();
        core = await NativeClient.open(
          dataDir: directory.path,
          libraryPath: File(
            '../../target/debug/libhomeverb_ffi.so',
          ).absolute.path,
          allowMockWrites: true,
        );
        controller = HomeController(
          core,
          LocalPreferences(File('${directory.path}/ui.json')),
        );
      });
      final app = GlobalKey<NativeAppState>();
      final originalPicker = FileSelectorPlatform.instance;
      final picker = SavePicker();
      FileSelectorPlatform.instance = picker;
      Future<void> settle() async {
        for (var i = 0; i < 200; i++) {
          await tester.pump(const Duration(milliseconds: 100));
          await tester.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 10)),
          );
          if (i >= 5 && !controller.busy) break;
        }
        expect(controller.busy, false, reason: 'Native operation must finish');
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
      }

      Future<void> tap(String label) async {
        final finder = find.text(label).last;
        await tester.ensureVisible(finder);
        await tester.runAsync(() async {
          await tester.tap(finder);
          await Future<void>.delayed(const Duration(milliseconds: 80));
        });
        await settle();
      }

      Future<void> route(String id) async {
        await tester.runAsync(() => app.currentState!.go(id));
        await settle();
      }

      try {
        await tester.runAsync(
          () => tester.pumpWidget(
            NativeApp(key: app, catalog: catalog, controller: controller),
          ),
        );
        await settle();
        expect(find.text('Your home, together'), findsOneWidget);
        controller.busy = true;
        controller.notifyListeners();
        await tester.pump();
        expect(find.text('Bedroom AC'), findsNothing);
        expect(find.text('Floor lamp'), findsNothing);
        controller.busy = false;
        controller.notifyListeners();
        await tester.pumpAndSettle();
        await tester.tap(find.byTooltip('Add device'));
        await settle();
        await tap('Discover Mock device');
        expect(controller.cards.length, 1);
        await tester.runAsync(() => tester.tap(find.text('mock-ac').first));
        await settle();
        expect(app.currentState!.id, 'C01');
        await tap('Refresh State');
        expect(controller.snapshot, isNotNull);
        await tap('Mode');
        expect(find.text('Bedroom AC'), findsNothing);
        await tap('heat');
        await tap('Save Mode');
        expect(app.currentState!.id, 'S06');
        expect(controller.receipt!['status'], 'verified');
        await route('C14');
        await tester.enterText(
          find.byKey(const ValueKey('field:Target temperature')),
          '100',
        );
        await tap('Save target');
        expect(find.text('Maximum: 30'), findsOneWidget);
        await tester.enterText(
          find.byKey(const ValueKey('field:Target temperature')),
          '23.5',
        );
        await tap('Save target');
        expect(controller.receipt!['status'], 'verified');
        await route('R01');
        await tap('Add Room');
        await tester.enterText(
          find.byKey(const ValueKey('field:Room name')),
          'Study',
        );
        await tap('Save Room');
        expect(controller.rooms.single['name'], 'Study');
        await tap('Add Room');
        for (final invalid in ['', 'x' * 81, 'Room\u0001name']) {
          final input = find.byKey(const ValueKey('field:Room name'));
          await tester.enterText(input, invalid);
          await tap('Save Room');
          expect(
            find.text('Use 1–80 characters without control characters.'),
            findsOneWidget,
          );
          expect(controller.rooms, hasLength(1));
          expect(app.currentState!.id, 'R02');
          expect(tester.widget<TextField>(input).controller!.text, invalid);
        }
        await tester.enterText(
          find.byKey(const ValueKey('field:Room name')),
          'Study',
        );
        await tap('Save Room');
        expect(controller.error, isNotNull);
        expect(
          find.text('Use 1–80 characters without control characters.'),
          findsNothing,
        );
        await route('D07');
        await tester.enterText(
          find.byKey(const ValueKey('field:Local name')),
          'Quiet AC',
        );
        await tap('Save Name');
        expect(controller.currentCard!['device']['alias'], 'Quiet AC');
        await route('R03');
        await tester.tap(find.byType(CheckboxListTile).first);
        await tester.pumpAndSettle();
        await tap('Move Devices');
        expect(
          controller.currentCard!['preferences']['room_id'],
          controller.rooms.single['id'],
        );
        await route('D05');
        await tap('Pin to Top');
        expect(controller.currentCard!['preferences']['favorite'], true);
        await route('D05');
        await tap('Unpin');
        expect(controller.currentCard!['preferences']['favorite'], false);
        await route('D05');
        await tap('Select Devices');
        await tap('Apply to Selected Devices');
        expect((controller.error as NativeFailure).code, 'INVALID_ARGUMENT');
        await tester.tap(find.byType(CheckboxListTile).first);
        await tester.pumpAndSettle();
        await tap('Apply to Selected Devices');
        expect(controller.error, isNull);
        expect(controller.batchResult!['status'], 'verified');
        expect(controller.batchResult!['items'], hasLength(1));
        await route('D03');
        await tester.enterText(
          find.byKey(const ValueKey('field:Search devices')),
          'missing',
        );
        await tester.pumpAndSettle();
        expect(controller.visibleCards, isEmpty);
        await tester.tap(find.byTooltip('Clear search'));
        await tester.pumpAndSettle();
        expect(controller.visibleCards.length, 1);
        await tester.enterText(
          find.byKey(const ValueKey('field:Search devices')),
          'missing',
        );
        await route('D01');
        expect(controller.search, isEmpty);
        await route('D08');
        await tap('Local network only — no cloud fallback');
        await tap('Save Preference');
        expect(controller.preferences.route(controller.selectedDevice), 'lan');
        await route('D08');
        await tap('Save Preference');
        expect(controller.preferences.route(controller.selectedDevice), 'lan');
        await route('R01');
        await tester.tap(find.byTooltip('Room actions Study'));
        await tester.pumpAndSettle();
        await tap('Rename');
        await tester.enterText(
          find.byKey(const ValueKey('field:Room name')),
          'Office',
        );
        await tap('Save Room');
        expect(controller.rooms.single['name'], 'Office');
        await tap('Add Room');
        await tester.enterText(
          find.byKey(const ValueKey('field:Room name')),
          'Bedroom',
        );
        await tap('Save Room');
        await tester.tap(find.byTooltip('Room actions Bedroom'));
        await tester.pumpAndSettle();
        await tap('Move up');
        expect(controller.rooms.first['name'], 'Bedroom');
        await tester.tap(find.byTooltip('Room actions Office'));
        await tester.pumpAndSettle();
        await tap('Delete');
        await tap('Delete Room');
        expect(controller.rooms.length, 1);
        expect(controller.currentCard!['preferences']['room_id'], isNull);
        await route('S10');
        await tester.enterText(
          find.byKey(const ValueKey('field:Sharing name')),
          'Study phone',
        );
        await tap('Save Name');
        expect(controller.preferences.sharingName, 'Study phone');
        await route('S05');
        expect(controller.history, isNotEmpty);
        await tester.runAsync(() => tester.tap(find.byType(ListTile).first));
        await settle();
        expect(app.currentState!.id, 'S06');
        controller.selectedDevice = null;
        await tap('Refresh State');
        expect(controller.error, isNull);
        await route('S07');
        expect(controller.diagnostics, isNotNull);
        final exact = const JsonEncoder.withIndent(
          '  ',
        ).convert(controller.diagnostics);
        await tap('Export Redacted Report');
        expect(picker.calls, 1);
        expect(controller.error, isNull);
        picker.path = '${directory.path}/report.json';
        await tap('Export Redacted Report');
        expect(File(picker.path!).readAsStringSync(), exact);
        picker.fail = true;
        await tap('Export Redacted Report');
        expect((controller.error as NativeFailure).code, 'EXPORT_FAILED');
        expect(File(picker.path!).readAsStringSync(), exact);
        await route('S08');
        await tap('Privacy');
        expect(find.byType(AlertDialog), findsOneWidget);
        await tap('Close');
        await tap('Licenses');
        expect(find.byType(LicensePage), findsOneWidget);
        app.currentState!.navigator.currentState!.pop();
        await settle();
        for (final id in ['P01', 'P02', 'H01', 'H05', 'S04', 'S09']) {
          await route(id);
          expect(
            find
                    .textContaining('unavailable', findRichText: true)
                    .evaluate()
                    .isNotEmpty ||
                find.text('Unavailable').evaluate().isNotEmpty,
            isTrue,
            reason: id,
          );
        }
        tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
        await expectLater(
          core.call('list_rooms'),
          throwsA(isA<NativeFailure>()),
        );
        tester.binding.handleAppLifecycleStateChanged(
          AppLifecycleState.resumed,
        );
        expect(controller.cards.length, 1);
      } finally {
        FileSelectorPlatform.instance = originalPicker;
        await tester.pumpWidget(const SizedBox());
        await tester.runAsync(() => core.close());
        directory.deleteSync(recursive: true);
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      }
    },
  );
}
