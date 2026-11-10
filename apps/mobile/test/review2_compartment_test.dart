import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'review_compartment_mode_test.dart' show ModeCompartment;

class CombinedGrant extends ModeCompartment implements NativeSessionClient {
  CombinedGrant({
    this.arguments = const {'target_temperature': 4.5, 'mode': 'eco'},
  });
  final Json arguments;
  @override
  Set<String> get methods => {'inspect_session_grants'};
  @override
  bool apiAtLeast(int minor) => true;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'inspect_session_grants') {
      final connection = (device['connections'] as List).firstWhere(
        (c) => c['id'] == fields['connection_id'],
      );
      return {
        'device_id': fields['device_id'],
        'connection_id': fields['connection_id'],
        'transport': connection['transport'],
        'source': 'session_policy',
        'provider_authorization': 'not_checked',
        'grants': [
          {
            'component_id': 'fridge',
            'capability': 'climate',
            'action': 'set',
            'argument_schema': {'const': arguments},
          },
        ],
      };
    }
    return super.call(method, fields);
  }
}

void main() {
  testWidgets('review: retain the form for a combined target/mode grant', (
    tester,
  ) async {
    final core = CombinedGrant();
    final c = HomeController(core, LocalPreferences.memory());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: c),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Example refrigerator').first);
    await tester.pumpAndSettle();
    expect(c.hasControlAction('climate', 'set', componentId: 'fridge'), true);
    expect(
      c.canControlValue(
        'climate',
        'set',
        'target_temperature',
        4.5,
        componentId: 'fridge',
      ),
      false,
    );
    expect(
      c.canControlValue('climate', 'set', 'mode', 'eco', componentId: 'fridge'),
      false,
    );
    expect(
      find.byKey(const ValueKey('component-control:fridge:climate:set')),
      findsOneWidget,
      reason:
          'The exact combined grant needs a form; singleton editors cannot express it.',
    );
    final form = find.byKey(
      const ValueKey('component-control:fridge:climate:set'),
    );
    await tester.ensureVisible(form);
    await tester.tap(form);
    await tester.pumpAndSettle();
    expect(app.currentState!.id, 'C10');
    expect(find.textContaining('{mode}'), findsNothing);
    expect(find.textContaining('Mode · Optional'), findsOneWidget);
    await tester.enterText(find.byType(TextField).first, '4.5');
    final eco = find.byKey(const ValueKey('parameter:mode:eco'));
    await tester.ensureVisible(eco);
    await tester.tap(eco);
    final apply = find.text('Apply');
    await tester.ensureVisible(apply);
    await tester.tap(apply);
    await tester.pumpAndSettle();
    final writes = core.calls
        .where((call) => call['method'] == 'execute')
        .toList();
    expect(writes, hasLength(1));
    expect(writes.single['command']['arguments'], {
      'target_temperature': 4.5,
      'mode': 'eco',
    });
    await tester.pumpWidget(const SizedBox());
  });
  testWidgets('mode-only grant keeps the authorized independent option', (
    tester,
  ) async {
    final core = CombinedGrant(arguments: {'mode': 'eco'});
    final c = HomeController(core, LocalPreferences.memory());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: c),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Example refrigerator').first);
    await tester.pumpAndSettle();
    final eco = find.byKey(const ValueKey('component-option:fridge:mode:eco'));
    expect(tester.widget<ChoiceChip>(eco).onSelected, isNotNull);
    expect(
      c.canControlValue(
        'climate',
        'set',
        'target_temperature',
        4.5,
        componentId: 'fridge',
      ),
      false,
    );
    expect(
      find.byKey(const ValueKey('component-control:fridge:climate:set')),
      findsNothing,
    );
    await tester.ensureVisible(eco);
    await tester.tap(eco);
    await tester.pumpAndSettle();
    final writes = core.calls
        .where((call) => call['method'] == 'execute')
        .toList();
    expect(writes, hasLength(1));
    expect(writes.single['command']['arguments'], {'mode': 'eco'});
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('review: inline enum respects parameter bounds', (tester) async {
    final core = ModeCompartment();
    final climate = core.capabilities.first;
    climate['properties']['humidity'] = {
      'type': 'number',
      'enum': [40, 50, 60],
      'readable': true,
      'writable': true,
    };
    climate['actions']['set']['parameters']['properties']['humidity'] = {
      'type': 'integer',
      'enum': [40, 50, 60, 65, 70, 55.5, 'bad'],
      'minimum': 50,
      'maximum': 65,
      'multipleOf': 10,
    };
    expect(
      validateValue(50, {
        'type': 'string',
        'enum': [50],
      }),
      isNotNull,
    );
    expect(
      validateValue(
        40,
        climate['actions']['set']['parameters']['properties']['humidity'],
      ),
      isNotNull,
    );
    final c = HomeController(core, LocalPreferences.memory());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: c),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Example refrigerator').first);
    await tester.pumpAndSettle();
    final option = find.byKey(
      const ValueKey('component-option:fridge:humidity:40'),
    );
    expect(option, findsOneWidget);
    expect(
      tester.widget<ChoiceChip>(option).onSelected,
      isNull,
      reason: 'An enum member excluded by minimum must not be enabled.',
    );
    for (final value in [65, 70, 55.5, 'bad']) {
      final chip = find.byKey(
        ValueKey('component-option:fridge:humidity:$value'),
      );
      expect(tester.widget<ChoiceChip>(chip).onSelected, isNull);
    }
    for (final value in [50, 60]) {
      final chip = find.byKey(
        ValueKey('component-option:fridge:humidity:$value'),
      );
      expect(tester.widget<ChoiceChip>(chip).onSelected, isNotNull);
    }
    await tester.pumpWidget(const SizedBox());
  });
  testWidgets('inline enum requires its observed prerequisite', (tester) async {
    final core = ModeCompartment();
    final climate = core.capabilities.first;
    climate['properties']['humidity'] = {
      'type': 'integer',
      'readable': true,
      'writable': true,
      'control_requires_observation': true,
    };
    climate['actions']['set']['parameters']['properties']['humidity'] = {
      'type': 'integer',
      'enum': [50, 60],
    };
    final c = HomeController(core, LocalPreferences.memory());
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(NativeApp(catalog: catalog, controller: c));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Example refrigerator').first);
    await tester.pumpAndSettle();
    expect(
      find.byKey(const ValueKey('component-option:fridge:humidity:50')),
      findsNothing,
    );
    expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
    await tester.pumpWidget(const SizedBox());
  });
}
