# PHPT Broad 1k Unsupported-Language Frontier: 2026-06-14

Issue: `ptn-wzwq`

This slice refreshes the broad PHPT 1k classifier on current `origin/master`
and maps the full `unsupported-language` bucket. It is a blocker map, not a
support claim. The selected rows cross parser, AST, class graph, metadata,
call lowering, local-storage, generator, and dynamic-symbol semantics, so
reopening the whole bucket is not credible as one implementation patch.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-wzwq-baseline-before
```

Generated manifest:
`.runtime/ptn-wzwq-baseline-before/20260614T062648Z/phpt-baseline-1000.txt`

Classification artifact:
`.runtime/phpt-progress/classification-20260614T062648Z.tsv`

PTN commit: `a8856615c283`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 425 | 575 |

Top blocker buckets:

| Bucket | Rows |
| --- | ---: |
| `unsupported-language` | 288 |
| `unsupported-class-metadata` | 143 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-diagnostics-runtime` | 16 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |

## Focused Unsupported-Language Slice

Selection from the broad classifier:

```sh
awk -F'\t' '$2=="unsupported-language" {print $1}' \
  .runtime/phpt-progress/classification-20260614T062648Z.tsv \
  > .runtime/ptn-wzwq/unsupported-language-frontier.txt
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-wzwq/unsupported-language-frontier.txt
```

Focused artifact:
`.runtime/phpt-progress/classification-20260614T063549Z.tsv`

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 288 | 0 | 288 | `unsupported-language` |

Unsupported-language reasons:

| Reason | Rows |
| --- | ---: |
| PHP attribute syntax and reflection metadata | 141 |
| Call-site or array unpacking (`...`) | 34 |
| Trait declarations | 25 |
| Interface declarations | 23 |
| Interface implementation checks | 15 |
| Anonymous class syntax (`new class`) | 15 |
| Nullable type hints (`?T`) | 14 |
| Static local variables | 11 |
| Variable variables and runtime symbol-table lookup | 8 |
| Named-argument binding for modeled array internals | 1 |
| Generator/yield lowering | 1 |

Path concentration:

| Path family | Rows |
| --- | ---: |
| `Zend/tests/attributes` | 187 |
| Other `Zend/tests` rows | 39 |
| `Zend/tests/anon` | 22 |
| `Zend/tests/arg_unpack` | 14 |
| `Zend/tests/array_unpack` | 14 |
| `ext/standard/tests/array` | 11 |
| `tests/basic` | 1 |

## Representative Rows

```text
Zend/tests/ArrayAccess/ArrayAccess_indirect_append.phpt
Zend/tests/anon/001.phpt
Zend/tests/anon/gh16067.phpt
Zend/tests/arg_unpack/basic.phpt
Zend/tests/arg_unpack/traversable_with_by_ref_parameters.phpt
Zend/tests/array_unpack/basic.phpt
Zend/tests/array_unpack/in_destructuring.phpt
Zend/tests/arrow_functions/003.phpt
Zend/tests/attributes/001_placement.phpt
Zend/tests/attributes/026_unpack_in_args.phpt
Zend/tests/attributes/override/020.phpt
Zend/tests/autoload/bug49908.phpt
Zend/tests/backtrace/bug69180-backtrace.phpt
ext/standard/tests/array/array_filter_invalid_mode.phpt
ext/standard/tests/array/array_fill_object.phpt
tests/basic/bug73969.phpt
```

## Why This Is A Blocker

The rows share a language/modeling boundary rather than one local runtime
helper:

- Attributes need parser support, declaration attachment, constant-expression
  argument evaluation, built-in attribute validation, metadata storage, and
  ReflectionAttribute APIs.
- Unpacking needs AST/IR nodes for spread arguments and array elements, call
  argument vector expansion, named/positional argument diagnostics,
  by-reference binding, traversable iteration, and array literal merge rules.
- Interfaces, traits, anonymous classes, and ArrayAccess rows need a fuller
  class graph, trait composition, interface contract validation, anonymous
  class naming/metadata, built-in interface behavior, and reflection exposure.
- Nullable type hints require type metadata plus coercion and diagnostics across
  parameters, return values, properties, and class constants.
- Static locals require persistent per-function storage that is distinct from
  globals and ordinary call-frame locals, including reference and default-value
  behavior.
- Variable variables need dynamic symbol-table lookup/mutation and explicit
  fallback boundaries.
- Generator/yield support needs suspension frames, iterator state, send/throw
  boundaries, return values, cleanup, and by-reference diagnostics.

Until those surfaces land generically, keeping these 288 rows classified gives
the broad baseline stable evidence instead of noisy parser/runtime failures.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-wzwq-baseline-before
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-wzwq/unsupported-language-frontier.txt
tools/run-bounded-phpt.sh tools/phpt-zend-operator-control-frontier-manifest.txt
```

Observed focused operator/control context:

| Manifest | Selected | Runnable | Passed | Failed | Excluded |
| --- | ---: | ---: | ---: | ---: | ---: |
| `tools/phpt-zend-operator-control-frontier-manifest.txt` | 26 | 25 | 15 | 10 | 1 |

The operator/control run confirms the current residual failure set, but its
largest coherent subgroups are below the 25-row broad-slice target. The
288-row unsupported-language map is the actionable high-yield blocker for this
slice.
