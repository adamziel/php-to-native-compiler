# PTN Status

Last refresh: 2026-06-13T12:48Z
Measured: `ptn-80xp` integration on current `origin/master` `d7431e8`;

## Operating Goal

Hold the RC line to generic PHP semantics while expanding toward broad
php-src PHPT coverage. Report numbers, not compatibility claims.

## Current Signal

Source unit tests 3/3; Native/compiler Rust suite 622/622; Native smoke matrix 6/6; PHPT bounded manifest 274/274; PHPT Zend rows 86/86; PHPT ext/standard rows 133/133.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 3/3 |
| Native/compiler Rust suite | 622/622 |
| Native smoke matrix | 6/6 |
| PHPT bounded manifest | 274/274 |
| PHPT Zend rows | 86/86 |
| PHPT ext/standard rows | 133/133 |
| PHPT focused stream rows | 2/2 |
| PHPT tests/basic+func+lang | 50/50 |
| PHPT other rows | 5/5 |
| PHPT COW manifest | 29/29 |
| Post-merge COW gate | 26/26 |
| PHPT callback manifest | 3/5 |
| PHPT include manifest | 2/2 |

## Rules

- Update `PROGRESS.md`, then run `tools/update-progress-mirrors.sh`.
- Keep mirrors compact and numeric.
- Never claim broad PHP compatibility from row-specific patches.
