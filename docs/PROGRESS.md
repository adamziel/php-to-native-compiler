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
  scopes. Current static variable reads, writes, `isset($name)`, parameter
  binding, default-parameter evaluation, and direct array write materialization
  now route through named symbol-table APIs without changing static variable
  behavior.
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
  multiple `isset` operands, and keeps array offsets, dynamic property names,
  non-public property operands, complex lvalues, and method dispatch
  unsupported.
- Added explicit stable parse diagnostics, fixture coverage, and `phpc run` CLI
  snapshots for unsupported static property access, static method calls, and
  class constant access through `::`.

Tested:

- `cargo test` passes.
- `cargo test -p php_runtime` passes with 17 runtime unit tests.
- `cargo test -p php_runtime array_` passes with 4 focused array value tests.
- `cargo test -p php_runtime scalar_comparison_matrix_matches_php_8_scalar_subset`
  passes.
- `cargo test -p phpc --test runtime_errors` passes with 19 runtime error tests.
- `cargo test -p phpc --test runtime_error_cli` passes with 1 CLI snapshot test
  covering 18 representative runtime error fixtures.
- `cargo test -p phpc --test functions_and_scopes` passes with 17
  user-function scope/default-parameter tests.
- `cargo test -p phpc --test unsupported_function_features_cli` passes with 1
  CLI snapshot test covering 6 representative unsupported function-feature
  fixtures.
- `cargo test -p phpc interpreter::tests::symbol_table` passes with 3 focused
  symbol-table unit tests.
- `cargo test -p phpc --test dynamic_features` passes with 7 tests covering
  static symbol-table behavior, dynamic function lookup behavior, and
  unsupported variable-variable/include/require/eval diagnostic coverage.
- `cargo test -p phpc --test unsupported_dynamic_features_cli` passes with 1
  CLI snapshot test covering 7 unsupported variable-variable,
  include/require, and eval fixtures.
- `cargo test -p phpc --test object_model` passes with 9 tests covering class
  metadata registration, minimal object instantiation, duplicate metadata
  diagnostics, undefined-class diagnostics, constructor rejection, public
  property reads/writes, public property `isset`, and stable parse diagnostics
  for unsupported object/class syntax.
- `cargo test -p phpc --test unsupported_object_features_cli` passes with 1 CLI
  snapshot test covering 7 unsupported object/class fixtures.
- `cargo test -p phpc --test php_comparison` passes.
- `cargo test -p phpc --test milestone1 emit_ir_rejects_array` passes with
  rejection coverage for array literals, array indexing, and array assignment.
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
- `cargo run -p phpc -- test` passes with 66 fixture tests.
- `cargo run -p phpc -- test --compare-php` passes with system `php`
  installed, comparing 28 fixtures and skipping 38 `.phpc-only` fixtures.
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
- `cargo run -p phpc -- test tests/fixtures/unsupported_function_features`
  passes with 6 unsupported function-feature fixtures.
- `cargo run -p phpc -- test --compare-php
  tests/fixtures/unsupported_function_features` passes with 6 `.phpc-only`
  PHP comparisons skipped.
- `cargo run -p phpc -- test tests/fixtures/unsupported_dynamic_features`
  passes with 7 unsupported dynamic-feature fixtures.
- `cargo run -p phpc -- test --compare-php
  tests/fixtures/unsupported_dynamic_features` passes with 7 `.phpc-only` PHP
  comparisons skipped.
- `cargo run -p phpc -- test tests/fixtures/unsupported_object_features`
  passes with 7 unsupported object/class fixtures.
- `cargo run -p phpc -- test --compare-php
  tests/fixtures/unsupported_object_features` passes with 7 `.phpc-only` PHP
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
- `cargo run -p phpc -- test tests/fixtures/runtime_errors` passes with 18
  runtime error fixtures.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/undefined_variable.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/undefined_variable.php:2:6: undefined variable '$missing'`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/non_numeric_string_arithmetic.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/non_numeric_string_arithmetic.php:2:6: invalid arithmetic for +: string is not numeric`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/unsupported_array_key.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/unsupported_array_key.php:2:11: invalid array key: bool keys are not supported; only int and string keys are implemented`.
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
- `tools/run-tests.sh` passes and now includes optional system PHP comparison.
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
- Scalar comparisons do not implement strict identity (`===`, `!==`), arrays,
  objects, resources, or edge cases around `NAN`/`INF` and PHP-version-specific
  float string precision.
- Arrays do not implement nested indexed writes, complex assignment lvalues,
  `$array[]` as a read expression, string offset access, `unset`, `foreach`,
  long `array()` syntax, destructuring, spread, references, copy-on-write
  containers, object/resource keys, or PHP's full boolean/null/float key
  coercion rules. Missing array-key reads fail with a stable runtime error
  instead of PHP's warning-and-`null` recovery. Writes to existing non-array
  scalar variables other than `null` are rejected instead of following PHP's
  full automatic conversion behavior. Negative-key auto-index behavior is not
  claimed beyond the current non-negative allocator, and arrays still reject
  native lowering.
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
  access to language constructs such as `isset` are unsupported.
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
- Object/class execution remains narrow. `new ClassName()` works only for
  declared constructor-free classes with no constructor arguments. Public
  instance property reads, direct-variable writes, and direct
  `isset($object->prop)` checks work by static property name, but method
  dispatch, dynamic property names, static property access, static method calls,
  class constants, `::class`, `$this`, constructor execution, visibility
  enforcement for non-public properties, nested and conditional classes,
  inheritance, interfaces, traits, typed/default/multiple properties, static
  property storage, magic methods, namespaces/autoloading, object identity/handle
  aliasing, array-offset/complex `isset` operands, object comparisons,
  object-to-string conversion, object callables, reflection, and native lowering
  are not implemented.

Next:

- Continue Milestone 5+ by adding explicit parse diagnostics for unsupported
  namespace and `use` declaration syntax before namespace-aware name resolution
  or imports exist.
