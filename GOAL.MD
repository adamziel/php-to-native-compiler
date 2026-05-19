# Goal: Full PHP and WordPress Compatibility

This project is a real PHP-to-native compiler effort in stable Rust. The
long-term target is full PHP compatibility and enough WordPress compatibility
to run core, common plugins, themes, and request flows honestly.

That target includes the hard parts: references, copy-on-write, dynamic
lookup, magic methods, object lifecycle, include/autoload state, standard
library and extension behavior, request/SAPI state, database and filesystem
integration, and native runtime support.

## What We Have Been Doing

The work so far has been building compatibility substrate, not randomly adding
stubs:

- A real lexer/parser and AST for the supported PHP subset.
- `phpc run`, an interpreter/runtime path with PHP-shaped boxed values.
- `phpc compile`, a narrower native path that emits LLVM IR/assembly only for
  supported lowering and rejects unsupported PHP constructs with explicit
  diagnostics.
- Fixture and system-PHP comparison tooling so behavior is checked instead of
  guessed.
- Bounded runtime subsystems for arrays, functions, objects, callbacks,
  includes, superglobals, sessions, cookies, headers, streams, filesystem
  probes, MySQLi/`wpdb` placeholder islands, and WordPress bootstrap smokes.
- Many reference/COW bridges for direct variables, arrays, object properties,
  callbacks, `foreach`, magic properties, `ArrayAccess`, and copied buckets.

This is useful foundation work. It is not full PHP support, and it is not real
WordPress support yet.

## Current Evidence

The current in-progress implementation frontier is Lane 2200:

- Lanes 2196 through 2200 move the stored `call_user_func_array()` argument
  path onto the same source-aware stored-root model used by reference slots.
  Argument arrays stored behind visible object properties, dynamic property
  names, and nested object-property array roots can now expose copied-source
  entries to supported callbacks inside magic/`ArrayAccess` method bodies.
  When a reference-return callback receives a value-copy array and returns a
  selected copied leaf by reference, the runtime promotes the proven source
  object-property leaf to a `PhpReferenceCell` instead of binding the returned
  reference to a stale local copy. Focused system-PHP coverage proves visible
  `__get()` and public `ArrayAccess::offsetGet()` holder-property argument
  arrays, dynamic holder properties, nested holder property arrays, and a
  by-value callback return. This remains bounded to supported interpreter
  syntax, visible object-property holder roots, concrete copied-source paths,
  and selected reference-backed leaves; it does not claim arbitrary callback
  containers, unsupported method-body syntax, whole-array reference identity,
  exact diagnostics, or native reference/string COW lowering.

- Lanes 2189 through 2195 close two higher-order method-body COW gaps.
  By-value `ArrayAccess::offsetGet()` and visible `__get()` results that are
  plain arrays can now be scanned for concrete `ArrayAccess` objects in
  intermediate buckets; deeper keyed writes, append writes, and covered
  reference-source bindings route into that inner object while preserving the
  PHP detached-parent/no-backing-write behavior for the overloaded outer
  value. Missing instance-method dispatch through `__call()` now has a
  source-aware path, so supported `offsetGet()` and `__get()` bodies such as
  `return $this->load($name);` preserve copied-array provenance returned by
  `__call()`. Focused system-PHP coverage proves ArrayAccess-returned and
  magic-returned arrays containing inner `ArrayAccess` objects for keyed,
  append, and by-reference paths, plus source-aware `__call()` returns from
  `ArrayAccess::offsetGet()` and `__get()`. This remains bounded to supported
  interpreter syntax, concrete object handles visible in returned arrays,
  supported `__call()` targets, and current copied-source propagation; it
  does not claim untracked dynamic callback containers, arbitrary unsupported
  PHP syntax, exact diagnostics, or native reference/string COW lowering.

- Lanes 2184 through 2188 extend the same concrete-container COW model to
  local and global array values, not just object-property arrays. Supported
  magic/`ArrayAccess` method bodies can now copy an array property into a
  local, or write through a string-keyed `$GLOBALS` array, and if an
  intermediate bucket contains an `ArrayAccess` object the runtime forwards the
  remaining keyed or append path into the existing source-aware `offsetSet()`
  machinery. Focused system-PHP coverage proves magic `__set()` local-copy
  keyed and append stores, outer `ArrayAccess::offsetSet()` local-copy stores,
  `$GLOBALS` array-container stores, and direct `$GLOBALS` `ArrayAccess` append
  stores while preserving selected reference-backed leaves and detaching plain
  copied leaves. This is still bounded to supported interpreter syntax,
  concrete array buckets/objects visible at execution time, and current
  copied-source propagation; it does not claim arbitrary unsupported PHP
  syntax, arbitrary side-effect recovery, untracked dynamic containers, exact
  diagnostics, or native reference/string COW lowering.

- Lanes 2179 through 2183 address two value-model COW holes instead of adding
  another method-name special case. Direct alias variables passed to
  by-reference parameters now promote their array-offset alias to a
  `PhpReferenceCell`, bind the caller-visible variable to that cell, and avoid
  stale path writeback, so replacing a parent array through one by-reference
  parameter does not reroute another child reference into the replacement
  array. Supported magic/`ArrayAccess` method-body writes can also traverse
  object-property arrays whose selected bucket contains an `ArrayAccess`
  object, forwarding keyed and append stores into the existing source-aware
  `offsetSet()` propagation path. Focused system-PHP coverage proves
  child-reference detachment after whole-root replacement, magic `__set()`
  keyed/dynamic/append stores through array-contained `ArrayAccess` objects,
  and outer `ArrayAccess::offsetSet()` bodies forwarding into inner
  array-contained `ArrayAccess` objects. This is still bounded to promoted
  reference cells, supported interpreter syntax, concrete tracked array
  buckets/objects, and current copied-source propagation; it does not claim
  arbitrary unsupported PHP syntax, arbitrary side effects, untracked dynamic
  containers, exact diagnostics, or native reference/string COW lowering.

- Lanes 2172 through 2178 broaden supported magic/`ArrayAccess` method-body
  dispatch while preserving copied-array COW provenance. Dynamic calls where
  the callee expression evaluates to a PHP array callable now reuse the
  existing direct callable-array invocation path, including the source-aware
  variant. `ReflectionMethod::invoke()` and `invokeArgs()` now re-enter
  supported user methods through source-aware invocation instead of dropping
  copied-source metadata. Focused system-PHP coverage proves
  `ArrayAccess::offsetSet()` copied-source storage through property-held and
  indexed dynamic array callables, by-value `ArrayAccess::offsetGet()` returns
  through dynamic array callables and reflection, magic `__set()` copied-source
  storage through a dynamic array callable, magic `__get()` reflection
  `invokeArgs()` returns, and reflection-dispatched `offsetSet()` helper
  storage. This removes real method-body dispatch holes, but still does not
  claim arbitrary unsupported PHP syntax, untracked dynamic containers, side
  effects that cannot be expressed through the supported interpreter/writeback
  paths, whole-array reference identity, exact diagnostics, or native
  reference/string COW lowering.

- Lanes 2167 through 2171 broaden supported magic/`ArrayAccess` method-body
  assignment targets for copied-source COW propagation. Expression-form
  non-direct holder writes such as `$this->holder->store[$key] = $value`,
  dynamic-property forms such as `$this->holder->$prop[$key] = $value`,
  non-direct append-with-suffix stores such as
  `$this->holder->store["outer"][]["payload"] = $value`, and direct dynamic
  property appends now reach the same object-property array write helpers as
  direct roots. Focused system-PHP coverage proves selected reference-backed
  leaves remain connected while ordinary copied leaves detach. This removes a
  parser/runtime hole in otherwise supported method bodies, but still does
  not claim arbitrary unsupported PHP syntax, untracked dynamic containers,
  whole-array reference identity, exact diagnostics, or native
  reference/string COW lowering.

- Lanes 2162 through 2166 route nested append assignment through the same
  source-aware assignment value path used by keyed stores. Direct nested
  `ArrayAccess` append targets, suffix append targets such as
  `$target["outer"][]["payload"]`, plain nested arrays, and string-keyed
  `$GLOBALS` paths now rehydrate copied-source arrays before storing them, so
  selected reference-backed leaves remain connected while ordinary copied
  leaves detach. Focused system-PHP coverage proves RHS arrays copied from
  direct by-value `ArrayAccess::offsetGet()`, visible magic `__get()`, and
  object-property-held `ArrayAccess` sources. This closes the covered
  append/suffix copied-source provenance gap, but still does not claim
  arbitrary unsupported PHP syntax or side effects inside magic/`ArrayAccess`
  methods, untracked dynamic containers, whole-array reference identity, exact
  diagnostics, or native reference/string COW lowering.

- Lanes 2157 through 2161 carry PHP's non-static closure binding into
  closures created inside supported magic/`ArrayAccess` method bodies. A
  closure created while `$this` is in scope now records the receiver and
  class/called-class context, and direct closure calls,
  `call_user_func($closure, ...)`, positional `call_user_func_array()`, and
  reference-return/source paths execute with that bound context. Focused
  system-PHP coverage proves by-value `ArrayAccess::offsetGet()` and visible
  `__get()` bodies where a bound closure overwrites the backing property
  after copying the old bucket, callback-dispatched bound closures through
  `call_user_func()` and `call_user_func_array()`, and
  `ArrayAccess::offsetSet()` storing a copied-source payload through a bound
  closure. This removes a real method-body callback hole, but it remains
  bounded to supported interpreter syntax and existing callable dispatch
  shapes; `Closure::bind`/`bindTo`, arbitrary unsupported PHP syntax,
  untracked dynamic containers, whole-array reference identity, exact
  diagnostics, and native reference lowering remain unsupported.

- Lanes 2152 through 2156 extend the detached object-property writeback model
  to method-local copied buckets returned by by-value `ArrayAccess::offsetGet()`
  and visible `__get()` bodies. When supported bodies copy a backing
  object-property array into a local, then overwrite the backing property
  directly, through a helper method, or through direct
  `call_user_func([$this, ...])`, the returned local copy now keeps selected
  reference-backed leaves tied to the detached caller alias while the new
  backing property stays separate. Focused system-PHP coverage proves direct,
  helper, and callback `offsetGet()` overwrites plus direct and callback
  magic `__get()` overwrites. This remains bounded to supported interpreter
  syntax, tracked object-property copy sources, and direct caller-scope
  callback dispatch; untracked dynamic containers, whole-array reference
  identity, exact diagnostics, and native reference lowering remain
  unsupported.

- Lanes 2148 through 2151 add a method-scope detached object-property path
  ledger for supported nested magic/`ArrayAccess` method bodies. When
  `offsetSet()` or `__set()` delegates to a helper method or a direct
  `call_user_func([$this, ...])` callback that overwrites a backing
  object-property array, stale caller aliases now detach to their pre-call
  fallback cells even if the overwritten path exists again afterward. Focused
  system-PHP coverage proves helper and callback overwrite detaches for both
  `ArrayAccess::offsetSet()` and magic `__set()`. This is a high-leverage
  side-effect/writeback step, but it remains bounded to supported method
  bodies and direct caller-scope callback dispatch; untracked dynamic
  containers, arbitrary unsupported PHP syntax, whole-array reference
  identity, exact diagnostics, and native reference lowering remain
  unsupported.

- Lanes 2144 through 2147 carry copied-array source metadata into supported
  `ArrayAccess::offsetSet()` setter calls reached through visible
  object-property roots, non-direct holder roots, dynamic property roots, and
  magic `__get()` roots that return an `ArrayAccess` object. When those
  setters store a copied object-property array through supported local/branch
  bodies, later by-value `offsetGet()` consumers preserve reference-backed
  leaves and keep ordinary copied leaves detached. Focused system-PHP
  coverage proves direct property-held, non-direct property-held, dynamic
  property-held, and magic-returned `ArrayAccess` keyed stores. This remains
  bounded to supported interpreter syntax and tracked copied-array sources;
  broader setter/storage metadata after append/suffix paths, helper/callback
  containers, untracked dynamic containers, whole-array reference identity,
  exact diagnostics, and native reference lowering remain unsupported.

- Lanes 2139 through 2143 route immediate by-value overloaded reference
  consumers through the source-aware method-body executor. Public by-value
  `ArrayAccess::offsetGet()` and visible by-value `__get()` bodies whose
  supported code reaches a backed object-property array copy through control
  flow, locals, or local references now preserve selected reference-backed
  leaves for nested keyed writes, reference assignment to returned leaves, and
  magic root reference assignment. Ordinary copied leaves still detach, and
  PHP's indirect-modification/no-backing-write behavior remains. Focused
  system-PHP coverage proves ArrayAccess and magic keyed writes, ArrayAccess
  and magic nested reference assignment, and magic root reference assignment.
  This is still bounded to supported interpreter syntax and tracked
  object-property sources; untracked dynamic containers, stored future-path
  metadata after broader setter bodies, whole-array reference identity, exact
  diagnostics, and native reference lowering remain unsupported.

- Lanes 2134 through 2138 route the covered magic/`ArrayAccess`
  side-effecting method dispatches through caller-scope alias transfer instead
  of executing them as isolated helper calls. Named `__set()`, visible
  `__isset()`, direct `ArrayAccess::offsetSet()`,
  `ArrayAccess::offsetUnset()`, and `ArrayAccess::offsetExists()` bodies now
  detach caller aliases when supported method-body code overwrites a tracked
  object-property container, including non-public backing properties, while
  preserving nested reference leaves in detached aliases. Focused system-PHP
  coverage proves magic assignment, magic observation, ArrayAccess keyed
  assignment, ArrayAccess unset, and ArrayAccess `isset()` shapes. This is
  still bounded to supported interpreter syntax and tracked object-property
  containers; untracked dynamic containers, arbitrary unsupported PHP
  side-effect syntax, whole-array reference identity, exact diagnostics, and
  native reference lowering remain unsupported.

