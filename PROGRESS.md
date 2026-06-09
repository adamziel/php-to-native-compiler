# Progress

## 2026-06-08

Restarted the project from scratch under `~/ptn-from-scratch`, guided by
`NEW_PROMPT.md`.

Integrated first vertical slice:

- Rust crate `ptn`.
- Lexer and parser for a narrow PHP subset with source spans.
- AST to PHP-aware IR lowering.
- C backend with a boxed-value runtime for `null`, booleans, integers, floats,
  and strings.
- CLI path:
  `ptn compile <input.php> -o <native-binary> [--emit-c]`.
- Native binary exercise test that compiles PHP `echo` into a C-backed native
  executable and runs it.

Unsupported edges are intentionally named rather than hidden: variables, arrays,
functions, classes, includes, references, copy-on-write, resources, exceptions,
and dynamic fallback are not implemented yet.

Next integrated production target:

- Add variables and assignment using a real symbol-table/runtime model that can
  scale to PHP references and copy-on-write.

## 2026-06-09

Recovered the checkpoint worktree onto current `origin/master` at
`ca130c503622ec9a479318d294bfd64d20e496a3` after stale pre-restart bundles
conflicted with the new from-scratch tree.

Integrated the next production slice:

- Lexer support for direct PHP variable tokens such as `$name`.
- Parser/AST support for direct assignment statements and variable reads in
  expressions.
- IR store/load instructions for named variables.
- Generated C runtime symbol table storing boxed `PtnValue` slots by name.
- Native tests proving assignment, reads, and overwrites in compiled binaries.

Added a runtime diagnostics boundary for direct variable reads:

- Generated native binaries route direct variable reads through `PtnRuntime`
  rather than treating the symbol table as an ordinary nullable map.
- Undefined direct variable reads emit a generic runtime warning and then yield
  `null`, so current echo behavior remains PHP-like without hiding the
  diagnostic boundary.
- Native tests prove defined and undefined reads in the same compiled binary.

Still unsupported: arrays, references, copy-on-write, globals/superglobals,
non-direct and reference-aware compound assignment, variable variables,
undefined-variable warning parity, functions, classes, resources, exceptions,
and dynamic fallback.

Added a non-harness differential telemetry path:

- `tools/diff-native-output.sh` compiles a file, snippet, or stdin through
  `ptn compile`, runs the native binary, runs the same input with system `php`,
  and compares stdout, stderr, and exit status.
- This does not claim PHPT compatibility. Direct PHPT execution remains blocked
  until PTN grows a PHP-compatible runner interface instead of only
  `ptn compile`.

Implemented the next boxed expression slice in the CAO worktree:

- Lexer support for `+` and `.` operator tokens.
- Parser/AST support for left-associative binary expressions with `+` binding
  tighter than string concatenation.
- IR value-expression operation nodes for numeric addition and string
  concatenation.
- Generated C code that materializes operands left-to-right into `PtnValue`
  temporaries before calling `ptn_add` and `ptn_concat`.
- Boxed runtime helpers for scalar numeric addition and scalar string
  concatenation, including basic scalar-to-number and scalar-to-string
  conversions.
- Native tests proving literal operations, variables, assignment results,
  chained expressions, numeric-string/float addition basics, and observable
  left-to-right operand evaluation in compiled binaries.

Still unsupported for binary operations: full PHP numeric-string conversion
warning parity, non-numeric string arithmetic diagnostics, complete overflow
parity, arrays, objects, references, and copy-on-write behavior.

Implemented statement-form `print` for the current supported expression subset:

- Lexer/parser/AST support for `print expr;` as a statement.
- IR lowering maps print statements to the same boxed output instruction used by
  echo, so generated C calls the existing `ptn_echo` conversion/output helper.
- Native tests prove print with literals, variables, and binary expressions.

Still unsupported for print: expression contexts such as
`$result = print "value";`, `echo print "value";`, return value `1` semantics,
and the parenthesized spelling `print(...)` in expression contexts.

Rebased the lexer/parser source-compatibility slice for comments and PHP tags
onto the print-enabled tree:

- PHP `//`, `#`, and `/* ... */` comments are skipped while preserving source
  span progression for following tokens.
- A Unix shebang is accepted only at the start of a file before `<?php`.
- The parser requires the supported `<?php` code envelope and accepts a
  trailing `?>` close tag when only whitespace follows it.
- Native tests prove comments, shebang, trailing close tags, echo, and
  statement-form print together through the compiled binary path without
  changing expression semantics.

Still unsupported for PHP tag handling: inline HTML before `<?php`, between PHP
blocks, or after a close tag; short open tags; doc comment retention; and full
PHP/HTML mode switching.

Implemented a narrow boxed unary/cast expression slice in the CAO worktree:

- Lexer support for `-`, `!`, `(`, `)`, and scalar cast keywords.
- Parser/AST support for parenthesized expression grouping, unary `-`, unary
  `!`, and `(int)`, `(float)`, `(string)`, and `(bool)` casts.
- IR value-expression operation nodes for unary operations and scalar casts.
- Generated C code that materializes operands once before calling boxed runtime
  helpers.
- Boxed runtime helpers for scalar numeric negation, PHP scalar truthiness
  negation, and scalar casts using the existing number/string conversion
  helpers.
- Native tests proving grouped negation, truthiness negation, scalar casts, and
  interaction with binary `+` and `.` expressions.

Still unsupported for unary/casts: arrays, objects, references, copy-on-write,
full numeric-string diagnostic parity, unsupported operand `TypeError` parity,
exact overflow behavior for scalar casts, and PHP increment/decrement
operators such as `++$value` and `--$value`.

Followed up after comparison/boolean integration by rejecting unsupported `++`
and `--` operators lexically. This prevents `--$value` from being compiled as
two supported unary negations before real PHP increment/decrement assignment
semantics exist.

Implemented a direct named-variable compound-assignment slice on top of the
unary/casts-enabled head:

- Lexer/parser/AST support for `$x += expr` and `$x .= expr` statements.
- IR lowering that rewrites compound assignment as a direct variable read, the
  existing boxed `+` or `.` operation, then a direct variable write.
- Native tests proving scalar add/concat compound assignment with print output,
  RHS grouping/casts through the same expression path, source-order
  undefined-variable diagnostics for the LHS read before the RHS read, and the
  unsupported boundary for other compound operators and non-direct lvalues.

