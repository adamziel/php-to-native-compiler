# Project Goal

## Slash Goal Objective

Build this repository into a real, honest PHP-to-native compiler in stable Rust
with a long-term target of full PHP and WordPress compatibility. The target
includes the difficult parts: references, copy-on-write, dynamic lookup,
multi-file execution, object semantics, standard-library and extension behavior,
database and filesystem integration, web/SAPI-like runtime state, and native
runtime integration.

The path to that target remains evidence-driven: advance one small verified PHP
feature, compatibility harness, runtime semantic, or native-lowering slice at a
time; keep `phpc run` and `phpc compile` boundaries explicit; keep tests green;
and never claim support without executable code, CLI coverage, documentation,
and named unsupported edge cases.

## Product Target

The project should become a useful PHP compiler/runtime toolchain, not a demo.
Its first durable target is a documented PHP subset that can:

- parse PHP source with a real lexer/parser, not regex parsing;
- execute supported programs through `phpc run` with PHP-shaped boxed runtime
  values;
- compile a smaller but steadily growing supported subset through
  `phpc compile --emit-ir` and `phpc compile --emit-asm`;
- reject unsupported PHP constructs with stable parse, runtime, or codegen
  diagnostics instead of emitting misleading output;
- compare supported interpreter behavior against system PHP where practical;
- preserve accurate support documentation for every supported and unsupported
  feature.

Long term, the compiler should support enough PHP semantics to run real PHP
applications, including WordPress core, representative plugins, themes, and
CLI/admin/front-end request flows. Dynamic PHP features may use interpreter or
runtime fallback zones, but those fallback zones must be explicit and tested,
and they must remain stepping stones toward compatibility rather than excuses to
avoid difficult semantics.

## What We Have Been Doing

The work so far has been building the substrate for compatibility without lying
about it. That means:

- a real lexer/parser instead of regex parsing;
- a PHP-shaped runtime value model for scalars, arrays, constants, and a minimal
  object/class slice;
- an interpreter path that can execute a growing tested subset through
  `phpc run`;
- a native path that deliberately compiles less than the interpreter and rejects
  unsupported lowering before emitting misleading IR or assembly;
- fixture and system-PHP comparison infrastructure so new behavior can be
  checked against PHP rather than guessed;
- stable unsupported diagnostics so missing features are visible work items,
  not silent miscompilations.

This is not yet "a PHP compiler that runs WordPress." It is the foundation that
lets the project move toward that goal while preserving correctness.

## Compatibility North Star

The compatibility goal has three tiers.

The goal is complete only when the repository can prove these outcomes with
committed commands and fixtures:

- PHP language conformance suites are pinned, runnable, and tracked by PHP
  version, with expected deviations documented.
- `phpc run` can execute multi-file PHP applications with namespaces,
  autoloading, dynamic lookup, references, copy-on-write, objects, exceptions,
  standard library behavior, and extension-backed services sufficient for real
  applications.
- `phpc compile` can either produce native executable behavior for a documented
  subset or reject unsupported constructs before lowering with stable
  diagnostics; no native path may claim WordPress compatibility by delegating to
  untracked interpreter behavior.
- A committed WordPress target can bootstrap, load plugins/themes, exercise
  representative front-end/admin/REST/WP-CLI flows, and compare observable
  behavior against stock PHP under documented host assumptions.
- The project can explain every remaining fallback zone as an explicit
  compatibility strategy with tests, not as an undocumented escape hatch.

### Tier 1: PHP Language Compatibility

Support the language behavior needed by current PHP branches, while keeping
version-specific behavior explicit. As of 2026-05-14, php.net lists PHP
8.2-8.5 as supported branches, with 8.2 in security support and 8.3-8.5 in the
active/security lifecycle. The project should track this moving target in docs
and tests instead of freezing an obsolete PHP version.

Required work includes:

- full expression, statement, declaration, namespace, include/require, and
  autoload semantics;
- references, aliasing, copy-on-write, foreach-by-reference, and mutation
  ordering;
- PHP object execution beyond the current public instance-method slice:
  constructors, non-public/static/dynamic methods, visibility,
  inheritance, interfaces, traits, enums, anonymous classes, static members,
  class constants, magic methods, and object identity;
