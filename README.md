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
- Double-quoted interpolation covers direct `$name`, simple `$items[$key]`
  offsets, braced `{$name}`/`{$items['key']}`, and deprecated legacy
  `${name}` variables.
- Double-quoted string escapes cover common control escapes plus `\xNN` and
  octal byte escapes, including high bytes in generated native binaries.
- Top-level user functions and declared class methods include scoped magic
  constant coverage for their current supported scope plus `func_num_args()`,
  `func_get_arg()`, and `func_get_args()` call-frame introspection, scalar type
  hints, and by-reference return aliases for variables, one-level array
  elements, local lifetimes, typed coercion, plain-assignment separation, and
  declared public constructor dispatch through the method path.
- Direct variable references, array element references, and by-reference
  userland parameters cover the first COW/reference boundary slice.
- Arithmetic rejects non-numeric strings and mixed array operands with modeled
  catchable `TypeError` diagnostics while preserving leading-numeric warnings.
- `count()` handles arrays and raises catchable `TypeError` diagnostics for
  non-array operands in the current boxed value domain.
- `array_chunk()` and `array_combine()` build fresh ordered arrays through the
  shared array runtime.
- `array_filter()` preserves keys while filtering arrays by PHP truthiness or
  modeled callbacks.
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

## RC Demo

```bash
cargo build --bin phpc
for f in examples/rc/*.php; do
  echo "== $f =="
  target/debug/phpc "$f"
done
```

The corpus exercises the current RC surface: scalar control flow, arrays and
internal functions, user functions, public class/object shells, direct static
properties, and public property `??=`.

Detailed history lives in beads, commits, and merge requests, not in this file.
