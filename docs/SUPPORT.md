# Support

This file tracks user-visible PHP behavior that the scratch compiler currently
supports in generated native binaries.

## RC Boundary

The release-candidate surface is the generic compiler/runtime path exercised by
`examples/rc`: scalar control flow, strings, ordered arrays, selected standard
internals, top-level functions, includes resolved at compile time, copy-on-write
array/reference slices, and public class/object shells. Public class support is
bounded to top-level declarations with public methods, direct public static
property reads/writes, public instance property reads/writes, and public
property `??=`, plus public non-static `__construct` dispatch through the
declared-method path.

Post-RC architecture remains explicit rather than hidden:

- Classes and inheritance: interfaces, traits, namespaces,
  non-public/typed/promoted properties, broad metadata/reflection, destructors,
  old-style constructors, class constants, and complete visibility-aware
  inherited property/method resolution remain outside the RC boundary.
- Static properties: direct public static reads/writes are supported, but
  visibility, inheritance, late static binding, typed/default metadata, and
  static-property compound or null-coalescing lvalues are post-RC.
- Magic methods: public declared instance `__construct` is supported during
  object construction, and public declared instance `__call` is supported as a
  fallback for direct object calls and supported object callable dispatch when
  no declared method matches. Public declared instance `__toString` is
  supported for current runtime string conversions. `__destruct`, `__get`,
  `__set`, `__isset`, `__unset`, `__callStatic`, `__invoke`, and related hooks
  remain unsupported.
- Non-static callables: direct object method calls and bounded
  `[$object, "method"]` callback dispatch work for declared and inherited
  public methods, and missing object methods fall through to supported
  `__call`. `is_callable()` validates the current string, closure,
  `["Class", "staticMethod"]`, and `[$object, "method"]` subset, including
  `__call`-capable objects and syntax-only checks. First-class callable syntax,
  non-public visibility, `__invoke`, `__callStatic`, and arbitrary dynamic
  instance method metadata remain post-RC.
- Object destructuring and object `Traversable` remain unsupported; current
  destructuring support is array/list lvalues.
- Property compound lvalues remain post-RC except public property `??=`:
  property `+=`, `.=` and other compounds, property inc/dec, nested/dynamic
  property lvalues, and static-property compounds are unsupported.

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
- Double-quoted strings with direct `$name`, simple variable-root array offset
  interpolation such as `$items[$key]`, `$items[name]`, and `$items[0]`,
  braced `{$name}`/`{$items['key']}` forms, and deprecated legacy `${name}`
  variables. Legacy dollar-brace variables emit the modeled PHP deprecation.
  Interpolated values use the same runtime variable/array reads, scalar or
  public `__toString` object casts, and concatenation paths as ordinary
  expressions.
- Double-quoted string escapes for `\n`, `\r`, `\t`, `\v`, `\f`, escaped
  backslash, quote, dollar, `\xNN`, and octal byte sequences. Hex and octal
  escapes can produce high bytes in native string literals; octal overflow
  diagnostics are not yet modeled.
- Plain heredoc and nowdoc literals with unindented closing labels are accepted
  as string values when their bodies do not require interpolation. Heredoc
  interpolation stops at an explicit unsupported diagnostic instead of being
  silently treated as literal text.
- Direct variable assignment and scalar reads through the generated runtime
  symbol table.
- Variable-variable reads, ordinary assignments, and simple array-offset writes
  with `$$name`, `$$$name`, and `${expr}` over scalar runtime names. Dynamic
  names are converted through the shared scalar string-conversion path for
  `null`, booleans, integers, floats, and strings; arrays, objects, closures,
  exceptions, and embedded-NUL string names stop at an explicit
  unsupported-name diagnostic. Dynamic-root array-offset writes such as
  `${$name}[$key] = $value` evaluate the dynamic name and offset expressions
  before the right-hand side, then reuse the shared array-path write helper.
- Assignment expressions for direct variables, array dimension/append lvalues,
  and list/short-array destructuring targets, including by-reference
  destructuring entries in the modeled reference-array subset.
- Assignment expressions in branch and loop conditions for the currently
  supported expression-assignment subset. Direct-variable compound condition
  assignments read the current value, evaluate the right-hand side, write the
  computed value, and branch on that assigned value's PHP truthiness.
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
  warning boundary. Non-numeric strings and mixed array operands for `+`, `-`,
  `*`, `**`, and `/` throw modeled catchable PHP `TypeError` diagnostics.
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
- Print statements and expression-form `print` use the same generated boxed
  output path as echo. Expression-form `print` returns `1` after evaluating and
  printing its operand, including assignment values, echo operands, binary
  operands, and parenthesized `print(...)` operands.
- Parenthesized expressions for grouping supported scalar expressions,
  including nested grouping.
- Unary `+` over boxed scalar numeric values.
- Unary `-` over boxed scalar numeric values.
- Unary `!` using boxed PHP scalar truthiness: `null`, `false`, numeric zero,
  `0.0`, `""`, and `"0"` are falsey; other supported scalar values are truthy.
- Unary bitwise `~` over supported boxed scalar values. String operands produce
  bytewise string results for non-NUL string data; other supported scalar
  operands are converted to integers through the bitwise numeric path. Float
  precision-loss deprecations respect `error_reporting()`, and out-of-range
  direct floats emit the modeled non-representable integer warning.
