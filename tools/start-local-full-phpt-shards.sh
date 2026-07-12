#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

run_dir="${PTN_PHPT_RESUME_RUN_DIR:-$repo_root/.runtime/local-full-phpt-current}"
run_dir="$(cd "$run_dir" && pwd -P)"
shard_count="${PTN_PHPT_SHARDS:-4}"
jobs_per_shard="${PTN_PHPT_JOBS_PER_SHARD:-4}"
timeout_seconds="${PTN_PHPT_TEST_TIMEOUT:-60}"
stream_batch_size="${PTN_PHPT_STREAM_BATCH_SIZE:-100}"
stall_timeout="${PTN_PHPT_STALL_TIMEOUT:-45}"
single_wall_timeout="${PTN_PHPT_SINGLE_WALL_TIMEOUT:-20}"
quarantine_extension_on_crash="${PTN_PHPT_QUARANTINE_EXTENSION_ON_CRASH:-1}"
php_src="${PHP_SRC_PHPT:-/home/claude/php-src-phpt}"
phpc_bin="${PHPC_BIN:-$repo_root/target/release/phpc}"

if ! [[ "$shard_count" =~ ^[1-9][0-9]*$ ]]; then
  echo "PTN_PHPT_SHARDS must be positive" >&2
  exit 1
fi
if ! [[ "$jobs_per_shard" =~ ^[1-9][0-9]*$ ]]; then
  echo "PTN_PHPT_JOBS_PER_SHARD must be positive" >&2
  exit 1
fi

tools/prepare-local-phpt-shards.py --run-dir "$run_dir" --shards "$shard_count" >/dev/null

plan="$run_dir/shards/plan.tsv"
launch_log="$run_dir/shards/launch.log"
while IFS=$'\t' read -r shard remaining manifest_rel; do
  [[ "$shard" == "shard" ]] && continue
  (( remaining > 0 )) || continue

  shard_id="$(printf '%02d' "$shard")"
  shard_dir="$run_dir/shards/shard-$shard_id"
  manifest="$run_dir/$manifest_rel"
  primary_session="ptn-full-phpt-shard-$shard_id"
  watchdog_session="ptn-full-phpt-shard-$shard_id-watchdog"
  watchdog_socket="ptn-phpt-shard-$shard_id-watchdog"

  if ! env -u TMUX tmux has-session -t "$primary_session" 2>/dev/null; then
    env -u TMUX \
      PTN_PHPT_SESSION="$primary_session" \
      PTN_PHPT_RESUME_RUN_DIR="$shard_dir" \
      PTN_PHPT_UPDATE_CURRENT_LINK=0 \
      PTN_PHPT_MANIFEST="$manifest" \
      PHP_SRC_PHPT="$php_src" \
      PHPC_BIN="$phpc_bin" \
      PTN_PHPT_TEST_TIMEOUT="$timeout_seconds" \
      PTN_PHPT_JOBS="$jobs_per_shard" \
      PTN_PHPT_STREAM_BATCH_SIZE="$stream_batch_size" \
      PTN_PHPT_STALL_TIMEOUT="$stall_timeout" \
      PTN_PHPT_SINGLE_WALL_TIMEOUT="$single_wall_timeout" \
      PTN_PHPT_QUARANTINE_EXTENSION_ON_CRASH="$quarantine_extension_on_crash" \
      tools/start-local-full-phpt-run.sh >> "$launch_log" 2>&1
  fi

  PTN_PHPT_PRIMARY_SESSION="$primary_session" \
    PTN_PHPT_WATCHDOG_SESSION="$watchdog_session" \
    PTN_PHPT_WATCHDOG_SOCKET="$watchdog_socket" \
    PTN_PHPT_RESUME_RUN_DIR="$shard_dir" \
    PTN_PHPT_MANIFEST="$manifest" \
    PHP_SRC_PHPT="$php_src" \
    PHPC_BIN="$phpc_bin" \
    PTN_PHPT_TEST_TIMEOUT="$timeout_seconds" \
    PTN_PHPT_JOBS="$jobs_per_shard" \
    PTN_PHPT_STREAM_BATCH_SIZE="$stream_batch_size" \
    PTN_PHPT_STALL_TIMEOUT="$stall_timeout" \
    PTN_PHPT_SINGLE_WALL_TIMEOUT="$single_wall_timeout" \
    PTN_PHPT_QUARANTINE_EXTENSION_ON_CRASH="$quarantine_extension_on_crash" \
    tools/start-local-full-phpt-watchdog.sh >> "$launch_log" 2>&1

  printf 'shard=%s tests=%s primary=%s watchdog=%s\n' \
    "$shard_id" "$remaining" "$primary_session" "$watchdog_session"
done < "$plan"

cat <<EOF
run_dir=$run_dir
workers=$((shard_count * jobs_per_shard))
watch=tools/phpt-local-sharded-progress.sh $run_dir 5
EOF
