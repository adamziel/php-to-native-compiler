# PHP Native Compiler Progress

Updated: 2026-05-27 05:45 CEST
Evaluation marker: `20260526T040843Z`
Strategy evaluator marker: `20260526T040843Z`

Accounting rule: progress counts only generalized, tested, committed, and
pushed primary work. Dirty WIP, lane-local claims, candidate artifacts,
review-only work, probe-only commits, docs-only substitutions, and broad tests
without focused proof do not increase capability bars.

Progress bars use 20 slots. One `#` is 5%. Percentages are intentionally
coarse; they do not move for narrow scaffolding unless the integrated behavior
changes the roadmap position.

## Executive Read

Overall integrated-roadmap progress: **80%** `[################----]`

Selected executable PHP semantics: **85%** `[#################---]`

Latest accounted source capability: `99d23aa4` routes generated-C receiver and
named static calls made from inside declared method frames through runtime
class-context access checks, so supported declared methods can call private
instance methods on `$this` and protected static methods on the declaring class
through shared source-call carriers while external receiver/static callers and
dynamic fallback ladders stay public-only. It keeps magic calls,
late-static binding, traits/interfaces, direct private/protected frame
shortcuts, and broad visibility/magic support blocked. Recent source commits
also route generated-C object static-receiver method calls through shared
static source-call carriers by deriving an owned class scope from object or
known class-string receivers, building the shared
`NativeCallArgumentsHandle`, preserving default/variadic frame-compatible
static method calls, and keeping `static::`, magic, traits, interfaces, full
late-static binding, and unsupported receiver shapes blocked (`3eb101aa`);
lower generated-C literal declared `Class::$prop`
reads and writes through request-owned runtime static-property storage,
including default initialization, reset, type and visibility checks, canonical
class/alias lookup, expression-result ownership, and unsupported
dynamic/static-member shape blockers (`22b3074f`), route generated-C `self::`
and `parent::` static source calls through declared class-context lookup,
shared `NativeCallArgumentsHandle` binding, and source-call carriers, including
protected/private callable-table metadata for runtime access checks
(`b4503c2e`), publish source-call signature metadata for selected runtime
callable builtins (`strlen`, string case/value helpers, `gettype`,
`is_numeric`, and `str_contains`) while keeping unsupported runtime builtins
like `count` blocked before argument construction (`7e9a237b`), let
generated-C by-reference arguments consume proven reference-return source-call
results (`defad66a`), add generated-C class alias
metadata without fake autoload success (`bae8d3fe`), route explicit by-value
generated-C function and method returns through the terminal-kind cleanup
handoff (`01da56ce`), add request-scoped runtime storage for declared static
properties (`d6798eb8`), route generated-C constructor `return <expr>;`
through an explicit diagnostic with cleanup (`bc04035b`), route selected
generated-C non-local object-property
assignments through true public-property reference owners (`cabdcde6`), route
selected generated-C property-held ArrayAccess `unset()` through true
object-property reference owners (`96ad8464`), route direct generated-C user-function
reference-return frames through native reference-returning frame signatures
(`b3e2a724`), add parser/runtime `use const` exact lookup (`e81aa43e`), route
selected property-held ArrayAccess read/write/RMW/`??=` owners through real
object-property reference owners (`6757bb43`), broaden generated-C
receiver/static method source calls to default/variadic frame-compatible
arities (`8886d9e5`), add parser/runtime `use function` exact lookup
(`c3968d0a`), allow bare `return;` in supported constructor bodies
(`bd0eafd0`), add object-property owner/fact/commit prerequisites
(`b3f16040`), LLVM user-class metadata parity (`d96cc2bb`), a generated-C
namespace/import/class-name/autoload-policy boundary (`cb8457f1`), and
terminal-kind diagnostic-result ABI support (`ac5004a3`).

Why the headline bars moved: the new source removes primary blockers
across object/class execution and backend parity: generated method frames can
write `$this` properties, selected constructors with supported bodies and bare
early returns run, constructor value returns now fail through an explicit
generated-C runtime diagnostic with cleanup, method/static source calls handle
default and variadic frame-compatible arities, `self::` and `parent::`
static source calls now run through class-context source-call carriers,
declared method-frame calls can now invoke supported private/protected
receiver/static methods through runtime class-context lookup-plus-invoke
carriers,
object static-receiver calls now derive runtime receiver scope and run through
shared static source-call carriers for exact/default/variadic
frame-compatible declared static methods,
literal declared `Class::$prop` reads and writes now execute through
request-owned runtime static-property storage instead of being rejected,
interpreter function imports resolve exactly,
interpreter const imports resolve exactly before namespace/global fallback,
direct generated-C user-function frames can return references through shared
source-call carriers and by-reference consumer paths, proven reference-return
direct/dynamic/receiver/static source calls can feed by-reference arguments
through shared carriers, selected runtime callable builtin signatures now feed
dynamic source-call argument binding while unsupported builtins are rejected
before side-effecting arguments are built, selected property-held
ArrayAccess owners now cover `unset()` through the same reference-owner commit
path as reads/writes/RMW, and selected external object-property assignments
reuse true public-property reference owners instead of temporary mutation
shortcuts,
explicit generated-C by-value function/method returns now pass through the
terminal-kind cleanup-transfer ABI before returning values to existing frame
callers,
runtime static-property storage now has request-scoped default initialization,
type/visibility enforcement, reference identity preservation, and request reset
coverage for later backend producers,
generated C handles parser-resolved namespace/import class policy without fake
autoload success and now records generated-C class aliases through normalized
metadata/canonical lookup boundaries, while missing-source autoload remains
blocked,
LLVM can declare and query user-class metadata through the
shared runtime ABI, terminal transfer now carries return/throw/exit kind, and
object-property owner facts now drive selected property-held ArrayAccess
production. The bars remain far from 100% because nested/append/increment
ArrayAccess owners, dynamic method names, late-static and magic method shapes,
named/spread arguments, native function/const-import lowering, broader
constructor execution, late-static binding,
arbitrary alias roots, LLVM and broader generated-C static-property producers,
typed-property
lowering, exceptions/cleanup, full SPL
autoload, visibility/magic breadth, and backend parity are still open.

Current critical path to 100%:

1. Extend expression-owned `NativeCallResultHandle` carriers and production
   source-call lowering over the lookup-plus-invoke ownership helpers,
   including remaining dynamic/late-static/magic method shapes,
   constructor allocation, closure argument handles, broader by-reference alias
   transfer, and spread ownership.
2. Continue migrating expression, statement, terminal, cleanup, lvalue,
   reference, and call-argument lowering onto produced
   `NativeDiagnosticResult` operands.
3. Route value/reference/return/deferred-cleanup consumers through the shared
   diagnostic-result and call-result carrier stack.
4. Broaden the integrated object-property owner/fact/commit boundary from
   selected non-local assignments and property-held ArrayAccess
   read/write/RMW/unset into nested/append/increment forms,
   references/COW, arbitrary alias-root writeback, and object/static/dynamic/
   typed property storage.
5. Implement real exception/Throwable propagation, catch/finally/destructor/
   shutdown cleanup, source-ordered diagnostics, and custom handler behavior.
