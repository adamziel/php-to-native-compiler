#!/usr/bin/env bash
set -euo pipefail

run_dir="${1:-.runtime/local-full-phpt-current}"
interval="${2:-5}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ -f "$run_dir/shards/plan.tsv" ]]; then
  exec "$script_dir/phpt-local-sharded-progress.sh" "$run_dir" "$interval"
fi

read_status() {
  local status="$run_dir/status.tsv"
  if [[ ! -f "$status" ]]; then
    printf 'Waiting for status file: %s\n' "$status"
    return
  fi
  awk -F '\t' '
    { value[$1]=$2 }
    END {
      total=value["total"] + 0
      reported=value["reported"] + 0
      remaining=value["remaining"] + 0
      passed=value["passed"] + 0
      failed=value["failed"] + 0
      skipped=value["skipped"] + 0
      warned=value["warned"] + 0
      crashed=value["crashed"] + 0
      pct = total ? (reported * 100.0 / total) : 0
      printf "PHPT local full run\n"
      printf "state: %s  mode: %s  updated: %s\n", value["state"], value["mode"], value["updated_at_utc"]
      printf "run_dir: %s\n", value["run_dir"]
      printf "reported: %d / %d (%.2f%%), remaining: %d\n", reported, total, pct, remaining
      printf "passed: %d  failed: %d  skipped: %d  warned: %d  crashed: %d\n", passed, failed, skipped, warned, crashed
      printf "current: %s\n", value["current"]
      printf "last: %s [%s]\n", value["last_row"], value["last_state"]
    }
  ' "$status"
}

while true; do
  clear 2>/dev/null || true
  read_status
  sleep "$interval"
done
