# Progress Log

## 2026-05-12

Implemented:

- Initialized the repository and Rust workspace.
- Added project rules, README, architecture, support, roadmap, extension status,
  and progress documentation.
- Implemented a stable Rust `php_runtime` crate with scalar boxed values:
  `Null`, `Bool`, `Int`, `Float`, and `String`.
- Implemented scalar echo conversion, truthiness, arithmetic, concatenation, and
  comparison helpers.
- Implemented the `phpc` compiler crate with lexer, parser, AST, interpreter,
  fixture test runner, and CLI.
- Implemented `phpc run <input.php>` for the Milestone 1 subset.
- Implemented `phpc compile <input.php> --emit-ir` for a narrow straight-line
  scalar subset using LLVM IR text.
- Implemented `phpc compile <input.php> --emit-asm`; it prefers `clang`/`llc`
  and currently falls back to `cc -S` through generated C when LLVM tools are not
  installed.
- Added Milestone 1 fixtures for echo, literals, variables, assignment,
  arithmetic, concatenation, `if`/`else`, `while`, function declaration, function
  call, and `return`.
- Added a small Milestone 2 scalar slice: `print` statements, unary minus,
  logical not, and fixtures for `null`/bool/string truthiness.
- Added optional `phpc test --compare-php [fixture-dir]` support. When system
  `php` is installed it compares fixture stdout, stderr, and exit code against
  `phpc run`; when `php` is absent it skips comparison and still runs committed
  expected-output fixtures.
- Added two narrow Milestone 2 scalar comparison fixtures for echo conversion,
  truthiness, and numeric-string arithmetic. This does not mark broader
  Milestone 2 support complete.
- Added builtin support for `strlen`, `isset`, `count`, `var_dump`, and
  `print_r` across the documented scalar/array subset with fixture coverage.
  Object handling was added later as a separate narrow slice.
- Added operational automation: `tools/checkpoint.sh`, `tools/codex-loop.sh`,
  `docs/OPERATIONS.md`, `docs/NEXT_TASKS.md`, and
  `docs/CODEX_LOOP_PROMPT.md`.
- Added structured runtime error categories and stable messages for undefined
  variables, arity mismatches, unsupported calls, and division by zero.
- Changed plain undefined variable reads to fail with a runtime error; direct
  `isset($name)` checks remain supported and return false for missing/null
  variables.
- Added `.phpc-only` fixture markers so project-specific runtime diagnostics can
  be exercised by the fixture runner without being compared to system PHP.
- Added explicit `phpc run` CLI snapshots for representative runtime errors,
  recording process exit code, stdout, and stderr for undefined variables,
  user-function arity mismatches, unsupported scalar `count()` calls, division
  by zero, non-numeric string arithmetic, and unsupported array keys.
- Completed a scalar arithmetic coercion slice for `null`, booleans, integers,
  floats, and well-formed numeric strings, including signed, decimal, exponent,
  and surrounding-whitespace numeric strings.
- Changed non-numeric string arithmetic to fail with a structured invalid
  arithmetic runtime error instead of silently coercing to zero.
- Added a PHP 8-style scalar comparison matrix for `==`, `!=`, `<`, `<=`, `>`,
  and `>=` across the implemented scalar value types, including `null`,
  booleans, integers, floats, empty strings, numeric strings, and non-numeric
  strings.
- Implemented an ordered `php_runtime` array value with integer/string keys,
  PHP-style decimal string key normalization, insertion-order preservation, and
  keyless append allocation for array literals.
- Added parser and interpreter support for short array literals `[]`, `[value]`,
  and `[key => value]` over the supported expression subset.
- Added parser and interpreter support for long `array(...)` literals as an
  alias for the current short-array literal subset, including keyless entries,
  keyed entries, trailing commas, and case-insensitive `ARRAY(...)` syntax.
- Added explicit stable parse diagnostics, fixture coverage, and `phpc run` CLI
  snapshots for unsupported array spread elements and array reference elements.
- Added parser and interpreter support for array indexed reads, direct variable
  indexed writes, and direct variable append writes over the current ordered
  array value model.
- Added write materialization for undefined and `null` array variables in the
  supported direct-variable array assignment subset.
- Added array-aware `count`, `print_r`, and `var_dump` behavior for the current
  ordered array value model. `strlen` remains scalar-only and rejects arrays.
- Added stable invalid-array-key diagnostics for array literal keys that do not
  evaluate to integers or strings.
- Added stable undefined-array-key and invalid-array-access diagnostics for the
  current array indexing and assignment subset.
- Added explicit LLVM IR rejection paths and tests for arrays, array indexing,
  and array assignment until native lowering exists.
- Added explicit local/global scope coverage for user functions: each
  user-function call gets a fresh local scope, parameters and local assignments
  shadow globals without mutating them, and plain reads of globals from inside
  functions remain undefined-variable runtime errors unless passed as arguments.
- Added parser support for `global` declarations as an explicit unsupported
  statement and a stable runtime diagnostic for attempts to import globals into
  function scope.
- Added explicit LLVM IR rejection coverage for `global` declarations until
  scope-import lowering exists.
- Added recursive user-function execution coverage and a fixed 128-frame
  user-function call-depth guard for runaway recursion, with a stable runtime
  diagnostic and committed CLI snapshot.
- Added parser and interpreter support for trailing default parameter values in
  user functions over the documented constant-expression subset, including
  required-to-total arity diagnostics and scalar/array default fixture coverage.
- Added explicit parser diagnostics, unit tests, fixture coverage, and
  `phpc run` CLI snapshots for unsupported function features: variadic
  parameters, variadic argument unpacking, references, anonymous functions,
  arrow functions, named arguments, and `declare(strict_types=1)`.
- Added a materialized interpreter symbol table for top-level and function-local
  scopes. Current static variable reads, writes, `unset($name)`,
  `isset($name)`, parameter binding, default-parameter evaluation, and direct
  array write materialization now route through named symbol-table APIs without
  changing static variable behavior.
- Added an explicit stable lex diagnostic, fixture coverage, and `phpc run` CLI
  snapshot for unsupported variable-variable syntax such as `$$name`.
- Designed the first include/require resolution boundary and added explicit
  stable parse diagnostics, fixture coverage, and `phpc run` CLI snapshots for
  unsupported `include`, `include_once`, `require`, and `require_once`
  constructs.
- Added runtime lookup infrastructure for dynamic function calls through
  string-valued expressions. The current slice resolves user-defined functions
  and the documented callable builtin subset, keeps unresolved names as stable
  undefined-function runtime errors, rejects non-string callees with a stable
  unsupported-call runtime error, and still rejects native lowering explicitly.
- Defined the first `eval` fallback boundary and added explicit stable parse
  diagnostics, fixture coverage, and `phpc run` CLI snapshots for unsupported
  direct `eval(...)` statement and expression forms. Eval execution remains
  unsupported.
- Added explicit stable parse diagnostics, fixture coverage, and `phpc run`
  CLI snapshots for unsupported namespace declarations and top-level `use`
  import declarations before namespace-aware name resolution or imports exist.
- Added explicit stable parse diagnostics, fixture coverage, and `phpc run`
  CLI snapshots for unsupported namespace-qualified function and class names
  such as `App\fn()` and `new App\Box()` before namespace-aware name resolution
  exists.
