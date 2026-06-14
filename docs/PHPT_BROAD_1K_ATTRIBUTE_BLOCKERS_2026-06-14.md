# Broad PHPT 1k Attribute Blocker Map: 2026-06-14

Issue: `ptn-oz24`

This slice used the broad PHPT baseline tooling on `origin/master` and selected
the PHP attribute cluster as the highest-yield semantic blocker. This is a
blocker map, not a support claim: full attribute support crosses parser,
metadata, validation, and Reflection surfaces and is not credible as a narrow
row-local fix.

## Evidence

Source state:

- PTN: `902454c7893e`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Commands:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-oz24-baseline
```

Generated broad manifest:

```text
.runtime/ptn-oz24-baseline/20260614T002319Z/phpt-baseline-1000.txt
```

Classify-only result:

| Measurement | Selected | Runnable | Excluded |
| --- | ---: | ---: | ---: |
| broad 1k classify-only | 1000 | 447 | 553 |

Top excluded clusters:

| Cluster | Rows |
| --- | ---: |
| PHP attributes | 141 |
| heredoc/nowdoc syntax | 70 |
| call-site/array unpacking | 34 |
| traits | 25 |
| interfaces | 23 |
| magic method metadata | 20 |
| non-public property metadata | 19 |

The PHP attribute cluster is the largest single semantic blocker in this broad
1k slice. All 141 attribute rows are classified by source syntax, not by PHPT
path.

## Attribute Sub-Buckets

| Attribute sub-bucket | Rows |
| --- | ---: |
| root attribute rows | 35 |
| `deprecated/` | 34 |
| `override/` | 18 |
| `delayed_target_validation/` | 18 |
| `constants/` | 17 |
| `nodiscard/` | 15 |
| `Attribute/` | 4 |

Representative rows:

```text
Zend/tests/attributes/001_placement.phpt
Zend/tests/attributes/004_name_resolution.phpt
Zend/tests/attributes/006_filter.phpt
Zend/tests/attributes/011_inheritance.phpt
Zend/tests/attributes/020_userland_attribute_validation.phpt
Zend/tests/attributes/026_unpack_in_args.phpt
Zend/tests/attributes/constants/multiple_attributes_grouped.phpt
Zend/tests/attributes/delayed_target_validation/validator_Attribute.phpt
Zend/tests/attributes/deprecated/functions/001.phpt
Zend/tests/attributes/nodiscard/001.phpt
Zend/tests/attributes/override/010.phpt
```

## Why This Is A Blocker

Attribute support needs generic compiler/runtime architecture:

- Lexer/parser support for attribute groups before declarations and parameters.
- AST storage for attribute names, arguments, grouping, nesting, source spans,
  and declaration attachment points.
- Constant-expression evaluation for attribute arguments.
- Metadata tables for classes, methods, functions, closures, parameters,
  properties, class constants, and global constants.
- Validation for `Attribute` flags, targets, repeatability, delayed target
  checks, and built-in attributes such as `Deprecated`, `Override`, and
  `NoDiscard`.
- Reflection APIs for `ReflectionAttribute` and `getAttributes()` across all
  attachable declarations.
- Integration with existing class/interface/trait/enum metadata, many of which
  are still bounded or unsupported in PTN.

Treating these rows as runnable would turn parser/metadata absence into noisy
failures. Keeping them classified gives the broad baseline a stable blocker
count until the attribute architecture is implemented.

## Verification

`cargo fmt --check` and `cargo test --test phpt_classifier` should remain green
for this report-only slice. A broad execution attempt was started after the
classify-only run but did not produce a completed summary before this blocker
map; the committed evidence is the broad 1k classify-only artifact above.
