# ADR-0074: Exact profiles from the October 22 Meiju reference

Status: accepted for source integration; physical acceptance remains pending.

## Decision

The owner requests integration of the supplied Meiju 11.7.10.6 analysis. Its
hash-pinned six-device contract is protocol evidence, not executable source,
authority, a login session or a claim of physical compatibility. Extract only
reviewed public product selectors and scalar facts; never execute bundled code.

The four exact wall-AC SN8/model pairs take precedence over the older community
category default. Conflicting model, category or portable-subtype evidence refuses
optional control admission. Their candidate optional catalog contains only
model-listed functions with reviewed independent encodings. `cool_energy_saving`
uses `cool_power_saving` 0/1, `strong_cooling` uses `strong_cool` 0/1, and
`light_sensing` uses `light_sensitive` 0/3. These are distinct from ECO, comfort
saving and boost. The existing exact grants, current-operation prerequisite
reads, reported-buzzer preservation and one-send outcome rules apply.

Do not reuse a generic paired ECO payload, auxiliary-heat/default/force behavior,
buzzer setting, or coupled anti-draft setter for a newly known exact profile.
Unsupported setters are omitted even if old saved descriptors contain them;
execution rechecks protected profile evidence independently.

Protected refrigerator product 310A2111, subtype 56 and the exact reviewed model
admit a third read-only `variable` component and a separate read-only
`vendor.midea.refrigerator.details` capability. Other products do not inherit
these fields. New reports remain observations: generic Lua responses can include
fields irrelevant to the physical appliance. Reported saving-star/ice-maker
states do not settle the current-cloud versus old-cache configuration conflict.
No mode label, default temperature or requested value becomes a reading. No
refrigerator write is admitted. Older descriptors omit the separate detail scope
until an explicit inventory refresh; device and connection identities stay fixed.

## Boundaries and compatibility

API 1.26 adds three feature names to the existing exact startup feature/value
union. Ordinary methods, trusted C functions, JSON envelope, SQLite schema 5,
protected payload 3, identity, routing and execution outcomes are unchanged.
Older libraries reject the new names. Known exact profiles intentionally expose
fewer optional actions than the historical category union; stale actions cannot
bypass the protected selector. Other reviewed community product overrides remain
unchanged. Foreground state-read scheduling is unchanged.

The new MAS routes use a different authentication/signing contract whose
material is excluded from the supplied package. Existing admitted community
material is not assumed equivalent. Modern TSL, new aggregate inventory,
statistics, history, configuration, resource downloads and vendor-service
requests remain blocked on an independently admitted authentication path.
The GX1200Max 760064BA firmware-1 TSL program/action conversion is not the
existing five legacy dishwasher profiles. No legacy grant or program number
implicitly admits it. Empty settingInfo, missing source resources, conditional
settings and unresolved response semantics remain explicit implementation gaps.

## Validation

Require archive/catalog reproducibility, all four profile pairs, wrong model and
subtype/category refusals, exact scalar bodies, missing/malformed prerequisite
refusal, acceptance/uncertainty/replay, three-zone separation, old-descriptor
behavior and malformed-read cache preservation. Flutter must render the new
controls and separate compartment from descriptors, including localized labels,
without brand/model branching. Packaging and real-device acceptance are reported
separately from fictional tests.
