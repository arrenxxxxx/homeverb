# Midea Auto Cloud attribution

Source: https://github.com/sususweet/midea_auto_cloud

Pinned revision: `ce0c4b88224def0f6b53714312c19f71aa9c2a50`.

Upstream files: `custom_components/midea_auto_cloud/core/cloud.py` and
`custom_components/midea_auto_cloud/core/security.py`,
`custom_components/midea_auto_cloud/device_mapping/T0xAC.py` and
`custom_components/midea_auto_cloud/climate.py`.

License: Apache License 2.0; the complete upstream license is in [LICENSE.txt](LICENSE.txt).
The inspected revision contains no NOTICE file. Retain this attribution and the
license when distributing the derived portions; HomeVerb's release license remains undecided.

HomeVerb modifications: a minimal Rust port of the Meiju password transforms,
request signing, session-key decoding, login/read request shapes and explicit AC
power control encoding, with bounded
operations, restricted HTTPS endpoints, conservative write outcomes, redacted native data and synthetic tests.
No default appliance keys, SmartHome routing, executable Lua downloads, automatic
reauthentication, retries, or background work are copied. The public compatibility
constants are admitted only for the personal MVP under ADR-0022. They do not
protect user credentials and do not establish provider approval or model support.
