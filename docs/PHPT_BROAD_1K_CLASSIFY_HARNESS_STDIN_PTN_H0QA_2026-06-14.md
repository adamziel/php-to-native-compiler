# Broad 1k PHPT Classify Harness Map (`ptn-h0qa`)

Generated: 2026-06-14T11:48Z.

## Scope

`ptn-h0qa` asked for a broad 1k PHPT cluster slice. I started from the
current broad 1k baseline tooling and found the credible generic movement was
in the classifier harness itself: classify-only runs could remain open on the
manifest stream before emitting a complete summary. The implementation keeps
row classification isolated from harness stdin and reads manifest files
directly.

This is not a compiler semantic pass-count change. It is a blocker-map
deliverable plus a harness fix that makes the 1k blocker map reproducible.

## Commands

```bash
PHPT_PROGRESS_DIR=.runtime/ptn-h0qa-progress-rebased \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-h0qa-baseline-rebased
PHPT_PROGRESS_DIR=.runtime/ptn-h0qa-progress-final \
  tools/run-bounded-phpt.sh --classify-harness-programs --classify-only \
  .runtime/ptn-h0qa-baseline-rebased/20260614T114112Z/phpt-baseline-1000.txt
cargo fmt --check
cargo test --test phpt_classifier
```

The first broad wrapper was capped during investigation, then the focused
`run-bounded-phpt.sh` classify-only command completed and produced the evidence
below.

## Evidence

Final completed broad 1k classify-only run:

- Corpus: `/home/claude/php-src-phpt`
- Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`
- Manifest: `.runtime/ptn-h0qa-baseline-rebased/20260614T114112Z/phpt-baseline-1000.txt`
- Classification summary:
  `.runtime/ptn-h0qa-progress-final/summary-20260614T114831Z.txt`
- Selected rows: 1000
- Runnable rows: 424
- Classified/excluded rows: 576
- Runnable bucket split: `Zend/tests` 114, `ext/standard/tests` 294,
  `tests` 16
- `run-tests-exit`: 0; no PHPT rows executed in classify-only mode

Earlier investigation reproduced the failure mode with partial artifacts:
`.runtime/phpt-progress/manifest-20260614T111735Z.txt` reached 709 manifest
rows while `.runtime/phpt-progress/classification-20260614T111735Z.tsv`
stopped at 708 classified rows before the wrapper was terminated. A later
timed run with the harness patch reached 764 classified rows before the local
240s cap expired, confirming forward progress rather than a row-specific
classifier decision.

## Blocker Counts

| Category | Rows |
|---|---:|
| runnable | 424 |
| unsupported-attribute-syntax-metadata | 141 |
| unsupported-object-string-conversion-metadata | 61 |
| unsupported-magic-method-metadata | 8 |
| unsupported-call-unpacking | 34 |
| unsupported-request-input-ini | 28 |
| unsupported-trait-declaration | 25 |
| unsupported-interface-declaration | 23 |
| unsupported-extension | 20 |
| unsupported-property-visibility-metadata | 19 |
| unsupported-diagnostics-runtime | 17 |
| unsupported-assertion-ini | 17 |
| unsupported-resource-limit-ini | 15 |
| unsupported-interface-implementation | 15 |
| unsupported-anonymous-class | 15 |
| unsupported-type-hint | 14 |
| sapi-behavior | 13 |
| unsupported-typed-property-metadata | 12 |
| unsupported-function-state | 11 |
| unsupported-class-contract-metadata | 9 |
| unsupported-autoload-metadata | 9 |
| unsupported-assertion-runtime | 9 |
| unsupported-internal-attribute-metadata | 8 |
| unsupported-dynamic-symbol | 8 |
| unsupported-readonly-property-metadata | 7 |
| unsupported-method-visibility-metadata | 7 |
| unsupported-diagnostics-ini | 5 |
| harness-cleanup | 4 |
| unsupported-internal-reflection-metadata | 3 |
| process-boundary | 3 |
| unsupported-scalar-format-ini | 2 |
| unsupported-opcache-ini | 2 |
| unsupported-host-path-ini | 2 |
| unsupported-function-disable-ini | 2 |
| skipif-precondition | 2 |
| unsupported-resource-limit | 1 |
| unsupported-internal-call-binding | 1 |
| unsupported-internal | 1 |
| unsupported-generator-runtime | 1 |
| external-service | 1 |
| environment-assumption | 1 |

## Cluster Decision

No current single compiler semantic implementation cluster looked credible for
a quick generic >=25-row pass movement:

- Class/object metadata remains the dominant blocker family at 362 rows when
  attribute, object-string conversion, residual magic method, trait/interface,
  anonymous-class, visibility, typed/readonly property, autoload, and
  reflection metadata categories are combined. That is an architecture slice,
  not a narrow safe patch.
- Call unpacking has 34 broad rows, but it crosses parser, lowering, call ABI,
  array unpacking, named argument ordering, and diagnostics.
- Request/SAPI/runtime configuration buckets are harness or process-boundary
  surfaces rather than native compiler semantics.

The committed behavior change therefore targets the PHPT baseline tooling:
classify-only can now complete the broad 1k manifest and emit a complete
blocker map for later semantic work.
