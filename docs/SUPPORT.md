# Support

This file tracks user-visible PHP behavior that the scratch compiler currently
supports in generated native binaries.

## RC Boundary

The release-candidate surface is the generic compiler/runtime path exercised by
`examples/rc`: scalar control flow, strings, ordered arrays, selected standard
internals, top-level functions, includes resolved at compile time, copy-on-write
array/reference slices, and public class/object shells. Public class support is
bounded to top-level declarations with public methods, direct public static
property reads/writes, read-side static property `isset()`/`empty()`/`??`
probes, direct static-property `??=` and compound assignment, public instance
property reads/writes, public property `??=` and compound assignment,
read-side property `isset()`/`empty()`/`??` probes, and
bounded private instance-property declarations read/written from
declaring-class methods, declared private/protected instance-property metadata
for initialization, dump labels, and `property_exists()` checks, inherited
static method calls through declared class names, and public non-static
`__construct` dispatch through the declared-method path.

Post-RC architecture remains explicit rather than hidden:

- Classes and inheritance: interfaces, traits, full visibility-aware/typed/
  promoted properties, broad metadata/reflection, destructors, old-style
  constructors, class constants, and complete inherited property/method
  resolution beyond the current declared-method and metadata slices remain
  outside the RC boundary.
- Namespaces: unbracketed namespace declarations, qualified names, and simple
  plus grouped class/function/constant imports are supported for the current
  top-level function/constant and declared-class subset. Bracketed namespace
  blocks, namespace fallback parity for arbitrary userland symbols,
  namespace/class constants, and namespace-sensitive reflection remain post-RC.
- Static properties: direct public static reads/writes, compound assignment,
  and `??=` are supported, and `property_exists()` can inspect the current
  declared static property metadata. Visibility, late static binding,
  typed/default metadata, and broader static-property lvalues are post-RC.
- Magic methods: public declared instance `__construct` is supported during
  object construction, and public declared instance `__call` is supported as a
  fallback for direct object calls and supported object callable dispatch when
  no declared method matches. Public declared instance `__toString` is
  supported for current runtime string conversions. Public declared instance
  `__invoke` is supported for direct object callable values and internal
  callback dispatch. `__destruct`, `__get`, `__set`, `__isset`, `__unset`,
  `__callStatic`, and related hooks remain unsupported.
- Non-static callables: direct object method calls and bounded
  `[$object, "method"]` callback dispatch work for declared and inherited
  public methods, and missing object methods fall through to supported
  `__call`. Objects with inherited public `__invoke` can be called directly or
  through supported internal callbacks. `is_callable()` validates the current
  string, closure, `["Class", "staticMethod"]`, `[$object, "method"]`, and
  invokable-object subset, including `__call`-capable objects and syntax-only
  checks. First-class callable syntax, non-public visibility, `__callStatic`,
  and arbitrary dynamic instance method metadata remain post-RC.
- Object destructuring and object `Traversable` remain unsupported; current
  destructuring support is array/list lvalues.
- Property lvalues remain post-RC outside public property `??=`, modeled
  property/static-property inc/dec, and direct property/static-property
  compound assignment: nested/dynamic property lvalues and broader static
  forms remain unsupported.

## Supported

- `<?php` open tag.
- A Unix shebang at byte 0 before `<?php`.
- PHP `//`, `#`, and `/* ... */` comments inside PHP code. One-line
  comments end at a newline or at a trailing `?>` close tag.
- A `?>` close tag that ends PHP mode and emits following inline output, with
  one immediately following newline swallowed. Inline HTML before the first
  open tag and between PHP blocks is emitted through the same native output
  path.
- Global-scope and unbracketed namespace-scope `const NAME = expr;`
  declarations for the currently supported constant-expression subset.
  Declared constants are visible to bare constant reads, `defined()`, and
  `constant()` under their resolved names. Duplicate declarations emit the
  modeled warning boundary and preserve the original value.
- Unbracketed `namespace Name\Parts;` declarations establish the lexical
  namespace for subsequent top-level declarations and statements.
  `__NAMESPACE__` yields that lexical namespace string. The namespace
  declaration must be the first declaration-bearing statement in the file.
- Simple `use Name\Parts as Alias;`, `use function Name\Parts as alias;`, and
  `use const Name\Parts as ALIAS;` declarations, plus grouped
  `use Prefix\{Name as Alias, function helper, const VALUE};` forms, resolve
  aliases for the current namespace. Class imports apply to unqualified names
  and to the first segment of qualified class names; function and constant
  imports apply to unqualified calls/reads in the current supported subset.
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
- Variable-variable reads, ordinary assignments, and simple array/string-offset
  writes with `$$name`, `$$$name`, and `${expr}` over scalar runtime names.
  Dynamic names are converted through the shared scalar string-conversion path
  for `null`, booleans, integers, floats, and strings; arrays, objects,
  closures, exceptions, and embedded-NUL string names stop at an explicit
  unsupported-name diagnostic. Dynamic-root array/string-offset writes such as
  `${$name}[$key] = $value` evaluate the dynamic name and offset expressions
  before the right-hand side, then reuse the shared array-path write helper.
  Dynamic-root array/string-offset compound assignments such as
  `${$name}[$key] += $value` evaluate the dynamic name and offset expressions
  before the right-hand side, then reuse the shared array-path assign-op read
  and write helpers.
  Dynamic-variable null coalescing assignments such as `${$name} ??= $value`
  and `${$name}[$key] ??= $value` evaluate the dynamic name and offset
  expressions before the right-hand side, quiet-read the target, lazily
  evaluate the right-hand side only for missing or `null` targets, and reuse
  the shared variable and array-path write helpers.
  Dynamic-root array/string-offset unsets such as `unset(${$name}[$key])`
  evaluate the dynamic name and offset expressions through the same path-unset
  helper used by direct array/string-offset unsets.
- Assignment expressions for direct variables, array dimension/append lvalues,
  and list/short-array destructuring targets, including by-reference
  destructuring entries in the modeled reference-array subset.
- Assignment expressions in branch and loop conditions for the currently
  supported expression-assignment subset. Direct-variable and variable-root
  array/append compound condition assignments evaluate the target once, read
  the current value through the same assign-op lookup path as statements,
  evaluate the right-hand side, write the computed value, and branch on that
  assigned value's PHP truthiness.
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
- Direct named-variable and variable-root array/append compound assignment for
  `+=`, `-=`, `*=`, `/=`, `%=`, `**=`, `.=`, `&=`, `|=`, `^=`, `<<=`, and
  `>>=`. The compiler lowers these as `read target`, the matching boxed binary
  helper, then `write target`. Array dimensions are evaluated once before the
  right-hand expression and use the shared array-path assign-op helpers for
  both statement and expression forms.
- Direct property and static-property compound assignment for the same
  operators over the current modeled read/write surface. The receiver or class
  and property name are evaluated once before the right-hand expression; the
  compiler then reads the current slot, applies the boxed binary helper, writes
  the assigned value back, and returns that assigned value.
- Direct variable and variable-root array assignment statements followed by
  keyword boolean `and`, `or`, or `xor` tails are parsed with PHP precedence:
  the assignment is evaluated first as the left operand, then the existing
  short-circuit or XOR boolean path evaluates the tail.
- Direct and dynamic named-variable null coalescing assignment `??=`. The
  compiler uses the same quiet lookup path as expression-form `??`, writes only
  when the variable is missing or `null`, and evaluates the right-hand
  expression lazily.
