# WordPress Source Pin

This fixture directory records the WordPress compatibility inventory policy. It
does not vendor WordPress core.

## Current External Target

- Version: WordPress 6.9.4
- Archive: `https://wordpress.org/wordpress-6.9.4.tar.gz`
- Local source variable: `WORDPRESS_ROOT`
- Inventory command:
  `tools/wordpress-inventory.sh --normalize "$WORDPRESS_ROOT"`
- Optional timeout override:
  `WORDPRESS_PROBE_TIMEOUT=10s tools/wordpress-inventory.sh --normalize "$WORDPRESS_ROOT"`
- Optional interpreter step budget:
  `PHPC_MAX_EXECUTION_STEPS=100 tools/wordpress-inventory.sh --normalize "$WORDPRESS_ROOT"`
- Optional include trace:
  `PHPC_TRACE_INCLUDES=1 tools/wordpress-inventory.sh --normalize "$WORDPRESS_ROOT"`
- Optional parser trace:
  `PHPC_TRACE_PARSE=1 tools/wordpress-inventory.sh --normalize "$WORDPRESS_ROOT"`

## Expected Output Policy

Committed inventory snapshots must use normalized output so local paths and
`PHPC_BIN` locations do not affect the fixture. The normalized form replaces:

- the WordPress checkout path with `<wordpress-root>`;
- the compiler executable path with `<phpc>`;
- the temporary bootstrap shim path with `<bootstrap-shim>`.

Each probe prints the timeout value and whether it timed out. The default
timeout is `30s` when the host provides GNU `timeout`; operators can override it
with `WORDPRESS_PROBE_TIMEOUT`. `PHPC_MAX_EXECUTION_STEPS` can diagnose
statement-execution loops, but it does not count parser/include/declaration
registration work. `PHPC_TRACE_INCLUDES=1` emits include paths to stderr before
parsing/execution so timeout runs preserve the current include frontier; the
inventory output records both first and last stderr lines. `PHPC_TRACE_PARSE=1`
emits parser frontier lines for top-level statements, class/interface/enum
members, and block statements; it is for timeout diagnosis and is not
PHP-visible output.

Do not commit WordPress core source into this repository until a separate size,
license, update, and checksum policy is accepted. The committed synthetic
inventory fixtures prove the output format, keep the direct `wp-settings.php`
probe visible, include a bootstrap-shim probe for the deterministic
`ABSPATH`/`$table_prefix` startup state, and include a silent
`wp-blog-header.php` front-controller smoke that exits `0` without
PHP-visible output. External WordPress source runs remain an operator-supplied
compatibility measurement. The shim does not load real database credentials,
salts, multisite constants, or host-specific settings; the front-controller
probe is a broader entry-flow smoke and must not be treated as plugin, theme,
admin, REST, database, HTTP, SAPI, or native WordPress support by itself.
