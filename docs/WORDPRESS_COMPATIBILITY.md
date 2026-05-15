# WordPress Compatibility

This file defines the first WordPress compatibility target and inventory
workflow. It does not claim WordPress support.

## Target

Last checked: 2026-05-14.

- First pinned core target: WordPress 6.9.4, the latest release listed in the
  WordPress.org release archive on 2026-05-14.
- Future update target: WordPress 7.0 after its scheduled 2026-05-20 release.
- PHP host target: PHP 8.3+ first, matching the current WordPress.org
  recommendation.
- Legacy awareness: WordPress documents a runtime floor of PHP 7.2.24+, but
  this compiler should not optimize for legacy behavior until current supported
  PHP branches are represented.

References:

- https://wordpress.org/download/releases/
- https://make.wordpress.org/core/2026/04/22/wordpress-7-0-release-party-updated-schedule/
- https://wordpress.org/about/requirements/

## Source Policy

Do not vendor WordPress core into this repository until there is a clear size,
license, and update policy. The first harness uses an external checkout or
download supplied by `WORDPRESS_ROOT`.

Recommended local source layout:

```sh
mkdir -p /tmp/phpc-wordpress
cd /tmp/phpc-wordpress
curl -LO https://wordpress.org/wordpress-6.9.4.tar.gz
tar -xzf wordpress-6.9.4.tar.gz
```

Then run the inventory from this repository:

```sh
tools/wordpress-inventory.sh /tmp/phpc-wordpress/wordpress
```

For committed snapshots and tests, use normalized output:

```sh
tools/wordpress-inventory.sh --normalize "$WORDPRESS_ROOT"
```

Normalized output replaces the local WordPress checkout path with
`<wordpress-root>` and the compiler executable path with `<phpc>`. The committed
policy lives in `tests/fixtures/compat/wordpress/source-pin.md`. The repository
also keeps a synthetic WordPress-shaped inventory fixture so the output format
and current direct-`wp-settings.php` bootstrap blocker are tested without
vendoring WordPress core.

## Inventory Script

`tools/wordpress-inventory.sh` is the first repeatable WordPress measurement
tool. It reports:

- detected WordPress version from `wp-includes/version.php`;
- total PHP file count;
- rough syntax-surface counts for include/require, namespaces, imports,
  interfaces, traits, enums, inheritance, exceptions, closures, and arrow
  functions;
- a `phpc run wp-settings.php` bootstrap probe, including exit status and the
  first stderr line.

The syntax counts are intentionally coarse inventory data, not parser support
claims.

The committed synthetic harness is exercised by:

```sh
cargo test -p phpc --test wordpress_inventory_cli -- --test-threads=1
```

## Expected Initial Blockers

The first bootstrap probe is expected to fail. Known blockers include:

