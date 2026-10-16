import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/screens.dart';
import 'package:homeverb_mobile/skin.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('sheet actions stay fully above Android system navigation', (
    tester,
  ) async {
    await (FontLoader('Inter')
          ..addFont(rootBundle.load('assets/fonts/Inter-Regular.otf'))
          ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.otf')))
        .load();
    final catalog = await tester.runAsync(ScreenCatalog.load);
    tester.view.physicalSize = const Size(1080, 2400);
    tester.view.devicePixelRatio = 2.625;
    addTearDown(() {
      tester.view.resetPhysicalSize();
      tester.view.resetDevicePixelRatio();
    });
    String? action;
    await tester.pumpWidget(
      MaterialApp(
        theme: windowTheme(),
        home: MediaQuery(
          data: const MediaQueryData(
            size: Size(1080 / 2.625, 2400 / 2.625),
            devicePixelRatio: 2.625,
            padding: EdgeInsets.only(top: 52, bottom: 48),
            viewPadding: EdgeInsets.only(top: 52, bottom: 48),
          ),
          child: Scaffold(
            body: ScreenView(
              id: 'C05',
              catalog: catalog!,
              screenOverride: {
                ...catalog.ui('C05'),
                'header': 'Choose Mode',
                'base': 'C01',
                'overlay': 'sheet',
                'blocks': [
                  [
                    'choices',
                    ['cool', 'heat', 'fan', 'dry', 'auto'],
                    0,
                  ],
                  ['button', 'Save Mode'],
                  ['button', 'Cancel', 'secondary'],
                ],
              },
              onAction: (value) => action = value,
              fields: {},
              choices: {},
              values: {},
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final cancel = find.widgetWithText(FilledButton, 'Cancel');
    final rect = tester.getRect(cancel);
    expect(rect.height, greaterThanOrEqualTo(48));
    expect(rect.bottom, lessThanOrEqualTo(2400 / 2.625 - 48));
    await tester.tapAt(Offset(rect.center.dx, rect.bottom - 2));
    expect(action, 'Cancel');
    expect(tester.takeException(), isNull);
  });
}
