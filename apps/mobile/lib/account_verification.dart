import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'client_contract.dart';
import 'localization.dart';
import 'skin.dart';

/// Foreground user interaction only. Timers never request codes or retry login.
class AccountVerificationPrompt extends StatefulWidget {
  const AccountVerificationPrompt({
    required this.challenge,
    this.cancellation,
    super.key,
  });
  final AccountVerificationPending challenge;
  final Listenable? cancellation;
  @override
  State<AccountVerificationPrompt> createState() =>
      _AccountVerificationPromptState();
}

class _AccountVerificationPromptState extends State<AccountVerificationPrompt>
    with WidgetsBindingObserver {
  final answer = TextEditingController();
  Timer? expiry, countdown;
  MemoryImage? image;
  AccountVerificationMethod? method;
  bool finished = false, invalid = false, imageFailed = false;
  bool get captcha => widget.challenge.kind == AccountVerificationKind.captcha;
  bool get canSend =>
      !captcha &&
      method != null &&
      widget.challenge.cooldown == Duration.zero &&
      widget.challenge.codeRequests < 3 &&
      widget.challenge.remaining > Duration.zero;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    if (captcha) image = MemoryImage(widget.challenge.image);
    method = widget.challenge.selectedMethod;
    if (method == null && widget.challenge.methods.length == 1) {
      method = widget.challenge.methods.single;
    }
    widget.cancellation?.addListener(cancel);
    expiry = Timer(widget.challenge.remaining, () => finish());
    if (!captcha) {
      countdown = Timer.periodic(const Duration(seconds: 1), (_) {
        if (mounted && !finished) setState(() {});
      });
    }
  }

  void clear() {
    expiry?.cancel();
    countdown?.cancel();
    answer.clear();
    image?.evict();
    widget.challenge.clearImage();
  }

  void cancel() {
    clear();
    scheduleMicrotask(() => finish());
  }

  void finish([AccountVerificationReply? reply]) {
    if (finished || !mounted) {
      reply?.clear();
      return;
    }
    setState(() => finished = true);
    clear();
    Navigator.of(context).pop(reply);
  }

  void submit() {
    if (finished ||
        imageFailed ||
        widget.challenge.remaining == Duration.zero) {
      finish();
      return;
    }
    final bytes = Uint8List.fromList(utf8.encode(answer.text));
    final valid = captcha
        ? RegExp(r'^[A-Za-z0-9]{1,32}$').hasMatch(answer.text)
        : method != null &&
              widget.challenge.methods.contains(method) &&
              bytes.isNotEmpty &&
              bytes.length <= 2048 &&
              !answer.text.runes.any((r) => r <= 32 || r >= 127 && r <= 159);
    if (!valid) {
      bytes.fillRange(0, bytes.length, 0);
      setState(() => invalid = true);
      return;
    }
    finish(
      AccountVerificationReply.answer(bytes, method: captcha ? null : method),
    );
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) finish();
  }

  @override
  void dispose() {
    widget.cancellation?.removeListener(cancel);
    WidgetsBinding.instance.removeObserver(this);
    clear();
    answer.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => PopScope<AccountVerificationReply>(
    onPopInvokedWithResult: (didPop, _) {
      if (didPop) {
        finished = true;
        clear();
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
                  if (captcha) ...[
                    const Copy(
                      'Enter the letters and numbers in the image. Verification expires shortly; sign in again if it closes.',
                      control: true,
                    ),
                    const SizedBox(height: 16),
                    Image(
                      image: image!,
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
                  ] else ...[
                    const Copy(
                      'Choose an available method and request a code, then enter it here. Verification expires shortly.',
                      control: true,
                    ),
                    const SizedBox(height: 12),
                    Wrap(
                      spacing: 8,
                      children: [
                        for (final offered in widget.challenge.methods)
                          ChoiceChip(
                            key: ValueKey(
                              'verification-method:${offered.name}',
                            ),
                            label: Copy(
                              offered == AccountVerificationMethod.phone
                                  ? 'Text Message'
                                  : 'Email Code',
                              control: true,
                            ),
                            selected: method == offered,
                            onSelected: (_) => setState(() {
                              method = offered;
                              answer.clear();
                              invalid = false;
                            }),
                          ),
                      ],
                    ),
                    TextButton(
                      key: const ValueKey('verification-send'),
                      onPressed: canSend
                          ? () => finish(AccountVerificationReply.send(method!))
                          : null,
                      child: const Copy(
                        'Send Verification Code',
                        control: true,
                      ),
                    ),
                    if (widget.challenge.codeRequests >= 3)
                      const Copy(
                        'Code request limit reached. Enter your code or sign in again.',
                        control: true,
                      )
                    else if (widget.challenge.cooldown > Duration.zero)
                      Copy(
                        UiMessage(
                          'You can request another code in {seconds} seconds.',
                          {
                            'seconds':
                                (widget.challenge.cooldown.inMilliseconds /
                                        1000)
                                    .ceil(),
                          },
                        ),
                        control: true,
                      ),
                  ],
                  const SizedBox(height: 16),
                  TextField(
                    key: ValueKey(
                      captcha ? 'captcha-answer' : 'verification-answer',
                    ),
                    controller: answer,
                    autocorrect: false,
                    enableSuggestions: false,
                    enableIMEPersonalizedLearning: false,
                    smartDashesType: SmartDashesType.disabled,
                    smartQuotesType: SmartQuotesType.disabled,
                    autofillHints: null,
                    keyboardType: TextInputType.visiblePassword,
                    maxLength: captcha ? 32 : 2048,
                    decoration: InputDecoration(
                      labelText: localize(context, 'Verification Code'),
                    ),
                    onSubmitted: (_) => submit(),
                  ),
                  if (invalid)
                    Copy(
                      captcha
                          ? 'Use only the letters and numbers shown.'
                          : 'Select a method and enter a valid verification code without spaces.',
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
