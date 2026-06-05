# 221205Z Zend/classes/sapi Regression Shard

Developer: developer-98
Lane: work_lanes#21
Gate: `phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
Evidence directory: `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
php-src pin: `f97ff597429a2fe633665a7e02d97c8077f9f90f`
Candidate/public head: `56fe9377fb46be00db5fdd30c966fdba406dc581`
Baseline pass set: `phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/current-passes.normalized.txt`

## Summary

This shard owns 22 latest-public PASS regressions:

- 15 under `php-src/Zend/tests/`
- 4 under `php-src/tests/classes/`
- 3 under `php-src/sapi/cli/`

Nineteen rows have explicit `FAILED` current status in
`current-status.normalized.tsv` and `all-results.txt`. The three `sapi/cli`
rows are in the baseline pass set and in
`regressions-from-latest-published-passes.txt`, but are absent from both
current status and all-results. Treat those as a control-plane/result-coverage
symptom until replay proves a semantic CLI failure.

No owned row contains `eval(` or `$$` in the pinned PHPT source.

## Likely Buckets

1. CLI wrapper / result-coverage gap: 3 rows.
   The current gate does not record a candidate status for these three
   baseline PASS rows.

2. Object lifecycle and destructor timing/visibility: 7 rows.
   Failures cluster around destructor ordering, premature destructor execution,
   protected destructor calls, iterator object lifetime, cyclic object cleanup,
   and closure cycles.

3. Property model and declaration compatibility: 7 rows.
   Failures cluster around readonly diagnostics, readonly class properties,
   trait readonly conflicts, property hooks satisfying interface properties,
   and property `#[Override]`.

4. Assertion / throwable / exception formatting: 4 rows.
   Failures include disabled assertion behavior, assertion message object
   stringification, exception/error serialization chains, and uncaught error
   formatting under `@`.

5. Internal inheritance compatibility diagnostic: 1 row.
   The candidate emits a direct declaration compatibility fatal instead of the
   expected "Could not check compatibility..." message when an unavailable
   parameter type is involved.

## Exact Row List

