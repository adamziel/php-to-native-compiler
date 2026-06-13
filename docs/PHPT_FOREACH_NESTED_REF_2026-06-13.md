# PHPT Nested Foreach Reference Slice: 2026-06-13

Scope: `ptn-550s.11` rows for live nested by-reference `foreach` mutation,
child-array rekeying through by-reference function parameters, and classified
plain variable-variable unset blockers.

## Runtime Change

- Live `foreach` iterators now remember their current array key and resolve the
  next slot from that key after the loop body mutates the active array. If the
  current key was unset, the iterator continues at the current physical slot so
  shifted successors are not skipped.
- `is_numeric()` is registered through the normal internal-function dispatch
  table and uses PHP-style decimal/exponent numeric-string recognition.
- Plain `unset($$name)` PHPT rows are classified as unsupported language
  surface; PTN still supports modeled dynamic-root array unsets separately.

## Focused Counts

Manifest: `tools/phpt-foreach-nested-ref-manifest.txt`

| Bucket | Rows | Pass | Classified |
| --- | ---: | ---: | ---: |
| nested-live-by-reference-foreach | 1 | 1 | 0 |
| by-reference-child-rekey-through-function | 1 | 1 | 0 |
| classified-dynamic-variable-unset | 1 | 0 | 1 |
| **Total** | **3** | **2** | **1** |

Classified blocker: unsupported plain variable-variable unset in
`Zend/tests/foreach/bug39036.phpt`.

## Evidence

- Before focused run:
  `.runtime/phpt-progress/manifest-20260613T195415Z.txt`
  selected 3 rows, ran 3, excluded 0, passed 0, failed 3.
- After focused manifest run:
  `.runtime/phpt-progress/manifest-20260613T221304Z.txt`
  selected 3 rows, ran 2, excluded 1, passed 2, failed 0.

## Verification

- `cargo test --test compile_native compile_is_numeric_internal_function_to_native_binary -- --nocapture`
- `cargo test --test phpt_classifier phpt_classifier_excludes_currently_unsupported_language_surfaces -- --nocapture`
- `cargo test --test foreach_by_ref_cow -- --nocapture`
- `tools/run-phpt-manifest.sh tools/phpt-foreach-nested-ref-manifest.txt`
- `tools/run-post-merge-cow-gate.sh`
- `tools/run-phpt-manifest.sh tools/phpt-cow-manifest.txt`
- `cargo test`