Still unsupported for compound assignment: `-=`, `*=`, `/=`, `%=`, `**=`, `&=`,
`|=`, `^=`, `<<=`, `>>=`, `??=`, array/object/string-offset/property/static
property/variable-variable lvalues, references, reference identity, and
copy-on-write interactions.

Rebased the comparison/boolean expression slice after boxed `+` and `.`
integration:

- Lexer/parser support for `==`, `!=`, `<`, `>`, `&&`, `||`, and parenthesized
  expressions.
- AST/IR binary operation variants for scalar comparisons and boolean
  short-circuit expressions.
- Generated C comparison emission that materializes operands left-to-right
  before calling boxed runtime comparison helpers.
- Generated C short-circuit emission for `&&` and `||` over boxed scalar PHP
  truthiness.
- Native tests proving scalar loose comparisons, numeric-string comparison
  basics, boolean short-circuit behavior, grouping, and left-to-right operand
  diagnostics.

Still unsupported for comparisons/booleans: `===`, `!==`, `<=>`, keyword
`and`/`or`, arrays, objects, references, copy-on-write behavior, PHP-exact
chained comparison parse errors, and complete PHP comparison parity for
unsupported value types.

Refined parenthesized expression grouping in the CAO worktree:

- Parenthesized expressions are preserved explicitly in the AST before IR
  lowering erases grouping to the inner value expression.
- Native tests prove grouped literals, grouped variable reads, grouped binary
  expressions, nested grouping, and grouped assignment right-hand sides.

Extended the boxed scalar comparison slice after the grouping-preservation head:

- Lexer/parser/AST/IR support for `<=` and `>=` at the same comparison
  precedence as `<` and `>`.
- Generated C comparison emission for `<=` and `>=` reuses the shared
  `ptn_compare_order` path through boxed runtime helpers, matching the existing
  scalar loose ordering behavior.
- Native tests prove integer, numeric-string, string, null, and boolean scalar
  `<=`/`>=` cases, plus parser coverage through grouped boolean expressions.

Still unsupported after this comparison extension: identity comparisons
`===`/`!==`, spaceship `<=>`, keyword `and`/`or`, arrays, objects, references,
copy-on-write behavior, PHP-exact chained comparison parse errors, and complete
PHP comparison parity for unsupported value types.

Added boxed scalar arithmetic operators beyond addition on the scalar
comparison-extension head:

- Lexer support for `*`, `/`, and `%` operator tokens, with binary `-` reusing
  the existing minus token from unary negation while preserving lexical
  rejection for unsupported `++` and `--`.
- Parser/AST support for left-associative binary `-`, `*`, `/`, and `%`
  expressions. Multiplicative operators bind tighter than additive operators,
  arithmetic binds tighter than string concatenation, and the existing
  comparison/boolean precedence levels remain below arithmetic.
- Parser/AST/IR support for direct named-variable `-=`, `*=`, `/=`, and `%=`
  compound assignments, lowering through the same direct load, boxed binary
  operation, and direct store path as `+=` and `.=` rather than through a
  separate runtime path.
- IR operation nodes for subtraction, multiplication, division, and modulo.
- Generated C dispatch to shared `PtnValue` runtime helpers while preserving
  left-to-right operand materialization.
- Boxed runtime helpers for scalar subtraction, multiplication, division, and
  modulo using the existing scalar numeric conversion path.
- Native tests proving literals, variables, assignment results, chained
  precedence, numeric-string arithmetic basics, modulo sign behavior, direct
  arithmetic compound assignment, and observable left-to-right operand
  evaluation in compiled binaries.

Still unsupported for arithmetic: full PHP numeric-string warning parity,
non-numeric string arithmetic diagnostics, exact division/modulo-by-zero
exception behavior, complete overflow parity, arrays, objects, references, and
copy-on-write behavior.

Still unsupported for compound assignment: `**=`, `&=`, `|=`, `^=`, `<<=`,
`>>=`, `??=`, array/object/string-offset/property/static-property/
variable-variable lvalues, references, reference identity, and copy-on-write
interactions.

Implemented a narrow internal-call/output slice for scalar `var_dump()` on top
of the grouping-aware baseline:

- Lexer/parser support for identifier tokens, simple statement-form calls such
  as `var_dump(expr, ...)`, and one close-tag inline-output segment after PHP
  mode ends.
- IR lowering for generic internal-call instructions carrying a function name
  and lowered argument values.
- Generated C internal-function dispatch with `var_dump` registered as the
  first internal function rather than special-cased source text.
- Scalar `var_dump` output for current boxed `PtnValue` types: `null`,
  booleans, integers, floats, and strings.
- A minimal `phpc` runner that can compile PHPT runner scripts/snippets through
  the existing native compiler path for supported rows.
- Native tests covering scalar `var_dump`, PHPT-shaped close-tag inline output,
  left-to-right argument evaluation, unsupported complex var_dump edges, and
  support-doc coverage for unsupported edges.

Still unsupported for `var_dump`: arrays, objects, resources, recursion,
references, embedded NUL strings, full PHP float precision/formatting edge
cases, and complete PHP CLI/PHPT runner parity.

Added braced branch control flow on top of the scalar `var_dump` head:

- Lexer/parser/AST support for braced `if`, `elseif`, and `else` statements,
  including grouped branch conditions and nested statement bodies.
- IR support for structured branch instructions whose conditions and branch
  bodies preserve statement nesting for future temporary, destructor,
  exception, reference, and copy-on-write handling.
- Generated C backend support for native branches over boxed PHP values through
  the existing runtime truthiness helper.
- Branch conditions reuse the existing scalar comparison, boolean
  short-circuit, unary, cast, and grouped-expression paths.
- Native tests prove `if`/`elseif`/`else`, nested branches, scalar truthiness,
  `var_dump` in branch bodies, and source-order condition diagnostics.
- Focused public PHPT telemetry through the minimal `phpc` runner passes
  `tests/lang/001.phpt`, `tests/lang/004.phpt`, `tests/lang/005.phpt`, and
  `tests/lang/006.phpt`.

Still unsupported for control flow: unbraced statements, alternate syntax,
loops, `switch`, `break`, `continue`, branch-condition assignments/increments,
arrays/objects/resources in truthiness, and exception/finally control-flow
edges.

