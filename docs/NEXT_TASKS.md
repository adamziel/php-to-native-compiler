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
- [ ] Implement `array_fill_keys($keys, $value)` for the current ordered array
  value model, including integer/string key-value conversion, duplicate-key
  overwrite behavior, non-array and unsupported-key diagnostics, fixture CLI
  coverage, documentation, and explicit native-codegen rejection.
