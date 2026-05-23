# Architecture

## Pipeline

The intended compiler pipeline is:

```text
PHP source
-> lexer/parser
-> AST
-> semantic analysis
-> IR
-> lowering/type specialization
-> LLVM IR text
-> assembly/object/executable
-> linked runtime
```

Milestone 1 implements the lexer, parser, AST, a direct interpreter/runtime
execution path, and a narrow LLVM IR text emitter for simple straight-line code.
The interpreter runs top-level statements in a global symbol table and creates a
fresh local symbol table for each user-function call. Local scopes can import
direct root variables through `global $name, ...;`; imported names route direct
reads and writes through the shared root symbol table, while `unset($name)`
drops the local import without deleting the root value. If the same local name
was previously bound to a local direct array-offset alias, importing the global
name clears that stale local alias binding so later reads/writes use the root
symbol slot. Top-level `global`
declarations preserve existing root values and materialize missing listed names
as `null`, matching the reached WordPress bootstrap initialization shape.
Direct string-keyed `$GLOBALS['name']` reads and writes are a bounded
root-symbol-table route for the reached WordPress object-cache bootstrap
assignment. Direct string-keyed `$GLOBALS['name'] =& $value` may also join the
same bounded alias metadata used by covered direct array-offset references, so
a selected root global, the source variable group, and the covered array slot
observe the same value in the current top-level/root-symbol-table slice. This
same alias metadata also backs the bounded by-reference `foreach` path for
direct nested array-offset roots, request-bag roots such as
`$_REQUEST["payload"]`, and string-keyed nested `$GLOBALS` paths such as
`$GLOBALS["bag"]["child"]`. The same path now covers direct visible
object-property array roots such as `$object->items` and
`$object->items["child"]`, plus direct dynamic-property spellings such as
`$object->{$name}` and `$object->{$name}["child"]` when the evaluated property
is visible in the current public/private/protected context, using the existing
public/context property alias root instead of a general PHP reference
container. By-reference `foreach` over an ordinary direct non-`Traversable`
object variable, such as `foreach ($object as $key => &$value)`, snapshots the
object's currently initialized public properties into ordered foreach entries
and binds each visited property through a public object-property alias root.
Loop writes therefore mutate the object's public properties, and the
post-loop value variable remains routed to the last visited public property
until `unset($value)`. By-value `foreach` over ordinary non-`Traversable`
objects snapshots the initialized public property names and reads each
property value when that name is reached, matching the current PHP-observed
behavior where a mutation to a later public property is visible when the loop
arrives at that property. By-value `foreach` over bounded userland
`Iterator` objects dispatches public `rewind()`, `valid()`, `current()`,
`key()`, and `next()` in PHP order through the existing method-call path;
`IteratorAggregate::getIterator()` is bounded to returning one of those
`Iterator` objects. When a by-value `Iterator::current()` result is an array
copied from a direct public `$this->property` array or selected literal-key
bucket such as `$this->callbacks[$priority]`, the interpreter can mirror
covered nested public object-property reference slots into the loop variable's
copied array. By-reference foreach over userland `Iterator` objects is
rejected with PHP-parity diagnostics because PHP itself forbids that shape.
Bounded non-direct named and dynamic property holder expressions in
by-reference `foreach`, such as `$holders["bag"]->items["child"]`,
`$holders["bag"]->{$name}["child"]`, method-context
`$this->holder()->items`, or `$this->holder()->{$name}`, evaluate the holder
once into a private temporary object root and then use that same public/context
property alias machinery when the selected property is visible. Direct and
visible property-held `ArrayAccess` offset-array roots, such as
`foreach ($bag["outer"] as &$value)` and
`foreach ($holder->{$name}["outer"] as &$value)`, reuse the exact bounded
by-reference `offsetGet($offset) { return $this->property[$offset]; }` bridge
and then apply the same foreach array-slot alias machinery to the returned
backing array slot. When a direct `ArrayAccess` object uses a public
`offsetGet($offset)` body with the same exact
`return $this->property[$offset];` shape, normal direct-variable assignment
from `$bag[$key]` can also mirror covered nested public-property reference
slots from the returned bucket copy, including by-value and by-reference
`offsetGet()` declarations. The same exact `ArrayAccess` bridge can recurse one
bounded level when an intermediate selected offset contains another
`ArrayAccess` object, including through a magic `__get()` array root or stored
`call_user_func_array()` reference argument array. Statement-form reference
assignment to direct and property-held `ArrayAccess` offset targets remains a
runtime boundary; the current runtime does not invent aliases for object array
dimensions.
When a direct alias variable is passed to a by-reference parameter and its
array-offset alias can be materialized as a real `PhpReferenceCell`, the
caller-visible variable is moved onto that cell for the call instead of using a
path-only writeback. This keeps child references attached to the old cell when
another by-reference path replaces the parent array. The same
method-body write helpers can also traverse a tracked object-property array
until a selected concrete bucket contains an `ArrayAccess` object, then pass
the remaining keyed or append path to that object's existing source-aware
`offsetSet()` machinery. The same dispatch now runs for supported local array
values and string-keyed `$GLOBALS` paths before the generic nested array writer
executes, so method bodies can use a local copy of `$this->bags`, a
`$GLOBALS["registry"]["selected"][$key]` write, or a `$GLOBALS["bag"][]`
append when the live path exposes a concrete contained `ArrayAccess` object.
This avoids a separate static body recognizer for those shapes, but it is still
not a general container graph: the runtime needs a concrete array bucket/object
and supported interpreter syntax.
The by-value overloaded-result path uses the same concrete-bucket rule after
emitting PHP's indirect-modification notice. If public by-value
`offsetGet()` or visible `__get()` returns a plain array that contains an
`ArrayAccess` object at an intermediate selected bucket, keyed writes, append
writes, and covered reference-source bindings forward the remaining path to
that inner object while keeping the overloaded parent detached. Missing
instance-method calls through `__call()` now enter a source-aware method path,
so delegated `offsetGet()`/`__get()` returns can preserve copied-array source
metadata instead of returning an untracked plain value.
Statement-form reference assignment from by-value exact-bridge `ArrayAccess`
offset sources follows PHP's indirect-modification notice/no-op behavior for
direct object roots, visible direct property-held roots, bounded non-direct
holder roots that evaluate once to a visible property-held object, and bounded
magic-property roots whose visible `__get()` returns such an `ArrayAccess`
object. Shapes such as `$holders["box"]->bag[$key]`,
`$holders["box"]->{$name}["outer"]["slot"]`,
`$registry->holder()->bag[$key]`, `make_holder($bag)->bag[$key]`,
`$box->missing[$key]`, and `$box->{$name}["outer"]["slot"]` receive only a
detached local value and later writes do not mutate the backing `ArrayAccess`
storage. The same focused by-value notice/no-op bridge covers append sources
such as `$holders["box"]->bag[]`, `$holders["box"]->{$name}[]`,
`$registry->holder()->bag[]`, `make_holder($bag)->bag[]`,
`$box->missing[]`, and `$box->{$name}[]`, using PHP's `offsetGet(null)`
behavior and the exact bridge's empty-string backing key.
Statement-form reference assignment can also bind expression-root
object-property array sources such as `factory()->items["slot"]` and
non-direct magic `__get()` append sources such as
`$holders["box"]->missing[]`.

Nested append assignment reuses the same assignment-value metadata path before
storing the appended value. When the RHS is a proven copied-source array from a
direct by-value `ArrayAccess::offsetGet()`, visible magic `__get()`, or
visible object-property-held `ArrayAccess` source, selected reference-backed
leaves are materialized into runtime reference cells before the value is
wrapped by any suffix append keys and stored into a direct nested array,
string-keyed `$GLOBALS` path, or covered direct `ArrayAccess` nested append
target. Ordinary copied leaves remain plain copied values.
Supported method-body expression assignments to non-direct holder
object-property arrays reuse this write model after the holder expression is
evaluated once into a temporary object root. Static and dynamic property names
and append-with-suffix forms then share the same object-property alias root,
reference-cell binding, copied-source mirroring, and alias-sync helpers as
direct object-property array writes. The path is still bounded to supported
interpreter syntax and tracked object-property roots; it is not arbitrary PHP
side-effect recovery for untracked dynamic containers.
Dynamic calls inside supported method bodies now accept callee expressions
that evaluate to the runtime's supported PHP array callable shape. Those calls
reuse the direct array-callable invocation helper, and the source-aware
dynamic-call variant reuses the matching copied-source helper so setter
payloads and by-value `offsetGet()` returns keep their tracked COW provenance
across property-held or indexed callback variables. This is dispatch
broadening for known callable-array values, not a general side-effect-analysis
model for arbitrary dynamic callback containers.
`ReflectionMethod::invoke()` and `invokeArgs()` follow the same rule for
supported reflected user methods: after validating the reflected target, the
interpreter enters the reflected function through the source-aware expression
or argument-array invocation path. Reflected helper returns and reflected
setter payloads therefore preserve the same copied-source metadata as direct
method calls. Reflection still only covers the runtime's supported reflection
targets and method bodies; it is not arbitrary reflection side-effect recovery.
Stored `call_user_func_array()` argument arrays now use the same stored-root
identity model for source-aware value arguments that reference slots already
used. Direct variables, visible object-property holder roots, dynamic holder
properties, and nested object-property array roots can expose copied-source
entries to callback invocation. If a reference-return callback returns a
selected leaf from a value-copy parameter, the interpreter first looks for a
source reference cell at the copied path and can promote a proven
object-property source leaf into a `PhpReferenceCell`; otherwise it falls back
to the detached local copied value. This is still bounded to concrete
stored-root paths and selected leaves, not arbitrary callback-container graph
analysis.
When the stored argument array itself lives behind a concrete `ArrayAccess`
holder, root recovery may use a statically proven public `offsetGet()` backing
bucket to translate `$holder["args"]` or `$this->holder["args"]` into the
holder object's tracked property path. By-value call setup records copied
sources under parameter paths such as `$value[0]`, whole-array assignments
copy those nested source paths into the destination object property, and
reference-return callbacks promote any mirrored holder path that points back
to the same copied source onto the source reference cell. This keeps direct
holders, visible `__get()` bodies, and public `ArrayAccess::offsetGet()`
bodies in the same COW model without claiming arbitrary `ArrayAccess` method
side effects.
Expression-position argument containers produced by the supported
literal-transform evaluator, currently including
`array_values(array($copy))` and `array_merge(array($copy), ...)`, can also
feed per-entry copied-source metadata into `call_user_func_array()` user
callback setup, including public object array-callable user methods. The same
entry-source mapping is also preserved for covered reference-return string and
closure callbacks and public object array-callable callbacks, where by-value
copied-source bindings are imported into the callee frame before selected
returned leaves are promoted. Covered string-keyed named argument containers
use the same parameter-name mapping before importing those bindings. This path
is still bounded to literal array-transform inputs whose entries can be mapped
to callback positions or parameter names; arbitrary transformed arrays and
untracked dynamic containers remain outside the model.
use the same parameter-name mapping before importing those bindings. For
reference-return string callbacks and public object array-callable callbacks
whose parameters are declared by reference, supported literal-transform
containers without real reference-backed entries take the same source-aware
value-copy path before the stricter reference-slot fallback; the existing
callback reference warning behavior remains, and copied-source bindings are
available for selected return-leaf promotion. This path is still bounded to
literal array-transform inputs whose entries can be mapped to callback
positions or parameter names; arbitrary transformed arrays and untracked
dynamic containers remain outside the model.

The method signature gate has a separate syntax-only path for COW-relevant
magic and `ArrayAccess` dispatch. Ordinary by-value user-function, method,
closure, `call_user_func()`, and `ReflectionFunction::invoke()` calls now run
enforceable parameter and return metadata through the shared call-frame
coercion boundary before local parameters are bound and after return values are
produced. That boundary covers the current weak scalar coercions, arrays,
objects, nullable/union/intersection declarations, and class/interface object
names visible on runtime object metadata. Dispatch through covered
magic/`ArrayAccess` helpers additionally accepts the standard PHP signatures
for `__get`, `__set`, `__isset`, `__unset`, `__call`, `__callStatic`,
`offsetGet`, `offsetSet`, `offsetExists`, and `offsetUnset` as syntax-only
metadata for the existing body executor. Typed by-reference parameters,
`void`/`never`, callable/iterable pseudo-types, `self`/`parent`/`static`,
exact `TypeError` behavior, `strict_types`, and native lowering remain
separate contracts.

Non-direct holder expressions use the same identity model after evaluating the
holder once into a temporary object root. The interpreter snapshots public
array copy-source metadata before the helper evaluation and restores only
unchanged same-cell arrays, so fetching a concrete holder does not erase local
copy provenance that a later reference-return callback needs. Equivalent
object-property roots are canonicalized by object identity, allowing separate
temporary roots for the same holder object to share alias metadata. This is
concrete holder identity preservation, not arbitrary graph or side-effect
reconstruction.

Internal alias writeback uses a similarly narrow preservation rule for copied
static locals. When supported magic/`ArrayAccess` bodies copy a whole
object-property array into `$bucket`, pass that local through a supported
helper, and later return `$bucket[$name]`, nested alias writeback and alias-root
sync can materialize reference cells below `$bucket` without erasing the
local's copied-source root. The retention runs only for nested paths whose
copied parent remains valid, and it reattaches the copied-source metadata
after the internal sync completes; ordinary root overwrites still clear the
metadata.

Array-valued alias groups are promoted only when alias metadata proves a PHP
reference already exists. During by-value helper setup and copied-array
assignment, the interpreter may turn a proven array-valued path alias into a
real `PhpReferenceCell`, overlay that cell into the copied array, and write the
same cell back to the caller alias group. This keeps whole-array referenced
slots connected across helper passthrough and supported reference-return
magic/`ArrayAccess` bodies without making ordinary plain nested array copies
share.

By-value reads from covered reference-return user functions and methods now
carry returned runtime cells as temporary copy-source roots. Before cloning the
returned array value, the caller resolves that cell back to any visible
static/global/object-property alias roots and materializes descendant reference
aliases into real runtime reference cells. This advances the value model away
from pure path provenance for direct method/global-function returns and
supported callback dispatch. Reference-return closure callbacks reached through
`call_user_func()` or positional `call_user_func_array()` use the same
source-aware return path on by-value reads instead of dropping the returned
copy source through the value-only closure helper, but this is still bounded to
cells and roots the
interpreter can observe.
That visible-root resolution now starts through a small
`SymbolTable` runtime lvalue handle for static, global, and
visible/context object-property roots. Runtime-cell copied-source alias
rehydration and reference-cell overlay now consume those handles directly when
matching descendant aliases, and helper/callback copied-source alias
mirror/import setup now resolves `ArrayCopySource` values to those handles
before populating copied locals or callee scopes. Reference-return path
promotion, existing-cell lookup, return-cell rehydration, copied-array alias
mirroring, helper/callee alias writeback, and copied-source mirror-path
promotion also resolve through handles before building the concrete alias path
needed for a specific operation. Copied-source reference-cell scans and
copied-source value reads also prefer live handle values before using the
bounded fallback for roots that still cannot expose a handle. Object-property
invalidation and detached-path rehydration checks use the same handle-based
property identity match for runtime-cell copied sources, and dirty
copied-source detection uses that same runtime lvalue identity when deciding
whether a returned copied array needs reference-cell promotion. Exporting a
static copied source across scopes consults dirty copied-source metadata as
well as the clean public-property source map before deciding that no portable
object-property source remains, and alias-group copied-source recovery uses
the same clean-or-dirty static lookup. Reference-promotion writes to static
copied arrays also preserve dirty-only copied-source metadata while rewriting
the root array, and object-property alias syncs preserve the same dirty-only
metadata while rewriting static alias groups. Reference-binding metadata sync
uses the same clean-or-dirty lookup before removing caller-side copied-source
metadata. Runtime-cell handle discovery includes
metadata, and direct by-reference argument setup uses the same lookup while
carrying the dirty marker into the callee frame. Closure capture source
recovery and prebound closure locals also preserve that dirty marker through
supported closure call frames. The fallback that rebuilds by-value
array-copy-source bindings from reference bindings also consults dirty static
metadata for plain caller-cell bindings. Runtime-cell handle discovery includes
metadata for plain caller-cell bindings. Assignment-expression result source
recovery and direct value-expression source recovery use the same
clean-or-dirty metadata lookup, and nested-write parent-replacement checks
treat dirty-only copied-source metadata as live when deciding whether the
copied-source record survives. The same clean-or-dirty copied-source entry
view is used by unchanged-source snapshots around holder evaluation and by
detached object-property source rehydration, so dirty-only static metadata can
restore after an unchanged helper evaluation and can still receive detached
reference leaves from a backing property replacement. Runtime-cell handle
discovery includes
initialized public object-property cells that share the same reference, so a
visible property root does not need pre-existing array-offset alias metadata
before it can participate in these handle-based copied-source operations.
The object-property overwrite path uses the same runtime-cell/property-cell
identity match before invalidating copied-source metadata or rehydrating
detached copied arrays, so runtime-cell sources reachable through a shared
holder property can preserve selected reference leaves before that property is
replaced. That match is based on initialized property reference-cell identity
rather than public-context lookup, so non-public holder properties can
participate when the runtime cell is already known. Direct object-property
compound assignment routes whole-property and nested property-array
read-modify-write stores through that same rehydrate/invalidate boundary, and
direct dynamic-property whole assignment uses it after the property expression
resolves to the tracked holder property. Dirty copied-source comparisons use
that same reachable-cell bridge
replaced or unset. That match is based on initialized property reference-cell
identity rather than public-context lookup, so non-public holder properties can
participate when the runtime cell is already known. Direct object-property
compound assignment routes whole-property and nested property-array
read-modify-write stores through that same rehydrate/invalidate boundary, and
direct dynamic-property whole assignment and nested object-property array unset
use it after the property expression resolves to the tracked holder property.
Object-property array-offset reference assignment now routes alias writes
through that same keyed boundary, including dynamic-property spellings once
the property expression resolves to the tracked holder property.
read-modify-write stores through that same rehydrate/invalidate boundary.
Dynamic and non-direct object-property compound targets resolve the
holder/property once and then reuse those same writeback places. Direct
dynamic-property whole assignment and nested object-property array unset use
the boundary after the property expression resolves to the tracked holder
property. Object-property array-offset reference assignment now routes alias
writes through that same keyed boundary, including dynamic-property spellings
once the property expression resolves to the tracked holder property.
Dirty copied-source comparisons use that same reachable-cell bridge
when one side is a runtime cell and the other side is the equivalent visible
object-property or alias root, so return/promotion decisions do not depend on
which root spelling recorded the dirty source. Detached copied-source writeback
guards also compare a runtime-cell source's selected leaf with recorded
detached leaf cells, so stale writeback is suppressed even after the visible
property has been rebound away from the old root cell. Broader dynamic holder
writeback and untracked containers without reachable cells remain later steps.

Parent-bucket replacement is a copied-source invalidation boundary for tracked
object-property provenance. Before a supported magic/`ArrayAccess` method body
overwrites a copied source parent such as `$this->store[$name]`, the
interpreter materializes selected reference leaves from the old bucket into
tracked copied locals, stored object-property payloads, and caller aliases.
The replacement bucket then gets fresh identity, and by-value alias writeback
skips source paths that were detached by that overwrite.

Stored-root recovery returns both the recovered concrete root and, when root
recovery already evaluated part of the expression, the argument-array value
read from that root. `call_user_func_array()` then consumes that carried value
instead of evaluating the original argument expression again. This preserves
PHP's single-evaluation ordering for supported helper calls, dynamic property
names, and index-key expressions while still letting the callback path use the
root metadata for copied-source promotion.

Reference-returning helper methods that are invoked for their value use the
same metadata transfer when they are direct visible method calls or supported
array-callable `call_user_func()` dispatches. The caller records copied-array
sources for by-value array arguments, the reference-return helper imports that
metadata into its local parameter, and successful helper execution syncs dirty
object-property copied-source metadata plus stale object aliases back to the
caller. This lets a helper return a shared holder by reference after unsetting
and recreating the holder's stored argument array without losing the source
leaf needed by a later reference-return callback. This is still scoped to
supported helper dispatch and portable copied-source roots, not arbitrary
effect analysis for every possible callback body.

Closure-valued helpers follow the same source-aware path for direct closure
calls, dynamic closure calls from variables or object properties, and
`call_user_func($closure, ...)`. Non-static closures created inside supported
method bodies retain their bound `$this` context, so closure helper writes to a
shared holder object are visible to the caller scope; the copied-source
metadata imported for by-value array arguments is then recorded on recreated
holder argument arrays and synced through the existing dirty object-property
ledger. This is still closure dispatch through the runtime's supported
closure metadata, not arbitrary callback graph recovery.

`ReflectionFunction::invoke()` and `invokeArgs()` first attempt that same
source-aware path for supported non-reference closure and user-function
targets. Direct reflection arguments use expression-argument copied-source
collection; `invokeArgs()` reuses the stored/literal argument-array
copied-source path used by `call_user_func_array()`. If the reflected target is
outside that bounded subset, reflection falls back to the existing value-only
invocation path and does not claim COW provenance preservation.

Direct caller-cell by-reference parameters now carry array-literal
copied-source path metadata alongside the shared value cell. When a supported
helper rebuilds a method-local `call_user_func_array()` argument container
through `&$args`, portable copied-source paths are imported into the helper
scope and dirty paths are synced back to the caller after normal return. This
lets local argument containers participate in the same source-leaf promotion
model as shared object-property holders without leaking callee-local symbol
names. Callback expressions that evaluate to supported array callables still
reuse the existing callable dispatch path; the new part is preserving the
argument-container provenance when the callback-producing helper mutates the
container first.

Object-property by-reference parameters use the same copied-source path
writeback when the target is a concrete object property with an imported
source. If a helper receives `&$this->args` and assigns a new array literal
containing a copied bucket, dirty portable paths from the helper frame are
recorded on that object property, so later stored-root callback dispatch can
recover the copied source without treating ordinary whole-array value copies
as source aliases.

Reference-return magic `__get()` uses static backing-root recognition only
when it can prove the root exactly. Unsupported static proofs fall through to
the executed reference-return body, which can bind real cells for supported
computed keys, repeated selector keys, appends, and constant backing keys
without leaking a stale static path. `ArrayAccess::offsetSet()` backing-target
recovery similarly tracks simple local aliases of the value parameter before
matching the final object-property store.

Direct array literals returned from supported function bodies use the same
reference and copied-source materialization path as array-literal assignments.
When a helper returns `array($copy)` or a nested literal built from a copied
bucket, the return boundary rehydrates reference-backed leaves into that
literal before the value leaves the callee. This keeps caller-side indexing of
direct helper and reflected user-function results aligned with PHP COW for
covered copied-source roots without introducing arbitrary callee-local path
names into the caller.

`call_user_func_array()` now has a value-copy fallback for supported general
expression argument arrays in the reference-return path. If an expression such
as a helper call returns an argument array, the interpreter uses the evaluated
array value directly; reference cells already materialized in that value are
available to the callback, and copied-source entry metadata is attached when
the argument-array expression itself has a portable source. Stored argument
arrays also check the reached slot for a concrete runtime reference cell before
requiring a portable alias group, which lets fresh helper-returned holder
objects participate without leaking local variable names. Dynamic instance
method calls evaluate the method expression and then reuse ordinary
source-aware instance dispatch.

By-reference function parameter binding uses the narrower offset-promotion
rules for ordinary nested offset aliases, but switches to the array-literal
promotion rules for covered array-offset aliases. That gives supported
magic/`ArrayAccess` method bodies the same whole-bucket reference-cell behavior
as an array literal containing `&$bucket`: a backing slot such as
`$this->store[$name]` can be passed to a helper, returned inside
`array(&$bucket)`, and mutated through the returned argument array without
falling back to a stale method-local writeback path. Because this only applies
when the alias group exposes a concrete runtime cell, copied locals placed into
`call_user_func_array()` arguments still remain detached from their original
backing store.

Source-aware by-value returns also rehydrate copied object-property reference
leaves at the callee boundary, before the local alias proof disappears. The
return path overlays existing reference-backed source slots when present and
uses array-literal promotion for proven object-property copy-source aliases.
Alias synchronization now carries reference cells between sibling aliases, so
updating a copied bucket's reference leaf does not flatten that leaf back to a
plain scalar before the returned value is assigned by the caller.

Explicit local references to copied bucket leaves use the same terminal-cell
rule. When a supported method body binds `$alias =& $bucket["ref"]["value"]`
and the alias group proves both the copied local path and the source
object-property path for that scalar leaf, the interpreter promotes all
terminal aliases in that group to one reference cell. That prevents by-value
helper writeback from reading a stale copied leaf and overwriting a source
reference mutation, while still refusing to promote parent arrays or missing
untracked containers.

Fresh helper-returned `stdClass` holder objects are now concrete containers for
this model. Named writes on allowed dynamic-property objects materialize public
dynamic slots through the same runtime policy as dynamic-property-name writes,
so helpers can return `$holder->args` arrays that carry copied-bucket metadata
into `call_user_func_array()` from supported magic and `ArrayAccess` bodies.
Object-property argument containers rebuilt through by-reference helper
parameters follow the same concrete-container rule once the target property is
known, with copied-source paths synced back to the caller property on normal
return.

Direct free-function calls
declared as returning by reference can also serve as by-reference `foreach`
iterable roots when the function returns a direct variable backed by a caller
variable cell, such as a by-reference parameter; the interpreter binds a
private temporary root to that returned cell before applying the existing
foreach array-slot alias machinery. This is still a
materialized-symbol-table model, not PHP's full reference-backed alias,
recursive `$GLOBALS` array, copy-on-write, dynamic global-name, broader
ArrayAccess iteration, `Iterator`/`IteratorAggregate`/`Traversable` object
by-reference execution, non-public or magic-property iterator bucket
reference-slot preservation, non-public or magic-property object iteration,
arbitrary dynamic object-property iterable roots, or included-file scope model.
`$_COOKIE`, `$_GET`, `$_POST`, `$_REQUEST`, and `$_FILES` are seeded in the
same root symbol table and route direct function-scope reads and writes through
that root storage. `$_SESSION` is materialized lazily into the same root symbol
table when the bounded CLI `session_start()` path succeeds and then routes
direct function-scope reads and writes through that same root storage,
including the current bounded direct nested array-offset alias metadata for
covered `$_SESSION["..."]` reference slots. The default request bags are empty
ordered arrays. For
explicit CLI request exercises, `PHPC_QUERY_STRING` seeds flat URL-encoded
and bracketed URL-encoded query pairs into `$_GET`, and
`PHPC_REQUEST_METHOD=POST` plus
`PHPC_CONTENT_TYPE=application/x-www-form-urlencoded` and `PHPC_REQUEST_BODY`
seeds flat and bracketed URL-encoded body pairs into `$_POST`. `PHPC_COOKIE`
is treated as an explicit semicolon-delimited cookie header seed for
`$_COOKIE` and `$_SERVER["HTTP_COOKIE"]`. `PHPC_FILES` is treated as an
explicit URL-encoded upload metadata seed for `$_FILES` using PHP-shaped keys
such as `async-upload[name]`, `async-upload[tmp_name]`,
`async-upload[error]`, and `async-upload[size]`; `error` and `size` leaf
values parse as decimal integers when possible. The same initial seed records
bounded upload provenance for `tmp_name` entries whose sibling `error` value
is `0`; `is_uploaded_file()` checks that request-local provenance and
`move_uploaded_file()` moves a registered local path once before clearing the
source provenance. Repeated scalar keys use
last-write-wins, `[]` bracket segments append in order, and keyed bracket
segments materialize nested arrays under the current ordered-array model. Dots
and spaces in top-level request names are normalized to underscores before
insertion, while bracket segment keys keep their current literal decoded text.
`$_REQUEST` starts as GET merged with POST at the top-level key. This is
deterministic request-state scaffolding only: it does not import browser
cookie headers from the host SAPI, merge cookies into `$_REQUEST`, handle
malformed or edge-case PHP request names with exact `parse_str()` behavior,
parse multipart uploads, create temporary upload files, validate host upload
state beyond the explicit seed, model failed-upload provenance, lock session
files, dispatch session save handlers, configure session cache limiters, or
model cache-header variants beyond the default no-cache trio or
`variables_order`/`request_order`. The only cross-process session persistence
path is the bounded `session.save_path`/`session_id()` file slice documented
below.

