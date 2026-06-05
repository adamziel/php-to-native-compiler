# PHPT Binary and Wrapper Availability

Agent: developer-217

Lane: 96, reassigned from the original dev133 artifact name.

Scope: read-only availability audit for focused PHPT replay lanes. No
compiler/runtime source files were edited, no rebuild was run, and no full PHPT
gate was run.

`DEVELOPMENT.md` was requested by the harness prompt but is not present under
`/home/claude/php-to-native-compiler` or this worktree.

## Decision

Focused replay lanes still cannot use the historical accepted/candidate
`PHPC_BIN` values recorded by the public gate artifacts. Both `/tmp` run roots
and both recorded release binaries are missing.

The shared wrapper and pinned php-src checkout are available and executable:

- wrapper: `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`
- php-src checkout: `/home/claude/php-src-phpt`
- php-src HEAD: `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- host PHP: `PHP 8.2.29`

Two executable release `phpc` binaries exist in a superseded lane scratch tree:

- `/dev/shm/phpc-lane70-replay-dev125/accepted-target/release/phpc`
- `/dev/shm/phpc-lane70-replay-dev125/candidate-target/release/phpc`

They run a minimal `phpc run` smoke, but their adjacent copied source trees are
not Git repositories, and lane 70 was superseded by lane 79 before producing a
canonical replay artifact. Treat them as convenient scratch binaries only, not
as authoritative accepted/candidate gate evidence unless a manager/integrator
accepts their provenance or rebuilds pinned binaries from the recorded commits.

## Historical Gate Inputs

Accepted baseline:

- evidence root: `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- run root: `/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- recorded `PHPC_BIN`: `/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc`
- source head: `0b917f67a37d9ca9779d77f87173b628431c2425`
- public score artifact: `7873/20294`

Blocked candidate:

- evidence root: `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- run root: `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- recorded `PHPC_BIN`: `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc`
- source head: `56fe9377fb46be00db5fdd30c966fdba406dc581`
- public score artifact: `7197/20294`
- normalized PASS regressions against the accepted pass baseline: `1166`

Both source commits exist in the source seed and current repository:

```text
0b917f67a37d9ca9779d77f87173b628431c2425: present
56fe9377fb46be00db5fdd30c966fdba406dc581: present
```

## Availability Matrix

| Item | Status | Notes |
| --- | --- | --- |
| Accepted evidence root | present | Durable logs and score artifacts exist. |
| Candidate evidence root | present | Durable logs and score artifacts exist. |
| Accepted `/tmp` run root | missing | `test -e` failed. |
| Candidate `/tmp` run root | missing | `test -e` failed. |
| Accepted historical `PHPC_BIN` | missing | `test -x` failed because the path does not exist. |
| Candidate historical `PHPC_BIN` | missing | `test -x` failed because the path does not exist. |
| `phpc-phpt-wrapper` | executable | Mode `-rwxrwx---`, size `2896`, sha256 `022ab2202ba5c4a25bfaa712f97f795be844634fcc7b7a1bef09e2e3eddf6e29`. |
| `php-src-phpt/run-tests.php` | executable | Mode `-rwxr-xr-x`, size `142935`, sha256 `7597d42eda0609ea823467e841bc1eec9b8e11f70de96f0868f66f15d05b5cf5`. |
| Scratch accepted release binary | executable, unpinned | sha256 `1c01dd3b33613143b014a14c59ad5601989298b0b762c6b54ef6abe2e306b5ad`; smoke `phpc run` printed `ok`. |
| Scratch candidate release binary | executable, unpinned | sha256 `9e16883b1fa50fd1baee4006b0871e790cace654afffe82986b6a259de608384`; smoke `phpc run` printed `ok`. |

## Wrapper Behavior

The wrapper requires an executable `PHPC_BIN` and exits `127` when the path is
not executable:

```sh
PHPC_BIN=${PHPC_BIN:-/home/claude/php-to-native-compiler/target/debug/phpc}

if [ ! -x "$PHPC_BIN" ]; then
  echo "phpc-phpt-wrapper: PHPC_BIN is not executable: $PHPC_BIN" >&2
  exit 127