- Lanes 2129 through 2133 broaden the supported magic/`ArrayAccess`
  method-body value model for array literals with reference elements.
  Value-position array literals inside supported method bodies can now promote
  covered object-property array offsets, including `$this` context roots and
  non-public backing properties, to runtime reference cells. Public by-value
  `ArrayAccess::offsetGet()` and visible `__get()` bodies can return
  `array("leaf" => &$this->store[$key]["leaf"])` directly or through helper
  methods, and nested writes through the returned overloaded value mutate the
  original reference leaf while ordinary copied leaves remain detached.
  Focused system-PHP coverage proves direct and helper `offsetGet()` returns,
  private-backed direct `__get()` returns, helper `__get()` returns, and
  helper-based `__set()` payload storage. This remains bounded to supported
  interpreter syntax and covered object-property array offsets; side-effecting
  dynamic property/key expressions, untracked dynamic containers,
  whole-array reference identity, exact diagnostics, and native reference
  lowering remain unsupported.

- Lanes 2126 through 2128 extend the method-scope alias transfer used by the
  previous public-property detach work to covered non-public object-property
  leaf aliases. Private and protected `$this->property` array-offset aliases
  imported from a caller can now detach when a direct method body or public
  `__unset()` body unsets the containing property, preserving the live
  reference leaf after control returns. Focused system-PHP coverage proves
  private method-body unset, private magic `__unset()`, and protected
  inherited method-body unset shapes. This remains bounded to selected leaf
  aliases and supported interpreter syntax; non-public ancestor-container
  alias detach on property overwrite, untracked dynamic containers,
  arbitrary future-path metadata recovery, whole-array reference identity,
  exact diagnostics, and native reference lowering remain unsupported.

- Lanes 2121 through 2125 preserve reference-cell identity when covered
  whole-container detach paths cut aliases loose. Direct array-root unsets,
  visible object-property unsets and overwrites, object-root unsets, direct
  method-body `unset($this->publicProperty)`, and public `__unset()` bodies
  now detach aliases to existing runtime reference cells instead of cloning
  plain fallback values, so aliases to ancestor containers keep nested
  reference leaves live. Focused system-PHP coverage proves static roots,
  visible property roots, object roots, direct method bodies, and public
  magic `__unset()` bodies. This remains bounded to aliases imported through
  visible public object-property roots and supported interpreter syntax;
  non-public ancestor-container alias detach, untracked dynamic containers,
  arbitrary future-path metadata recovery, whole-array reference identity,
  exact diagnostics, and native reference lowering remain unsupported.

- Lanes 2117 through 2120 broaden mixed nested `ArrayAccess` observation and
  unset chains beyond the previous one-key bridge. Direct `ArrayAccess`
  variables, visible property-held `ArrayAccess` objects, and magic `__get()`
  roots that return `ArrayAccess` objects now recursively execute
  `offsetExists()`, `offsetGet()`, and `offsetUnset()` for supported nested
  `isset`, `empty`, and `unset` paths. Public by-reference `offsetGet()`
  roots that return backing arrays can also unset nested array leaves through
  the returned reference after executing the method body. Focused system-PHP
  coverage proves direct, property-held, magic-provided, and by-reference
  array-returning nested chains. This remains bounded to supported
  interpreter syntax, direct/property-held/magic roots, and nested
  `ArrayAccess` or by-reference array returns; untracked dynamic containers,
  arbitrary future-path metadata recovery, whole-array reference identity,
  exact diagnostics, and native reference lowering remain unsupported.

- Lanes 2112 through 2116 execute covered magic array-offset observation and
  unset method bodies. Missing named and dynamic magic-property array-offset
  `isset`/`empty` forms now call `__isset()` when present, then `__get()` when
  PHP would inspect the returned value; returned arrays are inspected as
  detached values, and returned `ArrayAccess` objects call `offsetExists()`
  and `offsetGet()` for the covered one-key shape. Dynamic visible properties
  that hold an `ArrayAccess` object can now dispatch `unset($holder->$name[$key])`
  to `offsetUnset()`, and missing magic properties whose visible `__get()`
  returns an `ArrayAccess` object or a by-reference array can dispatch covered
  one-key unsets while preserving reference-backed side effects. Focused
  system-PHP coverage proves named and dynamic magic offset observation,
  magic-returned `ArrayAccess` observation/unset, dynamic property-held
  `ArrayAccess` unset, and by-reference magic array-offset unset. This remains
  bounded to supported interpreter syntax, direct named/dynamic property
  roots, and one-key `ArrayAccess` unset/observation shapes; untracked
  containers, deeper mixed chains, whole-container unsets with live reference
  leaves, whole-array reference identity, exact diagnostics, and native
  reference lowering remain unsupported.

- Lanes 2107 through 2111 execute the remaining covered dynamic-property
  observation magic and property-held `ArrayAccess` methods. Dynamic
  `isset($object->$name)` and `empty($object->$name)` now route through
  visible `__isset()` and, for `empty`, visible `__get()` using the evaluated
  dynamic property name; dynamic `unset($object->$name)` now calls visible
  `__unset()` when the property is missing. Dynamic properties that visibly
  hold an `ArrayAccess` object now call `offsetExists()` for `isset` and
  `offsetExists()` plus `offsetGet()` for `empty`. Focused system-PHP
  coverage proves these supported method bodies execute side effects that
  mutate reference-backed leaves. This remains bounded to supported
  interpreter syntax and direct dynamic-property-name forms; broader
  non-direct dynamic containers, container unsets that hold live reference
  leaves, whole-array reference identity, exact diagnostics, and native
  reference lowering remain unsupported.

- Lanes 2104 through 2106 route dynamic property assignment through visible
  `__set()` before falling back to dynamic public-property creation, matching
  the named-property magic path and preserving COW value cells through
  supported `__set()` bodies. Dynamic `$object->$name = ...` now rehydrates
  array-literal reference elements and copied reference-cell arrays before
  invoking `__set($name, $value)`, so bodies that store through local keys
  such as `$key = $name; $this->store[$key] = $value;` keep selected
  reference-backed leaves. Focused system-PHP coverage proves dynamic
  `__set()` with a literal reference array, a stored reference array, and a
  by-value `ArrayAccess::offsetGet()` RHS that returns an array with reference
  cells. This remains bounded to supported interpreter syntax, direct dynamic
  property names, and recoverable method-body effects; untracked dynamic
  containers, arbitrary future-path metadata recovery, whole-array reference
  identity, exact diagnostics, and native reference lowering remain
  unsupported.

- Lanes 2101 through 2103 remove one remaining static fast path from
  append-style `ArrayAccess::offsetSet(null, ...)` writes and add focused
  evidence that supported magic/`ArrayAccess` method bodies execute their
  side effects before COW reference-cell propagation is observed. Direct
  `ArrayAccess` append assignment now invokes `offsetSet()` instead of
  mutating the inferred backing bucket directly, then reuses the proven
  backing metadata only for post-call reference/provenance repair. Focused
  system-PHP coverage proves append `offsetSet(null, ...)` side effects,
  by-value `offsetGet()` side effects during nested reference-cell mutation,
  and visible by-value `__get()` side effects during nested reference-cell
  mutation. This remains bounded to supported interpreter syntax and
  recoverable method-body effects; untracked dynamic containers, arbitrary
  future-path metadata recovery, whole-array reference identity, exact
  diagnostics, and native reference lowering remain unsupported.

- Lanes 2096 through 2100 extend reference-return callback value-copy COW to
  actual reference cells carried inside by-value arguments, even when no
  copied-source metadata exists. `call_user_func()` and
  `call_user_func_array()` callbacks whose reached parameter is declared by
  reference still use PHP's warning/no-reference value-passing shape, but if
  the local value copy contains a reference-backed child slot then a returned
  reference such as `return $param["ref"];` binds to that cell. Literal and
  stored `call_user_func_array()` argument arrays are covered, and
  by-reference variadic rest parameters map positional entries into the local
  rest array so `return $values[0]["ref"];` and later rest offsets preserve
  selected reference leaves. Focused system-PHP coverage proves
  `call_user_func()`, literal `call_user_func_array()`, stored
  `call_user_func_array()`, variadic rest index zero, and variadic rest after
  a fixed argument. This remains bounded to supported callback bodies and
  value arrays that already carry runtime reference cells; arbitrary callback
  side effects, untracked dynamic containers, whole-array reference identity,
  exact diagnostics, and native reference lowering remain unsupported.

- Lanes 2091 through 2095 preserve PHP's by-value overloaded nested-write
  behavior when supported `ArrayAccess::offsetGet()` or visible `__get()`
  bodies return arrays that contain real reference cells. Nested keyed writes
  and nested appends still emit the overloaded indirect-modification notice
  and keep ordinary copied leaves detached, but writes that land on a
  reference-backed leaf update that referenced value. The same write helper
  rehydrates array-literal reference elements before assigning through a
  returned reference leaf, and by-value outer `ArrayAccess` results that are
  themselves `ArrayAccess` objects now hand nested keyed writes to the inner
  object's supported `offsetSet()` path. Focused system-PHP coverage proves
  direct `ArrayAccess` keyed writes, visible magic `__get()` keyed writes,
  direct `ArrayAccess` appends, visible magic `__get()` appends, nested
  array-literal reference assignment through a returned leaf, and an
  outer-to-inner by-value `ArrayAccess` object chain. This remains bounded to
  supported interpreter syntax, current reference-cell value paths, and
  covered nested write forms; untracked dynamic containers, whole-array
  reference identity, exact diagnostics, arbitrary future-path metadata
  recovery, and native reference lowering remain unsupported.

- Lanes 2086 through 2090 move a key part of magic/`ArrayAccess` COW from
  static bridge proofs into the value model. Direct variable assignments from
  array literals with reference elements now materialize real reference cells
  in the stored array after alias metadata is bound. By-value magic/
  `ArrayAccess` method arguments are rehydrated with those reference cells
  before arbitrary supported `offsetSet()`/`__set()` bodies execute, so bodies
  that store through locals such as `$key = $offset; $this->store[$key] =
  $value;` preserve selected reference leaves. Supported `offsetGet()` and
  `__get()` bodies that build and return local arrays containing reference
  elements now return copies with those reference cells intact. Focused
  system-PHP coverage proves direct `offsetSet()` literals, stored
  `offsetSet()` payloads, local-reference `offsetGet()` returns,
  local-reference `__get()` returns, and arbitrary local-key `__set()`
  storage. This is still bounded to supported interpreter syntax and the
  current reference cell/alias metadata model; unsupported PHP syntax,
  untracked dynamic containers, whole-array reference identity, exact
  diagnostics, and native reference lowering remain unsupported.

- Lanes 2081 through 2085 extend the same reference-return value-copy COW
  bridge through `call_user_func_array()` argument containers. Literal and
  direct stored argument arrays that contain covered by-value magic/
  `ArrayAccess` copied arrays now carry copied-source metadata into
  reference-return callbacks while preserving PHP's warning/no-reference
  value-passing shape, so returned child references such as
  `return $param["ref"];` can bind to selected nested reference-backed leaves.
  Focused system-PHP coverage proves literal `ArrayAccess` argument arrays,
  stored `ArrayAccess` argument arrays, stored visible by-value `__get()`
  argument arrays, closure callbacks, and public object array-callable
  callbacks. This remains bounded to traced literal/direct stored containers,
  supported interpreter-body syntax, copied-source metadata, and
  non-variadic value-copy callback arguments; arbitrary PHP syntax, untracked
  dynamic containers, arbitrary `offsetSet()` reference propagation,
  whole-array reference identity, exact diagnostics, and native reference
  lowering remain unsupported.

- Lanes 2076 through 2080 extend the executed by-value magic/`ArrayAccess`
  method-body path into reference-return callees. A reference-return function,
  dynamic string call, or `call_user_func()` callback whose reached parameter
  is declared by reference now receives the overloaded value copy with tracked
  copied-array provenance, so `return $param["ref"];` can return the selected
  nested reference-backed leaf instead of falling back to exact static bridge
  requirements. Focused system-PHP coverage proves public by-value
  `ArrayAccess::offsetGet()` branch-selected local references, visible
  by-value `__get()` branch-selected local references, dynamic string
  reference-return calls, `call_user_func()` string callbacks, and
  `call_user_func()` public object array-callable callbacks. This remains
  bounded to supported interpreter-body syntax, copied-source metadata, and
  non-variadic value-copy callback arguments; arbitrary PHP syntax, untracked
  dynamic containers, arbitrary `offsetSet()` reference propagation,
  whole-array reference identity, exact diagnostics, and native reference
  lowering remain unsupported.

- Lanes 2071 through 2075 remove another exact-static bridge from
  by-reference user-function parameters that receive overloaded by-value
  magic/`ArrayAccess` arrays. The interpreter now executes supported
  `ArrayAccess::offsetGet()` and `__get()` method bodies once and carries the
  returned copied-array provenance into the callee, rather than requiring the
  terminal backing bucket to be proven from a single exact return expression
  before execution. Focused system-PHP coverage proves branch-selected local
  references inside public by-value `offsetGet()`, direct and dynamic
  property-held `ArrayAccess` roots, direct by-value `__get()`, nested
  by-value magic array reads, and parent replacement below the returned magic
  copy. This is still bounded to supported interpreter-body syntax and
  tracked copied-array sources; arbitrary PHP syntax, untracked dynamic
  containers, whole-array reference identity, exact diagnostics, arbitrary
  `offsetSet()` reference propagation, and native reference lowering remain
  unsupported.

- Lanes 2066 through 2070 preserve copied-bucket COW behavior when a
  by-reference user-function parameter receives a covered by-value
  `ArrayAccess::offsetGet()` array result. PHP treats this as an overloaded
  value/no-reference path; the interpreter now mirrors that for direct
  `ArrayAccess` roots, mixed outer-to-inner `ArrayAccess` chains,
  direct property-held roots, side-effect-free dynamic property roots, and
  parent replacement below the returned copy. Focused system-PHP coverage
  proves selected nested reference-backed leaves update the original backing
  reference while ordinary parent/plain-array writes stay local. This is still
  bounded to statically proven terminal buckets; arbitrary side effects,
  unproven dynamic containers, whole-array reference identity, exact
  diagnostics, and native reference lowering remain unsupported.

