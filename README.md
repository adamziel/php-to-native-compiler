# PHP-to-Native Compiler

This project is an experimental PHP-to-native compiler implemented in stable Rust.
It is intentionally small and honest: implemented features are tested, unsupported
features are documented, and native code generation starts with LLVM IR text.

## Build

```sh
cargo build
```

## CLI

```sh
cargo run -p phpc -- run examples/hello.php
cargo run -p phpc -- compile examples/hello.php --emit-ir
cargo run -p phpc -- compile examples/hello.php --emit-asm
cargo run -p phpc -- test
```

The installed binary name is `phpc`.

`--emit-asm` prefers `clang` or `llc` for LLVM IR assembly emission. If neither
tool exists, it currently falls back to generating equivalent narrow-subset C and
running `cc -S`; this is a real assembly path, but it is documented as a
temporary bootstrap fallback rather than the long-term backend.
The test suite exercises `phpc compile --emit-asm` for the current lowerable
straight-line scalar echo/assignment subset with a normalized CLI summary rather
than snapshotting platform-specific assembly text.
It also snapshots a representative unsupported array program with backend tools
removed from `PATH`, proving `--emit-asm` reports the LLVM lowering rejection
before attempting assembly backend discovery.
A separate lowerable scalar snapshot removes backend tools from `PATH` and
records the stable missing-backend diagnostic for environments without
`clang`, `llc`, or `cc`.
Another lowerable scalar snapshot hides `clang` and `llc` while exposing only a
`cc` command, proving the documented `cc -S` fallback with normalized assembly
output checks instead of backend-specific assembly text.
A backend-failure snapshot exposes a deterministic fake `clang` that passes
discovery and then exits nonzero, pinning the stable `--emit-asm` diagnostic
shape without depending on toolchain-specific stderr.
A deterministic fake-`llc` snapshot hides `clang` and `cc` while exposing only
`llc`, proving the documented LLVM backend selection order without committing
backend-specific assembly text.
A failing fake-`llc` snapshot records the stable selected-`llc` failure
diagnostic when `clang` is unavailable.
A failing fake-`cc` snapshot records the stable C fallback failure diagnostic
when both LLVM assembly backends are unavailable and the fallback compiler
exits nonzero.
A discovery-edge snapshot exposes a fake `clang` command whose `--version`
probe fails while a fake `llc` probe succeeds, proving failed discovery probes
are skipped before fallback selection without committing backend-specific
assembly text.
A discovery-exhaustion snapshot exposes fake `clang`, `llc`, and `cc` commands
whose `--version` probes all fail, proving failed probes are treated the same
as unavailable tools and the stable missing-backend diagnostic is reported.
An empty-stderr backend-failure snapshot exposes a deterministic fake `clang`
that passes discovery and exits nonzero without diagnostic text, proving the
stable fallback detail for selected backend failures that produce no stderr.
An empty-stdout backend-success snapshot exposes a deterministic fake `clang`
that passes discovery and exits successfully without assembly text, proving the
stable rejection for selected backends that produce no stdout.
A success-with-stderr backend snapshot exposes a deterministic fake `clang`
that passes discovery, emits assembly stdout, writes a diagnostic to stderr,
and exits successfully, proving `phpc` returns the assembly and does not surface
backend stderr on successful assembly emission.
Additional success-with-stderr fallback snapshots expose deterministic fake
`llc` and `cc` tools with the same behavior, proving that boundary after LLVM
backend fallback selection and after the `cc -S` C fallback selection.
Additional empty-stderr fallback failure snapshots expose deterministic fake
`llc` and `cc` tools that exit nonzero without diagnostics, proving the stable
empty-stderr failure detail after LLVM backend fallback selection and after the
`cc -S` C fallback selection.
Additional empty-stdout fallback success snapshots expose deterministic fake
`llc` and `cc` tools that exit successfully without assembly text, proving the
stable empty-output diagnostic after LLVM backend fallback selection and after
the `cc -S` C fallback selection.
Additional whitespace-only fallback success snapshots expose deterministic fake
`llc` and `cc` tools that exit successfully with only whitespace on stdout,
proving the stable whitespace-only-output diagnostic after LLVM backend
fallback selection and after the `cc -S` C fallback selection.
A selected-backend whitespace-only success snapshot exposes a deterministic
fake `clang` that exits successfully with only whitespace on stdout, proving
the same stable diagnostic before fallback selection.
A selected-backend whitespace-with-stderr success snapshot exposes a
deterministic fake `clang` that exits successfully with only whitespace on
stdout and diagnostics on stderr, proving stdout validation wins and backend
stderr remains unsurfaced on invalid successful output.
A selected-backend whitespace-with-stderr precedence snapshot exposes the same
fake `clang` while `llc` and `cc` are available, proving that invalid selected
backend output is reported without fallback recovery.
An `llc` whitespace-with-stderr precedence snapshot exposes fake `llc` and
`cc` tools while `clang` is unavailable, proving invalid selected `llc` output
is reported without falling through to the `cc -S` fallback.
An `llc` empty-stdout precedence snapshot exposes fake `llc` and `cc` tools
while `clang` is unavailable, proving invalid selected `llc` empty output is
reported without falling through to the `cc -S` fallback.
Additional whitespace-with-stderr fallback snapshots expose deterministic fake
`llc` and `cc` tools with the same invalid successful-output behavior, proving
the same stdout-validation precedence after LLVM backend fallback selection and
after the `cc -S` C fallback selection.
A selected-backend input-validation snapshot exposes a deterministic fake
`clang` that validates representative generated LLVM IR markers arrive on
stdin before emitting normalized assembly, proving `phpc` feeds the selected
backend through stdin for the current lowerable scalar subset.
Fallback input-validation snapshots expose deterministic fake `llc` and `cc`
tools that validate representative generated LLVM IR or generated C fallback
markers arrive on stdin before emitting normalized assembly, proving the same
stdin handoff after LLVM backend fallback selection and after `cc -S` C
fallback selection.
Backend argument-validation snapshots expose deterministic fake `clang`,
`llc`, and `cc` tools that validate the expected assembly emission argument
vectors before accepting stdin and emitting normalized assembly.
Backend discovery probe argument-validation snapshots expose deterministic fake
`clang`, `llc`, and `cc` tools that require an exact single-argument
`--version` probe before selected or fallback assembly emission proceeds.
Backend discovery probe output snapshots expose deterministic fake `clang`,
`llc`, and `cc` tools whose successful `--version` probes write stdout and
stderr diagnostics, proving discovery output is ignored when the later
selected or fallback assembly emission succeeds.
Failed backend discovery probe output snapshots expose deterministic fake
`clang`, `llc`, and `cc` tools whose failed `--version` probes write stdout
and stderr diagnostics before fallback selection or missing-backend reporting,
proving failed-probe output is ignored and the stable backend selection result
is preserved.
Backend discovery probe start-failure snapshots expose deterministic fake
`clang`, `llc`, and `cc` command names that exist on `PATH` but cannot be
started for `--version`, proving those failed starts are treated as unavailable
before fallback selection or missing-backend reporting.
Backend discovery probe permission-denied snapshots expose deterministic fake
`clang`, `llc`, and `cc` command names that exist on `PATH` but are not
executable for `--version`, proving those failed probes are treated as
unavailable before fallback selection or missing-backend reporting.
A selected-backend start-failure snapshot exposes a deterministic fake
`clang` that passes discovery and then rewrites itself to use a missing
interpreter before assembly emission, proving `phpc` reports the stable
selected-backend start diagnostic when a previously discovered command cannot
be started.
A selected-backend permission-denied emission snapshot exposes a
deterministic fake `clang` that passes discovery and then removes its own
execute permission before assembly emission, proving the same stable
selected-backend start diagnostic is reported for permission-denied starts.
Fallback start-failure snapshots expose deterministic fake `llc` and `cc`
tools with the same race-like behavior, proving the stable fallback backend
start diagnostics after LLVM fallback selection and after the `cc -S` C
fallback selection.
Fallback permission-denied emission snapshots expose deterministic fake `llc`
and `cc` tools that pass discovery and then remove their own execute
permission before assembly emission, proving the same stable fallback backend
start diagnostics for permission-denied starts after discovery and that `llc`
permission-denied starts do not fall through to the `cc -S` C fallback.
A mixed scalar output snapshot uses a lowerable straight-line program with
both `echo` and `print`, proving the current native assembly CLI path accepts
that static scalar output subset without claiming runtime-backed conversion.
A matching C fallback mixed-output snapshot hides LLVM assembly tools and uses
a deterministic fake `cc` that validates generated C fallback markers for the
same static scalar `echo`/`print` boundary.
A scalar reassignment snapshot uses a deterministic fake `clang` that validates
generated LLVM IR for a straight-line program where later direct static-variable
assignments overwrite earlier scalar values before output. This pins the
current overwrite boundary without claiming native symbol-table storage,
references/copy-on-write behavior, linking/execution, exact native PHP errors,
or broader native lowering.
A matching C fallback reassignment snapshot hides LLVM assembly tools and uses
a deterministic fake `cc` that validates generated C fallback source for the
same straight-line overwrite boundary.
A matching `--emit-ir` reassignment snapshot commits the exact generated LLVM
IR and shows only the final overwritten scalar values are emitted.
A selected-`clang` type-introspection snapshot validates that the already
implemented native `is_callable(...)` false-folding IR is passed to the chosen
backend through stdin without broadening callable dispatch or native execution.
A selected-`clang` `function_exists($name)` snapshot validates the same stdin
handoff for already-implemented direct function-existence folding without
broadening runtime lookup, callable dispatch, or native execution.
A selected-`clang` `empty($name)` snapshot validates the same stdin handoff for
already-implemented direct-variable emptiness folding without broadening
symbol-table semantics, unset interactions, or native execution.
A selected-`clang` `isset($name)` snapshot validates the same stdin handoff for
already-implemented direct-variable existence folding without broadening
symbol-table semantics, unset interactions, or native execution.
A selected-`clang` `is_numeric($value)` snapshot validates the same stdin
handoff for already-implemented deterministic scalar/string numeric folding
without broadening runtime lookup, string coercion, or native execution.
A selected-`clang` `is_countable($value)`/`is_iterable($value)` snapshot
validates the same stdin handoff for already-implemented scalar/null
false-folding without broadening native array/object lowering or runtime-backed
type checks.
A selected-`clang` `is_object($value)`/`get_debug_type($value)` snapshot
validates the same stdin handoff for already-implemented scalar/null folding
without broadening native object lowering, object handles, or runtime-backed
type checks.
A selected-`clang` static metadata-exists snapshot validates the same stdin
handoff for already-implemented absent-class/interface/trait/enum false-folding
without adding native class metadata, autoloading, or object execution.
A selected-`clang` `strlen($value)` snapshot validates the same stdin handoff
for already-implemented known-string length folding without broadening string
coercion, dynamic calls, runtime lookup, or native execution.
A native `defined($name)` snapshot commits the folded `--emit-ir` and
normalized C fallback `--emit-asm` output for direct supported string names,
without broadening native constant values, source-order definitions, runtime
constant tables, or native execution.
The Milestone 569 native `defined($name)` snapshot extends that static table to
the current `SORT_REGULAR` built-in constant while still rejecting actual
constant-value lowering, runtime-backed lookup, dynamic calls, and exact native
PHP error behavior.
The Milestone 573 native `defined($name)` snapshot extends the same static
answer table to `SORT_NUMERIC` after runtime support landed, with folded
`--emit-ir` and C fallback `--emit-asm` coverage while preserving the broader
constant-table rejection boundary.
A selected-`clang` `defined("SORT_REGULAR")` snapshot validates that the same
folded LLVM IR reaches the chosen backend through stdin without changing
production lowering behavior.
A selected-`clang` `defined("SORT_NUMERIC")` snapshot validates the same
backend stdin handoff for the existing folded built-in constant answer without
changing production lowering behavior.
A selected-`clang` `defined("SORT_STRING")` snapshot validates the same
backend stdin handoff for the existing folded built-in constant answer without
changing production lowering behavior.
A selected-`clang` broader `defined($name)` constants snapshot validates the
same backend stdin handoff for the current exact built-in constant answer table
without changing production lowering behavior.
A backend-precedence snapshot exposes deterministic fake `clang`, `llc`, and
`cc` commands together, proving successful `clang` assembly emission is
selected before fallback tools when all candidates are available.
A fallback-precedence snapshot hides `clang` while exposing deterministic fake
`llc` and `cc` commands together, proving successful `llc` assembly emission is
selected before the `cc -S` C fallback when both fallback candidates are
available.
A selected-backend failure-precedence snapshot exposes deterministic fake
`clang`, `llc`, and `cc` commands together, makes selected `clang` fail
emission, and proves the selected-backend diagnostic is reported without
silently falling through to fallback tools.
A fallback failure-precedence snapshot hides `clang` while exposing
deterministic fake `llc` and `cc` commands together, makes selected `llc`
fail emission, and proves the `llc` diagnostic is reported without silently
falling through to the `cc -S` C fallback.
An empty-stderr fallback failure-precedence snapshot covers the same
`clang`-unavailable boundary when selected `llc` exits nonzero without
diagnostics, proving the stable empty-stderr `llc` diagnostic also blocks
`cc -S` fallback recovery.
An empty-stderr selected-backend failure-precedence snapshot exposes
deterministic fake `clang`, `llc`, and `cc` commands together, proves the
stable empty-stderr `clang` diagnostic is reported as final, and proves
fallback tools are not invoked after selected `clang` fails.
A selected-backend start-failure-precedence snapshot exposes deterministic
fake `clang`, `llc`, and `cc` commands together, rewrites selected `clang`
after discovery so it cannot be started for assembly emission, and proves the
stable selected-backend start diagnostic is reported without falling through
to fallback tools.
A fallback start-failure-precedence snapshot hides `clang` while exposing
deterministic fake `llc` and `cc` commands together, rewrites selected `llc`
after discovery so it cannot be started for assembly emission, and proves the
stable `llc` start diagnostic is reported without falling through to the
`cc -S` C fallback.

