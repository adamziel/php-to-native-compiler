# Support

This file tracks user-visible PHP behavior that the scratch compiler currently
supports in generated native binaries.

## Supported

- `<?php` open tag.
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
- Print statements use the same generated boxed output path as echo.

## Not Yet Supported

- PHP-exact diagnostic formatting, file names, line numbers, error handlers, and
  error reporting configuration.
- Full PHP numeric-string conversion warning parity, non-numeric string
  arithmetic diagnostics, and complete overflow parity.
- `print` as an expression returning `1`, including contexts such as assignment,
  echo operands, binary operands, and parenthesized `print(...)` syntax.
- Arrays, references, copy-on-write, globals, superglobals, compound assignment,
  functions, classes, objects, resources, exceptions, variable variables,
  includes, and dynamic fallback.
