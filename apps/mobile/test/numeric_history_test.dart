import 'dart:async';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/client_contract.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/history_view.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/numeric_history.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'accounts_test.dart' show AccountPreferences;
import 'controls_test.dart' show ControlsFixture, example, capability;

Json copy(Json value) => jsonDecode(jsonEncode(value)) as Json;
Json descriptor() => {
  'device': {
    'id': 'fictional-outlet',
    'components': [
      {
        'id': 'power-consumption-11',
        'capabilities': ['vendor.example.energy'],
      },
    ],
    'connections': [
      {'id': 'fictional-cloud', 'transport': 'cloud'},
    ],
  },
  'capabilities': [
    capability('vendor.example.energy', {
      'consumption': {
        'type': 'number',
        'readable': true,
        'unit': 'unrelated_live_unit',
      },
    }, {}),
  ],
};
NumericHistoryOption option([Json? raw, Json? desc]) =>
    NumericHistoryOption.parse(
      raw ?? example('history.options'),
      desc ?? descriptor(),
      'fictional-cloud',
    ).single;
final started = DateTime.utc(2026, 10, 4, 12);
final finished = started.add(const Duration(seconds: 1));
NumericHistoryQuery query() =>
    option().query('2026-10-01T00:00:00Z', '2026-10-03T00:00:00Z', started);