Added a bounded braced loop and direct variable increment/decrement slice:

- Lexer/parser/AST support for `while (expr) { statements }` with recursive
  statement bodies.
- Parser support for statement-form direct variable increment/decrement:
  `$name++;`, `++$name;`, `$name--;`, and `--$name;`.
- IR support for structured loop instructions and direct increment/decrement
  instructions. Loop conditions remain boxed value expressions and loop bodies
  preserve nested statements.
- Generated C evaluates loop conditions at the top of each iteration and uses
  the shared boxed scalar truthiness helper before emitting the body.
- Direct increment/decrement emits a runtime variable read, boxed numeric
  helper call, and runtime variable write, matching the existing direct
  variable mutation boundary.
- Native tests prove the public simple while-loop shape, prefix/postfix
  statement increments and decrements, and loop-condition rechecks.
- Focused public PHPT telemetry through `phpc` passes
  `tests/lang/002.phpt`.

Still unsupported for loops and increment/decrement: unbraced and alternate
syntax, `do while`, `for`, `foreach`, `switch`, `break`, `continue`,
increment/decrement expression result values, PHP-exact string/boolean edge
semantics, references, copy-on-write, and exception/finally loop edges.

Added boxed unary plus support:

- Parser/AST/IR support for unary `+` at the same unary precedence level as
  unary `-` and `!`.
- Generated C dispatch to a `ptn_positive` runtime helper that uses the shared
  scalar numeric conversion path and returns a boxed integer or float.
- Native tests prove grouped unary plus operands and PHP's unary precedence
  shape for `1/-2*5` and `6/+2*-3`.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/grammar/bug41401.phpt`.

Still unsupported for unary numeric conversion: arrays, objects, references,
copy-on-write, exact unsupported-operand `TypeError` parity, and full
numeric-string diagnostic parity.

Added scalar strict identity comparisons:

- Lexer/parser/AST/IR support for `===` and `!==` at equality precedence.
- Generated C comparison emission routes strict comparisons through a dedicated
  boxed runtime helper instead of reusing loose comparison coercions.
- Runtime scalar identity compares boxed type first, then scalar value for
  `null`, booleans, integers, floats, and strings. This preserves PHP scalar
  behavior such as `-0.0 === 0.0`, `1 !== 1.0`, and `"1" !== 1`.
- `phpc run <script.php>` is accepted as a wrapper-compatible alias for the
  existing compile-and-run path.
- Native tests prove strict scalar identity, negative-zero identity, and the
  `phpc run` alias.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/type_coercion/float_to_int/negative_zero_check.phpt`.

Still unsupported after this strict-comparison slice: `<=>`, keyword boolean
operators `and`/`or`, arrays, objects, resources, references, complete
comparison parity for unsupported value types, and PHP-exact chained comparison
parse errors.

Added expression-form internal calls and scalar `strlen()`:

- Parser/AST/IR support for named internal calls as value expressions, so calls
  can appear in echo operands, assignments, binary operands, and branch/loop
  conditions.
- Statement-form calls continue to lower through the same internal-call path
  and discard the boxed return value.
- Generated C internal-function dispatch now returns `PtnValue` from handlers
  while preserving left-to-right argument materialization.
- Registered `strlen()` as the second scalar internal function. It returns the
  byte length of the current boxed scalar string-conversion result.
- Native tests prove `strlen()` in expression contexts, left-to-right argument
  evaluation through call expressions, and statement-call return-value discard.
- Focused public PHPT telemetry through `phpc` passes
  `tests/func/001.phpt`.

Still unsupported for internal calls: functions other than `var_dump()` and
`strlen()`, arrays, objects, resources, references, embedded NUL byte string
length parity, PHP-exact argument parsing and diagnostics, and user-defined
functions.

Added boxed scalar bitwise `&` and `|` expressions:

- Lexer/parser/AST/IR support for binary `&` and `|` expressions with PHP-like
  precedence between equality comparisons and `&&`/`||`.
- Parser/AST/IR support for direct named-variable `&=` and `|=` compound
  assignments, lowered through the same direct read, boxed binary helper, and
  direct write path as the existing compound assignments.
- Generated C runtime helpers for PHP scalar bitwise behavior: string/string
  operands produce bytewise string results, while other currently supported
  scalar operands are converted to integers through the boxed numeric
  conversion path.