Array-offset references in the interpreter are currently represented as
symbol-table alias metadata, not general runtime reference containers. A direct
variable may route to one or more direct array-offset aliases; this lets the
current runtime mirror the bounded PHP behavior where copying a direct array
or visible object-property array that contains a referenced direct slot
preserves that slot's reference identity across a copied direct array. Copying
a literal-key direct nested array path, including auto-global request paths
such as `$_REQUEST["payload"]`, visible named object-property paths, and
visible direct dynamic object-property paths such as `$object->{$name}` and
`$object->{$name}["child"]`, also mirrors covered reference slots below the
copied path into the destination direct array variable. By-reference `foreach`
over that copied direct array consults the mirrored element alias group before
falling back to a plain copied slot, so loop writes and the post-loop lingering
reference update the original referenced slot for covered direct, request-bag,
and visible object-property copied-path aliases while non-reference copied
elements remain ordinary copied values. Variable, append, and side-effecting
copied path keys remain outside this bounded mirror and still require the
future reference/COW value model. When an
array-offset or visible named object-property array-offset reference target is
bound from a direct variable that already shares a direct variable-to-variable
cell, the interpreter rewires every direct name in that small source cell
group to the selected slot alias. By-reference user-function, instance-method,
named static method, `self::` static method, and late-bound `static::` static
method parameters can also accept a direct visible named object-property
array-offset argument through a narrower output-parameter bridge: the caller
slot is materialized and copied into the callee parameter, then the final
parameter value is written back to the visible property slot after normal
return. Public slots use public alias metadata; private/protected slots use
context-aware alias metadata when reached from a valid method visibility
context. Ordinary direct user-function calls that pass multiple by-reference
arguments which resolve to the same covered alias metadata, such as a direct
alias variable and its direct array slot or visible object-property array slot,
bind those callee parameters to one local cell for the duration of the call
before the usual bounded writeback. Direct user-function calls also accept non-direct holder plain
object-property array-offset arguments, such as
`handler($holders["bag"]->items["outer"]["slot"])`, when evaluating the
holder once yields an object whose selected property is visible in the current
context. The path uses the existing private temporary object root and writes
back through the property array alias metadata. The same non-direct holder
path continues to cover property-held `ArrayAccess` offset arguments, such as
`handler($holders["bag"]->store["outer"]["slot"])`, when the selected visible
property holds an `ArrayAccess` object using the exact bounded by-reference
`offsetGet($offset) { return $this->property[$offset]; }` bridge. These
bridges intentionally do not provide broad in-call reference-container
identity beyond covered same-alias ordinary direct user-function arguments and
do not cover mixed nested `ArrayAccess` chains, magic property references, or
arbitrary reference expressions.
Direct object-variable clone assignments also mirror public object-property
alias metadata and context-aware non-public
object-property alias metadata from the cloned source variable to the target
variable, so the current bounded reference-slot model can keep covered property
slots shared across a fresh object handle for the covered `clone $object`
assignment shape.
Prepared MySQLi result bindings use a separate deterministic statement-target
list rather than PHP reference containers: direct variables write to the caller
symbol table, and direct variable or direct object-property array-offset
bindings store their evaluated key path at bind time before
`mysqli_stmt_fetch()` copies deterministic placeholder row values into those
slots. The bounded fetch path can consume the executed statement result
directly without `mysqli_stmt_store_result()`, while buffered metadata such as
`mysqli_stmt_num_rows()` and `mysqli_stmt_data_seek()` stays tied to the
explicit store path.
Direct object-property bindings write through the same visible property path
used by ordinary property assignment, with dynamic public-property creation
limited to the current `stdClass`/`wpdb` object slice. Dynamic property target
expressions, real mysqlnd unbuffered network transfer, and arbitrary host
database rows remain outside this binding model.
Direct variable `unset($name)` removes the root symbol and, for covered direct
array roots plus direct object roots with public/context property array-slot
aliases, detaches aliases below that removed root by storing their last
observed values into the remaining direct alias variables. This is still the
same symbol-table alias metadata, not a reference container with destructor or
copy-on-write ordering semantics.
By-reference user-function parameters can also bind direct missing or
inaccessible declared named and dynamic object-property arguments through the
existing magic `__get()` reference-return cell path when visible public
`__get($name)` returns a direct variable by reference. The argument binding
shares that returned cell with the callee parameter for the duration of the
call. Direct user-function by-reference parameters can also bind array offsets
below that magic property, such as `$object->missing["slot"]` or
`$object->private["slot"]`, by temporarily rooting the returned direct-variable
cell and reusing the existing array-offset copy-in/writeback alias path. If
that returned direct-variable cell currently holds an `ArrayAccess` object,
the same temporary root may instead use the bounded public by-reference
`offsetGet($offset) { return $this->property[$offset]; }` bridge, now with the
same bounded literal int/string prefix or suffix key analysis around one or
more uses of the offset parameter, and write through the selected backing
property array slot. Named and dynamic
array-offset paths below magic properties on non-direct holders, such as
`$holders["box"]->missing["slot"]`, evaluate the holder expression once into a
temporary object root before reusing that same bridge for direct user-function
by-reference parameters and direct-variable reference assignment. This is
intentionally limited to the existing direct-variable reference-return body
shape, selected array offsets, direct user-function or direct-variable
reference-assignment paths, and the exact bounded ArrayAccess bridge. General
magic container identity, normal property-read magic fallback breadth,
arbitrary `__get()` return expressions, dynamic `offsetGet()` parameter keys,
non-literal `offsetGet()` path keys, side-effecting or broader
`ArrayAccess::offsetGet()` bodies, mixed nested `ArrayAccess` object chains,
and copy-on-write semantics still belong to the future reference-container
model.
Reference-returning `call_user_func_array()` sources use the same caller-cell
binding path as direct reference-returning function and method calls for the
current literal argument-array direct-variable and direct array-slot reference
element slice. Direct reference-returning free-function, visible
object-method, named-static-method, method-context `self::`/`parent::`/
`static::`, and dynamic-static-receiver assignment now also use the
array-offset writeback/alias result path when a reached by-reference parameter
is supplied by a direct array-offset or visible named object-property
array-offset argument and the function or method returns that parameter
directly. The same path can bind a returned direct array-offset expression such
as `return $param[$key];` or `return $param[$key][$subkey];` for
statement-form reference assignment, and can read that returned child slot by
value for normal reference-return invocation, when `$param` is one of those
copied-in covered parent array-slot arguments or a direct caller variable
bound through a by-reference parameter. For copied-in slots, after the local
parent array is written back, the returned key suffix is appended to the
caller alias group. For direct caller variables, the returned explicit key
suffix is mapped to that caller variable's child slot. If the direct caller
variable is itself backed by the bounded array-offset alias metadata, the
callee parameter now receives the alias group instead of looking for a normal
symbol-table cell, so returned child-slot suffixes remain attached to the
underlying request/global/array/property slot. Literal callback argument
arrays can also bind array-offset reference elements below direct named or
dynamic magic `__get()` properties when visible public `__get($name)` returns
a direct variable by reference; the interpreter temporarily roots that returned
cell and reuses the same copy-in/writeback alias path for normal
`call_user_func_array()` invocation and covered reference-return callback
sources. Literal callback argument arrays can also bind direct, named
property-held, and direct dynamic property-held `ArrayAccess` elements,
including method-context `$this->{$name}` roots for visible private or
protected holder properties and nested offset elements, when public by-reference
`offsetGet($offset)` has the exact bounded `return $this->property[$offset];`
shape; the interpreter maps that to the backing property array alias root plus
any nested child-key suffix instead of creating a real reference container.
Append-offset `ArrayAccess` reference sources such as `$bag[]`,
`$holder->bag[]`, and non-direct visible property-held forms such as
`$holders["box"]->bag[]` use the same bridge and model PHP's
`offsetGet(null)` call as the backing property array's empty-string key for
that exact body shape.
Statement-form direct-variable reference assignment from append offsets below
reference-returning magic `__get()` properties, such as
`$alias =& $object->missing[]` and `$alias =& $object->{$name}[]`, temporarily
roots the returned direct-variable cell and appends through the same bounded
array-offset alias metadata used for direct array append references. When that
visible `__get()` returns an `ArrayAccess` object with the exact by-value
`offsetGet($offset) { return $this->property[$offset]; }` bridge, selected
offset and append roots follow the bounded PHP notice/no-op path instead of
creating a backing alias. It does not make stored callback argument arrays
from magic `__get()` roots, non-public object-property array bridges, dynamic
ArrayAccess roots beyond the documented visible property-held and
magic-property sources, mixed nested `ArrayAccess` chains, general
magic-property reference containers, arbitrary append ArrayAccess bodies, or
stored array-offset metadata into general runtime reference containers.
By-reference
`foreach` currently consumes direct non-`Traversable` object variables for
initialized public-property by-reference iteration, direct visible named and
dynamic object-property array roots, bounded non-direct named and dynamic
property holder expressions that
evaluate to objects, direct free-function, direct visible
instance-method, direct named-static-method, method-context
`self::`/`parent::`/`static::`, dynamic static receiver, and bounded
`call_user_func_array()` reference-return iterable roots from this machinery,
plus direct, visible property-held, and bounded non-direct holder
property-held `ArrayAccess` offset-array roots backed by the exact bounded
by-reference `offsetGet()` bridge, including bounded direct caller-cell and
direct static-local cell cases. Non-direct holder `ArrayAccess` foreach roots
evaluate the holder expression once into the existing private temporary object
root, then reuse the same visible property-held `ArrayAccess` bridge before
falling back to ordinary property-array iteration. When
the reference-return call maps a returned child array such as
`return $param[$key];` to multiple covered aliases for direct names sharing a
caller cell, by-reference `foreach` binds each visited element through all of
those aliases so loop writes and the lingering post-loop reference keep the
current bounded alias group coherent.
Non-direct property holders outside that object-result foreach slice,
non-direct holder `ArrayAccess` roots outside the exact visible property-held
bridge, invisible selected properties, mixed nested `ArrayAccess` chains,
`Iterator`/`IteratorAggregate`/`Traversable` objects, non-public or
magic-property object iteration, property-return, array-offset-return beyond that
assignment-only covered parent-slot suffix shape, expression-return, magic
`__callStatic`, and callback forms outside the bounded
`call_user_func_array()` slice remain outside the executable foreach slice.
Whole-variable assignment to a direct array root and whole-property assignment
to a declared visible object-property root drop stale aliases for that root
before the replacement value is observed by future copies. Reassigning the
direct object variable also drops stale public and context-aware non-public
object-property roots for that object name. Direct dynamic-property assignment
from a reference array literal uses the evaluated property name as the public
or context-aware non-public object-property alias root, so later stored-array
callback use can reuse covered reference elements when the current method
context can see the property. Direct array-offset and direct visible
object-property array-offset assignment detach covered child aliases below the
explicit assigned path before overwriting that slot. Reference assignment that
rebinds an already covered direct array slot or public object-property array
slot detaches the old direct alias variables with their last observed value
before the slot joins the new direct-variable or storable slot source. The same
detach-before-join ordering is covered for bounded non-direct object holder
expressions such as `$holders["bag"]->items["slot"] =& $new` and
`$holders["bag"]->{$property}["slot"] =& $new` when the holder evaluates once
to an object whose selected visible property array slot is already represented
by the current object-property alias metadata. When the
assigned value is a direct array variable or covered literal-key array/property
path whose elements already have bounded reference alias metadata, the
assignment mirrors those child reference slots into the target array/property
slot so stored argument arrays remain usable by the current
`call_user_func_array()` reference path. This is still path metadata for
explicit keys, not PHP's general COW container graph; dynamic, magic,
side-effecting, and mixed `ArrayAccess` source or target paths remain outside
this mirror, except for bounded direct, visible property-held, and non-direct
holder `ArrayAccess` bucket-copy sources whose public `offsetGet($offset)`
body is exactly `return $this->property[$offset];`. Direct array-offset `unset(...)` paths, direct
visible object-property array-offset `unset(...)` paths, and direct visible
object-property `unset(...)` paths remove covered alias metadata for the
removed slot/property, and for child aliases below a removed parent slot or
property, while storing the last observed alias value back into the detached
direct alias variables. When multiple direct names shared the same covered
array/property-slot alias metadata before detachment, those names are rebound
to one shared cell so later writes through one detached name remain visible
through the other detached names. This models the bounded PHP behavior where
unsetting a referenced container slot or containing property deletes the
container entry without deleting the remaining reference variables. Direct variable
`unset($name)` also detaches covered aliases rooted below the removed direct
array/object variable.
Arbitrary nested copied reference slots beyond the literal copied path slice,
non-direct dynamic-property holders, non-public property-offset or magic clone
alias mirroring, dynamic ArrayAccess references, arbitrary ArrayAccess append
bodies, reference array literals outside direct variable, direct array-offset,
direct visible object-property, and direct dynamic-property assignment targets,
plain object-property `unset(...)` alias cleanup outside visible properties,
destructor side effects during alias destruction, arbitrary expression-root
rebind ordering outside the bounded visible non-direct object-holder slot
slice, and native lowering still require the future runtime reference/COW value
model.

## Compiler Crate

`compiler/` contains:

- lexer and parser for the supported PHP subset
- AST definitions
- interpreter bridge for `phpc run`
- LLVM IR text emission for currently lowerable code
- CLI and fixture test runner

The `phpc compile` CLI validates its requested output mode before reading or
parsing the input file. That keeps usage mistakes such as `--emit-object`
reported as CLI diagnostics even when the input path is missing or the source
would otherwise fail earlier in the compiler pipeline.

The parser is handwritten recursive descent. This keeps the early grammar easy
to audit while avoiding regex-based parsing. Unsupported syntax boundaries use
stable diagnostics before AST construction when accepting the syntax would
imply runtime or native semantics that do not exist yet. Statement-form
`list($a, $b) = expr;` now has an AST/runtime path for direct variable targets
and skipped positional slots. Statement-form short `[$a, $b] = expr;`
destructuring reuses that same AST/runtime path for direct variable targets,
while expression-position `list(...)`, keyed/nested/reference targets,
`foreach` destructuring, and non-variable targets remain parser boundaries.
Named call arguments stop at a dedicated parse boundary before AST
construction because correct support needs parameter-name metadata, duplicate
and unknown-name diagnostics, positional/named ordering, by-reference binding,
variadic collection, unpacking interaction, and native lowering.

The lexer maintains both character and byte offsets as it advances. Prefix
checks for PHP tags, heredoc terminators, and other byte-slice comparisons use
the maintained byte offset so large real-world files do not repeatedly rescan
the already-consumed character prefix.
When the lexer sees `?>`, it emits the intervening inline HTML up to the next
PHP open tag as a token consumed by the parser into an echo statement. The
current slice consumes one immediate newline after `?>`, matching PHP's common
close-tag newline behavior for the covered fixtures. Short echo open tags
such as `<?= $value ?>` stop in lexing with a dedicated unsupported diagnostic
until the lexer/parser can expand them into echo statements with correct source
mapping, inline-HTML interaction, and native rejection behavior.
Backtick shell execution expressions such as `` `whoami` `` also stop in
lexing with a dedicated unsupported diagnostic until command-string
interpolation, process execution, stdout capture, platform error behavior,
references/copy-on-write, and native lowering have an explicit implementation.

Assignment targets are intentionally narrower than expression reads. Direct
variables, direct array offsets, direct append offsets, direct object
properties, selected static properties, direct-variable nested array offset
paths, direct-variable append-at-depth paths, and direct-object-property
nested/append-at-depth array paths have explicit AST targets. The nested array
targets keep a variable or object-property root plus evaluated index
expressions so the interpreter can materialize missing array containers under
the current no-reference/no-copy-on-write model. Direct, dynamic, and
non-direct object-property array-offset compound assignment reuse the
object-property root plus evaluated index path for the current
read-modify-write slice after dynamic holders/properties are evaluated once.
Direct object-property
array-offset `isset(...)` reuses the same visible property plus evaluated index
path for the current presence-check slice. A visible direct object property
whose value is an `ArrayAccess` object can dispatch single-key read/write,
`isset`, `empty`, `??`, and `unset` operations through that contained object;
deeper mixed object/property/ArrayAccess chains and other nested
read-modify-write forms remain explicit boundaries.
Unset targets follow the same conservative pattern: direct variables,
direct/nested array offsets, selected static-property diagnostics, and
direct-object-property nested array offsets have explicit targets, while plain
object-property uninitialization and mixed target paths remain separate
boundaries.
PHP 8 nullsafe object access `?->` is tokenized so the parser can stop with a
specific unsupported syntax diagnostic before creating any AST node. Faithful
support still needs null-aware property/method chain evaluation,
short-circuiting, call argument ordering, assignment-target restrictions, and
native lowering.
PHP 8 `match` expressions remain a parser boundary rather than an AST node
until the expression model has strict arm matching, default/exhaustiveness
handling, throw-arm behavior, value evaluation ordering, reference/COW
interactions, and native lowering.
Parenthesized DNF-shaped type declarations such as `(A&B)|C` are also kept at
a parse boundary for parameters, return types, and typed properties until the
type metadata model can represent those shapes without implying runtime
enforcement.
Top-level trait declarations are parsed as metadata for empty traits,
supported properties, simple public instance methods, and simple public static
methods. A class body may use already-declared traits
with `use TraitName;`, repeated simple trait-use declarations, or one simple
comma-separated declaration such as `use TraitA, TraitB;`; the interpreter
composes those trait properties plus public instance/static methods onto the consuming class metadata
and stores the executable method bodies under the consuming class id, so
ordinary instance method dispatch works through `phpc run`. A narrow trait
body slice also accepts simple `use TraitName;` and `use TraitA, TraitB;`
declarations inside traits; classes consuming the outer trait receive the
nested traits' supported properties, public methods, and constants, while direct
class-trait metadata remains non-recursive. A narrow trait
method alias adaptation such as `use TraitName { method as alias; }` clones
the composed public instance or static method under the alias name while leaving the
original method available. The same alias path accepts an explicit `public`
marker with a same-use qualified trait target, such as
`use TraitA, TraitB { TraitA::method as public alias; }`, while still treating
the alias as an ordinary public method. Alias adaptations may also mark the new
alias `protected` or `private`; the original public trait method remains
available, and the non-public alias uses the existing method visibility checks
for dispatch, `method_exists()`, and `get_class_methods()`. Visibility-only
adaptations such as `use TraitName { method as protected; }` and
`use TraitName { method as private; }` change the original composed public
instance/static trait method visibility without creating an alias. A bounded trait
conflict adaptation such as
`use TraitA, TraitB { TraitA::method insteadof TraitB; }` is accepted for
public instance/static methods from traits in the same class-body `use`
declaration; composition registers the winner and skips the named loser method.
The same current slice accepts comma-separated loser lists such as
`TraitA::method insteadof TraitB, TraitC` when every loser trait appears in
the same class-body `use` declaration.
The current executable interaction slice also allows that selected winning
method to be exposed through a same-block explicit-public alias, such as
`use TraitA, TraitB { TraitA::method insteadof TraitB; TraitA::method as public alias; }`.
When a consuming class declares a public instance/static method with the same name as
a composed public trait method or alias, the class method takes precedence and
the trait method is skipped in the effective class method table. This lets a
concrete class override trait fallback methods while still satisfying current
interface method checks. When two different composed traits still provide the
same public instance method after class-method precedence and bounded
`insteadof` exclusions are applied, class registration stops with a stable
trait-conflict diagnostic instead of falling through to generic duplicate
method metadata. Trait-body `use` declarations inside traits reuse that same
bounded method-adaptation machinery for supported public instance/static methods, so
an outer trait can adapt nested trait aliases, visibility, and qualified
`insteadof` conflict winners before a class consumes the outer trait.
Public trait constants declared as `const NAME = ...` or
`public const NAME = ...` with the current class-constant expression subset are
composed into consuming classes and resolve through the existing
`ClassName::CONST`, `self::CONST`, and `static::CONST` paths. Supported trait
properties reuse the current class-property metadata/default subset and are
composed as consuming-class properties for object storage and reflection;
identical duplicate definitions are deduped, while incompatible duplicate
definitions stop with a stable trait-use diagnostic. Non-public/typed/abstract/final/static trait constants,
multi-constant trait declarations, trait constant adaptations, conflicting
trait/class constants, abstract/final or non-public trait methods,
broad executable conflict resolution beyond class-method precedence and the
current bounded `insteadof` slice, exact PHP fatal-error text for unresolved
trait conflicts, unqualified visibility-only adaptations across
multiple used traits, unqualified `insteadof`, trait property or constant
adaptations,
qualified or multi-trait alias edge cases beyond the current winner-alias slice,
`__TRAIT__` context, references/copy-on-write, nested or conditional trait
declarations, and native trait lowering remain explicit boundaries.

Top-level interface declarations are parsed as metadata for public method
signatures and public constants. The current inheritance slice accepts one or
more user parent interfaces declared before or after the child interface, such
as `interface Child extends Parent, OtherParent`, and flattens those parent
names into concrete class `implements` relationship metadata. Class registration
enforces public method presence for child and parent interface methods,
including methods supplied by the current public trait composition and alias
subset. Interface registration also checks child-interface redeclarations of
inherited methods and simple multi-parent method conflicts with the current
bounded metadata rules: a redeclaration may not require more parameters than
the inherited method, may not add a parameter type where the parent method is
untyped, must keep compatible parameter types when both sides are typed, and
must keep compatible typed parent return declarations. Compatible typed
relationships include exact text case-insensitively plus simple declared
class/interface contravariant parameter and covariant return relationships when
both type names resolve through current metadata. Public interface constants declared
as `const NAME = ...` or
`public const NAME = ...` use the current class-constant expression subset and
resolve through interface names, parent-interface inheritance, and
implementing-class class-constant lookup. Missing or cyclic parent interface
inheritance remains a stable runtime boundary.
typed/static/non-public/abstract/final or multi-constant interface
declarations, full variance/signature enforcement, namespace-aware type-name
resolution, union/intersection canonicalization, built-in/internal interface inheritance catalogs, exact PHP
diagnostics, and native lowering remain explicit boundaries.

Double-quoted string interpolation is represented explicitly in the AST for
the current simple `$name`, `{$name}`, array-offset, object-property, and
chained access slices instead of being rewritten to ordinary string
concatenation. The runtime evaluates those parts left to right through the
active symbol table and PHP-shaped echo-string conversion. Native lowering
rejects ordinary interpolated strings with a dedicated codegen diagnostic until
native interpolation part evaluation, PHP-shaped string conversion,
array/object lookup, `__toString` dispatch, runtime string allocation,
references/copy-on-write, and exact native diagnostics exist. Interpolated
`defined("SODIUM_$constant")` names continue to use the global-constant native
boundary until native constant tables and runtime string lookup semantics
exist. `${...}`, dynamic properties, static properties, and arbitrary complex
expressions remain lexer boundaries.

PHP error-control syntax is represented as an explicit AST wrapper for
`@expr`. The interpreter currently evaluates the wrapped expression normally
and deliberately does not suppress diagnostics, because the runtime still
models undefined variables, unsupported calls, invalid arithmetic, and other
conditions as fatal project diagnostics rather than recoverable PHP
warnings/notices. Native lowering rejects the wrapper until generated code has
a real diagnostic severity and suppression model.

Loop-control statements record an explicit positive integer depth in the AST.
The interpreter represents `break N;` and `continue N;` as control-flow signals
whose depth is consumed one active `while`, `for`, `do ... while`, `foreach`,
or `switch` level at a time. Plain `continue;` that targets a switch remains a
runtime boundary, while `continue 2;` can pass through the switch to an outer
loop. The generated-native C-link path has a bounded `if`/`else` statement
emitter for conditions that already lower through native truthiness or value
comparison boundaries; it emits branch bodies in scoped generators and accepts
them when persistent variable/cleanup state is unchanged after each branch. It
can also join cleanup-free scalar, string, and boolean variable states by
recording post-branch values as conditional C expressions when both branches
expose the same variable set and neither branch creates owned native cleanup
state. Native lowering still rejects one-sided undefined-variable unions,
mixed-type/native-value phis, cleanup stack joins, `elseif`, loops,
switch/goto/break/continue, and LLVM IR/assembly statement control flow.
The same generated-C path now lowers dynamic logical `&&`/`and` and `||`/`or`
through scoped RHS branches when both the left operand and selected RHS can use
the native truthiness boundary and the RHS leaves persistent compiler state
unchanged; skipped RHS operands are not evaluated. `xor` remains eager, and RHS
state merges, cleanup-owner joins, exact diagnostic ordering, and LLVM/assembly
parity remain separate contracts.
`for` headers store initializer, condition, and increment slots as ordered
lists. The interpreter executes initializer and increment actions left to
right; condition expressions also evaluate left to right and the final
expression's truthiness controls the loop.

Direct `exit()`/`die()` calls are treated as a bounded language-construct
termination signal rather than as ordinary callable builtins. The interpreter
preserves stdout accumulated so far and returns the current exit code for the
top-level CLI result. The construct is intentionally absent from
`function_exists()`/`is_callable()` lookup. Shutdown functions, destructors,
finally-during-exit ordering, output buffers, SAPI interaction, and native
termination lowering remain future runtime work. Native `--emit-ir` and
`--emit-asm` reject `exit()`/`die()` through a dedicated termination diagnostic
instead of the generic function-call boundary.

Cast expressions are represented as `Expr::Cast` with a small `CastKind`
covering `(string)`, `(int)/(integer)`, `(bool)/(boolean)`, and
`(float)/(double)`, and `(array)`. The interpreter owns PHP-shaped conversion
for the current scalar/null/array subset and keeps warning-producing or
object/resource-heavy cast behavior as runtime or parse boundaries. Native
lowering rejects all cast expressions with a dedicated diagnostic until
generated code has scalar conversion, array materialization, warning/recovery,
object/resource handling, and exact diagnostic behavior.

## Runtime Crate

`runtime/` contains the PHP-shaped boxed value model used by the interpreter and
future generated code helper calls.

Implemented now:

- `Null`
- `Bool`
- `Int`
- `Float`
- `String`
- ordered PHP arrays with integer/string keys, including a bounded numeric
  key-sort helper for the reached `ksort(..., SORT_NUMERIC)` path and ordered
  strict identity comparison over the current key/value model
- class metadata, object-shape descriptors, and minimal object values for
  `new ClassName(...)` over declared classes, process-local object handles,
  public instance properties including inherited public slots, inherited
  non-public instance slots with declaring-class ownership, single-parent
  metadata, execution-time registration for reached nested class declarations,
  metadata-only core `Exception`, `stdClass`, `PDO`, and `PDOStatement` class
  seeds,
  inherited method lookup, public/same-class private/protected same-class and
  child instance method dispatch, and public/inherited public instance
  `__construct` plus explicit parent/self method dispatch with scoped `$this`,
  parsed abstract/final class modifiers plus abstract/final method modifiers
  as metadata, with abstract class instantiation, final-parent inheritance,
  final method overrides, method visibility reductions, concrete classes with
  unimplemented abstract methods rejected at runtime, and inherited method
  static/non-static plus bounded non-constructor signature compatibility
  enforced at runtime, including required-parameter counts, parameter type
  metadata, and return type metadata,
  bounded `new self`/`new parent`/`new static` class-name resolution in active
  class contexts, and bounded direct-variable dynamic class-name instantiation
  for `new $class(...)`; missing named or direct-variable string class names
  invoke the current bounded autoload callbacks before the class table is
  rechecked; included class/interface declarations also use that callback path
  to load missing `extends` parents, direct `implements` interfaces, direct
  class-body trait uses, and parent interfaces before final registration
  validation,
  bounded named and dynamic property-name reads/writes for existing public
  slots, `stdClass` public dynamic slots, and the WordPress `wpdb`
  compatibility class's dynamic table-name slots, and bounded `clone`
  expressions that
  allocate fresh handles, shallow-copy current property slots, and dispatch
  visible non-static `__clone()` methods on cloned objects
- structured runtime error categories with stable diagnostic messages for the
  currently supported runtime failures
- PHP-ish echo conversion
- PHP-ish truthiness for the implemented value types
- basic arithmetic, comparison, and concatenation helpers
- key normalization for array strings that are valid decimal integers
- private `ArrayEntry` slot storage with value accessors (`value()`,
  `value_cloned()`, `value_mut()`, `set_value()`, and `into_value()`) that
  delegate through an explicit copy-on-write `ArraySlot`. Plain value-backed
  `ArraySlot` clones share their private `ArraySlotCell` handle until
  `value_mut()` or `set_value()` detaches the written slot; reference-backed
  slots keep sharing their `PhpReferenceCell`. Each cell has a stable internal
  `ArraySlotCellId`, but slot/cell equality remains value-based so PHP array
  equality does not depend on storage identity. This gives ordinary
  interpreter array copies real slot-level COW storage while the higher-level
  magic/`ArrayAccess` paths still use provenance ledgers for source recovery;
  `PhpArray::get_slot()` and `get_slot_mut()` expose normalized-key slot
  lookup without introducing aliasing. `PhpArray` also owns the nested path
  operations used by interpreter array-offset lvalues: path materialization,
  cloned path reads, reference-cell path reads, existing path value/checked
  writes, reference-cell writes, and value/reference appends all recurse
  through `ArraySlot` so COW detach and reference-cell sharing stay in the
  runtime instead of being duplicated in the compiler symbol table

