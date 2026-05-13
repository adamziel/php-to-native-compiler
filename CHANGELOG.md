# Changelog

## Unreleased

- Added explicit `phpc compile --emit-asm` CLI coverage for successful
  selected and fallback backend discovery probes that write stdout and stderr
  diagnostics. The Milestone 206 fixture runs as a lowerable scalar echo/print
  program, while assembly CLI tests invoke `--emit-asm` with temporary PATHs
  exposing deterministic fake `clang`, `llc`, and `cc` tools whose successful
  `--version` probes emit diagnostics before selected or fallback assembly
  emission succeeds. The committed snapshots prove successful probe output is
  ignored. Bundled toolchains, assembly linking/execution, full
  backend-specific discovery output semantics, exact native error objects, and
  broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for selected and
  fallback backend discovery probe argument validation. The Milestone 205
  fixture runs as a lowerable scalar echo/print program, while assembly CLI
  tests invoke `--emit-asm` with temporary PATHs exposing deterministic fake
  `clang`, `llc`, and `cc` tools that require an exact single-argument
  `--version` probe before accepting selected or fallback assembly emission.
  The committed snapshots prove the current backend discovery probe argument
  contract. Bundled toolchains, assembly linking/execution, full
  backend-specific discovery semantics, exact native error objects, and
  broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for selected and
  fallback backend argument validation. The Milestone 204 fixture runs as a
  lowerable scalar echo/print program, while assembly CLI tests invoke
  `--emit-asm` with temporary PATHs exposing deterministic fake `clang`,
  `llc`, and `cc` tools that pass discovery, validate the expected assembly
  emission argument vectors, then accept stdin and emit normalized assembly.
  The committed snapshots prove the current selected `clang`, fallback `llc`,
  and `cc -S` fallback command-line contracts. Bundled toolchains, assembly
  linking/execution, full backend-specific command-line compatibility, exact
  native error objects, and broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for fallback `llc` and
  `cc` backend input validation. The Milestone 203 fixture runs as a lowerable
  scalar echo/print program, while assembly CLI tests invoke `--emit-asm` with
  temporary PATHs exposing deterministic fake `llc` and `cc` tools that pass
  discovery, validate generated LLVM IR or generated C fallback source arrives
  on stdin with representative markers, then emit normalized assembly. The
  committed snapshots prove fallback-backend stdin handoff after LLVM backend
  fallback selection and after `cc -S` C fallback selection. Bundled
  toolchains, assembly linking/execution, backend-specific IR/C validation
  across real tools and broader lowered constructs, exact native error
  objects, and broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for selected `clang`
  backend input validation. The Milestone 202 fixture runs as a lowerable
  scalar echo/print program, while the assembly CLI test invokes `--emit-asm`
  with a temporary PATH exposing deterministic fake `clang` that passes
  discovery, validates generated LLVM IR arrives on stdin with representative
  `printf`, `main`, and `printf` call markers, then emits normalized assembly.
  The committed snapshot proves selected-backend stdin handoff for the current
  lowerable scalar subset. Bundled toolchains, assembly linking/execution,
  backend-specific IR validation across real tools and broader lowered
  constructs, exact native error objects, and broader native lowering remain
  explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for fallback `llc` and
  `cc` backend success cases that write stderr diagnostics while producing
  only whitespace assembly stdout. The Milestone 201 fixture runs as a
  lowerable scalar echo/print program, while assembly CLI tests invoke
  `--emit-asm` with temporary PATHs exposing deterministic fake `llc` and
  `cc` tools that pass discovery, accept generated input, write stderr
  diagnostics, emit only whitespace on stdout, and exit successfully. The
  committed snapshots prove stdout validation wins with stable `llc emitted
  whitespace-only assembly output` and `cc emitted whitespace-only assembly
  output` diagnostics and do not surface successful-backend stderr on invalid
  successful output after fallback selection. Bundled toolchains, assembly
  linking/execution, backend-specific assembly validation, exact native error
  objects, and broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for a selected
  `clang` backend success case that writes stderr diagnostics while producing
  only whitespace assembly stdout. The Milestone 200 fixture runs as a
  lowerable scalar echo/print program, while the assembly CLI test invokes
  `--emit-asm` with a temporary PATH exposing deterministic fake `clang` that
  passes discovery, accepts generated LLVM IR, writes stderr diagnostics, emits
  only whitespace on stdout, and exits successfully. The committed snapshot
  proves stdout validation wins with the stable `clang emitted
  whitespace-only assembly output` diagnostic and does not surface
  successful-backend stderr on invalid successful output. Bundled toolchains,
  assembly linking/execution, backend-specific assembly validation, exact
  native error objects, and broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for the selected
  `clang` backend success case that produces only whitespace assembly stdout.
  The Milestone 199 fixture runs as a lowerable scalar echo/print program,
  while the assembly CLI test invokes `--emit-asm` with a temporary PATH
  exposing deterministic fake `clang` that passes discovery, accepts generated
  LLVM IR, emits only whitespace on stdout, and exits successfully. The
  committed snapshot proves the stable `clang emitted whitespace-only assembly
  output` diagnostic before LLVM backend fallback selection. Bundled
  toolchains, assembly linking/execution, backend-specific assembly
  validation, exact native error objects, and broader native lowering remain
  explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for fallback backend
  success cases that produce only whitespace assembly stdout. The Milestone
  198 fixture runs as a lowerable scalar echo/print program, while assembly CLI
  tests invoke `--emit-asm` with temporary PATHs exposing deterministic fake
  `llc` and `cc` tools that pass discovery, accept generated input, emit only
  whitespace on stdout, and exit successfully. `phpc` now rejects that case
  with stable `llc emitted whitespace-only assembly output` and `cc emitted
  whitespace-only assembly output` diagnostics after LLVM backend fallback
  selection and after `cc -S` C fallback selection. Bundled toolchains,
  assembly linking/execution, backend-specific assembly validation, exact
  native error objects, and broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for fallback backend
  success cases that produce empty assembly stdout. The Milestone 197 fixture
  runs as a lowerable scalar echo/print program, while assembly CLI tests
  invoke `--emit-asm` with temporary PATHs exposing deterministic fake `llc`
  and `cc` tools that pass discovery, accept generated input, emit no stdout,
  and exit successfully. The committed snapshots prove the stable `llc
  emitted empty assembly output` and `cc emitted empty assembly output`
  diagnostics after LLVM backend fallback selection and after `cc -S` C
  fallback selection. Bundled toolchains, assembly linking/execution,
  backend-specific stdout/stderr guarantees, exact native error objects, and
  broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for fallback backend
  failures that exit nonzero without stderr diagnostics. The Milestone 196
  fixture runs as a lowerable scalar echo/print program, while assembly CLI
  tests invoke `--emit-asm` with temporary PATHs exposing deterministic fake
  `llc` and `cc` tools that pass discovery, accept generated input, emit no
  stderr, and exit nonzero. The committed snapshots prove the stable `backend
  exited without stderr` diagnostic detail after LLVM backend fallback
  selection and after `cc -S` C fallback selection. Bundled toolchains,
  assembly linking/execution, backend-specific stderr guarantees, exact native
  error objects, and broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for successful fallback
  backend paths that also write stderr diagnostics. The Milestone 195 fixture
  runs as a lowerable scalar echo/print program, while the assembly CLI tests
  invoke `--emit-asm` with temporary PATHs exposing deterministic fake `llc`
  and `cc` tools that pass discovery, accept generated input, emit nonempty
  assembly stdout, write stderr diagnostics, and exit successfully. The
  committed normalized snapshots prove `phpc` returns assembly from stdout
  without surfacing backend stderr after LLVM backend fallback selection and
  after `cc -S` C fallback selection. Bundled toolchains, assembly
  linking/execution, backend-specific stderr guarantees, exact native error
  objects, and broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for selected backend
  success cases that also write stderr diagnostics. The Milestone 194 fixture
  runs as a lowerable scalar echo/print program, while the assembly CLI test
  invokes `--emit-asm` with a temporary PATH exposing a deterministic fake
  `clang` that passes discovery, accepts generated LLVM IR, emits nonempty
  assembly stdout, writes a stderr diagnostic, and exits successfully. `phpc`
  now documents and tests that successful backend stderr is not surfaced and
  assembly is taken only from stdout. Bundled toolchains, assembly
  linking/execution, backend-specific stderr guarantees for every backend,
  exact native error objects, and broader native lowering remain explicit
  gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for selected backend
  success cases that produce empty assembly stdout. The Milestone 193 fixture
  runs as a lowerable scalar echo/print program, while the assembly CLI test
  invokes `--emit-asm` with a temporary PATH exposing a deterministic fake
  `clang` that passes discovery, accepts generated LLVM IR, emits no stdout,
  and exits successfully. `phpc` now rejects that case with the stable `clang
  emitted empty assembly output` diagnostic instead of accepting an empty
  assembly artifact. Bundled toolchains, assembly linking/execution,
  backend-specific stdout/stderr guarantees, exact native error objects, and
  broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for selected backend
  failures that exit nonzero without stderr. The Milestone 192 fixture runs as
  a lowerable scalar echo/print program, while the assembly CLI test invokes
  `--emit-asm` with a temporary PATH exposing a deterministic fake `clang` that
  passes discovery, accepts generated LLVM IR, emits no stderr, and exits
  nonzero. The committed snapshot pins the stable `backend exited without
  stderr` diagnostic detail. Bundled toolchains, assembly linking/execution,
  backend-specific stderr guarantees, exact native error objects, and broader
  native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for assembly backend
  discovery exhaustion. The Milestone 191 fixture runs as a lowerable scalar
  echo/print program, while the assembly CLI test invokes `--emit-asm` with a
  temporary PATH exposing fake `clang`, `llc`, and `cc` commands whose
  `--version` probes all fail. The committed snapshot pins the stable
  missing-backend diagnostic when command names exist but no candidate passes
  discovery. Bundled toolchains, assembly linking/execution,
  backend-specific discovery semantics, exact native error objects, and
  broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for assembly backend
  discovery fallback behavior. The Milestone 190 fixture runs as a lowerable
  scalar echo/print program, while the assembly CLI test invokes `--emit-asm`
  with a temporary PATH exposing a fake `clang` whose `--version` probe fails,
  a fake `llc` whose probe succeeds, and a fake `cc` that would fail if
  reached. The committed snapshot checks a normalized success summary, proving
  failed discovery probes are treated as unavailable before fallback
  selection. Bundled toolchains, assembly linking/execution, backend-specific
  discovery semantics for every tool, exact native error objects, and broader
  native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for `cc -S` fallback
  failure diagnostics. The Milestone 189 fixture runs as a lowerable scalar
  echo/print program, while the assembly CLI test invokes `--emit-asm` with a
  temporary PATH exposing only a deterministic fake `cc` that passes discovery
  and exits nonzero after accepting generated C fallback source. The committed
  snapshot pins the stable `cc failed to emit assembly` diagnostic shape
  without depending on real toolchain stderr. Bundled toolchains, assembly
  linking/execution, backend-specific stderr guarantees, exact native error
  objects, and broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for selected `llc`
  backend failure diagnostics. The Milestone 188 fixture runs as a lowerable
  scalar echo/print program, while the assembly CLI test invokes `--emit-asm`
  with a temporary PATH exposing only a deterministic fake `llc` that passes
  discovery and exits nonzero after accepting generated LLVM IR. The committed
  snapshot pins the stable `llc failed to emit assembly` diagnostic shape
  without depending on real toolchain stderr. Bundled toolchains, assembly
  linking/execution, backend-specific stderr guarantees, exact native error
  objects, and broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for the `llc`
  selected-backend path. The Milestone 187 fixture runs as a lowerable scalar
  echo/print program, while the assembly CLI test invokes `--emit-asm` with a
  temporary PATH exposing only a deterministic fake `llc`. The committed
  snapshot checks a normalized success summary rather than exact assembly text,
  proving `llc` is selected when `clang` is unavailable and before the `cc -S`
  fallback is considered. Backend-specific assembly, bundled toolchains,
  assembly linking/execution, exact native error objects, and broader native
  lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for selected backend
  failure diagnostics. The Milestone 186 fixture runs as a lowerable scalar
  echo/print program, while the assembly CLI test invokes `--emit-asm` with a
  temporary PATH exposing a deterministic fake `clang` that passes discovery
  and exits nonzero after accepting generated LLVM IR. The committed snapshot
  pins the stable `clang failed to emit assembly` diagnostic shape without
  depending on real toolchain stderr. Bundled toolchains, assembly
  linking/execution, backend-specific stderr guarantees, exact native error
  objects, and broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for the documented
  `cc -S` fallback path. The Milestone 185 fixture runs as a lowerable scalar
  echo/print program, and the assembly CLI test invokes `--emit-asm` with a
  temporary PATH that hides `clang` and `llc` while exposing only `cc`. The
  committed snapshot checks a normalized success summary rather than exact
  assembly text. Bundled toolchains, assembly linking/execution, exact native
  error objects, and broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for the current
  lowerable scalar subset when no assembly backend tools are available. The
  Milestone 184 fixture runs as a scalar echo/print program, while the assembly
  snapshot invokes `--emit-asm` with backend tools removed from `PATH` and
  records the stable missing-backend diagnostic after LLVM lowering succeeds.
  Bundled toolchains, assembly linking/execution, exact native error objects,
  and broader native lowering remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI rejection coverage for a
  representative unsupported native boundary. The Milestone 183 fixture runs
  as a small array/count program, while the assembly snapshot invokes
  `--emit-asm` with backend tools removed from `PATH` and records the stable
  LLVM array-lowering diagnostic. This pins that assembly emission exits before
  backend discovery when LLVM lowering rejects a program. Backend-independent
  native diagnostics, exact native error objects, and broader native lowering
  remain explicit gaps.
