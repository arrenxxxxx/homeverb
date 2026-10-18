import 'dart:convert';
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/session_permissions.dart';
import 'accounts_test.dart' show AccountPreferences;
import 'controls_test.dart' show example;
import 'session_permissions_test.dart' show inspection, PermissionCore;

Json dishwasherInspection({bool additional = true}) {
  final result = inspection();
  final start = example('capability.meiju-dishwasher');
  final cycles = example(
    additional
        ? 'capability.meiju-dishwasher-cycle-additional'
        : 'capability.meiju-dishwasher-cycle',
  );
  if (!additional) {
    start['actions']['start_cycle']['parameters']['properties'].remove(
      'additional',
    );
    start['actions']['start_cycle']['parameters']['required'] = ['program'];
  }
  result['options'] = [
    for (final entry in <String, dynamic>{
      ...start['actions'],
      ...cycles['actions'],
    }.entries)
      {
        'component_id': 'main',
        'capability': 'vendor.midea.dishwasher',
        'action': entry.key,
        'definition': entry.value,
      },
  ];
  return result;
}

class PreparedDishwasher extends PermissionCore implements ApprovalClient {
  PreparedDishwasher(super.inspected) {
    cards.single['device']['components'].single['capabilities'] = [
      'vendor.midea.dishwasher',
    ];
  }
  @override
  bool get supportsTrustedApproval => true;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'describe_device') {
      requests.add({'method': method, ...fields});
      return {
        'device': cards.single['device'],
        'capabilities': [
          {
            'id': 'vendor.midea.dishwasher',
            'properties': {},
            'actions': {
              if (published)
                for (final option in inspected['options'])
                  option['action'] as String: option['definition'],
            },
          },
        ],
      };
    }
    return super.call(method, fields);
  }

  @override
  Future<ApprovalPreview> prepareApproval(Json command) async =>
      throw StateError('Opening a device must not prepare a command.');
  @override
  Future<Json> executeApproved(ApprovalPreview preview) async =>
      throw StateError('Opening a device must not execute a command.');
  @override
  Future<void> discardApproval(ApprovalPreview preview) async {}
}

