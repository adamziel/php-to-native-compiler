#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest="$repo_root/tools/phpt-bounded-manifest.txt"
classify_only=0
classify_harness_programs=${PTN_PHPT_CLASSIFY_HARNESS_PROGRAMS:-0}

usage() {
    cat <<'EOF'
Usage: tools/run-bounded-phpt.sh [--classify-only] [--classify-harness-programs] [manifest]

Classify a PHPT manifest, then run runnable rows through php-src run-tests.php.

Environment:
  PTN_PHPT_TEST_TIMEOUT        Per-test run-tests.php timeout. Defaults to 3600
                               seconds to allow native compile startup latency.
  PTN_PHPT_RUN_TESTS_JOBS      Parallel run-tests.php workers. Defaults to 4.

Options:
  --classify-only              write classification and blocker manifests without
                               building phpc or running runnable PHPT rows
  --classify-harness-programs  also classify SKIPIF precondition harness rows
EOF
}

manifest_set=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --classify-only)
            classify_only=1
            shift
            ;;
        --classify-harness-programs)
            classify_harness_programs=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        --)
            shift
            break
            ;;
        -*)
            echo "unknown option: $1" >&2
            usage >&2
            exit 2
            ;;
        *)
            if [[ "$manifest_set" -eq 1 ]]; then
                echo "multiple manifests supplied: $manifest and $1" >&2
                usage >&2
                exit 2
            fi
            manifest=$1
            manifest_set=1
            shift
            ;;
    esac
done

