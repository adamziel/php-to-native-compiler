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

- [ ] Add explicit `phpc compile --emit-asm` CLI coverage for a fallback
  backend command that passes discovery but becomes non-executable before
  actual assembly emission, proving the stable fallback-backend start
  diagnostic is reported for permission-denied emission starts without
  silently falling through to later fallbacks, with deterministic test
  doubles, documentation, and named gaps for backend race conditions, bundled
  toolchains, exact native error objects, and broader native lowering.
