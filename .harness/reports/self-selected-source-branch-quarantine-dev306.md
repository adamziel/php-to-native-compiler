# Self-Selected Source Branch Quarantine Map

Lane: 123  
Agent: developer-306  
Generated: 2026-06-05T09:00Z  
Branch/worktree: `work/developer-306` /
`/home/claude/php-to-native-compiler/.harness/worktrees/developer-306`  
Current HEAD inspected: `7f61915aed0990e52cbaa91b2d7b1a16e3ed8c29`

Scope is read-only/report-only. No compiler, runtime, harness, or PHPT source
files were edited. No merge, checkout of another branch, full PHPT gate, or
public score update was performed. `DEVELOPMENT.md` was requested by role
instructions but is absent from this worktree and from the checked neighboring
worktree roots.

Accepted public score remains `7873 / 20294`. The blocked `221205Z` candidate
remains blocked at `7197 / 20294` with `1166` latest-public PASS regressions.
Nothing below is score-moving M0/M1 progress unless a later Integrator/Manager
explicitly accepts it with the required evidence.

## Decision Summary

Do not integrate the branches/events in the quarantine tables as M0/M1 progress
by branch head. They are either superseded self-selected compatibility work,
duplicate runtime repair work, or source-changing runtime candidates that have
focused unit evidence but have not been safely integrated and have no zero-
regression public PHPT gate.

Report-only artifacts that were already integrated should still be consumed by
exact artifact path/commit. In particular, `work/developer-83` has integrated
M0 report artifacts on earlier commits, but its current branch head is a later
source-changing runtime/docs/test commit and must stay separate.

Eval and variable-variable work remains late-priority. Existing late-row
artifacts are read-only planning tags; no source branch found in this audit
should be treated as accepted eval or variable-variable implementation.

## Quarantine Branches And Events

