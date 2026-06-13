# PTN Progress

Refresh: 2026-06-13T22:38Z
Measured: `ptn-550s.11` nested foreach/reference PHPT slice.

Slices cover callable/object, filesystem/string, property, COW, PHPT blockers,
readonly metadata, `throw`, Closure references, and nested `foreach`;
`ptn-550s.11` also adds `is_numeric()` dispatch.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 692 | 692 | 0 |
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
| PHPT array-internal COW frontier | 72 | 0 | 72 |
| PHPT COW foreach/reference frontier | 103 | 31 | 72 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT string/scalar alias rows | 35 | 23 | 12 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- Accepted bounded rows remain 485/485; classify-only reports 459 runnable and
  26 excluded outside the string slice.
- COW frontiers: array-internal 72/0/72; foreach/reference 103/51/31.
- Zend arrow rows 001-004 pass; remaining rows need closure metadata, nullable
  types, generator `yield`, or `assert.exception`.

## Verification

`ptn-550s.11` adds live nested by-reference `foreach` current-key advancement,
child rekeying, `is_numeric()` dispatch, and a plain variable-variable unset
classifier. Before evidence: selected 3, ran 3, passed
0, failed 3. Current evidence: nested manifest selected 3, ran 2, excluded 1,
passed 2/2 at `manifest-20260613T222839Z`; blocker is plain variable-variable
unset. COW oracle matched PHP 13/13; post-merge COW gate passed 26/26; COW PHPT
passed 54/54 at `manifest-20260613T223008Z`; smoke 6/6; build; inventory is
692 native/compiler plus 3 source.

Recent gates: `ptn-8d2u` Closure references passed; `ptn-qsmv.11` throw rows
2/2 with blockers `ptn-5sca`, `ptn-c284`, `ptn-feps`; `ptn-qsmv.12` readonly
5/5, classifier 15/15, Zend PHPT 8/8; `ptn-550s.9` foreach 6/6; `ptn-flje`
COW classify-only is 46/31/15.

Follow-ups remain typed properties, traits, magic methods, attributes, heredoc,
declared `Exception` subclasses, property/reference edges, nullable types,
generator `yield`, first-class callables, dynamic includes, unsupported
internals, `Traversable`, INI modes, process boundaries, formatter/callback
parity, and classifier batching.
