# Roadmap

## Milestone 1: Working Skeleton

- Rust workspace and CLI.
- Lexer/parser/AST for a meaningful PHP subset.
- Interpreter/runtime execution path for basic programs.
- Fixture test runner.
- LLVM IR text and assembly emission for a narrow native subset.

## Milestone 2: Value Model and Runtime

- Expand boxed values and coercions.
- Add structured runtime errors and more PHP comparison behavior.
- Add optional comparison mode against system PHP.

## Milestone 3: Arrays

- Ordered int/string-keyed map.
- Append, index read/write, key normalization, and simple copy-on-write.

## Milestone 4: Functions and Scopes

- Proper local/global scope handling, recursion hardening, default parameters,
  and early builtins.

## Milestone 5+: Dynamic PHP

- Materialized symbol tables.
- Includes.
- Objects/classes.
- Dynamic calls.
- `eval` runtime fallback.

## Extensions

See `docs/extensions/` for per-extension plans as they are added.

