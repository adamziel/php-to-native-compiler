# PHPT Broad 1k Object String Conversion Reconciliation: 2026-06-14 ptn-i47w

Issue: `ptn-i47w`

This note reconciles the older object-string classifier split with the current
post-`ptn-i0p3` bucket name. The active classifier category is:

```text
unsupported-object-string-conversion-metadata
```

The older `ptn-i47w` branch used `unsupported-object-string-conversion` while
the integrated split uses the more explicit metadata suffix. The PHPT row set
is the same 61-row `__toString()` object conversion frontier, and the canonical
committed manifest is:

```text
tools/phpt-object-string-conversion-metadata-ptn-i0p3-manifest.txt
```

## Current Evidence

Focused replay on the current integration branch:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-i47w-object-string-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-object-string-conversion-metadata-ptn-i0p3-manifest.txt
```

Expected current result:

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 61 | 0 | 61 | `unsupported-object-string-conversion-metadata` |

The broader 69-row magic-method focused manifest now splits into:

| Bucket | Rows |
| --- | ---: |
| `unsupported-object-string-conversion-metadata` | 61 |
| `unsupported-magic-method-metadata` | 8 |

## Boundary

These rows are still blockers, not runtime support claims. Generic support
requires class metadata for `__toString()`, shared boxed-value object-to-string
conversion, warning/exception behavior, recursion guards, loose comparison and
array-helper integration, and diagnostic/backtrace metadata for magic dispatch.

Keeping the 61 rows in a dedicated metadata bucket prevents broad PHPT telemetry
from mixing object conversion work into residual magic-method metadata.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
PHPT_PROGRESS_DIR=.runtime/ptn-i47w-object-string-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-object-string-conversion-metadata-ptn-i0p3-manifest.txt
```