- exceptions, errors, warnings, notices, stack traces, output buffering, and
  partial-output behavior;
- attributes, typed properties, type declarations, union/intersection types,
  nullable types, `strict_types`, generators, closures, arrow functions,
  first-class callables, named arguments, variadics, splat/unpack behavior, and
  modern control constructs such as `match`;
- source mapping and version-specific diagnostics that can be compared against
  system PHP.

### Tier 2: PHP Runtime, Standard Library, And Extensions

Support the runtime services real applications rely on, not only syntax.

Required work includes:

- core standard library behavior across strings, arrays, filesystem, streams,
  dates, regex, filters, sessions, serialization, hashes, password APIs, and
  reflection;
- extension slices used by real applications, with focused support docs for
  `json`, `mbstring`, `intl`, XML/DOM/SimpleXML, `curl`, `openssl`, `zip`,
  `zlib`, GD/ImageMagick boundaries, PDO/MySQLi, SQLite, opcache-like
  assumptions, and cache integrations where applicable;
- superglobals, request/server environment, cookies, uploaded files, headers,
  response status, ini settings, locale/timezone behavior, and stream wrappers;
- exact enough error, warning, coercion, and deprecation behavior to pass
  compatibility fixtures.

### Tier 3: WordPress Compatibility

Treat WordPress as an explicit application target, not an implied side effect.
As of 2026-05-14, WordPress.org recommends PHP 8.3 or greater, MariaDB 10.6+
or MySQL 8.0+, and HTTPS for a modern host baseline; it also documents legacy
runtime compatibility down to PHP 7.2.24+ for current WordPress. The compiler
goal is not met until this repo can run committed WordPress compatibility
targets with documented environment assumptions.

Required work includes:

- a committed WordPress compatibility harness using a pinned WordPress core
  version, starting with WordPress 6.9 as the current compatibility table row
  to track, then tracking later stable core deliberately;
- bootstrap/load tests for `wp-settings.php`, plugin/theme loading, hooks,
  filters, shortcode/block registration paths, REST routing, admin/front-end
  request setup, and WP-CLI-shaped execution where feasible;
- filesystem, database, HTTP, JSON, XML, mbstring/intl, password/hash,
  serialization, sessions/cookies, object cache, and options/transients
  behavior required by WordPress core;
- compatibility fixtures for representative plugins and themes after core
  bootstrap is credible;
- documented fallback policy for dynamic extension or host integrations that
  cannot yet be native-compiled.

External target references checked on 2026-05-14:

- PHP supported branches: https://www.php.net/supported-versions.php
- WordPress host requirements: https://wordpress.org/about/requirements/
- WordPress PHP compatibility table:
  https://make.wordpress.org/core/handbook/references/php-compatibility-and-wordpress-versions/

## Missing Work Reconciled

The major missing areas are not side quests. They are the compatibility roadmap.

For full PHP and WordPress compatibility, the highest-risk missing systems are:

1. Multi-file program structure: include/require, namespaces, aliases,
   autoloading, source mapping, and path resolution.
2. PHP value semantics: references, aliasing, copy-on-write, by-reference
   parameters/returns, and mutation ordering.
3. Object semantics: executable static dispatch, late static binding, typed and
   default properties, magic methods, interfaces, traits, enums, cloning,
   destructors, reflection, and exact visibility behavior.
4. Error and control behavior: PHP-shaped exceptions, warnings/notices,
   recoverable errors, output buffering, stack traces, and partial stdout.
5. Runtime services: standard-library and extension behavior for filesystem,
   streams, JSON, XML, HTTP, database, encoding, crypto, date/time, sessions,
   cookies, superglobals, and ini/environment state.
6. Native execution: runtime ABI, linked binaries, boxed values, stack frames,
   heap/lifetime management, helper calls, and native diagnostics.

- Native execution is missing linked binaries, runtime ABI, boxed values in
  generated code, stack frames, heap management, runtime helper calls,
  process-level execution semantics, and native diagnostics.
- The interpreter is missing large parts of PHP syntax and semantics:
  namespaces, include/require breadth beyond the first local
  `require`/`require_once` slice, autoload, references, copy-on-write, closures,
  generators, exceptions, type declarations, attributes, destructuring, spreads,
  resources, and exact warning/recovery behavior.
