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
- `echo` statements.
- Statement-form `print expr;` for the same scalar expression subset as
  `echo`; emitted native code uses the same boxed output conversion path.
- Simple internal calls such as `var_dump(expr, ...)`, `strlen(expr)`,
  `str_rot13(expr)`, `strcmp(expr, expr)`, `bin2hex(expr)`, `hex2bin(expr)`,
  `soundex(expr)`, `ceil(expr)`, `floor(expr)`, `sqrt(expr)`, `bindec(expr)`,
  `hexdec(expr)`, `octdec(expr)`, `pi()`, `getrandmax()`, `getmypid()`,
  `chr(expr)`, `ord(expr)`,
  `error_reporting(expr)`, `gettype(expr)`, scalar `is_*` type predicates,
  non-finite predicates such as `is_finite(expr)`, `is_infinite(expr)`, and
  `is_nan(expr)`,
  `defined(expr)`, and `function_exists(expr)`, lowered through IR
  internal-call nodes and generated C runtime dispatch.
- `var_dump()` output for the current boxed scalar `PtnValue` types: `null`,
  booleans, integers, floats, and strings. Finite floats use the shortest
  decimal spelling that round-trips to the same native double; `INF`, `-INF`,
  and `NAN` keep PHP-like special spellings.
- `strlen()` as an expression returning the byte length of the current boxed
  scalar string-conversion result.
- `str_rot13()` as an expression returning ASCII ROT13 over the current boxed
  scalar string-conversion result.
- `strcmp()` as an expression returning negative, zero, or positive comparison
  results over the current boxed scalar string-conversion results.
- `bin2hex()` as an expression returning lowercase hexadecimal bytes for the
  current boxed scalar string-conversion result.
- `hex2bin()` as an expression decoding hexadecimal pairs from the current
  boxed scalar string-conversion result, returning `false` with a warning
  boundary for odd-length or non-hexadecimal input.
- `soundex()` as an expression returning the PHP-style four-character ASCII
  soundex key for the current boxed scalar string-conversion result.
- `ceil()` and `floor()` as expressions returning boxed floats after the
  current boxed scalar numeric-conversion result.
- `sqrt()` as an expression returning a boxed float after the current boxed
  scalar numeric-conversion result.
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
  `defined()` checks the current constant registry, which currently includes
  `E_ERROR`, `PHP_EOL`, `DIRECTORY_SEPARATOR`, `PATH_SEPARATOR`,
  `PHP_INT_MIN`, `PHP_INT_MAX`, `PHP_INT_SIZE`, `INF`, `NAN`, `M_PI`, and the
  modeled PHP math constants `M_E`, `M_LOG2E`, `M_LOG10E`, `M_LN2`, `M_LN10`,
  `M_PI_2`, `M_PI_4`, `M_1_PI`, `M_2_PI`, `M_SQRTPI`, `M_2_SQRTPI`,
  `M_LNPI`, `M_EULER`, `M_SQRT2`, `M_SQRT1_2`, and `M_SQRT3`.
- String, integer, float, boolean, and null literals. Numeric literals accept
  PHP digit separators between digits; integer literals include decimal,
  legacy octal, binary `0b`/`0B`, and hexadecimal `0x`/`0X` forms.
- Double-quoted strings with direct `$name` variable interpolation. Interpolated
  variables use ordinary runtime variable reads, scalar string casts, and boxed
  concatenation.
- Direct variable assignment and reads for scalar values through the generated
  native runtime symbol table.
- Generic runtime diagnostics for undefined direct variable reads. The read
  still yields `null` after emitting a warning boundary.
- Boxed scalar `+`, `-`, `*`, `**`, `/`, and `%` numeric arithmetic and `.` string
  concatenation expressions, including chained expressions and assignment
  results. The parser treats `**` as right-associative with PHP's precedence
  relative to unary operators, `*`, `/`, and `%` as higher precedence than `+`
  and `-`, and arithmetic as higher precedence than `.`, while the backend emits
  runtime calls over `PtnValue` operands.
- Direct named-variable compound assignment for `+=`, `-=`, `*=`, `/=`, `%=`,
  `**=`, `.=`, `&=`, `|=`, `^=`, `<<=`, and `>>=`. These lower as a variable
  read, the matching boxed binary helper, then a variable write, preserving the
  existing undefined-variable diagnostic boundary.
- Parenthesized expressions, unary `+`, unary `-`, unary `!`, unary bitwise
  `~`, `(int)`, `(float)`, `(string)`, `(bool)`, and deprecated
  non-canonical `(integer)`, `(double)`, `(binary)`, and `(boolean)` casts for
  boxed scalar values. Unary, cast, and binary operations are emitted as runtime
  helper calls over `PtnValue` operands.
