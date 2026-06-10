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
- Statement-form expressions over the currently supported expression subset.
  Generated native code evaluates the boxed value for side effects and
  discards the result.
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
- Assignment expressions for direct variables, array dimension/append lvalues,
  and list/short-array destructuring targets, including by-reference
  destructuring entries in the modeled reference-array subset.
- Undefined direct variable reads emit a runtime warning with generated source
  path and line, then yield `null`.
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
  source-spanned undefined-variable diagnostics remain observable in source
  order.
- Direct named-variable null coalescing assignment `??=`. The compiler uses the
  same quiet lookup path as expression-form `??`, writes only when the variable
  is missing or `null`, and evaluates the right-hand expression lazily.
- Keyed array and string offset null coalescing assignment `??=`. Offset keys are
  evaluated once, the read side is quiet like `??`, and existing array/string
  write helpers perform the conditional store. Append-form `$a[] ??= ...` is
  rejected because PHP must read the target before assigning.
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
- Unparenthesized nested ternary expression statements are rejected with
  PHP-style source-spanned fatal diagnostics for the currently modeled
  forbidden associativity forms.
- Global-scope magic constants `__LINE__`, `__FILE__`, `__DIR__`,
  `__FUNCTION__`, `__METHOD__`, `__CLASS__`, `__TRAIT__`, and
  `__NAMESPACE__`. Scope-dependent names currently resolve to empty strings in
  global scope.
- Short array literals `[...]` and long-form `array(...)` literals with
  optional scalar keys, automatic integer keys, integer-string key
  canonicalization, insertion order, and duplicate-key replacement in the
  current literal-array value subset.
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
- String offset writes/mutation for direct-variable strings such as
  `$string[$offset] = $value` in the current C-string-backed scalar string
  subset. Integer-compatible offsets, negative offsets, integer strings,
  numeric-prefix strings with PHP-style illegal-offset warnings, and scalar
  offset casts are handled by the shared offset conversion path. Positive
  out-of-range writes pad with spaces; negative out-of-range writes emit the
  modeled illegal-offset warning and leave the string unchanged. The assigned
  value is converted to a string, empty string results throw `Error`, and
  multi-byte results emit the modeled first-byte warning before writing the
  first byte.
- Attempts to create references to/from string offsets in supported reference
  lvalue positions, including array literal reference elements, raise the
  modeled PHP `Error` through the runtime exception path.
- `array_key_exists()` over current ordered-array values, using the same
  integer/string key canonicalization path as array literals and reads. `null`
  keys emit the current PHP-like deprecation boundary and canonicalize to the
  empty string.
- `in_array()` over current ordered-array values, using shared loose equality
  or strict identity comparison and dereferencing references in both the needle
  and haystack entries.
- Array cursor reads and moves over direct variable ordered arrays through
  `current()`, `key()`, `reset()`, `end()`, `next()`, and `prev()`. Cursor-moving
  calls are currently limited to direct variable arguments; temporary arrays,
  array offsets, and other non-variable cursor mutation targets fail before
  code generation with an explicit unsupported diagnostic.
- Mutating array internals `array_pop($array)`, `array_push($array, ...)`, and
  `array_shift($array)` over direct variable ordered arrays. These calls detach
  shared array payloads before mutation; temporary arrays, array offsets, and
  other non-direct-variable mutation targets fail before code generation with an
  explicit unsupported diagnostic.
- Sort-family by-reference array mutators such as `sort()`, `asort()`,
  `usort()`, and `array_multisort()` remain unsupported and fail before code
  generation with an explicit unsupported diagnostic.
- `isset(expr[, ...])` and `empty(expr)` over variables, array reads, string
  offset reads, and currently supported value expressions. Variable and offset
  operands use a quiet existence lookup: missing variables, missing offsets,
  non-array containers, and out-of-range string offsets do not emit ordinary
  read warnings; `isset()` returns false for missing or `null` values, and
  `empty()` returns true for missing or PHP-falsey values.
- Expression-form null coalescing `left ?? right` over direct variables, array
  reads, string offset reads, and currently supported value expressions. The
  left operand uses the same quiet lookup path as `isset()`/`empty()`, returns
  present non-`null` values without evaluating the right operand, and evaluates
  the right operand only for missing or `null` left values.
- Boxed scalar and literal-array comparison operators `==`, `!=`, `===`, `!==`,
  `<`, `<=`, `>`, `>=`, and `<=>`. Strict array identity compares type, key
  order, key type, and value. Numeric comparisons involving `NAN` are treated
  as unordered, so equality and ordered scalar comparisons return false while
  `!=`/`!==` return true.
