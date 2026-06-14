# PHPT Broad 1k Zend Bug Regression Frontier: 2026-06-14

Issue: `ptn-exlu`

This slice used the broad 1k PHPT classifier on `origin/master` and selected
the runnable `Zend/tests/bug*.phpt` cluster. These rows are historical engine
regressions, but in the current broad frontier they form a useful semantic
cross-section of object lifecycle, dynamic dispatch, references, quiet reads,
diagnostics, error suppression, and string-offset behavior.

This is a blocker map, not an implementation claim. The failures cross several
runtime/compiler primitives, so a narrow row-specific fix would be the wrong
shape.

## Broad 1k Evidence

Initial classifier source state:

- PTN: `80da9cd3a587`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-exlu-before-classify
```

Result:

| Measurement | Selected | Runnable | Excluded |
| --- | ---: | ---: | ---: |
| broad 1k classify-only | 1000 | 429 | 571 |

Artifacts:

```text
.runtime/ptn-exlu-before-classify/20260614T032443Z/phpt-baseline-1000.txt
.runtime/phpt-progress/classification-20260614T032443Z.tsv
.runtime/phpt-progress/runnable-20260614T032443Z.txt
.runtime/phpt-progress/excluded-20260614T032443Z.tsv
```

The broad runnable manifest contains 37 root-level `Zend/tests/bug*.phpt` rows.
They are committed as:

```text
tools/phpt-zend-bug-regression-frontier-manifest.txt
```

## Focused Evidence

The first focused run selected all 37 rows, but did not reach a numeric summary
because `Zend/tests/bug35239.phpt` exhausted `run-tests.php` memory while
diffing recursive object-reference output:

```sh
tools/run-bounded-phpt.sh .runtime/ptn-exlu/zend-bug-runnable.txt
```

Partial log:

```text
.runtime/phpt-progress/run-20260614T033042Z-manifest.log
```

The completed focused run split out `bug35239.phpt` and selected the remaining
36 rows:

```sh
tools/run-bounded-phpt.sh .runtime/ptn-exlu/zend-bug-no35239-runnable.txt
```

Result:

| Manifest | Selected | Runnable | Passing | Failing | Harness blocked |
| --- | ---: | ---: | ---: | ---: | ---: |
| `Zend/tests/bug*.phpt` excluding `bug35239.phpt` | 36 | 36 | 18 | 18 | 0 |
| `Zend/tests/bug35239.phpt` | 1 | 1 | 0 | 0 | 1 |
| Total frontier | 37 | 37 | 18 | 18 | 1 |

Completed log after rebasing across `ptn-3a8d`:

```text
.runtime/phpt-progress/run-20260614T034120Z-manifest.log
```

Passing rows:

```text
Zend/tests/bug20242.phpt
Zend/tests/bug22836.phpt
Zend/tests/bug23104.phpt
Zend/tests/bug26077.phpt
Zend/tests/bug27304.phpt
Zend/tests/bug30080.phpt
Zend/tests/bug30407.phpt
Zend/tests/bug31177-2.phpt
Zend/tests/bug31177.phpt
Zend/tests/bug32428.phpt
Zend/tests/bug33282.phpt
Zend/tests/bug33558.phpt
Zend/tests/bug34062.phpt
Zend/tests/bug34879.phpt
Zend/tests/bug35163.phpt
Zend/tests/bug37715.phpt
Zend/tests/bug38469.phpt
Zend/tests/bug38808.phpt
```

## Blocker Map

| Rows | Representative rows | Generic gap |
| ---: | --- | --- |
| 1 | `bug20240.phpt` | Shutdown function registration and shutdown/destructor ordering are not modeled. The row fails at missing `register_shutdown_function()`, before destructor-order parity is exercised. |
| 2 | `bug27669.phpt`, `bug29104.phpt` | Dynamic static method names and nested function declarations inside methods need parser/AST/lowering support distinct from class-constant fetch and ordinary method-body statements. |
| 2 | `bug29015.phpt`, `bug33999.phpt` | Dynamic object member names, NUL-prefixed property diagnostics, and object-to-number cast warnings need shared object conversion/property-name semantics. |
| 4 | `bug31098.phpt`, `bug34786.phpt`, `bug39018.phpt`, `bug39018_2.phpt` | Quiet property/string-offset reads, huge numeric string offsets, string-offset warning emission, and nested `@` error-suppression state need a single runtime diagnostic channel that preserves PHP's suppression stack. |
| 5 | `bug31525.phpt`, `bug34064.phpt`, `bug34137.phpt`, `bug35163_2.phpt`, `bug35470.phpt` | By-reference assignment targets, append lvalues passed by reference, self-referential array aliases, and dynamic `global ${expr}` need broader reference/lvalue roots instead of the current direct-target subset. |
| 3 | `bug31720.phpt`, `bug33996.phpt`, `bug37251.phpt` | Callable and argument diagnostics must be catchable and message-compatible across object callbacks, missing required arguments, and typed method parameters. |
| 1 | `bug36513.phpt` | `eval()` remains outside the native surface; this row also depends on `highlight_string()`/inline-comment formatting after eval output. |
| 1 | `bug35239.phpt` | Recursive stdClass reference dumping currently exhausts the PHPT harness diff path instead of producing a stable pass/fail signal; the underlying semantic frontier is object references, recursion-aware dumping, and property aliasing. |

## Next Implementation Splits

The most credible implementation splits are:

1. Add a shutdown callback registry that uses the same callable dispatch and
   shutdown ordering as destructors.
2. Extend lvalue/reference roots for append-by-reference parameters and dynamic
   globals.
3. Unify quiet string-offset/property reads with the error-suppression runtime
   state, including very large offset literals.
4. Make missing-argument and object-callback diagnostics catchable through the
   same internal/userland call boundary.

Those are shared primitives. They should not be implemented by matching the
historical bug row names or expected output text.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-exlu-before-classify
tools/run-bounded-phpt.sh tools/phpt-zend-bug-regression-frontier-manifest.txt
rg -v '^Zend/tests/bug35239\\.phpt$' tools/phpt-zend-bug-regression-frontier-manifest.txt > .runtime/ptn-exlu/zend-bug-no35239-runnable.txt
tools/run-bounded-phpt.sh .runtime/ptn-exlu/zend-bug-no35239-runnable.txt
```