- Global-scope magic constants `__LINE__`, `__FILE__`, `__DIR__`,
  `__FUNCTION__`, `__METHOD__`, `__CLASS__`, `__TRAIT__`, and
  `__NAMESPACE__`. Scope-dependent names currently resolve to empty strings in
  global scope.
- Boxed scalar comparison and boolean expressions: `==`, `!=`, `===`, `!==`,
  `<`, `<=`, `>`, `>=`, `&&`, and `||`. Strict identity compares scalar type
  and value without coercion; numeric scalar comparisons involving `NAN`
  evaluate as unordered so equality and ordered comparisons return false;
  boolean operators short-circuit over boxed PHP truthiness for the currently
  supported scalar value types.
- Boxed scalar bitwise `&`, `^`, `|`, and unary `~` expressions.
  String/string binary operands and string unary `~` operands use PHP bytewise
  string results for non-NUL strings; other supported scalar operands are
  converted to integers through the current boxed numeric conversion path.
- Boxed scalar bit shifts `<<` and `>>`. Supported scalar operands are
  converted to integers through the current bitwise integer-conversion path,
  and operands are evaluated left-to-right before calling runtime helpers.
- Braced `if`, `elseif`, and `else` statements. Branch conditions use boxed
  scalar truthiness and the currently supported expression subset, including
  grouped expressions and scalar comparisons.
- Braced `while (expr) { statements }` loops over the currently supported
  scalar expression and statement subset.
- Braced `do { statements } while (expr);` loops over the currently supported
  scalar expression and statement subset. The body executes before the
  condition is checked.
- Braced `for (init; condition; update) { statements }` loops where init and
  update clauses use direct variable assignment, direct increment/decrement, or
  simple internal-call statements, and conditions use the currently supported
  scalar expression subset. Missing conditions are treated as true.
- Braced `switch (expr) { case expr: ... default: ... }` statements over the
  currently supported scalar expression and statement subset, including
  source-order case matching with boxed loose comparison, PHP-style fallthrough,
  one `default`, and simple `break;`.
- Statement-form direct variable `++` and `--`, such as `$i++;` and `--$i;`,
  using the boxed numeric arithmetic helper path.

Unsupported today:

- Arrays, objects, functions, classes, includes, references, copy-on-write,
  resources, exceptions, array/object/reference compound-assignment lvalues,
  compound operators other than `+=`, `-=`, `*=`, `/=`, `%=`, `**=`, `.=`, `&=`,
  `|=`, `^=`, `<<=`, and `>>=` (`??=`), `print` as an expression returning
  `1` even when spelled `print(...)`, increment/decrement operators, full
  PHP numeric-string and non-numeric string arithmetic diagnostics, exact
  division/modulo-by-zero exception behavior, exact numeric literal
  overflow/range parity, complete comparison parity for unsupported types,
  spaceship comparison operator,
  keyword boolean operators, chained comparison parse errors, unbraced and alternate
  control-flow syntax, `foreach`, explicit-level `break`
  such as `break 2`, `continue`, full switch parity for unsupported value
  types and alternate syntax, increment/decrement as expressions, PHP-exact
  increment/decrement semantics for strings/booleans and other edge values,
  for-loop comma expressions and non-direct-variable clause lvalues,
  complete overflow parity, exact scalar cast overflow behavior, PHP-exact
  warning text/file/line/error-handler behavior, inline HTML before `<?php` or
  between PHP blocks, internal functions outside the registered scalar subset,
  user constants and built-in constants other than the currently modeled
  `E_ERROR`, `PHP_EOL`, `DIRECTORY_SEPARATOR`, `PATH_SEPARATOR`,
  `PHP_INT_MIN`, `PHP_INT_MAX`, `PHP_INT_SIZE`, `INF`, `NAN`, `M_PI`, and the
  modeled PHP math `M_*` constants, arrays, objects, resources, recursion,
  references,
  embedded NUL string handling, complex/braced interpolation, interpolation of
  arrays/objects/offsets/properties/variable variables, exact `strcmp()`
  binary-string parity, exact `hex2bin()` embedded-NUL output parity and
  warning text/file-name parity, exact `soundex()` locale/non-ASCII parity and
  unsupported type diagnostics, exact `chr()` deprecation diagnostics, exact
  `ord()` argument type diagnostics, exact `ceil()`/`floor()`
  null/string/unsupported type diagnostics, exact `sqrt()` negative/non-finite
  edge parity, exact `getmypid()` process model parity across SAPIs and
  unsupported platforms, exact `error_reporting()` configuration/filtering
  behavior, non-canonical cast spellings beyond `(integer)`, `(double)`,
  `(binary)`, and `(boolean)`, and full PHP
  precision/formatting edge cases for
  `var_dump()`/`strlen()`/`bin2hex()`/`hex2bin()`/`soundex()`/base-conversion
  internals, scope-aware magic constants inside functions/classes/namespaces/
  includes, doc comment retention, variable variables, and dynamic fallback.
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