- Boxed scalar boolean operators `&&`, `||`, `and`, `or`, and `xor`.
  `&&`, `||`, `and`, and `or` short-circuit over PHP truthiness for the
  currently supported scalar values; `xor` evaluates both operands
  left-to-right.
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
  `print_r(expr[, return]);`, `strlen(expr);`, `str_rot13(expr);`, `strcmp(expr, expr);`,
  `str_contains(expr, expr);`, `str_starts_with(expr, expr);`,
  `str_ends_with(expr, expr);`, `quotemeta(expr);`,
  `chunk_split(expr[, expr[, expr]]);`, `strip_tags(expr);`,
  `md5(expr[, raw_output]);`,
  `sha1(expr[, raw_output]);`, `substr(expr, expr[, expr]);`, `bin2hex(expr);`,
  `hex2bin(expr);`, `quoted_printable_decode(expr);`, `dirname(expr);`,
  `soundex(expr);`, `ceil(expr);`, `floor(expr);`, `abs(expr);`, `sqrt(expr);`,
  `fdiv(expr, expr);`, `intdiv(expr, expr);`, `bindec(expr);`,
  `hexdec(expr);`, `octdec(expr);`, `pi();`, `getrandmax();`,
  `getmypid();`, `php_sapi_name();`,
  `phpversion([extension]);`, `intval(expr);`, `chr(expr);`, `ord(expr);`,
  `count(expr);`, `array_count_values(expr);`, `array_values(expr);`,
  `array_merge_recursive(expr, ...);`, `array_replace_recursive(expr, ...);`,
  `in_array(expr, expr[, expr]);`,
  `is_finite(expr);`, `is_infinite(expr);`, `is_nan(expr);`, and
  `error_reporting(expr);`.
- Expression-form internal calls for the currently registered functions,
  including `print_r(expr[, return])`, `strlen(expr)`, `str_rot13(expr)`, `strcmp(expr, expr)`,
  `str_contains(expr, expr)`, `str_starts_with(expr, expr)`,
  `str_ends_with(expr, expr)`, `quotemeta(expr)`,
  `chunk_split(expr[, expr[, expr]])`, `strip_tags(expr)`,
  `md5(expr[, raw_output])`,
  `sha1(expr[, raw_output])`, `substr(expr, expr[, expr])`, `bin2hex(expr)`,
  `hex2bin(expr)`, `quoted_printable_decode(expr)`, `dirname(expr)`,
  `soundex(expr)`, `ceil(expr)`, `floor(expr)`,
  `abs(expr)`, `sqrt(expr)`, `fdiv(expr, expr)`, `intdiv(expr, expr)`, `bindec(expr)`,
  `hexdec(expr)`, `octdec(expr)`, `pi()`, `getrandmax()`, `getmypid()`,
  `php_sapi_name()`, `phpversion([extension])`, `intval(expr)`, `chr(expr)`,
  `ord(expr)`,
  `count(expr)`, `array_count_values(expr)`, `array_values(expr)`, `array_merge_recursive(expr, ...)`,
  `array_replace_recursive(expr, ...)`,
  `in_array(expr, expr[, expr])`,
  `is_finite(expr)`, `is_infinite(expr)`, `is_nan(expr)`,
  `error_reporting(expr)`, `gettype(expr)`, scalar `is_*` type predicates, and
  `array_key_exists(expr, expr)` in echo operands, assignments, binary
  operands, and branch/loop conditions.
- Internal-call arguments are materialized left-to-right before generated C
  runtime dispatch.
- Top-level named user-defined functions with by-value positional parameters,
  direct by-reference positional parameters, local variable storage, ordinary
  `return` statements, implicit `null` returns, recursive calls, call-frame
  argument introspection, and minimal `null` parameter and return type
  declarations over the currently supported expression and statement subset.
  Calls may pass extra arguments. Duplicate declarations and declarations that
  collide with currently modeled internal function names are rejected.
- Direct variable reference aliases, grouped direct-variable aliases,
  single-dimension array element references, grouped single-dimension array
  element references, array literal reference elements, and by-value copies near
  references. Unsupported recursive, nested array, temporary offset, and other
  non-lvalue reference forms are rejected explicitly with source spans.
