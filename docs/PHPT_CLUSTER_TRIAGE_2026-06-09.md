# PHPT Cluster Triage: 2026-06-09

Evaluator task: find PHPT clusters adjacent to current support where one
generic semantic fix should unlock multiple rows without shaping behavior to
individual expected output.

Runner used:

```sh
cargo build --bin phpc
PHPC_BIN="$PWD/target/debug/phpc" php /home/claude/php-src-phpt/run-tests.php -q -p "$PWD/target/debug/phpc" <tests...>
```

`run-tests.php` consistently emits an initial self-info probe failure through
the minimal `phpc` runner (`run-test-info.php`, `PHP_VERSION`, and `?` parse
noise). The row-level PASS/FAIL summaries and generated `.diff` files below are
still usable evidence.

## 1. Add `print_r()` for Current Boxed Values

Priority: P1, small and likely to unlock rows immediately.

Command:

```sh
PHPC_BIN="$PWD/target/debug/phpc" php /home/claude/php-src-phpt/run-tests.php -q -p "$PWD/target/debug/phpc" \
  /home/claude/php-src-phpt/ext/standard/tests/general_functions/print_r_null.phpt \
  /home/claude/php-src-phpt/ext/standard/tests/general_functions/print_r_bools.phpt \
  /home/claude/php-src-phpt/ext/standard/tests/general_functions/print_r_strings.phpt \
  /home/claude/php-src-phpt/ext/standard/tests/general_functions/print_r_ints.phpt \
  /home/claude/php-src-phpt/ext/standard/tests/general_functions/print_r_arrays.phpt \
  /home/claude/php-src-phpt/tests/lang/array_shortcut_001.phpt
```

Evidence: 0/6 pass. The scalar and array rows fail at the same boundary:

```text
Fatal error: Call to undefined function print_r()
```

Minimal repro:

```sh
target/debug/phpc -r '$a = ["x" => 1, 2]; print_r($a);'
php -r '$a = ["x" => 1, 2]; print_r($a);'
```

Current `phpc` output is the undefined-function fatal. PHP prints the ordered
array:

```text
Array
(
    [x] => 1
    [0] => 2
)
```

Likely missing generic semantic: register `print_r()` in the internal-function
table and implement a reusable display printer over `PtnValue` for null,
booleans, integers, floats, strings, and ordered arrays. Reuse array insertion
order and key canonicalization already used by `var_dump()`, `foreach`, and
array comparisons. Keep recursion, references, objects, resources, and
`print_r($value, true)` as named follow-up boundaries if needed.

Follow-up bead filed: `ptn-06d` - Add `print_r()` internal for scalar and
ordered-array `PtnValue` output.

## 2. Make Runtime Strings Length-Aware

Priority: P1, broad string/internal payoff with direct evidence from existing
supported functions.

Command:

```sh
PHPC_BIN="$PWD/target/debug/phpc" php /home/claude/php-src-phpt/run-tests.php -q -p "$PWD/target/debug/phpc" \
  /home/claude/php-src-phpt/ext/standard/tests/strings/bin2hex.phpt \
  /home/claude/php-src-phpt/ext/standard/tests/strings/bin2hex_001.phpt \
  /home/claude/php-src-phpt/ext/standard/tests/strings/md5raw.phpt \
  /home/claude/php-src-phpt/ext/standard/tests/strings/quoted_printable_decode_basic.phpt \
  /home/claude/php-src-phpt/ext/standard/tests/strings/ord_basic.phpt \
  /home/claude/php-src-phpt/ext/standard/tests/strings/strlen.phpt
```

Evidence: 2/6 pass. Plain `bin2hex_001` and
`quoted_printable_decode_basic` pass. Byte-heavy rows fail:

- `bin2hex.phpt`: expected byte `00` at the front; actual output starts at
  `01`.
- `md5raw.phpt`: raw digest output truncates at embedded NUL bytes, e.g.
  expected `d41d8cd98f00b...`, actual `d41d8cd98f`.
- `ord_basic.phpt`: byte escape coverage currently misreads `"\x0A"` and
  `"\xFF"` before the loop can complete.
- `strlen.phpt`: this broad row later hits unsupported `@`, but its source
  contains multiple NUL-length cases that the minimal repro below isolates.

Minimal repros:

```sh
target/debug/phpc -r 'echo bin2hex(chr(0) . chr(1)), "\n";'
php -r 'echo bin2hex(chr(0) . chr(1)), "\n";'

target/debug/phpc -r 'echo bin2hex(md5("", true)), "\n";'
php -r 'echo bin2hex(md5("", true)), "\n";'

target/debug/phpc -r 'var_dump(strlen(chr(0)));'
php -r 'var_dump(strlen(chr(0)));'
```

Observed:

- `phpc`: `01`; PHP: `0001`
- `phpc`: `d41d8cd98f`; PHP: `d41d8cd98f00b204e9800998ecf8427e`
- `phpc`: `int(0)`; PHP: `int(1)`

Likely missing generic semantic: replace C-string-only string values with a
length-aware byte string representation and update scalar string conversion,
literal emission, `chr()`, `strlen()`, `bin2hex()`, digest raw output, offset
reads, concatenation, and output helpers to carry explicit lengths. Separately
add PHP `\xHH`/octal escape parsing; do not hide that inside individual
function fixes.

