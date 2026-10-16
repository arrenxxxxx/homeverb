import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/account_views.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'accounts_test.dart'
    show AccountFixture, AccountPreferences, firstAccount;

class InspectionFixture extends AccountFixture {
  Completer<Json>? response;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method.startsWith('inspect_')) {
      requests.add({'method': method, ...fields});
      return response!.future;
    }
    return super.call(method, fields);
  }
}

void main() {
  test(
    'current Core negotiates inspections and retains read-only authority',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'homeverb-api-122-',
      );
      final core = await NativeClient.open(
        dataDir: directory.path,
        libraryPath: File(
          '../../target/debug/${NativeClient.defaultLibrary()}',
        ).absolute.path,
      );
      try {
        expect(core.apiAtLeast(22), true);
        expect(
          core.methods,
          containsAll([
            'inspect_device_conflicts',
            'inspect_session_grants',
            'inspect_control_options',
          ]),
        );
        await core.call('discover');
        final cards = await core.call('list_device_cards') as List;
        final device = cards.single['device'] as Json;
        for (final connection in device['connections'] as List) {
          final result =
              await core.call('inspect_session_grants', {
                    'device_id': device['id'],
                    'connection_id': connection['id'],
                  })
                  as Json;
          expect(result['grants'], isEmpty);
          expect(result['source'], 'session_policy');
        }
      } finally {
        await core.close();
        await directory.delete(recursive: true);
      }
    },
  );

  test(
    'conflict inspection accepts unpublished bindings and fences late replies',
    () async {
      final examples =
          jsonDecode(
                File(
                  '../../packages/native-contract/account-examples.json',
                ).readAsStringSync(),
              )
              as Json;
      final result =
          (examples['cases'] as List).singleWhere(
                (entry) =>
                    entry['response_definition'] == 'inspect_device_conflicts',
              )['response']['data']
              as Json;
      final core = InspectionFixture();
      final c = HomeController(core, AccountPreferences());
      await c.selectAccount(firstAccount);
      const unpublished = 'unpublished-local-binding';
      c.accountInventory = {
        'devices': [
          {'device_id': unpublished},
        ],
      };
      core.response = Completer<Json>();
      final pending = c.inspectDeviceConflicts(unpublished);
      expect(core.requests.last, {
        'method': 'inspect_device_conflicts',
        'account_id': firstAccount,
        'device_id': unpublished,
      });
      c.setForeground(false);
      core.response!.complete(result);
      await pending;
      expect(c.accountConflicts, isNull);
      c.setForeground(true);
      c.accountInventory = {
        'devices': [
          {'device_id': unpublished},
        ],
      };
      core.response = Completer<Json>()..complete(result);
      expect(await c.inspectDeviceConflicts(unpublished), true);
      expect(c.accountConflicts, result);
      core.response = Completer<Json>()
        ..completeError(NativeFailure('RESOURCE_BUSY', 'Fictional refusal.'));
      expect(await c.inspectDeviceConflicts(unpublished), false);
      expect(c.accountConflicts, isNull);
      expect(
        core.requests.where(
          (r) =>
              ['refresh_account', 'discover', 'execute'].contains(r['method']),
        ),
        isEmpty,
      );
      c.dispose();
    },
  );

  test(
    'route inspection binds an exact saved connection and clears on cancellation',
    () async {
      final core = InspectionFixture();
      final c = HomeController(core, AccountPreferences());
      c.selectedDevice = 'device-local';
      c.description = {
        'device': {
          'connections': [
            {'id': 'connection-local'},
          ],
        },
      };
      expect(await c.inspectRoute('foreign', options: false), false);
      expect(core.requests, isEmpty);
      core.response = Completer<Json>();
      final pending = c.inspectRoute('connection-local', options: false);
      expect(core.requests.single, {
        'method': 'inspect_session_grants',
        'device_id': 'device-local',
        'connection_id': 'connection-local',
      });
      c.cancelPending();
      core.response!.complete({'grants': []});
      await pending;
      expect(c.routeInspection, isNull);
      c.dispose();
    },
  );

  test(
    'historical complete inventory time is labelled without live evidence',
    () async {
      final core = InspectionFixture();
      final c = HomeController(core, AccountPreferences());
      await c.selectAccount(firstAccount);
      expect(
        jsonEncode(accountBlocks(c, 'P09')),
        contains('No complete device-list time recorded'),
      );
      c.account!['last_complete_inventory_at'] = '2026-10-16T01:00:00Z';
      final view = jsonEncode(accountBlocks(c, 'P09'));
      expect(view, contains('2026-10-16T01:00:00Z'));
      expect(view, contains('not current authorization or appliance status'));
      c.dispose();
    },
  );
}
