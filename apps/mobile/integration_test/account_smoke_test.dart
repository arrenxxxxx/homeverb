import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets(
    'empty Meiju profile navigates to protected input without vendor calls',
    (tester) async {
      final directory = await Directory.systemTemp.createTemp(
        'homeverb-account-process-',
      );
      final core = await NativeClient.open(
        dataDir: directory.path,
        adapter: 'midea.meiju',
        allowMockWrites: false,
      );
      final controller = HomeController(
        core,
        LocalPreferences(File('${directory.path}/ui.json')),
      );
      final catalog = await ScreenCatalog.load();
      final app = GlobalKey<NativeAppState>(), boundary = GlobalKey();
      Future<void> settle() async {
        await tester.pumpAndSettle();
        for (var i = 0; i < 200 && controller.busy; i++) {
          await tester.pump(const Duration(milliseconds: 20));
        }
        await tester.pumpAndSettle();
      }

      Future<void> tap(String label) async {
        final finder = find.text(label).last;
        await tester.ensureVisible(finder);
        await tester.tap(finder);
        await settle();
      }

      try {
        await tester.pumpWidget(
          RepaintBoundary(
            key: boundary,
            child: NativeApp(
              key: app,
              catalog: catalog,
              controller: controller,
            ),
          ),
        );
        await settle();
        await app.currentState!.go('P01');
        await settle();
        expect(controller.accounts, isEmpty);
        await tap('Add Platform');
        await tap('Meiju');
        expect(app.currentState!.id, 'P04');
        await tap('Sign In and Retrieve Devices');
        expect(controller.accountReport, isNull);
        expect(
          find.textContaining('Use 1–256 ASCII characters'),
          findsOneWidget,
        );
        final login = find.byKey(const ValueKey('credential:Meiju account'));
        final password = find.byKey(const ValueKey('credential:Password'));
        await tester.enterText(login, 'fixture@example.invalid');
        await tester.enterText(password, 'fictional-pass');
        await tester.pumpAndSettle();
        expect(
          tester.widget<TextField>(login).controller!.text,
          'fixture@example.invalid',
        );
        expect(
          tester.widget<TextField>(password).controller!.text,
          'fictional-pass',
        );
        tester.binding.handleAppLifecycleStateChanged(
          AppLifecycleState.inactive,
        );
        tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.hidden);
        for (final phase in [
          AppLifecycleState.inactive,
          AppLifecycleState.hidden,
          AppLifecycleState.paused,
        ]) {
          tester.binding.handleAppLifecycleStateChanged(phase);
        }
        await tester.pump();
        expect(tester.widget<TextField>(login).controller!.text, isEmpty);
        expect(tester.widget<TextField>(password).controller!.text, isEmpty);
        for (final phase in [
          AppLifecycleState.hidden,
          AppLifecycleState.inactive,
          AppLifecycleState.resumed,
        ]) {
          tester.binding.handleAppLifecycleStateChanged(phase);
        }
        await settle();
        expect(await core.call('list_accounts'), isEmpty);
        const path = String.fromEnvironment('HOMEVERB_CAPTURE');
        if (path.isNotEmpty) {
          final image =
              await (boundary.currentContext!.findRenderObject()
                      as RenderRepaintBoundary)
                  .toImage(pixelRatio: 1);
          final data = await image.toByteData(format: ui.ImageByteFormat.png);
          await File(path).writeAsBytes(data!.buffer.asUint8List());
          image.dispose();
        }
        await tap('Cancel');
        expect(app.currentState!.id, 'P02');
        expect(tester.takeException(), isNull);
      } finally {
        await tester.pumpWidget(const SizedBox());
        await core.close();
        await directory.delete(recursive: true);
      }
    },
  );
}
