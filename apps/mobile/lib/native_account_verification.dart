part of 'native_client.dart';

const _verificationPromptCapacity = 131072;

class _VerificationBindings {
  _VerificationBindings(this.base);
  final NativeBindings base;
  late final begin = base.library
      .lookupFunction<
        Pointer<Utf8> Function(
          Uint64,
          Pointer<Utf8>,
          Pointer<Uint8>,
          Size,
          Pointer<Uint8>,
          Size,
          Pointer<Uint8>,
          Size,
        ),
        Pointer<Utf8> Function(
          int,
          Pointer<Utf8>,
          Pointer<Uint8>,
          int,
          Pointer<Uint8>,
          int,
          Pointer<Uint8>,
          int,
        )
      >('homeverb_begin_account_verification');
  late final answer = base.library
      .lookupFunction<
        Pointer<Utf8> Function(
          Uint64,
          Pointer<Utf8>,
          Pointer<Uint8>,
          Size,
          Pointer<Uint8>,
          Size,
        ),
        Pointer<Utf8> Function(
          int,
          Pointer<Utf8>,
          Pointer<Uint8>,
          int,
          Pointer<Uint8>,
          int,
        )
      >('homeverb_continue_account_verification');
  late final requestCode = base.library
      .lookupFunction<
        Pointer<Utf8> Function(Uint64, Pointer<Utf8>, Pointer<Uint8>, Size),
        Pointer<Utf8> Function(int, Pointer<Utf8>, Pointer<Uint8>, int)
      >('homeverb_request_account_verification_code');
  late final discard = base.library
      .lookupFunction<Bool Function(Uint64, Uint64), bool Function(int, int)>(
        'homeverb_discard_account_verification',
      );

  Json invoke(
    int handle,
    String action,
    Json metadata,
    int first,
    int firstLength,
    int second,
    int secondLength,
    int prompt,
  ) {
    final input = jsonEncode(metadata).toNativeUtf8();
    Pointer<Utf8>? output;
    final destination = Pointer<Uint8>.fromAddress(prompt);
    try {
      output = switch (action) {
        'begin' => begin(
          handle,
          input,
          Pointer<Uint8>.fromAddress(first),
          firstLength,
          Pointer<Uint8>.fromAddress(second),
          secondLength,
          destination,
          _verificationPromptCapacity,
        ),
        'answer' => answer(
          handle,
          input,
          Pointer<Uint8>.fromAddress(first),
          firstLength,
          destination,
          _verificationPromptCapacity,
        ),
        'send' => requestCode(
          handle,
          input,
          destination,
          _verificationPromptCapacity,
        ),
        _ => throw NativeFailure(
          'INVALID_ARGUMENT',
          'Invalid verification operation.',
        ),
      };
      if (output == nullptr) throw NativeFailure('NATIVE_ERROR', 'No response');
      return jsonDecode(output.toDartString()) as Json;
    } finally {
      calloc.free(input);
      if (output != null && output != nullptr) base.free(output);
    }
  }
}

class _NativeVerificationPending extends AccountVerificationPending {
  _NativeVerificationPending(
    this.id,
    this.kind,
    this.image,
    this.methods,
    this.lifetime,
    this.delay,
    this.codeRequests,
    this.selectedMethod,
  );
  final int id;
  @override
  final AccountVerificationKind kind;
  @override
  final Uint8List image;
  @override
  final List<AccountVerificationMethod> methods;
  @override
  final AccountVerificationMethod? selectedMethod;
  final Duration lifetime, delay;
  @override
  final int codeRequests;
  final clock = Stopwatch()..start();
  Duration left(Duration duration) {
    final remaining = duration - clock.elapsed;
    return remaining.isNegative ? Duration.zero : remaining;
  }

  @override
  Duration get remaining => left(lifetime);
  @override
  Duration get cooldown => left(delay);
  @override
  void clearImage() => image.fillRange(0, image.length, 0);
}

