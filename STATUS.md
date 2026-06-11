# PTN Status

Last refresh: 2026-06-11T20:31Z
Measured base: `origin/master` at `636306708`

## Operating Goal

Hold the RC line to generic PHP semantics. Do not trade the frozen PHPT cluster
map for row-shaped fixes.

## Current Signal

Source unit tests: 3/3. Rust native/compiler suite: 464/464. Native smoke:
6/6. Bounded PHPT manifest: 186/200. COW PHPT manifest: 29/29. Post-merge COW
gate: 25/25. Callback PHPT manifest: 2/2.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 3/3 |
| Native/compiler Rust suite | 464/464 |
| Native smoke matrix | 6/6 |
| PHPT bounded manifest | 186/200 |
| PHPT Zend rows | 76/76 |
| PHPT ext/standard rows | 69/77 |
| PHPT tests/basic+func+lang | 39/45 |
| PHPT other rows | 2/2 |
| PHPT COW manifest | 29/29 |
| Post-merge COW gate | 25/25 |
| PHPT callback manifest | 2/2 |

## Rules

- Update `PROGRESS.md` and mirrors every patrol.
- Report numbers, not essays.
- Never claim broad PHP compatibility from row-specific patches.
