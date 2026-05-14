# Compatibility Manifest

This file tracks compatibility as evidence. It does not claim full PHP or
WordPress support. Each entry must be one of:

- `pass`: the named command or fixture passes for the documented target.
- `fail`: the named command or fixture runs and exposes a compiler/runtime bug.
- `skipped-unsupported`: the case is intentionally skipped with a named
  unsupported feature.
- `not-covered`: no committed harness exists yet.

## External Targets

Last checked: 2026-05-14.

- PHP supported branches: 8.2, 8.3, 8.4, and 8.5.
- WordPress modern host recommendation: PHP 8.3+, MariaDB 10.6+ or MySQL 8.0+,
  and HTTPS.
- WordPress legacy runtime floor documented by WordPress.org: PHP 7.2.24+ and
  MySQL 5.5.5+, with upgrade strongly recommended.

References:

- https://www.php.net/supported-versions.php
- https://wordpress.org/about/requirements/
- https://make.wordpress.org/core/handbook/references/php-compatibility-and-wordpress-versions/

## Target Policy

PHP compatibility targets should track currently supported PHP branches first.
Older PHP behavior can be added only when it is needed for a named application
target and does not blur current-branch behavior.

WordPress compatibility starts with a pinned WordPress core version in this
repo's harness. After the first harness exists, current stable WordPress can be
tracked deliberately by updating the pin and recording the resulting blockers.

## Current Smoke Commands

These commands measure the current project subset, not full PHP compatibility:

```sh
cargo run -p phpc -- test
cargo run -p phpc -- test --compare-php
cargo run -p phpc -- test tests/fixtures/compat/php
cargo run -p phpc -- test --compare-php tests/fixtures/compat/php
cargo run -p phpc -- compile examples/hello.php --emit-ir
cargo run -p phpc -- compile examples/hello.php --emit-asm
cargo test -p phpc --test wordpress_inventory_cli -- --test-threads=1
```

The full integration gate remains:

```sh
tools/run-tests.sh
```

## Compatibility Status

| Target | Command or Evidence | Status | Notes |
| --- | --- | --- | --- |
| Current fixture suite through `phpc run` | `cargo run -p phpc -- test` | `pass` | Measures the documented subset only. |
| Optional system PHP comparison | `cargo run -p phpc -- test --compare-php` | `pass`/`skipped-unsupported` | Compares supported fixtures when system PHP is available; `.phpc-only` fixtures are skipped intentionally. |
| Cross-feature PHP smoke fixture | `cargo run -p phpc -- test --compare-php tests/fixtures/compat/php` | `pass` | One committed smoke fixture spans constants, functions, arrays, callback builtin use, class metadata, public properties, conditionals, foreach, and system PHP comparison. |
| Current supported PHP branches 8.2-8.5 | Branch-specific comparison matrix | `not-covered` | The suite does not yet run against a matrix of PHP binaries or branch-specific expected behavior. |
| php-src-style language compatibility | Imported or mirrored behavioral tests | `not-covered` | No committed php-src compatibility subset exists yet. |
| Native executable compatibility | Linked native run command | `not-covered` | `phpc compile` emits IR/assembly only; no linked executable path exists yet. The first scalar runtime ABI prerequisite is documented in `docs/NATIVE_RUNTIME_ABI.md`. |
| WordPress inventory output harness | `cargo test -p phpc --test wordpress_inventory_cli -- --test-threads=1` | `pass` | Synthetic WordPress-shaped tree pins normalized direct-`wp-settings.php` and bootstrap-shim probes without vendoring WordPress core. |
| WordPress core parse/load inventory | `tools/wordpress-inventory.sh --normalize /path/to/wordpress` | `skipped-unsupported` | Inventory command and committed output policy exist for external WordPress 6.9.4 source; a real external-source snapshot still needs an operator-supplied checkout. |
| WordPress bootstrap | Non-networked bootstrap smoke command | `not-covered` | Blocked by include/require, namespaces, runtime environment, filesystem, database, and extension coverage. |
| WordPress request/admin/WP-CLI flows | Pinned smoke fixtures | `not-covered` | Requires a credible bootstrap harness first. |
| Representative WordPress plugins/themes | Pinned plugin/theme fixtures | `not-covered` | Requires WordPress core bootstrap and extension/environment support first. |

## First Blockers To Convert Into Work

- Add a PHP branch/version manifest that records which local or CI PHP binaries
  are available for comparison.
- Add a small `tests/fixtures/compat/php` smoke group that intentionally spans
  multiple language areas and records unsupported skips by name.
- Use the WordPress bootstrap-shim inventory result to choose the next real
  compiler/runtime blocker without vendoring WordPress core.
- Define the first native runtime ABI slice before claiming native executable
  compatibility.
