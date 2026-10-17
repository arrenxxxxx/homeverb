import 'dart:async';
import 'package:flutter/foundation.dart';
import 'client_contract.dart';
import 'preferences.dart';
import 'session_permissions.dart';
import 'account_identity.dart';

enum LocalCleanup { cachedState, executionHistory }

enum StateRefreshView { none, cards, device }

// The compiled native hosts currently expose exact const grants or an object
// grant for Mock. Unknown schema constraints hide quick controls conservatively;
// they never become a broader permission or bypass Core's JSON Schema validator.
bool _matchesDisplayGrant(dynamic schema, Json arguments) {
  if (schema is bool) return schema;
  if (schema is! Map ||
      schema.keys.any(
        (key) => !{'const', 'type', 'title', 'description'}.contains(key),
      ))
    return false;
  if (schema['type'] != null && schema['type'] != 'object') return false;
  if (schema.containsKey('const')) {
    return schema['const'] is Map &&
        mapEquals(schema['const'] as Map, arguments);
  }
  return true;
}

class HomeController extends ChangeNotifier {
  HomeController(this.core, this.preferences, {this.reopen});
  CoreClient core;
  final Future<CoreClient> Function(List<SessionPermission>)? reopen;
  List<SessionPermission> _controlPermissions = const [];
  bool _foreground = true;
  bool get isForeground => _foreground;
  int get controlEpoch => _readEpoch;
  List<Json>? _controlPolicies;
  Timer? _automaticRefreshTimer;
  Future<void>? _automaticRefreshWork;
  StateRefreshView _automaticRefreshView = StateRefreshView.none;
  bool _automaticRefreshEnabled = false;
  int _automaticRefreshGeneration = 0;

  /// The production App host enables this schedule; Core/CLI/MCP never own it.
  void startAutomaticRefresh() {
    _automaticRefreshEnabled = true;
    _scheduleAutomaticRefresh();
  }

  void stopAutomaticRefresh() {
    _automaticRefreshEnabled = false;
    _pauseAutomaticRefresh();
  }

  void setAutomaticRefreshView(StateRefreshView view) {
    if (view != _automaticRefreshView) _pauseAutomaticRefresh();
    _automaticRefreshView = view;
    _scheduleAutomaticRefresh();
  }

  void _pauseAutomaticRefresh({bool cancelRead = true}) {
    _automaticRefreshTimer?.cancel();
    _automaticRefreshTimer = null;
    _automaticRefreshGeneration++;
    if (cancelRead && _automaticRefreshWork != null) core.cancelAll();
  }

  void _scheduleAutomaticRefresh() {
    if (!_automaticRefreshEnabled ||
        !_foreground ||
        busy ||
        _automaticRefreshView == StateRefreshView.none ||
        _automaticRefreshTimer != null)
      return;
    _automaticRefreshTimer = Timer.periodic(const Duration(seconds: 3), (_) {
      if (busy ||
          _automaticRefreshWork != null ||
          _coreNeedsReopen ||
          approvalPreview != null)
        return;
      final work = _refreshAutomatically(_automaticRefreshGeneration);
      _automaticRefreshWork = work;
      unawaited(work.whenComplete(() => _automaticRefreshWork = null));
    });
  }

  Future<void> _refreshAutomatically(int generation) async {
    final epoch = _readEpoch, client = core;
    bool current() =>
        _automaticRefreshEnabled &&
        _foreground &&
        !busy &&
        _automaticRefreshView != StateRefreshView.none &&
        generation == _automaticRefreshGeneration &&
        epoch == _readEpoch &&
        client == core;
    if (!current()) return;
    try {
      final devices = _automaticRefreshView == StateRefreshView.cards
          ? visibleCards.map((card) => card['device'] as Json).toList()
          : [if (description?['device'] case final Json device) device];
      for (final device in devices) {
        final parts = device['components'] as List? ?? [];
        if (device['support_status'] == 'unsupported' || parts.isEmpty)
          continue;
        final deviceId = device['id'] as String;
        final route = preferences.route(deviceId);
        for (final part in parts) {
          if (!current()) return;
          if ((part['capabilities'] as List? ?? []).isEmpty) continue;
          final componentId = part['id'] as String;
          try {
            final next =
                await client.call('read_state', {
                      'device': deviceId,
                      'component': componentId,
                      'connection': route,
                      'cached': false,
                    })
                    as Json;
            if (!current()) return;
            if (next['device_id'] != deviceId ||
                next['component_id'] != componentId)
              continue;
            _componentSnapshots[(deviceId, componentId)] = next;
            _componentReadErrors.remove((deviceId, componentId));
            if (componentId == parts.first['id']) snapshots[deviceId] = next;
            if (selectedDevice == deviceId && component == componentId)
              snapshot = next;
            notifyListeners();
          } catch (_) {
            // Keep previous evidence and manual errors; never notify on a failed read.
            if (!current()) return;
          }
        }
      }
    } catch (_) {
      // Optional automatic reads never surface an error or retry within a cycle.
    }
  }

