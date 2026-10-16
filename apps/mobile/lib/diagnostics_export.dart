import 'package:flutter/services.dart';

const _documents = MethodChannel('dev.homeverb/document_export');

Future<bool> exportAndroidDiagnostics(String exactPreview) async {
  final saved = await _documents.invokeMethod<bool>('saveDiagnostics', {
    'text': exactPreview,
  });
  if (saved == null) throw StateError('Missing document export result.');
  return saved;
}