Existing follow-up bead: `ptn-cqu.26` - PHPT binary string internals failures.
`ptn-cqu.29` is also relevant as a runtime-emission split precursor if the
string representation work needs lower-conflict runtime editing first.

## 3. Add User Function Call-Frame Introspection

Priority: P2, clean cluster over already-supported top-level user functions.

Command:

```sh
PHPC_BIN="$PWD/target/debug/phpc" php /home/claude/php-src-phpt/run-tests.php -q -p "$PWD/target/debug/phpc" \
  /home/claude/php-src-phpt/tests/lang/func_num_args.001.phpt \
  /home/claude/php-src-phpt/tests/lang/func_num_args.002.phpt \
  /home/claude/php-src-phpt/tests/lang/func_get_arg.001.phpt \
  /home/claude/php-src-phpt/tests/lang/func_get_args.001.phpt \
  /home/claude/php-src-phpt/Zend/tests/func_num_args_basic.phpt \
  /home/claude/php-src-phpt/Zend/tests/func_get_args_basic.phpt
```

Evidence: 0/6 pass. The clean `tests/lang` rows all fail on missing internals:

```text
Fatal error: Call to undefined function func_num_args()
Fatal error: Call to undefined function func_get_arg()
Fatal error: Call to undefined function func_get_args()
```

The larger Zend rows then expose additional future work such as `try`/classes
and `call_user_func()`, but the simple rows are already enough to justify a
generic call-frame slice.

Minimal repro:

```sh
target/debug/phpc -r 'function f($a) { $a = 5; echo func_get_arg(0), "\n"; var_dump(func_num_args()); } f(2, 3);'
php -r 'function f($a) { $a = 5; echo func_get_arg(0), "\n"; var_dump(func_num_args()); } f(2, 3);'
```

Likely missing generic semantic: generated user-function calls need a runtime
call frame containing actual argument count, argument slots, and parameter/local
slot mapping. Extra arguments should no longer disappear; `func_num_args()`,
`func_get_arg()`, and `func_get_args()` should read the active frame and report
global-scope/error cases through the common diagnostic channel. This also
creates the right foundation for argument-count diagnostics, variadics, and
`call_user_func*`.

Follow-up bead filed: `ptn-ebd` - Add user-function call frames and `func_*`
introspection.

## 4. Implement Array Element Lvalues, `unset()`, and Append

Priority: P2, larger than `print_r()` but directly adjacent to ordered arrays,
array reads, `isset()`/`empty()`, `foreach`, and `count()`.

Command:

```sh
PHPC_BIN="$PWD/target/debug/phpc" php /home/claude/php-src-phpt/run-tests.php -q -p "$PWD/target/debug/phpc" \
  /home/claude/php-src-phpt/tests/basic/array_key_exists_null_deprecation.phpt \
  /home/claude/php-src-phpt/tests/basic/array_null_offset_deprecation.phpt \
  /home/claude/php-src-phpt/tests/lang/array_shortcut_001.phpt \
  /home/claude/php-src-phpt/Zend/tests/offset_assign.phpt \
  /home/claude/php-src-phpt/Zend/tests/assign_dim_op_undef.phpt
```

Evidence: 1/5 pass. `array_key_exists_null_deprecation.phpt` passes. Rows that
write or unset array elements fail at the generic parser/lvalue boundary:

```text
Fatal error: expected assignment ... array_null_offset_deprecation.php on line 6
Fatal error: expected assignment ... assign_dim_op_undef.php on line 2
Fatal error: expected assignment ... offset_assign.php on line 3
```

`array_shortcut_001.phpt` fails for missing `print_r()`, so it belongs to the
first cluster rather than this mutation cluster.

Minimal repro:

```sh
target/debug/phpc -r '$a = []; $a["1"] = "one"; echo $a[1], "\n"; unset($a[1]); var_dump(isset($a[1]));'
php -r '$a = []; $a["1"] = "one"; echo $a[1], "\n"; unset($a[1]); var_dump(isset($a[1]));'
```

Likely missing generic semantic: parse and lower array-dimension lvalues for
ordinary assignment, compound assignment, append (`$a[] = ...`), and `unset()`.
The runtime should mutate the existing ordered map with current key
canonicalization, preserve insertion order, update next automatic integer key,
route null-key deprecations through the existing diagnostic boundary, and
model scalar-container errors/autovivification as a follow-on slice.

Follow-up bead filed: `ptn-zj6` - Add array dimension assignment, append, and
unset.

## Lower-Priority Adjacent Work

- Expression-form `print` is still unsupported: `target/debug/phpc -r 'echo print "x";'`
  reports `expected expression`, while PHP outputs `x1`. Public `print_*`
  rows are currently also blocked by complex interpolation and `@`, so this is
  not as clean a PHPT cluster as `print_r()`.
- Assignment expressions are still unsupported:
  `target/debug/phpc -r '$a = ($b = 3); var_dump($a, $b);'` reports an
  unexpected `=`, while PHP returns both `int(3)`. This is a useful future IR
  capability, but public rows often mix variable variables, objects, or file
  APIs, so it is lower priority than the four clusters above.
- Mixed PHP/HTML mode switching between PHP blocks remains a parser gap, but
  the obvious public row `tests/lang/024.phpt` combines inline HTML with
  variable variables, array writes, assignment expressions, and other broad
  features. Track it after smaller parser/value clusters land.
