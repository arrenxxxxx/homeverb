import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/compartment_controls.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/device_detail_controls.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';

import 'component_test.dart' show CompartmentFixture;
import 'controls_test.dart' show ControlsFixture, capability, setter, example;
import 'device_detail_controls_test.dart' show DishwasherFixture, dishwasher;

class MixedComponents extends DishwasherFixture {
  MixedComponents() {
    capabilities.single['component_id'] = 'wash-zone';
    final sensor = capability('sensor', {
      'humidity': {'type': 'number', 'readable': true, 'writable': false},
    }, {});
    sensor['component_id'] = 'sensor-zone';
    capabilities.insert(0, sensor);
    device['components'] = [
      {
        'id': 'sensor-zone',
        'capabilities': ['sensor'],
      },
      {
        'id': 'wash-zone',
        'capabilities': [dishwasher],
      },
    ];
    observations['sensor.humidity'] = 48;
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'inspect_session_grants') {
      return {
        'device_id': device['id'],
        'connection_id': 'fixture-cloud',
        'transport': 'cloud',
        'source': 'session_policy',
        'provider_authorization': 'not_checked',
        'grants': [
          for (final action in (capabilities.last['actions'] as Json).keys)
            {
              'component_id': 'wash-zone',
              'capability': dishwasher,
              'action': action,
              'argument_schema': true,
            },
        ],
      };
    }
    return super.call(method, fields);
  }
}

class GenericComponents extends ControlsFixture implements NativeSessionClient {
  GenericComponents() {
    capabilities.clear();
    for (final component in ['upper', 'lower', 'denied']) {
      final definition = capability(
        'vendor.fixture.zone',
        {
          'setting': {
            'type': 'boolean',
            'readable': true,
            'writable': true,
            'control_requires_observation': true,
          },
        },
        {
          'set_value': setter(
            {
              'setting': {'type': 'boolean'},
            },
            required: ['setting'],
          ),
        },
      );
      definition['component_id'] = component;
      capabilities.add(definition);
    }
    device['alias'] = 'Generic component fixture';
    device['connections'] = [
      {'id': 'fixture-lan', 'transport': 'lan'},
    ];
    device['components'] = [
      for (final component in ['upper', 'lower', 'denied'])
        {
          'id': component,
          'capabilities': ['vendor.fixture.zone'],
        },
    ];
  }

  @override
  Set<String> get methods => {'inspect_session_grants'};
  @override
  bool apiAtLeast(int minor) => true;

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'inspect_session_grants') {
      return {
        'device_id': device['id'],
        'connection_id': 'fixture-lan',
        'transport': 'lan',
        'source': 'session_policy',
        'provider_authorization': 'not_checked',
        'grants': [
          for (final component in ['upper', 'lower'])
            {
              'component_id': component,
              'capability': 'vendor.fixture.zone',
              'action': 'set_value',
              'argument_schema': true,
            },
        ],
      };
    }
    if (method == 'read_state') {
      calls.add({'method': method, ...fields});
      return {
        ...example('state'),
        'device_id': device['id'],
        'component_id': fields['component'],
        'observations': {
          if (fields['component'] != 'lower')
            'vendor.fixture.zone.setting': {
              'value': fields['component'] == 'upper',
            },
        },
      };
    }
    return super.call(method, fields);
  }
}

