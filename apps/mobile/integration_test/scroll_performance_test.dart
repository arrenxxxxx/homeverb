import 'dart:convert';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:homeverb_mobile/blocks.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/screens.dart';
import 'package:homeverb_mobile/skin.dart';

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets(
    'profile the production device grid with fictional cards and no Core session',
    (tester) async {
      expect(
        kProfileMode,
        isTrue,
        reason: 'Frame timings require profile mode.',
      );
      await LocaleSettings.setLocale(AppLocale.zhCn);
      final catalog = await ScreenCatalog.load();
      final boundary = GlobalKey();
      var actions = 0;
      void onAction(String _) => actions++;
      await tester.pumpWidget(
        TranslationProvider(
          child: MaterialApp(
            locale: AppLocale.zhCn.flutterLocale,
            theme: windowTheme(),
            home: Scaffold(
              body: RepaintBoundary(
                key: boundary,
                child: ScreenView(
                  id: 'D01',
                  catalog: catalog,
                  onAction: onAction,
                  fields: {},
                  choices: {},
                  values: {},
                  home: DeviceHome(
                    mode: 'all',
                    onAction: onAction,
                    cards: DeviceGrid(
                      keys: List.generate(12, (i) => 'fictional-$i'),
                      data: List.generate(
                        12,
                        (i) => [
                          'Fictional AC ${i + 1}',
                          'Unassigned',
                          '24°C',
                          'Saved observation',
                        ],
                      ),
                      onAction: onAction,
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      await precacheImage(
        const AssetImage('assets/ac.png'),
        tester.element(find.byType(ScreenView)),
      );
      await tester.pumpAndSettle();
      final scroll = find.byKey(const ValueKey('scroll:D01'));
      Future<void> sweep() async {
        for (final distance in [-600.0, 600.0, -600.0, 600.0]) {
          await tester.fling(scroll, Offset(0, distance), 1800);
          await tester.pumpAndSettle();
        }
      }

      // Decode images and warm the renderer before recording steady scrolling.
      await sweep();
      await binding.watchPerformance(sweep, reportKey: 'device_scroll');
      expect(actions, 0);
      expect(tester.takeException(), isNull);
      final report = binding.reportData!;
      final summary = report['device_scroll'] as Map;
      debugPrint(
        'SCROLL_PROFILE ' +
            jsonEncode({
              for (final entry in summary.entries)
                if (entry.value is num) '${entry.key}': entry.value,
            }),
      );
      const output = String.fromEnvironment('HOMEVERB_PERFORMANCE_OUTPUT');
      if (output.isNotEmpty) {
        await File(output).writeAsString(
          const JsonEncoder.withIndent('  ').convert(report) + '\n',
        );
      }
      await tester.pumpWidget(const SizedBox());
      await LocaleSettings.setLocale(AppLocale.enUs);
    },
    skip: !const bool.fromEnvironment('HOMEVERB_PERFORMANCE'),
  );
}
