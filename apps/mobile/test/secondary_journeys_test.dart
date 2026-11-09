import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/skin.dart';

import 'accounts_test.dart' show AccountPreferences;
import 'diagnostics_view_test.dart' show reportFixture;
import 'organization_test.dart' show OrganizationFixture;

class SecondaryFixture extends OrganizationFixture {
  Completer<void>? historyGate, diagnosticsGate;
  bool failHistory = false, failDiagnostics = false;

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'recent_executions' || method == 'device_executions') {
      calls.add({'method': method, ...fields});
      await historyGate?.future;
      if (failHistory) {
        throw NativeFailure('STORAGE_ERROR', 'Fictional history failure.');
      }
      return <Json>[];
    }
    if (method == 'diagnostics') {
      calls.add({'method': method, ...fields});
      await diagnosticsGate?.future;
      if (failDiagnostics) {
        throw NativeFailure('STORAGE_ERROR', 'Fictional diagnostics failure.');
      }
      final report = jsonDecode(jsonEncode(reportFixture())) as Json;
      report['recent_receipts'] = <Json>[];
      return report;
    }
    if (method == 'create_room') {
      calls.add({'method': method, ...fields});
      rooms.add({'id': 'fictional-new-room', 'name': fields['name']});
      return <String, dynamic>{};
    }
    if (method == 'describe_device') {
      calls.add({'method': method, ...fields});
      final card = cards.singleWhere(
        (card) => card['device']['id'] == fields['device'],
      );
      return {'device': card['device'], 'capabilities': <Json>[]};
    }
    if (method == 'read_state') {
      calls.add({'method': method, ...fields});
      throw NativeFailure('STATE_UNAVAILABLE', 'No fictional reading.');
    }
    return super.call(method, fields);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  Future<(SecondaryFixture, HomeController, GlobalKey<NativeAppState>)> open(
    WidgetTester tester,
  ) async {
    tester.view.physicalSize = const Size(780, 1688);
    tester.view.devicePixelRatio = 2;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final core = SecondaryFixture();
    core.cards.first['device']['alias'] = 'Alpha device';
    core.cards.first['preferences']['alias'] = 'Alpha device';
    final c = HomeController(core, AccountPreferences());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: c),
    );
    await tester.pumpAndSettle();
    return (core, c, app);
  }

  for (final history in [true, false]) {
    testWidgets(
      '${history ? 'history' : 'diagnostics'} loading, error and retry remain distinct',
      (tester) async {
        final (core, c, app) = await open(tester);
        final gate = Completer<void>();
        if (history) {
          core.historyGate = gate;
          core.failHistory = true;
        } else {
          core.diagnosticsGate = gate;
          core.failDiagnostics = true;
        }
        final navigation = app.currentState!.go(history ? 'S05' : 'S07');
        await tester.pump();
        expect(c.busy, true);
        expect(find.textContaining('No retained operations'), findsNothing);
        expect(find.text('No report loaded.'), findsNothing);
        expect(find.text('No recent operations'), findsNothing);
        gate.complete();
        await navigation;
        await tester.pumpAndSettle();
        expect(
          history ? c.historyLoadError : c.diagnosticsLoadError,
          isA<NativeFailure>(),
        );
        expect(find.textContaining('No retained operations'), findsNothing);
        expect(find.text('No report loaded.'), findsNothing);
        final retry = find.text(
          history ? 'Retry History' : 'Retry Diagnostics',
        );
        expect(retry, findsOneWidget);
        if (history) {
          core.historyGate = null;
          core.failHistory = false;
        } else {
          core.diagnosticsGate = null;
          core.failDiagnostics = false;
        }
        await tester.ensureVisible(retry);
        await tester.tap(retry);
        await tester.pumpAndSettle();
        expect(history ? c.historyLoadError : c.diagnosticsLoadError, isNull);
        expect(
          history
              ? find.textContaining('No retained operations')
              : find.text('No recent operations'),
          findsOneWidget,
        );
        expect(
          core.calls.where((call) => call['method'] == 'execute'),
          isEmpty,
        );
        expect(
          core.calls.where(
            (call) =>
                call['method'] ==
                (history ? 'recent_executions' : 'diagnostics'),
          ),
          hasLength(2),
        );
        await tester.pumpWidget(const SizedBox());
        c.dispose();
      },
    );
  }

  testWidgets('search and filter context survive opening a card and Back', (
    tester,
  ) async {
    final (core, c, app) = await open(tester);
    await app.currentState!.go('D03');
    await tester.pumpAndSettle();
    await tester.enterText(
      find.byKey(const ValueKey('field:Search devices')),
      'Alpha',
    );
    await tester.pumpAndSettle();
    expect(c.visibleCards, hasLength(1));
    await tester.tap(find.text('Alpha device').first);
    await tester.pumpAndSettle();
    expect(app.currentState!.id, 'C01');
    await tester.tap(find.byTooltip('Back'));
    await tester.pumpAndSettle();
    expect(app.currentState!.id, 'D03');
    expect(c.search, 'Alpha');
    expect(
      tester
          .widget<TextField>(find.byKey(const ValueKey('field:Search devices')))
          .controller!
          .text,
      'Alpha',
    );
    expect(c.visibleCards, hasLength(1));
    expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('empty room and empty search are distinguishable', (
    tester,
  ) async {
    final (core, c, app) = await open(tester);
    core.rooms.add({'id': 'empty-room', 'name': 'Empty Room'});
    expect(await c.initialize(), true);
    c.filter = 'empty-room';
    await app.currentState!.go('D01');
    await tester.pumpAndSettle();
    expect(find.textContaining('No devices in this room'), findsOneWidget);
    c.filter = 'All';
    await app.currentState!.go('D03');
    await tester.pumpAndSettle();
    await tester.enterText(
      find.byKey(const ValueKey('field:Search devices')),
      'no such device',
    );
    await tester.pumpAndSettle();
    expect(find.textContaining('No matching devices'), findsOneWidget);
    expect(find.textContaining('No devices in this room'), findsNothing);
    await tester.tap(find.byTooltip('Clear search'));
    await tester.pumpAndSettle();
    expect(c.search, isEmpty);
    expect(c.visibleCards, isNotEmpty);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('move starts at Unassigned and blocks an empty submission', (
    tester,
  ) async {
    final (core, c, app) = await open(tester);
    core.cards.first['preferences'] = <String, dynamic>{
      ...core.cards.first['preferences'] as Json,
      'room_id': null,
    };
    expect(await c.initialize(), true);
    c.selectedDevice = 'a';
    await app.currentState!.go('D05');
    await app.currentState!.action('Move to Room');
    await tester.pumpAndSettle();
    final tile = find
        .ancestor(
          of: find.text('Unassigned'),
          matching: find.byType(ChoiceTile),
        )
        .first;
    expect(tester.widget<ChoiceTile>(tile).selected, true);
    expect(c.selection, {'a'});
    await tester.tap(find.byKey(const ValueKey('select-device:a')));
    await tester.pumpAndSettle();
    expect(c.selection, isEmpty);
    final move = find.byWidgetPredicate(
      (widget) => widget is ActionButton && widget.label == 'Move Devices',
    );
    expect(tester.widget<ActionButton>(move).onPressed, isNull);
    expect(
      core.calls.where((call) => call['method'] == 'assign_devices_to_room'),
      isEmpty,
    );
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('device route choices reflect admitted transports', (
    tester,
  ) async {
    final (core, c, app) = await open(tester);
    for (final transports in [
      <String>[],
      ['lan'],
      ['cloud'],
      ['lan', 'cloud'],
    ]) {
      core.cards.first['device'] = <String, dynamic>{
        ...core.cards.first['device'] as Json,
        'connections': [
          for (final transport in transports)
            {'id': 'fictional-$transport', 'transport': transport},
        ],
      };
      expect(await c.initialize(), true);
      c.selectedDevice = 'a';
      await app.currentState!.go('D08');
      await tester.pumpAndSettle();
      expect(find.text('Auto — prefer local when available'), findsOneWidget);
      expect(
        find.text('Local network only — no cloud fallback'),
        transports.contains('lan') ? findsOneWidget : findsNothing,
      );
      expect(
        find.text('Platform cloud — requires internet'),
        transports.contains('cloud') ? findsOneWidget : findsNothing,
      );
      if (transports.length == 1 && transports.single == 'cloud') {
        await tester.tap(find.text('Platform cloud — requires internet'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Save Preference'));
        await tester.pumpAndSettle();
        expect(c.preferences.route('a'), 'cloud');
        expect(app.currentState!.id, 'D06');
      }
    }
    expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('adding a room returns to the move without assigning devices', (
    tester,
  ) async {
    final (core, c, app) = await open(tester);
    await app.currentState!.go('R03');
    await tester.pumpAndSettle();
    for (final id in ['a', 'x']) {
      final card = find.byKey(ValueKey('select-device:$id'));
      await tester.ensureVisible(card);
      await tester.tap(card);
      await tester.pumpAndSettle();
    }
    expect(c.selection, {'a', 'x'});
    final add = find.text('Add Room').last;
    await tester.ensureVisible(add);
    await tester.tap(add);
    await tester.pumpAndSettle();
    expect(app.currentState!.id, 'R02');
    await tester.enterText(
      find.byKey(const ValueKey('field:Room name')),
      'Studio',
    );
    await tester.tap(find.text('Save Room'));
    await tester.pumpAndSettle();
    expect(app.currentState!.id, 'R03');
    expect(c.selection, {'a', 'x'});
    expect(c.selectedRoom, 'fictional-new-room');
    final selected = find
        .ancestor(of: find.text('Studio'), matching: find.byType(ChoiceTile))
        .first;
    expect(tester.widget<ChoiceTile>(selected).selected, true);
    expect(
      core.calls.where((call) => call['method'] == 'assign_devices_to_room'),
      isEmpty,
    );
    final move = find.text('Move Devices');
    await tester.ensureVisible(move);
    await tester.tap(move);
    await tester.pumpAndSettle();
    final assignments = core.calls.where(
      (call) => call['method'] == 'assign_devices_to_room',
    );
    expect(assignments, hasLength(1));
    final assignment = assignments.single['assignment'] as Json;
    expect((assignment['device_ids'] as List).toSet(), {'a', 'x'});
    expect(assignment['room_id'], 'fictional-new-room');
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('settings hide unsupported sharing and retained rows navigate', (
    tester,
  ) async {
    final (core, c, app) = await open(tester);
    const destinations = {
      'Language': 'L01',
      'Rooms': 'R01',
      'Appearance': 'S02',
      'Connections': 'S03',
      'Recent Operations': 'S05',
      'Local Data': 'S04',
      'Diagnostics': 'S07',
      'About': 'S08',
    };
    for (final entry in destinations.entries) {
      await app.currentState!.go('S01');
      await tester.pumpAndSettle();
      expect(find.text('Send Authorization'), findsNothing);
      expect(find.text('Receive Authorization'), findsNothing);
      expect(find.text('This Device'), findsNothing);
      final row = find.text(entry.key).last;
      await tester.ensureVisible(row);
      await tester.tap(row);
      await tester.pumpAndSettle();
      expect(app.currentState!.id, entry.value, reason: entry.key);
    }
    expect(app.currentState!.id, 'S08');
    await tester.tap(find.text('Privacy'));
    await tester.pumpAndSettle();
    expect(
      find.textContaining('No telemetry or diagnostic uploads'),
      findsOneWidget,
    );
    await tester.tap(find.text('Close'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Licenses'));
    await tester.pumpAndSettle();
    expect(find.textContaining('HomeVerb'), findsWidgets);
    expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
}
