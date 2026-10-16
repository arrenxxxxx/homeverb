import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/blocks.dart';
import 'package:homeverb_mobile/screens.dart';
import 'package:homeverb_mobile/skin.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'component_test.dart' show CompartmentFixture;

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  late ScreenCatalog catalog;
  setUpAll(() async => catalog = await ScreenCatalog.load());

  testWidgets(
    'modal keyboard focus stays above its backdrop and Escape returns',
    (tester) async {
      final key = GlobalKey<PreviewAppState>();
      await tester.pumpWidget(
        PreviewApp(key: key, catalog: catalog, initial: 'C14'),
      );
      await tester.pumpAndSettle();
      final seen = <FocusNode>{};
      for (var i = 0; i < 12; i++) {
        await tester.sendKeyEvent(LogicalKeyboardKey.tab);
        await tester.pumpAndSettle();
        final focused = FocusManager.instance.primaryFocus!;
        seen.add(focused);
        final ancestors = <Element>[];
        focused.context!.visitAncestorElements((element) {
          ancestors.add(element);
          return true;
        });
        expect(
          ancestors.any(
            (element) =>
                element.widget is IgnorePointer &&
                (element.widget as IgnorePointer).ignoring,
          ),
          false,
          reason: 'A modal must not focus its inert backdrop',
        );
      }
      expect(seen.length, greaterThanOrEqualTo(2));
      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      await tester.pumpAndSettle();
      expect(key.currentState!.id, 'C01');
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets('native roots and nested pages work with Tab, Enter and Escape', (
    tester,
  ) async {
    final directory = Directory.systemTemp.createTempSync('homeverb-keyboard-');
    final c = HomeController(
      CompartmentFixture(),
      LocalPreferences(File('${directory.path}/preferences.json')),
    );
    final key = GlobalKey<NativeAppState>();
    await tester.pumpWidget(
      NativeApp(key: key, catalog: catalog, controller: c),
    );
    await tester.pumpAndSettle();
    Future<void> activate(String label) async {
      for (var i = 0; i < 40; i++) {
        await tester.sendKeyEvent(LogicalKeyboardKey.tab);
        await tester.pumpAndSettle();
        final context = FocusManager.instance.primaryFocus?.context;
        if (context == null) continue;
        final target = find.descendant(
          of: find.byElementPredicate((e) => identical(e, context)),
          matching: find.text(label),
        );
        if (target.evaluate().isNotEmpty) {
          await tester.runAsync(() async {
            await tester.sendKeyEvent(LogicalKeyboardKey.enter);
            for (var wait = 0; c.busy && wait < 100; wait++) {
              await Future<void>.delayed(const Duration(milliseconds: 10));
            }
          });
          expect(c.busy, false);
          await tester.pumpAndSettle();
          return;
        }
      }
      fail('Keyboard focus did not reach $label');
    }

    await activate('Settings');
    expect(key.currentState!.id, 'S01');
    await activate('Appearance');
    expect(key.currentState!.id, 'S02');
    await tester.runAsync(() async {
      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      for (var wait = 0; c.busy && wait < 100; wait++) {
        await Future<void>.delayed(const Duration(milliseconds: 10));
      }
    });
    expect(c.busy, false);
    await tester.pumpAndSettle();
    expect(key.currentState!.id, 'S01');
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
    directory.deleteSync(recursive: true);
  });

  testWidgets(
    'reduced motion keeps busy activity static without claiming a percentage',
    (tester) async {
      final semantics = tester.ensureSemantics();
      try {
        await tester.pumpWidget(
          MaterialApp(
            home: MediaQuery(
              data: const MediaQueryData(disableAnimations: true),
              child: Scaffold(
                body: ScreenView(
                  id: 'D01',
                  catalog: catalog,
                  onAction: (_) {},
                  fields: {},
                  choices: {},
                  values: {},
                  busy: true,
                ),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        expect(tester.binding.hasScheduledFrame, false);
        final busy = tester.getSemantics(find.byType(OperationProgress));
        expect(busy.label, 'Operation in progress');
        expect(busy.value, isEmpty);
        expect(busy.getSemanticsData().flagsCollection.isLiveRegion, true);
      } finally {
        semantics.dispose();
      }
    },
  );

  testWidgets('all atlas states expose labelled tap targets', (tester) async {
    final semantics = tester.ensureSemantics();
    try {
      for (final id in catalog.screens.keys) {
        tester.view.devicePixelRatio = 1;
        tester.view.physicalSize = id.startsWith('W')
            ? const Size(1024, 768)
            : const Size(390, 844);
        await tester.pumpWidget(
          PreviewApp(
            key: ValueKey(id),
            catalog: catalog,
            initial: id,
            testChrome: true,
          ),
        );
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull, reason: id);
        await expectLater(
          tester,
          meetsGuideline(labeledTapTargetGuideline),
          reason: id,
        );
      }
    } finally {
      semantics.dispose();
      tester.view.resetDevicePixelRatio();
      tester.view.resetPhysicalSize();
    }
  });

  testWidgets('fields, sliders and switches announce their purpose', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Column(
            children: [
              Blocks(
                [
                  ['field', 'Room name', 'Bedroom'],
                ],
                onAction: (_) {},
                fields: {},
                choices: {},
                values: {},
              ),
              ValueSlider(
                label: 'Brightness',
                displayValue: '40%',
                value: .4,
                onChanged: (_) {},
                onChangeEnd: (_) {},
                formatValue: (v) => '${(v * 100).round()}%',
              ),
              Rows(const [
                ['Power', 'On'],
              ], onAction: (_) {}),
            ],
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final nodes = <SemanticsNode>[];
    void collect(SemanticsNode node) {
      if (!node.isMergedIntoParent) nodes.add(node);
      node.visitChildren((child) {
        collect(child);
        return true;
      });
    }

    collect(
      tester
          .binding
          .renderViews
          .single
          .owner!
          .semanticsOwner!
          .rootSemanticsNode!,
    );
    final field = nodes.singleWhere(
      (node) => node.getSemanticsData().flagsCollection.isTextField,
    );
    expect(field.label, contains('Room name'));
    final slider = nodes.singleWhere(
      (node) => node.getSemanticsData().hasAction(SemanticsAction.increase),
    );
    expect(slider.label, contains('Brightness'));
    final toggle = nodes.singleWhere(
      (node) =>
          node.getSemanticsData().flagsCollection.isToggled != ui.Tristate.none,
    );
    expect(toggle.label, contains('Power'));
    semantics.dispose();
  });
}
