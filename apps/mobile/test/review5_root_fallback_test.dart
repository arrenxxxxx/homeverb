import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'review_compartment_mode_test.dart' show ModeCompartment;

class RootConstraintFixture extends ModeCompartment {
  RootConstraintFixture(
    String keyword,
    dynamic value, {
    bool invalidMode = false,
  }) {
    final parameters = capabilities.first['actions']['set']['parameters'];
    parameters[keyword] = value;
    if (invalidMode) {
      parameters['properties']['mode'] = {
        'type': 'string',
        'enum': ['normal', 'eco', 'dry'],
        'pattern': r'^(normal|eco)$',
      };
    }
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'plan' && fields['command']['arguments']['mode'] == 'dry') {
      calls.add({'method': method, ...fields});
      throw NativeFailure('INVALID_ARGUMENT', 'Fictional schema rejection.');
    }
    return super.call(method, fields);
  }
}

void main() {
  for (final scenario in <(String, dynamic)>[
    ('additionalProperties', <String, dynamic>{}),
    ('minProperties', 1.0),
  ]) {
    for (final mode in ['eco', 'dry']) {
      testWidgets('${scenario.$1} retains an editor for $mode', (tester) async {
        final core = RootConstraintFixture(
          scenario.$1,
          scenario.$2,
          invalidMode: mode == 'dry',
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
        expect(
          c.canControlValue(
            'climate',
            'set',
            'mode',
            'eco',
            componentId: 'fridge',
          ),
          true,
        );
        final inline = find.byKey(
          ValueKey('component-option:fridge:mode:$mode'),
        );
        expect(inline, findsOneWidget);
        expect(tester.widget<ChoiceChip>(inline).onSelected, isNull);
        final form = find.byKey(
          const ValueKey('component-control:fridge:climate:set'),
        );
        expect(
          form,
          findsOneWidget,
          reason: 'An unsupported root schema must retain the full editor.',
        );
        await tester.ensureVisible(form);
        await tester.tap(form);
        await tester.pumpAndSettle();
        expect(app.currentState!.id, 'C10');
        final choice = find.byKey(ValueKey('parameter:mode:$mode'));
        await tester.ensureVisible(choice);
        await tester.tap(choice);
        final apply = find.text('Apply');
        await tester.ensureVisible(apply);
        await tester.tap(apply);
        await tester.pumpAndSettle();
        final plans = core.calls
            .where((call) => call['method'] == 'plan')
            .toList();
        final writes = core.calls
            .where((call) => call['method'] == 'execute')
            .toList();
        expect(plans, hasLength(1));
        expect(plans.single['command']['arguments'], {'mode': mode});
        if (mode == 'eco') {
          expect(writes, hasLength(1));
          expect(writes.single['command']['arguments'], {'mode': 'eco'});
        } else {
          expect(writes, isEmpty);
          expect(c.error, isA<NativeFailure>());
        }
        await tester.pumpWidget(const SizedBox());
      });
    }
  }
}
