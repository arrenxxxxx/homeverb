import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart';
import 'climate_controls_test.dart' show ClimateFixture;

void main() {
  testWidgets('home card can open while quick power command is pending', (
    tester,
  ) async {
    final core = ClimateFixture();
    final c = HomeController(core, LocalPreferences.memory());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: c),
    );
    await tester.pumpAndSettle();
    final device = core.device['id'] as String;
    expect(c.cardPowerChoices(device), contains(false));
    core.executionGate = Completer<void>();
    final pending = app.currentState!.action('power:$device');
    for (var i = 0; i < 12; i++) await tester.pump();
    expect(core.calls.where((x) => x['method'] == 'execute'), hasLength(1));
    expect(app.currentState!.quickPowerDevice, device);
    await tester.tap(find.text('Controls fixture').first);
    await tester.pump();
    final route = app.currentState!.id;
    final during = core.calls.length;
    for (var i = 0; i < 5; i++) await tester.pump();
    expect(
      core.calls.length,
      during,
      reason: 'Device reads wait for the write to drain.',
    );
    core.executionGate!.complete();
    await pending;
    await tester.pumpAndSettle();
    expect(core.calls.where((x) => x['method'] == 'execute'), hasLength(1));
    expect(c.receipt?['status'], 'accepted');
    expect(c.receipt?['device_id'], device);
    await tester.pumpWidget(const SizedBox());
    expect(
      route,
      'C01',
      reason: 'A local pending control must not swallow device navigation.',
    );
  });
  testWidgets('background interrupts a pending quick write without replay', (
    tester,
  ) async {
    final core = ClimateFixture();
    final c = HomeController(core, LocalPreferences.memory());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: c),
    );
    await tester.pumpAndSettle();
    final device = core.device['id'] as String;
    core.executionGate = Completer<void>();
    final pending = app.currentState!.action('power:$device');
    for (var i = 0; i < 12; i++) await tester.pump();
    expect(core.calls.where((x) => x['method'] == 'execute'), hasLength(1));
    await tester.tap(find.text('Controls fixture').first);
    await tester.pump();
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    await tester.pump();
    core.executionGate!.complete();
    await pending;
    await tester.pump();
    expect(c.receipt, isNull);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    for (var i = 0; i < 20; i++) await tester.pump();
    expect(core.calls.where((x) => x['method'] == 'execute'), hasLength(1));
    await tester.pumpWidget(const SizedBox());
  });
}
