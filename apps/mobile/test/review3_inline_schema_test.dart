import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'component_test.dart' show example;
import 'review_compartment_mode_test.dart' show ModeCompartment;

class InlineSchemaFixture extends ModeCompartment {
  InlineSchemaFixture(Json schema) {
    final climate = capabilities.first;
    climate['properties']['humidity'] = {
      'type': 'number',
      'readable': true,
      'writable': true,
    };
    climate['actions']['set']['parameters']['properties']['humidity'] = schema;
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'execute' &&
        (fields['command']['arguments'] as Json).containsKey('humidity')) {
      calls.add({'method': method, ...fields});
      return {
        ...example('command-result.accepted'),
        'device_id': device['id'],
        'request_id': fields['command']['request_id'],
      };
    }
    return super.call(method, fields);
  }
}

void main() {
  Future<InlineSchemaFixture> open(WidgetTester tester, Json schema) async {
    final core = InlineSchemaFixture(schema);
    final controller = HomeController(core, LocalPreferences.memory());
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(catalog: catalog, controller: controller),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Example refrigerator').first);
    await tester.pumpAndSettle();
    return core;
  }

  Finder option(Object value) =>
      find.byKey(ValueKey('component-option:fridge:humidity:$value'));

  testWidgets('exclusive minimum rejects its boundary and sends legal choice', (
    tester,
  ) async {
    final core = await open(tester, {
      'type': 'number',
      'enum': [40, 50, 60],
      'exclusiveMinimum': 50,
    });
    expect(tester.widget<ChoiceChip>(option(40)).onSelected, isNull);
    expect(tester.widget<ChoiceChip>(option(50)).onSelected, isNull);
    expect(tester.widget<ChoiceChip>(option(60)).onSelected, isNotNull);
    expect(core.calls.where((c) => c['method'] == 'execute'), isEmpty);
    await tester.ensureVisible(option(60));
    await tester.tap(option(60));
    await tester.pumpAndSettle();
    final plans = core.calls.where((c) => c['method'] == 'plan').toList();
    final writes = core.calls.where((c) => c['method'] == 'execute').toList();
    expect(plans, hasLength(1));
    expect(writes, hasLength(1));
    expect(writes.single['command']['arguments'], {'humidity': 60});
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('exclusive maximum and const intersect enum', (tester) async {
    await open(tester, {
      'type': 'number',
      'enum': [40, 50, 60],
      'exclusiveMaximum': 60,
      'const': 50,
    });
    expect(tester.widget<ChoiceChip>(option(40)).onSelected, isNull);
    expect(tester.widget<ChoiceChip>(option(50)).onSelected, isNotNull);
    expect(tester.widget<ChoiceChip>(option(60)).onSelected, isNull);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('unknown enum constraints cannot enable an inline command', (
    tester,
  ) async {
    final core = await open(tester, {
      'type': 'number',
      'enum': [40, 50, 60],
      'not': {'const': 50},
    });
    for (final value in [40, 50, 60]) {
      expect(tester.widget<ChoiceChip>(option(value)).onSelected, isNull);
    }
    expect(core.calls.where((c) => c['method'] == 'execute'), isEmpty);
    await tester.pumpWidget(const SizedBox());
  });
  for (final scenario in <(String, Json, Set<num>)>[
    (
      'exclusive maximum',
      {
        'type': 'number',
        'enum': [40, 50, 60],
        'exclusiveMaximum': 60,
      },
      {40, 50},
    ),
    (
      'constant',
      {
        'type': 'number',
        'enum': [40, 50, 60],
        'const': 50,
      },
      {50},
    ),
    (
      'unknown format',
      {
        'type': 'number',
        'enum': [40, 50, 60],
        'format': 'unknown',
      },
      {},
    ),
    (
      'invalid exclusive bound',
      {
        'type': 'number',
        'enum': [40, 50, 60],
        'exclusiveMinimum': true,
      },
      {},
    ),
    (
      'invalid step',
      {
        'type': 'number',
        'enum': [40, 50, 60],
        'multipleOf': 0,
      },
      {},
    ),
    (
      'type mismatch',
      {
        'type': 'string',
        'enum': [40, 50, 60],
      },
      {},
    ),
    (
      'metadata',
      {
        'type': 'number',
        'enum': [40, 50, 60],
        'description': 'Fictional values',
      },
      {40, 50, 60},
    ),
    (
      'exact decimal step',
      {
        'type': 'number',
        'enum': [0.3, 0.300000001, 0.4],
        'multipleOf': 0.1,
      },
      {0.3, 0.4},
    ),
  ]) {
    testWidgets('inline schema handles ${scenario.$1}', (tester) async {
      await open(tester, scenario.$2);
      for (final value in scenario.$2['enum'] as List) {
        expect(
          tester.widget<ChoiceChip>(option(value)).onSelected != null,
          scenario.$3.contains(value),
          reason: '${scenario.$1}: $value',
        );
      }
      await tester.pumpWidget(const SizedBox());
    });
  }

  testWidgets('unsupported object constraints never enable an inline option', (
    tester,
  ) async {
    final core = InlineSchemaFixture({
      'type': 'number',
      'enum': [40, 50, 60],
    });
    core.capabilities.first['actions']['set']['parameters']['not'] = {
      'required': ['humidity'],
    };
    final controller = HomeController(core, LocalPreferences.memory());
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(catalog: catalog, controller: controller),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Example refrigerator').first);
    await tester.pumpAndSettle();
    expect(option(50), findsNothing);
    expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
    await tester.pumpWidget(const SizedBox());
  });
}
