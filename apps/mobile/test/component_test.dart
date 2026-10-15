import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

Json example(String name) =>
    jsonDecode(File('../../examples/$name.json').readAsStringSync()) as Json;

// Isolated contract fixture: never registered as a native adapter or app device.
class CompartmentFixture implements CoreClient {
  final device = example('device.refrigerator');
  final capabilities = [
    example('capability.fridge'),
    example('capability.freezer'),
  ];
  final calls = <Json>[];
  final targets = <String, num>{'fridge': 4, 'freezer': -18};
  final receipts = <Json>[];
  bool cacheMiss = false;
  Completer<void>? readGate;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    calls.add({'method': method, ...fields});
    switch (method) {
      case 'list_rooms':
        return <Json>[];
      case 'list_device_cards':
        return [
          {
            'device': device,
            'preferences': {'alias': null, 'room_id': null, 'favorite': false},
          },
        ];
      case 'describe_device':
        return {'device': device, 'capabilities': capabilities};
      case 'read_state':
        await readGate?.future;
        if (cacheMiss && fields['cached'] == true) {
          throw NativeFailure('STATE_UNAVAILABLE', 'No fixture cache.');
        }
        return {
          ...example('state'),
          'device_id': device['id'],
          'component_id': fields['component'],
          'from_cache': fields['cached'],
          'observations': {
            'climate.target_temperature': {
              ...example('state')['observations']['climate.target_temperature']
                  as Json,
              'value': targets[fields['component']],
            },
          },
        };
      case 'plan':
        return fields['command'];
      case 'execute':
        final command = fields['command'] as Json;
        expect(calls[calls.length - 2]['method'], 'plan');
        expect(calls[calls.length - 2]['command'], command);
        targets[command['component_id']] =
            command['arguments']['target_temperature'];
        final receipt = {
          ...example('command-result.verified'),
          'device_id': device['id'],
          'request_id': command['request_id'],
          'data': command['arguments'],
        };
        receipts.add(receipt);
        return receipt;
      case 'device_executions':
      case 'recent_executions':
        return receipts;
      default:
        throw StateError('Unexpected fixture method: $method');
    }
  }

  @override
  void cancelAll() {}
  @override
  void foreground(bool value) {}
  @override
  Future<void> close() async {}
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  test(
    'component references, scope and cached observations remain independent',
    () async {
      final core = CompartmentFixture();
      // A shared definition applies only to components that reference it.
      core.capabilities.add({
        'id': 'power',
        'actions': {},
        'properties': {},
        'events': {},
      });
      core.device['components'][0]['capabilities'].add('power');
      final controller = HomeController(
        core,
        LocalPreferences(File('/unused/preferences.json')),
      );
      await controller.initialize();
      await controller.selectDevice(core.device['id']);
      expect(controller.capability('power'), isNotNull);
      expect(controller.observed('climate.target_temperature'), 4);
      expect(await controller.selectComponent('freezer'), true);
      expect(controller.capability('power'), isNull);
      expect(
        controller.capability(
          'climate',
        )!['properties']['target_temperature']['maximum'],
        -16,
      );
      expect(controller.observed('climate.target_temperature'), -18);
      expect(
        core.calls
            .where((c) => c['method'] == 'read_state')
            .every((c) => c['cached'] == true),
        true,
      );
      expect(await controller.execute('power', 'set', {'state': true}), false);
      expect(core.calls.where((c) => c['method'] == 'plan'), isEmpty);
      await controller.execute('climate', 'set', {'target_temperature': -19});
      expect(controller.observed('climate.target_temperature'), -18);
      await controller.refresh();
      expect(controller.observed('climate.target_temperature'), -19);
      expect(
        controller.snapshots[core
            .device['id']]!['observations']['climate.target_temperature']['value'],
        4,
      );
      expect(await controller.selectComponent('missing'), false);
      expect(controller.component, 'freezer');
      expect(controller.observed('climate.target_temperature'), -19);
      core.cacheMiss = true;
      await controller.selectComponent('fridge');
      expect(controller.observed('climate.target_temperature'), 4);
      await controller.selectComponent('freezer');
      expect(controller.observed('climate.target_temperature'), -19);
    },
  );

  testWidgets('click compartment-specific bounds, steps and read-only controls', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(780, 1688);
    tester.view.devicePixelRatio = 2;
    WidgetController.hitTestWarningShouldBeFatal = true;
    addTearDown(() => WidgetController.hitTestWarningShouldBeFatal = false);
    final core = CompartmentFixture();
    final controller = HomeController(
      core,
      LocalPreferences(File('/unused/preferences.json')),
    );
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    Future<void> tap(String text) async {
      final target = find.text(text).last;
      await tester.ensureVisible(target);
      await tester.tap(target);
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
    }

    try {
      await tester.pumpWidget(
        NativeApp(key: app, catalog: catalog, controller: controller),
      );
      await tester.pumpAndSettle();
      await tap('Example refrigerator');
      expect(app.currentState!.id, 'C08');
      // Exercise the single-component detail view independently of the overview.
      await app.currentState!.go('C01');
      await tester.pumpAndSettle();
      core.readGate = Completer<void>();
      await tester.tap(find.text('freezer'));
      await tester.pump();
      expect(controller.busy, true);
      await tester.tap(find.text('fridge'));
      await tester.pump();
      final freezerSemantics = tester
          .widgetList<Semantics>(
            find.ancestor(
              of: find.text('freezer'),
              matching: find.byType(Semantics),
            ),
          )
          .where((widget) => widget.properties.selected != null)
          .single;
      expect(freezerSemantics.properties.selected, true);
      core.readGate!.complete();
      core.readGate = null;
      await tester.pumpAndSettle();
      expect(controller.component, 'freezer');
      expect(find.text('-18°'), findsOneWidget);
      await tap('-18°');
      final input = find.byKey(const ValueKey('field:Target temperature'));
      for (final invalid in {
        '4': 'Maximum: -16',
        '-18.5': 'Supported step: 1',
      }.entries) {
        await tester.enterText(input, invalid.key);
        await tap('Save target');
        expect(find.text(invalid.value), findsOneWidget);
        expect(core.receipts, isEmpty);
      }
      await tester.enterText(input, '-19');
      await tap('Save target');
      expect(app.currentState!.id, 'S06');
      expect(
        core.calls.lastWhere(
          (c) => c['method'] == 'execute',
        )['command']['component_id'],
        'freezer',
      );
      expect(core.targets['fridge'], 4);
      expect(controller.observed('climate.target_temperature'), -18);
      await tap('Refresh State');
      expect(controller.observed('climate.target_temperature'), -19);
      expect(core.receipts, hasLength(1));
      await tester.tap(find.byTooltip('Back'));
      await tester.pumpAndSettle();
      await tap('Cancel');
      await tap('fridge');
      await tap('4°');
      await tester.enterText(
        find.byKey(const ValueKey('field:Target temperature')),
        '4.5',
      );
      await tap('Save target');
      expect(
        core.calls.lastWhere(
          (c) => c['method'] == 'execute',
        )['command']['component_id'],
        'fridge',
      );
      expect(core.targets['freezer'], -19);
      core.capabilities.first['actions'] = <String, dynamic>{};
      await tester.tap(find.byTooltip('Back'));
      await tester.pumpAndSettle();
      await tap('Cancel');
      await tap('4°');
      expect(app.currentState!.id, 'C01');
      expect(find.text('Temperature is read-only here.'), findsOneWidget);
      await tap('+');
      expect(core.receipts, hasLength(2));
      expect(app.currentState!.id, 'C01');
    } finally {
      await tester.pumpWidget(const SizedBox());
      tester.view.resetPhysicalSize();
      tester.view.resetDevicePixelRatio();
    }
  });
}
