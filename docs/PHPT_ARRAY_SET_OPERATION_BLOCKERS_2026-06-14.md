# PHPT Array Set Operation Blockers: 2026-06-14

Issue: `ptn-igxz`

Broad baseline source:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

The generated broad manifest was
`.runtime/phpt-baseline/20260613T233819Z/phpt-baseline-1000.txt`, using
php-src revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

The classify-only run selected 1,000 PHPT rows, kept 443 runnable, and excluded
557. The focused `array_chunk*` slice from that runnable set was already green:
32 selected, 32 passed.

## Focused Diff Evidence

Focused manifest:

```sh
awk '/^ext\/standard\/tests\/array\/array_diff/ {print}' \
  .runtime/phpt-progress/runnable-20260613T233819Z.txt \
  > .runtime/ptn-igxz-array-diff-before.txt
tools/run-bounded-phpt.sh .runtime/ptn-igxz-array-diff-before.txt
```

Before implementation, `.runtime/phpt-progress/run-20260613T235031Z-manifest.log`
selected 39 runnable rows: 32 passed, 7 failed.

Resolved failures:

```text
ext/standard/tests/array/array_diff_1.phpt
ext/standard/tests/array/array_diff_single_array.phpt
ext/standard/tests/array/array_diff_uassoc_error.phpt
ext/standard/tests/array/array_diff_ukey_variation10.phpt
ext/standard/tests/array/array_diff_variation3.phpt
ext/standard/tests/array/array_diff_variation4.phpt
```

After implementation, `.runtime/phpt-progress/run-20260614T001003Z-manifest.log`
selected the same 39 runnable rows: 38 passed, 1 failed.

Remaining focused `array_diff*` failure:

```text
ext/standard/tests/array/array_diff_variation9.phpt
```

## Broader Diff/Intersect Evidence

Focused manifest:

```sh
awk '/^ext\/standard\/tests\/array\/array_(diff|intersect)/ {print}' \
  .runtime/phpt-progress/runnable-20260613T233819Z.txt \
  > .runtime/ptn-igxz-array-diff-intersect-after.txt
tools/run-bounded-phpt.sh .runtime/ptn-igxz-array-diff-intersect-after.txt
```

The current run generated
`.runtime/phpt-progress/run-20260614T001551Z-manifest.log` and selected 61
runnable rows: 58 passed, 3 failed, 0 skipped, 0 warned.

Remaining failures:

```text
ext/standard/tests/array/array_diff_variation9.phpt
ext/standard/tests/array/array_intersect_assoc_variation9.phpt
ext/standard/tests/array/array_intersect_variation9.phpt
```

## Implemented Generic Semantics

- Variadic array arguments in set operations now throw catchable `TypeError`
  exceptions instead of terminating the process.
- Variadic array and callback diagnostics omit synthetic parameter names where
  PHP reports an unnamed variadic argument position.
- `array_diff()`, `array_diff_assoc()`, `array_diff_key()`,
  `array_intersect()`, `array_intersect_assoc()`, and `array_intersect_key()`
  accept a single source array and return the PHP-compatible copy/intersection
  result.
- Callback-based set operations validate trailing callbacks before validating
  variadic array operands, preserving PHP diagnostic precedence.
- Malformed array callbacks report the generic callback validation reason
  `array callback must have exactly two members`.
- The lexer accepts leading-dot float literals such as `.5`, `.5e1`, and
  `.5E-1`.
- The internal-function registry keeps `array_filter()` before
  `array_find()`/`array_find_key()` so binary-search lookup remains valid.

## Remaining Blocker

All three remaining rows compare nested arrays in diff/intersect value paths.
PHP emits repeated `Array to string conversion` warnings while completing the
operation. PTN still completes the operation without matching that warning
cadence in these set-operation comparisons.

The next implementation slice should add a shared value-to-comparison-string
helper for array set operations that emits PHP-compatible conversion warnings
for arrays and can be reused by both diff and intersect value comparisons.
