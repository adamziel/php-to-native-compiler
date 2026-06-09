# PTN Status

Last refresh: 2026-06-09T17:54Z
Measured base: `ptn-cqu.47.11` rebased after `ptn-cqu.47.6`

## Operating Goal

Solve copy-on-write first. Other work is allowed only when it directly unblocks
COW correctness or COW evidence.

## Current Signal

Source unit tests: 3/3. Native compiled snippets: 287/287. Parsed bounded PHPT
rows: 121/171. COW-focused native tests: 5/5. Focused PHPT COW manifest: 2/29.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 3/3 |
| Native compiled snippets | 287/287 |
| PHPT Zend rows | 60/76 |
| PHPT ext/standard rows | 44/77 |
| PHPT tests/basic+func+lang | 17/18 |
| COW contract spec tests | 5/5 |
| COW-focused native tests | 5/5 |
| PHPT COW manifest | 2/29 |

## Rules

- Update `PROGRESS.md` and mirrors every 10 minutes.
- Report numbers, not essays.
- Use numeric dashboard cells.
- Never claim broad PHP compatibility from row-specific patches.
