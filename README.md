# PTN From Scratch

PTN is a fresh PHP-to-native binary compiler project. It is guided by
`NEW_PROMPT.md`: build full PHP compatibility through a generic compiler and
runtime architecture, not through row-shaped test patches.

The current first integrated slice is intentionally small but real:

- PHP source is lexed and parsed into an AST with source spans.
- The AST is lowered into a PHP-aware IR.
- The backend emits C containing a boxed PHP-value runtime.
- The system C compiler produces a native executable.
- Tests compile and execute a generated native binary.

Supported today:

- `<?php` open tag.
- A Unix shebang at the start of the file before `<?php`.
- PHP `//`, `#`, and `/* ... */` comments inside PHP code.
- A `?>` close tag that ends PHP mode and emits following inline output, with
  the first immediately following newline swallowed as PHP does.
- Global-scope `const NAME = expr;` declarations for the currently supported
  constant-expression subset. Declared constants are visible to bare constant
  reads, `defined()`, and `constant()`.
- `echo` statements.
- Statement-form `print expr;` for the same scalar expression subset as
  `echo`; emitted native code uses the same boxed output conversion path.
- Simple internal calls such as `var_dump(expr, ...)`, `strlen(expr)`,
  `str_rot13(expr)`, `strcmp(expr, expr)`, `str_contains(expr, expr)`,
  `quotemeta(expr)`, `md5(expr[, raw_output])`, `sha1(expr[, raw_output])`,
  `substr(expr, expr[, expr])`, `bin2hex(expr)`, `hex2bin(expr)`,
  `dirname(expr)`, `soundex(expr)`, `ceil(expr)`, `floor(expr)`,
  `sqrt(expr)`, `fdiv(expr, expr)`, `bindec(expr)`, `hexdec(expr)`,
  `octdec(expr)`, `pi()`, `getrandmax()`, `getmypid()`, `chr(expr)`,
  `ord(expr)`,
  `error_reporting(expr)`, `gettype(expr)`, scalar `is_*` type predicates,
  non-finite predicates such as `is_finite(expr)`, `is_infinite(expr)`, and
  `is_nan(expr)`, `define(expr, expr)`, `constant(expr)`,
  `defined(expr)`, and `function_exists(expr)`, lowered through IR
  internal-call nodes and generated C runtime dispatch.
- `var_dump()` output for the current boxed `PtnValue` types: `null`,
  booleans, integers, floats, strings, and ordered literal arrays. Finite
  floats use the shortest decimal spelling that round-trips to the same native
  double; `INF`, `-INF`, and `NAN` keep PHP-like special spellings.
- `strlen()` as an expression returning the byte length of the current boxed
  scalar string-conversion result.
- `str_rot13()` as an expression returning ASCII ROT13 over the current boxed
  scalar string-conversion result.
- `strcmp()` as an expression returning negative, zero, or positive comparison
  results over the current boxed scalar string-conversion results.
- `str_contains()` as an expression returning whether the needle scalar
  string-conversion result is present in the haystack scalar
  string-conversion result.
- `quotemeta()` as an expression prefixing PHP regex metacharacter bytes with
  backslashes after the current boxed scalar string-conversion result.
- `md5()` and `sha1()` as expressions returning lowercase hexadecimal digest
  output for the current boxed scalar string-conversion result. The optional
  `raw_output` argument returns raw digest bytes through the current
  C-string-backed value path.
- `substr()` as an expression returning a byte slice of the current boxed
  scalar string-conversion result, with start and optional length operands
  converted through the current boxed scalar integer-conversion path. Negative
  starts count back from the end, negative lengths truncate from the end,
  omitted or `null` lengths read to the end, and extreme negative offsets clamp
  to the beginning.
- `bin2hex()` as an expression returning lowercase hexadecimal bytes for the
  current boxed scalar string-conversion result.
- `hex2bin()` as an expression decoding hexadecimal pairs from the current
  boxed scalar string-conversion result, returning `false` with a warning
  boundary for odd-length or non-hexadecimal input.
- `dirname()` as an expression returning the parent directory from the current
  boxed scalar string-conversion result.
- `soundex()` as an expression returning the PHP-style four-character ASCII
  soundex key for the current boxed scalar string-conversion result.
- `ceil()` and `floor()` as expressions returning boxed floats after the
  current boxed scalar numeric-conversion result.
- `sqrt()` as an expression returning a boxed float after the current boxed
  scalar numeric-conversion result.
- `fdiv()` as an expression returning boxed IEEE-style floating-point division
  after the current boxed scalar numeric-conversion result, including zero
  divisors and non-finite operands.
