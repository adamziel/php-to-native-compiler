# PHP Native Compiler Progress

Updated: 2026-05-28 07:08 CEST
Primary branch: `master`
Latest source head: `0fa7b666 runtime: autoload enum_exists misses`

## Progress Score

This file is the public progress report for the project. AO workers and the
supervisor must update this file before claiming public progress.

Progress is the pinned php-src PHPT full-suite pass rate:

`passed runnable PHPTs / total runnable PHPTs`

Current score: **1118 / 20294 runnable PHPTs = 5.51%**.

The first full-suite baseline was recorded for Batch 001 stack10 on php-src
`f97ff597429a2fe633665a7e02d97c8077f9f90f`, run
`phpt-full-batch001-20260528T010422Z-php-src-f97ff59-base-3e702be4-stack10`.
Counts: 1118 passed, 19156 failed, 964 skipped, 20 xfailed, 0 borked;
`run-tests.php` exited 1. Evidence lives under
`/home/claude/supervised-php-compiler/state/logs/phpt-full-batch001-20260528T010422Z-php-src-f97ff59-base-3e702be4-stack10`.

## PHPT Harness

| Item | State | Evidence |
| --- | --- | --- |
| php-src pin | Done | `/home/claude/php-src-phpt` at `f97ff597429a2fe633665a7e02d97c8077f9f90f` |
| Static inventory | Done | 21,827 PHPT files; 12,777 static runnable candidates |
| `phpc` PHPT wrapper | Done | `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` |
| Skip/xfail ledger | Started | `/home/claude/supervised-php-compiler/state/php-core-suite-skip-ledger.tsv` |
| First full-suite baseline | Done | 1118 / 20294 runnable PHPTs passed (5.51%); run id `phpt-full-batch001-20260528T010422Z-php-src-f97ff59-base-3e702be4-stack10` |

Focused PHPT history is tracked separately in
`/home/claude/supervised-php-compiler/state/php-core-suite-focused-history.tsv`.
Focused passes prove candidate direction; they do not define project percent.

## Batch 001

Policy: stage 10 accepted generalized source PRs, run focused gates per PR, run
the full PHPT suite once after PR 10, repair regressions, then merge the whole
batch.

Current batch status: **10/10 accepted, not merged; full PHPT baseline recorded; regression/failure repair next**.
Independent reviewer `phpc-7` accepted r81 / PR #1 as Batch 001 PR 10
at 2026-05-28 02:59 CEST after accepted-stack apply, exact-shape audit,
focused Rust/compiler gates, and focused wrapper PHPT `Zend/tests/namespaces/ns_065.phpt`
passed (1/1).

Accepted for staging:

| # | Candidate | Main proof |
| ---: | --- | --- |
| 1 | Magic-method signature diagnostics | Current-head review, focused diagnostics gates |
| 2 | Symbol-table foreach owners | Current-head review, focused native-link gates |
| 3 | Exception/catch/finally propagation | Current-head review, focused exception gates |
| 4 | Generated-C return-reference sources | Current-head review, accepted-stack compatibility, focused reference-return gates |
| 5 | Closing-tag statement terminator | Focused `tests/basic/001.phpt`, invalid-syntax review, accepted-stack compatibility |
| 6 | Object lifecycle live roots | Caller-frame live-root review, accepted-stack compatibility, focused destructor gates |
| 7 | Grouped namespace class imports | Current-head review, accepted-stack compatibility, focused compiler gates, wrapper PHPT proof |
| 8 | By-reference foreach lingering slots | Accepted-stack review, focused `Zend/tests/foreach/foreach_reference.phpt`, slot-preserving array-copy gates |
| 9 | Magic method startup signature fatals | Accepted-stack review, focused `tests/classes/__call_002.phpt`, generalized magic contract gates |
| 10 | Multiple unbracketed namespace declarations | Independent accepted-stack review, focused `Zend/tests/namespaces/ns_065.phpt`, namespace parser/import gates |

Gate status and parked candidates:

