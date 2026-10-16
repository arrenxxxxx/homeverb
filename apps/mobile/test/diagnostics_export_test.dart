import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/diagnostics_export.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const channel = MethodChannel('dev.homeverb/document_export');
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
  tearDown(() => messenger.setMockMethodCallHandler(channel, null));

  test(
    'Android export sends the exact preview and distinguishes cancellation',
    () async {
      const preview = '{\n  "scope": "local_profile",\n  "device_count": 1\n}';
      for (final saved in [false, true]) {
        messenger.setMockMethodCallHandler(channel, (call) async {
          expect(call.method, 'saveDiagnostics');
          expect(call.arguments, {'text': preview});
          return saved;
        });
        expect(await exportAndroidDiagnostics(preview), saved);
      }
    },
  );

  test(
    'Android export propagates failure and rejects missing confirmation',
    () async {
      messenger.setMockMethodCallHandler(channel, (_) async {
        throw PlatformException(code: 'EXPORT_FAILED');
      });
      await expectLater(
        exportAndroidDiagnostics('{}'),
        throwsA(isA<PlatformException>()),
      );
      messenger.setMockMethodCallHandler(channel, (_) async => null);
      await expectLater(exportAndroidDiagnostics('{}'), throwsStateError);
    },
  );
}
