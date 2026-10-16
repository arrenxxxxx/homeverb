import 'dart:io';
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

const cycle = 'vendor.example.cycle';
Json command() => {
  'schema_version': '1.0',
  'request_id': uniqueId(),
  'device_id': 'dev-mock-ac',
  'component_id': 'main',
  'capability': cycle,
  'action': 'start',
  'arguments': {'program': 'normal'},
  'connection': 'cloud',
  'timeout_ms': 10000,
  'wait_for': 'verified',
};

class LostApprovalReply implements CoreClient, ApprovalClient {
  LostApprovalReply(this.native);
  final NativeClient native;
  int sends = 0;
  bool loseReply = true;
  @override
  bool get supportsTrustedApproval => true;
  @override
  Future<ApprovalPreview> prepareApproval(Json command) =>
      native.prepareApproval(command);
  @override
  Future<void> discardApproval(ApprovalPreview preview) =>
      native.discardApproval(preview);
  @override
  Future<Json> executeApproved(ApprovalPreview preview) async {
    sends++;
    final result = await native.executeApproved(preview);
    if (!loseReply) return result;
    throw NativeFailure('NATIVE_ERROR', 'Fictional lost reply.');
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) =>
      native.call(method, fields);
  @override
  void foreground(bool value) => native.foreground(value);
  @override
  void cancelAll() => native.cancelAll();
  @override
  Future<void> close() => native.close();
}

