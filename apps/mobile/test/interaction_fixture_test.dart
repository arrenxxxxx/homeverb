import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/native_client.dart';

import 'interaction_fixture.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const fridge = {
    'device': 'fictional-a',
    'component': 'fridge',
    'cached': false,
  };
  const freezer = {
    'device': 'fictional-a',
    'component': 'freezer',
    'cached': false,
  };

  testWidgets('held reads remain pending for five seconds and release once', (
    tester,
  ) async {
    final client = await InteractionFixture.open();
    final gate = client.hold('fictional-a', 'fridge');
    var completed = false;
    final read = client.call('read_state', fridge).then((result) {
      completed = true;
      return result;
    });
    await tester.pump(const Duration(seconds: 5));
    expect(completed, false);
    expect(client.activeReads, 1);
    expect(client.callsFor('read_state'), hasLength(1));
    gate.complete();
    await tester.pump();
    expect((await read)['component_id'], 'fridge');
    expect(client.activeReads, 0);
    expect(client.callsFor('execute'), isEmpty);
  });

  testWidgets('component failures are independent and deadlines are bounded', (
    tester,
  ) async {
    final client = await InteractionFixture.open();
    client.failures.add(('fictional-a', 'fridge'));
    final gate = client.hold('fictional-a', 'fridge');
    final failed = client.call('read_state', fridge);
    final failure = expectLater(failed, throwsA(isA<NativeFailure>()));
    final succeeded = await client.call('read_state', freezer);
    expect(succeeded['component_id'], 'freezer');
    await tester.pump(const Duration(seconds: 5));
    gate.complete();
    await tester.pump();
    await failure;
    client.deadline = const Duration(seconds: 2);
    client.hold('fictional-b', 'fridge');
    final timed = client.call('read_state', {
      ...fridge,
      'device': 'fictional-b',
    });
    final timeout = expectLater(timed, throwsA(isA<TimeoutException>()));
    await tester.pump(const Duration(seconds: 2));
    await timeout;
    expect(client.activeReads, 0);
    expect(client.callsFor('read_state'), hasLength(3));
    expect(client.callsFor('execute'), isEmpty);
  });

  test(
    'reopened sessions restore only their profile and device snapshots',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'interaction-fixture-',
      );
      addTearDown(() => directory.delete(recursive: true));
      final file = File('${directory.path}/fictional.json');
      final first = await InteractionFixture.open(profileFile: file);
      first.values[('fictional-a', 'fridge')] = 7;
      final saved = await first.call('read_state', fridge) as Json;
      await first.close();
      expect(first.closeCount, 1);
      await expectLater(first.call('read_state', fridge), throwsStateError);
      final restarted = await InteractionFixture.open(profileFile: file);
      final cached =
          await restarted.call('read_state', {...fridge, 'cached': true})
              as Json;
      expect(cached['observations'], saved['observations']);
      expect(cached['from_cache'], true);
      await expectLater(
        restarted.call('read_state', {...freezer, 'cached': true}),
        throwsA(isA<NativeFailure>()),
      );
      await expectLater(
        restarted.call('read_state', {
          ...fridge,
          'device': 'fictional-b',
          'cached': true,
        }),
        throwsA(isA<NativeFailure>()),
      );
      final other = await InteractionFixture.open(
        profileFile: file,
        profile: 'other-fictional-profile',
      );
      await expectLater(
        other.call('read_state', {...fridge, 'cached': true}),
        throwsA(isA<NativeFailure>()),
      );
      expect(restarted.callsFor('read_state'), hasLength(3));
      expect(restarted.callsFor('execute'), isEmpty);
    },
  );
}
