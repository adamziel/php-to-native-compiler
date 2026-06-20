This directory vendors the generated Ada URL parser source used by PTN's
native runtime for `Uri\WhatWg\Url`.

Source: `ada-url` crate 3.4.5, `deps/ada.cpp`, `deps/ada.h`, and
`deps/ada_c.h`.

Local PTN delta: the path percent-encode/signature tables leave `^`
unencoded to match PHP ext/uri WHATWG URL rows.

Boundary: Ada owns IDNA only for `Uri\WhatWg\Url` WHATWG host parsing and
serialization. PHP intl IDNA functions such as `idn_to_ascii()` and
`idn_to_utf8()` must use the ICU-backed intl boundary because they expose PHP
intl options, `idna_info`, and ICU error behavior.

The files are dual-licensed under MIT or Apache-2.0; see `LICENSE-MIT` and
`LICENSE-APACHE`.