void main({bool deviceRun = false}) {
  setUpAll(() async {
    if (!deviceRun) {
      expect(
        await rootBundle.loadString('assets/en-US.json'),
        File(
          '../../packages/presentation/locales/en-US.json',
        ).readAsStringSync(),
        reason: 'Bundled labels must match the shared catalog.',
      );
    }
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'))).load();
  });
  final library = deviceRun
      ? NativeClient.defaultLibrary()
      : File(
          '../../target/debug/${NativeClient.defaultLibrary()}',
        ).absolute.path;
  Future<NativeClient> open(Directory dir, {bool writes = true}) =>
      NativeClient.open(
        dataDir: dir.path,
        libraryPath: library,
        allowMockWrites: writes,
        scenario: 'approval-required',
      );
  test(
    'trusted native scope is immutable, handle-owned and consumed once',
    () async {
      final dir = await Directory.systemTemp.createTemp('homeverb-approval-');
      final otherDir = await Directory.systemTemp.createTemp(
        'homeverb-approval-other-',
      );
      final core = await open(dir), other = await open(otherDir);
      try {
        await core.call('discover');
        final denied = await core.call('execute', {'command': command()});
        expect(denied['status'], 'failed');
        expect(denied['error']['code'], 'POLICY_DENIED');
        expect(denied['attempts'], isEmpty);
        final historyBefore = await core.call('recent_executions');
        final input = command();
        final preview = await core.prepareApproval(input);
        expect(await core.call('recent_executions'), historyBefore);
        input['arguments']['program'] = 'changed';
        expect(preview.command['arguments']['program'], 'normal');
        expect(
          () => preview.command['arguments']['program'] = 'changed',
          throwsUnsupportedError,
        );
        expect(
          () => (preview.action['transports'] as List).clear(),
          throwsUnsupportedError,
        );
        await expectLater(
          other.executeApproved(preview),
          throwsA(isA<NativeFailure>()),
        );
        final result = await core.executeApproved(preview);
        expect(result['status'], 'accepted');
        expect(result['error']['code'], 'VERIFICATION_PENDING');
        await expectLater(
          core.executeApproved(preview),
          throwsA(isA<NativeFailure>()),
        );
        final saved = await core.call('get_execution', {
          'request_id': preview.command['request_id'],
        });
        expect(saved['status'], 'accepted');
      } finally {
        await core.close();
        await other.close();
        await dir.delete(recursive: true);
        await otherDir.delete(recursive: true);
      }
    },
  );
  test(
    'dismiss, navigation, background, version and grants refuse stale approvals',
    () async {
      final dir = await Directory.systemTemp.createTemp(
        'homeverb-approval-cancel-',
      );
      final core = await open(dir);
      try {
        await core.call('discover');
        for (final cancel in <Future<void> Function(ApprovalPreview)>[
          (p) => core.discardApproval(p),
          (p) async => core.cancelAll(),
          (p) async {
            core.foreground(false);
            core.foreground(true);
          },
        ]) {
          final preview = await core.prepareApproval(command());
          await cancel(preview);
          await expectLater(
            core.executeApproved(preview),
            throwsA(isA<NativeFailure>()),
          );
        }
        final pending = core.prepareApproval(command());
        core.cancelAll();
        await expectLater(pending, throwsA(isA<NativeFailure>()));
        expect(await core.call('recent_executions'), isEmpty);
        core.apiVersion = '1.12.0';
        expect(core.supportsTrustedApproval, false);
        await expectLater(
          core.prepareApproval(command()),
          throwsA(
            isA<NativeFailure>().having(
              (e) => e.code,
              'code',
              'API_UPGRADE_REQUIRED',
            ),
          ),
        );
      } finally {
        await core.close();
        await dir.delete(recursive: true);
      }
      final readDir = await Directory.systemTemp.createTemp(
        'homeverb-approval-read-',
      );
      final read = await open(readDir, writes: false);
      try {
        await read.call('discover');
        await expectLater(
          read.prepareApproval(command()),
          throwsA(isA<NativeFailure>()),
        );
      } finally {
        await read.close();
        await readDir.delete(recursive: true);
      }
    },
  );
  test(
    'lost confirmation reply recovers the original receipt without replay',
    () async {
      final dir = await Directory.systemTemp.createTemp(
        'homeverb-approval-recovery-',
      );
      final native = await open(dir);
      final core = LostApprovalReply(native);
      final state = HomeController(
        core,
        LocalPreferences(File('${dir.path}/ui.json')),
      );
      try {
        await state.discover();
        await state.selectDevice('dev-mock-ac');
        expect(
          await state.prepareApproval(cycle, 'start', {
            'program': 'normal',
          }, 'cloud'),
          true,
        );
        final request = state.approvalRequestId!;
        expect(await state.confirmApproval(), false);
        expect(state.approvalNeedsReceipt, true);
        expect(state.approvalPreview, isNull);
        expect(await state.confirmApproval(), false);
        expect(core.sends, 1);
        expect(await state.loadReceipt(request), true);
        expect(state.receipt!['status'], 'accepted');
        expect(core.sends, 1);
      } finally {
        state.cancelPending();
        await core.close();
        await dir.delete(recursive: true);
      }
    },
  );
  testWidgets(
    'click review, cancel, expiry, background and explicit confirmation',
    (tester) async {
      if (!deviceRun) {
        tester.view.physicalSize = const Size(780, 1688);
        tester.view.devicePixelRatio = 2;
      }
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final dir = Directory.systemTemp.createTempSync('homeverb-approval-ui-');
      late NativeClient core;
      late HomeController state;
      late LostApprovalReply bridge;
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.runAsync(() async {
        core = await open(dir);
        bridge = LostApprovalReply(core)..loseReply = false;
        state = HomeController(
          bridge,
          LocalPreferences(File('${dir.path}/ui.json')),
        );
        await state.discover();
        await state.selectDevice('dev-mock-ac');
      });
      final app = GlobalKey<NativeAppState>(), boundary = GlobalKey();
      Future<void> settle() async {
        for (var i = 0; i < 100 && state.busy; i++) {
          await tester.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 20)),
          );
          await tester.pump();
        }
        await tester.pumpAndSettle();
        expect(state.busy, false);
        expect(tester.takeException(), isNull);
      }

      Future<void> tap(String label) async {
        final target = find.text(label).last;
        await tester.ensureVisible(target);
        await tester.tap(target);
        await settle();
      }

      Future<void> form() async {
        await tester.runAsync(() => app.currentState!.go('C01'));
        await settle();
        await tap('Start');
        expect(app.currentState!.id, 'C10');
        await tap('normal');
        await tap('Cloud');
      }

      Future<void> capture(String name) async {
        await tester.runAsync(() async {
          final image =
              await (boundary.currentContext!.findRenderObject()
                      as RenderRepaintBoundary)
                  .toImage(pixelRatio: 2);
          final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
          final file = File(
            '${deviceRun ? Directory.systemTemp.path : 'build/verification'}/approval/$name.png',
          );
          await file.parent.create(recursive: true);
          await file.writeAsBytes(bytes!.buffer.asUint8List());
          image.dispose();
        });
      }

      try {
        await tester.pumpWidget(
          RepaintBoundary(
            key: boundary,
            child: NativeApp(key: app, catalog: catalog, controller: state),
          ),
        );
        await settle();
        await form();
        await tap('Review Command');
        expect(state.approvalPreview, isNotNull);
        expect(find.text('Arguments: {"program":"normal"}'), findsOneWidget);
        expect(
          find.textContaining('Connection: Provider cloud'),
          findsOneWidget,
        );
        expect(state.receipt, isNull);
        await capture('review');
        if (!deviceRun) {
          tester.view.physicalSize = const Size(640, 1688);
          tester.platformDispatcher.textScaleFactorTestValue = 2;
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull);
          await tester.ensureVisible(find.text('Confirm Command'));
          await capture('review-large-text');
          tester.platformDispatcher.clearTextScaleFactorTestValue();
          tester.view.physicalSize = const Size(780, 1688);
          await tester.pumpAndSettle();
        }
        await tap('Cancel');
        expect(state.approvalPreview, isNull);
        await tester.runAsync(
          () async => expect(await core.call('recent_executions'), isEmpty),
        );
        await form();
        await tap('Review Command');
        if (deviceRun) {
          await tester.runAsync(
            () => Future<void>.delayed(const Duration(seconds: 61)),
          );
          await tester.pump();
        } else {
          await tester.pump(const Duration(seconds: 61));
        }
        await settle();
        expect(state.approvalPreview, isNull);
        expect(find.text('Confirm Command'), findsNothing);
        expect(state.error.toString(), contains('APPROVAL_EXPIRED'));
        await form();
        await tap('Review Command');
        state.setForeground(false);
        state.setForeground(true);
        await settle();
        expect(state.approvalPreview, isNull);
        expect(find.text('Confirm Command'), findsNothing);
        await form();
        await tap('Review Command');
        final request = state.approvalPreview!.command['request_id'];
        await tap('Confirm Command');
        expect(app.currentState!.id, 'S06');
        expect(state.receipt!['status'], 'accepted');
        expect(state.receipt!['request_id'], request);
        expect(state.approvalPreview, isNull);
        await capture('accepted');
        bridge.loseReply = true;
        await form();
        await tap('Review Command');
        final lostRequest = state.approvalRequestId;
        await tap('Confirm Command');
        expect(find.text('Look Up Receipt'), findsOneWidget);
        expect(find.text('Confirm Command'), findsNothing);
        expect(bridge.sends, 2);
        await capture('lost-reply');
        await tap('Look Up Receipt');
        expect(app.currentState!.id, 'S06');
        expect(state.receipt!['request_id'], lostRequest);
        expect(state.receipt!['status'], 'accepted');
        expect(bridge.sends, 2);
        await tester.runAsync(
          () async => expect(
            (await core.call('recent_executions') as List),
            hasLength(2),
          ),
        );
      } finally {
        await tester.pumpWidget(const SizedBox());
        await tester.runAsync(() => core.close());
        dir.deleteSync(recursive: true);
      }
    },
  );
}
