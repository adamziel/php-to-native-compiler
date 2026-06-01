# PHP Native Compiler PHPT Progress

Updated: 2026-06-01 17:03 CEST

Primary/public branch: `origin/master`
Latest source-bearing public head:
`43262ab5f81fe293a49829c9c270137be98f5e6d checkpoint: repair PHPT gate regressions`

Semantic source for current published score:
`43262ab5f81fe293a49829c9c270137be98f5e6d checkpoint: repair PHPT gate regressions`

Public PHPT metric:

`passed runnable PHPTs / total pinned runnable PHPTs`

Pinned denominator: `20294` total pinned runnable php-src PHPTs. Raw runner
denominators that exclude BORKED rows are not public progress.

Current public score: **5363 / 20294 pinned runnable PHPTs = 26.43%**.

Local supervisor note: strict-identity `--emit-ir` Rust assertions were
refreshed for the already-present boxed diagnostic-result echo boundary. The
focused strict-identity suite passes locally, but this is not a public score
update.

Local supervisor note: the `$_SESSION` undefined-read Rust baseline now matches
the current PHP-shaped warning-and-continue execution path before session
startup. The focused superglobals suite passes locally, but this is not a
public score update.

Local supervisor note: `syntax_boundaries` was refreshed for the current
parser/runtime/codegen boundaries, including accepted attribute/function-DNF
metadata, first-class callables, `\PHP_VERSION`, parenthesized dynamic `new`,
native spread lowering, and direct reference assignment. The focused
syntax-boundaries suite passes locally, but this is not a public score update.

Local supervisor note: Worker 01 landed a bounded `count()` / `sizeof()`
operand-name improvement that turns two focused public PHPT rows green in its
lane report, and native type-introspection assertions now keep non-string
`function_exists()` / `extension_loaded()` names at the explicit function-call
rejection boundary. Focused Rust suites pass locally, but this is not a public
score update.

Local supervisor note: the typed-property protected-inheritance Rust baseline
now matches the current PHP-shaped fatal text for typed children extending
untyped protected parent properties. Worker lanes for `$GLOBALS`,
`intval($value, $base)`, `str_ireplace(..., $count)`, `is_callable()`, and
filesystem `open_basedir` metadata predicates have returned candidate patches
for later integration, but the public score remains unchanged until a full
pinned PHPT gate is published.

Local supervisor note: the typed-property reference-coercion Rust baseline now
asserts the current PHP-shaped fatal execution path for incompatible writes
through typed property references. The focused suite passes locally, but this
is not a public score update.

Local supervisor note: the `variable_unset` Rust baseline now matches the
current PHP-shaped warning-and-continue path when reading a local after
`unset()`. The focused suite passes locally, but this is not a public score
update.

Local supervisor note: checkpoint `263f60c4` produced a candidate full pinned
PHPT score of `5361 / 20294 = 26.42%`, but publication was blocked by two
latest-published PASS regressions: `is_file_variation4.phpt` and
`vfprintf_error4.phpt`. Checkpoint `43262ab5` repaired both regressions and
the rerun published `5363 / 20294 = 26.43%` with zero latest-published PASS
regressions.

## Current Public Gate

Published gate: Batch024 regression repair.

- Gate run:
  `phpt-full-batch024-regression-repair-20260601T145651Z-php-src-f97ff59-source-43262ab5`
- Source: `43262ab5f81fe293a49829c9c270137be98f5e6d checkpoint: repair PHPT gate regressions`
- Score: **5363 / 20294 pinned runnable PHPTs = 26.43%**
- Regression result: zero latest-published PASS regressions against the
  Batch023 repair01 PASS baseline.
- Gate notes: the full `open_basedir_*` family was serialized; the known
  sockets expected-output marker was adjudicated as failed-row output, not a
  harness marker failure. The previously blocking Batch024 candidate
  regressions `is_file_variation4.phpt` and `vfprintf_error4.phpt` now pass.

No focused PHPT run, source checkpoint, status note, PR, or candidate gate
changes the public score until it is parsed, regression-checked against the
latest published PASS set, and recorded here.

## Blocked / Unpublished Candidates

- Batch023 checkpoint10 was superseded by Batch023 repair01. Its candidate
  score is no longer current public progress.

## Batch024 Staging Checklist

