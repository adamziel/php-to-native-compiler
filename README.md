# PTN From Scratch

PTN compiles a growing subset of PHP into native binaries through a generic
compiler/runtime path:

`PHP source -> lexer/parser -> AST -> IR -> C runtime -> native executable`

Rule: implement reusable PHP semantics. Do not special-case PHPT filenames,
expected rows, or one-off outputs.

## Current Shape

- Rust crate and `phpc` compiler binary.
- Boxed C runtime for PHP-like values.
- Native execution tests for parser, IR, backend, runtime, and selected PHP
  behavior.
- Double-quoted interpolation covers direct `$name`, braced `{$name}`, and
  braced variable-root array offsets such as `{$items['key']}`.
- Double-quoted string escapes cover common control escapes plus `\xNN` and
  octal byte escapes, including high bytes in generated native binaries.
- Top-level user functions and declared class methods include scoped magic
  constant coverage for their current supported scope plus `func_num_args()`,
  `func_get_arg()`, and `func_get_args()` call-frame introspection, scalar type
  hints, and by-reference return aliases for variables, one-level array
  elements, local lifetimes, typed coercion, and plain-assignment separation.
- Direct variable references, array element references, and by-reference
  userland parameters cover the first COW/reference boundary slice.
- `count()` handles arrays and raises catchable `TypeError` diagnostics for
  non-array operands in the current boxed value domain.
- `range()` builds integer ranges through the ordered-array runtime.
- Bounded PHPT telemetry from a php-src checkout resolved by `PHP_SRC_PHPT`,
  `/home/claude/php-src-phpt`, or the `.runtime/php-src-phpt` cache.

## Status Files

- `PROGRESS.md`: compact test and porting dashboard.
- `STATUS.md`: current operating status.
- `progress.md`, `progress.html`, `STATUS.html`: short generated mirrors.

These files must stay under 500 words each. One progress patrol polecat refreshes
them about every 10 minutes.

## Commands

```bash
cargo test
tools/run-native-smoke-matrix.sh
tools/run-post-merge-cow-gate.sh
cargo build --bin phpc
tools/run-phpt-manifest.sh tools/phpt-manifest-200.txt
```

Detailed history lives in beads, commits, and merge requests, not in this file.
