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

Added explicit-level `break` and single-statement loop bodies:

- The parser now accepts `break N;` while preserving plain `break;` as level 1,
  and `while`, `do while`, and `for` bodies may be a single supported statement
  instead of only a braced block.
- Break levels lower through IR with source line metadata.
- The C backend maintains a stack of emitted loop/switch exit labels so
  `break N;` exits the requested active control target, and emits
  source-spanned generated-binary fatals when a level is not valid.
- Native tests prove nested switch/loop break levels, single-statement loop
  bodies, and invalid large break-level diagnostics.
- Focused public PHPT telemetry through `phpc` passes `tests/lang/021.phpt`
  and `Zend/tests/bug77660.phpt`.

Still unsupported after this control-flow slice: `continue`, `foreach`,
alternate control-flow syntax, unbraced switch bodies, branch-condition
assignments, for-loop comma expressions and non-direct-variable clause lvalues,
PHP-exact break/continue diagnostic timing/wording, invalid-goto restrictions,
and exception/finally edges.

Added source-spanned parse diagnostics for expression-context `(void)` casts:

- The parser recognizes exact `(void)` cast-prefix syntax while parsing an
  expression and reports PHP's unexpected-token parse error instead of falling
  through to a misleading statement terminator diagnostic.
- The diagnostic does not lower `(void)` as a runtime cast.
- Native/CLI tests prove the parser diagnostic and `phpc` parse-error
  rendering path.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/type_coercion/type_casts/cast_to_void_statement.phpt`.

Still unsupported after this diagnostic slice: statement-form `(void) expr;`
casts, broader parse-error wording parity, stack traces, error-handler routing,
and PHP-exact warning/notice formatting.

Added scalar `str_contains()`:

- Registered `str_contains()` through the existing generated C internal-function
  registry, so normal calls and `function_exists()` share the same
  case-insensitive lookup table.
- `str_contains()` converts both arguments through the current boxed scalar
  string-conversion path and returns whether the needle occurs in the haystack
  through the current C-string-backed string representation.
- Native tests prove the public `str_contains.phpt` source shape, scalar
  conversion basics, and registry exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/strings/str_contains.phpt`.

Still unsupported after this `str_contains()` slice: PHP-exact binary-string
behavior for embedded NUL values, unsupported array/object/resource/reference
operand diagnostics, and broader string runtime parity.

Added undefined `goto` label diagnostics:

- After parsing the supported statement tree, the parser now collects user
  labels and validates every `goto` target before IR lowering and C code
  generation.
- Undefined labels produce a source-spanned PHP-style fatal diagnostic instead
  of falling through to a C compiler error for an undefined generated label.
- Native/CLI tests prove parser validation and `phpc` fatal rendering.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/jump/jump06.phpt`.

Still unsupported after this goto diagnostic slice: duplicate labels,
forbidden-scope goto restrictions for jumps into or out of invalid scopes,
labels/goto inside unsupported functions/classes/try/finally constructs, and
PHP-exact invalid-goto diagnostic wording for broader cases.

Added duplicate label diagnostics:

- The parser now rejects repeated user labels during the same recursive
  validation pass used for `goto` target checks.
- Duplicate labels produce a source-spanned PHP-style fatal diagnostic at the
  second label span instead of falling through to duplicate generated C labels.
- Native/CLI tests prove parser validation and `phpc` fatal rendering.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/duplicate_label_error.phpt`.

Still unsupported after this label diagnostic slice: forbidden-scope goto
restrictions for jumps into or out of invalid scopes, labels/goto inside
unsupported functions/classes/try/finally constructs, and PHP-exact
invalid-goto diagnostic wording for broader cases.

Added scalar `fdiv()`:

- Registered `fdiv()` through the generated C internal-function registry, so
  normal calls and `function_exists()` share the same case-insensitive lookup
  table.
- `fdiv()` converts both operands through the current boxed scalar
  numeric-conversion path and returns boxed IEEE floating-point division
  results, preserving signed zeroes, infinities, and `NAN`.
- Native tests prove the public `fdiv.phpt` source shape, scalar conversion,
  and registry exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/math/fdiv.phpt`.

Still unsupported after this `fdiv()` slice: exact unsupported operand
diagnostics for arrays, objects, resources, and references, strict type
handling, and broader math-function coverage.

Added expression-form array reads:

- The parser/AST/IR now support array read expressions such as `$array[$key]`,
  nested reads, and reads from literal or grouped array expressions.
- Generated C materializes the container expression and index expression
  left-to-right before calling a shared runtime lookup helper.
- The runtime lookup reuses the ordered-array key canonicalization path used by
  array literals and comparisons, including integer-string keys, boolean keys,
  float truncation through the current array-key path, and `null` as the empty
  string key with a deprecation boundary.
- Missing keys and non-array containers yield `null` after a generic warning
  boundary.
- Native tests prove successful reads, nested reads, grouped/literal reads,
  key canonicalization, missing-key diagnostics, and non-array diagnostics.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/numeric_strings/array_offset.phpt`.