Planned runtime values and semantics:

- resources
- references
- copy-on-write containers

By-reference assignment has bounded execution slices for statement-form direct
variable sources. The interpreter symbol table stores direct variables as
mutable cells, so `$alias =& $value;` can bind both direct names to the same
cell in the current scope/global-routing model. Assignment through either name
updates the other, and `unset($alias)` or `unset($value)` removes only that
name binding while the cell remains alive through any remaining alias. Direct
array-offset reference sources now have narrow execution paths:
`$alias =& $array[$key];` and `$alias =& $array[$outer][$inner];` over a
direct array variable bind the target direct variable name to the selected
normalized-key array slot route. Missing keys and missing intermediate
containers on an existing array root are materialized before binding, and
undefined or `null` direct source roots are materialized as arrays containing
the selected `null` slot. Direct object-property array-offset reference sources
have a similarly narrow direct-variable target path:
`$alias =& $object->items[$key];` and
`$alias =& $object->items[$outer][$inner];` can bind the alias variable to an
explicit offset path inside a named visible property on a direct object
variable, including private `$this` and protected `$this`/peer-object roots
inside valid method visibility contexts. The route materializes a `null`
property as an array and missing selected slots as `null`. Writes through the
alias and through the direct array or supported object-property offset observe
the same value, and `unset($alias)` removes only the alias binding.
Append-at-depth reference sources have similarly bounded routes for direct
arrays and named visible object-property roots, including private `$this`,
protected `$this`, and protected peer-object roots when the source appears
inside a valid method visibility context. Dynamic public object-property
reference sources have similarly bounded routes for direct object variables.
Named visible object-property sources can bind through the current
public/private/protected method context. Missing or inaccessible declared
direct object properties can also dispatch to a visible non-static magic
`__get()` reference source when the method is declared by reference and returns
a direct variable through the existing reference-return method path; the alias
target binds to that returned variable cell rather than to a property slot.
Dynamic missing or inaccessible declared property names use the same magic
route after the property expression resolves to a string or integer. Non-array
roots, non-direct object expressions, general magic-property reference
containers, `__get()` returns of properties/offsets/expressions, dynamic
non-public append-source paths, `ArrayAccess` offsets, object-property
by-reference `foreach` iterables, exact broad by-reference `foreach`, full
reference containers, copy-on-write, and native lowering remain
future work. Direct variable sources
holding object values can also be assigned into direct array offsets under the
existing object-handle value model. Direct free-function
call sources have a narrow reference-return execution path when the function
declares `&`, returns a direct variable, and is used as a statement-form
reference-assignment source; the target name is bound to the returned variable
cell. Direct object method-call sources have the same narrow path for visible
non-static methods on object receivers. Direct named static method-call sources
have the same narrow path for visible static methods on direct class-name
receivers, and `self::` static method-call sources have the same path in active
class/method context. `parent::` static method-call sources have the same path
from active child class/method context when the resolved inherited method is
static. `static::` late-static method-call sources have the same path from
active class/method context when the called-class context resolves a visible
static method. Dynamic static receiver method-call sources have the same path
when the receiver evaluates to an object or class string and resolves a visible
static method. Missing static methods that would dispatch through magic
`__callStatic` are kept as an explicit reference-return boundary with a stable
unsupported-call diagnostic; magic reference-return dispatch is not modeled.
Direct array-offset reference targets also have narrow execution paths:
`$array[$key] =& $value;` works when the target root is a direct array
variable, the offset is explicit, and the source is a covered direct source
name: unaliased, part of a direct variable-to-variable alias group, or already
routed through covered array-offset alias metadata. `$array[] =& $value;`
works for the same direct root/source shape by appending through the runtime
array append cursor and routing the source group to the selected auto key.
`$array[$outer][$inner] =& $value;` works for explicit nested direct-array
targets by routing the source group to a normalized key path under the direct
array root. `$array[$outer][] =& $value;` uses the same key-path route after
appending through the runtime array append cursor at the selected nested
parent. Direct public object-property
array-offset targets have a similarly bounded route:
`$object->items[$key] =& $value;` can bind a covered direct source variable to
an explicit offset inside a declared public property on a direct object
variable. `$object->items[] =& $value;` and
`$object->groups[$key][] =& $value;` append through the same public-property
root route and bind the source name to the selected property-array auto key.
That route reads and writes the root through the existing public
object-property access path and materializes `null` property values as arrays
before selecting or appending the offset. The interpreter materializes missing
explicit keys, missing intermediate containers for the direct-array and
tested object-property target slices, and undefined or `null` direct-array
target roots through the direct-offset and nested-array materialization paths,
copies the current source value into the selected slot, then routes the source
name to that slot. Writes through either the source variable or the supported
direct array/object-property offset observe the same selected value, and
`unset($value)` detaches only the source name. When the covered public
object-property array is copied into a direct static variable, alias metadata
for referenced property slots is mirrored onto the copied static array so
writes through the source variable, the property slot, or the copied slot
synchronize for the covered key path. Undefined source variables start as
`null` before binding. PHP's deprecated false-root conversion, other non-array
roots, dynamic/magic/non-public property targets, non-direct sources,
non-static
`self::`/`parent::`/`static::`/dynamic-static sources, magic method reference
sources, full PHP reference containers, broader by-reference
`foreach` fidelity, mutation-ordering guarantees, alias rebinding, native
lowering, and copy-on-write remain future runtime work. ArrayAccess object
offset targets such as `$bag[$key] =& $value;` and property-held
`$holder->bag[$key] =& $value;` are an explicit runtime boundary with a stable
diagnostic, reflecting PHP's fatal behavior for assigning by reference to an
object array dimension while avoiding engine-specific notice/fatal text.
Direct array-offset reference sources mirror a subset of those same alias
routes for direct variable targets. `$alias =& $array[$key];` and
`$alias =& $array[$outer][$inner];` bind the target name to the selected
direct-array slot, materializing missing containers and selected slots as
needed. `$alias =& $array[];` and `$alias =& $array[$outer][];` append a
`null` slot through the runtime append cursor and bind the target name to that
selected slot. Direct public object-property sources such as
`$alias =& $object->property;` bind the target name to the selected property
root when that property is visible through the current public/private/protected
method context. Public property roots, private `$this->property` roots in the
declaring class, protected `$this->property` roots in visible class contexts,
and protected peer-object roots such as `$alias =& $other->property;` from a
valid child method context all reuse context-carrying object-property alias
metadata. Whole-property writes preserve those root aliases, while property
array-offset aliases detach to the previous property array when the whole
property is replaced. Dynamic public object-property sources such as
`$alias =& $object->$property;` reuse that same root route when the target
object is a direct variable, the property expression evaluates to a string or
integer name, and the selected property is public; allowed dynamic-property
objects such as `stdClass` materialize a missing selected property as `null`
before binding. Dynamic whole-property writes also detach narrower
property-array aliases before replacing the selected public property value.
Direct visible named object-property array-offset sources
follow the same bounded root route for named visible properties:
`$alias =& $object->items[$key];`,
`$alias =& $object->items[$outer][$inner];`,
`$alias =& $object->items[];`, and
`$alias =& $object->items[$outer][];`. Public properties use public alias
metadata, while private/protected properties use context-aware alias metadata
from valid method visibility contexts. String-keyed `$GLOBALS` append
reference sources such as `$alias =& $GLOBALS["bag"][];` and
`$alias =& $GLOBALS["bag"]["outer"][];` bind a direct alias variable to the
selected slot under the real global symbol table. `$GLOBALS[]` append
sources, non-string root keys, recursive `$GLOBALS` materialization,
dynamic-property sources on non-direct object expressions, dynamic
non-public property sources, magic property sources, non-variable reference
targets, full reference containers, copy-on-write, exact alias destruction
destructor ordering, and native lowering remain future work. Direct `ArrayAccess`
reference sources now have a narrow root bridge for `$alias =& $bag[$key]`,
property-held roots such as `$alias =& $holder->bag[$key]`, direct dynamic
property-held roots such as `$alias =& $holder->{$name}[$key]`, and literal
callback elements such as `array(&$holder->{$name}[$key])` when the direct
object variable or visible selected property value implements `ArrayAccess`, public
`offsetGet($offset)` returns by reference, and the method body is exactly the
current `return $this->property[$offset];` shape. Property-held roots are
parked in a hidden object-handle symbol and then reuse the same backing
property array alias metadata, including private/protected properties through
the declaring method context. Direct public property-held aliases remain bound
to the held `ArrayAccess` object if the holder property is later rebound.
Direct by-value or by-reference `offsetGet()` bucket-copy reads can mirror
covered nested public-property reference slots into a direct array variable
only for the same exact public `return $this->property[$offset];` body shape
and side-effect-free selected keys. The metadata mirror also discovers that
same bucket-copy source when the `ArrayAccess` object is reached through a
direct visible holder property or dynamic holder property, such as
`$holder->hook[10]` or `$holder->{$name}[10]`; the same evaluated-index
provenance also covers non-direct holder and expression-root holder reads such
as `$holders["box"]->hook[10]` and `make_holder($hook)->hook[10]` when the
final selected object has the exact bridge shape. By-value `offsetGet()` used
as a reference source follows PHP's bounded indirect-modification notice/no-op
path for the same exact bridge instead of creating a backing alias. Statement
form reference assignment from a direct object root such as
`$alias =& $bag[$key]`, a direct visible named or dynamic property-held root
such as `$alias =& $holder->bag[$key]` or
`$alias =& $holder->{$name}["outer"]["slot"]`, a visible property-held root
reached through a non-direct holder expression such as
`$alias =& $holders["box"]->bag[$key]`,
`$alias =& $holders["box"]->{$name}["outer"]["slot"]`,
`$alias =& $registry->holder()->bag["outer"]["slot"]`, or
`$alias =& make_holder($bag)->bag["outer"]["slot"]`, or another covered nested
child suffix snapshots the selected value into a detached local variable.
Append source forms such as `$alias =& $bag[]`, `$alias =& $holder->bag[]`,
`$alias =& $holder->{$name}[]`, `$alias =& $holders["box"]->bag[]`,
`$alias =& $holders["box"]->{$name}[]`,
`$alias =& $registry->holder()->bag[]`, `$alias =& make_holder($bag)->bag[]`,
`$alias =& $box->missing[]`, and `$alias =& $box->{$name}[]` use PHP's
`offsetGet(null)` path and snapshot the backing empty-string key value for
the exact bridge. These by-value reference-source forms emit the bounded
indirect-modification `E_NOTICE` and leave the backing `ArrayAccess` storage
unchanged when the alias is later written. Broader side-effecting or mixed
`ArrayAccess` chains remain outside that notice/no-op slice.
Function-parameter propagation of copied bucket provenance is covered for the
same direct-object, direct property-held, array-held, and expression-root
holder paths when the copied bucket is passed as a direct variable to a
by-value direct user-function parameter, direct closure parameter,
string-user-function `call_user_func()` parameter, or positional literal
`call_user_func_array()` parameter. Direct stored positional
`call_user_func_array()` argument-array variables now reuse that same bounded
provenance when the stored slot was populated from a direct copied-bucket
variable, such as `$args = [$bucket, "label"]`; the callback parameter imports
mirrored nested aliases from the stored argument slot and writes those nested
reference-slot updates back through the original copied-bucket alias group.
When a direct `ArrayAccess` object stores an array literal or copied array into
a keyed bucket through the exact public `offsetSet($offset, $value) {
$this->property[$offset] = $value; }` bridge, including the bounded literal
int/string prefix or suffix variants
`$this->property["bucket"][$offset] = $value;` and
`$this->property[$offset]["bucket"] = $value;`, the interpreter binds the
array literal's reference slots or mirrors the copied array's alias metadata
onto that backing bucket. Later exact by-value or by-reference
`offsetGet($offset) { return $this->property[$offset]; }` bucket copies can
therefore preserve those nested reference slots, including private/protected
backing properties reached through the method's declaring-class context.
Direct append stores through `$bag[] = $array` now cover the same stored-bucket
reference-slot propagation for two public `offsetSet(null, $value)` shapes:
the exact `$this->property[$offset] = $value;` bridge, where PHP's null offset
stores under the backing array's empty-string key, and the branchy append
bridge `if ($offset === null) { $this->property[] = $value; return; }
$this->property[$offset] = $value;`, where metadata is attached after the user
method call to the actual appended integer key. That branchy append-key bridge
can use the same literal prefix before both the append and the offset
parameter, such as
`if ($offset === null) { $this->property["bucket"][] = $value; return; }
$this->property["bucket"][$offset] = $value;`, and the same literal suffix
after both the append slot and offset parameter, such as
`if ($offset === null) { $this->property[]["bucket"] = $value; return; }
$this->property[$offset]["bucket"] = $value;`. The recognizer also accepts
the equivalent `if/else` body without a `return`, where the `else` branch is
exactly the keyed assignment for the same literal prefix/suffix path:
`if ($offset === null) { $this->property["outer"][]["leaf"] = $value; }
else { $this->property["outer"][$offset]["leaf"] = $value; }`. The same
keyed-assignment recognizer is reused for non-null keyed stores such as
`$bag["leaf"] = $array`, so stored reference metadata is attached to the
non-null branch's backing bucket after the method call. Direct visible property-held
append stores such as `$holder->bag[] = $array` reuse the same hidden
`ArrayAccess` object root used by property-held reference-source bridges, then
attach literal-reference or copied-array alias metadata to the held object's
backing bucket for those same exact empty-key and branchy append-key bridges.
Direct visible dynamic property-held append stores such as
`$holder->{$name}[] = $array` evaluate the property name once, select the same
concrete visible property, and reuse that property-held hidden-root path before
attaching metadata to the held object's backing bucket.
Non-direct holder visible property-held append stores such as
`$holders["box"]->bag[] = $array` evaluate the holder expression once into a
temporary object root, then reuse the same property-held `ArrayAccess` append
bridge for the selected visible named property. The narrow setup assignment
`$holders["box"]->bag = $value` writes through that evaluated holder object so
fixtures can establish the held `ArrayAccess` object without claiming broader
compound, dynamic, magic, or native property support.
Dynamic non-direct holder visible property-held append stores such as
`$holders["box"]->{$name}[] = $array` use the same temporary object-root path:
the holder expression is evaluated once, the dynamic property-name expression
is evaluated once, and the selected visible property reuses the property-held
append bridge. This does not add dynamic non-direct whole-property setup
assignment; that remains outside the COW append slice.
Visible property-held keyed stores use the same hidden-root machinery for the
held `ArrayAccess` object. Direct stores such as
`$holder->bag["leaf"] = $array` and non-direct named or dynamic holder stores
such as `$holders["box"]->bag["leaf"] = $array` or
`$holders["box"]->{$name}["leaf"] = $array` dispatch
`offsetSet($key, $value)` first, then ask the bounded `offsetSet()` backing
bucket recognizer for the concrete root and keys. When the exact non-branchy
or `if/else` keyed recognizer succeeds, array-literal reference slots or
copied-array alias metadata are attached to that backing bucket. Multi-key
property-held keyed stores reuse the hidden root differently: the runtime asks
the exact public by-reference `offsetGet($offset) { return
$this->property[$offset]; }` bridge for the parent path, then writes the
nested plain-array leaf through the returned backing alias. This covers
direct, non-direct named-holder, and non-direct dynamic-holder paths such as
`$holder->bag["outer"]["leaf"] = $array`,
`$holders["box"]->bag["outer"]["leaf"] = $array`, and
`$holders["box"]->{$name}["outer"]["leaf"] = $array`.
Direct magic-property-provided keyed `ArrayAccess` containers now use the same
object-level helpers after `__get()` returns the object handle. For
`$box->missing["leaf"] = $array`, the runtime calls visible public
`__get($name)` once, dispatches `offsetSet($key, $value)`, and attaches
metadata to the recognized backing bucket. For
`$box->missing["outer"]["leaf"] = $array`, it asks the exact public
by-reference `offsetGet()` bridge for the parent bucket and writes the nested
plain-array leaf through that alias. Non-direct magic keyed stores now have
distinct assignment-target variants from non-direct magic append stores, so
named and dynamic holder forms such as
`$holders["box"]->missing["leaf"] = $array`,
`$holders["box"]->missing["outer"]["leaf"] = $array`, and
`$holders["box"]->{$name}["leaf"] = $array` evaluate the holder once into a
temporary object root and reuse the same magic keyed bridge. Dynamic direct
magic keyed property names use a direct dynamic object-property array-index
assignment target, evaluate the property expression once, and then reuse the
same visible property-held or magic `ArrayAccess` keyed bridge for
`$box->{$name}["leaf"] = $array` and nested plain-array leaves. By-value
terminal/plain-array nested mutation, unsupported
`offsetSet()`/`offsetGet()`/`__get()` bodies, broader mixed chains, and native
lowering remain outside this path.
For supported method-body side effects, method scopes now carry a detached
object-property array-offset ledger alongside the existing alias metadata.
When a body overwrites or unsets a tracked object-property array, the scope
records the pre-write fallback for each detached path. Successful nested
method calls propagate that ledger back to the caller scope before stale alias
sync runs, so helper methods and direct `call_user_func([$this, ...])`
callbacks invoked by `ArrayAccess::offsetSet()` or magic `__set()` can detach
caller aliases even when the overwritten object-property path is immediately
recreated. The same detach operation scans tracked copied-array sources in the
scope where the detach happens and again during caller writeback: if a local
array copy came from the detached object-property path, the local copy is
rehydrated with the detached reference cell before the alias group is removed.
This keeps by-value `offsetGet()`/`__get()` locals returned after a
direct/helper/callback backing overwrite connected to the old detached
reference leaf while the recreated backing property remains separate.
Non-static anonymous closures created inside these method bodies record the
current `$this`, class context, and called-class context; direct invocation,
closure-valued `call_user_func()`, positional `call_user_func_array()`, and
covered reference-return/source closure paths then execute the callback with
that bound context, so `$this`-based backing writes inside the closure use the
same alias and copied-source propagation machinery. This is still a bounded
interpreter-scope writeback model rather than a general PHP reference
container, and it does not cover unsupported syntax, `Closure::bind`/`bindTo`,
untracked dynamic containers, exact diagnostics, or native lowering.
Direct magic-property append stores such as `$box->missing[] = $array` and
`$box->{$name}[] = $array` are a separate store path from magic append
reference sources. For the covered store shape, the runtime calls visible
public `__get($name)` once, requires the returned value to be an `ArrayAccess`
object, then dispatches `offsetSet(null, $value)` through the same hidden
`ArrayAccess` object-root propagation helper used by visible property-held
receivers. This path preserves stored-bucket nested reference metadata for the
exact empty-key and branchy append-key bridges, while the existing
`$alias =& $box->missing[]` reference-source path remains an
`offsetGet(null)` notice/no-op boundary.
Non-direct magic-property append stores such as
`$holders["box"]->missing[] = $array` and
`$holders["box"]->{$name}[] = $array` now reuse that same magic store route
after evaluating the holder expression once into a temporary object root. The
dynamic property-name expression is evaluated once for the dynamic spelling.
The covered object case calls `__get()` once for the append store and does not
call `__set()`, then attaches the appended array's reference metadata to the
returned `ArrayAccess` object's exact empty-key or branchy append-key backing
bucket.
For the focused one-key nested magic `ArrayAccess` append shape, such as
`$box->missing["outer"][] = $array` and
`$holders["box"]->{$name}["outer"][] = $array`, the same magic append-store
helper does not use `offsetSet(null, $value)`. It calls visible public
`__get($name)` once to obtain the `ArrayAccess` object, resolves the parent
bucket through exact public by-reference `offsetGet($offset) { return
$this->items[$offset]; }`, appends into that backing property alias, and
attaches array-literal or copied-array reference metadata to the actual
appended bucket. Nested array reads preserve provenance from an inner
`ArrayAccess` copy source, so a later copied bucket read such as
`$box->missing["outer"][0]` can mirror the stored bucket's nested reference
metadata. By-value `offsetGet()` remains the PHP notice/no-op boundary for
this lane.
For the focused deeper magic `ArrayAccess` append shape, such as
`$box->missing["outer"]["inner"][] = $array`, the runtime asks the
`ArrayAccess` reference-source resolver for the complete parent path before
the final append. When the first selected bucket is a plain array, that keeps
the existing behavior where `offsetGet("outer")` provides the backing alias
root and the remaining parent path is materialized as plain-array storage. If
that first selected bucket is another `ArrayAccess` object using the same
exact by-reference `offsetGet($offset) { return $this->items[$offset]; }`
bridge, the resolver recurses and the append lands in the inner object's
selected backing bucket. Copied-bucket reads such as
`$box->missing["outer"]["inner"][0]` can then mirror the stored bucket's
nested reference metadata. A by-value outer `offsetGet()` can also serve as
that intermediate object hop when it returns the nested `ArrayAccess` object
handle; the inner object still needs the exact by-reference `offsetGet()`
bridge for mutation. Broader mixed chains, by-value terminal/plain-array
mutation, and side-effecting `offsetGet()` bodies remain outside the current
model.
The same recursive resolver is used for statement-form reference assignment
sources when magic `__get()` itself returns a supported direct variable by
reference. For a source such as
`$alias =& $box->missing["outer"]["inner"]`, the alias can bind to the inner
`ArrayAccess` backing bucket even when the outer `offsetGet()` returns the
intermediate object by value. Direct append writes through that bound variable
use the alias-backed append helper, so `$alias[] = $array` mutates the inner
bucket instead of materializing a detached local array.
For the same focused object-handle root shape, by-value `__get()` can also
return the outer `ArrayAccess` object and then use the same recursive resolver
to bind the alias to the inner backing bucket. This is intentionally still an
object-handle bridge, not broad by-value magic-property reference semantics
for plain arrays or terminal by-value `offsetGet()` results.
The resolver is not hard-coded to exactly two objects: the tested three-object
chain follows by-value `ArrayAccess` object handles through outer and middle
buckets before binding to the terminal by-reference leaf bucket. The remaining
boundary is arbitrary side-effecting/broader `offsetGet()` semantics and
non-object or by-value terminal containers, not the mechanical recursion depth
of the current exact bridge.
Because these bridges often store equivalent object handles under hidden
runtime names, object-property array writes canonicalize their alias root
before detaching descendants and syncing aliases. That keeps direct backing
writes through a visible object name coherent with aliases originally reached
through a factory-return holder, magic `__get()`, and nested `ArrayAccess`
object handles.
For the covered magic/`ArrayAccess` alias-source path, local alias unset uses
the existing static-name detach machinery: removing the local variable binding
does not remove copied-bucket reference-slot aliases stored under the backing
object-property root. Reusing the local name therefore stays detached, while
subsequent backing-bucket writes can still synchronize the nested reference
slots that were copied into the bucket.
The same magic append-store helper also covers the focused plain-array route
when visible public `__get($name)` returns a direct variable by reference and
that returned cell currently holds an array or `null`. The runtime binds the
returned cell to a hidden temporary root, appends through the normal array
append-alias machinery, then canonicalizes the alias root back to a visible
static variable sharing that cell before attaching array-literal or copied
array reference metadata. That keeps later reads such as `$storage[0]`
connected to nested reference slots created through `$box->missing[]` or
`$holders["box"]->missing[]`. For one broader by-reference `__get()` body
shape, the helper recognizes a single direct `return $this->property;` body
when that property is visible to the caller's later backing write, and for
one focused private `$this->property` route where a same-class method later
mutates that private backing bucket. The same focused private route also
recognizes `return $this->{$name};` when `$name` is the `__get()` parameter,
using the requested inaccessible property name as the backing property. It
also recognizes the exact backing-array offset shape
`return $this->property[$name];`, where `$name` is the `__get()` parameter,
plus literal keys before or after that parameter, such as
`return $this->property["bucket"][$name];` and
`return $this->property[$name]["bucket"];`. Those literal keys and the
requested magic property are prepended under that backing property before
applying the append path. It appends into the corresponding
object-property alias root and mirrors copied reference-slot metadata there,
without creating a general object-property cell model. Statement-form
reference sources below the same analyzed backing-array `__get()` body, such
as `$alias =& $box->missing["outer"];`, now reuse that object-property alias
root and prefix-key analysis before binding the local alias, instead of
executing the reference return body through the direct-variable-only fallback.
Non-direct dynamic holder roots, such as
`$alias =& $holders["box"]->{$property}["outer"];`, evaluate the holder once
into a temporary object root and then follow the same magic source helper; the
object-property alias root is canonicalized so later writes through the real
holder object stay connected to aliases created through the temporary.
The same
object-property-root path is tested through a two-key parent append, so
`$box->missing["outer"]["inner"][]` attaches metadata below
`$box->store["outer"]["inner"]`. For the private route, instance-method
return sync now resynchronizes caller-side alias groups for the receiver
object after method-local writes through `$this->store...`, so nested
references copied into the private bucket update their original variables.
Broader private/protected `$this` property reference returns and arbitrary
method-local alias lifetimes remain explicit boundaries. For the focused
nested append shapes, the same helper passes the complete parent key path
into the array append-alias
machinery, so one-key paths such as `$box->missing["outer"][]` and
`$holders["box"]->{$name}["outer"][]`, plus the tested two-key paths such as
`$box->missing["outer"]["inner"][]` and
`$holders["box"]->{$name}["outer"]["inner"][]`, attach metadata to the
appended bucket under the returned cell's visible static root. By-value
`__get()` returning a
plain array or `null` is intentionally a notice/no-op path for the covered
array RHS shapes, matching PHP's indirect-modification behavior instead of
mutating detached storage.
By-value helper parameters that import copied-bucket provenance detach those
mirrored static-array provenance paths when the parameter variable is replaced:
writes through the copied bucket before replacement still write back to the
original reference slot, while later nested writes through the replacement
array remain local to the helper. Replacing the outer copied bucket variable,
unsetting and reusing the direct callback variable name, reusing a lingering
by-reference foreach callback variable by assigning a new array after the
copied bucket loop, and mutating two distinct nested reference slots in one
copied bucket match the focused PHP probes. Broader alias lifetime after
replacing non-direct containing properties, side-effecting or broader
`offsetGet()` bodies, mixed nested
ArrayAccess chains beyond the documented one-level bridge, non-empty append
paths below plain arrays returned from magic `__get()` beyond the focused
one-key parent append shape, dynamic non-direct whole-property setup
assignment,
dynamic property names that trigger unsupported fallback or inaccessible
properties,
arbitrary nested reference slots copied from ArrayAccess storage outside the
exact `offsetSet()`/`offsetGet()` bridge, dynamic or repeated
`offsetSet()` parameters in branchy append bridges, dynamic `offsetGet()`
parameter keys, non-literal bridge path keys, and
real reference containers remain future work.
String-keyed `$GLOBALS` reference targets also have narrow routes:
`$GLOBALS["name"] =& $value;`, `$GLOBALS["bag"]["slot"] =& $value;`, and
`$GLOBALS["list"][] =& $value;` bind the selected root global symbol or
root-global array slot to a direct source variable cell, including from
function scope. The source may be unaliased, part of a direct
variable-to-variable alias group, or already routed through covered
array-offset alias metadata; for the array-offset source shape, the selected
`$GLOBALS` slot is merged into the same bounded alias group. Missing root
globals, `null` root globals, and missing intermediate containers materialize
as arrays for the nested/append forms, and append targets use the runtime
array append cursor under the selected root global array. Writes through the
source name, the direct global variable path, the supported string-keyed
`$GLOBALS` offset path, and other covered alias-group slots observe the same
value, and unsetting the source name detaches only that name. Nested by-value
writes through supported string-keyed `$GLOBALS` paths route through the root
global symbol table and sync covered aliases. Non-string root keys,
`$GLOBALS[] =& $value`, non-direct sources, recursive `$GLOBALS`
materialization, full reference containers, copy-on-write, exact mutation
ordering, and native lowering remain future work.
By-reference function and method return declarations are represented as
function metadata so declaration-contained code can register. Normal invocation
of direct free-function, direct visible object-method, direct named static
method, `self::`, `parent::`, `static::`, and dynamic static receiver
reference-return calls can execute the same direct-variable return shape used
by statement-form reference assignment and then expose a by-value snapshot of
the returned cell. That path reuses the covered array-offset writeback
machinery for by-reference parameters. Direct named static method and dynamic
static receiver calls also use the bounded array-offset bridge below missing or
inaccessible declared magic properties when visible public `__get()` returns a
direct variable by reference. Normal named and dynamic property reads through
that same bounded reference-returning `__get()` direct-variable body shape now
borrow the returned cell long enough to produce a by-value snapshot, without
recording alias metadata for the read result. Direct-variable reference
assignment from named and dynamic array offsets below those magic properties
temporarily roots the returned cell in the symbol table and reuses the
existing array-offset alias metadata for the selected slot. That is still a
bounded alias bridge, not a general runtime reference container. Non-direct
holder array offsets below magic properties are limited to direct
user-function by-reference parameters and direct-variable reference
assignment, while the separate `ArrayAccess` append-store path covers the
documented direct and non-direct magic receivers whose `__get()` returns an
`ArrayAccess` object, and the separate plain-array append-store path covers
empty appends into direct-variable cells returned by reference from
`__get()` plus the focused one-key nested append shape. Deeper append offsets
below plain arrays returned from magic properties, non-direct magic-property
holders outside the selected array and append slices, broader `__get()` return
bodies such as properties, offsets, expressions, or nested-control-flow
returns, mixed nested `ArrayAccess` chains, and arbitrary reference-return
invocations still report stable runtime boundaries before any by-value return
is produced.
By-reference parameters are also metadata-first: omitted optional
by-reference parameters can use their defaults as ordinary local values, while
provided direct-variable by-reference arguments bind the callee parameter name
to the caller's variable cell for the duration of current user-function,
instance-method, constructor, named static method, `self::` static method, and
`parent::` instance/static method calls, and late-bound `static::` static
method calls. Writes through the parameter are visible before the call returns,
and `unset($param)` detaches only the callee's local name from the shared cell.
Direct array-offset arguments, including request-bag paths such as
`$_REQUEST["payload"]["slot"]`, and direct visible named object-property
array-offset arguments use a narrower copy-in/writeback bridge on the
user-function, instance-method, named static method, `self::` static method,
`parent::` instance/static method, and late-bound `static::` static method
dispatch paths.
The bridge keeps the local parameter's original cell so mutations before
`unset($param)` are written back at return, while later writes to a detached
local name are not. It still does not expose in-call PHP reference-container
identity.
Direct `call_user_func()` invocation of public `[object, method]` instance
callbacks and public `["ClassName", "method"]` static callbacks now mirrors
the same bounded PHP behavior already used for string user-function and
ordinary closure callbacks with reached by-reference parameters: each supplied
argument expression is evaluated by value, the interpreter emits a recoverable
warning for each reached non-variadic by-reference parameter, and callee writes
mutate only the callee-local parameter value rather than the caller variable or
array slot. Object receiver state still mutates through `$this` when the method
body writes visible properties.
`call_user_func_array()` reuses that same
direct cell binding for narrow string user-callbacks, public object-method
array callbacks, and public class-string static-method array callbacks where
the argument array is an unkeyed or integer-keyed literal containing
`&$directVariable` elements for reached by-reference parameters. The same
callback path can also copy in and write back direct array-offset elements,
including request/global bag paths such as `&$_REQUEST["payload"]["slot"]`
and `&$GLOBALS["bag"]["slot"]`, plus direct visible named object-property
array-offset elements such as `&$object->items[$group][$key]`, using the
bounded output-parameter bridge rather than in-call reference-container
identity. Visible named object-property callback roots use public alias
metadata for public slots and context-aware alias metadata for
private/protected slots reached from a valid method visibility context.
Literal reference elements whose direct variable is already backed
by covered array-offset alias metadata, such as
`array(&$payload, ...)` after `$payload =& $_REQUEST["payload"];`, reuse the
same alias group for copy-in/writeback instead of requiring a normal caller
variable cell. Statement-form reference assignment from a reference-returning
`call_user_func_array()` source can now return the same bounded direct
array-offset or visible named object-property array-offset route when the
callback returns that reached parameter, so the assigned alias is represented
in symbol-table alias metadata instead of as a general runtime reference
container. If the callback returns a child slot below that alias-backed
literal direct-variable parameter, the returned suffix is appended to the
underlying alias group. Direct stored argument
arrays can also satisfy reached by-reference
parameters when the selected slots were previously assigned by reference
through the covered direct array-offset target path. That path finds the
stored slot in the existing alias metadata, copies the current slot value into
the callee parameter, and writes the final parameter value back through the
same alias, which also syncs covered direct-variable, copied-array,
request-bag/global, public object-property, and context-aware non-public
object-property alias groups. For
statement-form reference assignment, a callback that returns that reached
parameter by reference now binds the assigned alias back to the same covered
alias group rather than only to the stored argument array slot. Normal
`call_user_func_array()` invocation can use the same copy-in/writeback bridge
when the callback function or public array-callable method is declared as
returning by reference, but the callback call itself still returns a value.
The stored argument array may be a direct variable or a direct visible named
object property, including private/protected properties reached from a valid
method visibility context. A stored argument-array variable may itself be
routed through covered array-offset alias metadata, including ordinary array
roots, request/global roots, and visible object-property array roots; slot
writes and stored-slot lookups compose the argument-array root alias with the
selected integer key before binding or writing back.
When a supported helper method mutates a shared holder object before returning
it, object-property copied-source metadata records dirty object/property keys
in the callee scope and syncs portable copied-source roots back to the caller
scope. This lets recreated holder properties such as `$this->holder->args`,
dynamic holder properties, and nested holder-array roots remain visible to the
stored `call_user_func_array()` source-aware path after the helper returns,
without treating arbitrary local symbol names as caller-visible provenance.
The stored callback path also has a narrow anonymous alias-group route for
reference assignments from covered append-offset sources into stored argument
slots, such as `$args[] =& $items[]` or
`$args["value"] =& $object->items[]`. The append source materializes a new
`null` slot, the stored argument target materializes its selected slot, and
both aliases are recorded together so later callback writeback and supported
reference-return binding sync the source slot and stored argument slot without
introducing a general runtime reference container.
Direct array literals assigned to stored callback argument variables can also
record reference elements below bounded magic `__get()` array roots, such as
`$args = array(&$object->missing["slot"])`, by temporarily rooting the
direct-variable cell returned from visible public reference-returning
`__get($name)` and binding the stored argument slot to the selected array
offset alias. Later normal callback writeback and supported reference-return
binding use the same stored-slot alias lookup as other stored argument arrays.
This remains limited to direct object magic-property roots and the existing
direct-variable-returning `__get()` body shape.
For the bounded `call_user_func_array()` reference path, string-keyed
argument-array entries can also bind by declared parameter name for current
user-function callbacks and public object/static array-callable methods. The
literal path maps direct variable, direct array-offset, and visible named
object-property array-offset reference elements to the named parameter, while
the stored-array path looks up the existing alias metadata by the same string
key before copy-in/writeback or reference-return alias binding.
The by-value callback path now uses the same parameter-name mapping for
current user functions and public object/static array-callable methods when
the argument array contains string keys and no reached by-reference parameter
requires alias binding. That path is still an interpreter argument-mapping
layer over materialized values: duplicate names, unknown names, positional
entries after named entries, and variadic named arguments stop with stable
runtime diagnostics instead of pretending a general PHP named-argument model
exists.
This deliberately does not model full PHP reference containers, reference
array literals stored by value, stored arrays whose reached slots were not
assigned by reference, non-direct stored array expressions beyond direct
visible named object-property arrays, direct reference assignment between
object-property array offsets without an intermediate alias variable,
execution past unknown or duplicate string-keyed callback argument names,
positional arguments after a string-keyed named argument, variadic named
callback arguments, dynamic key expressions in the literal reference
named-argument path,
dynamic ArrayAccess reference roots, append-offset ArrayAccess source roots
outside the exact direct/property-held/non-direct visible property-held
`offsetGet(null)` bridge, stored-array ArrayAccess roots outside the current
direct/property-held `offsetGet()` reference bridge, non-public property roots
outside the current valid
method-context named-property slice, dynamic static receiver callback
object-property array arguments, broader reference-return binding, exact
by-reference `foreach`, exact alias destruction/destructor ordering, or
copy-on-write.
By-reference `foreach` value syntax over a direct array variable has a bounded
interpreter path. Each iteration reads the active entry from the current
ordered array, writes the key variable by value, routes the value variable to
the active direct array slot for the body, and advances using the current array
order instead of an initial key snapshot. Appended elements and newly inserted
tail entries are visited by the same loop, and direct writes to the current
slot are visible through the loop variable. If the active direct array slot is
unset during the body, the value variable is detached onto the removed value;
a same-key reinsertion in that body does not retarget the value variable until
a later iteration reaches the reinserted tail entry. After loop completion, the
value variable remains routed to the last successfully iterated existing slot
until `unset($value)` detaches it. Empty array iteration creates no lingering
route. Temporary array-producing expressions such as array literals and direct
non-reference-returning function calls use the same direct-slot loop machinery
after the evaluated array is stored in an internal hidden array slot; this
preserves loop-body and post-loop value-variable aliasing to the temporary
without claiming mutation of a source lvalue. Direct string-keyed
`$GLOBALS["name"]` roots use the same alias metadata with a global-array root,
so the loop value can mutate and linger on slots under the real root symbol
table from top-level or function scope. This intentionally avoids claiming
full mutation-during-iteration fidelity, broad array reordering/replacement
semantics, nested lvalue iterable support such as `$items[0]` or
`$GLOBALS["name"]["child"]`, non-string-keyed `$GLOBALS` roots,
reference-returning call iterables, object/`Traversable` iteration,
destructuring targets,
array/object/`ArrayAccess` offset loop variables, nested-offset loop values,
full reference containers, or PHP copy-on-write.
- dynamic method/property names, broader visibility enforcement for
  non-public properties/constructors, static methods and broader static member
  semantics, magic methods beyond the current direct missing-property
  `__get`/`__isset`/`__set`/`__unset`, missing-method `__call`/`__callStatic`,
  direct object-to-string `__toString` including current interpolation,
  bounded core interface metadata, and direct/property-held `ArrayAccess`
  offset and compound-assignment/increment-decrement slices,
  typed/default property compatibility, broader
  `parent::`/`self::`/`static::`, broader
  inheritance and constructor semantics, exact nested declaration timing, and
  exact PHP object lifecycle behavior