| Branch / event | Lane(s) | Source surface observed | Quarantine reason | Handling |
| --- | ---: | --- | --- | --- |
| `work/developer-35` at `e147c033` | 7 | Prior dirty audit reported `compiler/src/interpreter.rs`, `compiler/tests/array_reverse.rs`, and `compiler/tests/array_slice.rs` in that lane context. | Self-selected generic PHP compatibility slice; superseded because M0/M1 regression control took priority. | Do not integrate lane 7 output. Any future array/runtime work needs a fresh owner, precheck, focused tests, CLI path, docs, and unsupported edges. |
| `work/developer-36` at `e147c033` | 9 | Branch commit touches `compiler/src/codegen.rs`, `compiler/src/interpreter.rs`, `compiler/tests/string_parse_builtins.rs`, and `docs/SUPPORT.md`. | Self-selected short echo tag work; manager marked it a near-term distraction until blocked PHPT regression classification is complete. | Keep short echo as unsupported unless a later explicit lane reopens it. |
| `work/developer-40` at `e147c033` plus dirty lane state | 10 | Prior dirty audit reported `README.md`, `compiler/src/lexer.rs`, support/architecture/progress docs, deleted unsupported short-echo fixtures, and new short-echo fixtures. | Self-selected compatibility branch superseded by M0/M1 scope; includes doc/support churn and fixture deletion risk. | Do not integrate without explicit reassignment and a new review of unsupported-edge claims. |
| `work/developer-43` at `e147c033` plus dirty lane state | 11 | Prior dirty audit reported `compiler/src/interpreter.rs`, `compiler/tests/string_algorithm_builtins.rs`, and `similar_text` fixture additions. | Self-selected standard-library compatibility slice after active lanes were already claimed. | Hold. It is not evidence for the blocked gate unless replay/classification selects the row and accepts a repair lane. |
| `work/developer-44` at `e147c033` | 12, 25 | `str_ireplace` implementation lane was superseded; lane 25 replaced it with a read-only replay selector. | Implementation work was out of scope after M0/M1 restriction. | Consume only lane 25 report artifact. Do not treat branch source as accepted `str_ireplace` support. |
| `work/developer-37` at `e147c033` plus dirty lane state | 13 | Prior dirty audit reported `README.md`, `compiler/tests/filemtime_builtin.rs`, docs, and filemtime fixtures. | File time metadata evidence/source lane paused until regression classification and control-plane cleanup. | Hold until a future file/stat lane is explicitly opened. |
| `work/developer-85` at `9f943b19` | no active lane row found | `compiler/src/codegen.rs`, `compiler/src/interpreter.rs`, `compiler/tests/string_case_builtin.rs`, docs, and `strtoupper` fixtures. | Source-changing compatibility branch outside the current M0/M1 artifact chain. | Do not integrate as scheduler capacity/progress without explicit Manager/Integrator review. |
| `work/developer-86` at `17e442ab` | no active lane row found | `compiler/src/interpreter.rs`, `runtime/src/lib.rs`, array-count tests, docs, and fixtures. | Source-changing runtime/interpreter branch outside the current accepted gate evidence. | Hold; not public metric progress. |
| `work/developer-88` at `8636ec0d` | no active lane row found | `compiler/src/interpreter.rs`, COW docs, loop memory, and a milestone fixture. | Unsliced runtime/COW source branch outside the current M0/M1 repair plan. | Hold. Future COW work should follow the hard-first prerequisite evidence lane, not this branch head. |
| `work/developer-83` at `2f8aec28` | 40 plus earlier report lanes | `README.md`, `docs/ARCHITECTURE.md`, `docs/NATIVE_RUNTIME_ABI.md`, `docs/PROGRESS.md`, `docs/SUPPORT.md`, `runtime/src/lib.rs`. | Earlier report artifacts from `work/developer-83` are integrated, but current head is a later source-changing runtime ABI/test repair. Integrator-8 explicitly excluded this head; Integrator-28 reverified it but did not merge due dirty overlap. | Consume already integrated report artifacts only. Do not integrate current branch head as report progress or PHPT metric movement. |
| `work/developer-114` at `26527dce` | 63, 76 | Runtime native string ABI test/source expectations; also had a superseded control-plane assignment in lane 76. | Duplicate stale php_runtime repair lane; superseded multiple times. Integrator-28 marked it not integrated because canonical current runtime candidate is lane 66. | Do not integrate. Retain for audit only unless Manager reopens a specific slice. |
| `work/developer-117` at `174370c4` | 65 | `runtime/src/lib.rs` and `compiler/tests/object_model.rs`. | Alternate duplicate runtime repair; prior disposable merge after `work/developer-120` conflicted in `runtime/src/lib.rs`. Integrator-28 marked it not safely merged. | Do not integrate without manual reconciliation against lane 66. |
| `work/developer-120` at `e04e3df9` | 66 | `runtime/src/lib.rs`. | Canonical focused php_runtime candidate passed `php_runtime --lib 419/419`, but is source-changing, not a public PHPT gate result, and was deferred because shared root had dirty `runtime/src/lib.rs`. | Candidate for Integrator handling only; not M0/M1 report progress and not public metric movement. |
| `work/developer-124` at `7a17b7ee` | 61 | `runtime/src/lib.rs` and `docs/PROGRESS.md`. | Runtime test-counter patch with focused checks; remaining package failures were explicitly noted. Integrator-28 did not merge due dirty overlap. | Hold for Integrator after overlap review; not score movement. |
| `work/developer-118` lane event | 67 | Intended php_runtime source repair, but owner became non-live and lane was superseded after lane 66/65 focused evidence. | Stale owner/duplicate runtime source lane. | Do not treat as active or accepted source work. |
| Lane 114 database/runtime slice | 114 | Dedicated report path `.harness/reports/wordpress-database-lane-quarantine-dev233.md`. | Manager explicitly assigned a separate quarantine lane for any self-selected WordPress/database runtime work. | Defer to lane 114 owner; do not integrate database/runtime work from that context. |