- Added explicit `phpc compile --emit-asm` CLI coverage for the current
  lowerable straight-line scalar echo/assignment subset. The Milestone 182
  fixture mirrors the scalar literal/static-variable `echo`/`print` path and
  the integration test compares a normalized success summary that checks exit
  status, nonempty assembly output, a `main` symbol, and `printf` references
  without snapshotting backend-specific assembly text. Assembly
  linking/execution, PHP zvals, native symbol-table storage,
  references/copy-on-write, exact native error objects, and broader native
  lowering remain explicit gaps.
- Added a native variable-read boundary. LLVM IR emission now rejects reads of
  variables that were not statically assigned earlier in the same
  straight-line native subset, including reads in `echo`, `print`, and
  assignment right-hand sides, with a specific codegen diagnostic until
  generated code has native symbol-table storage, undefined-variable
  diagnostics, references/copy-on-write behavior, and exact native error
  behavior; the C assembly fallback carries the same boundary for consistency.
  A Milestone 181 fixture pins the current `phpc run` undefined-variable
  runtime diagnostic, and a `phpc compile --emit-ir` CLI snapshot pins the
  native rejection. Dynamic variable variables remain an earlier lexer/parser
  boundary, and native PHP symbol tables, warning/continue behavior, exact
  native error objects, references/copy-on-write, and broader variable
  lowering remain explicit gaps.
