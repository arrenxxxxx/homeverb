# 7. Preview and authorize one Meiju AC power control

Status: experimental implementation with synthetic Core and local TLS tests. No
real control has been sent or physically verified by this phase. Native UI API
1.2.0 continues to use read-only Meiju sessions. This developer probe is a separate
trusted Core host, not a production CLI dependency for Flutter.

## Admitted scope and evidence

The four exact AC models in [the observed model table](../compatibility.md#observed-cloud-read-models)
can expose `main / power.set {state: boolean}` through an explicit Rust
`MeijuAdapter::with_ac_power_control()` opt-in. This describes experimental protocol
capability; it grants no authority. Core separately requires a grant for the exact
device, component, action, argument and cloud transport. Default construction still
exposes read-only mappings and refuses execution even if writable metadata exists.

The pinned [cloud client](https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/core/cloud.py)
uses `/mjl/v1/device/lua/control` with `applianceCode` and
`command.control`. Its [AC mapping](https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/device_mapping/T0xAC.py)
selects `power`; [climate on/off](https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/climate.py)
uses explicit on/off values. HomeVerb sends only `{"power":"on"}` or
`{"power":"off"}`. It neither toggles nor changes temperature/mode, copies stale
state into the command, downloads Lua, or sends an automatic restoring command.
No model-specific physical success follows from this source mapping.

AC temperature/fan constraints, dishwasher program/start semantics, and refrigerator
compartment writes remain pending. The dishwasher mappings can require preserving
other options when starting; the mere presence of an operator field does not admit
a generic cycle command. Refrigerator cooling changes need separate risk review.

## Outcome contract

The existing Command/Execution schemas and native API version remain unchanged.
See the synthetic [accepted](../../examples/command-result.meiju-accepted.json) and
[unknown](../../examples/command-result.meiju-unknown.json) fixtures.

| Evidence | Phase / result |
|---|---|
| Invalid input/binding, unavailable credentials, or preflight cancellation | `not_sent` / `failed` |
| Valid bounded JSON with exactly one numeric `code: 0` or string `code: "0"` | `accepted`; no physical completion claim |
| HTTP 401/403 or provider authorization codes 7400/40002 | `rejected` / `failed` with `AUTH_REQUIRED` |
| Other HTTP/provider errors, malformed/oversized reply, disconnect, or cancellation after possible transmission | `possibly_sent` / `unknown` |

HTTP 200 alone is insufficient. Unknown provider codes may describe downstream
failure after dispatch; they are not generalized into explicit rejection. The
control transport is separate from read/login transport and defaults to refusing
controls. It marks possible transmission immediately before HTTPS execution. After
this point even a connection error is conservative uncertainty. There are no
redirects, retries, fallback writes, or implicit reauthentication.

Verification is `unavailable`: existing cloud observations have no trustworthy
source time. `wait_for: verified` therefore retains `accepted` with
`VERIFICATION_PENDING`; no verification read, optimistic cache update or fabricated
completion follows. A later manual read is separate evidence. Saved request IDs
prevent automatic replay, including after reopening the profile.

## Prepare a local preview

Use the same private profile as the [read-only probe](06-meiju-probe.md). The local
device ID comes from its `inspect` output or native device cards; never substitute
a provider appliance code. Build both examples offline after dependencies exist:

```bash
cargo build -p homeverb-midea --examples --offline --locked
./target/debug/examples/control_meiju --help
./target/debug/examples/control_meiju \
  --data-dir "$HOME/.local/share/homeverb/meiju-pilot" \
  --device-id <local-device-id> --state off --request-id <unique-local-request-id>
```

The default is a preview with zero controls. It performs an explicit inventory
refresh for the selected account, checks current/historical cross-account bindings,
requires the device in the complete current inventory, and publishes descriptors
before Core preflight. Thus preview uses the secure store, vendor reads and local
metadata writes; it is not offline. Only the chosen target/state receives a grant.
It has a shared 20-second budget, no login, and Ctrl+C cancellation.

Experimental power descriptors remain in this profile until read-only rediscovery
replaces them. They never grant native/MCP write permission: those hosts have no
Meiju grants and the default adapter refuses controls independently. Merely showing
a writable property in cached metadata must not be interpreted as authorization.

On October 15, 2026 the built probe completed a real saved-account preview for the
unique local `KFR-35G/WF2` card: `power.set`, `state: false`, cloud-only plan and
`controls_sent: 0`. Inventory refresh and Core preflight passed without login.
This is not send, acknowledgement, appliance-state or off-LAN acceptance evidence.

## Owner-authorized physical acceptance

Before the first send, identify the exact appliance, confirm its current use and
obtain explicit owner permission for one `off` or `on` action. This is required by
[the delivery rules](../../AGENTS.md#delivery-workflow) and
[ADR-0022](../adr/0022-meiju-community-cloud-admission.md). Approval of the community
protocol does not authorize operating a household appliance.

After that permission, run the identical reviewed preview command with `--send`.
Keep the request ID fixed. Exit code 10 means accepted but unverified; 11 means
unknown; 1 means failed. Do not create a fresh ID to retry an uncertain result.
Compare the appliance and official app independently, then optionally perform an
explicit read. Record model/firmware, action, receipt status and physical/app
observations without private identifiers. Never label stale cloud data `verified`.
Restoring the prior state is another physical action requiring permission.

Off-LAN tests, mobile secure stores, Flutter write authorization, more AC settings
and the other appliance controls remain follow-up work. T10 is incomplete.
