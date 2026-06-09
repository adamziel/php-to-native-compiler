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
and the parenthesized spelling `print(...)` in expression contexts.

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
exact overflow behavior for scalar casts, and PHP increment/decrement
operators such as `++$value` and `--$value`.

Followed up after comparison/boolean integration by rejecting unsupported `++`
and `--` operators lexically. This prevents `--$value` from being compiled as
two supported unary negations before real PHP increment/decrement assignment
semantics exist.

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

Rebased the comparison/boolean expression slice after boxed `+` and `.`
integration:

- Lexer/parser support for `==`, `!=`, `<`, `>`, `&&`, `||`, and parenthesized
  expressions.
- AST/IR binary operation variants for scalar comparisons and boolean
  short-circuit expressions.
- Generated C comparison emission that materializes operands left-to-right
  before calling boxed runtime comparison helpers.
- Generated C short-circuit emission for `&&` and `||` over boxed scalar PHP
  truthiness.
- Native tests proving scalar loose comparisons, numeric-string comparison
  basics, boolean short-circuit behavior, grouping, and left-to-right operand
  diagnostics.

Still unsupported for comparisons/booleans: `===`, `!==`, `<=>`, keyword
`and`/`or`, arrays, objects, references, copy-on-write behavior, PHP-exact
chained comparison parse errors, and complete PHP comparison parity for
unsupported value types.

Refined parenthesized expression grouping in the CAO worktree:

- Parenthesized expressions are preserved explicitly in the AST before IR
  lowering erases grouping to the inner value expression.
- Native tests prove grouped literals, grouped variable reads, grouped binary
  expressions, nested grouping, and grouped assignment right-hand sides.

Extended the boxed scalar comparison slice after the grouping-preservation head:

- Lexer/parser/AST/IR support for `<=` and `>=` at the same comparison
  precedence as `<` and `>`.
- Generated C comparison emission for `<=` and `>=` reuses the shared
  `ptn_compare_order` path through boxed runtime helpers, matching the existing
  scalar loose ordering behavior.
- Native tests prove integer, numeric-string, string, null, and boolean scalar
  `<=`/`>=` cases, plus parser coverage through grouped boolean expressions.

Still unsupported after this comparison extension: identity comparisons
`===`/`!==`, spaceship `<=>`, keyword `and`/`or`, arrays, objects, references,
copy-on-write behavior, PHP-exact chained comparison parse errors, and complete
PHP comparison parity for unsupported value types.

Added boxed scalar arithmetic operators beyond addition on the scalar
comparison-extension head:

- Lexer support for `*`, `/`, and `%` operator tokens, with binary `-` reusing
  the existing minus token from unary negation while preserving lexical
  rejection for unsupported `++` and `--`.
- Parser/AST support for left-associative binary `-`, `*`, `/`, and `%`
  expressions. Multiplicative operators bind tighter than additive operators,
  arithmetic binds tighter than string concatenation, and the existing
  comparison/boolean precedence levels remain below arithmetic.
- Parser/AST/IR support for direct named-variable `-=`, `*=`, `/=`, and `%=`
  compound assignments, lowering through the same direct load, boxed binary
  operation, and direct store path as `+=` and `.=` rather than through a
  separate runtime path.
- IR operation nodes for subtraction, multiplication, division, and modulo.
- Generated C dispatch to shared `PtnValue` runtime helpers while preserving
  left-to-right operand materialization.
- Boxed runtime helpers for scalar subtraction, multiplication, division, and
  modulo using the existing scalar numeric conversion path.
- Native tests proving literals, variables, assignment results, chained
  precedence, numeric-string arithmetic basics, modulo sign behavior, direct
  arithmetic compound assignment, and observable left-to-right operand
  evaluation in compiled binaries.

Still unsupported for arithmetic: full PHP numeric-string warning parity,
non-numeric string arithmetic diagnostics, exact division/modulo-by-zero
exception behavior, complete overflow parity, arrays, objects, references, and
copy-on-write behavior.

Still unsupported for compound assignment: `**=`, `&=`, `|=`, `^=`, `<<=`,
`>>=`, `??=`, array/object/string-offset/property/static-property/
variable-variable lvalues, references, reference identity, and copy-on-write
interactions.

Implemented a narrow internal-call/output slice for scalar `var_dump()` on top
of the grouping-aware baseline:

- Lexer/parser support for identifier tokens, simple statement-form calls such
  as `var_dump(expr, ...)`, and one close-tag inline-output segment after PHP
  mode ends.
- IR lowering for generic internal-call instructions carrying a function name
  and lowered argument values.
- Generated C internal-function dispatch with `var_dump` registered as the
  first internal function rather than special-cased source text.
- Scalar `var_dump` output for current boxed `PtnValue` types: `null`,
  booleans, integers, floats, and strings.
- A minimal `phpc` runner that can compile PHPT runner scripts/snippets through
  the existing native compiler path for supported rows.
- Native tests covering scalar `var_dump`, PHPT-shaped close-tag inline output,
  left-to-right argument evaluation, unsupported complex var_dump edges, and
  support-doc coverage for unsupported edges.

Still unsupported for `var_dump`: arrays, objects, resources, recursion,
references, embedded NUL strings, full PHP float precision/formatting edge
cases, and complete PHP CLI/PHPT runner parity.

Added braced branch control flow on top of the scalar `var_dump` head:

- Lexer/parser/AST support for braced `if`, `elseif`, and `else` statements,
  including grouped branch conditions and nested statement bodies.
- IR support for structured branch instructions whose conditions and branch
  bodies preserve statement nesting for future temporary, destructor,
  exception, reference, and copy-on-write handling.
- Generated C backend support for native branches over boxed PHP values through
  the existing runtime truthiness helper.
- Branch conditions reuse the existing scalar comparison, boolean
  short-circuit, unary, cast, and grouped-expression paths.
- Native tests prove `if`/`elseif`/`else`, nested branches, scalar truthiness,
  `var_dump` in branch bodies, and source-order condition diagnostics.
- Focused public PHPT telemetry through the minimal `phpc` runner passes
  `tests/lang/001.phpt`, `tests/lang/004.phpt`, `tests/lang/005.phpt`, and
  `tests/lang/006.phpt`.

Still unsupported for control flow: unbraced statements, alternate syntax,
loops, `switch`, `break`, `continue`, branch-condition assignments/increments,
arrays/objects/resources in truthiness, and exception/finally control-flow
edges.

Added a bounded braced loop and direct variable increment/decrement slice:

- Lexer/parser/AST support for `while (expr) { statements }` with recursive
  statement bodies.
- Parser support for statement-form direct variable increment/decrement:
  `$name++;`, `++$name;`, `$name--;`, and `--$name;`.
- IR support for structured loop instructions and direct increment/decrement
  instructions. Loop conditions remain boxed value expressions and loop bodies
  preserve nested statements.
- Generated C evaluates loop conditions at the top of each iteration and uses
  the shared boxed scalar truthiness helper before emitting the body.
- Direct increment/decrement emits a runtime variable read, boxed numeric
  helper call, and runtime variable write, matching the existing direct
  variable mutation boundary.
- Native tests prove the public simple while-loop shape, prefix/postfix
  statement increments and decrements, and loop-condition rechecks.
- Focused public PHPT telemetry through `phpc` passes
  `tests/lang/002.phpt`.

Still unsupported for loops and increment/decrement: unbraced and alternate
syntax, `do while`, `for`, `foreach`, `switch`, `break`, `continue`,
increment/decrement expression result values, PHP-exact string/boolean edge
semantics, references, copy-on-write, and exception/finally loop edges.