void main() {
  test(
    'dishwasher grants enumerate immutable exact arguments only with trusted approval',
    () {
      final inspected = dishwasherInspection();
      expect(SessionPermission.choices(inspected), isEmpty);
      final permissions = SessionPermission.choices(
        inspected,
        trustedApproval: true,
      );
      expect(permissions, hasLength(75));
      final startup = SessionPermission.startup(permissions);
      final starts = startup['meiju_dishwasher_grants'] as List;
      final cycles = startup['meiju_dishwasher_cycle_grants'] as List;
      expect(starts, hasLength(60));
      expect(cycles, hasLength(15));
      for (final additional in [0, 9, 10, 13, 14]) {
        expect(
          starts.where((g) => g['arguments']['additional'] == additional),
          hasLength(12),
        );
        for (final action in ['pause_cycle', 'resume_cycle', 'cancel_cycle']) {
          expect(
            cycles,
            contains(
              equals({
                'device_id': inspected['device_id'],
                'action': action,
                'arguments': {'additional': additional},
              }),
            ),
          );
        }
      }
      expect(permissions.first.arguments, {
        'program': 'strong_wash',
        'additional': 0,
      });
      expect(
        () => permissions.first.arguments['additional'] = 9,
        throwsUnsupportedError,
      );
      expect(
        () => (permissions.first.value as Map)['program'] = 'changed',
        throwsUnsupportedError,
      );
      inspected['options']
              .first['definition']['parameters']['properties']['program']['enum'][0] =
          'changed';
      expect(permissions.first.arguments['program'], 'strong_wash');
      expect(
        SessionPermission.startup([...permissions, ...permissions]),
        startup,
      );
    },
  );

  test(
    'legacy cycle operations keep empty arguments without adding an option',
    () {
      final inspected = dishwasherInspection(additional: false);
      final permissions = SessionPermission.choices(
        inspected,
        trustedApproval: true,
      );
      final grants = SessionPermission.startup(permissions);
      expect(grants['meiju_dishwasher_grants'], hasLength(12));
      expect(grants['meiju_dishwasher_cycle_grants'], hasLength(3));
      expect(
        (grants['meiju_dishwasher_cycle_grants'] as List).every(
          (g) => (g['arguments'] as Map).isEmpty,
        ),
        true,
      );
    },
  );

  test(
    'unknown, optional, malformed and oversized schemas never create dishwasher authority',
    () {
      final original = dishwasherInspection();
      final start = original['options'].first as Json;
      final mutations = <void Function(Json)>[
        (v) => v['component_id'] = 'freezer',
        (v) => v['action'] = 'open_door',
        (v) => v['definition']['risk'] = 'high',
        (v) => v['definition']['risk'] = 'low',
        (v) => v['definition']['idempotency'] = 'set_value',
        (v) => v['definition']['atomic'] = true,
        (v) => v['definition']['independent_fields'] = true,
        (v) => v['definition']['transports'] = ['cloud', 'lan'],
        (v) => v['definition']['parameters']['additionalProperties'] = true,
        (v) => v['definition']['parameters']['required'] = ['program'],
        (v) =>
            v['definition']['parameters']['required'] = ['program', 'program'],
        (v) => v['definition']['parameters']['oneOf'] = [],
        (v) =>
            v['definition']['parameters']['properties']['additional']['enum'] =
                [0, 9.5],
        (v) =>
            v['definition']['parameters']['properties']['additional']['enum'] =
                [0, 0],
        (v) =>
            v['definition']['parameters']['properties']['program']['enum'] = [],
        (v) => v['definition']['parameters']['properties']['program']['enum'] =
            List.generate(65, (i) => 'program-$i'),
        (v) => v['definition']['parameters']['properties']['program']['enum'] =
            List.generate(13, (i) => 'program-$i'),
        (v) =>
            v['definition']['parameters']['properties']['program']['pattern'] =
                '.*',
      ];
      for (final mutate in mutations) {
        final altered = jsonDecode(jsonEncode(start)) as Json;
        mutate(altered);
        expect(
          SessionPermission.choices({
            ...original,
            'options': [altered],
          }, trustedApproval: true),
          isEmpty,
        );
      }
      for (final field in ['source', 'authorization']) {
        expect(
          SessionPermission.choices({
            ...original,
            field: 'untrusted',
          }, trustedApproval: true),
          isEmpty,
        );
      }
    },
  );

  test(
    'native startup accepts generated grants without accounts or appliance traffic',
    () async {
      final dir = await Directory.systemTemp.createTemp(
        'homeverb-dishwasher-startup-',
      );
      NativeClient? core;
      try {
        core = await NativeClient.open(
          dataDir: dir.path,
          adapter: 'midea.meiju',
          libraryPath: File(
            '../../target/debug/${NativeClient.defaultLibrary()}',
          ).absolute.path,
          permissions: SessionPermission.choices(
            dishwasherInspection(),
            trustedApproval: true,
          ),
        );
        expect(core.supportsTrustedApproval, true);
        expect(await core.call('list_devices'), isEmpty);
      } finally {
        await core?.close();
        await dir.delete(recursive: true);
      }
    },
  );

  test(
    'device opening prepares one selected dishwasher then reuses its exact session',
    () async {
      final inspected = dishwasherInspection();
      var session = PreparedDishwasher(inspected);
      final first = session;
      var reopened = 0;
      final controller = HomeController(
        session,
        AccountPreferences(),
        reopen: (permissions) async {
          reopened++;
          expect(permissions, hasLength(75));
          session = PreparedDishwasher(inspected);
          return session;
        },
      );
      try {
        await controller.initialize();
        expect(reopened, 0);
        expect(await controller.selectDevice(inspected['device_id']), true);
        expect(first.closed, true);
        expect(reopened, 1);
        expect(
          controller.hasControlAction('vendor.midea.dishwasher', 'start_cycle'),
          true,
        );
        expect(
          controller.hasControlAction('vendor.midea.dishwasher', 'pause_cycle'),
          true,
        );
        expect(
          session.requests.where((v) => v['method'] == 'refresh_account'),
          hasLength(1),
        );
        expect(await controller.selectDevice(inspected['device_id']), true);
        expect(reopened, 1);
        expect(
          session.requests.where((v) => v['method'] == 'refresh_account'),
          hasLength(1),
        );
        controller.preferences.values['route'] = 'lan';
        expect(
          controller.hasControlAction('vendor.midea.dishwasher', 'start_cycle'),
          false,
        );
        expect(
          session.requests.where(
            (v) => ['execute', 'plan'].contains(v['method']),
          ),
          isEmpty,
        );
      } finally {
        controller.dispose();
      }
    },
  );
}
