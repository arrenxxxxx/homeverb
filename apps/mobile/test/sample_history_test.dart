import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/client_contract.dart';
import 'package:homeverb_mobile/history_view.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/numeric_history.dart';
import 'controls_test.dart' show example, capability;
import 'numeric_history_test.dart'
    show HistoryFixture, showHistory, readHistory, copy, started, finished;

Json sampleDescriptor({bool unitless = false}) => {
  'device': {
    'id': 'fictional-sensor',
    'components': [
      {
        'id': 'environment-2',
        'capabilities': ['vendor.example.environment'],
      },
    ],
    'connections': [
      {'id': 'fictional-cloud', 'transport': 'cloud'},
    ],
  },
  'capabilities': [
    capability('vendor.example.environment', {
      'temperature': {
        'type': 'number',
        'readable': true,
        'minimum': -30,
        'maximum': 100,
        'multipleOf': 0.05,
        if (!unitless) 'unit': 'celsius',
      },
    }, {}),
  ],
};
NumericHistoryOption sampleOption({Json? raw, Json? descriptor}) =>
    NumericHistoryOption.parse(
      raw ?? example('history.samples.options'),
      descriptor ?? sampleDescriptor(),
      'fictional-cloud',
      includeReportedSamples: true,
    ).single;
NumericHistoryQuery sampleQuery([NumericHistoryOption? option]) =>
    (option ?? sampleOption()).query(
      '2026-10-01T00:00:00Z',
      '2026-10-03T00:00:00Z',
      started,
    );

