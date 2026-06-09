# Architecture

PTN starts with a conservative native backend while preserving the architecture
needed for full PHP compatibility.

Current pipeline:

```text
PHP source
-> lexer tokens with source spans
-> AST
-> PHP-aware IR
-> generated C with boxed-value runtime
-> native executable via system C compiler
```

The generated C backend is the first native-code path, not the final backend.
It lets the project exercise binary production from day one while the runtime
model grows toward full PHP semantics.

Current runtime/compiler slices:

- The lexer recognizes the supported PHP code envelope: optional byte-zero
  Unix shebang, required `<?php`, PHP comments inside the code region, and one
  `?>` close tag that switches to inline output through EOF. Inline HTML before
  `<?php` and multi-block PHP/HTML mode switching remain unsupported.
- Numeric literal lexing accepts PHP digit separators between valid digits and
  routes decimal, exponent float, legacy octal, binary `0b`/`0B`, and
  hexadecimal `0x`/`0X` spellings into the existing integer/float AST literal
  paths.
- Direct variables lower to generated C `PtnRuntime` symbol-table load/store
  calls.
- Direct variable reads pass through a runtime helper that emits a generic
  undefined-variable warning before yielding `null`.
- Scalar binary `+`, `-`, `*`, `/`, `%`, `.`, `&`, `^`, `|`, `<<`, and `>>` expressions lower
  to IR value-expression operation nodes. The C backend materializes operands
  into `PtnValue` temporaries in source order before calling boxed runtime
  helpers such as `ptn_add`, `ptn_subtract`, `ptn_multiply`, `ptn_divide`,
  `ptn_modulo`, `ptn_concat`, `ptn_bitwise_and`, `ptn_bitwise_xor`, and
  `ptn_bitwise_or`, `ptn_shift_left`, and `ptn_shift_right`.
  String/string bitwise operands use bytewise string helpers for non-NUL string
  data; other supported scalar operands convert through the current numeric
  path before integer bitwise operations. Shift operands always use the current
  bitwise integer-conversion path.
- Direct named-variable `+=`, `-=`, `*=`, `/=`, `%=`, `.=`, `&=`, `^=`, `|=`,
  `<<=`, and `>>=` lower in IR as a direct variable load, the same boxed binary
  helper used by the ordinary binary operator, and a direct variable store. This
  keeps left-to-right reads and undefined-variable diagnostics on the runtime
  read boundary rather than adding a separate compound-assignment runtime path.
- Statement-form `print expr;` lowers to the same boxed output IR instruction
  used by echo, so generated native code routes print output through the
  existing `ptn_echo` helper.
- Parenthesized expressions are parsed as grouping, while unary `+`, unary `-`,
  unary `!`, unary bitwise `~`, and scalar `(int)`, `(float)`, `(string)`, and
  `(bool)` casts lower to IR value-expression operation nodes. The C backend
  emits boxed runtime helper calls such as `ptn_positive`, `ptn_negate`,
  `ptn_not`, `ptn_bitwise_not`, and `ptn_cast_*`.
- Double-quoted strings with direct `$name` interpolation lower to ordinary
  value-expression concatenation: literal string segments, runtime variable
  reads, scalar string casts, and the existing boxed concat helper. Complex and
  braced interpolation remain outside this slice.
- Increment/decrement expression contexts are rejected while full PHP
  pre/post-increment value semantics are unsupported, so statement-form direct
  variable support is not confused with expression result behavior.
- Scalar comparison and boolean expressions share the same AST/IR binary node
  shape. Comparisons emit boxed booleans through runtime helpers, while `&&`
  and `||` emit native C branches that short-circuit over boxed PHP truthiness.
  The ordered comparison helpers share `ptn_compare_order`, so `<`, `<=`, `>`,
  and `>=` use one scalar ordering path. Numeric scalar comparisons involving
  `NAN` report an unordered result so equality and ordered comparisons return
  false. Strict scalar identity uses a separate helper that compares boxed type
  and value without numeric-string or boolean coercion.
