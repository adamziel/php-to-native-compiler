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
- Extended user-function default parameter values so omitted arguments can
  evaluate bare references to previously defined unqualified constants and the
  current built-in `ARRAY_FILTER_*` constants over the existing
  constant-expression subset, with stable undefined-constant diagnostics for
  missing default references.
- Added explicit parser diagnostics, unit tests, fixture coverage, and
  `phpc run` CLI snapshots for unsupported function features: variadic
  parameters, variadic argument unpacking, references, anonymous functions,
  arrow functions, named arguments, and `declare(strict_types=1)`.
- Added explicit parser diagnostics, unit tests, fixture coverage, and
  `phpc run` CLI snapshots for unsupported user-function parameter type
  declarations and return type declarations before executable type enforcement
  exists.
- Added explicit parser diagnostics, unit tests, fixture coverage, and
  `phpc run` CLI snapshots for unsupported function-local `static $name`
  declarations before static local storage exists.
- Added explicit parser diagnostics, unit tests, fixture coverage, and
  `phpc run` CLI snapshots for unsupported magic constants such as
  `__LINE__`, `__FILE__`, `__DIR__`, `__FUNCTION__`, `__CLASS__`,
  `__TRAIT__`, `__METHOD__`, and `__NAMESPACE__` before source-aware magic
  constant evaluation exists.
- Implemented `__LINE__` as the first executable magic constant, evaluated
  from the expression token's source line in ordinary expressions, default
  parameter values, and top-level `const` declarations. Other magic constants
  and native lowering remain unsupported.
- Implemented `__FILE__` as the second executable magic constant, evaluated
  from the current `phpc run` input path string when one is available in
  ordinary expressions, default parameter values, and top-level `const`
  declarations. Path-less library execution currently evaluates it as an empty
  string, and native lowering remains unsupported.
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
- Added `array_key_first($array)` support for the current ordered array value
  model. The supported slice returns the first inserted integer or string key
  as an `int` or `string`, returns `null` for an empty array, is available
  through string-valued dynamic function calls, has a stable diagnostic for
  non-array arguments, and still rejects native lowering explicitly through the
  current function-call codegen boundary.
- Added `array_key_last($array)` support for the current ordered array value
  model. The supported slice returns the last inserted integer or string key as
  an `int` or `string`, returns `null` for an empty array, is available through
  string-valued dynamic function calls, has a stable diagnostic for non-array
  arguments, and still rejects native lowering explicitly through the current
  function-call codegen boundary.
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
- Extended `array_keys` with `array_keys($array, $search_value)` loose
  filtering over the current scalar value subset. The supported slice scans
  values in insertion order with the current PHP 8-style loose scalar
  comparison rules, emits all matching integer/string keys as values in a new
  array reindexed from zero, is available through string-valued dynamic
  function calls, and has stable diagnostics for non-array first arguments and
  unsupported array/object search values or array/object values.
- Extended `array_keys` with `array_keys($array, $search_value, true)` strict
  filtering over the current scalar value subset. The supported slice scans
  values in insertion order with current scalar strict identity rules, emits all
  matching integer/string keys as values in a new array reindexed from zero,
  treats a boolean `false` third argument as the loose path, is available
  through string-valued dynamic function calls, and has a stable diagnostic for
  non-bool strict flags.
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
- Added `array_merge($left, $right)` support for two arrays over the current
  ordered integer/string key model. The supported slice processes the left
  array then the right array in insertion order, appends integer-keyed entries
  with new integer keys starting at zero, preserves string keys, overwrites
  duplicate string-key values with right-hand values without moving the first
  string-key slot, is available through string-valued dynamic function calls,
  and has stable diagnostics for non-array first or second arguments.
- Extended `array_merge` to the current variadic array-operand subset:
  zero-argument calls return an empty ordered array, one or more array operands
  are merged left to right with the same integer reindexing and string-key
  overwrite rules, string-valued dynamic calls use the same path, and stable
  diagnostics name the first non-array positional operand.
- Added `array_combine($keys, $values)` support for the current ordered array
  value model. The supported slice accepts two equal-length arrays, reads key
  values and value values in insertion-order lockstep, converts integer and
  string key values through the current array-key normalization rules, stores
  cloned values from the second array, overwrites duplicate result keys with
  later pairs without moving the first result-key slot, supports empty arrays
  and string-valued dynamic calls, and has stable diagnostics for non-array
  operands, length mismatches, and unsupported non-int/string key values.
- Added `array_intersect_key($array, ...$arrays)` support for the current
  ordered integer/string key model. The supported slice accepts two or more
  arrays, preserves first-array entries whose normalized keys exist in every
  subsequent array, preserves the first array's keys, values, and insertion
  order, supports empty and no-match results, is available through
  string-valued dynamic function calls, preserves the source arrays, and has
  stable diagnostics for non-array positional operands.
- Added `array_diff_key($left, $right)` support for the current ordered
  integer/string key model. The supported slice accepts two arrays, preserves
  first-array entries whose normalized keys are absent from the second array,
  preserves the first array's keys, values, and insertion order, supports
  empty, all-kept, and no-match results, is available through string-valued
  dynamic function calls, preserves the source arrays, and has stable
  diagnostics for non-array operands and unsupported variadic operands.
- Extended `array_diff_key` beyond the previous two-array slice. The supported
  slice now accepts two or more array operands, preserves first-array entries
  whose normalized integer/string keys are absent from every subsequent array,
  keeps first-array keys, values, insertion order, and append index behavior,
  supports string-valued dynamic calls, preserves source arrays, and reports
  stable diagnostics for non-array positional operands including variadic
  operands.
- Added `array_diff($array, ...$arrays)` support for the current scalar value
  subset. The supported slice accepts two or more array operands, compares
  values by their current PHP string forms, preserves first-array entries whose
  scalar comparison value is absent from every subsequent array, keeps
  first-array keys, values, insertion order, and append-index behavior,
  supports string-valued dynamic calls, preserves source arrays, and reports
  stable diagnostics for non-array positional operands including variadic
  operands and for non-scalar array/object value comparisons.
- Added `array_intersect($left, $right)` support for the current scalar value
  subset. The supported slice accepts exactly two array operands, compares
  values by their current PHP string forms, preserves first-array entries whose
  scalar comparison value is present in the second array, keeps first-array
  keys, values, insertion order, and append-index behavior, supports
  string-valued dynamic calls, preserves source arrays, and reports stable
  diagnostics for non-array operands, non-scalar array/object value
  comparisons, and unsupported variadic operands.
- Extended `array_intersect` beyond the previous two-array slice. The
  supported slice now accepts two or more array operands, preserves first-array
  entries whose current scalar string-form value is present in every
  subsequent array, keeps first-array keys, values, insertion order, and
  append-index behavior, supports string-valued dynamic calls, preserves source
  arrays, and reports stable diagnostics for non-array positional operands
  including variadic operands.
- Extended `array_diff` beyond the previous two-array slice. The supported
  slice now accepts two or more array operands, preserves first-array entries
  whose current scalar string-form value is absent from every subsequent array,
  keeps first-array keys, values, insertion order, and append-index behavior,
  supports string-valued dynamic calls, preserves source arrays, and reports
  stable diagnostics for non-array positional operands including variadic
  operands.
- Added `array_unique($array)` support for the current scalar string-form
  comparison subset. The supported slice preserves the first entry for each
  distinct scalar string form, keeps first-occurrence keys and insertion order,
  derives later append behavior from kept integer keys, supports string-valued
  dynamic calls, preserves the original array, and reports stable diagnostics
  for non-array operands, unsupported non-scalar values, and unsupported sort
  flags.
- Added `array_replace($array, $replacement)` support for two arrays over the
  current ordered integer/string key model. The supported slice clones the
  first array, overwrites matching replacement keys without moving existing
  slots, appends new replacement keys in replacement insertion order, preserves
  integer keys instead of reindexing them, supports string-valued dynamic
  calls, preserves the source arrays, and reports stable diagnostics for
  non-array operands.
- Extended `array_replace` beyond the previous two-array slice. The supported
  slice now accepts one or more array operands, applies replacement arrays
  left to right, preserves integer/string keys and replacement insertion order,
  supports string-valued dynamic calls, preserves source arrays, and reports
  stable diagnostics for non-array positional operands including variadic
  replacements.
- Added `array_flip($array)` support for the current ordered array value model.
  The supported slice uses integer and string source values as result keys with
  the current string-key normalization rules, writes original integer/string
  keys as result values, overwrites duplicate flipped keys with later source
  entries without moving the first flipped-key slot, is available through
  string-valued dynamic function calls, and has stable diagnostics for
  non-array arguments and unsupported non-int/string source values.
- Added `array_fill_keys($keys, $value)` support for the current ordered array
  value model. The supported slice uses integer and string key values as result
  keys with the current string-key normalization rules, stores the supplied
  value in each result slot with the current cloned value model, overwrites
  duplicate result keys with later key entries without moving the first
  result-key slot, is available through string-valued dynamic function calls,
  and has stable diagnostics for non-array key arguments and unsupported
  non-int/string key values.
- Added `array_count_values($array)` support for the current ordered array
  value model. The supported slice counts integer and string source values
  using the current string-key normalization rules, stores integer occurrence
  counts as result values, preserves the first counted-key position when later
  values increment an existing count, is available through string-valued
  dynamic function calls, and has stable diagnostics for non-array arguments
  and unsupported non-int/string source values.
- Added `array_sum($array)` support for the current scalar numeric-coercion
  subset. The supported slice accumulates `null`, booleans, integers, floats,
  and well-formed numeric strings in insertion order, returns an integer result
  until float input or checked integer overflow promotes to float, returns
  integer zero for empty arrays, is available through string-valued dynamic
  function calls, and has stable diagnostics for non-array operands,
  non-numeric strings, and non-scalar input values.
- Added `array_product($array)` support for the current scalar numeric-coercion
  subset. The supported slice multiplies `null`, booleans, integers, floats,
  and well-formed numeric strings in insertion order, returns an integer result
  until float input or checked integer overflow promotes to float, returns
  integer one for empty arrays, is available through string-valued dynamic
  function calls, and has stable diagnostics for non-array operands,
  non-numeric strings, and non-scalar input values.
- Added `array_reduce($array, $callback)` support for the current
  string-valued callback subset. The supported slice starts with a `null`
  accumulator, invokes user-function or callable-builtin callbacks with
  accumulator/current-value arguments in insertion order, returns the final
  callback result, returns `null` for empty arrays, supports string-valued
  dynamic calls to `array_reduce`, and has stable diagnostics for non-array
  operands, non-string callbacks, and unresolved callback names.
- Extended `array_reduce` with third-argument initial value support over the
  current value model. The supported slice uses the supplied initial value as
  the first accumulator, returns it unchanged for empty arrays, works through
  string-valued dynamic calls to `array_reduce`, and keeps callback support
  limited to string-valued user-function or callable-builtin names.