Still unsupported after this array-read slice: array writes/mutation,
append/unset/iteration, `isset()`/`empty()` offset semantics, long-form
`array(...)`, string offsets, recursive arrays, references, copy-on-write,
objects/resources as containers or keys, exact warning file-name/error-handler
parity, and broader array diagnostics.

Added source-spanned parse diagnostics for unterminated block comments:

- The lexer now reports EOF inside `/* ...` as a parse-error diagnostic using
  PHP's `Unterminated comment starting line N` wording at the opening comment
  span.
- The diagnostic stays in lexing and does not reach parsing, IR lowering, or C
  generation.
- Native/CLI tests prove the diagnostic kind, source span, and `phpc`
  parse-error rendering path.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/unterminated_comment.phpt`.

Still unsupported after this diagnostic slice: broader parse-error wording
parity, stack traces, error-handler routing, PHP-exact warning/notice
formatting, and unsupported PHP token recovery.

Added global-scope `const` declarations:

- The lexer/parser/AST now recognize global `const NAME = expr;` declarations,
  including comma-separated declarations and a bounded constant-expression
  subset made from current literals, arrays, unary/cast/grouped expressions,
  binary expressions, bare constants, and magic constants.
- Nested `const` declarations are rejected before lowering so unsupported local
  declaration shapes do not silently compile.
- IR lowers declarations to `DefineConstant`, and the generated runtime stores
  declared constants in a per-runtime constant table used by bare constant
  reads and `defined()` before falling back to the modeled built-in constants.
- Native tests prove parser acceptance/rejection and the public
  `const_eval_and.phpt` source shape.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/const_eval_and.phpt`.

Still unsupported after this global const slice: namespace/class constants,
dynamic `define()`/`constant()`, duplicate constant diagnostics, full
PHP-exact constant-expression parity, additional built-in/extension constants,
and eval contexts.

Added scalar digest internals through shared runtime dispatch:

- Registered `md5()` and `sha1()` in the generated C internal-function registry,
  so normal calls and `function_exists()` share the same case-insensitive lookup
  table and argument-count checks.
- `md5()` and `sha1()` convert the input through the current boxed scalar
  string-conversion path, compute the digest in the shared runtime, and return
  lowercase hexadecimal output by default.
- The optional `raw_output` argument is accepted for both functions and returns
  raw digest bytes through the current C-string-backed value path.
- Native tests prove ASCII digest vectors, optional raw-output handling through
  `bin2hex()`, scalar input conversion, and registry exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/strings/md5.phpt`,
  `ext/standard/tests/strings/md5_basic1.phpt`,
  `ext/standard/tests/strings/md5_basic2.phpt`,
  `ext/standard/tests/strings/sha1_basic.phpt`, and
  `ext/standard/tests/strings/sha1raw.phpt`.

Still unsupported after this scalar digest slice: length-aware binary string
storage, raw digest output containing embedded NUL bytes, embedded-NUL input
parity, `md5_file()`/`sha1_file()`, and unsupported
array/object/resource/reference operand diagnostics.

Added a shared integer-only scalar operator conversion boundary:

- Bitwise, shift, and modulo helpers now route scalar operands through the same
  integer-only conversion helper instead of limiting precision-loss diagnostics
  to direct float bitwise operands.
- Float operands that lose precision while converting for integer-only
  operators emit the current generic deprecation boundary before conversion.
- Float-string operands such as `"1.5"` now emit the matching float-string
  precision-loss deprecation boundary before integer-only operator conversion.
- Native tests prove unary bitwise not, bitwise compound assignment, modulo
  compound assignment, and float/float-string precision-loss deprecations
  through generated native binaries.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/type_coercion/float_to_int/warnings_float_literals_assignment_ops.phpt`
  and
  `Zend/tests/type_coercion/float_to_int/warnings_string_float_literals_assignment_ops.phpt`.

Still unsupported after this conversion slice: PHP-exact file names, source
line propagation, error-handler routing, and overflow parity for integer-only
operator conversions; arrays, objects, resources, references, and copy-on-write
behavior remain outside this scalar boundary.

Added source-spanned parse diagnostics for invalid legacy octal integer
literals:

- The lexer now recognizes legacy octal integer tokens containing `8` or `9`
  after PHP digit-separator normalization and reports PHP's generic
  `Invalid numeric literal` parse error before AST lowering or codegen.
- Leading-zero decimal floats such as `08.5` and `08e1` remain accepted through
  the existing float literal path, matching PHP's distinction between integer
  and float literal scanning.
- Native/CLI tests prove the lexer diagnostic kind, source line/column span,
  float non-regression coverage, and `phpc` parse-error rendering.
- Focused public PHPT telemetry through `phpc` passes
  `tests/lang/invalid_octal.phpt`.

Still unsupported after this numeric-literal diagnostic slice: invalid
separator diagnostics for forms such as `100_`, `10__0`, `0x_0123`, and
`1e_2`, invalid binary/hex suffix wording, exact numeric literal overflow/range
parity, and broader parse-error wording parity.

Added scalar `substr()`:

- Registered `substr()` through the generated C internal-function registry, so
  normal calls and `function_exists()` share the same case-insensitive lookup
  table.
