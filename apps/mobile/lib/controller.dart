import 'package:flutter/foundation.dart';
import 'native_client.dart';
import 'preferences.dart';

class HomeController extends ChangeNotifier {
  HomeController(this.core, this.preferences);
  final CoreClient core;
  final LocalPreferences preferences;
  List<Json> rooms = [], cards = [], history = [];
  Json? description, snapshot, receipt, diagnostics, batchResult;
  final Map<String, Json> snapshots = {};
  final Map<(String, String), Json> _componentSnapshots = {};
  final Map<(String, String), Object> _componentReadErrors = {};
  String? selectedDevice, selectedRoom, editingRoom;
  String component = 'main', filter = 'All', search = '';
  final Set<String> selection = {};
  int _readEpoch = 0;
  void cancelPending() {
    _readEpoch++;
    core.cancelAll();
  }

  void setForeground(bool foreground) {
    if (!foreground) _readEpoch++;
    core.foreground(foreground);
  }

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
  List<Json> get components =>
      ((description?['device']?['components'] as List?) ?? []).cast<Json>();
  Json? get selectedComponent =>
      components.where((part) => part['id'] == component).firstOrNull;
  List<Json> capabilitiesFor(String componentId) {
    final part = components
        .where((part) => part['id'] == componentId)
        .firstOrNull;
    return ((description?['capabilities'] as List?) ?? [])
        .cast<Json>()
        .where(
          (capability) =>
              (capability['component_id'] == null ||
                  capability['component_id'] == componentId) &&
              (part?['capabilities'] as List? ?? []).contains(capability['id']),
        )
        .toList();
  }

  List<Json> get capabilities => capabilitiesFor(component);
  Json? componentCapability(String componentId, String id) => capabilitiesFor(
    componentId,
  ).where((capability) => capability['id'] == id).firstOrNull;
  Json? capability(String id) => componentCapability(component, id);
  Json? componentSnapshot(String id) =>
      _componentSnapshots[(selectedDevice, id)];
  Object? componentReadError(String id) =>
      _componentReadErrors[(selectedDevice, id)];
  bool get hasCompartmentOverview =>
      components
          .where((part) => componentCapability(part['id'], 'climate') != null)
          .length >
      1;
  Future<bool> selectDevice(String id) => perform(() async {
    selectedDevice = id;
    description = null;
    snapshot = null;
    description = await core.call('describe_device', {'device': id}) as Json;
    component = components.first['id'] as String;
    await _loadComponentCache();
    if (hasCompartmentOverview) {
      for (final part in components.where((part) => part['id'] != component)) {
        try {
          await readComponent(part['id'], cached: true);
        } on NativeFailure catch (error) {
          if (!['STATE_UNAVAILABLE', 'CACHE_MISS'].contains(error.code))
            rethrow;
        }
      }
    }
  });
  Future<bool> selectComponent(String id) => perform(() async {
    if (!components.any((part) => part['id'] == id)) {
      throw NativeFailure('INVALID_ARGUMENT', 'Select an available component.');
    }
    component = id;
    await _loadComponentCache();
  });
  Future<void> _loadComponentCache() async {
    snapshot = _componentSnapshots[(selectedDevice!, component)];
    try {
      await read(cached: true);
    } on NativeFailure catch (e) {
      if (!['STATE_UNAVAILABLE', 'CACHE_MISS'].contains(e.code)) rethrow;
    }
  }

  Future<void> read({bool cached = false}) =>
      readComponent(component, cached: cached);
  Future<void> readComponent(String componentId, {bool cached = false}) async {
    final device = selectedDevice!, epoch = _readEpoch;
    final next =
        await core.call('read_state', {
              'device': device,
              'component': componentId,
              'connection': preferences.route(device),
              'cached': cached,
            })
            as Json;
    if (epoch != _readEpoch) {
      throw NativeFailure('CANCELLED', 'Component reading was cancelled.');
    }
    _componentSnapshots[(device, componentId)] = next;
    _componentReadErrors.remove((device, componentId));
    if (componentId == component) snapshot = next;
    if (componentId == components.first['id']) snapshots[device] = next;
  }

