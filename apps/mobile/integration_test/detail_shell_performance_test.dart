// Opt-in profile benchmark of the production NativeApp with fictional Core data.
// Run only on an isolated test application profile; see performance_driver.dart.
import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:homeverb_mobile/client_contract.dart';
import 'package:homeverb_mobile/controller.dart';
import 'package:homeverb_mobile/localization.dart';
import 'package:homeverb_mobile/main.dart';
import 'package:homeverb_mobile/preferences.dart';
import 'package:homeverb_mobile/screens.dart' hide Json;
import 'package:integration_test/integration_test.dart';

class _BenchmarkCore implements CoreClient {
  final device = <String, dynamic>{
    'schema_version': '1.0',
    'id': 'fictional-benchmark-device',
    'alias': 'Fictional benchmark appliance',
    'brand': 'Fictional',
    'model': 'Benchmark fixture',
    'support_status': 'mock-tested',
    'components': [
      {
        'id': 'main',
        'capabilities': ['climate'],
      },
    ],
    'connections': <Json>[],
  };
  final capability = <String, dynamic>{
    'schema_version': '1.0',
    'id': 'climate',
    'component_id': 'main',
    'revision': 1,
    'properties': {
      'current_temperature': {
        'type': 'number',
        'readable': true,
        'writable': false,
        'unit': 'celsius',
      },
    },
    'actions': <String, dynamic>{},
    'events': <String, dynamic>{},
  };
  Completer<void>? gate;
  int liveCalls = 0;
  int writes = 0;

  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async {
    switch (method) {
      case 'list_rooms':
        return <Json>[];
      case 'list_device_cards':
        return [
          {
            'device': device,
            'preferences': {'alias': null, 'room_id': null, 'favorite': false},
          },
        ];
      case 'describe_device':
        return {
          'device': device,
          'capabilities': [capability],
        };
      case 'read_state':
        final cached = fields['cached'] == true;
        if (!cached) {
          liveCalls++;
          await gate?.future;
        }
        return {
          'schema_version': '1.0',
          'device_id': device['id'],
          'component_id': 'main',
          'from_cache': cached,
          'observations': {
            'climate.current_temperature': {
              'value': 24,
              'source': 'lan',
              'observed_at': null,
              'received_at': '2026-10-14T05:00:01Z',
              'stale': true,
            },
          },
          'warnings': <String>[],
        };
      case 'execute':
      case 'plan':
        writes++;
        throw StateError('The benchmark cannot issue commands.');
      default:
        throw StateError('Unexpected fictional method: $method');
    }
  }

  @override
  void cancelAll() {}
  @override
  void foreground(bool value) {}
  @override
  Future<void> close() async {}
}

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('measure production detail shell with cached and held reads', (
    tester,
  ) async {
    expect(kProfileMode, isTrue, reason: 'Run the benchmark in profile mode.');
    final core = _BenchmarkCore();
    final c = HomeController(core, LocalPreferences.memory())
      ..startAutomaticRefresh();
    final app = GlobalKey<NativeAppState>();
    final catalog = await ScreenCatalog.load();
    await tester.pumpWidget(
      TranslationProvider(
        child: NativeApp(key: app, catalog: catalog, controller: c),
      ),
    );
    await tester.pumpAndSettle();
    final cold = <int>[];
    final cached = <int>[];
    final held = <int>[];
    final holdElapsed = <int>[];

    Future<void> sample(List<int> results, {required bool hold}) async {
      final gate = hold ? Completer<void>() : null;
      core.gate = gate;
      final previousReads = core.liveCalls;
      final watch = Stopwatch()..start();
      await tester.tap(find.text('Fictional benchmark appliance').first);
      var shellRendered = false;
      while (watch.elapsed < const Duration(seconds: 2)) {
        await tester.pump();
        if (app.currentState!.id == 'C01' &&
            find.byTooltip('Back').evaluate().isNotEmpty &&
            find.text('Fictional benchmark appliance').evaluate().isNotEmpty) {
          final painted = Completer<void>();
          WidgetsBinding.instance.addPostFrameCallback(
            (_) => painted.complete(),
          );
          await tester.pump();
          await painted.future;
          shellRendered = true;
          break;
        }
      }
      watch.stop();
      expect(
        shellRendered,
        true,
        reason: 'Detail shell did not render within two seconds.',
      );
      expect(find.byTooltip('Back'), findsOneWidget);
      expect(find.text('Fictional benchmark appliance'), findsWidgets);
      results.add(watch.elapsedMicroseconds);
      if (gate != null) {
        // The shell sample is taken before releasing a genuinely five-second
        // outstanding read. The wait is outside the measured interval.
        final holdWatch = Stopwatch()..start();
        await tester.runAsync(
          () => Future<void>.delayed(const Duration(seconds: 5)),
        );
        holdWatch.stop();
        holdElapsed.add(holdWatch.elapsedMicroseconds);
        expect(
          holdWatch.elapsed,
          greaterThanOrEqualTo(const Duration(seconds: 5)),
        );
        expect(core.liveCalls, greaterThan(previousReads));
        expect(gate.isCompleted, false);
        gate.complete();
      }
      await tester.pumpAndSettle();
      expect(c.openingDevice, false);
      await app.currentState!.go('D01', back: true);
      await tester.pumpAndSettle();
    }

    await sample(cold, hold: false);
    for (var i = 0; i < 20; i++) {
      await sample(cached, hold: false);
    }
    for (var i = 0; i < 20; i++) {
      await sample(held, hold: true);
    }
    expect(core.writes, 0);
    final view = tester.view;
    int p95(List<int> samples) {
      final sorted = [...samples]..sort();
      return sorted[(sorted.length * .95).ceil() - 1];
    }

    final report = <String, dynamic>{
      'method':
          'Stopwatch from tap through bounded visible C01 title/Back detection and a subsequent post-frame callback',
      'unit': 'microseconds',
      'cold_samples': cold,
      'warm_cached_samples': cached,
      'warm_held_five_seconds_samples': held,
      'warm_cached_p95_microseconds': p95(cached),
      'warm_held_p95_microseconds': p95(held),
      'held_read_elapsed_microseconds': holdElapsed,
      'held_read_wait_seconds': 5,
      'os': Platform.operatingSystem,
      'os_version': Platform.operatingSystemVersion,
      'physical_width': view.physicalSize.width,
      'physical_height': view.physicalSize.height,
      'device_pixel_ratio': view.devicePixelRatio,
      'live_read_calls': core.liveCalls,
      'command_calls': core.writes,
    };
    binding.reportData ??= <String, dynamic>{};
    binding.reportData!['detail_shell'] = report;
    debugPrint('DETAIL_SHELL_PROFILE ${jsonEncode(report)}');
    c.stopAutomaticRefresh();
    await tester.pumpWidget(const SizedBox());
  }, skip: !const bool.fromEnvironment('HOMEVERB_DETAIL_PERFORMANCE'));
}
