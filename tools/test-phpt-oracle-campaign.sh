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
test_php="$corpus/test-php"
cat > "$test_php" <<'EOF'
#!/usr/bin/env bash
exec /home/claude/ptn-oracle/php-cli-8c63ec/bin/php "$@"
EOF
chmod +x "$test_php"
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
git -C "$corpus" add .gitignore run-tests.php test-php tests
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

oracle_prepare=$(
  "$runner" \
    --corpus "$corpus" \
    --php "$target_php" \
    --out-root "$root/oracle-runs" \
    --jobs 2 \
    --timeout 15 \
    --expected-revision "$revision" \
    --expected-count 6 \
    --expected-inventory-sha "$inventory_sha" \
    --prepare-only
)
oracle_run_dir=$(printf '%s\n' "$oracle_prepare" | awk -F= '$1 == "campaign_run_dir" { print $2; exit }')
[[ -n "$oracle_run_dir" && -d "$oracle_run_dir" ]]
[[ $(awk -F '\t' '$1 == "campaign_kind" { print $2 }' "$oracle_run_dir/metadata.tsv") == target-php-oracle ]]
[[ $(awk -F '\t' '$1 == "test_php_binary" { print $2 }' "$oracle_run_dir/metadata.tsv") == "$target_php" ]]
awk -F '\t' 'BEGIN { OFS="\t" }
  $1 == "schema" { $2=1 }
  $1 == "test_php_binary" ||
  $1 == "test_php_binary_sha256" ||
  $1 == "test_php_source_binary" ||
  $1 == "test_php_version" ||
  $1 == "test_source_revision" ||
  $1 == "test_source_root" ||
  $1 == "ledger_tool_sha256" { next }
  { print }
' "$oracle_run_dir/metadata.tsv" > "$oracle_run_dir/metadata.tsv.schema1"
mv "$oracle_run_dir/metadata.tsv.schema1" "$oracle_run_dir/metadata.tsv"
"$runner" --resume "$oracle_run_dir" --foreground >/dev/null
[[ $(awk -F '\t' '$1 == "state" { print $2 }' "$oracle_run_dir/status.tsv") == finished ]]
[[ $(awk -F '\t' '$1 == "PASS" { print $2 }' "$oracle_run_dir/attempts/0001/summary.tsv") == 4 ]]

output=$(
  "$runner" \
    --corpus "$corpus" \
    --php "$target_php" \
    --test-php "$test_php" \
    --test-source-revision "$revision" \
    --test-source-root "$corpus" \
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
[[ $(awk -F '\t' '$1 == "campaign_kind" { print $2 }' "$run_dir/metadata.tsv") == strict-native-compiler ]]
[[ $(awk -F '\t' '$1 == "test_php_binary" { print $2 }' "$run_dir/metadata.tsv") == "$run_dir/bin/php-under-test" ]]
[[ $(awk -F '\t' '$1 == "test_php_source_binary" { print $2 }' "$run_dir/metadata.tsv") == "$test_php" ]]
[[ $(awk -F '\t' '$1 == "test_source_revision" { print $2 }' "$run_dir/metadata.tsv") == "$revision" ]]
[[ $(awk -F '\t' '$1 == "test_source_root" { print $2 }' "$run_dir/metadata.tsv") == "$corpus" ]]
[[ -x "$run_dir/bin/php-under-test" && -x "$run_dir/phpt-ledger.py" ]]

status="$run_dir/status.tsv"
attempt="$run_dir/attempts/0001"
[[ $(awk -F '\t' '$1 == "state" { print $2 }' "$status") == finished ]]
[[ $(awk -F '\t' '$1 == "ledger_state" { print $2 }' "$status") == complete ]]
[[ $(awk -F '\t' '$1 == "run_tests_exit" { print $2 }' "$status") == 0 ]]
[[ $(awk -F '\t' '$1 == "campaign_kind" { print $2 }' "$status") == strict-native-compiler ]]
[[ $(awk -F '\t' '$1 == "test_source_revision" { print $2 }' "$status") == "$revision" ]]
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