- Added positive native scalar echo/assignment coverage for the remaining
  lowerable straight-line subset: literal `null`, booleans, integers, floats,
  and strings; direct static-variable assignments from those values; direct
  reads of previously assigned static variables; and `echo`/`print` through
  generated static `printf` calls. A Milestone 180 fixture has `phpc run`,
  system-PHP comparison, LLVM IR shape, assembly-availability, and `phpc
  compile --emit-ir` CLI snapshot coverage. Native PHP zvals, symbol-table
  storage, dynamic values, references/copy-on-write, dynamic string
  allocation, exact PHP float formatting/error objects, and broader native
  lowering remain explicit gaps.
- Added a native concatenation boundary. LLVM IR emission now rejects `.`
  before lowering operands, with a specific codegen diagnostic until generated
  code has PHP string conversion, dynamic allocation, references/copy-on-write,
  and exact native error behavior; the C assembly fallback carries the same
  boundary for consistency. A runtime fixture still proves the current
  interpreter concatenation subset, and a `phpc compile --emit-ir` CLI
  snapshot pins the native rejection. Native string conversion, dynamic
  allocation, references/copy-on-write, exact native error objects, and broader
  string lowering remain explicit gaps.
- Added a native binary-arithmetic boundary. LLVM IR emission now rejects
  `+`, `-`, `*`, `/`, and `%` before lowering operands, with a specific
  codegen diagnostic until generated code has PHP numeric coercion, dynamic
  division/modulo zero checks, modulo coercions, references/copy-on-write, and
  exact native error behavior; the C assembly fallback carries the same
  boundary for consistency. This supersedes the earlier narrow native `/` and
  `%` slices. A runtime fixture still proves the current interpreter
  arithmetic subset, and a `phpc compile --emit-ir` CLI snapshot pins the
  native rejection. Native numeric coercion, dynamic zero checks, modulo
  coercions, references/copy-on-write, exact native error objects, and broader
  arithmetic lowering remain explicit gaps.
- Added a native mutation boundary. LLVM IR emission now rejects compound
  assignment, null coalescing assignment, increment/decrement, assignment
  expressions, direct variable `unset`, and multiple-operand `unset` before
  lowering operands or mutation targets, with a specific codegen diagnostic
  until generated code has read-modify-write ordering, null-aware mutation,
  unset symbol-table effects, references/copy-on-write, and exact native error
  behavior; the C assembly fallback carries the same boundary for consistency.
  A runtime fixture still proves the current interpreter mutation subset, and
  a `phpc compile --emit-ir` CLI snapshot pins the native rejection. Native
  read-modify-write ordering, null-aware mutation, unset symbol-table effects,
  references/copy-on-write, exact native error objects, and broader native
  lowering remain explicit gaps.
- Added a native control-flow boundary. LLVM IR emission now rejects
  `if`/`elseif`/`else`, `while`, `for`, `do ... while`, `switch`, `break`, and
  `continue` before lowering conditions, bodies, cases, or loop-control flow,
  with a specific codegen diagnostic until generated code has PHP truthiness,
  branch layout, loop control flow, switch fallthrough,
  references/copy-on-write side effects, and exact native error behavior; the
  C assembly fallback carries the same boundary for consistency. A runtime
  fixture still proves the current interpreter control-flow subset, and a
  `phpc compile --emit-ir` CLI snapshot pins the native rejection. Native
  branch layout, loop control, switch fallthrough, references/copy-on-write,
  exact native error objects, and broader native lowering remain explicit gaps.
