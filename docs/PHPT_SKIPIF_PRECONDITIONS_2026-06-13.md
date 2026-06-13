# PHPT Static SKIPIF Preconditions, 2026-06-13

Broad baseline source:

```bash
rm -rf .runtime/phpt-awta-before
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/phpt-awta-before
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/phpt-awta-before/20260613T221005Z/phpt-baseline-5000.txt

rm -rf .runtime/phpt-awta-after
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/phpt-awta-after
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/phpt-awta-after/20260613T222454Z/phpt-baseline-5000.txt
```

Both runs used php-src PHPT corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

## Modeled SKIPIF Predicates

The classifier now statically models only these `--SKIPIF--` preconditions:

- Sanitizer/environment opt-out gates using literal
  `getenv('SKIP_ASAN')`, `getenv('SKIP_MSAN')`, `getenv('SKIP_UBSAN')`,
  or `getenv('SKIP_PERF_SENSITIVE')` checks.
- Literal `PHP_INT_SIZE` comparisons with `===`, `!==`, `==`, `!=`, `<`,
  `<=`, `>`, or `>=` against an integer constant.
- Host locale availability guards using `setlocale(LC_ALL, ...)`, including
  the `setlocale(LC_ALL, 'invalid') === 'invalid'` sanity guard.

If a modeled condition says the row should skip on this host, the row is
classified as `skipif-precondition`. If all modeled conditions are satisfied,
the row continues through normal semantic blockers and can become `runnable`.
Any SKIPIF body outside these whitelisted forms remains excluded as
`harness-skipif`; the classifier still does not execute arbitrary harness PHP.

## Broad Count Movement

| Manifest | Before runnable | After runnable | Before harness-skipif | After harness-skipif | After skipif-precondition |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1k | 434 | 438 | 6 | 0 | 2 |
| 5k | 2,047 | 2,070 | 316 | 275 | 27 |

1k transitions:

- 4 rows moved from `harness-skipif` to `runnable`.
- 2 rows moved from `harness-skipif` to `skipif-precondition`.

5k transitions:

- 23 rows moved from `harness-skipif` to `runnable`.
- 18 rows moved from `harness-skipif` to `skipif-precondition`.
- 7 rows moved from `harness-cleanup` to `skipif-precondition` because the
  static 32-bit platform precondition dominates the cleanup blocker.
- 2 rows moved from `unsupported-class-metadata` to `skipif-precondition` for
  the same 32-bit platform reason.

Representative newly runnable rows:

- `Zend/tests/arginfo_zpp_mismatch.phpt`
- `Zend/tests/arginfo_zpp_mismatch_strict.phpt`
- `Zend/tests/binary.phpt`
- `Zend/tests/comparison/compare_001_64bit.phpt`
- `ext/standard/tests/array/array_fill_error2.phpt`
- `ext/standard/tests/array/range/range_variation1_64bit.phpt`

Representative modeled precondition exclusions:

- `Zend/tests/binary-32bit.phpt`
- `Zend/tests/comparison/compare_001.phpt`
- `Zend/tests/double_to_string.phpt`
- `ext/standard/tests/array/range/bug41121.phpt`
- `ext/standard/tests/file/fscanf_variation39.phpt`
- `tests/basic/consistent_float_string_casts.phpt`

Representative rows intentionally still excluded as arbitrary SKIPIF harnesses:

- `Zend/tests/bug54547.phpt`
- `Zend/tests/closures/gh12073.phpt`
- `ext/standard/tests/array/sort/locale_sort.phpt`
- `ext/standard/tests/file/tempnam_variation1.phpt`
- `tests/basic/timeout_variation_0.phpt`

## Validation

```bash
bash -n tools/phpt-classifier.sh
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/phpt-awta-after
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/phpt-awta-after/20260613T222454Z/phpt-baseline-5000.txt
```
