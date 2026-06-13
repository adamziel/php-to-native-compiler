# Array-Internal COW Blockers: 2026-06-13

Evidence base: `ptn-f0rp` on the current branch at 2026-06-13T22:09Z. This
slice keeps the broad ext/standard frontier manifest and narrows the generic
PHPT classifier evidence after modeling `array_splice()` and
`array_walk_recursive()`.

## Frontier Counts

`tools/phpt-array-internal-cow-frontier-manifest.txt` selects 72 rows:

| Bucket | Rows |
| --- | ---: |
| array-splice | 12 |
| recursive-walk | 14 |
| array-multisort | 20 |
| user-comparator-sort | 26 |
| **Total** | **72** |

Before `ptn-f0rp`, the same row set classified as 72 selected, 0 runnable, and
72 excluded. Current-branch classify-only output is 72 selected, 17 runnable,
and 55 excluded:

| Category | Rows |
| --- | ---: |
| unsupported-internal | 38 |
| unsupported-language | 8 |
| unsupported-class-metadata | 9 |

## Blocker Scope

The remaining `unsupported-internal` rows are source-evidence blockers for
helper families that require generic runtime work before they should be
measured as semantic failures:

- Destructor-reentrant `array_splice()` rows that mutate global array state
  from `__destruct()` while splice is destroying removed values.
- `array_multisort()` coordinated multi-array by-reference sorting, flags, and
  cursor mutation.
- `usort()`, `uasort()`, and `uksort()` user-comparator sort helpers with
  by-reference array mutation and COW separation.

Newly runnable modeled rows:

- `ext/standard/tests/array/array_splice_basic.phpt`
- `ext/standard/tests/array/array_walk/array_walk_recursive_basic1.phpt`

Representative remaining blocker rows:

- `ext/standard/tests/array/gh16649/array_splice_uaf_add_elements.phpt`
- `ext/standard/tests/array/sort/array_multisort_basic1.phpt`
- `ext/standard/tests/array/sort/usort_basic.phpt`
- `ext/standard/tests/array/sort/uasort_basic1.phpt`

## Verification

- Current classify-only measurement:
  `tools/run-bounded-phpt.sh --classify-only tools/phpt-array-internal-cow-frontier-manifest.txt`
  reported 72 selected, 17 runnable, and 55 excluded.
- Current bounded measurement:
  `tools/run-bounded-phpt.sh tools/phpt-array-internal-cow-frontier-manifest.txt`
  reported 17/17 runnable rows passing: `array-splice` 5/5 and
  `recursive-walk` 12/12.
- Classifier unit tests cover the modeled helper runnable cases, remaining
  mutating-internal source detection, and the string/comment false-positive
  guard.
- Focused native verification covers shared-reference replacement,
  `array_splice()`, and `array_walk_recursive()` native binaries.
