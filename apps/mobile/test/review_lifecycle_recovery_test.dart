import 'dart:async';
import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:flutter/material.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'accounts_test.dart' show AccountPreferences;
import 'device_opening_test.dart' show OrderedPermissionCore;
import 'component_test.dart' show example;
import 'diagnostics_view_test.dart' show reportFixture;
import 'secondary_journeys_test.dart' show SecondaryFixture;
import 'session_permissions_test.dart' show PermissionCore, inspection;

class RecoveryCore extends PermissionCore {
  RecoveryCore(super.inspected);
  bool failLive = false;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (closed) throw StateError('Closed fictional session used.');
    if (method == 'recent_executions') return <Json>[];
    if (method == 'diagnostics') return reportFixture();
    if (method == 'read_state') {
      if (failLive && fields['cached'] == false) {
        throw NativeFailure('NETWORK_UNAVAILABLE', 'Fictional read failure.');
      }
      return {
        ...example('state'),
        'device_id': fields['device'],
        'component_id': fields['component'],
        'observations': {
          'power.state': {'value': true},
        },
      };
    }
    if (method == 'get_execution') {
      return {
        'request_id': fields['request_id'],
        'device_id': inspected['device_id'],
        'status': 'unknown',
      };
    }
    final result = await super.call(method, fields);
    return method == 'describe_device'
        ? jsonDecode(jsonEncode(result))
        : result;
  }
}

class HeldRecoveryCore extends RecoveryCore {
  HeldRecoveryCore(super.inspected);
  final gate = Completer<void>();
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'read_state' && fields['cached'] == false) await gate.future;
    return super.call(method, fields);
  }
}

Future<void> frames(WidgetTester tester, [int count = 12]) async {
  for (var i = 0; i < count; i++) await tester.pump();
}

