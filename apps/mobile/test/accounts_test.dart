import 'dart:async';
import 'dart:io';
import 'dart:convert';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

const firstAccount = '00000000-0000-4000-8000-000000000001';
const secondAccount = '00000000-0000-4000-8000-000000000002';
const firstDevice = '10000000-0000-4000-8000-000000000001';
const secondDevice = '10000000-0000-4000-8000-000000000002';

class AccountPreferences extends LocalPreferences {
  AccountPreferences() : super(File('/unused-account-fixture.json'));
  @override
  Future<void> save(Json changes) async {
    values.addAll(changes);
  }
}

class AccountFixture implements CoreClient, AccountClient {
  @override
  String get adapter => 'midea.meiju';
  @override
  bool supportsAccountConnection = true;
  @override
  bool supportsPendingReauthorization = true;
  final requests = <Json>[];
  final connectionTargets = <String?>[];
  final connectionLabels = <String>[];
  final records = <Json>[
    for (final id in [firstAccount, secondAccount])
      {
        'id': id,
        'label': 'Home account',
        'provider': 'midea.meiju',
        'region': 'CN',
        'credential_state': 'stored',
        'authorization': 'not_checked',
      },
  ];
  final links = <String, List<String>>{
    firstAccount: [firstDevice],
    secondAccount: [secondDevice],
  };
  final cards = <Json>[
    for (final id in [firstDevice, secondDevice])
      {
        'device': {
          'id': id,
          'alias': 'Fixture AC',
          'model': 'Fixture',
          'support_status': 'unsupported',
          'components': [
            {'id': 'main', 'capabilities': []},
          ],
          'connections': [],
        },
        'preferences': {
          'alias': 'Fixture AC',
          'room_id': 'study',
          'favorite': true,
        },
      },
  ];
  String retrieval = 'complete', credentialState = 'stored';
  String? retrievalError, connectionError;
  bool pendingRemoval = false;
  int cancelled = 0;
  Completer<void>? connectionGate;
  Uint8List? ownedLogin, ownedPassword;
  Json record(String id) => records.singleWhere((a) => a['id'] == id);
  Json report(String id, String authentication) {
    record(id)['credential_state'] = credentialState;
    return {
      'account': {...record(id)},
      'authentication': authentication,
      'retrieval': {
        'status': retrieval,
        'device_ids': retrieval == 'complete' ? [...links[id]!] : [],
        'error': retrievalError == null
            ? null
            : {
                'code': retrievalError,
                'message': 'Fictional retrieval failure.',
              },
      },
    };
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    requests.add({'method': method, ...fields});
    final id = fields['account_id'] as String?;
    switch (method) {
      case 'list_rooms':
        return <Json>[
          {'id': 'study', 'name': 'Study'},
        ];
      case 'list_device_cards':
        return cards.map((c) => Map<String, dynamic>.from(c)).toList();
      case 'describe_device':
        return {
          'device': cards.singleWhere(
            (c) => c['device']['id'] == fields['device'],
          )['device'],
          'capabilities': <Json>[],
        };
      case 'list_accounts':
        return records.map((a) => Map<String, dynamic>.from(a)).toList();
      case 'get_account':
        return {
          'account': {...record(id!)},
          'device_ids': [...links[id]!],
        };
      case 'rename_account':
        record(id!)['label'] = fields['label'];
        return {...record(id)};
      case 'refresh_account':
      case 'recover_account':
        return report(id!, 'not_attempted');
      case 'remove_account':
        final expected = (fields['expected_device_ids'] as List).toSet();
        if (expected.length != links[id]!.length ||
            !expected.containsAll(links[id]!)) {
          throw NativeFailure(
            'ACCOUNT_SCOPE_CHANGED',
            'Review the changed local card set.',
          );
        }
        if (pendingRemoval) {
          record(id!)['credential_state'] = 'removing';
          return {
            'account_id': id,
            'device_ids': [...links[id]!],
            'status': 'pending',
            'remote_revocation': 'not_attempted',
            'error': {
              'code': 'SECRET_STORE_LOCKED',
              'message': 'Unlock secure storage and retry explicitly.',
            },
          };
        }
        final removed = [...links[id]!];
        records.removeWhere((a) => a['id'] == id);
        cards.removeWhere((c) => removed.contains(c['device']['id']));
        return {
          'account_id': id,
          'device_ids': removed,
          'status': 'removed',
          'remote_revocation': 'not_attempted',
          'error': null,
        };
      default:
        throw StateError('Unexpected fixture request: $method');
    }
  }