  @override
  void dispose() {
    stopAutomaticRefresh();
    super.dispose();
  }

  final Map<(String, String), Set<bool>> _cardPowerChoices = {};

  /// Local display choices, not authorization. Selection rechecks the session.
  Set<bool> cardPowerChoices(String device) {
    if (!_foreground) return {};
    final route = preferences.route(device);
    return {
      for (final entry in _cardPowerChoices.entries)
        if (entry.key.$1 == device &&
            (route == 'auto' || route == entry.key.$2))
          ...entry.value,
    };
  }

  Future<void> _loadCardPower() async {
    _cardPowerChoices.clear();
    final epoch = _readEpoch, client = core;
    void requireCurrent() {
      if (!_foreground || epoch != _readEpoch || client != core) {
        throw NativeFailure('CANCELLED', 'Card inspection was cancelled.');
      }
    }

    for (final card in cards) {
      requireCurrent();
      final device = card['device'] as Json;
      final parts = device['components'] as List? ?? [];
      // A component power property is not necessarily a whole-appliance switch.
      if (device['support_status'] == 'unsupported' ||
          parts.length != 1 ||
          !(parts.single['capabilities'] as List? ?? []).contains('power'))
        continue;
      final id = device['id'] as String;
      final componentId = parts.single['id'] as String;
      final choices = <String, Set<bool>>{};
      try {
        final connections = device['connections'] as List? ?? [];
        if (_canPrepareControls && supports('inspect_control_options')) {
          final cloud = connections.where((c) => c['transport'] == 'cloud');
          if (cloud.length != 1) continue;
          final connection = cloud.single;
          final inspected =
              await client.call('inspect_control_options', {
                    'device_id': id,
                    'connection_id': connection['id'],
                  })
                  as Json;
          requireCurrent();
          if (inspected['device_id'] != id ||
              inspected['connection_id'] != connection['id'])
            continue;
          choices['cloud'] = {
            for (final choice in SessionPermission.choices(inspected))
              if (choice.component == componentId &&
                  choice.capability == 'power' &&
                  choice.action == 'set' &&
                  choice.parameter == 'state' &&
                  choice.value is bool)
                choice.value as bool,
          };
        } else {
          final descriptor =
              await client.call('describe_device', {'device': id}) as Json;
          requireCurrent();
          if (descriptor['device']?['id'] != id) continue;
          final power = (descriptor['capabilities'] as List? ?? [])
              .where(
                (cap) =>
                    cap['id'] == 'power' &&
                    (cap['component_id'] == null ||
                        cap['component_id'] == componentId),
              )
              .singleOrNull;
          final action = power?['actions']?['set'] as Json?;
          final parameters = action?['parameters'] as Map?;
          final state = parameters?['properties']?['state'] as Map?;
          if (action?['risk'] != 'low' ||
              action?['idempotency'] != 'set_value' ||
              power?['properties']?['state']?['writable'] == false ||
              parameters?['type'] != 'object' ||
              parameters?['additionalProperties'] != false ||
              !listEquals(parameters?['required'] as List?, ['state']) ||
              (parameters?['properties'] as Map?)?.length != 1 ||
              parameters!.keys.any(
                (key) => !{
                  'type',
                  'properties',
                  'required',
                  'additionalProperties',
                  'minProperties',
                  'maxProperties',
                  'description',
                  'title',
                }.contains(key),
              ) ||
              (parameters['minProperties'] as num? ?? 0) > 1 ||
              (parameters['maxProperties'] as num? ?? 1) < 1 ||
              state?['type'] != 'boolean' ||
              state!.keys.any(
                (key) =>
                    !{'type', 'enum', 'description', 'title'}.contains(key),
              ))
            continue;
          for (final connection in connections) {
            final transport = connection['transport'] as String;
            if (!(action!['transports'] as List).contains(transport)) continue;
            List grants = [
              {'argument_schema': true},
            ];
            if (client is NativeSessionClient) {
              if (!supports('inspect_session_grants')) continue;
              final policy =
                  await client.call('inspect_session_grants', {
                        'device_id': id,
                        'connection_id': connection['id'],
                      })
                      as Json;
              requireCurrent();
              if (policy['device_id'] != id ||
                  policy['connection_id'] != connection['id'] ||
                  policy['transport'] != transport ||
                  policy['source'] != 'session_policy' ||
                  policy['provider_authorization'] != 'not_checked')
                continue;
              grants = (policy['grants'] as List)
                  .where(
                    (grant) =>
                        grant['component_id'] == componentId &&
                        grant['capability'] == 'power' &&
                        grant['action'] == 'set',
                  )
                  .toList();
            }
            choices.putIfAbsent(transport, () => {}).addAll({
              for (final value in [false, true])
                if ((state['enum'] == null ||
                        (state['enum'] as List).contains(value)) &&
                    grants.any(
                      (grant) => _matchesDisplayGrant(
                        grant['argument_schema'],
                        {'state': value},
                      ),
                    ))
                  value,
            });
          }
        }
        requireCurrent();
        for (final entry in choices.entries) {
          _cardPowerChoices[(id, entry.key)] = entry.value;
        }
        final cached =
            await client.call('read_state', {
                  'device': id,
                  'component': componentId,
                  'connection': preferences.route(id),
                  'cached': true,
                })
                as Json;
        requireCurrent();
        if (cached['device_id'] == id &&
            cached['component_id'] == componentId) {
          snapshots[id] = cached;
          _componentSnapshots[(id, componentId)] = cached;
        }
      } catch (_) {
        // Optional local inspection/cache failures never trigger live fallback.
        requireCurrent();
      }
    }
  }

