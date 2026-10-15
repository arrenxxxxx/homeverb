import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:path_provider/path_provider.dart';
import 'package:file_selector/file_selector.dart';
import 'blocks.dart';
import 'controller.dart';
import 'controls.dart';
import 'native_client.dart';
import 'preferences.dart';
import 'receipts.dart';
import 'screens.dart' hide Json;
import 'skin.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  LicenseRegistry.addLicense(() async* {
    yield LicenseEntryWithLineBreaks([
      'Inter',
    ], await rootBundle.loadString('assets/fonts/LICENSE.txt'));
  });
  try {
    final catalog = await ScreenCatalog.load();
    final directory = await getApplicationSupportDirectory();
    final prefs = LocalPreferences(File('${directory.path}/presentation.json'));
    await prefs.load();
    const library = String.fromEnvironment('HOMEVERB_LIBRARY');
    final core = await NativeClient.open(
      dataDir: '${directory.path}/native',
      libraryPath: library.isEmpty ? null : library,
      allowMockWrites: true,
    );
    runApp(
      NativeApp(catalog: catalog, controller: HomeController(core, prefs)),
    );
  } catch (e) {
    runApp(
      MaterialApp(
        theme: windowTheme(),
        home: Scaffold(
          body: LightBackground(
            child: SafeArea(
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    const Copy('HomeVerb', size: 30, bold: true),
                    const SizedBox(height: 24),
                    const Copy('Native Core unavailable', size: 23, bold: true),
                    const SizedBox(height: 16),
                    Copy('$e'),
                    const SizedBox(height: 20),
                    const Copy(
                      'Install the matching native library and restart. No device or account operation was started.',
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class NativeApp extends StatefulWidget {
  const NativeApp({required this.catalog, required this.controller, super.key});
  final ScreenCatalog catalog;
  final HomeController controller;
  @override
  State<NativeApp> createState() => NativeAppState();
}

class NativeAppState extends State<NativeApp> with WidgetsBindingObserver {
  HomeController get c => widget.controller;
  String id = 'D01';
  final List<String> history = [];
  final List<Json> receiptParents = [];
  final Map<String, TextEditingController> fields = {};
  final Map<String, int> choices = {};
  final Map<String, double> values = {};
  final navigator = GlobalKey<NavigatorState>();
  String? selectedMode;
  Json? actionCapability;
  String? actionName;
  String? formError;
  bool _disposed = false;
  int navigation = 0;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    c.addListener(changed);
    id = c.preferences.root;
    c.initialize();
  }

  void changed() {
    if (!_disposed && mounted) setState(() {});
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    c.core.foreground(state == AppLifecycleState.resumed);
  }

  @override
  void dispose() {
    _disposed = true;
    WidgetsBinding.instance.removeObserver(this);
    c.removeListener(changed);
    c.core.close();
    for (final field in fields.values) {
      field.dispose();
    }
    super.dispose();
  }

  void clearFields() {
    for (final f in fields.values) {
      f.dispose();
    }
    fields.clear();
    choices.clear();
    values.clear();
    formError = null;
  }

  Future<void> go(String target, {bool back = false}) async {
    if (!mounted) return;
    final deviceHistory = ['C01', 'D06'].contains(id);
    c.core.cancelAll();
    setState(() {
      navigation++;
      receiptParents.clear();
      if (!back) history.add(id);
      id = target;
      clearFields();
      if (target != 'D03') c.search = '';
    });
    if (['D01', 'P01', 'S01'].contains(target))
      await c.savePreferences({'root': target});
    if (target == 'S05') await c.loadHistory(deviceOnly: deviceHistory);
    if (target == 'S07') await c.loadDiagnostics();
  }

  Future<void> finish(Future<bool> work, String destination) async {
    final epoch = navigation;
    if (await work && mounted && epoch == navigation) await go(destination);
  }

  String input(String label) => fields[label]?.text ?? '';
  int choice() => choices.values.firstOrNull ?? 0;
  String get selectedRoute => ['auto', 'lan', 'cloud'][choice().clamp(0, 2)];
  Future<void> action(String label) async {
    if (label == 'Back' ||
        label == 'Cancel' ||
        label == 'Close' ||
        label == 'Not Now') {
      if (id == 'S06' && receiptParents.isNotEmpty) {
        c.receipt = receiptParents.removeLast();
        changed();
        return;
      }
      await go(history.isEmpty ? 'D01' : history.removeLast(), back: true);
      return;
    }
    if (label.startsWith('receipt:')) {
      await finish(c.loadReceipt(label.substring('receipt:'.length)), 'S06');
      return;
    }
    if (label.startsWith('receipt-item:') || label.startsWith('batch-item:')) {
      final nested = label.startsWith('receipt-item:');
      final source = nested ? c.receipt : c.batchResult;
      final index = int.tryParse(label.substring(label.indexOf(':') + 1));
      final items = source?['items'] as List? ?? [];
      if (index == null || index < 0 || index >= items.length || c.busy) return;
      if (nested) receiptParents.add(c.receipt!);
      c.receipt = items[index] as Json;
      if (nested) {
        changed();
      } else {
        await go('S06');
      }
      return;
    }
    if (label == 'Dismiss error') {
      c.error = null;
      changed();
      return;
    }
    if (label == 'Reload local data') {
      await c.initialize();
      return;
    }
    if (label.startsWith('control:')) {
      if (c.busy) return;
      final parts = label.split(':');
      final index = int.tryParse(parts[1]);
      if (index == null || index < 0 || index >= c.capabilities.length) return;
      actionCapability = c.capabilities[index];
      actionName = Uri.decodeComponent(parts[2]);
      await go('C10');
      return;
    }
    if (label.startsWith('component:')) {
      if (c.busy) return;
      clearFields();
      await c.selectComponent(label.substring('component:'.length));
      return;
    }
    if (label.startsWith('room:')) {
      final name = label.substring(5);
      c.filter = name == 'All' || name == 'Unassigned'
          ? name
          : c.rooms.firstWhere((r) => r['name'] == name)['id'];
      changed();
      return;
    }
    if (label.startsWith('open:') ||
        label.startsWith('power:') ||
        label.startsWith('menu:')) {
      final epoch = navigation,
          device = label.substring(label.indexOf(':') + 1);
      if (!await c.selectDevice(device)) return;
      if (!mounted || epoch != navigation) return;
      if (label.startsWith('power:')) {
        final state = c.observed('power.state');
        if (state is! bool) {
          c.error = NativeFailure(
            'STATE_UNAVAILABLE',
            'Refresh state before using a quick power action.',
          );
          changed();
          return;
        }
        await finish(c.execute('power', 'set', {'state': !state}), 'S06');
      } else {
        await go(label.startsWith('menu:') ? 'D05' : 'C01');
      }
      return;
    }
    if (label == 'Refresh State') {
      if (['D01', 'D11'].contains(id))
        await c.refreshCards();
      else if (id == 'S06')
        await c.refreshReceipt();
      else
        await c.refresh();
      return;
    }
    if (label == 'Discover Mock device' || label == 'Search Again') {
      await finish(c.discover(), 'D01');
      return;
    }
    if (label == 'Save Room') {
      formError = null;
      final name = input('Room name').trim();
      if (name.isEmpty ||
          name.runes.length > 80 ||
          RegExp(r'[\x00-\x1f\x7f]').hasMatch(name)) {
        setState(
          () => formError = 'Use 1–80 characters without control characters.',
        );
        return;
      }
      await finish(c.saveRoom(name, id: c.editingRoom), 'R01');
      return;
    }
    if (label == 'Delete Room' && c.editingRoom != null) {
      await finish(c.deleteRoom(c.editingRoom!), 'R01');
      return;
    }
    if (label == 'Save Name') {
      formError = null;
      if (id == 'S10') {
        final name = input('Sharing name').trim();
        if (name.isEmpty || name.length > 80) {
          setState(() => formError = 'Use 1–80 characters.');
          return;
        }
        await finish(c.savePreferences({'sharing_name': name}), 'S01');
      } else {
        await finish(
          c.saveDevice({
            'alias': input('Local name').trim().isEmpty
                ? null
                : input('Local name').trim(),
          }),
          'D06',
        );
      }
      return;
    }
    if (label == 'Pin to Top' || label == 'Unpin') {
      await finish(
        c.saveDevice({
          'favorite': !(c.currentCard?['preferences']['favorite'] ?? false),
        }),
        'D01',
      );
      return;
    }
    if (label == 'Move Devices') {
      if (c.selection.isEmpty) {
        c.error = NativeFailure(
          'INVALID_ARGUMENT',
          'Select at least one device to move.',
        );
        changed();
        return;
      }
      final index = choice();
      await finish(
        c.assign(index < c.rooms.length ? c.rooms[index]['id'] : null),
        'D01',
      );
      return;
    }
    if (label == 'Select Devices') {
      c.selection.clear();
      c.batchResult = null;
      await go('C12');
      return;
    }
    if (label == 'Apply to Selected Devices') {
      await c.executeSelectedPower(choice() == 0);
      return;
    }
    if (label == 'Save Preference') {
      final changes = id == 'D08'
          ? {
              'device_routes': {
                ...(c.preferences.values['device_routes'] as Map? ?? {}),
                c.selectedDevice!: selectedRoute,
              },
            }
          : {'route': selectedRoute};
      await finish(c.savePreferences(changes), id == 'D08' ? 'D06' : 'S01');
      return;
    }
    if (label == 'Power') {
      final state = c.observed('power.state');
      if (state is bool)
        await finish(c.execute('power', 'set', {'state': !state}), 'S06');
      else {
        c.error = NativeFailure(
          'STATE_UNAVAILABLE',
          'Read state before changing power.',
        );
        changed();
      }
      return;
    }
    if (label == 'Increase target' || label == 'Decrease target') {
      formError = null;
      final schema =
          c.capability(
                'climate',
              )?['actions']?['set']?['parameters']?['properties']?['target_temperature']
              as Json?;
      final current = c.observed('climate.target_temperature');
      if (schema == null || current is! num) {
        c.error = NativeFailure(
          'STATE_UNAVAILABLE',
          'Read a supported target first.',
        );
        changed();
        return;
      }
      final value =
          current +
          (schema['multipleOf'] ?? 1) * (label == 'Increase target' ? 1 : -1);
      final error = validateValue(value, schema);
      if (error != null) {
        setState(() => formError = error);
        return;
      }
      await finish(
        c.execute('climate', 'set', {'target_temperature': value}),
        'S06',
      );
      return;
    }
    if (label == 'Save Mode') {
      final modes =
          c.capability(
                'climate',
              )?['actions']?['set']?['parameters']?['properties']?['mode']?['enum']
              as List? ??
          [];
      if (modes.isNotEmpty)
        await finish(
          c.execute('climate', 'set', {
            'mode': modes[choice().clamp(0, modes.length - 1)],
          }),
          'S06',
        );
      return;
    }
    if (label == 'Save target') {
      formError = null;
      final value = num.tryParse(input('Target temperature'));
      final schema =
          c.capability(
                'climate',
              )?['actions']?['set']?['parameters']?['properties']?['target_temperature']
              as Json?;
      final error = schema == null
          ? 'Control unavailable.'
          : validateValue(value, schema);
      if (error != null) {
        setState(() => formError = error);
        return;
      }
      await finish(
        c.execute('climate', 'set', {'target_temperature': value}),
        'S06',
      );
      return;
    }
    if (label == 'Export Redacted Report') {
      await export();
      return;
    }
    if (label.startsWith('choice:')) return;
    if (label == 'Privacy' || label == 'Licenses') {
      if (label == 'Licenses') {
        showLicensePage(
          context: navigator.currentContext!,
          applicationName: 'HomeVerb',
          applicationVersion: '0.2.0',
        );
      } else {
        await showDialog<void>(
          context: navigator.currentContext!,
          builder: (context) => AlertDialog(
            title: const Text('Privacy'),
            content: const Text(
              'Local data only. No telemetry or automatic uploads. Diagnostics are saved locally only when you choose Export. Platform authorization and sharing are unavailable in this build.',
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context),
                child: const Text('Close'),
              ),
            ],
          ),
        );
      }
      return;
    }
    if (label == 'Add Room') {
      c.editingRoom = null;
      await go('R02');
      return;
    }
    const routes = {
      'Devices': 'D01',
      'Platforms': 'P01',
      'Settings': 'S01',
      'Search': 'D03',
      'Add device': 'A01',
      'Add Device': 'A01',
      'Manage rooms': 'R01',
      'Rooms': 'R01',
      'Appearance': 'S02',
      'This Device': 'S10',
      'Connections': 'S03',
      'Recent Operations': 'S05',
      'Local Data': 'S04',
      'Diagnostics': 'S07',
      'About': 'S08',
      'Rename': 'D07',
      'Name': 'D07',
      'Move to Room': 'R03',
      'Room': 'R03',
      'Move Existing Devices': 'R03',
      'Reorder Devices': 'D10',
      'Device Information': 'D06',
      'Device information': 'D06',
      'Connection': 'D08',
      'Mode': 'C02',
      'Target temperature': 'C14',
      'View Details': 'S06',
      'Preview Skin': 'S11',
      'Back to Appearance': 'S02',
      'Compatibility': 'S12',
      'Compatibility Details': 'S12',
      'Add Platform': 'P02',
      'Add Another Platform': 'P02',
      'Receive Authorization': 'H05',
      'Send Authorization': 'H01',
      'Back to Platforms': 'P02',
      'Text size and accessibility': 'D12',
      'Inspect cached state': 'D11',
      'Remove Local Data': 'S09',
      'View Operations': 'S05',
      'Done': 'D01',
    };
    if (routes[label] != null) {
      if (routes[label] == 'R03') {
        c.selectedRoom = c.currentCard?['preferences']?['room_id'];
        c.selection.clear();
        if (['D05', 'D06'].contains(id) && c.selectedDevice != null) {
          c.selection.add(c.selectedDevice!);
        }
      }
      await go(routes[label]!);
      return;
    }
    if (label == 'Balcony' || label == 'Nursery') {
      fields['Room name']?.text = label;
      changed();
      return;
    }
    c.error = NativeFailure(
      'UNAVAILABLE',
      'This operation is not exposed by native contract 1.0.',
    );
    changed();
  }

  Future<void> export() async {
    if (c.diagnostics == null) return;
    final exact = const JsonEncoder.withIndent('  ').convert(c.diagnostics);
    try {
      final location = await getSaveLocation(
        suggestedName: 'homeverb-diagnostics.json',
        acceptedTypeGroups: [
          const XTypeGroup(label: 'JSON', extensions: ['json']),
        ],
      );
      if (location == null) return;
      await XFile.fromData(
        utf8.encode(exact),
        mimeType: 'application/json',
        name: 'homeverb-diagnostics.json',
      ).saveTo(location.path);
    } catch (_) {
      c.error = NativeFailure(
        'EXPORT_FAILED',
        'Could not save the previewed report.',
      );
      changed();
    }
  }

  Json get screen => screenFor(id);
  Json screenFor(String id) {
    final original = widget.catalog.ui(id);
    final ui = Map<String, dynamic>.from(original);
    List<dynamic>? b;
    if (id.startsWith('P') || id.startsWith('H')) {
      b = [
        [
          'status',
          'Unavailable',
          id.startsWith('P')
              ? 'Platform authorization is not available in native contract 1.0.'
              : 'Nearby sharing is not available in native contract 1.0.',
          'warning',
        ],
        [
          'note',
          'No account is connected and no discovery, transfer or authorization has started.',
        ],
        ['button', 'Compatibility Details', 'secondary'],
      ];
    }
    final card = c.currentCard,
        device = card?['device'] as Json?,
        prefs = card?['preferences'] as Json?;
    switch (id) {
      case 'D01':
      case 'D11':
      case 'D12':
        if (c.cards.isEmpty) {
          b = c.busy
              ? [
                  ['note', 'Loading local devices…'],
                ]
              : c.error != null
              ? [
                  ['note', 'The local device list could not be loaded.'],
                  ['button', 'Reload local data', 'secondary'],
                ]
              : widget.catalog.ui('D04')['blocks'] as List;
        }
        break;
      case 'D03':
        b = [
          ['field', 'Search devices', c.search],
          ['note', 'Searches local device names, models and rooms.'],
        ];
        break;
      case 'D05':
        ui['header'] = device?['alias'] ?? 'Device';
        b = [
          [
            'rows',
            [
              ['Rename', '›'],
              ['Move to Room', '›'],
              [prefs?['favorite'] == true ? 'Unpin' : 'Pin to Top', '›'],
              ['Reorder Devices', '›'],
              ['Select Devices', '›'],
              ['Device Information', '›'],
            ],
          ],
          ['button', 'Close', 'secondary'],
        ];
        break;
      case 'D06':
        b = [
          [
            'hero',
            'ac',
            device?['alias'] ?? 'Device',
            c.roomName(prefs?['room_id']),
          ],
          [
            'rows',
            [
              ['Name', device?['alias'] ?? '—'],
              ['Room', c.roomName(prefs?['room_id'])],
              ['Model', device?['model'] ?? '—'],
              ['Connection', c.preferences.route(c.selectedDevice)],
            ],
          ],
          [
            'note',
            'Supported: ${c.capabilities.map((v) => v['id']).join(' · ')}',
          ],
          ['button', 'Recent Operations', 'secondary'],
        ];
        break;
      case 'D07':
        b = [
          ['field', 'Local name', prefs?['alias'] ?? ''],
          ['note', 'A blank name restores the adapter name.'],
          ['button', 'Save Name'],
          ['button', 'Cancel', 'secondary'],
        ];
        break;
      case 'D08':
      case 'S03':
        final connections = (device?['connections'] as List? ?? [])
            .map((v) => v['transport'])
            .toSet();
        b = [
          [
            'choices',
            [
              'Auto — prefer local when available',
              'Local network only — no cloud fallback',
              'Platform cloud — requires internet',
            ],
            ['auto', 'lan', 'cloud'].indexOf(
              c.preferences.route(id == 'D08' ? c.selectedDevice : null),
            ),
          ],
          [
            'note',
            id == 'D08'
                ? 'Available routes: ${connections.join(', ')}. Core validates every selected route.'
                : 'Device-specific preferences take priority. No background scanning.',
          ],
          ['button', 'Save Preference'],
        ];
        break;
      case 'R02':
      case 'R05':
        b = [
          [
            'field',
            'Room name',
            c.rooms
                    .where((r) => r['id'] == c.editingRoom)
                    .firstOrNull?['name'] ??
                '',
          ],
          [
            'chips',
            ['Balcony', 'Nursery'],
          ],
          ['button', 'Save Room'],
          ['button', 'Cancel', 'secondary'],
        ];
        break;
      case 'R04':
        ui['header'] = 'Delete ${c.roomName(c.editingRoom)}?';
        break;
      case 'R03':
        b = [
          [
            'note',
            'Select devices and a destination room. Access stays unchanged.',
          ],
          [
            'choices',
            [...c.rooms.map((r) => r['name']), 'Unassigned'],
            c.selectedRoom == null
                ? 0
                : c.rooms
                      .indexWhere((room) => room['id'] == c.selectedRoom)
                      .clamp(0, c.rooms.length),
          ],
          ['button', 'Move Devices'],
          ['button', 'Add Room', 'secondary'],
        ];
        break;
      case 'C01':
        ui['header'] = device?['alias'] ?? 'Device';
        ui['subtitle'] = c.roomName(prefs?['room_id']);
        final temperature = c.observed('climate.target_temperature');
        final targetAction = c.capability('climate')?['actions']?['set'];
        final power = c.capability('power');
        final powerAction = power?['actions']?['set'] as Json?;
        final canEditPower =
            powerAction != null &&
            actionUnavailable(power!, powerAction) == null &&
            powerAction['parameters']?['properties']?['state'] != null;

        final canEditTarget =
            targetAction?['risk'] == 'low' &&
            targetAction?['parameters']?['properties']?['target_temperature'] !=
                null;
        values['observed_target'] = temperature is num
            ? temperature.toDouble()
            : double.nan;
        fields
                .putIfAbsent('observed_mode', () => TextEditingController())
                .text =
            '${c.observed('climate.mode') ?? 'Not available'}';
        b = [
          if (c.components.length > 1)
            [
              'tabs',
              c.components.map((part) => '${part['id']}').toList(),
              c.components.indexWhere((part) => part['id'] == c.component),
              c.components.map((part) => 'component:${part['id']}').toList(),
            ],
          if (c.capability('climate') != null)
            [
              'climate',
              c.snapshot == null
                  ? 'No reading'
                  : '${c.snapshot!['from_cache'] == true ? 'Cached' : 'Read'} observation',
              canEditTarget,
            ],
          if (c.capability('climate') != null && !canEditTarget)
            ['note', 'Temperature is read-only here.'],
          if (power != null ||
              targetAction?['parameters']?['properties']?['mode'] != null ||
              c.capability(
                    'climate',
                  )?['properties']?['current_temperature']?['readable'] ==
                  true)
            [
              'rows',
              [
                if (c.capability('power') != null)
                  [
                    'Power',
                    c.observed('power.state') == null
                        ? 'Unknown'
                        : c.observed('power.state') == true
                        ? 'On'
                        : 'Off',
                    canEditPower ? 'Power' : null,
                  ],
                if (c.capability(
                      'climate',
                    )?['actions']?['set']?['parameters']?['properties']?['mode'] !=
                    null)
                  [
                    'Mode',
                    '${c.observed('climate.mode') ?? 'Unknown'}  ›',
                    targetAction?['risk'] == 'low' ? 'Mode' : null,
                  ],
                if (c.capability(
                      'climate',
                    )?['properties']?['current_temperature']?['readable'] ==
                    true)
                  [
                    'Current temperature',
                    propertyValue(
                      c.observed('climate.current_temperature'),
                      c.capability(
                        'climate',
                      )!['properties']['current_temperature'],
                    ),
                    null,
                  ],
              ],
            ],
          ...describedControls(c),
          ['button', 'Refresh State', 'secondary'],
          ['button', 'Device information', 'secondary'],
          ['button', 'Recent Operations', 'secondary'],
        ];
        break;
      case 'C02':
        b = [
          [
            'choices',
            c.capability(
                  'climate',
                )?['actions']?['set']?['parameters']?['properties']?['mode']?['enum'] ??
                [],
            0,
          ],
          ['button', 'Save Mode'],
          ['button', 'Cancel', 'secondary'],
        ];
        break;
      case 'C10':
        ui['base'] = 'C01';
        ui['overlay'] = 'sheet';
        ui['header'] = actionName == null
            ? 'Action unavailable'
            : '${controlLabel(actionCapability!['id'])} · ${controlLabel(actionName!)}';
        b = [
          ['note', 'Choose a supported action from the device page.'],
        ];
        break;
      case 'C12':
        ui['header'] = c.batchResult == null
            ? 'Select devices'
            : 'Operation results';
        b = c.batchResult == null
            ? [
                [
                  'note',
                  'Choose exact devices and a power action. Each outcome is recorded separately.',
                ],
                [
                  'choices',
                  ['Turn on', 'Turn off'],
                  0,
                ],
                ['button', 'Apply to Selected Devices'],
                ['button', 'Cancel', 'secondary'],
              ]
            : [
                [
                  'status',
                  outcomeLabel(c.batchResult!['status']),
                  'Inspect each item. Completed actions are not rolled back.',
                  outcomeColor(c.batchResult!['status']),
                ],
                [
                  'rows',
                  [
                    for (
                      var i = 0;
                      i < (c.batchResult!['items'] as List).length;
                      i++
                    )
                      [
                        receiptDeviceName(c.batchResult!['items'][i]),
                        outcomeLabel(c.batchResult!['items'][i]['status']),
                        'batch-item:$i',
                      ],
                  ],
                ],
                ['button', 'View Operations', 'secondary'],
                ['button', 'Done'],
              ];
        break;
      case 'C14':
        b = [
          [
            'note',
            'Observed target remains ${c.observed('climate.target_temperature') ?? 'unknown'}.',
          ],
          [
            'field',
            'Target temperature',
            '${c.observed('climate.target_temperature') ?? ''}',
          ],
          [
            'note',
            '${c.capability('climate')?['actions']?['set']?['parameters']?['properties']?['target_temperature']}',
          ],
          ['button', 'Save target'],
          ['button', 'Cancel', 'secondary'],
        ];
        break;
      case 'A01':
      case 'A02':
      case 'A03':
      case 'A04':
      case 'A06':
      case 'A07':
        b = [
          [
            'status',
            'Mock discovery',
            'Only a fictional appliance can be discovered by this host.',
            'neutral',
          ],
          ['button', 'Discover Mock device'],
          ['button', 'Add Platform', 'secondary'],
          ['note', 'No physical appliance or local network is scanned.'],
        ];
        break;
      case 'A05':
        b = [
          [
            'status',
            'Identity merge unavailable',
            'Names alone do not prove identity. No native merge method exists.',
            'warning',
          ],
        ];
        break;
      case 'S04':
      case 'S09':
        b = [
          [
            'status',
            'Cleanup unavailable',
            'Native contract 1.0 has no safe cleanup methods. Execution journals and replay safeguards remain intact.',
            'warning',
          ],
          ['button', 'Back', 'secondary'],
        ];
        break;
      case 'S10':
        b = [
          ['field', 'Sharing name', c.preferences.sharingName],
          ['note', 'Shown only during an explicit nearby sharing session.'],
          ['button', 'Save Name'],
        ];
        break;
      case 'S06':
        final r = c.receipt;
        b = r == null
            ? [
                ['note', 'No retained receipt selected.'],
              ]
            : receiptBlocks(r, receiptDeviceName(r));
        break;
      case 'S07':
        b = [
          [
            'note',
            c.diagnostics == null
                ? 'No report loaded.'
                : const JsonEncoder.withIndent('  ').convert(c.diagnostics),
          ],
          ['button', 'Export Redacted Report'],
          ['button', 'Cancel', 'secondary'],
        ];
        break;
      case 'S08':
        b = [
          ['hero', 'home', 'HomeVerb', 'One language for your appliances.'],
          ['note', '0.2.0 · Native Mock host · No real-device support'],
          [
            'rows',
            [
              ['Compatibility', '›'],
              ['Privacy', '›'],
              ['Licenses', '›'],
            ],
          ],
        ];
        break;
      case 'S12':
        b = [
          [
            'status',
            'Mock tested',
            'Linux native contract 1.0. Real platforms and mobile secure stores remain unavailable.',
            'neutral',
          ],
          [
            'note',
            'Support requires model, region and physical-device evidence.',
          ],
        ];
        break;
    }
    if (b != null) ui['blocks'] = b;
    if (formError != null && id == this.id)
      ui['blocks'] = [
        ['status', 'Check this value', formError, 'error'],
        ...ui['blocks'] as List,
      ];
    return ui;
  }

  Widget deviceCards() => DeviceGrid(
    keys: c.visibleCards.map((v) => v['device']['id'] as String).toList(),
    data: c.visibleCards.map((v) {
      final d = v['device'] as Json,
          p = v['preferences'] as Json,
          s = c.snapshots[d['id']];
      return [
        '${d['alias']}',
        c.roomName(p['room_id']),
        s?['observations']?['climate.target_temperature']?['value'] == null
            ? '—'
            : '${s!['observations']['climate.target_temperature']['value']}°C',
        s == null ? 'Not read' : 'Saved observation',
      ];
    }).toList(),
    onAction: action,
  );
  Widget? special() => specialFor(id);
  String roomDeviceCount(String? room) {
    final count = c.cards
        .where((card) => card['preferences']['room_id'] == room)
        .length;
    return '$count ${count == 1 ? 'device' : 'devices'}';
  }

  Widget? specialFor(String id) {
    if (['D01', 'D11', 'D12'].contains(id) && c.cards.isNotEmpty)
      return DeviceHome(
        mode: 'all',
        onAction: action,
        cards: deviceCards(),
        roomNames: [
          'All',
          ...c.rooms.map((v) => v['name'] as String),
          'Unassigned',
        ],
        filter: ['All', 'Unassigned'].contains(c.filter)
            ? c.filter
            : c.roomName(c.filter),
      );
    if (id == 'C10' && actionCapability != null && actionName != null) {
      final epoch = navigation;
      return CapabilityActionForm(
        key: ValueKey('$epoch:${actionCapability!['id']}:$actionName'),
        capability: actionCapability!,
        action: actionName!,
        observation: (property) =>
            c.snapshot?['observations']?[property] as Json?,
        error: () => c.error,
        onCancel: () => action('Cancel'),
        onSubmit: (arguments) async {
          final success = await c.execute(
            actionCapability!['id'],
            actionName!,
            arguments,
          );
          if (success && mounted && epoch == navigation) await go('S06');
          return success;
        },
      );
    }
    if (id == 'D03')
      return Column(
        children: [
          Blocks(
            screenFor(id)['blocks'] as List,
            onAction: action,
            fields: fields,
            choices: choices,
            values: values,
            onChanged: () {
              c.search = input('Search devices');
              changed();
            },
          ),
          const SizedBox(height: 12),
          deviceCards(),
        ],
      );
    if (id == 'R01')
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (c.rooms.isNotEmpty)
            Surface(
              radius: 18,
              child: ReorderableListView.builder(
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                padding: EdgeInsets.zero,
                buildDefaultDragHandles: false,
                itemCount: c.rooms.length,
                onReorderItem: (from, to) async {
                  if (c.busy) return;
                  final ids = c.rooms
                      .map((room) => room['id'] as String)
                      .toList();
                  ids.insert(to, ids.removeAt(from));
                  await c.reorderRooms(ids);
                },
                itemBuilder: (context, index) {
                  final r = c.rooms[index];
                  return Column(
                    key: ValueKey('room:${r['id']}'),
                    children: [
                      if (index > 0)
                        const Padding(
                          padding: EdgeInsets.symmetric(horizontal: 16),
                          child: Divider(
                            height: 0,
                            thickness: .6,
                            color: Color(0xffdeded7),
                          ),
                        ),
                      Padding(
                        padding: const EdgeInsets.only(left: 16, right: 6),
                        child: Row(
                          children: [
                            Expanded(child: Copy(r['name'])),
                            const SizedBox(width: 10),
                            Copy(
                              roomDeviceCount(r['id']),
                              size: 12,
                              color: muted,
                            ),
                            ReorderableDragStartListener(
                              index: index,
                              child: SizedBox(
                                width: 34,
                                height: 44,
                                child: PopupMenuButton<String>(
                                  tooltip: 'Room actions ${r['name']}',
                                  padding: EdgeInsets.zero,
                                  icon: const Glyph(
                                    'drag',
                                    size: 16,
                                    color: muted,
                                  ),
                                  onSelected: (v) async {
                                    c.editingRoom = r['id'];
                                    if (v == 'Rename')
                                      await go('R05');
                                    else if (v == 'Delete')
                                      await go('R04');
                                    else if (v == 'Move devices') {
                                      c.selection.clear();
                                      c.selectedRoom = r['id'];
                                      await go('R03');
                                    } else {
                                      final ids = c.rooms
                                              .map((e) => e['id'] as String)
                                              .toList(),
                                          i = c.rooms.indexOf(r),
                                          j = v == 'Move up' ? i - 1 : i + 1;
                                      if (j >= 0 && j < ids.length) {
                                        ids.removeAt(i);
                                        ids.insert(j, r['id']);
                                        await c.reorderRooms(ids);
                                      }
                                    }
                                  },
                                  itemBuilder: (_) =>
                                      [
                                            'Rename',
                                            'Delete',
                                            'Move devices',
                                            'Move up',
                                            'Move down',
                                          ]
                                          .map(
                                            (v) => PopupMenuItem(
                                              value: v,
                                              child: Text(v),
                                            ),
                                          )
                                          .toList(),
                                ),
                              ),
                            ),
                          ],
                        ),
                      ),
                    ],
                  );
                },
              ),
            ),
          const SizedBox(height: 12),
          Rows(
            [
              ['Unassigned', roomDeviceCount(null)],
            ],
            onAction: (_) async {
              c.filter = 'Unassigned';
              await go('D01');
            },
          ),
          ActionButton('Add Room', onPressed: () => action('Add Room')),
          const Copy(
            'Drag to reorder. Use a row menu to rename or delete.',
            color: muted,
          ),
        ],
      );
    if (id == 'R03' || (id == 'C12' && c.batchResult == null))
      return Column(
        children: [
          for (final card in c.cards)
            CheckboxListTile(
              title: Text(card['device']['alias']),
              value: c.selection.contains(card['device']['id']),
              onChanged: (v) {
                setState(() {
                  if (v == true)
                    c.selection.add(card['device']['id']);
                  else
                    c.selection.remove(card['device']['id']);
                });
              },
            ),
          Blocks(
            screenFor(id)['blocks'] as List,
            onAction: action,
            fields: fields,
            choices: choices,
            values: values,
            onChanged: changed,
          ),
        ],
      );
    if (id == 'D10')
      return Column(
        children: [
          for (final card in c.visibleCards)
            ListTile(
              title: Text(card['device']['alias']),
              trailing: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  for (final offset in [-1, 1])
                    IconButton(
                      tooltip: offset < 0 ? 'Move up' : 'Move down',
                      icon: Icon(
                        offset < 0 ? Icons.arrow_upward : Icons.arrow_downward,
                      ),
                      onPressed: () async {
                        final ids = c.visibleCards
                                .map((v) => v['device']['id'] as String)
                                .toList(),
                            index = c.visibleCards.indexOf(card),
                            target = index + offset;
                        if (target >= 0 && target < ids.length) {
                          ids.removeAt(index);
                          ids.insert(target, card['device']['id']);
                          await c.reorderDevices(ids);
                        }
                      },
                    ),
                ],
              ),
            ),
          ActionButton('Done', onPressed: () => go('D01')),
        ],
      );
    if (id == 'S05')
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (c.history.isEmpty)
            const Copy(
              'No retained operations. This does not prove that no past action occurred.',
            )
          else
            Rows([
              for (final receipt in c.history)
                [
                  receiptDeviceName(receipt),
                  '${outcomeLabel(receipt['status'])} · ${receiptTime(receipt['completed_at'], compact: true)}',
                  'receipt:${receipt['request_id']}',
                ],
            ], onAction: action),
          const Copy(
            'Select an operation to inspect its retained evidence. Sharing history is unavailable.',
            color: muted,
          ),
        ],
      );
    return null;
  }

  String receiptDeviceName(Json receipt) {
    for (final card in c.cards) {
      if (card['device']['id'] == receipt['device_id']) {
        return card['preferences']['alias'] ?? card['device']['alias'];
      }
    }
    return 'Retained device';
  }

  @override
  Widget build(BuildContext context) => MaterialApp(
    navigatorKey: navigator,
    debugShowCheckedModeBanner: false,
    theme: windowTheme(),
    home: PopScope(
      canPop: false,
      onPopInvokedWithResult: (popped, _) {
        if (!popped) action('Back');
      },
      child: Scaffold(
        body: ScreenView(
          key: id == 'S06' ? ObjectKey(c.receipt) : null,
          id: id,
          catalog: widget.catalog,
          screenOverride: screen,
          home: special(),
          resolveScreen: screenFor,
          resolveHome: specialFor,
          onAction: action,
          fields: fields,
          choices: choices,
          values: values,
          onChanged: changed,
          busy: c.busy,
          error: id == 'C10' && actionCapability != null ? null : c.error,
        ),
      ),
    ),
  );
}