The first native-runtime ABI prerequisite lives in
`docs/NATIVE_RUNTIME_ABI.md`. It exposes a C-compatible scalar handoff type for
`null`, booleans, integers, and floats, plus exported constructor symbols in
`php_runtime`. It also has probe-only owned byte-buffer helpers, an opaque
copied PHP string-handle helper surface, and a bounded valid-UTF-8
string-handle-to-runtime-value bridge. It also exposes a bounded diagnostic
handle path for native string-to-value conversion failures when a null string
handle or non-UTF-8 byte payload is supplied. Null-only opaque array, object,
resource, and reference handle shapes pin pointer-sized future ABI slots without
claiming storage or semantics for those PHP value kinds. This is intentionally
only an ABI seed for future generated-code runtime helper calls. The
compiler-side helper probe renders `usize`-shaped helper signatures from an
explicit pointer-width target so the ABI sketch can distinguish 32-bit and
64-bit targets and now includes a deterministic branch on a nullable
string-to-value result that clones, reports, and frees the diagnostic message
only on the failure path. Normal generated IR for the documented direct and selected
string-output slices now uses the same nullable-value branch shape and reports
string-to-value conversion diagnostics through the bounded stderr helper before
shared handle cleanup. Linked native execution, broad runtime helper calls from
normal generated IR, string interning, binary PHP string value handles,
array/object/resource/reference storage, copy-on-write, stack frames, general
diagnostics, and request or WordPress host state are still not implemented.

## Native Codegen

The first backend emits LLVM IR text and shells out to `clang` for assembly.
This is deliberately less work than an x86-64 backend and lets the project focus
on PHP semantics first.

Tradeoff: Milestone 1 native lowering is smaller than interpreter support. The
backend must return a codegen error for unsupported constructs rather than
pretend to compile them.
Expression-form `include`, `include_once`, `require`, and `require_once` have
a dedicated native rejection boundary, separate from the statement-form
multi-file boundary. This keeps the extra expression semantics visible:
included-file return values, `_once` de-duplication result values,
caller-scope side effects, source loading, path resolution, declaration
registration, source mapping, and exact diagnostics all remain future native
work.
Statement-form reference assignment has a dedicated native rejection boundary,
separate from general mutation lowering. `Stmt::ReferenceAssign` is rejected
before lowering its source or target operands in both LLVM IR emission and the
C assembly fallback path, including direct variable, array-offset,
object-property, function-call, method-call, static-call, magic `__get`, and
`ArrayAccess`-shaped sources or targets. Real support depends on native
reference containers, alias-aware symbol tables, copy-on-write, object/property
alias roots, and exact diagnostic behavior.
Object-property `ArrayAccess` offset shapes also have a dedicated native
rejection boundary for reads, writes, `isset`, `empty`, `unset`, and compound
paths. This prevents property-held object offsets that `phpc run` can dispatch
through `offsetGet`, `offsetSet`, `offsetExists`, and `offsetUnset` from being
reported as only generic array or object lowering gaps. Native execution still
requires object handles, ArrayAccess method dispatch, reference/COW semantics,
and exact PHP diagnostics. Direct `$value[$key]` remains on the generic array
boundary at codegen time unless later analysis proves `$value` is an
`ArrayAccess` object.
`instanceof` expressions have a dedicated native rejection boundary for the
current class/interface relationship checks. Both LLVM IR emission and the C
assembly fallback path reject the `instanceof` AST node before lowering the
left operand, so missing native class metadata tables, object handles,
inheritance/interface registries, class-name resolution, autoload interaction,
references/copy-on-write, and exact native diagnostics stay visible instead of
collapsing into the broader object/class boundary.
Method-call expressions also have a dedicated native rejection boundary for
instance calls, named static calls, object/static-receiver calls, `self::`,
`parent::`, and late-static `static::` calls. Both LLVM IR emission and the C
assembly fallback path reject the method-call AST node before lowering the
receiver or arguments, so missing native method lookup, receiver/static
receiver resolution, `$this` and late-static-binding context, argument/arity
diagnostics, visibility checks, references/copy-on-write, and exact native
method-call errors stay visible instead of collapsing into the broader
object/class diagnostic.
Class-name constants have a dedicated native rejection boundary for
`ClassName::class`, `self::class`, `parent::class`, and `static::class`. LLVM
IR emission rejects those AST nodes before lowering class-name resolution, so
missing native class-name resolution, active class/parent and
late-static-binding context, namespace/import canonicalization, autoload-free
class lookup interaction, references/copy-on-write, and exact native
class-name constant diagnostics stay visible instead of collapsing into the
broader static-member diagnostic.
Static class members have a dedicated native rejection boundary for class
constants, static property reads/writes, and dynamic static-property
receivers. Both LLVM IR emission and the C assembly fallback path reject those
AST nodes before lowering class/member operands, so missing native class
constant tables, static property storage, class context, late-static-binding
resolution, visibility checks, autoload/class lookup, references/copy-on-write,
and exact native static-member errors stay visible instead of collapsing into
the broader object/class diagnostic.

Current assembly emission order:

1. Generate LLVM IR text.
2. Use `clang` if available.
3. Use `llc` if available.
4. If no LLVM assembly tool is available, generate equivalent C for the same
   narrow lowerable subset and ask `cc -S` for assembly.

The C fallback exists only to keep `phpc compile --emit-asm` executable on
machines without LLVM tools. It must not grow into the primary backend without a
documented architecture decision.
Assembly CLI coverage intentionally checks a normalized success summary for the
current straight-line scalar echo/assignment subset instead of committing exact
assembly text, because `clang`, `llc`, and the `cc -S` fallback produce
platform- and toolchain-specific output.
Assembly rejection CLI coverage also runs a representative unsupported array
program with backend tools removed from `PATH`; this pins that LLVM lowering
diagnostics are returned before backend discovery or invocation.
A separate backend-absence CLI snapshot runs a lowerable scalar program with
backend tools removed from `PATH`; this pins the missing-backend diagnostic
after LLVM lowering has succeeded.
The C fallback path has its own CLI snapshot that hides `clang` and `llc` while
exposing only `cc`; it checks normalized success properties instead of exact
assembly text.
A selected-backend failure snapshot exposes a deterministic fake `clang` that
passes discovery and then exits nonzero after accepting generated LLVM IR. That
pins the CLI diagnostic shape for backend failure without committing real
toolchain stderr.
A selected-`llc` snapshot exposes only a deterministic fake `llc` while hiding
`clang` and `cc`; it pins backend selection order with normalized assembly
properties instead of backend-specific assembly text.
A selected-`llc` failure snapshot exposes only a deterministic fake `llc` that
passes discovery and then exits nonzero after accepting generated LLVM IR. That
pins the CLI diagnostic shape for `llc` failure without committing real
toolchain stderr.
A C fallback failure snapshot exposes only a deterministic fake `cc` that passes
discovery and then exits nonzero after accepting generated C fallback source.
That pins the CLI diagnostic shape for `cc -S` fallback failure without
committing real toolchain stderr.
A discovery-exhaustion snapshot exposes deterministic fake `clang`, `llc`, and
`cc` commands whose `--version` probes all fail. That pins the missing-backend
diagnostic when candidate command names exist on `PATH` but no backend passes
discovery.
An empty-stderr backend-failure snapshot exposes a deterministic fake `clang`
that passes discovery and exits nonzero without stderr after accepting
generated LLVM IR. That pins the fallback diagnostic detail for selected
backend failures when a tool provides no error text.
An empty-stdout backend-success snapshot exposes a deterministic fake `clang`
that passes discovery and exits successfully after accepting generated LLVM IR
without producing assembly text. That pins the selected-backend diagnostic for
empty stdout instead of treating an empty assembly artifact as success.
A success-with-stderr backend snapshot exposes a deterministic fake `clang`
that passes discovery, emits nonempty assembly stdout, writes a diagnostic to
stderr, and exits successfully. That pins the current boundary: successful
backend stderr is intentionally ignored by `phpc`, while assembly is taken only
from stdout.
Success-with-stderr fallback snapshots expose deterministic fake `llc` and
`cc` tools with the same successful behavior, proving the shared success path
after LLVM fallback selection and after the `cc -S` C fallback selection.
Empty-stderr fallback failure snapshots expose deterministic fake `llc` and
`cc` tools that exit nonzero without diagnostics. That pins the shared
`backend exited without stderr` diagnostic detail after LLVM fallback selection
and after the `cc -S` C fallback selection.
Empty-stdout fallback success snapshots expose deterministic fake `llc` and
`cc` tools that exit successfully without assembly text. That pins the shared
empty-output diagnostic after LLVM fallback selection and after the `cc -S` C
fallback selection.
Whitespace-only fallback success snapshots expose deterministic fake `llc` and
`cc` tools that exit successfully with only whitespace on stdout. That pins the
shared whitespace-only-output diagnostic after LLVM fallback selection and
after the `cc -S` C fallback selection.
A selected-backend whitespace-only success snapshot exposes a deterministic
fake `clang` that exits successfully with only whitespace on stdout. That pins
the same shared whitespace-only-output diagnostic before fallback selection.
A selected-backend whitespace-with-stderr success snapshot exposes a
deterministic fake `clang` that exits successfully with only whitespace on
stdout while writing stderr diagnostics. That pins stdout validation as the
reported failure and keeps successful-backend stderr unsurfaced even when the
successful stdout artifact is invalid.
A selected-backend whitespace-with-stderr precedence snapshot exposes the same
invalid successful `clang` output while `llc` and `cc` are also available. That
pins the no-recovery boundary after invalid selected-backend output.
A selected-backend empty-stdout-with-stderr precedence snapshot exposes invalid
successful `clang` output with no assembly stdout and stderr diagnostics while
`llc` and `cc` are also available. That pins stdout validation as the reported
failure and keeps fallback recovery disabled for empty selected-backend
artifacts with stderr.
An `llc` whitespace-with-stderr precedence snapshot exposes invalid successful
`llc` output while the `cc -S` fallback is also available and `clang` is
unavailable. That pins the no-recovery boundary after invalid selected `llc`
output.
An `llc` empty-stdout precedence snapshot exposes invalid successful selected
`llc` output with no assembly stdout while the `cc -S` fallback is also
available. That pins the same no-recovery boundary for empty selected-`llc`
artifacts.
An `llc` empty-stdout-with-stderr precedence snapshot covers the same boundary
when selected `llc` writes stderr diagnostics but emits no assembly stdout
while `cc` is available.
Whitespace-with-stderr fallback snapshots expose deterministic fake `llc` and
`cc` tools with the same invalid successful-output behavior. That pins the same
stdout-validation precedence after LLVM fallback selection and after the
`cc -S` C fallback selection.
A selected-backend input-validation snapshot exposes a deterministic fake
`clang` that validates representative generated LLVM IR markers on stdin before
emitting normalized assembly. That pins the selected-backend stdin handoff for
the current lowerable scalar subset without treating the test double as
backend-specific IR validation.
Fallback input-validation snapshots expose deterministic fake `llc` and `cc`
tools that validate representative generated LLVM IR or generated C fallback
markers on stdin before emitting normalized assembly. That pins the same stdin
handoff after LLVM backend fallback selection and after the `cc -S` C fallback
selection without treating the test doubles as backend-specific IR/C
validation.
Backend argument-validation snapshots expose deterministic fake `clang`,
`llc`, and `cc` tools that validate the exact selected-backend and fallback
assembly emission argument vectors before accepting stdin and emitting
normalized assembly. That pins the current command-line contract without
treating it as full backend-specific CLI compatibility.
Backend discovery probe argument-validation snapshots expose deterministic fake
`clang`, `llc`, and `cc` tools that require an exact single-argument
`--version` probe before selected or fallback assembly emission. That pins the
current discovery probe contract without treating it as full backend-specific
discovery semantics.
Backend discovery probe output snapshots expose deterministic fake `clang`,
`llc`, and `cc` tools whose successful `--version` probes write stdout and
stderr diagnostics before selected or fallback assembly emission. That pins
the current boundary that probe output is ignored after a successful probe,
without treating that as full backend-specific discovery output semantics.
Failed backend discovery probe output snapshots expose deterministic fake
`clang`, `llc`, and `cc` tools whose failed `--version` probes write stdout and
stderr diagnostics before fallback selection or missing-backend reporting.
That pins the current boundary that failed-probe output is ignored and failed
probes still behave like unavailable tools, without treating that as full
backend-specific failed-probe output semantics.
Backend discovery probe start-failure snapshots expose deterministic fake
`clang`, `llc`, and `cc` command names that exist on `PATH` but cannot be
started for `--version`. That pins the current boundary that probe start
failures are treated as unavailable before fallback selection or
missing-backend reporting, without treating that as full backend-specific
discovery semantics.
Backend discovery probe permission-denied snapshots expose deterministic fake
`clang`, `llc`, and `cc` command names that exist on `PATH` but are not
executable for `--version`. That pins the current boundary that permission
denied probe starts are treated as unavailable before fallback selection or
missing-backend reporting, without treating that as full backend-specific
discovery semantics.
A selected-backend start-failure snapshot exposes a deterministic fake `clang`
that passes discovery and then rewrites itself to use a missing interpreter
before assembly emission. That pins the current diagnostic for a race-like
case where a discovered command cannot be started later, without treating that
as full backend race-condition recovery semantics.
A selected-backend permission-denied emission snapshot exposes a deterministic
fake `clang` that passes discovery and then removes its own execute permission
before assembly emission. That pins the same selected-backend start diagnostic
for permission-denied starts after discovery, without treating that as full
backend race-condition recovery semantics.
Fallback start-failure snapshots expose deterministic fake `llc` and `cc`
tools with the same race-like behavior. Those pin the current diagnostics for
previously discovered fallback commands that cannot be started later, without
treating that as full backend race-condition recovery semantics.
Fallback permission-denied emission snapshots expose deterministic fake `llc`
and `cc` tools that pass discovery and then remove their own execute
permission before assembly emission. Those pin the same fallback backend start
diagnostics for permission-denied starts after discovery, including the
current no-recovery boundary where a selected `llc` start failure is reported
without falling through to the `cc -S` C fallback.
A mixed scalar output snapshot uses deterministic fake `clang` coverage for a
lowerable straight-line program with both `echo` and `print`. A matching C
fallback snapshot hides LLVM assembly tools and uses a deterministic fake `cc`
that validates generated C fallback markers for that same static scalar output
boundary. These snapshots do not claim runtime-backed output conversion,
linking, execution, or broader native lowering.
A scalar reassignment assembly snapshot uses a deterministic fake `clang` that
validates generated LLVM IR for a straight-line program where later direct
static-variable assignments overwrite earlier lowerable scalar values before
output. This pins the current per-lowering-pass overwrite behavior without
claiming native symbol-table storage, references/copy-on-write behavior,
linking, execution, exact native PHP errors, or broader native lowering.
A matching C fallback reassignment snapshot hides LLVM assembly tools and uses
a deterministic fake `cc` that validates generated C fallback source for the
same straight-line overwrite boundary.
A matching `--emit-ir` reassignment snapshot commits the exact generated LLVM
IR and shows only the final overwritten scalar values are emitted.
A selected-`clang` type-introspection snapshot validates that the already
implemented native `is_callable(...)` false-folding IR is passed to the chosen
backend through stdin without broadening callable dispatch or native execution.
A selected-`clang` `function_exists($name)` snapshot validates the same stdin
handoff for already-implemented direct function-existence folding without
broadening runtime lookup, callable dispatch, or native execution.
A selected-`clang` `empty($name)` snapshot validates the same stdin handoff for
already-implemented direct-variable emptiness folding without broadening
symbol-table semantics, unset interactions, or native execution.
A selected-`clang` `isset($name)` snapshot validates the same stdin handoff for
already-implemented direct-variable existence folding without broadening
symbol-table semantics, unset interactions, or native execution.
A selected-`clang` `is_numeric($value)` snapshot validates the same stdin
handoff for already-implemented deterministic scalar/string numeric folding
without broadening runtime lookup, string coercion, or native execution.
A selected-`clang` `is_countable($value)`/`is_iterable($value)` snapshot
validates the same stdin handoff for already-implemented scalar/null
false-folding without broadening native array/object lowering or runtime-backed
type checks.
A selected-`clang` `is_object($value)`/`get_debug_type($value)` snapshot
validates the same stdin handoff for already-implemented scalar/null folding
without broadening native object lowering, object handles, or runtime-backed
type checks.
A selected-`clang` static metadata-exists snapshot validates the same stdin
handoff for already-implemented absent-class/interface/trait/enum false-folding
without adding native class metadata, autoloading, or object execution.
A selected-`clang` `strlen($value)` snapshot validates the same stdin handoff
for already-implemented known-string length folding without broadening string
coercion, dynamic calls, runtime lookup, or native execution.
A native `defined($name)` snapshot commits folded `--emit-ir` output and
normalized C fallback `--emit-asm` output for direct supported string names,
without adding native constant values, source-order definitions, runtime
constant tables, or native execution.
The Milestone 569 native `defined($name)` snapshot adds the current
`SORT_REGULAR` built-in constant to that static answer table without changing
the broader native constant-table or dynamic-call boundaries.
The Milestone 573 native `defined($name)` snapshot adds `SORT_NUMERIC` to the
same static answer table with folded `--emit-ir` and normalized C fallback
`--emit-asm` coverage while preserving those broader boundaries.
A selected-`clang` `defined("SORT_REGULAR")` snapshot validates that the same
folded LLVM IR reaches the chosen backend through stdin without changing
production lowering behavior.
A selected-`clang` `defined("SORT_NUMERIC")` snapshot validates the same
backend stdin handoff for the existing folded built-in constant answer without
changing production lowering behavior.
A selected-`clang` `defined("SORT_STRING")` snapshot validates the same
backend stdin handoff for the existing folded built-in constant answer without
changing production lowering behavior.
A selected-`clang` broader `defined($name)` constants snapshot validates the
same backend stdin handoff for the current exact built-in constant answer table
without changing production lowering behavior.
A backend-precedence snapshot exposes deterministic fake `clang`, `llc`, and
`cc` commands together. It pins the current selection order by proving
successful `clang` assembly emission is used before LLVM or C fallback tools
when all candidates are available, without treating that as full
backend-specific discovery semantics.
A fallback-precedence snapshot hides `clang` while exposing deterministic fake
`llc` and `cc` commands together. It pins the current fallback selection order
by proving successful `llc` assembly emission is used before the `cc -S` C
fallback when both fallback candidates are available, without treating that as
full backend-specific discovery semantics.
A selected-backend failure-precedence snapshot exposes deterministic fake
`clang`, `llc`, and `cc` commands together. It pins the current no-fallback
failure boundary by proving a selected `clang` emission failure is reported
directly instead of silently falling through to `llc` or `cc`, without treating
that as full backend recovery semantics.
A fallback failure-precedence snapshot hides `clang` while exposing
deterministic fake `llc` and `cc` commands together. It pins the same
no-recovery boundary after fallback selection by proving a selected `llc`
emission failure is reported directly instead of silently falling through to
the `cc -S` C fallback, without treating that as full backend recovery
semantics.
An empty-stderr fallback failure-precedence snapshot covers the same boundary
when selected `llc` exits nonzero without diagnostics while `cc` is also
available. It pins the stable empty-stderr `llc` diagnostic as final rather
than falling through to the `cc -S` C fallback.
An empty-stderr selected-backend failure-precedence snapshot covers the same
no-recovery boundary when selected `clang` exits nonzero without diagnostics
while `llc` and `cc` are also available. It pins the stable empty-stderr
`clang` diagnostic as final rather than falling through to fallback tools.
A selected-backend start-failure-precedence snapshot covers the same
no-recovery boundary when selected `clang` passes discovery but cannot be
started while `llc` and `cc` are also available. It pins the stable
selected-backend start diagnostic as final rather than falling through to
fallback tools.
A fallback start-failure-precedence snapshot covers the same no-recovery
boundary after fallback selection when `clang` is unavailable and selected
`llc` passes discovery but cannot be started while `cc` is also available. It
pins the stable `llc` start diagnostic as final rather than falling through to
the `cc -S` C fallback.