- Scalar `(int)`, `(float)`, `(string)`, `(bool)`, and deprecated
  non-canonical `(integer)`, `(double)`, `(binary)`, and `(boolean)` casts over
  supported boxed scalar values.
- `phpc -d precision=N` accepts bounded integer precision values for generated
  native execution. Scalar float stringification for echo, string casts,
  concatenation, string internals such as `strlen()`, and `print_r()` uses that
  precision, defaulting to PHP's current 14 significant-digit boundary when the
  setting is absent. Scientific notation uses PHP-style uppercase `E`,
  unpadded exponent widths, and decimal mantissas; non-finite values stringify
  as `INF`, `-INF`, and `NAN`.
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
- Magic constants `__LINE__`, `__FILE__`, and `__DIR__`, plus global-scope
  `__FUNCTION__`, `__METHOD__`, `__CLASS__`, `__TRAIT__`, and
  `__NAMESPACE__` empty-string behavior. Top-level functions expose
  `__FUNCTION__` and `__METHOD__`; declared class methods expose
  `__FUNCTION__`, `__METHOD__`, and `__CLASS__` for the current method scope.
  Traits, namespaces, includes, and eval remain outside this boundary.
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
  first byte. String-offset compound assignment remains unsupported, but its
  key uses the same string-offset conversion diagnostics before the modeled PHP
  `Error`. Direct and nested attempts to unset string offsets throw the modeled
  PHP `Error` and leave the string unchanged.
- Attempts to create references to/from string offsets in supported reference
  lvalue positions, including array literal reference elements, raise the
  modeled PHP `Error` through the runtime exception path.
- `array_key_exists()` over current ordered-array values, using the same
  integer/string key canonicalization path as array literals and reads. `null`
  keys emit the current PHP-like deprecation boundary with the shared leading
  diagnostic separator and canonicalize to the empty string. Resource keys emit
  the PHP-like cast warning and canonicalize to their integer resource ID.
- `array_column()` over current ordered-array inputs, reading array entries or
  object properties by int/string column keys, returning whole rows for `null`
  column keys, optionally keying rows from an int/string index key, and using
  the usual PHP array-key canonicalization for numeric strings.
- `in_array()` over current ordered-array values, using shared loose equality
  or strict identity comparison and dereferencing references in both the needle
  and haystack entries.
- Array cursor reads and moves over ordered arrays through `current()`, `key()`,
  `reset()`, `end()`, `next()`, and `prev()`. Cursor-moving calls support direct
  variables and variable-root array paths such as `$items[0]`, detaching shared
  nested array payloads before mutation; temporary arrays and other
  non-variable-root cursor mutation targets fail before code generation with an
  explicit unsupported diagnostic.
- Mutating array internals `array_pop($array)`, `array_push($array, ...)`,
  `array_shift($array)`, `array_unshift($array, ...)`, `ksort($array)`, and
  `shuffle($array)` over direct variable ordered arrays. One-argument
  `array_pop()` and `array_shift()` also support variable-root array paths such
  as `$items[0]`, detaching shared nested array payloads before mutation.
  `ksort()` uses ascending default key order, and `shuffle()` reindexes values
  with integer keys. Temporary arrays, array paths for variadic
  `array_push()`/`array_unshift()`, array paths for `ksort()`/`shuffle()`, and
  other non-direct-variable mutation targets fail before code generation with
  an explicit unsupported diagnostic.
- Remaining sort-family by-reference array mutators such as `sort()`,
  `asort()`, `krsort()`, `usort()`, and `array_multisort()` remain unsupported
  and fail before code generation with an explicit unsupported diagnostic.
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
  scalar operands are converted to integers through the bitwise numeric path;
  float precision-loss deprecations, out-of-range float warnings, and
  leading-numeric-string warnings respect `error_reporting()`.
- Boxed scalar bit shifts `<<` and `>>`. Supported scalar operands are
  converted to integers through the current bitwise integer-conversion path,
  including the current float/float-string precision-loss deprecation boundary
  and leading-numeric-string warning boundary.
