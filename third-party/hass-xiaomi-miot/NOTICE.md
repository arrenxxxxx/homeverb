# Community Xiaomi Miot protocol attribution

HomeVerb's independently authored Rust captcha continuation uses wire behavior
and regression evidence from `al-one/hass-xiaomi-miot`, revision
`f78158257e19a21902adaf6bcc8440c3ba172a91`, inspected October 25, 2026.
The community project is distributed under [Apache-2.0](LICENSE).

Sources: [cloud client](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/xiaomi_cloud.py)
and [captcha tests](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/tests/test_cloud_auth_step2.py).
No upstream Python code, HA runtime, application credentials or account material
is bundled. No Xiaomi official HA restricted implementation is incorporated.
The original community project's notices remain applicable to any future direct
code reuse; dependency licenses must be checked separately at that time.

HomeVerb implements its own bounded operation ownership, protected single-use
state, dedicated TLS origin validation, image size/dimension checks and explicit
user-driven continuation. It does not adopt HA plaintext storage, background
relogin, arbitrary redirects or automatic request replay. This is community
compatibility work, not Xiaomi endorsement or physical-device acceptance.


The independently authored exact `cuco.plug.v3` daily statistics protocol also
uses model/key/ratio facts from the same revision's `core/device_customizes.py`,
request facts from `core/device.py`, and decoder evidence from `core/utils.py`,
`core/templates.py` and `tests/test_power_cost_statistics.py`, inspected October 27,
2026. No Python/template code is bundled. HomeVerb keeps bounded foreground
operations, explicit historical scope and unproven completeness, rather than HA
scheduling, calendar totals or restored state. See
[ADR-0088](../../docs/adr/0088-mijia-exact-daily-energy-history.md).

On October 29, 2026, the same pinned revision's `switch.py`, `core/device.py` and
`core/device_customizes.py` were inspected for exact outlet single-property
control behavior. BLE/event examples in `binary_sensor.py` and `sensor.py` were
also reviewed without transferring their keys to unrelated models. These are
protocol/design inputs for ADR-0091, not copied source or physical acceptance.

On October 30, 2026, the same pinned revision's `core/xiaomi_cloud.py`,
`core/device.py` and `core/miot_spec.py` supplied the action request object,
empty input/output references and differentiated error-code facts used by
[ADR-0107](../../docs/adr/0107-bounded-no-argument-actions.md). HomeVerb's Rust
implementation is independent and does not adopt automatic relogin/retry or
code-1 success semantics. Exact action definitions come from the separately
pinned public specification projection. No community Python code or HA runtime
is bundled; modern-profile interoperability and physical acceptance remain open.
