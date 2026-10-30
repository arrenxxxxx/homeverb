// Actual NativeApp pages rendered with fictional clients and no provider access.
// Opt in with --dart-define=HOMEVERB_CJK_FONT=/path/to/NotoSansCJK-Regular.ttc.
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/device_page.dart';
import 'package:homeverb_mobile/device_detail_controls.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart';
import 'package:homeverb_mobile/skin.dart';
import 'climate_controls_test.dart' show ClimateFixture;
import 'controls_test.dart' show setter;
import 'device_detail_controls_test.dart' show DishwasherFixture, dishwasher;
import 'refrigerator_modes_test.dart' show ReferenceFridgeFixture;

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const path = String.fromEnvironment('HOMEVERB_CJK_FONT');
  setUpAll(() async {
    if (path.isEmpty) return;
    const explicitBoldPath = String.fromEnvironment('HOMEVERB_CJK_BOLD_FONT');
    final boldPath = explicitBoldPath.isNotEmpty
        ? explicitBoldPath
        : '${File(path).parent.path}/NotoSansCJK-Bold.ttc';
    if (!File(path).existsSync() || !File(boldPath).existsSync()) {
      throw StateError('Provide regular and bold Noto CJK font paths.');
    }
    // Keep the production Latin/numeral face, with CJK in its own fallback
    // family. Never register Noto as Inter: that changes every numeral's shape.
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    await (FontLoader('Noto Sans CJK SC')
          ..addFont(File(path).readAsBytes().then(ByteData.sublistView))
          ..addFont(File(boldPath).readAsBytes().then(ByteData.sublistView)))
        .load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
  });
  test('capture keeps Inter numerals with a separate CJK fallback', () {
    final style = windowTheme().textTheme.bodyLarge!;
    expect(style.fontFamily, 'Inter');
    expect(style.fontFamilyFallback, contains('Noto Sans CJK SC'));
    double width(TextStyle textStyle) {
      final painter = TextPainter(
        text: TextSpan(text: '24°C 35% HomeVerb', style: textStyle),
        textDirection: TextDirection.ltr,
      )..layout();
      final result = painter.width;
      painter.dispose();
      return result;
    }

    expect(
      (width(
                const TextStyle(
                  fontFamily: 'Inter',
                  fontSize: 32,
                  fontWeight: FontWeight.w400,
                ),
              ) -
              width(
                const TextStyle(
                  fontFamily: 'Inter',
                  fontSize: 32,
                  fontWeight: FontWeight.w600,
                ),
              ))
          .abs(),
      greaterThan(.1),
      reason: 'The regular and semibold assets must remain distinct faces',
    );
    for (final weight in [FontWeight.w400, FontWeight.w600]) {
      final inter = TextStyle(
        fontFamily: 'Inter',
        fontSize: 32,
        fontWeight: weight,
      );
      final withFallback = inter.copyWith(
        fontFamilyFallback: windowFontFallback,
      );
      final noto = TextStyle(
        fontFamily: 'Noto Sans CJK SC',
        fontSize: 32,
        fontWeight: weight,
      );
      expect(width(withFallback), closeTo(width(inter), .01));
      expect((width(withFallback) - width(noto)).abs(), greaterThan(.1));
    }
  }, skip: path.isEmpty);
  for (final scenario in [
    'ac',
    'fridge-readonly',
    'dishwasher-standby',
    'dishwasher-running',
  ]) {
    testWidgets('capture actual fictional device page: $scenario', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(430, 932);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      LocaleSettings.setLocale(AppLocale.zhCn);
      addTearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      final app = GlobalKey<NativeAppState>(), boundary = GlobalKey();
      final preferences = LocalPreferences.memory()..values['locale'] = 'zh-CN';
      ClimateFixture? climate;
      ReferenceFridgeFixture? fridge;
      DishwasherFixture? wash;
      late HomeController controller;
      late String deviceId;
      if (scenario == 'ac') {
        climate = ClimateFixture();
        climate.device['alias'] = AppLocale.zhCn.translations.mobileBedroomAc;
        climate.capabilities.first['properties']['target_temperature'] = {
          'type': 'number',
          'readable': true,
          'writable': true,
          'unit': 'celsius',
        };
        climate.capabilities.first['actions']['set'] = setter(
          {
            'target_temperature': {
              'type': 'number',
              'minimum': 18,
              'maximum': 27,
              'multipleOf': .5,
            },
          },
          required: ['target_temperature'],
        );
        climate.observations['climate.target_temperature'] = 24;
        final vendor = climate.capabilities.last;
        for (final entry in {
          'horizontal_swing': false,
          'vertical_swing': true,
        }.entries) {
          vendor['properties'][entry.key] = {
            'type': 'boolean',
            'readable': true,
            'writable': true,
          };
          vendor['actions']['set_${entry.key}'] = setter(
            {
              entry.key: {'type': 'boolean'},
            },
            required: [entry.key],
          );
          climate.observations['${vendor['id']}.${entry.key}'] = entry.value;
        }
        for (final field in [
          ('horizontal_direction', ['left', 'center', 'right'], 'center'),
          ('vertical_direction', ['upper', 'lower'], 'upper'),
        ]) {
          final schema = {'type': 'string', 'enum': field.$2};
          vendor['properties'][field.$1] = {
            ...schema,
            'readable': true,
            'writable': true,
          };
          vendor['actions']['set_${field.$1}'] = setter(
            {field.$1: schema},
            required: [field.$1],
          );
          climate.observations['${vendor['id']}.${field.$1}'] = field.$3;
        }
        controller = HomeController(climate, preferences);
        deviceId = climate.device['id'];
      } else if (scenario == 'fridge-readonly') {
        fridge = ReferenceFridgeFixture();
        fridge.device['alias'] =
            AppLocale.zhCn.translations.mobilePreviewKitchenRefrigerator;
        for (final capability in fridge.capabilities) {
          capability['actions'] = <String, dynamic>{};
          for (final property in (capability['properties'] as Map).values) {
            property['writable'] = false;
          }
        }
        controller = HomeController(fridge, preferences);
        deviceId = fridge.device['id'];
      } else {
        wash = DishwasherFixture();
        wash.device['alias'] =
            AppLocale.zhCn.translations.mobilePreviewKitchenDishwasher;
        wash.observations['$dishwasher.work_status'] =
            scenario == 'dishwasher-running' ? 'work' : 'power_on';
        controller = HomeController(wash, preferences);
        deviceId = wash.device['id'];
      }
      addTearDown(controller.dispose);
      await tester.pumpWidget(
        RepaintBoundary(
          key: boundary,
          child: TranslationProvider(
            child: Directionality(
              textDirection: TextDirection.ltr,
              child: Column(
                children: [
                  Expanded(
                    child: NativeApp(
                      key: app,
                      catalog: catalog,
                      controller: controller,
                    ),
                  ),
                  // Test-only disclosure outside the production page footprint.
                  const Material(
                    color: Color(0xfff8f6f0),
                    child: SizedBox(
                      width: double.infinity,
                      height: 24,
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
      );
      await tester.pumpAndSettle();
      await controller.selectDevice(deviceId);
      await app.currentState!.go(scenario == 'fridge-readonly' ? 'C08' : 'C01');
      await tester.pumpAndSettle();
      expect(find.byType(DevicePage), findsOneWidget);
      final calls = climate?.calls ?? fridge?.calls ?? wash!.calls;
      final before = calls.length;
      if (scenario == 'dishwasher-standby') {
        for (final key in ['wash-program:eco_wash', 'wash-additional:9']) {
          final option = find.byKey(ValueKey(key));
          await tester.ensureVisible(option);
          await tester.tap(option);
          await tester.pumpAndSettle();
        }
      }
      final scrollable = find
          .descendant(
            of: find.byType(DevicePage),
            matching: find.byType(Scrollable),
          )
          .first;
      final position = tester.state<ScrollableState>(scrollable).position;
      position.jumpTo(0);
      await tester.pumpAndSettle();
      if (scenario == 'dishwasher-running') {
        final context = tester.element(find.byType(DeviceDetailControls));
        expect(find.textContaining('Eco wash'), findsNothing);
        expect(
          find.text(
            localize(
              context,
              UiMessage('Current program: {program}', {
                'program': localize(context, 'Eco wash'),
              }),
            ),
          ),
          findsOneWidget,
        );
      }
      expect(
        calls.length,
        before,
        reason: 'Rendering and local selection must not issue client calls',
      );
      expect(wash?.prepared, isNull);
      expect(tester.takeException(), isNull);
      Future<void> capture(String suffix) async {
        await tester.runAsync(() async {
          final image =
              await (boundary.currentContext!.findRenderObject()!
                      as RenderRepaintBoundary)
                  .toImage(pixelRatio: 2);
          final data = await image.toByteData(format: ui.ImageByteFormat.png);
          final directory = Directory('build/verification/device-design')
            ..createSync(recursive: true);
          File('${directory.path}/$scenario$suffix.png').writeAsBytesSync(
            data!.buffer.asUint8List(data.offsetInBytes, data.lengthInBytes),
          );
          image.dispose();
        });
      }

      await capture('');
      final height = (932 + position.maxScrollExtent).ceilToDouble();
      expect(height, lessThan(12000));
      tester.view.physicalSize = Size(430, height);
      await tester.pumpAndSettle();
      expect(position.maxScrollExtent, lessThanOrEqualTo(1));
      expect(tester.takeException(), isNull);
      expect(calls.length, before);
      await capture('-full');
      await tester.pumpWidget(const SizedBox());
    }, skip: path.isEmpty);
  }
}
