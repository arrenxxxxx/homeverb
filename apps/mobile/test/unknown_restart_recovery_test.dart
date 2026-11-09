import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

class CountingNativeClient implements NativeSessionClient {
  CountingNativeClient(this.inner);
  final NativeClient inner;
  final calls = <String>[];
  Future<void>? _closing;

  @override
  Set<String> get methods => inner.methods;
  @override
  bool apiAtLeast(int minor) => inner.apiAtLeast(minor);
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) {
    calls.add(method);
    return inner.call(method, fields);
  }

  @override
  void cancelAll() => inner.cancelAll();
  @override
  void foreground(bool value) => inner.foreground(value);
  @override
  Future<void> close() => _closing ??= inner.close();
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
  });

  testWidgets(
    'unknown receipt survives a new App and native session without replay',
    (tester) async {
      tester.view.physicalSize = const Size(780, 1688);
      tester.view.devicePixelRatio = 2;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final directory = Directory.systemTemp.createTempSync(
        'homeverb-unknown-',
      );
      final library = File(
        '../../target/debug/${NativeClient.defaultLibrary()}',
      ).absolute.path;
      late ScreenCatalog catalog;
      late CountingNativeClient client;
      late HomeController controller;
      var app = GlobalKey<NativeAppState>();

      Future<void> open({required bool allowMockWrites}) async {
        await tester.runAsync(() async {
          client = CountingNativeClient(
            await NativeClient.open(
              dataDir: directory.path,
              libraryPath: library,
              allowMockWrites: allowMockWrites,
              scenario: 'lost-ack',
            ),
          );
          controller = HomeController(
            client,
            LocalPreferences(File('${directory.path}/ui.json')),
          );
          catalog = await ScreenCatalog.load();
        });
        app = GlobalKey<NativeAppState>();
        await tester.pumpWidget(
          NativeApp(key: app, catalog: catalog, controller: controller),
        );
      }

      Future<void> settle() async {
        for (var i = 0; i < 200; i++) {
          await tester.pump(const Duration(milliseconds: 100));
          await tester.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 10)),
          );
          if (i >= 5 && !controller.busy) break;
        }
        expect(controller.busy, false);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
      }

      Future<void> tap(String label) async {
        final target = find.text(label).last;
        await tester.ensureVisible(target);
        await tester.runAsync(() => tester.tap(target));
        await settle();
      }

      try {
        await open(allowMockWrites: true);
        await settle();
        await tester.tap(find.byTooltip('Add device'));
        await settle();
        await tap('Discover Mock device');
        await tap('mock-ac');
        final power = find.byKey(const ValueKey('climate-power'));
        await tester.runAsync(() => tester.tap(power));
        await settle();
        expect(controller.receipt!['status'], 'unknown');
        final request = controller.receipt!['request_id'] as String;
        expect(
          client.calls.where((method) => method == 'execute'),
          hasLength(1),
        );
        await tap('View Details');
        expect(app.currentState!.id, 'S06');
        await tap('Refresh State');
        expect(controller.receipt!['status'], 'unknown');
        expect(
          client.calls.where((method) => method == 'execute'),
          hasLength(1),
        );

        // Close the original widget tree and native session, then reopen the same
        // isolated fictional profile in a fresh App and read-only session.
        final original = client;
        await tester.runAsync(original.close);
        await tester.pumpWidget(const SizedBox());
        await open(allowMockWrites: false);
        await settle();
        await tap('mock-ac');
        await tap('Recent Operations');
        expect(controller.history, hasLength(1));
        expect(controller.history.single['request_id'], request);
        expect(controller.history.single['status'], 'unknown');
        final entry = find.textContaining('Unknown').last;
        await tester.ensureVisible(entry);
        await tester.runAsync(() => tester.tap(entry));
        await settle();
        expect(app.currentState!.id, 'S06');
        expect(controller.receipt!['request_id'], request);
        expect(controller.receipt!['status'], 'unknown');
        await tap('Refresh State');
        expect(controller.receipt!['request_id'], request);
        expect(controller.receipt!['status'], 'unknown');
        expect(client.calls.where((method) => method == 'plan'), isEmpty);
        expect(client.calls.where((method) => method == 'execute'), isEmpty);
      } finally {
        await tester.runAsync(client.close);
        await tester.pumpWidget(const SizedBox());
        directory.deleteSync(recursive: true);
      }
    },
  );
}