Current native lowering accepts same-type `null`, boolean, integer, finite
float, known ASCII nonnumeric NUL-free string loose/ordering comparisons, and
identical string-pointer self-comparisons only after both operands are
lowerable by the current straight-line scalar subset. Other
loose, ordering, mixed-type, array, object, and coercive comparison cases
still reject at the comparison boundary.
That keeps generated code from implying PHP comparison coercions, array/object
comparison behavior, untracked `NAN`/`INF` edge cases, or exact native error
objects that only the interpreter path currently handles or diagnoses.
The remaining lowerable native subset is intentionally tiny: literal scalar
values, direct static-variable assignments from those values, direct reads of
previously assigned static variables, later direct static-variable assignments
overwriting earlier lowerable scalar values in the same straight-line lowering
pass, lowerable same-type integer or same-type float `+`, `-`, and `*`
expressions, lowerable same-type `null`, boolean, integer, finite float, known
ASCII nonnumeric NUL-free string loose/ordering comparisons, identical string
pointer self-comparisons, empty-string concatenation identity for lowerable
string operands, direct supported-string-name `defined($name)` checks against
the current exact built-in constant-name set, non-string `echo`/`print`
through static `printf` calls, and statement-form `echo`/`print` of direct
compile-time string values through the native string/value stdout helper ABI.
It does not
model PHP zvals, symbol-table storage, PHP numeric coercion,
references/copy-on-write, broad dynamic string-pointer output,
locale/version-specific float formatting, integer overflow promotion, assembly
linking/execution, or native PHP error objects. The native runtime ABI probe has
diagnostic-handle calls, one deterministic failure branch for string-to-value
conversion failures, a first nullable empty-array handle allocation/length/free
probe, and a null-only request-state handle probe. Normal generated LLVM for the
documented string-output slices branches on that helper's nullable value-handle
result and reports the diagnostic message to stderr on the failure path before
cleanup. The empty-array handle is ABI evidence only: generated PHP array
literals, array reads/writes, key normalization, references/COW, and non-empty
array storage still reject before native lowering. Request-state handles are ABI
shape only; native superglobal storage/population still rejects at the
request-state boundary.
Finite same-type float arithmetic and finite float unary-minus results are
bounded and tracked only for later scalar folds such as strict identity when
all possible results are proven. Float overflow, `INF`, and `NAN`
result-tracking edges remain out of scope for this value-set tracker.
A dedicated mixed `echo`/`print` assembly CLI snapshot pins that boundary with
a deterministic fake backend and a lowerable scalar fixture; it is coverage of
the existing static output path plus the narrow direct string `print` runtime
helper handoff, not broader native runtime output support.
Native reads of variables that have not been statically assigned earlier in
that same straight-line lowerer are rejected with a specific codegen
diagnostic until native symbol-table storage, undefined-variable diagnostics,
references/copy-on-write behavior, and exact native error behavior exist.
Native comparison lowering accepts same-type `null`, boolean, integer, finite
float, known ASCII nonnumeric NUL-free string loose/ordering comparisons, and
identical string-pointer self-comparisons for `==`, `!=`, `<`, `<=`, `>`, and
`>=`, plus strict identity `===` and `!==` when both operands are already
lowerable `null`, integers, booleans, floats, or strings in the straight-line
subset. Static
same-type scalar identity folds at compile time. Bounded integer, float,
string, and boolean identity fold when all
possible `===`/`!==` outcomes are proven identical. Identical lowerable
dynamic scalar operands fold for integers, booleans, already-lowerable string
pointers, and finite tracked floats, so `$x === $x` and `$x !== $x` avoid
runtime comparisons in those safe scalar cases. Identical lowerable integer
operands also fold for loose/ordering comparisons, including intentionally
untracked integer expressions such as overflow-sensitive shift results:
`$x == $x`, `$x <= $x`, and `$x >= $x` fold true, while `$x != $x`, `$x < $x`,
and `$x > $x` fold false. Dynamic boolean expression
operands compared with boolean literals fold for `$flag === true`, `true ===
$flag`, `$flag !== false`, and `false !== $flag` by reusing the original
native boolean expression, and inverse forms such as `$flag === false`, `false
=== $flag`, `$flag !== true`, and `true !== $flag` use the native boolean
inversion path. Dynamic boolean expression operands compared loosely with
boolean literals fold for `$flag == true`, `true == $flag`, `$flag != false`,
and `false != $flag` by reusing the native boolean expression, while inverse
forms such as `$flag == false`, `false == $flag`, `$flag != true`, and
`true != $flag` use the native boolean inversion path. Dynamic boolean
expression operands ordered against boolean literals also fold within boolean
semantics, reusing the expression, inverting it, or folding to a static boolean
for cases such as `$flag > false`, `$flag < true`, `$flag <= true`, and
`true >= $flag`. Same-type integer and finite-float loose/ordering
comparisons whose tracked possible operands prove one result fold to a static
boolean. Literal-only comparisons still fold, while ambiguous tracked
finite-float comparisons stay emitted as native comparisons.
Boolean expression comparisons whose tracked possible operands prove one
loose/ordering result also fold to that static boolean without emitting a
redundant native boolean comparison. Identical native boolean expression
operands also fold for loose/ordering comparisons, including ambiguous boolean
expressions: `$flag == $flag`, `$flag <= $flag`, and `$flag >= $flag` fold
true, while `$flag != $flag`, `$flag < $flag`, and `$flag > $flag` fold false.
Other ambiguous boolean expression comparisons stay emitted. Identical native
string pointer operands also fold for loose/ordering comparisons, including
untracked string pointer expressions whose possible value set exceeds the
current small tracker: `$text == $text`, `$text <= $text`, and `$text >=
$text` fold true, while `$text != $text`, `$text < $text`, and `$text >
$text` fold false. Non-identical unknown string comparisons stay rejected.
Statically known integer
strict-identity comparison results remain tracked for later boolean scalar
lowering even when the comparison itself stays emitted as `icmp`. Same-type
ambiguous dynamic integer and boolean identity lower to `icmp`, same-type
ambiguous dynamic float identity lowers to `fcmp`, and same-type ambiguous
already-lowerable string pointer identity lowers through `strcmp`. Those
dynamic scalar comparisons use PHP-shaped boolean echo output. Already
lowerable mixed scalar operands with different PHP scalar types fold without
runtime comparison calls, including dynamic integer, boolean, float, or string
pointer expression results when the opposite operand has a different static
scalar type. Known ASCII nonnumeric string loose/ordering comparisons lower
to a static boolean when every possible safe string outcome matches; ambiguous
safe string loose/ordering comparisons lower through `strcmp`. Statically
known boolean, integer, and finite-float loose/ordering comparison results
remain tracked for later boolean scalar lowering even when the comparison
itself stays emitted as `icmp`/`fcmp`; ambiguous bounded boolean,
finite-float, or string loose/ordering comparison results remain dynamic and
untracked.
It rejects ambiguous bounded integer, float, string, or boolean
identity, broader value-correlation proofs across related expressions such as
`$x` and `!$x`,
numeric-looking, non-identical unknown, non-ASCII, or NUL-containing string loose/ordering comparisons,
mixed null or other mixed-type comparisons, untracked or
non-finite float comparisons, dynamic null identity beyond static/type-only folds, PHP
truthiness conversion for loose logical operands, arrays, objects,
non-lowerable float sources, and dynamic string allocation beyond the static
straight-line subset until generated code has PHP comparison coercions,
non-scalar comparison behavior, references/copy-on-write side-effect behavior,
and exact native error behavior.
Native lowering accepts unary minus only when the operand is already a
lowerable integer or float, and logical not when the operand is already a
lowerable boolean or a native boolean expression result, when the operand is
`null`, or when a known integer, finite float, or string operand has one statically known PHP
truthiness result, in the straight-line subset. Dynamic boolean double
logical-not expressions such as `!!$flag` reuse the original native boolean
expression instead of emitting redundant inversions. Native lowering folds
double logical-not over known scalar operands such as integers, finite floats,
strings, and `null` through the same known-truthiness subset without emitting
boolean operations. Native lowering folds
logical not over single-result statically known native boolean expression
operands to the known boolean result in LLVM IR and in the C assembly fallback
when the C boolean expression has a tracked result. Known numeric logical-not
folds to a static boolean for zero and nonzero known integer/finite-float
operands when all possible values have the same truthiness. Known string
logical-not folds to a static boolean for `""`, `"0"`, and known-truthy string
operands when all possible string values have the same truthiness. Null
logical-not folds to `true` without claiming broader null truthiness beyond
the documented logical binary folding subset. Integer unary-minus results remain
statically tracked for later
checked integer arithmetic when all bounded possible negation results are
proven not to overflow; single-result statically known integer operands fold
to the known negated result without a redundant native unary-minus operation.
Finite float unary-minus results remain tracked for later strict-identity
folding when every possible negation result is proven; single-result
statically known nonzero finite float operands fold to the known negated
result without a redundant native unary-minus operation.
It rejects boolean, string, null, array, and object unary-minus operands until
generated code has PHP numeric coercion. It rejects ambiguous numeric or
string logical-not truthiness, untracked numeric/string logical-not
expressions, non-finite float logical-not truthiness, null truthiness outside
logical-not, other truthiness conversion, unary integer overflow behavior,
references/copy-on-write side-effect behavior, and exact native error
behavior.
Native logical lowering accepts `&&`, `||`, `and`, `or`, and `xor` only when
both operands are already lowerable booleans or native boolean expression
results, or when both already-lowerable scalar operands have one statically
known PHP truthiness result, in the straight-line subset. Static boolean pairs
fold, and static boolean identity and annihilator edges such as `true ||
$flag`, `false && $flag`, `$flag && true`, and `$flag xor false` preserve
proven boolean results for later scalar lowering. Identical native boolean
expression operands for `&&`/`and` and `||`/`or` reuse the existing expression
without a redundant native boolean operation, and identical native boolean
expression operands for `xor` fold to `false`. Native boolean expression
operations whose tracked possible operands prove one result fold to that
static boolean without a redundant native boolean operation. Known scalar
logical operands whose null, integer, finite-float, or string truthiness is
unambiguous fold to a static boolean result without emitting a native boolean
operation. Statically decisive known-left `&&`/`and` and `||`/`or`
short-circuit cases such as `false && rhs` and `true || rhs` lower without
lowering the skipped right-hand operand. The generated-C executable path also
uses scoped RHS branches for dynamic `&&`/`and` and `||`/`or` operands that
lower through native truthiness/value-result materialization, rejecting RHS
branches that need persistent state merging. Other dynamic boolean expressions
lower to native boolean operations with PHP-shaped boolean echo output. It
rejects general PHP truthiness conversion beyond the current native value
subset, `xor` right-hand skipping, selected/evaluated unsupported right-hand
operands, ambiguous scalar truthiness, untracked scalar logical operands,
non-finite float truthiness, null coalescing, arrays, objects,
references/copy-on-write side-effect behavior, exact native error behavior, and
LLVM/assembly parity.
Native lowering accepts binary `+`, `-`, and `*` when both operands are
already same-type lowerable floats, or when both operands are lowerable
integers and the integer result is statically proven not to overflow, in the
straight-line subset. It also accepts integer `%` when the divisor is a
statically known positive integer, and statically known modulo results remain
tracked for later checked integer arithmetic. Tracked integer expression
operands and integer literal operands for `$x % 1` fold to zero, and bounded
tracked integer expression operands whose possible values all produce the same
remainder for a positive literal divisor fold to that remainder. Integer
modulo by one also folds after both operands lower when the dividend is
intentionally untracked, such as an overflow-sensitive shift result; other
modulo cases still require a statically known positive divisor and keep the
documented runtime-check boundary. Identical tracked integer
expression operands and identical integer literal operands for `-` fold to
zero without emitting a redundant native subtraction, and identical tracked
finite float expression operands and identical finite float literals for `-`
fold to `0.0` without emitting a redundant native subtraction. Identical integer
subtraction also folds after both operands lower when the value is
intentionally untracked, such as overflow-sensitive shift results; other
non-identity arithmetic with such values still rejects because exact overflow
tracking is unavailable. Tracked integer
expression operands and integer literal operands for `$x + 0`, `0 + $x`, and
`$x - 0` reuse the existing value, and tracked integer expression operands and
integer literal operands for `$x * 1` and `1 * $x` also reuse the existing
value. Tracked integer expression operands and integer literal operands for
`$x * 0` and `0 * $x` fold to zero. The `+ 0`, `- 0`, `* 1`, and `* 0`
identity or annihilator forms also fold after both operands lower when the
other integer operand is intentionally untracked, such as overflow-sensitive
shift results; non-identity arithmetic with such values still rejects because
exact overflow tracking is unavailable. Tracked single-result integer expression
arithmetic for `+`, `-`, and `*` folds to the known integer literal after
checked overflow analysis when tracked possible integer operands prove one
result. Literal-only integer arithmetic and ambiguous tracked-expression plus
tracked-expression integer arithmetic stay emitted. Tracked finite float
expression operands and finite float literals for nonzero `$x + 0.0`, `0.0 + $x`, and
`$x - 0.0`, and for `$x * 1.0` and `1.0 * $x`, reuse the existing expression.
Single-result statically known nonzero finite `0.0 - $x` folds to the known
negated float literal. Tracked finite positive float expression operands and
finite positive float literals for `$x * 0.0` and `0.0 * $x` fold to positive
`0.0`. Single-result statically known nonzero finite `$x * -1.0` and
`-1.0 * $x` fold to the known negated float literal. Tracked finite nonzero
float expression arithmetic for `+`, `-`, and `*` folds to the known float
literal when tracked possible finite-float operands prove one nonzero result.
Literal-only float arithmetic, zero-result arithmetic, possible signed-zero,
negative, and non-finite float identity/subtraction or multiplication-by-zero
cases, and signed-zero-sensitive multiplication by `-1.0`, stay emitted or
rejected rather than being folded.
It rejects mixed
int/float arithmetic, strings, booleans, nulls, arrays, objects, `/`,
overflow-sensitive or not-statically-proven integer arithmetic, dynamic or
non-positive modulo divisors, modulo results that are not statically known
enough for later checked arithmetic, and modulo cases that need PHP coercion
or runtime checks until generated code has PHP numeric coercion, dynamic
division/modulo zero checks, modulo coercions, PHP integer overflow promotion,
references/copy-on-write side-effect behavior, and exact native error
behavior. Mixed int/float `+`, `-`, and `*` operands use a
mixed-numeric-specific codegen rejection until
generated code has PHP numeric promotion and exact result typing. Boolean,
null, and string operands in `+`, `-`, and `*` use a scalar-coercion-specific
codegen rejection until generated code has PHP numeric coercion and string
numeric parsing. Overflow-sensitive or not-statically-proven integer `+`, `-`,
and `*` cases use an integer-overflow-specific codegen rejection until
generated code has PHP integer overflow promotion and runtime checks. Native
`/` uses a division-specific codegen rejection until generated code has PHP
division semantics, runtime zero checks, and no misleading integer truncation.
Dynamic, zero, or non-positive
integer modulo divisors use a modulo-specific codegen rejection so this runtime
check boundary stays distinct from generic unsupported arithmetic operands.
Native lowering accepts string concatenation only when both operands are
already lowerable strings in the straight-line subset, including ternary
operands that prove one static string result, folding the result into a
generated static string constant. Empty-string concatenation identity also
folds for already-lowerable string operands, including untracked string pointer
expressions: `$text . ""` and `"" . $text` reuse `$text` without runtime
string allocation. It rejects PHP scalar-to-string conversion, non-empty
ambiguous string expressions, arrays, objects, resources, runtime string
allocation, references/copy-on-write side-effect behavior, and exact native
error behavior. Double-quoted interpolation remains a separate native codegen
boundary rather than being lowered as string concatenation.
Native lowering accepts `&&`, `||`, `and`, `or`, and `xor` only when operands
are already lowerable booleans or native boolean expression results, or when
already-lowerable scalar operands have one statically known PHP truthiness
result, in the straight-line subset. Static boolean pairs fold at compile time,
known scalar truthiness folds to a static boolean, and dynamic boolean
expressions lower to native boolean operations plus PHP-shaped boolean echo
output. Statically decisive known-left `&&`/`and` and `||`/`or` cases such as
`false && rhs` and `true || rhs` lower without lowering the skipped right-hand
operand. The generated-C executable path additionally emits scoped RHS
branches for dynamic `&&`/`and` and `||`/`or` operands that use the native
truthiness/value-result boundary and do not change persistent compiler state.
The native lowerer still rejects cases that need general PHP truthiness
conversion beyond the current native value subset, `xor` right-hand skipping,
selected/evaluated unsupported right-hand operands, or RHS state merges.
Ambiguous scalar truthiness, untracked scalar logical operands, non-finite
float truthiness, null coalescing, arrays, objects,
references/copy-on-write side-effect behavior, exact native error behavior,
LLVM/assembly parity, and broader native lowering remain unsupported.
Native lowering accepts integer bitwise `&`, `|`, `^`, and unary `~` only when
operands are already lowerable integers in the straight-line subset. Bounded
statically known integer bitwise and unary bitwise-not results remain tracked
for later checked integer arithmetic. Single-result statically known integer
operands for unary `~` fold to the known bitwise-not result without a
redundant native bitwise-not operation. Double unary bitwise-not `~~$x` over
an already-lowerable integer operand reuses `$x`, including intentionally
untracked integer expressions such as overflow-sensitive shift results.
Identical tracked integer expression
operands and identical integer literal operands for `&` and `|` reuse the
existing value, and identical tracked integer expression operands and
identical integer literal operands for `^` fold to zero. Identical integer
operands also fold after both operands lower when the value is intentionally
untracked, such as overflow-sensitive shift results: `$x & $x` and `$x | $x`
reuse `$x`, while `$x ^ $x` folds to zero. Tracked integer
expression operands and integer literal operands for `$x & -1` and `-1 & $x`,
and for `$x | 0`, `0 | $x`, `$x ^ 0`, and `0 ^ $x`, reuse the existing value.
Tracked integer expression operands and integer literal operands for
`$x & 0` and `0 & $x` fold to zero. Tracked integer expression operands and
integer literal operands for `$x | -1` and `-1 | $x` fold to `-1` after both
operands lower. Single-known integer operands for `$x ^ -1` and `-1 ^ $x`
fold to the known bitwise-not result. The `& 0`, `& -1`, `| 0`, and `^ 0`
identity or annihilator forms also fold after both operands lower when the
other integer operand is intentionally untracked, such as overflow-sensitive
shift results. Tracked integer expression bitwise
operations for `&`, `|`, and `^` fold to the known integer literal when
tracked possible integer operands prove one result. Literal-only integer
bitwise operations and ambiguous tracked-expression plus tracked-expression
bitwise operations stay emitted. It accepts integer shifts
`<<` and `>>` only when the left operand is already a lowerable integer and the
right operand is a literal count or tracked integer expression count that
proves one value from 0 through 63; right shifts use arithmetic shift for
signed integer results. Tracked integer expression operands and integer
literal operands for `$x << 0` and `$x >> 0` reuse the existing value. Tracked
shift-by-zero identities also fold after both operands lower when the left
integer operand is intentionally untracked, such as an overflow-sensitive shift
result. Tracked single-result integer expression shifts with static safe
nonzero counts fold to the known integer literal. Literal-only shifts and
non-single tracked integer shifts stay emitted. Bounded statically known safe shift results remain
tracked for later checked integer arithmetic; overflow-sensitive left-shift
result sets remain unknown so later arithmetic rejects them instead of
implying PHP overflow semantics. It rejects ambiguous dynamic shift counts,
negative or large counts, PHP bytewise string bitwise operations, scalar-to-int coercion
for non-integer operands, arrays,
objects, references/copy-on-write side-effect behavior, exact native error
behavior, linking/execution, and broader native lowering.
Native lowering accepts full ternary `condition ? if_true : if_false` only
when the condition is already a lowerable boolean or native boolean expression
and both branch values are already lowerable integers, booleans, floats,
strings, or both branches are `null` in the same straight-line subset, or when
the condition is a statically known boolean and both branch values are already
lowerable scalar values, or when the condition and both branches are the same
direct variable whose current value is already lowerable. Dynamic mixed-type branch values are rejected until
native tagged values exist. Dynamic non-null ternaries emit LLVM `select` or
the corresponding C conditional expression, identical static string branches
fold to that string without a pointer select, identical boolean expression
branches fold to the reused expression without a redundant boolean select,
identical tracked integer expression branches and identical integer literal
branches fold to the reused value without a redundant integer select, and
identical integer branches also fold after both branches lower when the integer
value is intentionally untracked, such as an overflow-sensitive shift result.
Identical direct-variable full ternaries such as `$value ? $value : $value`
reuse the direct variable value without proving truthiness when all three
operands are the same already-lowerable direct variable, including untracked
integer, non-finite float-producing, string pointer, and boolean expressions,
plus null values.
Identical tracked float expression branches and identical float literal
branches fold to the reused value without a redundant float select, and
identical float branches also fold after both branches lower when the value is
intentionally untracked, such as a non-finite overflowing float multiplication.
Dynamic boolean literal branches fold without a boolean select for `$flag ? true : false`,
`$flag ? false : true`, `$flag ? true : true`, and
`$flag ? false : false`, dynamic `null`/`null` ternaries fold to `null`, and
static boolean ternaries fold to the selected branch value. Dynamic integer,
finite-float, and boolean ternaries whose possible branch values collapse to a
single known result fold to that scalar without a redundant select; ambiguous
same-type ternaries stay emitted. Full ternary conditions with null or with
single-known integer, finite-float, or known-string truthiness lower only the
selected already-lowerable branch; direct null-variable conditions select the
false branch without lowering unsupported true-branch calls. Dynamic boolean
full ternaries still require both branches to lower before selection.
Ambiguous integer, float, or string conditions, untracked string conditions,
non-finite float result tracking, and non-finite float conditions remain
rejected, and dynamic branch skipping for unsupported or side-effecting
branches remains unsupported. Dynamic integer ternaries and later checked integer
arithmetic track up to four statically known possible values; combinations
with more possible results remain unsupported. The lowerer still evaluates all
operands for dynamic branch selection,
so it rejects cases that need general PHP truthiness or dynamic lazy branch
evaluation to skip unsupported or side-effecting branches. Native short ternary `?:`
accepts lowerable boolean conditions in the same straight-line subset; dynamic
boolean forms require a lowerable boolean fallback, static-false forms return
any already-lowerable scalar fallback, and static-true forms fold to `true`
without lowering the fallback. Single-known integer conditions also fold
through integer truthiness: proven nonzero integer conditions reuse the integer
result, and proven zero integer conditions use the fallback. Single-known
finite float conditions fold through float truthiness the same way, with
proven nonzero finite floats reusing the float result and proven zero floats
using the fallback. Known string conditions fold through PHP string
truthiness when all possible values have the same truthiness: non-empty
strings except `"0"` reuse the string result, while `""` and `"0"` use the
fallback. Identical direct string-variable short ternaries such as `$text ?:
$text`, identical direct integer- or float-variable short ternaries such as
`$value ?: $value`, and identical direct boolean-variable short ternaries such
as `$flag ?: $flag` also reuse already-lowerable expressions without proving
broader truthiness, including untracked string pointer expressions, untracked
integer expressions, untracked non-finite float-producing expressions, and
boolean expressions. Null short ternaries use the fallback for `null ?:
fallback`, including direct null-variable fallback forms such as
`$value ?: $value`; broader null
truthiness in logical binaries or null coalescing remains unsupported.
Ambiguous string truthiness, non-identical untracked integer, float, or string expressions,
non-finite float truthiness, other non-boolean truthiness, null coalescing `??`,
null-aware variable/offset/property lookup, arrays, objects,
references/copy-on-write side-effect behavior, exact native error behavior,
linking/execution, and broader native lowering remain unsupported.
Native lowering statically folds direct `gettype`, `is_null`, `is_bool`,
`is_int`/`is_integer`/`is_long`, `is_float`/`is_double`, `is_string`,
`is_array`, `is_scalar`, and `is_numeric` calls when their single argument is
already in the straight-line native scalar/null subset. Native `is_numeric`
also folds literal and tracked string values only when the current
numeric-string grammar proves the result statically. Direct `is_countable` and
`is_iterable` calls fold to `false` for already-lowerable scalar/null/string
operands only, direct `is_object` calls fold to `false` for already-lowerable
scalar/null/string operands only, and direct scalar/null/string
`get_debug_type` calls fold to the current runtime type-name strings.
Interpreter `is_iterable()` also recognizes object metadata that records
`implements Iterator` or `implements IteratorAggregate` after concrete class
registration verifies the required public non-static methods with no required
parameters. Direct concrete `implements Traversable` remains a runtime
boundary until broader built-in engine interface inheritance semantics exist;
this is not native lowering or object iterator execution.
Direct `is_callable($value)` calls fold when `$value` is an already-lowerable
string value with a uniform known lookup result in the documented builtin
table, or when `$value` is an already-lowerable non-string scalar/null value,
which folds to false. Direct `is_callable($value, $syntax_only)` calls also
fold when `$value` is an already-lowerable string or non-string scalar/null
value and `$syntax_only` is an already-lowerable boolean; true syntax-only
flags return true for string values without name lookup, non-string
scalar/null values return false, while false flags use the documented builtin
lookup table for strings.
Direct `function_exists($name)` calls fold only when `$name` is an
already-lowerable string value with a uniform known answer in the documented
builtin table. Documented builtin names fold to true and missing names fold to
false; user-defined function tables, namespace/autoload-aware lookup,
extension-loaded functions outside the documented table, dynamic callees, and
runtime callable dispatch remain outside native lowering.
`dirname()` is currently an interpreter-only path builtin for lexical
Unix-style local paths. Native `function_exists("dirname")` and
`is_callable("dirname")` use the known-function table, but direct native
`dirname(...)` calls still reject under the function-call boundary until native
path policy and string-return lowering are proven.
`version_compare()` is likewise interpreter-only: `phpc run` supports a
bounded numeric-component version grammar for WordPress bootstrap guards, while
native `function_exists("version_compare")` can see the name through the known
function table and direct native calls still reject under the function-call
boundary.
`sprintf()` is an interpreter-only bounded string formatter for the current
WordPress bootstrap guard/message paths. It supports literal text, `%%`, `%s`,
and `%N$s` placeholders using runtime echo-string conversion for values. Native
`function_exists("sprintf")` and `is_callable("sprintf")` can see the name
through the known function table, but direct native calls still reject under
the function-call boundary until varargs/string formatting helpers are lowered.
`strtolower()` is an interpreter-only bounded case-mapping builtin for current
scalar/null string-convertible values. It applies ASCII lowercase mapping over
runtime UTF-8 strings so WordPress bootstrap can normalize simple option
suffixes, while locale-sensitive casing, full Unicode case folding, binary
string behavior beyond valid UTF-8, and native lowering remain out of scope.
`trim()` is an interpreter-only bounded string-normalization builtin for
current scalar/null string-convertible values. The first slice implements the
default PHP whitespace mask for represented runtime strings so WordPress can
parse shorthand INI values, while custom masks, binary/null-byte edge cases,
and native lowering remain out of scope.
`ltrim()` and `rtrim()` are interpreter-only bounded one-sided string
normalization builtins for the same scalar/null string-convertible values.
They support the default PHP whitespace mask and reached literal masks such as
`/`, while character-mask ranges, broad binary edge cases, object/resource
operands, exact diagnostics, and native lowering remain out of scope.
`str_contains()` is an interpreter-only bounded string-search builtin for
current scalar/null string-convertible haystack and needle values. It uses the
current UTF-8 runtime string representation, keeps PHP's empty-needle `true`
result, and leaves binary string edge cases and native lowering out of scope.
`str_starts_with()` is an interpreter-only bounded string-prefix builtin for
the same scalar/null string-convertible haystack and needle subset. It keeps
PHP's empty-needle `true` result for represented runtime strings, and leaves
binary string edge cases, object/resource coercions, exact diagnostics, and
native lowering out of scope. Direct native `str_starts_with(...)` calls stop
at a dedicated string-prefix codegen boundary before argument lowering or
backend selection, while native function-table introspection can still see the
known builtin name.
`str_ends_with()` is an interpreter-only bounded string-suffix builtin for the
same scalar/null string-convertible haystack and needle subset. It keeps PHP's
empty-needle `true` result for represented runtime strings, and leaves binary
string edge cases, object/resource coercions, exact diagnostics, and native
lowering out of scope. Direct native `str_ends_with(...)` calls stop at a
dedicated string-suffix codegen boundary before argument lowering or backend
selection, while native function-table introspection can still see the known
builtin name.
`basename()` is an interpreter-only bounded lexical path builtin for Unix-style
local path strings and an optional string suffix. It does not consult the
filesystem and leaves Windows drive/UNC paths, stream wrappers, null-byte
behavior, locale/codepage details, broad scalar coercions, exact diagnostics,
and native path lowering out of scope. Direct native `basename(...)` calls stop
at a dedicated path-basename codegen boundary before argument lowering or
backend selection, while native function-table introspection can still see the
known builtin name.
`substr()` is an interpreter-only bounded string-slicing builtin for current
scalar/null string-convertible inputs, integer offsets, and optional integer
lengths. It uses byte positions over represented runtime strings and rejects
slices that would produce invalid UTF-8, leaving PHP-exact binary string
behavior, broad offset/length coercions, object/resource operands, diagnostics,
and native lowering out of scope.
`min()` is an interpreter-only bounded integer helper for the current WordPress
memory-limit clamp. It accepts two or more integer arguments and returns the
smallest integer, while array-form calls, mixed-type comparison rules, and
native lowering remain out of scope.
`rand()` is an interpreter-only deterministic random boundary for the reached
WordPress `wpdb::placeholder_escape()` salt path. The current slice accepts no
arguments and returns a fixed integer so compatibility probes are reproducible;
PHP random-state compatibility, min/max forms, seeding, cryptographic
randomness, and native lowering remain out of scope.
`uniqid()` and `hash_hmac()` are interpreter-only deterministic hash
boundaries for the same placeholder-escape path. `uniqid()` returns a fixed
prefix-based ID, and `hash_hmac()` currently supports lowercase hex
HMAC-SHA256 through the `hmac` and `sha2` crates. Broader hash algorithms,
raw binary output, exact entropy/time behavior, and native lowering remain out
of scope.
`strcasecmp()` is an interpreter-only bounded string comparison builtin for
current scalar/null string-convertible values. It compares valid UTF-8 runtime
strings by bytes with ASCII case folding and returns only sign values. Native
function-table introspection recognizes the name, while direct native calls
still reject until string comparison helpers and diagnostics are lowered.
`str_replace()` is an interpreter-only bounded string replacement builtin for
scalar/null string-convertible search, replacement, and subject values. Native
function-table introspection recognizes the name, while direct native calls
still reject until string allocation, array forms, count-output references, and
diagnostics have a lowered runtime model.
`call_user_func()` is an interpreter-only bounded callable dispatcher for
string callbacks resolving to current user functions or documented callable
builtins, plus current ordinary closure values. For string user-functions and
closure callbacks with reached non-variadic by-reference parameters, it
follows PHP's direct `call_user_func()` behavior instead of the ordinary
direct-call binding path: arguments are evaluated by value, a bounded
`E_WARNING` is routed through the current error-handler stack or stderr
fallback for each reached by-reference parameter, and callee writes stay local
to that callback frame. It does not implement array callables, `__invoke`,
variadic reference parameters, variadic unpacking, exact warning text/object
behavior, or native lowering.
`call_user_func_array()` is a separate interpreter-only callable dispatcher for
string callbacks, current public array-callable shapes, integer-keyed
positional argument arrays, and the bounded string-keyed named argument slice
for current user callbacks. String user-function callbacks, public
`[object, method]` instance callbacks, and public `["ClassName", "method"]`
static callbacks can bind reached by-reference parameters from unkeyed or
integer-keyed literal argument-array elements such as `array(&$value)` and
`array(10 => &$value)`, or from string-keyed elements whose keys match
declared parameter names, using the same direct caller cell as ordinary
by-reference user-function and method calls. Direct public object-property
array-offset reference elements use the current copy-in/writeback bridge for
that callback path. Direct stored argument arrays whose reached slots were
assigned by reference through the covered direct array-offset target path use
the existing alias metadata as a copy-in/writeback bridge, including for
normal callback invocations of reference-returning user functions or public
array-callable methods that mutate reached by-reference parameters.
Direct variable, direct array-offset, direct append-offset, nested
append-offset, direct visible object-property, and direct visible
object-property append-offset assignments from reference array literals now add
the selected literal slot to the same bounded alias metadata used by direct
reference assignment, so `$args = array(&$value)`,
`$registry["args"] = array(&$value)`,
`$store->args = array("value" => &$object->items[$key])`,
`$args[] = array(&$value)`, `$registry["groups"][] = array(&$value)`, and
`$store->groups[] = array(&$items["slot"])` can be used later as stored
`call_user_func_array()` argument arrays. Append targets first materialize the
array value at the runtime append key and then record literal reference slots
below that key. If the assigned direct variable is already backed by covered
array-offset alias metadata, such as `$args =& $registry["args"]`, the literal
reference slots are recorded below that aliased target path. This is still
keyed alias metadata over materialized array values, not a runtime reference
container. Reference array literals assigned into dynamic-property append
targets, `ArrayAccess` append targets, or other non-variable targets,
reference elements from arbitrary expressions, stored arrays without
covered reference-assigned slots,
execution past unknown or duplicate string-keyed argument names, variadic
named callback arguments, positional arguments after string-keyed named
arguments, closure invocation, `__invoke`, broader
named-argument semantics, exact warning behavior, and native lowering remain
unsupported.
`implode()` is an interpreter-only bounded array-to-string builtin for current
WordPress bootstrap message paths. It joins scalar/null array values in
insertion order with either an empty default separator or a string separator.
Native function-table introspection recognizes the name, while direct native
calls reject until array iteration, string allocation, and conversion
diagnostics have a lowered runtime model.
`ob_start()`, `ob_get_level()`, `ob_get_contents()`, `ob_get_length()`,
`ob_list_handlers()`, `ob_get_status()`, `ob_get_clean()`, `ob_get_flush()`,
`ob_clean()`, `ob_flush()`, `ob_end_clean()`, and `ob_end_flush()` are
interpreter-only output-buffer boundaries for the current WordPress
request/rendering path. The interpreter keeps a stack of string buffers;
PHP-visible output appends to the innermost active buffer, `ob_get_contents()`
peeks at that buffer without closing it, `ob_get_length()` reports its current
byte length, `ob_list_handlers()` reports one default handler name per active
buffer, `ob_get_status()` reports bounded default-handler status arrays for
the innermost buffer or for the full active stack, `ob_get_clean()` pops and
returns that buffer, `ob_get_flush()` closes the innermost buffer while
returning and flushing its contents outward, `ob_clean()` clears the innermost
buffer, `ob_flush()` moves its contents outward while keeping it active,
`ob_end_clean()` closes and discards it, `ob_end_flush()` closes it and
flushes its contents outward, and remaining
buffers flush outward to stdout when execution completes or the bounded
`exit()` path returns. Native function-table introspection recognizes the
names, while direct native calls reject until generated code has stdout capture
buffers, shutdown flushing, output-started/header interaction, SAPI
integration, and exact diagnostics.
`register_shutdown_function()` is modeled as an interpreter-only request/SAPI
shutdown queue. Registration evaluates and stores the callback plus extra
arguments in request-local interpreter state. Normal completion and the
bounded `exit()` path drain supported string user/builtin callbacks and public
object/static array callables in registration order before object destructors
and final output-buffer flushing; callbacks registered while the queue is
draining are appended and reached later in the same shutdown pass. Closure
callbacks remain a registration-only boundary because current closure values
store capture metadata but not executable closure bodies. The model does not
claim PHP's full fatal-error, finally, destructor, by-reference argument,
invokable-object, or native shutdown semantics.
`header()` is an interpreter-only web/SAPI boundary for the current WordPress
bootstrap/request path. It validates the current string/bool/int argument
shape, records the raw header line in deterministic in-process CLI request
state, and returns `null`. For ordinary colon-delimited header lines, default
replacement removes earlier lines with the same ASCII-case-insensitive field
name before appending the new line; `$replace = false` keeps duplicate lines.
Before output starts, it also updates request-local status state from an
explicit non-zero integer response-code argument, from bounded `HTTP/... NNN`
status lines, and from PHP-compatible `Location:` defaulting to `302` unless
the current status is `201` or already a redirect. A zero response-code
argument is treated as no explicit status. After output starts, late
`header()` calls leave the log/status unchanged and route a bounded
`E_WARNING` through the current error-handler stack or stderr fallback.
`http_response_code()` reads or writes that request-local status state and
preserves PHP's current previous-value return shape for the covered integer
argument slice.
`headers_list()` returns that header log as an ordered array of strings. Native
function-table introspection recognizes the names, while direct native calls
reject through a header-state boundary until response storage, diagnostics,
output-started tracking, status handling, and SAPI integration have a lowered
runtime model.
`header_remove()` mutates that deterministic CLI header log for the current
bounded subset: no arguments clear the log, and one string removes entries
whose raw header line has the same ASCII-case-insensitive field name before
the first colon. It leaves the log and response status unchanged after
unbuffered output has started and routes a bounded `E_WARNING` through the
current error-handler stack or stderr fallback. It still does not model
status-header removal, whitespace normalization, exact PHP warning text, or
full SAPI removal behavior.
`setcookie()` and `setrawcookie()` are interpreter-only header-state
boundaries. The current slice accepts a non-empty string cookie name that does
not contain PHP's forbidden cookie-name separator bytes, plus optional string
value, bounded positional attributes, or the bounded options-array attribute
form. It formats nonzero expiration timestamps as GMT dates, appends a
deterministic `Set-Cookie:` line to the same CLI header log, adds a bounded
`Max-Age` attribute for nonzero expirations using the current host clock
(`Max-Age=0` for past expirations), and replaces earlier deterministic
`Set-Cookie` lines with the same cookie name plus normalized non-empty
path/domain identity while preserving same-name cookies for different
path/domain identities. Domain identity matching lowercases non-empty ASCII
domain text for replacement only; the emitted header preserves the
caller-provided domain text.
`setcookie()` percent-encodes the value; `setrawcookie()` preserves the raw
string value. Options-array calls match the bounded PHP option-key set
ASCII-case-insensitively, use the last inserted value when duplicate
differently cased documented keys are present, and reject numeric keys and
unknown string keys before changing the deterministic header log. After
unbuffered output starts
these calls return `false`, leave
the header log unchanged, and route a bounded `E_WARNING` through the current
error-handler stack or stderr fallback; cookie-name encoding,
exact request-time/Date-header parity for future `Max-Age` values,
exact `ValueError` objects/text for invalid names/options,
IDNA/trailing-dot/domain-policy
canonicalization, exact warning text, SAPI emission, and native lowering
remain outside the model.
`session_start()` uses the same request-local output-started state for the
current bounded session lifecycle. Before unbuffered output it materializes the
in-memory `$_SESSION` root from a PHP-compatible `sess_<id>` file when
`session.save_path` was set through `ini_set()` and the current id is bounded
to ASCII letters, digits, underscores, or hyphens. When that file exists but
the current bounded scalar/array session decoder cannot parse it,
`session_start()` routes one recoverable warning and recovers with an empty
session array. If an explicit non-empty id
falls outside that bounded file-safe subset, the start returns `false`, routes a
bounded `E_WARNING`, and does not mark the session active, append headers, or
materialize `$_SESSION`. Without a file it falls back to the request-local
snapshot keyed by the current session id, or to an empty array when no snapshot
exists, and marks the session active. A fresh
successful start appends a deterministic
`Set-Cookie: PHPSESSID=<id>` line to the same request-local CLI header log used
by `header()`, `setcookie()`, and `headers_list()`, replacing earlier
deterministic `PHPSESSID` cookie lines with the same normalized non-empty path
and ASCII-case-insensitive normalized non-empty domain identity while keeping
same-name cookies for different path/domain identities; the bounded `use_cookies`
session option suppresses that line when it is falsey. The bounded
`cookie_lifetime`, `cookie_path`, `cookie_domain`, `cookie_secure`,
`cookie_httponly`, and `cookie_samesite` options only format deterministic
attributes on that in-memory header log; they do not model cookie encoding,
expiration-date formatting beyond the bounded `Max-Age` attribute, broader
replacement policy, or host SAPI emission. A fresh
successful start also appends the bounded default no-cache session headers to
the same CLI header log: `Expires: Thu, 19 Nov 1981 08:52:00 GMT`,
`Cache-Control: no-store, no-cache, must-revalidate`, and `Pragma: no-cache`.
The request-local session cache configuration defaults to limiter `nocache`
and expiration `180`. `session_cache_limiter()` and `session_cache_expire()`
read or update those values before output or active session state. The empty
limiter string suppresses the session cache headers on the next fresh start,
and restoring `nocache` re-enables the deterministic no-cache trio. The
`private`, `private_no_expire`, and `public` variants format bounded
deterministic headers from the configured expiration minutes. Public
`Expires` uses the bounded request timestamp, seeded by `PHPC_REQUEST_TIME`
or the fixed epoch CLI fallback; `Last-Modified` uses the main source file's
filesystem mtime when available and falls back to that request timestamp.
Other stored limiter strings remain an explicit `session_start()`
unsupported-call boundary until broader limiter variants, real SAPI `Date`
emission, and host-webserver request-time initialization exist.
`session_write_close()` stores the active `$_SESSION` array
back into that request-local snapshot map and, when the bounded save-path/id
file slice is active, writes string-keyed scalar and array values back to
`sess_<id>` before closing the status. A later `session_start()` for the same
id replaces the visible root from the session file or last closed snapshot, so
edits made while the session was closed are not promoted to active data. The recognized
`read_and_close` option reloads the file or snapshot
and immediately closes the bounded status back to `PHP_SESSION_NONE` while
keeping the in-memory session array visible. After unbuffered output it
returns `false`, leaves session status/data unchanged, and routes a bounded
`E_WARNING` through the current error-handler stack or stderr fallback before
applying options. Session cache-header variants outside the documented
empty, `nocache`, `private`, `private_no_expire`, and `public` behavior,
exact request-time/script-mtime parity, cookie encoding, expiration-date formatting,
broader replacement policy, trans-sid behavior, locking, save handlers, garbage
collection, broader PHP session-id policy, integer top-level session
keys, object/resource session serialization, exact malformed-session recovery
parity, option effects beyond the documented session-start options, exact
warning text, reference aliases across `_SESSION` root replacement, and native
lowering remain outside the model.
`headers_sent()` is an interpreter-only web/SAPI boundary. The current
slice tracks the first non-empty write that reaches unbuffered stdout. Echo,
print, `exit("message")`, `var_dump()`, `print_r()`, and outermost
`ob_get_flush()`/`ob_flush()`/`ob_end_flush()` stamp that state with the current source filename
and source line; output held only inside active output buffers does not. Direct
variable, direct array-offset, direct object-property, and direct
object-property array-offset filename/line output arguments are written with
`""`/`0` before output starts and the stamped file/line after it starts.
Dynamic object-property output targets, callback-mediated output targets, exact
warning text, SAPI differences, shutdown-time buffer flushing visibility, and
native lowering are not modeled.
`php_sapi_name()` is an interpreter-only request/SAPI identity boundary that
returns the same deterministic `cli` string as `PHP_SAPI`. It intentionally
does not query the host PHP binary, web-server SAPI, CGI/FPM state, or native
runtime state.
Direct native reads, indexed reads, `isset(...)`, and `empty(...)` over the
current request superglobals `$_SERVER`, `$_COOKIE`, `$_GET`, `$_POST`,
`$_REQUEST`, and `$_FILES` reject through a request-state codegen boundary
until generated code has request storage beyond the null-only request-state ABI
handle, SAPI population, `variables_order` policy, upload metadata,
references/copy-on-write, and exact diagnostics. This avoids folding unassigned
request bags as ordinary missing native variables.
`abs()` is an interpreter-only bounded numeric builtin for current integer and
finite-float values. It is intentionally narrower than PHP coercion until
numeric string, bool/null coercion, overflow, NaN/infinity, and native runtime
numeric diagnostics are modeled.
`microtime(true)` is an interpreter-only host-clock boundary for bootstrap
timing checks. It returns a finite float seconds value from `SystemTime`, while
the string-return forms stay unsupported until time virtualization, formatting,
precision, monotonicity, INI/timezone policy, and native runtime calls are
designed.
`ini_get()` is an interpreter-only configuration boundary backed by a
deterministic compatibility registry rather than host php.ini. It is intended
to make bootstrap decisions reproducible while leaving mutable INI state,
`ini_get_all()`, SAPI policy, extension-owned option catalogs, and native
runtime integration explicit future work.
`ignore_user_abort()` is an interpreter-only SAPI/connection-state placeholder
for entry flows such as WordPress `wp-cron.php`. It stores one deterministic
boolean in the interpreter, returns the previous setting as PHP's `0`/`1`
integer shape, and deliberately does not observe host client disconnects,
web-server/FastCGI/LiteSpeed behavior, request finishing, cron spawning, or
native runtime state.
`mysqli_connect()` is a placeholder-handle boundary, not database support. The
runtime and native function tables expose the name so early application
extension guards can move to the next compatibility blocker, and direct or
dynamic connection attempts now return deterministic placeholder `mysqli`
objects after validating the current scalar/null argument subset. Host socket
connections, authentication, real database selection, query/result behavior,
errors, full escaping, charset state, and native database calls are still
future work.
The current placeholder MySQLi slice also supports deterministic
`mysqli_real_connect()`, `mysqli_get_server_info()`, `mysqli_query()` for the
reached SQL-mode probe, `mysqli_select_db()`, and
`mysqli_real_escape_string()` for scalar/null string-convertible values over
the placeholder handle. These calls are compatibility probes, not host DB
integration. The interpreter has one narrow `wp_options` state island for exact
WordPress-shaped option writes and reads, including direct option-value,
autoload, option-name/option-value, exact value/autoload updates, and bounded
full-row option-name/value/autoload result shapes with or without
deterministic placeholder option IDs, plus exact option-name equality and
option-name-list deletes for current option/transient cleanup probes, and
direct/prepared autoload equality reads for alloptions-shaped probes, plus
prepared autoload-only option-name equality reads for the current
`update_option()` autoload reevaluation probe, plus prepared option-value
equality reads with `LIMIT 1` for transient-shaped `wpdb::get_var()` probes,
plus one-shot `mysqli_execute_query()` prepared option insert/update/replace/delete
mutations and deterministic `mysqli_stmt_insert_id()` metadata for
prepared-statement option/transient-shaped state probes,
plus exact explicit full-row-with-id and
star-projection option-name equality reads with and without `LIMIT 1` for
object-row/result/column `wpdb` probes, plus
SQL-mode-aware direct option-name equality reads and equality/`IN` deletes
whose single-quoted option-name literals use the placeholder handle's bounded
`NO_BACKSLASH_ESCAPES` branch, plus
bounded direct
`option_name LIKE '<pattern>'` result scans with `%` wildcards, `_`
single-character wildcards, backslash escapes, and a bounded
single-character `ESCAPE '<char>'` clause, plus prepared `option_name LIKE ?`
prefix result scans with the same bounded single-character `ESCAPE '<char>'`
clause and prepared deletes for transient-shaped option rows. Those scan
result shapes also accept the exact trailing `ORDER BY option_name` /
backticked `ORDER BY` suffix with optional `ASC`, while preserving the
deterministic ascending option-name order already used by the state island. A
further
bounded transient-timeout scan accepts exact direct/prepared option-name
projections with `option_name LIKE '<prefix>%' AND option_value < timestamp`
or the prepared equivalent for rows whose recorded option value parses below a
decimal threshold; the matching exact direct/prepared transient-timeout delete
shape removes only those timeout rows and reports deterministic affected-row
metadata. The state island also accepts one exact WordPress-shaped
multi-table transient cleanup delete over `wp_options` aliases `a` and `b`,
where `a.option_name` matches a trailing-percent payload prefix, does not
match the matching timeout prefix, and `b.option_name` is the supported
`CONCAT( '<timeout_prefix>', SUBSTRING( a.option_name, <offset> ) )` timeout
row whose decimal value is below the direct or prepared threshold. That slice
deletes both reached payload and timeout rows and updates affected-row
metadata. It also exposes deterministic `SHOW TABLES LIKE 'wp_options'`,
`DESCRIBE`/`DESC wp_options`, and `SHOW [FULL] COLUMNS FROM wp_options`
result rows plus deterministic `SHOW INDEX`/`SHOW KEYS FROM wp_options` rows
for the current four-column option-table schema so bounded install/update
probes can inspect table existence, primary/unique key markers, fixed
primary/unique index rows, and autoload default/collation metadata. A separate
per-handle dynamic schema island records the current bounded
`CREATE TABLE`/`ALTER TABLE` WordPress/dbDelta shapes and answers deterministic
`SHOW TABLES LIKE`, `SHOW TABLE STATUS LIKE`, `SHOW TABLE STATUS WHERE Name`,
`DESCRIBE`/`DESC`, `SHOW [FULL] COLUMNS`, `SHOW CREATE TABLE`, and
`SHOW INDEX`/`SHOW KEYS` probes against that recorded shape, including bounded
column `DEFAULT`/`DEFAULT NULL`/`NOT NULL`/`auto_increment` metadata, inline
column primary/unique/non-unique key metadata, bounded `ASC`/`DESC`
index-part ordering metadata for `SHOW INDEX` collation values, and
deterministic `SHOW CREATE TABLE` text. `SHOW INDEX`/`SHOW KEYS` can also
filter the recorded rows by bounded `Key_name` equality or `Key_name LIKE`
predicates before result materialization. Metadata `LIKE`
filters support exact patterns plus `%` wildcards, `_` single-character
wildcards, backslash-escaped `%`, `_`, and `\` literals, and a bounded
single-character `ESCAPE '<char>'` clause for table names, table status rows,
column names, and index names. Direct literal
`SHOW TABLE STATUS WHERE Name IN ('table', ...)` probes over the same recorded
schema state validate non-empty identifier-shaped table-name lists, skip
missing names, and return rows in deterministic table-name order. A per-handle
accepted
`SET SESSION sql_mode='NO_BACKSLASH_ESCAPES'` toggles the bounded schema
metadata `LIKE` parser so implicit backslash escaping is disabled for later
metadata filters on that handle while explicit `ESCAPE '<char>'` clauses
still work. The prepared statement path routes one string filter parameter
from `mysqli_execute_query()` or `mysqli_stmt_execute(..., array(...))` into
that same recorded schema metadata for the covered `SHOW TABLES`, `SHOW TABLE
STATUS`, `SHOW COLUMNS`, and `SHOW INDEX`/`SHOW KEYS` equality/`LIKE` filter
forms, including exact `SHOW TABLE STATUS WHERE Name = ?` table-name probes
and bounded prepared `SHOW TABLE STATUS WHERE Name LIKE ?` metadata
predicates, plus bounded `SHOW TABLE STATUS WHERE Name IN (?, ...)`
table-name lists that validate each parameter as an identifier-shaped string
and return rows in deterministic table-name order. The same path accepts
bounded table-identifier placeholders for `SHOW [FULL] COLUMNS FROM ?` and
`SHOW INDEX`/`SHOW KEYS FROM ?`, including the documented optional
`Field`/`Key_name` equality or `LIKE` filter placeholder as the next
parameter. One exact prepared
`information_schema.COLUMNS c LEFT JOIN information_schema.STATISTICS s`
query also projects recorded column and index-part metadata as `Field`,
`Type`, `Null`, `Key`, `Key_name`, `Seq_in_index`, and `Sub_part` rows for a
single identifier-shaped `c.TABLE_NAME = ?` parameter; it does not substitute
arbitrary SQL parameters. The same placeholder
transaction and savepoint helpers that
snapshot the `wp_options` state island also snapshot
and restore this bounded dynamic schema-state island for recorded
`CREATE TABLE`/`ALTER TABLE` metadata. It does not model arbitrary
multi-table deletes, subqueries, schema DDL beyond the documented bounded
`CREATE TABLE`/`ALTER TABLE` shapes, dbDelta diffs,
charset/collation negotiation, locks, real index inspection
beyond recorded schema-state rows, expression indexes, opclass/parser metadata,
duplicate aliases, malformed `CONCAT`/`SUBSTRING`
forms, SQL-mode behavior beyond the bounded schema metadata
`NO_BACKSLASH_ESCAPES` parser branch and direct option-name literal equality
reads/deletes, option-name `LIKE` wildcard semantics beyond direct read
filters, prepared option-name prefix scans, prepared option-row `ESCAPE`
clauses, and schema metadata filters, exact MySQL
affected-row or insert-ID edge cases, prepared schema placeholders beyond the
documented single string filter parameter, table-identifier metadata probes,
table-status `IN` lists, and the exact joined `COLUMNS`/`STATISTICS`
metadata projection, real
transactional
DDL/isolation/locking, or WordPress cleanup against tables outside the
deterministic `wp_options` state island; it is not a general SQL engine,
schema model, host database connection, PDO layer, or native database runtime.
`spl_autoload_register()` is currently an interpreter-only bounded
registration path: it accepts closure expressions, string user-function
callbacks, public `"ClassName::method"` static-method string callbacks,
public `[object, "method"]` instance-method array callbacks, and public
`["ClassName", "method"]` static-method array callbacks, plus object
callbacks with a public non-static `__invoke($name)` method, with optional
boolean flags and returns true. Supported non-closure callbacks are stored in
registration order, the current boolean `prepend` flag can place a callback at
the front, and truthy-autoload
`class_exists()`/`interface_exists()`/`trait_exists()` misses invoke those
callbacks with the requested class/interface/trait name before rechecking
metadata. That lets local include/require paths register autoloaded
class/interface/trait declarations through the existing included-file
declaration loader, including missing direct trait `use` names reached while
registering an included class declaration. Closure callbacks remain accepted
registration metadata only and report a stable unsupported autoload boundary if
lookup needs to invoke them. Non-public/static `__invoke`, invokable-object
dispatch outside autoloading, non-public methods, class-string non-static
methods, object static methods, `self::`/`parent::`/`static::` callback
strings, and exact callable validation remain unsupported. The same stored
callback vector backs `spl_autoload_functions()`, which reifies the current
bounded callback shapes as PHP values in dispatch order, and
`spl_autoload_unregister()`, which removes the first matching bounded callback
or returns false for valid unregistered callbacks. `spl_autoload_call($class)`
manually dispatches the same bounded callback vector for a string
class/interface/trait name and stops once any class-like metadata with that
name exists. `spl_autoload_extensions()` is a separate request-local string
slot initialized to PHP's `.inc,.php` default; string arguments replace the
slot, `null` reads it without mutation, and the value is exposed for parity
with autoload lifecycle introspection. The default `spl_autoload()` callback
uses that string as a comma-separated extension registry, lowercases the
requested class-like name, maps namespace separators to local path separators,
resolves candidate files through the same bounded local include resolver, and
includes the first existing file once. Registering `"spl_autoload"` stores it
as an ordinary function callback, but dispatch handles it as this builtin
probe instead of requiring a user function body. `class_alias()` uses that
same callback path when its source class or interface is missing and the
current bool-like autoload flag is truthy; successful class aliases insert an
additional case-insensitive name into the class table that points at the
original class metadata, so alias instantiation still creates objects whose
declared class is the source class. Runtime object relationship metadata
records active class-alias lookup names for the instantiated class and its
ancestors, which lets the current simple typed-property compatibility check
accept alias type names when the alias exists before object instantiation.
Successful interface aliases insert an
additional case-insensitive interface lookup entry that points at the original
interface declaration; relationship checks canonicalize that alias back to the
original interface name, instantiated object metadata records active aliases
for implemented declared interfaces for the same typed-property slice, and
`get_declared_interfaces()` still walks only real interface declarations.
Stream/URL paths, exact PHP warnings,
scalar-to-string extension coercions, recursive edge cases beyond the current
same-name guard, enum autoload lookup, trait aliasing, alias entries in
`get_declared_classes()`/`get_declared_interfaces()`, and native autoload
lowering remain outside this slice. Native function-table introspection
recognizes the names, while direct native calls reject under the function-call
boundary.
`assert()` is currently an interpreter-only assertion builtin for truthy
bootstrap guards. It evaluates one or two arguments normally, accepts scalar or
null descriptions as inert metadata, returns true for truthy assertions, and
keeps failing assertions behind a stable runtime boundary. Native
function-table introspection recognizes the name, while direct native calls
reject under the function-call boundary until generated code has assertion
policy, callbacks/exceptions, exact diagnostics, and unwinding behavior.
Runtime-backed constant behavior currently lives in the interpreter's
`ConstantTable`. Unbracketed namespace-scoped top-level `const NAME = value;`
declarations store canonical qualified names such as
`Sodium\CRYPTO_AUTH_BYTES`. String-name `defined(...)` and `constant(...)`
lookups accept qualified names with an optional leading global namespace
separator and answer from that deterministic table. Runtime string lookups for
declared class constants use the interpreter class metadata instead:
`ClassName::CONST` and `\ClassName::CONST` are split at runtime,
`defined(...)` reports true only for public declared or inherited constants,
and `constant(...)` resolves through the same class-constant visibility checks
as direct `ClassName::CONST`. This does not model bare namespace constant
fallback reads, autoload-triggered class discovery, broader `self`/`parent`/
`static` string names, host extension loading, full extension constant
inventories, or native lowering.
Direct `defined($name)` calls include the deterministic `PHP_VERSION_ID`,
`PHP_VERSION`, and 64-bit `PHP_INT_MAX` compatibility-target constants in the
built-in answer table. Bare global constant reads and `constant($name)` still
stay behind the native global-constant boundary until generated code has a real
constant table and version-policy model.
Direct `extension_loaded($name)` calls with already-lowerable string names fold
against the current deterministic bounded compatibility registry: `json` and
`hash` fold to true, while other names fold to false. Native code does not
query host PHP modules, `php.ini`, SAPI state, or dynamic extension loading.
`file_exists()` is currently an interpreter-only local filesystem metadata
builtin for the WordPress bootstrap drop-in check. It accepts one string local
path, rejects stream-wrapper paths, and returns a boolean for host filesystem
metadata existence. Relative paths check the process path first and then the
repository root for committed source-map fixture paths; this does not establish
include-path lookup, canonicalization, stream support, stat-cache semantics,
open_basedir, exact warnings, or native filesystem lowering. Native
function-table introspection recognizes the name, while direct native calls
reject under the function-call boundary.
`file_get_contents()` is also interpreter-only in the current runtime. It maps
`php://input` to the same explicit `PHPC_REQUEST_BODY` request seed used by
the request-bag scaffolding and otherwise keeps a bounded local UTF-8 text file
read. Non-URL local paths use the same process-path-then-repo-root relative
path policy as the filesystem metadata builtins. Local absolute `file://` URLs
with an empty host or `localhost` map to the referenced local path after the
bounded UTF-8 path portion is percent-decoded.
The current bounded second argument accepts a bool include-path flag; when true
for relative local paths, lookup follows the same include-path-then-source-
relative candidate order used by current `include`/`require` resolution. The
third argument accepts a bounded
stream-context resource or
`null`, and the fourth/fifth arguments apply integer offset plus optional
non-negative max length over the current UTF-8 string payload. Missing local
file reads and negative offsets before the start of the current local or
`php://input` payload emit a bounded PHP-style `E_WARNING`, return `false`,
and continue. That warning can route through the current request-local
`set_error_handler()` stack when the top stored handler is a string
user-function callback or public object/static array callable whose mask
includes `E_WARNING`; `restore_error_handler()` pops that bounded stack so the
previous registration becomes active again. A `false` handler return falls
through to the stderr warning path when `error_reporting()` still includes
`E_WARNING`, while other return values suppress the fallback. This is
intentionally a local recovery
path, not the general PHP warning/error-handler system. It does not model PHP
binary strings, exact PHP warning or handler `errstr` text, non-local
`file://` hosts, malformed percent escapes, decoded NUL bytes, non-UTF-8
percent-decoded paths, stream context
effects, wrapper-specific context behavior, exact byte offsets through
non-UTF-8 data, warning recovery for other stream/resource paths,
handler stack mutation edge cases during active handler dispatch,
`open_basedir` policy beyond the current request-local allow-list check for
local `file_get_contents()` and `fopen()` paths, stat caching, host SAPI body
streams, or native filesystem lowering. Direct
native `file_get_contents(...)` calls stop at a
dedicated filesystem-read codegen boundary before argument lowering or backend
selection, while native function-table introspection can still see the known
builtin name.
`filesize()` is interpreter-only for one string local path in the current
runtime. It uses the same process-path-then-repo-root relative path policy as
the other local metadata builtins, returns the host regular-file byte length
as an integer, and returns `false` for missing paths or non-file paths such as
directories. Successful host metadata reads are cached by resolved local path
until `clearstatcache()` clears all entries or `clearstatcache(false, $path)`
removes the matching entry. It rejects stream wrappers instead of modeling
wrapper metadata. Include-path lookup, full PHP stat-cache breadth,
`open_basedir`, exact warnings, non-UTF-8 paths, oversized file handling
beyond the current signed 64-bit integer subset, and native filesystem
lowering remain out of scope. Native function-table introspection recognizes
the name, while direct native calls reject under the function-call boundary.
`filemtime()` is interpreter-only for one string local path in the current
runtime. It uses the same process-path-then-repo-root relative path policy as
the other local metadata builtins, returns the host filesystem modification
time as a Unix-timestamp integer for existing local entries, and returns
`false` for missing paths. Successful host metadata reads share the same
bounded `clearstatcache()`-managed cache as `filesize()`. It rejects stream
wrappers instead of modeling wrapper metadata. Include-path lookup, full PHP
stat-cache breadth, `open_basedir`, exact warnings, non-UTF-8 paths,
pre-Unix-epoch timestamps, oversized timestamp handling beyond the current
signed 64-bit integer subset, and native filesystem lowering remain out of
scope. Native function-table
introspection recognizes the name, while direct native calls reject under the
function-call boundary.
`realpath()` is interpreter-only for one string local path. It uses the same
process-path-then-repo-root relative path policy as the metadata builtins,
returns a UTF-8 resolved host path for existing local paths, and returns
`false` for unresolved local paths. Successful resolutions also populate a
bounded request-local `realpath_cache_get()` table keyed by resolved path with
the current PHP-shaped `key`, `is_dir`, `realpath`, and `expires` fields.
Successful local `file_get_contents()` reads, local `fopen()` calls for paths
that existed before opening, and successful local include/require reads also
populate one bounded entry for the resolved target path. `clearstatcache(false)`
leaves that table intact,
`clearstatcache(true, $filename)` removes only a non-empty exact matching
cached resolved-path key, and one-argument `clearstatcache(true)` clears all
bounded realpath entries. `realpath_cache_size()` returns a deterministic
request-local integer over that same bounded table: empty caches report `0`,
and cached resolved UTF-8 path entries contribute a positive stable size for
empty/non-empty and clear/invalidation probes. Stream wrappers are rejected
instead of being modeled. Symlink policy differences, exact warning plus
`false` fidelity, include-path lookup, `open_basedir`, non-UTF-8 paths,
realpath-cache ancestor entries, cache entries from filesystem operations
beyond successful `realpath()`, local `file_get_contents()`, pre-existing local
`fopen()` paths, and successful local include/require reads, exact
realpath-cache key hashes and expiration policy,
exact `realpath_cache_size()` memory-byte accounting, and native filesystem
lowering remain out of scope.
Native function-table introspection recognizes the names, while direct native
`realpath(...)` calls stop at a dedicated filesystem-canonicalization codegen
boundary before argument lowering or backend selection until generated code has
native filesystem canonicalization, symlink/path policy, warning/false recovery,
include_path/open_basedir/stat cache, non-UTF-8 path handling, references/COW,
and exact native diagnostics; direct native
`realpath_cache_get()`/`realpath_cache_size()` calls still reject under the
generic function-call boundary.
`getcwd()` is an interpreter-only request-state/filesystem builtin for the
current CLI process. It accepts no arguments and returns the process current
working directory as a UTF-8 string, while function-table introspection
recognizes the name. It does not model `chdir()` state mutation, failure
returning `false`, non-UTF-8 working-directory paths, SAPI-specific working
directory policy, include-path interaction, `open_basedir`, exact warnings, or
native filesystem lowering. Direct native `getcwd()` calls stop at a dedicated
current-directory codegen boundary before argument lowering or backend output
until generated code has process/request cwd state, UTF-8/path policy, SAPI cwd
behavior, references/copy-on-write, and exact native diagnostics.
`is_writable()` is interpreter-only for one string local path. It uses the same
process-path-then-repo-root relative path policy as the filesystem metadata
builtins, rejects stream-wrapper paths, returns `false` for missing local
paths, and treats existing readonly host metadata as not writable. Permission
portability, exact warning behavior, include_path lookup, `open_basedir`,
stream wrappers, symlink policy, stat-cache behavior, TOCTOU semantics,
non-UTF-8 paths, and native filesystem lowering remain out of scope. Native
function-table introspection recognizes the name, while direct native
`is_writable(...)` calls still reject under the generic function-call
boundary.
`is_link()` is interpreter-only for one string local path. It uses the same
process-path-then-repo-root relative path policy as the filesystem metadata
builtins, rejects stream-wrapper paths, returns `true` for host symbolic links
detected through local symlink metadata, and returns `false` for ordinary files
and missing local paths. Include-path lookup, `open_basedir`, stream wrappers,
exact warning behavior, stat-cache behavior, TOCTOU semantics,
broken-symlink policy fidelity, non-UTF-8 paths, and native filesystem lowering
remain out of scope. Native function-table introspection recognizes the name,
while direct native `is_link(...)` calls still reject under the generic
function-call boundary.
The table includes interpreter-only array builtins such as
`array_change_key_case`, `array_column`, `array_is_list`, `array_product`,
`array_reduce`, and `array_filter`; direct calls to those builtins still reject
under the array-lowering boundary.
Direct `strlen($value)` calls fold only when `$value` is an already-lowerable
known string operand, including tracked string expressions whose possible
values have one uniform byte length. A selected-`clang` assembly snapshot
validates that this existing folded LLVM IR is handed to the chosen backend
through stdin without changing production lowering behavior. Non-string
coercions, arrays, objects, resources, references/copy-on-write, dynamic
calls, and exact native PHP diagnostics remain outside this slice.
Direct `str_starts_with(...)` calls reject through a dedicated native
string-prefix boundary before argument lowering or backend selection. Native
function-table introspection still recognizes `str_starts_with`, but native
call execution still lacks PHP string conversion, empty-needle handling,
binary byte semantics, argument diagnostics, references/copy-on-write, and
exact native diagnostics.
Direct `file_get_contents(...)` calls reject through a dedicated native
filesystem-read boundary before argument lowering or backend selection. Native
function-table introspection still recognizes `file_get_contents`, but native
call execution still lacks stream-wrapper handling, local file I/O, binary
string byte fidelity, warning plus `false` recovery, stream contexts,
include-path lookup, `open_basedir` and stat-cache behavior,
references/copy-on-write, and exact native diagnostics.
The first stream-resource slices are interpreter-only. `fopen("php://memory",
$mode)` and `fopen("php://temp", $mode)` allocate request-local resource ids
backed by a Rust string buffer for simple `r`, `w`, `a`, or `c` modes with
optional `+`, `b`, or `t` flags. `fopen("php://input", $mode)` allocates a
read-only request-local stream over the deterministic `PHPC_REQUEST_BODY`
seed, reports PHP/Input/`rb` metadata, and reuses the current seek/read cursor
machinery without modeling a host SAPI input stream. `fopen($localPath, $mode,
$use_include_path = false, $context = null)` uses a host local file handle for
the same simple mode grammar, optionally resolves through the existing bounded
include-path-then-source-relative candidate order, and accepts a bounded
stream-context resource without applying wrapper-specific context behavior.
Local open failures,
including missing read targets, emit a bounded `E_WARNING`, return `false`,
and continue through the current request-local warning-handler stack before
the stderr fallback. `stream_context_create()`
allocates a request-local context resource that stores array options and the
bounded params slice for `notification` plus `options`.
`stream_context_get_options()` returns those stored options, and
`stream_context_get_params()` returns the stored bounded params plus the
current `options` entry.
`stream_context_get_default()` lazily allocates and returns one request-local
default context resource, while `stream_context_set_default()` and
`stream_context_set_option()` merge string-keyed wrapper/option entries into
that same bounded context table. `stream_context_set_params()` updates the
stored bounded params and merges an `options` param into the same option table,
preserving the previous `notification` value when only options are supplied.
Unknown params, notification callback invocation, wrapper-specific effects, and
native lowering remain unsupported. `fwrite()`,
`fread()`, `rewind()`, `stream_get_contents()`, `feof()`, `ftell()`,
`fseek()`, `fstat()`, `stream_get_meta_data()`, and `fclose()` mutate, consume,
or inspect the resource cursor or bounded metadata; append-mode writes are
routed to EOF. A separate request-local directory resource table backs
`opendir()`, `readdir()`, `rewinddir()`, and `closedir()` for local UTF-8
directories, returning `.`, `..`, and sorted host entry names through the same
generic resource value shape. `fseek()` supports the built-in `SEEK_SET`,
`SEEK_CUR`, and `SEEK_END` constants for the current memory/temp/local-file
resource set, and `feof()` tracks the bounded EOF flag produced by exhaustive
reads. `fstat()` exposes buffer size for memory/temp/input handles and host
metadata for local files;
`stream_get_meta_data()` exposes deterministic wrapper/type/mode/URI,
seekable, unread-byte, and EOF metadata. Local file reads remain UTF-8 text
reads. This gives WordPress-style temporary request, cache-file stream, and
directory-scanning paths an executable path without claiming full PHP
resources: sockets, HTTP/FTP/phar wrappers, filters, context option effects
beyond persistence, broader wrapper/status metadata APIs, exact host
directory iteration order,
binary/non-UTF-8 byte strings, real SAPI body stream lifetime, writable
`php://input` edge behavior, `php://temp` spill-to-disk thresholds, permissions
policy, locking, stat-cache behavior, warning plus `false` recovery beyond the
documented local read/open slices, references/copy-on-write, and exact
resource id/type behavior remain out of scope. Native stream-resource calls
reject before lowering under a dedicated resource boundary, while
function-table introspection recognizes the names.
Direct `filesize(...)` calls reject through the native function-call boundary.
Native function-table introspection still recognizes `filesize`, but native
call execution still lacks filesystem metadata, warning plus `false` recovery,
include_path/open_basedir/stat-cache policy, stream-wrapper handling,
references/copy-on-write, and exact native diagnostics.
Direct `filemtime(...)` calls reject through the native function-call
boundary. Native function-table introspection still recognizes `filemtime`,
but native call execution still lacks filesystem modification-time metadata,
warning plus `false` recovery, include_path/open_basedir/stat-cache policy,
stream-wrapper handling, references/copy-on-write, and exact native
diagnostics.
Direct `realpath(...)` calls reject through a dedicated native
filesystem-canonicalization boundary before argument lowering or backend
selection. Native function-table introspection still recognizes `realpath`, but
native call execution still lacks filesystem canonicalization,
symlink/path policy, warning/false recovery, include_path/open_basedir/stat
cache, non-UTF-8 path handling, references/COW, and exact native diagnostics.
Native function-table introspection recognizes `is_writable`, but direct
native `is_writable(...)` calls still reject under the generic function-call
boundary until generated code has local writability checks, permission policy,
warnings, include_path and `open_basedir` handling, stream-wrapper
rejection/dispatch, symlink/stat-cache/TOCTOU behavior, non-UTF-8 path
handling, references/copy-on-write, and exact native diagnostics.
Native function-table introspection recognizes `is_link`, but direct native
`is_link(...)` calls still reject under the generic function-call boundary
until generated code has local symlink metadata lookup, include_path and
`open_basedir` handling, stream-wrapper rejection/dispatch, stat-cache/TOCTOU
behavior, broken-symlink policy fidelity, non-UTF-8 path handling,
references/copy-on-write, and exact native diagnostics.
Direct `defined($name)` calls fold only when `$name` is an already-lowerable
string value whose possible values are supported unqualified constant names
with a uniform answer against the current exact built-in constant-name set.
Exact `CASE_LOWER`, `CASE_UPPER`, `ARRAY_FILTER_USE_BOTH`,
`ARRAY_FILTER_USE_KEY`, `SORT_REGULAR`, `SORT_NUMERIC`, `SORT_STRING`,
`PHP_VERSION_ID`, `PHP_VERSION`, and `PHP_INT_MAX` names fold true, while other
supported unqualified names fold false.
Runtime-defined constants, source-order constant declarations, `define(...)`,
`constant(...)`, qualified names such as `\Sodium\CRYPTO_AUTH_BYTES`,
unsupported names, namespace-aware lookup, and exact native errors remain
outside this slice.
Direct `empty($name)` calls fold from the same straight-line static-variable
map used by direct `isset($name)`: missing variables and statically falsey
scalar/null values fold to true, while statically truthy scalar values fold to
false. Array offsets, object properties, arrays, complex operands,
unset/mutation interactions, ambiguous truthiness, references/copy-on-write,
and exact native error behavior remain outside this native slice.
Array/object operands remain rejected until native array/object lowering
exists. This is static folding, not runtime call dispatch. Dynamic calls,
wrong arity, non-string `function_exists` names, non-bool `is_callable`
syntax-only flags, callable-name output parameters, array/object/method
callables, direct `assert(...)`, callable builtin dispatch, runtime call lookup, stack-frame layout,
arity/type diagnostics, dynamic string-call dispatch, and exact native error
behavior remain unsupported. The
`define()`/`constant()` constant-table builtins and unsupported
`defined(...)` forms have a separate global-constant rejection boundary.
Native lowering also rejects user-function declarations and return statements,
including declarations whose parameter list uses the parser's optional
trailing-comma syntax, before traversing function bodies until generated code
has function symbol tables, stack-frame layout, default parameter binding,
recursion guards, return-value flow, bounded parameter/return type enforcement,
and exact native error behavior.
Native lowering rejects executable magic constants `__LINE__`, `__FILE__`,
`__DIR__`, `__FUNCTION__`, and `__METHOD__` until generated code has source
mapping, path canonicalization, function/method-context tracking, eval/include
source interaction rules, and exact native error behavior.
Native lowering rejects built-in constants, runtime-defined constants, bare
constant reads, top-level `const` declarations, `define()`/`constant()`, and
unsupported `defined(...)` forms before operand/argument lowering until
generated code has native constant tables, source-order definitions,
namespace-aware lookup, and exact native error behavior.
Native lowering rejects class declarations, clone expressions, `instanceof`
expressions, and method-call expressions before body, operand, receiver, or
argument lowering. Object
metadata builtins have a dedicated native rejection before operand or argument
lowering, except for direct static
false-folding of
`class_exists`, `interface_exists`, `trait_exists`, and `enum_exists` when
their name argument is already lowerable as a string and the optional autoload
argument is already lowerable as a boolean. That metadata-exists native slice
does not expose native class metadata, autoloading, or callable dispatch.
Native lowering also folds direct `property_exists` and `method_exists` to
false when both arguments are already lowerable strings, because class
declarations still reject and there is no native class member table.
Direct string/string `is_a` and `is_subclass_of` calls with optional lowerable
boolean `allow_string` flags fold to false for the same no-native-class-table
and no-native-inheritance boundary.
Broader object/class lowering remains rejected until generated code has native
object layout, object handles, visibility checks, method dispatch, class
metadata tables, property/method tables, inheritance/interface/trait/enum
registries, property-slot cloning, `__clone` dispatch, reference-slot
metadata, inheritance, autoload interaction, and exact native error behavior.
Object instantiation and constructor dispatch have a separate native rejection
until generated code has native object allocation, object handles, constructor
calls, visibility checks, autoload/class lookup, references/copy-on-write, and
exact native object-instantiation errors.
Instance property reads/writes and dynamic property-name access have a
separate native object-property boundary until generated code has native
object layout, property tables/slots, visibility checks, magic property hooks,
dynamic property policy, references/copy-on-write, and exact native
object-property errors.
The method-call boundary is more specific: native method support still needs
method tables and lookup, instance and static receiver resolution, `$this` and
called-class/late-static-binding context, argument/arity diagnostics,
visibility enforcement, reference/COW parameter and return behavior, and exact
native method-call errors.
Native lowering rejects array literals, array indexing, array assignment,
`foreach` array iteration, array offset `unset`, and array builtin function
calls before body, operand, argument, or callback lowering until generated code
has native array storage layout, key normalization, copy-on-write containers,
references, callback dispatch, and exact native error behavior.
Native lowering rejects `if`/`elseif`/`else`, including alternate
colon/`endif` syntax, `while`, `for`, `do ... while`, `switch`, `break`, and
`continue` before condition, body, case, or loop-control lowering until
generated code has PHP truthiness, branch layout, loop control flow, switch
fallthrough, references/copy-on-write side-effect behavior, and exact native
error behavior.
Native lowering rejects compound assignment, null coalescing assignment,
increment/decrement, assignment expressions, direct variable `unset`, and
multiple-operand `unset` before operands or mutation targets are lowered until
generated code has read-modify-write ordering, null-aware mutation, unset
symbol-table effects, references/copy-on-write behavior, and exact native error
behavior.

