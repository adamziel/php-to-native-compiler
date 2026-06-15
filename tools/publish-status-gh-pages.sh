#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

site_dir="${1:-_site}"
remote="${STATUS_PAGES_REMOTE:-origin}"
branch="${STATUS_PAGES_BRANCH:-gh-pages}"
dry_run="${STATUS_PAGES_DRY_RUN:-0}"

if [[ ! -d "$site_dir" ]]; then
  echo "missing site directory: $site_dir" >&2
  exit 1
fi
if [[ ! -f "$site_dir/index.html" ]]; then
  echo "missing site entry point: $site_dir/index.html" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
worktree_dir="$tmp_dir/worktree"
cleanup() {
  git worktree remove --force "$worktree_dir" >/dev/null 2>&1 || true
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

git fetch "$remote" "$branch:refs/remotes/${remote}/${branch}" || true
if git show-ref --verify --quiet "refs/remotes/${remote}/${branch}"; then
  git worktree add --detach "$worktree_dir" "${remote}/${branch}"
else
  git worktree add --detach "$worktree_dir" HEAD
  git -C "$worktree_dir" switch --orphan "$branch"
fi

find "$worktree_dir" -mindepth 1 -maxdepth 1 ! -name .git -exec rm -rf {} +
cp -R "$site_dir"/. "$worktree_dir"/
touch "$worktree_dir/.nojekyll"

git -C "$worktree_dir" add -A
if git -C "$worktree_dir" diff --cached --quiet; then
  echo "STATUS dashboard already current on $branch"
  exit 0
fi

if [[ "$dry_run" == "1" ]]; then
  git -C "$worktree_dir" diff --cached --name-status
  echo "dry run: not publishing $branch"
  exit 0
fi

git -C "$worktree_dir" \
  -c user.name="github-actions[bot]" \
  -c user.email="41898282+github-actions[bot]@users.noreply.github.com" \
  commit -m "Publish STATUS dashboard"
git -C "$worktree_dir" push "$remote" HEAD:"$branch"
