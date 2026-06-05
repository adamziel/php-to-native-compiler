# Accepted Score Accounting Audit

Lane: 26, developer-83

Scope: reconcile the published `7873 / 20294` accepted score against the
available gate artifacts, and explain the `7869`, `7197`, and `7196` counts
without moving public score.

## Evidence

- Planning state: `/home/claude/php-to-native-compiler/PLAN.md`
- Accepted gate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Blocked candidate gate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Harness SQLite goal measure in
  `/home/claude/php-to-native-compiler/.harness/harness.sqlite3`

`PLAN.md` is in the stale root checkout, not this worktree, but its current
authoritative-state section matches the SQLite goal: accepted public score
stays `7873 / 20294 = 38.79%` at `0b917f67`, and the `221205Z` candidate gate
is blocked at `7197 / 20294 = 35.46%` with `1166` PASS regressions.

## Reconciliation

| Count | Source | Meaning |
| --- | --- | --- |
| `7873` | accepted `public-comparable-score.tsv`, accepted `counts.tsv` | Raw aggregate PASS count used for the accepted public score. |
| `7869` | accepted `current-passes.normalized.txt`, accepted `pass-regression-summary.tsv` `current_passes` | Canonical normalized PASS row set used as the next regression baseline. |
| `7197` | candidate `public-comparable-score.tsv`, candidate `counts.tsv` | Raw aggregate PASS count from the blocked candidate gate. It is not accepted public progress. |
| `7196` | candidate `current-passes.normalized.txt`, candidate `pass-regression-summary.tsv` `current_passes` | Canonical normalized candidate PASS row set used for regression comparison. |
| `1166` | candidate `regressions-from-latest-published-passes.txt`; candidate `pass-regression-summary.tsv` | Latest-public PASS rows present in accepted normalized passes and absent from candidate normalized passes. |

The `7873` vs `7869` difference is raw aggregate PASS accounting versus
normalized pass-row accounting. In the accepted gate, `all-results.txt`
contains four raw PASSED entries with non-canonical PDO `common.phpt: ...`
paths that do not appear in `current-passes.normalized.txt`:

- `php-src/ext/pdo_mysql/tests/common.phpt: .../php-src/ext/pdo/tests/pdo_037.phpt`
- `php-src/ext/pdo_odbc/tests/common.phpt: .../php-src/ext/pdo/tests/pdo_037.phpt`
- `php-src/ext/pdo_pgsql/tests/common.phpt: .../php-src/ext/pdo/tests/pdo_037.phpt`
- `php-src/ext/pdo_sqlite/tests/common.phpt: .../php-src/ext/pdo/tests/pdo_037.phpt`

The `7197` vs `7196` difference is the same raw-versus-normalized split. In
the candidate gate, one raw PASSED non-canonical PDO row is excluded from
`current-passes.normalized.txt`:

- `php-src/ext/pdo_odbc/tests/common.phpt: .../php-src/ext/pdo/tests/pdo_037.phpt`

The regression math is set-based over normalized pass rows:

- accepted normalized passes: `7869`
- candidate normalized passes: `7196`
- still-passing accepted rows: `6703`
- candidate passes not in accepted normalized baseline: `493`
- accepted normalized rows missing from candidate: `1166`

`regressions-from-latest-published-passes.txt` exactly equals:

```text
accepted current-passes.normalized.txt
minus
candidate current-passes.normalized.txt
```

Those `1166` regressions block public score movement. The `493` candidate-only
passes are useful signal for future repair/feature work, but they do not offset
latest-public PASS regressions under the current metric.

## Artifact Checks

Commands run for this audit:

```sh
wc -l \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/current-passes.normalized.txt \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/public-comparable-score.tsv \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/pass-regression-summary.tsv \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/counts.tsv \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/regressions-from-latest-published-passes.txt
```

```sh
sed -n '1,120p' \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/public-comparable-score.tsv \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/pass-regression-summary.tsv \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/counts.tsv \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/public-comparable-score.tsv \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/pass-regression-summary.tsv \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/counts.tsv
```

A Python set comparison checked:

- candidate regression rows equal accepted normalized passes minus candidate
  normalized passes
- all `1166` regression rows are accepted normalized passes
- no regression row appears in candidate normalized passes
- candidate normalized passes contain `493` rows not in the accepted normalized
  baseline

Selected evidence hashes exist in each run's `evidence-files.sha256` for
`counts.tsv`, `current-passes.normalized.txt`, `pass-regression-summary.tsv`,
`public-comparable-score.tsv`, and the candidate regression list.

Both gates report `invalid_marker_hits	0`.

## Conclusion

The accepted public score remains `7873 / 20294 = 38.79%`. The blocked
candidate gate remains `7197 / 20294 = 35.46%` and must not move public score
because it has `1166` latest-public PASS regressions.

Use `7873` and `7197` for raw public score summaries. Use `7869` and `7196`
when discussing normalized pass-set regression accounting.