  Future<bool> refresh() => perform(() => read());
  Future<bool> refreshComponents() => perform(() async {
    Object? failure;
    for (final part in components) {
      final id = part['id'] as String;
      try {
        await readComponent(id);
      } catch (error) {
        if (error is NativeFailure && error.code == 'CANCELLED') rethrow;
        _componentReadErrors[(selectedDevice!, id)] = error;
        failure ??= error;
      }
    }
    if (failure != null) throw failure;
  });
  Future<bool> refreshReceipt() => perform(() async {
    final device = receipt?['device_id'] as String?;
    if (device == null) {
      throw NativeFailure(
        'STATE_UNAVAILABLE',
        'Select a device receipt to refresh.',
      );
    }
    final descriptor =
        await core.call('describe_device', {'device': device}) as Json;
    for (final part in descriptor['device']['components'] as List) {
      final result =
          await core.call('read_state', {
                'device': device,
                'component': part['id'],
                'connection': preferences.route(device),
                'cached': false,
              })
              as Json;
      _componentSnapshots[(device, part['id'] as String)] = result;
      _componentReadErrors.remove((device, part['id'] as String));
      if (part['id'] == descriptor['device']['components'][0]['id']) {
        snapshots[device] = result;
      }
      if (selectedDevice == device && component == part['id'])
        snapshot = result;
    }
  });
  Future<bool> refreshCards() => perform(() async {
    for (final c in visibleCards) {
      final d = c['device'] as Json;
      final result =
          await core.call('read_state', {
                'device': d['id'],
                'component': d['components'][0]['id'],
                'connection': preferences.route(d['id']),
                'cached': false,
              })
              as Json;
      snapshots[d['id']] = result;
      _componentReadErrors.remove((
        d['id'] as String,
        d['components'][0]['id'] as String,
      ));
      _componentSnapshots[(
            d['id'] as String,
            d['components'][0]['id'] as String,
          )] =
          result;
      if (selectedDevice == d['id'] && component == d['components'][0]['id']) {
        snapshot = result;
      }
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
    await reorder('reorder_rooms', {'room_ids': ids});
  });
  Future<bool> reorderDevices(List<String> visible) => perform(() async {
    final selected = visible.toSet();
    final expected = visibleCards.map((card) => card['device']['id']).toSet();
    if (selected.length != visible.length ||
        selected.length != expected.length ||
        !selected.containsAll(expected)) {
      throw NativeFailure(
        'INVALID_ARGUMENT',
        'Include every visible device exactly once.',
      );
    }
    var index = 0;
    final ids = cards
        .map((c) => c['device']['id'] as String)
        .map((id) => selected.contains(id) ? visible[index++] : id)
        .toList();
    await reorder('reorder_devices', {'device_ids': ids});
  });

  Future<void> reorder(String method, Json arguments) async {
    try {
      await core.call(method, arguments);
    } on NativeFailure catch (error) {
      if (error.code != 'INVALID_ARGUMENT') rethrow;
      await loadLocal();
      throw NativeFailure(
        'INVALID_ARGUMENT',
        'The list changed. Review the refreshed order and try again.',
      );
    }
    await loadLocal();
  }

  Future<bool> execute(
    String capabilityId,
    String action,
    Json arguments, {
    String? componentId,
  }) => perform(() async {
    final target = componentId ?? component;
    final descriptor =
        componentCapability(target, capabilityId)?['actions']?[action] as Json?;
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
      'component_id': target,
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
  Future<bool> executeSelectedPower(bool on) => perform(() async {
    if (selection.isEmpty || selection.length > 32) {
      throw NativeFailure('INVALID_ARGUMENT', 'Select 1–32 distinct devices.');
    }
    final commands = <Json>[];
    for (final device in selection) {
      final descriptor =
          await core.call('describe_device', {'device': device}) as Json;
      final candidates = <Json>[];
      for (final part in descriptor['device']['components'] as List) {
        for (final capability in descriptor['capabilities'] as List) {
          if (capability['id'] == 'power' &&
              (capability['component_id'] == null ||
                  capability['component_id'] == part['id']) &&
              capability['actions']?['set']?['risk'] == 'low' &&
              capability['actions']?['set']?['parameters']?['properties']?['state']?['type'] ==
                  'boolean') {
            candidates.add({'component': part['id']});
          }
        }
      }
      if (candidates.length != 1) {
        throw NativeFailure(
          'CAPABILITY_UNSUPPORTED',
          'Selection requires one unambiguous low-risk power control per device.',
        );
      }
      commands.add({
        'schema_version': '1.0',
        'request_id': uniqueId(),
        'device_id': device,
        'component_id': candidates.single['component'],
        'capability': 'power',
        'action': 'set',
        'arguments': {'state': on},
        'connection': preferences.route(device),
        'timeout_ms': 10000,
        'wait_for': 'verified',
      });
    }
    final batch = {'schema_version': '1.0', 'commands': commands};
    await core.call('plan_batch', {'batch': batch});
    batchResult = await core.call('execute_batch', {'batch': batch}) as Json;
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