void main() {
  testWidgets('mixed components expose dishwasher choices and other readings', (
    tester,
  ) async {
    final core = MixedComponents();
    final controller = HomeController(
      core,
      LocalPreferences(File('/unused/mixed-components.json')),
    );
    addTearDown(controller.dispose);
    expect(await controller.initialize(), true);
    expect(await controller.selectDevice(core.device['id']), true);
    expect(controller.component, 'sensor-zone');
    final actions = <String>[];
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SingleChildScrollView(
            child: DeviceDetailControls(
              controller: controller,
              onAction: actions.add,
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.byKey(const ValueKey('component-group:sensor-zone')),
      findsOneWidget,
    );
    expect(
      find.byKey(const ValueKey('component-group:wash-zone')),
      findsOneWidget,
    );
    expect(find.text('48'), findsOneWidget);
    expect(find.byType(ChoiceChip), findsNothing);
    expect(find.byType(ExpansionTile), findsNothing);
    expect(
      find.byKey(const ValueKey('component-dishwasher:wash-zone')),
      findsOneWidget,
    );
    for (final key in ['wash-program:eco_wash', 'wash-additional:9']) {
      final option = find.byKey(ValueKey(key));
      expect(option, findsOneWidget);
      await tester.ensureVisible(option);
      await tester.tap(option);
      await tester.pump();
    }
    final start = find.byKey(const ValueKey('control:0:start_cycle'));
    await tester.ensureVisible(start);
    await tester.tap(start);
    await tester.pump();
    expect(actions, hasLength(1));
    expect(
      Uri.decodeComponent(actions.single),
      'control-component:wash-zone:$dishwasher:start_cycle:{"program":"eco_wash","additional":9}',
    );
    expect(core.prepared, isNull);
    expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
    expect(tester.takeException(), isNull);
    expect(await controller.selectComponent('wash-zone'), true);
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SingleChildScrollView(
            child: DeviceDetailControls(
              controller: controller,
              onAction: actions.add,
            ),
          ),
        ),
      ),
    );
    await tester.pump();
    expect(
      find.byKey(const ValueKey('component-group:sensor-zone')),
      findsOneWidget,
    );
    expect(
      find.byKey(const ValueKey('component-dishwasher:wash-zone')),
      findsOneWidget,
    );
    expect(find.text('48'), findsOneWidget);
    expect(find.byType(ExpansionTile), findsNothing);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('component progress and uncertain outcomes stay local', (
    tester,
  ) async {
    final core = CompartmentFixture();
    final controller = HomeController(
      core,
      LocalPreferences(File('/unused/component-feedback.json')),
    );
    addTearDown(controller.dispose);
    await controller.initialize();
    await controller.selectDevice(core.device['id']);
    final actions = <String>[];
    Widget view({String? pending}) => MaterialApp(
      home: Scaffold(
        body: SingleChildScrollView(
          child: CompartmentControls(
            controller: controller,
            onAction: actions.add,
            onTarget: (_, __) {},
            pendingComponent: pending,
            outcomeComponent: 'freezer',
          ),
        ),
      ),
    );
    controller.receipt = {'device_id': core.device['id'], 'status': 'unknown'};
    await tester.pumpWidget(view(pending: 'freezer'));
    await tester.pump();
    expect(
      find.byKey(const ValueKey('component-pending:freezer')),
      findsOneWidget,
    );
    expect(
      find.byKey(const ValueKey('component-pending:fridge')),
      findsNothing,
    );
    expect(find.text('4°C'), findsWidgets);
    expect(find.text('-18°C'), findsWidgets);
    expect(find.text('View Details'), findsNothing);
    await tester.pumpWidget(view());
    await tester.pump();
    expect(
      find.byKey(const ValueKey('component-pending:freezer')),
      findsNothing,
    );
    expect(
      find.textContaining('This request may have executed.'),
      findsOneWidget,
    );
    final details = find.text('View Details');
    await tester.ensureVisible(details);
    await tester.tap(details);
    await tester.pump();
    expect(actions, ['View Details']);
    expect(controller.receipt!['status'], 'unknown');
    controller.receipt = {'device_id': core.device['id'], 'status': 'accepted'};
    controller.error = NativeFailure(
      'NETWORK_UNAVAILABLE',
      'Fictional readback failure.',
    );
    await tester.pumpWidget(view());
    await tester.pump();
    expect(
      find.textContaining('completion has not been verified'),
      findsOneWidget,
    );
    expect(controller.receipt!['status'], 'accepted');
    expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('generic components expose scoped readings and actions inline', (
    tester,
  ) async {
    final core = GenericComponents();
    final controller = HomeController(
      core,
      LocalPreferences(File('/unused/generic-multi.json')),
    );
    addTearDown(controller.dispose);
    await controller.initialize();
    expect(await controller.selectDevice(core.device['id']), true);
    final actions = <String>[];
    Widget view() => MaterialApp(
      home: Scaffold(
        body: SingleChildScrollView(
          child: DeviceDetailControls(
            controller: controller,
            onAction: actions.add,
          ),
        ),
      ),
    );
    await tester.pumpWidget(view());
    await tester.pumpAndSettle();
    for (final id in ['upper', 'lower', 'denied']) {
      expect(find.byKey(ValueKey('component-group:$id')), findsOneWidget);
    }
    expect(find.byType(ChoiceChip), findsNothing);
    expect(find.byType(ExpansionTile), findsNothing);
    expect(find.text('On'), findsOneWidget);
    expect(find.text('Off'), findsOneWidget);
    expect(find.text('—'), findsWidgets);
    expect(find.byKey(const ValueKey('control:0:set_value')), findsOneWidget);
    const lowerToken = 'control-component:lower:vendor.fixture.zone:set_value';
    const deniedToken =
        'control-component:denied:vendor.fixture.zone:set_value';
    expect(find.byKey(const ValueKey(lowerToken)), findsNothing);
    expect(find.byKey(const ValueKey(deniedToken)), findsNothing);
    controller.componentSnapshot(
      'lower',
    )!['observations']['vendor.fixture.zone.setting'] = {
      'value': false,
    };
    await tester.pumpWidget(view());
    await tester.pump();
    final lower = find.byKey(const ValueKey(lowerToken));
    expect(lower, findsOneWidget);
    expect(find.byKey(const ValueKey(deniedToken)), findsNothing);
    await tester.ensureVisible(lower);
    await tester.tap(lower);
    await tester.pump();
    expect(actions, [lowerToken]);
    expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
  });
  testWidgets('component actions and device information are inline', (
    tester,
  ) async {
    final core = CompartmentFixture();
    core.device['components'][0]['capabilities'].add('vendor.example.extra');
    core.capabilities.add(
      capability(
        'vendor.example.extra',
        {
          'indicator': {'type': 'boolean', 'readable': true, 'writable': true},
        },
        {
          'set_indicator': setter(
            {
              'indicator': {'type': 'boolean'},
            },
            required: ['indicator'],
          ),
        },
      ),
    );
    final controller = HomeController(
      core,
      LocalPreferences(File('/unused/expanded-device.json')),
    );
    addTearDown(controller.dispose);
    await controller.initialize();
    await controller.selectDevice(core.device['id']);
    final actions = <String>[];
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SingleChildScrollView(
            child: CompartmentControls(
              controller: controller,
              onAction: actions.add,
              onTarget: (_, __) => fail('No direct write is expected.'),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('More controls'), findsNothing);
    expect(find.byType(ExpansionTile), findsNothing);
    expect(find.text('Device information'), findsOneWidget);
    expect(find.text('Model'), findsOneWidget);
    final button = find.byKey(
      const ValueKey(
        'component-control:fridge:vendor.example.extra:set_indicator',
      ),
    );
    expect(button, findsOneWidget);
    core
            .capabilities
            .last['properties']['indicator']['control_requires_observation'] =
        true;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SingleChildScrollView(
            child: CompartmentControls(
              controller: controller,
              onAction: actions.add,
              onTarget: (_, __) => fail('No direct write is expected.'),
            ),
          ),
        ),
      ),
    );
    await tester.pump();
    expect(button, findsNothing);
    controller.componentSnapshot(
      'fridge',
    )!['observations']['vendor.example.extra.indicator'] = {
      'value': false,
    };
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SingleChildScrollView(
            child: CompartmentControls(
              controller: controller,
              onAction: actions.add,
              onTarget: (_, __) => fail('No direct write is expected.'),
            ),
          ),
        ),
      ),
    );
    await tester.pump();
    expect(button, findsOneWidget);
    await tester.ensureVisible(button);
    await tester.tap(button);
    await tester.pump();
    expect(actions, [
      'control-component:fridge:vendor.example.extra:set_indicator',
    ]);
    expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
    expect(tester.takeException(), isNull);
  });
}
