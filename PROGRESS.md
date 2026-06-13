# PTN Progress

Refresh: 2026-06-13T23:47Z
Measured: `ptn-feps` array_walk closure userdata parity after `ptn-awta`.

Slices cover callable/object, filesystem/string, property, COW, PHPT blockers,
by-ref diagnostics, and SKIPIF preconditions.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 703 | 703 | 0 |
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
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- Accepted bounded rows remain 485/485; classify-only reports 459 runnable and
  26 excluded.
- Array-internal COW frontier is 72/17/17; foreach/reference COW frontier is
  103/51/31.
- Broad reference-call bucket is 12 selected, 11 runnable, 1 excluded, 8
  passing; residuals cover append args, class-name checks, and
  `SensitiveParameterValue`.
- Static SKIPIF modeling current 1k classify-only is 447 runnable, 553
  excluded, 0 `harness-skipif`, 2 `skipif-precondition`; worker 5k evidence is
  2,070 runnable, 275 `harness-skipif`, 27 `skipif-precondition`.

## Verification

`ptn-begn` centralizes by-reference call arguments across direct calls,
`call_user_func*()`, methods, and bounded callable `array_multisort()`;
residual PHPTs are `ptn-hpxo`, `ptn-yl7i`, and `ptn-6x02`.

`ptn-5sca` adds property refs, constructor by-reference args, Exception/Error
`message` throws, and classifier narrowing. `ptn-awta` models static
`--SKIPIF--` sanitizer, `PHP_INT_SIZE`, and host-locale preconditions.

`ptn-feps` adds `array_walk()` warning-and-continue semantics for by-reference
userdata mismatches plus root-scoped object handle reuse. Focused evidence:
`cargo test array_walk`, object-handle reuse 1/1, and smoke 6/6; focused
`array_walk_closure.phpt` reaches the remaining exception trace section.

Follow-ups remain typed properties, traits, magic methods, attributes, heredoc,
broader `Exception` APIs, static-property refs, `Exception::getTrace()`,
array_walk closure exception traces, nullable types, generator `yield`, dynamic
includes, unsupported internals, `Traversable`, INI modes, and classifier
batching.
