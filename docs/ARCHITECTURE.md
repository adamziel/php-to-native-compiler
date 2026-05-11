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
The interpreter runs top-level statements in a global symbol table and creates a
fresh local symbol table for each user-function call; importing globals into
function scope through `global` declarations is not implemented.

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
- variable variables use materialized symbol tables; current variable-variable
  syntax is rejected with an explicit diagnostic before execution
- dynamic includes use runtime include resolution
- `eval` parses and executes in the caller scope

Executable semantics for those features are not implemented yet.

## Include/Require Resolution Design

`include`, `include_once`, `require`, and `require_once` are reserved by the
lexer/parser today and rejected with stable parse diagnostics before execution.
The first executable include/require slice should use these rules:

- the interpreter carries the current file path, process working directory, and
  include stack in runtime execution context
- only paths that evaluate to PHP strings are accepted at first
- absolute paths resolve directly
- relative paths resolve against the directory of the file containing the
  include/require expression
- `include_once` and `require_once` de-duplicate by canonical absolute path when
  the filesystem can canonicalize the target, and by normalized absolute path
  otherwise
- included files execute in the caller scope and may return a value through
  PHP's `return` statement
- native lowering rejects include/require until file loading, scope effects,
  and return-value behavior have explicit lowering support

Initial unsupported include/require behavior remains: `include_path` lookup,
current-working-directory fallback, stream wrappers, `phar://`, URL includes,
autoload interaction, opcache behavior, cycle detection beyond `_once`
de-duplication, and PHP's warning-vs-fatal recovery details.

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