- Simple statement-form internal calls such as `var_dump(expr, ...)`,
  `var_export(expr[, return]);`, `print_r(expr[, return]);`, `strlen(expr);`,
  `addcslashes(expr, expr);`,
  `stripcslashes(expr);`, `addslashes(expr);`, `stripslashes(expr);`,
  `str_rot13(expr);`, `str_shuffle(expr);`, `strcmp(expr, expr);`,
  `str_contains(expr, expr);`, `str_starts_with(expr, expr);`,
  `str_ends_with(expr, expr);`, `str_repeat(expr, expr);`, `quotemeta(expr);`,
  `chunk_split(expr[, expr[, expr]]);`, `strip_tags(expr);`,
  `join(expr[, expr]);`, `implode(expr[, expr]);`, `sprintf(expr, ...);`,
  `md5(expr[, raw_output]);`,
  `sha1(expr[, raw_output]);`, `substr(expr, expr[, expr]);`, `bin2hex(expr);`,
  `hex2bin(expr);`, `quoted_printable_decode(expr);`, `dirname(expr);`,
  `highlight_string(expr[, return]);`, `highlight_file(expr[, return]);`,
  `soundex(expr);`, `ceil(expr);`, `floor(expr);`, `abs(expr);`, `sqrt(expr);`,
  `pow(expr, expr);`, `fdiv(expr, expr);`, `intdiv(expr, expr);`, `bindec(expr);`,
  `hexdec(expr);`, `octdec(expr);`, `pi();`, `getrandmax();`,
  `getmypid();`, `ob_get_contents();`, `php_sapi_name();`,
  `phpversion([extension]);`, `intval(expr);`, `chr(expr);`, `ord(expr);`,
  `count(expr);`, `array_chunk(expr, expr[, expr]);`,
  `array_change_key_case(expr[, expr]);`, `array_column(expr, expr[, expr]);`,
  `array_combine(expr, expr);`, `array_count_values(expr);`,
  `array_fill(expr, expr, expr);`, `array_filter(expr[, expr[, expr]]);`,
  `array_intersect(expr, ...);`, `array_intersect_assoc(expr, ...);`,
  `array_udiff(expr, expr, callback);`,
  `array_udiff_assoc(expr, expr, callback);`,
  `array_udiff_uassoc(expr, expr, callback, callback);`,
  `array_values(expr);`, `array_merge(expr, ...);`,
  `array_merge_recursive(expr, ...);`,
  `array_replace_recursive(expr, ...);`,
  `call_user_func_array(expr, expr);`,
  `assert(expr[, description]);`,
  `in_array(expr, expr[, expr]);`,
  `is_callable(expr[, syntax_only]);`, `is_finite(expr);`,
  `is_infinite(expr);`, `is_nan(expr);`, and `error_reporting(expr);`.
- Expression-form internal calls for the currently registered functions,
  including `var_export(expr[, return])`, `print_r(expr[, return])`,
  `strlen(expr)`, `addcslashes(expr, expr)`,
  `stripcslashes(expr)`, `addslashes(expr)`, `stripslashes(expr)`,
  `str_rot13(expr)`, `str_shuffle(expr)`, `strcmp(expr, expr)`,
  `str_contains(expr, expr)`, `str_starts_with(expr, expr)`,
  `str_ends_with(expr, expr)`, `str_repeat(expr, expr)`, `quotemeta(expr)`,
  `chunk_split(expr[, expr[, expr]])`, `strip_tags(expr)`,
  `join(expr[, expr])`, `implode(expr[, expr])`, `sprintf(expr, ...)`,
  `md5(expr[, raw_output])`,
  `sha1(expr[, raw_output])`, `substr(expr, expr[, expr])`, `bin2hex(expr)`,
  `hex2bin(expr)`, `quoted_printable_decode(expr)`, `dirname(expr)`,
  `highlight_string(expr[, return])`, `highlight_file(expr[, return])`,
  `soundex(expr)`, `ceil(expr)`, `floor(expr)`,
  `abs(expr)`, `sqrt(expr)`, `pow(expr, expr)`, `fdiv(expr, expr)`, `intdiv(expr, expr)`, `bindec(expr)`,
  `hexdec(expr)`, `octdec(expr)`, `pi()`, `getrandmax()`,
  `getmypid()`, `ob_get_contents()`, `php_sapi_name()`,
  `phpversion([extension])`, `intval(expr)`, `chr(expr)`, `ord(expr)`,
  `count(expr)`, `array_chunk(expr, expr[, expr])`,
  `array_change_key_case(expr[, expr])`, `array_column(expr, expr[, expr])`,
  `array_combine(expr, expr)`, `array_count_values(expr)`,
  `array_fill(expr, expr, expr)`, `array_filter(expr[, expr[, expr]])`,
  `array_intersect(expr, ...)`, `array_intersect_assoc(expr, ...)`,
  `array_udiff(expr, expr, callback)`,
  `array_udiff_assoc(expr, expr, callback)`,
  `array_udiff_uassoc(expr, expr, callback, callback)`,
  `array_values(expr)`, `array_merge(expr, ...)`,
  `array_merge_recursive(expr, ...)`, `array_replace_recursive(expr, ...)`,
  `call_user_func_array(expr, expr)`,
  `assert(expr[, description])`,
  `in_array(expr, expr[, expr])`,
  `fopen(expr, expr[, expr[, expr]])`, `fclose(expr)`,
  `is_callable(expr[, syntax_only])`, `is_finite(expr)`,
  `is_infinite(expr)`, `is_nan(expr)`,
  `error_reporting(expr)`, `gettype(expr)`, scalar plus array/object/resource
  `is_*` type predicates, and
  `array_key_exists(expr, expr)` in echo operands, assignments, binary
  operands, and branch/loop conditions.
- Internal-call arguments are materialized left-to-right before generated C
  runtime dispatch.
- `highlight_string()` and `highlight_file()` use the current bounded source
  highlighting path for native binaries. With `return` truthy they return the
  highlighted string, otherwise they write it to stdout and return `true`;
  missing files emit modeled warnings and return `false`. `ob_get_contents()`
  returns `false` because active output buffers are not yet modeled.
- Direct `assert(expr)` calls throw catchable `AssertionError` values with
  PHP-style compiler-generated default messages when the assertion is false;
  explicit scalar descriptions are string-converted through the shared runtime
  path, and dynamic one-argument calls use PHP's empty fallback message.
  Assertion INI modes and throwable descriptions are not yet modeled.