## Dynamic Features

Dynamic PHP features will be implemented as runtime fallback zones:

- dynamic function calls use runtime lookup; the first implemented slice accepts
  string-valued callees that resolve to the documented callable builtin subset
  or user-defined functions
- variable variables use materialized symbol tables; current variable-variable
  syntax is rejected with an explicit diagnostic before execution
- dynamic includes will use runtime include resolution
- `eval` will parse and execute in the caller scope
- namespaces and imports use a bounded class-name plus same-namespace function
  declaration/call slice. Namespace-scoped function declarations register under
  their resolved names; unqualified direct calls inside a namespace first look
  for a same-namespace function and then fall back to global builtins/user
  functions. Function imports, qualified function calls, leading-backslash
  fully-qualified function calls, namespace-scoped constants, and dynamic
  string-name namespace expansion remain boundaries.
- global constants use a narrow interpreter constant table: exact uppercase
  `CASE_LOWER`, `CASE_UPPER`, `ARRAY_FILTER_USE_KEY`, and
  `ARRAY_FILTER_USE_BOTH` are available as bare built-in constants, while
  runtime-defined constants can be created with
  `define($name, $value)`, queried with `defined($name)`, and read with
  `constant($name)` or a bare unqualified constant name for the documented
  string-name and scalar/array value subset. Top-level single and grouped
  `const NAME = value;` declarations define unqualified constants at statement
  execution time over the current constant-expression and scalar/array value
  subset, including references to previously defined unqualified constants and
  the current built-in constant slice

