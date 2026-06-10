# PTN Status

Last refresh: 2026-06-10T10:46Z
Measured base: `origin/master` `e7d39874`

## Operating Goal

Solve copy-on-write first. Other work is allowed only when it directly unblocks
COW correctness or COW evidence.

## Current Signal

Source unit tests: 3/3. Native compiled snippets: 344/344. Bounded PHPT
manifest: 150/200. Focused PHPT COW manifest: 24/29. Native COW gates:
contract 7/7, reducers/oracles/internals 79/79, recursive diagnostics 9/9,
post-merge gate 25/25.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 3/3 |
| Native compiled snippets | 344/344 |
| PHPT bounded manifest | 150/200 |
| PHPT Zend rows | 67/76 |
| PHPT ext/standard rows | 47/77 |
| PHPT tests/basic+func+lang | 34/45 |
| PHPT other rows | 2/2 |
| COW contract spec tests | 7/7 |
| COW reducers/oracle/internals | 79/79 |
| Recursive reference diagnostics | 9/9 |
| Post-merge COW gate | 25/25 |
| PHPT COW manifest | 24/29 |

## Rules

- Update `PROGRESS.md` and mirrors every 10 minutes.
- Report numbers, not essays.
- Use numeric dashboard cells.
- Never claim broad PHP compatibility from row-specific patches.