- Arrays are interpreter-supported for a growing subset but still lack full PHP
  copy-on-write/reference behavior, by-reference iteration, recursive/nested
  aliasing semantics, object/resource edge cases, and native lowering.
- Objects/classes are still minimal: public instance method calls, same-class
  private instance method calls, protected same-class/child instance method
  calls, single-parent metadata, inherited method lookup, inherited
  public/protected/private instance property slots, child-context protected
  property access, private-parent property redeclaration as a separate child
  slot, compatible public/protected inherited property redeclarations sharing
  one runtime slot, inherited property redeclaration diagnostics, and
  public/inherited public instance `__construct` execution with scoped
  `$this`, plus narrow `ClassName::class`, `self::class`, and `parent::class`
  resolution and narrow class constants through `ClassName::CONST`,
  `self::CONST`, `parent::CONST`, and late-bound `static::CONST`, plus narrow
  static properties through `ClassName::$prop`, `self::$prop`,
  `parent::$prop`, and late-bound `static::$prop` with direct reads/writes,
  read-modify-write, `isset`/`empty`, `??`, `??=` forms, and stable diagnostics
  for PHP-forbidden static-property `unset(...)`, plus visible static-method
  dispatch through `ClassName::method(...)`, `$object::method(...)`,
  `$className::method(...)`, `self::method(...)`, `parent::method(...)`, and
  late-bound `static::method(...)`, now execute through `phpc run`, but
  interfaces, traits, enums, object-receiver static properties/constants,
  non-static dynamic static dispatch, broader class constant semantics,
  cloning, magic methods, dynamic method/property names, typed property
  compatibility and instance property defaults, broader constructor/inheritance
  rules, and native object lowering are still missing.
  The interpreter object-handle identity slice covers assignment,
  argument/return, array, foreach, strict identity, `spl_object_id`, and a
  current-subset `spl_object_hash`, but clone/destruction/reuse and full object
  execution are still missing.
- Standard-library and extension coverage is broad only in selected array and
  scalar areas; real app compatibility needs filesystem, streams, regex, JSON,
  XML, HTTP, database, crypto, encoding, date/time, reflection, sessions, and
  environment APIs.
- WordPress compatibility needs a pinned harness, bootstrap fixtures, database
  and filesystem integration strategy, hook/filter execution confidence,
  extension coverage, and plugin/theme smoke targets.
- The support matrix must keep distinguishing four things: parsed syntax,
  `phpc run` interpreter support, native IR/assembly support, and future
  native executable support.

## Current Baseline

The repository already has a working Rust workspace with:

- `php_runtime`: boxed PHP-shaped values for scalars, ordered arrays, class
  metadata, and minimal object values;
- `phpc`: lexer, parser, AST, interpreter, CLI, fixture runner, and narrow LLVM
  IR/assembly emission;
- `phpc run`: broad Milestone 1+ interpreter support for scalars, arrays,
  loops, functions, constants, selected builtins, minimal objects, and stable
  unsupported diagnostics;
- `phpc compile --emit-ir` and `--emit-asm`: intentionally narrower native
  lowering, mostly straight-line static scalar output and assignment behavior,
  plus explicit rejection coverage for unsupported constructs;
- fixture coverage under `tests/fixtures`, optional system PHP comparison, and
  operational automation under `tools/` and `docs/OPERATIONS.md`.

Read these before changing behavior: `AGENTS.md`, `docs/PROGRESS.md`,
`docs/ARCHITECTURE.md`, `docs/SUPPORT.md`, `docs/COMPATIBILITY.md`,
`docs/NATIVE_RUNTIME_ABI.md`, `docs/WORDPRESS_COMPATIBILITY.md`, `README.md`,
and `docs/LOOP_MEMORY.md` when running unattended loop automation.

## Definition of Done

A feature or milestone is complete only when all of these are true:

- Implementation code exists in the compiler, runtime, or CLI.
- Unit, integration, or fixture tests cover the behavior.
- A CLI exercise path exists through `phpc run`, `phpc compile --emit-ir`,
  `phpc compile --emit-asm`, or an explicit CLI snapshot.
- Documentation in `README.md`, `docs/SUPPORT.md`, and any focused design doc
  is accurate.
