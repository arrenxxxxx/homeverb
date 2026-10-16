import 'dart:io';
import 'dart:async';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/skin.dart';
import 'component_test.dart' show CompartmentFixture;

class OverviewFixture extends CompartmentFixture {
  final missingCaches = <String>{};
  final failedReads = <String>{};
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'read_state' &&
        (fields['cached'] == true ? missingCaches : failedReads).contains(
          fields['component'],
        )) {
      calls.add({'method': method, ...fields});
      throw NativeFailure(
        'STATE_UNAVAILABLE',
        '${fields['component']} fixture reading is unavailable.',
      );
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
  });
  test(
    'overview cache reads, explicit targets and partial refresh stay component scoped',
    () async {
      final core = OverviewFixture();
      final c = HomeController(
        core,
        LocalPreferences(File('/unused/overview.json')),
      );
      await c.initialize();
      await c.selectDevice(core.device['id']);
      expect(c.hasCompartmentOverview, true);
      expect(
        core.calls
            .where((call) => call['method'] == 'read_state')
            .map((call) => call['component']),
        ['fridge', 'freezer'],
      );
      expect(
        core.calls
            .where((call) => call['method'] == 'read_state')
            .every((call) => call['cached'] == true),
        true,
      );
      expect(c.component, 'fridge');
      expect(
        await c.execute('climate', 'set', {
          'target_temperature': -19,
        }, componentId: 'freezer'),
        true,
      );
      expect(c.component, 'fridge');
      expect(c.observed('climate.target_temperature'), 4);
      expect(
        c.componentSnapshot(
          'freezer',
        )!['observations']['climate.target_temperature']['value'],
        -18,
      );
      expect(core.calls.last['command']['component_id'], 'freezer');
      final writes = core.receipts.length;
      core.targets['fridge'] = 6;
      core.failedReads.add('freezer');
      expect(await c.refreshComponents(), false);
      expect(c.observed('climate.target_temperature'), 6);
      expect(
        c.componentSnapshot(
          'freezer',
        )!['observations']['climate.target_temperature']['value'],
        -18,
      );
      expect(c.componentReadError('freezer'), isA<NativeFailure>());
      expect(c.componentReadError('fridge'), isNull);
      core.failedReads.clear();
      expect(await c.refreshComponents(), true);
      expect(c.componentReadError('freezer'), isNull);
      expect(
        c.componentSnapshot(
          'freezer',
        )!['observations']['climate.target_temperature']['value'],
        -19,
      );
      expect(core.receipts.length, writes);
      expect(
        await c.execute('climate', 'set', {
          'target_temperature': 1,
        }, componentId: 'missing'),
        false,
      );
      expect(core.receipts.length, writes);
      expect(c.component, 'fridge');
    },
  );

  test(
    'cancelled or backgrounded refresh stops before the next component',
    () async {
      for (final background in [false, true]) {
        final core = OverviewFixture();
        final c = HomeController(
          core,
          LocalPreferences(File('/unused/overview.json')),
        );
        await c.initialize();
        await c.selectDevice(core.device['id']);
        final before = core.calls
            .where((call) => call['method'] == 'read_state')
            .length;
        core.targets['fridge'] = 7;
        core.readGate = Completer<void>();
        final refresh = c.refreshComponents();
        if (background) {
          c.setForeground(false);
          c.setForeground(true);
        } else {
          c.cancelPending();
        }
        core.readGate!.complete();
        expect(await refresh, false);
        expect((c.error as NativeFailure).code, 'CANCELLED');
        expect(
          core.calls.where((call) => call['method'] == 'read_state').length,
          before + 1,
        );
        expect(c.observed('climate.target_temperature'), 4);
        expect(c.componentReadError('fridge'), isNull);
        core.readGate = null;
        expect(await c.refreshComponents(), true);
        expect(c.observed('climate.target_temperature'), 7);
      }
    },
  );

  for (final missing in [false, true]) {
    testWidgets(
      missing
          ? 'overview displays missing and failed reads without invented values'
          : 'overview clicks preserve exact targets, observed state and caller navigation',
      (tester) async {
        tester.view.physicalSize = const Size(780, 1688);
        tester.view.devicePixelRatio = 2;
        tester.view.padding = const FakeViewPadding(top: 88, bottom: 28);
        debugDisableShadows = false;
        WidgetController.hitTestWarningShouldBeFatal = true;
        final core = OverviewFixture();
        if (missing) core.missingCaches.add('freezer');
        final c = HomeController(
          core,
          LocalPreferences(File('/unused/overview.json')),
        );
        final app = GlobalKey<NativeAppState>(), boundary = GlobalKey();
        final catalog = (await tester.runAsync(ScreenCatalog.load))!;
        Future<void> tap(Finder finder) async {
          await tester.ensureVisible(finder);
          await tester.tap(finder);
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull);
        }

        Future<void> back() => tap(find.byTooltip('Back'));
        InlineStepper stepper(String component) => tester.widget<InlineStepper>(
          find.byKey(ValueKey('component-stepper:$component')),
        );
        Future<void> capture(String name) async {
          await tester.runAsync(() async {
            final image =
                await (boundary.currentContext!.findRenderObject()
                        as RenderRepaintBoundary)
                    .toImage(pixelRatio: 2);
            final bytes = await image.toByteData(
              format: ui.ImageByteFormat.png,
            );
            final directory = Directory('build/verification/compartments')
              ..createSync(recursive: true);
            File(
              '${directory.path}/$name.png',
            ).writeAsBytesSync(bytes!.buffer.asUint8List());
            image.dispose();
          });
        }

        try {
          await tester.pumpWidget(
            RepaintBoundary(
              key: boundary,
              child: NativeApp(key: app, catalog: catalog, controller: c),
            ),
          );
          await tester.runAsync(
            () => precacheImage(
              const AssetImage('assets/window-light-background.png'),
              tester.element(find.byType(NativeApp)),
            ),
          );
          await tester.pumpAndSettle();
          await tap(find.text('Example refrigerator').first);
          expect(app.currentState!.id, 'C08');
          expect(find.text('Fridge'), findsOneWidget);
          expect(find.text('Freezer'), findsOneWidget);
          await capture(missing ? 'missing' : 'initial');
          if (missing) {
            expect(stepper('freezer').value, 'Unknown');
            expect(stepper('freezer').onIncrease, isNull);
            expect(stepper('freezer').onDecrease, isNull);
            core.targets['fridge'] = 5;
            core.failedReads.add('freezer');
            await tap(find.text('Refresh State'));
            expect(stepper('fridge').value, '5°C');
            expect(stepper('freezer').value, 'Unknown');
            expect(
              find.textContaining('Freezer could not be read:'),
              findsOneWidget,
            );
            expect(core.receipts, isEmpty);
            await capture('partial-read');
            core.failedReads.clear();
            await tap(find.text('Refresh State'));
            expect(c.error, isNull);
            expect(stepper('freezer').value, '-18°C');
            expect(
              find.textContaining('Freezer could not be read:'),
              findsNothing,
            );
            core.capabilities.last['actions'] = <String, dynamic>{};
            core
                    .capabilities
                    .last['properties']['target_temperature']['writable'] =
                false;
            await app.currentState!.go('C08');
            await tester.pumpAndSettle();
            expect(
              find.byKey(const ValueKey('component-stepper:freezer')),
              findsNothing,
            );
            expect(
              find.byKey(const ValueKey('edit-component:freezer')),
              findsNothing,
            );
            expect(find.text('Freezer target is read-only.'), findsOneWidget);
            expect(core.receipts, isEmpty);
            await capture('read-only');
          } else {
            expect(stepper('fridge').value, '4°C');
            expect(stepper('freezer').value, '-18°C');
            await tap(find.bySemanticsLabel('Increase Freezer target'));
            expect(app.currentState!.id, 'S06');
            expect(core.calls.last['command']['component_id'], 'freezer');
            expect(core.calls.last['command']['arguments'], {
              'target_temperature': -17,
            });
            expect(c.component, 'fridge');
            await back();
            expect(app.currentState!.id, 'C08');
            expect(stepper('freezer').value, '-18°C');
            await tap(find.bySemanticsLabel('Increase Fridge target'));
            expect(core.calls.last['command']['component_id'], 'fridge');
            expect(core.calls.last['command']['arguments'], {
              'target_temperature': 4.5,
            });
            await back();
            await tap(find.text('Refresh State'));
            expect(stepper('fridge').value, '4.5°C');
            expect(stepper('freezer').value, '-17°C');
            await capture('refreshed');
            await tap(find.byKey(const ValueKey('edit-component:freezer')));
            expect(app.currentState!.id, 'C14');
            expect(app.currentState!.screen['base'], 'C08');
            final field = find.byKey(
              const ValueKey('field:Target temperature'),
            );
            await tester.enterText(field, '-18.5');
            await tap(find.text('Save target'));
            expect(find.text('Supported step: 1'), findsOneWidget);
            expect(core.receipts, hasLength(2));
            await tester.enterText(field, '-20');
            await tap(find.text('Save target'));
            expect(core.calls.last['command']['component_id'], 'freezer');
            await back();
            await tap(find.text('Cancel'));
            expect(app.currentState!.id, 'C08');
            expect(c.component, 'freezer');
            await tap(find.bySemanticsLabel('Decrease Fridge target'));
            expect(core.calls.last['command']['component_id'], 'fridge');
            expect(core.calls.last['command']['arguments'], {
              'target_temperature': 4.0,
            });
            expect(c.component, 'freezer');
            await back();
            core
                    .capabilities
                    .first['actions']['set']['parameters']['properties']['mode'] =
                <String, dynamic>{
                  'type': 'string',
                  'enum': ['eco', 'normal'],
                };
            await app.currentState!.go('C08');
            await tester.pumpAndSettle();
            core
                .capabilities
                .first['actions']['set']['parameters']['required'] = [
              'target_temperature',
              'mode',
            ];
            await app.currentState!.go('C08');
            await tester.pumpAndSettle();
            expect(
              find.byKey(const ValueKey('component-stepper:fridge')),
              findsNothing,
            );
            await tap(find.byKey(const ValueKey('edit-component:fridge')));
            expect(app.currentState!.id, 'C10');
            expect(app.currentState!.screen['base'], 'C08');
            await tester.enterText(
              find.byKey(const ValueKey('parameter:target_temperature')),
              '6',
            );
            await tap(find.byKey(const ValueKey('parameter:mode:eco')));
            await tap(find.text('Apply'));
            expect(core.calls.last['command']['component_id'], 'fridge');
            expect(core.calls.last['command']['arguments'], {
              'target_temperature': 6,
              'mode': 'eco',
            });
            await back();
            await tap(find.text('Cancel'));
            expect(app.currentState!.id, 'C08');
            await tap(find.byKey(const ValueKey('open-component:fridge')));
            expect(app.currentState!.id, 'C01');
            expect(c.component, 'fridge');
            await back();
            expect(app.currentState!.id, 'C08');
          }
          tester.view.physicalSize = const Size(640, 1688);
          tester.platformDispatcher.textScaleFactorTestValue = 2;
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull);
          await capture(missing ? 'read-only-large-text' : 'large-text');
        } finally {
          await tester.pumpWidget(const SizedBox());
          tester.view.resetPhysicalSize();
          tester.view.resetDevicePixelRatio();
          tester.view.resetPadding();
          tester.platformDispatcher.clearTextScaleFactorTestValue();
          WidgetController.hitTestWarningShouldBeFatal = false;
          debugDisableShadows = true;
        }
      },
    );
  }
}
