# PTN Status

Last refresh: 2026-06-09T18:53Z
Measured base: `ptn-cqu.47.15` rebased after `ptn-cqu.47.16`

## Operating Goal

Solve copy-on-write first. Other work is allowed only when it directly unblocks
COW correctness or COW evidence.

## Current Signal

Source unit tests: 4/4. Native compiled snippets: 309/309. Bounded PHPT
manifest: 145/200. COW-focused native tests: 12/12. Focused PHPT COW manifest:
7/29.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 4/4 |
| Native compiled snippets | 309/309 |
| PHPT bounded manifest | 145/200 |
| PHPT Zend rows | 63/76 |
| PHPT ext/standard rows | 47/77 |
| PHPT tests/basic+func+lang | 33/45 |
| PHPT other rows | 2/2 |
| COW contract spec tests | 7/7 |
| COW-focused native tests | 12/12 |
| COW reducers/oracle/internals | 59/60 |
| PHPT COW manifest | 7/29 |

## Rules

- Update `PROGRESS.md` and mirrors every 10 minutes.
- Report numbers, not essays.
- Use numeric dashboard cells.
- Never claim broad PHP compatibility from row-specific patches.
