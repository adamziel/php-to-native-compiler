#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if ! command -v tmux >/dev/null 2>&1; then
  echo "tmux is required" >&2
  exit 1
fi

stamp="$(date -u +%Y%m%dT%H%M%SZ)"
session="ptn-full-phpt-local-$stamp"
run_dir="$repo_root/.runtime/local-full-phpt-$stamp"
current_link="$repo_root/.runtime/local-full-phpt-current"
php_src="${PHP_SRC_PHPT:-/home/claude/php-src-phpt}"
phpc_bin="${PHPC_BIN:-$repo_root/target/release/phpc}"
timeout_seconds="${PTN_PHPT_TEST_TIMEOUT:-60}"
jobs="${PTN_PHPT_JOBS:-8}"
stream_batch_size="${PTN_PHPT_STREAM_BATCH_SIZE:-500}"
stall_timeout="${PTN_PHPT_STALL_TIMEOUT:-600}"
single_wall_timeout="${PTN_PHPT_SINGLE_WALL_TIMEOUT:-300}"

mkdir -p "$run_dir"
ln -sfn "$run_dir" "$current_link"

runner="$run_dir/tmux-runner.sh"
cat > "$runner" <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd "$repo_root"
if [[ ! -x "$phpc_bin" ]]; then
  cargo build --locked --release --bin phpc
fi
exec tools/run-phpt-local-stream-supervisor.py \\
  --out-dir "$run_dir" \\
  --php-src "$php_src" \\
  --phpc-bin "$phpc_bin" \\
  --timeout "$timeout_seconds" \\
  --jobs "$jobs" \\
  --stream-batch-size "$stream_batch_size" \\
  --stall-timeout "$stall_timeout" \\
  --single-wall-timeout "$single_wall_timeout"
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
EOF
