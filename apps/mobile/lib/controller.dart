import 'package:flutter/foundation.dart';
import 'native_client.dart';
import 'preferences.dart';

class HomeController extends ChangeNotifier {
  HomeController(this.core, this.preferences);
  final CoreClient core;
  final LocalPreferences preferences;
  List<Json> rooms = [], cards = [], history = [];
  Json? description, snapshot, receipt, diagnostics;
  final Map<String, Json> snapshots = {};
  String? selectedDevice, selectedRoom, editingRoom;
  String component = 'main', filter = 'All', search = '';
  final Set<String> selection = {};
  bool busy = false;
  Object? error;

  Future<bool> perform(Future<void> Function() work) async {
    if (busy) return false;
    busy = true;
    error = null;
    notifyListeners();
    try {
      await work();
      return true;
    } catch (e) {
      error = e;
      return false;
    } finally {
      busy = false;
      notifyListeners();
    }
  }

  Future<void> loadLocal() async {
    final results = await Future.wait([
      core.call('list_rooms'),
      core.call('list_device_cards'),
    ]);
    rooms = (results[0] as List).cast<Json>();
    cards = (results[1] as List).cast<Json>();
  }

  Future<bool> initialize() => perform(loadLocal);
  Future<bool> discover() => perform(() async {
    await core.call('discover');
    await loadLocal();
  });
  String roomName(String? id) =>
      rooms
          .where((r) => r['id'] == id)
          .map((r) => r['name'] as String)
          .firstOrNull ??
      'Unassigned';
  List<Json> get visibleCards => cards.where((c) {
    final device = c['device'] as Json, prefs = c['preferences'] as Json;
    return (filter == 'All' ||
            prefs['room_id'] == filter ||
            (filter == 'Unassigned' && prefs['room_id'] == null)) &&
        '${device['alias']} ${device['model']} ${roomName(prefs['room_id'])}'
            .toLowerCase()
            .contains(search.toLowerCase());
  }).toList();
  Json? get currentCard =>
      cards.where((c) => c['device']['id'] == selectedDevice).firstOrNull;
  dynamic observed(String property) =>
      snapshot?['observations']?[property]?['value'];
  List<Json> get capabilities => ((description?['capabilities'] as List?) ?? [])
      .cast<Json>()
      .where((c) => c['component_id'] == null || c['component_id'] == component)
      .toList();
  Json? capability(String id) =>
      capabilities.where((c) => c['id'] == id).firstOrNull;
  Future<bool> selectDevice(String id) => perform(() async {
    selectedDevice = id;
    description = await core.call('describe_device', {'device': id}) as Json;
    component = description!['device']['components'][0]['id'] as String;
    snapshot = snapshots[id];
    try {
      await read(cached: true);
    } on NativeFailure catch (e) {
      if (!['STATE_UNAVAILABLE', 'CACHE_MISS'].contains(e.code)) rethrow;
    }
  });
  Future<void> read({bool cached = false}) async {
    final next =
        await core.call('read_state', {
              'device': selectedDevice,
              'component': component,
              'connection': preferences.route(selectedDevice),
              'cached': cached,
            })
            as Json;
    snapshot = next;
    snapshots[selectedDevice!] = next;
  }

