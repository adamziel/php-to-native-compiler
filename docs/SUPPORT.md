# Support

This file tracks user-visible PHP behavior that the scratch compiler currently
supports in generated native binaries.

## Supported

- `<?php` open tag.
- `echo` statements.
- String, integer, float, boolean, and null literals.
- Direct variable assignment and scalar reads through the generated runtime
  symbol table.
- Undefined direct variable reads emit a generic runtime warning and then yield
  `null`.

## Not Yet Supported

- PHP-exact diagnostic formatting, file names, line numbers, error handlers, and
  error reporting configuration.
- Arrays, references, copy-on-write, globals, superglobals, compound assignment,
  functions, classes, resources, exceptions, variable variables, includes, and
  dynamic fallback.
