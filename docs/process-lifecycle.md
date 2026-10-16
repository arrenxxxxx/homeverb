# Process lifecycle and the no-daemon rule

## Allowed execution units

| Entry point | Starts when | Stops when | Idle behavior |
|---|---|---|---|
| CLI | The user runs a command | Completion, deadline, or cancellation | No background main loop |
| App | The user opens and operates it in the foreground | Release networking after each task; cancel on backgrounding | UI/cache may remain; no periodic control |
| MCP stdio | A local host starts its child process | EOF, host shutdown, or termination | May await tools; must not scan devices proactively |
| Auth callback | The user explicitly starts login | One callback or authorization timeout | No retained listener |
| Authorization sharing | The user starts Send Authorization or Receive Authorization | Completion, cancellation, deadline, or backgrounding | No idle discovery, advertising, or receiver |

No-daemon MCP means no independently deployed service. It does not mean restarting the MCP process for every tool call. A host session may be long-lived and its MCP process may remain idle, but it must not detach into a device gateway. See the stdio reference in [R1](references.md).

## Resource ownership

Connections, discovery sockets, subscriptions, retries, and file locks belong to an `OperationScope`. Normal completion and cancellation release them. Forced termination cannot rely on `finally` or cleanup callbacks. On the next startup, recover persisted unresolved write phases without replaying the physical action.

The native [V3 TCP helper](adr/0046-bounded-v3-lan-codec.md) owns its socket inside
one supplied Operation through connect, handshake and one command exchange.
It exposes no retained connection and spawns no worker. Cancellation or dropping
the future closes the socket without clearing possible command transmission;
closing the socket cannot roll back bytes already accepted by the network stack.

Authentication callbacks may bind a temporary loopback port with a short deadline; never bind `0.0.0.0`. A protocol may require a temporary UDP socket. It must not expose a general control API or remain open after the operation.

The [foreground sharing decision](adr/0008-foreground-authorization-sharing.md) additionally permits bounded local-network discovery and transfer listeners within an explicit sharing operation. These expose only the reviewed transfer protocol on the selected local network, never a device-control API or relay. Merely opening Settings does not start them. Closing a sharing operation cannot retract credentials already imported by another installation.

## Mobile foreground and background

When backgrounded, reject new work and request cancellation of in-flight work. Unsent operations can end safely. A write already sent without an acknowledgement becomes `unknown`; tell the user to refresh when foregrounded again. A known accepted acknowledgement remains `accepted`. Do not rely on guaranteed background execution time, keep-alive tricks, or exact mobile scheduling.

## Appliance work is not client work

A washing machine may continue after the app sends `start` and exits. Closing the app should not reverse that action. Sending a new command tomorrow morning requires future client execution and is outside scope.

## Acceptance tests

For each entry point test normal exit, deadline, SIGINT/cancellation, forced termination, and device disconnect. Check for added services, startup entries, polling, and orphaned listeners. MCP must exit after host stdin closes and generate no appliance traffic while idle. A completed or expired auth callback must leave no listening port.
