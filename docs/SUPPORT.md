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
- Global-scope `const NAME = expr;` declarations for the currently supported
  constant-expression subset. Declared constants are visible to bare constant
  reads, `defined()`, and `constant()`. Duplicate declarations emit the modeled
  warning boundary and preserve the original value.
- `echo` statements.
- Statement-form `print expr;` for the same scalar expression subset as echo.
- String, integer, float, boolean, and null literals. Numeric literals accept
  PHP digit separators between digits; integer literals include decimal,
  legacy octal, explicit octal `0o`/`0O`, binary `0b`/`0B`, and hexadecimal
  `0x`/`0X` forms.
- Invalid legacy octal integer literals containing `8` or `9` are rejected with
  source-spanned PHP-style parse errors through `phpc`.
- Double-quoted strings with direct `$name` variable interpolation. Interpolated
  variables use the same runtime variable read, scalar string cast, and
  concatenation paths as ordinary expressions.
- Direct variable assignment and scalar reads through the generated runtime
  symbol table.
- Undefined direct variable reads emit a generic runtime warning and then yield
  `null`.
- Boxed scalar `+`, `-`, `*`, `**`, `/`, and `%` numeric arithmetic and `.`
  string concatenation. `**` is parsed right-associatively and binds with PHP's
  precedence relative to unary operators. Other arithmetic chains are parsed
  left-associatively, with `*`, `/`, and `%` binding tighter than `+` and `-`,
  and arithmetic binding tighter than `.`. Integer-only `%` conversions emit
  the current float/float-string precision-loss deprecation boundary when a
  scalar operand loses precision while converting to an integer, and leading
  numeric strings with trailing non-numeric data emit the current non-numeric
  warning boundary.
- Binary operands are materialized left-to-right before the generated C backend
  calls runtime helpers.
- Direct named-variable compound assignment for `+=`, `-=`, `*=`, `/=`, `%=`,
  `**=`, `.=`, `&=`, `|=`, `^=`, `<<=`, and `>>=`. The compiler lowers these as
  `read $x`, the matching boxed binary helper, then `write $x`. The direct
  variable read happens before the right-hand expression, so existing
  undefined-variable diagnostics remain observable in source order.
- Print statements use the same generated boxed output path as echo.
- Parenthesized expressions for grouping supported scalar expressions,
  including nested grouping.
- Unary `+` over boxed scalar numeric values.
- Unary `-` over boxed scalar numeric values.
- Unary `!` using boxed PHP scalar truthiness: `null`, `false`, numeric zero,
  `0.0`, `""`, and `"0"` are falsey; other supported scalar values are truthy.
- Unary bitwise `~` over supported boxed scalar values. String operands produce
  bytewise string results for non-NUL string data; other supported scalar
  operands are converted to integers through the current scalar numeric path,
  including the current float/float-string precision-loss deprecation boundary
  and leading-numeric-string warning boundary.
- Scalar `(int)`, `(float)`, `(string)`, `(bool)`, and deprecated
  non-canonical `(integer)`, `(double)`, `(binary)`, and `(boolean)` casts over
  supported boxed scalar values.
- Removed `(real)` cast syntax is rejected with a source-spanned PHP-style
  parse error through `phpc`.
- Removed `(unset)` cast syntax is rejected with a source-spanned PHP-style
  fatal error through `phpc`.
- Expression-context `(void)` cast syntax is rejected with a source-spanned
  PHP-style parse error through `phpc`.
- Unterminated block comments are rejected with a source-spanned PHP-style
  parse error through `phpc`.
- Global-scope magic constants `__LINE__`, `__FILE__`, `__DIR__`,
  `__FUNCTION__`, `__METHOD__`, `__CLASS__`, `__TRAIT__`, and
  `__NAMESPACE__`. Scope-dependent names currently resolve to empty strings in
  global scope.
- Short array literals `[...]` with optional scalar keys, automatic integer
  keys, integer-string key canonicalization, insertion order, and duplicate-key
  replacement in the current literal-array value subset.
- Array read expressions such as `$array[$key]`, including nested reads and
  reads from literal or grouped array expressions. Reads use the current
  ordered-array key canonicalization path; undefined keys and non-array
  containers emit a warning boundary and yield `null`.
