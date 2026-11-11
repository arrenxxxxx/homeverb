# GX1200Max dishwasher parity

Owner request: November 11, 2026. Target: the appliance experience in Meiju,
including usable commands and understandable observations. **Parity is not
complete.** This first phase implements exact-profile read presentation, not
current-model control admission.

## Evidence and current implementation

The pinned October 22 [reference archive](13-meiju-reference-integration.md)
identifies GX1200Max's product 760064BA and subtype 3. Its extracted official
plugin contract has source SHA-256
`47a3107ab5ae32e166f3ff486ef62c6bf3b1b6b445fe605344205f79c46ad9a5`.
Both recovered firmware branches have the same fourteen program values. The
reference's observed branch 1 and protocol version 51 are historical evidence;
HomeVerb must not copy them into another session as current device facts.

The new read-only `vendor.midea.dishwasher.details` capability requires all three
protected selectors: product, subtype and exact model. It is published on `main`
by explicit inventory refresh. Existing read descriptors remain usable and do
not receive undeclared fields. A successful read or this descriptor never adds
an action or a startup grant. The five legacy command profiles are unchanged.

| Observation | Reviewed source and behavior |
| --- | --- |
| Program | The fourteen official `modeList` tokens and the converter's `none` token have localized names; unfamiliar tokens remain in the existing raw capability and do not acquire an invented name |
| Cycle state | `work_status` / `base_status`: washing, pause, drying, storage, reservation, standby, power off, firmware update, drainage and ventilation; contradictory aliases reject the snapshot |
| Door | Only `base_door_on` is interpreted as the plugin's open-door flag; the older `doorswitch` integer is not assigned that polarity |
| Temperature | `base_cur_temp`, only when actually returned |
| Duration | `base_mode_time` is an estimate in standby and remaining minutes only for the reviewed running/storage/reservation phases; power-off/unknown/fault never becomes a running countdown |
| Reported settings | Actual storage/cyclic-storage flags, smart dosing, lower-zone boost and child lock; supported-option sentinel 3 remains absent, not false |

Program choices and setting readings are distinct. The App shows the new reported
names and phases on the detail page and home card, with read-only setting rows.
There are no fake selectable programs or switches on the unsupported control
profile. Missing fields retain the existing loading/dash behavior. Raw salt,
rinse-aid and water codes still require reviewed display semantics. All existing
cloud readings retain unavailable physical observation time and `stale: true`.

This phase reuses the existing bounded Lua-state request. It does not adopt TSL,
assume that every plugin-generated field is returned by that endpoint, or create
sample values for absent fields. Live field coverage after inventory refresh
remains unverified. The comparator reads only pinned data/source text; it does
not execute bundled JavaScript, Python or Lua.

```sh
python3 scripts/verify_dishwasher_read_mapping.py
python3 -m unittest discover -s tests -p test_dishwasher_read_mapping.py -v
cargo test -p homeverb-midea --offline --locked dishwasher_readings
```

## Remaining Meiju parity work

| User journey | Current result | Next concrete dependency |
| --- | --- | --- |
| View program, phase and time | New exact-profile decoder and localized display; fictional tests | Confirm which fields the current Lua endpoint returns; obtain bounded TSL response fixtures if necessary |
| Power on/off | Current model still has no command profile | Validate the selected device's Lua/TSL branch and fixed endpoint request/acknowledgement contract; use the existing trusted approval boundary |
| Choose and start a wash | Static fourteen-program catalog recovered; no commands admitted | Query/review current program-option constraints and firmware evidence, then implement exact arguments and trusted previews |
| Pause/resume/cancel | Official state-dependent TSL action names recovered | Distinguish `cancelWashing`, `cancelStorage`, `cancelDry`, `cancelAppointing` and `cancelVentilation`; do not substitute the legacy generic cancel command |
| Layered/boost wash, dosing, storage | Reported values only where explicitly returned | Review per-program supported flags, conflicting options and preservation; branch-specific storage behavior is unresolved |
| Salt/rinse aid, filter and detergent | Existing raw warnings only | Complete exact-model service association, response units and maintenance-reset effects; never turn warning codes into percentages |
| Last wash and monthly usage | Historical source summaries only | Implement authenticated, bounded provider report requests and complete response fixtures; do not display archive readings as live data |

The reference contains converter excerpts but omits raw provider responses and
original plugin/Lua resources. The current transport signs fixed JSON POSTs;
TSL uses additional GET/PUT/action routes and query/header conventions that have
not passed HomeVerb session compatibility tests. The previous two successful
modern POST probes do not validate those routes. These are concrete integration
gaps, not a need to relax command authorization or a reason to copy another
product profile. No real appliance write is authorized by this development phase.

Exact executed checks and remaining environment failures are recorded in the
[App verification record](../../apps/mobile/verification.md#november-11-gx1200max-read-presentation).
