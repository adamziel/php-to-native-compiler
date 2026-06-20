# Unicode, IDNA, Encoding, And Intl Runtime Boundary

Issue: `ptn-hvvb4.5`

This is the production policy for Unicode-sensitive PHP surfaces. PTN must not
grow local Unicode, IDNA, charset-conversion, collation, break-iteration,
calendar, or mbregex algorithms beyond narrow glue needed to call
PHP-compatible upstream libraries.

## Policy

- PHP strings remain byte strings with explicit lengths. Encoding-aware behavior
  starts only at extension/runtime boundaries that PHP defines as
  encoding-aware.
- `ext/intl` behavior is ICU-owned. `IntlBreakIterator`, `IntlCalendar`,
  `Collator`, `NumberFormatter`, `Locale`, `Normalizer`, `Grapheme`,
  `UConverter`, `Spoofchecker`, and intl IDNA functions should wrap ICU state,
  error codes, locale fallback, and version-dependent behavior. PTN local code
  may adapt boxed values, object lifetime, diagnostics, and PHP method/function
  dispatch only.
- URI WHATWG host parsing and serialization stay Ada-owned. PTN already vendors
  Ada URL for `Uri\WhatWg\Url`; its TR46/IDNA host handling is the source of
  truth for ext/uri WHATWG rows.
- `idn_to_ascii()` and `idn_to_utf8()` stay ICU-owned, not Ada-owned. Those
  functions expose PHP intl options, variants, `idna_info`, error codes, and ICU
  UTS #46 behavior. Sharing byte validation glue with URI code is acceptable;
  sharing Ada punycode/IDNA semantics into intl is not.
- `mbstring` encoding, detection, conversion, width, case mapping, MIME, and
  substitute-character behavior is libmbfl-owned. PTN local code may store
  `mbstring.*` INI state and marshal PHP values into libmbfl calls.
- `mbregex` behavior is Oniguruma-compatible. `mb_ereg*`, `mb_split()`, and
  regex search state must not expand as local regex or UTF-8 parser code.
- `iconv` behavior is system `iconv`/libiconv-compatible. PTN should route
  `iconv()`, `iconv_strlen()`, `iconv_strpos()`, `iconv_substr()`, MIME
  helpers, stream filters, and output handlers through a single iconv adapter
  that preserves PHP conversion, `//IGNORE`, and `//TRANSLIT` behavior.
- `default_charset`, `internal_encoding`, `input_encoding`, `output_encoding`,
  and `mbstring.*` settings belong in one runtime configuration table.
  Startup `internal_encoding` fallback and mbstring deprecation warnings are
  bootstrap compatibility glue; they are not a second charset policy.

## Current PTN Boundary

- `phpc -d` accepts and forwards `default_charset`, `internal_encoding`,
  `input_encoding`, `output_encoding`, and selected `mbstring.*` keys.
- `default_charset` currently feeds bounded standard string behavior such as
  HTML entity conversion. This is a charset resolver boundary, not a complete
  conversion engine.
- The parser recognizes broad `intl`, `iconv`, and `mbstring` function names so
  PHPT classification can progress, but recognition is not a support claim.
- The generated runtime models selected intl class metadata. Until ICU-backed
  object state lands, those classes are metadata shells and should not gain
  local break, calendar, collation, normalization, or conversion algorithms.
- Ada URL is the only approved local IDNA implementation path today, and only
  for `Uri\WhatWg\Url`.

## First Implementation Split

1. Keep the central runtime INI table as the only home for charset defaults.
   Extend it before adding per-extension charset globals.
2. Add an ICU adapter boundary for intl IDNA and `IntlBreakIterator` first:
   object state, locale/text storage, current/next/first/last/following/
   preceding/isBoundary, PHP errors, and `idna_info`.
3. Keep Ada wired only to `Uri\WhatWg\Url` host parsing/serialization. Do not
   move `idn_to_ascii()` or `idn_to_utf8()` onto Ada.
4. Add a libmbfl adapter for the first mbstring slice:
   `mb_internal_encoding()`, `mb_strlen()`, `mb_substr()`,
   `mb_convert_encoding()`, `mb_check_encoding()`, and `mb_detect_encoding()`.
5. Add an Oniguruma-compatible mbregex adapter before broadening `mb_ereg*` or
   `mb_split()` behavior. Existing recognition of these names must remain
   bounded until this adapter exists.
6. Add a system-iconv adapter for `iconv()`, `iconv_strlen()`, `iconv_strpos()`,
   `iconv_substr()`, and the iconv INI/default-charset rows.
7. Remove PHPT classifier exclusions and broaden runtime claims only after the
   corresponding adapter-backed rows pass.

Focused PHPT evidence for this split is
`tools/phpt-ptn-hvvb4.5-unicode-library-boundary-row-pack.txt`.

## Current Focused Evidence

Command:

```sh
PTN_PHPT_TEST_TIMEOUT=20 PTN_PHPT_JOBS=4 \
  tools/run-phpt-manifest.sh \
  tools/phpt-ptn-hvvb4.5-unicode-library-boundary-row-pack.txt
```

Local result on 2026-06-20:

| Selected | Runnable | Passed | Failed | Skipped | Excluded |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 53 | 51 | 21 | 27 | 3 | 2 |

The passing rows currently prove the bounded `default_charset`/HTML entity,
Ada `Uri\WhatWg\Url` IDNA, and limited metadata/ASCII mbstring/BreakIterator
surfaces. The failures cluster on the exact upstream-library adapters this
policy requires next: ICU intl IDNA/calendar/text state, libmbfl conversion and
detection, Oniguruma-compatible mbregex, and system iconv conversion.
