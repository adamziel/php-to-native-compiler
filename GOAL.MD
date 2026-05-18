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

The current in-progress implementation frontier is Lane 1775:

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
- Mixed and arbitrary nested `ArrayAccess` chains.
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

- Broader mixed nested `ArrayAccess` chains below magic-property roots,
  including by-value terminal/plain-array mutation and arbitrary mixed-chain
  shapes beyond the currently tested object-handle recursion.
- Broader but still proven `offsetGet()` and `__get()` return-body shapes.
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