## Current Status

Milestone 1 is in progress. The interpreter/runtime path supports a small PHP
subset:

- `echo`
- integer, float, and string literals
- static variables, assignment, direct static-variable assignment expressions
  (`$name = expr`) and direct array-offset assignment expressions
  (`$array[$key] = expr`) and direct public object-property assignment
  expressions (`$object->property = expr`) with right-to-left chained
  assignment result values, including compound-assignment and null-coalescing
  assignment expressions as chained right-hand values, direct append-offset
  assignment expressions (`$array[] = expr`) with standalone assignment result
  values,
  direct static-variable compound assignment (`+=`, `-=`, `*=`, `/=`, `%=`,
  `.=`, `&=`, `|=`, `^=`, `<<=`, `>>=`)
  in statements and
  expressions with assignment result values, direct array-offset compound
  assignment (`$array[$key] += expr` and related operators) in statements,
  expressions, and C-style `for` initializer/increment slots, direct
  public object-property compound assignment (`$object->property += expr` and
  related operators) in statements, expressions, and C-style `for`
  initializer/increment slots, direct array-offset pre/post increment and
  decrement for integer and float offset values in statements, expressions,
  and C-style `for` initializer/increment slots, direct public
  object-property pre/post
  increment and decrement for integer and float property values in statements,
  expressions, and C-style `for` initializer/increment slots, direct
  static-variable pre/post increment and decrement for integer and float
  variables in statements, expressions, and C-style `for`
  initializer/increment slots, and
  `unset($name)` through per-scope symbol tables. Assignment-expression values
  have executable coverage in function call arguments, array literal
  keys/values, `if`/loop conditions, and builtin arguments for the documented
  direct-target subset