- Added a native array boundary. LLVM IR emission now rejects array literals,
  array offset reads/writes, `foreach` array iteration, array offset `unset`,
  and array builtin function calls before lowering bodies, operands,
  arguments, or callbacks, with a specific codegen diagnostic until generated
  code has native array storage layout, key normalization, copy-on-write,
  references, callback dispatch, and exact native error behavior; the C
  assembly fallback carries the same boundary for consistency. A runtime
  fixture still proves the current interpreter array subset, and a
  `phpc compile --emit-ir` CLI snapshot pins the native rejection. Native
  array storage, key normalization, callback dispatch, references/copy-on-write,
  exact native error objects, and broader native lowering remain explicit gaps.
- Added a native object/class boundary. LLVM IR emission now rejects class
  declarations, object instantiation, public property reads/writes, and object
  metadata builtins before lowering bodies, operands, or arguments, with a
  specific codegen diagnostic until generated code has native object layout,
  handles, visibility, method dispatch, and exact native error behavior; the C
  assembly fallback carries the same boundary for consistency. A runtime
  fixture still proves the current interpreter object/class subset, and a
  `phpc compile --emit-ir` CLI snapshot pins the native rejection.
  Constructors, `$this`, methods, object handles, references/copy-on-write,
  exact native error objects, and broader native lowering remain explicit gaps.
- Added a native global-constant boundary. LLVM IR emission now rejects
  built-in constants, runtime-defined constants, bare constant reads,
  top-level `const` declarations, and `define()`/`constant()`/`defined()`
  before lowering values or arguments, with a specific codegen diagnostic
  until generated code has native constant tables, source-order definitions,
  namespace-aware lookup, and exact native error behavior; the C assembly
  fallback carries the same boundary for consistency. A runtime fixture still
  proves the current interpreter global-constant subset, and a
  `phpc compile --emit-ir` CLI snapshot pins the native rejection. Namespaces,
  class constants, references/copy-on-write, exact native error objects, and
  broader native lowering remain explicit gaps.
- Added a native magic-constant boundary. LLVM IR emission now rejects
  executable magic constants `__LINE__`, `__FILE__`, `__DIR__`, and
  `__FUNCTION__` before lowering them, with a specific codegen diagnostic
  until generated code has source mapping, path canonicalization,
  function-context lowering, eval/include source interaction rules, and exact
  native error behavior; the C assembly fallback carries the same boundary for
  consistency. A runtime fixture and CLI snapshot still prove the current
  interpreter magic-constant subset, and a `phpc compile --emit-ir` CLI
  snapshot pins the native rejection. References/copy-on-write and broader
  native lowering remain explicit gaps.
- Added a native user-function declaration/return boundary. LLVM IR emission
  now rejects function declarations and return statements before traversing
  function bodies, with a specific codegen diagnostic until generated code has
  function symbol tables, stack-frame layout, default parameter binding,
  recursion guards, return-value flow, and exact native error behavior; the C
  assembly fallback carries the same boundary for consistency. A runtime
  fixture still proves current user-function declarations, default parameters,
  and returns, and a `phpc compile --emit-ir` CLI snapshot pins the native
  rejection. References/copy-on-write and broader native lowering remain
  explicit gaps.
- Added a native function-call boundary. LLVM IR emission now rejects function
  calls, including direct callable builtins outside
  `define()`/`constant()`/`defined()`, user functions, and dynamic
  string-valued calls, before lowering arguments or callees, with a specific
  codegen diagnostic until generated code has runtime call lookup, stack
  frames, arity/type diagnostics, callback dispatch, and dynamic string-call
  dispatch; the C assembly fallback carries the same boundary for consistency.
  A runtime fixture still proves the current interpreter function-call subset,
  and a `phpc compile --emit-ir` CLI snapshot pins the native rejection.
  References/copy-on-write, exact native error objects, and broader native
  lowering remain explicit gaps.
- Added a native conditional-expression boundary. LLVM IR emission now rejects
  full ternary, short ternary, and null coalescing expressions before lowering
  branches or operands, with a specific codegen diagnostic until generated code
  has PHP truthiness, null-aware lookup, and branch side-effect ordering; the C
  assembly fallback carries the same boundary for consistency. A runtime
  fixture still proves the current interpreter conditional-expression subset,
  and a `phpc compile --emit-ir` CLI snapshot pins the native rejection.
  References/copy-on-write, exact native error objects, and broader native
  lowering remain explicit gaps.
- Added a native bitwise/shift boundary. LLVM IR emission now rejects `&`,
  `|`, `^`, unary `~`, `<<`, and `>>` before lowering operands, with a
  specific codegen diagnostic until generated code has PHP bytewise string
  behavior, scalar-to-int coercion, negative/large shift diagnostics, and exact
  native error behavior; the C assembly fallback carries the same boundary for
  consistency. A runtime fixture still proves the current bitwise/shift subset,
  and a `phpc compile --emit-ir` CLI snapshot pins the native rejection.
  References/copy-on-write, exact native error objects, and broader native
  lowering remain explicit gaps.
- Added a native logical-operator boundary. LLVM IR emission now rejects
  `&&`, `||`, `and`, `xor`, and `or` before lowering operands, with a specific
  codegen diagnostic until generated code has PHP truthiness conversion and
  short-circuit semantics; the C assembly fallback carries the same boundary
  for consistency. A runtime fixture still proves the current logical-operator
  subset, and a `phpc compile --emit-ir` CLI snapshot pins the native
  rejection. Truthiness over arrays/objects in generated code, side-effect
  ordering, references/copy-on-write, exact native error objects, and broader
  native lowering remain explicit gaps.
- Added a native comparison boundary. LLVM IR emission now rejects `==`, `!=`,
  `===`, `!==`, `<`, `<=`, `>`, and `>=` before lowering operands, with a
  specific codegen diagnostic until generated code has PHP comparison
  coercions and non-scalar comparison diagnostics; the C assembly fallback
  carries the same boundary for consistency. A runtime fixture still proves the
  current scalar comparison subset, and a `phpc compile --emit-ir` CLI
  snapshot pins the native rejection. PHP comparison coercions for all value
  types, arrays/objects, `NAN`/`INF`, references/copy-on-write, exact native
  error objects, and broader native lowering remain explicit gaps.
