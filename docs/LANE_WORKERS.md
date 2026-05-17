# Lane Workers

Use this guide when splitting `GOAL.MD` work across subagents. The goal is to
let parser, IR/lowering, runtime, compiler-output, and tests/docs workers make
progress independently without overwriting each other's files or turning focused
test runs into unbounded waits.

## Worktree Setup

Start each lane from the same clean base commit when possible:

```sh
git worktree add ../phpc-parser-lane -b lane/parser HEAD
git worktree add ../phpc-ir-lane -b lane/ir-lowering HEAD
git worktree add ../phpc-runtime-lane -b lane/runtime HEAD
git worktree add ../phpc-output-lane -b lane/compiler-output HEAD
git worktree add ../phpc-tests-docs-lane -b lane/tests-docs HEAD
```

If the main tree is already dirty, do not use `tools/checkpoint.sh` from that
tree. Either checkpoint from a lane worktree whose status has been inspected, or
wait for the integration owner to batch and test the combined changes.

Each worker should set a unique target directory so cargo builds do not fight
over one shared `target/`:

```sh
export CARGO_TARGET_DIR=/dev/shm/phpc-target-parser
export CARGO_BUILD_JOBS=1
export CARGO_INCREMENTAL=0
```

Use a lane-specific suffix for `CARGO_TARGET_DIR`, such as `ir`, `runtime`,
`output`, or `tests-docs`.

## Lane Ownership

Parser lane:

- Owns lexer, parser, AST shape, parse diagnostics, and syntax fixtures.
- Avoids runtime semantics and native support claims unless another lane has
  implemented and tested them.
- Typical focused checks: parser unit tests, syntax boundary tests, targeted
  fixture directories, and `git diff --check`.

IR/lowering lane:

- Owns `phpc compile --emit-ir`, `phpc compile --emit-asm`, native codegen
  diagnostics, deterministic backend/fallback snapshots, and native support
  docs.
- Avoids parser/runtime rewrites unless a slice cannot be proved without a
  named cross-lane handoff.
- Typical focused checks: relevant `native_*_boundary` tests, relevant
  `native_assembly_cli` snapshot tests, targeted fixture directories, and
  `git diff --check`.

Runtime lane:

- Owns `php_runtime`, interpreter behavior for `phpc run`, boxed values,
  builtins, object/array semantics, and system PHP comparison fixtures.
- Avoids native compile support claims unless the IR/lowering lane implements
  and tests them.
- Typical focused checks: affected runtime/compiler integration tests, targeted
  fixture directories, `phpc test --compare-php` for those directories, and
  `git diff --check`.

Compiler-output lane:

- Owns CLI behavior, emitted artifact shape, backend selection and failure
  behavior, stdout/stderr/exit contracts, and operational scripts.
- Avoids changing expression lowering internals while an IR/lowering milestone
  is active.
- Typical focused checks: specific CLI snapshot tests, focused emitted-artifact
  fixtures, script dry-runs where available, and `git diff --check`.

Tests/docs lane:

- Owns fixture organization, focused/full gate documentation, support matrix
  accuracy, progress records, operations docs, and unsupported-edge naming.
- Avoids implementation files unless fixing the documented verification surface
  requires a named handoff.
- Typical focused checks: documentation grep checks, targeted fixture runner
  commands when fixtures change, and `git diff --check`.

## Subagent Prompt Template

Give each subagent a lane, one milestone, and a forbidden-file boundary:

```text
Lane: <parser|ir-lowering|runtime|compiler-output|tests-docs>
Workspace: <absolute worktree path>
Objective: complete exactly one small milestone under GOAL.MD.
Ownership: <files and behavior this lane owns>.
Forbidden without handoff: <files or lanes to avoid>.
Required reading: AGENTS.md, GOAL.MD, docs/NEXT_TASKS.md, docs/PROGRESS.md,
docs/SUPPORT.md, README.md, docs/OPERATIONS.md, and any lane-specific doc.
Required output: files changed, commands run, focused verification result,
unsupported or remaining gaps, and whether the full suite was deferred.
Do not checkpoint or commit unless explicitly instructed.
```

## Focused Test Policy

