import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';
import 'dart:math';
import 'dart:typed_data';
import 'package:ffi/ffi.dart';

typedef Json = Map<String, dynamic>;

String uniqueId() => List.generate(
  24,
  (_) => Random.secure().nextInt(256).toRadixString(16).padLeft(2, '0'),
).join();

class NativeFailure implements Exception {
  NativeFailure(this.code, this.message);
  final String code, message;
  @override
  String toString() => '$code: $message';
}

/// Retain execution payloads even when they contain an embedded error.
dynamic decodeResponse(Json value) {
  if (value['status'] != null) {
    if (![
      'accepted',
      'verified',
      'failed',
      'unknown',
      'partial',
    ].contains(value['status'])) {
      throw NativeFailure('UNSUPPORTED_OUTCOME', '${value['status']}');
    }
    return value;
  }
  if (value['error'] case final Map error) {
    throw NativeFailure('${error['code']}', '${error['message']}');
  }
  return value['data'];
}

abstract interface class CoreClient {
  Future<dynamic> call(String method, [Json fields = const {}]);
  void foreground(bool value);
  void cancelAll();
  Future<void> close();
}

abstract interface class AccountClient {
  String get adapter;
  bool get supportsAccountConnection;
  bool get supportsPendingReauthorization;
  // Takes ownership of the byte arrays and clears them on every return path.
  Future<Json> connectAccount({
    required Uint8List login,
    required Uint8List password,
    required String label,
    String? accountId,
  });
}

abstract interface class ApprovalClient {
  bool get supportsTrustedApproval;
  Future<ApprovalPreview> prepareApproval(Json command);
  Future<Json> executeApproved(ApprovalPreview preview);
  Future<void> discardApproval(ApprovalPreview preview);
}

/// Native scope is immutable and is never reconstructed from editable form state.
class ApprovalPreview {
  ApprovalPreview._(Json data, this._clock)
    : _id = data['approval_id'] as int,
      command = _freeze(data['command']) as Json,
      descriptor = _freeze(data['descriptor']) as Json,
      action = _freeze(data['action']) as Json,
      connection = _freeze(data['connection']) as Json,
      expiresAt = DateTime.parse(data['expires_at'] as String);
  final int _id;
  final Stopwatch _clock;
  final Json command, descriptor, action, connection;
  final DateTime expiresAt;
  Duration get remaining {
    final duration = const Duration(seconds: 60) - _clock.elapsed;
    return duration.isNegative ? Duration.zero : duration;
  }

  static dynamic _freeze(dynamic value) => value is Map
      ? Map<String, dynamic>.unmodifiable(
          value.map((key, item) => MapEntry(key as String, _freeze(item))),
        )
      : value is List
      ? List<dynamic>.unmodifiable(value.map(_freeze))
      : value;
}

class NativeBindings {
  NativeBindings(String path)
    : library = path == 'process'
          ? DynamicLibrary.process()
          : DynamicLibrary.open(path);
  final DynamicLibrary library;
  late final open = library
      .lookupFunction<
        Pointer<Utf8> Function(Pointer<Utf8>),
        Pointer<Utf8> Function(Pointer<Utf8>)
      >('homeverb_open');
  late final invoke = library
      .lookupFunction<
        Pointer<Utf8> Function(Uint64, Pointer<Utf8>),
        Pointer<Utf8> Function(int, Pointer<Utf8>)
      >('homeverb_call');
  late final connect = library
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
      >('homeverb_connect_account');

  Json connectRequest(
    int handle,
    Json metadata,
    int loginAddress,
    int loginLength,
    int passwordAddress,
    int passwordLength,
  ) {
    final input = jsonEncode(metadata).toNativeUtf8();
    Pointer<Utf8>? output;
    try {
      output = connect(
        handle,
        input,
        Pointer<Uint8>.fromAddress(loginAddress),
        loginLength,
        Pointer<Uint8>.fromAddress(passwordAddress),
        passwordLength,
      );
      if (output == nullptr) throw NativeFailure('NATIVE_ERROR', 'No response');
      return jsonDecode(output.toDartString()) as Json;
    } finally {
      calloc.free(input);
      if (output != null && output != nullptr) free(output);
    }
  }

