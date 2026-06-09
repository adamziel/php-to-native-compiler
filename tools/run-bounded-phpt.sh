#!/usr/bin/env bash
set -euo pipefail

manifest=${1:-tools/phpt-bounded-manifest.txt}
php_src=${PHP_SRC_PHPT:-/home/claude/php-src-phpt}
out_dir=${PHPT_PROGRESS_DIR:-.runtime/phpt-progress}

if [[ ! -f "$manifest" ]]; then
    echo "manifest not found: $manifest" >&2
    exit 1
fi

if [[ ! -f "$php_src/run-tests.php" ]]; then
    echo "PHP source checkout not found or missing run-tests.php: $php_src" >&2
    exit 1
fi

mkdir -p "$out_dir"

timestamp=$(date -u +%Y%m%dT%H%M%SZ)
resolved_manifest="$out_dir/manifest-$timestamp.txt"
log="$out_dir/run-$timestamp.log"
summary="$out_dir/summary-$timestamp.txt"

tests=()
while IFS= read -r line || [[ -n "$line" ]]; do
    line=${line%%#*}
    line=${line#"${line%%[![:space:]]*}"}
    line=${line%"${line##*[![:space:]]}"}
    [[ -z "$line" ]] && continue

    test_path="$php_src/$line"
    if [[ ! -f "$test_path" ]]; then
        echo "manifest entry missing from corpus: $line" >&2
        exit 1
    fi

    tests+=("$test_path")
    printf '%s\n' "$line" >> "$resolved_manifest"
done < "$manifest"

if [[ "${#tests[@]}" -eq 0 ]]; then
    echo "manifest contains no runnable rows after comments/blank lines: $manifest" >&2
    exit 1
fi

cargo build --bin phpc

phpc_bin=${PHPC_BIN:-$PWD/target/debug/phpc}
commit=$(git rev-parse --short=12 HEAD)
start_epoch=$(date +%s)

set +e
PHPC_BIN="$phpc_bin" php "$php_src/run-tests.php" -q -p "$phpc_bin" "${tests[@]}" 2>&1 | tee "$log"
run_status=${PIPESTATUS[0]}
set -e

end_epoch=$(date +%s)
elapsed=$((end_epoch - start_epoch))

extract_count() {
    local label=$1
    awk -F: -v label="$label" '
        {
            name = $1
            gsub(/[[:space:]]+$/, "", name)
        }
        name == label {
            gsub(/^[[:space:]]+/, "", $2)
            split($2, parts, /[[:space:]]+/)
            print parts[1]
        }
    ' "$log" | tail -n 1
}

total=$(extract_count "Number of tests")
skipped=$(extract_count "Tests skipped")
warned=$(extract_count "Tests warned")
failed=$(extract_count "Tests failed")
passed=$(extract_count "Tests passed")

{
    echo "PHPT bounded patrol $timestamp"
    echo "commit: $commit"
    echo "corpus: $php_src"
    echo "manifest: $resolved_manifest"
    echo "log: $log"
    echo "command: cargo build --bin phpc; PHPC_BIN=\"$phpc_bin\" php $php_src/run-tests.php -q -p \"$phpc_bin\" <${#tests[@]} manifest paths>"
    echo "count: ${#tests[@]} selected PHPT rows"
    echo "result: passed=${passed:-unknown} failed=${failed:-unknown} skipped=${skipped:-unknown} warned=${warned:-unknown} elapsed=${elapsed}s"
    echo "run-tests-exit: $run_status"
} | tee "$summary"
