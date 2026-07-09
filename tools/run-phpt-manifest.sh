#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest="${1:-$repo_root/tools/phpt-manifest-200.txt}"
source "$repo_root/tools/phpt-corpus.sh"
source "$repo_root/tools/phpt-classifier.sh"
php_src="$(ptn_resolve_phpt_corpus "$repo_root")"
corpus_revision="$(ptn_phpt_corpus_revision "$php_src")"
log_dir="${PHPT_PROGRESS_DIR:-$repo_root/.runtime/phpt-progress}"
stamp="$(date -u +%Y%m%dT%H%M%SZ)-$$"
log="$log_dir/run-$stamp.log"
resolved_manifest="$log_dir/manifest-$stamp.txt"
runnable_manifest="$log_dir/runnable-$stamp.txt"
classification_tsv="$log_dir/classification-$stamp.tsv"
excluded_tsv="$log_dir/excluded-$stamp.tsv"
excluded_dir="$log_dir/excluded-$stamp"

if [[ "$manifest" == "-" ]]; then
  manifest_input="/dev/stdin"
elif [[ -r "$manifest" && ! -d "$manifest" ]]; then
  manifest_input="$manifest"
else
  echo "manifest not readable: $manifest" >&2
  exit 2
fi

mkdir -p "$log_dir"
mkdir -p "$excluded_dir"

paths=()
total_rows=0
excluded_rows=0
classify=${PTN_PHPT_CLASSIFY:-1}
declare -a excluded_category_order=()
declare -A excluded_category_count=()
declare -A excluded_category_manifest=()

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

emit_classification_summary() {
  {
    echo "[ptn-phpt-classify] selected=$total_rows runnable=${#paths[@]} excluded=$excluded_rows enabled=$classify"
    echo "[ptn-phpt-classify] all=$resolved_manifest runnable_manifest=$runnable_manifest classification=$classification_tsv excluded=$excluded_tsv"
    local category
    for category in "${excluded_category_order[@]}"; do
      echo "[ptn-phpt-classify] excluded.$category=${excluded_category_count[$category]} manifest=${excluded_category_manifest[$category]}"
    done
  }
}

: > "$resolved_manifest"
: > "$runnable_manifest"
: > "$classification_tsv"
: > "$excluded_tsv"

while IFS= read -r row; do
  row=${row%%#*}
  row=$(ptn_phpt_trim "$row")
  [[ -z "$row" ]] && continue
  if [[ "$row" = /* ]]; then
    path="$row"
  else
    path="$php_src/$row"
  fi
  if [[ ! -f "$path" ]]; then
    echo "PHPT row not found: $path" >&2
    exit 2
  fi
  total_rows=$((total_rows + 1))
  printf '%s\n' "$path" >> "$resolved_manifest"

  if [[ "$classify" == "0" ]]; then
    category="runnable"
    reason="classification disabled by PTN_PHPT_CLASSIFY=0"
  else
    classification="$(ptn_phpt_classify_row "$row" "$path" "$php_src")"
    category=${classification%%$'\t'*}
    reason=${classification#*$'\t'}
  fi

  printf '%s\t%s\t%s\n' "$row" "$category" "$reason" >> "$classification_tsv"
  if [[ "$category" == "runnable" ]]; then
    paths+=("$path")
    printf '%s\n' "$path" >> "$runnable_manifest"
  else
    excluded_rows=$((excluded_rows + 1))
    ensure_excluded_category "$category"
    excluded_category_count[$category]=$((excluded_category_count[$category] + 1))
    printf '%s\n' "$row" >> "${excluded_category_manifest[$category]}"
    printf '%s\t%s\t%s\n' "$row" "$category" "$reason" >> "$excluded_tsv"
  fi
done < "$manifest_input"

if [[ "$total_rows" -eq 0 ]]; then
  echo "manifest contains no selected rows after comments/blank lines: $manifest" >&2
  exit 2
fi

if [[ "${#paths[@]}" -eq 0 ]]; then
  emit_classification_summary | tee "$log"
  echo "[ptn-patrol] no runnable PHPT rows after classification; nothing passed or failed" | tee -a "$log"
  exit 0
fi

phpc_bin="${PHPC_BIN:-$repo_root/target/debug/phpc}"
phpt_test_timeout="${PTN_PHPT_TEST_TIMEOUT:-120}"
if [[ ! "$phpt_test_timeout" =~ ^[0-9]+$ || "$phpt_test_timeout" -le 0 ]]; then
  echo "PTN_PHPT_TEST_TIMEOUT must be a positive integer number of seconds: $phpt_test_timeout" >&2
  exit 2
fi
phpt_jobs="${PTN_PHPT_JOBS:-}"
run_tests_jobs=()
if [[ -n "$phpt_jobs" ]]; then
  if [[ ! "$phpt_jobs" =~ ^[0-9]+$ || "$phpt_jobs" -le 0 ]]; then
    echo "PTN_PHPT_JOBS must be a positive integer number of worker jobs: $phpt_jobs" >&2
    exit 2
  fi
  run_tests_jobs=("-j$phpt_jobs")
fi

cd "$repo_root"
if [[ -z "${PHPC_BIN:-}" ]]; then
  cargo build --bin phpc
fi

start="$(date +%s)"

set +e
(
  cd "$php_src"
  PHPC_BIN="$phpc_bin" php "$php_src/run-tests.php" -q "${run_tests_jobs[@]}" --set-timeout "$phpt_test_timeout" -p "$phpc_bin" "${paths[@]}"
) 2>&1 | tee "$log"
run_status="${PIPESTATUS[0]}"
set -e

emit_classification_summary | tee -a "$log"

elapsed="$(( $(date +%s) - start ))"
summary="$(awk '
  /Number of tests/ { tests=$5 }
  /Tests skipped/ { skipped=$4 }
  /Tests warned/ { warned=$4 }
  /Tests failed/ { failed=$4 }
  /Tests passed/ { passed=$4 }
  /Time taken/ { time=$4 }
  END {
    if (tests != "") {
      printf "tests=%s passed=%s failed=%s skipped=%s warned=%s run_tests_time=%ss", tests, passed, failed, skipped, warned, time
    }
  }
' "$log")"

{
  echo
  echo "[ptn-patrol] commit=$(git rev-parse --short HEAD) corpus_revision=$corpus_revision manifest=$resolved_manifest runnable_manifest=$runnable_manifest selected=$total_rows runnable=${#paths[@]} excluded=$excluded_rows timeout_seconds=$phpt_test_timeout jobs=${phpt_jobs:-1} elapsed=${elapsed}s status=$run_status"
  if [[ -n "$summary" ]]; then
    echo "[ptn-patrol] $summary"
  fi
} | tee -a "$log"

if [[ -n "$summary" ]]; then
  exit 0
fi

exit "$run_status"
