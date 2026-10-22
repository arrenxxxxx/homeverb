# ADR-0075: One-shot Meiju read compatibility probe

Status: accepted for the owner-requested local read experiment.

## Decision

The owner requests trying the existing HomeVerb account session with the new
reference interfaces. A missing independent modern login implementation does not
prove that an existing session is incompatible. The supplied read-only client
uses the same POST HMAC input structure as the already admitted community signer;
the excluded constants are not assumed equivalent without a live experiment.

An explicitly compiled `HOMEVERB_MEIJU_READ_PROBE=1` development library may issue
one fixed inventory-aggregate POST and one fixed empty-command Lua-state POST
per process. Requests follow corresponding foreground authenticated legacy reads,
use the same protected session and admitted signer, and inherit cancellation,
deadline, HTTPS origin, response cap, no redirect, no proxy and no retry rules.
The aggregate body and Lua caller match the supplied read-only client. No new
secret, raw token export, general URL execution or control request is admitted.
Default builds do not send probes. They are not scheduled outside the existing
foreground operation or repeated by the foreground state-read loop.

Probe responses never enter account admission, inventory, device descriptors,
permissions or state caches. Only closed result markers distinguish business
success, response shape, rejection, authentication refusal or transport/cancellation
failure. Provider messages, credentials, identifiers and raw responses are never
logged or persisted. HTTP success alone is not business success; business success
does not establish physical sample freshness or full interface compatibility.

## Compatibility and validation

No native API, schema, storage format, public method or action outcome changes.
This experiment does not replace the existing transport or admit any writes.
Use separate development APK/AAR artifacts, preserve the signed-in emulator's
data during replacement and restore the ordinary APK after the experiment.
Tests require exact read-trigger admission, protected-token retention, empty
state queries, closed rejection classification and one-shot bounds. Existing TLS,
response limits and lifecycle tests cover the shared transport implementation.
Record actual results and any limitations in the App verification record.
