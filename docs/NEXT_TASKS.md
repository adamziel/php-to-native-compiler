# Next Tasks

Use this as the queue for repeated Codex work. Take one unchecked task at a
time unless a task explicitly depends on another. Mark a task checked only after
code, tests, CLI coverage, documentation, and unsupported edge cases are all
handled.

## Milestone 2: Value Model and Runtime

- [x] Add structured runtime errors with stable messages and tests for at
  least undefined variables, arity mismatches, unsupported calls, and invalid
  arithmetic.
- [x] Complete scalar arithmetic coercion coverage for `Null`, `Bool`, numeric
  strings, non-numeric strings, `Int`, and `Float`; add fixture tests and
  document the remaining gaps from PHP's full coercion rules.
- [x] Add a scalar comparison behavior matrix for equality and relational
  operators across implemented value types, with runtime tests and fixture CLI
  coverage.
- [x] Complete optional system PHP comparison mode for the fixture runner,
  including docs, progress notes, and gating so the suite still passes when
  `php` is not installed.
- [x] Add explicit CLI exercises for representative runtime errors and record
  their stdout, stderr, and exit behavior.

## Milestone 3: Arrays

- [x] Implement an ordered PHP array value in `php_runtime` with int/string key
  normalization tests.
- [x] Parse and interpret short array literals `[]` and `[key => value]` for the
  supported scalar expression subset.
- [x] Implement array append, indexed reads, and indexed writes in the
  interpreter with fixture CLI coverage.
- [x] Document unsupported array behavior, including references, nested
  copy-on-write containers, destructuring, spread, and complex key coercions.
- [x] Add a small native-codegen rejection test proving arrays fail with an
  explicit codegen error until lowering exists.

## Milestone 4: Functions and Scopes

- [x] Separate local and global scope behavior for user functions, with tests
  for shadowing and unsupported `global`.
- [x] Add recursion coverage and a documented runtime guard for runaway calls.
- [x] Implement default parameters for user functions with parser, runtime, and
  fixture coverage.
- [x] Add the first small builtin function set with documented signatures,
  errors, tests, and CLI examples.
- [x] Document unsupported function features: variadics, references, closures,
  named arguments, strict types, and dynamic-call gaps outside the current
  string-name lookup subset.

## Milestone 5+: Dynamic PHP

- [x] Introduce a materialized symbol table path for future variable variables
  without changing current static variable behavior.
- [x] Design include/require resolution rules and add explicit unsupported
  diagnostics before implementing execution.
- [x] Add runtime lookup infrastructure for dynamic function calls and keep
  unresolved calls as explicit runtime errors.
- [x] Define the `eval` fallback boundary: parser entry point, caller scope
  behavior, diagnostics, and unsupported cases.
- [x] Sketch the minimal object/class metadata model before adding syntax.
- [x] Parse class declarations into a metadata registry while keeping object
  instantiation and member access unsupported.
- [x] Add a minimal object value/instantiation boundary for `new ClassName()`
  while keeping property access and method dispatch unsupported.
- [x] Add public instance property reads and writes for the current object value
  model while keeping method dispatch, constructors, and visibility enforcement
  unsupported.
- [x] Add `isset($object->publicProperty)` support for public instance
  properties while keeping array offsets, dynamic property names, non-public
  visibility enforcement, and method dispatch unsupported.
- [x] Add explicit parse diagnostics for unsupported static property, static
  method, and class constant syntax such as `ClassName::$prop`,
  `ClassName::method()`, and `ClassName::CONST` before implementing static
  member storage or dispatch.
- [x] Add explicit parse diagnostics for unsupported namespace and `use`
  declaration syntax before namespace-aware name resolution or imports exist.
- [x] Add explicit parse diagnostics for unsupported namespace-qualified
  function and class names such as `App\fn()` and `new App\Box()` before
  namespace-aware name resolution exists.

## Syntax Boundaries

- [x] Add explicit parse diagnostics for unsupported long `array(...)` syntax
  before implementing long array literals.
- [x] Add explicit parse diagnostics for unsupported `unset(...)` syntax before
  implementing unset.
- [x] Add explicit parse diagnostics for unsupported `foreach` syntax before
  implementing iteration.
- [x] Add explicit parse diagnostics for unsupported `for` syntax before
  implementing C-style loops.
- [x] Add explicit parse diagnostics for unsupported `do ... while` syntax
  before implementing do-while loops.
- [x] Add explicit parse diagnostics for unsupported `switch` syntax before
  implementing switch/case control flow.
- [x] Add explicit parse diagnostics for unsupported `break`/`continue` syntax
  before implementing loop-control execution.

## Milestone 6: Loop-Control Execution

- [x] Implement `break;` execution for innermost `while` loops with parser,
  interpreter, fixture, CLI snapshot, documentation, and explicit native-codegen
  rejection coverage where lowering remains unsupported.
- [x] Implement `continue;` execution for innermost `while` loops with parser,
  interpreter, fixture, CLI snapshot, documentation, and explicit native-codegen
  rejection coverage where lowering remains unsupported.

## Milestone 7: Builtin and Array Refinements

