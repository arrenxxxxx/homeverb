import 'dart:math';
import 'dart:typed_data';

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

/// Native hosts require current session inspection before displaying controls.
abstract interface class NativeSessionClient implements CoreClient {
  Set<String> get methods;
  bool apiAtLeast(int minor);
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

abstract interface class IdentityRecoveryPreview {
  Json get account;
  int get deviceCount;
}

/// Separate trusted UI surface. Never expose confirmation through ordinary JSON.
abstract interface class IdentityRecoveryClient {
  bool get supportsIdentityRecovery;
  Future<IdentityRecoveryPreview> prepareIdentityRecovery({
    required Uint8List login,
    required Uint8List password,
    required String accountId,
    required String label,
  });
  Future<Json> confirmIdentityRecovery(IdentityRecoveryPreview preview);
  Future<void> discardIdentityRecovery(IdentityRecoveryPreview preview);
}

/// Native implementations retain the private handle and immutable preview data.
abstract interface class ApprovalPreview {
  Json get command;
  Json get descriptor;
  Json get action;
  Json get connection;
  DateTime get expiresAt;
  Duration get remaining;
}