- `+`, `-`, `*`, `/`, `%`, `.`
- loose scalar comparisons and scalar strict identity comparisons used by
  control flow
- logical `&&`, `||`, `and`, `xor`, and `or` over the current truthiness
  rules, with boolean result values, short-circuit evaluation for `&&`, `||`,
  `and`, and `or`, PHP-style `&&`/`||` precedence, and lower-than-assignment
  word-operator precedence in the current expression and statement parser
  subset
- bitwise `&`, `|`, `^`, unary `~`, and shift operators `<<`/`>>` over the
  current integer/string subset, including PHP-style precedence, integer unary
  bitwise-not results for integer operands, integer binary results for
  non-string-string operands after current scalar-to-int coercion, bytewise
  string results when the resulting runtime string remains valid UTF-8, and
  PHP-shaped large-count shift behavior for the current integer result model;
  direct static-variable, direct array-offset, and direct public
  object-property compound assignments also support `&=`, `|=`, `^=`, `<<=`,
  and `>>=` through the interpreter path
- full ternary conditional expressions `$condition ? $if_true : $if_false`
  and short ternary expressions `$value ?: $fallback` over the current
  expression/value subset, with truthiness-based condition selection,
  condition-value reuse for short ternary, lazy branch/fallback evaluation,
  and executable coverage for mixes with `??` and assignment-expression
  branches over the documented direct-target subset
- `if` / `elseif` / `else`
- `while`, C-style `for` loops over the documented header subset,
  `do ... while` post-condition loops, `switch`/`case`/`default` statements
  over the current scalar comparison subset in brace or alternate
  `switch (...): ... endswitch;` form with `:` or `;` case/default separators,
  and array `foreach` in value-only and key/value forms, including `break;`
  for innermost loops or switch statements and `continue;` for the innermost
  executing loop
- function declarations with optional trailing commas in parameter lists,
  positional calls with optional trailing commas in argument lists, and
  trailing default parameter values over the documented constant-expression
  and unqualified constant reference subset, recursive calls up to the
  documented guard, `return`, and isolated local scopes for user-function calls
- dynamic function calls through string-valued expressions that resolve to the
  documented callable builtin subset or user-defined functions, with optional
  trailing commas in argument lists
- top-level class declarations registered as metadata, with property and method
  metadata for the documented subset
- minimal object instantiation with `new ClassName()` for declared classes that
  do not define constructors; public instance properties can be read and
  written by static property name and checked with `isset($object->name)`
- magic constants `__LINE__`, evaluated from the expression token's source
  line, `__FILE__`, evaluated from the current `phpc run` input path when one
  is available, `__DIR__`, evaluated as that path's parent directory, and
  `__FUNCTION__`, evaluated as the current user-function name or an empty
  string outside a function
- narrow global constant resolution: exact uppercase `CASE_LOWER`,
  `CASE_UPPER`, `ARRAY_FILTER_USE_KEY`, `ARRAY_FILTER_USE_BOTH`, and
  `SORT_REGULAR`/`SORT_NUMERIC`/`SORT_STRING` work as bare built-in
  constants, and `define($name, $value)`, `constant($name)`,
  `defined($name)`, and bare user-constant reads support unqualified names
  over the documented
  scalar/array value subset; top-level single and grouped `const NAME = value;`
  declarations are executable over the current constant-expression and
  scalar/array value subset, including references to previously defined
  unqualified constants and the current built-in constant slice
- short array literals and long `array(...)` literals with integer/string keys
- array indexed reads, indexed writes, and append writes for the documented
  direct-variable array subset
- `unset(...)` with one or more operands over direct static variables and
  direct array-variable offsets in the documented subset
- `foreach ($array as $value)` and `foreach ($array as $key => $value)` over
  the documented ordered array value model
- direct `isset($array[$key])` checks for the documented array-offset subset
- direct `empty($name)`, `empty($array[$key])`, and
  `empty($object->publicProperty)` checks for the documented
  variable/array-offset/public-property subset
- null coalescing `??` for direct static variables, direct array offsets, and
  direct public object properties over the current value model
- null coalescing assignment `$name ??= expr`, `$array[$key] ??= expr`, and
  `$object->publicProperty ??= expr` for direct static variables, direct array
  offsets, and direct public object properties over the current value model,
  including parenthesized expression forms that return the assigned or
  existing value
- builtins for the documented scalar/array/object subset: `strlen`, `isset`,
  `empty`, `count`, `array_key_exists` including null, boolean, and integral
  finite float key coercions, `array_key_first`, `array_key_last`,
  `array_is_list`, `array_values`, `array_keys`, including loose and strict
  scalar search-value filtering, `array_reverse` with default reindexing and
  boolean preserve-key behavior, `array_slice($array, $offset)`,
  `array_slice($array, $offset, $length)`, and null-length
  `array_slice($array, $offset, null)` with default integer-key reindexing
  and string-key preservation, plus boolean preserve-key mode for
  `array_slice`, `array_chunk($array, $length)` with default chunk-key
  reindexing and boolean preserve-key mode, `array_pad($array, $length,
  $value)` with positive right-padding, negative left-padding, and integer-key
  reindexing when padding is needed, `array_merge` over zero or more array
  operands, `array_replace` over one or more arrays with key-preserving
  left-to-right overwrite behavior, `array_combine` over equal-length
  key/value arrays with
  null/boolean/integer/string/integral-finite-float key values, variadic
  `array_intersect_key` and
  `array_diff_key` over two or more arrays with first-array key/value
  preservation, variadic `array_diff` and `array_intersect` over two or more
  arrays with scalar value comparisons and first-array key/value preservation,
  `array_unique($array[, SORT_REGULAR|SORT_NUMERIC|SORT_STRING])` with scalar
  deduplication and first-occurrence preservation, including numeric-mode
  comparison over the current scalar numeric-coercion subset,
  `array_flip` over integer/string array values,
  `array_change_key_case($array[, $case])` over ASCII string keys while
  preserving integer keys, with integer case flag `0`/`CASE_LOWER`
  lowercasing and any nonzero integer, including `CASE_UPPER`, uppercasing,
  `array_column($rows, $column_key[, $index_key])` over array rows and public
  object rows, reindexing extracted values from zero or indexing results by
  null/boolean/integer/string/integral-finite-float row values while skipping
  missing columns,
  `array_fill_keys` over null/boolean/integer/string/integral-finite-float key
  values, `array_count_values` over integer/string array values,
  `array_sum` over the current scalar
  numeric-coercion subset, `array_product` over the current scalar
  numeric-coercion subset,
  `array_reduce($array, $callback[, $initial])` over the current string-valued
  callback subset, `array_filter` without a callback, with a `null` callback,
  and with string-valued value-only callbacks, including explicit integer mode
  flag `0`, integer-string mode flag `"0"`, and boolean mode flag `false` for
  those value-only paths, plus string-valued key-only callbacks through integer
  or integer-string mode flag `2` and string-valued value/key callbacks
  through integer or integer-string mode flag `1` or boolean mode flag `true`,
  `define`, `constant`, and `defined` over the documented
  runtime-defined/built-in constant name slice, `array_map` over the current
  one-array null-callback identity, variadic null-callback zip, and variadic
  string-callback subset with one-array key preservation and multi-array
  reindexing,
  `in_array` and `array_search` including strict scalar searches,
  `gettype($value)`, `is_null($value)`, `is_bool($value)`, `is_int($value)`,
  `is_integer($value)`, `is_long($value)`, `is_float($value)`,
  `is_double($value)`, `is_string($value)`, `is_array($value)`,
  `is_scalar($value)`, `is_numeric($value)`, `is_countable($value)`,
  `is_iterable($value)`, `is_callable($value)`, `function_exists($name)`,
  `get_class($object)`, `is_object($value)`,
  `get_debug_type($value)`,
  `class_exists($name[, $autoload])`, `interface_exists($name[, $autoload])`,
  `trait_exists($name[, $autoload])`, `enum_exists($name[, $autoload])`, and
  `property_exists($object_or_class, $property)` and
  `method_exists($object_or_class, $method)` and
  `get_class_methods($object_or_class)` and
  `get_class_vars($class_name)` and
  `get_object_vars($object)` and
  `get_mangled_object_vars($object)` and
  `is_a($object_or_class, $class_name[, $allow_string])` and
  `is_subclass_of($object_or_class, $class_name[, $allow_string])` and
  `get_parent_class($object_or_class)` and `get_declared_classes()` and
  `get_declared_interfaces()` and `get_declared_traits()` over the current
  minimal object value model and declared-class metadata,
  `spl_object_id($object)` as an explicit unsupported object-handle identity
  boundary, `spl_object_hash($object)` as an explicit unsupported
  object-handle hash boundary,
  `var_dump`, and `print_r`
