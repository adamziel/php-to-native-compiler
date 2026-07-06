#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 NAME -- COMMAND [ARG ...]" >&2
}

if [ "$#" -lt 3 ] || [ "$2" != "--" ]; then
  usage
  exit 2
fi

name="$1"
shift 2

if ! command -v tmux >/dev/null 2>&1; then
  echo "tmux is required for detached checks" >&2
  exit 1
fi

safe_name="$(printf '%s' "$name" | tr -c 'A-Za-z0-9_.-' '-')"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
root="${PTN_DETACHED_CHECK_ROOT:-.runtime/detached-checks}"
run_dir="$root/${safe_name}-${stamp}"
mkdir -p "$run_dir"

runner="$run_dir/run.sh"
log="$run_dir/run.log"
status="$run_dir/status.tsv"
session="ptn-check-${safe_name}-${stamp}"

{
  printf '#!/usr/bin/env bash\n'
  printf 'set +e\n'
  printf 'cd %q || exit 97\n' "$(pwd)"
  printf 'log=%q\n' "$(pwd)/$log"
  printf 'status=%q\n' "$(pwd)/$status"
  printf ': > "$log"\n'
  printf 'printf "state\\trunning\\nstarted_at_utc\\t%%s\\n" "$(date -u +%%Y-%%m-%%dT%%H:%%M:%%SZ)" > "$status"\n'
  printf 'printf "command\\t%s\\n" ' "$(printf '%q ' "$@")"
  printf '>> "$status"\n'
  printf 'printf "\\n===== command START %%s =====\\n" "$(date -u +%%Y-%%m-%%dT%%H:%%M:%%SZ)" >> "$log"\n'
  printf '%q ' "$@"
  printf '>> "$log" 2>&1\n'
  printf 'code=$?\n'
  printf 'printf "===== command EXIT %%s %%s =====\\n" "$code" "$(date -u +%%Y-%%m-%%dT%%H:%%M:%%SZ)" >> "$log"\n'
  printf 'printf "exit\\t%%s\\nstate\\t%%s\\nfinished_at_utc\\t%%s\\n" "$code" "$([ "$code" -eq 0 ] && echo passed || echo failed)" "$(date -u +%%Y-%%m-%%dT%%H:%%M:%%SZ)" >> "$status"\n'
} > "$runner"
chmod +x "$runner"

tmux new-session -d -s "$session" "$runner"

printf 'session=%s\n' "$session"
printf 'run_dir=%s\n' "$(pwd)/$run_dir"
printf 'status=%s\n' "$(pwd)/$status"
printf 'log=%s\n' "$(pwd)/$log"
