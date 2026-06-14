# PHPT Runtime Diagnostics Blockers: 2026-06-14

Issue: `ptn-lrlt`

Broad baseline source:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-lrlt-before
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-lrlt-after2
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-lrlt-final-after
```

All runs used php-src revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b` and selected the same 1,000-row
manifest shape: 530 Zend rows, 384 ext/standard rows, and 86 core rows.

## Movement

This slice adds source-based classification for runtime diagnostic and assertion
mode APIs that PTN does not yet model generically. It does not classify basic
`assert()` calls or text-only mentions in strings/comments.

| Run | Timestamp | Runnable | Excluded |
| --- | --- | ---: | ---: |
| Before | `20260613T232012Z` | 436 | 564 |
| After | `20260614T000248Z` | 410 | 590 |
| Current-base after rebase | `20260614T002340Z` | 421 | 579 |

Net movement: 26 broad 1k rows newly classified from `runnable` into explicit
blocker buckets. After rebasing across the later SKIPIF and generator/Fiber
classifier work, the same two buckets remain 17 and 9 rows respectively; the
upstream current-base progress note records 447 runnable / 553 excluded before
this slice, and the rebased after-run records 421 runnable / 579 excluded.

New buckets:

| Bucket | Rows | Generic blocker |
| --- | ---: | --- |
| `unsupported-diagnostics-runtime` | 17 | `debug_backtrace()`/`debug_print_backtrace()` frame snapshots, user error/exception handler state, and `ErrorException` severity/trace metadata. |
| `unsupported-assertion-runtime` | 9 | Runtime `zend.assertions` switching, `assert_options()` callback/bail modes, namespace assertion resolution, assertion lvalue interactions, and assertion AST pretty-printing beyond PTN's catchable `AssertionError` subset. |

## Rows

`unsupported-diagnostics-runtime`:

```text
Zend/tests/ErrorException_construct.phpt
Zend/tests/ErrorException_getSeverity.phpt
Zend/tests/backtrace/bug28377.phpt
Zend/tests/backtrace/bug30828.phpt
Zend/tests/backtrace/bug70547.phpt
Zend/tests/backtrace/bug79108.phpt
Zend/tests/backtrace/bug_debug_backtrace.phpt
Zend/tests/backtrace/debug_backtrace_limit.phpt
Zend/tests/backtrace/debug_backtrace_with_include_and_this.phpt
Zend/tests/backtrace/debug_print_backtrace_from_main.phpt
Zend/tests/backtrace/debug_print_backtrace_limit.phpt
Zend/tests/bitwise_not_precision_exception.phpt
Zend/tests/bug29890.phpt
Zend/tests/bug29896.phpt
Zend/tests/bug30998.phpt
Zend/tests/bug35017.phpt
Zend/tests/bug35634.phpt
```

`unsupported-assertion-runtime`:

```text
Zend/tests/assert/bug70528.phpt
Zend/tests/assert/expect_016.phpt
Zend/tests/assert/expect_017.phpt
Zend/tests/assert/expect_018.phpt
Zend/tests/assert/expect_019.phpt
Zend/tests/assert/expect_020.phpt
Zend/tests/assert/gh11580.phpt
Zend/tests/assert/gh16293_001.phpt
Zend/tests/assert/gh16293_002.phpt
```

## Follow-Up Shape

Reopening these rows should be done by implementing shared runtime/compiler
surfaces, not by removing classifier rules one row at a time:

1. Model call-frame snapshots for `debug_backtrace()` and
   `debug_print_backtrace()`, including include/method context, argument
   snapshots, references, limits, and print formatting.
2. Add process-global user error/exception handler state with fallback behavior,
   error suppression interaction, and exception propagation from handlers.
3. Extend built-in exception metadata for `ErrorException` severity and trace
   rendering.
4. Model assertion runtime modes: `ini_set('zend.assertions', ...)`,
   `assert_options()`, callback/bail behavior, namespace resolution, and AST
   pretty-printing for currently unsupported assertion expression forms.

## Verification

```sh
bash -n tools/phpt-classifier.sh
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-lrlt-before
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-lrlt-after2
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-lrlt-final-after
```
