#!/usr/bin/env sh

# Infinite unattended Codex runner. Stop it with the process supervisor or Ctrl-C.
# This script intentionally has no loop-level stop condition.

repo_root=$(git rev-parse --show-toplevel 2>/dev/null)
if [ -z "$repo_root" ]; then
  printf '%s\n' "codex-yolo: not inside a git repository" >&2
  exit 1
fi

cd "$repo_root" || exit 1

memory_file="${CODEX_YOLO_MEMORY:-$repo_root/docs/LOOP_MEMORY.md}"
log_dir="${CODEX_YOLO_LOG_DIR:-$repo_root/.codex-yolo/logs}"
codex_bin="${CODEX_BIN:-codex}"
test_script="$repo_root/tools/run-tests.sh"
checkpoint_script="$repo_root/tools/checkpoint.sh"

mkdir -p "$log_dir"

if [ ! -f "$memory_file" ]; then
  {
    printf '%s\n' "# Codex YOLO Loop Memory"
    printf '\n%s\n' "Created by tools/codex-yolo-forever.sh."
  } >"$memory_file"
fi

append_memory() {
  message=$1
  timestamp=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
  {
    printf '\n## Loop Event %s\n\n' "$timestamp"
    printf '%s\n' "$message"
  } >>"$memory_file"
}

run_tests_no_stop() {
  phase=$1
  log_file=$2
  if [ -x "$test_script" ]; then
    printf 'codex-yolo: running tests %s\n' "$phase" | tee -a "$log_file"
    "$test_script" >>"$log_file" 2>&1
    return $?
  fi

  printf '%s\n' "codex-yolo: missing executable tools/run-tests.sh" | tee -a "$log_file"
  return 127
}

print_roadmap_summary() {
  phase=$1
  tasks_file="$repo_root/docs/NEXT_TASKS.md"
  head_commit=$(git rev-parse --short HEAD 2>/dev/null || printf 'unknown')
  dirty_count=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
  progress_width=${CODEX_YOLO_PROGRESS_WIDTH:-32}

  case "$progress_width" in
    '' | *[!0-9]*)
      progress_width=32
      ;;
  esac
  if [ "$progress_width" -lt 10 ]; then
    progress_width=10
  fi

  if [ ! -r "$tasks_file" ]; then
    printf 'codex-yolo: roadmap summary unavailable for %s; missing %s\n' "$phase" "$tasks_file"
    return 0
  fi

  awk -v phase="$phase" -v head_commit="$head_commit" -v dirty_count="$dirty_count" -v width="$progress_width" '
    function trim(value) {
      gsub(/^[[:space:]]+/, "", value)
      gsub(/[[:space:]]+$/, "", value)
      return value
    }

    function marker(index) {
      if (total[index] == 0) {
        return "[ ]"
      }
      if (done[index] == total[index]) {
        return "[x]"
      }
      if (index == active_milestone || done[index] > 0) {
        return "[>]"
      }
      return "[ ]"
    }

    /^##[[:space:]]+/ {
      milestone_count++
      title = $0
      sub(/^##[[:space:]]+/, "", title)
      milestone[milestone_count] = trim(title)
      next
    }

    /^[[:space:]]*-[[:space:]]+\[[xX ]\]/ {
      if (milestone_count == 0) {
        next
      }

      total[milestone_count]++
      total_tasks++

      if ($0 ~ /^[[:space:]]*-[[:space:]]+\[[xX]\]/) {
        done[milestone_count]++
        done_tasks++
      } else {
        open_tasks++
        if (next_task == "") {
          next_task = $0
          sub(/^[[:space:]]*-[[:space:]]+\[[[:space:]]\][[:space:]]*/, "", next_task)
          next_task = trim(next_task)
          active_milestone = milestone_count
        }
      }
      next
    }

    END {
      percent = total_tasks == 0 ? 0 : int((done_tasks * 100) / total_tasks)
      filled = total_tasks == 0 ? 0 : int((done_tasks * width) / total_tasks)
      bar = ""
      for (index = 1; index <= width; index++) {
        bar = bar (index <= filled ? "#" : "-")
      }

      printf "\n"
      printf "codex-yolo: roadmap summary (%s)\n", phase
      printf "  HEAD: %s\n", head_commit
      printf "  Worktree changes: %s file(s)\n", dirty_count
      printf "  Tasks: %d/%d complete, %d open (%d%%)\n", done_tasks, total_tasks, open_tasks, percent
      printf "  [%s] %d%%\n", bar, percent

      if (next_task != "") {
        printf "  Next: %s\n", next_task
      } else if (total_tasks > 0) {
        printf "  Next: none; roadmap queue is fully checked\n"
      } else {
        printf "  Next: none; no roadmap tasks found\n"
      }

      printf "\n"
      printf "  Milestones:\n"
      for (index = 1; index <= milestone_count; index++) {
        if (total[index] == 0) {
          continue
        }

        milestone_percent = int((done[index] * 100) / total[index])
        printf "    %s %-48s %3d/%-3d %3d%%", marker(index), milestone[index], done[index], total[index], milestone_percent
        if (index == active_milestone) {
          printf "  <- in progress"
        }
        printf "\n"
      }
      printf "\n"
    }
  ' "$tasks_file"
}

