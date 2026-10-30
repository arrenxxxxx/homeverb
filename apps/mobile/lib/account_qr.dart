import 'dart:async';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:qr/qr.dart';
import 'client_contract.dart';
import 'skin.dart';

/// Encodes locally; only the clearable module bitmap survives construction.
class AccountQrBitmap {
  AccountQrBitmap(Uint8List bytes) {
    if (bytes.isEmpty || bytes.length > 8192) throw const FormatException();
    final image = QrImage(
      QrCode.fromUint8List(
        data: bytes,
        errorCorrectLevel: QrErrorCorrectLevel.M,
      ),
    );
    size = image.moduleCount;
    modules = Uint8List(size * size);
    for (var y = 0; y < size; y++) {
      for (var x = 0; x < size; x++) {
        modules[y * size + x] = image.isDark(y, x) ? 1 : 0;
      }
    }
  }
  late final int size;
  late final Uint8List modules;
  void clear() => modules.fillRange(0, modules.length, 0);
  @override
  String toString() => 'AccountQrBitmap(<protected>)';
}

class AccountQrPainter extends CustomPainter {
  AccountQrPainter(this.bitmap);
  final AccountQrBitmap bitmap;
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()..isAntiAlias = false;
    canvas.drawRect(Offset.zero & size, paint..color = Colors.white);
    final unit = size.shortestSide / (bitmap.size + 8);
    paint.color = Colors.black;
    for (var y = 0; y < bitmap.size; y++) {
      for (var x = 0; x < bitmap.size; x++) {
        if (bitmap.modules[y * bitmap.size + x] == 1) {
          canvas.drawRect(
            Rect.fromLTWH((x + 4) * unit, (y + 4) * unit, unit, unit),
            paint,
          );
        }
      }
    }
  }

  @override
  bool shouldRepaint(AccountQrPainter oldDelegate) => true;
}

/// A local expiry timer never polls, regenerates or continues authentication.
class AccountQrPrompt extends StatefulWidget {
  const AccountQrPrompt({
    required this.challenge,
    this.cancellation,
    super.key,
  });
  final ModernAccountQr challenge;
  final Listenable? cancellation;
  @override
  State<AccountQrPrompt> createState() => _AccountQrPromptState();
}

class _AccountQrPromptState extends State<AccountQrPrompt>
    with WidgetsBindingObserver {
  AccountQrBitmap? bitmap;
  Timer? expiry;
  bool finished = false, cancelled = false;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    widget.cancellation?.addListener(cancel);
    try {
      if (widget.challenge.remaining > Duration.zero) {
        bitmap = AccountQrBitmap(widget.challenge.scanData);
      }
    } catch (_) {
      // Never expose encoder exceptions or protected input in UI/logs.
      bitmap = null;
    } finally {
      widget.challenge.clearScan();
    }
    expiry = Timer(widget.challenge.remaining, cancel);
  }

  void clear() {
    expiry?.cancel();
    bitmap?.clear();
    widget.challenge.clearScan();
  }

  void cancel() {
    cancelled = true;
    clear();
    scheduleMicrotask(() => finish());
  }

  void finish([bool approved = false]) {
    if (finished || !mounted) return;
    final result =
        approved &&
        !cancelled &&
        bitmap != null &&
        widget.challenge.remaining > Duration.zero;
    setState(() => finished = true);
    clear();
    Navigator.of(context).pop(result);
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) cancel();
  }

  @override
  void dispose() {
    widget.cancellation?.removeListener(cancel);
    WidgetsBinding.instance.removeObserver(this);
    clear();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => PopScope<bool>(
    onPopInvokedWithResult: (didPop, _) {
      if (didPop) {
        finished = true;
        clear();
      }
    },
    child: finished
        ? const SizedBox.shrink()
        : AlertDialog(
            title: const Copy('Sign In with QR Code', control: true),
            content: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Copy(
                    'Keep HomeVerb open. Scan with Xiaomi Home on another device and approve there, then select Continue. Leaving HomeVerb cancels this attempt.',
                    control: true,
                  ),
                  const SizedBox(height: 16),
                  if (bitmap != null)
                    ExcludeSemantics(
                      child: SizedBox(
                        width: 280,
                        child: AspectRatio(
                          aspectRatio: 1,
                          child: CustomPaint(
                            key: const ValueKey('account-qr'),
                            painter: AccountQrPainter(bitmap!),
                          ),
                        ),
                      ),
                    )
                  else
                    const Copy(
                      'The QR code could not be displayed. Cancel and sign in again.',
                      control: true,
                    ),
                ],
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => finish(),
                child: const Copy('Cancel', control: true),
              ),
              TextButton(
                key: const ValueKey('qr-continue'),
                onPressed: bitmap == null ? null : () => finish(true),
                child: const Copy('Continue', control: true),
              ),
            ],
          ),
  );
}