- [x] Implement direct `isset($array[$key])` support for array offset operands,
  with tests for existing keys, null values, missing keys, undefined arrays, and
  non-array targets while keeping complex lvalues explicitly unsupported.
- [x] Implement `array_key_exists($key, $array)` for the current ordered array
  value model, including null-value contrast against `isset`, invalid key and
  non-array diagnostics, fixture CLI coverage, and docs for unsupported key
  coercions.
- [x] Implement `empty(...)` for direct variables and direct array offsets over
  the current scalar/array value model, including undefined, missing, `null`,
  false, zero, empty string, and string `"0"` behavior, unsupported complex
  lvalue diagnostics, fixture CLI coverage, and documented gaps.
- [x] Implement `array_values($array)` for the current ordered array value
  model, including integer reindexing behavior, non-array diagnostics, fixture
  CLI coverage, and documented gaps.
- [x] Implement `array_keys($array)` for the current ordered array value model,
  including integer/string key value emission, non-array diagnostics, fixture
  CLI coverage, and documented gaps.
- [x] Implement `in_array($needle, $array)` over the current ordered array
  value model, including loose scalar comparison behavior, non-array
  diagnostics, fixture CLI coverage, and documented gaps around strict mode,
  objects, arrays, and references.
- [x] Implement `array_search($needle, $array)` over the current ordered array
  value model, including loose scalar comparison behavior, key return behavior,
  non-array diagnostics, fixture CLI coverage, and documented gaps around
  strict mode, objects, arrays, and references.

## Milestone 8: Array Iteration

- [x] Implement `foreach ($array as $value)` over the current ordered array
  value model, including parser/interpreter support, non-array diagnostics,
  fixture CLI coverage, documentation, unsupported gaps for by-reference forms,
  and explicit native-codegen rejection coverage while lowering remains
  unsupported.
- [x] Implement `foreach ($array as $key => $value)` key/value iteration over
  the current ordered array value model, including integer/string key emission,
  non-array diagnostics reuse, fixture CLI coverage, documentation,
  unsupported gaps for by-reference/object/destructuring forms, and explicit
  native-codegen rejection coverage while lowering remains unsupported.

## Milestone 9: Array Mutation

- [x] Implement direct `unset($array[$key])` for direct array variables over the
  current integer/string key subset, including missing-key behavior, fixture CLI
  coverage, documentation, and explicit native-codegen rejection while broader
  `unset` forms remain unsupported.
- [x] Implement direct `unset($name)` for static variables backed by the current
  symbol table, including undefined-variable no-op behavior, fixture CLI
  coverage, documentation, and explicit native-codegen rejection while property,
  append-offset, and nested unset forms remain unsupported.
- [x] Implement multiple-operand `unset(...)` over the currently supported
  direct variable and direct array-offset operands, including left-to-right
  behavior, fixture CLI coverage, documentation, and explicit native-codegen
  rejection while property, append-offset, and nested unset forms remain
  unsupported.

## Milestone 10: Syntax Expansion

- [x] Implement long `array(...)` literals as an alias for the current
  short-array literal subset, including keyed entries, fixture CLI coverage,
  documentation, and explicit unsupported gaps for references, spread, and
  unsupported key coercions.
- [x] Implement C-style `for (...)` loops over the current scalar expression
  and assignment subset, including initializer, condition, increment,
  `break;`/`continue;` behavior, fixture CLI coverage, documentation, and
  explicit native-codegen rejection while lowering remains unsupported.
- [x] Implement `do ... while` loops over the current scalar expression and
  assignment subset, including at-least-once execution, condition evaluation
  after the body, `break;`/`continue;` behavior, fixture CLI coverage,
  documentation, and explicit native-codegen rejection while lowering remains
  unsupported.
- [x] Implement `switch (...)` over the current scalar comparison subset,
  including `case`, `default`, fallthrough, `break;` behavior, fixture CLI
  coverage, documentation, and explicit native-codegen rejection while lowering
  remains unsupported.

## Milestone 11: Conditional Refinements

- [x] Implement `elseif` chains over the current `if` expression subset,
  including parser/interpreter coverage, fixture CLI coverage, documentation,
  and explicit native-codegen rejection while lowering remains unsupported.
- [x] Add explicit parse diagnostics for alternate `if`/`elseif`/`else`
  colon/`endif` syntax before implementing alternate conditional syntax,
  including parser coverage, fixture CLI coverage, documentation, and named
  unsupported gaps.

## Milestone 12: Comparison Refinements

- [x] Add explicit parse diagnostics for strict identity operators `===` and
  `!==` before implementing strict comparisons, including parser coverage,
  fixture CLI coverage, documentation, and named unsupported gaps.
- [x] Implement strict identity operators `===` and `!==` for the current
  scalar value subset only, including parser/runtime/fixture coverage,
  documentation, and explicit gaps for arrays, objects, resources, references,
  and native lowering.

## Milestone 13: Strict Array Search

- [x] Implement `in_array($needle, $array, true)` for the current scalar
  needle/value subset using strict identity semantics, including fixture CLI
  coverage, documentation, and explicit gaps for arrays, objects, resources,
  references, and native lowering.
