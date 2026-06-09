#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"

if git diff --name-only --diff-filter=U | grep -q .; then
    echo "checkpoint: unresolved conflicts; refusing to commit" >&2
    git diff --name-only --diff-filter=U >&2
    exit 1
fi

cargo fmt --all -- --check
cargo test
git diff --check

if [ -z "$(git status --porcelain=v1)" ]; then
    echo "checkpoint: no changes to commit"
    exit 0
fi

git add -A
git commit -m "${CHECKPOINT_MESSAGE:-checkpoint: integrated progress}"
