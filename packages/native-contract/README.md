# HomeVerb native contract

Start with the [Flutter handoff](../../docs/native-ui-contract.md). API version 1.7.0 has 30 callable JSON methods and one separate protected account-connection function; the JSON envelope schema is 1.0.

- [api.json](api.json) is compiled into the `get_api_info` response.
- [examples.json](examples.json) contains fictional requests and actual Mock native responses, including failures and lifecycle states. It is preview/test data, never a production device database.
- [account-examples.json](account-examples.json) provides fictional connection/refresh DTOs and local detail/rename/removal/recovery request/results; [native-connect.schema.json](../../schemas/native-connect.schema.json) defines nonsecret connection metadata. Secret input is separate native memory, never general JSON.
- [Startup](../../schemas/native-open.schema.json), [requests](../../schemas/native-request.schema.json), and [responses](../../schemas/native-response.schema.json) define the wire contract. Response definitions are keyed by method; `open` describes startup.
- [verify_native_contract.py](../../scripts/verify_native_contract.py) validates every method against the built Linux library and resolves all schema references offline.

Trusted startup can select `midea.meiju` for read-only access to previously connected local accounts; omitted selection remains Mock. Protected native account connection is callable; Flutter input UI and mobile secure stores remain pending. Explicit recovery validates pending saved Meiju credentials and retrieves cards without another login. Local account removal has durable pending/completed outcomes; explicit cache/history clearing preserves replay protection. Provider revocation, sharing, arbitrary identity merge and full-profile deletion remain unavailable. Use explicit unavailable states for those production actions; do not infer availability from a mockup. Keep Flutter-only preview repositories separate from the native session.