- [x] Implement `array_search($needle, $array, true)` for the current scalar
  needle/value subset using strict identity semantics and key-return behavior,
  including fixture CLI coverage, documentation, and explicit gaps for arrays,
  objects, resources, references, and native lowering.

## Milestone 14: Array Ordering Builtins

- [x] Implement `array_reverse($array)` for the current ordered array value
  model with default reindexing behavior, non-array diagnostics, fixture CLI
  coverage, documentation, and explicit gaps for references, copy-on-write
  containers, objects/resources, and native lowering.
- [x] Implement `array_reverse($array, true)` preserve-key behavior for the
  current ordered integer/string key model, including fixture CLI coverage,
  documentation, and explicit gaps for references, copy-on-write containers,
  objects/resources, and native lowering.

## Milestone 15: Array Combination Builtins

- [x] Implement `array_merge($left, $right)` for two arrays over the current
  ordered integer/string key model, including string-key overwrite behavior,
  integer-key reindexing, non-array diagnostics, fixture CLI coverage,
  documentation, and explicit gaps for references, copy-on-write containers,
  objects/resources, and native lowering.
- [x] Extend `array_merge` beyond the current two-array slice with
  zero-argument empty-array behavior and variadic array operands, including
  arity/type diagnostics, fixture CLI coverage, documentation, and explicit
  gaps for references, copy-on-write containers, objects/resources, and native
  lowering.

## Milestone 16: Array Key Filtering Builtins

- [x] Implement `array_keys($array, $search_value)` for the current scalar
  value subset using loose comparison semantics, including fixture CLI
  coverage, documentation, non-array/search-value diagnostics, and explicit
  gaps for arrays, objects, resources, references, and native lowering.
- [x] Implement `array_keys($array, $search_value, true)` for the current
  scalar value subset using strict identity semantics, including fixture CLI
  coverage, documentation, non-bool strict flag diagnostics, and explicit gaps
  for arrays, objects, resources, references, and native lowering.

## Milestone 17: Array Key Introspection Builtins

- [x] Implement `array_key_first($array)` for the current ordered array value
  model, including first-key return behavior for integer/string keys,
  empty-array `null` behavior, non-array diagnostics, fixture CLI coverage,
  documentation, and explicit native-codegen rejection.
- [x] Implement `array_key_last($array)` for the current ordered array value
  model, including last-key return behavior for integer/string keys,
  empty-array `null` behavior, non-array diagnostics, fixture CLI coverage,
  documentation, and explicit native-codegen rejection.

## Milestone 18: Array Transform Builtins

- [x] Implement `array_flip($array)` for the current ordered array value model,
  including integer/string value-to-key conversion, duplicate-key overwrite
  behavior, non-array and unsupported-value diagnostics, fixture CLI coverage,
  documentation, and explicit native-codegen rejection.
- [x] Implement `array_fill_keys($keys, $value)` for the current ordered array
  value model, including integer/string key-value conversion, duplicate-key
  overwrite behavior, non-array and unsupported-key diagnostics, fixture CLI
  coverage, documentation, and explicit native-codegen rejection.

## Milestone 19: Array Counting Builtins

- [x] Implement `array_count_values($array)` for the current ordered array
  value model, including integer/string value counting, non-array and
  unsupported-value diagnostics, fixture CLI coverage, documentation, and
  explicit native-codegen rejection.

## Milestone 20: Array Filtering Builtins

- [x] Implement `array_filter($array)` without a callback over the current
  ordered array value model, including falsey-value removal, key preservation,
  non-array diagnostics, fixture CLI coverage, documentation, callback
  unsupported gaps, and explicit native-codegen rejection.

## Milestone 21: Array Callback Builtins

- [x] Implement `array_filter($array, $callback)` for the first supported
  callback subset over the current ordered array value model, including
  string-valued callable names, value-only callback mode, key preservation,
  callback return truthiness, unresolved/non-callable diagnostics, fixture CLI
  coverage, documentation, explicit gaps for `ARRAY_FILTER_USE_KEY` and
  `ARRAY_FILTER_USE_BOTH`, and explicit native-codegen rejection.

## Milestone 22: Array Mapping Builtins

- [x] Implement `array_map($callback, $array)` for the first supported
  one-array callback subset over the current ordered array value model,
  including string-valued callable names, value callback arguments, integer
  reindexing behavior, unresolved/non-callable diagnostics, fixture CLI
  coverage, documentation, explicit gaps for multiple arrays, null callbacks,
  key preservation differences, references/copy-on-write, and explicit
  native-codegen rejection.

## Milestone 23: Array Mapping Follow-ups

- [x] Implement `array_map($callback, $left, $right)` for the first
  two-array string-callback subset, including lockstep value arguments,
  shortest/longest-array behavior documented against PHP, integer reindexing,
  diagnostics for unsupported extra arrays and callbacks, fixture CLI coverage,
  documentation, and explicit native-codegen rejection.

## Milestone 24: Array Mapping Key Preservation