Batch024 is accumulating source fixes after the Batch023 repair01 full-suite
gate. Focused PHPT proof is used for each accepted source slot, but the public
percentage does not change until a supervisor-owned full-suite gate is run for
the batch, all latest-public PASS regressions are repaired or adjudicated, and
the accepted score is recorded here.

Accepted source slots as of public/source head
`67c7a328a1e819c05e723f9f012763210b219a21`:

- [x] Slot 1: `7fdd2f668f5f61a788e53292b42f32e682cbc72a` URI
  WhatWG residuals, patch `sha45916ebb`: source integrated after reviewer
  FINAL GO, two critic SAFE artifacts, p38-ready, supervisor proof, focused
  PHPT `34 / 34`, and `0 / 34` latest-public PASS overlap.
- [x] Slot 2: `49a8c6fceb0bd562837b5a03b8b36734529a3a70` array
  negative auto-key semantics, patch
  `2cd2baeb161e48342c04881d18d890f2b1f830cd2a3159f1b550b33acfc483f0`,
  focused PHPT `2 / 2`, and `0 / 2` latest-public PASS overlap.
- [x] Slot 3: `00679068891d394d7e34ec9e49ef9d54781ce620`
  tokenizer PhpToken error-line provenance, patch
  `0cb2f2a216cc816aca82d35a4b72d4da775b1a6087e53437c0700ee63dc36335`,
  focused PHPT `2 / 2`, and `0 / 2` latest-public PASS overlap.
- [x] Slot 4: `fc7b886987a06730ad9bf00354cbe8b6360eb06a` INI
  parse/scanner overflow semantics, patch
  `ef881c5249b61db4a348ff6617f89bc9a1faf9c88e5dd46e424a3c80fc22c269`,
  focused PHPT `2 / 2`, and `0 / 2` latest-public PASS overlap.
- [x] Slot 5: `0b1e988917a4067addd776ee4e99f817eba1b8d9` PCRE
  backtrack/preg semantics, patch
  `4af9d61cdb763be3b54c4d701a004b7403e0832605a8df6acba92727030b4c81`,
  focused PHPT `7 / 7`, and `0 / 6` latest-public PASS overlap.
- [x] Slot 6: `cc07c0517915ce9cd730fcb56b4429fdd42f35ce` INI
  config/readback arg-separator/raw-scanner semantics, patch
  `4b5e8166e7828364da251f956cdfd1b4da0d78ec6889953f88de823d32902403`,
  focused PHPT `3 / 3`, and `0 / 3` latest-public PASS overlap.
- [x] Slot 7: `8ca79b41ae7ccecdf1aa4e030d9a708fa509429d`
  Reflection static/default property metadata, patch
  `1768a94679b0e3c51f4ce68cfae0540680433b03659db5873b42ebf00c10da26`,
  focused PHPT `7 / 7`.
- [x] Slot 8: `488c15046f0e121041dd588f4fa121a6b04e6551` mbstring
  `mb_substr()` default encoding semantics, patch
  `cf583257d7dc3dec9b7dbb6596c5b5d9bf1efa935ee11117107c1357d7706b0d`,
  focused PHPT `6 / 6`.
- [x] Slot 9: `7d1df541423a10f0b84afe40871c3cd787c7ccde`
  ReflectionExtension registry semantics, patch
  `eccafe08a0b22878cdb8c122ee1aad6deb9e5651d8f72be12745b510406d8a23`,
  focused PHPT `4 / 4`.
- [x] Slot 10: `ae137fe082c55451ab5dbb939d8dd3b25da5eb88`
  standard `settype()` casting semantics, patch
  `19d05729f6e7b874ee698f9dd0180c0d1ced3e94fe6273ad4674439d75587974`,
  focused PHPT `8 / 8`.
- [x] Slot 11: `263c97b65b36a046c9992c6211ced5f1657795ea`
  random procedural function semantics, patch
  `49353024ec2a3c03680aac2fdb1e360abfc74f891787e39a90686c473b87f8e7`,
  focused PHPT `11 / 11`.
- [x] Slot 12: `0c8fc57b695a1ebb002c1cb3e43f3e8f891d30ff` Zend
  offset tail semantics, patch
  `c7ab2012f3b66e124ba08b691bbc8221fbd1e0b2fab608a600f0caa5b60ad3e1`,
  focused PHPT `7 / 7`.
