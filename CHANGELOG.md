# Changelog

## Unreleased

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