void main() {
  testWidgets('failed foreground control recovery offers an explicit retry', (
    tester,
  ) async {
    final options = inspection();
    final old = HeldRecoveryCore(options);
    var opens = 0;
    final c = HomeController(
      old,
      AccountPreferences(),
      reopen: (_) async {
        opens++;
        if (opens == 1)
          throw NativeFailure('STATE_UNAVAILABLE', 'Fictional reopen failure.');
        return RecoveryCore(options);
      },
    );
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: c),
    );
    await tester.pumpAndSettle();
    await app.currentState!.go('C01');
    final work = c.selectDevice(options['device_id']);
    await frames(tester);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    old.gate.complete();
    await work;
    await frames(tester, 40);
    final retry = find.byKey(const ValueKey('device-control-retry'));
    expect(retry, findsOneWidget);
    expect(
      find.text('No controls are allowed for this device on this connection.'),
      findsNothing,
    );
    await tester.tap(retry);
    await frames(tester, 80);
    expect(c.capability('power')?['actions'], contains('set'));
    expect(find.byKey(const ValueKey('device-control-retry')), findsNothing);
    await tester.pumpWidget(const SizedBox());
  });
  testWidgets('interrupted opening restores controls once on foreground', (
    tester,
  ) async {
    final options = inspection();
    final old = OrderedPermissionCore(options);
    final next = PermissionCore(options);
    final c = HomeController(
      old,
      AccountPreferences(),
      reopen: (_) async => next,
    );
    expect(await c.initialize(), true);
    old.requests.clear();
    c.setAutomaticRefreshView(StateRefreshView.device);
    final opening = c.selectDevice(options['device_id']);
    await frames(tester);
    c.setForeground(false);
    c.setForeground(true);
    old.liveGate.complete();
    await opening;
    await frames(tester, 40);
    expect(c.capability('power')?['actions'], contains('set'));
    expect(
      old.requests.where((r) => r['method'] == 'inspect_control_options'),
      hasLength(1),
    );
    c.dispose();
  });

  for (final history in [true, false]) {
    testWidgets(
      '${history ? 'history' : 'diagnostics'} remains retryable when prior work drains',
      (tester) async {
        final core = SecondaryFixture();
        final c = HomeController(core, AccountPreferences());
        final gate = Completer<void>();
        final previous = c.perform(() => gate.future);
        await frames(tester);
        c.cancelPending();
        expect(await (history ? c.loadHistory() : c.loadDiagnostics()), false);
        expect(
          history ? c.historyState : c.diagnosticsState,
          LocalLoadState.failed,
        );
        gate.complete();
        await previous;
        expect(await (history ? c.loadHistory() : c.loadDiagnostics()), true);
        expect(
          history ? c.historyState : c.diagnosticsState,
          LocalLoadState.ready,
        );
        c.dispose();
      },
    );
    testWidgets(
      '${history ? 'history' : 'diagnostics'} reports session reopen failure',
      (tester) async {
        final options = inspection();
        final old = RecoveryCore(options);
        final c = HomeController(
          old,
          AccountPreferences(),
          reopen: (_) async => throw NativeFailure(
            'STATE_UNAVAILABLE',
            'Fictional reopen failure.',
          ),
        );
        expect(await c.selectDevice(options['device_id']), false);
        expect(await (history ? c.loadHistory() : c.loadDiagnostics()), false);
        expect(
          history ? c.historyState : c.diagnosticsState,
          LocalLoadState.failed,
        );
        expect(
          history ? c.historyLoadError : c.diagnosticsLoadError,
          isA<NativeFailure>(),
        );
        c.dispose();
      },
    );
    testWidgets(
      '${history ? 'history' : 'diagnostics'} becomes retryable after background',
      (tester) async {
        final core = SecondaryFixture();
        final c = HomeController(core, AccountPreferences());
        final gate = Completer<void>();
        if (history) {
          core.historyGate = gate;
        } else {
          core.diagnosticsGate = gate;
        }
        final work = history ? c.loadHistory() : c.loadDiagnostics();
        await frames(tester);
        c.setForeground(false);
        c.setForeground(true);
        gate.complete();
        expect(await work, false);
        await frames(tester);
        expect(
          history ? c.historyState : c.diagnosticsState,
          LocalLoadState.failed,
        );
        if (history) {
          core.historyGate = null;
        } else {
          core.diagnosticsGate = null;
        }
        expect(await (history ? c.loadHistory() : c.loadDiagnostics()), true);
        expect(
          history ? c.historyState : c.diagnosticsState,
          LocalLoadState.ready,
        );
        c.dispose();
      },
    );

    testWidgets(
      '${history ? 'history' : 'diagnostics'} binds the reopened session',
      (tester) async {
        final options = inspection();
        final old = RecoveryCore(options),
            late = RecoveryCore(options),
            fresh = RecoveryCore(options);
        final gate = Completer<CoreClient>();
        var opens = 0;
        final c = HomeController(
          old,
          AccountPreferences(),
          reopen: (_) => ++opens == 1 ? gate.future : Future.value(fresh),
        );
        final opening = c.selectDevice(options['device_id']);
        await frames(tester);
        expect(opens, 1);
        c.setForeground(false);
        gate.complete(late);
        expect(await opening, false);
        c.setForeground(true);
        expect(await (history ? c.loadHistory() : c.loadDiagnostics()), true);
        expect(
          history ? c.historyState : c.diagnosticsState,
          LocalLoadState.ready,
        );
        expect(c.core, same(fresh));
        c.dispose();
      },
    );
  }

  for (final operation in [
    'cards',
    'components',
    'device',
    'receipt-refresh',
    'receipt-lookup',
  ]) {
    testWidgets('$operation manual work binds the reopened session', (
      tester,
    ) async {
      final options = inspection();
      final old = RecoveryCore(options),
          late = RecoveryCore(options),
          fresh = RecoveryCore(options);
      final gate = Completer<CoreClient>();
      var opens = 0;
      final c = HomeController(
        old,
        AccountPreferences(),
        reopen: (_) => ++opens == 1 ? gate.future : Future.value(fresh),
      );
      final opening = c.selectDevice(options['device_id']);
      await frames(tester);
      expect(opens, 1);
      c.setForeground(false);
      gate.complete(late);
      expect(await opening, false);
      c.setForeground(true);
      c.receipt = {
        'request_id': 'fictional-receipt',
        'device_id': options['device_id'],
        'status': 'unknown',
      };
      if (operation == 'components' || operation == 'device')
        fresh.failLive = true;
      final result = switch (operation) {
        'cards' => await c.refreshCards(),
        'components' => await c.refreshComponents(),
        'device' => await c.refresh(),
        'receipt-refresh' => await c.refreshReceipt(),
        _ => await c.loadReceipt('fictional-receipt'),
      };
      if (fresh.failLive) {
        expect(result, false);
        expect((c.error as NativeFailure).code, 'NETWORK_UNAVAILABLE');
      } else {
        expect(result, true);
      }
      expect(c.core, same(fresh));
      c.dispose();
    });
  }
}