| Item | State |
| --- | --- |
| Batch 001 full PHPT gate | Done in AO session `phpc-11`; first baseline recorded at 1118 / 20294 runnable PHPTs (5.51%) |
| Full-suite count guard | Done; `all-results.txt` used `PASSED/FAILED/SKIPPED/XFAILED`, the parser counted those statuses, and the verified row is in `state/php-core-suite-history.tsv` |
| PR #4 by-reference call expressions | Batch002 stack decision says PR #4 supersedes r82/PR #2; use PR #4 as the by-reference candidate because it covers `Zend/tests/bug39944.phpt` plus adjacent return/pass-by-reference PHPTs |
| PR #5 named by-reference arguments | GO-CANDIDATE after independent review on accepted stack10 + PR #4 + PR #5; focused Rust gates passed and wrapper PHPTs `Zend/tests/named_params/references.phpt`, `tests/lang/passByReference_007.phpt`, and `tests/lang/returnByReference.002.phpt` passed 3/3 |
| PR #6 foreach reference-backed `print_r()` | GO-CANDIDATE after independent review on accepted stack10; focused Rust/build gates passed and wrapper PHPT `tests/lang/foreach_with_references_001.phpt` plus foreach anchors passed after a generalized reference-backed array formatting fix |
| PR #7 magic `__call()` by-reference array args | GO-CANDIDATE after refreshed stack-safe independent review and p14 `SAFE-FOR-PROGRESS`; focused Rust/build/fixture gates passed and wrapper PHPTs `tests/classes/__call_003.phpt` plus `tests/classes/__call_001.phpt` passed 2/2; no full-suite run and no percent change |
| PR #7 follow-up: `__call_004` static-syntax fallback to current `__call()` | GO-CANDIDATE after independent review on accepted stack10 plus reviewed Batch002 through refreshed PR #7 and p14 `SAFE-FOR-PROGRESS`; focused Rust/build/fixture gates passed and wrapper PHPTs `tests/classes/__call_004.phpt`, `tests/classes/__call_003.phpt`, and `tests/classes/__call_001.phpt` passed 3/3; no full-suite run and no percent change |
| PR #8 `passByReference_002` real-stack refresh | GO-CANDIDATE after independent review on accepted stack10 plus reviewed Batch002 through refreshed PR #7 and p14 `SAFE-FOR-PROGRESS`; focused Rust/build gates passed and wrapper PHPTs `tests/lang/passByReference_002.phpt` plus `tests/lang/passByReference_004.phpt` passed 2/2; no full-suite run and no percent change |
| PR #13 `passByReference_012` / `array_shift()` by-reference builtin | GO-CANDIDATE after refreshed independent review on public base `49a44b0d` and p14 `SAFE-FOR-PROGRESS`; patch applies without a progress hunk, focused Rust `array_shift_builtin` tests passed 5/5, `cargo build -p phpc` passed, and wrapper PHPTs `tests/lang/passByReference_012.phpt`, `tests/lang/passByReference_008.phpt`, and `tests/lang/passByReference_009.phpt` passed 3/3; no full-suite run and no percent change |
| PR #9 `passByReference_004` real-stack PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent review and p14 `SAFE-FOR-PROGRESS`; the reviewed PR #8 stack already passes the target, the stale PR #9 patch is obsolete, and reviewer PHPT rerun `tests/lang/passByReference_004.phpt` plus `tests/lang/passByReference_002.phpt` passed 2/2; no source patch, no full-suite run, and no percent change |
| PR #16 `returnByReference.004` uppercase `Class` declaration keyword | GO-CANDIDATE after independent review and p14 `SAFE-FOR-PROGRESS`; focused `object_model` Rust gate, `cargo build -p phpc`, and wrapper PHPTs `tests/lang/returnByReference.004.phpt`, `tests/lang/returnByReference.002.phpt`, and `tests/lang/returnByReference.003.phpt` passed 3/3; no full-suite run and no percent change |
| PR #17 `returnByReference.006` dynamic-call return-by-reference fallback | GO-CANDIDATE after independent p7 review and p14 `SAFE-FOR-PROGRESS`; patch SHA `9cf5386634f214fb83e1517337ad4ea12f89662808f3c305143ebd3fcf1ec12e`; focused Rust gate, `cargo build -p phpc`, and wrapper PHPTs `tests/lang/returnByReference.006.phpt` plus `tests/lang/returnByReference.003.phpt` passed 2/2; no full-suite run and no percent change |
| PR #10 `passByReference_006` real-stack `var_dump()` reference visibility | GO-CANDIDATE after independent review and p14 `SAFE-FOR-PROGRESS`; patch SHA `3c5a5ef37747de2a52374eabe35f674e4732cbe588bd742a4bc7ae6e0ca4304b`; focused Rust gate, `cargo build -p phpc`, and wrapper PHPTs `tests/lang/passByReference_006.phpt`, `tests/lang/passByReference_004.phpt`, and `tests/lang/passByReference_002.phpt` passed 3/3; no full-suite run and no percent change |
| p19 `passByReference_005` repair2 missing-variable by-reference cells and non-referenceable argument fatal | GO-CANDIDATE after independent p7 review and p14 `SAFE-FOR-PROGRESS`; patch SHA `6c1fb034e7f598f214069728fc3c46bfd2e718742f7a5c6bcd8823f403a4a6ab`; `cargo fmt`, focused Rust `missing_variable_reads_warn_and_reference_arguments_materialize_null_cells`, `cargo build -p phpc`, and wrapper PHPTs `tests/lang/passByReference_005.phpt`, `_006.phpt`, `_004.phpt`, and `_002.phpt` passed 4/4; no full-suite run and no percent change |
| PR #19 `passByReference_003` undefined call-argument recovery | GO-CANDIDATE after independent review and p14 `SAFE-FOR-PROGRESS`; patch SHA `4f54ff6bd5517848b77f711e04373a1a571a6e8548dd7d2e31be9d7fab8a2ad6`; focused Rust gates, `cargo build -p phpc`, and wrapper PHPTs `tests/lang/passByReference_003.phpt`, `tests/lang/passByReference_001.phpt`, and `tests/lang/passByReference_007.phpt` passed 3/3; no full-suite run and no percent change |
| PR #18 `returnByReference.008` dynamic instance-method return-by-reference fallback | GO-CANDIDATE after independent review and p14 `SAFE-FOR-PROGRESS`; patch SHA `1c655efefb2e1aba956e912c4fe0a3c18f870497ff1b37f890cb53219f038d4f`; focused Rust gate, `cargo build -p phpc`, and wrapper PHPTs `tests/lang/returnByReference.008.phpt`, `tests/lang/returnByReference.004.phpt`, `tests/lang/returnByReference.003.phpt`, and `tests/lang/returnByReference.009.phpt` passed 4/4; no full-suite run and no percent change |
| p16 `call_static` magic static callable dispatch | GO-CANDIDATE after independent p7 review and p14 `SAFE-FOR-PROGRESS`; patch SHA `561b597f23a510902f02d9dd4cb25b23c46b15e279dea5d232f269b7b1639613`; focused Rust gate, `cargo build -p phpc`, and wrapper PHPT `Zend/tests/magic_methods/call_static.phpt` passed; nearby anchor `Zend/tests/magic_methods/call_static_002.phpt` failed in both candidate and reviewed-baseline runs and is recorded as pre-existing/non-regression; no full-suite run and no percent change |
| `Zend/tests/magic_methods/bug32429.phpt` `method_exists()` with `__call` PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing `method_exists()` behavior correctly returns false for an absent method even when `__call` exists, and wrapper PHPT `Zend/tests/magic_methods/bug32429.phpt` passed 1/1; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug36006.phpt` destructor `$this` / parent-destructor cleanup PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing destructor cleanup behavior passes wrapper PHPT `Zend/tests/magic_methods/bug36006.phpt` 1/1; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug37707.phpt` clone-new `__clone` PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing clone handling invokes `__clone` for unassigned `clone new` expressions and passes wrapper PHPT `Zend/tests/magic_methods/bug37707.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug36759.phpt` shutdown destructor order PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing shutdown destructor order behavior passes wrapper PHPT `Zend/tests/magic_methods/bug36759.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug38146.phpt` `__get` array-return foreach PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing runtime iterates over an array returned by `__get` in foreach/read context and passes wrapper PHPT `Zend/tests/magic_methods/bug38146.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug44899_2.phpt` `__isset` / `empty()` / `__get` PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing magic-property behavior handles the `__isset`/`empty()`/`__get` interaction and passes wrapper PHPT `Zend/tests/magic_methods/bug44899_2.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug47353.phpt` destructor object-allocation loop PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing destructor/object-allocation handling passes wrapper PHPT `Zend/tests/magic_methods/bug47353.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug51822.phpt` static-property destructor ordering PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing static-property object lifetime/destructor ordering handles the target and passes wrapper PHPT `Zend/tests/magic_methods/bug51822.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug54372.phpt` chained `__get` receiver method-call PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing chained magic property access preserves the object returned by `__get()` for the following method call, passing wrapper PHPT `Zend/tests/magic_methods/bug54372.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| p15 `returnByReference.005` rebased object-receiver static reference-return dispatch | GO-CANDIDATE after independent p7 review and p14 `SAFE-FOR-PROGRESS`; patch SHA `9487557714a456f2b3f416af7db1ed9866c6428dd6072bd143afe6a86dd27895`; focused Rust gate, `cargo build -p phpc`, and wrapper PHPTs `tests/lang/returnByReference.005.phpt`, `tests/lang/returnByReference.004.phpt`, and `tests/lang/returnByReference.003.phpt` passed 3/3; no full-suite run and no percent change |
| `Zend/tests/dereference/dereference_005.phpt` array dereference PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing generalized ArrayAccess/object dereference handling passes wrapper PHPT `Zend/tests/dereference/dereference_005.phpt` 1/1; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/dereference/dereference_008.phpt` dynamic-method array dereference PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing dynamic-method dereference/reference behavior passes wrapper PHPT `Zend/tests/dereference/dereference_008.phpt` 1/1; no source patch, no cargo gate, no full-suite run, and no percent change |
| `tests/lang/passByReference_008.phpt` / `tests/lang/passByReference_009.phpt` duplicate by-reference/by-value call-frame PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing call-frame argument binding/evaluation semantics pass wrapper PHPTs 2/2; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/type_declarations/typed_properties_011.phpt` typed-property array reference fetch PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing typed-property/reference behavior allows fetching a typed property by reference into an array and passes wrapper PHPT `Zend/tests/type_declarations/typed_properties_011.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/list/list_004.phpt` `list()` assignment from array reference PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing list-assignment/reference behavior reads from an aliased array and passes wrapper PHPT `Zend/tests/list/list_004.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/list/bug65969.phpt` chain assignment with `list()` PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing chain assignment behavior lets `list()` destructure the same array assigned to an object property, passing wrapper PHPT `Zend/tests/list/bug65969.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/list/bug72395.phpt` `list()` regression PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing list/foreach behavior covers the php-src regression target, passing wrapper PHPT `Zend/tests/list/bug72395.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/variadic/basic.phpt` basic variadic argument packing PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing variadic call binding packs surplus arguments and passes wrapper PHPT `Zend/tests/variadic/basic.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/variadic/optional_params.phpt` optional-parameter-before-variadic PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and durable p14 `SAFE-FOR-PROGRESS`; existing variadic call binding preserves optional defaults before collecting surplus arguments and passes wrapper PHPT `Zend/tests/variadic/optional_params.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/variadic/removing_parameter_error.phpt` remove required parameter before variadic PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing declaration compatibility accepts removing the required parameter before a variadic parameter, passing wrapper PHPT `Zend/tests/variadic/removing_parameter_error.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/variadic/variadic_implements_non_variadic.phpt` variadic implementation widening PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing method compatibility behavior accepts an implementation that widens a non-variadic interface method to a variadic child method, passing wrapper PHPT `Zend/tests/variadic/variadic_implements_non_variadic.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/bug39944.phpt` reference invocation | PR #2/r82 is parked/superseded for Batch002; do not stack it with PR #4 because both conflict in `compiler/src/interpreter.rs` and `compiler/tests/functions_and_scopes.rs` |
| Magic visibility warnings | PR #3 is `REBASE-NEEDED` for Batch 002 after r81/stack10 due docs conflict; production/test hunks replay |
| Foreach `$GLOBALS` lane | PASS-NO-PATCH accepted by reviewer; accepted stack10 passes `foreach_unset_globals`, `foreach_reference`, and `foreach_temp_array_expr_with_refs` |
| `Zend/tests/foreach/foreach_unset_globals.phpt` foreach over local array while unsetting `$GLOBALS` PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after corrected independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach behavior traverses the local array while unsetting matching `$GLOBALS[$key]` entries without mutating the iterated array, passing wrapper PHPT `Zend/tests/foreach/foreach_unset_globals.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| Foreach object-property by-reference lane | GO-CANDIDATE after independent review; focused PHPT `Zend/tests/foreach/foreach_by_ref_to_property.phpt` plus foreach anchors passed 3/3, with PR #3/#4 stack compatibility checks |
| `Zend/tests/foreach/foreach_reference.phpt` by-reference foreach lingering alias PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior preserves the last-element alias through `array_values()` and `array_reverse()` and passes wrapper PHPT `Zend/tests/foreach/foreach_reference.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_temp_array_expr_with_refs.phpt` temporary array references foreach PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior iterates temporary arrays containing references by reference and mutates the original variables, passing wrapper PHPT `Zend/tests/foreach/foreach_temp_array_expr_with_refs.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_by_ref_repacking_insert.phpt` by-reference foreach packed-to-hash repacking PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior handles packed-to-hash insert/repacking at the end of by-reference iteration and passes wrapper PHPT `Zend/tests/foreach/foreach_by_ref_repacking_insert.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/goto_in_foreach.phpt` goto into foreach body PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/goto control flow enters the foreach body and continues after the loop, passing wrapper PHPT `Zend/tests/foreach/goto_in_foreach.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/foreach/bug37046.phpt` nested foreach static-scope PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/static-local behavior handles nested foreach loops over arrays returned by a function using a static local, passing wrapper PHPT `Zend/tests/foreach/bug37046.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_005.phpt` nested by-reference foreach PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior passes wrapper PHPT `Zend/tests/foreach/foreach_005.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_006.phpt` repeated by-reference foreach constant-array PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior passes wrapper PHPT `Zend/tests/foreach/foreach_006.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_007.phpt` by-reference foreach append-at-end PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior visits the newly inserted element and passes wrapper PHPT `Zend/tests/foreach/foreach_007.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_008.phpt` nested by-reference foreach unset PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior handles nested by-reference foreach while elements are unset and passes wrapper PHPT `Zend/tests/foreach/foreach_008.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_009.phpt` nested by-reference foreach sparsified-array PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior handles nested by-reference foreach over a sparsified/resized array while inserting during inner iteration and passes wrapper PHPT `Zend/tests/foreach/foreach_009.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_014.phpt` by-reference foreach `array_pop()` pointer PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior handles by-reference foreach iterator pointer behavior when `array_pop()` removes the last element during iteration and passes wrapper PHPT `Zend/tests/foreach/foreach_014.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| Anonymous-class dynamic-call blocker | AO scout classified this as NO-GO for Batch 001 PR 10; deferred as a broader parser/interpreter/native feature |
| PHPT focused queue | `tests/classes/__set__get_002.phpt` passes on the 9/10 stack; r85 queue now feeds additional coder lanes |
| Codex thread-store permissions | Fixed current session directory execute bit; smoke passed |
| Disk/data cleanup | Reclaimed Codex SQLite WAL; `/home` currently has 286G free |
| Agent Orchestrator migration | AO is installed, configured, polling this project, and persistent critic/reviewer/progress-reporter/coder roles are active |

## AO Control Plane

AO dashboard: `http://localhost:3000/projects/php-to-native-compiler`.

