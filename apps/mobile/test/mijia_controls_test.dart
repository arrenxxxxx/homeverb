import 'dart:io';
import 'package:flutter/material.dart';
import 'package:homeverb_mobile/controls.dart';
import 'package:homeverb_mobile/device_detail_controls.dart';
import 'package:homeverb_mobile/skin.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/session_permissions.dart';
import 'accounts_test.dart' show AccountPreferences, firstAccount;
import 'session_permissions_test.dart' show PermissionCore, inspection;

Json fanInspection() {
  final result = inspection();
  Json option(String property, Json domain) => {
    'component_id': 'fan-2',
    'capability': property == 'state' ? 'power' : 'vendor.xiaomi.mijia.fan',
    'action': property == 'state' ? 'set' : 'set_$property',
    'definition': {
      'parameters': {
        'type': 'object',
        'properties': {property: domain},
        'required': [property],
        'additionalProperties': false,
      },
      'transports': ['cloud'],
      'risk': 'low',
      'idempotency': 'set_value',
      'atomic': false,
      'independent_fields': false,
      'verification': 'unavailable',
    },
  };
  result['options'] = [
    option('state', {'type': 'boolean'}),
    option('level', {
      'type': 'integer',
      'minimum': 1,
      'maximum': 4,
      'multipleOf': 1,
    }),
    option('mode', {
      'type': 'integer',
      'enum': [0, 2],
    }),
    option('horizontal_swing', {'type': 'boolean'}),
    option('horizontal_angle', {
      'type': 'integer',
      'enum': [30, 90],
    }),
  ];
  return result;
}

class FanPermissionCore extends PermissionCore {
  FanPermissionCore(super.inspected, {this.includeTarget = true}) {
    adapter = 'homeverb.platforms';
    cards.single['device']['components'][0]['id'] = 'fan-2';
    cards.single['device']['alias'] = 'Fictional fan';
    cards.single['device']['components'][0]['capabilities'] = [
      'power',
      'vendor.xiaomi.mijia.fan',
    ];
  }
  final bool includeTarget;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'refresh_account') {
      requests.add({'method': method, ...fields});
      published = true;
      final result = report(firstAccount, 'not_attempted');
      result['retrieval'] = {
        'status': 'partial',
        'device_ids': includeTarget ? [inspected['device_id']] : [],
        'error': {
          'code': 'MIJIA_INVENTORY_PARTIAL',
          'message': 'Fictional partial inventory.',
          'retryable': false,
        },
      };
      return result;
    }
    if (method == 'describe_device') {
      final result = await super.call(method, fields) as Json;
      final capabilities = <String, Json>{};
      for (final option in inspected['options'] as List) {
        final id = option['capability'] as String;
        final cap = capabilities.putIfAbsent(
          id,
          () => {
            'id': id,
            'component_id': 'fan-2',
            'revision': 1,
            'properties': <String, dynamic>{},
            'actions': <String, dynamic>{},
            'events': <String, dynamic>{},
          },
        );
        for (final entry
            in (option['definition']['parameters']['properties'] as Json)
                .entries) {
          cap['properties'][entry.key] = {
            ...entry.value as Json,
            'readable': true,
            'writable': published,
          };
        }
        if (published) cap['actions'][option['action']] = option['definition'];
      }
      result['capabilities'] = capabilities.values.toList();
      return result;
    }
    return super.call(method, fields);
  }
}

