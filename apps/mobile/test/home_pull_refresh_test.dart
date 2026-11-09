import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/screens.dart';
import 'package:homeverb_mobile/skin.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  late ScreenCatalog catalog;
  setUpAll(() async => catalog = await ScreenCatalog.load());
  setUp(() => LocaleSettings.setLocale(AppLocale.enUs));

  Future<void> showScreen(
    WidgetTester tester, {
    String id = 'D01',
    required Future<void> Function() refresh,
    Widget home = const SizedBox(height: 20),
  }) async {
    tester.view.physicalSize = const Size(390, 844);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    await tester.pumpWidget(
      MaterialApp(
        theme: windowTheme(),
        home: Scaffold(
          body: ScreenView(
            id: id,
            catalog: catalog,
            onAction: (_) {},
            fields: {},
            choices: {},
            values: {},
            home: home,
            onHomeRefresh: refresh,
          ),
        ),
      ),
    );
    await tester.pump();
  }

  for (final id in ['D01', 'D03', 'D11', 'D12']) {
    testWidgets('$id refreshes from a short scrollable', (tester) async {
      var calls = 0;
      final gate = Completer<void>();
      await showScreen(
        tester,
        id: id,
        refresh: () {
          calls++;
          return gate.future;
        },
      );
      expect(find.byTooltip('Refresh State'), findsNothing);
      final scroll = find.byKey(PageStorageKey('scroll:$id'));
      expect(tester.getSize(scroll).height, greaterThan(600));
      expect(
        tester.widget<SingleChildScrollView>(scroll).physics,
        isA<AlwaysScrollableScrollPhysics>(),
      );
      final gesture = await tester.startGesture(tester.getCenter(scroll));
      await gesture.moveBy(const Offset(0, 25));
      await tester.pump();
      await gesture.moveBy(const Offset(0, 5));
      await tester.pump();
      expect(find.text('Pull to refresh'), findsOneWidget);
      await gesture.moveBy(const Offset(0, 360));
      await tester.pump();
      expect(find.text('Release to refresh'), findsOneWidget);
      await gesture.up();
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 250));
      expect(calls, 1);
      expect(find.text('Refreshing devices…'), findsOneWidget);
      gate.complete();
      await tester.pumpAndSettle();
      expect(find.text('Refreshing devices…'), findsNothing);
      expect(tester.takeException(), isNull);
    });
  }

  testWidgets('accessible refresh shares the active operation', (tester) async {
    var calls = 0;
    final gate = Completer<void>();
    await showScreen(
      tester,
      refresh: () {
        calls++;
        return gate.future;
      },
    );
    final semantics = tester.widget<Semantics>(
      find
          .descendant(
            of: find.byType(HomePullRefresh),
            matching: find.byWidgetPredicate(
              (widget) =>
                  widget is Semantics &&
                  widget.properties.customSemanticsActions?.isNotEmpty == true,
            ),
          )
          .first,
    );
    final actions = semantics.properties.customSemanticsActions!;
    expect(actions.keys.single.label, 'Refresh State');
    actions.values.single();
    actions.values.single();
    await tester.pump();
    expect(calls, 1);
    expect(find.text('Refreshing devices…'), findsOneWidget);
    gate.complete();
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
  });

  testWidgets('horizontal room scrolling does not start refresh', (
    tester,
  ) async {
    var calls = 0;
    await showScreen(
      tester,
      refresh: () async => calls++,
      home: DeviceHome(
        mode: 'all',
        onAction: (_) {},
        roomNames: List.generate(12, (index) => 'Room $index'),
        cards: const SizedBox(),
      ),
    );
    await tester.drag(find.byType(ListView).first, const Offset(-150, 0));
    await tester.pumpAndSettle();
    expect(calls, 0);
    expect(find.text('Pull to refresh'), findsNothing);
  });
}
