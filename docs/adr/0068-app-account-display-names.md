# ADR-0068: Use optional account remarks and masked sign-in labels

Status: accepted for the owner-requested App naming policy on October 17, 2026.

## Decision

The owner requested recognizable account names, with priority given to a user
remark, then available provider user information, then the account used to sign in.
The native platform page exposes separate Account details and Rename Account
buttons, a compact local reload action and Add Platform. It omits routine stored
authorization explanations without claiming that local credentials establish access.

An explicit local account label is the remark. Existing custom labels remain
literal user data. The historical default Meiju account is treated as an unnamed
account. Native API 1.22 supplies no reviewed provider nickname or login field;
that unavailable source is skipped, rather than guessed from protected subjects,
device names or IDs. Provider profile retrieval is not implemented by this change.

After successful foreground sign-in, Flutter retains only a masked display
projection of the submitted login in its private `account_login_labels` preference
map, keyed by the returned local account UUID. Phone labels retain three leading
and four trailing digits, email labels mask the local part, and other logins retain
only short fragments. No original login, password, token or provider subject enters
preferences, general JSON, logs or diagnostics. Failed, cancelled and late sign-in
results cannot install a new label. Credential buffers retain their existing
clearing and protected native transport.

New connections without a remark use this masked projection as their nonsecret
local display label. Native identity matching still uses protected server evidence,
never this label. Reconnecting preserves existing native labels and organization;
the App can display its captured fallback for a historical unnamed account.
Clearing a remark sends the existing rename method a nonempty captured fallback,
or the historical default if none exists. It never sends an empty native label.
Successful local account listing prunes fallback labels for accounts no longer
present, including after confirmed removal. This does not read the secret store.

## Compatibility and limits

This refines the frontend label policy without changing ADR-0026's prohibition on
guessing a login from native metadata. It adds no native user-info field, C/JSON
method, permission or authentication source. Native API 1.22, SQLite schema 5,
protected payload 3 and account identity guards are unchanged. The App preference
format remains version 1 with an optional nonsecret display map.

Earlier installations did not retain the submitted login. Their unnamed accounts
keep a localized provider-account placeholder until the user supplies a remark or
explicitly signs in again. Do not inspect credentials, silently sign in, infer a
phone number or claim that this display update recovered historical user details.
Duplicate visible names retain separate local identity suffixes and exact targets.

## Verification

Use fictional credentials and isolated preferences to verify masking, persistence,
remark priority, clearing, exact-account editing, removal cleanup and failed/late
authentication. Check button affordances, disabled actions, localization and narrow
enlarged-text layouts. APK/emulator evidence and real reauthorization remain
separate; no real appliance command is required.