- Added the first internal object/class metadata sketch in `php_runtime`:
  ordered class tables with stable `ClassId` handles, class/property/method
  metadata, visibility flags, instance/static flags, duplicate
  class/member diagnostics, and derived object shapes for future instance
  property layout.
- Added explicit stable parse diagnostics, fixture coverage, and `phpc run` CLI
  snapshots for unsupported object/class syntax; after class declarations and
  minimal object instantiation became supported, the unsupported coverage
  remains focused on method calls, dynamic property names, anonymous classes,
  and unsupported class forms such as inheritance.
- Parsed top-level class declarations into the runtime metadata registry while
  keeping object execution narrow. The accepted class-member subset records
  property names, method names, visibility, and static flags; duplicate class
  and member names route through stable runtime metadata diagnostics. Nested
  classes, inheritance, typed/default/multiple properties, member access, and
  native lowering still reject explicitly.
- Added a minimal object value/instantiation boundary for `new ClassName()`:
  declared constructor-free classes can be instantiated, class lookup is
  case-insensitive, instance properties are initialized to `null`, static
  properties are skipped, object values are truthy, direct `isset($object)`
  works, and `print_r` can render the current object shape.
- Added stable runtime diagnostics and tests for undefined classes, unsupported
  constructors/constructor arguments, object-to-string conversion, and object
  comparisons.
- Added public instance property reads and direct-variable writes for the
  current object value model. Static property names are case-sensitive, writes
  mutate the current object value stored in the variable, `print_r` renders
  updated property slots, and stable diagnostics cover undefined properties,
  non-object property targets, and non-public property access.
- Added explicit LLVM IR rejection coverage for object property reads and writes
  until native object slot lowering exists. Method dispatch, dynamic property
  names, `$this`, visibility enforcement for non-public properties,
  constructors, PHP object handle identity/aliasing, and native object lowering
  remain unsupported.
- Added direct `isset($object->publicProperty)` support for public instance
  properties in the current object value model. The supported slice checks
  direct object-variable operands, treats null slots, missing property names,
  undefined target variables, and non-object target variables as false, supports
  multiple `isset` operands, and keeps dynamic property names, non-public
  property operands, complex lvalues, and method dispatch
  unsupported.
- Added direct `isset($array[$key])` support for array offsets on direct array
  variables. The supported slice checks integer/string keyed offsets, treats
  existing non-null slots as true, treats null slots, missing keys, undefined
  target variables, and non-array target variables as false, supports multiple
  `isset` operands, and keeps nested/complex offset operands explicitly
  unsupported.
- Added `array_key_exists($key, $array)` support for the current ordered array
  value model. The supported slice accepts integer/string keys, reports true
  for existing keys even when the stored value is `null`, reports false for
  missing keys, is available through string-valued dynamic function calls, and
  has stable diagnostics for unsupported key values and non-array second
  arguments.
- Added `array_values($array)` support for the current ordered array value
  model. The supported slice preserves value insertion order, returns a new
  array reindexed with integer keys starting at zero, is available through
  string-valued dynamic function calls, and has a stable diagnostic for
  non-array arguments.
- Added `array_keys($array)` support for the current ordered array value model.
  The supported slice preserves key insertion order, emits integer/string keys
  as values in a new array reindexed from zero, is available through
  string-valued dynamic function calls, and has a stable diagnostic for
  non-array arguments.
- Added `array_reverse($array)` and `array_reverse($array, false)` support for
  the current ordered array value model. The supported default slice returns a
  new array in reverse insertion order, reindexes integer-keyed entries from
  zero while preserving string keys, and is available through string-valued
  dynamic function calls.
- Added `array_reverse($array, true)` preserve-key support for the current
  ordered integer/string key model. The supported slice reverses insertion
  order while preserving integer and string keys, is available through
  string-valued dynamic function calls, and has stable diagnostics for
  non-array arguments and non-bool `preserve_keys` flag values.
- Added `in_array($needle, $array)` support for the current ordered array value
  model. The supported slice scans values in insertion order, uses the current
  loose scalar comparison rules by default, also supports the boolean strict
  flag for current scalar needles/values, is available through string-valued
  dynamic function calls, and has stable diagnostics for non-array haystacks,
  non-bool strict flags, and unsupported array/object needles or values.
- Added `array_search($needle, $array)` support for the current ordered array
  value model. The supported slice scans values in insertion order with the
  current loose scalar comparison rules by default, also supports the boolean
  strict flag for current scalar needles/values, returns the first matching
  integer or string key, returns `false` for misses, is available through
  string-valued dynamic function calls, and has stable diagnostics for
  non-array haystacks, non-bool strict flags, and unsupported array/object
  needles or values.
- Added `empty(...)` support for direct variables and direct array offsets over
  the current scalar/array value model. The supported slice treats undefined
  variables, missing array keys, undefined array variables, non-array array
  targets, `null`, `false`, zero, empty strings, string `"0"`, and empty arrays
  as empty, uses current truthiness for existing values, and has a stable
  diagnostic for unsupported complex lvalues.
- Added explicit stable parse diagnostics, fixture coverage, and `phpc run` CLI
  snapshots for unsupported static property access, static method calls, and
  class constant access through `::`.
- Added explicit stable parse diagnostics, fixture coverage, and `phpc run` CLI
  snapshots for unsupported long `array(...)` literal syntax before long array
  literals are implemented.
- Added explicit stable parse diagnostics, fixture coverage, and `phpc run` CLI
  snapshots for unsupported broader `unset(...)` forms before direct variable
  removal was implemented. The unsupported forms still excluded from execution
  are property, append-offset, and nested/complex removal.
- Added explicit stable parse diagnostics, fixture coverage, and `phpc run`
  CLI snapshots for unsupported `foreach (...)` syntax before array/object
  iteration is implemented.
- Added explicit stable parse diagnostics, fixture coverage, and `phpc run`
  CLI snapshots for unsupported `for (...)` syntax before C-style loops are
  implemented.
- Added explicit stable parse diagnostics, fixture coverage, and `phpc run`
  CLI snapshots for unsupported `do ... while` syntax before post-condition
  loops are implemented.
- Added explicit stable parse diagnostics, fixture coverage, and `phpc run`
  CLI snapshots for unsupported `switch (...)` syntax before switch/case
  control flow is implemented.
- Added explicit stable parse diagnostics, fixture coverage, and `phpc run`
  CLI snapshots for unsupported `break`/`continue` syntax before loop-control
  execution is implemented.
- Implemented `break;` execution for the innermost currently executing `while`
  loop. The parser accepts statement-form `break;`, the interpreter propagates
  loop-control flow through nested statement blocks and consumes it at the
  nearest `while`, and `break;` outside an active loop fails with a stable
  invalid-loop-control runtime diagnostic.
- Implemented `continue;` execution for the innermost currently executing
  `while` loop. The parser accepts statement-form `continue;`, the interpreter
  propagates loop-control flow through nested statement blocks and consumes it
  at the nearest `while`, and `continue;` outside an active loop fails with a
  stable invalid-loop-control runtime diagnostic.
