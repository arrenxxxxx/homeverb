import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/device_detail_controls.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/skin.dart';
import 'accounts_test.dart' show AccountPreferences;
import 'controls_test.dart' show capability;
import 'mijia_test.dart' show MijiaReadFixture;

void main() {
  for (final locale in [AppLocale.enUs, AppLocale.zhCn]) {
    testWidgets(
      'reference scalar readings retain units, absence and read-only behavior in $locale',
      (tester) async {
        LocaleSettings.setLocale(locale);
        addTearDown(() => LocaleSettings.setLocale(AppLocale.enUs));
        tester.view.physicalSize = const Size(640, 1600);
        tester.view.devicePixelRatio = 2;
        tester.platformDispatcher.textScaleFactorTestValue = 2;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
        final core = MijiaReadFixture('doorbell');
        core.capabilities
          ..clear()
          ..add(
            capability('vendor.xiaomi.mijia.environment', {
              'relative_humidity_1': {
                'type': 'integer',
                'readable': true,
                'writable': false,
                'unit': 'percentage',
              },
              'pm2_5_density_2': {
                'type': 'number',
                'readable': true,
                'writable': false,
              },
              'temperature_3': {
                'type': 'number',
                'readable': true,
                'writable': false,
                'unit': 'celsius',
              },
            }, {}),
          );
        core.device['components'] = [
          {
            'id': 'main',
            'capabilities': ['vendor.xiaomi.mijia.environment'],
          },
        ];
        core.observations
          ..clear()
          ..addAll({
            'vendor.xiaomi.mijia.environment.relative_humidity_1': 55,
            'vendor.xiaomi.mijia.environment.pm2_5_density_2': 0,
            'vendor.xiaomi.mijia.environment.temperature_3': null,
          });
        final controller = HomeController(core, AccountPreferences());
        expect(await controller.selectDevice(core.device['id']), true);
        await tester.pumpWidget(
          TranslationProvider(
            child: MaterialApp(
              theme: windowTheme(),
              home: Scaffold(
                body: SingleChildScrollView(
                  child: DeviceDetailControls(
                    controller: controller,
                    onAction: (_) => fail('Read-only panel executed an action'),
                  ),
                ),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        expect(
          find.text(
            locale == AppLocale.zhCn
                ? '\u76f8\u5bf9\u6e7f\u5ea6'
                : 'Relative humidity',
          ),
          findsOneWidget,
        );
        expect(find.textContaining('55%'), findsOneWidget);
        expect(
          controller.observed(
            'vendor.xiaomi.mijia.environment.pm2_5_density_2',
          ),
          0,
        );
        expect(
          controller.observed('vendor.xiaomi.mijia.environment.temperature_3'),
          isNull,
        );
        expect(
          core.calls.where(
            (c) => ['execute', 'plan', 'refresh_account'].contains(c['method']),
          ),
          isEmpty,
        );
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        controller.dispose();
      },
    );
  }
}
