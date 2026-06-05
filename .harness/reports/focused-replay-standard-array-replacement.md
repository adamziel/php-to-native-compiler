# Focused Standard Array Replay Replacement

Agent: developer-148

Lane: 79, replacement for failed/non-live developer-106.

Scope: read-only M0 focused replay classification for selected standard array
rows from the blocked candidate gate
`phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`.
No compiler/runtime source files were edited, no full PHPT gate was run, and no
eval or variable-variable work was performed.

`DEVELOPMENT.md` was requested by the harness prompt but is not present in this
worktree.

## Evidence Roots

Accepted baseline:

- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Public/source head: `0b917f67a37d9ca9779d77f87173b628431c2425`
- Public score artifact: `7873/20294`

Blocked candidate:

- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Public/source head: `56fe9377fb46be00db5fdd30c966fdba406dc581`
- Public score artifact: `7197/20294`
- PASS-regression artifact: `regressions-from-latest-published-passes.txt`
  reports the accepted-vs-candidate regression set used here.

Shared PHPT checkout and wrapper:

- `/home/claude/php-src-phpt`
- `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`

Prior selector used:

- `.harness/reports/standard-array-replay-selector.md`

## Replay Preflight

The durable gate artifacts record historical `PHPC_BIN` paths under deleted
`/tmp` run roots. Both required historical binaries were unavailable during this
replacement lane, so accepted-vs-candidate execution replay could not be run
without rebuilding, which is outside this lane's low-CPU artifact/replay scope.

Preflight command:

```sh
ACC=/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67
CAND=/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377

for p in "$ACC" "$CAND" /home/claude/php-src-phpt /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper; do
  if [ -e "$p" ]; then printf 'exists %s\n' "$p"; else printf 'missing %s\n' "$p"; fi
done

for p in \
  "/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc" \
  "/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc"
do
  if [ -x "$p" ]; then printf 'executable %s\n' "$p"; else printf 'not-executable %s\n' "$p"; fi
done
```

Observed result:

```text
exists /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67
exists /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377
exists /home/claude/php-src-phpt
exists /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
not-executable /tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc
not-executable /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc
```

This lane therefore classifies rows from durable accepted/candidate result
artifacts. A true focused replay remains possible only after restored or
rebuilt accepted and candidate `phpc` binaries are supplied.

## Row Classification

All eight rows are in the blocked candidate's PASS-regression list. All eight
are accepted/latest-public `PASSED` rows, absent from candidate normalized
status, absent from candidate aggregate results, absent from candidate shard
logs searched by exact path, and have no `SKIPIF` section. The correct
classification for each row is `control-plane absent`.

| Row | Accepted status | Candidate status | Candidate `all-results.txt` | SKIPIF | Classification |
| --- | --- | --- | --- | --- | --- |
| `php-src/ext/standard/tests/array/array_chunk2.phpt` | `PASSED` | `ABSENT` | `ABSENT` | no | control-plane absent |
| `php-src/ext/standard/tests/array/array_count_values.phpt` | `PASSED` | `ABSENT` | `ABSENT` | no | control-plane absent |
| `php-src/ext/standard/tests/array/array_diff_single_array.phpt` | `PASSED` | `ABSENT` | `ABSENT` | no | control-plane absent |
| `php-src/ext/standard/tests/array/array_filter_basic.phpt` | `PASSED` | `ABSENT` | `ABSENT` | no | control-plane absent |
| `php-src/ext/standard/tests/array/array_map_basic.phpt` | `PASSED` | `ABSENT` | `ABSENT` | no | control-plane absent |
| `php-src/ext/standard/tests/array/array_merge.phpt` | `PASSED` | `ABSENT` | `ABSENT` | no | control-plane absent |
| `php-src/ext/standard/tests/array/array_walk/array_walk_basic1.phpt` | `PASSED` | `ABSENT` | `ABSENT` | no | control-plane absent |
| `php-src/ext/standard/tests/array/sort/array_multisort_basic1.phpt` | `PASSED` | `ABSENT` | `ABSENT` | no | control-plane absent |

This is not evidence of semantic failure for these rows. It is evidence that
the candidate gate did not preserve row-level results for them.

