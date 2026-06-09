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
summary="$out_dir/summary-$timestamp.txt"
bucket_dir="$out_dir/buckets-$timestamp"

mkdir -p "$bucket_dir"
: > "$resolved_manifest"

trim() {
    local value=$1
    value=${value#"${value%%[![:space:]]*}"}
    value=${value%"${value##*[![:space:]]}"}
    printf '%s' "$value"
}

declare -a bucket_order=()
declare -A bucket_abs_manifest=()
declare -A bucket_count=()
declare -A bucket_expected=()
declare -A bucket_rel_manifest=()
declare -A bucket_slug=()

ensure_bucket() {
    local bucket=$1
    local expected=${2:-}

    if [[ ! -v "bucket_slug[$bucket]" ]]; then
        local slug
        slug=$(printf '%s' "$bucket" \
            | tr '[:upper:]' '[:lower:]' \
            | sed 's/[^a-z0-9._-]/-/g; s/--*/-/g; s/^-//; s/-$//')
        [[ -n "$slug" ]] || slug="bucket"

        local base=$slug
        local suffix=2
        while [[ -e "$bucket_dir/$slug.paths" || -e "$bucket_dir/$slug.txt" ]]; do
            slug="$base-$suffix"
            suffix=$((suffix + 1))
        done

        bucket_slug[$bucket]=$slug
        bucket_expected[$bucket]=$expected
        bucket_count[$bucket]=0
        bucket_rel_manifest[$bucket]="$bucket_dir/$slug.txt"
        bucket_abs_manifest[$bucket]="$bucket_dir/$slug.paths"
        : > "${bucket_rel_manifest[$bucket]}"
        : > "${bucket_abs_manifest[$bucket]}"
        bucket_order+=("$bucket")
    elif [[ -n "$expected" ]]; then
        bucket_expected[$bucket]=$expected
    fi
}

current_bucket=manifest
total_rows=0
while IFS= read -r line || [[ -n "$line" ]]; do
    trimmed=$(trim "$line")
    if [[ "$trimmed" =~ ^#[[:space:]]*bucket:[[:space:]]*([^[:space:]]+)[[:space:]]+rows=([0-9]+)([[:space:]]|$) ]]; then
        current_bucket=${BASH_REMATCH[1]}
        ensure_bucket "$current_bucket" "${BASH_REMATCH[2]}"
        continue
    fi

    row=${trimmed%%#*}
    row=$(trim "$row")
    [[ -z "$row" ]] && continue

    if [[ "$row" = /* ]]; then
        test_path="$row"
    else
        test_path="$php_src/$row"
    fi
    if [[ ! -f "$test_path" ]]; then
        echo "manifest entry missing from corpus: $row" >&2
        exit 1
    fi

    ensure_bucket "$current_bucket"
    printf '%s\n' "$row" >> "$resolved_manifest"
    printf '%s\n' "$row" >> "${bucket_rel_manifest[$current_bucket]}"
    printf '%s\n' "$test_path" >> "${bucket_abs_manifest[$current_bucket]}"
    bucket_count[$current_bucket]=$((bucket_count[$current_bucket] + 1))
    total_rows=$((total_rows + 1))
done < "$manifest"

if [[ "$total_rows" -eq 0 ]]; then
    echo "manifest contains no runnable rows after comments/blank lines: $manifest" >&2
    exit 1
fi

for bucket in "${bucket_order[@]}"; do
    expected=${bucket_expected[$bucket]:-}
    if [[ -n "$expected" && "${bucket_count[$bucket]}" -ne "$expected" ]]; then
        echo "bucket '$bucket' declares rows=$expected but contains ${bucket_count[$bucket]} rows" >&2
        exit 1
    fi
done

cargo build --bin phpc

phpc_bin=${PHPC_BIN:-$PWD/target/debug/phpc}
commit=$(git rev-parse --short=12 HEAD)
start_epoch=$(date +%s)

extract_count() {
    local label=$1
    local log=$2
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

require_count() {
    local label=$1
    local value=$2
    local bucket=$3

    if [[ ! "$value" =~ ^[0-9]+$ ]]; then
        echo "missing numeric '$label' count for bucket '$bucket'" >&2
        exit 1
    fi
}

aggregate_total=0
aggregate_skipped=0
aggregate_warned=0
aggregate_failed=0
aggregate_passed=0
aggregate_run_status=0

: > "$summary"

{
    echo "PHPT bounded patrol $timestamp"
    echo "commit: $commit"
    echo "corpus: $php_src"
    echo "manifest: $resolved_manifest"
    echo "command: cargo build --bin phpc; PHPC_BIN=\"$phpc_bin\" php $php_src/run-tests.php -q -p \"$phpc_bin\" <bucket manifest paths>"
    echo "count: $total_rows selected PHPT rows in ${#bucket_order[@]} buckets"
} | tee "$summary"

for bucket in "${bucket_order[@]}"; do
    slug=${bucket_slug[$bucket]}
    log="$out_dir/run-$timestamp-$slug.log"
    mapfile -t tests < "${bucket_abs_manifest[$bucket]}"
    bucket_start=$(date +%s)

    set +e
    PHPC_BIN="$phpc_bin" php "$php_src/run-tests.php" -q -p "$phpc_bin" "${tests[@]}" 2>&1 | tee "$log"
    run_status=${PIPESTATUS[0]}
    set -e

    bucket_end=$(date +%s)
    bucket_elapsed=$((bucket_end - bucket_start))

    total=$(extract_count "Number of tests" "$log")
    skipped=$(extract_count "Tests skipped" "$log")
    warned=$(extract_count "Tests warned" "$log")
    failed=$(extract_count "Tests failed" "$log")
    passed=$(extract_count "Tests passed" "$log")

    require_count "Number of tests" "$total" "$bucket"
    require_count "Tests skipped" "$skipped" "$bucket"
    require_count "Tests warned" "$warned" "$bucket"
    require_count "Tests failed" "$failed" "$bucket"
    require_count "Tests passed" "$passed" "$bucket"

    aggregate_total=$((aggregate_total + total))
    aggregate_skipped=$((aggregate_skipped + skipped))
    aggregate_warned=$((aggregate_warned + warned))
    aggregate_failed=$((aggregate_failed + failed))
    aggregate_passed=$((aggregate_passed + passed))
    if [[ "$run_status" -ne 0 ]]; then
        aggregate_run_status=$run_status
    fi

    {
        echo "bucket: $bucket rows=${bucket_count[$bucket]} tests=$total passed=$passed failed=$failed skipped=$skipped warned=$warned elapsed=${bucket_elapsed}s run-tests-exit=$run_status log=$log"
    } | tee -a "$summary"
done

end_epoch=$(date +%s)
elapsed=$((end_epoch - start_epoch))

{
    echo "result: buckets=${#bucket_order[@]} rows=$total_rows tests=$aggregate_total passed=$aggregate_passed failed=$aggregate_failed skipped=$aggregate_skipped warned=$aggregate_warned elapsed=${elapsed}s"
    echo "run-tests-exit: $aggregate_run_status"
} | tee -a "$summary"
