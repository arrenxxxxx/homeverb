import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter/material.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/session_permissions.dart';
import 'accounts_test.dart'
    show AccountFixture, AccountPreferences, firstAccount;

Json inspection() {
  final examples = jsonDecode(
    File(
      '../../packages/native-contract/account-examples.json',
    ).readAsStringSync(),
  );
  return (examples['cases'] as List).firstWhere(
        (v) =>
            v['response_definition'] == 'inspect_control_options' &&
            (v['response']['data']?['options'] as List? ?? []).isNotEmpty,
      )['response']['data']
      as Json;
}

class PermissionCore extends AccountFixture {
  PermissionCore(this.inspected) {
    cards.clear();
    cards.add({
      'device': {
        'id': inspected['device_id'],
        'alias': 'Fictional wall AC',
        'model': 'Fixture',
        'support_status': 'experimental',
        'components': [
          {
            'id': 'main',
            'capabilities': ['power'],
          },
        ],
        'connections': [
          {
            'id': inspected['connection_id'],
            'transport': 'cloud',
            'account_id': firstAccount,
          },
        ],
      },
      'preferences': {'room_id': null, 'favorite': false},
    });
  }
  final Json inspected;
  bool closed = false, published = false, failRefresh = false;
  Completer<Json>? inspectionGate;
  @override
  Future<void> close() async {
    closed = true;
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'inspect_control_options') {
      requests.add({'method': method, ...fields});
      return inspectionGate == null ? inspected : await inspectionGate!.future;
    }
    if (method == 'describe_device') {
      requests.add({'method': method, ...fields});
      return {
        'device': cards.single['device'],
        'capabilities': [
          {
            'id': 'power',
            'version': 1,
            'properties': {},
            'actions': published
                ? {'set': inspected['options'].first['definition']}
                : {},
          },
        ],
      };
    }
    if (method == 'read_state') {
      requests.add({'method': method, ...fields});
      throw NativeFailure('STATE_UNAVAILABLE', 'No fictional cached state.');
    }
    if (method == 'refresh_account') {
      if (failRefresh) {
        requests.add({'method': method, ...fields});
        throw NativeFailure(
          'NETWORK_UNAVAILABLE',
          'Fictional refresh failure.',
        );
      }
      published = true;
    }
    return super.call(method, fields);
  }
}