- String offset read expressions such as `$string[$offset]` for the current
  C-string-backed scalar string subset. Integer-compatible offsets, negative
  offsets, nested reads, integer strings, numeric prefix strings with PHP-style
  illegal-offset warnings, and scalar cast warnings for `null`, booleans, and
  floats are handled by the shared offset-read helper; out-of-range reads emit
  an uninitialized-offset warning and return an empty string.
- `array_key_exists()` over current ordered-array values, using the same
  integer/string key canonicalization path as array literals and reads. `null`
  keys emit the current PHP-like deprecation boundary and canonicalize to the
  empty string.
- Boxed scalar and literal-array comparison operators `==`, `!=`, `===`, `!==`,
  `<`, `<=`, `>`, `>=`, and `<=>`. Strict array identity compares type, key
  order, key type, and value. Numeric comparisons involving `NAN` are treated
  as unordered, so equality and ordered scalar comparisons return false while
  `!=`/`!==` return true.
- Boxed scalar boolean operators `&&` and `||`, with short-circuit evaluation
  over PHP truthiness for the currently supported scalar values.
- Boxed scalar bitwise `&`, `^`, and `|` operators. When both operands are strings,
  the result is a bytewise string for non-NUL string data. Other supported
  scalar operands are converted to integers through the current scalar numeric
  conversion path, including the current float/float-string precision-loss
  deprecation boundary and leading-numeric-string warning boundary.
- Boxed scalar bit shifts `<<` and `>>`. Supported scalar operands are
  converted to integers through the current bitwise integer-conversion path,
  including the current float/float-string precision-loss deprecation boundary
  and leading-numeric-string warning boundary.
- Simple statement-form internal calls such as `var_dump(expr, ...)`,
  `strlen(expr);`, `str_rot13(expr);`, `strcmp(expr, expr);`,
  `str_contains(expr, expr);`, `str_starts_with(expr, expr);`,
  `str_ends_with(expr, expr);`, `quotemeta(expr);`,
  `chunk_split(expr[, expr[, expr]]);`, `strip_tags(expr);`,
  `md5(expr[, raw_output]);`,
  `sha1(expr[, raw_output]);`, `substr(expr, expr[, expr]);`, `bin2hex(expr);`,
  `hex2bin(expr);`, `quoted_printable_decode(expr);`, `dirname(expr);`,
  `soundex(expr);`, `ceil(expr);`, `floor(expr);`, `sqrt(expr);`,
  `fdiv(expr, expr);`, `intdiv(expr, expr);`, `bindec(expr);`,
  `hexdec(expr);`, `octdec(expr);`, `pi();`, `getrandmax();`,
  `getmypid();`, `php_sapi_name();`,
  `phpversion([extension]);`, `intval(expr);`, `chr(expr);`, `ord(expr);`,
  `is_finite(expr);`, `is_infinite(expr);`, `is_nan(expr);`, and
  `error_reporting(expr);`.
- Expression-form internal calls for the currently registered functions,
  including `strlen(expr)`, `str_rot13(expr)`, `strcmp(expr, expr)`,
  `str_contains(expr, expr)`, `str_starts_with(expr, expr)`,
  `str_ends_with(expr, expr)`, `quotemeta(expr)`,
  `chunk_split(expr[, expr[, expr]])`, `strip_tags(expr)`,
  `md5(expr[, raw_output])`,
  `sha1(expr[, raw_output])`, `substr(expr, expr[, expr])`, `bin2hex(expr)`,
  `hex2bin(expr)`, `quoted_printable_decode(expr)`, `dirname(expr)`,
  `soundex(expr)`, `ceil(expr)`, `floor(expr)`,
  `sqrt(expr)`, `fdiv(expr, expr)`, `intdiv(expr, expr)`, `bindec(expr)`,
  `hexdec(expr)`, `octdec(expr)`, `pi()`, `getrandmax()`, `getmypid()`,
  `php_sapi_name()`, `phpversion([extension])`, `intval(expr)`, `chr(expr)`,
  `ord(expr)`,
  `is_finite(expr)`, `is_infinite(expr)`, `is_nan(expr)`,
  `error_reporting(expr)`, `gettype(expr)`, scalar `is_*` type predicates, and
  `array_key_exists(expr, expr)` in echo operands, assignments, binary
  operands, and branch/loop conditions.
- Internal-call arguments are materialized left-to-right before generated C
  runtime dispatch.
