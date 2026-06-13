# PTN Progress

Refresh: 2026-06-14T00:46Z
Measured: `ptn-6x02` SensitiveParameterValue reflection after `ptn-vwyp`.

Slices cover callable/object, filesystem/string, property, COW, by-ref
diagnostics, generator/Fiber blockers, and sensitive-parameter reflection.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 705 | 705 | 0 |
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
| PHPT broad reference-call bucket | 12 | 9 | 3 |
| PHPT generator/fiber COW boundary bucket | 12 | 0 | 12 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- Accepted bounded rows remain 485/485; classify-only reports 459 runnable and
  26 excluded.
- Array-internal COW frontier is 72/17/17; foreach/reference COW frontier is
  103/51/31.
- Broad reference-call bucket is 12 selected, 11 runnable, 1 excluded, 9
  passing; residuals cover append args and class-name checks.
- Static SKIPIF current 1k classify-only is 447 runnable and 553 excluded;
  worker 5k is 2,070 runnable.
- Generator/fiber COW boundary bucket is 12 selected, 0 runnable, 12 excluded.

## Verification

`ptn-vwyp` classifies the broad COW generator/fiber bucket before it enters pass
counts: COW classify-only selected 46 rows, kept 32 runnable, and excluded 14;
classifier tests passed 18/18.

`ptn-begn` centralizes by-reference call arguments; current residual PHPTs are
`ptn-hpxo` and `ptn-yl7i`. `ptn-5sca` adds property refs, constructor
by-reference args, Exception/Error `message` throws, and classifier narrowing.
`ptn-awta` models static `--SKIPIF--` preconditions.

`ptn-feps` adds `array_walk()` warning-and-continue semantics for by-reference
userdata mismatches plus root-scoped object handle reuse. Focused
`array_walk_closure.phpt` reaches the remaining exception trace section.

`ptn-6x02` models `SensitiveParameterValue` as an internal object with a
private stored `value`, plus bounded `ReflectionClass::getProperty()` and
`ReflectionProperty::getValue()` access through shared object property metadata.
Evidence: full `cargo test` passed 705/705, smoke passed 6/6, and the rebased
broad reference-call bucket measured 12 selected, 11 runnable, 1 excluded,
passed 9/11 with residuals `ptn-hpxo` and `ptn-yl7i`.

Follow-ups remain typed properties, traits, magic methods, attributes,
broader `Exception` APIs, static-property refs, trace APIs, nullable types,
generator/Fiber execution, dynamic includes, unsupported internals,
`Traversable`, INI modes, and classifier batching.
