# PHPT Broad 1k Core/Basic Operator Frontier: 2026-06-14

Issue: `ptn-s7ug`

This slice refreshed the broad 1k PHPT classifier on current `origin/master`
and selected the residual core/basic operator-control-flow rows that are not
part of the already mapped broad standard-array, class/metadata, request/SAPI,
assignment/reference, or Zend historical bug-regression frontiers.

This is a blocker map, not a support claim. The focused failures cross several
generic runtime/compiler boundaries, so a narrow implementation patch is not a
credible 25-row move for this slice.

## Broad 1k Evidence

Source state:

- PTN: `ade9b65af2f0`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-s7ug-baseline-rebased
```

Generated broad manifest:

```text
.runtime/ptn-s7ug-baseline-rebased/20260614T055517Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/classification-20260614T055517Z.tsv
.runtime/phpt-progress/runnable-20260614T055517Z.txt
.runtime/phpt-progress/excluded-20260614T055517Z.tsv
```

Classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 425 | 575 |

Top current exclusions:

| Classification | Rows |
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

## Focused Evidence

Committed focused manifest:

```text
tools/phpt-core-basic-operator-frontier-manifest.txt
```

Selection from the current broad 1k runnable manifest:

```sh
awk -F '\t' '$2=="runnable" && \
  ($1 ~ /^Zend\/tests\/add_/ || $1=="Zend/tests/and_001.phpt" || \
   $1 ~ /^Zend\/tests\/ast\// || $1=="Zend/tests/binary.phpt" || \
   $1 ~ /^Zend\/tests\/break_error_/ || $1 ~ /^tests\/basic\//) {print $1}' \
  .runtime/phpt-progress/classification-20260614T055517Z.tsv
```

Focused run:

```sh
tools/run-bounded-phpt.sh tools/phpt-core-basic-operator-frontier-manifest.txt
```

Artifacts:

```text
.runtime/phpt-progress/run-20260614T060022Z-manifest.log
.runtime/phpt-progress/classification-20260614T060022Z.tsv
.runtime/phpt-progress/runnable-20260614T060022Z.txt
```

Result:

| Selected | Runnable | Passed | Failed |
| ---: | ---: | ---: | ---: |
| 34 | 34 | 18 | 16 |

Passing rows:

```text
Zend/tests/add_001.phpt
Zend/tests/add_002.phpt
Zend/tests/add_003.phpt
Zend/tests/add_004.phpt
Zend/tests/add_005.phpt
Zend/tests/add_006.phpt
Zend/tests/add_007.phpt
Zend/tests/add_optional_by_ref_arg.phpt
Zend/tests/and_001.phpt
Zend/tests/ast/zend-pow-assign.phpt
tests/basic/001.phpt
tests/basic/006.phpt
tests/basic/007.phpt
tests/basic/008.phpt
tests/basic/009.phpt
tests/basic/010.phpt
tests/basic/array_key_exists_null_deprecation.phpt
tests/basic/array_null_offset_deprecation.phpt
```

## Blocker Map

| Rows | Failed rows | Generic blocker |
| ---: | --- | --- |
| 3 | `Zend/tests/ast/ast_serialize_backtick_literal.phpt`, `Zend/tests/ast/ast_serialize_floats.phpt`, `Zend/tests/ast/gh21072.phpt` | AST/export and constant-expression diagnostics need source-level AST preservation for backticks, float literal spelling, and `(unset)` cast diagnostics inside class constant expressions. The current paths either reject backticks as unsupported tokens, normalize `0.0` to `0`, or stop at the broader class-declaration boundary. |
| 4 | `Zend/tests/break_error_001.phpt`, `Zend/tests/break_error_002.phpt`, `Zend/tests/break_error_003.phpt`, `Zend/tests/break_error_004.phpt` | `break`/`continue` validation needs PHP-compatible level expression parsing and compile-time fatal diagnostics for non-positive, non-integer, out-of-context, and excessive-level cases. Current output is either empty or a generic parse error. |
| 1 | `Zend/tests/binary.phpt` | Binary integer literal parsing needs full radix/overflow parity. The row reaches wide binary literals where PHP transitions through integer and float results, while PTN currently reports an invalid integer literal. |
| 3 | `tests/basic/bug45986.phpt`, `tests/basic/bug54514.phpt`, `tests/basic/build_date.phpt` | Core filesystem/process/build metadata is incomplete: `rename()` diagnostics, `PHP_BINARY`, and `PHP_BUILD_DATE` are missing or not modeled through shared runtime metadata. |
| 3 | `tests/basic/encoding.phpt`, `tests/basic/ini_parse_quantity_basic.phpt`, `tests/basic/ini_parse_quantity_warnings.phpt` | INI/config helper state is incomplete. `ini_set()` and encoding state are not modeled broadly enough, and `ini_parse_quantity()` plus its warning compatibility are missing. |
| 2 | `tests/basic/header_register_callback.phpt`, `tests/basic/header_register_callback_after_output.phpt` | CLI SAPI header state lacks `header_register_callback()` registration, header-sent ordering, and output boundary semantics. |

## Next Implementation Splits

1. Add parser/diagnostic coverage for `break` level expressions before runtime
   work; this should be a focused parser/control-flow slice.
2. Extend numeric literal parsing for large binary/radix literals using the
   same overflow path as decimal/hex literals.
3. Treat AST export rows as a separate source-metadata feature; they need
   source-preserving AST serialization rather than runtime output tweaks.
4. Split core basic internals into explicit runtime primitives:
   `rename()` filesystem diagnostics, process/build constants, ini quantity
   parsing, and SAPI header callback state.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-s7ug-baseline-rebased
tools/run-bounded-phpt.sh tools/phpt-core-basic-operator-frontier-manifest.txt
```
