import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/rendering.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/controls.dart';
import 'package:homeverb_mobile/device_detail_controls.dart';
import 'package:homeverb_mobile/device_presentation.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:homeverb_mobile/skin.dart';
import 'accounts_test.dart'
    show AccountFixture, AccountPreferences, firstAccount, secondAccount;
import 'controls_test.dart' show ControlsFixture, capability;

class MijiaAccountFixture extends AccountFixture {
  MijiaAccountFixture() {
    adapter = 'homeverb.platforms';
    accountProviders = {'midea.meiju', 'xiaomi.mijia'};
    record(firstAccount)['provider'] = 'xiaomi.mijia';
    record(firstAccount)['label'] = 'Mijia account';
    record(secondAccount)['label'] = 'Meiju Home';
    cards.first['device']['alias'] = 'Entry doorbell';
  }
  @override
  Json report(String id, String authentication) {
    final value = super.report(id, authentication);
    if (record(id)['provider'] == 'xiaomi.mijia') {
      value['retrieval'] = {
        'status': 'partial',
        'device_ids': [...links[id]!],
        'error': {
          'code': 'MIJIA_INVENTORY_PARTIAL',
          'message': 'Fictional partial coverage.',
          'retryable': false,
        },
      };
    }
    return value;
  }
}

class MijiaReadFixture extends ControlsFixture
    implements NativeSessionClient, AccountClient {
  MijiaReadFixture(String family) {
    final fields = switch (family) {
      'fan' => {
        'level': {'type': 'integer', 'readable': true, 'writable': false},
        'horizontal_swing': {
          'type': 'boolean',
          'readable': true,
          'writable': false,
        },
      },
      'lock' => {
        'status': {'type': 'integer', 'readable': true, 'writable': false},
      },
      _ => {
        'volume': {'type': 'integer', 'readable': true, 'writable': false},
      },
    };
    capabilities
      ..clear()
      ..add(capability('vendor.xiaomi.mijia.$family', fields, {}));
    if (family == 'fan')
      capabilities.add(
        capability('power', {
          'state': {'type': 'boolean', 'readable': true, 'writable': false},
        }, {}),
      );
    device['alias'] = 'Fictional $family';
    device['connections'] = [
      {
        'id': 'fictional-cloud',
        'adapter': 'xiaomi.mijia',
        'transport': 'cloud',
      },
    ];
    observations
      ..clear()
      ..addAll(switch (family) {
        'fan' => {
          'power.state': false,
          'vendor.xiaomi.mijia.fan.level': 3,
          'vendor.xiaomi.mijia.fan.horizontal_swing': null,
        },
        'lock' => {'vendor.xiaomi.mijia.lock.status': 255},
        _ => {'vendor.xiaomi.mijia.doorbell.volume': 42},
      });
  }
  Completer<void>? inspectionGate, inspectionStarted;
  @override
  String get adapter => 'homeverb.platforms';
  @override
  Set<String> get accountProviders => {'midea.meiju', 'xiaomi.mijia'};
  @override
  bool get supportsAccountConnection => true;
  @override
  bool get supportsPendingReauthorization => true;
  @override
  bool apiAtLeast(int minor) => minor <= 24;
  @override
  Set<String> get methods => {
    'inspect_control_options',
    'inspect_session_grants',
  };
  @override
  Future<Json> connectAccount({
    required Uint8List login,
    required Uint8List password,
    required String label,
    String? accountId,
    String? provider,
  }) async {
    login.fillRange(0, login.length, 0);
    password.fillRange(0, password.length, 0);
    throw StateError('The read fixture must never authenticate.');
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'inspect_control_options') {
      inspectionStarted?.complete();
      await inspectionGate?.future;
      calls.add({'method': method, ...fields});
      throw NativeFailure(
        'CAPABILITY_UNSUPPORTED',
        'This fixture has no control options.',
      );
    }
    if (method == 'inspect_session_grants') {
      calls.add({'method': method, ...fields});
      return {
        'device_id': device['id'],
        'connection_id': 'fictional-cloud',
        'transport': 'cloud',
        'source': 'session_policy',
        'provider_authorization': 'not_checked',
        'grants': <Json>[],
      };
    }
    return super.call(method, fields);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  late ScreenCatalog catalog;
  setUpAll(() async {
    await (FontLoader(
      'Inter',
    )..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))).load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
    catalog = await ScreenCatalog.load();
  });
  tearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
  testWidgets(
    'Mijia connects once, preserves partial cards and reauthorizes the same provider',
    (tester) async {
      tester.view.physicalSize = const Size(780, 1688);
      tester.view.devicePixelRatio = 2;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final core = MijiaAccountFixture();
      final c = HomeController(core, AccountPreferences());
      final key = GlobalKey<NativeAppState>();
      final boundary = GlobalKey();
      Future<void> capture(String name) => tester.runAsync(() async {
        final output = Directory('build/verification/mijia')
          ..createSync(recursive: true);
        final image =
            await (boundary.currentContext!.findRenderObject()
                    as RenderRepaintBoundary)
                .toImage(pixelRatio: 2);
        final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
        await File(
          '${output.path}/$name.png',
        ).writeAsBytes(bytes!.buffer.asUint8List());
        image.dispose();
      });
      await tester.pumpWidget(
        TranslationProvider(
          child: RepaintBoundary(
            key: boundary,
            child: NativeApp(key: key, catalog: catalog, controller: c),
          ),
        ),
      );
      await tester.pumpAndSettle();
      await key.currentState!.go('P02');
      await tester.pumpAndSettle();
      await capture('platforms');
      expect(find.text('Meiju'), findsOneWidget);
      expect(find.text('Mijia'), findsOneWidget);
      await tester.tap(find.text('Mijia'));
      await tester.pumpAndSettle();
      await capture('signin');
      expect(find.text('Mijia sign-in'), findsOneWidget);
      await tester.enterText(
        find.byKey(const ValueKey('credential:Mijia account')),
        'fiction@example.invalid',
      );
      await tester.enterText(
        find.byKey(const ValueKey('credential:Password')),
        'fiction-password',
      );
      await tester.ensureVisible(find.text('Sign In and Retrieve Devices'));
      await tester.tap(find.text('Sign In and Retrieve Devices'));
      await tester.pumpAndSettle();
      await capture('account');
      expect(key.currentState!.id, 'P09');
      expect(core.connectionProviders, ['xiaomi.mijia']);
      expect(core.connectionTargets, [null]);
      expect(core.ownedLogin!.every((byte) => byte == 0), true);
      expect(core.ownedPassword!.every((byte) => byte == 0), true);
      expect(find.text('Devices retrieved'), findsOneWidget);
      expect(
        find.text('Retrieved devices: 1. The list may be incomplete.'),
        findsOneWidget,
      );
      expect(find.text('Entry doorbell'), findsOneWidget);
      expect(c.cards, hasLength(2));
      expect(
        core.requests.where(
          (r) => [
            'discover',
            'refresh_account',
            'recover_account',
          ].contains(r['method']),
        ),
        isEmpty,
      );
      c.selectedAccountProvider = 'midea.meiju';
      expect(c.connectionProvider, 'xiaomi.mijia');
      expect(
        await c.connectAccount(
          Uint8List.fromList('fiction@example.invalid'.codeUnits),
          Uint8List.fromList('fiction-password'.codeUnits),
          '',
        ),
        true,
      );
      expect(core.connectionProviders.last, 'xiaomi.mijia');
      expect(core.connectionTargets.last, firstAccount);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );
  testWidgets('Chinese Mijia form and partial cards fit at large text', (
    tester,
  ) async {
    LocaleSettings.setLocale(AppLocale.zhCn);
    tester.view.physicalSize = const Size(780, 1688);
    tester.view.devicePixelRatio = 2;
    tester.platformDispatcher.textScaleFactorTestValue = 2;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
    final core = MijiaAccountFixture();
    final c = HomeController(core, AccountPreferences());
    final key = GlobalKey<NativeAppState>();
    await tester.pumpWidget(
      TranslationProvider(
        child: NativeApp(key: key, catalog: catalog, controller: c),
      ),
    );
    await tester.pumpAndSettle();
    c.selectedAccountProvider = 'xiaomi.mijia';
    await key.currentState!.go('P04');
    await tester.pumpAndSettle();
    expect(
      find.text(localize(key.currentContext!, 'Mijia sign-in')),
      findsOneWidget,
    );
    expect(
      find.byKey(const ValueKey('credential:Mijia account')),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
    await c.selectAccount(firstAccount);
    c.accountReport = core.report(firstAccount, 'not_attempted');
    await key.currentState!.go('P09');
    await tester.pumpAndSettle();
    expect(find.text('Entry doorbell'), findsOneWidget);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
  });
  for (final family in ['fan', 'lock', 'doorbell']) {
    testWidgets(
      '$family displays observed values without controls or fabricated power',
      (tester) async {
        final core = MijiaReadFixture(family);
        var reopened = false;
        final c = HomeController(
          core,
          AccountPreferences(),
          reopen: (_) async {
            reopened = true;
            return core;
          },
        );
        expect(await c.selectDevice(core.device['id'] as String), true);
        expect(reopened, false);
        await tester.pumpWidget(
          MaterialApp(
            theme: windowTheme(),
            home: Scaffold(
              body: SingleChildScrollView(
                child: DeviceDetailControls(
                  controller: c,
                  onAction: (_) {
                    throw StateError('Read-only fixture must not execute.');
                  },
                ),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        expect(
          find.text(
            'No controls are allowed for this device on this connection.',
          ),
          findsOneWidget,
        );
        if (family == 'lock') {
          expect(find.text('Lock status code'), findsOneWidget);
          expect(find.text('255 · Stale'), findsOneWidget);
        }
        if (family == 'doorbell') {
          expect(find.text('Doorbell volume'), findsOneWidget);
          expect(find.text('42 · Stale'), findsOneWidget);
        }
        if (family == 'fan') {
          expect(c.observed('power.state'), false);
          expect(find.text('Unknown · Stale'), findsOneWidget);
        } else {
          expect(c.observed('power.state'), isNull);
        }
        expect(
          core.calls.where(
            (v) => ['execute', 'plan', 'refresh_account'].contains(v['method']),
          ),
          isEmpty,
        );
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        c.dispose();
      },
    );
  }
  test(
    'late unsupported control inspection cannot resume a background device view',
    () async {
      final core = MijiaReadFixture('fan')
        ..inspectionGate = Completer<void>()
        ..inspectionStarted = Completer<void>();
      final c = HomeController(
        core,
        AccountPreferences(),
        reopen: (_) async => core,
      );
      final selecting = c.selectDevice(core.device['id'] as String);
      await core.inspectionStarted!.future;
      c.setForeground(false);
      core.inspectionGate!.complete();
      expect(await selecting, false);
      expect((c.error as NativeFailure).code, 'CANCELLED');
      expect(
        core.calls.where((call) => call['method'] == 'read_state'),
        isEmpty,
      );
      c.dispose();
    },
  );

  test('native units and decorative service families preserve semantics', () {
    expect(propertyValue(61, {'unit': 'percentage'}), '61%');
    expect(propertyValue(90, {'unit': 'arcdegrees'}), '90°');
    expect(propertyValue(null, {'unit': 'percentage'}), 'Unknown');
    expect(componentLabel({'id': 'battery-3'}), 'Battery');
    expect(
      deviceArtwork({
        'components': [
          {
            'id': 'fan-2',
            'capabilities': ['power'],
          },
        ],
      }),
      'fan',
    );
    expect(
      deviceArtwork({
        'components': [
          {
            'id': 'lock-2',
            'capabilities': ['vendor.xiaomi.mijia.lock'],
          },
        ],
      }),
      'lock',
    );
  });
}
