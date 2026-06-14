# PHPT Broad 1k Assertion Runtime Frontier: 2026-06-14

Issue: `ptn-7lsl`

This slice used the broad PHPT 1k baseline on current `origin/master` and
focused the `Zend/tests/assert` cluster. The result is a blocker map rather
than an implementation patch: the only runnable assertion rows in this broad
slice already pass, while the remaining rows require assertion INI/runtime
state, disabled-function registry mutation, static locals, or generator
lowering.

The php-src corpus was `/home/claude/php-src-phpt` at revision:

```text
8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-7lsl-before
```

Generated broad manifest:

```text
.runtime/ptn-7lsl-before/20260614T054022Z/phpt-baseline-1000.txt
```

Broad classifier summary:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 429 | 571 |

Top broad categories relevant to this slice:

| Category | Rows |
| --- | ---: |
| `unsupported-assertion-ini` | 17 |
| `unsupported-assertion-runtime` | 9 |
| `unsupported-function-disable-ini` | 2 |

## Focused Manifest

The focused manifest is checked in as:

```text
tools/phpt-assertion-runtime-frontier-manifest.txt
```

It was derived from the generated broad 1k tier with:

```sh
rg '^Zend/tests/assert/' \
  .runtime/ptn-7lsl-before/20260614T054022Z/phpt-baseline-1000.txt \
  > .runtime/ptn-7lsl/assert-broad1k-manifest.txt
```

## Focused Classifier Evidence

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-7lsl/assert-classify-before \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-assertion-runtime-frontier-manifest.txt
```

Result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 29 | 2 | 27 |

Focused categories:

| Category | Rows | Blocker |
| --- | ---: | --- |
| `unsupported-assertion-ini` | 15 | Configurable `assert.exception` behavior and assertion mode switching are outside PTN's current catchable `AssertionError` subset. |
| `unsupported-assertion-runtime` | 9 | Runtime `zend.assertions`, `assert_options()` callback/mode state, namespace-aware assert resolution, assertion AST rendering, and assertion lvalue-mode interactions are not modeled. |
| `unsupported-language` | 2 | One row requires generator/yield lowering; one row requires static local variables. |
| `unsupported-function-disable-ini` | 1 | `disable_functions` mutates the runtime function table, while PTN currently emits a fixed function registry. |

Runnable rows:

```text
Zend/tests/assert/bug70293.phpt
Zend/tests/assert/expect_empty_stmt_bug.phpt
```

## Focused Native Evidence

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-7lsl/assert-run-before \
  timeout 300 tools/run-bounded-phpt.sh \
  tools/phpt-assertion-runtime-frontier-manifest.txt
```

Result:

| Selected | Runnable | Excluded | Passed | Failed | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 29 | 2 | 27 | 2 | 0 | 0 | 0 |

The runnable rows are already green on the native PHPT path, so reopening the
remaining 27 rows requires new generic semantics rather than row-level fixes.

## Implementation Boundary

No credible one-patch implementation move reaches the 25-row target in this
cluster:

- The largest subcluster is `assert.exception` / assertion INI mode behavior
  with 15 rows. That needs runtime assertion configuration and diagnostics
  rather than a parser or one-helper change.
- Runtime assertion state accounts for 9 rows. The required surface spans
  `zend.assertions`, `assert_options()` callbacks, namespace-aware assertion
  function resolution, assertion expression rendering, and lvalue-mode
  interactions.
- The residual language blockers are unrelated to assertion runtime state:
  generators/yield and static locals.
- The 2 runnable rows already pass.

Recommended next implementation order:

1. Add a shared runtime assertion options/state object covering
   `zend.assertions`, `assert.exception`, and `assert_options()`.
2. Thread assertion diagnostics through that state, including namespace-aware
   `assert()` resolution and expression rendering.
3. Add fixed function-registry mutation only when the compiler has a generic
   disabled-function model, not as an assertion-row special case.
4. Keep generator/yield and static-local rows in their existing language
   blocker buckets until those broader features exist.

## Verification

```sh
cargo fmt --check
PHPT_PROGRESS_DIR=.runtime/ptn-7lsl/assert-classify-before \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-assertion-runtime-frontier-manifest.txt
PHPT_PROGRESS_DIR=.runtime/ptn-7lsl/assert-run-before \
  timeout 300 tools/run-bounded-phpt.sh \
  tools/phpt-assertion-runtime-frontier-manifest.txt
```
