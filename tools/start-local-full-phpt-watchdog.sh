#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

run_dir="${PTN_PHPT_RESUME_RUN_DIR:-$repo_root/.runtime/local-full-phpt-current}"
if [[ ! -d "$run_dir" ]]; then
  echo "PHPT run directory does not exist: $run_dir" >&2
  exit 1
fi

primary_session="${PTN_PHPT_PRIMARY_SESSION:-ptn-full-phpt-local-primary}"
watchdog_session="${PTN_PHPT_WATCHDOG_SESSION:-ptn-full-phpt-watchdog}"
watchdog_socket="${PTN_PHPT_WATCHDOG_SOCKET:-ptn-phpt-watchdog}"
interval="${PTN_PHPT_WATCHDOG_INTERVAL:-5}"
manifest="${PTN_PHPT_MANIFEST:-}"
php_src="${PHP_SRC_PHPT:-/home/claude/php-src-phpt}"
phpc_bin="${PHPC_BIN:-$repo_root/target/release/phpc}"
timeout_seconds="${PTN_PHPT_TEST_TIMEOUT:-60}"
jobs="${PTN_PHPT_JOBS:-8}"
stream_batch_size="${PTN_PHPT_STREAM_BATCH_SIZE:-500}"
stall_timeout="${PTN_PHPT_STALL_TIMEOUT:-600}"
single_wall_timeout="${PTN_PHPT_SINGLE_WALL_TIMEOUT:-300}"

if ! [[ "$interval" =~ ^[1-9][0-9]*$ ]]; then
  echo "PTN_PHPT_WATCHDOG_INTERVAL must be a positive number of seconds" >&2
  exit 1
fi
if [[ -n "$manifest" && ! -f "$manifest" ]]; then
  echo "PHPT manifest does not exist: $manifest" >&2
  exit 1
fi

runner="$run_dir/tmux-watchdog.sh"
cat > "$runner" <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd "$repo_root"
manifest="$manifest"
manifest_args=()
if [[ -n "\$manifest" ]]; then
  manifest_args=(--manifest "\$manifest")
fi
primary_seen=0

start_primary() {
  env -u TMUX \\
    PTN_PHPT_SESSION="$primary_session" \\
    PTN_PHPT_RESUME_RUN_DIR="$run_dir" \\
    PTN_PHPT_UPDATE_CURRENT_LINK=0 \\
    PTN_PHPT_MANIFEST="\$manifest" \\
    PHP_SRC_PHPT="$php_src" \\
    PHPC_BIN="$phpc_bin" \\
    PTN_PHPT_TEST_TIMEOUT="$timeout_seconds" \\
    PTN_PHPT_JOBS="$jobs" \\
    PTN_PHPT_STREAM_BATCH_SIZE="$stream_batch_size" \\
    PTN_PHPT_STALL_TIMEOUT="$stall_timeout" \\
    PTN_PHPT_SINGLE_WALL_TIMEOUT="$single_wall_timeout" \\
    tools/start-local-full-phpt-run.sh
}

recover_one_row() {
  tools/run-phpt-local-stream-supervisor.py \\
    --out-dir "$run_dir" \\
    "\${manifest_args[@]}" \\
    --php-src "$php_src" \\
    --phpc-bin "$phpc_bin" \\
    --timeout "$timeout_seconds" \\
    --jobs "$jobs" \\
    --stream-batch-size "$stream_batch_size" \\
    --stall-timeout "$stall_timeout" \\
    --single-wall-timeout "$single_wall_timeout" \\
    --recover-one-row
}

while true; do
  state="\$(awk -F '\\t' '\$1 == \"state\" { print \$2; exit }' \"$run_dir/status.tsv\" 2>/dev/null || true)"
  if [[ "\$state" == "complete" ]]; then
    printf '%s corpus complete; watchdog exiting\\n' "\$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$run_dir/watchdog.log"
    exit 0
  fi

  if env -u TMUX tmux has-session -t "$primary_session" 2>/dev/null; then
    primary_seen=1
    sleep "$interval"
    continue
  fi

  if [[ "\$primary_seen" == "1" ]]; then
    printf '%s primary disappeared; isolating first unreported row\\n' "\$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$run_dir/watchdog.log"
    recover_one_row >> "$run_dir/watchdog.log" 2>&1 || \\
      printf '%s one-row recovery failed\\n' "\$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$run_dir/watchdog.log"
    primary_seen=0
  fi

  printf '%s primary session missing; restarting\\n' "\$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$run_dir/watchdog.log"
  if start_primary >> "$run_dir/watchdog.log" 2>&1; then
    primary_seen=1
  else
    printf '%s primary launch command failed\\n' "\$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$run_dir/watchdog.log"
  fi
  sleep "$interval"
done
EOF
chmod +x "$runner"

if tmux -L "$watchdog_socket" has-session -t "$watchdog_session" 2>/dev/null; then
  echo "watchdog session already running: $watchdog_session"
else
  tmux -L "$watchdog_socket" new-session -d -s "$watchdog_session" "$runner"
  echo "watchdog_session=$watchdog_session"
fi

cat <<EOF
primary_session=$primary_session
run_dir=$run_dir
watch=tools/phpt-local-progress.sh .runtime/local-full-phpt-current 5
log=$run_dir/watchdog.log
EOF