fi
```

That means focused replay workers must supply a real accepted or candidate
binary before invoking `run-tests.php`; otherwise failures will be wrapper
configuration failures, not PHPT signal.

## Exact Checks Used

Historical path check:

```sh
paths=(
/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67
/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc
/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377
/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc
/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
/home/claude/php-src-phpt/run-tests.php
)
for p in "${paths[@]}"; do
  if [ -e "$p" ]; then
    stat -c '%F %A %s %y %n' "$p"
    if [ -f "$p" ]; then [ -x "$p" ] && echo "EXECUTABLE $p" || echo "NOT_EXECUTABLE $p"; fi
  else
    echo "MISSING $p"
  fi
done
```

Observed historical result:

```text
MISSING /tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67
MISSING /tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc
MISSING /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377
MISSING /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc
EXECUTABLE /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
EXECUTABLE /home/claude/php-src-phpt/run-tests.php
```

Scratch binary search:

```sh
find /tmp /dev/shm -maxdepth 6 -type f -name phpc -perm -111 -print 2>/dev/null | sort
```

Observed relevant release binaries:

```text
/dev/shm/phpc-lane70-replay-dev125/accepted-target/release/phpc
/dev/shm/phpc-lane70-replay-dev125/candidate-target/release/phpc
```

Smoke check:

```sh
for bin in \
  /dev/shm/phpc-lane70-replay-dev125/accepted-target/release/phpc \
  /dev/shm/phpc-lane70-replay-dev125/candidate-target/release/phpc
do
  tmp=$(mktemp /tmp/phpc-lane96-smoke.XXXXXX.php)
  printf '<?php echo "ok\\n";\n' > "$tmp"
  "$bin" run "$tmp"
  rm -f "$tmp"
done
```

Observed:

```text
ok
ok
```

Provenance check:

```sh
git -C /dev/shm/phpc-lane70-replay-dev125/accepted rev-parse HEAD
git -C /dev/shm/phpc-lane70-replay-dev125/candidate rev-parse HEAD
```

Observed:

```text
not_git
not_git
```

## Recommended Replay Contract

For durable focused replay, use only one of these two options:

1. Restore the exact historical `/tmp/phpt-full-current-score-*` run roots with
   their recorded `cargo-target/release/phpc` binaries.
2. Rebuild release binaries from the recorded commits and persist them under a
   durable path with a small manifest containing:
   - source commit
   - `cargo build --release -p phpc` command
   - `CARGO_TARGET_DIR`
   - binary path
   - binary sha256
   - one `phpc run` smoke result

The found `/dev/shm/phpc-lane70-replay-dev125/*/release/phpc` binaries may be
useful for a local, explicitly caveated smoke, but they should not be used to
adjudicate accepted-vs-candidate score movement because their commit provenance
is not verifiable from the scratch tree.

## Next Deterministic Action

Create a tiny rebuild lane, or fold it into the next replay lane, to rebuild:

```sh
SOURCE_REPO_SEED=/home/claude/supervised-php-compiler/state/worktrees/batch024-current-supervisor-20260601T093225
REBUILD_ROOT=/dev/shm/phpc-phpt-replay-binaries-221205Z

git clone --no-local "$SOURCE_REPO_SEED" "$REBUILD_ROOT/accepted-src"
git -C "$REBUILD_ROOT/accepted-src" checkout -f 0b917f67a37d9ca9779d77f87173b628431c2425
CARGO_TARGET_DIR="$REBUILD_ROOT/accepted-target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo build --release -p phpc --manifest-path "$REBUILD_ROOT/accepted-src/Cargo.toml"

git clone --no-local "$SOURCE_REPO_SEED" "$REBUILD_ROOT/candidate-src"
git -C "$REBUILD_ROOT/candidate-src" checkout -f 56fe9377fb46be00db5fdd30c966fdba406dc581
CARGO_TARGET_DIR="$REBUILD_ROOT/candidate-target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo build --release -p phpc --manifest-path "$REBUILD_ROOT/candidate-src/Cargo.toml"
```

After that, replay workers can set:

```sh
export PHPC_BIN="$REBUILD_ROOT/accepted-target/release/phpc"
export TEST_PHP_EXECUTABLE=/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
```

and the analogous candidate path. This remains a focused replay input, not a
full public-score gate.