- stable runtime diagnostics for the currently covered runtime errors,
  including unresolved or non-string dynamic function calls, unsupported
  `get_called_class()` calls before method/static class context exists,
  unsupported `spl_object_id($object)` calls before PHP object handle identity
  exists, unsupported `spl_object_hash($object)` calls before PHP object
  handle hash support exists,
  `global` declarations, duplicate or unsupported `define(...)` constant
  definitions, unsupported `defined(...)` name arguments, unknown bare global
  constants outside the current built-in/runtime-defined slice such as
  `PHP_VERSION`,
  invalid `break`/`continue` outside a loop, and runaway recursion
- stable lex/parse diagnostics for unsupported dynamic/function features
  including variable variables, include/require/eval constructs,
  namespace and `use` declarations, namespace-qualified function/class names,
  variadics, references, parameter and return type declarations, static local
  variable declarations, magic constants other than executable `__LINE__`,
  `__FILE__`, `__DIR__`, and `__FUNCTION__`, with `__METHOD__`,
  `__CLASS__`, `__TRAIT__`, and `__NAMESPACE__` held on
  context-specific parse diagnostics,
  closures, named arguments, empty call arguments, empty parameter slots,
  `declare(strict_types=1)`,
  unsupported nested, namespace-aware, or dynamic-value `const` declarations,
  unsupported first-class callable syntax such as `strlen(...)` and
  `$callback(...)`,
  unsupported magic class names in `new` expressions such as `new self()`,
  `new parent()`, and `new static()`,
  unsupported array spread/reference elements, unsupported `list(...)` and
  `[...]` destructuring assignment targets, unsupported broader
  `unset(...)` forms such as property, append-offset, and nested unset,
  unsupported exponentiation syntax `**` and `**=`,
  unsupported `foreach` by-reference/destructuring forms, unsupported
  comma-separated `for` header expression lists, expression-form
  `do ... while`, alternate `if`/`elseif`/`else` colon/`endif` syntax,
  expression-form `switch`, malformed alternate switch bodies,
  `break`/`continue` depth arguments, unsupported exception syntax (`throw`,
  `try`, `catch`, and
  `finally`), unsupported generator `yield` and `yield from` expressions,
  unsupported PHP 8 `match` expressions, unsupported `goto` statements and
  labels, unsupported heredoc/nowdoc multiline string syntax, unsupported
  unparenthesized nested ternary expressions, unsupported
  append-offset chained assignment
  expressions, unsupported complex or nested assignment-expression targets,
  unsupported append-offset `??=`,
  unsupported compound assignment targets outside direct static variables,
  direct array offsets, and direct public object properties,
  unsupported increment/decrement targets outside direct static variables,
  direct array offsets, and direct public object properties, and chained
  increment/decrement expressions,
  unsupported chained coalescing and unsupported append-offset null
  coalescing assignment forms, object
  method calls, dynamic property names, anonymous classes,
  unsupported class forms, `abstract`/`final`/`readonly` class modifiers,
  unsupported `abstract`/`final`/`readonly` class member modifiers,
  unsupported typed property declarations, unsupported property defaults,
  unsupported multiple property declarations, unsupported class constant
  declarations, unsupported `$this` usage, unsupported `clone` expressions,
  unsupported `instanceof` expressions, unsupported `ClassName::class`
  expressions, unsupported magic static receivers such as `self::`,
  `parent::`, and `static::`, trait declarations, trait use inside classes,
  interface declarations, interface implementation clauses, enum declarations,
  static member access, and class constants

