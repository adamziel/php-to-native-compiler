#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if ! command -v tmux >/dev/null 2>&1; then
  echo "tmux is required" >&2
  exit 1
fi

stamp="$(date -u +%Y%m%dT%H%M%SZ)"
session="${PTN_PHPT_SESSION:-ptn-full-phpt-local-$stamp}"
resume_run_dir="${PTN_PHPT_RESUME_RUN_DIR:-}"
if [[ -n "$resume_run_dir" ]]; then
  run_dir="$resume_run_dir"
  if [[ ! -d "$run_dir" ]]; then
    echo "resume run directory does not exist: $run_dir" >&2
    exit 1
  fi
else
  run_dir="$repo_root/.runtime/local-full-phpt-$stamp"
fi
current_link="$repo_root/.runtime/local-full-phpt-current"
update_current_link="${PTN_PHPT_UPDATE_CURRENT_LINK:-1}"
php_src="${PHP_SRC_PHPT:-/home/claude/php-src-phpt}"
phpc_bin="${PHPC_BIN:-$repo_root/target/release/phpc}"
manifest="${PTN_PHPT_MANIFEST:-}"
timeout_seconds="${PTN_PHPT_TEST_TIMEOUT:-60}"
jobs="${PTN_PHPT_JOBS:-8}"
stream_batch_size="${PTN_PHPT_STREAM_BATCH_SIZE:-500}"
stall_timeout="${PTN_PHPT_STALL_TIMEOUT:-600}"
single_wall_timeout="${PTN_PHPT_SINGLE_WALL_TIMEOUT:-300}"

if [[ -n "$manifest" && ! -f "$manifest" ]]; then
  echo "PHPT manifest does not exist: $manifest" >&2
  exit 1
fi

mkdir -p "$run_dir"
if [[ "$update_current_link" != "0" ]]; then
  ln -sfn "$run_dir" "$current_link"
fi

runner="$run_dir/tmux-runner.sh"
cat > "$runner" <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd "$repo_root"
if [[ ! -x "$phpc_bin" ]]; then
  cargo build --locked --release --bin phpc
fi
manifest="$manifest"
manifest_args=()
if [[ -n "\$manifest" ]]; then
  manifest_args=(--manifest "\$manifest")
fi
while true; do
  if tools/run-phpt-local-stream-supervisor.py \\
    --out-dir "$run_dir" \\
    "\${manifest_args[@]}" \\
    --php-src "$php_src" \\
    --phpc-bin "$phpc_bin" \\
    --timeout "$timeout_seconds" \\
    --jobs "$jobs" \\
    --stream-batch-size "$stream_batch_size" \\
    --stall-timeout "$stall_timeout" \\
    --single-wall-timeout "$single_wall_timeout" \\
    >> "$run_dir/supervisor.log" 2>&1; then
    exit 0
  else
    exit_code=\$?
    printf '%s supervisor exited %s; retrying in 5s\\n' "\$(date -u +%Y-%m-%dT%H:%M:%SZ)" "\$exit_code" >> "$run_dir/supervisor-restarts.log"
    sleep 5
  fi
done
EOF
chmod +x "$runner"

tmux new-session -d -s "$session" "$runner"

cat <<EOF
session=$session
run_dir=$run_dir
current=$current_link
watch=tools/phpt-local-progress.sh .runtime/local-full-phpt-current 5
status=$run_dir/status.tsv
results=$run_dir/row-results.tsv
log=$run_dir/run-tests.log
manifest=$manifest
EOF