  /// Presentation-only intersection. Core still plans and validates every write.
  bool canControlValue(
    String capability,
    String action,
    String parameter,
    Object value,
  ) => _controlRules(capability, action).any(
    (rule) => _matchesDisplayGrant(rule['argument_schema'], {parameter: value}),
  );

  bool hasControlAction(String capability, String action) =>
      _controlRules(capability, action).any((rule) {
        final schema = rule['argument_schema'];
        if (schema is bool) return schema;
        if (schema is! Map) return false;
        if (schema['const'] case final Map args) {
          return _matchesDisplayGrant(schema, args.cast<String, dynamic>());
        }
        return _matchesDisplayGrant(schema, {});
      });

  Iterable<Json> _controlRules(String capability, String action) sync* {
    final definition =
        componentCapability(component, capability)?['actions']?[action]
            as Json?;
    if (definition == null) return;
    final route = preferences.route(selectedDevice);
    final transports = definition['transports'] as List? ?? [];
    if (core is NativeSessionClient) {
      for (final policy in _controlPolicies ?? <Json>[]) {
        if ((route != 'auto' && route != policy['transport']) ||
            !transports.contains(policy['transport']))
          continue;
        for (final grant in policy['grants'] as List) {
          if (grant['component_id'] == component &&
              grant['capability'] == capability &&
              grant['action'] == action)
            yield grant as Json;
        }
      }
    } else if (_canPrepareControls) {
      if ((route != 'auto' && route != 'cloud') ||
          !transports.contains('cloud'))
        return;
      for (final p in _controlPermissions) {
        if (p.device == selectedDevice &&
            p.component == component &&
            p.capability == capability &&
            p.action == action) {
          yield {
            'argument_schema': {
              'const': {p.parameter: p.value},
            },
          };
        }
      }
    } else {
      // Injected CoreClient fixtures supply planning policy. Production clients
      // always require current native-session inspection, never saved metadata.
      if (route == 'auto' || transports.contains(route))
        yield {'argument_schema': true};
    }
  }

