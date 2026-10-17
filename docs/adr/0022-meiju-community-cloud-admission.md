# ADR-0022: Admit a scoped Meiju community cloud exception

Status: accepted by explicit owner approval on October 15, 2026. Supersedes ADR-0005 only for the scoped Meiju community protocol below.

## Problem and verified evidence

The required journey is an existing Meiju account's devices appearing automatically in HomeVerb, followed by direct cloud reads and permitted controls away from home. A LAN-only adapter or manual token import does not complete it.

The [official v3 API](https://mis-cdn.smartmidea.net/docs/control-midea-cloud-devices/cloud-2-cloud-v3-api.html), section 5.2.3, requires `client_secret` for token acquisition and refresh. Section 5.8 signs requests with the developer secret. The inspected page does not document PKCE. Absence from this page is not proof that no alternative can ever be arranged with the provider.

The [official Android guide](https://iot.midea.com/docs/control-midea-cloud-devices/android-sdk.html), sections 3.1 and 3.2, describes a provisioning SDK and an app receiving a token through a third-party server. It does not establish the independent native account/control flow required here. These official public documents were read on the decision date; no developer application was registered or provider contacted.

The community [Midea Auto Cloud client](https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/core/cloud.py) uses a different, app-facing protocol: `/v1/user/login/id/get`, then `/mj/user/login`, followed by account-scoped inventory and control requests. Its [security implementation](https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/core/security.py) uses shared app material for password transforms, request HMAC and session-key decoding. That material is present in the pinned public source; publication does not establish provider approval, confidentiality classification, or stable service access. No values are reproduced here.

## Previous decision boundary

[ADR-0005](0005-native-auth.md) prohibits embedding confidential shared application secrets to claim native support. The [integration guide](../development/05-midea.md) also prohibits copying shared credentials or default device keys to bypass that gate. That rule previously blocked bundling the community signing material. The owner has now explicitly approved this ADR. The inventory parser and account vault alone still do not establish working authentication.

## Accepted owner decision

The owner approved a narrow community-protocol exception for the personal Meiju MVP, while keeping the no-backend architecture. This allows reviewed, pinned app-compatibility constants from the cited public implementation solely for the account owner's direct Meiju flow. Treat those constants as recoverable client material, never as protection for HomeVerb or a user's credentials. This decision does not classify the material as vendor-approved or grant legal/provider permission.

The exception amends the shared-material restriction for this specific community protocol only. It does not permit confidential developer credentials, other users' credentials, copied default appliance keys, account-access bypass, a HomeVerb server/relay, downloaded executable protocols, or background activity. It does not authorize real-device writes or publication. Apache-2.0 source obligations and the project's release-license decision still apply.

Implementation proceeds in separately verified and committed phases:

1. Port the minimum signing/password/session codec with fictional known-answer vectors and pinned attribution. Keep real user input in trusted native memory and secure storage, outside general JSON, command arguments and logs.
2. Add a bounded HTTPS transport restricted to the reviewed Meiju destination and exact endpoints. Disable redirects, implicit reauthentication and retries. Test cancellation, malformed/oversized bodies and redacted errors before live access.
3. Connect explicit native login to the account vault and automatic inventory retrieval. Persist encrypted provider identity mappings; preserve existing device IDs and local organization. Report authorization and retrieval outcomes separately.
4. Validate an owner-supplied account through a local trusted flow, initially read-only. Add cloud state and capability mapping, then separately authorized writes with independent verification. Keep exact-model and off-LAN acceptance required.
5. Publish additive native account contracts for Flutter only when callable. Keep API 1.0.0 unchanged until that verified phase.

ADR-0005 remains unchanged for every other protocol and for confidential developer credentials. Do not replace the required remote-control journey with manual keys, LAN-only control, or a hidden backend. The existing Flutter Mock contract remains available while the admitted flow is implemented.

## Risks and acceptance

Community API changes, additional login challenges, regional differences and provider restrictions can interrupt access; there is no official-app feature-parity or service-stability guarantee. Local account state must never override a provider rejection. Separate the public compatibility material from private passwords, session tokens and device identity evidence in review and diagnostics.

Acceptance requires actual account-device enumeration, preserved identity across reconnect, protected session storage, bounded foreground lifecycle and model-specific cloud reads/writes. No source review, codec test or Mock result substitutes for those checks. Owner approval removes the protocol-admission blocker; it does not establish implementation, account access, physical support, or release readiness.

## October 17 open-source scope clarification

The owner requested open-source preparation for HomeVerb as a whole, including the
existing Meiju integration, and rejected a duplicate integration-specific approval
requirement. The original personal-MVP wording describes the initial admission;
it must not be used to block that later instruction or require the same scope to
be approved again. Prepare the reviewed existing source for publication under the
selected Apache-2.0 project license, preserving upstream terms and notices.

This clarification creates no new protocol, credential or device-write authority.
The integration remains unofficial; public upstream provenance does not imply
vendor endorsement or guaranteed service access. Actual repository visibility
changes and pushes still follow the separately authorized delivery workflow.
