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
  skip. The shim probe now reaches unsupported `throw` syntax at
  `<bootstrap-shim>:178:13`.
  Real bootstrap still needs a faithful entrypoint policy, include-path/autoload
  behavior, source mapping, and PHP's warning/fatal details;
- namespace and import resolution;
- class inheritance, interfaces, traits, and modern object semantics;
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