round=1
while :; do
  timestamp=$(date -u '+%Y%m%dT%H%M%SZ')
  round_log="$log_dir/round-${timestamp}-${round}.log"
  prompt_file="$log_dir/prompt-${timestamp}-${round}.md"

  head_commit=$(git rev-parse --short HEAD 2>/dev/null || printf 'unknown')
  status_short=$(git status --short 2>/dev/null || true)
  next_tasks=$(sed -n '1,240p' docs/NEXT_TASKS.md 2>/dev/null || true)
  progress=$(sed -n '1,220p' docs/PROGRESS.md 2>/dev/null || true)
  memory=$(sed -n '1,260p' "$memory_file" 2>/dev/null || true)

  {
    printf '# Unattended Codex YOLO Loop Prompt\n\n'
    printf 'You are working in `%s`.\n\n' "$repo_root"
    printf 'This is an unattended infinite loop run. Work in small, honest checkpoints.\n'
    printf 'Do not claim broad PHP support. Do not add placeholders that pretend features work.\n\n'
    printf 'Read and obey these files first:\n\n'
    printf -- '- `AGENTS.md`\n'
    printf -- '- `README.md`\n'
    printf -- '- `docs/ARCHITECTURE.md`\n'
    printf -- '- `docs/SUPPORT.md`\n'
    printf -- '- `docs/PROGRESS.md`\n'
    printf -- '- `docs/OPERATIONS.md`\n'
    printf -- '- `docs/NEXT_TASKS.md`\n'
    printf -- '- `docs/LOOP_MEMORY.md`\n\n'
    printf 'Current loop round: `%s`.\n' "$round"
    printf 'Current HEAD before this pass: `%s`.\n\n' "$head_commit"
    printf 'Current `git status --short` before this pass:\n\n```text\n%s\n```\n\n' "$status_short"
    printf 'Durable loop memory snapshot:\n\n```markdown\n%s\n```\n\n' "$memory"
    printf 'Progress snapshot:\n\n```markdown\n%s\n```\n\n' "$progress"
    printf 'Next task queue snapshot:\n\n```markdown\n%s\n```\n\n' "$next_tasks"
    printf 'Instructions for this pass:\n\n'
    printf '1. If the worktree is dirty, inspect the dirty changes first and repair/integrate them before starting a new task.\n'
    printf '2. Otherwise, take the first unchecked task in `docs/NEXT_TASKS.md` that can be completed honestly in this pass.\n'
    printf '3. Implement a small correct slice with executable code, tests, CLI coverage, docs, and named unsupported gaps.\n'
    printf '4. Run `tools/run-tests.sh` and any focused command needed to prove the change.\n'
    printf '5. Update `docs/PROGRESS.md`, `docs/SUPPORT.md`, `docs/NEXT_TASKS.md`, and `docs/LOOP_MEMORY.md`.\n'
    printf '6. Commit only through `tools/checkpoint.sh "specific message"` after tests pass.\n'
    printf '7. If tests fail, keep working until they pass or record the blocker in `docs/LOOP_MEMORY.md`; do not mark the task done.\n'
  } >"$prompt_file"

  print_roadmap_summary "round $round start" | tee -a "$round_log"
  append_memory "- Starting round $round at $timestamp from HEAD \`$head_commit\`."

  printf 'codex-yolo: round %s starting at %s\n' "$round" "$timestamp" | tee -a "$round_log"
  run_tests_no_stop "before round $round" "$round_log"
  pre_test_status=$?
  append_memory "- Pre-round $round test exit code: \`$pre_test_status\`."

  if command -v "$codex_bin" >/dev/null 2>&1; then
    printf 'codex-yolo: running %s exec in yolo mode\n' "$codex_bin" | tee -a "$round_log"
    "$codex_bin" exec \
      --dangerously-bypass-approvals-and-sandbox \
      -C "$repo_root" \
      <"$prompt_file" >>"$round_log" 2>&1
    codex_status=$?
  else
    printf 'codex-yolo: %s not found on PATH\n' "$codex_bin" | tee -a "$round_log"
    codex_status=127
  fi
  append_memory "- Codex round $round exit code: \`$codex_status\`. Log: \`$round_log\`."

  run_tests_no_stop "after round $round" "$round_log"
  post_test_status=$?
  append_memory "- Post-round $round test exit code: \`$post_test_status\`."

  if [ "$post_test_status" -eq 0 ] && [ -x "$checkpoint_script" ]; then
    append_memory "- Post-round $round tests passed; running checkpoint for this round."
    "$checkpoint_script" "codex yolo checkpoint: round $round" >>"$round_log" 2>&1
  else
    append_memory "- No checkpoint after round $round because tests failed or checkpoint script is missing."
  fi

  round=$((round + 1))
done
