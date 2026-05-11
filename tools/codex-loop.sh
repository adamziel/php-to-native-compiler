#!/usr/bin/env sh
set -eu

die() {
  printf '%s\n' "$*" >&2
  exit 1
}

repo_root=$(git rev-parse --show-toplevel 2>/dev/null) || die "codex-loop: not inside a git repository"
cd "$repo_root"

test_script="$repo_root/tools/run-tests.sh"
checkpoint_script="$repo_root/tools/checkpoint.sh"
prompt_file="${CODEX_LOOP_PROMPT:-$repo_root/docs/CODEX_LOOP_PROMPT.md}"

[ -x "$test_script" ] || die "codex-loop: missing executable tools/run-tests.sh"
[ -x "$checkpoint_script" ] || die "codex-loop: missing executable tools/checkpoint.sh"
[ -r "$prompt_file" ] || die "codex-loop: prompt file is not readable: $prompt_file"

MAX_ROUNDS="${MAX_ROUNDS:-1}"
case "$MAX_ROUNDS" in
  '' | *[!0-9]*)
    die "codex-loop: MAX_ROUNDS must be a positive integer"
    ;;
esac
[ "$MAX_ROUNDS" -ge 1 ] || die "codex-loop: MAX_ROUNDS must be at least 1"

if [ -z "${CODEX_RUNNER:-}" ]; then
  cat >&2 <<'EOF'
codex-loop: CODEX_RUNNER is required.
Set it to a non-interactive Codex command that reads the prompt from stdin.
Example:
  CODEX_RUNNER='codex exec' MAX_ROUNDS=3 tools/codex-loop.sh
EOF
  exit 2
fi

runner_name=${CODEX_RUNNER%% *}
case "$runner_name" in
  */*)
    [ -x "$runner_name" ] || die "codex-loop: CODEX_RUNNER starts with '$runner_name', which is not executable"
    ;;
  *)
    command -v "$runner_name" >/dev/null 2>&1 ||
      die "codex-loop: CODEX_RUNNER starts with '$runner_name', which is not on PATH"
    ;;
esac

run_tests() {
  phase=$1
  printf 'codex-loop: running full project test suite %s\n' "$phase"
  if ! "$test_script"; then
    die "codex-loop: tests failed $phase; stopping"
  fi
}

round=1
while [ "$round" -le "$MAX_ROUNDS" ]; do
  printf 'codex-loop: starting round %s of %s\n' "$round" "$MAX_ROUNDS"
  run_tests "before round $round"

  printf 'codex-loop: running CODEX_RUNNER for round %s with prompt %s\n' "$round" "$prompt_file"
  if ! CODEX_LOOP_ROUND=$round CODEX_LOOP_MAX_ROUNDS=$MAX_ROUNDS CODEX_LOOP_PROMPT_FILE=$prompt_file sh -c "$CODEX_RUNNER" <"$prompt_file"; then
    die "codex-loop: CODEX_RUNNER failed in round $round; stopping"
  fi

  run_tests "after round $round"

  if ! "$checkpoint_script" "codex checkpoint: round $round of $MAX_ROUNDS"; then
    die "codex-loop: checkpoint failed in round $round; stopping"
  fi

  round=$((round + 1))
done

printf 'codex-loop: completed %s round(s)\n' "$MAX_ROUNDS"