- `var_dump()` output for current boxed values: `NULL`, `bool(...)`,
  `int(...)`, `float(...)`, `string(length) "value"`, and ordered literal
  arrays. Finite floats use the shortest decimal spelling that round-trips to
  the same native double, with PHP-style uppercase `E` and unpadded exponent
  widths when scientific notation is required; `INF`, `-INF`, and `NAN` keep
  PHP-like special spellings.
- `print_r()` output for current boxed values, including scalar output,
  ordered-array formatting, nested arrays, and string-return mode through the
  optional second argument.
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
  for odd-length or non-hexadecimal input. Consecutive invalid-input warnings
  use the modeled PHP display separation, and unsupported array/object
  operands raise a catchable `TypeError`.
- `quoted_printable_decode()` over current boxed scalar values after scalar
  string conversion, decoding `=HH` byte escapes and soft line breaks through
  the current C-string-backed value path.
- `dirname()` over current boxed scalar values after scalar string conversion,
  returning the parent directory for the current C-string-backed path.
- `soundex()` over current boxed scalar values after scalar string conversion,
  returning a PHP-style four-character ASCII soundex key.
- `ceil()` and `floor()` over current boxed scalar values after scalar numeric
  conversion, returning boxed floats.
- `abs()` over current boxed scalar values after scalar numeric conversion,
  returning boxed integer or float magnitudes and emitting the modeled
  PHP null-deprecation boundary.
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
  with bounded string/base conversion for supported bases, including PHP-style
  `0x`/`0b` prefix handling and integer-range saturation.
- `chr()` over current boxed scalar values after scalar integer conversion,
  returning a one-byte string with byte values constrained modulo 256.
- `ord()` over current boxed scalar values after scalar string conversion,
  returning the first byte as an integer. Empty and multi-byte strings emit
  PHP-like deprecation diagnostics with the internal-call source line.
- `count()` over current boxed arrays, returning their length as an integer.
- `array_count_values()` over current boxed arrays, counting dereferenced
  integer and string values with PHP array-key canonicalization. Unsupported
  value types emit the modeled PHP warning boundary and are skipped.
- `array_values()` over current boxed arrays, preserving insertion order while
  returning a freshly reindexed ordered array of cloned values.
- `array_merge_recursive()` and `array_replace_recursive()` over current boxed
  arrays, preserving ordered-map key behavior while cloning dereferenced values
  across COW boundaries.
- `in_array()` over current boxed arrays, returning whether the needle matches
  any entry under loose or strict comparison. References are read through the
  same dereferencing path as other comparison internals.
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
- `foreach (expr as $value) statement` and
  `foreach (expr as $key => $value) statement` loops over current boxed ordered
  arrays. The iterable expression is evaluated once, array entries are visited
  in current insertion order, optional keys and values are assigned through the
  ordinary runtime variable table before the body, and current
  `break`/`continue` level semantics apply inside the body. By-value foreach
  retains the iterable array payload for the loop snapshot, so appends and
  unsets through the source variable or its aliases detach through ordinary
  array copy-on-write and do not change the active iteration set. Non-array
  iterables emit a PHP-style warning with the generated source path, line, and
  operand type, then skip the loop body.
- Braced `switch (expr) { case expr: ... default: ... }` statements over the
  currently supported scalar expression and statement subset. The generated
  native code evaluates the switch expression once, compares case expressions
  in source order with boxed loose `==` semantics, honors a single `default`,
  allows PHP-style fallthrough, and supports `break;` or explicit-level
  `break N;` from the active emitted switch/loop target stack.
- `continue;` and explicit-level `continue N;` over active loop/switch targets.
  The generated backend jumps to the appropriate loop continuation point, runs
  `for` update clauses on `continue`, and emits the current PHP-style warning
  boundary when a `continue` targets a `switch`.
- User labels such as `L1:` and `goto L1;` statements inside the currently
  generated main function, including source-spanned fatal diagnostics for
  undefined target labels, duplicate labels, and `goto` jumps into active loop
  or switch scopes.
- Top-level class declarations with public static methods in the current
  function subset. Static methods are registered in the callable table under
  `Class::method`, can be called directly with `Class::method(...)`, and can be
  used by dynamic calls or internal callbacks through `"Class::method"` and
  `["Class", "method"]` callable values.
- `new stdClass` expressions, boxed object handles, and public dynamic property
  reads/writes such as `$object->name` and `$object->name = expr`. Object
  assignment shares the object handle, so property writes through an alias are
  visible through the original variable. Property assignment expressions return
  the assigned value, and property reads can flow through generated user
  functions and string-callable `call_user_func()` dispatch. This is a bounded
  public-property object-storage slice; PHP class declarations, visibility,
  inheritance, methods, static properties, magic methods, destructors, and
  reflection metadata remain outside this support boundary.