- Added explicit parse diagnostics for unsupported `break` loop-depth arguments
  such as `break 2;` and unsupported `continue` loop-depth arguments such as
  `continue 2;`.
- Added explicit LLVM IR and assembly rejection paths for `break` and
  `continue` until native loop-control lowering exists.
- Implemented value-only `foreach ($array as $value)` over the current ordered
  array value model. The parser accepts the supported statement form, the
  interpreter iterates values in insertion order over a snapshot of array
  entries, loop variables are written into the active scope, `break;` and
  `continue;` target the innermost active `foreach`, non-array iterables fail
  with a stable runtime diagnostic, by-reference forms remain explicit parse
  diagnostics, and native lowering rejects `foreach` explicitly.
- Implemented `foreach ($array as $key => $value)` over the current ordered
  array value model. Integer and string keys are emitted into the direct key
  loop variable as PHP values, values are emitted into the direct value loop
  variable, insertion order and snapshot behavior match the current value-only
  `foreach` model, and non-array iterables reuse the stable `invalid foreach`
  runtime diagnostic.
- Implemented direct `unset($array[$key])` for direct array variables over the
  current integer/string key subset. Existing keys are removed while preserving
  remaining insertion order, missing keys are no-ops, undefined and `null`
  target variables are no-ops, append allocation does not reuse removed integer
  keys, existing non-array targets fail with a stable invalid-array-access
  runtime diagnostic, broader unset forms remain explicit parse diagnostics,
  and native lowering rejects array-offset unset explicitly.
- Implemented direct `unset($name)` for static variables backed by the active
  materialized symbol table. Existing symbols are removed from the current
  top-level or function-local scope, undefined names are no-ops, later plain
  reads use the existing undefined-variable diagnostic, fixture CLI coverage
  records current-scope and local-scope behavior, property/append-offset/nested
  unset forms remain explicit parse diagnostics, and
  native lowering rejects variable unset explicitly.
- Implemented multiple-operand `unset(...)` over the currently supported direct
  variable and direct array-offset operands. Operands execute left to right,
  array-offset key expressions are evaluated in operand order, missing
  variables and missing array keys remain no-ops, unsupported property,
  append-offset, and nested unset forms remain explicit parse diagnostics, and
  native lowering rejects multiple-operand unset explicitly.
- Implemented C-style `for (...)` loops over the current expression and
  assignment subset. The parser accepts statement-form loops with optional
  initializer, condition, and increment slots; initializer/increment slots
  support one expression or assignment; omitted conditions behave as true;
  `break;` exits the innermost loop; `continue;` runs the increment before the
  next condition check; comma-separated header expression lists remain explicit
  parse diagnostics; and native lowering rejects `for` loops explicitly.
- Implemented `do ... while` loops over the current expression and assignment
  subset. The parser accepts block and single-statement bodies, evaluates the
  condition after each body execution, guarantees at-least-once execution,
  treats `continue;` as a jump to the post-condition check, consumes `break;`
  at the innermost active loop, keeps expression-form `do ... while` rejected
  with a stable parse diagnostic, and rejects native lowering explicitly.
- Implemented statement-form brace `switch` over the current scalar
  loose-comparison subset. The parser accepts `case` and `default` labels, the
  interpreter evaluates the switch expression once, matches cases with current
  loose `==` semantics, executes default only when no case matches, preserves
  fallthrough, consumes `break;` at the switch boundary without exiting an
  outer loop, rejects `continue;` reaching a switch body with a stable runtime
  diagnostic, keeps expression-form and alternate-syntax switches as stable
  parse diagnostics, and rejects native lowering explicitly.
- Implemented `elseif` chains over the current `if` expression subset. The
  parser accepts contiguous `elseif` clauses after brace-block or
  single-statement `if`/`elseif` bodies, chains them through the existing
  nested-`if` AST shape, evaluates conditions left to right until the first
  truthy branch, preserves optional final `else` behavior, and keeps native
  conditional lowering rejected explicitly.
- Added explicit stable parse diagnostics, fixture coverage, and `phpc run`
  CLI snapshots for unsupported alternate `if`/`elseif`/`else` colon/`endif`
  conditional syntax before alternate conditional execution is implemented.
- Added an interim explicit parse-diagnostic boundary for unsupported strict
  identity operators `===` and `!==`; that boundary has since been replaced by
  executable scalar strict identity support.
- Implemented strict identity operators `===` and `!==` for the current scalar
  value subset. The parser now accepts the strict identity tokens, the
  interpreter evaluates `null`, booleans, integers, floats, and strings with
  type-and-value semantics and no coercion, array/object operands fail with
  stable unsupported-comparison runtime diagnostics, and native lowering still
  rejects strict comparisons explicitly.
- Implemented `in_array($needle, $array, true)` for the current scalar
  needle/value subset using the same scalar strict identity semantics. The
  two-argument form remains loose, `in_array($needle, $array, false)` routes to
  the loose path, string-valued dynamic calls can use the strict flag, non-bool
  strict flags fail with a stable runtime diagnostic, array/object
  needles/values remain unsupported comparison gaps, and native lowering still
  rejects the builtin call explicitly.
- Implemented `array_search($needle, $array, true)` for the current scalar
  needle/value subset using the same scalar strict identity semantics and
  existing key-return behavior. The two-argument form remains loose,
  `array_search($needle, $array, false)` routes to the loose path,
  string-valued dynamic calls can use the strict flag, non-bool strict flags
  fail with a stable runtime diagnostic, array/object needles/values remain
  unsupported comparison gaps, and native lowering still rejects the builtin
  call explicitly.

Tested:

- `cargo test` passes.
- `cargo test -p php_runtime` passes with 29 runtime unit tests.
- `cargo test -p php_runtime array_` passes with 14 focused array value tests.
- `cargo test -p php_runtime in_array` passes with 3 focused loose/strict
  array-search tests.
- `cargo test -p php_runtime array_search` passes with 3 focused loose/strict
  array-search key-return tests.
- `cargo test -p php_runtime scalar_comparison_matrix_matches_php_8_scalar_subset`
  passes.
- `cargo test -p php_runtime strict_identity` passes with 2 focused strict
  identity tests.
- `cargo test -p phpc --test runtime_errors` passes with 24 runtime error tests.
- `cargo test -p phpc --test runtime_error_cli` passes with 1 CLI snapshot test
  covering 38 representative runtime error fixtures.
- `cargo test -p phpc --test strict_identity` passes with 4 tests covering
  scalar strict identity execution, array/object strict identity diagnostics,
  and LLVM IR rejection.
- `cargo test -p phpc --test comparison_refinements_cli` passes with 1 CLI
  snapshot test covering the Milestone 12 strict identity fixture.
- `cargo test -p phpc --test functions_and_scopes` passes with 17
  user-function scope/default-parameter tests.
- `cargo test -p phpc --test unsupported_function_features_cli` passes with 1
  CLI snapshot test covering 6 representative unsupported function-feature
  fixtures.
- `cargo test -p phpc interpreter::tests::symbol_table` passes with 4 focused
  symbol-table unit tests.
- `cargo test -p phpc --test dynamic_features` passes with 9 tests covering
  static symbol-table behavior, dynamic function lookup behavior, and
  unsupported variable-variable/include/require/eval/namespace/use and
  namespace-qualified name diagnostic coverage.