String-valued dynamic function lookup and the narrow local `require path;` /
`require_once path;` / `include path;` / `include_once path;` statement slice
are executable today.
Variable-variable execution and `eval` remain design boundaries; direct
`eval(...)` syntax is reserved and rejected with a stable parse diagnostic.
Namespace declarations and top-level class `use` import declarations execute
as metadata/no-op statements in `phpc run`, but the native path still rejects
namespace declarations/imports before scalar folding or backend execution.
First-class callable syntax such as `strlen(...)` and `$callback(...)` also
stops at a stable parse diagnostic until Closure creation and callable object
semantics exist.
Call-site argument unpacking such as `handler(...$args)` stops at a dedicated
parse diagnostic until iterable expansion order, string-keyed named-argument
interaction, by-reference argument propagation, variadic collection, duplicate
argument diagnostics, and native lowering exist.
Call-time by-reference arguments such as `handler(&$value)` also stop at a
dedicated parse diagnostic because the current by-reference parameter execution
slice uses ordinary direct-variable call arguments and callee parameter
metadata; legacy call-site `&` syntax, alias setup, default handling,
variadic/unpacking interaction, references/copy-on-write, and native lowering
remain unsupported.
No-capture anonymous closure expressions, static anonymous closure expressions,
and non-static arrow function expressions allocate runtime closure values in
`phpc run`, which can be assigned, read, truth-tested, reflected, and invoked
through the bounded direct `$closure(...)`, `call_user_func()`,
positional-array `call_user_func_array()`, and `ReflectionFunction::invoke()`
paths. Invocation prebinds the closure's by-value captured snapshot into the
callee local scope and then reuses the user-function call frame machinery.
For non-static anonymous closures created while `$this` is visible, the
closure id also records the bound object plus class and called-class context.
The documented direct, callback, and reflection invocation paths pass that
context into the user-function frame, while `static function` closures leave
`$this` unbound.
Direct `$closure(...)` invocation also routes through the checked direct-call
argument evaluator, so current direct-variable and direct array-offset
by-reference parameter bindings are installed before the closure body executes
and written back through the existing alias metadata.
For `call_user_func_array()` closure callbacks, the same checked callback
argument evaluator used by user functions builds covered reference parameter
bindings, then the closure frame combines those bindings with the captured
locals before executing.
For direct `call_user_func($closure, ...)`, the closure frame deliberately uses
the value-only callback argument path and emits the same bounded recoverable
warnings as string user-function callbacks when a reached non-variadic closure
parameter is declared by reference; direct-variable by-reference captures are
still prebound as cells.
Closure rebinding through `Closure::bind`/`bindTo` is not represented yet.
Arrow values do not bind implicit captures or execute their synthetic return
bodies. Static arrow functions stop at a dedicated parse boundary until
no-`$this` binding, implicit capture metadata, references/copy-on-write, and
native lowering exist.
`phpc run` supports an opt-in execution-step budget via
`PHPC_MAX_EXECUTION_STEPS`; it is enforced at statement execution and loop
iteration boundaries to diagnose runtime loops, but it intentionally does not
count parser work, declaration registration, or native lowering.
`PHPC_TRACE_INCLUDES=1` writes include/require target paths to process stderr
before each target is parsed/executed so external timeout probes can retain the
last include frontier.
`PHPC_TRACE_PARSE=1` writes parser frontier lines for top-level statements,
class/interface/enum members, and block statements. It is an operational trace,
not PHP-visible output, and is intended for external timeout diagnosis.
Magic class names in `new` expressions, including `new self()`,
`new parent()`, and `new static()`, stop at a stable parse diagnostic until
class context tracking, parent resolution, and late static binding exist.
`is_callable($value, true)` has a narrow syntax-only array callable shape check
for two-element arrays keyed `0` and `1` whose first value is a string class
name or current object and whose second value is a string method name. This
does not resolve classes or methods and is not callable dispatch. Normal
`is_callable([$receiver, $method])` resolution checks the same shape against
current declared method metadata: object receivers are true for public declared
methods, and class-string receivers are true for public static declared
methods. Array/object callable dynamic invocation, private/protected
caller-context method callability, method calls, first-class callable syntax,
and namespace/autoload-aware callable resolution are still outside the
implemented dynamic-call subset. Constant names that are lexed as language keywords or
literals cannot be read bare, and case-insensitive legacy constants, extension
constants, namespace-qualified constants, nested or namespace-aware `const`
declarations, dynamic `const` values, class constants through
`constant(...)`/unsupported `defined(...)` forms, typed and multi-declarator
class constants, `static::CONST`, references/copy-on-write for constant values,
and broader constant lowering are still outside the implemented constant
subset. Namespace-qualified and leading-backslash fully-qualified constant
reads stop at dedicated parse diagnostics until namespace-aware constant-table
lookup, fallback behavior, imports, and native lowering exist. Direct
`ClassName::CONST`, `self::CONST`, and `parent::CONST`
execution use class metadata instead of the global constant table. Native
lowering currently folds only direct
supported-string-name `defined($name)` checks and otherwise rejects the
global-constant slice explicitly rather than emitting partial constant-table
code.

## Namespace/Import Boundary

The first executable namespace/import slice is class-name only. The parser
accepts one unbracketed named `namespace` declaration per file and simple
top-level class `use` imports with optional aliases. Class declarations and
class-like references in `extends`, `new`, `instanceof`, static members, and
`ClassName::class` are stored as canonical names without a leading slash and
resolved through the lexical namespace/import table. Namespace and `use`
statements are execution no-ops in `phpc run` because the parser has already
resolved the class-like names in the AST.
Class inheritance uses the same resolved class-like names. Parent classes must
already be present in the interpreter's class metadata table from the current
program or from an executed include/require path; class lookup does not invoke
autoload callbacks.

Unsupported namespace/import behavior remains: bracketed namespace blocks,
global namespace blocks, multiple namespaces in one file, namespace-scoped
functions/constants, namespace-qualified function calls, leading-backslash
fully-qualified function calls, namespace-qualified constant reads,
leading-backslash fully-qualified constant reads, grouped imports, function
imports, constant imports, string-name import expansion, trait `use`
execution, `__NAMESPACE__`, autoload interaction, exact PHP diagnostics,
partial-output behavior, and namespace-aware native lowering. The native path
rejects namespace declarations/imports before scalar folding or backend
execution until native symbol tables and namespace context exist.

## Object/Class Boundary

The current object/class step is a narrow object-execution boundary, not full
PHP object execution. `php_runtime` has a `PhpClassTable`, stable `ClassId`
handles, class metadata, property metadata, method metadata, visibility
markers, derived object shapes for instance-property layout, and minimal
object values. Class and
method lookup are case-insensitive, property lookup is case-sensitive, and
duplicate class/member metadata produces structured runtime errors.

