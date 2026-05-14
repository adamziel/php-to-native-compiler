# WordPress Source Pin

This fixture directory records the WordPress compatibility inventory policy. It
does not vendor WordPress core.

## Current External Target

- Version: WordPress 6.9.4
- Archive: `https://wordpress.org/wordpress-6.9.4.tar.gz`
- Local source variable: `WORDPRESS_ROOT`
- Inventory command:
  `tools/wordpress-inventory.sh --normalize "$WORDPRESS_ROOT"`

## Expected Output Policy

Committed inventory snapshots must use normalized output so local paths and
`PHPC_BIN` locations do not affect the fixture. The normalized form replaces:

- the WordPress checkout path with `<wordpress-root>`;
- the compiler executable path with `<phpc>`.

Do not commit WordPress core source into this repository until a separate size,
license, update, and checksum policy is accepted. The committed synthetic
inventory fixture proves the output format and current first bootstrap blocker;
external WordPress source runs remain an operator-supplied compatibility
measurement.
