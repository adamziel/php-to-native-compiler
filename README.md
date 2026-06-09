# PTN From Scratch

PTN compiles a growing subset of PHP into native binaries through a generic
compiler/runtime path:

`PHP source -> lexer/parser -> AST -> IR -> C runtime -> native executable`

Rule: implement reusable PHP semantics. Do not special-case PHPT filenames,
expected rows, or one-off outputs.

## Current Shape

- Rust crate and `phpc` compiler binary.
- Boxed C runtime for PHP-like values.
- Ordered arrays with reads, variable-root mutation, `foreach`, key predicates,
  and bounded variable-root cursor internals: `current()`, `key()`, `reset()`,
  `end()`, `next()`, and `prev()`.
- Native execution tests for parser, IR, backend, runtime, and selected PHP
  behavior.
- Top-level user functions include scoped `__FUNCTION__` and `__METHOD__`
  magic-constant coverage.
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
cargo build --bin phpc
PHPC_BIN="$PWD/target/debug/phpc" php /home/claude/php-src-phpt/run-tests.php -q -p "$PWD/target/debug/phpc" <manifest paths>
```

Detailed history lives in beads, commits, and merge requests, not in this file.
