# ADR-0115: Bounded reported Mijia event occurrences

Status: Accepted; typed protocol implemented; Core/native/App adoption pending.
Date: October 30, 2026.

## Evidence

The public [plugin SDK documentation](https://github.com/MiEcosystem/miot-plugin-sdk/blob/a48234baf2f3e7faba09c6c7bd18efe7197841bf/miot-sdk/service/smarthome.js)
explicitly documents getDeviceData for /user/get_user_device_data with type=event
and an MIoT key of siid.eiid. It distinguishes these keys from decimal BLE object
IDs and custom Wi-Fi names. The documented request contains did, key, type,
time_start/time_end in epoch seconds and a bounded limit; raw grouping is default.
The smarthome.js SHA-256 is
a4f056c43d2c485703eef6d043c74bb22791f11081a24cb594f2f05ed9b5b418.
Only interface facts are used; no SDK implementation or comments are copied.
The public snapshot is not a live provider/retention guarantee.

The previously pinned Apache community client revision
f78158257e19a21902adaf6bcc8440c3ba172a91 supplies the existing request envelope and
result-record channel. The digest-pinned October 22 archive supplies 78 exact
model/spec/service/event definitions. This closes the event-key evidence gap for
this distinct history route; it does not establish /miotspec/event/get, MQTT push,
BLE object history, a continuation cursor or payload interpretation.

## Typed phase

Project all 78 public exact definitions into a separate event catalog. The typed
query selects an existing model/spec/SIID/EIID tuple, never a raw key, URL or
user-selected UID. Request one page of at most 31 records in an explicit positive
half-open UTC window of at most 32 days, with no future end. No automatic page,
retry, relogin, renewal, timer, subscription or background worker is introduced.

Use the already reviewed legacy xiaomiio history origin/codec or the separate
modern mijia community-history origin/per-field codec. Do not use the reference
continuous-codec origin, cross session credentials or switch protocols. Modern
entries require matching protected subject/encrypted subject, allowed logical
kind and valid exact saved mapping. The later adapter phase must independently
validate protected account/device/root membership, lifecycle and read policy.

The result projects only reported occurrence timestamps. Require every object
record's did/key/type to match the selected request; a missing or foreign
correlation refuses the page. Missing values and malformed/out-of-window times
are counted as rejected. Provider value, actor, identifier, media URL and opaque
payload fields are never returned, logged, stored or interpreted. A value field
only establishes that the row has the expected record shape; its contents have
no admitted semantics. Preserve equal timestamps as separate reported rows:
one-second timestamps do not prove identity, uniqueness or physical event count.
Sort by reported time without merging or deduplicating occurrences.

Completeness remains unproven, including empty results. A full page sets
limit_reached but does not establish more data or supply a cursor. Occurrence
history is not current state, an alarm service, action verification, physical
causality or a guarantee of event delivery. In particular, a reported lock/fire/
water event cannot be used to infer current safety or operation state.

## Compatibility and remaining work

This phase adds typed Rust session methods only. Native API 1.43, ordinary JSON,
CLI/MCP, App, stored descriptors, SQLite 5 and protected payload 3 are unchanged.
The unreleased Rust CloudEndpoint and ModernReadEndpoint enums gain EventHistory;
exhaustive downstream matches must handle it. The 85fefd8 APK does not acquire
event history from this phase. All 78 events
still require Core/Host/App adoption; argument-bearing events additionally need
reviewed payload semantics. Full pagination, retention and live subscription
remain unresolved. Do not label the reference event requirement complete.

Validate every catalog tuple and both-protocol requests with fictional
transports, exact correlation, response/count/window bounds, malformed envelopes,
private-field omission, equal timestamps, auth failure, cancellation/deadline/drop
and no retry. Real account/provider/phone/model acceptance remains pending.
