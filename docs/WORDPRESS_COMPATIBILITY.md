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
