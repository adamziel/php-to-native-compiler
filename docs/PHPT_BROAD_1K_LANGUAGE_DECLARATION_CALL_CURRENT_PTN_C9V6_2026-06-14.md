# PHPT Broad 1k Language Declaration/Call Current Frontier: 2026-06-14 ptn-c9v6

Issue: `ptn-c9v6`

This slice records the current broad 1k language declaration, call expansion,
and dynamic function/type blocker frontier after the class-declaration and
attribute-metadata classifier splits. It is a blocker map and committed union
manifest, not a runtime implementation claim.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-c9v6-baseline-1k
```

Generated broad manifest:

```text
.runtime/ptn-c9v6-baseline-1k/20260614T115234Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/summary-20260614T115234Z.txt
.runtime/phpt-progress/classification-20260614T115234Z.tsv
.runtime/phpt-progress/runnable-20260614T115234Z.txt
.runtime/phpt-progress/excluded-20260614T115234Z.tsv
```

State:

```text
PTN: 98ef0cd0cc71
php-src PHPT corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Largest current classifier buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-attribute-syntax-metadata` | 141 |
| `unsupported-magic-method-metadata` | 69 |
| `unsupported-call-unpacking` | 34 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-trait-declaration` | 25 |
| `unsupported-interface-declaration` | 23 |
| `unsupported-extension` | 20 |
| `unsupported-property-visibility-metadata` | 19 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-anonymous-class` | 15 |
| `unsupported-interface-implementation` | 15 |
| `unsupported-resource-limit-ini` | 15 |
| `unsupported-type-hint` | 14 |
| `sapi-behavior` | 13 |
| `unsupported-typed-property-metadata` | 12 |
| `unsupported-function-state` | 11 |

## Focused Union Manifest

Committed focused manifest:

```text
tools/phpt-language-declaration-call-current-ptn-c9v6-manifest.txt
```

Selection:

```sh
sort \
  .runtime/phpt-progress/excluded-20260614T115234Z/unsupported-call-unpacking.txt \
  .runtime/phpt-progress/excluded-20260614T115234Z/unsupported-trait-declaration.txt \
  .runtime/phpt-progress/excluded-20260614T115234Z/unsupported-interface-declaration.txt \
  .runtime/phpt-progress/excluded-20260614T115234Z/unsupported-interface-implementation.txt \
  .runtime/phpt-progress/excluded-20260614T115234Z/unsupported-anonymous-class.txt \
  .runtime/phpt-progress/excluded-20260614T115234Z/unsupported-type-hint.txt \
  .runtime/phpt-progress/excluded-20260614T115234Z/unsupported-function-state.txt \
  .runtime/phpt-progress/excluded-20260614T115234Z/unsupported-dynamic-symbol.txt \
  .runtime/phpt-progress/excluded-20260614T115234Z/unsupported-generator-runtime.txt \
  .runtime/phpt-progress/excluded-20260614T115234Z/unsupported-internal-call-binding.txt \
  -o tools/phpt-language-declaration-call-current-ptn-c9v6-manifest.txt
```

Focused replay:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-c9v6-language-declaration-call-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-language-declaration-call-current-ptn-c9v6-manifest.txt
```
Focused artifact:

```text
.runtime/ptn-c9v6-language-declaration-call-focused/classification-20260614T115814Z.tsv
```

Focused result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 147 | 0 | 147 |

After rebasing over docs-only broad-slice maps through `3b4d326ab`, the same
focused replay on `7e4303d25708` wrote:

```text
.runtime/ptn-c9v6-language-declaration-call-focused-current/classification-20260614T120257Z.tsv
```

with the same 147 selected, 0 runnable, and 147 excluded result.

## Category Split

| Classification | Rows | Implementation boundary |
| --- | ---: | --- |
| `unsupported-call-unpacking` | 34 | Call-site spread, ordered argument expansion, array unpacking, named/string key rules, and by-reference binding. |
| `unsupported-trait-declaration` | 25 | Trait declarations, composition, aliases, precedence, conflict diagnostics, and class-table integration. |
| `unsupported-interface-declaration` | 23 | Interface declarations, constants, method contracts, inherited interface graphs, and metadata tables. |
| `unsupported-interface-implementation` | 15 | `implements` validation, method compatibility checks, runtime interface metadata, and internal interface contracts. |
| `unsupported-anonymous-class` | 15 | Anonymous class expression lowering, generated names, constructor dispatch, source metadata, and reflection naming. |
| `unsupported-type-hint` | 14 | Nullable and `never` type metadata, coercion, return validation, and control-flow diagnostics. |
| `unsupported-function-state` | 11 | Function-local `static` storage, initializer timing, recursion, references, and shutdown behavior. |
| `unsupported-dynamic-symbol` | 8 | Variable variables and explicit runtime symbol-table lookup/mutation. |
| `unsupported-generator-runtime` | 1 | Generator/yield lowering, suspension frames, cleanup, and by-reference boundaries. |
| `unsupported-internal-call-binding` | 1 | Named-argument binding for modeled internal functions. |

Grouped another way, the frontier is 78 class-like declaration rows, 34
unpacking rows, and 35 dynamic function/type rows.

## Path Concentration

| Path family | Rows |
| --- | ---: |
| `Zend/tests/attributes` | 46 |
| `Zend/tests/anon` | 22 |
| Top-level `Zend/tests` rows | 16 |
| `Zend/tests/arg_unpack` | 14 |
| `Zend/tests/array_unpack` | 13 |
| `ext/standard/tests/array` | 11 |
| `Zend/tests/ArrayAccess` | 10 |
| `Zend/tests/arrow_functions` | 5 |
| `Zend/tests/backtrace` | 5 |
| `Zend/tests/assert` | 2 |
| `Zend/tests/autoload` | 2 |
| `tests/basic` | 1 |

## Relation To Existing Maps

This manifest is a current union crosswalk. It complements the narrower
focused maps for call unpacking, trait declarations, interface declarations and
implementation checks, and function/dynamic type rows. It is useful as a
single broad-slice regression target, while the implementation work should
still land one generic subsystem at a time.

## Blocker Boundary

The 147 rows are above the broad-slice threshold, but implementing them as one
patch is not credible. They require independent parser, semantic-model, IR,
runtime, and diagnostics layers:

1. Class-like declarations need class-table graph support for traits,
   interfaces, anonymous classes, contracts, reflection, and diagnostics.
2. Call and array unpacking need PHP-aware argument expansion with named-key
   validation, by-reference binding, iterable conversion, and evaluation-order
   preservation.
3. Nullable/`never` type hints, function-local statics, and variable variables
   need shared type metadata, call-frame state, and dynamic symbol-table
   boundaries.
4. Generator runtime and internal named-argument binding are small row counts,
   but they belong to large generic services and should not be handled as
   exact PHPT-row patches.

Until those layers exist, keeping this union classified avoids turning one
known language frontier into noisy parser/runtime failures.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-c9v6-baseline-1k
PHPT_PROGRESS_DIR=.runtime/ptn-c9v6-language-declaration-call-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-language-declaration-call-current-ptn-c9v6-manifest.txt
```
