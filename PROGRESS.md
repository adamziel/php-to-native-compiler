# PTN Progress

Refresh: 2026-06-13T21:46Z
Measured: `ptn-8d2u` Closure callable/reference frontier.

Slices cover callable/object, filesystem/string, property, COW, PHPT blockers,
readonly, userland `throw`, and Closure reference boundaries.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 688 | 688 | 0 |
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

- Accepted bounded rows remain 485/485; classify-only reports 459 runnable
  and 26 excluded rows outside the string slice.
- COW frontiers: array-internal 72/0/72; foreach/reference 103 selected,
  51 runnable, 31 passing.
- Zend arrow rows 001-004 pass; remaining rows need closure metadata, nullable
  types, generator `yield`, or `assert.exception` ini.

## Verification

`ptn-8d2u` adds Closure `use` validation, capture-preserving
`Closure::bindTo()` clones, `Closure::fromCallable()` wrappers,
`ReflectionFunction` name/count metadata, and `Closure::__invoke` by-reference
diagnostics. Gates: closure 8/8, call_user_func 3/3, reflection_function 1/1,
is_callable 2/2, COW 26/26, smoke 6/6, `cargo build --bin phpc`, and inventory
688 native/compiler plus 3 source tests.

Recent gates: `ptn-qsmv.11` userland `throw` support reran pass rows 2/2 after
readonly rebase; remaining blockers are `ptn-5sca`, `ptn-c284`, and
`ptn-feps`. `ptn-qsmv.12` readonly gates passed native/parser 5/5, classifier
15/15, and Zend PHPT 8/8 runnable rows. `ptn-550s.9` Zend foreach
object/property rerun passed 6/6. `ptn-x8p9` maps 73 `unsupported-ini` rows;
`ptn-flje` COW classify-only is 46 selected, 31 runnable, 15 excluded.

Follow-ups remain typed properties, traits, magic methods, attributes, heredoc,
declared `Exception` subclasses, property by-reference assignment,
`Exception::getTrace()`, `array_walk()` by-reference userdata, nullable types,
generator `yield`, first-class callables, dynamic includes, unsupported
internals, scalar-offset lvalues, `Traversable`, INI modes, process
boundaries, formatter/callback parity, and classifier batching.