- Simple calls lower to IR internal-call value expressions carrying a
  normalized function name, lowered arguments, and the source line for current
  internal diagnostic boundaries. The generated C backend materializes
  arguments left-to-right and dispatches through a small internal function
  registry. Statement-form calls discard the returned boxed value. `var_dump`
  formats boxed scalar runtime values and returns `null`, with finite floats
  using the shortest decimal spelling that round-trips to the same native
  double while preserving `INF`, `-INF`, and `NAN` spellings; `strlen` returns
  the byte length of the current boxed scalar string conversion; `str_rot13`
  returns ASCII ROT13 output for that string conversion; `strcmp` compares two
  scalar string-conversion results through the current C-string-backed bytewise
  path; `bin2hex` returns lowercase hexadecimal bytes for that same string
  conversion; `hex2bin` decodes hexadecimal byte pairs from the current scalar
  string conversion and returns `false` with a warning boundary for invalid
  input; `ceil` and `floor` return boxed floats after the current scalar
  numeric-conversion path; `sqrt` returns a boxed float after that same numeric
  conversion path; `pi` returns the boxed `M_PI` math constant; `getrandmax`
  returns the modeled maximum random integer; `getmypid` returns the generated
  native process id;
  `bindec`, `hexdec`, and `octdec` parse scalar string-conversion results
  through shared base-conversion helpers with prefix handling and line-aware
  deprecation diagnostics; `chr` constructs one-byte strings from scalar
  integer conversion; `ord` observes the first byte of scalar string
  conversion; `error_reporting` currently accepts zero or one argument and
  returns a placeholder reporting level; `gettype` and scalar `is_*` predicates
  query the current boxed scalar/null value domain, while `is_finite`,
  `is_infinite`, and `is_nan` query modeled non-finite float constants;
  `function_exists` shares the registry lookup path; and `defined` checks the
  current constant-registry boundary, which includes the modeled PHP `E_ERROR`,
  `PHP_EOL`, `DIRECTORY_SEPARATOR`, `PATH_SEPARATOR`, `PHP_INT_MIN`,
  `PHP_INT_MAX`, `PHP_INT_SIZE`, `INF`, `NAN`, `M_PI`, and modeled PHP math
  `M_*` constants from `constants_basic.phpt`. Fixed-arity internal functions
  record min/max arity metadata while `var_dump` remains variadic.
- Braced `if`, `elseif`, and `else` statements lower to structured IR branch
  instructions. Conditions remain boxed value expressions, and the C backend
  emits native branches that call the shared scalar truthiness helper.
- `elseif` is represented as an else branch containing another structured
  branch, so future exception edges, temporaries, destructor timing,
  references, and copy-on-write behavior can stay attached to the statement
  tree.
- Braced `while` statements lower to structured IR loop instructions. The C
  backend evaluates the boxed condition at the top of each iteration and uses
  the shared scalar truthiness helper before emitting the loop body.
- Braced `do while` statements lower to structured IR loop instructions with
  the same boxed condition and nested statement representation as `while`. The
  C backend emits the body before materializing the condition and breaking on
  falsey boxed truthiness.
- Braced `for` statements lower to structured IR loop instructions with
  initializer, optional condition, update, and body instruction lists. The C
  backend emits initializers once, checks boxed scalar truthiness before each
  iteration when a condition is present, emits the body, then emits updates.
- Statement-form direct variable increment/decrement lowers to a runtime read,
  boxed numeric increment/decrement helper, and runtime write. Expression-value
  semantics for pre/post increment remain outside this slice.

Near-term architecture targets:

- PHP ordered arrays.
- References and copy-on-write.
- Function and class metadata.
- Broader diagnostics and exception channels.
- Full PHP numeric-string conversions, non-numeric string arithmetic
  diagnostics, warnings, scalar cast overflow behavior, exact
  division/modulo-by-zero exception behavior, and complete overflow behavior for
  arithmetic helpers.
- PHP-exact file/line/error-handler behavior for bitwise float-to-int
  precision-loss diagnostics, and overflow parity for bitwise integer
  conversions and shifts.
- Array, object, and reference lvalues for compound assignment, plus
  unsupported compound operators beyond `+=`, `-=`, `*=`, `/=`, `%=`, `.=`,
  `&=`, `|=`, `^=`, `<<=`, and `>>=`: `**=` and `??=`.
- Complete comparison parity for arrays, objects, references, chained
  comparison parse errors, spaceship operator, keyword boolean operators, and
  unsupported scalar edge cases.
- A broader internal-function module system with shared argument parsing,
  metadata, unsupported array/object/resource/reference diagnostics, and
  PHP-exact `var_dump` precision/formatting beyond the current scalar
  round-trip float path plus `strlen`/`bin2hex`/`hex2bin` byte-string behavior,
  `strcmp` binary-string parity, scalar math diagnostic/type parity,
  base-conversion precision/range parity, and PHP-exact `getmypid` process
  model parity across SAPIs and unsupported platforms.
- User-defined functions, classes/methods, constants beyond the currently
  modeled `E_ERROR`, `PHP_EOL`, `PHP_INT_MIN`, `PHP_INT_MAX`, `PHP_INT_SIZE`,
  `INF`, `NAN`, `M_PI`, and modeled PHP math `M_*` constants, namespaced
  symbols, autoloading, and
  disabled-functions behavior in symbol-existence predicates.
- Broader control flow: unbraced and alternate syntax, `foreach`,
  explicit-level `break`, `continue`, for-loop comma expressions and
  non-direct-variable clause lvalues, and exception/finally edges.
- Full PHP increment/decrement semantics, including expression result values,
  strings, booleans, arrays/objects, references, and copy-on-write behavior.
- Explicit fallback boundaries for `eval`, variable variables, and runtime
  symbol mutation.