- `substr()` converts the input through the current boxed scalar
  string-conversion path and converts start/length through the current boxed
  scalar integer-conversion path.
- The runtime implements byte-offset slicing for the current C-string-backed
  scalar values: non-negative starts clamp at the end, negative starts count
  back from the end, negative lengths truncate from the end, omitted or `null`
  lengths read to the end, and `INT64_MIN`-sized negative offsets clamp without
  overflow.
- Native tests prove the public `substr_int_min.phpt` source shape, scalar
  conversion, positive and negative bounds, omitted/null length, and registry
  exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/strings/substr_int_min.phpt`.

Still unsupported after this `substr()` slice: exact binary-string behavior for
embedded NUL values, unsupported array/object/resource/reference operand
diagnostics, strict type handling, and broader string-runtime parity.

Added dynamic runtime constants through `define()` and `constant()`:

- Registered `define()` and `constant()` through the existing generated C
  internal-function registry, so normal calls and `function_exists()` share the
  same case-insensitive lookup table.
- Runtime constants created by `define()` use the same per-runtime constant
  table as global `const` declarations, bare constant reads, `defined()`, and
  `constant()`.
- Dynamic constant names are converted through the current boxed scalar
  string-conversion path, including numeric and empty-string names in the
  supported subset.
- Duplicate `define()` calls preserve the original value, return `false`, and
  emit a PHP-like warning boundary.
- Native tests prove runtime constant creation, dynamic lookup, registry
  visibility, numeric/empty names, and duplicate handling.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/constants/constants_001.phpt`.

Still unsupported after this dynamic constant slice: namespace/class constants,
`define()`'s legacy case-insensitive flag, exact unsupported argument type
diagnostics, additional built-in/extension constants, exception behavior for
`constant()` failures, and eval contexts.

Added source-spanned unexpected-token parse diagnostics for modeled parser
delimiter sites:

- The parser now formats unexpected tokens at statement terminators and right
  parentheses as PHP-style parse errors carrying the offending token kind,
  token text where available, and the original source span.
- The lexer now recognizes explicit octal integer literal prefixes `0o` and
  `0O` with digit separators, so modern octal tokens participate in the same
  expression parser and diagnostic path as other integer literals.
- Native/CLI tests prove explicit octal tokenization, parse-error kind and
  source line/column spans for unexpected `{` and integer tokens, and `phpc`
  parse-error rendering.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/grammar/alternative_offset_syntax_compile_error_outside_const_expr.phpt`,
  `Zend/tests/oct_whitespace.phpt`, and
  `Zend/tests/type_declarations/mixed/casting/mixed_cast_error.phpt`.

Still unsupported after this parser-diagnostic slice: exact parse-error
wording at other grammar sites, direct parser support for classes and reserved
keyword handling inside class/member declarations, `eval()` parsing, invalid
control-character diagnostics inside runtime-generated code, exact numeric
literal overflow/range parity, and invalid numeric separator diagnostics beyond
the already modeled legacy-octal boundary.

Added scalar `quotemeta()` through shared runtime dispatch:

- Registered `quotemeta()` in the generated C internal-function registry, so
  normal calls and `function_exists()` share the same case-insensitive lookup
  table and argument-count checks.
- `quotemeta()` converts its input through the current boxed scalar string path
  and prefixes PHP regex metacharacter bytes with backslashes through the
  current C-string-backed value representation.
- Single-quoted and double-quoted string lexing now preserves the backslash for
  unrecognized escape sequences, matching PHP's source string behavior for
  scalar strings such as `"\+"` and `'\t'`.
- Native tests prove the public `quotemeta_basic.phpt` and
  `quotemeta_basic_1.phpt` source shapes, scalar conversion, string-escape
  preservation, and registry exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/strings/quotemeta_basic.phpt` and
  `ext/standard/tests/strings/quotemeta_basic_1.phpt`.

Still unsupported after this `quotemeta()` slice: embedded-NUL input parity,
unsupported array/object/resource/reference operand diagnostics, and full PHP
string-escape parity for octal, hexadecimal, Unicode, vertical-tab, escape, and
form-feed spellings.

Added read-only string offset fetches:

- The generated runtime now handles `PTN_STRING` containers in the shared
  offset-read helper instead of sending every string container through the
  non-array diagnostic path.
- String offset reads support integer offsets, negative offsets,
  integer-compatible string offsets, numeric-prefix string offsets with the
  current illegal-offset warning boundary, nested reads from one-byte string
  results, and scalar cast warnings for `null`, booleans, and floats.
- Out-of-range reads emit a generic `Uninitialized string offset` warning and
  return an empty string through the existing boxed string value path.
