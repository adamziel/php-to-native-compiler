# PTN Progress

Refresh: 2026-06-13T21:34Z
Measured: `ptn-qsmv.12` readonly class/property metadata after gates.

Slices cover callable/object, filesystem/string, property, COW, `array_walk()`,
PHPT blockers, and readonly metadata. `ptn-qsmv.12` adds readonly parsing,
write-once properties, inheritance guards, and classifier narrowing.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 685 | 685 | 0 |
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

`ptn-qsmv.12` parses `readonly class` and readonly property modifiers, carries
readonly metadata through AST/IR/native declarations, enforces write-once
initialization, reports uninitialized reads, rejects dynamic properties on
readonly classes, and keeps unsupported readonly rows classified separately.
Refinery gates: readonly native/parser tests 5/5, PHPT classifier 15/15, and
targeted Zend readonly PHPT selected 12, runnable 8, excluded 4, passed 8/8.

`ptn-550s.9` focused Zend foreach object/property rerun passed 6/6:
`bug34310`, `bug39017`, `bug39825`, `foreach_010`, `foreach_018`, and
`foreach_by_ref_to_property`. `ptn-x8p9` maps 73 `unsupported-ini` rows;
`assert.exception` probe passed 5/17, so it is blocker evidence.

`ptn-flje` COW classify-only is 46 selected, 31 runnable,
15 excluded. `ptn-550s.8` reruns seven `array_walk()` rows: six pass and
`bug39576` is class-metadata.

Follow-ups remain typed properties, traits, magic methods, attributes,
heredoc/nowdoc, `throw`, nullable types, generator `yield`, first-class
callables, dynamic includes, unsupported internals, scalar-offset lvalues,
`Traversable`, INI runtime modes, process boundaries, formatter/callback
parity, and classifier scan batching.
