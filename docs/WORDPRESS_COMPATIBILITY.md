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
  probe now reaches
  `runtime error at <bootstrap-shim>:9:19: unsupported call closure: anonymous function values and invocation are not implemented`.
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