- Native tests prove successful reads, nested reads, negative offsets,
  out-of-range diagnostics, and scalar offset casts.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/string_offset_int_min_max.phpt`.

Quarantined nearby public offset rows that should not block this slice:
`Zend/tests/str_offset_001.phpt` still requires user-defined functions before
its read-only string-offset assertions can run, and
`Zend/tests/numeric_strings/string_offset.phpt` still requires `foreach` and
try/catch exception handling around unsupported string offset key types.

Still unsupported after this string-offset read slice: string offset writes,
append, unset, `isset()`/`empty()`/null-coalescing offset semantics,
string-offset references, embedded NUL string offsets, resources/objects as
offset keys, exact `TypeError` exception behavior, PHP-exact warning file names
and error-handler routing, and broader array/object/reference offset semantics.

Added a scalar integer-conversion follow-up for exponent numeric strings:

- Registered `intval()` in the generated C internal-function registry, so
  normal calls and `function_exists()` share the same case-insensitive lookup
  and argument-count checks.
- `intval()` uses the current boxed scalar integer-conversion path, with a
  bounded string/base conversion branch for supported bases.
- Integer-only operator conversions now emit the current non-numeric warning
  boundary for leading numeric strings with trailing non-numeric data before
  converting the numeric prefix. This reuses the same modulo, bitwise, and shift
  conversion boundary that already emits float/float-string precision-loss
  deprecations.
- Native tests prove exponent-form numeric strings through explicit casts,
  `intval()`, `%`, `|`, base-aware `intval()`, and `function_exists()`
  registry lookup.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/int_conversion_exponents.phpt`.

Still unsupported after this integer-conversion slice: exact PHP file names,
source lines, error-handler routing, complete numeric-string classification
parity, exact scalar conversion overflow/range parity, and arrays, objects,
resources, references, and copy-on-write behavior.

Added plain compound blocks and script-level `return`:

- The lexer/parser/AST now recognize statement-form `return` and plain
  compound statement blocks `{ ... }` in the currently supported statement
  subset.
- Blocks lower transparently to their inner instructions, matching PHP's lack
  of block-local variable scope while preserving recursive label/goto
  validation and nested global-`const` rejection.
- Script-level `return;` and `return expr;` lower to a native early exit from
  the generated `main`. Optional return expressions are evaluated first through
  the current boxed expression path, then runtime state is freed and the native
  process exits successfully.
- Native tests prove block-contained labels/gotos, nested blocks, and
  return-expression evaluation before early exit.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/jump/jump14.phpt`.

Still unsupported after this block/return slice: PHP-exact `return` value
propagation for includes/functions, `return` inside unsupported functions,
classes, and `try`/`finally` contexts, forbidden-scope goto restrictions beyond
the currently modeled validator, and alternate control-flow syntax.

Added loop/switch entry restrictions for `goto` labels:

- Parser label validation now records the active loop/switch control path for
  each script-level label and validates every `goto` against that path.
- `goto` may jump within the same active control region or out to an enclosing
  script/block label, but source-spanned PHP-style fatals reject jumps into a
  loop or switch from outside that control region.
- Plain blocks remain transparent for label visibility and do not introduce
  jump restrictions.
- Native tests prove rejected forward/backward jumps into `while` and `switch`,
  accepted jumps out of a loop to an outer label, accepted jumps within one
  loop, and `phpc` fatal rendering.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/jump/jump07.phpt`, `Zend/tests/jump/jump08.phpt`,
  `Zend/tests/jump/jump09.phpt`, and `Zend/tests/jump/jump10.phpt`.

Still unsupported after this goto-restriction slice: labels/goto inside
unsupported functions, classes, `foreach`, and `try`/`finally` constructs,
PHP-exact invalid-goto wording for broader unsupported constructs, and
alternate control-flow syntax.

Added duplicate handling for global `const` declarations:

- IR constant-definition instructions now carry the source line of each
  declared constant name.
- Generated C uses the shared duplicate-aware runtime constant insertion helper
  for global `const` declarations, matching the existing duplicate `define()`
  boundary instead of overwriting previously defined constants.
- Duplicate global `const` declarations and `const` redeclarations after
  `define()` preserve the original runtime constant value and emit the modeled
  PHP-like warning boundary.