- Added a native string arithmetic boundary. LLVM IR emission now rejects
  string operands for `+`, `-`, `*`, `/`, and `%` with a specific codegen
  diagnostic until generated code has numeric-string coercion and non-numeric
  string diagnostics; the C assembly fallback carries the same boundary for
  consistency. A runtime fixture still proves the current `phpc run`
  numeric-string arithmetic subset, and a `phpc compile --emit-ir` CLI
  snapshot pins the native rejection. PHP warning/recovery behavior,
  references/copy-on-write, exact native error objects, and broader numeric
  lowering remain explicit gaps.
- Added a native dynamic division boundary for `/`: LLVM IR emission and the C
  assembly fallback now reject runtime-computed divisors until generated code
  has an explicit zero-check path. A runtime fixture still exercises the same
  program through `phpc run`, and a `phpc compile --emit-ir` CLI snapshot pins
  the codegen diagnostic. PHP-shaped native `DivisionByZeroError` objects,
  warning/recovery behavior, string numeric coercions, references/copy-on-write,
  and broader numeric lowering remain explicit gaps.
- Added a native division safety boundary for `/`: LLVM IR emission and the C
  assembly fallback now reject statically known zero divisors before emitting
  native division, with focused unit coverage and a `phpc compile --emit-ir`
  CLI snapshot. Dynamic zero checks, PHP-shaped native `DivisionByZeroError`
  objects, warning/recovery behavior, string numeric coercions,
  references/copy-on-write, and broader numeric lowering remain explicit gaps.
- Implemented a narrow native modulo lowering slice. LLVM IR now emits `srem`
  and the C assembly fallback emits C `%` for integer `%` expressions when the
  divisor is a nonzero integer known at compile time, with focused unit
  coverage, a `phpc compile --emit-ir` CLI snapshot, fixture coverage, system
  PHP comparison, and assembly smoke coverage. Native modulo still rejects
  non-integer operands, dynamic divisors, modulo by zero, interpreter-only PHP
  coercions, exact native error objects, references/copy-on-write, and broader
  native lowering.
- Implemented modulo compound assignment `%=` for the existing direct
  static-variable, direct array-offset, and direct public object-property
  compound-assignment target subset. Statement, expression, and C-style `for`
  header contexts reuse the current read-modify-write path and modulo runtime
  helper, with fixture/CLI coverage, system PHP comparison, stable
  modulo-by-zero diagnostics, and explicit native-codegen rejection.
  Append-offset/nested targets, arrays/objects as modulo values, exact native
  warning/error objects, references/copy-on-write, and native lowering remain
  explicit gaps.
- Implemented modulo `%` over the current integer-coercion subset, with
  multiplicative parser precedence, integer remainder results, null/bool/int/
  float/well-formed numeric-string coercions, fixture/CLI coverage, system PHP
  comparison for the supported fixture, stable modulo-by-zero diagnostics, and
  explicit native-codegen rejection. Non-numeric strings, arrays/objects,
  float-to-int precision warnings, exact native error objects,
  references/copy-on-write, and native lowering remain explicit gaps.
- Implemented bitwise and shift compound assignment operators `&=`, `|=`,
  `^=`, `<<=`, and `>>=` for the existing direct static-variable, direct
  array-offset, and direct public object-property compound-assignment target
  subset. Statement, expression, and C-style `for` header contexts reuse the
  current read-modify-write path and bitwise/shift runtime helpers, with
  fixture/CLI coverage, system PHP comparison, stable diagnostics, and
  explicit native-codegen rejection. Append-offset/nested targets,
  arrays/objects as bitwise values, exact native warning/error objects,
  references/copy-on-write, and native lowering remain explicit gaps.
- Implemented shift operators `<<` and `>>` over the current scalar-to-int
  coercion subset, with PHP-compatible precedence between additive expressions
  and concatenation, integer result behavior, large-count handling, fixture/CLI
  coverage, system PHP comparison, stable negative-count diagnostics, and
  explicit native-codegen rejection. String operands that are not numeric,
  arrays/objects, exact native `ArithmeticError`/`TypeError` objects,
  warning/deprecation recovery for float-to-int precision loss, bitwise/shift
  compound assignment, references/copy-on-write, and native lowering remain
  explicit gaps.
- Implemented unary bitwise not `~` over the current integer/string slice,
  with integer result behavior, UTF-8-preserving string byte behavior,
  precedence/assignment-expression coverage, fixture/CLI coverage, system PHP
  comparison for the supported fixture, stable diagnostics for non-UTF-8
  string results and unsupported non-int/non-string operands, and explicit
  native-codegen rejection. Arbitrary binary string output, exact native
  `TypeError` objects, references/copy-on-write, shifts, bitwise compound
  assignment, and native lowering remain explicit gaps.
- Added Milestone 152 executable coverage for ternary expressions mixed with
  null-coalescing expressions and assignment-expression branches. The covered
  slice pins `??` precedence in ternary conditions and branches, lazy selected
  branch/fallback behavior, short-ternary condition-value behavior after
  coalescing, and branch-local direct assignment/compound assignment/`??=`
  result semantics with fixture/CLI coverage and system PHP comparison while
  keeping unparenthesized nested ternaries, throw expressions inside arms,
  references/copy-on-write, exact native error objects, and native lowering as
  explicit gaps.
- Implemented short ternary expressions `$value ?: $fallback` over the current
  expression/value subset, including condition-value reuse, lazy fallback
  evaluation, scalar truthiness coverage, value-context fixture/CLI coverage,
  system PHP comparison, and explicit native-codegen rejection while
  unparenthesized nested ternaries, throw expressions inside arms,
  references/copy-on-write, exact native error objects, and native lowering
  remain explicit gaps.
- Implemented full ternary conditional expressions
  `$condition ? $if_true : $if_false` over the current expression/value subset,
  including truthiness-based condition selection, lazy branch evaluation,
  parenthesized nested ternaries, value-context fixture/CLI coverage, system
  PHP comparison, and explicit native-codegen rejection while short ternary,
  unparenthesized nested ternaries, throw expressions inside arms,
  references/copy-on-write, exact native error objects, and native lowering
  remain explicit gaps.
- Added Milestone 149 executable coverage for assignment-expression values in
  non-echo expression contexts: function-call arguments, array literal
  keys/values, `if`/`while`/`for` conditions, and builtin arguments. The
  covered slice proves direct assignment, compound assignment, and null
  coalescing assignment values through fixture/CLI coverage with system PHP
  comparison while keeping nested lvalues, append-offset chained assignment,
  append-offset `??=`, references/copy-on-write, exact native error objects,
  and native lowering as explicit gaps.
