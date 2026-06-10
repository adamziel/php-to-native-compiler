# PTN Status

Last refresh: 2026-06-10T11:04Z
Measured base: `origin/master` at `567c84106`

## Operating Goal

Solve generic PHP semantics first. COW remains a priority, but the `ptn-4yt`
compatibility wave is closed; remaining COW PHPT failures are tracked as
semantic gaps, not as stale epic work.

## Current Signal

Source unit tests: 3/3. Native compiled snippets: 344/344. Bounded PHPT
manifest: 150/200. COW evidence bundle: 126/126. Focused PHPT COW manifest:
24/29.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 3/3 |
| Native compiled snippets | 344/344 |
| Native smoke matrix | 6/6 |
| PHPT bounded manifest | 150/200 |
| PHPT Zend rows | 67/76 |
| PHPT ext/standard rows | 47/77 |
| PHPT tests/basic+func+lang | 34/45 |
| PHPT other rows | 2/2 |
| COW evidence bundle | 126/126 |
| PHPT COW manifest | 24/29 |

## Rules

- Update `PROGRESS.md` and mirrors every 10 minutes.
- Report numbers, not essays.
- Use numeric dashboard cells.
- Never claim broad PHP compatibility from row-specific patches.