- Direct and dynamic-root keyed array and string offset null coalescing
  assignment `??=`. Offset keys are evaluated once, the read side is quiet like
  `??`, and existing array/string write helpers perform the conditional store.
  Append-form `$a[] ??= ...` and `${$name}[] ??= ...` are rejected because PHP
  must read the target before assigning.
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
- Statement-form `(void) expr;` casts evaluate the operand for side effects and
  discard the result.
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
- Simple full ternaries `condition ? if_true : if_false` and short ternaries
  `condition ?: if_false` evaluate the condition once, only evaluate the
  selected branch, and return boxed branch values through the shared value
  path.
- Unparenthesized nested ternary expressions are rejected with
  PHP-style source-spanned fatal diagnostics for the currently modeled
  forbidden associativity forms.
- Magic constants `__LINE__`, `__FILE__`, and `__DIR__`, plus global-scope
  `__FUNCTION__`, `__METHOD__`, `__CLASS__`, and `__TRAIT__` empty-string
  behavior. `__NAMESPACE__` exposes the current unbracketed lexical namespace.
  Top-level functions expose `__FUNCTION__` and `__METHOD__`; declared class
  methods expose `__FUNCTION__`, `__METHOD__`, and `__CLASS__` for the current
  method scope. Traits, includes, and eval remain outside this boundary.
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
  first byte. String-offset compound assignment evaluates its key through the
  same string-offset conversion path before raising the modeled PHP `Error`.
  Direct and nested attempts to unset string offsets throw the modeled PHP
  `Error` and leave the string unchanged.
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
  `array_shift($array)`, `array_unshift($array, ...)`, `asort($array)`,
  `arsort($array)`, `ksort($array)`, `krsort($array)`, `sort($array)`,
  `rsort($array)`, `natsort($array)`, `natcasesort($array)`, and
  `shuffle($array)` over direct variable ordered arrays.
  One-argument
  `array_pop()` and `array_shift()` also support variable-root array paths such
  as `$items[0]`, detaching shared nested array payloads before mutation.
  `asort()`/`arsort()` use default ascending/descending value order while
  preserving keys, `ksort()`/`krsort()` use default ascending/descending key
  order, `sort()`/`rsort()` use default ascending/descending value order and
  reindex values with integer keys, `natsort()`/`natcasesort()` use natural
  string value order while preserving keys, with `natcasesort()` comparing
  ASCII letters case-insensitively, and `shuffle()` reindexes values with
  integer keys.
  Direct `sort()`/`rsort()`/`asort()`/`arsort()`/`ksort()`/`krsort()` calls
  accept omitted flags, `SORT_REGULAR`, or literal `0`, all of which use the
  same regular-sort helper path.
  Temporary arrays, array paths for variadic
  `array_push()`/`array_unshift()`, array paths for `asort()`/`ksort()`/
  `sort()`/`arsort()`/`krsort()`/`rsort()`/`natsort()`/`natcasesort()`/
  `shuffle()`, and other non-direct-variable mutation targets fail before code
  generation with an explicit unsupported diagnostic.
- Other sort flags, `natsort()` extra flags, dynamic flagged sort-family calls,
  and remaining sort-family by-reference array mutators such as `usort()`,
  `uasort()`, `uksort()`, and `array_multisort()` remain unsupported and fail
  with an explicit unsupported diagnostic.
- `isset(expr[, ...])` and `empty(expr)` over variables, array reads, string
  offset reads, property reads, static property reads, and currently supported
  value expressions. Variable, offset, property, and static-property operands
  use a quiet existence lookup: missing variables, missing offsets, missing
  properties, missing declared-class static properties, non-array/non-object
  containers, and out-of-range string offsets do not emit ordinary read
  warnings; `isset()` returns false for missing or `null` values, and `empty()`
  returns true for missing or PHP-falsey values.
- Expression-form null coalescing `left ?? right` over direct variables, array
  reads, string offset reads, property reads, static property reads, and
  currently supported value expressions. The left operand uses the same quiet
  lookup path as
  `isset()`/`empty()`, returns present non-`null` values without evaluating the
  right operand, and evaluates the right operand only for missing or `null`
  left values.
- Full ternary `condition ? if_true : if_false` and short ternary
  `condition ?: if_false` expressions over the current boxed expression subset.
  Conditions use PHP truthiness, only the selected arm is evaluated, and short
  ternary evaluates the condition expression once before reusing it when truthy.
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
  `strcasecmp(expr, expr);`,
  `strncmp(expr, expr, expr);`, `strncasecmp(expr, expr, expr);`,
  `str_contains(expr, expr);`,
  `str_starts_with(expr, expr);`,
  `strpos(expr, expr[, expr]);`, `stripos(expr, expr[, expr]);`,
  `strrpos(expr, expr[, expr]);`, `strripos(expr, expr[, expr]);`,
  `strstr(expr, expr[, expr]);`, `stristr(expr, expr[, expr]);`,
  `substr_count(expr, expr[, expr[, expr]]);`, `strpbrk(expr, expr);`,
  `str_ends_with(expr, expr);`, `str_pad(expr, expr[, expr[, expr]]);`,
  `str_repeat(expr, expr);`, `str_split(expr[, expr]);`,
  `strtolower(expr);`, `strtoupper(expr);`, `strrchr(expr, expr[, expr]);`,
  `strrev(expr);`, `ucfirst(expr);`,
  `lcfirst(expr);`, `quotemeta(expr);`,
  `trim(expr[, expr]);`, `ltrim(expr[, expr]);`, `rtrim(expr[, expr]);`,
  `chop(expr[, expr]);`,
  `chunk_split(expr[, expr[, expr]]);`, `nl2br(expr[, use_xhtml]);`,
  `strip_tags(expr);`,
  `explode(expr, expr[, expr]);`,
  `join(expr[, expr]);`, `implode(expr[, expr]);`, `sprintf(expr, ...);`,
  `printf(expr, ...);`, `json_encode(expr[, expr[, expr]]);`,
  `crc32(expr);`,
  `md5(expr[, raw_output]);`,
  `sha1(expr[, raw_output]);`, `substr(expr, expr[, expr]);`, `bin2hex(expr);`,
  `hex2bin(expr);`, `quoted_printable_decode(expr);`, `dirname(expr[, levels]);`,
  `highlight_string(expr[, return]);`, `highlight_file(expr[, return]);`,
  `soundex(expr);`, `ceil(expr);`, `floor(expr);`, `abs(expr);`, `sqrt(expr);`,
  `pow(expr, expr);`, `fdiv(expr, expr);`, `intdiv(expr, expr);`, `bindec(expr);`,
  `hexdec(expr);`, `octdec(expr);`, `pi();`, `getrandmax();`,
  `getmypid();`, `getcwd();`, `chdir(expr);`, `get_cfg_var(expr);`,
  `get_loaded_extensions([zend_extensions]);`, `ini_get(expr);`,
  `localeconv();`,
  `ob_get_contents();`, `php_ini_scanned_files();`, `php_sapi_name();`,
  `php_uname([mode]);`, `phpversion([extension]);`, `preg_match(expr, expr);`,
  `realpath(expr);`, `scandir(expr[, sorting_order[, context]]);`,
  `setlocale(expr, expr[, ...]);`, `str_replace(expr, expr, expr[, count]);`,
  `zend_version();`,
  `boolval(expr);`, `floatval(expr);`, `doubleval(expr);`, `intval(expr);`,
  `chr(expr);`, `ord(expr);`,
  `count(expr[, mode]);`, `sizeof(expr[, mode]);`,
  `array_chunk(expr, expr[, expr]);`,
  `array_change_key_case(expr[, expr]);`, `array_column(expr, expr[, expr]);`,
  `array_combine(expr, expr);`, `array_count_values(expr);`,
  `array_fill(expr, expr, expr);`, `array_pad(expr, expr, expr);`,
  `array_filter(expr[, expr[, expr]]);`,
  `array_keys(expr[, expr[, expr]]);`,
  `array_map(expr, expr[, ...]);`, `array_reduce(expr, expr[, expr]);`,
  `array_walk(expr, expr[, expr]);`,
  `array_intersect(expr, ...);`, `array_intersect_assoc(expr, ...);`,
  `array_is_list(expr);`, `array_key_first(expr);`, `array_key_last(expr);`,
  `array_udiff(expr, expr, callback);`,
  `array_udiff_assoc(expr, expr, callback);`,
  `array_udiff_uassoc(expr, expr, callback, callback);`,
  `array_product(expr);`, `array_search(expr, expr[, expr]);`,
  `array_slice(expr, expr[, expr[, expr]]);`,
  `array_values(expr);`, `array_keys(expr[, expr[, expr]]);`,
  `array_merge(expr, ...);`,
  `array_merge_recursive(expr, ...);`,
  `array_replace(expr, ...);`,
  `array_replace_recursive(expr, ...);`,
  `call_user_func(expr[, ...]);`, `call_user_func_array(expr, expr);`,
  `assert(expr[, description]);`,
  `in_array(expr, expr[, expr]);`,
  `is_callable(expr[, syntax_only]);`, `spl_object_id(expr);`,
  `spl_object_hash(expr);`, `is_finite(expr);`,
  `is_countable(expr);`, `is_iterable(expr);`, `is_infinite(expr);`,
  `is_nan(expr);`, and
  `error_reporting(expr);`.