- Lanes 2061 through 2065 broaden source-aware by-value method bodies to
  preserve copied-array provenance when the returned local was first bound by
  reference to a covered object-property array source. Covered forms include
  visible public `__get()` and ordinary public methods returning `$bucket`
  after `$bucket =& $this->store[$name]`, returning `$items[$name]` after
  `$items =& $this->store`, and side-effect-free dynamic-property bucket
  selection through `$this->{$property}[$name]`. Focused system-PHP coverage
  proves selected nested reference slots remain shared in the returned copy
  while ordinary nested arrays detach. This is still bounded to proven local
  alias bindings; arbitrary side effects, unproven dynamic containers,
  whole-array reference identity, exact diagnostics, and native reference
  lowering remain unsupported.

- Lanes 2056 through 2060 close a proven scalar-parent miss below covered
  by-value `ArrayAccess::offsetGet()` results. Nested keyed writes, nested
  appends, property-held reference targets, and magic-provided reference
  targets now reject `true`, integer, float, and string parents instead of
  silently no-oping. The `false` parent path remains a no-op against the
  backing bucket while emitting PHP's automatic-conversion deprecation.
  Focused runtime-error fixtures pin stable project diagnostics for the
  currently non-catchable fatal paths, and a `false` no-op/deprecation fixture
  compares directly against system PHP. This is still bounded to covered
  by-value magic/`ArrayAccess` paths; arbitrary side effects, complete mixed
  `ArrayAccess` object chains, exact PHP `Throwable` objects/diagnostics,
  whole-array reference identity, and native reference lowering remain
  unsupported.

- Lanes 2051 through 2055 preserve copied-bucket COW behavior for bounded
  `call_user_func()` callbacks whose reached callback parameter is variadic
  and declared by reference. PHP warns and still passes values; the
  interpreter now builds the variadic rest array as values while preserving
  copied-source metadata for each covered rest element path such as
  `$values[0]`. Covered bodies include visible by-value `__get()`, public
  by-value `ArrayAccess::offsetGet()`, string callbacks, closure callbacks,
  public object array-callable callbacks, rest parameters at index zero, and
  rest parameters after a fixed argument. Focused system-PHP coverage proves
  nested writes through selected reference-backed leaves update the original
  backing reference, ordinary nested arrays detach, and a later parent
  replacement stays local to the returned copy. This is still bounded to
  supported interpreter syntax and tracked copied-source roots; arbitrary
  untracked side effects, whole-array reference identity, exact diagnostics,
  and native reference lowering remain unsupported.

- Lanes 2046 through 2050 preserve copied-bucket COW behavior for bounded
  `call_user_func()` callbacks whose reached callback parameter is declared
  by reference. PHP warns and passes a value for these calls; the interpreter
  now mirrors that no-reference call while carrying source-aware copied-array
  metadata into visible by-value `__get()` and public by-value
  `ArrayAccess::offsetGet()` bodies. Covered callback shapes include direct
  string callbacks, dynamic string callbacks, closure callbacks, and public
  object array-callable callbacks. Focused system-PHP coverage proves nested
  writes through selected reference-backed leaves update the original backing
  reference while ordinary nested arrays detach in the returned copy. This is
  still bounded to supported interpreter syntax and tracked copied-source
  roots; arbitrary untracked side effects, whole-array reference identity,
  exact diagnostics, and native reference lowering remain unsupported.

- Lanes 2041 through 2045 route exact by-value magic/`ArrayAccess`
  backing-bucket returns through the same source-aware method executor as
  broader supported method bodies, instead of letting the older exact-return
  fast paths drop copied-bucket provenance. Covered shapes include visible
  by-value `__get()` returning `$this->property[$name]`, returning a whole
  `$this->property` array, feeding a later nested read from the returned magic
  array, returning an inaccessible dynamic `$this->{$name}` property from
  inside `__get()`, and public by-value `ArrayAccess::offsetGet()` returning
  a literal-prefix `$this->property["bucket"][$offset]` bucket. Focused
  system-PHP coverage proves selected reference-backed leaves remain shared
  after the returned copy is mutated while ordinary nested arrays detach. This
  removes a legacy exact-return source-loss path, but it is still bounded to
  supported interpreter syntax and tracked object-property/`ArrayAccess`
  roots; arbitrary untracked side effects, whole-array reference identity,
  exact diagnostics, and native reference lowering remain unsupported.

- Lanes 2036 through 2040 preserve copied-bucket COW provenance across later
  supported method-body container mutations instead of losing it when a local
  array container root is rewritten. Covered by-value magic `__get()`/public
  `ArrayAccess::offsetGet()` bodies now retain tracked copied-source paths
  after scalar nested writes below a copied container, while still removing
  overwritten paths. Focused system-PHP coverage proves selected
  reference-backed leaves remain shared after local array-container movement,
  local object-property container movement, side-effecting helper-selected
  keys, and loop/branch bodies. This is broader than the previous
  return-expression source recovery, but it is still bounded to supported
  interpreter syntax and tracked object-property/`ArrayAccess` roots;
  arbitrary PHP syntax, untracked dynamic containers, whole-array reference
  identity, exact diagnostics, and native reference lowering remain
  unsupported.

- Lanes 2031 through 2035 preserve copied-bucket COW provenance through
  bounded `call_user_func_array()` callback containers whose reached callback
  parameter is by reference. Covered bodies include visible by-value
  `__get()`, public by-value `ArrayAccess::offsetGet()`, literal
  `array(&$bucket)` argument arrays, stored local `array(&$bucket)` argument
  arrays, closure callbacks, and public object array-callable callbacks.
  Focused system-PHP coverage proves callback leaf writes update the selected
  original backing reference and the returned copy keeps that selected
  reference-backed leaf shared while ordinary nested arrays detach. This is
  still not arbitrary callback/container support: dynamic containers that
  cannot be traced to a current-scope literal/local, arbitrary object graphs,
  side-effecting property names or keys, whole-array reference identity, exact
  diagnostics, and native reference lowering remain unsupported.

- Lanes 2026 through 2030 preserve COW behavior for bounded sequential helper
  bodies that first mutate a copied reference-backed leaf and then replace
  that reference's parent before the copied bucket is returned. Covered bodies
  include visible by-value `__get()`, public by-value
  `ArrayAccess::offsetGet()`, visible local object-property containers,
  dynamic local properties, `$this` instance-method helpers, and direct
  closure helpers. Focused system-PHP coverage proves the earlier leaf write
  updates the selected original backing reference while the later parent
  replacement stays local to the returned copy. This is still not arbitrary
  method-body support: arbitrary object graphs, side-effecting property names
  or keys, `call_user_func*` callback containers, whole-array reference
  identity, exact diagnostics, and native reference lowering remain
  unsupported.

- Lanes 2021 through 2025 preserve COW behavior when those visible local
  object-property containers are passed by reference to a helper inside a
  supported by-value magic/`ArrayAccess` method body. Covered bodies include
  visible by-value `__get()`, public by-value `ArrayAccess::offsetGet()`,
  dynamic local properties, `$this` instance-method helpers, and direct
  closure helpers. Focused system-PHP coverage proves helper leaf writes update
  the selected original backing reference while ordinary nested arrays stay
  local to the returned copy. This is not arbitrary method-body support:
  arbitrary object graphs, side-effecting property names or keys,
  `call_user_func*` callback containers, whole-array reference identity, exact
  diagnostics, and native reference lowering remain unsupported.

- Lanes 2016 through 2020 preserve copied-array COW provenance when
  supported by-value magic/`ArrayAccess` method bodies store a proven backed
  bucket inside a visible property of a current-scope local object container
  before returning it. Covered bodies include visible by-value `__get()`,
  public by-value `ArrayAccess::offsetGet()`, dynamically selected local
  object properties, a non-reference helper returning the local property by
  value, and a nested array-literal container stored in the local property.
  Focused system-PHP coverage proves the returned copy keeps selected
  reference-backed nested slots shared while ordinary nested arrays detach.
  This is bounded to visible local object-property containers with
  side-effect-free property/index recovery; arbitrary object graphs,
  side-effecting property names or keys, whole-array reference identity, exact
  diagnostics, and native reference lowering remain unsupported.

- Lanes 2011 through 2015 preserve PHP COW behavior when supported
  by-value magic/`ArrayAccess` method bodies pass a proven copied bucket into
  a by-reference helper that mutates a nested reference-backed leaf before
  returning it. Covered bodies include visible by-value `__get()`, public
  by-value `ArrayAccess::offsetGet()`, direct free-function helpers, `$this`
  instance-method helpers, dynamic string helper calls, and direct closure
  helpers. Focused system-PHP coverage proves the helper's nested-leaf write
  updates the original backing reference, the returned copy keeps that
  selected reference shared, and ordinary nested arrays detach. Sequential
  helper bodies that mutate a copied reference leaf and later replace that
  reference's parent in the same helper, whole-array reference identity, exact
  diagnostics, and native reference lowering remain unsupported.

- Lanes 2006 through 2010 preserve COW detach behavior when supported
  by-value magic/`ArrayAccess` method bodies replace a parent of a live
  copied-bucket reference. Covered bodies include visible by-value `__get()`,
  public by-value `ArrayAccess::offsetGet()`, scalar parent overwrite,
  direct by-reference helper parent replacement, and by-reference closure
  capture parent replacement. Focused system-PHP coverage proves the original
  backing reference keeps its previous value while the replacement stays local
  to the returned copy. Nested-leaf writes through by-reference helper
  parameters, arbitrary helper/callback side effects, whole-array reference
  identity, exact diagnostics, and native reference lowering remain
  unsupported.

- Lanes 2001 through 2005 preserve copied-array COW provenance when supported
  by-value magic/`ArrayAccess` method bodies capture a proven backed bucket
  by reference in a closure. Covered bodies include visible by-value
  `__get()`, public by-value `ArrayAccess::offsetGet()`, direct closure
  invocation, `call_user_func($closure)`, `call_user_func_array($closure,
  $args)`, and a bounded nested-leaf mutation through the captured bucket
  before it is returned. Focused system-PHP coverage proves the returned copy
  keeps selected reference-backed nested slots shared while ordinary nested
  arrays detach. Replacing a parent of a live copied-bucket reference,
  arbitrary capture roots, whole-array reference identity, exact diagnostics,
  and native reference lowering remain unsupported.

- Lanes 1996 through 2000 preserve copied-array COW provenance when supported
  by-value magic/`ArrayAccess` method bodies pass a proven backed bucket
  through a by-reference helper parameter before returning it. Covered bodies
  include direct free-function helpers, `$this` instance-method helpers,
  public by-value `ArrayAccess::offsetGet()` helpers, direct closure helpers,
  and dynamic string helper calls. Focused system-PHP coverage proves the
  returned copy keeps selected reference-backed nested slots shared while
  ordinary nested arrays detach. Writes performed inside the by-reference
  helper through that parameter, `call_user_func()` warning/no-reference
  semantics, `call_user_func_array()` by-reference callback containers,
  whole-array reference identity, exact diagnostics, and native reference
  lowering remain unsupported.

- Lanes 1991 through 1995 preserve copied-array COW provenance when supported
  by-value magic/`ArrayAccess` method bodies hide a proven backed bucket
  inside a local array container before returning it. Covered bodies include
  local array-literal containers, nested local array-literal containers,
  direct keyed local-container writes, nested keyed local-container writes,
  and a stored `call_user_func_array()` argument array built from the
  recovered container slot. Focused system-PHP coverage proves the selected
  reference-backed nested slot stays shared while ordinary nested arrays
  detach. This is still bounded to current-scope local array containers with
  side-effect-free int/string container keys; object containers outside the
  later visible local-property slice, by-reference helper/callback
  parameters, whole-array reference identity, exact diagnostics, and native
  reference lowering remain unsupported.

- Lanes 1986 through 1990 add append-offset expressions in by-reference
  function-argument position for the covered direct array and proven
  `ArrayAccess` backing-bucket bridges. Calls such as
  `mutate($items["x"][])`, `mutate($bag["x"][])`,
  `mutate($box->missing["x"][])`, `mutate($box->{$name}["x"][])`, and
  `mutate($holders["box"]->missing["x"][])` now parse, bind the appended
  slot to the by-reference parameter, materialize selected `false` parents as
  arrays, and write callee mutations back to the original bucket. Append
  expressions outside call-argument positions remain unsupported as reads,
  non-reference call parameters still cannot consume append expressions as
  values, and exact deprecation/diagnostic text and native reference lowering
  remain unsupported.

- Lanes 1982 through 1985 preserve by-value closure-capture provenance inside
  magic/`ArrayAccess` method bodies. When a visible by-value `__get()` or
  public by-value `ArrayAccess::offsetGet()` body assigns a proven backed
  bucket to a local and captures that local with `use ($bucket)`, direct
  closure invocation, `call_user_func($closure)`, and
  `call_user_func_array($closure, $args)` can return the captured copy while
  keeping selected reference-backed nested slots shared and ordinary nested
  arrays detached. This is limited to by-value closure captures whose source
  is proven at closure creation; by-reference captures, capture sources
  hidden inside arbitrary array containers, arbitrary closure side effects,
  whole-array reference identity, exact diagnostics, and native reference
  lowering remain unsupported.

- Lanes 1977 through 1981 broaden by-value magic/`ArrayAccess`
  method-body copied-array provenance through callback/helper bodies. Visible
  by-value `__get()` and public by-value `ArrayAccess::offsetGet()` bodies
  can now preserve proven backed-array copy sources through dynamic string
  helper calls, direct closure helper calls, closure callbacks via
  `call_user_func()`, closure callbacks via literal or traced-local
  `call_user_func_array()` argument arrays, and public object array-callable
  helpers via `call_user_func()`/`call_user_func_array()`. Focused
  system-PHP coverage proves the reference-backed nested slot stays shared
  while ordinary nested arrays detach. Dynamic `call_user_func_array()`
  containers that cannot be traced to a current-scope evaluated literal,
  closure capture provenance outside the covered by-value local-copy captures,
  by-reference helper/callback parameters, arbitrary side effects outside the
  supported interpreter subset, whole-array reference identity, exact
  diagnostics, and native reference lowering remain unsupported.

