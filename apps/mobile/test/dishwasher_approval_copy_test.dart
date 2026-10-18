import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/approval_review.dart';
import 'package:homeverb_mobile/native_client.dart';

// Display-only fixture; NativeClient refuses externally constructed previews.
class DisplayPreview implements ApprovalPreview {
  DisplayPreview(String action, bool additional) {
    command = {
      'request_id': 'fictional-request',
      'device_id': 'fictional-device',
      'component_id': 'main',
      'capability': 'vendor.midea.dishwasher',
      'action': action,
      'arguments': {if (additional) 'additional': 9},
    };
  }
  @override
  late final Json command;
  @override
  Json get descriptor => {
    'device': {'alias': 'Fictional dishwasher'},
  };
  @override
  Json get connection => {'transport': 'cloud', 'id': 'fictional-cloud'};
  @override
  DateTime get expiresAt => DateTime.utc(2026, 10, 18);
  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnsupportedError('Display-only preview');
}

void main() {
  for (final action in ['power_on', 'power_off', 'pause_cycle']) {
    for (final additional in [false, true]) {
      testWidgets('confirmation names explicit effects: $action/$additional', (
        tester,
      ) async {
        final labels = Map<String, String>.from(
          jsonDecode(
                File(
                  '../../packages/presentation/locales/en-US.json',
                ).readAsStringSync(),
              )
              as Map,
        );
        var confirmed = 0;
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SingleChildScrollView(
                child: ApprovalReview(
                  preview: DisplayPreview(action, additional),
                  labels: labels,
                  onConfirm: () => confirmed++,
                  onCancel: () {},
                ),
              ),
            ),
          ),
        );
        final suffix = additional ? 'confirmAdditional' : 'confirm';
        final expected = labels['vendor.midea.dishwasher.$action.$suffix']!
            .replaceAll('{device}', 'Fictional dishwasher')
            .replaceAll('{additional}', 'One extra rinse');
        expect(find.text(expected), findsOneWidget);
        expect(confirmed, 0);
        expect(tester.takeException(), isNull);
      });
    }
  }
}
