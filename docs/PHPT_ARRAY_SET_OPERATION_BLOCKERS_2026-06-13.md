# PHPT Array Set Operation Blockers: 2026-06-13

Issue: `ptn-6fbw`

Broad baseline source:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

The generated 1k broad manifest was
`.runtime/phpt-baseline/20260613T180826Z/phpt-baseline-1000.txt`, using
php-src revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

The classify-only baseline selected 1,000 broad PHPT rows, kept 430 runnable,
and excluded 570:

```text
classification.unsupported-language: rows=404
classification.unsupported-class-metadata: rows=51
classification.unsupported-ini: rows=73
classification.unsupported-extension: rows=20
classification.sapi-behavior: rows=13
classification.harness-cleanup: rows=4
classification.process-boundary: rows=3
classification.external-service: rows=1
classification.environment-assumption: rows=1
```

The current 1k runnable set contains 274 `ext/standard/tests/array` rows.
A broad array run was started from
`.runtime/ptn-6fbw-array-runnable-before.txt`; it reached
`array_fill_error2.phpt` and was terminated because PTN spawned long-lived
`phpc` processes for the deliberate huge-allocation case
`array_fill(0, 2147483647, 1)`. That row needs a generic allocation-size guard
before it can stay in long focused array runs.

Focused set-operation evidence:

```sh
awk '/^ext\/standard\/tests\/array\/array_(diff|intersect)/ {print}' \
  .runtime/phpt-progress/runnable-20260613T180826Z.txt \
  > .runtime/ptn-6fbw-array-diff-intersect-current.txt
tools/run-bounded-phpt.sh .runtime/ptn-6fbw-array-diff-intersect-current.txt
```

The focused run generated
`.runtime/phpt-progress/run-20260613T181328Z-manifest.log` and selected 62
runnable rows: 47 passed, 15 failed, 0 skipped, 0 warned.

## Failure Buckets

### Set-Operation Value Comparison and Stringification

10 rows fail because `array_diff()` / `array_intersect()` value comparison does
not yet fully model PHP's string comparison path for binary strings, multiline
strings, arrays, and objects. The non-key set operation path currently funnels
through `ptn_array_value_strings_equal()` and
`ptn_array_intersect_or_diff()` in
`src/backend/runtime/internals_internal_functions.c`.

Rows:

```text
ext/standard/tests/array/array_diff_assoc_variation10.phpt
ext/standard/tests/array/array_diff_leak_custom_type_checks.phpt
ext/standard/tests/array/array_diff_variation10.phpt
ext/standard/tests/array/array_diff_variation3.phpt
ext/standard/tests/array/array_diff_variation4.phpt
ext/standard/tests/array/array_diff_variation9.phpt
ext/standard/tests/array/array_intersect_assoc_variation10.phpt
ext/standard/tests/array/array_intersect_assoc_variation9.phpt
ext/standard/tests/array/array_intersect_variation10.phpt
ext/standard/tests/array/array_intersect_variation9.phpt
```

Representative gaps:

- Binary-safe rows compare strings containing NUL bytes or escaped bytes.
- Multiline string rows show PTN preserving escaped `\t` / `\n` text where PHP
  has actual tab/newline bytes.
- Nested-array rows expect repeated `Array to string conversion` warnings
  while still completing the set operation.

### Argument Validation and Catchable Diagnostics

3 rows fail because argument validation exits as a fatal error instead of
raising the catchable PHP `TypeError`/`ArgumentCountError` shape expected by
the PHPT rows.

Rows:

```text
ext/standard/tests/array/array_diff_1.phpt
ext/standard/tests/array/array_diff_single_array.phpt
ext/standard/tests/array/array_diff_uassoc_error.phpt
```

Representative gaps:

- `array_diff([], 1)` should throw a catchable `TypeError`; PTN currently emits
  `Fatal error: array_diff(): Argument #2 ($arrays) must be of type array, int given`.
- Single-argument `array_diff($array)` should return the input array for the
  tested set-operation variants; PTN currently treats it as an arity fatal.
- `array_diff_uassoc()` callback-position validation must identify invalid
  callbacks before treating callback operands as array arguments.

### Missing Callback Diagnostics

2 rows fail because missing callback names reached through set-operation
callbacks currently call `ptn_call_callable()` and produce
`Fatal error: Call to undefined function unknown_function()` instead of PHP's
argument-specific callback validation message.

Rows:

```text
ext/standard/tests/array/array_diff_ukey_variation10.phpt
ext/standard/tests/array/array_intersect_ukey_variation8.phpt
```

The generic fix belongs in the callback argument-validation path shared by
`array_diff_ukey()`, `array_intersect_ukey()`, and the other callback-based
set helpers.

## Next Architecture Step

The next implementation slice should add a shared PHP array set-operation
comparison helper that:

1. Converts values to PHP comparison strings with the same warning behavior as
   PHP for arrays/objects.
2. Preserves binary strings and multiline literal bytes through parser,
   runtime storage, comparison, and var-dump output.
3. Validates callback arguments before dispatch so missing callbacks produce
   `Argument #N must be a valid callback` diagnostics.
4. Keeps invalid array arguments catchable where PHP expects a catchable
   exception instead of terminating the process.
5. Adds size guards for array-producing internals such as `array_fill()` before
   broad array runs include huge-allocation PHPT rows.
