import 'dart:ffi';
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/native_client.dart';

const accountId = '00000000-0000-4000-8000-000000000001';
void main() {
  late Directory directory;
  late String path;
  late void Function(int) mode;
  late void Function() release;
  late int Function() entered, begins, continues, sessions;
  setUpAll(() async {
    directory = await Directory.systemTemp.createTemp(
      'homeverb-modern-devices-',
    );
    path = '${directory.path}/${NativeClient.defaultLibrary()}';
    final compile = await Process.run('cc', [
      Platform.isMacOS ? '-dynamiclib' : '-shared',
      '-fPIC',
      '-std=c11',
      '-D_DEFAULT_SOURCE',
      '-DPUBLISH_DEVICES',
      'test/fixtures/modern_account_bridge.c',
      '-o',
      path,
    ]);
    expect(compile.exitCode, 0, reason: '${compile.stderr}');
    final library = DynamicLibrary.open(path);
    mode = library.lookupFunction<Void Function(Int), void Function(int)>(
      'fixture_mode',
    );
    release = library.lookupFunction<Void Function(), void Function()>(
      'fixture_release',
    );
    entered = library.lookupFunction<Int Function(), int Function()>(
      'fixture_entered',
    );
    begins = library.lookupFunction<Int Function(), int Function()>(
      'fixture_begins',
    );
    continues = library.lookupFunction<Int Function(), int Function()>(
      'fixture_continues',
    );
    sessions = library.lookupFunction<Int Function(), int Function()>(
      'fixture_sessions',
    );
  });
  tearDownAll(() async => directory.delete(recursive: true));
  Future<NativeClient> open() => NativeClient.open(
    dataDir: directory.path,
    libraryPath: path,
    adapter: 'xiaomi.mijia',
  );
  Future<void> waitEntered() async {
    for (var i = 0; i < 500 && entered() == 0; i++) {
      await Future<void>.delayed(const Duration(milliseconds: 2));
    }
    expect(entered(), 1);
  }

  test(
    'API 1.37 opts into device continuation and explicit device operations',
    () async {
      mode(0);
      final client = await open();
      try {
        final qr = await client.beginModernAccountLogin(
          label: 'Home',
          accountId: accountId,
        );
        final result = await client.continueModernAccountLogin(qr);
        expect(result.kind, ModernAccountResultKind.connectedDevices);
        expect(result.directory, isNull);
        expect(result.deviceReport!['authentication'], 'succeeded');
        expect(result.deviceReport!['retrieval']['status'], 'partial');
        expect(result.deviceReport!['retrieval']['device_ids'], [
          '40000000-0000-4000-8000-000000000001',
        ]);
        expect(qr.scanData.every((v) => v == 0), true);
        expect(
          () => result.deviceReport!['retrieval']['device_ids'].add('other'),
          throwsUnsupportedError,
        );
        expect(
          () => result.deviceReport!['account']['id'] = 'other',
          throwsUnsupportedError,
        );
        for (final operation in ModernAccountOperation.values) {
          final response = await client.modernAccountSession(
            accountId,
            operation,
          );
          expect(
            response.kind,
            operation == ModernAccountOperation.renew
                ? ModernAccountResultKind.renewed
                : ModernAccountResultKind.devices,
          );
          if (operation == ModernAccountOperation.renew) {
            expect(response.deviceReport, isNull);
          } else {
            expect(response.directory, isNull);
            expect(response.deviceReport!['authentication'], 'not_attempted');
          }
        }
        expect(begins(), 1);
        expect(continues(), 1);
        expect(sessions(), 3);
      } finally {
        await client.close();
      }
    },
  );

  test(
    'partial empty and failed device reports retain errors without retry',
    () async {
      for (final value in [12, 22]) {
        mode(value);
        final client = await open();
        try {
          final qr = await client.beginModernAccountLogin(label: 'Home');
          final result = await client.continueModernAccountLogin(qr);
          expect(result.deviceReport!['authentication'], 'succeeded');
          expect(
            result.deviceReport!['retrieval']['status'],
            value == 12 ? 'failed' : 'partial',
          );
          expect(result.deviceReport!['retrieval']['device_ids'], isEmpty);
          expect(
            result.deviceReport!['retrieval']['error']['code'],
            value == 12 ? 'MIJIA_REQUEST_FAILED' : 'MIJIA_INVENTORY_PARTIAL',
          );
          expect(begins(), 1);
          expect(continues(), 1);
          expect(sessions(), 0);
        } finally {
          await client.close();
        }
      }
    },
  );

  test(
    'device reports reject completeness identity authority duplicate bounds and wrong result kinds',
    () async {
      for (final value in [8, 9, 10, 11, 15, 16, 17, 18, 19, 20, 21]) {
        mode(0);
        final client = await open();
        try {
          final qr = await client.beginModernAccountLogin(
            label: 'Home',
            accountId: accountId,
          );
          mode(value);
          for (final work in [
            () => client.continueModernAccountLogin(qr),
            () => client.modernAccountSession(
              accountId,
              ModernAccountOperation.refresh,
            ),
            () => client.modernAccountSession(
              accountId,
              ModernAccountOperation.recover,
            ),
          ]) {
            await expectLater(
              work(),
              throwsA(
                isA<NativeFailure>().having(
                  (e) => e.code,
                  'code',
                  'UNSUPPORTED_OUTCOME',
                ),
              ),
              reason: 'fixture mode $value',
            );
          }
          expect(qr.scanData.every((v) => v == 0), true);
          expect(client.active, isEmpty);
        } finally {
          await client.close();
        }
      }
    },
  );

  test(
    'late device publication results cannot cross cancel background or close',
    () async {
      for (final phase in [6, 7]) {
        for (final ending in ['cancel', 'background', 'close']) {
          mode(0);
          final client = await open();
          final qr = await client.beginModernAccountLogin(label: 'Home');
          mode(phase);
          final work = phase == 6
              ? client.continueModernAccountLogin(qr)
              : client.modernAccountSession(
                  accountId,
                  ModernAccountOperation.refresh,
                );
          final check = expectLater(work, throwsA(isA<NativeFailure>()));
          await waitEntered();
          if (ending == 'cancel') {
            client.cancelAll();
          } else if (ending == 'background') {
            client.foreground(false);
          } else {
            await client.close();
          }
          release();
          await check;
          expect(qr.scanData.every((v) => v == 0), true);
          expect(client.active, isEmpty);
          await client.close();
        }
      }
    },
  );
}
