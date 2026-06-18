This directory vendors the generated Ada URL parser source used by PTN's
native runtime for `Uri\WhatWg\Url`.

Source: `ada-url` crate 3.4.5, `deps/ada.cpp`, `deps/ada.h`, and
`deps/ada_c.h`.

Local PTN delta: the path percent-encode/signature tables leave `^`
unencoded to match PHP ext/uri WHATWG URL rows.

The files are dual-licensed under MIT or Apache-2.0; see `LICENSE-MIT` and
`LICENSE-APACHE`.
