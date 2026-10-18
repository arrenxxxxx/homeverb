import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/controls.dart';
import 'package:homeverb_mobile/device_detail_controls.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/skin.dart';
import 'controls_test.dart' show ControlsFixture, example;
import 'component_test.dart' show CompartmentFixture;

const dishwasher = 'vendor.midea.dishwasher';

// Fictional native policy and approval boundary; never opens a provider session.
class DishwasherFixture extends ControlsFixture
    implements NativeSessionClient, ApprovalClient {
  DishwasherFixture() {
    final descriptor = example('capability.meiju-dishwasher');
    descriptor['actions'].addAll(
      example('capability.meiju-dishwasher-cycle-additional')['actions'],
    );
    capabilities
      ..clear()
      ..add(descriptor);
    device['alias'] = 'Fictional dishwasher';
    device['connections'] = [
      {'id': 'fixture-cloud', 'transport': 'cloud'},
    ];
    observations
      ..clear()
      ..addAll({
        '$dishwasher.temperature': 42,
        '$dishwasher.remaining_minutes': 35,
        '$dishwasher.additional': 9,
      });
  }
  bool allowed = true;
  @override
  bool supportsTrustedApproval = true;
  Json? prepared;
  @override
  Set<String> get methods => {'inspect_session_grants'};
  @override
  bool apiAtLeast(int minor) => minor <= 23;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'inspect_session_grants') {
      calls.add({'method': method, ...fields});
      return {
        'device_id': device['id'],
        'connection_id': 'fixture-cloud',
        'transport': 'cloud',
        'source': 'session_policy',
        'provider_authorization': 'not_checked',
        'grants': [
          if (allowed)
            for (final action in (capabilities.single['actions'] as Json).keys)
              {
                'component_id': 'main',
                'capability': dishwasher,
                'action': action,
                'argument_schema': {
                  'const': {
                    if (action == 'start_cycle') 'program': 'eco_wash',
                    'additional': 9,
                  },
                },
              },
        ],
      };
    }
    return super.call(method, fields);
  }

  @override
  Future<ApprovalPreview> prepareApproval(Json command) async {
    prepared = command;
    throw NativeFailure(
      'POLICY_DENIED',
      'Fictional approval stop before transmission.',
    );
  }

  @override
  Future<Json> executeApproved(ApprovalPreview preview) async =>
      throw StateError('Unexpected send');
  @override
  Future<void> discardApproval(ApprovalPreview preview) async {}
}

class ScopedFridge extends CompartmentFixture implements NativeSessionClient {
  @override
  Set<String> get methods => {'inspect_session_grants'};
  @override
  bool apiAtLeast(int minor) => minor <= 23;
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method != 'inspect_session_grants') return super.call(method, fields);
    final connection = (device['connections'] as List).firstWhere(
      (v) => v['id'] == fields['connection_id'],
    );
    return {
      'device_id': device['id'],
      'connection_id': connection['id'],
      'transport': connection['transport'],
      'source': 'session_policy',
      'provider_authorization': 'not_checked',
      'grants': [
        {
          'component_id': 'freezer',
          'capability': 'climate',
          'action': 'set',
          'argument_schema': {
            'const': {'target_temperature': -17},
          },
        },
      ],
    };
  }
}

