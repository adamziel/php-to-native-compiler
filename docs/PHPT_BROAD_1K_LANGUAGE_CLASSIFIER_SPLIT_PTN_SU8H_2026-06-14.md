# PHPT Broad 1k Language Classifier Split: 2026-06-14 ptn-su8h

Issue: `ptn-su8h`

This records the `ptn-su8h` classifier split evidence against the current
classifier taxonomy. The original branch split the broad 1k
`unsupported-language` bucket into semantic buckets, but it used earlier bucket
names such as `unsupported-unpacking`, `unsupported-type-metadata`,
`unsupported-function-local-static`, `unsupported-coroutine-runtime`, and
`unsupported-internal-named-arguments`.

That behavior is already integrated on `master` through the later language
classifier split and follow-up category work. Current bucket names are:

| Current bucket | Rows |
| --- | ---: |
| `unsupported-class-declaration` | 78 |
| `unsupported-call-unpacking` | 34 |
| `unsupported-type-hint` | 14 |
| `unsupported-function-state` | 11 |
| `unsupported-dynamic-symbol` | 8 |
| `unsupported-generator-runtime` | 1 |
| `unsupported-internal-call-binding` | 1 |
| Total split from coarse language bucket | 147 |

The current split is documented in
`PHPT_BROAD_1K_LANGUAGE_CLASSIFIER_SPLIT_PTN_18TP_2026-06-14.md`, with
category follow-ups for class declarations and call unpacking. This note exists
so the older `ptn-su8h` merge request is integrated without reverting to stale
bucket names.

## Validation

Current classifier validation:

```text
cargo test --test phpt_classifier
```

The suite covers the current language split buckets, including
`unsupported-class-declaration`, `unsupported-call-unpacking`,
`unsupported-type-hint`, `unsupported-function-state`,
`unsupported-dynamic-symbol`, `unsupported-generator-runtime`,
`unsupported-expression-diagnostics`, and `unsupported-internal-call-binding`.

The focused call-unpacking replay remains:

```text
.runtime/ptn-fhpx-call-unpacking-current/classification-20260614T102739Z.tsv
```

with 34 selected rows, 0 runnable rows, and all 34 excluded as
`unsupported-call-unpacking`.
