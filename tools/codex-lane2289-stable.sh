#!/usr/bin/env sh
set -eu

repo_root=$(git rev-parse --show-toplevel 2>/dev/null) || {
  printf '%s\n' "codex-lane2289-stable: not inside a git repository" >&2
  exit 1
}
cd "$repo_root"

codex_bin="${CODEX_BIN:-codex}"
duration_seconds="${STABLE_SECONDS:-3600}"
reasoning_effort="${CODEX_REASONING_EFFORT:-medium}"
log_dir="${CODEX_STABLE_LOG_DIR:-$repo_root/.codex-stable/lane2289}"
state_file="$log_dir/state.log"
last_message_file="$log_dir/last-message.md"
lock_dir="$log_dir/lock"

case "$duration_seconds" in
  '' | *[!0-9]*)
    printf '%s\n' "codex-lane2289-stable: STABLE_SECONDS must be a positive integer" >&2
    exit 2
    ;;
esac
[ "$duration_seconds" -ge 60 ] || {
  printf '%s\n' "codex-lane2289-stable: STABLE_SECONDS must be at least 60" >&2
  exit 2
}

command -v "$codex_bin" >/dev/null 2>&1 || {
  printf 'codex-lane2289-stable: %s not found on PATH\n' "$codex_bin" >&2
  exit 127
}

mkdir -p "$log_dir"
if ! mkdir "$lock_dir" 2>/dev/null; then
  printf 'codex-lane2289-stable: another runner appears active: %s\n' "$lock_dir" >&2
  exit 3
fi
trap 'rmdir "$lock_dir" 2>/dev/null || true' EXIT INT TERM

start_epoch=$(date +%s)
end_epoch=$((start_epoch + duration_seconds))
round=1
crashes=0

write_prompt() {
  prompt_file=$1
  round_log=$2
  now_iso=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
  head_commit=$(git rev-parse --short HEAD 2>/dev/null || printf 'unknown')
  status_short=$(git status --short 2>/dev/null || true)
  next_unchecked=$(awk '/^- \[ \]/ { sub(/^- \[ \][[:space:]]*/, ""); print; exit }' docs/NEXT_TASKS.md 2>/dev/null || true)

  {
    printf '# Stable Lane 2289 Codex Worker\n\n'
    printf 'You are working in `%s` at `%s`.\n\n' "$repo_root" "$now_iso"
    printf 'Continue the exact php-to-native compiler task already in progress: Lane 2289 runtime-cell copy-source work and its immediate next COW frontier. Do not restart the project, do not switch goals, and do not broaden claims beyond executable evidence.\n\n'
    printf 'Critical stability instructions:\n\n'
    printf -- '- Do not use `codex resume` and do not try to recover the old session transcript. The old thread has multi-million-token history and is the suspected crash trigger.\n'
    printf -- '- Reconstruct context only from the current worktree files listed below.\n'
    printf -- '- Run one heavyweight command at a time. Do not run build/test/fmt/checkpoint commands in parallel.\n'
    printf -- '- Use bounded commands for Rust checks: `timeout`, `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and `RUST_TEST_THREADS=1` for test binaries.\n'
    printf -- '- Redirect verbose command output to files under `%s`; inspect and report short tails only.\n' "$log_dir"
    printf -- '- Do not run `tools/checkpoint.sh` until focused checks are stable. It runs the full suite and commits only after that gate passes.\n'
    printf -- '- Do not run `rustfmt` or `cargo fmt` on `compiler/src/interpreter.rs` in this VM. Evidence: rustfmt was OOM-killed at about 7.7 GiB RSS on 2026-05-20. Use `git diff --check` plus focused compile/tests instead.\n'
    printf -- '- Prefer focused validation first: `git diff --check`, `cargo check -q -p phpc`, focused `cargo test`, and `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone2289`.\n'
    printf -- '- Leave unrelated dirty work alone. If the current dirty Lane 2289 patch needs repair, repair it in place.\n\n'
    printf 'Read these files first, but summarize internally instead of dumping them:\n\n'
    printf -- '- `AGENTS.md`\n'
    printf -- '- `GOAL.MD`\n'
    printf -- '- `docs/PROGRESS.md` near the newest Lane 2289 entries\n'
    printf -- '- `docs/ARCHITECTURE.md`\n'
    printf -- '- `docs/SUPPORT.md`\n'
    printf -- '- `docs/NEXT_TASKS.md`\n'
    printf -- '- `docs/LOOP_MEMORY.md`\n'
    printf -- '- Current diffs in `compiler/src/interpreter.rs`, docs, and `tests/fixtures/milestone2289/`\n\n'
    printf 'Current HEAD before this worker: `%s`.\n\n' "$head_commit"
    printf 'Current git status:\n\n```text\n%s\n```\n\n' "$status_short"
    printf 'First unchecked task from `docs/NEXT_TASKS.md`:\n\n```text\n%s\n```\n\n' "$next_unchecked"
    if [ -s "$last_message_file" ]; then
      printf 'Previous worker final message:\n\n```text\n'
      tail -80 "$last_message_file"
      printf '\n```\n\n'
    fi
    printf 'This runner log is `%s`. Append concise handoff notes to `docs/LOOP_MEMORY.md` before finishing.\n\n' "$round_log"
    printf 'Required final response: short status with changes, tests run, blockers if any, and next step. Keep working until either the Lane 2289 patch is honestly checkpoint-ready or a concrete blocker is documented.\n'
  } >"$prompt_file"
}

printf 'codex-lane2289-stable: starting at %s for %s seconds\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$duration_seconds" | tee -a "$state_file"
printf 'codex-lane2289-stable: repo=%s\n' "$repo_root" | tee -a "$state_file"
printf 'codex-lane2289-stable: this runner intentionally starts fresh codex exec sessions, never resume\n' | tee -a "$state_file"

while [ "$(date +%s)" -lt "$end_epoch" ]; do
  timestamp=$(date -u '+%Y%m%dT%H%M%SZ')
  round_log="$log_dir/round-${timestamp}-${round}.log"
  prompt_file="$log_dir/prompt-${timestamp}-${round}.md"
  write_prompt "$prompt_file" "$round_log"

  printf '\n[%s] round %s start; crashes so far=%s; log=%s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$round" "$crashes" "$round_log" | tee -a "$state_file"
  set +e
  "$codex_bin" exec \
    -c "model_reasoning_effort=\"$reasoning_effort\"" \
    --dangerously-bypass-approvals-and-sandbox \
    -C "$repo_root" \
    --output-last-message "$last_message_file" \
    - <"$prompt_file" >>"$round_log" 2>&1
  status=$?
  set -e

  printf '[%s] round %s exit=%s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$round" "$status" | tee -a "$state_file"
  if [ "$status" -ne 0 ]; then
    crashes=$((crashes + 1))
    {
      printf '\n## Stable runner crash %s\n\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
      printf -- '- Round: `%s`\n' "$round"
      printf -- '- Exit status: `%s`\n' "$status"
      printf -- '- Log: `%s`\n' "$round_log"
      printf -- '- Adjustment: next round remains fresh `codex exec`, avoids resume, avoids rustfmt on `compiler/src/interpreter.rs`, keeps checks bounded, and uses log tails only.\n'
    } >>docs/LOOP_MEMORY.md
    printf 'codex-lane2289-stable: nonzero exit; last log lines:\n' | tee -a "$state_file"
    tail -80 "$round_log" | tee -a "$state_file"
  fi

  round=$((round + 1))
done

printf 'codex-lane2289-stable: completed stability window at %s; crashes=%s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$crashes" | tee -a "$state_file"
[ "$crashes" -eq 0 ]