During a lane slice, run the smallest tests that prove the changed behavior.
The full gate remains mandatory before checkpoint batches unless a blocker is
recorded.

Start with the narrowest executable proof:

- one affected Rust integration test file, or one named test inside it;
- one exact fixture directory when fixtures changed;
- PHP comparison only for fixture directories intended to match system PHP;
- one CLI snapshot integration test when `.cli` files changed;
- `cargo fmt --check` only when Rust files changed;
- `git diff --check -- <changed-files>` for every lane.

Do not run workspace `cargo test` as the first verification step for a narrow
lane slice. Escalate from a single named test, to the whole affected test file,
to the exact fixture directory, and then to `tools/run-tests.sh` when the change
touches shared infrastructure or before checkpoint batches.

Record the exact focused commands in `docs/PROGRESS.md`. If the full gate is
deferred, say so directly and name the next point where it must run.

## Unsupported Syntax Boundary Checklist

Use this checklist when closing a parser-lane unsupported syntax boundary.
The goal is a stable, honest rejection surface, not partial support.

- Name the boundary as lex, parse, runtime, or codegen. Syntax that cannot be
  tokenized safely should stop in lexing; reserved tokens or grammar forms
  should stop in parsing; accepted interpreter-only constructs must still have
  explicit native codegen rejection when lowering is unsupported.
- Pin the diagnostic message and source location with focused coverage in
  `compiler/tests/syntax_boundaries.rs`. Include representative forms, casing
  where relevant, expression-position forms where relevant, and an
  `emit_ir_source(...)` assertion when native lowering must reject before
  backend invocation.
- Add or update a fixture under
  `tests/fixtures/unsupported_syntax_features/` with `.php`, `.stderr`,
  `.exit`, `.cli`, and `.phpc-only` files. The committed fixture should prove
  the `phpc run` CLI path, while `.phpc-only` keeps project-specific
  diagnostics out of system PHP comparison.
- Update `README.md`, `docs/SUPPORT.md`, and `docs/ARCHITECTURE.md` when the
  boundary changes the public support surface. Name the unsupported edge cases
  directly, including future runtime and native lowering work that is not being
  implemented.
- Update `docs/NEXT_TASKS.md` and `docs/PROGRESS.md` with the milestone,
  focused commands, full-gate status, and whether checkpointing was skipped
  because the active tree contains unrelated work.
- Run focused checks before any full gate:
  `cargo test -p phpc --test syntax_boundaries <boundary-name> -- --test-threads=1`;
  `cargo test -p phpc --test unsupported_syntax_features_cli -- --test-threads=1`;
  `cargo run -p phpc -- test tests/fixtures/unsupported_syntax_features`;
  `cargo run -p phpc -- test --compare-php tests/fixtures/unsupported_syntax_features`;
  direct `phpc run <fixture>` for the new boundary fixture; direct
  `phpc compile <fixture> --emit-ir` when the construct should fail before
  native lowering or backend invocation; plus scoped `git diff --check`.

Current examples:

- `yield` and `yield from`: parse boundary with unit coverage in
  `unsupported_yield_syntax_has_stable_parse_errors`,
  `emit_ir_rejects_yield_syntax_at_parse_boundary`, and the
  `unsupported_yield` fixture set.
- `goto` statements and labels: parse boundary with unit coverage in
  `unsupported_goto_syntax_has_stable_parse_errors`,
  `emit_ir_rejects_goto_syntax_at_parse_boundary`, and the `unsupported_goto`
  fixture set.
- heredoc/nowdoc strings: lex boundary with unit coverage in
  `unsupported_heredoc_nowdoc_syntax_has_stable_lex_errors`,
  `emit_ir_rejects_heredoc_nowdoc_syntax_at_lex_boundary`, and the
  `unsupported_heredoc` fixture set.

## Handoff Notes

When a lane touches shared files such as `README.md`, `docs/SUPPORT.md`,
`docs/ARCHITECTURE.md`, `docs/NEXT_TASKS.md`, or `docs/PROGRESS.md`, keep the
edit narrow and include a handoff note in the final report.

Cross-lane handoff notes must name:

- affected lane;
- files touched;
- focused commands already run;
- full-suite status;
- remaining unsupported or unverified behavior.