class HistoryFixture extends ControlsFixture implements NativeSessionClient {
  HistoryFixture() {
    capabilities
      ..clear()
      ..addAll((descriptor()['capabilities'] as List).cast<Json>());
    device.addAll(descriptor()['device'] as Json);
  }
  int minor = 33, cancelled = 0;
  bool empty = false, malformed = false;
  Completer<void>? gate, inspectionGate;
  @override
  Set<String> methods = {'inspect_history_options', 'read_history'};
  @override
  bool apiAtLeast(int value) => minor >= value;
  @override
  void cancelAll() {
    cancelled++;
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'inspect_history_options') {
      calls.add({'method': method, ...fields});
      await inspectionGate?.future;
      final result = example('history.options');
      result['connection_id'] = fields['connection_id'];
      if (empty) result['options'] = [];
      return result;
    }
    if (method == 'read_history') {
      calls.add({'method': method, ...fields});
      await gate?.future;
      final request = fields['history'] as Json;
      return {
        ...example('history.result'),
        ...request,
        'received_at': DateTime.now().toUtc().toIso8601String(),
        'source': malformed ? 'lan' : 'cloud',
        'records': [
          {'reported_at': request['start'], 'value': 0},
          {
            'reported_at': historyUtc(
              historyTime(request['start']).add(const Duration(seconds: 1)),
            ),
            'value': 1.25,
          },
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
                builder: (_) => NumericHistoryDialog(controller: c),
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
  final button = find.byKey(const ValueKey('history-read'));
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

  test('options preserve historical unit and own their target', () {
    final raw = example('history.options');
    final parsed = option(raw);
    raw['options'][0]['target']['property'] = 'changed';
    expect(parsed.target['property'], 'consumption');
    expect(parsed.unit, 'kWh');
    expect(() => parsed.target['property'] = 'changed', throwsUnsupportedError);
    expect(query().json, example('history.request'));
  });
  for (final mutation in <String, void Function(Json)>{
    'wrong device': (v) => v['device_id'] = 'other',
    'wrong connection': (v) => v['connection_id'] = 'other',
    'unknown field': (v) => v['url'] = 'unexpected',
    'wrong target': (v) => v['options'][0]['target']['component_id'] = 'other',
    'timezone assumption': (v) => v['options'][0]['bucket_timezone'] = 'UTC',
    'excess range': (v) => v['options'][0]['max_window_seconds'] = 2764801,
    'excess records': (v) => v['options'][0]['max_records'] = 32,
    'bad unit': (v) => v['options'][0]['unit'] = 'kWh\n',
    'duplicate target with reordered keys': (v) {
      final duplicate = copy(v['options'][0]);
      duplicate['target'] = {
        'property': 'consumption',
        'capability': 'vendor.example.energy',
        'component_id': 'power-consumption-11',
      };
      v['options'].add(duplicate);
    },
  }.entries) {
    test('options refuse ${mutation.key}', () {
      final raw = example('history.options');
      mutation.value(raw);
      expect(() => option(raw), throwsA(isA<NativeFailure>()));
    });
  }
  test('options require readable numeric component capability', () {
    for (final field in ['type', 'readable']) {
      final desc = descriptor();
      desc['capabilities'][0]['properties']['consumption'][field] =
          field == 'type' ? 'boolean' : false;
      expect(() => option(null, desc), throwsA(isA<NativeFailure>()));
    }
  });
  for (final invalid in [
    '2026-02-30T00:00:00Z',
    '2026-10-01T00:00:60Z',
    '2026-10-01T00:00:00.000Z',
    '2026-10-01T00:00:00+00:00',
    '2026-10-01',
    ' 2026-10-01T00:00:00Z',
  ]) {
    test(
      'query rejects noncanonical $invalid',
      () => expect(
        () => option().query(invalid, '2026-10-03T00:00:00Z', started),
        throwsA(isA<NativeFailure>()),
      ),
    );
  }
  test('range requires positive bounded past interval', () {
    for (final range in [
      ('2026-10-03T00:00:00Z', '2026-10-03T00:00:00Z'),
      ('2026-10-04T00:00:00Z', '2026-10-03T00:00:00Z'),
      ('2026-10-01T00:00:00Z', '2026-10-05T00:00:00Z'),
      ('2026-08-01T00:00:00Z', '2026-10-03T00:00:00Z'),
    ]) {
      expect(
        () => option().query(range.$1, range.$2, started),
        throwsA(isA<NativeFailure>()),
      );
    }
  });
  test(
    'records preserve zero, fractions, missing periods and rejection evidence',
    () {
      final parsed = NumericHistoryResult.parse(
        example('history.result'),
        query(),
        started,
        finished,
      );
      expect(parsed.records.map((v) => v.value), [0, 1.25]);
      expect(parsed.rejected, 1);
      expect(parsed.limitReached, false);
      final raw = example('history.result')..['records'] = [];
      expect(
        NumericHistoryResult.parse(raw, query(), started, finished).records,
        isEmpty,
      );
      raw['rejected_records'] = 31;
      raw['limit_reached'] = true;
      expect(
        NumericHistoryResult.parse(
          raw,
          query(),
          started,
          finished,
        ).limitReached,
        true,
      );
    },
  );
  for (final mutation in <String, void Function(Json)>{
    'other source': (v) => v['source'] = 'lan',
    'other range': (v) => v['start'] = '2026-10-02T00:00:00Z',
    'other unit': (v) => v['unit'] = 'Wh',
    'other target': (v) => v['target']['property'] = 'other',
    'stale receipt': (v) => v['received_at'] = '2026-10-03T12:00:00Z',
    'future receipt': (v) => v['received_at'] = '2026-10-05T12:00:00Z',
    'false completeness': (v) => v['completeness'] = 'complete',
    'unknown fields': (v) => v['total'] = 10,
    'negative value': (v) => v['records'][0]['value'] = -1,
    'nonfinite value': (v) => v['records'][0]['value'] = double.infinity,
    'fractional record timestamp': (v) =>
        v['records'][0]['reported_at'] = '2026-10-01T16:00:00.000Z',
    'end included': (v) => v['records'][1]['reported_at'] = v['end'],
    'duplicate time': (v) =>
        v['records'][1]['reported_at'] = v['records'][0]['reported_at'],
    'unsorted': (v) => v['records'] = (v['records'] as List).reversed.toList(),
    'excess count': (v) => v['rejected_records'] = 31,
    'incorrect limit': (v) => v['limit_reached'] = true,
  }.entries) {
    test('result refuses ${mutation.key}', () {
      final raw = example('history.result');
      mutation.value(raw);
      expect(
        () => NumericHistoryResult.parse(raw, query(), started, finished),
        throwsA(isA<NativeFailure>()),
      );
    });
  }
  test('API and both methods are required', () {
    final core = HistoryFixture();
    expect(supportsNumericHistory(core), true);
    core.minor = 32;
    expect(supportsNumericHistory(core), false);
    core.minor = 33;
    core.methods.remove('read_history');
    expect(supportsNumericHistory(core), false);
    expect(supportsNumericHistory(ControlsFixture()), false);
  });

  testWidgets(
    'open inspects locally; explicit read keeps evidence without polling',
    (tester) async {
      final core = HistoryFixture();
      final c = await showHistory(tester, core);
      expect(core.calls.map((v) => v['method']), ['inspect_history_options']);
      await tester.enterText(
        find.byKey(const ValueKey('history-start')),
        '2026-10-01T00:00:00Z',
      );
      await tester.enterText(
        find.byKey(const ValueKey('history-end')),
        '2026-10-03T00:00:00Z',
      );
      await readHistory(tester);
      expect(core.calls.last, {
        'method': 'read_history',
        'history': example('history.request'),
      });
      expect(find.textContaining('0 kWh'), findsOneWidget);
      expect(find.textContaining('1.25 kWh'), findsOneWidget);
      expect(find.textContaining('Completeness is unproven'), findsOneWidget);
      expect(find.text('Rejected records: 1'), findsOneWidget);
      await tester.pump(const Duration(seconds: 15));
      expect(core.calls.length, 2);
      await tester.enterText(
        find.byKey(const ValueKey('history-end')),
        '2026-10-02T00:00:00Z',
      );
      await tester.pump();
      expect(find.textContaining('1.25 kWh'), findsNothing);
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
        find.byKey(const ValueKey('history-start')),
        'invalid',
      );
      await readHistory(tester);
      expect(core.calls.length, 1);
      expect(find.textContaining('INVALID_ARGUMENT'), findsOneWidget);
      await tester.enterText(
        find.byKey(const ValueKey('history-start')),
        '2026-10-01T00:00:00Z',
      );
      await tester.enterText(
        find.byKey(const ValueKey('history-end')),
        '2026-10-03T00:00:00Z',
      );
      await readHistory(tester);
      expect(find.textContaining('INVALID_HISTORY'), findsOneWidget);
      expect(find.textContaining('1.25 kWh'), findsNothing);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
  testWidgets('empty options never expose a read', (tester) async {
    final core = HistoryFixture()..empty = true;
    final c = await showHistory(tester, core);
    expect(find.byKey(const ValueKey('history-read')), findsNothing);
    expect(find.textContaining('No historical series'), findsOneWidget);
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
      expect(find.byType(NumericHistoryDialog), findsNothing);
      expect(core.cancelled, greaterThan(0));
      core.gate!.complete();
      await tester.pumpAndSettle();
      expect(find.textContaining('1.25 kWh'), findsNothing);
      expect(core.calls.where((v) => v['method'] == 'read_history').length, 1);
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
                builder: (_) => NumericHistoryDialog(controller: c),
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
    expect(find.byType(NumericHistoryDialog), findsNothing);
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
      expect(find.textContaining('1.25 kWh'), findsOneWidget);
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
      final button = find.text('Historical readings');
      await tester.ensureVisible(button);
      await tester.tap(button);
      await tester.pumpAndSettle();
      final count = core.calls.length;
      await tester.pump(const Duration(seconds: 12));
      expect(core.calls.length, count);
      expect(core.calls.last['method'], 'inspect_history_options');
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      await tester.pump(const Duration(seconds: 4));
      expect(core.calls.last['method'], 'read_state');
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
