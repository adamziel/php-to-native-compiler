# Superseded-Lane Dirty Worktree Audit

Audit time: 2026-06-05T00:18:35Z
Auditor: developer-100, lane 28, branch `work/developer-100`

This is diagnostic/control-plane work only. It does not implement PHP
compatibility, does not exercise the PHPT gate, and cannot move the public PHPT
score. The accepted public score remains 7873/20294 at `0b917f67`; the
`phpt-full-current-score-20260604T221205Z` candidate remains blocked at
7197/20294 with 1166 PASS regressions. Eval and variable-variable rows remain
late-priority and were not implemented or replayed here.

## Inspection Scope

Commands and data sources used:

- Harness DB reads/updates from
  `/home/claude/php-to-native-compiler/.harness/harness.sqlite3` using Python's
  built-in SQLite driver because the `sqlite3` CLI and harness SQLite MCP tools
  were not available in this session.
- `git status --short --branch --untracked-files=all` for each requested
  lane worktree.
- Lane artifact path existence checks from `work_lanes.notes` report paths and
  explicit lane 28 report checks.

No compiler/runtime/source files were edited. No full PHPT gate was run.
`DEVELOPMENT.md` was requested by role instructions but is absent from both this
worktree and the main checkout.

## Requested Lane Statuses

| Lane | DB status | Agent / branch | Worktree | Dirty status |
| --- | --- | --- | --- | --- |
| 2 | completed | developer-94 / `work/developer-94` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-94` | Clean; tracks `origin/work/developer-94`. Assigned artifact `.harness/reports/221205Z-pass-regression-manifest.md` exists in the worktree. |
| 3 | in_progress | developer-92 / `work/developer-92` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-92` | Clean. Shared with completed lane 16, so integrate by exact lane artifact/commit only. |
| 4 | completed | developer-96 / `work/developer-96` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-96` | Clean; tracks `origin/work/developer-96`. Assigned artifact `.harness/reports/blocked-221205Z-progress-refresh.md` exists in the worktree. |
| 5 | completed | developer-95 / `work/developer-95` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-95` | Clean; tracks `origin/work/developer-95`. Assigned artifact `.harness/reports/phpt-manifest-late-row-tags.md` exists in the worktree. |
| 7 | superseded | developer-35 / `work/developer-35` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-35` | Dirty source/test work: `compiler/src/interpreter.rs`, `compiler/tests/array_reverse.rs`, `compiler/tests/array_slice.rs`. |
| 8 | in_progress | developer-80 / `work/developer-80` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-80` | Clean. Control-plane harness command-selection lane; no artifact path recorded in lane notes. |
| 9 | superseded | developer-36 / `work/developer-36` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-36` | Clean. Superseded short-echo lane should remain audit-only unless reassigned. |
| 10 | superseded | developer-40 / `work/developer-40` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-40` | Dirty source/docs/fixture work: modified `README.md`, `compiler/src/lexer.rs`, `docs/ARCHITECTURE.md`, `docs/PROGRESS.md`, `docs/SUPPORT.md`; deleted `tests/fixtures/unsupported_syntax_features/unsupported_short_echo_tag.*`; untracked `tests/fixtures/milestone2305/short_echo_inline_html.*` and `tests/fixtures/milestone2305/short_echo_tag.*`. |
| 11 | superseded | developer-43 / `work/developer-43` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-43` | Dirty source/test/fixture work: modified `compiler/src/interpreter.rs`, `compiler/tests/string_algorithm_builtins.rs`; untracked `tests/fixtures/milestone2303/similar_text_basic.*`. |
| 12 | superseded | developer-44 / `work/developer-44` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-44` | Clean. Superseded `str_ireplace` lane has no dirty worktree state. |
| 13 | superseded | developer-37 / `work/developer-37` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-37` | Dirty docs/test/fixture work: modified `README.md`, `compiler/tests/filemtime_builtin.rs`, `docs/ARCHITECTURE.md`, `docs/SUPPORT.md`, `tests/fixtures/milestone1313/filemtime_local_metadata.php`, `tests/fixtures/milestone1313/filemtime_local_metadata.stdout`. |
| 14 | in_progress | developer-101 / `work/developer-101` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-101` | Clean. Lane was reassigned during this audit from failed developer-88 to developer-101; expected artifact `.harness/reports/221205Z-standard-array.md` is not present yet. |
| 15 | in_progress | developer-89 / `work/developer-89` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-89` | Clean. Expected artifact `.harness/reports/221205Z-standard-strings.md` is not present yet. |
| 16 | completed | developer-92 / `work/developer-92` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-92` | Clean. Same branch/worktree as active lane 3; completed artifact is present centrally at `/home/claude/php-to-native-compiler/.harness/reports/221205Z-standard-filesystem-http.md` but not in this worktree. |
| 17 | completed | developer-83 / `work/developer-83` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-83` | Dirty due to untracked `.harness/reports/221205Z-late-priority-overlap.md`, which appears unrelated to requested lanes 17/20/24/25 and belongs to later lane 30 work. Assigned artifact `.harness/reports/221205Z-standard-scalar-misc.md` exists in the worktree. |
| 18 | in_progress | developer-90 / `work/developer-90` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-90` | Clean. Expected artifact `.harness/reports/221205Z-spl.md` is not present yet. |
| 19 | in_progress | developer-91 / `work/developer-91` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-91` | Clean. Expected artifact `.harness/reports/221205Z-reflection.md` is not present yet. |
| 20 | completed | developer-83 / `work/developer-83` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-83` | Dirty due to the same untracked `.harness/reports/221205Z-late-priority-overlap.md`. Assigned artifact `.harness/reports/221205Z-secondary-ext.md` exists in the worktree. |
| 21 | in_progress | developer-98 / `work/developer-98` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-98` | Dirty only by untracked assigned report artifact `.harness/reports/221205Z-zend-classes-sapi.md`. Not integration-ready until committed/validated by that lane. |
| 22 | in_progress | developer-93 / `work/developer-93` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-93` | Clean. Expected artifact `.harness/reports/221205Z-status-symptom-crosscheck.md` is not present yet. |
| 23 | queued | developer-38 / `work/developer-38` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-38` | Clean. DB still marks lane queued; expected artifact `.harness/reports/221205Z-source-diff-risk.md` is not present. |
| 24 | completed | developer-83 / `work/developer-83` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-83` | Dirty due to the same untracked `.harness/reports/221205Z-late-priority-overlap.md`. Assigned artifact `.harness/reports/late-row-tag-crosscheck.md` exists in the worktree. |
| 25 | completed | developer-83 / `work/developer-83` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-83` | Dirty due to the same untracked `.harness/reports/221205Z-late-priority-overlap.md`. Assigned artifact `.harness/reports/221205Z-standard-strings-replace-replay.md` exists in the worktree. |
| 28 | in_progress | developer-100 / `work/developer-100` | `/home/claude/php-to-native-compiler/.harness/worktrees/developer-100` | Clean before this report was created. Prior lane 28 artifact was missing from central reports, developer-87, and developer-100 worktrees before this audit. |

## Lane 28 Replacement Notes

- Original lane 28 owner developer-68 is stopped; `work/developer-68` is clean.
- Replacement developer-87 is marked crashed/ended with missing tmux window.
  `work/developer-87` is clean, and no
  `.harness/reports/superseded-lane-dirty-audit.md` existed there.
- Manager assignment message 49 for developer-87 is `undeliverable`; it did not
  produce a lane 28 artifact.
- Auditor notes say developer-87 branch head `e147c033` is unrelated source work
  and must not count as lane 28 output.

## Integration Hazards

- Superseded lanes 7, 10, 11, and 13 contain uncommitted source/docs/test work.
  These are the highest-risk dirty branches because they touch compiler/runtime
  behavior or support documentation after the manager restricted current work to
  M0/M1 regression control.
- Lanes 7 and 11 both modify `compiler/src/interpreter.rs`; integrating both
  without reassignment would create a direct source-conflict and would violate
  the active M0/M1 scope.
- Lanes 10 and 13 both modify `README.md` and support/architecture docs. Lane 10
  also removes unsupported short-echo fixtures while adding short-echo support
  fixtures, contradicting the current late/superseded implementation policy.
- `work/developer-83` is reused for lanes 17, 20, 24, and 25 and currently has
  an untracked `.harness/reports/221205Z-late-priority-overlap.md` from later
  lane 30 work. Do not bundle that untracked file into any lane 17/20/24/25
  integration.
- Lane 16 is completed on the same clean `work/developer-92` branch as active
  lane 3. Treat lane 16's existing artifact/commit separately from any future
  lane 3 output.
- Lane 21 has an untracked assigned report artifact but no committed artifact
  yet; it is an active in-progress report, not a completed integration input.
- Lane 14 was reassigned during the audit. Stale developer-88 is failed/ended
  and clean, but its prior unrelated output must not be treated as lane 14
  metric progress. Current owner developer-101 is clean and has not produced the
  lane 14 report yet.

## Branches That Must Not Be Integrated As Metric Progress

Do not integrate these branches as PHPT metric progress unless a manager or
integrator explicitly reassigns/reviews them:

- Superseded implementation branches: `work/developer-35`,
  `work/developer-36`, `work/developer-40`, `work/developer-43`,
  `work/developer-44`, `work/developer-37`.
- Stale lane 28 branches: `work/developer-68`, `work/developer-87`.
- Stale lane 14 branch: `work/developer-88`.
- Uncommitted late-priority artifact on `work/developer-83`:
  `.harness/reports/221205Z-late-priority-overlap.md`; this is not progress for
  requested lanes 17, 20, 24, or 25.
- Active in-progress report branches `work/developer-80`, `work/developer-89`,
  `work/developer-90`, `work/developer-91`, `work/developer-93`,
  `work/developer-98`, `work/developer-101`, and `work/developer-92` should not
  be counted as metric progress until their lane artifacts are committed,
  pushed if applicable, and accepted. They are control-plane/regression
  classification lanes, not score-moving compatibility fixes.

## Missing Or Inaccessible Paths

- No requested lane worktree path was missing or inaccessible during this audit.
- Before this report was written, the lane 28 report was missing from the
  central reports directory and from the developer-87/developer-100 worktrees.
- Some expected active-lane report artifacts are not present yet because those
  lanes are still in progress or queued: lanes 14, 15, 18, 19, 22, and 23.
