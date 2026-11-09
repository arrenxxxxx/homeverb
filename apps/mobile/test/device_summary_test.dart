import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/device_summary.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/native_client.dart';

Json snapshot(Map<String, Object?> values) => {
  'observations': {
    for (final entry in values.entries) entry.key: {'value': entry.value},
  },
};

void main() {
  testWidgets('card summaries use each declared appliance observation', (
    tester,
  ) async {
    late BuildContext context;
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (value) {
            context = value;
            return const SizedBox();
          },
        ),
      ),
    );
    ({Object value, Object caption}) summary(
      Json device,
      Map<String, Json> snapshots,
    ) => deviceSummary(context, device, (component) => snapshots[component]);
    Json device(List<Json> parts) => {'components': parts};
    Json part(String id, List<String> capabilities) => {
      'id': id,
      'capabilities': capabilities,
    };

    final ac = summary(
      device([
        part('main', ['climate']),
      ]),
      {
        'main': snapshot({
          'climate.target_temperature': 24.5,
          'climate.mode': 'cool',
        }),
      },
    );
    expect(localize(context, ac.value), '24.5°C');
    expect(localize(context, ac.caption), 'Cool');

    final fridge = summary(
      device([
        part('fridge', ['climate']),
        part('freezer', ['climate']),
        part('variable', ['climate']),
      ]),
      {
        'fridge': snapshot({'climate.target_temperature': 4}),
        'freezer': snapshot({'climate.current_temperature': -18}),
        'variable': snapshot({'climate.target_temperature': 2}),
      },
    );
    expect(localize(context, fridge.value), '4°C');
    expect(
      localize(context, fridge.caption),
      'Fridge target · Freezer current -18°C · Variable zone target 2°C',
    );

    const dishwasher = 'vendor.midea.dishwasher';
    final wash = summary(
      device([
        part('main', [dishwasher]),
      ]),
      {
        'main': snapshot({
          '$dishwasher.mode': 'eco_wash',
          '$dishwasher.work_status': 'work',
          '$dishwasher.remaining_minutes': 42,
          'climate.target_temperature': 99,
        }),
      },
    );
    expect(localize(context, wash.value), 'Eco wash');
    expect(localize(context, wash.caption), 'Washing · 42 min');
    final absent = summary(
      device([
        part('main', ['climate']),
      ]),
      {},
    );
    expect(localize(context, absent.value), '—');
    expect(localize(context, absent.caption), '—');
  });

  testWidgets('Chinese zone labels preserve target and current meanings', (
    tester,
  ) async {
    await LocaleSettings.setLocale(AppLocale.zhCn);
    addTearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
    late BuildContext context;
    await tester.pumpWidget(
      TranslationProvider(
        child: MaterialApp(
          home: Builder(
            builder: (value) {
              context = value;
              return const SizedBox();
            },
          ),
        ),
      ),
    );
    final device = {
      'components': [
        {
          'id': 'fridge',
          'capabilities': ['climate'],
        },
        {
          'id': 'freezer',
          'capabilities': ['climate'],
        },
      ],
    };
    final values = {
      'fridge': snapshot({'climate.target_temperature': 4}),
      'freezer': snapshot({'climate.current_temperature': -18}),
    };
    final result = deviceSummary(context, device, (id) => values[id]);
    expect(localize(context, result.value), '4°C');
    expect(localize(context, result.caption), '\u51b7\u85cf\u5ba4\u76ee\u6807\u503c · \u51b7\u51bb\u5ba4\u5f53\u524d -18°C');
  });
}
