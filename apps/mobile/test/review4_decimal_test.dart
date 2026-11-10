import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/skin.dart';

import 'component_test.dart' show CompartmentFixture;
import 'review2_compartment_test.dart' show CombinedGrant;

void main() {
  Future<void> open(
    WidgetTester tester,
    HomeController controller, {
    GlobalKey<NativeAppState>? app,
  }) async {
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: controller),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Example refrigerator').first);
    await tester.pumpAndSettle();
  }

  for (final scenario in <(num, num, num, num, bool, num?)>[
    (4.1, 0.1, 0, 8, true, 4.2),
    (4.1, 0.1, 0, 8, false, 4.0),
    (-4.1, 0.1, -8, 0, true, -4.0),
    (-4.1, 0.1, -8, 0, false, -4.2),
    (4.25, 0.25, 0, 8, true, 4.5),
    (4.2, 0.1, 0, 4.2, true, null),
    (0, 0.1, 0, 8, false, null),
    (4.15, 0.1, 0, 8, true, null),
  ]) {
    testWidgets(
      'decimal step ${scenario.$1} ${scenario.$5 ? '+' : '-'} ${scenario.$2} yields ${scenario.$6}',
      (tester) async {
        final core = CompartmentFixture();
        core.targets['fridge'] = scenario.$1;
        final climate = core.capabilities.first;
        for (final schema in [
          climate['properties']['target_temperature'],
          climate['actions']['set']['parameters']['properties']['target_temperature'],
        ]) {
          schema['multipleOf'] = scenario.$2;
          schema['minimum'] = scenario.$3;
          schema['maximum'] = scenario.$4;
        }
        final c = HomeController(core, LocalPreferences.memory());
        await open(tester, c);
        final stepper = tester.widget<InlineStepper>(
          find.byKey(const ValueKey('component-stepper:fridge')),
        );
        final callback = scenario.$5 ? stepper.onIncrease : stepper.onDecrease;
        if (scenario.$6 == null) {
          expect(callback, isNull);
          expect(
            core.calls.where((call) => call['method'] == 'execute'),
            isEmpty,
          );
        } else {
          expect(callback, isNotNull);
          callback!();
          await tester.pumpAndSettle();
          final plans = core.calls
              .where((call) => call['method'] == 'plan')
              .toList();
          final writes = core.calls
              .where((call) => call['method'] == 'execute')
              .toList();
          expect(plans, hasLength(1));
          expect(writes, hasLength(1));
          expect(writes.single['command']['arguments'], {
            'target_temperature': scenario.$6,
          });
          expect(
            plans.single['command']['arguments'],
            writes.single['command']['arguments'],
          );
        }
        await tester.pumpWidget(const SizedBox());
      },
    );
  }
  testWidgets('decimal candidate matches the exact session grant and command', (
    tester,
  ) async {
    final core = CombinedGrant(arguments: {'target_temperature': 4.2});
    core.targets['fridge'] = 4.1;
    final climate = core.capabilities.first;
    climate['properties']['target_temperature']['multipleOf'] = 0.1;
    climate['actions']['set']['parameters']['properties']['target_temperature']['multipleOf'] =
        0.1;
    final controller = HomeController(core, LocalPreferences.memory());
    await open(tester, controller);
    final stepper = tester.widget<InlineStepper>(
      find.byKey(const ValueKey('component-stepper:fridge')),
    );
    expect(stepper.onIncrease, isNotNull);
    expect(stepper.onDecrease, isNull);
    stepper.onIncrease!();
    await tester.pumpAndSettle();
    final writes = core.calls
        .where((call) => call['method'] == 'execute')
        .toList();
    expect(writes, hasLength(1));
    expect(writes.single['command']['arguments'], {'target_temperature': 4.2});
    await tester.pumpWidget(const SizedBox());
  });
}
