# Focused PHPT Replay Cookbook

Lane 29 scope: read-only artifact audit for the blocked candidate gate
`phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`.
This cookbook is diagnostic only. Focused replay of a few PHPT rows cannot move
the accepted public score and must not be reported as a full-suite score gate.

No compiler/runtime source edits and no full PHPT gate are part of this
cookbook.

## Evidence Roots

Accepted baseline:

- Evidence directory: `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Gate script: `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/run_gate.sh`
- Public/source head: `0b917f67a37d9ca9779d77f87173b628431c2425`
- php-src pin: `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- Public score artifact: `public-comparable-score.tsv` reports `7873/20294`
- Accepted current-pass baseline used by the candidate: `current-passes.normalized.txt`
- Accepted normalized statuses: `current-status.normalized.tsv`
- Accepted complete rows: `all-results.txt`
- Accepted preflight: `current-score-gate-preflight.tsv`

Blocked candidate:

- Evidence directory: `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Gate script: `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/run_gate.sh`
- Worker status: `/home/claude/supervised-php-compiler/state/workers/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377.status.md`
- Public/source head: `56fe9377fb46be00db5fdd30c966fdba406dc581`
- php-src pin: `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- Public score artifact: `public-comparable-score.tsv` reports `7197/20294`
- Preflight: `current-score-gate-preflight.tsv`
- Candidate normalized passes: `current-passes.normalized.txt`
- Candidate normalized statuses: `current-status.normalized.tsv`
- Candidate complete rows: `all-results.txt`
- Regression rows: `regressions-from-latest-published-passes.txt`
- Regression summary: `pass-regression-summary.tsv` reports `1166` PASS regressions
- Candidate baseline copy: `baseline-passes.normalized.txt`

Shared wrapper and php-src evidence:

- Wrapper: `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`
- Shared php-src checkout: `/home/claude/php-src-phpt`
- Shared php-src checkout currently resolves to `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- Source seed: `/home/claude/supervised-php-compiler/state/worktrees/batch024-current-supervisor-20260601T093225`

## Missing Replay Evidence

Immediate replay with the historical binaries is blocked because these
historical run roots and binaries are no longer present:

- `/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- `/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc`
- `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc`

The wrapper and pinned shared php-src checkout exist, and both accepted and
candidate commits are present in the source seed and this repository. To replay
now, restore or rebuild release `phpc` binaries for the accepted and candidate
commits, then substitute the resulting paths for `PHPC_BIN` below. Do not run
`run_gate.sh` for focused replay.

## Sample Rows

Selected rows are from the candidate regression artifact. They avoid eval and
variable-variable areas.

| PHPT row | Accepted status | Candidate status |
| --- | --- | --- |
| `php-src/ext/bcmath/tests/number/properties_unset.phpt` | `PASSED` | `FAILED` |
| `php-src/ext/date/tests/DatePeriod_properties2.phpt` | `PASSED` | `FAILED` |
| `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt` | `PASSED` | `BORKED` |
| `php-src/ext/phar/tests/bug79797.phpt` | `PASSED` | missing from candidate normalized status |
| `php-src/ext/posix/tests/001.phpt` | `PASSED` | missing from candidate normalized status |

The candidate regression set has three explicit `BORKED` rows, twenty-seven
explicit `FAILED` rows, and 1136 rows that are in the accepted pass baseline but
missing from candidate normalized status.

## Choosing 3-5 Rows

Use artifact joins only. Do not run a gate.

```sh
ACC=/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67
CAND=/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377

# Candidate regression rows with explicit candidate statuses.
awk -F '\t' '
  NR==FNR { reg[$0]=1; next }
  $2 in reg && ($1=="FAILED" || $1=="BORKED") { print $1 "\t" $2 }
' "$CAND/regressions-from-latest-published-passes.txt" \
  "$CAND/current-status.normalized.tsv" |
  rg -v '(^|/)(eval|variable|variables|var)' |
  sed -n '1,20p'

# Candidate regression rows that are absent from candidate status output.
awk -F '\t' '
  NR==FNR { reg[$0]=1; next }
  $2 in reg { seen[$2]=1 }
  END { for (p in reg) if (!(p in seen)) print "MISSING\t" p }
' "$CAND/regressions-from-latest-published-passes.txt" \
  "$CAND/current-status.normalized.tsv" |
  sort |
  rg -v '(^|/)(eval|variable|variables|var)' |
  sed -n '1,20p'

# Confirm accepted-vs-candidate status for chosen rows.
for row in \
  php-src/ext/bcmath/tests/number/properties_unset.phpt \
  php-src/ext/date/tests/DatePeriod_properties2.phpt \
  php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt \
  php-src/ext/phar/tests/bug79797.phpt \
  php-src/ext/posix/tests/001.phpt
do
  printf '%s\n' "$row"
  printf '  accepted: '
  awk -F '\t' -v p="$row" '$2==p { print $1; found=1; exit } END { if (!found) print "MISSING" }' \
    "$ACC/current-status.normalized.tsv"
  printf '  candidate: '
  awk -F '\t' -v p="$row" '$2==p { print $1; found=1; exit } END { if (!found) print "MISSING" }' \
    "$CAND/current-status.normalized.tsv"
done
```

Pick at least one explicit `FAILED`, one explicit `BORKED`, and one `MISSING`
candidate row when possible. Keep the row list to three to five PHPT files.