- `pi()` as an expression returning the modeled boxed float value of `M_PI`.
- `getrandmax()` as an expression returning the modeled maximum random integer.
- `getmypid()` as an expression returning the generated native process id.
- `bindec()`, `hexdec()`, and `octdec()` as expressions over the current boxed
  scalar string-conversion result, accepting the matching PHP base prefix and
  returning integers or floats based on native integer range.
- `chr()` as an expression returning a one-byte string from the current boxed
  scalar integer-conversion result, with byte values constrained modulo 256.
- `ord()` as an expression returning the first byte of the current boxed scalar
  string-conversion result, including PHP-like deprecation diagnostics for
  empty and multi-byte strings.
- `error_reporting()` accepts zero or one argument and returns the current
  placeholder reporting level. Runtime error filtering is not modeled yet.
- `gettype()` and scalar type predicates for the current boxed scalar
  `PtnValue` types: `is_null()`, `is_bool()`, `is_int()`/`is_integer()`/
  `is_long()`, `is_float()`/`is_double()`, `is_string()`, `is_scalar()`,
  `is_finite()`, `is_infinite()`, and `is_nan()`.
- Symbol-existence predicates for currently modeled runtime tables:
  `function_exists()` checks the generated internal-function registry, and
  `defined()` checks global `const` declarations, constants created with
  `define()`, plus the modeled constant registry, which currently includes
  `E_ERROR`, `PHP_EOL`,
  `DIRECTORY_SEPARATOR`, `PATH_SEPARATOR`, `PHP_INT_MIN`, `PHP_INT_MAX`,
  `PHP_INT_SIZE`, `INF`, `NAN`, `M_PI`, and the modeled PHP math constants
  `M_E`, `M_LOG2E`, `M_LOG10E`, `M_LN2`, `M_LN10`, `M_PI_2`, `M_PI_4`,
  `M_1_PI`, `M_2_PI`, `M_SQRTPI`, `M_2_SQRTPI`, `M_LNPI`, `M_EULER`,
  `M_SQRT2`, `M_SQRT1_2`, and `M_SQRT3`.
- `define()` creates runtime constants over the current boxed value subset and
  returns `false` with a warning when the requested name is already defined.
  `constant()` reads the same runtime and modeled built-in constant registry
  using the current scalar string-conversion result for the name.
- String, integer, float, boolean, and null literals. Numeric literals accept
  PHP digit separators between digits; integer literals include decimal,
  legacy octal, explicit octal `0o`/`0O`, binary `0b`/`0B`, and hexadecimal
  `0x`/`0X` forms.
- Invalid legacy octal integer literals containing `8` or `9` are rejected with
  source-spanned PHP-style parse errors through the `phpc` runner.
- Double-quoted strings with direct `$name` variable interpolation. Interpolated
  variables use ordinary runtime variable reads, scalar string casts, and boxed
  concatenation. Unrecognized backslash escapes in single-quoted and
  double-quoted strings preserve the backslash and escaped byte.
- Direct variable assignment and reads for scalar values through the generated
  native runtime symbol table.
- Generic runtime diagnostics for undefined direct variable reads. The read
  still yields `null` after emitting a warning boundary.
- Boxed scalar `+`, `-`, `*`, `**`, `/`, and `%` numeric arithmetic and `.` string
  concatenation expressions, including chained expressions and assignment
  results. The parser treats `**` as right-associative with PHP's precedence
  relative to unary operators, `*`, `/`, and `%` as higher precedence than `+`
  and `-`, and arithmetic as higher precedence than `.`, while the backend emits
  runtime calls over `PtnValue` operands. Integer-only `%` conversions emit the
  current float/float-string precision-loss deprecation boundary when a scalar
  operand loses precision while converting to an integer.
- Direct named-variable compound assignment for `+=`, `-=`, `*=`, `/=`, `%=`,
  `**=`, `.=`, `&=`, `|=`, `^=`, `<<=`, and `>>=`. These lower as a variable
  read, the matching boxed binary helper, then a variable write, preserving the
  existing undefined-variable diagnostic boundary.
- Parenthesized expressions, unary `+`, unary `-`, unary `!`, unary bitwise
  `~`, `(int)`, `(float)`, `(string)`, `(bool)`, and deprecated
  non-canonical `(integer)`, `(double)`, `(binary)`, and `(boolean)` casts for
  boxed scalar values. Unary, cast, and binary operations are emitted as runtime
  helper calls over `PtnValue` operands.
- Removed `(real)` cast syntax is rejected with a source-spanned PHP-style
  parse error through the `phpc` runner.
- Removed `(unset)` cast syntax is rejected with a source-spanned PHP-style
  fatal error through the `phpc` runner.