- Top-level named user-defined functions with by-value positional parameters,
  local variable storage, ordinary `return` statements, implicit `null`
  returns, recursive calls, and minimal `null` parameter and return type
  declarations over the currently supported expression and statement subset.
  Calls may pass extra arguments, which are currently ignored because argument
  introspection is not modeled yet. Duplicate declarations and declarations
  that collide with currently modeled internal function names are rejected.
- `var_dump()` output for current boxed values: `NULL`, `bool(...)`,
  `int(...)`, `float(...)`, `string(length) "value"`, and ordered literal
  arrays. Finite floats use the shortest decimal spelling that round-trips to
  the same native double, with PHP-style uppercase `E` and unpadded exponent
  widths when scientific notation is required; `INF`, `-INF`, and `NAN` keep
  PHP-like special spellings.
- `strlen()` over current boxed scalar values after scalar string conversion.
- `str_rot13()` over current boxed scalar values after scalar string conversion,
  returning ASCII ROT13 output while leaving non-letters unchanged.
- `strcmp()` over current boxed scalar values after scalar string conversion,
  returning a negative integer, zero, or a positive integer from bytewise
  comparison of the current C-string-backed values.
- `str_contains()` over current boxed scalar values after scalar string
  conversion, returning whether the needle string is present in the haystack
  string through the current C-string-backed value path.
- `str_starts_with()` and `str_ends_with()` over current boxed scalar values
  after scalar string conversion, returning whether the haystack has the
  requested prefix or suffix through the current C-string-backed value path.