- Lanes 1973 through 1976 broaden nested append reference sources below
  proven `ArrayAccess` backing buckets, including false-parent
  materialization. Direct `ArrayAccess` roots, visible by-value magic
  `__get()` roots returning an `ArrayAccess` object, dynamic magic-property
  spellings, and non-direct magic-property holders can now bind
  `$root["key"][]` through the existing public by-reference `offsetGet()`
  bridge, append into the selected backing bucket, and keep the alias live.
  Focused coverage proves PHP-parity conversion of a selected `false` bucket
  to an array. Parser support for append expressions in by-reference argument
  position, arbitrary `ArrayAccess`/magic bodies outside the supported
  interpreter subset, whole-array reference identity, exact diagnostics, and
  native reference lowering remain unsupported.

- Lanes 1969 through 1972 broaden by-value magic/`ArrayAccess` method-body
  execution for copied-array COW provenance through caught thrown-object
  paths. Visible by-value `__get()` and public by-value
  `ArrayAccess::offsetGet()` bodies can now throw a supported object inside
  the method-body executor, catch it with a matching supported class, run
  bounded `finally` side effects, and return a proven backed object-property
  array copy directly or through a tracked local while preserving selected
  reference-backed nested slots. This is not general PHP exception support:
  uncaught throws, non-object throws, exact Throwable enforcement and
  diagnostics, arbitrary unsupported method-body syntax, whole-array reference
  identity, and native reference lowering remain unsupported.

- Lanes 1964 through 1968 broaden the same mixed nested `ArrayAccess` binding
  to by-reference user-function arguments. Direct `ArrayAccess` roots,
  visible property-held roots, method-returned and factory-returned holder
  roots, dynamic selected holder properties, and visible by-value `__get()`
  roots can now pass a selected inner by-reference `ArrayAccess` bucket to a
  by-reference parameter when the by-value outer `offsetGet()` returns the
  inner object handle. Callee writes route back to the inner backing bucket.
  This is still bounded to supported interpreter-body execution and covered
  lvalue bindings; arbitrary PHP side effects outside that subset, arbitrary
  magic/`ArrayAccess` bodies, whole-array reference identity, exact
  diagnostics, and native reference lowering remain unsupported.

- Lanes 1959 through 1963 broaden mixed nested `ArrayAccess` reference-source
  binding for visible property-held roots reached through non-direct holder
  expressions. When a method-returned or factory-returned holder exposes a
  visible property whose public by-value `offsetGet()` returns an inner
  `ArrayAccess` object, the runtime can continue into the inner public
  by-reference `offsetGet()` binding. Focused coverage includes direct
  method-return holders, factory-return holders, dynamic property selection,
  nested append sources, and storable reference sources feeding another array
  slot. This is still a bounded executed-interpreter path; arbitrary PHP side
  effects outside the supported body subset, arbitrary magic/`ArrayAccess`
  bodies, whole-array reference identity, exact diagnostics, and native
  reference lowering remain unsupported.

- Lanes 1955 through 1958 extend `call_user_func_array()` helper-wrapper
  copied-array provenance beyond direct literal positional argument arrays.
  Visible by-value `__get()` and public by-value `ArrayAccess::offsetGet()`
  method bodies can now pass a proven backed bucket through a direct local
  argument-array variable, and literal or direct local string-keyed named
  argument arrays can bind the copied bucket to a matching non-reference user
  callback parameter while preserving selected reference-backed nested slots.
  Dynamic/stored argument containers that cannot be traced to a current-scope
  evaluated literal, closures, array callbacks, by-reference callback
  parameters, arbitrary PHP side effects outside the interpreter subset,
  whole-array reference identity, exact diagnostics, and native reference
  lowering remain unsupported.

- Lanes 1950 through 1954 extend helper-call copied-array provenance to
  literal positional `call_user_func_array()` wrappers for non-reference user
  callbacks. Visible by-value `__get()` and public by-value
  `ArrayAccess::offsetGet()` bodies can pass a proven backed bucket through a
  literal argument array, including explicit integer-keyed positional arrays,
  and return the callback parameter or a tracked local while preserving the
  copied bucket's selected reference-backed nested slots. A by-value
  user-call argument sourced from an `ArrayAccess::offsetGet()` wrapper keeps
  the same referenced slot shared in the callee. Dynamic/stored/named
  `call_user_func_array()` argument arrays, closures, array callbacks,
  by-reference callback parameters, arbitrary PHP side effects, whole-array
  reference identity, exact diagnostics, and native reference lowering remain
  unsupported.

- Lanes 1945 through 1949 preserve by-value copied-array COW provenance when
  visible magic/`ArrayAccess` method bodies pass a proven backed bucket
  through non-reference user helper calls before returning it. Covered calls
  include direct user functions, helper locals inside those functions,
  instance-method helpers, public by-value `ArrayAccess::offsetGet()` helper
  bodies, and string user callbacks through `call_user_func()`. The callee
  parameter records the argument's proven array-copy source, so returns of
  the parameter or a tracked local keep selected reference-backed nested
  slots shared while ordinary nested values stay detached. This does not yet
  cover `call_user_func_array()`, closures, array callbacks, by-reference
  helper parameters, arbitrary PHP side effects, whole-array reference
  identity, exact diagnostics, or native reference lowering.

- Lanes 1940 through 1944 preserve by-value copied-array COW provenance when
  visible magic/`ArrayAccess` method bodies return a proven backed bucket
  through selected wrapper expressions. Covered bodies include error-control
  returns, array-cast returns that keep an already-array value, local
  assignment-expression returns, and local null-coalescing-assignment returns.
  The `??=` variable assignment path records RHS provenance when the
  assignment branch runs. Focused coverage includes visible by-value
  `__get()` wrapper returns and a by-value user-call argument sourced from an
  `ArrayAccess::offsetGet()` assignment-expression body. Arbitrary PHP side
  effects, unsupported method-body syntax, side-effecting copied-key parity,
  whole-array reference identity, exact diagnostics, and native reference
  lowering remain unsupported.

- Lanes 1935 through 1939 preserve by-value copied-array COW provenance when
  visible magic/`ArrayAccess` method bodies return a backed bucket through
  conditional expressions. Covered bodies include full ternary returns,
  short-ternary returns of a tracked local bucket, and null-coalescing
  fallback returns for object-property backed buckets. The copied array keeps
  selected reference-backed nested slots shared while ordinary nested values
  remain detached, including a by-value user-call argument sourced from an
  `ArrayAccess` null-coalescing body. This is a bounded source-aware
  expression broadening; arbitrary PHP side effects, unsupported method-body
  syntax, side-effecting copied-key parity, whole-array reference identity,
  exact diagnostics, and native reference lowering remain unsupported.

- Lanes 1931 through 1934 prove the same alias-root by-value COW provenance
  for covered auto-superglobal buckets. Public by-value
  `ArrayAccess::offsetGet()` and visible by-value `__get()` bodies can return
  `$_REQUEST` buckets by value while preserving selected reference-backed
  nested slots in the copied array. Tracked locals assigned from those request
  buckets keep the same provenance, and by-value user-call arguments sourced
  from the returned copy preserve referenced nested slots in the callee while
  ordinary nested values remain detached. This is bounded to the current
  auto-superglobal interpreter roots and does not claim full SAPI/request
  semantics, arbitrary PHP side effects, arbitrary unsupported method-body
  syntax, whole-array reference identity, exact diagnostics, or native
  reference lowering.

- Lanes 1926 through 1930 generalize by-value copied-array COW provenance for
  magic/`ArrayAccess` method bodies beyond object-property backing roots to
  covered alias-root sources. Public by-value `ArrayAccess::offsetGet()` and
  visible by-value `__get()` bodies can now return direct `$GLOBALS` buckets
  or imported-global buckets by value while preserving selected
  reference-backed nested slots in the copied array. Tracked locals assigned
  from those global buckets keep the same provenance, and by-value user-call
  arguments sourced from the returned copy preserve referenced nested slots
  inside the callee while ordinary nested values remain detached. This is
  still bounded to the current interpreter subset and the covered alias roots;
  arbitrary PHP side effects, arbitrary unsupported method-body syntax,
  whole-array reference identity, exact diagnostics, and native reference
  lowering remain unsupported.

- Lanes 1921 through 1925 preserve reference-backed slots when a
  by-reference assignment selects inside a by-value magic/`ArrayAccess`
  copied array result. The covered paths still emit PHP's bounded
  indirect-modification notice/no-op diagnostic for the overloaded access,
  but if the selected nested element in the copied array is a reference cell,
  the alias remains tied to that original reference-backed slot while ordinary
  selected elements stay detached. Focused coverage includes direct by-value
  `ArrayAccess`, dynamic property-held `ArrayAccess`, by-value `__get()`
  plain arrays, magic-provided `ArrayAccess` objects whose inner
  `offsetGet()` returns by value, and a mixed by-value `ArrayAccess` chain
  with a by-value terminal copied-array result. Unsupported shapes still
  include arbitrary PHP side effects outside the interpreter subset,
  arbitrary magic/`ArrayAccess` method bodies beyond supported statements,
  exact diagnostics, whole-array reference identity, and native reference
  lowering.

- Lanes 1916 through 1920 add bounded direct `$GLOBALS` value
  materialization. Reading `$GLOBALS` now produces a PHP array snapshot of the
  current root global symbol table in both top-level and function scope.
  Ordinary unaliased globals detach when the snapshot is mutated, while direct
  variable reference groups and nested reference-backed array slots remain
  shared inside the copied snapshot, matching the covered COW behavior. The
  previously covered `$GLOBALS["GLOBALS"]` spelling appears as an ordinary
  key when that root global exists. This is still not full PHP
  `$GLOBALS` semantics: PHP's exact global insertion order, dynamic recursive
  self contents, `$GLOBALS` by-reference binding, non-string keyed direct
  `$GLOBALS` access, broad alias lifetime across returned function-local
  metadata, exact diagnostics, and native reference lowering remain
  unsupported.

- Lanes 1911 through 1915 broaden mixed nested `ArrayAccess` chains in
  by-reference method bodies when each reached object can be statically routed
  through the existing proven `offsetGet()` alias bridge. A reference-return
  body can now return `$this->inner[$offset]`,
  `$this->inner[$offset]["leaf"]`, `$this->{$property}[$offset]`,
  `$this->pick()[$offset]`, or a helper-local `$bag[$offset]` when the
  selected object is an `ArrayAccess` object whose public by-reference
  `offsetGet()` returns a covered lvalue. The returned alias is remapped back
  to the caller-visible object handle, so writes through the alias and backing
  bucket stay shared. Side-effecting key-order parity, unsupported PHP
  syntax/builtins outside the interpreter subset, by-value inner
  `offsetGet()` reference containers, arbitrary mixed chains, exact
  diagnostics, whole-array reference identity, and native reference lowering
  remain unsupported.

- Lanes 1907 through 1910 preserve copied-array COW reference slots across
  by-value user-call boundaries when the argument expression is a proven
  magic/`ArrayAccess` backed array copy. Direct user functions, instance
  methods, `call_user_func()` user callbacks, and literal or stored
  `call_user_func_array()` argument arrays now pass array values whose covered
  referenced elements remain reference-backed inside the callee. Callee writes
  to those referenced elements update the original backing slot, while writes
  to ordinary nested values stay local to the copied array. This remains
  bounded to proven copied-array sources in the current interpreter subset;
  arbitrary callback signatures, unsupported PHP side effects, whole-array
  reference identity, exact diagnostics, and native reference lowering remain
  unsupported.

- Lanes 1903 through 1906 preserve copied-array COW provenance through
  executed reference-return helper chains in magic/`ArrayAccess` bodies. A
  reference-return method that returns another reference-return call now keeps
  the returned array-offset binding instead of collapsing it to a detached
  cell, and by-reference `__get()` value reads derive the backed
  object-property copy source from that binding. Focused coverage includes
  public by-reference `ArrayAccess::offsetGet()` helper bodies, public
  by-reference `__get()` helper bodies, dynamic-property and non-direct holder
  magic reads, and a mixed magic-provided `ArrayAccess` chain. This is an
  executed-method-body improvement inside the current interpreter subset; it
  still does not claim unsupported syntax/builtins, arbitrary PHP side
  effects outside that subset, thrown exception unwinding, whole-array
  reference identity, exact diagnostics, or native reference lowering.

- Lanes 1899 through 1902 add a bounded builtin-callback reference/COW
  parity slice for selected array-mutating builtins. `call_user_func()` now
  evaluates `array_pop`, `array_unshift`, and `next` callback arguments by
  value, emits the covered reference-parameter warning, returns the result
  from the copied array, and leaves the caller array unchanged.
  `call_user_func_array()` now honors reference cells in literal or stored
  argument arrays for selected first-array-parameter builtins (`array_pop`,
  `array_unshift`, `ksort`, and `next`), mutating the referenced caller array
  without a warning. Assigning a builtin callback result by reference for
  ordinary supported builtin value calls emits the covered notice and binds
  only a detached local value. Unsupported builtin functions, broader
  callback signatures, exact warning/notice text, whole-array reference
  identity, full PHP internal pointer semantics, and native reference
  lowering remain unsupported.