  Future<void> _inspectControlPolicies() async {
    _controlPolicies = null;
    if (core is! NativeSessionClient || !supports('inspect_session_grants'))
      return;
    final device = selectedDevice!, epoch = _readEpoch;
    final policies = <Json>[];
    for (final connection in description!['device']['connections'] as List) {
      final policy =
          await core.call('inspect_session_grants', {
                'device_id': device,
                'connection_id': connection['id'],
              })
              as Json;
      _requireCurrentDevice(device, epoch);
      if (policy['device_id'] != device ||
          policy['connection_id'] != connection['id'] ||
          policy['transport'] != connection['transport'] ||
          policy['source'] != 'session_policy' ||
          policy['provider_authorization'] != 'not_checked' ||
          policy['grants'] is! List) {
        throw NativeFailure(
          'POLICY_DENIED',
          'Control policy does not match this connection.',
        );
      }
      policies.add(policy);
    }
    _controlPolicies = policies;
  }

  bool _coreNeedsReopen = false;
  bool get _canPrepareControls =>
      reopen != null &&
      !isMock &&
      (core is! NativeSessionClient ||
          (core as NativeSessionClient).apiAtLeast(22));

  void _requireCurrentDevice(String device, int epoch) {
    if (!_foreground || epoch != _readEpoch || selectedDevice != device) {
      throw NativeFailure('CANCELLED', 'Device setup was cancelled.');
    }
  }

  /// Owner-approved default: admit every reviewed low-risk value for the device
  /// being opened. This is foreground host policy, never a preference or command.
  Future<void> _prepareDeviceControls(String device) async {
    if (!_canPrepareControls) return;
    final epoch = _readEpoch;
    _requireCurrentDevice(device, epoch);
    final connections = (description?['device']?['connections'] as List? ?? [])
        .where((connection) => connection['transport'] == 'cloud')
        .toList();
    if (connections.length != 1) return;
    final connection = connections.single;
    final inspected =
        await core.call('inspect_control_options', {
              'device_id': device,
              'connection_id': connection['id'],
            })
            as Json;
    _requireCurrentDevice(device, epoch);
    if (inspected['device_id'] != device ||
        inspected['connection_id'] != connection['id'] ||
        inspected['source'] != 'saved_inventory' ||
        inspected['authorization'] != 'not_checked') {
      throw NativeFailure(
        'POLICY_DENIED',
        'Control options do not match this device.',
      );
    }
    final next = SessionPermission.choices(inspected);
    if (next.isEmpty) return;
    final previousIds = _controlPermissions.map((p) => p.identity).toSet();
    final nextIds = next.map((p) => p.identity).toSet();
    if (setEquals(previousIds, nextIds)) return;
    SessionPermission.startup(next);
    await _replaceControlSession(next);
    final refreshedEpoch = _readEpoch;
    _requireCurrentDevice(device, refreshedEpoch);
    if (next.isNotEmpty) {
      // Opening this device is the bounded, on-demand operation. Refresh only its
      // account to publish descriptors under the newly configured exact policy.
      final result =
          await core.call('refresh_account', {
                'account_id': connection['account_id'],
              })
              as Json;
      _requireCurrentDevice(device, refreshedEpoch);
      validateAccountReport(result);
      final retrieval = result['retrieval'] as Json;
      if (retrieval['status'] != 'complete') {
        // Retry must rebuild the policy and refresh; never use stale actions.
        _controlPermissions = const [];
        throw NativeFailure(
          retrieval['error']?['code'] ?? 'INVENTORY_INCOMPLETE',
          'Device setup did not complete. Open the device again to retry.',
        );
      }
      await loadLocal();
      _requireCurrentDevice(device, refreshedEpoch);
    }
    final current =
        await core.call('describe_device', {'device': device}) as Json;
    _requireCurrentDevice(device, refreshedEpoch);
    description = current;
    _controlPermissions = List.unmodifiable(next);
  }

  Future<void> _replaceControlSession(List<SessionPermission> next) async {
    cancelPending();
    final epoch = _readEpoch;
    _coreNeedsReopen = true;
    _controlPermissions = const [];
    _controlPolicies = null;
    await core.close();
    if (!_foreground || epoch != _readEpoch) {
      throw NativeFailure('CANCELLED', 'Device setup was cancelled.');
    }
    final replacement = await reopen!(List.unmodifiable(next));
    if (!_foreground || epoch != _readEpoch) {
      await replacement.close();
      throw NativeFailure('CANCELLED', 'Device setup was cancelled.');
    }
    core = replacement;
    _coreNeedsReopen = false;
    await loadLocal();
  }

