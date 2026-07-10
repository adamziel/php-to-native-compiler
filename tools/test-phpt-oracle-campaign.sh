#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
runner="$repo_root/tools/run-phpt-oracle-campaign.sh"
target_php=/home/claude/ptn-oracle/php-cli-8c63ec/bin/php
target_run_tests=/home/claude/ptn-oracle/php-src-8c63ec/run-tests.php

[[ -x "$target_php" ]] || { echo "target PHP is unavailable" >&2; exit 1; }
[[ -f "$target_run_tests" ]] || { echo "target run-tests.php is unavailable" >&2; exit 1; }

root=$(mktemp -d)
trap 'rm -rf "$root"' EXIT
corpus="$root/corpus"
mkdir -p "$corpus/tests"
cp "$target_run_tests" "$corpus/run-tests.php"

cat > "$corpus/.gitignore" <<'EOF'
*.diff
*.exp
*.log
*.out
*.php
*.phps
!run-tests.php
EOF

cat > "$corpus/tests/01-pass.phpt" <<'EOF'
--TEST--
oracle ledger pass
--FILE--
<?php echo "pass\n"; ?>
--EXPECT--
pass
EOF

cat > "$corpus/tests/02-skip.phpt" <<'EOF'
--TEST--
oracle ledger skip
--SKIPIF--
<?php echo "skip: fixture"; ?>
--FILE--
<?php echo "not run\n"; ?>
--EXPECT--
not run
EOF

cat > "$corpus/tests/03-xfail.phpt" <<'EOF'
--TEST--
oracle ledger xfail
--XFAIL--
fixture failure
--FILE--
<?php echo "actual\n"; ?>
--EXPECT--
expected
EOF

printf 'canonical\n' > "$corpus/tests/tracked-fixture.txt"
cat > "$corpus/tests/04-mutate-fixture.phpt" <<'EOF'
--TEST--
oracle campaign uses a disposable worktree
--FILE--
<?php
$fixture = __DIR__ . '/tracked-fixture.txt';
file_put_contents($fixture, "mutated\n");
echo file_get_contents($fixture);
?>
--EXPECT--
mutated
EOF

mkdir -p "$corpus/tests/redirected"
cat > "$corpus/tests/05-redirect.phpt" <<'EOF'
--TEST--
oracle ledger redirect parent
--REDIRECTTEST--
return array(
    'ENV' => array(),
    'TESTS' => __DIR__ . '/tests/redirected',
);
EOF

cat > "$corpus/tests/redirected/child.phpt" <<'EOF'
--TEST--
oracle ledger redirected child
--FILE--
<?php echo "redirected\n"; ?>
--EXPECT--
redirected
EOF

git -C "$corpus" init -q
git -C "$corpus" config user.email fixture@example.invalid
git -C "$corpus" config user.name fixture
git -C "$corpus" add .gitignore run-tests.php tests
git -C "$corpus" commit -qm fixture
revision=$(git -C "$corpus" rev-parse HEAD)
inventory="$root/inventory.txt"
(
  cd "$corpus"
  find . -path './.git' -prune -o -type f -name '*.phpt' -print |
    sed 's#^\./##' |
    LC_ALL=C sort > "$inventory"
)
inventory_sha=$(sha256sum "$inventory" | awk '{print $1}')

output=$(
  "$runner" \
    --corpus "$corpus" \
    --php "$target_php" \
    --out-root "$root/runs" \
    --jobs 2 \
    --timeout 15 \
    --expected-revision "$revision" \
    --expected-count 6 \
    --expected-inventory-sha "$inventory_sha" \
    --foreground
)
run_dir=$(printf '%s\n' "$output" | awk -F= '$1 == "campaign_run_dir" { print $2; exit }')
[[ -n "$run_dir" && -d "$run_dir" ]] || { echo "campaign run directory missing" >&2; exit 1; }
[[ -s "$run_dir/php-n-modules.txt" && -s "$run_dir/php-ini.txt" ]]

status="$run_dir/status.tsv"
attempt="$run_dir/attempts/0001"
[[ $(awk -F '\t' '$1 == "state" { print $2 }' "$status") == finished ]]
[[ $(awk -F '\t' '$1 == "ledger_state" { print $2 }' "$status") == complete ]]
[[ $(awk -F '\t' '$1 == "run_tests_exit" { print $2 }' "$status") == 0 ]]
[[ $(awk -F '\t' '$1 == "PASS" { print $2 }' "$attempt/summary.tsv") == 4 ]]
[[ $(awk -F '\t' '$1 == "SKIP" { print $2 }' "$attempt/summary.tsv") == 1 ]]
[[ $(awk -F '\t' '$1 == "XFAIL" { print $2 }' "$attempt/summary.tsv") == 1 ]]
[[ $(awk -F '\t' '$1 == "unresolved_inventory" { print $2 }' "$attempt/summary.tsv") == 0 ]]
[[ -s "$attempt/run-tests.log" && -s "$attempt/run-tests-results.tsv" && -s "$attempt/ledger.tsv" ]]
grep -q 'Spawning 2 workers' "$attempt/run-tests.log"
[[ ! -e "$attempt/tmp" ]]
[[ $(< "$corpus/tests/tracked-fixture.txt") == canonical ]]
[[ -z $(git -C "$corpus" status --porcelain=v1 --untracked-files=all) ]]
[[ $(< "$attempt/corpus/tests/tracked-fixture.txt") == mutated ]]
grep -q '^ M tests/tracked-fixture.txt$' "$attempt/worktree-status.txt"
awk -F '\t' '
  NR > 1 && $4 == "tests/redirected/child.phpt" && $5 == "tests/05-redirect.phpt" { found=1 }
  END { exit !found }
' "$attempt/ledger.tsv"
if grep -Fq "$attempt/corpus" "$attempt/ledger.tsv"; then
  echo "absolute attempt worktree path leaked into ledger" >&2
  exit 1
fi

set +e
resume_output=$("$runner" --resume "$run_dir" --foreground 2>&1)
resume_status=$?
set -e
[[ "$resume_status" -ne 0 && "$resume_output" == *"cannot be launched from state: finished"* ]]
[[ ! -e "$run_dir/attempts/0002" ]]

echo "test-phpt-oracle-campaign: ok"
