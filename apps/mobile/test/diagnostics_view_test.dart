import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/blocks.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/diagnostics_view.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/skin.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

Json reportFixture() =>
    jsonDecode(File('../../examples/diagnostics.json').readAsStringSync())
        as Json;

// This fixture returns only the versioned allowlisted report; it contacts no device.
class DiagnosticsFixture implements CoreClient {
  Completer<Json>? pending;
  bool fail = false;
  final calls = <String>[];
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    calls.add(method);
    expect(method, 'diagnostics');
    if (fail)
      throw NativeFailure('STORAGE_ERROR', 'Fixture report read failed.');
    return pending == null ? reportFixture() : pending!.future;
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
  late ScreenCatalog catalog;
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    catalog = await ScreenCatalog.load();
  });

  test(
    'diagnostic cards preserve empty, uncertain and truncated report evidence',
    () {
      final report = reportFixture();
      final before = jsonEncode(report);
      final blocks = diagnosticsBlocks(report, exporting: false);
      expect(blocks[0][1][0], ['App version', '0.2.0', null]);
      expect(blocks[0][1][1], ['Generated', '2026-10-14 10:00 UTC', null]);
      expect(blocks[1][2][0], ['Outcome', 'Unknown', null]);
      expect(blocks[1][2][1], ['Error', 'WRITE_OUTCOME_UNKNOWN', null]);
      expect(jsonDecode(jsonEncode(blocks[1][2][2])), [
        'Phase',
        'Possibly sent · LAN',
        null,
      ]);
      expect(jsonEncode(report), before);
      report['recent_receipts'][0]['attempts_omitted'] = 2;
      expect(
        jsonEncode(diagnosticsBlocks(report, exporting: false)),
        contains('LAN (retained)'),
      );
      report['device_count'] = 0;
      report['room_count'] = 0;
      report['recent_receipts'] = [];
      final empty = diagnosticsBlocks(report, exporting: true);
      expect(empty[0][1][3], ['Devices / rooms', '0 / 0', null]);
      expect(empty[1][2][0], ['Outcome', 'No recent operations', null]);
      expect(empty[1][2][3], ['Attempts omitted', '—', null]);
      expect(empty[3], ['button', 'Export Redacted Report', 'disabled']);
      expect(diagnosticsBlocks(null, exporting: false)[1], [
        'button',
        'Export Redacted Report',
        'disabled',
      ]);
    },
  );

  test(
    'failed and cancelled diagnostic loads cannot expose an old or late report',
    () async {
      final directory = Directory.systemTemp.createTempSync(
        'homeverb-diagnostics-',
      );
      final core = DiagnosticsFixture();
      final controller = HomeController(
        core,
        LocalPreferences(File('${directory.path}/ui.json')),
      );
      try {
        expect(await controller.loadDiagnostics(), isTrue);
        expect(controller.diagnostics, isNotNull);
        core.fail = true;
        expect(await controller.loadDiagnostics(), isFalse);
        expect(controller.diagnostics, isNull);
        expect((controller.error as NativeFailure).code, 'STORAGE_ERROR');
        core.fail = false;
        for (final background in [false, true]) {
          for (final fail in [false, true]) {
            core.pending = Completer<Json>();
            final load = controller.loadDiagnostics();
            expect(controller.busy, isTrue);
            expect(controller.diagnostics, isNull);
            if (background) {
              controller.setForeground(false);
            } else {
              controller.cancelPending();
            }
            if (fail) {
              core.pending!.completeError(
                NativeFailure('STORAGE_ERROR', 'Late fixture failure.'),
              );
            } else {
              core.pending!.complete(reportFixture());
            }
            await load;
            expect(controller.diagnostics, isNull);
            expect(controller.error, isNull);
            expect(controller.busy, isFalse);
            controller.setForeground(true);
          }
        }
        expect(core.calls, everyElement('diagnostics'));
      } finally {
        controller.dispose();
        directory.deleteSync(recursive: true);
      }
    },
  );

  testWidgets(
    'diagnostic cards reflow and expose a full-report review action',
    (tester) async {
      final originalShadows = debugDisableShadows;
      debugDisableShadows = false;
      tester.view.devicePixelRatio = 2;
      final output = Directory('build/verification/diagnostics')
        ..createSync(recursive: true);
      final boundary = GlobalKey();
      try {
        for (final (width, scale) in [(390.0, 1.0), (320.0, 2.0)]) {
          tester.view.physicalSize = Size(width * 2, 1688);
          final actions = <String>[];
          final screen = {
            ...catalog.ui('S07'),
            'blocks': diagnosticsBlocks(reportFixture(), exporting: false),
          };
          await tester.pumpWidget(
            RepaintBoundary(
              key: boundary,
              child: MaterialApp(
                debugShowCheckedModeBanner: false,
                theme: windowTheme(),
                home: Material(
                  child: MediaQuery(
                    data: MediaQueryData(
                      size: Size(width, 844),
                      devicePixelRatio: 2,
                      textScaler: TextScaler.linear(scale),
                    ),
                    child: ScreenView(
                      id: 'S07',
                      catalog: catalog,
                      screenOverride: screen,
                      onAction: actions.add,
                      fields: {},
                      choices: {},
                      values: {},
                      testChrome: true,
                    ),
                  ),
                ),
              ),
            ),
          );
          await tester.runAsync(
            () => precacheImage(
              const AssetImage('assets/window-light-background.png'),
              tester.element(find.byType(ScreenView)),
            ),
          );
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull);
          expect(find.byType(Rows), findsNWidgets(2));
          expect(find.text('WRITE_OUTCOME_UNKNOWN'), findsOneWidget);
          await tester.runAsync(() async {
            final image =
                await (boundary.currentContext!.findRenderObject()
                        as RenderRepaintBoundary)
                    .toImage(pixelRatio: 2);
            final bytes = await image.toByteData(
              format: ui.ImageByteFormat.png,
            );
            File(
              '${output.path}/S07-${width.toInt()}-${scale.toInt()}.png',
            ).writeAsBytesSync(bytes!.buffer.asUint8List());
            image.dispose();
          });
          await tester.ensureVisible(find.text('App version'));
          await tester.tap(find.text('App version'));
          expect(actions, isEmpty);
          await tester.ensureVisible(find.text(diagnosticReviewNote));
          await tester.tap(find.text(diagnosticReviewNote));
          expect(actions, ['Review Full Report']);
          await tester.pumpAndSettle();
        }
      } finally {
        debugDisableShadows = originalShadows;
        tester.view.resetDevicePixelRatio();
        tester.view.resetPhysicalSize();
      }
    },
  );
}