Do not treat a passing focused command as proof for another lane's milestone
unless the command covers that lane's requirements directly.

## Current Lane Queue

Use `docs/NEXT_TASKS.md` as the source of truth for milestone status. The
current split is:

- Tests/docs lane: Milestone 1410 closes the 1406-1409 WordPress-focused
  implementation-batch queue refresh and keeps the next split anchored in the
  `GOAL.MD` compatibility ledger.
- Parser/runtime lane: Milestone 1411 targets a larger object/interface
  blocker, such as constructor/destructor fidelity, magic hooks reached by
  WordPress, trait alias and conflict rules, reflection metadata objects,
  static member lookup interactions, class/interface diagnostics, or
  autoload/class-alias lifecycle parity.
- Runtime lane: Milestone 1412 targets a larger reference/COW blocker, such as
  real reference containers, ArrayAccess reference roots, append-offset
  callback roots, string-keyed callback reference arguments, alias lifetime
  cleanup across function/include boundaries, broader array/object
  copy-on-write, or request/session alias behavior that blocks WordPress.
- Runtime or IR/lowering lane: Milestone 1413 targets request/SAPI,
  filesystem, and stream behavior that blocks real WordPress requests, such as
  output-started/header fidelity, shutdown/fatal/destructor edge ordering,
  upload/session/cookie persistence, include-path/stat-cache behavior, stream
  filters/wrappers, wrapper-specific stream context behavior, request-body
  lifetime, or host filesystem edge cases.
- Compatibility/runtime lane: Milestone 1414 targets executable WordPress
  database/bootstrap evidence: broader `wpdb`/MySQLi result or mutation
  behavior, prepared option/transient query shapes, object-cache/transient
  persistence, option API state, hooks under realistic callback shapes,
  deterministic plugin/theme loading probes, or a bootstrap/request probe that
  moves past its next real blocker.
- Tests/docs lane: Milestone 1415 is the next tests/docs slot after the next
  WordPress-focused implementation batch.

