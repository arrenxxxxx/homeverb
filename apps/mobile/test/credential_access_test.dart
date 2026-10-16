import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/account_views.dart';
import 'package:homeverb_mobile/credential_access.dart';

class FixtureAccess implements CredentialAccess {
  CredentialReadiness result = CredentialReadiness.ready;
  Completer<CredentialReadiness>? pending;
  int checks = 0, opens = 0;
  @override
  Future<CredentialReadiness> check() async {
    checks++;
    return pending == null ? result : await pending!.future;
  }

  @override
  Future<bool> openSecuritySettings() async {
    opens++;
    return true;
  }
}

Widget signIn(
  FixtureAccess access,
  Future<void> Function(Uint8List, Uint8List) submit,
) => MaterialApp(
  home: Scaffold(
    body: SingleChildScrollView(
      child: AccountSignIn(
        credentialAccess: access,
        busy: false,
        needsReload: false,
        onSubmit: submit,
        onReload: () {},
        onCancel: () {},
      ),
    ),
  ),
);

Future<void> enter(WidgetTester tester) async {
  await tester.enterText(
    find.byKey(const ValueKey('credential:Meiju account')),
    'fixture@example.invalid',
  );
  await tester.enterText(
    find.byKey(const ValueKey('credential:Password')),
    'fictional-password',
  );
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'Android metadata channel fails closed on absent, invalid and rejected replies',
    () async {
      const access = PlatformCredentialAccess(android: true);
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      addTearDown(
        () => messenger.setMockMethodCallHandler(
          PlatformCredentialAccess.channel,
          null,
        ),
      );
      expect(await access.check(), CredentialReadiness.unavailable);
      for (final entry in <Object?, CredentialReadiness>{
        'ready': CredentialReadiness.ready,
        'screen_lock_required': CredentialReadiness.screenLockRequired,
        'locked': CredentialReadiness.locked,
        'unavailable': CredentialReadiness.unavailable,
        'unexpected': CredentialReadiness.unavailable,
        null: CredentialReadiness.unavailable,
        true: CredentialReadiness.unavailable,
      }.entries) {
        messenger.setMockMethodCallHandler(PlatformCredentialAccess.channel, (
          call,
        ) async {
          expect(call.method, 'check');
          expect(call.arguments, isNull);
          return entry.key;
        });
        expect(await access.check(), entry.value);
      }
      messenger.setMockMethodCallHandler(
        PlatformCredentialAccess.channel,
        (_) async => throw PlatformException(code: 'unavailable'),
      );
      expect(await access.check(), CredentialReadiness.unavailable);
      expect(await access.openSecuritySettings(), isFalse);
      expect(
        await const PlatformCredentialAccess(android: false).check(),
        CredentialReadiness.notApplicable,
      );
    },
  );

  testWidgets(
    'unsecured devices show setup before accepting account credentials',
    (tester) async {
      final access = FixtureAccess()
        ..result = CredentialReadiness.screenLockRequired;
      var submissions = 0;
      await tester.pumpWidget(
        signIn(access, (_, _) async {
          submissions++;
        }),
      );
      await tester.pumpAndSettle();
      expect(find.byType(TextField), findsNothing);
      expect(find.text('Sign In and Retrieve Devices'), findsNothing);
      await tester.tap(find.text('Open Android Security Settings'));
      await tester.pumpAndSettle();
      expect(access.opens, 1);
      expect(submissions, 0);
      access.result = CredentialReadiness.ready;
      await tester.tap(find.text('Check Again'));
      await tester.pumpAndSettle();
      expect(find.byType(TextField), findsNWidgets(2));
      expect(submissions, 0);
    },
  );

  testWidgets(
    'submission rechecks storage and never sends when protection changed',
    (tester) async {
      final access = FixtureAccess();
      var submissions = 0;
      await tester.pumpWidget(
        signIn(access, (_, _) async {
          submissions++;
        }),
      );
      await tester.pumpAndSettle();
      await enter(tester);
      access.result = CredentialReadiness.screenLockRequired;
      await tester.tap(find.text('Sign In and Retrieve Devices'));
      await tester.pumpAndSettle();
      expect(access.checks, 2);
      expect(submissions, 0);
      expect(find.byType(TextField), findsNothing);
      expect(find.text('Open Android Security Settings'), findsOneWidget);
    },
  );

  testWidgets(
    'backgrounding invalidates an in-flight preflight without submitting or retaining input',
    (tester) async {
      final access = FixtureAccess();
      var submissions = 0;
      await tester.pumpWidget(
        signIn(access, (_, _) async {
          submissions++;
        }),
      );
      await tester.pumpAndSettle();
      await enter(tester);
      final pending = Completer<CredentialReadiness>();
      access.pending = pending;
      await tester.tap(find.text('Sign In and Retrieve Devices'));
      await tester.pump();
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
      pending.complete(CredentialReadiness.ready);
      await tester.pumpAndSettle();
      expect(submissions, 0);
      access.pending = null;
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpAndSettle();
      for (final field in tester.widgetList<TextField>(
        find.byType(TextField),
      )) {
        expect(field.controller!.text, isEmpty);
      }
      expect(submissions, 0);
    },
  );

  testWidgets(
    'ready storage submits once with cleared fields and zeroed temporary buffers',
    (tester) async {
      final access = FixtureAccess();
      var submissions = 0;
      Uint8List? login, password;
      await tester.pumpWidget(
        signIn(access, (l, p) async {
          submissions++;
          login = l;
          password = p;
          expect(String.fromCharCodes(l), 'fixture@example.invalid');
          for (final field in tester.widgetList<TextField>(
            find.byType(TextField),
          )) {
            expect(field.controller!.text, isEmpty);
          }
        }),
      );
      await tester.pumpAndSettle();
      await enter(tester);
      await tester.tap(find.text('Sign In and Retrieve Devices'));
      await tester.pumpAndSettle();
      expect(submissions, 1);
      expect(login!.every((v) => v == 0), isTrue);
      expect(password!.every((v) => v == 0), isTrue);
    },
  );
}