- Lanes 1895 through 1898 close a bounded recursive string-keyed
  `$GLOBALS["GLOBALS"]` slice. Direct nested writes, reference sources,
  append reference sources, and root overwrites now treat the first
  `"GLOBALS"` key as the ordinary root global variable named `GLOBALS`,
  matching the covered PHP behavior instead of inventing a recursive global
  table. Focused coverage includes nested writes that leave `$x` unchanged,
  deep reference-slot write-through below `$GLOBALS["GLOBALS"]`, function
  scope append references, and repeated/root reference-bound
  `$GLOBALS["GLOBALS"]` overwrites. Full direct `$GLOBALS` array
  materialization, dynamic global names, non-string `$GLOBALS` keys,
  `$GLOBALS[] =&`, exact diagnostics, whole-array reference identity, and
  native reference lowering remain unsupported.

- Lanes 1890 through 1894 preserve by-value copied-array COW provenance
  through executed, non-reference magic/`ArrayAccess` method bodies in the
  interpreter subset. Public by-value `ArrayAccess::offsetGet()` and
  `__get()` can now return backed object-property array copies through
  supported control flow, tracked locals, and helper method calls, including
  nested method-call targets such as `$this->bucket($offset)["group"]` and a
  mixed magic-provided `ArrayAccess` chain. This is still not a claim of
  arbitrary PHP side effects: unsupported syntax/builtins remain unsupported,
  thrown exception unwinding, full direct `$GLOBALS` array materialization,
  arbitrary callback/builtin reference returns, whole-array reference identity,
  exact diagnostics, and native reference lowering remain unsupported.

- Lanes 1885 through 1889 preserve by-value method-return copied-array COW
  provenance for direct `ArrayAccess` offset return expressions backed by the
  proven `offsetGet()` bridge. Covered roots include `return $this[$group]`,
  property-held roots such as `return $this->bag[$group]`, dynamic
  property-held roots, expression roots such as `return $this->make()[$group]`,
  and by-value magic `__get()` roots returning the `ArrayAccess` object. This
  remains bounded to public `offsetGet()` shapes that return a statically
  proven object-property array bucket; arbitrary `offsetGet()` result
  expressions, untracked temporaries, thrown exception unwinding,
  whole-array reference identity, exact diagnostics, and native reference
  lowering remain unsupported.

- Lanes 1880 through 1884 preserve by-value method-return copied-array COW
  provenance through bounded local temporary returns. A local variable assigned
  from a proven object-property array copy can now be returned while retaining
  the same mirrored reference slots. Covered sources include whole visible
  `$this->store` copies, dynamic `$this->{$property}` copies, selected
  `$this->store[$group]` copies, local `ArrayAccess` offset copies backed by a
  proven `offsetGet()` bridge, and by-value Iterator loop variables whose
  `current()` value carried direct object-property array provenance. This
  remains bounded to tracked local variables in the interpreter subset;
  arbitrary returned expressions, untracked temporary construction,
  side-effecting copied-key expressions, thrown exception unwinding,
  whole-array reference identity, exact diagnostics, and native reference
  lowering remain unsupported.

- Lanes 1875 through 1879 preserve by-value method-return copied-array COW
  provenance when the covered return expression is reached from supported
  `foreach` bodies. The source-aware method-return path now covers by-value
  array iteration, by-reference foreach over supported array roots, by-value
  foreach over ordinary public-property objects, bounded userland
  `Iterator`, and bounded `IteratorAggregate` returning a userland
  `Iterator`. Focused coverage includes ArrayAccess-backed array,
  by-reference, public-object, and IteratorAggregate foreach returns plus a
  magic `__get()` Iterator return. This remains bounded to the interpreter
  subset and the existing direct visible object-property array return shapes;
  arbitrary returned expressions, untracked temporary construction, thrown
  exception unwinding, whole-array reference identity, exact diagnostics, and
  native reference lowering remain unsupported.

- Lanes 1871 through 1874 preserve the same by-value method-return
  copied-array COW provenance when the covered return expression is reached
  through supported method-body control flow. Executed `if`/`else`,
  `switch`/`case`/`default`, `while`/`do while`/`for`, and `try`/`finally`
  paths can now carry provenance for direct visible `$this` object-property
  array returns after supported magic/`ArrayAccess` aliases have established
  reference slots below the backing property. Focused coverage includes
  ArrayAccess-backed `if`, `switch`, and loop returns plus a magic `__get()`
  `try`/`finally` return. This remains bounded to the interpreter subset and
  the existing direct visible object-property array return shapes; arbitrary
  returned temporaries, thrown exception unwinding, whole-array reference
  identity, exact diagnostics, and native reference lowering remain
  unsupported.

- Lanes 1867 through 1870 preserve copied-array COW provenance when a
  by-value user method returns a backed object-property array after supported
  magic/`ArrayAccess` aliases have already created reference slots below that
  property. Covered public/private `ArrayAccess::offsetGet()` and
  `__get()` paths now keep scalar slot write-through when a later method
  returns `$this->store`, `$this->{$property}`, or `$this->store[$group]`.
  This is bounded to by-value method returns whose returned expression is a
  direct visible object-property array path with side-effect-free copied keys;
  arbitrary returned temporaries, whole-array reference identity, exact
  diagnostics, and native reference lowering remain unsupported.

- Lanes 1864 through 1866 tighten scalar/string parent failures for direct
  variable, `$GLOBALS`, and imported-global array-offset reference sources.
  Covered direct keyed, append, and nested append `=&` paths now use the same
  bounded failure classes as expression-root temporaries: string integer
  offsets, non-integer string offsets, string root appends, nested string
  appends, and `true`/int/float scalar-as-array failures. `null` and `false`
  materialization remains unchanged. Exact PHP fatal text/stack traces,
  object/resource parent failures, and native reference lowering remain
  unsupported.

- Lanes 1861 through 1863 extend the same executed reference-return body
  global-root alias path to imported globals and auto-superglobal array slots.
  Direct reference-return helpers can now `global $store; return
  $store[$key];`, and covered public by-reference `__get()`/
  `ArrayAccess::offsetGet()` bodies can return imported global or
  superglobal buckets such as `return $_REQUEST[$offset];` while preserving
  COW/reference-slot identity. This remains bounded to globals routed through
  the current symbol table, direct array-offset lvalues, and the existing
  interpreter subset; recursive `$GLOBALS` materialization, arbitrary
  callback/builtin reference-return sources, exact diagnostics, and native
  reference lowering remain unsupported.

- Lanes 1858 through 1860 broaden executed magic/`ArrayAccess`
  reference-return method bodies to return string-keyed `$GLOBALS` array
  slots as proven lvalues. Covered public by-reference `__get()` and
  `ArrayAccess::offsetGet()` bodies can now execute supported side effects,
  compute local keys, and return `$GLOBALS["store"][$key]`; delegated
  reference-return helpers, including dynamic string helper calls, share the
  same global-root alias path. This is bounded to direct string-keyed
  `$GLOBALS` paths inside the existing interpreter method-body subset and
  still does not claim recursive `$GLOBALS` materialization, arbitrary
  callback/builtin reference-return sources, full PHP side effects, exact
  diagnostics, or native reference lowering.

- Lanes 1855 through 1857 tighten scalar/string temporary behavior for
  expression-root array-offset and append reference sources. Direct variable
  targets still materialize `null` and `false` temporaries as detached arrays
  for keyed and nested append paths, while `true`, int, and float temporaries
  now fail with the bounded scalar-as-array error. String temporaries split
  the covered PHP-shaped failures for integer offset references, non-integer
  string offsets, root appends, and nested string-offset appends. Exact PHP
  fatal text/stack traces remain outside the current diagnostic parity claim,
  and native reference lowering is still unsupported.

- Lanes 1850 through 1854 broaden executed magic/`ArrayAccess`
  reference-return method-body delegation for non-static helpers reached
  through PHP's static call syntaxes inside an object context. Covered
  by-reference `ArrayAccess::offsetGet()` and `__get()` bodies can now return
  `self::helper(...)`, `parent::helper(...)`, `static::helper(...)`,
  compatible `ClassName::helper(...)`, or compatible dynamic static receiver
  calls such as `$this::helper(...)` when the resolved helper returns a
  supported lvalue by reference. The current `$this` object is used for the
  non-static helper call, matching the covered PHP object-context behavior.
  This still does not claim arbitrary PHP outside the interpreter subset,
  incompatible typed signatures, full exception unwinding, exact diagnostics,
  or native reference lowering.

- Lanes 1846 through 1849 extend append-offset expression-root reference
  sources below selected `ArrayAccess` buckets. Direct variable targets can
  now append below arrays returned from an expression-root `offsetGet()`, and
  can continue into selected inner `ArrayAccess` objects before applying the
  append. Covered examples include `$alias =& make_bag()["outer"][]`,
  `$alias =& $factory->make()["outer"][]`, and
  `$alias =& make_outer()["box"][]`, including the by-value outer object
  handle case where the selected bucket is an inner `ArrayAccess` object. This
  still does not claim scalar/string temporary reference parity, arbitrary PHP
  outside the interpreter subset, exact diagnostic streams, or native
  reference lowering.

- Lanes 1842 through 1845 add append-offset expression-root reference
  sources. Direct variable targets can now bind to appended slots below
  returned arrays, null temporaries, and direct function/method results that
  evaluate to public by-reference `ArrayAccess` objects, such as
  `$alias =& make_array()[]`, `$alias =& make_array()["outer"][]`,
  `$alias =& make_bag()[]`, and `$alias =& $factory->make()[]`. `ArrayAccess`
  append roots pass `null` to `offsetGet()` while the backing array key uses
  PHP's empty-string null-key coercion. This still does not claim
  scalar/string temporary reference parity, arbitrary PHP outside the
  interpreter subset, exact diagnostic streams, or native reference lowering.

- Lanes 1838 through 1841 derive copied-array COW provenance from executed
  public by-reference `ArrayAccess::offsetGet()` bodies during normal reads.
  Direct, property-held, expression-root, and magic-provided `ArrayAccess`
  roots such as `$bucket = $bag[10]`, `$bucket = $holder->bag[10]`, and
  `$bucket = $box->missing[10]` can now run supported side effects and local
  alias setup inside `offsetGet()` before preserving reference slots in the
  copied array. This still does not claim arbitrary PHP outside the
  interpreter subset, arbitrary magic/`ArrayAccess` side effects that the
  interpreter cannot execute, exact diagnostic streams, or native reference
  lowering.

- Lanes 1834 through 1837 add expression-root array-offset reference
  sources. Direct variable targets can now bind to selected buckets below
  arrays or `ArrayAccess` objects returned by calls and method calls, such as
  `$alias =& make_array()["slot"]`, `$alias =& make_bag()["slot"]`, and
  `$alias =& $factory->make()["slot"]`. Returned array literals now preserve
  supported `&` elements as real reference cells in ordinary expression
  evaluation, so references embedded in a returned array remain live when the
  returned temporary is selected. The same expression-root bridge can recurse
  through a proven by-value outer `ArrayAccess` object into a by-reference
  inner `ArrayAccess` bucket. This still does not claim scalar/string
  temporary reference parity, arbitrary callback/builtin reference sources,
  arbitrary PHP outside the interpreter subset, or native reference lowering.

- Lanes 1830 through 1833 close the whole by-value magic-property root
  counterpart to the preceding array-offset work. Direct, dynamic, and
  non-direct holder roots such as `$alias =& $box->missing`,
  `$alias =& $box->{$name}`, and `$alias =& $holders["box"]->missing` now
  execute public by-value `__get()` bodies, emit the bounded
  indirect-modification notice, and bind the target to a detached PHP value
  instead of creating or mutating a dynamic property. By-reference parameter
  calls receive the same detached temporary. Object values remain PHP object
  handles, so mutating the detached object mutates the returned object itself.
  This still does not claim arbitrary magic/`ArrayAccess` bodies outside the
  interpreter subset, arbitrary mixed chains, exact diagnostic streams, or
  native reference lowering.

- Lanes 1827 through 1829 add the magic-property counterpart to the
  by-value `ArrayAccess` work. Public by-value `__get()` bodies that return
  plain arrays now execute in covered direct and non-direct holder
  array-offset reference sources, produce detached PHP no-op aliases, and do
  not mutate the backing array. Public by-value `__get()` returning a
  side-effecting by-value outer `ArrayAccess` object can also continue into a
  proven inner by-reference `ArrayAccess` bucket. This still does not claim
  arbitrary `__get()` bodies outside the interpreter subset, arbitrary mixed
  chains, exact diagnostic streams, or native reference lowering.

- Lanes 1824 through 1826 execute side-effecting public by-value
  `ArrayAccess::offsetGet()` bodies in the covered reference-source and
  nested mutation paths before applying PHP's detached/no-op result. Direct
  roots, property-held roots, and mixed chains where a by-value outer
  `offsetGet()` returns an inner by-reference `ArrayAccess` object now
  preserve the executed method side effects and the inner alias. Append
  reference sources pass `null` to `offsetGet()` while null array keys coerce
  to the empty-string bucket inside the executed body. This still does not
  claim arbitrary `ArrayAccess` method bodies outside the interpreter subset,
  by-value `__get()` plain-array reference sources, arbitrary mixed chains,
  exact PHP diagnostic streams, or native reference lowering.

- Lanes 1821 through 1823 add direct reference-assignment sources for
  declared static-property roots and selected static-property array buckets.
  Direct variable targets can now bind to `ClassName::$slots`,
  `ClassName::$slots[$key]`, nested selected buckets, `static::$slots`, and
  dynamic receiver forms such as `$class::$slots[$key]` and
  `$object::$slots`. This still does not claim arbitrary reference-assignment
  targets from static-property sources, arbitrary static-property reference
  containers beyond cell-backed declared roots and selected buckets, typed
  static-property enforcement through returned aliases, arbitrary PHP syntax
  or side effects outside the executed interpreter subset, exact PHP
  diagnostics, or native reference lowering.

