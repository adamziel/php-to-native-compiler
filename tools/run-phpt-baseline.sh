#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$repo_root/tools/phpt-corpus.sh"

usage() {
  cat <<'EOF'
Usage: tools/run-phpt-baseline.sh [--scope broad|full] [--tier N|all] [--generate-only] [--classify-only] [--out-dir DIR]

Generate deterministic PHPT baseline manifests from the canonical php-src
corpus. The default broad scope preserves the legacy Zend/ext-standard/core
1k/5k/10k family. The full scope inventories every local .phpt row and writes
the 1k/5k/10k/20k/all full-corpus family.

By default the selected tier is run through run-tests.php. Use --classify-only
to generate blocker maps without building phpc or running selected runnable
rows.

Options:
  --scope broad             legacy broad buckets: Zend/tests, ext/standard/tests, tests
  --scope full              all php-src .phpt rows, bucketed by top-level family
  --full-corpus             alias for --scope full
  --broad                   alias for --scope broad

Defaults:
  --tier 1000
  --out-dir .runtime/phpt-baseline

Environment:
  PHP_SRC_PHPT          php-src checkout with run-tests.php
  PHPT_BASELINE_DIR    output directory for generated tier manifests
  PHPT_BASELINE_SCOPE  broad or full when --scope is omitted
  PHPT_BASELINE_TIER   tier to run when --tier is omitted
EOF
}

out_dir=${PHPT_BASELINE_DIR:-$repo_root/.runtime/phpt-baseline}
scope=${PHPT_BASELINE_SCOPE:-broad}
run_tier=${PHPT_BASELINE_TIER:-1000}
generate_only=0
classify_only=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --generate-only)
      generate_only=1
      shift
      ;;
    --classify-only)
      classify_only=1
      shift
      ;;
    --tier)
      [[ $# -ge 2 ]] || { echo "--tier requires a value" >&2; exit 2; }
      run_tier=$2
      shift 2
      ;;
    --out-dir)
      [[ $# -ge 2 ]] || { echo "--out-dir requires a value" >&2; exit 2; }
      out_dir=$2
      shift 2
      ;;
    --scope)
      [[ $# -ge 2 ]] || { echo "--scope requires a value" >&2; exit 2; }
      scope=$2
      shift 2
      ;;
    --full-corpus)
      scope=full
      shift
      ;;
    --broad)
      scope=broad
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      if [[ "$1" == --* ]]; then
        echo "unknown option: $1" >&2
        usage >&2
        exit 2
      fi
      run_tier=$1
      shift
      ;;
    esac
done

case "$scope" in
  broad|full) ;;
  *)
    echo "--scope must be 'broad' or 'full': $scope" >&2
    exit 2
    ;;
esac

if [[ "$generate_only" -eq 1 && "$classify_only" -eq 1 ]]; then
  echo "--generate-only and --classify-only are mutually exclusive" >&2
  exit 2
fi

if [[ "$run_tier" != "all" && ! "$run_tier" =~ ^[0-9]+$ ]]; then
  echo "--tier must be a positive integer or 'all': $run_tier" >&2
  exit 2
fi
if [[ "$run_tier" != "all" && "$run_tier" -le 0 ]]; then
  echo "--tier must be positive: $run_tier" >&2
  exit 2
fi

php_src="$(ptn_resolve_phpt_corpus "$repo_root")"
corpus_revision="$(ptn_phpt_corpus_revision "$php_src")"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
manifest_dir="$out_dir/$stamp"
inventory="$manifest_dir/inventory.txt"

mkdir -p "$manifest_dir"

ptn_baseline_slug() {
  printf '%s' "$1" \
    | tr '[:upper:]' '[:lower:]' \
    | sed 's/[^a-z0-9._-]/-/g; s/--*/-/g; s/^-//; s/-$//'
}

ptn_full_bucket_for_row() {
  local row=$1
  local first second rest

  IFS=/ read -r first second rest <<< "$row"
  case "$first" in
    ext|sapi)
      if [[ -n "${second:-}" ]]; then
        printf '%s/%s\n' "$first" "$second"
      else
        printf '%s\n' "$first"
      fi
      ;;
    *)
      printf '%s\n' "$first"
      ;;
  esac
}

