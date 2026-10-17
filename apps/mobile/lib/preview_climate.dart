import 'package:flutter/material.dart';
import 'client_contract.dart';
import 'climate_controls.dart';
import 'controller.dart';
import 'preferences.dart';
import 'skin.dart';

/// Fictional presentation data. This client cannot open Core or send commands.
class _PreviewClient implements CoreClient {
  @override
  Future<dynamic> call(String method, [Json fields = const {}]) async =>
      throw UnsupportedError('The visual preview has no native operations.');
  @override
  void foreground(bool value) {}
  @override
  void cancelAll() {}
  @override
  Future<void> close() async {}
}

class PreviewClimate extends StatefulWidget {
  const PreviewClimate({required this.onAction, super.key});
  final void Function(String) onAction;
  @override
  State<PreviewClimate> createState() => _PreviewClimateState();
}

class _PreviewClimateState extends State<PreviewClimate> {
  final c = HomeController(_PreviewClient(), LocalPreferences.memory());
  String airflow = 'positions';

  @override
  void initState() {
    super.initState();
    configure();
  }

  Json capability(String id, Map<String, (Json, Object)> fields) => {
    'schema_version': '1.0',
    'id': id,
    'revision': 1,
    'properties': {
      for (final field in fields.entries)
        field.key: {...field.value.$1, 'readable': true, 'writable': true},
    },
    'actions': {
      for (final field in fields.entries)
        'set_${field.key}': {
          'parameters': {
            'type': 'object',
            'properties': {
              field.key: {
                for (final entry in field.value.$1.entries)
                  if (entry.key != 'unit') entry.key: entry.value,
              },
            },
            'required': [field.key],
            'additionalProperties': false,
          },
          'transports': ['lan'],
          'risk': 'low',
          'idempotency': 'set_value',
          'atomic': true,
          'verification': 'unavailable',
        },
    },
    'events': {},
  };

  void configure() {
    final fields = <String, Map<String, (Json, Object)>>{
      'climate': {
        'target_temperature': (
          {
            'type': 'number',
            'minimum': 16,
            'maximum': 30,
            'multipleOf': 0.5,
            'unit': 'celsius',
          },
          24,
        ),
        'mode': (
          {
            'type': 'string',
            'enum': ['auto', 'cool', 'heat', 'dry', 'fan'],
          },
          'cool',
        ),
      },
      'power': {
        'state': ({'type': 'boolean'}, true),
      },
      'fan': {
        'speed': (
          {'type': 'integer', 'minimum': 1, 'maximum': 5, 'multipleOf': 1},
          3,
        ),
        if (airflow != 'none') ...{
          'horizontal_swing': ({'type': 'boolean'}, false),
          'vertical_swing': ({'type': 'boolean'}, true),
        },
        if (airflow == 'positions') ...{
          'horizontal_direction': (
            {'type': 'integer', 'minimum': 0, 'maximum': 4, 'multipleOf': 1},
            2,
          ),
          'vertical_direction': (
            {'type': 'integer', 'minimum': 0, 'maximum': 5, 'multipleOf': 1},
            3,
          ),
        },
      },
      'vendor.homeverb.preview': {
        'display_light': ({'type': 'boolean'}, true),
        'button_sound': ({'type': 'boolean'}, false),
        'sleep': ({'type': 'boolean'}, false),
      },
    };
    final capabilities = [
      for (final entry in fields.entries) capability(entry.key, entry.value),
    ];
    capabilities.first['properties']['current_temperature'] = {
      'type': 'number',
      'readable': true,
      'writable': false,
      'unit': 'celsius',
    };
    const device = 'preview-climate';
    c.selectedDevice = device;
    c.description = {
      'device': {
        'id': device,
        'components': [
          {'id': 'main', 'capabilities': fields.keys.toList()},
        ],
      },
      'capabilities': capabilities,
    };
    final now = DateTime.now().toUtc().toIso8601String();
    c.snapshot = {
      'observations': {
        for (final entry in fields.entries)
          for (final field in entry.value.entries)
            '${entry.key}.${field.key}': {
              'value': field.value.$2,
              'received_at': now,
              'stale': false,
            },
        'climate.current_temperature': {
          'value': 26,
          'received_at': now,
          'stale': false,
        },
      },
    };
  }

  Future<bool> simulate(
    String capability,
    String action,
    Json arguments,
  ) async {
    setState(() {
      for (final entry in arguments.entries) {
        c.snapshot!['observations']['$capability.${entry.key}']['value'] =
            entry.value;
      }
    });
    return true;
  }

  @override
  void dispose() {
    c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      Wrap(
        spacing: 12,
        runSpacing: 8,
        crossAxisAlignment: WrapCrossAlignment.center,
        children: [
          const Copy('Preview only', size: 12, color: muted),
          DropdownButton<String>(
            key: const ValueKey('preview-airflow'),
            value: airflow,
            items: [
              for (final option in const {
                'positions': 'Air direction',
                'swing': 'Horizontal swing',
                'none': 'Unavailable',
              }.entries)
                DropdownMenuItem(
                  value: option.key,
                  child: Copy(option.value, size: 12),
                ),
            ],
            onChanged: (value) {
              if (value == null) return;
              setState(() {
                airflow = value;
                configure();
              });
            },
          ),
        ],
      ),
      const SizedBox(height: 8),
      ClimateControls(
        key: ValueKey('preview-climate:$airflow'),
        controller: c,
        onAction: widget.onAction,
        onExecute: simulate,
      ),
    ],
  );
}