- Added `array_filter($array)` support without a callback for the current
  ordered array value model. The supported slice removes values that are falsey
  under the current PHP-shaped truthiness rules, preserves original
  integer/string keys and insertion order for kept entries, is available
  through string-valued dynamic function calls, and has a stable diagnostic for
  non-array arguments.
- Extended `array_filter` with `array_filter($array, null)` support over the
  current ordered array value model. The supported slice reuses the no-callback
  falsey-value filtering path, preserves original integer/string keys and
  insertion order, and is available through string-valued dynamic calls to
  `array_filter`.
- Added `array_filter($array, $callback)` support for the first callback slice
  over the current ordered array value model. The supported slice accepts
  callbacks that evaluate to string-valued user-function or callable-builtin
  names, invokes the callback in value-only mode, preserves keys whose callback
  return value is truthy, is available when `array_filter` itself is called
  dynamically by string name, and has stable diagnostics for non-string
  callbacks, unresolved callback names, and unsupported mode flags outside the
  current subset.
- Extended `array_filter` with explicit integer mode flag `0` for the current
  `null` callback and string-valued value-only callback paths. The supported
  slice reuses the existing value filtering behavior, works through
  string-valued dynamic calls to `array_filter`, and keeps named
  `ARRAY_FILTER_*` constants and non-int mode coercions unsupported.
- Extended `array_filter` with integer mode flag `2` for the current
  string-valued key-only callback path. The supported slice invokes
  user-function or callable-builtin callbacks once per entry with the current
  integer or string key as the only argument, preserves entries whose callback
  return value is truthy, works through string-valued dynamic calls to
  `array_filter`, and keeps named `ARRAY_FILTER_USE_KEY` constants
  unsupported.
- Extended `array_filter` with integer mode flag `1` for the current
  string-valued value/key callback path. The supported slice invokes
  user-function or callable-builtin callbacks once per entry with the value and
  then the current integer or string key, preserves entries whose callback
  return value is truthy, works through string-valued dynamic calls to
  `array_filter`, and keeps named `ARRAY_FILTER_USE_BOTH` constants
  unsupported.
- Added explicit parse diagnostics, parser coverage, fixture coverage, and
  `phpc run` CLI snapshots for unsupported bare global constants such as
  `ARRAY_FILTER_USE_KEY` and `ARRAY_FILTER_USE_BOTH` before constant
  resolution exists.
- Added a narrow built-in global constant slice for exact uppercase
  `ARRAY_FILTER_USE_KEY` and `ARRAY_FILTER_USE_BOTH`. The supported slice
  parses those bare identifiers as integer values `2` and `1`, exercises them
  as named `array_filter` mode arguments with fixture and CLI coverage, keeps
  other bare constants on stable parse diagnostics, and rejects native lowering
  explicitly.
- Added the first `constant(...)` lookup boundary. The supported slice resolves
  string names `ARRAY_FILTER_USE_KEY` and `ARRAY_FILTER_USE_BOTH` to the same
  integer values as bare constants, works through string-valued dynamic calls to
  `constant`, has fixture and CLI coverage, keeps unknown constant names and
  non-string names on stable runtime diagnostics, and still rejects native
  lowering through the current function-call codegen boundary.
- Added an explicit `define(...)` boundary before user-defined constants are
  executable. Direct calls and string-valued dynamic calls to `define` now
  reach a stable unsupported-call runtime diagnostic, fixture and CLI coverage
  record the exit behavior, and direct `define(...)` calls reject native
  lowering explicitly.
- Added a first runtime-defined constant table. The supported slice accepts
  `define($name, $value)` for unqualified string names and current
  scalar/array constant values, returns `true` on successful definitions,
  resolves runtime-defined and exact built-in `ARRAY_FILTER_*` constants
  through `constant($name)`, works through string-valued dynamic calls to
  `define` and `constant`, clones array values on lookup, and has stable
  diagnostics for duplicate definitions, built-in redefinition attempts,
  unsupported names, unsupported object-containing values, unknown
  `constant(...)` names, and the legacy third `define(...)` flag.
- Added bare user constant reads for runtime-defined unqualified constants over
  the current name/value subset. The supported slice resolves bare
  runtime-defined names through the same interpreter constant table as
  `constant($name)`, keeps exact built-in `ARRAY_FILTER_*` bare constants on
  that path, clones array values on lookup, works inside user functions, and
  reports unknown bare constants with a stable runtime diagnostic.
- Added `defined($name)` support over the current built-in/runtime-defined
  constant table. The supported slice accepts the same unqualified string-name
  subset as `constant($name)`, returns true for current built-in or
  runtime-defined names, returns false for supported missing names, works
  through string-valued dynamic calls to `defined`, and has stable diagnostics
  for non-string or unsupported names.
- Added explicit stable parse diagnostics, parser coverage, fixture coverage,
  and `phpc run` CLI snapshots for unsupported top-level `const NAME = value;`
  declarations before constant-declaration execution exists.
- Implemented top-level `const NAME = value;` declarations over the current
  constant-expression and scalar/array value subset. The supported slice
  defines unqualified constants in source order, supports bare reads,
  `constant($name)`, and `defined($name)` on declared constants, clones array
  constant values on lookup through the existing constant table, reports
  duplicate declarations with stable diagnostics, keeps nested,
  namespace-aware, and dynamic-value declarations on explicit parse
  diagnostics, and rejects native lowering explicitly.
- Extended top-level `const` declarations with grouped declarators such as
  `const A = 1, B = 2;` over the same current constant-expression and value
  subset. The supported slice defines grouped constants left to right,
  preserves existing bare read, `constant($name)`, `defined($name)`, and array
  cloning behavior, reports duplicate definitions at the later duplicate
  declarator, keeps namespace-aware, class constant, dynamic-value,
  references/copy-on-write, and native lowering gaps explicit, and has fixture
  CLI coverage.
- Extended top-level `const` declaration values so the current
  constant-expression subset can reference previously defined unqualified
  constants and the exact built-in `ARRAY_FILTER_*` constant slice. Grouped
  declarations keep left-to-right behavior, so later declarators may reference
  earlier declarators in the same statement, while forward references fail with
  the existing stable undefined-constant runtime diagnostic.
- Added `array_map($callback, $array)` support for the first mapping slice over
  the current ordered array value model. The supported slice accepts callbacks
  that evaluate to string-valued user-function or callable-builtin names,
  invokes the callback in value-only mode, preserves original integer/string
  keys after the later key-preservation alignment, is available when
  `array_map` itself is called dynamically by string name, and has stable
  diagnostics for non-array operands, non-string callbacks, unresolved callback
  names, unsupported `null` callbacks, and unsupported extra input arrays
  beyond the current subset.
- Added `array_map($callback, $left, $right)` support for the first two-array
  mapping slice over the current ordered array value model. The supported slice
  accepts string-valued user-function or callable-builtin callbacks, invokes the
  callback with left/right values in insertion-order lockstep, follows PHP's
  longest-array behavior by supplying `null` for missing values from the shorter
  array, reindexes mapped results from integer key zero, is available when
  `array_map` itself is called dynamically by string name, and has stable
  diagnostics for non-array third operands and more than two string-callback
  input arrays.
- Aligned one-array `array_map($callback, $array)` with PHP key preservation
  for the current string-callback subset. The one-array form now preserves
  original integer/string keys and insertion order, including string-valued
  dynamic calls to `array_map`, while the existing two-array form remains
  reindexed from integer key zero.
- Added `array_map(null, $array)` identity mapping for one input array over
  the current ordered array value model. The supported slice preserves
  integer/string keys, insertion order, append behavior after copied integer
  keys, and is available when `array_map` itself is called dynamically by
  string name.
- Added `array_map(null, $left, $right)` support for the first multi-array
  null-callback zip slice over the current ordered array value model. The
  supported slice returns reindexed two-element arrays in insertion-order
  lockstep, follows PHP's longest-array behavior by padding the shorter input
  with `null`, supports string-valued dynamic calls to `array_map`, and was
  later extended to variadic null-callback zip arities.
- Extended `array_map(null, ...)` beyond two input arrays over the current
  ordered array value model. The supported slice returns reindexed tuple arrays
  in insertion-order lockstep up to the longest input, pads missing values from
  shorter inputs with `null`, supports string-valued dynamic calls to
  `array_map`, preserves the original input arrays, and was later followed by
  variadic string-callback mapping.
- Extended `array_map($callback, ...)` beyond two input arrays over the current
  ordered array value model. The supported slice accepts string-valued
  user-function or callable-builtin callbacks, invokes the callback with one
  insertion-order value from each input array, pads shorter inputs with `null`,
  reindexes mapped results from integer key zero, supports string-valued
  dynamic calls to `array_map`, and preserves the original arrays.
- Added `array_slice($array, $offset)` support for the current ordered array
  value model. The supported slice accepts arrays and integer offsets only,
  returns entries from the requested insertion-order offset to the end,
  supports negative offsets counted back from the end, reindexes integer-keyed
  entries from zero while preserving string keys, is available through
  string-valued dynamic function calls, and has stable diagnostics for
  non-array first arguments and non-int offsets.
- Extended `array_slice` with the integer length argument over the current
  ordered array value model. The supported slice accepts positive, zero, and
  negative integer lengths, keeps default integer-key reindexing and string-key
  preservation, supports string-valued dynamic calls, preserves the original
  array, has stable diagnostics for non-int lengths, and was later followed by
  null-length and boolean preserve-key forms.
- Extended `array_slice` with `null` length over the current ordered array
  value model. The supported slice treats `array_slice($array, $offset, null)`
  as a to-end slice, keeps default integer-key reindexing and string-key
  preservation, supports string-valued dynamic calls, preserves the original
  array, and was later followed by boolean preserve-key mode.
- Extended `array_slice` with boolean preserve-key mode over the current
  ordered integer/string key model. The supported slice preserves integer and
  string keys when the fourth argument is `true`, keeps the default
  integer-key reindexing and string-key preservation when it is `false`,
  supports `null` length with preserve-key mode, supports string-valued dynamic
  calls, preserves the original array, and has a stable diagnostic for
  non-bool preserve-key flags.
- Added `array_chunk($array, $length)` support for the current ordered array
  value model. The supported slice accepts arrays and positive integer
  lengths, splits values in insertion order into nested arrays of that size,
  reindexes every inner chunk from integer key zero regardless of original
  integer/string keys, returns an empty array for empty inputs, supports
  string-valued dynamic calls, preserves the original array, and has stable
  diagnostics for non-array inputs, non-int lengths, and non-positive lengths;
  it was later followed by boolean preserve-key mode.
- Extended `array_chunk` with boolean preserve-key mode over the current
  ordered integer/string key model. The supported slice preserves original
  integer and string keys inside each chunk when the third argument is `true`,
  keeps default chunk-key reindexing when it is `false`, supports
  string-valued dynamic calls, preserves the original array, and has a stable
  diagnostic for non-bool preserve-key flags.
