# HomeVerb native contract

Start with the [Flutter handoff](../../docs/native-ui-contract.md). API version 1.1.0 has 22 callable methods; the JSON envelope schema is 1.0.

- [api.json](api.json) is compiled into the `get_api_info` response.
- [examples.json](examples.json) contains fictional requests and actual Mock native responses, including failures and lifecycle states. It is preview/test data, never a production device database.
- [Startup](../../schemas/native-open.schema.json), [requests](../../schemas/native-request.schema.json), and [responses](../../schemas/native-response.schema.json) define the wire contract. Response definitions are keyed by method; `open` describes startup.
- [verify_native_contract.py](../../scripts/verify_native_contract.py) validates every method against the built Linux library and resolves all schema references offline.

Trusted startup can select `midea.meiju` for read-only access to previously connected local accounts; omitted selection remains Mock. Account login/input, sharing, arbitrary identity-merge and destructive-cleanup methods are still unavailable. Use explicit unavailable states for those production actions; do not infer availability from a mockup. Keep Flutter-only preview repositories separate from the native session.
