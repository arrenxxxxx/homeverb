import 'dart:typed_data';

/// Display-only projection of the submitted login, never an identity guard.
String? maskedAccountLogin(Uint8List login) {
  if (login.isEmpty ||
      login.length > 256 ||
      login.any((byte) => byte < 0x21 || byte > 0x7e)) {
    return null;
  }
  String part(int start, int end) =>
      String.fromCharCodes(login.sublist(start, end));
  var phoneStart = 0;
  if (login.length == 14 && part(0, 3) == '+86') phoneStart = 3;
  if (login.length - phoneStart == 11 &&
      login.skip(phoneStart).every((byte) => byte >= 0x30 && byte <= 0x39)) {
    return '${part(phoneStart, phoneStart + 3)}****${part(login.length - 4, login.length)}';
  }
  final at = login.indexOf(0x40);
  if (at > 0 && at < login.length - 1) {
    final end = login.length < at + 61 ? login.length : at + 61;
    return '${part(0, 1)}***${part(at, end)}';
  }
  if (login.length < 5) return '${part(0, 1)}***';
  return '${part(0, 2)}***${part(login.length - 2, login.length)}';
}

/// Platform labels are account presentation only, never device business semantics.
String accountPlatformName(String? provider) => switch (provider) {
  'xiaomi.mijia' => 'Mijia',
  'midea.meiju' => 'Meiju',
  _ => 'Platform',
};
String accountDefaultLabel(String? provider) => switch (provider) {
  'xiaomi.mijia' => 'Mijia account',
  'midea.meiju' => 'Meiju account',
  _ => 'Account name',
};