- Added `array_is_list($array)` support over the current ordered integer/string
  key model. The supported slice returns true for empty arrays and arrays whose
  entries have exact ordered integer keys `0..n-1`, treats normalized numeric
  string keys such as `"0"` as integer keys through the existing key model,
  returns false for gaps, string keys, negative keys, and out-of-order integer
  keys, is available through string-valued dynamic calls, and has a stable
  diagnostic for non-array operands.
- Added `array_pad($array, $length, $value)` support over the current ordered
  integer/string key model. The supported slice returns an unchanged clone when
  `abs($length)` is not larger than the input size, right-pads for positive
  lengths, left-pads for negative lengths, preserves string keys, reindexes
  integer-keyed input entries from zero when padding is needed, supports
  string-valued dynamic calls, preserves the original array, and has stable
  diagnostics for non-array operands, non-int lengths, and oversized padding
  requests.
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
- `cargo test -p php_runtime` passes with 74 runtime unit tests.
- `cargo test -p php_runtime array_` passes with 59 focused array value tests.
- `cargo test -p php_runtime array_is_list` passes with 1 focused list-shape
  runtime test.
- `cargo test -p php_runtime array_pad` passes with 3 focused array-padding
  runtime tests.
- `cargo test -p php_runtime array_key_first` passes with 1 focused
  first-key runtime test.
- `cargo test -p php_runtime array_key_last` passes with 1 focused last-key
  runtime test.
- `cargo test -p php_runtime array_keys` passes with 5 focused key-emission,
  loose key-filtering, and strict key-filtering runtime tests.
- `cargo test -p php_runtime array_merge` passes with 2 focused
  array-combination runtime tests.
- `cargo test -p php_runtime array_combine` passes with 2 focused
  array-pairing runtime tests.
- `cargo test -p php_runtime array_intersect_key` passes with 2 focused
  array-key-set runtime tests.
- `cargo test -p php_runtime array_diff_key` passes with 2 focused
  array-key-difference runtime tests.
- `cargo test -p php_runtime array_diff` passes with 5 focused array
  difference runtime tests covering key-difference, scalar value-difference,
  and variadic scalar value-difference behavior.
- `cargo test -p php_runtime array_intersect` passes with 5 focused array
  intersection runtime tests covering key-intersection, scalar
  value-intersection, and variadic value-intersection behavior.
- `cargo test -p php_runtime array_unique` passes with 2 focused array
  deduplication runtime tests.
- `cargo test -p php_runtime array_replace` passes with 2 focused array
  replacement runtime tests.
- `cargo test -p php_runtime array_flip` passes with 2 focused array-transform
  runtime tests.
- `cargo test -p php_runtime array_fill_keys` passes with 2 focused
  array-transform runtime tests.
- `cargo test -p php_runtime array_count_values` passes with 2 focused
  array-counting runtime tests.
- `cargo test -p php_runtime array_sum` passes with 2 focused numeric
  aggregation runtime tests.
- `cargo test -p php_runtime array_product` passes with 2 focused numeric
  product runtime tests.
- `cargo test -p php_runtime array_filter` passes with 1 focused
  array-filtering runtime test.
- `cargo test -p php_runtime array_slice` passes with 4 focused array-slicing
  runtime tests.
- `cargo test -p php_runtime array_chunk` passes with 2 focused array-chunking
  runtime tests.
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
  covering 112 representative runtime error fixtures.
- `cargo test -p phpc --test strict_identity` passes with 4 tests covering
  scalar strict identity execution, array/object strict identity diagnostics,
  and LLVM IR rejection.
- `cargo test -p phpc --test comparison_refinements_cli` passes with 1 CLI
  snapshot test covering the Milestone 12 strict identity fixture.
- `cargo test -p phpc --test functions_and_scopes` passes with 21
  user-function scope/default-parameter tests.
- `cargo test -p phpc --test unsupported_function_features_cli` passes with 1
  CLI snapshot test covering 8 representative unsupported function-feature
  fixtures.
- `cargo test -p phpc interpreter::tests::symbol_table` passes with 4 focused
  symbol-table unit tests.
- `cargo test -p phpc --test dynamic_features` passes with 31 tests covering
  static symbol-table behavior, dynamic function lookup behavior,
  runtime-defined constant, bare constant, and `defined(...)` behavior, and
  unsupported variable-variable/include/require/eval/namespace/use,
  single and grouped top-level const declarations, const declaration reference
  behavior, and namespace-qualified name diagnostic coverage.
- `cargo test -p phpc --test user_constants_cli` passes with 1 CLI snapshot
  test covering the Milestone 61, Milestone 62, Milestone 63, Milestone 65,
  Milestone 66, and Milestone 67 user constant fixtures.
- `cargo test -p phpc --test unsupported_dynamic_features_cli` passes with 1
  CLI snapshot test covering 14 unsupported variable-variable,
  include/require, eval, top-level const declaration, namespace, use, and
  namespace-qualified name fixtures.
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
- `cargo test -p phpc --test array_key_first` passes with first integer/string
  key return behavior, empty-array `null` behavior, dynamic string-call
  coverage, stable diagnostics for non-array arguments, and LLVM IR rejection
  coverage.
- `cargo test -p phpc --test array_key_last` passes with last integer/string
  key return behavior, empty-array `null` behavior, dynamic string-call
  coverage, stable diagnostics for non-array arguments, and LLVM IR rejection
  coverage.
- `cargo test -p phpc --test array_values` passes with `array_values`
  reindexing behavior, dynamic string-call coverage, original-array
  preservation, and stable diagnostics for non-array arguments.
- `cargo test -p phpc --test array_keys` passes with `array_keys`
  integer/string key emission, loose and strict scalar search-value filtering,
  dynamic string-call coverage, original-array preservation, stable diagnostics
  for non-array arguments, non-bool strict flags, and unsupported search-value
  comparison gaps, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_reverse` passes with `array_reverse`
  reverse-order behavior, numeric-key reindexing, string-key preservation,
  preserve-key behavior for integer and string keys, dynamic string-call
  coverage, original-array preservation, non-array diagnostics, non-bool
  `preserve_keys` diagnostics, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_merge` passes with zero-argument,
  one-array, two-array, and variadic `array_merge` behavior, integer-key
  reindexing, string-key overwrite behavior, dynamic string-call coverage,
  original-array preservation, first/second/third non-array diagnostics, and
  LLVM IR rejection coverage.
- `cargo test -p phpc --test array_flip` passes with integer/string
  value-to-key conversion, duplicate-key overwrite behavior, dynamic
  string-call coverage, original-array preservation, non-array diagnostics,
  unsupported-value diagnostics, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_fill_keys` passes with integer/string
  key-value conversion, duplicate-key overwrite behavior, dynamic string-call
  coverage, original-key-array preservation, non-array diagnostics,
  unsupported-key diagnostics, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_count_values` passes with integer/string
  value counting, string-key normalization, duplicate-count increments,
  dynamic string-call coverage, original-array preservation, non-array
  diagnostics, unsupported-value diagnostics, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_sum` passes with scalar numeric
  accumulation, integer/float result behavior, empty-array behavior, dynamic
  string-call coverage, non-array diagnostics, unsupported-value diagnostics,
  and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_product` passes with scalar numeric
  multiplication, integer/float result behavior, zero and empty-array behavior,
  dynamic string-call coverage, non-array diagnostics, unsupported-value
  diagnostics, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_reduce` passes with accumulator/current
  value callback invocation, `null` initial accumulator behavior, empty-array
  `null` behavior, third-argument initial accumulator behavior, empty-array
  initial-value return behavior, callback-returned array accumulators, dynamic
  string-call coverage, original-array preservation, non-array/callback
  diagnostics, callback arity diagnostics, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_filter` passes with falsey-value removal,
  explicit `null` callback falsey filtering, value-only string callback
  execution for user functions and callable builtins, explicit integer mode
  flag `0` for the current null-callback and string-callback paths, integer
  mode flag `2` for key-only string callbacks, integer mode flag `1` for
  value/key string callbacks, key preservation, dynamic string-call coverage,
  original-array preservation, non-array/callback/mode diagnostics, and LLVM
  IR rejection coverage.
- `cargo test -p phpc --test array_map` passes with one-array null-callback
  identity mapping, variadic null-callback zip mapping with longest-array
  `null` padding and integer reindexing, one-array value-only string callback
  execution and key preservation, two-array string-callback execution with
  longest-array `null` padding and integer reindexing, dynamic string-call
  coverage, original-array preservation, non-array/callback/extra array
  diagnostics, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_slice` passes with `array_slice` offset,
  integer-length, null-length, and boolean preserve-key behavior,
  positive/negative/out-of-range offsets, positive/zero/negative lengths,
  default integer-key reindexing, string-key preservation, preserved integer
  keys, dynamic string-call coverage, original-array preservation,
  non-array/non-int/non-null diagnostics, non-bool preserve-key diagnostics,
  and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_chunk` passes with `array_chunk` positive
  length behavior, default chunk-key reindexing for integer and string keys,
  boolean preserve-key behavior, empty-input behavior, dynamic string-call
  coverage, original-array preservation, non-array/non-int/non-positive
  diagnostics, non-bool preserve-key diagnostics, and LLVM IR rejection
  coverage.
- `cargo test -p phpc --test array_is_list` passes with empty-list true
  behavior, exact zero-based ordered integer-key detection, normalized numeric
  string-key behavior, false results for gaps/string keys/negative
  keys/out-of-order integer keys, dynamic string-call coverage, non-array
  diagnostics, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_pad` passes with positive right-padding,
  negative left-padding, no-op key-shape preservation, empty-array padding,
  dynamic string-call coverage, original-array preservation, non-array/non-int
  diagnostics, oversized-padding diagnostics, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_combine` passes with integer/string
  key-value conversion, duplicate-key overwrite behavior, empty-array
  behavior, dynamic string-call coverage, original-array preservation,
  non-array diagnostics, length-mismatch diagnostics, unsupported-key
  diagnostics, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_intersect_key` passes with first-array
  key/value preservation, normalized integer/string key matching, empty and
  no-match result behavior, dynamic string-call coverage, original-array
  preservation, variadic intersection behavior, non-array positional operand
  diagnostics, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_diff_key` passes with first-array key/value
  preservation for keys absent from the second array or from all variadic
  operands, normalized integer/string key matching, empty, all-kept, and
  no-match result behavior, dynamic string-call coverage, original-array
  preservation, non-array positional operand diagnostics, and LLVM IR rejection
  coverage.
- `cargo test -p phpc --test array_diff` passes with first-array key/value
  preservation for scalar values absent from the second array or from all
  variadic operands, PHP string-form value comparison coverage, empty,
  all-kept, and no-match result behavior, dynamic string-call coverage,
  original-array preservation, non-array positional operand diagnostics,
  unsupported non-scalar comparison diagnostics, and LLVM IR rejection
  coverage.
