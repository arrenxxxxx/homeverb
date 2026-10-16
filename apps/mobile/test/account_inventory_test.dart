import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/account_views.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'accounts_test.dart'
    show AccountFixture, AccountPreferences, firstAccount;

class InventoryFixture extends AccountFixture {
  late final Json inventory;
  Completer<void>? gate;
  String? failure;

  void load(Json example) {
    inventory = example;
    cards[0]['device']['id'] = inventory['devices'][0]['device_id'];
    cards[0]['device']['alias'] = 'Exact saved card';
    cards[0]['device']['connections'] = [
      {'id': inventory['devices'][0]['connection_id']},
    ];
    cards[1]['device']['id'] = inventory['devices'][1]['device_id'];
    cards[1]['device']['alias'] = 'Mismatched connection card';
    cards[1]['device']['connections'] = [
      {'id': 'different-connection'},
    ];
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method != 'inspect_account_inventory')
      return super.call(method, fields);
    requests.add({'method': method, ...fields});
    if (gate != null) await gate!.future;
    if (failure != null)
      throw NativeFailure(failure!, 'Fictional query failure.');
    return jsonDecode(jsonEncode(inventory));
  }
}

Future<InventoryFixture> fixture() async {
  final examples =
      jsonDecode(
            await File(
              '../../packages/native-contract/account-examples.json',
            ).readAsString(),
          )
          as Json;
  final example = examples['cases'] as List;
  return InventoryFixture()..load(
    example.singleWhere(
          (entry) =>
              entry['response_definition'] == 'inspect_account_inventory',
        )['response']['data']
        as Json,
  );
}

void main() {
  setUpAll(() async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
  });

  testWidgets(
    'explicit saved inventory clicks preserve exact bindings and query failures',
    (tester) async {
      tester.view.physicalSize = const Size(780, 1688);
      tester.view.devicePixelRatio = 2;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final shadows = debugDisableShadows;
      debugDisableShadows = false;
      addTearDown(() => debugDisableShadows = shadows);
      final core = (await tester.runAsync(fixture))!;
      final controller = HomeController(core, AccountPreferences());
      final key = GlobalKey<NativeAppState>(), boundary = GlobalKey();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        RepaintBoundary(
          key: boundary,
          child: NativeApp(key: key, catalog: catalog, controller: controller),
        ),
      );
      await tester.pumpAndSettle();
      await tester.runAsync(
        () => precacheImage(
          const AssetImage('assets/window-light-background.png'),
          key.currentContext!,
        ),
      );
      await tester.pumpAndSettle();
      await controller.selectAccount(firstAccount);
      await key.currentState!.go('P09');
      await tester.pumpAndSettle();
      expect(
        core.requests.where((r) => r['method'] == 'inspect_account_inventory'),
        isEmpty,
      );
      final originalCards = jsonEncode(controller.cards);
      Future<void> inspect() async {
        final button = find.text('Inspect Saved Inventory');
        await tester.ensureVisible(button);
        await tester.tap(button);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
      }

      await inspect();
      expect(core.requests.last, {
        'method': 'inspect_account_inventory',
        'account_id': firstAccount,
      });
      expect(find.text('Exact saved card'), findsOneWidget);
      expect(find.text('Mismatched connection card'), findsNothing);
      expect(
        find.text('Saved binding without a matching card'),
        findsNWidgets(2),
      );
      for (final text in [
        'Listed in the last complete inventory',
        'Not listed · live requests blocked',
        'Membership unconfirmed',
      ]) {
        expect(find.text(text), findsOneWidget);
      }
      expect(jsonEncode(controller.cards), originalCards);
      expect(
        core.requests.where(
          (r) => [
            'refresh_account',
            'recover_account',
            'read_state',
            'discover',
          ].contains(r['method']),
        ),
        isEmpty,
      );
      final output = Directory('build/verification/inventory')
        ..createSync(recursive: true);
      Future<void> capture(String name) async {
        await tester.runAsync(() async {
          final image =
              await (boundary.currentContext!.findRenderObject()
                      as RenderRepaintBoundary)
                  .toImage(pixelRatio: 2);
          final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
          File(
            '${output.path}/$name.png',
          ).writeAsBytesSync(bytes!.buffer.asUint8List());
          image.dispose();
        });
      }

      await tester.ensureVisible(
        find.text('Listed in the last complete inventory'),
      );
      await tester.pumpAndSettle();
      await capture('saved-membership');
      for (final code in [
        'SECRET_STORE_LOCKED',
        'RESOURCE_BUSY',
        'CREDENTIAL_CORRUPT',
      ]) {
        core.failure = code;
        await inspect();
        expect(controller.accountInventory, isNull);
        expect((controller.error as NativeFailure).code, code);
        expect(
          find.textContaining('No saved bindings were returned.'),
          findsNothing,
        );
      }
      core.failure = null;
      core.inventory['devices'] = <Json>[];
      await inspect();
      expect(
        find.textContaining('This does not prove the cloud account is empty.'),
        findsOneWidget,
      );
      await capture('empty-saved-inventory');
      tester.view.physicalSize = const Size(640, 1688);
      tester.platformDispatcher.textScaleFactorTestValue = 2;
      addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      await capture('large-text');
      await key.currentState!.go('P01');
      await tester.pumpAndSettle();
      expect(controller.accountInventory, isNull);
      expect(jsonEncode(controller.cards), originalCards);
      debugDisableShadows = shadows;
      await tester.pumpWidget(const SizedBox());
      controller.dispose();
    },
  );

  test(
    'saved inventory discards late success and failure after cancellation and background',
    () async {
      final core = await fixture();
      final controller = HomeController(core, AccountPreferences());
      await controller.initialize();
      await controller.selectAccount(firstAccount);
      for (final background in [false, true]) {
        for (final failure in [null, 'SECRET_STORE_LOCKED']) {
          core.failure = failure;
          core.gate = Completer<void>();
          final pending = controller.inspectAccountInventory();
          if (background) {
            controller.setForeground(false);
          } else {
            controller.cancelPending();
          }
          core.gate!.complete();
          await pending;
          expect(controller.accountInventory, isNull);
          expect(controller.error, isNull);
          controller.setForeground(true);
        }
      }
      core.gate = null;
      core.failure = null;
      for (final state in ['pending', 'removing']) {
        controller.account!['credential_state'] = state;
        final count = core.requests.length;
        expect(await controller.inspectAccountInventory(), false);
        expect(core.requests.length, count);
        expect(
          accountBlocks(controller, 'P09').toString(),
          isNot(contains('Inspect Saved Inventory')),
        );
      }
      controller.dispose();
    },
  );
}
