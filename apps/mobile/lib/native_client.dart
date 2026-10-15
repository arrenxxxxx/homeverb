import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';
import 'dart:math';
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

class NativeClient implements CoreClient {
  NativeClient._(this.path, this.handle) : bindings = NativeBindings(path);
  final String path;
  final int handle;
  final NativeBindings bindings;
  final Set<String> active = {};
  Set<String> methods = {};
  bool _foreground = true, _closed = false;
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
  }) async {
    final path = libraryPath ?? defaultLibrary();
    final response = await Isolate.run(
      () => NativeBindings(path).request({
        'schema_version': '1.0',
        'data_dir': dataDir,
        'allow_mock_writes': allowMockWrites,
        'scenario': scenario,
      }),
    );
    final data = decodeResponse(response) as Json;
    final client = NativeClient._(path, data['handle'] as int);
    try {
      final info = await client.call('get_api_info') as Json;
      if (!('${info['api_version']}').startsWith('1.')) {
        throw NativeFailure(
          'API_UPGRADE_REQUIRED',
          'Native API major 1 is required.',
        );
      }
      client.methods = (info['methods'] as Map).keys.cast<String>().toSet();
      return client;
    } catch (_) {
      await client.close();
      rethrow;
    }
  }

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (_closed) throw NativeFailure('HOST_CLOSED', 'Reopen the application.');
    if (!_foreground)
      throw NativeFailure('HOST_BACKGROUND', 'Return to the foreground.');
    if (methods.isNotEmpty && !methods.contains(method)) {
      throw NativeFailure('METHOD_UNAVAILABLE', method);
    }
    if (active.length >= 8)
      throw NativeFailure('BUSY', 'Wait for active operations.');
    final id = uniqueId(), libraryPath = path, nativeHandle = handle;
    active.add(id);
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
      active.remove(id);
    }
  }

  @override
  void foreground(bool value) {
    _foreground = value;
    if (!_closed) bindings.foreground(handle, value);
  }

  @override
  void cancelAll() {
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