- Implemented chained assignment mixes where the right-hand value is a direct
  compound assignment or direct null coalescing assignment expression, such as
  `$left = ($right += expr)` and `$left = ($right ??= expr)`. The supported
  slice covers current direct static variables, direct array offsets, and
  direct public object properties, preserves lazy `??=` RHS evaluation, has
  fixture/CLI coverage with system PHP comparison, and keeps native emission
  on the existing explicit assignment-expression rejection path. Append-offset
  chains, nested/complex lvalues, references/copy-on-write, exact native error
  objects, and native lowering remain explicit gaps.
- Implemented chained `=` assignment expressions over the current
  direct-variable, direct array-offset, and direct public object-property
  assignment-expression subset, including right-to-left result semantics,
  fixture/CLI coverage, system PHP comparison, stable unsupported append-chain
  snapshots, documentation, and native-codegen rejection while append-offset
  chained assignment, nested/complex lvalues, references/copy-on-write, exact
  native error objects, and native lowering remain explicit gaps.
- Implemented expression-position null coalescing assignment
  `($name ??= expr)`, `($array[$key] ??= expr)`, and
  `($object->property ??= expr)` over the current direct-variable,
  direct-array-offset, and direct public-property value model, including lazy
  right-hand evaluation, assigned/existing value results, fixture/CLI
  coverage, system PHP comparison, documentation, and native-codegen rejection
  while append-offset `??=`, nested lvalues, dynamic property names,
  non-public visibility context, magic methods, references/copy-on-write,
  exact native error objects, and native lowering remain explicit gaps.
- Implemented expression-position direct append-offset assignment
  `($array[] = expr)` over the current ordered array value model, including
  appended-value expression results, undefined/null target materialization,
  non-array-target diagnostics, fixture/CLI coverage, system PHP comparison,
  documentation, and native-codegen rejection while nested append/offset
  targets, object properties, references/copy-on-write, exact native error
  objects, broader lvalue ordering, and native lowering remain explicit gaps.
- Implemented expression-position direct public object-property assignment
  `($object->property = expr)` over existing declared public property slots,
  including assigned-value expression results, RHS-before-target-error
  behavior, non-object-target diagnostics, fixture/CLI coverage, system PHP
  comparison, documentation, and native-codegen rejection while dynamic
  property names, missing-property materialization, non-public visibility
  context, nested properties/offsets, references/copy-on-write, exact native
  error objects, and native lowering remain explicit gaps.
- Implemented expression-position direct array-offset assignment
  `($array[$key] = expr)` over the current ordered array value model,
  including assignment result values, key-before-RHS evaluation order,
  undefined/null target materialization, non-array target diagnostics,
  fixture/CLI coverage, system PHP comparison, documentation, and
  native-codegen rejection while append offsets, nested offsets,
  references/copy-on-write, exact native error objects, and native lowering
  remain explicit gaps. Object-property assignment expressions were
  implemented in a later slice.
- Implemented direct array-offset pre/post increment and decrement
  `++$array[$key]`, `$array[$key]++`, `--$array[$key]`, and
  `$array[$key]--` over existing integer/string keyed entries whose current
  values are integers or floats, including statement, expression, and C-style
  `for` header forms, pre/post expression result values, missing-key,
  non-array-target, and unsupported-string diagnostics, fixture/CLI coverage,
  system PHP comparison, documentation, and native-codegen rejection while
  append offsets, nested offsets, PHP string increment semantics,
  references/copy-on-write, exact native warning/error behavior, broader PHP
  coercion recovery, and native lowering remain explicit gaps.
- Implemented direct public object-property pre/post increment and decrement
  `++$object->property`, `$object->property++`, `--$object->property`, and
  `$object->property--` over existing declared public integer/float property
  slots, including statement, expression, and C-style `for` header forms,
  pre/post expression result values, missing-property, non-public-property,
  non-object-target, and unsupported-string diagnostics, fixture/CLI coverage,
  documentation, and native-codegen rejection while string increment
  semantics, dynamic property names, missing-property materialization,
  non-public visibility context, nested properties/offsets,
  references/copy-on-write, exact native warning/error behavior, broader PHP
  coercion recovery, and native lowering remain explicit gaps.
- Implemented direct public object-property compound assignment
  `$object->property += expr`, `-=`, `*=`, `/=`, and `.=` over existing
  declared public property slots, including statement, expression, and
  C-style `for` header forms, updated-value expression results, RHS ordering,
  missing-property, non-public-property, and non-object-target diagnostics,
  fixture/CLI coverage, system PHP comparison, documentation, and
  native-codegen rejection while dynamic property names, missing-property
  materialization, non-public visibility context, nested properties/offsets,
  references/copy-on-write, exact native error objects, broader PHP warning
  recovery, and native lowering remain explicit gaps.
- Implemented direct array-offset compound assignment
  `$array[$key] += expr`, `-=`, `*=`, `/=`, and `.=` over existing
  integer/string keyed array entries, including statement, expression, and
  C-style `for` header forms, updated-value expression results, single key
  evaluation before RHS evaluation, missing-key and non-array diagnostics,
  fixture/CLI coverage, system PHP comparison, documentation, and
  native-codegen rejection while append offsets, nested offsets, object
  properties, references/copy-on-write, exact native error objects, broader PHP
  warning recovery, and native lowering remain explicit gaps.
- Implemented expression-position direct static-variable compound assignment
  `($name += expr)`, `-=`, `*=`, `/=`, and `.=` over the current scalar value
  model, including updated-value expression results, read/write ordering, RHS
  call ordering, fixture/CLI coverage, system PHP comparison, documentation,
  and native-codegen rejection while array/object targets,
  references/copy-on-write, exact native error objects, broader PHP coercion
  recovery, and native lowering remain explicit gaps.
- Implemented expression-position direct static-variable assignment
  `$name = expr` over the current value model, including assignment result
  values, read/write ordering, RHS call ordering, fixture/CLI coverage, system
  PHP comparison, documentation, and native-codegen rejection while chained
  assignments, nested assignment-expression targets, references,
  copy-on-write, exact native error objects, and native lowering remain
  explicit gaps. Direct array-offset, object-property, append-offset, and
  null coalescing assignment expressions were implemented in later slices.
