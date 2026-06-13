# PTN Progress

Refresh: 2026-06-13T23:15Z
Measured: `ptn-begn` callable prefer-ref diagnostics after `ptn-f0rp`.

Slices cover callable/object, filesystem/string, property, COW, PHPT blockers,
readonly metadata, `throw`, Closure refs, nested `foreach`, array-internal COW,
and callable prefer-ref diagnostics.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 697 | 697 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 485 | 485 | 0 |
| PHPT Zend rows | 119 | 119 | 0 |
| PHPT ext/standard rows | 280 | 280 | 0 |
| PHPT focused array key/callback set rows | 75 | 38 | 37 |
| PHPT focused stream rows | 2 | 2 | 0 |
| PHPT focused cwd rows | 2 | 2 | 0 |
| PHPT focused filesystem/path/process rows | 46 | 13 | 33 |
| PHPT tests/basic+func+lang | 78 | 78 | 0 |
| PHPT other rows | 8 | 8 | 0 |
| PHPT COW manifest | 54 | 54 | 0 |
| PHPT nested foreach/reference rows | 3 | 2 | 1 |
| PHPT array-internal COW frontier | 72 | 17 | 55 |
| PHPT COW foreach/reference frontier | 103 | 31 | 72 |
| PHPT broad reference-call bucket | 12 | 8 | 4 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT string/scalar alias rows | 35 | 23 | 12 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- Accepted bounded rows remain 485/485; classify-only reports 459 runnable and
  26 excluded outside the string slice.
- Array-internal COW frontier is 72/17/17; foreach/reference COW frontier is
  103/51/31.
- Broad reference-call bucket is 12 selected, 11 runnable, 1 excluded, and
  8 passing. Follow-ups: append-form by-ref call args, class-name type checks,
  and `SensitiveParameterValue` reflection.

## Verification

`ptn-begn` centralizes by-reference call arguments across direct calls,
`call_user_func*()`, declared methods, and bounded callable `array_multisort()`.
Call-result temporaries emit modeled only-variables notices; fixed-parameter
dynamic mismatches warn and use by-value locals; method calls preserve
reference candidates; append-reference overflow has diagnostic precedence.
Worker evidence: broad reference-call PHPT selected 12, runnable 11, excluded
1, passed 8/11; residuals are `ptn-hpxo`, `ptn-yl7i`, and `ptn-6x02`.
`ptn-f0rp` remains 17/17 on array-internal COW, COW PHPT 54/54, and COW gate
26/26.

Recent gates: `ptn-550s.11` nested foreach/reference 2/2 with one classified
blocker; `ptn-8d2u` Closure refs passed; `ptn-qsmv.11` throw rows 2/2.

Follow-ups remain typed properties, traits, magic methods, attributes,
heredoc/nowdoc, declared `Exception` subclasses, property/reference edges,
nullable types, generator `yield`, dynamic includes, unsupported internals,
`Traversable`, INI modes, process boundaries, formatter/callback parity, and
classifier batching.