  late final prepareApproval = library
      .lookupFunction<
        Pointer<Utf8> Function(Uint64, Pointer<Utf8>),
        Pointer<Utf8> Function(int, Pointer<Utf8>)
      >('homeverb_prepare_approval');
  late final executeApproved = library
      .lookupFunction<
        Pointer<Utf8> Function(Uint64, Pointer<Utf8>),
        Pointer<Utf8> Function(int, Pointer<Utf8>)
      >('homeverb_execute_approved');
  late final discardApproval = library
      .lookupFunction<Bool Function(Uint64, Uint64), bool Function(int, int)>(
        'homeverb_discard_approval',
      );

  Json approvalRequest(int handle, Json data, {required bool execute}) {
    final input = jsonEncode(data).toNativeUtf8();
    Pointer<Utf8>? output;
    try {
      output = execute
          ? executeApproved(handle, input)
          : prepareApproval(handle, input);
      if (output == nullptr) throw NativeFailure('NATIVE_ERROR', 'No response');
      return jsonDecode(output.toDartString()) as Json;
    } finally {
      calloc.free(input);
      if (output != null && output != nullptr) free(output);
    }
  }

  late final free = library
      .lookupFunction<
        Void Function(Pointer<Utf8>),
        void Function(Pointer<Utf8>)
      >('homeverb_string_free');
  late final cancel = library
      .lookupFunction<
        Bool Function(Uint64, Pointer<Utf8>),
        bool Function(int, Pointer<Utf8>)
      >('homeverb_cancel');
  late final foreground = library
      .lookupFunction<Void Function(Uint64, Bool), void Function(int, bool)>(
        'homeverb_set_foreground',
      );
  late final close = library
      .lookupFunction<Void Function(Uint64), void Function(int)>(
        'homeverb_close',
      );

  Json request(Json data, {int? handle}) {
    final input = jsonEncode(data).toNativeUtf8();
    Pointer<Utf8>? output;
    try {
      output = handle == null ? open(input) : invoke(handle, input);
      if (output == nullptr) throw NativeFailure('NATIVE_ERROR', 'No response');
      return jsonDecode(output.toDartString()) as Json;
    } finally {
      calloc.free(input);
      if (output != null && output != nullptr) free(output);
    }
  }
}

class NativeClient implements CoreClient, AccountClient, ApprovalClient {
  NativeClient._(this.path, this.handle, this.adapter)
    : bindings = NativeBindings(path);
  final String path;
  final int handle;
  @override
  final String adapter;
  String apiVersion = '';
  bool apiAtLeast(int minor) =>
      apiVersion.startsWith('1.') &&
      (int.tryParse(apiVersion.split('.')[1]) ?? -1) >= minor;
  @override
  bool get supportsAccountConnection =>
      apiAtLeast(2) &&
      methods.contains('list_accounts') &&
      methods.contains('refresh_account');
  @override
  bool get supportsPendingReauthorization =>
      supportsAccountConnection && apiAtLeast(8);
  @override
  bool get supportsTrustedApproval => apiAtLeast(13);
  final Set<ApprovalPreview> _approvals = {};
  int _approvalGeneration = 0;
  final NativeBindings bindings;
  final Set<String> active = {};
  Set<String> methods = {};
  bool _foreground = true, _closed = false, _suspended = false;
  static String defaultLibrary() => Platform.isIOS
      ? 'process'
      : Platform.isMacOS
      ? 'libhomeverb_ffi.dylib'
      : Platform.isWindows
      ? 'homeverb_ffi.dll'
      : 'libhomeverb_ffi.so';

