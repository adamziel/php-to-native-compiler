# PTN From Scratch

PTN compiles PHP into native binaries through a generic compiler/runtime path:

`PHP source -> lexer/parser -> AST -> IR -> C runtime -> native executable`

Rule: implement reusable PHP semantics. Do not special-case PHPT filenames,
expected rows, or one-off outputs.

## Current Priority

Copy-on-write is the only implementation focus. Other implementation work
allowed: 0, unless it directly unblocks COW correctness or COW evidence.

## Current Counts

Last refresh: 2026-06-09T16:56Z
Commit: `70e7254bcae0`

| Format / source | Ported | Passing | Failing | Needed |
| --- | ---: | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 | 0 |
| Native PHP snippets | 273 | 273 | 0 | 0 |
| COW-adjacent native | 1 | 1 | 0 | 5 |
| PHPT bounded total | 200 | 138 | 62 | 62 |
| PHPT Zend | 76 | 60 | 16 | 16 |
| PHPT ext/standard | 77 | 46 | 31 | 31 |
| PHPT tests/* | 47 | 32 | 15 | 15 |

## COW Blockers

5: payload refcounts, detach-on-write, nested container cloning, by-value
`foreach` mutation visibility, function-boundary value separation.

## Current Runtime

Rust crate, `phpc`, boxed C runtime, native integration tests, scoped
`__FUNCTION__`/`__METHOD__`, and catchable `count()` non-array diagnostics.

## Status Files

- `PROGRESS.md`: compact test and porting dashboard.
- `STATUS.md`: current operating status.
- `progress.md`, `progress.html`, `STATUS.html`: short generated mirrors.

## Commands

```bash
cargo test
cargo build --bin phpc
PHPC_BIN="$PWD/target/debug/phpc" php /home/claude/php-src-phpt/run-tests.php -q -p "$PWD/target/debug/phpc" <manifest paths>
```
