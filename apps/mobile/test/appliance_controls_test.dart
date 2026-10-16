import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/appliance_controls.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart';
import 'package:homeverb_mobile/skin.dart';
import 'controls_test.dart' show ControlsFixture, capability, setter;
import 'package:homeverb_mobile/native_client.dart' as native;

class ApplianceFixture extends ControlsFixture {
  @override
  Future<dynamic> call(String method, [native.Json fields = const {}]) async {
    if (method == 'device_executions') {
      calls.add({'method': method, ...fields});
      return <native.Json>[];
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
  for (final family in [
    'light',
    'fan',
    'media',
    'sensor',
    'appliance_cycle',
    'numeric_fan',
  ]) {
    testWidgets(
      '$family dedicated layout uses declared controls and records outcomes',
      (tester) async {
        tester.view.physicalSize = const Size(780, 1688);
        tester.view.devicePixelRatio = 2;
        tester.view.padding = const FakeViewPadding(top: 88, bottom: 28);
        debugDisableShadows = false;
        WidgetController.hitTestWarningShouldBeFatal = true;
        final core = ApplianceFixture();
        final numeric = family == 'numeric_fan';
        final semanticFamily = numeric ? 'fan' : family;
        core.capabilities.removeWhere(
          (capability) => capability['id'] != semanticFamily,
        );
        final descriptor = core.capabilities.single;
        if (family == 'light') {
          core.capabilities.add(
            capability(
              'power',
              {
                'state': {
                  'type': 'boolean',
                  'readable': true,
                  'writable': true,
                },
              },
              {
                'set': setter(
                  {
                    'state': {'type': 'boolean'},
                  },
                  required: ['state'],
                ),
              },
            ),
          );
          core.observations['power.state'] = true;
        }

        if (numeric) {
          descriptor['properties']['speed']['type'] = 'integer';
          descriptor['actions']['set']['parameters']['properties']['speed'] = {
            'type': 'integer',
            'minimum': 2,
            'maximum': 8,
            'multipleOf': 2,
          };
          core.observations['fan.speed'] = 4;
        }
        final c = HomeController(
          core,
          LocalPreferences(File('/unused/appliance-ui.json')),
        );
        final catalog = (await tester.runAsync(ScreenCatalog.load))!;
        final app = GlobalKey<NativeAppState>(), boundary = GlobalKey();
        Future<void> tap(Finder f) async {
          await tester.ensureVisible(f);
          await tester.tap(f);
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull);
        }

        Future<void> returnToDevice() async {
          await app.currentState!.go('C01');
          await tester.pumpAndSettle();
        }

        Future<void> capture(String suffix) async {
          await tester.runAsync(() async {
            final rendered =
                await (boundary.currentContext!.findRenderObject()
                        as RenderRepaintBoundary)
                    .toImage(pixelRatio: 2);
            final bytes = await rendered.toByteData(
              format: ui.ImageByteFormat.png,
            );
            final output = Directory('build/verification/appliances')
              ..createSync(recursive: true);
            File(
              '${output.path}/$family-$suffix.png',
            ).writeAsBytesSync(bytes!.buffer.asUint8List());
            rendered.dispose();
          });
        }

        try {
          await tester.pumpWidget(
            RepaintBoundary(
              key: boundary,
              child: NativeApp(key: app, catalog: catalog, controller: c),
            ),
          );
          await tester.runAsync(() async {
            for (final name in [
              'window-light-background',
              'lamp',
              'tv',
              'sensor',
            ]) {
              await precacheImage(
                AssetImage('assets/$name.png'),
                tester.element(find.byType(NativeApp)),
              );
            }
          });
          await tester.pumpAndSettle();
          await c.selectDevice(core.device['id']);
          await returnToDevice();
          expect(find.byType(ApplianceControls), findsOneWidget);
          expect(applianceFamily(c), semanticFamily);
          await capture('initial');
          if (family == 'light') {
            final slider = find.descendant(
              of: find.byKey(
                const ValueKey('appliance-slider:light:brightness'),
              ),
              matching: find.byType(Slider),
            );
            expect(tester.widget<Slider>(slider).min, 10);
            expect(tester.widget<Slider>(slider).max, 80);
            core.denyPlan = true;
            await tester.ensureVisible(slider);
            await tester.drag(slider, const Offset(45, 0));
            await tester.pumpAndSettle();
            expect(app.currentState!.id, 'C01');
            expect(
              core.calls.where((call) => call['method'] == 'execute'),
              isEmpty,
            );
            expect(c.observed('light.brightness'), 35);
            final retained = tester.widget<Slider>(slider).value;
            expect(retained % 5, 0);
            expect(retained, isNot(35));
            expect(find.textContaining('Observed: 35%'), findsOneWidget);
            await capture('denied');
            core.denyPlan = false;
            await tester.ensureVisible(slider);
            await tester.drag(slider, const Offset(-15, 0));
            await tester.pumpAndSettle();
            expect(app.currentState!.id, 'S06');
            final command = core.calls.lastWhere(
              (call) => call['method'] == 'execute',
            )['command'];
            expect(command['capability'], 'light');
            expect(command['arguments'].keys, ['brightness']);
            expect(command['arguments']['brightness'] % 5, 0);
            expect(c.observed('light.brightness'), 35);
            await returnToDevice();
            final temperature = find.descendant(
              of: find.byKey(
                const ValueKey('appliance-slider:light:color_temperature'),
              ),
              matching: find.byType(Slider),
            );
            expect(tester.widget<Slider>(temperature).onChanged, isNull);
            final semantics = tester.ensureSemantics();
            await tester.ensureVisible(temperature);
            await tester.pumpAndSettle();
            final values = <String>[];
            void collectValues(SemanticsNode node) {
              values.add(node.value);
              node.visitChildren((child) {
                collectValues(child);
                return true;
              });
            }

            collectValues(tester.getSemantics(temperature));
            expect(values, contains('Color temperature Unknown'));
            expect(values.any((value) => value.contains('2200')), false);
            semantics.dispose();

            await tap(find.text('Set color temperature'));
            expect(app.currentState!.id, 'C10');
            expect(
              find.byKey(const ValueKey('parameter:color_temperature')),
              findsOneWidget,
            );
            await tester.enterText(
              find.byKey(const ValueKey('parameter:color_temperature')),
              '3000',
            );
            await tap(find.text('Apply'));
            expect(
              core.calls.lastWhere(
                (call) => call['method'] == 'execute',
              )['command']['arguments'],
              {'color_temperature': 3000},
            );
            await returnToDevice();
            await tap(find.byKey(const ValueKey('Power')));
            expect(
              core.calls.lastWhere(
                (call) => call['method'] == 'execute',
              )['command']['arguments'],
              {'state': false},
            );
            expect(c.observed('power.state'), true);
            final originalSet = descriptor['actions'].remove('set');
            descriptor['actions']['flash'] = <String, dynamic>{
              ...originalSet,
              'idempotency': 'non_idempotent',
            };
            await returnToDevice();
            expect(find.byType(ValueSlider), findsNothing);
            await tap(find.text('Flash'));
            expect(app.currentState!.id, 'C10');
            expect(
              find.textContaining(
                'An uncertain result must not be blindly repeated',
              ),
              findsOneWidget,
            );
            descriptor['actions'].remove('flash');
            descriptor['actions']['set'] = originalSet;
            descriptor['actions']['set']['parameters']['required'] = [
              'brightness',
              'color_temperature',
            ];
            await returnToDevice();
            expect(find.byType(ValueSlider), findsNothing);
            await tap(find.text('Set'));
            expect(app.currentState!.id, 'C10');
            expect(
              find.byKey(const ValueKey('parameter:brightness')),
              findsOneWidget,
            );
            expect(
              find.byKey(const ValueKey('parameter:color_temperature')),
              findsOneWidget,
            );
            descriptor['properties']['brightness']['writable'] = false;
            await returnToDevice();
            expect(
              tester
                  .widget<ActionButton>(
                    find.widgetWithText(ActionButton, 'Set'),
                  )
                  .onPressed,
              isNull,
            );
          } else if (family == 'fan') {
            expect(find.text('quiet'), findsOneWidget);
            expect(find.text('boost'), findsOneWidget);
            await tap(
              find.byKey(const ValueKey('appliance-option:speed:boost')),
            );
            expect(c.receipt!['status'], 'accepted');
            expect(c.observed('fan.speed'), 'quiet');
            expect(
              core.calls.lastWhere(
                (call) => call['method'] == 'execute',
              )['command']['arguments'],
              {'speed': 'boost'},
            );
            await returnToDevice();
            await tap(find.byKey(const ValueKey('property:oscillation')));
            expect(app.currentState!.id, 'C10');
            await tap(find.byKey(const ValueKey('parameter:oscillation:true')));
            await tap(find.text('Apply'));
            expect(
              core.calls.lastWhere(
                (call) => call['method'] == 'execute',
              )['command']['arguments'],
              {'oscillation': true},
            );
          } else if (numeric) {
            await tap(find.bySemanticsLabel('Increase Speed'));
            expect(
              core.calls.lastWhere(
                (call) => call['method'] == 'execute',
              )['command']['arguments'],
              {'speed': 6},
            );
            expect(c.observed('fan.speed'), 4);
            await returnToDevice();
            await tap(find.bySemanticsLabel('Decrease Speed'));
            expect(
              core.calls.lastWhere(
                (call) => call['method'] == 'execute',
              )['command']['arguments'],
              {'speed': 2},
            );
            core.observations['fan.speed'] = 8;
            await c.refresh();
            await returnToDevice();
            final stepper = tester.widget<InlineStepper>(
              find.byType(InlineStepper),
            );
            expect(stepper.onIncrease, isNull);
            expect(stepper.onDecrease, isNotNull);
          } else if (family == 'media') {
            await tap(find.byKey(const ValueKey('property:source')));
            await tap(find.byKey(const ValueKey('parameter:source:USB')));
            await tap(find.text('Apply'));
            expect(
              core.calls.lastWhere(
                (call) => call['method'] == 'execute',
              )['command']['arguments'],
              {'source': 'USB'},
            );
            await returnToDevice();
            final volume = find.descendant(
              of: find.byKey(const ValueKey('appliance-slider:media:volume')),
              matching: find.byType(Slider),
            );
            expect(tester.widget<Slider>(volume).min, -60);
            expect(tester.widget<Slider>(volume).max, 0);
            await tester.ensureVisible(volume);
            await tester.drag(volume, const Offset(25, 0));
            await tester.pumpAndSettle();
            expect(
              core.calls.lastWhere(
                (call) => call['method'] == 'execute',
              )['command']['action'],
              'set_volume',
            );
            await returnToDevice();
            await tap(find.byKey(const ValueKey('property:muted')));
            expect(
              core.calls.lastWhere(
                (call) => call['method'] == 'execute',
              )['command']['arguments'],
              {'muted': true},
            );
            await returnToDevice();
            await tap(find.text('Play'));
            expect(
              core.calls.lastWhere(
                (call) => call['method'] == 'execute',
              )['command']['action'],
              'play',
            );
            await returnToDevice();
            await tap(find.text('Pause'));
            expect(
              core.calls.lastWhere(
                (call) => call['method'] == 'execute',
              )['command']['action'],
              'pause',
            );
          } else if (family == 'sensor') {
            expect(find.text('No current reading'), findsOneWidget);
            expect(find.text('Unknown'), findsOneWidget);
            expect(find.byType(Slider), findsNothing);
            expect(
              core.calls.where((call) => call['method'] == 'execute'),
              isEmpty,
            );
            core.observations['sensor.humidity'] = 0;
            await tap(find.text('Refresh State'));
            expect(find.text('No current reading'), findsNothing);
            expect(find.text('0% · Stale'), findsOneWidget);
            expect(
              core.calls.where((call) => call['method'] == 'execute'),
              isEmpty,
            );
          } else {
            final start = tester.widget<ActionButton>(
              find.widgetWithText(ActionButton, 'Start Cycle'),
            );
            expect(start.onPressed, isNull);
            expect(find.textContaining('Trusted approval'), findsOneWidget);
            expect(
              core.calls.where((call) => call['method'] == 'execute'),
              isEmpty,
            );
          }
          await returnToDevice();
          await tap(find.text('Device information'));
          expect(app.currentState!.id, 'D06');
          await returnToDevice();
          await tap(find.text('Recent Operations'));
          expect(app.currentState!.id, 'S05');
          expect(
            core.calls.lastWhere(
              (call) => call['method'] == 'device_executions',
            )['device_id'],
            core.device['id'],
          );
          await returnToDevice();
          tester.view.physicalSize = const Size(640, 1688);
          tester.platformDispatcher.textScaleFactorTestValue = 2;
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull);
          await capture('large-text');
          for (final call in core.calls.where(
            (call) => call['method'] == 'execute',
          )) {
            final index = core.calls.indexOf(call);
            expect(core.calls[index - 1]['method'], 'plan');
            expect(core.calls[index - 1]['command'], call['command']);
          }
        } finally {
          await tester.pumpWidget(const SizedBox());
          tester.view.resetPhysicalSize();
          tester.view.resetDevicePixelRatio();
          tester.view.resetPadding();
          tester.platformDispatcher.clearTextScaleFactorTestValue();
          debugDisableShadows = true;
          WidgetController.hitTestWarningShouldBeFatal = false;
        }
      },
    );
  }
}
