import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/device_detail_controls.dart';
import 'package:homeverb_mobile/device_summary.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'controls_test.dart' show example;
import 'device_detail_controls_test.dart' show DishwasherFixture, dishwasher;

const details = '$dishwasher.details';

class GxDishwasherFixture extends DishwasherFixture {
  GxDishwasherFixture({String state = 'work_pause'}) {
    capabilities.first['actions'] = <String, dynamic>{};
    capabilities.first['properties'].remove('additional');
    capabilities.add(example('capability.meiju-dishwasher-details'));
    device['model'] = 'Fictional-GX1200';
    device['components'] = [
      {
        'id': 'main',
        'capabilities': [dishwasher, details],
      },
    ];
    observations
      ..clear()
      ..addAll({
        '$dishwasher.work_status': state,
        '$dishwasher.mode': 'wash_dry',
        '$details.cycle_state': state,
        '$details.program': 'wash_dry',
        '$details.door_state': 'closed',
        '$details.temperature': 47.5,
        if (state != 'power_on') '$details.remaining_minutes': 26,
        if (state == 'power_on') '$details.estimated_minutes': 90,
        '$details.smart_dose': true,
        '$details.child_lock': false,
      });
  }
}

void main() {
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  testWidgets(
    'old cache does not expose raw new phase tokens before normalized readings arrive',
    (tester) async {
      final core = GxDishwasherFixture(state: 'dry');
      core.observations.removeWhere((key, _) => key.startsWith('$details.'));
      final controller = HomeController(core, LocalPreferences.memory());
      addTearDown(controller.dispose);
      await controller.selectDevice(core.device['id']);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: DeviceDetailControls(
                controller: controller,
                onAction: (_) {},
              ),
            ),
          ),
        ),
      );
      expect(find.text('dry'), findsNothing);
      expect(find.text('wash_dry'), findsNothing);
      expect(find.text('Unknown'), findsWidgets);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );
  for (final scenario in [
    ('work_pause', 'Paused'),
    ('dry', 'Drying'),
    ('keep', 'Storing'),
    ('order', 'Scheduled'),
    ('drain', 'Draining'),
    ('power_on', 'Standby'),
  ]) {
    testWidgets(
      'exact read scope presents ${scenario.$1} without command authority',
      (tester) async {
        final core = GxDishwasherFixture(state: scenario.$1);
        final controller = HomeController(core, LocalPreferences.memory());
        addTearDown(controller.dispose);
        await controller.selectDevice(core.device['id']);
        final before = core.calls.length;
        final dispatched = <String>[];
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SingleChildScrollView(
                child: DeviceDetailControls(
                  controller: controller,
                  onAction: dispatched.add,
                ),
              ),
            ),
          ),
        );
        expect(find.text(scenario.$2), findsOneWidget);
        expect(find.textContaining('Wash and dry'), findsOneWidget);
        expect(find.text('Door closed'), findsOneWidget);
        expect(find.text('Smart dosing'), findsOneWidget);
        expect(find.text('Enabled'), findsOneWidget);
        expect(find.text('Disabled'), findsOneWidget);
        expect(find.byType(Switch), findsNothing);
        expect(find.byType(RadioListTile<Object>), findsNothing);
        expect(find.text('Unknown'), findsNothing);
        expect(core.prepared, isNull);
        expect(dispatched, isEmpty);
        expect(core.calls.length, before);
        if (scenario.$1 == 'power_on') {
          expect(find.text('Estimated program time'), findsOneWidget);
          expect(find.text('90 min'), findsOneWidget);
          final context = tester.element(find.byType(DeviceDetailControls));
          final summary = deviceSummary(
            context,
            core.device,
            controller.componentSnapshot,
          );
          expect(
            localize(context, summary.caption),
            contains('Estimated 90 min'),
          );
        } else {
          expect(find.text('26 min'), findsOneWidget);
        }
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
      },
    );
  }

  testWidgets(
    'Chinese details and card use mapped names at narrow large text',
    (tester) async {
      LocaleSettings.setLocale(AppLocale.zhCn);
      tester.view.physicalSize = const Size(320, 900);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final core = GxDishwasherFixture(state: 'dry');
      final controller = HomeController(core, LocalPreferences.memory());
      addTearDown(controller.dispose);
      await controller.selectDevice(core.device['id']);
      late BuildContext context;
      await tester.pumpWidget(
        TranslationProvider(
          child: MaterialApp(
            home: Scaffold(
              body: Builder(
                builder: (value) {
                  context = value;
                  return MediaQuery(
                    data: const MediaQueryData(
                      textScaler: TextScaler.linear(2),
                    ),
                    child: SingleChildScrollView(
                      child: DeviceDetailControls(
                        controller: controller,
                        onAction: (_) {},
                      ),
                    ),
                  );
                },
              ),
            ),
          ),
        ),
      );
      expect(
        find.text(sourceForKey('$details.cycle_state.dry')!),
        findsNothing,
      );
      expect(find.text('wash_dry'), findsNothing);
      final summary = deviceSummary(
        context,
        core.device,
        controller.componentSnapshot,
      );
      expect(
        localize(context, summary.value),
        localize(context, sourceForKey('$details.program.wash_dry')!),
      );
      expect(
        localize(context, summary.caption),
        contains(localize(context, 'Drying')),
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets(
    'absent detail values keep missing readings and never invent options',
    (tester) async {
      final core = GxDishwasherFixture();
      core.observations.clear();
      final controller = HomeController(core, LocalPreferences.memory());
      addTearDown(controller.dispose);
      await controller.selectDevice(core.device['id']);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: DeviceDetailControls(
                controller: controller,
                onAction: (_) {},
              ),
            ),
          ),
        ),
      );
      expect(find.text('—'), findsWidgets);
      expect(find.text('Reported wash settings'), findsNothing);
      expect(find.text('Door closed'), findsNothing);
      expect(find.text('Standby'), findsNothing);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );
}