- `cargo test -p phpc --test array_intersect` passes with first-array
  key/value preservation for scalar values present in every subsequent array,
  PHP string-form value comparison coverage, empty, all-kept, no-match, and
  variadic result behavior, dynamic string-call coverage, original-array
  preservation, non-array positional operand diagnostics, unsupported
  non-scalar comparison diagnostics, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_unique` passes with first-occurrence
  key/value preservation, PHP string-form value deduplication coverage,
  dynamic string-call coverage, original-array preservation, non-array operand
  diagnostics, unsupported non-scalar comparison diagnostics, unsupported sort
  flag diagnostics, and LLVM IR rejection coverage.
- `cargo test -p phpc --test array_replace` passes with key-preserving
  replacement overwrite behavior, variadic left-to-right replacement behavior,
  one-array clone behavior, new-key insertion order, dynamic string-call
  coverage, source-array preservation, non-array positional operand
  diagnostics, and LLVM IR rejection coverage.
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
- `cargo test -p phpc --test array_combination_builtins_cli` passes with 1 CLI
  snapshot test covering the Milestone 15 `array_merge` fixture.
- `cargo test -p phpc --test array_key_filtering_builtins_cli` passes with 1
  CLI snapshot test covering the Milestone 16 `array_keys` filter fixture.
- `cargo test -p phpc --test array_key_introspection_builtins_cli` passes with
  1 CLI snapshot test covering the Milestone 17 `array_key_first` and
  `array_key_last` fixtures.
- `cargo test -p phpc --test array_transform_builtins_cli` passes with 1 CLI
  snapshot test covering the Milestone 18 `array_flip` and `array_fill_keys`
  fixtures.
- `cargo test -p phpc --test array_counting_builtins_cli` passes with 1 CLI
  snapshot test covering the Milestone 19 `array_count_values` fixture.
- `cargo test -p phpc --test array_numeric_aggregation_builtins_cli` passes
  with 1 CLI snapshot test covering the Milestone 49 `array_sum` fixture and
  the Milestone 50 `array_product` fixture.
- `cargo test -p phpc --test array_reduction_builtins_cli` passes with 1 CLI
  snapshot test covering the Milestone 51 and Milestone 52 `array_reduce`
  fixtures.
- `cargo test -p phpc --test array_filtering_builtins_cli` passes with 1 CLI
  snapshot test covering the Milestone 20, Milestone 21, Milestone 53,
  Milestone 54, Milestone 55, and Milestone 56 `array_filter` fixtures.
- `cargo test -p phpc --test array_mapping_builtins_cli` passes with CLI
  snapshot tests covering the Milestone 22, Milestone 23, Milestone 25,
  Milestone 26, Milestone 27, and Milestone 28 `array_map` fixtures.
- `cargo test -p phpc --test array_slicing_builtins_cli` passes with 1 CLI
  snapshot test covering the Milestone 29, Milestone 30, Milestone 31, and
  Milestone 32 `array_slice` fixtures.
- `cargo test -p phpc --test array_chunking_builtins_cli` passes with 1 CLI
  snapshot test covering the Milestone 33 `array_chunk` fixture.
- `cargo test -p phpc --test array_chunking_preserve_keys_builtins_cli` passes
  with 1 CLI snapshot test covering the Milestone 34 `array_chunk` fixture.
- `cargo test -p phpc --test array_list_introspection_builtins_cli` passes with
  1 CLI snapshot test covering the Milestone 35 `array_is_list` fixture.
- `cargo test -p phpc --test array_padding_builtins_cli` passes with 1 CLI
  snapshot test covering the Milestone 36 `array_pad` fixture.
- `cargo test -p phpc --test array_pairing_builtins_cli` passes with 1 CLI
  snapshot test covering the Milestone 37 `array_combine` fixture.
- `cargo test -p phpc --test array_key_set_builtins_cli` passes with 1 CLI
  snapshot test covering the Milestone 38 `array_intersect_key` fixture.
- `cargo test -p phpc --test array_key_set_variadic_builtins_cli` passes with
  1 CLI snapshot test covering the Milestone 40 variadic
  `array_intersect_key` fixture.
- `cargo test -p phpc --test array_key_difference_builtins_cli` passes with 1
  CLI snapshot test covering the Milestone 39 `array_diff_key` fixture.
- `cargo test -p phpc --test array_key_difference_variadic_builtins_cli`
  passes with 1 CLI snapshot test covering the Milestone 41 variadic
  `array_diff_key` fixture.
- `cargo test -p phpc --test array_value_difference_builtins_cli` passes with
  1 CLI snapshot test covering the Milestone 42 `array_diff` fixture and the
  Milestone 45 variadic `array_diff` fixture.
- `cargo test -p phpc --test array_value_intersection_builtins_cli` passes with
  1 CLI snapshot test covering the Milestone 43 `array_intersect` fixture and
  the Milestone 44 variadic `array_intersect` fixture.
- `cargo test -p phpc --test array_value_deduplication_builtins_cli` passes
  with 1 CLI snapshot test covering the Milestone 46 `array_unique` fixture.
- `cargo test -p phpc --test array_replacement_builtins_cli` passes with 1 CLI
  snapshot test covering the Milestone 47 `array_replace` fixture.
- `cargo test -p phpc --test array_replacement_variadic_builtins_cli` passes
  with 1 CLI snapshot test covering the Milestone 48 variadic `array_replace`
  fixture.
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
- `cargo run -p phpc -- test` passes with 245 fixture tests.
- `cargo run -p phpc -- test --compare-php` passes with system `php`
  installed, comparing 100 fixtures and skipping 145 `.phpc-only` fixtures.
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
  passes with 8 unsupported function-feature fixtures.
- `cargo run -p phpc -- test --compare-php
  tests/fixtures/unsupported_function_features` passes with 8 `.phpc-only`
  PHP comparisons skipped.
- `cargo run -p phpc -- test tests/fixtures/unsupported_dynamic_features`
  passes with 12 unsupported dynamic-feature fixtures.
- `cargo run -p phpc -- test --compare-php
  tests/fixtures/unsupported_dynamic_features` passes with 12 `.phpc-only` PHP
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
- `cargo run -p phpc -- test tests/fixtures/milestone61` passes with 1
  runtime-defined constant fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone61` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone61/runtime_defined_constants.php`
  prints the committed `define(...)`/`constant(...)` output for scalar values,
  array constants, function-scope lookup, and string-valued dynamic calls.
- `cargo run -p phpc -- test tests/fixtures/milestone62` passes with 1 bare
  runtime-defined constant fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone62` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone62/bare_runtime_defined_constants.php`
  prints the committed bare constant output for scalar values, cloned array
  constants, function-scope lookup, built-in constants, and string-valued
  dynamic `define` calls.
- `cargo run -p phpc -- test tests/fixtures/milestone63` passes with 1
  `defined(...)` constant-introspection fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone63` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone63/defined_constants.php`
  prints the committed `defined(...)` output for built-in constants,
  before/after runtime definitions, supported missing names, function-scope
  lookup, and string-valued dynamic calls.
- `cargo run -p phpc -- test tests/fixtures/milestone65` passes with 1
  top-level const declaration fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone65` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone65/top_level_const_declarations.php`
  prints the committed output for scalar const declarations, array const
  cloning, bare reads, `constant($name)`, `defined($name)`, and function-body
  reads after declaration.
- `cargo run -p phpc -- test tests/fixtures/milestone66` passes with 1
  grouped top-level const declaration fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone66` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone66/grouped_const_declarations.php`
  prints the committed output for grouped scalar and array const declarations,
  cloned array constant reads, `defined($name)`, and function-body reads after
  declaration.
- `cargo run -p phpc -- test tests/fixtures/milestone67` passes with 1 const
  declaration reference fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone67` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone67/const_declaration_references.php`
  prints the committed output for const declaration values that reference
  previous `define(...)` constants, previous `const` declarations, earlier
  grouped declarators, and the current built-in `ARRAY_FILTER_*` constants.
- `cargo run -p phpc -- test tests/fixtures/unsupported_dynamic_features`
  passes with 14 unsupported dynamic feature fixtures.
- `cargo run -p phpc -- test --compare-php tests/fixtures/unsupported_dynamic_features`
  passes with 14 `.phpc-only` skips.
- `cargo run -p phpc -- run tests/fixtures/unsupported_dynamic_features/unsupported_const_declaration.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_dynamic_features/unsupported_const_declaration.php:2:10: unsupported const declaration: namespace-qualified constant declarations are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/unsupported_dynamic_features/unsupported_const_value.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_dynamic_features/unsupported_const_value.php:2:18: const declaration values only support constant expressions in the current subset`.
- `cargo run -p phpc -- test tests/fixtures/runtime_errors` passes with 112
  runtime error fixtures.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/const_duplicate.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/const_duplicate.php:3:1: constant APP_NAME is already defined`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/const_grouped_duplicate.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/const_grouped_duplicate.php:2:47: constant APP_NAME is already defined`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/const_forward_reference.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/const_forward_reference.php:2:17: undefined constant LATER`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/undefined_variable.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/undefined_variable.php:2:6: undefined variable '$missing'`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/non_numeric_string_arithmetic.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/non_numeric_string_arithmetic.php:2:6: invalid arithmetic for +: string is not numeric`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/unsupported_array_key.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/unsupported_array_key.php:2:11: invalid array key: bool keys are not supported; only int and string keys are implemented`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/define_duplicate.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/define_duplicate.php:3:1: constant APP_NAME is already defined`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/defined_non_string.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/defined_non_string.php:2:6: unsupported call defined(): name argument must be string in the current subset, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/defined_unsupported_name.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/defined_unsupported_name.php:2:6: unsupported call defined(): constant name must be a non-empty unqualified identifier in the current subset, got 123BAD`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_key_exists_invalid_key.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_key_exists_invalid_key.php:3:6: invalid array key: bool keys are not supported; only int and string keys are implemented`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_key_exists_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_key_exists_non_array.php:2:6: unsupported call array_key_exists(): second argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_key_first_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_key_first_non_array.php:2:6: unsupported call array_key_first(): argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_key_last_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_key_last_non_array.php:2:6: unsupported call array_key_last(): argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_is_list_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_is_list_non_array.php:2:6: unsupported call array_is_list(): argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_values_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_values_non_array.php:2:6: unsupported call array_values(): argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_keys_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_keys_non_array.php:2:6: unsupported call array_keys(): argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_keys_array_search_value.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_keys_array_search_value.php:3:6: unsupported call array_keys(): array search values and array values are not implemented`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_keys_strict_flag_non_bool.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_keys_strict_flag_non_bool.php:3:6: unsupported call array_keys(): strict mode argument must be bool in the current subset, got string`.
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
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_slice_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_slice_non_array.php:2:6: unsupported call array_slice(): first argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_slice_offset_non_int.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_slice_offset_non_int.php:3:6: unsupported call array_slice(): offset argument must be int in the current subset, got string`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_slice_length_non_int.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_slice_length_non_int.php:3:6: unsupported call array_slice(): length argument must be int or null in the current subset, got string`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_slice_preserve_keys_non_bool.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_slice_preserve_keys_non_bool.php:3:6: unsupported call array_slice(): preserve_keys argument must be bool in the current subset, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_chunk_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_chunk_non_array.php:2:6: unsupported call array_chunk(): first argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_chunk_length_non_int.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_chunk_length_non_int.php:3:6: unsupported call array_chunk(): length argument must be int in the current subset, got string`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_chunk_length_non_positive.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_chunk_length_non_positive.php:3:6: unsupported call array_chunk(): length argument must be greater than 0 in the current subset, got 0`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_chunk_preserve_keys_non_bool.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_chunk_preserve_keys_non_bool.php:3:6: unsupported call array_chunk(): preserve_keys argument must be bool in the current subset, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_pad_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_pad_non_array.php:2:6: unsupported call array_pad(): first argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_pad_length_non_int.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_pad_length_non_int.php:3:6: unsupported call array_pad(): length argument must be int in the current subset, got string`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_pad_length_too_large.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_pad_length_too_large.php:2:6: unsupported call array_pad(): padding length must be at most 1048576 in the current subset, got 1048577`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_merge_first_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_merge_first_non_array.php:3:6: unsupported call array_merge(): first argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_merge_second_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_merge_second_non_array.php:3:6: unsupported call array_merge(): second argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_merge_third_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_merge_third_non_array.php:4:6: unsupported call array_merge(): third argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_replace_first_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_replace_first_non_array.php:3:6: unsupported call array_replace(): first argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_replace_second_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_replace_second_non_array.php:3:6: unsupported call array_replace(): second argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_replace_third_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_replace_third_non_array.php:4:6: unsupported call array_replace(): third argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_combine_first_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_combine_first_non_array.php:3:6: unsupported call array_combine(): first argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_combine_second_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_combine_second_non_array.php:3:6: unsupported call array_combine(): second argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_combine_length_mismatch.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_combine_length_mismatch.php:4:6: unsupported call array_combine(): keys and values must have the same number of elements in the current subset, got 2 and 1`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_combine_unsupported_key.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_combine_unsupported_key.php:4:6: unsupported call array_combine(): key values must be int or string in the current subset, got bool`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_intersect_key_first_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_intersect_key_first_non_array.php:3:6: unsupported call array_intersect_key(): first argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_intersect_key_second_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_intersect_key_second_non_array.php:3:6: unsupported call array_intersect_key(): second argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_intersect_key_third_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_intersect_key_third_non_array.php:4:6: unsupported call array_intersect_key(): third argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_diff_key_first_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_diff_key_first_non_array.php:3:6: unsupported call array_diff_key(): first argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_diff_key_second_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_diff_key_second_non_array.php:3:6: unsupported call array_diff_key(): second argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_diff_key_third_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_diff_key_third_non_array.php:4:6: unsupported call array_diff_key(): third argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_diff_first_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_diff_first_non_array.php:3:6: unsupported call array_diff(): first argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_diff_second_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_diff_second_non_array.php:3:6: unsupported call array_diff(): second argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_diff_array_value.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_diff_array_value.php:4:6: unsupported call array_diff(): values must be scalar in the current subset, got array`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_diff_third_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_diff_third_non_array.php:4:6: unsupported call array_diff(): third argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_intersect_first_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_intersect_first_non_array.php:3:6: unsupported call array_intersect(): first argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_intersect_second_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_intersect_second_non_array.php:3:6: unsupported call array_intersect(): second argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_intersect_array_value.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_intersect_array_value.php:4:6: unsupported call array_intersect(): values must be scalar in the current subset, got array`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_intersect_third_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_intersect_third_non_array.php:4:6: unsupported call array_intersect(): third argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_unique_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_unique_non_array.php:2:6: unsupported call array_unique(): argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_unique_array_value.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_unique_array_value.php:3:6: unsupported call array_unique(): values must be scalar in the current subset, got array`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_unique_sort_flag.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_unique_sort_flag.php:3:6: unsupported call array_unique(): sort flags are not supported in the current subset`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_flip_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_flip_non_array.php:2:6: unsupported call array_flip(): argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_flip_unsupported_value.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_flip_unsupported_value.php:3:6: unsupported call array_flip(): values must be int or string in the current subset, got bool`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_fill_keys_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_fill_keys_non_array.php:2:6: unsupported call array_fill_keys(): first argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_fill_keys_unsupported_key.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_fill_keys_unsupported_key.php:3:6: unsupported call array_fill_keys(): key values must be int or string in the current subset, got bool`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_count_values_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_count_values_non_array.php:2:6: unsupported call array_count_values(): argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_count_values_unsupported_value.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_count_values_unsupported_value.php:3:6: unsupported call array_count_values(): values must be int or string in the current subset, got bool`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_sum_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_sum_non_array.php:2:6: unsupported call array_sum(): argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_sum_non_numeric_string.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_sum_non_numeric_string.php:3:6: unsupported call array_sum(): values must be numeric in the current subset, got non-numeric string`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_sum_array_value.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_sum_array_value.php:3:6: unsupported call array_sum(): values must be numeric scalar in the current subset, got array`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_product_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_product_non_array.php:2:6: unsupported call array_product(): argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_product_non_numeric_string.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_product_non_numeric_string.php:3:6: unsupported call array_product(): values must be numeric in the current subset, got non-numeric string`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_product_array_value.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_product_array_value.php:3:6: unsupported call array_product(): values must be numeric scalar in the current subset, got array`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_reduce_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_reduce_non_array.php:2:6: unsupported call array_reduce(): first argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_reduce_callback_non_string.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_reduce_callback_non_string.php:3:6: unsupported call array_reduce(): callback must evaluate to string, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_reduce_callback_undefined.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_reduce_callback_undefined.php:3:6: undefined function missing_reduce()`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_filter_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_filter_non_array.php:2:6: unsupported call array_filter(): argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_filter_callback_non_string.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_filter_callback_non_string.php:3:6: unsupported call array_filter(): callback must evaluate to string, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_filter_callback_undefined.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_filter_callback_undefined.php:3:6: undefined function missing_filter()`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_filter_mode_unsupported.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_filter_mode_unsupported.php:3:6: unsupported call array_filter(): mode flag must be integer 0, 1, or 2 in the current subset, got 3`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_map_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_map_non_array.php:2:6: unsupported call array_map(): second argument must be array, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_map_callback_non_string.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_map_callback_non_string.php:3:6: unsupported call array_map(): callback must evaluate to string, got int`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_map_callback_undefined.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_map_callback_undefined.php:3:6: undefined function missing_map()`.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/array_map_third_non_array.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/array_map_third_non_array.php:3:6: unsupported call array_map(): third argument must be array, got int`.
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
- `cargo run -p phpc -- run tests/fixtures/milestone15/array_merge.php`
  prints the committed `array_merge` output with zero-argument empty-array
  behavior, one-array reindexing, variadic integer-key reindexing, string-key
  overwrite behavior, original-array preservation, and dynamic string-call
  coverage.
- `cargo run -p phpc -- test tests/fixtures/milestone15` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone15` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone16/array_keys_filter.php`
  prints the committed loose and strict scalar `array_keys` filter output.
- `cargo run -p phpc -- test tests/fixtures/milestone16` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone16` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone17/array_key_first.php`
  prints the committed `array_key_first` output with string-key return,
  normalized integer-key return, empty-array `null`, and dynamic string-call
  coverage.
- `cargo run -p phpc -- run tests/fixtures/milestone17/array_key_last.php`
  prints the committed `array_key_last` output with appended integer-key
  return, string-key return, normalized integer-key return, empty-array `null`,
  and dynamic string-call coverage.
- `cargo run -p phpc -- test tests/fixtures/milestone17` passes with 2
  fixtures.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone17` passes
  with 2 system PHP comparisons.
