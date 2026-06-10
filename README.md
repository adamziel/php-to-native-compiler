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
- Top-level user functions include scoped `__FUNCTION__` and `__METHOD__`
  magic-constant coverage plus `func_num_args()`, `func_get_arg()`, and
  `func_get_args()` call-frame introspection, scalar type hints, and
  by-reference return aliases for variables, one-level array elements, local
  lifetimes, typed coercion, and plain-assignment separation.
- Direct variable references, array element references, and by-reference
  userland parameters cover the first COW/reference boundary slice.
- `count()` handles arrays and raises catchable `TypeError` diagnostics for
  non-array operands in the current boxed value domain.
- Bounded PHPT telemetry from `/home/claude/php-src-phpt`.

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
PHPC_BIN="$PWD/target/debug/phpc" php /home/claude/php-src-phpt/run-tests.php -q -p "$PWD/target/debug/phpc" <manifest paths>
```

Detailed history lives in beads, commits, and merge requests, not in this file.
