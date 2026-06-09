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
non-direct and reference-aware compound assignment, variable variables,
undefined-variable warning parity, functions, classes, resources, exceptions,
and dynamic fallback.

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

Implemented statement-form `print` for the current supported expression subset:

- Lexer/parser/AST support for `print expr;` as a statement.
- IR lowering maps print statements to the same boxed output instruction used by
  echo, so generated C calls the existing `ptn_echo` conversion/output helper.
- Native tests prove print with literals, variables, and binary expressions.

Still unsupported for print: expression contexts such as
`$result = print "value";`, `echo print "value";`, return value `1` semantics,
and parenthesized `print(...)` syntax.

Rebased the lexer/parser source-compatibility slice for comments and PHP tags
onto the print-enabled tree:

- PHP `//`, `#`, and `/* ... */` comments are skipped while preserving source
  span progression for following tokens.
- A Unix shebang is accepted only at the start of a file before `<?php`.
- The parser requires the supported `<?php` code envelope and accepts a
  trailing `?>` close tag when only whitespace follows it.
- Native tests prove comments, shebang, trailing close tags, echo, and
  statement-form print together through the compiled binary path without
  changing expression semantics.

Still unsupported for PHP tag handling: inline HTML before `<?php`, between PHP
blocks, or after a close tag; short open tags; doc comment retention; and full
PHP/HTML mode switching.

Implemented a narrow boxed unary/cast expression slice in the CAO worktree:

- Lexer support for `-`, `!`, `(`, `)`, and scalar cast keywords.
- Parser/AST support for parenthesized expression grouping, unary `-`, unary
  `!`, and `(int)`, `(float)`, `(string)`, and `(bool)` casts.
- IR value-expression operation nodes for unary operations and scalar casts.
- Generated C code that materializes operands once before calling boxed runtime
  helpers.
- Boxed runtime helpers for scalar numeric negation, PHP scalar truthiness
  negation, and scalar casts using the existing number/string conversion
  helpers.
- Native tests proving grouped negation, truthiness negation, scalar casts, and
  interaction with binary `+` and `.` expressions.

Still unsupported for unary/casts: arrays, objects, references, copy-on-write,
full numeric-string diagnostic parity, unsupported operand `TypeError` parity,
and exact overflow behavior for scalar casts.

Implemented a direct named-variable compound-assignment slice on top of the
unary/casts-enabled head:

- Lexer/parser/AST support for `$x += expr` and `$x .= expr` statements.
- IR lowering that rewrites compound assignment as a direct variable read, the
  existing boxed `+` or `.` operation, then a direct variable write.
- Native tests proving scalar add/concat compound assignment with print output,
  RHS grouping/casts through the same expression path, source-order
  undefined-variable diagnostics for the LHS read before the RHS read, and the
  unsupported boundary for other compound operators and non-direct lvalues.

Still unsupported for compound assignment: `-=`, `*=`, `/=`, `%=`, `**=`, `&=`,
`|=`, `^=`, `<<=`, `>>=`, `??=`, array/object/string-offset/property/static
property/variable-variable lvalues, references, reference identity, and
copy-on-write interactions.
