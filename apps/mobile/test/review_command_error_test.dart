import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/device_page.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart';

import 'climate_controls_test.dart' show ClimateFixture;

void main() {
  testWidgets('a denied new command is not masked by an older receipt', (
    tester,
  ) async {
    final core = ClimateFixture();
    final controller = HomeController(core, LocalPreferences.memory());
    final key = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: key, catalog: catalog, controller: controller),
    );
    await tester.pumpAndSettle();
    expect(await controller.selectDevice(core.device['id']), true);
    await key.currentState!.go('C01');
    await tester.pumpAndSettle();
    expect(
      await key.currentState!.executeOnDevice('power', 'set', {'state': false}),
      true,
    );
    await tester.pumpAndSettle();
    expect(controller.receipt?['status'], 'accepted');
    core.denyPlan = true;
    expect(
      await key.currentState!.executeOnDevice('power', 'set', {'state': true}),
      false,
    );
    await tester.pumpAndSettle();
    expect(
      tester.widget<DevicePage>(find.byType(DevicePage)).error,
      same(controller.error),
    );
    expect(controller.error, isNotNull);
    expect(find.text('Accepted'), findsNothing);
    await tester.pumpWidget(const SizedBox());
  });
}