- Top-level named user-defined functions with by-value positional parameters,
  direct by-reference positional parameters, final variadic parameters,
  trailing scalar and literal-array default parameter values including omitted
  `null` defaults, local variable storage, ordinary `return` statements,
  implicit `null` returns, recursive calls, call-frame argument introspection,
  and minimal `null` parameter and return type declarations over the currently
  supported expression and statement subset. Direct calls may omit defaulted trailing
  arguments, pass extra positional arguments, or use named arguments for direct
  generated user-function calls: argument expressions are evaluated
  left-to-right, values are bound to parameters by name, call-frame
  introspection observes parameter order, and unknown or overwritten parameter
  names raise the modeled fatal boundary. Duplicate declarations and
  declarations that collide with currently modeled internal function names are
  rejected. Required parameters after optional parameters, variadic defaults,
  object defaults, reference defaults, and non-constant default expressions are
  rejected before code generation.
- Direct variable reference aliases, grouped direct-variable aliases,
  single-dimension array element references, grouped single-dimension array
  element references, array literal reference elements, and by-value copies near
  references. Unsupported recursive, nested array, temporary offset, and other
  non-lvalue reference forms are rejected explicitly with source spans.
- `var_dump()` output for current boxed values: `NULL`, `bool(...)`,
  `int(...)`, `float(...)`, `string(length) "value"`, and ordered literal
  arrays. Finite floats use the shortest decimal spelling that round-trips to
  the same native double, keep integer-valued floats below `1e17` in fixed
  decimal notation, and use PHP-style uppercase `E`, decimal mantissas, and
  unpadded exponent widths when scientific notation is required. `INF`,
  `-INF`, and `NAN` keep PHP-like special spellings.
- `print_r()` output for current boxed values, including scalar output,
  ordered-array formatting, nested arrays, and string-return mode through the
  optional second argument.
- `var_export()` output for current boxed null, boolean, integer, float, string,
  and ordered-array values, including nested arrays and string-return mode
  through the optional second argument.
- `strlen()` over current boxed scalar values and objects with a public
  declared `__toString()` after shared string conversion.
- Bounded `highlight_string()` and `highlight_file()` use the current
  PHP-style escaped source-byte highlighter: the optional return flag returns a
  string instead of writing to stdout, `highlight_file()` reads ordinary
  filesystem paths, and missing files emit PHP-style highlighting warnings
  before returning false. `ob_get_contents()` returns `false` because output
  buffers are not yet modeled.
- `join()`/`implode()` concatenate current ordered-array values in iteration
  order. The one-argument form uses an empty separator; the two-argument form
  uses the shared string-argument path for the separator. Entries use the
  length-aware runtime string conversion path, and nested arrays emit the
  current array-to-string warning boundary.
- `sprintf()` over current scalar values supports `%%`, `%s`, `%d`/`%i`,
  `%u`, `%o`, `%x`/`%X`, `%b`, `%c`, and `%f`/`%F`/`%e`/`%E`/`%g`/`%G`
  conversions with ordinary flags, width, and precision. String formatting is
  length-aware for embedded NUL bytes in current string operands.
- Shared string-argument checking for common string/byte internals: supported
  scalar values and objects with a public declared `__toString()` are coerced
  through the same length-aware operand path, `null` arguments emit the modeled
  deprecation, and arrays, objects without the current `__toString()` support,
  closures, and exceptions throw the modeled `TypeError` boundary for `strlen()`,
  `str_rot13()`, `str_shuffle()`, `strcmp()`, `str_contains()`,
  `str_starts_with()`, `str_ends_with()`, `str_repeat()`, three-argument
  `strtr()`, `quotemeta()`, `chunk_split()` string/separator arguments,
  `strip_tags()`, `md5()`, `sha1()`, `substr()`, `addcslashes()`,
  `addslashes()`, `stripcslashes()`, `stripslashes()`, `bin2hex()`,
  `hex2bin()`, `quoted_printable_decode()`, `soundex()`, and `ord()`.
- `str_rot13()` over current boxed scalar values after scalar string conversion,
  returning ASCII ROT13 output while leaving non-letters unchanged.
- `str_shuffle()` over current boxed scalar values after scalar string
  conversion, returning a byte-shuffled string without mutating the input.
- `strcmp()` over current boxed scalar values after scalar string conversion,
  returning a negative integer, zero, or a positive integer from bytewise
  comparison of the current C-string-backed values.
- `str_contains()` over current boxed scalar values after scalar string
  conversion, returning whether the needle string is present in the haystack
  string through the current C-string-backed value path.
- `str_starts_with()` and `str_ends_with()` over current boxed scalar values
  after scalar string conversion, returning whether the haystack has the
  requested prefix or suffix through the current C-string-backed value path.
- `str_repeat()` over current boxed scalar values after scalar string
  conversion, repeating the input by a count converted through the current
  scalar integer conversion path and rejecting negative counts with the modeled
  `ValueError` boundary.
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
- `addcslashes()` and `stripcslashes()` over current boxed scalar values after
  scalar string conversion. `addcslashes()` supports literal character lists
  and ascending `a..z` byte ranges; `stripcslashes()` decodes common C-style
  escapes plus octal and one- or two-digit hexadecimal byte escapes.