bucket_names=()
bucket_files=()
bucket_counts=()

available_rows=0

if [[ "$scope" == "broad" ]]; then
  bucket_names=(zend standard core)
  bucket_roots=(Zend/tests ext/standard/tests tests)
  for i in "${!bucket_names[@]}"; do
    root=${bucket_roots[$i]}
    if [[ ! -d "$php_src/$root" ]]; then
      echo "PHPT corpus bucket root missing: $php_src/$root" >&2
      exit 2
    fi

    rows_file="$manifest_dir/all-${bucket_names[$i]}.txt"
    (cd "$php_src" && find "$root" -type f -name '*.phpt' | LC_ALL=C sort) > "$rows_file"
    count=$(wc -l < "$rows_file")
    count=${count//[[:space:]]/}
    bucket_files[$i]=$rows_file
    bucket_counts[$i]=$count
    available_rows=$((available_rows + count))
  done
else
  all_rows_file="$manifest_dir/full-corpus-inventory.txt"
  bucket_index="$manifest_dir/full-corpus-buckets.tsv"

  (cd "$php_src" && find . -path './.git' -prune -o -type f -name '*.phpt' -print \
    | sed 's#^\./##' \
    | LC_ALL=C sort) > "$all_rows_file"

  : > "$bucket_index"
  while IFS= read -r row || [[ -n "$row" ]]; do
    [[ -n "$row" ]] || continue
    printf '%s\t%s\n' "$(ptn_full_bucket_for_row "$row")" "$row" >> "$bucket_index"
  done < "$all_rows_file"

  mapfile -t bucket_names < <(cut -f1 "$bucket_index" | LC_ALL=C sort -u)
  for i in "${!bucket_names[@]}"; do
    bucket=${bucket_names[$i]}
    rows_file="$manifest_dir/all-$(ptn_baseline_slug "$bucket").txt"
    awk -F '\t' -v bucket="$bucket" '$1 == bucket { print $2 }' "$bucket_index" > "$rows_file"
    count=$(wc -l < "$rows_file")
    count=${count//[[:space:]]/}
    bucket_files[$i]=$rows_file
    bucket_counts[$i]=$count
    available_rows=$((available_rows + count))
  done
fi

if [[ "$available_rows" -eq 0 ]]; then
  echo "no PHPT rows found in $scope baseline buckets" >&2
  exit 1
fi

if [[ "$scope" == "full" ]]; then
  default_tiers=(1000 5000 10000 20000)
  manifest_prefix=phpt-full-corpus
  raw_generation_tiers=("${default_tiers[@]}" all)
else
  default_tiers=(1000 5000 10000)
  manifest_prefix=phpt-baseline
  raw_generation_tiers=("${default_tiers[@]}")
fi
if [[ "$run_tier" != "all" ]]; then
  raw_generation_tiers+=("$run_tier")
fi

mapfile -t numeric_generation_tiers < <(
  printf '%s\n' "${raw_generation_tiers[@]}" \
    | awk '$0 != "all" && !seen[$0]++' \
    | sort -n
)
include_generation_all=0
if printf '%s\n' "${raw_generation_tiers[@]}" | grep -qx 'all'; then
  include_generation_all=1
fi
generation_tiers=("${numeric_generation_tiers[@]}")
if [[ "$include_generation_all" -eq 1 ]]; then
  generation_tiers+=(all)
fi

declare -A manifest_by_tier=()

write_manifest() {
  local requested=$1
  local selected=$requested
  if [[ "$requested" == "all" ]]; then
    selected=$available_rows
  elif [[ "$selected" -gt "$available_rows" ]]; then
    selected=$available_rows
  fi

  local manifest="$manifest_dir/$manifest_prefix-${requested}.txt"
  local -a allocated=()
  local -a remainder=()
  local assigned=0

  for i in "${!bucket_names[@]}"; do
    allocated[$i]=$((selected * bucket_counts[$i] / available_rows))
    remainder[$i]=$((selected * bucket_counts[$i] % available_rows))
    assigned=$((assigned + allocated[$i]))
  done

  while [[ "$assigned" -lt "$selected" ]]; do
    local best=-1
    local best_remainder=-1
    for i in "${!bucket_names[@]}"; do
      if [[ "${allocated[$i]}" -lt "${bucket_counts[$i]}" && "${remainder[$i]}" -gt "$best_remainder" ]]; then
        best=$i
        best_remainder=${remainder[$i]}
      fi
    done
    if [[ "$best" -lt 0 ]]; then
      echo "could not allocate PHPT baseline rows for tier $requested" >&2
      exit 1
    fi
    allocated[$best]=$((allocated[$best] + 1))
    assigned=$((assigned + 1))
  done

  {
    echo "# Generated by tools/run-phpt-baseline.sh"
    echo "# generated-at: $stamp"
    echo "# corpus: $php_src"
    echo "# corpus-revision: $corpus_revision"
    echo "# scope: $scope"
    echo "# requested-rows: $requested"
    echo "# selected-rows: $selected"
    if [[ "$scope" == "broad" ]]; then
      echo "# source-buckets: Zend/tests, ext/standard/tests, tests"
    else
      echo "# source-buckets: full php-src corpus"
    fi

    for i in "${!bucket_names[@]}"; do
      if [[ "${allocated[$i]}" -eq 0 ]]; then
        continue
      fi
      echo
      echo "# bucket: ${bucket_names[$i]} rows=${allocated[$i]}"
      head -n "${allocated[$i]}" "${bucket_files[$i]}"
    done
  } > "$manifest"

  manifest_by_tier[$requested]=$manifest
  if [[ "$scope" == "broad" ]]; then
    printf 'manifest: scope=%s tier=%s rows=%s zend=%s standard=%s core=%s path=%s\n' \
      "$scope" "$requested" "$selected" "${allocated[0]}" "${allocated[1]}" "${allocated[2]}" "$manifest" \
      | tee -a "$inventory"
  else
    printf 'manifest: scope=%s tier=%s rows=%s buckets=%s path=%s\n' \
      "$scope" "$requested" "$selected" "${#bucket_names[@]}" "$manifest" \
      | tee -a "$inventory"
    for i in "${!bucket_names[@]}"; do
      if [[ "${allocated[$i]}" -gt 0 ]]; then
        printf 'manifest.%s.%s: rows=%s\n' \
          "$requested" "${bucket_names[$i]}" "${allocated[$i]}" >> "$inventory"
      fi
    done
  fi
}

{
  echo "PHPT $scope baseline $stamp"
  echo "corpus: $php_src"
  echo "corpus-revision: $corpus_revision"
  echo "scope: $scope"
  if [[ "$scope" == "broad" ]]; then
    printf 'available: rows=%s zend=%s standard=%s core=%s\n' \
      "$available_rows" "${bucket_counts[0]}" "${bucket_counts[1]}" "${bucket_counts[2]}"
  else
    printf 'available: rows=%s buckets=%s inventory=%s bucket-index=%s\n' \
      "$available_rows" "${#bucket_names[@]}" "$all_rows_file" "$bucket_index"
    for i in "${!bucket_names[@]}"; do
      printf 'available.%s: rows=%s\n' "${bucket_names[$i]}" "${bucket_counts[$i]}"
    done
  fi
} | tee "$inventory"

for tier in "${generation_tiers[@]}"; do
  write_manifest "$tier"
done

if [[ "$generate_only" -eq 1 ]]; then
  echo "generated-only: $manifest_dir" | tee -a "$inventory"
  exit 0
fi

run_tiers=()
if [[ "$run_tier" == "all" ]]; then
  run_tiers=("${default_tiers[@]}")
  if [[ "$scope" == "full" ]]; then
    run_tiers+=(all)
  fi
else
  run_tiers=("$run_tier")
fi

for tier in "${run_tiers[@]}"; do
  manifest=${manifest_by_tier[$tier]:-}
  if [[ -z "$manifest" ]]; then
    echo "internal error: no generated manifest for tier $tier" >&2
    exit 1
  fi
  echo "running: tier=$tier manifest=$manifest"
  bounded_args=()
  if [[ "$classify_only" -eq 1 ]]; then
    bounded_args+=(--classify-only)
  fi
  "$repo_root/tools/run-bounded-phpt.sh" --classify-harness-programs "${bounded_args[@]}" "$manifest"
done