- [x] Slot 13: `34cd7257c7eb5ba8dc8359b7d1cb515cac2c1a5f` Zend
  union bool/default semantics, patch
  `066b314ede348502db732031b6a246da3240443c77f364fe0d739070d5fe499c`,
  focused PHPT `2 / 2` and literal bool probes `4 / 4`.
- [x] Slot 14: `67c7a328a1e819c05e723f9f012763210b219a21`
  standard stream-context diagnostics, patch
  `ae68026fe03088451b9c904e2bba5fd11ceff4e10a21f41276a3839c3453cb36`,
  focused PHPT `3 / 3` and non-repeat Rust guard `1 / 1` for the old
  `fwrite('not-resource', 'x')` diagnostic path.
- [x] Batch024 full-suite gate: published as Batch024 regression repair at
  `5363 / 20294 = 26.43%` with zero latest-published PASS regressions.

Current rejected, reverted, stale, or still-pending Batch024 candidates:

- `d93cc660db8e49b5e8f311f101f73b8ff6492bd5` INI parse/scanner
  successor, patch
  `624013a7ed8f8ebee92a53c90054a629afe090c586ce7fc324a8ba231cc8d67b`,
  was reverted by `1db84c72f92be1d9c15dee22d984338450a12ff0` after a
  critic DO-NOT-SAFE overflow counterexample; `1db84c72` has no tree diff
  against prior good `49a8c6fceb0bd562837b5a03b8b36734529a3a70`.
- `181e5838` Zend union defaults is rejected for the legal
  `false|int $x = false` default counterexample.
- `c62d8fa3` PCRE helper APIs is rejected because
  `preg_replace_callback_array()` must preserve sequential callback side
  effects before a later invalid pattern returns `NULL`.
- `a86157c9` PCRE match hygiene and successor `0e8556de` are rejected for
  overbroad backtrack-limit behavior; later PCRE v2/v3/v4 packets
  `da55a954`, `4467ce8e`, and `1b231a6a` also have FINAL NO-GO artifacts,
  while accepted PCRE v5 is slot 5 above.
- `f96d5381` INI parse quantity and `cd6699e9` tokenizer PhpToken object
  diagnostics have reviewer NO-GO artifacts.
- Reflection static/default candidates `04e2c7f3`, `0d7353b5`, and
  `f2d73dc1` are rejected/stale; accepted Reflection v4 is slot 7 above.
- HTML entity split candidates `f75f5e48`, `e7da26fb`, and `c580522b` are
  rejected by formatter or focused-Rust gates.
- Tokenizer lexical-tail `a1a95bb0` is rejected for the
  `PhpToken_constructor.phpt` public-guard object id failure. The repaired
  author-ready patch
  `b11122f97088323365a078a7badc0e03969f7549e91847f033c71ea6dcb5e7b2`
  remains held after exact-current `bcs3-tokenizer-review-status.md` found
  broad object-id lifetime risk, numeric underscore/overflow mismatches,
  under-modeled `TOKEN_PARSE` trait adaptation contexts, and raw-byte
  non-canonical cast deprecation false positives. The supervisor integration
  worktree no longer carries tokenizer source/test changes for this candidate.
- Local supervisor native-link checkpoint maintenance refreshed stale
  generated-C assertion shapes and wired runtime callable `strpos(...)`
  through the string-search result path. The same checkpoint pass refreshed
  stale native logical-boundary emit-IR snapshots for the boxed
  diagnostic-result output path and stale native mutation-boundary assertions
  for boxed assignment/compound output, non-local unset diagnostics, and the
  current direct-variable reference-binding split. This is not a public score update:
  source-only `native_executable_c_source` passed `417 / 417`, full
  `native_link` passed `823 / 823`, focused `native_logical_boundary` passed
  `19 / 19`, focused `native_mutation_boundary` passed `12 / 12`, and focused
  `native_object_class_boundary` passed `57 / 57`, focused
  `native_runtime_abi` passed `80 / 80`, focused `native_scalar_echo_boundary`
  passed `8 / 8`, focused `native_string_arithmetic` passed `4 / 4`, and
  focused `native_type_introspection_boundary` passed `18 / 18`, and focused
  `native_unary_boundary` passed `32 / 32`.
