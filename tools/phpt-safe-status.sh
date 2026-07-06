#!/usr/bin/env bash
set -euo pipefail

limit="${1:-12}"
case "$limit" in
  ''|*[!0-9]*)
    limit=12
    ;;
esac

root="${PTN_DETACHED_CHECK_ROOT:-.runtime/detached-checks}"

printf 'detached_checks\n'
if [ -d "$root" ]; then
  find "$root" -maxdepth 2 -name status.tsv -printf '%T@ %p\n' 2>/dev/null \
    | sort -nr \
    | head -n "$limit" \
    | while IFS=' ' read -r _ status; do
        run_dir="$(dirname "$status")"
        name="$(basename "$run_dir")"
        state="$(awk -F '\t' '$1 == "state" { value = $2 } END { print value }' "$status")"
        exit_code="$(awk -F '\t' '$1 == "exit" { value = $2 } END { print value }' "$status")"
        started="$(awk -F '\t' '$1 == "started_at_utc" { value = $2 } END { print value }' "$status")"
        finished="$(awk -F '\t' '$1 == "finished_at_utc" { value = $2 } END { print value }' "$status")"
        printf '%s\tstate=%s\texit=%s\tstarted=%s\tfinished=%s\n' \
          "$name" "${state:-unknown}" "${exit_code:-}" "${started:-}" "${finished:-}"
      done
else
  printf 'none\n'
fi

printf '\ntmux_check_sessions\n'
if command -v tmux >/dev/null 2>&1; then
  tmux list-sessions -F '#{session_name}\twindows=#{session_windows}\tattached=#{session_attached}' 2>/dev/null \
    | awk -v limit="$limit" 'BEGIN { count = 0 } /^ptn-check-/ { print; count++; if (count >= limit) exit } END { if (count == 0) print "none" }'
else
  printf 'tmux unavailable\n'
fi

latest_dashboard="/home/claude/.local/state/ptn-full-phpt-dashboard-loop/latest.tsv"
printf '\nactive_partial_dashboard_snapshot\n'
if [ -f "$latest_dashboard" ]; then
  awk -F '\t' '
    $1 == "refreshed_at_utc" ||
    $1 == "complete" ||
    $1 == "active_source_commit" ||
    $1 == "active_run" ||
    $1 == "active_tests" ||
    $1 == "active_passed" ||
    $1 == "active_failed" ||
    $1 == "active_skipped" ||
    $1 == "active_warned" ||
    $1 == "active_unknown" { print $1 "=" $2 }
  ' "$latest_dashboard"
else
  printf 'missing\n'
fi
