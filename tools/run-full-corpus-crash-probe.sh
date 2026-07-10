#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
run_root="${PTN_FULL_CRASH_PROBE_ROOT:-/home/claude/.local/state/ptn-full-corpus-crash-probe}"
run_dir="$run_root/runs/$stamp-$$"
worktree="$run_dir/worktree"
status="$run_dir/status.tsv"
summary="$run_dir/summary.tsv"
launcher_log="$run_dir/launcher.log"
shard_count="${PTN_FULL_CRASH_PROBE_SHARDS:-8}"
shard_concurrency="${PTN_FULL_CRASH_PROBE_CONCURRENCY:-2}"
jobs_per_shard="${PTN_FULL_CRASH_PROBE_JOBS_PER_SHARD:-1}"
test_timeout="${PTN_FULL_CRASH_PROBE_TEST_TIMEOUT:-900}"
compiler_profile="${PTN_FULL_CRASH_PROBE_PROFILE:-release}"
minimum_free_gib="${PTN_FULL_CRASH_PROBE_MIN_FREE_GIB:-24}"
expected_corpus_revision="${PTN_FULL_CRASH_PROBE_CORPUS_REVISION:-8c63ec400ce8e07c57a8d9499317b96a8beafb8b}"
expected_corpus_count="${PTN_FULL_CRASH_PROBE_CORPUS_COUNT:-21867}"

