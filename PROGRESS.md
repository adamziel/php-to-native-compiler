# PTN Progress

Refresh: 2026-06-13T21:40Z
Measured: `ptn-qsmv.11` throw after `ptn-qsmv.12`.

Slices cover callable/object, filesystem/string, property, COW, PHPT blockers,
readonly, and userland `throw` through boxed exception propagation.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 684 | 684 | 0 |
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
| PHPT array-internal COW frontier | 72 | 0 | 72 |
| PHPT COW foreach/reference frontier | 103 | 31 | 72 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT string/scalar alias rows | 35 | 23 | 12 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- No failures among 485 accepted bounded rows. Classify-only reports
  459 runnable and 26 excluded rows outside the string slice.
- Array-internal COW frontier: 72 selected, 0 runnable, 72 excluded. COW
  foreach/reference frontier: 103 selected, 51 runnable, 31 passing.
- Zend arrow rows 001-004 pass; remaining rows need closure metadata, nullable
  types, generator `yield`, or `assert.exception` ini.

## Verification

`ptn-qsmv.11` adds parser/AST/IR/backend support for userland `throw`, boxed
exception propagation, modeled `Exception`/`Error` construction, class lookup,
catch matching, and non-object throw diagnostics.
Worker gates before readonly rebase passed `cargo fmt --check`, full
`cargo test`, smoke 6/6, and PHPT 2/4; blockers are `ptn-5sca`, `ptn-c284`,
and `ptn-feps`. Refinery reran pass rows after rebase: 2/2.

`ptn-qsmv.12` adds readonly parsing, write-once properties, inheritance guards,
and classifier narrowing. Gates: readonly native/parser 5/5,
classifier 15/15, Zend readonly PHPT selected 12, runnable 8, excluded 4,
passed 8/8.

`ptn-550s.9` Zend foreach object/property rerun passed 6/6. `ptn-x8p9` maps
73 `unsupported-ini` rows; `assert.exception` probe passed 5/17. `ptn-flje`
COW classify-only is 46 selected, 31 runnable, 15 excluded.

Follow-ups remain typed properties, traits, magic methods, attributes,
heredoc, declared `Exception` subclasses, property by-reference
assignment, `Exception::getTrace()`, `array_walk()` by-reference userdata,
nullable types, generator `yield`, first-class callables, dynamic includes,
unsupported internals, scalar-offset lvalues, `Traversable`, INI modes,
process boundaries, formatter/callback parity, and classifier batching.