void main() {
  test(
    'fan permissions preserve exact component and actual values without Meiju grants',
    () {
      final choices = SessionPermission.choices(fanInspection());
      expect(choices.length, 12);
      final startup = SessionPermission.startup(choices);
      expect(startup.keys, ['mijia_fan_grants']);
      final entries = startup['mijia_fan_grants'] as List;
      expect(entries.every((entry) => entry['component_id'] == 'fan-2'), true);
      expect(
        entries
            .where((entry) => entry['property'] == 'level')
            .map((entry) => entry['value']),
        [1, 2, 3, 4],
      );
      expect(
        entries
            .where((entry) => entry['property'] == 'mode')
            .map((entry) => entry['value']),
        [0, 2],
      );
      final changed = fanInspection();
      changed['options'][1]['definition']['parameters']['properties']['level']['maximum'] =
          2;
      expect(
        SessionPermission.choices(
          changed,
        ).where((p) => p.parameter == 'level').map((p) => p.value),
        [1, 2],
      );
    },
  );

  test(
    'fan permission preparation refuses other services, risk and unsupported value schemas',
    () {
      for (final edit in <void Function(Json)>[
        (option) => option['component_id'] = 'lock-2',
        (option) => option['action'] = 'unlock',
        (option) => option['definition']['risk'] = 'high',
        (option) =>
            option['definition']['parameters']['properties']['level'] = {
              'type': 'string',
              'enum': ['3'],
            },
        (option) =>
            option['definition']['parameters']['properties']['level']['not'] = {
              'const': 3,
            },
      ]) {
        final inspected = fanInspection();
        final option = inspected['options'][1] as Json;
        edit(option);
        inspected['options'] = [option];
        expect(SessionPermission.choices(inspected), isEmpty);
      }
    },
  );

  for (final includeTarget in [true, false]) {
    test(
      'partial fan inventory prepares controls only with positive target publication: $includeTarget',
      () async {
        final inspected = fanInspection();
        final old = FanPermissionCore(inspected);
        final next = FanPermissionCore(inspected, includeTarget: includeTarget);
        final c = HomeController(
          old,
          AccountPreferences(),
          reopen: (permissions) async {
            expect(SessionPermission.startup(permissions).keys, [
              'mijia_fan_grants',
            ]);
            return next;
          },
        );
        expect(await c.initialize(), true);
        expect(await c.selectDevice(inspected['device_id']), includeTarget);
        expect(
          next.requests.where((r) => r['method'] == 'refresh_account').length,
          1,
        );
        expect(
          next.requests.where((r) => ['plan', 'execute'].contains(r['method'])),
          isEmpty,
        );
        if (includeTarget) expect(c.hasControlAction('power', 'set'), true);
        c.dispose();
      },
    );
  }

  testWidgets(
    'fan detail exposes localized bounded controls without sending when opened',
    (tester) async {
      final inspected = fanInspection();
      final old = FanPermissionCore(inspected),
          next = FanPermissionCore(inspected);
      final c = HomeController(
        old,
        AccountPreferences(),
        reopen: (_) async => next,
      );
      expect(await c.initialize(), true);
      expect(await c.selectDevice(inspected['device_id']), true);
      String? selected;
      await tester.pumpWidget(
        MaterialApp(
          theme: windowTheme(),
          home: Scaffold(
            body: SingleChildScrollView(
              child: DeviceDetailControls(
                controller: c,
                onAction: (action) {
                  selected = action;
                },
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Set fan level'), findsOneWidget);
      expect(find.text('Set fan oscillation'), findsOneWidget);
      final button = find.byKey(const ValueKey('control:1:set_level'));
      await tester.ensureVisible(button);
      await tester.tap(button);
      await tester.pumpAndSettle();
      expect(selected, 'control:1:set_level');
      expect(
        next.requests.where((r) => ['plan', 'execute'].contains(r['method'])),
        isEmpty,
      );
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );

  testWidgets(
    'fan form rejects out-of-domain values and submits exact scalar arguments',
    (tester) async {
      final option = fanInspection()['options'][1];
      Json? submitted;
      await tester.pumpWidget(
        MaterialApp(
          theme: windowTheme(),
          home: Scaffold(
            body: SingleChildScrollView(
              child: CapabilityActionForm(
                capability: {
                  'id': 'vendor.xiaomi.mijia.fan',
                  'properties': {},
                  'actions': {'set_level': option['definition']},
                },
                action: 'set_level',
                observation: (_) => null,
                onSubmit: (arguments) async {
                  submitted = arguments;
                  return true;
                },
                onCancel: () {},
                error: () => null,
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final field = find.byKey(const ValueKey('parameter:level'));
      await tester.enterText(field, '5');
      await tester.tap(find.text('Apply'));
      await tester.pumpAndSettle();
      expect(submitted, isNull);
      expect(find.textContaining('Maximum: 4'), findsOneWidget);
      await tester.enterText(field, '3');
      await tester.tap(find.text('Apply'));
      await tester.pumpAndSettle();
      expect(submitted, {'level': 3});
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );

  test(
    'native Mijia-only and combined sessions accept exact fan startup configuration locally',
    () async {
      for (final adapter in ['xiaomi.mijia', 'homeverb.platforms']) {
        final directory = await Directory.systemTemp.createTemp(
          'homeverb-fan-startup-',
        );
        NativeClient? client;
        try {
          client = await NativeClient.open(
            dataDir: directory.path,
            adapter: adapter,
            libraryPath: '../../target/debug/${NativeClient.defaultLibrary()}',
            permissions: SessionPermission.choices(fanInspection()),
          );
          expect(client.apiAtLeast(25), true);
          expect(await client.call('list_devices'), isEmpty);
        } finally {
          await client?.close();
          await directory.delete(recursive: true);
        }
      }
    },
  );
}
