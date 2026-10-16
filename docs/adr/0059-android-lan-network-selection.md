# ADR-0059: Select and recheck an Android-assigned LAN interface

Status: accepted for local platform integration and isolated acceptance.

## Decision

Add `org.homeverb.NativeLanNetworks` to the production Android AAR. The trusted
Flutter Android controller owns one instance and mirrors foreground/close events
to it alongside the native Session. It starts backgrounded, has no listener,
socket, service or worker, and only queries Android connectivity metadata when
the foreground user explicitly requests network choices or confirms a choice.

Use `ConnectivityManager` network capabilities and link properties, requiring
`ACCESS_NETWORK_STATE`. Return only assigned private/link-local IPv4 host
addresses on direct Wi-Fi or Ethernet networks with prefixes accepted by the
native scanner. Exclude VPN, cellular and other transports; refuse an active
default VPN instead of attempting to bypass it. Internet validation is not a
condition: a local appliance network need not have Internet access. The collector
binds a local address, so conflicting assignments of that address to different
networks/interfaces/prefixes are excluded rather than choosing an ambiguous route.

Return immutable, nonserializable Selection objects owned by that instance and
lifecycle generation, with a 60-second monotonic lifetime. Only the interface
name, transport, IPv4 display/numeric value and prefix are available for trusted
preview use. Keep network handles private and redact `toString`. A Selection is
not a peer identity, model mapping, credential or connection-link approval.

Before preparing API 1.18's native scan, and again before consuming its confirmed
preview, call `requireCurrent(selection)`. It re-reads Android metadata and checks
the exact network/interface/address/prefix/transport tuple, ownership, lifecycle
and expiry. Changing selection requires discarding the native preview and obtaining
new consent. Background/close invalidates all selections. No mutex is held over
platform calls; lifecycle is checked again afterwards. Platform errors and denied
metadata permission return fixed errors, never an empty success or OS messages.

The helper never requests permission or asserts that sockets are authorized.
`ACCESS_NETWORK_STATE` permits metadata inspection only. The host still owns
actual user consent, `INTERNET` and applicable Android local-network runtime
permission. Android 16 opt-in protection and Android 17 target-SDK enforcement
follow the [official platform guidance](https://developer.android.com/privacy-and-security/local-network-permission).
Do not request Android 17's `ACCESS_LOCAL_NETWORK` for a lower target SDK or
change platform protection to make a test pass. Network changes after the final
check remain possible; bounded socket errors and endpoint authentication remain
independent requirements. There is no process-wide network binding or VPN bypass.

## Verification and compatibility

The native API remains 1.18 with 33 JSON methods and 13 C exports. No Rust,
credential, storage, grant or cloud-routing semantics change. The production
manifest adds only normal `ACCESS_NETWORK_STATE` alongside existing `INTERNET`.
The Flutter worktree owns its Android controller and runtime permission UI.

Compile against official Android stubs and exercise the same production helper
with injected fictional metadata/clock on the JVM. Cover eligibility, ambiguous
addresses, exact revalidation, foreign/expired/invalidated selections, errors,
concurrent lifecycle changes and redaction. Package no test source in AARs.

The isolated acceptance APK may request only `ACCESS_NETWORK_STATE`, keeping
`INTERNET` absent. Its network phase reads actual disposable-emulator connectivity
metadata and checks lifecycle/selection behavior without opening a socket,
connecting an account or invoking discovery. APK permission/export/alignment
inspection and exact runtime assertions are required. Physical phone, Android
16/17 permission enforcement, Flutter consent and actual LAN routing remain
separate acceptance gates.
