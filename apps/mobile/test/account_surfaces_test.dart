import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/account_surfaces.dart';
import 'package:homeverb_mobile/account_views.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/skin.dart';

import 'accounts_test.dart'
    show AccountFixture, AccountPreferences, firstAccount;

void main() {
  Future<HomeController> controller() async {
    final c = HomeController(AccountFixture(), AccountPreferences());
    await c.initialize();
    return c;
  }

  Widget page(Widget child, {String skin = 'sky_blue', double scale = 1}) =>
      MaterialApp(
        theme: windowTheme(skinId: skin),
        home: MediaQuery(
          data: MediaQueryData(
            size: const Size(320, 844),
            textScaler: TextScaler.linear(scale),
          ),
          child: Scaffold(
            body: SingleChildScrollView(
              child: Padding(padding: const EdgeInsets.all(20), child: child),
            ),
          ),
        ),
      );
  testWidgets(
    'picker separates connecting from viewing and does not connect itself',
    (tester) async {
      final c = await controller();
      addTearDown(c.dispose);
      final actions = <String>[];
      await tester.pumpWidget(
        page(PlatformPicker(controller: c, onAction: actions.add)),
      );
      await tester.tap(find.byKey(const ValueKey('platform:meiju')));
      expect(actions, ['Choose Meiju']);
      await tester.ensureVisible(find.text('View linked accounts'));
      await tester.tap(find.text('View linked accounts'));
      expect(actions, ['Choose Meiju', 'Back to Accounts']);
      expect((c.core as AccountFixture).connectionTargets, isEmpty);
      expect(find.text('Xiaomi'), findsNothing);
      expect(find.text('Hisense'), findsNothing);
    },
  );
  testWidgets(
    'details shows known local association and safe explicit actions',
    (tester) async {
      final c = await controller();
      addTearDown(c.dispose);
      await c.selectAccount(firstAccount);
      final actions = <String>[];
      await tester.pumpWidget(
        page(AccountDetails(controller: c, onAction: actions.add)),
      );
      expect(find.text('1 devices associated locally'), findsOneWidget);
      expect(find.text('Saved on this device'), findsOneWidget);
      expect(
        find.text(
          'Saved locally does not mean currently online or still authorized.',
        ),
        findsOneWidget,
      );
      await tester.ensureVisible(find.text('Refresh Devices'));
      await tester.tap(find.text('Refresh Devices'));
      expect(actions, ['Refresh Devices']);
      expect(
        (c.core as AccountFixture).requests.where(
          (r) => [
            'refresh_account',
            'read_state',
            'execute',
          ].contains(r['method']),
        ),
        isEmpty,
      );
    },
  );
  testWidgets(
    'pending cleanup keeps the original recovery explanation and actions',
    (tester) async {
      final c = await controller();
      addTearDown(c.dispose);
      final core = c.core as AccountFixture;
      core.record(firstAccount)['credential_state'] = 'removing';
      await c.selectAccount(firstAccount);
      await tester.pumpWidget(
        page(AccountDetails(controller: c, onAction: (_) {})),
      );
      expect(
        find.text('Finish removing this account from this device.'),
        findsOneWidget,
      );
      expect(find.text('Retry local cleanup'), findsOneWidget);
      expect(find.text('Reauthorize'), findsNothing);
    },
  );
  for (final skin in ['window_light', 'sky_blue']) {
    testWidgets('account surfaces fit narrow double-text $skin', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(320, 844);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final c = await controller();
      addTearDown(c.dispose);
      await c.selectAccount(firstAccount);
      for (final child in [
        PlatformAccounts(controller: c, onAction: (_) {}),
        PlatformPicker(controller: c, onAction: (_) {}),
        AccountDetails(controller: c, onAction: (_) {}),
      ]) {
        await tester.pumpWidget(page(child, skin: skin, scale: 2));
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
      }
    });
  }
  testWidgets('busy account details and picker cannot dispatch actions', (
    tester,
  ) async {
    final c = await controller();
    addTearDown(c.dispose);
    await c.selectAccount(firstAccount);
    c.busy = true;
    final actions = <String>[];
    await tester.pumpWidget(
      page(PlatformPicker(controller: c, onAction: actions.add)),
    );
    expect(
      tester
          .widget<OutlinedButton>(find.byKey(const ValueKey('platform:meiju')))
          .onPressed,
      isNull,
    );
    await tester.pumpWidget(
      page(AccountDetails(controller: c, onAction: actions.add)),
    );
    final refresh = tester.widget<ActionButton>(
      find.byWidgetPredicate(
        (w) => w is ActionButton && w.label == 'Refresh Devices',
      ),
    );
    expect(refresh.onPressed, isNull);
    expect(actions, isEmpty);
  });
}