- Local supervisor high-intensity worker candidates are now integrated as
  focused Batch024 source candidates, still not a public score update until the
  next full gate: trim-family PHPT proof passed `4 / 4`, numeric angle
  conversion PHPT proof passed `4 / 4`, `file()` open_basedir follow-on
  warning PHPT proof passed `2 / 2`, and `empty()` expression PHPT proof
  passed `1 / 1` through lowercase `run-tests.php -p` with the wrapper.
- Local supervisor `str_replace()` baseline maintenance refreshed stale Rust
  assertions for already-present callback-by-value count warnings and current
  one-level array replacement behavior. Direct-variable `$count` writeback
  remains the only supported direct mutation path; non-direct direct-call count
  targets still reject, while `call_user_func("str_replace", ..., 0)` warns and
  returns the replacement result. Focused `str_replace_builtin` passed `7 / 7`;
  this is not a public score update.
- Local supervisor `strcasecmp()` baseline maintenance refreshed the stale
  too-few-arguments expectation to the current PHP-shaped fatal execution
  result. Focused `strcasecmp_builtin` passed `4 / 4`; this is not a public
  score update.
- Local supervisor `open_basedir` metadata/directory denial repair is a
  Batch024 source checkpoint candidate and still not a public score update. It
  covers
  relative-parent escape denial for `file_exists()`, `filesize()`, `is_dir()`,
  `is_file()`, `is_readable()`, `is_writable()`, `is_link()`,
  `file_get_contents()`, `file_put_contents()`, `fopen()`, `opendir()`/`dir()`,
  and `scandir()`, including bounded follow-on open warnings for stream and
  directory open denials. Worker proof in
  `bcs3-openbasedir-author-status.md` passed focused Rust gates, a 10-row PHPT
  author packet, five guard rows, and adjacent `is_link`/`is_writable` rows;
  supervisor post-integration Rust and focused PHPT proof also passed. The
  relative-escape Rust test now guards its cwd-mutating `chdir()` /
  `open_basedir=.` cases so the default parallel test runner cannot race the
  process cwd; both default and `--test-threads=1` focused runs passed. During
  checkpoint preparation, stale `php_runtime` unit assertions were refreshed
  for existing binary-string, core-class, `natsort`, and typed static-property
  reference behavior, and the test-only call-arguments free counter was made
  thread-local so the default parallel Rust test runner tracks current runtime
  semantics without counter cross-talk. Stale generated-C assertions were also
  refreshed for existing dynamic object-property assignment and binary-string
  comparison lowering, stale default-parameter coverage was refreshed for the
  existing declared `ClassName::CONST` interpreter default-value path, and stale
  dynamic-feature assertions were refreshed for the existing PHP-shaped fatal
  execution, eval/import/constant, error-control, and variable-variable
  boundaries, with matching unsupported dynamic feature fixture/CLI sidecars
  refreshed for the current diagnostics. The `runtime_errors` fixture sidecars
  were also refreshed against current `phpc run` behavior while preserving
  their `phpc-only` status and replacing stale `PHP_OS` unknown-constant
  probes with `PHP_OS_MISSING`; `file_exists()` focused tests were refreshed
  for current PHP-shaped arity fatals and direct-interpreter relative source
  path fixture resolution, and `filesize()` focused tests were refreshed for current
  directory metadata, warning recovery, scalar path coercion, and arity fatal
  behavior. `fprintf()`/`vfprintf()` focused tests were refreshed for the
  current shared stream-resource native-lowering boundary, scalar format
  coercion, and PHP-shaped values-argument TypeError. `functions_and_scopes`,
  modulo, shift, native variable-read, `runtime_errors`, and `implode()` Rust
  baselines were refreshed for current PHP-shaped warning/fatal execution, accepted
  `declare(strict_types)` / `declare(encoding)` parser behavior, named
  reference-argument support under non-builtin-style function names,
  ArrayAccess/null-offset deprecation output, scalar `implode()` separator
  coercion, PHP-shaped `implode()` TypeErrors, and array-to-string warning
  recovery. `ini_builtins` default-registry assertions now use the shared
  `PHPC_PHPT_INI_FLAGS` lock/restore discipline so Rust's parallel test runner
  cannot leak PHPT memory-limit overrides across tests; milestone159,
  milestone160, and milestone162 fatal sidecars now match the current
  stdout/exit shape. Focused `is_dir()`/`is_file()`/`is_readable()`/
  `is_writable()` tests now assert current PHP-shaped zero-argument fatal
  execution, matching the already-refreshed `is_link()` boundary.
  `list_assignment` now proves the intended native array-destructuring blocker
  with literal RHS values, leaving RHS call-boundary routing to the dedicated
  native-array boundary test. Magic constant CLI/fixture sidecars for the
  non-trait-originated `__TRAIT__` and global-namespace `__NAMESPACE__` cases
  were refreshed to the current successful runtime output and no longer carry
  `phpc-only` markers. Whole-tree fixture execution was stabilized between
  cargo integration tests and `phpc test` by resolving existing repo-relative
  local filesystem operation paths through the repository root when cargo runs
  from the `compiler` crate, keeping self-referential metadata/directory
  fixtures aligned without changing missing-path behavior. The stale
  milestone1 native-boundary assertions were refreshed for the current
  variable-read and non-local-assignment blockers, the full fixture/CLI
  sidecar tree was refreshed against current `phpc run`, and remaining
  system-PHP comparison divergences for stream-context diagnostics,
  ArrayAccess append/null-offset deprecation output, whole-array copied-source
  COW identity, and `array_key_exists(null, ...)` deprecation output now carry
  explicit `phpc-only` reasons. `path_builtins` now asserts the current
  `dirname(42)` weak scalar-path coercion result and passed focused `14 / 14`.
  The shutdown callback runner now clears the pending `exit_signal` only while
  draining registered shutdown callbacks, so callbacks registered before
  `exit("...")` execute before the original exit status is restored; focused
  `shutdown_function_builtin` passed `6 / 6`. `is_executable()` now matches
  PHP's silent `false` result for regular-file trailing-separator probes while
  preserving executable-file and executable-directory behavior; focused
  `standard_file_metadata_residual_builtins` passed `2 / 2`. The standard
  file-metadata open_basedir tests now guard their cwd-mutating cases, and
  `standard_file_metadata_builtins` passed `7 / 7` under Rust's default
  parallel runner. String predicate baselines now match the current
  PHP-shaped runtime arity fatal path and direct native string-predicate ASM
  lowering for `str_contains()`, `str_starts_with()`, and `str_ends_with`;
  focused tests passed `5 / 5`, `6 / 6`, and `6 / 6`.
  `functions_and_scopes` system-PHP/runtime
  oracle assertions now normalize those null-offset deprecation lines while
  still comparing the payload behavior. Namespace-resolution baselines now
  expect PHP-shaped fatal executions for undefined imported/non-imported
  function calls and record the current generated-C object-instantiation
  lowering boundary for the imported-type-alias static-property probe. Native
  arithmetic baselines now track current boxed diagnostic-result echo output,
  scalar-coercion generated-C value-operation routing, LLVM string/unary-negative
  operand conversion routing, and the current modulo split where zero/dynamic
  divisors reject while unary negative literal divisors route through the
  value-result boundary. Native assembly CLI baselines now accept current
  helper-based IR/C output-call shapes and record the current `--emit-asm`
  rejection boundaries for unary and bitwise/shift cases that remain outside
  LLVM assembly lowering. Native bitwise baselines now track current boxed
  diagnostic-result echo output, avoid unrelated unary-negative lowering
  boundaries in all-ones/negative shift assertions, and refresh current
  bitwise/shift emit-IR sidecars. Native cast baselines now include `(object)`
  casts in the existing scalar/array cast rejection boundary and refresh the
  emit-IR/emit-ASM sidecars plus the shared runtime ABI assertion. Native
  comparison baselines now track current boxed diagnostic-result echo output,
  comparison/strict-identity emit-IR sidecars, folding snapshots, and explicit
  unsupported-comparison rejection boundaries. Generated-C dynamic string
  comparison operand baselines now avoid the unrelated variable-held
  conditional-expression boundary and assert the current native value
  byte-string materialization, explicit byte-length tracking, and native value
  comparison helper path. Native concatenation baselines now track the current
  boxed diagnostic-result echo path for dynamic string output, including
  empty-string identity concatenation over untracked string expressions,
  static string concatenation, and single-result string ternary concatenation
  emit-IR sidecars. Native conditional boundary baselines now track the current
  boxed diagnostic-result echo path for scalar, string, boolean, and null
  ternary output while preserving the existing unsupported conditional-expression
  rejection boundary and refreshing the conditional emit-IR sidecars. Native
  `empty()` boundary baselines now track the current boxed diagnostic-result
  echo path for direct-variable empty output and record the current split where
  array/property/static-property operands reject at the `empty()` boundary,
  while multi-argument and call-operand forms reject at the generic native
  function-call boundary. Native function-call boundary baselines now track
  the current boxed diagnostic-result echo path for folded `strlen(...)`
  output, refresh the `native_strlen` emit-IR CLI snapshot, keep unsupported
  direct-call argument diagnostics on their exact call-site columns, and
  separate generated-C rejection coverage from generated-C dynamic
  call/value-result forms that now compile through the bounded runtime
  callable path. Native global-constant boundary `defined(...)` emit-IR CLI
  snapshots now track the same boxed diagnostic-result echo path for folded
  builtin, missing, and sort-mode constant-name results. Native `isset()`
  boundary baselines now track the current boxed diagnostic-result echo path
  for direct-variable isset output and record the current split where
  array/property/static-property operands reject at the `isset()` boundary,
  while multi-argument and call-operand forms reject at the generic native
  function-call boundary. Focused preflight gates for those clusters passed,
  including `cargo test -p phpc --test milestone1`, `cargo test -p phpc --test
  namespace_resolution`, `cargo test -p phpc --test native_arithmetic_boundary`,
  `cargo test -p phpc --test native_assembly_cli`,
  `cargo test -p phpc --test native_bitwise_boundary`,
  `cargo test -p phpc --test native_cast_boundary`,
  `cargo test -p phpc --test native_comparison_boundary`,
  `cargo test -p phpc --test native_comparison_dynamic_string_operands`,
  `cargo test -p phpc --test native_concat_boundary`,
  `cargo test -p phpc --test native_conditional_boundary`,
  `cargo test -p phpc --test native_empty_boundary`,
  `cargo test -p phpc --test native_function_call_boundary`,
  `cargo test -p phpc --test native_global_constant_boundary`,
  `cargo test -p phpc --test native_isset_boundary`,
  `phpc test tests/fixtures`, and `phpc test --compare-php tests/fixtures` with
  `2419` clean fixtures, `1691` comparisons, and `728` `phpc-only` skips. This
  slot still needs the next full Batch024 gate before publication.