- `cargo test -p phpc --test unsupported_dynamic_features_cli` passes with 1
  CLI snapshot test covering 11 unsupported variable-variable,
  include/require, eval, namespace, use, and namespace-qualified name fixtures.
- `cargo test -p phpc --test object_model` passes with 9 tests covering class
  metadata registration, minimal object instantiation, duplicate metadata
  diagnostics, undefined-class diagnostics, constructor rejection, public
  property reads/writes, public property `isset`, and stable parse diagnostics
  for unsupported object/class syntax.
- `cargo test -p phpc --test unsupported_object_features_cli` passes with 1 CLI
  snapshot test covering 7 unsupported object/class fixtures.
- `cargo test -p phpc --test syntax_boundaries` passes with long
  `array(...)` literal execution coverage and stable parse diagnostic coverage
  for unsupported array spread/reference elements, unsupported broader
  `unset(...)` forms, unsupported `foreach` by-reference and destructuring
  forms, expression-form `foreach`, unsupported `for` header expression lists,
  expression-form `for`, expression-form `do ... while`, expression-form and
  alternate-syntax `switch`, unsupported switch case separators, alternate
  `if`/`elseif`/`else` colon/`endif` syntax, and unsupported
  `break`/`continue` loop-depth arguments.
- `cargo test -p phpc --test for_loop` passes with C-style `for` loop
  coverage for initializer/condition/increment execution, optional header
  slots, uppercase `FOR`, single-statement bodies, and `break;`/`continue;`
  behavior.
- `cargo test -p phpc --test do_while` passes with post-condition loop
  coverage for at-least-once execution, uppercase `DO`/`WHILE`,
  single-statement bodies, and `break;`/`continue;` behavior.
- `cargo test -p phpc --test switch` passes with switch coverage for loose
  scalar matching, default placement, fallthrough, uppercase `SWITCH`/`CASE`,
  and `break;` behavior inside an enclosing loop.
- `cargo test -p phpc --test elseif` passes with `elseif` branch selection,
  skipped later-condition coverage, `else` fallback behavior,
  single-statement bodies, and uppercase tail keyword coverage.
- `cargo test -p phpc --test unsupported_syntax_features_cli` passes with 1 CLI
  snapshot test covering 10 unsupported syntax fixtures.
- `cargo test -p phpc --test syntax_expansion_cli` passes with 1 CLI snapshot
  test covering the Milestone 10 syntax expansion fixtures.
- `cargo test -p phpc --test conditional_refinements_cli` passes with 1 CLI
  snapshot test covering the Milestone 11 conditional-refinement fixtures.
- `cargo test -p phpc --test loop_control_cli` passes with 1 CLI snapshot test
  covering `break;` and `continue;` execution for innermost `while` loops.
- `cargo test -p phpc --test foreach` passes with value-only and key/value
  ordered array iteration, innermost `break;`/`continue;`, and non-array
  iterable diagnostic coverage.
- `cargo test -p phpc --test foreach_cli` passes with 1 CLI snapshot test
  covering the Milestone 8 `foreach` fixtures.
- `cargo test -p phpc --test array_unset` passes with direct array-offset
  unset behavior, missing-key no-op behavior, undefined/`null` target no-op
  behavior, append-index preservation, and non-array target diagnostics.
- `cargo test -p phpc --test variable_unset` passes with direct variable unset
  behavior, undefined-name no-op behavior, current-scope function-local
  behavior, reassignment after unset, and the later-read undefined-variable
  diagnostic.
- `cargo test -p phpc --test multiple_unset` passes with multiple-operand
  direct variable/array-offset unset behavior and left-to-right array key
  expression evaluation coverage.
- `cargo test -p phpc --test array_mutation_cli` passes with 1 CLI snapshot
  test covering the Milestone 9 array mutation fixtures.
- `cargo test -p phpc --test array_isset` passes with direct array-offset
  `isset` behavior and unsupported complex-lvalue coverage.
- `cargo test -p phpc --test array_key_exists` passes with direct
  `array_key_exists` behavior, null-value contrast against `isset`, dynamic
  string-call coverage, and stable diagnostics for unsupported key values and
  non-array second arguments.
- `cargo test -p phpc --test array_values` passes with `array_values`
  reindexing behavior, dynamic string-call coverage, original-array
  preservation, and stable diagnostics for non-array arguments.
- `cargo test -p phpc --test array_keys` passes with `array_keys`
  integer/string key emission, dynamic string-call coverage, original-array
  preservation, and stable diagnostics for non-array arguments.
- `cargo test -p phpc --test array_reverse` passes with `array_reverse`
  reverse-order behavior, numeric-key reindexing, string-key preservation,
  preserve-key behavior for integer and string keys, dynamic string-call
  coverage, original-array preservation, non-array diagnostics, non-bool
  `preserve_keys` diagnostics, and LLVM IR rejection coverage.
- `cargo test -p phpc --test in_array` passes with `in_array` loose scalar
  search behavior, strict scalar search behavior, dynamic string-call coverage,
  non-array haystack diagnostics, non-bool strict-flag diagnostics, explicit
  array/object comparison gap coverage, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_search` passes with `array_search` loose
  and strict scalar key-return behavior, dynamic string-call coverage,
  non-array haystack diagnostics, non-bool strict-flag diagnostics, explicit
  array/object comparison gap coverage, and LLVM IR rejection coverage.
- `cargo test -p phpc --test empty` passes with direct variable and direct
  array-offset `empty` behavior plus unsupported complex-lvalue coverage.
- `cargo test -p phpc --test array_refinements_cli` passes with 1 CLI snapshot
  test covering the Milestone 7 array refinement fixtures.
- `cargo test -p phpc --test strict_array_search_cli` passes with 1 CLI
  snapshot test covering the Milestone 13 strict `in_array` and `array_search`
  fixtures.
- `cargo test -p phpc --test array_ordering_builtins_cli` passes with 1 CLI
  snapshot test covering the Milestone 14 `array_reverse` fixture.
- `cargo test -p phpc --test php_comparison` passes.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_array` passes with
  rejection coverage for short array literals, array indexing, and array
  assignment.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_long_arrays_until_native_lowering_exists`
  passes with rejection coverage for long `array(...)` literals before native
  array lowering exists.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_global_declarations_until_scope_imports_exist`
  passes with rejection coverage for `global` declarations.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_dynamic_function_calls_until_native_lowering_exists`
  passes with rejection coverage for dynamic function calls.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_class_declarations_until_native_metadata_lowering_exists`
  passes with rejection coverage for class declarations.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_object_instantiation_until_native_lowering_exists`
  passes with rejection coverage for object instantiation.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_object_property`
  passes with rejection coverage for object property reads and writes.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_break_until_native_loop_control_lowering_exists`
  passes with rejection coverage for `break` statements before native
  loop-control lowering exists.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_continue_until_native_loop_control_lowering_exists`
  passes with rejection coverage for `continue` statements before native
  loop-control lowering exists.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_foreach_until_native_iteration_lowering_exists`
  passes with rejection coverage for `foreach` statements before native
  iteration lowering exists.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_foreach_key_value_until_native_iteration_lowering_exists`
  passes with rejection coverage for key/value `foreach` before native
  iteration lowering exists.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_for_until_native_loop_lowering_exists`
  passes with rejection coverage for `for` loops before native loop lowering
  exists.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_do_while_until_native_loop_lowering_exists`
  passes with rejection coverage for `do ... while` loops before native loop
  lowering exists.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_elseif_until_native_conditional_lowering_exists`
  passes with rejection coverage for `elseif` chains before native conditional
  lowering exists.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_variable_unset_until_native_lowering_exists`
  passes with rejection coverage for direct variable unset before native
  symbol-table mutation lowering exists.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_multiple_unset_until_native_lowering_exists`
  passes with rejection coverage for multiple-operand unset before native
  symbol-table/array-offset mutation lowering exists.
- `cargo run -p phpc -- test` passes with 123 fixture tests.
- `cargo run -p phpc -- test --compare-php` passes with system `php`
  installed, comparing 51 fixtures and skipping 72 `.phpc-only` fixtures.
- `cargo run -p phpc -- test tests/fixtures/milestone3` passes with 2 array
  fixtures.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone3` passes
  with 2 system PHP comparisons.