if [[ ! "$shard_count" =~ ^[1-9][0-9]*$ || ${#shard_count} -gt 2 || 10#$shard_count -gt 64 ]]; then
  printf 'PTN_FULL_CRASH_PROBE_SHARDS must be a positive integer no greater than 64: %s\n' "$shard_count" >&2
  exit 2
fi
if [[ ! "$shard_concurrency" =~ ^[1-9][0-9]*$ || ${#shard_concurrency} -gt 1 || 10#$shard_concurrency -gt 8 || 10#$shard_concurrency -gt 10#$shard_count ]]; then
  printf 'PTN_FULL_CRASH_PROBE_CONCURRENCY must be between 1 and min(8, shards): %s\n' "$shard_concurrency" >&2
  exit 2
fi
if [[ ! "$jobs_per_shard" =~ ^[1-9][0-9]*$ || ${#jobs_per_shard} -gt 2 || 10#$jobs_per_shard -gt 16 ]]; then
  printf 'PTN_FULL_CRASH_PROBE_JOBS_PER_SHARD must be a positive integer no greater than 16: %s\n' "$jobs_per_shard" >&2
  exit 2
fi
if [[ ! "$test_timeout" =~ ^[1-9][0-9]*$ || ${#test_timeout} -gt 4 || 10#$test_timeout -gt 3600 ]]; then
  printf 'PTN_FULL_CRASH_PROBE_TEST_TIMEOUT must be a positive integer no greater than 3600 seconds: %s\n' "$test_timeout" >&2
  exit 2
fi
if [[ ! "$minimum_free_gib" =~ ^[1-9][0-9]*$ || ${#minimum_free_gib} -gt 3 ]]; then
  printf 'PTN_FULL_CRASH_PROBE_MIN_FREE_GIB must be a positive integer up to three digits: %s\n' "$minimum_free_gib" >&2
  exit 2
fi
if [[ ! "$expected_corpus_revision" =~ ^[0-9a-f]{40}$ ]]; then
  printf 'PTN_FULL_CRASH_PROBE_CORPUS_REVISION must be a full lowercase hexadecimal Git revision: %s\n' "$expected_corpus_revision" >&2
  exit 2
fi
if [[ ! "$expected_corpus_count" =~ ^[1-9][0-9]*$ || ${#expected_corpus_count} -gt 6 || 10#$expected_corpus_count -gt 100000 ]]; then
  printf 'PTN_FULL_CRASH_PROBE_CORPUS_COUNT must be a positive integer no greater than 100000: %s\n' "$expected_corpus_count" >&2
  exit 2
fi

case "$compiler_profile" in
  debug)
    phpc_bin="$worktree/target/debug/phpc"
    cargo_build_args=(build --locked --bin phpc)
    ;;
  release)
    phpc_bin="$worktree/target/release/phpc"
    cargo_build_args=(build --locked --release --bin phpc)
    ;;
  *)
    printf 'PTN_FULL_CRASH_PROBE_PROFILE must be debug or release: %s\n' "$compiler_profile" >&2
    exit 2
    ;;
esac

if [[ -z "${PHP_SRC_PHPT:-}" ]]; then
  printf 'PHP_SRC_PHPT must name a dedicated clean php-src checkout for a full campaign\n' >&2
  exit 2
fi
php_src="$PHP_SRC_PHPT"
if [[ ! -d "$php_src" || ! -f "$php_src/run-tests.php" ]]; then
  printf 'PHP_SRC_PHPT does not name a php-src checkout with run-tests.php: %s\n' "$php_src" >&2
  exit 2
fi
corpus_head="$(git -C "$php_src" rev-parse HEAD 2>/dev/null || true)"
if [[ "$corpus_head" != "$expected_corpus_revision" ]]; then
  printf 'PHP_SRC_PHPT revision differs from the expected corpus revision: actual=%s expected=%s\n' \
    "${corpus_head:-unknown}" "$expected_corpus_revision" >&2
  exit 2
fi
if ! git -C "$php_src" diff --quiet --ignore-submodules -- ||
  ! git -C "$php_src" diff --cached --quiet --ignore-submodules -- ||
  [[ -n "$(git -C "$php_src" ls-files --others --exclude-standard --directory | head -n 1)" ]]; then
  printf 'PHP_SRC_PHPT must be clean; use a disposable pinned checkout for the full campaign\n' >&2
  exit 2
fi
actual_corpus_count="$(git -C "$php_src" ls-files -- '*.phpt' | wc -l | tr -d '[:space:]')"
if [[ "$actual_corpus_count" != "$expected_corpus_count" ]]; then
  printf 'PHP_SRC_PHPT PHPT count differs from the expected corpus count: actual=%s expected=%s\n' \
    "$actual_corpus_count" "$expected_corpus_count" >&2
  exit 2
fi

if ! git -C "$repo_root" diff --quiet --ignore-submodules -- src tests tools Cargo.toml Cargo.lock build.rs 2>/dev/null ||
  ! git -C "$repo_root" diff --cached --quiet --ignore-submodules -- src tests tools Cargo.toml Cargo.lock build.rs 2>/dev/null; then
  printf 'compiler source or campaign tools are dirty; commit the intended changes before launching a full campaign\n' >&2
  exit 2
fi

mkdir -p "$run_root/runs"
available_kib="$(df -Pk "$run_root" | awk 'NR == 2 { print $4; exit }')"
if [[ ! "$available_kib" =~ ^[0-9]+$ ]]; then
  printf 'could not determine available disk space for %s\n' "$run_root" >&2
  exit 2
fi
minimum_free_kib=$((10#$minimum_free_gib * 1024 * 1024))
if (( 10#$available_kib < minimum_free_kib )); then
  printf 'insufficient free disk space for full PHPT campaign: available=%sKiB required=%sGiB\n' \
    "$available_kib" "$minimum_free_gib" >&2
  exit 2
fi

if ! mkdir "$run_dir"; then
  printf 'could not create unique campaign run directory: %s\n' "$run_dir" >&2
  exit 2
fi

write_status() {
  local state=$1
  local tmp="$status.tmp.$$"
  {
    printf 'state\t%s\n' "$state"
    printf 'stamp\t%s\n' "$stamp"
    printf 'run_dir\t%s\n' "$run_dir"
    printf 'worktree\t%s\n' "$worktree"
    printf 'shards\t%s\n' "$shard_count"
    printf 'shard_concurrency\t%s\n' "$shard_concurrency"
    printf 'jobs_per_shard\t%s\n' "$jobs_per_shard"
    printf 'test_timeout\t%s\n' "$test_timeout"
    printf 'compiler_profile\t%s\n' "$compiler_profile"
    printf 'minimum_free_gib\t%s\n' "$minimum_free_gib"
    printf 'available_kib\t%s\n' "$available_kib"
    printf 'corpus_path\t%s\n' "$php_src"
    printf 'corpus_revision\t%s\n' "$corpus_head"
    printf 'corpus_count\t%s\n' "$actual_corpus_count"
    printf 'phpc_bin\t%s\n' "$phpc_bin"
    printf 'updated_at_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    if [[ -d "$worktree/.git" || -f "$worktree/.git" ]]; then
      printf 'source_commit\t%s\n' "$(git -C "$worktree" rev-parse HEAD 2>/dev/null || true)"
    fi
  } > "$tmp"
  mv -f "$tmp" "$status"
}

summarize() {
  local selected=0 tests=0 passed=0 failed=0 skipped=0 warned=0 complete=0 running=0 crashed=0
  local shard_status key value last_state
  for shard_status in "$run_dir"/shards/shard-*/status.tsv; do
    [[ -f "$shard_status" ]] || continue
    last_state=
    while IFS=$'\t' read -r key value; do
      case "$key" in
        selected|tests|passed|failed|skipped|warned)
          [[ "$value" =~ ^[0-9]+$ && ${#value} -le 9 ]] || continue
          case "$key" in
            selected) selected=$((selected + 10#$value)) ;;
            tests) tests=$((tests + 10#$value)) ;;
            passed) passed=$((passed + 10#$value)) ;;
            failed) failed=$((failed + 10#$value)) ;;
            skipped) skipped=$((skipped + 10#$value)) ;;
            warned) warned=$((warned + 10#$value)) ;;
          esac
          ;;
        state) last_state=$value ;;
      esac
    done < "$shard_status"
    case "$last_state" in
      passed|failed) complete=$((complete + 1)) ;;
      running) running=$((running + 1)) ;;
      crashed) crashed=$((crashed + 1)) ;;
    esac
  done
  {
    printf 'updated_at_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf 'shards\t%s\n' "$shard_count"
    printf 'completed_shards\t%s\n' "$complete"
    printf 'running_shards\t%s\n' "$running"
    printf 'crashed_shards\t%s\n' "$crashed"
    printf 'selected\t%s\n' "$selected"
    printf 'tests\t%s\n' "$tests"
    printf 'passed\t%s\n' "$passed"
    printf 'failed\t%s\n' "$failed"
    printf 'skipped\t%s\n' "$skipped"
    printf 'warned\t%s\n' "$warned"
  } > "$summary"
}

extract_metrics() {
  local log=$1
  awk '
    /\[ptn-patrol\] tests=/ {
      seen = 1
      for (i = 1; i <= NF; i++) {
        split($i, kv, "=")
        if (kv[1] == "tests" || kv[1] == "passed" || kv[1] == "failed" || kv[1] == "skipped" || kv[1] == "warned") {
          values[kv[1]] = kv[2]
        }
      }
    }
    END {
      required[1] = "tests"; required[2] = "passed"; required[3] = "failed"; required[4] = "skipped"; required[5] = "warned"
      if (!seen) exit 2
      for (i = 1; i <= 5; i++) {
        key = required[i]
        if (!(key in values) || values[key] !~ /^[0-9]+$/) exit 2
      }
      for (i = 1; i <= 5; i++) {
        key = required[i]
        printf "%s\t%s\n", key, values[key]
      }
    }
  ' "$log"
}

run_shard() {
  local index=$1
  local shard_dir="$run_dir/shards/shard-$(printf '%02d' "$index")"
  local manifest="$shard_dir/manifest.txt"
  local log="$shard_dir/run.log"
  local shard_status="$shard_dir/status.tsv"
  local child_pid=
  local log_sha256=
  shard_interrupted() {
    local code=$1
    trap - HUP INT TERM
    set +e
    if [[ -n "${child_pid:-}" ]]; then
      kill -TERM "$child_pid" 2>/dev/null || true
      wait "$child_pid" 2>/dev/null || true
    fi
    {
      printf 'exit\t%s\n' "$code"
      printf 'state\tinterrupted\n'
      printf 'finished_at_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    } >> "$shard_status"
    exit "$code"
  }
  mkdir -p "$shard_dir"
  {
    printf 'state\trunning\n'
    printf 'started_at_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf 'shard\t%s\n' "$index"
    printf 'log\t%s\n' "$log"
  } > "$shard_status"

  trap 'shard_interrupted 129' HUP
  trap 'shard_interrupted 130' INT
  trap 'shard_interrupted 143' TERM

  set +e
  (
    cd "$worktree" || exit 97
    tools/phpt-full-corpus-shard.sh --shard-index "$index" --shard-count "$shard_count" --out "$manifest"
    selected="$(grep -Ev '^[[:space:]]*(#|$)' "$manifest" | wc -l | tr -d '[:space:]')"
    printf 'selected\t%s\n' "$selected" >> "$shard_status"
    PHPT_PROGRESS_DIR="$shard_dir/progress" \
      PHP_SRC_PHPT="$php_src" \
      PTN_PHPT_AUTO_FETCH=0 \
      PTN_PHPT_CLASSIFY=0 \
      PTN_PHPT_JOBS="$jobs_per_shard" \
      PTN_PHPT_TEST_TIMEOUT="$test_timeout" \
      PTN_PHPT_STRICT_ALL_PASS=1 \
      PHPC_BIN="$phpc_bin" \
      tools/run-phpt-manifest.sh "$manifest"
  ) > "$log" 2>&1 &
  child_pid=$!
  wait "$child_pid"
  local code=$?
  child_pid=
  local metrics
  metrics="$(extract_metrics "$log")"
  local metrics_code=$?
  log_sha256="$(sha256sum "$log" 2>/dev/null | awk '{print $1}')"
  local digest_code=$?
  set -e
  if [[ "$digest_code" -ne 0 || ! "$log_sha256" =~ ^[0-9a-f]{64}$ ]]; then
    log_sha256=
    if [[ "$code" -eq 0 ]]; then
      code=2
    fi
  fi
  trap - HUP INT TERM
  if [[ "$metrics_code" -ne 0 ]]; then
    metrics=$'tests\t0\npassed\t0\nfailed\t0\nskipped\t0\nwarned\t0'
    if [[ "$code" -eq 0 ]]; then
      code=2
    fi
  fi
  {
    printf 'exit\t%s\n' "$code"
    printf 'state\t%s\n' "$([[ "$code" -eq 0 ]] && echo passed || echo failed)"
    printf 'finished_at_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf 'log_sha256\t%s\n' "$log_sha256"
    printf '%s\n' "$metrics"
  } >> "$shard_status"
  return "$code"
}

shard_pids=()
campaign_interrupted() {
  local code=$1
  trap - HUP INT TERM
  set +e
  for pid in "${shard_pids[@]}"; do
    kill -TERM "$pid" 2>/dev/null || true
  done
  for pid in "${shard_pids[@]}"; do
    wait "$pid" 2>/dev/null || true
  done
  summarize
  write_status interrupted
  exit "$code"
}
trap 'campaign_interrupted 129' HUP
trap 'campaign_interrupted 130' INT
trap 'campaign_interrupted 143' TERM

write_status preparing
{
  printf '===== full corpus crash probe %s =====\n' "$stamp"
  printf 'repo=%s\nrun_dir=%s\ncompiler_profile=%s\nphpc_bin=%s\n' \
    "$repo_root" "$run_dir" "$compiler_profile" "$phpc_bin"
} > "$launcher_log"

if ! git -C "$repo_root" worktree add --detach "$worktree" HEAD >> "$launcher_log" 2>&1; then
  write_status failed
  exit 1
fi
write_status building
if ! (
  cd "$worktree"
  mkdir -p .runtime
  cargo "${cargo_build_args[@]}"
) >> "$launcher_log" 2>&1; then
  write_status failed
  exit 1
fi

write_status running
mkdir -p "$run_dir/shards"
wait_code=0
wait_for_shard() {
  local completed_pid=
  local remaining=()
  local found=0
  if ! wait -n -p completed_pid "${shard_pids[@]}"; then
    wait_code=1
  fi
  for pid in "${shard_pids[@]}"; do
    if [[ "$pid" == "$completed_pid" ]]; then
      found=1
    else
      remaining+=("$pid")
    fi
  done
  if [[ "$found" -ne 1 ]]; then
    printf 'could not identify completed shard process: %s\n' "${completed_pid:-unknown}" >> "$launcher_log"
    wait_code=1
    return 1
  fi
  shard_pids=("${remaining[@]}")
  summarize
}

for ((i = 0; i < shard_count; i++)); do
  while [[ "${#shard_pids[@]}" -ge "$shard_concurrency" ]]; do
    wait_for_shard || true
  done
  run_shard "$i" &
  shard_pids+=("$!")
done

while [[ "${#shard_pids[@]}" -gt 0 ]]; do
  wait_for_shard || true
done
summarize
write_status "$([[ "$wait_code" -eq 0 ]] && echo finished || echo failed)"
exit "$wait_code"
