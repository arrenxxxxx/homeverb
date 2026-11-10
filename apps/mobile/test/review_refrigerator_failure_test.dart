import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/native_client.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;

import 'refrigerator_modes_test.dart' show ModeFixture;

class FailedRefrigerator extends ModeFixture {
  final gate = Completer<void>();
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    if (method == 'read_state' && fields['component'] == 'fridge') {
      if (fields['cached'] == false) await gate.future;
      throw NativeFailure('STATE_UNAVAILABLE', 'Fictional read failed.');
    }
    return super.call(method, fields);
  }
}

void main() {
  testWidgets('supported refrigerator rows survive first-read failure', (
    tester,
  ) async {
    final core = FailedRefrigerator();
    final c = HomeController(core, LocalPreferences.memory());
    final app = GlobalKey<NativeAppState>();
    final catalog = (await tester.runAsync(ScreenCatalog.load))!;
    await tester.pumpWidget(
      NativeApp(key: app, catalog: catalog, controller: c),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Example refrigerator').first);
    for (var i = 0; i < 10; i++) await tester.pump();
    Finder readings() => find.byWidgetPredicate(
      (widget) =>
          widget.key?.toString().contains('refrigerator-reading:') == true,
    );
    final before = readings().evaluate().map((e) => e.widget.key).toSet();
    expect(before.length, greaterThan(10));
    expect(find.byType(CircularProgressIndicator), findsWidgets);
    core.gate.complete();
    await tester.pumpAndSettle();
    final after = readings().evaluate().map((e) => e.widget.key).toSet();
    expect(after, before);
    expect(
      find.descendant(of: readings(), matching: find.text('—')),
      findsWidgets,
    );
    expect(c.componentReadError('fridge'), isA<NativeFailure>());
    await tester.pumpWidget(const SizedBox());
  });
}
