# PHP Native Compiler Progress

Updated: 2026-05-26 03:04 CEST
Evaluation marker: `20260526T010140Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, architecture
notes, dashboard-only commits, and evaluator-only dashboard refreshes are not
source capability.

## Executive Snapshot

Overall estimated progress: **95%** `[###################-]`

Executable PHP semantics: **95%** `[###################-]`

Primary was clean and aligned with `origin/master` at
`d76828e3 docs: account request-state frame handoff` before this evaluator
refresh. Latest primary-integrated source capability remains
`b27bbb20 native: add request-state frame environment handoff`.

Since the previous evaluator marker, the request/global direct generated-C
user-function frame handoff was accounted and pushed as `d76828e3`. No newer
compiler/runtime source commit has landed after `b27bbb20`.

## Current Integrated Baseline

`b27bbb20` replaced root-symbol-only generated user-function metadata with
`CFrameEnvironmentRequirement { root_symbols, request_state }` and propagates
that requirement through direct generated-C user-function callable handoff.
Direct calls now carry root-symbol and request-state frame needs through the
shared callable table, call arguments, call frame, wrapper, and invoke ABI.

Focused evidence covers direct request reads, request mutation,
`$GLOBALS["_GET"]` alias mutation, mixed ordinary `global` plus request-state
frame needs, direct-call propagation, and callable ABI behavior across
representative surfaces. This is integrated primary capability. Dynamic
callable request-state handoff, closure frame-environment capture/handoff,
`$GLOBALS` self-cells, full request/global alias parity, request writeback,
includes, variable variables, object/method environments, references/COW, and
cleanup/unwind parity remain open.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Primary-integrated read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong shared value, reference, symbol, call-frame, callable, request-state, diagnostic operand-list, and selected object/runtime contracts. Remaining runtime gaps include callable lookup parity, dynamic/closure request-state handoff, constructors, magic/autoload, and cleanup/unwind execution. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C has the freshest executable semantics; LLVM/direct assembly lag several semantic packets. Direct user-function calls now consume root/request frame requirements through the callable ABI. Dynamic callable compiler consumption remains lane-local. |
| Executable PHP semantics | **95%** | `[###################-]` | Many selected executable islands exist, but broad PHP behavior is not complete despite the project-local 95% accounting. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and byte-preserving `explode()` / `str_split()` slots are integrated. Binary source bytes, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **82%** | `[################----]` | Selected reference-source/lvalue extraction and reference-backed closure captures are integrated. Full executable reference binding, COW, arbitrary alias roots, foreach, ArrayAccess, and broad writeback remain incomplete. |
| Symbols, globals, request state | **75%** | `[###############-----]` | Selected globals, root-symbol consumers, request-key blockers, append-shaped symbol reference sources, and direct request-state frame handoff are integrated. `$GLOBALS` self-cells, request writeback, includes, variable variables, and dynamic/closure handoff remain incomplete. |
| Calls, functions, frames | **92%** | `[##################--]` | Callable table/arguments/result/frame ABI, runtime callable-value dispatch, direct user-function consumers, and direct root/request frame handoff are integrated. Dynamic callable compiler consumption, callable parity, closures, constructors, and cleanup/unwind parity remain open. |
| Objects, properties, methods | **53%** | `[###########---------]` | Selected public property/reference-source and allocation-risk metadata exist. Full visibility, magic, dynamic/static/typed properties, trait/method execution, constructors/destructors, and object lifetime cleanup remain open. |
| Control flow, cleanup, diagnostics | **51%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, diagnostics, and operand-list blocker boundaries exist. Broad unwind/finally/destructor/shutdown execution, exact diagnostic ordering, and executable cleanup semantics remain open. |
| Broad integrated verification | **92%** | `[##################--]` | Recent focused gates are strong and nonzero, including request/global direct handoff smoke. Full broad suites still have known current-primary failures and backend parity gaps. |

## Primary-Integrated Checklist

- [x] Primary clean/synced before this refresh at `d76828e3`.
- [x] Latest source capability baseline is `b27bbb20`.
- [x] Direct generated-C user-function calls carry root-symbol and
  request-state frame-environment requirements through the shared callable ABI.
- [x] Runtime callable table, call arguments/result/frame ABI, and selected
  runtime callable-value dispatch are integrated.
- [x] Direct generated-C user-function call consumers are integrated across
  zero/fixed/default/variadic calls and by-reference argument transport.
- [x] Generic diagnostic operation/operand-list blocker boundary is integrated.
- [x] Reference-binding operand-list blocker requirements are integrated as
  diagnostics/blockers, not executable reference binding.
- [x] Request/global direct handoff accounting is pushed; source and pages
  progress files were reported consistent by the accounting worker.

## Lane-Local Candidate Work

Lane-local work below is not counted as primary capability until integrated,
tested, committed, and pushed on primary.

| Candidate | Toward primary integration | Toward full feature | Current status |
| --- | ---: | ---: | --- |
| Dynamic callable compiler consumer | **85%** `[#################---]` | **64%** `[#############-------]` | `ready-for-primary-review`. Rebased onto `CFrameEnvironmentRequirement`, applies to current source head, focused gates pass, and owned scope is `compiler/src/codegen.rs` plus `compiler/tests/native_link.rs`. Still missing callable parity and dynamic builtin registration. |
| Assignment-lvalue operand-list requirements | **85%** `[#################---]` | **16%** `[###-----------------]` | Formal review says `go-for-primary-integrator`; diagnostic/blocker infrastructure only. Conflicts with RMW tag allocation if both are imported unchanged. |
| RMW-lvalue operand-list requirements | **80%** `[################----]` | **16%** `[###-----------------]` | Formal review says `go-for-primary-integrator`; must rebase/renumber if assignment lands first. |
| Object/ArrayAccess write blocker boundary | **80%** `[################----]` | **20%** `[####----------------]` | Formal review says `go-for-primary-integrator`; adds rejection-boundary classification, not executable `offsetSet()` / `offsetUnset()` or writeback. |
| Cleanup/unwind requirement boundary | **80%** `[################----]` | **19%** `[####----------------]` | Formal review says `go-for-primary-integrator`; diagnostic requirement boundary only, not Throwable propagation, destructor execution, or finally/unwind semantics. |
| Trait effective-method metadata | **70%** `[##############------]` | **17%** `[###-----------------]` | Reconciled proof passes in a temp checkout, but it is metadata-only and does not execute trait methods, constructors, destructors, or object lifetime cleanup. |
| Broad dirty lane evidence | **20%** `[####----------------]` | **varies** | Inventory found 86 dirty candidate/lane worktrees out of 98 inspected and zero primary contamination. Broad lanes remain evidence pools, not direct integration inputs. |

## In Progress But Not Counted

- [ ] Dynamic callable compiler consumption through the shared runtime
  callable-value ABI and `CFrameEnvironmentRequirement`.
- [ ] Assignment/RMW diagnostic operand-list packets; they need serialized
  runtime diagnostic tag allocation.
- [ ] Object/ArrayAccess write and cleanup/unwind blocker boundaries; both
  need ordering decisions and remain rejection-boundary work.
- [ ] Trait effective-method metadata; useful for later object/destructor work,
  but not executable trait semantics.
- [ ] Broad-lane extraction and review; useful only when narrowed into exact,
  current-primary candidate packets.

## Not Done

- [ ] Full executable reference binding, references/COW identity, arbitrary
  alias roots, alias-preserving write-through, foreach, and broad writeback.
- [ ] `$GLOBALS` self-cells, executable request storage/writeback,
  request/global alias parity, request foreach, includes, and variable
  variables.
- [ ] Dynamic callable primary integration, `Class::method` strings, namespace
  fallback, autoload, magic calls, callable arrays/object breadth, dynamic
  builtin registration, named/spread arguments, constructors, and return
  references.
- [ ] Closure frame-environment capture/handoff, closure object lifetime,
  lexical capture/reference binding, and non-descriptor closure execution.
- [ ] Full object model: visibility, magic properties/methods, typed/dynamic/
  static properties, traits, interfaces, ArrayAccess execution, constructors,
  destructors, and object lifetime cleanup.
- [ ] Throwable construction, throw propagation, catch matching, finally
  execution, shutdown/destructor ordering, cleanup ownership, and exact PHP
  diagnostic ordering/text.
- [ ] Backend parity for LLVM/direct assembly with the newest generated-C
  semantic packets.

## Resource And Verification Posture

- `/dev/shm`: 40G total, 24G used, 17G available, 58% used; `du -sh /dev/shm`
  reported 24G.
- `/home` filesystem: 459G total, 215G used, 226G available, 49% used;
  `du -sh /home` reported 136G.
- Memory: 43Gi total, about 35Gi available.
- Swap remains high: 23Gi used of 29Gi.
- Continue disk-backed `/tmp` target dirs, `umask 0007`,
  `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, focused nonzero gates, and no
  broad test waves until swap pressure drops.