- `cargo run -p phpc -- run tests/fixtures/milestone18/array_flip.php` prints
  the committed `array_flip` output with integer/string value-to-key
  conversion, duplicate-key overwrites, original-array preservation, append
  behavior after flipped integer keys, and dynamic string-call coverage.
- `cargo run -p phpc -- run tests/fixtures/milestone18/array_fill_keys.php`
  prints the committed `array_fill_keys` output with integer/string
  key-value conversion, duplicate-key overwrites, original-key-array
  preservation, append behavior after filled integer keys, and dynamic
  string-call coverage.
- `cargo run -p phpc -- test tests/fixtures/milestone18` passes with 2
  fixtures.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone18` passes
  with 2 system PHP comparisons.
- `cargo run -p phpc -- run tests/fixtures/milestone19/array_count_values.php`
  prints the committed `array_count_values` output with integer/string value
  counting, string-key normalization, duplicate-count increments,
  original-array preservation, append behavior after counted integer keys, and
  dynamic string-call coverage.
- `cargo run -p phpc -- test tests/fixtures/milestone19` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone19` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone49/array_sum.php` prints
  the committed `array_sum` output with scalar numeric accumulation,
  integer/float result behavior, empty-array behavior, preserved source
  values, and dynamic string-call coverage.
- `cargo run -p phpc -- test tests/fixtures/milestone49` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone49` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone50/array_product.php`
  prints the committed `array_product` output with scalar numeric
  multiplication, integer/float result behavior, zero and empty-array behavior,
  preserved source values, and dynamic string-call coverage.
- `cargo run -p phpc -- test tests/fixtures/milestone50` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone50` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone51/array_reduce.php`
  prints the committed `array_reduce` output with accumulator/current-value
  callback invocation, empty-array `null` behavior, callback-returned array
  accumulators, source preservation, and dynamic string-call coverage.
- `cargo run -p phpc -- test tests/fixtures/milestone51` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone51` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone20/array_filter.php`
  prints the committed `array_filter` output with falsey-value removal, key
  preservation, append behavior after preserved integer keys, original-array
  preservation, and dynamic string-call coverage.
- `cargo run -p phpc -- test tests/fixtures/milestone20` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone20` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone21/array_filter_callback.php`
  prints the committed `array_filter` callback output with string-named
  user-function callbacks, key preservation, callback return truthiness,
  append behavior after preserved integer keys, and string-valued dynamic calls
  to `array_filter`.
- `cargo run -p phpc -- run tests/fixtures/milestone21/array_filter_builtin_callback.php`
  prints the committed `array_filter` output with a string-named callable
  builtin callback.
- `cargo run -p phpc -- test tests/fixtures/milestone21` passes with 2
  fixtures.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone21` passes
  with 2 system PHP comparisons.
- `cargo run -p phpc -- run tests/fixtures/milestone53/array_filter_null_callback.php`
  prints the committed `array_filter($array, null)` output with falsey-value
  removal, key preservation, and string-valued dynamic calls to `array_filter`.
- `cargo run -p phpc -- test tests/fixtures/milestone53` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone53` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone54/array_filter_mode_zero.php`
  prints the committed `array_filter(..., 0)` output for the current
  null-callback and string-callback value-only paths, including key
  preservation and string-valued dynamic calls to `array_filter`.
