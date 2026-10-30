part of 'native_client.dart';

const _modernScanCapacity = 8192;

class _ModernBindings {
  _ModernBindings(this.base);
  final NativeBindings base;
  late final begin = base.library
      .lookupFunction<
        Pointer<Utf8> Function(Uint64, Pointer<Utf8>, Pointer<Uint8>, Size),
        Pointer<Utf8> Function(int, Pointer<Utf8>, Pointer<Uint8>, int)
      >('homeverb_begin_modern_account_login');
  late final answer = base.library
      .lookupFunction<
        Pointer<Utf8> Function(Uint64, Pointer<Utf8>),
        Pointer<Utf8> Function(int, Pointer<Utf8>)
      >('homeverb_continue_modern_account_login');
  late final session = base.library
      .lookupFunction<
        Pointer<Utf8> Function(Uint64, Pointer<Utf8>),
        Pointer<Utf8> Function(int, Pointer<Utf8>)
      >('homeverb_modern_account_session');
  late final discard = base.library
      .lookupFunction<Bool Function(Uint64, Uint64), bool Function(int, int)>(
        'homeverb_discard_modern_account_login',
      );
  Json invoke(int handle, String action, Json metadata, int address) {
    final input = jsonEncode(metadata).toNativeUtf8();
    Pointer<Utf8>? output;
    try {
      output = switch (action) {
        'begin' => begin(
          handle,
          input,
          Pointer<Uint8>.fromAddress(address),
          _modernScanCapacity,
        ),
        'continue' => answer(handle, input),
        'session' => session(handle, input),
        _ => throw NativeFailure(
          'INVALID_ARGUMENT',
          'Invalid account operation.',
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

class _NativeModernQr extends ModernAccountQr {
  _NativeModernQr(this.id, this.scanData, this.lifetime, this.accountId);
  final int id;
  final String? accountId;
  @override
  final Uint8List scanData;
  final Duration lifetime;
  final clock = Stopwatch()..start();
  @override
  Duration get remaining {
    final left = lifetime - clock.elapsed;
    return left.isNegative ? Duration.zero : left;
  }

  @override
  void clearScan() => scanData.fillRange(0, scanData.length, 0);
}

Json _modernObject(dynamic value, Set<String> fields) {
  if (value is! Json ||
      value.length != fields.length ||
      value.keys.any((key) => !fields.contains(key)))
    throw const FormatException();
  return value;
}

Json _modernError(dynamic value) {
  final error = _modernObject(value, {'code', 'message', 'retryable'});
  if (error['code'] is! String ||
      (error['code'] as String).isEmpty ||
      error['message'] is! String ||
      error['retryable'] is! bool)
    throw const FormatException();
  return Map.unmodifiable(error);
}

Json _modernAccount(dynamic value, String? expectedId) {
  final account = _modernObject(value, {
    'id',
    'provider',
    'region',
    'label',
    'credential_state',
    'authorization',
    'last_complete_inventory_at',
  });
  final id = account['id'],
      label = account['label'],
      time = account['last_complete_inventory_at'];
  if (id is! String ||
      !RegExp(
        r'^[0-9a-fA-F]{8}(-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}$',
      ).hasMatch(id) ||
      (expectedId != null && id != expectedId) ||
      account['provider'] != 'xiaomi.mijia' ||
      account['region'] != 'CN' ||
      label is! String ||
      label.isEmpty ||
      label.runes.length > 80 ||
      ![
        'stored',
        'pending',
        'removing',
      ].contains(account['credential_state']) ||
      account['authorization'] != 'not_checked' ||
      (time != null && (time is! String || DateTime.tryParse(time) == null)))
    throw const FormatException();
  return Map.unmodifiable(account);
}

ModernAccountResult _modernResult(
  dynamic value,
  ModernAccountResultKind kind,
  String? expectedId,
) {
  final body = _modernObject(value, {
    'kind',
    if (kind == ModernAccountResultKind.renewed) 'account' else 'report',
  });
  final wireKind = kind == ModernAccountResultKind.connectedDevices
      ? 'connected_devices'
      : kind.name;
  if (body['kind'] != wireKind) throw const FormatException();
  if (kind == ModernAccountResultKind.renewed)
    return ModernAccountResult(
      kind: kind,
      account: _modernAccount(body['account'], expectedId),
    );
  if (kind == ModernAccountResultKind.connectedDevices ||
      kind == ModernAccountResultKind.devices) {
    final report = _modernObject(body['report'], {
      'account',
      'authentication',
      'retrieval',
    });
    if (report['authentication'] !=
        (kind == ModernAccountResultKind.connectedDevices
            ? 'succeeded'
            : 'not_attempted')) {
      throw const FormatException();
    }
    final retrieval = _modernObject(report['retrieval'], {
      'status',
      'device_ids',
      'error',
    });
    final ids = retrieval['device_ids'];
    final uuid = RegExp(
      r'^[0-9a-fA-F]{8}(-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}$',
    );
    if (!['partial', 'failed'].contains(retrieval['status']) ||
        ids is! List ||
        ids.length > 4096 ||
        ids.any((id) => id is! String || !uuid.hasMatch(id)) ||
        ids.map((id) => (id as String).toLowerCase()).toSet().length !=
            ids.length ||
        (retrieval['status'] == 'failed' && ids.isNotEmpty)) {
      throw const FormatException();
    }
    final account = _modernAccount(report['account'], expectedId);
    return ModernAccountResult(
      kind: kind,
      account: account,
      deviceReport: Map.unmodifiable({
        'account': account,
        'authentication': report['authentication'],
        'retrieval': Map<String, dynamic>.unmodifiable({
          'status': retrieval['status'],
          'device_ids': List<String>.unmodifiable(ids.cast<String>()),
          'error': _modernError(retrieval['error']),
        }),
      }),
    );
  }
  final report = _modernObject(body['report'], {
    'account',
    'authentication',
    'directory',
  });
  if (report['authentication'] !=
      (kind == ModernAccountResultKind.connected
          ? 'succeeded'
          : 'not_attempted'))
    throw const FormatException();
  final directory = _modernObject(report['directory'], {
    'status',
    'logical_entry_count',
    'complete',
    'issues',
    'error',
  });
  final count = directory['logical_entry_count'], issues = directory['issues'];
  if (!['partial', 'failed'].contains(directory['status']) ||
      count is! int ||
      count < 0 ||
      count > 4096 ||
      directory['complete'] != false ||
      issues is! List ||
      issues.length > 64)
    throw const FormatException();
  final failed = directory['status'] == 'failed';
  if (failed
      ? count != 0 || issues.isNotEmpty || directory['error'] == null
      : directory['error'] != null)
    throw const FormatException();
  return ModernAccountResult(
    kind: kind,
    account: _modernAccount(report['account'], expectedId),
    directory: ModernDirectoryResult(
      count: count,
      failed: failed,
      issues: List.unmodifiable(issues.map(_modernError)),
      error: failed ? _modernError(directory['error']) : null,
    ),
  );
}

mixin _NativeModernAccount {
  NativeClient get _modernClient => this as NativeClient;
  final Set<_NativeModernQr> _modernChallenges = {};
  int _modernGeneration = 0;
  bool get supportsModernAccountLogin =>
      _modernClient.apiAtLeast(36) &&
      _modernClient.accountProviders.contains('xiaomi.mijia');
  void _clearModernChallenges() {
    _modernGeneration++;
    for (final pending in _modernChallenges) {
      pending.clearScan();
    }
    _modernChallenges.clear();
  }

  void _discardModernId(int id) {
    final client = _modernClient;
    if (!client._closed)
      _ModernBindings(client.bindings).discard(client.handle, id);
  }

  void _requireModern(int generation) {
    if (!supportsModernAccountLogin)
      throw NativeFailure(
        'API_UPGRADE_REQUIRED',
        'QR sign-in requires native API 1.36.',
      );
    final client = _modernClient;
    if (client._closed ||
        !client._foreground ||
        generation != _modernGeneration)
      throw NativeFailure('CANCELLED', 'Sign-in was cancelled.');
  }

  Future<ModernAccountQr> beginModernAccountLogin({
    required String label,
    String? accountId,
  }) async {
    final client = _modernClient, generation = _modernGeneration;
    Pointer<Uint8>? buffer;
    String? operation;
    int? challengeId;
    Uint8List? scan;
    var retained = false;
    try {
      _requireModern(generation);
      operation = client.beginOperation();
      buffer = calloc<Uint8>(_modernScanCapacity);
      final raw = decodeResponse(
        await _modernOnWorker(client.path, client.handle, 'begin', {
          'schema_version': '1.0',
          'operation_id': operation,
          'timeout_ms': 180000,
          'label': label,
          'account_id': accountId,
          'provider': 'xiaomi.mijia',
        }, buffer.address),
      );
      if (raw is Json && raw['challenge_id'] is int && raw['challenge_id'] > 0)
        challengeId = raw['challenge_id'] as int;
      _requireModern(generation);
      final result = _modernObject(raw, {
        'kind',
        'challenge_id',
        'scan_length',
        'expires_in_ms',
      });
      final length = result['scan_length'], lifetime = result['expires_in_ms'];
      if (result['kind'] != 'qr' ||
          challengeId == null ||
          length is! int ||
          length < 1 ||
          length > _modernScanCapacity ||
          lifetime is! int ||
          lifetime < 1 ||
          lifetime > 180000)
        throw const FormatException();
      scan = Uint8List.fromList(buffer.asTypedList(length));
      // Reviewed scan URLs are printable ASCII; do not construct a managed URL string.
      if (scan.any((b) => b < 33 || b > 126)) throw const FormatException();
      final pending = _NativeModernQr(
        challengeId,
        scan,
        Duration(milliseconds: lifetime),
        accountId,
      );
      _modernChallenges.add(pending);
      retained = true;
      return pending;
    } on FormatException {
      throw NativeFailure(
        'UNSUPPORTED_OUTCOME',
        'Unsupported sign-in response.',
      );
    } finally {
      if (buffer != null) {
        buffer
            .asTypedList(_modernScanCapacity)
            .fillRange(0, _modernScanCapacity, 0);
        calloc.free(buffer);
      }
      if (!retained) {
        scan?.fillRange(0, scan.length, 0);
        if (challengeId != null) _discardModernId(challengeId);
      }
      if (operation != null) client.endOperation(operation);
    }
  }

  Future<ModernAccountResult> continueModernAccountLogin(
    ModernAccountQr pending,
  ) async {
    final client = _modernClient, generation = _modernGeneration;
    String? operation;
    var owned = false;
    try {
      _requireModern(generation);
      if (pending is! _NativeModernQr || !_modernChallenges.remove(pending))
        throw NativeFailure(
          'MIJIA_AUTH_CHALLENGE_REQUIRED',
          'Start sign-in again.',
        );
      owned = true;
      pending.clearScan();
      final remaining = pending.remaining.inMilliseconds;
      if (remaining < 1)
        throw NativeFailure(
          'MIJIA_AUTH_CHALLENGE_REQUIRED',
          'Start sign-in again.',
        );
      operation = client.beginOperation();
      final result = decodeResponse(
        await _modernOnWorker(client.path, client.handle, 'continue', {
          'schema_version': '1.0',
          'operation_id': operation,
          'timeout_ms': remaining.clamp(1, 120000),
          'challenge_id': pending.id,
          if (client.apiAtLeast(37)) 'publish_devices': true,
        }, 0),
      );
      _requireModern(generation);
      return _modernResult(
        result,
        client.apiAtLeast(37)
            ? ModernAccountResultKind.connectedDevices
            : ModernAccountResultKind.connected,
        pending.accountId,
      );
    } on FormatException {
      throw NativeFailure(
        'UNSUPPORTED_OUTCOME',
        'Unsupported sign-in response.',
      );
    } finally {
      if (owned && pending is _NativeModernQr) _discardModernId(pending.id);
      if (operation != null) client.endOperation(operation);
    }
  }

  Future<void> discardModernAccountLogin(ModernAccountQr pending) async {
    if (pending is _NativeModernQr && _modernChallenges.remove(pending)) {
      pending.clearScan();
      _discardModernId(pending.id);
    }
  }

  Future<ModernAccountResult> modernAccountSession(
    String accountId,
    ModernAccountOperation action,
  ) async {
    final client = _modernClient, generation = _modernGeneration;
    String? operation;
    try {
      _requireModern(generation);
      operation = client.beginOperation();
      final result = decodeResponse(
        await _modernOnWorker(client.path, client.handle, 'session', {
          'schema_version': '1.0',
          'operation_id': operation,
          'timeout_ms': 30000,
          'account_id': accountId,
          'operation':
              client.apiAtLeast(37) && action != ModernAccountOperation.renew
              ? '${action.name}_devices'
              : action.name,
        }, 0),
      );
      _requireModern(generation);
      return _modernResult(
        result,
        action == ModernAccountOperation.renew
            ? ModernAccountResultKind.renewed
            : client.apiAtLeast(37)
            ? ModernAccountResultKind.devices
            : ModernAccountResultKind.directory,
        accountId,
      );
    } on FormatException {
      throw NativeFailure(
        'UNSUPPORTED_OUTCOME',
        'Unsupported sign-in response.',
      );
    } finally {
      if (operation != null) client.endOperation(operation);
    }
  }
}
// Worker closures carry nonsecret metadata and a caller-owned address, never scan bytes.
Future<Json> _modernOnWorker(
  String path,
  int handle,
  String action,
  Json metadata,
  int address,
) => Isolate.run(
  () => _ModernBindings(
    NativeBindings(path),
  ).invoke(handle, action, metadata, address),
);
