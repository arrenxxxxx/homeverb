import 'package:flutter/material.dart';
import 'screens.dart';
import 'localization.dart';
import 'preview_climate.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  const locale = String.fromEnvironment(
    'PREVIEW_LOCALE',
    defaultValue: 'zh-CN',
  );
  await applyLocalePreference(locale);
  final catalog = await ScreenCatalog.load();
  const initial = String.fromEnvironment('PREVIEW_SCREEN', defaultValue: 'D01');
  runApp(
    TranslationProvider(
      child: PreviewApp(
        catalog: catalog,
        initial: catalog.screens.containsKey(initial) ? initial : 'D01',
        contentBuilder: (id, onAction) =>
            id == 'C01' ? PreviewClimate(onAction: onAction) : null,
      ),
    ),
  );
}
