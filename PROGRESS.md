# PTN Progress

Refresh: 2026-06-13T22:45Z
Measured: `ptn-f0rp` array-internal COW helpers after `ptn-550s.11`.

Slices cover callable/object, filesystem/string, property, COW, PHPT blockers,
readonly metadata, `throw`, Closure references, nested `foreach`, and
array-internal COW helper parity.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 693 | 693 | 0 |
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
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT string/scalar alias rows | 35 | 23 | 12 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- Accepted bounded rows remain 485/485; classify-only reports 459 runnable and
  26 excluded outside the string slice.
- Array-internal COW frontier is 72 selected, 17 runnable, 17 passing, and 55
  excluded; foreach/reference COW frontier remains 103/51/31.
- Zend arrow rows 001-004 pass; remaining rows need closure metadata, nullable
  types, generator `yield`, or `assert.exception`.

## Verification

`ptn-f0rp` preserves references for `array_replace*()`, adds modeled
`array_splice()` mutation, and walks recursive leaves through callback dispatch.
Array-internal COW frontier evidence: 72 selected, 17 runnable, 55 excluded,
17/17 passed. `ptn-550s.11` nested by-reference `foreach` evidence remains
2/2 runnable rows with one plain variable-variable unset blocker. COW oracle
matched PHP 13/13; post-merge COW gate passed 26/26; COW PHPT passed 54/54;
smoke passed 6/6; build passed; inventory is 693 native/compiler plus
3 source.

Recent gates: `ptn-8d2u` Closure references passed; `ptn-qsmv.11` throw rows
2/2 with blockers `ptn-5sca`, `ptn-c284`, `ptn-feps`; `ptn-qsmv.12` readonly
5/5, classifier 15/15, Zend PHPT 8/8.

Follow-ups remain typed properties, traits, magic methods, attributes,
heredoc/nowdoc, declared `Exception` subclasses, property/reference edges,
nullable types, generator `yield`, first-class callables, dynamic includes,
unsupported internals, destructor-reentrant `array_splice()`, `Traversable`,
INI modes, process boundaries, formatter/callback parity, and classifier
batching.