- `cargo run -p phpc -- test tests/fixtures/milestone54` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone54` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone55/array_filter_use_key.php`
  prints the committed `array_filter(..., 2)` output for the current
  string-valued key-only callback path, including integer/string key callback
  arguments, key preservation, callable-builtin callbacks, and string-valued
  dynamic calls to `array_filter`.
- `cargo run -p phpc -- test tests/fixtures/milestone55` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone55` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone56/array_filter_use_both.php`
  prints the committed `array_filter(..., 1)` output for the current
  string-valued value/key callback path, including value-then-key callback
  arguments, key preservation, `null` callback mode interaction, and
  string-valued dynamic calls to `array_filter`.
- `cargo run -p phpc -- test tests/fixtures/milestone56` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone56` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone22/array_map.php` prints
  the committed `array_map` output with string-named user-function callbacks,
  one-array key preservation, append behavior after preserved integer keys,
  original-array preservation, and string-valued dynamic calls to `array_map`.
- `cargo run -p phpc -- test tests/fixtures/milestone22` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone22` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone23/array_map_two_arrays.php`
  prints the committed two-array `array_map` output with longest-array `null`
  padding, integer reindexing, original-array preservation, and string-valued
  dynamic calls to `array_map`.
- `cargo run -p phpc -- test tests/fixtures/milestone23` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone23` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone25/array_map_null_callback.php`
  prints the committed one-array `array_map(null, $array)` identity output
  with key preservation, append behavior after copied integer keys,
  original-array preservation, and string-valued dynamic calls to `array_map`.
- `cargo run -p phpc -- test tests/fixtures/milestone25` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone25` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone26/array_map_null_zip.php`
  prints the committed two-array `array_map(null, $left, $right)` zip output
  with longest-array `null` padding, integer reindexing, append behavior after
  copied integer keys, original-array preservation, and string-valued dynamic
  calls to `array_map`.
- `cargo run -p phpc -- test tests/fixtures/milestone26` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone26` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone27/array_map_null_variadic.php`
  prints the committed variadic `array_map(null, ...)` zip output with
  longest-array `null` padding, integer reindexing, append behavior after
  copied integer keys, original-array preservation, and string-valued dynamic
  calls to `array_map`.
- `cargo run -p phpc -- test tests/fixtures/milestone27` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone27` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone28/array_map_variadic_callback.php`
  prints the committed variadic string-callback `array_map` output with
  longest-array `null` padding, integer reindexing, original-array
  preservation, string-valued dynamic calls to `array_map`, and callable
  builtin callback coverage.
- `cargo run -p phpc -- test tests/fixtures/milestone28` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone28` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone29/array_slice.php`
  prints the committed offset-only `array_slice` output with positive,
  negative, and out-of-range offsets, integer-key reindexing, string-key
  preservation, original-array preservation, and string-valued dynamic calls to
  `array_slice`.
- `cargo run -p phpc -- test tests/fixtures/milestone29` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone29` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone30/array_slice_length.php`
  prints the committed integer-length `array_slice` output with positive,
  zero, and negative lengths, default integer-key reindexing, string-key
  preservation, original-array preservation, and string-valued dynamic calls to
  `array_slice`.
- `cargo run -p phpc -- test tests/fixtures/milestone30` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone30` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone31/array_slice_null_length.php`
  prints the committed null-length `array_slice` output with to-end slicing,
  default integer-key reindexing, string-key preservation, original-array
  preservation, and string-valued dynamic calls to `array_slice`.
- `cargo run -p phpc -- test tests/fixtures/milestone31` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone31` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone32/array_slice_preserve_keys.php`
  prints the committed preserve-key `array_slice` output with preserved
  integer/string keys, default `false` behavior, null-length preserve-key
  behavior, original-array preservation, and string-valued dynamic calls to
  `array_slice`.
- `cargo run -p phpc -- test tests/fixtures/milestone32` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone32` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone33/array_chunk.php`
  prints the committed `array_chunk` output with positive length chunking,
  default reindexing for original integer and string keys, empty-input
  behavior, original-array preservation, and string-valued dynamic calls to
  `array_chunk`.
- `cargo run -p phpc -- test tests/fixtures/milestone33` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone33` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone34/array_chunk_preserve_keys.php`
  prints the committed `array_chunk` output with boolean preserve-key chunking,
  default/false reindexing, original-array preservation, and string-valued
  dynamic calls to `array_chunk`.
- `cargo run -p phpc -- test tests/fixtures/milestone34` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone34` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone35/array_is_list.php`
  prints the committed `array_is_list` output with empty-list true behavior,
  zero-based ordered integer-key detection, normalized numeric string keys,
  false results for gaps/string keys/negative keys/out-of-order integer keys,
  reindexing contrast, and string-valued dynamic calls to `array_is_list`.
- `cargo run -p phpc -- test tests/fixtures/milestone35` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone35` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone36/array_pad.php` prints
  the committed `array_pad` output with positive right-padding, negative
  left-padding, no-op key-shape preservation, empty-array padding, and
  string-valued dynamic calls to `array_pad`.
- `cargo run -p phpc -- test tests/fixtures/milestone36` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone36` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone37/array_combine.php`
  prints the committed `array_combine` output with integer/string key-value
  conversion, duplicate-key overwrites, empty-array behavior, original-array
  preservation, and string-valued dynamic calls to `array_combine`.
- `cargo run -p phpc -- test tests/fixtures/milestone37` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone37` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone38/array_intersect_key.php`
  prints the committed `array_intersect_key` output with first-array key/value
  preservation, normalized integer/string key matching, empty and no-match
  results, original-array preservation, and string-valued dynamic calls to
  `array_intersect_key`.
- `cargo run -p phpc -- test tests/fixtures/milestone38` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone38` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone40/array_intersect_key_variadic.php`
  prints the committed variadic `array_intersect_key` output with
  intersection across all subsequent arrays, original-array preservation, and
  string-valued dynamic calls to `array_intersect_key`.
- `cargo run -p phpc -- test tests/fixtures/milestone40` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone40` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone39/array_diff_key.php`
  prints the committed `array_diff_key` output with first-array key/value
  preservation for keys absent from the second array, normalized integer/string
  key matching, empty, all-kept, and no-match results, original-array
  preservation, and string-valued dynamic calls to `array_diff_key`.
- `cargo run -p phpc -- test tests/fixtures/milestone39` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone39` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone41/array_diff_key_variadic.php`
  prints the committed variadic `array_diff_key` output with differences
  against all subsequent arrays, original-array preservation, append-index
  behavior, and string-valued dynamic calls to `array_diff_key`.
- `cargo run -p phpc -- test tests/fixtures/milestone41` passes with 1 fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone41` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone42/array_diff.php` prints
  the committed `array_diff` output with scalar string-form value comparisons,
  first-array key/value preservation, original-array preservation,
  append-index behavior, and string-valued dynamic calls to `array_diff`.
- `cargo run -p phpc -- test tests/fixtures/milestone42` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone42` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone43/array_intersect.php`
  prints the committed `array_intersect` output with scalar string-form value
  comparisons, first-array key/value preservation, original-array
  preservation, append-index behavior, and string-valued dynamic calls to
  `array_intersect`.
- `cargo run -p phpc -- test tests/fixtures/milestone43` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone43` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone44/array_intersect_variadic.php`
  prints the committed variadic `array_intersect` output with scalar
  string-form value intersection across all subsequent arrays,
  first-array key/value preservation, original-array preservation,
  append-index behavior, and string-valued dynamic calls to `array_intersect`.
- `cargo run -p phpc -- test tests/fixtures/milestone44` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone44` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone45/array_diff_variadic.php`
  prints the committed variadic `array_diff` output with scalar string-form
  value differences against all subsequent arrays, first-array key/value
  preservation, original-array preservation, append-index behavior, and
  string-valued dynamic calls to `array_diff`.
- `cargo run -p phpc -- test tests/fixtures/milestone45` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone45` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone46/array_unique.php`
  prints the committed `array_unique` output with scalar string-form
  deduplication, first-occurrence key/value preservation, original-array
  preservation, append behavior derived from kept integer keys, and
  string-valued dynamic calls to `array_unique`.
- `cargo run -p phpc -- test tests/fixtures/milestone46` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone46` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone47/array_replace.php`
  prints the committed `array_replace` output with key-preserving replacement
  overwrites, new replacement-key insertion order, original-array
  preservation, append-index behavior, and string-valued dynamic calls to
  `array_replace`.
- `cargo run -p phpc -- test tests/fixtures/milestone47` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone47` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone48/array_replace_variadic.php`
  prints the committed variadic `array_replace` output with left-to-right
  replacement overwrites, source-array preservation, append-index behavior,
  one-array clone behavior, and string-valued dynamic calls to `array_replace`.
- `cargo run -p phpc -- test tests/fixtures/milestone48` passes with 1
  fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone48` passes
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
- `cargo run -p phpc -- run tests/fixtures/unsupported_object_features/unsupported_clone_expression.php`
  exits 1 and reports `parse error at tests/fixtures/unsupported_object_features/unsupported_clone_expression.php:6:9: unsupported clone expression: object handle copying and __clone dispatch are not implemented`.
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
- `cargo run -p phpc -- compile tests/fixtures/milestone29/array_slice.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone30/array_slice_length.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone31/array_slice_null_length.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone32/array_slice_preserve_keys.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone33/array_chunk.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone34/array_chunk_preserve_keys.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone35/array_is_list.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone36/array_pad.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone37/array_combine.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone38/array_intersect_key.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone40/array_intersect_key_variadic.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone39/array_diff_key.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone41/array_diff_key_variadic.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone42/array_diff.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone43/array_intersect.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone44/array_intersect_variadic.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone45/array_diff_variadic.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone46/array_unique.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone47/array_replace.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone48/array_replace_variadic.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone49/array_sum.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo run -p phpc -- compile tests/fixtures/milestone50/array_product.php --emit-ir`
  exits 1 with the current explicit array native-lowering rejection before
  emitting misleading native code.
- `cargo test -p phpc --test array_reduce` includes explicit LLVM IR rejection
  coverage for `array_reduce` until native function-call lowering exists.
- `cargo run -p phpc -- test tests/fixtures/milestone52` passes with 1
  `array_reduce` initial-value fixture.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone52` passes
  with 1 system PHP comparison.
- `cargo run -p phpc -- run tests/fixtures/milestone52/array_reduce_initial.php`
  prints the committed scalar, array, empty-array, and dynamic-call initial
  accumulator output.
- `cargo run -p phpc -- test tests/fixtures/runtime_errors` passes with 112
  runtime-error fixtures.