- `quotemeta()` over current boxed scalar values after scalar string
  conversion, prefixing `.`, `\`, `+`, `*`, `?`, `[`, `^`, `]`, `(`, `$`, and
  `)` bytes with backslashes through the current C-string-backed value path.
- `chunk_split()` over current boxed scalar values after scalar string
  conversion, using a chunk length converted through the current scalar integer
  conversion path and an ending converted through the current scalar string
  conversion path. The defaults are length `76` and ending `"\r\n"`.
- `strip_tags()` over current boxed scalar values after scalar string
  conversion, removing complete `<...>`, `<?...?>`, `<%...%>`, and HTML
  comment tag regions through the current C-string-backed value path.
- `md5()` and `sha1()` over current boxed scalar values after scalar string
  conversion, returning lowercase hexadecimal digest output, or raw digest
  bytes when the optional `raw_output` argument is truthy and the current
  C-string-backed value path can carry the produced bytes.
- `substr()` over current boxed scalar values after scalar string conversion,
  with start and optional length converted through the current scalar integer
  conversion path. Negative starts count back from the end, negative lengths
  truncate from the end, omitted or `null` lengths read to the end, and extreme
  negative offsets clamp to the beginning.
- `bin2hex()` over current boxed scalar values after scalar string conversion,
  returning lowercase hexadecimal byte output.
- `hex2bin()` over current boxed scalar values after scalar string conversion,
  decoding hexadecimal byte pairs and returning `false` with a warning boundary
  for odd-length or non-hexadecimal input.
- `quoted_printable_decode()` over current boxed scalar values after scalar
  string conversion, decoding `=HH` byte escapes and soft line breaks through
  the current C-string-backed value path.
- `dirname()` over current boxed scalar values after scalar string conversion,
  returning the parent directory for the current C-string-backed path.
- `soundex()` over current boxed scalar values after scalar string conversion,
  returning a PHP-style four-character ASCII soundex key.
- `ceil()` and `floor()` over current boxed scalar values after scalar numeric
  conversion, returning boxed floats.
- `sqrt()` over current boxed scalar values after scalar numeric conversion,
  returning a boxed float.
- `fdiv()` over current boxed scalar values after scalar numeric conversion,
  returning boxed floating-point division results, including zero divisors,
  signed zeroes, infinities, and `NAN`.
- `intdiv()` over current boxed scalar values after scalar integer conversion,
  returning a boxed integer quotient for supported non-zero divisors.
- `pi()` returns the modeled boxed float value of the `M_PI` constant.
- `getrandmax()` returns the modeled maximum random integer.
- `getmypid()` returns the generated native process id.
- `php_sapi_name()` returns the modeled CLI SAPI name.
- `phpversion()` returns the modeled PHP version string. The optional
  extension argument returns the same version for `core`, `standard`, and an
  empty extension name, and `false` for unmodeled extension names.
- `bindec()`, `hexdec()`, and `octdec()` over current boxed scalar values after
  scalar string conversion. The runtime accepts matching `0b`, `0x`, and `0o`
  prefixes, ignores invalid base digits with a deprecation boundary, and
  returns integers until the parsed value exceeds native integer range, then
  floats.
- `intval()` over current boxed scalar values after scalar integer conversion,
  with bounded string/base conversion for supported bases.
- `chr()` over current boxed scalar values after scalar integer conversion,
  returning a one-byte string with byte values constrained modulo 256.
- `ord()` over current boxed scalar values after scalar string conversion,
  returning the first byte as an integer. Empty and multi-byte strings emit
  PHP-like deprecation diagnostics with the internal-call source line.
- `error_reporting()` currently accepts zero or one scalar argument and returns
  a placeholder integer level. It does not configure diagnostic filtering yet.
- `gettype()` over current boxed scalar values, returning `NULL`, `boolean`,
  `integer`, `double`, or `string`.
- Scalar type predicates over current boxed scalar values: `is_null()`,
  `is_bool()`, `is_int()`, `is_integer()`, `is_long()`, `is_float()`,
  `is_double()`, `is_string()`, `is_scalar()`, `is_finite()`,
  `is_infinite()`, and `is_nan()`.
- `function_exists()` over generated user-function declarations and the
  currently registered internal-function names.
- `define()` creates runtime constants over the current boxed value subset,
  returning `false` with a warning when the requested name is already defined.
- `constant()` reads the same runtime and modeled built-in constant registry
  using the current scalar string-conversion result for the name.
- `defined()` over global `const` declarations, constants created with
  `define()`, plus the current modeled constant registry, including `E_ERROR`,
  `PHP_EOL`, `DIRECTORY_SEPARATOR`,
  `PATH_SEPARATOR`, `PHP_INT_MIN`, `PHP_INT_MAX`, `PHP_INT_SIZE`, `INF`,
  `NAN`, `M_PI`, and the modeled PHP math constants `M_E`, `M_LOG2E`,
  `M_LOG10E`, `M_LN2`, `M_LN10`, `M_PI_2`, `M_PI_4`, `M_1_PI`, `M_2_PI`,
  `M_SQRTPI`, `M_2_SQRTPI`, `M_LNPI`, `M_EULER`, `M_SQRT2`, `M_SQRT1_2`,
  and `M_SQRT3`. Other ordinary names report as undefined.
- Duplicate global `const` declarations and `const` redeclarations after
  `define()` emit the modeled duplicate-constant warning boundary and preserve
  the original runtime constant value.
- A minimal `phpc` runner for supported PHPT rows. It compiles scripts or `-r`
  snippets to temporary native binaries through the normal compiler pipeline.
- Braced and single-statement `if`, `elseif`, and `else` statements whose
  conditions and bodies use the currently supported scalar expression and
  statement subset.
- Plain compound statement blocks `{ ... }` over the currently supported
  statement subset. Blocks do not introduce a variable scope, and labels/gotos
  remain visible through recursive validation.
- Script-level `return;` and `return expr;` statements. Optional return
  expressions are evaluated through the current boxed expression path, then the
  generated native program frees runtime state and exits successfully.
- `while (expr) statement` loops where the condition and braced or
  single-statement body use the currently supported scalar expression and
  statement subset.
- `do statement while (expr);` loops where the braced or single-statement body
  and condition use the currently supported scalar expression and statement
  subset. The body executes once before the first condition check.
- `for (init; condition; update) statement` loops where init and update clauses
  use direct variable assignment, direct increment/decrement, or simple
  internal-call statements, conditions use the currently supported scalar
  expression subset, and the body is either a braced block or one supported
  statement. Missing conditions are treated as true.
- Braced `switch (expr) { case expr: ... default: ... }` statements over the
  currently supported scalar expression and statement subset. The generated
  native code evaluates the switch expression once, compares case expressions
  in source order with boxed loose `==` semantics, honors a single `default`,
  allows PHP-style fallthrough, and supports `break;` or explicit-level
  `break N;` from the active emitted switch/loop target stack.
- User labels such as `L1:` and `goto L1;` statements inside the currently
  generated main function, including source-spanned fatal diagnostics for
  undefined target labels, duplicate labels, and `goto` jumps into active loop
  or switch scopes.
- Source-spanned compile diagnostics emitted through `phpc` use PHP-style fatal
  or parse-error boundaries with the source file and line. This currently
  covers duplicate `default:` clauses in `switch`, duplicate labels, undefined
  `goto` labels, invalid `goto` jumps into loop or switch scopes, removed
  `(real)` and `(unset)` cast syntax,
  expression-context `(void)` cast syntax, unterminated block comments, and
  invalid legacy octal integer literals containing `8` or `9`, plus
  unexpected-token parse errors at modeled statement terminators and right
  parentheses. Global `const` declaration terminators report the
  const-specific `"," or ";"` expected-token set.
- Statement-form direct variable increment/decrement: `$name++;`, `++$name;`,
  `$name--;`, and `--$name;`.

## Not Yet Supported

- PHP-exact diagnostic formatting beyond source-spanned compile fatals and
  parse errors, warning file names, line numbers, error handlers, and error
  reporting configuration.
- Full PHP numeric-string conversion warning parity, non-numeric string
  arithmetic diagnostics, exact division/modulo-by-zero exception behavior,
  exact numeric literal overflow/range parity, complete overflow parity, and
  invalid numeric-separator/radix diagnostic parity beyond invalid legacy
  octal integers, unsupported grammar-site parse-error wording, and exact
  scalar cast overflow behavior.
- Prefix and postfix increment/decrement operators such as `++$value` and
  `--$value`.
- `print` as an expression returning `1`, including contexts such as assignment,
  echo operands, binary operands, and the parenthesized spelling `print(...)`.
- Keyword boolean operators `and`/`or`, PHP-exact chained comparison parse
  errors, and complete comparison parity for unsupported value types.
- Unbraced switch bodies, alternate control-flow syntax, `foreach`, `continue`,
  branch-condition assignments, for-loop comma expressions and
  non-direct-variable clause lvalues, PHP-exact break/continue diagnostics,
  labels/goto inside unsupported functions, classes, and `try`/`finally`
  constructs, and exception/finally control-flow edges.
- PHP-exact `return` value propagation for includes/functions and return
  inside unsupported function/class contexts.
- Switch alternate syntax and switch behavior for arrays, objects, references,
  copy-on-write, and exceptions.
- Increment/decrement as expressions, including pre/post result values in echo,
  assignment, binary operands, function arguments, or branch conditions.
- PHP-exact increment/decrement semantics for strings, booleans, arrays,
  objects, references, copy-on-write, overflow edge cases, and diagnostics.
- Inline HTML before `<?php` or between PHP blocks.
- Complex/braced string interpolation and interpolation of arrays, objects,
  offsets, properties, variable variables, or other non-direct-variable forms.
- Internal functions outside the registered scalar subset.
- Namespace/class constants, global `const` duplicate diagnostics and ordering
  parity with runtime `define()`, `define()`'s legacy case-insensitive flag, and
  built-in PHP/extension constants other than the currently modeled `E_ERROR`,
  `PHP_EOL`, `DIRECTORY_SEPARATOR`, `PATH_SEPARATOR`, `PHP_INT_MIN`,
  `PHP_INT_MAX`, `PHP_INT_SIZE`, `INF`, `NAN`, `M_PI`, and modeled PHP math
  `M_*` constants in `defined()`/`constant()`.
- Function forms beyond top-level named by-value declarations, including
  default arguments, variadics, named arguments, by-reference parameters or
  returns, nested or conditional declarations, closures, methods, dynamic
  calls, namespaces, globals, static locals, `func_get_arg()`/
  `func_get_args()`/`func_num_args()`, and PHP-exact function/include return
  propagation.
- Type predicate coverage for arrays, objects, resources, and references.
- Array element mutation, append/unset/iteration, long-form
  `array(...)`, recursive arrays, objects, resources, references, copy-on-write,
  and `var_dump()` reference identity output.
- Exact `array_key_exists()` TypeError parity for unsupported key/container
  types, object property checks, resources, references, and error-handler
  routing.
- String offset writes/mutation, `isset()`/`empty()`/null-coalescing offset
  semantics, string-offset references, string-offset unset, and complete
  TypeError/exception parity for unsupported string offset key types.
- Embedded NUL strings in runtime string values, `var_dump()` string
  length/output, `strlen()`, `str_rot13()`, `strcmp()`, `bin2hex()`, `chr()`,
  `hex2bin()`, `str_contains()`, `quotemeta()`, `chunk_split()`,
  `strip_tags()`, `quoted_printable_decode()`, `md5()`, `sha1()`, `substr()`,
  `soundex()`, `ord()`, or bitwise string results.
- Exact `strcmp()` binary-string behavior for embedded NUL bytes and
  unsupported array/object/resource/reference operands.
- Exact `str_contains()` binary-string behavior for embedded NUL bytes and
  unsupported array/object/resource/reference operands.
- Exact `str_starts_with()`/`str_ends_with()` binary-string behavior for
  embedded NUL bytes and unsupported array/object/resource/reference operands.
- Exact `quotemeta()` embedded-NUL behavior and unsupported
  array/object/resource/reference operand diagnostics.
- Exact `chunk_split()` embedded-NUL behavior, non-positive length exception
  parity, and unsupported array/object/resource/reference operand diagnostics.
- Exact `strip_tags()` binary-string behavior, allowed-tags argument support,
  malformed/incomplete tag parity, and unsupported
  array/object/resource/reference operand diagnostics.
- Exact `quoted_printable_decode()` embedded-NUL output behavior and
  unsupported array/object/resource/reference operand diagnostics.
- `md5()`/`sha1()` raw binary output containing NUL bytes, embedded-NUL input
  parity, and unsupported array/object/resource/reference operand diagnostics.
- Exact `substr()` binary-string behavior for embedded NUL bytes and
  unsupported array/object/resource/reference operands.
- Exact `chr()` diagnostics for out-of-range integers or float-to-int precision
  loss.
- Exact `ord()` strict-types and unsupported-type diagnostics.
- Exact `ceil()`/`floor()` null deprecations, string and unsupported-type
  diagnostics, and complete special-float parity.
- Exact `sqrt()` diagnostics and complete negative/non-finite float parity.
- Exact `fdiv()` unsupported-type diagnostics for arrays, objects, resources,
  and references.
- Exact `intdiv()` catchable exception behavior for zero divisors,
  `PHP_INT_MIN / -1`, and unsupported array/object/resource/reference operands.
- Exact diagnostics and full precision/range parity for `intval()`, `bindec()`,
  `hexdec()`, and `octdec()` on very large or unsupported values.
- Exact `hex2bin()` warning text/file-name parity and unsupported
  array/object/resource/reference diagnostics.
- Exact `dirname()` edge parity for unusual paths, embedded NULs, and
  unsupported array/object/resource/reference operands.
- Exact `soundex()` locale/non-ASCII behavior and unsupported
  array/object/resource/reference operand diagnostics.
- Exact non-finite formatting outside current scalar `var_dump()` output and
  complete non-finite comparison parity for unsupported arrays, objects,
  resources, and references.
- Full PHP float precision and formatting edge cases for `var_dump()` or
  `strlen()` input conversion.
- Complete PHP CLI and PHPT runner option parity for `phpc`.
- Doc comment retention for reflection or metadata. Comments are skipped today.
- PHP-exact `error_reporting()` configuration/filtering behavior.
- PHP-exact `getmypid()` process model parity across SAPIs and unsupported
  platforms.
- PHP-exact version, SAPI, and extension metadata beyond the modeled CLI/core/
  standard boundary.
- Cast spelling diagnostics beyond the currently modeled non-canonical aliases
  and removed `(real)`/`(unset)` plus expression-context `(void)` boundaries.
- Statement-form `(void) expr;` casts.
- Scope-aware magic constants inside functions, methods, classes, traits,
  namespaces, includes, and eval contexts.
- PHP-exact file names, line numbers, error-handler routing, and overflow
  parity for integer-only operator conversion diagnostics, including bitwise,
  shift, and modulo diagnostics.
- Compound assignment operators other than `+=`, `-=`, `*=`, `/=`, `%=`, `**=`,
  `.=`, `&=`, `|=`, `^=`, `<<=`, and `>>=`: `??=`.
- Array, object, string-offset, property, static-property, variable-variable,
  reference, and other non-direct-variable compound-assignment lvalues.
- Reference semantics for compound assignment, including reference identity,
  copy-on-write interactions, and by-reference visibility during writes.
- Arrays, references, copy-on-write, globals, superglobals, classes, objects,
  resources, exceptions, variable variables, includes, and dynamic
  fallback.
