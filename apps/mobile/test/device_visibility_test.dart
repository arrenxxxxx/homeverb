import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/client_contract.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'accounts_test.dart'
    show AccountFixture, firstAccount, firstDevice, secondDevice;
import 'state_refresh_test.dart' show RefreshFixture;

class RoomAccountFixture extends AccountFixture {
  RoomAccountFixture() {
    for (final card in cards) {
      card['preferences'] = Map<String, dynamic>.from(card['preferences']);
    }
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'assign_devices_to_room') {
      requests.add({'method': method, ...fields});
      final assignment = fields['assignment'];
      for (final card in cards) {
        if ((assignment['device_ids'] as List).contains(card['device']['id'])) {
          card['preferences']['room_id'] = assignment['room_id'];
        }
      }
      return [];
    }
    return super.call(method, fields);
  }
}

void main() {
  test(
    'visibility survives queued edits, disk restart and inventory reload',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'homeverb-visibility-',
      );
      addTearDown(() => directory.delete(recursive: true));
      final file = File('${directory.path}/preferences.json');
      final preferences = LocalPreferences(file);
      await Future.wait([
        preferences.setDeviceVisible(firstDevice, false),
        preferences.save({'skin_id': 'sky_blue'}),
        preferences.setDeviceVisible(secondDevice, false),
      ]);
      final restored = LocalPreferences(file);
      await restored.load();
      expect(restored.deviceVisible(firstDevice), false);
      expect(restored.deviceVisible(secondDevice), false);
      expect(restored.skin, 'sky_blue');
      final core = AccountFixture();
      final c = HomeController(core, restored);
      addTearDown(c.dispose);
      expect(await c.initialize(), true);
      expect(c.visibleCards, isEmpty);
      await c.reloadAccountViews();
      expect(await c.selectAccount(firstAccount), true);
      expect(await c.retrieveAccount(), true);
      expect(c.visibleCards, isEmpty);
      expect(c.cards, hasLength(2));
      expect(await c.setDeviceVisible(firstDevice, true), true);
      expect(c.visibleCards.single['device']['id'], firstDevice);
      final again = LocalPreferences(file);
      await again.load();
      expect(again.deviceVisible(firstDevice), true);
      expect(again.deviceVisible(secondDevice), false);
      expect(
        core.requests.any(
          (call) => [
            'remove_account',
            'execute',
            'set_device_preferences',
          ].contains(call['method']),
        ),
        false,
      );
    },
  );

  test('failed persistence keeps the displayed card and selection', () async {
    final directory = await Directory.systemTemp.createTemp(
      'homeverb-visibility-fail-',
    );
    addTearDown(() => directory.delete(recursive: true));
    final parentFile = File('${directory.path}/not-a-directory');
    await parentFile.writeAsString('fictional');
    final c = HomeController(
      AccountFixture(),
      LocalPreferences(File('${parentFile.path}/preferences.json')),
    );
    addTearDown(c.dispose);
    expect(await c.initialize(), true);
    c.selection.add(firstDevice);
    expect(await c.setDeviceVisible(firstDevice, false), false);
    expect(c.preferences.deviceVisible(firstDevice), true);
    expect(c.selection, contains(firstDevice));
    expect(c.error, isA<FileSystemException>());
  });

  testWidgets('hidden cards are excluded from automatic foreground reads', (
    tester,
  ) async {
    final core = RefreshFixture();
    final c = HomeController(core, LocalPreferences.memory());
    addTearDown(c.dispose);
    expect(await c.initialize(), true);
    expect(await c.setDeviceVisible(core.device['id'], false), true);
    c.setAutomaticRefreshView(StateRefreshView.cards);
    c.startAutomaticRefresh();
    core.calls.clear();
    await tester.pump(const Duration(seconds: 3));
    expect(core.liveReads.map((r) => r['device']), ['second-fictional-device']);
    c.stopAutomaticRefresh();
  });

  testWidgets(
    'card menu assigns only its device to a room without opening reads',
    (tester) async {
      final core = RoomAccountFixture();
      final c = HomeController(core, LocalPreferences.memory());
      final app = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: app, catalog: catalog, controller: c),
      );
      await tester.pumpAndSettle();
      core.requests.clear();
      await tester.tap(find.byKey(const ValueKey('card-menu:$firstDevice')));
      await tester.pumpAndSettle();
      expect(
        tester.getTopLeft(find.text('Assign room')).dy,
        lessThan(tester.getTopLeft(find.text('Remove device')).dy),
      );
      await tester.tap(find.text('Assign room'));
      await tester.pumpAndSettle();
      expect(app.currentState!.id, 'R03');
      expect(c.selection, {firstDevice});
      expect(c.selectedRoom, 'study');
      expect(core.requests, isEmpty);
      await tester.ensureVisible(find.text('Unassigned'));
      await tester.tap(find.text('Unassigned'));
      await tester.pumpAndSettle();
      await tester.ensureVisible(find.text('Move Devices'));
      await tester.tap(find.text('Move Devices'));
      await tester.pumpAndSettle();
      expect(app.currentState!.id, 'D01');
      expect(
        core.requests
            .where((r) => r['method'] == 'assign_devices_to_room')
            .single['assignment'],
        {
          'device_ids': [firstDevice],
          'room_id': null,
        },
      );
      expect(core.cards.first['preferences']['room_id'], isNull);
      expect(core.cards.last['preferences']['room_id'], 'study');
      expect(c.preferences.deviceVisible(firstDevice), true);
      expect(core.requests.any((r) => r['method'] == 'execute'), false);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets(
    'card removal and account selection share exact device visibility',
    (tester) async {
      final core = AccountFixture();
      final c = HomeController(core, LocalPreferences.memory());
      final app = GlobalKey<NativeAppState>();
      final catalog = (await tester.runAsync(ScreenCatalog.load))!;
      await tester.pumpWidget(
        NativeApp(key: app, catalog: catalog, controller: c),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('card-menu:$firstDevice')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Remove device'));
      await tester.pumpAndSettle();
      expect(
        find.byKey(const ValueKey('card-menu:$firstDevice')),
        findsNothing,
      );
      expect(
        find.byKey(const ValueKey('card-menu:$secondDevice')),
        findsOneWidget,
      );
      await app.currentState!.action('Platforms');
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('account:$firstAccount')));
      await tester.pumpAndSettle();
      final checkbox = find.byKey(
        const ValueKey('device-visible:$firstDevice'),
      );
      await tester.ensureVisible(checkbox);
      expect(tester.widget<Checkbox>(checkbox).value, false);
      await tester.tap(checkbox);
      await tester.pumpAndSettle();
      expect(app.currentState!.id, 'P09');
      expect(tester.widget<Checkbox>(checkbox).value, true);
      await tester.tap(checkbox);
      await tester.pumpAndSettle();
      expect(tester.widget<Checkbox>(checkbox).value, false);
      await tester.tap(checkbox);
      await tester.pumpAndSettle();
      await app.currentState!.action('Devices');
      await tester.pumpAndSettle();
      expect(
        find.byKey(const ValueKey('card-menu:$firstDevice')),
        findsOneWidget,
      );
      expect(
        core.requests.any(
          (r) => r['method'] == 'execute' || r['method'] == 'remove_account',
        ),
        false,
      );
      for (final device in [firstDevice, secondDevice]) {
        await tester.tap(find.byKey(ValueKey('card-menu:$device')));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Remove device'));
        await tester.pumpAndSettle();
      }
      expect(
        find.text('Choose displayed devices in Platforms'),
        findsOneWidget,
      );
      expect(c.cards, hasLength(2));
      expect(c.visibleCards, isEmpty);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );
}