- `tools/run-tests.sh` passes with 256 fixtures, 104 system PHP comparisons,
  and 152 `.phpc-only` skips.
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
  warning/TypeError details are not modeled. `array_key_first` and
  `array_key_last` are limited to array arguments and do not yet model
  reference/copy-on-write container effects, exact native `TypeError` objects,
  or native lowering.
  `array_is_list`, `array_values`, `array_keys`, `array_reverse`,
  `array_slice`, `array_chunk`, and `array_pad` are limited to array arguments,
  clone values under the current by-value model, require boolean preserve-key
  flags for `array_reverse`, `array_slice`, and `array_chunk` when those
  arguments are supplied, and do not yet model PHP references, copy-on-write
  containers, object handle identity preservation, resource values, non-bool
  preserve-key coercion, exact native `TypeError` objects, or native lowering.
  `array_is_list` detects only the current ordered integer/string key model and
  does not model reference/copy-on-write container effects.
  `array_slice` currently requires integer offsets and integer or null length
  arguments when supplied.
  `array_chunk` currently requires positive integer lengths and boolean
  preserve-key flags when supplied. Non-int lengths, non-positive lengths,
  non-bool preserve-key coercion, references, copy-on-write containers, object
  handle identity preservation, resource values, exact native
  `ValueError`/`TypeError` objects, and native lowering are not implemented.
  `array_pad` currently requires an integer length, uses a stable diagnostic
  for requests that would insert more than 1,048,576 padding entries, and does
  not implement non-int length coercion, exact native `ValueError`/`TypeError`
  objects, references, copy-on-write containers, object handle identity
  preservation, resource values, or native lowering.
  `array_keys` search-value filtering is implemented for the current scalar
  loose-comparison and strict-identity subsets, but array, object, resource, or
  reference search values, array, object, resource, or reference array values,
  and non-bool strict-flag coercion are not implemented.
  `array_merge` now accepts zero or more array arguments, but non-array
  argument recovery beyond the current stable diagnostics, references,
  copy-on-write containers, object handle identity preservation, resource
  values, exact native `TypeError` objects, and native lowering are not
  implemented.
  `array_combine` is limited to equal-length array operands whose key values
  are integers or strings. Unsupported `null`, bool, float, array, object,
  future resource, and reference key values fail with a stable project
  diagnostic instead of PHP's broader key coercions or native exception
  behavior; references, copy-on-write containers, object handle identity
  preservation for object values, resource values, exact native
  `ValueError`/`TypeError` objects, and native lowering are not implemented.
  `array_intersect_key` accepts two or more array operands over the current
  integer/string key model, but references, copy-on-write containers, object
  handle identity preservation for object values, resource values, exact native
  `TypeError` objects, and native lowering are not implemented.
  `array_diff_key` accepts two or more array operands over the current
  integer/string key model, but references, copy-on-write containers, object
  handle identity preservation for object values, resource values, exact native
  `TypeError` objects, and native lowering are not implemented.
  `array_diff` accepts two or more array operands over the current scalar value
  subset and compares values by current PHP string forms, but non-scalar
  comparisons, references, copy-on-write containers, object/resource values,
  exact native `TypeError` objects, PHP warning-and-string-conversion behavior
  for arrays/objects, and native lowering are not implemented.
  `array_intersect` accepts two or more array operands over the current scalar
  value subset and compares values by current PHP string forms, but non-scalar
  comparisons, references, copy-on-write containers, object/resource values,
  exact native `TypeError` objects, PHP warning-and-string-conversion behavior
  for arrays/objects, and native lowering are not implemented.
  `array_unique` accepts one array operand over the current scalar value subset
  and compares values by current PHP string forms, but sort flags, non-scalar
  comparisons, references, copy-on-write containers, object/resource values,
  exact native `TypeError` objects, PHP warning-and-string-conversion behavior
  for arrays/objects, and native lowering are not implemented.
  `array_replace` accepts one or more array operands over the current
  integer/string key model, but references, copy-on-write containers, object
  handle identity preservation for object values, resource values, exact native
  `TypeError` objects, and native lowering are not implemented.
  `array_flip` is limited to arrays whose source values are integers or
  strings. Unsupported `null`, bool, float, array, object, future resource, and
  reference values fail with a stable project diagnostic instead of PHP's
  warning-and-skip behavior; references, copy-on-write containers, exact native
  warning/`TypeError` objects, and native lowering are not implemented.
  `array_fill_keys` is limited to arrays whose key values are integers or
  strings. Unsupported `null`, bool, float, array, object, future resource, and
  reference key values fail with a stable project diagnostic instead of PHP's
  warning-and-skip behavior; references, copy-on-write containers, object handle
  identity for object fill values, exact native warning/`TypeError` objects,
  and native lowering are not implemented.
  `array_count_values` is limited to arrays whose source values are integers or
  strings. Unsupported `null`, bool, float, array, object, future resource, and
  reference values fail with a stable project diagnostic instead of PHP's
  warning-and-skip behavior; references, copy-on-write containers, exact native
  warning/`TypeError` objects, resource values, and native lowering are not
  implemented.
  `array_sum` is limited to `null`, booleans, integers, floats, and
  well-formed numeric strings. Non-numeric strings and non-scalar values fail
  with stable project diagnostics instead of PHP's warning/recovery behavior;
  references, copy-on-write containers, object/resource values, exact native
  `TypeError` objects, PHP warning recovery, and native lowering are not
  implemented.
  `array_product` has the same numeric scalar input boundary as `array_sum`;
  non-numeric strings and non-scalar values fail with stable project
  diagnostics instead of PHP's warning/recovery behavior. References,
  copy-on-write containers, object/resource values, exact native `TypeError`
  objects, PHP warning recovery, and native lowering are not implemented.
  `array_reduce` callback support is limited to string-valued user-function or
  callable-builtin names. Array/object callables, closures, first-class
  callables, method calls, reference and copy-on-write behavior, object handle
  identity preservation, resource values, exact native `TypeError` objects,
  and native lowering are not implemented.
  `array_filter` callback support is limited to omitted callbacks, explicit
  `null` callbacks, string-valued user-function or callable-builtin names in
  value-only mode with explicit integer mode flag `0`, and string-valued
  key-only callbacks with explicit integer mode flag `2`, and string-valued
  value/key callbacks with explicit integer mode flag `1`. Array/object
  callables, closures, first-class callables, method calls, named
  `ARRAY_FILTER_*` constants, integer mode flags outside `0`, `1`, and `2`,
  non-int mode coercions such as `false`, reference and copy-on-write
  behavior, object handle identity preservation, resource values, exact native
  `TypeError`
  objects, and native lowering are not implemented.
  `array_map` callback support is limited to one-array null-callback identity
  mapping, variadic null-callback zip mapping, and variadic input arrays with
  string-valued user-function or callable-builtin names. The one-array forms
  preserve original integer/string keys, while multi-array forms reindex mapped
  results from integer key zero. Array/object callables, closures,
  first-class callables, method calls,
  references, copy-on-write behavior, object handle identity preservation,
  resource values, exact native `TypeError` objects, and native lowering are
  not implemented.
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
  limited to trailing defaults over the documented constant-expression and
  unqualified constant-reference subset. Constant references are resolved from
  constants defined before the omitted argument is bound, plus the current
  built-in `ARRAY_FILTER_*` constants. Forward references at call time,
  namespace-aware constants, class constants, dynamic defaults,
  references/copy-on-write behavior, and native lowering for defaults are not
  implemented. Non-constant defaults and required parameters after defaults are
  rejected by the parser. Variadic parameters, argument unpacking, references,
  parameter and return type declarations, closures and arrow functions, named
  arguments, and `declare(strict_types=1)` now fail with explicit parse
  diagnostics. Static local variable declarations inside functions also fail
  with an explicit parse diagnostic before static local storage exists.
  `__LINE__` evaluates from expression source spans, `__FILE__` evaluates
  from the current `phpc run` input path string when one is available, and
  `__DIR__` evaluates as that path's parent directory. Other magic constants
  fail with explicit parse diagnostics before source/context-aware magic
  constant evaluation exists.
  Nullable, union, and intersection types, `mixed`, `void`/`never`,
  class/interface type names, coercive versus strict typing, variance, runtime
  type enforcement, static local initialization expressions, per-function
  persistence, references, recursion/reentrancy behavior, canonical absolute
  `__FILE__`/`__DIR__` paths matching PHP exactly, eval/include source
  mapping, function/method/class magic constant context,
  namespace and trait magic constants, native static-local lowering, magic
  constant lowering, and native type lowering are not implemented.
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
- Global constant resolution is limited to exact uppercase
  `ARRAY_FILTER_USE_KEY` and `ARRAY_FILTER_USE_BOTH` as bare built-in
  constants, plus runtime-defined constants created with `define($name,
  $value)`, introspected with `defined($name)`, and read through
  `constant($name)` or bare unqualified names for the current string-name and
  scalar/array value subset. Top-level single and grouped `const NAME =
  value;` declarations work for unqualified names and the current
  constant-expression/scalar-array value subset, including references to
  previously defined unqualified constants and the current built-in
  `ARRAY_FILTER_*` constants. Forward references, other built-in constants,
  names lexed as language keywords or literals for bare reads, magic constants
  other than `__LINE__`, `__FILE__`, and `__DIR__`, case-insensitive legacy
  constants, extension constants, namespace-qualified constants, nested
  declarations, dynamic declaration values, class constants through
  `constant()`/`defined()`, references/copy-on-write behavior for constant
  values, and native lowering for constants are not implemented.
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

- Added `__DIR__` as the third executable magic constant, evaluated from the
  current `phpc run` input path's parent directory in ordinary expressions,
  default parameter values, and top-level `const` declarations. Path-less
  library execution currently evaluates it as an empty string, paths without a
  parent directory evaluate to `.`, canonical PHP absolute path behavior and
  eval/include source mapping remain unsupported, and native lowering still
  rejects it explicitly.
- Added `__FUNCTION__` as the fourth executable magic constant, evaluated as
  the current user-function name in ordinary expressions and default parameter
  values, and as an empty string outside a function. Method/class/trait/
  namespace context constants, closure context, eval/include source mapping,
  exact canonical PHP behavior, and native lowering remain unsupported.
- Tightened the `__METHOD__` boundary with a stable parse diagnostic tied to
  the current missing method-dispatch and method-context execution path, plus
  fixture and CLI snapshot coverage. `__METHOD__` remains unsupported until a
  real method execution slice exists.
- Tightened the `__CLASS__` boundary with a stable parse diagnostic tied to
  the current missing class-context tracking path, plus fixture and CLI
  snapshot coverage. `__CLASS__` remains unsupported until class-context magic
  constant evaluation exists.
- Tightened the `__TRAIT__` boundary with a stable parse diagnostic tied to
  the current missing trait declaration/use and trait-context tracking path,
  plus fixture and CLI snapshot coverage. `__TRAIT__` remains unsupported until
  trait-context magic constant evaluation exists.
- Tightened the `__NAMESPACE__` boundary with a stable parse diagnostic tied
  to the current missing namespace-aware name-resolution path, plus fixture and
  CLI snapshot coverage. `__NAMESPACE__` remains unsupported until namespace
  context magic constant evaluation exists.
- Added an explicit stable parse diagnostic, parser coverage, fixture coverage,
  and `phpc run` CLI snapshot for unsupported `trait` declarations before trait
  parsing or trait use execution exists. Trait methods, properties, constants,
  conflict resolution, aliases, visibility changes, namespace-aware traits, and
  native lowering remain unsupported.