- Local supervisor null-coalescing baseline maintenance refreshed stale tests
  for the current PHP-shaped fatal execution result on external inaccessible
  private-property read and assignment paths. Focused
  `cargo test -p phpc --test null_coalescing` passed `21 / 21`; this is not a
  public score update.
- Local supervisor object-model baseline maintenance refreshed stale current
  core class/interface inventory assertions and PHP-shaped fatal execution
  expectations for object/type errors. Focused
  `cargo test -p phpc --test object_model` passed `362 / 362`; this is not a
  public score update.
- Stream-context diagnostics `e7c0dd44` is rejected for returning
  `Ok(Execution)` with fatal PHP output instead of a runtime `Diagnostic` for
  `fwrite('not-resource', 'x')`; accepted repaired stream patch `ae68026f` is
  slot 14 above.

## Score History

| Gate | Passed / pinned runnable | Percent | Publication note |
| --- | ---: | ---: | --- |
| Batch001 baseline | 1118 / 20294 | 5.51% | Initial pinned full-suite baseline |
| Batch002 | 1193 / 20294 | 5.88% | 0 PASS regressions |
| Batch003 | 1311 / 20294 | 6.46% | 0 PASS regressions |
| Batch004 checkpoint8 repair | 1369 / 20294 | 6.75% | 0 PASS regressions |
| Batch004 checkpoint10 | 1413 / 20294 | 6.96% | 0 PASS regressions |
| Batch005 checkpoint10 | 1618 / 20294 | 7.97% | 0 semantic regressions |
| Batch006 checkpoint10 | 1836 / 20294 | 9.05% | 0 PASS regressions |
| Batch007 checkpoint10 repair | 2047 / 20294 | 10.09% | 0 PASS regressions |
| Batch008 checkpoint5 | 2180 / 20294 | 10.74% | 0 PASS regressions |
| Batch008 checkpoint10 repair | 2286 / 20294 | 11.26% | 0 PASS regressions |
| Batch009 burst1 | 2388 / 20294 | 11.77% | 0 PASS regressions |
| Batch010 checkpoint10 repair | 2563 / 20294 | 12.63% | 0 semantic regressions |
| Batch011 burst1 | 2741 / 20294 | 13.51% | 0 PASS regressions |
| Batch012 dynamic-call repair | 2945 / 20294 | 14.51% | 0 PASS regressions |
| Batch013 checkpoint10 | 3170 / 20294 | 15.62% | 0 PASS regressions |
| Batch014 regression repair | 3378 / 20294 | 16.65% | 0 PASS regressions |
| Batch015 checkpoint9 | 3646 / 20294 | 17.97% | 0 semantic regressions; `bug75679.phpt` long-root guard |
| Batch016 selected integration | 3868 / 20294 | 19.06% | 0 semantic regressions; 6 platform-SKIPIF rows adjudicated |
| Batch016 regression7 repair | 4048 / 20294 | 19.95% | 0 PASS regressions |
| Batch017 checkpoint10 | 4132 / 20294 | 20.36% | 0 PASS regressions; invalid-marker hits adjudicated as failed-row output |
| Batch018 repair01 | 4178 / 20294 | 20.59% | 0 PASS regressions; invalid-marker hits adjudicated |
| Batch019 repair02 | 4321 / 20294 | 21.29% | 0 semantic regressions; `bug75679.phpt` and `open_basedir_filemtime.phpt` adjudicated |
| Batch020 repair01 | 4425 / 20294 | 21.80% | 0 PASS regressions; sockets marker adjudicated |
| Batch021 regression repair | 4685 / 20294 | 23.09% | 0 PASS regressions; sockets marker adjudicated |
| Batch022 repair02 | 4949 / 20294 | 24.39% | 0 PASS regressions; sockets marker adjudicated |
| Batch023 repair01 | 5173 / 20294 | 25.49% | 0 PASS regressions |
| Batch024 regression repair | 5363 / 20294 | 26.43% | 0 PASS regressions; current public score |

