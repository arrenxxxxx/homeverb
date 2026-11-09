import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/native_client.dart';

import 'accounts_test.dart' show AccountPreferences;
import 'controls_test.dart' show ControlsFixture, capability, setter;

class HeldPlan extends ControlsFixture {
  final planGate = Completer<void>();
  final planStarted = Completer<void>();
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'plan') {
      calls.add({'method': method, ...fields});
      planStarted.complete();
      await planGate.future;
      return fields['command'];
    }
    return super.call(method, fields);
  }
}

class HeldBatchPlan extends ControlsFixture {
  HeldBatchPlan() {
    capabilities
      ..clear()
      ..add(
        capability(
          'power',
          {
            'state': {'type': 'boolean', 'readable': true, 'writable': true},
          },
          {
            'set': setter(
              {
                'state': {'type': 'boolean'},
              },
              required: ['state'],
            ),
          },
        ),
      );
  }
  final planGate = Completer<void>();
  final planStarted = Completer<void>();
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'plan_batch') {
      calls.add({'method': method, ...fields});
      planStarted.complete();
      await planGate.future;
      return fields['batch'];
    }
    return super.call(method, fields);
  }
}

class HeldExecution extends ControlsFixture {
  final reply = Completer<void>();
  final started = Completer<void>();
  final journal = <String, Json>{};
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'execute') {
      calls.add({'method': method, ...fields});
      final command = fields['command'] as Json;
      final receipt = <String, dynamic>{
        'request_id': command['request_id'],
        'device_id': command['device_id'],
        'status': 'unknown',
      };
      journal[command['request_id'] as String] = receipt;
      started.complete();
      await reply.future;
      return receipt;
    }
    if (method == 'get_execution') {
      calls.add({'method': method, ...fields});
      return journal[fields['request_id']]!;
    }
    return super.call(method, fields);
  }
}

class FictionalPreview implements ApprovalPreview {
  FictionalPreview(this.command);
  @override
  final Json command;
  @override
  Json get descriptor => {};
  @override
  Json get action => {};
  @override
  Json get connection => {};
  @override
  DateTime get expiresAt => DateTime.now().add(const Duration(minutes: 1));
  @override
  Duration get remaining => const Duration(minutes: 1);
}

class HeldApproval extends ControlsFixture implements ApprovalClient {
  HeldApproval() {
    capabilities.add(
      capability('vendor.fixture.cycle', {}, {
        'start': setter({}, risk: 'elevated', idempotency: 'non_idempotent'),
      }),
    );
  }
  final reply = Completer<void>();
  final started = Completer<void>();
  int sends = 0;
  bool failReply = false;
  @override
  bool get supportsTrustedApproval => true;
  @override
  Future<ApprovalPreview> prepareApproval(Json command) async =>
      FictionalPreview(command);
  @override
  Future<Json> executeApproved(ApprovalPreview preview) async {
    sends++;
    started.complete();
    await reply.future;
    if (failReply) throw NativeFailure('NATIVE_ERROR', 'Fictional lost reply.');
    return {
      'request_id': preview.command['request_id'],
      'device_id': preview.command['device_id'],
      'status': 'accepted',
    };
  }

  @override
  Future<void> discardApproval(ApprovalPreview preview) async {}
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  for (final interrupt in ['navigation', 'background-resume']) {
    testWidgets(
      'held write result is recovered without replay after $interrupt',
      (tester) async {
        final core = HeldExecution();
        final c = HomeController(core, AccountPreferences());
        await c.initialize();
        expect(await c.selectDevice(core.device['id']), true);
        final work = c.execute('light', 'set', {'brightness': 40});
        await core.started.future;
        final request = core.journal.keys.single;
        if (interrupt == 'navigation') {
          c.cancelPending();
        } else {
          c.setForeground(false);
          c.setForeground(true);
        }
        final otherReceipt = <String, dynamic>{
          'device_id': 'other-device',
          'request_id': 'other-request',
          'status': 'unknown',
        };
        c.receipt = otherReceipt;
        core.reply.complete();
        expect(await work, false);
        expect(c.receipt, same(otherReceipt));
        expect(c.error, isNull);
        expect(
          core.calls.where((call) => call['method'] == 'execute'),
          hasLength(1),
        );
        expect(await c.loadReceipt(request), true);
        expect(c.receipt!['request_id'], request);
        expect(c.receipt!['status'], 'unknown');
        expect(
          core.calls.where((call) => call['method'] == 'execute'),
          hasLength(1),
        );
        expect(
          core.calls.where((call) => call['method'] == 'get_execution'),
          hasLength(1),
        );
        c.dispose();
      },
    );

    testWidgets('held batch plan never sends after $interrupt', (tester) async {
      final core = HeldBatchPlan();
      final c = HomeController(core, AccountPreferences());
      await c.initialize();
      c.selection.add(core.device['id'] as String);
      final work = c.executeSelectedPower(true);
      await core.planStarted.future;
      if (interrupt == 'navigation') {
        c.cancelPending();
      } else {
        c.setForeground(false);
        c.setForeground(true);
      }
      core.planGate.complete();
      expect(await work, false);
      expect(
        core.calls.where((call) => call['method'] == 'execute_batch'),
        isEmpty,
      );
      expect(c.batchResult, isNull);
      expect(c.error, isNull);
      c.dispose();
    });

    testWidgets('held plan never sends after $interrupt', (tester) async {
      final core = HeldPlan();
      final c = HomeController(core, AccountPreferences());
      await c.initialize();
      expect(await c.selectDevice(core.device['id']), true);
      final work = c.execute('light', 'set', {'brightness': 40});
      await core.planStarted.future;
      if (interrupt == 'navigation') {
        c.cancelPending();
      } else {
        c.setForeground(false);
        c.setForeground(true);
      }
      core.planGate.complete();
      expect(await work, false);
      expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
      expect(c.receipt, isNull);
      expect(c.error, isNull);
      c.dispose();
    });
  }

  for (final interrupt in ['navigation', 'background-resume']) {
    for (final failReply in [false, true]) {
      testWidgets(
        'held trusted reply is fenced after $interrupt, failure=$failReply',
        (tester) async {
          final core = HeldApproval()..failReply = failReply;
          final c = HomeController(core, AccountPreferences());
          await c.initialize();
          expect(await c.selectDevice(core.device['id']), true);
          expect(
            await c.prepareApproval('vendor.fixture.cycle', 'start', {}, 'lan'),
            true,
          );
          final request = c.approvalRequestId;
          expect(request, isNotNull);
          final work = c.confirmApproval();
          await core.started.future;
          expect(c.approvalPreview, isNull);
          if (interrupt == 'navigation') {
            c.cancelPending();
          } else {
            c.setForeground(false);
            c.setForeground(true);
          }
          final otherReceipt = <String, dynamic>{
            'device_id': 'other-device',
            'request_id': 'other-request',
            'status': 'unknown',
          };
          c.receipt = otherReceipt;
          core.reply.complete();
          expect(await work, false);
          expect(c.receipt, same(otherReceipt));
          expect(c.error, isNull);
          expect(c.approvalPreview, isNull);
          expect(c.approvalRequestId, request);
          expect(c.approvalNeedsReceipt, true);
          expect(core.sends, 1);
          c.dispose();
        },
      );
    }
  }
}
