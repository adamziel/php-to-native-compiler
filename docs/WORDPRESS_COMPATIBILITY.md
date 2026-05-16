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
- a `phpc run wp-settings.php` direct settings probe, including exit status and
  first/last stderr lines;
- a generated bootstrap-shim probe that defines `ABSPATH` and a conventional
  `$table_prefix` before requiring `wp-settings.php`;
- a `wp-blog-header.php` front-controller probe when that file exists, so
  post-bootstrap entry-flow blockers are tracked separately from the shim.

The syntax counts are intentionally coarse inventory data, not parser support
claims.

The committed synthetic harness is exercised by:

```sh
cargo test -p phpc --test wordpress_inventory_cli -- --test-threads=1
```

## Current Probe Status

The direct `wp-settings.php` probe still fails because it is not a valid
WordPress entrypoint without `ABSPATH`. The bootstrap-shim probe and the
`wp-blog-header.php` front-controller probe now exit `0` with no stdout under
the current deterministic placeholder database and CLI assumptions. Known
historical blockers and remaining full-support gaps include:

- include/require breadth beyond the first local
  `require`/`require_once`/`include`/`include_once` slice. The
  WordPress-shaped `require ABSPATH . WPINC . '/load.php';`, `require_once
  ABSPATH . WPINC . '/plugin.php';`, conditional `include WP_CONTENT_DIR .
  '/advanced-cache.php';`, and must-use plugin loop `include_once $mu_plugin;`
  forms are now executable or skippable in focused fixtures. Top-level
  `global ...;` declarations are accepted as no-ops. The inventory now reports
  both the direct `wp-settings.php` probe, which reaches undefined `ABSPATH`,
  and a bootstrap-shim probe, which defines `ABSPATH` and the conventional
  `$table_prefix = 'wp_';` startup variable before loading `wp-settings.php`.
  Against real WordPress 6.9.4, the previous
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
  Milestone 692 extension-registry policy, which Milestone 733 later widens for
  the current `json` and `hash` WordPress bootstrap requirement checks. The previous
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
  Milestone 726 covers bounded function-scope `global $name, ...;` imports for
  direct variable names by routing imported reads/writes through the root global
  symbol table and materializing missing imported globals as `null`. This is
  not PHP's full reference-backed aliasing, `$GLOBALS`, dynamic global names,
  superglobals, included-file scope behavior, copy-on-write, exact
  warning/notice behavior, partial-output behavior, or native lowering. The
  bootstrap-shim probe now reaches
  `runtime error at <bootstrap-shim>:160:17: undefined constant PHP_VERSION`.
  Milestone 727 covers the deterministic PHP 8.3 `PHP_VERSION` compatibility
  string for bare reads, `constant(...)`, and `defined(...)`, while keeping
  host patch-level coupling, component version constants, `phpversion()`,
  `version_compare()`, extension versions, exact diagnostics, partial-output
  behavior, and native constant values explicit. The bootstrap-shim probe now
  reaches
  `runtime error at <bootstrap-shim>:162:7: undefined function version_compare()`.
  Milestone 728 covers bounded `version_compare()` for numeric version strings
  and the operator forms needed by the first WordPress PHP-version guard. This
  is not PHP's full version grammar, pre-release ordering, arbitrary
  separators, invalid-argument warnings, `phpversion()`/extension version
  coupling, exact diagnostics, partial-output behavior, or native lowering.
  The bootstrap-shim probe now reaches
  `runtime error at <bootstrap-shim>:183:28: undefined function sprintf()`.
  Milestone 729 covers bounded `sprintf()` for literal text, `%%`, `%s`, and
  `%N$s` string placeholders used by the current bootstrap-shim message paths.
  This is not PHP's full format grammar, numeric formatting, width/precision
  handling, locale behavior, warning behavior, partial-output behavior, or
  native lowering. The bootstrap-shim probe now reaches
  `runtime error at <bootstrap-shim>:193:3: undefined function header()`.
  Milestone 730 covers bounded `header()` as a no-op web/SAPI boundary for a
  string header line plus optional bool replacement flag and integer response
  code. This is not response header storage, status-code state,
  replacement/removal behavior, output-sent warning behavior, SAPI/web-server
  integration, exact diagnostics, partial-output behavior, or native lowering.
  The bootstrap-shim probe now reaches
  `runtime error at <bootstrap-shim>:195:8: undefined function implode()`.
  Milestone 731 covers bounded `implode()` for current scalar/null array values
  joined by an empty default separator or a string separator. This is not the
  legacy reversed argument order, nested arrays, object/resource values, exact
  warning behavior, partial-output behavior, or native lowering. The
  bootstrap-shim probe now reaches
  `runtime error at <bootstrap-shim>:196:3: undefined function exit()`.
  Milestone 732 covers bounded direct `exit()`/`die()` termination with current
  stdout preservation and integer/string/null argument handling. This is not
  dynamic/callable invocation, exact status normalization, shutdown functions,
  destructor/finally ordering, output buffering, SAPI behavior, or native
  lowering. The bootstrap-shim probe now reaches WordPress' missing-extension
  guard and terminates with exit code `1`, 126 stdout bytes, and no stderr
  because the then-current `extension_loaded()` policy still reported an empty
  extension registry.
  Milestone 733 replaces that empty registry with a bounded compatibility
  registry that reports `json` and `hash` as loaded while leaving other
  extensions false. This is not host extension discovery, extension aliases,
  extension versions, native extension functions/constants, `php.ini`/SAPI
  integration, dynamic loading, exact diagnostics, partial-output behavior, or
  native extension support. The bootstrap-shim probe now reaches
  `runtime error at <bootstrap-shim>:203:8: undefined function file_exists()`.
  Milestone 734 adds bounded `file_exists()` execution for one string local
  path, rejecting stream-wrapper paths and returning host filesystem metadata
  existence for files and directories, including committed repo-relative
  source-map fixture paths. This is not include-path lookup, stream wrapper
  support, canonicalization/symlink policy, permissions/warning
  fidelity, open_basedir, stat caching, TOCTOU semantics, host-independent
  filesystem support, partial-output behavior, or native filesystem lowering.
  The bootstrap-shim probe now reaches
  `lex error at <bootstrap-shim>:3891:12: unsupported string interpolation: only simple $name and {$name} interpolation in double-quoted strings is implemented; array offsets, object/static properties, and complex interpolation are not implemented`.
  Milestone 735 adds bounded direct array-offset and object-property
  interpolation for `{$items['key']}`, `{$items[$key]}`, `$items[key]`,
  integer offset keys, and `{$object->property}`. This is not nested offsets,
  dynamic property names, static properties, `${...}`, variable variables,
  arbitrary expression interpolation, heredoc/nowdoc, exact diagnostics, or
  native lowering. The bootstrap-shim probe now reaches
  `lex error at <bootstrap-shim>:4225:9: unsupported heredoc/nowdoc string syntax: multiline string literals are not implemented`.
  Milestone 736 adds bounded heredoc/nowdoc syntax for unindented identifier
  labels, with heredoc using the current interpolation subset and nowdoc
  staying literal. This is not indentation stripping, full quoted-label
  handling, arbitrary label whitespace, malformed-label recovery, exact
  diagnostics, or native lowering. The bootstrap-shim probe now reaches
  `lex error at <bootstrap-shim>:7267:17: unsupported string interpolation: only simple $name, {$name}, direct array offsets, and direct object properties in double-quoted strings are implemented; ${...}, nested offsets, dynamic properties, static properties, and complex interpolation are not implemented`.
  Milestone 737 adds bounded chained property/array-offset interpolation for
  shapes such as `{$block->context['displayLayout']['columns']}`. This is not
  dynamic property names, static properties, `${...}`, variable variables,
  arbitrary expression interpolation, exact diagnostics, or native lowering.
  The bootstrap-shim probe now reaches
  `parse error at <bootstrap-shim>:856:4: expected expression, found static`.
  Milestone 738 adds bounded `static function (...) { ... }` parsing by
  reusing the current inert anonymous-closure value path. This is not static
  closure binding semantics, capture binding, invocation, callback
  integration, exact PHP `Closure` behavior, type enforcement, or native
  lowering. After this slice, the bootstrap-shim probe no longer reports the
  previous static-closure parse error, but it ran for more than a minute
  without a new diagnostic and was stopped manually; the next compatibility
  task is a timeout-bounded/instrumented shim probe to identify the new runtime
  path or loop.
  Milestone 739 adds timeout-bounded inventory probes. With
  `WORDPRESS_PROBE_TIMEOUT=10s`, the direct `wp-settings.php` probe still
  reaches the `ABSPATH` runtime error, while the bootstrap-shim probe reports
  `timed_out: yes`, exit `124`, zero stdout, and no stderr. This is not
  WordPress bootstrap support; it only makes the post-static-closure long path
  measurable and points the next lane at runtime tracing or an execution
  budget.
  Milestone 740 adds that runtime execution-step budget through
  `PHPC_MAX_EXECUTION_STEPS`, but the real bootstrap-shim probe still times out
  at `30s` with budgets as low as `100`, zero stdout, and no stderr. This
  means the current long path is not ordinary statement execution; the next
  measurement lane needs parser/include/declaration-registration tracing or a
  pre-execution budget.
  Milestone 741 adds include tracing and records the inventory's last stderr
  line. With `PHPC_TRACE_INCLUDES=1`, the current bootstrap-shim timeout's
  last include frontier is
  `<wordpress-root>/wp-includes/sodium_compat/src/Compat.php`; running that
  file directly also times out under a 10s outer timeout with no stderr. The
  next concrete compatibility lane is parser/declaration performance or
  budgeting for that large Sodium compatibility class.
  Milestone 742 adds parser frontier tracing, fixes the lexer's maintained byte
  offset so large files do not rescan already-consumed source for byte-prefix
  checks, and adds bounded user-function variadic parameters. After that, the
  real bootstrap-shim probe no longer times out in
  `wp-includes/sodium_compat/src/Compat.php` and advances to
  `parse error at <bootstrap-shim>:2099:16: unsupported for:
  comma-separated initializer, condition, or increment expression lists are not
  implemented; use at most one assignment or expression per header slot`.
  Variadic argument unpacking, by-reference variadics, type enforcement, exact
  PHP diagnostics, and native lowering remain unsupported.
  Milestone 743 adds comma-separated `for` header expression lists, executing
  initializer, condition, and increment lists left to right and using the final
  condition expression's truthiness. After that, the bootstrap-shim probe
  advances to `parse error at <bootstrap-shim>:3909:1: expected expression,
  found <`, likely around PHP close/open tag or inline HTML handling in the
  generated include path.
  Milestone 744 adds bounded inline HTML output between `?>` and the next PHP
  open tag, including single-newline consumption after close tags. After that,
  the bootstrap-shim probe advances to
  `parse error at <bootstrap-shim>:4451:14: unsupported dynamic property
  access: dynamic property names are not implemented`.
  Milestone 745 adds bounded dynamic object-property names for existing public
  slots and `stdClass` public dynamic slots. This covers the WordPress
  `_wp_json_sanity_check()` assignment shape `$output->$clean_id = ...` at the
  previous blocker. With
  `WORDPRESS_PROBE_TIMEOUT=30s PHPC_MAX_EXECUTION_STEPS=100000 PHPC_TRACE_INCLUDES=1`,
  the direct `wp-settings.php` probe still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe now exits without timing out, emits zero
  stdout bytes, starts stderr with
  `phpc trace include: <wordpress-root>/wp-settings.php`, and reaches
  `parse error at <bootstrap-shim>:4955:17: unsupported reference expression:
  references are not implemented`. This is not full dynamic-property support,
  reference support, or WordPress bootstrap support.
  Milestone 746 accepts statement-form by-reference assignment syntax as a
  runtime boundary for direct variable sources, which lets guarded and
  declaration-contained `$alias =& $value;` code parse without implementing
  aliasing. After that slice, the direct `wp-settings.php` probe still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches `parse error at <bootstrap-shim>:5047:28: unsupported foreach:
  by-reference iteration is not implemented; only by-value iteration is
  supported`. This is not reference support, by-reference foreach support, or
  WordPress bootstrap support.
  Milestone 747 accepts by-reference `foreach` value syntax as a runtime
  boundary, so `foreach ($items as &$value)` and
  `foreach ($items as $key => &$value)` can appear in guarded or
  declaration-contained code without blocking parse. Reached loops still fail
  with the stable unsupported by-reference iteration runtime diagnostic. After
  that slice, the direct `wp-settings.php` probe still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches `parse error at <bootstrap-shim>:5188:31: expected ';' after reference assignment`,
  corresponding to `wp-includes/functions.php` assigning a reference from an
  array offset source with `$input_array = &$input_array[ $path_element ];`.
  This is not executable by-reference foreach support, general reference
  assignment support, aliasing, copy-on-write, or WordPress bootstrap support.
  Milestone 748 widens statement-form by-reference assignment syntax to accept
  direct array-offset sources such as `$alias =& $array[$key];` as the same
  runtime boundary. After that slice, the direct `wp-settings.php` probe still
  reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches
  `parse error at <bootstrap-shim>:5463:28: unsupported assignment expression target: only direct static variables, direct array offsets, direct append offsets, and direct object properties are implemented; nested targets are not implemented`,
  corresponding to `wp-includes/functions.php` assigning into
  `$submenu['themes.php'][]`. This is not executable reference assignment,
  append-at-depth support, nested assignment support, copy-on-write, or
  WordPress bootstrap support.
  Milestone 749 implements append-at-depth assignment expressions for
  direct-variable nested array paths such as `$array[$key][] = expr` and
  `$array[$outer][$inner][] = expr`. After that slice, the direct
  `wp-settings.php` probe still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches
  `parse error at <bootstrap-shim>:3149:47: unsupported unset: only direct variables like unset($name), direct array offset removal like unset($array[$key]), and direct static property operands like unset(ClassName::$property) are implemented; object property, append, and nested unset forms are not implemented`,
  corresponding to `wp-includes/option.php` and
  `unset( $new_allowed_options[ $option_group ][ $pos ] );`. This is not
  nested `unset`, copy-on-write, exact PHP warning/fatal behavior, or WordPress
  bootstrap support.
  Milestone 750 implements nested direct-variable array-offset `unset(...)`
  for paths such as `unset($array[$outer][$inner])`. After that slice, the
  direct `wp-settings.php` probe still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches
  `parse error at <bootstrap-shim>:21:21: unsupported property default: instance property default values are not implemented`,
  corresponding to `wp-includes/pomo/mo.php` and `public $_nplurals = 2;`.
  This is not instance property default support, typed property support,
  copy-on-write, exact property initialization semantics, or WordPress
  bootstrap support.
  Milestone 751 implements untyped instance property defaults for the current
  constant-expression subset, including the previous `public $_nplurals = 2;`
  blocker. After that slice, the direct `wp-settings.php` probe still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches
  `parse error at <bootstrap-shim>:301:46: unsupported reference assignment: only direct variable and direct array-offset reference sources are parsed before reference semantics exist`,
  corresponding to `wp-includes/pomo/mo.php` and
  `$entry = &$this->make_entry( $original, $translation );`. This is not
  method-call reference assignment support, real PHP alias/reference
  semantics, typed property support, multi-property defaults,
  copy-on-write, exact property initialization semantics, or WordPress
  bootstrap support.
  Milestone 752 parses direct method-call reference-assignment sources as the
  existing runtime boundary. After that slice, the direct `wp-settings.php`
  probe still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches
  `parse error at <bootstrap-shim>:302:38: unsupported assignment expression target: only direct static variables, direct array offsets, direct append offsets, nested array offsets, append-at-depth targets, and direct object properties are implemented`,
  corresponding to `wp-includes/pomo/mo.php` and
  `$this->entries[ $entry->key() ] = &$entry;`. This is not object-property
  array-offset assignment target support, real PHP alias/reference semantics,
  copy-on-write, exact PHP diagnostics, or WordPress bootstrap support.
  Milestone 753 parses direct object-property array-offset reference-assignment
  targets as the existing runtime boundary. After that slice, the direct
  `wp-settings.php` probe still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches
  `parse error at <bootstrap-shim>:319:19: unsupported reference return: returning functions by reference is not implemented`,
  corresponding to `wp-includes/pomo/mo.php` and
  `public function &make_entry( $original, $translation )`. This is not
  by-reference return support, real PHP alias/reference semantics,
  copy-on-write, exact PHP diagnostics, or WordPress bootstrap support.
  Milestone 754 parses by-reference function and method return declarations as
  runtime boundaries. After that slice, the direct `wp-settings.php` probe
  still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches
  `parse error at <bootstrap-shim>:15:1: unsupported class modifier: abstract, final, and readonly class modifiers are not implemented`.
  This is not abstract/final/readonly class support, abstract-method
  enforcement, final inheritance/method enforcement, readonly class semantics,
  real PHP alias/reference semantics, copy-on-write, exact PHP diagnostics, or
  WordPress bootstrap support.
  Milestone 755 parses `abstract`, `final`, and `readonly` class modifiers plus
  `abstract`/`final` method modifiers as metadata, and rejects abstract class
  instantiation as a runtime boundary. After that slice, the direct
  `wp-settings.php` probe still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches
  `parse error at <bootstrap-shim>:63:26: unsupported magic class name: self, parent, and static class name resolution is not implemented`.
  This is not full magic class-name resolution, abstract-method implementation
  enforcement, final inheritance/method enforcement, readonly class semantics,
  PHP `Error` objects, exact diagnostics, or WordPress bootstrap support.
  Milestone 756 resolves `new self`, `new parent`, and `new static` in active
  class/method contexts and accepts no-argument forms without parentheses.
  After that slice, the direct `wp-settings.php` probe still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches
  `parse error at <bootstrap-shim>:131:70: unsupported assignment expression target: only direct static variables, direct array offsets, direct append offsets, nested array offsets, append-at-depth targets, and direct object properties are implemented`.
  This is not full dynamic class-name instantiation, anonymous classes, exact
  PHP `Error` objects, or WordPress bootstrap support.
  Milestone 757 implements direct-object-property nested array assignment and
  append-at-depth assignment for direct object variables and named properties.
  After that slice, the direct `wp-settings.php` probe still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches
  `parse error at <bootstrap-shim>:165:19: unsupported unset: object property unset is not implemented; property uninitialization, magic methods, and typed property semantics are not modeled`,
  corresponding to
  `unset( $this->loaded_translations[ $locale ][ $textdomain ][ $i ] );` in
  `wp-includes/l10n/class-wp-translation-controller.php`. This is not generic
  object/ArrayAccess target support, references, copy-on-write, exact PHP
  diagnostics, or WordPress bootstrap support.
  Milestone 758 implements nested object-property array-offset `unset(...)`
  for direct object variables and named properties while keeping plain
  `unset($object->property)` as an explicit boundary. After that slice, the
  direct `wp-settings.php` probe still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches
  `parse error at <bootstrap-shim>:24:13: unsupported include expression: expression-form include and include return values are not implemented; use statement-form include path; for existing local files`,
  corresponding to `$result = include $this->file;` in
  `wp-includes/l10n/class-wp-translation-file-php.php`. This is not
  expression-form include execution, include return-value behavior, generic
  dynamic include semantics, exact PHP diagnostics, or WordPress bootstrap
  support.
  Milestone 759 implements expression-form `include`, `include_once`,
  `require`, and `require_once` for the current local-file subset, including
  include return values, normal-completion return value `1`, and `_once`
  duplicate return value `true`. After that slice, the direct
  `wp-settings.php` probe still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches
  `parse error at <bootstrap-shim>:1557:9: unsupported array destructuring: only simple positional statement-form list($a, $b) = expr targets are implemented; short [...], expression-position list(...), nested, keyed, skipped, reference, and non-variable targets are not implemented`.
  This is not full include-path lookup, stream wrapper, phar, autoload,
  opcache, exact warning/fatal, or WordPress bootstrap support.
  Milestone 760 implements skipped positional slots in statement-form
  `list(...) = expr;`, covering
  `list( , $textdomain, $language ) = $match;` in `wp-includes/l10n.php`.
  After that slice, the direct `wp-settings.php` probe still reports
  `runtime error at <wordpress-root>/wp-settings.php:34:9: undefined constant ABSPATH`;
  the bootstrap-shim probe exits without timing out, emits zero stdout bytes,
  starts stderr with `phpc trace include: <wordpress-root>/wp-settings.php`,
  and reaches
  `parse error at <bootstrap-shim>:19:21: unsupported interface implementation: implements clauses are not implemented`.
  This is not short `[...]` destructuring, keyed/nested/reference
  destructuring, exact missing-offset warning behavior, native array
  destructuring lowering, or WordPress bootstrap support.
  Milestone 761 implements class `implements` metadata for comma-separated
  interface names and relationship checks through `is_a`, `is_subclass_of`,
  and `instanceof`, including inherited metadata and unresolved
  built-in/internal interface names as metadata-only relationships. It advances
  the real bootstrap-shim probe past
  `final class WP_Hook implements Iterator, ArrayAccess` in
  `wp-includes/class-wp-hook.php` to
  `runtime error at <bootstrap-shim>:1428:2: unsupported call reference assignment: references and aliasing are not implemented`,
  corresponding to `$l10n[ $domain ] = &$noop_translations;` in
  `wp-includes/l10n.php:1428`. This is not interface method enforcement,
  interface constants, interface inheritance, built-in/internal interface
  catalogs, variance/signature checks, autoload-triggered interface discovery,
  exact PHP fatal behavior, native lowering, or WordPress bootstrap support.
  Milestone 762 implements the bounded object-handle `=&` slice needed for
  that `NOOP_Translations` path: direct variable sources holding current object
  values can be assigned into direct variable or direct array-offset targets.
  It advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:231:20: undefined function str_replace()`.
  This is not scalar/array reference aliasing, source/target rebinding,
  by-reference array-offset or method-call sources, object-property array
  targets, copy-on-write, native lowering, or WordPress bootstrap support.
  Milestone 763 implements bounded three-argument `str_replace()` for
  scalar/null string-convertible arguments and advances the real
  bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:3839:2: undefined function call_user_func()`,
  corresponding to `call_user_func( $the_['function'] )` in
  `wp-includes/class-wp-hook.php:339`. This is not array
  search/replace/subject behavior, the `$count` output argument,
  object/resource coercion, exact warning behavior, binary string edge cases,
  native lowering, or WordPress bootstrap support.
  Milestone 764 implements bounded `call_user_func()` dispatch for string
  callables resolving to current user functions or documented callable
  builtins. It advances the real bootstrap-shim probe past
  `call_user_func( $the_['function'] )` in
  `wp-includes/class-wp-hook.php:339` to
  `runtime error at <bootstrap-shim>:4955:3: unsupported call reference assignment: references and aliasing are not implemented`,
  corresponding to `$parsed_args =& $args;` in
  `wp-includes/functions.php:4955`. This is not array callable dispatch,
  closure invocation, `__invoke`, `call_user_func_array`, references, variadic
  unpacking, exact PHP warning behavior, native lowering, or WordPress
  bootstrap support.
  Milestone 765 implements the bounded direct-variable array `=&` path needed
  for that `wp_parse_args()` assignment. Direct variable sources holding
  current array or object values can be assigned into direct variables; this
  stores the current value under the existing no-reference/no-copy-on-write
  value model. It advances the real bootstrap-shim probe to
  `runtime error at <bootstrap-shim>:108:9: undefined function strcasecmp()`,
  corresponding to `wp-includes/compat.php:108`. This is not alias cells,
  source/target rebinding, scalar references, nested/array-offset reference
  sources, direct array-offset targets for array values, object-property
  targets, copy-on-write, exact PHP diagnostics, native lowering, or WordPress
  bootstrap support.
  Milestone 766 implements bounded `strcasecmp()` for exactly two scalar/null
  string-convertible arguments with ASCII case folding. It advances the real
  bootstrap-shim probe past `wp-includes/compat.php:108` to
  `runtime error at <bootstrap-shim>:3890:10: undefined function headers_sent()`,
  corresponding to `wp-includes/functions.php:3890`. This is not broad scalar
  coercion, array/object/resource operands, binary/locale edge cases, exact PHP
  diagnostics, native lowering, or WordPress bootstrap support.
  Milestone 767 implements bounded no-argument `headers_sent()` as `false` for
  the current no-header-state runtime shim. It advances the real
  bootstrap-shim probe past `wp-includes/functions.php:3890` to
  `runtime error at <bootstrap-shim>:1469:9: undefined function abs()`,
  corresponding to `wp-includes/load.php:1469`. This is not filename/line
  output arguments, output-started tracking, header storage, output buffers,
  SAPI differences, exact warnings, native lowering, or WordPress bootstrap
  support.
  Milestone 768 implements bounded `abs()` for current integer and finite-float
  values, covering the reached `absint()` path after its explicit `(int)` cast.
  It advances the real bootstrap-shim probe past `wp-includes/load.php:1469`
  to
  `runtime error at <bootstrap-shim>:1547:2: undefined function header_remove()`,
  corresponding to `wp-includes/functions.php:1547`. This is not
  integer-minimum overflow, numeric string coercion, bool/null coercion,
  array/object/resource operands, NaN/infinity behavior, exact diagnostics,
  native lowering, or WordPress bootstrap support.
  Milestone 769 implements bounded `header_remove()` as a no-op for no argument
  or one string header name. It advances the real bootstrap-shim probe past
  `wp-includes/functions.php:1547` into WordPress'
  `wp_check_php_mysql_versions()` guard in `wp-includes/load.php:202`; the
  probe exits with status `1` and emits WordPress' missing `mysqli` extension
  HTML page. This is not response header storage/removal, output-sent warnings,
  SAPI behavior, database or mysqli support, native lowering, or WordPress
  bootstrap support.
  Milestone 770 implements an explicit `mysqli_connect` database-extension
  boundary: function/callability metadata and dynamic lookup see the name, but
  attempted connection calls fail with a stable unsupported-database runtime
  diagnostic. It advances the real bootstrap-shim probe past the missing-MySQL
  guard to
  `runtime error at <bootstrap-shim>:39:6: undefined variable '$wp_filter'`,
  corresponding to the early `if ( $wp_filter )` check in
  `wp-includes/plugin.php:39`. This is not mysqli extension support, database
  I/O, resources/objects, result sets, escaping, errors, PDO support, native
  lowering, or WordPress bootstrap support.
  Milestone 771 implements the narrow top-level `global` materialization
  behavior needed for that `$wp_filter` check: missing names declared by a
  top-level `global` statement become `null` and falsey, while ordinary
  undefined variable reads remain stable runtime errors. The real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:39:33: undefined function microtime()`.
  This is not full PHP warning/notice behavior, general undefined-variable
  recovery, variable variables, references, superglobals, native lowering, or
  WordPress bootstrap support.
  Milestone 772 implements bounded `microtime(true)` as a finite host-clock
  float seconds value and exposes the name through function/callability
  metadata. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:42:23: undefined function ini_get()`.
  This is not the no-argument/string-return `microtime()` format, deterministic
  time virtualization, precision/monotonicity guarantees, INI/timezone policy,
  exact diagnostics, native lowering, or WordPress bootstrap support.
  Milestone 773 implements bounded deterministic `ini_get()` registry reads,
  including the reached `memory_limit` option. The real bootstrap-shim probe
  now advances to
  `runtime error at <bootstrap-shim>:1688:11: undefined function strtolower()`,
  corresponding to `wp-includes/load.php:1688`. This is not host php.ini
  discovery, mutable INI state, `ini_set()`/`ini_get_all()`, SAPI differences,
  full option catalogs, exact diagnostics, native lowering, or WordPress
  bootstrap support.
  Milestone 774 implements bounded `strtolower()` for current scalar/null
  string-convertible values with ASCII lowercase mapping. The real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:1688:23: undefined function trim()`,
  corresponding to `wp-includes/load.php:1688`. This is not locale-sensitive
  case mapping, full Unicode case folding, binary string compatibility beyond
  valid UTF-8 runtime strings, array/object/resource coercions, exact
  diagnostics, native lowering, or WordPress bootstrap support.
  Milestone 775 implements bounded default-mask `trim()` for current
  scalar/null string-convertible values. The real bootstrap-shim probe now
  advances to
  `runtime error at <bootstrap-shim>:1689:11: unsupported call (int): leading-numeric string cast behavior is not implemented`,
  corresponding to `wp-includes/load.php:1689`. This is not custom character
  mask support, binary/null-byte string compatibility beyond the current
  represented runtime-string subset, array/object/resource coercions, exact
  diagnostics, native lowering, or WordPress bootstrap support.
  Milestone 776 implements bounded leading-numeric `(int)` string casts for
  the reached shorthand memory-limit path. The real bootstrap-shim probe now
  advances to
  `runtime error at <bootstrap-shim>:1691:7: undefined function str_contains()`,
  corresponding to `wp-includes/load.php:1691`. This is not exact PHP warning
  recovery for leading-numeric strings, full numeric grammar compatibility,
  non-finite/out-of-range cast support, array/object/resource coercions, exact
  diagnostics, native lowering, or WordPress bootstrap support.
  Milestone 777 implements bounded `str_contains()` for the reached shorthand
  memory-limit suffix checks. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:1700:9: undefined function min()`,
  corresponding to `wp-includes/load.php:1700`. This is not binary string
  compatibility beyond valid UTF-8 runtime strings, array/object/resource
  coercions, exact diagnostics, native lowering, or WordPress bootstrap
  support.
  Milestone 778 implements bounded integer `min()` and `PHP_INT_MAX` for the
  reached memory-limit clamp. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:1724:14: unsupported call isset(): only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported`,
  corresponding to the nested `isset( $ini_all[ $setting ]['access'] )` check
  in `wp-includes/load.php:1724`. This is not array-form `min()`, mixed-type
  comparison rules, broad `isset(...)` expression support, exact diagnostics,
  native lowering, or WordPress bootstrap support.
  Milestone 779 implements bounded direct-variable rooted nested array-offset
  `isset(...)` for that reached path. The real bootstrap-shim probe now
  advances to
  `runtime error at <bootstrap-shim>:87:38: undefined function is_readable()`,
  corresponding to `wp-includes/error-protection.php:87`. This is not
  non-variable array roots, object dimensions, nested `empty(...)`/`??` parity,
  PHP warning/notice suppression details, references/copy-on-write, exact
  diagnostics, native lowering, or WordPress bootstrap support.
  Milestone 780 implements bounded local-path `is_readable()` for that reached
  check. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:95:2: undefined function register_shutdown_function()`,
  corresponding to `wp-includes/error-protection.php:95`. This is not full
  filesystem support, stream wrappers, include-path lookup, portable permission
  modeling, stat-cache behavior, exact diagnostics, native lowering, shutdown
  function semantics, or WordPress bootstrap support.
  Milestone 781 implements bounded `register_shutdown_function()` registration
  for the reached fatal-error-handler callable. The real bootstrap-shim probe
  now advances to
  `runtime error at <bootstrap-shim>:73:1: undefined function date_default_timezone_set()`.
  This is not shutdown callback execution, callback ordering, argument
  delivery, fatal-error context, output-buffer/destructor/finally interaction,
  exact diagnostics, native lowering, date/timezone support, or WordPress
  bootstrap support.
  Milestone 782 implements bounded `date_default_timezone_set('UTC')` for the
  reached startup path. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:42:50: undefined variable '$_SERVER'`,
  corresponding to the `wp_fix_server_vars()` startup path. This is not full
  date/timezone support, timezone database validation, global timezone state,
  warning behavior, native lowering, request/SAPI superglobals, or WordPress
  bootstrap support.
  Milestone 783 implements bounded deterministic CLI request startup state:
  a seeded root `$_SERVER` array for the reached `wp_fix_server_vars()` path
  and `PHP_SAPI` as `cli`. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:46:35: undefined function preg_match()`.
  This is not full regex support, full web/SAPI request state, environment
  import policy, `$GLOBALS` aliasing, references/copy-on-write, cookies,
  uploads, all superglobals, header state, exact warning behavior, native
  lowering, or WordPress bootstrap support.
  Milestone 784 implements bounded `preg_match()` for the reached
  `wp_fix_server_vars()` SAPI-name pattern. The real bootstrap-shim probe now
  advances to
  `runtime error at <bootstrap-shim>:635:3: undefined function error_reporting()`.
  This is not full PCRE support, captures/matches output, flags, offsets,
  invalid-pattern warning behavior, byte/Unicode regex fidelity, exact
  diagnostics, native lowering, or WordPress bootstrap support.
  Milestone 785 implements bounded `error_reporting()` integer mask state and
  the reached `E_*` constants. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:666:10: undefined function is_dir()`.
  This is not PHP warning/notice/deprecation filtering, ini integration,
  disabled-function policy, exact diagnostics, native lowering, or WordPress
  bootstrap support.
  Milestone 786 implements bounded `is_dir()` for local one-string paths. The
  real bootstrap-shim probe now advances to
  `parse error at <bootstrap-shim>:124:22: expected property name after '->', found {`,
  corresponding to a braced dynamic object-property access path. This is not
  full filesystem support, stream wrappers, symlink/canonicalization policy,
  permissions/open_basedir behavior, stat-cache fidelity, native lowering,
  dynamic object-property support, or WordPress bootstrap support.
  Milestone 787 implements braced dynamic object-property names for the
  existing public-slot/`stdClass` dynamic-property runtime subset. The real
  bootstrap-shim probe now advances to
  `parse error at <bootstrap-shim>:173:7: unsupported magic constant __METHOD__: method context evaluation requires method dispatch, which is not implemented`.
  This is not full dynamic-property support, magic property hooks, non-public
  dynamic visibility, references/copy-on-write, native lowering, `__METHOD__`
  support, or WordPress bootstrap support.
  Milestone 788 implements bounded `__METHOD__` evaluation for the current
  function/method-context subset. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:53:2: undefined function set_error_handler()`.
  This is not full magic-constant support, trait/closure magic context,
  exact namespace/source mapping, native lowering, error-handler registration,
  or WordPress bootstrap support.
  Milestone 789 implements bounded `set_error_handler()` registration for
  current callable shapes and integer masks. The real bootstrap-shim probe now
  advances to
  `runtime error at <bootstrap-shim>:54:3: unsupported call closure: closure capture binding is not implemented`.
  This is not handler invocation, warning/notice/deprecation routing,
  `restore_error_handler()`, exact PHP error handling, closure capture binding,
  native lowering, or WordPress bootstrap support.
  Milestone 790 implements explicit `use (...)` capture binding for inert
  closure values, enough to register the reached WordPress error handler
  callback. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:70:2: unsupported call preg_match(): pattern modifiers are not implemented in the current subset`.
  This is not closure invocation, true by-reference aliasing, copy-on-write,
  callback execution, warning/error-handler routing, full regex/PCRE support,
  native lowering, or WordPress bootstrap support.
  Milestone 791 implements bounded `preg_match()` `u`-modifier handling for
  the reached `_wp_can_use_pcre_u()` startup probe. The real bootstrap-shim
  probe now advances to
  `runtime error at <bootstrap-shim>:71:2: undefined function restore_error_handler()`.
  This is not captures/matches output, flags, offsets, broad PCRE/modifier
  support, warning/error-handler routing, native lowering, or WordPress
  bootstrap support.
  Milestone 792 implements bounded `restore_error_handler()` cleanup for the
  reached `_wp_can_use_pcre_u()` path. The real bootstrap-shim probe now
  advances to
  `parse error at <bootstrap-shim>:254:31: expected property name after '->', found public`.
  This is not true handler-stack behavior, handler invocation,
  warning/error-handler routing, native lowering, keyword-named object-property
  parsing, or WordPress bootstrap support.
  Milestone 793 implements keyword-named object-property parsing after `->` for
  the reached `$object->public` path. The real bootstrap-shim probe now
  advances to
  `parse error at <bootstrap-shim>:418:48: unsupported array reference element: references are not implemented`.
  This is not keyword method calls, full dynamic property behavior,
  references/aliasing, copy-on-write, native lowering, or WordPress bootstrap
  support.
  Milestone 794 implements bounded array literal reference elements for the
  reached `array( &$this )` path by evaluating the current value without
  creating an alias. The real bootstrap-shim probe now advances to
  `parse error at <bootstrap-shim>:671:32: unsupported reference assignment: only direct variable, direct array-offset, and method-call reference sources are parsed before reference semantics exist`.
  This is not true reference aliasing, reference containers, copy-on-write,
  by-reference hook argument mutation, native lowering, or WordPress bootstrap
  support.
  Milestone 795 implements bounded object-property reference-assignment sources
  for the reached `$GLOBALS['posts'] = & $wp_query->posts` path by copying the
  current array/object value without creating an alias. The real bootstrap-shim
  probe now advances to
  `parse error at <bootstrap-shim>:832:15: unsupported unset: object property unset is not implemented; property uninitialization, magic methods, and typed property semantics are not modeled`.
  This is not true reference aliasing, reference containers, copy-on-write,
  object-property unset, native lowering, or WordPress bootstrap support.
  Milestone 796 implements bounded direct and dynamic object-property
  `unset(...)` operands for the reached `unset($this->$name)` path in
  `wp-includes/class-wpdb.php:832`. It nulls the current visible property slot
  instead of modeling true property removal or uninitialization. The real
  bootstrap-shim probe now advances to
  `parse error at <bootstrap-shim>:4127:38: unsupported magic constant __CLASS__: class context evaluation requires class-context tracking, which is not implemented`.
  This is not typed-property uninitialization, physical dynamic-slot removal,
  magic `__unset` dispatch, references/copy-on-write, native lowering, or
  WordPress bootstrap support.
  Milestone 797 implements bounded executable `__CLASS__` evaluation in method
  class context for the reached `wp_debug_backtrace_summary( __CLASS__ )` path
  in `wp-includes/class-wpdb.php:4127`. The real bootstrap-shim probe now
  advances to
  `runtime error at <bootstrap-shim>:1970:3: undefined function mysqli_report()`.
  This is not trait/namespace magic-constant support, anonymous-class exact
  names, source-mapping fidelity, native lowering, real mysqli extension
  behavior, or WordPress bootstrap support.
  Milestone 798 implements a bounded `mysqli_report()` report-mode boundary
  for the reached `mysqli_report( MYSQLI_REPORT_OFF )` path in
  `wp-includes/class-wpdb.php:1970`. It defines the reached report constants,
  accepts `MYSQLI_REPORT_OFF` and `MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT`,
  stores the current mode, and returns `true`. The real bootstrap-shim probe
  now advances to
  `runtime error at <bootstrap-shim>:1972:16: undefined function mysqli_init()`.
  This is not real mysqli extension state, `mysqli_init()`, connections,
  resources/objects, query/result behavior, warning/error routing, exact
  diagnostics, native lowering, or WordPress bootstrap support.
  Milestone 799 implements a bounded `mysqli_init()` placeholder-handle
  boundary for the reached `wpdb::db_connect()` startup path after
  `mysqli_report()`. It returns a placeholder `mysqli` object with
  `connect_errno = 0` and `connect_error = null`, and the real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:2082:17: undefined function strpos()`.
  This is not real mysqli initialization, connection state, host I/O,
  resources, query/result behavior, escaping, charset handling,
  warning/error routing, exact diagnostics, native lowering, or WordPress
  bootstrap support.
  Milestone 800 implements a bounded `strpos()` runtime slice for the reached
  `parse_db_host()` path. It supports scalar/null string-convertible haystack
  and needle arguments, optional integer offsets, empty needles, negative
  offsets, byte-position matching, and `false` for no match. The real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:2092:8: undefined function substr_count()`.
  This is not full PHP string/encoding/coercion diagnostics, native lowering,
  or WordPress bootstrap support.
  Milestone 801 implements bounded `substr_count()` for the reached
  `parse_db_host()` path. It supports scalar/null string-convertible haystack
  and needle arguments, optional integer offset/length slicing, negative
  offsets and lengths within the current bounds rules, non-overlapping
  byte-position counts, and short-slice zero counts. The real bootstrap-shim
  probe now advances to
  `runtime error at <bootstrap-shim>:2101:14: unsupported call preg_match(): matches output, flags, and offset arguments are not implemented; pass exactly two arguments in the current subset`.
  This is not full PHP string/encoding/coercion diagnostics, native lowering,
  or WordPress bootstrap support.
  Milestone 802 implements bounded `preg_match()` direct matches-output support
  for the reached `parse_db_host()` path. It preserves the existing
  two-argument literal regex subset, accepts a third direct `$matches`
  variable, clears matches on no match, and recognizes the two exact WordPress
  db-host named-capture patterns for current IPv4-ish and bracketed IPv6-ish
  startup paths. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:1997:5: undefined function mysqli_real_connect()`.
  This is not full PCRE, general capture-group behavior, flags, offsets,
  invalid-pattern warnings, full single-quoted string escape fidelity, native
  lowering, real database connectivity, or WordPress bootstrap support.
  Milestone 803 implements a bounded `mysqli_real_connect()` placeholder
  success boundary for the reached `wpdb::db_connect()` path. It accepts the
  current WordPress call shape for the placeholder `mysqli` object, records
  `connect_errno = 0` and `connect_error = null`, and returns `true` without
  opening a host connection. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:4138:10: undefined function preg_replace()`.
  This is not real database connectivity, authentication, query/result
  behavior, escaping, charset handling, warning/error routing, exact
  diagnostics, PDO, native database lowering, or WordPress bootstrap support.
  Milestone 804 implements bounded `preg_replace()` for the reached
  `wpdb::db_version()` database-version cleanup path. It supports exactly
  `/[^0-9.].*/` with an empty replacement and scalar/null subject, returning
  the leading ASCII digits/dots prefix. The real bootstrap-shim probe now
  advances to
  `runtime error at <bootstrap-shim>:4149:10: undefined function mysqli_get_server_info()`.
  This is not full PCRE replacement behavior, arrays, callbacks, limit/count
  output, invalid-pattern warnings, exact diagnostics, native lowering, real
  database server metadata, or WordPress bootstrap support.
  Milestone 805 implements bounded `mysqli_get_server_info()` for the
  placeholder `mysqli` object and returns deterministic
  `8.0.0-phpc-placeholder` server info. The real bootstrap-shim probe now
  advances to
  `runtime error at <bootstrap-shim>:904:10: undefined function compact()`.
  This is not real server negotiation, connection-state validation, host I/O,
  extension resources, query/result behavior, exact diagnostics, native
  database lowering, or WordPress bootstrap support.
  Milestone 806 implements bounded `compact()` for the reached
  `wpdb::determine_charset()` path. It accepts direct string variable names,
  reads the current caller scope, omits missing variables, and returns an array
  keyed by found names. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:951:11: undefined function mysqli_query()`.
  This is not full `compact()` array/nested argument behavior,
  variable-variable interaction, warning behavior, native lowering, real query
  execution, result resources, or WordPress bootstrap support.
  Milestone 807 implements bounded `mysqli_query()` for the reached
  `wpdb::set_sql_mode()` path. It accepts the placeholder `mysqli` object and
  exactly `SELECT @@SESSION.sql_mode`, returning `false` as a deterministic
  empty/no-result boundary. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:1203:14: undefined function mysqli_select_db()`.
  This is not real query execution, result resources, row iteration,
  SQL errors/warnings, connection state, native database lowering, or
  WordPress bootstrap support.
  Milestone 808 implements bounded `mysqli_select_db()` for the placeholder
  `mysqli` object and string/null database names, returning deterministic
  `true`. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:143:28: undefined variable '$table_prefix'`.
  This is not real database selection, database existence validation,
  connection state, host I/O, exact diagnostics, native database lowering, or
  WordPress bootstrap support.
  Milestone 809 defines the conventional `$table_prefix = 'wp_';` startup
  variable in the inventory bootstrap shim and proves it through the synthetic
  included-file inventory fixture. The real bootstrap-shim probe now advances
  to
  `runtime error at <bootstrap-shim>:1006:8: unsupported call preg_match(): only slash-delimited patterns are implemented in the current subset`.
  This is not real `wp-config.php` loading, database credentials, salts/keys,
  multisite table-prefix validation, host-specific settings, native lowering,
  or WordPress bootstrap support.
  Milestone 810 implements the exact WordPress table-prefix validation
  `preg_match()` guard `|[^a-z0-9_]|i`, returning no match for conventional
  prefixes such as `wp_` and a match for the first invalid character. The real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:1034:5: undefined property wpdb::$categories`.
  This is not arbitrary regex delimiter support, broad character-class or
  case-insensitive PCRE behavior, invalid-pattern warnings, native regex
  lowering, or WordPress bootstrap support.
  Milestone 811 implements bounded dynamic public property materialization for
  the WordPress `wpdb` compatibility class so reached table-name assignments
  such as `$this->$table = $prefixed_table` can create `categories` and related
  slots. The real bootstrap-shim probe now advances to
  `parse error at <bootstrap-shim>:499:3: unsupported compound assignment target: only direct static variables, direct array offsets, direct object properties, and supported static properties are implemented; append offsets and nested targets are not implemented`.
  This is not general `#[AllowDynamicProperties]`, magic property behavior,
  non-public dynamic access, exact notices/deprecations, references,
  copy-on-write, native lowering, or WordPress bootstrap support.
  Milestone 812 implements direct object-property array-offset compound
  assignment for the reached `WP_Object_Cache::incr()` mutation
  `$this->cache[ $group ][ $key ] += $offset;`. The real bootstrap-shim probe
  now advances to
  `runtime error at <bootstrap-shim>:359:2: unsupported call add_global_groups(): receiver must be object, got null`.
  This is not append-offset compound assignment, nested variable compound
  assignment, mixed object/property/ArrayAccess targets,
  references/copy-on-write, native lowering, real object-cache bootstrap, or
  WordPress bootstrap support.
  Milestone 813 implements bounded direct `$GLOBALS['name']` root-symbol
  routing for the reached `wp_cache_init()` object-cache assignment, so later
  `global $wp_object_cache;` imports see the object written through
  `$GLOBALS['wp_object_cache']`. This is not full PHP `$GLOBALS` array aliasing,
  recursive `$GLOBALS` materialization, non-string keyed `$GLOBALS`, dynamic
  globals, references/copy-on-write, exact warning/notice behavior, or native
  lowering. Milestone 814 implements direct object-property array-offset
  `isset(...)` for the reached `WP_Hook::add_filter()` priority check
  `$priority_existed = isset( $this->callbacks[ $priority ] );`. This is not
  arbitrary object-dimension `isset(...)`, dynamic property paths, ArrayAccess,
  references/copy-on-write, exact diagnostics, or native lowering. The real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:98:4: undefined function ksort()`.
  Milestone 815 implements bounded `ksort($array, SORT_NUMERIC)` for direct
  variable arrays and direct object-property arrays, covering the reached
  `WP_Hook::add_filter()` priority ordering path. This is not full PHP sort
  semantics, `SORT_REGULAR`, `SORT_STRING`, natural/locale sorts, mixed
  non-numeric key comparison, broad by-reference argument handling, exact
  diagnostics, or native lowering. The real bootstrap-shim probe now advances
  to
  `runtime error at <bootstrap-shim>:1780:7: unsupported call wp_cache_get(): reference parameter invocation is not implemented`.
  Milestone 816 allows omitted optional by-reference parameters to use their
  default value without binding aliases, covering the reached
  `wp_cache_get( 'notoptions', 'options' )` shape where `$found` is omitted.
  This is not real reference parameter binding, output-parameter writes, alias
  cells, references/copy-on-write, exact diagnostics, or native lowering. The
  real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:154:9: unsupported call get(): reference parameter invocation is not implemented`,
  corresponding to `WP_Object_Cache::get(..., $found)` with a provided direct
  variable output parameter.
  Milestone 817 implements bounded direct-variable by-reference parameter
  copy-back, covering `WP_Object_Cache::get(..., $found)` setting `$found` on
  the reached cache-miss path. This is not true alias binding during
  execution, non-variable reference arguments, rebinding aliases, reference
  containers, copy-on-write, exact diagnostics, or native lowering. The real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:1283:15: undefined function mysqli_real_escape_string()`,
  corresponding to the reached `wpdb::_real_escape()` option lookup path.
  Milestone 818 implements bounded `mysqli_real_escape_string()` for the
  reached `wpdb::_real_escape()` path. It accepts the existing placeholder
  `mysqli` object and scalar/null string-convertible data, returning
  deterministic MySQL-style escaping for NUL, newline, carriage return,
  backslash, quotes, and Ctrl-Z. This is not real connection charset state,
  host database behavior, warning/error routing, binary or invalid-string
  fidelity, exact escaping edge cases, SQL execution, or native lowering. The
  real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:2422:71: undefined function rand()`,
  corresponding to `wp-includes/class-wpdb.php:2422` in
  `wpdb::placeholder_escape()`.
  Milestone 819 implements bounded deterministic no-argument `rand()` for the
  reached salt path in `wpdb::placeholder_escape()`. This is not PHP random
  state compatibility, min/max arguments, swapped bounds, seeding,
  `mt_rand()`/`srand()` coupling, cryptographic randomness, exact diagnostics,
  or native lowering. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:2424:25: undefined function hash_hmac()`,
  corresponding to the adjacent placeholder hashing call in
  `wp-includes/class-wpdb.php:2424`.
  Milestone 820 implements bounded deterministic `uniqid()` and
  HMAC-SHA256 `hash_hmac()` for the reached
  `wpdb::placeholder_escape()` hash expression, using Rust `hmac`/`sha2`
  crates for the digest. This is not broad PHP hash extension support,
  `hash()`, `hash_equals()`, `hash_hmac_algos()`, algorithms beyond SHA-256,
  raw binary output, exact entropy/time behavior, cryptographic guarantees for
  generated IDs, exact diagnostics, or native lowering. The real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:241:8: unsupported comparison: strict identity for arrays is not implemented`.
  Milestone 821 implements strict identity for current ordered arrays by
  comparing identical keys in insertion order and recursively comparing values
  with strict identity over the implemented value model. This covers the
  reached empty-array/list-shape comparison paths such as
  `array() === $value` and `array_values($arr) === $arr`. This is not reference
  identity, copy-on-write semantics, recursive-array cycle handling, resource
  support, Closure object identity, exact PHP diagnostics, or native lowering.
  The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:3495:12: undefined function ltrim()`,
  likely in the localization path around `wp-includes/l10n.php:1051`.
  Milestone 822 implements bounded `ltrim()` for scalar/null
  string-convertible values with PHP's default left-trim mask and non-empty
  literal character masks without range syntax. This covers the reached
  `wpdb::check_safe_collation()` query trim at
  `wp-includes/class-wpdb.php:3495`, including the `"\r\n\t ("` mask. This is
  not full PHP charlist range semantics, empty-mask behavior, binary/null-byte
  edge cases beyond current runtime strings, array/object/resource coercions,
  exact diagnostics, or native lowering. The real bootstrap-shim probe now
  advances to
  `runtime error at <bootstrap-shim>:3496:8: unsupported call preg_match(): only the u pattern modifier is implemented in the current subset`.
  Milestone 823 implements a bounded `preg_match()` slice for that exact
  safe-collation read-query classifier,
  `/^(?:SHOW|DESCRIBE|DESC|EXPLAIN|CREATE)\s/i`, with ASCII-case-insensitive
  keyword matching and one following ASCII whitespace character. This is not
  broad PCRE alternation/grouping, general `i` modifier support, broad `\s`
  semantics, capture-group fidelity, exact diagnostics, or native lowering.
  The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:3474:16: unsupported call preg_match(): regex metacharacter [ is not implemented in the current subset`,
  corresponding to the adjacent ASCII-check pattern
  `/[^\x00-\x7F]/` in `wp-includes/class-wpdb.php:3474`.
  Milestone 824 implements that exact `wpdb::check_ascii()` non-ASCII detector
  over the current valid UTF-8 runtime string model. This is not broad
  bracket-class support, arbitrary ranges, byte-level capture fidelity for
  multi-byte characters, binary/invalid UTF-8 behavior, exact PCRE diagnostics,
  or native lowering. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:203:2: undefined function array_unshift()`.
  Milestone 825 implements bounded direct-variable `array_unshift()` for the
  reached bootstrap path: prepended values are evaluated left to right, integer
  keys are reindexed, string keys are preserved, the caller variable is
  mutated, and the new count is returned. This is not broad by-reference
  argument handling, non-variable array targets, value-only dynamic
  invocation, references/copy-on-write, exact warning behavior, or native
  lowering. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:328:48: undefined function current()`.
  Milestone 826 implements bounded `current()` over current ordered arrays:
  it returns the first inserted value and returns `false` for empty arrays.
  This is not PHP's mutable internal array-pointer model,
  `next()`/`reset()` interaction, object support, references/copy-on-write,
  exact warning behavior, or native lowering. The real bootstrap-shim probe
  now advances to
  `runtime error at <bootstrap-shim>:341:15: undefined function call_user_func_array()`.
  Milestone 827 implements bounded `call_user_func_array()` for string
  callbacks, public `[object, method]` instance callbacks, public
  `[class, method]` static callbacks, and integer-keyed ordered arrays used as
  positional argument lists. This is not broad callable support, closure or
  `__invoke` dispatch, non-public method callbacks, by-reference argument
  propagation, string-keyed named arguments, exact warning behavior, or native
  lowering. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:346:23: undefined function next()`.
  Milestone 828 implements bounded `next()` over the current ordered-array
  cursor model for direct variable arrays and the reached direct
  object-property array-offset shape, covering
  `next( $this->iterations[ $nesting_level ] )` in `WP_Hook::apply_filters()`.
  This is not full PHP internal array-pointer semantics, broad lvalue targets,
  value-only dynamic calls, object operands, `reset()`/`end()`/`prev()`
  interaction, references/copy-on-write, exact warning behavior, or native
  lowering. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:207:2: undefined function array_pop()`.
  Milestone 829 implements bounded direct-variable `array_pop()` for the
  reached hook cleanup path such as `array_pop( $wp_current_filter )`: it
  removes and returns the last inserted value, returns `null` for empty arrays,
  and preserves the reached append-index behavior after integer-key pops. This
  is not broad by-reference array handling, object-property array targets,
  value-only dynamic calls, full internal pointer side effects,
  references/copy-on-write, exact warning behavior, or native lowering. The
  real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:2357:20: unsupported call mysqli_query(): only the WordPress SQL mode probe SELECT @@SESSION.sql_mode is implemented in the current subset`,
  corresponding to `wp-includes/class-wpdb.php:2357` in `wpdb::_do_query()`.
  Milestone 830 extends the bounded MySQLi placeholder for the reached
  `wpdb::_do_query()` options bootstrap path: the current placeholder
  `mysqli_query()` accepts the WordPress autoloaded-options and all-options
  `SELECT option_name, option_value FROM <prefix>options...` reads as
  deterministic empty/no-result boundaries, `mysqli_errno()` returns `0`,
  `mysqli_error()` returns an empty string, and `mysqli_result` exists as core
  metadata for reached `instanceof` checks. This is not real SQL execution,
  real result resources, row fetching, affected rows, insert ids,
  charset/collation handling, prepared statements, transactions,
  errors/warnings, or native database integration. The real bootstrap-shim
  probe now advances to
  `runtime error at <bootstrap-shim>:2312:8: unsupported call preg_match(): only the u pattern modifier is implemented in the current subset`,
  corresponding to the following `wpdb::query()` query-classification regex.
  Milestone 831 implements the adjacent bounded `wpdb::query()` classifier
  regexes for DDL, DML, and insert/replace queries, and widens the empty
  options-table MySQLi placeholder for the reached option-name cache-priming
  and single-option reads. This is not broad PCRE support, broad SQL parsing,
  real SQL execution, result resources, row fetching, database contents,
  affected-row/insert-id state, exact warnings/errors, or native lowering. The
  real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:3045:17: unsupported call empty(): only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported`,
  corresponding to a reached complex `empty(...)` operand.
  Milestone 832 extends `empty(...)` for direct nested array-offset and direct
  object-property array-offset paths, covering the reached
  `$this->last_result[$y]` shape in `wpdb::load_col_info()`. It also widens
  the deterministic empty MySQLi placeholder for reached WordPress metadata
  probes such as `SHOW FULL COLUMNS FROM ...` and `DESCRIBE ...`. This is not
  arbitrary lvalue support, magic property/ArrayAccess behavior,
  references/copy-on-write, broad SQL parsing, result resources, row fetching,
  database contents, exact warnings/errors, or native lowering. The real
  bootstrap-shim probe now advances to
  `parse error at <bootstrap-shim>:283:14: unsupported object static property access: object receiver static properties are not implemented`.
  Milestone 833 implements bounded dynamic static-receiver property reads and
  direct writes for `$object::$property` and `$className::$property`, covering
  the reached `$phpmailer::$validator = static function (...) { ... };` shape
  in `wp-includes/pluggable.php:283`. This is not object receiver class
  constants, `$object::class`, compound assignment, increment/decrement,
  `isset`/`empty`/`??`/`??=`, unset, references/copy-on-write, magic hooks,
  autoload, exact diagnostics, or native lowering. The real bootstrap-shim
  probe now advances to
  `runtime error at <bootstrap-shim>:6316:35: undefined array key "SCRIPT_FILENAME"`.
  Milestone 834 seeds `$_SERVER['SCRIPT_FILENAME']` as a deterministic CLI
  placeholder for the reached `wp_guess_url()` path. This is not a full
  SAPI/request environment, host web-server state, CGI/FPM path translation,
  document-root mapping, exact warnings, or native lowering. The real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:78:47: undefined function str_ends_with()`.
  Milestone 835 implements bounded `str_ends_with()` for the reached
  `wp_fix_server_vars()` guard over `$_SERVER['SCRIPT_FILENAME']`. This is not
  broad string coercion, object/resource operands, binary/invalid UTF-8 edge
  behavior, exact diagnostics, or native lowering. The real bootstrap-shim
  probe now advances to
  `runtime error at <bootstrap-shim>:6335:21: undefined function substr()`.
  Milestone 836 implements bounded `substr()` for the reached bootstrap
  string-slicing path. This is not full PHP binary string slicing, broad
  offset/length coercion, object/resource operands, exact diagnostics, or
  native lowering. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:6337:13: unsupported call preg_replace(): only the WordPress database-version cleanup pattern /[^0-9.].*/ is implemented in the current subset`.
  Milestone 837 widens bounded `preg_replace()` for the reached
  `#/[^/]*$#i` empty-replacement path-tail cleanup used by `wp_guess_url()`.
  This is not broad PCRE replacement, arrays, captures/backrefs, callbacks,
  limit/count output, exact diagnostics, or native lowering. The real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:6344:23: undefined array key "HTTP_HOST"`.
  Milestone 838 seeds `$_SERVER['HTTP_HOST']` as deterministic `localhost` for
  the reached `wp_guess_url()` path. This is not full SAPI/request state,
  trusted Host-header validation, proxy/web-server routing, HTTPS/port
  handling, exact warnings, or native lowering. The real bootstrap-shim probe
  now advances to
  `runtime error at <bootstrap-shim>:6347:9: undefined function rtrim()`.
  Milestone 839 implements bounded `rtrim()` for the reached `wp_guess_url()`
  path normalization. This is not full PHP charlist range behavior,
  binary/null-byte edge cases, object/resource operands, exact diagnostics, or
  native lowering. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:958:2: undefined function wp_redirect()`.
  Milestone 840 implements runtime registration for conditional/nested
  function declarations, covering the reached guarded `wp_redirect()`
  declaration in `wp-includes/pluggable.php` without pretending it is a
  builtin. This is not full PHP declaration timing for every edge case,
  unbraced nested declarations, closure invocation, reference-return aliasing,
  autoload-aware callable discovery, or native lowering. The real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:1565:15: undefined function preg_replace_callback()`.
  Milestone 841 implements the bounded `preg_replace_callback()` path for the
  reached `wp_sanitize_redirect()` UTF-8 sanitizer regex and exact
  `_wp_sanitize_utf8_in_redirect` string callback. This is not broad PCRE
  callback replacement, pattern/subject arrays, callback arrays/closures/method
  callables, limit/count/flags handling, exact diagnostics, or native lowering.
  The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:1566:15: unsupported call preg_replace(): only the WordPress database-version cleanup pattern /[^0-9.].*/ and path-tail pattern #/[^/]*$#i are implemented in the current subset`.
  Milestone 842 widens bounded `preg_replace()` for the reached
  `|[^a-z0-9-~+_.?#=&;,/:%!*\[\]()@]|i` empty-replacement redirect sanitizer
  cleanup pattern. This is not broad PCRE replacement, arrays, callbacks,
  captures/backrefs, limit/count output, exact diagnostics, or native lowering.
  The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:2018:13: unsupported call preg_replace(): only the WordPress database-version cleanup pattern /[^0-9.].*/, path-tail pattern #/[^/]*$#i, and redirect sanitizer cleanup pattern |[^a-z0-9-~+_.?#=&;,/:%!*\[\]()@]|i are implemented in the current subset`.
  Milestone 843 widens bounded `preg_replace()` for the reached
  `pluggable.php` `#^www\.#` mail-host cleanup and the adjacent KSES
  null-cleanup patterns `/[\x00-\x08\x0B\x0C\x0E-\x1F]/` and `/\\\\+0+/`.
  This is not broad PCRE replacement, arrays, callbacks, captures/backrefs,
  limit/count output, full PHP string escape behavior, exact diagnostics, or
  native lowering. The real bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:4440:14: unsupported call str_replace(): count output arguments are not implemented; pass exactly three arguments in the current subset`.
  Milestone 844 adds bounded direct-variable count-output support for the
  reached `str_replace($search, '', $subject, $count)` call in
  `wp-includes/formatting.php:4440`. This is not array search/replacement
  support, broad by-reference output semantics, indirect callable count output,
  exact warnings, binary string edge cases, or native lowering. The real
  bootstrap-shim probe now advances to
  `runtime error at <bootstrap-shim>:4440:14: unsupported call str_replace(): search argument arrays are not implemented in the current subset`.
  Milestone 845 adds bounded search-array support for the same `_deep_replace()`
  path when each search value is scalar/null string-convertible and the
  replacement and subject remain scalar/null string-convertible. This is not
  replacement arrays, subject arrays, nested search arrays, exact warnings,
  binary string edge cases, broad PHP references, or native lowering. The real
  bootstrap-shim probe now exits `0` with no stdout; with include tracing
  enabled, stderr contains include trace lines ending at
  `<wordpress-root>/wp-includes/pluggable.php`.
  Milestone 846 adds a separate front-controller probe for
  `wp-blog-header.php`. Against real WordPress 6.9.4 this reaches
  `wp-includes/class-wpdb.php:1511`, the `wpdb::prepare()` placeholder
  normalization `preg_replace()` pattern
  `/%(?:%|$|(?!($allowed_format)?[sdfFi]))/` with replacement `'%%\\1'`, and
  reports that broader PCRE replacement shape as the current front-controller
  runtime blocker.
  Milestone 847 adds a bounded implementation for that exact placeholder
  normalization shape. The real front-controller probe now advances to
  `runtime error at <wordpress-root>/wp-blog-header.php:1514:18: undefined function preg_split()`,
  the following `wpdb::prepare()` placeholder extraction call.
  Milestone 848 adds a bounded implementation for that exact
  `preg_split()` extraction shape, with `limit` `-1` and
  `PREG_SPLIT_DELIM_CAPTURE`. The real front-controller probe now advances to
  `runtime error at <wordpress-root>/wp-blog-header.php:1763:12: undefined function vsprintf()`,
  the `wpdb::prepare()` formatting call after placeholder parsing.
  Milestone 849 adds bounded `vsprintf()` support for that formatting path and
  expands the shared formatter for the reached `%s`/`%d`/`%F` WordPress
  prepare subset. The real front-controller probe now advances to
  `runtime error at <wordpress-root>/wp-blog-header.php:935:5: unsupported call mysqli_query(): only the WordPress SQL mode probe and empty wp_options SELECT placeholders are implemented in the current subset; got SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'`.
  Milestone 850 adds a deterministic `true` boundary for that exact
  `mysqli_query()` charset setup statement. The real front-controller probe
  now exits `0` with no stdout under the current placeholder database and CLI
  assumptions; with include tracing enabled, stderr contains include trace
  lines ending at `<wordpress-root>/wp-includes/pluggable.php`. This proves a
  bounded smoke path, not plugin/theme/admin/REST, real database, HTTP,
  filesystem, SAPI, rendered request, or native WordPress support.
  Milestone 851 commits a synthetic front-controller smoke for that shape.
  Milestones 852 and 853 add the first placeholder empty `mysqli_result`
  lifecycle and a synthetic empty `wpdb::query()` consumption smoke.
  Milestone 854 sharpens the boundary for remaining `SELECT` statements.
  Milestone 855 adds one deterministic row-backed placeholder result for
  `SELECT ID, post_title FROM wp_posts WHERE ID = 1`, and Milestone 856 adds a
  synthetic `wpdb::get_results()` smoke that stores that row in
  `last_result`, increments `num_rows`, and returns the row array. Milestone
  857 adds associative array hydration for the same deterministic row through
  `mysqli_fetch_assoc()`. Milestone 858 adds a synthetic
  `wpdb::get_results($query, ARRAY_A)` smoke for that associative row path.
  Milestone 859 adds `mysqli_fetch_array($result, MYSQLI_ASSOC)` for the same
  deterministic row while keeping numeric and mixed modes explicit
  boundaries. Milestone 860 adds a synthetic `wpdb::get_results($query,
  ARRAY_A)` smoke that consumes the row through that explicit
  `mysqli_fetch_array(..., MYSQLI_ASSOC)` path. Milestone 861 adds numeric,
  explicit mixed, and omitted-mode default mixed fetch-array hydration for the
  same deterministic row. Milestone 862 adds a synthetic `wpdb::get_results()`
  smoke that consumes the omitted-mode default mixed row shape. Milestone 863
  adds deterministic `mysqli_fetch_row()` numeric row hydration for the same
  placeholder result, and Milestone 864 adds a synthetic
  `wpdb::get_results($query, ARRAY_N)` smoke over that numeric row branch.
  Milestone 865 adds bounded `mysqli_data_seek()` cursor reset for placeholder
  results, and Milestone 866 adds a synthetic `wpdb` smoke that rewinds and
  rereads the placeholder row. Milestone 867 adds bounded
  `mysqli_num_rows()` for the placeholder result row count without moving the
  fetch cursor, and Milestone 868 adds a synthetic `wpdb` smoke that records
  that placeholder row count through `$this->num_rows`. Milestone 869 adds
  bounded clean-state `mysqli_affected_rows()` and `mysqli_insert_id()` zero
  metadata for placeholder handles, and Milestone 870 adds a synthetic `wpdb`
  smoke that records that zero metadata through query bookkeeping. Milestone
  871 adds bounded `mysqli_set_charset("utf8mb4")` placeholder success for the
  reached charset setup shape, and Milestone 872 adds a synthetic `wpdb`
  smoke that records that placeholder charset setup through local
  charset/collation bookkeeping. These are still deterministic harness
  milestones, not SQL execution, real database state, duplicate-column or
  warning/error fidelity, real `wpdb` output-mode fidelity, real
  affected-row/insert-id state, real charset/collation behavior,
  plugin/theme/admin/REST, SAPI, rendered request, or native WordPress support.
  Milestone 873 then sharpens mutation SQL into an explicit unsupported
  `mysqli_query()` boundary for leading `INSERT`/`UPDATE`/`DELETE`/`REPLACE`
  statements instead of pretending those queries mutate placeholder state, and
  Milestone 874 adds a synthetic `wpdb::query()` bookkeeping smoke that reaches
  that boundary through a WordPress-shaped `UPDATE wp_options ...` path. This
  does not add real update/insert/delete execution, affected-row or insert-id
  mutation, transactions, database state, partial-output fidelity, or native
  lowering. Milestone 875 adds deterministic placeholder `mysqli_ping()`
  success for current `mysqli` objects as a future connection-check boundary;
  it does not perform a real connection liveness check, reconnect, socket I/O,
  or host database integration. Milestone 876 adds a synthetic
  `wpdb::check_connection()` smoke that records deterministic ready state after
  that placeholder ping path without claiming real reconnection or database
  liveness behavior. Milestone 877 adds deterministic placeholder
  `mysqli_get_host_info()` metadata for current `mysqli` objects without
  inspecting a real host, transport, socket, protocol, or live connection.
  Milestone 878 adds a synthetic WordPress-shaped `wpdb` host-info bookkeeping
  smoke that records that deterministic metadata without claiming real
  connection metadata fidelity. Milestone 879 adds deterministic zeroed
  `mysqli_stat()` placeholder server-status metadata without real counters,
  thread/table state, live connection inspection, or host database integration.
  Milestone 880 adds a synthetic WordPress-shaped `wpdb` server-status
  bookkeeping smoke that records that deterministic status string without
  claiming real server status fidelity. Milestone 881 adds bounded
  deterministic `mysqli_autocommit($handle, bool $mode)` placeholder success
  without real autocommit state, transactions, commit/rollback behavior, or
  host database integration. Milestone 882 wires that placeholder through a
  synthetic WordPress-shaped `wpdb` autocommit bookkeeping smoke without
  claiming real WordPress transaction fidelity. Milestone 883 adds bounded
  deterministic `mysqli_begin_transaction()` placeholder success without real
  transaction state, autocommit state changes, commit/rollback behavior, or
  host database integration. Milestone 884 wires that placeholder through a
  synthetic WordPress-shaped `wpdb` transaction-start bookkeeping smoke without
  claiming real WordPress transaction fidelity. Milestone 885 adds bounded
  deterministic `mysqli_commit()`/`mysqli_rollback()` placeholder success
  without real transaction state, savepoints, database mutation, warning/error
  fidelity, or host database integration. Milestone 886 wires those
  placeholders through a synthetic WordPress-shaped `wpdb` commit/rollback
  bookkeeping smoke without claiming real WordPress transaction fidelity.
  Milestone 887 adds deterministic clean `mysqli_sqlstate()` and
  `mysqli_warning_count()` placeholder metadata without real SQLSTATE,
  warning-count, or host error-state tracking. Milestone 888 wires that clean
  placeholder metadata through a synthetic WordPress-shaped `wpdb` error-state
  bookkeeping smoke without claiming real database warning/error fidelity.
  Milestone 889 adds deterministic `mysqli_get_client_info()` and
  `mysqli_get_proto_info()` placeholder client/protocol metadata without real
  client-library detection, protocol negotiation, host connection metadata, or
  PHP deprecation/warning fidelity. Milestone 890 wires that placeholder
  metadata through a synthetic WordPress-shaped `wpdb` connection-metadata
  bookkeeping smoke without claiming real database client/protocol fidelity.
  Milestone 891 adds deterministic `mysqli_get_client_version()` placeholder
  metadata without real client-library version detection, host database
  integration, extension configuration fidelity, or native database lowering.
  Milestone 892 wires that placeholder through a synthetic WordPress-shaped
  `wpdb` client-version bookkeeping smoke without claiming real database
  client-version fidelity. Milestone 893 adds deterministic
  `mysqli_get_server_version()` placeholder metadata without real
  server-version detection, host database integration, protocol negotiation,
  server capability inspection, warning/error fidelity, or native database
  lowering. Milestone 894 wires that placeholder through a synthetic
  WordPress-shaped `wpdb` server-version bookkeeping smoke without claiming
  real database server-version fidelity. Milestone 895 adds deterministic
  `mysqli_get_connection_stats()` placeholder metadata without real mysqlnd
  statistics, client/server traffic accounting, query accounting, memory
  accounting, connection reuse state, warning/error fidelity, host database
  integration, or native database lowering. Milestone 896 wires that
  placeholder through a synthetic WordPress-shaped `wpdb`
  connection-statistics bookkeeping smoke without claiming real WordPress
  database connection-statistics fidelity. Milestone 897 adds deterministic
  `mysqli_thread_id()` placeholder metadata without real server-thread
  inspection, connection identity, reconnect behavior, `mysqli_kill()`
  integration, host database integration, warning/error fidelity, or native
  database lowering. Milestone 898 wires that placeholder through a synthetic
  WordPress-shaped `wpdb` thread-id bookkeeping smoke without claiming real
  WordPress database connection identity or server-thread fidelity. Milestone
  899 adds deterministic `mysqli_get_charset()` placeholder charset/collation
  metadata without real charset negotiation, client-library/server metadata
  inspection, collation state, charset mutation tracking, escaping behavior
  changes, warning/error fidelity, or native database lowering. Milestone 900
  wires that placeholder through a synthetic WordPress-shaped `wpdb`
  charset/collation bookkeeping smoke without claiming real WordPress
  charset/collation negotiation or escaping fidelity. Milestone 901 adds
  deterministic `mysqli_character_set_name()` placeholder charset-name
  metadata without real charset negotiation, connection charset state tracking,
  escaping behavior changes, warning/error fidelity, or native database
  lowering. Milestone 902 wires that placeholder through a synthetic
  WordPress-shaped `wpdb` charset-name bookkeeping smoke without claiming real
  WordPress charset negotiation or escaping fidelity. Milestone 903 adds
  deterministic `mysqli_field_count()` clean field-count metadata without
  most-recent-query tracking, result metadata tracking, SQL execution state,
  warning/error fidelity, host database integration, or native database
  lowering. Milestone 904 wires that placeholder through a synthetic
  WordPress-shaped `wpdb` field-count bookkeeping smoke without claiming real
  last-query field-count fidelity. Milestone 905 adds deterministic
  `mysqli_close()` placeholder lifecycle metadata without real host connection
  teardown, handle invalidation, server resource release, close-after-use
  diagnostics, warning/error fidelity, or native database lowering. Milestone
  906 wires that placeholder through a synthetic WordPress-shaped `wpdb` close
  bookkeeping smoke without claiming real WordPress disconnect behavior.
  Milestone 907 adds deterministic
  `mysqli_options(..., MYSQLI_OPT_INT_AND_FLOAT_NATIVE, ...)` placeholder
  client-option metadata without real option negotiation, result
  type-conversion behavior, connection state mutation, host database
  integration, warning/error fidelity, or native database lowering. Milestone
  908 wires that placeholder through a synthetic WordPress-shaped `wpdb`
  options bookkeeping smoke without claiming real client-option behavior.
  Milestone 909 adds deterministic
  `mysqli_connect_errno()`/`mysqli_connect_error()` clean connect-error
  metadata without failed connection tracking, host extension error state,
  report-mode behavior, PHP warning/error/exception fidelity, or native
  database lowering. Milestone 910 wires that placeholder through a synthetic
  WordPress-shaped `wpdb` connection error-state bookkeeping smoke without
  claiming real connection failure fidelity. Milestone 911 adds deterministic
  `mysqli_info()` clean statement-information metadata without real SQL
  statement-info tracking, mutation summaries, host database state,
  warning/error fidelity, or native database lowering. Milestone 912 wires
  that placeholder through a synthetic WordPress-shaped `wpdb` query-info
  bookkeeping smoke without claiming real SQL statement information. Milestone
  913 adds deterministic `mysqli_get_warnings()` clean warning-chain metadata
  without real warning objects, warning iteration, SQL warning metadata, host
  database state, warning/error fidelity, or native database lowering.
  Milestone 914 wires that placeholder through a synthetic WordPress-shaped
  `wpdb` query-warning bookkeeping smoke without claiming real SQL warning
  metadata. Milestone 915 adds deterministic
  `mysqli_store_result()`/`mysqli_use_result()` clean no-pending-result
  metadata without real buffered or unbuffered result transfer, pending result
  tracking, host database state, warning/error fidelity, or native database
  lowering. Milestone 916 wires that placeholder through a synthetic
  WordPress-shaped `wpdb` connection result-drain bookkeeping smoke without
  claiming real result buffering or unbuffered result lifecycle fidelity.
  Milestone 917 adds deterministic `mysqli_kill()` placeholder thread-id kill
  metadata without real server-thread killing, connection invalidation,
  reconnect behavior, host database state, warning/error fidelity, or native
  database lowering. Milestone 918 wires that placeholder through a synthetic
  WordPress-shaped `wpdb` connection thread-lifecycle bookkeeping smoke
  without claiming real thread killing, reconnect, or connection invalidation
  fidelity. Milestone 919 adds deterministic `mysqli_change_user()`
  placeholder user/database-change metadata without real authentication,
  selected-database state, server session reset, transaction rollback,
  temporary-table cleanup, locked-table cleanup, host database state,
  warning/error fidelity, or native database lowering. Milestone 920 wires
  that placeholder through a synthetic WordPress-shaped `wpdb` connection
  user/database-change bookkeeping smoke without claiming real authentication,
  selected-database, or session-reset fidelity. Milestone 921 adds
  deterministic `mysqli_refresh()` placeholder refresh metadata over exposed
  deprecated `MYSQLI_REFRESH_*` flags without real table/log/cache flush
  behavior, replication reset, server status reset, connection/session
  mutation, PHP deprecation/warning fidelity, host database state, or native
  database lowering. Milestone 922 wires that placeholder through a synthetic
  WordPress-shaped `wpdb` connection refresh bookkeeping smoke without
  claiming real flush, replication reset, status reset, or session mutation
  fidelity. Milestone 923 adds deterministic `mysqli_real_query()` placeholder
  charset setup execution metadata without real query execution, pending
  result tracking, result object creation, mutation state, host database state,
  warning/error fidelity, or native database lowering. Milestone 924 wires
  that placeholder through a synthetic WordPress-shaped `wpdb` connection
  query bookkeeping smoke without claiming real SQL execution, pending result
  tracking, result object creation, mutation state, connection charset
  mutation, host database state, warning/error fidelity, or native database
  lowering. Milestone 925 adds deterministic `mysqli_multi_query()`
  placeholder charset setup execution metadata without real multi-statement
  execution, pending result queues,
  `mysqli_more_results()`/`mysqli_next_result()` state, result object
  creation, mutation state, host database state, warning/error fidelity, or
  native database lowering. Milestone 926 wires that placeholder through a
  synthetic WordPress-shaped `wpdb` connection query bookkeeping smoke without
  claiming real multi-statement execution, pending result queues, result object
  creation, mutation state, connection charset mutation, host database state,
  warning/error fidelity, or native database lowering. Milestone 927 adds
  deterministic `mysqli_reap_async_query()` clean no-async-result placeholder
  metadata without `MYSQLI_ASYNC`, `mysqli_poll()`, async socket readiness,
  pending async result queues, result object creation, host database state,
  warning/error fidelity, or native database lowering. Milestone 928 wires
  that placeholder through a synthetic WordPress-shaped `wpdb` connection
  async-result bookkeeping smoke without claiming real async query execution,
  `MYSQLI_ASYNC`, `mysqli_poll()`, socket readiness, pending async result
  queues, host database state, warning/error fidelity, or native database
  lowering. Milestone 929 exposes `MYSQLI_ASYNC` and `mysqli_poll()` metadata
  while keeping reached `mysqli_poll()` calls as a stable async-readiness
  boundary without real polling, by-reference read/error/reject array mutation,
  pending async result queues, host socket state, host database state,
  warning/error fidelity, or native database lowering. Milestone 930 wires
  that metadata and boundary through a synthetic WordPress-shaped `wpdb`
  connection method without claiming real polling, by-reference result arrays,
  socket readiness, host query execution, warning/error fidelity, or native
  database lowering. Milestone 931 adds deterministic
  `mysqli_get_links_stats()` host-link metadata without real persistent-link
  tracking, host client-library state, sockets, connection reuse state,
  warning/error fidelity, or native database lowering. Milestone 932 wires
  that metadata through a synthetic WordPress-shaped `wpdb` host-link
  bookkeeping smoke without claiming real persistent-link tracking, sockets,
  host client-library state, connection reuse state, warning/error fidelity, or
  native database lowering. Milestone 933 adds deterministic
  `mysqli_dump_debug_info()` placeholder host-diagnostics metadata without
  MySQL DBUG trace output, host client-library debug state, socket inspection,
  host database state, warning/error fidelity, or native database lowering.
  Milestone 934 wires that metadata through a synthetic WordPress-shaped
  `wpdb` connection diagnostics smoke without claiming MySQL DBUG trace
  output, host client-library debug state, socket inspection, host database
  state, warning/error fidelity, or native database lowering. Milestone 935
  adds deterministic `mysqli_debug()` DBUG-configuration metadata without
  MySQL DBUG option parsing, trace-file creation, host client-library debug
  state mutation, socket inspection, host database state, warning/error
  fidelity, or native database lowering. Milestone 936 wires that metadata
  through a synthetic WordPress-shaped `wpdb` connection diagnostics smoke
  without claiming MySQL DBUG option parsing, trace-file creation, host
  client-library debug state mutation, socket inspection, host database state,
  warning/error fidelity, or native database lowering. Milestone 937 adds a
  small deterministic zeroed `mysqli_get_client_stats()` mysqlnd client
  statistics subset without PHP's full mysqlnd table, real client-library
  traffic accounting, memory accounting, connection reuse state, sockets, host
  database state, warning/error fidelity, or native database lowering.
  Milestone 938 wires that metadata through a synthetic WordPress-shaped
  `wpdb` diagnostics smoke without claiming PHP's full mysqlnd table, real
  client-library accounting, memory accounting, connection reuse state,
  sockets, host database state, warning/error fidelity, or native database
  lowering. Milestone 939 adds deterministic empty `mysqli_error_list()` clean
  error-list metadata without real warning/error list tracking, SQLSTATE
  history, host client-library state, socket state, host database state,
  warning/error fidelity, or native database lowering. Milestone 940 wires
  that metadata through a synthetic WordPress-shaped `wpdb` diagnostics smoke
  without claiming real warning/error list tracking, SQLSTATE history, host
  client-library state, socket state, host database state, warning/error
  fidelity, or native database lowering. Milestone 941 adds bounded
  deterministic `mysqli_thread_safe()` client-library thread-safety metadata
  without host client-library build-flag inspection, real thread-safety
  configuration, host client-library state, socket state, host database state,
  warning/error fidelity, or native database lowering.
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

The first committed WordPress-shaped smoke target is:

```text
Run `cargo test -p phpc --test wordpress_inventory_cli -- --test-threads=1`
and verify the synthetic `wp-blog-header.php` front-controller probe exits 0
with no stdout.
```

External-source WordPress runs remain operator-supplied measurements unless a
separate source-size, license, update, and checksum policy is accepted. The
normalized output policy exists, and the latest external-source run is recorded
in `docs/PROGRESS.md` rather than vendoring WordPress core.