  @override
  Future<Json> connectAccount({
    required Uint8List login,
    required Uint8List password,
    required String label,
    String? accountId,
  }) async {
    connectionTargets.add(accountId);
    connectionLabels.add(label);
    ownedLogin = login;
    ownedPassword = password;
    login.fillRange(0, login.length, 0);
    password.fillRange(0, password.length, 0);
    if (connectionGate != null) await connectionGate!.future;
    if (connectionError != null)
      throw NativeFailure(
        connectionError!,
        'Fictional protected connection failure.',
      );
    return report(accountId ?? firstAccount, 'succeeded');
  }

  @override
  void cancelAll() {
    cancelled++;
  }

  @override
  void foreground(bool value) {}
  @override
  Future<void> close() async {}
}

void main() {
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
  });

  testWidgets(
    'account clicks use exact identities, preserve partial results and confirm changed removal scope',
    (tester) async {
      tester.view.physicalSize = const Size(780, 1688);
      tester.view.devicePixelRatio = 2;
      tester.view.padding = const FakeViewPadding(top: 88, bottom: 28);
      WidgetController.hitTestWarningShouldBeFatal = true;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
        tester.view.resetPadding();
        WidgetController.hitTestWarningShouldBeFatal = false;
      });
      final core = AccountFixture();
      final prefs = AccountPreferences();
      final controller = HomeController(core, prefs);
      final key = GlobalKey<NativeAppState>(), boundary = GlobalKey();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      final captures = <Json>[];
      final output = Directory('build/verification/accounts')
        ..createSync(recursive: true);
      Future<void> capture(String trigger) async {
        await tester.pumpAndSettle();
        final name =
            '${(captures.length + 1).toString().padLeft(3, '0')}-${key.currentState!.id}.png';
        await tester.runAsync(() async {
          final img =
              await (boundary.currentContext!.findRenderObject()
                      as RenderRepaintBoundary)
                  .toImage(pixelRatio: 2);
          final bytes = await img.toByteData(format: ui.ImageByteFormat.png);
          File(
            '${output.path}/$name',
          ).writeAsBytesSync(bytes!.buffer.asUint8List());
          img.dispose();
        });
        captures.add({
          'file': name,
          'screen': key.currentState!.id,
          'trigger': trigger,
        });
      }

      Future<void> tap(String text) async {
        final finder = find.text(text).evaluate().isNotEmpty
            ? find.text(text).last
            : find.byTooltip(text).last;
        await tester.ensureVisible(finder);
        await tester.tap(finder);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        await capture('Click: $text');
      }

      Future<void> account(String id) async {
        await key.currentState!.go('P01');
        await tester.pumpAndSettle();
        await tester.ensureVisible(find.byKey(ValueKey('account:$id')));
        await tester.tap(find.byKey(ValueKey('account:$id')));
        await tester.pumpAndSettle();
        expect(controller.selectedAccount, id);
        await capture('Open exact fictional account');
      }

      await tester.pumpWidget(
        RepaintBoundary(
          key: boundary,
          child: NativeApp(key: key, catalog: catalog, controller: controller),
        ),
      );
      await tester.pumpAndSettle();
      await key.currentState!.go('P01');
      await tester.pumpAndSettle();
      expect(find.textContaining('000000000001'), findsOneWidget);
      expect(find.textContaining('000000000002'), findsOneWidget);
      await capture('Same-name accounts have visible local identity suffixes');
      await account(secondAccount);
      expect(
        core.requests.every(
          (r) => ![
            'refresh_account',
            'recover_account',
            'discover',
            'read_state',
          ].contains(r['method']),
        ),
        true,
      );
      await tap('Fixture AC');
      expect(find.text('Unsupported model'), findsOneWidget);
      expect(find.text('Refresh State'), findsNothing);
      await tap('Back');
      await tap('Manage account');
      await tap('Account name');
      await tester.enterText(
        find.byKey(const ValueKey('field:Account name')),
        'x' * 81,
      );
      await tap('Save Account Name');
      expect(key.currentState!.id, 'P03');
      await tester.enterText(
        find.byKey(const ValueKey('field:Account name')),
        'Studio',
      );
      await tap('Save Account Name');
      expect(core.record(secondAccount)['label'], 'Studio');
      expect(core.record(firstAccount)['label'], 'Home account');
      core.retrieval = 'partial';
      core.retrievalError = 'MIDEA_INVENTORY_PARTIAL';
      await tap('Retrieve Devices');
      expect(find.text('Some devices could not be updated'), findsOneWidget);
      expect(controller.cards, hasLength(2));
      expect(core.connectionTargets, isEmpty);
      core.retrieval = 'failed';
      core.retrievalError = 'AUTH_REQUIRED';
      await tap('Retrieve Devices');
      expect(find.text('Could not update devices'), findsOneWidget);
      await tap('Reauthorize');
      final login = find.byKey(const ValueKey('credential:Meiju account'));
      final password = find.byKey(const ValueKey('credential:Password'));
      expect(tester.widget<TextField>(password).obscureText, true);
      expect(
        tester.widget<TextField>(password).enableIMEPersonalizedLearning,
        false,
      );
      await tap('Sign In and Retrieve Devices');
      expect(core.connectionTargets, isEmpty);
      await tester.enterText(login, 'fixture@example.invalid');
      await tester.enterText(password, 'gggg');
      await tester.pump();
      final maskedPadding = tester
          .widget<TextField>(password)
          .decoration!
          .contentPadding;
      await tester.enterText(password, 'AAAA');
      await tester.pump();
      expect(
        tester.widget<TextField>(password).decoration!.contentPadding,
        maskedPadding,
      );
      await tester.enterText(password, 'fictional-pass');
      core.retrieval = 'complete';
      core.retrievalError = null;
      await tap('Sign In and Retrieve Devices');
      expect(core.connectionTargets, [secondAccount]);
      expect(core.ownedLogin!.every((v) => v == 0), true);
      expect(core.ownedPassword!.every((v) => v == 0), true);
      expect(key.currentState!.id, 'P08');
      expect(
        core.requests.where((r) => r['method'] == 'refresh_account'),
        hasLength(2),
      );
      expect(prefs.values.toString(), isNot(contains('fictional-pass')));
      expect(core.record(secondAccount)['label'], 'Studio');
      core.record(secondAccount)['credential_state'] = 'pending';
      await account(secondAccount);
      await tap('Recover Saved Authorization');
      expect(
        core.requests.lastWhere(
          (r) => r['method'] == 'recover_account',
        )['account_id'],
        secondAccount,
      );
      expect(core.connectionTargets, hasLength(1));
      expect(controller.account!['credential_state'], 'stored');
      await account(secondAccount);
      expect(find.text('Revoke Access · if supported'), findsNothing);
      if (find.text('Remove From This Device').evaluate().isEmpty) {
        await tap('Manage account');
      }
      await tap('Remove From This Device');
      await tap('Cancel');
      if (find.text('Remove From This Device').evaluate().isEmpty) {
        await tap('Manage account');
      }
      expect(
        core.requests.where((r) => r['method'] == 'remove_account'),
        isEmpty,
      );
      await tap('Remove From This Device');
      core.links[secondAccount]!.add('10000000-0000-4000-8000-000000000003');
      await tap('Remove Locally');
      expect((controller.error as NativeFailure).code, 'ACCOUNT_SCOPE_CHANGED');
      expect(find.text('Remove Locally'), findsNothing);
      expect(
        core.requests.lastWhere(
          (r) => r['method'] == 'remove_account',
        )['expected_device_ids'],
        [secondDevice],
      );
      await tap('Review removal scope');
      await tester.ensureVisible(find.text('Saved card · 000000000003').last);
      expect(
        find.text('Saved card · 000000000003').hitTestable(),
        findsOneWidget,
      );
      expect(find.text('Details unavailable').hitTestable(), findsOneWidget);
      core.pendingRemoval = true;
      await tap('Remove Locally');
      expect(controller.accountRemoval!['status'], 'pending');
      expect(key.currentState!.id, 'P09');
      expect(controller.account!['credential_state'], 'removing');
      expect(find.text('Reauthorize'), findsNothing);
      expect(find.text('Account name'), findsNothing);
      await tap('Retry local cleanup');
      core.pendingRemoval = false;
      await tap('Remove Locally');
      expect(key.currentState!.id, 'P01');
      expect(core.records, hasLength(1));
      expect(core.records.single['id'], firstAccount);
      expect(controller.cards.single['device']['id'], firstDevice);
      expect(controller.rooms.single['id'], 'study');
      expect(
        core.requests.where((r) => r['method'] == 'remove_account'),
        hasLength(3),
      );
      expect(
        core.requests.any(
          (r) => ['execute', 'discover', 'read_state'].contains(r['method']),
        ),
        false,
      );
      File('${output.path}/manifest.json').writeAsStringSync(
        jsonEncode({
          'source':
              'Isolated synthetic account fixtures; no provider or secret-store call',
          'size': [780, 1688],
          'captures': captures,
        }),
      );
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets(
    'sign-in clears fields before completion and ignores a late cancelled result',
    (tester) async {
      final core = AccountFixture();
      final controller = HomeController(core, AccountPreferences());
      final key = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: key, catalog: catalog, controller: controller),
      );
      await tester.pumpAndSettle();
      await controller.selectAccount(firstAccount);
      await key.currentState!.go('P04');
      await tester.pumpAndSettle();
      final login = find.byKey(const ValueKey('credential:Meiju account')),
          password = find.byKey(const ValueKey('credential:Password'));
      await tester.enterText(login, 'fixture@example.invalid');
      await tester.enterText(password, 'fictional-pass');
      final loginController = tester.widget<TextField>(login).controller!;
      final passwordController = tester.widget<TextField>(password).controller!;
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.hidden);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      await tester.pump();
      expect(loginController.text, isEmpty);
      expect(passwordController.text, isEmpty);
      expect(login, findsNothing);
      expect(password, findsNothing);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.hidden);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpAndSettle();
      await tester.enterText(login, 'fixture@example.invalid');
      await tester.enterText(password, 'fictional-pass');
      core.connectionGate = Completer<void>();
      final submit = find.text('Sign In and Retrieve Devices');
      await tester.ensureVisible(submit);
      await tester.tap(submit);
      await tester.pump();
      expect(tester.widget<TextField>(login).controller!.text, isEmpty);
      expect(tester.widget<TextField>(password).controller!.text, isEmpty);
      expect(find.text('Connecting and retrieving devices…'), findsOneWidget);
      await tester.ensureVisible(find.text('Cancel').last);
      await tester.tap(find.text('Cancel').last);
      await tester.pump();
      final destination = key.currentState!.id;
      expect(destination, isNot('P04'));
      core.connectionGate!.complete();
      await tester.pumpAndSettle();
      expect(key.currentState!.id, destination);
      expect(core.connectionTargets, [firstAccount]);
      expect(core.cancelled, greaterThan(0));
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets('unavailable connection uses a native platform heading', (
    tester,
  ) async {
    final core = AccountFixture()..supportsAccountConnection = false;
    final controller = HomeController(core, AccountPreferences());
    final key = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: key, catalog: catalog, controller: controller),
    );
    await tester.pumpAndSettle();
    await key.currentState!.go('P02');
    await tester.pumpAndSettle();
    await tester.tap(find.text('Meiju'));
    await tester.pumpAndSettle();
    expect(key.currentState!.id, 'P13');
    expect(find.text('Platform connection'), findsOneWidget);
    expect(find.text('Flow unavailable'), findsOneWidget);
    expect(find.text('Example Climate'), findsNothing);
    expect(core.connectionTargets, isEmpty);
    await tester.tap(find.text('Back to Accounts'));
    await tester.pumpAndSettle();
    expect(key.currentState!.id, 'P01');
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets(
    'completed account result shows only the device count and next steps',
    (tester) async {
      final core = AccountFixture();
      core.links[firstAccount] = [firstDevice, secondDevice];
      final controller = HomeController(core, AccountPreferences());
      final key = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: key, catalog: catalog, controller: controller),
      );
      await tester.pumpAndSettle();
      await controller.selectAccount(firstAccount);
      await controller.connectAccount(
        Uint8List.fromList('fixture@example.invalid'.codeUnits),
        Uint8List.fromList('fictional-pass'.codeUnits),
        'Home account',
      );
      await key.currentState!.go('P08');
      await tester.pumpAndSettle();
      expect(find.text('Device list updated'), findsOneWidget);
      expect(find.text('2 devices found.'), findsOneWidget);
      expect(find.text('Retrieve Devices'), findsNothing);
      expect(find.text('Reauthorize'), findsNothing);
      expect(core.connectionTargets, [firstAccount]);
      expect(
        core.requests.where((r) => r['method'] == 'refresh_account'),
        isEmpty,
      );
      expect(find.text('Find Devices'), findsNothing);
      await tester.ensureVisible(find.text('View Devices'));
      await tester.tap(find.text('View Devices'));
      await tester.pumpAndSettle();
      expect(key.currentState!.id, 'D01');
      expect(controller.visibleCards, hasLength(2));
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets(
    'new account catalog connects once and empty inventory preserves saved cards',
    (tester) async {
      final core = AccountFixture();
      core.links[firstAccount] = [];
      final controller = HomeController(core, AccountPreferences());
      final key = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: key, catalog: catalog, controller: controller),
      );
      await tester.pumpAndSettle();
      await key.currentState!.go('P01');
      await tester.pumpAndSettle();
      Future<void> tap(String label) async {
        final finder = find.text(label).last;
        await tester.ensureVisible(finder);
        await tester.tap(finder);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
      }

      await tap('Add Platform');
      await tap('Meiju');
      expect(key.currentState!.id, 'P04');
      expect(find.text('Meiju sign-in'), findsOneWidget);
      expect(find.textContaining('Example Home'), findsNothing);
      await tester.enterText(
        find.byKey(const ValueKey('credential:Meiju account')),
        'fixture@example.invalid',
      );
      await tester.enterText(
        find.byKey(const ValueKey('credential:Password')),
        'fictional-pass',
      );
      await tap('Sign In and Retrieve Devices');
      expect(key.currentState!.id, 'P08');
      expect(core.connectionTargets, [null]);
      expect(find.text('0 devices found.'), findsOneWidget);
      expect(controller.cards, hasLength(2));
      expect(
        core.requests.any(
          (r) => [
            'discover',
            'refresh_account',
            'recover_account',
          ].contains(r['method']),
        ),
        false,
      );
      await tap('View Devices');
      expect(key.currentState!.id, 'D01');
      expect(controller.visibleCards, hasLength(2));
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets(
    'expired pending recovery offers exact-account sign-in at large text',
    (tester) async {
      tester.view.physicalSize = const Size(640, 1480);
      tester.view.devicePixelRatio = 2;
      tester.platformDispatcher.textScaleFactorTestValue = 2;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
        tester.platformDispatcher.clearTextScaleFactorTestValue();
      });
      final core = AccountFixture();
      core.record(firstAccount)['credential_state'] = 'pending';
      core.credentialState = 'pending';
      core.retrieval = 'failed';
      core.retrievalError = 'AUTH_REQUIRED';
      final controller = HomeController(core, AccountPreferences());
      final key = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: key, catalog: catalog, controller: controller),
      );
      await tester.pumpAndSettle();
      await controller.selectAccount(firstAccount);
      await key.currentState!.go('P10');
      await tester.pumpAndSettle();
      Future<void> tap(String label) async {
        final finder = find.text(label).last;
        await tester.ensureVisible(finder);
        await tester.tap(finder);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
      }

      await tap('Recover Saved Authorization');
      expect(controller.account!['credential_state'], 'pending');
      expect(
        controller.accountReport!['retrieval']['error']['code'],
        'AUTH_REQUIRED',
      );
      expect(core.connectionTargets, isEmpty);
      await tap('Sign In to the Original Account');
      await tester.enterText(
        find.byKey(const ValueKey('credential:Meiju account')),
        'fixture@example.invalid',
      );
      await tester.enterText(
        find.byKey(const ValueKey('credential:Password')),
        'fictional-pass',
      );
      core.credentialState = 'stored';
      core.retrieval = 'complete';
      core.retrievalError = null;
      await tap('Sign In and Retrieve Devices');
      expect(core.connectionTargets, [firstAccount]);
      expect(controller.account!['credential_state'], 'stored');
      expect(controller.cards, hasLength(2));
      expect(core.record(firstAccount)['label'], 'Home account');
      expect(controller.cards.first['preferences']['room_id'], 'study');
      await tester.pumpWidget(const SizedBox());
    },
  );

  test(
    'pending-account version gate, unknown outcomes and interrupted connection require local reload',
    () async {
      final core = AccountFixture(), prefs = AccountPreferences();
      final controller = HomeController(core, prefs);
      core.record(firstAccount)['credential_state'] = 'pending';
      core.supportsPendingReauthorization = false;
      await controller.initialize();
      await controller.selectAccount(firstAccount);
      expect(controller.canSignIn, false);
      core.supportsPendingReauthorization = true;
      expect(controller.canSignIn, true);
      core.connectionError = 'SECRET_STORE_WRITE_UNKNOWN';
      final login = Uint8List.fromList('fictional'.codeUnits),
          password = Uint8List.fromList('fictional'.codeUnits);
      expect(
        await controller.connectAccount(login, password, 'Local label'),
        false,
      );
      expect(controller.accountNeedsReload, true);
      expect(login.every((v) => v == 0), true);
      expect(password.every((v) => v == 0), true);
      expect(await controller.loadAccounts(), true);
      expect(controller.account!['credential_state'], 'pending');
      expect(core.connectionTargets, [firstAccount]);
      expect(core.requests.any((r) => r['method'] == 'recover_account'), false);
      core.retrieval = 'future-success';
      expect(await controller.retrieveAccount(recover: true), false);
      expect((controller.error as NativeFailure).code, 'UNSUPPORTED_OUTCOME');
      expect(controller.accountNeedsReload, true);
    },
  );
}