- `addslashes()` and `stripslashes()` over current boxed scalar values after
  scalar string conversion. `addslashes()` escapes NUL, single quote, double
  quote, and backslash bytes; `stripslashes()` removes backslashes and decodes
  backslash-zero to NUL.
- `dirname()` over current boxed scalar values after scalar string conversion,
  returning the parent directory for the current binary-safe path. Empty paths,
  embedded NUL bytes, platform directory separators, null-argument
  deprecations, and modeled TypeError diagnostics for array/object operands are
  handled through the runtime path.
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
- `pow()` calls the same boxed numeric exponentiation helper as the `**`
  operator.
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
  returning a one-byte string with byte values constrained modulo 256 and
  emitting the modeled out-of-range deprecation for integers outside
  `[0, 255]`.
- `ord()` over current boxed scalar values after scalar string conversion,
  returning the first byte as an integer. Empty and multi-byte strings emit
  PHP-like deprecation diagnostics with the internal-call source line.
- `count()` over current boxed arrays, returning their length as an integer.
- `array_count_values()` over current boxed arrays, counting dereferenced
  integer and string values with PHP array-key canonicalization. Unsupported
  value types emit the modeled PHP warning boundary and are skipped.
- `array_chunk()` over current boxed arrays, returning fresh ordered chunk
  arrays. The chunk length uses the current scalar integer conversion path,
  non-positive lengths throw the modeled PHP `ValueError`, values are cloned
  from dereferenced entries, and the optional preserve-keys flag controls
  whether original integer/string keys are cloned or each chunk is reindexed
  from zero.
- `array_combine()` over current boxed key/value arrays, pairing entries by
  insertion order, converting key values through the shared
  `array_fill_keys()` canonicalization path, cloning values into a fresh
  ordered array while preserving reference values, and throwing the modeled PHP
  `ValueError` when the arrays have different element counts.
- `array_fill()` over boxed start/count operands and mixed boxed values,
  returning a fresh ordered array keyed from the requested integer start.
  Negative counts throw the modeled PHP `ValueError`; filled array/object
  payloads use the shared COW clone path.
- `array_filter()` over current boxed arrays, preserving original keys in a
  fresh ordered result. With a `null` callback it keeps values that are truthy
  under the shared PHP truthiness helper; otherwise it dispatches through the
  shared callable path with value, key, or value/key arguments according to
  `ARRAY_FILTER_USE_KEY` and `ARRAY_FILTER_USE_BOTH`, and rejects unknown modes
  with the modeled PHP `ValueError`.
- `array_flip()` over current boxed arrays, flipping dereferenced integer and
  string values into ordered-map keys and using the original keys as values.
  Unsupported value types emit the modeled PHP warning boundary and are skipped.
- `array_change_key_case()` over current boxed arrays, preserving integer keys,
  converting string keys through ASCII lower/upper casing, and cloning
  dereferenced values into a fresh ordered array. The optional case flag accepts
  `CASE_LOWER`/`CASE_UPPER` and rejects other values with the modeled PHP
  `ValueError`.
- `array_column()` over current boxed arrays, selecting array entries or object
  properties by int/string column key, returning full rows for `null`
  column keys, optionally using row values as result keys, skipping missing
  columns, and cloning dereferenced values across COW boundaries.
- `array_values()` over current boxed arrays, preserving insertion order while
  returning a freshly reindexed ordered array of cloned values.
- `array_merge()` over current boxed arrays, appending integer-keyed entries
  with fresh sequential keys, overwriting string-keyed entries by key while
  preserving insertion order, and cloning dereferenced values across COW
  boundaries.
- `array_intersect()` and `array_intersect_assoc()` over current boxed arrays,
  preserving entries from the first array in a fresh ordered result. Value
  matches use PHP-style string forms; the associative variant also requires a
  matching key in every compared array.
- `array_udiff()`, `array_udiff_assoc()`, and `array_udiff_uassoc()` over
  current boxed arrays, using shared callable dispatch for value comparisons
  and, for `array_udiff_uassoc()`, key comparisons.
- `range()` over current boxed integer-convertible start, end, and optional
  step arguments, returning ordered arrays of integer values and throwing the
  modeled `ValueError` for zero or out-of-range steps.
- `array_merge_recursive()` and `array_replace_recursive()` over current boxed
  arrays, preserving ordered-map key behavior while cloning dereferenced values
  across COW boundaries.
- `in_array()` over current boxed arrays, returning whether the needle matches
  any entry under loose or strict comparison. References are read through the
  same dereferencing path as other comparison internals.
- `error_reporting()` accepts zero or one scalar argument, returns the previous
  PHP-style mask on writes or current mask on reads, and filters the modeled
  shared warning/deprecation/notice emitters. Expression-level `@` suppression
  still stacks independently with the configured mask.
- `gettype()` over current boxed values, returning `NULL`, `boolean`,
  `integer`, `double`, `string`, `array`, `object`, `resource`, or
  `resource (closed)` for the currently modeled scalar, array, object,
  Closure, exception, and stream-resource value domains.
- Type predicates over current boxed scalar and selected non-scalar values:
  `is_array()`, `is_object()`, `is_null()`,
  `is_bool()`, `is_int()`, `is_integer()`, `is_long()`, `is_float()`,
  `is_double()`, `is_string()`, `is_scalar()`, `is_finite()`,
  `is_infinite()`, `is_nan()`, and `is_resource()` for open stream resources.