- Lanes 1818 through 1820 move declared static properties onto shared
  reference cells and add whole static-property reference-return roots.
  Covered by-reference functions, magic `__get()`, and
  `ArrayAccess::offsetGet()` bodies can now return named, `self::`,
  `static::`, object-receiver, or dynamic class-string static properties such
  as `return static::$slots;` or `return $class::$slots;`; aliases observe
  later direct static-property overwrites and writes through aliases update
  the property. This still does not claim arbitrary static-property reference
  containers beyond the cell-backed declared roots, typed static-property
  enforcement through returned aliases, arbitrary PHP syntax or side effects
  outside the executed interpreter subset, exact PHP diagnostics, or native
  reference lowering.

- Lanes 1815 through 1817 add bounded static-property array-offset roots to
  reference-return execution. Covered by-reference functions and magic/static
  methods can now return named, `self::`, `static::`, and dynamic class-string
  static property slots such as `return self::$slots[$key];` and
  `return $class::$slots["nested"][$key];`, with the selected bucket promoted
  to the shared reference-cell path. This still does not claim arbitrary
  static-property reference containers, typed static-property enforcement
  through returned aliases, arbitrary PHP syntax or side effects outside the
  executed interpreter subset, exact PHP diagnostics, or native reference
  lowering.

- Lanes 1812 through 1814 parse and execute direct dynamic-call
  reference-assignment sources. `$fn(...)` can now bind a returned reference
  when `$fn` evaluates to a supported function string, by-reference closure,
  or `[object-or-class, method]` array callable, including class-string array
  callables that route through bounded `__callStatic()`. This still does not
  claim arbitrary callable arrays, builtin callbacks as reference-return
  sources, arbitrary PHP syntax or side effects outside the executed
  interpreter subset, incompatible typed signatures, full exception unwinding,
  exact PHP diagnostics, or native reference lowering.

- Lanes 1809 through 1811 add bounded magic `__callStatic()` reference-return
  sources. Missing named static, `self::`, `static::`, and dynamic static
  receiver calls can now dispatch to public static by-reference
  `__callStatic()` when its executed body returns a proven variable,
  object-property, or array-offset lvalue. Class-string array callables through
  `call_user_func()` and `call_user_func_array()` share the same bridge. This
  does not claim direct `$cb(...)` reference-assignment parser support,
  arbitrary callable arrays, builtin callbacks as reference-return sources,
  arbitrary PHP syntax or side effects outside the executed interpreter
  subset, incompatible typed signatures, full exception unwinding, exact PHP
  diagnostics, or native reference lowering.

- Lanes 1806 through 1808 add bounded magic `__call()` reference-return
  sources to the executed magic/`ArrayAccess` bridge. Covered direct missing
  instance method calls and object array callables that resolve through
  public by-reference `__call()` can now return proven backing buckets while
  preserving caller-side suffix keys.

- Lanes 1803 through 1805 add bounded array-callable reference-return sources
  to the same executed magic/`ArrayAccess` bridge. Covered bodies can now
  `return $cb(...)` when `$cb` is an object/class array callable, or use
  `call_user_func($cb, ...)` with object and static array callables, as long as
  the resolved method returns a proven lvalue by reference.

- Lanes 1800 through 1802 add bounded by-reference closure callbacks to the
  executed magic/`ArrayAccess` reference-return body path. Covered bodies can
  now return a direct by-reference closure call, `call_user_func()` with a
  by-reference closure or function callback, and `call_user_func_array()` with
  a by-reference closure callback when the returned lvalue stays inside the
  proven variable/object-property/array-offset subset. Helper-local direct
  object-property array offsets such as `return $box->store[$key]` are remapped
  back to a caller-visible object root before caller suffix keys are applied.

- Lanes 1797 through 1799 add dynamic string function-call delegation in the
  executed magic/`ArrayAccess` reference-return body path. Covered bodies can
  now assign a helper name to a local variable and `return $fn(...)` when that
  helper returns a covered lvalue by reference. The same bundle also maps
  object-property roots returned from helper local scopes, such as
  `return $box->store`, back to a stable caller-visible object root before
  caller suffix keys are applied.

- Lanes 1794 through 1796 let supported magic/`ArrayAccess`
  reference-return bodies delegate their returned lvalue through another
  reference-returning call expression. Covered bodies can now `return
  $this->helper()`, where the helper returns a backing property root or
  selected backing bucket by reference, and can also `return helper(...)` when
  a direct helper returns a covered by-reference object-property array
  argument. The delegated result is promoted to the same reference-cell path
  used by direct lvalue returns before caller suffix keys are applied.

- Lanes 1791 through 1793 add property-root reference-return bridges for
  executed magic/`ArrayAccess` method bodies. Covered public by-reference
  `__get()` bodies can now run side effects and return `$this->property` or
  `$this->{$name}` by reference, then let direct reference assignments bind to
  that root or let the caller apply nested array keys while preserving copied
  COW reference-slot write-through. The same
  property-root alias promotion now lets public by-reference
  `ArrayAccess::offsetGet()` return a whole backing property such as
  `$this->items` before the caller suffix selects the final bucket.

- Lanes 1788 through 1790 close two bounded real-code gaps in the same
  magic/`ArrayAccess` reference-return body path. Catch matching now has
  system-PHP evidence for subclass-thrown objects caught by a parent
  `Exception` clause, and public by-reference `ArrayAccess::offsetGet()` /
  covered by-reference `__get()` bodies accept syntax-only `mixed` return
  metadata plus safe covered parameter metadata (`mixed $offset` and
  `string`/`mixed $name`) without rejecting otherwise executable COW bridges.

- Lanes 1785 through 1787 cover thrown-object catch paths inside the supported
  magic/`ArrayAccess` reference-return body executor. Public by-reference
  `ArrayAccess::offsetGet()` and covered by-reference `__get()` bodies can
  now throw an object inside a `try`, match a supported catch type by class or
  subclass, bind the catch variable, run catch statements, execute `finally`,
  and return a covered lvalue from the catch body while preserving copied COW
  reference-slot write-through.

- Lanes 1782 through 1784 preserve by-reference closure captures when the
  captured local is backed by covered magic/`ArrayAccess` array-offset alias
  metadata. Direct closure invocation can now mutate captured direct,
  non-direct holder, and escaping function-local aliases that point at covered
  magic/`__get()` and public by-reference `ArrayAccess::offsetGet()` backing
  buckets, while keeping copied COW reference-slot write-through. Internal
  hidden object roots are canonicalized toward readable visible roots before
  the capture escapes.

- Lanes 1779 through 1781 cover computed backing keys in the executed
  magic/`ArrayAccess` reference-return body path. Public by-reference
  `ArrayAccess::offsetGet()` and covered by-reference `__get()` bodies now
  preserve returned lvalues when the returned `$this` backing array path uses
  supported runtime-evaluated key expressions, including concatenated local
  keys, inline concatenated keys, and computed nested suffix keys.

- Lanes 1776 through 1778 propagate `goto` across nested supported
  reference-return body lists. Public by-reference
  `ArrayAccess::offsetGet()` and covered by-reference `__get()` bodies now
  preserve returned lvalues after jumps out of nested `if`, loop, switch,
  by-value `foreach`, by-reference `foreach`, and `try` bodies, including
  `finally` cleanup and by-reference foreach lingering-slot binding before
  the selected backing bucket is returned.

- Lanes 1773 through 1775 widen the supported magic/`ArrayAccess`
  reference-return method-body executor. Public by-reference
  `ArrayAccess::offsetGet()` and covered by-reference `__get()` bodies now
  preserve returned lvalues through by-reference `foreach` loops over
  supported array roots, by-value ordinary object public-property foreach,
  by-value bounded `Iterator`/`IteratorAggregate` foreach, and same-block
  labels/`goto`, while keeping copied COW reference-slot write-through for the
  selected backing bucket.

- Lane 1772 preserves reference-return lvalues through by-value array
  `foreach` bodies in public by-reference `ArrayAccess::offsetGet()` and
  covered by-reference `__get()` bodies, including local `continue` before the
  selected backing bucket is returned.

- Lane 1771 allows covered reference-return expressions in public
  by-reference `ArrayAccess::offsetGet()` and `__get()` bodies to bind through
  dynamically selected `$this->{$property}[...]` backing-property roots. The
  selected property name is evaluated in the method body before the bucket is
  materialized, preserving copied COW reference-slot write-through for the
  selected backing property.

- Lane 1770 preserves reference-return lvalues after multi-level
  `break`/`continue` control in nested loops inside public by-reference
  `ArrayAccess::offsetGet()` and covered by-reference `__get()` bodies,
  including propagation through `if`, `switch` break-depth handling, and
  `try`/`finally` cleanup before a loop exits.

- Lane 1769 preserves reference-return lvalues through `try` bodies, skipped
  catch clauses when no exception is thrown, and `finally` bodies in public
  by-reference `ArrayAccess::offsetGet()` and covered by-reference `__get()`
  bodies. `finally` side effects run before binding the returned bucket, and a
  supported lvalue returned from `finally` overrides an earlier try return,
  preserving COW reference-slot write-through.

- Lane 1768 preserves reference-return lvalues through `while`, `do while`,
  and `for` bodies in public by-reference `ArrayAccess::offsetGet()` and
  covered by-reference `__get()` bodies, so loop-selected backing buckets keep
  COW reference-slot write-through.

- Lane 1767 preserves reference-return lvalues through
  `switch`/`case`/`default` bodies in public by-reference
  `ArrayAccess::offsetGet()` and covered by-reference `__get()` bodies, so
  case-selected backing buckets keep COW reference-slot write-through.

- Lane 1766 preserves reference-return lvalues through executed `if`/`else`
  branches in public by-reference `ArrayAccess::offsetGet()` and covered
  by-reference `__get()` bodies, so branch-selected backing buckets keep COW
  reference-slot write-through.

- Lane 1765 routes public by-reference `ArrayAccess::offsetGet()` and covered
  by-reference `__get()` array-offset returns through the assignment-capable
  reference-return executor. Supported top-level method-body statements now
  run before binding the returned bucket, including side effects,
  missing-bucket initialization, and local backing aliases in the covered
  return forms.

- Lane 1693 focused COW work covers deeper magic-property `ArrayAccess`
  nested append mutation for exact public by-reference
  `offsetGet()` backing-property bridges of the form
  `return $this->items[$offset];`.
- Covered forms include direct, dynamic, non-direct holder, and dynamic
  non-direct holder shapes such as
  `$box->missing["outer"]["inner"][] = $array`.
- Copied-bucket reads preserve nested reference slots for the covered shapes.
- By-value `offsetGet()` keeps PHP's indirect-modification notice/no-op
  behavior for the covered deeper append shape.
- Lane 1694 extends that append helper to one focused mixed nested
  `ArrayAccess` chain where the first selected backing bucket contains another
  exact by-reference `ArrayAccess` object, so
  `$box->missing["outer"]["inner"][] = $array` can append into the inner
  object's selected backing bucket and preserve copied-bucket reference slots.
- Lane 1695 extends the same focused mixed chain when the outer
  `offsetGet()` returns the intermediate inner `ArrayAccess` object by value;
  the object handle is followed to the inner by-reference backing bucket.
- Lane 1696 extends that by-value outer mixed chain to a focused
  statement-form reference source, so
  `$alias =& $box->missing["outer"]["inner"]; $alias[] = $array;` binds and
  appends through the inner backing bucket when `__get()` returns a supported
  direct variable by reference.
- Lane 1697 extends that focused reference-source shape when by-value
  `__get()` returns the outer `ArrayAccess` object handle; the alias still
  binds through the mixed chain to the inner by-reference backing bucket.
- Lane 1698 proves the same reference-source machinery through a longer
  three-object mixed chain, with two by-value intermediate `ArrayAccess`
  object handles before the terminal by-reference backing bucket.
- Lane 1699 extends the focused reference-source path to a factory-return
  holder root and fixes equivalent object-property alias synchronization, so
  writes through the visible backing object property update aliases that were
  originally reached through hidden magic/`ArrayAccess` object handles.
- Lane 1700 pins local alias unset/detach behavior for the covered
  magic/`ArrayAccess` reference-source path: unsetting and reusing the local
  alias does not overwrite the backing bucket, while nested references inside
  the backing bucket continue to synchronize.
- Lane 1701 broadens the magic plain-array append route so by-reference
  `__get()` may return a visible `$this->store` property, not only a direct
  variable/global, for the focused append-and-copied-reference-slot shape.
- Lane 1702 pins the same visible `$this->store` route through a tested
  two-key parent append path, so `$box->missing["outer"]["inner"][] = $array`
  appends into `$box->store["outer"]["inner"]` and preserves copied reference
  slots for later direct backing-property writes.
- Lane 1703 extends the focused route to a private `$this->store` property
  returned by by-reference `__get()`: an outside append through
  `$box->missing["outer"][] = $array` stores reference-slot metadata under the
  private backing property, and a later same-class method write through
  `$this->store["outer"][0]...` resynchronizes caller-visible aliases with the
  original referenced variables.
- Lane 1704 adds the focused dynamic private-property return form
  `public function &__get($name) { return $this->{$name}; }` for the same
  append/method-write COW shape, so an inaccessible private property selected
  by the magic property name can be appended to and later method-local writes
  resynchronize copied nested reference slots.
- Lane 1705 adds the focused backing-array offset return form
  `public function &__get($name) { return $this->store[$name]; }`, so the
  requested magic property becomes a prefix key under a private backing array
  before the append path and same-class method writes synchronize copied
  nested reference slots.
- Lane 1706 extends that backing-array form to literal suffix buckets such as
  `public function &__get($name) { return $this->store[$name]["bucket"]; }`,
  so the requested magic property and literal suffix become prefix keys before
  the append path and same-class method writes synchronize copied nested
  reference slots.
- Lane 1707 generalizes the same exact backing-array path so literal prefix
  buckets may appear before the `__get()` parameter, such as
  `public function &__get($name) { return $this->store["bucket"][$name]; }`;
  the literal prefix and requested magic property become prefix keys before
  the append path and method-local backing writes synchronize copied nested
  reference slots.
