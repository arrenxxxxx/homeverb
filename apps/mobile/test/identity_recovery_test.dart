import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:homeverb_mobile/account_views.dart';
import 'credential_access_test.dart' show FixtureAccess;
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/client_contract.dart';
import 'accounts_test.dart'
    show AccountFixture, AccountPreferences, firstAccount;

class Preview implements IdentityRecoveryPreview {
  @override
  Json account = {'id': firstAccount, 'label': 'Kept account'};
  @override
  int deviceCount = 1;
}

class RecoveryFixture extends AccountFixture implements IdentityRecoveryClient {
  @override
  bool supportsIdentityRecovery = true;
  int prepared = 0, confirmed = 0, discarded = 0, closed = 0;
  bool uncertain = false;
  @override
  Future<IdentityRecoveryPreview> prepareIdentityRecovery({
    required Uint8List login,
    required Uint8List password,
    required String accountId,
    required String label,
  }) async {
    prepared++;
    expect(accountId, firstAccount);
    login.fillRange(0, login.length, 0);
    password.fillRange(0, password.length, 0);
    return Preview();
  }

  @override
  Future<Json> confirmIdentityRecovery(IdentityRecoveryPreview preview) async {
    confirmed++;
    if (uncertain)
      throw NativeFailure(
        'SECRET_STORE_WRITE_UNKNOWN',
        'Fictional uncertainty.',
      );
    return report(firstAccount, 'succeeded');
  }

  @override
  Future<void> discardIdentityRecovery(IdentityRecoveryPreview preview) async {
    discarded++;
  }

  @override
  Future<void> close() async {
    closed++;
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  testWidgets(
    'restore action consumes protected fields through the separate recovery path',
    (tester) async {
      var normal = 0, recovery = 0;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: AccountSignIn(
                reauthorizing: true,
                credentialAccess: FixtureAccess(),
                busy: false,
                needsReload: false,
                onReload: () {},
                onCancel: () {},
                onSubmit: (login, password) async {
                  normal++;
                },
                onRecover: (login, password) async {
                  recovery++;
                  expect(String.fromCharCodes(login), 'fixture@example.test');
                  expect(String.fromCharCodes(password), 'Fictional password');
                  for (final field in tester.widgetList<TextField>(
                    find.byType(TextField),
                  )) {
                    expect(field.controller!.text, isEmpty);
                  }
                },
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const ValueKey('credential:Meiju account')),
        'fixture@example.test',
      );
      await tester.enterText(
        find.byKey(const ValueKey('credential:Password')),
        'Fictional password',
      );
      await tester.ensureVisible(find.text('Restore Saved Account'));
      await tester.tap(find.text('Restore Saved Account'));
      await tester.pumpAndSettle();
      expect(recovery, 1);
      expect(normal, 0);
    },
  );
  for (final mode in ['confirm', 'decline', 'background', 'uncertain']) {
    test(
      'legacy recovery $mode requires actual consent and clears credentials',
      () async {
        final old = RecoveryFixture()..uncertain = mode == 'uncertain';
        final replacement = AccountFixture();
        var reopened = 0;
        final controller = HomeController(
          old,
          AccountPreferences(),
          reopen: (grants) async {
            expect(grants, isEmpty);
            reopened++;
            return replacement;
          },
        )..selectedAccount = firstAccount;
        final login = Uint8List.fromList('fictional@example.test'.codeUnits);
        final password = Uint8List.fromList('Fictional password'.codeUnits);
        final result = await controller.recoverIdentity(
          login,
          password,
          'Kept account',
          (preview) async {
            expect(old.confirmed, 0);
            expect(preview.deviceCount, 1);
            expect(login.every((b) => b == 0), true);
            if (mode == 'background') controller.setForeground(false);
            return mode != 'decline';
          },
        );
        expect(old.prepared, 1);
        expect(old.confirmed, mode == 'confirm' || mode == 'uncertain' ? 1 : 0);
        expect(old.discarded, 1);
        expect(reopened, mode == 'confirm' || mode == 'uncertain' ? 1 : 0);
        expect(login.every((b) => b == 0), true);
        expect(password.every((b) => b == 0), true);
        expect(result, mode == 'confirm' || mode == 'decline');
        expect(old.requests.where((r) => r['method'] == 'execute'), isEmpty);
        controller.dispose();
      },
    );
  }
}