- [x] Align one-array `array_map($callback, $array)` key preservation with PHP
  for the current string-callback subset, including fixture updates, CLI
  coverage, documentation, and explicit gaps for null callbacks,
  references/copy-on-write, and native lowering.

## Milestone 25: Array Map Null Callback

- [x] Implement `array_map(null, $array)` identity mapping for one input array
  over the current ordered array value model, including key behavior, fixture
  CLI coverage, documentation, and explicit gaps for multi-array zip modes,
  references/copy-on-write, and native lowering.

## Milestone 26: Array Map Null Callback Zip

- [x] Implement `array_map(null, $left, $right)` for the first multi-array
  null-callback zip slice over the current ordered array value model, including
  longest-array `null` padding, integer reindexing, fixture CLI coverage,
  documentation, and explicit gaps for broader zip arities,
  references/copy-on-write, object handle identity preservation, and native
  lowering.

## Milestone 27: Array Map Null Callback Variadic Follow-ups

- [x] Extend `array_map(null, ...)` beyond two input arrays over the current
  ordered array value model, including longest-array `null` padding,
  integer reindexing, fixture CLI coverage, documentation, and explicit gaps
  for variadic string-callback mapping, references/copy-on-write, object handle
  identity preservation, and native lowering.

## Milestone 28: Array Map Callback Variadic Follow-ups

- [x] Extend `array_map($callback, ...)` beyond two input arrays for the
  current string-valued user-function/callable-builtin callback subset,
  including longest-array `null` padding, integer reindexing, fixture CLI
  coverage, documentation, and explicit gaps for references/copy-on-write,
  object handle identity preservation, array/object callables, closures,
  method calls, and native lowering.

## Milestone 29: Array Slicing Builtins

- [x] Implement `array_slice($array, $offset)` for the current ordered array
  value model, including default integer-key reindexing, string-key
  preservation, non-array/non-int diagnostics, fixture CLI coverage,
  documentation, and explicit gaps for length, preserve-keys mode,
  references/copy-on-write, object handle identity preservation, resources,
  exact native `TypeError` objects, and native lowering.

## Milestone 30: Array Slicing Follow-ups

- [x] Extend `array_slice` with the integer length argument over the current
  ordered array value model, including positive, negative, and zero length
  behavior, non-int length diagnostics, fixture CLI coverage, documentation,
  and explicit gaps for preserve-keys mode, references/copy-on-write, object
  handle identity preservation, resources, exact native `TypeError` objects,
  and native lowering.

## Milestone 31: Array Slicing Null Length

- [x] Extend `array_slice` with `null` length as a to-end slice over the
  current ordered array value model, including fixture CLI coverage,
  documentation, and explicit gaps for preserve-keys mode,
  references/copy-on-write, object handle identity preservation, resources,
  exact native `TypeError` objects, and native lowering.

## Milestone 32: Array Slicing Preserve Keys

- [x] Extend `array_slice` with boolean preserve-key mode over the current
  ordered integer/string key model, including default/false behavior,
  true-preserve behavior for integer and string keys, null-length interaction,
  non-bool diagnostics, fixture CLI coverage, documentation, and explicit gaps
  for references/copy-on-write, object handle identity preservation, resources,
  exact native `TypeError` objects, and native lowering.

## Milestone 33: Array Chunking Builtins

- [x] Implement `array_chunk($array, $length)` over the current ordered array
  value model, including positive length behavior, integer-key reindexing,
  string-key behavior, non-array/non-int/non-positive diagnostics, fixture CLI
  coverage, documentation, and explicit gaps for preserve-key mode,
  references/copy-on-write, object handle identity preservation, resources,
  exact native `ValueError`/`TypeError` objects, and native lowering.

## Milestone 34: Array Chunking Preserve Keys

- [x] Extend `array_chunk` with boolean preserve-key mode over the current
  ordered integer/string key model, including true-preserve behavior for
  integer and string keys, default/false reindexing behavior,
  non-bool diagnostics, fixture CLI coverage, documentation, and explicit gaps
  for references/copy-on-write, object handle identity preservation, resources,
  exact native `ValueError`/`TypeError` objects, and native lowering.

## Milestone 35: Array List Introspection

- [x] Implement `array_is_list($array)` over the current ordered
  integer/string key model, including empty-array true behavior, exact
  zero-based consecutive integer-key detection, string-key false behavior,
  non-array diagnostics, fixture CLI coverage, documentation, and explicit
  gaps for references/copy-on-write, exact native `TypeError` objects, and
  native lowering.

## Milestone 36: Array Padding Builtin

- [x] Implement `array_pad($array, $length, $value)` over the current ordered
  integer/string key model, including positive right-padding, negative
  left-padding, no-op behavior when the requested size is not larger than the
  input, non-array/non-int diagnostics, fixture CLI coverage, documentation,
  and explicit gaps for references/copy-on-write, exact native `ValueError` and
  `TypeError` objects, and native lowering.

## Milestone 37: Array Pairing Builtin

