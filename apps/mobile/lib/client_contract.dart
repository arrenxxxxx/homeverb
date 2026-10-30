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
  Set<String> get accountProviders;
  bool get supportsPendingReauthorization;
  // Takes ownership of the byte arrays and clears them on every return path.
  Future<Json> connectAccount({
    required Uint8List login,
    required Uint8List password,
    required String label,
    String? accountId,
    String? provider,
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

/// Dedicated foreground UI flow; images and answers must never be persisted.
sealed class AccountLoginProgress {}

class AccountLoginConnected extends AccountLoginProgress {
  AccountLoginConnected(this.report);
  final Json report;
}

abstract class AccountLoginCaptcha extends AccountLoginProgress {
  Uint8List get image;
  Duration get remaining;
  void clearImage();
}

abstract interface class InteractiveAccountClient {
  bool get supportsInteractiveLogin;
  Future<AccountLoginProgress> beginAccountLogin({
    required Uint8List login,
    required Uint8List password,
    required String label,
    String? accountId,
  });
  Future<AccountLoginProgress> continueAccountLogin(
    AccountLoginCaptcha challenge,
    Uint8List answer,
  );
  Future<void> discardAccountLogin(AccountLoginCaptcha challenge);
}

enum AccountVerificationKind { captcha, verification }

enum AccountVerificationMethod { phone, email }

/// Trusted prompt state; never log, serialize or persist its owned image or answers.
abstract class AccountVerificationPending extends AccountLoginProgress {
  AccountVerificationKind get kind;
  Uint8List get image;
  List<AccountVerificationMethod> get methods;
  AccountVerificationMethod? get selectedMethod;
  Duration get remaining;
  Duration get cooldown;
  int get codeRequests;
  void clearImage();
}

class AccountVerificationReply {
  AccountVerificationReply.send(this.method) : answer = null;
  AccountVerificationReply.answer(this.answer, {this.method});
  final AccountVerificationMethod? method;
  final Uint8List? answer;
  bool get requestsCode => answer == null;
  void clear() => answer?.fillRange(0, answer!.length, 0);
}

abstract interface class AccountVerificationClient {
  bool get supportsAccountVerification;
  Future<AccountLoginProgress> beginAccountVerification({
    required Uint8List login,
    required Uint8List password,
    required String label,
    String? accountId,
  });
  Future<AccountLoginProgress> answerAccountVerification(
    AccountVerificationPending challenge,
    Uint8List answer, {
    AccountVerificationMethod? method,
  });
  Future<AccountLoginProgress> requestAccountVerificationCode(
    AccountVerificationPending challenge,
    AccountVerificationMethod method,
  );
  Future<void> discardAccountVerification(AccountVerificationPending challenge);
}

/// Owned foreground scan data. Never log, serialize, navigate to or persist it.
abstract class ModernAccountQr {
  Uint8List get scanData;
  Duration get remaining;
  void clearScan();
  @override
  String toString() => 'ModernAccountQr([REDACTED])';
}

enum ModernAccountOperation { refresh, recover, renew }

enum ModernAccountResultKind {
  connected,
  directory,
  renewed,
  connectedDevices,
  devices,
}

class ModernDirectoryResult {
  const ModernDirectoryResult({
    required this.count,
    required this.failed,
    required this.issues,
    this.error,
  });
  final int count;
  final bool failed;
  final List<Json> issues;
  final Json? error;
}

class ModernAccountResult {
  const ModernAccountResult({
    required this.kind,
    required this.account,
    this.directory,
    this.deviceReport,
  });
  final ModernAccountResultKind kind;
  final Json account;
  final ModernDirectoryResult? directory;
  final Json? deviceReport;
}

abstract interface class ModernAccountClient {
  bool get supportsModernAccountLogin;
  Future<ModernAccountQr> beginModernAccountLogin({
    required String label,
    String? accountId,
  });
  Future<ModernAccountResult> continueModernAccountLogin(
    ModernAccountQr challenge,
  );
  Future<void> discardModernAccountLogin(ModernAccountQr challenge);
  Future<ModernAccountResult> modernAccountSession(
    String accountId,
    ModernAccountOperation operation,
  );
}