## Late-Priority Guardrail

The read-only late-row reports remain the governing evidence:

- `.harness/reports/phpt-manifest-late-row-tags.md`
- `.harness/reports/late-row-tag-crosscheck.md`
- `.harness/reports/221205Z-late-priority-overlap.md`
- `.harness/reports/late-priority-guardrail-active-replays-dev135.md`

The blocked `221205Z` overlap is only five rows: four `eval` rows and one
lexical variable-variable caveat. Those rows should be excluded from first-wave
repair selection unless a manager explicitly opens a late-priority boundary
lane. No branch in this quarantine map is accepted eval or variable-variable
support.

## Integration Rules

- Report artifacts may be consumed by exact artifact path and accepted commit.
  Do not use later source-changing heads on the same producer branch as report
  evidence.
- Runtime repair candidates need Integrator-owned dirty-overlap resolution and
  focused revalidation on the merged candidate. They still do not move public
  PHPT score without a zero-regression pinned public gate or accepted
  adjudication.
- Superseded self-selected implementation branches should remain audit-only.
  If a future task reopens one feature, start a fresh lane with explicit owned
  files, precheck/postcheck rows, tests, CLI exercise, docs, and named
  unsupported edge cases.
- Reserve/capacity events with `source_edits=false` are not listed as
  quarantine hazards; they are non-source scheduler state only.

## Commands And Data Sources

SQLite was accessed through Python's standard `sqlite3` module because both the
SQLite MCP wrappers and the `sqlite3` CLI were unavailable.

Commands and inputs used:

```sh
sed -n '1,260p' .harness/reports/superseded-lane-dirty-audit.md
sed -n '1,260p' .harness/reports/221205Z-source-diff-risk.md
sed -n '1,260p' .harness/reports/report-artifact-branch-map-dev131.md
sed -n '1,220p' .harness/reports/late-priority-guardrail-active-replays-dev135.md
sed -n '1,220p' .harness/reports/221205Z-late-priority-overlap.md
```

```sh
python3 - <<'PY'
import sqlite3
path='/home/claude/php-to-native-compiler/.harness/harness.sqlite3'
con=sqlite3.connect(path)
con.row_factory=sqlite3.Row
# Queried selected lanes 7,9,10,11,12,13,23,24,25,28,30,35,36,
# 40,61,63,65,66,67,68,69,100,111,112,114,123 and related events.
PY
```

```sh
git branch --list 'work/developer-35' 'work/developer-36' \
  'work/developer-37' 'work/developer-40' 'work/developer-43' \
  'work/developer-44' 'work/developer-83' 'work/developer-85' \
  'work/developer-86' 'work/developer-88' 'work/developer-114' \
  'work/developer-117' 'work/developer-120' 'work/developer-124' \
  --format='%(refname:short) %(objectname:short) %(committerdate:iso8601) %(subject)'
git for-each-ref refs/remotes/origin/work/developer-83 \
  refs/remotes/origin/work/developer-114 \
  refs/remotes/origin/work/developer-117 \
  refs/remotes/origin/work/developer-120 \
  refs/remotes/origin/work/developer-124 \
  --format='%(refname:short) %(objectname:short) %(committerdate:iso8601) %(subject)'
git show --stat --name-only --oneline --format='%h %s' \
  e147c03368275980f1852b3ce6b02be31fa8b679 \
  2f8aec28c4276a17a68986ad7d9b387513adebf5 \
  9f943b19 17e442ab 8636ec0d \
  26527dce7d950a5afe069436e3d611984eee29b3 \
  174370c4 e04e3df9a49f3a1cce20764279bc83cc81a48ebf \
  7a17b7eee5edb4ec2f2a12aa01d8ffddf2793d90 --
git status --short --branch --untracked-files=all
```

No recursive `.harness/worktrees` scan was performed. Dirty-file details for
older superseded self-selected lanes come from the already integrated
`.harness/reports/superseded-lane-dirty-audit.md` artifact.