- [x] Implement `array_combine($keys, $values)` over the current ordered array
  value model, including integer/string key-value conversion, length mismatch
  diagnostics, non-array diagnostics, fixture CLI coverage, documentation, and
  explicit gaps for references/copy-on-write, exact native `ValueError` and
  `TypeError` objects, object/resource keys, and native lowering.

## Milestone 38: Array Key Set Builtins

- [x] Implement `array_intersect_key($left, $right)` over the current ordered
  integer/string key model, including first-array key/value preservation,
  non-array diagnostics, fixture CLI coverage, documentation, and explicit
  gaps for variadic operands, references/copy-on-write, exact native
  `TypeError` objects, object/resource values, and native lowering.

## Milestone 39: Array Key Difference Builtins

- [x] Implement `array_diff_key($left, $right)` over the current ordered
  integer/string key model, including first-array key/value preservation for
  keys absent from the second array, non-array diagnostics, fixture CLI
  coverage, documentation, and explicit gaps for variadic operands,
  references/copy-on-write, exact native `TypeError` objects, object/resource
  values, and native lowering.

## Milestone 40: Array Key Set Variadic Follow-ups

- [x] Extend `array_intersect_key` beyond the current two-array slice with
  variadic array operands, including intersection across all subsequent
  arrays, non-array variadic operand diagnostics, fixture CLI coverage,
  documentation, and explicit gaps for references/copy-on-write, exact native
  `TypeError` objects, object/resource values, and native lowering.

## Milestone 41: Array Key Difference Variadic Follow-ups

- [x] Extend `array_diff_key` beyond the current two-array slice with variadic
  array operands, including differences against all subsequent arrays,
  non-array variadic operand diagnostics, fixture CLI coverage, documentation,
  and explicit gaps for references/copy-on-write, exact native `TypeError`
  objects, object/resource values, and native lowering.

## Milestone 42: Array Value Difference Builtins

- [x] Implement `array_diff($left, $right)` over the current scalar value
  subset, including first-array key/value preservation for values absent from
  the second array, non-array diagnostics, unsupported non-scalar comparison
  diagnostics, fixture CLI coverage, documentation, and explicit gaps for
  variadic operands, references/copy-on-write, exact native `TypeError`
  objects, object/resource values, and native lowering.

## Milestone 43: Array Value Intersection Builtins

- [x] Implement `array_intersect($left, $right)` over the current scalar value
  subset, including first-array key/value preservation for values present in
  the second array, non-array diagnostics, unsupported non-scalar comparison
  diagnostics, fixture CLI coverage, documentation, and explicit gaps for
  variadic operands, references/copy-on-write, exact native `TypeError`
  objects, object/resource values, and native lowering.

## Milestone 44: Array Value Intersection Variadic Follow-ups

- [x] Extend `array_intersect` beyond the current two-array slice with
  variadic array operands, including intersection across all subsequent arrays,
  non-array variadic operand diagnostics, fixture CLI coverage, documentation,
  and explicit gaps for non-scalar comparisons, references/copy-on-write,
  exact native `TypeError` objects, object/resource values, and native
  lowering.

## Milestone 45: Array Value Difference Variadic Follow-ups

- [x] Extend `array_diff` beyond the current two-array slice with variadic
  array operands, including differences against all subsequent arrays,
  non-array variadic operand diagnostics, fixture CLI coverage, documentation,
  and explicit gaps for non-scalar comparisons, references/copy-on-write,
  exact native `TypeError` objects, object/resource values, and native
  lowering.

## Milestone 46: Array Value Deduplication Builtins

- [x] Implement `array_unique($array)` over the current scalar string-form
  comparison subset, including first-occurrence key/value preservation,
  non-array diagnostics, unsupported non-scalar value diagnostics, fixture CLI
  coverage, documentation, and explicit gaps for sort flags,
  references/copy-on-write, exact native `TypeError` objects,
  object/resource values, and native lowering.

## Milestone 47: Array Replacement Builtins

- [x] Implement `array_replace($array, $replacement)` for two arrays over the
  current ordered integer/string key model, including replacement overwrite
  behavior, new-key insertion order, non-array diagnostics, fixture CLI
  coverage, documentation, and explicit gaps for references/copy-on-write,
  exact native `TypeError` objects, object/resource values, and native
  lowering.

## Milestone 48: Array Replacement Variadic Follow-ups

- [x] Extend `array_replace` beyond the current two-array slice with variadic
  replacement arrays, including left-to-right overwrite behavior across all
  replacements, non-array variadic operand diagnostics, fixture CLI coverage,
  documentation, and explicit gaps for references/copy-on-write, exact native
  `TypeError` objects, object/resource values, and native lowering.

## Milestone 49: Array Numeric Aggregation Builtins

- [x] Implement `array_sum($array)` over the current scalar numeric-coercion
  subset, including integer/float accumulation behavior, non-array diagnostics,
  fixture CLI coverage, documentation, and explicit gaps for PHP warning
  recovery, object/resource values, references/copy-on-write, exact native
  `TypeError` objects, and native lowering.

## Milestone 50: Array Numeric Product Builtins

