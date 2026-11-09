// Opt-in historical baseline of actual NativeApp pages using fictional data.
// The captures describe the pre-implementation behavior, not UX acceptance.
import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'accounts_test.dart' show AccountPreferences;
import 'interaction_fixture.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const capture = bool.fromEnvironment('HOMEVERB_INTERACTION_BASELINE');
  late ScreenCatalog catalog;
  setUpAll(() async {
    if (!capture) return;
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
    catalog = await ScreenCatalog.load();
  });

  testWidgets('capture delayed entry and terminal first read failure', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(780, 1688);
    tester.view.devicePixelRatio = 2;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final core = await InteractionFixture.open();
    final device = core.device['id'] as String;
    final gate = Completer<void>();
    core.cachedGates[(device, 'fridge')] = gate;
    final controller = HomeController(core, AccountPreferences());
    final app = GlobalKey<NativeAppState>();
    final boundary = GlobalKey();
    addTearDown(controller.dispose);
    await tester.pumpWidget(
      TranslationProvider(
        child: RepaintBoundary(
          key: boundary,
          child: NativeApp(key: app, catalog: catalog, controller: controller),
        ),
      ),
    );
    await tester.pumpAndSettle();
    await tester.runAsync(
      () => precacheImage(
        const AssetImage('assets/window-light-background.png'),
        app.currentContext!,
      ),
    );
    await tester.pumpAndSettle();
    Future<void> save(String name) async {
      await tester.runAsync(() async {
        final image =
            await (boundary.currentContext!.findRenderObject()!
                    as RenderRepaintBoundary)
                .toImage(pixelRatio: 2);
        final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
        final directory = Directory('build/verification/interaction-baseline')
          ..createSync(recursive: true);
        await File(
          '${directory.path}/$name.png',
        ).writeAsBytes(bytes!.buffer.asUint8List());
        image.dispose();
      });
    }

    await tester.tap(find.text('Example refrigerator').first);
    await tester.pump();
    await tester.pump(const Duration(seconds: 5));
    expect(app.currentState!.id, 'D01');
    expect(controller.busy, true);
    expect(core.callsFor('read_state'), hasLength(1));
    await save('held-entry');
    gate.complete();
    await tester.pumpAndSettle();
    expect(app.currentState!.id, 'C08');
    core.failures.add((device, 'fridge'));
    core.failures.add((device, 'freezer'));
    await app.currentState!.action('Refresh State');
    await tester.pump();
    expect(controller.error, isNotNull);
    expect(core.callsFor('execute'), isEmpty);
    await save('terminal-read-failure');
    await tester.pumpWidget(const SizedBox());
  }, skip: !capture);
}
