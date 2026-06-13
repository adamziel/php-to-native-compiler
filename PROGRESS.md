# PTN Progress

Refresh: 2026-06-13T20:06Z
Measured: `ptn-qsmv.10` arrow-function parser/lowering after `ptn-550s.2`.

Recent RC slices cover constants, includes, closures, object callables, PHPT
blockers, streams, filesystem/path helpers, strings, asymmetric property set
visibility, COW maps, function-boundary PHPT, quiet string-offset diagnostics,
foreach/reference COW classification, and arrow functions with implicit
lexical captures.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 664 | 664 | 0 |
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

- No known failures among the 485 accepted bounded rows. Classify-only reports
  459 runnable rows and 26 excluded rows outside the string slice.
- Array-internal COW frontier: 72 selected, 0 runnable, 72 excluded
  (58 internal, 9 language, 5 class-metadata). COW foreach/reference frontier:
  103 selected, 51 runnable, 31 passing, 72 needs work.
- Zend arrow rows are source-specific: 001-004 pass; 005 needs
  `ReflectionFunction::getClosureThis`; 006/007 need nullable type hints;
  008 needs generator `yield`; `gh7900` needs `assert.exception` ini.

## Verification

`ptn-qsmv.10` lowers `fn`/`static fn` through the closure path with implicit
by-value captures, nested capture propagation, by-reference returns,
typed/variadic params, quiet missing-capture reads, and static-fn `$this`
exclusion. Focused checks: `cargo test --test compile_native arrow` 4/4,
`cargo test --test phpt_classifier` 12/12, and Zend arrow PHPT 001-004 4/4.

Recent COW gates remain: `ptn-550s.5` string/scalar alias PHPT 23/23,
`ptn-550s.4` COW manifest 54/54, and `ptn-550s.3` array-internal COW
frontier 72/0/72. Follow-ups remain typed properties, traits, magic methods,
attributes, heredoc/nowdoc, userland `throw`, readonly metadata, nullable
types, generator `yield`, first-class callables, dynamic includes, unsupported
internals, scalar-offset lvalues, `Traversable`, formatter/callback parity,
process boundaries, and classifier scan batching.