- Expression-context `(void)` cast syntax is rejected with a source-spanned
  PHP-style parse error through the `phpc` runner.
- Unterminated block comments are rejected with a source-spanned PHP-style
  parse error through the `phpc` runner.
- Unexpected tokens at currently modeled statement terminators and right
  parentheses are rejected with source-spanned PHP-style parse errors through
  the `phpc` runner.
- Global-scope magic constants `__LINE__`, `__FILE__`, `__DIR__`,
  `__FUNCTION__`, `__METHOD__`, `__CLASS__`, `__TRAIT__`, and
  `__NAMESPACE__`. Scope-dependent names currently resolve to empty strings in
  global scope.
- Short array literals `[...]` with optional scalar keys, automatic integer
  keys, integer-string key canonicalization, insertion order, and duplicate-key
  replacement in the current literal-array value subset.
- Array read expressions such as `$array[$key]`, including nested reads and
  reads from literal or grouped array expressions. Reads use the current
  ordered-array key canonicalization path and return `null` with a warning
  boundary for undefined keys or non-array containers.
- String offset read expressions such as `$string[$offset]` for the current
  C-string-backed scalar string subset. Integer-compatible offsets, negative
  offsets, nested reads, integer strings, numeric prefix strings with PHP-style
  illegal-offset warnings, and scalar cast warnings for `null`, booleans, and
  floats are handled by the shared offset-read helper; out-of-range reads emit
  an uninitialized-offset warning and return an empty string.
- Boxed scalar and literal-array comparison and boolean expressions: `==`,
  `!=`, `===`, `!==`, `<`, `<=`, `>`, `>=`, `<=>`, `&&`, and `||`. Strict
  identity compares type, key order, key type, and value; numeric scalar
  comparisons involving `NAN` evaluate as unordered so equality and ordered
  comparisons return false. Boolean operators short-circuit over boxed PHP
  truthiness for the currently supported value types.
- Boxed scalar bitwise `&`, `^`, `|`, and unary `~` expressions.
  String/string binary operands and string unary `~` operands use PHP bytewise
  string results for non-NUL strings; other supported scalar operands are
  converted to integers through the current boxed numeric conversion path,
  including the current float/float-string precision-loss deprecation boundary.
- Boxed scalar bit shifts `<<` and `>>`. Supported scalar operands are
  converted to integers through the current bitwise integer-conversion path,
  including the current float/float-string precision-loss deprecation boundary,
  and operands are evaluated left-to-right before calling runtime helpers.
- Braced and single-statement `if`, `elseif`, and `else` statements. Branch
  conditions use boxed scalar truthiness and the currently supported expression
  subset, including grouped expressions and scalar comparisons.
- `while (expr) statement` loops over the currently supported scalar expression
  and statement subset, with either a braced block or one supported statement
  as the body.
- `do statement while (expr);` loops over the currently supported scalar
  expression and statement subset, with either a braced block or one supported
  statement as the body. The body executes before the condition is checked.
- `for (init; condition; update) statement` loops where init and update clauses
  use direct variable assignment, direct increment/decrement, or simple
  internal-call statements, conditions use the currently supported scalar
  expression subset, and the body is either a braced block or one supported
  statement. Missing conditions are treated as true.
- Braced `switch (expr) { case expr: ... default: ... }` statements over the
  currently supported scalar expression and statement subset, including
  source-order case matching with boxed loose comparison, PHP-style fallthrough,
  one `default`, and `break;` or explicit-level `break N;` over active
  switch/loop targets.
- User labels such as `L1:` and `goto L1;` statements inside the currently
  generated main function, including source-spanned fatal diagnostics for
  undefined target labels and duplicate labels.
- Statement-form direct variable `++` and `--`, such as `$i++;` and `--$i;`,
  using the boxed numeric arithmetic helper path.

Unsupported today:

- Long-form `array(...)`, array element mutation, append/unset,
  iteration, recursive arrays, arrays with references/copy-on-write, objects,
  functions, classes, includes, references, resources, exceptions,
  string-offset writes/mutation, `isset()`/`empty()`/null-coalescing offset
  semantics, string-offset references/unset,
  array/object/reference compound-assignment lvalues,
  compound operators other than `+=`, `-=`, `*=`, `/=`, `%=`, `**=`, `.=`, `&=`,
  `|=`, `^=`, `<<=`, and `>>=` (`??=`), `print` as an expression returning
  `1` even when spelled `print(...)`, increment/decrement operators, full
  PHP numeric-string and non-numeric string arithmetic diagnostics, exact
  division/modulo-by-zero exception behavior, exact numeric literal
  overflow/range and invalid-separator/radix diagnostic parity, complete
  comparison parity for unsupported types,
  keyword boolean operators, chained comparison parse errors, unbraced switch
  bodies and alternate control-flow syntax, `foreach`, `continue`, PHP-exact
  break/continue diagnostics, forbidden-scope goto restrictions for jumps
  into/out of invalid scopes, full
  switch parity for unsupported value types and alternate syntax,
  increment/decrement as expressions, PHP-exact
  increment/decrement semantics for strings/booleans and other edge values,
  for-loop comma expressions and non-direct-variable clause lvalues,
  complete overflow parity, exact scalar cast overflow behavior, PHP-exact
  warning text/file/line/error-handler behavior, inline HTML before `<?php` or
  between PHP blocks, internal functions outside the registered scalar subset,
  namespace/class constants, global `const` duplicate diagnostics and ordering
  parity with runtime `define()`, `define()`'s legacy case-insensitive flag,
  and built-in constants other than the currently modeled `E_ERROR`, `PHP_EOL`,
  `DIRECTORY_SEPARATOR`, `PATH_SEPARATOR`, `PHP_INT_MIN`, `PHP_INT_MAX`,
  `PHP_INT_SIZE`, `INF`, `NAN`, `M_PI`, and the modeled PHP math `M_*`
  constants, objects, resources, recursion, references,
  embedded NUL string handling, complex/braced interpolation, interpolation of
  arrays/objects/offsets/properties/variable variables, exact `strcmp()`,
  `str_contains()`, `quotemeta()`, and `substr()` binary-string parity, exact
  `md5()`/`sha1()` raw-output and embedded-NUL input parity, exact `hex2bin()`
  embedded-NUL output parity and warning text/file-name parity, exact
  `dirname()` edge parity for unusual paths and unsupported operands, exact
  `soundex()` locale/non-ASCII parity and unsupported type diagnostics, exact
  `chr()` deprecation diagnostics, exact `ord()` argument type diagnostics,
  exact `ceil()`/`floor()`
  null/string/unsupported type diagnostics, exact `sqrt()` negative/non-finite
  edge parity, exact `fdiv()` unsupported operand diagnostics, exact
  `getmypid()` process model parity across SAPIs and unsupported platforms,
  exact `error_reporting()` configuration/filtering behavior, unsupported cast
  spelling diagnostics beyond the currently modeled aliases and removed cast
  boundaries, statement-form `(void) expr;` casts, and full PHP
  precision/formatting edge cases for
  `var_dump()`/`strlen()`/`bin2hex()`/`hex2bin()`/`str_contains()`/
  `quotemeta()`/`md5()`/`sha1()`/`substr()`/`soundex()`/base-conversion
  internals, scope-aware magic constants inside functions/classes/namespaces/includes, exact
  file/line/error-handler behavior for integer-only operator precision-loss
  diagnostics, doc comment retention, variable variables, and dynamic fallback.
  These are architecture targets, not excuses for exact-shape hacks.

## Build

```sh
cargo build
```

## Test

```sh
cargo test
```

## Compile a PHP File

```sh
cargo run --bin ptn -- compile examples/hello.php -o /tmp/ptn-hello
/tmp/ptn-hello
```

## PHPT Runner Telemetry

PTN includes a minimal `phpc` runner for direct PHPT execution of the currently
supported native subset:

```sh
cargo build --bin phpc
PHPC_BIN="$PWD/target/debug/phpc" php /path/to/php-src/run-tests.php -q -p "$PWD/target/debug/phpc" /path/to/test.phpt
```

The runner compiles each script or `-r` snippet to a temporary native binary
and forwards the result. It is not a complete PHP CLI implementation.

## Differential Native Output Telemetry

For the currently supported subset, compare native output against the system PHP
CLI with:

```sh
tools/diff-native-output.sh --snippet '<?php echo "Hello ", 42, "\n";'
tools/diff-native-output.sh examples/hello.php
```

The command compiles the input through `ptn compile`, runs the produced native
binary, runs the same input with `php`, and compares stdout, stderr, and exit
status. It is telemetry for supported snippets only; it is not a PHPT pass-count
claim.

## Production Workflow

A task is ready only when it is integrated into the branch that will be pushed.
Local experiments and patch files are inventory, not progress.

The production line follows continuous improvement principles:

- Keep small branch-ready changes moving.
- Never stop all development on one integration problem.
- Split oversized work when it blocks flow.
- Make defects visible immediately in `PROGRESS.md`.
- Prefer generic runtime/compiler capabilities over one-off fixes.
- Push integrated progress often so remote history reflects actual movement.
