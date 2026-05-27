# PHP Native Compiler Progress

Updated: 2026-05-27 10:40 CEST
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

Latest accounted source capability: `2e2625eb` routes selected generated-C
descriptor-closure reference returns into real reference carriers for proven
descriptor-backed closures. The supported path admits by-reference closure
returns from direct by-reference parameters or captures, invokes proven
descriptor closures through the runtime reference-return helper, and feeds the
owned reference into both by-reference argument transfer and direct reference
assignment materialization without copying values to fake aliases. Unknown or
mixed callables, callable arrays, invokable objects, method descriptors,
non-descriptor closures, unsupported closure reference return sources,
descriptor identity lost through symbol-table-only storage, broader
reference/COW ownership, and LLVM/backend parity remain blocked. Recent source
commit `16e7dec5` routes selected generated-C static-property null-coalescing
assignment through the shared static-property lvalue/storage boundary for
literal class, object/class-string, `self`, `parent`, and declared
method-frame `static` receivers. Recent source commit `3d32236a` routes
selected generated-C static-property `isset()` and `empty()` through the same
shared lvalue/storage boundary. Recent source commit `8c922fa2` routes
selected generated-C direct root ArrayAccess append-with-keyed-suffix
assignments such as `$bag[]["leaf"] = $value` through a generalized root
ArrayAccess owner boundary. The supported path materializes suffix keys before RHS evaluation,
wraps the RHS through the shared appended-slot value boundary, calls
`offsetSet(null, wrapped_value)` on the root ArrayAccess object, preserves
assignment-expression result ownership separately from the appended slot value,
and threads cleanup/diagnostics through the existing owner commit boundary.
Function-call suffix-key expressions, unknown root facts, arbitrary alias
roots, static-property roots, reference-returning `offsetGet()`, broad
references/COW, cleanup/unwind breadth, spread/unpack, and LLVM/backend parity
remain blocked. `7ff18dba` also routes selected generated-C constructor
allocation-plus-invoke through declared class metadata, typed-property-aware
allocation, shared `NativeCallArgumentsHandle` ownership, `$this` binding,
called scope, caller access context, allocated receiver cleanup, and dedicated
constructor value-return diagnostics. Recent source commits also route selected
generated-C nested ArrayAccess `unset()` and append-with-keyed-suffix
assignments through the generalized owner-stack path for direct-variable and
visible property-held roots. Supported unset paths materialize the root owner,
descend through by-value `offsetGet()` intermediates, execute leaf
`offsetUnset()`, reverse-write changed parents with `offsetSet()`, and commit
the original owner. Supported keyed append paths materialize suffix keys before
RHS evaluation, wrap the RHS as nested native arrays through the shared
appended-slot boundary, append at the owner-stack leaf with
`offsetSet(null, value)`, reverse-write parents, commit the original owner, and
keep assignment-expression result ownership separate from the appended slot
value. Reference-returning `offsetGet()`, arbitrary alias roots,
non-direct/unknown property holders, static-property roots, broad
references/COW, cleanup/unwind breadth, spread/unpack, and LLVM/backend parity
remain blocked. Recent source commits also route selected generated-C
static-property plain assignment, compound assignment, and pre/post
increment/decrement through a shared lvalue target for literal class receivers,
object/class-string receivers, and relative `self`, `parent`, and method-frame
`static` receivers. Supported paths read and write request-owned
static-property storage through the same runtime APIs, derive dynamic receiver
scope through the shared receiver-scope ABI, preserve expression-result
ownership for compound and pre/post operations, and keep parser
object-static-property lvalues flowing to codegen instead of falling behind
prefix/compound blockers. Computed static-property names, top-level
`static::$prop`, static-property references, `??=`, `unset`, array-offset
mutation, magic/static overloading, traits/interfaces, autoload
breadth, broad references/COW, and LLVM/backend parity remain blocked. Recent
source commits also route selected generated-C nested ArrayAccess
null-coalescing assignments through the generalized owner-stack path for
direct-variable and visible property-held roots. Supported paths materialize
the root owner, descend through by-value `offsetGet()` intermediates, probe the
leaf with `offsetExists()`, read present leaves with `offsetGet()`, lazily
evaluate the RHS only for missing/null leaves, write mutated leaves with
`offsetSet()`, reverse-write parents only on mutation branches, commit the
original owner, and preserve the `??=` expression result. Recent source
commits also route selected generated-C static
method source calls through the shared runtime `__callStatic` fallback
boundary. Normal declared static method hits still win, missing or inaccessible
static methods fall back to public static `__callStatic($name, $args)`, and
non-static methods called statically remain hard failures rather than falling
through to magic. The integrated paths cover literal `Class::method(...)`,
object-static receivers, `self::`, `parent::`, and bounded declared-frame
`static::` source calls without generated method-name ladders or fixture-shaped
dispatch. Named-argument magic `$args` key preservation, malformed magic
signature parity, traits/interfaces/effective method tables, aliases/autoload,
callable-object static magic shapes, full `$args` reference/COW parity, and
LLVM/direct assembly parity remain blocked. Recent source commits also
initialize generated-C declared static-property storage through the shared bulk
metadata/default ABI. Generated C now consumes declared-property metadata arrays
for names, visibility, type declarations, defaults, and static flags, registers
defaults only for static properties, and preserves non-static metadata so static
lookups still honor shadow boundaries. Typed static-property defaults and writes
now report through the runtime static-property type/visibility diagnostics
instead of using the old single-property generated-C registration path. Dynamic
class/property static shapes, static-property references, unset, compound
mutation, increment/decrement, `??=`, append/nested static-property
offset mutation, magic/static overloading, traits/interfaces, autoload breadth,
full type parity, and LLVM/backend parity remain blocked. Recent source commits
also route
descriptor-backed closure calls through shared `NativeCallArgumentsHandle`
production and runtime closure-invoke call-result carriers. Proven
descriptor-closure callable facts now bypass the broad dynamic callable name
path, evaluate arguments through the same source-call argument machinery as
functions and methods, and consume result/value/reference/discard closure
invocation helpers without matching one fixture, arity, or local variable name.
Unknown or mixed callable identities, callable arrays, invokable objects,
non-descriptor closure handoff, descriptor/method/closure reference-return
breadth, spread/unpack, broader request-state handoff, and LLVM/backend parity
remain blocked. Recent source commits also route selected generated-C object
and declared class-string static-property receivers through the shared runtime
receiver-scope ABI before using request-owned static-property storage.
Supported paths cover direct reads and plain assignments such as `$object::$p`
and `$classString::$p`, preserve assignment expression results, free receiver
and scope handles on failure paths, and reuse the same static-storage read/write
ABI as literal and relative static-property producers. Top-level
`static::$prop`, dynamic property names, static-property references, compound
mutation/unset, magic/static overloading, traits/interfaces,
autoload breadth, LLVM parity, and reference/COW static-property breadth remain
blocked. Recent source commits
also route selected generated-C nested
ArrayAccess pre/post increment and decrement through the generalized nested
owner-stack write context for direct-variable and property-held roots.
Supported paths descend through by-value `offsetGet()`, read the leaf, compute
the native increment/decrement replacement, preserve pre/post expression-result
semantics, write the leaf with `offsetSet()`, perform reverse parent writeback,
and commit the root owner; newer append, null-coalescing assignment, unset, and
keyed-append-suffix paths reuse the same owner stack for direct and
property-held roots. Root keyed-suffix append without owner-stack descent,
reference-returning `offsetGet()`, arbitrary
alias roots, broad references/COW, cleanup/unwind breadth, spread/unpack, and
LLVM/backend parity remain blocked. Recent source
commits also add parser/AST source-ordered named call-argument nodes plus shared
call-argument normalization that binds source-order positional/named arguments
to parameter-order slots across required, optional/default, by-reference, and
variadic parameters. Generated C now lowers selected compiler-known direct
user-function calls and method/static/dynamic source-call carriers through that
normalizer before building shared `NativeCallArgumentsHandle` values. Named
builtins, constructors, unknown dynamic callables, magic `__call`/`__callStatic`
fallback named-argument parity, spread/unpack, malformed magic
signatures, broader runtime/inherited/trait/interface signatures, LLVM/direct
assembly parity, and backend-wide named-argument parity remain blocked. Recent
source commits also route generated-C dynamic receiver method calls on compiler-known object
receivers with declared instance `__call($name, $args)` through the shared
runtime lookup-plus-invoke dispatcher. Normal method hits still win, missing or
inaccessible receiver methods fall back to public non-static `__call`, and
runtime packs the original method name plus value snapshots of original
argument slots into the magic `$args` array while preserving class-context
private method dispatch. Named-argument `__callStatic` fallback, malformed
magic signature warning/fatal parity, traits/interfaces/effective method tables,
aliases/autoload,
callable-object fallbacks, full reference/COW alias behavior for `$args`, and
LLVM/direct assembly parity remain blocked. Recent source commits also lower exact
parser-resolved generated-C `use const` aliases for same-compilation-unit scalar
user constants and supported builtin constants through shared constant
metadata/import markers.
Exact imported misses reject without namespace/global fallback, and ordinary
bare constants, arrays and dynamic constant expressions, duplicate/builtin
collisions beyond explicit rejection, class constants, include/autoload
discovery, dynamic alias roots, function-frame constant lookup, LLVM constant
lowering, and broad backend/global constant parity remain blocked. Recent
source commits also route generated-C `static::$prop` reads/writes in declared
method frames through the same shared relative static-property runtime storage
used by `self::$prop` and `parent::$prop`, threading the active method-frame
called scope into the runtime ABI so root and descendant static storage stay
distinct. Top-level `static::$prop`, dynamic class/property names,
static-property references, compound
mutation/unset/isset/empty, magic/static overloading, traits/interfaces,
autoload breadth, LLVM parity, and reference/COW static-property breadth remain
blocked (`bf8fd335`); route selected generated-C nested ArrayAccess compound
assignments through
the generalized nested owner-stack path for direct-variable and property-held
roots: each supported path descends through by-value `offsetGet()`, reads the
leaf, computes the native binary replacement value, writes the leaf with
`offsetSet()`, performs reverse parent writeback, and commits the root owner.
Nested `??=`, append, increment, decrement, reference-returning `offsetGet()`,
arbitrary alias roots, broader reference/COW semantics, cleanup/unwind breadth,
and LLVM parity remain blocked (`ba5769e2`); route generated-C `self::$prop`
and `parent::$prop` reads/writes from declared method frames through shared
relative static-property runtime storage, with one program-wide request storage
handle shared across top-level and method-frame static-property operations
(`cfc7c0ee`); route generated-C receiver
method calls with unknown runtime method-name expressions through the shared
lookup-plus-invoke source-call carrier when the receiver has compiler-known
object class facts and no declared `__call`. Runtime lookup now normalizes
scalar dynamic method-name values before access-context lookup, preserving
non-scalar diagnostics and static-through-object rejections. Declared
`__call`/broader `__callStatic`, magic argument packing, traits/interfaces, arbitrary
aliases/autoload, callable-object fallbacks, broader byte/encoding diagnostic
parity, and LLVM parity remain blocked (`7ae07fd7`); route
bounded generated-C
`static::method(...)` calls in declared method frames through runtime
called-scope handles and static source-call carriers, keeping lexical class
context only for visibility/access checks. Descendant-only targets,
override-visible late-static breadth, traits, interfaces, magic
broader `__callStatic`, dynamic static properties, `static::class`, `static::$prop`,
LLVM parity, and broad inheritance/interface resolution remain blocked. Recent
source commits also lower selected generated-C
declared typed instance properties by emitting per-property type/default
metadata, allocating objects through the runtime typed-property metadata ABI,
initializing default values, and routing known typed instance-property writes
through diagnostic mutation so type failures report instead of mutating
silently. Typed static properties, dynamic/magic properties, unsupported object
type declarations, broad reference/COW property semantics, and LLVM/backend
parity remain blocked (`f1816273`); preserve bounded generated-C
function and method return-through-finally cleanup by queuing finalizer output
cleanup operands into the terminal-kind return transfer before return-value
handoff, while keeping top-level try returns, throw/catch, exit/goto unwind,
finally-return replacement, source-call cleanup in return operands, destructor
ordering, and full exception/finally semantics blocked (`2ca9d0e3`); lower
selected generated-C nested ArrayAccess assignments through a
generalized owner stack for direct-variable roots and property-held roots: each
supported path descends through by-value `offsetGet()`, writes the leaf with
`offsetSet()`, performs reverse parent writeback, and commits the root owner
while keeping reference-returning `offsetGet()`, nested RMW, `??=`, append,
increment, decrement, and broader COW/reference forms blocked (`e8268187`);
they also lower exact parser-resolved generated-C `use function` aliases for
same-compilation-unit user functions and selected signature-backed runtime
callable builtins through the shared source-call argument and lookup/invoke
stack, preserving exact missing imported runtime names and unsupported imported
builtin lookup-before-argument guards while keeping dynamic alias roots,
include/require discovery, autoload, broad runtime builtin imports, native
const-import lowering, and LLVM/backend parity blocked (`5bacf1aa`); route
compiler-known generated-C dynamic receiver-method calls with known string names
through the shared source-call carrier stack, including class-context access for
declared method frames and runtime rejection of static descriptors used through
object dispatch, while keeping declared `__call`, broad magic dispatch,
traits/interfaces, unknown scalar method names, late-static behavior, and
unsupported receiver shapes blocked (`5a4f10a5`); route selected
generated-C property-held ArrayAccess append and pre/post increment/decrement
through real object-property reference owners, including conversion
diagnostics, expression-result ownership, owner writeback, and unsupported-owner
blockers while keeping nested offsets, unknown dynamic properties,
reference-returning `offsetGet()`, static-property owners, broad COW/reference
semantics, and backend parity blocked (`ad74d35b`); route generated-C receiver
and named static calls made from inside declared method frames through runtime
class-context access checks, so supported declared methods can call private
instance methods on `$this` and protected static methods on the declaring class
through shared source-call carriers while external receiver/static callers and
dynamic fallback ladders stay public-only (`99d23aa4`); route generated-C
object static-receiver method calls through shared
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
across object/class execution and backend parity: source-ordered named
arguments now parse into explicit AST nodes and selected generated-C direct
user-function plus method/static/dynamic source-call carriers bind them through
shared parameter-order normalization instead of treating them as unsupported
syntax or exact-shape call lowering,
selected nested ArrayAccess pre/post increment and decrement now reuse the
owner-stack descent, leaf mutation, reverse writeback, and root-commit boundary
instead of staying behind the broad non-assignment mutation blocker,
generated method frames can
write `$this` properties, selected constructors with supported bodies and bare
early returns run, constructor value returns now fail through an explicit
generated-C runtime diagnostic with cleanup, method/static source calls handle
default and variadic frame-compatible arities, `self::` and `parent::`
static source calls now run through class-context source-call carriers,
declared method-frame calls can now invoke supported private/protected
receiver/static methods through runtime class-context lookup-plus-invoke
carriers,
compiler-known dynamic receiver-method calls with known string names now reuse
source-call carriers instead of generated name-comparison ladders, including
class-context private/protected access from declared method frames,
runtime-produced dynamic receiver-method names now also route through runtime
lookup-plus-invoke carriers for compiler-known object receiver facts,
declared instance `__call` receiver fallbacks now share that runtime dispatch
boundary and pack PHP magic `$name`/`$args` values instead of reviving generated
comparison ladders,
object static-receiver calls now derive runtime receiver scope and run through
shared static source-call carriers for exact/default/variadic
frame-compatible declared static methods,
bounded declared-frame `static::method(...)` calls now use runtime called scope
instead of lexical `self::` dispatch while preserving lexical access context,
literal declared `Class::$prop` reads and writes now execute through
request-owned runtime static-property storage instead of being rejected,
declared-frame `self::$prop` and `parent::$prop` now share that request storage
through relative static-property receivers,
declared-frame `static::$prop` now shares it with the active called-scope
receiver so inherited static-property methods can read and write descendant
storage without collapsing to lexical `self`,
object and declared class-string static-property receivers now derive runtime
receiver scope and reuse request-owned static-property storage for direct reads
and plain assignments,
declared typed instance properties now allocate with runtime type/default
metadata, initialize defaults, and enforce typed writes through diagnostic
mutation in generated C,
generated-C exact `use function` aliases now execute for same-unit user
functions and selected runtime callable builtins through shared source-call
arguments, while unsupported imported runtime builtins fail before argument
side effects,
generated-C exact `use const` aliases now execute for same-unit scalar user
constants and supported builtin constants through parser/import metadata while
ordinary bare and broader constant lookup shapes remain blocked,
descriptor-backed closure calls now use shared call-argument handles and
closure invoke result carriers instead of falling through the broad dynamic
callable name path,
selected nested ArrayAccess assignments now execute through owner-stack
descent, leaf write, reverse parent writeback, and root commit for direct and
property-held roots,
selected nested ArrayAccess compound assignments now reuse that owner-stack
boundary with leaf reads, native binary replacement values, reverse parent
writeback, and direct/property root commits,
bounded generated-C function and method returns through active `finally` bodies
now preserve finalizer stdout/diagnostics as terminal-transfer cleanup operands
before returning the original value,
interpreter function imports resolve exactly,
interpreter const imports resolve exactly before namespace/global fallback,
direct generated-C user-function frames can return references through shared
source-call carriers and by-reference consumer paths, proven reference-return
direct/dynamic/receiver/static source calls can feed by-reference arguments
through shared carriers, selected runtime callable builtin signatures now feed
dynamic source-call argument binding while unsupported builtins are rejected
before side-effecting arguments are built, selected property-held
ArrayAccess owners now cover append and pre/post increment/decrement in
addition to `unset()` through the same reference-owner commit path as
reads/writes/RMW, and selected external object-property assignments reuse true
public-property reference owners instead of temporary mutation
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
object-property owner facts now drive selected property-held and nested
ArrayAccess production. The bars remain far from 100% because root keyed-suffix
append without owner-stack descent, reference-returning ArrayAccess owners, static magic named-argument
and signature breadth,
malformed magic signature parity, remaining override-visible late-static and
broader magic method shapes,
spread arguments and unsupported named builtin/constructor/fallback call
families, broader const discovery/lookup and function-import
coverage, broader
constructor execution, late-static binding,
arbitrary alias roots, LLVM and remaining generated-C static-property producers,
broader static-property reference/`??=`/unset/isset/empty and dynamic/magic property breadth,
exceptions/cleanup, full SPL
autoload, visibility/magic breadth, and backend parity are still open.

