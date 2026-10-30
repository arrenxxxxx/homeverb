import 'dart:convert';

/// An exact startup permission derived by the trusted foreground controller.
/// This is never loaded from ordinary preferences, a request or a descriptor.
class SessionPermission {
  const SessionPermission._(
    this.device,
    this.component,
    this.capability,
    this.action,
    this.parameter,
    this.value,
    this.startupField,
  );
  final String device, component, capability, action, parameter, startupField;
  final Object value;
  bool get requiresReferenceFanApi =>
      startupField == 'mijia_fan_grants' &&
      const {
        'horizontal_swing_included_angle_6',
        'vertical_swing_7',
        'vertical_swing_included_angle_8',
        'anion_9',
      }.contains(parameter);
  bool get requiresReferenceSettingApi =>
      startupField == "mijia_setting_grants";
  bool get requiresAuxiliarySettingApi =>
      requiresReferenceSettingApi &&
      const {'light-6', 'function-3'}.contains(component);
  bool get requiresOutletApi => startupField == "mijia_outlet_grants";
  String get identity =>
      jsonEncode([device, component, capability, action, value]);

  Map<String, dynamic> get arguments => parameter == 'arguments'
      ? Map<String, dynamic>.unmodifiable(value as Map)
      : {parameter: value};

  Map<String, dynamic> get startupEntry => {
    'device_id': device,
    if (startupField == 'meiju_dishwasher_cycle_grants') 'action': action,
    if (startupField == 'mijia_fan_grants' ||
        startupField == 'mijia_setting_grants' ||
        startupField == 'mijia_outlet_grants') ...{
      'component_id': component,
      'property': parameter,
      'value': value,
    } else if (startupField == 'meiju_ac_feature_grants') ...{
      'feature': parameter,
      'value': value,
    } else
      parameter: value,
  };

  // Host configuration translation, not vendor wire encoding. The UI derives
  // values from the exact inspected schema, never this mapping's union.
  static const _startup = {
    ('power', 'set'): ('meiju_power_grants', 'state'),
    ('climate', 'set'): ('meiju_temperature_grants', 'target_temperature'),
    ('vendor.midea.ac', 'set_fan_speed'): ('meiju_fan_grants', 'wind_speed'),
    ('vendor.midea.ac', 'activate_mode'): ('meiju_mode_grants', 'mode'),
  };

  static const _referenceSettings = {
    ('light-6', 'power', 'set'): 'state',
    ('light-6', 'vendor.xiaomi.mijia.light', 'set_brightness_2'):
        'brightness_2',
    ('function-3', 'vendor.xiaomi.mijia.function', 'set_quiet_5'): 'quiet_5',
    ('light-2', 'power', 'set'): 'state',
    ('light-2', 'vendor.xiaomi.mijia.light', 'set_brightness_2'):
        'brightness_2',
    ('light-2', 'vendor.xiaomi.mijia.light', 'set_color_temperature_3'):
        'color_temperature_3',
    ('light-2', 'vendor.xiaomi.mijia.light', 'set_mode_7'): 'mode_7',
    ('light-2', 'vendor.xiaomi.mijia.light', 'set_default_power_on_state_9'):
        'default_power_on_state_9',
    ('light-2', 'vendor.xiaomi.mijia.light', 'set_flex_switch_12'):
        'flex_switch_12',
    ('light-2', 'vendor.xiaomi.mijia.light', 'set_night_light_switch_13'):
        'night_light_switch_13',
    ('air_purifier-2', 'power', 'set'): 'state',
    ('air_purifier-2', 'vendor.xiaomi.mijia.air_purifier', 'set_fan_level_2'):
        'fan_level_2',
    ('air_purifier-2', 'vendor.xiaomi.mijia.air_purifier', 'set_mode_3'):
        'mode_3',
    ('indicator_light-5', 'vendor.xiaomi.mijia.indicator_light', 'set_on_1'):
        'on_1',
    ('alarm-6', 'vendor.xiaomi.mijia.alarm', 'set_alarm_1'): 'alarm_1',
    (
      'physical_controls_locked-7',
      'vendor.xiaomi.mijia.physical_controls_locked',
      'set_physical_controls_locked_1',
    ): 'physical_controls_locked_1',
    (
      'air_purifier_favorite-8',
      'vendor.xiaomi.mijia.air_purifier_favorite',
      'set_favorite_fan_level_1',
    ): 'favorite_fan_level_1',
  };

