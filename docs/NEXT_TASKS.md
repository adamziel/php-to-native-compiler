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

- [ ] Parser lane: choose the next small syntax or parse-diagnostic boundary
  from the documented unsupported gaps, add parser/unit coverage plus
  `phpc run` CLI snapshots where applicable, and do not widen runtime or native
  support claims unless another lane implements and tests the behavior.

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

- [ ] IR/lowering/compiler-output lane: choose the next honest native runtime
  integration slice: target-data-layout-aware helper signatures, boxed scalar
  construction in generated LLVM, a linker command prototype that still rejects
  executable mode clearly, or a documented blocker if the current LLVM text
  backend cannot model C ABI helper calls safely.

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
  Milestone 755 parses `abstract`, `final`, and `readonly` class modifiers plus
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
- [ ] Runtime/regex lane: widen bounded `preg_match()` for the reached
  WordPress startup pattern delimiter shape while keeping broad PCRE syntax,
  callbacks, flags/offsets beyond the existing slice, exact warnings, and
  native lowering named unless implemented.

## Latest Checkpoint

- Before the current Milestone 809 checkpoint, the latest committed checkpoint
  is `0e56bc7 runtime: add bounded mysqli select db`, covering Milestone 808.

## Tests/Docs Lane: Parallel Worker Operations

- [x] Document the lane/subagent worktree protocol, focused-test command shape,
  lane ownership boundaries, and handoff requirements so parser, IR/lowering,
  runtime, compiler-output, and tests/docs workers can advance separate
  milestones without sharing one cargo target directory or overwriting active
  implementation slices.