- Expression-form internal calls for the currently registered functions,
  including `var_export(expr[, return])`, `print_r(expr[, return])`,
  `strlen(expr)`, `addcslashes(expr, expr)`,
  `stripcslashes(expr)`, `addslashes(expr)`, `stripslashes(expr)`,
  `str_rot13(expr)`, `str_shuffle(expr)`, `strcmp(expr, expr)`,
  `strcasecmp(expr, expr)`,
  `strncmp(expr, expr, expr)`, `strncasecmp(expr, expr, expr)`,
  `str_contains(expr, expr)`,
  `str_starts_with(expr, expr)`,
  `strpos(expr, expr[, expr])`, `stripos(expr, expr[, expr])`,
  `strrpos(expr, expr[, expr])`, `strripos(expr, expr[, expr])`,
  `strstr(expr, expr[, expr])`, `stristr(expr, expr[, expr])`,
  `substr_count(expr, expr[, expr[, expr]])`, `strpbrk(expr, expr)`,
  `str_ends_with(expr, expr)`, `str_pad(expr, expr[, expr[, expr]])`,
  `str_repeat(expr, expr)`, `str_split(expr[, expr])`,
  `strtolower(expr)`, `strtoupper(expr)`, `strrchr(expr, expr[, expr])`,
  `strrev(expr)`, `ucfirst(expr)`,
  `lcfirst(expr)`, `quotemeta(expr)`,
  `trim(expr[, expr])`, `ltrim(expr[, expr])`, `rtrim(expr[, expr])`,
  `chop(expr[, expr])`,
  `chunk_split(expr[, expr[, expr]])`, `nl2br(expr[, use_xhtml])`,
  `strip_tags(expr)`,
  `explode(expr, expr[, expr])`,
  `join(expr[, expr])`, `implode(expr[, expr])`, `sprintf(expr, ...)`,
  `printf(expr, ...)`, `json_encode(expr[, expr[, expr]])`,
  `crc32(expr)`,
  `md5(expr[, raw_output])`,
  `sha1(expr[, raw_output])`, `substr(expr, expr[, expr])`, `bin2hex(expr)`,
  `hex2bin(expr)`, `quoted_printable_decode(expr)`,
  `basename(expr[, suffix])`, `dirname(expr[, levels])`,
  `pathinfo(expr[, flags])`,
  `highlight_string(expr[, return])`, `highlight_file(expr[, return])`,
  `soundex(expr)`, `ceil(expr)`, `floor(expr)`,
  `abs(expr)`, `sqrt(expr)`, `pow(expr, expr)`, `fdiv(expr, expr)`, `intdiv(expr, expr)`, `bindec(expr)`,
  `hexdec(expr)`, `octdec(expr)`, `pi()`, `getrandmax()`,
  `getmypid()`, `getcwd()`, `chdir(expr)`, `get_cfg_var(expr)`,
  `get_loaded_extensions([zend_extensions])`, `ini_get(expr)`,
  `localeconv()`,
  `ob_get_contents()`, `php_ini_scanned_files()`, `php_sapi_name()`,
  `php_uname([mode])`, `phpversion([extension])`, `preg_match(expr, expr)`,
  `realpath(expr)`, `scandir(expr[, sorting_order[, context]])`,
  `setlocale(expr, expr[, ...])`,
  `str_replace(expr, expr, expr[, count])`, `zend_version()`,
  `boolval(expr)`, `floatval(expr)`, `doubleval(expr)`, `intval(expr)`,
  `chr(expr)`, `ord(expr)`,
  `count(expr[, mode])`, `sizeof(expr[, mode])`,
  `array_chunk(expr, expr[, expr])`,
  `array_change_key_case(expr[, expr])`, `array_column(expr, expr[, expr])`,
  `array_combine(expr, expr)`, `array_count_values(expr)`,
  `array_fill(expr, expr, expr)`, `array_pad(expr, expr, expr)`,
  `array_filter(expr[, expr[, expr]])`,
  `array_keys(expr[, expr[, expr]])`,
  `array_map(expr, expr[, ...])`, `array_reduce(expr, expr[, expr])`,
  `array_walk(expr, expr[, expr])`,
  `array_intersect(expr, ...)`, `array_intersect_assoc(expr, ...)`,
  `array_is_list(expr)`, `array_key_first(expr)`, `array_key_last(expr)`,
  `array_udiff(expr, expr, callback)`,
  `array_udiff_assoc(expr, expr, callback)`,
  `array_udiff_uassoc(expr, expr, callback, callback)`,
  `array_product(expr)`, `array_search(expr, expr[, expr])`,
  `array_slice(expr, expr[, expr[, expr]])`,
  `array_values(expr)`, `array_keys(expr[, expr[, expr]])`,
  `array_merge(expr, ...)`,
  `array_merge_recursive(expr, ...)`, `array_replace(expr, ...)`,
  `array_replace_recursive(expr, ...)`,
  `call_user_func(expr[, ...])`, `call_user_func_array(expr, expr)`,
  `assert(expr[, description])`,
  `in_array(expr, expr[, expr])`,
  `fopen(expr, expr[, expr[, expr]])`, `fclose(expr)`,
  `stream_get_meta_data(expr)`,
  `file_get_contents(expr[, use_include_path[, context[, offset[, length]]]])`,
  `is_callable(expr[, syntax_only])`, `is_countable(expr)`, `is_iterable(expr)`, `is_finite(expr)`,
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
  and minimal scalar/array/`null` parameter and return type declarations plus
  return-only `void` declarations over the currently supported expression and
  statement subset. Direct calls may omit defaulted trailing arguments, pass
  extra positional arguments, or use named arguments for direct
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
- `global` statements bind function-local names to the root global symbol table,
  so direct and callback-dispatched user functions can read, write, and return
  modeled global variables by reference.
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
  ordered-array values, declared object shells through
  `\Class::__set_state(array(...))`, and `stdClass` through `(object)
  array(...)`, including nested arrays/objects, embedded-NUL string escaping as
  single-quoted segments concatenated with `"\0"`, and string-return mode
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
- `printf()` shares the bounded `sprintf()` formatter, writes the formatted
  bytes to stdout, and returns the emitted byte length.