class SampleHistoryFixture extends HistoryFixture {
  SampleHistoryFixture({this.unitless = false}) {
    minor = 38;
    final desc = sampleDescriptor(unitless: unitless);
    device.addAll(desc['device'] as Json);
    capabilities
      ..clear()
      ..addAll((desc['capabilities'] as List).cast<Json>());
  }
  final bool unitless;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'inspect_history_options') {
      calls.add({'method': method, ...fields});
      await inspectionGate?.future;
      final result = example('history.samples.options');
      if (unitless) result['options'][0]['unit'] = null;
      if (fields['include_reported_samples'] != true) result['options'] = [];
      return result;
    }
    if (method == 'read_history') {
      calls.add({'method': method, ...fields});
      await gate?.future;
      final request = fields['history'] as Json;
      return {
        ...example('history.samples.result'),
        ...request,
        'unit': unitless ? null : 'celsius',
        'received_at': DateTime.now().toUtc().toIso8601String(),
        'records': [
          {'reported_at': request['start'], 'value': -12.3},
        ],
      };
    }
    return super.call(method, fields);
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

  test('sample admission is explicit and queries retain aggregation', () {
    expect(
      () => NumericHistoryOption.parse(
        example('history.samples.options'),
        sampleDescriptor(),
        'fictional-cloud',
      ),
      throwsA(isA<NativeFailure>()),
    );
    expect(sampleQuery().json, example('history.samples.request'));
    final result = NumericHistoryResult.parse(
      example('history.samples.result'),
      sampleQuery(),
      started,
      finished,
    );
    expect(result.records.map((r) => r.value), [-12.3, 1.25]);
  });
  test('daily and sample series may share a target but duplicates refuse', () {
    final raw = example('history.samples.options');
    final daily = copy(raw['options'][0]);
    daily['aggregation'] = 'reported_daily';
    daily['unit'] = 'kWh';
    raw['options'].add(daily);
    expect(
      NumericHistoryOption.parse(
        raw,
        sampleDescriptor(),
        'fictional-cloud',
        includeReportedSamples: true,
      ).length,
      2,
    );
    raw['options'].add(copy(raw['options'][0]));
    expect(
      () => NumericHistoryOption.parse(
        raw,
        sampleDescriptor(),
        'fictional-cloud',
        includeReportedSamples: true,
      ),
      throwsA(isA<NativeFailure>()),
    );
  });
  test(
    'sample units are exact, unitless stays null and daily requires a unit',
    () {
      final raw = example('history.samples.options');
      raw['options'][0]['unit'] = 'kWh';
      expect(() => sampleOption(raw: raw), throwsA(isA<NativeFailure>()));
      raw['options'][0]['unit'] = null;
      expect(() => sampleOption(raw: raw), throwsA(isA<NativeFailure>()));
      final option = sampleOption(
        raw: raw,
        descriptor: sampleDescriptor(unitless: true),
      );
      final result = example('history.samples.result')..['unit'] = null;
      expect(
        NumericHistoryResult.parse(
          result,
          sampleQuery(option),
          started,
          finished,
        ).query.option.unit,
        isNull,
      );
      raw['options'][0]['aggregation'] = 'reported_daily';
      expect(
        () => sampleOption(
          raw: raw,
          descriptor: sampleDescriptor(unitless: true),
        ),
        throwsA(isA<NativeFailure>()),
      );
    },
  );
  test('sample target and numeric domain are owned immutable copies', () {
    final raw = example('history.samples.options');
    final desc = sampleDescriptor();
    desc['capabilities'][0]['properties']['temperature']['enum'] = [
      -12.3,
      1.25,
    ];
    final option = sampleOption(raw: raw, descriptor: desc);
    raw['options'][0]['target']['property'] = 'other';
    desc['capabilities'][0]['properties']['temperature']['enum'][0] = 99.0;
    desc['capabilities'][0]['properties']['temperature']['minimum'] = 0;
    expect(option.target['property'], 'temperature');
    expect(option.domain['minimum'], -30);
    expect(option.domain['enum'], [-12.3, 1.25]);
    expect(() => option.domain['minimum'] = 0, throwsUnsupportedError);
    expect(() => option.domain['enum'][0] = 0, throwsUnsupportedError);
    NumericHistoryResult.parse(
      example('history.samples.result'),
      sampleQuery(option),
      started,
      finished,
    );
  });
  for (final fault in [
    'below minimum',
    'above maximum',
    'off step',
    'integer',
    'enum',
    'nonfinite',
    'string',
    'private metadata',
    'wrong aggregation',
    'wrong unit',
  ]) {
    test('sample result refuses $fault', () {
      final desc = sampleDescriptor();
      final raw = example('history.samples.result');
      final property = desc['capabilities'][0]['properties']['temperature'];
      switch (fault) {
        case 'below minimum':
          raw['records'][0]['value'] = -30.05;
        case 'above maximum':
          raw['records'][0]['value'] = 100.05;
        case 'off step':
          raw['records'][0]['value'] = 1.23;
        case 'integer':
          property['type'] = 'integer';
        case 'enum':
          property['enum'] = [0, 1];
        case 'nonfinite':
          raw['records'][0]['value'] = double.infinity;
        case 'string':
          raw['records'][0]['value'] = '-12.3';
        case 'private metadata':
          raw['records'][0]['uid'] = 'fictional-actor';
        case 'wrong aggregation':
          raw['aggregation'] = 'reported_daily';
        case 'wrong unit':
          raw['unit'] = null;
      }
      expect(
        () => NumericHistoryResult.parse(
          raw,
          sampleQuery(sampleOption(descriptor: desc)),
          started,
          finished,
        ),
        throwsA(isA<NativeFailure>()),
      );
    });
  }
  test('daily results remain nonnegative with sample support enabled', () {
    final options = example('history.samples.options');
    options['options'][0]['aggregation'] = 'reported_daily';
    final raw = example('history.samples.result')
      ..['aggregation'] = 'reported_daily';
    expect(
      () => NumericHistoryResult.parse(
        raw,
        sampleQuery(sampleOption(raw: options)),
        started,
        finished,
      ),
      throwsA(isA<NativeFailure>()),
    );
  });
  for (final version in [37, 38]) {
    testWidgets('API 1.$version negotiates sample inspection', (tester) async {
      final core = SampleHistoryFixture()..minor = version;
      final c = await showHistory(tester, core);
      expect(
        core.calls.single.containsKey('include_reported_samples'),
        version >= 38,
      );
      if (version >= 38) {
        expect(core.calls.single['include_reported_samples'], isTrue);
        expect(find.text('Reported samples'), findsOneWidget);
        await readHistory(tester);
        expect(core.calls.last['history']['aggregation'], 'reported_samples');
        expect(find.textContaining('-12.3 celsius'), findsOneWidget);
        expect(find.textContaining('not current state'), findsOneWidget);
        await tester.pump(const Duration(seconds: 12));
        expect(core.calls.length, 2);
      } else {
        expect(find.byKey(const ValueKey('history-read')), findsNothing);
        expect(find.textContaining('No historical series'), findsOneWidget);
      }
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    });
  }
  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    testWidgets('unitless samples fit narrow enlarged ${locale.name}', (
      tester,
    ) async {
      await LocaleSettings.setLocale(locale);
      tester.view.physicalSize = const Size(360, 800);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final c = await showHistory(
        tester,
        SampleHistoryFixture(unitless: true),
        scale: 2,
      );
      await readHistory(tester);
      expect(find.textContaining('\n-12.3'), findsOneWidget);
      expect(find.textContaining('-12.3 null'), findsNothing);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    });
  }
  testWidgets('background cancels samples and discards a late response', (
    tester,
  ) async {
    final core = SampleHistoryFixture()..gate = Completer<void>();
    final c = await showHistory(tester, core);
    await readHistory(tester, settle: false);
    c.setForeground(false);
    await tester.pumpAndSettle();
    expect(find.byType(NumericHistoryDialog), findsNothing);
    expect(core.cancelled, greaterThan(0));
    core.gate!.complete();
    await tester.pumpAndSettle();
    expect(find.textContaining('-12.3'), findsNothing);
    expect(core.calls.where((v) => v['method'] == 'read_history').length, 1);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
}
