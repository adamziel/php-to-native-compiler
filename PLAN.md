# Plan

## Refined Goal

Build the stable Rust PHP-to-native compiler toward full PHP compatibility as
measured by official PHP PHPT pass counts. The primary execution surface for
compatibility is `phpc run`; `phpc compile --emit-ir` and `--emit-asm` must
continue to reject unsupported lowering instead of emitting misleading native
code.

`eval` and variable-variable support are explicitly late-priority work. They
may remain unsupported while the team closes other PHPT rows, as long as the
unsupported boundaries stay documented and tested.

## Current Authoritative State

- The local checkout `/home/claude/php-to-native-compiler` is dirty and stale
  at `e147c03368275980f1852b3ce6b02be31fa8b679`, behind `origin/master` by
  550 commits. Preserve it; do not treat its local progress files as current
  product truth.
- The current remote head observed during planning is
  `4d95c3df99a7b4856d0d4f3ef98e866e589189cb`.
- The latest accepted public PHPT score remains `7873 / 20294 = 38.79%` at
  source `0b917f67a37d9ca9779d77f87173b628431c2425`, with zero latest-public
  PASS regressions.
- A later full gate, `phpt-full-current-score-20260604T221205Z`, finished
  `FINAL / BLOCKED-PASS-REGRESSIONS`: `7197 / 20294 = 35.46%`, with `1166`
  PASS regressions against the accepted public baseline. This gate must not
  move the public score.
- Local `/home/claude/php-src-phpt` contains `21827` `.phpt` files. A content
  scan found `142` files mentioning `eval(`, `86` mentioning variable-variable
  patterns, and `226` unique files in the combined late-priority set. These
  counts are planning inputs, not a replacement for the pinned full-gate
  denominator.

## Deterministic Success Metric

Primary metric:

`accepted_public_phpt_passes / pinned_public_runnable_denominator`

A score update is accepted only when a full pinned PHPT gate:

1. Builds and runs `phpc` from the exact source commit being scored.
2. Uses the pinned php-src checkout and `phpc-phpt-wrapper`.
3. Reports aggregate PHPT counts and normalized current PASS rows.
4. Has zero PASS regressions versus the latest accepted public baseline, or has
   audited/adjudicated regression rows accepted by the auditor.
5. Records source commit, php-src pin, command shape, pass/fail/skip counts,
   denominator, regression count, and artifact paths.

Secondary lane metric:

Focused selected PHPT rows may be used to guide work only when each lane records
precheck and postcheck counts, exact PHPT paths, focused Rust tests, CLI
exercise path, docs updates, and named unsupported edge cases. Focused selected
rows are evidence, not public progress, until converted by a full zero-regression
gate.

Late-priority metric:

Rows whose PHPT content depends on `eval` or variable variables are tagged as
late. Do not spend implementation lanes on them until non-late regressions and
higher-yield compatibility clusters are exhausted, unless they block the gate
infrastructure itself.

## Milestones

### M0: Stop The Regression Bleed

Goal: classify and reduce the `1166` PASS regressions from the `221205Z`
blocked gate before broad new source integration.

Deliverables:

- Regression manifest split by extension, test directory, failure mode, and
  likely source/harness bucket.
- Replays of representative latest-public PASS rows on the accepted
  `0b917f67` binary and the candidate binary.
- First repair lanes only for deterministic clusters with small, provable
  fixes.
- A full pinned gate that returns to zero latest-public PASS regressions.

### M1: Stabilize Measurement And Control Plane

Goal: make scheduler-visible state match the authoritative PHPT scoring state.

Deliverables:

- `PLAN.md`, SQLite goal measure, metric sample, and planner insights updated.
- Stale default `python -m unittest` lane superseded for this Rust/PHP project.
- Dashboard/progress inputs refreshed through the dedicated progress-maintainer
  path, not by ad hoc score edits.
- Reproducible PHPT manifest command documented for all rows and late-priority
  `eval`/variable-variable rows.

### M2: Convert Focused Proof Into Public Score

Goal: resume compatibility expansion only after M0 is under control.

Priority lanes:

- Zend language/runtime semantics: exceptions, object lifecycle, strings,
  arrays, type coercions, and diagnostics.
- Standard library clusters with many failing PHPT rows: strings, arrays,
  filesystem, date/time, JSON/hash/password, PCRE, SPL, reflection.
- SAPI/runtime substrate: output buffering, headers/cookies/sessions, streams,
  uploads, shutdown/destructors, request state.
- Database and WordPress-facing runtime: mysqli/mysqlnd/PDO-like behavior,
  `wpdb`, object cache/options/transients, bootstrap probes.
- Native boundary lane: keep `compile` rejection honest, and lower only when a
  tested native runtime ABI exists.

### M3: Endgame Compatibility

Goal: after non-late PHPT clusters and regressions are mostly closed, decide
whether to implement `eval` and variable variables.

Deliverables:

- Updated unsupported-boundary docs if still deferred.
- If attempted, separate parser/runtime/native plans for `eval` and variable
  variables, with PHPT rows isolated from other compatibility work.

## Next Team Plan

1. Auditor: verify this metric and reject any public score movement from
   `221205Z` until PASS regressions are classified.
2. Manager: replace the stale default failing-test lane with regression
   triage, measurement refresh, and narrowly owned repair lanes.
3. Developer lane A: build the `221205Z` PASS-regression manifest and cluster
   top failures by directory and symptom.
4. Developer lane B: replay a small stratified sample of regression rows
   against accepted and candidate binaries to classify harness versus semantic
   failures.
5. Tests/docs lane: keep public progress unchanged, document the blocked gate,
   and ensure late-priority `eval`/variable-variable rows are tagged but not
   pulled into near-term implementation.
6. Only after a zero-regression repair path exists: resume focused PHPT
   compatibility lanes with precheck/postcheck evidence and full-gate
   conversion.

## Operating Rules For Lanes

- Use clean, current worktrees for source work. Do not integrate from the stale
  dirty root checkout without an explicit preservation and rebase decision.
- Use unique `CARGO_TARGET_DIR`, `CARGO_BUILD_JOBS=1`, and
  `CARGO_INCREMENTAL=0` for worker builds.
- Run focused checks first, then full gates only at checkpoint or score
  conversion boundaries.
- Never claim support without implementation code, tests, CLI exercise path,
  docs, and named unsupported edge cases.
- Use `tools/checkpoint.sh` for checkpoint commits; it stages the full tree, so
  inspect dirty state first.
