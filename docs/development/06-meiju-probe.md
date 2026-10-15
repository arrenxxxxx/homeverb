# 6. Verify a Meiju account locally

This is an explicit read-only developer probe for the owner-approved personal MVP
under [ADR-0022](../adr/0022-meiju-community-cloud-admission.md). It signs requests
directly to the mainland Meiju service, stores the session in Linux Secret Service,
and automatically retrieves existing bound appliances. It never sends appliance
controls or installs a service. The explicit `core-inspect` command additionally
publishes model-scoped read-only device cards and caches validated Core snapshots.

## Prerequisites

Use the existing mainland Midea Meiju account that already owns the appliances.
SmartHome and other regions are not admitted. An unlocked Linux Secret Service
collection must be available. No HomeVerb account, HA, gateway, manual appliance
keys or LAN discovery is required. The reference protocol currently accepts only
bounded ASCII account/password input; unsupported input or an additional provider
challenge fails without bypass or automatic retry.

Build from the repository root:

```bash
cargo build -p homeverb-midea --example connect_meiju --locked
```

After dependencies are cached, add `--offline`. Run the resulting binary in your
own interactive terminal. Never put account credentials in chat, command arguments,
environment variables, shell scripts, ordinary files or public JSON requests.

## Connect and automatically retrieve

```bash
./target/debug/examples/connect_meiju --data-dir "$HOME/.local/share/homeverb/meiju-pilot" connect
```

Both account and password prompts disable terminal echo. Passwords are discarded
after login; only the session, opaque account subject and appliance bindings enter
the system secret store. SQLite contains random local references and metadata.
The network operation has one 180-second budget after input; Ctrl+C cancels it.
No control request is available in this binary.

Successful output contains only the random local account ID, credential-storage
state, retrieval outcome and counts for the three candidate families plus others.
`physically_verified` remains false. Device counts are inventory evidence, not
model compatibility or fresh state. A successful empty list differs from failed or
partial retrieval. The probe does not print provider identifiers, names, tokens,
password transformations, session keys or raw responses.

Connecting the same provider subject again reuses its existing account and device
IDs and preserves local preferences. A supplied existing account ID additionally
refuses a changed subject before session replacement. Account registration is
serialized across this profile; ordinary credential work retains account leases.
No nickname, account spelling or device name is sufficient identity evidence.

## Refresh without another login

Use the random `local_account_id` returned by the previous command:

```bash
./target/debug/examples/connect_meiju --data-dir "$HOME/.local/share/homeverb/meiju-pilot" list
./target/debug/examples/connect_meiju --data-dir "$HOME/.local/share/homeverb/meiju-pilot" refresh <local-account-id>
```

`list` uses only local metadata. `refresh` loads the protected session and performs
one bounded inventory operation. It never reads a saved password, automatically
logs in or retries. Expired authorization requires explicit reconnection:

```bash
./target/debug/examples/connect_meiju --data-dir "$HOME/.local/share/homeverb/meiju-pilot" connect --account-id <local-account-id>
```

A provider login rejection does not replace the previous session. If authentication
and secure persistence succeed but retrieval fails, output retains the local account
and distinguishes the retrieval error; use an explicit refresh rather than blindly
logging in again. Partial inventories never remove earlier bindings. Unknown or
unsupported appliance families receive private bindings, not invented controls.

## Inspect mapped cloud state

```bash
./target/debug/examples/connect_meiju --data-dir "$HOME/.local/share/homeverb/meiju-pilot" inspect <local-account-id>
```

This explicit operation retrieves the inventory and queries each supported family's
mapped cloud state once under the shared budget. Output contains product models,
random local device IDs and validated mapped properties only. It does not print raw
provider responses or appliance identifiers. Refrigerator compartments share one
local device ID. Missing fields remain absent. All cloud observations retain an
unavailable source timestamp and `stale: true`; a new response does not prove fresh
physical state. This protocol probe does not register Core descriptors or enable
controls. Per-device failures remain explicit; cancellation/deadline stop further work.

## Publish read-only cards and inspect through Core

```bash
./target/debug/examples/connect_meiju --data-dir "$HOME/.local/share/homeverb/meiju-pilot" core-inspect
```

This explicit command composes the production `MeijuAdapter` with Core, discovers
all saved mainland Meiju accounts, admits descriptors and reads every component
through Core's validation/cache path. It publishes local cards and cached state in
this profile; it does not log in. Output includes only models, component/property
names, support status, stale flags and fixed errors, without private state values.
There is one shared 180-second budget and no automatic retries or controls.

The six [observed models](../compatibility.md#observed-cloud-read-models) have
experimental read-only mappings. Other models remain visible as unsupported;
no brand-wide support, temperature limits or writable controls are inferred.
Partial discovery preserves existing cards with `MIDEA_INVENTORY_PARTIAL`.
Conflicting cross-account bindings require explicit reconciliation, including
historical bindings whose appliance is missing from the latest inventory.

On October 15, 2026 this command published six real cards and successfully read
seven component snapshots, with zero failures. All source timestamps remained
unproven and every observation stale. Physical/app comparison and off-LAN tests
remain pending. Native API 1.1.0 can open this same profile with explicit `adapter: "midea.meiju"`;
see the [native startup contract](../native-ui-contract.md#real-meiju-read-only-startup).
API 1.2 adds protected account connection and list/refresh methods; Flutter input UI remains separate integration work.

## Interpret failures and record evidence

- `SECRET_STORE_UNAVAILABLE` / `SECRET_STORE_LOCKED`: restore access to the system
  store; no plaintext fallback is available. The probe checks storage before prompting.
- `INPUT_UNAVAILABLE`: use an interactive terminal. Piped credentials are refused.
- `AUTH_REQUIRED`: explicit reauthorization is required; no automatic password replay.
- `MIDEA_REQUEST_REJECTED`: provider rejection or an unsupported challenge. Do not
  expose the raw provider response to diagnose it.
- `MIDEA_REQUEST_FAILED` / `MIDEA_RESPONSE_INVALID`: transport/protocol failure.
  Report only the fixed error code; the exact provider response is not logged.
- `MIDEA_INVENTORY_PARTIAL`: a Core discovery did not receive every home; existing
  cards were preserved. Retry explicitly when the account/network issue is resolved.
- `DEVICE_IDENTITY_CONFLICT`: review conflicting protected device evidence; never
  reset or merge bindings just to create a card.
- `ACCOUNT_IDENTITY_CONFLICT`: reconnect to the correct existing local account or
  review identity evidence. Never reset bindings to force a match.
- `ACCOUNT_SESSION_UNCONFIRMED` / `SECRET_STORE_WRITE_UNKNOWN`: an interrupted or
  uncertain local mutation needs trusted exact-byte recovery. The probe deliberately
  does not delete/recreate accounts or automatically overwrite pending material.
- `RESOURCE_BUSY`: another explicit account operation owns the profile/account lease.

Record only region, app version, device counts and fixed error codes for the initial
read-only acceptance. Exact product models/firmware can be supplied separately
without serial numbers. Then verify repeated retrieval, reconnection identity and
an external-network run. Physical comparison, owner-authorized writes and
Flutter input UI and phone credential stores are subsequent stages. The [integration guide](05-midea.md)
records real connect, saved-session refresh, protocol inspection and Core reads:
four air conditioners, one dishwasher and one refrigerator. Physical model tests
remain pending; synthetic tests do not establish them.
