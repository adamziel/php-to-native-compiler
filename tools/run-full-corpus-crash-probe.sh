#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
run_root="${PTN_FULL_CRASH_PROBE_ROOT:-/home/claude/.local/state/ptn-full-corpus-crash-probe}"
run_dir="$run_root/runs/$stamp"
worktree="$run_dir/worktree"
status="$run_dir/status.tsv"
summary="$run_dir/summary.tsv"
launcher_log="$run_dir/launcher.log"
shard_count="${PTN_FULL_CRASH_PROBE_SHARDS:-24}"
jobs_per_shard="${PTN_FULL_CRASH_PROBE_JOBS_PER_SHARD:-1}"
test_timeout="${PTN_FULL_CRASH_PROBE_TEST_TIMEOUT:-900}"

mkdir -p "$run_dir"

write_status() {
  local state=$1
  {
    printf 'state\t%s\n' "$state"
    printf 'stamp\t%s\n' "$stamp"
    printf 'run_dir\t%s\n' "$run_dir"
    printf 'worktree\t%s\n' "$worktree"
    printf 'shards\t%s\n' "$shard_count"
    printf 'jobs_per_shard\t%s\n' "$jobs_per_shard"
    printf 'test_timeout\t%s\n' "$test_timeout"
    printf 'updated_at_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    if [[ -d "$worktree/.git" || -f "$worktree/.git" ]]; then
      printf 'source_commit\t%s\n' "$(git -C "$worktree" rev-parse --short=12 HEAD 2>/dev/null || true)"
    fi
  } > "$status"
}

summarize() {
  local selected=0 tests=0 passed=0 failed=0 skipped=0 warned=0 complete=0 running=0 crashed=0
  local shard_status key value last_state
  for shard_status in "$run_dir"/shards/shard-*/status.tsv; do
    [[ -f "$shard_status" ]] || continue
    last_state=
    while IFS=$'\t' read -r key value; do
      case "$key" in
        selected) selected=$((selected + ${value:-0})) ;;
        tests) tests=$((tests + ${value:-0})) ;;
        passed) passed=$((passed + ${value:-0})) ;;
        failed) failed=$((failed + ${value:-0})) ;;
        skipped) skipped=$((skipped + ${value:-0})) ;;
        warned) warned=$((warned + ${value:-0})) ;;
        state) last_state=$value ;;
      esac
    done < "$shard_status"
    case "$last_state" in
      passed|failed) complete=$((complete + 1)) ;;
      running) running=$((running + 1)) ;;
      crashed) crashed=$((crashed + 1)) ;;
    esac
  done
  {
    printf 'updated_at_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf 'shards\t%s\n' "$shard_count"
    printf 'completed_shards\t%s\n' "$complete"
    printf 'running_shards\t%s\n' "$running"
    printf 'crashed_shards\t%s\n' "$crashed"
    printf 'selected\t%s\n' "$selected"
    printf 'tests\t%s\n' "$tests"
    printf 'passed\t%s\n' "$passed"
    printf 'failed\t%s\n' "$failed"
    printf 'skipped\t%s\n' "$skipped"
    printf 'warned\t%s\n' "$warned"
  } > "$summary"
}

run_shard() {
  local index=$1
  local shard_dir="$run_dir/shards/shard-$(printf '%02d' "$index")"
  local manifest="$shard_dir/manifest.txt"
  local log="$shard_dir/run.log"
  local shard_status="$shard_dir/status.tsv"
  mkdir -p "$shard_dir"
  {
    printf 'state\trunning\n'
    printf 'started_at_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf 'shard\t%s\n' "$index"
    printf 'log\t%s\n' "$log"
  } > "$shard_status"

  set +e
  (
    cd "$worktree" || exit 97
    tools/phpt-full-corpus-shard.sh --shard-index "$index" --shard-count "$shard_count" --out "$manifest"
    selected="$(grep -Ev '^[[:space:]]*(#|$)' "$manifest" | wc -l | tr -d '[:space:]')"
    printf 'selected\t%s\n' "$selected" >> "$shard_status"
    PHPT_PROGRESS_DIR="$shard_dir/progress" \
      PTN_PHPT_CLASSIFY=0 \
      PTN_PHPT_JOBS="$jobs_per_shard" \
      PTN_PHPT_TEST_TIMEOUT="$test_timeout" \
      PHPC_BIN="$worktree/target/debug/phpc" \
      tools/run-phpt-manifest.sh "$manifest"
  ) > "$log" 2>&1
  local code=$?
  set -e

  local metrics
  metrics="$(awk '
    /\[ptn-patrol\] tests=/ {
      for (i = 1; i <= NF; i++) {
        split($i, kv, "=")
        if (kv[1] == "tests" || kv[1] == "passed" || kv[1] == "failed" || kv[1] == "skipped" || kv[1] == "warned") {
          values[kv[1]] = kv[2]
        }
      }
    }
    END {
      printf "tests\t%s\npassed\t%s\nfailed\t%s\nskipped\t%s\nwarned\t%s\n", values["tests"] + 0, values["passed"] + 0, values["failed"] + 0, values["skipped"] + 0, values["warned"] + 0
    }
  ' "$log")"
  {
    printf 'exit\t%s\n' "$code"
    printf 'state\t%s\n' "$([[ "$code" -eq 0 ]] && echo passed || echo failed)"
    printf 'finished_at_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf '%s\n' "$metrics"
  } >> "$shard_status"
}

write_status preparing
{
  printf '===== full corpus crash probe %s =====\n' "$stamp"
  printf 'repo=%s\nrun_dir=%s\n' "$repo_root" "$run_dir"
} > "$launcher_log"

git -C "$repo_root" worktree add --detach "$worktree" HEAD >> "$launcher_log" 2>&1
write_status building
(
  cd "$worktree"
  mkdir -p .runtime
  cargo build --bin phpc
) >> "$launcher_log" 2>&1

write_status running
mkdir -p "$run_dir/shards"
for ((i = 0; i < shard_count; i++)); do
  run_shard "$i" &
done

while true; do
  summarize
  live=0
  for job in $(jobs -p); do
    if kill -0 "$job" 2>/dev/null; then
      live=1
      break
    fi
  done
  [[ "$live" -eq 1 ]] || break
  sleep 60
done

set +e
wait
wait_code=$?
set -e
summarize
write_status "$([[ "$wait_code" -eq 0 ]] && echo finished || echo failed)"
exit "$wait_code"
