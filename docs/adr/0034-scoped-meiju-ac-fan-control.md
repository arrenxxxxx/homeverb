# ADR-0034: Admit explicit vendor AC fan-speed controls

Status: accepted for offline implementation; owner-deferred physical tests remain pending.

## Evidence and scope

The pinned Midea Auto Cloud revision `ce0c4b88224def0f6b53714312c19f71aa9c2a50`
maps default AC fan presets to `wind_speed` values 20, 40, 60, 80, 100 and 102.
Its `climate.py` fan setter sends only the selected mapping through the same cloud
control endpoint used for power. Source labels are silent, low, medium, high, full
and auto, respectively. They are vendor presets, not percentages or universal fan
semantics. Other upstream profiles differ, and mode-dependent effectiveness is
not established for the owner's devices.

The source cloud method also accepts optional `command.status`, and its ordinary
entity/device path supplies current cached state as that context. HomeVerb omits
it to avoid copying stale values into a control request. Source inspection does
not prove that the vendor Lua path preserves other settings without this context;
physical acceptance must check power, mode and temperature preservation as well as
the requested fan setting before promoting model support.

Admit only these six codes experimentally for the four exact AC model strings
already admitted for reads and power. Keep family and protected model checks at
execution. This is a scoped candidate mapping, not model/firmware certification;
no other AC, refrigerator or dishwasher gains this control. The adapter must be
explicitly opted in. Default hosts stay read-only. Temperature ranges and further
settings require separate evidence.

## Decision

Expose `main / vendor.midea.ac.set_fan_speed` with exactly one integer argument,
`wind_speed`, restricted to the six reviewed codes. Keep the existing readable
vendor property permissive: an observed different code is not writable merely
because it was read. Do not map the action to a universal fan-speed capability.

Send exactly `command.control: {"wind_speed": <code>}` beside the protected
appliance target. Do not include power, mode, temperature, stale state or a second
command. In off/auto/dry or other device-managed states the setting may not take
effect; do not power on or change mode to force it. Code-zero acknowledgement is
accepted, not verified. Preserve the existing conservative unknown/rejection,
deadline, cancellation, no-retry, no-fallback and receipt-replay rules.

Native API 1.9.0 adds optional trusted startup `meiju_fan_grants`, at most 64 unique
`{device_id, wind_speed}` pairs. Each permission covers one canonical local UUID v4,
one reviewed vendor code, `main`, that action and cloud only. Power grants remain
independent. Neither permission enables the other action. Startup validates all
grants before opening a profile and performs no account or network work. The host
must obtain explicit user consent before supplying permissions; ordinary JSON
requests, tool input and device metadata cannot edit authority.

## Compatibility and verification

Older libraries reject the new startup field. Negotiate API 1.9 on an ordinary
read-only handle before using it. Existing startup configuration, 30 methods, seven
C functions, JSON envelopes and SQLite schema 4 retain their behavior. Grants are
immutable for a session and never stored as authority in SQLite. Reopening without
fan grants refuses fan control even if cached descriptors were writable.

Verify exact wire fields for all admitted codes, argument/model/binding refusal,
permission separation, unsupported appliances, ordinary-request rejection, no
startup account/network work, accepted/unknown/rejected outcomes, no state/cache side effects,
replay and lifecycle cancellation. Use fictional storage/transports and the Linux
C ABI; keep Android library build evidence separate from phone acceptance. Update
third-party attribution. No real control or owner-account mutation is authorized
by implementing this ADR.
