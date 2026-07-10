#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
script="$repo_root/tools/run-phpt-oracle-campaign.sh"
ledger_tool="$repo_root/tools/phpt-oracle-ledger.py"
detached_tool="$repo_root/tools/run-detached-check.sh"

default_revision=8c63ec400ce8e07c57a8d9499317b96a8beafb8b
default_count=21867
default_inventory_sha256=0950cac307e46b1f14fa0251688bb3690391db9b0e270837bd006f645f645137
default_corpus=/home/claude/ptn-oracle/php-src-8c63ec
default_php=/home/claude/ptn-oracle/php-cli-8c63ec/bin/php

usage() {
  cat <<'EOF'
Usage:
  tools/run-phpt-oracle-campaign.sh [OPTIONS]
  tools/run-phpt-oracle-campaign.sh --resume RUN_DIR [--foreground]

Prepare and run one target-PHP oracle campaign using one official php-src
run-tests.php parent process. The default launch is detached through
run-detached-check.sh. No compiler strict-run behavior is changed.

Options:
  --corpus DIR                 php-src checkout
  --php FILE                   target PHP CLI used as harness and test binary
  --out-root DIR               campaign root (default: .runtime/phpt-oracle)
  --jobs N                     run-tests.php workers (default: 8)
  --timeout N                  per-test timeout seconds (default: 300)
  --expected-revision REV      exact corpus Git revision
  --expected-count N           exact PHPT inventory count
  --expected-inventory-sha SHA exact sorted inventory SHA-256
  --prepare-only               validate and prepare artifacts without running
  --foreground                 run in the current shell instead of tmux
  --resume RUN_DIR             start a new full attempt in an existing campaign
  -h, --help                   show this help

Each attempt runs in a disposable detached Git worktree, never in the canonical
corpus checkout. It retains raw run-tests output, official -W results, a
normalized ledger, hashes, and unresolved rows. A resumed attempt reruns the
exact full inventory; it never merges partial evidence.
EOF
}

die() {
  printf 'run-phpt-oracle-campaign: %s\n' "$1" >&2
  exit "${2:-2}"
}

is_positive_integer() {
  [[ "$1" =~ ^[1-9][0-9]*$ ]]
}

is_sha256() {
  [[ "$1" =~ ^[0-9a-f]{64}$ ]]
}

is_revision() {
  [[ "$1" =~ ^[0-9a-f]{40}([0-9a-f]{24})?$ ]]
}

assert_tsv_scalar() {
  local name=$1 value=$2
  [[ -n "$value" && "$value" != *$'\t'* && "$value" != *$'\r'* && "$value" != *$'\n'* ]] ||
    die "$name contains an empty or unsafe metadata value"
}

absolute_dir() {
  (cd "$1" && pwd -P)
}

absolute_file() {
  local directory base
  directory=$(dirname "$1")
  base=$(basename "$1")
  printf '%s/%s\n' "$(cd "$directory" && pwd -P)" "$base"
}

sha256_file() {
  sha256sum "$1" | awk '{print $1}'
}

utc_now() {
  date -u +%Y-%m-%dT%H:%M:%SZ
}