- `cargo run -p phpc -- test tests/fixtures/milestone4` passes with 3 function
  scope/default-parameter fixtures.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone4` passes
  with 3 system PHP comparisons.
- `cargo run -p phpc -- test tests/fixtures/milestone5` passes with 6 static
  symbol-table, dynamic-function, class-declaration, object-instantiation, and
  public object-property fixtures.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone5` passes
  with 6 system PHP comparisons.
- `cargo run -p phpc -- test tests/fixtures/milestone6` passes with 2
  loop-control fixtures.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone6` passes
  with 2 system PHP comparisons.
- `cargo run -p phpc -- test tests/fixtures/milestone7` passes with 7
  array-refinement fixtures.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone7` passes
  with 7 system PHP comparisons.
- `cargo run -p phpc -- test tests/fixtures/milestone8` passes with 2
  `foreach` fixtures.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone8` passes
  with 2 system PHP comparisons.
- `cargo run -p phpc -- test tests/fixtures/milestone9` passes with 3 array
  mutation fixtures.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone9` passes
  with 3 system PHP comparisons.
- `cargo run -p phpc -- test tests/fixtures/milestone10` passes with 4 syntax
  expansion fixtures.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone10` passes
  with 4 system PHP comparisons.
- `cargo run -p phpc -- test tests/fixtures/milestone11` passes with 1
  conditional-refinement fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone11` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- test tests/fixtures/unsupported_function_features`
  passes with 6 unsupported function-feature fixtures.
- `cargo run -p phpc -- test --compare-php
  tests/fixtures/unsupported_function_features` passes with 6 `.phpc-only`
  PHP comparisons skipped.
- `cargo run -p phpc -- test tests/fixtures/unsupported_dynamic_features`
  passes with 11 unsupported dynamic-feature fixtures.
- `cargo run -p phpc -- test --compare-php
  tests/fixtures/unsupported_dynamic_features` passes with 11 `.phpc-only` PHP
  comparisons skipped.
- `cargo run -p phpc -- test tests/fixtures/unsupported_object_features`
  passes with 7 unsupported object/class fixtures.
- `cargo run -p phpc -- test --compare-php
  tests/fixtures/unsupported_object_features` passes with 7 `.phpc-only` PHP
  comparisons skipped.
- `cargo test -p phpc --test syntax_boundaries unsupported_alternate_if_forms_are_rejected_with_stable_parse_error`
  passes with stable parse diagnostics for alternate `if`, `elseif`, and
  `else` colon syntax.
- `cargo test -p phpc --test unsupported_syntax_features_cli` passes with the
  unsupported syntax CLI snapshots, including alternate conditional syntax.
- `cargo run -p phpc -- test tests/fixtures/unsupported_syntax_features`
  passes with 10 unsupported syntax fixtures.
- `cargo run -p phpc -- test --compare-php
  tests/fixtures/unsupported_syntax_features` passes with 10 `.phpc-only` PHP
  comparisons skipped.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone2` passes
  with system `php` installed, comparing 7 Milestone 2 fixtures.
- `PATH=/nonexistent ./target/debug/phpc test --compare-php tests/fixtures/milestone2`
  passes, reporting 7 PHP comparisons skipped.
- `PATH=/nonexistent ./target/debug/phpc test --compare-php tests/fixtures/milestone3`
  passes, reporting 2 PHP comparisons skipped.
- `cargo run -p phpc -- test tests/fixtures/milestone2` passes with 7
  Milestone 2 fixtures.
- `cargo run -p phpc -- run tests/fixtures/milestone2/scalar_comparison_matrix.php`
  prints the committed 100-row scalar comparison matrix.
- `cargo run -p phpc -- run tests/fixtures/milestone3/array_literals.php`
  prints the committed array literal/count/print_r/truthiness output.
- `cargo run -p phpc -- run tests/fixtures/milestone3/array_indexing.php`
  prints the committed array append/indexed read/indexed write output.
- `cargo run -p phpc -- run tests/fixtures/milestone10/long_array_literals.php`
  prints the committed long array literal/count/print_r/indexed-read output.
- `cargo run -p phpc -- run tests/fixtures/milestone10/for_loops.php`
  prints the committed C-style `for` loop output with `continue;`, `break;`,
  omitted header slots, and increment behavior.
- `cargo run -p phpc -- run tests/fixtures/milestone10/do_while_loops.php`
  prints the committed `do ... while` output with at-least-once execution,
  post-condition `continue;`, `break;`, and uppercase single-statement syntax.
- `cargo run -p phpc -- run tests/fixtures/milestone10/switch_statements.php`
  prints the committed `switch` output with loose scalar matching,
  fallthrough, default placement, and switch-local `break;`.
- `cargo run -p phpc -- run tests/fixtures/milestone11/elseif_chains.php`
  prints the committed `elseif` chain output with first-match branch
  selection, skipped later conditions, single-statement bodies, and final
  `else` fallback.