- `fopen()` opens filesystem-backed streams through the shared resource value
  model, and `fclose()` closes those resources. Closed stream resources remain
  boxed values for `gettype()` and `var_dump()` but no longer satisfy
  `is_resource()`.
- `function_exists()` over generated user-function declarations and the
  currently registered internal-function names.
- `is_callable()` over current string, closure, static method array, and object
  method array callable values, including inherited public object methods,
  supported `__call` fallback, and the optional syntax-only flag. The third
  by-reference callable-name output parameter is not yet supported.
- `call_user_func_array()` expands current ordered-array argument values through
  the shared callable dispatch path, preserving reference entries for the
  current by-reference callable subset. Unreferenced values passed to
  by-reference user parameters warn and return `null` without calling the
  target.
- `assert()` over current boxed assertion expressions. Truthy assertions return
  `true`; falsey assertions throw a modeled `AssertionError`. One-argument
  direct calls carry a compiler-generated default assertion message from the
  parsed expression tree.
- `define()` creates runtime constants over the current boxed value subset,
  returning `false` with a warning when the requested name is already defined.
  Its legacy third `$case_insensitive` argument is accepted for PHP 8 parity:
  truthy values emit the modeled ignored-argument warning and all constants
  remain case-sensitive.
- `constant()` reads the same runtime and modeled built-in constant registry
  using the current scalar string-conversion result for the name.
- `defined()` over global `const` declarations, constants created with
  `define()`, plus the current modeled constant registry, including `E_ERROR`,
  `PHP_EOL`, `DIRECTORY_SEPARATOR`,
  `PATH_SEPARATOR`, `PHP_INT_MIN`, `PHP_INT_MAX`, `PHP_INT_SIZE`, the PHP
  `E_*` error-reporting mask constants, `INF`, `NAN`, `M_PI`, and the modeled
  PHP math constants `M_E`, `M_LOG2E`, `M_LOG10E`, `M_LN2`, `M_LN10`,
  `M_PI_2`, `M_PI_4`, `M_1_PI`, `M_2_PI`, `M_SQRTPI`, `M_2_SQRTPI`,
  `M_LNPI`, `M_EULER`, `M_SQRT2`, `M_SQRT1_2`, `M_SQRT3`,
  `ARRAY_FILTER_USE_BOTH`, and `ARRAY_FILTER_USE_KEY`. Other ordinary names
  report as undefined.
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
- `include expr` and `require expr` over compile-time-resolved string paths,
  including string literals and `__DIR__`/`__FILE__` concatenation. Included
  statement-only files are compiled into native helpers that share the caller's
  current variable frame, emit ordinary output at the include point, return the
  included `return expr;` value, return `null` for `return;`, and return
  `int(1)` when the included file reaches EOF without `return`.
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
- `foreach (expr as target) statement` and
  `foreach (expr as key_target => value_target) statement` loops over current
  boxed ordered arrays. The iterable expression is evaluated once, array
  entries are visited in current insertion order, optional keys and values are
  assigned through ordinary assignment-target storage for direct variables and
  array dimensions before the body, and current
  `break`/`continue` level semantics apply inside the body. By-value foreach
  retains the iterable array payload for the loop snapshot, so appends and
  unsets through the source variable or its aliases detach through ordinary
  array copy-on-write and do not change the active iteration set. Non-array
  iterables emit a PHP-style warning with the generated source path, line, and
  operand type, including `true`/`false` boolean spelling, then skip the loop
  body.
- Braced `switch (expr) { case expr: ... default: ... }` statements over the
  currently supported expression and statement subset, including arrays,
  references, and public-property object shells through boxed loose `==`
  semantics. The generated native code evaluates the switch expression once,
  compares case expressions in source order, honors a single `default`, allows
  PHP-style fallthrough, and supports `break;` or explicit-level `break N;`
  from the active emitted switch/loop target stack.
- `continue;` and explicit-level `continue N;` over active loop/switch targets.
  The generated backend jumps to the appropriate loop continuation point, runs
  `for` update clauses on `continue`, and emits the current PHP-style warning
  boundary when a `continue` targets a `switch`, including inside generated
  user-function bodies.
- User labels such as `L1:` and `goto L1;` statements inside the currently
  generated main function, including source-spanned fatal diagnostics for
  undefined target labels, duplicate labels, and `goto` jumps into active loop
  or switch scopes.
- Top-level class declarations with public static and instance methods in the
  current function subset. Static methods are registered in the callable table
  under `Class::method`, can be called directly with `Class::method(...)`, and
  can be used by dynamic calls or internal callbacks through `"Class::method"`
  and `["Class", "method"]` callable values. Declared class names and declared
  method names are exposed through bounded `class_exists()` and
  `method_exists()` metadata, with case-insensitive lookup and `stdClass`
  recognized as the current built-in object shell. Declared and inherited
  public instance methods can be called directly through object receivers and
  through `[$object, "method"]` callable values, including internal callback
  dispatch. Public `__construct` methods in declared classes are invoked
  during `new Class(...)` after declared property defaults are installed,
  using the same method dispatch, `$this` binding, inherited public method
  lookup, positional argument/default-parameter handling, and return-value
  cleanup as other declared instance methods. Missing direct and callable
  object method dispatch falls through to inherited public
  `__call($name, $args)` when present; the generated helper supplies the
  attempted method name and an ordered argument array. `is_callable()` reports
  the supported string, closure, static-method array, object-method array,
  inherited method, and `__call` fallback subset, with optional syntax-only
  checks for valid callable shapes.