mixin _NativeAccountVerification {
  NativeClient get _verificationClient => this as NativeClient;
  final Set<_NativeVerificationPending> _verificationChallenges = {};
  int _verificationGeneration = 0;
  bool get supportsAccountVerification =>
      _verificationClient.apiAtLeast(32) &&
      _verificationClient.accountProviders.contains('xiaomi.mijia');
  void _clearVerificationChallenges() {
    _verificationGeneration++;
    for (final pending in _verificationChallenges) {
      pending.clearImage();
    }
    _verificationChallenges.clear();
  }

  void _discardVerificationId(int id) {
    if (!_verificationClient._closed)
      _VerificationBindings(
        _verificationClient.bindings,
      ).discard(_verificationClient.handle, id);
  }

  AccountLoginProgress _verificationProgress(
    Json value,
    Pointer<Uint8> output,
    int generation,
    AccountVerificationMethod? selected,
    bool sent,
  ) {
    final client = _verificationClient;
    final id = value['challenge_id'];
    var retained = false;
    Uint8List? image;
    try {
      if (client._closed ||
          !client._foreground ||
          generation != _verificationGeneration)
        throw NativeFailure('CANCELLED', 'Sign-in was cancelled.');
      if (value['kind'] == 'connected' && value['report'] is Json && !sent)
        return AccountLoginConnected(value['report'] as Json);
      final kind = value['kind'],
          length = value['prompt_length'],
          lifetime = value['expires_in_ms'];
      if (id is! int ||
          id <= 0 ||
          length is! int ||
          length < 1 ||
          lifetime is! int ||
          lifetime < 1 ||
          lifetime > 180000 ||
          !['captcha', 'verification'].contains(kind))
        throw const FormatException();
      final allowed = {
        'kind',
        'challenge_id',
        'prompt_length',
        'expires_in_ms',
        if (kind == 'captcha') 'media_type' else 'methods',
        if (sent) ...['code_requests', 'cooldown_ms'],
      };
      if (value.keys.any((key) => !allowed.contains(key)))
        throw const FormatException();
      final methods = <AccountVerificationMethod>[];
      if (kind == 'captcha') {
        if (sent ||
            length > _verificationPromptCapacity ||
            !['image/png', 'image/jpeg'].contains(value['media_type']))
          throw const FormatException();
        image = Uint8List.fromList(output.asTypedList(length));
      } else {
        final offered = value['methods'];
        if (length > 8192 ||
            offered is! List ||
            offered.isEmpty ||
            offered.length > 2)
          throw const FormatException();
        for (final method in offered) {
          final decoded = switch (method) {
            'phone' => AccountVerificationMethod.phone,
            'email' => AccountVerificationMethod.email,
            _ => throw const FormatException(),
          };
          if (methods.contains(decoded)) throw const FormatException();
          methods.add(decoded);
        }
        // Native code delivery needs no browser URL. Leave it in the cleared C buffer.
        image = Uint8List(0);
      }
      final requests = sent ? value['code_requests'] : 0,
          delay = sent ? value['cooldown_ms'] : 0;
      if (requests is! int ||
          requests < 0 ||
          requests > 3 ||
          (sent && requests < 1) ||
          delay is! int ||
          delay < 0 ||
          delay > 86400000)
        throw const FormatException();
      final pending = _NativeVerificationPending(
        id,
        kind == 'captcha'
            ? AccountVerificationKind.captcha
            : AccountVerificationKind.verification,
        image,
        List.unmodifiable(methods),
        Duration(milliseconds: lifetime),
        Duration(milliseconds: delay),
        requests,
        selected,
      );
      _verificationChallenges.add(pending);
      retained = true;
      return pending;
    } on FormatException {
      throw NativeFailure(
        'UNSUPPORTED_OUTCOME',
        'Unsupported sign-in response.',
      );
    } finally {
      if (!retained) {
        image?.fillRange(0, image.length, 0);
        if (id is int && id > 0) _discardVerificationId(id);
      }
    }
  }

  Future<AccountLoginProgress> beginAccountVerification({
    required Uint8List login,
    required Uint8List password,
    required String label,
    String? accountId,
  }) => _verificationOperation(
    'begin',
    login: login,
    password: password,
    label: label,
    accountId: accountId,
  );
  Future<AccountLoginProgress> answerAccountVerification(
    AccountVerificationPending challenge,
    Uint8List answer, {
    AccountVerificationMethod? method,
  }) => _verificationOperation(
    'answer',
    pending: challenge,
    answer: answer,
    method: method,
  );
  Future<AccountLoginProgress> requestAccountVerificationCode(
    AccountVerificationPending challenge,
    AccountVerificationMethod method,
  ) => _verificationOperation('send', pending: challenge, method: method);

  Future<AccountLoginProgress> _verificationOperation(
    String action, {
    Uint8List? login,
    Uint8List? password,
    String? label,
    String? accountId,
    AccountVerificationPending? pending,
    Uint8List? answer,
    AccountVerificationMethod? method,
  }) async {
    final client = _verificationClient, generation = _verificationGeneration;
    Pointer<Uint8>? first, second, prompt;
    final firstBytes = action == 'begin' ? login : answer;
    final firstLength = firstBytes?.length ?? 0,
        secondLength = password?.length ?? 0;
    String? operation;
    var owned = false;
    try {
      if (!supportsAccountVerification)
        throw NativeFailure(
          'API_UPGRADE_REQUIRED',
          'Account verification requires native API 1.32.',
        );
      if (action == 'begin') {
        if (login == null ||
            password == null ||
            firstLength < 1 ||
            firstLength > 256 ||
            secondLength < 1 ||
            secondLength > 256 ||
            login.any((b) => b < 33 || b > 126) ||
            password.any((b) => b < 32 || b > 126))
          throw NativeFailure(
            'INVALID_ARGUMENT',
            'Use bounded ASCII sign-in input.',
          );
      } else {
        if (pending is! _NativeVerificationPending ||
            !_verificationChallenges.remove(pending))
          throw NativeFailure(
            'MIJIA_AUTH_CHALLENGE_REQUIRED',
            'Start sign-in again.',
          );
        owned = true;
        pending.clearImage();
        if (pending.remaining == Duration.zero)
          throw NativeFailure(
            'MIJIA_AUTH_CHALLENGE_REQUIRED',
            'Start sign-in again.',
          );
        if (pending.kind == AccountVerificationKind.captcha) {
          if (action != 'answer' ||
              method != null ||
              answer == null ||
              answer.isEmpty ||
              answer.length > 32 ||
              answer.any(
                (b) =>
                    !(b >= 48 && b <= 57 ||
                        b >= 65 && b <= 90 ||
                        b >= 97 && b <= 122),
              ))
            throw NativeFailure(
              'INVALID_ARGUMENT',
              'Enter the letters and numbers shown.',
            );
        } else {
          if (method == null || !pending.methods.contains(method))
            throw NativeFailure(
              'INVALID_ARGUMENT',
              'Choose an available verification method.',
            );
          if (action == 'send' &&
              (pending.codeRequests >= 3 || pending.cooldown > Duration.zero))
            throw NativeFailure(
              'MIJIA_AUTH_CHALLENGE_REQUIRED',
              'Wait before requesting another code.',
            );
          if (action == 'answer' &&
              (answer == null || answer.isEmpty || answer.length > 2048))
            throw NativeFailure(
              'INVALID_ARGUMENT',
              'Enter the verification code.',
            );
        }
      }
      operation = client.beginOperation();
      if (firstLength > 0) {
        first = calloc<Uint8>(firstLength);
        first.asTypedList(firstLength).setAll(0, firstBytes!);
        firstBytes.fillRange(0, firstLength, 0);
      }
      if (secondLength > 0) {
        second = calloc<Uint8>(secondLength);
        second.asTypedList(secondLength).setAll(0, password!);
        password.fillRange(0, secondLength, 0);
      }
      prompt = calloc<Uint8>(_verificationPromptCapacity);
      final metadata = <String, dynamic>{
        'schema_version': '1.0',
        'operation_id': operation,
        'timeout_ms': 180000,
        if (action == 'begin') ...{
          'label': label,
          'account_id': accountId,
          'provider': 'xiaomi.mijia',
        } else ...{
          'challenge_id': (pending as _NativeVerificationPending).id,
          if (action == 'send')
            'method': method!.name
          else
            'answer_kind': method?.name ?? 'captcha',
        },
      };
      final value =
          decodeResponse(
                await _verificationOnWorker(
                  client.path,
                  client.handle,
                  action,
                  metadata,
                  first?.address ?? 0,
                  firstLength,
                  second?.address ?? 0,
                  secondLength,
                  prompt.address,
                ),
              )
              as Json;
      return _verificationProgress(
        value,
        prompt,
        generation,
        method,
        action == 'send',
      );
    } finally {
      login?.fillRange(0, login.length, 0);
      password?.fillRange(0, password.length, 0);
      answer?.fillRange(0, answer.length, 0);
      if (first != null) {
        first.asTypedList(firstLength).fillRange(0, firstLength, 0);
        calloc.free(first);
      }
      if (second != null) {
        second.asTypedList(secondLength).fillRange(0, secondLength, 0);
        calloc.free(second);
      }
      if (prompt != null) {
        prompt
            .asTypedList(_verificationPromptCapacity)
            .fillRange(0, _verificationPromptCapacity, 0);
        calloc.free(prompt);
      }
      if (owned && pending is _NativeVerificationPending)
        _discardVerificationId(pending.id);
      if (operation != null) client.endOperation(operation);
    }
  }

  Future<void> discardAccountVerification(
    AccountVerificationPending pending,
  ) async {
    pending.clearImage();
    if (pending is _NativeVerificationPending &&
        _verificationChallenges.remove(pending))
      _discardVerificationId(pending.id);
  }
}

// Worker closures carry addresses and nonsecret metadata only, never credential arrays.
Future<Json> _verificationOnWorker(
  String path,
  int handle,
  String action,
  Json metadata,
  int first,
  int firstLength,
  int second,
  int secondLength,
  int prompt,
) => Isolate.run(
  () => _VerificationBindings(NativeBindings(path)).invoke(
    handle,
    action,
    metadata,
    first,
    firstLength,
    second,
    secondLength,
    prompt,
  ),
);