- `cargo run -p phpc -- test tests/fixtures/runtime_errors` passes with 38
  runtime error fixtures.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/undefined_variable.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/undefined_variable.php:2:6: undefined variable '$missing'`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/non_numeric_string_arithmetic.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/non_numeric_string_arithmetic.php:2:6: invalid arithmetic for +: string is not numeric`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/unsupported_array_key.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/unsupported_array_key.php:2:11: invalid array key: bool keys are not supported; only int and string keys are implemented`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_key_exists_invalid_key.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_key_exists_invalid_key.php:3:6: invalid array key: bool keys are not supported; only int and string keys are implemented`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_key_exists_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_key_exists_non_array.php:2:6: unsupported call array_key_exists(): second argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_values_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_values_non_array.php:2:6: unsupported call array_values(): argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_keys_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_keys_non_array.php:2:6: unsupported call array_keys(): argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/in_array_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/in_array_non_array.php:2:6: unsupported call in_array(): second argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/in_array_strict_flag_non_bool.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/in_array_strict_flag_non_bool.php:3:6: unsupported call in_array(): strict mode argument must be bool in the current subset, got string`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/in_array_array_value.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/in_array_array_value.php:3:6: unsupported call in_array(): array needles and array values are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_search_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_search_non_array.php:2:6: unsupported call array_search(): second argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_search_strict_flag_non_bool.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_search_strict_flag_non_bool.php:3:6: unsupported call array_search(): strict mode argument must be bool in the current subset, got string`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_search_array_value.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_search_array_value.php:3:6: unsupported call array_search(): array needles and array values are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_reverse_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_reverse_non_array.php:2:6: unsupported call array_reverse(): argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_reverse_preserve_keys_non_bool.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_reverse_preserve_keys_non_bool.php:3:6: unsupported call array_reverse(): preserve_keys argument must be bool in the current subset, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/undefined_array_key.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/undefined_array_key.php:3:6: undefined array key 0`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/implicit_global_read.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/implicit_global_read.php:4:12: undefined variable '$value'`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/unsupported_global.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/unsupported_global.php:4:5: unsupported global declaration: importing globals into function scope is not implemented`.
- `cargo run -p phpc -- run tests/fixtures/milestone4/recursive_factorial.php`
  prints the committed recursive factorial output.
- `cargo run -p phpc -- run tests/fixtures/milestone4/default_parameters.php`
  prints the committed default-parameter output.
- `cargo run -p phpc -- run tests/fixtures/milestone5/symbol_table_static_variables.php`
  prints the committed static symbol-table output.
- `cargo run -p phpc -- run tests/fixtures/milestone5/dynamic_function_lookup.php`
  prints the committed dynamic user-function and builtin lookup output.
- `cargo run -p phpc -- run tests/fixtures/milestone5/class_declarations.php`
  prints the committed class metadata registration output.
- `cargo run -p phpc -- run tests/fixtures/milestone5/object_instantiation.php`
  prints the committed object truthiness, `isset`, and `print_r` output.
- `cargo run -p phpc -- run tests/fixtures/milestone5/object_properties.php`
  prints the committed public property read/write and object rendering output.
- `cargo run -p phpc -- run tests/fixtures/milestone5/object_isset.php`
  prints the committed public object-property `isset` output.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/undefined_dynamic_function.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/undefined_dynamic_function.php:3:6: undefined function missing()`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/invalid_dynamic_callable.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/invalid_dynamic_callable.php:3:6: unsupported call dynamic function call: callable expression must evaluate to string, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/runaway_recursion.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/runaway_recursion.php:3:12: maximum user function call depth exceeded for loop(): limit 128`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/duplicate_class.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/duplicate_class.php:3:1: class box is already defined`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/undefined_class.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/undefined_class.php:2:8: undefined class Missing`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/object_to_string.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/object_to_string.php:4:6: invalid string conversion: object of class Box cannot be converted to string`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/undefined_object_property.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/undefined_object_property.php:4:6: undefined property Box::$missing`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/invalid_property_target.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/invalid_property_target.php:3:6: invalid property access: cannot read property $name from int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/non_public_property_access.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/non_public_property_access.php:6:6: unsupported object property access: non-public property Box::$secret requires visibility enforcement, which is not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_function_features/unsupported_named_argument.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_function_features/unsupported_named_argument.php:5:12: unsupported named argument: named arguments are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_function_features/unsupported_strict_types.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_function_features/unsupported_strict_types.php:2:1: unsupported declare directive: strict_types is not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_dynamic_features/unsupported_variable_variable.php`
  exits 1 and reports `lex error at tests/fixtures/unsupported_dynamic_features/unsupported_variable_variable.php:3:1: unsupported variable variable: variable variables are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_dynamic_features/unsupported_require_once_expression.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_dynamic_features/unsupported_require_once_expression.php:2:7: unsupported require_once: include/require resolution and execution are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_dynamic_features/unsupported_eval.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_dynamic_features/unsupported_eval.php:2:1: unsupported eval: eval parsing and caller-scope execution are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_dynamic_features/unsupported_eval_expression.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_dynamic_features/unsupported_eval_expression.php:2:11: unsupported eval: eval parsing and caller-scope execution are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_dynamic_features/unsupported_namespace.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_dynamic_features/unsupported_namespace.php:2:1: unsupported namespace declaration: namespace-aware name resolution is not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_dynamic_features/unsupported_use_declaration.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_dynamic_features/unsupported_use_declaration.php:2:1: unsupported use declaration: namespace imports are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_dynamic_features/unsupported_namespace_qualified_function.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_dynamic_features/unsupported_namespace_qualified_function.php:2:4: unsupported namespace-qualified function name: namespace-aware function resolution is not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_dynamic_features/unsupported_namespace_qualified_class.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_dynamic_features/unsupported_namespace_qualified_class.php:2:15: unsupported namespace-qualified class name: namespace-aware class resolution is not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_syntax_features/unsupported_array_spread.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_syntax_features/unsupported_array_spread.php:3:16: unsupported array spread: spread elements are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_syntax_features/unsupported_array_reference.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_syntax_features/unsupported_array_reference.php:3:16: unsupported array reference element: references are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_syntax_features/unsupported_unset.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_syntax_features/unsupported_unset.php:3:13: unsupported unset: only direct variables like unset($name) and direct array offset removal like unset($array[$key]) are implemented; property, append, and nested unset forms are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_syntax_features/unsupported_foreach.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_syntax_features/unsupported_foreach.php:3:20: unsupported foreach: destructuring loop targets are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_syntax_features/unsupported_for.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_syntax_features/unsupported_for.php:3:12: unsupported for: comma-separated initializer, condition, or increment expression lists are not implemented; use at most one assignment or expression per header slot`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_syntax_features/unsupported_do_while.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_syntax_features/unsupported_do_while.php:2:6: unsupported do-while: do-while loops are only supported as statements in the current subset`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_syntax_features/unsupported_switch.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_syntax_features/unsupported_switch.php:3:16: unsupported switch: alternate colon/endswitch syntax is not implemented; use brace switch blocks`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_syntax_features/unsupported_alternate_if.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_syntax_features/unsupported_alternate_if.php:5:23: unsupported if: alternate if/elseif/else colon/endif syntax is not implemented; use brace blocks or single-statement bodies`.
- `cargo run -p phpc -- run tests/fixtures/milestone12/strict_identity_scalars.php`
  prints the committed scalar strict identity matrix.
- `cargo run -p phpc -- test tests/fixtures/milestone12` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone12` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone13/in_array_strict.php`
  prints the committed strict scalar `in_array` output.
- `cargo run -p phpc -- run tests/fixtures/milestone13/array_search_strict.php`
  prints the committed strict scalar `array_search` key-return output.
- `cargo run -p phpc -- test tests/fixtures/milestone13` passes with 2
  fixtures.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone13` passes
  with 2 system PHP comparisons.
- `cargo run -p phpc -- run tests/fixtures/milestone14/array_reverse.php`
  prints the committed `array_reverse` output with default integer-key
  reindexing, string-key preservation, preserve-key behavior for integer and
  string keys, and dynamic string-call coverage.
