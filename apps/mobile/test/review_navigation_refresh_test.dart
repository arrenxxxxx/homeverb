import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart';

import 'accounts_test.dart' show AccountPreferences;
import 'device_opening_test.dart' show OpeningClient, deviceA;
import 'interaction_fixture.dart';

void main() {
  testWidgets(
    'opening a card during manual refresh enters immediately and drains',
    (tester) async {
      final inner = await InteractionFixture.open();
      final core = OpeningClient(inner);
      final controller = HomeController(core, AccountPreferences());
      final app = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: app, catalog: catalog, controller: controller),
      );
      await tester.pumpAndSettle();
      final gate = inner.hold(deviceA, 'fridge');
      final refresh = app.currentState!.action('Refresh State');
      await tester.pump();
      expect(inner.activeReads, 1);
      final opening = app.currentState!.action('open:$deviceA');
      await tester.pump();
      expect(app.currentState!.id, anyOf('C01', 'C08'));
      expect(inner.activeReads, 1);
      expect(inner.maxActiveReads, 1);
      gate.complete();
      await refresh;
      await opening;
      await tester.pumpAndSettle();
      expect(controller.selectedDevice, deviceA);
      expect(inner.maxActiveReads, 1);
      await tester.pumpWidget(const SizedBox());
    },
  );
}
