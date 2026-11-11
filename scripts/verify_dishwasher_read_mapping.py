#!/usr/bin/env python3
"""Check read-only GX1200Max facts against the pinned, unevaluated plugin extract."""
import hashlib
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE = ROOT / 'docs/adapter-references/2026-10-22/meiju-11.7.10.6-analysis-20261022.zip'
SHA256 = 'fa4d5c978b7a7b0197f64c7cbd8badff49f57ecf83034e1f6ae93c038831e453'
MEMBER = 'meiju-11.7.10.6-analysis/analysis/runtime/waydroid/account-cache/dishwasher-protocol-contract.json'


def verify(source, capability):
    assert source['sn8'] == '760064BA'
    assert source['source']['sha256'] == '47a3107ab5ae32e166f3ff486ef62c6bf3b1b6b445fe605344205f79c46ad9a5'
    props = capability['properties']
    assert not capability['actions']
    assert all(p['readable'] and not p['writable'] for p in props.values())
    # Both recovered branches agree on program names. No firmware identity is
    # synthesized from the reference's historical runtime branch.
    for branch in source['firmware_settings']:
        programs = [p['value'] for p in branch['settings']['modeList']]
        assert props['program']['enum'] == programs + ['none']
    methods = {item['name']: item['source'] for item in source['converter_methods']}
    converter = methods['convertThingToLua']
    for state in props['cycle_state']['enum']:
        assert f'o2.work_status = "{state}", o2.base_status = "{state}"' in converter
    for fragment in [
        '1 == e2[t3] ? o2[i2] = 1 : 2 == e2[t3] && (o2[i2] = 0)',
        'o2.more_keep_device = e2[t3] ? 1 : 0',
        'o2.more_cycle_keep_device = e2[t3] ? 1 : 0',
        'o2[i2] = e2[`${t3}Supported`] ? e2[t3] ? 1 : 0 : 3',
        '[8].includes(e2["dishwasher.workStatus"])',
        '[5, 6, 3, 11, 15].includes(e2["dishwasher.workStatus"])',
        'o2[i2] = 60 * e2[t3]',
    ]:
        assert fragment in converter
    assert '!!this.deviceStatus.base_door_on' in methods['checkStatus']
    mapping = source['mappings']['_t']['value']
    for name, wire in {
        'doorState': 'base_door_on', 'currentWashingTemperature': 'base_cur_temp',
        'remainingRunningTime': 'base_mode_time', 'washTime': 'base_mode_time',
        'smartDose': 'more_auto_throw', 'boostLowerZone': 'more_region_reinforce',
    }.items():
        assert mapping['dishwasher.' + name] == wire
    assert mapping['childLock.childLockSwitch'] == 'lock_on'
    assert props['door_state']['enum'] == ['open', 'closed']


def main():
    assert hashlib.sha256(ARCHIVE.read_bytes()).hexdigest() == SHA256
    with zipfile.ZipFile(ARCHIVE) as archive:
        source = json.loads(archive.read(MEMBER))
    capability = json.loads((ROOT / 'examples/capability.meiju-dishwasher-details.json').read_text())
    verify(source, capability)
    print('PASS: GX1200Max read mappings, both program catalogs, phase/time and flag semantics; no source execution or network.')


if __name__ == '__main__':
    main()
