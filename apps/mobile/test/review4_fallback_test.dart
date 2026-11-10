import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'review_compartment_mode_test.dart' show ModeCompartment;

class PatternFixture extends ModeCompartment {
  PatternFixture({bool includeInvalid = false}) {
    capabilities.first['actions']['set']['parameters']['properties']['mode'] = {
      'type': 'string',
      'enum': ['normal', 'eco', if (includeInvalid) 'dry'],
      'pattern': r'^(normal|eco)$',
    };
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'plan' && fields['command']['arguments']['mode'] == 'dry') {
      calls.add({'method': method, ...fields});
      throw NativeFailure('INVALID_ARGUMENT', 'Fictional pattern rejection.');
    }
    return super.call(method, fields);
  }
}

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

  for (final mode in ['eco', 'dry']) {
    testWidgets('patterned mode $mode is handled through the capability form', (
      tester,
    ) async {
      final core = PatternFixture(includeInvalid: mode == 'dry');
      final c = HomeController(core, LocalPreferences.memory());
      final app = GlobalKey<NativeAppState>();
      await open(tester, c, app: app);
      final inline = find.byKey(ValueKey('component-option:fridge:mode:$mode'));
      expect(tester.widget<ChoiceChip>(inline).onSelected, isNull);
      final form = find.byKey(
        const ValueKey('component-control:fridge:climate:set'),
      );
      expect(form, findsOneWidget);
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
      final writes = core.calls
          .where((call) => call['method'] == 'execute')
          .toList();
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
