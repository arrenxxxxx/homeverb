import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/client_contract.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/event_history_view.dart';
import 'package:homeverb_mobile/event_history.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'accounts_test.dart' show AccountPreferences;
import 'controls_test.dart' show ControlsFixture, example, capability;

Json copy(Json value) => jsonDecode(jsonEncode(value)) as Json;
Json descriptor() => {
  'device': {
    'id': 'fictional-event-source',
    'components': [
      {
        'id': 'event-sensor',
        'capabilities': ['vendor.example.sensor'],
      },
    ],
    'connections': [
      {'id': 'fictional-cloud', 'transport': 'cloud'},
    ],
  },
  'capabilities': [
    {
      ...capability('vendor.example.sensor', {}, {}),
      'events': {
        'reported_activity': {
          'history': {'projection': 'occurrence_times'},
        },
      },
    },
  ],
};
EventHistoryOption option([Json? raw, Json? desc]) => EventHistoryOption.parse(
  raw ?? example('event-history.options'),
  desc ?? descriptor(),
  'fictional-cloud',
).single;
final started = DateTime.utc(2026, 10, 4, 12);
final finished = started.add(const Duration(seconds: 1));
EventHistoryQuery query() =>
    option().query('2026-10-01T00:00:00Z', '2026-10-03T00:00:00Z', started);

class HistoryFixture extends ControlsFixture implements NativeSessionClient {
  HistoryFixture() {
    capabilities
      ..clear()
      ..addAll((descriptor()['capabilities'] as List).cast<Json>());
    device.addAll(descriptor()['device'] as Json);
  }
  int minor = 44, cancelled = 0;
  bool empty = false, malformed = false;
  Completer<void>? gate, inspectionGate;
  @override
  Set<String> methods = {'inspect_event_history_options', 'read_event_history'};
  @override
  bool apiAtLeast(int value) => minor >= value;
  @override
  void cancelAll() {
    cancelled++;
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'inspect_event_history_options') {
      calls.add({'method': method, ...fields});
      await inspectionGate?.future;
      final result = example('event-history.options');
      result['connection_id'] = fields['connection_id'];
      if (empty) result['options'] = [];
      return result;
    }
    if (method == 'read_event_history') {
      calls.add({'method': method, ...fields});
      await gate?.future;
      final request = fields['history'] as Json;
      return {
        ...example('event-history.result'),
        ...request,
        'received_at': DateTime.now().toUtc().toIso8601String(),
        'source': malformed ? 'lan' : 'cloud',
        'records': [
          {'reported_at': request['start']},
          {'reported_at': request['start']},
        ],
      };
    }
    return super.call(method, fields);
  }
}

