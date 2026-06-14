# PTN Progress

Refresh: 2026-06-14T00:42Z
Measured: `ptn-kgqa` array predicate/find helpers after `ptn-550s.10`.

Slices cover callable/object, filesystem/string, property, COW, foreach, broad
PHPT blockers, generator/Fiber blockers, and array predicate/find helpers.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 706 | 706 | 0 |
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
| PHPT foreach list destructuring rows | 4 | 4 | 0 |
| PHPT broad reference-call bucket | 12 | 8 | 4 |
| PHPT generator/fiber COW boundary bucket | 12 | 0 | 12 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT focused array predicate/find rows | 4 | 4 | 0 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- Accepted bounded rows remain 485/485; classify-only is 459 runnable/26
  excluded.
- Array-internal COW is 72/17/17; foreach/reference is 103/51/31.
- Broad reference-call bucket is 12 selected, 11 runnable, 1 excluded, 8 pass.
- Static SKIPIF 1k is 447 runnable/553 excluded.
- Generator/fiber COW boundary bucket is 12 selected/0 runnable/12 excluded.

## Verification

`ptn-kgqa` adds generic callback scanning for `array_all()`, `array_any()`,
`array_find()`, and `array_find_key()`: `(value, key)` args, short-circuiting,
callback validation, and exception propagation. Focused native/parser tests
pass; focused PHPT is 4/4.

`ptn-550s.10` adds long `list(...)` destructuring edges, list-as-key and
empty-list diagnostics, and scalar row-read warnings. Focused PHPT is 4/4; COW
gate is 26/26.

`ptn-vwyp` classifies COW generator/fiber rows before they enter pass counts:
46 selected, 32 runnable, 14 excluded. `ptn-2juv` refreshes broad 1k evidence:
1,000 selected, 447 runnable, and 553 classified.

Follow-ups remain typed properties, traits, magic methods, attributes,
Exception traces, nullable types, generator/Fiber execution, dynamic includes,
callback-array parity, `Traversable`, and INI modes.