- Native tests prove integer bitwise results, bytewise string `&`/`|`,
  string compound assignment, and DEL-byte string output in compiled binaries.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/and_001.phpt` and `Zend/tests/or_001.phpt`.

Still unsupported after this bitwise slice: bitwise `^`, unary bitwise `~`, bit
shifts, compound `^=`, `<<=`, `>>=`, exact float-to-int bitwise diagnostics and
overflow parity, embedded NUL bytes in string values, arrays, objects,
references, and copy-on-write behavior.

Added a bounded braced `do while` loop slice:

- Lexer/parser/AST support for `do { statements } while (expr);` with recursive
  braced statement bodies and boxed scalar conditions.
- IR support for structured post-test loop instructions. The loop body and
  condition keep the same nested statement and value-expression shapes as
  braced `while`.
- Generated C emits the body first, then materializes the boxed condition and
  breaks when the shared PHP truthiness helper reports false.
- Native tests prove a countdown loop using direct decrement and the post-test
  behavior where the body runs once before an initially false condition is
  checked.
- Focused public PHPT telemetry through `phpc` passes
  `tests/lang/027.phpt`.

Still unsupported for loops/control flow: unbraced and alternate syntax, `for`,
`foreach`, `break`, `continue`, loop-condition assignments and increment/
decrement expressions, PHP-exact increment/decrement edge semantics,
references, copy-on-write, and exception/finally loop edges.

Added a scalar type-query internal-function slice:

- Registered `gettype()` and scalar type predicates through the existing
  generated C internal-function registry rather than through parser or output
  special cases.
- `gettype()` reports the current boxed scalar value types as PHP names:
  `NULL`, `boolean`, `integer`, `double`, and `string`.
- Added `is_null()`, `is_bool()`, `is_int()`, `is_integer()`, `is_long()`,
  `is_float()`, `is_double()`, `is_string()`, and `is_scalar()` over the
  current boxed scalar/null `PtnValue` domain.
- The registry now records a maximum argument count for fixed-arity functions
  while preserving variadic `var_dump()`.
- Native tests prove the type-query family and the public
  `tests/lang/bug30726.phpt` source shape.
- Focused public PHPT telemetry through `phpc` passes
  `tests/lang/bug30726.phpt`.

Still unsupported for type-query internals: arrays, objects, resources,
references, user-defined functions, PHP-exact argument diagnostics, and the
broader standard-library `is_*` families such as filesystem, callable,
iterable, countable, finite, infinite, and NaN checks.

Added a bounded braced switch/case/default slice on the scalar bitwise head:

- Lexer/parser/AST support for braced `switch (expr) { ... }`, `case expr:`,
  one `default:`, and simple statement-form `break;`.
- Switch bodies preserve source-order case groups for PHP fallthrough instead
  of flattening cases into independent branches.
- IR support for structured switch instructions carrying the lowered switch
  expression, case expressions, default group, and nested statement bodies.
- Generated C evaluates the switch expression once, compares case expressions
  in source order through the existing boxed loose-comparison helper, jumps to
  the matched case or default, preserves fallthrough, and lowers simple
  `break;` to the innermost emitted switch/loop end label.
- Native tests prove the public simple-switch shape, default fallthrough, and
  that case expressions stop evaluating once a match is found.
- Focused public PHPT telemetry through `phpc` passes `tests/lang/003.phpt`.

Still unsupported after this switch slice: unbraced and alternate switch
syntax, `break` with explicit levels such as `break 2`, `continue`, `for`,
`foreach`, arrays, objects, references, copy-on-write, and exception/finally
control-flow edges.

Added PHP-style source-spanned compile fatals for the `phpc` runner:

- Parser duplicate-`default` switch diagnostics now carry PHP's canonical
  "Switch statements may only contain one default clause" wording at the
  duplicate `default:` source span.
- `phpc` preserves structured compiler diagnostics and renders source-spanned
  compile failures as `Fatal error: ... in <file> on line <line>` instead of
  the internal `phpc: ... at line:column` form.
- Native/integration tests prove the parser diagnostic and `phpc` CLI fatal
  output for a duplicate switch default.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/switch/034.phpt`.

Still unsupported for diagnostics: PHP-exact warning/notice formatting,
error-handler routing, complete parse error wording, stack traces, and
diagnostic parity for unsupported runtime types and control-flow edges.

Added the remaining scalar bitwise XOR slice:

- Lexer/parser/AST/IR support for binary `^` expressions between bitwise `&`
  and `|` precedence, matching the current PHP-like scalar precedence model.
- Parser/AST/IR support for direct named-variable `^=` compound assignment,
  lowered through the same direct read, boxed binary helper, and direct write
  path as the existing direct compound assignments.
- Generated C runtime helper support for scalar XOR: string/string operands
  produce bytewise string results for non-NUL string data, while other
  currently supported scalar operands convert to integers through the existing
  scalar bitwise path.
- Registered scalar `bin2hex()` through the generated C internal-function
  registry so bytewise string XOR can be observed in native output.
- Native tests prove integer XOR, bytewise string XOR, `^=` assignment, and
  `bin2hex()` over scalar values.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/xor_002.phpt` and `Zend/tests/xor_003.phpt`.

Still unsupported after this XOR slice: unary bitwise `~`, bit shifts,
compound `<<=` and `>>=`, exact float-to-int bitwise diagnostics and overflow
parity, embedded NUL bytes in runtime strings and string bitwise results,
arrays, objects, references, and copy-on-write behavior.

Added scalar symbol-existence internal predicates:

- Registered `defined()` and `function_exists()` through the existing generated
  C internal-function registry.
- The registry now has a shared case-insensitive internal-function lookup used
  by both native dispatch and `function_exists()`.
- `function_exists()` checks the current generated internal-function registry
  and returns boxed booleans for registered and absent names.
- `defined()` checks an explicit current constant-registry boundary. Constants
  are not modeled yet, so ordinary names report as undefined rather than
  pretending user or extension constants exist.
- Native tests prove registered and absent function lookups, case-insensitive
  internal function names, `defined()` return typing, and the public
  `tests/lang/bug27443.phpt` source shape.
- Focused public PHPT telemetry through `phpc` passes
  `tests/lang/bug27443.phpt` and
  `tests/output/sapi_windows_vt100_support_notwindows.phpt`.

Still unsupported for symbol-existence internals: user-defined functions,
classes/methods, user-defined constants, PHP built-in and extension constants,
namespaced symbols, autoloading, disabled-functions behavior, and PHP-exact
argument diagnostics.

Added a scalar byte-construction internal-function slice:

- Registered `chr()` through the existing generated C internal-function
  registry, so normal calls and `function_exists()` share the same
  case-insensitive lookup table.
- `chr()` converts the current boxed scalar argument through the shared scalar
  integer-conversion path, constrains the result modulo 256, and returns a
  one-byte string in the current C-string runtime representation.
- Native tests prove ordinary byte output, newline output, byte wrapping,
  scalar string input conversion, and `function_exists("chr")`.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/strings/chr_basic.phpt`.

Still unsupported after this `chr()` slice: embedded NUL bytes in runtime
strings, exact out-of-range and float-to-int deprecation diagnostics, arrays,
objects, resources, references, and copy-on-write behavior.

Added unary scalar bitwise-not support:

- Lexer/parser/AST/IR support for unary `~` expressions at the existing unary
  precedence level.
- Generated C dispatch to a shared `ptn_bitwise_not` runtime helper rather
  than special-casing output.
- Runtime scalar bitwise-not semantics: string operands produce bytewise
  string results for non-NUL string data, while other currently supported
  scalar operands convert to integers through the same scalar bitwise integer
  path used by `&`, `|`, and `^`.
- Float operands that lose precision during bitwise integer conversion now
  emit a generic precision-loss deprecation boundary before conversion.
- Native tests prove integer, string, and float unary bitwise-not behavior,
  including observable string bytes through `bin2hex()`.
- Focused public PHPT telemetry through `phpc` passes `Zend/tests/not_001.phpt`.

Still unsupported after this bitwise-not slice: bit shifts, compound `<<=` and
`>>=`, PHP-exact file/line/error-handler behavior for float-to-int bitwise
diagnostics, bitwise integer overflow parity, embedded NUL bytes in runtime
strings and string bitwise results, arrays, objects, references, and
copy-on-write behavior.

