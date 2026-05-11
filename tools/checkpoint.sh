#!/usr/bin/env sh
set -eu

die() {
  printf '%s\n' "$*" >&2
  exit 1
}

repo_root=$(git rev-parse --show-toplevel 2>/dev/null) || die "checkpoint: not inside a git repository"
cd "$repo_root"

test_script="$repo_root/tools/run-tests.sh"
[ -x "$test_script" ] || die "checkpoint: missing executable tools/run-tests.sh"

if [ "$#" -gt 0 ]; then
  message=$*
elif [ -n "${CHECKPOINT_MESSAGE:-}" ]; then
  message=$CHECKPOINT_MESSAGE
else
  message="checkpoint: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
fi

printf '%s\n' "checkpoint: running full project test suite"
if ! "$test_script"; then
  die "checkpoint: tests failed; refusing to commit"
fi

if git diff --quiet --ignore-submodules -- &&
  git diff --cached --quiet --ignore-submodules -- &&
  [ -z "$(git ls-files --others --exclude-standard)" ]; then
  printf '%s\n' "checkpoint: tree clean; no commit created"
  exit 0
fi

git add -A

if git diff --cached --quiet --exit-code; then
  printf '%s\n' "checkpoint: no staged changes after git add -A; no commit created"
  exit 0
fi

printf 'checkpoint: committing current changes: %s\n' "$message"
git commit -m "$message"
