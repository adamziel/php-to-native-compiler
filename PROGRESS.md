# PTN Progress

Refresh: 2026-06-14T00:23Z
Measured: `ptn-2juv` broad 1k frontier map after `ptn-vwyp`.

Slices cover callable/object, filesystem/string, property, COW, by-ref
diagnostics, broad PHPT blockers, and generator/Fiber blockers.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 704 | 704 | 0 |
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
- Broad reference-call bucket is 12 selected, 11 runnable, 1 excluded, 8
  passing; residuals cover append args, class-name checks, and
  `SensitiveParameterValue`.
- Static SKIPIF 1k classify-only is 447 runnable and 553 excluded.
- Generator/fiber COW boundary bucket is 12 selected, 0 runnable, 12 excluded.

## Verification

`ptn-vwyp` classifies broad COW generator/fiber rows before they enter pass
counts: COW classify-only selected 46 rows, kept 32 runnable, and excluded 14;
classifier tests passed 18/18.

`ptn-2juv` refreshes broad 1k evidence: 1,000 selected rows, 447 runnable, and
553 classified. Runnable buckets are 276 `ext/standard`, 155 `Zend`, and 16
`tests/basic`. Largest array frontier groups are diff/intersect helpers at 68
rows and `array_chunk()` at 32 rows; the pre-rebase full run stalled in the Zend
bucket at `Zend/tests/ErrorException_getSeverity.phpt`.

Recent reference work covers by-reference calls, property refs,
preconditions, and `array_walk()` userdata handling.

Follow-ups remain typed properties, traits, magic methods, attributes,
Exception trace APIs, nullable types, generator/Fiber execution, dynamic
includes, unsupported internals, `Traversable`, and INI modes.