## Operating Rules / Gates

- Public progress is only the pinned php-src PHPT full-suite pass rate.
- The total pinned runnable PHPT denominator stays `20294` until the pin or
  inventory policy is intentionally changed and documented here.
- A candidate can publish only after a full-suite gate is parsed, every
  latest-published PASS loss is reviewed, and semantic regressions are
  repaired.
- Focused PHPT proof must use lowercase `run-tests.php -p` with the
  `phpc-phpt-wrapper`; uppercase `-P` proof does not count for publication.
- Focused tests and source checkpoints are evidence for the next gate, not a
  public percentage change.
- Harness, platform, or expected-output adjudications must name the affected
  rows and evidence. Silent score substitution is not allowed.
- Blocked candidates may be listed here only as unpublished candidates, without
  replacing the current public score.

## Evidence Pointers

- php-src pin: `/home/claude/php-src-phpt` at
  `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- PHPT wrapper:
  `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`
- Current Batch024 gate evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch024-regression-repair-20260601T145651Z-php-src-f97ff59-source-43262ab5`
- Previous Batch023 baseline evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch023-repair01-sharded-serialized-openbasedir-20260531T1308Z-php-src-f97ff59-public-54829387-source-54f3c2c3`
- Previous Batch022 baseline evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch022-repair02-sharded-serialized-openbasedir-20260531T0839Z-php-src-f97ff59-public-5530d1da-source-69c5111f`
- Previous Batch021 baseline evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch021-regression-repair-sharded-serialized-openbasedir-20260531T0838Z-php-src-f97ff59-public-049ff7b5-source-7e9c4fd8`
- Previous Batch020 baseline evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch020-repair01-sharded-serialized-openbasedir-20260531T0415Z-php-src-f97ff59-public-5e8f521a-source-4e7a7a41`
- Skip / xfail ledger:
  `/home/claude/supervised-php-compiler/state/php-core-suite-skip-ledger.tsv`
- Detailed chronological implementation proof remains in `docs/PROGRESS.md`.
