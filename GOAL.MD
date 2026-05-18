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

The current in-progress implementation frontier is Lane 1721:

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
- `docs/PROGRESS.md` and `docs/LOOP_MEMORY.md` record the focused
  verification for this lane. Before a checkpoint commit, use
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