Current critical path to 100%:

1. Extend expression-owned `NativeCallResultHandle` carriers and production
   source-call lowering over the lookup-plus-invoke ownership helpers,
   including remaining dynamic/magic method shapes and late-static override
   breadth,
   constructor allocation, non-descriptor closure handoff, broader by-reference alias
   transfer, and spread ownership.
2. Continue migrating expression, statement, terminal, cleanup, lvalue,
   reference, and call-argument lowering onto produced
   `NativeDiagnosticResult` operands.
3. Route value/reference/return/deferred-cleanup consumers through the shared
   diagnostic-result and call-result carrier stack.
4. Broaden the integrated object-property owner/fact/commit boundary from
   selected non-local assignments, property-held ArrayAccess
   read/write/RMW/unset/append/increment, and selected nested
   assignment/RMW/increment/decrement/append/`??=`/unset/keyed-append suffix
   into root keyed-suffix append, reference-returning forms,
   references/COW, arbitrary alias-root writeback, and broader
   object/static/dynamic/magic property storage and broader static-property
   mutation/reference surfaces.
5. Implement real exception/Throwable propagation, catch/finally/destructor/
   shutdown cleanup, source-ordered diagnostics, and custom handler behavior.
6. Broaden namespace/const import production breadth, function-import
   discovery/fallback breadth, namespace fallback, autoload, broader aliases,
   visibility/magic, constructors, spread arguments, unsupported named
   builtin/constructor/fallback call families,
   descriptor/method/closure return references, and backend parity.

