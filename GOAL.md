# Goal: Full PHP and WordPress Compatibility

This project is moving from a small honest PHP subset toward a PHP-shaped
runtime and native compiler capable of running WordPress correctly.

The work so far has not been random feature chasing. It has been building the
substrate WordPress depends on:

- PHP values: boxed scalars, ordered PHP arrays, objects, resources, and stable
  diagnostics.
- Control flow and syntax: loops, switch, functions, classes, closures,
  includes, callbacks, and unsupported-feature boundaries.
- Runtime state: superglobals, sessions, cookies, headers, streams, filesystem
  probes, shutdown callbacks, and bounded request/SAPI behavior.
- WordPress-specific evidence: inventory fixtures and probes around bootstrap,
  options, object cache, `wpdb`, schema helpers, hooks, plugin/theme loading,
  and front-controller smoke paths.
- References and COW: many bounded slices proving aliasing, by-reference
  parameters, callback argument arrays, magic properties, `ArrayAccess`,
  copied arrays, object properties, and foreach reference behavior.
- Native path discipline: `phpc compile` emits LLVM IR/assembly for narrower
  supported code and rejects unsupported PHP lowering instead of generating
  misleading native code.

## Success Criteria

Full compatibility is not complete until these are true:

- `phpc run` can execute representative WordPress front-controller, admin,
  REST, plugin/theme, install/update, and CLI-style flows without unsupported
  runtime gaps.
- PHP language behavior is modeled with PHP-shaped values, references,
  copy-on-write, errors, object lifecycle, autoloading, namespaces, traits,
  interfaces, closures, generators, iterators, resources, and stream behavior
  where WordPress or common plugins rely on it.
- WordPress database, object-cache, options/transients, hooks, filesystem,
  HTTP, cron, media upload, plugin/theme loading, and REST behavior are covered
  by executable fixtures or inventory probes.
- `phpc compile` can either lower a feature to correct native/runtime helper
  calls or reject it with a precise documented boundary.
- Every support claim has implementation code, tests, a CLI exercise path,
  documentation, and named unsupported edges.

## Current Proof

The latest recorded full gate passed with:

- `1703` fixture tests passed, `0` failed.
- `1010` system PHP comparisons.
- `693` `phpc-only` skipped fixtures.

That proves the current bounded subset. It does not prove full PHP or full
WordPress compatibility.

## Missing Before WordPress Can Be Considered Supported

### PHP Reference and COW Model

- General PHP reference containers instead of mostly symbol-table alias
  metadata.
- Broad array/object copy-on-write identity, including nested arrays, objects,
  references, magic properties, and `ArrayAccess` interactions.
- Destructor and shutdown side effects during alias destruction and object
  lifecycle operations.
- Arbitrary expression-root reference targets and sources, not only bounded
  direct or temporarily rooted shapes.
- General magic-property and mixed `ArrayAccess` reference containers.
- Real `Iterator`, `IteratorAggregate`, and `Traversable` foreach semantics,
  including by-reference iteration where PHP allows it.
- Superglobal and included-scope lifetime fidelity.
- Native reference/COW lowering through runtime helper calls.

### Object Model and Dynamic PHP

- Constructors, destructors, cloning, autoloading, class aliases, traits,
  interfaces, abstract/final enforcement, visibility edge cases, late static
  binding breadth, and exact PHP object lifecycle ordering.
- `__call`, `__callStatic`, `__invoke`, `__sleep`, `__wakeup`, `__serialize`,
  `__unserialize`, and broader magic-method behavior.
- Namespaces, imports, qualified names, constants, attributes, enum boundaries,
  exceptions, throw/catch/finally, generators, `yield`, Fibers, and reflection
  breadth.
- Closure capture/reference behavior beyond the covered direct shapes.

### Request, SAPI, Streams, and Filesystem

- Real webserver/SAPI integration instead of deterministic CLI request seeds.
- Multipart upload parsing, temp-file lifecycle, upload validation, and media
  handling.
- Broader stream wrappers, filters, `phar://`, network streams, contexts,
  binary string fidelity, permissions, locking, and exact warning/fatal text.
- Output buffering, headers, cookies, sessions, shutdown order, cache headers,
  save handlers, garbage collection, and concurrency behavior at PHP parity.

### WordPress Runtime Surface

- Full bootstrap through `wp-settings.php` under realistic request state.
- Hooks/actions/filters behavior under real plugin/theme load order.
- `wpdb` SQL behavior beyond deterministic bounded schema/query slices.
- Object cache, options, transients, cron, rewrite rules, REST routing, admin
  screens, media, plugins, themes, block registration, and update/install flows.
- Compatibility probes for popular plugin and theme patterns, not only core
  synthetic fixtures.

### Native Compiler and Runtime ABI

- Native PHP array, object, string, resource, reference, and request-state
  handles.
- Generated calls into runtime helpers for arrays, objects, references,
  streams, WordPress host state, and diagnostics.
- Linked native execution for representative WordPress-compatible paths.
- Clear parity between interpreter support and native support boundaries.

## Roadmap Discipline

Work should proceed in focused compatibility lanes, but every lane must land as
evidence, not promises:

- Runtime semantics lane: references/COW, object lifecycle, iteration,
  exceptions, magic methods, dynamic calls, and PHP error behavior.
- Request/SAPI lane: sessions, headers, cookies, uploads, streams, filesystem,
  shutdown, output buffering, and host request state.
- WordPress lane: bootstrap probes, hooks, `wpdb`, options/cache/transients,
  REST/admin/plugin/theme flows, and real-world compatibility inventories.
- Native lane: runtime ABI helpers, generated helper calls, handles, linked
  execution, and precise rejection boundaries.
- Docs/tests lane: support matrix, architecture notes, progress log, fixture
  coverage, compare-PHP coverage, and checkpoint gates.

## Next Concrete Target

Continue from Milestone 1670 with the next highest-impact remaining COW
compatibility gap: `ArrayAccess` reference/COW targets. Milestone 1669 closed
the bounded WP_Hook-style iterator bucket case where by-value
`Iterator::current()` returns a copied public-property array bucket that is
then iterated by reference. The next step is proving bounded reference
assignment to `ArrayAccess` offset targets and side-effecting/bucket-copy
`offsetGet()` provenance. After that, continue into broader reference
containers, COW identity, SPL iterator behavior, and native reference lowering.

After that, keep moving through the missing areas above until the WordPress
inventory can run through real bootstrap flows without unsupported gaps.
