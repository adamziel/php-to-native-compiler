# PTN Status

Last refresh: 2026-06-09T16:33Z
Commit: `69d7e10`

## Operating Goal

Grow PHP-to-native support through generic compiler/runtime semantics while
keeping generated binaries independent of the PHP interpreter.

## Current Signal

Last known compact PHPT sample: 54 / 59 selected rows passing. The progress
patrol must replace this with fresh bounded telemetry every cycle.

## Active Buckets

| Bucket | State |
| --- | --- |
| Rust/unit tests | Keep green before merge |
| Native compiled snippets | Main proof path for new semantics |
| PHPT Zend rows | Good signal for language/runtime semantics |
| PHPT ext/standard rows | Current failures concentrate around array basics |
| Docs/status | Keep under 500 words per visible file |

## Rules

- Update `PROGRESS.md` and mirrors every 10 minutes.
- Report counts, not essays.
- Track ported, passing, failing, and still-needed work by bucket.
- Never claim broad PHP compatibility from row-specific patches.