  static Future<NativeClient> open({
    required String dataDir,
    String? libraryPath,
    bool allowMockWrites = false,
    String scenario = 'normal',
    String adapter = 'homeverb.mock',
  }) async {
    final path = libraryPath ?? defaultLibrary();
    final response = await Isolate.run(
      () => NativeBindings(path).request({
        'schema_version': '1.0',
        'data_dir': dataDir,
        'allow_mock_writes': allowMockWrites,
        'scenario': scenario,
        if (adapter != 'homeverb.mock') 'adapter': adapter,
      }),
    );
    final data = decodeResponse(response) as Json;
    final client = NativeClient._(
      path,
      data['handle'] as int,
      data['adapter'] as String? ?? 'homeverb.mock',
    );
    try {
      final info = await client.call('get_api_info') as Json;
      if (!('${info['api_version']}').startsWith('1.')) {
        throw NativeFailure(
          'API_UPGRADE_REQUIRED',
          'Native API major 1 is required.',
        );
      }
      client.apiVersion = info['api_version'] as String;
      client.methods = (info['methods'] as Map).keys.cast<String>().toSet();
      return client;
    } catch (error) {
      await client.close();
      if (error is NativeFailure && error.code == 'INVALID_ARGUMENT') {
        throw NativeFailure(
          'API_UPGRADE_REQUIRED',
          'A native library with API negotiation is required.',
        );
      }
      rethrow;
    }
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    final id = beginOperation(), libraryPath = path, nativeHandle = handle;
    if (methods.isNotEmpty && !methods.contains(method)) {
      endOperation(id);
      throw NativeFailure('METHOD_UNAVAILABLE', method);
    }
    final request = {
      'schema_version': '1.0',
      'operation_id': id,
      'timeout_ms': 10000,
      'request': {'method': method, ...fields},
    };
    try {
      return decodeResponse(
        await Isolate.run(
          () => NativeBindings(
            libraryPath,
          ).request(request, handle: nativeHandle),
        ),
      );
    } finally {
      endOperation(id);
    }
  }

  @override
  Future<ApprovalPreview> prepareApproval(Json command) async {
    if (!supportsTrustedApproval) {
      throw NativeFailure(
        'API_UPGRADE_REQUIRED',
        'Trusted approval requires API 1.13 or later.',
      );
    }
    final operation = beginOperation(), generation = _approvalGeneration;
    final clock = Stopwatch()..start();
    final libraryPath = path, nativeHandle = handle;
    final request = {
      'schema_version': '1.0',
      'operation_id': operation,
      'command': command,
    };
    try {
      final data =
          decodeResponse(
                await Isolate.run(
                  () => NativeBindings(
                    libraryPath,
                  ).approvalRequest(nativeHandle, request, execute: false),
                ),
              )
              as Json;
      final preview = ApprovalPreview._(data, clock);
      if (generation != _approvalGeneration || _closed || !_foreground) {
        final approvalId = preview._id;
        await Isolate.run(
          () => NativeBindings(
            libraryPath,
          ).discardApproval(nativeHandle, approvalId),
        );
        throw NativeFailure('CANCELLED', 'The confirmation was cancelled.');
      }
      _approvals.add(preview);
      return preview;
    } finally {
      endOperation(operation);
    }
  }

  @override
  Future<Json> executeApproved(ApprovalPreview preview) async {
    if (!supportsTrustedApproval || !_approvals.contains(preview)) {
      throw NativeFailure('APPROVAL_INVALID', 'Review a new confirmation.');
    }
    final operation = beginOperation(),
        libraryPath = path,
        nativeHandle = handle;
    _approvals.remove(preview);
    final request = {
      'schema_version': '1.0',
      'operation_id': operation,
      'approval_id': preview._id,
    };
    try {
      return decodeResponse(
            await Isolate.run(
              () => NativeBindings(
                libraryPath,
              ).approvalRequest(nativeHandle, request, execute: true),
            ),
          )
          as Json;
    } finally {
      endOperation(operation);
    }
  }

  @override
  Future<void> discardApproval(ApprovalPreview preview) async {
    if (!_approvals.remove(preview) || _closed) return;
    final libraryPath = path, nativeHandle = handle, approvalId = preview._id;
    await Isolate.run(
      () =>
          NativeBindings(libraryPath).discardApproval(nativeHandle, approvalId),
    );
  }

  String beginOperation() {
    if (_closed) throw NativeFailure('HOST_CLOSED', 'Reopen the application.');
    if (!_foreground || _suspended) {
      throw NativeFailure('HOST_BACKGROUND', 'Return to the foreground.');
    }
    if (active.length >= 8)
      throw NativeFailure('BUSY', 'Wait for active operations.');
    final id = uniqueId();
    active.add(id);
    return id;
  }