void main() {
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
  });
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));

  test('cycle arguments do not turn read-only observations into setters', () {
    final descriptor = DishwasherFixture().capabilities.single;
    for (final action in (descriptor['actions'] as Json).values) {
      expect(
        actionUnavailable(descriptor, action, trustedApproval: true),
        isNull,
      );
      expect(actionUnavailable(descriptor, action), contains('approval'));
      expect(
        actionUnavailable(descriptor, {
          ...action,
          'risk': 'high',
        }, trustedApproval: true),
        contains('approval'),
      );
      expect(
        actionUnavailable(descriptor, {
          ...action,
          'idempotency': 'set_value',
        }, trustedApproval: true),
        contains('read-only'),
      );
    }
  });

  test(
    'compartment permissions never borrow another component grant',
    () async {
      final core = ScopedFridge();
      final c = HomeController(
        core,
        LocalPreferences(File('/unused/detail.json')),
      );
      await c.initialize();
      await c.selectDevice(core.device['id']);
      expect(
        c.hasControlAction('climate', 'set', componentId: 'fridge'),
        false,
      );
      expect(
        c.hasControlAction('climate', 'set', componentId: 'freezer'),
        true,
      );
      expect(
        c.canControlValue(
          'climate',
          'set',
          'target_temperature',
          -17,
          componentId: 'freezer',
        ),
        true,
      );
      expect(
        c.canControlValue(
          'climate',
          'set',
          'target_temperature',
          -19,
          componentId: 'freezer',
        ),
        false,
      );
      expect(c.component, 'fridge');
      c.dispose();
    },
  );

  for (final allowed in [false, true]) {
    testWidgets(
      'dishwasher detail buttons respect current permission: $allowed',
      (tester) async {
        tester.view.physicalSize = const Size(780, 1688);
        tester.view.devicePixelRatio = 2;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
        final core = DishwasherFixture()..allowed = allowed;
        final c = HomeController(
          core,
          LocalPreferences(File('/unused/detail.json')),
        );
        final catalog = (await tester.runAsync(ScreenCatalog.load))!;
        final app = GlobalKey<NativeAppState>(), boundary = GlobalKey();
        Future<void> tap(Finder finder) async {
          await tester.ensureVisible(finder);
          await tester.tap(finder);
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull);
        }

        Future<void> capture(String name) async {
          await tester.runAsync(() async {
            final image =
                await (boundary.currentContext!.findRenderObject()
                        as RenderRepaintBoundary)
                    .toImage(pixelRatio: 2);
            final bytes = await image.toByteData(
              format: ui.ImageByteFormat.png,
            );
            final dir = Directory('build/verification/appliance-details')
              ..createSync(recursive: true);
            File(
              '${dir.path}/$name.png',
            ).writeAsBytesSync(bytes!.buffer.asUint8List());
            image.dispose();
          });
        }

        await tester.pumpWidget(
          RepaintBoundary(
            key: boundary,
            child: NativeApp(key: app, catalog: catalog, controller: c),
          ),
        );
        await tester.pumpAndSettle();
        await tap(find.text('Fictional dishwasher').first);
        await tester.runAsync(() async {
          for (final image in tester.widgetList<Image>(find.byType(Image))) {
            await precacheImage(
              image.image,
              tester.element(find.byType(DeviceDetailControls)),
            );
          }
        });
        await tester.pumpAndSettle();
        expect(find.byType(DeviceDetailControls), findsOneWidget);
        expect(find.text('42°C · Stale'), findsOneWidget);
        expect(find.text('One extra rinse · Stale'), findsOneWidget);
        expect(find.text('Unknown'), findsWidgets);
        await capture(allowed ? 'dishwasher-status' : 'dishwasher-read-only');
        final start = find.byKey(const ValueKey('control:0:start_cycle'));
        if (allowed) {
          expect(tester.widget(start), isA<ActionButton>());
          await tester.ensureVisible(start);
          await tester.pumpAndSettle();
          expect(
            tester.getTopLeft(start).dy,
            tester
                .getTopLeft(find.byKey(const ValueKey('control:0:pause_cycle')))
                .dy,
          );
          await capture('dishwasher-buttons');
          for (final action in [
            'pause_cycle',
            'resume_cycle',
            'cancel_cycle',
            'power_on',
            'power_off',
          ]) {
            await tap(find.byKey(ValueKey('control:0:$action')));
            expect(
              find.byKey(const ValueKey('parameter:additional:9')),
              findsOneWidget,
            );
            expect(core.prepared, isNull);
            await tap(find.text('Cancel'));
          }
          for (final action in ['power_on', 'power_off']) {
            await tap(find.byKey(ValueKey('control:0:$action')));
            await tap(find.text('Cloud'));
            await tap(find.text('Review Command'));
            expect(core.prepared, isNull);
            await tap(find.byKey(const ValueKey('parameter:additional:9')));
            await tap(find.text('Review Command'));
            expect(core.prepared?['action'], action);
            expect(core.prepared?['arguments'], {'additional': 9});
            expect(core.prepared?['connection'], 'cloud');
            expect(core.calls.where((v) => v['method'] == 'execute'), isEmpty);
            await tap(find.text('Cancel'));
            core.prepared = null;
          }
          await tap(start);
          await tap(find.byKey(const ValueKey('parameter:program:eco_wash')));
          await tap(find.text('Cloud'));
          await tap(find.text('Review Command'));
          expect(
            core.prepared,
            isNull,
            reason: 'Additional option must be explicitly selected.',
          );
          await tap(find.byKey(const ValueKey('parameter:additional:9')));
          await tap(find.text('Review Command'));
          expect(core.prepared?['arguments'], {
            'program': 'eco_wash',
            'additional': 9,
          });
          expect(core.prepared?['connection'], 'cloud');
          await tap(find.text('Cancel'));
          core.allowed = false;
          await c.selectDevice(core.device['id']);
          await tester.pumpAndSettle();
          expect(
            start,
            findsNothing,
            reason: 'Saved actions must not survive permission revocation.',
          );
        } else {
          expect(start, findsNothing);
          expect(
            find.text(
              'No controls are allowed for this device on this connection.',
            ),
            findsOneWidget,
          );
        }
        expect(core.calls.where((v) => v['method'] == 'execute'), isEmpty);
        core.allowed = allowed;
        await c.selectDevice(core.device['id']);
        await LocaleSettings.setLocale(AppLocale.zhCn);
        tester.view.physicalSize = const Size(640, 1688);
        tester.platformDispatcher.textScaleFactorTestValue = 2;
        await tester.pumpAndSettle();
        if (allowed) {
          await tester.ensureVisible(start);
          await tester.pumpAndSettle();
          expect(
            tester
                .getTopLeft(find.byKey(const ValueKey('control:0:pause_cycle')))
                .dy,
            greaterThan(tester.getTopLeft(start).dy),
          );
        }
        await capture('dishwasher-large-text-$allowed');
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
      },
    );
  }
}
