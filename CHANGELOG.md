# Changelog

## Unreleased

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
  rejection. Bitwise operators, logical `xor`, references/copy-on-write side
  effects, exact native error objects, and native lowering remain unsupported.
