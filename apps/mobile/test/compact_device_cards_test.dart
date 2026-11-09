import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/blocks.dart';
import 'package:homeverb_mobile/device_presentation.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/screens.dart';
import 'package:homeverb_mobile/skin.dart';

const _ids = [
  'ac-on',
  'ac-off',
  'ac-unknown',
  'ac-disabled',
  'fridge',
  'washer',
];

Map<String, DevicePresentation> _presentations({bool updating = false}) => {
  for (final id in _ids)
    id: DevicePresentation(
      artwork: id == 'fridge'
          ? 'refrigerator'
          : id == 'washer'
          ? 'dishwasher'
          : 'ac',
      power: id == 'ac-unknown' ? null : id == 'ac-on',
      showPower: id.startsWith('ac'),
      powerEnabled: !{'ac-unknown', 'ac-disabled'}.contains(id),
      updating: updating && id == 'ac-on',
    ),
};

List<List<Object>> _data({bool chinese = false, bool caption = false}) {
  final t = AppLocale.zhCn.translations;
  return [
    [
      chinese ? t.mobileBedroomAc : 'Bedroom AC',
      'Bedroom',
      '24.0°C',
      caption ? 'Read-only' : '',
    ],
    [
      chinese ? '${t.mobileBedroomAc} 02' : 'Study AC',
      'Unassigned',
      '25.0°C',
      '',
    ],
    [chinese ? '${t.mobileBedroomAc} 03' : 'Guest AC', 'Unassigned', '—', ''],
    [
      chinese ? '${t.mobileBedroomAc} 04' : 'Office AC',
      'Unassigned',
      '23.0°C',
      '',
    ],
    [
      chinese ? t.mobilePreviewKitchenRefrigerator : 'Kitchen refrigerator',
      'Kitchen',
      '4.0°C',
      '',
    ],
    [
      chinese ? t.mobilePreviewKitchenDishwasher : 'Kitchen dishwasher',
      'Kitchen',
      'Powered off',
      '',
    ],
  ];
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const font = String.fromEnvironment('HOMEVERB_CJK_FONT');
  const boldFont = String.fromEnvironment('HOMEVERB_CJK_BOLD_FONT');
  late ScreenCatalog catalog;
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
    if (font.isNotEmpty) {
      await (FontLoader('Noto Sans CJK SC')
            ..addFont(File(font).readAsBytes().then(ByteData.sublistView))
            ..addFont(
              File(
                boldFont.isNotEmpty
                    ? boldFont
                    : '${File(font).parent.path}/NotoSansCJK-Bold.ttc',
              ).readAsBytes().then(ByteData.sublistView),
            ))
          .load();
    }
    catalog = await ScreenCatalog.load();
  });
  setUp(() => LocaleSettings.setLocale(AppLocale.enUs));
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));

  Future<void> grid(
    WidgetTester tester, {
    double width = 390,
    double scale = 1,
    bool highContrast = false,
    String skin = 'sky_blue',
    bool updating = false,
    bool caption = false,
    void Function(String)? action,
  }) async {
    tester.view.physicalSize = Size(width, 1000);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    await tester.pumpWidget(
      MaterialApp(
        theme: windowTheme(skinId: skin, highContrast: highContrast),
        home: MediaQuery(
          data: MediaQueryData(
            size: Size(width, 1000),
            textScaler: TextScaler.linear(scale),
            highContrast: highContrast,
          ),
          child: Scaffold(
            body: SingleChildScrollView(
              child: Padding(
                padding: const EdgeInsets.all(14),
                child: DeviceGrid(
                  keys: _ids,
                  data: _data(caption: caption),
                  presentations: _presentations(updating: updating),
                  onAction: action ?? (_) {},
                ),
              ),
            ),
          ),
        ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 200));
  }

  testWidgets(
    'blue cards fit three compact rows and keep warm layout unchanged',
    (tester) async {
      await grid(tester);
      final cards = find.byType(DeviceTile);
      final rects = [for (var i = 0; i < 6; i++) tester.getRect(cards.at(i))];
      expect(rects[0].height, inInclusiveRange(150, 180));
      expect(rects[0].top, rects[1].top);
      expect(rects[2].left, rects[0].left);
      expect(rects[2].top - rects[0].bottom, closeTo(10, .01));
      expect(rects[1].left - rects[0].right, closeTo(10, .01));
      expect(find.text('24.0°C'), findsOneWidget);
      expect(find.text('—'), findsOneWidget);
      expect(tester.takeException(), isNull);
      await grid(tester, skin: 'window_light');
      expect(tester.getSize(find.byType(DeviceTile).first).height, 233);
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets('compact card opens directly and has no extra menu gestures', (
    tester,
  ) async {
    final actions = <String>[];
    await grid(tester, action: actions.add);
    final power = find.byKey(const ValueKey('card-power:ac-on'));
    final open = find.byKey(const ValueKey('card-open:ac-on'));
    expect(tester.getSize(power).width, greaterThanOrEqualTo(48));
    expect(tester.getSize(power).height, greaterThanOrEqualTo(48));
    expect(find.byKey(const ValueKey('card-menu:ac-on')), findsNothing);
    await tester.tap(power);
    await tester.pump();
    expect(actions, ['power:ac-on']);
    await tester.tapAt(tester.getTopLeft(open) + const Offset(25, 25));
    await tester.pump();
    expect(actions.last, 'open:ac-on');
    await tester.longPressAt(tester.getTopLeft(open) + const Offset(25, 25));
    await tester.pump();
    expect(actions.where((v) => v.startsWith('menu:')), isEmpty);
    expect(tester.takeException(), isNull);
  });

  testWidgets(
    'unknown, disabled and pending power remain disabled without hiding observations',
    (tester) async {
      final actions = <String>[];
      await grid(tester, updating: true, caption: true, action: actions.add);
      for (final id in ['ac-on', 'ac-unknown', 'ac-disabled']) {
        final target = find.byKey(ValueKey('card-power:$id'));
        expect(tester.widget<InkWell>(target).onTap, isNull);
        await tester.tap(target);
      }
      expect(actions, isEmpty);
      expect(find.byType(OperationPulse), findsOneWidget);
      final powerSemantics = tester
          .widgetList<Semantics>(
            find.ancestor(
              of: find.byKey(const ValueKey('card-power:ac-on')),
              matching: find.byType(Semantics),
            ),
          )
          .singleWhere((s) => s.properties.label == 'Power Bedroom AC');
      expect(powerSemantics.properties.enabled, isFalse);
      expect(powerSemantics.properties.toggled, isTrue);
      expect(powerSemantics.properties.value, 'Updating device…');
      expect(find.text('24.0°C'), findsOneWidget);
      expect(find.text('Read-only'), findsOneWidget);
      expect(find.byKey(const ValueKey('card-power:fridge')), findsNothing);
      expect(find.byKey(const ValueKey('card-power:washer')), findsNothing);
      final off = find.byKey(const ValueKey('card-power:ac-off'));
      expect(tester.widget<InkWell>(off).onTap, isNotNull);
      await tester.tap(off);
      expect(actions, ['power:ac-off']);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets('blue power faces are pale but retain readable distinct states', (
    tester,
  ) async {
    await grid(tester);
    Color face(String id) {
      final containers = tester.widgetList<Container>(
        find.descendant(
          of: find.byKey(ValueKey('card-power:$id')),
          matching: find.byType(Container),
        ),
      );
      return containers
          .map((c) => c.decoration)
          .whereType<BoxDecoration>()
          .singleWhere((d) => d.shape == BoxShape.circle)
          .color!;
    }

    final on = face('ac-on'), off = face('ac-off');
    expect(on, isNot(off));
    expect(on.computeLuminance(), greaterThan(.55));
    expect(off.computeLuminance(), greaterThan(.55));
    final glyph = tester.widget<Glyph>(
      find.descendant(
        of: find.byKey(const ValueKey('card-power:ac-on')),
        matching: find.byType(Glyph),
      ),
    );
    final ratio =
        (on.computeLuminance() + .05) / (glyph.color.computeLuminance() + .05);
    expect(ratio, greaterThanOrEqualTo(4.5));
    final offTile = find.byWidgetPredicate(
      (w) => w is DeviceTile && w.deviceKey == 'ac-off',
    );
    final offSurface = tester.widget<Surface>(
      find.descendant(of: offTile, matching: find.byType(Surface)),
    );
    expect(offSurface.backgroundColor, isNot(const Color(0xfff0f1ee)));
    expect(tester.takeException(), isNull);
  });

  testWidgets('disabled power has its own disabled semantics node', (
    tester,
  ) async {
    final handle = tester.ensureSemantics();
    try {
      await grid(tester);
      final node = tester.getSemantics(
        find.byKey(const ValueKey('card-power:ac-unknown')),
      );
      final data = node.getSemanticsData();
      expect(data.label, contains('Power Guest AC'));
      expect(data.label, isNot(contains('Open Guest AC')));
      expect(data.flagsCollection.isEnabled, ui.Tristate.isFalse);
      expect(data.flagsCollection.isButton, isTrue);
      expect(data.value, 'Powered off');
      expect(data.flagsCollection.isToggled, ui.Tristate.isFalse);
      final shortcut = find.byKey(const ValueKey('card-power:ac-unknown'));
      expect(
        find.descendant(
          of: shortcut,
          matching: find.byIcon(Icons.power_off_outlined),
        ),
        findsOneWidget,
      );
      expect(find.byTooltip('Power unknown'), findsNothing);
      expect(tester.takeException(), isNull);
    } finally {
      handle.dispose();
    }
  });

  for (final contrast in [false, true]) {
    testWidgets(
      'compact cards reflow with 320px doubled text (contrast=$contrast)',
      (tester) async {
        await grid(
          tester,
          width: 320,
          scale: 2,
          highContrast: contrast,
          caption: true,
        );
        final cards = find.byType(DeviceTile);
        expect(
          tester.getRect(cards.at(1)).left,
          tester.getRect(cards.at(0)).left,
        );
        expect(
          tester.getRect(cards.at(1)).top,
          greaterThan(tester.getRect(cards.at(0)).bottom),
        );
        for (var i = 0; i < 6; i++) {
          await tester.ensureVisible(cards.at(i));
          await tester.pump();
          expect(tester.takeException(), isNull);
        }
        expect(find.text('Read-only'), findsOneWidget);
      },
    );
  }

  for (final size in [const Size(430, 956), const Size(360, 800)]) {
    for (final skin in ['sky_blue', 'window_light']) {
      testWidgets(
        'capture compact home $skin ${size.width.toInt()}x${size.height.toInt()}',
        (tester) async {
          tester.view.physicalSize = size;
          tester.view.devicePixelRatio = 1;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          LocaleSettings.setLocale(AppLocale.zhCn);
          final capture = GlobalKey();
          await tester.pumpWidget(
            RepaintBoundary(
              key: capture,
              child: TranslationProvider(
                child: MaterialApp(
                  theme: windowTheme(skinId: skin),
                  debugShowCheckedModeBanner: false,
                  home: Material(
                    child: Column(
                      children: [
                        Expanded(
                          child: MediaQuery(
                            data: MediaQueryData(
                              size: size,
                              padding: const EdgeInsets.only(
                                top: 24,
                                bottom: 16,
                              ),
                            ),
                            child: ScreenView(
                              id: 'D01',
                              catalog: catalog,
                              onAction: (_) {},
                              fields: {},
                              choices: {},
                              values: {},
                              home: DeviceHome(
                                mode: 'all',
                                onAction: (_) {},
                                roomNames: const ['All', 'Bedroom', 'Kitchen'],
                                cards: DeviceGrid(
                                  keys: _ids,
                                  data: _data(chinese: true),
                                  presentations: _presentations(),
                                  onAction: (_) {},
                                ),
                              ),
                            ),
                          ),
                        ),
                        const Material(
                          color: Colors.white,
                          child: SizedBox(
                            height: 24,
                            width: double.infinity,
                            child: Center(
                              child: Copy(
                                'Interface preview · Fictional data',
                                size: 10,
                              ),
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          );
          await tester.pumpAndSettle();
          await tester.runAsync(() async {
            final context = tester.element(find.byType(DeviceGrid));
            for (final image in tester.widgetList<Image>(find.byType(Image))) {
              await precacheImage(image.image, context);
            }
          });
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull);
          await tester.runAsync(() async {
            final image =
                await (capture.currentContext!.findRenderObject()!
                        as RenderRepaintBoundary)
                    .toImage(pixelRatio: 2);
            final bytes = (await image.toByteData(
              format: ui.ImageByteFormat.png,
            ))!;
            final directory = Directory('build/verification/compact-home')
              ..createSync(recursive: true);
            File(
              '${directory.path}/$skin-${size.width.toInt()}x${size.height.toInt()}.png',
            ).writeAsBytesSync(
              bytes.buffer.asUint8List(
                bytes.offsetInBytes,
                bytes.lengthInBytes,
              ),
            );
            image.dispose();
          });
          await tester.pumpWidget(const SizedBox());
        },
        skip: font.isEmpty,
      );
    }
  }
}