- Public static property declarations in top-level classes, using the supported
  constant-expression default subset. Generated native code initializes
  declaration-backed static slots before top-level statements, supports
  `Class::$name` reads and writes, resolves `self::$name` inside declared
  methods, and throws modeled PHP `Error` diagnostics for undeclared static
  properties.
- `new stdClass` and declared-class object shells, boxed object handles, public
  dynamic property reads/writes such as `$object->name`, and public declared
  instance properties with supported constant defaults. Object assignment
  shares the object handle, declared defaults are initialized on construction,
  property assignment expressions return the assigned value, and property reads
  can flow through generated user functions and string-callable
  `call_user_func()` dispatch. Public property null coalescing assignment
  `$object->name ??= expr` quiet-reads the property and lazily evaluates the
  right-hand expression. Non-public, typed, inherited, constructor-promoted,
  magic, destructor, and reflection property metadata remain outside this
  support boundary.
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
  const-specific `"," or ";"` expected-token set, and removed alternative
  `{}` offsets inside braced string interpolation report the current PHP
  unexpected-token parse error. Unsupported class members and class-constant
  fetch syntax are recognized and reported as class metadata boundaries.
- Direct variable increment/decrement over current boxed integer and float
  values. Statement forms `$name++;`, `++$name;`, `$name--;`, and `--$name;`
  write the updated value. Expression forms such as `++$name`, `$name++`,
  `--$name`, and `$name--` return the PHP pre/post result value while applying
  the same side effect.
- Statement-form array dimension increment/decrement over variable-root array
  paths such as `$items[$key]++;` and `--$items[0];`. The parser lowers these
  forms through the same array-path compound-assignment helpers as `+= 1` and
  `-= 1`.

## Not Yet Supported

- PHP-exact diagnostic formatting beyond source-spanned compile fatals and
  parse errors, warning file names, line numbers, error handlers, and error
  reporting configuration.
- Full PHP numeric-string conversion warning parity, non-numeric string
  arithmetic diagnostics, exact division/modulo-by-zero exception behavior,
  exact numeric literal overflow/range parity, complete overflow parity, and
  invalid numeric-separator/radix diagnostic parity beyond invalid legacy
  octal integers, remaining unsupported grammar-site parse-error wording, and
  exact scalar cast overflow behavior.
- Increment/decrement expression targets beyond direct variables, and property
  or static-property increment/decrement targets.
- Keyword boolean tails after direct assignment statements, ternary expressions
  beyond the modeled nested associativity diagnostics, PHP-exact chained
  comparison parse errors, and complete comparison parity for unsupported value
  types.
- Unbraced switch bodies, alternate control-flow syntax,
  for-loop comma expressions and
  non-direct-variable clause lvalues, PHP-exact break/continue diagnostics
  beyond the currently modeled level/context fatals and switch-target warning,
  labels/goto inside unsupported functions, classes, and `try`/`finally`
  constructs, and exception/finally control-flow edges.
- Object `Traversable`, destructuring foreach targets, and PHP-exact
  `foreach` diagnostics outside the current array/non-array warning lane.
- PHP-exact include behavior beyond compile-time-resolved statement-only files,
  including dynamic paths, include paths, missing-file warning/return behavior,
  `include_once`/`require_once`, declaration-bearing include files, and return
  inside unsupported function/class contexts.
- Switch alternate syntax and switch behavior for arrays, objects, references,
  copy-on-write, and exceptions.
- PHP-exact increment/decrement semantics for null, booleans, strings, arrays,
  objects, references, copy-on-write, and diagnostics beyond the current
  integer/float direct-variable expression slice.
- Inline HTML before `<?php` or between PHP blocks.
- Remaining complex string interpolation forms, including object/property
  interpolation, variable variables, arbitrary expressions/calls, append
  offsets, and non-variable-root offsets.
- Heredoc interpolation, flexible indentation, and exact label diagnostics
  beyond the current plain heredoc/nowdoc string-literal slice.
- Internal functions outside the registered internal-function subset.
- Exact undefined-constant and unsupported-expression-statement diagnostics.
- Namespace/class constants, global `const` duplicate diagnostics and ordering
  parity with runtime `define()`, and built-in PHP/extension constants other
  than the currently modeled `E_*` error masks, `PHP_EOL`,
  `DIRECTORY_SEPARATOR`, `PATH_SEPARATOR`, `PHP_INT_MIN`, `PHP_INT_MAX`,
  `PHP_INT_SIZE`, `INF`, `NAN`, `M_PI`, and modeled PHP math `M_*` constants
  in `defined()`/`constant()`.
- Function forms beyond top-level named declarations and the public class-method
  callable slice, plus the bounded `stdClass` public-property storage slice,
  including array/object default arguments, named arguments outside direct
  generated user-function calls, by-reference returns, nested or conditional
  declarations, closures, old-style constructor dispatch, full class metadata,
  namespaces, globals, static locals, and remaining PHP-exact function return
  propagation.