`php_runtime` also contains a tested object/class metadata registry and minimal
object values. `phpc run` can instantiate declared constructor-free classes,
read/write public instance properties by static name, and check those public
properties with `isset` and `empty`. `empty($object->name)` returns true for
null and falsey public slots, missing properties, undefined target variables,
and non-object target variables in the current direct-object-variable subset.
`gettype($value)` returns PHP legacy type names for the current boxed value
model (`NULL`, `boolean`, `integer`, `double`, `string`, `array`, and
`object`). `is_null`, `is_bool`, `is_int`/`is_integer`/`is_long`,
`is_float`/`is_double`, `is_string`, `is_array`, and `is_scalar` report the
current value category without coercion. `is_numeric` returns true for
integers, floats, and well-formed numeric strings using the same current
numeric-string subset as scalar arithmetic. `is_countable` and `is_iterable`
return true for arrays and false for the current scalar/null/object values.
`is_callable($value)` supports the current string function-name subset: it
returns true for names that resolve to current user functions or documented
callable builtins, and false for missing names or non-string values.
`is_callable($value, $syntax_only)` accepts boolean syntax-only flags; for
string values, `true` reports callable syntax without resolving the name, while
`false` uses the current function lookup path. Scalar non-string values return
false. Syntax-only array callable checks accept only the current two-element
`[class-or-object, method]` shape with integer keys `0` and `1`, where the
first value is a string class name or current object and the second value is a
string method name; this shape check does not resolve classes or methods.
Normal array callable resolution checks the same two-element shape against the
current declared method metadata: object receivers are true for public declared
methods, and class-string receivers are true for public static declared
methods. Array callable invocation, callable-name output, object `__invoke`
callables, private/protected caller-context method callability, first-class
callable syntax, namespace/autoload resolution, and exact native callable
diagnostics are not implemented. Native lowering folds only direct
calls whose value argument is an already-lowerable string or non-string
scalar/null value and whose optional syntax-only flag is an already-lowerable
boolean; true syntax-only flags return true for string values, non-string
scalar/null values return false, while false or omitted flags use the
documented native builtin lookup table for strings.
`function_exists($name)` checks string names against the current runtime
function table, including user functions and documented callable builtins, and
is available through string-valued dynamic calls. Native lowering folds only
direct calls whose name argument is an already-lowerable string with a uniform
known result in the documented builtin table. Non-string name coercion,
namespace/autoload-aware lookup, extension-loaded functions beyond documented
builtins, user-defined native function tables, dynamic native calls, and exact
native `TypeError`/deprecation behavior are not implemented.
Broader type checks such as legacy aliases not available in the target PHP
version, resource-aware checks, `Traversable` object semantics, and
`Countable` object/interface semantics are not implemented.
`get_class($object)` returns the declared class name for those minimal object
values, `is_object($value)` returns true only for those current object values,
and `get_debug_type($value)` returns scalar/array type names or the declared
class name for current object values.
`class_exists($name[, $autoload])` checks the current declared-class metadata
case-insensitively for string class names; the autoload flag is accepted only
as a boolean and does not trigger autoloading.
`interface_exists($name[, $autoload])` accepts the same string-name and boolean
autoload boundary, returns false for all supported calls because interface
metadata is not represented yet, and does not trigger autoloading.
`trait_exists($name[, $autoload])` accepts the same string-name and boolean
autoload boundary, returns false for all supported calls because trait metadata
is not represented yet, and does not trigger autoloading.
`enum_exists($name[, $autoload])` accepts the same string-name and boolean
autoload boundary, returns false for all supported calls because enum metadata
is not represented yet, and does not trigger autoloading.
`property_exists($object_or_class, $property)` accepts current object values or
string class names, uses case-sensitive declared property names, reports
declared public/protected/private and static properties, and returns false for
missing classes.
`method_exists($object_or_class, $method)` accepts current object values or
string class names, uses case-insensitive declared method names, reports
declared public/protected/private and static methods, and returns false for
missing classes. `get_class_methods($object_or_class)` accepts current object
values or declared string class names and returns public declared methods in
declaration order. `get_class_vars($class_name)` accepts declared string class
names and returns public declared property names in declaration order with
`null` values because property defaults are not implemented.
`get_object_vars($object)` accepts current object values and returns public
instance property names with their current slot values in declaration order.
`get_mangled_object_vars($object)` accepts current object values and returns
public, protected, and private instance slots in declaration order using
PHP-style property keys: public names as-is, protected names as
`\0*\0name`, and private names as `\0ClassName\0name`.
`is_a($object_or_class, $class_name[, $allow_string])` checks exact class
identity only: object inputs are accepted, string object/class inputs are
considered only when `allow_string` is true, and target class names use the
current case-insensitive class metadata lookup.
`is_subclass_of($object_or_class, $class_name[, $allow_string])` validates the
same object/string and class-name argument boundary, but because inheritance is
not represented yet it returns false for exact-class and no-parent metadata
cases. `get_parent_class($object_or_class)` accepts current object values or
declared string class names and returns false for all supported inputs until
parent class metadata exists. `get_declared_classes()` returns a zero-indexed
array of classes declared in the current parsed program in declaration order,
and is also available through string-valued dynamic function calls.
`get_declared_interfaces()` returns an empty zero-indexed array because
interface declarations and internal interface metadata are not represented yet;
it is also available through string-valued dynamic function calls.
`get_declared_traits()` returns an empty zero-indexed array because trait
declarations and internal trait metadata are not represented yet; it is also
available through string-valued dynamic function calls.
`get_called_class()` is recognized as a zero-argument callable boundary, but
direct and string-valued dynamic calls currently fail with a stable unsupported
runtime diagnostic until method/static class context exists.
`spl_object_id($object)` is recognized as a one-argument callable boundary, but
object arguments currently fail with a stable unsupported runtime diagnostic
until PHP object handle identity, reuse, clone behavior, and destruction are
modeled; non-object arguments fail with a stable type-boundary diagnostic.
`spl_object_hash($object)` is recognized as a one-argument callable boundary,
but object arguments currently fail with a stable unsupported runtime diagnostic
until PHP object handle hash behavior is modeled on top of object identity;
non-object arguments fail with a stable type-boundary diagnostic.
Constructors, `$this` object context binding, method
dispatch, dynamic property names, visibility enforcement for non-public
properties, inheritance and interface relationship checks, object handle
identity, clone expressions, `instanceof` relationship checks, class-name
constant resolution through `::class`, property default values, constructor
arguments, multiple properties in one declaration, class constant
declarations, static property storage, static method dispatch, magic static
receivers such as `self::`, `parent::`, and `static::`, class constants, trait
use inside classes, enum declarations, built-in/internal/extension classes in
`get_declared_classes()`, true results from `interface_exists()`,
true results from `trait_exists()`, true results from `enum_exists()`,
declared/built-in/internal interface entries in `get_declared_interfaces()`,
declared/built-in/internal trait entries in
`get_declared_traits()`, non-public/context-sensitive method listing for
`get_class_methods()`, inherited/trait/interface methods, anonymous classes,
`get_called_class()` class context and late static binding behavior,
`spl_object_id()` handle reuse and clone/destructor interactions,
`spl_object_hash()` hash formatting, handle reuse, and clone/destructor
interactions,
exact native class/interface/method/property ordering, `get_class_vars()` property
defaults, inheritance/trait/interface properties, context-sensitive visibility,
`get_object_vars()` dynamic properties and non-public visibility context,
`get_mangled_object_vars()` dynamic properties, property defaults,
inheritance/trait/interface properties, non-public visibility context,
`array_column()` lossy or non-finite float index values, array/object/resource
index values, magic `__get`, `ArrayAccess`, and exact visibility-context
behavior for non-public properties,
`empty($object->name)` dynamic
property names, non-public visibility context, complex lvalues, magic methods,
`unset($object->name)` property uninitialization, typed/uninitialized property
behavior, dynamic property names, non-public visibility context, magic
`__unset`, references/copy-on-write, and native object lowering are not
supported yet.

