import 'package:flutter/widgets.dart';
import 'gen/strings.g.dart';
import 'gen/source_messages.g.dart';
import 'client_contract.dart';

export 'gen/strings.g.dart';
export 'gen/source_messages.g.dart' show sourceForKey;

/// User/provider values are never looked up as interface translation keys.
class LiteralText {
  const LiteralText(this.value);
  final Object? value;
  @override
  String toString() => value?.toString() ?? '';
  String toJson() => toString();
}

/// Parameters stay separate from the authored template, including nested labels.
class UiError {
  const UiError(this.code);
  final String code;
  @override
  String toString() => code;
  String toJson() => code;
}

class UiMessage {
  const UiMessage(this.source, [this.arguments = const {}]);
  final String source;
  final Map<String, Object> arguments;
  @override
  String toString() => source.replaceAllMapped(
    RegExp(r'\{([a-zA-Z]+)\}'),
    (m) => arguments[m[1]]?.toString() ?? m[0]!,
  );
  String toJson() => toString();
}

Translations translationsOf(BuildContext context) =>
    context
        .dependOnInheritedWidgetOfExactType<
          InheritedLocaleData<AppLocale, Translations>
        >()
        ?.translations ??
    AppLocale.enUs.translations;

String localize(BuildContext context, Object? value) {
  if (value is LiteralText) return value.toString();
  final t = translationsOf(context);
  if (value is UiError)
    return '${value.code}: ${translateError(t, value.code)}';
  if (value is UiMessage) {
    final arguments = value.arguments.map(
      (key, value) => MapEntry(
        key,
        value is UiMessage ? localize(context, value) : value.toString(),
      ),
    );
    return translateSource(t, value.source, arguments) ?? value.toString();
  }
  final source = value?.toString() ?? '';
  return translateSource(t, source, const {}) ?? source;
}

String localizedError(BuildContext context, Object error) {
  final code = error is NativeFailure ? error.code : 'UNKNOWN';
  return '$code: ${translateError(translationsOf(context), code)}';
}

String? canonicalLocale(Object? value) =>
    switch (value?.toString().replaceAll('_', '-').toLowerCase()) {
      'zh-cn' || 'zhcn' => 'zh-CN',
      'en-us' || 'enus' => 'en-US',
      _ => null,
    };

Future<void> applyLocalePreference(String? preference) async {
  if (preference == null) {
    await LocaleSettings.useDeviceLocale();
  } else {
    await LocaleSettings.setLocaleRaw(canonicalLocale(preference) ?? 'en-US');
  }
}
