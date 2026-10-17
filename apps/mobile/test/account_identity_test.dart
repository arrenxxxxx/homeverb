import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/account_identity.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'accounts_test.dart'
    show AccountFixture, AccountPreferences, firstAccount;

Uint8List bytes(String value) => Uint8List.fromList(value.codeUnits);

void main() {
  test(
    'login labels mask actual inputs without retaining complete identifiers',
    () {
      expect(maskedAccountLogin(bytes('13812345678')), '138****5678');
      expect(maskedAccountLogin(bytes('+8613812345678')), '138****5678');
      expect(
        maskedAccountLogin(bytes('pilot@example.invalid')),
        'p***@example.invalid',
      );
      expect(maskedAccountLogin(bytes('fictional-user')), 'fi***er');
      expect(maskedAccountLogin(bytes('ab')), 'a***');
      expect(maskedAccountLogin(bytes('a b')), isNull);
      expect(maskedAccountLogin(bytes('')), isNull);
      expect(maskedAccountLogin(Uint8List.fromList([0xff])), isNull);
    },
  );

  test(
    'remarks override masked login, clearing restores it, removal prunes it',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'homeverb-account-name-',
      );
      final prefs = LocalPreferences(
        File('${directory.path}/preferences.json'),
      );
      final core = AccountFixture()
        ..record(firstAccount)['label'] = 'Meiju account';
      final c = HomeController(core, prefs);
      try {
        await c.initialize();
        final login = bytes('13812345678'),
            password = bytes('fictional-password');
        expect(await c.connectAccount(login, password, ''), true);
        expect(core.connectionLabels.single, '138****5678');
        expect(c.accountDisplayName(c.account!), '138****5678');
        expect(c.accountRemark(c.account!), isEmpty);
        expect(login.every((byte) => byte == 0), true);
        expect(password.every((byte) => byte == 0), true);
        final reopened = LocalPreferences(prefs.file!);
        await reopened.load();
        expect(reopened.accountLoginLabel(firstAccount), '138****5678');
        final stored = await prefs.file!.readAsString();
        expect(stored, isNot(contains('13812345678')));
        expect(stored, isNot(contains('fictional-password')));
        expect(await c.renameAccount('Upstairs'), true);
        expect(c.accountDisplayName(c.account!), 'Upstairs');
        expect(await c.renameAccount(''), true);
        expect(c.accountDisplayName(c.account!), '138****5678');
        expect(c.accountRemark(c.account!), isEmpty);
        expect(await c.reviewAccountRemoval(), true);
        expect(await c.removeAccount(), true);
        expect(prefs.accountLoginLabel(firstAccount), isNull);
        expect(
          jsonDecode(await prefs.file!.readAsString())['account_login_labels'],
          isEmpty,
        );
      } finally {
        c.dispose();
        await directory.delete(recursive: true);
      }
    },
  );

  test(
    'failed or late authentication cannot persist a new login label',
    () async {
      for (final late in [false, true]) {
        final core = AccountFixture()
          ..record(firstAccount)['label'] = 'Meiju account';
        final prefs = AccountPreferences();
        final c = HomeController(core, prefs);
        if (late) {
          core.connectionGate = Completer<void>();
        } else {
          core.connectionError = 'AUTH_REQUIRED';
        }
        final login = bytes('pilot@example.invalid'),
            password = bytes('fictional-password');
        final connecting = c.connectAccount(login, password, '');
        await Future<void>.delayed(Duration.zero);
        if (late) {
          c.setForeground(false);
          core.connectionGate!.complete();
        }
        await connecting;
        expect(prefs.accountLoginLabel(firstAccount), isNull);
        expect(login.every((byte) => byte == 0), true);
        expect(password.every((byte) => byte == 0), true);
        c.dispose();
      }
    },
  );
}