- `cargo run -p phpc -- test tests/fixtures/milestone14` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone14` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/strict_identity_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/strict_identity_array.php:2:6: unsupported comparison: strict identity for arrays is not implemented`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/strict_identity_object.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/strict_identity_object.php:5:6: unsupported comparison: strict identity for objects is not implemented`.
- `cargo run -p phpc -- run tests/fixtures/milestone6/break_while.php`
  prints `0,1,2,after:2`.
- `cargo run -p phpc -- run tests/fixtures/milestone6/continue_while.php`
  prints `1,3,4,5,after:5`.
- `cargo run -p phpc -- run tests/fixtures/milestone7/array_offset_isset.php`
  prints the committed direct array-offset `isset` output.
- `cargo run -p phpc -- run tests/fixtures/milestone7/array_key_exists.php`
  prints the committed `array_key_exists` output.
- `cargo run -p phpc -- run tests/fixtures/milestone7/array_values.php`
  prints the committed `array_values` reindexing output.
- `cargo run -p phpc -- run tests/fixtures/milestone7/array_keys.php`
  prints the committed `array_keys` key-emission output.
- `cargo run -p phpc -- run tests/fixtures/milestone7/in_array.php`
  prints the committed loose scalar `in_array` output.
- `cargo run -p phpc -- run tests/fixtures/milestone7/array_search.php`
  prints the committed loose scalar `array_search` key-return output.
- `cargo run -p phpc -- run tests/fixtures/milestone7/empty.php`
  prints the committed direct-variable and direct array-offset `empty` output.
- `cargo run -p phpc -- run tests/fixtures/milestone8/foreach_values.php`
  prints the committed value-only ordered array `foreach` output.
- `cargo run -p phpc -- run tests/fixtures/milestone8/foreach_key_values.php`
  prints the committed key/value ordered array `foreach` output.
- `cargo run -p phpc -- run tests/fixtures/milestone9/array_unset.php`
  prints the committed direct array-offset `unset` output.
- `cargo run -p phpc -- run tests/fixtures/milestone9/variable_unset.php`
  prints the committed direct variable `unset` output.
- `cargo run -p phpc -- run tests/fixtures/milestone9/multiple_unset.php`
  prints the committed multiple-operand `unset` output.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/foreach_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/foreach_non_array.php:2:1: invalid foreach: can only iterate arrays in the current subset, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/unset_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/unset_non_array.php:3:1: invalid array access: cannot unset offset on int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/unsupported_empty_complex_lvalue.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/unsupported_empty_complex_lvalue.php:3:12: unsupported call empty(): only direct variables and direct array offset operands are supported`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/break_outside_loop.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/break_outside_loop.php:2:1: invalid loop control: break cannot be used outside a loop`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/continue_outside_loop.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/continue_outside_loop.php:2:1: invalid loop control: continue cannot be used outside a loop`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_syntax_features/unsupported_break.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_syntax_features/unsupported_break.php:3:5: unsupported break: loop-depth arguments are not implemented; only 'break;' for the innermost loop is supported`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_syntax_features/unsupported_continue.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_syntax_features/unsupported_continue.php:3:5: unsupported continue: loop-depth arguments are not implemented; only 'continue;' for the innermost loop is supported`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_object_features/unsupported_class_inheritance.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_object_features/unsupported_class_inheritance.php:2:13: unsupported class inheritance: extends is not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_object_features/unsupported_anonymous_class.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_object_features/unsupported_anonymous_class.php:2:12: unsupported anonymous class: anonymous classes are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_object_features/unsupported_object_access.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_object_features/unsupported_object_access.php:6:5: unsupported method call: method dispatch is not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_object_features/unsupported_dynamic_property.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_object_features/unsupported_dynamic_property.php:7:7: unsupported dynamic property access: dynamic property names are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_object_features/unsupported_static_property.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_object_features/unsupported_static_property.php:2:4: unsupported static property access: static property storage is not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_object_features/unsupported_static_method.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_object_features/unsupported_static_method.php:2:4: unsupported static method call: static method dispatch is not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_object_features/unsupported_class_constant.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_object_features/unsupported_class_constant.php:2:4: unsupported class constant access: class constants are not implemented`.
- `cargo run -p phpc -- compile tests/fixtures/milestone3/array_literals.php --emit-ir`
  exits 1 with `arrays are supported by phpc run but not LLVM IR emission yet`.
- `cargo run -p phpc -- compile tests/fixtures/milestone10/long_array_literals.php --emit-ir`
  exits 1 with `arrays are supported by phpc run but not LLVM IR emission yet`.
- `cargo run -p phpc -- compile tests/fixtures/milestone3/array_indexing.php --emit-ir`
  exits 1 with an explicit array codegen rejection before emitting misleading
  native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone5/class_declarations.php --emit-ir`
  exits 1 with an explicit class-declaration codegen rejection before emitting
  misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone5/object_properties.php --emit-ir`
  exits 1 with an explicit class-declaration codegen rejection; focused unit
  tests cover explicit object-property read/write rejection before native
  lowering exists.
- `cargo run -p phpc -- compile tests/fixtures/runtime_errors/break_outside_loop.php --emit-ir`
  exits 1 with an explicit `break` codegen rejection before emitting
  misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/runtime_errors/continue_outside_loop.php --emit-ir`
  exits 1 with an explicit `continue` codegen rejection before emitting
  misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/runtime_errors/foreach_non_array.php --emit-ir`
  exits 1 with an explicit `foreach` codegen rejection before emitting
  misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/runtime_errors/unset_non_array.php --emit-ir`
  exits 1 with an explicit `array offset unset` codegen rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone9/variable_unset.php --emit-ir`
  exits 1 with an explicit `variable unset` codegen rejection before emitting
  misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone10/do_while_loops.php --emit-ir`
  exits 1 with an explicit `do-while loops` codegen rejection before emitting
  misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone10/switch_statements.php --emit-ir`
  exits 1 with an explicit `switch statements` codegen rejection before
  emitting misleading native code.
- `tools/run-tests.sh` passes with 123 fixtures, 51 system PHP comparisons,
  and 72 `.phpc-only` skips.
- `cargo run -p phpc -- run examples/hello.php` prints `hello`.
- `cargo run -p phpc -- compile tests/fixtures/milestone1/basic_arithmetic.php --emit-ir`
  emits LLVM IR containing native arithmetic and `printf` calls.
- `cargo run -p phpc -- compile tests/fixtures/milestone1/basic_arithmetic.php --emit-asm`
  emits native assembly through the available `cc` fallback in this environment.

Still fails:

- No known failing tests.
- `--emit-asm` does not use LLVM tools in this environment because neither
  `clang` nor `llc` is installed; the documented `cc -S` fallback is used.
- LLVM/assembly lowering intentionally rejects functions, calls, control flow,
  comparisons, dynamic values, and unknown variables.
- Leading numeric strings with trailing non-numeric characters, such as
  `"10 apples"`, are rejected instead of warning and continuing with the leading
  number. PHP's warning/notice recovery mode and exact integer-overflow
  promotion rules remain unsupported.
- Scalar strict identity (`===`, `!==`) is implemented only for the current
  scalar value subset. Strict identity for arrays, objects, resources,
  references, object handle identity, native lowering, and edge cases around
  `NAN`/`INF` and PHP-version-specific float string precision are not
  implemented; array/object strict identity operands fail with stable runtime
  diagnostics.
- Array literal spread elements and reference elements fail with stable parse
  diagnostics. `unset(...)` operands are limited to direct variables and direct
  array-offset operands on direct variables; object property removal,
  append-offset unset, and nested/complex unset operands fail with stable parse
  diagnostics.
  Comma-separated `for` initializer, condition, and increment expression lists
  fail with stable parse diagnostics; expression-form `do ... while` fails with
  a stable parse diagnostic; alternate `if`/`elseif`/`else` colon/`endif`
  syntax, expression-form `switch`, alternate switch syntax, and semicolon case
  separators fail with stable parse diagnostics. Nested
  indexed writes, complex assignment lvalues, `$array[]` as a read expression,
  string offset access, by-reference `foreach`, object iteration, destructuring
  loop targets, references, copy-on-write containers, unsupported switch inputs
  outside the scalar comparison subset, and object/resource keys are also
  unsupported, as are PHP's full boolean/null/float key coercion rules. The
  current `foreach` array forms snapshot array
  entries at loop start and do not claim PHP's full mutation/aliasing behavior
  while the iterated array is modified.
  Missing array-key reads
  fail with a stable runtime error instead of PHP's
  warning-and-`null` recovery. Direct array-offset `isset` is limited to direct
  variable targets and integer/string keys; nested/complex offset operands
  remain unsupported. Direct `empty` is limited to direct variables and direct
  array offsets; nested offsets, object properties, append offsets, general
  expression operands, unsupported key coercions, and dynamic access to
  `empty` are not implemented. `array_key_exists` is limited to integer/string
  keys and array second arguments; PHP's broader key coercions and
  warning/TypeError details are not modeled. `array_values`, `array_keys`, and
  `array_reverse` are limited to array arguments, clone values under the
  current by-value model, require a boolean `array_reverse` preserve-key flag
  when that argument is supplied, and do not yet model PHP references,
  copy-on-write containers, object handle identity preservation, resource
  values, non-bool preserve-key coercion, or native lowering.
  `array_keys` search-value filtering and strict mode are not implemented.
  `in_array` and `array_search` are limited to loose scalar searches and strict
  scalar searches when the third argument is a boolean. Strict searches
  involving array/object needles or haystack values, resource/reference
  behavior, non-bool strict-flag coercion, copy-on-write containers, exact
  native `TypeError` objects, and native lowering for function calls are not
  implemented.
  Direct variable `unset` removes only the current top-level or function-local
  symbol-table entry; it does not implement dynamic variable names, `$GLOBALS`,
  superglobal behavior, references, or copy-on-write alias effects. Multiple
  supported `unset(...)` operands execute left to right, but each operand is
  still limited to the current direct-variable/direct-array-offset subset.
  Direct array-offset `unset` treats undefined and `null` targets as no-ops
  but does not model PHP's warning for undefined variables; existing non-array
  targets fail with a stable project diagnostic instead of PHP's exact
  `Error` object. Writes to existing non-array scalar variables other than
  `null` are rejected instead of following PHP's full automatic conversion
  behavior. Negative-key auto-index behavior is not claimed beyond the current
  non-negative allocator, and arrays still reject native lowering.
- Loop-control execution is limited to statement-form `break;` and `continue;`
  inside active `while`, supported `for`, supported `do ... while`, and
  supported array `foreach` loops, plus statement-form `break;` inside
  supported `switch` statements.
  Loop-depth arguments, invalid top-level/function-level loop control recovery
  beyond the stable runtime diagnostic, `continue;` behavior inside `switch`,
  `finally`/exception behavior, and native loop-control lowering are not
  implemented.
- Conditional execution is limited to statement-form `if`/`elseif`/`else`
  bodies using brace blocks or single statements. Alternate
  `if`/`elseif`/`else` colon/`endif` syntax now fails with a stable parse
  diagnostic; alternate conditional execution, nested alternate conditional
  parsing, mixed brace/colon conditional recovery, and native conditional
  lowering are not implemented.
- Runtime errors abort the current `phpc run` command with a stable diagnostic;
  PHP `Throwable` objects, stack traces, warning/notice recovery, user error
  handlers, and preservation of partial stdout before a fatal runtime error are
  not implemented.
- Function scope imports through `global` are not implemented. Function-local
  reads of top-level variables still fail as undefined variables unless values
  are passed as arguments; PHP's warning-and-`null` recovery for undefined local
  variables is not modeled. Recursive calls use a fixed 128-frame
  user-function guard rather than PHP's native stack or memory exhaustion
  behavior, and the guard is not configurable. Default parameter support is
  limited to trailing defaults over the documented constant-expression subset;
  non-constant defaults and required parameters after defaults are rejected by
  the parser. Variadic parameters, argument unpacking, references, closures and
  arrow functions, named arguments, and `declare(strict_types=1)` now fail with
  explicit parse diagnostics; their PHP runtime semantics are not implemented.
  Dynamic function calls are limited to string-valued function names resolving
  to current user functions or the documented callable builtins; array/object
  callables, method calls, first-class callable syntax, `call_user_func`,
  namespace-qualified callable resolution, autoload interaction, and dynamic
  access to language constructs such as `isset` and `empty` are unsupported.
- Variable variables remain unsupported. `$$name` and `${...}` fail with the
  current stable lex diagnostic instead of resolving a runtime-computed symbol
  name, and dynamic symbol-table lookup from PHP values is not implemented.
- Include/require execution remains unsupported. `include`, `include_once`,
  `require`, and `require_once` fail with stable parse diagnostics; include
  path lookup, current-working-directory fallback, stream wrappers, URL
  includes, `phar://`, `_once` de-duplication, caller-scope file execution,
  included-file return values, and PHP's warning-vs-fatal recovery behavior are
  not implemented.
