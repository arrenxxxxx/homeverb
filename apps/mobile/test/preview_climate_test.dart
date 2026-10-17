import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/climate_controls.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/preview_climate.dart';
import 'package:homeverb_mobile/screens.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUp(() => LocaleSettings.setLocale(AppLocale.enUs));
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));

  Future<void> open(WidgetTester tester) async {
    tester.view.physicalSize = const Size(860, 1864);
    tester.view.devicePixelRatio = 2;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      TranslationProvider(
        child: PreviewApp(
          catalog: catalog,
          initial: 'C01',
          contentBuilder: (id, onAction) =>
              id == 'C01' ? PreviewClimate(onAction: onAction) : null,
        ),
      ),
    );
    await tester.pumpAndSettle();
  }

  testWidgets(
    'preview reuses current climate controls with memory-only edits',
    (tester) async {
      await open(tester);
      expect(find.byType(ClimateControls), findsOneWidget);
      expect(find.text('24°C'), findsOneWidget);
      await tester.tap(find.byKey(const ValueKey('climate-increase')));
      await tester.pumpAndSettle();
      expect(find.text('24.5°C'), findsOneWidget);
      final control = tester.widget<ClimateControls>(
        find.byType(ClimateControls),
      );
      expect(control.controller.preferences.file, isNull);
      expect(control.controller.receipt, isNull);
      expect(control.controller.observed('climate.target_temperature'), 24.5);
      await tester.tap(find.byKey(const ValueKey('climate-power')));
      await tester.pumpAndSettle();
      expect(control.controller.observed('power.state'), false);
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets('preview airflow scenarios hide unsupported positions and axes', (
    tester,
  ) async {
    await open(tester);
    expect(
      find.byKey(const ValueKey('climate-stepper:horizontal_direction')),
      findsOneWidget,
    );
    Future<void> select(String label) async {
      final dropdown = find.byKey(const ValueKey('preview-airflow'));
      await tester.ensureVisible(dropdown);
      await tester.tap(dropdown);
      await tester.pumpAndSettle();
      await tester.tap(
        find.widgetWithText(DropdownMenuItem<String>, label).last,
      );
      await tester.pumpAndSettle();
    }

    await select('Horizontal swing');
    expect(
      find.byKey(const ValueKey('climate-stepper:horizontal_direction')),
      findsNothing,
    );
    expect(
      find.byKey(const ValueKey('climate-switch:horizontal_swing')),
      findsOneWidget,
    );
    await select('Unavailable');
    expect(
      find.byKey(const ValueKey('climate-switch:horizontal_swing')),
      findsNothing,
    );
    expect(
      find.byKey(const ValueKey('climate-switch:vertical_swing')),
      findsNothing,
    );
    expect(find.byKey(const ValueKey('climate-row:Mode')), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
