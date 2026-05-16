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

- [x] Add explicit parse diagnostics for unsupported `clone` expressions before
  object handle copying or `__clone` dispatch exists, including parser
  coverage, fixture CLI coverage, documentation, and named gaps for object
  identity, shallow/deep property copying, `__clone`, references, inheritance,
  exact PHP `Error` objects, and native lowering.

## Milestone 92: Instanceof Expression Boundary

- [x] Add explicit parse diagnostics for unsupported `instanceof` expressions
  before class/interface relationship checks exist, including parser coverage,
  fixture CLI coverage, documentation, and named gaps for inheritance,
  interfaces, namespaces, autoloading, exact PHP `Error` objects, and native
  lowering.

## Milestone 93: Class Name Constant Boundary

- [x] Add explicit parse diagnostics for unsupported `ClassName::class`
  expressions before class-name constant resolution exists, including parser
  coverage, fixture CLI coverage, documentation, and named gaps for namespaces,
  aliases/imports, magic class names, autoloading, exact PHP behavior, and
  native lowering.

## Milestone 94: Magic Static Receiver Boundary

- [x] Add explicit parse diagnostics for unsupported magic static receivers
  such as `self::`, `parent::`, and `static::` before class-context,
  parent-class, or late-static-binding resolution exists, including parser
  coverage, fixture CLI coverage, documentation, and named gaps for static
  property storage, static method dispatch, class constants, inheritance, exact
  PHP `Error` objects, and native lowering.

## Milestone 95: Interface Implementation Boundary

- [x] Add explicit parse diagnostics for unsupported `implements` clauses
  before interface metadata and class/interface relationship checks exist,
  including parser coverage, fixture CLI coverage, documentation, and named
  gaps for interface method enforcement, inheritance, namespaces, autoloading,
  exact PHP `Error` objects, and native lowering.

## Milestone 96: Trait Use Boundary

- [x] Add explicit parse diagnostics for unsupported trait use inside class
  bodies before trait composition exists, including parser coverage, fixture
  CLI coverage, documentation, and named gaps for trait
  methods/properties/constants, conflict resolution, aliases, visibility
  adaptations, namespaces, exact PHP parse/error objects, and native lowering.

## Milestone 97: Object Introspection Builtins

- [x] Implement `get_class($object)` over the current minimal object value
  model, including declared class-name return behavior, string-valued
  dynamic-call availability, non-object diagnostics, fixture CLI coverage,
  documentation, and explicit gaps for inheritance, aliases/imports, anonymous
  classes, magic class names, object handle identity, exact native `TypeError`
  objects, and native lowering.

## Milestone 98: Object Type Introspection Builtins

- [x] Implement `is_object($value)` over the current minimal object value
  model, including true results for current object values, false results for
  scalars and arrays, string-valued dynamic-call availability, fixture CLI
  coverage, documentation, and explicit gaps for inheritance-aware object
  checks, proxy/extension object behavior, exact native reflection/type-system
  interactions, and native lowering.

## Milestone 99: Debug Type Introspection Builtin

- [x] Implement `get_debug_type($value)` over the current scalar/array/minimal
  object value model, including scalar and array type-name results, declared
  class-name results for current object values, string-valued dynamic-call
  availability, fixture CLI coverage, documentation, and explicit gaps for
  inheritance aliases/imports, anonymous classes, resources, exact native
  reflection/type-system interactions, and native lowering.

## Milestone 100: Class Metadata Introspection Builtin

- [x] Implement `class_exists($name[, $autoload])` over the current declared
  class metadata table, including case-insensitive class-name lookup,
  string-valued dynamic-call availability, non-string name and non-bool
  autoload diagnostics, fixture CLI coverage, documentation, and explicit gaps
  for autoloading, namespaces/import aliases, anonymous classes, exact native
  `TypeError` behavior, and native lowering.

## Milestone 101: Property Metadata Introspection Builtin

- [x] Implement `property_exists($object_or_class, $property)` over the current
  declared property metadata, including object and string-class inputs,
  case-sensitive property lookup, public/protected/private and static property
  reporting, invalid argument diagnostics, fixture CLI coverage, documentation,
  and explicit gaps for dynamic properties, autoloading, namespaces/import
  aliases, reflection behavior, exact native `TypeError` behavior, and native
  lowering.

## Milestone 102: Method Metadata Introspection Boundary

- [x] Add the next honest `method_exists($object_or_class, $method)` boundary
  over the current method metadata: either a stable unsupported diagnostic or a
  narrow executable metadata lookup slice with string-valued dynamic-call
  coverage, invalid argument diagnostics, fixture CLI coverage, documentation,
  and explicit gaps for method dispatch, inheritance, traits, interfaces,
  aliases/imports, autoloading, visibility behavior, exact native `TypeError`
  behavior, and native lowering.

## Milestone 103: Class Relationship Introspection Boundary

- [x] Add the next honest `is_a($object_or_class, $class_name[, $allow_string])`
  boundary over the current minimal object/class metadata: either a stable
  unsupported diagnostic or a narrow executable exact-class lookup slice with
  string-valued dynamic-call coverage, invalid argument diagnostics, fixture
  CLI coverage, documentation, and explicit gaps for inheritance, interfaces,
  traits, aliases/imports, autoloading, case handling, exact native
  `TypeError` behavior, and native lowering.

## Milestone 104: Subclass Relationship Boundary

- [x] Add the next honest `is_subclass_of($object_or_class, $class_name[, $allow_string])`
  boundary over the current no-inheritance class metadata: either a stable
  unsupported diagnostic or a narrow executable false-for-exact-class/no-parent
  slice with string-valued dynamic-call coverage, invalid argument diagnostics,
  fixture CLI coverage, documentation, and explicit gaps for inheritance,
  interfaces, traits, aliases/imports, autoloading, exact native `TypeError`
  behavior, and native lowering.

## Milestone 105: Parent Class Introspection Boundary

- [x] Add the next honest `get_parent_class($object_or_class)` boundary over
  the current no-inheritance class metadata: either a stable unsupported
  diagnostic or a narrow executable false-for-no-parent slice with
  string-valued dynamic-call coverage, invalid argument diagnostics, fixture
  CLI coverage, documentation, and explicit gaps for inheritance, interfaces,
  aliases/imports, namespace-aware names, autoloading, default `$this`
  behavior, exact native `TypeError` behavior, and native lowering.

## Milestone 106: Declared Class List Introspection Boundary

- [x] Add the next honest `get_declared_classes()` boundary over the current
  declared-class metadata: either a stable unsupported diagnostic or a narrow
  executable list slice for classes declared in the current program, with
  string-valued dynamic-call coverage, fixture CLI coverage, documentation,
  and explicit gaps for built-in/internal classes, extension classes,
  anonymous classes, autoloading, namespaces/import aliases, exact native
  ordering, and native lowering.

## Milestone 107: Class Method List Introspection Boundary

- [x] Add the next honest `get_class_methods($object_or_class)` boundary over
  the current method metadata: either a stable unsupported diagnostic or a
  narrow executable list slice for current object/string class inputs, with
  string-valued dynamic-call coverage, invalid argument diagnostics, fixture
  CLI coverage, documentation, and explicit gaps for inheritance, traits,
  interfaces, visibility filtering differences, aliases/imports, autoloading,
  exact native ordering, and native lowering.

## Milestone 108: Class Property Default List Boundary

- [x] Add the next honest `get_class_vars($class_name)` boundary over the
  current property metadata: either a stable unsupported diagnostic or a narrow
  executable list slice for declared string class inputs, with string-valued
  dynamic-call coverage, invalid argument diagnostics, fixture CLI coverage,
  documentation, and explicit gaps for property defaults, visibility filtering
  differences, inheritance, traits, aliases/imports, autoloading, exact native
  ordering, and native lowering.

## Milestone 109: Object Property Value List Boundary

- [x] Add the next honest `get_object_vars($object)` boundary over the current
  minimal object value model: either a stable unsupported diagnostic or a
  narrow executable public-property value slice for current object inputs, with
  string-valued dynamic-call coverage, invalid argument diagnostics, fixture
  CLI coverage, documentation, and explicit gaps for dynamic properties,
  visibility context, references/copy-on-write, inheritance, traits,
  aliases/imports, exact native ordering, and native lowering.

## Milestone 110: Declared Interface List Boundary

- [x] Add the next honest `get_declared_interfaces()` boundary over the current
  no-interface metadata model: either a stable unsupported diagnostic or a
  narrow executable empty-list slice with string-valued dynamic-call coverage,
  arity diagnostics, fixture CLI coverage, documentation, and explicit gaps
  for declared interfaces, built-in/internal interfaces, autoloading,
  namespaces/import aliases, exact native ordering, and native lowering.

## Milestone 111: Declared Trait List Boundary

- [x] Add the next honest `get_declared_traits()` boundary over the current
  no-trait metadata model: either a stable unsupported diagnostic or a narrow
  executable empty-list slice with string-valued dynamic-call coverage, arity
  diagnostics, fixture CLI coverage, documentation, and explicit gaps for
  declared traits, built-in/internal traits, autoloading, namespaces/import
  aliases, exact native ordering, and native lowering.

## Milestone 112: Interface Existence Boundary

- [x] Add the next honest `interface_exists($name[, $autoload])` boundary over
  the current no-interface metadata model: either a stable unsupported
  diagnostic or a narrow executable always-false slice with string-valued
  dynamic-call coverage, invalid argument diagnostics, fixture CLI coverage,
  documentation, and explicit gaps for declared interfaces, built-in/internal
  interfaces, autoloading, namespaces/import aliases, exact native `TypeError`
  behavior, and native lowering.

## Milestone 113: Trait Existence Boundary

- [x] Add the next honest `trait_exists($name[, $autoload])` boundary over the
  current no-trait metadata model: either a stable unsupported diagnostic or a
  narrow executable always-false slice with string-valued dynamic-call
  coverage, invalid argument diagnostics, fixture CLI coverage, documentation,
  and explicit gaps for declared traits, built-in/internal traits, autoloading,
  namespaces/import aliases, exact native `TypeError` behavior, and native
  lowering.

## Milestone 114: Enum Existence Boundary

- [x] Add the next honest `enum_exists($name[, $autoload])` boundary over the
  current no-enum metadata model: either a stable unsupported diagnostic or a
  narrow executable always-false slice with string-valued dynamic-call
  coverage, invalid argument diagnostics, fixture CLI coverage, documentation,
  and explicit gaps for declared enums, built-in/internal enums, autoloading,
  namespaces/import aliases, exact native `TypeError` behavior, and native
  lowering.

## Milestone 115: Called Class Context Boundary

- [x] Add the next honest `get_called_class()` boundary before method/static
  class context exists: either a stable unsupported diagnostic or a narrow
  executable context-aware slice, with parser/runtime coverage, fixture CLI
  coverage, documentation, and explicit gaps for method dispatch, late static
  binding, inheritance, namespaces/import aliases, exact native `Error`
  behavior, and native lowering.

## Milestone 116: Object Identity Boundary

- [x] Add the next honest `spl_object_id($object)` boundary before PHP object
  handle identity exists: either a stable unsupported diagnostic or a narrow
  executable identity slice for current minimal objects, with runtime coverage,
  fixture CLI coverage, documentation, and explicit gaps for handle reuse,
  references/copy-on-write, clone semantics, destructors, exact native
  `TypeError` behavior, and native lowering.

## Milestone 117: Object Hash Boundary

- [x] Add the next honest `spl_object_hash($object)` boundary before PHP
  object handle identity exists: either a stable unsupported diagnostic or a
  narrow executable hash slice for current minimal objects, with runtime
  coverage, fixture CLI coverage, documentation, and explicit gaps for handle
  reuse, references/copy-on-write, clone semantics, destructors, exact native
  `TypeError` behavior, and native lowering.

## Milestone 118: Mangled Object Vars Boundary

- [x] Add the next honest `get_mangled_object_vars($object)` boundary over the
  current minimal object value model before non-public property-name mangling
  and visibility-context behavior exist: either a stable unsupported
  diagnostic or a narrow executable public-property slice, with runtime
  coverage, fixture CLI coverage, documentation, and explicit gaps for
  protected/private name mangling, dynamic properties, references/copy-on-write,
  inheritance, traits, aliases/imports, exact native `TypeError` behavior, and
  native lowering.

## Milestone 119: Object Property Empty Boundary

- [x] Add `empty($object->publicProperty)` support for direct object-variable
  operands over the current public instance property model, including null,
  falsey, missing-property, undefined-variable, and non-object behavior,
  fixture CLI coverage, documentation, native-codegen rejection, and explicit
  gaps for dynamic property names, non-public visibility context, complex
  lvalues, magic methods, references/copy-on-write, and native lowering.

## Milestone 120: Object Property Unset Boundary

- [x] Add the next honest `unset($object->publicProperty)` boundary for direct
  object-variable operands before PHP property uninitialization semantics
  exist: either stable explicit diagnostics with parser/fixture CLI coverage
  or a narrow executable public-property slice with documented behavior,
  native-codegen rejection, and explicit gaps for typed/uninitialized
  properties, dynamic property names, non-public visibility context, magic
  methods, references/copy-on-write, and native lowering.

## Milestone 121: Exception Syntax Boundary

- [x] Add explicit diagnostics for unsupported exception syntax before
  exception objects or stack unwinding exist, including `throw`,
  `try`/`catch`/`finally`, parser coverage, fixture CLI coverage,
  documentation, and named gaps for `Throwable`, `Exception`, custom
  exception classes, `finally` execution, stack traces, exact native error
  objects, and native lowering.

## Milestone 122: Match Expression Boundary

- [x] Add explicit diagnostics for unsupported PHP 8 `match` expressions
  before expression-form branching exists, including parser coverage, fixture
  CLI coverage, documentation, and named gaps for strict arm matching,
  default arms, exhaustiveness errors, thrown expressions inside arms, value
  evaluation order, exact native error objects, and native lowering.

## Milestone 123: Ternary Expression Boundary

- [x] Add explicit diagnostics for unsupported ternary conditional expressions
  (`$condition ? $if_true : $if_false` and `$value ?: $fallback`) before
  expression-form branching exists, including parser coverage, fixture CLI
  coverage, documentation, and named gaps for condition truthiness, short
  ternary evaluation, nesting/precedence, thrown expressions inside arms,
  exact native error objects, and native lowering.

## Milestone 124: Null Coalescing Expression Boundary

- [x] Add explicit diagnostics for unsupported null coalescing expressions
  (`$value ?? $fallback`) before null-aware expression-form branching exists,
  including parser coverage, fixture CLI coverage, documentation, and named
  gaps for undefined-variable behavior, chained coalescing, precedence,
  assignment forms, exact native error objects, and native lowering.

## Milestone 125: Null Coalescing Expression Slice

- [x] Implement the first executable `??` expression slice for direct static
  variables and direct array offsets over the current value model, including
  undefined-variable, missing-key, null, falsey-non-null, and fallback
  evaluation behavior, fixture CLI coverage, documentation, explicit native
  lowering rejection, and named gaps for chained precedence interactions,
  `??=` assignment, property offsets, dynamic lvalues, references/copy-on-write,
  exact native error objects, and native lowering.

## Milestone 126: Null Coalescing Property Follow-up

- [x] Implement `??` for direct public object-property operands over the
  current minimal object value model, including null, falsey-non-null,
  missing-property, undefined-variable, and non-object target behavior, fixture
  CLI coverage, documentation, explicit native-codegen rejection, and named
  gaps for dynamic property names, non-public visibility context, magic
  methods, references/copy-on-write, exact native error objects, `??=`, and
  native lowering.

## Milestone 127: Null Coalescing Assignment Boundary

- [x] Add the next honest `??=` assignment path: either a stable explicit
  diagnostic that remains accurate after `??` expression support, or a first
  executable direct-variable slice with lazy fallback assignment behavior,
  fixture CLI coverage, documentation, native-codegen rejection, and named
  gaps for array offsets, object properties, dynamic lvalues,
  references/copy-on-write, exact native error objects, and native lowering.

## Milestone 128: Null Coalescing Array Assignment Follow-up

- [x] Implement direct array-offset `$array[$key] ??= expr` over the current
  ordered array value model, including undefined/null/missing-key behavior,
  lazy fallback assignment, non-array target behavior, fixture CLI coverage,
  documentation, native-codegen rejection, and named gaps for append offsets,
  nested offsets, object properties, dynamic lvalues, references/copy-on-write,
  exact native error objects, and native lowering.

## Milestone 129: Null Coalescing Object Assignment Follow-up

- [x] Implement direct public object-property `$object->property ??= expr` over
  the current minimal object value model, including undefined/null/missing
  property behavior, lazy fallback assignment, non-object target behavior,
  fixture CLI coverage, documentation, native-codegen rejection, and named gaps
  for dynamic property names, non-public visibility context, magic methods,
  references/copy-on-write, exact native error objects, and native lowering.

## Milestone 130: Assignment Expression Boundary

- [x] Add explicit parse diagnostics for expression-position assignment forms
  such as `($name = expr)` and `($name ??= expr)` before assignment expressions
  are executable values, including parser coverage, fixture CLI coverage,
  documentation, native-codegen parse-boundary behavior, and named gaps for
  assignment result values, chained assignments, lvalue evaluation order,
  references/copy-on-write, and exact native error objects.

## Milestone 131: Compound Assignment Boundary

- [x] Add explicit parse diagnostics for unsupported compound assignment forms
  such as `$name += expr`, `$name -= expr`, `$name *= expr`, `$name /= expr`,
  and `$name .= expr` before compound assignment execution exists, including
  parser coverage, fixture CLI coverage, documentation, native-codegen
  parse-boundary behavior, and named gaps for read-modify-write ordering,
  array/object targets, references/copy-on-write, numeric/string coercions,
  and exact native error objects.

## Milestone 132: Direct Variable Compound Assignment Slice

- [x] Implement direct static-variable compound assignment for `$name += expr`,
  `$name -= expr`, `$name *= expr`, `$name /= expr`, and `$name .= expr` over
  the current scalar value model, including read-modify-write behavior,
  undefined-variable diagnostics, fixture CLI coverage, documentation,
  native-codegen rejection while lowering remains unsupported, and explicit
  gaps for array/object targets, references/copy-on-write, increment/decrement
  operators, exact native error objects, and broader PHP coercion recovery.

## Milestone 133: Increment/Decrement Boundary

- [x] Add explicit parse diagnostics for unsupported pre/post increment and
  decrement operators such as `++$name`, `$name++`, `--$name`, and `$name--`
  before executable increment/decrement semantics exist, including parser
  coverage, fixture CLI coverage, documentation, native-codegen parse-boundary
  behavior, and named gaps for strings, arrays, objects, references,
  copy-on-write, and exact native warning/error behavior.

## Milestone 134: Direct Variable Increment/Decrement Slice

- [x] Implement direct static-variable pre/post increment and decrement for
  integer and float variables over statement-level forms, including
  read-modify-write behavior, undefined-variable diagnostics, fixture CLI
  coverage, documentation, native-codegen rejection while lowering remains
  unsupported, and explicit gaps for strings, arrays, objects, references,
  copy-on-write, expression result values, exact native warning/error behavior,
  and broader PHP coercion recovery.

## Milestone 135: For Header Increment/Decrement Slice

- [x] Implement direct static-variable pre/post increment and decrement in
  C-style `for` initializer/increment slots for integer and float variables,
  including loop execution behavior, undefined-variable and unsupported-type
  diagnostics, fixture CLI coverage, documentation, native-codegen rejection
  while lowering remains unsupported, and explicit gaps for strings,
  array/object targets, expression result values, references/copy-on-write,
  exact native warning/error behavior, broader coercion recovery, and native
  lowering.

## Milestone 136: Increment/Decrement Expression Follow-up

- [x] Implement expression-position direct static-variable pre/post increment
  and decrement for integer and float variables, including pre-vs-post result
  values, read-modify-write behavior, undefined-variable and unsupported-type
  diagnostics, fixture CLI coverage, documentation, native-codegen rejection
  while lowering remains unsupported, and explicit gaps for strings,
  array/object targets, chained increment/decrement expressions,
  references/copy-on-write, exact native warning/error behavior, broader
  coercion recovery, and native lowering.

## Milestone 137: Direct Variable Assignment Expression Slice

- [x] Implement expression-position direct static-variable assignment
  `$name = expr` for the current value model, including assignment result
  values, read/write ordering, fixture CLI coverage, documentation,
  native-codegen rejection while lowering remains unsupported, and explicit
  gaps for chained assignments, array/object targets, references,
  copy-on-write, exact native error objects, and native lowering.

## Milestone 138: Direct Variable Compound Assignment Expression Slice

- [x] Implement expression-position direct static-variable compound assignment
  such as `($name += expr)` for the current scalar value model, including
  assignment result values, read/write ordering, fixture CLI coverage,
  documentation, native-codegen rejection while lowering remains unsupported,
  and explicit gaps for array/object targets, references, copy-on-write,
  exact native error objects, broader PHP coercion recovery, and native
  lowering.

## Milestone 139: Array Offset Compound Assignment Boundary

- [x] Add the next honest path for `$array[$key] += expr` and related
  array-offset compound assignment forms: either executable direct
  array-offset read-modify-write semantics over the current ordered array
  model, or a tightened explicit diagnostic boundary with fixture CLI coverage,
  documentation, native-codegen behavior, and named gaps for append offsets,
  nested offsets, object properties, references, copy-on-write, exact native
  error objects, broader PHP coercion recovery, and native lowering.

## Milestone 140: Object Property Compound Assignment Boundary

- [x] Add the next honest path for `$object->property += expr` and related
  object-property compound assignment forms: either executable direct
  public-property read-modify-write semantics over the current object value
  model, or a tightened explicit diagnostic boundary with fixture CLI coverage,
  documentation, native-codegen behavior, and named gaps for dynamic property
  names, non-public visibility context, missing properties, references,
  copy-on-write, exact native error objects, broader PHP coercion recovery, and
  native lowering.

## Milestone 141: Object Property Increment Boundary

- [x] Add the next honest path for `++$object->property`, `$object->property++`,
  `--$object->property`, and `$object->property--`: either executable direct
  public-property increment/decrement semantics over the current object value
  model, or a tightened explicit diagnostic boundary with fixture CLI coverage,
  documentation, native-codegen behavior, and named gaps for string increment
  semantics, dynamic property names, non-public visibility context, missing
  properties, references, copy-on-write, exact native warning/error behavior,
  broader PHP coercion recovery, and native lowering.

## Milestone 142: Array Offset Increment Boundary

- [x] Add the next honest path for `++$array[$key]`, `$array[$key]++`,
  `--$array[$key]`, and `$array[$key]--`: either executable direct
  array-offset increment/decrement semantics over the current ordered array
  value model, or a tightened explicit diagnostic boundary with fixture CLI
  coverage, documentation, native-codegen behavior, and named gaps for append
  offsets, nested offsets, string increment semantics, missing keys,
  references, copy-on-write, exact native warning/error behavior, broader PHP
  coercion recovery, and native lowering.

## Milestone 143: Array Offset Assignment Expression Boundary

- [x] Add the next honest path for expression-position direct array-offset
  assignment such as `($array[$key] = expr)`: either executable assignment
  result semantics over the current ordered array value model, or a tightened
  explicit diagnostic boundary with fixture CLI coverage, documentation,
  native-codegen behavior, and named gaps for append offsets, nested offsets,
  object properties, references, copy-on-write, exact native error objects,
  lvalue evaluation order, and native lowering.

## Milestone 144: Object Property Assignment Expression Boundary

- [x] Add the next honest path for expression-position direct public
  object-property assignment such as `($object->property = expr)`: either
  executable assignment result semantics over the current object value model,
  or a tightened explicit diagnostic boundary with fixture CLI coverage,
  documentation, native-codegen behavior, and named gaps for dynamic property
  names, non-public visibility context, missing properties,
  references/copy-on-write, exact native error objects, lvalue evaluation
  order, and native lowering.

## Milestone 145: Append Offset Assignment Expression Boundary

- [x] Add the next honest path for expression-position array append
  assignment such as `($array[] = expr)`: either executable append assignment
  result semantics over the current ordered array value model, or a tightened
  explicit diagnostic boundary with fixture CLI coverage, documentation,
  native-codegen behavior, and named gaps for nested append offsets, object
  properties, references/copy-on-write, exact native error objects, lvalue
  evaluation order, and native lowering.

## Milestone 146: Null Coalescing Assignment Expression Boundary

- [x] Add the next honest path for expression-position null coalescing
  assignment such as `($name ??= expr)`, `($array[$key] ??= expr)`, and
  `($object->property ??= expr)`: either executable assignment result
  semantics over the current direct-variable/direct-offset/direct-property
  value model, or a tightened explicit diagnostic boundary with fixture CLI
  coverage, documentation, native-codegen behavior, and named gaps for nested
  lvalues, append offsets, dynamic property names, references/copy-on-write,
  lazy evaluation order, exact native error objects, and native lowering.

## Milestone 147: Chained Assignment Expression Boundary

- [x] Add the next honest path for chained assignment expressions such as
  `$left = $right = expr`: either executable right-to-left assignment result
  semantics for the current direct-variable/direct-offset/direct-property
  assignment-expression subset, or a tighter documented diagnostic boundary
  with fixture CLI coverage, native-codegen behavior, and named gaps for
  nested lvalues, append offsets, references/copy-on-write, exact native error
  objects, and native lowering.

## Milestone 148: Assignment Expression Follow-up Boundaries

- [x] Add the next honest boundary or executable slice for chained compound
  assignment and null coalescing assignment mixes such as `$left = ($right +=
  expr)` and `$left = ($right ??= expr)`, including parser/runtime tests,
  fixture CLI coverage, documentation, native-codegen behavior, and named gaps
  for nested lvalues, append offsets, references/copy-on-write, exact native
  error objects, and native lowering.

## Milestone 149: Assignment Expression Value Context Coverage

- [x] Add explicit executable coverage or tightened diagnostics for assignment
  expressions used as values in non-echo expression contexts such as function
  call arguments, array literal keys/values, `if`/loop conditions, and builtin
  arguments, including fixture CLI coverage, documentation, native-codegen
  behavior, and named gaps for nested lvalues, references/copy-on-write, exact
  native error objects, and native lowering.

## Milestone 150: Ternary Conditional Expression Slice

- [x] Implement full ternary conditional expressions
  `$condition ? $if_true : $if_false` over the current expression/value subset,
  including truthiness, lazy branch evaluation, nesting/precedence coverage,
  fixture CLI coverage, documentation, native-codegen rejection while lowering
  remains unsupported, and explicit gaps for short ternary `$value ?: $fallback`,
  throw expressions inside arms, references/copy-on-write, exact native error
  objects, and native lowering.

## Milestone 151: Short Ternary Conditional Expression Slice

- [x] Implement short ternary expressions `$value ?: $fallback` over the
  current expression/value subset, including condition value reuse, lazy
  fallback evaluation, truthiness coverage, fixture CLI coverage,
  documentation, native-codegen rejection while lowering remains unsupported,
  and explicit gaps for unparenthesized nested ternaries, throw expressions
  inside arms, references/copy-on-write, exact native error objects, and native
  lowering.

## Milestone 152: Ternary Precedence Coverage

- [x] Add explicit executable coverage for ternary expressions mixed with
  null-coalescing expressions and assignment-expression branches, including
  precedence/lazy-evaluation assertions, fixture CLI coverage, documentation,
  native-codegen rejection while lowering remains unsupported, and explicit
  gaps for unparenthesized nested ternaries, throw expressions inside arms,
  references/copy-on-write, exact native error objects, and native lowering.

## Milestone 153: Logical Operator Boundary

- [x] Add the next honest boundary or executable slice for logical operators
  `&&`, `||`, `and`, and `or`, including parser/runtime behavior, fixture CLI
  coverage, documentation, native-codegen behavior, and named gaps for
  precedence, short-circuiting, assignment-expression interaction,
  references/copy-on-write, exact native error objects, and native lowering.

## Milestone 154: Bitwise Operator Boundary

- [x] Add the next honest boundary or executable slice for bitwise operators
  `&`, `|`, and `^`, including parser/runtime behavior, fixture CLI coverage,
  documentation, native-codegen behavior, and named gaps for integer/string
  operand semantics, precedence, assignment-expression interaction,
  references/copy-on-write, exact native warning/error behavior, and native
  lowering.

## Milestone 155: Logical Xor Boundary

- [x] Add the next honest boundary or executable slice for logical `xor`,
  including parser/runtime behavior, fixture CLI coverage, documentation,
  native-codegen behavior, and named gaps for precedence, assignment-expression
  interaction, references/copy-on-write, exact native error behavior, and
  native lowering.

## Milestone 156: Bitwise Not Boundary

- [x] Add the next honest boundary or executable slice for unary bitwise not
  `~`, including parser/runtime behavior, fixture CLI coverage,
  documentation, native-codegen behavior, and named gaps for integer/string
  operand semantics, references/copy-on-write, exact native warning/error
  behavior, and native lowering.

## Milestone 157: Shift Operator Boundary

- [x] Add the next honest boundary or executable slice for shift operators
  `<<` and `>>`, including parser/runtime behavior, fixture CLI coverage,
  documentation, native-codegen behavior, and named gaps for integer operand
  coercion, negative shift counts, overflow behavior, references/copy-on-write,
  exact native warning/error behavior, and native lowering.

## Milestone 158: Bitwise Compound Assignment Boundary

- [x] Add the next honest boundary or executable slice for bitwise and shift
  compound assignment operators such as `&=`, `|=`, `^=`, `<<=`, and `>>=`,
  including parser/runtime behavior or stable diagnostics, fixture CLI
  coverage, documentation, native-codegen behavior, and named gaps for
  read-modify-write ordering, array/object targets, references/copy-on-write,
  exact native warning/error behavior, and native lowering.

## Milestone 159: Modulo Operator Boundary

- [x] Add the next honest boundary or executable slice for the modulo operator
  `%`, including parser/runtime behavior or stable diagnostics, fixture CLI
  coverage, documentation, native-codegen behavior, and named gaps for
  division-by-zero behavior, integer coercion, references/copy-on-write, exact
  native warning/error behavior, and native lowering.

## Milestone 160: Modulo Compound Assignment Boundary

- [x] Add the next honest boundary or executable slice for modulo compound
  assignment `%=`, including parser/runtime behavior or stable diagnostics,
  fixture CLI coverage, documentation, native-codegen behavior, and named gaps
  for read-modify-write ordering, array/object targets,
  references/copy-on-write, exact native warning/error behavior, and native
  lowering.

## Milestone 161: Native Modulo Lowering Boundary

- [x] Add the next honest boundary or executable slice for native modulo
  lowering, either by lowering a narrow integer `%` subset in LLVM IR/C
  assembly emission or by tightening explicit codegen diagnostics with fixture
  coverage, documentation, and named gaps for non-int coercions,
  modulo-by-zero behavior, references/copy-on-write, exact native error
  objects, and broader native lowering.

## Milestone 162: Native Division Safety Boundary

- [x] Add the next honest native-codegen safety boundary for division `/`,
  starting with compile-time zero-divisor diagnostics for statically known
  zero divisors in LLVM IR/C assembly emission, fixture CLI coverage,
  documentation, and named gaps for dynamic zero checks, PHP-shaped
  `DivisionByZeroError` objects, warnings/recovery, references/copy-on-write,
  and broader numeric lowering.

## Milestone 163: Native Dynamic Division Boundary

- [x] Add the next honest native-codegen safety boundary for division `/` with
  dynamic divisors, either by inserting a narrow runtime zero check for the
  current emitted numeric subset or by rejecting dynamic divisors explicitly
  until PHP-shaped native `DivisionByZeroError` objects exist; include fixture
  CLI coverage, documentation, and named gaps for warning/recovery,
  references/copy-on-write, string numeric coercions, and broader numeric
  lowering.

## Milestone 164: Native String Arithmetic Boundary

- [x] Add the next honest native-codegen boundary for string operands in
  arithmetic, either by implementing a narrow compile-time numeric-string
  coercion slice for lowerable arithmetic expressions or by tightening
  diagnostics and fixture CLI coverage for native string arithmetic rejection;
  include documentation and named gaps for PHP warning/recovery,
  non-numeric-string diagnostics, references/copy-on-write, exact native error
  objects, and broader numeric lowering.

## Milestone 165: Native Comparison Boundary

- [x] Add the next honest native-codegen boundary for comparison operators,
  either by lowering a narrow scalar comparison subset in LLVM IR/C assembly or
  by tightening explicit diagnostics with fixture CLI coverage, documentation,
  and named gaps for PHP comparison coercions, arrays/objects, `NAN`/`INF`,
  references/copy-on-write, exact native error objects, and broader native
  lowering.

## Milestone 166: Native Logical Operator Boundary

- [x] Add the next honest native-codegen boundary for logical operators
  `&&`, `||`, `and`, `xor`, and `or`, either by lowering a narrow boolean
  result subset with short-circuit behavior where required or by tightening
  explicit diagnostics with fixture CLI coverage, documentation, and named gaps
  for truthiness over arrays/objects, side-effect ordering,
  references/copy-on-write, exact native error objects, and broader native
  lowering.

## Milestone 167: Native Bitwise Operator Boundary

- [x] Add the next honest native-codegen boundary for bitwise and shift
  operators `&`, `|`, `^`, `~`, `<<`, and `>>`, either by lowering a narrow
  integer-only subset or by tightening explicit diagnostics with fixture CLI
  coverage, documentation, and named gaps for string bytewise operations,
  scalar-to-int coercion, negative/large shifts, references/copy-on-write,
  exact native error objects, and broader native lowering.

## Milestone 168: Native Conditional Expression Boundary

- [x] Add the next honest native-codegen boundary for ternary and/or null
  coalescing expressions, either by lowering a narrow side-effect-safe scalar
  subset or by tightening explicit diagnostics with fixture CLI coverage,
  documentation, and named gaps for PHP truthiness, null-aware lookup,
  side-effect ordering, references/copy-on-write, exact native error objects,
  and broader native lowering.

## Milestone 169: Native Function Call Boundary

- [x] Add the next honest native-codegen boundary for function calls, including
  user functions, callable builtins, and dynamic string-valued calls, either by
  lowering a narrow direct-call subset or by tightening explicit diagnostics
  with fixture CLI coverage, documentation, and named gaps for runtime call
  lookup, arity/type diagnostics, stack frames, callbacks, references,
  exact native error objects, and broader native lowering.

## Milestone 170: Native Function Declaration Boundary

- [x] Add the next honest native-codegen boundary for user-function
  declarations and returns, either by lowering a narrow no-capture direct-call
  subset or by tightening explicit diagnostics with fixture CLI coverage,
  documentation, and named gaps for function symbol tables, stack-frame layout,
  default parameters, recursion guards, return-value flow, references,
  exact native error objects, and broader native lowering.

## Milestone 171: Native Magic Constant Boundary

- [x] Add the next honest native-codegen boundary for executable magic
  constants `__LINE__`, `__FILE__`, `__DIR__`, and `__FUNCTION__`, either by
  lowering a narrow source-aware subset or by tightening explicit diagnostics
  with fixture CLI coverage, documentation, and named gaps for source mapping,
  path canonicalization, function context, eval/include interactions,
  references, exact native error objects, and broader native lowering.

## Milestone 172: Native Global Constant Boundary

- [x] Add the next honest native-codegen boundary for built-in,
  runtime-defined, bare-read, and top-level declared global constants, either
  by lowering a narrow constant table subset or by tightening explicit
  diagnostics with fixture CLI coverage, documentation, and named gaps for
  source-order definitions, runtime `define(...)`, `constant()`/`defined()`,
  namespaces, class constants, references/copy-on-write, exact native error
  objects, and broader native lowering.

## Milestone 173: Native Object/Class Boundary

- [x] Add the next honest native-codegen boundary for class declarations,
  object instantiation, public property reads/writes, and object metadata
  builtins, either by lowering a narrow object metadata subset or by tightening
  explicit diagnostics with fixture CLI coverage, documentation, and named
  gaps for object handles, constructors, `$this`, method dispatch, visibility,
  references/copy-on-write, exact native error objects, and broader native
  lowering.

## Milestone 174: Native Array Boundary

- [x] Add the next honest native-codegen boundary for array literals, array
  offset reads/writes, `foreach`/`unset` array operations, and array builtins,
  either by lowering a narrow ordered-array subset or by tightening explicit
  diagnostics with fixture CLI coverage, documentation, and named gaps for
  array storage layout, key normalization, copy-on-write, references,
  callbacks, exact native error objects, and broader native lowering.

## Milestone 175: Native Control-Flow Boundary

- [x] Add the next honest native-codegen boundary for control-flow statements
  such as `if`/`elseif`/`else`, `while`, `for`, `do ... while`, `switch`,
  `break`, and `continue`, either by lowering a narrow structured-control
  subset or by tightening explicit diagnostics with fixture CLI coverage,
  documentation, and named gaps for PHP truthiness, branch layout, loop
  control flow, switch fallthrough, references/copy-on-write side effects,
  exact native error objects, and broader native lowering.

## Milestone 176: Native Mutation Boundary

- [x] Add the next honest native-codegen boundary for mutation forms that are
  still only interpreter-backed, including compound assignment, null
  coalescing assignment, increment/decrement, assignment expressions, direct
  variable unset, and multiple-operand unset, either by lowering a narrow
  direct-variable subset or by tightening explicit diagnostics with fixture CLI
  coverage, documentation, and named gaps for read-modify-write ordering,
  null-aware mutation, unset symbol-table effects, references/copy-on-write,
  exact native error objects, and broader native lowering.

## Milestone 177: Native Unary Boundary

- [x] Add the next honest native-codegen boundary for unary minus and logical
  not, either by lowering a narrow scalar subset or by tightening explicit
  diagnostics with fixture CLI coverage, documentation, and named gaps for PHP
  numeric coercion, truthiness conversion, references/copy-on-write, exact
  native error objects, and broader native lowering.

## Milestone 178: Native Arithmetic Boundary

- [x] Add the next honest native-codegen boundary for binary arithmetic
  operators `+`, `-`, `*`, `/`, and `%`, either by broadening a narrow native
  scalar subset or by tightening explicit diagnostics with fixture CLI
  coverage, documentation, and named gaps for PHP numeric coercion, dynamic
  division/modulo zero checks, modulo coercions, references/copy-on-write,
  exact native error objects, and broader native lowering.

## Milestone 179: Native Concatenation Boundary

- [x] Add the next honest native-codegen boundary for string concatenation `.`
  before generated code claims PHP echo/string conversion, dynamic allocation,
  references/copy-on-write side effects, exact native error objects, or broader
  string lowering.

## Milestone 180: Native Scalar Echo Boundary

- [x] Add the next honest native-codegen boundary for remaining straight-line
  scalar echo/assignment lowering, either by proving the current literal and
  static-variable subset with stronger CLI coverage or by tightening explicit
  diagnostics with documentation and named gaps for PHP string formatting,
  dynamic values, references/copy-on-write, exact native error objects, and
  broader native lowering.

## Milestone 181: Native Variable Read Boundary

- [x] Add the next honest native-codegen boundary for undefined or dynamic
  variable reads in the straight-line lowerer, including explicit diagnostics,
  fixture CLI coverage, documentation, and named gaps for native symbol-table
  storage, references/copy-on-write, exact PHP undefined-variable error
  objects, and broader native lowering.

## Milestone 182: Native Assembly CLI Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI exercise coverage for the
  current lowerable scalar echo/assignment subset without snapshotting
  platform-specific assembly text, including tests, documentation, and named
  gaps for linking/execution, PHP zvals, symbol-table storage,
  references/copy-on-write, exact native error objects, and broader native
  lowering.

## Milestone 183: Native Assembly Rejection CLI Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI rejection coverage for a
  representative unsupported native boundary, proving assembly emission exits
  before invoking backend tools when LLVM lowering rejects a program, including
  tests, documentation, and named gaps for backend-independent native
  diagnostics, exact native error objects, and broader native lowering.

## Milestone 184: Native Assembly Backend Absence CLI Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for the lowerable
  scalar subset when no assembly backend tools are available, including a
  stable missing-backend diagnostic snapshot, documentation, and named gaps for
  bundled toolchains, linking/execution, exact native error objects, and
  broader native lowering.

## Milestone 185: Native Assembly C Fallback CLI Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for the documented
  `cc -S` fallback path when LLVM assembly tools are unavailable but a C
  compiler backend exists, including normalized output rather than
  platform-specific assembly text, documentation, and named gaps for bundled
  toolchains, linking/execution, exact native error objects, and broader native
  lowering.

## Milestone 186: Native Assembly Backend Failure CLI Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for selected backend
  failure diagnostics when an available assembly backend exits nonzero,
  including stable stderr normalization, documentation, and named gaps for
  backend-specific diagnostics, bundled toolchains, exact native error objects,
  and broader native lowering.

## Milestone 187: Native Assembly LLC Selection CLI Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for the `llc`
  selected-backend path when `clang` is unavailable but `llc` is available,
  using normalized output or deterministic test doubles rather than
  platform-specific assembly text, including documentation and named gaps for
  backend-specific assembly, bundled toolchains, linking/execution, exact
  native error objects, and broader native lowering.

## Milestone 188: Native Assembly LLC Failure CLI Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for selected `llc`
  backend failure diagnostics when `clang` is unavailable and available `llc`
  exits nonzero, including stable stderr normalization, documentation, and
  named gaps for backend-specific diagnostics, bundled toolchains, exact
  native error objects, and broader native lowering.

## Milestone 189: Native Assembly C Fallback Failure CLI Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for `cc -S` fallback
  failure diagnostics when `clang` and `llc` are unavailable and available
  `cc` exits nonzero, including stable stderr normalization, documentation,
  and named gaps for backend-specific diagnostics, bundled toolchains, exact
  native error objects, and broader native lowering.

## Milestone 190: Native Assembly Backend Discovery Edge Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for assembly backend
  discovery edge cases where a candidate backend command exists but fails its
  `--version` probe before fallback selection, including deterministic test
  doubles, stable diagnostics or fallback behavior, documentation, and named
  gaps for bundled toolchains, backend-specific discovery semantics, exact
  native error objects, and broader native lowering.

## Milestone 191: Native Assembly Backend Discovery Exhaustion Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for assembly backend
  discovery exhaustion where `clang`, `llc`, and `cc` commands exist but all
  fail their `--version` probes, including deterministic test doubles, the
  stable missing-backend diagnostic, documentation, and named gaps for bundled
  toolchains, backend-specific discovery semantics, exact native error objects,
  and broader native lowering.

## Milestone 192: Native Assembly Empty Backend Stderr Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for selected backend
  failures that exit nonzero without stderr output, including deterministic
  test doubles, stable diagnostics, documentation, and named gaps for
  backend-specific diagnostics, bundled toolchains, exact native error objects,
  and broader native lowering.

## Milestone 193: Native Assembly Empty Backend Output Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for selected backend
  success cases that produce empty assembly stdout, either by rejecting empty
  output with a stable diagnostic or documenting and testing the current
  behavior, including deterministic test doubles, documentation, and named gaps
  for backend-specific diagnostics, bundled toolchains, exact native error
  objects, and broader native lowering.

## Milestone 194: Native Assembly Successful Backend Stderr Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for selected backend
  success cases that also write stderr diagnostics, either by documenting and
  testing the current success-with-stderr behavior or rejecting it with a
  stable diagnostic, including deterministic test doubles, documentation, and
  named gaps for backend-specific diagnostics, bundled toolchains, exact native
  error objects, and broader native lowering.

## Milestone 195: Native Assembly Successful Fallback Stderr Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for the successful
  `llc` and `cc` backend paths when they also write stderr diagnostics,
  including deterministic test doubles, normalized output, documentation, and
  named gaps for backend-specific diagnostics, bundled toolchains, exact native
  error objects, and broader native lowering.

## Milestone 196: Native Assembly Empty Fallback Stderr Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for selected `llc`
  and `cc` backend failures that exit nonzero without stderr output, including
  deterministic test doubles, stable diagnostics, documentation, and named gaps
  for backend-specific diagnostics, bundled toolchains, exact native error
  objects, and broader native lowering.

## Milestone 197: Native Assembly Empty Fallback Output Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for selected `llc`
  and `cc` backend success cases that produce empty assembly stdout, including
  deterministic test doubles, stable diagnostics, documentation, and named gaps
  for backend-specific diagnostics, bundled toolchains, exact native error
  objects, and broader native lowering.

## Milestone 198: Native Assembly Whitespace Fallback Output Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for selected fallback
  backend success cases that produce only whitespace assembly stdout, either by
  rejecting whitespace-only output with stable diagnostics or by documenting and
  testing the current behavior, including deterministic test doubles,
  documentation, and named gaps for backend-specific assembly validation,
  bundled toolchains, exact native error objects, and broader native lowering.

## Milestone 199: Native Assembly Whitespace Selected-Backend Output Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for the selected
  `clang` backend success case that produces only whitespace assembly stdout,
  proving the shared whitespace-only-output diagnostic applies before fallback
  selection too, including deterministic test doubles, documentation, and
  named gaps for backend-specific assembly validation, bundled toolchains,
  exact native error objects, and broader native lowering.

## Milestone 200: Native Assembly Invalid Success Output Stderr Precedence

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for a selected
  `clang` backend success case that writes stderr diagnostics while producing
  whitespace-only assembly stdout, proving stdout validation wins and backend
  stderr remains unsurfaced on invalid successful output, including
  deterministic test doubles, documentation, and named gaps for
  backend-specific assembly validation, bundled toolchains, exact native error
  objects, and broader native lowering.

## Milestone 201: Native Assembly Invalid Fallback Output Stderr Precedence

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for fallback `llc`
  and `cc` backend success cases that write stderr diagnostics while producing
  whitespace-only assembly stdout, proving stdout validation wins and backend
  stderr remains unsurfaced on invalid successful output after fallback
  selection, including deterministic test doubles, documentation, and named
  gaps for backend-specific assembly validation, bundled toolchains, exact
  native error objects, and broader native lowering.

## Milestone 202: Native Assembly Backend Input Validation

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for a selected
  `clang` backend test double that validates the generated LLVM IR arrives on
  stdin with representative `main` and `printf` markers before emitting
  assembly, including deterministic test doubles, normalized CLI output,
  documentation, and named gaps for backend-specific IR validation, bundled
  toolchains, exact native error objects, and broader native lowering.

## Milestone 203: Native Assembly Fallback Input Validation

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for fallback `llc`
  and `cc` backend test doubles that validate generated input arrives on stdin
  with representative LLVM IR or generated C markers before emitting assembly,
  including deterministic test doubles, normalized CLI output, documentation,
  and named gaps for backend-specific IR/C validation, bundled toolchains,
  exact native error objects, and broader native lowering.

## Milestone 204: Native Assembly Backend Argument Validation

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for selected and
  fallback backend invocation arguments, using deterministic `clang`, `llc`,
  and `cc` test doubles that validate expected argument vectors before
  accepting stdin and emitting assembly, including normalized CLI output,
  documentation, and named gaps for backend-specific command-line compatibility,
  bundled toolchains, exact native error objects, and broader native lowering.

## Milestone 205: Native Assembly Backend Probe Argument Validation

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for assembly backend
  discovery probe argument vectors, using deterministic `clang`, `llc`, and
  `cc` test doubles that validate exact `--version` probe invocations before
  selected/fallback assembly emission, including normalized CLI output,
  documentation, and named gaps for backend-specific discovery semantics,
  bundled toolchains, exact native error objects, and broader native lowering.

## Milestone 206: Native Assembly Backend Probe Output Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for successful
  assembly backend discovery probes that write stdout/stderr diagnostics while
  still passing discovery, using deterministic selected and fallback backend
  test doubles, including normalized CLI output, documentation, and named gaps
  for backend-specific discovery output semantics, bundled toolchains, exact
  native error objects, and broader native lowering.

## Milestone 207: Native Assembly Failed Probe Output Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for failed assembly
  backend discovery probes that write stdout/stderr diagnostics before
  fallback selection or missing-backend reporting, using deterministic backend
  test doubles, including normalized CLI output, documentation, and named gaps
  for backend-specific failed-probe output semantics, bundled toolchains, exact
  native error objects, and broader native lowering.

## Milestone 208: Native Assembly Backend Start Failure Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for an assembly
  backend command that passes discovery but cannot be started for actual
  assembly emission, using a deterministic race-like test double, including
  stable diagnostics, documentation, and named gaps for backend race
  conditions, bundled toolchains, exact native error objects, and broader
  native lowering.

## Milestone 209: Native Assembly Fallback Backend Start Failure Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for fallback `llc`
  and `cc` backend commands that pass discovery but cannot be started for
  actual assembly emission, using deterministic race-like test doubles,
  including stable diagnostics, documentation, and named gaps for backend race
  conditions, bundled toolchains, exact native error objects, and broader
  native lowering.

## Milestone 210: Native Assembly Backend Selection Precedence

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for backend
  selection precedence when `clang`, `llc`, and `cc` are all available,
  proving selected `clang` is used before fallback tools with deterministic
  test doubles, documentation, and named gaps for full backend-specific
  discovery semantics, bundled toolchains, exact native error objects, and
  broader native lowering.

## Milestone 211: Native Assembly Fallback Selection Precedence

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for fallback backend
  selection precedence when `clang` is unavailable and both `llc` and `cc` are
  available, proving selected `llc` is used before the C fallback with
  deterministic test doubles, documentation, and named gaps for full
  backend-specific discovery semantics, bundled toolchains, exact native error
  objects, and broader native lowering.

## Milestone 212: Native Assembly Selected Backend Failure Precedence

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for selected backend
  failure precedence when `clang` passes discovery but fails assembly emission
  while fallback `llc` and `cc` commands are also available, proving the
  selected-backend failure is reported without silently falling through to
  fallback tools, with deterministic test doubles, documentation, and named
  gaps for full backend recovery semantics, bundled toolchains, exact native
  error objects, and broader native lowering.

## Milestone 213: Native Assembly Fallback Failure Precedence

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for fallback backend
  failure precedence when `clang` is unavailable, `llc` passes discovery but
  fails assembly emission, and `cc` is also available, proving the selected
  `llc` failure is reported without silently falling through to the `cc -S`
  fallback, with deterministic test doubles, documentation, and named gaps for
  full backend recovery semantics, bundled toolchains, exact native error
  objects, and broader native lowering.

## Milestone 214: Native Assembly Empty-Stderr Fallback Failure Precedence

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for fallback backend
  failure precedence when `clang` is unavailable, selected `llc` passes
  discovery but exits nonzero without stderr, and `cc` is also available,
  proving the stable empty-stderr `llc` diagnostic is reported without
  silently falling through to the `cc -S` fallback, with deterministic test
  doubles, documentation, and named gaps for full backend recovery semantics,
  bundled toolchains, exact native error objects, and broader native lowering.

## Milestone 215: Native Assembly Empty-Stderr Selected Backend Failure Precedence

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for selected backend
  failure precedence when selected `clang` passes discovery but exits nonzero
  without stderr while fallback `llc` and `cc` commands are also available,
  proving the stable empty-stderr `clang` diagnostic is reported without
  silently falling through to fallback tools, with deterministic test doubles,
  documentation, and named gaps for full backend recovery semantics, bundled
  toolchains, exact native error objects, and broader native lowering.

## Milestone 216: Native Assembly Selected Backend Start-Failure Precedence

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for selected backend
  start-failure precedence when selected `clang` passes discovery but cannot be
  started for assembly emission while fallback `llc` and `cc` commands are
  also available, proving the stable selected-backend start diagnostic is
  reported without silently falling through to fallback tools, with
  deterministic test doubles, documentation, and named gaps for full backend
  recovery semantics, bundled toolchains, exact native error objects, and
  broader native lowering.

## Milestone 217: Native Assembly Fallback Backend Start-Failure Precedence

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for fallback backend
  start-failure precedence when `clang` is unavailable, selected `llc` passes
  discovery but cannot be started for assembly emission while `cc` is also
  available, proving the stable `llc` start diagnostic is reported without
  silently falling through to the `cc -S` fallback, with deterministic test
  doubles, documentation, and named gaps for full backend recovery semantics,
  bundled toolchains, exact native error objects, and broader native lowering.

## Milestone 218: Native Assembly Probe Start-Failure Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for discovery probe
  start-failure cases where candidate backend command names exist but cannot
  be started for `--version`, proving those failed starts are treated as
  unavailable before fallback selection or missing-backend diagnostics, with
  deterministic test doubles, documentation, and named gaps for bundled
  toolchains, backend-specific discovery semantics, exact native error
  objects, and broader native lowering.

## Milestone 219: Native Assembly Probe Permission-Denied Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for discovery probe
  permission-denied cases where candidate backend command names exist on
  `PATH` but are not executable for `--version`, proving those probe failures
  are treated as unavailable before fallback selection or missing-backend
  diagnostics, with deterministic test doubles, documentation, and named gaps
  for bundled toolchains, backend-specific discovery semantics, exact native
  error objects, and broader native lowering.

## Milestone 220: Native Assembly Selected Backend Permission-Denied Emission Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for a selected
  backend command that passes discovery but becomes non-executable before
  actual assembly emission, proving the stable selected-backend start
  diagnostic is reported for permission-denied emission starts, with
  deterministic test doubles, documentation, and named gaps for backend race
  conditions, bundled toolchains, exact native error objects, and broader
  native lowering.

## Milestone 221: Native Assembly Fallback Backend Permission-Denied Emission Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for a fallback
  backend command that passes discovery but becomes non-executable before
  actual assembly emission, proving the stable fallback-backend start
  diagnostic is reported for permission-denied emission starts without
  silently falling through to later fallbacks, with deterministic test
  doubles, documentation, and named gaps for backend race conditions, bundled
  toolchains, exact native error objects, and broader native lowering.

## Milestone 222: Native Scalar Print Assembly Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for a lowerable
  straight-line scalar program that mixes `echo` and `print`, including
  fixture CLI coverage, documentation of the current native scalar output
  boundary, and named gaps for control flow, runtime-backed output conversion,
  exact native PHP errors, and broader native lowering.

## Milestone 223: Native Scalar Output C Fallback Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for the documented
  `cc -S` fallback using a lowerable straight-line scalar program that mixes
  `echo` and `print`, including fixture CLI coverage, documentation of the C
  fallback output boundary, and named gaps for runtime-backed output
  conversion, exact native PHP errors, linking/execution, and broader native
  lowering.

## Milestone 224: Native Scalar Reassignment Assembly Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for a lowerable
  straight-line scalar reassignment program, including fixture CLI coverage,
  documentation of the current static-variable overwrite boundary, and named
  gaps for native symbol-table storage, references/copy-on-write, exact native
  PHP errors, linking/execution, and broader native lowering.

## Milestone 225: Native Scalar Reassignment C Fallback Coverage

- [x] Add explicit `phpc compile --emit-asm` CLI coverage for the documented
  `cc -S` fallback using a lowerable straight-line scalar reassignment program,
  including fixture CLI coverage, generated C fallback validation,
  documentation of the current static-variable overwrite boundary, and named
  gaps for native symbol-table storage, references/copy-on-write, exact native
  PHP errors, linking/execution, and broader native lowering.

## Milestone 226: Native Scalar Reassignment IR Snapshot Coverage

- [x] Add explicit `phpc compile --emit-ir` CLI coverage for a lowerable
  straight-line scalar reassignment program, including committed IR snapshot
  coverage that shows only the final overwritten scalar values are emitted,
  documentation of the current static-variable overwrite boundary, and named
  gaps for native symbol-table storage, references/copy-on-write, exact native
  PHP errors, linking/execution, and broader native lowering.

## Milestone 227: Native Scalar Reassignment Unit Coverage

- [x] Add focused unit coverage for `emit_ir_source` on a lowerable
  straight-line scalar reassignment program, asserting final overwritten scalar
  values are emitted and overwritten values are absent before broader native
  symbol-table storage, references/copy-on-write, exact native PHP errors,
  linking/execution, and broader native lowering exist.

## Milestone 228: Native Scalar Reassignment ASM API Coverage

- [x] Add focused API-level coverage for `emit_asm_source` on a lowerable
  straight-line scalar reassignment program, using the existing available
  backend skip pattern and documenting that this only proves assembly emission
  succeeds for the current static overwrite subset, not linking/execution,
  native symbol-table storage, references/copy-on-write, exact native PHP
  errors, or broader native lowering.

## Milestone 229: Native Scalar Reassignment Boundary Consolidation

- [x] Review the scalar reassignment native-lowering coverage added in
  Milestones 224 through 228 for duplicate fixture or test structure, then
  consolidate helpers or documentation only where it reduces maintenance risk
  without weakening the explicit CLI, fallback, IR snapshot, and API coverage.

## Milestone 230: Native Scalar Reassignment Focused Regression Run

- [x] Run the focused scalar reassignment regression set covering Milestones
  224 through 229, including fixture tests, system PHP comparisons, `--emit-ir`
  snapshot coverage, `--emit-asm` selected-backend and C-fallback CLI coverage,
  and API-level unit coverage. Fix any failures before considering the
  reassignment boundary complete enough for a later full-suite gate.

## Milestone 231: Native Scalar Reassignment Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the scalar reassignment
  native-lowering coverage, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 232: Native Scalar Reassignment Checkpoint Decision

- [x] Decide whether to checkpoint the scalar reassignment native-lowering
  coverage now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 233: Native Integer Arithmetic Lowering Slice

- [x] Lower static straight-line integer `+`, `-`, and `*` expressions through
  `phpc compile --emit-ir` and `--emit-asm`, including deterministic LLVM IR
  and C fallback assembly CLI coverage, fixture coverage through `phpc run`
  and system PHP comparison, updated diagnostics for unsupported arithmetic
  operands/operators, documentation, and named gaps for PHP numeric coercion,
  floats, `/`, `%`, overflow behavior, references/copy-on-write, exact native
  PHP errors, linking/execution, and broader native lowering.

## Milestone 234: Native Integer Arithmetic Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native integer
  arithmetic lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 235: Native Integer Arithmetic Checkpoint Decision

- [x] Decide whether to checkpoint the native integer arithmetic lowering
  slice now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 236: Native Integer Unary Minus Lowering Slice

- [x] Lower static straight-line integer unary minus through `phpc compile
  --emit-ir` and `--emit-asm`, including deterministic LLVM IR and C fallback
  assembly CLI coverage, fixture coverage through `phpc run` and system PHP
  comparison, updated diagnostics for unsupported unary operands/operators,
  documentation, and named gaps for PHP numeric coercion, floats, booleans,
  strings, nulls, arrays, objects, logical not, overflow behavior,
  references/copy-on-write, exact native PHP errors, linking/execution, and
  broader native lowering.

## Milestone 237: Native Integer Unary Minus Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native integer unary
  minus lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 238: Native Integer Unary Minus Checkpoint Decision

- [x] Decide whether to checkpoint the native integer unary minus lowering
  slice now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 239: Native Boolean Logical Not Lowering Slice

- [x] Lower static straight-line boolean logical not through `phpc compile
  --emit-ir` and `--emit-asm`, including deterministic LLVM IR and C fallback
  assembly CLI coverage, fixture coverage through `phpc run` and system PHP
  comparison, updated diagnostics for unsupported unary operands/operators,
  documentation, and named gaps for general PHP truthiness conversion, numeric
  coercion, float/string/null/array/object unary operands, overflow behavior,
  references/copy-on-write, exact native PHP errors, linking/execution, and
  broader native lowering.

## Milestone 240: Native Boolean Logical Not Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native boolean logical
  not lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 241: Native Boolean Logical Not Checkpoint Decision

- [x] Decide whether to checkpoint the native boolean logical not lowering
  slice now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 242: Native Static Strict Identity Lowering Slice

- [x] Lower static straight-line strict identity `===` and `!==` for already
  lowerable integer literals and booleans through `phpc compile --emit-ir` and
  `--emit-asm`, including deterministic LLVM IR and C fallback assembly CLI
  coverage, fixture coverage through `phpc run` and system PHP comparison,
  updated diagnostics for unsupported comparison operands/operators,
  documentation, and named gaps for loose comparisons, ordering comparisons,
  strings, floats, nulls, arrays, objects, dynamic integer expression results,
  PHP comparison coercions, references/copy-on-write, exact native PHP errors,
  linking/execution, and broader native lowering.

## Milestone 243: Native Static Strict Identity Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native static strict
  identity lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 244: Native Static Strict Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native static strict identity lowering
  slice now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 245: Native Static String Concatenation Lowering Slice

- [x] Lower static straight-line string concatenation `.` for already lowerable
  string operands through `phpc compile --emit-ir` and `--emit-asm`, including
  deterministic LLVM IR and C fallback assembly CLI coverage, fixture coverage
  through `phpc run` and system PHP comparison, updated diagnostics for
  unsupported concatenation operands/operators, documentation, and named gaps
  for PHP scalar-to-string conversion, arrays, objects, resources,
  references/copy-on-write, exact native PHP errors, linking/execution, and
  broader native lowering.

## Milestone 246: Native Static String Concatenation Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native static string
  concatenation lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 247: Native Static String Concatenation Checkpoint Decision

- [x] Decide whether to checkpoint the native static string concatenation
  lowering slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 248: Native Static String Strict Identity Lowering Slice

- [x] Extend native strict identity `===` and `!==` lowering to already
  lowerable string operands through `phpc compile --emit-ir` and `--emit-asm`,
  including deterministic LLVM IR and C fallback assembly CLI coverage, fixture
  coverage through `phpc run` and system PHP comparison, documentation, and
  named gaps for loose comparisons, ordering comparisons, floats, nulls,
  arrays, objects, dynamic string allocation beyond the static straight-line
  subset, PHP comparison coercions, references/copy-on-write, exact native PHP
  errors, linking/execution, and broader native lowering.

## Milestone 249: Native Static String Strict Identity Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native static string
  strict-identity lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 250: Native Static String Strict Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native static string strict-identity
  lowering slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 251: Native Static Float Strict Identity Lowering Slice

- [x] Extend native strict identity `===` and `!==` lowering to already
  lowerable float operands through `phpc compile --emit-ir` and `--emit-asm`,
  including deterministic LLVM IR and C fallback assembly CLI coverage, fixture
  coverage through `phpc run` and system PHP comparison, documentation, and
  named gaps for loose comparisons, ordering comparisons, nulls, arrays,
  objects, mixed int/float identity semantics beyond static rejection, NaN and
  non-literal float sources, PHP comparison coercions, references/copy-on-write,
  exact native PHP errors, linking/execution, and broader native lowering.

## Milestone 252: Native Static Float Strict Identity Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native static float
  strict-identity lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 253: Native Static Float Strict Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native static float strict-identity
  lowering slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 254: Native Static Null Strict Identity Lowering Slice

- [x] Extend native strict identity `===` and `!==` lowering to already
  lowerable `null` operands through `phpc compile --emit-ir` and `--emit-asm`,
  including deterministic LLVM IR and C fallback assembly CLI coverage, fixture
  coverage through `phpc run` and system PHP comparison, documentation, and
  named gaps for mixed null/scalar identity semantics beyond static rejection,
  loose comparisons, ordering comparisons, arrays, objects, PHP comparison
  coercions, references/copy-on-write, exact native PHP errors,
  linking/execution, and broader native lowering.

## Milestone 255: Native Static Null Strict Identity Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native static null
  strict-identity lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 256: Native Static Null Strict Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native static null strict-identity
  lowering slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 257: Native Mixed Scalar Strict Identity Lowering Slice

- [x] Extend native strict identity `===` and `!==` lowering for already
  lowerable scalar operands with different PHP scalar types, including dynamic
  integer expression results where the type alone determines identity, through
  `phpc compile --emit-ir` and `--emit-asm`; include deterministic LLVM IR and
  C fallback assembly CLI coverage, fixture coverage through `phpc run` and
  system PHP comparison, documentation, and named gaps for same-type dynamic
  integer identity, loose comparisons, ordering comparisons, arrays, objects,
  PHP comparison coercions, references/copy-on-write, exact native PHP errors,
  linking/execution, and broader native lowering.

## Milestone 258: Native Mixed Scalar Strict Identity Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native mixed scalar
  strict-identity lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 259: Native Mixed Scalar Strict Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native mixed scalar strict-identity
  lowering slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 260: Native Dynamic Integer Strict Identity Lowering Slice

- [x] Extend native strict identity `===` and `!==` lowering for same-type
  dynamic integer operands in the straight-line subset, including integer
  expression results compared with integer literals or previously assigned
  integer expressions, through `phpc compile --emit-ir` and `--emit-asm`;
  include deterministic LLVM IR and C fallback assembly CLI coverage, fixture
  coverage through `phpc run` and system PHP comparison, documentation, and
  named gaps for floats, strings, booleans, nulls beyond already static or
  type-only folds, loose comparisons, ordering comparisons, arrays, objects,
  PHP comparison coercions, references/copy-on-write, exact native PHP errors,
  linking/execution, and broader native lowering.

## Milestone 261: Native Dynamic Integer Strict Identity Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native dynamic integer
  strict-identity lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 262: Native Dynamic Integer Strict Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native dynamic integer strict-identity
  lowering slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 263: Native Dynamic Boolean Strict Identity Lowering Slice

- [x] Extend native strict identity `===` and `!==` lowering for same-type
  dynamic boolean operands in the straight-line subset, including boolean
  expression results compared with boolean literals or previously assigned
  boolean expressions, through `phpc compile --emit-ir` and `--emit-asm`;
  include deterministic LLVM IR and C fallback assembly CLI coverage, fixture
  coverage through `phpc run` and system PHP comparison, documentation, and
  named gaps for PHP truthiness conversion, logical operator lowering,
  dynamic floats, strings, and nulls beyond already static or type-only folds,
  loose comparisons, ordering comparisons, arrays, objects, PHP comparison
  coercions, references/copy-on-write, exact native PHP errors,
  linking/execution, and broader native lowering.

## Milestone 264: Native Dynamic Boolean Strict Identity Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native dynamic boolean
  strict-identity lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 265: Native Dynamic Boolean Strict Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native dynamic boolean strict-identity
  lowering slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 266: Native Mixed Dynamic Boolean Strict Identity Coverage Slice

- [x] Add explicit native strict identity `===` and `!==` coverage for dynamic
  boolean operands compared with different scalar types, proving the
  straight-line type-only fold through `phpc compile --emit-ir` and
  `--emit-asm`; include deterministic LLVM IR and C fallback assembly CLI
  coverage, fixture coverage through `phpc run` and system PHP comparison,
  documentation, and named gaps for PHP truthiness conversion, logical
  operator lowering, dynamic floats, strings, and nulls beyond already static
  or type-only folds, loose comparisons, ordering comparisons, arrays, objects,
  PHP comparison coercions, references/copy-on-write, exact native PHP errors,
  linking/execution, and broader native lowering.

## Milestone 267: Native Mixed Dynamic Boolean Strict Identity Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native mixed dynamic
  boolean strict-identity coverage slice, fix any failures, and document the
  result in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under
  unattended loop automation.

## Milestone 268: Native Mixed Dynamic Boolean Strict Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native mixed dynamic boolean
  strict-identity coverage slice now or continue into the next smallest
  native-lowering slice. If checkpointing, use `tools/checkpoint.sh` and
  include the focused and full-suite test results in the checkpoint message.

## Milestone 269: Native Dynamic Boolean Logical Not Lowering Slice

- [x] Extend native logical not `!` lowering for already-native dynamic boolean
  expression operands in the straight-line subset, including boolean results
  produced by strict-identity lowering, through `phpc compile --emit-ir` and
  `--emit-asm`; include deterministic LLVM IR and C fallback assembly CLI
  coverage, fixture coverage through `phpc run` and system PHP comparison,
  documentation, and named gaps for PHP truthiness conversion, logical
  operator lowering, dynamic floats, strings, and nulls, arrays, objects,
  references/copy-on-write, exact native PHP errors, linking/execution, and
  broader native lowering.

## Milestone 270: Native Dynamic Boolean Logical Not Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native dynamic boolean
  logical-not lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 271: Native Dynamic Boolean Logical Not Checkpoint Decision

- [x] Decide whether to checkpoint the native dynamic boolean logical-not
  lowering slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 272: Native Boolean Logical Operator Lowering Slice

- [x] Extend native logical operator lowering for already lowerable boolean
  operands in the straight-line subset, including static booleans and native
  boolean expression results for `&&`, `||`, `and`, `or`, and `xor`, through
  `phpc compile --emit-ir` and `--emit-asm`; include deterministic LLVM IR and
  C fallback assembly CLI coverage, fixture coverage through `phpc run` and
  system PHP comparison, documentation, and named gaps for PHP truthiness
  conversion, short-circuiting with unsupported or side-effecting right-hand
  operands, dynamic floats, strings, and nulls, arrays, objects,
  references/copy-on-write, exact native PHP errors, linking/execution, and
  broader native lowering.

## Milestone 273: Native Boolean Logical Operator Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native boolean logical
  operator lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 274: Native Boolean Logical Operator Checkpoint Decision

- [x] Decide whether to checkpoint the native boolean logical-operator lowering
  slice now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 275: Native Integer Bitwise Lowering Slice

- [x] Lower integer bitwise `&`, `|`, `^`, and unary `~` for operands that are
  already lowerable integers in the straight-line subset through `phpc compile
  --emit-ir` and `--emit-asm`; include deterministic LLVM IR and C fallback
  assembly CLI coverage, fixture coverage through `phpc run` and system PHP
  comparison, documentation, and named gaps for PHP bytewise string bitwise
  behavior, scalar-to-int coercion for non-integer operands, shift operators
  and shift diagnostics, arrays, objects, references/copy-on-write, exact
  native PHP errors, linking/execution, and broader native lowering.

## Milestone 276: Native Integer Bitwise Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native integer bitwise
  lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 277: Native Integer Bitwise Checkpoint Decision

- [x] Decide whether to checkpoint the native integer bitwise lowering slice
  now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 278: Native Integer Shift Lowering Slice

- [x] Lower integer shift `<<` and `>>` for operands that are already lowerable
  integers in the straight-line subset when the shift count is statically known
  and in the range 0 through 63, through `phpc compile --emit-ir` and
  `--emit-asm`; include deterministic LLVM IR and C fallback assembly CLI
  coverage, fixture coverage through `phpc run` and system PHP comparison,
  documentation, and named gaps for dynamic shift counts, negative and large
  shift-count diagnostics, PHP bytewise string bitwise behavior, scalar-to-int
  coercion for non-integer operands, arrays, objects, references/copy-on-write,
  exact native PHP errors, linking/execution, and broader native lowering.

## Milestone 279: Native Integer Shift Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native integer shift
  lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 280: Native Integer Shift Checkpoint Decision

- [x] Decide whether to checkpoint the native integer shift lowering slice now
  or continue into the next smallest native-lowering slice. If checkpointing,
  use `tools/checkpoint.sh` and include the focused and full-suite test results
  in the checkpoint message.

## Milestone 281: Native Boolean Ternary Lowering Slice

- [x] Lower full ternary `condition ? if_true : if_false` when the condition is
  already a lowerable boolean or native boolean expression and both branch
  values are already lowerable integers or booleans in the straight-line
  subset, through `phpc compile --emit-ir` and `--emit-asm`; include
  deterministic LLVM IR and C fallback assembly CLI coverage, fixture coverage
  through `phpc run` and system PHP comparison, documentation, and named gaps
  for PHP truthiness conversion, lazy branch evaluation for unsupported or
  side-effecting branches, short ternary, null coalescing, arrays, objects,
  references/copy-on-write, exact native PHP errors, linking/execution, and
  broader native lowering.

## Milestone 282: Native Boolean Ternary Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native boolean ternary
  lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 283: Native Boolean Ternary Checkpoint Decision

- [x] Decide whether to checkpoint the native boolean ternary lowering slice
  now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 284: Native Float Ternary Lowering Slice

- [x] Extend full ternary `condition ? if_true : if_false` native lowering to
  branch values that are already lowerable floats in the straight-line subset,
  through `phpc compile --emit-ir` and `--emit-asm`; include deterministic LLVM
  IR and C fallback assembly CLI coverage, fixture coverage through `phpc run`
  and system PHP comparison, documentation, and named gaps for PHP truthiness
  conversion, lazy branch evaluation for unsupported or side-effecting
  branches, short ternary, null coalescing, arrays, objects,
  references/copy-on-write, exact native PHP errors, linking/execution, and
  broader native lowering.

## Milestone 285: Native Float Ternary Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native float ternary
  lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 286: Native Float Ternary Checkpoint Decision

- [x] Decide whether to checkpoint the native float ternary lowering slice now
  or continue into the next smallest native-lowering slice. If checkpointing,
  use `tools/checkpoint.sh` and include the focused and full-suite test results
  in the checkpoint message.

## Milestone 287: Native String Ternary Lowering Slice

- [x] Extend full ternary `condition ? if_true : if_false` native lowering to
  branch values that are already lowerable strings in the straight-line subset,
  through `phpc compile --emit-ir` and `--emit-asm`; include deterministic LLVM
  IR and C fallback assembly CLI coverage, fixture coverage through `phpc run`
  and system PHP comparison, documentation, and named gaps for PHP truthiness
  conversion, lazy branch evaluation for unsupported or side-effecting
  branches, short ternary, null coalescing, arrays, objects,
  references/copy-on-write, runtime string allocation beyond the static string
  subset, exact native PHP errors, linking/execution, and broader native
  lowering.

## Milestone 288: Native String Ternary Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native string ternary
  lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 289: Native String Ternary Checkpoint Decision

- [x] Decide whether to checkpoint the native string ternary lowering slice now
  or continue into the next smallest native-lowering slice. If checkpointing,
  use `tools/checkpoint.sh` and include the focused and full-suite test results
  in the checkpoint message.

## Milestone 290: Native Static Boolean Mixed Ternary Folding Slice

- [x] Fold full ternary `condition ? if_true : if_false` when the condition is
  a statically known boolean and both branch values are already lowerable scalar
  values, including mixed selected/unselected branch types, through
  `phpc compile --emit-ir` and `--emit-asm`; include deterministic LLVM IR and
  C fallback assembly CLI coverage, fixture coverage through `phpc run` and
  system PHP comparison, documentation, and named gaps for dynamic mixed-type
  ternaries, PHP truthiness conversion, lazy branch evaluation for unsupported
  or side-effecting branches, short ternary, null coalescing, arrays, objects,
  references/copy-on-write, exact native PHP errors, linking/execution, and
  broader native lowering.

## Milestone 291: Native Static Boolean Mixed Ternary Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native static boolean
  mixed ternary folding slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 292: Native Static Boolean Mixed Ternary Checkpoint Decision

- [x] Decide whether to checkpoint the native static boolean mixed ternary
  folding slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 293: Native Float Arithmetic Lowering Slice

- [x] Lower static straight-line float `+`, `-`, and `*` expressions when both
  operands are already lowerable floats through `phpc compile --emit-ir` and
  `--emit-asm`; include deterministic LLVM IR and C fallback assembly CLI
  coverage, fixture coverage through `phpc run` and system PHP comparison,
  documentation, and named gaps for mixed int/float arithmetic, PHP numeric
  coercion, `/`, `%`, division/modulo zero checks, modulo coercions, overflow
  behavior, references/copy-on-write, exact native PHP errors,
  linking/execution, and broader native lowering.

## Milestone 294: Native Float Arithmetic Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native float arithmetic
  lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 295: Native Float Arithmetic Checkpoint Decision

- [x] Decide whether to checkpoint the native float arithmetic lowering slice
  now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 296: Native Float Unary Minus Lowering Slice

- [x] Lower static straight-line float unary minus through `phpc compile
  --emit-ir` and `--emit-asm`; include deterministic LLVM IR and C fallback
  assembly CLI coverage, fixture coverage through `phpc run` and system PHP
  comparison, documentation, and named gaps for boolean/string/null unary-minus
  coercion, arrays, objects, overflow behavior, references/copy-on-write, exact
  native PHP errors, linking/execution, and broader native lowering.

## Milestone 297: Native Float Unary Minus Full-Suite Gate

- [x] Run `tools/run-tests.sh` before checkpointing the native float unary
  minus lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 298: Native Float Unary Minus Checkpoint Decision

- [x] Decide whether to checkpoint the native float unary minus lowering slice
  now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 299: Native Dynamic Float Strict Identity Lowering Slice

- [x] Extend native strict identity `===` and `!==` lowering for same-type
  dynamic float operands in the straight-line subset. Preserve static float
  identity folding, emit LLVM `fcmp` and C fallback comparisons for dynamic
  lowerable float expressions, keep loose/order comparison and non-lowerable
  operand rejection explicit, and add fixture, CLI, assembly fallback, docs,
  and focused verification.

## Milestone 300: Native Dynamic Float Strict Identity Full-Suite Gate

- [x] Run the full project test gate for the native dynamic float strict
  identity lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 301: Native Dynamic Float Strict Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native dynamic float strict identity
  lowering slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 302: Native Mixed Dynamic Float Strict Identity Coverage

- [x] Add explicit native strict identity `===` and `!==` coverage for dynamic
  float expression results compared with different scalar types. Prove the
  type-only fold keeps the lowerable float expression boundary honest, emits no
  runtime comparison or numeric output, and preserves the existing loose/order
  comparison and non-lowerable operand rejection boundaries.

## Milestone 303: Native Mixed Dynamic Float Strict Identity Full-Suite Gate

- [x] Run the full project test gate for the native mixed dynamic float strict
  identity coverage slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 304: Native Mixed Dynamic Float Strict Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native mixed dynamic float strict
  identity coverage slice now or continue into the next smallest
  native-lowering slice. If checkpointing, use `tools/checkpoint.sh` and
  include the focused and full-suite test results in the checkpoint message.

## Milestone 305: Native Dynamic String Strict Identity Lowering Slice

- [x] Extend native strict identity `===` and `!==` lowering for same-type
  dynamic string pointer operands in the straight-line subset. Use `strcmp` for
  already lowerable string pointers, keep runtime string allocation and dynamic
  null identity unsupported, and add fixture, CLI, assembly fallback, docs, and
  focused verification.

## Milestone 306: Native Dynamic String Strict Identity Full-Suite Gate

- [x] Run the full project test gate for the native dynamic string strict
  identity lowering slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 307: Native Dynamic String Strict Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native dynamic string strict identity
  lowering slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 308: Native Mixed Dynamic String Strict Identity Coverage

- [x] Add explicit native strict identity `===` and `!==` coverage for dynamic
  string pointer expression results compared with different scalar types. Prove
  the type-only fold keeps the lowerable string pointer boundary honest, emits
  no runtime string comparison or numeric output, and preserves the existing
  loose/order comparison and unsupported operand boundaries.

## Milestone 309: Native Mixed Dynamic String Strict Identity Full-Suite Gate

- [x] Run the full project test gate for the native mixed dynamic string strict
  identity coverage slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 310: Native Mixed Dynamic String Strict Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native mixed dynamic string strict
  identity coverage slice now or continue into the next smallest
  native-lowering slice. If checkpointing, use `tools/checkpoint.sh` and
  include the focused and full-suite test results in the checkpoint message.

## Milestone 311: Native Null Ternary Folding Slice

- [x] Extend native full ternary lowering for dynamic boolean conditions when
  both branches are `null`. Fold the result to `null` without emitting a native
  select, branch, numeric output, or tagged value, and keep mixed dynamic branch
  values, null coalescing, PHP truthiness, and side-effecting branch laziness
  unsupported.

## Milestone 312: Native Null Ternary Full-Suite Gate

- [x] Run the full project test gate for the native null ternary folding slice,
  fix any failures, and document the result in `docs/PROGRESS.md` and
  `docs/LOOP_MEMORY.md` if running under unattended loop automation.

## Milestone 313: Native Null Ternary Checkpoint Decision

- [x] Decide whether to checkpoint the native null ternary folding slice now or
  continue into the next smallest native-lowering slice. If checkpointing, use
  `tools/checkpoint.sh` and include the focused and full-suite test results in
  the checkpoint message.

## Milestone 314: Native Integer Modulo Lowering Slice

- [x] Extend native arithmetic lowering for integer `%` when both operands are
  already lowerable integers and the divisor is a statically known positive
  integer. Emit LLVM `srem` and C `%`, keep division, dynamic/zero/non-positive
  divisors, PHP numeric coercion, negative-divisor/min-int edge cases, and
  runtime modulo diagnostics unsupported, and add fixture, CLI, assembly
  fallback, docs, and focused verification.

## Milestone 315: Native Integer Modulo Full-Suite Gate

- [x] Run the full project test gate for the native integer modulo lowering
  slice, fix any failures, and document the result in `docs/PROGRESS.md` and
  `docs/LOOP_MEMORY.md` if running under unattended loop automation.

## Milestone 316: Native Integer Modulo Checkpoint Decision

- [x] Decide whether to checkpoint the native integer modulo lowering slice now
  or continue into the next smallest native-lowering slice. If checkpointing,
  use `tools/checkpoint.sh` and include the focused and full-suite test results
  in the checkpoint message.

## Milestone 317: Native Modulo Runtime-Check Boundary Slice

- [x] Add the next honest native-codegen boundary or executable slice for
  modulo cases that need runtime checks, such as dynamic integer divisors or
  non-positive static divisors. Keep PHP numeric coercions, modulo coercions,
  negative-divisor/min-int edge cases, references/copy-on-write, exact native
  PHP errors, and broader native arithmetic lowering explicitly unsupported
  unless executable code, fixtures, CLI coverage, docs, and focused tests prove
  the narrower behavior.

## Milestone 318: Native Modulo Runtime-Check Boundary Full-Suite Gate

- [x] Run the full project test gate for the native modulo runtime-check
  boundary slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 319: Native Modulo Runtime-Check Boundary Checkpoint Decision

- [x] Decide whether to checkpoint the native modulo runtime-check boundary
  slice now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 320: Native Integer Division Boundary Slice

- [x] Add the next honest native-codegen boundary or executable slice for
  integer division `/`, either by tightening diagnostics and CLI fixture
  coverage for runtime-check/coercion gaps or by lowering a narrow safe subset
  only if division-by-zero, truncation, overflow, PHP numeric coercions,
  references/copy-on-write, exact native PHP errors, and broader native
  arithmetic behavior remain explicitly unsupported unless executable code,
  fixtures, CLI coverage, docs, and focused tests prove otherwise.

## Milestone 321: Native Integer Division Boundary Full-Suite Gate

- [x] Run the full project test gate for the native integer division boundary
  slice, fix any failures, and document the result in `docs/PROGRESS.md` and
  `docs/LOOP_MEMORY.md` if running under unattended loop automation.

## Milestone 322: Native Integer Division Boundary Checkpoint Decision

- [x] Decide whether to checkpoint the native integer division boundary slice
  now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 323: Native Mixed Numeric Arithmetic Boundary Slice

- [x] Add the next honest native-codegen boundary or executable slice for
  mixed int/float arithmetic, either by tightening diagnostics and CLI fixture
  coverage for PHP numeric-coercion gaps or by lowering a narrow safe subset
  only if PHP result types, overflow/INF/NAN behavior, scalar coercions,
  references/copy-on-write, exact native PHP errors, and broader native
  arithmetic behavior remain explicitly unsupported unless executable code,
  fixtures, CLI coverage, docs, and focused tests prove otherwise.

## Milestone 324: Native Mixed Numeric Arithmetic Boundary Full-Suite Gate

- [x] Run the full project test gate for the native mixed numeric arithmetic
  boundary slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 325: Native Mixed Numeric Arithmetic Boundary Checkpoint Decision

- [x] Decide whether to checkpoint the native mixed numeric arithmetic boundary
  slice now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 326: Native Scalar Coercion Arithmetic Boundary Slice

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-coercion arithmetic operands such as booleans, nulls, or numeric
  strings in `+`, `-`, and `*`, either by tightening diagnostics and CLI
  fixture coverage for PHP numeric-coercion gaps or by lowering a narrow safe
  subset only if PHP result types, string numeric parsing, warnings/recovery,
  overflow/INF/NAN behavior, references/copy-on-write, exact native PHP errors,
  and broader native arithmetic behavior remain explicitly unsupported unless
  executable code, fixtures, CLI coverage, docs, and focused tests prove
  otherwise.

## Milestone 327: Native Scalar Coercion Arithmetic Boundary Full-Suite Gate

- [x] Run the full project test gate for the native scalar-coercion arithmetic
  boundary slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 328: Native Scalar Coercion Arithmetic Boundary Checkpoint Decision

- [x] Decide whether to checkpoint the native scalar-coercion arithmetic
  boundary slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 329: Native Integer Overflow Arithmetic Boundary Slice

- [x] Add the next honest native-codegen boundary or executable slice for
  integer overflow behavior in native `+`, `-`, and `*`, either by tightening
  diagnostics and CLI fixture coverage for overflow-sensitive cases or by
  lowering a narrow checked subset only if PHP integer/float promotion,
  overflow diagnostics or conversion behavior, references/copy-on-write, exact
  native PHP errors, and broader native arithmetic behavior remain explicitly
  unsupported unless executable code, fixtures, CLI coverage, docs, and focused
  tests prove otherwise.

## Milestone 330: Native Integer Overflow Arithmetic Full-Suite Gate

- [x] Run the full project test gate for the native integer-overflow arithmetic
  boundary slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 331: Native Integer Overflow Arithmetic Checkpoint Decision

- [x] Decide whether to checkpoint the native integer-overflow arithmetic
  boundary slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 332: Native Integer Arithmetic Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  integer-producing expressions that are currently not statically tracked after
  lowering, such as unary minus, bitwise operators, shifts, or integer
  ternaries. Either preserve safe known-result tracking where executable code,
  fixtures, CLI coverage, docs, and focused tests prove it, or reject the
  not-statically-proven cases with a specific diagnostic until native PHP
  overflow behavior, runtime checks, references/copy-on-write, exact native
  PHP errors, and broader native expression lowering exist.

## Milestone 333: Native Integer Unary-Minus Result Tracking Full-Suite Gate

- [x] Run the full project test gate for the native integer unary-minus
  result-tracking slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 334: Native Integer Unary-Minus Result Tracking Checkpoint Decision

- [x] Decide whether to checkpoint the native integer unary-minus
  result-tracking slice now or continue into the next smallest native-lowering
  slice. If checkpointing, use `tools/checkpoint.sh` and include the focused
  and full-suite test results in the checkpoint message.

## Milestone 335: Native Integer Bitwise Result Tracking Slice

- [x] Add the next honest native-codegen boundary or executable slice for
  integer bitwise results used by later checked integer arithmetic. Preserve
  safe known-result tracking for lowerable integer `&`, `|`, `^`, and `~`
  where executable code, fixtures, CLI coverage, docs, and focused tests prove
  it, or reject not-statically-proven cases with a specific diagnostic until
  native PHP bitwise coercions, overflow behavior, runtime checks,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering exist.

## Milestone 336: Native Integer Bitwise Result Tracking Full-Suite Gate

- [x] Run the full project test gate for the native integer bitwise
  result-tracking slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 337: Native Integer Bitwise Result Tracking Checkpoint Decision

- [x] Decide whether to checkpoint the native integer bitwise result-tracking
  slice now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 338: Native Integer Shift Result Tracking Slice

- [x] Add the next honest native-codegen boundary or executable slice for
  integer shift results used by later checked integer arithmetic. Preserve safe
  known-result tracking for lowerable integer `<<` and `>>` with statically
  known safe counts where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or reject not-statically-proven cases with a specific
  diagnostic until native PHP shift coercions, shift diagnostics, overflow
  behavior, runtime checks, references/copy-on-write, exact native PHP errors,
  and broader native expression lowering exist.

## Milestone 339: Native Integer Shift Result Tracking Full-Suite Gate

- [x] Run the full project test gate for the native integer shift
  result-tracking slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 340: Native Integer Shift Result Tracking Checkpoint Decision

- [x] Decide whether to checkpoint the native integer shift result-tracking
  slice now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 341: Native Integer Ternary Result Tracking Slice

- [x] Add the next honest native-codegen boundary or executable slice for
  integer ternary results used by later checked integer arithmetic. Preserve
  safe known-result tracking for lowerable integer full-ternary branches when
  both branch values are statically known and checked, or reject
  not-statically-proven cases with a specific diagnostic until native PHP
  truthiness, branch laziness with side effects, overflow behavior, runtime
  checks, references/copy-on-write, exact native PHP errors, and broader native
  expression lowering exist.

## Milestone 342: Native Integer Ternary Result Tracking Full-Suite Gate

- [x] Run the full project test gate for the native integer ternary
  result-tracking slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 343: Native Integer Ternary Result Tracking Checkpoint Decision

- [x] Decide whether to checkpoint the native integer ternary result-tracking
  slice now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 344: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 345: Native Integer Modulo Result Tracking Full-Suite Gate

- [x] Run the full project test gate for the native integer modulo
  result-tracking slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 346: Native Integer Modulo Result Tracking Checkpoint Decision

- [x] Decide whether to checkpoint the native integer modulo result-tracking
  slice now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 347: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 348: Native Bounded Integer Result Tracking Full-Suite Gate

- [x] Run the full project test gate for the native bounded integer
  result-tracking slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 349: Native Bounded Integer Result Tracking Checkpoint Decision

- [x] Decide whether to checkpoint the native bounded integer result-tracking
  slice now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 350: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 351: Native Bounded Integer Bitwise/Shift Result Tracking Full-Suite Gate

- [x] Run the full project test gate for the native bounded integer
  bitwise/shift result-tracking slice, fix any failures, and document the
  result in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under
  unattended loop automation.

## Milestone 352: Native Bounded Integer Bitwise/Shift Result Tracking Checkpoint Decision

- [x] Decide whether to checkpoint the native bounded integer bitwise/shift
  result-tracking slice now or continue into the next smallest native-lowering
  slice. If checkpointing, use `tools/checkpoint.sh` and include the focused
  and full-suite test results in the checkpoint message.

## Milestone 353: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 354: Native Bounded Integer Strict-Identity Fold Full-Suite Gate

- [x] Run the full project test gate for the native bounded integer
  strict-identity folding slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 355: Native Bounded Integer Strict-Identity Fold Checkpoint Decision

- [x] Decide whether to checkpoint the native bounded integer strict-identity
  folding slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 356: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 357: Native Boolean Logical Folding Full-Suite Gate

- [x] Run the full project test gate for the native boolean logical folding
  slice, fix any failures, and document the result in `docs/PROGRESS.md` and
  `docs/LOOP_MEMORY.md` if running under unattended loop automation.

## Milestone 358: Native Boolean Logical Folding Checkpoint Decision

- [x] Decide whether to checkpoint the native boolean logical folding slice
  now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 359: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 360: Native Bounded String Strict-Identity Fold Full-Suite Gate

- [x] Run the full project test gate for the native bounded string
  strict-identity folding slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 361: Native Bounded String Strict-Identity Fold Checkpoint Decision

- [x] Decide whether to checkpoint the native bounded string strict-identity
  folding slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 362: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 363: Native Bounded Float Strict-Identity Fold Full-Suite Gate

- [x] Run the full project test gate for the native bounded float
  strict-identity folding slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 364: Native Bounded Float Strict-Identity Fold Checkpoint Decision

- [x] Decide whether to checkpoint the native bounded float strict-identity
  folding slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 365: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 366: Native Bounded Boolean Strict-Identity Fold Full-Suite Gate

- [x] Run the full project test gate for the native bounded boolean
  strict-identity folding slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 367: Native Bounded Boolean Strict-Identity Fold Checkpoint Decision

- [x] Decide whether to checkpoint the native bounded boolean strict-identity
  folding slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 368: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 369: Native Bounded Float Arithmetic Result Tracking Full-Suite Gate

- [x] Run the full project test gate for the native bounded float arithmetic
  result-tracking slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 370: Native Bounded Float Arithmetic Result Tracking Checkpoint Decision

- [x] Decide whether to checkpoint the native bounded float arithmetic
  result-tracking slice now or continue into the next smallest native-lowering
  slice. If checkpointing, use `tools/checkpoint.sh` and include the focused
  and full-suite test results in the checkpoint message.

## Milestone 371: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 372: Native Reflexive Scalar Strict-Identity Full-Suite Gate

- [x] Run the full project test gate for the native reflexive scalar
  strict-identity folding slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 373: Native Reflexive Scalar Strict-Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native reflexive scalar strict-identity
  folding slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 374: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 375: Native Integer Strict-Identity Result Tracking Full-Suite Gate

- [x] Run the full project test gate for the native integer strict-identity
  result-tracking slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 376: Native Integer Strict-Identity Result Tracking Checkpoint Decision

- [x] Decide whether to checkpoint the native integer strict-identity
  result-tracking slice now or continue into the next smallest native-lowering
  slice. If checkpointing, use `tools/checkpoint.sh` and include the focused
  and full-suite test results in the checkpoint message.

## Milestone 377: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 378: Native Integer Comparison Result Tracking Full-Suite Gate

- [x] Run the full project test gate for the native integer loose/ordering
  comparison result-tracking slice, fix any failures, and document the result
  in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 379: Native Integer Comparison Result Tracking Checkpoint Decision

- [x] Decide whether to checkpoint the native integer loose/ordering
  comparison result-tracking slice now or continue into the next smallest
  native-lowering slice. If checkpointing, use `tools/checkpoint.sh` and
  include the focused and full-suite test results in the checkpoint message.

## Milestone 380: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 381: Native Float Comparison Result Tracking Full-Suite Gate

- [x] Run the full project test gate for the native finite float loose/ordering
  comparison result-tracking slice, fix any failures, and document the result
  in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 382: Native Float Comparison Result Tracking Checkpoint Decision

- [x] Decide whether to checkpoint the native finite float loose/ordering
  comparison result-tracking slice now or continue into the next smallest
  native-lowering slice. If checkpointing, use `tools/checkpoint.sh` and
  include the focused and full-suite test results in the checkpoint message.

## Milestone 383: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 384: Native Boolean Comparison Result Tracking Full-Suite Gate

- [x] Run the full project test gate for the native boolean loose/ordering
  comparison result-tracking slice, fix any failures, and document the result
  in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 385: Native Boolean Comparison Result Tracking Checkpoint Decision

- [x] Decide whether to checkpoint the native boolean loose/ordering comparison
  result-tracking slice now or continue into the next smallest native-lowering
  slice. If checkpointing, use `tools/checkpoint.sh` and include the focused
  and full-suite test results in the checkpoint message.

## Milestone 386: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 387: Native String Comparison Result Tracking Full-Suite Gate

- [x] Run the full project test gate for the native known ASCII nonnumeric
  string loose/ordering comparison result-tracking slice, fix any failures, and
  document the result in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if
  running under unattended loop automation.

## Milestone 388: Native String Comparison Result Tracking Checkpoint Decision

- [x] Decide whether to checkpoint the native known ASCII nonnumeric string
  loose/ordering comparison result-tracking slice now or continue into the
  next smallest native-lowering slice. If checkpointing, use
  `tools/checkpoint.sh` and include the focused and full-suite test results in
  the checkpoint message.

## Milestone 389: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 390: Native Null Comparison Boundary Full-Suite Gate

- [x] Run the full project test gate for the native same-type null
  loose/ordering comparison slice, fix any failures, and document the result
  in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 391: Native Null Comparison Boundary Checkpoint Decision

- [x] Decide whether to checkpoint the native same-type null loose/ordering
  comparison slice now or continue into the next smallest native-lowering
  slice. If checkpointing, use `tools/checkpoint.sh` and include the focused
  and full-suite test results in the checkpoint message.

## Milestone 392: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 393: Native Broader ASCII String Comparison Full-Suite Gate

- [x] Run the full project test gate for the native known ASCII nonnumeric
  NUL-free string comparison boundary expansion, fix any failures, and document
  the result in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under
  unattended loop automation.

## Milestone 394: Native Broader ASCII String Comparison Checkpoint Decision

- [x] Decide whether to checkpoint the native known ASCII nonnumeric NUL-free
  string comparison boundary expansion now or continue into the next smallest
  native-lowering slice. If checkpointing, use `tools/checkpoint.sh` and
  include the focused and full-suite test results in the checkpoint message.

## Milestone 395: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 396: Native Identical String Ternary Full-Suite Gate

- [x] Run the full project test gate for the native identical string ternary
  folding slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 397: Native Identical String Ternary Checkpoint Decision

- [x] Decide whether to checkpoint the native identical string ternary folding
  slice now or continue into the next smallest native-lowering slice. If
  checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 398: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 399: Native Identical Boolean Expression Ternary Full-Suite Gate

- [x] Run the full project test gate for the native identical boolean
  expression ternary folding slice, fix any failures, and document the result
  in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 400: Native Identical Boolean Expression Ternary Checkpoint Decision

- [x] Decide whether to checkpoint the native identical boolean expression
  ternary folding slice now or continue into the next smallest native-lowering
  slice. If checkpointing, use `tools/checkpoint.sh` and include the focused
  and full-suite test results in the checkpoint message.

## Milestone 401: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 402: Native Identical Integer Expression Ternary Full-Suite Gate

- [x] Run the full project test gate for the native identical integer
  expression ternary folding slice, fix any failures, and document the result
  in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 403: Native Identical Integer Expression Ternary Checkpoint Decision

- [x] Decide whether to checkpoint the native identical integer expression
  ternary folding slice now or continue into the next smallest native-lowering
  slice. If checkpointing, use `tools/checkpoint.sh` and include the focused
  and full-suite test results in the checkpoint message.

## Milestone 404: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 405: Native Identical Float Expression Ternary Full-Suite Gate

- [x] Run the full project test gate for the native identical float expression
  ternary folding slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 406: Native Identical Float Expression Ternary Checkpoint Decision

- [x] Decide whether to checkpoint the native identical float expression
  ternary folding slice now or continue into the next smallest native-lowering
  slice. If checkpointing, use `tools/checkpoint.sh` and include the focused
  and full-suite test results in the checkpoint message.

## Milestone 407: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 408: Native Identical Boolean Expression Logical Full-Suite Gate

- [x] Run the full project test gate for the native identical boolean
  expression logical `&&`/`||` folding slice, fix any failures, and document
  the result in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under
  unattended loop automation.

## Milestone 409: Native Identical Boolean Expression Logical Checkpoint Decision

- [x] Decide whether to checkpoint the native identical boolean expression
  logical folding slice now or continue into the next smallest native-lowering
  slice. If checkpointing, use `tools/checkpoint.sh` and include the focused
  and full-suite test results in the checkpoint message.

## Milestone 410: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 411: Native Identical Boolean Expression Xor Full-Suite Gate

- [x] Run the full project test gate for the native identical boolean
  expression `xor` folding slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 412: Native Identical Boolean Expression Xor Checkpoint Decision

- [x] Decide whether to checkpoint the native identical boolean expression
  `xor` folding slice now or continue into the next smallest native-lowering
  slice. If checkpointing, use `tools/checkpoint.sh` and include the focused
  and full-suite test results in the checkpoint message.

## Milestone 413: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 414: Native Identical Integer Expression Bitwise Full-Suite Gate

- [x] Run the full project test gate for the native identical integer
  expression bitwise folding slice, fix any failures, and document the result
  in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended
  loop automation.

## Milestone 415: Native Identical Integer Expression Bitwise Checkpoint Decision

- [x] Decide whether to checkpoint the native identical integer expression
  bitwise folding slice now or continue into the next smallest native-lowering
  slice. If checkpointing, use `tools/checkpoint.sh` and include the focused
  and full-suite test results in the checkpoint message.

## Milestone 416: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 417: Native Identical Integer Expression Subtraction Full-Suite Gate

- [x] Run the full project test gate for the native identical integer
  expression subtraction folding slice, fix any failures, and document the
  result in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under
  unattended loop automation.

## Milestone 418: Native Identical Integer Expression Subtraction Checkpoint Decision

- [x] Decide whether to checkpoint the native identical integer expression
  subtraction folding slice now or continue into the next smallest
  native-lowering slice. If checkpointing, use `tools/checkpoint.sh` and
  include the focused and full-suite test results in the checkpoint message.

## Milestone 419: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 420: Native Identical Float Expression Subtraction Full-Suite Gate

- [x] Run the full project test gate for the native identical finite float
  expression subtraction folding slice, fix any failures, and document the
  result in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under
  unattended loop automation.

## Milestone 421: Native Identical Float Expression Subtraction Checkpoint Decision

- [x] Decide whether to checkpoint the native identical finite float expression
  subtraction folding slice now or continue into the next smallest
  native-lowering slice. If checkpointing, use `tools/checkpoint.sh` and
  include the focused and full-suite test results in the checkpoint message.

## Milestone 422: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 423: Native Integer Additive Identity Full-Suite Gate

- [x] Run the full project test gate for the native tracked integer additive
  identity folding slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 424: Native Integer Additive Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native tracked integer additive identity
  folding slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 425: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 426: Native Integer Multiplicative Identity Full-Suite Gate

- [x] Run the full project test gate for the native tracked integer
  multiplicative identity folding slice, fix any failures, and document the
  result in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under
  unattended loop automation.

## Milestone 427: Native Integer Multiplicative Identity Checkpoint Decision

- [x] Decide whether to checkpoint the native tracked integer multiplicative
  identity folding slice now or continue into the next smallest native-lowering
  slice. If checkpointing, use `tools/checkpoint.sh` and include the focused
  and full-suite test results in the checkpoint message.

## Milestone 428: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 429: Native Integer Multiplication-By-Zero Full-Suite Gate

- [x] Run the full project test gate for the native tracked integer
  multiplication-by-zero folding slice, fix any failures, and document the
  result in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under
  unattended loop automation.

## Milestone 430: Native Integer Multiplication-By-Zero Checkpoint Decision

- [x] Decide whether to checkpoint the native tracked integer
  multiplication-by-zero folding slice now or continue into the next smallest
  native-lowering slice. If checkpointing, use `tools/checkpoint.sh` and
  include the focused and full-suite test results in the checkpoint message.

## Milestone 431: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 432: Native Integer Shift-By-Zero Full-Suite Gate

- [x] Run the full project test gate for the native tracked integer
  shift-by-zero folding slice, fix any failures, and document the result in
  `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under unattended loop
  automation.

## Milestone 433: Native Integer Shift-By-Zero Checkpoint Decision

- [x] Decide whether to checkpoint the native tracked integer shift-by-zero
  folding slice now or continue into the next smallest native-lowering slice.
  If checkpointing, use `tools/checkpoint.sh` and include the focused and
  full-suite test results in the checkpoint message.

## Milestone 434: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 435: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 436: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 437: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 438: Native Bitwise Identity Batch Full-Suite Gate

- [x] Run the full project test gate for the focused native bitwise identity
  batch after Milestones 435 through 437, fix any failures, and document the
  result in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under
  unattended loop automation.

## Milestone 439: Native Bitwise Identity Batch Checkpoint Decision

- [x] Decide whether to checkpoint the native bitwise identity batch now or
  continue into the next smallest native-lowering slice. If checkpointing, use
  `tools/checkpoint.sh` and include the focused and full-suite test results in
  the checkpoint message.

## Milestone 440: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 441: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 442: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 443: Native Boolean Identity Batch Full-Suite Gate

- [x] Run the full project test gate for the focused native boolean identity
  batch after Milestones 440 through 442, fix any failures, and document the
  result in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under
  unattended loop automation.

## Milestone 444: Native Boolean Identity Batch Checkpoint Decision

- [x] Decide whether to checkpoint the native boolean identity batch now or
  continue into the next smallest native-lowering slice. If checkpointing, use
  `tools/checkpoint.sh` and include the focused and full-suite test results in
  the checkpoint message.

## Milestone 445: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 446: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 447: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 448: Native Boolean Folding Batch Full-Suite Gate

- [x] Run the full project test gate for the focused native boolean folding
  batch after Milestones 445 through 447, fix any failures, and document the
  result in `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` if running under
  unattended loop automation.

## Milestone 449: Native Boolean Folding Batch Checkpoint Decision

- [x] Decide whether to checkpoint the native boolean folding batch now or
  continue into the next smallest native-lowering slice. If checkpointing, use
  `tools/checkpoint.sh` and include the focused and full-suite test results in
  the checkpoint message.

## Milestone 450: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 451: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 452: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 453: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 454: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 455: Native Integer Literal Shift-by-Zero Folding

- [x] Extend native shift-by-zero folding to integer literal operands for the
  current lowerable shift subset. Preserve the existing static shift-count
  boundary, prove the LLVM and C fallback paths with focused tests and CLI
  fixtures, and keep dynamic counts, negative or large counts, PHP scalar
  coercion, string bitwise behavior, references/copy-on-write, exact native PHP
  errors, and broader native expression lowering as named unsupported gaps.

## Milestone 456: Native Single-Known Integer Bitwise-Not Folding

- [x] Fold unary `~` for statically known single-result integer operands in the
  current lowerable bitwise subset. Preserve emitted native bitwise-not
  operations for multi-value bounded operands, prove LLVM and C fallback paths
  with focused tests and CLI fixtures, and keep PHP bytewise string bitwise
  behavior, scalar-to-int coercion, arrays, objects, references/copy-on-write,
  exact native PHP errors, and broader native expression lowering as named
  unsupported gaps.

## Milestone 457: Native Single-Known Integer Unary-Minus Folding

- [x] Fold unary `-` for statically known single-result integer operands in the
  current lowerable unary subset. Preserve emitted native unary-minus
  operations for multi-value bounded operands, keep overflow-sensitive integer
  negation rejected, prove LLVM and C fallback paths with focused tests and CLI
  fixtures, and keep PHP numeric coercion, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering as named
  unsupported gaps.

## Milestone 458: Native Single-Known Float Unary-Minus Folding

- [x] Fold unary `-` for statically known single-result nonzero finite float
  operands in the current lowerable unary subset. Preserve signed-zero,
  overflow/INF/NAN, multi-value bounded operands, PHP numeric coercion,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported or non-folded boundaries, and prove
  LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 459: Native Single-Known Boolean Logical-Not Folding

- [x] Fold LLVM IR logical not over statically known single-result native
  boolean expression operands in the current lowerable unary subset. Keep
  ambiguous boolean expressions and the C assembly fallback's comparison-shaped
  logical-not expressions honest, prove behavior with focused tests and CLI
  fixtures, and keep PHP truthiness coercion, short-circuit side effects,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported gaps.

## Milestone 460: Native C Fallback Boolean Logical-Not Folding

- [x] Extend the C assembly fallback's tracked boolean result handling so
  comparison-shaped integer strict-identity operands can feed single-known
  boolean logical-not folding. Prove the LLVM and C fallback paths with
  focused tests and CLI fixtures, and keep ambiguous boolean expressions, PHP
  truthiness coercion, short-circuit side effects, references/copy-on-write,
  exact native PHP errors, and broader native expression lowering as named
  unsupported gaps.

## Milestone 461: Native Nonzero Float Additive Identity Folding

- [x] Fold tracked finite nonzero float additive identities in the current
  native arithmetic subset: `$x + 0.0`, `0.0 + $x`, and `$x - 0.0` reuse the
  existing expression in LLVM IR and the C assembly fallback. Preserve emitted
  arithmetic for possible signed-zero identities, keep non-finite float
  behavior, PHP numeric coercion, references/copy-on-write, exact native PHP
  errors, and broader native expression lowering as named unsupported or
  non-folded boundaries, and prove the LLVM and C fallback paths with focused
  tests and CLI fixtures.

## Milestone 462: Native Float Left-Zero Subtraction Folding

- [x] Fold `0.0 - $x` to the known negated literal when `$x` is a
  single-result statically known nonzero finite float operand in the current
  native arithmetic subset. Preserve emitted arithmetic for possible
  signed-zero left-zero subtraction, keep non-finite float behavior, PHP
  numeric coercion, references/copy-on-write, exact native PHP errors, and
  broader native expression lowering as named unsupported or non-folded
  boundaries, and prove the LLVM and C fallback paths with focused tests and
  CLI fixtures.

## Milestone 463: Native Positive Float Multiplication-By-Zero Folding

- [x] Fold `$x * 0.0` and `0.0 * $x` to positive `0.0` when `$x` is a
  statically known finite positive float operand in the current native
  arithmetic subset. Preserve emitted arithmetic for negative and
  signed-zero-sensitive multiplication-by-zero cases, keep non-finite float
  behavior, PHP numeric coercion, references/copy-on-write, exact native PHP
  errors, and broader native expression lowering as named unsupported or
  non-folded boundaries, and prove the LLVM and C fallback paths with focused
  tests and CLI fixtures.

## Milestone 464: Native Float Multiplication-By-Negative-One Folding

- [x] Fold `$x * -1.0` and `-1.0 * $x` to the known negated literal when `$x`
  is a single-result statically known nonzero finite float operand in the
  current native arithmetic subset. Preserve emitted arithmetic for possible
  signed-zero multiplication by `-1.0`, keep non-finite float behavior, PHP
  numeric coercion, references/copy-on-write, exact native PHP errors, and
  broader native expression lowering as named unsupported or non-folded
  boundaries, and prove the LLVM and C fallback paths with focused tests and
  CLI fixtures.

## Milestone 465: Native Tracked Float Arithmetic Folding

- [x] Fold tracked single-result finite nonzero float expression arithmetic for
  `$x + literal`, `$x - literal`, and `$x * literal` to the known float literal
  when exactly one operand is a tracked expression in the current native
  arithmetic subset. Preserve emitted arithmetic for literal-only float
  arithmetic, zero-result arithmetic, signed-zero-sensitive cases, non-finite
  float behavior, PHP numeric coercion, references/copy-on-write, exact native
  PHP errors, and broader native expression lowering as named unsupported or
  non-folded boundaries, and prove the LLVM and C fallback paths with focused
  tests and CLI fixtures.

## Milestone 466: Native Tracked Integer Arithmetic Folding

- [x] Fold tracked single-result integer expression arithmetic with exactly
  one tracked expression operand and one literal operand for `+`, `-`, and `*`
  to the known integer literal after checked overflow analysis in the current
  native arithmetic subset. Preserve emitted arithmetic for literal-only
  integer arithmetic and tracked-expression plus tracked-expression integer
  arithmetic, keep overflow-sensitive arithmetic, PHP numeric coercion,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported or non-folded boundaries, and prove
  the LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 467: Native Tracked Integer Bitwise Folding

- [x] Fold tracked single-result integer expression bitwise operations with
  exactly one tracked expression operand and one literal operand for `&`, `|`,
  and `^` to the known integer literal in the current native bitwise subset.
  Preserve emitted operations for literal-only integer bitwise and
  tracked-expression plus tracked-expression integer bitwise expressions, keep
  PHP bytewise string bitwise behavior, scalar-to-int coercion, references and
  copy-on-write, exact native PHP errors, and broader native expression
  lowering as named unsupported or non-folded boundaries, and prove the LLVM
  and C fallback paths with focused tests and CLI fixtures.

## Milestone 468: Native Tracked Integer Shift Folding

- [x] Fold tracked single-result integer expression shifts with static safe
  nonzero counts to the known integer literal in the current native shift
  subset. Preserve emitted operations for literal-only shifts and non-single
  tracked integer shifts, keep overflow-sensitive left shifts, dynamic or
  invalid shift counts, PHP scalar-to-int coercion, string bitwise behavior,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported or non-folded boundaries, and prove
  the LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 469: Native Tracked Integer Comparison Folding

- [x] Fold same-type integer loose/ordering comparisons with exactly one
  tracked single-result integer expression operand and one integer literal
  operand to a static boolean in the current native comparison subset.
  Preserve literal-only comparison folding, emitted comparisons for non-single
  tracked integer operands, PHP comparison coercion gaps, references and
  copy-on-write, exact native PHP errors, and broader native expression
  lowering as named unsupported or non-folded boundaries, and prove the LLVM
  and C fallback paths with focused tests and CLI fixtures.

## Milestone 470: Native Tracked Float Comparison Folding

- [x] Fold same-type finite-float loose/ordering comparisons with exactly one
  tracked single-result float expression operand and one finite-float literal
  operand to a static boolean in the current native comparison subset.
  Preserve literal-only comparison folding, emitted comparisons for non-single
  tracked float operands, non-finite float behavior, PHP comparison coercion
  gaps, references and copy-on-write, exact native PHP errors, and broader
  native expression lowering as named unsupported or non-folded boundaries,
  and prove the LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 471: Native Bounded String Comparison Folding

- [x] Fold known ASCII nonnumeric NUL-free string loose/ordering comparisons
  to a static boolean when every possible safe string outcome matches in the
  current native comparison subset. Preserve emitted `strcmp` comparisons for
  ambiguous bounded string operands, keep numeric-looking, unknown, non-ASCII,
  and NUL-containing string comparisons rejected, and keep PHP comparison
  coercion gaps, references and copy-on-write, exact native PHP errors, and
  broader native expression lowering as named unsupported or non-folded
  boundaries. Prove the LLVM and C fallback paths with focused tests and CLI
  fixtures.

## Milestone 472: Native Single-Result Scalar Ternary Folding

- [x] Fold dynamic integer, finite-float, and boolean ternaries whose possible
  branch values collapse to a single known result without emitting a redundant
  select or C conditional expression. Preserve emitted ternaries for ambiguous
  same-type branches, keep mixed-type branches rejected until native tagged
  values exist, and keep PHP truthiness/coercion gaps, branch side-effect
  ordering, references and copy-on-write, exact native PHP errors, and broader
  native expression lowering as named unsupported or non-folded boundaries.
  Prove the LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 473: Native Boolean Expression Comparison Folding

- [x] Fold same-type boolean expression loose/ordering comparisons to a static
  boolean when tracked possible boolean operands prove one result in the
  current native comparison subset. Preserve emitted native comparisons for
  ambiguous boolean expressions, keep PHP truthiness/coercion gaps,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported or non-folded boundaries, and prove
  the LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 474: Native Bounded Integer Comparison Folding

- [x] Fold same-type integer loose/ordering comparisons to a static boolean
  when tracked possible integer operands prove one result in the current
  native comparison subset. Preserve emitted native comparisons for ambiguous
  bounded integer comparisons, keep PHP numeric/coercion gaps,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported or non-folded boundaries, and prove
  the LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 475: Native Bounded Float Comparison Folding

- [x] Fold same-type finite-float loose/ordering comparisons to a static
  boolean when tracked possible finite-float operands prove one result in the
  current native comparison subset. Preserve emitted native comparisons for
  ambiguous bounded float comparisons, keep non-finite float behavior, PHP
  numeric/coercion gaps, references/copy-on-write, exact native PHP errors, and
  broader native expression lowering as named unsupported or non-folded
  boundaries, and prove the LLVM and C fallback paths with focused tests and
  CLI fixtures.

## Milestone 476: Native Boolean Logical Known-Result Folding

- [x] Fold native boolean expression `&&`/`and`, `||`/`or`, and `xor`
  operations to a static boolean when tracked possible boolean operands prove
  one result in the current native logical subset. Preserve emitted native
  logical operations for ambiguous boolean expressions, keep PHP truthiness,
  short-circuit side-effect gaps, references/copy-on-write, exact native PHP
  errors, and broader native expression lowering as named unsupported or
  non-folded boundaries, and prove the LLVM and C fallback paths with focused
  tests and CLI fixtures.

## Milestone 477: Native Tracked-Expression Integer Arithmetic Folding

- [x] Fold tracked integer expression arithmetic for `+`, `-`, and `*` to the
  known integer literal when tracked possible integer operands prove one result
  after checked overflow analysis in the current native arithmetic subset.
  Preserve emitted arithmetic for literal-only integer arithmetic and
  ambiguous tracked-expression plus tracked-expression integer arithmetic, keep
  overflow-sensitive arithmetic, PHP numeric coercion, references/copy-on-write,
  exact native PHP errors, and broader native expression lowering as named
  unsupported or non-folded boundaries, and prove the LLVM and C fallback paths
  with focused tests and CLI fixtures.

## Milestone 478: Native Tracked-Expression Integer Bitwise Folding

- [x] Fold tracked integer expression bitwise operations for `&`, `|`, and
  `^` to the known integer literal when tracked possible integer operands
  prove one result in the current native bitwise subset. Preserve emitted
  operations for literal-only integer bitwise and ambiguous tracked-expression
  plus tracked-expression integer bitwise expressions, keep PHP bytewise string
  bitwise behavior, scalar-to-int coercion, references and copy-on-write,
  exact native PHP errors, and broader native expression lowering as named
  unsupported or non-folded boundaries, and prove the LLVM and C fallback paths
  with focused tests and CLI fixtures.

## Milestone 479: Native Tracked Integer Shift Count Folding

- [x] Accept tracked integer expression shift counts when they prove one safe
  count from 0 through 63 in the current native shift subset, using that
  proven count for LLVM IR and C fallback lowering. Preserve emitted
  literal-left shifts, reject ambiguous tracked shift counts and invalid
  counts, keep PHP scalar-to-int coercion, string bitwise behavior,
  overflow-sensitive shifts, references/copy-on-write, exact native PHP errors,
  and broader native expression lowering as named unsupported or non-folded
  boundaries, and prove the LLVM and C fallback paths with focused tests and
  CLI fixtures.

## Milestone 480: Native Tracked-Expression Float Arithmetic Folding

- [x] Fold tracked finite nonzero float expression arithmetic for `+`, `-`,
  and `*` to the known float literal when tracked possible finite-float
  operands prove one nonzero result in the current native arithmetic subset.
  Preserve emitted operations for literal-only float arithmetic, zero-result
  float arithmetic, and ambiguous tracked-expression plus tracked-expression
  float arithmetic, keep signed-zero-sensitive behavior, non-finite floats, PHP
  numeric coercion, references/copy-on-write, exact native PHP errors, and
  broader native expression lowering as named unsupported or non-folded
  boundaries, and prove the LLVM and C fallback paths with focused tests and
  CLI fixtures.

## Milestone 481: Native Single-Result String Ternary Concatenation Folding

- [x] Fold string concatenation operands that are ternary expressions proving
  one static string result into the existing generated static string constant
  path. Preserve rejection for ambiguous string ternary concatenation, PHP
  scalar-to-string conversion, runtime string allocation, arrays, objects,
  resources, references/copy-on-write, exact native PHP errors, and broader
  native expression lowering as named unsupported boundaries, and prove the
  LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 482: Native Boolean Short Ternary Lowering

- [x] Lower short ternary `?:` for already-lowerable boolean conditions in the
  current straight-line native subset. Preserve rejection for non-boolean
  truthiness/coercion, non-boolean fallback values when the fallback is needed,
  null coalescing, arrays, objects, references/copy-on-write, exact native PHP
  errors, and broader native expression lowering as named unsupported
  boundaries, and prove the LLVM and C fallback paths with focused tests and
  CLI fixtures.

## Milestone 483: Native Static-False Short Ternary Scalar Fallback Folding

- [x] Fold static-false short ternary `false ?: fallback` to any
  already-lowerable scalar fallback in the current straight-line native subset,
  while preserving static-true fallback laziness. Preserve rejection for
  dynamic non-boolean truthiness/coercion, dynamic non-boolean fallback values,
  null coalescing, arrays, objects, references/copy-on-write, exact native PHP
  errors, and broader native expression lowering as named unsupported
  boundaries, and prove the LLVM and C fallback paths with focused tests and
  CLI fixtures.

## Milestone 484: Native Single-Known Integer Short Ternary Folding

- [x] Fold short ternary `?:` when an already-lowerable integer condition has
  one statically known truthiness result in the current straight-line native
  subset: proven nonzero integer conditions reuse the integer result, and
  proven zero integer conditions use the fallback. Preserve rejection for
  ambiguous integer truthiness, broader non-boolean truthiness/coercion, null
  coalescing, arrays, objects, references/copy-on-write, exact native PHP
  errors, and broader native expression lowering as named unsupported
  boundaries, and prove the LLVM and C fallback paths with focused tests and
  CLI fixtures.

## Milestone 485: Native Single-Known Float Short Ternary Folding

- [x] Fold short ternary `?:` when an already-lowerable finite float condition
  has one statically known truthiness result in the current straight-line
  native subset: proven nonzero finite float conditions reuse the float result,
  and proven zero float conditions use the fallback. Preserve rejection for
  ambiguous float truthiness, non-finite floats, broader non-boolean
  truthiness/coercion, null coalescing, arrays, objects,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported boundaries, and prove the LLVM and
  C fallback paths with focused tests and CLI fixtures.

## Milestone 486: Native Known-String Short Ternary Folding

- [x] Fold short ternary `?:` when an already-lowerable known string condition
  has one statically known PHP string-truthiness result in the current
  straight-line native subset: known truthy strings reuse the string result,
  and known falsey `""`/`"0"` strings use the fallback. Preserve rejection for
  ambiguous string truthiness, untracked string expressions, broader
  non-boolean truthiness/coercion, null coalescing, arrays, objects,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported boundaries, and prove the LLVM and
  C fallback paths with focused tests and CLI fixtures.

## Milestone 487: Native Known-String Logical-Not Folding

- [x] Fold logical not `!` when an already-lowerable known string operand has
  one statically known PHP string-truthiness result in the current
  straight-line native subset: known falsey `""`/`"0"` strings fold to `true`,
  and known truthy strings fold to `false`. Preserve rejection for ambiguous
  string truthiness, untracked string expressions, broader truthiness/coercion,
  arrays, objects, references/copy-on-write, exact native PHP errors, and
  broader native expression lowering as named unsupported boundaries, and
  prove the LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 488: Native Known-Numeric Logical-Not Folding

- [x] Fold logical not `!` when an already-lowerable known integer or finite
  float operand has one statically known PHP truthiness result in the current
  straight-line native subset: known zero numeric operands fold to `true`, and
  known nonzero numeric operands fold to `false`. Preserve rejection for
  ambiguous numeric truthiness, non-finite floats, untracked numeric
  expressions, broader truthiness/coercion, arrays, objects,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported boundaries, and prove the LLVM and
  C fallback paths with focused tests and CLI fixtures.

## Milestone 489: Native Known-Scalar Logical Truthiness Folding

- [x] Fold logical `&&`, `||`, and `xor` when both already-lowerable scalar
  operands have one statically known PHP truthiness result in the current
  straight-line native subset. Preserve rejection for ambiguous scalar
  truthiness, untracked scalar operands, non-finite floats, null truthiness
  beyond this documented folding subset, short-circuit cases that would need skipped unsupported or side-effecting
  operands, arrays, objects, references/copy-on-write, exact native PHP errors,
  and broader native expression lowering as named unsupported boundaries, and
  prove the LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 490: Native Null Logical-Not Folding

- [x] Fold logical not `!null` to `true` in the current straight-line native
  subset. Preserve rejection for broader null truthiness beyond the documented
  logical binary and conditional folding subsets, non-null ambiguous truthiness, arrays, objects,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported boundaries, and prove the LLVM and
  C fallback paths with focused tests and CLI fixtures.

## Milestone 491: Native Null Short Ternary Fallback Folding

- [x] Fold short ternary `?:` when an already-lowerable `null` condition is
  used in the current straight-line native subset: `null ?: fallback` lowers to
  the fallback. Preserve rejection for broader null truthiness beyond the
  documented logical binary folding subset, null coalescing, lazy unsupported branch skipping,
  arrays, objects, references/copy-on-write, exact native PHP errors, and
  broader native expression lowering as named unsupported boundaries, and prove
  the LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 492: Native Single-Known Integer Full Ternary Condition Folding

- [x] Fold full ternary `?:` condition selection when an already-lowerable
  integer condition has one statically known PHP truthiness result in the
  current straight-line native subset: known nonzero integer conditions select
  the true branch, and known zero integer conditions select the false branch.
  Preserve rejection for ambiguous integer truthiness, unsupported branch
  skipping, broader PHP truthiness/coercion, null coalescing, arrays, objects,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported boundaries, and prove the LLVM and C
  fallback paths with focused tests and CLI fixtures.

## Milestone 493: Native Single-Known Float Full Ternary Condition Folding

- [x] Fold full ternary `?:` condition selection when an already-lowerable
  finite-float condition has one statically known PHP truthiness result in the
  current straight-line native subset: known nonzero finite-float conditions
  select the true branch, and known zero finite-float conditions select the
  false branch. Preserve rejection for ambiguous float truthiness, non-finite
  floats, unsupported branch skipping, broader PHP truthiness/coercion, null
  coalescing, arrays, objects, references/copy-on-write, exact native PHP
  errors, and broader native expression lowering as named unsupported
  boundaries, and prove the LLVM and C fallback paths with focused tests and
  CLI fixtures.

## Milestone 494: Native Known-String Full Ternary Condition Folding

- [x] Fold full ternary `?:` condition selection when an already-lowerable known
  string condition has one statically known PHP string-truthiness result in the
  current straight-line native subset: known truthy strings select the true
  branch, and known falsey `""`/`"0"` strings select the false branch. Preserve
  rejection for ambiguous string truthiness, untracked string expressions,
  unsupported branch skipping, broader PHP truthiness/coercion, null coalescing,
  arrays, objects, references/copy-on-write, exact native PHP errors, and
  broader native expression lowering as named unsupported boundaries, and prove
  the LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 495: Native Null Full Ternary Condition Folding

- [x] Fold full ternary `?:` condition selection when an already-lowerable
  `null` condition is used in the current straight-line native subset:
  `null ? true_branch : false_branch` lowers to the false branch after both
  branches lower. Preserve rejection for unsupported branch skipping, broader
  null truthiness beyond the documented logical binary folding subset, null coalescing, arrays, objects,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported boundaries, and prove the LLVM and C
  fallback paths with focused tests and CLI fixtures.

## Milestone 496: Native Null Logical Truthiness Folding

- [x] Fold logical `&&`, `||`, and `xor` when both operands are already
  lowerable and any `null` operand has statically known falsey PHP truthiness in
  the current straight-line native subset. Preserve rejection for
  short-circuit cases that would need skipped unsupported or side-effecting
  operands, null coalescing, arrays, objects, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering as named
  unsupported boundaries, and prove the LLVM and C fallback paths with focused
  tests and CLI fixtures.

## Milestone 497: Native Static Full Ternary Selected-Branch Lowering

- [x] Lower only the selected full ternary branch when the condition has a
  statically known truthiness result in the current straight-line native subset,
  allowing unsupported unselected branches such as arrays to stay unlowered.
  Preserve rejection for dynamic branch skipping, dynamic PHP truthiness,
  ambiguous scalar truthiness, null coalescing, arrays in selected branches,
  objects, references/copy-on-write, exact native PHP errors, and broader
  native expression lowering as named unsupported boundaries, and prove the
  LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 498: Native Static Logical Short-Circuit Folding

- [x] Lower statically decisive known-left logical `&&`/`and` and `||`/`or`
  cases without lowering the skipped right-hand operand in the current
  straight-line native subset. Preserve rejection for `xor` right-hand skipping,
  dynamic short-circuiting, selected/evaluated unsupported right-hand operands,
  ambiguous scalar truthiness, untracked scalar operands, null coalescing,
  arrays, objects, references/copy-on-write, exact native PHP errors, and
  broader native expression lowering as named unsupported boundaries, and prove
  the LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 499: Native Integer Bitwise OR All-Ones Folding

- [x] Fold tracked integer expression and integer literal `$x | -1` and
  `-1 | $x` forms to `-1` after both operands lower in the current
  straight-line native subset. Preserve rejection for PHP scalar-to-int
  coercion, string bitwise behavior in native lowering, arrays, objects,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported boundaries, and prove the LLVM and
  C fallback paths with focused tests and CLI fixtures.

## Milestone 500: Native Integer Bitwise XOR All-Ones Folding

- [x] Fold single-known integer `$x ^ -1` and `-1 ^ $x` forms to the known
  bitwise-not result after both operands lower in the current straight-line
  native subset. Preserve emitted/tracked behavior for ambiguous integer
  operands and rejection for PHP scalar-to-int coercion, string bitwise
  behavior in native lowering, arrays, objects, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering as named
  unsupported boundaries, and prove the LLVM and C fallback paths with focused
  tests and CLI fixtures.

## Milestone 501: Native Untracked Integer Bitwise Identity Folding

- [x] Fold `& 0`, `& -1`, `| 0`, and `^ 0` identity or annihilator forms after
  both operands lower even when the other already-lowerable integer operand is
  intentionally untracked, such as overflow-sensitive shift results. Preserve
  emitted untracked shift results, rejection for arithmetic that would need
  exact overflow tracking, PHP scalar-to-int coercion, string bitwise behavior
  in native lowering, arrays, objects, references/copy-on-write, exact native
  PHP errors, and broader native expression lowering as named unsupported
  boundaries, and prove the LLVM and C fallback paths with focused tests and
  CLI fixtures.

## Milestone 502: Native Untracked Integer Arithmetic Identity Folding

- [x] Fold `+ 0`, `- 0`, `* 1`, and `* 0` identity or annihilator forms after
  both operands lower even when the other already-lowerable integer operand is
  intentionally untracked, such as overflow-sensitive shift results. Preserve
  emitted untracked shift results, rejection for non-identity arithmetic that
  would need exact overflow tracking, PHP scalar coercion, references/copy-on-write,
  exact native PHP errors, and broader native expression lowering as named
  unsupported boundaries, and prove the LLVM and C fallback paths with focused
  tests and CLI fixtures.

## Milestone 503: Native Untracked Integer Shift-By-Zero Folding

- [x] Fold `$x << 0` and `$x >> 0` after both operands lower even when the
  already-lowerable left integer operand is intentionally untracked, such as an
  overflow-sensitive shift result. Preserve emitted untracked shift results,
  rejection for nonzero untracked shift/result-tracking cases that would imply
  exact overflow semantics, PHP scalar coercion, references/copy-on-write,
  exact native PHP errors, and broader native expression lowering as named
  unsupported boundaries, and prove the LLVM and C fallback paths with focused
  tests and CLI fixtures.

## Milestone 504: Native Untracked Reflexive Integer Comparison Folding

- [x] Fold same-expression integer loose/ordering comparisons after both
  operands lower even when the integer operand is intentionally untracked, such
  as an overflow-sensitive shift result: `$x == $x`, `$x <= $x`, and
  `$x >= $x` fold true, while `$x != $x`, `$x < $x`, and `$x > $x` fold false.
  Preserve emitted untracked source values, PHP comparison coercion gaps,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported boundaries, and prove the LLVM and
  C fallback paths with focused tests and CLI fixtures.

## Milestone 505: Native Untracked Identical Integer Bitwise Folding

- [x] Fold identical integer bitwise operands after both operands lower even
  when the integer value is intentionally untracked, such as an
  overflow-sensitive shift result: `$x & $x` and `$x | $x` reuse `$x`, while
  `$x ^ $x` folds to zero. Preserve emitted untracked source values, PHP
  scalar-to-int coercion gaps, string bitwise behavior in native lowering,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported boundaries, and prove the LLVM and
  C fallback paths with focused tests and CLI fixtures.

## Milestone 506: Native Untracked Identical Integer Subtraction Folding

- [x] Fold identical integer subtraction after both operands lower even when
  the integer value is intentionally untracked, such as an overflow-sensitive
  shift result. Preserve emitted untracked source values, rejection for other
  non-identity arithmetic that would need exact overflow tracking, PHP scalar
  coercion, references/copy-on-write, exact native PHP errors, and broader
  native expression lowering as named unsupported boundaries, and prove the
  LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 507: Native Untracked Integer Modulo-By-One Folding

- [x] Fold integer modulo by one after both operands lower even when the
  dividend is intentionally untracked, such as an overflow-sensitive shift
  result. Preserve emitted untracked source values, rejection for other modulo
  cases that need runtime divisor checks, PHP scalar coercion,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported boundaries, and prove the LLVM and
  C fallback paths with focused tests and CLI fixtures.

## Milestone 508: Native Untracked Identical Integer Ternary Folding

- [x] Fold identical integer full-ternary branches after both branches lower
  even when the integer value is intentionally untracked, such as an
  overflow-sensitive shift result. Preserve emitted untracked source values,
  dynamic mixed-type branch rejection, unsupported truthiness/coercion gaps,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported boundaries, and prove the LLVM and
  C fallback paths with focused tests and CLI fixtures.

## Milestone 509: Native Scalar Result Tracking Follow-up

- [x] Fold identical float full-ternary branches after both branches lower even
  when the float value is intentionally untracked, such as a non-finite
  overflowing float multiplication. Preserve emitted untracked source values,
  non-finite float result-tracking and truthiness gaps, dynamic mixed-type
  branch rejection, unsupported truthiness/coercion gaps,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported boundaries, and prove the LLVM and
  C fallback paths with focused tests and CLI fixtures.

## Milestone 510: Native Scalar Result Tracking Follow-up

- [x] Fold double unary bitwise-not `~~$x` over already-lowerable integer
  operands to `$x`, including intentionally untracked integer expressions such
  as overflow-sensitive shift results. Preserve emitted untracked source
  values, rejection for non-integer native bitwise operands, PHP scalar-to-int
  coercion gaps, string bitwise behavior in native lowering,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported boundaries, and prove the LLVM and
  C fallback paths with focused tests and CLI fixtures.

## Milestone 511: Native Scalar Result Tracking Follow-up

- [x] Fold double logical-not `!!$x` over already-lowerable known scalar
  operands through the existing known-truthiness subset: integers, finite
  floats, strings, and `null`. Preserve rejection for ambiguous truthiness,
  untracked numeric/string operands, non-finite floats, arrays, objects,
  references/copy-on-write, exact native PHP errors, and broader native
  expression lowering as named unsupported boundaries, and prove the LLVM and
  C fallback paths with focused tests and CLI fixtures.

## Milestone 512: Native Scalar Result Tracking Follow-up

- [x] Fold identical native boolean expression loose/ordering comparisons after
  both operands lower, including ambiguous boolean expressions: `$flag ==
  $flag`, `$flag <= $flag`, and `$flag >= $flag` fold true, while `$flag !=
  $flag`, `$flag < $flag`, and `$flag > $flag` fold false. Preserve the
  source ambiguous boolean expression, other ambiguous boolean comparisons,
  PHP comparison coercion gaps, references/copy-on-write, exact native PHP
  errors, and broader native expression lowering as named unsupported
  boundaries, and prove the LLVM and C fallback paths with focused tests and
  CLI fixtures.

## Milestone 513: Native Scalar Result Tracking Follow-up

- [x] Fold identical native string pointer loose/ordering comparisons after
  both operands lower, even when the string pointer expression is intentionally
  untracked because its possible value set exceeds the current small tracker:
  `$text == $text`, `$text <= $text`, and `$text >= $text` fold true, while
  `$text != $text`, `$text < $text`, and `$text > $text` fold false. Preserve
  the source untracked string expression in LLVM IR, keep non-identical unknown
  string comparisons rejected until safe value sets or runtime-backed string
  comparison semantics exist, and prove the LLVM and C fallback paths with
  focused tests and CLI fixtures.

## Milestone 514: Native Scalar Result Tracking Follow-up

- [x] Fold empty-string concatenation identity for already-lowerable string
  operands, including untracked string pointer expressions whose possible
  value set exceeds the current small tracker: `$text . ""` and `"" . $text`
  reuse `$text` without runtime string allocation. Preserve the source
  untracked string expression in LLVM IR, keep non-empty ambiguous string
  concatenation rejected until runtime string allocation exists, and prove the
  LLVM and C fallback paths with focused tests and CLI fixtures.

## Milestone 515: Native Scalar Result Tracking Follow-up

- [x] Fold identical direct string-variable short ternary identity for
  already-lowerable string pointer operands, including untracked string pointer
  expressions whose possible value set exceeds the current small tracker:
  `$text ?: $text` reuses `$text` without proving string truthiness. Preserve
  the source untracked string expression in LLVM IR, keep non-identical
  untracked string short ternaries rejected until general native PHP
  truthiness/lazy fallback behavior exists, and prove the LLVM and C fallback
  paths with focused tests and CLI fixtures.

## Milestone 516: Native Scalar Result Tracking Follow-up

- [x] Fold identical direct integer-variable short ternary identity for
  already-lowerable integer operands, including intentionally untracked
  integer expressions such as overflow-sensitive shift results: `$value ?:
  $value` reuses `$value` without proving integer truthiness. Preserve the
  source untracked integer expression in LLVM IR, keep non-identical untracked
  integer short ternaries rejected until general native PHP truthiness/lazy
  fallback behavior exists, and prove the LLVM and C fallback paths with
  focused tests and CLI fixtures.

## Milestone 517: Native Scalar Result Tracking Follow-up

- [x] Fold identical direct float-variable short ternary identity for
  already-lowerable float operands, including intentionally untracked
  non-finite float-producing expressions such as overflowing float
  multiplication: `$value ?: $value` reuses `$value` without proving float
  truthiness. Preserve the source untracked float expression in LLVM IR, keep
  non-identical untracked float short ternaries rejected until general native
  PHP truthiness/lazy fallback behavior exists, and prove the LLVM and C
  fallback paths with focused tests and CLI fixtures.

## Milestone 518: Native Scalar Result Tracking Follow-up

- [x] Fold identical direct boolean-variable short ternary identity for
  already-lowerable boolean expression operands: `$flag ?: $flag` reuses
  `$flag` without emitting a redundant boolean select. Preserve the source
  boolean expression in LLVM IR, keep non-identical boolean short ternaries on
  the existing lowerable boolean fallback path, and prove the LLVM and C
  fallback paths with focused tests and CLI fixtures.

## Milestone 519: Native Scalar Result Tracking Follow-up

- [x] Fold identical direct-variable full ternary identity for already-lowerable
  scalar operands, starting with intentionally untracked integer expressions:
  `$value ? $value : $value` reuses `$value` without proving integer
  truthiness. Preserve the source untracked integer expression in LLVM IR,
  keep non-identical untracked integer full ternaries rejected until general
  native PHP truthiness/lazy branch behavior exists, and prove the LLVM and C
  fallback paths with focused tests and CLI fixtures.

## Milestone 520: Native Scalar Result Tracking Follow-up

- [x] Prove identical direct-variable full ternary identity for already-lowerable
  float operands, including intentionally untracked non-finite float-producing
  expressions: `$value ? $value : $value` reuses `$value` without proving
  float truthiness. Preserve the source untracked float expression in LLVM IR,
  keep non-identical untracked float full ternaries rejected until general
  native PHP truthiness/lazy branch behavior exists, and prove the LLVM and C
  fallback paths with focused tests and CLI fixtures.

## Milestone 521: Native Scalar Result Tracking Follow-up

- [x] Prove identical direct-variable full ternary identity for already-lowerable
  string pointer operands, including untracked string pointer expressions whose
  possible value set exceeds the current small tracker: `$text ? $text :
  $text` reuses `$text` without proving string truthiness. Preserve the source
  untracked string expression in LLVM IR, keep non-identical untracked string
  full ternaries rejected until general native PHP truthiness/lazy branch
  behavior exists, and prove the LLVM and C fallback paths with focused tests
  and CLI fixtures.

## Milestone 522: Native Scalar Result Tracking Follow-up

- [x] Prove identical direct-variable full ternary identity for already-lowerable
  boolean expression operands: `$flag ? $flag : $flag` reuses `$flag` without
  emitting a redundant boolean select. Preserve the source boolean expression
  in LLVM IR, keep non-identical boolean full ternaries on the existing
  lowerable boolean branch path, and prove the LLVM and C fallback paths with
  focused tests and CLI fixtures.

## Milestone 523: Native Scalar Result Tracking Follow-up

- [x] Add the next honest native-codegen boundary or executable slice for
  scalar-producing expressions that are currently lowerable but not tracked
  precisely enough for later scalar lowering. Preserve safe known-result
  tracking only where executable code, fixtures, CLI coverage, docs, and
  focused tests prove it, or keep/reinforce a specific diagnostic until native
  PHP truthiness/coercion, runtime checks, references/copy-on-write, exact
  native PHP errors, and broader native expression lowering exist.

## Milestone 530: Runtime Type-Introspection Follow-up

- [x] Add `is_numeric($value)` for the current runtime value model, including
  direct and string-valued dynamic calls, fixture/CLI coverage, system PHP
  comparison, support documentation, and explicit native-codegen rejection
  while runtime-backed function-call lowering remains unsupported.

## Milestone 531: Parser Lane Call Syntax

- [x] Parser lane: accept optional trailing commas in positional call argument
  lists while preserving rejection for empty, named, unpacked, and reference
  arguments.

## Milestone 532: Runtime Type-Introspection Follow-up

- [x] Runtime lane: add `is_countable($value)` for current arrays while naming
  unsupported `Countable` object/interface semantics and native lowering.

## Milestone 533: IR/Lowering Type-Introspection Follow-up

- [x] IR/lowering lane: statically fold direct scalar/null
  type-introspection calls in native output while keeping dynamic calls,
  arrays, objects, wrong arity, and runtime-backed callable dispatch rejected.

## Milestone 534: Parser Lane Function Syntax

- [x] Parser lane: accept optional trailing commas in user-function and
  class-method declaration parameter lists while preserving rejection for empty
  parameter slots and keeping native function declaration lowering rejected.

## Milestone 535: Runtime Type-Introspection Follow-up

- [x] Runtime lane: add `is_iterable($value)` for current arrays while naming
  unsupported `Traversable`/generator object semantics and native lowering.

## Milestone 536: IR/Lowering Type-Introspection Follow-up

- [x] IR/lowering lane: statically fold direct scalar/null and proven
  string-valued `is_numeric` calls in native output while keeping dynamic
  calls, arrays, objects, wrong arity, `is_countable`, `is_iterable`, and
  runtime-backed callable dispatch rejected.

## Milestone 537: IR/Lowering Type-Introspection Follow-up

- [x] IR/lowering lane: statically fold direct scalar/null/string
  `is_countable` and `is_iterable` calls to false in native output while
  keeping arrays, objects, dynamic calls, wrong arity, and runtime-backed
  callable dispatch rejected.

## Milestone 538: IR/Lowering Object-Introspection Follow-up

- [x] IR/lowering lane: statically fold direct scalar/null/string
  `is_object` calls to false and direct scalar/null/string `get_debug_type`
  calls to current runtime type-name strings while keeping arrays, objects,
  dynamic calls, wrong arity, and broader object metadata lowering rejected.

## Milestone 539: IR/Lowering Object-Introspection Follow-up

- [x] IR/lowering lane: statically fold direct `class_exists`,
  `interface_exists`, `trait_exists`, and `enum_exists` calls with
  already-lowerable string names and optional already-lowerable boolean
  autoload flags to false in native output while keeping arrays, objects,
  dynamic calls, wrong arity, broader object metadata, native class tables,
  autoloading, and runtime-backed callable dispatch rejected.

## Milestone 540: IR/Lowering Object-Introspection Follow-up

- [x] IR/lowering lane: statically fold direct `property_exists` and
  `method_exists` calls with already-lowerable string class names and
  already-lowerable string member names to false in native output while
  keeping arrays, objects, non-string arguments, dynamic calls, wrong arity,
  native class/member tables, autoloading, and runtime-backed callable dispatch
  rejected.

## Milestone 541: IR/Lowering Object-Introspection Follow-up

- [x] IR/lowering lane: statically fold direct `is_a` and `is_subclass_of`
  calls with already-lowerable string object/class names, already-lowerable
  string target class names, and optional already-lowerable boolean
  `allow_string` flags to false in native output while keeping arrays, objects,
  non-string arguments, dynamic calls, wrong arity, native class tables,
  inheritance, autoloading, and runtime-backed callable dispatch rejected.

## Milestone 542: Runtime Type-Introspection Follow-up

- [x] Runtime lane: add `is_callable($value)` for the current one-argument
  string function-name subset, returning true for current user functions and
  documented callable builtins and false for missing names or non-string
  values, while naming unsupported optional arguments, array/object callables,
  method callables, namespace/autoload resolution, and native lowering.

## Milestone 543: Runtime Type-Introspection Follow-up

- [x] Runtime lane: extend `is_callable` with the optional boolean
  `syntax_only` flag for the current string function-name subset, reporting
  string callable syntax without name resolution when the flag is true while
  keeping callable-name output, array/object callables, method callables,
  namespace/autoload resolution, and native lowering unsupported.

## Milestone 544: Runtime Type-Introspection Follow-up

- [x] Runtime lane: add `function_exists($name)` for the current runtime
  string function-name subset, checking current user functions and documented
  callable builtins while naming unsupported non-string name coercion,
  namespace/autoload-aware lookup, extension-loaded functions beyond documented
  builtins, exact native `TypeError`/deprecation behavior, and native lowering.

## Milestone 545: Parser Generator Boundary

- [x] Parser lane: add an explicit stable parse diagnostic for unsupported
  generator `yield` and `yield from` syntax in statement and expression
  positions, while naming unsupported generator functions, generator objects,
  key/value yields, by-reference yields, `send`/`throw`/`return` semantics,
  delegation, and native lowering.

## Milestone 546: Native Direct-Variable `isset` Folding

- [x] IR/lowering lane: statically fold direct `isset($name)` in
  `phpc compile --emit-ir` and the `--emit-asm` C fallback for the current
  straight-line static-variable map, while keeping array offsets, object
  properties, complex operands, multiple operands, unset/mutation
  interactions, references/copy-on-write, and exact native error behavior
  explicitly rejected.

## Milestone 547: Independent Lane Candidates

- [x] IR/lowering lane: statically fold direct `function_exists($name)` calls
  in `phpc compile --emit-ir` and the `--emit-asm` C fallback when `$name` is
  an already-lowerable string with a uniform known result in the documented
  builtin table, while keeping user-defined function tables, dynamic calls,
  wrong arity, non-string names, namespace/autoload-aware lookup,
  extension-loaded functions beyond documented builtins, exact native
  `TypeError`/deprecation behavior, and runtime-backed callable dispatch
  unsupported.

## Milestone 548: Independent Lane Candidates

- [x] IR/lowering lane: statically fold direct `is_callable($value)` and
  `is_callable($value, $syntax_only)` calls in `phpc compile --emit-ir` and
  the `--emit-asm` C fallback when `$value` is an already-lowerable string and
  the optional syntax-only flag is an already-lowerable boolean, while keeping
  callable-name output, array/object/method callables, dynamic calls, wrong
  arity, non-string values, non-bool syntax-only flags, user-defined native
  function tables, namespace/autoload-aware lookup, extension-loaded functions
  beyond documented builtins, exact native `TypeError`/deprecation behavior,
  and runtime-backed callable dispatch unsupported.

## Milestone 549: Independent Lane Candidates

- [x] IR/lowering lane: statically fold direct `is_callable(...)` calls with
  already-lowerable non-string scalar/null values to false in
  `phpc compile --emit-ir` and the `--emit-asm` C fallback, including known
  boolean syntax-only flags, while keeping callable-name output,
  array/object/method callables, dynamic calls, wrong arity, non-bool
  syntax-only flags, user-defined native function tables,
  namespace/autoload-aware lookup, extension-loaded functions beyond
  documented builtins, exact native `TypeError`/deprecation behavior, and
  runtime-backed callable dispatch unsupported.

## Milestone 550: Independent Lane Candidates

- [x] Parser lane: add stable parse diagnostics and CLI coverage for
  unsupported `goto` statements and labels before implementing jump targets,
  cross-scope jump validation, `finally` interaction, or native lowering.

## Milestone 551: Independent Lane Candidates

- [x] Runtime lane: extend `is_callable($value, true)` syntax-only handling to
  recognize current two-element array callable shapes such as
  `["ClassName", "method"]` and `[$object, "method"]` without resolving or
  invoking them, while keeping normal array callable resolution, dynamic
  invocation, callable-name output, visibility-sensitive method callability,
  `__invoke`, static callable strings, namespace/autoload behavior, exact
  native `TypeError` behavior, and native lowering unsupported.

## Milestone 552: Independent Lane Candidates

- [x] Compiler-output lane: add selected-`clang` `--emit-asm` CLI coverage for
  an already-implemented native type-introspection folding slice, proving the
  chosen backend receives deterministic LLVM IR through stdin while keeping
  production lowering behavior unchanged.

## Milestone 553: Independent Lane Candidates

- [x] Runtime lane: consider normal-mode `is_callable([$object_or_class,
  $method])` resolution against the current declared method metadata without
  implementing array callable invocation, inheritance, trait/interface method
  lookup, visibility-sensitive caller context, `__call`, namespace/autoload
  behavior, exact native `TypeError` behavior, or native lowering.

## Milestone 554: Independent Lane Candidates

- [x] Parser lane: add stable diagnostics for unsupported PHP heredoc/nowdoc
  string syntax before implementing multiline string tokenization,
  interpolation, indentation stripping, or native string lowering.

## Milestone 555: Independent Lane Candidates

- [x] Tests/docs lane: add an unsupported syntax boundary closure checklist to
  the lane worker docs, using the current `yield`, `goto`, and heredoc/nowdoc
  boundaries as examples. Keep this slice documentation and verification only:
  no compiler, runtime, or codegen edits.

## Milestone 556: Independent Lane Candidates

- [x] IR/lowering lane: consider native direct-variable `empty($name)` folding
  for the straight-line scalar/null lowering subset, including missing
  variables as true and explicit codegen rejections for array offsets, object
  properties, complex operands, arrays, unset interactions, and ambiguous
  truthiness.

## Milestone 557: Independent Lane Candidates

- [x] Runtime lane: consider `array_change_key_case($array, $case =
  CASE_LOWER)` for the current ordered array model, preserving integer keys,
  lowercasing/uppercasing ASCII string keys, preserving insertion order and
  duplicate overwrite behavior, and naming gaps for invalid arguments,
  Unicode/locale behavior, references/copy-on-write, and native lowering.

## Milestone 558: Independent Lane Candidates

- [x] Compiler-output lane: add selected-`clang` `--emit-asm` CLI coverage for
  existing native direct `function_exists($name)` folding, proving the fake
  backend receives deterministic folded LLVM IR through stdin without changing
  production lowering behavior.

## Milestone 559: Independent Lane Candidates

- [x] Compiler-output lane: add selected-`clang` `--emit-asm` CLI coverage for
  existing native direct-variable `empty($name)` folding, proving the fake
  backend receives deterministic folded LLVM IR through stdin without changing
  production lowering behavior.

## Milestone 560: Independent Lane Candidates

- [x] Tests/docs lane: refresh the lane-worker current queue and post-559
  rotation notes so parser, IR/lowering, runtime, compiler-output, and
  tests/docs workers each have one small milestone candidate and stale
  completed-lane assignments do not persist.

## Milestone 561: Independent Lane Candidates

- [x] Parser lane: implement alternate `switch (...): ... endswitch;` syntax
  by reusing the existing switch AST/interpreter path, adding stable
  diagnostics for malformed alternate switch forms, CLI coverage, docs, and
  named unsupported edge cases while keeping native switch lowering rejected.

## Milestone 562: Independent Lane Candidates

- [x] IR/lowering lane: add native direct `strlen($value)` folding for
  already-lowerable known string operands, with narrow `--emit-ir` and
  fallback `--emit-asm` coverage, and keep non-string coercions, arrays,
  objects, dynamic calls, runtime lookup, and exact native PHP errors
  explicitly rejected.

## Milestone 563: Independent Lane Candidates

- [x] Runtime lane: add `array_unique($array, SORT_STRING)` for `phpc run`
  over the current scalar value subset, update the unsupported sort-flag
  runtime error coverage, add focused fixtures and system PHP comparison, and
  keep other sort flags, non-scalar values, references/copy-on-write, exact PHP
  warnings, and native lowering explicitly unsupported.

## Milestone 564: Independent Lane Candidates

- [x] Compiler-output lane: add selected-`clang` `--emit-asm` CLI coverage for
  existing native direct-variable `isset($name)` folding, proving the fake
  backend receives deterministic folded LLVM IR through stdin without changing
  production lowering behavior.

## Milestone 565: Independent Lane Candidates

- [x] Compiler-output lane: add selected-`clang` `--emit-asm` CLI coverage for
  existing native direct `is_numeric($value)` folding, proving deterministic
  folded LLVM IR reaches the selected backend through stdin without changing
  production lowering behavior.

## Milestone 566: Independent Lane Candidates

- [x] IR/lowering lane: choose the next already-implemented scalar/string
  builtin with a deterministic native folding opportunity, add narrow
  `--emit-ir` and fallback `--emit-asm` coverage, and keep arrays, objects,
  dynamic calls, runtime lookup, and exact native PHP errors explicitly
  rejected.

## Milestone 567: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the documented unsupported gaps, implement it through `phpc run` with
  focused fixtures and system PHP comparison where applicable, and keep native
  lowering explicitly rejected.

## Milestone 568: Independent Lane Candidates

- [x] Compiler-output lane: choose the next selected-`clang` `--emit-asm`
  coverage target from an existing native scalar/string folding slice, add a
  deterministic fake-backend stdin validation snapshot, and leave production
  lowering unchanged.

## Milestone 569: Independent Lane Candidates

- [x] IR/lowering lane: choose the next small deterministic native folding
  slice from the already-supported scalar/string boundary, add focused
  `--emit-ir` and fallback `--emit-asm` coverage, and keep runtime-backed
  lookup, arrays, objects, dynamic calls, and exact native PHP errors
  explicitly rejected.

## Milestone 570: Independent Lane Candidates

- [x] Runtime lane: add `array_unique($array, SORT_NUMERIC)` for `phpc run`
  over the current scalar numeric-coercion subset, update unsupported
  sort-flag runtime error coverage, add focused fixtures and system PHP
  comparison, and keep other sort flags, non-numeric values,
  references/copy-on-write, exact PHP warnings, and native lowering explicitly
  unsupported.

## Milestone 571: Independent Lane Candidates

- [x] Parser lane: choose the next narrow parser syntax expansion or syntax
  boundary from the documented unsupported gaps, add parser/fixture coverage,
  update support docs with named unsupported edge cases, and avoid runtime or
  native lowering changes unless a later lane explicitly takes them. Selected
  candidate: stable parse diagnostics for unsupported `list(...)` and `[...]`
  array destructuring assignment targets.

## Milestone 572: Independent Lane Candidates

- [x] Compiler-output lane: choose another selected-backend or fallback CLI
  contract gap from existing native output behavior, add deterministic CLI
  coverage, and leave production lowering behavior unchanged unless the gap is
  itself in CLI/output handling.

## Milestone 573: Independent Lane Candidates

- [x] IR/lowering lane: choose the next small deterministic native folding
  slice from the already-supported scalar/string boundary, add focused
  `--emit-ir` and fallback `--emit-asm` coverage, and keep runtime-backed
  lookup, arrays, objects, dynamic calls, and exact native PHP errors
  explicitly rejected. Selected candidate: extend static native
  `defined($name)` folding to the current `SORT_NUMERIC` built-in constant.

## Milestone 574: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected.

## Milestone 575: Independent Lane Candidates

- [x] Compiler-output lane: choose the next deterministic CLI artifact or
  backend contract coverage target from existing native output behavior, add
  focused snapshot coverage, and leave production lowering behavior unchanged
  unless the gap is itself in CLI/output handling. Selected candidate:
  selected-`clang` stdin validation for existing native
  `defined("SORT_NUMERIC")` folding.

## Milestone 576: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected. Selected candidate: support current
  `array_key_exists($key, $array)` null and boolean key coercions.

## Milestone 577: Independent Lane Candidates

- [x] Compiler-output lane: choose the next deterministic CLI artifact or
  backend contract coverage target from existing native output behavior, add
  focused snapshot coverage, and leave production lowering behavior unchanged
  unless the gap is itself in CLI/output handling. Selected candidate:
  selected-`clang` stdin validation for existing native
  `defined("SORT_STRING")` folding.

## Milestone 578: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected. Selected candidate: support
  no-warning integral finite float key coercions for
  `array_key_exists($key, $array)`.

## Milestone 579: Independent Lane Candidates

- [x] Compiler-output lane: choose the next deterministic CLI artifact or
  backend contract coverage target from existing native output behavior, add
  focused snapshot coverage, and leave production lowering behavior unchanged
  unless the gap is itself in CLI/output handling. Selected candidate:
  selected-`clang` stdin validation for the existing broader native
  `defined($name)` built-in constant answer table.

## Milestone 580: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected. Selected candidate: support
  `array_combine($keys, $values)` null and boolean key-value coercions.

## Milestone 581: Independent Lane Candidates

- [x] Compiler-output lane: choose the next deterministic CLI artifact or
  backend contract coverage target from existing native output behavior, add
  focused snapshot coverage, and leave production lowering behavior unchanged
  unless the gap is itself in CLI/output handling. Selected candidate:
  selected-`clang` stdin validation for existing native scalar/null
  `is_countable($value)` and `is_iterable($value)` false-folding.

## Milestone 582: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected. Selected candidate: support
  `array_combine($keys, $values)` integral finite float key-value coercions.

## Milestone 583: Independent Lane Candidates

- [x] Compiler-output lane: choose the next deterministic CLI artifact or
  backend contract coverage target from existing native output behavior, add
  focused snapshot coverage, and leave production lowering behavior unchanged
  unless the gap is itself in CLI/output handling. Selected candidate:
  selected-`clang` stdin validation for existing native scalar/null
  `is_object($value)` false-folding and `get_debug_type($value)` folding.

## Milestone 584: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected. Selected candidate: support
  `array_fill_keys($keys, $value)` null, boolean, and integral finite float
  key-value coercions.

## Milestone 585: Independent Lane Candidates

- [x] Compiler-output lane: choose the next deterministic CLI artifact or
  backend contract coverage target from existing native output behavior, add
  focused snapshot coverage, and leave production lowering behavior unchanged
  unless the gap is itself in CLI/output handling. Selected candidate:
  selected-`clang` stdin validation for existing native static metadata-exists
  false-folding.

## Milestone 586: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected. Selected candidate:
  `array_column($rows, $column_key)` for array rows and public object rows.

## Milestone 587: Independent Lane Candidates

- [x] Parser lane: choose the next small syntax or parse-diagnostic boundary
  from the documented unsupported gaps, add parser/unit coverage plus
  `phpc run` CLI snapshots where applicable, and do not widen runtime or native
  support claims unless another lane implements and tests the behavior.
  Selected candidate: recognize and reject unsupported exponentiation syntax
  `**` and `**=` with a stable parse diagnostic.

## Milestone 588: Independent Lane Candidates

- [x] IR/lowering lane: choose the next narrow native IR/lowering refinement
  or precise rejection boundary from already documented interpreter behavior,
  add focused `--emit-ir`/`--emit-asm` coverage, and keep unsupported native
  cases rejected before misleading backend output. Selected candidate:
  include `array_change_key_case` in native `function_exists`/`is_callable`
  callable lookup folding while keeping direct array builtin calls rejected.

## Milestone 589: Independent Lane Candidates

- [x] Tests/docs lane: refresh the split-lane queue after the 583/584 batch,
  keep `GOAL.MD`, `docs/LANE_WORKERS.md`, `docs/NEXT_TASKS.md`,
  `docs/SUPPORT.md`, and `docs/PROGRESS.md` aligned, and record the focused
  test/full-gate policy without requiring workspace-wide tests for every
  narrow lane slice. Selected candidate: close the post-583/584 queue refresh,
  carry forward the focused lane-test policy, and leave the next active
  per-lane milestones at compiler-output 590, IR/lowering 591, parser 592,
  runtime 593, and tests/docs 594.

## Milestone 590: Independent Lane Candidates

- [x] Compiler-output lane: choose the next deterministic CLI artifact or
  backend contract coverage target from existing native output behavior, add
  focused snapshot coverage, and leave production lowering behavior unchanged
  unless the gap is itself in CLI/output handling. Selected candidate:
  selected-`clang` empty assembly stdout remains a final diagnostic without
  falling back to `llc` or `cc`.

## Milestone 591: Independent Lane Candidates

- [x] IR/lowering lane: choose the next narrow native IR/lowering refinement
  or precise rejection boundary from already documented interpreter behavior,
  add focused `--emit-ir`/`--emit-asm` coverage, and keep unsupported native
  cases rejected before misleading backend output. Selected candidate:
  native callable lookup folding coverage for `array_column` while direct
  `array_column(...)` native execution remains rejected.

## Milestone 592: Independent Lane Candidates

- [x] Parser lane: choose the next small syntax or parse-diagnostic boundary
  from the documented unsupported gaps, add parser/unit coverage plus
  `phpc run` CLI snapshots where applicable, and do not widen runtime or native
  support claims unless another lane implements and tests the behavior.
  Selected candidate: first-class callable syntax such as `strlen(...)` and
  `$callback(...)` now stops at a stable parse boundary.

## Milestone 593: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected. Selected candidate:
  `array_column($rows, $column_key, $index_key)` for int/string result index
  values over array rows and current public object rows.

## Milestone 594: Independent Lane Candidates

- [x] Tests/docs lane: refresh the split-lane queue after the current 590-593
  implementation batch, keep `GOAL.MD`, `docs/LANE_WORKERS.md`,
  `docs/NEXT_TASKS.md`, `docs/SUPPORT.md`, and `docs/PROGRESS.md` aligned,
  record focused verification commands, and identify the next checkpoint point
  where the serialized full gate should run. Selected candidate: close the
  590-593 split-lane batch in planning docs, open the next per-lane queue, and
  record that `tools/run-tests.sh` should run before the next checkpoint batch.

## Milestone 595: Independent Lane Candidates

- [x] Compiler-output lane: choose the next deterministic CLI artifact or
  backend contract coverage target from existing native output behavior, add
  focused snapshot coverage, and leave production lowering behavior unchanged
  unless the gap is itself in CLI/output handling. Selected candidate:
  selected-`clang` whitespace-only assembly stdout remains a final diagnostic
  without falling back to `llc` or `cc`.

## Milestone 596: Independent Lane Candidates

- [x] IR/lowering lane: choose the next narrow native IR/lowering refinement
  or precise rejection boundary from already documented interpreter behavior,
  add focused `--emit-ir`/`--emit-asm` coverage, and keep unsupported native
  cases rejected before misleading backend output. Selected candidate:
  native callable lookup folding coverage for `array_count_values` while
  direct `array_count_values(...)` native execution remains rejected.

## Milestone 597: Independent Lane Candidates

- [x] Parser lane: choose the next small syntax or parse-diagnostic boundary
  from the documented unsupported gaps, add parser/unit coverage plus
  `phpc run` CLI snapshots where applicable, and do not widen runtime or native
  support claims unless another lane implements and tests the behavior.
  Selected candidate: magic class names in `new` expressions such as
  `new self()`, `new parent()`, and `new static()` now stop at a stable parse
  boundary.

## Milestone 598: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected. Selected candidate:
  `get_mangled_object_vars($object)` now includes public, protected, and
  private instance slots with PHP-style mangled keys.

## Milestone 599: Independent Lane Candidates

- [x] Tests/docs lane: refresh the split-lane queue after the next
  implementation batch, keep `GOAL.MD`, `docs/LANE_WORKERS.md`,
  `docs/NEXT_TASKS.md`, `docs/SUPPORT.md`, and `docs/PROGRESS.md` aligned,
  record focused verification commands, and identify the next checkpoint point
  where the serialized full gate should run. Selected candidate: close the
  595-598 split-lane batch in planning docs, open the next per-lane queue, and
  record that `tools/run-tests.sh` should run before checkpointing.

## Milestone 600: Independent Lane Candidates

- [x] Compiler-output lane: choose the next deterministic CLI artifact or
  backend contract coverage target from existing native output behavior, add
  focused snapshot coverage, and leave production lowering behavior unchanged
  unless the gap is itself in CLI/output handling. Selected candidate:
  selected-`clang` success with whitespace-only stdout and stderr diagnostics
  remains a final invalid-output diagnostic without falling back to `llc` or
  `cc`.

## Milestone 601: Independent Lane Candidates

- [x] IR/lowering lane: choose the next narrow native IR/lowering refinement
  or precise rejection boundary from already documented interpreter behavior,
  add focused `--emit-ir`/`--emit-asm` coverage, and keep unsupported native
  cases rejected before misleading backend output. Selected candidate:
  native callable lookup folding coverage for `array_sum` while direct
  `array_sum(...)` native execution remains rejected.

## Milestone 602: Independent Lane Candidates

- [x] Parser lane: choose the next small syntax or parse-diagnostic boundary
  from the documented unsupported gaps, add parser/unit coverage plus
  `phpc run` CLI snapshots where applicable, and do not widen runtime or native
  support claims unless another lane implements and tests the behavior.
  Selected candidate: anonymous class expressions such as `new class {}` and
  `new class() {}` now stop at a stable parse boundary.

## Milestone 603: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected.
  Selected candidate: `array_column($rows, $column_key, $index_key)` now
  accepts null, boolean, and integral finite float row values as result index
  values, while lossy/non-finite floats, arrays, objects, resources, and native
  lowering remain unsupported.

## Milestone 604: Independent Lane Candidates

- [x] Tests/docs lane: refresh the split-lane queue after the next
  implementation batch, keep `GOAL.MD`, `docs/LANE_WORKERS.md`,
  `docs/NEXT_TASKS.md`, `docs/SUPPORT.md`, and `docs/PROGRESS.md` aligned,
  record focused verification commands, and identify the next checkpoint point
  where the serialized full gate should run. Selected candidate: close the
  600-603 split-lane batch in planning docs, open the next per-lane queue, and
  record that `tools/run-tests.sh` should run before checkpointing.

## Milestone 605: Independent Lane Candidates

- [x] Compiler-output lane: choose the next deterministic CLI artifact or
  backend contract coverage target from existing native output behavior, add
  focused snapshot coverage, and leave production lowering behavior unchanged
  unless the gap is itself in CLI/output handling. Selected candidate:
  selected `llc` success with whitespace-only stdout and stderr diagnostics
  remains a final invalid-output diagnostic without falling back to the
  `cc -S` fallback.

## Milestone 606: Independent Lane Candidates

- [x] IR/lowering lane: choose the next narrow native IR/lowering refinement
  or precise rejection boundary from already documented interpreter behavior,
  add focused `--emit-ir`/`--emit-asm` coverage, and keep unsupported native
  cases rejected before misleading backend output. Selected candidate:
  native callable lookup folding coverage for `array_product` while direct
  `array_product(...)` native execution remains rejected.

## Milestone 607: Independent Lane Candidates

- [x] Parser lane: choose the next small syntax or parse-diagnostic boundary
  from the documented unsupported gaps, add parser/unit coverage plus
  `phpc run` CLI snapshots where applicable, and do not widen runtime or native
  support claims unless another lane implements and tests the behavior.
  Selected candidate: clone expressions such as `clone $object` now have
  focused parser, `--emit-ir`, and `phpc run` fixture coverage for the existing
  stable parse boundary.

## Milestone 608: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected. Selected candidate:
  `array_filter($array, $callback, $mode)` and the null-callback form now
  accept boolean mode flags, with `false` using the value-only mode and `true`
  using the value/key mode while native lowering remains rejected.

## Milestone 609: Independent Lane Candidates

- [x] Tests/docs lane: refresh the split-lane queue after the next
  implementation batch, keep `GOAL.MD`, `docs/LANE_WORKERS.md`,
  `docs/NEXT_TASKS.md`, `docs/SUPPORT.md`, and `docs/PROGRESS.md` aligned,
  record focused verification commands, and identify the next checkpoint point
  where the serialized full gate should run. Selected candidate: close the
  605-608 split-lane batch in planning docs, open the next per-lane queue, and
  record that `tools/run-tests.sh` should run before checkpointing.

## Milestone 610: Independent Lane Candidates

- [x] Compiler-output lane: choose the next deterministic CLI artifact or
  backend contract coverage target from existing native output behavior, add
  focused snapshot coverage, and leave production lowering behavior unchanged
  unless the gap is itself in CLI/output handling. Selected candidate:
  selected `llc` success with empty stdout remains a final invalid-output
  diagnostic without falling back to the `cc -S` fallback.

## Milestone 611: Independent Lane Candidates

- [x] IR/lowering lane: choose the next narrow native IR/lowering refinement
  or precise rejection boundary from already documented interpreter behavior,
  add focused `--emit-ir`/`--emit-asm` coverage, and keep unsupported native
  cases rejected before misleading backend output. Selected candidate:
  native callable lookup folding coverage for `array_reduce` while direct
  `array_reduce(...)` native execution remains rejected.

## Milestone 612: Independent Lane Candidates

- [x] Parser lane: choose the next small syntax or parse-diagnostic boundary
  from the documented unsupported gaps, add parser/unit coverage plus
  `phpc run` CLI snapshots where applicable, and do not widen runtime or native
  support claims unless another lane implements and tests the behavior.
  Selected candidate: `instanceof` expressions such as
  `$object instanceof ClassName` now have focused parser, `--emit-ir`, and
  `phpc run` fixture coverage for the existing stable parse boundary.

## Milestone 613: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected. Selected candidate: `array_filter`
  mode flags now accept trimmed integer-string values that parse to `0`, `1`,
  or `2`, covering value-only, value/key, and key-only callback modes plus the
  null-callback falsey filtering path while native lowering remains rejected.

## Milestone 614: Independent Lane Candidates

- [x] Tests/docs lane: refresh the split-lane queue after the next
  implementation batch, keep `GOAL.MD`, `docs/LANE_WORKERS.md`,
  `docs/NEXT_TASKS.md`, `docs/SUPPORT.md`, and `docs/PROGRESS.md` aligned,
  record focused verification commands, and identify the next checkpoint point
  where the serialized full gate should run. Selected candidate: close the
  610-613 split-lane batch in planning docs, open the next per-lane queue, and
  record that `tools/run-tests.sh` should run before checkpointing.

## Milestone 615: Independent Lane Candidates

- [x] Compiler-output lane: choose the next deterministic CLI artifact or
  backend contract coverage target from existing native output behavior, add
  focused snapshot coverage, and leave production lowering behavior unchanged
  unless the gap is itself in CLI/output handling. Selected candidate:
  selected `clang` success with empty stdout and stderr diagnostics remains a
  final invalid-output diagnostic without falling back to `llc` or the
  `cc -S` fallback, and without surfacing successful-backend stderr.

## Milestone 616: Independent Lane Candidates

- [x] IR/lowering lane: choose the next narrow native IR/lowering refinement
  or precise rejection boundary from already documented interpreter behavior,
  add focused `--emit-ir`/`--emit-asm` coverage, and keep unsupported native
  cases rejected before misleading backend output. Selected candidate:
  native callable lookup folding coverage for `array_filter` while direct
  `array_filter(...)` native execution remains rejected.

## Milestone 617: Independent Lane Candidates

- [x] Parser lane: choose the next small syntax or parse-diagnostic boundary
  from the documented unsupported gaps, add parser/unit coverage plus
  `phpc run` CLI snapshots where applicable, and do not widen runtime or native
  support claims unless another lane implements and tests the behavior.
  Selected candidate: `interface Name {}` declarations now have focused
  parser, `--emit-ir`, and `phpc run` fixture coverage for the existing stable
  parse boundary.

## Milestone 618: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected. Selected candidate: `array_filter`
  mode flags now accept finite integral floats and integral numeric strings
  such as `1.0`, `"2.0"`, and `"0e0"` while lossy mode values remain rejected.

## Milestone 619: Independent Lane Candidates

- [x] Tests/docs lane: refresh the split-lane queue after the next
  implementation batch, keep `GOAL.MD`, `docs/LANE_WORKERS.md`,
  `docs/NEXT_TASKS.md`, `docs/SUPPORT.md`, and `docs/PROGRESS.md` aligned,
  record focused verification commands, and identify the next checkpoint point
  where the serialized full gate should run. Selected candidate: close the
  615-618 split-lane batch in planning docs, open the next per-lane queue, and
  record the passing serialized full gate before checkpointing.

## Milestone 620: Independent Lane Candidates

- [x] Compiler-output lane: choose the next deterministic CLI artifact or
  backend contract coverage target from existing native output behavior, add
  focused snapshot coverage, and leave production lowering behavior unchanged
  unless the gap is itself in CLI/output handling. Selected candidate: add
  selected-`llc` empty-stdout-with-stderr no-`cc`-fallback coverage for the
  existing `--emit-asm` backend stdout-validation boundary.

## Milestone 621: Independent Lane Candidates

- [x] IR/lowering lane: choose the next narrow native IR/lowering refinement
  or precise rejection boundary from already documented interpreter behavior,
  add focused `--emit-ir`/`--emit-asm` coverage, and keep unsupported native
  cases rejected before misleading backend output. Selected candidate: add
  native callable lookup folding coverage for `array_is_list` while direct
  `array_is_list(...)` native execution remains rejected.

## Milestone 622: Independent Lane Candidates

- [x] Parser lane: choose the next small syntax or parse-diagnostic boundary
  from the documented unsupported gaps, add parser/unit coverage plus
  `phpc run` CLI snapshots where applicable, and do not widen runtime or native
  support claims unless another lane implements and tests the behavior.
  Selected candidate: add stable unsupported `trait` declaration parse-boundary
  coverage and fixture snapshots while trait metadata/use execution remains
  unsupported.

## Milestone 623: Independent Lane Candidates

- [x] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected. Selected candidate: accept current
  bool-like scalar autoload flags for `class_exists`, `interface_exists`,
  `trait_exists`, and `enum_exists` through `phpc run` while keeping null,
  array, object, autoload side effects, and native non-bool autoload lowering
  unsupported.

## Milestone 624: Independent Lane Candidates

- [x] Tests/docs lane: refresh the split-lane queue after the next
  implementation batch, keep `GOAL.MD`, `docs/LANE_WORKERS.md`,
  `docs/NEXT_TASKS.md`, `docs/SUPPORT.md`, and `docs/PROGRESS.md` aligned,
  record focused verification commands, and identify the next checkpoint point
  where the serialized full gate should run. Selected candidate: close the
  620-623 split-lane batch in planning docs, open the next per-lane queue, and
  record the passing serialized full gate before checkpointing.

## Milestone 625: Independent Lane Candidates

- [ ] Compiler-output lane: choose the next deterministic CLI artifact or
  backend contract coverage target from existing native output behavior, add
  focused snapshot coverage, and leave production lowering behavior unchanged
  unless the gap is itself in CLI/output handling.

## Milestone 626: Independent Lane Candidates

- [ ] IR/lowering lane: choose the next narrow native IR/lowering refinement
  or precise rejection boundary from already documented interpreter behavior,
  add focused `--emit-ir`/`--emit-asm` coverage, and keep unsupported native
  cases rejected before misleading backend output.

## Milestone 627: Independent Lane Candidates

- [x] Parser lane: choose the next small syntax or parse-diagnostic boundary
  from the documented unsupported gaps, add parser/unit coverage plus
  `phpc run` CLI snapshots where applicable, and do not widen runtime or native
  support claims unless another lane implements and tests the behavior.
  Selected candidate: Milestone 1042 implements statement-form short
  `[$a, $b] = expr;` destructuring as an alias for the existing positional
  `list(...)` direct-variable subset, with skipped slots and trailing commas,
  while keeping keyed/nested/reference/non-variable/expression-position and
  `foreach` destructuring unsupported.

## Milestone 628: Independent Lane Candidates

- [ ] Runtime lane: choose the next small array/object builtin refinement from
  the remaining documented unsupported gaps, implement it through `phpc run`
  with focused fixtures and system PHP comparison where applicable, and keep
  native lowering explicitly rejected.

## Milestone 629: Independent Lane Candidates

- [ ] Tests/docs lane: refresh the split-lane queue after the next
  implementation batch, keep `GOAL.MD`, `docs/LANE_WORKERS.md`,
  `docs/NEXT_TASKS.md`, `docs/SUPPORT.md`, and `docs/PROGRESS.md` aligned,
  record focused verification commands, and identify the next checkpoint point
  where the serialized full gate should run.

## Milestone 630: Compatibility Harness Foundation

- [x] Tests/docs lane: create the first PHP/WordPress compatibility manifest
  that names target PHP branches, target WordPress core version policy,
  required host/environment assumptions, smoke-suite commands, and status
  categories (`pass`, `fail`, `skipped-unsupported`, `not-covered`). This must
  not claim compatibility; it only makes the missing work measurable. Selected
  candidate: added `docs/COMPATIBILITY.md` with current PHP/WordPress external
  targets, current subset smoke commands, status categories, current
  not-covered WordPress/native/PHP-branch matrix entries, and first blockers to
  convert into work.

## Milestone 631: PHP Compatibility Smoke Targets

- [x] Runtime/parser lane: add the first tiny PHP-compatibility smoke fixture
  group that is intentionally broader than a unit feature slice, compare it
  against system PHP where feasible, and record every skipped or failing case
  with a named unsupported reason. Selected candidate: added
  `tests/fixtures/compat/php/cross_feature_smoke.php`, spanning constants,
  functions, arrays, callback builtin use, class metadata, public properties,
  conditionals, `foreach`, and system PHP comparison with no skips.

## Milestone 632: WordPress Bootstrap Inventory

- [x] Tests/docs lane: add a pinned WordPress core compatibility inventory plan
  that starts with parse/load blockers for `wp-settings.php` and bootstrap
  dependencies, names required PHP extensions and host state, and defines the
  first non-networked smoke target. Do not vendor WordPress or claim bootstrap
  support until a repeatable harness exists. Selected candidate: added
  `docs/WORDPRESS_COMPATIBILITY.md` and `tools/wordpress-inventory.sh`, pinning
  the first target to external WordPress 6.9.4 source, naming 7.0 as a future
  update target after its scheduled 2026-05-20 release, and defining the first
  `wp-settings.php` inventory probe without vendoring WordPress or claiming
  bootstrap support.

## Milestone 633: Program Structure Prerequisite

- [x] Parser/runtime lane: choose the smallest include/require or namespace
  prerequisite needed by the WordPress bootstrap inventory, implement or tighten
  its explicit unsupported boundary with CLI coverage, and keep native lowering
  honest. Selected candidate: added a WordPress-shaped `require ABSPATH .
  WPINC . '/load.php';` unsupported-boundary parser case and CLI fixture,
  matching the first expected `wp-settings.php` bootstrap blocker while
  include/require execution remains unimplemented.

## Milestone 634: Native Runtime Prerequisite

- [x] IR/lowering/compiler-output lane: define the first runtime-backed native
  execution prerequisite as code or a design artifact with tests: generated-code
  ABI, boxed value handoff, runtime helper call, link/run command, or a precise
  rejection boundary that blocks misleading native compatibility claims.
  Selected candidate: added the first `php_runtime` C-compatible scalar ABI
  surface for `null`, booleans, integers, and floats, with exported constructor
  symbols, runtime `Value` conversion, focused unit tests, and
  `docs/NATIVE_RUNTIME_ABI.md` documenting the remaining native-execution gaps.

## Milestone 635: Native Runtime ABI Follow-up

- [x] IR/lowering/compiler-output lane: choose the next smallest runtime ABI
  bridge after scalar value construction: generated LLVM declaration/use of one
  exported runtime helper, string ownership design, echo helper handoff, or a
  link-command prototype that still reports unsupported executable mode clearly.
  Selected candidate: added scalar echo sizing/writing helper symbols for
  `NativeScalarValue`, with focused runtime tests for required length reporting,
  partial buffer writes, and null-buffer sizing. Generated LLVM does not call
  these helpers yet.

## Milestone 636: Native Runtime Helper Lowering

- [x] IR/lowering/compiler-output lane: add the first generated LLVM declaration
  or deterministic IR snapshot that references an exported native runtime helper
  without linking/executing yet, or document the exact blocker if current crate
  artifact layout prevents a truthful helper-call snapshot. Added a deterministic
  scalar echo helper IR probe snapshot and compiler test that names
  `phpc_native_scalar_echo_len` and `phpc_native_scalar_echo_write`, while
  documenting that normal `--emit-ir` still does not emit linked runtime helper
  calls.

## Milestone 637: Native Runtime Helper Lowering Follow-up

- [x] IR/lowering/compiler-output lane: choose the next honest native runtime
  integration slice: target-data-layout-aware helper signatures, boxed scalar
  construction in generated LLVM, a linker command prototype that still rejects
  executable mode clearly, or a documented blocker if the current LLVM text
  backend cannot model C ABI helper calls safely. Selected candidate:
  Milestone 1044 adds explicit target-pointer-width rendering for the scalar
  echo native runtime helper probe, with committed 32-bit `usize` snapshot
  coverage while preserving the current host-width default. This does not emit
  helper calls from normal `phpc compile --emit-ir`, link native executables, or
  claim boxed strings, arrays, objects, references/copy-on-write, diagnostics,
  or WordPress host/runtime state in native lowering.

## Milestone 638: WordPress Inventory Snapshot Harness

- [x] Tests/docs/compatibility lane: add a committed WordPress inventory output
  policy and deterministic synthetic harness without vendoring WordPress core.
  Added normalized `tools/wordpress-inventory.sh --normalize` output, a
  synthetic WordPress-shaped CLI test, and fixture policy for the pinned
  WordPress 6.9.4 external-source target.

## Milestone 639: WordPress External Inventory Snapshot

- [ ] Tests/docs/compatibility lane: run the normalized inventory against an
  operator-supplied WordPress 6.9.4 checkout, review the counts and first
  bootstrap blocker, and commit a real external-source snapshot only if the
  output is stable and the repository still does not need to vendor core.

## Milestone 640: Interpreter Object Handle Identity

- [x] Runtime/object lane: move current interpreter object values from inline
  cloneable payloads to cloneable process-local handles so supported value
  copies preserve shared property slots and object identity. Added strict object
  identity, `spl_object_id`, current-subset `spl_object_hash`, and tests for
  assignment, function argument/return, array, and foreach object-handle copies.

## Milestone 641: Public Instance Method Dispatch

- [x] Parser/runtime/object lane: add the next executable object slice:
  static-name public instance method calls with scoped `$this` binding and
  native lowering rejection, while keeping constructors, inheritance, dynamic
  method names, visibility context, magic methods, and references explicit.

## Milestone 642: Object Constructor Dispatch

- [x] Runtime/object lane: add the next constructor slice for
  `new ClassName(...)` with declared public `__construct` execution and scoped
  `$this`, while keeping promoted properties, inheritance/parent constructors,
  non-public constructor visibility, named arguments, references/copy-on-write,
  exact native error objects, and native lowering explicit.

## Milestone 643: Non-public Method and Constructor Visibility Boundary

- [x] Runtime/object lane: add the next honest visibility slice: either execute
  private methods/constructors only from same-class `$this` context with tests,
  or introduce a sharper runtime call-context model and keep non-public access
  rejected with more precise diagnostics. Added same-class private instance
  method dispatch with runtime class-context tracking, while protected lookup,
  private constructors without an in-class construction surface, inheritance,
  traits, magic methods, references/copy-on-write, exact native error objects,
  and native lowering remain explicit.

## Milestone 644: Protected Visibility and Inheritance Prerequisite

- [x] Runtime/object lane: choose the next smallest honest visibility step:
  implement single-parent `extends` metadata and protected same-class/child
  method lookup, or document the exact inheritance/call-context blocker with
  sharper diagnostics. Implemented single-parent class metadata, inherited
  instance method lookup, protected same-class/child method dispatch, and
  parent-aware `is_a`, `is_subclass_of`, `get_parent_class`, `method_exists`,
  and `get_class_methods`. Milestone 645 later added inherited public
  property slots; parent constructors, non-public inherited property
  visibility, trait composition, static dispatch, magic methods,
  references/copy-on-write, exact native error objects, and native lowering
  explicit.

## Milestone 645: Inherited Property Layout Boundary

- [x] Runtime/object lane: add the next honest inheritance slice for inherited
  public property slots and property metadata lookup on child objects, or
  document the object-layout blocker before parent constructors and `parent::`
  calls. Added inherited public instance slots on child objects, parent-aware
  `property_exists`, `get_class_vars`, and `get_object_vars`, and PHP
  comparison fixture coverage. Kept non-public inherited property slots,
  constructor inheritance, property overrides/conflicts, trait composition,
  static dispatch, magic methods, references/copy-on-write, exact native error
  objects, and native lowering explicit.

## Milestone 646: Parent Constructor and Parent Method Boundary

- [x] Runtime/object lane: choose the next smallest parent-call slice:
  inherited public constructor lookup, `parent::__construct`, or
  `parent::method()` parsing/dispatch if feasible. Implemented inherited
  public constructor lookup and dispatch for `new Child(...)` when the child
  has no constructor, with `$this` bound to the child object and method context
  from the declaring parent. Kept explicit `parent::__construct`,
  `parent::method()`, non-public constructor visibility, static
  properties/methods, property override compatibility, trait composition,
  magic methods, references/copy-on-write, exact native error objects, and
  native lowering explicit.

## Milestone 647: Static Parent Receiver Boundary

- [x] Runtime/object lane: add the next smallest parent-call slice by parsing
  and diagnosing or executing `parent::__construct`/`parent::method()` with
  correct class context. Implemented explicit parent method calls in active
  instance method/constructor context, reusing the current `$this` object,
  resolving through the current class's parent chain, and preserving public
  and protected visibility checks. Kept parent calls outside instance context,
  parent calls without a parent class, private/static parent methods, parent
  static property/constant access, `self::`/`static::`, broader static
  properties/methods, late static binding, property override compatibility,
  trait composition, magic methods, references/copy-on-write, exact native
  error objects, and native lowering explicit.

## Milestone 648: Parent Static Receiver Refinement

- [x] Runtime/object lane: choose the next smallest parent/static receiver
  refinement: static parent method diagnostics, parent static property/constant
  parse diagnostics, `self::` class-context design, or a focused visibility
  compatibility improvement for explicit parent calls. Added distinct parse
  diagnostics and CLI fixture coverage for parent static property access,
  parent class constants, and `parent::class`, while keeping
  `parent::method(...)` as the only supported parent receiver slice. Kept
  static storage, late static binding, `self::`/`static::`, property override
  compatibility, trait composition, magic methods, references/copy-on-write,
  exact native error objects, and native lowering explicit.

## Milestone 649: Self Receiver Boundary

- [x] Runtime/object lane: choose the next smallest `self::` or static member
  boundary: parse and reject `self::method(...)` with class-context-aware
  diagnostics, implement a narrow same-class instance call if feasible, or
  document why static/self dispatch needs a broader storage/design step first.
  Implemented narrow `self::method(...)` instance-context dispatch with current
  `$this`, current class/inherited method lookup, public/protected/private
  visibility checks, and explicit native rejection. Added distinct diagnostics
  for unsupported self static properties, self class constants, and
  `self::class`. Kept static properties, static methods, class constants,
  `static::`, late static binding, property override compatibility, trait
  composition, magic methods, references/copy-on-write, exact native error
  objects, and native lowering explicit.

## Milestone 650: Static Receiver Continuation

- [x] Runtime/object lane: choose the next smallest static receiver slice:
  static method diagnostics through `self::`/`parent::`, a `static::` parse
  refinement, same-class protected/private constructor visibility, or a
  documented blocker for static storage and late static binding. Implemented
  protected constructor calls from same-class or child-class method context
  through ordinary `new ClassName(...)` expressions and split private vs
  protected constructor diagnostics. Kept private constructors outside
  same-class construction context, protected constructors outside
  same-class/child-class construction context, static properties, static
  methods, class constants, property override compatibility, trait
  composition, magic methods, references/copy-on-write, exact native error
  objects, and native lowering explicit.

## Milestone 651: Static Receiver Continuation

- [x] Parser/static-receiver lane: choose the next smallest static receiver or object
  visibility slice: static method diagnostics through `self::`/`parent::`, a
  `static::` parse refinement, private constructor same-class construction
  surface, or a documented blocker for static storage and late static binding.
  Implemented the `static::` parse refinement: `static::$prop`,
  `static::method(...)`, `static::CONST`, and `static::class` now produce
  distinct stable diagnostics without adding late static binding, static
  method dispatch, static property storage, class constants, property override
  compatibility, trait composition, magic methods, references/copy-on-write,
  exact native error objects, or native lowering.

## Milestone 652: Object Visibility Continuation

- [x] Runtime/object lane: choose the next smallest reachable visibility slice:
  same-class private/protected property read/write from active instance method
  context, private constructor same-class construction only if a reachable
  execution surface exists, or a documented blocker for non-public inherited
  property slots. Implemented plain reads and direct writes for exact-class
  private/protected property slots while executing a same-class instance
  method, including same-class peer objects. Kept inherited non-public
  property storage, child-context protected property access, non-public
  property `isset`/`empty`/compound/increment/null-coalescing forms, static
  properties, class constants, property override compatibility, trait
  composition, magic methods, references/copy-on-write, exact native error
  objects, and native lowering explicit.

## Milestone 653: Object Visibility Continuation

- [x] Runtime/object lane: choose the next smallest object visibility slice:
  protected property access from child method context if inherited non-public
  slots are modeled first, non-public property `isset`/`empty` context for the
  exact-class subset, compound/increment read-modify-write for same-class
  non-public properties, or a documented blocker for property declaration
  ownership metadata. Implemented direct `isset($object->property)` and
  `empty($object->property)` for exact-class private/protected slots in active
  same-class method context, including same-class peer objects. Kept inherited
  non-public property slots, child-context protected property access,
  compound/increment/null-coalescing forms, static properties, class constants,
  property override compatibility, trait composition, magic methods,
  references/copy-on-write, exact native error objects, and native lowering
  explicit.

## Milestone 654: Object Visibility Continuation

- [x] Runtime/object lane: choose the next smallest object property slice:
  compound assignment or increment/decrement read-modify-write for same-class
  non-public properties, property declaration ownership metadata needed for
  inherited non-public slots, or a documented blocker for child protected
  property visibility. Implemented compound assignment and pre/post
  increment/decrement for exact-class private/protected property slots in
  active same-class method context, including same-class peer objects. Kept
  inherited non-public property slots, child-context protected property
  access, non-public null-coalescing forms, static properties, class
  constants, property override compatibility, trait composition, magic
  methods, references/copy-on-write, exact native error objects, and native
  lowering explicit.

## Milestone 655: Object Visibility Continuation

- [x] Runtime/object lane: choose the next smallest object property slice:
  same-class non-public null coalescing / `??=` forms, property declaration
  ownership metadata needed for inherited non-public slots, or a documented
  blocker for child protected property visibility. Implemented direct
  null-coalescing and null-coalescing assignment for exact-class
  private/protected property slots in active same-class method context,
  including same-class peer objects. Kept inherited non-public property
  slots, child-context protected property access, static properties, class
  constants, property override compatibility, trait composition, magic
  methods, references/copy-on-write, exact native error objects, and native
  lowering explicit.

## Milestone 656: Object Visibility Continuation

- [x] Runtime/object lane: choose the next object-model ownership slice:
  property declaration ownership metadata needed for inherited non-public
  slots, child protected property visibility, or a documented blocker if the
  current flat object slot representation must change first. Implemented
  inherited non-public instance property slots with declaring class id/name
  ownership. Parent-declared methods now read/write parent private/protected
  slots on child objects, and `get_mangled_object_vars`/debug output use the
  declaring class name for private inherited keys. Kept child protected
  property visibility, static properties, class constants, property override
  compatibility, trait composition, magic methods, references/copy-on-write,
  exact native error objects, and native lowering explicit.

## Milestone 657: Object Visibility Continuation

- [x] Runtime/object lane: choose the next property visibility slice:
  child-context protected property access for parent-declared protected slots,
  property override compatibility/conflict diagnostics, or a documented
  blocker if protected visibility needs broader class relationship metadata.
  Implemented child-context protected property access by passing the active
  class plus ancestors into runtime property access. Private slots still
  require exact declaring-class context, while protected slots are visible from
  the declaring class or a child-class method context across reads, writes,
  `isset`/`empty`, read-modify-write, and null-coalescing forms. Kept property
  override compatibility/conflict diagnostics, static properties, class
  constants, trait composition, magic methods, references/copy-on-write, exact
  native error objects, and native lowering explicit.

## Milestone 658: Object Visibility Continuation

- [x] Runtime/object lane: choose the next object compatibility slice:
  property override compatibility/conflict diagnostics, duplicate inherited
  public/protected slot behavior, or a documented blocker if exact PHP
  property layout requires a broader class-declaration validation pass.
  Implemented inherited property redeclaration validation for the current
  untyped property subset: private parent properties may be redeclared as
  separate child slots, inherited public/protected properties reject staticness
  changes and visibility reduction, and otherwise-compatible non-private
  redeclarations were left for the following shared-slot layout milestone.
  Kept static properties, class constants, trait composition, magic methods,
  references/copy-on-write, exact native error objects, and native lowering
  explicit.

## Milestone 659: Object Visibility Continuation

- [x] Runtime/object lane: choose the next object layout slice: implement
  shared-slot layout for compatible non-private property redeclarations,
  static property storage diagnostics/execution, or a documented blocker if
  typed/default property metadata must land first. Implemented shared-slot
  layout for compatible non-private property redeclarations by collapsing
  inherited public/protected slots during object allocation and updating the
  effective visibility from compatible child declarations. Parent and child
  methods now see the same slot, public redeclarations expose the shared slot
  publicly, and private parent redeclarations remain separate child slots.
  Kept typed/default property compatibility, static properties, class
  constants, trait composition, magic methods, references/copy-on-write, exact
  native error objects, and native lowering explicit.

## Milestone 660: Object Visibility Continuation

- [x] Runtime/object lane: choose the next static/class-name slice:
  `ClassName::class`, `self::class`, `parent::class`, static property storage
  diagnostics/execution, real class constants, or a documented blocker if
  typed/default property metadata must land first. Implemented narrow
  class-name constant resolution: named receivers return the source-spelled
  class string without requiring class metadata, `self::class` resolves to the
  active declaring class, and `parent::class` resolves to that class's
  immediate parent during instance method/constructor execution. Kept
  `static::class`, static properties, static methods, real class constants,
  trait composition, magic methods, references/copy-on-write, exact native
  error objects, and native lowering explicit.

## Milestone 661: Object Visibility Continuation

- [x] Runtime/object lane: add narrow class constant declarations and lookup.
  Implemented class metadata for constants, `const NAME = value;` and
  `public|protected|private const NAME = value;` for the current
  constant-expression subset, inherited case-sensitive lookup through
  `ClassName::CONST`, `self::CONST`, and `parent::CONST`, visibility checks,
  CLI fixtures, and native codegen rejection. Kept typed constants, multiple
  constants in one declaration, `static::CONST`, dynamic
  `constant("Class::CONST")`/`defined("Class::CONST")`, static properties,
  static methods, late static binding, trait composition, magic methods,
  references/copy-on-write, exact native error objects, and native lowering
  explicit.

## Milestone 662: Object Static Continuation

- [x] Runtime/object lane: add narrow static property storage and access.
  Implemented class-level storage for untyped/no-default static properties
  initialized to `null`, plus case-sensitive `ClassName::$prop`,
  `self::$prop`, and `parent::$prop` reads and direct writes with inherited
  slot lookup and current visibility checks. Kept `static::$prop`, static
  methods, typed/default static properties, dynamic static property names,
  late static binding, trait composition, magic methods,
  references/copy-on-write, exact native error objects, and native lowering
  explicit.

## Milestone 663: Object Static Continuation

- [x] Runtime/object lane: implement the next static-property mutation slice.
  Added compound assignment, pre/post increment/decrement, and `??=` for
  declared untyped/no-default static properties through `ClassName::$prop`,
  `self::$prop`, and `parent::$prop`, including inherited declaring-class
  storage, current visibility checks, expression results, C-style `for`
  expression coverage, CLI fixture coverage, system PHP comparison, and native
  mutation rejection tests. Kept `static::$prop`, static methods,
  typed/default static properties, dynamic static property names,
  late static binding, trait composition, magic methods,
  references/copy-on-write, exact native error objects, and native lowering
  explicit.

## Milestone 664: Object Static Continuation

- [x] Runtime/object lane: implement static-property `isset`/`empty`/`??`.
  Added null-aware static property operands for declared untyped/no-default
  static properties through `ClassName::$prop`, `self::$prop`, and
  `parent::$prop`, including inherited declaring-class storage, current
  visibility checks, fallback behavior for missing declared property names,
  CLI fixture coverage, system PHP comparison for the public/missing subset,
  and native rejection coverage for `isset`, `empty`, and `??`. Kept
  `static::$prop`, static methods, typed/default static properties, dynamic
  static property names, static-property `unset`, late static binding, trait
  composition, magic methods, references/copy-on-write, exact native error
  objects, and native lowering explicit.

## Milestone 665: Object Static Continuation

- [x] Runtime/object lane: add the static-property `unset(...)` boundary.
  Parsed `unset(ClassName::$prop)`, `unset(self::$prop)`, and
  `unset(parent::$prop)` now report stable runtime diagnostics matching the
  PHP-forbidden operation instead of removing static storage, with CLI fixture
  coverage and native mutation rejection coverage. Kept storage-removing
  static-property unset, `static::$prop`, static methods, typed/default static
  properties, dynamic static property names, late static binding, trait
  composition, magic methods, references/copy-on-write, exact native error
  objects, and native lowering explicit.

## Milestone 666: Object Static Continuation

- [x] Runtime/object lane: add the named static-method call boundary. Parsed
  `ClassName::method(...)` now reaches runtime, resolves the named class and
  declared/inherited method metadata, and reports stable diagnostics before
  argument evaluation or executable static dispatch. Missing classes and
  missing methods reuse existing stable diagnostics, and native lowering
  rejects the parsed form through the object/class boundary. Kept executable
  static method dispatch, `static::method(...)`, late static binding, trait
  composition, magic methods, references/copy-on-write, exact native error
  objects, and native lowering explicit.

## Milestone 667: Object Static Continuation

- [x] Runtime/object lane: implement the first named static-method dispatch
  slice. `ClassName::method(...)` now executes declared or inherited visible
  static methods through `phpc run`, evaluates positional arguments after
  metadata and arity checks, runs without `$this`, and preserves the declaring
  class as the active class context for current `self::class` and static
  property access. Kept static dispatch through `self::`, `parent::`, object
  receivers, and late-bound `static::method(...)`, broader class constant
  semantics, typed/default static property metadata, trait composition, magic
  methods, references/copy-on-write, exact native error objects, and native
  lowering explicit.

## Milestone 668: Object Static Continuation

- [x] Runtime/object lane: implement `self::`/`parent::` static method
  dispatch prerequisites. `self::method(...)` and `parent::method(...)` now
  execute resolved visible static methods from active class context without
  binding `$this`, preserve the resolved declaring class as active class
  context, and keep non-static calls without current `$this` as stable runtime
  diagnostics. Kept object-receiver static method dispatch, late-bound
  `static::method(...)`, late static binding, trait composition, magic
  methods, references/copy-on-write, exact native error objects, and native
  lowering explicit.

## Milestone 669: Object Static Continuation

- [x] Runtime/object lane: implement untyped static property defaults for the
  current constant-expression subset. Declared static properties can now use
  default values such as scalars and constant-expression arithmetic, with
  class-level storage initialized before execution; instance property defaults
  and typed static properties remain explicit unsupported boundaries.

## Milestone 670: Object Static Continuation

- [x] Runtime/object lane: add called-class context prerequisites for late
  static binding. `get_called_class()` and `static::class` now read a tracked
  called-class context in current instance and static method calls, including
  forwarding through `self::` and `parent::`; outside method/static class
  context they keep stable runtime diagnostics. Kept late-bound
  `static::method(...)`, `static::$prop`, `static::CONST`, typed static
  property metadata, trait composition, magic methods, references/copy-on-write,
  exact native error objects, and native lowering explicit.

## Milestone 671: Object Static Continuation

- [x] Runtime/object lane: implement the narrow `static::method(...)` dispatch
  slice. Late static method calls now resolve visible static methods through
  the active called class, preserve called-class context through nested
  `static::`, `self::`, and `parent::` calls, and keep top-level
  `static::method(...)` plus non-static method targets as stable runtime
  diagnostics. Kept object-receiver static method dispatch, late-bound
  `static::$prop`, `static::CONST`, typed static property metadata, trait
  composition, magic methods, references/copy-on-write, exact native error
  objects, and native lowering explicit.

## Milestone 672: Object Static Continuation

- [x] Runtime/object lane: implement the narrow `static::$prop` late-bound
  static property slice. Late static property reads, direct writes, compound
  assignment, pre/post increment/decrement, `isset`, `empty`, `??`, and `??=`
  now resolve through the active called class and reuse current static-property
  storage/visibility rules. Top-level `static::$prop` and PHP-forbidden
  `unset(static::$prop)` keep stable runtime diagnostics. Kept `static::CONST`,
  typed static property metadata, trait composition, magic methods,
  references/copy-on-write, exact native error objects, and native lowering
  explicit.

## Milestone 673: Object Static Continuation

- [x] Runtime/object lane: implement the narrow `static::CONST` late-bound
  class constant slice. Late static class constant reads now resolve through
  the active called class, reuse current inherited constant lookup and
  visibility rules, and keep top-level `static::CONST` as a stable runtime
  diagnostic. Kept typed/static/multi-declarator class constants, dynamic
  `constant("Class::CONST")`/`defined("Class::CONST")`, trait composition,
  magic methods, references/copy-on-write, exact native error objects, and
  native lowering explicit.

## Milestone 674: Object Static Continuation

- [x] Runtime/object lane: add a distinct typed static property declaration
  boundary before typed static property execution exists. Static property type
  declarations now fail with a stable diagnostic naming the missing typed
  metadata, uninitialized state, and write enforcement pieces instead of the
  generic instance typed-property boundary. Kept typed static property storage,
  default validation, write enforcement, inheritance compatibility, trait
  composition, magic methods, references/copy-on-write, exact native error
  objects, and native lowering explicit.

## Milestone 675: Object Static Continuation

- [x] Runtime/object lane: implement the narrow object-receiver static method
  slice. `$object::method(...)` now resolves visible static methods through the
  receiver object's class, executes without `$this`, and preserves that object
  class as the called-class context for `static::` and `get_called_class()`.
  Kept dynamic class-string static method receivers, object receiver static
  properties/constants, non-static object static dispatch, trait composition,
  magic methods, references/copy-on-write, exact native error objects, and
  native lowering explicit.

## Milestone 676: Object Static Continuation

- [x] Runtime/object lane: implement dynamic class-string static method
  receivers. `$className::method(...)` now accepts declared class-name strings,
  resolves visible static methods through that class, executes without `$this`,
  and preserves the receiver class as called-class context. Kept object
  receiver static properties/constants, non-static dynamic static dispatch,
  broader class constant semantics, typed static property metadata, trait
  composition, magic methods, references/copy-on-write, exact native error
  objects, and native lowering explicit.

## Milestone 677: Object Static / WordPress Bridge Continuation

- [x] Runtime/WordPress bridge lane: implement the first narrow
  `require path;` execution slice. `require` statements now accept paths that
  evaluate to strings, including constant/string concatenation such as
  `ABSPATH . WPINC . '/load.php'`, resolve local paths relative to the current
  source file, register top-level functions/classes from required files, and
  execute included statements in caller scope. Kept `include`, `include_once`,
  expression-form `require`, expression-form `require_once`, include-path
  lookup, stream/URL paths, exact include return values, autoload/opcache
  behavior, exact PHP warning/fatal recovery, and native lowering explicit.

## Milestone 678: WordPress Bridge Continuation

- [x] Runtime/WordPress bridge lane: run the normalized WordPress inventory
  against a WordPress 6.9.4 checkout after the narrow `require` slice. The
  external run reported 1288 PHP files and moved the first bootstrap blocker to
  `wp-settings.php:53:1`, statement-form `require_once`.

## Milestone 679: WordPress Bridge Continuation

- [x] Runtime/WordPress bridge lane: implement narrow statement-form
  `require_once path;` for local string paths with resolved-file
  de-duplication. Keep expression-form `require_once`, `include`,
  `include_once`, include-path lookup, streams/URLs, exact include return
  values, declaration-order dependencies, exact warning/fatal recovery, and
  native lowering explicit. The follow-up external WordPress inventory now
  reaches `wp-settings.php:100:2`, statement-form `include`.

## Milestone 680: WordPress Bridge Continuation

- [x] Runtime/WordPress bridge lane: implement narrow statement-form
  `include path;` for existing local files, enough to parse and skip the
  false-by-default optional `advanced-cache.php` branch. Keep expression-form
  `include`, `include_once`, missing-file include warning/recovery,
  include-path lookup, streams/URLs, exact include return values,
  declaration-order dependencies, exact warning/fatal recovery, and native
  lowering explicit. The follow-up external WordPress inventory now reaches
  `wp-settings.php:471:2`, statement-form `include_once $mu_plugin`.

## Milestone 681: WordPress Bridge Continuation

- [x] Runtime/WordPress bridge lane: implement statement-form
  `include_once path;`, first for the must-use plugin loop shape. It reuses the
  resolved-file de-duplication table, covers variable path values in loops, and
  keeps expression-form `include_once`, include-path lookup, streams/URLs,
  missing optional include warning/recovery, exact include return values, and
  native lowering explicit. The follow-up external WordPress inventory now
  reaches `wp-settings.php:33:1`, top-level `global ...;`.

## Milestone 682: WordPress Bridge Continuation

- [x] Runtime/WordPress bridge lane: implement top-level `global ...;`
  declarations as no-op/import-compatible statements so WordPress bootstrap can
  pass the early version-variable import statement in `wp-settings.php`. Keep
  function-scope `global`, reference binding to `$GLOBALS`, superglobal
  semantics, exact warning behavior, and native lowering explicit. The
  follow-up external WordPress inventory now reaches `wp-settings.php:34:9`,
  undefined constant `ABSPATH`, because the probe runs `wp-settings.php`
  directly.

## Milestone 683: WordPress Bootstrap Probe Continuation

- [x] Tests/docs/compatibility lane: make the WordPress inventory probe honest
  about entrypoint assumptions by adding a normalized bootstrap-shim probe while
  keeping the direct `wp-settings.php` probe visible. WordPress core remains
  unvendored, normalized output now hides the temporary shim path, and the real
  WordPress 6.9.4 shim result reaches
  `wp-includes/compat-utf8.php:47:25`, unsupported parameter type declarations.

## Milestone 684: WordPress Bridge Continuation

- [x] Parser/runtime lane: accept syntax-only parameter/return type
  declarations and reference parameter declarations enough to register
  WordPress's early `compat-utf8.php` helper signatures. Runtime invocation of
  typed functions and by-reference parameter functions remains rejected with
  stable diagnostics, keeping type enforcement, coercion, reference binding,
  exact `TypeError` behavior, reflection metadata, and native lowering explicit.
  The follow-up bootstrap-shim inventory now reaches
  `wp-includes/compat-utf8.php:130:16`, where hexadecimal integer literals such
  as `0xC2` are not yet lexed.

## Milestone 685: WordPress Bridge Continuation

- [x] Parser/runtime lane: implement hexadecimal integer literals over the
  current integer value subset so WordPress's early UTF-8 scanner body can
  parse, including fixture comparison against system PHP and explicit gaps for
  overflow behavior, binary/octal literal variants, numeric string coercion
  interactions, and native lowering. The follow-up bootstrap-shim inventory now
  reaches `wp-includes/compat-utf8.php:140:4`, `goto invalid_utf8;`.

## Milestone 686: WordPress Bridge Continuation

- [x] Parser/runtime lane: implement bounded `goto` statements and labels
  enough to parse WordPress's early UTF-8 scanner error path, including tests
  for active statement-list jumps and explicit gaps for exact PHP target
  validation, duplicate label diagnostics, jumps into nested blocks,
  cross-function jumps, included-file label boundaries, `finally` interaction,
  and native lowering.

## Milestone 687: WordPress Bridge Continuation

- [x] Parser/runtime lane: implement bounded `(string)` cast expressions for
  WordPress's `_wp_utf8_encode_fallback()` and `_wp_iso_8859_1_to_utf8()` paths,
  with tests, CLI coverage, docs, and named unsupported edges for
  array-to-string warning recovery, object `__toString()` and cast error
  behavior, resources, non-string casts, exact PHP diagnostics, and native
  lowering.

## Milestone 688: WordPress Bridge Continuation

- [x] Parser/runtime lane: implement bounded function-local `static` variable
  declarations enough for WordPress's `_wp_can_use_pcre_u()` path, with tests,
  CLI coverage, docs, and named unsupported edges for dynamic initialization,
  references, variable variables, recursion/reentrancy edge behavior,
  included-file behavior, exact PHP diagnostics, reflection behavior, and native
  lowering.

## Milestone 689: WordPress Bridge Continuation

- [x] Parser/runtime lane: implement or explicitly bound anonymous closures and
  `use` captures enough for WordPress's `_wp_can_use_pcre_u()` error-handler
  path, with tests, CLI coverage, docs, and named unsupported edges for
  by-reference capture semantics, callback invocation, exact PHP diagnostics,
  and native lowering.

## Milestone 690: WordPress Bridge Continuation

- [x] Parser/runtime lane: implement or explicitly bound alternate
  `if`/`elseif`/`else` colon/`endif` syntax enough for the current WordPress
  bootstrap shim, with tests, CLI coverage, docs, and named unsupported edges
  for exact PHP diagnostics, nested alternate syntax edge cases, mixed
  brace/colon recovery, source mapping, and native lowering.

## Milestone 691: WordPress Bridge Continuation

- [x] Parser/runtime lane: implement or explicitly bound `instanceof` enough
  for WordPress's `Countable` checks in `wp-includes/compat.php`, with tests,
  CLI coverage, docs, and named unsupported edges for class/interface
  relationship breadth, autoload behavior, namespace-aware class names, exact
  PHP diagnostics, and native lowering.

## Milestone 692: WordPress Bridge Continuation

- [x] Runtime/builtin lane: implement or explicitly bound `extension_loaded()`
  for WordPress's early compatibility probes, with tests, CLI coverage, docs,
  and named unsupported edges for exact extension inventory policy, case
  normalization, host PHP/module discovery, side effects, exact PHP
  diagnostics, and native lowering.

## Milestone 693: WordPress Bridge Continuation

- [x] Runtime/constants lane: implement or explicitly bound `PHP_VERSION_ID` as
  a built-in constant for WordPress's sodium compatibility loader, with tests,
  CLI coverage, docs, and named unsupported edges for PHP-version target
  policy, host/runtime version coupling, exact constant catalog behavior, and
  native lowering.

## Milestone 694: WordPress Bridge Continuation

- [x] Runtime/path builtin lane: implement or explicitly bound `dirname()` for
  WordPress's bootstrap path construction, with tests, CLI coverage, docs, and
  named unsupported edges for path normalization policy, Windows paths, stream
  wrappers, exact PHP diagnostics, and native lowering.

## Milestone 695: WordPress Bridge Continuation

- [x] Runtime/autoload lane: implement or explicitly bound
  `spl_autoload_register()` for WordPress's sodium compat loader, with tests,
  CLI coverage, docs, and named unsupported edges for closure value/runtime
  semantics, autoload stack behavior, namespace/class resolution, exact PHP
  diagnostics, and native lowering.

## Milestone 696: WordPress Bridge Continuation

- [x] Syntax/attribute lane: implement or explicitly bound PHP attribute syntax
  for WordPress class declarations, with tests, CLI coverage, docs, and named
  unsupported edges for attribute metadata storage, reflection behavior,
  namespace resolution, constructor argument evaluation, exact PHP diagnostics,
  and native lowering.

## Milestone 697: WordPress Bridge Continuation

- [x] Runtime/exception lane: implement or explicitly bound `throw` statements
  for WordPress's sodium compat class, with tests, CLI coverage, docs, and
  named unsupported edges for exception object modeling, stack unwinding,
  catch/finally behavior, stack traces, exact PHP diagnostics, and native
  lowering.

## Milestone 698: WordPress Bridge Continuation

- [x] Runtime/exception lane: implement or explicitly bound
  `try`/`catch`/`finally` syntax for WordPress's sodium compat class, with
  tests, CLI coverage, docs, and named unsupported edges for exception object
  matching, stack unwinding, catch variable binding, finally execution,
  stack traces, partial-output behavior, exact PHP diagnostics, and native
  lowering.

## Milestone 699: WordPress Bridge Continuation

- [x] Runtime/cast lane: implement or explicitly bound the next WordPress
  bootstrap cast expression blocker beyond the current `(string)` slice, with
  tests, CLI coverage, docs, and named unsupported edges for exact PHP scalar
  cast warnings, object/array/resource behavior, partial-output behavior,
  exact PHP diagnostics, and native lowering.

## Milestone 700: WordPress Bridge Continuation

- [x] Runtime/assignment lane: implement or explicitly bound the next WordPress
  bootstrap array destructuring assignment blocker, with tests, CLI coverage,
  docs, and named unsupported edges for list assignment semantics,
  references/copy-on-write, nested destructuring, missing-key notices,
  exact PHP diagnostics, partial-output behavior, and native lowering.

## Milestone 701: WordPress Bridge Continuation

- [x] Runtime/cast lane: implement or explicitly bound the next WordPress
  bootstrap cast-expression blocker, `(bool)` at
  `wp-includes/sodium_compat/src/Compat.php:572`, with tests, CLI coverage,
  docs, and named unsupported edges for scalar/array/object/resource boolean
  cast behavior, exact PHP diagnostics, partial-output behavior, and native
  lowering.

## Milestone 702: WordPress Bridge Continuation

- [x] Parser/runtime lane: implement or explicitly bound the next WordPress
  bootstrap default-parameter blocker, a `self::CRYPTO_GENERICHASH_BYTES`
  class-constant default in
  `wp-includes/sodium_compat/src/Compat.php:1714`, with tests, CLI coverage,
  docs, and named unsupported edges for class-constant default evaluation,
  class context, inheritance/visibility, exact PHP diagnostics, partial-output
  behavior, and native lowering.

## Milestone 703: WordPress Bridge Continuation

- [x] Parser/runtime lane: implement or explicitly bound the next WordPress
  bootstrap nested class declaration blocker reported at
  `<bootstrap-shim>:7:5`, with tests, CLI coverage, docs, and named unsupported
  edges for conditional/nested declaration timing, redeclaration behavior,
  class table ordering, source mapping, exact PHP diagnostics, partial-output
  behavior, and native lowering.

## Milestone 704: WordPress Bridge Continuation

- [x] Runtime/object lane: implement or explicitly bound the next WordPress
  bootstrap class-metadata blocker, `undefined class Exception` at
  `<bootstrap-shim>:7:5`, with tests, CLI coverage, docs, and named unsupported
  edges for built-in exception class metadata, throwable inheritance,
  constructor behavior, stack traces, try/catch execution, fatal diagnostics,
  partial-output behavior, and native lowering.

## Milestone 705: WordPress Bridge Continuation

- [x] Parser/name-resolution lane: implement the bounded class-name namespace
  slice for the WordPress bootstrap namespace declaration blocker reported at
  `<bootstrap-shim>:2:1`. The parser now accepts one unbracketed named
  namespace per file, simple top-level class imports with optional aliases,
  and namespace/import resolution for class declarations plus class-like
  references in `extends`, `new`, `instanceof`, static members, and
  `ClassName::class`. Tests, fixtures, docs, and native rejection coverage
  name the remaining unsupported edges at that point: bracketed/global/multiple
  namespaces, namespace-scoped functions/constants, grouped/function/constant
  imports, namespace-qualified function calls, string-name import expansion,
  `__NAMESPACE__`, autoload interaction, exact PHP diagnostics,
  partial-output behavior, and native lowering. The synthetic WordPress shim
  now reaches the interface declaration blocker at `<bootstrap-shim>:4:1`.
- [x] Parser/runtime lane: implement the bounded declared-interface metadata
  slice for the WordPress bootstrap interface declaration blocker reported at
  `<bootstrap-shim>:4:1`. The parser now accepts top-level interface
  declarations and public method signatures, registers interface names in a
  class-like case-insensitive registry, keeps `class_exists()` class-only, and
  powers `interface_exists()` plus `get_declared_interfaces()` for declared
  user interfaces. Tests, fixtures, docs, and native rejection coverage name
  the remaining unsupported edges: interface inheritance, constants,
  non-public/static methods, `implements` clauses, implementation enforcement,
  built-in/internal interfaces, autoload interaction, exact PHP diagnostics,
  partial-output behavior, and native lowering. The synthetic WordPress shim
  now reaches the trait declaration blocker at `<bootstrap-shim>:5:1`.
- [x] Parser/runtime lane: implement the bounded declared-trait metadata slice
  for the WordPress bootstrap trait declaration blocker reported at
  `<bootstrap-shim>:5:1`. The parser now accepts empty top-level trait
  declarations, registers trait names in the class-like case-insensitive
  registry, powers `trait_exists()` plus `get_declared_traits()` for declared
  user traits, and keeps native lowering rejecting trait declarations before
  backend execution. Tests, fixtures, docs, and native rejection coverage name
  the remaining unsupported edges: trait members, class `use` composition,
  conflict resolution, aliasing, built-in/internal traits, autoload
  interaction, exact PHP diagnostics, partial-output behavior, and native
  lowering.
- [x] Parser/runtime lane: implement the bounded declared unit-enum metadata
  slice for the WordPress bootstrap enum declaration blocker reported at
  `<bootstrap-shim>:6:1`. The parser now accepts top-level unbacked enum
  declarations with bare `case Name;` members, registers enum names in the
  class-like case-insensitive registry, powers `enum_exists()`, reports
  declared enums through `class_exists()` and `get_declared_classes()`, and
  keeps native lowering rejecting enum declarations before backend execution.
  Tests, fixtures, docs, and native rejection coverage name the remaining
  unsupported edges: enum case objects/value access, backed enum values, enum
  methods/constants/properties, enum interface implementation, built-in/internal
  enums, autoload interaction, exact PHP diagnostics, partial-output behavior,
  and native lowering.
- [x] Parser/runtime lane: implement the bounded arrow-function syntax slice
  for the WordPress bootstrap arrow-function blocker reported at
  `<bootstrap-shim>:9:10`. The parser now accepts `fn (...) => expr` as a
  closure-shaped expression with a synthetic return body, reached arrow
  expressions fail at the explicit closure-value runtime boundary, native
  lowering rejects closure expressions with a closure-specific codegen
  boundary, and docs/tests name the remaining unsupported edges: closure
  values, capture binding, `$this` binding, callable invocation, callback
  integration, exact PHP diagnostics, partial-output behavior, and native
  lowering.
- [x] Parser/runtime lane: implement or explicitly bound the next WordPress
  bootstrap missing-parent class blocker reported at `<bootstrap-shim>:7:1`,
  with tests, CLI coverage, docs, and named unsupported edges for autoload,
  include ordering, exact PHP fatal behavior, partial-output behavior, and
  native lowering. The Milestone 710 slice proves namespaced single-parent
  metadata when the parent is already declared, keeps absent parents as a
  stable runtime boundary, updates the synthetic WordPress shim to declare the
  missing parent, and advances the shim to the reached `try` execution
  boundary at `<bootstrap-shim>:9:1`.
- [x] Parser/runtime lane: implement or explicitly bound the next WordPress
  bootstrap reached `try` blocker reported at `<bootstrap-shim>:9:1`, with
  tests, CLI coverage, docs, and named unsupported edges for exception objects,
  catch matching/binding, `finally`, partial-output behavior, exact PHP
  warning/fatal semantics, and native lowering. The Milestone 711 slice
  executes non-throwing try bodies, skips catches without a thrown exception,
  runs finally after normal try completion, keeps reached `throw` as the
  current runtime boundary, and advances the synthetic WordPress shim to the
  anonymous-closure value boundary at `<bootstrap-shim>:9:19`.
- [x] Parser/runtime lane: implement or explicitly bound the next WordPress
  bootstrap anonymous-closure value blocker reported at `<bootstrap-shim>:9:19`,
  with tests, CLI coverage, docs, and named unsupported edges for closure
  allocation, explicit captures, invocation, callable integration, `$this`
  binding, references/copy-on-write, exact PHP diagnostics, partial-output
  behavior, and native lowering. The Milestone 712 slice allocated inert
  no-capture anonymous closure values, kept explicit captures and invocation
  as stable runtime boundaries, kept arrow closure evaluation unsupported, and
  advanced the synthetic WordPress shim to the arrow closure value boundary at
  `<bootstrap-shim>:10:10` before Milestone 713.
- [x] Parser/runtime lane: implement or explicitly bound the next WordPress
  bootstrap arrow-closure value blocker reported at `<bootstrap-shim>:10:10`,
  with tests, CLI coverage, docs, and named unsupported edges for implicit
  capture binding, invocation, callable integration, `$this` binding, static
  closures, references/copy-on-write, exact PHP diagnostics, partial-output
  behavior, and native lowering. The Milestone 713 slice allocates inert arrow
  closure values, keeps invocation/callback/capture execution unsupported, and
  advances the synthetic WordPress bootstrap-shim probe to exit 0 with no
  stderr.
- [x] Parser/runtime lane: implement the real WordPress 6.9.4 bootstrap-shim
  namespace-scoped function declaration blocker reported at
  `<bootstrap-shim>:23:5`, with tests, CLI coverage, docs, and named
  unsupported edges for function imports, qualified/fully-qualified function
  calls, namespace-scoped constants, exact PHP diagnostics, partial-output
  behavior, and native lowering. The Milestone 714 slice supports
  namespace-scoped function declarations and unqualified same-namespace calls,
  while the real inventory now reaches
  `runtime error at <bootstrap-shim>:997:6: unsupported call defined(): constant name must be a non-empty unqualified identifier in the current subset, got \Sodium\CRYPTO_AUTH_BYTES`.
- [x] Runtime/constants lane: implemented the real WordPress 6.9.4
  bootstrap-shim qualified `defined()` constant-name blocker at
  `<bootstrap-shim>:997:6` and the adjacent namespace-scoped `const`
  declarations reached by the sodium compatibility bootstrap. The Milestone
  715 slice supports qualified runtime constant names for `define()`,
  `defined()`, and `constant()`, resolves top-level `const NAME = value;`
  under the active unbracketed namespace, and keeps bare namespace constant
  fallback reads, class constants through `defined()`/`constant()`, full
  extension constant catalogs, host extension discovery/loading, exact PHP
  diagnostics, partial-output behavior, and native lowering unsupported. The
  real inventory now reports direct `wp-settings.php` still stops at
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`,
  while the bootstrap-shim probe reaches
  `runtime error at <bootstrap-shim>:68:9: undefined function assert()`.
- [x] Runtime/builtins lane: implemented the real WordPress 6.9.4
  bootstrap-shim `assert()` blocker at `<bootstrap-shim>:68:9`. The
  Milestone 716 slice accepts truthy runtime assertions with scalar/null
  descriptions as inert metadata, exposes `assert` through
  `function_exists()`/`is_callable()` and dynamic string calls, keeps failing
  assertions as a runtime boundary, and keeps native direct/dynamic
  `assert(...)` rejected. Assertion INI policy, callbacks, `AssertionError`,
  `Throwable` descriptions, exact warning/fatal behavior, PHP 8.3
  deprecations, partial-output behavior, and native lowering remain
  unsupported. The real inventory now reports direct `wp-settings.php` still
  stops at
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`,
  while the bootstrap-shim probe reaches
  `runtime error at <bootstrap-shim>:106:10: unsupported call defined(): constant name must be a non-empty supported identifier or qualified name in the current subset, got SODIUM_$constant`.
- [x] Runtime/constants lane: implemented the real WordPress
  6.9.4 bootstrap-shim dynamic constant-name probe
  `defined("SODIUM_$constant")` at `<bootstrap-shim>:106:10`. The Milestone
  717 slice adds simple double-quoted `$name` interpolation for runtime string
  names, keeps braced/complex interpolation, dynamic extension constant
  aliases, class-constant string lookup, full extension constant catalogs,
  exact PHP diagnostics, partial-output behavior, and native lowering
  unsupported, and advances the real bootstrap-shim probe to
  `lex error at <bootstrap-shim>:468:12: unsupported string interpolation: only simple $name interpolation in double-quoted strings is implemented; braced/complex interpolation is not implemented`.
- [x] Parser/runtime lane: implemented the real WordPress 6.9.4
  bootstrap-shim braced simple-variable interpolation blocker at
  `<bootstrap-shim>:468:12`, corresponding to
  `wp-includes/compat-utf8.php:468` and `$utf8 .= "{$byte1}{$byte2}";`. The
  Milestone 718 slice supports simple `{$name}` parts through the existing
  interpolated-string AST/runtime path, keeps complex braced interpolation,
  array offsets, object/static properties, variable variables, `${...}`,
  heredoc/nowdoc, exact PHP diagnostics, partial-output behavior, and native
  lowering unsupported, and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:106:41: unsupported call defined(): constant name must be a non-empty supported identifier or qualified name in the current subset, got ParagonIE_Sodium_Compat::LIBRARY_VERSION_MAJOR`.
- [x] Runtime/constants lane: implemented bounded class-constant
  string names for `defined($name)`/`constant($name)` in the real WordPress
  6.9.4 bootstrap-shim sodium compatibility guard, starting with
  `ParagonIE_Sodium_Compat::LIBRARY_VERSION_MAJOR` at
  `<bootstrap-shim>:106:41`. The Milestone 719 slice accepts
  `ClassName::CONST` and `\ClassName::CONST` runtime strings for declared
  class metadata, reports `defined(...)` true only for public constants, reuses
  existing visibility checks for `constant(...)`, keeps broader `self`,
  `parent`, and `static` string names, autoload-triggered discovery,
  enum/interface constants beyond current metadata, typed/static/multi
  declarators, exact PHP diagnostics, partial-output behavior, and native
  lowering unsupported, and advances the bootstrap-shim probe to
  `lex error at <bootstrap-shim>:665:56: unexpected character '@'`.
- [x] Parser/runtime lane: implemented bounded PHP error-control
  syntax `@expr` for the real WordPress 6.9.4 bootstrap-shim blocker at
  `<bootstrap-shim>:665:56`, corresponding to
  `wp-includes/sodium_compat/src/Core/Util.php:605` and
  `$c = (int) @($c & -1);`. The Milestone 720 slice tokenizes and parses
  `@expr` as an explicit AST wrapper, evaluates the operand normally through
  `phpc run`, keeps actual warning/notice/deprecation suppression,
  recoverable diagnostics, expression-specific recovery values, exact PHP
  warning/fatal behavior, partial-output behavior, and native lowering
  unsupported, and advances the bootstrap-shim probe to
  `parse error at <bootstrap-shim>:1015:20: unsupported cast expression: only (string), (int), and (bool) casts are implemented`.
- [x] Parser/runtime lane: implemented bounded `(float)`/`(double)`
  casts for the real WordPress 6.9.4 bootstrap-shim blocker at
  `<bootstrap-shim>:1015:20`, likely corresponding to
  `wp-includes/sodium_compat/src/Core/Util.php:255` and
  `$mixedVar = (float) $mixedVar;`. The Milestone 721 slice converts the
  current scalar/null subset, keeps array/object/resource casts,
  leading-numeric warning behavior, non-finite values, exact PHP diagnostics,
  partial-output behavior, and native lowering unsupported, and reveals that
  the current bootstrap-shim cast blocker is now `(array)` at
  `wp-includes/sodium_compat/src/PHP52/SplFixedArray.php:47`.
- [x] Parser/runtime lane: implement or explicitly bound `(array)` casts for
  the real WordPress 6.9.4 bootstrap-shim blocker at
  `<bootstrap-shim>:1015:20`, corresponding to
  `wp-includes/sodium_compat/src/PHP52/SplFixedArray.php:47` and
  `return (array) $this->internalArray;`, with tests, CLI coverage, docs, and
  named unsupported edges for object property materialization/mangling,
  scalar-to-array rules, resources, references/copy-on-write, exact PHP
  diagnostics, partial-output behavior, and native lowering. The Milestone 722
  slice supports null/scalar/array casts, keeps object/Closure/resource-heavy
  behavior explicit, and advances the real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:1075:43: unsupported assignment expression target: only direct static variables, direct array offsets, direct append offsets, and direct object properties are implemented; nested targets are not implemented`.
- [x] Parser/runtime lane: implement or explicitly bound nested assignment
  expression targets such as multi-level array offsets for the real WordPress
  6.9.4 bootstrap-shim blocker at `<bootstrap-shim>:1075:43`, with tests,
  CLI coverage, docs, and named unsupported edges for references,
  copy-on-write, append-at-depth, missing-container materialization, evaluation
  order, object/ArrayAccess targets, exact PHP diagnostics, partial-output
  behavior, and native lowering. The Milestone 723 slice supports
  direct-variable nested array-offset assignment expressions, keeps
  append-at-depth and nested read-modify-write forms explicit, and advances the
  real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:1324:9: unsupported clone expression: object handle copying and __clone dispatch are not implemented`.
- [x] Parser/runtime lane: implement or explicitly bound `clone` expressions
  for the real WordPress 6.9.4 bootstrap-shim blocker at
  `<bootstrap-shim>:1324:9`, with tests, CLI coverage, docs, and named
  unsupported edges for `__clone` dispatch, private/protected clone methods,
  object handle identity, nested object/reference properties,
  references/copy-on-write, exact PHP diagnostics, partial-output behavior, and
  native lowering. The Milestone 724 slice supports bounded `clone $object`
  expressions for objects without declared `__clone` methods, keeps `__clone`
  and references/copy-on-write explicit, and advances the real bootstrap-shim
  probe to
  `parse error at <bootstrap-shim>:1610:6: unsupported break: loop-depth arguments are not implemented; only 'break;' for the innermost loop is supported`.
- [x] Parser/runtime lane: implement or explicitly bound loop-depth
  `break N;`/`continue N;` control flow for the real WordPress 6.9.4
  bootstrap-shim blocker at `<bootstrap-shim>:1610:6`, corresponding to
  `wp-includes/load.php:1610` and `break 2;`, with tests, CLI coverage, docs,
  and named unsupported edges for invalid depths, switch/loop stack behavior,
  `continue N`, exact PHP diagnostics, partial-output behavior, and native
  lowering. The Milestone 725 slice supports positive integer literal depths,
  keeps dynamic/invalid depths and native lowering explicit, and advances the
  real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:158:2: unsupported global declaration: importing globals into function scope is not implemented`.
- [x] Runtime lane: implement or explicitly bound function-scope
  `global $name, ...;` imports for the real WordPress 6.9.4 bootstrap-shim
  blocker at `<bootstrap-shim>:158:2`, with tests, CLI coverage, docs, and
  named unsupported edges for references/aliasing, copy-on-write, dynamic
  variable names, unset interactions, included-file scope, exact PHP
  diagnostics, partial-output behavior, and native lowering. The Milestone 726
  slice supports direct variable imports through the shared root symbol table,
  materializes missing imported globals as `null`, treats `unset($name)` after
  import as dropping the local import without deleting the root value, keeps
  reference/COW semantics and native lowering explicit, and advances the real
  bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:160:17: undefined constant PHP_VERSION`.
- [x] Runtime lane: implement or explicitly bound the `PHP_VERSION` built-in
  compatibility constant for the real WordPress 6.9.4 bootstrap-shim blocker
  at `<bootstrap-shim>:160:17`, with tests, CLI coverage, docs, and named
  unsupported edges for PHP-version policy, related version constants,
  `phpversion()`/`version_compare()` behavior, extension versions, exact PHP
  diagnostics, partial-output behavior, and native lowering. The Milestone 727
  slice supports a deterministic PHP 8.3 `PHP_VERSION` string through bare
  reads, `constant(...)`, `defined(...)`, dynamic string-name lookup, and native
  `defined(...)` folding, keeps host/version catalog breadth explicit, and
  advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:162:7: undefined function version_compare()`.
- [x] Runtime/builtin lane: implement or explicitly bound `version_compare()`
  for the real WordPress 6.9.4 bootstrap-shim blocker at
  `<bootstrap-shim>:162:7`, with tests, CLI coverage, docs, and named
  unsupported edges for PHP's full version-string grammar, operator argument
  forms, invalid argument diagnostics, pre-release ordering, extension version
  coupling, partial-output behavior, and native lowering. The Milestone 728
  slice supports numeric version components and the common comparison
  operators, keeps full PHP version semantics and native lowering explicit, and
  advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:183:28: undefined function sprintf()`.
- [x] Runtime/string builtin lane: implement or explicitly bound `sprintf()`
  for the real WordPress 6.9.4 bootstrap-shim blocker at
  `<bootstrap-shim>:183:28`, with tests, CLI coverage, docs, and named
  unsupported edges for PHP's full format grammar, argument reordering,
  width/precision/star modifiers, locale behavior, array/object/resource
  conversions, warning behavior, partial-output behavior, and native lowering.
  The Milestone 729 slice supports string formats with literal text, `%%`,
  `%s`, and `%N$s`, keeps full PHP formatting semantics and native lowering
  explicit, and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:193:3: undefined function header()`.
- [x] Runtime/web-SAPI lane: implement or explicitly bound `header()` for the
  real WordPress 6.9.4 bootstrap-shim blocker at `<bootstrap-shim>:193:3`,
  with tests, CLI coverage, docs, and named unsupported edges for response
  header storage, status-code parsing, replacement behavior, header removal,
  output-sent warnings, SAPI/web-server integration, partial-output behavior,
  exact PHP diagnostics, and native lowering. The Milestone 730 slice accepts a
  string header line with optional bool replacement flag and integer response
  code as a no-op returning `null`, keeps real response/SAPI semantics and
  native lowering explicit, and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:195:8: undefined function implode()`.
- [x] Runtime/string-array builtin lane: implement or explicitly bound
  `implode()` for the real WordPress 6.9.4 bootstrap-shim blocker at
  `<bootstrap-shim>:195:8`, with tests, CLI coverage, docs, and named
  unsupported edges for argument-order overloads, non-string array values,
  nested arrays, object/resource conversions, exact warning behavior,
  partial-output behavior, and native lowering. The Milestone 731 slice joins
  current scalar/null array values with an empty default separator or string
  separator, keeps broader PHP conversion and native lowering explicit, and
  advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:196:3: undefined function exit()`.
- [x] Runtime/control builtin lane: implement or explicitly bound `exit()` /
  `die()` for the real WordPress 6.9.4 bootstrap-shim blocker at
  `<bootstrap-shim>:196:3`, with tests, CLI coverage, docs, and named
  unsupported edges for process termination semantics, integer exit-code
  handling, string message output, finally/destructor/shutdown-function
  behavior, partial-output behavior, and native lowering. The Milestone 732
  slice treats direct `exit()`/`die()` as a language-construct termination
  signal, keeps callable lookup false, supports omitted/null/int/string
  arguments in the current subset, and advances the real bootstrap-shim probe
  to WordPress' missing-extension guard: exit code `1`, 126 stdout bytes, and
  no stderr under the then-current empty `extension_loaded()` policy.
- [x] Runtime/extensions lane: replace the deterministic empty
  `extension_loaded()` registry with a bounded compatibility registry for the
  WordPress 6.9.4 bootstrap requirement checks, with tests, CLI coverage, docs,
  and named unsupported edges for host extension discovery, extension aliases,
  extension version APIs, native extension functions/constants, configuration,
  exact diagnostics, partial-output behavior, and native lowering. The
  Milestone 733 slice reports `json` and `hash` as loaded, keeps other
  extensions false, and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:203:8: undefined function file_exists()`.
- [x] Runtime/filesystem lane: implement or explicitly bound `file_exists()`
  for the real WordPress 6.9.4 bootstrap-shim blocker at `<bootstrap-shim>:203:8`,
  with tests, CLI coverage, docs, and named unsupported edges for filesystem
  metadata policy, path canonicalization, relative paths, stream wrappers,
  permissions, TOCTOU behavior, host filesystem coupling, partial-output
  behavior, and native lowering. The Milestone 734 slice accepts one string
  local path, rejects stream-wrapper paths, exposes the name through the current
  callable table, keeps direct native calls behind the function-call boundary,
  and advances the real bootstrap-shim probe to
  `lex error at <bootstrap-shim>:3891:12: unsupported string interpolation: only simple $name and {$name} interpolation in double-quoted strings is implemented; array offsets, object/static properties, and complex interpolation are not implemented`.
- [x] Parser/string lane: implement or explicitly bound the next WordPress
  6.9.4 bootstrap-shim complex interpolation blocker at `<bootstrap-shim>:3891:12`,
  with tests, CLI coverage, docs, and named unsupported edges for array-offset
  interpolation, object/static property interpolation, `${...}` forms,
  variable variables, expression interpolation, escaping/source spans, PHP
  diagnostic fidelity, and native lowering. The Milestone 735 slice supports
  direct array-offset interpolation for string, integer, bare-string, and
  variable keys plus direct object-property interpolation, keeps nested offsets,
  dynamic/static properties, `${...}`, arbitrary expressions, heredoc/nowdoc,
  and native lowering unsupported, and advances the real bootstrap-shim probe to
  `lex error at <bootstrap-shim>:4225:9: unsupported heredoc/nowdoc string syntax: multiline string literals are not implemented`.
- [x] Parser/string lane: implement or explicitly bound the next WordPress
  6.9.4 bootstrap-shim heredoc/nowdoc blocker at `<bootstrap-shim>:4225:9`,
  with tests, CLI coverage, docs, and named unsupported edges for label parsing,
  indentation stripping, interpolation, nowdoc non-interpolation, source spans,
  exact diagnostics, and native lowering. The Milestone 736 slice accepts
  unindented identifier-label heredoc/nowdoc, trims the line ending before the
  terminator, evaluates heredoc with the current interpolation subset, keeps
  nowdoc literal, and advances the real bootstrap-shim probe to
  `lex error at <bootstrap-shim>:7267:17: unsupported string interpolation: only simple $name, {$name}, direct array offsets, and direct object properties in double-quoted strings are implemented; ${...}, nested offsets, dynamic properties, static properties, and complex interpolation are not implemented`.
- [x] Parser/string lane: implement or explicitly bound the next WordPress
  6.9.4 bootstrap-shim nested interpolation blocker at `<bootstrap-shim>:7267:17`,
  likely covering chained object-property/array-offset interpolation such as
  `{$block->context['displayLayout']['columns']}`, with tests, CLI coverage,
  docs, and named unsupported edges for chain parsing, null/non-array/non-object
  diagnostics, dynamic properties, static properties, `${...}`, exact
  diagnostics, and native lowering. The Milestone 737 slice supports chained
  property/array-offset interpolation over current arrays and objects, keeps
  dynamic properties, static properties, `${...}`, arbitrary expression
  interpolation, exact diagnostics, and native lowering unsupported, and
  advances the real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:856:4: expected expression, found static`.
- [x] Parser/function lane: implement or explicitly bound the next WordPress
  6.9.4 bootstrap-shim `static function (...)` closure blocker at
  `<bootstrap-shim>:856:4`, with tests, CLI coverage, docs, and named
  unsupported edges for static closure binding, capture semantics, invocation,
  type declarations, callbacks, and native lowering. Milestone 738 parses
  static anonymous closures as inert closure values, keeps binding/capture/
  invocation/callback/native behavior unsupported, and removes the previous
  `expected expression, found static` parser blocker.
- [x] WordPress runtime lane: add a timeout-bounded or instrumented
  bootstrap-shim probe after Milestone 738, because the normalized shim no
  longer reports a quick parse error and instead ran for more than a minute
  before manual termination. Use the narrowed probe to identify the next
  concrete runtime blocker or loop without claiming WordPress bootstrap
  success. Milestone 739 adds `WORDPRESS_PROBE_TIMEOUT` support and reports
  the current local WordPress 6.9.4 bootstrap shim as `timed_out: yes` at
  `10s`, exit `124`, zero stdout, and no stderr.
- [x] Runtime diagnostics lane: add a bounded execution budget or trace mode
  for `phpc run` that reports the last source span/function/include frame
  before timeout/step exhaustion, then use it on the post-Milestone 738
  WordPress bootstrap shim to identify the next concrete loop or runtime
  blocker. Milestone 740 adds `PHPC_MAX_EXECUTION_STEPS` and proves it catches
  runtime loops, but the real bootstrap shim still times out at `30s` even
  with `PHPC_MAX_EXECUTION_STEPS=100`, so the long path is before normal
  statement execution budget exhaustion.
- [x] Parser/include diagnostics lane: add tracing or a bounded budget around
  include parsing and declaration registration, then rerun the WordPress
  bootstrap shim to identify the current long path before implementing another
  PHP feature. Milestone 741 adds `PHPC_TRACE_INCLUDES=1` and inventory
  `last_stderr_line`; the current timeout frontier is
  `<wordpress-root>/wp-includes/sodium_compat/src/Compat.php`.
- [x] Parser/performance diagnostics lane: profile or budget parsing and
  declaration registration for
  `<wordpress-root>/wp-includes/sodium_compat/src/Compat.php`. Milestone 742
  adds `PHPC_TRACE_PARSE=1` parser frontier logging and fixes lexer byte-offset
  tracking so the 4530-line Sodium compatibility class completes directly under
  a 10s outer timeout instead of timing out before parser trace output.
- [x] Runtime/parser lane: implement bounded final-position variadic
  parameters for user functions so `...$items` collects extra positional
  arguments into a current ordered array, with system-PHP comparison coverage.
  Variadic argument unpacking, by-reference variadics, type enforcement, exact
  diagnostics, and native lowering remain unsupported.
- [x] Parser/runtime lane: implement or explicitly bound comma-separated
  `for` header expression lists. The real WordPress 6.9.4 bootstrap shim now
  advances past the previous `parse error at <bootstrap-shim>:2099:16`
  blocker.
- [x] Parser/runtime lane: diagnose and implement or explicitly bound the next
  WordPress bootstrap-shim parse blocker. The real WordPress 6.9.4 bootstrap
  shim reached `parse error at <bootstrap-shim>:3909:1: expected expression,
  found <`; Milestone 744 adds bounded inline HTML output between `?>` and the
  next PHP open tag.
- [x] Object/runtime lane: implement or explicitly bound dynamic property
  access. Milestone 745 adds bounded dynamic property-name reads/writes for
  existing public slots and `stdClass` public dynamic slots. The real WordPress
  6.9.4 bootstrap shim now advances past the previous
  `<bootstrap-shim>:4451:14` dynamic-property blocker and reaches
  `parse error at <bootstrap-shim>:4955:17: unsupported reference expression:
  references are not implemented`.
- [x] References/value-model lane: implement or explicitly bound reference
  expressions for the next real WordPress bootstrap-shim blocker at
  `<bootstrap-shim>:4955:17`. Milestone 746 accepts statement-form
  by-reference assignment as a runtime boundary for direct variable sources.
- [x] Foreach/value-model lane: implement or explicitly bound by-reference
  `foreach` iteration for the next real WordPress bootstrap-shim blocker at
  `<bootstrap-shim>:5047:28`. Milestone 747 accepts by-reference foreach value
  syntax as a runtime boundary and advances the real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:5188:31: expected ';' after reference assignment`.
- [x] References/value-model lane: implement or explicitly bound
  by-reference assignment from direct array-offset sources for the real
  WordPress bootstrap-shim blocker at `<bootstrap-shim>:5188:31`. Milestone
  748 accepts `$alias =& $array[$key];` as a runtime boundary and advances the
  real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:5463:28: unsupported assignment expression target: only direct static variables, direct array offsets, direct append offsets, and direct object properties are implemented; nested targets are not implemented`.
- [x] Arrays/value-model lane: implement or explicitly bound append-at-depth
  assignment expressions for the next real WordPress bootstrap-shim blocker at
  `<bootstrap-shim>:5463:28`, corresponding to
  `$submenu['themes.php'][] = ...` in `wp-includes/functions.php`. Milestone
  749 implements append-at-depth assignment for direct-variable nested array
  paths and advances the real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:3149:47: unsupported unset: only direct variables like unset($name), direct array offset removal like unset($array[$key]), and direct static property operands like unset(ClassName::$property) are implemented; object property, append, and nested unset forms are not implemented`.
- [x] Arrays/value-model lane: implement or explicitly bound nested
  direct-variable array-offset `unset(...)` for the next real WordPress
  bootstrap-shim blocker at `<bootstrap-shim>:3149:47`, corresponding to
  `unset( $new_allowed_options[ $option_group ][ $pos ] );` in
  `wp-includes/option.php`. Milestone 750 implements nested direct-variable
  array-offset `unset(...)` and advances the real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:21:21: unsupported property default: instance property default values are not implemented`.
- [x] Object/runtime lane: implement or explicitly bound instance property
  default values for the next real WordPress bootstrap-shim blocker at
  `<bootstrap-shim>:21:21`, corresponding to `public $_nplurals = 2;` in
  `wp-includes/pomo/mo.php`. Milestone 751 implements untyped instance
  property defaults for the current constant-expression subset and advances the
  real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:301:46: unsupported reference assignment: only direct variable and direct array-offset reference sources are parsed before reference semantics exist`.
- [x] Parser/reference lane: implement or explicitly bound by-reference
  assignment from method-call sources for the next real WordPress
  bootstrap-shim blocker at `<bootstrap-shim>:301:46`, corresponding to
  `$entry = &$this->make_entry( $original, $translation );` in
  `wp-includes/pomo/mo.php`. Milestone 752 parses direct method-call reference
  sources as the existing runtime boundary and advances the real
  bootstrap-shim probe to
  `parse error at <bootstrap-shim>:302:38: unsupported assignment expression target: only direct static variables, direct array offsets, direct append offsets, nested array offsets, append-at-depth targets, and direct object properties are implemented`.
- [x] Parser/reference lane: implement or explicitly bound by-reference
  assignment into object-property array-offset targets for the next real
  WordPress bootstrap-shim blocker at `<bootstrap-shim>:302:38`,
  corresponding to `$this->entries[ $entry->key() ] = &$entry;` in
  `wp-includes/pomo/mo.php`. Milestone 753 parses this direct target shape as
  the existing runtime boundary and advances the real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:319:19: unsupported reference return: returning functions by reference is not implemented`.
- [x] Parser/reference lane: implement or explicitly bound by-reference
  method returns for the next real WordPress bootstrap-shim blocker at
  `<bootstrap-shim>:319:19`, corresponding to
  `public function &make_entry( $original, $translation )` in
  `wp-includes/pomo/mo.php`. Milestone 754 parses by-reference function and
  method return declarations as runtime boundaries and advances the real
  bootstrap-shim probe to
  `parse error at <bootstrap-shim>:15:1: unsupported class modifier: abstract, final, and readonly class modifiers are not implemented`.
- [x] Parser/object lane: implement or explicitly bound class modifiers for
  the next real WordPress bootstrap-shim blocker at `<bootstrap-shim>:15:1`.
  Milestone 755 parses `abstract` and `final` class modifiers plus
  `abstract`/`final` method modifiers as metadata, rejects abstract class
  instantiation as a runtime boundary, and advances the real bootstrap-shim
  probe to
  `parse error at <bootstrap-shim>:63:26: unsupported magic class name: self, parent, and static class name resolution is not implemented`.
- [x] Parser/object lane: implement or explicitly bound broader
  `self::class`, `parent::class`, and `static::class` class-name resolution for
  the next real WordPress bootstrap-shim blocker at `<bootstrap-shim>:63:26`.
  Milestone 756 covers the reached magic class-name instantiation form by
  resolving `new self`, `new parent`, and `new static` in active class/method
  contexts and advances the real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:131:70: unsupported assignment expression target: only direct static variables, direct array offsets, direct append offsets, nested array offsets, append-at-depth targets, and direct object properties are implemented`.
- [x] Parser/value-model lane: identify and implement or explicitly bound the
  next assignment-expression target shape at `<bootstrap-shim>:131:70`.
  Milestone 757 implements direct-object-property nested array assignment and
  append-at-depth assignment for targets such as
  `$this->loaded_files[$translation_file][$locale][$textdomain] = $moe` and
  `$this->loaded_translations[$locale][$textdomain][] = $moe`, advancing the
  real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:165:19: unsupported unset: object property unset is not implemented; property uninitialization, magic methods, and typed property semantics are not modeled`.
- [x] Parser/value-model lane: implement or explicitly bound nested
  object-property array-offset `unset(...)` for the next real WordPress
  bootstrap-shim blocker at `<bootstrap-shim>:165:19`, corresponding to
  `unset( $this->loaded_translations[ $locale ][ $textdomain ][ $i ] );` in
  `wp-includes/l10n/class-wp-translation-controller.php`. Milestone 758
  implements that direct-object-property nested array unset subset and
  advances the real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:24:13: unsupported include expression: expression-form include and include return values are not implemented; use statement-form include path; for existing local files`.
- [x] Runtime/include lane: implement or explicitly bound expression-form
  `include` with return values for the next real WordPress bootstrap-shim
  blocker at `<bootstrap-shim>:24:13`, corresponding to
  `$result = include $this->file;` in
  `wp-includes/l10n/class-wp-translation-file-php.php`.
  Milestone 759 implements expression-form `include`, `include_once`,
  `require`, and `require_once` for the current local-file subset, including
  include return values and `_once` loaded-file return values. It advances the
  real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:1557:9: unsupported array destructuring: only simple positional statement-form list($a, $b) = expr targets are implemented; short [...], expression-position list(...), nested, keyed, skipped, reference, and non-variable targets are not implemented`.
- [x] Parser/runtime lane: implement or explicitly bound the next array
  destructuring shape for the real WordPress bootstrap-shim blocker at
  `<bootstrap-shim>:1557:9`, while preserving the existing positional
  `list($a, $b) = expr;` statement subset and documenting unsupported keyed,
  nested, reference, expression-position, and non-variable target semantics.
  Milestone 760 implements skipped positional slots in statement-form
  `list(...) = expr;`, covering `list( , $textdomain, $language ) = $match;`
  in `wp-includes/l10n.php`, and advances the real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:19:21: unsupported interface implementation: implements clauses are not implemented`.
- [x] Parser/object lane: implement or explicitly bound interface
  implementation metadata for the next real WordPress bootstrap-shim blocker at
  `<bootstrap-shim>:19:21`, while keeping interface method enforcement,
  inheritance, constants, variance checks, autoload, and native lowering named
  as unsupported unless implemented.
  Milestone 761 implements class `implements` metadata for comma-separated
  interface names and relationship checks through `is_a`,
  `is_subclass_of`, and `instanceof`, including inherited metadata and
  unresolved built-in/internal interface names as metadata-only relationships.
  It advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1428:2: unsupported call reference assignment: references and aliasing are not implemented`,
  corresponding to `$l10n[ $domain ] = &$noop_translations;` in
  `wp-includes/l10n.php:1428`.
- [x] Runtime/value-model lane: implement real PHP reference assignment for
  the next real WordPress bootstrap-shim blocker at
  `wp-includes/l10n.php:1428`, or explicitly widen the existing runtime
  boundary only if the reached code can continue honestly without references.
  This must address alias containers, symbol-table/array-slot binding,
  copy-on-write interaction, mutation ordering, and exact unsupported
  edge-case docs before claiming support.
  Milestone 762 implements only the honest object-handle subset for direct
  variable sources into direct variable or direct array-offset targets. It
  advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:231:20: undefined function str_replace()`.
- [x] Builtin/runtime lane: implement a bounded `str_replace()` slice for the
  next real WordPress bootstrap-shim blocker while documenting unsupported
  array search/replace forms, count output argument references, object/resource
  coercions, binary/string edge cases, and native lowering.
  Milestone 763 implements scalar/null string-convertible `str_replace()` for
  the three-argument form and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:3839:2: undefined function call_user_func()`,
  corresponding to `call_user_func( $the_['function'] )` in
  `wp-includes/class-wp-hook.php:339`.
- [x] Callable/runtime lane: implement a bounded `call_user_func()` slice for
  the next real WordPress bootstrap-shim blocker, including direct string
  callables and current array callable metadata where required, while keeping
  references, variadic unpacking, `call_user_func_array`, closure invocation,
  `__invoke`, exact warnings, and native lowering named unless implemented.
  Milestone 764 implements string callables only and advances the real
  bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:4955:3: unsupported call reference assignment: references and aliasing are not implemented`,
  corresponding to `$parsed_args =& $args;` in
  `wp-includes/functions.php:4955`.
- [x] Runtime/value-model lane: implement or honestly bound direct variable
  array by-reference assignment for the next real WordPress bootstrap-shim
  blocker at `wp-includes/functions.php:4955`, while keeping alias rebinding,
  scalar references, nested/array-offset sources, copy-on-write, mutation
  ordering, exact PHP diagnostics, and native lowering named unless
  implemented.
  Milestone 765 implements the direct-variable source to direct-variable target
  subset when the source currently holds an array or object value. It advances
  the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:108:9: undefined function strcasecmp()`,
  corresponding to `wp-includes/compat.php:108`.
- [x] Builtin/runtime lane: implement a bounded `strcasecmp()` slice for the
  next real WordPress bootstrap-shim blocker, covering the reached
  case-insensitive string comparison path while documenting unsupported broad
  scalar coercions, array/object/resource operands, binary/locale edge cases,
  exact PHP diagnostics, and native lowering unless implemented.
  Milestone 766 implements exact-two-argument scalar/null string-convertible
  `strcasecmp()` with ASCII case folding and advances the real bootstrap-shim
  probe to
  `runtime error at <bootstrap-shim>:3890:10: undefined function headers_sent()`,
  corresponding to `wp-includes/functions.php:3890`.
- [x] Web/SAPI runtime lane: implement a bounded `headers_sent()` slice for the
  next real WordPress bootstrap-shim blocker, likely returning `false` for the
  current no-header-state CLI shim unless output/header state is implemented.
  Keep by-reference filename/line output arguments, output-buffer interaction,
  header storage, SAPI differences, exact warnings, and native lowering named
  unless implemented.
  Milestone 767 implements no-argument `headers_sent()` as `false` for the
  current no-header-state shim and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1469:9: undefined function abs()`,
  corresponding to `wp-includes/load.php:1469`.
- [x] Numeric builtin lane: implement a bounded `abs()` slice for the next real
  WordPress bootstrap-shim blocker, covering the reached integer-cast
  `absint()` path while documenting unsupported float overflow/NaN/infinity
  behavior, non-scalar operands, exact diagnostics, and native lowering unless
  implemented.
  Milestone 768 implements integer and finite-float `abs()` and advances the
  real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1547:2: undefined function header_remove()`,
  corresponding to `wp-includes/functions.php:1547`.
- [x] Web/SAPI runtime lane: implement a bounded `header_remove()` slice for
  the next real WordPress bootstrap-shim blocker, likely as a no-op for current
  string header names while documenting all-header removal, header storage,
  output-sent warnings, SAPI behavior, exact diagnostics, and native lowering
  unless implemented.
  Milestone 769 implements no-argument and string-name `header_remove()` as a
  no-op returning `null`. It advances the real bootstrap-shim probe into
  WordPress' `wp_check_php_mysql_versions()` missing-MySQL-extension path in
  `wp-includes/load.php:202`, producing the missing `mysqli` extension HTML
  page and exit code `1` instead of a compiler/runtime unsupported diagnostic.
- [x] Database/extension lane: define the honest next WordPress database
  compatibility step after the missing-MySQL-extension guard. Options include a
  bounded `function_exists('mysqli_connect')`/extension presence policy only if
  it immediately leads to explicit unsupported mysqli/database operations, or a
  real minimal mysqli/PDO compatibility plan with host assumptions. Do not
  claim MySQL support without executable database behavior, tests, docs, and
  named unsupported edges.
  Milestone 770 exposes `mysqli_connect` through function/callability metadata
  and dynamic lookup but makes all direct and dynamic connection calls an
  explicit unsupported database boundary. It advances the real
  bootstrap-shim probe past the missing-MySQL-extension guard to
  `runtime error at <bootstrap-shim>:39:6: undefined variable '$wp_filter'`,
  corresponding to `wp-includes/plugin.php:39`.
- [x] Runtime compatibility lane: implement or explicitly bound PHP-shaped
  undefined-variable reads for the reached WordPress `if ( $wp_filter )` path,
  likely warning-plus-`null`/falsey behavior in `phpc run`, while preserving
  stable diagnostics and documenting unsupported warning reporting,
  partial-output behavior, native lowering, and cases where undefined reads
  must remain hard boundaries.
  Milestone 771 implements the narrow top-level `global` materialization slice:
  missing names declared by top-level `global` become `null`, while ordinary
  undefined variable reads still fail with the stable runtime diagnostic. It
  advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:39:33: undefined function microtime()`.
- [x] Time builtin lane: implement a bounded `microtime()` slice for the next
  WordPress bootstrap blocker, likely covering `microtime(true)` as a finite
  float timestamp while documenting nondeterminism, precision, string-return
  format, time source policy, monotonicity, tests, and native lowering unless
  implemented.
  Milestone 772 implements `microtime(true)` as a host-clock finite float
  seconds value and keeps the string-return forms unsupported. It advances the
  real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:42:23: undefined function ini_get()`.
- [x] INI/config lane: implement a bounded `ini_get()` slice for the next
  WordPress bootstrap blocker, starting with the reached option name and a
  deterministic runtime configuration policy. Keep host php.ini discovery,
  mutable ini state, value typing/stringification, SAPI differences, exact
  false-vs-string behavior, and native lowering named unless implemented.
  Milestone 773 implements a deterministic string registry for `ini_get()` and
  advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1688:11: undefined function strtolower()`,
  corresponding to `wp-includes/load.php:1688`.
- [x] String builtin lane: implement a bounded `strtolower()` slice for the
  next WordPress bootstrap blocker, starting with ASCII/UTF-8 runtime strings
  used by `wp_convert_hr_to_bytes()`. Keep locale, binary string edge cases,
  array/object/resource coercions, exact diagnostics, and native lowering named
  unless implemented.
  Milestone 774 implements the current scalar/null string-convertible
  `strtolower()` slice with ASCII lowercase mapping and advances the real
  bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1688:23: undefined function trim()`,
  corresponding to `wp-includes/load.php:1688`.
- [x] String builtin lane: implement a bounded `trim()` slice for the next
  WordPress bootstrap blocker, starting with the default character mask needed
  by `wp_convert_hr_to_bytes()`. Keep custom character masks, binary string
  edge cases, array/object/resource coercions, exact diagnostics, and native
  lowering named unless implemented.
  Milestone 775 implements the default-mask scalar/null string-convertible
  `trim()` slice and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1689:11: unsupported call (int): leading-numeric string cast behavior is not implemented`,
  corresponding to `wp-includes/load.php:1689`.
- [x] Cast/coercion lane: implement a bounded leading-numeric `(int)` string
  cast slice for the next WordPress bootstrap blocker, starting with the
  shorthand memory strings produced by `wp_convert_hr_to_bytes()`. Keep PHP's
  warning/recovery details, whitespace/sign/decimal/exponent grammar,
  overflow, binary string edge cases, exact diagnostics, and native lowering
  named unless implemented.
  Milestone 776 implements a bounded leading-numeric string prefix scanner for
  `(int)` casts and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1691:7: undefined function str_contains()`,
  corresponding to `wp-includes/load.php:1691`.
- [x] String builtin lane: implement a bounded `str_contains()` slice for the
  next WordPress bootstrap blocker, starting with scalar/null string-convertible
  haystack and needle arguments. Keep binary string edge cases, array/object/
  resource coercions, exact diagnostics, empty-needle behavior, and native
  lowering named unless implemented.
  Milestone 777 implements the current scalar/null string-convertible
  `str_contains()` slice with empty-needle `true` behavior and advances the
  real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1700:9: undefined function min()`,
  corresponding to `wp-includes/load.php:1700`.
- [x] Math/value builtin lane: implement a bounded `min()` slice for the next
  WordPress bootstrap blocker, starting with the reached two integer arguments
  in `wp_convert_hr_to_bytes()`. Keep array argument forms, mixed-type
  comparison rules, object/resource operands, exact diagnostics, and native
  lowering named unless implemented.
  Milestone 778 implements integer `min()` plus `PHP_INT_MAX` and advances the
  real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1724:14: unsupported call isset(): only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported`,
  corresponding to `isset( $ini_all[ $setting ]['access'] )` in
  `wp-includes/load.php:1724`.
- [x] Runtime/null-aware lane: implement a bounded nested array-offset
  `isset(...)` operand slice for the reached WordPress
  `isset( $ini_all[ $setting ]['access'] )` path, preserving false for missing
  or null intermediates without warnings. Keep arbitrary expressions, object
  dimensions, references/copy-on-write, exact warning suppression, and native
  lowering named unless implemented.
  Milestone 779 implements direct-variable rooted nested array-offset
  `isset(...)` and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:87:38: undefined function is_readable()`,
  corresponding to `wp-includes/error-protection.php:87`.
- [x] Filesystem/runtime lane: implement a bounded `is_readable()` builtin
  slice for the reached WordPress fatal-error-handler override check, starting
  with one string path argument and host filesystem metadata. Keep stream
  wrappers, include_path behavior, permissions portability, warning behavior,
  non-string coercions, cache invalidation, exact diagnostics, and native
  lowering named unless implemented.
  Milestone 780 implements the local one-string path slice and advances the
  real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:95:2: undefined function register_shutdown_function()`,
  corresponding to `wp-includes/error-protection.php:95`.
- [x] Runtime/SAPI lane: implement a bounded `register_shutdown_function()`
  slice for the reached WordPress fatal-error-handler registration path,
  starting with accepting/registering callable metadata without executing
  shutdown callbacks unless a tested shutdown phase exists. Keep callback
  invocation ordering, argument passing, by-reference callbacks, output
  buffering, destructor/finally interaction, fatal-error context,
  exact diagnostics, and native lowering named unless implemented.
  Milestone 781 implements current callable validation and PHP-compatible
  `null` return without executing callbacks, and advances the real
  bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:73:1: undefined function date_default_timezone_set()`.
- [x] Date/time runtime lane: implement a bounded
  `date_default_timezone_set()` slice for the reached WordPress bootstrap
  timezone initialization path. Keep timezone identifier validation breadth,
  global timezone state interactions, `date_default_timezone_get()`, ini/date
  extension behavior, warning behavior, exact diagnostics, and native lowering
  named unless implemented.
  Milestone 782 implements the reached `UTC` setter slice and advances the
  real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:42:50: undefined variable '$_SERVER'`,
  corresponding to the `wp_fix_server_vars()` startup path.
- [x] Runtime/request-state lane: materialize a bounded `$_SERVER` superglobal
  array for the reached WordPress `wp_fix_server_vars()` path. Keep broad SAPI
  request state, environment import policy, mutation/reference behavior,
  case/key completeness, exact warning behavior, native lowering, and other
  superglobals named unless implemented.
  Milestone 783 implements deterministic CLI `$_SERVER` defaults plus
  `PHP_SAPI = "cli"` and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:46:35: undefined function preg_match()`.
- [x] Regex/runtime lane: implement a bounded `preg_match()` slice for the
  reached WordPress `wp_fix_server_vars()` SAPI-name path. Keep full PCRE
  behavior, captures/matches output, flags, offsets, invalid-pattern warnings,
  byte/Unicode edge cases, subject coercions, exact diagnostics, and native
  lowering named unless implemented.
  Milestone 784 implements a bounded slash-delimited literal pattern slice and
  advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:635:3: undefined function error_reporting()`.
- [x] Error/reporting runtime lane: implement a bounded `error_reporting()`
  slice for the reached WordPress startup path. Keep mutable reporting masks,
  constants such as `E_ALL`, interaction with warnings/notices/deprecations,
  ini state, previous-mask return behavior, exact diagnostics, and native
  lowering named unless implemented.
  Milestone 785 implements bounded integer mask state plus reached `E_*`
  constants and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:666:10: undefined function is_dir()`.
- [x] Filesystem runtime lane: implement a bounded `is_dir()` slice for the
  reached WordPress startup path. Keep stream wrappers, include-path behavior,
  symlink/canonicalization policy, permission/open_basedir behavior,
  non-string coercions, stat-cache behavior, exact diagnostics, and native
  lowering named unless implemented.
  Milestone 786 implements the local one-string path slice and advances the
  real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:124:22: expected property name after '->', found {`.
- [x] Object/parser runtime lane: implement or explicitly bound braced dynamic
  object-property access such as `$object->{$name}` for the reached WordPress
  startup path. Keep arbitrary expressions, writes, unset/isset/empty/null
  coalescing behavior, magic properties, non-public visibility context,
  references/copy-on-write, exact diagnostics, and native lowering named
  unless implemented.
  Milestone 787 implements braced dynamic-property reads and direct-variable
  root writes through the existing dynamic-property runtime subset, and
  advances the real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:173:7: unsupported magic constant __METHOD__: method context evaluation requires method dispatch, which is not implemented`.
- [x] Object/magic-constant lane: implement bounded `__METHOD__` evaluation
  for the reached WordPress method context. Keep trait methods, closure
  contexts, static method edge cases, case/original-name fidelity,
  namespace-qualified names, source mapping, native lowering, and exact PHP
  behavior named unless implemented.
  Milestone 788 implements current function/method-context `__METHOD__`
  evaluation and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:53:2: undefined function set_error_handler()`.
- [x] Error runtime lane: implement or explicitly bound `set_error_handler()`
  registration for the reached WordPress fatal/error handling path. Keep
  callback invocation, warning/notice/deprecation routing, previous-handler
  return semantics, error-level filtering, shutdown/fatal interactions,
  restoration, exact PHP diagnostics, and native lowering named unless
  implemented.
  Milestone 789 implements bounded registration and advances the real
  bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:54:3: unsupported call closure: closure capture binding is not implemented`.
- [x] Closure runtime lane: implement or explicitly bound by-value and
  by-reference closure capture binding for the reached WordPress
  `set_error_handler(function (...) use (&$utf8_pcre) { ... })` path. Keep
  closure invocation, alias/reference semantics, copy-on-write, `$this`
  binding, static closures, exact capture timing, native lowering, and exact
  PHP diagnostics named unless implemented.
  Milestone 790 implements explicit capture binding for inert closure values
  and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:70:2: unsupported call preg_match(): pattern modifiers are not implemented in the current subset`.
- [x] Regex runtime lane: widen bounded `preg_match()` for the reached
  `//u` startup probe. Keep full PCRE syntax, captures/matches output, flags,
  offsets, warning/error-handler routing, Unicode semantics, broad modifiers,
  exact diagnostics, and native lowering named unless implemented.
  Milestone 791 implements bounded `u`-modifier handling for the existing
  literal subset and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:71:2: undefined function restore_error_handler()`.
- [x] Error-handler runtime lane: implement bounded `restore_error_handler()`
  for the reached WordPress `_wp_can_use_pcre_u()` cleanup path. Keep true
  handler-stack behavior, handler invocation, warning/notice/deprecation
  routing, error-level filtering, shutdown/fatal interactions, exact PHP
  diagnostics, and native lowering named unless implemented.
  Milestone 792 implements bounded cleanup and advances the real
  bootstrap-shim probe to
  `parse error at <bootstrap-shim>:254:31: expected property name after '->', found public`.
- [x] Parser/object lane: accept keyword tokens as object property names after
  `->` for the reached `$object->public` WordPress startup path. Keep keyword
  method names, dynamic/static properties, visibility semantics, magic
  property hooks, references/copy-on-write, exact diagnostics, and native
  lowering named unless implemented.
  Milestone 793 implements keyword-named direct object-property parsing and
  advances the real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:418:48: unsupported array reference element: references are not implemented`.
- [x] Parser/reference lane: implement or explicitly bound array element
  reference syntax for the reached WordPress startup path. Keep aliasing,
  reference containers, copy-on-write, by-reference iteration/returns,
  destructuring references, exact diagnostics, and native lowering named unless
  implemented.
  Milestone 794 parses array literal reference values and evaluates their
  current values without aliasing, advancing the real bootstrap-shim probe to
  `parse error at <bootstrap-shim>:671:32: unsupported reference assignment: only direct variable, direct array-offset, and method-call reference sources are parsed before reference semantics exist`.
- [x] Parser/reference lane: widen reference-assignment parsing for the reached
  WordPress startup path. Keep true aliases, reference containers,
  copy-on-write, object/static/dynamic lvalues, by-reference return sources,
  exact diagnostics, and native lowering named unless implemented.
  Milestone 795 parses object-property reference-assignment sources and copies
  current array/object values without aliasing, advancing the real
  bootstrap-shim probe to
  `parse error at <bootstrap-shim>:832:15: unsupported unset: object property unset is not implemented; property uninitialization, magic methods, and typed property semantics are not modeled`.
- [x] Parser/object lane: implement bounded `unset($object->property)` for the
  reached WordPress startup path. Keep typed-property uninitialization, magic
  `__unset`, visibility context beyond the current visible-property slice,
  array/object mixed unset targets, references/copy-on-write, exact
  diagnostics, and native lowering named unless implemented.
  Milestone 796 accepts direct and dynamic object-property unset operands such
  as `unset($object->property)` and `unset($object->$name)` for direct object
  variables, nulls the current visible property slot, and advances the real
  bootstrap-shim probe to
  `parse error at <bootstrap-shim>:4127:38: unsupported magic constant __CLASS__: class context evaluation requires class-context tracking, which is not implemented`.
- [x] Runtime/magic-constant lane: implement bounded `__CLASS__` evaluation in
  class context for the reached WordPress startup path. Keep trait/namespace
  edge cases, closure rebinding, anonymous-class exact names, source mapping,
  native lowering, and other magic constants named unless implemented.
  Milestone 797 evaluates executable `__CLASS__` to the active class name in
  method context and to an empty string outside class context, advancing the
  real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1970:3: undefined function mysqli_report()`.
- [x] Runtime/database lane: add a bounded `mysqli_report()` compatibility
  boundary for the reached WordPress startup path. Keep real mysqli extension
  state, report mode validation beyond the reached constants, connection/query
  behavior, warning/error routing, exact diagnostics, and native lowering named
  unless implemented.
  Milestone 798 exposes `mysqli_report`, defines the reached report constants,
  accepts `MYSQLI_REPORT_OFF` and `MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT`,
  stores the current mode, and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1972:16: undefined function mysqli_init()`.
- [x] Runtime/database lane: add an honest `mysqli_init()` boundary for the
  reached WordPress startup path. Decide whether to return a minimal mysqli
  object/resource handle that only survives until the next connection call, or
  to report a stable unsupported database initialization diagnostic. Keep real
  connections, host IO, query/result behavior, escaping, charset state,
  warning/error routing, exact diagnostics, and native lowering named unless
  implemented.
  Milestone 799 returns a placeholder `mysqli` object with `connect_errno = 0`
  and `connect_error = null`, exposes `mysqli_init` through dynamic lookup and
  native function metadata, and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:2082:17: undefined function strpos()`.
- [x] Runtime/string lane: add a bounded `strpos()` implementation for the
  reached WordPress `parse_db_host()` path. Preserve exact PHP behavior for
  empty needles, offsets, false-vs-zero return checks, binary string matching,
  invalid argument diagnostics, array/object arguments, and native lowering
  unless those slices are implemented and tested.
  Milestone 800 implements scalar/null string-convertible haystack and needle
  arguments, optional integer offsets, empty-needle/effective-offset behavior,
  negative offsets, byte-position matching, and `false` for no match,
  advancing the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:2092:8: undefined function substr_count()`.
- [x] Runtime/string lane: add a bounded `substr_count()` implementation for
  the reached WordPress `parse_db_host()` path. Preserve PHP-exact offset and
  length behavior, empty-needle diagnostics, overlap behavior, scalar
  coercions, array/object arguments, and native lowering unless those slices
  are implemented and tested.
  Milestone 801 implements scalar/null string-convertible haystack and needle
  arguments, optional integer offset and length slicing, negative offsets and
  lengths within the current bounds rules, non-overlapping byte-position
  counts, and short-slice zero counts, advancing the real bootstrap-shim probe
  to `runtime error at <bootstrap-shim>:2101:14: unsupported call preg_match(): matches output, flags, and offset arguments are not implemented; pass exactly two arguments in the current subset`.
- [x] Runtime/regex lane: add bounded `preg_match()` matches-output support for
  the reached WordPress `parse_db_host()` path. Preserve capture naming,
  optional unmatched groups, flags, offsets, full PCRE behavior, invalid
  pattern warnings, exact diagnostics, and native lowering unless those slices
  are implemented and tested.
  Milestone 802 accepts a third direct `$matches` variable, writes match `0`
  for current literal patterns, clears matches on no match, recognizes the two
  exact WordPress db-host named-capture patterns reached in
  `class-wpdb.php`, and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1997:5: undefined function mysqli_real_connect()`.
- [x] Runtime/database lane: add an honest bounded `mysqli_real_connect()`
  boundary for the reached WordPress startup path. Decide whether the current
  placeholder `mysqli` object should record deterministic connection state for
  the bootstrap path or whether the call should stop with a stable unsupported
  database-connection diagnostic. Keep real host I/O, authentication,
  socket/port handling, database selection, warnings/errors, charset state,
  query/result behavior, escaping, exact diagnostics, PDO, and native database
  lowering named unless implemented.
  Milestone 803 accepts the current WordPress call shape for the placeholder
  `mysqli` object, writes `connect_errno = 0` and `connect_error = null`,
  returns deterministic fake success without host I/O, and advances the real
  bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:4138:10: undefined function preg_replace()`.
- [x] Runtime/regex lane: add bounded `preg_replace()` support for the reached
  WordPress `wp_debug_backtrace_summary()` startup path. Start from the exact
  call shape and pattern/replacement subject semantics reached by the probe;
  keep full PCRE replacement behavior, arrays of patterns/replacements,
  callbacks, limits/count output, invalid-pattern warnings, encoding edge
  cases, exact diagnostics, and native lowering named unless implemented.
  Milestone 804 implements exactly the WordPress database-version cleanup
  pattern `/[^0-9.].*/` with an empty replacement and scalar/null subject,
  returning the leading ASCII digits/dots prefix, and advances the real
  bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:4149:10: undefined function mysqli_get_server_info()`.
- [x] Runtime/database lane: add bounded `mysqli_get_server_info()` support for
  the reached WordPress `wpdb::db_server_info()` path. Prefer a deterministic
  placeholder server-version string tied to the current fake mysqli connection
  boundary; keep real server negotiation, host I/O, extension resources,
  connection-state validation, errors/warnings, exact diagnostics, and native
  database lowering named unless implemented.
  Milestone 805 returns deterministic placeholder server info
  `8.0.0-phpc-placeholder` for the placeholder `mysqli` object and advances
  the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:904:10: undefined function compact()`.
- [x] Runtime/array-symbol lane: add bounded `compact()` support for the
  reached WordPress startup path. Start with direct string variable-name
  arguments over the current symbol table, preserve PHP-shaped omission of
  missing names if implemented, and keep array arguments, nested arrays,
  invalid names, warning behavior, variable-variable interactions, exact
  diagnostics, and native lowering named unless implemented.
  Milestone 806 accepts one or more direct string variable-name arguments,
  reads the current caller scope, omits missing variables, returns an array
  keyed by found names, and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:951:11: undefined function mysqli_query()`.
- [x] Runtime/database lane: add a bounded `mysqli_query()` boundary for the
  reached WordPress `set_sql_mode()` path. Start with the exact
  `SELECT @@SESSION.sql_mode` query shape and decide whether to return an
  empty/false result boundary or a placeholder result object that can feed the
  immediately following `mysqli_fetch_array()` call. Keep real query
  execution, result resources, row iteration, SQL errors/warnings, connection
  state, escaping, exact diagnostics, and native database lowering named
  unless implemented.
  Milestone 807 accepts the placeholder `mysqli` object and exactly
  `SELECT @@SESSION.sql_mode`, returns `false` as a deterministic
  empty/no-result boundary, and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1203:14: undefined function mysqli_select_db()`.
- [x] Runtime/database lane: add bounded `mysqli_select_db()` support for the
  reached WordPress `select()` path. Start with the placeholder `mysqli` object
  and scalar/null database-name argument shape reached by the probe; keep real
  database selection, connection state, errors/warnings, exact diagnostics, and
  native database lowering named unless implemented.
  Milestone 808 accepts the placeholder `mysqli` object and string/null
  database names, returns deterministic `true`, and advances the real
  bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:143:28: undefined variable '$table_prefix'`.
- [x] WordPress/config lane: model the reached `$table_prefix` startup state in
  the bootstrap-shim inventory path. Decide whether the inventory shim should
  define the WordPress config variable before loading `wp-settings.php`, or
  whether the runtime should provide a documented compatibility default. Keep
  real `wp-config.php` loading, secret/key constants, database credentials,
  multisite table-prefix validation, exact diagnostics, and native lowering
  named unless implemented.
  Milestone 809 defines `$table_prefix = 'wp_';` in the inventory bootstrap
  shim before requiring `wp-settings.php`, proves the synthetic included-file
  path can see it, and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1006:8: unsupported call preg_match(): only slash-delimited patterns are implemented in the current subset`.
- [x] Runtime/regex lane: widen bounded `preg_match()` for the reached
  WordPress startup pattern delimiter shape while keeping broad PCRE syntax,
  callbacks, flags/offsets beyond the existing slice, exact warnings, and
  native lowering named unless implemented.
  Milestone 810 recognizes the exact WordPress table-prefix validation pattern
  `|[^a-z0-9_]|i`, returns no match for conventional prefixes such as `wp_`,
  and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1034:5: undefined property wpdb::$categories`.
- [x] Runtime/object lane: model the reached `wpdb::$categories` table-name
  initialization state without hiding broader object-property semantics. Start
  from the `wpdb::tables('old')` path reached after `set_prefix()`, and keep
  dynamic properties, declared/default property initialization, visibility,
  references/copy-on-write, exact diagnostics, and native lowering named unless
  implemented.
  Milestone 811 materializes dynamic public slots on the WordPress `wpdb`
  compatibility class for reached table-name assignments and advances the real
  bootstrap-shim probe to
  `parse error at <bootstrap-shim>:499:3: unsupported compound assignment target: only direct static variables, direct array offsets, direct object properties, and supported static properties are implemented; append offsets and nested targets are not implemented`.
- [x] Parser/runtime mutation lane: support the reached compound-assignment
  target shape, likely an append or nested target in the next WordPress
  bootstrap path. Keep broad nested compound assignment, mixed
  object/property/ArrayAccess targets, references/copy-on-write, exact
  diagnostics, and native lowering named unless implemented.
  Milestone 812 supports direct object-property array-offset compound
  assignment for the reached `$this->cache[ $group ][ $key ] += $offset;`
  object-cache path and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:359:2: unsupported call add_global_groups(): receiver must be object, got null`.
- [x] Runtime/object-cache lane: model the reached object-cache bootstrap state
  so `wp_cache_add_global_groups()` calls `add_global_groups()` on a real
  `WP_Object_Cache` placeholder instead of `null`. Keep real cache persistence,
  cache groups, eviction, global/non-persistent group semantics, object-cache
  drop-ins, exact diagnostics, and native lowering named unless implemented.
  Milestone 813 supports bounded direct `$GLOBALS['name']` root-symbol routing
  for the reached `wp_cache_init()` assignment and advances the real
  bootstrap-shim probe to the reached `WP_Hook::add_filter()` object-property
  array `isset(...)` path.
- [x] Runtime/hook lane: support the reached direct object-property
  array-offset `isset(...)` path in `WP_Hook::add_filter()` while keeping
  arbitrary object-dimension `isset(...)`, dynamic property paths, ArrayAccess,
  references/copy-on-write, exact diagnostics, and native lowering named unless
  implemented. Milestone 814 advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:98:4: undefined function ksort()`.
- [x] Runtime/array-ordering lane: implement the reached bounded
  `ksort($this->callbacks, SORT_NUMERIC)` behavior for `WP_Hook::add_filter()`
  without claiming full PHP sort semantics. Keep by-reference argument
  handling outside the reached target shapes, locale/string/natural sorts,
  mixed key comparison edge cases, stable-sort guarantees, exact diagnostics,
  and native lowering named unless implemented.
  Milestone 815 advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1780:7: unsupported call wp_cache_get(): reference parameter invocation is not implemented`.
- [x] Runtime/reference-parameter lane: handle the reached `wp_cache_get()`
  call shape where the function declaration contains an optional by-reference
  `$found` parameter but the current `get_option()` call omits it. Keep real
  reference parameter binding, output-parameter writes, alias cells,
  copy-on-write, exact diagnostics, and native lowering named unless
  implemented.
  Milestone 816 advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:154:9: unsupported call get(): reference parameter invocation is not implemented`.
- [x] Runtime/reference-output lane: implement the reached provided direct
  variable by-reference output-parameter shape for
  `WP_Object_Cache::get(..., $found)` without claiming full aliasing. Keep
  non-variable reference arguments, rebinding aliases, reference containers,
  copy-on-write, exact diagnostics, and native lowering named unless
  implemented.
  Milestone 817 advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:1283:15: undefined function mysqli_real_escape_string()`.
- [x] Runtime/mysqli lane: implement the reached bounded
  `mysqli_real_escape_string($this->dbh, $data)` behavior for
  `wpdb::_real_escape()` over the existing placeholder `mysqli` object and
  scalar string-convertible data. Keep connection charset state, real database
  connection behavior, warning/error routing, exact escaping edge cases,
  binary/invalid-string behavior, SQL execution, and native lowering named
  unless implemented.
  Milestone 818 advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:2422:71: undefined function rand()`,
  corresponding to `wp-includes/class-wpdb.php:2422` in
  `wpdb::placeholder_escape()`.
- [x] Runtime/random lane: implement the reached bounded `rand()` behavior for
  `wpdb::placeholder_escape()` without claiming cryptographic randomness or
  full PHP random-state compatibility. Keep seeding, mt/rand state coupling,
  min/max argument forms, swapped bounds, exact warnings/errors,
  deterministic-test policy, and native lowering named unless implemented.
  Milestone 819 advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:2424:25: undefined function hash_hmac()`,
  corresponding to `wp-includes/class-wpdb.php:2424` in
  `wpdb::placeholder_escape()`.
- [x] Runtime/hash lane: implement the reached bounded `hash_hmac('sha256',
  uniqid($salt, true), $salt)` behavior for `wpdb::placeholder_escape()`.
  Decide whether to use a narrow deterministic helper or a standard Rust crate.
  Keep broad algorithms, binary/raw output variants, array/object coercions,
  exact warnings/errors, cryptographic guarantees, host entropy, and native
  lowering named unless implemented.
  Milestone 820 uses the Rust `hmac` and `sha2` crates and advances the real
  bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:241:8: unsupported comparison: strict identity for arrays is not implemented`.
- [x] Runtime/comparison lane: implement strict identity for current arrays
  over the ordered array value model, enough for reached WordPress empty-array
  comparison shapes such as `array() === $value` and
  `array_values($arr) === $arr`. Keep references, recursive arrays,
  object/resource values, copy-on-write identity, exact diagnostics, and native
  lowering named unless implemented.
  Milestone 821 advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:3495:12: undefined function ltrim()`,
  likely in the localization path around `wp-includes/l10n.php:1051`.
- [x] Runtime/string lane: implement the reached bounded `ltrim($value, $mask)`
  behavior for WordPress path/query handling. Keep broad character-mask range
  semantics, binary/invalid UTF-8 behavior, array/object coercions, exact
  warnings/errors, locale-sensitive assumptions, and native lowering named
  unless implemented.
  Milestone 822 covers default-mask and non-empty literal-mask `ltrim()`
  calls, including slash masks and `"\r\n\t ("`, and advances the real
  bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:3496:8: unsupported call preg_match(): only the u pattern modifier is implemented in the current subset`,
  corresponding to `wp-includes/class-wpdb.php:3496`.
- [x] Runtime/regex lane: extend the bounded `preg_match()` subset for the
  reached case-insensitive `i` modifier in
  `/^(?:SHOW|DESCRIBE|DESC|EXPLAIN|CREATE)\s/i`. Keep broad PCRE syntax,
  modifier combinations, captures beyond the current subset, warnings/errors,
  Unicode/locale details, and native lowering named unless implemented.
  Milestone 823 covers the exact WordPress safe-collation read-query
  classifier and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:3474:16: unsupported call preg_match(): regex metacharacter [ is not implemented in the current subset`,
  corresponding to `wp-includes/class-wpdb.php:3474`.
- [x] Runtime/regex lane: implement the reached bounded ASCII-check
  `preg_match('/[^\x00-\x7F]/', $input_string)` path in
  `wpdb::check_ascii()`. Keep broad bracket classes, ranges beyond this exact
  byte range, binary/invalid UTF-8 behavior, exact PCRE diagnostics, and native
  lowering named unless implemented.
  Milestone 824 covers the exact WordPress non-ASCII detector and advances the
  real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:203:2: undefined function array_unshift()`.
- [x] Runtime/array lane: implement the reached bounded `array_unshift()`
  behavior for direct variable arrays in the WordPress bootstrap path. Keep
  broad by-reference argument handling, non-variable array targets, mixed
  key-preservation edge cases beyond PHP's documented integer reindexing,
  references/copy-on-write, exact warnings/errors, and native lowering named
  unless implemented.
  Milestone 825 covers direct-variable ordered-array mutation, string-key
  preservation, integer-key reindexing, return count, and string-valued direct
  dynamic calls, and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:328:48: undefined function current()`.
- [x] Runtime/array lane: implement the reached bounded `current()` behavior
  for current ordered arrays in the WordPress bootstrap path. Keep internal
  array pointer semantics beyond the reached shape, references/copy-on-write,
  non-array diagnostics, exact warnings, interaction with `next()`/`reset()`,
  and native lowering named unless implemented.
  Milestone 826 covers the first-value ordered-array slice and advances the
  real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:341:15: undefined function call_user_func_array()`.
- [x] Runtime/callable lane: implement the reached bounded
  `call_user_func_array()` behavior for string callables with ordered-array
  argument lists. Keep array/object callable forms, by-reference argument
  propagation, named-argument/spread edge cases, exceptions/warnings, autoload,
  namespace nuances beyond the current lookup table, and native lowering named
  unless implemented.
  Milestone 827 also covers the reached public `[object, method]` and
  `[class, method]` array callable shapes and advances the real
  bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:346:23: undefined function next()`.
- [x] Runtime/array lane: implement the reached bounded `next()` behavior for
  current ordered arrays in the WordPress hook iteration path. Keep full
  internal array-pointer semantics, object operands, interaction with
  `current()`/`reset()`/`end()`/`prev()`, references/copy-on-write, exact
  warnings, and native lowering named unless implemented.
  Milestone 828 covers direct variable arrays and the reached
  `$this->iterations[$level]` object-property array-offset shape, and advances
  the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:207:2: undefined function array_pop()`.
- [x] Runtime/array lane: implement the reached bounded `array_pop()` behavior
  for direct variable arrays in the WordPress hook cleanup path. Keep broad
  by-reference argument handling, non-variable array targets, object-property
  array targets unless reached and implemented, internal pointer side effects,
  references/copy-on-write, exact warnings, and native lowering named unless
  implemented.
  Milestone 829 covers direct-variable ordered-array pop mutation, empty-array
  null return, reached append-index behavior after integer-key pops, and
  string-valued direct dynamic calls, and advances the real bootstrap-shim
  probe to
  `runtime error at <bootstrap-shim>:2357:20: unsupported call mysqli_query(): only the WordPress SQL mode probe SELECT @@SESSION.sql_mode is implemented in the current subset`,
  corresponding to `wp-includes/class-wpdb.php:2357` in `wpdb::_do_query()`.
- [x] Runtime/mysqli lane: extend the bounded `mysqli_query()` placeholder for
  the reached `wpdb::_do_query()` option-query path without claiming real SQL
  execution. Keep host database behavior, result resources, row fetching,
  affected rows, errors/warnings, charset/collation, prepared statements,
  transactions, binary/invalid-string behavior, and native database calls
  named unless implemented.
  Milestone 830 covers the reached empty WordPress options-table SELECT
  placeholders plus clean `mysqli_errno()`/`mysqli_error()` bookkeeping for the
  placeholder handle, and advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:2312:8: unsupported call preg_match(): only the u pattern modifier is implemented in the current subset`,
  corresponding to the following `wpdb::query()` query-classification regex.
- [x] Runtime/regex lane: add the next bounded `preg_match()` slice for the
  reached `wpdb::query()` DDL/DML classifier regex after the empty options
  query path. Keep broad PCRE parsing, capture fidelity, non-`u`/`i` modifier
  behavior outside the exact reached patterns, binary/invalid-string behavior,
  exact diagnostics, and native lowering named unless implemented.
  Milestone 831 covers the adjacent DDL/DML/insert-replace classifier regexes
  and widens the empty options-table MySQLi placeholder for the reached
  option-name cache-priming and single-option reads. The real bootstrap-shim
  probe now advances to
  `runtime error at <bootstrap-shim>:3045:17: unsupported call empty(): only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported`.
- [x] Runtime/expression lane: extend `empty(...)` for the reached complex
  WordPress operand at `<bootstrap-shim>:3045:17` without broadening arbitrary
  lvalue support beyond tested parser/runtime shapes. Keep magic property
  hooks, ArrayAccess, references/copy-on-write, warning fidelity, and native
  lowering named unless implemented.
  Milestone 832 covers direct nested array-offset paths and direct
  object-property array-offset paths, and also widens the deterministic empty
  MySQLi placeholder for reached `SHOW FULL COLUMNS FROM ...` and
  `DESCRIBE ...` metadata probes. The real bootstrap-shim probe now advances
  to
  `parse error at <bootstrap-shim>:283:14: unsupported object static property access: object receiver static properties are not implemented`.
- [x] Parser/object lane: implement the next bounded object-receiver static
  property parsing/runtime slice for the reached WordPress bootstrap-shim
  blocker at `<bootstrap-shim>:283:14`. Keep object receiver class constants,
  `$object::class`, broader `static::` forms, compound assignment,
  increment/decrement, `isset`/`empty`/`??`/`??=`, magic hooks,
  references/copy-on-write, exact diagnostics, and native lowering named
  unless implemented.
  Milestone 833 covers `$object::$property` and `$className::$property`
  reads/direct writes for current object and declared class-name string
  receivers. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:6316:35: undefined array key "SCRIPT_FILENAME"`.
- [x] Runtime/superglobal lane: seed or otherwise bound the reached
  `$_SERVER['SCRIPT_FILENAME']` startup path in the WordPress bootstrap shim
  without claiming a full SAPI/request environment. Keep broader server
  variables, web-server request state, path translation, CGI/FPM differences,
  exact warning behavior, and native lowering named unless implemented.
  Milestone 834 seeds `SCRIPT_FILENAME` as `/index.php` in the deterministic
  CLI `$_SERVER` placeholder. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:78:47: undefined function str_ends_with()`.
- [x] Runtime/string lane: implement the reached bounded
  `str_ends_with($_SERVER['SCRIPT_FILENAME'], 'php.cgi')` path from
  `wp-includes/load.php:78`. Keep broad string coercions, binary/invalid UTF-8
  behavior, array/object/resource operands, exact diagnostics, and native
  lowering named unless implemented.
  Milestone 835 covers two-argument scalar/null string-convertible
  `str_ends_with()` for direct calls, string-valued dynamic calls,
  `function_exists`, and `is_callable`. The real bootstrap-shim probe now
  advances to
  `runtime error at <bootstrap-shim>:6335:21: undefined function substr()`.
- [x] Runtime/string lane: implement the next bounded `substr()` path reached
  by the WordPress bootstrap shim. Keep negative offset/length edge behavior,
  byte-vs-character semantics, broad scalar coercions, array/object/resource
  operands, exact diagnostics, and native lowering named unless implemented.
  Milestone 836 covers scalar/null string-convertible input, integer offsets,
  and optional integer lengths over byte positions when the result remains
  valid UTF-8. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:6337:13: unsupported call preg_replace(): only the WordPress database-version cleanup pattern /[^0-9.].*/ is implemented in the current subset`.
- [x] Runtime/regex lane: widen the bounded `preg_replace()` implementation
  for the next reached WordPress bootstrap pattern, without claiming broad PCRE
  replacement semantics. Keep callback replacement, arrays, captures/backrefs,
  modifiers, limits/count output, exact diagnostics, and native lowering named
  unless implemented.
  Milestone 837 covers the exact `#/[^/]*$#i` empty-replacement path-tail
  cleanup used by `wp_guess_url()`. The real bootstrap-shim probe now advances
  to
  `runtime error at <bootstrap-shim>:6344:23: undefined array key "HTTP_HOST"`.
- [x] Runtime/superglobal lane: seed or otherwise bound the reached
  `$_SERVER['HTTP_HOST']` startup path in the WordPress bootstrap shim without
  claiming a full SAPI/request environment. Keep host header validation,
  proxy/web-server state, request routing, HTTPS/port handling, exact warnings,
  and native lowering named unless implemented.
  Milestone 838 seeds `HTTP_HOST` as `localhost` in the deterministic CLI
  `$_SERVER` placeholder. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:6347:9: undefined function rtrim()`.
- [x] Runtime/string lane: implement the next bounded `rtrim()` path reached
  by the WordPress bootstrap shim. Keep broad charlist range behavior,
  binary/null-byte edge cases, object/resource operands, exact diagnostics, and
  native lowering named unless implemented.
  Milestone 839 covers scalar/null string-convertible `rtrim()` with the
  default PHP whitespace mask and non-empty literal character masks such as
  `/`. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:958:2: undefined function wp_redirect()`.
- [x] Runtime/WordPress bootstrap lane: decide the smallest honest boundary for
  the reached `wp_redirect()` call. Inspect whether the function should have
  been declared by the current include path before adding any runtime shim; do
  not mask an include/declaration registration bug with a broad builtin stub.
  Milestone 840 confirms `wp_redirect()` is declared conditionally in
  WordPress `pluggable.php`, and fixes the interpreter to register
  conditional/nested function declarations when execution reaches them. The
  real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:1565:15: undefined function preg_replace_callback()`.
- [x] Runtime/regex callback lane: inspect the reached
  `preg_replace_callback()` call and implement the smallest honest subset for
  that exact WordPress bootstrap shape. Do not claim broad PCRE callback
  replacement support; keep pattern arrays, subject arrays, callback forms,
  captures/backrefs, limits/count output, invalid-pattern warnings, exact
  diagnostics, closure invocation gaps, and native lowering named unless
  implemented.
  Milestone 841 covers the exact `wp_sanitize_redirect()` UTF-8 sanitizer
  pattern and `_wp_sanitize_utf8_in_redirect` string callback. The real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:1566:15: unsupported call preg_replace(): only the WordPress database-version cleanup pattern /[^0-9.].*/ and path-tail pattern #/[^/]*$#i are implemented in the current subset`.
- [x] Runtime/regex lane: widen bounded `preg_replace()` only for the reached
  WordPress redirect sanitizer cleanup pattern
  `|[^a-z0-9-~+_.?#=&;,/:%!*\[\]()@]|i` with an empty replacement string.
  Keep pattern arrays, replacement arrays, subject arrays, callbacks,
  captures/backrefs, limit/count output, invalid-pattern warnings,
  byte/Unicode edge cases, exact diagnostics, and native lowering named unless
  implemented.
  Milestone 842 covers this exact redirect sanitizer cleanup pattern. The real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:2018:13: unsupported call preg_replace(): only the WordPress database-version cleanup pattern /[^0-9.].*/, path-tail pattern #/[^/]*$#i, and redirect sanitizer cleanup pattern |[^a-z0-9-~+_.?#=&;,/:%!*\[\]()@]|i are implemented in the current subset`.
- [x] Runtime/regex lane: inspect the reached `preg_replace()` at
  `<bootstrap-shim>:2018:13`, identify the originating WordPress source path,
  and implement only the exact next pattern/replacement shape if it is a small
  honest subset. Keep broad PCRE replacement support, replacement backrefs,
  arrays, callbacks, limit/count output, exact diagnostics, and native lowering
  named unless implemented.
  Milestone 843 traces this to `wp-includes/kses.php:2018` and covers the
  exact KSES control-character cleanup pattern, the adjacent slash-zero cleanup
  pattern, and the previously reached `pluggable.php` mail-host cleanup
  pattern. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:4440:14: unsupported call str_replace(): count output arguments are not implemented; pass exactly three arguments in the current subset`.
- [x] Runtime/string lane: implement the reached `str_replace()` count-output
  argument shape without claiming broad by-reference output semantics. Inspect
  the originating WordPress source, support only direct-variable count outputs
  if appropriate, and keep array/object operands, broad references, exact PHP
  warning behavior, and native lowering named unless implemented.
  Milestone 844 traces this to `wp-includes/formatting.php:4440` in
  `_deep_replace()` and covers direct-variable count output for the existing
  scalar/null string-convertible `str_replace()` subset. The real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:4440:14: unsupported call str_replace(): search argument arrays are not implemented in the current subset`.
- [x] Runtime/string lane: implement the reached `_deep_replace()` array-search
  shape for `str_replace($search, '', $subject, $count)` without claiming broad
  PHP array replacement behavior. Inspect WordPress' caller shapes, support
  only scalar/null subject and array search values if appropriate, update count
  aggregation, and keep replacement arrays, subject arrays, nested arrays,
  object/resource coercions, exact warnings, binary string edge cases, and
  native lowering named unless implemented.
  Milestone 845 covers the reached scalar/null string-convertible search-array
  values with scalar replacement/subject and direct-variable count output. The
  real bootstrap-shim probe now exits `0` with no stdout.
- [x] WordPress harness lane: expand the compatibility target beyond the
  current bootstrap shim. Add a committed, reproducible next probe for a real
  WordPress entry flow, such as a `wp-load.php`/front-controller shaped harness
  with documented minimal config and host assumptions, without claiming plugin,
  theme, admin, REST, database, HTTP, or native WordPress support until those
  flows have executable coverage.
  Milestone 846 adds `front_controller_probe` for `wp-blog-header.php` when
  present. Against real WordPress 6.9.4 it reaches
  `wp-includes/class-wpdb.php:1511`, the `wpdb::prepare()` placeholder
  normalization `preg_replace()` pattern
  `/%(?:%|$|(?!($allowed_format)?[sdfFi]))/` with replacement `'%%\\1'`.
- [x] Runtime/regex lane: inspect and implement the reached front-controller
  `preg_replace()` pattern in `wp-includes/class-wpdb.php:1511` only if it can
  be represented as a bounded honest subset. Keep dynamic PCRE variables,
  captures/backrefs beyond the reached replacement, arrays, callbacks,
  limit/count output, exact warnings, invalid patterns, SQL/database semantics,
  and native lowering named unless implemented.
  Milestone 847 covers the exact `wpdb::prepare()` placeholder-normalization
  pattern and replacement. The real front-controller probe now advances to
  `runtime error at <wordpress-root>/wp-blog-header.php:1514:18: undefined function preg_split()`.
- [x] Runtime/regex lane: inspect and implement the reached
  `preg_split()` placeholder extraction call in
  `wp-includes/class-wpdb.php:1514` only as a bounded WordPress subset. Keep
  broad PCRE splitting, flags beyond the reached shape, delimiter capture
  behavior beyond what the call proves, offset semantics, invalid-pattern
  warnings, SQL/database semantics, and native lowering named unless
  implemented.
  Milestone 848 covers the exact `wpdb::prepare()` placeholder-extraction
  pattern with `limit` `-1` and `PREG_SPLIT_DELIM_CAPTURE`. The real
  front-controller probe now advances to
  `runtime error at <wordpress-root>/wp-blog-header.php:1763:12: undefined function vsprintf()`.
- [x] Runtime/formatting lane: inspect and implement the reached
  `vsprintf()` call in `wp-includes/class-wpdb.php:1763` only as a bounded
  WordPress `wpdb::prepare()` subset. Keep broad format-string behavior,
  argument unpacking/reference semantics, locale-sensitive formatting,
  warnings, SQL/database execution, and native lowering named unless
  implemented.
  Milestone 849 covers `vsprintf()` over current ordered arrays and expands
  the shared formatter for the reached `%s`/`%d`/`%F` WordPress prepare
  shapes. The real front-controller probe now advances to
  `runtime error at <wordpress-root>/wp-blog-header.php:935:5: unsupported call mysqli_query(): only the WordPress SQL mode probe and empty wp_options SELECT placeholders are implemented in the current subset; got SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'`.
- [x] Runtime/mysqli lane: inspect and implement the reached WordPress
  `mysqli_query()` charset setup statement
  `SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'` as a deterministic
  placeholder boundary. Keep real DB connections, result resources, charset
  negotiation, SQL execution, error-state fidelity, and native lowering named
  unless implemented.
  Milestone 850 returns `true` for that exact placeholder setup query. The real
  front-controller probe now exits `0` with no stdout under deterministic
  placeholder database and CLI assumptions; this is still not full WordPress,
  plugin/theme/admin/REST, real database, HTTP/filesystem, SAPI, or native
  support.
- [x] WordPress harness lane: add a committed deterministic fixture or smoke
  target for the now-passing `wp-blog-header.php` front-controller probe,
  while preserving the clear distinction between placeholder CLI/bootstrap
  success and full WordPress request support.
  Milestone 851 adds a synthetic normalized inventory smoke in which
  `wp-blog-header.php` loads through `wp-load.php`, `wp-config.php`,
  `wp-settings.php`, and a small `wpdb` charset setup path, then exits `0`
  with no stdout. It proves the committed harness shape only; real WordPress
  source, plugin/theme/admin/REST flows, real database/result resources,
  HTTP/filesystem/SAPI behavior, and native lowering remain outside this
  smoke.
- [x] Runtime/mysqli lane: add the first deterministic `mysqli_result`
  lifecycle boundary for reached WordPress-style empty result consumption,
  such as `mysqli_fetch_object()`, `mysqli_free_result()`,
  `mysqli_more_results()`, and `mysqli_next_result()` over placeholder empty
  results, without claiming real SQL execution or result resources.
  Milestone 852 returns a placeholder `mysqli_result` object for the exact
  synthetic empty result query `SELECT * FROM wp_posts WHERE 1 = 0`, with
  `mysqli_num_fields()` returning `0`, `mysqli_fetch_field()` and
  `mysqli_fetch_object()` returning `false`, `mysqli_free_result()` returning
  `null`, and multi-result probes returning `false` on the placeholder
  connection. Real SQL execution, non-empty rows, field metadata, result
  resources, and native database calls remain unsupported.
- [x] WordPress harness lane: add a deterministic post-bootstrap synthetic
  `wpdb::query()`/result-consumption smoke that exercises the empty
  `mysqli_result` lifecycle through a WordPress-shaped class method without
  real database state.
  Milestone 853 adds a `phpc-only` synthetic `wpdb::query()` fixture that
  stores `$this->result`, loops over `mysqli_fetch_object()`, frees the
  placeholder result, drains placeholder multi-result state, and verifies
  empty `last_result`/`num_rows` state. It is not real SQL execution,
  non-empty row hydration, field metadata, WordPress query-state fidelity, or
  native lowering.
- [x] Runtime/mysqli lane: inspect the next real WordPress database path after
  empty result consumption and choose either a bounded row-shape fixture or an
  explicit unsupported diagnostic for non-empty result sets.
  Milestone 854 chooses the explicit boundary: after recognized deterministic
  SQL-mode, charset setup, empty options/metadata, and exact empty-result
  placeholders, remaining `SELECT` queries now fail with a specific
  non-empty-result-set diagnostic. A `phpc-only` fixture covers
  `SELECT * FROM wp_posts WHERE ID = 1`; real row hydration, field metadata,
  result resources, SQL execution, and database state remain unsupported.
- [x] Runtime/mysqli lane: design and implement the first placeholder
  row-shape representation for
  deterministic `mysqli_fetch_object()` results, including how rows, fields,
  result cursors, object hydration, and error state will be represented before
  implementing any non-empty query.
  Milestone 855 adds interpreter-owned placeholder result state for the exact
  query `SELECT ID, post_title FROM wp_posts WHERE ID = 1`, exposing two
  deterministic field names and one `stdClass` row through
  `mysqli_num_fields()`, `mysqli_fetch_field()`, `mysqli_fetch_object()`, and
  `mysqli_free_result()`. It is still not SQL execution, real database state,
  WordPress content fidelity, real metadata, or native database lowering.
- [x] WordPress harness lane: add a synthetic `wpdb::get_results()` or
  equivalent one-row smoke that consumes the Milestone 855 deterministic
  row-backed result through WordPress-shaped query state without claiming real
  database support.
  Milestone 856 adds a `phpc-only` synthetic `wpdb::get_results()` fixture
  that runs the exact seed-post placeholder query, fetches one object row,
  stores it in `$this->last_result`, increments `$this->num_rows`, frees the
  result, drains placeholder multi-result state, and returns the row array.
  It is not real SQL execution, database state, WordPress query fidelity,
  cache behavior, real post content, or native lowering.
- [x] Runtime/mysqli lane: add the next explicit boundary for post-query
  operations that WordPress will need before real database support, such as
  deterministic `mysqli_fetch_assoc()`/array row hydration or a named
  unsupported diagnostic for unsupported fetch modes.
  Milestone 857 adds deterministic `mysqli_fetch_assoc()` support for the
  seed-post placeholder result. It shares the placeholder row cursor with
  `mysqli_fetch_object()`, returns an associative PHP array keyed by `ID` and
  `post_title`, then returns `false`. It is not `mysqli_fetch_array()`, numeric
  indexes, broad fetch modes, SQL execution, real database state, or native
  lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` associative-result smoke
  that consumes the Milestone 857 `mysqli_fetch_assoc()` path through a
  WordPress-shaped `ARRAY_A` or equivalent result mode without claiming real
  database support.
  Milestone 858 adds a `phpc-only` synthetic `wpdb::get_results($query,
  ARRAY_A)` fixture that takes an associative fetch branch, stores the row in
  `$this->last_result`, increments `$this->num_rows`, frees the placeholder
  result, drains placeholder multi-result state, and returns the row array.
  It is not real WordPress `wpdb` output-mode fidelity, core constants, SQL
  execution, database state, cache behavior, or native lowering.
- [x] Runtime/mysqli lane: add a named boundary or deterministic slice for the
  next common fetch mode, such as `mysqli_fetch_array()` with documented
  unsupported numeric/mixed index behavior before broadening beyond the
  seed-post placeholder query.
  Milestone 859 adds deterministic `mysqli_fetch_array($result,
  MYSQLI_ASSOC)` support for the seed-post placeholder result and exposes the
  `MYSQLI_ASSOC`/`MYSQLI_NUM`/`MYSQLI_BOTH` constants. Omitted-mode
  `MYSQLI_BOTH`, numeric rows, and mixed rows remain named unsupported
  boundaries.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that exercises a
  fallback or compatibility path using `mysqli_fetch_array($result,
  MYSQLI_ASSOC)` without claiming real `wpdb` output-mode fidelity or real
  database support.
  Milestone 860 adds a `phpc-only` synthetic `wpdb::get_results($query,
  ARRAY_A)` fixture that consumes the exact seed-post placeholder result
  through `mysqli_fetch_array($result, MYSQLI_ASSOC)`, stores the associative
  row in `$this->last_result`, increments `$this->num_rows`, frees the
  placeholder result, and drains placeholder multi-result state. It is not
  real WordPress `wpdb` output-mode fidelity, numeric or mixed fetch-array
  support, SQL execution, database state, cache behavior, warnings/errors, or
  native lowering.
- [x] Runtime/mysqli lane: add the next deterministic result-fetch slice or a
  sharper named boundary after `MYSQLI_ASSOC` fetch-array support, such as
  `mysqli_fetch_array($result, MYSQLI_NUM)`/`MYSQLI_BOTH` for the seed-post
  placeholder row or a deliberately scoped `mysqli_data_seek()`/cursor-reset
  boundary, with tests, CLI coverage, docs, and unsupported edges.
  Milestone 861 implements `mysqli_fetch_array($result, MYSQLI_NUM)`, explicit
  `MYSQLI_BOTH`, and omitted-mode default `MYSQLI_BOTH` for the deterministic
  seed-post placeholder result. Invalid modes remain a named unsupported
  boundary, and broad query/result support, duplicate-column fidelity,
  warnings/errors, SQL execution, real database state, and native lowering
  remain missing.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that exercises the
  default or mixed `mysqli_fetch_array()` row shape after Milestone 861 without
  claiming real `wpdb` output-mode fidelity or real database support.
  Milestone 862 adds a `phpc-only` synthetic `wpdb::get_results($query,
  ARRAY_A)` fixture that consumes the exact seed-post placeholder result
  through omitted-mode `mysqli_fetch_array($result)`, stores the mixed
  numeric/associative row in `$this->last_result`, increments
  `$this->num_rows`, frees the placeholder result, and drains placeholder
  multi-result state. It is not real WordPress `wpdb` output-mode fidelity,
  SQL execution, database state, cache behavior, duplicate-column behavior,
  warnings/errors, or native lowering.
- [x] Runtime/mysqli lane: inspect the next WordPress database-result boundary
  after deterministic fetch-array modes and choose a small tested slice, such
  as scoped `mysqli_data_seek($result, 0)` cursor reset for placeholder
  results, deterministic `mysqli_fetch_row()`, or a sharper named boundary for
  unsupported cursor/result operations.
  Milestone 863 chooses deterministic `mysqli_fetch_row()` for the seed-post
  placeholder result. It returns numeric keys `0` and `1`, shares the result
  cursor, returns `false` after the one row, and is visible through runtime and
  native metadata lookup. It is not broad result fetching, cursor seeking, SQL
  execution, real database state, warnings/errors, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that exercises
  `mysqli_fetch_row()` through a WordPress-shaped numeric-result branch without
  claiming real `wpdb` output-mode fidelity or real database support.
  Milestone 864 adds a `phpc-only` synthetic `wpdb::get_results($query,
  ARRAY_N)` fixture that consumes the exact seed-post placeholder result
  through `mysqli_fetch_row()`, stores the numeric row in `$this->last_result`,
  increments `$this->num_rows`, frees the placeholder result, and drains
  placeholder multi-result state. It is not real WordPress `wpdb` output-mode
  fidelity, WordPress core constants, SQL execution, database state, cache
  behavior, warnings/errors, cursor seeking, or native lowering.
- [x] Runtime/mysqli lane: add the next cursor/result operation as a
  deterministic slice or explicit named boundary, with `mysqli_data_seek()` on
  placeholder results as the likely next small target before broadening query
  execution.
  Milestone 865 adds bounded `mysqli_data_seek($result, $offset)` support for
  placeholder result state. Integer in-range offsets reset the row cursor,
  negative and out-of-range offsets return `false`, and non-integer offsets
  remain a named unsupported boundary. This is not real buffered/unbuffered
  result behavior, SQL execution, database state, warnings/errors, or native
  lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that exercises
  cursor reset/re-read behavior through `mysqli_data_seek($result, 0)` without
  claiming real `wpdb` result caching or database support.
  Milestone 866 adds a `phpc-only` synthetic `wpdb::get_results($query,
  ARRAY_A)` fixture that consumes the exact seed-post placeholder result once
  through `mysqli_fetch_assoc()`, rewinds it with `mysqli_data_seek($result,
  0)`, consumes it again through `mysqli_fetch_row()`, frees the placeholder
  result, and drains placeholder multi-result state. It is not real WordPress
  `wpdb` result caching, SQL execution, database state, cache behavior,
  warnings/errors, broad cursor semantics, or native lowering.
- [x] Runtime/mysqli lane: inspect the next result/database boundary after
  placeholder cursor reset and choose a small tested slice, such as deterministic
  `mysqli_num_rows()` for placeholder results or a named boundary for affected
  rows/insert IDs before any broader SQL execution.
  Milestone 867 implements bounded `mysqli_num_rows($result)` for placeholder
  result state. It returns the stored buffered row count for the exact empty
  result and deterministic seed-post result, does not advance the fetch cursor,
  and is visible through runtime and native metadata lookup. This is not real
  buffered/unbuffered result behavior, SQL execution, database state,
  affected-row/insert-id state, warnings/errors, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that exercises
  `mysqli_num_rows()` through a WordPress-shaped result-count path without
  claiming real `wpdb` database state or query fidelity.
  Milestone 868 adds a `phpc-only` synthetic `wpdb::get_results($query,
  ARRAY_A)` fixture that sets `$this->num_rows` from `mysqli_num_rows()` for
  both the exact empty-result placeholder and deterministic seed-post
  placeholder before consuming rows through `mysqli_fetch_assoc()`. It is not
  real WordPress `wpdb` query fidelity, SQL execution, database state, cache
  behavior, affected-row/insert-id state, warnings/errors, broad result-count
  semantics, or native lowering.
- [x] Runtime/mysqli lane: inspect the next mutation/result metadata boundary
  after placeholder row counts and choose a small tested slice, such as
  deterministic `mysqli_affected_rows()`/`mysqli_insert_id()` state for
  no-op placeholder queries or an explicit named boundary before broader SQL
  execution.
  Milestone 869 implements bounded clean-state `mysqli_affected_rows($handle)`
  and `mysqli_insert_id($handle)` for placeholder `mysqli` objects. Both return
  deterministic `0` before and after the reached charset setup no-result query,
  reject non-`mysqli` handles with stable diagnostics, and are visible through
  runtime and native metadata lookup. This is not mutation SQL execution, real
  affected-row/insert-id state, transactions, warnings/errors, host database
  integration, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_affected_rows()`/`mysqli_insert_id()` clean state through
  WordPress-shaped query bookkeeping without claiming real mutation query
  behavior.
  Milestone 870 adds a `phpc-only` synthetic `wpdb::query()` fixture that
  records `$this->rows_affected = 0` and `$this->insert_id = 0` from
  `mysqli_affected_rows()`/`mysqli_insert_id()` after the exact charset setup
  no-result query. It is not real WordPress mutation query behavior, SQL
  execution, database state, transactions, warnings/errors, real
  affected-row/insert-id state, or native lowering.
- [x] Runtime/mysqli lane: inspect the next database boundary after clean
  mutation metadata and choose a small tested slice, such as an explicit
  `mysqli_set_charset()` placeholder boundary or a named unsupported diagnostic
  for mutation SQL before broadening query execution.
  Milestone 871 implements bounded `mysqli_set_charset($handle, $charset)` for
  placeholder `mysqli` objects. It accepts string `utf8mb4`
  case-insensitively, returns deterministic `true`, rejects non-`mysqli`
  handles, non-string charsets, and unsupported charset names with stable
  diagnostics, and is visible through runtime and native metadata lookup. This
  is not real connection charset negotiation, collation state, escaping charset
  fidelity, warnings/errors, host database integration, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that exercises
  `mysqli_set_charset()` through a WordPress-shaped charset setup method
  without claiming real charset/collation behavior.
  Milestone 872 adds a `phpc-only` synthetic `wpdb::set_charset()` fixture that
  calls `mysqli_set_charset($this->dbh, "utf8mb4")`, records the requested
  charset/collation properties, and records the successful placeholder result.
  It is not real WordPress charset negotiation, collation behavior, connection
  state, SQL escaping charset fidelity, warnings/errors, host database
  integration, or native lowering.
- [x] Runtime/mysqli lane: inspect the next database boundary after charset
  setup and choose a small tested slice, such as a named unsupported diagnostic
  for mutation SQL (`INSERT`/`UPDATE`/`DELETE`) or a deterministic placeholder
  for one reached WordPress metadata/update query before broader SQL execution.
  Milestone 873 adds a sharper `mysqli_query()` mutation-SQL boundary for
  `INSERT`, `UPDATE`, `DELETE`, and `REPLACE` query strings. These now report
  that mutation SQL is not implemented and that affected-row/insert-id state is
  deterministic clean placeholder metadata, instead of falling through to the
  generic query rejection. This is not mutation SQL execution, real
  affected-row/insert-id state, transactions, database state, warnings/errors,
  host database integration, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that exercises the
  explicit mutation-SQL boundary through WordPress-shaped query bookkeeping
  without claiming real update/insert/delete behavior.
  Milestone 874 adds a `phpc-only` synthetic `wpdb::query()` fixture that
  records the attempted `UPDATE wp_options ...` query and then reaches the
  stable `mysqli_query()` mutation-SQL unsupported diagnostic. It is not real
  update/insert/delete behavior, SQL execution, affected-row or insert-id
  mutation, transactions, database state, warnings/errors, host database
  integration, partial-output fidelity, or native lowering.
- [x] Runtime/mysqli lane: inspect the next database boundary after explicit
  mutation-query rejection and choose a small tested slice, such as
  deterministic connection-state metadata (`mysqli_ping()`,
  `mysqli_get_host_info()`, or `mysqli_stat()`) or a sharper named boundary
  for transactions before broader SQL execution.
  Milestone 875 implements bounded `mysqli_ping($handle)` for placeholder
  `mysqli` objects. It returns deterministic `true`, rejects non-`mysqli`
  handles with stable diagnostics, and is visible through runtime and native
  metadata lookup. This is not a real liveness check, reconnect behavior,
  socket I/O, host database integration, warnings/errors, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that exercises the
  deterministic `mysqli_ping()` placeholder through a WordPress-shaped
  connection-check method without claiming real reconnection or database
  liveness behavior.
  Milestone 876 adds a `phpc-only` synthetic `wpdb::check_connection()`
  fixture that calls `mysqli_ping($this->dbh)`, records that the check ran,
  and records deterministic ready state after placeholder success. It is not
  real WordPress reconnection behavior, socket I/O, host database integration,
  warnings/errors, real `wpdb` state fidelity, or native lowering.
- [x] Runtime/mysqli lane: inspect the next connection metadata boundary after
  placeholder ping and choose a small tested slice, such as deterministic
  `mysqli_get_host_info()` or `mysqli_stat()` metadata, before any real host
  database state is claimed.
  Milestone 877 implements bounded `mysqli_get_host_info($handle)` for
  placeholder `mysqli` objects. It returns deterministic
  `localhost via TCP/IP (phpc-placeholder)`, rejects non-`mysqli` handles with
  stable diagnostics, and is visible through runtime and native metadata
  lookup. This is not real host, transport, socket, protocol,
  connection-liveness, reconnect, warning/error, or native database behavior.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records
  deterministic `mysqli_get_host_info()` through WordPress-shaped connection
  metadata bookkeeping without claiming real host or transport state.
  Milestone 878 adds a `phpc-only` synthetic `wpdb` fixture that calls
  `mysqli_get_host_info($this->dbh)`, records the deterministic placeholder
  host-info string on local object state, and records that the check ran. It
  is not real WordPress connection metadata fidelity, host/transport/socket
  state, protocol metadata, live connection inspection, warnings/errors, host
  database integration, or native lowering.
- [x] Runtime/mysqli lane: inspect the next connection metadata boundary after
  host-info placeholders and choose a small tested slice, such as deterministic
  `mysqli_stat()` metadata or a named transaction/autocommit boundary, before
  broader SQL execution or real host state is claimed.
  Milestone 879 implements bounded `mysqli_stat($handle)` for placeholder
  `mysqli` objects. It returns deterministic zeroed server-status metadata,
  rejects non-`mysqli` handles with stable diagnostics, and is visible through
  runtime and native metadata lookup. This is not real server status, live
  connection inspection, query counters, thread/table metadata, host database
  integration, warnings/errors, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records
  deterministic `mysqli_stat()` output through WordPress-shaped connection
  status bookkeeping without claiming real server status or counters.
  Milestone 880 adds a `phpc-only` synthetic `wpdb` fixture that calls
  `mysqli_stat($this->dbh)`, records the deterministic zeroed status string on
  local object state, and records that the status check ran. It is not real
  WordPress server-status fidelity, live connection inspection, query counters,
  thread/table metadata, host database integration, warnings/errors, or native
  lowering.
- [x] Runtime/mysqli lane: inspect the next database boundary after
  placeholder server metadata and choose a small tested slice, such as a named
  `mysqli_autocommit()`/transaction boundary or deterministic placeholder
  error-state metadata, before broader SQL execution is claimed.
  Milestone 881 implements bounded `mysqli_autocommit($handle, $mode)` for
  placeholder `mysqli` objects. It accepts boolean modes, returns
  deterministic `true`, rejects non-`mysqli` handles and non-bool modes with
  stable diagnostics, and is visible through runtime and native metadata
  lookup. This is not real autocommit state, transaction start/end,
  commit/rollback behavior, host database integration, warnings/errors, or
  native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that exercises the
  bounded `mysqli_autocommit()` placeholder through a WordPress-shaped
  transaction/autocommit bookkeeping method without claiming real transaction
  or database-state behavior.
  Milestone 882 adds a `phpc-only` synthetic `wpdb` fixture that toggles
  placeholder autocommit off and on through `mysqli_autocommit($this->dbh,
  false/true)`, records local bookkeeping state, and records that both
  placeholder calls ran. It is not real WordPress transaction behavior, real
  autocommit state, commit/rollback behavior, host database integration,
  warnings/errors, or native lowering.
- [x] Runtime/mysqli lane: inspect the next database boundary after
  autocommit placeholders and choose a small tested slice, such as a named
  `mysqli_begin_transaction()`/`mysqli_commit()`/`mysqli_rollback()` boundary
  or deterministic placeholder error-state metadata, before broader SQL
  execution is claimed.
  Milestone 883 implements bounded `mysqli_begin_transaction($handle, $flags,
  $name)` for placeholder `mysqli` objects. It accepts omitted flags/name,
  flags value `0`, and null/string names, returns deterministic `true`,
  rejects non-`mysqli` handles, nonzero flags, non-int flags, and unsupported
  name values with stable diagnostics, and is visible through runtime and
  native metadata lookup. This is not real transaction state, autocommit state
  mutation, commit/rollback behavior, savepoints, host database integration,
  warnings/errors, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that exercises the
  bounded `mysqli_begin_transaction()` placeholder through WordPress-shaped
  transaction bookkeeping without claiming real transaction or database-state
  behavior.
  Milestone 884 adds a `phpc-only` synthetic `wpdb` fixture that calls
  `mysqli_begin_transaction($this->dbh, 0, $name)`, records local transaction
  bookkeeping state, and records that the placeholder transaction-start path
  ran. It is not real WordPress transaction behavior, real transaction state,
  autocommit state mutation, commit/rollback behavior, savepoints, host
  database integration, warnings/errors, or native lowering.
- [x] Runtime/mysqli lane: inspect the next transaction boundary after
  begin-transaction placeholders and choose a small tested slice, such as
  bounded `mysqli_commit()`/`mysqli_rollback()` placeholder success or a
  deterministic placeholder error-state boundary, before broader SQL execution
  is claimed.
  Milestone 885 implements bounded `mysqli_commit($handle, $flags, $name)` and
  `mysqli_rollback($handle, $flags, $name)` for placeholder `mysqli` objects.
  They accept omitted flags/name, flags value `0`, and null/string names,
  return deterministic `true`, reject non-`mysqli` handles, nonzero flags,
  non-int flags, and unsupported name values with stable diagnostics, and are
  visible through runtime and native metadata lookup. This is not real
  commit/rollback behavior, transaction state, autocommit state mutation,
  savepoints, host database integration, warnings/errors, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that exercises the
  bounded `mysqli_commit()`/`mysqli_rollback()` placeholders through
  WordPress-shaped transaction bookkeeping without claiming real transaction or
  database-state behavior.
  Milestone 886 adds a `phpc-only` synthetic `wpdb` fixture that begins a
  placeholder transaction, commits it, begins another placeholder transaction,
  rolls it back, and records local transaction bookkeeping state. It is not
  real WordPress transaction behavior, real transaction state, autocommit state
  mutation, database mutation, savepoints, host database integration,
  warnings/errors, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi state boundary after
  placeholder transaction completion and choose a small tested slice, such as
  deterministic `mysqli_sqlstate()`/`mysqli_warning_count()` placeholder
  error-state metadata or a sharper named diagnostic, before broader SQL
  execution is claimed.
  Milestone 887 implements bounded `mysqli_sqlstate($handle)` and
  `mysqli_warning_count($handle)` for placeholder `mysqli` objects. They
  return deterministic clean-state metadata (`00000` and `0`), reject
  non-`mysqli` handles with stable diagnostics, and are visible through runtime
  and native metadata lookup. This is not real SQLSTATE tracking,
  warning-count tracking, host database integration, warnings/errors, or
  native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_sqlstate()`/`mysqli_warning_count()` placeholders through
  WordPress-shaped error-state bookkeeping without claiming real database
  warning/error fidelity.
  Milestone 888 adds a `phpc-only` synthetic `wpdb` fixture that calls
  `mysqli_errno()`, `mysqli_error()`, `mysqli_sqlstate()`, and
  `mysqli_warning_count()` on the placeholder handle, records local error-state
  bookkeeping, and verifies the clean deterministic metadata. It is not real
  WordPress database error fidelity, SQLSTATE tracking, warning-count tracking,
  host database integration, warnings/errors, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi boundary after clean
  error-state metadata and choose a small tested slice, such as deterministic
  `mysqli_get_client_info()`/`mysqli_get_proto_info()` metadata or a sharper
  named diagnostic, before broader SQL execution or real host state is claimed.
  Milestone 889 implements bounded deterministic client/protocol metadata for
  placeholder `mysqli` objects. `mysqli_get_client_info()` accepts no argument,
  `null`, or a current placeholder handle and returns
  `mysqlnd 8.0.0-phpc-placeholder`; `mysqli_get_proto_info($handle)` returns
  protocol version `10`; unsupported forms report stable diagnostics; both
  names are visible through runtime and native metadata lookup. This is not
  real client-library detection, protocol negotiation, host connection
  metadata, PHP deprecation/warning fidelity, or native database lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_get_client_info()`/`mysqli_get_proto_info()` placeholders
  through WordPress-shaped connection metadata bookkeeping without claiming
  real database client/protocol fidelity.
  Milestone 890 adds a `phpc-only` synthetic `wpdb` fixture that records
  deterministic placeholder client info and protocol version on local object
  state and verifies that the metadata check ran. It is not real WordPress
  database client/protocol fidelity, client-library detection, protocol
  negotiation, host database integration, warnings/errors, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi boundary after
  client/protocol placeholder metadata and choose a small tested slice, such as
  deterministic `mysqli_get_client_version()` metadata or a sharper named
  diagnostic, before broader SQL execution or real host state is claimed.
  Milestone 891 implements bounded deterministic
  `mysqli_get_client_version()` metadata. It accepts no arguments, returns
  integer version `80000`, rejects argument-bearing calls with stable arity
  diagnostics, and is visible through runtime and native metadata lookup. This
  is not real client-library version detection, host database integration, PHP
  extension configuration fidelity, or native database lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_get_client_version()` placeholder through WordPress-shaped
  connection metadata bookkeeping without claiming real database client-version
  fidelity.
  Milestone 892 adds a `phpc-only` synthetic `wpdb` fixture that records
  deterministic placeholder client-version metadata on local object state and
  verifies that the metadata check ran. It is not real WordPress database
  client-version fidelity, client-library version detection, host database
  integration, extension configuration fidelity, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi metadata boundary after
  client-version placeholders and choose a small tested slice, such as
  deterministic `mysqli_get_server_version()` metadata or a sharper named
  diagnostic, before broader SQL execution or real host state is claimed.
  Milestone 893 implements bounded deterministic
  `mysqli_get_server_version($handle)` metadata. It accepts current
  placeholder `mysqli` handles, returns integer version `80000` matching the
  existing `8.0.0-phpc-placeholder` server-info string, rejects non-`mysqli`
  handles with stable diagnostics, and is visible through runtime and native
  metadata lookup. This is not real server-version detection, host database
  integration, protocol negotiation, server capability inspection,
  warnings/errors, or native database lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_get_server_version()` placeholder through WordPress-shaped
  connection metadata bookkeeping without claiming real database server-version
  fidelity.
  Milestone 894 adds a `phpc-only` synthetic `wpdb` fixture that records
  deterministic placeholder server info and server-version metadata on local
  object state and verifies that the metadata check ran. It is not real
  WordPress database server-version fidelity, server-version detection, host
  database integration, protocol negotiation, server capability inspection,
  warnings/errors, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi boundary after server
  metadata placeholders and choose a small tested slice, such as deterministic
  `mysqli_get_connection_stats()` empty metadata or a sharper named diagnostic,
  before broader SQL execution or real host state is claimed.
  Milestone 895 implements bounded deterministic
  `mysqli_get_connection_stats($handle)` metadata. It accepts current
  placeholder `mysqli` handles, returns an eight-key array with stable zeroed
  traffic/query counters and deterministic placeholder connection counters,
  rejects non-`mysqli` handles with stable diagnostics, and is visible through
  runtime and native metadata lookup. This is not real mysqlnd statistics,
  client/server traffic accounting, query accounting, memory accounting,
  connection reuse state, host database integration, warnings/errors, or native
  database lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_get_connection_stats()` placeholder through WordPress-shaped
  connection statistics bookkeeping without claiming real mysqlnd/client
  statistics fidelity.
  Milestone 896 adds a `phpc-only` synthetic `wpdb` fixture that records the
  deterministic placeholder connection-statistics array on local object state
  and verifies stable traffic/query/connection counters. It is not real
  WordPress database connection-statistics fidelity, mysqlnd statistics,
  client/server traffic accounting, query accounting, memory accounting,
  connection reuse state, host database integration, warnings/errors, or
  native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi boundary after
  connection-statistics placeholders and choose a small tested slice, such as
  deterministic `mysqli_thread_id()`/`mysqli_get_charset()` metadata or a
  sharper named diagnostic, before broader SQL execution or real host state is
  claimed.
  Milestone 897 implements bounded deterministic `mysqli_thread_id($handle)`
  metadata. It accepts current placeholder `mysqli` handles, returns
  deterministic integer id `1`, rejects non-`mysqli` handles with stable
  diagnostics, and is visible through runtime and native metadata lookup. This
  is not real server-thread inspection, server-side thread allocation,
  connection identity, reconnect behavior, `mysqli_kill()` integration, host
  database integration, warnings/errors, or native database lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_thread_id()` placeholder through WordPress-shaped connection
  metadata bookkeeping without claiming real server-thread or connection-id
  fidelity.
  Milestone 898 adds a `phpc-only` synthetic `wpdb` fixture that records
  deterministic placeholder thread-id metadata on local object state and
  verifies that the metadata check ran. It is not real WordPress database
  connection identity, server-thread fidelity, reconnect behavior,
  `mysqli_kill()` integration, host database integration, warnings/errors, or
  native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi boundary after thread-id
  placeholders and choose a small tested slice, such as deterministic
  `mysqli_get_charset()` metadata or a sharper named diagnostic, before
  broader SQL execution or real host state is claimed.
  Milestone 899 implements bounded deterministic `mysqli_get_charset($handle)`
  metadata. It accepts current placeholder `mysqli` handles, returns a
  `stdClass`-shaped object with stable utf8mb4 charset/collation fields,
  rejects non-`mysqli` handles with stable diagnostics, and is visible through
  runtime and native metadata lookup. This is not real charset negotiation,
  client-library/server metadata inspection, collation state, charset mutation
  tracking, escaping behavior changes, warnings/errors, or native database
  lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_get_charset()` placeholder through WordPress-shaped charset
  metadata bookkeeping without claiming real charset/collation negotiation or
  escaping fidelity.
  Milestone 900 adds a `phpc-only` synthetic `wpdb` fixture that records
  deterministic placeholder charset, collation, and charset-number metadata on
  local object state and verifies that the metadata check ran. It is not real
  WordPress charset/collation negotiation, connection charset state, escaping
  fidelity, client-library/server metadata inspection, host database
  integration, warnings/errors, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi boundary after charset
  metadata placeholders and choose a small tested slice, such as deterministic
  `mysqli_character_set_name()` metadata or a sharper named diagnostic, before
  broader SQL execution or real host state is claimed.
  Milestone 901 implements bounded deterministic
  `mysqli_character_set_name($handle)` metadata. It accepts current
  placeholder `mysqli` handles, returns deterministic `utf8mb4`, rejects
  non-`mysqli` handles with stable diagnostics, and is visible through runtime
  and native metadata lookup. This is not real charset negotiation,
  client-library/server metadata inspection, connection charset state tracking,
  collation state, escaping behavior changes, warnings/errors, or native
  database lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_character_set_name()` placeholder through WordPress-shaped
  charset-name bookkeeping without claiming real charset negotiation or
  escaping fidelity.
  Milestone 902 adds a `phpc-only` synthetic `wpdb` fixture that records
  deterministic placeholder charset-name metadata on local object state and
  verifies that the metadata check ran. It is not real WordPress charset
  negotiation, connection charset state, escaping fidelity,
  client-library/server metadata inspection, host database integration,
  warnings/errors, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi boundary after charset-name
  placeholders and choose a small tested slice, such as deterministic
  `mysqli_field_count()` metadata or a sharper named diagnostic, before
  broader SQL execution or real host state is claimed.
  Milestone 903 implements bounded deterministic `mysqli_field_count($handle)`
  metadata. It accepts current placeholder `mysqli` handles, returns
  deterministic clean-state field count `0`, rejects non-`mysqli` handles with
  stable diagnostics, and is visible through runtime and native metadata
  lookup. This is not most-recent-query tracking, result metadata tracking,
  SQL execution state, host database integration, warnings/errors, or native
  database lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_field_count()` placeholder through WordPress-shaped query
  metadata bookkeeping without claiming real last-query field-count fidelity.
  Milestone 904 adds a `phpc-only` synthetic `wpdb` fixture that records the
  deterministic clean field-count placeholder on local query metadata and
  verifies that the metadata check ran. It is not real WordPress last-query
  field-count fidelity, result metadata tracking, SQL execution state, host
  database integration, warnings/errors, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi connection or result
  lifecycle boundary used by WordPress, such as `mysqli_close()` or sharper
  result-state diagnostics, before claiming real connection teardown or host
  database state.
  Milestone 905 implements bounded deterministic `mysqli_close($handle)`
  lifecycle support. It accepts current placeholder `mysqli` handles, returns
  deterministic `true`, rejects non-`mysqli` handles with stable diagnostics,
  and is visible through runtime and native metadata lookup. This is not real
  host connection teardown, handle invalidation, server resource release,
  close-after-use diagnostics, warnings/errors, or native database lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_close()` placeholder through a WordPress-shaped connection
  teardown method without claiming real disconnect behavior or resource
  lifecycle fidelity.
  Milestone 906 adds a `phpc-only` synthetic `wpdb` fixture that records the
  placeholder close result on local connection bookkeeping and verifies that
  the close path ran. It is not real WordPress disconnect behavior, host
  connection teardown, handle invalidation, server resource release,
  close-after-use diagnostics, warnings/errors, or native database lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi option or lifecycle
  boundary used by WordPress, such as `mysqli_options()` and the related option
  constants, before claiming real client-option negotiation or host connection
  state.
  Milestone 907 implements bounded deterministic `mysqli_options($handle,
  MYSQLI_OPT_INT_AND_FLOAT_NATIVE, $value)` support for bool/int option values.
  It exposes `MYSQLI_OPT_INT_AND_FLOAT_NATIVE` as `201`, returns deterministic
  `true`, rejects unsupported handles/options/values with stable diagnostics,
  and is visible through runtime and native metadata lookup. This is not real
  client-option negotiation, result type-conversion behavior, connection state
  mutation, warnings/errors, host database integration, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_options()` placeholder through a WordPress-shaped connection
  initialization method without claiming real client-option behavior.
  Milestone 908 adds a `phpc-only` synthetic `wpdb` fixture that records the
  placeholder `MYSQLI_OPT_INT_AND_FLOAT_NATIVE` option result on local
  connection-option bookkeeping before placeholder connect. It is not real
  WordPress client-option behavior, result type-conversion behavior,
  connection state mutation, host database integration, warnings/errors, or
  native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi diagnostic or metadata
  boundary used by WordPress after options setup, such as connect-time
  errno/error state after option handling or a sharper unsupported diagnostic,
  before claiming real host connection state.
  Milestone 909 implements bounded deterministic `mysqli_connect_errno()` and
  `mysqli_connect_error()` clean-state support. They accept no arguments,
  return `0` and `null`, reject argument-bearing calls with stable arity
  diagnostics, and are visible through runtime and native metadata lookup.
  This is not failed connection tracking, host extension error state,
  report-mode behavior, warnings/errors/exceptions, or native database
  lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_connect_errno()`/`mysqli_connect_error()` placeholders
  through a WordPress-shaped connection error-state method without claiming real
  connection failure fidelity.
  Milestone 910 adds a `phpc-only` synthetic `wpdb` fixture that records the
  deterministic clean procedural connect-error placeholders on local
  connection error bookkeeping after placeholder options/connect. It is not
  real WordPress connection failure fidelity, host extension error state,
  report-mode behavior, warnings/errors/exceptions, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi diagnostic or metadata
  boundary used by WordPress after clean connect-error state, such as
  `mysqli_info()` or a sharper unsupported diagnostic, before claiming real SQL
  execution metadata.
  Milestone 911 implements bounded deterministic `mysqli_info($handle)` clean
  statement-information support. It accepts current placeholder `mysqli`
  handles, returns `null`, rejects non-`mysqli` handles with stable
  diagnostics, and is visible through runtime and native metadata lookup. This
  is not real statement information tracking, mutation summaries, warnings,
  errors, host database state, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_info()` placeholder through a WordPress-shaped query
  bookkeeping method without claiming real SQL statement information.
  Milestone 912 adds a `phpc-only` synthetic `wpdb` fixture that records clean
  `mysqli_info()` placeholder metadata after a placeholder charset setup query.
  It is not real SQL statement information, mutation summaries, warnings,
  errors, host database state, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi warning or diagnostic
  boundary used by WordPress after clean statement-information state, such as
  `mysqli_get_warnings()` or a sharper unsupported diagnostic, before claiming
  real warning metadata.
  Milestone 913 implements bounded deterministic `mysqli_get_warnings($handle)`
  clean warning-chain support. It accepts current placeholder `mysqli` handles,
  returns `false`, rejects non-`mysqli` handles with stable diagnostics, and is
  visible through runtime and native metadata lookup. This is not real warning
  objects, warning iteration, SQL warning metadata, host database state,
  warnings/errors, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_get_warnings()` placeholder through a WordPress-shaped query
  warning bookkeeping method without claiming real SQL warning metadata.
  Milestone 914 adds a `phpc-only` synthetic `wpdb` fixture that records clean
  `mysqli_get_warnings()` placeholder metadata after a placeholder charset setup
  query. It is not real SQL warning objects, warning iteration, warnings,
  errors, host database state, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi result or metadata boundary
  used by WordPress after clean warning-chain state, such as
  `mysqli_store_result()`/`mysqli_use_result()` or a sharper unsupported
  diagnostic, before claiming broader result lifecycle fidelity.
  Milestone 915 implements bounded deterministic
  `mysqli_store_result($handle)`/`mysqli_use_result($handle)` clean
  no-pending-result support. They accept current placeholder `mysqli` handles,
  return `false`, reject non-`mysqli` handles with stable diagnostics, and are
  visible through runtime and native metadata lookup. This is not buffered or
  unbuffered result transfer, pending result tracking, warnings/errors, host
  database state, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_store_result()`/`mysqli_use_result()` placeholders through a
  WordPress-shaped connection result-drain method without claiming real result
  buffering or unbuffered result lifecycle fidelity.
  Milestone 916 adds a `phpc-only` synthetic `wpdb` fixture that records clean
  `mysqli_store_result()`/`mysqli_use_result()` placeholder metadata after a
  placeholder charset setup query. It is not real result buffering,
  unbuffered result lifecycle behavior, pending result tracking, warnings,
  errors, host database state, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi result cleanup or metadata
  boundary used by WordPress after connection-level store/use result clean
  state, such as `mysqli_kill()` or a sharper unsupported diagnostic, before
  claiming broader connection result lifecycle fidelity.
  Milestone 917 implements bounded deterministic `mysqli_kill($handle,
  $process_id)` support for current placeholder `mysqli` handles and integer
  process ids. It returns `true` only for the deterministic placeholder
  `mysqli_thread_id()` value `1`, returns `false` for other ids, rejects
  unsupported handles/process-id values with stable diagnostics, and is visible
  through runtime and native metadata lookup. This is not real server-thread
  killing, connection invalidation, reconnect behavior, warnings/errors, host
  database state, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_kill()` placeholder through a WordPress-shaped connection
  thread lifecycle method without claiming real thread killing, reconnect, or
  connection invalidation fidelity.
  Milestone 918 adds a `phpc-only` synthetic `wpdb` fixture that records
  deterministic `mysqli_thread_id()` plus `mysqli_kill()` placeholder metadata
  on local connection-thread bookkeeping and verifies that the placeholder
  connection remains open. It is not real server-thread killing, connection
  invalidation, reconnect behavior, warnings/errors, host database state, or
  native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi connection lifecycle or
  metadata boundary after placeholder thread-kill bookkeeping, such as
  `mysqli_refresh()`/`mysqli_change_user()` or a sharper unsupported
  diagnostic, before claiming broader connection lifecycle fidelity.
  Milestone 919 implements bounded deterministic
  `mysqli_change_user($handle, $username, $password, $database)` support for
  current placeholder `mysqli` handles, string credentials, and string or null
  database names. It returns deterministic `true`, rejects unsupported
  handles/credentials/database values with stable diagnostics, and is visible
  through runtime and native metadata lookup. This is not real authentication,
  database selection, server session reset, transaction rollback,
  temporary-table cleanup, locked-table cleanup, host database state,
  warnings/errors, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_change_user()` placeholder through a WordPress-shaped
  connection user/database lifecycle method without claiming real
  authentication, selected-database, or session-reset fidelity.
  Milestone 920 adds a `phpc-only` synthetic `wpdb` fixture that records
  deterministic `mysqli_change_user()` placeholder metadata on local
  user/database-change bookkeeping and verifies that the placeholder connection
  remains open. It is not real authentication, database selection, server
  session reset, transaction rollback, temporary-table cleanup, locked-table
  cleanup, warnings/errors, host database state, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi connection/session
  lifecycle boundary after placeholder user/database-change bookkeeping, such
  as `mysqli_refresh()` or a sharper unsupported diagnostic, before claiming
  broader connection/session lifecycle fidelity.
  Milestone 921 implements bounded deterministic `mysqli_refresh($handle,
  $flags)` support for current placeholder `mysqli` handles and nonzero
  integer combinations of exposed deprecated `MYSQLI_REFRESH_*` flags. It
  returns deterministic `true`, rejects unsupported handles/flags with stable
  diagnostics, exposes `MYSQLI_REFRESH_REPLICA` as an alias of
  `MYSQLI_REFRESH_SLAVE`, and is visible through runtime and native metadata
  lookup. This is not real table/log/cache flush behavior, replication reset,
  server status reset, connection/session mutation, PHP deprecation/warning
  fidelity, host database state, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_refresh()` placeholder through a WordPress-shaped connection
  refresh method without claiming real flush, replication reset, status reset,
  or session mutation fidelity.
  Milestone 922 adds a `phpc-only` synthetic `wpdb` fixture that records
  deterministic `mysqli_refresh()` placeholder metadata on local refresh
  bookkeeping, verifies the `MYSQLI_REFRESH_REPLICA` alias, and verifies that
  the placeholder connection remains open. It is not real flush behavior,
  replication reset, server status reset, connection/session mutation,
  warnings/errors, host database state, PHP deprecation fidelity, or native
  lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi execution or connection
  boundary after placeholder refresh bookkeeping, such as `mysqli_real_query()`
  or a sharper unsupported diagnostic, before claiming broader query execution
  fidelity.
  Milestone 923 implements bounded deterministic `mysqli_real_query($handle,
  $query)` support for current placeholder `mysqli` handles and the exact
  WordPress charset setup statement. It returns deterministic `true`, rejects
  result-producing SQL before pending result state is claimed, rejects mutation
  SQL and unsupported query values with stable diagnostics, and is visible
  through runtime and native metadata lookup. This is not real query execution,
  pending result tracking for `mysqli_store_result()`/`mysqli_use_result()`,
  result object creation, mutation state, host database state, warnings/errors,
  or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_real_query()` charset setup placeholder through a
  WordPress-shaped connection query method without claiming real query
  execution, pending result, or connection charset mutation fidelity.
  Milestone 924 adds a `phpc-only` synthetic `wpdb` fixture that records
  deterministic `mysqli_real_query()` charset setup placeholder success through
  local query bookkeeping, then verifies clean no-pending-result
  `mysqli_store_result()` and `mysqli_use_result()` metadata. It is not real
  SQL execution, pending result tracking, result object creation, mutation
  state, connection charset mutation, warnings/errors, host database state, or
  native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi execution boundary after
  placeholder `mysqli_real_query()` bookkeeping, such as `mysqli_multi_query()`
  or a sharper unsupported diagnostic for unsupported query/pending-result
  state, before claiming broader SQL execution or multi-result fidelity.
  Milestone 925 implements bounded deterministic `mysqli_multi_query($handle,
  $query)` support for current placeholder `mysqli` handles and the exact
  WordPress charset setup statement. It returns deterministic `true`, rejects
  multi-statement SQL, result-producing SQL, mutation SQL, and unsupported
  query values with stable diagnostics, and is visible through runtime and
  native metadata lookup. This is not real multi-statement execution, pending
  result queues, `mysqli_more_results()`/`mysqli_next_result()` state, result
  object creation, mutation state, host database state, warnings/errors, or
  native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_multi_query()` charset setup placeholder through a
  WordPress-shaped connection query method without claiming real
  multi-statement execution, pending result queues, or connection charset
  mutation fidelity.
  Milestone 926 adds a `phpc-only` synthetic `wpdb` fixture that records
  deterministic `mysqli_multi_query()` charset setup placeholder success
  through local query bookkeeping, then verifies clean no-more-results and
  no-pending-result `mysqli_more_results()`, `mysqli_next_result()`,
  `mysqli_store_result()`, and `mysqli_use_result()` metadata. It is not real
  multi-statement execution, pending result queues, result object creation,
  mutation state, connection charset mutation, warnings/errors, host database
  state, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi asynchronous or
  multi-result execution boundary after placeholder `mysqli_multi_query()`
  bookkeeping, such as `mysqli_reap_async_query()`/`mysqli_poll()` visibility
  or a sharper unsupported diagnostic, before claiming broader async,
  multi-result, or host database execution fidelity.
  Milestone 927 implements bounded deterministic
  `mysqli_reap_async_query($handle)` support for current placeholder `mysqli`
  handles. It returns deterministic `false` for no pending async result,
  rejects unsupported handles with stable diagnostics, and is visible through
  runtime and native metadata lookup. This is not `MYSQLI_ASYNC`,
  `mysqli_poll()`, async socket readiness, pending async result queues, result
  object creation, host database state, warnings/errors, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_reap_async_query()` clean no-async-result placeholder through
  a WordPress-shaped connection method without claiming real async query,
  `mysqli_poll()`, socket-readiness, or host database execution fidelity.
  Milestone 928 adds a `phpc-only` synthetic `wpdb` fixture that records
  deterministic `mysqli_reap_async_query()` clean no-async-result metadata
  through local async-result bookkeeping and verifies that the placeholder
  connection remains open. It is not real async query execution,
  `MYSQLI_ASYNC`, `mysqli_poll()`, socket readiness, pending async result
  queues, warnings/errors, host database state, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi async readiness boundary
  after clean `mysqli_reap_async_query()` bookkeeping, such as
  `mysqli_poll()`/`MYSQLI_ASYNC`, or add a sharper unsupported diagnostic
  before claiming broader async socket-readiness or host query execution
  fidelity.
  Milestone 929 exposes `MYSQLI_ASYNC = 8` and makes `mysqli_poll()` visible
  through runtime and native metadata lookup, then rejects reached
  `mysqli_poll()` calls with a stable diagnostic naming async socket readiness
  and by-reference read/error/reject array mutation as missing. This is not
  real async query execution, `mysqli_poll()` readiness, by-reference array
  mutation, pending async result queues, host socket state, warnings/errors,
  host database state, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_poll()`/`MYSQLI_ASYNC` metadata and stable async-readiness
  boundary through a WordPress-shaped connection method without claiming real
  polling, by-reference result arrays, socket readiness, or host query
  execution fidelity.
  Milestone 930 adds a `phpc-only` synthetic `wpdb` fixture that records
  `mysqli_poll()` function/callability metadata and `MYSQLI_ASYNC = 8`, then
  reaches the stable `mysqli_poll()` async-readiness boundary through a
  connection method. It is not real async polling, by-reference
  read/error/reject array mutation, socket readiness, pending async result
  queues, warnings/errors, host database state, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi connection option or async
  execution boundary after `mysqli_poll()`, such as `mysqli_get_links_stats()`,
  `mysqli_dump_debug_info()`, or a sharper unsupported diagnostic, before
  claiming broader mysqli host-state fidelity.
  Milestone 931 implements bounded deterministic
  `mysqli_get_links_stats()` support for the no-argument call. It returns
  zeroed `total`, `active_plinks`, and `cached_plinks` metadata, rejects
  argument-bearing calls with a stable arity diagnostic, and is visible through
  runtime and native metadata lookup. This is not real persistent-link
  tracking, host client-library state, sockets, connection reuse state,
  warnings/errors, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_get_links_stats()` host-link metadata through a
  WordPress-shaped connection method without claiming real persistent-link,
  socket, host client-library, or connection reuse fidelity.
  Milestone 932 adds a `phpc-only` synthetic `wpdb` fixture that records
  deterministic zeroed `mysqli_get_links_stats()` host-link metadata through a
  WordPress-shaped bookkeeping method. It is not real persistent-link
  tracking, sockets, host client-library state, connection reuse state,
  warnings/errors, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi host diagnostics boundary
  after host-link stats, such as `mysqli_dump_debug_info()` or a sharper
  unsupported diagnostic, before claiming broader mysqli debug or host-state
  fidelity.
  Milestone 933 implements bounded deterministic `mysqli_dump_debug_info()`
  support for current placeholder handles. It returns deterministic `true`,
  rejects non-`mysqli` handles with a stable diagnostic, and is visible through
  runtime and native metadata lookup. This is not MySQL DBUG trace output,
  host client-library debug state, socket inspection, host database state,
  warning/error fidelity, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` smoke that records the
  bounded `mysqli_dump_debug_info()` host-diagnostics placeholder through a
  WordPress-shaped connection diagnostics method without claiming MySQL DBUG
  trace output, host client-library debug state, sockets, host database state,
  warning/error fidelity, or native lowering.
  Milestone 934 adds a `phpc-only` synthetic `wpdb` fixture that records
  callable metadata, deterministic `mysqli_dump_debug_info()` success, and
  placeholder connection liveness through a WordPress-shaped diagnostics
  method. It is not MySQL DBUG trace output, host client-library debug state,
  socket inspection, host database state, warning/error fidelity, or native
  lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi host-state or diagnostics
  boundary after deterministic dump-debug metadata, such as a sharper
  unsupported diagnostic for real debug trace behavior, the next missing
  WordPress-reached MySQLi function, or a real-state replacement for one of
  the current placeholders before claiming broader database fidelity.
  Milestone 935 implements bounded deterministic `mysqli_debug()` support for
  the current scalar/null string-convertible options boundary. It returns
  deterministic `true`, rejects array options with a stable diagnostic, and is
  visible through runtime and native metadata lookup. This is not MySQL DBUG
  option parsing, trace-file creation, host client-library debug state
  mutation, socket inspection, host database state, warning/error fidelity, or
  native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` diagnostics smoke that
  records bounded `mysqli_debug()` metadata through a WordPress-shaped
  connection diagnostics method without claiming MySQL DBUG option parsing,
  trace-file creation, host client-library debug state mutation, sockets, host
  database state, warning/error fidelity, or native lowering.
  Milestone 936 adds a `phpc-only` synthetic `wpdb` fixture that records
  callable metadata, deterministic `mysqli_debug()` DBUG-configuration
  success, and placeholder connection liveness through a WordPress-shaped
  diagnostics method. It is not MySQL DBUG option parsing, trace-file
  creation, host client-library debug state mutation, socket inspection, host
  database state, warning/error fidelity, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi host-state boundary after
  debug metadata, such as `mysqli_get_client_stats()` or a sharper unsupported
  diagnostic for real mysqlnd client statistics, before claiming broader
  client-library state fidelity.
  Milestone 937 implements bounded deterministic `mysqli_get_client_stats()`
  support for the no-argument call. It returns a small zeroed local subset of
  mysqlnd-style client statistics, rejects argument-bearing calls with a stable
  arity diagnostic, and is visible through runtime and native metadata lookup.
  This is not PHP's full mysqlnd client statistics table, real client-library
  traffic accounting, memory accounting, connection reuse state, sockets, host
  database state, warning/error fidelity, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` diagnostics smoke that
  records bounded `mysqli_get_client_stats()` metadata through a
  WordPress-shaped connection diagnostics method without claiming PHP's full
  mysqlnd client statistics table, real client-library accounting, sockets,
  host database state, warning/error fidelity, or native lowering.
  Milestone 938 adds a `phpc-only` synthetic `wpdb` fixture that records the
  small zeroed `mysqli_get_client_stats()` subset through a WordPress-shaped
  diagnostics method. It is not PHP's full mysqlnd client statistics table,
  real client-library accounting, memory accounting, connection reuse state,
  sockets, host database state, warning/error fidelity, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi error-list or statement
  boundary after client stats, such as `mysqli_error_list()` or a sharper
  unsupported diagnostic for real warning/error list state, before claiming
  broader database error fidelity.
  Milestone 939 implements bounded deterministic `mysqli_error_list()` support
  for current placeholder handles. It returns an empty array for clean local
  error-list state, rejects non-`mysqli` handles with a stable diagnostic, and
  is visible through runtime and native metadata lookup. This is not real
  warning/error list tracking, SQLSTATE history, host client-library state,
  socket state, host database state, warning/error fidelity, or native
  lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` diagnostics smoke that
  records bounded `mysqli_error_list()` clean metadata through a
  WordPress-shaped connection diagnostics method without claiming real
  warning/error list tracking, SQLSTATE history, host client-library state,
  sockets, host database state, warning/error fidelity, or native lowering.
  Milestone 940 adds a `phpc-only` synthetic `wpdb` fixture that records the
  empty local `mysqli_error_list()` clean metadata through a WordPress-shaped
  diagnostics method. It is not real warning/error list tracking, SQLSTATE
  history, host client-library state, sockets, host database state,
  warning/error fidelity, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi statement or error-state
  boundary after clean error-list metadata, such as `mysqli_thread_safe()` or
  an explicit unsupported diagnostic for statement APIs, before claiming
  broader mysqli extension fidelity.
  Milestone 941 implements bounded deterministic `mysqli_thread_safe()`
  support for client-library thread-safety metadata. It returns deterministic
  `true`, rejects argument-bearing calls with a stable arity diagnostic, and
  is visible through runtime and native metadata lookup. This is not host
  client-library build-flag inspection, real thread-safety configuration, host
  client-library state, socket state, host database state, warning/error
  fidelity, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` diagnostics smoke that
  records bounded `mysqli_thread_safe()` metadata through a WordPress-shaped
  connection diagnostics method without claiming host client-library
  build-flag inspection, real thread-safety configuration, host
  client-library state, sockets, host database state, warning/error fidelity,
  or native lowering.
  Milestone 942 adds a `phpc-only` synthetic `wpdb` fixture that records the
  deterministic `mysqli_thread_safe()` truthy metadata through a
  WordPress-shaped diagnostics method. It is not host client-library
  build-flag inspection, real thread-safety configuration, host
  client-library state, sockets, host database state, warning/error fidelity,
  or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi statement or connection
  metadata boundary after thread-safety metadata, such as statement lifecycle
  APIs (`mysqli_stmt_init`, `mysqli_prepare`) or a sharper unsupported
  diagnostic, before claiming broader mysqli extension fidelity.
  Milestone 943 exposes `mysqli_stmt_init()` and `mysqli_prepare()` through
  callable metadata and turns reached statement lifecycle calls into stable
  unsupported diagnostics. This is not statement object allocation, prepared
  SQL parsing, parameter/result binding, statement execution, result metadata,
  host database state, warning/error fidelity, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` prepared-statement smoke
  that reaches the explicit `mysqli_prepare()` boundary through a
  WordPress-shaped method without claiming statement objects, binding,
  execution, result metadata, host database state, warning/error fidelity, or
  native lowering.
  Milestone 944 adds a `phpc-only` synthetic `wpdb` fixture that reaches the
  explicit `mysqli_prepare()` unsupported diagnostic through a WordPress-shaped
  option lookup method. It is not statement object allocation, prepared SQL
  parsing, parameter/result binding, statement execution, result metadata,
  host database state, warning/error fidelity, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi statement API boundary
  after prepared-statement lifecycle visibility, such as
  `mysqli_stmt_bind_param()`/`mysqli_stmt_execute()` callable metadata and
  explicit unsupported diagnostics, before claiming broader prepared statement
  fidelity.
  Milestone 945 exposes `mysqli_stmt_bind_param()` and
  `mysqli_stmt_execute()` through callable metadata and turns reached binding
  or execution calls into stable unsupported diagnostics. This is not
  statement object allocation, by-reference parameter binding, type-string
  validation, prepared statement execution, result state, host database
  execution, warning/error fidelity, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` prepared-statement smoke
  that reaches the explicit bind/execute boundary through a WordPress-shaped
  method without claiming statement objects, by-reference binding, type-string
  validation, statement execution, result state, host database state,
  warning/error fidelity, or native lowering.
  Milestone 946 adds `phpc-only` synthetic `wpdb` fixtures that reach the
  explicit `mysqli_stmt_bind_param()` and `mysqli_stmt_execute()` unsupported
  diagnostics through WordPress-shaped methods. These are not statement object
  allocation, by-reference binding, type-string validation, statement
  execution, result state, host database state, warning/error fidelity, or
  native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi statement-result API
  boundary after bind/execute visibility, such as
  `mysqli_stmt_get_result()`/`mysqli_stmt_close()` callable metadata and
  explicit unsupported diagnostics, before claiming broader prepared statement
  fidelity.
  Milestone 947 exposes `mysqli_stmt_get_result()` and
  `mysqli_stmt_close()` through callable metadata and turns reached result
  materialization or statement cleanup calls into stable unsupported
  diagnostics. This is not statement object allocation, mysqlnd result
  transfer, result metadata, resource cleanup, lifecycle state, host database
  execution, warning/error fidelity, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` prepared-statement result
  or cleanup smoke that reaches the explicit `mysqli_stmt_get_result()` or
  `mysqli_stmt_close()` boundary through a WordPress-shaped method without
  claiming statement objects, result transfer, result metadata, cleanup state,
  host database state, warning/error fidelity, or native lowering.
  Milestone 948 adds `phpc-only` synthetic `wpdb` fixtures that reach the
  explicit `mysqli_stmt_get_result()` and `mysqli_stmt_close()` unsupported
  diagnostics through WordPress-shaped methods. These are not statement object
  allocation, mysqlnd result transfer, result metadata, resource cleanup,
  lifecycle state, host database state, warning/error fidelity, or native
  lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi statement metadata/error
  API boundary after result/cleanup visibility, such as
  `mysqli_stmt_errno()`/`mysqli_stmt_error()`/`mysqli_stmt_affected_rows()`
  callable metadata and explicit unsupported or clean-placeholder diagnostics,
  before claiming broader prepared statement fidelity.
  Milestone 949 exposes `mysqli_stmt_errno()`, `mysqli_stmt_error()`, and
  `mysqli_stmt_affected_rows()` through callable metadata and turns reached
  statement metadata calls into stable unsupported diagnostics. This is not
  statement object allocation, statement error-state tracking, statement
  error-message tracking, statement execution state, affected-row metadata,
  host database execution, warning/error fidelity, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` prepared-statement
  metadata/error smoke that reaches the explicit statement errno/error or
  affected-row boundary through a WordPress-shaped method without claiming
  statement objects, statement error state, affected-row metadata, host
  database state, warning/error fidelity, or native lowering.
  Milestone 950 adds `phpc-only` synthetic `wpdb` fixtures that reach the
  explicit `mysqli_stmt_errno()`, `mysqli_stmt_error()`, and
  `mysqli_stmt_affected_rows()` unsupported diagnostics through
  WordPress-shaped methods. These are not statement object allocation,
  statement error-state tracking, statement error-message tracking, statement
  execution state, affected-row metadata, host database state, warning/error
  fidelity, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi statement result/cursor
  boundary after statement metadata visibility, such as
  `mysqli_stmt_store_result()`/`mysqli_stmt_num_rows()`/
  `mysqli_stmt_fetch()` callable metadata and explicit unsupported
  diagnostics, before claiming broader prepared statement fidelity.
  Milestone 951 exposes `mysqli_stmt_store_result()`,
  `mysqli_stmt_num_rows()`, and `mysqli_stmt_fetch()` through callable
  metadata and turns reached statement result/cursor calls into stable
  unsupported diagnostics. This is not statement object allocation, buffered
  result storage, statement row-count metadata, cursor advancement, bound
  result buffers, host database rows, warning/error fidelity, or native
  lowering.
- [x] WordPress harness lane: add synthetic `wpdb` prepared-statement
  result/cursor smokes that reach the explicit `mysqli_stmt_store_result()`,
  `mysqli_stmt_num_rows()`, or `mysqli_stmt_fetch()` boundary through
  WordPress-shaped methods without claiming statement objects, buffered result
  storage, cursor/fetch state, bound result variables, result-row
  materialization, statement row-count metadata, host database state,
  warning/error fidelity, or native lowering.
  Milestone 952 adds `phpc-only` synthetic `wpdb` fixtures that reach the
  explicit `mysqli_stmt_store_result()`, `mysqli_stmt_num_rows()`, and
  `mysqli_stmt_fetch()` unsupported diagnostics through WordPress-shaped
  methods. These are not statement object allocation, buffered result storage,
  cursor/fetch state, bound result variables, result-row materialization,
  statement row-count metadata, host database state, warning/error fidelity,
  or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi prepared-statement
  output-binding boundary after result/cursor visibility, such as
  `mysqli_stmt_bind_result()` callable metadata and explicit unsupported
  diagnostics, before claiming broader prepared statement fidelity.
  Milestone 953 exposes `mysqli_stmt_bind_result()` through callable metadata
  and turns reached result-output binding calls into stable unsupported
  diagnostics. This is not statement object allocation, by-reference result
  binding, result buffer mutation, fetch integration, host database execution,
  warning/error fidelity, or native lowering.
- [x] WordPress harness lane: add a synthetic `wpdb` prepared-statement
  result-binding smoke that reaches the explicit
  `mysqli_stmt_bind_result()` boundary through a WordPress-shaped method
  without claiming statement objects, by-reference result binding, result
  buffer mutation, fetch integration, host database state, warning/error
  fidelity, or native lowering.
  Milestone 954 adds a `phpc-only` synthetic `wpdb` fixture that reaches the
  explicit `mysqli_stmt_bind_result()` unsupported diagnostic through a
  WordPress-shaped method. This is not statement object allocation,
  by-reference result binding, result buffer mutation, fetch integration, host
  database state, warning/error fidelity, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi prepared-statement result
  metadata/cleanup boundary after result-binding visibility, such as
  `mysqli_stmt_result_metadata()`/`mysqli_stmt_field_count()`/
  `mysqli_stmt_free_result()` callable metadata and explicit unsupported
  diagnostics, before claiming broader prepared statement fidelity.
  Milestone 955 exposes `mysqli_stmt_result_metadata()`,
  `mysqli_stmt_field_count()`, and `mysqli_stmt_free_result()` through
  callable metadata and turns reached result metadata/cleanup calls into
  stable unsupported diagnostics. This is not statement object allocation,
  statement result metadata objects, field metadata transfer, field-count
  state, result buffers, statement result cleanup state, host database
  execution, warning/error fidelity, or native lowering.
- [x] WordPress harness lane: add synthetic `wpdb` prepared-statement result
  metadata/cleanup smokes that reach the explicit
  `mysqli_stmt_result_metadata()`, `mysqli_stmt_field_count()`, or
  `mysqli_stmt_free_result()` boundary through WordPress-shaped methods
  without claiming statement objects, statement result metadata, field
  metadata transfer, field-count state, result buffers, result cleanup state,
  host database state, warning/error fidelity, or native lowering.
  Milestone 956 adds `phpc-only` synthetic `wpdb` fixtures that reach the
  explicit `mysqli_stmt_result_metadata()`, `mysqli_stmt_field_count()`, and
  `mysqli_stmt_free_result()` unsupported diagnostics through WordPress-shaped
  methods. These are not statement object allocation, statement result
  metadata objects, field metadata transfer, field-count state, result
  buffers, statement result cleanup state, host database state, warning/error
  fidelity, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi prepared-statement
  positioning/attribute boundary after result metadata visibility, such as
  `mysqli_stmt_data_seek()`/`mysqli_stmt_attr_get()`/
  `mysqli_stmt_attr_set()` callable metadata and explicit unsupported
  diagnostics, before claiming broader prepared statement fidelity.
  Milestone 957 exposes `mysqli_stmt_data_seek()`,
  `mysqli_stmt_attr_get()`, and `mysqli_stmt_attr_set()` through callable
  metadata and turns reached statement positioning/attribute calls into
  stable unsupported diagnostics. This is not statement object allocation,
  buffered result cursor state, offset seeking, statement attribute catalogs,
  option registry state, option mutation, host database execution,
  warning/error fidelity, or native lowering.
- [x] WordPress harness lane: add synthetic `wpdb` prepared-statement
  positioning/attribute smokes that reach the explicit
  `mysqli_stmt_data_seek()`, `mysqli_stmt_attr_get()`, or
  `mysqli_stmt_attr_set()` boundary through WordPress-shaped methods without
  claiming statement objects, buffered result cursor state, offset seeking,
  statement attribute catalogs, option registry state, option mutation, host
  database state, warning/error fidelity, or native lowering.
  Milestone 958 adds `phpc-only` synthetic `wpdb` fixtures that reach the
  explicit `mysqli_stmt_data_seek()`, `mysqli_stmt_attr_get()`, and
  `mysqli_stmt_attr_set()` unsupported diagnostics through WordPress-shaped
  methods. These are not statement object allocation, buffered result cursor
  state, offset seeking, statement attribute catalogs, option registry state,
  option mutation, host database state, warning/error fidelity, or native
  lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi prepared-statement
  parameter streaming/reset/multi-result boundary, such as
  `mysqli_stmt_send_long_data()`/`mysqli_stmt_reset()`/
  `mysqli_stmt_more_results()`/`mysqli_stmt_next_result()` callable metadata
  and explicit unsupported diagnostics, before claiming broader prepared
  statement fidelity.
  Milestone 959 exposes `mysqli_stmt_send_long_data()`,
  `mysqli_stmt_reset()`, `mysqli_stmt_more_results()`, and
  `mysqli_stmt_next_result()` through callable metadata and turns reached
  parameter streaming, reset, and multi-result calls into stable unsupported
  diagnostics. This is not statement object allocation, long-parameter
  streaming, packet buffering, statement parameter state, statement state
  reset, buffered result cleanup, parameter/result lifecycle state,
  multi-result state, pending result queues, host database execution,
  warning/error fidelity, or native lowering.
- [x] WordPress harness lane: add synthetic `wpdb` prepared-statement
  parameter streaming/reset/multi-result smokes that reach the explicit
  `mysqli_stmt_send_long_data()`, `mysqli_stmt_reset()`,
  `mysqli_stmt_more_results()`, or `mysqli_stmt_next_result()` boundary
  through WordPress-shaped methods without claiming statement objects,
  long-parameter streaming, packet buffering, statement parameter state,
  statement state reset, buffered result cleanup, parameter/result lifecycle
  state, multi-result state, pending result queues, host database state,
  warning/error fidelity, or native lowering.
  Milestone 960 adds `phpc-only` synthetic `wpdb` fixtures that reach the
  explicit `mysqli_stmt_send_long_data()`, `mysqli_stmt_reset()`,
  `mysqli_stmt_more_results()`, and `mysqli_stmt_next_result()` unsupported
  diagnostics through WordPress-shaped methods. These are not statement object
  allocation, long-parameter streaming, packet buffering, statement parameter
  state, statement state reset, buffered result cleanup, parameter/result
  lifecycle state, multi-result state, pending result queues, host database
  state, warning/error fidelity, or native lowering.
- [x] Runtime/mysqli lane: inspect the next MySQLi prepared-statement
  diagnostics/insert metadata boundary, such as `mysqli_stmt_sqlstate()`,
  `mysqli_stmt_warning_count()`, or `mysqli_stmt_insert_id()` callable
  metadata and explicit unsupported diagnostics, before claiming broader
  prepared statement fidelity.
  Milestone 961 exposes `mysqli_stmt_sqlstate()`,
  `mysqli_stmt_warning_count()`, and `mysqli_stmt_insert_id()` through
  callable metadata and turns reached statement diagnostics/insert metadata
  calls into stable unsupported diagnostics. This is not statement object
  allocation, statement SQLSTATE tracking, statement warning tracking,
  statement diagnostic state, statement execution state, statement insert-id
  metadata, host database execution, warning/error fidelity, or native
  lowering.
- [x] WordPress harness lane: add synthetic `wpdb` prepared-statement
  diagnostics/insert metadata smokes that reach the explicit
  `mysqli_stmt_sqlstate()`, `mysqli_stmt_warning_count()`, or
  `mysqli_stmt_insert_id()` boundary through WordPress-shaped methods without
  claiming statement objects, statement SQLSTATE tracking, statement warning
  tracking, statement diagnostic state, statement execution state, statement
  insert-id metadata, host database state, warning/error fidelity, or native
  lowering.
  Milestone 962 adds `phpc-only` synthetic `wpdb` fixtures that reach the
  explicit `mysqli_stmt_sqlstate()`, `mysqli_stmt_warning_count()`, and
  `mysqli_stmt_insert_id()` unsupported diagnostics through WordPress-shaped
  methods. These are not statement object allocation, statement SQLSTATE
  tracking, statement warning tracking, statement diagnostic state, statement
  execution state, statement insert-id metadata, host database state,
  warning/error fidelity, or native lowering.
- [x] Runtime/mysqli lane: correct the misidentified statement field metadata
  helper surface after auditing local PHP. Milestone 967 removes the non-PHP
  `mysqli_stmt_fetch_fields()`/`mysqli_stmt_fetch_field()` names from callable
  metadata and implements the real placeholder-result helpers
  `mysqli_fetch_fields()`, `mysqli_fetch_field_direct()`,
  `mysqli_field_seek()`, and `mysqli_field_tell()` for the current seed-post
  result subset.
- [x] Runtime/mysqli lane: inspect the next MySQLi prepared-statement
  lifecycle/diagnostic metadata boundary, such as `mysqli_stmt_prepare()`,
  `mysqli_stmt_param_count()`, `mysqli_stmt_get_warnings()`, or
  `mysqli_stmt_error_list()` callable metadata and explicit unsupported
  diagnostics, before claiming broader prepared statement fidelity.
  Milestone 965 exposes `mysqli_stmt_prepare()`,
  `mysqli_stmt_param_count()`, `mysqli_stmt_get_warnings()`, and
  `mysqli_stmt_error_list()` through callable metadata and turns reached
  statement prepare/parameter-count and diagnostic-list calls into stable
  unsupported diagnostics. This is not statement object allocation, prepared
  SQL parsing, parameter metadata, warning-chain objects, error-list arrays,
  statement diagnostic state, host database execution, warning/error fidelity,
  or native lowering.
- [x] WordPress harness lane: add synthetic `wpdb` prepared-statement
  prepare/parameter-count and diagnostic-list smokes that reach the explicit
  `mysqli_stmt_prepare()`, `mysqli_stmt_param_count()`,
  `mysqli_stmt_get_warnings()`, or `mysqli_stmt_error_list()` boundary through
  WordPress-shaped methods without claiming statement objects, prepared SQL
  parsing, parameter metadata, warning-chain objects, error-list arrays, host
  database state, warning/error fidelity, or native lowering.
  Milestone 966 adds `phpc-only` synthetic `wpdb` fixtures that reach the
  explicit `mysqli_stmt_prepare()`, `mysqli_stmt_param_count()`,
  `mysqli_stmt_get_warnings()`, and `mysqli_stmt_error_list()` unsupported
  diagnostics through WordPress-shaped methods. These are not statement object
  allocation, prepared SQL parsing, parameter metadata, warning-chain objects,
  error-list arrays, host database state, warning/error fidelity, or native
  lowering.
- [x] Runtime/database lane: audit the remaining database API surface against
  current WordPress blockers after the MySQLi prepared-statement procedural
  boundaries, then add the next explicit runtime boundary or small deterministic
  behavior slice with parser/runtime tests, CLI fixtures, docs, and native
  rejection coverage where lowering remains unsupported.
  Milestone 967 corrects the misidentified statement field metadata surface:
  local PHP does not expose `mysqli_stmt_fetch_fields()` or
  `mysqli_stmt_fetch_field()`, so those names are no longer advertised, and
  the real placeholder-result helpers `mysqli_fetch_fields()`,
  `mysqli_fetch_field_direct()`, `mysqli_field_seek()`, and
  `mysqli_field_tell()` now execute for the current seed-post result subset.
- [x] Runtime/database lane: inspect the next real MySQLi result/connection
  helper gap from the audited PHP surface, such as `mysqli_fetch_lengths()`,
  `mysqli_fetch_all()`, `mysqli_fetch_column()`,
  `mysqli_fetch_field_direct()` metadata breadth, or savepoint/SSL/alias
  helpers, and add the next bounded behavior or explicit runtime boundary with
  tests, CLI fixtures, docs, and native rejection coverage where lowering
  remains unsupported.
  Milestone 968 implements `mysqli_fetch_lengths()` for the current
  deterministic seed-post result: it returns `false` before any row fetch and
  returns a zero-indexed integer array for the most recently fetched row after
  `mysqli_fetch_object()`, `mysqli_fetch_assoc()`, `mysqli_fetch_row()`, or
  `mysqli_fetch_array()`. This is not real host result metadata,
  binary/protocol length accounting, full result resources, warning/error
  fidelity, or native lowering.
- [x] Runtime/database lane: inspect the next real MySQLi result/connection
  helper gap from the audited PHP surface, such as `mysqli_fetch_all()`,
  `mysqli_fetch_column()`, savepoint/SSL/alias helpers, or broader result
  metadata fields, and add the next bounded behavior or explicit runtime
  boundary with tests, CLI fixtures, docs, and native rejection coverage where
  lowering remains unsupported.
  Milestone 969 implements `mysqli_fetch_all()` and `mysqli_fetch_column()`
  for the current deterministic seed-post result. `mysqli_fetch_all()` drains
  remaining placeholder rows with the supported `MYSQLI_NUM`, `MYSQLI_ASSOC`,
  and `MYSQLI_BOTH` shapes, while `mysqli_fetch_column()` consumes one row and
  returns a selected integer column, `null` for a missing column, or `false`
  when no row remains. This is not real SQL execution, host result storage,
  broad result resources, duplicate-column fidelity, warning/error fidelity,
  unbuffered result behavior, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi connection/helper
  gap from the audited PHP surface, such as `mysqli_savepoint()`,
  `mysqli_release_savepoint()`, `mysqli_ssl_set()`, `mysqli_set_opt()`/
  `mysqli_options()` alias behavior, or broader result metadata fields, and
  add the next bounded behavior or explicit runtime boundary with tests, CLI
  fixtures, docs, and native rejection coverage where lowering remains
  unsupported.
  Milestone 970 implements deterministic `mysqli_savepoint()` and
  `mysqli_release_savepoint()` placeholder helpers for the current `mysqli`
  object and string savepoint names. This is not real host transaction state,
  savepoint creation/release/validation, rollback-to-savepoint behavior,
  warning/error fidelity, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi connection/helper
  gap from the audited PHP surface, such as `mysqli_ssl_set()`,
  `mysqli_set_opt()`/`mysqli_options()` alias behavior,
  `mysqli_escape_string()` alias behavior, or broader result metadata fields,
  and add the next bounded behavior or explicit runtime boundary with tests,
  CLI fixtures, docs, and native rejection coverage where lowering remains
  unsupported.
  Milestone 971 implements `mysqli_set_opt()` as the current
  `MYSQLI_OPT_INT_AND_FLOAT_NATIVE` bool/int alias of `mysqli_options()`, and
  `mysqli_escape_string()` as the deterministic scalar/null escaping alias of
  `mysqli_real_escape_string()`. This is not real client option negotiation,
  result type conversion changes, connection charset-sensitive escaping,
  binary/invalid string fidelity, host database state, warning/error fidelity,
  or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi connection/helper
  gap from the audited PHP surface, such as `mysqli_ssl_set()`, broader
  `mysqli_options()` option catalogs, broader escaping charset fidelity, or
  result metadata fields, and add the next bounded behavior or explicit
  runtime boundary with tests, CLI fixtures, docs, and native rejection
  coverage where lowering remains unsupported.
  Milestone 972 implements deterministic `mysqli_ssl_set()` placeholder
  behavior for the current `mysqli` object and string/null SSL option
  arguments. This is not TLS configuration, file validation, SSL negotiation
  during `mysqli_real_connect()`, connection state mutation, host
  client-library inspection, warning/error fidelity, or native database
  lowering.
- [x] Runtime/database lane: inspect the next real MySQLi connection/helper
  gap from the audited PHP surface, such as broader `mysqli_options()` option
  catalogs, broader escaping charset fidelity, `mysqli_real_connect()` SSL
  interaction, or result metadata fields, and add the next bounded behavior or
  explicit runtime boundary with tests, CLI fixtures, docs, and native
  rejection coverage where lowering remains unsupported.
  Milestone 973 implements a broader deterministic `mysqli_options()`/
  `mysqli_set_opt()` option catalog for timeout/network integer options, init
  command and local-data-dir string options, and local-infile/SSL/expired
  password bool-or-int options. This is not real option storage,
  timeout/network behavior, local-infile behavior, init command execution,
  path validation, result type conversion changes, connection state mutation,
  warning/error fidelity, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi connection/helper
  gap from the audited PHP surface, such as broader escaping charset fidelity,
  `mysqli_real_connect()` SSL/option interaction, local-infile option effects,
  or result metadata fields, and add the next bounded behavior or explicit
  runtime boundary with tests, CLI fixtures, docs, and native rejection
  coverage where lowering remains unsupported.
  Milestone 974 exposes deterministic `MYSQLI_CLIENT_*` client-flag constants
  and bounds `mysqli_real_connect()` flags to combinations of those constants,
  including reached SSL/options setup paths. This is not real client capability
  negotiation, TLS negotiation, SSL certificate verification, option storage,
  host connection state, warning/error fidelity, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi connection/helper
  gap from the audited PHP surface, such as broader escaping charset fidelity,
  local-infile option effects, result metadata fields, or prepared-statement
  execution boundaries, and add the next bounded behavior or explicit runtime
  boundary with tests, CLI fixtures, docs, and native rejection coverage where
  lowering remains unsupported.
  Milestone 975 broadens deterministic `mysqli_result` field metadata objects
  for the current seed-post `ID` and `post_title` placeholder fields. This is
  not real host field metadata, SQL-derived table or database metadata,
  protocol flag/type fidelity, collation negotiation, duplicate-column
  behavior, warning/error fidelity, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi connection/helper
  gap from the audited PHP surface, such as broader escaping charset fidelity,
  local-infile option effects, prepared-statement execution boundaries, or
  pending result state for `mysqli_store_result()`/`mysqli_use_result()`, and
  add the next bounded behavior or explicit runtime boundary with tests, CLI
  fixtures, docs, and native rejection coverage where lowering remains
  unsupported.
  Milestone 976 adds deterministic `mysqli_real_query()` pending-result state
  for the current seed-post and empty-result SQL placeholders, consumable
  through `mysqli_store_result()`/`mysqli_use_result()`. This is not general
  SQL execution, real buffered or unbuffered result transfer, host connection
  pending-result queues, multi-result state, mutation state,
  warning/error fidelity, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi connection/helper
  gap from the audited PHP surface, such as broader escaping charset fidelity,
  local-infile option effects, prepared-statement object lifecycle, or
  multi-result pending queues, and add the next bounded behavior or explicit
  runtime boundary with tests, CLI fixtures, docs, and native rejection
  coverage where lowering remains unsupported.
  Milestone 977 adds deterministic `mysqli_stmt` placeholder object lifecycle
  for `mysqli_stmt_init()`, `mysqli_prepare()`, `mysqli_stmt_prepare()`,
  `mysqli_stmt_param_count()`, `mysqli_stmt_reset()`, and
  `mysqli_stmt_close()`. This is not prepared SQL parsing, real parameter
  metadata, by-reference binding, execution, result metadata transfer,
  statement diagnostics, host database state, warning/error fidelity, or
  native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi connection/helper
  gap from the audited PHP surface, such as broader escaping charset fidelity,
  local-infile option effects, statement binding/execution boundaries, or
  multi-result pending queues, and add the next bounded behavior or explicit
  runtime boundary with tests, CLI fixtures, docs, and native rejection
  coverage where lowering remains unsupported.
  Milestone 978 adds deterministic clean diagnostic metadata for active
  placeholder `mysqli_stmt` objects: clean errno/error, SQLSTATE, warning
  count, warning-chain, error-list, affected-row, and insert-id reads now
  execute directly and through a WordPress-shaped `wpdb` smoke. This is not
  failed-prepare tracking, statement execution diagnostics, warning-chain
  objects, real error-list entries, affected-row metadata, insert-id metadata,
  host database state, PHP warning/error fidelity, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as statement
  binding/execution boundaries with active placeholder statements, broader
  escaping charset fidelity, local-infile option effects, result buffering, or
  multi-result pending queues, and add the next bounded behavior or explicit
  runtime boundary with tests, CLI fixtures, docs, and native rejection
  coverage where lowering remains unsupported.
  Milestone 979 adds deterministic `mysqli_stmt_result_metadata()`,
  `mysqli_stmt_field_count()`, and `mysqli_stmt_free_result()` behavior for
  active placeholder statements over the current seed-post WordPress SELECT
  field metadata shape, including a WordPress-shaped `wpdb` smoke. This is not
  prepared binding, statement execution, statement result rows, mysqlnd result
  transfer, broad SQL metadata, host database metadata, PHP warning/error
  fidelity, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as active-statement
  binding/execution boundaries, statement result buffering/fetching,
  broader escaping charset fidelity, local-infile option effects, or
  multi-result pending queues, and add the next bounded behavior or explicit
  runtime boundary with tests, CLI fixtures, docs, and native rejection
  coverage where lowering remains unsupported.
  Milestone 980 adds deterministic unbound `mysqli_stmt_execute()` plus
  `mysqli_stmt_get_result()` behavior for active placeholder statements over
  the current seed-post WordPress SELECT shape, including a WordPress-shaped
  `wpdb` smoke. This is not bound-parameter execution, array parameter
  execution, mutation execution, unknown SELECT metadata, real mysqlnd
  transfer, host database state, PHP warning/error fidelity, or native
  database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as statement result
  buffering/fetching, active-statement binding boundaries, broader escaping
  charset fidelity, local-infile option effects, or multi-result pending
  queues, and add the next bounded behavior or explicit runtime boundary with
  tests, CLI fixtures, docs, and native rejection coverage where lowering
  remains unsupported.
  Milestone 981 adds deterministic `mysqli_stmt_store_result()` and
  `mysqli_stmt_num_rows()` behavior for active placeholder statements after
  the current unbound seed-post WordPress SELECT execution path, including a
  WordPress-shaped `wpdb` smoke. This is not by-reference result binding,
  output-buffer mutation, `mysqli_stmt_fetch()`, real mysqlnd buffering, broad
  SQL execution, host database rows, PHP warning/error fidelity, or native
  database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as
  active-statement binding boundaries, statement fetch boundaries, broader
  escaping charset fidelity, local-infile option effects, or multi-result
  pending queues, and add the next bounded behavior or explicit runtime
  boundary with tests, CLI fixtures, docs, and native rejection coverage where
  lowering remains unsupported.
  Milestone 982 adds deterministic clean statement multi-result state:
  `mysqli_stmt_more_results()` and `mysqli_stmt_next_result()` now return
  `false` for active placeholder statements, including a WordPress-shaped
  `wpdb` smoke. This is not multi-statement execution, pending statement
  result queues, cursor advancement, host database state, PHP warning/error
  fidelity, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as
  active-statement attribute state, active-statement binding boundaries,
  statement fetch boundaries, broader escaping charset fidelity, local-infile
  option effects, or connection multi-result pending queues, and add the next
  bounded behavior or explicit runtime boundary with tests, CLI fixtures,
  docs, and native rejection coverage where lowering remains unsupported.
  Milestone 983 adds deterministic `mysqli_stmt_attr_get()` and
  `mysqli_stmt_attr_set()` placeholder state for active statements, including
  PHP-matching statement attribute and cursor-type constants plus a
  WordPress-shaped `wpdb` smoke. This is not real mysqlnd cursor behavior,
  prefetch behavior, max-length metadata recalculation, host database state,
  PHP warning/error fidelity, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as
  active-statement cursor positioning, active-statement binding boundaries,
  statement fetch boundaries, broader escaping charset fidelity, local-infile
  option effects, or connection multi-result pending queues, and add the next
  bounded behavior or explicit runtime boundary with tests, CLI fixtures,
  docs, and native rejection coverage where lowering remains unsupported.
  Milestone 984 adds bounded `mysqli_stmt_data_seek()` placeholder cursor
  state for active buffered statement results, including a WordPress-shaped
  `wpdb` smoke. This is not `mysqli_stmt_fetch()`, bound-result fetching,
  by-reference output-buffer mutation, real mysqlnd cursor behavior, host
  database rows, PHP warning/error fidelity, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as
  direct-variable result binding/fetching, active-statement parameter
  binding, broader escaping charset fidelity, local-infile option effects, or
  connection multi-result pending queues, and add the next bounded behavior
  or explicit runtime boundary with tests, CLI fixtures, docs, and native
  rejection coverage where lowering remains unsupported.
  Milestone 985 adds deterministic direct-variable
  `mysqli_stmt_bind_result()` plus buffered `mysqli_stmt_fetch()` placeholder
  row copying for the current seed-post statement result, including a
  WordPress-shaped `wpdb` smoke. This is not true by-reference aliasing,
  unbuffered statement fetching, bound-parameter execution, real mysqlnd
  cursor behavior, host database state, PHP warning/error fidelity, or native
  database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as
  active-statement parameter binding/execution, broader escaping charset
  fidelity, local-infile option effects, or connection multi-result pending
  queues, and add the next bounded behavior or explicit runtime boundary with
  tests, CLI fixtures, docs, and native rejection coverage where lowering
  remains unsupported.
  Milestone 986 adds deterministic direct-variable
  `mysqli_stmt_bind_param()` plus bound placeholder `mysqli_stmt_execute()`
  support for exact known statement SQL shapes, including a WordPress-shaped
  `wpdb` smoke. This is not true by-reference aliasing, later variable
  mutation, array parameter execution, mutation SQL, broad SQL execution,
  host database state, PHP warning/error fidelity, or native database
  lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as
  later-mutation behavior for bound parameters, broader escaping charset
  fidelity, local-infile option effects, connection multi-result pending
  queues, or the next real database integration gap, and add the next bounded
  behavior or explicit runtime boundary with tests, CLI fixtures, docs, and
  native rejection coverage where lowering remains unsupported.
  Milestone 987 adds bounded execute-time refresh for direct-variable
  `mysqli_stmt_bind_param()` values on direct `mysqli_stmt_execute()` calls,
  including a WordPress-shaped `wpdb` smoke. This is not true by-reference
  aliasing, cross-scope reference cells, array parameter execution, mutation
  SQL, broad SQL execution, host database state, PHP warning/error fidelity,
  call-user-func refresh behavior, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as callback
  dispatch for bound statement execution, real reference aliasing around bound
  parameters/results, broader escaping charset fidelity, local-infile option
  effects, connection multi-result pending queues, or the next real database
  integration gap, and add the next bounded behavior or explicit runtime
  boundary with tests, CLI fixtures, docs, and native rejection coverage where
  lowering remains unsupported.
  Milestone 988 adds bounded callback-dispatched `mysqli_stmt_execute()`
  refresh through `call_user_func()` and positional `call_user_func_array()`,
  including a WordPress-shaped `wpdb` smoke. This is not true by-reference
  aliasing, cross-scope reference cells, named-argument callback dispatch,
  array parameter execution, mutation SQL, broad SQL execution, host database
  state, PHP warning/error fidelity, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as positional
  params-array execution, real reference aliasing around bound
  parameters/results, broader escaping charset fidelity, local-infile option
  effects, connection multi-result pending queues, or the next real database
  integration gap, and add the next bounded behavior or explicit runtime
  boundary with tests, CLI fixtures, docs, and native rejection coverage where
  lowering remains unsupported.
  Milestone 989 adds bounded positional params-array support for
  `mysqli_stmt_execute($stmt, array(...))`, including a WordPress-shaped
  `wpdb` smoke. This is not named params arrays, true by-reference aliasing,
  mutation SQL, broad SQL execution, host database state, PHP warning/error
  fidelity, mysqlnd behavior, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as statement
  long-data chunk state, real reference aliasing around bound
  parameters/results, named params-array behavior, broader escaping charset
  fidelity, local-infile option effects, connection multi-result pending
  queues, or the next real database integration gap, and add the next bounded
  behavior or explicit runtime boundary with tests, CLI fixtures, docs, and
  native rejection coverage where lowering remains unsupported.
  Milestone 990 adds deterministic `mysqli_stmt_send_long_data()`
  placeholder chunk state for active statements, including a WordPress-shaped
  `wpdb` smoke. This is not real blob binding, packet buffering, send timing,
  execution integration, host database state, PHP warning/error fidelity,
  mysqlnd behavior, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as `b` marker
  long-data execution integration, real reference aliasing around bound
  parameters/results, named params-array behavior, broader escaping charset
  fidelity, local-infile option effects, connection multi-result pending
  queues, or the next real database integration gap, and add the next bounded
  behavior or explicit runtime boundary with tests, CLI fixtures, docs, and
  native rejection coverage where lowering remains unsupported.
  Milestone 991 adds bounded `b` type-marker support for
  `mysqli_stmt_bind_param()` with deterministic long-data execution
  integration, including a WordPress-shaped `wpdb` smoke. This is not real
  blob binding, packet buffering, send timing, mutation SQL, broad SQL
  execution, host database state, PHP warning/error fidelity, mysqlnd
  behavior, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as real reference
  aliasing around bound parameters/results, named params-array behavior,
  broader escaping charset fidelity, local-infile option effects, connection
  multi-result pending queues, or the next real database integration gap, and
  add the next bounded behavior or explicit runtime boundary with tests, CLI
  fixtures, docs, and native rejection coverage where lowering remains
  unsupported.
  Milestone 992 adds deterministic single-statement `mysqli_multi_query()`
  pending result placeholders for exact known seed-post and empty-result SQL
  shapes, including a WordPress-shaped `wpdb` smoke. This is not true
  multi-statement execution, connection result queues,
  `mysqli_more_results()`/`mysqli_next_result()` advancement, mutation SQL,
  broad SQL execution, host database state, PHP warning/error fidelity,
  mysqlnd behavior, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as real reference
  aliasing around bound parameters/results, named params-array behavior,
  broader escaping charset fidelity, local-infile option effects, true
  connection multi-result queues, mutation SQL, or the next real database
  integration gap, and add the next bounded behavior or explicit runtime
  boundary with tests, CLI fixtures, docs, and native rejection coverage where
  lowering remains unsupported.
  Milestone 993 adds bounded deterministic multi-result queue state for
  `mysqli_multi_query()` when every semicolon-separated statement is an exact
  known result placeholder, including a WordPress-shaped `wpdb` smoke. This is
  not true SQL execution, mixed no-result/result statement handling, broad
  multi-statement parsing, mutation SQL, host database state, PHP
  warning/error fidelity, mysqlnd behavior, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as mixed
  no-result/result multi-statement handling, real reference aliasing around
  bound parameters/results, named params-array behavior, broader escaping
  charset fidelity, local-infile option effects, mutation SQL, or the next
  real database integration gap, and add the next bounded behavior or explicit
  runtime boundary with tests, CLI fixtures, docs, and native rejection
  coverage where lowering remains unsupported.
  Milestone 994 adds bounded mixed no-result/result queue slots for known
  charset setup statements before or after exact known result placeholders,
  including a WordPress-shaped `wpdb` smoke. This is not true SQL execution,
  broad multi-statement parsing, arbitrary no-result statements, mutation SQL,
  host database state, PHP warning/error fidelity, mysqlnd behavior, or native
  database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as arbitrary
  no-result statement handling, real reference aliasing around bound
  parameters/results, named params-array behavior, broader escaping charset
  fidelity, local-infile option effects, mutation SQL, or the next real
  database integration gap, and add the next bounded behavior or explicit
  runtime boundary with tests, CLI fixtures, docs, and native rejection
  coverage where lowering remains unsupported.
  Milestone 995 adds deterministic SQL-mode no-result slots for
  `mysqli_real_query()` and `mysqli_multi_query()` around the exact WordPress
  `SELECT @@SESSION.sql_mode` probe, including a WordPress-shaped `wpdb`
  smoke. This is not arbitrary no-result SQL, true SQL execution, broad
  multi-statement parsing, mutation SQL, host database state, PHP
  warning/error fidelity, mysqlnd behavior, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as additional
  exact no-result statements, real reference aliasing around bound
  parameters/results, named params-array behavior, broader escaping charset
  fidelity, local-infile option effects, mutation SQL, or the next real
  database integration gap, and add the next bounded behavior or explicit
  runtime boundary with tests, CLI fixtures, docs, and native rejection
  coverage where lowering remains unsupported.
  Milestone 996 adds placeholder MySQLi option storage with a bounded
  `MYSQLI_OPT_LOCAL_INFILE` effect on `LOAD DATA LOCAL INFILE` boundaries,
  including a WordPress-shaped `wpdb` smoke. This is not real client option
  negotiation, local file loading, `LOAD DATA` execution, mutation state, path
  validation, host database state, PHP warning/error fidelity, mysqlnd
  behavior, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as additional exact
  no-result statements, real reference aliasing around bound
  parameters/results, named params-array behavior, broader escaping charset
  fidelity, init-command option effects, mutation SQL, or the next real
  database integration gap, and add the next bounded behavior or explicit
  runtime boundary with tests, CLI fixtures, docs, and native rejection
  coverage where lowering remains unsupported.
  Milestone 997 adds bounded `MYSQLI_INIT_COMMAND` handling at
  `mysqli_real_connect()` for exact deterministic no-result init commands,
  including a WordPress-shaped `wpdb` smoke. This is not real client option
  negotiation, server-side init-command execution, broad SQL execution,
  mutation state, connection charset mutation, host database state, PHP
  warning/error fidelity, mysqlnd behavior, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as additional exact
  no-result statements, real reference aliasing around bound
  parameters/results, named params-array behavior, broader escaping charset
  fidelity, mutation SQL, or the next real database integration gap, and add
  the next bounded behavior or explicit runtime boundary with tests, CLI
  fixtures, docs, and native rejection coverage where lowering remains
  unsupported.
  Milestone 998 adds bounded SQL-mode assignment no-result placeholders for
  the WordPress `SET SESSION sql_mode='...'` shape. `mysqli_query()` returns
  deterministic `true` for a strict quoted mode-list subset, and
  `mysqli_real_query()`/`mysqli_multi_query()` expose the same shape as a
  no-result slot that can participate in deterministic queues, including a
  WordPress-shaped `wpdb` smoke. This is not arbitrary `SET` execution, real
  SQL-mode mutation, server session state, broad SQL parsing, mutation SQL
  support, host database state, PHP warning/error fidelity, mysqlnd behavior,
  or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as real reference
  aliasing around bound parameters/results, named params-array behavior,
  broader escaping charset fidelity, mutation SQL state, transaction state,
  host-backed query execution, or the next real database integration gap, and
  add the next bounded behavior or explicit runtime boundary with tests, CLI
  fixtures, docs, and native rejection coverage where lowering remains
  unsupported.
  Milestone 999 tightens `mysqli_stmt_execute($stmt, $params)` params-array
  validation to require a PHP list array in the current subset. Named/string
  keyed arrays and sparse integer-keyed arrays now fail with a stable
  unsupported diagnostic instead of being silently treated as positional
  values, including direct MySQLi and WordPress-shaped `wpdb` boundary
  fixtures. This is not named params-array support, broad mysqlnd parameter
  binding, true by-reference aliasing, mutation SQL, host database state, PHP
  warning/error fidelity, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as real reference
  aliasing around bound parameters/results, broader escaping charset fidelity,
  mutation SQL state, transaction state, host-backed query execution, PDO
  visibility, or the next real database integration gap, and add the next
  bounded behavior or explicit runtime boundary with tests, CLI fixtures,
  docs, and native rejection coverage where lowering remains unsupported.
  Milestone 1000 adds bounded `mysqli_execute_query()` support for exact
  known placeholder SQL shapes. The runtime exposes the PHP 8.2+
  prepare-bind-execute convenience API through function/callability metadata,
  accepts optional PHP-list scalar/null params arrays, returns deterministic
  `mysqli_result` placeholders for exact known SELECT shapes, and returns
  `true` for current deterministic no-result shapes, including a
  WordPress-shaped `wpdb` smoke. This is not broad prepared SQL execution,
  named params-array support, hidden statement status-copy fidelity, mutation
  SQL, host database state, PHP warning/error fidelity, mysqlnd behavior, or
  native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as real reference
  aliasing around bound parameters/results, broader escaping charset fidelity,
  mutation SQL state, transaction state, host-backed query execution, PDO
  visibility, or the next real database integration gap, and add the next
  bounded behavior or explicit runtime boundary with tests, CLI fixtures,
  docs, and native rejection coverage where lowering remains unsupported.
  Milestone 1001 adds bounded `mysqli_execute()` procedural alias support over
  the current `mysqli_stmt_execute()` placeholder subset. The runtime exposes
  the alias through direct calls, function/callability metadata, dynamic string
  calls, and callback-dispatched refresh paths, including direct MySQLi and
  WordPress-shaped `wpdb` smokes. This is not broader statement execution,
  named params-array support, true by-reference aliasing, mutation SQL, host
  database state, PHP warning/error fidelity, mysqlnd behavior, or native
  database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as procedural
  connection helpers, real reference aliasing around bound parameters/results,
  broader escaping charset fidelity, mutation SQL state, transaction state,
  host-backed query execution, PDO visibility, or the next real database
  integration gap, and add the next bounded behavior or explicit runtime
  boundary with tests, CLI fixtures, docs, and native rejection coverage where
  lowering remains unsupported.
  Milestone 1002 adds bounded `mysqli_connect()` placeholder handle
  construction for direct and dynamic string-valued calls. It accepts zero to
  six current connection arguments, validates the current scalar/null argument
  subset, and returns a clean placeholder `mysqli` object that can flow into
  existing deterministic query and metadata boundaries, including direct MySQLi
  and WordPress-shaped `wpdb` smokes. This is not host socket connections,
  authentication, real database selection, init-command execution, server-state
  population, liveness proof, PHP warning/error fidelity, mysqlnd behavior, or
  native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi/PDO gap from the
  audited PHP surface, such as PDO visibility, real reference aliasing around
  bound parameters/results, broader escaping charset fidelity, mutation SQL
  state, transaction state, host-backed query execution, or the next real
  database integration gap, and add the next bounded behavior or explicit
  runtime boundary with tests, CLI fixtures, docs, and native rejection
  coverage where lowering remains unsupported.
  Milestone 1003 adds bounded PDO/pdo_mysql visibility with an explicit
  connection boundary. The deterministic extension registry reports `pdo` and
  `pdo_mysql`, the core class table seeds metadata-only `PDO` and
  `PDOStatement`, and reached `new PDO(...)` attempts fail with a stable
  unsupported diagnostic. This is not DSN parsing, host database connections,
  authentication, PDO driver behavior, statement preparation/execution, result
  fetching, transactions, attributes, error modes, `PDOException`, persistent
  connections, or native database lowering.
- [x] Runtime/database lane: inspect the next real MySQLi/PDO metadata gap
  from the audited PHP surface, such as PDO class constants, `PDOException`,
  real reference aliasing around bound parameters/results, broader escaping
  charset fidelity, mutation SQL state, transaction state, host-backed query
  execution, or the next real database integration gap, and add the next
  bounded behavior or explicit runtime boundary with tests, CLI fixtures,
  docs, and native rejection coverage where lowering remains unsupported.
  Milestone 1004 adds a bounded public integer `PDO` class-constant catalog
  for current error-mode, fetch-mode, and MySQL init-command metadata checks:
  `ATTR_ERRMODE`, `ERRMODE_SILENT`, `ERRMODE_WARNING`,
  `ERRMODE_EXCEPTION`, `ATTR_DEFAULT_FETCH_MODE`, `FETCH_ASSOC`, `FETCH_NUM`,
  `FETCH_BOTH`, and `MYSQL_ATTR_INIT_COMMAND`. This is not a full PDO
  constant catalog, `PDOException`, PDO attributes, error-mode behavior, DSN
  parsing, host database connections, statement execution, transactions,
  warning/error fidelity, or native database lowering.
- [x] PHP semantics lane: inspect the next difficult reference/copy-on-write
  gap, such as direct variable reference cells, by-reference `foreach`,
  reference returns, array-offset/object-property references, or alias-aware
  parameter passing, and add the next bounded behavior or explicit runtime
  boundary with tests, CLI fixtures, docs, and native rejection coverage where
  lowering remains unsupported.
  Milestone 1005 adds bounded direct variable-to-variable reference cells for
  statement-form `$alias =& $value;`. Both direct names share one mutable cell
  in the current scope/global-routing model, writes through either name are
  visible through the other, and `unset($alias)` detaches that name without
  deleting the source cell. This is not array-offset references,
  object-property references, by-reference `foreach`, reference returns,
  by-reference parameter aliasing during execution, source/target rebinding
  beyond direct names, full PHP reference containers, copy-on-write, or native
  lowering.
- [x] Runtime/database lane: inspect the next real MySQLi statement or
  connection/helper gap from the audited PHP surface, such as existing empty
  WordPress option/metadata query semantics, result-mode arguments, real
  reference aliasing around bound parameters/results, broader escaping charset
  fidelity, mutation SQL state, transaction state, host-backed query
  execution, or the next real database integration gap, and add the next
  bounded behavior or explicit runtime boundary with tests, CLI fixtures,
  docs, and native rejection coverage where lowering remains unsupported.
  Milestone 1006 makes direct `mysqli_query()` return deterministic empty
  `mysqli_result` placeholders for the exact reached WordPress options-table
  and metadata read shapes instead of returning `false`. The SQL-mode probe
  remains a false no-result boundary. This is not host-backed query execution,
  real table/schema reads, real field metadata, warning/error fidelity,
  mutation SQL, mysqlnd behavior, or native database lowering.
- [x] Object semantics lane: inspect the next magic method gap from the audited
  PHP/WordPress surface, such as missing-property `__get`/`__isset`,
  `__set`, `__unset`, dynamic property-name magic, `__call`, or ArrayAccess,
  and add the next bounded behavior or explicit runtime boundary with tests,
  CLI fixtures, docs, and native rejection coverage where lowering remains
  unsupported.
  Milestone 1007 adds bounded direct missing-property magic for ordinary
  `$object->name` reads, `isset($object->name)`, and `empty($object->name)`.
  Existing visible slots still use direct storage; missing slots call visible
  non-static `__get($name)` or `__isset($name)` under the current instance
  method execution model. This is not dynamic property-name magic,
  object-dimension magic, `__set`, `__unset`, `__call`, ArrayAccess,
  typed/uninitialized property behavior, exact warning/visibility diagnostics,
  recursion edge-case fidelity, references/copy-on-write, or native lowering.
- [x] Object semantics lane: inspect the adjacent missing-property write magic
  gap and add bounded direct `__set($name, $value)` behavior or an explicit
  runtime boundary with tests, CLI fixtures, docs, and native rejection
  coverage where lowering remains unsupported.
  Milestone 1008 adds bounded direct missing-property `__set($name, $value)`
  dispatch for ordinary direct `$object->name = expr` writes. Existing visible
  slots still write directly; missing slots call visible non-static `__set`
  when declared or inherited, ignore its return value, and keep assignment
  expressions returning the assigned value. This is not dynamic property-name
  magic, inaccessible-property `__set` fidelity, nested object-property array
  writes through magic, compound assignment, `??=`, increment/decrement,
  `__unset`, `__call`, ArrayAccess, typed/uninitialized property behavior,
  exact warning/visibility diagnostics, references/copy-on-write, or native
  lowering.
- [x] Object semantics lane: inspect the adjacent direct object-property
  `unset(...)` magic gap and add bounded `__unset($name)` behavior or an
  explicit runtime boundary with tests, CLI fixtures, docs, and native
  rejection coverage where lowering remains unsupported.
  Milestone 1009 adds bounded direct missing-property `__unset($name)`
  dispatch for ordinary direct `unset($object->name)`. Existing visible slots
  are still nulled under the current storage model; missing visible slots call
  visible non-static `__unset` when declared or inherited and ignore its
  return value. This is not true PHP property removal/uninitialization,
  inaccessible-property `__unset` fidelity, dynamic property-name magic,
  object-dimension magic, ArrayAccess, typed/uninitialized property behavior,
  exact warning/visibility diagnostics, references/copy-on-write, or native
  lowering.
- [x] Object semantics lane: inspect the adjacent missing-method magic gap and
  add bounded instance `__call($name, $args)` behavior or an explicit runtime
  boundary with tests, CLI fixtures, docs, and native rejection coverage where
  lowering remains unsupported.
  Milestone 1010 adds bounded instance `__call($name, $args)` dispatch for
  missing direct `$object->method(...)` calls. Declared visible methods still
  dispatch first; missing methods evaluate positional arguments left to right,
  package them in a zero-indexed PHP array, invoke visible non-static
  `__call`, and return its result. This is not inaccessible-method `__call`
  fidelity, dynamic method-name magic beyond the existing static-token
  method-call form, `__callStatic`, named arguments, splat/unpack behavior,
  by-reference argument aliasing, exact warning/visibility diagnostics,
  recursion edge-case fidelity, references/copy-on-write, or native lowering.
- [x] Object semantics lane: inspect the adjacent static missing-method magic
  gap and add bounded `__callStatic($name, $args)` behavior or an explicit
  runtime boundary with tests, CLI fixtures, docs, and native rejection
  coverage where lowering remains unsupported.
  Milestone 1011 adds bounded static `__callStatic($name, $args)` dispatch for
  missing named, dynamic-receiver, `self::`, and late `static::` method calls.
  Declared visible static methods still dispatch first; missing methods
  evaluate positional arguments left to right, package them in a zero-indexed
  PHP array, invoke visible static `__callStatic`, and preserve the receiver
  class as called-class context. This is not inaccessible-method
  `__callStatic` fidelity, `parent::` missing-method magic, dynamic
  method-name syntax, named arguments, splat/unpack behavior, by-reference
  argument aliasing, exact warning/visibility diagnostics, recursion edge-case
  fidelity, references/copy-on-write, or native lowering.
- [x] Object semantics lane: inspect the adjacent object-to-string magic gap
  and add bounded `__toString()` behavior or an explicit runtime boundary with
  tests, CLI fixtures, docs, and native rejection coverage where lowering
  remains unsupported.
  Milestone 1012 adds bounded object string conversion through visible
  non-static `__toString()` for `echo $object`, `print $object`,
  `(string) $object`, and binary concatenation. Objects without `__toString`
  keep the existing invalid string-conversion diagnostic; static
  `__toString()` and non-string returns remain stable boundaries. This is not
  `Stringable` metadata, object interpolation, heredoc conversion, compound
  concat assignment, exact `TypeError`/fatal behavior, recursion edge-case
  fidelity, references/copy-on-write, or native lowering.
- [x] Object semantics lane: inspect the next object protocol gap from the
  audited PHP/WordPress surface, such as concat compound assignment through
  `__toString`, ArrayAccess, object interpolation, `__clone` dispatch,
  destructors, or inaccessible-member magic fidelity, and add the next bounded
  behavior or explicit runtime boundary with tests, CLI fixtures, docs, and
  native rejection coverage where lowering remains unsupported.
  Milestone 1013 extends bounded object string conversion through visible
  non-static `__toString()` to concat compound assignment `.=` over current
  supported compound-assignment targets. This is not `Stringable` metadata,
  object interpolation, heredoc conversion, exact non-string-return
  `TypeError` objects, references/copy-on-write aliasing during
  read-modify-write, recursion edge-case fidelity, or native lowering.
- [x] Object semantics lane: add bounded direct object-offset `ArrayAccess`
  dispatch for classes that record `implements ArrayAccess` in current class
  metadata.
  Milestone 1014 covers direct `$object[$key]` reads, direct writes and append
  writes, direct `isset`, `empty`, `??`, and `unset` offset contexts by
  dispatching visible non-static `offsetGet`, `offsetSet`, `offsetExists`, and
  `offsetUnset` methods. This is not nested/mixed object-property/ArrayAccess
  paths, ArrayAccess increment/decrement, ArrayAccess iteration, built-in
  interface enforcement/signature validation, typed method invocation, exact
  diagnostics, references/copy-on-write, or native lowering.
- [x] Object semantics lane: add bounded direct object-offset `ArrayAccess`
  compound assignment for classes that record `implements ArrayAccess` in
  current class metadata.
  Milestone 1015 covers direct `$object[$key] op= expr` by dispatching visible
  non-static `offsetGet($key)`, applying the current compound-assignment
  helper, and dispatching visible non-static `offsetSet($key, $value)` for the
  updated value. This is not nested/mixed object-property/ArrayAccess paths,
  append compound assignment, ArrayAccess increment/decrement, ArrayAccess
  iteration, built-in interface enforcement/signature validation, typed method
  invocation, exact diagnostics, references/copy-on-write, or native lowering.
- [x] Object semantics lane: add bounded direct object-offset `ArrayAccess`
  increment/decrement for classes that record `implements ArrayAccess` in
  current class metadata.
  Milestone 1016 covers direct `++$object[$key]`, `$object[$key]++`,
  `--$object[$key]`, and `$object[$key]--` by dispatching visible non-static
  `offsetGet($key)`, applying the current integer/float increment/decrement
  helper to PHP's current by-value temporary result, and not dispatching
  `offsetSet($key, $value)`. This is not by-reference `offsetGet()` mutation,
  indirect-modification notice fidelity, nested/mixed object-property/
  ArrayAccess paths, append compound assignment, ArrayAccess iteration,
  built-in interface enforcement/signature validation, typed method
  invocation, string increment/decrement semantics, exact diagnostics,
  references/copy-on-write, or native lowering.
- [x] Object semantics lane: add bounded object string conversion through
  visible non-static `__toString()` for current double-quoted string and
  heredoc interpolation.
  Milestone 1017 covers simple `$object`, array-offset object values, direct
  object-property values, and supported braced interpolation chains by reusing
  the same echo-string conversion path as `echo` and binary concatenation.
  This is not `Stringable` metadata, `${...}` interpolation, dynamic/static
  property interpolation, arbitrary expression interpolation, exact
  non-string-return `TypeError` objects, recursion edge-case fidelity,
  references/copy-on-write, or native lowering.
- [x] Object semantics lane: add bounded `Stringable` core-interface metadata
  for the current object string-conversion slice.
  Milestone 1018 covers `interface_exists("Stringable")`,
  `get_declared_interfaces()` including the bounded core entry, explicit
  `implements Stringable` metadata, and `instanceof`, `is_a()`, and
  `is_subclass_of()` relationship checks for classes with a resolved public
  non-static `__toString()`, including inherited methods. This is not a broad
  built-in interface catalog, interface method enforcement, PHP fatal
  validation for invalid `__toString()` declarations, exact `Stringable` type
  diagnostics, reflection metadata, references/copy-on-write, or native
  lowering.
- [x] Object semantics lane: add a bounded core interface catalog for internal
  interface names already reached by WordPress-shaped object metadata.
  Milestone 1019 covers `Traversable`, `IteratorAggregate`, `Iterator`,
  `Serializable`, `ArrayAccess`, `Countable`, and `Stringable` in
  `interface_exists()` and `get_declared_interfaces()`, while relationship
  checks still use explicit `implements` metadata except for the existing
  bounded `Stringable` `__toString()` rule. This is not interface method
  enforcement, iterator execution, array-access/countable protocol validation,
  broad internal interface coverage, reflection metadata, exact diagnostics,
  references/copy-on-write, or native lowering.
- [x] Object semantics lane: add bounded direct object-property `ArrayAccess`
  offset dispatch for visible properties whose value is an `ArrayAccess`
  object.
  Milestone 1020 covers `$holder->bag[$key]` read/write, `isset`, `empty`,
  `??`, and `unset` by dispatching `offsetGet`, `offsetSet`, `offsetExists`,
  and `offsetUnset` on the property value. This is not nested `ArrayAccess`
  chains, append offsets, compound assignment, increment/decrement,
  magic-property-provided containers, ArrayAccess iteration, by-reference
  `offsetGet()` mutation, protocol/signature enforcement, exact diagnostics,
  references/copy-on-write, or native lowering.
- [x] Object semantics lane: add bounded direct object-property `ArrayAccess`
  append dispatch for visible properties whose value is an `ArrayAccess`
  object.
  Milestone 1021 covers `$holder->bag[] = $value` by dispatching
  `offsetSet(null, $value)` on the property-held object and preserving the
  assignment expression result. This is not nested `ArrayAccess` chains,
  keyed object-property `ArrayAccess` compound assignment,
  increment/decrement, magic-property-provided containers, ArrayAccess
  iteration, by-reference `offsetGet()` mutation, protocol/signature
  enforcement, exact diagnostics, references/copy-on-write, or native
  lowering.
- [x] Object semantics lane: add bounded keyed compound assignment for direct
  object-property `ArrayAccess` offsets.
  Milestone 1022 covers `$holder->bag[$key] op= expr` by dispatching
  `offsetGet($key)`, applying the existing compound-assignment helper, and
  dispatching `offsetSet($key, $value)` on the property-held object. This is
  not nested `ArrayAccess` chains, append compound assignment,
  increment/decrement, magic-property-provided containers, ArrayAccess
  iteration, by-reference `offsetGet()` mutation, protocol/signature
  enforcement, exact diagnostics, references/copy-on-write, or native
  lowering.
- [x] Object semantics lane: add bounded increment/decrement for direct
  object-property `ArrayAccess` offsets.
  Milestone 1023 covers `++$holder->bag[$key]`, `$holder->bag[$key]++`,
  `--$holder->bag[$key]`, and `$holder->bag[$key]--` by dispatching
  `offsetGet($key)` and applying the existing integer/float update helper to
  PHP's current by-value temporary result without dispatching
  `offsetSet($key, $value)`. This is not nested `ArrayAccess` chains, append
  compound assignment, magic-property-provided containers, ArrayAccess
  iteration, by-reference `offsetGet()` mutation, indirect-modification notice
  fidelity, protocol/signature enforcement, exact diagnostics,
  references/copy-on-write, or native lowering.
- [x] Object semantics lane: add bounded `Countable` object protocol support.
  Milestone 1040 covers `is_countable($object)` for object values whose class
  metadata records `implements Countable`, and `count($object)` dispatches a
  visible non-static `count()` method with an integer result. This is not full
  interface signature enforcement, magic `__call` fallback,
  `Traversable`/iterator protocol support, inaccessible/static count methods,
  non-integer count-result coercion, exact diagnostics, references/copy-on-write,
  or native lowering.
- [x] Object semantics lane: add bounded `is_iterable()` metadata support for
  iterator-like objects.
  Milestone 1043 covers `is_iterable($object)` for object values whose class
  metadata records `implements Traversable`, `implements Iterator`, or
  `implements IteratorAggregate`. This is not object `foreach`, `Iterator`
  method execution, `IteratorAggregate::getIterator()`, generators,
  direct-`Traversable` validation rules, protocol/signature enforcement, exact
  diagnostics, references/copy-on-write, or native lowering.
- [ ] Object semantics lane: inspect the next object protocol gap from the
  audited PHP/WordPress surface, such as nested ArrayAccess chains,
  append compound assignment, ArrayAccess iteration, method enforcement for
  internal interfaces, `__clone` dispatch, destructors, or
  inaccessible-member magic fidelity, and add the next bounded behavior or
  explicit runtime boundary with tests, CLI fixtures, docs, and native
  rejection coverage where lowering remains unsupported.
- [x] Runtime/database lane: add a bounded real-state island for exact
  WordPress-shaped `wp_options` MySQLi insert/readback.
  Milestone 1024 covers exact direct
  `INSERT INTO wp_options (option_name, option_value, autoload) VALUES (...)`
  followed by exact direct
  `SELECT option_value FROM wp_options WHERE option_name = ... LIMIT 1` on the
  same placeholder handle. The insert records the string option value in
  per-handle state, sets `mysqli_affected_rows($handle)` to `1`, advances
  deterministic `mysqli_insert_id($handle)`, and the select exposes the value
  through the existing placeholder result/fetch path. This is not broad SQL
  parsing, escaping/quoting fidelity, schema/index behavior, UPDATE/DELETE/
  REPLACE, transactions, host database execution, warning/error fidelity, PDO,
  prepared-statement mutation state, or native lowering.
- [x] Runtime/database lane: add bounded exact `wp_options` MySQLi
  update/readback over the current per-handle state island.
  Milestone 1025 covers exact direct
  `UPDATE wp_options SET option_value = ... WHERE option_name = ...` after an
  exact supported insert. Existing recorded option names update their string
  value, set `mysqli_affected_rows($handle)` to `1`, and later exact
  option-value `SELECT` reads expose the updated value through the existing
  placeholder result/fetch path. Missing option names are successful zero-row
  updates. This is not broad SQL parsing, escaped quote handling,
  schema/index behavior, INSERT-on-duplicate behavior, DELETE/REPLACE,
  transactions, host database execution, warning/error fidelity, PDO,
  prepared-statement mutation state, or native lowering.
- [x] Runtime/database lane: add bounded exact `wp_options` MySQLi
  delete/readback over the current per-handle state island.
  Milestone 1026 covers exact direct
  `DELETE FROM wp_options WHERE option_name = ...` after an exact supported
  insert has created the per-handle state island. Existing recorded option
  names are removed, set `mysqli_affected_rows($handle)` to `1`, and later
  exact option-value `SELECT` reads return an empty placeholder result.
  Missing option names are successful zero-row deletes. This is not broad SQL
  parsing, escaped quote handling, schema/index behavior,
  INSERT-on-duplicate behavior, DELETE breadth, REPLACE, transactions, host
  database execution, warning/error fidelity, PDO, prepared-statement mutation
  state, or native lowering.
- [x] Runtime/database lane: add bounded `wp_options` MySQLi row readback over
  the current per-handle state island.
  Milestone 1027 covers exact direct
  `SELECT option_name, option_value FROM wp_options`, exact direct
  `SELECT option_name, option_value FROM wp_options WHERE autoload IN ( 'yes',
  'on', 'auto-on', 'auto' )`, and exact direct `WHERE option_name IN (...)`
  row reads after exact supported inserts. All/autoload reads return recorded
  option rows in deterministic option-name order; explicit `IN (...)` reads
  preserve requested name order and skip missing names. This is not broad SQL
  parsing, escaped quote handling, schema/index behavior, ordering/collation
  fidelity, autoload mutation beyond exact inserts, INSERT-on-duplicate
  behavior, DELETE breadth, REPLACE, transactions, host database execution,
  warning/error fidelity, PDO, prepared-statement mutation state, or native
  lowering.
- [x] Runtime/database lane: add bounded prepared `wp_options` MySQLi value
  readback over the current per-handle state island.
  Milestone 1028 covers exact
  `SELECT option_value FROM wp_options WHERE option_name = ?` reads through
  `mysqli_stmt_execute()`/`mysqli_stmt_get_result()` and
  `mysqli_execute_query($handle, $query, array($name))` for string option-name
  parameters on the same placeholder handle. Missing names return an empty
  placeholder result. This is not broad prepared SQL execution, prepared
  mutation state, non-string option-name parameter coercion, result binding
  fidelity beyond the existing placeholder result path, host database
  execution, PDO, or native lowering.
- [x] Runtime/database lane: add bounded prepared `wp_options` MySQLi row
  readback over the current per-handle state island.
  Milestone 1029 covers exact
  `SELECT option_name, option_value FROM wp_options WHERE option_name = ?`
  reads through `mysqli_stmt_execute()`/`mysqli_stmt_get_result()` and
  `mysqli_execute_query($handle, $query, array($name))` for string option-name
  parameters on the same placeholder handle. Missing names return an empty
  zero-field placeholder result. This is not broad prepared SQL execution,
  prepared mutation state, non-string option-name parameter coercion, result
  binding fidelity beyond exact metadata, host database execution, PDO, or
  native lowering.
- [x] Runtime/database lane: add bounded prepared `wp_options` MySQLi update
  state over the current per-handle state island.
  Milestone 1030 covers exact
  `UPDATE wp_options SET option_value = ? WHERE option_name = ?` execution
  through `mysqli_stmt_execute()` for string value/name parameters on the same
  placeholder handle after exact supported insert state exists. Existing
  recorded option names update their value, set
  `mysqli_stmt_affected_rows($stmt)` and `mysqli_affected_rows($handle)` to
  `1`, and missing names are successful zero-row updates. This is not broad
  prepared SQL execution, prepared INSERT/DELETE/REPLACE state, non-string
  parameter coercion, transactions, host database execution, PDO, or native
  lowering.
- [x] Runtime/database lane: add bounded prepared `wp_options` MySQLi delete
  state over the current per-handle state island.
  Milestone 1031 covers exact
  `DELETE FROM wp_options WHERE option_name = ?` execution through
  `mysqli_stmt_execute()` for string option-name parameters on the same
  placeholder handle after exact supported insert state exists. Existing
  recorded option names are removed, set
  `mysqli_stmt_affected_rows($stmt)` and `mysqli_affected_rows($handle)` to
  `1`, and missing names are successful zero-row deletes. This is not broad
  prepared SQL execution, prepared INSERT/REPLACE state, non-string parameter
  coercion, transactions, host database execution, PDO, or native lowering.
- [x] Runtime/database lane: add bounded prepared `wp_options` MySQLi insert
  state over the current per-handle state island.
  Milestone 1032 covers exact
  `INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)`
  execution through `mysqli_stmt_execute()` for string option-name,
  option-value, and autoload parameters on the same placeholder handle.
  Inserted option names are recorded in the state island, set
  `mysqli_stmt_affected_rows($stmt)` and `mysqli_affected_rows($handle)` to
  `1`, advance deterministic `mysqli_insert_id($handle)`, and are visible to
  later exact option-value reads. This is not broad prepared SQL execution,
  duplicate-key or INSERT-on-duplicate semantics, prepared REPLACE state,
  non-string parameter coercion, transactions, host database execution, PDO,
  or native lowering.
- [x] Runtime/database lane: add bounded prepared `wp_options` MySQLi replace
  state over the current per-handle state island.
  Milestone 1033 covers exact
  `REPLACE INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)`
  execution through `mysqli_stmt_execute()` for string option-name,
  option-value, and autoload parameters on the same placeholder handle.
  Existing recorded options are replaced with statement/connection affected
  rows set to `2`; missing options are inserted with affected rows set to `1`.
  Both paths advance deterministic `mysqli_insert_id($handle)` and are visible
  to later exact option-value reads. This is not broad prepared SQL execution,
  `INSERT ... ON DUPLICATE KEY UPDATE`, real unique-index enforcement,
  non-string parameter coercion, transactions, host database execution, PDO,
  or native lowering.
- [x] Runtime/database lane: add bounded prepared `wp_options` MySQLi
  insert-on-duplicate state over the current per-handle state island.
  Milestone 1034 covers exact WordPress-style
  `INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)
  ON DUPLICATE KEY UPDATE ...` execution through `mysqli_stmt_execute()` for
  string option-name, option-value, and autoload parameters on the same
  placeholder handle. Existing recorded options update with
  statement/connection affected rows set to `2`; missing options insert with
  affected rows set to `1`. Both paths advance deterministic
  `mysqli_insert_id($handle)` and are visible to later exact option-value
  reads. This is not broad prepared SQL execution, real unique-index
  enforcement, no-op update affected-row fidelity, non-string parameter
  coercion, transactions, host database execution, PDO, or native lowering.
- [x] Runtime/database lane: add bounded direct `wp_options` MySQLi
  insert-on-duplicate state over the current per-handle state island.
  Milestone 1035 covers exact WordPress-style
  `INSERT INTO wp_options (option_name, option_value, autoload) VALUES (...)
  ON DUPLICATE KEY UPDATE ...` execution through `mysqli_query()` for
  single-quoted string option-name, option-value, and autoload values on the
  same placeholder handle. Existing recorded options update with connection
  affected rows set to `2`; missing options insert with affected rows set to
  `1`. Both paths advance deterministic `mysqli_insert_id($handle)` and are
  visible to later exact option-value reads. This is not broad SQL parsing,
  real unique-index enforcement, no-op update affected-row fidelity, escaped
  quote handling, transactions, host database execution, PDO, or native
  lowering.
- [x] Runtime/database lane: inspect the next real database-state gap from the
  audited PHP/WordPress surface, such as transaction state, broader escaping
  fidelity, host-backed query execution, PDO, real schema/index behavior, or
  the next `wpdb` state consumer, and add the next bounded behavior or
  explicit runtime boundary with tests, CLI fixtures, docs, and native
  rejection coverage where lowering remains unsupported.
  Milestone 1036 covers bounded transaction snapshots for the current exact
  per-handle `wp_options` state island: `mysqli_begin_transaction()` and
  `mysqli_autocommit(false)` capture option state, `mysqli_rollback()`
  restores it, and `mysqli_commit()`/`mysqli_autocommit(true)` keep later
  changes. This is not real host transaction state, savepoints,
  isolation/locking, auto-increment rollback fidelity, broad SQL execution,
  PDO, or native lowering.
- [x] Runtime/database lane: inspect the next real database-state gap from the
  audited PHP/WordPress surface, such as broader escaping fidelity,
  host-backed query execution, PDO, real schema/index behavior, savepoints, or
  the next `wpdb` state consumer, and add the next bounded behavior or
  explicit runtime boundary with tests, CLI fixtures, docs, and native
  rejection coverage where lowering remains unsupported.
  Milestone 1037 covers bounded savepoint snapshots for the current exact
  per-handle `wp_options` state island: `mysqli_savepoint()` records a named
  option-state snapshot, `mysqli_rollback($handle, 0, $name)` restores it, and
  `mysqli_release_savepoint()` removes it. This is not real host savepoint
  state, savepoint nesting/release diagnostics, isolation/locking, broad SQL
  execution, PDO, or native lowering.
- [x] Runtime/database lane: inspect the next real database-state gap from the
  audited PHP/WordPress surface, such as broader escaping fidelity,
  host-backed query execution, PDO, real schema/index behavior, nested
  transactions, or the next `wpdb` state consumer, and add the next bounded
  behavior or explicit runtime boundary with tests, CLI fixtures, docs, and
  native rejection coverage where lowering remains unsupported.
  Milestone 1038 covers bounded MySQL-style escaped single-quoted literals for
  exact direct `wp_options` option insert/update/delete/readback SQL shapes,
  including `mysqli_real_escape_string()`-style escaped quotes, double quotes,
  backslashes, newlines, and carriage returns. This is not broad SQL parsing,
  SQL-mode-aware escaping, character-set/collation fidelity, host database
  execution, PDO, or native lowering.
- [x] Runtime/database lane: add bounded exact direct `wp_options` MySQLi
  replace state over the current per-handle state island.
  Milestone 1039 covers exact direct
  `REPLACE INTO wp_options (option_name, option_value, autoload) VALUES (...)`
  execution through `mysqli_query()` for single-quoted string option-name,
  option-value, and autoload values on the same placeholder handle. Existing
  recorded options replace with connection affected rows set to `2`; missing
  options insert with affected rows set to `1`. Both paths advance
  deterministic `mysqli_insert_id($handle)` and are visible to later exact
  option-value reads. This is not broad SQL parsing, real `REPLACE`/
  unique-index/delete-trigger/auto-increment fidelity, schema/index behavior,
  host database execution, PDO, or native lowering.
- [x] Runtime/database lane: add bounded exact direct `wp_options` MySQLi
  autoload readback over the current per-handle state island.
  Milestone 1041 covers exact direct
  `SELECT autoload FROM wp_options WHERE option_name = ... LIMIT 1` execution
  through `mysqli_query()` for single-quoted string option names on the same
  placeholder handle. Existing recorded options expose their recorded autoload
  value through the existing placeholder result/fetch path, missing options
  return an empty placeholder result, and the read sets
  `mysqli_affected_rows($handle)` to `0`. This is not broad SQL parsing,
  arbitrary projection, SQL-mode-aware escaping, character-set/collation
  fidelity, schema/index behavior, host database execution, PDO, prepared
  statement autoload reads, or native lowering.
- [ ] Runtime/database lane: inspect the next real database-state gap from the
  audited PHP/WordPress surface, such as host-backed query execution, PDO, real
  schema/index behavior, nested transactions, broader SQL-mode-aware escaping,
  or the next `wpdb` state consumer, and add the next bounded behavior or
  explicit runtime boundary with tests, CLI fixtures, docs, and native
  rejection coverage where lowering remains unsupported.

## Milestone 1045: Reference/COW Continuation

- [x] Runtime/value-model lane: replace the current direct-variable
  by-reference parameter copy-back path with shared caller/callee variable
  cells during function and method execution. Supplied direct-variable
  by-reference parameters now observe caller-visible writes before return, and
  `unset($param)` detaches only the callee local name. This is not full PHP
  reference containers, array/object offset references, reference returns,
  by-reference `foreach`, copy-on-write, non-variable arguments, broader
  callback reference behavior, or native lowering.

## Milestone 1046: Reference/COW Continuation

- [x] Runtime/value-model lane: extend the shared caller/callee variable-cell
  path to direct-variable by-reference constructor parameters in the current
  public/inherited constructor dispatch subset. Constructor writes through
  `&$param` are now visible before `new ClassName($value)` returns, and
  `unset($param)` detaches only the constructor's local name. This is not full
  PHP reference containers, array/object offset references, reference returns,
  by-reference `foreach`, copy-on-write, non-variable constructor arguments,
  static constructor support, or native object lowering.

## Milestone 1047: Reference/COW Continuation

- [x] Runtime/value-model lane: add a bounded direct-variable
  reference-return assignment path for direct free-function calls. In the
  current subset, `$alias =& identity($value);` can bind the alias name to the
  direct variable cell returned by
  `function &identity(&$value) { return $value; }`, and `unset($alias)`
  detaches only the alias name. This is not normal reference-return
  invocation, method-call reference-return sources, non-direct return
  expressions, nested-control-flow returns, array/object offset aliases,
  by-reference `foreach`, full PHP reference containers, copy-on-write, or
  native lowering.

## Milestone 1048: Reference/COW Continuation

- [x] Runtime/value-model lane: extend the bounded direct-variable
  reference-return assignment path to direct object method calls. In the
  current subset, `$alias =& $object->identity($value);` can bind the alias
  name to the direct variable cell returned by a visible non-static
  `public function &identity(&$value) { return $value; }`, and `unset($alias)`
  detaches only the alias name. This is not normal reference-return
  invocation, magic/static/parent/self reference-return method sources,
  non-direct return expressions, nested-control-flow returns, array/object
  offset aliases, by-reference `foreach`, full PHP reference containers,
  copy-on-write, or native lowering.

## Milestone 1049: Reference/COW Continuation

- [x] Runtime/value-model lane: extend the bounded direct-variable
  reference-return assignment path to direct named static method calls. In the
  current subset, `$alias =& Box::identity($value);` can bind the alias name to
  the direct variable cell returned by a visible static
  `public static function &identity(&$value) { return $value; }`, and
  `unset($alias)` detaches only the alias name. This is not normal
  reference-return invocation, `self::`, `parent::`, `static::`, dynamic
  static receiver, or magic `__callStatic` reference-return method sources,
  non-direct return expressions, nested-control-flow returns, array/object
  offset aliases, by-reference `foreach`, full PHP reference containers,
  copy-on-write, or native lowering.

## Milestone 1050: Reference/COW Continuation

- [x] Runtime/value-model lane: extend the bounded direct-variable
  reference-return assignment path to `self::` static method calls in an active
  class/method context. In the current subset,
  `$alias =& self::identity($value);` can bind the alias name to the direct
  variable cell returned by a visible static
  `public static function &identity(&$value) { return $value; }`, and
  `unset($alias)` detaches only the alias name. This is not normal
  reference-return invocation, non-static `self::` sources, `parent::`,
  `static::`, dynamic static receiver, or magic `__callStatic`
  reference-return method sources, non-direct return expressions,
  nested-control-flow returns, array/object offset aliases, by-reference
  `foreach`, full PHP reference containers, copy-on-write, or native lowering.

## Milestone 1051: Reference/COW Continuation

- [x] Runtime/value-model lane: extend the bounded direct-variable
  reference-return assignment path to `parent::` static method calls in an
  active child class/method context. In the current subset,
  `$alias =& parent::identity($value);` can bind the alias name to the direct
  variable cell returned by a visible inherited static
  `public static function &identity(&$value) { return $value; }`, and
  `unset($alias)` detaches only the alias name. This is not normal
  reference-return invocation, non-static `parent::` sources, missing-parent
  parent calls, `static::`, dynamic static receiver, or magic `__callStatic`
  reference-return method sources, non-direct return expressions,
  nested-control-flow returns, array/object offset aliases, by-reference
  `foreach`, full PHP reference containers, copy-on-write, or native lowering.

## Milestone 1052: Reference/COW Continuation

- [x] Runtime/value-model lane: extend the bounded direct-variable
  reference-return assignment path to `static::` late-static method calls in
  an active class/method context. In the current subset,
  `$alias =& static::identity($value);` can bind the alias name to the direct
  variable cell returned by a visible late-bound static
  `public static function &identity(&$value) { return $value; }`, including
  base and child called-class contexts, and `unset($alias)` detaches only the
  alias name. This is not normal reference-return invocation,
  `static::` sources outside class/method context, non-static `static::`
  sources, dynamic static receiver, or magic `__callStatic` reference-return
  method sources, non-direct return expressions, nested-control-flow returns,
  array/object offset aliases, by-reference `foreach`, full PHP reference
  containers, copy-on-write, or native lowering.

## Milestone 1053: Reference/COW Continuation

- [x] Runtime/value-model lane: extend the bounded direct-variable
  reference-return assignment path to dynamic static receiver method calls when
  the receiver evaluates to an object or class string. In the current subset,
  `$alias =& $class::identity($value);` and
  `$alias =& $object::identity($value);` can bind the alias name to the direct
  variable cell returned by a visible static
  `public static function &identity(&$value) { return $value; }`, and
  `unset($alias)` detaches only the alias name. This is not normal
  reference-return invocation, non-object/non-string dynamic receivers,
  non-static dynamic static receiver sources, magic `__callStatic`
  reference-return method sources, non-direct return expressions,
  nested-control-flow returns, array/object offset aliases, by-reference
  `foreach`, full PHP reference containers, copy-on-write, or native lowering.

## Milestone 1054: Reference/COW Continuation

- [x] Runtime/value-model lane: add an explicit runtime boundary for magic
  `__callStatic` reference-return method sources in statement-form reference
  assignments. Missing static methods such as
  `$alias =& Box::missing($value);` now report a stable unsupported
  `Box::__callStatic()` diagnostic when the receiver class declares or
  inherits `__callStatic`, instead of falling through to a plain undefined
  static method error. This is not magic reference-return dispatch, PHP
  reference containers, argument array aliasing, array/object offset aliases,
  by-reference `foreach`, copy-on-write, or native lowering.

## Milestone 1055: Reference/COW Continuation

- [x] Runtime/value-model lane: add a bounded direct-array-variable
  by-reference `foreach` copy-back execution slice. In the current subset,
  `foreach ($items as $key => &$item) { ... }` snapshots the direct array
  variable's keys, writes the key by value, executes the body with the current
  element value in the loop variable, and copies that loop variable back into
  the same array slot after each iteration. This is not true array slot cells,
  lingering post-loop references, mutation-during-iteration fidelity,
  non-direct by-reference iterables, object/Traversable iteration,
  copy-on-write, or native lowering.

## Milestone 1056: Reference/COW Continuation

- [x] Runtime/value-model lane: add array-entry accessor groundwork for future
  slot/reference cells without changing observable array behavior.
  `ArrayEntry.value` is private, runtime/compiler callers go through
  `value()`, `value_cloned()`, `value_mut()`, `set_value()`, or `into_value()`,
  and a focused runtime test pins that cloned arrays still have independent
  entry values when overwritten. This is not direct array-offset references,
  real slot cells, copy-on-write, exact by-reference `foreach`, object-property
  offsets, or native lowering.

## Milestone 1057: Reference/COW Continuation

- [x] Runtime/value-model lane: replace raw `ArrayEntry` value storage with an
  explicit clone-by-value `ArraySlot` wrapper while preserving observable array
  behavior. `ArrayEntry` now stores a slot, its value accessors delegate
  through that slot, and a focused runtime test proves that cloned slots holding
  nested PHP arrays still have independent values when the clone is mutated.
  This is not shared reference cells, direct array-offset references,
  copy-on-write, exact by-reference `foreach`, object-property offsets, or
  native lowering.

## Milestone 1058: Reference/COW Continuation

- [x] Runtime/value-model lane: add explicit normalized-key array slot lookup
  helpers on top of `ArraySlot`. `PhpArray::get_slot()` and `get_slot_mut()`
  expose the storage object for an existing key, `PhpArray::get()` and
  `insert()` route through that boundary, and a focused runtime test proves
  mutable slot lookup updates only the selected array while cloned arrays keep
  independent slot values. This is not shared reference cells, direct
  array-offset references, copy-on-write, exact by-reference `foreach`,
  object-property offsets, or native lowering.

## Milestone 1059: Reference/COW Continuation

- [x] Runtime/value-model lane: route direct array-offset reference-assignment
  sources through the new array slot lookup boundary before rejecting. In the
  current subset, `$alias =& $array[$key];` evaluates the key, reads the direct
  array variable, reaches normalized-key `ArraySlot` lookup, and reports a
  stable diagnostic naming the missing array slot reference cells. This is not
  shared reference cells, missing-offset reference materialization,
  copy-on-write, exact by-reference `foreach`, object-property offsets, or
  native lowering.

## Milestone 1060: Reference/COW Continuation

- [x] Runtime/value-model lane: introduce a private `ArraySlotCell` layer
  under `ArraySlot` while preserving clone-by-value behavior. `ArraySlot`
  delegates all value access/mutation through the internal cell, and a focused
  runtime test proves cloning a slot allocates an independent cell whose
  mutation does not affect the original. This is not shared reference cells,
  direct array-offset references, missing-offset reference materialization,
  copy-on-write, exact by-reference `foreach`, object-property offsets, or
  native lowering.

## Milestone 1061: Reference/COW Continuation

- [x] Runtime/value-model lane: add stable internal identity for array slot
  cells without changing observable value equality. `ArraySlotCell` now carries
  an `ArraySlotCellId`, `ArraySlot::cell_id()` exposes it, cloning a slot
  allocates a distinct cell id with a cloned value, and focused runtime tests
  prove distinct same-value slots still compare equal. This is not shared
  reference cells, direct array-offset references, missing-offset reference
  materialization, copy-on-write, exact by-reference `foreach`,
  object-property offsets, or native lowering.

## Milestone 1062: Reference/COW Continuation

- [x] Runtime/value-model lane: add an internal shared-cell primitive behind
  `ArraySlot` while preserving public by-value writes. `ArraySlot` now stores
  its private cell behind a shared handle, normal slot cloning still allocates
  a distinct cloned-value cell, and public writes detach from any shared cell
  before mutation. This is not PHP-visible array-offset aliasing, missing-offset
  reference materialization, copy-on-write, exact by-reference `foreach`,
  object-property offsets, or native lowering.

## Milestone 1063: Reference/COW Continuation

- [x] Runtime/value-model lane: route one bounded PHP-visible direct
  array-offset reference-source slice through the interpreter. In the current
  subset, `$alias =& $array[$key];` works when the source is a direct array
  variable, the evaluated normalized key already exists, and the target is a
  direct variable: alias writes update the selected array slot, direct array
  offset writes are visible through the alias, chained direct aliases share
  that slot route, and `unset($alias)` detaches only the alias name. This is
  not missing-offset reference materialization, nested/object/`ArrayAccess`
  offsets, direct array-offset reference targets, exact by-reference
  `foreach`, copy-on-write, or native lowering.

## Milestone 1064: Reference/COW Continuation

- [x] Runtime/value-model lane: materialize missing keys for the bounded direct
  array-offset reference-source slice. In the current subset,
  `$alias =& $array[$key];` over an existing direct array variable now
  materializes an absent normalized key as `null`, then binds the direct target
  variable to that slot route. This is not undefined/null root materialization,
  nested/object/`ArrayAccess` offsets, direct array-offset reference targets,
  exact by-reference `foreach`, copy-on-write, or native lowering.

## Milestone 1065: Reference/COW Continuation

- [x] Runtime/value-model lane: materialize undefined or `null` direct roots
  for the bounded direct array-offset reference-source slice. In the current
  subset, `$alias =& $array[$key];` now converts an undefined or `null` direct
  source root into an array containing the selected normalized key as `null`,
  then binds the direct target variable to that slot route. Alias writes update
  the materialized slot, direct offset writes are visible through the alias,
  and non-array roots remain an explicit runtime boundary. This is not
  nested/object/`ArrayAccess` offsets, direct array-offset reference targets,
  exact by-reference `foreach`, copy-on-write, or native lowering.

## Milestone 1066: Reference/COW Continuation

- [x] Runtime/value-model lane: add lingering post-loop reference behavior for
  the existing direct-array-variable by-reference `foreach` slice. In the
  current subset, `foreach ($items as $key => &$item) { ... }` snapshots the
  initial keys, copies body writes back to each selected slot, and after loop
  completion leaves the value variable bound to the last successfully iterated
  array slot until `unset($item)` detaches it. Empty array iteration creates no
  lingering reference. This is not mutation-during-iteration fidelity,
  non-direct iterable support, object/`Traversable` iteration, foreach
  destructuring, full PHP reference containers, copy-on-write, or native
  lowering.

## Milestone 1067: Reference/COW Continuation

- [x] Runtime/value-model lane: add a bounded direct array-offset
  reference-target slice for unaliased direct variable sources. In the current
  subset, `$array[$key] =& $value;` works when the target root is a direct
  array variable, the offset is explicit, and the source is a direct unaliased
  variable name. Missing keys and undefined or `null` target roots materialize
  through the direct-offset array path, undefined source variables begin as
  `null`, writes through the source variable and direct offset observe the same
  selected value, and `unset($value)` detaches only the source name. This is
  not existing alias groups, source names already routed through array-offset
  aliases, `$GLOBALS`, append targets, nested/object/`ArrayAccess` targets,
  non-direct sources, full PHP reference containers, copy-on-write, exact alias
  rebinding/mutation ordering, or native lowering.

## Milestone 1068: Reference/COW Continuation

- [x] Runtime/value-model lane: add a bounded direct array-append
  reference-target slice for unaliased direct variable sources. In the current
  subset, `$array[] =& $value;` works when the target root is a direct array
  variable and the source is a direct unaliased variable name. Existing arrays
  append through the runtime array append cursor and bind the source name to
  the selected auto key, undefined or `null` target roots materialize as
  arrays, undefined source variables begin as `null`, writes through the source
  variable and appended direct offset observe the same selected value, and
  `unset($value)` detaches only the source name. This is not existing alias
  groups, source names already routed through array-offset aliases, `$GLOBALS`,
  PHP's deprecated false-root conversion, other non-array roots,
  nested/object/`ArrayAccess` append targets, non-direct sources, full PHP
  reference containers, copy-on-write, exact alias rebinding/mutation ordering,
  or native lowering.

## Milestone 1069: Reference/COW Continuation

- [x] Runtime/value-model lane: add a bounded nested direct array-offset
  reference-target slice for unaliased direct variable sources. In the current
  subset, `$array[$outer][$inner] =& $value;` works when the root is a direct
  array variable, every offset is explicit, and the source is a direct
  unaliased variable name. The internal alias route now stores a normalized key
  path, missing intermediate containers and undefined or `null` target roots
  materialize through the nested-array path, undefined source variables begin
  as `null`, writes through the source variable and nested direct offset
  observe the same selected value, and `unset($value)` detaches only the source
  name. This is not existing alias groups, source names already routed through
  array-offset aliases, `$GLOBALS`, nested append reference targets,
  object-property/`ArrayAccess` targets, non-direct sources, full PHP reference
  containers, copy-on-write, exact alias rebinding/mutation ordering, or native
  lowering.

## Milestone 1070: Reference/COW Continuation

- [x] Runtime/value-model lane: add a bounded nested direct array-append
  reference-target slice for unaliased direct variable sources. In the current
  subset, `$array[$outer][] =& $value;` works when the root is a direct array
  variable, every parent offset is explicit, and the source is a direct
  unaliased variable name. Missing parent containers and undefined or `null`
  target roots materialize through the nested-array path, the runtime append
  cursor selects the nested auto key, undefined source variables begin as
  `null`, writes through the source variable and appended nested direct offset
  observe the same selected value, and `unset($value)` detaches only the source
  name. This is not existing alias groups, source names already routed through
  array-offset aliases, `$GLOBALS`, object-property/`ArrayAccess` targets,
  non-direct sources, full PHP reference containers, copy-on-write, exact alias
  rebinding/mutation ordering, or native lowering.

## Milestone 1071: Reference/COW Continuation

- [x] Runtime/value-model lane: add a bounded direct public object-property
  array-offset reference-target slice for unaliased direct variable sources. In
  the current subset, `$object->items[$key] =& $value;` works when the target
  object is a direct object variable, the property is declared public and
  reached through the existing public property access path, the target uses an
  explicit offset, and the source is a direct unaliased variable name. A `null`
  public property materializes as an array, writes through the source variable
  and direct object-property array offset observe the same selected value, and
  `unset($value)` detaches only the source name. This is not existing alias
  groups, source names already routed through array-offset aliases, `$GLOBALS`,
  object-property append targets, untested deeper object-property reference
  paths, dynamic/magic/non-public properties, `ArrayAccess` reference targets,
  non-direct sources, full PHP reference containers, copy-on-write, exact alias
  rebinding/mutation ordering, or native lowering.

## Milestone 1072: Reference/COW Continuation

- [x] Runtime/value-model lane: add bounded direct public object-property
  array-append and deeper explicit reference-target coverage for unaliased
  direct variable sources. In the current subset, `$object->items[] =& $value;`
  and `$object->groups[$key][] =& $value;` work when the target object is a
  direct object variable, the property is declared public and reached through
  the existing public property access path, every parent offset is explicit,
  and the source is a direct unaliased variable name. The same milestone also
  covers the tested deeper explicit path
  `$object->groups[$outer][$inner] =& $value;`. A `null` public property and
  missing parent containers materialize as arrays, append targets use the
  runtime array append cursor, writes through the source variable and direct
  object-property array offset observe the same selected value, and
  `unset($value)` detaches only the source name. This is not existing alias
  groups, source names already routed through array-offset aliases, `$GLOBALS`,
  dynamic/magic/non-public properties, `ArrayAccess` reference targets,
  non-direct sources, full PHP reference containers, copy-on-write, exact alias
  rebinding/mutation ordering, or native lowering.

## Milestone 1073: Reference/COW Continuation

- [x] Runtime/value-model lane: add stable direct and property-held
  `ArrayAccess` reference-target boundaries for unaliased direct variable
  sources. In the current subset, `$bag[$key] =& $value;` and
  `$holder->bag[$key] =& $value;` detect object roots implementing
  `ArrayAccess` and report a specific structured runtime diagnostic. This
  keeps the project aligned with PHP's fatal behavior for assigning by
  reference to an object array dimension without trying to byte-match
  engine-specific fatal text. The CLI fixtures are intentionally `phpc-only`.
  This is not dynamic/magic/non-public property support, `$GLOBALS` reference
  target semantics, non-direct sources, full PHP reference containers,
  copy-on-write, exact alias rebinding/mutation ordering, by-reference
  `ArrayAccess::offsetGet()` indirect-modification fidelity, or native
  lowering.

## Milestone 1074: Reference/COW Continuation

- [x] Runtime/value-model lane: add a bounded direct-array by-reference
  `foreach` mutation-fidelity slice. In the current subset,
  `foreach ($items as $key => &$item)` over a direct array variable routes the
  loop value variable to the active direct array slot for the body and advances
  against the current ordered array, so appended elements/new tail entries are
  visited and direct writes to the current slot are visible through the loop
  variable. This is not full PHP reference containers, copy-on-write,
  non-direct iterables, object/`Traversable` iteration, foreach destructuring,
  array/object/`ArrayAccess` offset loop variables, exact
  removed-and-reinserted current-slot reference identity, broad array
  reordering/replacement semantics, or native lowering.

## Milestone 1075: Reference/COW Continuation

- [x] Runtime/value-model lane: add a bounded direct-array by-reference
  `foreach` current-slot unset/reinsert slice. In the current subset, unsetting
  the active direct array slot detaches the loop value variable onto the
  removed value, so same-key reinsertion during that body does not retarget the
  value variable until a later iteration reaches the reinserted tail entry.
  This is not full PHP reference containers, copy-on-write, non-direct
  iterables, object/`Traversable` iteration, foreach destructuring,
  array/object/`ArrayAccess` offset loop variables, nested-offset loop values,
  broad array reordering/replacement semantics, or native lowering.

## Milestone 1076: Reference/COW Continuation

- [x] Runtime/value-model lane: add a bounded direct string-keyed `$GLOBALS`
  reference-target slice for direct unaliased variable sources. In the current
  subset, `$GLOBALS["name"] =& $value;` binds the named root global symbol to
  the source variable cell at top level and from function scope. Writes through
  the source variable, direct global variable, and `$GLOBALS` offset observe
  the same value, and `unset($value)` detaches only the source name. This is
  not non-string keys, append/nested `$GLOBALS` reference targets, non-direct
  sources, recursive `$GLOBALS` materialization, full PHP reference
  containers, copy-on-write, exact alias rebinding/mutation ordering, or
  native lowering.

## Milestone 1077: Reference/COW Continuation

- [x] Runtime/value-model lane: add bounded nested/append string-keyed
  `$GLOBALS` reference-target slices for direct unaliased variable sources. In
  the current subset, `$GLOBALS["bag"]["slot"] =& $value;` and
  `$GLOBALS["list"][] =& $value;` bind the selected root-global array slot to
  the source variable cell when the first `$GLOBALS` key is a string. Missing
  root globals, `null` root globals, and missing intermediate containers
  materialize as arrays; append targets use the runtime array append cursor.
  Supported nested `$GLOBALS` by-value writes route through the root global
  table. This is not non-string root keys, `$GLOBALS[] =& $value`, non-direct
  sources, source names already routed through array-offset aliases, recursive
  `$GLOBALS` materialization, full PHP reference containers, copy-on-write,
  exact alias rebinding/mutation ordering, or native lowering.

## Milestone 1078: Reference/COW Continuation

- [x] Runtime/value-model lane: add a bounded by-reference `foreach`
  temporary-array expression slice. In the current subset,
  `foreach ([...] as &$value)` and `foreach (function_returning_array() as
  &$value)` evaluate array literals or direct non-reference-returning function
  call results, store the temporary array in an internal hidden array slot,
  route the loop value variable to each active temporary slot during the body,
  and preserve PHP's post-loop lingering-reference behavior for the last
  temporary slot. This is not nested lvalue iterable support such as
  `$items[0]`, reference-returning call iterables, object/`Traversable`
  iteration, foreach destructuring, array/object/`ArrayAccess` offset loop
  variables, full PHP reference containers, copy-on-write, exact mutation
  ordering, or native lowering.

## Milestone 1089: Public Object-Property Reference Sources

- [x] Runtime/value-model lane: add bounded direct public object-property
  reference-source aliases. `$alias =& $object->property;` now aliases a
  direct variable target to a named declared public property on a direct
  object variable, including scalar and array property values. Whole-property
  assignment preserves whole-property aliases while still detaching narrower
  array-offset aliases into the previous property array. This is not dynamic
  property source aliasing, magic `__get` by-reference behavior, non-public
  property source aliasing, non-direct object expressions, non-variable
  reference targets, ArrayAccess reference sources, full PHP reference
  containers, copy-on-write containers, exact alias destruction ordering, or
  native lowering.
- [ ] Runtime/value-model lane: choose the next reference/COW gap from
  remaining copied reference slots, remaining by-reference `foreach` fidelity,
  array/object copy-on-write split behavior, dynamic/magic/non-public property
  reference targets, remaining `$GLOBALS` reference semantics, by-reference
  `ArrayAccess::offsetGet()` indirect-modification fidelity, or native
  lowering boundaries, and add the next bounded behavior or explicit
  diagnostic with PHP comparison coverage where applicable.
- [ ] WordPress entry-flow lane: choose the next real entry blocker from a
  stricter XML-RPC request-body trace, REST/front controller request state,
  cron/request/SAPI fidelity, or a stricter admin/AJAX trace, and keep the
  probe documented as an external measurement unless a normalized fixture is
  added.

## Milestone 1088: `$GLOBALS` Append Reference Sources

- [x] Runtime/value-model lane: add bounded string-keyed `$GLOBALS` append
  array-offset reference sources. `$alias =& $GLOBALS["bag"][];` and
  `$alias =& $GLOBALS["bag"]["outer"][];` parse and execute when the alias
  target is a direct variable and the root global key is a string. Missing
  root globals and parent containers materialize as arrays, the selected
  appended slot starts as `null`, and writes through the alias, direct global
  variable path, or supported `$GLOBALS` path observe the same value. This is
  not `$GLOBALS[]` append-source support, non-string root key support,
  recursive `$GLOBALS` materialization, dynamic global names, non-variable
  reference targets, ArrayAccess reference sources, full PHP reference
  containers, copy-on-write containers, exact alias destruction ordering, or
  native lowering.

## Milestone 1087: Append Reference Sources

- [x] Runtime/value-model lane: add bounded append array-offset reference
  sources for direct arrays and direct public object-property arrays.
  `$alias =& $array[];`, `$alias =& $array[$outer][];`,
  `$alias =& $object->items[];`, and
  `$alias =& $object->items[$outer][];` parse and execute when the alias
  target is a direct variable, the parent path is explicit, and the
  object-property root is a named declared public property on a direct object
  variable. Missing roots and parent containers materialize as arrays, the
  selected appended slot starts as `null`, and writes through the alias or
  selected source path observe the same value. This is not `$GLOBALS` append
  reference-source support, dynamic/magic/non-public property reference
  sources, non-direct object expressions, non-variable reference targets,
  ArrayAccess reference sources, full PHP reference containers, copy-on-write
  containers, exact alias destruction ordering, or native lowering.

## Milestone 1086: Nested Reference Sources

- [x] Runtime/value-model lane: add bounded nested array-offset reference
  sources for direct arrays and direct public object-property arrays.
  `$alias =& $array[$outer][$inner];` and
  `$alias =& $object->items[$outer][$inner];` parse and execute when the alias
  target is a direct variable, every offset is explicit, and the object-property
  root is a named declared public property on a direct object variable. Missing
  path containers and selected slots materialize through existing alias-root
  metadata. This is not append-at-depth reference-source support,
  dynamic/magic/non-public property reference sources, non-direct object
  expressions, non-variable reference targets, ArrayAccess reference sources,
  full PHP reference containers, copy-on-write containers, exact alias
  destruction ordering, or native lowering.

## Milestone 1085: Object-Property Reference Sources

- [x] Runtime/value-model lane: add a bounded direct object-property
  array-offset reference-source slice. `$alias =& $object->items[$key];`
  parses and executes when the target is a direct variable, the object source
  is a direct variable, the property is a named declared public property, and
  the offset is explicit. Missing selected slots and `null` properties
  materialize through the existing public-property alias-root metadata. This
  is not nested object-property source support, dynamic/magic/non-public
  property reference sources, non-direct object expressions, non-variable
  reference targets, ArrayAccess reference sources, full PHP reference
  containers, copy-on-write containers, exact alias destruction ordering, or
  native lowering.

## Milestone 1084: Object-Property Copied Reference Slots

- [x] Runtime/value-model lane: add a bounded object-property array-copy
  reference-element slice. When a declared public object property array has a
  covered direct object-property array-offset reference target and is copied
  into a direct variable, the copied slot joins the same bounded alias group.
  Plain object-property arrays without reference elements still copy by value,
  and whole-property or whole-object-variable assignment drops stale
  property-root aliases. This is not object-property reference-source support,
  dynamic/magic/non-public property reference targets, arbitrary nested copied
  reference slots, ArrayAccess references, full PHP reference containers,
  copy-on-write containers, exact alias destruction ordering, or native
  lowering.

## Milestone 1083: WordPress XML-RPC Request Body Placeholder

- [x] Runtime/builtin lane: add a bounded `file_get_contents('php://input')`
  empty request-body placeholder for the reached WordPress XML-RPC CLI probe,
  while keeping local filesystem reads, other stream wrappers, contexts,
  offsets/lengths, and native lowering unsupported.

## Latest Completed Checkpoint

- The latest committed checkpoint is
  `14addaea runtime: add public property reference sources`, covering
  Milestone 1089. The serialized checkpoint gate passed with 1294 fixture
  tests, 739
  system PHP comparisons, and 555 skipped PHP comparisons.

## Milestone 1090: Dynamic Public Property Reference Sources

- [x] Runtime/value-model lane: add a bounded dynamic public object-property
  reference-source slice. `$alias =& $object->$property;` parses and executes
  when the alias target is a direct variable, the source object is a direct
  variable, and the evaluated property name is a string or integer public
  property name. Existing declared public properties and existing dynamic
  public properties alias through the same public-property root route, and
  allowed dynamic-property objects such as `stdClass` materialize a missing
  selected property as `null` before binding. This is not dynamic-property
  source support for non-direct object expressions, missing dynamic properties
  on classes that do not allow dynamic public slots, magic `__get`
  by-reference behavior, non-public property source aliases, non-variable
  reference targets, ArrayAccess reference sources, full PHP reference
  containers, copy-on-write containers, exact alias destruction ordering, or
  native lowering.

## Milestone 1091: Non-Public Property Reference Sources

- [x] Runtime/value-model lane: add a bounded named non-public
  object-property reference-source slice for active method visibility
  contexts. `$alias =& $this->privateProperty;`,
  `$alias =& $this->protectedProperty;`, and protected peer-object sources
  such as `$alias =& $other->protectedProperty;` parse and execute when the
  alias target is a direct variable, the source object is a direct variable,
  and the selected property is visible under the current
  public/private/protected method context. The alias uses context-aware
  object-property root metadata, so writes through either the alias or visible
  property path observe the same value. This is not dynamic non-public
  property source support, non-public property array-offset source support,
  non-direct object expressions, inaccessible private/protected property
  magic `__get` fallback from outside context, magic `__get` by-reference
  behavior, non-variable reference targets, ArrayAccess reference sources,
  full PHP reference containers, copy-on-write containers, exact alias
  destruction ordering, or native lowering.

## Milestone 1092: ArrayAccess Reference-Source Boundary

- [x] Runtime/value-model lane: add a stable `ArrayAccess` reference-source
  boundary. `$alias =& $bag[$key];` and
  `$alias =& $holder->bag[$key];` now report a specific unsupported-call
  diagnostic when the source root is an `ArrayAccess` object, covering direct,
  nested, and append source shapes rooted at a direct object variable or a
  public object property holding an `ArrayAccess` object. This is not
  by-reference `ArrayAccess::offsetGet()` aliasing, by-value `offsetGet()`
  indirect-modification notice fidelity, missing-key materialization through
  `offsetGet()`, append-source `offsetGet(null)` behavior,
  magic/property-provided ArrayAccess containers, full PHP reference
  containers, copy-on-write containers, exact warning/fatal wording, or native
  lowering.

## Milestone 1093: Magic `__get` Reference Sources

- [x] Runtime/value-model lane: add a bounded magic `__get`
  reference-source slice. `$alias =& $object->missing;` and
  `$alias =& $object->$property;` parse and execute when the alias target is a
  direct variable, the source object is a direct variable, the selected
  property is undefined, a visible non-static `__get()` method exists, that
  method is declared by reference, and its body returns a direct variable
  through the existing reference-return subset. The alias binds to the
  returned variable cell, so writes through the alias and returned variable
  observe the same value. This is not non-reference-returning `__get()`
  support, `__get()` property/array-offset/expression reference-return
  support, inaccessible private/protected property magic fallback fidelity
  from outside context, non-direct object expression support, dynamic
  non-public magic-property behavior, full PHP reference containers,
  copy-on-write containers, exact notices, or native lowering.

## Milestone 1094: Clone Reference-Slot Mirroring

- [x] Runtime/value-model lane: add bounded public-property
  reference-slot mirroring for direct clone assignments. `$copy = clone
  $object;` mirrors existing public object-property and public object-property
  array-offset alias metadata from the direct source object variable to the
  direct clone target, so writes through the alias, original property path, or
  clone property path observe the same bounded reference slot. This is not
  clone alias mirroring for non-direct clone expressions, non-public property
  aliases, magic-property aliases, ArrayAccess references, declared `__clone`
  dispatch, private/protected clone-method visibility behavior, full PHP
  reference containers, copy-on-write containers, exact alias destruction
  ordering, or native lowering.

## Milestone 1095: Promoted Property Parameter Boundary

- [x] Parser/syntax-boundary lane: add a stable parse diagnostic for
  unsupported promoted constructor property parameters such as
  `public string $name`, `private $id`, and
  `protected readonly string $name` in parameter lists. This keeps constructor
  property promotion out of the AST/runtime until property declaration,
  constructor initialization ordering, typed/readonly property enforcement,
  reflection metadata, exact PHP diagnostics, and native lowering are
  implemented.

## Milestone 1096: Native Reference-Assignment Boundary

- [x] IR/lowering lane: give statement-form `=&` a dedicated native codegen
  rejection boundary for `phpc compile --emit-ir` and `--emit-asm`, separate
  from the broader mutation boundary. The rejection fires before lowering
  direct variable, array-offset, object-property, function-call, method-call,
  static-call, magic `__get`, or `ArrayAccess` reference sources or targets.
  This is not native reference containers, alias-aware symbol tables,
  copy-on-write, object/property alias roots, source-operand lowering,
  backend execution for `=&`, or exact native PHP diagnostics.

## Milestone 1097: Fixture Runner Summary Contract

- [x] Compiler-output/CLI lane: add deterministic `phpc test --compare-php`
  CLI coverage proving that compared fixtures and sibling `.phpc-only`
  opt-outs are both reflected in the summary counts. The test uses a fake
  `php` binary on `PATH` so the contract is independent of the host PHP
  installation. This does not change fixture execution semantics, system PHP
  comparison behavior, comparison normalization, PHP-version-specific
  diagnostics, or runtime/native support.

## Milestone 1098: Context Clone Reference-Slot Mirroring

- [x] Runtime/value-model lane: extend bounded direct clone assignment
  reference-slot mirroring to context-aware object-property aliases created
  through a valid method visibility context. `$copy = clone $object;` now
  mirrors `ContextObjectProperty` alias roots, preserving the stored
  private/protected access context, so writes through the alias, original
  non-public property path, or cloned non-public property path observe the same
  bounded reference slot inside the covered method-context slice. This is not
  clone alias mirroring for non-direct clone expressions, non-public property
  array-offset aliases, dynamic non-public properties, magic-property aliases,
  ArrayAccess references, declared `__clone` dispatch, full PHP reference
  containers, copy-on-write containers, exact alias destruction ordering, or
  native lowering.

## Milestone 1099: Nullsafe Object Operator Boundary

- [x] Parser/syntax-boundary lane: add a stable parse diagnostic for
  unsupported PHP 8 nullsafe object access such as `$user?->name` and
  `$user?->profile()`. This keeps null-aware property/method chaining out of
  the AST/runtime until short-circuit evaluation, mixed `->`/`?->` chain
  ordering, call argument evaluation behavior, assignment-target restrictions,
  exact PHP diagnostics, and native lowering are implemented.

## Milestone 1100: Native Clone Boundary

- [x] IR/lowering lane: give `clone` expressions a dedicated native codegen
  rejection boundary for `phpc compile --emit-ir` and `--emit-asm`, separate
  from the broader object/class boundary. This is not native object handles,
  property-slot cloning, `__clone` dispatch, clone-method visibility,
  reference-slot metadata, full references, copy-on-write, backend execution,
  or exact native PHP diagnostics.

## Milestone 1101: Compile Mode Validation Contract

- [x] Compiler-output/CLI lane: validate unsupported `phpc compile` emit modes
  before input IO or parsing, with a deterministic CLI snapshot for
  `--emit-object` on a missing input path. This is not object-file emission,
  new backends, backend fallback recovery, parser/runtime behavior, or native
  lowering support.

## Milestone 1102: Non-Public Property Array-Offset Reference Sources

- [x] Runtime/value-model lane: add a bounded non-public
  object-property array-offset reference-source slice for active method
  visibility contexts. `$alias =& $this->privateItems[$key];`,
  `$alias =& $this->protectedItems[$key];`, and protected peer-object sources
  such as `$alias =& $other->items[$key];` parse and execute when the alias
  target is a direct variable, the source object is a direct variable, the
  property name is explicit, the offset path is explicit, and the selected
  property is visible under the current public/private/protected method
  context. Missing selected slots and `null` property roots materialize the
  same way as the existing visible public-property route. This is not dynamic
  non-public property source support, non-public append-source support,
  non-public property array-offset clone alias mirroring, non-direct object
  expressions, inaccessible private/protected property magic fallback from
  outside context, ArrayAccess reference sources, full PHP reference
  containers, copy-on-write containers, exact alias destruction ordering, or
  native lowering.

## Milestone 1103: DNF Type Declaration Boundary

- [x] Parser/syntax-boundary lane: add a stable parse diagnostic for
  unsupported parenthesized DNF-shaped type declarations such as `(A&B)|C` in
  parameter, return, instance property, and static property declaration
  positions. This is not DNF type metadata, runtime type enforcement,
  coercion, variance, reflection, typed property storage, or native lowering.

## Milestone 1104: Native ArrayAccess Boundary

- [x] IR/lowering lane: add a dedicated native codegen rejection boundary for
  property-held `ArrayAccess` object-offset read, write, `isset`, `empty`,
  `unset`, and compound/increment paths in `phpc compile --emit-ir` and
  `--emit-asm`. The diagnostic names missing native ArrayAccess dispatch
  through `offsetGet`, `offsetSet`, `offsetExists`, and `offsetUnset`, object
  handles, references/copy-on-write, and exact PHP diagnostics. This does not
  implement native ArrayAccess execution or infer direct `$bag[$key]` roots as
  objects when the syntax is indistinguishable from ordinary arrays.

## Milestone 1105: Compatibility Gap Map Refresh

- [x] Tests/docs/roadmap lane: reconcile the remaining PHP and WordPress
  compatibility work into `GOAL.MD` as an audit-friendly gap map. The map
  covers full references/copy-on-write, object semantics, native lowering,
  standard library/extensions, database/PDO/MySQLi realism,
  filesystem/streams, request/SAPI/server state, Composer/autoload/multifile
  behavior, WordPress entry flows, and verification gates. This is a roadmap
  and documentation milestone only; it does not claim PHP or WordPress
  compatibility and does not change implementation, fixtures, or support
  status.

## Milestone 1106: Next Parser Boundary

- [x] Parser lane: add a stable parse diagnostic for unsupported readonly
  property declarations, including untyped, typed, static, and reordered
  modifier forms such as `public readonly string $id` and
  `readonly public string $id`. This keeps readonly property metadata,
  initialization rules, write-once enforcement, reflection behavior, and
  native lowering unsupported while replacing the previous broad class-member
  modifier diagnostic with a property-specific boundary.

## Milestone 1107: Non-Public Property Append Reference Sources

- [x] Runtime/value-model lane: extend the bounded object-property append
  reference-source path to named non-public properties in active method
  visibility contexts. `$alias =& $this->privateItems[];`,
  `$alias =& $this->protectedItems[];`, and protected peer-object append
  sources such as `$alias =& $other->items[];` parse and execute when the
  alias target is a direct variable, the source object is a direct variable,
  the property name is explicit, every parent offset for append-at-depth is
  explicit, and the selected property is visible under the current
  public/private/protected method context. `null` property roots and missing
  parent containers materialize as arrays before binding the appended `null`
  slot. This is not dynamic non-public append-source support,
  dynamic/magic-property append-source support, non-public property
  array-offset clone alias mirroring, non-direct object expressions,
  inaccessible private/protected property magic fallback from outside
  context, ArrayAccess reference sources, full PHP reference containers,
  copy-on-write containers, exact alias destruction ordering, or native
  lowering.

## Milestone 1108: Native Include/Require Expression Boundary

- [x] IR/lowering lane: add a dedicated native rejection for expression-form
  `include`, `include_once`, `require`, and `require_once`, naming include
  return values, `_once` de-duplication results, caller-scope side effects,
  and multi-file execution instead of falling through to the statement-form
  multi-file diagnostic.

## Milestone 1109: Next Compiler-Output Contract

- [x] Compiler-output lane: strengthen the deterministic
  `phpc test --compare-php` summary contract by splitting skipped comparison
  fixtures into missing-system-`php` and sibling `.phpc-only` reasons. This
  changes only CLI reporting and test-runner accounting; it does not change
  fixture execution semantics, comparison normalization, PHP-version-specific
  diagnostics, runtime behavior, native lowering, or PHP support claims.

## Milestone 1110: Lane Queue Refresh

- [x] Tests/docs lane: after Milestones 1106-1109 landed, refresh
  `GOAL.MD`, `docs/LANE_WORKERS.md`, `docs/NEXT_TASKS.md`,
  `docs/PROGRESS.md`, and affected support docs so the next batch has one
  active milestone per lane and a clear full-gate checkpoint point. This is a
  planning/documentation milestone only; it does not change runtime behavior,
  native lowering, fixtures, or PHP/WordPress support claims.

## Milestone 1111: Next Parser Boundary

- [x] Parser lane: add a stable parse diagnostic for unsupported readonly
  class declarations. `readonly class Value { ... }`, `final readonly class
  Value {}`, and `readonly final class Value {}` now fail before class metadata
  registration with a diagnostic naming the missing readonly class metadata,
  typed-property enforcement, initialization/write rules, reflection, and
  native lowering. Duplicate `readonly readonly class` declarations still use
  the existing duplicate-modifier diagnostic.

## Milestone 1112: Next Runtime Value/Object Slice

- [x] Runtime lane: add bounded clone/reference-slot mirroring for
  context-aware non-public object-property array-offset aliases created inside
  valid method visibility contexts. The covered slice includes private
  `$this->items[$key]`, private appended slots, and protected peer-object
  slots while preserving public clone mirroring and the non-public reference
  source behavior from the prior lanes. Dynamic non-public clone mirroring,
  magic-property clone mirroring, non-direct object expressions, ArrayAccess
  reference containers, full PHP reference containers, copy-on-write, exact
  alias destruction ordering, and native lowering remain unsupported.

## Milestone 1113: Next Native Boundary

- [x] IR/lowering lane: add a dedicated native termination rejection for
  `exit()` and `die()` calls. `phpc compile --emit-ir` and `--emit-asm` now
  reject those constructs before generic function-call lowering with a
  diagnostic naming termination control flow, exit status/stdout handoff,
  shutdown functions, destructors/finally ordering, output buffers, SAPI
  interaction, and exact native diagnostics.

## Milestone 1114: Next Compiler-Output Contract

- [x] Compiler-output lane: add `phpc test --list-fixtures [fixture-dir]`, a
  deterministic audit-only fixture manifest. The command prints sorted fixture
  paths, recognized expectation files, and PHP-comparison eligibility without
  parsing, executing, or comparing fixtures.

## Milestone 1115: Next Tests/Docs Queue Refresh

- [x] Tests/docs lane: after Milestones 1111-1114 land, refresh the lane
  queue, progress log, support docs, and compatibility-gap notes, then run the
  serialized full gate before checkpointing.

## Milestone 1116: Next Parser Boundary

- [x] Parser lane: add a stable parse diagnostic for unsupported non-property
  readonly class members such as `readonly function id() {}`, `public readonly
  function id() {}`, and `readonly const ID = 1;`. Existing readonly property
  and readonly class diagnostics remain unchanged. This is not readonly
  method support, readonly class constant support, readonly runtime
  enforcement, reflection behavior, or native lowering.

## Milestone 1117: Next Runtime Value/Object Slice

- [x] Runtime lane: add bounded final class inheritance enforcement. Declared
  `final class Base {}` still registers and instantiates in the current object
  model, while `class Child extends Base {}` reports a stable runtime boundary
  when `Base` is final. This preserves abstract-class instantiation
  rejection, inherited property behavior, and nested class registration
  timing. Final method override enforcement, abstract method implementation
  enforcement, readonly semantics, autoload-triggered discovery, exact PHP
  `Error` objects/diagnostics, and native lowering remain unsupported.

## Milestone 1118: Next Native Boundary

- [x] IR/lowering lane: add a dedicated native rejection for `global`
  declarations. `phpc compile --emit-ir` and `--emit-asm` now reject global
  declarations with a diagnostic naming root symbol-table imports,
  local/global aliasing, `$GLOBALS` interactions, references/copy-on-write,
  included-file scope interactions, and exact native diagnostics instead of
  the previous broad ad hoc message.

## Milestone 1119: Next Compiler-Output Contract

- [x] Compiler-output lane: refine `phpc test --list-fixtures` with
  deterministic aggregate totals for PHP-comparison eligible fixtures,
  `.phpc-only` fixtures, and recognized `.stdout`, `.stderr`, `.exit`, and
  `.phpc-only` sidecars. This does not change fixture execution or
  `--compare-php` behavior.

## Milestone 1120: Next Tests/Docs Queue Refresh

- [x] Tests/docs lane: after Milestones 1116-1119 land, refresh the lane
  queue, progress log, support docs, and compatibility-gap notes, then run the
  serialized full gate before checkpointing.

## Milestone 1121: Next Parser Boundary

- [x] Parser lane: add stable parse diagnostics for unsupported
  `abstract`/`final` non-method class members. Properties now report an
  abstract/final property boundary, while class constants report an
  abstract/final class-constant boundary. Supported abstract/final methods,
  duplicate modifier diagnostics, abstract-final method combination
  diagnostics, readonly boundaries, and typed-property diagnostics remain
  preserved.

## Milestone 1122: Next Runtime Value/Object Slice

- [x] Runtime lane: add bounded final method override enforcement. Inherited
  final methods still execute, while a child method declaration with the same
  case-insensitive name reports a stable runtime boundary during class
  registration. This preserves final class inheritance enforcement,
  abstract-class instantiation rejection, inherited property behavior, and
  nested class registration timing. Abstract method implementation
  enforcement, trait composition, interface enforcement, method visibility
  compatibility, exact PHP `Error` objects/diagnostics, and native lowering
  remain unsupported.

## Milestone 1123: Next Native Boundary

- [x] IR/lowering lane: add a dedicated native rejection for function and
  method `static` local declarations. `phpc compile --emit-ir` and
  `--emit-asm` now reject static locals with a diagnostic naming persistent
  per-function storage, initialization ordering, local scope interaction,
  references/copy-on-write, recursion, and exact native diagnostics instead of
  the broader function-declaration boundary.

## Milestone 1124: Next Compiler-Output Contract

- [x] Compiler-output lane: refine `phpc test --list-fixtures` with
  deterministic recognized orphan sidecar reporting. The manifest now reports
  sidecars with `.stdout`, `.stderr`, `.exit`, or `.phpc-only` extensions that
  do not have a matching `.php` fixture without changing normal `phpc test`
  execution or `--compare-php` behavior.

## Milestone 1125: Next Tests/Docs Queue Refresh

- [x] Tests/docs lane: after Milestones 1121-1124 land, refresh the lane
  queue, progress log, support docs, and compatibility-gap notes, then run the
  serialized full gate before checkpointing.

## Milestone 1126: Next Parser Boundary

- [x] Parser lane: add a stable parse boundary for unsupported PHP asymmetric
  property visibility, such as `public private(set) string $id;`. The parser
  now reports the missing property visibility metadata, typed-property
  storage/enforcement, reflection behavior, and native lowering instead of a
  misleading duplicate-visibility diagnostic.

## Milestone 1127: Next Runtime Value/Object Slice

- [x] Runtime lane: add bounded abstract method implementation enforcement
  for concrete classes. Concrete classes that declare or inherit abstract
  methods without a concrete implementation now report a stable runtime
  boundary during class registration, while abstract child classes can defer
  implementation and concrete descendants can satisfy the obligation. Method
  visibility compatibility, method signature compatibility, trait/interface
  enforcement, exact PHP `Error` objects/diagnostics/exit parity, and native
  object/class lowering remain unsupported.

## Milestone 1128: Next Native Boundary

- [x] IR/lowering lane: add a dedicated native dynamic function-call
  rejection. `phpc compile --emit-ir` and `--emit-asm` now reject variable
  calls such as `$name(...)` with a diagnostic naming callable expression
  evaluation, runtime function lookup, stack frames, arity/type diagnostics,
  callback dispatch, and exact native callable errors.

## Milestone 1129: Next Compiler-Output Contract

- [x] Compiler-output lane: add `phpc test --list-fixtures-json
  [fixture-dir]`, a deterministic machine-readable fixture manifest. The JSON
  contract reports the same sorted fixtures, expectation metadata,
  PHP-comparison eligibility, aggregate counts, and orphan sidecars as the
  text manifest without parsing, executing, or comparing fixtures.

## Milestone 1130: Next Tests/Docs Queue Refresh

- [x] Tests/docs lane: after Milestones 1126-1129 land, refresh the lane
  queue, progress log, support docs, and compatibility-gap notes, then run the
  serialized full gate before checkpointing.

## Milestone 1131: Next Parser Boundary

- [x] Parser lane: add a stable grouped `use` declaration parse boundary for
  forms such as `use App\{Controller, Service};` and `use {App\Controller};`.
  Grouped class, function, and const imports, import metadata expansion,
  namespace-aware native lowering, and exact PHP diagnostics remain
  unsupported.

## Milestone 1132: Next Runtime Value/Object Slice

- [x] Runtime lane: add bounded inherited method visibility compatibility
  enforcement. Child classes may keep or widen inherited non-private method
  visibility, while reductions such as public-to-protected report a stable
  runtime boundary. Method signature compatibility, static/non-static method
  compatibility, trait/interface enforcement, exact PHP diagnostics, and
  native object/class lowering remain unsupported.

## Milestone 1133: Next Native Boundary

- [x] IR/lowering lane: add a dedicated native error-control rejection.
  `phpc compile --emit-ir` and `--emit-asm` now reject `@expr` with a
  diagnostic naming diagnostic severity, warning/notice/deprecation
  suppression, `error_reporting()` mask interaction, recoverable expression
  values, and exact native diagnostics instead of the broader unary/cast
  boundary.

## Milestone 1134: Next Compiler-Output Contract

- [x] Compiler-output lane: extend the JSON fixture-manifest contract to
  version 2 with sorted `compatibility_targets` summaries for
  `compat/<target>` directories, including targets with no executable `.php`
  fixtures yet.

## Milestone 1135: Next Tests/Docs Queue Refresh

- [x] Tests/docs lane: after Milestones 1131-1134 land, refresh the lane
  queue, progress log, support docs, and compatibility-gap notes, then run the
  serialized full gate before checkpointing.

## Milestone 1136: Next Parser Boundary

- [x] Parser lane: choose the next small unsupported syntax or
  parse-diagnostic boundary from the refreshed full-compatibility gap map.
  Prefer a PHP/WordPress surface that still falls through to a broad or
  misleading diagnostic. Add stable focused coverage, CLI fixture evidence
  where applicable, and keep runtime/native support claims unchanged.
  Milestone 1136 adds a PHP property-hook parse boundary for
  `public string $name { get => ...; }` and related get/set hook shapes,
  naming the missing hook metadata, backing/virtual property behavior,
  typed-property storage/enforcement, references, reflection, and native
  lowering.

## Milestone 1137: Next Runtime Value/Object Slice

- [x] Runtime lane: add bounded inherited method static/non-static
  compatibility enforcement. Child methods that redeclare inherited
  non-private methods must keep the inherited staticness, while static-to-
  instance and instance-to-static changes report a stable runtime boundary
  during class registration. Method signature compatibility, trait/interface
  enforcement, exact PHP diagnostics, and native object/class lowering remain
  unsupported.

## Milestone 1138: Next Native Boundary

- [x] IR/lowering lane: choose one precise native rejection or tiny lowering
  refinement from interpreter behavior that is already documented. `phpc
  compile --emit-ir` and `--emit-asm` must either lower the exact supported
  slice or reject it before misleading backend output.
  Milestone 1138 adds a dedicated native cast-expression rejection for
  interpreter-supported cast syntax until native scalar conversion, array
  materialization, warning/recovery behavior, object/resource handling,
  references/copy-on-write, and exact diagnostics exist.

## Milestone 1139: Next Compiler-Output Contract

- [x] Compiler-output lane: add an audit-only `phpc test --php-versions-json`
  contract that reports configured PHP comparison binaries from
  `PHPC_PHP_BINARIES` or default `php`, parsed versions, project-tracked PHP
  8.2-8.5 branch coverage, and missing tracked branches without running
  fixtures or broadening PHP support claims.

## Milestone 1140: Next Tests/Docs Queue Refresh

- [x] Tests/docs lane: after Milestones 1136-1139 land, refresh the lane
  queue, progress log, support docs, and compatibility-gap notes, then run the
  serialized full gate before checkpointing.

## Milestone 1141: Next Parser Boundary

- [x] Parser lane: choose the next small unsupported syntax or
  parse-diagnostic boundary from the refreshed full-compatibility gap map.
  Prefer a PHP/WordPress surface that still falls through to a broad or
  misleading diagnostic. Add stable focused coverage, CLI fixture evidence
  where applicable, and keep runtime/native support claims unchanged.
  Milestone 1141 refines unsupported `declare(...)` parse diagnostics so
  `declare(ticks=1);` names missing tick handlers/execution hooks and
  `declare(encoding="UTF-8");` names missing source encoding, lexer decoding,
  and runtime text handling, while preserving the existing `strict_types`
  diagnostic and leaving declare behavior unimplemented.

## Milestone 1142: Runtime Method Signature Compatibility Slice

- [x] Runtime lane: enforce the bounded inherited method required-parameter
  compatibility rule. Child methods other than `__construct` that redeclare
  inherited non-private methods may keep the same required count or add
  optional parameters, while adding required parameters reports a stable
  class-registration boundary. Private parent redeclarations remain separate.
  Full PHP signature variance, type compatibility, return-type compatibility,
  named arguments, trait/interface enforcement, and exact PHP `Error` objects
  remain unsupported.

## Milestone 1143: Dedicated Native Method-Call Rejection

- [x] IR/lowering lane: add a dedicated native method-call rejection for
  instance, named static, object/static-receiver, `self::`, `parent::`, and
  late-static `static::` calls before receiver or argument lowering. The
  diagnostic names missing native method lookup, receiver/static receiver
  resolution, `$this` and late-static-binding context, argument/arity
  diagnostics, visibility, references/copy-on-write, and exact native
  method-call errors without changing interpreter behavior.

## Milestone 1144: Next Compiler-Output Contract

- [x] Compiler-output lane: add an audit-only `phpc test --compare-php-json
  [fixture-dir]` contract that runs the existing optional system-PHP
  comparison path and emits deterministic aggregate fixture pass/fail,
  compared, skipped, missing-system-`php`, and `.phpc-only` counts. This does
  not broaden PHP support claims, normalize PHP-version-specific diagnostics,
  replace committed expectations, or prove branch-specific compatibility.

## Milestone 1145: Next Tests/Docs Queue Refresh

- [x] Tests/docs lane: after Milestones 1141-1144 land, refresh the lane
  queue, progress log, support docs, and compatibility-gap notes, then run the
  serialized full gate before checkpointing.

## Milestone 1146: Next Parser Boundary

- [x] Parser lane: refine unsupported namespace import forms beyond simple
  class imports. `use function App\Demo\fn_name;` and
  `use const App\Demo\VALUE as Alias;` now have dedicated parse diagnostics
  naming missing function/constant import metadata, namespace-aware
  function/constant lookup, alias handling, fallback lookup, and native
  lowering. Simple class `use` behavior remains supported, grouped-use
  diagnostics remain the grouped boundary, and runtime/native support claims
  are unchanged.

## Milestone 1147: Runtime Interface Method Presence Slice

- [x] Runtime lane: enforce public method presence for concrete classes that
  implement declared user interfaces, including inherited `implements`
  metadata from abstract parents. Keep the slice bounded to method-name/public
  presence only, with tests, milestone fixtures, docs, and explicit gaps for
  interface inheritance, parameter/return type compatibility, built-in/internal
  interface method enforcement, traits, exact PHP `Error` objects, autoload,
  and native lowering.

## Milestone 1148: Native Object-Property Boundary

- [x] IR/lowering lane: add a dedicated native object-property rejection for
  documented interpreter property behavior. `phpc compile --emit-ir` and
  `--emit-asm` now reject instance property reads/writes and dynamic
  property-name access before misleading backend output, while class,
  metadata, method-call, clone, and ArrayAccess boundaries remain separate.

## Milestone 1149: Next Compiler-Output Contract

- [x] Compiler-output lane: extend `phpc test --list-fixtures-json
  [fixture-dir]` to `contract_version` 3 with deterministic
  `phpc_only_reason` text read from sibling `.phpc-only` marker files. This
  improves comparison opt-out auditability without changing normal fixture
  execution, `--compare-php` behavior, compatibility-target counts, or PHP
  support claims.

## Milestone 1150: Next Tests/Docs Queue Refresh

- [x] Tests/docs lane: after Milestones 1146-1149 land, refresh the lane
  queue, progress log, support docs, and compatibility-gap notes, then run the
  serialized full gate before checkpointing.

## Milestone 1151: Next Parser Boundary

- [ ] Parser lane: choose the next small unsupported syntax or
  parse-diagnostic boundary from the refreshed full-compatibility gap map.
  Prefer a PHP/WordPress surface that still falls through to a broad or
  misleading diagnostic. Add stable focused coverage, CLI fixture evidence
  where applicable, and keep runtime/native support claims unchanged.

## Milestone 1152: Next Runtime Value/Object Slice

- [ ] Runtime lane: choose one bounded runtime slice from the refreshed gap
  map, preferably a remaining reference/COW, object-semantics, request-state,
  filesystem, database, or WordPress-probe blocker already reached by fixtures
  or external probes. Prove it with focused tests, CLI coverage, system PHP
  comparison where applicable, and named unsupported edges.

## Milestone 1153: Next Native Boundary

- [ ] IR/lowering lane: choose one precise native rejection or tiny lowering
  refinement from interpreter behavior that is already documented. `phpc
  compile --emit-ir` and `--emit-asm` must either lower the exact supported
  slice or reject it before misleading backend output.

## Milestone 1154: Next Compiler-Output Contract

- [ ] Compiler-output lane: choose one deterministic CLI, fixture-runner,
  compatibility-manifest, or backend artifact contract that improves
  auditability without broadening PHP support claims.

## Milestone 1155: Next Tests/Docs Queue Refresh

- [ ] Tests/docs lane: after Milestones 1151-1154 land, refresh the lane
  queue, progress log, support docs, and compatibility-gap notes, then run the
  serialized full gate before checkpointing.

## Tests/Docs Lane: Parallel Worker Operations

- [x] Document the lane/subagent worktree protocol, focused-test command shape,
  lane ownership boundaries, and handoff requirements so parser, IR/lowering,
  runtime, compiler-output, and tests/docs workers can advance separate
  milestones without sharing one cargo target directory or overwriting active
  implementation slices.
