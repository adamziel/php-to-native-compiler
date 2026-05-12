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

## Current Status

Milestone 1 is in progress. The interpreter/runtime path supports a small PHP
subset:

- `echo`
- integer, float, and string literals
- static variables, assignment, and `unset($name)` through per-scope symbol
  tables
- `+`, `-`, `*`, `/`, `.`
- loose scalar comparisons and scalar strict identity comparisons used by
  control flow
- `if` / `elseif` / `else`
- `while`, C-style `for` loops over the documented header subset,
  `do ... while` post-condition loops, `switch`/`case`/`default` statements
  over the current scalar comparison subset, and array `foreach` in value-only
  and key/value forms, including `break;` for innermost loops or switch
  statements and `continue;` for the innermost executing loop
- function declarations, positional calls with trailing default parameter
  values over the documented constant-expression and unqualified constant
  reference subset, recursive calls up to the documented guard, `return`, and
  isolated local scopes for user-function calls
- dynamic function calls through string-valued expressions that resolve to the
  documented callable builtin subset or user-defined functions
- top-level class declarations registered as metadata, with property and method
  metadata for the documented subset
- minimal object instantiation with `new ClassName()` for declared classes that
  do not define constructors; public instance properties can be read and
  written by static property name and checked with `isset($object->name)`
- magic constants `__LINE__`, evaluated from the expression token's source
  line, and `__FILE__`, evaluated from the current `phpc run` input path when
  one is available
- narrow global constant resolution: exact uppercase `ARRAY_FILTER_USE_KEY` and
  `ARRAY_FILTER_USE_BOTH` work as bare built-in constants, and
  `define($name, $value)`, `constant($name)`, `defined($name)`, and bare
  user-constant reads support unqualified names over the documented
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
- direct `empty($name)` and `empty($array[$key])` checks for the documented
  variable/array-offset subset
- builtins for the documented scalar/array/object subset: `strlen`, `isset`,
  `empty`, `count`, `array_key_exists`, `array_key_first`, `array_key_last`,
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
  integer/string key values, variadic `array_intersect_key` and
  `array_diff_key` over two or more arrays with first-array key/value
  preservation, variadic `array_diff` and `array_intersect` over two or more
  arrays with scalar value comparisons and first-array key/value preservation,
  `array_unique` with scalar string-form value deduplication and
  first-occurrence preservation,
  `array_flip` over integer/string array values,
  `array_fill_keys` over integer/string key values, `array_count_values` over
  integer/string array values, `array_sum` over the current scalar
  numeric-coercion subset, `array_product` over the current scalar
  numeric-coercion subset,
  `array_reduce($array, $callback[, $initial])` over the current string-valued
  callback subset, `array_filter` without a callback, with a `null` callback,
  and with string-valued value-only callbacks, including explicit integer mode
  flag `0` for those value-only paths, plus string-valued key-only callbacks
  through integer mode flag `2` and string-valued value/key callbacks through
  integer mode flag `1`, `define`, `constant`, and `defined` over the documented
  runtime-defined/built-in constant name slice, `array_map` over the current
  one-array null-callback identity, variadic null-callback zip, and variadic
  string-callback subset with one-array key preservation and multi-array
  reindexing,
  `in_array` and `array_search` including strict scalar searches, `var_dump`,
  and `print_r`
- stable runtime diagnostics for the currently covered runtime errors,
  including unresolved or non-string dynamic function calls, unsupported
  `global` declarations, duplicate or unsupported `define(...)` constant
  definitions, unsupported `defined(...)` name arguments, unknown bare global
  constants outside the current built-in/runtime-defined slice such as
  `PHP_VERSION`,
  invalid `break`/`continue` outside a loop, and runaway recursion
- stable lex/parse diagnostics for unsupported dynamic/function features
  including variable variables, include/require/eval constructs,
  namespace and `use` declarations, namespace-qualified function/class names,
  variadics, references, parameter and return type declarations, static local
  variable declarations, magic constants other than executable `__LINE__` and
  `__FILE__`,
  closures, named arguments,
  `declare(strict_types=1)`,
  unsupported nested, namespace-aware, or dynamic-value `const` declarations,
  unsupported array spread/reference elements, unsupported broader
  `unset(...)` forms such as property, append-offset, and nested unset,
  unsupported `foreach` by-reference/destructuring forms, unsupported
  comma-separated `for` header expression lists, expression-form
  `do ... while`, alternate `if`/`elseif`/`else` colon/`endif` syntax,
  expression-form or alternate-syntax `switch`, `break`/`continue` depth
  arguments, object method calls, dynamic property names, anonymous classes,
  unsupported class forms, static member access, and class constants

`php_runtime` also contains a tested object/class metadata registry and minimal
object values. `phpc run` can instantiate declared constructor-free classes,
read/write public instance properties by static name, and check those public
properties with `isset`, but constructors, `$this`, method dispatch, dynamic
property names, visibility enforcement for non-public properties, object handle
identity, static property storage, static method dispatch, class constants, and
native object lowering are not supported yet.

LLVM IR emission currently supports a smaller straight-line subset and rejects
unsupported programs with a structured codegen error.

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
