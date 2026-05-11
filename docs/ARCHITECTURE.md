# Architecture

## Pipeline

The intended compiler pipeline is:

```text
PHP source
-> lexer/parser
-> AST
-> semantic analysis
-> IR
-> lowering/type specialization
-> LLVM IR text
-> assembly/object/executable
-> linked runtime
```

Milestone 1 implements the lexer, parser, AST, a direct interpreter/runtime
execution path, and a narrow LLVM IR text emitter for simple straight-line code.

## Compiler Crate

`compiler/` contains:

- lexer and parser for the supported PHP subset
- AST definitions
- interpreter bridge for `phpc run`
- LLVM IR text emission for currently lowerable code
- CLI and fixture test runner

The parser is handwritten recursive descent. This keeps the early grammar easy
to audit while avoiding regex-based parsing.

## Runtime Crate

`runtime/` contains the PHP-shaped boxed value model used by the interpreter and
future generated code helper calls.

Implemented now:

- `Null`
- `Bool`
- `Int`
- `Float`
- `String`
- ordered PHP arrays with integer/string keys
- structured runtime error categories with stable diagnostic messages for the
  currently supported runtime failures
- PHP-ish echo conversion
- PHP-ish truthiness for the implemented value types
- basic arithmetic, comparison, and concatenation helpers
- key normalization for array strings that are valid decimal integers

Planned runtime values:

- objects and class metadata
- resources
- references
- copy-on-write containers

## Native Codegen

The first backend emits LLVM IR text and shells out to `clang` for assembly.
This is deliberately less work than an x86-64 backend and lets the project focus
on PHP semantics first.

Tradeoff: Milestone 1 native lowering is smaller than interpreter support. The
backend must return a codegen error for unsupported constructs rather than
pretend to compile them.

Current assembly emission order:

1. Generate LLVM IR text.
2. Use `clang` if available.
3. Use `llc` if available.
4. If no LLVM assembly tool is available, generate equivalent C for the same
   narrow lowerable subset and ask `cc -S` for assembly.

The C fallback exists only to keep `phpc compile --emit-asm` executable on
machines without LLVM tools. It must not grow into the primary backend without a
documented architecture decision.

## Dynamic Features

Dynamic PHP features will be implemented as runtime fallback zones:

- dynamic function calls use runtime lookup
- variable variables use materialized symbol tables
- dynamic includes use runtime include resolution
- `eval` parses and executes in the caller scope

None of those features are implemented yet.

## Fixture Tests

Fixture tests are stored as `.php` files with sibling `.stdout`, `.stderr`, and
`.exit` files. The runner strips one final editor newline from `.stdout` and
`.stderr` fixtures. A fixture that needs to assert an actual trailing newline
should include a blank final line.

When `phpc` intentionally differs from system PHP, a sibling `.phpc-only` marker
keeps the fixture in the normal runner while skipping optional system PHP
comparison.

## Extension Model

Zend extension loading is not an early target. Selected extensions will be
implemented as runtime modules with documented dependencies and semantic gaps.