if [[ $# -gt 0 ]]; then
    if [[ "$manifest_set" -eq 1 ]]; then
        echo "multiple manifests supplied: $manifest and $1" >&2
        usage >&2
        exit 2
    fi
    manifest=$1
    shift
fi

if [[ $# -gt 0 ]]; then
    echo "unexpected extra arguments: $*" >&2
    usage >&2
    exit 2
fi

source "$repo_root/tools/phpt-corpus.sh"
export PTN_PHPT_CLASSIFY_HARNESS_PROGRAMS=$classify_harness_programs
source "$repo_root/tools/phpt-classifier.sh"
php_src="$(ptn_resolve_phpt_corpus "$repo_root")"
corpus_revision="$(ptn_phpt_corpus_revision "$php_src")"
out_dir=${PHPT_PROGRESS_DIR:-$repo_root/.runtime/phpt-progress}

if [[ ! -f "$manifest" ]]; then
    echo "manifest not found: $manifest" >&2
    exit 1
fi

mkdir -p "$out_dir"

timestamp=$(date -u +%Y%m%dT%H%M%SZ)-$$
resolved_manifest="$out_dir/manifest-$timestamp.txt"
runnable_manifest="$out_dir/runnable-$timestamp.txt"
classification_tsv="$out_dir/classification-$timestamp.tsv"
excluded_tsv="$out_dir/excluded-$timestamp.tsv"
excluded_dir="$out_dir/excluded-$timestamp"
summary="$out_dir/summary-$timestamp.txt"
bucket_dir="$out_dir/buckets-$timestamp"

mkdir -p "$bucket_dir"
mkdir -p "$excluded_dir"
: > "$resolved_manifest"
: > "$runnable_manifest"
: > "$classification_tsv"
: > "$excluded_tsv"

trim() {
    local value=$1
    value=${value#"${value%%[![:space:]]*}"}
    value=${value%"${value##*[![:space:]]}"}
    printf '%s' "$value"
}

declare -a bucket_order=()
declare -a excluded_category_order=()
declare -A bucket_abs_manifest=()
declare -A bucket_count=()
declare -A bucket_expected=()
declare -A bucket_rel_manifest=()
declare -A bucket_selected_count=()
declare -A bucket_slug=()
declare -A excluded_category_count=()
declare -A excluded_category_manifest=()

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
        bucket_selected_count[$bucket]=0
        bucket_rel_manifest[$bucket]="$bucket_dir/$slug.txt"
        bucket_abs_manifest[$bucket]="$bucket_dir/$slug.paths"
        : > "${bucket_rel_manifest[$bucket]}"
        : > "${bucket_abs_manifest[$bucket]}"
        bucket_order+=("$bucket")
    elif [[ -n "$expected" ]]; then
        bucket_expected[$bucket]=$expected
    fi
}

ensure_excluded_category() {
    local category=$1

    if [[ ! -v "excluded_category_count[$category]" ]]; then
        local slug
        slug=$(ptn_phpt_category_slug "$category")
        excluded_category_count[$category]=0
        excluded_category_manifest[$category]="$excluded_dir/$slug.txt"
        : > "${excluded_category_manifest[$category]}"
        excluded_category_order+=("$category")
    fi
}

current_bucket=manifest
selected_rows=0
runnable_rows=0
excluded_rows=0
classify=${PTN_PHPT_CLASSIFY:-1}
if [[ "$classify" != "0" ]]; then
    export PTN_PHPT_SECTION_CACHE_DIR="$out_dir/section-cache-$timestamp"
    ptn_phpt_build_section_cache "$manifest" "$php_src" "$PTN_PHPT_SECTION_CACHE_DIR"
    ptn_phpt_load_section_cache_index "$PTN_PHPT_SECTION_CACHE_DIR/index.tsv"
fi
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
    bucket_selected_count[$current_bucket]=$((bucket_selected_count[$current_bucket] + 1))
    selected_rows=$((selected_rows + 1))

    if [[ "$classify" == "0" ]]; then
        category="runnable"
        reason="classification disabled by PTN_PHPT_CLASSIFY=0"
    else
        classification=$(ptn_phpt_classify_row "$row" "$test_path" "$php_src" </dev/null)
        category=${classification%%$'\t'*}
        reason=${classification#*$'\t'}
    fi

    printf '%s\t%s\t%s\n' "$row" "$category" "$reason" >> "$classification_tsv"
    if [[ "$category" != "runnable" ]]; then
        ensure_excluded_category "$category"
        excluded_category_count[$category]=$((excluded_category_count[$category] + 1))
        printf '%s\n' "$row" >> "${excluded_category_manifest[$category]}"
        printf '%s\t%s\t%s\n' "$row" "$category" "$reason" >> "$excluded_tsv"
        excluded_rows=$((excluded_rows + 1))
        continue
    fi

    printf '%s\n' "$row" >> "$runnable_manifest"
    printf '%s\n' "$row" >> "${bucket_rel_manifest[$current_bucket]}"
    printf '%s\n' "$test_path" >> "${bucket_abs_manifest[$current_bucket]}"
    bucket_count[$current_bucket]=$((bucket_count[$current_bucket] + 1))
    runnable_rows=$((runnable_rows + 1))
done < "$manifest"

if [[ "$selected_rows" -eq 0 ]]; then
    echo "manifest contains no selected rows after comments/blank lines: $manifest" >&2
    exit 1
fi

for bucket in "${bucket_order[@]}"; do
    expected=${bucket_expected[$bucket]:-}
    if [[ -n "$expected" && "${bucket_selected_count[$bucket]}" -ne "$expected" ]]; then
        echo "bucket '$bucket' declares rows=$expected but contains ${bucket_selected_count[$bucket]} selected rows" >&2
        exit 1
    fi
done

emit_classification_summary() {
    {
        echo "classification: enabled=$classify selected=$selected_rows runnable=$runnable_rows excluded=$excluded_rows"
        echo "classification-files: all=$resolved_manifest runnable=$runnable_manifest classification=$classification_tsv excluded=$excluded_tsv"
        local category
        for category in "${excluded_category_order[@]}"; do
            echo "classification.$category: rows=${excluded_category_count[$category]} manifest=${excluded_category_manifest[$category]}"
        done
    }
}

if [[ "$runnable_rows" -eq 0 ]]; then
    {
        echo "PHPT bounded patrol $timestamp"
        echo "commit: $(git rev-parse --short=12 HEAD)"
        echo "corpus: $php_src"
        echo "manifest: $resolved_manifest"
        echo "count: $selected_rows selected PHPT rows; 0 runnable; $excluded_rows excluded by classification"
    } | tee "$summary"
    emit_classification_summary | tee -a "$summary"
    echo "result: buckets=${#bucket_order[@]} selected=$selected_rows runnable=0 excluded=$excluded_rows tests=0 passed=0 failed=0 skipped=0 warned=0 elapsed=0s" | tee -a "$summary"
    echo "run-tests-exit: 0" | tee -a "$summary"
    exit 0
fi

if [[ "$classify_only" -eq 1 ]]; then
    {
        echo "PHPT bounded patrol $timestamp"
        echo "commit: $(git rev-parse --short=12 HEAD)"
        echo "corpus: $php_src"
        echo "corpus-revision: $corpus_revision"
        echo "manifest: $resolved_manifest"
        echo "runnable-manifest: $runnable_manifest"
        command_line="tools/run-bounded-phpt.sh --classify-only"
        if [[ "$classify_harness_programs" == "1" ]]; then
            command_line+=" --classify-harness-programs"
        fi
        command_line+=" $manifest"
        echo "command: $command_line"
        echo "count: $selected_rows selected PHPT rows; $runnable_rows runnable; $excluded_rows excluded by classification in ${#bucket_order[@]} buckets"
    } | tee "$summary"
    emit_classification_summary | tee -a "$summary"
    echo "result: buckets=${#bucket_order[@]} selected=$selected_rows runnable=$runnable_rows excluded=$excluded_rows tests=0 passed=0 failed=0 skipped=0 warned=0 elapsed=0s" | tee -a "$summary"
    echo "run-tests-exit: 0" | tee -a "$summary"
    exit 0
fi

cargo build --bin phpc

phpc_bin=${PHPC_BIN:-$PWD/target/debug/phpc}
phpt_test_timeout=${PTN_PHPT_TEST_TIMEOUT:-3600}
if [[ ! "$phpt_test_timeout" =~ ^[0-9]+$ || "$phpt_test_timeout" -le 0 ]]; then
    echo "PTN_PHPT_TEST_TIMEOUT must be a positive integer number of seconds: $phpt_test_timeout" >&2
    exit 2
fi
phpt_run_tests_jobs=${PTN_PHPT_RUN_TESTS_JOBS:-4}
if [[ ! "$phpt_run_tests_jobs" =~ ^[0-9]+$ || "$phpt_run_tests_jobs" -le 0 ]]; then
    echo "PTN_PHPT_RUN_TESTS_JOBS must be a positive integer: $phpt_run_tests_jobs" >&2
    exit 2
fi
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
    echo "corpus-revision: $corpus_revision"
    echo "manifest: $resolved_manifest"
    echo "runnable-manifest: $runnable_manifest"
    echo "command: cargo build --bin phpc; cd \"$php_src\" && PHPC_BIN=\"$phpc_bin\" PTN_PHPT_RUN_TESTS_JOBS=\"$phpt_run_tests_jobs\" php $php_src/run-tests.php -q --set-timeout \"$phpt_test_timeout\" -p \"$phpc_bin\" <bucket manifest paths>"
    echo "timeout-seconds: $phpt_test_timeout"
    echo "run-tests-jobs: $phpt_run_tests_jobs"
    echo "count: $selected_rows selected PHPT rows; $runnable_rows runnable; $excluded_rows excluded by classification in ${#bucket_order[@]} buckets"
    emit_classification_summary
} | tee "$summary"

for bucket in "${bucket_order[@]}"; do
    slug=${bucket_slug[$bucket]}
    log="$out_dir/run-$timestamp-$slug.log"
    mapfile -t tests < "${bucket_abs_manifest[$bucket]}"
    bucket_start=$(date +%s)

    if [[ "${bucket_count[$bucket]}" -eq 0 ]]; then
        {
            echo "bucket: $bucket selected=${bucket_selected_count[$bucket]} runnable=0 tests=0 passed=0 failed=0 skipped=0 warned=0 elapsed=0s run-tests-exit=0 log="
        } | tee -a "$summary"
        continue
    fi

    set +e
    run_tests_args=(-q --set-timeout "$phpt_test_timeout" -p "$phpc_bin")
    if [[ "$phpt_run_tests_jobs" -gt 1 ]]; then
        run_tests_args+=("-j$phpt_run_tests_jobs")
    fi
    (
      cd "$php_src"
      PHPC_BIN="$phpc_bin" \
        TEST_PHP_CGI_EXECUTABLE="$phpc_bin" \
        TEST_PHP_CGI_EXECUTABLE_ESCAPED="'$phpc_bin'" \
        php "$php_src/run-tests.php" "${run_tests_args[@]}" "${tests[@]}"
    ) 2>&1 | tee "$log"
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
        echo "bucket: $bucket selected=${bucket_selected_count[$bucket]} runnable=${bucket_count[$bucket]} tests=$total passed=$passed failed=$failed skipped=$skipped warned=$warned elapsed=${bucket_elapsed}s run-tests-exit=$run_status log=$log"
    } | tee -a "$summary"
done

end_epoch=$(date +%s)
elapsed=$((end_epoch - start_epoch))

{
    echo "result: buckets=${#bucket_order[@]} selected=$selected_rows runnable=$runnable_rows excluded=$excluded_rows tests=$aggregate_total passed=$aggregate_passed failed=$aggregate_failed skipped=$aggregate_skipped warned=$aggregate_warned elapsed=${elapsed}s"
    echo "run-tests-exit: $aggregate_run_status"
} | tee -a "$summary"
