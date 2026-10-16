import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/screens.dart';
import 'package:homeverb_mobile/blocks.dart';
import 'package:homeverb_mobile/skin.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  late ScreenCatalog catalog;
  setUpAll(() async {
    final font = FontLoader('Inter')
      ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf'));
    await font.load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
    catalog = await ScreenCatalog.load();
  });
  testWidgets('all 82 states render native widgets and capture screenshots', (
    tester,
  ) async {
    debugDisableShadows = false;
    addTearDown(() => debugDisableShadows = true);
    final output = Directory('build/verification/screens');
    output.createSync(recursive: true);
    final boundary = GlobalKey();
    for (final id in catalog.screens.keys) {
      final wide = id.startsWith('W');
      tester.view.devicePixelRatio = 2;
      tester.view.physicalSize = wide
          ? const Size(2048, 1536)
          : const Size(780, 1688);
      await tester.pumpWidget(
        RepaintBoundary(
          key: boundary,
          child: PreviewApp(
            key: ValueKey(id),
            catalog: catalog,
            initial: id,
            testChrome: true,
          ),
        ),
      );
      await tester.runAsync(() async {
        for (final asset in [
          'window-light-background',
          'ac',
          'lamp',
          'tv',
          'sensor',
        ]) {
          await precacheImage(
            AssetImage('assets/$asset.png'),
            tester.element(find.byType(PreviewApp)),
          );
        }
      });
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull, reason: id);
      if (id == 'P08') {
        expect(find.text('Account result'), findsOneWidget);
        expect(find.text('Device retrieval complete'), findsOneWidget);
        expect(find.text('Find Devices'), findsNothing);
        expect(find.text('View Devices'), findsOneWidget);
        expect(find.textContaining('Authorization finished'), findsNothing);
        expect(find.byType(LinearProgressIndicator), findsNothing);
      }
      await tester.runAsync(() async {
        final image =
            await (boundary.currentContext!.findRenderObject()
                    as RenderRepaintBoundary)
                .toImage(pixelRatio: 2);
        final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
        File(
          '${output.path}/$id.png',
        ).writeAsBytesSync(bytes!.buffer.asUint8List());
        image.dispose();
      });
      expect(
        find.byType(Text),
        findsWidgets,
        reason: '$id must contain real text widgets',
      );
    }
    tester.view.resetPhysicalSize();
    tester.view.resetDevicePixelRatio();
    debugDisableShadows = true;
  });
  testWidgets(
    'root navigation, sheets, actual input, selection and caller back',
    (tester) async {
      tester.view.physicalSize = const Size(780, 1688);
      tester.view.devicePixelRatio = 2;
      final key = GlobalKey<PreviewAppState>();
      await tester.pumpWidget(
        PreviewApp(key: key, catalog: catalog, testChrome: true),
      );
      await tester.pumpAndSettle();
      Future<void> tap(String text) async {
        final target = find.text(text).last;
        await tester.ensureVisible(target);
        await tester.tap(target);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
      }

      await tap('Settings');
      expect(key.currentState!.id, 'S01');
      await tap('Appearance');
      expect(key.currentState!.id, 'S02');
      await tap('Preview Skin');
      expect(key.currentState!.id, 'S11');
      await tap('Back to Appearance');
      expect(key.currentState!.id, 'S02');
      await tester.tap(find.byTooltip('Back'));
      await tester.pumpAndSettle();
      key.currentState!.go('R02');
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(TextField), 'Kitchen edited');
      expect(find.text('Kitchen edited'), findsOneWidget);
      await tap('Cancel');
      key.currentState!.go('C01');
      await tester.pumpAndSettle();
      await tap('Mode');
      expect(key.currentState!.id, 'C02');
      await tap('Heat');
      expect(key.currentState!.choices.values.single, 1);
      await tap('Save Mode');
      expect(key.currentState!.id, 'C03');
      key.currentState!.go('H03');
      await tester.pumpAndSettle();
      await tap('Codes Do Not Match');
      expect(key.currentState!.id, 'H14');
      key.currentState!.go('A04');
      await tester.pumpAndSettle();
      final selectedChoices = find.byWidgetPredicate(
        (widget) => widget is Semantics && widget.properties.selected == true,
      );
      expect(selectedChoices, findsNWidgets(2));
      await tap('Floor lamp · Local network');
      expect(selectedChoices, findsOneWidget);
      await tap('Bedroom AC · Possible duplicate');
      expect(selectedChoices, findsNWidgets(2));
      tester.view.resetPhysicalSize();
      tester.view.resetDevicePixelRatio();
    },
  );
  testWidgets('inline steppers have separate preview-only actions', (
    tester,
  ) async {
    final key = GlobalKey<PreviewAppState>();
    await tester.pumpWidget(
      PreviewApp(key: key, catalog: catalog, initial: 'C06', testChrome: true),
    );
    await tester.pumpAndSettle();
    for (final direction in ['Increase', 'Decrease']) {
      await tester.tap(find.bySemanticsLabel('$direction Speed'));
      await tester.pumpAndSettle();
      expect(key.currentState!.id, 'C06');
      expect(
        find.text('Preview only: no device setting was changed.'),
        findsOneWidget,
      );
    }
    key.currentState!.go('C08');
    await tester.pumpAndSettle();
    for (final direction in ['Increase', 'Decrease']) {
      final controls = find.bySemanticsLabel('$direction Target');
      expect(controls, findsNWidgets(2));
      for (var i = 0; i < 2; i++) {
        await tester.ensureVisible(controls.at(i));
        await tester.tap(controls.at(i));
        await tester.pumpAndSettle();
        expect(key.currentState!.id, 'C08');
        expect(tester.takeException(), isNull);
      }
    }
  });

  testWidgets('all screens reflow at 320 width and 200 percent text', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(640, 1688);
    tester.view.devicePixelRatio = 2;
    for (final id in catalog.screens.keys.where((id) => !id.startsWith('W'))) {
      await tester.pumpWidget(
        MediaQuery(
          data: const MediaQueryData(textScaler: TextScaler.linear(2)),
          child: PreviewApp(
            key: ValueKey('large-$id'),
            catalog: catalog,
            initial: id,
            testChrome: true,
          ),
        ),
      );
      await tester.runAsync(() async {
        for (final asset in [
          'window-light-background',
          'ac',
          'lamp',
          'tv',
          'sensor',
        ]) {
          await precacheImage(
            AssetImage('assets/$asset.png'),
            tester.element(find.byType(PreviewApp)),
          );
        }
      });
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull, reason: id);
    }
    tester.view.resetPhysicalSize();
    tester.view.resetDevicePixelRatio();
  });
  testWidgets('all states reflow in landscape at 150 percent text', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1688, 780);
    tester.view.devicePixelRatio = 2;
    for (final id in catalog.screens.keys) {
      await tester.pumpWidget(
        MediaQuery(
          data: const MediaQueryData(textScaler: TextScaler.linear(1.5)),
          child: PreviewApp(
            key: ValueKey('landscape-$id'),
            catalog: catalog,
            initial: id,
            testChrome: true,
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull, reason: id);
    }
    tester.view.resetPhysicalSize();
    tester.view.resetDevicePixelRatio();
  });
  testWidgets('all forms remain editable above the keyboard', (tester) async {
    tester.view.physicalSize = const Size(780, 1688);
    tester.view.devicePixelRatio = 2;
    tester.view.viewInsets = const FakeViewPadding(bottom: 560);
    for (final id in catalog.screens.keys.where(
      (id) => (catalog.ui(id)['blocks'] as List).any((b) => b[0] == 'field'),
    )) {
      await tester.pumpWidget(
        PreviewApp(
          key: ValueKey('keyboard-$id'),
          catalog: catalog,
          initial: id,
          testChrome: true,
        ),
      );
      await tester.pumpAndSettle();
      final input = find.byType(TextField).last;
      await tester.ensureVisible(input);
      await tester.tap(input);
      await tester.enterText(input, 'Editable input');
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull, reason: id);
      expect(
        tester.getBottomRight(input).dy,
        lessThanOrEqualTo(564),
        reason: '$id input must stay above the keyboard',
      );
    }
    tester.view.resetViewInsets();
    tester.view.resetPhysicalSize();
    tester.view.resetDevicePixelRatio();
  });
  testWidgets('click all 201 explicitly simulated atlas branches', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(780, 1688);
    tester.view.devicePixelRatio = 2;
    final key = GlobalKey<PreviewAppState>();
    await tester.pumpWidget(PreviewApp(key: key, catalog: catalog));
    await tester.pumpAndSettle();
    expect(catalog.branches.length, 201);
    for (final branch in catalog.branches) {
      key.currentState!.go(branch['source']);
      await tester.pumpAndSettle();
      await tester.tap(
        find.text('UI preview · Tap to browse states · No native operations'),
      );
      await tester.pumpAndSettle();
      final button = find.byKey(
        ValueKey('branch:${branch['source']}:${branch['action']}'),
      );
      await tester.scrollUntilVisible(
        button,
        160,
        scrollable: find.byType(Scrollable).last,
      );
      await tester.tap(button);
      await tester.pumpAndSettle();
      expect(
        key.currentState!.id,
        branch['target'],
        reason: '${branch['source']}: ${branch['action']}',
      );
      expect(tester.takeException(), isNull);
    }
    tester.view.resetPhysicalSize();
    tester.view.resetDevicePixelRatio();
  });

  testWidgets(
    'repeated groups and choices retain fractional prototype geometry',
    (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: windowTheme(),
          home: Scaffold(
            body: SizedBox(
              width: 350,
              child: Column(
                children: [
                  for (final title in [
                    'Example Home',
                    'Studio account',
                    'Example TV',
                  ])
                    Rows(
                      const [
                        ['Room', 'Living room', null],
                      ],
                      title: title,
                      onAction: (_) {},
                    ),
                  ChoiceTile(
                    label: 'Living Room',
                    selected: false,
                    onTap: () {},
                  ),
                  ChoiceTile(label: 'Bedroom', selected: true, onTap: () {}),
                ],
              ),
            ),
          ),
        ),
      );
      final groups = find.byType(Rows);
      for (var i = 0; i < 3; i++) {
        expect(tester.getSize(groups.at(i)).height, closeTo(90, .001));
        expect(tester.getTopLeft(groups.at(i)).dy, closeTo(i * 90, .001));
      }
      final choices = find.byType(ChoiceTile);
      expect(tester.getSize(choices.first).height, closeTo(61.2, .001));
      expect(tester.getTopLeft(choices.last).dy, closeTo(331.2, .001));
      await tester.pumpWidget(
        MaterialApp(
          theme: windowTheme(),
          home: Scaffold(
            body: SizedBox(
              width: 180,
              child: ChoiceTile(
                label: 'Primary bedroom upstairs',
                selected: true,
                onTap: () {},
              ),
            ),
          ),
        ),
      );
      // Two lines are one centered group, including the unchanged nine-pixel gap.
      expect(
        tester.getSize(find.byType(ChoiceTile)).height,
        closeTo(79.4, .001),
      );
      final text = tester.getRect(find.text('Primary bedroom upstairs'));
      final face = tester.getRect(find.byType(InkWell));
      expect(text.center.dy, closeTo(face.center.dy, .001));
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets('long card names retain semantics without covering readings', (
    tester,
  ) async {
    const alias =
        'Upstairs bedroom climate controller with a long label for visiting family';
    const room = 'North upstairs bedroom and home office beside the landing';
    final semantics = tester.ensureSemantics();
    tester.view.physicalSize = const Size(780, 1688);
    tester.view.devicePixelRatio = 2;
    try {
      for (final (width, scale) in [(350.0, 1.0), (300.0, 1.0), (300.0, 2.0)]) {
        final actions = <String>[];
        await tester.pumpWidget(
          MaterialApp(
            theme: windowTheme(),
            home: Scaffold(
              body: MediaQuery(
                data: MediaQueryData(
                  size: const Size(390, 844),
                  devicePixelRatio: 2,
                  textScaler: TextScaler.linear(scale),
                ),
                child: SingleChildScrollView(
                  child: SizedBox(
                    width: width,
                    child: DeviceGrid(
                      keys: const ['ac'],
                      data: const [
                        [alias, room, '24°C', 'Cooling'],
                      ],
                      onAction: actions.add,
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        expect(tester.getSize(find.byType(DeviceGrid)).width, width);
        final titleBox = tester.getRect(find.text(alias));
        final roomBox = tester.getRect(find.text(room));
        final valueBox = tester.getRect(find.text('24°C'));
        expect(
          titleBox.bottom,
          lessThanOrEqualTo(roomBox.top),
          reason: 'Name covers room at width $width and scale $scale',
        );
        expect(
          roomBox.bottom,
          lessThanOrEqualTo(valueBox.top),
          reason: 'Room covers reading at width $width and scale $scale',
        );
        expect(
          find.bySemanticsLabel(RegExp(RegExp.escape('Open $alias'))),
          findsOneWidget,
        );
        expect(
          find.bySemanticsLabel(RegExp(RegExp.escape(room))),
          findsOneWidget,
        );
        await tester.ensureVisible(find.text(alias));
        await tester.tap(find.text(alias));
        expect(actions, ['open:ac']);
      }
    } finally {
      semantics.dispose();
      tester.view.resetPhysicalSize();
      tester.view.resetDevicePixelRatio();
    }
  });
  testWidgets('long information headings retain semantics without overlap', (
    tester,
  ) async {
    const alias =
        'Upstairs bedroom climate controller with a long label for visiting family';
    const room = 'North upstairs bedroom and home office beside the landing';
    final semantics = tester.ensureSemantics();
    try {
      for (final scale in [1.0, 2.0]) {
        await tester.pumpWidget(
          MaterialApp(
            theme: windowTheme(),
            home: Scaffold(
              body: MediaQuery(
                data: MediaQueryData(textScaler: TextScaler.linear(scale)),
                child: SingleChildScrollView(
                  child: SizedBox(
                    width: 300,
                    child: Blocks(
                      const [
                        ['hero', 'ac', alias, room],
                      ],
                      onAction: (_) {},
                      fields: {},
                      choices: {},
                      values: {},
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        final titleBox = tester.getRect(find.text(alias));
        final roomBox = tester.getRect(find.text(room));
        expect(titleBox.bottom, lessThanOrEqualTo(roomBox.top));
        expect(find.bySemanticsLabel(alias), findsOneWidget);
        expect(find.bySemanticsLabel(room), findsOneWidget);
      }
    } finally {
      semantics.dispose();
    }
  });
}
