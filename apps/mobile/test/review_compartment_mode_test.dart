import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'component_test.dart' show CompartmentFixture, example;

class ModeCompartment extends CompartmentFixture {
  ModeCompartment() {
    final climate = capabilities.first;
    climate['properties']['mode'] = {
      'type': 'string',
      'enum': ['normal', 'eco'],
      'readable': true,
      'writable': true,
    };
    climate['actions']['set']['parameters']['properties']['mode'] = {
      'type': 'string',
      'enum': ['normal', 'eco'],
    };
    climate['actions']['set']['parameters']['required'] = <String>[];
    climate['actions']['set']['parameters']['minProperties'] = 1;
  }
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'execute' &&
        (fields['command']['arguments'] as Json).containsKey('mode')) {
      calls.add({'method': method, ...fields});
      return {
        ...example('command-result.accepted'),
        'device_id': device['id'],
        'request_id': fields['command']['request_id'],
      };
    }
    final result = await super.call(method, fields);
    if (method == 'read_state' && fields['component'] == 'fridge') {
      return {
        ...result as Json,
        'observations': {
          ...result['observations'] as Json,
          'climate.mode': {'value': 'normal'},
        },
      };
    }
    return result;
  }
}

void main() {
  testWidgets('zone mode remains visible and independently selectable', (
    tester,
  ) async {
    final core = ModeCompartment();
    final c = HomeController(core, LocalPreferences.memory());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: c),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Example refrigerator').first);
    await tester.pumpAndSettle();
    expect(app.currentState!.id, 'C08');
    expect(
      find.byKey(const ValueKey('component-stepper:fridge')),
      findsOneWidget,
    );
    final normal = find.byKey(
      const ValueKey('component-option:fridge:mode:normal'),
    );
    final eco = find.byKey(const ValueKey('component-option:fridge:mode:eco'));
    expect(normal, findsOneWidget);
    expect(eco, findsOneWidget);
    expect(tester.widget<ChoiceChip>(normal).selected, true);
    await tester.ensureVisible(eco);
    await tester.tap(eco);
    await tester.pumpAndSettle();
    final writes = core.calls
        .where((call) => call['method'] == 'execute')
        .toList();
    expect(writes, hasLength(1));
    expect(writes.single['command']['arguments'], {'mode': 'eco'});
    expect(
      c.componentSnapshot('fridge')?['observations']?['climate.mode']?['value'],
      'normal',
    );
    await tester.pumpWidget(const SizedBox());
  });
}
