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
