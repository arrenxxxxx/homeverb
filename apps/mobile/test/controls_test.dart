import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/controls.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

Json example(String name) =>
    jsonDecode(File('../../examples/$name.json').readAsStringSync()) as Json;
Json setter(
  Json properties, {
  List<String> required = const [],
  String risk = 'low',
  String idempotency = 'set_value',
}) => {
  'parameters': {
    'type': 'object',
    'properties': properties,
    'required': required,
    'minProperties': properties.isEmpty ? 0 : 1,
    'additionalProperties': false,
  },
  'transports': ['lan'],
  'risk': risk,
  'idempotency': idempotency,
  'atomic': true,
  'verification': 'state_match',
};
Json capability(String id, Json properties, Json actions) => {
  'schema_version': '1.0',
  'id': id,
  'revision': 1,
  'properties': properties,
  'actions': actions,
  'events': {},
};

// This fixture exercises future capability descriptors, not a shipped adapter.
class ControlsFixture implements CoreClient {
  final calls = <Json>[];
  bool denyPlan = false;
  Completer<void>? executionGate;
  final capabilities = [
    capability(
      'light',
      {
        'brightness': {
          'type': 'integer',
          'readable': true,
          'writable': true,
          'unit': 'percent',
        },
        'color_temperature': {
          'type': 'integer',
          'readable': true,
          'writable': true,
          'unit': 'kelvin',
        },
      },
      {
        'set': setter({
          'brightness': {
            'type': 'integer',
            'minimum': 10,
            'maximum': 80,
            'multipleOf': 5,
          },
          'color_temperature': {
            'type': 'integer',
            'minimum': 2200,
            'maximum': 6000,
            'multipleOf': 100,
          },
        }),
      },
    ),
    capability(
      'fan',
      {
        'speed': {'type': 'string', 'readable': true, 'writable': true},
        'oscillation': {'type': 'boolean', 'readable': true, 'writable': true},
      },
      {
        'set': setter({
          'speed': {
            'type': 'string',
            'enum': ['quiet', 'boost'],
          },
          'oscillation': {'type': 'boolean'},
        }),
      },
    ),
    capability(
      'media',
      {
        'source': {'type': 'string', 'readable': true, 'writable': true},
        'muted': {'type': 'boolean', 'readable': true, 'writable': true},
        'volume': {'type': 'number', 'readable': true, 'writable': true},
      },
      {
        'select_source': setter(
          {
            'source': {
              'type': 'string',
              'enum': ['HDMI 1', 'USB'],
            },
          },
          required: ['source'],
        ),
        'mute': setter(
          {
            'muted': {'type': 'boolean'},
          },
          required: ['muted'],
        ),
        'set_volume': setter(
          {
            'volume': {
              'type': 'number',
              'minimum': -60,
              'maximum': 0,
              'multipleOf': 0.5,
            },
          },
          required: ['volume'],
        ),
        'play': setter({}),
        'pause': setter({}),
      },
    ),
    capability('sensor', {
      'humidity': {
        'type': 'number',
        'readable': true,
        'writable': false,
        'unit': 'percent',
      },
    }, {}),
    capability('appliance_cycle', {}, {
      'start': setter(
        {
          'program': {
            'enum': ['cotton', 'rinse'],
          },
        },
        required: ['program'],
        risk: 'elevated',
        idempotency: 'non_idempotent',
      ),
    }),
  ];
  late final device = {
    ...example('device'),
    'alias': 'Controls fixture',
    'components': [
      {'id': 'main', 'capabilities': capabilities.map((c) => c['id']).toList()},
    ],
  };
  final observations = <String, dynamic>{
    'light.brightness': 35,
    'fan.speed': 'quiet',
    'fan.oscillation': null,
    'media.muted': false,
    'media.volume': -20,
  };
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
        return {
          ...example('state'),
          'device_id': device['id'],
          'component_id': fields['component'],
          'observations': {
            for (final e in observations.entries)
              e.key: {
                'value': e.value,
                'source': 'lan',
                'observed_at': null,
                'received_at': '2026-10-15T00:00:00Z',
                'stale': true,
              },
          },
        };
      case 'plan':
        if (denyPlan)
          throw NativeFailure('POLICY_DENIED', 'Fixture permission denied.');
        return {};
      case 'execute':
        await executionGate?.future;
        return {
          ...example('command-result.accepted'),
          'device_id': device['id'],
          'request_id': fields['command']['request_id'],
          'data': fields['command']['arguments'],
        };
      default:
        throw StateError('Unexpected control fixture call: $method');
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
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
  });
  test('unsupported shapes, read-only setters and risk stay unavailable', () {
    final fixture = ControlsFixture();
    final cycle = fixture.capabilities.last;
    expect(
      actionUnavailable(cycle, cycle['actions']['start']),
      contains('approval'),
    );
    final readOnly = capability('sensor', {
      'value': {'writable': false},
    }, {});
    expect(
      actionUnavailable(
        readOnly,
        setter({
          'value': {'type': 'number'},
        }),
      ),
      contains('read-only'),
    );
    expect(
      actionUnavailable(
        readOnly,
        setter({
          'values': {'type': 'array'},
        }),
      ),
      contains('not supported'),
    );
    expect(propertyValue(null, {'unit': 'percent'}), 'Unknown');
  });
  testWidgets(
    'click described light, fan and media controls with validation and no optimistic observations',
    (tester) async {
      tester.view.physicalSize = const Size(780, 1688);
      tester.view.devicePixelRatio = 2;
      WidgetController.hitTestWarningShouldBeFatal = true;
      final core = ControlsFixture();
      final controller = HomeController(
        core,
        LocalPreferences(File('/unused/control-preferences.json')),
      );
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      final app = GlobalKey<NativeAppState>();
      final boundary = GlobalKey();
      final captures = <Json>[];
      final output = Directory('build/verification/controls')
        ..createSync(recursive: true);
      Future<void> capture(String state) async {
        await tester.runAsync(() async {
          final image =
              await (boundary.currentContext!.findRenderObject()
                      as RenderRepaintBoundary)
                  .toImage(pixelRatio: 2);
          final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
          final name = '${captures.length.toString().padLeft(2, '0')}.png';
          File(
            '${output.path}/$name',
          ).writeAsBytesSync(bytes!.buffer.asUint8List());
          captures.add({
            'file': name,
            'state': state,
            'width': image.width,
            'height': image.height,
          });
          image.dispose();
        });
      }

      Future<void> tap(Finder finder) async {
        await tester.ensureVisible(finder);
        await tester.tap(finder);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
      }

      Future<void> open(int index, String action) async {
        if (app.currentState!.id != 'C01') {
          await app.currentState!.go('C01');
          await tester.pumpAndSettle();
        }
        await tap(find.byKey(ValueKey('control:$index:$action')));
        expect(app.currentState!.id, 'C10');
        expect(find.text('Washing machine'), findsNothing);
        await capture('Open fixture control $index / $action');
      }

      Future<void> enter(String name, String value) async {
        final field = find.byKey(ValueKey('parameter:$name'));
        await tester.ensureVisible(field);
        await tester.enterText(field, value);
        await tester.pumpAndSettle();
      }

      Json getCommand() =>
          core.calls.lastWhere((c) => c['method'] == 'execute')['command']
              as Json;
      try {
        await tester.pumpWidget(
          RepaintBoundary(
            key: boundary,
            child: NativeApp(
              key: app,
              catalog: catalog,
              controller: controller,
            ),
          ),
        );
        await tester.pumpAndSettle();
        await tap(find.text('Controls fixture').first);
        expect(find.text('Unknown'), findsWidgets);
        expect(find.text('Trusted approval is unavailable.'), findsOneWidget);
        expect(find.byKey(const ValueKey('control:4:start')), findsNothing);
        await open(0, 'set');
        expect(
          find.byKey(const ValueKey('parameter-slider:brightness')),
          findsNothing,
        );
        await tap(find.text('Apply'));
        expect(find.textContaining('Choose at least'), findsOneWidget);
        await enter('brightness', '83');
        await tap(find.text('Apply'));
        expect(find.textContaining('Maximum: 80'), findsOneWidget);
        await enter('brightness', '42');
        await tap(find.text('Apply'));
        expect(find.textContaining('Supported step: 5'), findsOneWidget);
        expect(core.calls.where((c) => c['method'] == 'plan'), isEmpty);
        await enter('brightness', '40');
        final slider = find.byKey(
          const ValueKey('parameter-slider:brightness'),
        );
        await tester.ensureVisible(slider);
        await tester.drag(slider, const Offset(30, 0));
        await tester.pumpAndSettle();
        final chosen = num.parse(
          tester
              .widget<TextField>(
                find.byKey(const ValueKey('parameter:brightness')),
              )
              .controller!
              .text,
        );
        expect(chosen % 5, 0);
        await capture('Light brightness draft and slider');
        await tap(find.text('Apply'));
        expect(getCommand()['arguments'], {'brightness': chosen});
        expect(getCommand()['capability'], 'light');
        expect(getCommand()['component_id'], 'main');
        expect(controller.observed('light.brightness'), 35);
        expect(controller.receipt!['status'], 'accepted');
        await open(0, 'set');
        await enter('color_temperature', '2301');
        await tap(find.text('Apply'));
        expect(find.textContaining('Supported step: 100'), findsOneWidget);
        await enter('color_temperature', '2700');
        await tap(find.text('Apply'));
        expect(getCommand()['arguments'], {'color_temperature': 2700});
        await open(1, 'set');
        await tap(find.byKey(const ValueKey('parameter:speed:boost')));
        await tap(find.byKey(const ValueKey('parameter:oscillation:false')));
        await tap(find.text('Apply'));
        expect(getCommand()['arguments'], {
          'speed': 'boost',
          'oscillation': false,
        });
        expect(controller.observed('fan.oscillation'), isNull);
        await open(2, 'select_source');
        await tap(find.text('Apply'));
        expect(find.text('Choose Source.'), findsOneWidget);
        await tap(find.byKey(const ValueKey('parameter:source:USB')));
        await tap(find.text('Apply'));
        expect(getCommand()['arguments'], {'source': 'USB'});
        await open(2, 'mute');
        await tap(find.byKey(const ValueKey('parameter:muted:true')));
        await tap(find.text('Apply'));
        expect(getCommand()['arguments'], {'muted': true});
        await open(2, 'set_volume');
        await enter('volume', '-19.5');
        core.denyPlan = true;
        final before = core.calls.where((c) => c['method'] == 'execute').length;
        await tap(find.text('Apply'));
        expect(app.currentState!.id, 'C10');
        expect(
          find.textContaining('Fixture permission denied.'),
          findsOneWidget,
        );
        await capture('Permission denied with draft retained');
        expect(
          core.calls.where((c) => c['method'] == 'execute').length,
          before,
        );
        expect(
          tester
              .widget<TextField>(find.byKey(const ValueKey('parameter:volume')))
              .controller!
              .text,
          '-19.5',
        );
        core.denyPlan = false;
        await tap(find.text('Apply'));
        expect(getCommand()['arguments'], {'volume': -19.5});
        await open(2, 'play');
        await tap(find.text('Apply'));
        expect(getCommand()['arguments'], isEmpty);
        expect(getCommand()['action'], 'play');
        await open(2, 'pause');
        await tap(find.text('Apply'));
        expect(getCommand()['arguments'], isEmpty);
        expect(getCommand()['action'], 'pause');

        for (var i = 0; i < core.calls.length; i++) {
          if (core.calls[i]['method'] == 'execute') {
            expect(core.calls[i - 1]['method'], 'plan');
            expect(core.calls[i - 1]['command'], core.calls[i]['command']);
          }
        }
        await open(0, 'set');
        await enter('brightness', '45');
        core.executionGate = Completer<void>();
        final count = core.calls.where((c) => c['method'] == 'execute').length;
        final apply = find.text('Apply');
        await tester.ensureVisible(apply);
        await tester.tap(apply);
        await tester.pump();
        expect(controller.busy, true);
        expect(
          tester
              .widget<FilledButton>(find.widgetWithText(FilledButton, 'Apply'))
              .onPressed,
          isNull,
        );
        await tester.tap(apply);
        await tester.pump();
        expect(
          core.calls.where((c) => c['method'] == 'execute').length,
          count + 1,
        );
        await tester.ensureVisible(find.text('Cancel'));
        await tester.tap(find.text('Cancel'));
        await tester.pump();
        expect(app.currentState!.id, 'C01');
        core.executionGate!.complete();
        await tester.pumpAndSettle();
        expect(app.currentState!.id, 'C01');
        await open(0, 'set');
        tester.view.physicalSize = const Size(640, 1688);
        tester.platformDispatcher.textScaleFactorTestValue = 2;
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        await capture('Narrow fixture form at 200 percent text');
        tester.view.viewInsets = const FakeViewPadding(bottom: 560);
        await tester.pumpAndSettle();
        await enter('brightness', '50');
        final visibleField = find.byKey(const ValueKey('parameter:brightness'));
        await tester.ensureVisible(visibleField);
        await tester.pumpAndSettle();
        expect(tester.getRect(visibleField).bottom, lessThanOrEqualTo(564));
        await capture('Narrow large-text form with simulated keyboard');
        tester.view.resetViewInsets();
        await tester.pumpAndSettle();

        await tap(find.text('Cancel'));
        expect(app.currentState!.id, 'C01');
        File('${output.path}/manifest.json').writeAsStringSync(
          '${const JsonEncoder.withIndent('  ').convert({'scope': 'Isolated descriptor fixtures, not native adapters or phone evidence', 'captures': captures})}\n',
        );
      } finally {
        await tester.pumpWidget(const SizedBox());
        tester.view.resetViewInsets();
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
        tester.platformDispatcher.clearTextScaleFactorTestValue();
        WidgetController.hitTestWarningShouldBeFatal = false;
      }
    },
  );
}
