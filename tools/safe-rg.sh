#!/usr/bin/env bash
set -euo pipefail

limit="${SAFE_RG_LIMIT:-80}"
if [ "${1:-}" = "--limit" ]; then
  shift
  limit="${1:-80}"
  shift || true
fi

case "$limit" in
  ''|*[!0-9]*)
    printf 'safe-rg: invalid limit: %s\n' "$limit" >&2
    exit 2
    ;;
esac

if [ "$#" -lt 1 ]; then
  printf 'usage: %s [--limit N] PATTERN [PATH...]\n' "$0" >&2
  exit 2
fi

pattern="$1"
shift

if [ "$#" -eq 0 ]; then
  set -- src tests AGENTS.md README.md STATUS.md
fi

for path in "$@"; do
  case "$path" in
    /|/home|/home/claude|/home/claude/|~|~/*)
      printf 'safe-rg: refusing broad workspace root: %s\n' "$path" >&2
      exit 2
      ;;
  esac
done

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

set +e
rg -n --no-heading --color never -m 20 --max-filesize 1M "$pattern" "$@" \
  | head -n "$limit" >"$tmp"
status="${PIPESTATUS[0]}"
set -e

cat "$tmp"

lines="$(wc -l <"$tmp" | tr -d ' ')"
if [ "$lines" -ge "$limit" ]; then
  printf 'safe-rg: output truncated at %s lines; narrow the path or pattern.\n' "$limit" >&2
fi

if [ "$status" -eq 0 ] || [ "$status" -eq 1 ] || [ "$status" -eq 141 ]; then
  exit 0
fi
exit "$status"