Required live roles:

| Role | Responsibility |
| --- | --- |
| Critic | Read-only audit for exact-shape lowering, shallow evidence, stale artifacts, and premature completion |
| Reviewer | Independent candidate apply/review/focused-gate proof before Batch 001 acceptance |
| Progress reporter | Keeps this `PROGRESS.md` file and durable supervisor state current after material AO events |
| Coders | Work disjoint focused PHPT lanes from the queue; each lane must produce a patch, PASS-NO-PATCH, or NO-GO artifact |

Current AO snapshot: `phpc-orchestrator` supervising; `phpc-14` critic;
`phpc-7` reviewer; `phpc-8` progress reporter; active coder/support lanes
`phpc-15`, `phpc-16`, `phpc-17`, `phpc-18`, and `phpc-19`. Current
public-progress watch targets are typed-properties `typed_properties_020`/successors, magic `bug61970`/successors,
variadic `variadic_changed_byref_error`/override successors, foreach
`bug35106`/successors, list `bug71030`/successors, the next
p7-reviewed candidate plus p14 `SAFE-FOR-PROGRESS` audit, current coder lane
artifacts, and any new full-suite PHPT row. Known no-go/not-safe items,
including unsuffixed `bug44899`, `bug46238`, `bug48248`, `foreach_010`,
`foreach_016` and `foreach_list_001`, remain excluded. Extra sessions `phpc-22` and stale `phpc-2` are
killed/not active roster capacity.

