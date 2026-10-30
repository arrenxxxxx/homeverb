import 'dart:ffi';
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/native_client.dart';

const accountId = '00000000-0000-4000-8000-000000000001';
void main() {
  late Directory directory;
  late String path, oldPath;
  late void Function(int) mode;
  late void Function() release;
  late int Function() entered, discarded, begins, continues, sessions;
  setUpAll(() async {
    directory = await Directory.systemTemp.createTemp('homeverb-modern-abi-');
    path = '${directory.path}/${NativeClient.defaultLibrary()}';
    oldPath = '${directory.path}/old-${NativeClient.defaultLibrary()}';
    for (final old in [false, true]) {
      final result = await Process.run('cc', [
        Platform.isMacOS ? '-dynamiclib' : '-shared',
        '-fPIC',
        '-std=c11',
        '-D_DEFAULT_SOURCE',
        if (old) '-DOMIT_MODERN',
        'test/fixtures/modern_account_bridge.c',
        '-o',
        old ? oldPath : path,
      ]);
      expect(result.exitCode, 0, reason: '${result.stderr}');
    }
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
    discarded = library.lookupFunction<Int Function(), int Function()>(
      'fixture_discarded',
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
  Future<NativeClient> open({bool old = false}) => NativeClient.open(
    dataDir: directory.path,
    libraryPath: old ? oldPath : path,
    adapter: 'xiaomi.mijia',
  );
  Future<void> waitEntered() async {
    for (var i = 0; i < 500 && entered() == 0; i++) {
      await Future<void>.delayed(const Duration(milliseconds: 2));
    }
    expect(entered(), 1);
  }

  test(
    'protected binary scan and explicit operations use separate native entry points',
    () async {
      mode(0);
      final client = await open();
      try {
        expect(client.supportsModernAccountLogin, true);
        final qr = await client.beginModernAccountLogin(label: 'Home');
        expect(qr.toString(), 'ModernAccountQr([REDACTED])');
        expect(qr.scanData, 'https://fictional.invalid/PRIVATE'.codeUnits);
        expect(qr.remaining.inMilliseconds, inInclusiveRange(1, 179000));
        expect(continues(), 0);
        expect(sessions(), 0);
        final connected = await client.continueModernAccountLogin(qr);
        expect(connected.kind, ModernAccountResultKind.connected);
        expect(connected.account['id'], accountId);
        expect(connected.directory!.count, 3);
        expect(qr.scanData.every((b) => b == 0), true);
        expect(continues(), 1);
        await expectLater(
          client.continueModernAccountLogin(qr),
          throwsA(isA<NativeFailure>()),
        );
        expect(continues(), 1);
        for (final operation in ModernAccountOperation.values) {
          final result = await client.modernAccountSession(
            accountId,
            operation,
          );
          expect(
            result.kind,
            operation == ModernAccountOperation.renew
                ? ModernAccountResultKind.renewed
                : ModernAccountResultKind.directory,
          );
        }
        expect(sessions(), 3);
        expect(begins(), 1);
      } finally {
        await client.close();
      }
    },
  );
  test('old library without QR symbols refuses before symbol lookup', () async {
    final client = await open(old: true);
    try {
      expect(client.supportsModernAccountLogin, false);
      await expectLater(
        client.beginModernAccountLogin(label: 'Home'),
        throwsA(
          isA<NativeFailure>().having(
            (e) => e.code,
            'code',
            'API_UPGRADE_REQUIRED',
          ),
        ),
      );
      await expectLater(
        client.modernAccountSession(accountId, ModernAccountOperation.renew),
        throwsA(isA<NativeFailure>()),
      );
    } finally {
      await client.close();
    }
  });
  test(
    'malformed and private QR responses discard their native handles',
    () async {
      for (final value in [1, 2, 3, 4]) {
        mode(value);
        final client = await open();
        try {
          await expectLater(
            client.beginModernAccountLogin(label: 'Home'),
            throwsA(isA<NativeFailure>()),
          );
          expect(discarded(), 1);
          expect(client.active, isEmpty);
        } finally {
          await client.close();
        }
      }
    },
  );
  test(
    'foreign client cannot consume or clear another client challenge',
    () async {
      mode(0);
      final first = await open(), second = await open();
      try {
        final qr = await first.beginModernAccountLogin(label: 'Home');
        await expectLater(
          second.continueModernAccountLogin(qr),
          throwsA(isA<NativeFailure>()),
        );
        await second.discardModernAccountLogin(qr);
        expect(qr.scanData.any((b) => b != 0), true);
        expect(continues(), 0);
        await first.discardModernAccountLogin(qr);
        expect(qr.scanData.every((b) => b == 0), true);
        expect(discarded(), 1);
      } finally {
        await first.close();
        await second.close();
      }
    },
  );
  test(
    'background cancel and close clear pending scan memory without continuation',
    () async {
      for (final lifecycle in [0, 1, 2]) {
        mode(0);
        final client = await open();
        final qr = await client.beginModernAccountLogin(label: 'Home');
        if (lifecycle == 0)
          client.foreground(false);
        else if (lifecycle == 1)
          client.cancelAll();
        else
          await client.close();
        expect(qr.scanData.every((b) => b == 0), true);
        await expectLater(
          client.continueModernAccountLogin(qr),
          throwsA(isA<NativeFailure>()),
        );
        expect(continues(), 0);
        await client.close();
      }
    },
  );
  test(
    'late begin cannot restore scan material after lifecycle invalidation',
    () async {
      for (final lifecycle in [0, 1, 2]) {
        mode(5);
        final client = await open();
        final result = client.beginModernAccountLogin(label: 'Home');
        final check = expectLater(result, throwsA(isA<NativeFailure>()));
        await waitEntered();
        if (lifecycle == 0)
          client.foreground(false);
        else if (lifecycle == 1)
          client.cancelAll();
        else
          await client.close();
        release();
        await check;
        expect(client.active, isEmpty);
        expect(continues(), 0);
        await client.close();
      }
    },
  );
  test(
    'late continuation and saved-session results are rejected after cancellation',
    () async {
      for (final phase in [6, 7]) {
        mode(0);
        final client = await open();
        final qr = await client.beginModernAccountLogin(label: 'Home');
        mode(phase);
        final result = phase == 6
            ? client.continueModernAccountLogin(qr)
            : client.modernAccountSession(
                accountId,
                ModernAccountOperation.renew,
              );
        final check = expectLater(result, throwsA(isA<NativeFailure>()));
        await waitEntered();
        client.cancelAll();
        release();
        await check;
        expect(qr.scanData.every((b) => b == 0), true);
        expect(client.active, isEmpty);
        await client.close();
      }
    },
  );
  test(
    'closed result parser refuses wrong provider completeness counts and account identity',
    () async {
      for (final value in [8, 9, 10, 11]) {
        mode(0);
        final client = await open();
        try {
          final qr = await client.beginModernAccountLogin(
            label: 'Home',
            accountId: accountId,
          );
          mode(value);
          await expectLater(
            client.continueModernAccountLogin(qr),
            throwsA(isA<NativeFailure>()),
          );
          await expectLater(
            client.modernAccountSession(
              accountId,
              ModernAccountOperation.refresh,
            ),
            throwsA(isA<NativeFailure>()),
          );
          expect(qr.scanData.every((b) => b == 0), true);
        } finally {
          await client.close();
        }
      }
    },
  );
  test(
    'directory failure preserves successful authentication and never retries',
    () async {
      mode(12);
      final client = await open();
      try {
        final qr = await client.beginModernAccountLogin(label: 'Home');
        final result = await client.continueModernAccountLogin(qr);
        expect(result.kind, ModernAccountResultKind.connected);
        expect(result.directory!.failed, true);
        expect(result.directory!.count, 0);
        expect(result.directory!.error!['code'], 'MIJIA_REQUEST_FAILED');
        expect(begins(), 1);
        expect(continues(), 1);
        expect(sessions(), 0);
      } finally {
        await client.close();
      }
    },
  );
  test(
    'expired challenge clears bytes and refuses before native polling',
    () async {
      mode(14);
      final client = await open();
      try {
        final qr = await client.beginModernAccountLogin(label: 'Home');
        await Future<void>.delayed(const Duration(milliseconds: 10));
        await expectLater(
          client.continueModernAccountLogin(qr),
          throwsA(isA<NativeFailure>()),
        );
        expect(continues(), 0);
        expect(qr.scanData.every((b) => b == 0), true);
        expect(discarded(), 1);
      } finally {
        await client.close();
      }
    },
  );
}
