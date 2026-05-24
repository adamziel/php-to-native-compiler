# PHP Native Compiler Progress

Updated: 2026-05-24 23:15 CEST
Evaluation marker: `20260524T211514Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as product capability. Dirty primary WIP, probe-only commits,
dashboard-only commits, lane-local candidates, historical worktrees,
blocker-only classifiers, and status-file claims are excluded until selected,
gated, committed, pushed, and reflected here as semantic product progress.

Current live pushed primary head:
`12fe04ea docs: update progress dashboard`

Latest counted semantic/test baseline:
`b217e2b4 codegen: block destructor-observable native allocation`

Recent pushed but uncounted primary state:

- `28bb7423 docs: update progress after destructor blocker` records the
  destructor-blocker semantic commit in this dashboard.
- `2967110c codegen: expose symbol table abi probe` exposes ABI/probe
  visibility only. It does not implement generalized PHP variable
  assignment/readback through symbol-table storage.
- `59343e11 docs: update progress dashboard` is management metadata only.
- `16209c55 docs: update progress dashboard` is management metadata only.
- `12fe04ea docs: update progress dashboard` is management metadata only.

Current dirty primary WIP:

- `compiler/src/interpreter.rs`
- `compiler/tests/preg_replace_callback_builtin.rs`

The dirty `preg_replace_callback()` work is still uncounted. Current diff hash
is `3df34a396549d950bba3dbf21f9b79d7932ae9903fa304cf0c10318f28f0c3d0`, and
`git diff --check` passes on the dirty files. The work appears to be moving
toward bounded regex/callback execution, but it remains dirty and lacks a
current owner status plus current-head cargo gate evidence. The dashboard says
an active preg fixer exists, but live tmux and worker-status checks did not
confirm that owner during this review.

## Executive Read

Overall estimated progress: **85%** `[#################---]`

Executable PHP semantics: **85%** `[#################---]`

Primary has real integrated islands: descriptor closures, direct and implicit
captures, supported by-reference closure parameters and captures,
typed/default/variadic by-value closure parameters, static anonymous closures,
static arrows, callable-array public method-frame dispatch, supported
non-static closure `$this` binding, callable-object `__invoke` dispatch through
supported public method frames, runtime string-valued declared-class
construction for selected cases, and destructor-observable allocation blocking
for declared objects.

This is still not general PHP. The hardest remaining cliffs are full callable
lookup/invocation beyond selected generated-C families, closure rebinding APIs,
references/COW identity, request/`$GLOBALS` alias parity, includes, variable
variables, object visibility/magic/dynamic/static property behavior,
cleanup/unwind/finally and destructor execution, exact diagnostics, and backend
parity.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong shared value, array, string, diagnostic, reference, symbol, call-frame, object, method, callable-array, and descriptor-closure surfaces exist for selected paths. The symbol-table ABI probe is visibility only. |
| Compiler/backend consumers | **97%** | `[###################-]` | Generated-C consumes many shared ABIs; LLVM and C assembly consume selected ABI families. Object lowering, direct assembly parity, and many nested consumers remain blocked. |
| Executable PHP semantics | **85%** | `[#################---]` | Focused linked/runtime programs cover many closure/callable/object islands, but execution is still selected rather than general PHP. |
| Arrays, lvalues, references, COW | **67%** | `[#############-------]` | Useful selected lvalue/reference paths feed closure parameters and captures; full COW, arbitrary roots, foreach, object joins, property references, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-imports improved; request superglobal imports, includes, variable variables, exact unset/global alias behavior, and executable symbol-table lowering remain incomplete. |
| Calls, functions, frames | **82%** | `[################----]` | Bounded functions, descriptor closures, callable arrays, callable objects, public method frames, and constructor frames work in selected generated-C cases. |
| Objects, properties, methods | **45%** | `[#########-----------]` | Useful public declared-object subset exists and supported public `__invoke` frames are callable. Non-public/contextual visibility, overrides, interfaces/traits, broader magic methods, dynamic/static properties, destructors, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, and destructor blockers exist; broad unwind, handlers, destructor execution, shutdown flushing, and source-ordered diagnostics remain open. |
| Broad integrated verification | **84%** | `[#################---]` | Focused gates are strong for recent counted slices. Broad dirty-checkout gates are currently blocked by preg WIP, and backend parity gaps remain. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Object-offset ArrayAccess error-control classifier | **90%** `[##################--]` | **10%** `[##------------------]` | GO candidate at hash `9637c2c66471017bbd31d602913f3480d44ef5cc414e885f5807a29ebd`. Clean one-file compiler diff with current-primary apply/rustfmt/diff-check evidence. Diagnostic/blocker progress only; no runtime `ArrayAccess` dispatch. |
| Object-property reference-slot mutation | **75%** `[###############-----]` | **35%** `[#######-------------]` | Strong executable candidate at hash `c60ed1c30dc1d979da1fc44641ae4378a3629e45c42da0d9621faf10519e56e8`, but newer triage still reports scoped rustfmt failure. Needs formatting, new hash, and focused `/tmp` gate reruns. |
| Dirty `preg_replace_callback()` WIP | **40%** `[########------------]` | **20%** `[####----------------]` | Current diff-check passes and the direction is more reusable than the old WordPress-only shortcut, but the WIP is dirty, ungated at current head, and current ownership was not confirmed. Not counted. |
| Symbol-table executable storage | **20%** `[####----------------]` | **20%** `[####----------------]` | ABI/runtime helper visibility exists through `2967110c`; generalized PHP variable assignment/readback through symbol tables is still not implemented. |
| JSON/type-conversion extraction | **40%** `[########------------]` | **25%** `[#####---------------]` | Lane-local JSON line-terminator and related conversion-result work has broad consumers; extract one focused candidate and audit overlap before primary review. |
| Constructor visibility/call preflight extraction | **35%** `[#######-------------]` | **20%** `[####----------------]` | Lane-local constructor visibility preflight looks semantically useful but remains buried in cumulative call-semantics work. |
| Control-flow target-state extraction | **35%** `[#######-------------]` | **45%** `[#########-----------]` | Lane-local switch/loop target-state and recursive backend work has useful gates, but needs a narrow current-primary candidate. |
| Broad lane extraction backlog | **30%** `[######--------------]` | **30%** `[######--------------]` | Many lanes report useful generalized work, but most remain broad dirty worktrees without stable primary-base candidates. |

## Done / In Progress / Not Done

Primary-integrated and counted:

- [x] Descriptor-backed by-value closure invocation.
- [x] Direct by-value closure captures and non-static arrow implicit captures.
- [x] Untyped by-reference descriptor closure parameters.
- [x] Supported root/reference and promoted frame-local by-reference captures.
- [x] Typed/default/variadic by-value descriptor closure parameters.
- [x] Supported static anonymous descriptor closures and static arrow closures.
- [x] Supported non-static closure `$this` binding inside active object frames.
- [x] Callable-array invocation for supported public static/object method frames.
- [x] Callable-object invocation through supported public `__invoke` method frames.
- [x] Runtime string-valued declared-class `new` for constructorless and supported public-constructor classes.
- [x] Destructor-observable declared-class allocation is blocked before generated-C native allocation through declared-class metadata, hierarchy lookup, and dynamic class-name facts.
- [x] Bounded public declared-object properties, methods, statics, constructors, named `instanceof`, and same-family aggregate equality.

In progress but uncounted:

- [ ] Pushed symbol-table ABI probe `2967110c`; useful visibility, not generalized PHP symbol storage.
- [ ] Dirty bounded `preg_replace_callback()` callback-execution WIP; promising but still dirty, ungated, and ownership-unconfirmed for primary accounting.
- [ ] Lane-local `object-arrayaccess-error-control-retry` diagnostic classifier candidate at hash `9637c2c66471017bbd31d602913f3480d44ef5cc414e885f36589f5807a29ebd`.
- [ ] Lane-local `object-property-reference-slots` mutation/reference-slot candidate at hash `c60ed1c30dc1d979da1fc44641ae4378a3629e45c42da0d9621faf10519e56e8`, pending formatting and rerun gates.
- [ ] Lane-local JSON/type-conversion, constructor/call preflight, control-flow, cleanup-contract, diagnostic sequencing, comparison, object-policy, symbol/global, and array/reference candidates that need clean extraction.

Not done:

- [ ] Full callable lookup and invocation beyond selected strings, closures, arrays, and public `__invoke` objects, including magic/visibility/rebinding rules.
- [ ] Runtime `ArrayAccess` dispatch for `offsetGet`, `offsetExists`, `offsetSet`, and `offsetUnset`.
- [ ] Full references/COW identity and arbitrary alias roots.
- [ ] Request/`$GLOBALS` parity, includes, variable variables, and dynamic symbol behavior.
- [ ] General object model: non-public methods, overrides, interfaces/traits, magic methods, dynamic/static properties, typed properties, destructors.
- [ ] Complete cleanup/unwind/finally/destructor/output-buffer shutdown behavior.
- [ ] Exact/source-ordered diagnostics and warning/error continuation.
- [ ] LLVM/direct assembly parity for recent generated-C semantics.

## Recent Primary-Integrated Work

- `b217e2b4`: generated-C declared-object allocation now blocks
  destructor-observable native allocation before emitting allocation branches.
  Destructor declarations are recorded as declared-class metadata, inherited
  through class hierarchy lookup, and checked against runtime string-valued
  dynamic class-name facts. This is cleanup/unwind safety only.
- `7679dc0e`: generated-C runtime dynamic calls can invoke supported declared
  callable objects through public `__invoke` method frames.
- `53c8a283`: supported non-static regular closures and arrows created inside
  active object method/constructor frames bind `$this` through the shared
  descriptor capture/callback path.
- `8f5d8fb3` and `79496862`: supported static arrow and static anonymous
  descriptor closures reuse the shared descriptor closure stack.
- `7a43e1ac`: runtime dynamic calls can invoke syntax-valid callable arrays
  that resolve to supported generated public static/object method frames.

## Current Work Snapshot

Primary-integrated and counted:

- [x] Counted semantic baseline remains `b217e2b4`.
- [x] Overall and executable-semantics estimates remain 85%.

Pushed but uncounted:

- [ ] `28bb7423 docs: update progress after destructor blocker` is progress metadata.
- [ ] `2967110c codegen: expose symbol table abi probe` exposes helper declarations/probe calls but does not execute generalized PHP symbol storage.
- [ ] `59343e11 docs: update progress dashboard` is progress metadata.
- [ ] `16209c55 docs: update progress dashboard` is progress metadata.
- [ ] `12fe04ea docs: update progress dashboard` is progress metadata.

Dirty primary but uncounted:

- [ ] `preg_replace_callback()` WIP remains unrelated to the next candidate and not ready for primary accounting.
- [ ] Dashboard ownership of the preg fixer was not confirmed by live tmux/status checks during this review.

Lane-local but uncounted:

- [ ] `object-arrayaccess-error-control-retry` is the current GO candidate for centralized unsupported object-offset diagnostics.
- [ ] `object-property-reference-slots` is the next stronger executable candidate once formatted and re-gated.
- [ ] JSON/type-conversion, constructor visibility, and control-flow lanes should be extracted into narrow current-primary candidates before primary review.

## Review Notes

Resource pressure remains a steering constraint. `/dev/shm` is `22G` total,
`21G` used, `1.8G` available, `92%` used; `du -sh /dev/shm` reports `21G`.
`/home` has filesystem headroom at `459G` total, `166G` used, `275G`
available, `38%` used. Live `du -sh /home` did not complete within 25 seconds
and emitted permission errors in container overlay directories. New gates
should use disk-backed `/tmp` targets, `CARGO_BUILD_JOBS=1`,
`CARGO_INCREMENTAL=0`, and focused filters until shared memory recovers.