- [x] Implement `array_product($array)` over the current scalar
  numeric-coercion subset, including integer/float accumulation behavior,
  non-array diagnostics, fixture CLI coverage, documentation, and explicit
  gaps for PHP warning recovery, object/resource values,
  references/copy-on-write, exact native `TypeError` objects, and native
  lowering.

## Milestone 51: Array Reduction Builtin

- [x] Implement `array_reduce($array, $callback)` for the current string-valued
  callback subset, including accumulator/current value callback invocation,
  non-array and callback diagnostics, fixture CLI coverage, documentation, and
  explicit gaps for references/copy-on-write, array/object callables,
  closures, exact native `TypeError` objects, and native lowering.

## Milestone 52: Array Reduction Initial Value

- [x] Extend `array_reduce` with third-argument initial value support over the
  current value model, including empty-array behavior, callback invocation with
  the supplied initial accumulator, fixture CLI coverage, documentation, and
  explicit gaps for references/copy-on-write, array/object callables,
  closures, exact native `TypeError` objects, and native lowering.

## Milestone 53: Array Filtering Null Callback

- [x] Implement `array_filter($array, null)` as the same falsey-value filtering
  path as omitted callbacks, including fixture CLI coverage, documentation,
  and explicit gaps for callback modes, references/copy-on-write, exact native
  `TypeError` objects, and native lowering.

## Milestone 54: Array Filtering Mode Follow-ups

- [x] Extend `array_filter` with integer mode flag `0` for the current
  null-callback and value-only string-callback paths, including fixture CLI
  coverage, documentation, and explicit gaps for key/key-value callback modes,
  named `ARRAY_FILTER_*` constants, references/copy-on-write, exact native
  `TypeError` objects, and native lowering.

## Milestone 55: Array Filtering Key Mode

- [x] Implement integer mode flag `2`/`ARRAY_FILTER_USE_KEY` behavior for
  `array_filter($array, $callback, 2)` over the current string-valued callback
  subset, including integer/string key callback arguments, fixture CLI
  coverage, documentation, and explicit gaps for named constants, key/value
  callback mode, array/object callables, closures, references/copy-on-write,
  exact native `TypeError` objects, and native lowering.

## Milestone 56: Array Filtering Key/Value Mode

- [x] Implement integer mode flag `1`/`ARRAY_FILTER_USE_BOTH` behavior for
  `array_filter($array, $callback, 1)` over the current string-valued callback
  subset, including callback invocation with value and key arguments, fixture
  CLI coverage, documentation, and explicit gaps for named constants,
  array/object callables, closures, references/copy-on-write, exact native
  `TypeError` objects, and native lowering.

## Milestone 57: Global Constant Boundary

- [x] Add explicit parse diagnostics for unsupported bare global constants such
  as `ARRAY_FILTER_USE_BOTH` before implementing constant resolution, including
  parser coverage, fixture CLI coverage, documentation, and named unsupported
  gaps for user-defined constants, extension constants, namespaces, and native
  lowering.

## Milestone 58: Built-in Constant Slice

- [x] Implement a narrow built-in global constant resolution slice for
  `ARRAY_FILTER_USE_KEY` and `ARRAY_FILTER_USE_BOTH`, including parser/runtime
  support, `array_filter` named-mode fixture CLI coverage, documentation, and
  explicit gaps for user-defined constants, extension constants, namespaces,
  `constant()` lookup, and native lowering.

## Milestone 59: Constant Lookup Boundary

- [x] Add an explicit `constant()` boundary: either a stable unsupported
  diagnostic for `constant(...)` before executable lookup exists, or the first
  narrow executable slice with parser/runtime tests, fixture CLI coverage,
  docs, and named gaps for user-defined, extension, namespace-qualified, and
  native-lowered constants.

## Milestone 60: User Constant Definition Boundary

- [x] Add an explicit `define(...)` boundary before user-defined constants are
  executable, including parser/runtime tests, fixture CLI coverage,
  documentation, and named gaps for runtime-defined constants, case-insensitive
  legacy constants, namespaces, extension constants, and native lowering.

## Milestone 61: User Constant Definition Slice

- [x] Implement a first runtime-defined constant table for
  `define($name, $value)` and `constant($name)` over a narrow string-name and
  scalar/array value subset, including duplicate-definition diagnostics,
  fixture CLI coverage, documentation, and explicit gaps for bare user
  constants, case-insensitive legacy constants, namespaces, extension
  constants, class constants, references/copy-on-write, and native lowering.

## Milestone 62: Bare User Constant Reads

- [x] Implement bare user constant reads for runtime-defined unqualified
  constants over the current name/value subset, including parser/runtime
  lookup, undefined/unsupported constant diagnostics, fixture CLI coverage,
  documentation, and explicit gaps for namespaces, extension constants, class
  constants, case-insensitive legacy constants, references/copy-on-write, and
  native lowering.

## Milestone 63: Constant Introspection Builtin

- [x] Implement `defined($name)` over the current built-in/runtime-defined
  constant table, including string-name validation, true/false behavior for
  existing and missing constants, string-valued dynamic-call availability,
  fixture CLI coverage, documentation, and explicit gaps for namespaces,
  extension constants, class constants, case-insensitive legacy constants,
  references/copy-on-write, and native lowering.