- include/require breadth beyond the first local
  `require`/`require_once`/`include`/`include_once` slice. The
  WordPress-shaped `require ABSPATH . WPINC . '/load.php';`, `require_once
  ABSPATH . WPINC . '/plugin.php';`, conditional `include WP_CONTENT_DIR .
  '/advanced-cache.php';`, and must-use plugin loop `include_once $mu_plugin;`
  forms are now executable or skippable in focused fixtures. Top-level
  `global ...;` declarations are accepted as no-ops. The inventory now reports
  both the direct `wp-settings.php` probe, which reaches undefined `ABSPATH`,
  and a bootstrap-shim probe, which defines `ABSPATH` before loading
  `wp-settings.php`. Against real WordPress 6.9.4, the previous
  `goto invalid_utf8;` blocker in `wp-includes/compat-utf8.php` is covered by
  the bounded Milestone 686 `goto`/label runtime slice, and the previous
  `(string)` cast blocker in that file is covered by the bounded Milestone 687
  cast slice. The previous function-local `static` blocker in
  `wp-includes/compat.php` is covered by the bounded Milestone 688 static-local
  runtime slice, and the previous anonymous closure syntax blocker in
  `compat.php:54` is covered by the Milestone 689 syntax-only closure slice.
  The previous alternate `if (...) : ... endif;` blocker in the bootstrap shim
  is covered by Milestone 690 through the existing conditional runtime path.
  The previous `instanceof Countable` blocker in `wp-includes/compat.php` is
  covered by the bounded Milestone 691 `instanceof` runtime slice. The shim
  probe's previous `extension_loaded()` blocker is covered by the bounded
  Milestone 692 empty extension-registry policy. The previous
  `PHP_VERSION_ID` blocker is covered by the bounded Milestone 693 PHP 8.3
  compatibility-target constant. The previous `dirname()` blocker is covered
  by the bounded Milestone 694 lexical Unix-style path builtin. The previous
  shim-probe `spl_autoload_register()` blocker is covered by the bounded
  Milestone 695 no-op autoload registration policy. The previous PHP attribute
  syntax blocker is covered by the bounded Milestone 696 syntax-only attribute
  skip. The previous unsupported `throw` syntax blocker is covered only by the
  bounded Milestone 697 throw-statement runtime boundary, not by real PHP
  exception support. The previous unsupported `try/catch/finally` syntax
  blocker is covered only by the bounded Milestone 698 try-block runtime
  boundary, not by real PHP exception execution. The previous unsupported
  `(int)` cast syntax blocker is covered by the bounded Milestone 699
  scalar/null integer-cast slice. The previous unsupported simple positional
  `list(...)` assignment blocker at `<bootstrap-shim>:419:9` is covered by
  Milestone 700. The previous unsupported `(bool)` cast blocker at
  `<bootstrap-shim>:572:16`, corresponding to
  `wp-includes/sodium_compat/src/Compat.php:572`, is covered by Milestone 701.
  The previous default-parameter constant-expression blocker at
  `<bootstrap-shim>:1714:23`, corresponding to
  `self::CRYPTO_GENERICHASH_BYTES` in
  `wp-includes/sodium_compat/src/Compat.php:1714`, is covered by Milestone 702.
  The previous nested class declaration blocker at `<bootstrap-shim>:7:5` is
  covered by Milestone 703. The previous metadata-only `Exception` blocker at
  `<bootstrap-shim>:7:5` is covered by Milestone 704. The previous namespace
  declaration/import blocker at `<bootstrap-shim>:2:1` is covered for the
  current class-name slice by Milestone 705. The previous interface declaration
  blocker at `<bootstrap-shim>:4:1` is covered for declared interface metadata
  by Milestone 706. The previous trait declaration blocker at
  `<bootstrap-shim>:5:1` is covered for empty declared trait metadata by
  Milestone 707. The previous enum declaration blocker at
  `<bootstrap-shim>:6:1` is covered for declared unit-enum metadata by
  Milestone 708. The previous arrow-function parse blocker at
  `<bootstrap-shim>:9:10` is covered for the current syntax-only closure
  boundary by Milestone 709. The previous missing-parent class blocker at
  `<bootstrap-shim>:7:1` is covered for already-declared namespaced parents by
  Milestone 710. The previous reached `try` blocker at `<bootstrap-shim>:9:1`
  is covered for non-throwing try-body execution by Milestone 711. The shim
  previous anonymous closure value blocker at `<bootstrap-shim>:9:19` is
  covered for inert no-capture closure values by Milestone 712. The previous
  arrow closure value blocker at `<bootstrap-shim>:10:10` is covered for inert
  arrow closure values by Milestone 713. The synthetic bootstrap-shim probe now
  exits 0 with no stderr. The real WordPress 6.9.4 bootstrap-shim inventory's
  previous namespace-scoped function declaration blocker at
  `<bootstrap-shim>:23:5` is covered by Milestone 714 for namespace-scoped
  declarations and unqualified calls. The previous real WordPress
  bootstrap-shim `defined()` blocker at `<bootstrap-shim>:997:6` for
  `\Sodium\CRYPTO_AUTH_BYTES` is covered by Milestone 715's bounded qualified
  runtime-constant name slice, along with the adjacent namespace-scoped
  `const` declarations reached by the sodium compatibility bootstrap. This is
  not full sodium support, full extension constant catalog support, or native
  lowering. The real inventory now reports direct `wp-settings.php` still
  stops at
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`,
  while the previous bootstrap-shim `assert()` blocker at
  `<bootstrap-shim>:68:9` is covered by Milestone 716 for truthy assertion
  guards used by the sodium compatibility bootstrap. This is not full PHP
  assertion policy, callback, `AssertionError`, warning/fatal, or native
  support. It then advanced to
  `runtime error at <bootstrap-shim>:106:10: unsupported call defined(): constant name must be a non-empty supported identifier or qualified name in the current subset, got SODIUM_$constant`.
  That previous dynamic constant-name probe is covered by Milestone 717's
  simple double-quoted `$name` interpolation slice for runtime string names.
  This is not full string interpolation, sodium support, extension constant
  catalog support, class-constant string lookup, or native lowering. The real
  inventory now reports direct `wp-settings.php` still stops at
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`,
  while the bootstrap-shim probe then reached
  `lex error at <bootstrap-shim>:468:12: unsupported string interpolation: only simple $name interpolation in double-quoted strings is implemented; braced/complex interpolation is not implemented`.
  That previous braced simple-variable interpolation blocker, corresponding to
  `wp-includes/compat-utf8.php:468` and `$utf8 .= "{$byte1}{$byte2}";`, is
  covered by Milestone 718 for simple `{$name}` parts only. This is not full
  complex interpolation, array-offset/object/static-property interpolation,
  `${...}`, heredoc/nowdoc, or native lowering. The bootstrap-shim probe now
  passes the `ParagonIE_Sodium_Compat::LIBRARY_VERSION_MAJOR`
  class-constant string lookup at `<bootstrap-shim>:106:41` through Milestone
  719's bounded `defined("ClassName::CONST")`/`constant("ClassName::CONST")`
  runtime slice for declared class metadata. This is not full class-constant
  string lookup for `self`/`parent`/`static`, autoload-triggered class
  discovery, enum cases/interface constants beyond current metadata,
  typed/static/multi-declarator class constants, exact PHP diagnostics,
  partial-output behavior, or native lowering. The bootstrap-shim probe now
  reaches
  `lex error at <bootstrap-shim>:665:56: unexpected character '@'`,
  corresponding to `wp-includes/sodium_compat/src/Core/Util.php:605` and
  `$c = (int) @($c & -1);`. That previous error-control syntax blocker is
  covered by Milestone 720 as a transparent runtime wrapper only. This is not
  actual PHP warning/notice/deprecation suppression, recoverable diagnostic
  severity, expression recovery values, `error_reporting()` mask behavior,
  exact PHP warning/fatal behavior, partial-output behavior, or native
  lowering. The bootstrap-shim probe now reaches
  `parse error at <bootstrap-shim>:1015:20: unsupported cast expression: only (string), (int), and (bool) casts are implemented`,
  likely corresponding to the sodium compatibility `(float)` cast at
  `wp-includes/sodium_compat/src/Core/Util.php:255`. Milestone 721 covers
  `(float)` and `(double)` for the current scalar/null subset only. This is
  not full leading-numeric warning/recovery behavior, non-finite string
  behavior, array/object/resource casts, exact PHP diagnostics,
  partial-output behavior, or native lowering. The bootstrap-shim probe now
  reaches
  `parse error at <bootstrap-shim>:1015:20: unsupported cast expression: only (string), (int), (bool), and (float) casts are implemented`,
  corresponding to the sodium compatibility `(array)` cast at
  `wp-includes/sodium_compat/src/PHP52/SplFixedArray.php:47` and
  `return (array) $this->internalArray;`.
  Milestone 722 covers that current null/scalar/array `(array)` cast slice.
  This is not object-to-array property materialization, mangled visibility-key
  behavior, Closure object array-cast behavior, resources,
  references/copy-on-write, exact PHP diagnostics, partial-output behavior, or
  native lowering. The bootstrap-shim probe now reaches
  `parse error at <bootstrap-shim>:1075:43: unsupported assignment expression target: only direct static variables, direct array offsets, direct append offsets, and direct object properties are implemented; nested targets are not implemented`.
  Milestone 723 covers direct-variable nested array-offset assignment
  expressions for the sodium compatibility context-array paths. This is not
  append-at-depth, nested compound assignment, nested `??=`, nested
  increment/decrement, mixed object/property/ArrayAccess targets,
  references/copy-on-write, exact PHP warning behavior, partial-output
  behavior, or native lowering. The bootstrap-shim probe now reaches
  `parse error at <bootstrap-shim>:1324:9: unsupported clone expression: object handle copying and __clone dispatch are not implemented`.
  Milestone 724 covers bounded `clone $object` expressions by allocating a
  fresh object handle and shallow-copying current property slots when the class
  does not declare `__clone`. This is not `__clone` dispatch, private/protected
  clone-method visibility behavior, reference/copy-on-write behavior,
  destructor/reuse behavior, exact PHP diagnostics, partial-output behavior, or
  native lowering. The bootstrap-shim probe now reaches
  `parse error at <bootstrap-shim>:1610:6: unsupported break: loop-depth arguments are not implemented; only 'break;' for the innermost loop is supported`,
  corresponding to `wp-includes/load.php:1610` and `break 2;`.
  Milestone 725 covers positive integer literal loop-depth control flow for
  `break N;` and `continue N;`, including `break 2;` out of a switch nested in
  a loop. This is not dynamic loop-depth expressions, zero/negative depth
  handling, exact PHP diagnostics, partial-output behavior, or native lowering.
  The bootstrap-shim probe now reaches
  `runtime error at <bootstrap-shim>:158:2: unsupported global declaration: importing globals into function scope is not implemented`.
  Real bootstrap still needs a faithful entrypoint policy, include-path/autoload
  behavior, source mapping, and PHP's warning/fatal details;
- namespace behavior beyond the current class-name/import slice;
- class inheritance, interface implementation/enforcement, trait
  members/composition, and modern object semantics;
- exceptions and PHP-shaped warning/error behavior;
- filesystem, streams, HTTP, database, JSON, XML, mbstring/intl, password/hash,
  date/time, sessions/cookies, and request superglobals;
- dynamic hooks, filters, callbacks, autoloading, plugin/theme discovery, and
  host state.

## First Non-Networked Smoke Target

The first WordPress smoke target is:

```text
Run `tools/wordpress-inventory.sh` against a local WordPress 6.9.4 tree and
record the first `phpc run wp-settings.php` blocker.
```

That target becomes a real compatibility fixture only after any external-source
snapshot is reviewed for stability and size. The normalized output policy now
exists, and the latest throwaway external-source run is recorded in
`docs/PROGRESS.md` rather than vendoring WordPress core.