- Implemented expression-position direct static-variable pre/post increment
  and decrement for existing integer and float variables, including
  pre-vs-post result values, read-modify-write behavior, undefined-variable
  and unsupported-string diagnostics, fixture/CLI coverage, system PHP
  comparison, documentation, and native-codegen rejection while string
  increment semantics, array/object targets, chained increment/decrement
  expressions, references, copy-on-write, and broader PHP warning recovery
  remain explicit gaps.
- Implemented direct static-variable pre/post increment and decrement in
  C-style `for` initializer and increment slots for existing integer and float
  variables, including loop execution behavior, undefined-variable and
  unsupported-string diagnostics, fixture/CLI coverage, system PHP comparison,
  documentation, and native-codegen rejection through the existing `for`
  lowering boundary.
- Implemented statement-level direct static-variable pre/post increment and
  decrement for existing integer and float variables, including
  read-modify-write behavior, undefined-variable and unsupported-string
  diagnostics, fixture/CLI coverage, system PHP comparison, documentation, and
  native-codegen rejection while expression-position forms and array/object
  targets remain explicit parse boundaries.
- Added explicit unsupported pre/post increment and decrement diagnostics
  before executable statement-level semantics existed. After the executable
  direct-variable int/float slice, the retained diagnostics cover
  unsupported array/object targets.
- Implemented direct static-variable compound assignment for `+=`, `-=`, `*=`,
  `/=`, and `.=` over the current scalar value model, including
  read-modify-write behavior in statements and `for` headers, undefined
  left-hand diagnostics, fixture/CLI coverage, system PHP comparison, docs,
  and native-codegen rejection.
- Added explicit unsupported compound assignment diagnostics for `+=`, `-=`,
  `*=`, `/=`, and `.=` forms before read-modify-write semantics exist, with
  parser regression coverage, fixture/CLI snapshots, documentation, and
  native emission rejection at the parse boundary.
- Added explicit unsupported expression-position assignment diagnostics before
  direct static-variable assignment expressions existed. After the executable
  direct-variable, direct array-offset, direct object-property, append-offset,
  and null coalescing assignment slices, the retained diagnostics cover
  chained assignments and nested assignment-expression targets.
- Implemented direct public object-property `??=` for
  `$object->property ??= expr`, including lazy initialization of existing
  declared public null slots, preservation of falsey non-null values, stable
  diagnostics for missing properties, undefined targets, and non-object
  targets, fixture/CLI coverage, documentation, and native-codegen rejection.
- Implemented direct array-offset `??=` for `$array[$key] ??= expr`, including
  lazy fallback assignment for missing/null slots, materialization of
  undefined/null target arrays, non-array target diagnostics, fixture/CLI
  coverage, documentation, and native-codegen rejection.
- Implemented the first executable `??=` slice for direct static variables,
  with lazy right-hand evaluation, preservation of non-null falsey values,
  fixture/CLI coverage, documentation, native-codegen rejection, and explicit
  unsupported diagnostics for array-offset and object-property assignment
  targets.
- Extended `??` to direct public object-property operands, with fallback
  behavior for missing properties, undefined/non-object targets, and null
  property slots, preservation of falsey non-null property values, fixture/CLI
  coverage, documentation, and native-codegen rejection.
- Implemented the first executable `??` slice for direct static variables and
  direct array offsets, with isset-like fallback behavior for undefined,
  missing, and null values, lazy fallback evaluation for present non-null
  values, fixture/CLI coverage, documentation, and native-codegen rejection.
- Kept explicit unsupported diagnostics for unparenthesized chained null
  coalescing while broader null-aware expression behavior remains
  unimplemented.
- Added explicit unsupported ternary conditional expression diagnostics for
  full and short ternary forms before expression-form branching exists, with
  parser regression coverage, fixture/CLI coverage, documentation, and native
  emission rejection at the parse boundary.
- Added explicit unsupported PHP 8 `match` expression diagnostics before
  expression-form branching exists, with parser regression coverage,
  fixture/CLI coverage, documentation, and native emission rejection at the
  parse boundary.
- Added explicit unsupported exception syntax diagnostics for `throw`
  statements/expressions and `try`/`catch`/`finally` blocks before exception
  objects or stack unwinding exist, with parser regression coverage,
  fixture/CLI coverage, documentation, and native emission rejection at the
  parse boundary.
- Added an explicit unsupported `unset($object->publicProperty)` boundary
  before object property uninitialization semantics exist, with parser
  regression coverage, fixture/CLI coverage, documentation, and native
  emission rejection at the parse boundary.
- Added `empty($object->publicProperty)` for direct object-variable operands
  over the current public instance property model, with truthiness behavior for
  public slots, empty results for missing properties, undefined target
  variables, and non-object targets, plus fixture/CLI coverage, documentation,
  and native-codegen rejection coverage.
- Added `get_mangled_object_vars($object)` for the current minimal object
  value model as a public-property slice, with direct-call and dynamic-call
  runtime coverage, fixture coverage, non-object diagnostics, documentation,
  and native-codegen rejection coverage.
- Added `spl_object_hash($object)` as an explicit unsupported object-handle
  hash boundary before PHP object handle hash behavior exists, with direct-call
  and dynamic-call runtime coverage, non-object diagnostics, fixture and CLI
  snapshots, documentation, and native-codegen rejection coverage.
- Added `spl_object_id($object)` as an explicit unsupported object-handle
  identity boundary before PHP object handles exist, with direct-call and
  dynamic-call runtime coverage, non-object diagnostics, fixture and CLI
  snapshots, documentation, and native-codegen rejection coverage.
- Added `get_called_class()` as an explicit unsupported boundary before
  method/static class context exists, with runtime coverage, dynamic-call
  coverage, fixture coverage, a `phpc run` CLI snapshot, and native-codegen
  rejection coverage.
- Added `enum_exists($name[, $autoload])` as an always-false boundary for the
  current no-enum metadata model, with runtime coverage, fixture coverage,
  dynamic-call coverage, invalid-argument diagnostics, and a `phpc run` CLI
  snapshot.
