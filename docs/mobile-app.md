# Mobile app design

## Independent execution

Flutter UI -> in-process bindings -> Core -> in-process adapters -> appliance LAN or vendor cloud. There is no initial HA/NAS address screen and no hidden dependency on a desktop CLI forwarding requests.

Initial screens: platform accounts, device list, device details/dynamic controls, recent outcomes, and settings/diagnostics. Complex dashboards, charts, rule editors, and gateway settings are excluded.

## Dynamic controls

Writable booleans use switches; bounded numbers use steppers or inputs; enums use selections; read-only values use status displays; actions use parameterized forms. Units, steps, and disabled conditions come from capability metadata, not hardcoded brand pages. Specialized device views still issue the same Command model.

An unknown power property must not display "off". Show a requested target separately as "Setting..." rather than overwriting the last observation. Display `accepted` as "Accepted; not yet confirmed" and `unknown` as "May have executed; refresh to verify". Do not encourage repeating a start action.

## Lifecycle

Read, write, and bounded observation happen only for explicit foreground tasks. Backgrounding cancels work and closes subscriptions; already-started appliance behavior may continue. On reopen, display cache age and network only when the user requests it. No keep-alive or system scheduling is added.

## Permissions and credentials

Request local-network or protocol permissions when the relevant feature is used. Explain denial and its consequences; do not silently switch to an unapproved cloud path. Access tokens through platform secret-store interfaces. Flutter UI must not retain unnecessary copies of long-lived secrets. Exclude sensitive data from clipboard operations and default diagnostic exports.

## Cross-platform validation

Start with a mock device to verify dynamic forms, invalid arguments, cancellation, unknown outcomes, repeated calls in one process, and cleanup on backgrounding. Then validate real pairing, LAN access, and capability changes. Android and iOS require independent acceptance; desktop FFI success does not substitute for phone testing.

## No implied guarantees

An app installation does not imply every brand, region, account, model, background state stream, or unattended automation works. Only the compatibility matrix defines verified support.