## Milestone 64: Global Constant Declaration Boundary

- [x] Add explicit parse diagnostics for unsupported top-level `const NAME =
  value;` declarations before implementing constant declarations, including
  parser coverage, fixture CLI coverage, documentation, and named gaps for
  namespace-aware constants, class constants, dynamic values, and native
  lowering.

## Milestone 65: Global Constant Declaration Slice

- [x] Implement top-level `const NAME = value;` declarations over the current
  constant-expression and value subset, including parser/interpreter support,
  duplicate and unsupported-value diagnostics, fixture CLI coverage,
  documentation, and explicit gaps for namespace-aware constants, class
  constants, references/copy-on-write, dynamic values, and native lowering.

## Milestone 66: Grouped Global Constant Declarations

- [x] Implement grouped top-level `const A = value, B = value;` declarations
  over the current constant-expression and value subset, including
  left-to-right duplicate diagnostics, fixture CLI coverage, documentation,
  and explicit gaps for namespace-aware constants, class constants,
  references/copy-on-write, dynamic values, and native lowering.

## Milestone 67: Constant Expression References

- [x] Allow top-level `const` declaration values to reference previously
  defined unqualified constants and the current built-in global constant
  slice, including left-to-right grouped declaration behavior, undefined-name
  diagnostics, fixture CLI coverage, documentation, and explicit gaps for
  forward references, namespace-aware constants, class constants,
  references/copy-on-write, dynamic values, and native lowering.

## Milestone 68: Default Parameter Constant References

- [x] Allow user-function default parameter values to reference previously
  defined unqualified constants and the current built-in global constant slice,
  including omitted-argument behavior, undefined-name diagnostics, fixture CLI
  coverage, documentation, and explicit gaps for forward references,
  namespace-aware constants, class constants, dynamic defaults,
  references/copy-on-write, and native lowering.

## Milestone 69: Function Type Declaration Boundaries

- [x] Add explicit parse diagnostics for unsupported user-function parameter
  type declarations and return type declarations before executable type
  enforcement exists, including parser coverage, fixture CLI coverage,
  documentation, and named gaps for nullable/union/intersection types,
  `mixed`, `void`/`never`, class/interface names, coercive vs strict typing,
  variance, and native lowering.

## Milestone 70: Static Local Boundary

- [x] Add explicit parse diagnostics for unsupported static local variable
  declarations inside functions before static local storage exists, including
  parser coverage, fixture CLI coverage, documentation, and named gaps for
  initialization expressions, per-function persistence, references,
  recursion/reentrancy behavior, and native lowering.

## Milestone 71: Magic Constant Boundary

- [x] Add explicit parse diagnostics for unsupported magic constants such as
  `__FUNCTION__`, `__METHOD__`, `__CLASS__`, `__FILE__`, `__DIR__`, and
  `__LINE__` before source-aware magic constant evaluation exists, including
  parser coverage, fixture CLI coverage, documentation, and named gaps for
  function/method/class context, line/file/dir source mapping, namespaces,
  traits, and native lowering.

## Milestone 72: Magic Constant Line Slice

- [x] Implement `__LINE__` as the first executable magic constant using
  expression source spans, including parser/interpreter support, fixture CLI
  coverage, documentation, and explicit gaps for `__FILE__`, `__DIR__`,
  `__FUNCTION__`, `__METHOD__`, `__CLASS__`, namespaces, traits, and native
  lowering.

## Milestone 73: Magic Constant File Slice

- [x] Implement `__FILE__` as the next executable magic constant using the
  current input path where available, including parser/interpreter support,
  fixture CLI coverage, documentation, and explicit gaps for `__DIR__`,
  eval/include source mapping, function/method/class context constants,
  namespaces, traits, and native lowering.

## Milestone 74: Magic Constant Directory Slice

- [x] Implement `__DIR__` as the next executable magic constant over the
  current input path boundary, including parser/interpreter support, fixture
  CLI coverage, documentation, and explicit gaps for eval/include source
  mapping, function/method/class context constants, namespaces, traits,
  canonical PHP path behavior, and native lowering.

## Milestone 75: Magic Constant Function Slice

- [x] Implement `__FUNCTION__` as the next executable magic constant for the
  current user-function context, including parser/interpreter support, fixture
  CLI coverage, documentation, and explicit gaps for method/class/trait
  context constants, namespaces, closures, eval/include source mapping,
  canonical PHP behavior, and native lowering.

## Milestone 76: Magic Constant Method Boundary

- [x] Define the next honest `__METHOD__` path, either by adding a stable
  unsupported diagnostic tied to the current no-method-dispatch boundary or by
  first implementing a minimal method execution slice that can evaluate
  `__METHOD__` honestly, including parser/interpreter support, fixture CLI
  coverage, documentation, and explicit gaps for class/trait/namespace
  contexts, closures, eval/include source mapping, canonical PHP behavior, and
  native lowering.

## Milestone 77: Magic Constant Class Boundary

