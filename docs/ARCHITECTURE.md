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
- class metadata and object-shape descriptors; top-level class declarations can
  register metadata, but PHP object syntax is not executable yet
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

- dynamic function calls use runtime lookup; the first implemented slice accepts
  string-valued callees that resolve to the documented callable builtin subset
  or user-defined functions
- variable variables use materialized symbol tables; current variable-variable
  syntax is rejected with an explicit diagnostic before execution
- dynamic includes will use runtime include resolution
- `eval` will parse and execute in the caller scope

Only the string-valued dynamic function lookup slice is executable today.
Variable-variable execution, include/require execution, and `eval` remain design
boundaries; direct `eval(...)` syntax is reserved and rejected with a stable
parse diagnostic. Array/object callables, method calls, first-class callable
syntax, and namespace/autoload-aware callable resolution are still outside the
implemented dynamic-call subset.

## Object/Class Metadata Boundary

The current object/class step is metadata registration, not executable PHP
object syntax. `php_runtime` has a `PhpClassTable`, stable `ClassId` handles,
class metadata, property metadata, method metadata, visibility markers, and
derived object shapes for instance-property layout. Class and method lookup are
case-insensitive, property lookup is case-sensitive, and duplicate class/member
metadata produces structured runtime errors.

`phpc run` parses top-level `class Name { ... }` declarations into that
metadata registry. The accepted member subset records public/protected/private
visibility, static flags, property names without defaults, and method names
whose parameters/bodies use the existing function parser subset. Class
declarations do not allocate objects, bind `$this`, execute methods, or expose
reflection.

`phpc run` still rejects `new` and `->` syntax with stable parse diagnostics,
and native lowering rejects class declarations explicitly. See
`docs/OBJECT_MODEL.md` for the named unsupported edge cases that must stay
rejected until object values and dispatch exist.

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

## Eval Fallback Design

`eval` is reserved by the lexer/parser today and rejected with a stable parse
diagnostic before execution. The first executable `eval` slice should use these
rules:

- parse direct `eval(<expr>)` as a special language construct, not as an
  ordinary function or dynamic callable
- require exactly one argument and evaluate that argument in the caller scope
- accept only string-valued code for the first slice
- parse the evaluated string with a dedicated eval-fragment parser entry point
  that reads a statement list without requiring a `<?php` opening tag
- execute the resulting statements against the caller's current symbol table, so
  assignments affect the same local or top-level scope that called `eval`
- let `return` inside the evaluated fragment produce the `eval(...)` expression
  value; falling off the end should produce `null`
- keep native lowering rejecting `eval` until parser re-entry, source mapping,
  caller-scope effects, and return behavior have explicit lowering support

Initial unsupported eval behavior remains: non-string eval arguments, exact
`ParseError` object semantics, source mapping for diagnostics inside evaluated
strings, functions/classes declared from evaluated code, nested eval,
include/require inside eval, references/copy-on-write interactions,
`GLOBALS`/superglobal behavior, namespaces/use declarations, opcache behavior,
and PHP's exact warning/fatal recovery details.

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
