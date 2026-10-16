import 'dart:async';
import 'dart:io';
import 'package:flutter/services.dart';

enum CredentialReadiness {
  notApplicable,
  ready,
  screenLockRequired,
  locked,
  unavailable,
}

abstract interface class CredentialAccess {
  Future<CredentialReadiness> check();
  Future<bool> openSecuritySettings();
}

/// A local host preflight, not proof that a later Keystore operation will succeed.
class PlatformCredentialAccess implements CredentialAccess {
  const PlatformCredentialAccess({this.android});
  final bool? android;
  static const channel = MethodChannel('dev.homeverb/credential_access');
  bool get isAndroid => android ?? Platform.isAndroid;

  @override
  Future<CredentialReadiness> check() async {
    if (!isAndroid) return CredentialReadiness.notApplicable;
    try {
      return switch (await channel
          .invokeMethod<Object?>('check')
          .timeout(const Duration(seconds: 3))) {
        'ready' => CredentialReadiness.ready,
        'screen_lock_required' => CredentialReadiness.screenLockRequired,
        'locked' => CredentialReadiness.locked,
        _ => CredentialReadiness.unavailable,
      };
    } on PlatformException {
      return CredentialReadiness.unavailable;
    } on TimeoutException {
      return CredentialReadiness.unavailable;
    } on MissingPluginException {
      return CredentialReadiness.unavailable;
    }
  }

  @override
  Future<bool> openSecuritySettings() async {
    if (!isAndroid) return false;
    try {
      return await channel
              .invokeMethod<Object?>('openSecuritySettings')
              .timeout(const Duration(seconds: 3)) ==
          true;
    } on PlatformException {
      return false;
    } on TimeoutException {
      return false;
    } on MissingPluginException {
      return false;
    }
  }
}