Future<HomeController> showHistory(
  WidgetTester tester,
  HistoryFixture core, {
  double scale = 1,
}) async {
  final c = HomeController(core, AccountPreferences());
  c.selectedDevice = core.device['id'];
  c.description = {'device': core.device, 'capabilities': core.capabilities};
  await tester.pumpWidget(
    TranslationProvider(
      child: MaterialApp(
        builder: (context, child) => MediaQuery(
          data: MediaQuery.of(
            context,
          ).copyWith(textScaler: TextScaler.linear(scale)),
          child: child!,
        ),
        home: Builder(
          builder: (context) => Scaffold(
            body: TextButton(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => EventHistoryDialog(controller: c),
              ),
              child: const Text('Open fixture'),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('Open fixture'));
  await tester.pumpAndSettle();
  return c;
}

Future<void> readHistory(WidgetTester tester, {bool settle = true}) async {
  final button = find.byKey(const ValueKey('event-history-read'));
  await tester.ensureVisible(button);
  await tester.tap(button);
  if (settle) {
    await tester.pumpAndSettle();
  } else {
    await tester.pump();
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async {
    await (FontLoader(
      'Inter',
    )..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))).load();
  });
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));

  test('event-only options own targets and preserve duplicate timestamps', () {
    final raw = example('event-history.options');
    final parsed = option(raw);
    raw['options'][0]['target']['event'] = 'changed';
    expect(parsed.target['event'], 'reported_activity');
    expect(() => parsed.target['event'] = 'changed', throwsUnsupportedError);
    expect(query().json, example('event-history.request'));
    final result = EventHistoryResult.parse(
      example('event-history.result'),
      query(),
      started,
      finished,
    );
    expect(result.records.length, 2);
    expect(result.records[0], result.records[1]);
    expect(() => result.records.clear(), throwsUnsupportedError);
  });
  test('event query requires exact past bounded whole-second UTC window', () {
    for (final range in [
      ('invalid', '2026-10-03T00:00:00Z'),
      ('2026-02-30T00:00:00Z', '2026-10-03T00:00:00Z'),
      ('2026-10-01T00:00:00.0Z', '2026-10-03T00:00:00Z'),
      ('2026-10-01T00:00:00+00:00', '2026-10-03T00:00:00Z'),
      ('2026-10-03T00:00:00Z', '2026-10-03T00:00:00Z'),
      ('2026-10-03T00:00:00Z', '2026-10-01T00:00:00Z'),
      ('2026-08-01T00:00:00Z', '2026-10-03T00:00:00Z'),
      ('2026-10-01T00:00:00Z', '2027-10-03T00:00:00Z'),
      ('1970-01-01T00:00:00Z', '1970-01-02T00:00:00Z'),
    ]) {
      expect(
        () => option().query(range.$1, range.$2, started),
        throwsA(isA<NativeFailure>()),
      );
    }
  });
  test(
    'options reject foreign bindings, invented events and remote payload fields',
    () {
      for (final mutate in <void Function(Json)>[
        (v) => v['device_id'] = 'foreign',
        (v) => v['connection_id'] = 'foreign',
        (v) => v['payload'] = 'PRIVATE',
        (v) => v['options'][0]['projection'] = 'payload',
        (v) => v['options'][0]['target']['event'] = 'unknown',
        (v) => v['options'][0]['max_records'] = 32,
        (v) => v['options'][0]['max_window_seconds'] = 2764801,
        (v) => v['options'].add(copy(v['options'][0])),
      ]) {
        final raw = example('event-history.options');
        mutate(raw);
        expect(() => option(raw), throwsA(isA<NativeFailure>()));
      }
      for (final mutate in <void Function(Json)>[
        (v) => v['capabilities'][0]['events'] = {},
        (v) =>
            v['capabilities'][0]['events']['reported_activity']['history']['projection'] =
                'payload',
        (v) => v['capabilities'][0]['component_id'] = 'foreign',
        (v) => v['device']['components'][0]['capabilities'] = [],
      ]) {
        final desc = descriptor();
        mutate(desc);
        expect(() => option(null, desc), throwsA(isA<NativeFailure>()));
      }
    },
  );
  test(
    'results refuse payloads, changed selection, inconsistent counts and stale receipts',
    () {
      for (final mutate in <void Function(Json)>[
        (v) => v['device_id'] = 'foreign',
        (v) => v['connection_id'] = 'foreign',
        (v) => v['source'] = 'lan',
        (v) => v['projection'] = 'payload',
        (v) => v['target']['event'] = 'other',
        (v) => v['cursor'] = 'PRIVATE',
        (v) => v['completeness'] = 'complete',
        (v) => v['limit_reached'] = true,
        (v) => v['rejected_records'] = 30,
        (v) => v['received_at'] = '2026-10-03T00:00:00Z',
        (v) => v['received_at'] = '2026-10-05T00:00:00Z',
        (v) => v['records'][0]['value'] = 'PRIVATE',
        (v) => v['records'][0]['actor'] = 'PRIVATE',
        (v) => v['records'][0]['reported_at'] = '2026-10-01T00:00:00.1Z',
        (v) => v['records'][0]['reported_at'] = '2026-09-30T00:00:00Z',
        (v) => v['records'][1]['reported_at'] = '2026-10-03T00:00:00Z',
        (v) => v['records'][0]['reported_at'] = '2026-10-02T00:00:00Z',
      ]) {
        final raw = example('event-history.result');
        mutate(raw);
        expect(
          () => EventHistoryResult.parse(raw, query(), started, finished),
          throwsA(isA<NativeFailure>()),
        );
      }
    },
  );
  test('all 78 pinned event selectors have scoped options and shared labels', () {
    final events =
        jsonDecode(
              File(
                '../../crates/mijia/src/event_profiles.json',
              ).readAsStringSync(),
            )
            as List;
    final english =
        jsonDecode(
              File(
                '../../packages/presentation/locales/en-US.json',
              ).readAsStringSync(),
            )
            as Json;
    final chinese =
        jsonDecode(
              File(
                '../../packages/presentation/locales/zh-CN.json',
              ).readAsStringSync(),
            )
            as Json;
    expect(events.length, 78);
    for (final event in events) {
      final service = (event['service_type'] as String).split(':');
      final name = (event['event_type'] as String)
          .split(':')[3]
          .replaceAll('-', '_');
      final part = '${service[3].replaceAll('-', '_')}-${event['siid']}';
      final cap =
          'vendor.xiaomi.mijia.event.${service[1].replaceAll('-', '_')}.${service[3].replaceAll('-', '_')}';
      final eventId = '${name}_${event['eiid']}';
      final desc = descriptor();
      desc['device']['components'] = [
        {
          'id': part,
          'capabilities': [cap],
        },
      ];
      desc['capabilities'] = [
        {
          ...capability(cap, {}, {}),
          'component_id': part,
          'events': {
            eventId: {
              'history': {'projection': 'occurrence_times'},
            },
          },
        },
      ];
      final raw = example('event-history.options');
      raw['options'][0]['target'] = {
        'component_id': part,
        'capability': cap,
        'event': eventId,
      };
      expect(option(raw, desc).target, raw['options'][0]['target']);
      final key = '$cap.$eventId';
      expect(sourceForKey(key), english[key]);
      expect(english[key], isA<String>());
      expect(chinese[key], isA<String>());
      expect(chinese[key], isNot(english[key]));
    }
  });
  test('empty and full rejected pages retain limits without invented rows', () {
    for (final count in [0, 31]) {
      final raw = example('event-history.result');
      raw['records'] = [];
      raw['rejected_records'] = count;
      raw['limit_reached'] = count == 31;
      final result = EventHistoryResult.parse(raw, query(), started, finished);
      expect(result.records, isEmpty);
      expect(result.rejected, count);
      expect(result.limitReached, count == 31);
    }
  });
  test(
    'receipt refresh skips event-only services and keeps unknown outcomes',
    () async {
      final core = HistoryFixture();
      (core.device['components'] as List).add({
        'id': 'main',
        'capabilities': ['power'],
      });
      core.capabilities.add(
        capability('power', {
          'state': {'type': 'boolean', 'readable': true},
        }, {}),
      );
      final c = HomeController(core, AccountPreferences());
      c.receipt = {'device_id': core.device['id'], 'status': 'unknown'};
      expect(await c.refreshReceipt(), true);
      expect(
        core.calls
            .where((v) => v['method'] == 'read_state')
            .map((v) => v['component']),
        ['main'],
      );
      expect(c.receipt!['status'], 'unknown');
      expect(core.calls.any((v) => v['method'] == 'read_event_history'), false);
      c.dispose();
    },
  );
  test('event API needs both method names and version 1.44', () {
    final core = HistoryFixture();
    expect(supportsEventHistory(core), true);
    core.minor = 43;
    expect(supportsEventHistory(core), false);
    core.minor = 44;
    core.methods.remove('read_event_history');
    expect(supportsEventHistory(core), false);
  });
  testWidgets(
    'open inspects locally; explicit read keeps evidence without polling',
    (tester) async {
      final core = HistoryFixture();
      final c = await showHistory(tester, core);
      expect(core.calls.map((v) => v['method']), [
        'inspect_event_history_options',
      ]);
      await tester.enterText(
        find.byKey(const ValueKey('event-history-start')),
        '2026-10-01T00:00:00Z',
      );
      await tester.enterText(
        find.byKey(const ValueKey('event-history-end')),
        '2026-10-03T00:00:00Z',
      );
      await readHistory(tester);
      expect(core.calls.last, {
        'method': 'read_event_history',
        'history': example('event-history.request'),
      });
      expect(find.text('2026-10-01T00:00:00Z'), findsNWidgets(3));
      expect(find.textContaining('Completeness is unproven'), findsOneWidget);
      expect(find.textContaining('Completeness is unproven'), findsOneWidget);
      expect(find.text('Rejected records: 1'), findsOneWidget);
      await tester.pump(const Duration(seconds: 15));
      expect(core.calls.length, 2);
      await tester.enterText(
        find.byKey(const ValueKey('event-history-end')),
        '2026-10-02T00:00:00Z',
      );
      await tester.pump();
      expect(find.textContaining('Completeness is unproven'), findsNothing);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
  testWidgets(
    'invalid input never sends; mismatched results are not displayed',
    (tester) async {
      final core = HistoryFixture()..malformed = true;
      final c = await showHistory(tester, core);
      await tester.enterText(
        find.byKey(const ValueKey('event-history-start')),
        'invalid',
      );
      await readHistory(tester);
      expect(core.calls.length, 1);
      expect(find.textContaining('INVALID_ARGUMENT'), findsOneWidget);
      await tester.enterText(
        find.byKey(const ValueKey('event-history-start')),
        '2026-10-01T00:00:00Z',
      );
      await tester.enterText(
        find.byKey(const ValueKey('event-history-end')),
        '2026-10-03T00:00:00Z',
      );
      await readHistory(tester);
      expect(find.textContaining('INVALID_HISTORY'), findsOneWidget);
      expect(find.textContaining('Completeness is unproven'), findsNothing);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
  testWidgets('empty options never expose a read', (tester) async {
    final core = HistoryFixture()..empty = true;
    final c = await showHistory(tester, core);
    expect(find.byKey(const ValueKey('event-history-read')), findsNothing);
    expect(find.textContaining('No event history'), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
  for (final reason in [
    'close',
    'cancel',
    'background',
    'device change',
    'session replacement',
  ]) {
    testWidgets('$reason discards a late result and cancels the request', (
      tester,
    ) async {
      final core = HistoryFixture()..gate = Completer<void>();
      final c = await showHistory(tester, core);
      await readHistory(tester, settle: false);
      if (reason == 'close') {
        await tester.tap(find.text('Close'));
      } else if (reason == 'background') {
        c.setForeground(false);
      } else if (reason == 'cancel') {
        c.cancelPending();
      } else if (reason == 'device change') {
        c.selectedDevice = 'other';
        c.notifyListeners();
      } else {
        c.core = HistoryFixture();
        c.notifyListeners();
      }
      await tester.pumpAndSettle();
      expect(find.byType(EventHistoryDialog), findsNothing);
      expect(core.cancelled, greaterThan(0));
      core.gate!.complete();
      await tester.pumpAndSettle();
      expect(find.textContaining('Completeness is unproven'), findsNothing);
      expect(
        core.calls.where((v) => v['method'] == 'read_event_history').length,
        1,
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    });
  }
  testWidgets('each connection is inspected only on selection', (tester) async {
    final core = HistoryFixture();
    (core.device['connections'] as List).add({
      'id': 'second-cloud',
      'transport': 'cloud',
    });
    final c = await showHistory(tester, core);
    expect(core.calls.length, 1);
    await tester.tap(find.text('Connection 2: Cloud'));
    await tester.pumpAndSettle();
    expect(core.calls.last['connection_id'], 'second-cloud');
    await readHistory(tester);
    expect(core.calls.last['history']['connection_id'], 'second-cloud');
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
  testWidgets('dismissal during inspection never publishes late options', (
    tester,
  ) async {
    final core = HistoryFixture()..inspectionGate = Completer<void>();
    final c = HomeController(core, AccountPreferences());
    c.selectedDevice = core.device['id'];
    c.description = {'device': core.device, 'capabilities': core.capabilities};
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) => Scaffold(
            body: TextButton(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => EventHistoryDialog(controller: c),
              ),
              child: const Text('Open fixture'),
            ),
          ),
        ),
      ),
    );
    await tester.tap(find.text('Open fixture'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));
    c.cancelPending();
    await tester.pumpAndSettle();
    core.inspectionGate!.complete();
    await tester.pumpAndSettle();
    expect(find.byType(EventHistoryDialog), findsNothing);
    expect(core.calls.length, 1);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    testWidgets('narrow enlarged history remains usable in ${locale.name}', (
      tester,
    ) async {
      await LocaleSettings.setLocale(locale);
      tester.view.physicalSize = const Size(360, 800);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final c = await showHistory(tester, HistoryFixture(), scale: 2);
      await readHistory(tester);
      expect(find.byType(EventHistoryDialog), findsOneWidget);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    });
  }
  testWidgets(
    'device detail opens history and suspends automatic state reads',
    (tester) async {
      final core = HistoryFixture();
      final c = HomeController(core, AccountPreferences());
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      final app = GlobalKey<NativeAppState>();
      await tester.pumpWidget(
        NativeApp(key: app, controller: c, catalog: catalog),
      );
      await tester.pumpAndSettle();
      await c.selectDevice(core.device['id']);
      await app.currentState!.go('C01');
      c.startAutomaticRefresh();
      await tester.pumpAndSettle();
      final button = find.text('Event history');
      await tester.ensureVisible(button);
      await tester.tap(button);
      await tester.pumpAndSettle();
      final count = core.calls.length;
      await tester.pump(const Duration(seconds: 12));
      expect(core.calls.length, count);
      expect(core.calls.last['method'], 'inspect_event_history_options');
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      await tester.pump(const Duration(seconds: 4));
      expect(core.calls.where((v) => v['method'] == 'read_state'), isEmpty);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
