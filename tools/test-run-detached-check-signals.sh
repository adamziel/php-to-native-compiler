#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
runner="$repo_root/tools/run-detached-check.sh"

if ! command -v tmux >/dev/null 2>&1; then
  echo "SKIP: tmux is unavailable"
  exit 0
fi

test_root="$(mktemp -d "${TMPDIR:-/tmp}/ptn-detached-signal.XXXXXX")"
declare -a sessions=()

cleanup() {
  local session
  for session in "${sessions[@]}"; do
    tmux kill-session -t "$session" 2>/dev/null || true
  done
  rm -rf "$test_root"
}
trap cleanup EXIT

status_value() {
  local status=$1
  local key=$2
  awk -F '\t' -v key="$key" '$1 == key { value = $2 } END { print value }' "$status" 2>/dev/null
}

wait_for_nonempty_status() {
  local status=$1
  local key=$2
  local deadline=$((SECONDS + 20))

  while (( SECONDS < deadline )); do
    WAIT_VALUE="$(status_value "$status" "$key")"
    if [[ -n "$WAIT_VALUE" ]]; then
      return 0
    fi
    sleep 0.1
  done
  return 1
}

wait_for_status() {
  local status=$1
  local key=$2
  local expected=$3
  local deadline=$((SECONDS + 20))

  while (( SECONDS < deadline )); do
    if [[ "$(status_value "$status" "$key")" == "$expected" ]]; then
      return 0
    fi
    sleep 0.1
  done
  return 1
}

wait_for_log_text() {
  local log=$1
  local expected=$2
  local deadline=$((SECONDS + 20))

  while (( SECONDS < deadline )); do
    if grep -Fq -- "$expected" "$log" 2>/dev/null; then
      return 0
    fi
    sleep 0.1
  done
  return 1
}

start_check() {
  local name=$1
  shift
  local launch

  launch="$(PTN_DETACHED_CHECK_ROOT="$test_root" "$runner" "$name" -- "$@")"
  CHECK_DIR="$(awk -F= '$1 == "run_dir" { print $2 }' <<< "$launch")"
  CHECK_SESSION="$(awk -F= '$1 == "session" { print $2 }' <<< "$launch")"
  if [[ -z "$CHECK_DIR" || -z "$CHECK_SESSION" ]]; then
    echo "failed to launch detached check: $name" >&2
    return 1
  fi
  sessions+=("$CHECK_SESSION")
}

start_check "signal-hup" bash -c 'printf "hup-child-started\n"; sleep 2; printf "hup-child-completed\n"'
hup_status="$CHECK_DIR/status.tsv"
hup_log="$CHECK_DIR/run.log"
wait_for_log_text "$hup_log" "hup-child-started"
wait_for_nonempty_status "$hup_status" runner_pid
hup_pid="$WAIT_VALUE"
kill -HUP "$hup_pid"
wait_for_status "$hup_status" state passed
[[ "$(status_value "$hup_status" exit)" == "0" ]]
wait_for_log_text "$hup_log" "hup-child-completed"

start_check "signal-term" bash -c 'trap '\''printf "term-child-stopped\n"; exit 0'\'' TERM; printf "term-child-started\n"; while :; do sleep 1; done'
term_status="$CHECK_DIR/status.tsv"
term_log="$CHECK_DIR/run.log"
wait_for_log_text "$term_log" "term-child-started"
wait_for_nonempty_status "$term_status" runner_pid
term_pid="$WAIT_VALUE"
kill -TERM "$term_pid"
wait_for_status "$term_status" state interrupted
[[ "$(status_value "$term_status" exit)" == "143" ]]
wait_for_log_text "$term_log" "term-child-stopped"

start_check "signal-timeout" timeout 1 bash -c 'sleep 10'
timeout_status="$CHECK_DIR/status.tsv"
wait_for_status "$timeout_status" state failed
[[ "$(status_value "$timeout_status" exit)" == "124" ]]

echo "PASS: detached HUP survives; TERM cancels; timeout remains enforced"