- `json_encode()` covers current scalar values, strings with JSON escaping,
  list arrays, associative arrays, and public object properties. Unsupported
  runtime-only values and excessive recursion return `false`; option flags are
  not yet modeled.
- Shared string-argument checking for common string/byte internals: supported
  scalar values and objects with a public declared `__toString()` are coerced
  through the same length-aware operand path, `null` arguments emit the modeled
  deprecation, and arrays, objects without the current `__toString()` support,
  closures, and exceptions throw the modeled `TypeError` boundary, including
  object class names and `Closure` in the reported given-type, for `strlen()`,
  `str_rot13()`, `str_shuffle()`, `strcmp()`, `strcasecmp()`, `strncmp()`,
  `strncasecmp()`, `str_contains()`,
  `str_starts_with()`, `str_ends_with()`, `str_repeat()`, `str_split()`,
  three-argument `strtr()`, `strrchr()`, `strrev()`, `str_pad()`,
  `ucfirst()`, `lcfirst()`,
  `strtolower()`,
  `strtoupper()`, `quotemeta()`, `chunk_split()` string/separator arguments,
  `nl2br()`, `strpbrk()` string/characters arguments,
  `explode()` separator/string arguments,
  `trim()`/`ltrim()`/`rtrim()`/`chop()` string and characters arguments,
  `strip_tags()`, `crc32()`, `md5()`, `sha1()`, `substr()`,
  `addcslashes()`, `addslashes()`, `stripcslashes()`, `stripslashes()`,
  `bin2hex()`, `hex2bin()`, `quoted_printable_decode()`, `soundex()`, and
  `ord()`.
- `str_rot13()` over current boxed scalar values after scalar string conversion,
  returning ASCII ROT13 output while leaving non-letters unchanged.
- `str_shuffle()` over current boxed scalar values after scalar string
  conversion, returning a byte-shuffled string without mutating the input.
- `str_split()` over current boxed scalar values after scalar string
  conversion, chunking binary-safe bytes into an ordered array. The length
  argument uses the shared scalar integer path and throws the modeled
  `ValueError` boundary for non-positive lengths.
- `strcmp()` over current boxed scalar values after scalar string conversion,
  returning a negative integer, zero, or a positive integer from bytewise
  comparison of the current C-string-backed values.
- `strcasecmp()` over current boxed scalar values after scalar string
  conversion, returning a negative integer, zero, or a positive integer from
  bytewise comparison after ASCII-only case folding.
- `strncmp()` over current boxed scalar values after scalar string conversion,
  with the length argument converted through the shared scalar integer path.
  It compares at most the requested byte prefix length, preserves embedded NUL
  bytes, returns a normalized negative/zero/positive integer, and throws the
  modeled `ValueError` boundary for negative lengths.
- `strncasecmp()` over current boxed scalar values after scalar string
  conversion, with the length argument converted through the shared scalar
  integer path. It compares at most the requested byte prefix length after
  ASCII-only case folding, preserves embedded NUL bytes, returns a normalized
  negative/zero/positive integer, and throws the modeled `ValueError` boundary
  for negative lengths.
- `str_contains()` over current boxed scalar values after scalar string
  conversion, returning whether the needle string is present in the haystack
  string through the current C-string-backed value path.
- `strpos()`/`stripos()` and `strrpos()`/`strripos()` over current boxed scalar
  values after scalar string conversion, returning byte offsets or `false`.
  Optional offsets are converted through the typed integer argument path and
  enforce PHP's haystack-contained bounds.
- `strstr()`/`stristr()` over current boxed scalar values after scalar string
  conversion, returning the matched suffix or the optional before-needle prefix
  through length-aware byte slices.
- `substr_count()` over current boxed scalar values after scalar string
  conversion, counting non-overlapping byte matches across optional offset and
  nullable length bounds.
- `str_starts_with()` and `str_ends_with()` over current boxed scalar values
  after scalar string conversion, returning whether the haystack has the
  requested prefix or suffix through the current C-string-backed value path.
- `strrchr()` over current boxed scalar values after scalar string conversion,
  returning the suffix from the last occurrence of the first needle byte, or
  the prefix before that byte when the optional flag is truthy.
- `str_pad()` over current boxed scalar values after scalar string conversion,
  padding by byte length with optional pad string and `STR_PAD_LEFT`,
  `STR_PAD_RIGHT`, or `STR_PAD_BOTH` mode constants. Empty pad strings and
  invalid pad modes throw the modeled `ValueError` boundary.
- `str_repeat()` over current boxed scalar values after scalar string
  conversion, repeating the input by a count converted through the current
  scalar integer conversion path and rejecting negative counts with the modeled
  `ValueError` boundary.
- `strtolower()` and `strtoupper()` over current boxed scalar values after
  scalar string conversion, mapping ASCII letters and preserving other bytes.
- `strrev()` over current boxed scalar values after scalar string conversion,
  reversing bytes while preserving explicit string length and embedded NULs.
- `ucfirst()` and `lcfirst()` over current boxed scalar values after scalar
  string conversion, mapping the initial ASCII byte to upper/lowercase and
  preserving remaining bytes.
- `trim()`, `ltrim()`, `rtrim()`, and `chop()` over current boxed scalar values
  after scalar string conversion. `chop()` shares the `rtrim()` right-trim
  semantics. The default PHP trim bytes are modeled, and the optional
  characters argument supports literal byte sets plus ascending `a..z`-style
  byte ranges over the current length-aware string path.
