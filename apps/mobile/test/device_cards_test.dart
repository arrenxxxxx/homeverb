import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/blocks.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/device_presentation.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/skin.dart';
import 'accounts_test.dart' show AccountPreferences;
import 'climate_controls_test.dart' show ClimateFixture;
import 'session_permissions_test.dart' show inspection, PermissionCore;

class CardFixture extends ClimateFixture {
  Completer<void>? postReadGate;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'read_state' && fields['cached'] != true) {
      await postReadGate?.future;
    }
    return super.call(method, fields);
  }
}

class NativeCardFixture extends CardFixture implements NativeSessionClient {
  List<Json> grants = [];
  @override
  Set<String> get methods => {'inspect_session_grants'};
  @override
  bool apiAtLeast(int minor) => minor <= 22;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'inspect_session_grants') {
      calls.add({'method': method, ...fields});
      final connection = (device['connections'] as List).singleWhere(
        (c) => c['id'] == fields['connection_id'],
      );
      return {
        'device_id': device['id'],
        'connection_id': connection['id'],
        'transport': connection['transport'],
        'source': 'session_policy',
        'provider_authorization': 'not_checked',
        'grants': grants,
      };
    }
    return super.call(method, fields);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
  });
  setUp(() => LocaleSettings.setLocale(AppLocale.enUs));
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));

  Json appliance(List<Json> components) => {
    'alias': 'Renamable appliance',
    'components': components,
  };

  test(
    'artwork follows component metadata rather than aliases or identifiers',
    () {
      final fridge = appliance([
        {
          'id': 'fridge',
          'capabilities': ['power', 'climate'],
        },
        {
          'id': 'freezer',
          'capabilities': ['power', 'climate'],
        },
      ]);
      final dishwasher = appliance([
        {
          'id': 'main',
          'capabilities': ['vendor.midea.dishwasher'],
        },
      ]);
      expect(deviceArtwork(fridge), 'refrigerator');
      expect(deviceArtwork(dishwasher), 'dishwasher');
      fridge['alias'] = 'Bedroom AC';
      dishwasher['alias'] = 'Light';
      expect(deviceArtwork(fridge), 'refrigerator');
      expect(deviceArtwork(dishwasher), 'dishwasher');
      expect(deviceArtwork(appliance([])), 'appliance');
    },
  );

  test(
    'native cards intersect exact grants and route without provider work',
    () async {
      final core = NativeCardFixture();
      final c = HomeController(core, AccountPreferences());
      final id = core.device['id'] as String;
      expect(await c.initialize(), true);
      expect(c.cardPowerChoices(id), isEmpty);
      core.grants = [
        {
          'component_id': 'main',
          'capability': 'power',
          'action': 'set',
          'argument_schema': {
            'const': {'state': false},
          },
        },
      ];
      await c.loadLocal();
      expect(c.cardPowerChoices(id), {false});
      expect(c.snapshots[id]!['observations']['power.state']['value'], true);
      core.grants.single['argument_schema'] = {
        'not': {
          'const': {'state': true},
        },
      };
      await c.loadLocal();
      expect(c.cardPowerChoices(id), isEmpty);
      expect(
        core.calls
            .where((v) => v['method'] == 'read_state')
            .every((v) => v['cached'] == true),
        true,
      );
      expect(
        core.calls.any(
          (v) => [
            'plan',
            'execute',
            'discover',
            'refresh_account',
          ].contains(v['method']),
        ),
        false,
      );
      c.dispose();
    },
  );

  test(
    'reviewed options allow card preparation without granting or refreshing',
    () async {
      final inspected = inspection();
      final core = PermissionCore(inspected);
      var reopened = false;
      final c = HomeController(
        core,
        AccountPreferences(),
        reopen: (_) async {
          reopened = true;
          return core;
        },
      );
      expect(await c.initialize(), true);
      expect(c.cardPowerChoices(inspected['device_id']), {false, true});
      expect(reopened, false);
      expect(
        core.requests.any(
          (v) => ['refresh_account', 'plan', 'execute'].contains(v['method']),
        ),
        false,
      );
      await c.preferences.save({
        'device_routes': {inspected['device_id']: 'lan'},
      });
      expect(c.cardPowerChoices(inspected['device_id']), isEmpty);
      c.dispose();
    },
  );

  test(
    'component-only power and read-only descriptors have no card shortcut',
    () async {
      final core = CardFixture();
      final c = HomeController(core, AccountPreferences());
      final id = core.device['id'] as String;
      core.capabilities
          .singleWhere((cap) => cap['id'] == 'power')['actions']
          .clear();
      await c.initialize();
      expect(c.cardPowerChoices(id), isEmpty);
      core.device['components'] = [
        {
          'id': 'fridge',
          'capabilities': ['power', 'climate'],
        },
        {
          'id': 'freezer',
          'capabilities': ['power', 'climate'],
        },
      ];
      core.calls.clear();
      await c.loadLocal();
      expect(c.cardPowerChoices(id), isEmpty);
      expect(core.calls.map((v) => v['method']), [
        'list_rooms',
        'list_device_cards',
      ]);
      c.dispose();
    },
  );

  test('foreign and late card inspections cannot publish choices', () async {
    final inspected = inspection();
    final core = PermissionCore(inspected);
    final c = HomeController(
      core,
      AccountPreferences(),
      reopen: (_) async => core,
    );
    core.inspectionGate = Completer<Json>()
      ..complete({...inspected, 'connection_id': 'foreign-local-connection'});
    expect(await c.initialize(), true);
    expect(c.cardPowerChoices(inspected['device_id']), isEmpty);
    core.inspectionGate = Completer<Json>();
    final pending = c.initialize();
    await Future<void>.delayed(Duration.zero);
    c.setForeground(false);
    core.inspectionGate!.complete(inspected);
    expect(await pending, false);
    c.setForeground(true);
    expect(c.cardPowerChoices(inspected['device_id']), isEmpty);
    c.dispose();
  });

  Future<(CardFixture, HomeController, GlobalKey<NativeAppState>)> home(
    WidgetTester tester,
  ) async {
    tester.view.physicalSize = const Size(780, 1688);
    tester.view.devicePixelRatio = 2;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final core = CardFixture();
    final c = HomeController(core, AccountPreferences());
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    final key = GlobalKey<NativeAppState>();
    await tester.pumpWidget(
      TranslationProvider(
        child: NativeApp(key: key, catalog: catalog, controller: c),
      ),
    );
    await tester.pumpAndSettle();
    return (core, c, key);
  }

  testWidgets(
    'home power stays observed and locally pending through command and read',
    (tester) async {
      final (core, c, app) = await home(tester);
      final button = find.byKey(ValueKey('card-power:${core.device['id']}'));
      DevicePresentation presentation() =>
          tester.widget<DeviceTile>(find.byType(DeviceTile)).presentation!;
      Color? cardColor() => tester
          .widget<Surface>(
            find.descendant(
              of: find.byType(DeviceTile),
              matching: find.byType(Surface),
            ),
          )
          .backgroundColor;
      expect(presentation().power, true);
      expect(cardColor(), Colors.white);
      core.executionGate = Completer<void>();
      core.postReadGate = Completer<void>();
      core.calls.clear();
      await tester.tap(button);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 200));
      expect(
        find.descendant(of: button, matching: find.byType(OperationPulse)),
        findsOneWidget,
      );
      expect(find.byType(OperationProgress), findsNothing);
      expect(presentation().power, true);
      expect(cardColor(), Colors.white);
      expect(tester.widget<InkWell>(button).onTap, isNull);
      await app.currentState!.action('power:${core.device['id']}');
      expect(core.calls.where((v) => v['method'] == 'execute'), hasLength(1));
      core.executionGate!.complete();
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 100));
      expect(presentation().updating, true);
      expect(presentation().power, true);
      expect(cardColor(), Colors.white);
      core.postReadGate!.complete();
      await tester.pumpAndSettle();
      expect(presentation().power, false);
      expect(cardColor(), const Color(0xfff0f1ee));
      expect(presentation().updating, false);
      expect(app.currentState!.id, 'D01');
      expect(c.receipt!['status'], 'accepted');
      expect(find.byType(SnackBar), findsNothing);
      expect(
        core.calls.where(
          (v) => v['method'] == 'read_state' && v['cached'] != true,
        ),
        hasLength(1),
      );
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets(
    'unknown observation disables power without inventing an on state',
    (tester) async {
      final (core, c, app) = await home(tester);
      c.snapshots.clear();
      app.currentState!.changed();
      await tester.pump();
      final button = find.byKey(ValueKey('card-power:${core.device['id']}'));
      expect(tester.widget<InkWell>(button).onTap, isNull);
      expect(
        tester
            .widget<Surface>(
              find.descendant(
                of: find.byType(DeviceTile),
                matching: find.byType(Surface),
              ),
            )
            .backgroundColor,
        const Color(0xfff0f1ee),
      );
      core.calls.clear();
      await app.currentState!.action('power:${core.device['id']}');
      expect(core.calls, isEmpty);
      await tester.pumpWidget(const SizedBox());
    },
  );

  for (final (width, scale, contrast) in [
    (390.0, 1.0, false),
    (320.0, 2.0, true),
  ]) {
    testWidgets(
      'distinct artwork and power states fit $width at $scale text scale',
      (tester) async {
        tester.view.physicalSize = Size(width * 2, 2100);
        tester.view.devicePixelRatio = 2;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final capture = GlobalKey();
        final presentations = <String, DevicePresentation>{
          for (final kind in ['ac', 'refrigerator', 'dishwasher'])
            kind: DevicePresentation(
              artwork: kind,
              power: kind == 'ac',
              showPower: kind == 'ac',
              powerEnabled: true,
              updating: false,
            ),
          'off': const DevicePresentation(
            artwork: 'ac',
            power: false,
            showPower: true,
            powerEnabled: true,
            updating: false,
          ),
        };
        await tester.pumpWidget(
          MaterialApp(
            home: MediaQuery(
              data: MediaQueryData(
                size: Size(width, 1050),
                devicePixelRatio: 2,
                textScaler: TextScaler.linear(scale),
                highContrast: contrast,
              ),
              child: Scaffold(
                body: SingleChildScrollView(
                  child: RepaintBoundary(
                    key: capture,
                    child: Padding(
                      padding: const EdgeInsets.all(20),
                      child: DeviceGrid(
                        keys: presentations.keys.toList(),
                        presentations: presentations,
                        data: [
                          for (final kind in presentations.keys)
                            [kind, 'Unassigned', '—', ''],
                        ],
                        onAction: (_) {},
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        await tester.runAsync(() async {
          final context = tester.element(find.byType(DeviceGrid));
          await Future.wait([
            for (final image in tester.widgetList<Image>(find.byType(Image)))
              precacheImage(image.image, context),
          ]);
        });
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        for (final tile in find.byType(DeviceTile).evaluate()) {
          final surface = find.descendant(
            of: find.byWidget(tile.widget),
            matching: find.byType(Surface),
          );
          final decoration =
              tester
                      .widget<DecoratedBox>(
                        find
                            .descendant(
                              of: surface,
                              matching: find.byType(DecoratedBox),
                            )
                            .first,
                      )
                      .decoration
                  as BoxDecoration;
          final isOn = (tile.widget as DeviceTile).presentation!.power == true;
          expect(
            decoration.color,
            contrast || isOn ? Colors.white : const Color(0xfff0f1ee),
          );
          expect(decoration.gradient, isNull);
          for (final text in tester.widgetList<Text>(
            find.descendant(of: surface, matching: find.byType(Text)),
          )) {
            final foreground = text.style!.color!;
            expect(
              (decoration.color!.computeLuminance() + .05) /
                  (foreground.computeLuminance() + .05),
              greaterThanOrEqualTo(4.5),
            );
          }
        }
        expect(
          find.byKey(const ValueKey('card-power:refrigerator')),
          findsNothing,
        );
        expect(
          find.byKey(const ValueKey('card-power:dishwasher')),
          findsNothing,
        );
        expect(find.byTooltip('Turn Off'), findsOneWidget);
        expect(find.byTooltip('Turn On'), findsOneWidget);
        final on = find.descendant(
          of: find.byKey(const ValueKey('card-power:ac')),
          matching: find.byWidgetPredicate(
            (w) => w is Container && w.decoration is BoxDecoration,
          ),
        );
        final off = find.descendant(
          of: find.byKey(const ValueKey('card-power:off')),
          matching: find.byWidgetPredicate(
            (w) => w is Container && w.decoration is BoxDecoration,
          ),
        );
        expect(
          (tester.widget<Container>(on).decoration as BoxDecoration).color,
          isNot(
            (tester.widget<Container>(off).decoration as BoxDecoration).color,
          ),
        );
        final assets = tester.widgetList<Image>(find.byType(Image)).map((w) {
          final provider = w.image is ResizeImage
              ? (w.image as ResizeImage).imageProvider
              : w.image;
          return (provider as AssetImage).assetName;
        });
        expect(
          assets,
          containsAll([
            'assets/refrigerator-v1.png',
            'assets/dishwasher-v1.png',
          ]),
        );
        const path = String.fromEnvironment('HOMEVERB_CARD_CAPTURE');
        if (path.isNotEmpty && scale == 1) {
          final render =
              capture.currentContext!.findRenderObject()
                  as RenderRepaintBoundary;
          await tester.runAsync(() async {
            final image = await render.toImage(pixelRatio: 2);
            final bytes = (await image.toByteData(
              format: ui.ImageByteFormat.png,
            ))!.buffer.asUint8List();
            await File(path).writeAsBytes(bytes);
            image.dispose();
          });
        }
        await tester.pumpWidget(const SizedBox());
      },
    );
  }
}