Added a bounded braced `for` loop slice:

- Lexer/parser/AST support for `for (init; condition; update) { statements }`
  with recursive braced statement bodies.
- For-loop initializer and update clauses reuse the existing direct assignment,
  direct increment/decrement, and simple internal-call statement forms rather
  than introducing a separate mutation path.
- IR support for structured for-loop instructions carrying initializer,
  optional condition, update, and body instruction lists.
- Generated C emits initializers once, checks the boxed scalar condition before
  each iteration when present, emits the body, then emits updates. Simple
  `break;` exits the loop before updates, matching the existing loop control
  boundary.
- Native tests prove prefix/postfix update forms and break/update ordering.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/concat/concat_002.phpt`.

Still unsupported after this for-loop slice: unbraced and alternate syntax,
`foreach`, `continue`, explicit-level `break` such as `break 2`, comma
expressions in conditions, non-direct-variable clause lvalues, increment/
decrement expression result values, references, copy-on-write, and
exception/finally loop edges.

Added a scalar byte-observation internal-function slice:

- Registered `ord()` through the existing generated C internal-function
  registry, so normal calls and `function_exists()` share the same
  case-insensitive lookup table.
- Internal-call IR and generated C dispatch now preserve the source line for
  the call expression, allowing internal diagnostics to report a line boundary
  generically instead of using a fixed placeholder.
- `ord()` converts the current boxed scalar argument through the shared scalar
  string-conversion path and returns the first byte as an integer.
- Empty strings and multi-byte strings emit PHP-like deprecation diagnostics
  before returning `0` or the first byte, respectively.
- Native tests prove ordinary byte output, scalar boolean conversion,
  `chr()`/`ord()` interaction for high bytes, `function_exists("ord")`, and the
  public `ord_not_1_byte.phpt` source shape.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/strings/ord_not_1_byte.phpt`.

Still unsupported after this `ord()` slice: embedded NUL bytes in runtime
strings, strict-types/internal argument `TypeError` parity, arrays, objects,
resources, references, and copy-on-write behavior.

Added a scalar bit-shift and modeled-constant slice:

- Lexer/parser/AST/IR support for binary `<<` and `>>` expressions at PHP-like
  shift precedence between concatenation and additive arithmetic.
- Generated C dispatch to boxed `ptn_shift_left` and `ptn_shift_right` runtime
  helpers after materializing operands left-to-right.
- Shift operands convert through the current bitwise integer-conversion path,
  sharing the existing scalar numeric-string conversion and float precision-loss
  diagnostic boundary used by `&`, `|`, `^`, and unary `~`.
- Bare identifier expressions now lower as constant reads when they are not
  followed by a call argument list.
- The constant registry now models PHP `E_ERROR`, and `defined("E_ERROR")`
  observes that registry entry while ordinary names remain undefined.
- Registered `error_reporting()` as a zero-or-one-argument internal function
  with the current placeholder reporting-level behavior.
- Native tests prove shift parsing, numeric-string shifts, `E_ERROR` constant
  reads, and `defined("E_ERROR")`.
- Focused public PHPT telemetry through `phpc` passes
  `tests/lang/operators/bitwiseShiftLeft_variationStr2.phpt` and
  `tests/lang/operators/bitwiseShiftRight_variationStr2.phpt`.

Still unsupported after this shift slice: compound `<<=` and `>>=`, PHP-exact
`error_reporting()` configuration/filtering behavior, user-defined constants,
PHP built-in and extension constants beyond `E_ERROR`, exact shift diagnostic
formatting and exception behavior, bitwise integer overflow parity, embedded
NUL bytes in runtime strings and string bitwise results, arrays, objects,
references, and copy-on-write behavior.

Added a scalar base-conversion internal-function slice:

- Registered `bindec()`, `hexdec()`, and `octdec()` through the existing
  generated C internal-function registry, so normal calls and
  `function_exists()` share the same case-insensitive lookup table.
- The three internals convert the current boxed scalar argument through the
  shared scalar string-conversion path, trim surrounding ASCII whitespace,
  accept their matching PHP base prefixes (`0b`, `0x`, `0o`), and parse valid
  base digits into boxed integers until native integer range is exceeded, then
  boxed floats.
- Invalid base-conversion characters are ignored after emitting a generic
  deprecation boundary through the existing line-aware internal diagnostic path.
- Native tests prove ordinary conversion, case-insensitive registry exposure,
  invalid-character diagnostics, and the public `variation2.phpt` prefix-only
  source shapes.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/math/bindec_variation2.phpt`,
  `ext/standard/tests/math/hexdec_variation2.phpt`, and
  `ext/standard/tests/math/octdec_variation2.phpt`.

Still unsupported after this base-conversion slice: arrays, objects, resources,
references, copy-on-write behavior, exact unsupported-type diagnostics, embedded
NUL strings, and complete PHP precision/range parity for very large
base-conversion inputs.

Added direct scalar compound shift assignment:

- Lexer/parser/AST/IR support for direct named-variable `<<=` and `>>=`
  compound assignment, lowering through the existing direct load, boxed shift
  helper, and direct store path used by other compound operators.
- Fully qualified bare constant reads such as `\PHP_EOL` now parse through the
  constant-read expression path, and the modeled constant registry includes
  `PHP_EOL` alongside `E_ERROR`.
- Native tests prove compound shift assignment over scalar float literals and
  `\PHP_EOL` output through generated native binaries.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/type_coercion/float_to_int/no_warnings_compatible_float_literals_assignment_ops.phpt`.

Still unsupported after this compound-shift slice: `**=` and `??=`, array/
object/string-offset/property/static-property/variable-variable compound
assignment lvalues, references and copy-on-write behavior, PHP-exact
diagnostic formatting for shift/bitwise conversions, user-defined constants,
and built-in or extension constants beyond the currently modeled `E_ERROR` and
`PHP_EOL`.

Added scalar `ceil()` and `floor()` internal functions:

- Registered `ceil()` and `floor()` through the existing generated C
  internal-function registry, so normal calls and `function_exists()` share the
  same case-insensitive lookup table.
- The functions convert the current boxed scalar argument through the shared
  scalar numeric-conversion path and return boxed floats.
- Generated C now links the standard math library for runtime scalar math
  helpers that need it.
