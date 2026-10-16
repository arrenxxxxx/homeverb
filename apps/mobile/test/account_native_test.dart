import 'dart:io';
import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/account_views.dart';
import 'package:homeverb_mobile/preferences.dart';

void main() {
  final library = File('../../target/debug/libhomeverb_ffi.so').absolute.path;
  test(
    'protected FFI clears owned buffers on native rejection and wrapper refusals',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'homeverb-account-ffi-',
      );
      final core = await NativeClient.open(
        dataDir: directory.path,
        libraryPath: library,
      );
      try {
        expect(core.adapter, 'homeverb.mock');
        expect(core.supportsAccountConnection, true);
        expect(core.supportsPendingReauthorization, true);
        for (final refusal in [
          'native',
          'background',
          'capacity',
          'version',
          'invalid',
        ]) {
          final login = Uint8List.fromList('fixture@example.invalid'.codeUnits);
          final password = Uint8List.fromList(
            refusal == 'invalid' ? [0, 255] : 'fictional-password'.codeUnits,
          );
          if (refusal == 'background') core.foreground(false);
          if (refusal == 'capacity')
            core.active.addAll(List.generate(8, (i) => 'fixture-$i'));
          if (refusal == 'version') core.apiVersion = '1.0.0';
          final expected = {
            'native': 'AUTH_FLOW_UNSUPPORTED',
            'background': 'HOST_BACKGROUND',
            'capacity': 'BUSY',
            'version': 'API_UPGRADE_REQUIRED',
            'invalid': 'INVALID_ARGUMENT',
          }[refusal];
          await expectLater(
            core.connectAccount(
              login: login,
              password: password,
              label: 'Fictional',
            ),
            throwsA(
              isA<NativeFailure>().having((e) => e.code, 'code', expected),
            ),
          );
          expect(login.every((b) => b == 0), true);
          expect(password.every((b) => b == 0), true);
          core.active.clear();
          core.apiVersion = '1.9.0';
          core.foreground(true);
        }
        final queuedLogin = Uint8List.fromList(
          'fixture@example.invalid'.codeUnits,
        );
        final queuedPassword = Uint8List.fromList(
          'fictional-password'.codeUnits,
        );
        final queued = core.connectAccount(
          login: queuedLogin,
          password: queuedPassword,
          label: 'Fictional',
        );
        core.foreground(false);
        core.foreground(true);
        await expectLater(
          queued,
          throwsA(
            isA<NativeFailure>().having(
              (e) => e.code,
              'code',
              anyOf('AUTH_FLOW_UNSUPPORTED', 'HOST_BACKGROUND', 'CANCELLED'),
            ),
          ),
        );
        expect(queuedLogin.every((b) => b == 0), true);
        expect(queuedPassword.every((b) => b == 0), true);
        expect(core.active, isEmpty);
        expect(await core.call('list_accounts'), isEmpty);
      } finally {
        await core.close();
        await directory.delete(recursive: true);
      }
    },
  );

  test(
    'explicit read-only Meiju startup has a separate empty local profile',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'homeverb-empty-meiju-',
      );
      final core = await NativeClient.open(
        dataDir: directory.path,
        libraryPath: library,
        adapter: 'midea.meiju',
        allowMockWrites: false,
      );
      try {
        expect(core.adapter, 'midea.meiju');
        expect(
          core.methods,
          containsAll([
            'list_accounts',
            'get_account',
            'rename_account',
            'remove_account',
            'recover_account',
            'inspect_account_inventory',
          ]),
        );
        expect(await core.call('list_accounts'), isEmpty);
        expect(await core.call('list_device_cards'), isEmpty);
        final controller = HomeController(
          core,
          LocalPreferences(File('${directory.path}/presentation.json')),
        );
        const absentAccount = '00000000-0000-4000-8000-000000000001';
        controller.selectedAccount = absentAccount;
        controller.accountDetails = {
          'account': {
            'id': absentAccount,
            'label': 'Fictional absent account',
            'region': 'CN',
            'credential_state': 'stored',
          },
          'device_ids': <String>[],
        };
        expect(await controller.inspectAccountInventory(), false);
        expect((controller.error as NativeFailure).code, 'ACCOUNT_NOT_FOUND');
        expect(controller.accountInventory, isNull);
        core.methods.remove('inspect_account_inventory');
        expect(
          accountBlocks(controller, 'P09').toString(),
          isNot(contains('Inspect Saved Inventory')),
        );
        expect(await controller.inspectAccountInventory(), false);
        expect(
          (controller.error as NativeFailure).code,
          'UNSUPPORTED_OPERATION',
        );
        controller.dispose();
        // No discovery, credential lookup or vendor operation is requested.
      } finally {
        await core.close();
        await directory.delete(recursive: true);
      }
    },
  );
}