- Lane 1708 extends the same analyzed backing-array `__get()` return body to
  statement-form reference sources such as
  `$alias =& $box->missing["outer"]; $alias[] = $array;`. The alias now binds
  directly to the object-property backing slot
  `$this->store["missing"]["outer"]`, so later same-class writes through that
  private backing array synchronize copied nested reference slots.
- Lane 1709 pins that same statement-source machinery through a non-direct
  dynamic holder root such as
  `$alias =& $holders["box"]->{$property}["outer"];`, with the temporary
  holder root canonicalized back to the same object-property backing slot for
  later method-local synchronization.
- Lane 1710 widens the exact `ArrayAccess::offsetGet()` reference-source
  bridge from only `return $this->items[$offset];` to the same bounded
  literal-key family used by the magic `__get()` work, such as
  `return $this->items["bucket"][$offset];` and
  `return $this->items[$offset]["bucket"];`.
- Lane 1711 applies the same literal-key family to the exact
  `ArrayAccess::offsetSet()` backing-property write bridge, so assignments
  like `$bag["leaf"] = $array` preserve copied reference-slot metadata when
  `offsetSet($offset, $value)` writes to
  `$this->items["bucket"][$offset]` or `$this->items[$offset]["bucket"]`.
- Lane 1712 extends the branchy `offsetSet(null, $value)` append bridge to
  literal prefix buckets such as
  `if ($offset === null) { $this->items["bucket"][] = $value; return; }
  $this->items["bucket"][$offset] = $value;`, preserving copied
  reference-slot metadata under the actual appended key.
- Lane 1713 extends that branchy append bridge to literal suffix buckets after
  the append slot, such as
  `if ($offset === null) { $this->items[]["bucket"] = $value; return; }
  $this->items[$offset]["bucket"] = $value;`, preserving copied
  reference-slot metadata under the actual appended key plus suffix bucket.
- Lane 1714 accepts the equivalent `if/else` `offsetSet()` append body without
  a `return`, such as
  `if ($offset === null) { $this->items["outer"][]["leaf"] = $value; }
  else { $this->items["outer"][$offset]["leaf"] = $value; }`, preserving
  copied reference-slot metadata under the actual appended key for the same
  literal prefix/suffix family.
- Lane 1715 accepts repeated uses of the `ArrayAccess` offset parameter in
  exact non-branchy `offsetGet()`/`offsetSet()` backing paths, such as
  `return $this->items[$offset][$offset];` and
  `$this->items[$offset][$offset] = $value;`, so copied reference slots bind
  to the repeated-key backing bucket.
- Lane 1716 applies the `if/else` `offsetSet()` body recognizer to keyed
  stores such as `$bag["leaf"] = $array`, preserving copied reference-slot
  metadata when the non-null branch writes through the same literal
  prefix/suffix family.
- Lane 1717 extends keyed store propagation to visible property-held
  `ArrayAccess` objects for covered one-key shapes such as
  `$holder->bag["leaf"] = $array`, `$holders["box"]->bag["leaf"] = $array`,
  and `$holders["box"]->{$name}["leaf"] = $array`.
- Lane 1718 extends property-held keyed stores to nested plain-array leaves
  below exact by-reference `offsetGet()` parent buckets, such as
  `$holder->bag["outer"]["leaf"] = $array`.
- Lane 1719 extends direct magic-property-provided `ArrayAccess` keyed stores
  such as `$box->missing["leaf"] = $array` and
  `$box->missing["outer"]["leaf"] = $array`, while leaving non-direct magic
  keyed stores for a future parser/runtime split because those shapes still
  share the same AST as non-direct append stores.
- Lane 1720 splits non-direct magic `ArrayAccess` keyed stores from append
  stores in the assignment-target AST and runtime. Non-direct named and
  dynamic holder forms such as `$holders["box"]->missing["leaf"] = $array`,
  `$holders["box"]->missing["outer"]["leaf"] = $array`, and
  `$holders["box"]->{$name}["leaf"] = $array` now preserve copied nested
  reference-slot metadata through the same tested magic/`ArrayAccess`
  backing-bucket bridges, while non-direct append stores continue through
  their append-specific path.
- Lane 1721 adds direct dynamic magic-property keyed stores such as
  `$box->{$name}["leaf"] = $array` and
  `$box->{$name}["outer"]["leaf"] = $array` by introducing a direct dynamic
  object-property array-index assignment target and routing it through the
  same visible property-held and magic `ArrayAccess` keyed bridges. Direct
  dynamic append stores remain on their append-specific route.
- Lane 1722 bundles several related fast-track COW slices: by-reference magic
  `__get()` keyed/nested plain-array stores mutate the returned backing array
  cell and preserve copied nested reference slots; by-value magic and
  property-held `ArrayAccess` plain-array keyed/nested stores use PHP's
  indirect-modification notice/no-op behavior; covered nested write,
  reference, and append helpers convert `false` parents to arrays while other
  scalar parents remain errors; and exact `__get()`/`offsetGet()` backing
  analyzers accept one local copy of the magic/offset parameter before the
  return. The same analyzer broadening is tested through a mixed by-value
  outer/by-reference inner `ArrayAccess` chain.
- Lane 1723 pins scalar-parent behavior for the currently covered nested COW
  paths: `false`/`null` parents materialize as arrays while preserving copied
  reference-slot metadata through direct nested stores, direct nested
  appends, visible object-property nested stores, by-reference magic
  `__get()` nested stores, and property-held `ArrayAccess` nested keyed
  stores. `true`, `int`, and `string` parents remain explicit runtime-error
  boundaries for direct, magic, and property-held `ArrayAccess` shapes.
- Lane 1724 extends property-held `ArrayAccess` nested append stores:
  direct, non-direct, and dynamic holder forms such as
  `$holder->bag["outer"][] = $array`,
  `$holders["box"]->bag["outer"][] = $array`, and
  `$holders["box"]->{$property}["outer"][] = $array` append into the
  by-reference `offsetGet()` backing bucket and preserve copied nested
  reference-slot metadata under the appended key.
- Lane 1725 broadens the proven backing-bucket analyzer by one shape:
  `__get()` and `offsetGet()` bodies may first bind a local variable by
  reference to a direct `$this->property`, then return from that local with
  the existing literal/parameter-copy index rules. A focused fixture covers
  by-reference magic plain-array writes and a by-value magic/mixed
  `ArrayAccess` nested append chain using that local backing alias.
- Lane 1726 starts the real value-model refactor after the bridge bundles:
  array slots and object properties now share a common internal PHP value-cell
  primitive with stable cell identity, value-based equality, and clone-by-value
  behavior. The runtime also has a focused PHP reference-cell container
  primitive with shared identity and write-through alias behavior. This is
  substrate for later COW separation and interpreter wiring, not PHP-visible
  general references yet.
- Lane 1727 wires that reference-cell substrate into array slot storage:
  `ArraySlot` can now hold either a by-value cell or a PHP reference cell.
  Cloning a reference-backed slot preserves the reference container and writes
  through either slot update the shared container, while ordinary value-backed
  slots keep clone-by-value and detach-on-write behavior.
- Lane 1728 wires the same reference-cell substrate into object-property
  storage. Public property reads, writes, `isset`/`empty`, object clone
  storage, object-to-array enumeration, and diagnostic output can now read
  reference-backed properties through cloned values, and focused runtime
  coverage proves writes through a cloned object property update the shared
  reference container.
- Lane 1729 moves interpreter direct variable/reference cells and
  by-reference closure captures onto the same runtime `PhpReferenceCell`
  primitive. Direct variable aliases, by-reference parameters/returns, magic
  reference-return cells, and closure capture cells now share the runtime
  reference-cell identity instead of a compiler-local `Rc<RefCell<Value>>`.
- Lane 1730 starts wiring array element reference bindings into the new slot
  storage. Direct `$array[$key] =& $var` and `$array[] =& $var` store
  reference-backed `ArraySlot`s for direct variable sources, while the
  existing alias side table remains as scaffolding for broader nested,
  object-property, and mixed alias groups.
- Lane 1731 extends that wiring to nested direct array targets and direct
  object-property array targets/appends. Direct variable sources now install
  reference-backed slots for shapes such as
  `$array["outer"]["slot"] =& $var`, `$array["outer"][] =& $var`,
  `$object->items["slot"] =& $var`, and
  `$object->items["outer"][] =& $var`, while alias-backed sources and broader
  mixed/magic roots still use the bounded alias scaffolding. Method-local
  `$this` object-property targets also stay on that scaffolding to preserve
  existing by-value `ArrayAccess` bucket reuse parity.
- Lane 1732 covers the same direct object-property reference target path when
  the property name is evaluated dynamically, such as
  `$object->{$name}["slot"] =& $var` and
  `$object->{$name}["outer"][] =& $var`.
- Lane 1733 lets alias-backed source variables reuse a real reference-backed
  array slot when one is statically visible, so bindings from
  `$alias =& $items["slot"]` into direct array, nested array, and direct
  object-property array targets reuse the original `PhpReferenceCell` instead
  of copying only the current value.
- Lane 1734 promotes statically proven value-backed alias terminals to real
  reference-backed array slots before reusing them as sources. Covered roots
  are direct static arrays and direct public object-property arrays, including
  dynamically selected public property names, so `$target["copy"] =& $alias`
  can share the same `PhpReferenceCell` after `$alias =& $items["slot"]` or
  `$alias =& $object->{$name}["slot"]` even when the original terminal was
  not already reference-backed.
- Lane 1735 fixes the equivalent string-keyed `$GLOBALS` source-alias route:
  statement-form sources such as `$alias =& $GLOBALS["bag"]["slot"]` now use
  the real global-root alias representation before value-backed terminal
  promotion, so later target writes stay visible through `$GLOBALS`.
- Lane 1736 covers method-local `$this` object-property array reference
  targets for direct variable sources. Explicit bindings such as
  `$this->items["slot"] =& $var`, `$this->items["outer"][] =& $var`, and the
  same dynamic-property/private-property forms now install real
  reference-backed slots, while array-literal reference propagation through
  `$this` stays on the alias scaffold to preserve by-value `ArrayAccess`
  bucket reuse parity.
- Lane 1737 pins by-value terminal/plain-array nested append no-op parity for
  property-held `ArrayAccess`, magic-provided `ArrayAccess`, and by-value
  magic plain-array roots.
- Lane 1738 implements the matching reference-target bridge for supported
  property-held `ArrayAccess` and by-reference magic plain-array roots:
  keyed targets and parent appends such as
  `$holder->bag["parent"]["leaf"] =& $var` and
  `$holder->bag["parent"][] =& $var` now bind through exact by-reference
  `offsetGet()`/`__get()` backing buckets, materializing false parents like
  PHP. Direct, dynamic-property, and non-direct holder keyed forms are covered;
  broader append suffixes and arbitrary method bodies remain outside the
  subset.
- Lane 1739 moves whole object-property reference sources onto real runtime
  `PhpReferenceCell` storage for supported visible/context property reads.
  Statement-form aliases such as `$items =& $box->items`,
  `$items =& $box->{$name}`, method-local `$hidden =& $this->hidden`, and
  function-return expression roots such as `$items =& make()->items` now
  promote the property itself to a reference-backed object property, so object
  clones preserve PHP's shared property reference identity instead of relying
  only on array-offset alias side metadata. Alias-backed array target roots
  also install source reference cells when binding through a statically known
  alias path.
- Lane 1740 extends that property reference-cell path to missing dynamic
  properties on dynamic-capable objects such as `stdClass`. Named, dynamic,
  and function-return expression roots now materialize the missing property
  as a reference-backed dynamic public property, so aliases and object clones
  share writes like PHP. Array-offset and append reference sources below a
  missing dynamic property materialize the property as an array and preserve
  the nested reference through clones. Declared classes that do not allow
  dynamic public properties remain an explicit undefined-property boundary.
- Lane 1741 attaches typed-property write constraints to promoted object
  property `PhpReferenceCell`s. Direct assignments, variable-to-variable
  aliases, and ordinary by-reference parameter writes through a typed
  property alias now use the same bounded weak scalar coercion/error path as
  ordinary typed-property writes, including reference-backed property clones
  and method-local private property references.
- Lane 1742 carries those constraints through reference-backed array slots
  that reuse a typed property reference cell. Covered direct array, nested
  array, public object-property array, and compound-assignment writes to such
  slots coerce or reject through the typed-property constraint instead of
  overwriting the shared cell unchecked.
- Lane 1743 routes checked reference-cell writes through the interpreter's
  live class/interface metadata resolver, so property aliases and
  reference-backed array slots respect `class_alias()` names registered after
  an object was instantiated when enforcing class/interface typed-property
  constraints.
- Lane 1744 pins whole object-property `unset` for promoted reference-backed
  properties. Covered public, typed public, clone-shared public, and
  method-local private properties detach the property slot from the alias cell;
  later alias writes do not resurrect the property, and detached typed aliases
  no longer enforce the old property type constraint.
- Lane 1745 routes covered alias-backed writes through typed reference-cell
  checks. Local variables bound to reference-backed array slots, by-reference
  function writeback into such slots, `$GLOBALS` alias roots, and alias-rooted
  array-offset writes now coerce or reject through the original typed-property
  reference constraint instead of overwriting the cell unchecked.
- Lane 1746 extends that checked-write path to statically proven
  magic/`ArrayAccess` backing-bucket writes that reuse the same alias helper.
  The covered evidence pins property-held `ArrayAccess` nested backing writes
  that would otherwise overwrite an `int` typed-property reference slot with
  an array value.
- Lane 1747 broadens nested scalar `ArrayAccess` writes over the same proven
  by-reference backing buckets. Direct `ArrayAccess`, property-held,
  dynamic-property-held, non-direct holder, dynamic non-direct holder, and
  magic-provided `ArrayAccess` roots now write scalar values through the
  backing reference slot and keep typed-property reference coercion.
- Lane 1748 extends nested `ArrayAccess` append writes over proven
  by-reference backing buckets to scalar payloads and the currently parsed
  direct object-property suffix shape after `[]`. Covered roots include direct
  `ArrayAccess`, property-held, non-direct holder, dynamic non-direct holder,
  and magic-provided `ArrayAccess` append paths, with suffix payloads such as
  `$holder->bag["outer"][]["leaf"] = $value` preserving copied nested
  reference slots.
