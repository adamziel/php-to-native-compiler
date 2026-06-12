#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest="${1:-$repo_root/tools/phpt-manifest-200.txt}"
source "$repo_root/tools/phpt-corpus.sh"
php_src="$(ptn_resolve_phpt_corpus "$repo_root")"
log_dir="$repo_root/.runtime/phpt-progress"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
log="$log_dir/run-$stamp.log"
resolved_manifest="$log_dir/manifest-$stamp.txt"

if [[ "$manifest" == "-" ]]; then
  manifest_input="/dev/stdin"
elif [[ -r "$manifest" && ! -d "$manifest" ]]; then
  manifest_input="$manifest"
else
  echo "manifest not readable: $manifest" >&2
  exit 2
fi

mkdir -p "$log_dir"

paths=()
while IFS= read -r row; do
  [[ -z "$row" || "$row" =~ ^[[:space:]]*# ]] && continue
  if [[ "$row" = /* ]]; then
    path="$row"
  else
    path="$php_src/$row"
  fi
  if [[ ! -f "$path" ]]; then
    echo "PHPT row not found: $path" >&2
    exit 2
  fi
  paths+=("$path")
done < "$manifest_input"

printf '%s\n' "${paths[@]}" > "$resolved_manifest"

cd "$repo_root"
cargo build --bin phpc

phpc_bin="${PHPC_BIN:-$repo_root/target/debug/phpc}"
start="$(date +%s)"

set +e
PHPC_BIN="$phpc_bin" php "$php_src/run-tests.php" -q -p "$phpc_bin" "${paths[@]}" 2>&1 | tee "$log"
run_status="${PIPESTATUS[0]}"
set -e

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
  echo "[ptn-patrol] commit=$(git rev-parse --short HEAD) manifest=$resolved_manifest rows=${#paths[@]} elapsed=${elapsed}s status=$run_status"
  if [[ -n "$summary" ]]; then
    echo "[ptn-patrol] $summary"
  fi
} | tee -a "$log"

if [[ -n "$summary" ]]; then
  exit 0
fi

exit "$run_status"
