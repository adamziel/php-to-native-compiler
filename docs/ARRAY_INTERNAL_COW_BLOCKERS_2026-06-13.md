# Array-Internal COW Blockers: 2026-06-13

Evidence base: `ptn-550s.3` on `origin/master@8d8b61cc2`, refreshed at
2026-06-13T18:50Z. This slice adds a broad ext/standard frontier manifest and
generic PHPT classifier evidence for mutating array-internal helper families
that PTN does not model yet.

## Frontier Counts

`tools/phpt-array-internal-cow-frontier-manifest.txt` selects 72 rows:

| Bucket | Rows |
| --- | ---: |
| array-splice | 12 |
| recursive-walk | 14 |
| array-multisort | 20 |
| user-comparator-sort | 26 |
| **Total** | **72** |

Before this classifier change, the same row set classified as 58 runnable and
14 excluded. Current-branch classify-only output is 72 selected, 0 runnable,
and 72 excluded:

| Category | Rows |
| --- | ---: |
| unsupported-internal | 58 |
| unsupported-language | 9 |
| unsupported-class-metadata | 5 |

## Blocker Scope

The new `unsupported-internal` rows are source-evidence blockers for helper
families that require generic runtime work before they should be measured as
semantic failures:

- `array_splice()` by-reference array mutation, replacement, reindexing, and
  destructor-sensitive COW separation.
- `array_walk_recursive()` recursive by-reference callback traversal and
  mutation visibility.
- `array_multisort()` coordinated multi-array by-reference sorting, flags, and
  cursor mutation.
- `usort()`, `uasort()`, and `uksort()` user-comparator sort helpers with
  by-reference array mutation and COW separation.

Representative newly classified rows:

- `ext/standard/tests/array/array_splice_basic.phpt`
- `ext/standard/tests/array/array_walk/array_walk_recursive_basic1.phpt`
- `ext/standard/tests/array/sort/array_multisort_basic1.phpt`
- `ext/standard/tests/array/sort/usort_basic.phpt`
- `ext/standard/tests/array/sort/uasort_basic1.phpt`

## Verification

- Before measurement:
  `tools/run-bounded-phpt.sh --classify-only .runtime/phpt-frontier/array-internal-cow-unsupported-before.txt`
  reported 72 selected, 58 runnable, and 14 excluded.
- Current measurement:
  `tools/run-bounded-phpt.sh --classify-only tools/phpt-array-internal-cow-frontier-manifest.txt`
  reported 72 selected, 0 runnable, and 72 excluded.
- Classifier unit tests cover the new `unsupported-internal` source detection
  and the string/comment false-positive guard.
- Focused native/COW verification also passed: `cargo fmt --check`,
  `cargo test --test phpt_classifier`, `cargo test --test cow_native_reducers`,
  and `tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt` at 29/29.
