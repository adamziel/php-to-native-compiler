# PHPT Binary and Wrapper Availability Recheck

Agent: developer-305

Lane: 122

Scope: read-only artifact availability recheck after restart. No compiler,
runtime, harness source, or php-src files were edited. No release rebuild and no
full PHPT gate were run.

`DEVELOPMENT.md` was requested by the harness prompt but is absent under
`/home/claude/php-to-native-compiler` and this worktree.

## Decision

Focused replay lanes still do not have an authoritative accepted/candidate
`PHPC_BIN` pair from the historical public gate run roots. The durable evidence
directories are present, but both recorded `/tmp/.../cargo-target/release/phpc`
paths are missing.

The shared wrapper and pinned php-src checkout are available and executable:

- wrapper: `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`
- wrapper sha256: `022ab2202ba5c4a25bfaa712f97f795be844634fcc7b7a1bef09e2e3eddf6e29`
- php-src checkout: `/home/claude/php-src-phpt`
- php-src HEAD: `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- `run-tests.php` sha256: `7597d42eda0609ea823467e841bc1eec9b8e11f70de96f0868f66f15d05b5cf5`
- host PHP: `PHP 8.2.29`

Current scratch binaries are not enough to adjudicate accepted-vs-candidate
PHPT replay:

- `/dev/shm/phpc-lane70-replay-dev125/accepted-target/release/phpc` and
  `/dev/shm/phpc-lane70-replay-dev125/candidate-target/release/phpc` are still
  executable and pass a tiny `phpc run` smoke, but their adjacent source trees
  are not Git repositories. Treat them as unpinned scratch binaries only.
- `/tmp/phpt-focused-replay-reflection-dev221/cargo-target-accepted/release/phpc`
  is executable and passes a tiny `phpc run` smoke, but there is no matching
  candidate release binary under that scratch root, and its source directories
  are not Git repositories.
- Several debug `phpc` binaries exist under `/tmp` and `/dev/shm`; they pass a
  tiny smoke but are not release accepted/candidate replay inputs.

## Historical Gate Inputs

Accepted baseline:

- evidence root: `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- run root: `/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- recorded `PHPC_BIN`: `/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc`
- source head: `0b917f67a37d9ca9779d77f87173b628431c2425`
- public score: `7873 / 20294`

Blocked candidate:

- evidence root: `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- run root: `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- recorded `PHPC_BIN`: `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc`
- source head: `56fe9377fb46be00db5fdd30c966fdba406dc581`
- public score: `7197 / 20294`
- latest-public PASS regressions: `1166`

Both source commits exist in the current repository.

## Availability Matrix

| Item | Status | Evidence |
| --- | --- | --- |
| Accepted evidence root | present | Directory exists under durable `state/logs`. |
| Candidate evidence root | present | Directory exists under durable `state/logs`. |
| Accepted `/tmp` run root | missing | `exists=false` for the recorded run root. |
| Candidate `/tmp` run root | missing | `exists=false` for the recorded run root. |
| Accepted historical `PHPC_BIN` | missing | Wrapper exits `127` when pointed at the recorded path. |
| Candidate historical `PHPC_BIN` | missing | `exists=false` for the recorded path. |
| `phpc-phpt-wrapper` | executable | Mode `-rwxrwx---`, size `2896`, sha256 above. |
| `php-src-phpt/run-tests.php` | executable | Mode `-rwxr-xr-x`, size `142935`, sha256 above. |
| Lane70 scratch accepted release binary | executable, unpinned | sha256 `1c01dd3b33613143b014a14c59ad5601989298b0b762c6b54ef6abe2e306b5ad`; smoke `ok`. |
| Lane70 scratch candidate release binary | executable, unpinned | sha256 `9e16883b1fa50fd1baee4006b0871e790cace654afffe82986b6a259de608384`; smoke `ok`. |
| Reflection scratch accepted release binary | executable, incomplete pair | sha256 `9f59cc18945a531679967ed669e8f12fb8f5eb622fdc464bc292ca7c1188feb6`; smoke `ok`; no candidate release pair found. |

## Wrapper Behavior

The wrapper still requires an executable `PHPC_BIN` before it will execute a
PHPT FILE script through `phpc run`. Direct checks showed:

| Check | `PHPC_BIN` | Exit | Output |
| --- | --- | --- | --- |
| Missing historical accepted path | recorded accepted `/tmp/.../release/phpc` | `127` | `phpc-phpt-wrapper: PHPC_BIN is not executable: ...` |
| Scratch lane70 accepted path | `/dev/shm/phpc-lane70-replay-dev125/accepted-target/release/phpc` | `0` | `wrapper-ok` |

This separates wrapper health from binary availability: the wrapper works when
given a real executable, but the historical accepted/candidate replay paths are
gone.

## Focused Replay Lane Impact

Active focused replay/planning lanes `81`, `82`, `83`, `85`, `86`, and `87`
are in progress, and lane `84` is integrated. The existing replay cookbook and
current report artifacts still describe focused accepted-vs-candidate replay as
blocked until accepted and candidate release binaries are restored or rebuilt.
The latest visible lane84 validation row (`test_runs#123`) explicitly records
`focused_replay: unavailable_missing_historical_phpc_bin`.

Current conclusion: no focused replay lane has a verified authoritative
accepted/candidate `PHPC_BIN` pair. Lanes may use the unpinned scratch binaries
only for explicitly caveated local smoke, not for public score movement or
accepted-vs-candidate adjudication.

## Exact Checks Used

Path and hash checks:

```sh
python - <<'PY'
from pathlib import Path
import os, stat, hashlib, json
paths = [
  '/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67',
  '/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc',
  '/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377',
  '/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc',
  '/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper',
  '/home/claude/php-src-phpt',
  '/home/claude/php-src-phpt/run-tests.php',
]
for p in paths:
    q = Path(p)
    print(p, q.exists(), os.access(q, os.X_OK))
PY
```

Scratch binary search:

```sh
find /tmp /dev/shm -maxdepth 6 -type f -name phpc -perm -111 -print
```

Smoke checks:

```sh
python - <<'PY'
import subprocess, tempfile, os
for phpc in [
  '/tmp/phpt-focused-replay-reflection-dev221/cargo-target-accepted/release/phpc',
  '/dev/shm/phpc-lane70-replay-dev125/accepted-target/release/phpc',
  '/dev/shm/phpc-lane70-replay-dev125/candidate-target/release/phpc',
]:
    f = tempfile.NamedTemporaryFile('w', suffix='.php', delete=False)
    f.write('<?php echo "ok\\n";\\n')
    f.close()
    print(phpc, subprocess.run([phpc, 'run', f.name], text=True,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE).returncode)
    os.unlink(f.name)
PY
```

Wrapper direct check:

```sh
PHPC_BIN=/dev/shm/phpc-lane70-replay-dev125/accepted-target/release/phpc \
  /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper /tmp/smoke.php
```

## Next Deterministic Action

Do not run another full PHPT gate to solve this. Either restore the exact
historical `/tmp/phpt-full-current-score-*` run roots, or create a tiny rebuild
lane that builds release binaries from:

- accepted: `0b917f67a37d9ca9779d77f87173b628431c2425`
- candidate: `56fe9377fb46be00db5fdd30c966fdba406dc581`

Persist the rebuilt binaries under a durable path with a manifest recording
source commit, build command, target directory, binary sha256, and one `phpc
run` smoke. Focused replay lanes can then set `PHPC_BIN` to those rebuilt
paths and use the existing wrapper plus `/home/claude/php-src-phpt`.
