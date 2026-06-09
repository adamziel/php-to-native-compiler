# Architecture

PTN starts with a conservative native backend while preserving the architecture
needed for full PHP compatibility.

Current pipeline:

```text
PHP source
-> lexer tokens with source spans
-> AST
-> PHP-aware IR
-> generated C with boxed-value runtime
-> native executable via system C compiler
```

The generated C backend is the first native-code path, not the final backend.
It lets the project exercise binary production from day one while the runtime
model grows toward full PHP semantics.

Current runtime/compiler slices:

- The lexer recognizes the supported PHP code envelope: optional byte-zero
  Unix shebang, required `<?php`, PHP comments inside the code region, and one
  `?>` close tag that switches to inline output through EOF. Inline HTML before
  `<?php` and multi-block PHP/HTML mode switching remain unsupported.
- Direct variables lower to generated C `PtnRuntime` symbol-table load/store
  calls.
- Direct variable reads pass through a runtime helper that emits a generic
  undefined-variable warning before yielding `null`.
- Scalar binary `+`, `-`, `*`, `/`, `%`, and `.` expressions lower to IR
  value-expression operation nodes. The C backend materializes operands into
  `PtnValue` temporaries in source order before calling boxed runtime helpers
  such as `ptn_add`, `ptn_subtract`, `ptn_multiply`, `ptn_divide`,
  `ptn_modulo`, and `ptn_concat`.
- Direct named-variable `+=`, `-=`, `*=`, `/=`, `%=`, and `.=` lower in IR as a
  direct variable load, the same boxed binary helper used by the ordinary
  binary operator, and a direct variable store. This keeps left-to-right reads
  and undefined-variable diagnostics on the runtime read boundary rather than
  adding a separate compound-assignment runtime path.
- Statement-form `print expr;` lowers to the same boxed output IR instruction
  used by echo, so generated native code routes print output through the
  existing `ptn_echo` helper.
- Parenthesized expressions are parsed as grouping, while unary `+`, unary `-`,
  unary `!`, and scalar `(int)`, `(float)`, `(string)`, and `(bool)` casts lower to IR
  value-expression operation nodes. The C backend emits boxed runtime helper
  calls such as `ptn_positive`, `ptn_negate`, `ptn_not`, and `ptn_cast_*`.
- Increment/decrement tokens are rejected while PHP assignment-style
  pre/post-increment semantics are unsupported, so spellings such as `--$value`
  cannot be mistaken for two unary negations.
- Scalar comparison and boolean expressions share the same AST/IR binary node
  shape. Comparisons emit boxed booleans through runtime helpers, while `&&`
  and `||` emit native C branches that short-circuit over boxed PHP truthiness.
  The ordered comparison helpers share `ptn_compare_order`, so `<`, `<=`, `>`,
  and `>=` use one scalar ordering path.
- Simple statement-form calls lower to IR internal-call instructions carrying a
  normalized function name and lowered arguments. The generated C backend
  materializes arguments left-to-right and dispatches through a small internal
  function registry. `var_dump` is currently the only registered internal
  function, and it formats the boxed scalar runtime values directly.
- Braced `if`, `elseif`, and `else` statements lower to structured IR branch
  instructions. Conditions remain boxed value expressions, and the C backend
  emits native branches that call the shared scalar truthiness helper.
- `elseif` is represented as an else branch containing another structured
  branch, so future exception edges, temporaries, destructor timing,
  references, and copy-on-write behavior can stay attached to the statement
  tree.
- Braced `while` statements lower to structured IR loop instructions. The C
  backend evaluates the boxed condition at the top of each iteration and uses
  the shared scalar truthiness helper before emitting the loop body.
- Statement-form direct variable increment/decrement lowers to a runtime read,
  boxed numeric increment/decrement helper, and runtime write. Expression-value
  semantics for pre/post increment remain outside this slice.

Near-term architecture targets:

- PHP ordered arrays.
- References and copy-on-write.
- Function and class metadata.
- Broader diagnostics and exception channels.
- Full PHP numeric-string conversions, non-numeric string arithmetic
  diagnostics, warnings, scalar cast overflow behavior, exact
  division/modulo-by-zero exception behavior, and complete overflow behavior for
  arithmetic helpers.
- Array, object, and reference lvalues for compound assignment, plus
  unsupported compound operators beyond `+=`, `-=`, `*=`, `/=`, `%=`, and `.=`:
  `**=`, `&=`, `|=`, `^=`, `<<=`, `>>=`, `??=`.
- Complete comparison parity for arrays, objects, references, chained
  comparison parse errors, identity/spaceship operators, keyword boolean
  operators, and unsupported scalar edge cases.
- A broader internal-function module system with shared argument parsing,
  metadata, unsupported array/object/resource/reference diagnostics, and
  PHP-exact `var_dump` precision/formatting behavior.
- Broader control flow: unbraced and alternate syntax, `do while`, `for`,
  `foreach`, `switch`, `break`, `continue`, and exception/finally edges.
- Full PHP increment/decrement semantics, including expression result values,
  strings, booleans, arrays/objects, references, and copy-on-write behavior.
- Explicit fallback boundaries for `eval`, variable variables, and runtime
  symbol mutation.
