# PTN Status

Last refresh: 2026-06-09T23:02Z
Measured base: `9d5c6070073e` (`ptn-cqu.26`) on `origin/master`

## Operating Goal

Solve copy-on-write first. Other work is allowed only when it directly unblocks
COW correctness or COW evidence.

## Current Signal

Source unit tests: 4/4. Native compiled snippets: 312/312. Bounded PHPT
manifest: 146/200. COW-focused native tests: 13/13. Focused PHPT COW manifest:
9/29.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 4/4 |
| Native compiled snippets | 312/312 |
| PHPT bounded manifest | 146/200 |
| PHPT Zend rows | 63/76 |
| PHPT ext/standard rows | 48/77 |
| PHPT tests/basic+func+lang | 33/45 |
| PHPT other rows | 2/2 |
| COW contract spec tests | 7/7 |
| COW-focused native tests | 13/13 |
| COW reducers/oracle/internals | 75/75 |
| PHPT COW manifest | 9/29 |

## Rules

- Update `PROGRESS.md` and mirrors every 10 minutes.
- Report numbers, not essays.
- Use numeric dashboard cells.
- Never claim broad PHP compatibility from row-specific patches.
