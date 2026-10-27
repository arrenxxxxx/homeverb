import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/native_client.dart';

void main() {
  late Directory directory;
  late String path;
  late void Function(int) mode;
  late void Function() release;
  late int Function() entered, discarded;
  setUpAll(() async {
    directory = await Directory.systemTemp.createTemp(
      'homeverb-verification-abi-',
    );
    path = '${directory.path}/${NativeClient.defaultLibrary()}';
    final result = await Process.run('cc', [
      Platform.isMacOS ? '-dynamiclib' : '-shared',
      '-fPIC',
      '-std=c11',
      '-D_DEFAULT_SOURCE',
      'test/fixtures/account_verification_bridge.c',
      '-o',
      path,
    ]);
    expect(result.exitCode, 0, reason: '${result.stderr}');
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
  });
  tearDownAll(() async => directory.delete(recursive: true));
  Future<NativeClient> open() => NativeClient.open(
    dataDir: directory.path,
    libraryPath: path,
    adapter: 'xiaomi.mijia',
  );
  Future<AccountLoginProgress> begin(NativeClient client) async {
    final login = Uint8List.fromList('fiction'.codeUnits),
        password = Uint8List.fromList('Fiction!'.codeUnits);
    try {
      return await client.beginAccountVerification(
        login: login,
        password: password,
        label: 'Home',
      );
    } finally {
      expect(login.every((b) => b == 0), true);
      expect(password.every((b) => b == 0), true);
    }
  }

  test(
    'four-export ABI carries binary prompts and explicit method with cleared answers',
    () async {
      mode(0);
      final client = await open();
      try {
        final captcha = await begin(client) as AccountVerificationPending;
        expect(captcha.kind, AccountVerificationKind.captcha);
        expect(captcha.image, 'fictional-image'.codeUnits);
        final answer = Uint8List.fromList('Ab12'.codeUnits);
        final prompt =
            await client.answerAccountVerification(captcha, answer)
                as AccountVerificationPending;
        expect(answer.every((b) => b == 0), true);
        expect(captcha.image.every((b) => b == 0), true);
        expect(prompt.image, isEmpty);
        expect(prompt.methods, AccountVerificationMethod.values);
        final sent =
            await client.requestAccountVerificationCode(
                  prompt,
                  AccountVerificationMethod.email,
                )
                as AccountVerificationPending;
        expect(sent.selectedMethod, AccountVerificationMethod.email);
        expect(sent.codeRequests, 1);
        expect(sent.cooldown.inMilliseconds, inInclusiveRange(59000, 60000));
        expect(sent.image, isEmpty);
        final ticket = Uint8List.fromList('123456'.codeUnits);
        final connected =
            await client.answerAccountVerification(
                  sent,
                  ticket,
                  method: AccountVerificationMethod.email,
                )
                as AccountLoginConnected;
        expect(connected.report, {'fictional': true});
        expect(ticket.every((b) => b == 0), true);
        await expectLater(
          client.requestAccountVerificationCode(
            sent,
            AccountVerificationMethod.email,
          ),
          throwsA(isA<NativeFailure>()),
        );
      } finally {
        await client.close();
      }
    },
  );
  for (final variant in [2, 3, 4]) {
    test('malformed native prompt $variant discards handle', () async {
      mode(variant);
      final client = await open();
      try {
        await expectLater(
          begin(client),
          throwsA(
            isA<NativeFailure>().having(
              (e) => e.code,
              'code',
              'UNSUPPORTED_OUTCOME',
            ),
          ),
        );
        expect(discarded(), 1);
      } finally {
        await client.close();
      }
    });
  }
  test('generation invalidation discards a late native prompt', () async {
    mode(5);
    final client = await open();
    try {
      final future = begin(client);
      final assertion = expectLater(
        future,
        throwsA(
          isA<NativeFailure>().having((e) => e.code, 'code', 'CANCELLED'),
        ),
      );
      final timeout = Stopwatch()..start();
      while (entered() == 0 && timeout.elapsed < const Duration(seconds: 5)) {
        await Future<void>.delayed(const Duration(milliseconds: 1));
      }
      expect(entered(), 1);
      client.cancelAll();
      release();
      await assertion;
      expect(discarded(), 1);
      expect(client.active, isEmpty);
    } finally {
      release();
      await client.close();
    }
  });
  test('foreign session cannot answer a pending challenge', () async {
    mode(0);
    final first = await open(), second = await open();
    try {
      final pending = await begin(first) as AccountVerificationPending;
      final bytes = Uint8List.fromList('Ab12'.codeUnits);
      await expectLater(
        second.answerAccountVerification(pending, bytes),
        throwsA(isA<NativeFailure>()),
      );
      expect(bytes.every((b) => b == 0), true);
      await first.discardAccountVerification(pending);
      expect(pending.image.every((b) => b == 0), true);
    } finally {
      await first.close();
      await second.close();
    }
  });
}
