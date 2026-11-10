import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/refresh_feedback.dart';
import 'package:homeverb_mobile/screens.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'review_lifecycle_recovery_test.dart' show RecoveryCore;
import 'session_permissions_test.dart' show inspection;

void main() {
  for (final operation in ['cards', 'device', 'receipt']) {
    testWidgets('manual $operation refresh publishes a session-reopen failure', (
      tester,
    ) async {
      final options = inspection();
      final old = RecoveryCore(options);
      final c = HomeController(
        old,
        LocalPreferences.memory(),
        reopen: (_) async {
          throw NativeFailure(
            'CREDENTIAL_STORE_UNAVAILABLE',
            'Fictional vault failure.',
          );
        },
      );
      addTearDown(c.dispose);
      expect(await c.selectDevice(options['device_id']), false);
      c.receipt = {
        'device_id': options['device_id'],
        'request_id': 'fictional-request',
        'status': 'unknown',
      };
      final success = switch (operation) {
        'cards' => await c.refreshCards(),
        'device' => await c.refresh(),
        _ => await c.refreshReceipt(),
      };
      expect(success, false);
      expect(
        c.error,
        isA<NativeFailure>(),
        reason:
            'An explicit refresh failed before its work callback but should still offer actionable error feedback.',
      );
    });
  }
  for (final background in [false, true]) {
    testWidgets(
      'reopen failure after ${background ? "background" : "navigation"} is suppressed',
      (tester) async {
        final options = inspection();
        final gate = Completer<void>();
        var calls = 0;
        final c = HomeController(
          RecoveryCore(options),
          LocalPreferences.memory(),
          reopen: (_) async {
            calls++;
            if (calls > 1) await gate.future;
            throw NativeFailure(
              'CREDENTIAL_STORE_UNAVAILABLE',
              'Fictional vault failure.',
            );
          },
        );
        addTearDown(c.dispose);
        expect(await c.selectDevice(options['device_id']), false);
        final pending = c.refreshCards();
        await tester.pump();
        if (background) {
          c.setForeground(false);
        } else {
          c.cancelPending();
        }
        gate.complete();
        expect(await pending, false);
        expect(c.error, isNull);
      },
    );
  }
  testWidgets('home refresh shows one feedback after session reopen fails', (
    tester,
  ) async {
    final options = inspection();
    final c = HomeController(
      RecoveryCore(options),
      LocalPreferences.memory(),
      reopen: (_) async {
        throw NativeFailure(
          'CREDENTIAL_STORE_UNAVAILABLE',
          'Fictional vault failure.',
        );
      },
    );
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: c),
    );
    await tester.pumpAndSettle();
    expect(await c.selectDevice(options['device_id']), false);
    await app.currentState!.go('D01');
    await tester.pump();
    await app.currentState!.action('Refresh State');
    await tester.pump();
    expect(find.byType(RefreshFeedback), findsOneWidget);
    expect(c.error, isA<NativeFailure>());
    await tester.pumpWidget(const SizedBox());
  });
}
