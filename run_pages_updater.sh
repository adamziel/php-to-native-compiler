#!/usr/bin/env bash
set -u

umask 022

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STATE_DIR="${PAGES_UPDATER_STATE_DIR:-$ROOT/.pages-updater}"
LOG_DIR="$STATE_DIR/logs"
TMP_DIR="$STATE_DIR/tmp"
SOURCE_REPO="${SOURCE_REPO:-/home/claude/php-to-native-compiler}"
LANE_ROOT="${LANE_ROOT:-/home/claude}"
INTERVAL_SECONDS="${INTERVAL_SECONDS:-900}"
REMOTE="${PAGES_REMOTE:-origin}"
BRANCH="${PAGES_BRANCH:-gh-pages}"
FINGERPRINT_VERSION="20260522T0155"

mkdir -p "$LOG_DIR" "$TMP_DIR"

exec 9>"$STATE_DIR/updater.lock"
if ! flock -n 9; then
  echo "pages updater is already running"
  exit 1
fi

echo "$$" > "$STATE_DIR/updater.pid"

timestamp() {
  date -u +"%Y-%m-%dT%H:%M:%SZ"
}

new_log_path() {
  printf "%s/update-%s.log" "$LOG_DIR" "$(date -u +"%Y%m%dT%H%M%SZ")"
}

log_line() {
  printf "[%s] %s\n" "$(timestamp)" "$*" | tee -a "$CURRENT_LOG"
}

run_cmd() {
  log_line "+ $*"
  "$@" >>"$CURRENT_LOG" 2>&1
}

compute_fingerprint() {
  local input="$TMP_DIR/fingerprint-input.txt"
  : >"$input"
  printf "fingerprint-version=%s\n" "$FINGERPRINT_VERSION" >>"$input"
  printf "source-repo=%s\n" "$SOURCE_REPO" >>"$input"
  printf "lane-root=%s\n" "$LANE_ROOT" >>"$input"

  git -C "$SOURCE_REPO" rev-parse HEAD >>"$input" 2>/dev/null || true
  git -C "$SOURCE_REPO" status --short -- PROGRESS.md docs/PROGRESS.md >>"$input" 2>/dev/null || true

  local rel file lane_dir
  for rel in \
    PROGRESS.md \
    README.md \
    docs/ARCHITECTURE.md \
    docs/COW_COVERAGE_MATRIX.md \
    docs/NEXT_TASKS.md \
    docs/ROADMAP.md \
    docs/SUPPORT.md
  do
    file="$SOURCE_REPO/$rel"
    if [ -f "$file" ]; then
      printf "\n--- source:%s ---\n" "$rel" >>"$input"
      sha256sum "$file" >>"$input"
    fi
  done

  while IFS= read -r file; do
    case "$file" in
      "$LANE_ROOT"/phpc-lane-*/docs/PROGRESS.md)
        lane_dir="$(dirname "$(dirname "$file")")"
        printf "\n--- lane:%s ---\n" "$file" >>"$input"
        sha256sum "$file" >>"$input"
        git -C "$lane_dir" rev-parse HEAD >>"$input" 2>/dev/null || true
        git -C "$lane_dir" branch --show-current >>"$input" 2>/dev/null || true
        ;;
    esac
  done < <(find "$LANE_ROOT" -maxdepth 3 -type f -path "*/docs/PROGRESS.md" 2>/dev/null | sort)

  sha256sum "$input" | awk '{ print $1 }'
}

validate_site() {
  run_cmd node --check tools/build-site.mjs || return 1
  run_cmd node --check assets/site.js || return 1
  log_line "+ internal link check"
  node - <<'NODE' >>"$CURRENT_LOG" 2>&1
const fs = require('fs');
const path = require('path');
const root = process.cwd();
const files = [];
function walk(dir) {
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    if (ent.name === '.git') continue;
    const p = path.join(dir, ent.name);
    if (ent.isDirectory()) walk(p);
    else if (p.endsWith('.html')) files.push(p);
  }
}
walk(root);
const missing = [];
for (const file of files) {
  const html = fs.readFileSync(file, 'utf8');
  const re = /href="([^"]+)"/g;
  let m;
  while ((m = re.exec(html))) {
    const href = m[1];
    if (/^(https?:|mailto:|#)/.test(href)) continue;
    const target = href.split('#')[0];
    if (!target) continue;
    const resolved = path.resolve(path.dirname(file), target);
    if (!fs.existsSync(resolved)) {
      missing.push(`${path.relative(root, file)} -> ${href}`);
    }
  }
}
console.log(`checked ${files.length} html files`);
if (missing.length) {
  console.error(missing.join('\n'));
  process.exit(1);
}
NODE
  return $?
}