6. Broaden namespace/function/const import production lowering, namespace
   fallback, autoload, broader aliases, visibility/magic, constructors, named/spread
   arguments, descriptor/method/closure return references, and backend parity.

## Roadmap Bars

| Workstream | Integrated | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **90%** | `[##################--]` | Strong selected-path value, byte-string, array, reference, symbol, callable, call-frame/result, diagnostic-result, terminal-kind, request-state, lvalue, ArrayAccess, class metadata value, request-scoped static-property storage, generated-C class alias metadata, function/const-import exact lookup, and autoload-policy boundary surfaces. Remaining gaps include arbitrary alias transfer, full autoload, namespace fallback, magic calls, closure frame handoff, cleanup/unwind parity, and broader lookup parity. |
| Compiler/backend consumers | **85%** | `[#################---]` | Generated C has the freshest consumers for calls, callable facts, selected object/class metadata, namespace/import class policy, selected ArrayAccess/lvalue paths, selected non-local object-property assignment owner commits, declared static-property read/write storage, value-result casts, explicit by-value return terminal handoff, diagnostic-result family consumers, discarded statement-expression diagnostic operands, echo/print output diagnostic operands, and control-transfer cleanup report bridging. It now rejects function and const imports explicitly at the production boundary. LLVM shares user-class metadata declaration/exists routing plus the discarded-expression, output operand, and cleanup report bridge paths, while direct assembly still lags newer object-offset/lvalue/static-property/runtime ABIs and most semantic result operands remain unmigrated. |
| Executable PHP semantics | **85%** | `[#################---]` | Many executable islands exist, including bounded method/static source-call production, generated method-frame `$this` property assignment, selected non-local object-property assignment commits, selected declared literal static-property reads/writes, selected constructor bodies with bare early returns, selected property-held ArrayAccess read/write/RMW/`??=`/unset owners, namespace/import class policy, interpreter function/const-import exact lookup, and value-returning class metadata consumers, but broad assignment/RMW/writeback, references/COW, nested/append ArrayAccess breadth, dynamic/static-property shapes, cleanup/unwind/finally/destructors, exact diagnostics, native function/const-import lowering, and backend parity remain open. |
| Strings and byte semantics | **60%** | `[############--------]` | Byte-backed values and selected byte-preserving string-array slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **80%** | `[################----]` | Selected lvalue/reference-source extraction, ReferenceSlot owner facts, object-property owner/fact/commit prerequisites, selected non-local object-property assignment commits, reference-cell predicates, membership helpers, RMW array-lvalue owner/writeback, selected direct/generated-object ArrayAccess RMW/`??=` paths, and selected property-held ArrayAccess read/write/RMW/`??=`/unset owners are integrated. Nested/append/increment ArrayAccess production, arbitrary alias roots, foreach breadth, broader writeback, and full COW remain incomplete. |
| Symbols, globals, request state | **70%** | `[##############------]` | Selected globals, root-symbol consumers, active symbol-table consumers, request-key blockers, append-shaped symbol reference-source materialization, direct generated-C request-state frame handoff, and dynamic user-function handoff proof exist. `$GLOBALS` self-cells, closure request-state handoff, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **90%** | `[##################--]` | Runtime callable table/value dispatch, selected runtime builtin source-call signatures/blockers, call arguments/frame/result ABI, conditional handoff, generated-C direct/dynamic callable consumers, declared-method registration/wrapper frames, callable return facts, by-reference argument transport, descriptor closures, closure returns, request-state frame handoff, access-context lookup ABI, lookup-plus-invoke exactly-once argument ownership helpers, source-call result carrier selectors, selected production source-call carrier emission, direct generated user-function lookup-plus-invoke production, direct generated user-function reference-return frames, explicit by-value return terminal handoff, method/static source-call target operands, method/static source-call binding operands, method/static signature fallback selection, selected direct/dynamic/receiver/static/self/parent reference-return source-call alias transfer into by-reference arguments, executable receiver/static/self/parent/object-static method source-call production for exact, default, and variadic frame-compatible arities where class context or receiver scope is known, explicit generated-C constructor value-return diagnostics, and interpreter `use function`/`use const` exact lookup are integrated. Unknown runtime callables, dynamic/late-static/magic method shapes, broader builtin/native/inherited/trait/interface signature metadata, broader by-reference alias transfer, native function/const-import lowering, named/spread breadth, descriptor/method/closure and broader return references, broader constructor allocation/execution, cleanup/unwind, and backend parity remain open. |
| Objects, properties, methods | **85%** | `[#################---]` | Selected object metadata, value-returning class metadata consumers, LLVM/generated-C user-class metadata consumers, generated-C namespace/import class policy, generated-C class alias metadata and canonical class/member lookup, public property reference-source extraction, method-frame `$this` property assignment, selected non-local object-property assignment commits, object-property owner/fact/commit prerequisites, object-property reference-slot mutation, selected property-held ArrayAccess read/write/RMW/`??=`/unset owners, request-scoped runtime static-property storage plus generated-C declared literal static-property read/write producers, generated-C ArrayAccess consumers for compiler-known generated objects, dynamic generated class-name producers, object-call argument handles, declared-method callable-table publication, bounded executable receiver/static/self/parent/object-static method production through access-context source-call carriers, selected constructor bodies with bare early returns and explicit value-return diagnostics, allocatable class metadata, user-class metadata registry consumers, and access-context preflights exist. Nested/append ArrayAccess production, magic/unknown-runtime-dynamic-call/clone/static-property breadth, full late-static binding, broader class-alias/autoload parity, broader visibility parity, generated-C/LLVM typed-property lowering, destructors, interfaces/traits execution, references/COW, broader constructor allocation/execution, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **70%** | `[##############------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostic blockers, owned diagnostic-result list contracts, consumer contracts, backend family consumers, deferred-cleanup blockers, control-transfer cleanup result consumers, terminal cleanup transfer ABI, terminal-kind ABI, explicit by-value return terminal handoff, cleanup-frame producers/source metadata/report bridges, cleanup-frame stack aggregation, cleanup-frame enqueue validation, try-body call-boundary preflight, report sinks, continuation helpers, discarded statement-expression operands, and echo/print output operands exist. Broad unwind/finally/destructor/shutdown execution, cleanup result production from real control flow, executable reference binding, remaining semantic diagnostic-result producer migration, and source-ordered diagnostics remain open. |
| Broad integrated verification | **75%** | `[###############-----]` | Focused gates around recent source work are strong, with several primary integration gates now covering linked generated-C class/method/constructor programs, LLVM class metadata routing, terminal-kind ABI behavior, and owner-boundary regressions. Broad verification is still constrained by lane extraction cost, stale candidate expectations, heavy formatter/log pressure, and backend parity gaps. |

## Recently Accounted Source Work

| Commit | Capability | Proof shape |
| --- | --- | --- |
| `99d23aa4` | Generated C now carries declared-class caller scope into receiver-method and named static source-call wrappers emitted from declared method frames, allowing supported private `$this->method()` and protected `Class::method()` calls through runtime class-context access checks while keeping external receiver/static callers, dynamic fallback ladders, callable-object/array shortcuts, magic calls, late-static binding, traits/interfaces, and direct private/protected frame dispatch blocked. | Runtime access-context lookup diagnostics, generated-C source proof for class-context receiver/static carriers, linked executable private/protected class-context program, object-static source-call regressions, method/static/default/variadic/self/parent carrier regressions, runtime builtin signatures, static-property storage/production regressions, source-call byref regressions, constructor value-return diagnostics, non-local object-property regressions, fmt, diff check. |
| `3eb101aa` | Generated C now routes object static-receiver calls such as `$obj::method()` and known class-string receiver forms through shared static source-call carriers, deriving an owned runtime class scope from object/class-string receivers, building `NativeCallArgumentsHandle` once, supporting exact/default/variadic frame-compatible declared public static methods, and keeping non-static methods, `static::`, magic, traits/interfaces, full late-static binding, and unsupported receivers blocked. | Object-static signature contract proofs, runtime receiver-scope helper proof, generated-C source proofs for object-static exact/default/variadic carriers, linked object-static executable proofs, adjacent receiver/static/default/variadic/self/parent source-call regressions, runtime builtin signature/dynamic builtin regressions, static-property production/storage regressions, source-call byref regressions, constructor value-return diagnostics, non-local object-property and class-alias regressions, fmt, diff check. |
| `22b3074f` | Generated C now lowers declared literal `Class::$prop` reads and writes for compiler-known classes through request-owned runtime static-property storage, registering defaults/reset handles, preserving assignment expression results, routing type/visibility/canonical class lookup through the runtime ABI, and keeping dynamic class/property, `self::`/`parent::`/`static::`, object-static, magic, compound/unset/reference, LLVM, and assembly static-member forms blocked. | Runtime static-property suite, generated-C static-property source proof, linked executable static-property program, unsupported dynamic-shape blocker proof, self/parent static regressions, builtin-signature/runtime dynamic builtin regressions, class-alias regressions, constructor value-return diagnostics, non-local object-property owner regressions, fmt, diff check. |
| `b4503c2e` | Generated-C `self::` and `parent::` static source calls now carry active declared class and parent-class context, publish protected/private static method metadata for runtime access checks, and invoke through class-context static source-call carriers while keeping direct static ladders public-only and `static::`/late-static binding blocked. | Self/parent source-call unit proof, generated-C source proof, linked self/parent executable proof, builtin-signature regressions, source-call byref regressions, constructor value-return diagnostics, non-local object-property owner regressions, static-property runtime regressions, fmt, diff check. |
| `7e9a237b` | Selected runtime callable builtins now expose arity, by-reference, return, and source-call support metadata for dynamic source-call argument binding; unsupported runtime builtins such as `count` remain blocked at lookup time before argument construction, without fake `count()` semantics. | Builtin signature metadata unit proof, runtime callable signature/dispatch/boundary proof, generated-C known/runtime dynamic builtin source proof, linked runtime dynamic builtin executable proof, unsupported builtin side-effect guard, source-call byref regressions, terminal-return, class-alias, non-local object-property, and static-property regressions, fmt, diff check. |
| `defad66a` | Generated-C by-reference arguments can now materialize proven reference-return source-call results from direct user functions, dynamic callable identities, receiver methods, and named static methods through shared source-call carriers while keeping by-value produced calls on the alias-transfer blocker. | Source-call byref generated-source proof, linked byref alias executable and cleanup-failure proof, by-value produced-call blocker proof, direct reference-return regressions, terminal-return regressions, class-alias regressions, constructor value-return regressions, non-local object-property and static-property runtime regressions, fmt, diff check. |
| `bae8d3fe` | Generated C can register class aliases for already-declared metadata targets, resolve canonical class/member metadata through aliases, preserve alias conflicts and missing-source autoload boundaries, and keep LLVM direct `class_alias()` lowering rejected. | Runtime alias metadata tests, generated-C source and linked executable class-alias metadata proof, missing autoload boundary proof, namespace alias policy regressions, LLVM rejection/interpreter regressions, terminal-return regressions, static-property runtime regression, constructor value-return regressions, non-local object-property and property-held ArrayAccess owner regressions, fmt, diff check. |
| `01da56ce` | Generated-C explicit by-value function and method returns now produce terminal-surface `NativeDiagnosticResult` values, transfer return terminals through cleanup handling, and extract owned return values back into existing native-value and closure-value frame contracts while preserving reference-return and constructor value-return paths. | Terminal-return generated-source proof, runtime return-handoff ABI proof, linked executable function/method return proof, constructor value-return regressions, direct reference-return regressions, property-held ArrayAccess owner/unset regressions, non-local object-property owner regressions, static-property runtime storage regression, fmt, diff check. |
| `d6798eb8` | Runtime declared static properties now have request-scoped default initialization and reset, per-class/per-property visibility shadowing, assignment type checks, reference identity preservation, and late-static receiver resolution through storage helpers. Generated-C/LLVM static-property production lowering remains blocked. | Runtime static-property storage tests, existing non-local assignment/unset owner boundary regressions, property-held ArrayAccess owner/unset executable regressions, constructor value-return diagnostic regression, fmt, diff check. |
| `bc04035b` | Generated-C constructor `return <expr>;` no longer pre-rejects during constructor validation; generated constructor frames return status `2`, free the returned value and receiver at the call site, and report the explicit `constructor value returns are not implemented` diagnostic through the existing failure cleanup path while preserving public/non-static and reference/typed/global-import blockers. | Constructor value-return generated-source proof, linked executable diagnostic proof, constructor dispatch/blocker regressions, direct reference-return regression, property-held ArrayAccess owner/unset regressions, non-local object-property owner regressions, fmt, diff check. |
| `cabdcde6` | Generated-C non-local object-property assignments now materialize literal and single-known dynamic public-property owners through reference slots, clone the assigned value for expression result semantics, commit replacements through reference writeback, update owner facts, and keep unknown dynamic, nested, and static property assignment shapes blocked. | Generated-C source proof for owner/reference commit shape, linked executable proof across literal, replacement, and dynamic property holders, reference-backed dynamic property/value proof, unsupported-shape rejection proof, property-held ArrayAccess owner/unset regressions, fmt, diff check. |
| `96ad8464` | Generated-C property-held ArrayAccess `unset()` now materializes literal and single-known dynamic object-property holders through public-property reference owners, invokes the ArrayAccess write/unset ABI, commits the mutated holder through reference writeback, and keeps nested/non-direct/unknown dynamic owners blocked. | Generated-C source proof for write/unset ABI and owner commit shape, linked executable proof across literal and dynamic property holders, unsupported owner-shape rejection proof, fmt, diff check. |
| `b3e2a724` | Generated-C direct user-function reference-return frames now accept by-reference parameter return sources, use native reference-returning frame signatures, preserve callable-wrapper reference ownership, route reference consumers through source-call reference carriers, and keep by-value return sources rejected. | Focused reference-return frame contract tests, alias-transfer result-vector source proof, by-value source rejection proof, source-call carrier selector regressions, direct user-function frame source/link regressions, fmt, diff check. |
| `e81aa43e` | Parser/runtime `use const` imports carry import-kind metadata, resolve arbitrary aliases/default aliases exactly before namespace/global fallback, preserve non-imported namespace-then-global fallback, reject aliases that conflict with existing imports or same-namespace constant declarations in either order, and keep generated C/LLVM at explicit production rejection boundaries. | Namespace-resolution proof across const aliases, default aliases, exact-missing no-fallback, non-import fallback, declaration/import conflict guards, class/function import regressions, generated-C and LLVM rejection, unsupported const-use CLI snapshot, focused global-constant rejection, fmt, diff check. |
| `6757bb43` | Generated-C property-held ArrayAccess owners materialize literal and single-known dynamic object-property values through public-property reference owners, use constructor `$this` property fact summaries for direct `new` assignments, commit writes/RMW/`??=` through reference writeback, and keep unknown/nested/append/increment owner shapes blocked. | Codegen owner/fact unit proof, generated-C source proof for property-held read/write/RMW/`??=`, linked executable proof across literal and dynamic property holders, unsupported-owner regression, existing ArrayAccess RMW/`??=` regressions, `$this` assignment and constructor regressions, method/static default-variadic regression, fmt, diff check. |
| `8886d9e5` | Generated-C receiver-method and named static-method source-call production now accepts frame-compatible default and variadic arities by synthesizing omitted defaults and variadic packs into the shared `NativeCallArgumentsHandle` before invoking existing source-call carriers. | Contract unit proof for exact/forward versus default/variadic frame plans, generated-C source proof for receiver/static carrier paths and variadic packing, linked executable proof across receiver/static default and variadic calls, existing exact-arity method/static and constructor regressions, fmt, diff check. |
| `c3968d0a` | Parser/runtime `use function` imports now carry import-kind metadata, resolve arbitrary aliases/default aliases exactly before namespace fallback, reject alias conflicts with same-namespace function declarations/imports, and keep generated-C at an explicit production rejection boundary. | Runtime namespace-resolution proof for aliases, default aliases, non-imported fallback, exact-missing no-global-fallback, alias-conflict guards, generated-C rejection, class-import regressions, linked namespace alias/class policy regressions, constructor regression, fmt, diff check. |
| `bd0eafd0` | Generated-C declared constructor validation allows bare `return;` early exits while keeping `return <value>` blocked, using the existing method-frame constructor dispatch and declared allocation paths. | Linked constructor executable covering default args, required args, `$this` assignments, guarded bare returns, dynamic constructor allocation/dispatch regressions, unsupported constructor value-return guard, method-frame `$this` assignment regressions, object-model dynamic class-name proof, fmt, diff check. |
| `b3f16040` | Structured object-property owner/fact/commit prerequisite boundary tracks literal and dynamic property writes without stringified paths, materializes dormant object-property owners through public-property reference slots, commits replacements through reference writeback, and preserves ArrayAccess owner cleanup ownership. | Codegen unit proof for owner materialization, fact invalidation, and commit cleanup; focused generated-C ArrayAccess owner-boundary/rejection/`??=` regressions; fmt, diff check. |
| `d96cc2bb` | LLVM user-class metadata parity declares top-level user classes, parents, methods, and properties through the shared runtime registry and routes LLVM `class_exists()`, `method_exists()`, and `property_exists()` through the metadata-exists ABI. | LLVM IR declaration/call proof, LLVM assembly acceptance, object-model metadata ABI tests, class-boundary expectation updates, generated-C metadata regressions, fmt, diff check. |
| `cb8457f1` | Generated-C accepts parser-resolved namespace/import class policy, named class constants, leading-backslash/case-normalized dynamic class matching, and `class_exists()` autoload policy without pretending autoload succeeded. | Runtime class-name/autoload-policy tests, generated-C source proof, linked namespace/import executable, exact namespaced `class_exists` precedence, missing default-autoload terminal boundary, metadata registry regression, fmt, diff check. |
| `ac5004a3` | Runtime/compiler diagnostic results can carry return/throw/exit terminal kinds through cleanup transfer, report sinks, terminal-list inspection, and backend declaration surfaces without enabling production return/throw/exit lowering. | Runtime ABI tests across all terminal kinds, terminal cleanup masking, invalid/missing ownership blockers, report sinks, backend surface tests, existing terminal transfer/report regressions, fmt, diff check. |
| `04b18506` | Generated non-static method and constructor frames can assign literal or dynamic `$this` properties through the shared object-property mutation ABI while preserving receiver/replacement ownership, diagnostics, cleanup, and assignment-expression value semantics. | Generated-C source proof updated for source-call carrier callers, linked `$this` assignment success/failure programs, declared method and constructor executable regressions, object-property mutation ABI regression, fmt, diff check. |
| `c8aeb771` | Bounded generated-C receiver-method and named static-method source-call production runs through shared target operands, binding operands, signature fallback contracts, source-call carriers, and runtime diagnostics when exact frame-compatible signatures are known. | Generated-C carrier-shape proof, linked executable method/static proof, signature fallback carrier regression, native source-call carrier regressions, declared static dispatch guard, fmt, diff check. |
| `4e5e2709` | Runtime and generated-C value-returning class metadata consumers for `get_parent_class()`, `class_parents()`, `get_declared_classes()`, `get_class_methods()`, and `get_class_vars()` over registered user classes and selected core class metadata. | Runtime ABI test across parent chain, declared classes, method/property visibility, and ownership; generated-C source proof; linked executable proof including user and core metadata; direct-call diagnostics regression; fmt, diff check. |
| `69526631` | Generated-C dynamic source-call reference results can be materialized as by-reference arguments when the callee is proven reference-returning through scoped callable-string metadata or native callable identity summaries, while unsupported produced calls still hit the existing blockers. | Generated-C source proof across direct and dynamic consumers, multiple reference-return methods, multiple symbols, and multiple by-reference positions; linked executable success and cleanup-failure proofs; existing adjacent alias-transfer/runtime ABI regressions; fmt, diff check. |
| `2cd4f628` | Runtime user-class metadata registry plus generated-C declaration and class/member metadata-exists consumers for declared user classes, inherited methods/properties, and runtime value operands. | Runtime registry test across declared class, parent, inherited method/property, and diagnostics; generated-C source proof; linked executable proof for class/member/property existence; package checks; fmt, diff check. |
| `8d5f3715` | Method/static signature fallback contract classifies declared receiver/static method metadata as known scoped callable-string signatures or runtime fallback, and feeds the selection through shared source-call binding operands and carriers without adding executable method/static production lowering. | Contract classification test across known, missing, arity-mismatch, heterogeneous, and runtime-dynamic metadata; binding/carrier test across receiver, static, and runtime fallback selections; existing source-call emitter/carrier regression; fmt, diff check. |
| `53bef000` | Source-call binding operands compose method/static target preinvoke cleanup, owner cleanup, signature-driven by-reference argument binding, and the shared `NativeCallArgumentsHandle` path across receiver-method and static-method carrier families. | Source-call emitter carrier test across direct, callable-value, materialized callable, receiver-method, and static-method targets with by-reference signature handoff; carrier selector tests; fmt, diff check. |
| `a7054ed1` | Runtime terminal cleanup transfer consumes a pending terminal result plus aggregated cleanup results, preserves terminal values across non-terminal cleanup, releases them after terminal cleanup, and exposes the ABI to LLVM/generated C without enabling production lowering. | Runtime ABI tests across int/string/array terminal values, mixed cleanup lists, terminal cleanup, and ownership blockers; backend declaration regression; existing cleanup sequencing regression; fmt, diff check. |
| `d67c7f14` | Receiver-method and static-method source-call target operands now compose access-context lookup-plus-invoke helpers with `NativeSourceCallResultCarrier` and the shared exactly-once `NativeCallArgumentsHandle` path, while separating pre-invocation failure cleanup from post-invocation auxiliary cleanup. | Source-call emitter carrier test across direct, callable-value, materialized callable, receiver-method, and static-method targets; carrier selector tests; lookup-plus-invoke declaration test; fmt, diff check. |
| `4d6f0f1e` | Cleanup-frame enqueueing validates producer surfaces and rejects terminal-source value ownership before LLVM/generated C backend emission while preserving the existing cleanup-frame stack/report path. | Cross-backend cleanup-frame surface and terminal-value blocker test, stack aggregation regression, cleanup report bridge regression, fmt, diff check. |
| `8ebeae19` | Cleanup frames can aggregate nested frame stacks in innermost-first unwind order while preserving `NativeDiagnosticCleanupFrameSource` metadata and feeding the existing cleanup report bridge across LLVM and generated C. | New stack aggregation test across LLVM/generated C, cleanup-frame operand/source regression, cleanup report bridge regression, runtime cleanup sequencing, fmt, diff check. |
| `964e3e2b` | Direct generated-C user-function calls route through direct named lookup-plus-invoke source-call carriers and reusable target operands while preserving the shared call-arguments ownership path. | Carrier/emitter and carrier-selector unit tests, generated-C link/run proof across zero/fixed/default/variadic arities and by-reference argument transport, fmt, diff check. |
| `0430efcc` | Cleanup frames carry terminal/control-transfer/deferred-cleanup source metadata and report helpers now consume frames instead of raw result slices. | Compiler cleanup-frame source tests across accepted/rejected operands and both backends, existing report regression, runtime cleanup sequencing, fmt, diff check. |
| `75f20f3f` | Selected generated-C production source-call paths build call arguments once and invoke dynamic callable values, scoped callable-string reference assignments, and materialized direct user-function callables through source-call carriers. | Carrier/emitter unit tests, source-call selector tests, generated-C link/run proof for dynamic callable values and direct user-function frames, fmt, diff check. |
| `d26c64f7` | LLVM/generated-C cleanup frames queue cleanup-surface diagnostic-result operands and reject non-cleanup surfaces before reporting through the control-transfer cleanup bridge. | Compiler cleanup-frame test across value, diagnostic, null, rejected non-cleanup surfaces, both backends, existing bridge regression, runtime cleanup sequencing, fmt, diff check. |
| `50d19f99` | LLVM/generated-C cleanup report bridge consumes already-produced cleanup diagnostic-result operands through the control-transfer cleanup consumer and diagnostics-only report sink. | Compiler bridge test across value, diagnostic, null, non-empty, and empty cleanup lists; fmt, diff check. |
| `7891fcf3` | Runtime converter and compiler selectors compose source-call target helpers with owned-result, value, reference, discard, and diagnostic-result consumers. | Runtime value/reference/failure/null conversion test, compiler carrier selector/declaration tests, fmt, diff check. |
| `8b53ed25` | Control-transfer cleanup `NativeDiagnosticResult` lists consume already-produced cleanup operands in source order, preserve diagnostics, free owned values, and stop after terminal diagnostics. | Runtime ABI shape tests across value, warning, terminal, null-entry, null-list, and empty-list inputs; compiler backend family consumer tests, fmt, diff check. |
| `b7e6f117` | Direct callable, receiver-method, and static-method lookup-plus-invoke helpers consume `NativeCallArgumentsHandle` exactly once and expose compiler helper selection/declarations. | Runtime ownership tests across lookup/invoke success and failure, callable access-context regressions, compiler selector/declaration tests, fmt, diff check. |
| `a991cf34` | Echo operands and statement-form `print` lower into owned `NativeDiagnosticResult` output operands and report/free through the shared echo sink. | Compiler output-operand tests, runtime echo-sink diagnostic test, executable generated-C link/run proof, fmt, diff check. |
| `dcdd330f` | Discarded expression statements lower into owned `NativeDiagnosticResult` operands and report/free through diagnostics-only sinks in LLVM and generated C. | Compiler result-operand tests, `native_runtime_abi` sink tests, executable generated-C link/run proof, fmt, diff check. |
| `5902369c` | Shared callable access-context lookup, allocatable class metadata, diagnostic-result continuation helpers, and stderr/echo report sinks. | Runtime focused tests, compiler ABI declaration tests, `native_runtime_abi` tests, fmt, diff check. |
| `950a17fe` | LLVM/generated-C diagnostic-result family consumers over already-produced operand lists. | Family selector, backend emission, empty-list, missing-runtime-ABI, fmt, diff check. |
| `81c60f38` | Runtime diagnostic-result consumer contracts for value-required and cleanup families. | Result-list ownership, terminal preservation, null/empty list behavior. |
| `099b76fc` | Owned `NativeDiagnosticResult` value/diagnostic/null result contract. | Value, diagnostic, null, list cleanup, adjacent blocker behavior. |
| `08d00fe1` | Conditional call-frame result handoff for short-ternary/null-coalescing families. | Success/diagnostic preservation and cleanup-sensitive blockers. |
| `7fb9db15` | Shared object metadata/type-introspection builtin preflight. | `class_exists`, `property_exists`, `is_a`, direct-call cleanup diagnostics. |
| `a3826e2f` | Generated-C dynamic instance method name normalization through native value helper. | Runtime lookup normalization and generated-C helper selection. |
| `3ac78d8b` | Shared generated-C object-call argument handles. | Constructor, method, static, callable-array, invokable-object argument families. |
| `73195f96` | Native value-result cast diagnostics. | Array-to-string warnings over direct and compound value-result paths. |
| `0bebd2e9` | By-reference alias-transfer result boundary for produced call results. | Direct generated user-function call consumers for echo/print/discard. |
| `b3d90dbc` | Runtime/compiler reference-cell predicate and membership boundaries. | `isset`, `empty`, truthiness, `array_key_exists` over value/reference subjects. |
| `05214fd4` | Compiler-known declared-method callable identities and return summaries. | Public/static/object receiver policy, callable identities, return-summary resolution. |

## Active Roadmap Items

Primary-integrated capability and candidate/lane-local work are separated.

| Item | Primary Integrated | Candidate Readiness | Toward Full Feature | Status |
| --- | ---: | ---: | ---: | --- |
| Diagnostic-result carrier stack | **100%** `[####################]` | **100%** `[####################]` | **60%** `[############--------]` | Runtime/result contracts, family consumers, continuation helpers, report sinks, discarded statement-expression operands, echo/print output operands, source-call result conversion, control-transfer cleanup result consumers/report bridges, and cleanup-frame producers are integrated. Terminal producers, semantic cleanup result production from real control flow, lvalue, reference, RMW, and call-argument operands still need exact ownership and ordering migrations. |
| Callable access context and class metadata | **100%** `[####################]` | **100%** `[####################]` | **65%** `[#############-------]` | Shared runtime access-context policy, lookup-plus-invoke argument ownership, source-call result carrier selectors, selected production source-call carrier emission, direct generated user-function lookup-plus-invoke production, direct user-function reference-return frame consumers, bounded generated-C method/static source-call production including default and variadic frame-compatible arities, method/static source-call target and binding operands, method/static signature fallback selection, allocatable-class metadata, generated-C user-class metadata-exists consumers, and value-returning class metadata consumers are integrated for selected function/method/static/constructor/class lookup preflights. Constructor execution, dynamic method names, named/spread arguments, runtime/builtin/inherited/trait/interface signature metadata, closure argument-handle ownership, namespace/function/const fallback, autoload, magic, and full visibility parity remain open. |
| ArrayAccess compiler consumers | **100%** `[####################]` | **100%** `[####################]` | **65%** `[#############-------]` | Generated-C direct-object/direct-variable read, `isset`, `empty`, `??`, write, append, unset, compound assignment, `??=`, and selected property-held literal/single-known dynamic object-property read/write/RMW/`??=`/unset owners are integrated for compiler-known generated declared `ArrayAccess` objects. Nested property owners, append RMW, increment/decrement, reference-returning `offsetGet`, references/COW, cleanup/unwind, and backend parity remain open. |
| ReferenceSlot owner facts | **100%** `[####################]` | **100%** `[####################]` | **50%** `[##########----------]` | Compiler-visible native reference handles can recover facts, source owners, and commit writeback for selected variable, non-local object-property assignment, and property-held ArrayAccess paths. Arbitrary alias roots, request/superglobal path facts, broader property-held reference binding, closure callback fact transport, references/COW, and backend parity remain open. |
| Callable identity return summaries | **100%** `[####################]` | **100%** `[####################]` | **60%** `[############--------]` | Generated functions, declared methods/static methods, descriptor closures, known strings, definite `__invoke` objects, compiler-known callable arrays, and selected direct user-function reference-return frames can publish or consume selected return facts. Unknown runtime callables, builtins, non-descriptor closures, recursive summaries, descriptor/method/closure reference returns, property/magic producers, references/COW, and backend parity remain open. |
| Dynamic object/interface fact carrier | **100%** `[####################]` | **100%** `[####################]` | **65%** `[#############-------]` | Generated declared objects, known dynamic class-name `new`, copies, gotos, branches, generated-callable returns, descriptor closures, known string/invokable/callable-array summaries, compiler-visible reference slots, and selected declared static-property producers feed existing object/interface consumers. Broader properties, clones, dynamic/static property shapes, arbitrary symbols, unknown runtime callables/arrays, and non-descriptor closures remain open. |
| Cleanup/unwind execution | **30%** `[######--------------]` | **30%** `[######--------------]` | **30%** `[######--------------]` | Requirement/preflight boundaries, cleanup result consumers, cleanup report bridges, terminal cleanup transfer ABI, cleanup-frame producer queues, cleanup-frame source metadata, nested cleanup-frame stack aggregation, and cleanup-frame enqueue validation are integrated. Actual exception propagation, catch/finally/destructor/shutdown execution, production cleanup operand enqueueing from real control flow, terminal-kind lowering, and object lifetime cleanup are still not implemented. |
| Broad dirty lane extraction backlog | **0%** `[--------------------]` | **35%** `[#######-------------]` | **35%** `[#######-------------]` | Dirty call, diagnostic, object, control-flow, symbol, byte/string, and array lanes remain evidence pools until split into fresh current-head candidates with focused proof. |

## Done

- Runtime callable table plus call arguments/frame/result ABI.
- Runtime callable-value dispatch for selected function names, callable arrays,
  descriptor closures, inherited methods, bound receivers, and object
  `__invoke`.
- Runtime lookup-plus-invoke helpers for direct callable, receiver-method, and
  static-method families consume `NativeCallArgumentsHandle` exactly once
  across lookup failure, invoke failure, result handoff, and discard/value/
  reference consumers.
- Source-call result carrier selectors compose direct named, receiver-method,
  static-method, materialized-callable, and callable-value targets with owned
  result, value, reference, discard, and diagnostic-result consumers.
- Generated-C receiver-method and static-method source-call target operands
  compose access-context lookup-plus-invoke helpers with shared source-call
  carriers and distinguish pre-invocation failure cleanup from post-invocation
  auxiliary cleanup.
- Source-call binding operands compose method/static targets with signature-
  driven by-reference argument binding over the shared
  `NativeCallArgumentsHandle` emitter.
- Selected runtime callable builtin signatures feed dynamic source-call
  argument binding and return facts; unsupported runtime builtins such as
  `count` stay blocked before argument construction.
- Method/static signature fallback selection classifies declared receiver and
  static method metadata as known scoped callable-string signatures or runtime
  fallback, then feeds that selection through shared source-call binding
  operands and carriers.
- Bounded generated-C receiver-method and named static-method calls execute
  through shared source-call target operands, binding operands, result
  carriers, and runtime diagnostics when exact frame-compatible signatures are
  known.
- Generated-C receiver-method and named static-method calls with supported
  default or variadic declared method parameters synthesize frame-shaped call
  arguments through the shared `NativeCallArgumentsHandle` and source-call
  carriers.
- Selected generated-C production source-call paths build
  `NativeCallArgumentsHandle` once and invoke dynamic callable values,
  scoped callable-string reference assignments, and direct generated
  user-function lookup-plus-invoke calls through source-call carriers.
- Selected generated-C by-reference call arguments can materialize proven
  reference-return direct, dynamic, receiver-method, and named static-method
  source-call results through the shared source-call reference carrier and
  `NativeCallArgumentsHandle` push-reference path.
- Direct generated-C user-function calls and dynamic generated-C callee
  expressions through shared runtime callable lookup/invocation.
- Direct generated-C user-function reference-return frames can return
  by-reference parameter sources through native reference handles and feed
  reference consumers through source-call carriers.
- Explicit by-value generated-C function and method returns route through the
  terminal-kind `NativeDiagnosticResult` cleanup handoff before returning
  values to existing native-value and closure-value frame contracts.
- Generated declared-method callable-table registration and wrapper frames.
- Generated-C user-class declarations register class, parent, method, and
  property metadata, and generated-C `class_exists()`, `method_exists()`, and
  `property_exists()` consume that registry through runtime value operands and
  shared diagnostics.
- Generated-C `get_parent_class()`, `class_parents()`,
  `get_declared_classes()`, `get_class_methods()`, and `get_class_vars()`
  consume shared runtime class metadata value handles for selected registered
  user classes and core class metadata.
- LLVM top-level user-class declarations and LLVM `class_exists()`,
  `method_exists()`, and `property_exists()` route through the shared class
  metadata registry ABI.
- Generated-C parser-resolved namespace/import class policy, named
  `ClassName::class`, leading-backslash/case-normalized dynamic class-name
  matching, and explicit `class_exists()` autoload policy boundary.
- Generated-C `class_alias()` registers normalized aliases for already-declared
  metadata targets and routes class/member metadata consumers through canonical
  alias lookup without fake autoload success.
- Parser/runtime `use function` imports with import-kind metadata, arbitrary
  aliases/default aliases, exact imported lookup without global suffix
  fallback, and alias-conflict guards. Generated C rejects function imports at
  an explicit production boundary until native imported-call lowering exists.
- Parser/runtime `use const` imports with import-kind metadata, arbitrary
  aliases/default aliases, exact imported lookup before namespace/global
  fallback, declaration/import conflict guards, and explicit generated-C/LLVM
  production rejection boundaries.
- Generated non-static method/constructor frames can assign literal and
  dynamic `$this` properties through the shared object-property mutation ABI.
- Generated-C selected non-local object-property assignments route literal and
  single-known dynamic public-property owners through reference-slot commits.
- Generated-C declared constructors with supported bodies can use bare
  `return;` early exits, and constructor `return <value>` now reports an
  explicit generated-C diagnostic with cleanup instead of pre-rejecting.
- Runtime declared static-property storage initializes defaults per request,
  preserves reference identity, enforces visibility shadowing and type checks,
  resolves late-static receivers, and resets storage between request states.
- Generated-C declared literal `Class::$prop` reads and writes route through
  request-owned runtime static-property storage with default/reset
  registration, type/visibility checks, canonical alias lookup, assignment
  result ownership, and explicit unsupported-shape blockers.
- Runtime terminal-kind diagnostic results can preserve return/throw/exit kind
  through cleanup transfer and report sinks.
- Structured generated-C object-property owner/fact/commit prerequisite
  boundary over public-property reference slots, including conservative fact
  invalidation and cleanup ownership for future property-held ArrayAccess.
- Receiver-free static `Class::method` string callable lookup through the
  runtime callable-value ABI.
- Shared diagnostic operation/operand-list blocker boundary.
- Owned diagnostic-result contracts, family consumers, continuation helpers,
  and report sinks for selected diagnostic-result paths.
- Discarded expression statements in LLVM and generated C lower through owned
  `NativeDiagnosticResult` statement operands and diagnostics-only report
  sinks.
- Echo operands and statement-form `print` in LLVM and generated C lower
  through owned `NativeDiagnosticResult` output operands and the shared echo
  report sink, including array-to-string conversion diagnostics.
- Control-transfer cleanup result lists have a runtime/backend consumer that
  consumes already-produced cleanup operands in source order, frees owned
  values, preserves diagnostics, and stops after terminal diagnostics.
- Runtime terminal cleanup transfer can preserve a pending terminal value
  through non-terminal cleanup result sequencing and release it when cleanup
  becomes terminal.
- LLVM and generated C have a shared control-transfer cleanup report bridge
  over already-produced cleanup diagnostic-result operands.
- LLVM and generated C have cleanup-frame producer queues that accept only
  cleanup-surface `NativeDiagnosticResult` operands before feeding the
  control-transfer cleanup report bridge.
- Cleanup frames carry terminal/control-transfer/deferred-cleanup source
  metadata and control-transfer cleanup reports consume frames instead of raw
  result slices.
- Cleanup-frame stacks aggregate nested frames innermost-first while preserving
  source metadata before feeding the shared cleanup report bridge.
- Cleanup-frame enqueueing validates cleanup surfaces and rejects
  terminal-source value ownership across LLVM and generated C before backend
  cleanup-frame emission.
- Reference-binding, assignment-lvalue, and RMW-lvalue operand-list blockers.
- Generated-C selected RMW array-lvalue owner/writeback for local native arrays
  and active-symbol/global-import reference-slot owners.
- Cleanup/unwind requirement diagnostics/preflight.
- Runtime ArrayAccess read/exists and write/append/unset dispatch ABIs.
- Generated-C ArrayAccess read/isset/write/append/unset/empty/null-coalesce/
  RMW/`??=` consumers for compiler-known generated declared `ArrayAccess`
  objects.
- Generated-C selected property-held ArrayAccess read/isset/empty/`??`, write,
  unset, compound RMW, and `??=` owners for literal and single-known dynamic
  object properties proven by constructor `$this` property facts.
- Shared generated-C native value/object facts for selected generated declared
  object producers and callable return summaries.
- Shared generated-C ReferenceSlot value owner source/commit and reference-cell
  fact ledger for compiler-visible native reference handles.
- Declared-class allocation cleanup-risk metadata and allocatable-class lookup
  metadata.
- Selected reference-source/lvalue extraction, reference-backed closure capture
  materialization, descriptor closure returns, byte-backed PHP string values,
  and byte-preserving selected string-array slots.

## Not Done

- Dynamic ArrayAccess producers beyond known generated declared-class `new` and
  direct generated-callable return summaries.
- Nested and non-direct property-held ArrayAccess production,
  append/increment forms, unknown dynamic property owners, dynamic class-name
  holders without definite facts, and reference-returning ArrayAccess
  semantics on top of the integrated object-property owner boundary.
- LLVM static-property producers, generated-C dynamic/self/parent/static/
  object-static static-property shapes, dynamic/typed property lowering, and
  full method/object model execution beyond the selected generated `$this`
  assignment and declared static-property subsets.
- Full reference/COW identity and arbitrary alias-root writeback.
- Actual exception/Throwable propagation, catch matching/binding, `finally`,
  destructors, shutdown cleanup, and object lifetime cleanup.
- Full SPL autoload, broader class-alias parity, native function/const-import
  lowering, broader namespace/function/const fallback, broader visibility,
  magic calls, broader constructor allocation/execution, named/spread
  arguments, and return references.
- Broader source-call production lowering over expression-owned
  `NativeCallResultHandle` carriers, including dynamic method names,
  object-static receiver calls, late-static binding, constructor allocation,
  closure argument-handle invocation ownership, direct function/method/static produced-call
  by-reference alias transfer, unknown runtime callable reference returns,
  runtime/builtin/inherited/trait/interface signature metadata, named/spread
  argument metadata, and spread ownership.
- Remaining semantic diagnostic-result operand migration for throw/exit/default
  return terminals, cleanup frame/result production from real control flow,
  lvalue, reference, RMW, and call-argument families; exact PHP diagnostics,
  source ordering, suppression/custom handlers, and backend parity across
  generated C, LLVM, and direct assembly.
- Pending diagnostic production from real control-flow cleanup, remaining
  terminal-kind lowering over the terminal transfer ABI, and exact
  `finally`/destructor/shutdown sequencing.

## Latest Focused Verification

For `e81aa43e`:

- `cargo fmt --all --check`
- `git diff --check`
- `cargo test -p phpc --test namespace_resolution -- --test-threads=1`
- `cargo test -p phpc --test unsupported_dynamic_features_cli cli_unsupported_dynamic_feature_snapshots_match_committed_outputs -- --exact --test-threads=1`
- `cargo test -p phpc --test native_global_constant_boundary emit_ir_rejects_bare_constant_reads_with_specific_boundary -- --test-threads=1`

For `6757bb43`:

- `cargo fmt --all --check`
- `git diff --check`
- `cargo test -p phpc --lib native_object_property_owner -- --test-threads=1`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_property_held_arrayaccess_through_object_property_owner_boundary -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_property_held_arrayaccess_owner_program -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link native_executable_c_source_rejects_arrayaccess_rmw_unsupported_owner_shapes -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_arrayaccess_rmw_nullcoalesce_assignment_through_owner_boundary -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_arrayaccess_rmw_nullcoalesce_assignment_runtime_consumer_program -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link this_property_assignment -- --test-threads=1`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_declared_class_constructor_program -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_declared_method_static_default_variadic_calls_through_source_call_carriers -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_declared_method_static_default_variadic_source_call_program -- --exact --test-threads=1`

For `b3f16040`:

- `cargo test -p phpc native_object_property_owner -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_arrayaccess_reference_slot_owners_through_value_owner_boundary -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_rejects_arrayaccess_rmw_unsupported_owner_shapes -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_arrayaccess_rmw_nullcoalesce_assignment_through_owner_boundary -- --nocapture`
- `cargo fmt --check`
- `git diff --check`

For `d96cc2bb`:

- `cargo test -p phpc --test native_runtime_abi native_user_class_metadata_registry_emit_ir_routes_llvm_through_runtime_abis -- --exact`
- `cargo test -p phpc --test native_runtime_abi native_user_class_metadata_registry_emit_asm_accepts_llvm_runtime_abi_routing -- --exact`
- `cargo test -p phpc --test object_model emit_ir_routes_class_exists_through_native_metadata_abi_and_folds_other_absent_metadata_calls -- --exact`
- `cargo test -p phpc --test object_model emit_ir_routes_absent_native_property_and_method_exists_calls_through_metadata_abi -- --exact`
- `cargo test -p phpc --test native_object_class_boundary metadata_registry`
- `cargo test -p phpc --test object_model until_native_object_lowering_exists`
- `cargo test -p phpc --lib native_object_metadata_call_preflight_reuses_direct_call_diagnostics`
- `cargo fmt --check`
- `git diff --check`

For `cb8457f1`, `ac5004a3`, and `04b18506`:

- `cargo test -p phpc --test native_link emit_exe_links_and_runs_namespace_alias_class_policy_program -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_namespaced_class_exists_user_function_takes_exact_precedence -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_class_exists_missing_default_reports_autoload_boundary -- --nocapture`
- `cargo test -p phpc --test native_runtime_abi terminal_kind -- --nocapture`
- `cargo test -p phpc --lib native_diagnostic_result_terminal_kind -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_this_property_assignment_in_generated_method_frames -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_this_property_assignment_reports_mutation_failure_from_method_frame -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_declared_class_constructor_program -- --nocapture`
- `cargo fmt --check`
- `git diff --check`

For `c8aeb771`:

- `cargo test -p phpc --test native_link native_executable_c_source_routes_declared_method_static_calls_through_source_call_carriers -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_declared_method_static_source_call_program -- --nocapture`
- `cargo test -p phpc --lib native_method_static_signature_fallback_contract_feeds_shared_binding_and_carriers -- --nocapture`
- `cargo test -p phpc --lib native_source_call -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_declared_static_methods_through_frame_dispatch -- --nocapture`
- `cargo fmt --check`
- `git diff --check`

For `4e5e2709`:

- `cargo test -p php_runtime native_user_class_metadata_registry_feeds_value_metadata_surfaces -- --nocapture`
- `cargo test -p phpc --test native_link user_class_value_metadata_registry -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_value_metadata_consumers_through_runtime_registry -- --nocapture`
- `cargo test -p phpc --lib native_object_metadata_call_preflight_reuses_direct_call_diagnostics -- --nocapture`
- `cargo fmt --check`
- `git diff --check`

For `69526631`:

- `cargo test -p phpc --test native_link native_executable_c_source_transfers_reference_return_source_calls_into_byref_arguments -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_source_call_reference_alias_byref_argument_program -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_reports_source_call_reference_alias_argument_cleanup_failure -- --nocapture`
- `cargo test -p phpc --test native_runtime_abi native_c_byref_produced_argument_consumers_use_alias_transfer_result_vector -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_scoped_callable_string_signature_program -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_plans_scoped_callable_string_signature_arguments -- --nocapture`
- `cargo test -p phpc --test native_runtime_abi native_call_frame_byref_alias_transfer_result_vector_preserves_targets_and_families -- --nocapture`
- `cargo test -p phpc --test native_runtime_abi native_c_adjacent_argument_binding_avoids_alias_transfer_without_produced_byref_args -- --nocapture`
- `cargo fmt --check`
- `git diff --check`

For `8d5f3715`, `53bef000`, `a7054ed1`, `d67c7f14`, `4d6f0f1e`, `8ebeae19`, `964e3e2b`,
`0430efcc`, `75f20f3f`, `d26c64f7`, `50d19f99`, and `7891fcf3`:

- `cargo test -p phpc --lib native_method_static_signature_fallback_contract_classifies_declared_and_runtime_metadata -- --nocapture`
- `cargo test -p phpc --lib native_method_static_signature_fallback_contract_feeds_shared_binding_and_carriers -- --nocapture`
- `cargo test -p phpc --lib native_diagnostic_cleanup_frame_stack_aggregates_nested_pending_diagnostics_for_unwind -- --nocapture`
- `cargo test -p phpc --lib native_diagnostic_cleanup_frames_enforce_cleanup_surface_and_terminal_value_ownership -- --nocapture`
- `cargo test -p phpc --lib native_diagnostic_result_control_transfer_cleanup_reports_reusable_cleanup_operands -- --nocapture`
- `cargo test -p phpc --test native_runtime_abi native_diagnostic_result_control_transfer_cleanup_sequences_result_shapes -- --nocapture`
- `cargo test -p phpc --test native_runtime_abi terminal_value_transfer -- --nocapture`
- `cargo test -p phpc --lib native_source_call_emitter_builds_arguments_once_and_routes_carriers -- --nocapture`
- `cargo test -p phpc --lib native_source_call -- --nocapture`
- `cargo test -p phpc --lib native_invoke_result_helper_selector_routes_lookup_plus_invoke_families -- --nocapture`
- `cargo test -p phpc --lib native_callable_runtime_boundary_declares_lookup_plus_invoke_helpers -- --nocapture`
- `cargo test -p phpc --test native_function_call_boundary native_executable_direct_user_function_calls_use_runtime_callable_abi_across_arities -- --nocapture`
- `cargo test -p phpc --test native_function_call_boundary native_executable_direct_user_function_calls_preserve_reference_arguments_through_runtime_callable_abi -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_dynamic_string_callable_value_program -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_direct_user_function_frame_program -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_dynamic_callable_values_through_runtime_abi -- --nocapture`
- `cargo test -p php_runtime native_diagnostic_result_from_call_result_consumes_value_reference_failure_and_null -- --nocapture`
- `cargo test -p phpc --lib native_source_call_result_carrier_routes_targets_and_consumers_through_owned_results -- --nocapture`
- `cargo test -p phpc --lib native_callable_runtime_boundary_declares_call_result_diagnostic_converter -- --nocapture`
- `cargo fmt --check -p phpc -p php_runtime`
- `git diff --check`