- `quotemeta()` over current boxed scalar values after scalar string
  conversion, prefixing `.`, `\`, `+`, `*`, `?`, `[`, `^`, `]`, `(`, `$`, and
  `)` bytes with backslashes through the current C-string-backed value path.
- `chunk_split()` over current boxed scalar values after scalar string
  conversion, using a chunk length converted through the current scalar integer
  conversion path and an ending converted through the current scalar string
  conversion path. The defaults are length `76` and ending `"\r\n"`. Input and
  ending bytes are length-aware, including embedded NUL bytes, and empty input
  returns the ending string like PHP.
- `explode()` over current boxed scalar separator and string values after
  scalar string conversion, returning ordered arrays of length-aware string
  segments. Empty separators throw the modeled PHP `ValueError`; positive,
  zero, and negative limits follow PHP's bounded split and tail-dropping
  behavior.
- `strip_tags()` over current boxed scalar values after scalar string
  conversion, removing complete `<...>`, `<?...?>`, `<%...%>`, and HTML
  comment tag regions through the current C-string-backed value path.
- `crc32()` over current boxed scalar values after scalar string conversion,
  returning the unsigned CRC-32 checksum in the current integer value and
  preserving embedded NUL bytes through the length-aware string path.
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
  returning the parent directory for the current binary-safe path and optional
  positive `levels` count. Empty paths, embedded NUL bytes, platform directory
  separators, null-argument deprecations, invalid-level `ValueError`s, and
  modeled TypeError diagnostics for array/object operands are handled through
  the runtime path.
- `pathinfo()` over current boxed scalar values after scalar string conversion,
  returning ordered dirname/basename/extension/filename arrays or individual
  components selected by `PATHINFO_*` flags. Empty paths, trailing separators,
  embedded NUL bytes, invalid flag `ValueError`s, null-argument deprecations,
  and modeled TypeError diagnostics for array/object operands are handled
  through the runtime path.
- `soundex()` over current boxed scalar values after scalar string conversion,
  returning a PHP-style four-character ASCII soundex key.
- `str_replace()` over current scalar, stringable-object, and ordered-array
  search/replacement/subject operands after PHP string conversion. Search
  arrays are applied sequentially, replacement arrays use the matching entry or
  an empty string when missing, scalar replacements apply to every search
  entry, reference-backed array entries are dereferenced, subject arrays
  preserve their keys, and the optional `$count` argument accumulates
  replacements by reference. Invalid resource, closure, exception,
  non-stringable object operands, and scalar-search/array-replace calls throw
  catchable PHP-style `TypeError` diagnostics.
- `nl2br()` over current boxed scalar values after scalar string conversion,
  inserting `<br />` by default or `<br>` when the optional XHTML flag is false
  before `\n`, `\r`, `\r\n`, and `\n\r` newline sequences.
- `strpbrk()` over current boxed scalar values after scalar string conversion,
  returning the binary-safe suffix beginning at the first byte present in the
  `characters` set, `false` when no byte matches, and a catchable `ValueError`
  when `characters` is empty.
- `getcwd()` returns the process current working directory as a string, and
  `chdir()` changes it for scalar path operands. Embedded NUL paths return
  `false` with the current warning boundary; failing host calls return `false`
  with the shared filesystem warning path.
- `ceil()` and `floor()` over current boxed scalar values after PHP numeric
  parameter conversion, returning boxed floats. `null` emits the modeled
  deprecation and yields `0.0`; booleans, integers, floats, fully numeric
  decimal strings, and overflowing numeric strings are accepted. Empty strings,
  non-numeric strings, leading-numeric strings with trailing junk, C-style hex
  strings, arrays, objects, closures, exceptions, resources, and references
  throw catchable `TypeError` values.
- `abs()` over current boxed scalar values after scalar numeric conversion,
  returning boxed integer or float magnitudes and emitting the modeled
  PHP null-deprecation boundary; unsupported operands and malformed numeric
  strings raise catchable `TypeError` diagnostics.
- `sqrt()` over current boxed scalar values after scalar numeric conversion,
  returning a boxed float; unsupported operands and malformed numeric strings
  raise catchable `TypeError` diagnostics.
- `fdiv()` over current boxed scalar values after scalar numeric conversion,
  returning boxed floating-point division results, including zero divisors,
  signed zeroes, infinities, and `NAN`; unsupported operands and malformed
  numeric strings raise catchable `TypeError` diagnostics.
- `intdiv()` over current boxed scalar values after scalar integer conversion,
  returning a boxed integer quotient for supported non-zero divisors. Zero
  divisors throw catchable `DivisionByZeroError` values, and the
  `PHP_INT_MIN / -1` overflow edge throws a catchable `ArithmeticError`.
  Unsupported array, object, closure, exception, and resource operands throw
  catchable `TypeError` values with PHP-style integer parameter diagnostics.
- `pi()` returns the modeled boxed float value of the `M_PI` constant.
- `pow()` calls the same boxed numeric exponentiation helper as the `**`
  operator.
- `getrandmax()` returns the modeled maximum random integer.
- `getmypid()` returns the generated native process id.
- `php_sapi_name()` and the `PHP_SAPI` constant return the modeled CLI SAPI
  name.
- `phpversion()` and the `PHP_VERSION`, `PHP_MAJOR_VERSION`,
  `PHP_MINOR_VERSION`, `PHP_RELEASE_VERSION`, `PHP_EXTRA_VERSION`,
  `PHP_VERSION_ID`, `PHP_ZTS`, and `PHP_DEBUG` constants return modeled PHP
  version/build metadata. The optional extension argument returns the same
  version for `core`, `date`, `pcre`, `reflection`, `standard`, and an empty
  extension name, and `false` for unmodeled extension names.
- `zend_version()` returns the modeled Zend Engine version string.
- `get_loaded_extensions()` returns the modeled loaded extension names
  `Core`, `date`, `pcre`, `reflection`, and `standard`;
  `get_loaded_extensions(true)` returns an empty array because Zend extensions
  are outside the current runtime boundary.
- Locale category constants `LC_ALL`, `LC_COLLATE`, `LC_CTYPE`,
  `LC_MESSAGES`, `LC_MONETARY`, `LC_NUMERIC`, and `LC_TIME` are available
  through the modeled constant registry with stable PHP values. `setlocale()`
  maps those values to the platform C locale API for current scalar categories
  and scalar/array locale candidates; locale `"0"` queries the current
  category, `"C"`/`"POSIX"` use the stable C locale, and unavailable locale
  names return `false`.
  `localeconv()` returns the current C `struct lconv` fields as an ordered PHP
  array, including grouping arrays.
- `bindec()`, `hexdec()`, and `octdec()` over current boxed scalar values after
  scalar string conversion. The runtime accepts matching `0b`, `0x`, and `0o`
  prefixes, ignores invalid base digits with a deprecation boundary, and
  returns integers until the parsed value exceeds native integer range, then
  floats.
- `boolval()` over current boxed values through the shared PHP truthiness
  helper.
- `floatval()` and `doubleval()` over current boxed scalar and array values
  through the shared scalar numeric conversion path.
- `intval()` over current boxed scalar values after scalar integer conversion,
  with bounded string/base conversion for supported bases, including PHP-style
  `0x`/`0b` prefix handling and integer-range saturation.
- `chr()` over current boxed scalar values after scalar integer conversion,
  returning a one-byte string with byte values constrained modulo 256 and
  emitting the modeled out-of-range deprecation for integers outside
  `[0, 255]`. Null operands emit the modeled PHP parameter deprecation and
  convert to `0`; finite float and float-string operands that lose precision
  during integer conversion emit the modeled call-site deprecation and respect
  `error_reporting()`/`@` suppression. Non-finite floats and unsupported
  array/object/closure/exception/resource operands throw catchable `TypeError`
  values through the shared integer-argument validation path.
- `ord()` over current boxed scalar values after scalar string conversion,
  returning the first byte as an integer. Empty and multi-byte strings emit
  PHP-like deprecation diagnostics with the internal-call source line.
- `count()` and `sizeof()` over current boxed arrays, returning their length as
  an integer in `COUNT_NORMAL` mode and recursively counting nested arrays in
  `COUNT_RECURSIVE` mode. Recursive cycles emit the modeled recursion warning
  and do not descend further.
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
- `array_pad()` over current boxed arrays, integer-convertible lengths, and
  mixed pad values. No-op padding clones the original ordered map, positive
  padding appends cloned pad values, negative padding prepends cloned pad
  values, and source integer keys are reindexed only when padding occurs while
  string keys stay ordered and preserved. Oversized requested lengths throw the
  modeled PHP `ValueError`.
- `array_filter()` over current boxed arrays, preserving original keys in a
  fresh ordered result. With a `null` callback it keeps values that are truthy
  under the shared PHP truthiness helper; otherwise it dispatches through the
  shared callable path with value, key, or value/key arguments according to
  `ARRAY_FILTER_USE_KEY` and `ARRAY_FILTER_USE_BOTH`, and rejects unknown modes
  with the modeled PHP `ValueError`.
- `array_map()` over current boxed arrays, dispatching through the shared
  callable path for one or more input arrays. A `null` callback returns current
  zipped row values. Single-array calls preserve source keys; multi-array calls
  use sequential integer keys.
- `array_reduce()` over current boxed arrays, dispatching carry/value pairs
  through the shared callable path and supporting an optional initial value.
- `array_walk()` over direct-variable arrays, dispatching value/key pairs and
  optional user data through the shared callable path while preserving PHP's
  by-reference mutation behavior for walked values.
- `array_flip()` over current boxed arrays, flipping dereferenced integer and
  string values into ordered-map keys and using the original keys as values.
  Unsupported value types emit the modeled PHP warning boundary and are skipped.
- `array_is_list()` over current boxed arrays, returning whether ordered keys
  are exactly integer `0..n-1` in insertion order. Non-array operands throw the
  modeled catchable PHP `TypeError`.
- `array_key_first()` and `array_key_last()` over current boxed arrays,
  returning the first or last ordered integer/string key, or `null` for empty
  arrays.
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
- `array_keys()` over current boxed arrays, preserving insertion order while
  returning a freshly reindexed array of integer/string keys. Optional
  `search_value` filtering uses the shared loose comparison path by default and
  the shared identity path when `strict` is truthy.
- `array_unique()` over current boxed arrays, preserving the first key/value
  for each value under PHP's default string-value comparison and returning a
  fresh ordered array. Omitted flags and `SORT_STRING` use this supported path;
  other sort flags remain an explicit unsupported runtime boundary.
- `array_search()` over current boxed arrays, returning the first matching
  integer/string key under the same loose or strict comparison path as
  `in_array()`, or `false` when no entry matches.
- `array_slice()` over current boxed arrays, slicing by insertion order into a
  fresh ordered array. Positive and negative offsets, omitted or `null`
  lengths, negative lengths, and the preserve-keys flag are supported; integer
  keys are reindexed by default while string keys are preserved.
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
- `array_product()` over current boxed arrays, multiplying entries after the
  same dereferencing and numeric conversion path used by `array_sum()`, and
  returning `1` for empty arrays.
- `range()` over current boxed integer-convertible start, end, and optional
  step arguments, returning ordered arrays of integer values and throwing the
  modeled `ValueError` for zero or out-of-range steps.
- `array_merge_recursive()`, `array_replace()`, and
  `array_replace_recursive()` over current boxed arrays, preserving ordered-map
  key behavior while cloning dereferenced values across COW boundaries.
- `in_array()` over current boxed arrays, returning whether the needle matches
  any entry under loose or strict comparison. References are read through the
  same dereferencing path as other comparison internals.
- `error_reporting()` accepts zero or one scalar argument, returns the previous
  PHP-style mask on writes or current mask on reads, and filters the modeled
  shared warning/deprecation/notice emitters. The `phpc` runner also accepts
  bounded `-d display_errors=false/0/off/no` settings to suppress modeled
  diagnostic display while preserving catchable exception values.
  Expression-level `@` suppression still stacks independently with the
  configured mask.
- `basename()` strips trailing platform path separators from the current
  binary-safe string path, returns the final path segment, and removes a
  matching non-empty suffix only when the suffix is shorter than that segment.
- `gettype()` over current boxed values, returning `NULL`, `boolean`,
  `integer`, `double`, `string`, `array`, `object`, `resource`, or
  `resource (closed)` for the currently modeled scalar, array, object,
  Closure, exception, and stream-resource value domains.
- Type predicates over current boxed scalar and selected non-scalar values:
  `is_array()`, `is_object()`, `is_null()`,
  `is_bool()`, `is_int()`, `is_integer()`, `is_long()`, `is_float()`,
  `is_double()`, `is_string()`, `is_scalar()`, `is_countable()` for arrays,
  `is_iterable()` for arrays in the current non-`Traversable` object subset,
  `is_finite()`,
  `is_infinite()`, `is_nan()`, and `is_resource()` for open stream resources.
- `fopen()` opens filesystem-backed streams through the shared resource value
  model, and `fclose()` closes those resources. Closed stream resources remain
  boxed values for `gettype()` and `var_dump()` but no longer satisfy
  `is_resource()`. `stream_get_meta_data()` reports the current file-stream
  metadata slice for open `fopen()` streams: timeout/blocking/eof flags,
  wrapper and stream type, original mode and URI, unread byte count, and
  seekability.
- `file_get_contents()` reads filesystem-backed paths into binary-safe strings
  using the shared file-read helper, with bounded `offset` and nullable
  `length` handling plus PHP-style negative-length `ValueError`.
- `function_exists()` over generated user-function declarations, including
  resolved namespaced declarations, and the currently registered
  internal-function names.
- `property_exists()` over current object-or-class operands, including
  declared instance/static property metadata with inherited-private exclusion
  and stdClass dynamic property slots. Invalid non-object/non-string first
  operands throw modeled `TypeError`s; property-name arguments use the current
  weak string-argument coercion path.
- `is_callable()` over current string, closure, static method array, object
  method array, and public `__invoke` object callable values, including
  inherited public object methods, supported `__call` fallback, inherited
  `__invoke`, and the optional syntax-only flag. Plain objects without
  `__invoke` are not callable, including in syntax-only mode. The third
  by-reference callable-name output parameter is not yet supported.
- `call_user_func()` dispatches current string, closure, static method array,
  object method array, and public `__invoke` object callable values through
  the shared callable path, including user-function `global` bindings.
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
  `PHP_EOL`, `DIRECTORY_SEPARATOR`, `PATH_SEPARATOR`, `PHP_INT_MIN`,
  `PHP_INT_MAX`, `PHP_INT_SIZE`, the modeled PHP version/platform metadata
  constants, the PHP `E_*` error-reporting mask constants, `INF`, `NAN`,
  `M_PI`, and the modeled
  PHP math constants `M_E`, `M_LOG2E`, `M_LOG10E`, `M_LN2`, `M_LN10`,
  `M_PI_2`, `M_PI_4`, `M_1_PI`, `M_2_PI`, `M_SQRTPI`, `M_2_SQRTPI`,
  `M_LNPI`, `M_EULER`, `M_SQRT2`, `M_SQRT1_2`, `M_SQRT3`,
  `ARRAY_FILTER_USE_BOTH`, `ARRAY_FILTER_USE_KEY`, `STR_PAD_LEFT`,
  `STR_PAD_RIGHT`, `STR_PAD_BOTH`, and modeled `LC_*` locale category
  constants. Other ordinary names report as undefined.
- `setlocale()` accepts the standard C locale category constants for current
  locale queries (`0`, `"0"`, and `null`), string locale names, variadic
  fallback candidates, and ordered array candidates. It returns the first
  successful libc locale string or `false` when no candidate applies.
- Duplicate global `const` declarations and `const` redeclarations after
  `define()` emit the modeled duplicate-constant warning boundary and preserve
  the original runtime constant value.
- A minimal `phpc` runner for supported PHPT rows. It compiles scripts or `-r`
  snippets to temporary native binaries through the normal compiler pipeline.
  `-d precision=N` and `-d error_reporting=N` influence modeled runtime
  behavior; `-d display_errors=value` and `-d zend.assertions=value` are
  accepted for PHPT harness parity and are visible through `ini_get()`.
- Braced and single-statement `if`, `elseif`, and `else` statements whose
  conditions and bodies use the currently supported scalar expression and
  statement subset.
- Plain compound statement blocks `{ ... }` over the currently supported
  statement subset. Blocks do not introduce a variable scope, and labels/gotos
  remain visible through recursive validation.
- Script-level `return;` and `return expr;` statements. Optional return
  expressions are evaluated through the current boxed expression path, then the
  generated native program frees runtime state and exits successfully.
- `include expr`, `include_once expr`, `require expr`, and `require_once expr`
  over compile-time-resolved string paths, including string literals and
  `__DIR__`/`__FILE__` concatenation. Included statement-only files are
  compiled into native helpers that share the caller's current variable frame,
  emit ordinary output at the include point, return the included `return expr;`
  value, return `null` for `return;`, and return `int(1)` when the included
  file reaches EOF without `return`. Once-include forms are keyed by the
  canonical compiled include file and return `true` without re-executing files
  already executed by `include`, `include_once`, `require`, or `require_once`.
- `while (expr) statement` loops where the condition and braced or
  single-statement body use the currently supported scalar expression and
  statement subset.
- `do statement while (expr);` loops where the braced or single-statement body
  and condition use the currently supported scalar expression and statement
  subset. The body executes once before the first condition check.
- `for (init; condition; update) statement` loops where init and update clauses
  use direct variable assignment, direct variable or variable-root array-offset
  increment/decrement, or simple internal-call statements, conditions use the
  currently supported scalar expression subset, and the body is either a
  braced block or one supported statement. Missing conditions are treated as
  true.
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
- Top-level class declarations, including resolved namespaced class names, with
  public static and instance methods in the current function subset. Static
  methods are registered in the callable table under `Class::method`, can be
  called directly with `Class::method(...)`, including inherited static
  methods resolved through the current declared-method lookup, and can be used
  by dynamic calls or internal callbacks through `"Class::method"` and
  `["Class", "method"]` callable values. Declared class names and declared
  method names are exposed through bounded `class_exists()` and
  `method_exists()` metadata, with case-insensitive lookup and `stdClass`
  recognized as the current built-in object shell. Declared instance/static
  property names are exposed through bounded `property_exists()` metadata.
  Object class names are exposed through bounded `get_class($object)` for
  current object, closure, and exception values; non-object operands throw a
  modeled `TypeError`. Current object, closure, and exception values receive
  stable runtime object identities exposed through `spl_object_id()` and
  PHP-shaped `spl_object_hash()` strings; non-object operands throw modeled
  `TypeError`s. Declared and inherited public instance methods can be called
  directly through object receivers and through `[$object, "method"]` callable
  values, including internal callback dispatch. Public `__construct` methods
  in declared classes are invoked
  during `new Class(...)` after declared property defaults are installed,
  using the same method dispatch, `$this` binding, inherited public method
  lookup, positional argument/default-parameter handling, and return-value
  cleanup as other declared instance methods. Missing direct and callable
  object method dispatch falls through to inherited public
  `__call($name, $args)` when present; the generated helper supplies the
  attempted method name and an ordered argument array. Objects with inherited
  public `__invoke` can be called directly as `$object(...)` and through
  `call_user_func()`/`call_user_func_array()` using the same declared-method
  dispatch. `is_callable()` reports
  the supported string, closure, static-method array, object-method array,
  invokable-object, inherited method, and `__call` fallback subset, with
  optional syntax-only checks for valid callable shapes.
- Public static property declarations in top-level classes, using the supported
  constant-expression default subset. Generated native code initializes
  declaration-backed static slots before top-level statements, supports
  `Class::$name` reads, writes, compounds, and pre/post inc/dec, resolves
  `self::$name` inside declared methods, quiet-probes `isset()`, `empty()`, and
  expression-form `??` over declared-class static properties, and supports
  direct static-property null coalescing assignment `Class::$name ??= expr` and
  `self::$name ??= expr` with quiet reads and lazy right-hand evaluation.
  `property_exists()` sees declared static properties through class metadata
  using the same inherited-private exclusion as PHP. Ordinary undeclared static
  property reads/writes and inc/dec throw modeled PHP `Error` diagnostics.
- `new stdClass` and declared-class object shells, boxed object handles, public
  dynamic property reads/writes such as `$object->name`, and public declared
  instance properties with supported constant defaults. Object assignment
  shares the object handle, declared defaults are initialized on construction,
  property assignment expressions return the assigned value, and property reads
  can flow through generated user functions and string-callable
  `call_user_func()` dispatch. Public property null coalescing assignment
  `$object->name ??= expr` quiet-reads the property and lazily evaluates the
  right-hand expression. Property `isset()`, `empty()`, and expression-form
  `??` quiet-probe the current object property storage; inaccessible private
  declared properties behave as missing outside their declaring class. Property
  pre/post inc/dec and direct compound assignments use the modeled property
  read/write path. Declared private/protected instance properties are
  initialized with the same storage path and preserve metadata for `var_dump()`
  private/protected labels and `property_exists()` checks. Private declared
  properties are read/written from methods of their declaring class and
  rejected for outside reads/writes with modeled `Error`; full protected
  visibility, inherited property resolution beyond current metadata checks,
  typed properties, constructor promotion, magic, destructors, and reflection
  property metadata remain outside this support boundary.
- Source-spanned compile diagnostics emitted through `phpc` use PHP-style fatal
  or parse-error boundaries with the source file and line. This currently
  covers duplicate `default:` clauses in `switch`, duplicate labels, undefined
  `goto` labels, invalid `goto` jumps into loop or switch scopes, removed
  `(real)` and `(unset)` cast syntax,
  expression-context `(void)` cast syntax, unterminated block comments, and
  invalid legacy octal integer literals containing `8` or `9`, plus
  unparenthesized nested ternary fatal diagnostics and
  unexpected-token parse errors at modeled statement terminators and right
  parentheses, including expression-level ternary `? :` sites outside the
  currently modeled nested-ternary diagnostic. Global `const` declaration
  terminators report the const-specific `"," or ";"` expected-token set, and
  removed alternative `{}` offsets inside braced string interpolation report
  the current PHP unexpected-token parse error. Unsupported class members and
  class-constant fetch syntax are recognized and reported as class metadata
  boundaries.
- Direct variable, variable-root array-offset, modeled property, and direct
  static-property increment/decrement over boxed integer and float values,
  null, booleans, numeric strings, empty strings, and alphanumeric string
  increment. Statement forms such as `$name++;`, `$items[$key]--;`, and
  `$object->count++;` write the updated value. Expression forms such as
  `++$name`, `$name++`, `++$items[$key]`, `$items[$key]--`, and
  `++ClassName::$counter` return the PHP pre/post result value while applying
  the same side effect. Array, object, closure, exception, resource, and
  reference operands throw modeled catchable `TypeError` messages for the
  current target slice.

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
- Increment/decrement targets beyond direct variables, variable-root array
  offsets, modeled properties, and direct static properties, including append
  offsets, dynamic-variable array offsets, temporary array reads, nested or
  dynamic property lvalues, and broader static-property forms.
- Ternary precedence edges beyond the modeled nested associativity diagnostics,
  PHP-exact chained comparison parse errors, and complete comparison parity for
  unsupported value types.
- Unbraced switch bodies, alternate control-flow syntax,
  for-loop comma expressions and
  non-direct-variable clause lvalues, PHP-exact break/continue diagnostics
  beyond the currently modeled level/context fatals and switch-target warning,
  labels/goto inside unsupported functions, classes, and `try`/`finally`
  constructs, and exception/finally control-flow edges.
- Object `Traversable`, destructuring foreach targets, and PHP-exact
  `foreach` diagnostics outside the current array/non-array warning lane.
- PHP-exact include behavior beyond compile-time-resolved statement-only files,
  including fully dynamic paths outside bounded candidate sets, include paths,
  missing-file warning/return behavior, declaration-bearing include files, and
  return inside unsupported function/class contexts.
- Switch alternate syntax and switch behavior for arrays, objects, references,
  copy-on-write, and exceptions.
- Remaining PHP-exact increment/decrement target and value edges, including
  unsupported target roots, Unicode/string edge cases, references,
  copy-on-write, and diagnostic parity beyond the current modeled target
  slice.
- Remaining complex string interpolation forms, including object/property
  interpolation, variable variables, arbitrary expressions/calls, append
  offsets, and non-variable-root offsets.
- Heredoc interpolation, flexible indentation, and exact label diagnostics
  beyond the current plain heredoc/nowdoc string-literal slice.
- Internal functions outside the registered internal-function subset.
- Exact undefined-constant and unsupported-expression-statement diagnostics.
- Class constants, namespace fallback parity for arbitrary userland functions
  and constants, global `const` duplicate diagnostics and ordering parity with
  runtime `define()`, and built-in PHP/extension constants other than the
  currently modeled `E_*` error masks, `PHP_EOL`,
  `DIRECTORY_SEPARATOR`, `PATH_SEPARATOR`, `PHP_INT_MIN`, `PHP_INT_MAX`,
  `PHP_INT_SIZE`, `INF`, `NAN`, `M_PI`, modeled PHP math `M_*` constants,
  array-filter mode constants, sort mode constants, `STR_PAD_*` constants, and
  modeled `LC_*` locale constants in `defined()`/`constant()`.
- Function forms beyond top-level named declarations and the public class-method
  callable slice, plus the bounded `stdClass` public-property storage slice,
  including array/object default arguments, named arguments outside direct
  generated user-function calls, by-reference returns, nested or conditional
  declarations, closures, old-style constructor dispatch, full class metadata,
  globals, static locals, and remaining PHP-exact function return propagation.
- Type predicate coverage for full PHP resource and reference metadata beyond
  the current open-stream `is_resource()` and file-stream metadata slices.
- Unsupported recursive arrays, full class/object metadata, broad resources
  beyond the current stream slice, complete reference identity,
  copy-on-write, and `var_dump()` reference identity beyond the currently
  modeled ordered-array, direct-reference, and `stdClass` public-property
  behavior.
- `array_key_exists()` object property checks, references, and error-handler
  routing beyond the current ordered-array/resource-key slice.
- String-offset append, compound assignment, reference `isset()`/`empty()` and
  null-coalescing semantics, property reference targets, and complete
  TypeError/exception parity for unsupported string offset key types.
- Embedded NUL strings in runtime values and embedded NUL string array keys,
  `var_dump()` string
  length/output, `strlen()`, `str_rot13()`, `strcmp()`, `strcasecmp()`,
  `bin2hex()`, `chr()`, `hex2bin()`, `str_contains()`, `strpos()`, `stripos()`,
  `strrpos()`, `strripos()`, `strstr()`, `stristr()`, `substr_count()`,
  `quotemeta()`, `trim()`, `ltrim()`, `rtrim()`, `chop()`, `strip_tags()`,
  `quoted_printable_decode()`, `addcslashes()`,
  `stripcslashes()`, `md5()`, `sha1()`, `substr()`, `soundex()`, `ord()`, or
  bitwise string results.
- Exact `strcmp()` resource/reference operand parity and object string
  conversion outside the current public declared `__toString()` support.
- Exact `strcasecmp()` resource/reference operand parity and object string
  conversion outside the current public declared `__toString()` support.
- Exact `strncmp()` resource/reference operand parity, oversized length
  diagnostics, and object string conversion outside the current public
  declared `__toString()` support.
- Exact `join()`/`implode()` diagnostics, resource/reference operand parity,
  and object string conversion outside the current public declared
  `__toString()` support.
- Exact `str_contains()` resource/reference operand parity and object string
  conversion outside the current public declared `__toString()` support.
- Exact `str_starts_with()`/`str_ends_with()` resource/reference operand parity
  and object string conversion outside the current public declared
  `__toString()` support.
- Exact `str_pad()` resource/reference operand parity, oversized allocation
  diagnostics, and object string conversion outside the current public declared
  `__toString()` support.
- Exact `quotemeta()` resource/reference operand parity and object string
  conversion outside the current public declared `__toString()` support.
- Exact `trim()`/`ltrim()`/`rtrim()`/`chop()` malformed-charlist warning parity plus
  resource/reference operand parity and object string conversion outside the
  current public declared `__toString()` support.
- Exact `chunk_split()` resource/reference operand parity and object string
  conversion outside the current public declared `__toString()` support.
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
- Complete `sprintf()` formatter coverage, including positional arguments,
  full length/flag behavior, binary-string edges, and unsupported operand
  diagnostics beyond the current bounded scalar subset.
- `md5()`/`sha1()` raw binary output containing NUL bytes, embedded-NUL input
  parity, plus `crc32()`/`md5()`/`sha1()` resource/reference operand parity and
  object string conversion outside the current public declared `__toString()`
  support.
- Exact `substr()` binary-string behavior for embedded NUL bytes and
  resource/reference operand parity plus object string conversion outside the
  current public declared `__toString()` support.
- Exact `chr()` diagnostics for remaining strict-types/reference operand edges.
- Exact `ord()` strict-types and unsupported-type diagnostics.
- Exact `abs()` complete overflow parity beyond the current boxed numeric path.
- `count()` support for `Countable` objects and exact non-array diagnostics.
- Complete `sqrt()` negative/non-finite float parity beyond the current scalar
  numeric path.
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
- Exact `nl2br()` optional boolean argument diagnostics plus resource/reference
  operand parity and object string conversion outside the current public
  declared `__toString()` support.
- Complete `str_replace()` object-element conversion/error parity and remaining
  diagnostic edge cases beyond the current ordered-array operand slice.
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
  standard/reflection names and loaded-extension list.
- Cast spelling diagnostics beyond the currently modeled non-canonical aliases
  and removed `(real)`/`(unset)` plus expression-context `(void)` boundaries.
- Scope-aware magic constants inside traits, includes, and eval contexts, plus
  remaining namespace-sensitive reflection and metadata parity.
- PHP-exact file names, line numbers, custom error-handler routing, and
  overflow parity for remaining integer-only operator conversion diagnostics,
  including shift and modulo diagnostics.
- Object lvalues, dynamic-variable by-reference lvalues, append-form
  null-coalescing, property reference targets, nested/dynamic property
  compound lvalues, and static-property lvalues outside modeled direct
  reads/writes/`??=`/compound/inc/dec and read-side quiet probes.
- Remaining reference semantics for compound assignment outside direct
  variables and modeled array elements, including full copy-on-write
  interactions and by-reference visibility during writes.
- Arrays, references, copy-on-write, globals, superglobals, classes, objects,
  resources, exceptions, variable variables, dynamic includes, and dynamic
  fallback.