- Added `trait_exists($name[, $autoload])` as an always-false boundary for
  the current no-trait metadata model, with runtime coverage, fixture
  coverage, dynamic-call coverage, invalid-argument diagnostics, and a
  `phpc run` CLI snapshot.
- Added `interface_exists($name[, $autoload])` as an always-false boundary for
  the current no-interface metadata model, with runtime coverage, fixture
  coverage, dynamic-call coverage, invalid-argument diagnostics, and a
  `phpc run` CLI snapshot.
- Added `get_declared_traits()` as an empty trait-list boundary for the current
  no-trait metadata model, with runtime coverage, fixture coverage,
  dynamic-call coverage, and a `phpc run` CLI snapshot.
- Added `get_declared_interfaces()` as an empty interface-list boundary for
  the current no-interface metadata model, with runtime coverage, fixture
  coverage, dynamic-call coverage, and a `phpc run` CLI snapshot.
- Added `get_parent_class($object_or_class)` as a no-inheritance metadata
  boundary that accepts current object/declared-string inputs, returns false
  until parent metadata exists, and has runtime, fixture, and CLI snapshot
  coverage.
- Added `is_subclass_of($object_or_class, $class_name[, $allow_string])` as a
  no-inheritance metadata boundary that validates supported arguments, returns
  false for current exact-class/no-parent cases, and has runtime, fixture, and
  CLI snapshot coverage.
- Added `property_exists($object_or_class, $property)` for current object
  values and declared class metadata, with runtime coverage, fixture coverage,
  and `phpc run` CLI snapshots.
- Added `class_exists($name[, $autoload])` for the current declared-class
  metadata table, with runtime coverage, fixture coverage, and `phpc run` CLI
  snapshots.
- Added `get_debug_type($value)` for the current scalar/array/minimal object
  value model, with runtime coverage, fixture coverage, and a `phpc run` CLI
  snapshot.
- Added `is_object($value)` for the current minimal object value model, with
  runtime coverage, fixture coverage, and a `phpc run` CLI snapshot.
- Added an explicit unsupported trait-use-in-class boundary with parser
  regression coverage, fixture coverage, and a `phpc run` CLI snapshot.
- Added an explicit unsupported `implements` clause boundary with parser
  regression coverage, fixture coverage, and a `phpc run` CLI snapshot.
- Added an explicit unsupported magic static receiver boundary for `self::`,
  `parent::`, and `static::`, with parser regression coverage, fixture
  coverage, and a `phpc run` CLI snapshot.
- Added an explicit unsupported `ClassName::class` boundary with parser
  regression coverage, fixture coverage, and a `phpc run` CLI snapshot.
- Added an explicit unsupported `instanceof` expression boundary with parser
  regression coverage, fixture coverage, and a `phpc run` CLI snapshot.
- Added an explicit unsupported `clone` expression boundary with parser
  regression coverage, fixture coverage, and a `phpc run` CLI snapshot.
- Added explicit unsupported constructor execution and constructor argument
  CLI snapshots for the current minimal object instantiation boundary.
- Added an explicit unsupported `$this` object-context boundary with parser
  regression coverage, fixture coverage, and a `phpc run` CLI snapshot.
- Added an explicit unsupported class constant declaration boundary with
  parser regression coverage, fixture coverage, and a `phpc run` CLI snapshot.
- Added an explicit unsupported multiple property declaration boundary with
  parser regression coverage, fixture coverage, and a `phpc run` CLI snapshot.
- Added an explicit unsupported property default boundary with parser
  regression coverage, fixture coverage, and a `phpc run` CLI snapshot.
- Added explicit unsupported `abstract`, `final`, and `readonly` class member
  modifier boundaries with parser regression coverage, fixture coverage, and
  `phpc run` CLI snapshots.
- Added explicit unsupported `abstract`, `final`, and `readonly` class modifier
  boundaries with parser regression coverage, fixture coverage, and `phpc run`
  CLI snapshots.
- Added an explicit unsupported `enum` declaration boundary with parser
  regression coverage, fixture coverage, and a `phpc run` CLI snapshot.
- Added an explicit unsupported `interface` declaration boundary with parser
  regression coverage, fixture coverage, and a `phpc run` CLI snapshot.
- Added an explicit unsupported `trait` declaration boundary with parser
  regression coverage, fixture coverage, and a `phpc run` CLI snapshot.
- Tightened the unsupported `__NAMESPACE__` magic-constant boundary with a
  namespace-resolution-specific parse diagnostic, parser regression coverage,
  fixture coverage, and a `phpc run` CLI snapshot.
- Implemented logical `&&`, `||`, `and`, and `or` over current PHP-shaped
  truthiness, with short-circuit evaluation, boolean results, PHP-style
  symbolic precedence, lower-than-assignment word-operator precedence,
  fixture/CLI coverage, system PHP comparison, and explicit native-codegen
  rejection. Logical `xor`, references/copy-on-write side effects, exact
  native error objects, and native lowering remain unsupported.
- Implemented bitwise `&`, `|`, and `^` over the current integer/string
  subset, with PHP-style precedence, integer results for current scalar-to-int
  operands, bytewise string-string results when the runtime string remains
  valid UTF-8, fixture/CLI coverage, system PHP comparison, stable runtime
  diagnostics for non-numeric mixed strings, and explicit native-codegen
  rejection. Arbitrary binary strings outside UTF-8, arrays/objects, bitwise
  compound assignment, unary `~`, shifts, exact PHP warning/error objects,
  references/copy-on-write, and native lowering remain explicit gaps.
- Implemented logical `xor` over current PHP-shaped truthiness, with boolean
  results, both-operand evaluation, PHP-style word precedence between `and`
  and `or`, lower-than-assignment behavior, assignment-expression operand
  coverage, fixture/CLI coverage, system PHP comparison, and explicit
  native-codegen rejection. Operator-overloaded extension values, exact native
  error objects, references/copy-on-write, and native lowering remain explicit
  gaps.
- Added a native unary boundary. LLVM IR emission now rejects unary minus and
  logical not before lowering operands with a specific diagnostic, the C
  assembly fallback has the same boundary, and a Milestone 177 fixture plus
  `phpc compile --emit-ir` CLI snapshot pin both the current interpreter unary
  behavior and the native rejection. Native PHP numeric coercion, truthiness
  conversion, references/copy-on-write side effects, exact native error
  objects, and broader native unary lowering remain explicit gaps.