- Type predicate coverage for full PHP resource and reference metadata beyond
  the current open-stream `is_resource()` slice.
- Unsupported recursive arrays, full class/object metadata, broad resources
  beyond the current stream slice, complete reference identity,
  copy-on-write, and `var_dump()` reference identity beyond the currently
  modeled ordered-array, direct-reference, and `stdClass` public-property
  behavior.
- `array_key_exists()` object property checks, references, and error-handler
  routing beyond the current ordered-array/resource-key slice.
- String-offset append, compound assignment, property/reference `isset()`/
  `empty()` and null-coalescing semantics, and complete TypeError/exception
  parity for unsupported string offset key types.
- Embedded NUL strings in runtime string values, `var_dump()` string
  length/output, `var_export()`, `strlen()`, `str_rot13()`, `strcmp()`,
  `bin2hex()`, `chr()`, `hex2bin()`, `str_contains()`, `quotemeta()`, `chunk_split()`,
  `strip_tags()`, `quoted_printable_decode()`, `addcslashes()`,
  `stripcslashes()`, `md5()`, `sha1()`, `substr()`, `soundex()`, `ord()`, or
  bitwise string results.
- Exact `strcmp()` resource/reference operand parity and object string
  conversion outside the current public declared `__toString()` support.
- Exact `str_contains()` resource/reference operand parity and object string
  conversion outside the current public declared `__toString()` support.
- Exact `str_starts_with()`/`str_ends_with()` resource/reference operand parity
  and object string conversion outside the current public declared
  `__toString()` support.
- Exact `quotemeta()` resource/reference operand parity and object string
  conversion outside the current public declared `__toString()` support.
- Exact `chunk_split()` non-positive length exception parity plus
  resource/reference operand parity and object string conversion outside the
  current public declared `__toString()` support.
- Exact `strip_tags()` binary-string behavior, allowed-tags argument support,
  malformed/incomplete tag parity, plus resource/reference operand parity and
  object string conversion outside the current public declared `__toString()`
  support.
- Exact `quoted_printable_decode()` embedded-NUL output behavior and
  resource/reference operand parity plus object string conversion outside the
  current public declared `__toString()` support.
- Exact `addcslashes()` invalid-range warning parity, malformed charlist edge
  cases, embedded-NUL/high-byte parity, plus resource/reference operand parity
  and object string conversion outside the current public declared
  `__toString()` support.
- Exact `stripcslashes()` embedded-NUL/high-byte parity plus resource/reference
  operand parity and object string conversion outside the current public
  declared `__toString()` support.
- Exact `addslashes()`/`stripslashes()` resource/reference operand parity and
  object string conversion outside the current public declared `__toString()`
  support.
- `md5()`/`sha1()` raw binary output containing NUL bytes, embedded-NUL input
  parity, plus resource/reference operand parity and object string conversion
  outside the current public declared `__toString()` support.
- Exact `substr()` binary-string behavior for embedded NUL bytes and
  resource/reference operand parity plus object string conversion outside the
  current public declared `__toString()` support.
- Exact `chr()` float-to-int precision-loss diagnostics.
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
- Exact `hex2bin()` warning text/file-name parity plus resource/reference
  operand parity and object string conversion outside the current public
  declared `__toString()` support.
- Exact `dirname()` object class-name TypeError wording and unsupported
  resource/reference operand parity.
- Exact `soundex()` locale/non-ASCII behavior plus resource/reference operand
  parity and object string conversion outside the current public declared
  `__toString()` support.
- Complete non-finite comparison parity for unsupported arrays, objects,
  resources, and references.
- Remaining PHP float precision and formatting edge cases plus
  conversion-diagnostic parity outside the current scalar stringification and
  `var_dump()` slices.
- Complete PHP CLI and PHPT runner option parity for `phpc`.
- Doc comment retention for reflection or metadata. Comments are skipped today.
- Complete PHP-exact `error_reporting()` coverage for diagnostic paths that
  still bypass the shared warning/deprecation/notice emitters.
- PHP-exact `getmypid()` process model parity across SAPIs and unsupported
  platforms.
- PHP-exact version, SAPI, and extension metadata beyond the modeled CLI/core/
  standard boundary.
- Cast spelling diagnostics beyond the currently modeled non-canonical aliases
  and removed `(real)`/`(unset)` plus expression-context `(void)` boundaries.
- Statement-form `(void) expr;` casts.
- Scope-aware magic constants inside traits, namespaces, includes, and eval
  contexts.
- PHP-exact file names, line numbers, custom error-handler routing, and
  overflow parity for remaining integer-only operator conversion diagnostics,
  including shift and modulo diagnostics.
- Object lvalues, dynamic-variable array-offset compound/null-coalescing/
  by-reference lvalues, append-form null-coalescing, property null-coalescing
  expressions/`isset()`/`empty()`, property compound-assignment operators
  outside modeled public-property `??=`, and static-property
  compound/null-coalescing lvalues outside modeled direct reads/writes.
- Remaining reference semantics for compound assignment outside direct
  variables and modeled array elements, including full copy-on-write
  interactions and by-reference visibility during writes.
- Arrays, references, copy-on-write, globals, superglobals, classes, objects,
  resources, exceptions, variable variables, dynamic includes, and dynamic
  fallback.
