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
- Simple internal calls such as `var_dump(expr, ...)` and `strlen(expr)`,
  lowered through IR internal-call nodes and generated C runtime dispatch.
- `var_dump()` output for the current boxed scalar `PtnValue` types: `null`,
  booleans, integers, floats, and strings.
- `strlen()` as an expression returning the byte length of the current boxed
  scalar string-conversion result.
- String, integer, float, boolean, and null literals.
- Direct variable assignment and reads for scalar values through the generated
  native runtime symbol table.
- Generic runtime diagnostics for undefined direct variable reads. The read
  still yields `null` after emitting a warning boundary.
- Boxed scalar `+`, `-`, `*`, `/`, and `%` numeric arithmetic and `.` string
  concatenation expressions, including chained expressions and assignment
  results. The parser treats `*`, `/`, and `%` as higher precedence than `+`
  and `-`, and arithmetic as higher precedence than `.`, while the backend
  emits runtime calls over `PtnValue` operands.
- Direct named-variable compound assignment for `+=`, `-=`, `*=`, `/=`, `%=`,
  `.=`, `&=`, and `|=`. These lower as a variable read, the matching boxed
  binary helper, then a variable write, preserving the existing
  undefined-variable diagnostic boundary.
- Parenthesized expressions, unary `+`, unary `-`, unary `!`, and `(int)`,
  `(float)`, `(string)`, and `(bool)` casts for boxed scalar values. Unary, cast, and
  binary operations are emitted as runtime helper calls over `PtnValue`
  operands.
- Boxed scalar comparison and boolean expressions: `==`, `!=`, `===`, `!==`,
  `<`, `<=`, `>`, `>=`, `&&`, and `||`. Strict identity compares scalar type
  and value without coercion; boolean operators short-circuit over boxed PHP
  truthiness for the currently supported scalar value types.
- Boxed scalar bitwise `&` and `|` expressions. String/string operands use PHP
  bytewise string results for non-NUL strings; other supported scalar operands
  are converted to integers through the current boxed numeric conversion path.
- Braced `if`, `elseif`, and `else` statements. Branch conditions use boxed
  scalar truthiness and the currently supported expression subset, including
  grouped expressions and scalar comparisons.
- Braced `while (expr) { statements }` loops over the currently supported
  scalar expression and statement subset.
- Statement-form direct variable `++` and `--`, such as `$i++;` and `--$i;`,
  using the boxed numeric arithmetic helper path.

Unsupported today:

- Arrays, objects, functions, classes, includes, references, copy-on-write,
  resources, exceptions, array/object/reference compound-assignment lvalues,
  compound operators other than `+=`, `-=`, `*=`, `/=`, `%=`, `.=`, `&=`, and
  `|=` (`**=`, `^=`, `<<=`, `>>=`, `??=`), `print` as an expression returning
  `1` even when spelled `print(...)`, increment/decrement operators, full
  PHP numeric-string and non-numeric string arithmetic diagnostics, exact
  division/modulo-by-zero exception behavior, complete comparison parity for
  unsupported types, spaceship comparison operator, bitwise `^`, `~`, shifts,
  keyword boolean operators, chained comparison parse errors, unbraced and alternate
  control-flow syntax, `do while`, `for`, `foreach`, `switch`, `break`,
  `continue`, increment/decrement as expressions, PHP-exact increment/
  decrement semantics for strings/booleans and other edge values, complete
  overflow parity, exact scalar cast overflow behavior, PHP-exact warning
  text/file/line/error-handler behavior, inline HTML before `<?php` or between
  PHP blocks, internal functions other than `var_dump()` and `strlen()`, arrays, objects,
  resources, recursion, references, embedded NUL string handling, and full PHP
  precision/formatting edge cases for `var_dump()`/`strlen()`, doc comment retention,
  variable variables, and dynamic fallback. These are architecture targets, not
  excuses for exact-shape hacks.

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
