import 'dart:async';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'client_contract.dart';
import 'localization.dart';
import 'skin.dart';

/// A user answer only: no image upload, automatic solving or sign-in retry.
class AccountCaptchaPrompt extends StatefulWidget {
  const AccountCaptchaPrompt({
    required this.challenge,
    this.cancellation,
    super.key,
  });
  final AccountLoginCaptcha challenge;
  final Listenable? cancellation;
  @override
  State<AccountCaptchaPrompt> createState() => _AccountCaptchaPromptState();
}

class _AccountCaptchaPromptState extends State<AccountCaptchaPrompt>
    with WidgetsBindingObserver {
  final answer = TextEditingController();
  Timer? expiry;
  late final MemoryImage image;
  bool invalid = false, finished = false, imageFailed = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    image = MemoryImage(widget.challenge.image);
    widget.cancellation?.addListener(cancel);
    expiry = Timer(widget.challenge.remaining, () => finish());
  }

  void cancel() {
    answer.clear();
    image.evict();
    widget.challenge.clearImage();
    // Navigation can invalidate the flow during parent disposal.
    scheduleMicrotask(() => finish());
  }

  void finish([Uint8List? value]) {
    if (finished || !mounted) {
      value?.fillRange(0, value.length, 0);
      return;
    }
    setState(() => finished = true);
    expiry?.cancel();
    answer.clear();
    image.evict();
    widget.challenge.clearImage();
    Navigator.of(context).pop(value);
  }

  void submit() {
    if (finished ||
        imageFailed ||
        widget.challenge.remaining == Duration.zero) {
      finish();
      return;
    }
    if (!RegExp(r'^[A-Za-z0-9]{1,32}$').hasMatch(answer.text)) {
      setState(() => invalid = true);
      return;
    }
    finish(Uint8List.fromList(answer.text.codeUnits));
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) finish();
  }

  @override
  void dispose() {
    expiry?.cancel();
    widget.cancellation?.removeListener(cancel);
    WidgetsBinding.instance.removeObserver(this);
    answer.clear();
    answer.dispose();
    image.evict();
    widget.challenge.clearImage();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => PopScope<Uint8List>(
    onPopInvokedWithResult: (didPop, _) {
      if (didPop) {
        setState(() => finished = true);
        expiry?.cancel();
        answer.clear();
        image.evict();
        widget.challenge.clearImage();
      }
    },
    child: finished
        ? const SizedBox.shrink()
        : AlertDialog(
            title: const Copy('Complete Sign-in Verification', control: true),
            content: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  const Copy(
                    'Enter the letters and numbers in the image. Verification expires shortly; sign in again if it closes.',
                    control: true,
                  ),
                  const SizedBox(height: 16),
                  Image(
                    image: image,
                    height: 100,
                    fit: BoxFit.contain,
                    excludeFromSemantics: true,
                    errorBuilder: (_, _, _) {
                      imageFailed = true;
                      return const Copy(
                        'The verification image could not load. Cancel and sign in again.',
                        control: true,
                      );
                    },
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    key: const ValueKey('captcha-answer'),
                    controller: answer,
                    autocorrect: false,
                    enableSuggestions: false,
                    enableIMEPersonalizedLearning: false,
                    smartDashesType: SmartDashesType.disabled,
                    smartQuotesType: SmartQuotesType.disabled,
                    autofillHints: null,
                    keyboardType: TextInputType.visiblePassword,
                    maxLength: 32,
                    decoration: InputDecoration(
                      labelText: localize(context, 'Verification Code'),
                    ),
                    onSubmitted: (_) => submit(),
                  ),
                  if (invalid)
                    const Copy(
                      'Use only the letters and numbers shown.',
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
                onPressed: submit,
                child: const Copy('Verify and Continue', control: true),
              ),
            ],
          ),
  );
}