  static const _outletSettings = {
    ('switch-2', 'power', 'set'): ('state', 'elevated'),
    ('switch-2', 'vendor.xiaomi.mijia.switch', 'set_default_power_on_state_2'):
        ('default_power_on_state_2', 'elevated'),
    (
      'physical_controls_locked-7',
      'vendor.xiaomi.mijia.physical_controls_locked',
      'set_physical_controls_locked_1',
    ): (
      'physical_controls_locked_1',
      'elevated',
    ),
    ('indicator_light-13', 'vendor.xiaomi.mijia.indicator_light', 'set_on_1'): (
      'on_1',
      'low',
    ),
  };

  static List<SessionPermission> choices(
    Map<String, dynamic> inspection, {
    bool trustedApproval = false,
  }) {
    if (inspection['source'] != 'saved_inventory' ||
        inspection['authorization'] != 'not_checked' ||
        inspection['device_id'] is! String)
      return [];
    final result = <SessionPermission>[];
    for (final option in inspection['options'] as List? ?? []) {
      if (trustedApproval &&
          option['capability'] == 'vendor.midea.dishwasher') {
        result.addAll(_dishwasherChoices(inspection['device_id'], option));
        continue;
      }
      final propertiesForMapping =
          option['definition']?['parameters']?['properties'];
      final featureParameter =
          propertiesForMapping is Map && propertiesForMapping.length == 1
          ? propertiesForMapping.keys.single
          : null;
      final component = option['component_id'];
      final fanComponent =
          component is String &&
          RegExp(r'^fan-[1-9][0-9]{0,9}$').hasMatch(component);
      final fanParameter =
          (option['capability'] == 'power' && option['action'] == 'set')
          ? 'state'
          : (option['capability'] == 'vendor.xiaomi.mijia.fan' &&
                const {
                  'level',
                  'mode',
                  'horizontal_swing',
                  'horizontal_angle',
                  'horizontal_swing_included_angle_6',
                  'vertical_swing_7',
                  'vertical_swing_included_angle_8',
                  'anion_9',
                }.contains(featureParameter) &&
                option['action'] == 'set_$featureParameter')
          ? featureParameter as String
          : null;
      final settingParameter =
          _referenceSettings[(
            component,
            option['capability'],
            option['action'],
          )];
      final outlet =
          _outletSettings[(component, option['capability'], option['action'])];
      // The shared physical-lock identifier uses separate risk and startup families.
      final isOutlet =
          outlet != null &&
          !(settingParameter != null && option['definition']?['risk'] == 'low');
      final mapping = isOutlet
          ? ('mijia_outlet_grants', outlet.$1)
          : settingParameter != null
          ? ('mijia_setting_grants', settingParameter)
          : fanComponent
          ? (fanParameter == null ? null : ('mijia_fan_grants', fanParameter))
          : _startup[(option['capability'], option['action'])] ??
                (option['capability'] == 'vendor.midea.ac' &&
                        featureParameter is String &&
                        option['action'] == 'set_' + featureParameter
                    ? ('meiju_ac_feature_grants', featureParameter)
                    : null);
      final definition = option['definition'] as Map?;
      final risk = definition?['risk'];
      if (mapping == null ||
          (component != 'main' &&
              !fanComponent &&
              settingParameter == null &&
              !isOutlet) ||
          (isOutlet
              ? risk != outlet.$2 ||
                    (outlet.$2 == 'elevated' && !trustedApproval)
              : risk != 'low') ||
          definition?['idempotency'] != 'set_value' ||
          !(definition?['transports'] as List? ?? []).contains('cloud'))
        continue;
      if (isOutlet &&
          (definition?['transports'] is! List ||
              (definition!['transports'] as List).length != 1 ||
              definition['atomic'] != false ||
              definition['independent_fields'] != false ||
              definition['verification'] != 'unavailable'))
        continue;
      final parameters = definition?['parameters'] as Map?;
      final properties = parameters?['properties'] as Map?;
      if (parameters?['type'] != 'object' ||
          properties?.length != 1 ||
          properties?[mapping.$2] is! Map ||
          (parameters?['required'] as List? ?? []).any((p) => p != mapping.$2))
        continue;
      final schema = properties![mapping.$2] as Map;
      final auxiliary =
          settingParameter != null &&
          const {'light-6', 'function-3'}.contains(component);
      if (auxiliary &&
          schema['type'] !=
              (mapping.$2 == 'brightness_2' ? 'integer' : 'boolean'))
        continue;
      if (isOutlet &&
          schema['type'] !=
              (mapping.$2 == 'default_power_on_state_2'
                  ? 'integer'
                  : 'boolean'))
        continue;
      if (fanComponent &&
          (const {
                'state',
                'horizontal_swing',
                'vertical_swing_7',
                'anion_9',
              }.contains(mapping.$2)
              ? schema['type'] != 'boolean'
              : !const {'integer', 'number'}.contains(schema['type'])))
        continue;
      if (parameters!['additionalProperties'] != false ||
          parameters.keys.any(
            (key) => !{
              'type',
              'properties',
              'required',
              'additionalProperties',
              'description',
              'title',
            }.contains(key),
          ) ||
          (parameters['required'] as List? ?? []).length != 1 ||
          schema.keys.any(
            (key) => !{
              'type',
              'enum',
              'minimum',
              'maximum',
              'multipleOf',
              'description',
              'title',
            }.contains(key),
          ))
        continue;
      bool valid(Object value) {
        if (auxiliary &&
            mapping.$2 == 'brightness_2' &&
            !const [0, 1, 2].contains(value))
          return false;
        if (isOutlet &&
            mapping.$2 == 'default_power_on_state_2' &&
            !const [0, 1, 2].contains(value))
          return false;
        if (!(switch (schema['type']) {
          'boolean' => value is bool,
          'string' => value is String,
          'number' => value is num && value.isFinite,
          'integer' =>
            value is num && value.isFinite && value == value.roundToDouble(),
          _ => false,
        }))
          return false;
        if (value is num) {
          if (schema['minimum'] is num && value < schema['minimum'])
            return false;
          if (schema['maximum'] is num && value > schema['maximum'])
            return false;
          if (schema['multipleOf'] case final num step) {
            if (!step.isFinite ||
                step <= 0 ||
                value / step != (value / step).roundToDouble())
              return false;
          }
        }
        return true;
      }

      final values = <Object>[];
      if (schema['enum'] is List) {
        for (final value in schema['enum'] as List) {
          if (value is String || value is bool || value is num)
            values.add(value);
        }
      } else if (schema['type'] == 'boolean') {
        values.addAll([false, true]);
      } else if (schema['type'] == 'number' || schema['type'] == 'integer') {
        final min = schema['minimum'],
            max = schema['maximum'],
            step = schema['multipleOf'];
        if (min is! num ||
            max is! num ||
            step is! num ||
            !min.isFinite ||
            !max.isFinite ||
            !step.isFinite ||
            step <= 0 ||
            max < min ||
            (max - min) / step > (settingParameter != null ? 4095 : 255))
          continue;
        for (var i = 0; i <= ((max - min) / step).floor(); i++) {
          final value = min + i * step;
          // Existing reviewed targets use exactly representable half steps.
          if (value / step != (value / step).roundToDouble()) continue;
          values.add(value);
        }
      }
      if (values.length >
          (mapping.$1 == 'mijia_setting_grants'
              ? 4096
              : mapping.$1 == 'meiju_temperature_grants' ||
                    mapping.$1 == 'meiju_ac_feature_grants' ||
                    mapping.$1 == 'mijia_fan_grants'
              ? 256
              : 64))
        continue;
      for (final value in values.where(valid)) {
        result.add(
          SessionPermission._(
            inspection['device_id'],
            component as String,
            option['capability'],
            option['action'],
            mapping.$2,
            value,
            mapping.$1,
          ),
        );
      }
    }
    return List.unmodifiable(result);
  }