- Native tests prove the public `floorceil.phpt` source shape and
  case-insensitive registry exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/math/floorceil.phpt`.

Still unsupported after this `ceil()`/`floor()` slice: arrays, objects,
resources, references, copy-on-write behavior, PHP-exact null deprecations,
string and unsupported-type diagnostics, and complete special-float parity.

Added finite/infinite/NaN scalar predicates and constants:

- The modeled constant registry now includes PHP `INF` and `NAN` alongside the
  existing scalar constants.
- Registered `is_finite()`, `is_infinite()`, and `is_nan()` through the
  generated C internal-function registry, so normal calls and
  `function_exists()` share the existing case-insensitive lookup table.
- The predicates operate over the current boxed scalar value domain, returning
  PHP booleans and using C `isfinite`, `isinf`, and `isnan` for boxed floats.
- Native tests prove the public `bug74039.phpt` source shape for positive
  infinity, negative infinity, and NaN, plus case-insensitive registry
  exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/math/bug74039.phpt`.

Still unsupported after this non-finite float slice: complete PHP
float-formatting parity for `NAN`/`INF` in all output paths, full comparison
parity for non-finite floats outside these predicates, user-defined constants,
and extension constants beyond the currently modeled scalar constants.

Added scalar PHP integer limit constants:

- The constant registry now models `PHP_INT_MIN`, `PHP_INT_MAX`, and
  `PHP_INT_SIZE` alongside the existing scalar constants.
- Bare constant reads and `defined()` share the same modeled constant lookup,
  so the integer limits are observable through both paths.
- Native tests prove the 64-bit integer min/max/size values and `defined()`
  visibility.
- Focused public PHPT telemetry through `phpc` passes
  `tests/lang/constants/PHP_INT_64bit.phpt`.

Still unsupported after this constant slice: user-defined constants,
namespace-sensitive constants, dynamic `constant()`, and built-in or extension
constants beyond the currently modeled scalar registry entries.

Added NaN-aware scalar comparison ordering:

- The boxed scalar numeric comparison helper now reports unordered results when
  either numeric operand is `NAN` instead of collapsing the comparison to
  equality.
- Loose equality and ordered operators map unordered numeric comparisons to
  PHP-style false results, while `!=`/`!==` remain true through the existing
  comparison emission.
- Mixed scalar comparison branches for booleans, null, and strings stay on the
  existing PHP-shaped paths instead of treating every `NAN` operand alike.
- Native tests prove the public `nan-comparison-false.phpt` source shape plus
  nearby bool/null/string `NAN` comparison behavior.
- Focused public PHPT telemetry through `phpc` passes
  `tests/lang/operators/nan-comparison-false.phpt`.

Still unsupported after this NaN comparison slice: exact `NAN`/`INF`
formatting, arrays, objects, resources, references, copy-on-write behavior,
spaceship comparison, and complete comparison parity for unsupported value
types.

Added scalar string ROT13 and comparison internal functions:

- Registered `str_rot13()` and `strcmp()` through the existing generated C
  internal-function registry, so normal calls and `function_exists()` share the
  same case-insensitive lookup table.
- `str_rot13()` converts the current boxed scalar argument through the shared
  scalar string-conversion path and applies ASCII ROT13 while leaving
  non-letters unchanged.
- `strcmp()` converts both boxed scalar arguments through the shared scalar
  string-conversion path and returns a negative integer, zero, or a positive
  integer from the current bytewise C-string comparison.
- Native tests prove the public `str_rot13_basic.phpt` source shape, including
  nested internal calls, braced branches, assignment round trips, scalar
  comparison of `strcmp()` output, and case-insensitive registry exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/strings/str_rot13_basic.phpt`.

Still unsupported after this `str_rot13()`/`strcmp()` slice: arrays, objects,
resources, references, copy-on-write behavior, embedded NUL strings, exact
binary-string comparison parity, and exact unsupported-type diagnostics.

Added scalar `pi()` and the `M_PI` math constant:

- Registered `pi()` through the existing generated C internal-function
  registry, so normal calls and `function_exists()` share the same
  case-insensitive lookup table.
- The modeled constant registry now includes `M_PI` alongside the existing
  scalar constants, so direct constant reads and `defined()` share the same
  lookup boundary.
- `pi()` and `M_PI` return the same boxed float value through the current scalar
  runtime paths.
- Native tests prove the public `pi_basic.phpt` source shape, direct `M_PI`
  output, and registry exposure through `function_exists()` and `defined()`.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/math/pi_basic.phpt`.

Still unsupported after this `pi()`/`M_PI` slice: the rest of the predefined
math constants, exact precision/formatting parity across all output paths, and
user-defined or extension constants beyond the currently modeled scalar
constant registry.

Added scalar `sqrt()`:

- Registered `sqrt()` through the existing generated C internal-function
  registry, so normal calls and `function_exists()` share the same
  case-insensitive lookup table.
- `sqrt()` converts the current boxed scalar argument through the shared scalar
  numeric-conversion path and returns a boxed float from the generated C runtime
  math helper.
- Native tests prove the public `sqrt_basic.phpt` source shape and
  case-insensitive registry exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/math/sqrt_basic.phpt`.

Still unsupported after this `sqrt()` slice: PHP-exact diagnostics for null,
string, array/object/resource/reference operands, complete negative/non-finite
float parity, and precision/formatting parity beyond the current scalar output
paths.

Added direct variable interpolation in double-quoted strings:

- The lexer, AST, parser, and IR now represent double-quoted strings with direct
  `$name` interpolation separately from plain string literals.
- Direct interpolation lowers through ordinary runtime variable reads, scalar
  string casts, and boxed concatenation helpers instead of a separate output
  path.
- Escaped `\$name` remains literal text.
- Native tests prove parser representation, direct interpolation output, and the
  public nested switch/for source shape from `tests/lang/020.phpt`.
- Focused public PHPT telemetry through `phpc` passes `tests/lang/020.phpt`.

Still unsupported after this interpolation slice: complex/braced
interpolation, interpolation of arrays, objects, offsets, properties, variable
variables, references, copy-on-write interactions, embedded NUL string parity,
and PHP-exact interpolation diagnostics.

Added numeric literal separator and radix parsing:

- The lexer now accepts PHP digit separators between digits in decimal integer,
  float, exponent, hexadecimal, binary, and legacy-octal numeric literals.
- Hexadecimal `0x`/`0X`, binary `0b`/`0B`, and legacy-octal integer literals
  lower to the existing integer literal token path, while decimal/exponent
  forms lower to the existing integer or float token paths.
- Separator placement stays lexical: separators are consumed only between
  valid digits for the current radix or decimal/exponent component, leaving
  invalid adjacent/trailing separator text for the normal parser diagnostic
  path.
- Native tests prove token values across decimal, float, exponent, hex, binary,
  and octal forms plus the public valid numeric-literal-separator source shape.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/numeric_literal_separator/numeric_literal_separator_001.phpt`.

