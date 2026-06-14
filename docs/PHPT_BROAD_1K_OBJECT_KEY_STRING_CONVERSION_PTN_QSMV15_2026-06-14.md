# PHPT Broad 1k Object Key String Conversion: 2026-06-14 ptn-qsmv.15

Issue: `ptn-qsmv.15`

This slice reopens the 61 broad `__toString()` object-conversion rows by
making shared boxed string conversion runtime-aware, then lands a focused
25-row green pack around array key/value helpers and the Zend `define()`
object-key case.

## Semantic Change

- `ptn_value_to_string_operand_with_runtime()` now dispatches public
  `__toString()` for objects and throws a catchable `Error` with the class
  name when an object has no string conversion.
- Key-producing helpers such as `array_fill_keys()` and `array_combine()` use
  the runtime-aware conversion path for object key values.
- User array literals keep source-line metadata for key diagnostics, allowing
  resource keys to warn with the resource id before casting to integer.
- The broad classifier no longer excludes every `function __toString()` row as
  `unsupported-object-string-conversion-metadata`; residual non-`__toString`
  magic hooks remain classified as `unsupported-magic-method-metadata`.

## Broad 1k Movement

Original hook baseline on the assigned branch:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-qsmv15-before-baseline
```

Result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 424 | 576 |

After rebasing onto current `origin/master`, the upstream dashboard recorded
`ptn-qsmv.17` at 440 runnable / 560 classified. The rebased qsmv.15 branch
then measured:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-qsmv15-final-after-baseline
```

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 501 | 499 |

`unsupported-magic-method-metadata` remains at 8 rows. The former
`unsupported-object-string-conversion-metadata` bucket is gone from the broad
1k classifier output.

Full rebased broad execution:

```sh
tools/run-phpt-baseline.sh --tier 1000 \
  --out-dir .runtime/ptn-qsmv15-rebased-after-baseline-full
```

Summary from `.runtime/phpt-progress/summary-20260614T143639Z.txt`:

| Bucket | Runnable | Passed | Failed |
| --- | ---: | ---: | ---: |
| Zend | 132 | 72 | 60 |
| standard | 353 | 285 | 68 |
| core | 16 | 8 | 8 |
| total | 501 | 365 | 136 |

## Focused Row Pack

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-qsmv15-rebased-focused-object-key-pack \
  tools/run-bounded-phpt.sh \
  tools/phpt-object-key-string-conversion-ptn-qsmv15-manifest.txt
```

Final post-rebase focused sweep:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-qsmv15-final-focused-object-key-pack \
  tools/run-bounded-phpt.sh \
  tools/phpt-object-key-string-conversion-ptn-qsmv15-manifest.txt
```

Result:

| Selected | Runnable | Passed | Failed |
| ---: | ---: | ---: | ---: |
| 25 | 25 | 25 | 0 |

Newly passing rows:

```text
Zend/tests/bug37811.phpt
ext/standard/tests/array/array_combine_variation4.phpt
ext/standard/tests/array/array_combine_variation5.phpt
ext/standard/tests/array/array_diff_assoc_variation3.phpt
ext/standard/tests/array/array_diff_variation8.phpt
ext/standard/tests/array/array_fill_keys_variation1.phpt
ext/standard/tests/array/array_fill_keys_variation2.phpt
ext/standard/tests/array/array_fill_keys_variation4.phpt
ext/standard/tests/array/array_fill_variation3.phpt
ext/standard/tests/array/array_intersect_assoc_variation7.phpt
ext/standard/tests/array/array_intersect_assoc_variation8.phpt
ext/standard/tests/array/array_intersect_variation7.phpt
ext/standard/tests/array/array_intersect_variation8.phpt
ext/standard/tests/array/array_key_exists_variation1.phpt
ext/standard/tests/array/array_map_variation4.phpt
ext/standard/tests/array/array_map_variation5.phpt
ext/standard/tests/array/array_merge_recursive_variation4.phpt
ext/standard/tests/array/array_merge_recursive_variation5.phpt
ext/standard/tests/array/array_merge_variation3.phpt
ext/standard/tests/array/array_pad_variation3.phpt
ext/standard/tests/array/array_push_variation2.phpt
ext/standard/tests/array/array_reverse_variation3.phpt
ext/standard/tests/array/array_reverse_variation4.phpt
ext/standard/tests/array/array_reverse_variation5.phpt
ext/standard/tests/array/array_shift_variation2.phpt
```

All 25 rows also passed during the rebased full broad execution.

## Verification

```sh
bash -n tools/phpt-classifier.sh
cargo fmt --check
cargo test parser_accepts_array_literals_and_spaceship_expressions -- --nocapture
cargo test --test phpt_classifier -- --nocapture
cargo test --test compile_native compile_array_key_string_conversion_to_native_binary -- --nocapture
cargo test --test compile_native compile_object_to_string_conversion_to_native_binary -- --nocapture
PHPT_PROGRESS_DIR=.runtime/ptn-qsmv15-final-focused-object-key-pack \
  tools/run-bounded-phpt.sh \
  tools/phpt-object-key-string-conversion-ptn-qsmv15-manifest.txt
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-qsmv15-final-after-baseline
tools/run-phpt-baseline.sh --tier 1000 \
  --out-dir .runtime/ptn-qsmv15-rebased-after-baseline-full
```