- Source-spanned compile diagnostics emitted through `phpc` use PHP-style fatal
  or parse-error boundaries with the source file and line. This currently
  covers duplicate `default:` clauses in `switch`, duplicate labels, undefined
  `goto` labels, invalid `goto` jumps into loop or switch scopes, removed
  `(real)` and `(unset)` cast syntax,
  expression-context `(void)` cast syntax, unterminated block comments, and
  invalid legacy octal integer literals containing `8` or `9`, plus
  unparenthesized nested ternary fatal diagnostics and
  unexpected-token parse errors at modeled statement terminators and right
  parentheses. Global `const` declaration terminators report the
  const-specific `"," or ";"` expected-token set. Unsupported class members and
  class-constant fetch syntax are recognized and reported as class metadata
  boundaries.
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
- Keyword boolean tails after direct assignment statements, ternary expressions
  beyond the modeled nested associativity diagnostics, PHP-exact chained
  comparison parse errors, and complete comparison parity for unsupported value
  types.
- Unbraced switch bodies, alternate control-flow syntax,
  branch-condition assignments, for-loop comma expressions and
  non-direct-variable clause lvalues, PHP-exact break/continue diagnostics
  beyond the currently modeled level/context fatals and switch-target warning,
  labels/goto inside unsupported functions, classes, and `try`/`finally`
  constructs, and exception/finally control-flow edges.
- Object `Traversable`, destructuring foreach targets, and PHP-exact
  `foreach` diagnostics outside the current array/non-array warning lane.
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
- Internal functions outside the registered internal-function subset.
- Exact undefined-constant and unsupported-expression-statement diagnostics.
- Namespace/class constants, global `const` duplicate diagnostics and ordering
  parity with runtime `define()`, `define()`'s legacy case-insensitive flag, and
  built-in PHP/extension constants other than the currently modeled `E_ERROR`,
  `PHP_EOL`, `DIRECTORY_SEPARATOR`, `PATH_SEPARATOR`, `PHP_INT_MIN`,
  `PHP_INT_MAX`, `PHP_INT_SIZE`, `INF`, `NAN`, `M_PI`, and modeled PHP math
  `M_*` constants in `defined()`/`constant()`.
- Function forms beyond top-level named declarations and the public-static
  class-method callable slice, plus the bounded `stdClass` public-property
  storage slice, including default arguments, variadics, named arguments,
  by-reference returns, nested or conditional declarations, closures, instance
  methods, full class metadata, namespaces, globals, static locals, and
  PHP-exact function/include return propagation.
- Type predicate coverage for arrays, objects, resources, and references.
- Unsupported recursive arrays, full class/object metadata, resources,
  complete reference identity, copy-on-write, and `var_dump()` reference
  identity beyond the currently modeled ordered-array, direct-reference, and
  `stdClass` public-property behavior.
- Exact `array_key_exists()` TypeError parity for unsupported key/container
  types, object property checks, resources, references, and error-handler
  routing.
- String-offset append, unset, compound assignment, property/reference
  `isset()`/`empty()` and null-coalescing semantics, and complete
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
- Exact `abs()` diagnostics for unsupported array/object/resource/reference
  operands and complete overflow parity beyond the current boxed numeric path.
- `count()` support for `Countable` objects and exact non-array diagnostics.
- Exact `sqrt()` diagnostics and complete negative/non-finite float parity.
- Exact `fdiv()` unsupported-type diagnostics for arrays, objects, resources,
  and references.
- Exact `intdiv()` catchable exception behavior for zero divisors,
  `PHP_INT_MIN / -1`, and unsupported array/object/resource/reference operands.
- Exact diagnostics and full precision/range parity for `intval()`, `bindec()`,
  `hexdec()`, and `octdec()` on remaining very large or unsupported values.
- Exact `hex2bin()` source file-name parity, uncaught TypeError stack
  formatting, resource operands, and remaining reference/object edge
  diagnostics.
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
- Object, property, static-property, variable-variable, append-form
  null-coalescing, and remaining non-direct-variable compound-assignment
  lvalues outside modeled keyed array/string offsets.
- Remaining reference semantics for compound assignment outside direct
  variables and modeled array elements, including full copy-on-write
  interactions and by-reference visibility during writes.
- Arrays, references, copy-on-write, globals, superglobals, classes, objects,
  resources, exceptions, variable variables, includes, and dynamic
  fallback.
