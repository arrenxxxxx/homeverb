import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/device_detail_controls.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';

import 'device_expanded_test.dart' show GenericComponents;

/// Models independent Core responses rather than sharing mutable fixture maps
/// with the controller's saved description.
class ChangingDescription extends GenericComponents {
  Completer<void>? descriptionGate;
  Completer<void>? liveReadGate;
  bool cacheCleared = false;

  @override
  Set<String> get methods => {...super.methods, 'clear_cached_state'};

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'describe_device') {
      await descriptionGate?.future;
      final value = await super.call(method, fields);
      return jsonDecode(jsonEncode(value));
    }
    if (method == 'clear_cached_state') {
      calls.add({'method': method});
      cacheCleared = true;
      return {'removed_snapshots': 3};
    }
    if (method == 'recent_executions') return <Json>[];
    if (method == 'read_state') {
      if (fields['cached'] == true && cacheCleared) {
        calls.add({'method': method, ...fields});
        throw NativeFailure('STATE_UNAVAILABLE', 'No fictional cache.');
      }
      if (fields['cached'] == false) await liveReadGate?.future;
    }
    return super.call(method, fields);
  }
}

Widget deviceView(HomeController controller) => MaterialApp(
  home: Scaffold(
    body: SingleChildScrollView(
      child: ListenableBuilder(
        listenable: controller,
        builder: (context, _) =>
            DeviceDetailControls(controller: controller, onAction: (_) {}),
      ),
    ),
  ),
);

Future<void> flush(WidgetTester tester, [int count = 8]) async {
  for (var i = 0; i < count; i++) {
    await tester.pump();
  }
}

void main() {
  testWidgets('reopening reconciles removed descriptors and authority', (
    tester,
  ) async {
    final core = ChangingDescription();
    final c = HomeController(core, LocalPreferences.memory());
    addTearDown(c.dispose);
    expect(await c.initialize(), true);
    final device = core.device['id'] as String;
    expect(await c.selectDevice(device), true);
    await tester.pumpWidget(deviceView(c));
    await tester.pumpAndSettle();
    const control = ValueKey('control:0:set_value');
    expect(find.byKey(control), findsOneWidget);
    expect(c.hasControlAction('vendor.fixture.zone', 'set_value'), true);

    // Change the Core descriptor without mutating the already saved response.
    final upper = (core.device['components'] as List).first as Json;
    (upper['capabilities'] as List).clear();
    core.capabilities.removeWhere((item) => item['component_id'] == 'upper');
    core.descriptionGate = Completer<void>();
    c.cancelPending();
    final opening = c.selectDevice(device);
    await flush(tester);
    expect(c.componentCapability('upper', 'vendor.fixture.zone'), isNotNull);
    expect(c.hasControlAction('vendor.fixture.zone', 'set_value'), false);
    expect(find.byKey(control), findsNothing);

    core.descriptionGate!.complete();
    await flush(tester);
    expect(await opening, true);
    await flush(tester);
    expect(c.componentCapability('upper', 'vendor.fixture.zone'), isNull);
    expect(find.byKey(control), findsNothing);
    expect(
      await c.execute('vendor.fixture.zone', 'set_value', {'setting': false}),
      false,
    );
    expect((c.error as NativeFailure).code, 'CAPABILITY_UNSUPPORTED');
    expect(core.calls.where((call) => call['method'] == 'plan'), isEmpty);
    expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('cleared readings do not return while a new live read is held', (
    tester,
  ) async {
    final core = ChangingDescription();
    final c = HomeController(core, LocalPreferences.memory());
    addTearDown(c.dispose);
    expect(await c.initialize(), true);
    final device = core.device['id'] as String;
    expect(await c.selectDevice(device), true);
    await tester.pumpWidget(deviceView(c));
    await tester.pumpAndSettle();
    expect(c.observed('vendor.fixture.zone.setting'), true);
    expect(find.text('On'), findsOneWidget);
    core.calls.clear();

    expect(await c.clearLocalData(LocalCleanup.cachedState), true);
    await flush(tester);
    expect(c.componentSnapshot('upper'), isNull);
    expect(c.snapshots, isEmpty);
    expect(find.text('On'), findsNothing);
    expect(
      core.calls.where((call) => call['method'] == 'read_state'),
      isNotEmpty,
    );
    expect(
      core.calls
          .where((call) => call['method'] == 'read_state')
          .every((call) => call['cached'] == true),
      true,
    );

    core.liveReadGate = Completer<void>();
    c.cancelPending();
    final opening = c.selectDevice(device);
    await flush(tester);
    expect(c.readingLoading('upper'), true);
    expect(c.componentSnapshot('upper'), isNull);
    expect(find.text('On'), findsNothing);
    await tester.pump(const Duration(seconds: 5));
    expect(c.componentSnapshot('upper'), isNull);
    core.liveReadGate!.complete();
    await flush(tester, 20);
    expect(await opening, true);
    await flush(tester);
    expect(c.observed('vendor.fixture.zone.setting'), true);
    expect(find.text('On'), findsOneWidget);
    expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
    await tester.pumpWidget(const SizedBox());
  });
}
