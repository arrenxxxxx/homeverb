import 'package:flutter/material.dart';
import 'screens.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final catalog = await ScreenCatalog.load();
  const initial = String.fromEnvironment('PREVIEW_SCREEN', defaultValue: 'D01');
  runApp(
    PreviewApp(
      catalog: catalog,
      initial: catalog.screens.containsKey(initial) ? initial : 'D01',
    ),
  );
}
