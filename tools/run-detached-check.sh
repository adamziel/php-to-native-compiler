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

command_name=${1##*/}
command_name=${command_name//$'\t'/ }
command_name=${command_name//$'\r'/ }
command_name=${command_name//$'\n'/ }
if [[ ${#command_name} -gt 80 ]]; then
  command_name="${command_name:0:77}..."
fi
command_summary="program=$command_name argc=$#"

if ! command -v tmux >/dev/null 2>&1; then
  echo "tmux is required for detached checks" >&2
  exit 1
fi

safe_name="$(printf '%s' "$name" | tr -c 'A-Za-z0-9_.-' '-')"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
root="${PTN_DETACHED_CHECK_ROOT:-.runtime/detached-checks}"
run_dir="$root/${safe_name}-${stamp}"
mkdir -p "$run_dir"
run_dir="$(cd "$run_dir" && pwd)"

runner="$run_dir/run.sh"
log="$run_dir/run.log"
status="$run_dir/status.tsv"
session="ptn-check-${safe_name}-${stamp}"

{
  printf '#!/usr/bin/env bash\n'
  printf 'set +e\n'
  printf 'cd %q || exit 97\n' "$(pwd)"
  printf 'log=%q\n' "$log"
  printf 'status=%q\n' "$status"
  printf ': > "$log"\n'
  printf 'printf "state\\trunning\\nstarted_at_utc\\t%%s\\n" "$(date -u +%%Y-%%m-%%dT%%H:%%M:%%SZ)" > "$status"\n'
  printf 'printf "runner_pid\\t%%s\\n" "$$" >> "$status"\n'
  printf 'printf "command\\t%%s\\n" %q >> "$status"\n' "$command_summary"
  printf 'write_final() {\n'
  printf '  code="$1"\n'
  printf '  state="$2"\n'
  printf '  printf "===== command EXIT %%s %%s =====\\n" "$code" "$(date -u +%%Y-%%m-%%dT%%H:%%M:%%SZ)" >> "$log"\n'
  printf '  printf "exit\\t%%s\\nstate\\t%%s\\nfinished_at_utc\\t%%s\\n" "$code" "$state" "$(date -u +%%Y-%%m-%%dT%%H:%%M:%%SZ)" >> "$status"\n'
  printf '}\n'
  printf 'command_pid=\n'
  printf 'interrupted() {\n'
  printf '  signal="$1"\n'
  printf '  code="$2"\n'
  printf '  trap - INT TERM\n'
  printf '  if [ -n "${command_pid:-}" ]; then\n'
  printf '    kill -s "$signal" "$command_pid" 2>/dev/null\n'
  printf '    wait "$command_pid" 2>/dev/null || true\n'
  printf '  fi\n'
  printf '  write_final "$code" interrupted\n'
  printf '  exit "$code"\n'
  printf '}\n'
  printf '# A detached tmux pane may receive HUP when its client/session changes.\n'
  printf '# Preserve the check and pass the ignored disposition to its child.\n'
  printf "trap '' HUP\n"
  printf "trap 'interrupted INT 130' INT\n"
  printf "trap 'interrupted TERM 143' TERM\n"
  printf 'export PTN_DETACHED_CHECK=1\n'
  printf 'printf "\\n===== command START %%s =====\\n" "$(date -u +%%Y-%%m-%%dT%%H:%%M:%%SZ)" >> "$log"\n'
  printf '%q ' "$@"
  printf '>> "$log" 2>&1 &\n'
  printf 'command_pid=$!\n'
  printf 'wait "$command_pid"\n'
  printf 'code=$?\n'
  printf 'if [ "$code" -eq 0 ]; then\n'
  printf '  state=passed\n'
  printf 'else\n'
  printf '  state=failed\n'
  printf 'fi\n'
  printf 'write_final "$code" "$state"\n'
} > "$runner"
chmod +x "$runner"

tmux new-session -d -s "$session" "$runner"

printf 'session=%s\n' "$session"
printf 'run_dir=%s\n' "$run_dir"
printf 'status=%s\n' "$status"
printf 'log=%s\n' "$log"