- Native tests prove a `define()` followed by `const` redeclaration and
  duplicate names inside one comma-separated `const` declaration.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/constants/constants_008.phpt`.

Still unsupported after this duplicate global-constant slice:
namespace/class constants, namespaced constant declaration semantics,
`define()`'s legacy case-insensitive flag, exact warning file-name/error-handler
parity, additional built-in/extension constants, exception behavior for
`constant()` failures, and eval contexts.

Refined global `const` declaration terminator diagnostics:

- Global `const` declarations now use a context-specific terminator check after
  parsing one or more declarations. Unexpected tokens at that point produce the
  generic PHP-style unexpected-token parse error with an expected-token set of
  `"," or ";"`, while ordinary statement terminators keep their existing
  behavior.
- Native/CLI tests prove parser spans and `phpc` rendering for an unexpected
  `{` after a const-expression value.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/grammar/alternative_offset_syntax_compile_error_in_const_expr.phpt`.

Still unsupported after this const-diagnostic slice: complex/braced
interpolation parse diagnostics such as
`Zend/tests/grammar/alternative_offset_syntax_in_encaps_string.phpt`, exact
parse-error expected-token sets at other grammar sites, namespace/class
constants, and broader parser coverage for unsupported declarations.

Added scalar `chunk_split()`:

- Registered `chunk_split()` in the generated C internal-function registry, so
  normal calls and `function_exists()` share the same case-insensitive lookup
  table.
- `chunk_split()` converts the input through the current boxed scalar
  string-conversion path, the optional chunk length through the current boxed
  scalar integer-conversion path, and the optional ending through the current
  boxed scalar string-conversion path.
- The runtime emits an ending after each produced chunk, using PHP's current
  default chunk length `76` and default ending `"\r\n"` for omitted arguments.
- Native tests prove the public `chunk_split_basic.phpt` source shape, scalar
  conversion, default arguments, custom endings, and registry exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/strings/chunk_split_basic.phpt`.

Still unsupported after this `chunk_split()` slice: embedded-NUL input and
ending parity, exact non-positive length `ValueError` behavior, unsupported
array/object/resource/reference operand diagnostics, and complete binary-string
runtime parity.

Added modeled versioning internals:

- Registered `php_sapi_name()` and `phpversion()` through the generated C
  internal-function registry, so normal calls and `function_exists()` share the
  same case-insensitive lookup and argument-count checks.
- `php_sapi_name()` returns the modeled CLI SAPI name for generated native
  binaries.
- `phpversion()` returns the modeled PHP version string for core, `standard`,
  and empty extension names, and returns `false` for unmodeled extension names.
- Native tests prove the public `php_sapi_name.phpt` and `phpversion.phpt`
  source shapes, extension-name lookup, unknown-extension results, and registry
  exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/versioning/php_sapi_name.phpt` and
  `ext/standard/tests/versioning/phpversion.phpt`.

Still unsupported after this versioning slice: PHP-exact SAPI identity outside
the modeled CLI runner boundary, exact PHP/extension version metadata,
extension loading state, and versioning behavior for unsupported SAPIs.

Added scalar `intdiv()` and PHP-style `var_dump()` float exponent spelling:

- Registered `intdiv()` in the generated C internal-function registry, so
  normal calls and `function_exists()` share the same case-insensitive lookup
  and argument-count checks.
- `intdiv()` converts both operands through the current boxed scalar
  integer-conversion path, including the existing float-to-int precision-loss
  diagnostic boundary, then returns the truncating integer quotient for
  supported non-zero divisors.
- `var_dump()` finite float formatting now normalizes scientific notation to
  PHP-style uppercase `E` with unpadded exponent widths while preserving the
  shortest round-trip decimal selection.
- Native tests prove signed `intdiv()` quotients, scalar string and float
  conversion, registry exposure, and exponent-form `var_dump()` output.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/zend_signed_multiply-64bit.phpt` and
  `Zend/tests/zend_signed_multiply-64bit-2.phpt`.

Still unsupported after this `intdiv()` slice: PHP-exact catchable exception
objects and messages for zero divisors and `PHP_INT_MIN / -1`, unsupported
array/object/resource/reference operands, exact warning file names and line
numbers for integer conversion diagnostics, and complete float formatting
parity outside current scalar `var_dump()`.

Added `array_key_exists()` over the current ordered-array subset:

- Registered `array_key_exists()` in the generated C internal-function
  registry, so normal calls and `function_exists()` share the same
  case-insensitive lookup and argument-count checks.
- The runtime reuses the ordered-array integer/string key canonicalization path
  already used by array literals and array reads, including integer-string
  keys and `null` canonicalizing to the empty string.
- `null` keys emit the current PHP-like `array_key_exists()` deprecation
  boundary before lookup.
- Native tests prove present/missing keys, null-key deprecation, empty-string
  and integer-string keys, null stored values, and registry exposure.
- Focused public PHPT telemetry through `phpc` passes
  `tests/basic/array_key_exists_null_deprecation.phpt`.

Still unsupported after this `array_key_exists()` slice: exact TypeError
parity for unsupported key/container types, object property checks, resources,
references, error-handler routing, and broader array/object/reference
semantics.

Added scalar `strip_tags()`:

- Registered one-argument `strip_tags()` in the generated C internal-function
  registry, so normal calls and `function_exists()` share the same
  case-insensitive lookup table.
- `strip_tags()` converts its input through the current boxed scalar
  string-conversion path and removes complete `<...>`, `<?...?>`, `<%...%>`,
  and HTML comment tag regions through the current C-string-backed value
  representation.
- Native tests prove the public `bug70720.phpt` source shape, scalar
  conversion, incomplete-tag preservation, adjacent tag stripping, and registry
  exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/strings/bug70720.phpt`.

Still unsupported after this `strip_tags()` slice: allowed-tags argument
support, embedded-NUL input parity, exact malformed/incomplete tag behavior,
unsupported array/object/resource/reference operand diagnostics, and full
binary-string runtime parity.

Added a minimal generic user-function pipeline:

- The parser now accepts top-level named function declarations with by-value
  positional parameters, ordinary `return` statements, duplicate declaration
  diagnostics, modeled internal-name redeclaration diagnostics, and minimal
  `null` parameter and return type hints.
- IR lowering carries user-function declarations separately from top-level
  statements, and the C backend emits user-function wrappers with a local
  runtime symbol table, recursive dispatch, implicit `null` returns, arity
  checks for missing required parameters, PHP-like acceptance of extra
  arguments without introspection, and shared user/internal call dispatch.
- `function_exists()` now consults generated user-function declarations as
  well as the internal-function registry.
- Native tests prove parsing, local parameter scope, early and implicit
  returns, recursion, user-function registry exposure, internal-name
  redeclaration rejection, and `null` type checks.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/type_declarations/standalone_null.phpt` and
  `tests/func/004.phpt`.

Still unsupported after this user-function slice: nested or conditional
function declarations, defaults, variadics, named arguments, by-reference
parameters or returns, closures, methods/classes, namespaces, dynamic calls,
globals/superglobals/global declarations, static locals,
`func_get_arg()`/`func_get_args()`/`func_num_args()`, non-`null` type
declarations, PHP-exact function return/include propagation, PHP-exact
function/type diagnostic wording, and scope-aware magic constants inside
functions.

Added scalar `quoted_printable_decode()`:

- Registered `quoted_printable_decode()` in the generated C internal-function
  registry, so normal calls and `function_exists()` share the same
  case-insensitive lookup table.
- The runtime converts the input through the current boxed scalar
  string-conversion path, decodes `=HH` hexadecimal byte escapes, removes
  `=\n` and `=\r\n` soft line breaks, and copies other bytes unchanged through
  the current C-string-backed value path.
- Native tests prove the public `quoted_printable_decode()` source shapes from
  `ext/standard/tests/general_functions/002.phpt` and
  `ext/standard/tests/general_functions/006.phpt`, scalar conversion, and
  registry exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/general_functions/002.phpt` and
  `ext/standard/tests/general_functions/006.phpt`.

Still unsupported after this `quoted_printable_decode()` slice: embedded-NUL
decoded output parity, unsupported array/object/resource/reference operand
diagnostics, and complete binary-string runtime parity.

Added scalar prefix/suffix string predicates:

- Registered `str_starts_with()` and `str_ends_with()` in the generated C
  internal-function registry, so normal calls and `function_exists()` share the
  same case-insensitive lookup table.
- Both functions convert haystack and needle through the current boxed scalar
  string-conversion path and return boxed booleans for prefix/suffix matches
  using the current C-string-backed value representation.
- Native tests prove the public `str_starts_with.phpt` and
  `str_ends_with.phpt` source shapes, scalar conversion, empty-needle behavior,
  and registry exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/strings/str_starts_with.phpt` and
  `ext/standard/tests/strings/str_ends_with.phpt`.

Still unsupported after this prefix/suffix string slice: PHP-exact
binary-string behavior for embedded NUL values, unsupported
array/object/resource/reference operand diagnostics, and complete binary-string
runtime parity.

Added keyword boolean operators for supported expression contexts:

- The lexer/parser now recognize `and`, `or`, and `xor` as boolean operators
  with PHP's low keyword precedence tiers below symbolic `||`/`&&`.
- AST/IR lowering carries boolean `xor` separately from bitwise `^`; generated
  C evaluates both `xor` operands left-to-right and returns boxed PHP
  truthiness inequality.
- Existing `and`/`or` expression lowering reuses the boolean short-circuit
  path while preserving the lower keyword precedence.
- Direct assignment statements parse RHS expressions at the symbolic-boolean
  precedence boundary and report a generic unsupported diagnostic if an
  unparenthesized keyword boolean tail remains, avoiding silent miscompiles
  until assignment expressions are modeled.
- Native tests prove keyword precedence, keyword `and`/`or` short-circuiting,
  keyword `xor` operand evaluation, and the assignment-tail diagnostic.

Still unsupported after this keyword-boolean slice: assignment expressions and
their PHP-exact precedence around `=`, branch-condition assignments, complete
PHP comparison parity for unsupported value types, and chained comparison
parse-error parity.

Added source-spanned fatal diagnostics for unparenthesized nested ternary
expression statements:

- The lexer now tokenizes `?` so modeled parser sites can diagnose unsupported
  ternary forms instead of failing at lexing.
- Expression-statement parsing recognizes the three PHP-forbidden nested
  associativity shapes `a ? b : c ? d : e`, `a ?: b ? c : d`, and
  `a ? b : c ?: d`, and reports the matching PHP-style compile fatal.
- The diagnostic remains parser-only; ternary expressions are not lowered or
  evaluated.
- Native/CLI tests prove parser messages and `phpc` fatal rendering.

Still unsupported after this ternary-diagnostic slice: executable ternary
expressions, expression statements beyond diagnostics, nested ternaries inside
larger supported expressions, and broader parse/fatal wording parity.

Added expression-form `isset()`/`empty()` over quiet variable and offset
lookups:

- The parser/AST/IR now represent `isset(expr[, ...])` and `empty(expr)` as
  PHP language constructs instead of normal internal-function calls.
- Long-form `array(...)` literals now lower to the same ordered-array literal
  representation as `[...]` for the currently supported array element subset.
- Generated C uses a shared quiet lookup result for variables, array offsets,
  and string offsets. Missing variables, missing array keys, non-array
  containers, and out-of-range string offsets no longer emit ordinary read
  warnings inside `isset()` and `empty()`.
- `isset()` returns false for missing or `null` values and short-circuits
  multiple arguments. `empty()` returns true for missing or PHP-falsey values.
- Native tests prove present/null/missing keys, undefined variables, nested
  array reads, nested string-offset reads, long-form arrays, and quiet
  non-numeric string offset checks.
- Focused public PHPT telemetry through `phpc` passes
  `tests/lang/empty_variation.phpt` and
  `tests/strings/offsets_chaining_3.phpt`.

Still unsupported after this `isset()`/`empty()` slice: array writes/mutation,
unset, null-coalescing offsets, object/property/reference semantics, variable
variables, exact unsupported key/container TypeError parity, exact float
offset conversion diagnostics, resources, error-handler routing, and broader
array/object/reference behavior.

Added long-form array literals plus `count()` and `abs()` internals:

- Parser support for long-form `array(...)` literals now lowers to the same
  ordered-array AST/IR/runtime path as short `[...]` literals, including keyed
  elements, automatic integer keys, insertion order, and nested literal arrays.
- Registered `count()` through the generated C internal-function registry.
  Current support returns the length of boxed arrays and exposes the function
  through the same case-insensitive lookup used by `function_exists()`.
- Registered `abs()` through the generated C internal-function registry. It
  uses the shared boxed scalar numeric-conversion path, preserves integer
  results where possible, returns floats for float inputs or integer overflow,
  and emits the modeled null-deprecation boundary.
- Native tests prove long-form arrays, `count()` in a `for` condition, array
  reads, `abs()` over the current scalar subset, and registry exposure.
- Focused public PHPT telemetry through `phpc` passes
  `ext/standard/tests/math/abs_basic.phpt`.

Still unsupported after this array/count/abs slice: array mutation,
append/unset/iteration, recursive arrays, references, copy-on-write,
`Countable` objects, exact non-array `count()` diagnostics, exact `abs()`
unsupported operand diagnostics, and complete integer overflow/formatting
parity beyond the current boxed numeric path.

Added statement-form expression evaluation:

- The parser/AST now accept supported PHP expressions as statements when they
  are not a more specific assignment, increment/decrement, label, or call
  statement form.
- Expression statements lower through the existing boxed value-expression path
  and generated native code materializes the value for side effects before
  discarding it.
- Default-only switch statements now explicitly discard the evaluated switch
  subject in generated C, preserving subject side effects without unused-temp
  compiler warnings.
- Native tests prove constant, variable, grouped, array-offset, and internal
  call expression-statement parsing; runtime evaluation/discard behavior; and
  a switch `default` body where `return;` prevents a following bare constant
  expression from executing.
- Direct variable-load IR now carries source lines, and generated native
  undefined-variable warnings include the source path and line, matching the
  selected PHPT's warning shape.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/code_before_loop_var_free.phpt`.

Reviewed nearby public switch/jump rows on the same base:
`tests/lang/bug26696.phpt`, `Zend/tests/switch/bug26281.phpt`,
`Zend/tests/switch/bug26696.phpt`, and `Zend/tests/switch/bug26801.phpt`
already pass before this patch, so they are not claimed as new coverage.

Still unsupported after this expression-statement slice: exact
undefined-constant diagnostics, expression statements for unsupported
expression forms, broad PHP-exact warning channels/error-handler routing,
`continue`, `foreach`, functions/classes, and exception/finally control-flow
edges.

Added generic `continue` loop-control semantics:

- The lexer/parser/AST/IR now support `continue;` and explicit-level
  `continue N;` using the same structured control-flow path as `break N;`.
- The C backend now tracks active control targets with separate break and
  continue labels. `while` continues recheck the condition, `do while`
  continues jump to the post-test condition, and `for` continues run update
  clauses before the next condition check.
- `continue` targeting a `switch` emits the current PHP-style warning boundary
  and exits the switch, including an outer-loop `continue N` suggestion when
  the active control stack makes one available.
- Native tests prove parser acceptance, `while`, `do while`, `for`, explicit
  loop/switch levels, switch-target warnings, and excessive-level fatal
  diagnostics.
- Public PHPT telemetry was run for
  `Zend/tests/switch/continue_targeting_switch_warning.phpt` and
  `tests/lang/024.phpt`; those rows remain blocked by unsupported function
  declarations and inline HTML between PHP blocks, respectively, so this entry
  does not claim a new public PHPT pass.

Still unsupported after this continue slice: public continue PHPT rows that
require functions or broad mixed PHP/HTML parser support, `foreach`, alternate
control-flow syntax, unbraced switch bodies, branch-condition assignments,
for-loop comma expressions and non-direct-variable clause lvalues,
PHP-exact break/continue diagnostic timing/wording beyond the current
level/context fatals and switch-target warning, functions/classes, and
exception/finally control-flow edges.

Added native `foreach` over current ordered arrays:

- The lexer/parser/AST/IR now support statement-form
  `foreach (expr as $value) statement` and
  `foreach (expr as $key => $value) statement` with direct variable bindings.
- The generated C backend evaluates the iterable expression once, creates a
  shared runtime `PtnArrayIterator` over the current boxed ordered-array
  entries, assigns optional key and value variables through the existing
  runtime symbol-table write path, and visits entries in insertion order.
- `foreach` bodies reuse the current structured loop target stack, so existing
  `break` and `continue` levels work inside foreach bodies.
- Native tests prove value-only loops, key/value loops, iterable evaluation
  once, insertion-order keys including automatic integer keys, and
  break/continue behavior.

Still unsupported after this foreach slice: by-reference `foreach`, array
mutation during iteration, copy-on-write/reference identity and exact mutation
visibility, object `Traversable`, destructuring targets, exact non-array
diagnostic parity, recursive arrays, references, and broader array/object
semantics.

Added hash-assisted lookup for larger ordered arrays:

- Generated C `PtnArray` values now keep insertion order in the existing entry
  vector while optionally allocating an open-addressed key index for arrays
  with at least 16 literal entries.
- The index preserves current PHP-shaped key behavior: integer/string key
  canonicalization, duplicate-key replacement without moving the original
  entry, automatic integer key progression, and foreach iteration over entry
  order.
- Existing array lookup users now share the same indexed path when available:
  literal duplicate replacement, array reads, quiet offset lookup for
  `isset()`/`empty()`, `array_key_exists()`, and array equality/order
  comparison lookups.
- Native tests prove a larger array with duplicate string-key replacement,
  integer-string key canonicalization, `count()`, `array_key_exists()`,
  `isset()`, `empty()`, and `foreach` insertion order.
- Native benchmark proof on a 1,024-entry ordered array with 1.2M repeated
  reads kept output `570000000` and improved from 1677/1466/1494 ms before the
  index to 95/115/107 ms after the index in this workspace.
- Focused public PHPT telemetry through `phpc` on
  `tests/basic/array_key_exists_null_deprecation.phpt`,
  `tests/basic/array_null_offset_deprecation.phpt`, and
  `tests/lang/array_shortcut_001.phpt` reported 1 pass and 2 failures. This
  slice does not claim the failing rows because they remain broader unsupported
  array-surface coverage, not hash-index behavior.

No new PHP surface is claimed by this performance slice. User-level array
element mutation, append/unset, `array_merge()`, mutation-visible iteration,
copy-on-write/reference identity, recursive arrays, and broader array/object
semantics remain unsupported.

Added a repeatable generated-native execution benchmark path:

- `tools/bench-native-execution.sh` builds the current `ptn` compiler, compiles
  three representative PHP snippets to native binaries, keeps generated C for a
  standalone rebuild timing, and samples native runtime separately from build
  timings.
- The benchmark set covers scalar arithmetic/control-flow loops, string
  concatenation with registered internal functions, and ordered array
  key/value `foreach` iteration using currently supported PHP semantics.
- The benchmark report records commit, host/resource notes, command lines,
  Rust compiler build time, integrated `ptn compile` time, standalone generated
  C rebuild time, native runtime samples, and deterministic stdout checks for
  reviewable future speed deltas.

Optimized generated string concatenation runtime behavior:

- The generated C `ptn_concat` helper now uses a borrowed/owned string operand
  view so already-string operands and static scalar string conversions do not
  allocate duplicate temporary buffers before the joined result allocation.
- Non-string numeric conversions still allocate conversion buffers through the
  existing formatting path, and concat frees only those owned conversion
  buffers after copying into the final result.
- Codegen remains unchanged, so binary operands continue to materialize
  left-to-right before the shared boxed concat helper runs.
- Native tests prove chained `$x = $x . ...` concatenation and looped `.=` use
  the same compiled binary path after the runtime optimization.
- A native benchmark with 6000 chained and compound concat loop iterations
  produced the same `46890 34890` output and improved from about `real
  1.35-1.37s` before the change to `real 0.84-0.87s` after the change.

Optimized generated C string internal operands:

- Added a shared borrowed-or-owned string operand helper for generated C runtime
  internals. Direct `PTN_STRING` values now pass their existing C string pointer
  through string-processing internals without first allocating a duplicate,
  while non-string operands still use the existing scalar string-conversion
  path and release the owned conversion afterward.
- Switched current string-consuming internals including `strlen()`,
  `str_rot13()`, `strcmp()`, `str_contains()`, `str_starts_with()`,
  `str_ends_with()`, `quotemeta()`, `chunk_split()`, `strip_tags()`, `md5()`,
  `sha1()`, `substr()`, `dirname()`, `bin2hex()`, `hex2bin()`,
  `quoted_printable_decode()`, `soundex()`, `phpversion()`, `bindec()`,
  `hexdec()`, `octdec()`, and `ord()` to use that fast path.
- Native tests prove direct-string behavior, scalar conversion fallback
  behavior, and emitted C shape showing the direct string operand helper rather
  than unconditional `ptn_value_to_string(args...)` conversion in those
  internals.

Still unsupported after this optimization: embedded-NUL string parity, exact
unsupported array/object/resource/reference diagnostics for these internals,
and broader binary-string length tracking beyond the current C-string-backed
value model.
