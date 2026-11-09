import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/climate_controls.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';

import 'unknown_restart_recovery_test.dart' show CountingNativeClient;

void main() {
  testWidgets('saved readings remain visible after native grant revocation', (
    tester,
  ) async {
    final directory = Directory.systemTemp.createTempSync('homeverb-grant-');
    final library = File(
      '../../target/debug/${NativeClient.defaultLibrary()}',
    ).absolute.path;
    late CountingNativeClient first;
    CountingNativeClient? second;
    HomeController? firstController, secondController;
    try {
      await tester.runAsync(() async {
        first = CountingNativeClient(
          await NativeClient.open(
            dataDir: directory.path,
            libraryPath: library,
            allowMockWrites: true,
          ),
        );
        firstController = HomeController(first, LocalPreferences.memory());
        expect(await firstController!.discover(), true);
        final device = firstController!.cards.single['device']['id'] as String;
        expect(await firstController!.selectDevice(device), true);
        expect(firstController!.hasControlAction('power', 'set'), true);
        expect(firstController!.observed('power.state'), isA<bool>());
        await first.close();

        second = CountingNativeClient(
          await NativeClient.open(
            dataDir: directory.path,
            libraryPath: library,
          ),
        );
        secondController = HomeController(second!, LocalPreferences.memory());
        expect(await secondController!.initialize(), true);
        expect(
          await secondController!.selectDevice(device, readOnOpen: false),
          true,
        );
      });
      final c = secondController!;
      expect(c.snapshot!['from_cache'], true);
      expect(c.observed('power.state'), isA<bool>());
      expect(c.hasControlAction('power', 'set'), false);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: ClimateControls(
                controller: c,
                onAction: (_) {},
                onExecute: (_, _, _) async => false,
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('climate-power')), findsNothing);
      expect(
        find.text(
          c.observed('power.state') == true ? 'Powered on' : 'Powered off',
        ),
        findsOneWidget,
      );
      final result = await tester.runAsync(
        () => c.execute('power', 'set', {'state': !c.observed('power.state')}),
      );
      expect(result, false);
      expect(
        (c.error as NativeFailure).code,
        anyOf('CAPABILITY_UNSUPPORTED', 'POLICY_DENIED'),
      );
      expect(second!.calls.where((method) => method == 'execute'), isEmpty);
      await tester.pumpWidget(const SizedBox());
    } finally {
      firstController?.dispose();
      secondController?.dispose();
      await tester.runAsync(() async {
        await first.close();
        await second?.close();
      });
      directory.deleteSync(recursive: true);
    }
  });
}
