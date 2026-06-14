# PHPT Broad 1k Recursive Dump Frontier: 2026-06-14

Issue: `ptn-iyhh`

This slice refreshes broad 1k evidence and fixes one generic runtime blocker:
recursive object properties in `var_dump()` and `debug_zval_dump()` now share
the existing dump recursion guard used for arrays. The immediate broad impact
is that `Zend/tests/bug35239.phpt` no longer emits unbounded recursive object
output and no longer exhausts php-src `run-tests.php` memory while the broad
1k Zend bucket is running.

## Broad 1k Baseline

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Artifacts:

- Broad manifest:
  `.runtime/phpt-baseline/20260614T043824Z/phpt-baseline-1000.txt`
- Classifier summary:
  `.runtime/phpt-progress/summary-20260614T043824Z.txt`
- Corpus revision:
  `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

The broad 1k classifier selected 1,000 rows, kept 430 runnable, and excluded
570:

| Classification | Rows |
| --- | ---: |
| `runnable` | 430 |
| `unsupported-language` | 281 |
| `unsupported-class-metadata` | 144 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |
| Other classifier buckets | 26 |

## Full-Run Blocker

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000
```

Artifacts:

- Broad manifest:
  `.runtime/phpt-baseline/20260614T044331Z/phpt-baseline-1000.txt`
- Classifier summary:
  `.runtime/phpt-progress/summary-20260614T044331Z.txt`

Before this patch, the full broad run reached the Zend bucket and stopped at
`Zend/tests/bug35239.phpt`:

```text
Fatal error: Allowed memory size of 134217728 bytes exhausted
missing numeric 'Number of tests' count for bucket 'zend'
```

The row builds a recursive `stdClass` graph and expects `var_dump()` to emit
`*RECURSION*` for the repeated object. PTN already tracked recursive arrays in
dump output, but recursive object properties were not tracked, so generated
output grew until the harness process exhausted memory.

## Focused Manifest

Committed manifest:
`tools/phpt-recursive-dump-frontier-manifest.txt`

The broad 1k manifest contains five rows with `*RECURSION*` expectations:

| Row | Broad classification |
| --- | --- |
| `Zend/tests/bug35163.phpt` | runnable |
| `Zend/tests/bug35163_2.phpt` | runnable |
| `Zend/tests/bug35163_3.phpt` | `unsupported-class-metadata` |
| `Zend/tests/bug35239.phpt` | runnable |
| `ext/standard/tests/array/array_map_variation2.phpt` | runnable |

The committed focused manifest includes the four runnable rows.

Focused run after the patch:

```sh
tools/run-bounded-phpt.sh tools/phpt-recursive-dump-frontier-manifest.txt
```

Artifact:
`.runtime/phpt-progress/run-20260614T062119Z-manifest.log`

| Selected | Runnable | Passed | Failed | Excluded | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 3 | 2 | 1 | 1 | 0 | 0 |

## Row Outcomes

| Row | Result after patch | Notes |
| --- | --- | --- |
| `Zend/tests/bug35163.phpt` | pass | Existing array/reference recursion behavior stayed green. |
| `Zend/tests/bug35163_2.phpt` | excluded | Current classifier marks the row `unsupported-language`. |
| `Zend/tests/bug35239.phpt` | pass | Newly fixed recursive object `var_dump()` output. |
| `ext/standard/tests/array/array_map_variation2.phpt` | fail | Remaining `array_map()` reference propagation blocker. |

## Remaining Blockers

The recursive-dump frontier is now small inside the broad 1k tier, but the
corpus has 60 rows with `*RECURSION*` expectations across dump, print, serialize,
GC, closures, weak references, and lazy-object areas. The next credible slices
are separate:

| Rows | Scope | Blocker |
| ---: | --- | --- |
| 1 broad runnable | Array/reference recursion output | `array_map_variation2.phpt` still depends on reference-preserving callback result propagation. |
| 1 broad classified | Array/reference recursion output | `bug35163_2.phpt` is now classified before execution. |
| 1 broad classified | Readonly/property metadata | `bug35163_3.phpt` remains classified because readonly property mutation diagnostics are outside the modeled property subset. |
| 60 corpus | Wider recursion expectations | `print_r()`, `serialize()`, GC, weak references, closures, lazy objects, and debug-info recursion require additional runtime trackers beyond this dump-only patch. |

## Verification

```sh
cargo test compile_var_dump_recursive_object_to_native_binary --test compile_native
tools/run-bounded-phpt.sh tools/phpt-recursive-dump-frontier-manifest.txt
```
