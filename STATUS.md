# PTN Status

Last refresh: 2026-06-12T21:39Z
Measured: `ptn-xlc0` rebased on current `origin/master` `a89a563b2`.

## Operating Goal

Hold the RC line to generic PHP semantics while expanding toward broad
php-src PHPT coverage. Report numbers, not compatibility claims.

## Current Signal

Source unit tests 3/3; Native/compiler Rust suite 578/578; Native smoke matrix 6/6; PHPT bounded manifest 231/231; PHPT Zend rows 80/80; PHPT ext/standard rows 101/101.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 3/3 |
| Native/compiler Rust suite | 578/578 |
| Native smoke matrix | 6/6 |
| PHPT bounded manifest | 231/231 |
| PHPT Zend rows | 80/80 |
| PHPT ext/standard rows | 101/101 |
| PHPT focused stream rows | 2/2 |
| PHPT tests/basic+func+lang | 45/45 |
| PHPT other rows | 5/5 |
| PHPT COW manifest | 29/29 |
| Post-merge COW gate | 26/26 |
| PHPT callback manifest | 4/4 |
| PHPT include manifest | 2/2 |

## Rules

- Update `PROGRESS.md`, then run `tools/update-progress-mirrors.sh`.
- Keep mirrors compact and numeric.
- Never claim broad PHP compatibility from row-specific patches.