Still unsupported after this numeric-literal slice: exact numeric literal
overflow/range parity and PHP-exact parse diagnostic wording for invalid
separator placements.

Added modeled PHP math constants and shortest round-trip finite float
`var_dump()` formatting:

- The modeled constant registry now includes `M_E`, `M_LOG2E`, `M_LOG10E`,
  `M_LN2`, `M_LN10`, `M_PI_2`, `M_PI_4`, `M_1_PI`, `M_2_PI`, `M_SQRTPI`,
  `M_2_SQRTPI`, `M_LNPI`, `M_EULER`, `M_SQRT2`, `M_SQRT1_2`, and `M_SQRT3`
  alongside the existing `M_PI` value.
- Bare constant reads and `defined()` share the same constant lookup boundary.
- `var_dump()` finite float formatting now chooses the shortest decimal
  spelling that round-trips to the same native double, while preserving `INF`,
  `-INF`, and `NAN` spelling for non-finite values.
- Native tests prove the public `constants_basic.phpt` source shape and
  representative `defined()` visibility.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/math/constants_basic.phpt`.

Still unsupported after this math-constants slice: user-defined or extension
constants beyond the modeled scalar constant registry, full PHP float
precision/formatting parity outside the current scalar `var_dump()` path, and
array/object/resource/reference output semantics.

Added scalar `getrandmax()` and `getmypid()` internals:

- Registered `getrandmax()` and `getmypid()` through the generated C
  internal-function registry, so normal calls and `function_exists()` share the
  same case-insensitive lookup table.
- `getrandmax()` returns the modeled maximum random integer as a boxed scalar
  integer.
- `getmypid()` returns the generated native process id through `_getpid()` on
  Windows and `getpid()` elsewhere.
- Native tests prove both public PHPT source shapes plus case-insensitive
  registry exposure for each function.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/math/getrandmax_basic.phpt` and
  `ext/standard/tests/general_functions/getmypid_basic.phpt`.

Still unsupported after this `getrandmax()`/`getmypid()` slice: random-number
generation state, seeding, platform-specific RNG range variation, other random
APIs, PHP-exact SAPI/process-control interactions, and unavailable-platform
diagnostics.

Added modeled directory/path separator constants:

- The modeled constant registry now includes `DIRECTORY_SEPARATOR` and
  `PATH_SEPARATOR` using target-platform C preprocessor values.
- Bare constant reads and `defined()` share the same constant lookup boundary.
- Native tests prove direct reads and registry visibility.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/directory/directory_constants.phpt`.

Still unsupported after this directory-constant slice: other directory
extension constants and PHP-exact constant availability differences across
SAPIs or unusual target platforms.

Added scalar `hex2bin()`:

- Registered `hex2bin()` through the generated C internal-function registry,
  so normal calls and `function_exists()` share the same case-insensitive
  lookup table.
- `hex2bin()` decodes hexadecimal byte pairs from the current boxed scalar
  string-conversion path.
- Odd-length or non-hexadecimal input returns `false` with a warning boundary.
- Native tests prove the public `hex2bin_basic.phpt` source shape, nested
  `bin2hex(hex2bin(...))`, registry exposure, and invalid-input warnings.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/strings/hex2bin_basic.phpt`.

Still unsupported after this `hex2bin()` slice: embedded-NUL decoded output
parity from the current C-string-backed runtime, exact warning file names and
line text, and unsupported array/object/resource/reference diagnostics.

Added scalar exponentiation operators:

- The lexer and parser now accept `**` and direct named-variable `**=`.
- `**` parses as right-associative with PHP precedence relative to unary
  operators and casts, so `-3 ** 2` groups as `-(3 ** 2)`.
- Direct `$x **= expr` lowers through the existing compound-assignment path:
  read the variable, evaluate the right-hand expression, call the boxed power
  helper, then write the variable.
- The generated C runtime keeps integer results for non-negative integer
  exponents that fit in `int64_t`, with `pow()` fallback for other scalar
  numeric cases.
- Native tests prove precedence, associativity, grouped exponentiation, `**=`,
  and the public `pow-operator.phpt` source shape.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/math/pow-operator.phpt`.

Still unsupported after this exponentiation slice: PHP-exact numeric-string
diagnostics, full overflow/exception parity, array/object/resource/reference
operands, non-direct-variable `**=` lvalues, and compound `??=`.

Added deprecated non-canonical `(boolean)` casts:

- The lexer/parser now distinguish `(boolean)` from canonical `(bool)` while
  preserving the normal cast expression path and source line.
- IR cast values carry source line metadata so the backend can emit PHP-like
  deprecation diagnostics for the non-canonical spelling.
- Generated C emits the deprecation and then delegates to the existing boxed
  bool conversion helper.
- Native tests prove parser distinction and the public PHPT source shape.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/type_coercion/type_casts/non_canonical_boolean_cast.phpt`.

Still unsupported after this non-canonical cast slice: `(integer)`, `(double)`,
`(real)`, `(binary)`, `(unset)`, array/object casts, and exact scalar cast
overflow diagnostics.

Added global-scope magic constants:

- The AST and IR now represent magic constants separately from ordinary bare
  constants for `__LINE__`, `__FILE__`, `__DIR__`, `__FUNCTION__`,
  `__METHOD__`, `__CLASS__`, `__TRAIT__`, and `__NAMESPACE__`.
- Parser recognition is case-insensitive and happens before ordinary constant
  lookup.
- `compile_file()` passes source file and parent-directory metadata into IR
  lowering so generated C can emit global `__FILE__` and `__DIR__` values.
- Global-scope `__LINE__` emits the source line; scope-dependent magic
  constants currently emit empty strings until functions, classes, traits, and
  namespaces exist.