- Lane 1749 closes the remaining parsed-syntax gap for those suffix payloads:
  direct roots such as `$bag["outer"][]["leaf"] = $value`, direct dynamic
  property roots, non-direct holder roots, dynamic non-direct holder roots,
  and non-direct magic-provided `ArrayAccess` roots now parse and route to the
  same proven by-reference backing-bucket append path.
- Lane 1750 tightens alias lifetime after array-offset `unset`: when an unset
  array slot was backed by a PHP reference cell, detached local aliases keep
  that reference cell instead of degrading to a plain value snapshot. Covered
  direct, nested, and object-property array-offset unsets keep typed-property
  reference coercion/error behavior after detach.
- Lane 1751 broadens the proven magic/`ArrayAccess` return-body analyzer by
  one static local-alias shape: a public by-reference `__get()` or
  `offsetGet()` method may bind a local by reference to an indexed
  `$this->property[...]` backing bucket and return that local, or a literal
  child below it. The covered `ArrayAccess` nested write shape preserves
  typed-reference slots; the matching plain magic `__get()` value-write shape
  routes through the same backing bucket.
- Lane 1752 keeps reference cells alive when an alias-backed direct variable
  is passed to a by-reference parameter. If the covered alias group can be
  promoted to a real reference cell, the callee parameter binds to that cell
  instead of a temporary copy/writeback value, so references stored by the
  callee remain live after return and later magic backing-bucket writes still
  update the original typed-property reference.
- Lane 1753 applies the same cell-preservation rule to by-reference closure
  captures. If `use (&$name)` captures a variable routed through a covered
  alias group, closure invocation prebinds the capture to the promoted
  reference cell so references stored inside the closure remain connected to
  the original typed-property reference after return.
- Lane 1754 broadens the proven `__get()`/`offsetGet()` backing analyzer by
  one side-effect-free key shape: locals initialized from literal int/string
  keys may be used as backing-path indexes, while non-literal/dynamic key
  variables and arbitrary method bodies remain unsupported.
- Lane 1755 applies the same side-effect-free local literal-key shape to the
  proven `ArrayAccess::offsetSet()` backing analyzer for exact keyed stores
  and branchy null-append/keyed bodies, preserving copied reference slots in
  the selected backing bucket.
- Lane 1756 broadens that same `offsetSet()` analyzer to one local copy of
  the offset parameter before the backing assignment, matching the existing
  bounded parameter-copy family without accepting arbitrary side effects.
- Lane 1757 extends `offsetSet()` backing analysis to local variables bound by
  reference to `$this->property` or indexed `$this->property[...]` buckets,
  covering exact keyed stores plus branchy null-append/keyed bodies.
- Lane 1758 tightens scalar-parent parity for by-value and by-reference magic
  plain-array paths: by-value `__get()` returning `false` now follows PHP's
  indirect-modification no-op behavior for covered nested keyed and append
  writes, adjacent scalar returns fail as scalar offset writes instead of
  undefined properties, and by-reference `__get()` append roots convert
  `false` parents to arrays while preserving copied typed-reference slots.
- Lane 1759 extends root-level `false` materialization for direct variable,
  `$GLOBALS`, and object-property nested keyed/append writes, preserving
  copied typed-reference slots after those roots become arrays.
- Lane 1760 pins the matching root-level `false` materialization for direct,
  `$GLOBALS`, object-property, and append reference targets so reference
  assignments into those newly materialized arrays keep typed-reference cells
  live.
- Lane 1761 emits the PHP false-to-array `E_DEPRECATED` diagnostic through
  user error handlers for the covered root-level false array write and
  reference-target materialization paths.
- Lane 1762 extends that diagnostic to covered object-property array-offset
  and append reference targets when a visible/context property root is
  provably `false`, while leaving inaccessible and magic fallback cases on
  their existing bounded paths.
- Lane 1763 starts the string COW identity slice for bounded ASCII integer
  offsets: direct variable, visible/context object-property, and array-bucket
  string offset reads/writes now preserve by-value copy isolation and
  by-reference write-through, including negative in-range offsets and
  out-of-range space padding.
- Lane 1764 extends that bounded ASCII string-offset slice to integer-form
  string offset keys such as `"01"`, `"+2"`, and `" 1"` for direct,
  object-property, and array-bucket roots while keeping normal array key
  normalization unchanged.
- `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` record the focused
  verification for this bundle. Before a checkpoint commit, use
  `tools/checkpoint.sh`, which runs the full project gate.

## Fast-Track Build Plan

The immediate goal is not one microscopic COW lane per checkpoint. The next
1-2 hour build window should bundle related compatibility slices and run
focused verification until the bundle is ready for one full checkpoint.

Prioritize these COW gaps in order:

1. By-value terminal/plain-array nested mutation parity for the currently
   covered magic/`ArrayAccess` shapes.
2. Scalar parent overwrite/error parity for covered append/keyed/reference
   paths.
3. Broader mixed nested `ArrayAccess` chains where the runtime can statically
   prove the same backing-bucket bridge.
4. One bounded broadening of `__get()` or `offsetGet()` return-body analysis
   beyond the exact forms already supported, without claiming arbitrary
   side-effecting method bodies.

Verification discipline for this fast track:

- Run targeted fixture and system-PHP comparisons while implementing.
- Run adjacent regression checks for the affected lane family.
- Run `cargo check`, `cargo fmt --check`, and `git diff --check` before the
  bundle checkpoint.
- Run `tools/checkpoint.sh` once for the bundle, not after every tiny case.

## Success Criteria

Full PHP and WordPress compatibility is complete only when these gates are
covered by implementation, tests, CLI exercise paths, documentation, and named
unsupported edges:

- `phpc run` executes representative WordPress front-end, admin, AJAX, REST,
  cron, XML-RPC, install/update, plugin, theme, and CLI-like flows without
  unsupported runtime gaps.
- PHP values model references, copy-on-write, arrays, strings, objects,
  resources, exceptions, destructors, shutdown behavior, type coercions, and
  alias lifetime rules as coherent runtime semantics rather than per-fixture
  bridges.
- Multi-file PHP works across real trees: include/require, include paths,
  namespaces, imports, autoloading, conditional declarations, globals,
  superglobals, and request-local state.
- WordPress-critical standard library and extension surfaces work or fail
  through stable named boundaries: PCRE, JSON, date/time, hash/password,
  mbstring/iconv-like string handling, filesystem, streams, output buffering,
  sessions, cookies, uploads, HTTP/curl-like behavior, XML, ZIP, image
  metadata, mysqli/mysqlnd, and PDO-style database behavior.
- `phpc compile --emit-ir` and `--emit-asm` either lower features through a
  correct native runtime ABI or reject unsupported lowering before emitting
  misleading native code.

## Missing Before WordPress Can Run

The path from the current bounded bridges to full PHP/WordPress support is:

1. Collapse the targeted reference/COW bridges into a coherent value model
   with real PHP reference containers, array/object/string COW identity, and
   alias lifetime semantics across calls, includes, iterators, magic methods,
   shutdown, and errors.
2. Finish dynamic PHP execution: exceptions, object lifecycle, autoloading,
   namespaces/imports, traits/interfaces/enums/anonymous classes, reflection,
   dynamic dispatch, and internal class catalogs.
3. Expand the SAPI/runtime substrate from deterministic CLI seeds to realistic
   request, stream, filesystem, upload, output-buffer, header, cookie, session,
   and shutdown behavior.
4. Replace `wpdb` placeholder islands with real database semantics: MySQL
   connection state, charset/collation, arbitrary SQL, results, transactions,
   persistence, errors, and cache invalidation.
5. Run actual WordPress flows end to end and make unsupported gaps observable:
   front-end, admin, AJAX, REST, cron, XML-RPC, install/update, plugins,
   themes, blocks, media, localization, auth, nonces, roles, rewrite rules,
   HTTP API, and CLI-like maintenance paths.

### 1. PHP Reference and COW Model

Still missing:

- General PHP reference containers.
- Broad array/object/string copy-on-write identity.
- References inside arrays and object properties without special-case bridges.
- Arbitrary expression-root reference targets and sources.
- Mixed and arbitrary nested `ArrayAccess` chains, especially untracked
  dynamic containers and callback-dispatched containers outside the current
  supported interpreter/writeback paths.
- Broader magic `__get()`/`__set()`/`__isset()`/`__unset()` behavior beyond
  the tested direct-variable, visible `$this->property`, private
  `$this->property`, `$this->{$name}`, and `$this->property[$name]`
  plus literal-prefix/suffix method-write append slices.
- Alias detach, destructor, shutdown, and error-ordering parity.
- Reference/COW semantics across function calls, returns, includes,
  superglobals, iterators, and native lowering.

Current COW status: many high-value reached shapes are implemented and tested,
but the runtime still uses bounded alias metadata and targeted bridges. COW is
not done as a general PHP value model.

### 2. Object Model and Dynamic PHP

Still missing:

- Full constructors, destructors, clone lifecycle, object destruction order,
  abstract/final enforcement, variance, signature compatibility, and exact
  visibility diagnostics.
- Traits, interfaces, enums, anonymous classes, attributes, class aliases,
  internal class/interface catalogs, reflection breadth, and serialization
  hooks.
- Dynamic function/method/static/property lookup breadth, `__call`,
  `__callStatic`, `__invoke`, late static binding edge cases, and autoload
  integration.
- Exceptions, throw/catch/finally unwinding, generators, Fibers, fatal vs
  recoverable error distinctions, and warning/notice suppression parity.

### 3. Request, SAPI, Streams, and Filesystem

Still missing:

- Real webserver/SAPI integration beyond deterministic CLI request seeds.
- Multipart upload parsing, temp-file lifecycle, upload validation, and media
  handling.
- Output buffering, headers, cookies, sessions, save handlers, garbage
  collection, shutdown order, and concurrency behavior at PHP parity.
- Broader stream wrappers, filters, `phar://`, network streams, contexts,
  binary string fidelity, permissions, locking, metadata, and exact warning
  text.

### 4. Database and Persistence

Still missing:

- Real or faithfully emulated MySQL connections, authentication, charset and
  collation state, arbitrary SQL, result sets, prepared statements,
  transactions, errors, indexes, constraints, persistence, and mysqlnd/mysqli
  fidelity.
- Durable WordPress object cache, options, transients, schema install/update,
  query mutation, and cache invalidation behavior.
- PDO-style behavior where WordPress plugins require it.

### 5. WordPress Runtime Surface

Still missing:

- Full bootstrap through `wp-settings.php` under realistic request state.
- Front-end routing, admin screens, AJAX, REST routing, cron, XML-RPC,
  install/update, plugin loading, theme loading, block registration/rendering,
  media/filesystem updates, localization, rewrite rules, roles/capabilities,
  nonces, auth/session flows, and HTTP API behavior.
- Compatibility probes for common plugin and theme patterns, not only core
  synthetic fixtures and placeholder database paths.

### 6. Native Compiler Runtime ABI

Still missing:

- Native handles for PHP values, heap arrays, strings, objects, resources,
  references, COW containers, request state, and diagnostics.
- Generated calls into runtime helpers for arrays, objects, references,
  dispatch, includes, exceptions, streams, filesystem/database calls, and
  WordPress host state.
- Linked native execution for representative interpreter-supported PHP paths.
- Clear parity tracking between interpreter support and native support.

## Current Focus

Do not spread effort across cosmetic or unrelated work. The priority order is:

1. Finish the reference/COW value-model gaps that keep showing up as special
   cases: mixed `ArrayAccess`, broader magic-property returns, arbitrary
   reference roots, detach/lifetime behavior, and native reference boundaries.
2. Turn object semantics from reached slices into coherent PHP behavior:
   autoload, interfaces/traits, magic calls, destructors, reflection, and
   exact diagnostics.
3. Broaden request/SAPI/filesystem/stream behavior enough for real WordPress
   request flows instead of deterministic CLI placeholders.
4. Replace deterministic `wpdb`/MySQLi islands with real or faithfully
   emulated database behavior.
5. Grow the native runtime ABI only where interpreter semantics are already
   pinned and tested.

## Lane Policy

Use parallel lanes only when they are independent and can land with their own
evidence:

- Reference/COW lane.
- Object/dynamic PHP lane.
- Request/SAPI/streams/filesystem lane.
- WordPress database/bootstrap lane.
- Native runtime ABI lane.
- Docs/tests lane.

Each lane must finish with code, tests, CLI coverage where applicable,
documentation, named unsupported edges, and a checkpoint through
`tools/checkpoint.sh` before commit.

## Immediate Next Work

The next implementation work should stay focused on the current largest gap:
reference/COW.

Highest-value concrete candidates:

- Broader mixed nested `ArrayAccess` chains below direct, property-held, and
  magic roots, especially object-chain bridges for nested append/keyed writes,
  reference-return bodies, callback reference arguments, and no-reference
  callback value paths where backing buckets can be statically proven.
- Broader but still proven `offsetGet()`, `__get()`, and ordinary method
  return-body alias shapes.
- Broader method-return or factory expression roots beyond the one covered
  magic/`ArrayAccess` reference-source backing-write shape.
- Broader lifetime/detach tests for covered magic and `ArrayAccess` aliases,
  especially destructor, shutdown, and error-ordering interactions.
- Native diagnostics for any reference/COW path that remains interpreter-only.

After those COW slices are coherent, move the same focused-lane discipline to
the next largest blocker: object/dynamic PHP, then request/SAPI/filesystem,
then real database/bootstrap behavior.

## Non-Goals

- Claiming full PHP support before executable evidence exists.
- Claiming WordPress support from placeholder or synthetic bootstrap success.
- Broad standard-library stubs that silently return plausible values.
- Regex parser rewrites.
- Native compilation of constructs that only the interpreter can currently
  execute.