| Row | Current status | Symptom | Likely bucket |
| --- | --- | --- | --- |
| `php-src/sapi/cli/tests/002.phpt` | missing from current status/results | Baseline PASS row for CLI `-r` shell execution is absent from candidate result files. | CLI wrapper / result coverage |
| `php-src/sapi/cli/tests/021.phpt` | missing from current status/results | Baseline PASS row for executable shebang script behavior is absent from candidate result files. | CLI wrapper / result coverage |
| `php-src/sapi/cli/tests/bug70006.phpt` | missing from current status/results | Baseline PASS row for `STDOUT` as a default argument is absent from candidate result files. | CLI wrapper / result coverage |
| `php-src/tests/classes/ctor_dtor.phpt` | `FAILED` | Destructor output is emitted too early. The diff shows `early::__destruct` and `late::__destruct` before the expected positions. | Destructor timing |
| `php-src/tests/classes/destructor_and_echo.phpt` | `FAILED` | Row expects output before final destructor output; durable shard stdout records FAIL but no `run-tests.log` diff was captured for shard 04. | Destructor timing |
| `php-src/tests/classes/factory_and_singleton_002.phpt` | `FAILED` | Candidate fatals on `Call to protected test::__destruct() from global scope` during construction/destruction flow; expected normal singleton output plus shutdown warning. | Destructor visibility / lifecycle |
| `php-src/tests/classes/iterators_002.phpt` | `FAILED` | Inner iterator destructor runs immediately after construction in the candidate, before expected rewind/current iteration sequence. | Iterator object lifetime |
| `php-src/Zend/tests/assert/expect_008.phpt` | `FAILED` | Disabled assertion side-effect row records FAIL; durable shard stdout records FAIL but no shard 04 diff was captured. | Assertion side effects |
| `php-src/Zend/tests/assert/expect_011.phpt` | `FAILED` | Candidate reports `undefined property MyExpectations::$string` instead of the expected `AssertionError` message with custom expectation stringification. | Assertion / throwable stringification |
| `php-src/Zend/tests/attributes/deprecated/property_readonly_001.phpt` | `FAILED` | Diagnostic text differs: candidate reports protected-set readonly/global-scope wording; expected generic `Cannot modify readonly property Deprecated::$message`. | Readonly diagnostic parity |
| `php-src/Zend/tests/attributes/deprecated/property_readonly_002.phpt` | `FAILED` | Same readonly diagnostic mismatch for `Deprecated::$since`. | Readonly diagnostic parity |
| `php-src/Zend/tests/attributes/override/properties_08.phpt` | `FAILED` | Candidate fatals that `Foo` has an unimplemented interface property hook; expected `Done` for trait property satisfying interface property with `#[Override]`. | Property hook/interface compatibility |
| `php-src/Zend/tests/bug73989.phpt` | `FAILED` | Expected `OK`; candidate output block is empty. PHPT repeatedly creates cyclic objects whose destructor invokes a captured closure. | Object lifecycle / closure cycle cleanup |
| `php-src/Zend/tests/gc/bug63635.phpt` | `FAILED` | Expected sequence `0` through `19` and `ok`; failure snippet begins with missing expected output, indicating premature stop or output loss in cyclic object/destructor GC stress. | Object lifecycle / GC |
| `php-src/Zend/tests/property_hooks/gh19548_002.phpt` | `FAILED` | Candidate fatals that class `C1` must implement `I1::$a::get` and `I1::$b::get`; expected inherited concrete properties to satisfy hooked interface properties. | Property hook/interface compatibility |
| `php-src/Zend/tests/property_hooks/gh19548.phpt` | `FAILED` | Same hooked interface/inherited property shape as `gh19548_002`; durable shard stdout records FAIL but no shard 03 diff was captured. | Property hook/interface compatibility |
| `php-src/Zend/tests/readonly_classes/readonly_class_property1.phpt` | `FAILED` | Row exercises implicit readonly on normal properties of a readonly class; durable shard stdout records FAIL but no shard 03 diff was captured. | Readonly class property semantics |
| `php-src/Zend/tests/readonly_classes/readonly_class_property2.phpt` | `FAILED` | Row exercises implicit readonly on promoted constructor properties of a readonly class; durable shard stdout records FAIL but no shard 04 diff was captured. | Readonly class property semantics |
| `php-src/Zend/tests/readonly_props/readonly_trait_mismatch.phpt` | `FAILED` | Row expects trait composition fatal for readonly mismatch; durable shard stdout records FAIL but no shard 04 diff was captured. | Trait property compatibility |
| `php-src/Zend/tests/serialize/bug76502.phpt` | `FAILED` | Row serializes/unserializes mixed `Exception`/`Error` previous chains; durable shard stdout records FAIL but no shard 03 diff was captured. | Throwable serialization |
| `php-src/Zend/tests/type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt` | `FAILED` | Candidate emits normal declaration compatibility fatal; expected unavailable-class compatibility-check fatal. | Internal inheritance diagnostic |
| `php-src/Zend/tests/uncaught_exception_error_supression.phpt` | `FAILED` | Row checks that `@` does not suppress uncaught `Error`; durable shard stdout records FAIL but no shard 03 diff was captured. | Uncaught throwable formatting |

## Highest-Signal Replay Set

Replay these first before any repair lane is drafted:

- `php-src/sapi/cli/tests/002.phpt`
- `php-src/sapi/cli/tests/021.phpt`
- `php-src/sapi/cli/tests/bug70006.phpt`
- `php-src/tests/classes/ctor_dtor.phpt`
- `php-src/tests/classes/factory_and_singleton_002.phpt`
- `php-src/Zend/tests/property_hooks/gh19548_002.phpt`
- `php-src/Zend/tests/attributes/override/properties_08.phpt`
- `php-src/Zend/tests/assert/expect_011.phpt`
- `php-src/Zend/tests/type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt`

The first three isolate control-plane/result coverage. The remaining rows
separate lifecycle/destructor behavior, property hook/interface compatibility,
assertion object stringification, and exact inheritance diagnostics.

## Evidence Commands

Commands run from `/home/claude/php-to-native-compiler/.harness/worktrees/developer-98`:

- `rg -n '^(php-src/(Zend/tests|tests/classes|sapi/cli)/)' regressions-from-latest-published-passes.txt`
- Python script over `regressions-from-latest-published-passes.txt`,
  `current-status.normalized.tsv`, and `all-results.txt` to count owned rows
  and map statuses.
- Python/title scan over `/home/claude/php-src-phpt` to extract `--TEST--`
  titles and feature markers.
- Targeted `rg`/`sed` inspection of shard `results.txt`, `stdout.log`, and
  available `run-tests.log` files.

No source files were edited and no PHPT replay/full gate was run for this
read-only classification lane.
