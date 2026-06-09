# PTN Status

Last refresh: 2026-06-09T16:41Z
Commit: `ea176b8`

## Operating Goal

Grow PHP-to-native support through generic compiler/runtime semantics while
keeping generated binaries independent of the PHP interpreter.

## Current Signal

Current mandate: solve copy-on-write first. Other work is allowed only when it
directly unblocks COW correctness or evidence.

Latest measured local tests: source unit 3/3 passing; native integration
268/268 passing. Latest parsed bounded PHPT log records 121/171 passing rows.

## Active Buckets

| Bucket | Count |
| --- | --- |
| Source unit tests | 3/3 passing |
| Native compiled snippets | 268/268 passing |
| PHPT Zend rows | 60/76 passing |
| PHPT ext/standard rows | 44/77 passing |
| PHPT tests/basic+func+lang | 17/18 passing |
| COW-focused tests | 0 dedicated rows yet |

## Rules

- Update `PROGRESS.md` and mirrors every 10 minutes.
- Report counts, not essays.
- Use only numeric counts in dashboard cells.
- Never claim broad PHP compatibility from row-specific patches.
