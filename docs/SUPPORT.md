# Support

This file tracks user-visible PHP behavior that the scratch compiler currently
supports in generated native binaries.

## Supported

- `<?php` open tag.
- A Unix shebang at byte 0 before `<?php`.
- PHP `//`, `#`, and `/* ... */` comments inside PHP code. One-line
  comments end at a newline or at a trailing `?>` close tag.
- A trailing `?>` close tag when only whitespace follows it.
- `echo` statements.
- Statement-form `print expr;` for the same scalar expression subset as echo.
- String, integer, float, boolean, and null literals.
- Direct variable assignment and scalar reads through the generated runtime
  symbol table.
- Undefined direct variable reads emit a generic runtime warning and then yield
  `null`.
- Boxed scalar `+` numeric addition and `.` string concatenation. Chained
  expressions are parsed left-associatively, with `+` binding tighter than `.`.
- Binary operands are materialized left-to-right before the generated C backend
  calls runtime helpers.
- Direct named-variable compound assignment for `+=` and `.=`. The compiler
  lowers `$x += expr` as `read $x`, boxed `+`, then `write $x`; `$x .= expr`
  lowers the same way through boxed string concatenation. The direct variable
  read happens before the right-hand expression, so existing undefined-variable
  diagnostics remain observable in source order.
- Print statements use the same generated boxed output path as echo.
- Parenthesized expressions for grouping supported scalar expressions,
  including nested grouping.
- Unary `-` over boxed scalar numeric values.
- Unary `!` using boxed PHP scalar truthiness: `null`, `false`, numeric zero,
  `0.0`, `""`, and `"0"` are falsey; other supported scalar values are truthy.
- Scalar `(int)`, `(float)`, `(string)`, and `(bool)` casts over supported boxed
  scalar values.
- Boxed scalar comparison operators `==`, `!=`, `<`, `<=`, `>`, and `>=`.
- Boxed scalar boolean operators `&&` and `||`, with short-circuit evaluation
  over PHP truthiness for the currently supported scalar values.

## Not Yet Supported

- PHP-exact diagnostic formatting, file names, line numbers, error handlers, and
  error reporting configuration.
- Full PHP numeric-string conversion warning parity, non-numeric string
  arithmetic diagnostics, complete overflow parity, and exact scalar cast
  overflow behavior.
- Prefix and postfix increment/decrement operators such as `++$value` and
  `--$value`.
- `print` as an expression returning `1`, including contexts such as assignment,
  echo operands, binary operands, and the parenthesized spelling `print(...)`.
- Comparison operators `===`, `!==`, `<=>`, keyword boolean operators
  `and`/`or`, PHP-exact chained comparison parse errors, and complete
  comparison parity for unsupported value types.
- Inline HTML before `<?php`, between PHP blocks, or after a closing PHP tag.
- Doc comment retention for reflection or metadata. Comments are skipped today.
- Compound assignment operators other than `+=` and `.=`: `-=`, `*=`, `/=`,
  `%=`, `**=`, `&=`, `|=`, `^=`, `<<=`, `>>=`, and `??=`.
- Array, object, string-offset, property, static-property, variable-variable,
  reference, and other non-direct-variable compound-assignment lvalues.
- Reference semantics for compound assignment, including reference identity,
  copy-on-write interactions, and by-reference visibility during writes.
- Arrays, references, copy-on-write, globals, superglobals, functions, classes,
  objects, resources, exceptions, variable variables, includes, and dynamic
  fallback.