- `docs/PROGRESS.md` records the change after meaningful behavior changes.
- Unsupported edge cases are named directly.
- Native lowering either implements the behavior honestly or rejects it with a
  specific codegen diagnostic.
- The relevant focused tests pass during the slice; checkpoint or integration
  batches run `tools/run-tests.sh` unless a blocker is documented.

## Engineering Constraints

- Use stable Rust for compiler and runtime code.
- Prefer small, correct PHP subsets over broad placeholder claims.
- Keep dynamic PHP features as explicit runtime fallback zones.
- Runtime values must remain PHP-shaped: boxed values first, ordered int/string
  keyed arrays, references and copy-on-write when implemented.
- Do not replace the parser with regular-expression parsing.
- Add crates only when they materially reduce work.
- Do not silently widen support in docs before tests prove it.
- Do not collapse interpreter support and native lowering into one claim:
  `phpc run` may support a construct while `phpc compile` still rejects it.
- Keep the `cc -S` assembly fallback documented as a bootstrap fallback, not the
  primary backend.

## Execution Lanes

The goal is intentionally broad, so day-to-day work should be split into
independent lanes. Each lane owns one milestone at a time and should use a
separate git worktree when running in parallel with other workers.

Lanes:

- Parser lane: lexer, parser, AST shape, parse diagnostics, and syntax-focused
  fixtures. This lane must not widen runtime or native support claims unless
  another lane implements and tests the behavior.
- IR/lowering lane: `phpc compile --emit-ir`, `phpc compile --emit-asm`,
  native codegen diagnostics, deterministic backend/fallback snapshots, and
  native support documentation.
- Runtime lane: `php_runtime`, interpreter semantics for `phpc run`, boxed
  value behavior, builtins, object/array semantics, and system PHP comparison
  fixtures.
- Compiler output lane: CLI behavior, emitted artifact shape, backend
  selection/failure behavior, future link/run output, stdout/stderr/exit
  contracts, and operational scripts.
- Tests/docs lane: fixture organization, focused/full test gates, support
  matrix accuracy, progress records, operations docs, and unsupported-edge
  naming.

Parallel work rules:

- One worktree per lane and one active milestone per worker. Avoid multiple
  workers editing the same source files, fixture directories, or docs section.
- Each worker records its lane and milestone in `docs/PROGRESS.md` before
  handing off or checkpointing.
- A worker may run focused tests for its lane during the slice, but a checkpoint
  batch must run the serialized full gate or document a blocker.
- Cross-lane changes require an explicit handoff note naming the affected lane,
  files, tests, and remaining risks.
- Shared files such as `docs/SUPPORT.md`, `docs/ARCHITECTURE.md`,
  `README.md`, `docs/NEXT_TASKS.md`, and `compiler/src/codegen.rs` require
  extra care: keep edits scoped, avoid broad rewrites, and do not overwrite
  another lane's unmerged work.

## Milestone Plan

### 0. Keep The Baseline Honest

Maintain the existing interpreter, native lowering, docs, and test suite as the
floor for all work.

Exit criteria:

- `tools/run-tests.sh` passes.
- `docs/NEXT_TASKS.md` has a clear next unchecked task.
- `docs/SUPPORT.md` distinguishes `phpc run` support from native compile
  support.
- New diagnostics are stable enough for fixture snapshots.

### 1. Finish Native Scalar Lowering

Grow `phpc compile --emit-ir` and `--emit-asm` from static scalar output toward
a coherent straight-line scalar subset.

Scope:

- static scalar reassignment and overwrite coverage;
- native scalar arithmetic for supported int/float/literal subsets;
- native comparison, logical, bitwise, unary, and conditional expression slices
  only when PHP coercion and error boundaries are explicit;
- compile-time diagnostics for unsafe static cases and explicit rejection for
  dynamic cases until runtime-backed checks exist.

Exit criteria:

- Lowered scalar programs have CLI assembly coverage and fixture coverage.
- Unsupported scalar cases fail before backend invocation with specific codegen
  diagnostics.
- Generated IR/assembly tests use deterministic assertions rather than
  platform-specific assembly text.

### 2. Link Runtime-Backed Native Execution

Move native compilation beyond standalone emitted assembly text by linking
generated code with runtime helpers.

