import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/appliance_controls.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/skin.dart';

import 'accounts_test.dart' show AccountPreferences;
import 'controls_test.dart' show ControlsFixture;

class HeldAppliance extends ControlsFixture {
  final gate = Completer<void>();
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'read_state') {
      calls.add({'method': method, ...fields});
      if (fields['cached'] == true) {
        throw NativeFailure('CACHE_MISS', 'No fictional reading.');
      }
      await gate.future;
      throw NativeFailure('NETWORK_UNAVAILABLE', 'Fictional read failure.');
    }
    return super.call(method, fields);
  }
}

class DeniedAppliance extends ControlsFixture implements NativeSessionClient {
  @override
  Set<String> get methods => {'inspect_session_grants'};
  @override
  bool apiAtLeast(int minor) => true;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'inspect_session_grants') {
      final connection = (device['connections'] as List).singleWhere(
        (connection) => connection['id'] == fields['connection_id'],
      );
      return {
        'device_id': device['id'],
        'connection_id': connection['id'],
        'transport': connection['transport'],
        'source': 'session_policy',
        'provider_authorization': 'not_checked',
        'grants': <Json>[],
      };
    }
    return super.call(method, fields);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('specialized writes keep observed values and local uncertainty', (
    tester,
  ) async {
    final core = ControlsFixture();
    core.capabilities.removeWhere((capability) => capability['id'] != 'light');
    final controller = HomeController(core, AccountPreferences());
    await controller.initialize();
    expect(await controller.selectDevice(core.device['id']), true);
    final gate = Completer<void>();
    final actions = <String>[];
    var sends = 0;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SingleChildScrollView(
            child: ApplianceControls(
              controller: controller,
              family: 'light',
              onAction: actions.add,
              onExecute: (_, __, ___) async {
                sends++;
                await gate.future;
                controller.receipt = {
                  'device_id': core.device['id'],
                  'status': 'unknown',
                };
                return true;
              },
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final slider = find.descendant(
      of: find.byKey(const ValueKey('appliance-slider:light:brightness')),
      matching: find.byType(Slider),
    );
    await tester.drag(slider, const Offset(40, 0));
    await tester.pump();
    expect(sends, 1);
    expect(controller.observed('light.brightness'), 35);
    expect(tester.widget<Slider>(slider).value, 35);
    expect(tester.widget<Slider>(slider).onChanged, isNull);
    expect(find.byType(OperationPulse), findsOneWidget);
    gate.complete();
    await tester.pump();
    await tester.pump();
    expect(controller.receipt!['status'], 'unknown');
    expect(controller.observed('light.brightness'), 35);
    expect(
      find.textContaining('This request may have executed.'),
      findsOneWidget,
    );
    final details = find.text('View Details');
    await tester.ensureVisible(details);
    await tester.tap(details);
    await tester.pump();
    expect(actions, ['View Details']);
    expect(sends, 1);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
    controller.dispose();
  });
  for (final family in ['light', 'fan', 'media', 'sensor', 'appliance_cycle']) {
    testWidgets('$family first read uses local loading then missing values', (
      tester,
    ) async {
      final core = HeldAppliance();
      core.capabilities.removeWhere((capability) => capability['id'] != family);
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
      expect(find.byType(ApplianceControls), findsOneWidget);
      expect(controller.readingLoading('main'), true);
      expect(
        find.descendant(
          of: find.byType(ApplianceControls),
          matching: find.byType(CircularProgressIndicator),
        ),
        findsWidgets,
      );
      core.gate.complete();
      for (var i = 0; i < 8; i++) {
        await tester.pump();
      }
      expect(controller.readingLoading('main'), false);
      expect(
        find.descendant(
          of: find.byType(ApplianceControls),
          matching: find.byType(CircularProgressIndicator),
        ),
        findsNothing,
      );
      expect(find.text('—'), findsWidgets);
      expect(find.text('Unknown'), findsNothing);
      expect(find.text('Refresh State'), findsNothing);
      expect(find.textContaining('Latest read'), findsNothing);
      expect(find.textContaining('out of date'), findsNothing);
      expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    });
  }

  testWidgets('specialized controls do not enable without current grants', (
    tester,
  ) async {
    final core = DeniedAppliance();
    core.capabilities.removeWhere((capability) => capability['id'] != 'fan');
    final controller = HomeController(core, AccountPreferences());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: controller),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Controls fixture').first);
    await tester.pumpAndSettle();
    expect(find.byType(ApplianceControls), findsOneWidget);
    expect(
      find.byKey(const ValueKey('appliance-option:speed:boost')),
      findsNothing,
    );
    expect(find.byKey(const ValueKey('property:oscillation')), findsNothing);
    for (final button in tester.widgetList<ActionButton>(
      find.byType(ActionButton),
    )) {
      if (button.label == 'Set') expect(button.onPressed, isNull);
    }
    expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
    await tester.pumpWidget(const SizedBox());
  });
}
