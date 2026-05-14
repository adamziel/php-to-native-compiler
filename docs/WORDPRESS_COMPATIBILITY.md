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

- include/require breadth beyond the first local `require` slice. The
  WordPress-shaped `require ABSPATH . WPINC . '/load.php';` path form is now
  executable in focused fixtures, but real bootstrap still needs broader file
  loading, `_once`, include-path/autoload behavior, source mapping, and PHP's
  warning/fatal details;
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

That target becomes a real compatibility fixture only after the source pin and
expected output policy are committed. The normalized output policy now exists;
the next step is an operator-supplied run against the pinned external WordPress
6.9.4 source tree, with any resulting snapshot reviewed before committing.