- Native tests prove parser recognition and the public PHPT source shape.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/constants/magic_const_in_global_scope.phpt`.

Still unsupported after this magic-constant slice: scope-aware magic constant
values inside functions, methods, classes, traits, namespaces, includes, and
eval contexts.

Extended deprecated non-canonical scalar cast aliases:

- The lexer/parser now accept `(integer)`, `(double)`, and `(binary)` as cast
  spellings distinct from canonical `(int)`, `(float)`, and `(string)`.
- AST and IR cast nodes preserve the non-canonical cast kind and source line so
  generated runtime diagnostics can be emitted before conversion.
- Generated C uses a shared `ptn_cast_noncanonical()` helper for all supported
  non-canonical spellings, including the already-supported `(boolean)`, then
  dispatches to the canonical boxed scalar cast helpers.
- Native tests prove parser distinction and the public non-canonical cast PHPT
  source shapes.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/type_coercion/type_casts/non_canonical_integer_cast.phpt`,
  `Zend/tests/type_coercion/type_casts/non_canonical_double_cast.phpt`, and
  `Zend/tests/type_coercion/type_casts/non_canonical_binary_cast.phpt`.

Still unsupported after this non-canonical cast extension: `(real)`, `(unset)`,
array/object casts, and exact scalar cast overflow diagnostics.

Added scalar `soundex()`:

- Registered `soundex()` through the existing generated C internal-function
  registry, so normal calls and `function_exists()` share the same
  case-insensitive lookup table.
- Added PHP-style ASCII soundex key generation over the current boxed scalar
  string-conversion result, including leading non-letter skipping, duplicate
  code suppression, vowel/`h`/`w`/`y` reset behavior, and `0000` for inputs
  without ASCII letters.
- Native tests prove the public `soundex_basic.phpt` source shape, registry
  exposure, and reset edge cases such as `Ashcraft`, `Tymczak`, and `Pfister`.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/strings/soundex_basic.phpt`.

Still unsupported after this `soundex()` slice: PHP-exact behavior for
unsupported array/object/resource/reference operands, locale/non-ASCII
collation differences beyond the current ASCII scan, exact diagnostics for
unsupported value domains, and embedded-NUL string storage parity.

Added scalar `dirname()`:

- Registered `dirname()` through the generated C internal-function registry, so
  normal calls and `function_exists()` share the same case-insensitive lookup
  table.
- `dirname()` returns the parent directory from the current boxed scalar
  string-conversion path, recognizing both `/` and `\` separators.
- Native tests prove the public `dir-constant-normal.phpt` source shape and
  registry exposure.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/constants/dir-constant-normal.phpt`.

Still unsupported after this `dirname()` slice: PHP-exact path edge behavior for
unusual roots/trailing separators, embedded NUL path strings, and unsupported
array/object/resource/reference operands.

Added basic labels, `goto`, and single-statement `if` bodies for jump control
flow:

- The lexer/parser/AST/IR now support user labels such as `L1:` and
  `goto L1;` statements in the currently generated main function.
- `if`, `elseif`, and `else` bodies may now be either braced blocks or a single
  supported statement, preserving the existing braced lowering path.
- Generated C emits PHP label/goto statements as deterministic prefixed C
  labels, keeping the implementation generic rather than PHPT-output-specific.
- Native tests prove representative public jump source shapes, and focused
  public PHPT telemetry through `phpc` passes `Zend/tests/jump/jump01.phpt`,
  `Zend/tests/jump/jump02.phpt`, `Zend/tests/jump/jump03.phpt`, and
  `Zend/tests/jump/jump04.phpt`.

Still unsupported after this jump slice: PHP-exact invalid-goto diagnostics and
restrictions for jumps into or out of forbidden scopes, labels/goto inside
unsupported functions/classes/try/finally constructs, alternate control-flow
syntax, `continue`, `foreach`, explicit-level `break`, and unbraced loop/switch
bodies.

Added source-spanned parse diagnostics for removed `(real)` casts:

- Compiler diagnostics now carry a generic kind so `phpc` can render
  source-spanned parser errors as `Parse error:` while preserving existing
  source-spanned compiler fatals as `Fatal error:`.
- The parser recognizes only the exact removed cast-prefix syntax `(real)` at
  the cast boundary, reports PHP's canonical removal message, and does not
  lower it as a runtime cast or deprecated alias.
- Native/CLI tests prove the parser diagnostic kind and `phpc` parse-error
  rendering while preserving duplicate-switch-default fatal rendering.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/type_coercion/type_casts/real_cast.phpt`.

Still unsupported after this parse-diagnostic slice: broader parse-error
wording parity, `(unset)`, array/object casts, stack traces, error-handler
routing, and PHP-exact warning/notice formatting.

Added short literal arrays and array comparison/spaceship support:

- The lexer/parser/AST/IR now support short array literals with optional
  `key => value` entries.
- The generated runtime represents ordered arrays with integer/string key
  canonicalization, automatic integer keys, and duplicate-key replacement.
- Loose equality, strict identity, ordered comparison, and `<=>` now cover the
  current literal-array value subset alongside scalar comparisons.
- `var_dump()`, `gettype()`, truthiness, and scalar conversion boundaries now
  name array values instead of treating them as wholly unsupported.
- Native tests prove nested literal parsing, array `<=>`, key identity, registry
  interactions, and public bug source shapes.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/bug69891.phpt`, `Zend/tests/bug69892.phpt`, and
  `Zend/tests/bug69893.phpt`.

Still unsupported after this array comparison slice: long-form `array(...)`,
array element access/mutation, append/unset/iteration, references,
copy-on-write, recursive arrays, array unpacking, object/resource comparison
parity, keyword boolean operators, chained comparison parse-error parity, and
exact diagnostics.

Added source-spanned fatal diagnostics for removed `(unset)` casts:

- The parser recognizes only the exact removed cast-prefix syntax `(unset)` at
  the cast boundary, reports PHP's canonical removal message, and does not
  lower it as a runtime cast or deprecated alias.
- Native/CLI tests prove the parser diagnostic and `phpc` fatal-error
  rendering path.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/unset/unset_cast_removed.phpt`.

Still unsupported after this removed-cast diagnostic slice: broader
parse-error/fatal wording parity, array/object casts, stack traces,
error-handler routing, and PHP-exact warning/notice formatting.
