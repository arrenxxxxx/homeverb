import 'dart:ffi';
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/native_client.dart';

void main() {
  for (final mode in ['old', 'missing', 'current', 'default']) {
    test('event startup negotiates before enabled open: $mode', () async {
      final dir = await Directory.systemTemp.createTemp(
        'homeverb-event-bridge-',
      );
      try {
        final path = '${dir.path}/${NativeClient.defaultLibrary()}';
        final built = await Process.run('cc', [
          Platform.isMacOS ? '-dynamiclib' : '-shared',
          '-fPIC',
          '-std=c11',
          '-D_DEFAULT_SOURCE',
          if (mode == 'old') '-DFIXTURE_API_VERSION="1.43.0"',
          if (mode == 'missing') '-DFIXTURE_METHODS=""',
          'test/fixtures/event_startup_bridge.c',
          '-o',
          path,
        ]);
        expect(built.exitCode, 0, reason: '${built.stderr}');
        final lib = DynamicLibrary.open(path);
        int count(String key) =>
            lib.lookupFunction<Uint32 Function(), int Function()>(key)();
        final work = NativeClient.open(
          dataDir: dir.path,
          libraryPath: path,
          adapter: 'xiaomi.mijia',
          mijiaEventHistory: mode != 'default',
        );
        if (['old', 'missing'].contains(mode)) {
          await expectLater(
            work,
            throwsA(
              isA<NativeFailure>().having(
                (e) => e.code,
                'code',
                'API_UPGRADE_REQUIRED',
              ),
            ),
          );
          expect(count('fixture_opened'), 1);
          expect(count('fixture_closed'), 1);
          expect(count('fixture_enabled'), 0);
        } else {
          final client = await work;
          expect(client.apiAtLeast(44), true);
          expect(count('fixture_opened'), mode == 'current' ? 2 : 1);
          expect(count('fixture_enabled'), mode == 'current' ? 1 : 0);
          await client.close();
          expect(count('fixture_closed'), count('fixture_opened'));
        }
      } finally {
        await dir.delete(recursive: true);
      }
    });
  }
  test(
    'actual native 1.44 enables empty Mijia compositions without account work',
    () async {
      final library = File(
        '../../target/debug/${NativeClient.defaultLibrary()}',
      ).absolute.path;
      for (final adapter in ['xiaomi.mijia', 'homeverb.platforms']) {
        final dir = await Directory.systemTemp.createTemp(
          'homeverb-event-native-',
        );
        try {
          final client = await NativeClient.open(
            dataDir: dir.path,
            libraryPath: library,
            adapter: adapter,
            mijiaEventHistory: true,
          );
          try {
            expect(client.apiAtLeast(44), true);
            expect(
              client.methods,
              containsAll([
                'inspect_event_history_options',
                'read_event_history',
              ]),
            );
            expect(await client.call('list_accounts'), isEmpty);
            expect(await client.call('list_devices'), isEmpty);
            await expectLater(
              client.call('inspect_event_history_options', {
                'device_id': 'fictional',
                'connection_id': 'cloud',
              }),
              throwsA(
                isA<NativeFailure>().having(
                  (e) => e.code,
                  'code',
                  'DEVICE_NOT_FOUND',
                ),
              ),
            );
          } finally {
            await client.close();
          }
        } finally {
          await dir.delete(recursive: true);
        }
      }
    },
  );
  test(
    'invalid event startup adapter fails before opening the library',
    () async {
      for (final adapter in ['homeverb.mock', 'midea.meiju']) {
        await expectLater(
          NativeClient.open(
            dataDir: '/unused',
            libraryPath: '/must-not-load',
            adapter: adapter,
            mijiaEventHistory: true,
          ),
          throwsA(
            isA<NativeFailure>().having(
              (e) => e.code,
              'code',
              'INVALID_ARGUMENT',
            ),
          ),
        );
      }
    },
  );
}
