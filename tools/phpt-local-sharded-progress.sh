#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
run_dir="${1:-$repo_root/.runtime/local-full-phpt-current}"
interval="${2:-5}"
run_dir="$(cd "$run_dir" && pwd -P)"
baseline="$run_dir/shards/baseline.tsv"
plan="$run_dir/shards/plan.tsv"
rate_history="$run_dir/shards/progress-rate.tsv"

if [[ ! -f "$baseline" || ! -f "$plan" ]]; then
  echo "sharded PHPT state is not initialized under: $run_dir" >&2
  exit 1
fi

value() {
  local key="$1"
  local file="$2"
  awk -F '\t' -v key="$key" '$1 == key { print $2; exit }' "$file"
}

record_rate_sample() {
  local timestamp="$1"
  local completed="$2"
  printf '%s\t%s\n' "$timestamp" "$completed" >> "$rate_history"
}

completion_rate() {
  local timestamp="$1"
  local completed="$2"
  local window="$3"
  local target=$((timestamp - window))
  local sample
  sample="$(awk -F '\t' -v target="$target" '
    $1 <= target && $1 >= selected { selected = $1; completed = $2 }
    END { if (selected != "") print selected "\t" completed }
  ' "$rate_history" 2>/dev/null)"
  if [[ -z "$sample" ]]; then
    printf 'warming up'
    return
  fi
  local previous_timestamp previous_completed
  IFS=$'\t' read -r previous_timestamp previous_completed <<< "$sample"
  awk -v now="$timestamp" -v completed="$completed" \
    -v previous_timestamp="$previous_timestamp" -v previous_completed="$previous_completed" '
      BEGIN {
        elapsed = now - previous_timestamp
        rate = elapsed ? (completed - previous_completed) * 60 / elapsed : 0
        printf "%.1f tests/min", rate
      }
    '
}

while true; do
  total="$(value total "$baseline")"
  reported="$(value reported "$baseline")"
  passed="$(value passed "$baseline")"
  failed="$(value failed "$baseline")"
  skipped="$(value skipped "$baseline")"
  warned="$(value warned "$baseline")"
  crashed="$(value crashed "$baseline")"

  clear 2>/dev/null || true
  printf 'PHPT local full run (sharded)\n'
  printf 'run_dir: %s\n' "$run_dir"
  printf '\n'

  while IFS=$'\t' read -r shard shard_total manifest; do
    [[ "$shard" == "shard" ]] && continue
    shard_id="$(printf '%02d' "$shard")"
    status="$run_dir/shards/shard-$shard_id/status.tsv"
    shard_state="waiting"
    shard_reported=0
    shard_passed=0
    shard_failed=0
    shard_skipped=0
    shard_warned=0
    shard_crashed=0
    shard_current=""
    if [[ -f "$status" ]]; then
      shard_state="$(value state "$status")"
      shard_reported="$(value reported "$status")"
      shard_passed="$(value passed "$status")"
      shard_failed="$(value failed "$status")"
      shard_skipped="$(value skipped "$status")"
      shard_warned="$(value warned "$status")"
      shard_crashed="$(value crashed "$status")"
      shard_current="$(value current "$status")"
    fi
    reported=$((reported + shard_reported))
    passed=$((passed + shard_passed))
    failed=$((failed + shard_failed))
    skipped=$((skipped + shard_skipped))
    warned=$((warned + shard_warned))
    crashed=$((crashed + shard_crashed))
    printf 'shard %s: %-8s %5s / %-5s  %s\n' \
      "$shard_id" "$shard_state" "$shard_reported" "$shard_total" "$shard_current"
  done < "$plan"

  percent="$(awk -v done="$reported" -v all="$total" 'BEGIN { printf "%.2f", 100 * done / all }')"
  timestamp="$(date -u +%s)"
  record_rate_sample "$timestamp" "$reported"
  one_minute_rate="$(completion_rate "$timestamp" "$reported" 60)"
  ten_minute_rate="$(completion_rate "$timestamp" "$reported" 600)"
  printf '\nreported: %s / %s (%s%%), remaining: %s\n' \
    "$reported" "$total" "$percent" "$((total - reported))"
  printf 'passed: %s  failed: %s  skipped: %s  warned: %s  crashed: %s\n' \
    "$passed" "$failed" "$skipped" "$warned" "$crashed"
  printf 'completion rate: 1m %s  10m %s\n' "$one_minute_rate" "$ten_minute_rate"
  sleep "$interval"
done
