# PHPT Broad 1k Class Metadata Split: 2026-06-14

Issue: `ptn-0fmh`

This slice refines the broad 1k class/object metadata blocker classification on
`origin/master`. It does not claim new class runtime support. The goal is to
move the previous coarse `unsupported-class-metadata` bucket into actionable
generic metadata families so implementation work can target the next largest
semantic boundary.

## Broad 1k Evidence

Before command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-0fmh-baseline-current
```

Generated broad manifest:
`.runtime/ptn-0fmh-baseline-current/20260614T072606Z/phpt-baseline-1000.txt`

Before classification artifact:
`.runtime/phpt-progress/classification-20260614T072606Z.tsv`

After command, using the same manifest:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-0fmh-baseline-current/20260614T072606Z/phpt-baseline-1000.txt
```

After classification artifact:
`.runtime/phpt-progress/classification-20260614T081126Z.tsv`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Run | Selected | Runnable | Excluded | Coarse class metadata rows |
| --- | ---: | ---: | ---: | ---: |
| Before | 1000 | 425 | 575 | 143 |
| After | 1000 | 425 | 575 | 0 |

The 143 rows are still excluded, but they now have precise metadata categories
instead of a single coarse bucket.

## Focused Manifest

Committed manifest:
`tools/phpt-broad-class-metadata-split-manifest.txt`

Focused evidence command:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-broad-class-metadata-split-manifest.txt
```

Focused summary:
`.runtime/ptn-0fmh-focused-current/summary-20260614T101325Z.txt`

Focused classification artifact:
`.runtime/ptn-0fmh-focused-current/classification-20260614T101325Z.tsv`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 143 | 0 | 143 |

## Split Counts

| Category | Rows | Zend | ext/standard |
| --- | ---: | ---: | ---: |
| `unsupported-magic-method-metadata` | 69 | 9 | 60 |
| `unsupported-property-visibility-metadata` | 19 | 10 | 9 |
| `unsupported-typed-property-metadata` | 12 | 12 | 0 |
| `unsupported-autoload-metadata` | 9 | 9 | 0 |
| `unsupported-class-contract-metadata` | 9 | 9 | 0 |
| `unsupported-attribute-metadata` | 8 | 8 | 0 |
| `unsupported-method-visibility-metadata` | 7 | 6 | 1 |
| `unsupported-readonly-property-metadata` | 7 | 7 | 0 |
| `unsupported-internal-reflection-metadata` | 3 | 3 | 0 |

## Blocker Map

The largest next implementation target is
`unsupported-magic-method-metadata`. It covers `__get`, `__set`, `__isset`,
`__unset`, `__debugInfo`, `__toString`, and related object conversion/reflection
effects. The broad rows are not isolated class tests: 60 of the 69 rows are
standard array-helper rows where object conversion, property access, callback
validation, or dump/stringification reaches magic dispatch.

The visibility and typed-property groups should remain separate. They need
declared slot lookup across public/protected/private scopes, inherited-private
slot separation, typed-property initialization/coercion/error paths, and
diagnostics for static and readonly properties.

Autoload and internal reflection are not local parser gaps. They require
runtime class-table mutation and a more complete internal metadata registry.

## Verification

```sh
cargo test --test phpt_classifier
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-broad-class-metadata-split-manifest.txt
```

`cargo fmt --check` should be run before submission with the rest of the gate
commands.
