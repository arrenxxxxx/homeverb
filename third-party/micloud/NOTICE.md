# micloud protocol attribution

HomeVerb's independent Rust Mijia login and encrypted cloud request implementation
is based on protocol behavior documented by MIT-licensed `Squachen/micloud`,
commit `dc002b87fc09ef36199715cfa10f1eaa2d2d4210`, inspected October 18, 2026.
Copyright (c) 2020 Sammy Svensson. The [MIT license](LICENSE) is retained here.

Source: [micloud.py](https://github.com/Squachen/micloud/blob/dc002b87fc09ef36199715cfa10f1eaa2d2d4210/micloud/micloud.py)
and [miutils.py](https://github.com/Squachen/micloud/blob/dc002b87fc09ef36199715cfa10f1eaa2d2d4210/micloud/miutils.py).

HomeVerb adds bounded operation ownership, allowlisted TLS endpoints, protected
buffers, sanitized errors, strict decoding and no automatic retries or logging.
It does not ship the Python client or its plaintext session/device export behavior.
Xiaomi and Mijia are vendor names; no endorsement or official authorization is claimed.
