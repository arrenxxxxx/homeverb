import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/blocks.dart' show Climate;
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/controls.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'accounts_test.dart' show AccountPreferences;
import 'climate_controls_test.dart' show ClimateFixture;
import 'interaction_fixture.dart';

class HeldClimate extends ClimateFixture {
  final gate = Completer<void>();
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'read_state') {
      if (fields['cached'] == true) {
        throw NativeFailure('CACHE_MISS', 'No fictional reading.');
      }
      await gate.future;
      throw NativeFailure('NETWORK_UNAVAILABLE', 'Fictional read failure.');
    }
    return super.call(method, fields);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('missing and non-finite readings have no invented unit', () {
    expect(propertyValue(null, {'unit': 'celsius'}), '—');
    expect(propertyValue(double.nan, {'unit': 'celsius'}), '—');
    expect(propertyValue(double.infinity, {'unit': 'celsius'}), '—');
    expect(propertyValue(24, {'unit': 'celsius'}), '24°C');
  });

  testWidgets('AC target and indoor reading settle to a bare dash on failure', (
    tester,
  ) async {
    final core = HeldClimate();
    final controller = HomeController(core, AccountPreferences());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: controller),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Controls fixture').first);
    for (var i = 0; i < 8; i++) {
      await tester.pump();
    }
    expect(controller.readingLoading('main'), true);
    expect(find.byType(CircularProgressIndicator), findsWidgets);
    core.gate.complete();
    for (var i = 0; i < 8; i++) {
      await tester.pump();
    }
    expect(controller.readingLoading('main'), false);
    expect(find.text('—°C'), findsNothing);
    expect(find.text('—'), findsWidgets);
    expect(find.text('Indoor —'), findsOneWidget);
    expect(find.text('Refresh State'), findsNothing);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('single-component climate fallback loads and fails locally', (
    tester,
  ) async {
    final core = HeldClimate();
    core.capabilities.removeWhere(
      (capability) => capability['id'] != 'climate',
    );
    core.capabilities.single['properties']['target_temperature'] = {
      'type': 'number',
      'readable': true,
      'writable': false,
      'unit': 'celsius',
    };
    final controller = HomeController(core, AccountPreferences());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: controller),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Controls fixture').first);
    for (var i = 0; i < 8; i++) {
      await tester.pump();
    }
    expect(app.currentState!.id, 'C01');
    expect(controller.capability('power'), isNull);
    expect(controller.readingLoading('main'), true);
    final target = find.byType(Climate);
    expect(target, findsOneWidget);
    expect(
      find.descendant(
        of: target,
        matching: find.byType(CircularProgressIndicator),
      ),
      findsOneWidget,
    );
    expect(find.text('Refresh State'), findsNothing);
    core.gate.complete();
    for (var i = 0; i < 8; i++) {
      await tester.pump();
    }
    expect(controller.readingLoading('main'), false);
    expect(
      find.descendant(
        of: target,
        matching: find.byType(CircularProgressIndicator),
      ),
      findsNothing,
    );
    expect(
      find.descendant(of: target, matching: find.text('—')),
      findsNWidgets(2),
    );
    expect(find.text('—°'), findsNothing);
    expect(find.textContaining('NaN'), findsNothing);
    expect(find.text('Refresh State'), findsNothing);
    expect(find.textContaining('Updated just now'), findsNothing);
    expect(find.textContaining('measurement time'), findsNothing);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets(
    'component readings load independently and failed fields settle',
    (tester) async {
      final core = await InteractionFixture.open();
      for (final capability in core.capabilities) {
        capability['actions'] = <String, dynamic>{};
      }
      final device = core.device['id'] as String;
      final first = core.hold(device, 'fridge');
      final second = core.hold(device, 'freezer');
      core.failures.add((device, 'fridge'));
      final controller = HomeController(core, AccountPreferences());
      final app = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: app, catalog: catalog, controller: controller),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Example refrigerator').first);
      for (var i = 0; i < 8; i++) {
        await tester.pump();
      }
      final fridge = find.byKey(const ValueKey('readonly-zone:fridge'));
      final freezer = find.byKey(const ValueKey('readonly-zone:freezer'));
      expect(
        find.descendant(
          of: fridge,
          matching: find.byType(CircularProgressIndicator),
        ),
        findsWidgets,
      );
      expect(
        find.descendant(
          of: freezer,
          matching: find.byType(CircularProgressIndicator),
        ),
        findsWidgets,
      );
      first.complete();
      for (var i = 0; i < 8; i++) {
        await tester.pump();
      }
      expect(
        find.descendant(
          of: fridge,
          matching: find.byType(CircularProgressIndicator),
        ),
        findsNothing,
      );
      expect(
        find.descendant(of: fridge, matching: find.text('—')),
        findsWidgets,
      );
      expect(
        find.descendant(
          of: freezer,
          matching: find.byType(CircularProgressIndicator),
        ),
        findsWidgets,
      );
      second.complete();
      for (var i = 0; i < 8; i++) {
        await tester.pump();
      }
      expect(
        find.descendant(
          of: freezer,
          matching: find.byType(CircularProgressIndicator),
        ),
        findsNothing,
      );
      expect(
        find.descendant(of: freezer, matching: find.text('-18°C')),
        findsOneWidget,
      );
      expect(
        find.descendant(of: freezer, matching: find.text('—')),
        findsWidgets,
      );
      expect(find.text('Refresh State'), findsNothing);
      expect(find.textContaining('measurement time'), findsNothing);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );
}
