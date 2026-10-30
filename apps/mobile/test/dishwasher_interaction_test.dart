import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/appliance_panels.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/device_typography.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/skin.dart';

import 'device_detail_controls_test.dart' show DishwasherFixture, dishwasher;

void main() {
  for (final scenario in [
    ('work', null, 'pause_cycle'),
    ('work', 'pause', 'resume_cycle'),
    ('power_on', null, 'start_cycle'),
    ('power_off', null, 'start_cycle'),
    ('unexpected', null, null),
    ('work', 'unexpected', null),
  ]) {
    testWidgets('cycle controls follow reported state $scenario', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(640, 1688);
      tester.view.devicePixelRatio = 2;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final core = DishwasherFixture();
      core.observations['$dishwasher.work_status'] = scenario.$1;
      if (scenario.$2 != null)
        core.observations['$dishwasher.operator'] = scenario.$2;
      final controller = HomeController(
        core,
        LocalPreferences(File('/unused/cycle.json')),
      );
      await controller.selectDevice(core.device['id']);
      final dispatched = <String>[];
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: DishwasherPanel(
                controller: controller,
                onAction: dispatched.add,
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      for (final action in ['start_cycle', 'pause_cycle', 'resume_cycle']) {
        expect(
          find.byKey(ValueKey('control:0:$action')),
          action == scenario.$3 ? findsOneWidget : findsNothing,
        );
      }
      if (scenario.$3 == 'pause_cycle' || scenario.$3 == 'resume_cycle') {
        expect(
          find.byKey(const ValueKey('control:0:cancel_cycle')),
          findsOneWidget,
        );
        expect(find.byType(RadioListTile<Object>), findsNothing);
        expect(find.byIcon(Icons.thermostat_outlined), findsOneWidget);
        expect(find.byIcon(Icons.door_front_door_outlined), findsOneWidget);
        expect(find.byIcon(Icons.tune), findsOneWidget);
        final action = find.byKey(ValueKey('control:0:${scenario.$3}'));
        expect(tester.widget<ActionButton>(action).style, '');
        expect(
          tester
              .widget<ActionButton>(
                find.byKey(const ValueKey('control:0:cancel_cycle')),
              )
              .style,
          'secondary',
        );
        await tester.ensureVisible(action);
        await tester.tap(action);
        await tester.pump();
        expect(dispatched.single, 'control:0:${scenario.$3}');
        expect(
          core.prepared,
          isNull,
          reason: 'The panel opens review; it never consumes trusted approval.',
        );
      } else if (scenario.$3 == null) {
        expect(
          find.text(
            'Cycle state is unknown. Refresh before choosing a wash action.',
          ),
          findsOneWidget,
        );
      } else {
        final start = find.byKey(const ValueKey('control:0:start_cycle'));
        expect(tester.widget<ActionButton>(start).onPressed, isNull);
        expect(find.byType(RadioListTile<Object>), findsNWidgets(5));
        expect(find.text('42°C'), findsNothing);
        // Collapsed groups preserve every descriptor option without flooding
        // the initial view, and selections remain drafts only.
        for (final parameter in ['program', 'additional']) {
          final more = find.byKey(ValueKey('more-wash-$parameter'));
          await tester.ensureVisible(more);
          await tester.tap(more);
          await tester.pumpAndSettle();
          final schema = controller.capability(
            dishwasher,
          )!['actions']['start_cycle']['parameters']['properties'][parameter];
          for (final value in schema['enum'] as List) {
            expect(
              find.byKey(
                ValueKey(
                  parameter == 'program'
                      ? 'wash-program:$value'
                      : 'wash-additional:$value',
                ),
              ),
              findsOneWidget,
            );
          }
          await tester.ensureVisible(more);
          await tester.tap(more);
          await tester.pumpAndSettle();
        }
        expect(dispatched, isEmpty);

        for (final key in ['wash-program:eco_wash', 'wash-additional:9']) {
          final option = find.byKey(ValueKey(key));
          await tester.ensureVisible(option);
          await tester.tap(option);
          await tester.pump();
        }
        for (final key in ['wash-program:eco_wash', 'wash-additional:9']) {
          final title =
              tester
                      .widget<RadioListTile<Object>>(find.byKey(ValueKey(key)))
                      .title!
                  as Copy;
          expect(title.size, DeviceTypography.option);
          expect(title.lineHeight, DeviceTypography.lineHeight);
          expect(
            tester.getSize(find.byKey(ValueKey(key))).height,
            greaterThanOrEqualTo(44),
          );
          expect(title.bold, isTrue);
          expect(title.color, accent);
        }
        expect(dispatched, isEmpty);
        expect(core.observations['$dishwasher.work_status'], scenario.$1);
        await tester.ensureVisible(start);
        await tester.tap(start);
        await tester.pump();
        expect(
          Uri.decodeComponent(dispatched.single),
          contains('{"program":"eco_wash","additional":9}'),
        );
      }
      if (scenario.$3 != null) {
        final callback = tester
            .widget<ActionButton>(
              find.byKey(ValueKey('control:0:${scenario.$3}')),
            )
            .onPressed!;
        final before = dispatched.length;
        final state =
            controller.snapshot!['observations']['$dishwasher.work_status'];
        final oldValue = state['value'];
        state['value'] = 'unknown-now';
        callback();
        state['value'] = oldValue;
        controller.busy = true;
        callback();
        controller.busy = false;
        final device = controller.selectedDevice;
        controller.selectedDevice = 'another-device';
        callback();
        controller.selectedDevice = device;
        controller.setForeground(false);
        callback();
        controller.setForeground(true);
        callback();
        expect(
          dispatched.length,
          before,
          reason:
              'Captured callbacks revalidate state and foreground before opening review.',
        );
      }
      if (scenario.$3 == 'start_cycle') {
        controller.busy = true;
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SingleChildScrollView(
                child: DishwasherPanel(
                  controller: controller,
                  onAction: dispatched.add,
                ),
              ),
            ),
          ),
        );
        await tester.pump();
        expect(
          tester
              .widget<ActionButton>(
                find.byKey(const ValueKey('control:0:start_cycle')),
              )
              .onPressed,
          isNull,
        );
        expect(
          tester
              .widget<RadioListTile<Object>>(
                find.byKey(const ValueKey('wash-program:eco_wash')),
              )
              .enabled,
          false,
        );
        controller.busy = false;
      }
      expect(core.calls.where((call) => call['method'] == 'execute'), isEmpty);
      expect(tester.takeException(), isNull);
      controller.dispose();
    });
  }
}