- Added an explicit stable parse diagnostic, parser coverage, fixture coverage,
  and `phpc run` CLI snapshot for unsupported `interface` declarations before
  interface parsing or implementation execution exists. Interface constants,
  method signatures, inheritance, namespace-aware interfaces,
  class/interface type names, and native lowering remain unsupported.
- Added an explicit stable parse diagnostic, parser coverage, fixture coverage,
  and `phpc run` CLI snapshot for unsupported `enum` declarations before enum
  parsing or case/value execution exists. Unit enums, backed enums, cases,
  values, methods, interface implementations, namespace-aware enums, exact PHP
  parse/error objects, and native lowering remain unsupported.
- Added explicit stable parse diagnostics, parser coverage, fixture coverage,
  and `phpc run` CLI snapshots for unsupported `abstract`, `final`, and
  `readonly` class modifiers before modifier-aware class parsing exists.
  Abstract classes/methods, final inheritance restrictions, readonly
  classes/properties, namespace-aware classes, exact PHP parse/error objects,
  and native lowering remain unsupported.
- Added explicit stable parse diagnostics, parser coverage, fixture coverage,
  and `phpc run` CLI snapshots for unsupported `abstract`, `final`, and
  `readonly` class member modifiers before modifier-aware member parsing
  exists. Abstract methods, final methods, readonly properties, property
  initialization rules, inheritance interactions, exact PHP parse/error
  objects, and native lowering remain unsupported.
- Added an explicit stable parse diagnostic, parser coverage for named,
  nullable, and union type forms, fixture coverage, and a `phpc run` CLI
  snapshot for unsupported typed property declarations before typed property
  storage or enforcement exists. Nullable/union/intersection enforcement,
  default values, readonly interactions, inheritance, reflection, exact PHP
  parse/error objects, and native lowering remain unsupported.
- Added an explicit stable parse diagnostic, parser coverage, fixture coverage,
  and a `phpc run` CLI snapshot for unsupported property default values before
  property initializer execution exists. Constant-expression defaults,
  array/object defaults, readonly initialization rules, inheritance/reflection
  behavior, exact PHP parse/error objects, and native lowering remain
  unsupported.
- Added explicit parser coverage, fixture coverage, and a `phpc run` CLI
  snapshot for unsupported multiple properties in one class property
  declaration before multi-property metadata parsing exists. Per-property
  defaults, mixed visibility/static handling, typed multi-property
  declarations, reflection behavior, exact PHP parse/error objects, and native
  lowering remain unsupported.
- Added an explicit stable parse diagnostic, parser coverage, fixture
  coverage, and a `phpc run` CLI snapshot for unsupported class constant
  declarations before class constant metadata or lookup exists. Visibility,
  typed constants, inheritance/override behavior, interface constants,
  reflection behavior, exact PHP parse/error objects, and native lowering
  remain unsupported.
- Added an explicit stable parse diagnostic, parser coverage, fixture
  coverage, and a `phpc run` CLI snapshot for unsupported `$this` usage before
  method execution and object context binding exist. Method dispatch,
  constructor context, closure binding, static method behavior, inheritance,
  exact PHP `Error` objects, and native lowering remain unsupported.
- Added explicit committed fixture coverage and `phpc run` CLI snapshots for
  unsupported constructor execution and constructor arguments. Declared
  `__construct` methods and `new ClassName(...)` argument lists still fail with
  stable runtime diagnostics until user-constructor execution, `$this` binding,
  property initialization, visibility, inheritance, promoted properties, exact
  PHP `Error` objects, and native lowering exist.
- Added an explicit stable parse diagnostic, parser coverage, fixture
  coverage, and a `phpc run` CLI snapshot for unsupported `clone` expressions
  before object handle copying or `__clone` dispatch exists. Object identity,
  shallow/deep property copying, `__clone`, references, inheritance, exact PHP
  `Error` objects, and native lowering remain unsupported.
- Added an explicit stable parse diagnostic, parser coverage, fixture
  coverage, and a `phpc run` CLI snapshot for unsupported `instanceof`
  expressions before class/interface relationship checks exist. Inheritance,
  interface implementation checks, namespace-aware class names, autoloading,
  exact PHP `Error` objects, and native lowering remain unsupported.
- Added an explicit stable parse diagnostic, parser coverage, fixture
  coverage, and a `phpc run` CLI snapshot for unsupported `ClassName::class`
  expressions before class-name constant resolution exists. Namespaces,
  aliases/imports, magic class names such as `self`/`parent`/`static`,
  autoloading, exact PHP behavior, and native lowering remain unsupported.
- Added an explicit stable parse diagnostic, parser coverage, fixture
  coverage, and a `phpc run` CLI snapshot for unsupported magic static
  receivers such as `self::`, `parent::`, and `static::` before class-context,
  parent-class, or late-static-binding resolution exists. Static property
  storage, static method dispatch, class constants, inheritance, exact PHP
  `Error` objects, and native lowering remain unsupported.
- Added an explicit stable parse diagnostic, parser coverage, fixture
  coverage, and a `phpc run` CLI snapshot for unsupported `implements`
  clauses before interface metadata or class/interface relationship checks
  exist. Interface method enforcement, inheritance interactions, namespace-aware
  interface names, autoloading, exact PHP `Error` objects, and native lowering
  remain unsupported.
- Added an explicit stable parse diagnostic, parser coverage, fixture
  coverage, and a `phpc run` CLI snapshot for unsupported trait use inside
  classes before trait composition exists. Trait methods/properties/constants,
  conflict resolution, aliases, visibility adaptations, namespace-aware traits,
  exact PHP parse/error objects, and native lowering remain unsupported.
- Added `get_class($object)` support for the current minimal object value
  model, including string-valued dynamic-call availability, non-object
  diagnostics, fixture CLI coverage, documentation, and explicit native-codegen
  rejection. Inheritance, aliases/imports, anonymous classes, magic class
  names, object handle identity, exact native `TypeError` objects, and native
  lowering remain unsupported.
- Added `is_object($value)` support for the current minimal object value model,
  including false results for scalars and arrays, string-valued dynamic-call
  availability, fixture CLI coverage, documentation, and explicit
  native-codegen rejection through the current object/function-call boundaries.
  Inheritance-aware object checks, proxy/extension object behavior, exact
  native reflection/type-system interactions, and native lowering remain
  unsupported.
- Added `get_debug_type($value)` support for the current scalar/array/minimal
  object value model, including scalar and array type-name results, declared
  class-name results for current object values, string-valued dynamic-call
  availability, fixture CLI coverage, documentation, and explicit
  native-codegen rejection through the current object/function-call boundaries.
  Inheritance aliases/imports, anonymous classes, resources, exact native
  reflection/type-system interactions, and native lowering remain unsupported.
- Added `class_exists($name[, $autoload])` support for the current declared
  class metadata table, including case-insensitive string-name lookup,
  string-valued dynamic-call availability, non-string name and non-bool
  autoload diagnostics, fixture CLI coverage, documentation, and explicit
  native-codegen rejection through the current function-call boundary.
  Autoloading, namespace/import aliases, anonymous classes, exact native
  `TypeError` behavior, and native lowering remain unsupported.
- Added `property_exists($object_or_class, $property)` support over the current
  declared property metadata. The supported slice accepts current object values
  or string class names, uses case-sensitive property names, reports declared
  public/protected/private and static properties, returns false for missing
  properties or missing string class names, works through string-valued dynamic
  calls, and has fixture CLI coverage plus stable diagnostics for unsupported
  argument types. Dynamic properties created outside declarations, autoload
  side effects, namespace/import aliases, reflection behavior, exact native
  `TypeError` behavior, and native lowering remain unsupported.
- Added `method_exists($object_or_class, $method)` support over the current
  declared method metadata. The supported slice accepts current object values
  or string class names, uses case-insensitive method names, reports declared
  public/protected/private and static methods, returns false for missing
  methods or missing string class names, works through string-valued dynamic
  calls, and has fixture CLI coverage plus stable diagnostics for unsupported
  argument types. Method dispatch, inheritance, traits, interfaces,
  aliases/imports, autoloading, visibility behavior beyond metadata reporting,
  exact native `TypeError` behavior, and native lowering remain unsupported.
- Added `is_a($object_or_class, $class_name[, $allow_string])` support over
  the current minimal object/class metadata as an exact-class lookup slice. The
  supported path accepts current object values, accepts string first arguments
  only when `allow_string` is true, uses case-insensitive class metadata lookup
  for class-name strings, returns false for missing source or target classes,
  works through string-valued dynamic calls, and has fixture CLI coverage plus
  stable diagnostics for unsupported class-name and allow-string argument
  types. Inheritance, interfaces, traits, aliases/imports, namespace-aware
  names, autoloading, exact native `TypeError` behavior, and native lowering
  remain unsupported.
- Added `is_subclass_of($object_or_class, $class_name[, $allow_string])` as a
  no-inheritance class relationship boundary. The supported path accepts
  current object values and string first arguments, considers string first
  arguments only when `allow_string` is true, validates string class names and
  boolean allow-string flags, returns false for exact-class, missing-class, and
  no-parent metadata cases, works through string-valued dynamic calls, and has
  fixture CLI coverage. Inheritance, interfaces, traits, aliases/imports,
  namespace-aware names, autoloading, exact native `TypeError` behavior, and
  native lowering remain unsupported.
- Added `get_parent_class($object_or_class)` as a no-inheritance parent-class
  boundary. The supported path accepts current object values and declared
  string class names, returns false for all supported inputs because parent
  class metadata is not represented yet, works through string-valued dynamic
  calls, and has fixture CLI coverage plus stable diagnostics for unsupported
  argument types and missing string classes. Inheritance, interfaces,
  aliases/imports, namespace-aware names, autoloading, default `$this`
  behavior, exact native `TypeError` behavior, and native lowering remain
  unsupported.
- Added `get_declared_classes()` over the current declared-class metadata. The
  supported slice returns a zero-indexed array of classes declared in the
  current parsed program in declaration order, works through string-valued
  dynamic calls, has fixture CLI coverage, and rejects native lowering through
  the current function-call boundary. Built-in/internal/extension classes,
  anonymous classes, autoloading, namespaces/import aliases, exact native
  ordering, and native lowering remain unsupported.
- Added `get_class_methods($object_or_class)` over the current declared method
  metadata. The supported slice accepts current object values and declared
  string class names, returns a zero-indexed array of public method names in
  declaration order including public static methods, works through
  string-valued dynamic calls, has fixture CLI coverage plus stable diagnostics
  for unsupported target values and missing string classes, and rejects native
  lowering through the current function-call boundary. Inheritance, traits,
  interfaces, aliases/imports, namespace-aware names, autoloading,
  non-public/context-sensitive visibility listing, exact native ordering and
  `TypeError` behavior, and native lowering remain unsupported.

Next:

- Continue with the next small object/class metadata introspection boundary,
  starting with `get_class_vars($class_name)` if it can be completed honestly
  over the current property metadata.