- [x] Define the next honest `__CLASS__` path before executable class-context
  magic constant evaluation exists, including parser coverage, fixture CLI
  coverage, documentation, and explicit gaps for method/class context tracking,
  traits, namespaces, eval/include source mapping, canonical PHP behavior, and
  native lowering.

## Milestone 78: Magic Constant Trait Boundary

- [x] Define the next honest `__TRAIT__` path before executable trait-context
  magic constant evaluation exists, including parser coverage, fixture CLI
  coverage, documentation, and explicit gaps for trait declarations/use,
  method/class/trait context tracking, namespaces, eval/include source mapping,
  canonical PHP behavior, and native lowering.

## Milestone 79: Magic Constant Namespace Boundary

- [x] Define the next honest `__NAMESPACE__` path before executable
  namespace-context magic constant evaluation exists, including parser
  coverage, fixture CLI coverage, documentation, and explicit gaps for
  namespace declarations/imports, namespace-aware name resolution,
  eval/include source mapping, canonical PHP behavior, and native lowering.

## Milestone 80: Trait Declaration Boundary

- [x] Add explicit parse diagnostics for unsupported `trait` declarations
  before trait parsing or trait use execution exists, including parser
  coverage, fixture CLI coverage, documentation, and named gaps for trait
  methods/properties/constants, conflict resolution, aliases, visibility
  changes, namespace-aware traits, and native lowering.

## Milestone 81: Interface Declaration Boundary

- [x] Add explicit parse diagnostics for unsupported `interface` declarations
  before interface parsing or implementation execution exists, including parser
  coverage, fixture CLI coverage, documentation, and named gaps for interface
  constants, method signatures, inheritance, namespace-aware interfaces,
  class/interface type names, and native lowering.

## Milestone 82: Enum Declaration Boundary

- [x] Add explicit parse diagnostics for unsupported `enum` declarations before
  enum parsing or case/value execution exists, including parser coverage,
  fixture CLI coverage, documentation, and named gaps for backed enums, unit
  enums, methods, interfaces, namespace-aware enums, and native lowering.

## Milestone 83: Class Modifier Boundaries

- [x] Add explicit parse diagnostics for unsupported `abstract`, `final`, and
  `readonly` class modifiers before modifier-aware class parsing exists,
  including parser coverage, fixture CLI coverage, documentation, and named
  gaps for abstract methods/classes, final inheritance restrictions, readonly
  classes/properties, namespace-aware classes, and native lowering.

## Milestone 84: Class Member Modifier Boundaries

- [x] Add explicit parse diagnostics for unsupported `abstract`, `final`, and
  `readonly` class member modifiers before modifier-aware member parsing exists,
  including parser coverage, fixture CLI coverage, documentation, and named
  gaps for abstract methods, final methods, readonly properties, property
  initialization rules, inheritance interactions, and native lowering.

## Milestone 85: Typed Property Boundary

- [x] Add explicit parse diagnostics for unsupported typed property
  declarations before typed property storage or enforcement exists, including
  parser coverage for named, nullable, and union type forms, fixture CLI
  coverage, documentation, and named gaps for nullable/union/intersection
  enforcement, default values, readonly interactions, inheritance, reflection,
  exact PHP parse/error objects, and native lowering.

## Milestone 86: Property Default Boundary

- [x] Add explicit parse diagnostics and CLI coverage for unsupported property
  default values before property initializer execution exists, including
  documentation and named gaps for constant-expression defaults, array/object
  defaults, readonly initialization rules, inheritance/reflection behavior, and
  native lowering.

## Milestone 87: Multiple Property Declaration Boundary

- [x] Add explicit parse diagnostics and CLI coverage for unsupported multiple
  properties in one class property declaration before multi-property metadata
  parsing exists, including documentation and named gaps for per-property
  defaults, mixed visibility/static handling, typed properties, reflection, and
  native lowering.

## Milestone 88: Class Constant Declaration Boundary

- [x] Add explicit parse diagnostics and CLI coverage for unsupported class
  constant declarations before class constant metadata or lookup exists,
  including documentation and named gaps for visibility, typed constants,
  inheritance/override behavior, interface constants, reflection, and native
  lowering.

## Milestone 89: Object Context Boundary

- [x] Add an explicit diagnostic boundary for unsupported `$this` usage before
  method execution and object context binding exist, including parser/runtime
  coverage, fixture CLI coverage, documentation, and named gaps for method
  dispatch, constructor context, closures, static methods, inheritance, and
  native lowering.

## Milestone 90: Constructor Execution Boundary

- [x] Add explicit fixture and CLI coverage for unsupported constructor
  execution and constructor arguments before object initialization code can run
  user constructors, including runtime diagnostics, documentation, and named
  gaps for `$this` binding, property initialization, visibility, inheritance,
  promoted properties, exact PHP `Error` objects, and native lowering.

## Milestone 91: Clone Expression Boundary

- [ ] Add explicit parse diagnostics for unsupported `clone` expressions before
  object handle copying or `__clone` dispatch exists, including parser
  coverage, fixture CLI coverage, documentation, and named gaps for object
  identity, shallow/deep property copying, `__clone`, references, inheritance,
  exact PHP `Error` objects, and native lowering.
