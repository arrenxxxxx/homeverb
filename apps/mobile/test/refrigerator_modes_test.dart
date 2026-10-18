import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/appliance_panels.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'component_test.dart' show CompartmentFixture, example;

class ModeFixture extends CompartmentFixture {
  ModeFixture() {
    capabilities.add(example('capability.meiju-refrigerator'));
    device['components'][0]['capabilities'].add('vendor.midea.refrigerator');
  }
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    final value = await super.call(method, fields);
    if (method == 'read_state' && fields['component'] == 'fridge') {
      value['observations'].addAll({
        'vendor.midea.refrigerator.storage_mode': {
          'value': true,
          'stale': true,
        },
        'vendor.midea.refrigerator.freezing_mode': {
          'value': false,
          'stale': true,
        },
        'vendor.midea.refrigerator.left_flexzone_temperature': {
          'value': -2,
          'stale': true,
        },
      });
    }
    return value;
  }
}

void main() {
  testWidgets(
    'reported refrigerator modes never invent missing features or writes',
    (tester) async {
      final core = ModeFixture();
      final c = HomeController(
        core,
        LocalPreferences(File('/unused/modes.json')),
      );
      await c.initialize();
      await c.selectDevice(core.device['id']);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: RefrigeratorModes(controller: c),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Rapid cooling'), findsOneWidget);
      expect(find.text('Rapid freezing'), findsOneWidget);
      expect(find.text('On'), findsOneWidget);
      expect(find.text('Off'), findsOneWidget);
      expect(find.text('-2°C'), findsOneWidget);
      expect(find.text('Holiday mode'), findsNothing);
      expect(
        core.calls.where(
          (call) => ['execute', 'plan'].contains(call['method']),
        ),
        isEmpty,
      );
      expect(tester.takeException(), isNull);
      c.dispose();
    },
  );
}
