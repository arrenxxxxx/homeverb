import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/appliance_panels.dart';
import 'package:homeverb_mobile/compartment_controls.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/localization.dart';
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

class ReferenceFridgeFixture extends ModeFixture {
  ReferenceFridgeFixture() {
    device['components'].add({
      'id': 'variable',
      'capabilities': ['climate'],
    });
    final climate = example('capability.fridge');
    climate['component_id'] = 'variable';
    climate['actions'] = <String, dynamic>{};
    for (final property in (climate['properties'] as Json).values) {
      property['writable'] = false;
    }
    capabilities.add(climate);
    targets['variable'] = 6;
    device['components'][0]['capabilities'].add(
      'vendor.midea.refrigerator.details',
    );
    capabilities.add({
      'schema_version': '1.0',
      'id': 'vendor.midea.refrigerator.details',
      'component_id': 'fridge',
      'revision': 1,
      'properties': {
        'deep_cold_mode': {
          'type': 'boolean',
          'readable': true,
          'writable': false,
        },
        'e_cleaning_percentage': {
          'type': 'number',
          'unit': 'percent',
          'readable': true,
          'writable': false,
        },
        'storage_door_state': {
          'type': 'string',
          'readable': true,
          'writable': false,
        },
      },
      'actions': <String, dynamic>{},
      'events': <String, dynamic>{},
    });
  }
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    final result = await super.call(method, fields);
    if (method == 'read_state' && fields['component'] == 'fridge') {
      result['observations'].addAll({
        'vendor.midea.refrigerator.details.deep_cold_mode': {
          'value': true,
          'stale': true,
        },
        'vendor.midea.refrigerator.details.e_cleaning_percentage': {
          'value': 35,
          'stale': true,
        },
        'vendor.midea.refrigerator.details.storage_door_state': {
          'value': 'closed',
          'stale': true,
        },
      });
    }
    return result;
  }
}

void main() {
  testWidgets(
    'reference refrigerator has a separate read-only third zone and detail readings',
    (tester) async {
      final core = ReferenceFridgeFixture();
      final c = HomeController(
        core,
        LocalPreferences(File('/unused/reference-fridge.json')),
      );
      await c.initialize();
      await c.selectDevice(core.device['id']);
      await tester.pumpWidget(
        TranslationProvider(
          child: MaterialApp(
            home: Scaffold(
              body: SingleChildScrollView(
                child: CompartmentControls(
                  controller: c,
                  onAction: (_) {},
                  onTarget: (_, __) {
                    throw StateError('No reference target write.');
                  },
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Variable zone'), findsOneWidget);
      expect(find.text('6°C'), findsOneWidget);
      expect(find.text('Deep cooling'), findsOneWidget);
      expect(find.text('35%'), findsOneWidget);
      expect(find.text('Fridge door state'), findsOneWidget);
      expect(
        find.byKey(const ValueKey('edit-component:variable')),
        findsNothing,
      );
      expect(
        find.byKey(const ValueKey('component-stepper:variable')),
        findsNothing,
      );
      LocaleSettings.setLocale(AppLocale.zhCn);
      await tester.pumpAndSettle();
      expect(find.text('\u53d8\u6e29\u533a'), findsOneWidget);
      expect(find.text('\u6df1\u51b7'), findsOneWidget);
      expect(
        core.calls.where((v) => ['plan', 'execute'].contains(v['method'])),
        isEmpty,
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
      LocaleSettings.setLocale(AppLocale.enUs);
    },
  );
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
      expect(find.text('Stale'), findsNWidgets(3));
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