## Roadmap Bars

| Workstream | Integrated | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **90%** | `[##################--]` | Strong selected-path value, byte-string, array, reference, symbol, callable, call-frame/result, diagnostic-result, terminal-kind, request-state, lvalue, ArrayAccess, class metadata value, descriptor-closure call-result helpers, request-scoped static-property storage plus bulk metadata/default registration and object/class-string receiver-scope resolution, typed instance-property metadata/allocation/write diagnostics, selected receiver `__call` and static `__callStatic` dispatch, generated-C class alias metadata, function/const-import exact lookup, and autoload-policy boundary surfaces. Remaining gaps include arbitrary alias transfer, full autoload, namespace fallback, malformed magic signature parity, broader closure frame handoff, cleanup/unwind parity, and broader lookup parity. |
| Compiler/backend consumers | **85%** | `[#################---]` | Generated C has the freshest consumers for calls, callable facts, selected descriptor-closure calls, selected object/class metadata, namespace/import class policy, exact function-import aliases for same-unit user functions and selected runtime builtins, exact const-import aliases for same-unit scalar user constants and supported builtins, typed declared instance-property allocation/write metadata, selected bulk typed static-property metadata/default registration, selected literal, object/class-string receiver, and relative `self`/`parent`/`static` static-property read/write storage, selected ArrayAccess/lvalue paths, selected non-local object-property assignment owner commits, value-result casts, explicit by-value return terminal handoff, diagnostic-result family consumers, discarded statement-expression diagnostic operands, echo/print output diagnostic operands, and control-transfer cleanup report bridging. It still rejects ordinary/broader constant lookup and unsupported function-import shapes at explicit production boundaries. LLVM shares user-class metadata declaration/exists routing plus the discarded-expression, output operand, and cleanup report bridge paths, while direct assembly still lags newer object-offset/lvalue/static-property/runtime ABIs and most semantic result operands remain unmigrated. |
| Executable PHP semantics | **85%** | `[#################---]` | Many executable islands exist, including bounded method/static/dynamic-receiver and descriptor-closure source-call production, generated method-frame `$this` property assignment, selected non-local object-property assignment commits, selected declared literal, object/class-string receiver, and `self`/`parent`/`static` static-property reads/writes plus selected static-property observation/compound/increment/decrement mutations, selected typed static-property metadata/default registration and writes, selected declared typed instance-property defaults and typed writes, selected constructor bodies with bare early returns, selected property-held ArrayAccess read/write/RMW/`??=`/unset/append/increment owners, selected nested ArrayAccess assignment, compound-assignment, pre/post increment/decrement, append, null-coalescing assignment, unset, and keyed-append-suffix owner stacks for direct and property-held roots, selected direct-root ArrayAccess keyed-suffix append, namespace/import class policy, interpreter function/const-import exact lookup, generated-C exact function-import aliases for same-unit user functions and selected runtime builtins, generated-C exact const-import aliases for same-unit scalar user constants and supported builtins, and value-returning class metadata consumers, but broad assignment/RMW/writeback, references/COW, reference-returning nested ArrayAccess breadth, unknown dynamic/static-property shapes, broader static-property reference/`??=`/unset and dynamic/magic property breadth, cleanup/unwind/finally/destructors, exact diagnostics, broader const discovery/lookup, broader function-import coverage, and backend parity remain open. |
| Strings and byte semantics | **60%** | `[############--------]` | Byte-backed values and selected byte-preserving string-array slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **85%** | `[#################---]` | Selected lvalue/reference-source extraction, ReferenceSlot owner facts, object-property owner/fact/commit prerequisites, selected non-local object-property assignment commits, reference-cell predicates, membership helpers, RMW array-lvalue owner/writeback, selected direct/generated-object ArrayAccess RMW/`??=` paths, selected direct-root ArrayAccess keyed-suffix append, selected property-held ArrayAccess read/write/RMW/`??=`/unset/append/increment owners, and selected nested ArrayAccess assignment, compound-assignment, pre/post increment/decrement, append, null-coalescing assignment, unset, and keyed-append-suffix owner stacks are integrated. Reference-returning `offsetGet()`, arbitrary alias roots, foreach breadth, broader writeback, and full COW remain incomplete. |
| Symbols, globals, request state | **70%** | `[##############------]` | Selected globals, root-symbol consumers, active symbol-table consumers, request-key blockers, append-shaped symbol reference-source materialization, direct generated-C request-state frame handoff, and dynamic user-function handoff proof exist. `$GLOBALS` self-cells, closure request-state handoff, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **90%** | `[##################--]` | Runtime callable table/value dispatch, selected runtime builtin source-call signatures/blockers, call arguments/frame/result ABI, source-order/parameter-order named call-argument normalization, conditional handoff, generated-C direct/dynamic callable consumers, declared-method registration/wrapper frames, callable return facts, by-reference argument transport, descriptor closures, descriptor-closure invocation through shared argument/result carriers, closure returns, request-state frame handoff, access-context lookup ABI, lookup-plus-invoke exactly-once argument ownership helpers, source-call result carrier selectors, selected production source-call carrier emission, direct generated user-function lookup-plus-invoke production, direct user-function reference-return frames, exact generated-C `use function` aliases for same-unit user functions and selected runtime callable builtins, explicit by-value return terminal handoff, method/static source-call target operands, method/static source-call binding operands, method/static signature fallback selection, selected direct/dynamic/receiver/static/self/parent reference-return source-call alias transfer into by-reference arguments, executable receiver/static/self/parent/object-static/late-static method source-call production for exact, default, variadic, and selected named-argument frame-compatible arities where class context or receiver/called scope is known, selected generated-C dynamic receiver-method source-call production for known string and runtime-produced method names including declared receiver `__call` fallback, selected generated-C static method `__callStatic` fallback, explicit generated-C constructor value-return diagnostics, and interpreter/generated-C exact function/const import islands are integrated. Unknown runtime callables, named-argument static magic fallback, malformed magic signature parity, broader late-static override resolution, broader builtin/native/inherited/trait/interface signature metadata, broader by-reference alias transfer, broader const-import discovery/fallback and function-import discovery/fallback, spread and unsupported named builtin/constructor/fallback breadth, descriptor/method/closure and broader return references, broader constructor allocation/execution, cleanup/unwind, and backend parity remain open. |
| Objects, properties, methods | **85%** | `[#################---]` | Selected object metadata, value-returning class metadata consumers, LLVM/generated-C user-class metadata consumers, generated-C namespace/import class policy, generated-C class alias metadata and canonical class/member lookup, public property reference-source extraction, method-frame `$this` property assignment, selected non-local object-property assignment commits, object-property owner/fact/commit prerequisites, object-property reference-slot mutation, selected property-held ArrayAccess read/write/RMW/`??=`/unset/append/increment owners, selected direct-root ArrayAccess keyed-suffix append, selected nested ArrayAccess assignment, compound-assignment, pre/post increment/decrement, append, null-coalescing assignment, unset, and keyed-append-suffix owner stacks, request-scoped runtime static-property storage plus bulk declared-property metadata/default arrays and generated-C declared literal, object/class-string receiver, and relative `self`/`parent`/`static` static-property read/write/observation and selected mutation producers, generated-C declared typed instance-property allocation/default/write diagnostics, generated-C ArrayAccess consumers for compiler-known generated objects, dynamic generated class-name producers, object-call argument handles, declared-method callable-table publication, bounded executable receiver/static/self/parent/object-static/dynamic/late-static method production through access-context source-call carriers, selected declared receiver `__call` and static `__callStatic` fallback dispatch, selected constructor bodies with bare early returns and explicit value-return diagnostics, allocatable class metadata, user-class metadata registry consumers, and access-context preflights exist. Reference-returning nested ArrayAccess breadth, named-argument `__callStatic` fallback/malformed-magic/mixed-runtime-dynamic-call/clone/static-property breadth, full late-static override/interface/trait binding, broader dynamic/static-property breadth, broader class-alias/autoload parity, broader visibility parity, broader static-property reference/`??=`/unset and dynamic/magic property breadth, destructors, interfaces/traits execution, references/COW, broader constructor allocation/execution, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **70%** | `[##############------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostic blockers, owned diagnostic-result list contracts, consumer contracts, backend family consumers, deferred-cleanup blockers, control-transfer cleanup result consumers, terminal cleanup transfer ABI, terminal-kind ABI, explicit by-value return terminal handoff, bounded function/method return-through-finally cleanup operands, cleanup-frame producers/source metadata/report bridges, cleanup-frame stack aggregation, cleanup-frame enqueue validation, try-body call-boundary preflight, report sinks, continuation helpers, discarded statement-expression operands, and echo/print output operands exist. Broad unwind/finally/destructor/shutdown execution, cleanup result production from arbitrary control flow, executable reference binding, remaining semantic diagnostic-result producer migration, and source-ordered diagnostics remain open. |
| Broad integrated verification | **75%** | `[###############-----]` | Focused gates around recent source work are strong, with several primary integration gates now covering linked generated-C class/method/constructor programs, LLVM class metadata routing, terminal-kind ABI behavior, and owner-boundary regressions. Broad verification is still constrained by lane extraction cost, stale candidate expectations, heavy formatter/log pressure, and backend parity gaps. |

## Recently Accounted Source Work

| Commit | Capability | Proof shape |
| --- | --- | --- |
| `2e2625eb` | Generated C now routes selected descriptor-closure reference returns through a real runtime reference carrier for proven descriptor-backed closures. By-reference closure returns are admitted only when the return source is a direct by-reference parameter or capture, and the resulting reference can feed both by-reference argument transfer and direct reference assignment materialization with alias/write-through behavior. Unknown/mixed callables, callable arrays, invokable objects, method descriptors, non-descriptor closures, unsupported by-ref return sources, identity loss through symbol-table-only closure storage, broader reference/COW ownership, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=16 failures=0`, covering fmt, diff checks, `cargo check -p phpc`, descriptor-closure reference carrier unit proof, generated-C and linked executable reference-return consumer proofs, descriptor closure and source-call reference filters, magic static, named, static property, nested and property-held ArrayAccess, constructor, exact import, and class-alias filters. Gate log: `state/workers/logs/phpc-primary-descriptor-closure-reference-return-r2-integration-20260527.gates.log` sha256 `a31b72882552c114b2db8f69b5a12bc95c82d00e0b96ea7ffd11aad647695cb1`. |
| `16e7dec5` | Generated C now routes selected static-property `??=` through the shared static-property lvalue target for literal class, object/class-string, `self`, `parent`, and method-frame `static` receivers. Supported paths use read-for-`isset` probes, skip RHS evaluation for present non-null storage, lazily write missing/null/uninitialized storage through typed/visibility static-property APIs, and preserve expression-result ownership. Computed names, top-level `static::$prop`, references, `unset`, array-offset mutation, magic/static overloading, broader references/COW, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=15 failures=0`, covering fmt, diff checks, `cargo check -p phpc`, runtime static-property tests, static-property and static-property mutation native-link filters, object static-property receivers, descriptor closures, magic static, nested and property-held ArrayAccess, constructors, source-call references, exact imports, and class aliases. Gate log: `state/workers/logs/phpc-primary-static-property-nullcoalesce-r2-integration-20260527.gates.log` sha256 `ae3fba1e97863bcf7068e4298d0103d9fe5bafb35a6a9c6aa6205470f201a6f8`. |
| `ba46f5e2` | Generated C now routes selected nested ArrayAccess `unset()` and append-with-keyed-suffix assignments through the generalized owner stack for direct-variable and visible property-held roots. Supported unset paths perform by-value `offsetGet()` descent, leaf `offsetUnset()`, reverse parent writeback, and root commit. Supported keyed append paths materialize suffix keys, wrap the RHS through the shared appended-slot value boundary, append at the leaf with `offsetSet(null, value)`, reverse-write parents, commit the root owner, and preserve assignment expression results separately from the appended value. Root keyed-suffix append without owner-stack descent, reference-returning `offsetGet()`, arbitrary alias/static roots, broader references/COW, cleanup/unwind breadth, spread/unpack, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=18 failures=0`, covering fmt, diff checks, nested owner-stack units, exact generated-C and linked executable proofs for nested unset and keyed-append suffix, nested ArrayAccess regressions, property-held ArrayAccess owners, descriptor closures, magic static calls, static-property and static-property mutation regressions, object static-property receivers, source-call references, exact imports, class aliases, and `cargo check -q -p phpc`. |
| `1202542e` | Generated C now routes selected static-property plain assignment, compound assignment, and pre/post increment/decrement through a shared lvalue target for literal class, object/class-string, `self`, `parent`, and method-frame `static` receivers. The path reuses runtime static-property storage read/write APIs, derives object/class-string scope through the shared receiver-scope ABI, preserves compound and pre/post expression-result ownership, and lets parser object-static-property lvalues reach codegen for prefix/compound mutations. Computed names, top-level `static::$prop`, static-property references, `??=`, `unset`, `isset`, `empty`, array-offset mutation, magic/static overloading, broader references/COW, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=14 failures=0`, covering fmt, diff checks, runtime static-property tests, static-property native-link proofs, static-property mutation executable proof, object static-property receivers, descriptor closures, magic static calls, source-call references, nested ArrayAccess, property-held ArrayAccess, class aliases, and `cargo check -p php_runtime -p phpc`. |
| `8a8d85ed` | Generated C now routes selected nested ArrayAccess null-coalescing assignments through the generalized owner stack for direct-variable and property-held roots. Supported paths probe the leaf with `offsetExists()`, read present leaves, lazily evaluate the RHS only for missing/null leaves, write mutated leaves with `offsetSet()`, reverse-write parents only on mutation branches, commit the root owner, and preserve the `??=` expression result. Nested unset, append-with-keyed-suffix, reference-returning `offsetGet()`, arbitrary alias/static roots, broader references/COW, cleanup/unwind breadth, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=14 failures=0`, covering fmt, diff checks, nested owner-stack units, exact generated-C and linked executable nested `??=` proofs, nested ArrayAccess regressions, property-held ArrayAccess owners, descriptor closures, magic static calls, static properties, source-call references, class aliases, and `cargo check -p phpc`. |
| `5feb8cfe` | Generated C routes selected nested ArrayAccess append assignments through the generalized owner stack for direct-variable and property-held roots. Supported paths materialize the root owner, descend through by-value `offsetGet()` intermediates, append at the leaf with `offsetSet(null, value)`, reverse-write parents, commit the original owner, and preserve assignment expression result ownership. Append-with-keyed-suffix, nested unset, reference-returning `offsetGet()`, arbitrary alias/static roots, broader references/COW, cleanup/unwind breadth, spread/unpack, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=14 failures=0`, covering fmt, diff checks, nested owner-stack units, exact generated-C and linked executable nested append proofs, nested ArrayAccess regressions, property-held ArrayAccess owners, descriptor closures, magic static calls, static properties, source-call references, class aliases, and `cargo check -p phpc`. |
| `f65be9b1` | Generated C now routes selected static method source calls through runtime `__callStatic` fallback. Declared static method hits still win, missing or inaccessible static methods fall back to public static `__callStatic($name, $args)`, non-static methods called statically stay hard failures, and literal class, object-static receiver, `self::`, `parent::`, and bounded declared-frame `static::` calls share the runtime lookup-plus-invoke boundary. Named-argument magic fallback, malformed magic signatures, traits/interfaces/effective method tables, aliases/autoload, callable-object static magic shapes, `$args` reference/COW parity, and LLVM/direct assembly parity remain blocked. | Primary integration gates passed with `SUMMARY passes=25 failures=0`, covering fmt, diff check, runtime `__callStatic` lookup-plus-invoke, runtime method lookup helpers, native source-call signature fallback contracts, generated-C source and linked executable magic-static proofs, named static magic fallback blocker, descriptor closures, object static-property receivers, named arguments, magic dynamic/static calls, object-static/class-context/self-parent/late-static/static-property paths, typed properties, exact imports, source-call references, nested ArrayAccess, class aliases, and `cargo check -p php_runtime -p phpc`. |
| `c2ffab4b` | Generated C now initializes declared static-property storage through the bulk metadata/default ABI, consuming property name, visibility, type, default, and static-flag arrays from class metadata. Static defaults are registered only for static properties while non-static metadata remains visible to static lookup shadowing, and typed static-property defaults/writes now use the runtime diagnostic path. Dynamic class/property names, static-property references, unset/isset/empty, compound mutation, increment/decrement, `??=`, append/nested static-property offset mutation, magic/static overloading, traits/interfaces, full type parity, and LLVM/backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=16 failures=0`, covering fmt, runtime static-property tests, generated-C static-property native-link proofs, object static-property receivers, descriptor closure calls, named arguments, magic/runtime dynamic method calls, exact imports, late static, typed declared instance properties, source-call references, class aliases, nested ArrayAccess, diff check, and `cargo check -p php_runtime -p phpc`. |
| `55aef1ee` | Generated C now routes proven descriptor-backed closure calls through shared `NativeCallArgumentsHandle` production and descriptor closure invoke helpers for result, value, reference, and discard consumers. The compiler selects this path from callable identity facts, not from a source spelling, fixture, arity, local variable name, or generated-C substring. Unknown/mixed callables, callable arrays, invokable objects, non-descriptor closure handoff, descriptor/method/closure reference-return breadth, spread/unpack, broader request-state handoff, and LLVM/backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=28 failures=0`, covering runtime closure invoke helpers, generated-C source proof, linked descriptor-closure executable proof, named arguments, exact imports, magic/runtime/dynamic method source calls, late/static properties, object static-property receivers, object-static calls, typed properties, try/finally, ArrayAccess reference slots, property-held and nested ArrayAccess, source-call references, class aliases, fmt, diff check, and `cargo check -p php_runtime -p phpc`. |
| `147ad3b5` | Generated C now routes selected object and declared class-string static-property receivers through a shared runtime receiver-scope helper before using request-owned static-property storage for direct reads and plain assignments. Object instance receivers and class-string values share the same runtime scope derivation and static read/write ABI; unsupported receiver scopes still report diagnostics and clean up owned handles. Top-level `static::$prop`, dynamic property names, static-property references, compound mutation/unset/isset/empty, magic/static overloading, traits/interfaces, autoload breadth, LLVM parity, and reference/COW static-property breadth remain blocked. | Primary integration gates passed with `SUMMARY passes=20 failures=0`, covering runtime receiver-scope helpers, generated-C object/class-string static-property source and linked executable proof, named argument, magic receiver, exact import, declared static-property, late-static, runtime dynamic method, typed property, nested/property-held ArrayAccess, reference-source, constructor value-return, non-local property, source-call reference alias, class-alias, fmt, diff checks, and `cargo check -p php_runtime -p phpc`. |
| `28ce0faa` | Generated C now routes selected nested ArrayAccess pre/post increment and decrement through the generalized owner-stack write context for direct-variable and property-held roots, preserving pre/post expression results while doing by-value `offsetGet()` descent, leaf read, native increment/decrement replacement, leaf `offsetSet()`, reverse parent writeback, and root commit. Nested `??=`, append, unset, reference-returning `offsetGet()`, arbitrary alias roots, broad references/COW, cleanup/unwind breadth, spread/unpack, and LLVM/backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=47 failures=0`, covering nested owner-stack units, generated-C source and linked executable proof for direct/property-root nested increment/decrement, nested assignment and compound-assignment regressions, reference-returning and unsupported nested mutation blockers, property-held ArrayAccess owner regressions, named call arguments, magic receiver calls, exact function/const imports, late/static properties, runtime dynamic methods, typed properties, source-call byref, constructor value-return diagnostics, class aliases, fmt, diff checks, and `cargo check -p phpc`. |
| `425f1ef5` | Generated C now lowers selected named call arguments for compiler-known direct user functions and method/static/dynamic source-call carriers through shared source-order/parameter-order normalization. The normalizer preserves source-order evaluation, binds named values to required/optional/default/by-reference/variadic parameters, rejects duplicate/unknown/positional-after-named/unpack shapes at explicit boundaries, and then feeds parameter-order `NativeCallArgumentsHandle` construction. Named builtins, constructors, unsupported dynamic callables, magic fallback named-argument parity, spread/unpack, broader signature metadata, and LLVM/direct-assembly parity remain blocked. | Primary integration gates passed with `SUMMARY passes=38 failures=0`, covering parser/AST named-argument boundaries, shared normalizer unit proof, generated-C source proof for direct user functions and method/static/dynamic source-call carriers, linked named function/method/dynamic-method executable proof, explicit unsupported builtin/dynamic fallback/unpack blockers, magic dynamic method regressions, exact function/const imports, late/static properties, typed properties, nested and property-held ArrayAccess, source-call references, class aliases, runtime magic lookup, fmt, diff check, cargo check, and cached diff check. |
| `eedd0879` | Generated-C dynamic receiver method calls on compiler-known receivers with declared instance `__call($name, $args)` now use the shared runtime lookup-plus-invoke dispatcher. Normal method hits still win, missing or inaccessible methods fall back to public non-static `__call`, and magic arguments pack the original method name plus value snapshots of the original call arguments. `__callStatic`, malformed magic signature parity, traits/interfaces/effective tables, aliases/autoload, callable-object fallbacks, full `$args` reference/COW alias behavior, and LLVM/direct-assembly parity remain blocked. | Primary integration gates passed with `SUMMARY passes=18 failures=0`, covering runtime magic `__call` lookup-plus-invoke, method lookup filtering, generated-C magic dynamic source proof, linked magic dynamic executable proof, dynamic/class-context/object-static/late-static/static-property/typed-property/nested ArrayAccess/exact const/source-call reference/class-alias regressions, fmt, diff check, and cached diff check. |
| `a4d151b8` | Generated C now lowers exact parser-resolved `use const` aliases for same-compilation-unit scalar user constants and supported builtin constants through shared constant metadata/import markers. Exact imported misses reject without namespace/global fallback; ordinary bare constants, arrays/dynamic constant expressions, class constants, include/autoload discovery, dynamic alias roots, function-frame constant lookup, LLVM constant lowering, and broad backend/global constant parity remain blocked. | Primary integration gates passed with `SUMMARY passes=23 failures=0`, covering const-import generated-C source proof, exact-miss and declaration-shape blockers, parser/runtime exact lookup rules, namespace/import focused module, linked exact imported-const executable proof, LLVM constant blockers, function-import regressions, runtime dynamic methods, late-static properties, nested ArrayAccess RMW, typed properties, class aliases, try/finally cleanup, fmt, diff check, and cached diff check. |
| `bf8fd335` | Generated C now routes declared-frame `static::$prop` reads/writes through shared relative static-property runtime storage using the active called-scope receiver, so inherited static-property methods update descendant storage instead of lexical `self`. Top-level `static::$prop`, object static-property receivers, dynamic class/property names, static-property references, compound mutation/unset/isset/empty, magic/static overloading, traits/interfaces, autoload breadth, LLVM parity, and reference/COW static-property breadth remain blocked. | Primary integration gates passed with `SUMMARY passes=15 failures=0`, covering generated-C source proof, linked late-static property executable proof, static-property native-link and runtime filters, runtime dynamic method and late-static method filters, typed declared instance-property and object-static regressions, nested ArrayAccess RMW source and executable regressions, runtime dynamic method linked executable proof, typed-property failure proof, fmt, diff check, and cached diff check. |
| `ba5769e2` | Generated C now routes selected nested ArrayAccess compound assignments through the generalized owner stack for direct-variable and property-held roots, including by-value `offsetGet()` descent, leaf read, native binary replacement, leaf `offsetSet()`, reverse parent writeback, and root commit. Nested `??=`, append, increment, decrement, reference-returning `offsetGet()`, arbitrary alias roots, broader COW/reference forms, cleanup/unwind breadth, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=24 failures=0`, covering nested owner-stack unit proof, generated-C direct/property-root RMW source proof, linked nested ArrayAccess RMW executable proof, existing nested assignment source and executable regressions, reference-returning `offsetGet()` and non-assignment mutation blockers, property-held and direct ArrayAccess owner regressions, relative static properties, runtime dynamic methods, late-static calls, typed properties, fmt, diff check, and cached diff check. |
| `cfc7c0ee` | Generated C now routes declared-frame `self::$prop` and `parent::$prop` reads/writes through shared relative static-property runtime storage, using one program-wide request storage handle across top-level and method-frame static-property operations. `static::$prop`, object-static property receivers, dynamic class/property names, static-property references, compound mutation/unset/isset/empty, magic/static overloading, traits/interfaces, autoload breadth, LLVM parity, and reference/COW static-property breadth remain blocked. | Primary integration gates passed with `SUMMARY passes=20 failures=0`, covering generated-C source proof, linked self/parent static-property execution, declared static-property storage, unsupported static-property blockers, self/parent static calls, late-static, dynamic methods, object-static carriers, runtime static-property storage, typed instance properties, constructor value diagnostics, class aliases, try/finally cleanup, property-held and nested ArrayAccess, exact/unsupported imports, fmt, and diff check. |
| `7ae07fd7` | Generated-C receiver dynamic method calls with unknown runtime method-name expressions now route through the shared lookup-plus-invoke source-call carrier when receiver facts are compiler-known and the class has no declared `__call`; runtime lookup normalizes scalar method-name values before access-context lookup. Declared `__call`/`__callStatic`, magic argument packing, traits/interfaces, arbitrary aliases/autoload, callable-object fallbacks, broader byte/encoding diagnostic parity, and LLVM parity remain blocked. | Primary integration gates passed with `SUMMARY passes=20 failures=0`, covering runtime method-name normalization, access-context preflights, signature fallback contracts, generated-C source proof, linked runtime dynamic method executable proof, magic dynamic method blockers, dynamic/class-context/object-static carriers, late-static, property-held ArrayAccess, source-call byref, runtime dynamic builtins, declared static properties, typed instance properties, static-property runtime storage, fmt, and diff check. |
| `441de993` | Generated C now routes bounded declared-frame `static::method(...)` calls through runtime called-scope handles and static source-call carriers, while preserving lexical class context only for protected/private access checks. Descendant-only targets, override-visible late-static breadth, traits, interfaces, magic `__callStatic`, dynamic static properties, `static::class`, `static::$prop`, LLVM parity, and broad inheritance/interface resolution remain blocked. | Primary integration gates passed with `SUMMARY passes=21 failures=0`, covering late-static source and linked executable proof, runtime called-scope dispatch, typed instance-property regressions, try/finally, imports, dynamic/object/static/self-parent/class-context carriers, property-held and nested ArrayAccess, declared static properties, source-call byref, constructor value diagnostics, non-local object-property owners, class aliases, object call dispatch handles, fmt, and diff check. |
| `f1816273` | Generated C now lowers selected declared typed instance properties by emitting per-property type/default metadata, allocating declared objects through a typed-property metadata runtime ABI, initializing defaults, and routing known typed instance-property writes through diagnostic mutation. Typed static properties, dynamic/magic properties, unsupported object type declarations, broad reference/COW property semantics, and LLVM/backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=24 failures=0`, covering typed instance-property generated source and linked executable success/failure proof, runtime declared-class allocation metadata, unsupported declared-class feature blockers, finally return cleanup, exact imports, late-static/object-static/class-context carriers, static-property storage, non-local property owners, property-held and nested ArrayAccess, source-call byref, constructor value-return diagnostics, class aliases, declared object-property regressions, fmt, and diff check. |
| `2ca9d0e3` | Generated C now routes bounded function and method returns through active `finally` bodies by reporting finalizer output, enqueueing cleanup-surface operands, transferring return terminal kind with a non-empty cleanup list, and then taking the original return value. Top-level try returns, throw/catch, exit/goto unwind, finally-return replacement, destructor/shutdown ordering, return operands with source-call cleanup, and full exception/finally semantics remain blocked. | Primary integration gates passed with `SUMMARY passes=36 failures=0`, covering native diagnostic ABI, source-call carriers, nested ArrayAccess smoke, terminal-kind ABI, try/finally source and linked executable proof, top-level try-return blocker, try-unwind blockers, exact function-import regressions, dynamic method/object-static/class-context carriers, property-held ArrayAccess owners, declared static-property storage, byref/source-call alias transfer, constructor value-return diagnostics, non-local owner commits, class aliases, native function destructor boundary, shutdown functions, fmt, and diff check. |
| `e8268187` | Generated C now lowers selected nested ArrayAccess assignments through a generalized owner stack for direct-variable and property-held roots, including by-value `offsetGet()` descent, leaf `offsetSet()`, reverse parent writeback, and root commit. Reference-returning `offsetGet()`, nested RMW/`??=`/append/increment/decrement, broader COW/reference forms, and backend parity remain blocked. | Nested owner-stack unit proof, generated-C direct/property-root source proof, linked nested ArrayAccess executable proof, reference-returning `offsetGet()` blocker, non-assignment nested mutation blockers, direct nested append guard, property-held ArrayAccess owner regressions, declared static-property regressions, object-static/class-context/source-call byref regressions, constructor value-return and non-local property owner regressions, runtime builtin and class-alias regressions, fmt, diff check. |
| `5bacf1aa` | Generated C now lowers exact parser-resolved `use function` aliases for same-compilation-unit user functions and selected signature-backed runtime callable builtins through shared source-call arguments and lookup/invoke carriers, preserves exact missing imported runtime names, and keeps unsupported imported runtime builtins such as `count()` behind lookup-before-argument side-effect guards. Dynamic alias roots, include/require discovery, autoload, broad runtime builtin imports, native const-import lowering, and backend parity remain blocked. | Function-import namespace proof, generated-C imported runtime-builtin source proof, linked imported user-function executable proof, linked imported runtime-builtin executable proof, unsupported imported builtin side-effect guard, dynamic builtin regressions, dynamic method regressions, object-static regressions, property-held ArrayAccess owner regressions, declared static-property regressions, terminal-return regression, runtime callable builtin regressions, fmt, diff check. |
| `5a4f10a5` | Generated C now routes compiler-known dynamic receiver-method calls with known string method names through shared source-call carriers, including class-context access for declared method frames and runtime rejection of static descriptors used through object dispatch, while keeping declared `__call`, broad magic dispatch, traits/interfaces, unknown scalar method names, late-static behavior, and unsupported receiver shapes blocked. | Runtime access-context/static-through-object diagnostic proof, generated-C dynamic receiver source proof, linked dynamic receiver executable proofs, class-context dynamic private-call proof, magic boundary proof, object-static/class-context/self-parent/static-method carrier regressions, property-held ArrayAccess append/increment regressions, declared static-property storage/production regressions, source-call byref and direct reference-return regressions, runtime builtin regressions, fmt, diff check. |
| `ad74d35b` | Generated C now lowers selected property-held ArrayAccess append and pre/post increment/decrement through existing object-property reference owners for literal and single-known dynamic properties, commits mutated holders through reference writeback, preserves expression-result ownership and conversion diagnostics, and keeps nested offsets, unknown dynamic properties, reference-returning `offsetGet()`, static-property owners, broad COW/reference semantics, and backend parity blocked. | Property-held ArrayAccess source and linked executable append/increment proofs, conversion-diagnostic executable proof, unsupported-owner blocker proof, adjacent ArrayAccess RMW/`??=`/unset owner regressions, non-local object-property owner regressions, static-property production/storage regressions, object-static source-call regressions, source-call byref and self/parent regressions, class-context method-call regressions, fmt, diff check. |
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
| Callable access context and class metadata | **100%** `[####################]` | **100%** `[####################]` | **65%** `[#############-------]` | Shared runtime access-context policy, lookup-plus-invoke argument ownership, source-call result carrier selectors, selected production source-call carrier emission, direct generated user-function lookup-plus-invoke production, direct user-function reference-return frame consumers, bounded generated-C method/static source-call production including default, variadic, and selected named-argument frame-compatible arities, method/static source-call target and binding operands, method/static signature fallback selection, allocatable-class metadata, generated-C user-class metadata-exists consumers, and value-returning class metadata consumers are integrated for selected function/method/static/constructor/class lookup preflights. Constructor execution, dynamic method names, spread and unsupported named call families, runtime/builtin/inherited/trait/interface signature metadata, non-descriptor closure argument-handle ownership, namespace/function/const fallback, autoload, magic, and full visibility parity remain open. |
| ArrayAccess compiler consumers | **100%** `[####################]` | **100%** `[####################]` | **75%** `[###############-----]` | Generated-C direct-object/direct-variable read, `isset`, `empty`, `??`, write, append, keyed-suffix append, unset, compound assignment, `??=`, selected property-held literal/single-known dynamic object-property read/write/RMW/`??=`/unset owners, and selected nested assignment/RMW/increment/decrement/append/`??=`/unset/keyed-append-suffix owner stacks are integrated for compiler-known generated declared `ArrayAccess` objects. Reference-returning `offsetGet`, arbitrary alias roots, references/COW, cleanup/unwind, and backend parity remain open. |
| ReferenceSlot owner facts | **100%** `[####################]` | **100%** `[####################]` | **50%** `[##########----------]` | Compiler-visible native reference handles can recover facts, source owners, and commit writeback for selected variable, non-local object-property assignment, and property-held ArrayAccess paths. Arbitrary alias roots, request/superglobal path facts, broader property-held reference binding, closure callback fact transport, references/COW, and backend parity remain open. |
| Callable identity return summaries | **100%** `[####################]` | **100%** `[####################]` | **60%** `[############--------]` | Generated functions, declared methods/static methods, descriptor closures, known strings, definite `__invoke` objects, compiler-known callable arrays, and selected direct user-function reference-return frames can publish or consume selected return facts. Unknown runtime callables, builtins, non-descriptor closures, recursive summaries, descriptor/method/closure reference returns, property/magic producers, references/COW, and backend parity remain open. |
| Dynamic object/interface fact carrier | **100%** `[####################]` | **100%** `[####################]` | **65%** `[#############-------]` | Generated declared objects, known dynamic class-name `new`, copies, gotos, branches, generated-callable returns, descriptor closures, known string/invokable/callable-array summaries, compiler-visible reference slots, and selected declared static-property producers feed existing object/interface consumers. Broader properties, clones, dynamic/static property shapes, arbitrary symbols, unknown runtime callables/arrays, and non-descriptor closures remain open. |
| Cleanup/unwind execution | **30%** `[######--------------]` | **30%** `[######--------------]` | **30%** `[######--------------]` | Requirement/preflight boundaries, cleanup result consumers, cleanup report bridges, terminal cleanup transfer ABI, cleanup-frame producer queues, cleanup-frame source metadata, nested cleanup-frame stack aggregation, and cleanup-frame enqueue validation are integrated. Actual exception propagation, catch/finally/destructor/shutdown execution, production cleanup operand enqueueing from real control flow, terminal-kind lowering, and object lifetime cleanup are still not implemented. |
| Broad dirty lane extraction backlog | **0%** `[--------------------]` | **35%** `[#######-------------]` | **35%** `[#######-------------]` | Dirty call, diagnostic, object, control-flow, symbol, byte/string, and array lanes remain evidence pools until split into fresh current-head candidates with focused proof. |

## Done

- Runtime callable table plus call arguments/frame/result ABI.
- Parser/AST named call-argument nodes plus shared call-argument normalization
  bind source-order positional/named arguments to parameter-order required,
  optional/default, by-reference, and variadic slots for selected generated-C
  direct user-function and method/static/dynamic source-call carriers.
- Runtime callable-value dispatch for selected function names, callable arrays,
  descriptor closures, inherited methods, bound receivers, and object
  `__invoke`.
- Runtime lookup-plus-invoke helpers for direct callable, receiver-method, and
  static-method families consume `NativeCallArgumentsHandle` exactly once
  across lookup failure, invoke failure, result handoff, and discard/value/
  reference consumers.
- Runtime receiver-method lookup-plus-invoke can fall back to selected declared
  instance `__call($name, $args)` for missing or inaccessible object receiver
  methods, packing the original method name and value snapshots of the original
  argument slots while preserving normal class-context method hits.
- Runtime static-method lookup-plus-invoke can fall back to selected public
  static `__callStatic($name, $args)` for missing or inaccessible static method
  calls while preserving normal declared static hits and non-static static-call
  failures.
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
- Generated-C selected static method source calls route missing or inaccessible
  static targets through shared runtime `__callStatic` lookup-plus-invoke
  fallback without generated method-name ladders.
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
- Generated-C declared static-property storage initialization consumes bulk
  declared-property metadata/default arrays for names, visibility, types,
  defaults, and static flags, including selected typed static-property
  defaults/writes through runtime diagnostics.
- Generated-C selected object and declared class-string static-property
  receiver reads and plain assignments route through runtime receiver-scope
  derivation and request-owned static-property storage.
- Generated-C proven descriptor-closure calls route through shared call
  argument handles and runtime closure-invoke result carriers.
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
- Generated-C selected nested ArrayAccess assignment, compound-assignment,
  pre/post increment/decrement, append, and `??=` owner stacks for
  direct-variable and property-held roots, including by-value descent, leaf
  mutation, reverse parent writeback, lazy null-coalescing RHS branches, and
  root commit.
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
- Nested ArrayAccess root keyed-suffix append/reference-returning production,
  non-direct property-held ArrayAccess forms, unknown dynamic property owners,
  dynamic class-name holders without definite facts, and reference-returning
  ArrayAccess semantics on top of the integrated object-property owner
  boundary.
- LLVM static-property producers, generated-C dynamic and broader
  object/static-property shapes, broader static-property reference/`??=`,
  unset/isset/empty lowering, dynamic/magic property lowering, and full
  method/object model execution beyond the selected generated `$this`
  assignment and declared static-property subsets.
- Full reference/COW identity and arbitrary alias-root writeback.
- Actual exception/Throwable propagation, catch matching/binding, `finally`,
  destructors, shutdown cleanup, and object lifetime cleanup.
- Full SPL autoload, broader class-alias parity, broader const-import
  discovery/fallback, broader function-import discovery/fallback,
  broader namespace/function/const fallback, broader visibility,
  named-argument `__callStatic` fallback, malformed magic signature parity,
  broader magic-call coverage, broader constructor allocation/execution,
  spread arguments,
  unsupported named builtin/constructor/fallback call families, and return
  references.
- Broader source-call production lowering over expression-owned
  `NativeCallResultHandle` carriers, including remaining dynamic/magic method
  shapes, late-static binding, constructor allocation,
  non-descriptor closure invocation ownership, direct function/method/static produced-call
  by-reference alias transfer, unknown runtime callable reference returns,
  runtime/builtin/inherited/trait/interface signature metadata, unsupported
  named argument consumers, and spread ownership.
- Remaining semantic diagnostic-result operand migration for throw/exit/default
  return terminals, cleanup frame/result production from real control flow,
  lvalue, reference, RMW, and call-argument families; exact PHP diagnostics,
  source ordering, suppression/custom handlers, and backend parity across
  generated C, LLVM, and direct assembly.
- Pending diagnostic production from real control-flow cleanup, remaining
  terminal-kind lowering over the terminal transfer ABI, and exact
  `finally`/destructor/shutdown sequencing.

## Latest Focused Verification

For `1202542e`:

- `cargo fmt --all -- --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test -p php_runtime static_property -- --nocapture`
- `cargo test -p phpc --test native_link static_property -- --nocapture`
- `cargo test -p phpc --test native_link static_property_mutation -- --nocapture`
- `cargo test -p phpc --test native_link object_static_property_receiver -- --nocapture`
- Focused adjacent regressions for descriptor closures, magic static calls,
  source-call references, nested ArrayAccess, property-held ArrayAccess, and
  class aliases, plus `cargo check -q -p php_runtime -p phpc`. Primary
  integration log:
  `/home/claude/supervised-php-compiler/state/workers/logs/phpc-primary-static-property-mutation-r4-integration-20260527.gates.log`
  (`SUMMARY passes=14 failures=0`, sha256
  `f74ebb2b211e2a3b08f6e47879acc6e62fefbf99e3d61b5a279072211066835e`).

For `8a8d85ed`:

- `cargo fmt --all -- --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test -p phpc --lib native_nested_arrayaccess_owner_stack -- --test-threads=1`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_nested_arrayaccess_null_coalesce_owner_stack_for_direct_and_property_roots -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_nested_arrayaccess_null_coalesce_owner_stack_program -- --exact --test-threads=1`
- Focused adjacent regressions for nested ArrayAccess, property-held
  ArrayAccess, descriptor closures, magic static calls, static properties,
  source-call references, and class aliases, plus `cargo check -q -p phpc`.
  Primary integration log:
  `/home/claude/supervised-php-compiler/state/workers/logs/phpc-primary-nested-arrayaccess-nullcoalesce-r3-integration-20260527.gates.log`
  (`SUMMARY passes=14 failures=0`, sha256
  `eccbf159904d5c93cbb338668436db77b726d6d08174b100069ab96fca53318a`).

For `f65be9b1`:

- `cargo fmt --all -- --check`
- `git diff --check`
- `cargo test -p php_runtime call_static -- --nocapture`
- `cargo test -p php_runtime native_method_lookup -- --nocapture`
- `cargo test -p php_runtime native_lookup_plus_invoke -- --nocapture`
- `cargo test -p phpc --lib native_method_static_signature_fallback_contract -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_magic_static_methods_through_runtime_dispatch_boundary -- --exact --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_magic_static_method_source_call_program -- --exact --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_blocks_named_static_magic_fallback_without_shared_contract -- --exact --nocapture`
- Focused adjacent regressions for descriptor closures, object static-property
  receivers, named arguments, magic dynamic/static calls, object-static,
  class-context, self/parent/static, late static, static properties, typed
  instance properties, exact imports, source-call references, nested
  ArrayAccess, and class aliases.
- `cargo check -p php_runtime -p phpc`
- Primary integration log:
  `/tmp/phpc-primary-callstatic-magic-r2-integration-20260527.gates.log`
  (`SUMMARY passes=25 failures=0`).

For `c2ffab4b`:

- `cargo fmt --all -- --check`
- `cargo test -p php_runtime static_property -- --nocapture`
- `cargo test -p phpc --test native_link static_property -- --nocapture`
- `cargo test -p phpc --test native_link object_static_property_receiver -- --nocapture`
- `cargo test -p phpc --test native_link descriptor_closure -- --nocapture`
- Focused adjacent regressions for named arguments, magic/runtime dynamic
  methods, exact imports, late static, typed declared instance properties,
  source-call references, class aliases, and nested ArrayAccess.
- `git diff --check`
- `cargo check -p php_runtime -p phpc`
- Primary integration log:
  `/tmp/phpc-primary-typed-static-property-r11-integration-20260527.gates.log`
  (`SUMMARY passes=16 failures=0`).

For `55aef1ee`:

- `cargo fmt --all -- --check`
- `cargo test -p php_runtime native_closure_invoke_helpers_bridge_call_arguments_to_call_results -- --nocapture`
- `cargo test -p phpc generated_c_descriptor_closure_calls_use_shared_call_arguments_and_results -- --nocapture`
- `cargo test -p phpc --test native_link descriptor_closure -- --nocapture`
- Focused adjacent regressions for named arguments, exact imports,
  magic/runtime/dynamic method source calls, late/static properties,
  object static-property receivers, object-static calls, typed properties,
  try/finally, ArrayAccess reference slots, property-held and nested
  ArrayAccess, source-call references, and class aliases. Primary integration
  log:
  `/tmp/phpc-primary-closure-call-production-r8-integration-20260527.gates.log`
  (`SUMMARY passes=28 failures=0`).

For `147ad3b5`:

- `cargo fmt --all -- --check`
- `cargo test -p php_runtime static_property -- --nocapture`
- `cargo test -p phpc --test native_link object_static_property_receiver -- --nocapture`
- Focused adjacent regressions for named arguments, magic receiver calls, exact
  imports, declared/late static properties, runtime dynamic methods, typed
  properties, nested/property-held ArrayAccess, reference-slot owners,
  constructor value-return diagnostics, non-local object properties,
  source-call reference aliases, and class aliases. Primary integration log:
  `/tmp/phpc-primary-static-property-object-receiver-r4-integration-20260527.gates.log`
  (`SUMMARY passes=20 failures=0`).

For `28ce0faa`:

- `cargo fmt -q -p phpc -- --check`
- `cargo test -q -p phpc --lib native_nested_arrayaccess_owner_stack -- --test-threads=1`
- `cargo test -q -p phpc --test native_link native_executable_c_source_routes_nested_arrayaccess_increment_decrement_owner_stack_for_direct_and_property_roots -- --exact --test-threads=1`
- `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_nested_arrayaccess_increment_decrement_owner_stack_program -- --exact --test-threads=1`
- Focused adjacent regressions for nested assignment/RMW, unsupported
  reference-returning/non-assignment nested mutations, property-held ArrayAccess
  owners, named call arguments, magic receiver calls, exact function/const
  imports, late/static properties, runtime dynamic methods, typed properties,
  source-call by-reference arguments, constructor value-return diagnostics, and
  class aliases. Primary integration log:
  `/tmp/phpc-primary-nested-arrayaccess-mutation-r3-integration-20260527.gates.log`
  (`SUMMARY passes=47 failures=0`).

For `425f1ef5`:

- `cargo fmt -- --check`
- `git diff --check`
- `cargo test -q -p phpc --lib call_arguments::tests:: --`
- `cargo test -q -p phpc --test syntax_boundaries named_arguments_parse_as_source_ordered_call_argument_nodes -- --exact --nocapture`
- `cargo test -q -p phpc --test syntax_boundaries emit_ir_rejects_named_builtin_arguments_at_codegen_boundary -- --exact --nocapture`
- `cargo test -q -p phpc --test native_link native_executable_c_source_lowers_named_user_function_arguments_through_shared_normalization -- --exact --nocapture`
- `cargo test -q -p phpc --test native_link native_executable_c_source_lowers_named_method_source_call_arguments_through_carriers -- --exact --nocapture`
- `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_named_dynamic_method_source_call_argument_program -- --exact --nocapture`
- Focused adjacent regressions for magic dynamic methods, exact function/const
  imports, late/static properties, typed properties, nested/property-held
  ArrayAccess, source-call references, class aliases, runtime magic lookup, and
  native source-call carrier contracts. Primary integration log:
  `/tmp/phpc-primary-named-callargs-r15-integration-20260527.gates.log`
  (`SUMMARY passes=38 failures=0`).

For `eedd0879`:

- `cargo fmt -- --check`
- `git diff --check`
- `cargo test -q -p php_runtime --lib tests::native_method_lookup_plus_invoke_dispatches_missing_and_inaccessible_methods_to_magic_call -- --exact --nocapture`
- `cargo test -q -p php_runtime --lib native_method_lookup -- --nocapture`
- `cargo test -q -p php_runtime --lib native_lookup_plus_invoke -- --nocapture`
- `cargo test -q -p phpc --test native_link native_executable_c_source_routes_magic_dynamic_methods_through_runtime_dispatch_boundary -- --exact --nocapture`
- `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_magic_dynamic_method_source_call_program -- --exact --nocapture`
- Focused adjacent regressions for dynamic method source calls, class-context
  method calls, object-static calls, late-static/static-property paths, typed
  properties, nested ArrayAccess, source-call reference aliases, class aliases,
  and exact const imports. Primary integration log:
  `/tmp/phpc-primary-magic-call-r2-integration-20260527.gates.log`
  (`SUMMARY passes=18 failures=0`).

For `e8268187`:

- `cargo fmt -q -p phpc -- --check`
- `cargo test -q -p phpc --lib native_nested_arrayaccess_owner_stack -- --test-threads=1`
- `cargo test -q -p phpc --test native_link native_executable_c_source_routes_nested_arrayaccess_owner_stack_for_direct_and_property_roots -- --exact --test-threads=1`
- `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_nested_arrayaccess_owner_stack_program -- --exact --test-threads=1`
- `cargo test -q -p phpc --test native_link native_executable_c_source_rejects_nested_arrayaccess_reference_returning_offsetget -- --exact --test-threads=1`
- `cargo test -q -p phpc --test native_link native_executable_c_source_rejects_nested_arrayaccess_non_assignment_mutations -- --exact --test-threads=1`
- Focused adjacent regressions for property-held ArrayAccess owners, declared
  static-property storage, object-static and class-context source-call carriers,
  source-call by-reference aliasing, constructor value-return diagnostics,
  non-local property owner commits, runtime dynamic builtins, and class-alias
  metadata. Primary integration log:
  `/tmp/phpc-primary-nested-arrayaccess-r9-integration-20260527.gates.log`
  (`SUMMARY passes=41 failures=0`).

For `5bacf1aa`:

- `cargo fmt --all -- --check`
- `git diff --check`
- `cargo test -q -p phpc --test namespace_resolution function_imports_resolve_aliases_and_keep_non_imported_fallback -- --exact`
- `cargo test -q -p phpc --test namespace_resolution function_imports_use_exact_lookup_without_global_suffix_fallback -- --exact`
- `cargo test -q -p phpc --test namespace_resolution missing_imported_function_and_non_imported_namespaced_calls_report_distinct_runtime_names -- --exact`
- `cargo test -q -p phpc --test namespace_resolution generated_c_lowers_imported_runtime_builtin_function_boundary -- --exact`
- `cargo test -q -p phpc --test namespace_resolution generated_c_rejects_qualified_imported_type_builtin_without_exact_user_function -- --exact`
- `cargo test -q -p phpc --test native_link native_executable_c_source_lowers_exact_imported_user_function_aliases -- --exact`
- `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_exact_imported_user_function_alias_program -- --exact`
- `cargo test -q -p phpc --test native_link native_executable_c_source_lowers_exact_imported_runtime_builtin_aliases -- --exact`
- `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_exact_imported_runtime_builtin_alias_program -- --exact`
- `cargo test -q -p phpc --test native_link native_executable_c_source_preserves_unsupported_imported_builtin_lookup_boundary -- --exact`
- `cargo test -q -p phpc --test native_link emit_exe_reports_unsupported_imported_builtin_before_arguments -- --exact`
- Focused adjacent regressions for dynamic builtins, dynamic methods,
  object-static source calls, property-held ArrayAccess owners, declared
  static-property storage, terminal returns, and runtime callable builtin
  signatures. Primary integration log:
  `/tmp/phpc-primary-function-import-r12-integration-20260527.gates.log`
  (`SUMMARY passes=26 failures=0`).

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
