import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/controls.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/skin.dart';

import 'accounts_test.dart' show AccountPreferences;
import 'controls_test.dart' show ControlsFixture;

class HeldGenericCommand extends ControlsFixture {
  HeldGenericCommand(this.status);
  final String status;
  final gate = Completer<void>();
  bool failReadback = false;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'execute') {
      calls.add({'method': method, ...fields});
      final command = fields['command'] as Json;
      await gate.future;
      return {
        'request_id': command['request_id'],
        'device_id': command['device_id'],
        'status': status,
        'verification': null,
        'attempts': <Json>[],
        'data': null,
      };
    }
    if (method == 'read_state' && fields['cached'] == false && failReadback) {
      calls.add({'method': method, ...fields});
      throw NativeFailure('NETWORK_UNAVAILABLE', 'Fictional readback failure.');
    }
    return super.call(method, fields);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  for (final status in ['accepted', 'unknown', 'failed']) {
    testWidgets('generic form keeps $status and offers receipt recovery', (
      tester,
    ) async {
      final core = HeldGenericCommand(status);
      final controller = HomeController(core, AccountPreferences());
      final app = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: app, catalog: catalog, controller: controller),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Controls fixture').first);
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('control:0:set')));
      await tester.pumpAndSettle();
      expect(app.currentState!.id, 'C10');
      await tester.enterText(
        find.byKey(const ValueKey('parameter:brightness')),
        '40',
      );
      final apply = find.text('Apply');
      await tester.ensureVisible(apply);
      core.failReadback = true;
      await tester.tap(apply);
      await tester.pump();
      expect(controller.observed('light.brightness'), 35);
      expect(
        find.descendant(
          of: find.byType(CapabilityActionForm),
          matching: find.byType(OperationPulse),
        ),
        findsOneWidget,
      );
      expect(
        tester
            .widget<FilledButton>(find.widgetWithText(FilledButton, 'Apply'))
            .onPressed,
        isNull,
      );
      expect(
        core.calls.where((call) => call['method'] == 'execute'),
        hasLength(1),
      );
      core.gate.complete();
      await tester.pumpAndSettle();
      expect(app.currentState!.id, 'C01');
      expect(controller.receipt!['status'], status);
      expect(controller.observed('light.brightness'), 35);
      final request = controller.receipt!['request_id'];
      final details = find.text('View Details');
      expect(details, findsOneWidget);
      await tester.ensureVisible(details);
      await tester.tap(details);
      await tester.pumpAndSettle();
      expect(app.currentState!.id, 'S06');
      core.failReadback = false;
      await tester.ensureVisible(find.text('Refresh State'));
      await tester.tap(find.text('Refresh State'));
      await tester.pumpAndSettle();
      expect(controller.receipt!['request_id'], request);
      expect(controller.receipt!['status'], status);
      expect(
        core.calls.where((call) => call['method'] == 'execute'),
        hasLength(1),
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    });
  }
}
