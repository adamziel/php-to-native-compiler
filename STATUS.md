# PTN Status

Last refresh: 2026-06-09T16:56Z
Commit: `70e7254bcae0`

## Mandate

Copy-on-write first. Non-COW implementation work allowed: 0, except direct COW
prerequisites.

## Current Signal

Local tests: 276/276 passed. Bounded PHPT: 138/200 passed, 62/200 failed,
0/200 skipped, 0/200 warned. COW-adjacent PHPT failures: 32/200.

## Active Buckets

| Bucket | Ported | Pass | Fail | Need |
| --- | ---: | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 | 0 |
| Native compiled snippets | 273 | 273 | 0 | 0 |
| COW-adjacent native | 1 | 1 | 0 | 5 |
| PHPT bounded total | 200 | 138 | 62 | 62 |
| PHPT Zend rows | 76 | 60 | 16 | 16 |
| PHPT ext/standard rows | 77 | 46 | 31 | 31 |
| PHPT tests/* rows | 47 | 32 | 15 | 15 |

## COW Blockers

5: shared payload refcounts, detach-on-write, nested container cloning,
by-value `foreach` mutation visibility, function-boundary value separation.
