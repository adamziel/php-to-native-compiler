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

## Expected Initial Blockers

The first bootstrap probe is expected to fail. Known blockers include:

- include/require execution; the first pinned boundary is the WordPress-shaped
  `require ABSPATH . WPINC . '/load.php';` form used by bootstrap loading;
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
expected output policy are committed.