  /// Host configuration only; all writes still consume a trusted native preview.
  static List<SessionPermission> _dishwasherChoices(String device, Map option) {
    final action = option['action'];
    if (option['component_id'] != 'main' ||
        !{
          'start_cycle',
          'pause_cycle',
          'resume_cycle',
          'cancel_cycle',
          'power_on',
          'power_off',
        }.contains(action))
      return [];
    final definition = option['definition'];
    if (definition is! Map ||
        definition['risk'] != 'elevated' ||
        definition['idempotency'] != 'non_idempotent' ||
        definition['atomic'] != false ||
        definition['independent_fields'] != false ||
        definition['verification'] != 'unavailable' ||
        definition['transports'] is! List ||
        (definition['transports'] as List).length != 1 ||
        (definition['transports'] as List).single != 'cloud')
      return [];
    final schema = definition['parameters'];
    if (schema is! Map ||
        schema['type'] != 'object' ||
        schema['additionalProperties'] != false ||
        schema.keys.any(
          (key) => !{
            'type',
            'properties',
            'required',
            'additionalProperties',
            'description',
            'title',
          }.contains(key),
        ))
      return [];
    final properties = schema['properties'];
    final required = schema['required'] ?? const [];
    if (properties is! Map ||
        required is! List ||
        required.length != required.toSet().length ||
        required.length != properties.length ||
        required.any((name) => !properties.containsKey(name)) ||
        properties.keys.any(
          (name) =>
              name != 'additional' &&
              !(name == 'program' && action == 'start_cycle'),
        ) ||
        (action == 'start_cycle' && !properties.containsKey('program')))
      return [];
    var combinations = <Map<String, Object>>[{}];
    // Stable key ordering makes exact-grant identities independent of JSON order.
    for (final name in ['program', 'additional']) {
      if (!properties.containsKey(name)) continue;
      final parameter = properties[name];
      if (parameter is! Map ||
          parameter['type'] != (name == 'program' ? 'string' : 'integer') ||
          parameter.keys.any(
            (key) => !{'type', 'enum', 'description', 'title'}.contains(key),
          ))
        return [];
      final values = parameter['enum'];
      if (values is! List ||
          values.isEmpty ||
          values.length > 64 ||
          values.length != values.toSet().length ||
          values.any(
            (value) => name == 'program'
                ? value is! String || value.trim().isEmpty
                : value is! int,
          ) ||
          combinations.length * values.length > 64)
        return [];
      combinations = [
        for (final args in combinations)
          for (final value in values) {...args, name: value as Object},
      ];
    }
    return [
      for (final args in combinations)
        SessionPermission._(
          device,
          'main',
          'vendor.midea.dishwasher',
          action as String,
          'arguments',
          Map<String, Object>.unmodifiable(args),
          action == 'start_cycle'
              ? 'meiju_dishwasher_grants'
              : 'meiju_dishwasher_cycle_grants',
        ),
    ];
  }

  static Map<String, dynamic> startup(List<SessionPermission> permissions) {
    final fields = <String, dynamic>{};
    final seen = <String>{};
    for (final permission in permissions) {
      if (!seen.add(permission.identity)) continue;
      final entries =
          fields.putIfAbsent(
                permission.startupField,
                () => <Map<String, dynamic>>[],
              )
              as List;
      entries.add(permission.startupEntry);
      final limit = permission.startupField == 'mijia_setting_grants'
          ? 4096
          : permission.startupField == 'meiju_temperature_grants'
          ? 256
          : permission.startupField == 'meiju_ac_feature_grants' ||
                permission.startupField == 'mijia_fan_grants'
          ? 1024
          : 64;
      if (entries.length > limit)
        throw StateError('Too many exact session permissions.');
    }
    return fields;
  }
}