  Future<bool> refresh() => perform(() => read());
  Future<bool> refreshCards() => perform(() async {
    for (final c in visibleCards) {
      final d = c['device'] as Json;
      snapshots[d['id']] =
          await core.call('read_state', {
                'device': d['id'],
                'component': d['components'][0]['id'],
                'connection': preferences.route(d['id']),
                'cached': false,
              })
              as Json;
    }
  });
  Future<bool> saveRoom(String name, {String? id}) => perform(() async {
    await core.call(id == null ? 'create_room' : 'rename_room', {
      'name': name.trim(),
      if (id != null) 'room_id': id,
    });
    await loadLocal();
  });
  Future<bool> deleteRoom(String id) => perform(() async {
    await core.call('delete_room', {'room_id': id});
    if (filter == id) filter = 'Unassigned';
    await loadLocal();
  });
  Future<bool> assign(String? room) => perform(() async {
    await core.call('assign_devices_to_room', {
      'assignment': {
        'device_ids': selection.isEmpty ? [selectedDevice] : selection.toList(),
        'room_id': room,
      },
    });
    await loadLocal();
    selection.clear();
  });
  Future<bool> saveDevice(Json changes) => perform(() async {
    final p = currentCard!['preferences'] as Json;
    await core.call('set_device_preferences', {
      'device_id': selectedDevice,
      'preferences': {
        'alias': p['alias'],
        'room_id': p['room_id'],
        'favorite': p['favorite'],
        ...changes,
      },
    });
    await loadLocal();
  });
  Future<bool> reorderRooms(List<String> ids) => perform(() async {
    await core.call('reorder_rooms', {'room_ids': ids});
    await loadLocal();
  });
  Future<bool> reorderDevices(List<String> visible) => perform(() async {
    final selected = visible.toSet();
    var index = 0;
    final ids = cards
        .map((c) => c['device']['id'] as String)
        .map((id) => selected.contains(id) ? visible[index++] : id)
        .toList();
    await core.call('reorder_devices', {'device_ids': ids});
    await loadLocal();
  });
  Future<bool> execute(
    String capabilityId,
    String action,
    Json arguments,
  ) => perform(() async {
    final descriptor = capability(capabilityId)?['actions']?[action] as Json?;
    if (descriptor == null)
      throw NativeFailure(
        'CAPABILITY_UNSUPPORTED',
        'This action is not available.',
      );
    if (descriptor['risk'] != 'low') {
      throw NativeFailure(
        'APPROVAL_UNAVAILABLE',
        'Trusted approval is not available in this host.',
      );
    }
    final command = {
      'schema_version': '1.0',
      'request_id': uniqueId(),
      'device_id': selectedDevice,
      'component_id': component,
      'capability': capabilityId,
      'action': action,
      'arguments': arguments,
      'connection': preferences.route(selectedDevice),
      'timeout_ms': 10000,
      'wait_for': 'verified',
    };
    await core.call('plan', {'command': command});
    receipt = await core.call('execute', {'command': command}) as Json;
    // Observations only change after an explicit read, never from a requested value.
  });
  Future<bool> loadHistory({bool deviceOnly = false}) => perform(() async {
    history =
        (await core.call(
                  deviceOnly ? 'device_executions' : 'recent_executions',
                  deviceOnly ? {'device_id': selectedDevice} : {},
                )
                as List)
            .cast<Json>();
  });
  Future<bool> loadReceipt(String id) => perform(() async {
    receipt = await core.call('get_execution', {'request_id': id}) as Json;
  });
  Future<bool> loadDiagnostics() => perform(() async {
    diagnostics = await core.call('diagnostics') as Json;
  });
  Future<bool> savePreferences(Json changes) =>
      perform(() => preferences.save(changes));
}

String? validateValue(dynamic value, Json schema) {
  if (schema['enum'] is List && !(schema['enum'] as List).contains(value))
    return 'Choose a supported value.';
  if (schema['type'] == 'boolean' && value is! bool) return 'Choose On or Off.';
  if (schema['type'] == 'number' || schema['type'] == 'integer') {
    if (value is! num || !value.isFinite) return 'Enter a number.';
    if (schema['type'] == 'integer' && value != value.roundToDouble())
      return 'Use a whole number.';
    if (schema['minimum'] is num && value < schema['minimum'])
      return 'Minimum: ${schema['minimum']}';
    if (schema['maximum'] is num && value > schema['maximum'])
      return 'Maximum: ${schema['maximum']}';
    if (schema['multipleOf'] case final num step) {
      if ((value / step - (value / step).round()).abs() > 1e-8)
        return 'Supported step: $step';
    }
  }
  return null;
}
