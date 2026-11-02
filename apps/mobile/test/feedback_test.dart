import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/feedback.dart';
import 'package:homeverb_mobile/skin.dart';

void main() {
  for (final contrast in [false, true]) {
    for (final skin in ['window_light', 'sky_blue']) {
      testWidgets(
        'notice and toast actions stay separate in $skin at large text contrast=$contrast',
        (tester) async {
          tester.view.physicalSize = const Size(320, 900);
          tester.view.devicePixelRatio = 1;
          tester.platformDispatcher.textScaleFactorTestValue = 2;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
          var actions = 0;
          await tester.pumpWidget(
            MaterialApp(
              theme: windowTheme(skinId: skin),
              builder: (context, child) => MediaQuery(
                data: MediaQuery.of(context).copyWith(highContrast: contrast),
                child: child!,
              ),
              home: Scaffold(
                body: SingleChildScrollView(
                  child: Column(
                    children: [
                      FeedbackNotice(
                        title: 'Sign in again',
                        message: 'Your saved devices are kept.',
                        warning: true,
                        actionLabel: 'Go to sign in',
                        onAction: () => actions++,
                      ),
                      Container(
                        color: const Color(0xff263241),
                        padding: const EdgeInsets.all(16),
                        child: FeedbackToast(
                          message:
                              'Could not refresh. Your last readings are kept.',
                          actionLabel: 'Retry',
                          onAction: () => actions++,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          );
          expect(tester.takeException(), isNull);
          final notice = tester.widget<Container>(
            find.byKey(const ValueKey('feedback-notice')),
          );
          if (contrast)
            expect((notice.decoration as BoxDecoration).color, Colors.white);
          final button = find.widgetWithText(OutlinedButton, 'Go to sign in');
          expect(tester.getSize(button).height, greaterThanOrEqualTo(48));
          await tester.tap(find.text('Your saved devices are kept.'));
          await tester.tap(
            find.text('Could not refresh. Your last readings are kept.'),
          );
          expect(actions, 0);
          await tester.tap(find.text('Go to sign in'));
          await tester.tap(find.text('Retry'));
          expect(actions, 2);
          expect(tester.takeException(), isNull);
        },
      );
    }
  }

  testWidgets(
    'queue deduplicates, bounds backlog and clears all navigation feedback',
    (tester) async {
      final queue = FeedbackQueue();
      late BuildContext context;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (c) {
                context = c;
                return const SizedBox();
              },
            ),
          ),
        ),
      );
      void show(String key) => queue.show(
        context,
        key: key,
        content: FeedbackToast(message: key),
      );
      show('first');
      show('first');
      show('second');
      show('third');
      show('dropped');
      await tester.pumpAndSettle();
      expect(find.text('first'), findsOneWidget);
      await tester.pump(const Duration(seconds: 9));
      await tester.pumpAndSettle();
      expect(find.text('second'), findsOneWidget);
      queue.clear();
      await tester.pumpAndSettle();
      expect(find.byType(FeedbackToast), findsNothing);
      await tester.pump(const Duration(seconds: 30));
      expect(find.byType(FeedbackToast), findsNothing);
      show('first');
      await tester.pumpAndSettle();
      expect(find.text('first'), findsOneWidget);
      queue.clear();
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets(
    'screen reader action remains available and reduced motion is honored',
    (tester) async {
      final queue = FeedbackQueue();
      late BuildContext context;
      await tester.pumpWidget(
        MaterialApp(
          home: MediaQuery(
            data: const MediaQueryData(
              accessibleNavigation: true,
              disableAnimations: true,
            ),
            child: Scaffold(
              body: Builder(
                builder: (c) {
                  context = c;
                  return const SizedBox();
                },
              ),
            ),
          ),
        ),
      );
      queue.show(
        context,
        key: 'read',
        hasAction: true,
        content: FeedbackToast(
          message: 'Read failed',
          actionLabel: 'Retry',
          onAction: () {},
        ),
      );
      await tester.pumpAndSettle();
      await tester.pump(const Duration(minutes: 1));
      expect(find.text('Retry'), findsOneWidget);
      queue.clear();
      await tester.pumpWidget(const SizedBox());
    },
  );
}