metadata_value() {
  local file=$1 key=$2
  [[ -f "$file" && ! -L "$file" ]] || die "metadata is not a regular file: $file"
  [[ $(wc -c < "$file") -le 65536 ]] || die "metadata is oversized: $file"
  local values=()
  mapfile -t values < <(awk -F '\t' -v key="$key" 'NF == 2 && $1 == key { print $2 }' "$file")
  [[ ${#values[@]} -eq 1 ]] || die "metadata key must occur exactly once: $key"
  assert_tsv_scalar "metadata $key" "${values[0]}"
  printf '%s\n' "${values[0]}"
}

status_value() {
  local file=$1 key=$2
  [[ -f "$file" && ! -L "$file" ]] || die "status is not a regular file: $file"
  [[ $(wc -c < "$file") -le 65536 ]] || die "status is oversized: $file"
  awk -F '\t' -v key="$key" '
    NF == 2 && $1 == key { value=$2; found++ }
    END { if (found == 1) print value; else exit 1 }
  ' "$file" ||
    die "status key must occur exactly once: $key"
}

append_event() {
  local run_dir=$1 event=$2 attempt=$3 detail=$4
  detail=${detail//$'\t'/ }
  detail=${detail//$'\r'/ }
  detail=${detail//$'\n'/ }
  printf '%s\t%s\t%s\t%s\n' "$(utc_now)" "$event" "$attempt" "$detail" >> "$run_dir/events.tsv"
}

write_campaign_status() {
  local run_dir=$1 state=$2 outcome=$3 attempt=$4 run_tests_exit=$5 ledger_state=$6 attempt_dir=$7
  local metadata="$run_dir/metadata.tsv" temporary="$run_dir/.status.tsv.$$"
  {
    printf 'schema\t1\n'
    printf 'state\t%s\n' "$state"
    printf 'outcome\t%s\n' "$outcome"
    printf 'attempt\t%s\n' "$attempt"
    printf 'updated_at_utc\t%s\n' "$(utc_now)"
    printf 'corpus_revision\t%s\n' "$(metadata_value "$metadata" corpus_revision)"
    printf 'inventory_count\t%s\n' "$(metadata_value "$metadata" inventory_count)"
    printf 'inventory_sha256\t%s\n' "$(metadata_value "$metadata" inventory_sha256)"
    printf 'run_tests_exit\t%s\n' "$run_tests_exit"
    printf 'ledger_state\t%s\n' "$ledger_state"
    printf 'attempt_dir\t%s\n' "$attempt_dir"
  } > "$temporary"
  mv -f "$temporary" "$run_dir/status.tsv"
}

validate_prepared_campaign() {
  local run_dir=$1 metadata="$1/metadata.tsv" inventory="$1/inventory.txt"
  [[ -d "$run_dir" && ! -L "$run_dir" ]] || die "campaign directory is invalid: $run_dir"
  [[ -f "$inventory" && ! -L "$inventory" ]] || die "campaign inventory is invalid: $inventory"

  local corpus php revision count inventory_sha actual_revision actual_count actual_sha
  corpus=$(metadata_value "$metadata" corpus_root)
  php=$(metadata_value "$metadata" php_binary)
  revision=$(metadata_value "$metadata" corpus_revision)
  count=$(metadata_value "$metadata" inventory_count)
  inventory_sha=$(metadata_value "$metadata" inventory_sha256)
  [[ -d "$corpus" && -f "$corpus/run-tests.php" ]] || die "prepared corpus is unavailable"
  [[ -x "$php" && -f "$php" ]] || die "prepared PHP binary is unavailable"
  is_revision "$revision" || die "prepared corpus revision is malformed"
  is_positive_integer "$count" || die "prepared inventory count is malformed"
  is_sha256 "$inventory_sha" || die "prepared inventory SHA-256 is malformed"

  actual_revision=$(git -C "$corpus" rev-parse HEAD)
  [[ "$actual_revision" == "$revision" ]] || die "prepared corpus revision changed"
  if [[ -n "$(git -C "$corpus" status --porcelain=v1 --untracked-files=all \
    --ignore-submodules=none | sed -n '1p')" ]]; then
    die "prepared corpus worktree is not clean"
  fi
  actual_count=$(wc -l < "$inventory")
  actual_count=${actual_count//[[:space:]]/}
  actual_sha=$(sha256_file "$inventory")
  [[ "$actual_count" == "$count" ]] || die "prepared inventory count changed"
  [[ "$actual_sha" == "$inventory_sha" ]] || die "prepared inventory SHA-256 changed"
  [[ "$(sha256_file "$php")" == "$(metadata_value "$metadata" php_binary_sha256)" ]] ||
    die "prepared PHP binary changed"
  [[ "$(sha256_file "$corpus/run-tests.php")" == "$(metadata_value "$metadata" run_tests_sha256)" ]] ||
    die "prepared run-tests.php changed"
  [[ -f "$run_dir/php-n-modules.txt" && ! -L "$run_dir/php-n-modules.txt" ]] ||
    die "prepared PHP module inventory is missing"
  [[ -f "$run_dir/php-ini.txt" && ! -L "$run_dir/php-ini.txt" ]] ||
    die "prepared PHP INI summary is missing"
  [[ "$(sha256_file "$run_dir/php-n-modules.txt")" == "$(metadata_value "$metadata" php_n_modules_sha256)" ]] ||
    die "prepared PHP module inventory changed"
  [[ "$(sha256_file "$run_dir/php-ini.txt")" == "$(metadata_value "$metadata" php_ini_sha256)" ]] ||
    die "prepared PHP INI summary changed"
}

prepare_campaign() {
  local corpus=$1 php=$2 out_root=$3 jobs=$4 timeout=$5 revision=$6 expected_count=$7 expected_sha=$8
  [[ -d "$corpus" ]] || die "corpus directory does not exist: $corpus"
  corpus=$(absolute_dir "$corpus")
  [[ -f "$corpus/run-tests.php" ]] || die "run-tests.php is missing from corpus"
  [[ -x "$php" && -f "$php" ]] || die "target PHP is not executable: $php"
  php=$(absolute_file "$php")
  is_positive_integer "$jobs" || die "--jobs must be a positive integer"
  is_positive_integer "$timeout" || die "--timeout must be a positive integer"
  [[ "$jobs" -le 256 ]] || die "--jobs must not exceed 256"
  [[ "$timeout" -le 86400 ]] || die "--timeout must not exceed 86400 seconds"
  is_revision "$revision" || die "--expected-revision must be a full hexadecimal revision"
  is_positive_integer "$expected_count" || die "--expected-count must be a positive integer"
  [[ "$expected_count" -le 100000 ]] || die "--expected-count must not exceed 100000"
  is_sha256 "$expected_sha" || die "--expected-inventory-sha must be a lowercase SHA-256"

  local actual_revision dirty
  actual_revision=$(git -C "$corpus" rev-parse HEAD)
  [[ "$actual_revision" == "$revision" ]] ||
    die "corpus revision mismatch: expected $revision, found $actual_revision"
  dirty=$(git -C "$corpus" status --porcelain=v1 --untracked-files=all \
    --ignore-submodules=none | sed -n '1p')
  [[ -z "$dirty" ]] || die "corpus worktree is not clean"

  mkdir -p "$out_root"
  out_root=$(absolute_dir "$out_root")
  local stamp run_dir inventory count inventory_sha php_version
  stamp="$(date -u +%Y%m%dT%H%M%SZ)-$$"
  run_dir="$out_root/oracle-$stamp"
  mkdir -p "$run_dir/attempts"
  inventory="$run_dir/inventory.txt"
  (
    cd "$corpus"
    find . -path './.git' -prune -o -type f -name '*.phpt' -print |
      sed 's#^\./##' |
      LC_ALL=C sort > "$inventory"
  )
  count=$(wc -l < "$inventory")
  count=${count//[[:space:]]/}
  inventory_sha=$(sha256_file "$inventory")
  [[ "$count" == "$expected_count" ]] ||
    die "inventory count mismatch: expected $expected_count, found $count"
  [[ "$inventory_sha" == "$expected_sha" ]] || die "inventory SHA-256 mismatch"
  php_version=$("$php" -n -r 'printf("%s|%s", PHP_VERSION, PHP_SAPI);')
  "$php" -n -m > "$run_dir/php-n-modules.txt" 2>&1
  "$php" --ini > "$run_dir/php-ini.txt" 2>&1
  [[ $(wc -c < "$run_dir/php-n-modules.txt") -le 1048576 ]] ||
    die "target PHP module inventory is unexpectedly large"
  [[ $(wc -c < "$run_dir/php-ini.txt") -le 1048576 ]] ||
    die "target PHP INI summary is unexpectedly large"

  assert_tsv_scalar corpus "$corpus"
  assert_tsv_scalar php "$php"
  assert_tsv_scalar php_version "$php_version"
  {
    printf 'schema\t1\n'
    printf 'campaign_kind\ttarget-php-oracle\n'
    printf 'created_at_utc\t%s\n' "$(utc_now)"
    printf 'corpus_root\t%s\n' "$corpus"
    printf 'corpus_revision\t%s\n' "$actual_revision"
    printf 'inventory_count\t%s\n' "$count"
    printf 'inventory_sha256\t%s\n' "$inventory_sha"
    printf 'php_binary\t%s\n' "$php"
    printf 'php_binary_sha256\t%s\n' "$(sha256_file "$php")"
    printf 'php_version_sapi\t%s\n' "$php_version"
    printf 'php_n_modules_sha256\t%s\n' "$(sha256_file "$run_dir/php-n-modules.txt")"
    printf 'php_ini_sha256\t%s\n' "$(sha256_file "$run_dir/php-ini.txt")"
    printf 'run_tests_sha256\t%s\n' "$(sha256_file "$corpus/run-tests.php")"
    printf 'jobs\t%s\n' "$jobs"
    printf 'timeout_seconds\t%s\n' "$timeout"
  } > "$run_dir/metadata.tsv"
  printf 'timestamp_utc\tevent\tattempt\tdetail\n' > "$run_dir/events.tsv"
  append_event "$run_dir" prepared 0 "inventory validated"
  write_campaign_status "$run_dir" prepared not_started 0 - not_created -
  printf '%s\n' "$run_dir"
}

write_attempt_status() {
  local attempt_dir=$1 state=$2 outcome=$3 attempt=$4 started=$5 finished=$6 run_exit=$7 parser_exit=$8
  local temporary="$attempt_dir/.status.tsv.$$"
  {
    printf 'schema\t1\n'
    printf 'state\t%s\n' "$state"
    printf 'outcome\t%s\n' "$outcome"
    printf 'attempt\t%s\n' "$attempt"
    printf 'started_at_utc\t%s\n' "$started"
    printf 'finished_at_utc\t%s\n' "$finished"
    printf 'run_tests_exit\t%s\n' "$run_exit"
    printf 'ledger_exit\t%s\n' "$parser_exit"
    if [[ -d "$attempt_dir/corpus" ]]; then
      printf 'worktree_revision\t%s\n' \
        "$(git -C "$attempt_dir/corpus" rev-parse HEAD 2>/dev/null || printf unavailable)"
    fi
    for artifact in worktree-create.log worktree-inventory.txt worktree-status.txt \
      run-tests.log run-tests-results.tsv ledger.tsv summary.tsv unresolved.tsv; do
      if [[ -f "$attempt_dir/$artifact" ]]; then
        printf '%s_sha256\t%s\n' "${artifact//[^A-Za-z0-9]/_}" "$(sha256_file "$attempt_dir/$artifact")"
      fi
    done
  } > "$temporary"
  mv -f "$temporary" "$attempt_dir/status.tsv"
}

execute_campaign() {
  local run_dir=$1
  run_dir=$(absolute_dir "$run_dir")
  validate_prepared_campaign "$run_dir"
  command -v flock >/dev/null 2>&1 || die "flock is required"
  command -v setsid >/dev/null 2>&1 || die "setsid is required"
  exec 9> "$run_dir/worker.lock"
  flock -n 9 || die "another campaign worker holds the run lock" 1

  local metadata="$run_dir/metadata.tsv" status="$run_dir/status.tsv"
  local prior_state prior_attempt
  prior_state=$(status_value "$status" state)
  prior_attempt=$(status_value "$status" attempt)
  case "$prior_state" in
    prepared|failed|interrupted) ;;
    running) append_event "$run_dir" recovered_stale_running "$prior_attempt" "worker lock was free" ;;
    finished) die "campaign already has a complete attempt" 1 ;;
    *) die "unsupported campaign state: $prior_state" ;;
  esac
  [[ "$prior_attempt" =~ ^[0-9]+$ ]] || die "campaign attempt counter is malformed"

  local attempt=$((prior_attempt + 1)) attempt_name attempt_dir
  printf -v attempt_name '%04d' "$attempt"
  attempt_dir="$run_dir/attempts/$attempt_name"
  mkdir "$attempt_dir"
  local started
  started=$(utc_now)
  write_attempt_status "$attempt_dir" running in_progress "$attempt" "$started" - - -
  write_campaign_status "$run_dir" running in_progress "$attempt" - not_created "attempts/$attempt_name"
  append_event "$run_dir" started "$attempt" "one run-tests.php parent"

  local corpus work_corpus php jobs timeout inventory raw log run_pid= signal_name= signal_code=
  corpus=$(metadata_value "$metadata" corpus_root)
  work_corpus="$attempt_dir/corpus"
  php=$(metadata_value "$metadata" php_binary)
  jobs=$(metadata_value "$metadata" jobs)
  timeout=$(metadata_value "$metadata" timeout_seconds)
  is_positive_integer "$jobs" && [[ "$jobs" -le 256 ]] || die "prepared jobs value is invalid"
  is_positive_integer "$timeout" && [[ "$timeout" -le 86400 ]] || die "prepared timeout value is invalid"
  inventory="$run_dir/inventory.txt"
  raw="$attempt_dir/run-tests-results.tsv"
  log="$attempt_dir/run-tests.log"
  : > "$raw"
  : > "$log"

  local worktree_exit=0 setup_error=
  set +e
  git -C "$corpus" worktree add --detach "$work_corpus" \
    "$(metadata_value "$metadata" corpus_revision)" > "$attempt_dir/worktree-create.log" 2>&1
  worktree_exit=$?
  set -e
  if [[ "$worktree_exit" -ne 0 ]]; then
    setup_error=worktree_creation_failed
  else
    (
      cd "$work_corpus"
      find . -path './.git' -prune -o -type f -name '*.phpt' -print |
        sed 's#^\./##' |
        LC_ALL=C sort > "$attempt_dir/worktree-inventory.txt"
    )
    local worktree_count worktree_sha worktree_revision
    worktree_count=$(wc -l < "$attempt_dir/worktree-inventory.txt")
    worktree_count=${worktree_count//[[:space:]]/}
    worktree_sha=$(sha256_file "$attempt_dir/worktree-inventory.txt")
    worktree_revision=$(git -C "$work_corpus" rev-parse HEAD)
    if [[ "$worktree_revision" != "$(metadata_value "$metadata" corpus_revision)" ]]; then
      setup_error=worktree_revision_mismatch
    elif [[ "$worktree_count" != "$(metadata_value "$metadata" inventory_count)" ]]; then
      setup_error=worktree_inventory_count_mismatch
    elif [[ "$worktree_sha" != "$(metadata_value "$metadata" inventory_sha256)" ]]; then
      setup_error=worktree_inventory_sha256_mismatch
    elif [[ "$(sha256_file "$work_corpus/run-tests.php")" != "$(metadata_value "$metadata" run_tests_sha256)" ]]; then
      setup_error=worktree_run_tests_sha256_mismatch
    elif [[ -n "$(git -C "$work_corpus" status --porcelain=v1 --untracked-files=all \
      --ignore-submodules=none | sed -n '1p')" ]]; then
      setup_error=worktree_not_clean_at_start
    fi
  fi
  if [[ -n "$setup_error" ]]; then
    local setup_finished
    setup_finished=$(utc_now)
    write_attempt_status "$attempt_dir" failed "$setup_error" "$attempt" \
      "$started" "$setup_finished" "$worktree_exit" -
    write_campaign_status "$run_dir" failed "$setup_error" "$attempt" \
      "$worktree_exit" not_created "attempts/$attempt_name"
    append_event "$run_dir" failed "$attempt" "$setup_error"
    printf 'campaign_state=failed\noutcome=%s\nattempt=%s\nrun_dir=%s\nattempt_dir=%s\n' \
      "$setup_error" "$attempt" "$run_dir" "$attempt_dir"
    return 2
  fi
  forward_signal() {
    signal_name=$1
    signal_code=$2
    trap - INT TERM
    if [[ -n "${run_pid:-}" ]]; then
      kill -s "$signal_name" -- "-$run_pid" 2>/dev/null || true
    fi
  }
  if [[ "${PTN_DETACHED_CHECK:-0}" == 1 ]]; then
    trap '' HUP
  else
    trap 'forward_signal HUP 129' HUP
  fi
  trap 'forward_signal INT 130' INT
  trap 'forward_signal TERM 143' TERM

  local run_tests_args=(-q "-j$jobs" --set-timeout "$timeout")
  run_tests_args+=(-p "$php" -r "$inventory" -W "$raw")
  set +e
  (
    cd "$work_corpus" || exit 97
    exec setsid env \
      NO_INTERACTION=1 \
      REPORT_EXIT_STATUS=1 \
      TEST_PHP_EXECUTABLE="$php" \
      TEST_PHP_LOG_FORMAT=LEODS \
      "$php" "$work_corpus/run-tests.php" "${run_tests_args[@]}"
  ) > "$log" 2>&1 &
  run_pid=$!
  wait "$run_pid"
  local run_exit=$?
  run_pid=
  set -e
  trap - INT TERM HUP
  if [[ -n "$signal_code" ]]; then
    run_exit=$signal_code
  fi

  set +e
  python3 "$ledger_tool" \
    --inventory "$inventory" \
    --results "$raw" \
    --ledger "$attempt_dir/ledger.tsv" \
    --summary "$attempt_dir/summary.tsv" \
    --unresolved "$attempt_dir/unresolved.tsv" \
    --corpus-root "$work_corpus" \
    --expected-count "$(metadata_value "$metadata" inventory_count)" \
    --expected-sha256 "$(metadata_value "$metadata" inventory_sha256)"
  local ledger_exit=$?
  set -e

  local state outcome ledger_state final_exit
  if [[ -n "$signal_code" ]]; then
    state=interrupted
    outcome="signal_${signal_name}"
    ledger_state=$([[ "$ledger_exit" -eq 0 ]] && printf complete || printf partial)
    final_exit=$signal_code
  elif [[ "$ledger_exit" -eq 0 ]]; then
    state=finished
    ledger_state=complete
    if [[ "$run_exit" -eq 0 ]]; then
      outcome=completed
      final_exit=0
    else
      outcome=test_failures
      final_exit=$run_exit
    fi
  else
    state=failed
    outcome=incomplete_or_invalid_ledger
    ledger_state=$([[ "$ledger_exit" -eq 1 ]] && printf partial || printf invalid)
    final_exit=2
  fi
  local finished
  finished=$(utc_now)
  git -C "$work_corpus" status --porcelain=v1 --untracked-files=all \
    --ignore-submodules=none > "$attempt_dir/worktree-status.txt"
  write_attempt_status "$attempt_dir" "$state" "$outcome" "$attempt" "$started" "$finished" "$run_exit" "$ledger_exit"
  write_campaign_status "$run_dir" "$state" "$outcome" "$attempt" "$run_exit" "$ledger_state" "attempts/$attempt_name"
  append_event "$run_dir" "$state" "$attempt" "$outcome"
  printf 'campaign_state=%s\noutcome=%s\nattempt=%s\nrun_tests_exit=%s\nledger_state=%s\nrun_dir=%s\nattempt_dir=%s\n' \
    "$state" "$outcome" "$attempt" "$run_exit" "$ledger_state" "$run_dir" "$attempt_dir"
  return "$final_exit"
}

launch_campaign() {
  local run_dir=$1 foreground=$2
  validate_prepared_campaign "$run_dir"
  local state
  state=$(status_value "$run_dir/status.tsv" state)
  case "$state" in
    prepared|failed|interrupted) ;;
    running)
      if ! flock -n "$run_dir/worker.lock" true; then
        die "campaign worker is still active" 1
      fi
      ;;
    *) die "campaign cannot be launched from state: $state" 1 ;;
  esac
  if [[ "$foreground" -eq 1 ]]; then
    execute_campaign "$run_dir"
  else
    local name
    name="phpt-oracle-$(basename "$run_dir")"
    PTN_DETACHED_CHECK_ROOT="$run_dir/detached" \
      "$detached_tool" "$name" -- "$script" --execute "$run_dir"
    printf 'campaign_run_dir=%s\ncampaign_status=%s/status.tsv\n' "$run_dir" "$run_dir"
  fi
}

if [[ "${1:-}" == --execute ]]; then
  [[ $# -eq 2 ]] || die "--execute requires exactly one campaign directory"
  execute_campaign "$2"
  exit $?
fi

corpus=$default_corpus
php=$default_php
out_root="$repo_root/.runtime/phpt-oracle"
jobs=${PTN_ORACLE_JOBS:-8}
timeout=${PTN_ORACLE_TIMEOUT:-300}
expected_revision=$default_revision
expected_count=$default_count
expected_inventory_sha=$default_inventory_sha256
prepare_only=0
foreground=0
resume=

while [[ $# -gt 0 ]]; do
  case "$1" in
    --corpus) [[ $# -ge 2 ]] || die "--corpus requires a value"; corpus=$2; shift 2 ;;
    --php) [[ $# -ge 2 ]] || die "--php requires a value"; php=$2; shift 2 ;;
    --out-root) [[ $# -ge 2 ]] || die "--out-root requires a value"; out_root=$2; shift 2 ;;
    --jobs) [[ $# -ge 2 ]] || die "--jobs requires a value"; jobs=$2; shift 2 ;;
    --timeout) [[ $# -ge 2 ]] || die "--timeout requires a value"; timeout=$2; shift 2 ;;
    --expected-revision) [[ $# -ge 2 ]] || die "--expected-revision requires a value"; expected_revision=$2; shift 2 ;;
    --expected-count) [[ $# -ge 2 ]] || die "--expected-count requires a value"; expected_count=$2; shift 2 ;;
    --expected-inventory-sha)
      [[ $# -ge 2 ]] || die "--expected-inventory-sha requires a value"
      expected_inventory_sha=$2
      shift 2
      ;;
    --prepare-only) prepare_only=1; shift ;;
    --foreground) foreground=1; shift ;;
    --resume) [[ $# -ge 2 ]] || die "--resume requires a value"; resume=$2; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) die "unknown option: $1" ;;
  esac
done

if [[ -n "$resume" ]]; then
  [[ "$prepare_only" -eq 0 ]] || die "--prepare-only cannot be combined with --resume"
  run_dir=$(absolute_dir "$resume")
else
  run_dir=$(prepare_campaign "$corpus" "$php" "$out_root" "$jobs" "$timeout" \
    "$expected_revision" "$expected_count" "$expected_inventory_sha")
fi

printf 'campaign_run_dir=%s\ninventory=%s/inventory.txt\nmetadata=%s/metadata.tsv\nstatus=%s/status.tsv\n' \
  "$run_dir" "$run_dir" "$run_dir" "$run_dir"
if [[ "$prepare_only" -eq 0 ]]; then
  launch_campaign "$run_dir" "$foreground"
fi
