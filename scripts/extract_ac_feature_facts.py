#!/usr/bin/env python3
"""Extract reviewed scalar protocol facts from the pinned public AC mapping.

The input is parsed as AST, never imported or executed. Review additions to the
allowlist and the generated JSON together; this is not a runtime plugin loader.
"""
import argparse
import hashlib
import ast
import json
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('source', type=Path)
args = parser.parse_args()
source = args.source.read_bytes()
if hashlib.sha256(source).hexdigest() != 'b6a2b9e5b170c22dc838d1de93edbe512f969fae75f170b8ea053a4974a450db':
    raise SystemExit('Expected the reviewed ce0c4b88224def0f6b53714312c19f71aa9c2a50 mapping.')
tree = ast.parse(source.decode('utf-8'))
root = next(n.value for n in tree.body if isinstance(n, ast.Assign))

def literal(node):
    if isinstance(node, ast.Constant):
        return node.value
    if isinstance(node, (ast.List, ast.Tuple)):
        return [literal(x) for x in node.elts]
    if isinstance(node, ast.Dict):
        return {str(literal(k)): literal(v) for k, v in zip(node.keys, node.values)}
    if isinstance(node, ast.Attribute):
        return node.attr
    if isinstance(node, ast.Name):
        return node.id
    return None

aliases = {
    'wind_swing_lr': 'horizontal_swing', 'wind_swing_ud': 'vertical_swing',
    'wind_swing_lr_under': 'lower_horizontal_swing',
    'wind_swing_lr_angle': 'horizontal_direction', 'wind_swing_ud_angle': 'vertical_direction',
    'no_wind_sense': 'breezeless', 'screen_display': 'display', 'buzzer': 'button_sound',
    'prevent_straight_wind': 'avoid_direct_airflow', 'prevent_super_cool': 'prevent_overcooling',
    'dry': 'dry_after_shutdown', 'ptc': 'auxiliary_heat', 'aux_heat': 'auxiliary_heat',
    'follow_body_sense': 'follow_airflow', 'strong_wind': 'boost',
    'comfort_power_save': 'comfort_energy_saving', 'energy_save': 'sleep',
}
excluded = {'22012895', '21293193', '17497071', '10693145', '106J6363',
            '26093137', '26093139', '26096947', '22012227', '22013167', '22013285', '22251749'}
profiles = []
skipped = []
for key_node, value_node in zip(root.keys, root.values):
    key = literal(key_node)
    codes = key if isinstance(key, list) else [key]
    portable = codes == ['subtype', 524]
    wall = key == 'default_wall_air_conditioner'
    if not portable and not wall and (not all(isinstance(c, str) and len(c) == 8 for c in codes) or any(c in excluded for c in codes)):
        continue
    cfg = literal(value_node)
    entities = cfg.get('entities', {})
    features = {}

    def add(wire, choices, read_field=None, command=None, schema=None):
        if not choices:
            return
        name = aliases.get(wire, wire)
        options = []
        for value, control in choices:
            options.append({'value': value, 'control': {**(command or {}), **control}})
        values = [v['value'] for v in options]
        typ = 'boolean' if all(isinstance(v, bool) for v in values) else 'integer' if all(isinstance(v, int) for v in values) else 'string'
        features[name] = {'name': name, 'wire': wire, 'read_field': read_field or wire,
                          'parameter': schema or {'type': typ, **({} if typ == 'boolean' else {'enum': values})}, 'options': options}

    thermo = entities.get('CLIMATE', {}).get('thermostat', {})
    presets = thermo.get('preset_modes', {})
    for preset, wire in [('eco', 'eco'), ('comfort', 'comfort_power_save'), ('boost', 'strong_wind'), ('sleep', 'energy_save')]:
        on = presets.get(preset)
        off = {k: v for k, v in presets.get('none', {}).items() if on and k in on}
        if on and off and wire in on:
            add(wire, [(False, off), (True, on)])
    swings = list(thermo.get('swing_modes', {}).values())
    for wire in ['wind_swing_lr', 'wind_swing_ud', 'wind_swing_lr_under']:
        if any(s.get(wire) == 'on' for s in swings) and any(s.get(wire) == 'off' for s in swings):
            add(wire, [(False, {wire: 'off'}), (True, {wire: 'on'})])
    for wire, config in entities.get('SWITCH', {}).items():
        if wire in {'power', 'prepare_food', 'quick_fry', 'remove_arofene'}:
            continue
        if config.get('condition') or config.get('include_current') or config.get('merge_attributes') or wire in {'self_clean', 'disinfect'}:
            skipped.append((key, wire))
            continue
        pair = config.get('rationale', cfg.get('rationale', ['off', 'on']))
        if len(pair) != 2:
            continue
        field = config.get('attribute', wire)
        add(field, [(False, {field: pair[0]}), (True, {field: pair[1]})], config.get('state_attribute'), config.get('command'))
    for wire, config in entities.get('SELECT', {}).items():
        if config.get('condition'):
            skipped.append((key, wire))
            continue
        controls = list(config.get('options', {}).values())
        if not controls or any(wire not in c for c in controls):
            skipped.append((key, wire))
            continue
        raw_values = [c[wire] for c in controls]
        if len(set(raw_values)) != len(raw_values):
            skipped.append((key, wire))
            continue
        add(wire, list(zip(raw_values, controls)), config.get('query', wire))
    for wire, config in entities.get('NUMBER', {}).items():
        if wire == 'quick_prepare_food_angle':
            continue
        if config.get('condition'):
            skipped.append((key, wire))
            continue
        low, high, step = config.get('min'), config.get('max'), config.get('step', 1)
        if not all(isinstance(v, int) for v in [low, high, step]) or step <= 0 or (high-low)//step > 255:
            skipped.append((key, wire))
            continue
        name = aliases.get(wire, wire)
        features[name] = {'name':name, 'wire':wire, 'read_field':wire,
                          'parameter':{'type':'integer', 'minimum':low, 'maximum':high, 'multipleOf':step}, 'options':[]}
    # The climate auxiliary-heat field is reviewed even if no separate switch exists.
    if 'auxiliary_heat' not in features and thermo.get('aux_heat'):
        wire = thermo['aux_heat']
        add(wire, [(False, {wire:'off'}), (True, {wire:'on'})])
    profiles.append({'selector': 'portable' if portable else 'wall' if wall else 'product',
                     'codes': [] if portable or wall else codes,
                     'queries': [q['query_type'] for q in cfg.get('queries', []) if q and 'query_type' in q],
                     'preserve': cfg.get('centralized', []), 'features': list(features.values())})

output = Path(__file__).resolve().parents[1] / 'crates/midea/src/ac_features.json'
output.write_text(json.dumps(profiles, indent=2, ensure_ascii=True) + '\n')
print([(p['selector'], p['codes'], [f['name'] for f in p['features']]) for p in profiles])
print('Unreviewed coupled/conditional controls:', skipped)