LLVM IR emission currently supports a smaller straight-line subset: literal
`null`, booleans, integers, floats, and strings, direct static-variable
assignments from those lowerable values, direct reads of previously assigned
static variables, and later direct static-variable assignments overwriting
earlier lowerable scalar values in the same straight-line lowering pass,
direct `isset($name)` checks over the same current static-variable map, `echo`,
and `print`. Native `isset($name)` folds to false for missing or statically
null variables and true for statically assigned non-null lowerable values.
Native `empty($name)` folds to true for missing variables and statically
falsey lowerable scalar/null values (`null`, `false`, `0`, `0.0`, `""`, and
`"0"`), and false for statically truthy lowerable scalar values. Array offsets,
object properties, complex operands, multiple operands, arrays, unset
interactions, ambiguous truthiness, references/copy-on-write behavior, and
exact native error behavior remain unsupported. Echo conversion for this native subset is limited
to the current literal scalar formatting path: `null` and `false` emit nothing,
`true` emits `1`, integers use `%lld`, floats use `%g`, and strings are emitted
through static string constants. Native binary arithmetic currently lowers
`+`, `-`, and `*` when both operands are already same-type lowerable floats, or
when both operands are lowerable integers and the integer result is statically
proven not to overflow, in the same straight-line subset. It lowers integer
`%` only when the divisor is a statically known positive integer in that
subset, and statically known modulo results remain tracked for later checked
integer arithmetic. Tracked integer expression operands and integer literal
operands for `$x % 1` fold to zero, and bounded tracked integer expression
operands whose possible values all produce the same remainder for a positive
literal divisor fold to that remainder. Integer modulo by one also folds after
both operands lower when the dividend is intentionally untracked, such as an
overflow-sensitive shift result; other modulo cases still require a statically
known positive divisor and keep the documented runtime-check boundary.
Identical tracked integer expression
operands and identical integer literal operands for `-` fold to zero without a redundant
native subtraction, and identical tracked finite float expression operands and
identical finite float literals for `-` fold to `0.0` without a redundant
native subtraction. Identical integer subtraction also folds after both
operands lower when the value is intentionally untracked, such as
overflow-sensitive shift results; other non-identity arithmetic with such
values still rejects because exact overflow tracking is unavailable. Tracked integer expression operands and integer literal
operands for `$x + 0`, `0 + $x`, and `$x - 0` reuse the existing value, and
tracked integer expression operands and integer literal operands for `$x * 1`
and `1 * $x` also reuse the existing value. Tracked integer expression
operands and integer literal operands for `$x * 0` and `0 * $x` fold to zero.
The `+ 0`, `- 0`, `* 1`, and `* 0` identity or annihilator forms also fold
after both operands lower when the other integer operand is intentionally
untracked, such as overflow-sensitive shift results; non-identity arithmetic
with such values still rejects because exact overflow tracking is unavailable.
Tracked integer expression arithmetic for `+`, `-`, and `*` folds to the
known integer literal after checked overflow analysis when tracked possible
integer operands prove one result. Literal-only integer arithmetic and
ambiguous tracked-expression plus tracked-expression integer arithmetic stay
emitted.
Tracked finite float expression operands and finite float literals for
nonzero `$x + 0.0`, `0.0 + $x`, and `$x - 0.0`, and for
`$x * 1.0` and `1.0 * $x`, reuse the existing expression. Single-result
statically known nonzero finite `0.0 - $x` folds to the known negated float
literal. Tracked finite positive float expression operands and finite positive
float literals for `$x * 0.0` and `0.0 * $x` fold to positive `0.0`.
Single-result statically known nonzero finite `$x * -1.0` and `-1.0 * $x`
fold to the known negated float literal. Tracked finite nonzero float
expression arithmetic for `+`, `-`, and `*` folds to the known float literal
when tracked possible finite-float operands prove one nonzero result.
Literal-only float arithmetic, zero-result arithmetic, possible signed zero,
negative, and non-finite float identity/subtraction or
multiplication-by-zero cases, and signed-zero-sensitive multiplication by
`-1.0`, stay emitted or rejected rather than being folded. Mixed int/float
arithmetic, PHP numeric coercions, `/`, dynamic or non-positive modulo
divisors, division/modulo zero checks, modulo coercions, negative-divisor and
min-int modulo edge cases, modulo results that
are not statically known enough for later checked arithmetic, integer overflow
promotion, overflow/INF/NAN behavior, references/copy-on-write behavior, and
exact native error behavior remain unsupported. Mixed int/float `+`, `-`, and
`*` operands are rejected with a
mixed-numeric-specific diagnostic until generated code has PHP numeric
promotion and exact result typing. Boolean, null, and string operands in `+`,
`-`, and `*` are rejected with a scalar-coercion-specific diagnostic until
generated code has PHP numeric coercion and string numeric parsing.
Overflow-sensitive or not-statically-proven integer `+`, `-`, and `*` cases
are rejected with an integer-overflow-specific diagnostic until generated code
has PHP integer overflow promotion and runtime checks. Native `/` is rejected
with a division-specific codegen diagnostic until generated code has PHP
division semantics, runtime zero checks, and no misleading integer truncation.
Dynamic, zero, or non-positive integer modulo divisors are rejected with a
modulo-specific codegen diagnostic until native runtime checks exist; the
remaining arithmetic gaps are rejected with a specific codegen
diagnostic. Finite same-type float `+`, `-`, and `*` results remain bounded
and tracked for later strict-identity folding when every possible result is
proven; float overflow/INF/NAN result tracking remains unsupported. Reads of
variables that were not statically assigned earlier in the same straight-line
native lowering pass are rejected with a specific codegen diagnostic until
generated code has native symbol-table storage, undefined-variable diagnostics,
references/copy-on-write behavior, and exact native error behavior. Native
string concatenation `.` currently lowers when both operands are already
lowerable strings in the same straight-line subset, including ternary operands
that prove one static string result; the result is folded into a generated
static string constant. Empty-string concatenation identity also folds for
already-lowerable string operands, including untracked string pointer
expressions: `$text . ""` and `"" . $text` reuse `$text` without runtime string
allocation. PHP scalar-to-string conversion for concatenation, non-empty
ambiguous string expressions, arrays, objects, resources, runtime string
allocation, references/copy-on-write behavior, and exact native error behavior
remain unsupported and are rejected with a specific codegen diagnostic.
Native comparison lowering currently accepts same-type `null`, boolean,
integer, finite float, known ASCII nonnumeric NUL-free string loose/ordering
comparisons, and identical string-pointer self-comparisons for `==`, `!=`,
`<`, `<=`, `>`, and `>=`, plus strict identity `===` and `!==` for already
lowerable `null`, integers, booleans, floats, and strings in the same
straight-line subset. Static same-type scalar
identity folds at compile time, bounded integer, float, string, and boolean identity fold when all
possible `===`/`!==` outcomes are proven identical. Identical lowerable
dynamic scalar operands fold for integers, booleans, already-lowerable string
pointers, and finite tracked floats, so `$x === $x` and `$x !== $x` avoid
runtime comparisons in those safe scalar cases. Identical lowerable integer
operands also fold for loose/ordering comparisons, including intentionally
untracked integer expressions such as overflow-sensitive shift results:
`$x == $x`, `$x <= $x`, and `$x >= $x` fold true, while `$x != $x`, `$x < $x`,
and `$x > $x` fold false. Dynamic boolean expression
operands compared with boolean literals fold for `$flag === true`, `true ===
$flag`, `$flag !== false`, and `false !== $flag` by reusing the original
native boolean expression, and inverse forms such as `$flag === false`, `false
=== $flag`, `$flag !== true`, and `true !== $flag` use the native boolean
inversion path. Dynamic boolean expression operands compared loosely with
boolean literals fold for `$flag == true`, `true == $flag`, `$flag != false`,
and `false != $flag` by reusing the native boolean expression, while inverse
forms such as `$flag == false`, `false == $flag`, `$flag != true`, and
`true != $flag` use the native boolean inversion path. Dynamic boolean
expression operands ordered against boolean literals also fold within boolean
semantics, reusing the expression, inverting it, or folding to a static boolean
for cases such as `$flag > false`, `$flag < true`, `$flag <= true`, and
`true >= $flag`. Same-type integer and finite-float loose/ordering
comparisons whose tracked possible operands prove one result fold to a static
boolean. Literal-only comparisons still fold, while ambiguous tracked
finite-float comparisons stay emitted as native comparisons.
Boolean expression comparisons whose tracked possible operands prove one
loose/ordering result also fold to that static boolean without emitting a
redundant native boolean comparison. Identical native boolean expression
operands also fold for loose/ordering comparisons, including ambiguous boolean
expressions: `$flag == $flag`, `$flag <= $flag`, and `$flag >= $flag` fold
true, while `$flag != $flag`, `$flag < $flag`, and `$flag > $flag` fold false.
Other ambiguous boolean expression comparisons stay emitted. Identical native
string pointer operands also fold for loose/ordering comparisons, including
untracked string pointer expressions whose possible value set exceeds the
current small tracker: `$text == $text`, `$text <= $text`, and `$text >=
$text` fold true, while `$text != $text`, `$text < $text`, and `$text >
$text` fold false. Non-identical unknown string comparisons stay rejected.
Statically known integer
strict-identity comparison results remain tracked for later boolean scalar
lowering even when the comparison itself stays emitted as `icmp`. Same-type
ambiguous dynamic integer, boolean, float, and already-lowerable string pointer
identity lower through native comparisons and PHP-shaped boolean echo output,
and already lowerable mixed scalar operands with different PHP scalar types
fold without emitting runtime comparison calls. Ambiguous dynamic string
identity uses `strcmp` for string pointers produced by the current native
string ternary subset. Known ASCII nonnumeric string loose/ordering
comparisons fold to a static boolean when every possible safe string outcome
matches; ambiguous safe string loose/ordering comparisons lower through
`strcmp`. Statically known boolean, integer, and finite-float loose/ordering
comparison results remain tracked for later boolean scalar lowering even when
the comparison itself stays emitted as `icmp`/`fcmp`; ambiguous bounded
boolean, finite-float, or string loose/ordering comparison results
remain dynamic and untracked. Ambiguous bounded integer, float, string, or boolean
identity, broader value-correlation proofs across related expressions such as
`$x` and `!$x`,
numeric-looking, non-identical unknown, non-ASCII, or NUL-containing string loose/ordering comparisons,
mixed null or other mixed-type comparisons, untracked or
non-finite float comparisons, dynamic null identity beyond static/type-only folds, PHP
truthiness conversion for loose logical operands, array/object comparisons,
non-lowerable float sources, dynamic string allocation beyond the static
straight-line subset, PHP comparison coercions, and non-scalar comparison
diagnostics remain unsupported and are rejected with a specific codegen
diagnostic. Native unary
lowering currently accepts unary minus on already lowerable integers or floats
and logical not on already lowerable booleans or native boolean expression
results, on `null`, or on known integers, finite floats, and strings whose
possible values all have the same PHP truthiness, in the same straight-line subset.
Dynamic boolean double logical-not expressions such as `!!$flag` reuse the
original native boolean expression instead of emitting redundant inversions.
Double logical-not over known scalar operands such as integers, finite floats,
strings, and `null` folds through the same known-truthiness subset without
emitting boolean operations.
Native lowering folds logical not over single-result statically known native
boolean expression operands to the known boolean result in LLVM IR and in the
C assembly fallback when the C boolean expression has a tracked result. Known
numeric logical-not folds to a static boolean for zero and nonzero known
integer/finite-float operands when all possible values have the same
truthiness. Known string logical-not folds to a static boolean for `""`, `"0"`,
and known-truthy string operands when all possible string values have the same
truthiness. Null logical-not folds to `true` without claiming broader null
truthiness beyond the documented logical binary folding subset. Integer
unary-minus results remain
statically tracked for later checked integer arithmetic when all bounded
possible negation results are proven not to overflow; single-result
statically known integer operands fold to the known negated result without a
redundant native unary-minus operation. Boolean, string, null, array, and
object unary-minus operands, PHP numeric coercion, ambiguous numeric or string
logical-not truthiness, untracked numeric/string logical-not expressions,
non-finite float logical-not truthiness, null truthiness outside logical-not,
other truthiness conversion, unary integer overflow behavior, float overflow/INF/NAN result tracking,
references/copy-on-write behavior, and exact native error behavior remain
unsupported and are rejected with a specific codegen
diagnostic. Finite float unary-minus results remain tracked for later
strict-identity folding when every possible negation result is proven;
single-result statically known nonzero finite float operands fold to the known
negated result without a redundant native unary-minus operation. Native binary
logical lowering currently accepts `&&`, `||`, `and`, `or`, and `xor` only
when both operands are already lowerable booleans or native boolean expression
results, or when both already-lowerable scalar operands have one statically
known PHP truthiness result, in the same straight-line subset; static boolean
pairs fold, and static boolean identity and annihilator edges such as `true ||
$flag`, `false && $flag`, `$flag && true`, and `$flag xor false` preserve the
proven boolean result for later scalar lowering. Identical native boolean
expression operands for `&&`/`and` and `||`/`or` reuse the existing expression
without a redundant native boolean operation, and identical native boolean
expression operands for `xor` fold to `false`. Native boolean expression
operations whose tracked possible operands prove one result fold to that
static boolean without a redundant native boolean operation. Known scalar
logical operands whose null, integer, finite-float, or string truthiness is
unambiguous fold to a static boolean result without emitting a native boolean
operation; statically decisive known-left `&&`/`and` and `||`/`or`
short-circuit cases such as `false && rhs` and `true || rhs` lower without
lowering the skipped right-hand operand. Ambiguous dynamic boolean expressions
lower to native boolean operations with PHP-shaped boolean echo output. General
PHP truthiness conversion, dynamic short-circuiting, `xor` right-hand skipping,
selected/evaluated unsupported right-hand operands, ambiguous scalar truthiness,
untracked scalar logical operands, non-finite float truthiness, null
coalescing, arrays, objects, references/copy-on-write behavior, exact native
error behavior, linking/execution, and broader native lowering remain
unsupported and are rejected with a specific codegen
diagnostic. Native bitwise lowering currently accepts binary `&`, `|`, and
`^`, plus unary `~`, only when operands are already lowerable integers in the
same straight-line subset. Bounded statically known integer bitwise and unary
bitwise-not results remain tracked for later checked integer arithmetic.
Single-result statically known integer operands for unary `~` fold to the
known bitwise-not result without a redundant native bitwise-not operation.
Double unary bitwise-not `~~$x` over an already-lowerable integer operand
reuses `$x`, including intentionally untracked integer expressions such as
overflow-sensitive shift results.
Identical tracked integer expression operands and identical integer literal
operands for `&` and `|` reuse the existing value, and identical tracked
integer expression operands and identical integer literal operands for `^`
fold to zero. Identical integer operands also fold after both operands lower
when the value is intentionally untracked, such as overflow-sensitive shift
results: `$x & $x` and `$x | $x` reuse `$x`, while `$x ^ $x` folds to zero.
Tracked integer expression operands and integer literal operands
for `$x & -1` and `-1 & $x`, and for `$x | 0`, `0 | $x`, `$x ^ 0`, and
`0 ^ $x`, reuse the existing value. Tracked integer expression operands and
integer literal operands for `$x & 0` and `0 & $x` fold to zero, while
`$x | -1` and `-1 | $x` fold to `-1` after both operands lower. Single-known
integer operands for `$x ^ -1` and `-1 ^ $x` fold to the known bitwise-not
result. The `& 0`, `& -1`, `| 0`, and `^ 0` identity or annihilator forms also
fold after both operands lower when the other integer operand is intentionally
untracked, such as overflow-sensitive shift results.
Tracked integer expression bitwise operations for `&`, `|`, and `^` fold to
the known integer literal when tracked possible integer operands prove one
result. Literal-only integer bitwise operations and ambiguous
tracked-expression plus tracked-expression bitwise operations stay emitted.
Native shift lowering accepts `<<` and `>>` only for already lowerable integer
left operands with literal shift counts or tracked integer expression counts
that prove one value from 0 through 63; right shifts use arithmetic shift for
signed integer results. Bounded statically known safe shift results remain
tracked for later checked integer arithmetic; tracked integer expression
operands and integer literal operands for `$x << 0` and `$x >> 0` reuse the
existing value. Those shift-by-zero identities also fold after both operands
lower when the left integer operand is intentionally untracked, such as an
overflow-sensitive shift result. Tracked single-result integer expression shifts
with static safe nonzero counts fold to the known integer literal. Literal-only
shifts and non-single tracked integer shifts stay emitted. Overflow-sensitive left-shift
result sets remain unknown so later arithmetic rejects them instead of
implying PHP overflow semantics. Ambiguous dynamic shift counts, negative or
large counts, PHP bytewise string bitwise behavior, scalar-to-int
coercion for non-integer operands, arrays, objects, references/copy-on-write
behavior, exact native error behavior, linking/execution, and broader native
lowering remain unsupported and are rejected with a specific codegen
diagnostic. Native ternary lowering accepts
full ternary `condition ? if_true : if_false` only when the condition is
already a lowerable boolean or native boolean expression and both branch values
are already lowerable integers, booleans, floats, strings, or both branches
are `null` in the same straight-line subset, or when the condition is a
statically known boolean and both branch values are already lowerable scalar
values, or when the condition and both branches are the same direct variable
whose current value is already lowerable. Dynamic mixed-type branch values are rejected until native tagged
values exist. Dynamic non-null ternaries emit `select` rather than branch
blocks, identical static string branches fold to that string without a pointer
select, identical boolean expression branches fold to the reused expression
without a redundant boolean select, identical tracked integer expression
branches and identical integer literal branches fold to the reused value
without a redundant integer select, and identical integer branches also fold
after both branches lower when the integer value is intentionally untracked,
such as an overflow-sensitive shift result. Identical direct-variable full
ternaries such as `$value ? $value : $value` reuse the direct variable value
without proving truthiness when all three operands are the same already-lowerable
direct variable, including untracked integer, non-finite float-producing, and
string pointer expressions, boolean expressions, and null values. Identical tracked float expression
branches and identical float literal branches fold to the reused value
without a redundant float select, and identical float branches also fold after
both branches lower when the value is intentionally untracked, such as a
non-finite overflowing float multiplication. Dynamic boolean literal branches fold
without a boolean select for `$flag ? true : false`,
`$flag ? false : true`, `$flag ? true : true`, and
`$flag ? false : false`, dynamic `null`/`null` ternaries fold to `null`, and
static boolean ternaries fold to the selected branch value. Dynamic integer,
finite-float, and boolean ternaries whose possible branch values collapse to a
single known result fold to that scalar without a redundant select; ambiguous
same-type ternaries stay emitted. Full ternary conditions with null or with
single-known integer, finite-float, or known-string truthiness lower only the
selected already-lowerable branch; direct null-variable conditions select the
false branch without lowering unsupported true-branch calls. Dynamic boolean
full ternaries still require both branches to lower before selection.
Ambiguous integer, float, or string conditions, untracked string conditions,
non-finite float result tracking, and non-finite float conditions remain
rejected, and dynamic branch skipping for unsupported or side-effecting
branches remains unsupported. Dynamic
integer ternaries and later checked integer arithmetic track up to four
statically known possible values; combinations with more possible results
remain unsupported. Native short ternary `?:` accepts lowerable boolean
conditions in the same straight-line subset; dynamic boolean forms require a
lowerable boolean fallback, static-false forms return any already-lowerable
scalar fallback, and static-true forms fold to `true` without lowering the
fallback. Single-known integer conditions also fold through integer
truthiness: proven nonzero integer conditions reuse the integer result, and
proven zero integer conditions use the fallback. Single-known finite float
conditions fold through float truthiness the same way, with proven nonzero
finite floats reusing the float result and proven zero floats using the
fallback. Known string conditions fold through PHP string truthiness when all
possible values have the same truthiness: non-empty strings except `"0"` reuse
the string result, while `""` and `"0"` use the fallback. Identical direct
boolean-, integer-, float-, and string-variable short ternaries such as
`$flag ?: $flag`, `$value ?: $value`, and `$text ?: $text` also reuse
already-lowerable expressions without proving broader truthiness, including
boolean expressions, untracked integer expressions, untracked non-finite
float-producing expressions, and untracked string pointer expressions. Null
short ternaries use the fallback for `null ?:
fallback`, including direct null-variable fallback forms such as
`$value ?: $value`; broader null truthiness in logical
binaries or null coalescing remains unsupported. General PHP
truthiness, lazy branch evaluation for unsupported or side-effecting branches,
ambiguous string truthiness, non-identical untracked integer, float, or string expressions,
non-finite float truthiness, other non-boolean truthiness, null coalescing `??`, null-aware
lookup, arrays, objects, references/copy-on-write behavior, exact native error
behavior, linking/execution, and broader native lowering remain unsupported
and are rejected with a specific codegen diagnostic. Native lowering
statically folds direct `gettype`, `is_null`, `is_bool`,
`is_int`/`is_integer`/`is_long`, `is_float`/`is_double`, `is_string`,
`is_array`, `is_scalar`, and `is_numeric` calls only when their single argument
is already in the straight-line native scalar/null subset. Native
`is_numeric` also folds literal and tracked string values only when the current
numeric-string grammar proves the result statically. Direct `is_countable` and
`is_iterable` and `is_object` calls fold to `false` for already-lowerable
scalar/null/string operands only, and direct scalar/null/string
`get_debug_type` calls fold to the current runtime type-name strings.
Direct `strlen($value)` calls fold only when `$value` is an already-lowerable
known string operand, including tracked string expressions whose possible
values have one uniform byte length. A selected-`clang` assembly snapshot
validates that this existing folded LLVM IR is handed to the chosen backend
through stdin without changing production lowering behavior.
Direct `class_exists`, `interface_exists`, `trait_exists`, and `enum_exists`
calls with already-lowerable string names and optional already-lowerable
boolean autoload flags fold to `false` in native output because native
lowering still rejects class/interface/trait/enum declarations and has no
autoload or native class table.
Direct `property_exists` and `method_exists` calls with already-lowerable
string class names and already-lowerable string member names also fold to
`false` for the same no-native-class-table boundary.
Direct `is_a` and `is_subclass_of` calls with already-lowerable string
object/class names, already-lowerable string target class names, and optional
already-lowerable boolean `allow_string` flags fold to `false` without
claiming inheritance or native class-table support.
Direct `is_callable($value)` calls fold when `$value` is an already-lowerable
string value with a uniform known lookup result in the documented builtin
table, or when `$value` is an already-lowerable non-string scalar/null value,
which folds to `false`. Direct `is_callable($value, $syntax_only)` calls also
fold when `$value` is an already-lowerable string or non-string scalar/null
value and `$syntax_only` is an already-lowerable boolean: true syntax-only
flags return true for string values without name lookup, non-string
scalar/null values return false, while false flags use the same documented
builtin lookup as the one-argument form.
Direct `function_exists($name)` calls fold when `$name` is an already-lowerable
string value with a uniform known answer in the documented builtin table:
documented callable builtins, including `array_change_key_case` and
`array_column`, `array_count_values`, `array_sum`, `array_product`, and
`array_reduce`, fold to
`true`, and missing names fold to `false`. Direct calls to array builtins such
as `array_change_key_case(...)`, `array_column(...)`, `array_sum(...)`, and
`array_product(...)` and callback-driven forms such as `array_reduce(...)`
still reject under the native array-lowering boundary.
Direct `defined($name)` calls fold when `$name` is an already-lowerable string
whose possible values are supported unqualified constant names with a uniform
answer against the current exact built-in constant table: `CASE_LOWER`,
`CASE_UPPER`, `ARRAY_FILTER_USE_BOTH`, `ARRAY_FILTER_USE_KEY`, and
`SORT_REGULAR`, `SORT_NUMERIC`, and `SORT_STRING` fold to true, while other
supported unqualified names fold to false.
Array/object operands remain rejected until native array/object lowering
exists. Dynamic calls, wrong arity, non-string `function_exists` names,
non-string `strlen` operands and exact string-coercion diagnostics,
non-bool `is_callable` syntax-only flags, callable-name output parameters,
array/object/method callables,
user-defined functions in native output, namespace/import/autoload-aware
lookup, extension-loaded functions outside the documented builtin table,
general callable builtin dispatch, runtime call lookup, stack frames, type
diagnostics, unsupported `defined(...)` names, and dynamic string-call
dispatch remain unsupported.
Executable magic constants `__LINE__`, `__FILE__`, `__DIR__`, and
`__FUNCTION__` are rejected by native lowering until generated code has source
mapping, path canonicalization, and function-context tracking.
Built-in constants, runtime-defined constants, bare constant reads, top-level
`const` declarations, `define()`/`constant()`, and unsupported
`defined(...)` forms are rejected by native lowering until generated code has
native constant values, source-order definitions, namespace-aware lookup, and
exact native error behavior.
Class declarations, object instantiation, public property reads/writes, and
object metadata builtins beyond scalar/null/string `is_object`,
scalar/null/string `get_debug_type`, and direct string-name metadata-exists
false folding, including string/string `property_exists` and `method_exists`,
and string/string relationship false folding for `is_a` and `is_subclass_of`,
are rejected by native lowering until generated code has native object layout,
handles, visibility, method dispatch, class metadata tables, inheritance,
autoload interaction, and exact native error behavior.
Array literals, array offset reads/writes, `foreach` array iteration, array
offset `unset`, and array builtins are rejected by native lowering until
generated code has native array storage layout, key normalization,
copy-on-write, references, callback dispatch, and exact native error behavior.
Control-flow statements including `if`/`elseif`/`else`, `while`, `for`,
`do ... while`, `switch`, `break`, and `continue` are rejected by native
lowering until generated code has PHP truthiness, branch layout, loop control,
switch fallthrough, references/copy-on-write side-effect behavior, and exact
native error behavior.
Mutation forms that are still interpreter-only, including compound assignment,
null coalescing assignment, increment/decrement, assignment expressions, direct
variable `unset`, and multiple-operand `unset`, are rejected by native lowering
until generated code has read-modify-write ordering, null-aware mutation, unset
symbol-table effects, references/copy-on-write, and exact native error behavior.
Unsupported programs and broader PHP coercions are rejected with structured
codegen errors.

Fixture tests live under `tests/fixtures`. For editor-friendly expected-output
files, the test runner strips one final newline from `.stdout` and `.stderr`
fixtures; use a blank final line when the expected program output should include
a trailing newline. Fixtures with a sibling `.phpc-only` marker are still tested
by `phpc`, but are skipped by optional system PHP comparison when the project
intentionally reports different diagnostics.

See `docs/SUPPORT.md` for the detailed support matrix.

## Operations

Operational automation lives in `docs/OPERATIONS.md`.

- `tools/run-tests.sh` runs the full project test suite.
- `tools/checkpoint.sh "message"` runs the suite and commits all current changes
  only if tests pass.
- `tools/codex-loop.sh` runs a bounded Codex supervisor loop when
  `CODEX_RUNNER` is set.
- `tools/codex-yolo-forever.sh` runs an infinite unattended yolo loop with
  durable memory in `docs/LOOP_MEMORY.md`.
