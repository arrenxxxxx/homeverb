import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter/material.dart';
import 'package:homeverb_mobile/permission_review.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/session_permissions.dart';
import 'accounts_test.dart' show AccountFixture, AccountPreferences;

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
  bool closed = false;
  @override
  Future<void> close() async {
    closed = true;
  }
}

void main() {
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
  testWidgets(
    'permission selection requires a separate review and consent click',
    (tester) async {
      final inspected = inspection();
      inspected['options'] = [inspected['options'].first];
      Set<String>? granted;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: PermissionReview(
                inspection: inspected,
                deviceName: 'Fictional wall AC',
                busy: false,
                onConfirm: (values) async {
                  granted = values;
                  return true;
                },
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Power off'));
      await tester.pump();
      expect(granted, isNull);
      await tester.ensureVisible(find.text('Review Selected Permissions'));
      await tester.tap(find.text('Review Selected Permissions'));
      await tester.pumpAndSettle();
      expect(granted, isNull);
      expect(find.text('Power on'), findsNothing);
      await tester.ensureVisible(find.text('Allow Selected Controls'));
      await tester.tap(find.text('Allow Selected Controls'));
      await tester.pump();
      expect(granted, {SessionPermission.choices(inspected).first.identity});
    },
  );
  test(
    'exact choices use inspected model limits and separate startup grants',
    () {
      final inspected = inspection();
      final choices = SessionPermission.choices(inspected);
      expect(choices.length, 42);
      final grants = SessionPermission.startup(choices);
      expect((grants['meiju_power_grants'] as List).map((v) => v['state']), [
        false,
        true,
      ]);
      expect((grants['meiju_temperature_grants'] as List).length, 29);
      final narrow = inspected['options']
          .last['definition']['parameters']['properties']['target_temperature'];
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
    'consent reopens without network or write and revocation removes all authority',
    () async {
      final old = PermissionCore(), next = PermissionCore();
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
      final inspected = inspection();
      c.selectedDevice = inspected['device_id'];
      c.description = {
        'device': {
          'connections': [
            {'id': inspected['connection_id'], 'transport': 'cloud'},
          ],
        },
      };
      c.routeInspection = inspected;
      final selected = SessionPermission.choices(inspected).first;
      expect(await c.allowControlChoices(inspected, {selected.identity}), true);
      expect(starts.single.single.identity, selected.identity);
      expect(c.sessionPermissions.length, 1);
      expect(next.requests.map((v) => v['method']), [
        'list_rooms',
        'list_device_cards',
      ]);
      expect(c.preferences.values, isEmpty);
      expect(await c.revokeSessionControls(), true);
      expect(starts.last, isEmpty);
      expect(c.sessionPermissions, isEmpty);
      c.dispose();
    },
  );

  test(
    'navigation, foreign values and background during reopen invalidate consent',
    () async {
      final old = PermissionCore(), next = PermissionCore();
      final pending = Completer<CoreClient>();
      var opens = 0;
      final c = HomeController(
        old,
        AccountPreferences(),
        reopen: (grants) {
          opens++;
          return pending.future;
        },
      );
      final inspected = inspection();
      final selected = SessionPermission.choices(inspected).first;
      c.selectedDevice = inspected['device_id'];
      c.description = {
        'device': {
          'connections': [
            {'id': inspected['connection_id'], 'transport': 'cloud'},
          ],
        },
      };
      c.routeInspection = inspected;
      expect(await c.allowControlChoices(inspected, {'foreign'}), false);
      expect(opens, 0);
      c.cancelPending();
      expect(
        await c.allowControlChoices(inspected, {selected.identity}),
        false,
      );
      expect(opens, 0);
      c.routeInspection = inspected;
      final work = c.allowControlChoices(inspected, {selected.identity});
      await Future<void>.delayed(Duration.zero);
      expect(opens, 1);
      c.setForeground(false);
      pending.complete(next);
      expect(await work, false);
      expect(next.closed, true);
      expect(c.permissionSessionNeedsReopen, true);
      expect(c.sessionPermissions, isEmpty);
      expect(next.requests, isEmpty);
      c.dispose();
    },
  );
}