## Commands Run

Read required session documents:

```sh
sed -n '1,220p' AGENTS.md
sed -n '1,260p' docs/PROGRESS.md
sed -n '1,260p' docs/ARCHITECTURE.md
sed -n '1,260p' docs/SUPPORT.md
sed -n '1,260p' README.md
sed -n '1,260p' DEVELOPMENT.md
sed -n '1,220p' docs/LOOP_MEMORY.md
```

`DEVELOPMENT.md` failed with `No such file or directory`; the other required
files were present.

Confirm gate roots, scores, and recorded harness shape:

```sh
ACC=/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67
CAND=/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377

for d in "$ACC" "$CAND"; do
  printf '\n== %s ==\n' "$d"
  ls -1 "$d" | sed -n '1,120p'
  printf 'score: '; sed -n '1,8p' "$d/public-comparable-score.tsv" 2>/dev/null
  printf 'summary: '; sed -n '1,8p' "$d/pass-regression-summary.tsv" 2>/dev/null
done

sed -n '1,220p' "$ACC/environment.txt"
rg -n 'run-tests|PHPC_BIN|TEST_PHP|shard|temp-source|temp-target|phpc-phpt-wrapper|cargo-target|release/phpc' "$ACC/run_gate.sh"
sed -n '1,220p' "$CAND/environment.txt"
rg -n 'run-tests|PHPC_BIN|TEST_PHP|shard|temp-source|temp-target|phpc-phpt-wrapper|cargo-target|release/phpc' "$CAND/run_gate.sh"
```

Confirm source pins and commit availability:

```sh
git -C /home/claude/php-src-phpt rev-parse HEAD
git rev-parse HEAD
git rev-parse --verify 0b917f67a37d9ca9779d77f87173b628431c2425^{commit}
git rev-parse --verify 56fe9377fb46be00db5fdd30c966fdba406dc581^{commit}
```

Observed:

```text
f97ff597429a2fe633665a7e02d97c8077f9f90f
dc768f6b865ce9c96a6a9cdf8fae1c3c2daba310
0b917f67a37d9ca9779d77f87173b628431c2425
56fe9377fb46be00db5fdd30c966fdba406dc581
```

Generate the row table:

```sh
python - <<'PY'
from pathlib import Path
import re

acc=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67')
cand=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
php=Path('/home/claude/php-src-phpt')
rows=[
'php-src/ext/standard/tests/array/array_chunk2.phpt',
'php-src/ext/standard/tests/array/array_count_values.phpt',
'php-src/ext/standard/tests/array/array_diff_single_array.phpt',
'php-src/ext/standard/tests/array/array_filter_basic.phpt',
'php-src/ext/standard/tests/array/array_map_basic.phpt',
'php-src/ext/standard/tests/array/array_merge.phpt',
'php-src/ext/standard/tests/array/array_walk/array_walk_basic1.phpt',
'php-src/ext/standard/tests/array/sort/array_multisort_basic1.phpt',
]

def status_map(path):
    out={}
    for line in path.read_text(errors='replace').splitlines():
        parts=line.split('\t',1)
        if len(parts)==2:
            out[parts[1]]=parts[0]
    return out

acc_status=status_map(acc/'current-status.normalized.tsv')
cand_status=status_map(cand/'current-status.normalized.tsv')
reg=set((cand/'regressions-from-latest-published-passes.txt').read_text().splitlines())
base_pass=set((cand/'baseline-passes.normalized.txt').read_text().splitlines())
cand_pass=set((cand/'current-passes.normalized.txt').read_text().splitlines())
all_text=(cand/'all-results.txt').read_text(errors='replace')

print('| Row | In regression list | Accepted status | Candidate status | Candidate all-results | SKIPIF | Title |')
print('| --- | --- | --- | --- | --- | --- | --- |')
for row in rows:
    phpt=php/row.removeprefix('php-src/')
    text=phpt.read_text(errors='replace')
    m=re.search(r'--TEST--\n(.*?)(?=\n--[A-Z]+--|\Z)', text, re.S)
    title=' '.join(m.group(1).strip().split()) if m else ''
    skip='yes' if re.search(r'--SKIPIF--', text) else 'no'
    all_presence='present' if row in all_text else 'ABSENT'
    print(f"| `{row}` | {row in reg} | {acc_status.get(row,'MISSING')} | {cand_status.get(row,'ABSENT')} | {all_presence} | {skip} | {title} |")

print('\nset checks:')
for row in rows:
    print(row, 'baseline_pass=', row in base_pass, 'candidate_pass=', row in cand_pass)
PY
```

