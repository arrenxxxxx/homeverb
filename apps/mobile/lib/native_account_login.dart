part of 'native_client.dart';

class _NativeLoginCaptcha extends AccountLoginCaptcha {
  _NativeLoginCaptcha(this.id, this.image, this.lifetime);
  final int id;
  @override
  final Uint8List image;
  final Duration lifetime;
  final clock = Stopwatch()..start();
  @override
  Duration get remaining {
    final left = lifetime - clock.elapsed;
    return left.isNegative ? Duration.zero : left;
  }

  @override
  void clearImage() => image.fillRange(0, image.length, 0);
}

mixin _InteractiveNativeLogin {
  NativeClient get _client => this as NativeClient;
  final Set<_NativeLoginCaptcha> _loginChallenges = {};
  int _loginGeneration = 0;
  bool get supportsInteractiveLogin =>
      _client.apiAtLeast(28) &&
      _client.accountProviders.contains('xiaomi.mijia');

  void _clearLoginChallenges() {
    _loginGeneration++;
    for (final challenge in _loginChallenges) {
      challenge.clearImage();
    }
    _loginChallenges.clear();
  }

  AccountLoginProgress _loginProgress(Json value, int generation) {
    final client = _client;
    if (client._closed ||
        !client._foreground ||
        generation != _loginGeneration) {
      if (value['kind'] == 'captcha' &&
          value['challenge_id'] is int &&
          !client._closed) {
        client.bindings.discardAccountLogin(
          client.handle,
          value['challenge_id'] as int,
        );
      }
      throw NativeFailure('CANCELLED', 'Sign-in was cancelled.');
    }
    if (value['kind'] == 'connected' && value['report'] is Json) {
      return AccountLoginConnected(value['report'] as Json);
    }
    if (value['kind'] == 'captcha') {
      final id = value['challenge_id'],
          encoded = value['image_base64'],
          lifetime = value['expires_in_ms'];
      Uint8List? image;
      try {
        if (id is! int ||
            id <= 0 ||
            encoded is! String ||
            encoded.length > 174764 ||
            lifetime is! int ||
            lifetime <= 0 ||
            lifetime > 180000 ||
            !['image/png', 'image/jpeg'].contains(value['media_type'])) {
          throw const FormatException();
        }
        image = base64Decode(encoded);
        if (image.isEmpty || image.length > 131072)
          throw const FormatException();
        final challenge = _NativeLoginCaptcha(
          id,
          image,
          Duration(milliseconds: lifetime),
        );
        _loginChallenges.add(challenge);
        return challenge;
      } catch (_) {
        image?.fillRange(0, image.length, 0);
        if (id is int && id > 0)
          client.bindings.discardAccountLogin(client.handle, id);
      }
    }
    throw NativeFailure('UNSUPPORTED_OUTCOME', 'Unsupported sign-in response.');
  }

  Future<AccountLoginProgress> beginAccountLogin({
    required Uint8List login,
    required Uint8List password,
    required String label,
    String? accountId,
  }) async {
    final generation = _loginGeneration;
    final value = await _client._connectAccount(
      login: login,
      password: password,
      label: label,
      accountId: accountId,
      provider: 'xiaomi.mijia',
      interactive: true,
    );
    return _loginProgress(value, generation);
  }

  Future<AccountLoginProgress> continueAccountLogin(
    AccountLoginCaptcha challenge,
    Uint8List answer,
  ) async {
    final client = _client,
        length = answer.length,
        generation = _loginGeneration;
    Pointer<Uint8>? memory;
    String? operation;
    var owned = false;
    try {
      if (!supportsInteractiveLogin ||
          challenge is! _NativeLoginCaptcha ||
          !_loginChallenges.remove(challenge)) {
        throw NativeFailure(
          'MIJIA_AUTH_CHALLENGE_REQUIRED',
          'Start sign-in again.',
        );
      }
      owned = true;
      challenge.clearImage();
      if (challenge.remaining == Duration.zero ||
          length < 1 ||
          length > 32 ||
          answer.any(
            (b) =>
                !(b >= 48 && b <= 57 ||
                    b >= 65 && b <= 90 ||
                    b >= 97 && b <= 122),
          )) {
        client.bindings.discardAccountLogin(client.handle, challenge.id);
        throw NativeFailure(
          'INVALID_ARGUMENT',
          'Enter the letters and numbers shown.',
        );
      }
      operation = client.beginOperation();
      memory = calloc<Uint8>(length);
      memory.asTypedList(length).setAll(0, answer);
      answer.fillRange(0, length, 0);
      final metadata = <String, dynamic>{
        'schema_version': '1.0',
        'operation_id': operation,
        'timeout_ms': 180000,
        'challenge_id': challenge.id,
      };
      final value =
          decodeResponse(
                await _answerLoginOnWorker(
                  client.path,
                  client.handle,
                  metadata,
                  memory.address,
                  length,
                ),
              )
              as Json;
      return _loginProgress(value, generation);
    } finally {
      answer.fillRange(0, length, 0);
      if (memory != null) {
        memory.asTypedList(length).fillRange(0, length, 0);
        calloc.free(memory);
      }
      // Also discard when worker admission or allocation failed before native consumption.
      if (owned && challenge is _NativeLoginCaptcha && !client._closed) {
        client.bindings.discardAccountLogin(client.handle, challenge.id);
      }
      if (operation != null) client.endOperation(operation);
    }
  }

  Future<void> discardAccountLogin(AccountLoginCaptcha challenge) async {
    if (challenge is! _NativeLoginCaptcha ||
        !_loginChallenges.remove(challenge))
      return;
    challenge.clearImage();
    if (!_client._closed)
      _client.bindings.discardAccountLogin(_client.handle, challenge.id);
  }
}

Future<Json> _answerLoginOnWorker(
  String path,
  int handle,
  Json metadata,
  int address,
  int length,
) => Isolate.run(
  () => NativeBindings(
    path,
  ).answerLoginRequest(handle, metadata, address, length),
);
