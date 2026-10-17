import 'dart:convert';
import 'dart:io';
import 'client_contract.dart';
import 'localization.dart';

class LocalPreferences {
  LocalPreferences(File file) : file = file;
  LocalPreferences.memory() : file = null;
  final File? file;
  Json values = {};
  String get skin => 'window_light';
  String? get locale => canonicalLocale(values['locale']);
  String get sharingName => values['sharing_name'] as String? ?? 'Home phone';
  String get root => values['root'] as String? ?? 'D01';
  String? accountLoginLabel(String id) {
    final labels = values['account_login_labels'];
    final value = labels is Map ? labels[id] : null;
    return value is String &&
            value.length <= 80 &&
            value.contains('***') &&
            RegExp(r'^[\x21-\x7e]+$').hasMatch(value)
        ? value
        : null;
  }

  Future<void> saveAccountLoginLabel(String id, String label) => save({
    'account_login_labels': {
      for (final entry
          in (values['account_login_labels'] is Map
                  ? values['account_login_labels'] as Map
                  : {})
              .entries)
        if (entry.key is String && accountLoginLabel(entry.key) != null)
          entry.key as String: entry.value,
      id: label,
    },
  });

  Future<void> retainAccountLoginLabels(Set<String> ids) async {
    final labels = values['account_login_labels'];
    if (labels is! Map || labels.keys.every(ids.contains)) return;
    await save({
      'account_login_labels': {
        for (final id in ids)
          if (accountLoginLabel(id) case final String label) id: label,
      },
    });
  }

  String route([String? device]) =>
      (values['device_routes'] as Map?)?[device] as String? ??
      values['route'] as String? ??
      'auto';
  Future<void> load() async {
    final file = this.file;
    if (file == null) return;
    try {
      final data = jsonDecode(await file.readAsString());
      if (data is Map<String, dynamic> && data['schema_version'] == 1) {
        values = data;
        if (!['auto', 'lan', 'cloud'].contains(values['route']))
          values.remove('route');
        if (!['D01', 'P01', 'S01'].contains(values['root']))
          values.remove('root');
        final routes = values['device_routes'];
        values['device_routes'] = routes is Map
            ? Map.fromEntries(
                routes.entries.where(
                  (e) =>
                      e.key is String &&
                      ['auto', 'lan', 'cloud'].contains(e.value),
                ),
              )
            : {};
        if (canonicalLocale(values['locale']) == null) values.remove('locale');
        if (values['sharing_name'] is! String) values.remove('sharing_name');
      }
    } on FileSystemException {
      /* Missing or unreadable preferences use defaults. */
    } on FormatException {
      /* Corrupt preferences do not affect Core data. */
    }
  }

  Future<void> save(Json changes) async {
    final next = {...values, ...changes, 'schema_version': 1, 'skin_id': skin};
    final file = this.file;
    if (file != null) {
      await file.parent.create(recursive: true);
      final pending = File('${file.path}.pending');
      await pending.writeAsString(jsonEncode(next), flush: true);
      await pending.rename(file.path);
    }
    values = next;
  }
}
