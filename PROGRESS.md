# Progress

## 2026-06-08

Restarted the project from scratch under `~/ptn-from-scratch`, guided by
`NEW_PROMPT.md`.

Integrated first vertical slice:

- Rust crate `ptn`.
- Lexer and parser for a narrow PHP subset with source spans.
- AST to PHP-aware IR lowering.
- C backend with a boxed-value runtime for `null`, booleans, integers, floats,
  and strings.
- CLI path:
  `ptn compile <input.php> -o <native-binary> [--emit-c]`.
- Native binary exercise test that compiles PHP `echo` into a C-backed native
  executable and runs it.

Unsupported edges are intentionally named rather than hidden: variables, arrays,
functions, classes, includes, references, copy-on-write, resources, exceptions,
and dynamic fallback are not implemented yet.

Next integrated production target:

- Add variables and assignment using a real symbol-table/runtime model that can
  scale to PHP references and copy-on-write.

## 2026-06-09

Recovered the checkpoint worktree onto current `origin/master` at
`ca130c503622ec9a479318d294bfd64d20e496a3` after stale pre-restart bundles
conflicted with the new from-scratch tree.

Integrated the next production slice:

- Lexer support for direct PHP variable tokens such as `$name`.
- Parser/AST support for direct assignment statements and variable reads in
  expressions.
- IR store/load instructions for named variables.
- Generated C runtime symbol table storing boxed `PtnValue` slots by name.
- Native tests proving assignment, reads, and overwrites in compiled binaries.

Added a runtime diagnostics boundary for direct variable reads:

- Generated native binaries route direct variable reads through `PtnRuntime`
  rather than treating the symbol table as an ordinary nullable map.
- Undefined direct variable reads emit a generic runtime warning and then yield
  `null`, so current echo behavior remains PHP-like without hiding the
  diagnostic boundary.
- Native tests prove defined and undefined reads in the same compiled binary.

Still unsupported: arrays, references, copy-on-write, globals/superglobals,
compound assignment, variable variables, undefined-variable warning parity,
functions, classes, resources, exceptions, and dynamic fallback.

Added a non-harness differential telemetry path:

- `tools/diff-native-output.sh` compiles a file, snippet, or stdin through
  `ptn compile`, runs the native binary, runs the same input with system `php`,
  and compares stdout, stderr, and exit status.
- This does not claim PHPT compatibility. Direct PHPT execution remains blocked
  until PTN grows a PHP-compatible runner interface instead of only
  `ptn compile`.

Implemented the next boxed expression slice in the CAO worktree:

- Lexer support for `+` and `.` operator tokens.
- Parser/AST support for left-associative binary expressions with `+` binding
  tighter than string concatenation.
- IR value-expression operation nodes for numeric addition and string
  concatenation.
- Generated C code that materializes operands left-to-right into `PtnValue`
  temporaries before calling `ptn_add` and `ptn_concat`.
- Boxed runtime helpers for scalar numeric addition and scalar string
  concatenation, including basic scalar-to-number and scalar-to-string
  conversions.
- Native tests proving literal operations, variables, assignment results,
  chained expressions, numeric-string/float addition basics, and observable
  left-to-right operand evaluation in compiled binaries.

Still unsupported for binary operations: full PHP numeric-string conversion
warning parity, non-numeric string arithmetic diagnostics, complete overflow
parity, arrays, objects, references, and copy-on-write behavior.
