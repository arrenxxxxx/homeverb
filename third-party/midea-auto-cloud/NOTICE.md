# Midea Auto Cloud attribution

Source: https://github.com/sususweet/midea_auto_cloud

Pinned revision: `ce0c4b88224def0f6b53714312c19f71aa9c2a50`.

Upstream files: `custom_components/midea_auto_cloud/core/cloud.py` and
`custom_components/midea_auto_cloud/core/security.py`,
`custom_components/midea_auto_cloud/device_mapping/T0xAC.py`,
`custom_components/midea_auto_cloud/device_mapping/T0xE1.py`,
`custom_components/midea_auto_cloud/core/device.py` and
`custom_components/midea_auto_cloud/climate.py` and
`custom_components/midea_auto_cloud/__init__.py`.

License: Apache License 2.0; the complete upstream license is in [LICENSE.txt](LICENSE.txt).
The inspected revision contains no NOTICE file. Retain this attribution and the
license when distributing the derived portions; HomeVerb uses [Apache-2.0](../../LICENSE); these upstream terms remain applicable.

HomeVerb modifications: a minimal Rust port of the Meiju password transforms,
request signing, session-key decoding, login/read request shapes, protected product
mapping evidence and explicit AC
power, vendor fan-preset, combined power-on/mode and protected-profile temperature
selection/control encoding, and reviewed E1 dishwasher program/explicit additional-option
encoding and profile-scoped pause/resume/cancel payloads through independent exact
grants and trusted approval, with optional raw operator observations and bounded
operations, restricted HTTPS endpoints, conservative write outcomes, redacted native data and synthetic tests.
No default appliance keys, SmartHome routing, executable Lua downloads, automatic
reauthentication, retries, or background work are copied. The public compatibility
constants are included in the owner-requested open-source preparation under
ADR-0022's October 17 clarification. They do not
protect user credentials and do not establish provider approval or model support.