  void endOperation(String id) {
    active.remove(id);
    if (active.isEmpty && _foreground && !_closed && _suspended) {
      _suspended = false;
      bindings.foreground(handle, true);
    }
  }

  @override
  Future<Json> connectAccount({
    required Uint8List login,
    required Uint8List password,
    required String label,
    String? accountId,
  }) async {
    Pointer<Uint8>? loginMemory, passwordMemory;
    String? id;
    final loginLength = login.length, passwordLength = password.length;
    try {
      if (!supportsAccountConnection) {
        throw NativeFailure(
          'API_UPGRADE_REQUIRED',
          'Protected account connection requires native API 1.2 or later.',
        );
      }
      if (loginLength < 1 ||
          loginLength > 256 ||
          passwordLength < 1 ||
          passwordLength > 256 ||
          login.any((b) => b < 33 || b > 126) ||
          password.any((b) => b < 32 || b > 126)) {
        throw NativeFailure(
          'INVALID_ARGUMENT',
          'Use 1–256 ASCII bytes for each credential.',
        );
      }
      id = beginOperation();
      loginMemory = calloc<Uint8>(loginLength);
      passwordMemory = calloc<Uint8>(passwordLength);
      loginMemory.asTypedList(loginLength).setAll(0, login);
      passwordMemory.asTypedList(passwordLength).setAll(0, password);
      login.fillRange(0, loginLength, 0);
      password.fillRange(0, passwordLength, 0);
      final loginAddress = loginMemory.address,
          passwordAddress = passwordMemory.address;
      final libraryPath = path, nativeHandle = handle;
      final metadata = <String, dynamic>{
        'schema_version': '1.0',
        'operation_id': id,
        'timeout_ms': 180000,
        'label': label,
        'account_id': accountId,
      };
      return decodeResponse(
            await _connectOnWorker(
              libraryPath,
              nativeHandle,
              metadata,
              loginAddress,
              loginLength,
              passwordAddress,
              passwordLength,
            ),
          )
          as Json;
    } finally {
      login.fillRange(0, loginLength, 0);
      password.fillRange(0, passwordLength, 0);
      if (loginMemory != null) {
        loginMemory.asTypedList(loginLength).fillRange(0, loginLength, 0);
        calloc.free(loginMemory);
      }
      if (passwordMemory != null) {
        passwordMemory
            .asTypedList(passwordLength)
            .fillRange(0, passwordLength, 0);
        calloc.free(passwordMemory);
      }
      if (id != null) endOperation(id);
    }
  }

  @override
  void foreground(bool value) {
    _foreground = value;
    if (_closed) return;
    if (!value) {
      _approvalGeneration++;
      _approvals.clear();
      _suspended = true;
      bindings.foreground(handle, false);
    } else if (active.isEmpty) {
      _suspended = false;
      bindings.foreground(handle, true);
    }
  }

  @override
  void cancelAll() {
    _approvalGeneration++;
    final hadApprovals = _approvals.isNotEmpty;
    _approvals.clear();
    // Hold the native gate closed until even not-yet-started isolates drain.
    if ((active.isNotEmpty || hadApprovals) && !_closed) {
      _suspended = true;
      bindings.foreground(handle, false);
    }
    if (active.isEmpty && _foreground && !_closed && _suspended) {
      _suspended = false;
      bindings.foreground(handle, true);
    }
    for (final id in active.toList()) {
      final input = id.toNativeUtf8();
      try {
        bindings.cancel(handle, input);
      } finally {
        calloc.free(input);
      }
    }
  }

  @override
  Future<void> close() async {
    if (_closed) return;
    foreground(false);
    _closed = true;
    final libraryPath = path, nativeHandle = handle;
    await Isolate.run(() => NativeBindings(libraryPath).close(nativeHandle));
  }
}

// Keep the worker closure in a scope without managed credential arrays.
Future<Json> _connectOnWorker(
  String path,
  int handle,
  Json metadata,
  int loginAddress,
  int loginLength,
  int passwordAddress,
  int passwordLength,
) => Isolate.run(
  () => NativeBindings(path).connectRequest(
    handle,
    metadata,
    loginAddress,
    loginLength,
    passwordAddress,
    passwordLength,
  ),
);