void main() {
  test(
    'optional AC startup permissions retain exact independent feature values',
    () {
      final inspected = inspection();
      final cap = jsonDecode(
        File(
          '../../examples/capability.meiju-ac-features.json',
        ).readAsStringSync(),
      );
      for (final action in [
        'set_eco',
        'set_breezeless',
        'set_horizontal_direction',
      ]) {
        inspected['options'].add({
          'component_id': 'main',
          'capability': 'vendor.midea.ac',
          'action': action,
          'definition': cap['actions'][action],
        });
      }
      final permissions = SessionPermission.choices(inspected);
      final grants =
          SessionPermission.startup(permissions)['meiju_ac_feature_grants']
              as List;
      expect(grants.length, 22);
      expect(
        grants
            .where((g) => g['feature'] == 'eco')
            .map((g) => g['value'])
            .toList(),
        [false, true],
      );
      expect(
        grants
            .where((g) => g['feature'] == 'breezeless')
            .map((g) => g['value'])
            .toList(),
        [0, 1, 2, 3],
      );
      expect(
        grants.every(
          (g) =>
              g.keys.toSet().containsAll(['device_id', 'feature', 'value']) &&
              g.length == 3,
        ),
        isTrue,
      );
      expect(
        SessionPermission.startup([
          ...permissions,
          ...permissions,
        ])['meiju_ac_feature_grants'],
        grants,
      );
      cap['actions']['set_eco']['risk'] = 'elevated';
      inspected['options'].removeWhere(
        (g) =>
            g['action'] == 'set_breezeless' ||
            g['action'] == 'set_horizontal_direction' ||
            (g['action'] as String).startsWith('set_') &&
                !{'set_eco', 'set_fan_speed'}.contains(g['action']),
      );
      expect(
        SessionPermission.startup(
          SessionPermission.choices(inspected),
        ).containsKey('meiju_ac_feature_grants'),
        isFalse,
      );
    },
  );
  test(
    'native Meiju startup accepts exact host grants without account or networking',
    () async {
      final dir = await Directory.systemTemp.createTemp(
        'homeverb-permission-native-',
      );
      NativeClient? core;
      try {
        core = await NativeClient.open(
          dataDir: dir.path,
          libraryPath: File(
            '../../target/debug/${NativeClient.defaultLibrary()}',
          ).absolute.path,
          adapter: 'midea.meiju',
          permissions: SessionPermission.choices(inspection()),
        );
        expect(core.apiAtLeast(22), true);
        expect(await core.call('list_devices'), isEmpty);
        expect(await core.call('recent_executions'), isEmpty);
      } finally {
        await core?.close();
        await dir.delete(recursive: true);
      }
    },
  );
  test(
    'exact choices use inspected model limits and separate startup grants',
    () {
      final inspected = inspection();
      final choices = SessionPermission.choices(inspected);
      expect(choices.length, 52);
      final grants = SessionPermission.startup(choices);
      expect((grants['meiju_power_grants'] as List).map((v) => v['state']), [
        false,
        true,
      ]);
      expect((grants['meiju_temperature_grants'] as List).length, 29);
      final narrow = inspected['options'].firstWhere(
        (option) => option['capability'] == 'climate',
      )['definition']['parameters']['properties']['target_temperature'];
      narrow['minimum'] = 20;
      narrow['maximum'] = 22;
      narrow['multipleOf'] = 1;
      final temperatures = SessionPermission.choices(
        inspected,
      ).where((p) => p.capability == 'climate');
      expect(temperatures.map((p) => p.value), [20, 21, 22]);
      narrow['not'] = {'const': 21};
      expect(
        SessionPermission.choices(
          inspected,
        ).where((p) => p.capability == 'climate'),
        isEmpty,
      );
      inspected['options'][0]['definition']['risk'] = 'elevated';
      expect(
        SessionPermission.choices(
          inspected,
        ).where((p) => p.capability == 'power'),
        isEmpty,
      );
    },
  );

  test(
    'opening a device admits all reviewed controls without a permission form or startup refresh',
    () async {
      final inspected = inspection();
      final old = PermissionCore(inspected), next = PermissionCore(inspected);
      final starts = <List<SessionPermission>>[];
      final c = HomeController(
        old,
        AccountPreferences(),
        reopen: (grants) async {
          expect(old.closed, true);
          starts.add(grants);
          return next;
        },
      );
      expect(await c.initialize(), true);
      expect(
        old.requests.any(
          (v) => ['refresh_account', 'discover'].contains(v['method']),
        ),
        false,
      );
      expect(await c.selectDevice(inspected['device_id']), true);
      expect(
        starts.single.map((p) => p.identity).toSet(),
        SessionPermission.choices(inspected).map((p) => p.identity).toSet(),
      );
      expect(c.capability('power')!['actions'], contains('set'));
      expect(
        next.requests
            .where((v) => v['method'] == 'refresh_account')
            .single['account_id'],
        firstAccount,
      );
      expect(
        next.requests.any(
          (v) => ['execute', 'plan', 'discover'].contains(v['method']),
        ),
        false,
      );
      expect(c.preferences.values, isEmpty);
      expect(await c.selectDevice(inspected['device_id']), true);
      expect(starts.length, 1);
      expect(
        next.requests.where((v) => v['method'] == 'refresh_account').length,
        1,
      );
      c.dispose();
    },
  );

  test(
    'failed descriptor refresh retries preparation on the next explicit device open',
    () async {
      final inspected = inspection();
      final old = PermissionCore(inspected),
          failing = PermissionCore(inspected)..failRefresh = true;
      final recovered = PermissionCore(inspected);
      var opens = 0;
      final c = HomeController(
        old,
        AccountPreferences(),
        reopen: (_) async => ++opens == 1 ? failing : recovered,
      );
      expect(await c.selectDevice(inspected['device_id']), false);
      expect((c.error as NativeFailure).code, 'NETWORK_UNAVAILABLE');
      expect(c.capability('power')!['actions'], isEmpty);
      expect(await c.selectDevice(inspected['device_id']), true);
      expect(opens, 2);
      expect(c.capability('power')!['actions'], contains('set'));
      c.dispose();
    },
  );

  test(
    'foreign inspection and navigation during inspection cannot install controls',
    () async {
      final inspected = inspection();
      final old = PermissionCore(inspected);
      var opens = 0;
      final c = HomeController(
        old,
        AccountPreferences(),
        reopen: (_) async {
          opens++;
          return PermissionCore(inspected);
        },
      );
      old.inspectionGate = Completer<Json>();
      final work = c.selectDevice(inspected['device_id']);
      await Future<void>.delayed(Duration.zero);
      c.cancelPending();
      old.inspectionGate!.complete(inspected);
      expect(await work, false);
      expect(opens, 0);
      old.inspectionGate = Completer<Json>()
        ..complete({...inspected, 'connection_id': 'foreign'});
      expect(await c.selectDevice(inspected['device_id']), false);
      expect((c.error as NativeFailure).code, 'POLICY_DENIED');
      expect(opens, 0);
      c.dispose();
    },
  );

  test(
    'background during reopening closes the replacement and supports explicit recovery',
    () async {
      final inspected = inspection();
      final old = PermissionCore(inspected),
          lateCore = PermissionCore(inspected);
      final readonly = PermissionCore(inspected),
          recovered = PermissionCore(inspected);
      final pending = Completer<CoreClient>();
      var opens = 0;
      final c = HomeController(
        old,
        AccountPreferences(),
        reopen: (_) {
          opens++;
          return opens == 1
              ? pending.future
              : Future.value(opens == 2 ? readonly : recovered);
        },
      );
      final work = c.selectDevice(inspected['device_id']);
      await Future<void>.delayed(Duration.zero);
      expect(opens, 1);
      c.setForeground(false);
      pending.complete(lateCore);
      expect(await work, false);
      expect(lateCore.closed, true);
      expect(lateCore.requests, isEmpty);
      c.setForeground(true);
      expect(await c.selectDevice(inspected['device_id']), true);
      expect(opens, 3);
      expect(c.capability('power')!['actions'], contains('set'));
      c.dispose();
    },
  );

  testWidgets(
    'device information has no session-control selection or revoke actions',
    (tester) async {
      final inspected = inspection();
      final c = HomeController(
        PermissionCore(inspected),
        AccountPreferences(),
        reopen: (_) async => PermissionCore(inspected),
      );
      final key = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: key, catalog: catalog, controller: c),
      );
      await tester.pumpAndSettle();
      expect(await c.selectDevice(inspected['device_id']), true);
      await key.currentState!.go('D06');
      await tester.pumpAndSettle();
      expect(find.text('Device information'), findsOneWidget);
      for (final label in [
        'Review control options',
        'Allowed controls for this session',
        'Revoke Session Controls',
        'Allow controls for this session',
        'Retrieve Control Capabilities',
      ]) {
        expect(find.text(label), findsNothing);
      }
      expect(find.byType(CheckboxListTile), findsNothing);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
