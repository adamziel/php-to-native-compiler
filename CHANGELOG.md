# Changelog

## Unreleased

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