Scope:

- define a stable ABI between generated code and `php_runtime`;
- lower boxed value creation, echo/print conversion, and selected runtime helper
  calls;
- add an executable native path only when process exit, stdout, stderr, and
  diagnostics can be tested;
- keep `--emit-ir` and `--emit-asm` useful as inspectable compiler stages.

Exit criteria:

- At least one non-trivial program can be compiled, linked, executed, and
  compared to `phpc run`.
- Runtime-backed native diagnostics are tested.
- Docs state exactly which constructs are native-executable versus
  interpreter-only.

### 3. Runtime Semantics: References and Copy-On-Write

Implement the PHP value semantics that affect real programs before expanding
syntax too far.

Scope:

- reference cells and aliasing boundaries;
- array/object copy-on-write behavior;
- function parameter and return interactions with references;
- mutation behavior for arrays, objects, foreach, and builtins;
- explicit unsupported diagnostics where exact PHP behavior remains out of
  scope.

Exit criteria:

- Aliasing and copy-on-write behavior has targeted runtime tests and CLI
  fixtures.
- Existing array/object builtins either honor the new semantics or document and
  test their temporary limits.
- Native lowering rejects or uses runtime helpers honestly.

### 4. Object Execution

Turn current class metadata and minimal object values into executable object
semantics.

Scope:

- broader method dispatch beyond public static-name instance calls;
- broader constructor semantics beyond public/inherited public instance
  `__construct` and explicit parent calls, including non-public constructor
  visibility;
- visibility enforcement for public/protected/private access;
- static methods, broader static member semantics, broader class constant
  semantics, broader late static binding, and broader class-name resolution;
- full object identity behavior including clone/destruction/reuse boundaries,
  exact `spl_object_hash` formatting, and native object handles;
- inheritance after the single-class model is stable.

Exit criteria:

- Public static-name instance method calls execute through `phpc run` with
  scoped `$this`; broader dispatch remains named and tested as unsupported.
- Constructor and visibility behavior have fixtures and stable errors.
- Introspection builtins are updated to reflect implemented object semantics.
- Native object lowering remains explicit: implemented subset or precise
  rejection.

### 5. Program Structure

Add PHP program-structure features needed by real codebases.

Scope:

- include/require resolution and execution;
- namespaces, imports, aliases, and qualified name resolution;
- autoload boundary design before automatic class loading;
- source mapping for magic constants across files;
- clear fallback policy for highly dynamic cases.

Exit criteria:

- Multi-file programs can run through `phpc run` for a documented subset.
- Include paths, repeated includes, and failure modes have CLI coverage.
- Namespace-aware functions, constants, and classes are tested.
- Unsupported autoload and dynamic resolution cases are named.

### 6. Exceptions and Error Model

Replace project-specific fatal shortcuts with more PHP-shaped error behavior
where it matters.

Scope:

- `throw`, `try`, `catch`, and `finally`;
- `Throwable`, `Exception`, and core error object boundaries;
- partial stdout and output buffering behavior;
- warnings/notices where PHP continues execution;
- stack traces or an explicit documented substitute.

Exit criteria:

- Exception control flow has fixtures for normal, thrown, caught, and finally
  paths.
- Runtime diagnostics distinguish project compiler errors from PHP runtime
  errors.
- Tests pin stdout/stderr/exit behavior.

### 7. Standard Library and Extensions

Expand builtins according to real program needs and extension plans, not by
surface-area guessing.

Scope:

- prioritize core functions used by fixtures and target applications;
- implement `json`, `mbstring`, XML, `curl`, `openssl`, database, cache, and
  compression extension slices only behind focused docs in `docs/extensions/`;
- keep unsupported extension functions and constants documented.

Exit criteria:

- Every builtin or extension slice has signature docs, type/error behavior,
  fixture coverage, and unsupported edge cases.
- Extension support claims are scoped to executable code.
- Dynamic function-call availability is tested when a builtin is callable.

### 8. Application Compatibility Targets

Choose small real PHP programs as compatibility targets once the runtime is
ready enough.

Scope:

- start with self-contained scripts using arrays, functions, classes, includes,
  and selected core builtins;
