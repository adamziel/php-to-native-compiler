# Support

This file tracks user-visible PHP behavior that the scratch compiler currently
supports in generated native binaries.

## Supported

- `<?php` open tag.
- A Unix shebang at byte 0 before `<?php`.
- PHP `//`, `#`, and `/* ... */` comments inside PHP code. One-line
  comments end at a newline or at a trailing `?>` close tag.
- A `?>` close tag that ends PHP mode and emits following inline output, with
  one immediately following newline swallowed.
- `echo` statements.
- Statement-form `print expr;` for the same scalar expression subset as echo.
- String, integer, float, boolean, and null literals.
- Direct variable assignment and scalar reads through the generated runtime
  symbol table.
- Undefined direct variable reads emit a generic runtime warning and then yield
  `null`.
- Boxed scalar `+`, `-`, `*`, `/`, and `%` numeric arithmetic and `.` string
  concatenation. Chained expressions are parsed left-associatively, with `*`,
  `/`, and `%` binding tighter than `+` and `-`, and arithmetic binding tighter
  than `.`.
- Binary operands are materialized left-to-right before the generated C backend
  calls runtime helpers.
- Direct named-variable compound assignment for `+=`, `-=`, `*=`, `/=`, `%=`,
  `.=`, `&=`, and `|=`. The compiler lowers these as `read $x`, the matching
  boxed binary helper, then `write $x`. The direct variable read happens before
  the right-hand expression, so existing undefined-variable diagnostics remain
  observable in source order.
- Print statements use the same generated boxed output path as echo.
- Parenthesized expressions for grouping supported scalar expressions,
  including nested grouping.
- Unary `+` over boxed scalar numeric values.
- Unary `-` over boxed scalar numeric values.
- Unary `!` using boxed PHP scalar truthiness: `null`, `false`, numeric zero,
  `0.0`, `""`, and `"0"` are falsey; other supported scalar values are truthy.
- Scalar `(int)`, `(float)`, `(string)`, and `(bool)` casts over supported boxed
  scalar values.
- Boxed scalar comparison operators `==`, `!=`, `===`, `!==`, `<`, `<=`, `>`,
  and `>=`. Strict scalar identity compares type and value without coercion.
- Boxed scalar boolean operators `&&` and `||`, with short-circuit evaluation
  over PHP truthiness for the currently supported scalar values.
- Boxed scalar bitwise `&` and `|` operators. When both operands are strings,
  the result is a bytewise string for non-NUL string data. Other supported
  scalar operands are converted to integers through the current scalar numeric
  conversion path.
- Simple statement-form internal calls such as `var_dump(expr, ...)` and
  `strlen(expr);`.
- Expression-form internal calls for the currently registered scalar functions,
  including `strlen(expr)` in echo operands, assignments, binary operands, and
  branch/loop conditions.
- Internal-call arguments are materialized left-to-right before generated C
  runtime dispatch.
- `var_dump()` output for current boxed scalar values: `NULL`, `bool(...)`,
  `int(...)`, `float(...)`, and `string(length) "value"`.
- `strlen()` over current boxed scalar values after scalar string conversion.
- A minimal `phpc` runner for supported PHPT rows. It compiles scripts or `-r`
  snippets to temporary native binaries through the normal compiler pipeline.
- Braced `if`, `elseif`, and `else` statements whose conditions and bodies use
  the currently supported scalar expression and statement subset.
- Braced `while (expr) { statements }` loops where the condition and body use
  the currently supported scalar expression and statement subset.
- Braced `do { statements } while (expr);` loops where the body and condition
  use the currently supported scalar expression and statement subset. The body
  executes once before the first condition check.
- Statement-form direct variable increment/decrement: `$name++;`, `++$name;`,
  `$name--;`, and `--$name;`.

## Not Yet Supported

- PHP-exact diagnostic formatting, file names, line numbers, error handlers, and
  error reporting configuration.
- Full PHP numeric-string conversion warning parity, non-numeric string
  arithmetic diagnostics, exact division/modulo-by-zero exception behavior,
  complete overflow parity, and exact scalar cast overflow behavior.
- Prefix and postfix increment/decrement operators such as `++$value` and
  `--$value`.
- `print` as an expression returning `1`, including contexts such as assignment,
  echo operands, binary operands, and the parenthesized spelling `print(...)`.
- Comparison operator `<=>`, keyword boolean operators `and`/`or`, PHP-exact
  chained comparison parse errors, and complete comparison parity for
  unsupported value types.
- Unbraced control-flow statements, alternate control-flow syntax, `for`,
  `foreach`, `switch`, `break`, `continue`, branch-condition assignments, and
  exception/finally control-flow edges.
- Increment/decrement as expressions, including pre/post result values in echo,
  assignment, binary operands, function arguments, or branch conditions.
- PHP-exact increment/decrement semantics for strings, booleans, arrays,
  objects, references, copy-on-write, overflow edge cases, and diagnostics.
- Inline HTML before `<?php` or between PHP blocks.
- Internal functions other than `var_dump()` and `strlen()`.
- Arrays, objects, resources, recursive structures, references, and
  `var_dump()` reference identity output.
- Embedded NUL strings in runtime string values, `var_dump()` string
  length/output, or `strlen()`.
- Full PHP float precision and formatting edge cases for `var_dump()` or
  `strlen()` input conversion.
- Complete PHP CLI and PHPT runner option parity for `phpc`.
- Doc comment retention for reflection or metadata. Comments are skipped today.
- Bitwise `^`, unary bitwise `~`, and bit shifts.
- Compound assignment operators other than `+=`, `-=`, `*=`, `/=`, `%=`, `.=`,
  `&=`, and `|=`: `**=`, `^=`, `<<=`, `>>=`, and `??=`.
- Array, object, string-offset, property, static-property, variable-variable,
  reference, and other non-direct-variable compound-assignment lvalues.
- Reference semantics for compound assignment, including reference identity,
  copy-on-write interactions, and by-reference visibility during writes.
- Arrays, references, copy-on-write, globals, superglobals, functions, classes,
  objects, resources, exceptions, variable variables, includes, and dynamic
  fallback.