run_once() {
  CURRENT_LOG="$(new_log_path)"
  : >"$CURRENT_LOG"
  echo "running" >"$STATE_DIR/last-status"

  cd "$ROOT" || return 1
  log_line "starting pages update cycle"
  log_line "source repo: $SOURCE_REPO"
  log_line "lane root: $LANE_ROOT"

  if [ "$(git branch --show-current)" != "$BRANCH" ]; then
    log_line "wrong branch: expected $BRANCH"
    git status --short --branch >>"$CURRENT_LOG" 2>&1 || true
    return 1
  fi

  if ! git diff --quiet || ! git diff --cached --quiet; then
    log_line "pages checkout has local changes; refusing to overwrite them"
    git status --short >>"$CURRENT_LOG" 2>&1 || true
    return 1
  fi

  find .git/objects -type d -exec chmod 755 {} + 2>/dev/null || true
  run_cmd git fetch "$REMOTE" "$BRANCH" || return 1
  run_cmd git merge --ff-only "$REMOTE/$BRANCH" || return 1

  local current_fingerprint previous_fingerprint
  current_fingerprint="$(compute_fingerprint)"
  previous_fingerprint="$(cat "$STATE_DIR/last-fingerprint" 2>/dev/null || true)"

  if [ "$current_fingerprint" = "$previous_fingerprint" ]; then
    log_line "source fingerprint unchanged: $current_fingerprint"
    echo "idle $(timestamp) $current_fingerprint" >"$STATE_DIR/last-status"
    return 0
  fi

  log_line "source fingerprint changed: ${previous_fingerprint:-none} -> $current_fingerprint"
  run_cmd env SOURCE_REPO="$SOURCE_REPO" LANE_ROOT="$LANE_ROOT" node tools/build-site.mjs || return 1
  printf "%s\n" "$current_fingerprint" >"$STATE_DIR/last-fingerprint"
  validate_site || return 1

  find .git/objects -type d -exec chmod 755 {} + 2>/dev/null || true
  run_cmd git add . || return 1

  if git diff --cached --quiet; then
    log_line "no staged changes after generation"
    echo "no-change $(timestamp) $current_fingerprint" >"$STATE_DIR/last-status"
    return 0
  fi

  run_cmd git commit -m "Refresh progress report pages" -m "Generated at $(timestamp). Source fingerprint: $current_fingerprint" || return 1

  if ! git push "$REMOTE" "$BRANCH" >>"$CURRENT_LOG" 2>&1; then
    log_line "push failed; attempting rebase and retry"
    run_cmd git pull --rebase "$REMOTE" "$BRANCH" || return 1
    run_cmd git push "$REMOTE" "$BRANCH" || return 1
  fi

  if command -v gh >/dev/null 2>&1; then
    gh api repos/adamziel/php-to-native-compiler/pages/builds/latest --jq '{status: .status, commit: .commit, error: .error.message}' >>"$CURRENT_LOG" 2>&1 || true
  fi

  echo "pushed $(timestamp) $current_fingerprint" >"$STATE_DIR/last-status"
  log_line "cycle completed and pushed"
}

if [ "${PAGES_UPDATER_RUN_ONCE:-0}" = "1" ]; then
  run_once
  exit $?
fi

while true; do
  if ! run_once; then
    echo "failed $(timestamp)" >"$STATE_DIR/last-status"
    log_line "cycle failed; retrying after ${INTERVAL_SECONDS}s"
  fi
  sleep "$INTERVAL_SECONDS"
done