- Eval execution remains unsupported. Direct `eval(...)` fails with a stable
  parse diagnostic; eval-fragment parsing without `<?php`, caller-scope
  execution, `return` values from evaluated code, diagnostics inside evaluated
  strings, functions/classes declared from evaluated code, nested eval,
  include/require inside eval, and exact PHP `ParseError`/warning behavior are
  not implemented.
- Namespace execution and imports remain unsupported. `namespace` declarations,
  top-level `use` import declarations, and namespace-qualified function/class
  names fail with stable parse diagnostics; bracketed namespace blocks, global
  namespace blocks, multiple namespaces in one file, executable qualified and
  fully qualified function/class references, aliases, grouped imports, function
  imports, constant imports, trait `use` execution, autoload interaction, and
  namespace-aware native lowering are not implemented.
- Object/class execution remains narrow. `new ClassName()` works only for
  declared constructor-free classes with no constructor arguments. Public
  instance property reads, direct-variable writes, and direct
  `isset($object->prop)` checks work by static property name, but method
  dispatch, dynamic property names, static property access, static method calls,
  class constants, `::class`, `$this`, constructor execution, visibility
  enforcement for non-public properties, nested and conditional classes,
  inheritance, interfaces, traits, typed/default/multiple properties, static
  property storage, magic methods, namespaces/autoloading, object identity/handle
  aliasing, complex `isset` operands, object comparisons,
  object-to-string conversion, object callables, reflection, and native lowering
  are not implemented.

Next:

- Implement `array_merge($left, $right)` for two arrays over the current
  ordered integer/string key model.