## Row List

Use absolute paths into the available pinned php-src checkout for focused
replay:

```sh
REPLAY_ROOT=/tmp/phpt-focused-replay-lane29
PHP_SRC=/home/claude/php-src-phpt

install -d -m 700 "$REPLAY_ROOT"
cat > "$REPLAY_ROOT/sample-regressions.tests" <<'EOF'
/home/claude/php-src-phpt/ext/bcmath/tests/number/properties_unset.phpt
/home/claude/php-src-phpt/ext/date/tests/DatePeriod_properties2.phpt
/home/claude/php-src-phpt/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt
/home/claude/php-src-phpt/ext/phar/tests/bug79797.phpt
/home/claude/php-src-phpt/ext/posix/tests/001.phpt
EOF
```

## Accepted Baseline Replay Shape

Historical gate values:

- `RUN_ROOT=/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- historical `PHPC_BIN=$RUN_ROOT/cargo-target/release/phpc`
- historical php-src cwd: `$RUN_ROOT/php-src`
- historical command used `php run-tests.php ...` from that php-src cwd

Focused replay shape using the available pinned php-src checkout:

```sh
REPLAY_ROOT=/tmp/phpt-focused-replay-lane29
PHP_SRC=/home/claude/php-src-phpt
WRAPPER=/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
ACCEPTED_OUT="$REPLAY_ROOT/accepted"

export PHPC_BIN=/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc
export TEST_PHP_EXECUTABLE="$WRAPPER"
export TEST_PHP_ARGS=
export TMPDIR="$ACCEPTED_OUT/tmp"
export TEMP="$TMPDIR"
export TMP="$TMPDIR"
export PHPC_PHPT_TIMEOUT_SECONDS=55
export PHPC_PHPT_KILL_AFTER_SECONDS=5
export PHPT_SYSTEM_PHP=php
export TEST_PHP_SRCDIR="$PHP_SRC"
export NO_INTERACTION=1

install -d -m 700 "$ACCEPTED_OUT" "$TMPDIR" "$ACCEPTED_OUT/phpt-tmp"
cd "$PHP_SRC"
php run-tests.php -q -n \
  -p "$TEST_PHP_EXECUTABLE" \
  -r "$REPLAY_ROOT/sample-regressions.tests" \
  -W "$ACCEPTED_OUT/results.txt" \
  -s "$ACCEPTED_OUT/run-tests.log" \
  --no-color \
  --set-timeout 65 \
  --temp-source "$PHP_SRC" \
  --temp-target "$ACCEPTED_OUT/phpt-tmp" \
  > "$ACCEPTED_OUT/stdout.log" \
  2> "$ACCEPTED_OUT/stderr.log"
```

This command shape is low-CPU because it runs only the sample row list with one
`run-tests.php` process. It is not runnable until `PHPC_BIN` points to an
existing accepted-baseline release binary.

## Candidate Replay Shape

Historical gate values:

- `RUN_ROOT=/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- historical `PHPC_BIN=$RUN_ROOT/cargo-target/release/phpc`
- historical php-src cwd: `$RUN_ROOT/php-src`
- historical command used a copied harness, `php "$RUN_ROOT/run-tests-harnesses/shard-N/run-tests.php" ...`, from the php-src cwd

Focused replay shape using the available pinned php-src checkout:

```sh
REPLAY_ROOT=/tmp/phpt-focused-replay-lane29
PHP_SRC=/home/claude/php-src-phpt
WRAPPER=/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
CANDIDATE_OUT="$REPLAY_ROOT/candidate"

export PHPC_BIN=/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc
export TEST_PHP_EXECUTABLE="$WRAPPER"
export TEST_PHP_ARGS=
export TMPDIR="$CANDIDATE_OUT/tmp"
export TEMP="$TMPDIR"
export TMP="$TMPDIR"
export PHPC_PHPT_TIMEOUT_SECONDS=55
export PHPC_PHPT_KILL_AFTER_SECONDS=5
export PHPT_SYSTEM_PHP=php
export TEST_PHP_SRCDIR="$PHP_SRC"
export NO_INTERACTION=1

install -d -m 700 "$CANDIDATE_OUT" "$TMPDIR" "$CANDIDATE_OUT/phpt-tmp"
cd "$PHP_SRC"
php run-tests.php -q -n \
  -p "$TEST_PHP_EXECUTABLE" \
  -r "$REPLAY_ROOT/sample-regressions.tests" \
  -W "$CANDIDATE_OUT/results.txt" \
  -s "$CANDIDATE_OUT/run-tests.log" \
  --no-color \
  --set-timeout 65 \
  --temp-source "$PHP_SRC" \
  --temp-target "$CANDIDATE_OUT/phpt-tmp" \
  > "$CANDIDATE_OUT/stdout.log" \
  2> "$CANDIDATE_OUT/stderr.log"
```

This command shape is low-CPU because it runs only the sample row list with one
`run-tests.php` process. It is not runnable until `PHPC_BIN` points to an
existing candidate release binary.

## Interpreting Focused Results

Focused replay can confirm whether selected rows still reproduce accepted vs
candidate behavior under a restored binary. It does not validate shard
coverage, denominator, missing-row behavior, aggregation, or publication
eligibility. Only a full supervised current-score gate can move the public
score, and lane 29 explicitly does not run one.