  final LocalPreferences preferences;
  List<Json> rooms = [], cards = [], history = [];
  Json? description, snapshot, receipt, diagnostics, batchResult;
  Json? accountConflicts, routeInspection;
  final Map<String, Json> snapshots = {};
  final Map<(String, String), Json> _componentSnapshots = {};
  final Map<(String, String), Object> _componentReadErrors = {};
  String? selectedDevice, selectedRoom, editingRoom;
  String component = 'main', filter = 'All', search = '';
  final Set<String> selection = {};
  int _readEpoch = 0;
  void cancelPending() {
    _pauseAutomaticRefresh(cancelRead: false);
    discardApproval();
    _readEpoch++;
    accountInventory = null;
    accountConflicts = null;
    routeInspection = null;
    core.cancelAll();
  }

  void setForeground(bool foreground) {
    _foreground = foreground;
    if (!foreground) {
      _pauseAutomaticRefresh(cancelRead: false);
      discardApproval();
      _readEpoch++;
      accountInventory = null;
      accountConflicts = null;
      routeInspection = null;
    }
    core.foreground(foreground);
    if (foreground) _scheduleAutomaticRefresh();
    if (!foreground) notifyListeners();
  }

  bool busy = false;
  Object? error;

  Future<bool> perform(Future<void> Function() work) async {
    if (busy) return false;
    busy = true;
    error = null;
    final epoch = _readEpoch;
    final automaticRead = _automaticRefreshWork;
    _pauseAutomaticRefresh();
    notifyListeners();
    try {
      // Native cancellation keeps admission closed until its active call drains.
      if (automaticRead != null) {
        await automaticRead;
        if (!_foreground || epoch != _readEpoch) {
          throw NativeFailure(
            'CANCELLED',
            'Operation was cancelled before starting.',
          );
        }
      }
      if (_coreNeedsReopen) await _replaceControlSession(const []);
      await work();
      return true;
    } catch (e) {
      error = e;
      return false;
    } finally {
      busy = false;
      _scheduleAutomaticRefresh();
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
    await _loadCardPower();
  }

  Future<bool> initialize() => perform(() async {
    if (accountNeedsReload) {
      await reloadAccountViews();
      return;
    }
    if (cleanupNeedsReload) {
      await reloadAfterCleanup();
      return;
    }
    await loadLocal();
    if (accountClient != null && supports('list_accounts')) {
      accounts = (await core.call('list_accounts') as List).cast<Json>();
    }
  });
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
    routeInspection = null;
    description = null;
    _controlPolicies = null;
    snapshot = null;
    final epoch = _readEpoch;
    description = await core.call('describe_device', {'device': id}) as Json;
    _requireCurrentDevice(id, epoch);
    await _prepareDeviceControls(id);
    component = components.first['id'] as String;
    if (description!['device']['support_status'] == 'unsupported') return;
    await _inspectControlPolicies();
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
      if (d['support_status'] == 'unsupported') continue;
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

  ApprovalPreview? approvalPreview;
  Timer? _approvalTimer;
  String? approvalRequestId;
  bool approvalNeedsReceipt = false;
  ApprovalClient? get approvalClient =>
      core is ApprovalClient ? core as ApprovalClient : null;
  bool get supportsTrustedApproval =>
      approvalClient?.supportsTrustedApproval ?? false;

  void discardApproval() {
    final previous = approvalPreview;
    approvalPreview = null;
    _approvalTimer?.cancel();
    _approvalTimer = null;
    if (previous != null) {
      unawaited(
        approvalClient!.discardApproval(previous).catchError((Object _) {
          // Local consent is already revoked; native expiry/lifecycle also prevents reuse.
        }),
      );
    }
  }

  Future<bool> prepareApproval(
    String capabilityId,
    String action,
    Json arguments,
    String route,
  ) => perform(() async {
    discardApproval();
    final epoch = _readEpoch;
    if (!supportsTrustedApproval ||
        capability(capabilityId)?['actions']?[action]?['risk'] != 'elevated') {
      throw NativeFailure(
        'APPROVAL_UNAVAILABLE',
        'This action has no trusted confirmation path.',
      );
    }
    if (!['lan', 'cloud'].contains(route)) {
      throw NativeFailure(
        'INVALID_ARGUMENT',
        'Choose one explicit connection.',
      );
    }
    final preview = await approvalClient!.prepareApproval({
      'schema_version': '1.0',
      'request_id': uniqueId(),
      'device_id': selectedDevice,
      'component_id': component,
      'capability': capabilityId,
      'action': action,
      'arguments': arguments,
      'connection': route,
      'timeout_ms': 10000,
      'wait_for': 'verified',
    });
    if (epoch != _readEpoch) {
      await approvalClient!.discardApproval(preview);
      throw NativeFailure('CANCELLED', 'The confirmation was cancelled.');
    }
    approvalNeedsReceipt = false;
    approvalPreview = preview;
    approvalRequestId = preview.command['request_id'] as String;
    _approvalTimer = Timer(preview.remaining, () {
      discardApproval();
      error = NativeFailure(
        'APPROVAL_EXPIRED',
        'Review the command again before confirming.',
      );
      notifyListeners();
    });
  });

  Future<bool> confirmApproval() => perform(() async {
    final preview = approvalPreview;
    if (preview == null || preview.remaining == Duration.zero) {
      discardApproval();
      throw NativeFailure('APPROVAL_INVALID', 'Review a new confirmation.');
    }
    // Consume UI consent before awaiting. A failed/uncertain call is never replayed.
    approvalPreview = null;
    _approvalTimer?.cancel();
    _approvalTimer = null;
    receipt = null;
    try {
      receipt = await approvalClient!.executeApproved(preview);
      approvalNeedsReceipt = false;
    } catch (_) {
      approvalNeedsReceipt = true;
      rethrow;
    }
  });

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
  List<Json> accounts = [];
  Json? accountDetails, accountReport, accountRemoval, accountInventory;
  String? selectedAccount;
  List<String>? removalScope;
  bool accountNeedsReload = false;
  AccountClient? get accountClient =>
      core is AccountClient ? core as AccountClient : null;
  bool get isMock => accountClient?.adapter != 'midea.meiju';
  Json? get account => accountDetails?['account'] as Json?;
  String accountRemark(Json account) {
    final label = account['label'] as String? ?? '';
    return label == 'Meiju account' ||
            label == preferences.accountLoginLabel(account['id'])
        ? ''
        : label;
  }

  String? accountDisplayName(Json account) {
    final remark = accountRemark(account);
    return remark.isNotEmpty
        ? remark
        : preferences.accountLoginLabel(account['id']);
  }

  bool get canSignIn =>
      !isMock &&
      accountClient!.supportsAccountConnection &&
      (account?['credential_state'] != 'pending' ||
          accountClient!.supportsPendingReauthorization) &&
      account?['credential_state'] != 'removing';
  List<String> get accountDeviceIds =>
      (accountDetails?['device_ids'] as List? ?? []).cast<String>();
  List<Json> get accountCards => cards
      .where((card) => accountDeviceIds.contains(card['device']['id']))
      .toList();

  Future<void> reloadAccountViews() async {
    accountInventory = null;
    accountConflicts = null;
    routeInspection = null;
    accounts = (await core.call('list_accounts') as List).cast<Json>();
    await preferences.retainAccountLoginLabels(
      accounts.map((account) => account['id'] as String).toSet(),
    );
    await loadLocal();
    if (selectedAccount != null) {
      if (accounts.any((a) => a['id'] == selectedAccount)) {
        accountDetails =
            await core.call('get_account', {'account_id': selectedAccount})
                as Json;
      } else {
        selectedAccount = null;
        accountDetails = null;
      }
    }
    final currentIds = cards.map((card) => card['device']['id']).toSet();
    snapshots.removeWhere((id, _) => !currentIds.contains(id));
    _componentSnapshots.removeWhere((key, _) => !currentIds.contains(key.$1));
    _componentReadErrors.removeWhere((key, _) => !currentIds.contains(key.$1));
    selection.removeWhere((id) => !currentIds.contains(id));
    if (selectedDevice != null && !currentIds.contains(selectedDevice)) {
      _readEpoch++;
      selectedDevice = null;
      description = null;
      snapshot = null;
    }
    accountNeedsReload = false;
  }

  Future<bool> loadAccounts() => perform(reloadAccountViews);
  Future<bool> selectAccount(String id) => perform(() async {
    selectedAccount = id;
    accountInventory = null;
    accountConflicts = null;
    routeInspection = null;
    accountDetails = null;
    accountReport = null;
    accountRemoval = null;
    removalScope = null;
    accountDetails = await core.call('get_account', {'account_id': id}) as Json;
    await loadLocal();
  });

  Future<bool> inspectAccountInventory() => perform(() async {
    accountInventory = null;
    accountConflicts = null;
    routeInspection = null;
    final id = selectedAccount;
    if (id == null ||
        account?['credential_state'] != 'stored' ||
        !supports('inspect_account_inventory')) {
      throw NativeFailure(
        'UNSUPPORTED_OPERATION',
        'Select a stored account with saved-inventory inspection support.',
      );
    }
    final epoch = _readEpoch;
    try {
      final result =
          await core.call('inspect_account_inventory', {'account_id': id})
              as Json;
      if (epoch != _readEpoch || selectedAccount != id) return;
      accountInventory = result;
    } catch (_) {
      if (epoch != _readEpoch || selectedAccount != id) return;
      rethrow;
    }
  });

  Future<bool> inspectDeviceConflicts(String deviceId) => perform(() async {
    accountConflicts = null;
    final id = selectedAccount;
    final bindings = accountInventory?['devices'] as List? ?? [];
    if (id == null ||
        !supports('inspect_device_conflicts') ||
        !bindings.any((binding) => binding['device_id'] == deviceId)) {
      throw NativeFailure(
        'INVALID_ARGUMENT',
        'Select a saved inventory binding.',
      );
    }
    final epoch = _readEpoch;
    try {
      final result =
          await core.call('inspect_device_conflicts', {
                'account_id': id,
                'device_id': deviceId,
              })
              as Json;
      if (epoch == _readEpoch && selectedAccount == id)
        accountConflicts = result;
    } catch (_) {
      if (epoch != _readEpoch || selectedAccount != id) return;
      rethrow;
    }
  });

  Future<bool> inspectRoute(
    String connectionId, {
    required bool options,
  }) => perform(() async {
    routeInspection = null;
    final id = selectedDevice;
    final method = options
        ? 'inspect_control_options'
        : 'inspect_session_grants';
    final connections = description?['device']?['connections'] as List? ?? [];
    if (id == null ||
        !supports(method) ||
        !connections.any((connection) => connection['id'] == connectionId)) {
      throw NativeFailure(
        'INVALID_ARGUMENT',
        'Select a saved device connection.',
      );
    }
    final epoch = _readEpoch;
    try {
      final result =
          await core.call(method, {
                'device_id': id,
                'connection_id': connectionId,
              })
              as Json;
      if (epoch == _readEpoch && selectedDevice == id) routeInspection = result;
    } catch (_) {
      if (epoch != _readEpoch || selectedDevice != id) return;
      rethrow;
    }
  });

  Future<bool> renameAccount(String label) => perform(() async {
    accountNeedsReload = true;
    await core.call('rename_account', {
      'account_id': selectedAccount,
      'label': label.trim().isNotEmpty
          ? label.trim()
          : preferences.accountLoginLabel(selectedAccount!) ?? 'Meiju account',
    });
    await reloadAccountViews();
  });

  Future<bool> retrieveAccount({bool recover = false}) => perform(() async {
    accountInventory = null;
    accountConflicts = null;
    routeInspection = null;
    accountNeedsReload = true;
    accountReport = null;
    accountReport =
        await core.call(recover ? 'recover_account' : 'refresh_account', {
              'account_id': selectedAccount,
            })
            as Json;
    validateAccountReport(accountReport!);
    await reloadAccountViews();
  });

  Future<bool> connectAccount(
    Uint8List login,
    Uint8List password,
    String label,
  ) async {
    try {
      final loginLabel = maskedAccountLogin(login);
      return await perform(() async {
        if (!canSignIn)
          throw NativeFailure(
            'AUTH_FLOW_UNSUPPORTED',
            'This account flow is unavailable.',
          );
        accountNeedsReload = true;
        accountReport = null;
        final epoch = _readEpoch;
        accountReport = await accountClient!.connectAccount(
          login: login,
          password: password,
          label: label.trim().isNotEmpty
              ? label.trim()
              : loginLabel ?? 'Meiju account',
          accountId: selectedAccount,
        );
        validateAccountReport(accountReport!);
        selectedAccount = accountReport!['account']['id'] as String;
        if (_foreground &&
            epoch == _readEpoch &&
            loginLabel != null &&
            accountReport!['authentication'] == 'succeeded') {
          await preferences.saveAccountLoginLabel(selectedAccount!, loginLabel);
        }
        await reloadAccountViews();
      });
    } finally {
      login.fillRange(0, login.length, 0);
      password.fillRange(0, password.length, 0);
    }
  }

  void validateAccountReport(Json report) {
    final retrieval = report['retrieval'] as Json;
    if (!['complete', 'partial', 'failed'].contains(retrieval['status']) ||
        !['succeeded', 'not_attempted'].contains(report['authentication']) ||
        (retrieval['status'] == 'complete' && retrieval['error'] != null)) {
      throw NativeFailure(
        'UNSUPPORTED_OUTCOME',
        'Unrecognized account result. Reload local accounts.',
      );
    }
  }

  Future<bool> reviewAccountRemoval() => perform(() async {
    removalScope = null;
    accountNeedsReload = true;
    await reloadAccountViews();
    if (selectedAccount == null)
      throw NativeFailure(
        'ACCOUNT_NOT_FOUND',
        'The account is no longer saved.',
      );
    removalScope = List<String>.unmodifiable(accountDeviceIds);
    accountRemoval = null;
  });

  Future<bool> removeAccount() => perform(() async {
    final scope = removalScope;
    if (scope == null || selectedAccount == null || accountNeedsReload) {
      throw NativeFailure(
        'INVALID_ARGUMENT',
        'Review the local removal scope first.',
      );
    }
    accountNeedsReload = true;
    accountRemoval = null;
    try {
      accountRemoval =
          await core.call('remove_account', {
                'account_id': selectedAccount,
                'expected_device_ids': scope,
              })
              as Json;
      final status = accountRemoval!['status'];
      if (!['removed', 'pending'].contains(status) ||
          (status == 'removed' && accountRemoval!['error'] != null)) {
        throw NativeFailure(
          'UNSUPPORTED_OUTCOME',
          'Unrecognized removal result. Reload local accounts.',
        );
      }
      await reloadAccountViews();
    } finally {
      // Any new attempt must display and confirm a newly reviewed local scope.
      removalScope = null;
    }
  });

  bool supports(String method) =>
      core is! NativeSessionClient ||
      (core as NativeSessionClient).methods.contains(method);
  int? cleanupCount;
  LocalCleanup? cleanupResult;
  bool cleanupNeedsReload = false;

  Future<bool> clearLocalData(LocalCleanup scope) => perform(() async {
    cleanupCount = null;
    cleanupResult = null;
    final method = scope == LocalCleanup.cachedState
        ? 'clear_cached_state'
        : 'clear_execution_history';
    // Fence late reads and invalidate affected views even after a lost reply.
    _readEpoch++;
    cleanupNeedsReload = true;
    Json result;
    try {
      result = await core.call(method) as Json;
    } finally {
      diagnostics = null;
      if (scope == LocalCleanup.cachedState) {
        snapshots.clear();
        _componentSnapshots.clear();
        _componentReadErrors.clear();
        snapshot = null;
      } else {
        history = [];
        receipt = null;
        batchResult = null;
      }
    }
    cleanupCount =
        result[scope == LocalCleanup.cachedState
                ? 'removed_snapshots'
                : 'removed_receipts']
            as int;
    cleanupResult = scope;
    await reloadAfterCleanup();
  });

  Future<void> reloadAfterCleanup() async {
    await loadLocal();
    history = (await core.call('recent_executions') as List).cast<Json>();
    // No cached-state miss ever falls through to a live read.
    if (selectedDevice != null &&
        currentCard != null &&
        description != null &&
        description!['device']['support_status'] != 'unsupported') {
      for (final part in components) {
        try {
          await readComponent(part['id'], cached: true);
        } on NativeFailure catch (error) {
          if (!['CACHE_MISS', 'STATE_UNAVAILABLE'].contains(error.code))
            rethrow;
        }
      }
    }
    cleanupNeedsReload = false;
  }

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
    diagnostics = null;
    final epoch = _readEpoch;
    try {
      final report = await core.call('diagnostics') as Json;
      if (epoch == _readEpoch) diagnostics = report;
    } catch (_) {
      if (epoch == _readEpoch) rethrow;
    }
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