## Current Rules

- No exact-shape production lowering for individual PHPTs.
- No docs-only or tests-only progress.
- No full PHPT suite for every change.
- No batch merge before 10 accepted PRs, a full PHPT run, and regression repair.
- Legacy roadmap bars are retired; use PHPT pass rate as the only percent.

## Recent Source Anchors

| Commit | Capability | Gate log |
| --- | --- | --- |
| `0fa7b666` | Interpreter `enum_exists()` now uses SPL autoload callback/recheck for enum misses. | `state/logs/phpc-primary-enum-autoload-a5abdbb5-20260528.gates.log` |
| `2ef16e0d` | Request-scope `throw` inside active generated-C `finally` replays `finally` before the current unsupported-throw fatal boundary. | `state/logs/phpc-primary-throw-finally-fd52417e-20260528.gates.log` |
| `9c49c29b` | Generated-C comparison aborts now use cleanup-aware native error exits. | `state/logs/phpc-primary-comparison-abort-cleanup-4ed1624e-20260528.gates.log` |
| `d97a9fcf` | Dynamic runtime-registry missing required includes run active generated-C `finally` before fatal diagnostics. | `state/logs/phpc-primary-dynamic-include-finally-8a0a982f-20260528.gates.log` |

Detailed worker logs, PHPT inventory, batch review reports, and skip policy live
under `/home/claude/supervised-php-compiler/state/`.