`phpc run` parses top-level `class Name { ... }` declarations into that metadata
registry. The accepted member subset records public/protected/private
visibility, static flags, property names without defaults, and method names
whose parameters/bodies use the existing function parser subset. `new
ClassName(...)` can instantiate a declared class and execute a public or
inherited public `__construct` method with `$this` bound to the new object
handle. Classes without constructors still require no constructor arguments.
Objects whose class declares or inherits a public non-static no-argument
`__destruct` method are tracked after successful allocation and run during
normal shutdown in reverse allocation order; current shallow clones are tracked
as separate handles for that same destructor path. User-declared destructors
are validated when class members register, so non-public, static, or
parameterized `__destruct` declarations fail before object allocation instead
of being discovered by the shutdown queue.
The allocated object stores class identity and `null` instance-property slots
in inherited parent-to-child declaration order while skipping static
properties. Compatible public/protected redeclarations share the inherited
slot with the descendant visibility, while private parent redeclarations stay
separate child slots. Each slot carries the declaring class id/name used for
non-public access checks and private-property mangling.

`phpc run` can read and write public instance properties by static property
name, for example `$box->name` and `$box->name = "Ada"`. Writes mutate the
current object value stored in that variable. Direct `isset($box->name)` checks
the current public slot without treating a missing name as a property read.
Direct `empty($box->name)` checks the current public slot truthiness and treats
null slots, missing names, undefined target variables, and non-object target
variables as empty for the current direct-object-variable subset.
`get_class($object)` returns the declared class name stored in the current
minimal object value and is also available through string-valued dynamic
function calls. `get_debug_type($value)` reports current scalar/array type
names and the declared class name for current object values.
`class_exists($name[, $autoload])` checks the interpreter's already-registered
class metadata table by string class name, using the same case-insensitive
class lookup as instantiation. The autoload flag accepts current bool-like
scalar values; when truthy and the initial lookup misses, currently registered
string user-function callbacks, public `"ClassName::method"` static-method
string callbacks, public `[object, "method"]` instance-method array callables,
and public `["ClassName", "method"]` static-method array callables run before
the metadata check returns.
Missing named or direct-variable string class names in `new` expressions use
the same bounded autoload callback path before reporting the current
undefined-class diagnostic.
`null`, arrays, objects, references, and exact PHP
deprecation/`TypeError` behavior remain unsupported for that flag.
`interface_exists($name[, $autoload])`, `trait_exists($name[, $autoload])`,
and `enum_exists($name[, $autoload])` accept the same string-name and
bool-like scalar autoload boundary. `interface_exists()` shares the bounded
autoload callback path with `class_exists()`, and `trait_exists()` uses that
same path for declared top-level trait metadata. Enum lookups still check only
already-declared metadata. Invokable objects, closures at invocation time,
non-public methods, class-string non-static methods, object static methods,
arbitrary callable arrays, exact PHP warning/throw behavior, and native
autoload lowering remain unsupported. `class_exists()` also reports true for
declared enums in the current class-like metadata slice.
`property_exists($object_or_class, $property)` checks the same declared and
inherited property metadata for current object values or string class names,
with case-sensitive property names and no autoload side effects.
`get_class_vars($class_name)` accepts declared string class names and returns
public declared and inherited property names in child-to-parent declaration
order with supported constant-expression default values or `null` for
properties without defaults.
`get_object_vars($object)` accepts current object values and returns public
exact and inherited instance property names with their current slot values in
parent-to-child slot order.
`get_mangled_object_vars($object)` accepts current object values and returns
public, protected, and private instance slots in declaration order with
PHP-style property keys: public names as-is, protected names as `\0*\0name`,
and private names as `\0ClassName\0name` using the declaring class name.
Dynamic properties, readonly property metadata/enforcement, non-constant or
typed property defaults, promoted constructor properties, trait/interface
properties, and non-public property visibility-context behavior beyond
same-declaring-class private access and class/ancestor protected access remain
outside the current object model.
`is_a($object_or_class, $class_name[, $allow_string])` checks exact-class,
single-parent ancestor, and recorded `implements` metadata relationships
against the current metadata table.
`is_subclass_of(...)` shares the same argument boundary and walks the current
single-parent chain plus inherited `implements` metadata while keeping
exact-class and missing-class cases false.
`$value instanceof Name` uses the same current object class metadata,
single-parent chain, and recorded `implements` metadata for object values.
Non-object values return false. `implements` names are recorded as metadata so
internal names can participate in relationships without being declared. For
interfaces declared in the current parsed program, runtime class registration
checks concrete classes, including concrete children of abstract implementors,
for public methods with the required interface method names and required static
or non-static method shape. Parent interfaces declared before or after the
child are flattened into both relationship metadata and method-presence checks.
Implementations may omit an interface parameter type or repeat the same type
text case-insensitively, but may not add a parameter type to an untyped
interface parameter or substitute a different type for a typed interface
parameter. Implementations may add a return type to an untyped interface
method, but a typed interface method requires the implementation to declare the
same return type text case-insensitively. Public static methods satisfy only
static interface method requirements and do not satisfy non-static interface
method requirements. This is a bounded compatibility check only; full
parameter variance, broader return type covariance/contravariance,
type subtyping, alias/import resolution, union/intersection canonicalization,
cyclic parent-interface inheritance beyond stable rejection, broad
built-in/internal interface method enforcement beyond the current
`Countable`, `Iterator`, and `IteratorAggregate` shape checks, exact PHP error
objects, autoload behavior, and native lowering remain separate work. A bounded
core interface catalog seeds `interface_exists()` and `get_declared_interfaces()` for
`Traversable`, `IteratorAggregate`, `Iterator`, `Serializable`, `ArrayAccess`,
`Countable`, and `Stringable`. Protocol execution is added one narrow slice at
a time: current `ArrayAccess` paths dispatch selected offset methods, current
`Stringable`/`__toString` paths cover string conversion, and current
`Countable` paths require concrete implementors to expose a public non-static
`count()` method with no required parameters, let `is_countable($object)`
recognize recorded `implements Countable` metadata, and let `count($object)`
dispatch that method with an integer result. Current `Iterator` and
`IteratorAggregate` paths require concrete implementors to expose their
required public non-static methods with no required parameters so
`is_iterable($object)` can observe those metadata relationships. Full internal
interface signature enforcement, tentative return-type notices, object
`foreach`, iterator method execution, `IteratorAggregate::getIterator()`
dispatch, broad protocol composition, references/copy-on-write, exact
diagnostics, and native lowering remain
separate runtime work.
`get_parent_class($object_or_class)` accepts current object values or declared
string class names and returns the immediate parent class name when one is
recorded, otherwise false.
`class_implements($object_or_class[, $autoload])` is a bounded
reflection-style metadata builtin over the current class/interface table. It
accepts object values or string class names, optionally invokes the existing
class autoload path for string misses, and returns an associative
interface-name array. The collection follows system PHP for the covered
single-parent/user-interface cases, including parent-class interfaces before
child-class interfaces.
`class_uses($object_or_class[, $autoload])` is the matching bounded
class/trait metadata builtin. It accepts object values or string class names,
optionally invokes the existing class autoload path for string misses, and
returns an associative array of the resolved class's direct trait names. It
does not recurse into parent classes or synthesize broader Reflection metadata.
`class_parents($object_or_class[, $autoload])` walks the same class metadata
chain from immediate parent to root and returns a PHP-shaped associative array
of parent class names. This keeps `class_uses()` non-recursive while enabling
covered userland recursive trait helpers that explicitly combine parent names
with direct trait metadata.
`ReflectionClass` is a bounded runtime metadata object for declared
class-like values. `new ReflectionClass($object_or_class)` accepts object
values and string names for the current declared class, interface, and trait
tables, including the existing autoload callback path for string misses. The
object stores request-local reflection state and dispatches the current
`getName()`, `getShortName()`, `isInterface()`, `isTrait()`,
`isInstantiable()`, `getParentClass()`, `getInterfaceNames()`,
`hasMethod($name)`, `getFileName()`, `getStartLine()`, `getEndLine()`, and
`getDocComment()` methods directly through the interpreter. Class-like source
paths are tracked in interpreter-side metadata when class, interface, and
trait declarations are loaded from a known CLI/fixture or include path; start
and end lines plus directly preceding `/** ... */` doc-comments come from the
parsed declaration. The same
request-local reflection path now covers declared user-class properties:
`hasProperty($name)`, `getProperty($name)`, and zero-argument
`getProperties()` resolve current class metadata, include inherited public and
protected properties, and exclude inherited private properties when reflecting
a child class. This is metadata only.
`ReflectionMethod` uses the same core placeholder class plus request-local
state pattern for declared user class, interface, and trait methods. The
constructor accepts an object or class-like string plus a string method name,
resolves inherited class methods through the existing method metadata chain,
and exposes the bounded source metadata methods, modifier predicates,
`getDeclaringClass()`, parameter counts, `getParameters()`,
`hasReturnType()`, and `getReturnType()` through interpreter dispatch.
Class-method source paths are tracked in the same method-signature table as
parameter and return metadata when declarations are loaded from a known
CLI/fixture or include path; start/end lines and directly preceding
`/** ... */` doc-comments come from the parsed method declaration. Interface
and trait methods currently retain parsed line/doc-comment metadata but do not
persist declaration source-file paths. Return type objects reuse the same request-local
`ReflectionNamedType`, `ReflectionUnionType`, and `ReflectionIntersectionType`
state as the property type metadata slice.
`ReflectionFunction` follows that same core placeholder plus request-local
state pattern for declared user functions named by string. It stores parsed
user-function metadata for `getName()`, source file, start/end lines, direct
docblock text, parameter counts, `getParameters()`, `hasReturnType()`,
`getReturnType()`, and `returnsReference()`. Source paths are tracked beside
registered function declarations because the AST remains source-text scoped;
included files record the include source path at declaration-registration time.
The lexer preserves bounded `/** ... */` doc-comment tokens so the parser can
attach a directly preceding docblock to a function declaration.
`ReflectionFunction::invoke()` and `invokeArgs()` re-enter the existing
user-function call path with evaluated by-value arguments; `invokeArgs()` uses
the current ordered PHP array entries as positional values, not PHP 8 named
argument semantics for string keys. A named internal-function slice re-enters
the existing builtin dispatcher for `strlen`, `strtolower`, `trim`, `ltrim`,
`rtrim`, `strcasecmp`, `str_contains`, `str_starts_with`, `str_ends_with`,
`strpos`, `substr`, `sprintf`, `implode`, `basename`, `dirname`, `defined`,
`function_exists`, and `php_sapi_name`. Closure expressions also register a
request-local `ReflectionFunction` metadata snapshot, parsed body, and captured
by-value snapshot keyed by closure id, so direct closure invocation,
closure-valued `call_user_func()`/`call_user_func_array()` callbacks, and
`new ReflectionFunction($closure)` can execute the body for the current
untyped positional by-value argument slice, plus covered direct
`$closure(...)` by-reference parameters over direct-variable and direct
nested array-offset arguments, and the covered `call_user_func_array()`
closure-callback reference parameter slice when the argument array carries
explicit covered reference elements. Reflection also
exposes the bounded `{closure}` name, current source file/start/end line
metadata, parameter metadata, return type, false doc-comment metadata, and false
by-reference-return status. Explicit `use (&$name)` closure captures store the
source direct-variable cell, and closure invocation prebinds that cell into the
closure local scope so closure writes update the captured variable even after
the creating user-function scope has returned. The function path intentionally
rejects other internal functions, by-reference capture of array-offset/property/
ArrayAccess alias roots outside the documented alias metadata, array-callable
`call_user_func()` forms beyond the
bounded public object/static method by-value slice, variadic reference
parameters through `call_user_func()`, typed
parameter/return declarations during invocation,
named closure callback arguments, and reference returns until those callable
execution and aliasing sources exist, and doc-comment association does not yet
model attributes or every PHP trivia edge case.
`ReflectionMethod::invoke()` and `invokeArgs()` use the same request-local
method metadata and re-enter the existing method call path for declared
user-class methods, including public, protected, and private reflected
methods. Non-static invocation preserves `$this` object identity for mutations,
uses the reflected declaring class as the method context, and keeps the target
object's class as the called class for `static::` lookup; static invocation
uses the reflected class for inherited `static::` lookup and accepts `null` or
object targets. Static trait methods reflected from the trait itself also
execute through the by-value argument path when the target is `null` or an
object, with a narrow trait reflection context for `__CLASS__`, `__METHOD__`,
`self::class`, `static::class`, `get_called_class()`, and static
`self::method()`/`static::method()` calls that resolve to executable methods
on the reflected trait. Interface methods and other abstract reflected methods
stop at a stable `ReflectionMethod::invoke` runtime boundary before dispatch,
mirroring PHP's abstract-method invocation rule without materializing exact
`ReflectionException` objects. Non-static trait methods, trait class constants,
`parent` context behavior, `new self`/`new static` from reflected traits,
internal methods, by-reference invocation,
reference returns, typed parameter/return declarations during invocation, and
native lowering remain outside this bounded reflection
invocation path. Method `invokeArgs()` has the same positional-only array
treatment as function `invokeArgs()`.
`ReflectionParameter` follows the same request-local state pattern for
parameters reached from `ReflectionMethod::getParameters()`,
`ReflectionFunction::getParameters()`,
`new ReflectionParameter([$object_or_class, $method], $parameter)`, or
`new ReflectionParameter($function, $parameter)` for declared user function
strings. Parameter objects store copied function or method metadata plus the selected parsed parameter
metadata, so the interpreter can answer names, positions, declaring
class/function, optional/default availability and values, by-reference flags,
variadic flags, type-presence checks, nullability checks, and simple named
type metadata. `ReflectionParameter::getType()` now materializes a
request-local `ReflectionNamedType` object for a single parsed named type, or
a request-local `ReflectionUnionType`/`ReflectionIntersectionType` object for
bounded union and pure intersection parameter types, and stores the copied
type names, nullable flags, and builtin flags in the interpreter. Untyped
parameters return `null`. It does not expose
attributes, files, line numbers, doc comments, extension/internal metadata,
closure parameter targets, invocation-time reference binding, exact exception
objects, DNF type objects, runtime argument/return type enforcement, or native
lowering.
`ReflectionProperty` uses a core placeholder class plus request-local state for
the selected declaring class id/name, property name, visibility, static flag,
directly preceding property doc-comment text, and optional property type
metadata. The interpreter answers name, doc comment, declaring class,
modifier-mask, visibility/static predicates, default-value availability and
value, and `hasType()`/`getType()`. Simple named property types materialize
the existing request-local `ReflectionNamedType` object. Bounded union and pure
intersection property types materialize request-local `ReflectionUnionType` or
`ReflectionIntersectionType` objects whose `getTypes()` entries are
`ReflectionNamedType` objects. Runtime typed-property enforcement reuses the
current scalar coercion and class/interface relationship checks for those
bounded property type shapes. Property file/line metadata, attributes and exact
docblock association across unusual trivia, parenthesized DNF property types,
exact PHP union scalar coercion preference rules, reference/COW interactions,
and native lowering remain unsupported.
`ReflectionClass::getTraitNames()` and `getTraits()` read the direct trait-name
list already stored on runtime class metadata by the trait-composition pass
for user classes, and direct trait-body `use` declarations for user traits.
`getTraits()` wraps each direct user trait name in the same request-local
`ReflectionClass` state used by direct trait reflection. Trait reflection also
expands simple methods imported from direct trait-body `use` declarations for
`getMethod()`, `getMethods()`, and `hasMethod()`, using the reflected trait as
the declaring class for that bounded metadata slice. The slice intentionally
does not add built-in/internal trait catalogs, exact adapted trait-method
ordering, recursive conflict/adaptation edge cases, or native lowering.
`get_declared_classes()` lists classes and unit enums declared in the current
parsed program;
`get_declared_interfaces()` lists interfaces declared in the current parsed
program; `get_declared_traits()` lists top-level traits declared in the current
parsed program, including traits that contain currently supported public
instance methods.
`get_called_class()` is a zero-argument runtime builtin that reads the
interpreter's called-class context in current instance and static method calls;
outside method or static class context it fails with a stable unsupported-call
diagnostic.
`spl_object_id($object)` accepts current object values and returns a
process-local handle id; non-object inputs fail with the current stable
type-boundary diagnostic.
`spl_object_hash($object)` accepts current object values and returns a stable
current-subset hash derived from the handle id; exact system PHP hash formatting
is not claimed yet, and non-object inputs fail with the current stable
type-boundary diagnostic.
Missing properties, non-object targets, and non-public properties outside the
current private/protected method context still produce stable runtime
diagnostics for normal reads/writes. Public, same-class private, and protected
same-class/child instance methods can execute through `phpc run` with `$this`
bound to the receiver object handle, and inherited public constructors execute
during child instantiation. Protected constructors can execute from same-class
or child-class method context. Explicit
`parent::method(...)` and `parent::__construct(...)` calls execute from active
instance method/constructor context against the current class's parent chain
with the current `$this` object. `self::method(...)` calls execute from active
instance method/constructor context against the current class and inherited
method chain with the current `$this` object. Objects do not enforce
non-public property visibility beyond same-declaring-class private access and
class/ancestor protected access for plain read/write, `isset`/`empty`,
read-modify-write, and null-coalescing forms, compatible public/protected
property redeclarations sharing one runtime slot, or full constructor
visibility.
Objects do not expose reflection, implement
dynamic method/property names, broader `parent::`/`self::`/`static::`,
typed/default property compatibility, broader inheritance/constructor
semantics, `__clone` dispatch, destructor behavior beyond the current normal
shutdown public/no-argument slice, handle reuse behavior, or exact PHP
lifecycle behavior.
Named static method syntax through `ClassName::method(...)`,
`$object::method(...)`, `$className::method(...)`, `self::method(...)`,
`parent::method(...)`, and `static::method(...)` executes declared or inherited
visible static methods in the interpreter without binding `$this`. Dynamic
receivers through `$object::method(...)` and `$className::method(...)` use the
receiver class as the called-class context; `static::method(...)` resolves
through the active called class and forwards that called-class context into
nested calls.
`ClassName::class` returns the source-spelled class string, `self::class` and
`parent::class` resolve from the active declaring class context, and
`static::class` resolves from the active called-class context in current
instance and static method execution. Direct `ClassName::CONST`, `self::CONST`,
`parent::CONST`, and late-bound `static::CONST` resolve declared or inherited
class constants through runtime class metadata in the interpreter; typed
constants, multiple constants in one declaration, and broader dynamic
class-constant string lookup beyond loaded `ClassName::CONST` names remain
unsupported. Direct `ClassName::$prop`, `self::$prop`, `parent::$prop`,
and late-bound `static::$prop` resolve untyped static properties through
interpreter-owned class-level storage, initialize from the current
constant-expression default subset or `null`, and support direct reads/writes,
compound assignment, pre/post increment/decrement, `isset`, `empty`, `??`,
`??=`, and stable runtime diagnostics for PHP-forbidden static-property
`unset(...)`; typed static properties, dynamic names, storage-removing
static-property unset, and top-level `static::$prop` execution remain
unsupported.
Native lowering
rejects class declarations, inheritance metadata, class-name constants, class
constants, static properties, object instantiation, and object metadata
builtins with a specific object/class codegen diagnostic. Instance
object-property reads/writes, including dynamic property-name and
magic-property forms, are rejected with a separate object-property diagnostic
that names missing native object layout, property tables/slots, visibility,
magic property hooks, dynamic property policy, references/copy-on-write, and
exact native object-property errors. The narrow direct string-name
metadata-exists false-folding slice,
string/string `property_exists`/`method_exists` false-folding slice, and
string/string `is_a`/`is_subclass_of` false-folding slice remain the only
native object/class metadata exceptions. Native method-call lowering
separately rejects instance, named static, object/static-receiver, `self::`,
`parent::`, and late-static `static::` calls until generated code has native
method lookup, receiver resolution, `$this` and late-static-binding context,
argument/arity diagnostics, visibility, references/copy-on-write, and exact
native method-call errors. See `docs/OBJECT_MODEL.md` for the named
unsupported edge cases.

## Include/Require Resolution Design

The first executable include/require slice is now a narrow `require path;`,
`require_once path;`, `include path;`, and `include_once path;` subset for
local files in statement and expression position. It uses these rules:

- the interpreter carries the current file path in runtime execution context
- only paths that evaluate to PHP strings are accepted
- absolute paths resolve directly
- relative paths search the current `include_path` string managed by
  `get_include_path()` and `set_include_path()` before falling back to the
  directory of the file containing the construct; the default include path is
  `"."`, path-list entries use `PATH_SEPARATOR`, and empty entries are treated
  as `.`
- bounded local absolute `file://` URLs with an empty host or `localhost`
  percent-decode the UTF-8 path portion and map to the referenced local path
  for include/require execution and `_once` de-duplication
- included files are parsed as PHP files with `<?php`, register top-level
  function/class declarations into the active interpreter, and execute in the
  caller scope
- successful local include/require reads populate the bounded request-local
  realpath cache entry for the resolved target path, matching the same
  empty/non-empty and clear behavior exposed by `realpath_cache_get()` and
  `realpath_cache_size()`
- `require_once` and `include_once` de-duplicate by resolved local file,
  including files first loaded through non-once `require` or `include`
- top-level `return` in an included file returns to the including file
- statement forms ignore the include return value
- expression forms return the included file's top-level return value, `1` for
  normal completion, or `true` when a `_once` construct skips an already loaded
  file
- missing local `include` and `include_once` reads emit two bounded
  `E_WARNING` diagnostics through the current error-handler stack or stderr
  fallback, return `false` in expression position, continue execution, and do
  not mark the missing file as loaded for `_once` de-duplication
- missing local `require` and `require_once` reads emit the same bounded
  `E_WARNING` pair, then append a bounded PHP-shaped fatal stderr line, set
  the interpreter exit signal to `255`, and stop subsequent statement
  execution for both statement and expression forms
- native lowering rejects include/require until file loading, scope effects,
  and return-value behavior have explicit lowering support

Unsupported include/require behavior remains: process-current-working-directory
behavior beyond the default `"."` include path entry and fallback used when no
source file is available, failed-include realpath-cache side effects,
malformed `file://` percent escapes, decoded NUL bytes, non-UTF-8
percent-decoded paths, non-local `file://` hosts, `phar://`, URL includes,
autoload interaction, opcache behavior, declaration-order dependencies such as
a required file declaring `class Child extends Base` only after requiring the
base class, exact source mapping for declarations after include, and exact
PHP warning-vs-fatal text, fatal `Error` object/stack trace shape,
shutdown/destructor ordering after fatal, and broader recovery details.

## Eval Fallback Design

`eval` is reserved by the lexer/parser today and rejected with a stable parse
diagnostic before execution. The first executable `eval` slice should use these
rules:

- parse direct `eval(<expr>)` as a special language construct, not as an
  ordinary function or dynamic callable
- require exactly one argument and evaluate that argument in the caller scope
- accept only string-valued code for the first slice
- parse the evaluated string with a dedicated eval-fragment parser entry point
  that reads a statement list without requiring a `<?php` opening tag
- execute the resulting statements against the caller's current symbol table, so
  assignments affect the same local or top-level scope that called `eval`
- let `return` inside the evaluated fragment produce the `eval(...)` expression
  value; falling off the end should produce `null`
- keep native lowering rejecting `eval` until parser re-entry, source mapping,
  caller-scope effects, and return behavior have explicit lowering support

Initial unsupported eval behavior remains: non-string eval arguments, exact
`ParseError` object semantics, source mapping for diagnostics inside evaluated
strings, functions/classes declared from evaluated code, nested eval,
include/require inside eval, references/copy-on-write interactions,
`GLOBALS`/superglobal behavior, namespaces/use declarations, opcache behavior,
and PHP's exact warning/fatal recovery details.

## Attribute Boundary

PHP attributes are currently syntax-only metadata for simple no-argument
`#[Name]` blocks; the lexer skips them before the parser sees the surrounding
declaration. Attribute blocks with constructor-style arguments such as
`#[Route('/wp-json/demo')]` stop at a dedicated lex diagnostic because
attribute argument evaluation, reflection data, target validation,
namespace-aware attribute name resolution, repeated-attribute rules,
references/copy-on-write behavior, and native lowering do not exist yet.
Ordinary `#` comments remain comments, including `# [` with whitespace before
the bracket.

## Cast Boundary

Cast expressions are AST-backed, but execution is intentionally limited to the
current scalar/null/array runtime value model. `(string)` handles scalar/null
values through the runtime echo-string conversion boundary. `(int)`/`(integer)`
handles scalar/null values through a narrow integer-cast policy for WordPress
bootstrap parsing and focused fixtures, including bounded leading-numeric
string prefixes. `(array)` handles `null`, scalars, and already-array values
only. Object-to-array property materialization, resources, exact PHP
warning/recovery behavior for leading-numeric strings, numeric grammar outside
the current prefix scanner, non-finite or out-of-range float behavior, and
native cast lowering remain explicit boundaries.

## Exception Boundary

Exception syntax is reserved by the lexer/parser today, but the boundary is no
longer a single parse-only category. The runtime seeds a metadata-only
`Exception` class so class lookup, no-argument instantiation, and user classes
extending `Exception` use the same object metadata table as declared classes.
It does not model `Throwable`, constructor state, exception methods, stack
traces, or unwinding.

Statement-form `throw expr;` and `try`/`catch`/`finally` blocks build AST nodes
so guarded WordPress compatibility code can parse and be skipped by normal
control flow. If execution reaches a throw statement, the interpreter reports a
stable unsupported runtime boundary without evaluating the throw operand. If
execution reaches a try block, the interpreter executes the normal no-throw
body path, skips catch bodies when no exception is thrown, and runs finally
bodies after normal try completion. Reached throws and other runtime errors do
not unwind into catch/finally handling in this slice.

Throw expressions, malformed try blocks, and standalone `catch`/`finally`
remain parse boundaries. Native lowering rejects `throw` statements before
emitting LLVM IR or assembly until `Throwable`/`Exception` objects, stack
unwinding, catch/finally dispatch, stack traces, and exact native error behavior
have explicit runtime and IR semantics. Native lowering rejects
`try`/`catch`/`finally` blocks through a dedicated boundary until catch type
matching, catch variable binding, finally execution during normal and
exceptional control flow, references/copy-on-write, stack traces, and exact
native try-block diagnostics exist.

## Match Expression Boundary

PHP 8 `match` expressions are reserved by the lexer/parser today and rejected
with a stable parse diagnostic before execution. They do not build AST nodes yet
because expression-form branching needs explicit semantics for strict arm
matching, default arms, exhaustiveness errors, thrown expressions inside arms,
value evaluation order, and exact native error objects. Native lowering must
continue rejecting `match` until those runtime and IR semantics exist.

## Goto Boundary

`goto target;` statements and `target:` labels build AST nodes for the current
interpreter path. Execution resolves labels in the active statement list and
lets nested statements propagate jumps outward, which covers the WordPress
UTF-8 scanner's forward error-path labels without pretending to implement every
PHP jump rule.

Exact PHP compile-time validation, duplicate label diagnostics, jumps into
nested blocks, cross-function jumps, included-file label boundaries,
interaction with future `finally` execution, and native control-flow lowering
remain explicit unsupported zones.

## Heredoc/Nowdoc Boundary

Heredoc and nowdoc syntax is tokenized directly in the lexer for the current
unindented identifier-label subset. Heredoc reuses the existing double-quoted
interpolation parts, while nowdoc emits a literal string. The lexer trims the
line ending immediately before the terminator to match PHP's runtime value.
Indentation stripping, broader quoted-label forms, malformed-label recovery,
exact diagnostics, and native lowering remain explicit boundaries.

## Fixture Tests

Fixture tests are stored as `.php` files with sibling `.stdout`, `.stderr`, and
`.exit` files. The runner strips one final editor newline from `.stdout` and
`.stderr` fixtures. A fixture that needs to assert an actual trailing newline
should include a blank final line.

When `phpc` intentionally differs from system PHP, a sibling `.phpc-only` marker
keeps the fixture in the normal runner while skipping optional system PHP
comparison.

## Extension Model

Zend extension loading is not an early target. Selected extensions will be
implemented as runtime modules with documented dependencies and semantic gaps.
Until that exists, `extension_loaded()` uses a bounded compiler/runtime
compatibility registry. It is intentionally just enough for current WordPress
bootstrap requirement checks and does not claim host extension support,
extension functions/constants, extension versions, or dynamic loading.
The current `mysqli_connect()` path is a placeholder-handle boundary used to
get past WordPress-shaped procedural connection code; it does not mark the
`mysqli` extension loaded and does not provide executable host database
behavior. PDO is currently a visibility/metadata boundary only:
`pdo`/`pdo_mysql` extension names, `PDO`/`PDOStatement` class metadata, and a
bounded public integer `PDO` constant catalog for common error-mode,
fetch-mode, and MySQL init-command checks are exposed, while `new PDO(...)`
remains an explicit unsupported host-database boundary.