- then move to framework-shaped examples;
- move to WordPress deliberately: first bootstrap/load targets, then selected
  admin/front-end/WP-CLI flows, then representative plugin and theme fixtures;
- keep database, filesystem, HTTP, cache, extension, and host-environment
  assumptions pinned per target.

Exit criteria:

- Each target has a committed fixture or sample app.
- `phpc run` behavior is compared against system PHP where feasible.
- Native compile behavior is either supported, partially supported with
  documented fallback, or rejected with exact diagnostics.
- WordPress targets record the WordPress version, PHP target version, required
  extensions, database/filesystem assumptions, and known unsupported behavior.

### 9. Compatibility Harnesses

Make compatibility measurable before claiming it.

Scope:

- add a PHP-version compatibility manifest that names target PHP branches,
  feature groups, fixtures, expected comparison mode, and known unsupported
  deltas;
- add a path for importing or mirroring small php-src-style behavioral tests
  where licensing and scope allow;
- add a pinned WordPress core compatibility harness that starts with parse/load
  inventory, then bootstrap fixtures, then request/CLI smoke tests;
- report compatibility status as data, not prose: pass, fail, skipped with a
  named unsupported reason, or not-yet-covered.

Exit criteria:

- A single command can run the committed PHP compatibility smoke suite.
- A single command can run the committed WordPress compatibility smoke suite
  without requiring undocumented host state.
- Failures distinguish compiler bugs from intentionally unsupported gaps.
- `docs/SUPPORT.md` and any generated manifest agree on support status.

### 10. Release-Quality Toolchain

Make the compiler usable outside the test suite.

Scope:

- stable CLI UX and exit codes;
- structured diagnostics with source spans;
- installable binary packaging;
- reproducible backend discovery;
- benchmark and smoke-test commands;
- contributor docs that explain how to add a PHP feature correctly.

Exit criteria:

- A user can install/build `phpc`, run examples, inspect support docs, and
  understand unsupported failures.
- CI or local automation runs the full test suite and representative CLI
  smoke tests.
- Release notes distinguish interpreter, IR, assembly, and native-executable
  support.

## Near-Term Queue

Start from `docs/NEXT_TASKS.md`; it is the source of truth for checked and
unchecked work. Current state:

- Milestones 615-624 are closed and checkpointed.
- Milestone 625: compiler-output lane, choose the next deterministic CLI
  artifact or backend contract coverage target from existing native output
  behavior.
- Milestone 626: IR/lowering lane, choose the next narrow native IR or lowering
  refinement, or a precise rejection boundary, from already documented
  interpreter behavior.
- Milestone 627: parser lane, choose the next small syntax or parse-diagnostic
  boundary without widening runtime or native support claims.
- Milestone 628: runtime lane, choose the next small array/object builtin
  refinement from documented unsupported gaps.
- Milestone 629: tests/docs lane, keep the lane queue, focused-test policy,
  support matrix, and progress records current after the active implementation
  batch.
- Milestones 630-634 should begin the compatibility-harness turn: convert the
  full-PHP/WordPress north star into manifests, smoke fixtures, and explicit
  pass/fail/skipped reporting instead of more prose-only planning.

Prefer depth in one documented subset over breadth across unimplemented PHP
features.

## Verification Commands

Use focused commands while developing:

```sh
cargo test -p php_runtime
cargo test -p phpc
cargo run -p phpc -- test
cargo run -p phpc -- test --compare-php tests/fixtures/milestoneN
cargo run -p phpc -- run tests/fixtures/milestoneN/example.php
cargo run -p phpc -- compile tests/fixtures/milestoneN/example.php --emit-ir
cargo run -p phpc -- compile tests/fixtures/milestoneN/example.php --emit-asm
```

Use the full gate before checkpointing or after an integration batch that
combines lane work:

```sh
tools/run-tests.sh
```

When running unattended loop automation, checkpoint only through:

```sh
tools/checkpoint.sh "short accurate message"
```

## Non-Goals For Now

- Unproven full PHP compatibility claims.
- Unproven WordPress compatibility claims.
- Zend extension loading.
- Regex parser rewrites.
- A custom x86-64 backend before the LLVM/runtime path is justified.
- Silent native compilation of constructs that only the interpreter supports.
- Broad standard-library stubs that return placeholder values.