Search candidate artifacts for row-level diagnostics:

```sh
CAND=/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377

for row in \
php-src/ext/standard/tests/array/array_chunk2.phpt \
php-src/ext/standard/tests/array/array_count_values.phpt \
php-src/ext/standard/tests/array/array_diff_single_array.phpt \
php-src/ext/standard/tests/array/array_filter_basic.phpt \
php-src/ext/standard/tests/array/array_map_basic.phpt \
php-src/ext/standard/tests/array/array_merge.phpt \
php-src/ext/standard/tests/array/array_walk/array_walk_basic1.phpt \
php-src/ext/standard/tests/array/sort/array_multisort_basic1.phpt; do
  printf '\n== %s ==\n' "$row"
  rg -n --fixed-strings "$row" "$CAND"/current-status.normalized.tsv "$CAND"/all-results.txt "$CAND"/shard-*/stdout.log "$CAND"/shard-*/run-tests.log "$CAND"/shard-*/results.txt 2>/dev/null || printf 'no candidate artifact hit\n'
done
```

Each selected row printed `no candidate artifact hit`.

## Focused Replay Command Shape

These commands were not executed in this lane because the historical binaries
were unavailable. They are the exact focused replay shape to use once accepted
and candidate `PHPC_BIN` paths are restored.

```sh
REPLAY_ROOT=/tmp/phpt-focused-replay-lane79-standard-array
PHP_SRC=/home/claude/php-src-phpt
WRAPPER=/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper

install -d -m 700 "$REPLAY_ROOT"
cat > "$REPLAY_ROOT/standard-array.tests" <<'EOF'
/home/claude/php-src-phpt/ext/standard/tests/array/array_chunk2.phpt
/home/claude/php-src-phpt/ext/standard/tests/array/array_count_values.phpt
/home/claude/php-src-phpt/ext/standard/tests/array/array_diff_single_array.phpt
/home/claude/php-src-phpt/ext/standard/tests/array/array_filter_basic.phpt
/home/claude/php-src-phpt/ext/standard/tests/array/array_map_basic.phpt
/home/claude/php-src-phpt/ext/standard/tests/array/array_merge.phpt
/home/claude/php-src-phpt/ext/standard/tests/array/array_walk/array_walk_basic1.phpt
/home/claude/php-src-phpt/ext/standard/tests/array/sort/array_multisort_basic1.phpt
EOF
```

Accepted:

```sh
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
  -r "$REPLAY_ROOT/standard-array.tests" \
  -W "$ACCEPTED_OUT/results.txt" \
  -s "$ACCEPTED_OUT/run-tests.log" \
  --no-color \
  --set-timeout 65 \
  --temp-source "$PHP_SRC" \
  --temp-target "$ACCEPTED_OUT/phpt-tmp" \
  > "$ACCEPTED_OUT/stdout.log" \
  2> "$ACCEPTED_OUT/stderr.log"
```

Candidate:

```sh
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
  -r "$REPLAY_ROOT/standard-array.tests" \
  -W "$CANDIDATE_OUT/results.txt" \
  -s "$CANDIDATE_OUT/run-tests.log" \
  --no-color \
  --set-timeout 65 \
  --temp-source "$PHP_SRC" \
  --temp-target "$CANDIDATE_OUT/phpt-tmp" \
  > "$CANDIDATE_OUT/stdout.log" \
  2> "$CANDIDATE_OUT/stderr.log"
```

## Conclusion

The selected standard array rows should stay in the control-plane/completeness
bucket until replay binaries exist. Assigning runtime implementation from these
eight rows would be premature: the candidate artifacts have no per-row FAIL,
BORK, SKIP, or PASS evidence for them.