Milestones 555-560 closed the first split-lane batch, Milestones 561, 571, 587,
592, 597, 602, 607, 612, 617, 1201, 1206, 1211, 1216, 1221, 1226, 1231, 1236, 1241, 1246, 1251, 1256, 1261, 1266, 1271, 1276, 1281, 1286, 1291, 1301, 1306, 1311, 1316, 1321, 1326, 1331, 1336, 1341, 1346, 1351, 1356, 1361, 1366, 1371, 1376, 1381, 1386, 1391, 1396, 1401, and 1406 closed recent parser slots, Milestones 565, 568,
572, 575, 577, 579, 581, 583, 585, 590, 595, 600, 605, 610, 615, 1204, 1209, 1214, and 1219 closed recent
compiler-output slots, Milestones 567, 570, 574, 576, 578, 580, 582, 584, 586,
593, 598, 603, 608, 613, 618, 1202, 1207, 1212, 1217, 1222, 1223, 1227, 1228, 1232, 1233, 1237, 1238, 1242, 1243, 1247, 1248, 1252, 1253, 1257, 1258, 1262, 1263, 1267, 1268, 1272, 1273, 1277, 1278, 1282, 1283, 1287, 1288, 1292, 1293, 1302, 1303, 1307, 1308, 1312, 1313, 1317, 1318, 1322, 1323, 1327, 1328, 1332, 1333, 1337, 1338, 1342, 1343, 1347, 1348, 1352, 1353, 1357, 1358, 1362, 1363, 1367, 1368, 1372, 1373, 1377, 1378, 1382, 1383, 1387, 1388, 1392, 1393, 1397, 1398, 1402, 1403, 1407, and 1408 closed recent runtime slots, and Milestones 569,
573, 588, 591, 596, 601, 606, 611, 616, 1203, 1208, 1213, and 1218 closed recent IR/lowering slots.
Milestones 1224, 1229, 1234, 1239, 1244, 1249, 1254, 1259, 1264, 1269, 1274, 1279, 1284, 1289, 1294, 1304, 1309, 1314, 1319, 1324, 1329, 1334, 1339, 1344, 1349, 1354, 1359, 1364, 1369, 1374, 1379, 1384, 1389, 1394, 1399, 1404, and 1409 closed recent WordPress compatibility evidence work. Milestones
604, 609, 614, 619, 1205, 1210, 1215, 1220, 1225, 1230, 1235, 1240, 1245, 1250, 1255, 1260, 1265, 1270, 1275, 1280, 1285, 1290, 1295, 1300, 1305, 1310, 1315, 1320, 1325, 1330, 1335, 1340, 1345, 1350, 1355, 1360, 1365, 1370, 1375, 1380, 1385, 1390, 1395, 1400, 1405, and 1410 closed recent tests/docs
queue refreshes.
Milestones 1156 through 1159 closed the latest parser/runtime/IR/compiler-output
implementation batch, and Milestone 1160 refreshed the next-lane queue.
Milestones 1161 through 1164 closed the latest parser/runtime/IR/compiler-output
implementation batch, and Milestone 1165 refreshed the next-lane queue.
Milestones 1166 through 1169 closed the latest parser/runtime/IR/compiler-output
implementation batch, and Milestone 1170 refreshed the next-lane queue.
Milestones 1171 through 1174 closed the latest parser/runtime/IR/compiler-output
implementation batch, and Milestone 1175 refreshed the next-lane queue.
Milestones 1176 through 1179 closed the latest parser/runtime/IR/compiler-output
implementation batch, and Milestone 1180 refreshed the next-lane queue.
Milestones 1181 through 1184 closed the latest parser/runtime/IR/compiler-output
implementation batch, and Milestone 1185 refreshed the next-lane queue.
Milestones 1186 through 1189 closed the latest parser/runtime/IR/compiler-output
implementation batch, and Milestone 1190 refreshed the next-lane queue.
Milestones 1191 through 1194 closed the latest parser/runtime/IR/compiler-output
implementation batch, and Milestone 1195 refreshed the next-lane queue.
Milestones 1196 through 1199 closed the latest parser/runtime/IR/compiler-output
implementation batch, and Milestone 1200 refreshed the next-lane queue.
Milestone 1094 closed the runtime clone/reference-slot mirroring slice,
Milestone 1095 closed the promoted constructor property parameter parse
boundary, Milestone 1096 closed the native reference-assignment rejection
boundary, and Milestone 1097 closed fixture-runner compare-summary coverage.
Milestone 1098 closed context-aware non-public clone reference-slot mirroring,
Milestone 1099 closed the nullsafe object operator parse boundary, Milestone
1100 closed the native clone rejection boundary, and Milestone 1101 closed
compile emit-mode validation precedence. Milestone 1105 refreshed the
compatibility gap map and next-lane queue without changing implementation or
fixture behavior. Milestone 1109 closed the compiler-output comparison-summary
skip-reason contract for `phpc test --compare-php`. Milestones 1106 through
1109 closed the latest parser/runtime/IR/compiler-output implementation batch,
Milestone 1110 refreshed the next-lane queue, Milestones 1111 through 1114
closed the latest parser/runtime/IR/compiler-output implementation batch, and
Milestone 1115 refreshed the next-lane queue. Milestones 1116 through 1119
closed the latest parser/runtime/IR/compiler-output implementation batch, and
Milestone 1120 refreshed the next-lane queue. Milestones 1121 through 1124
closed the latest parser/runtime/IR/compiler-output implementation batch, and
Milestone 1125 refreshed the next-lane queue. Milestones 1126 through 1129
closed the latest parser/runtime/IR/compiler-output implementation batch, and
Milestone 1130 refreshed the next-lane queue. Milestones 1131 through 1134
closed a parser/runtime/IR/compiler-output implementation batch, and
Milestone 1135 refreshed the next-lane queue. Milestones 1136 through 1139
closed a parser/runtime/IR/compiler-output implementation batch, and
Milestone 1140 refreshed the next-lane queue. Milestones 1141 through 1144
closed a parser/runtime/IR/compiler-output implementation batch, and
Milestone 1145 refreshed the next-lane queue. Milestones 1146 through 1149
closed the latest parser/runtime/IR/compiler-output implementation batch, and
Milestone 1150 refreshed the next-lane queue.
The next batch should again keep one active milestone per lane and should use
separate worktrees and separate `CARGO_TARGET_DIR` values when workers run in
parallel.
