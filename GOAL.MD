# Project Goal

## Slash Goal Objective

Build this repository into a real, honest PHP-to-native compiler in stable Rust
with a long-term target of full PHP and WordPress compatibility. The target
includes the difficult parts: references, copy-on-write, dynamic lookup,
multi-file execution, object semantics, standard-library and extension behavior,
database and filesystem integration, web/SAPI-like runtime state, and native
runtime integration.

The path to that target remains evidence-driven: advance one small verified PHP
feature, compatibility harness, runtime semantic, or native-lowering slice at a
time; keep `phpc run` and `phpc compile` boundaries explicit; keep tests green;
and never claim support without executable code, CLI coverage, documentation,
and named unsupported edge cases.

## Product Target

The project should become a useful PHP compiler/runtime toolchain, not a demo.
Its first durable target is a documented PHP subset that can:

- parse PHP source with a real lexer/parser, not regex parsing;
- execute supported programs through `phpc run` with PHP-shaped boxed runtime
  values;
- compile a smaller but steadily growing supported subset through
  `phpc compile --emit-ir` and `phpc compile --emit-asm`;
- reject unsupported PHP constructs with stable parse, runtime, or codegen
  diagnostics instead of emitting misleading output;
- compare supported interpreter behavior against system PHP where practical;
- preserve accurate support documentation for every supported and unsupported
  feature.

Long term, the compiler should support enough PHP semantics to run real PHP
applications, including WordPress core, representative plugins, themes, and
CLI/admin/front-end request flows. Dynamic PHP features may use interpreter or
runtime fallback zones, but those fallback zones must be explicit and tested,
and they must remain stepping stones toward compatibility rather than excuses to
avoid difficult semantics.

## What We Have Been Doing

The work so far has been building the substrate for compatibility without lying
about it. That means:

- a real lexer/parser instead of regex parsing;
- a PHP-shaped runtime value model for scalars, arrays, constants, and a minimal
  object/class slice;
- an interpreter path that can execute a growing tested subset through
  `phpc run`;
- a native path that deliberately compiles less than the interpreter and rejects
  unsupported lowering before emitting misleading IR or assembly;
- fixture and system-PHP comparison infrastructure so new behavior can be
  checked against PHP rather than guessed;
- stable unsupported diagnostics so missing features are visible work items,
  not silent miscompilations.

This is not yet "a PHP compiler that runs WordPress." It is the foundation that
lets the project move toward that goal while preserving correctness.

Recent WordPress-directed milestones have been walking the real WordPress
6.9.4 bootstrap-shim forward through concrete blockers: references as explicit
runtime boundaries, class modifiers, magic `new self`/`parent`/`static`
instantiation, direct-object-property nested array assignment, nested
object-property array-offset `unset(...)`, expression-form `include`/`require`
return values, skipped positional `list(...)` destructuring, class
`implements` relationship metadata, bounded object-handle `=&`, bounded
`str_replace()`, bounded `call_user_func()`, a bounded direct-variable
array `=&` path for `wp_parse_args()`, bounded `strcasecmp()`, bounded
no-argument `headers_sent()`, bounded `abs()`, bounded `header_remove()`, and
a bounded `mysqli_connect` function-table boundary, top-level `global`
null-materialization, bounded `microtime(true)`, and bounded deterministic
`ini_get()` registry reads, bounded `strtolower()`, bounded `trim()`, a
bounded leading-numeric `(int)` string-cast slice, bounded `str_contains()`,
bounded integer `min()`, and bounded nested direct-variable array-offset
`isset(...)`, bounded local-path `is_readable()`, bounded shutdown
registration, bounded `date_default_timezone_set('UTC')`, bounded
deterministic CLI `$_SERVER`/`PHP_SAPI` startup state, bounded
`preg_match()`, bounded `error_reporting()` mask state, bounded `is_dir()`,
bounded braced dynamic object-property names, bounded `__METHOD__`
magic-constant evaluation, bounded `set_error_handler()` registration, and
bounded explicit closure capture binding for inert closure values, and bounded
`preg_match()` `u`-modifier handling for the reached UTF-8 startup probe, and
bounded `restore_error_handler()` cleanup, and keyword-named object-property
reads/writes after `->`, and bounded array literal reference elements that
currently evaluate the referenced value without aliasing, and bounded
object-property reference-assignment sources that copy current array/object
values without aliasing, and bounded direct/dynamic object-property
`unset(...)` operands that currently null visible property slots instead of
modeling true property uninitialization. The current shim frontier
has moved past WordPress' missing-MySQL-extension guard in
`wp-includes/load.php:202` and the early top-level `$wp_filter` global
initialization in `wp-includes/plugin.php:39`; it has moved past the missing
`is_readable()` filesystem builtin in the fatal-error-handler override check
and the fatal-error-handler `register_shutdown_function()` registration, past
timezone setup, past the first request/SAPI startup state, and past the first
regex startup guard, past the first error-reporting mask setup, and past the
first startup directory check, the first braced dynamic object-property access,
the first `__METHOD__` magic constant in method context, the first closure
capture in the `set_error_handler()` path, the reached `preg_match('//u', '')`
UTF-8 probe, the following `restore_error_handler()` cleanup, and the reached
array literal reference element, and the reached object-property
reference-assignment source, the reached `unset($this->$name)` path,
executable `__CLASS__` in class context, and the reached `mysqli_report()`
mode call, the reached `mysqli_init()` handle creation, bounded `strpos()`
and `substr_count()` matching, the reached `preg_match()` matches-output path,
the reached `mysqli_real_connect()` placeholder-success path, and the reached
`preg_replace()` database-version cleanup path, and the reached
`mysqli_query()` SQL-mode empty-result boundary, and the reached
`mysqli_select_db()` placeholder-success path, and the bootstrap shim now
provides the conventional `$table_prefix = 'wp_';` startup variable, and
bounded `preg_match()` now covers the reached table-prefix validation guard;
bounded `wpdb` dynamic table-name properties now cover the reached
`wpdb::$categories` assignment path; direct object-property array-offset
compound assignment now covers the reached object-cache mutation path; direct
`$GLOBALS['name']` reads/writes now cover the reached object-cache bootstrap
global assignment; direct object-property array-offset `isset(...)` now covers
the reached `WP_Hook::add_filter()` priority check; bounded `ksort()` now
covers the reached hook-priority ordering path; direct-variable by-reference
output parameters cover the reached `WP_Object_Cache::get(..., $found)`
cache-miss path; bounded `mysqli_real_escape_string()` now covers the reached
`wpdb::_real_escape()` option lookup path; bounded deterministic `rand()` now
covers the reached `wpdb::placeholder_escape()` salt path; bounded
`uniqid()` plus HMAC-SHA256 `hash_hmac()` now cover the reached placeholder
hash path.
Subsequent bounded milestones advanced the same shim through nested
`empty(...)` paths and deterministic MySQLi metadata placeholders, dynamic
object/class-string static property access for the PHPMailer validator
assignment, deterministic CLI `$_SERVER['SCRIPT_FILENAME']` startup state, and
the reached `str_ends_with($_SERVER['SCRIPT_FILENAME'], 'php.cgi')` guard,
bounded `substr()` string slicing for the following bootstrap path, and bounded
`preg_replace()` path-tail cleanup, plus deterministic `HTTP_HOST` startup
state, plus bounded `rtrim()` path normalization, plus conditional/nested
function declaration registration for the reached guarded `wp_redirect()`
declaration, plus bounded `preg_replace_callback()` for the reached
`wp_sanitize_redirect()` UTF-8 sanitizer callback, plus bounded
`preg_replace()` redirect sanitizer cleanup, mail-host cleanup, and KSES null
cleanup patterns, plus bounded `str_replace()` direct-variable count output
and search-array replacement for the reached `_deep_replace()` path. As of
Milestone 845, direct `wp-settings.php` still stops at missing `ABSPATH`, while
the real WordPress 6.9.4 bootstrap shim exits `0` with no stdout under the
current deterministic CLI/bootstrap assumptions.
Milestones 846 through 853 then broadened that work from the bootstrap shim
into a front-controller and database-consumption smoke path without pretending
that placeholder behavior is real WordPress support. Milestone 846 added a
separate `wp-blog-header.php` front-controller probe. Milestones 847 and 848
covered the reached `wpdb::prepare()` placeholder-normalization
`preg_replace()` and placeholder-extraction `preg_split()` shapes. Milestone
849 added bounded `vsprintf()` support for the reached `wpdb::prepare()`
formatting path. Milestone 850 added a deterministic `mysqli_query()` success
boundary for the exact charset setup statement
`SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'`, after which the real
WordPress 6.9.4 front-controller probe exits `0` with no stdout under the
current placeholder database and CLI assumptions. Milestone 851 committed a
synthetic normalized front-controller smoke for that path. Milestone 852 added
the first placeholder empty `mysqli_result` lifecycle boundary, and Milestone
853 added a synthetic `wpdb::query()` smoke that stores, fetches, frees, and
drains that empty result through WordPress-shaped object state. Milestones 854
through 862 then moved the database-consumption smoke from an explicit
non-empty-`SELECT` boundary into one deterministic seed-post row exposed through
object, associative, numeric, fetch-row, and mixed fetch-array paths, plus
synthetic `wpdb::get_results()` harnesses for those row shapes, including a
numeric `ARRAY_N`-style smoke, bounded cursor reset/re-read paths, and bounded
placeholder row-count reads wired through a WordPress-shaped `$num_rows` path,
plus clean-state affected-row and insert-id metadata boundaries wired through
WordPress-shaped query bookkeeping, a bounded `mysqli_set_charset()`
placeholder charset boundary wired through a WordPress-shaped setup method,
an explicit mutation-SQL unsupported boundary before any placeholder state
mutation is claimed, and a synthetic `wpdb::query()` bookkeeping smoke that
reaches that mutation boundary through a WordPress-shaped `UPDATE wp_options`
path, plus a deterministic `mysqli_ping()` placeholder liveness boundary for
future WordPress-shaped connection checks, now wired through a synthetic
`wpdb::check_connection()` smoke, plus deterministic
`mysqli_get_host_info()` placeholder connection metadata wired through a
synthetic WordPress-shaped `wpdb` host-info bookkeeping smoke, plus
deterministic zeroed `mysqli_stat()` placeholder server-status metadata wired
through a synthetic WordPress-shaped `wpdb` server-status bookkeeping smoke,
plus a bounded deterministic `mysqli_autocommit()` placeholder transaction
boundary for boolean modes wired through a synthetic WordPress-shaped `wpdb`
autocommit bookkeeping smoke, plus a bounded deterministic
`mysqli_begin_transaction()` placeholder transaction-start boundary wired
through a synthetic WordPress-shaped `wpdb` transaction bookkeeping smoke,
plus bounded deterministic `mysqli_commit()`/`mysqli_rollback()` placeholder
transaction-completion boundaries wired through a synthetic WordPress-shaped
`wpdb` commit/rollback bookkeeping smoke, plus deterministic clean
`mysqli_sqlstate()`/`mysqli_warning_count()` placeholder error-state metadata
wired through a synthetic WordPress-shaped `wpdb` error-state bookkeeping
smoke, plus deterministic `mysqli_get_client_info()`/`mysqli_get_proto_info()`
placeholder client/protocol metadata for the next database connection metadata
boundary, now wired through a synthetic WordPress-shaped `wpdb`
connection-metadata bookkeeping smoke, plus deterministic
`mysqli_get_client_version()` placeholder client-version metadata wired through
a synthetic WordPress-shaped `wpdb` client-version bookkeeping smoke, plus
deterministic `mysqli_get_server_version()` placeholder server-version
metadata for the next server metadata boundary, now wired through a synthetic
WordPress-shaped `wpdb` server-version bookkeeping smoke, plus deterministic
`mysqli_get_connection_stats()` placeholder connection-statistics metadata,
now wired through a synthetic WordPress-shaped `wpdb` connection-statistics
bookkeeping smoke, plus deterministic `mysqli_thread_id()` placeholder
thread-id metadata, now wired through a synthetic WordPress-shaped `wpdb`
thread-id bookkeeping smoke, plus deterministic `mysqli_get_charset()`
placeholder charset/collation metadata, now wired through a synthetic
WordPress-shaped `wpdb` charset/collation bookkeeping smoke, plus deterministic
`mysqli_character_set_name()` placeholder charset-name metadata, now wired
through a synthetic WordPress-shaped `wpdb` charset-name bookkeeping smoke,
plus deterministic `mysqli_field_count()` clean field-count metadata, now
wired through a synthetic WordPress-shaped `wpdb` field-count bookkeeping
smoke, plus deterministic `mysqli_close()` placeholder lifecycle metadata,
now wired through a synthetic WordPress-shaped `wpdb` close bookkeeping smoke,
plus deterministic `mysqli_options()` client-option metadata for the reached
`MYSQLI_OPT_INT_AND_FLOAT_NATIVE` placeholder boundary, now wired through a
synthetic WordPress-shaped `wpdb` options bookkeeping smoke, plus
deterministic `mysqli_connect_errno()`/`mysqli_connect_error()` clean
connect-error metadata, now wired through a synthetic WordPress-shaped `wpdb`
connect-error bookkeeping smoke.
These are
evidence that the compiler can walk farther through WordPress-shaped execution;
they are not evidence of real database, request, plugin/theme/admin/REST, SAPI,
or native WordPress compatibility.
Each step is intentionally small because it only counts after parser/runtime
code, tests, CLI fixtures, docs, and named unsupported edges are in place.

## Compatibility North Star

The compatibility goal has three tiers.

The goal is complete only when the repository can prove these outcomes with
committed commands and fixtures:

- PHP language conformance suites are pinned, runnable, and tracked by PHP
  version, with expected deviations documented.
- `phpc run` can execute multi-file PHP applications with namespaces,
  autoloading, dynamic lookup, references, copy-on-write, objects, exceptions,
  standard library behavior, and extension-backed services sufficient for real
  applications.
- `phpc compile` can either produce native executable behavior for a documented
  subset or reject unsupported constructs before lowering with stable
  diagnostics; no native path may claim WordPress compatibility by delegating to
  untracked interpreter behavior.
- A committed WordPress target can bootstrap, load plugins/themes, exercise
  representative front-end/admin/REST/WP-CLI flows, and compare observable
  behavior against stock PHP under documented host assumptions.
- The project can explain every remaining fallback zone as an explicit
  compatibility strategy with tests, not as an undocumented escape hatch.

### Tier 1: PHP Language Compatibility

Support the language behavior needed by current PHP branches, while keeping
version-specific behavior explicit. As of 2026-05-14, php.net lists PHP
8.2-8.5 as supported branches, with 8.2 in security support and 8.3-8.5 in the
active/security lifecycle. The project should track this moving target in docs
and tests instead of freezing an obsolete PHP version.

Required work includes:

- full expression, statement, declaration, namespace, include/require, and
  autoload semantics;
- references, aliasing, copy-on-write, foreach-by-reference, and mutation
  ordering;
- PHP object execution beyond the current public instance-method slice:
  constructors, non-public/static/dynamic methods, visibility,
  inheritance, interfaces, traits, enums, anonymous classes, static members,
  class constants, magic methods, and object identity;
- exceptions, errors, warnings, notices, stack traces, output buffering, and
  partial-output behavior;
- attributes, typed properties, type declarations, union/intersection types,
  nullable types, `strict_types`, generators, closures, arrow functions,
  first-class callables, named arguments, variadics, splat/unpack behavior, and
  modern control constructs such as `match`;
- source mapping and version-specific diagnostics that can be compared against
  system PHP.

### Tier 2: PHP Runtime, Standard Library, And Extensions

Support the runtime services real applications rely on, not only syntax.

Required work includes:

- core standard library behavior across strings, arrays, filesystem, streams,
  dates, regex, filters, sessions, serialization, hashes, password APIs, and
  reflection;
- extension slices used by real applications, with focused support docs for
  `json`, `mbstring`, `intl`, XML/DOM/SimpleXML, `curl`, `openssl`, `zip`,
  `zlib`, GD/ImageMagick boundaries, PDO/MySQLi, SQLite, opcache-like
  assumptions, and cache integrations where applicable;
- superglobals, request/server environment, cookies, uploaded files, headers,
  response status, ini settings, locale/timezone behavior, and stream wrappers;
- exact enough error, warning, coercion, and deprecation behavior to pass
  compatibility fixtures.

### Tier 3: WordPress Compatibility

Treat WordPress as an explicit application target, not an implied side effect.
As of 2026-05-14, WordPress.org recommends PHP 8.3 or greater, MariaDB 10.6+
or MySQL 8.0+, and HTTPS for a modern host baseline; it also documents legacy
runtime compatibility down to PHP 7.2.24+ for current WordPress. The compiler
goal is not met until this repo can run committed WordPress compatibility
targets with documented environment assumptions.

Required work includes:

- a committed WordPress compatibility harness using a pinned WordPress core
  version, starting with WordPress 6.9 as the current compatibility table row
  to track, then tracking later stable core deliberately;
- bootstrap/load tests for `wp-settings.php`, plugin/theme loading, hooks,
  filters, shortcode/block registration paths, REST routing, admin/front-end
  request setup, and WP-CLI-shaped execution where feasible;
- filesystem, database, HTTP, JSON, XML, mbstring/intl, password/hash,
  serialization, sessions/cookies, object cache, and options/transients
  behavior required by WordPress core;
- compatibility fixtures for representative plugins and themes after core
  bootstrap is credible;
- documented fallback policy for dynamic extension or host integrations that
  cannot yet be native-compiled.

External target references checked on 2026-05-14:

- PHP supported branches: https://www.php.net/supported-versions.php
- WordPress host requirements: https://wordpress.org/about/requirements/
- WordPress PHP compatibility table:
  https://make.wordpress.org/core/handbook/references/php-compatibility-and-wordpress-versions/

## Missing Work Reconciled

The major missing areas are not side quests. They are the compatibility roadmap.

For full PHP and WordPress compatibility, the highest-risk missing systems are:

1. Multi-file program structure: include/require, namespaces, aliases,
   autoloading, source mapping, and path resolution.
2. PHP value semantics: references, aliasing, copy-on-write, by-reference
   parameters/returns, and mutation ordering.
3. Object semantics: executable static dispatch, late static binding, typed and
   default properties, magic methods, interfaces, traits, enums, cloning,
   destructors, reflection, and exact visibility behavior.
4. Error and control behavior: PHP-shaped exceptions, warnings/notices,
   recoverable errors, output buffering, stack traces, and partial stdout.
5. Runtime services: standard-library and extension behavior for filesystem,
   streams, JSON, XML, HTTP, database, encoding, crypto, date/time, sessions,
   cookies, superglobals, and ini/environment state.
6. Native execution: runtime ABI, linked binaries, boxed values, stack frames,
   heap/lifetime management, helper calls, and native diagnostics.

Reconciliation into concrete workstreams:

- **PHP conformance harness:** add a committed PHP-version manifest, pin a
  small imported or mirrored php-src-style behavior subset where licensing and
  scope allow, and record expected deviations per PHP branch. Current artifact:
  fixture runner plus optional system PHP comparison. Missing artifact:
  versioned conformance manifest and suite.
- **Reference/COW value model:** replace the bounded copy/copy-back and
  no-alias reference slices with reference cells, alias-aware symbol slots,
  array/object copy-on-write, by-reference foreach semantics, and mutation
  ordering tests. Current artifact: documented runtime boundaries and focused
  reference fixtures. Missing artifact: runtime reference container and COW
  tests that compare against PHP.
- **Object/runtime semantics:** turn metadata-heavy class support into full
  execution semantics for magic methods, typed properties, trait composition,
  interface enforcement, enum cases, destructors, reflection, and exact
  visibility. Current artifact: tested minimal object/class model. Missing
  artifact: executable object compatibility matrix.
- **Error model:** implement PHP-shaped warnings/notices/deprecations,
  exception objects and unwinding, `@` suppression, stack traces, output
  buffers, and partial-output behavior. Current artifact: stable project
  diagnostics and bounded `throw`/`try` boundaries. Missing artifact:
  recoverable warning/error execution tests.
- **Runtime services/extensions:** move from placeholder extension boundaries
  to focused implementations for filesystem/streams, JSON, XML, mbstring/intl,
  HTTP/curl, crypto/password, date/time, serialization, sessions/cookies,
  headers, ini/environment state, MySQLi/PDO/SQLite, and cache APIs. Current
  artifact: selected builtins plus `docs/extensions/`. Missing artifact:
  extension-specific compatibility docs with real host-backed or deterministic
  substitute tests.
- **WordPress harness:** replace placeholder-only progress with pinned,
  committed WordPress core targets that cover bootstrap, front-end, admin,
  REST, WP-CLI-like execution, plugin/theme loading, database-backed queries,
  filesystem/media, object cache, options/transients, hooks, filters, and
  block registration. Current artifact: external WordPress 6.9.4 inventory and
  synthetic front-controller/`wpdb` smokes. Missing artifact: committed
  WordPress core compatibility harness with documented host assumptions.
- **Native runtime integration:** add linked native executable support with
  boxed values, runtime helper calls, stack frames, heap/lifetime management,
  diagnostics, and explicit fallback/rejection policy for dynamic PHP. Current
  artifact: LLVM IR/assembly for a narrow scalar subset plus scalar ABI seed.
  Missing artifact: linked native execution compared against `phpc run` and
  system PHP for a non-trivial program.

For WordPress specifically, Milestone 724 advanced the bootstrap-shim probe
past `clone` expressions by adding fresh object handles and shallow property
slot copying for objects without declared `__clone` methods. Milestone 725
then advanced past loop-depth `break 2;` by letting active loops and switches
consume positive integer literal `break N;`/`continue N;` depths. Milestone
726 then advanced past function-scope `global $name, ...;` imports by routing
direct imported globals through the root symbol table. Milestone 727 then
added the deterministic `PHP_VERSION` compatibility constant. Milestone 728
then advanced past the first `version_compare()` guard with a bounded
numeric-component comparator. Milestone 729 then advanced past the first
`sprintf()` message-format path with a bounded string-placeholder formatter.
Milestone 730 then advanced past the first `header()` path with a no-op
web/SAPI boundary that validates the current argument shape and returns `null`.
Milestone 731 then advanced past the first missing-extension `implode()` path
with a bounded scalar/null array join builtin. Milestone 732 then advanced past
the first `exit(1)` termination path with a bounded direct `exit()`/`die()`
construct. The bootstrap-shim now reaches WordPress' missing-extension guard
and exits with code `1`, stdout, and no stderr. Milestone 733 then replaced
the empty `extension_loaded()` registry with a bounded compatibility registry
for the current `json` and `hash` WordPress requirement checks. Milestone 734
then advanced past the first filesystem drop-in probe with bounded
`file_exists()` execution for one string local path. Milestone 735 then
advanced past the first complex string interpolation blocker with bounded
direct array-offset and object-property interpolation. Milestone 736 then
advanced past the first heredoc/nowdoc blocker with bounded unindented
identifier-label heredoc/nowdoc syntax. Milestone 737 then advanced past the
first nested interpolation blocker with bounded chained property/array-offset
interpolation. Milestone 738 then advanced past the first `static function`
closure parse blocker by treating static anonymous closures as inert closure
values. Milestone 739 then made the WordPress inventory probes timeout-bounded:
with `WORDPRESS_PROBE_TIMEOUT=10s`, the direct probe still reports the
`ABSPATH` runtime error and the bootstrap-shim probe reports `timed_out: yes`,
exit `124`, zero stdout, and no stderr. The next step is a runtime execution
budget or trace mode to identify the last source span/function/include frame
before the shim times out. Milestone 740 added `PHPC_MAX_EXECUTION_STEPS`,
which catches statement-execution loops, but the bootstrap shim still times out
at `30s` even with `PHPC_MAX_EXECUTION_STEPS=100`; the current long path is
therefore before ordinary statement execution budget exhaustion and needs
parser/include/declaration-registration tracing. Milestone 741 added
`PHPC_TRACE_INCLUDES=1` and inventory `last_stderr_line` reporting; the current
bootstrap-shim timeout frontier is
`<wordpress-root>/wp-includes/sodium_compat/src/Compat.php`, and that file also
timed out when run directly under a 10s outer timeout with no stderr.
Milestone 742 added `PHPC_TRACE_PARSE=1`, fixed maintained lexer byte offsets
so large files no longer repeatedly rescan consumed source for byte-prefix
checks, and added bounded final-position variadic user-function parameters.
The real bootstrap-shim probe now advances past the Sodium compatibility class
and variadic parameter blocker. Milestone 743 added comma-separated `for`
header expression lists, and the next bootstrap-shim blocker is
`parse error at <bootstrap-shim>:3909:1: expected expression, found <`.
Milestone 744 added bounded inline HTML output between `?>` and the next PHP
open tag. Milestone 745 added bounded dynamic property-name reads/writes for
existing public slots and `stdClass` public dynamic slots. Milestone 746 added
statement-form by-reference assignment syntax as a runtime boundary. Milestone
747 added by-reference `foreach` value syntax as a runtime boundary. Milestone
748 added by-reference assignment from direct array-offset sources as the same
runtime boundary. Milestone 749 added append-at-depth assignment for
direct-variable nested array paths. Milestone 750 added nested direct-variable
array-offset `unset(...)`. Milestone 751 added untyped instance property
defaults for the current constant-expression subset and advanced past
`public $_nplurals = 2;` in `wp-includes/pomo/mo.php`; the current
bootstrap-shim blocker is now reference assignment from a method-call source at
`<bootstrap-shim>:301:46`, corresponding to
`$entry = &$this->make_entry( $original, $translation );` in
`wp-includes/pomo/mo.php`. Milestone 752 then parsed that method-call
reference source as the existing runtime boundary; the current bootstrap-shim
blocker is now by-reference assignment into an object-property array-offset
target at `<bootstrap-shim>:302:38`, corresponding to
`$this->entries[ $entry->key() ] = &$entry;` in `wp-includes/pomo/mo.php`.
Milestone 753 then parsed that direct target shape as the existing runtime
boundary. Milestone 754 then parsed by-reference function and method return
declarations as runtime boundaries. Milestone 755 then parsed class and method
modifiers for the current metadata subset; the current bootstrap-shim blocker
then became broader magic class-name resolution at `<bootstrap-shim>:63:26`.
Milestone 756 resolved `new self`, `new parent`, and `new static` in active
class/method contexts. Milestone 757 added direct-object-property nested array
assignment and append-at-depth assignment. Milestone 758 added nested
object-property array-offset `unset(...)`. Milestone 759 then added
expression-form `include`, `include_once`, `require`, and `require_once` return
values for the current local-file subset. Milestone 760 added skipped
positional slots in statement-form `list(...) = expr;`. Milestone 761 then
recorded class `implements` metadata for relationship checks while leaving
interface enforcement and built-in/internal interface catalogs explicit future
work. Milestone 762 added a bounded object-handle `=&` slice for direct
variable sources into direct array-offset targets, advancing past
`wp-includes/l10n.php:1428`. Milestone 763 added a bounded scalar/null
`str_replace()` builtin. Milestone 764 added bounded string-callable
`call_user_func()` dispatch. Milestone 765 added a bounded direct-variable
array `=&` path for `$parsed_args =& $args;` in
`wp-includes/functions.php:4955`. Milestone 766 added bounded `strcasecmp()`.
Milestone 767 added bounded no-argument `headers_sent()`. Milestone 768 added
bounded `abs()`. Milestone 769 added bounded `header_remove()`. Milestone 770
added a `mysqli_connect` function-table boundary. Milestone 771 added
top-level `global` null-materialization. Milestone 772 added bounded
`microtime(true)`. Milestone 773 added bounded deterministic `ini_get()`
registry reads. Milestone 774 added bounded `strtolower()`; the current
bootstrap-shim frontier is now an undefined `trim()` call in
`wp_convert_hr_to_bytes()`. Milestone 775 added bounded default-mask `trim()`;
Milestone 776 added bounded leading-numeric `(int)` string casts. Milestone
777 added bounded `str_contains()`. Milestone 778 added bounded integer
`min()`. Milestone 779 added bounded nested direct-variable array-offset
`isset(...)`. Milestone 780 added bounded local-path `is_readable()`.
Milestone 781 added bounded `register_shutdown_function()` registration, and
Milestone 782 added bounded `date_default_timezone_set('UTC')`. Milestone 783
added bounded deterministic CLI `$_SERVER` and `PHP_SAPI` startup state.
Milestone 784 added bounded `preg_match()` for the reached SAPI-name pattern.
Milestone 785 added bounded `error_reporting()` mask state and reached `E_*`
constants. Milestone 786 added bounded `is_dir()` for local filesystem paths,
and Milestone 787 added braced dynamic object-property reads/writes through the
existing dynamic-property runtime subset. Milestone 788 added bounded
`__METHOD__` magic-constant evaluation in current function/method contexts.
Milestone 789 added bounded `set_error_handler()` registration. Milestone 790
added explicit `use (...)` closure capture binding for inert closure values.
Milestone 791 added bounded `preg_match()` `u`-modifier handling for the
reached UTF-8 startup probe. Milestone 792 added bounded
`restore_error_handler()` cleanup. Milestone 793 added keyword-named
object-property reads/writes after `->`. Milestone 794 added bounded array
literal reference elements without aliasing. Milestone 795 added bounded
object-property reference-assignment sources without aliasing. Milestone 796
added bounded direct/dynamic object-property `unset(...)` operands that null
visible slots. Milestone 797 added bounded executable `__CLASS__` evaluation in
class context. Milestone 798 added bounded `mysqli_report()` mode handling.
Milestone 799 added a bounded `mysqli_init()` placeholder handle. Milestone
800 added bounded `strpos()` matching for the reached `parse_db_host()` path.
Milestone 801 added bounded `substr_count()` matching. Milestone 802 added
bounded `preg_match()` direct matches-output support for the reached
`parse_db_host()` path. Milestone 803 added bounded deterministic
`mysqli_real_connect()` success for the current placeholder handle. Milestone
804 added bounded `preg_replace()` support for the reached database
version cleanup path. Milestone 805 added deterministic
`mysqli_get_server_info()` placeholder server metadata for the current fake
mysqli handle. Milestone 806 added bounded `compact()` caller-scope variable
packing for the reached charset/collation path. Milestone 807 added bounded
`mysqli_query()` false-result behavior for the reached SQL mode probe.
Milestone 808 added bounded deterministic `mysqli_select_db()` success for the
placeholder handle. Milestone 809 added the conventional `$table_prefix`
startup variable to the WordPress inventory bootstrap shim. The current
Milestone 810 added the exact WordPress table-prefix validation
`preg_match()` guard. Milestone 811 added bounded `wpdb` dynamic public
table-name property materialization. Milestone 812 added direct
object-property array-offset compound assignment for the reached object-cache
mutation path. Milestone 813 added bounded direct `$GLOBALS['name']`
read/write routing to the root global table for the reached object-cache
bootstrap assignment. Milestone 814 added direct object-property array-offset
`isset(...)` for the reached `WP_Hook::add_filter()` priority check. Milestone
815 added bounded `ksort($array, SORT_NUMERIC)` for direct variable arrays and
direct object-property arrays. The current frontier is now the reached
provided by-reference parameter invocation boundary in
`WP_Object_Cache::get(..., $found)`. Milestone 816 covers omitted optional
by-reference parameters with defaults, which moves past `wp_cache_get()`'s
omitted `$found` parameter in `get_option()` but not the later provided
`$found` output variable. Milestone 817 covers direct-variable by-reference
parameter copy-back for the reached `WP_Object_Cache::get(..., $found)` output
shape. Milestone 818 covers bounded `mysqli_real_escape_string()` for the
reached `wpdb::_real_escape()` option lookup path over the existing
placeholder `mysqli` object. The current frontier is now missing bounded
`rand()` support in the reached `wpdb::placeholder_escape()` path at
`wp-includes/class-wpdb.php:2422`. Milestone 819 covers bounded deterministic
no-argument `rand()` for that path. The current frontier is now missing
bounded `hash_hmac()` support in `wpdb::placeholder_escape()` at the adjacent
`wp-includes/class-wpdb.php:2424` call. Milestone 820 covers the reached
bounded `uniqid($salt, true)` and `hash_hmac('sha256', ..., $salt)` path.
Milestone 821 covers strict identity for current ordered arrays, including the
reached empty-array/list-shape comparisons. Milestone 822 covers bounded
`ltrim()` for the reached `wpdb::check_safe_collation()` query trim. The
Milestone 823 covers the exact case-insensitive `preg_match()` read-query
classifier in that same WordPress path. Milestone 824 covers the adjacent
`wpdb::check_ascii()` regex byte-range pattern. Milestone 825 covers the
reached direct-variable `array_unshift()` bootstrap call, including integer-key
reindexing, string-key preservation, caller-variable mutation, and return
count. Milestone 826 covers the reached ordered-array `current()` first-value
path. Milestone 827 covers the reached bounded `call_user_func_array()`
dispatch shape for string callbacks, public method array callables, and
integer-keyed positional argument arrays. Milestone 828 covers the reached
bounded `next()` cursor-advance path for direct arrays and
`$this->iterations[$level]`. Milestone 829 covers the reached direct-variable
`array_pop()` hook cleanup path. Milestone 830 covers the reached empty
WordPress options-table `mysqli_query()` placeholders in `wpdb::_do_query()`
plus clean `mysqli_errno()`/`mysqli_error()` bookkeeping for the placeholder
handle. Milestone 831 covers the adjacent `wpdb::query()` DDL/DML classifier
regexes plus the reached empty option-name cache-priming and single-option
MySQLi reads. Milestone 832 covers the reached direct nested array-offset and
direct object-property array-offset `empty(...)` paths plus deterministic empty
MySQLi metadata-query placeholders for `SHOW FULL COLUMNS FROM ...` and
`DESCRIBE ...`. Milestone 833 covers the reached dynamic static-receiver
property assignment shape `$phpmailer::$validator = static function (...)`
through bounded `$object::$property` and `$className::$property` reads/direct
writes. Milestone 834 seeds the deterministic CLI
`$_SERVER['SCRIPT_FILENAME']` startup key, and
Milestone 835 covers the reached `str_ends_with()` CGI guard, Milestone 836
covers bounded `substr()` slicing, and Milestone 837 covers the reached
`preg_replace('#/[^/]*$#i', '', ...)` path-tail cleanup. Milestone 838 seeds
the reached deterministic `$_SERVER['HTTP_HOST']` placeholder. Milestone 839
covers bounded `rtrim()` path normalization. The current bootstrap-shim
frontier is now
`runtime error at <bootstrap-shim>:958:2: undefined function wp_redirect()`.
The object-lifecycle lane still includes `__clone` dispatch, visibility around
clone methods, nested object/reference properties, references/copy-on-write,
destructors/reuse, and exact PHP diagnostics.

- Native execution is missing linked binaries, runtime ABI, boxed values in
  generated code, stack frames, heap management, runtime helper calls,
  process-level execution semantics, and native diagnostics.
- The interpreter is missing large parts of PHP syntax and semantics:
  namespaces, include/require breadth beyond the current local string-path
  statement/expression slice, autoload, references,
  copy-on-write, closures with real binding/invocation, generators,
  exceptions, type declarations,
  attributes, destructuring, spreads, resources, and exact warning/recovery
  behavior.
- Arrays are interpreter-supported for a growing subset but still lack full PHP
  copy-on-write/reference behavior, by-reference iteration, recursive/nested
  aliasing semantics, object/resource edge cases, and native lowering.
- Objects/classes are still minimal: public instance method calls, same-class
  private instance method calls, protected same-class/child instance method
  calls, single-parent metadata, inherited method lookup, inherited
  public/protected/private instance property slots, child-context protected
  property access, private-parent property redeclaration as a separate child
  slot, compatible public/protected inherited property redeclarations sharing
  one runtime slot, inherited property redeclaration diagnostics, and
  public/inherited public instance `__construct` execution with scoped
  `$this`, plus narrow `ClassName::class`, `self::class`, and `parent::class`
  resolution and narrow class constants through `ClassName::CONST`,
  `self::CONST`, `parent::CONST`, and late-bound `static::CONST`, plus narrow
  static properties through `ClassName::$prop`, `self::$prop`,
  `parent::$prop`, and late-bound `static::$prop` with direct reads/writes,
  read-modify-write, `isset`/`empty`, `??`, `??=` forms, and stable diagnostics
  for PHP-forbidden static-property `unset(...)`, plus visible static-method
  dispatch through `ClassName::method(...)`, `$object::method(...)`,
  `$className::method(...)`, `self::method(...)`, `parent::method(...)`, and
  late-bound `static::method(...)`, now execute through `phpc run`, but
  interfaces, traits, enums, object-receiver class constants,
  non-static dynamic static dispatch, broader class constant semantics,
  clone behavior beyond the current shallow no-`__clone` slice, magic methods,
  dynamic method/property names, typed property compatibility, property
  defaults beyond the current untyped constant-expression instance/static
  subset, broader constructor/inheritance rules, and native object lowering are
  still missing.
  The interpreter object-handle identity slice covers assignment,
  argument/return, array, foreach, strict identity, `spl_object_id`, and a
  current-subset `spl_object_hash`, and bounded clone expressions, but
  `__clone`, destruction/reuse, and full object execution are still missing.
- Standard-library and extension coverage is broad only in selected array and
  scalar areas; real app compatibility needs filesystem, streams, regex, JSON,
  XML, HTTP, database, crypto, encoding, date/time, reflection, sessions, and
  environment APIs.
- WordPress compatibility needs a pinned harness, bootstrap fixtures, database
  and filesystem integration strategy, hook/filter execution confidence,
  extension coverage, request/server state, and plugin/theme smoke targets.
- The support matrix must keep distinguishing four things: parsed syntax,
  `phpc run` interpreter support, native IR/assembly support, and future
  native executable support.

## What Is Still Missing Before Full WordPress Support

Full WordPress support is still missing at several layers:

1. Bootstrap correctness: direct `wp-settings.php` still needs a faithful
   entrypoint/environment policy for `ABSPATH`, include path, configuration,
   source mapping, and autoload behavior.
2. PHP language semantics: references, copy-on-write, by-reference
   calls/returns, closures with capture and invocation, generators, `match`,
   named arguments, spread/unpack, full
   namespace/import forms, and version-specific parse/runtime behavior.
3. Object model: full constructors/inheritance, interfaces, traits, enums,
   magic methods, dynamic properties/methods, typed properties and property
   defaults beyond the current constant-expression subset,
   reflection, cloning/destructors, late static binding, and exact visibility
   behavior.
4. Error model: warnings/notices/deprecations, recoverable errors, `@`
   suppression, exceptions and stack unwinding, output buffering, partial
   output, stack traces, and PHP-shaped fatal behavior.
5. Runtime services: filesystem and streams, JSON, XML/DOM/SimpleXML, regex,
   mbstring/intl, date/time, password/hash/crypto, serialization, sessions,
   cookies, headers, superglobals, ini/environment state, HTTP, database
   access, object cache, options/transients, and extension constants/functions.
6. WordPress flows: core bootstrap, installer/config loading, database-backed
   requests, hooks/filters, plugin/theme discovery and loading, front-end,
   admin, REST, cron, media/filesystem, blocks, and WP-CLI-like entrypoints.
7. Native path: generated code for boxed values, arrays, objects, calls, stack
   frames, heap/lifetime management, runtime helper ABI, diagnostics, linked
   executable behavior, and stable rejection for every unsupported lowering
   case.

## Current Baseline

The repository already has a working Rust workspace with:

- `php_runtime`: boxed PHP-shaped values for scalars, ordered arrays, class
  metadata, and minimal object values;
- `phpc`: lexer, parser, AST, interpreter, CLI, fixture runner, and narrow LLVM
  IR/assembly emission;
- `phpc run`: broad Milestone 1+ interpreter support for scalars, arrays,
  loops, alternate conditional syntax, functions, syntax-only closure
  declarations, constants, selected builtins including a deterministic
  bounded `json`/`hash` `extension_loaded()` policy, bounded `PHP_VERSION_ID` and
  `PHP_VERSION`, bounded `dirname()`, bounded no-op `spl_autoload_register()`,
  bounded `file_exists()` for one local string path,
  syntax-only PHP
  attributes, bounded throw-statement runtime boundaries, minimal objects,
  bounded `instanceof`, a class-name-only namespace/import slice, and stable
  unsupported diagnostics;
- `phpc compile --emit-ir` and `--emit-asm`: intentionally narrower native
  lowering, mostly straight-line static scalar output and assignment behavior,
  plus explicit rejection coverage for unsupported constructs;
- fixture coverage under `tests/fixtures`, optional system PHP comparison, and
  operational automation under `tools/` and `docs/OPERATIONS.md`.
- a repeatable external WordPress 6.9.4 inventory flow whose bootstrap-shim
  probe currently advances through bounded loop-depth control flow,
  function-scope `global` imports, the `PHP_VERSION` compatibility constant,
  the first bounded `version_compare()` slice, and the first bounded
  `sprintf()`, `header()`, `implode()`, `exit()`, `extension_loaded()`, and
  `file_exists()` slices, plus bounded direct array-offset/object-property
  string interpolation, bounded heredoc/nowdoc syntax, bounded chained
  interpolation, bounded static-closure parsing, timeout-bounded WordPress
  inventory probes, include/parser tracing, maintained lexer byte offsets for
  large source files, bounded final-position variadic parameters, and bounded
  dynamic property-name reads/writes for existing public slots and `stdClass`,
  statement-form by-reference assignment syntax as a runtime boundary,
  by-reference `foreach` value syntax as a runtime boundary, and
  by-reference assignment from direct array-offset sources as a runtime
  boundary, bounded object-handle and direct-variable array `=&` value-model
  slices, append-at-depth assignment for direct-variable nested array paths,
  nested direct-variable array-offset `unset(...)`, and untyped instance
  property defaults for the current constant-expression subset, plus
  by-reference assignment from method-call sources and into direct
  object-property array-offset targets as the same runtime boundary, and
  by-reference function/method return declarations as runtime boundaries, plus
  `abstract`/`final`/`readonly` class modifiers and `abstract`/`final` method
  modifiers as metadata, and `new self`/`new parent`/`new static` magic
  class-name instantiation in active class/method contexts, and bounded
  `strcasecmp()`, bounded no-argument `headers_sent()`, bounded `abs()`,
  bounded `header_remove()`, a `mysqli_connect` boundary, top-level `global`
  null-materialization, bounded `microtime(true)`, and bounded deterministic
  `ini_get()` registry reads, bounded `strtolower()`, and bounded default-mask
  `trim()`, bounded leading-numeric `(int)` string casts, bounded
  `str_contains()`, bounded integer `min()`, bounded nested direct-variable
  array-offset `isset(...)`, bounded local-path `is_readable()`, bounded
  shutdown registration, bounded timezone setting, bounded deterministic
  CLI `$_SERVER`/`PHP_SAPI` startup state, bounded `preg_match()`, and bounded
  `error_reporting()` mask state, bounded `is_dir()`, and bounded braced
  dynamic object-property names, and bounded `__METHOD__` magic-constant
  evaluation, bounded `set_error_handler()` registration, bounded explicit
  closure capture binding, bounded `preg_match()` `u`-modifier handling, and
  bounded `restore_error_handler()` cleanup, and keyword-named object-property
  parsing after `->`, and bounded array literal reference elements without
  aliasing, and bounded object-property reference-assignment sources without
  aliasing, and bounded direct/dynamic object-property `unset(...)` operands
  that null visible property slots, and bounded executable `__CLASS__`
  evaluation in class context, and bounded `mysqli_report()` mode handling for
  the reached startup call, and bounded `mysqli_init()` placeholder-handle
  creation, bounded `strpos()` matching, bounded `substr_count()` counts, and
  bounded `preg_match()` direct matches-output support for the reached
  db-host parse path, and bounded `mysqli_real_connect()` deterministic
  placeholder success, bounded `preg_replace()` database-version cleanup,
  bounded `mysqli_get_server_info()` placeholder metadata, bounded `compact()`
  caller-scope packing, bounded `mysqli_query()` false-result behavior, and
  bounded `mysqli_select_db()` deterministic success. The inventory bootstrap
  shim now supplies `$table_prefix = 'wp_';` as startup state, and bounded
  `preg_match()` now covers the reached table-prefix validation guard. Bounded
  `wpdb` dynamic public table-name properties cover the reached
  `wpdb::$categories` initialization path. Direct object-property array-offset
  compound assignment covers the reached object-cache mutation path. Bounded
  direct `$GLOBALS['name']` read/write routing covers the reached object-cache
  bootstrap global assignment, and direct object-property array-offset
  `isset(...)` covers the reached `WP_Hook::add_filter()` priority check.
  Bounded `ksort($array, SORT_NUMERIC)` covers the reached hook-priority
  ordering path. Omitted optional by-reference parameters with defaults can now
  execute without reference binding. Direct-variable by-reference parameters
  copy the callee's final value back to the caller, covering the reached
  `WP_Object_Cache::get(..., $found)` output shape. Bounded
  `mysqli_real_escape_string()` covers the reached `wpdb::_real_escape()`
  option lookup path over the placeholder MySQLi handle. Bounded deterministic
  `rand()` covers the reached `wpdb::placeholder_escape()` salt path. Bounded
  `uniqid()` plus HMAC-SHA256 `hash_hmac()` covers the reached placeholder
  hash path. Strict identity for current arrays covers the reached
  empty-array/list-shape comparisons. Bounded `ltrim()` and two exact
  WordPress `preg_match()` slices cover the reached safe-collation classifier
  and ASCII check. Bounded direct-variable `array_unshift()` covers the reached
  bootstrap array prepend path. Bounded `current()` covers the reached
  ordered-array first-value shape. Bounded `call_user_func_array()` covers the
  reached hook-dispatch callable shape. Bounded `next()` covers the reached
  array-pointer iteration shape. Bounded `array_pop()` covers the reached hook
  cleanup shape. Bounded `mysqli_query()` options-table placeholders plus
  clean `mysqli_errno()` and `mysqli_error()` bookkeeping cover the reached
  `wpdb::_do_query()` empty options query path without claiming real database
  execution. Bounded WordPress `preg_match()` query classifiers and empty
  option-name MySQLi reads cover the following `wpdb::query()` path. Bounded
  direct nested array-offset and object-property array-offset `empty(...)`
  paths cover the reached `wpdb::load_col_info()` shape, and deterministic
  empty MySQLi metadata-query placeholders cover reached `SHOW FULL COLUMNS`
  and `DESCRIBE` probes. Bounded dynamic static-receiver property reads/direct
  writes cover the reached PHPMailer validator assignment. A bounded
  `SCRIPT_FILENAME` CLI seed covers the reached URL-guessing read, and a
  bounded `str_ends_with()` slice covers the following CGI guard, bounded
  `substr()` covers the next string-slicing path, and bounded `preg_replace()`
  covers the reached path-tail cleanup, and deterministic `HTTP_HOST` startup
  state covers the next host read, and bounded `rtrim()` covers the following
  path normalization, and conditional/nested function declaration registration
  covers the guarded `wp_redirect()` declaration in `pluggable.php`, and
  bounded `preg_replace_callback()` covers the reached
  `wp_sanitize_redirect()` UTF-8 sanitizer callback, and bounded
  `preg_replace()` covers the reached redirect sanitizer cleanup pattern,
  mail-host cleanup, and KSES null cleanup patterns, and bounded
  `str_replace()` direct-variable count output plus search-array handling cover
  the reached `_deep_replace()` loop in `wp-includes/formatting.php:4440`.
  The bootstrap shim now exits `0`. Milestones 846-853 additionally add a
  front-controller probe, bounded `wpdb::prepare()` regex/formatting support,
  the reached MySQLi charset setup boundary, a committed silent
  front-controller smoke, a placeholder empty `mysqli_result` lifecycle, and a
  synthetic `wpdb::query()` empty-result smoke. The real WordPress 6.9.4
  front-controller probe exits `0` with no stdout under deterministic
  placeholder database and CLI assumptions. The direct `wp-settings.php` probe
  still stops at undefined `ABSPATH`, which is an entrypoint/environment issue.
  Real front-end/admin/REST/plugin/theme flows, real database state,
  HTTP/filesystem/SAPI behavior, and native WordPress support remain missing.

Read these before changing behavior: `AGENTS.md`, `docs/PROGRESS.md`,
`docs/ARCHITECTURE.md`, `docs/SUPPORT.md`, `docs/COMPATIBILITY.md`,
`docs/NATIVE_RUNTIME_ABI.md`, `docs/WORDPRESS_COMPATIBILITY.md`, `README.md`,
and `docs/LOOP_MEMORY.md` when running unattended loop automation.

## Definition of Done

A feature or milestone is complete only when all of these are true:

- Implementation code exists in the compiler, runtime, or CLI.
- Unit, integration, or fixture tests cover the behavior.
- A CLI exercise path exists through `phpc run`, `phpc compile --emit-ir`,
  `phpc compile --emit-asm`, or an explicit CLI snapshot.
- Documentation in `README.md`, `docs/SUPPORT.md`, and any focused design doc
  is accurate.
- `docs/PROGRESS.md` records the change after meaningful behavior changes.
- Unsupported edge cases are named directly.
- Native lowering either implements the behavior honestly or rejects it with a
  specific codegen diagnostic.
- The relevant focused tests pass during the slice; checkpoint or integration
  batches run `tools/run-tests.sh` unless a blocker is documented.

## Engineering Constraints

- Use stable Rust for compiler and runtime code.
- Prefer small, correct PHP subsets over broad placeholder claims.
- Keep dynamic PHP features as explicit runtime fallback zones.
- Runtime values must remain PHP-shaped: boxed values first, ordered int/string
  keyed arrays, references and copy-on-write when implemented.
- Do not replace the parser with regular-expression parsing.
- Add crates only when they materially reduce work.
- Do not silently widen support in docs before tests prove it.
- Do not collapse interpreter support and native lowering into one claim:
  `phpc run` may support a construct while `phpc compile` still rejects it.
- Keep the `cc -S` assembly fallback documented as a bootstrap fallback, not the
  primary backend.

## Execution Lanes

The goal is intentionally broad, so day-to-day work should be split into
independent lanes. Each lane owns one milestone at a time and should use a
separate git worktree when running in parallel with other workers.

Lanes:

- Parser lane: lexer, parser, AST shape, parse diagnostics, and syntax-focused
  fixtures. This lane must not widen runtime or native support claims unless
  another lane implements and tests the behavior.
- IR/lowering lane: `phpc compile --emit-ir`, `phpc compile --emit-asm`,
  native codegen diagnostics, deterministic backend/fallback snapshots, and
  native support documentation.
- Runtime lane: `php_runtime`, interpreter semantics for `phpc run`, boxed
  value behavior, builtins, object/array semantics, and system PHP comparison
  fixtures.
- Compiler output lane: CLI behavior, emitted artifact shape, backend
  selection/failure behavior, future link/run output, stdout/stderr/exit
  contracts, and operational scripts.
- Tests/docs lane: fixture organization, focused/full test gates, support
  matrix accuracy, progress records, operations docs, and unsupported-edge
  naming.

Parallel work rules:

- One worktree per lane and one active milestone per worker. Avoid multiple
  workers editing the same source files, fixture directories, or docs section.
- Each worker records its lane and milestone in `docs/PROGRESS.md` before
  handing off or checkpointing.
- A worker may run focused tests for its lane during the slice, but a checkpoint
  batch must run the serialized full gate or document a blocker.
- Cross-lane changes require an explicit handoff note naming the affected lane,
  files, tests, and remaining risks.
- Shared files such as `docs/SUPPORT.md`, `docs/ARCHITECTURE.md`,
  `README.md`, `docs/NEXT_TASKS.md`, and `compiler/src/codegen.rs` require
  extra care: keep edits scoped, avoid broad rewrites, and do not overwrite
  another lane's unmerged work.

## Milestone Plan

### 0. Keep The Baseline Honest

Maintain the existing interpreter, native lowering, docs, and test suite as the
floor for all work.

Exit criteria:

- `tools/run-tests.sh` passes.
- `docs/NEXT_TASKS.md` has a clear next unchecked task.
- `docs/SUPPORT.md` distinguishes `phpc run` support from native compile
  support.
- New diagnostics are stable enough for fixture snapshots.

### 1. Finish Native Scalar Lowering

Grow `phpc compile --emit-ir` and `--emit-asm` from static scalar output toward
a coherent straight-line scalar subset.

Scope:

- static scalar reassignment and overwrite coverage;
- native scalar arithmetic for supported int/float/literal subsets;
- native comparison, logical, bitwise, unary, and conditional expression slices
  only when PHP coercion and error boundaries are explicit;
- compile-time diagnostics for unsafe static cases and explicit rejection for
  dynamic cases until runtime-backed checks exist.

Exit criteria:

- Lowered scalar programs have CLI assembly coverage and fixture coverage.
- Unsupported scalar cases fail before backend invocation with specific codegen
  diagnostics.
- Generated IR/assembly tests use deterministic assertions rather than
  platform-specific assembly text.

### 2. Link Runtime-Backed Native Execution

Move native compilation beyond standalone emitted assembly text by linking
generated code with runtime helpers.

Scope:

- define a stable ABI between generated code and `php_runtime`;
- lower boxed value creation, echo/print conversion, and selected runtime helper
  calls;
- add an executable native path only when process exit, stdout, stderr, and
  diagnostics can be tested;
- keep `--emit-ir` and `--emit-asm` useful as inspectable compiler stages.

Exit criteria:

- At least one non-trivial program can be compiled, linked, executed, and
  compared to `phpc run`.
- Runtime-backed native diagnostics are tested.
- Docs state exactly which constructs are native-executable versus
  interpreter-only.

### 3. Runtime Semantics: References and Copy-On-Write

Implement the PHP value semantics that affect real programs before expanding
syntax too far.

Scope:

- reference cells and aliasing boundaries;
- array/object copy-on-write behavior;
- function parameter and return interactions with references;
- mutation behavior for arrays, objects, foreach, and builtins;
- explicit unsupported diagnostics where exact PHP behavior remains out of
  scope.

Exit criteria:

- Aliasing and copy-on-write behavior has targeted runtime tests and CLI
  fixtures.
- Existing array/object builtins either honor the new semantics or document and
  test their temporary limits.
- Native lowering rejects or uses runtime helpers honestly.

### 4. Object Execution

Turn current class metadata and minimal object values into executable object
semantics.

Scope:

- broader method dispatch beyond public static-name instance calls;
- broader constructor semantics beyond public/inherited public instance
  `__construct` and explicit parent calls, including non-public constructor
  visibility;
- visibility enforcement for public/protected/private access;
- static methods, broader static member semantics, broader class constant
  semantics, broader late static binding, and broader class-name resolution;
- full object identity behavior including clone/destruction/reuse boundaries,
  exact `spl_object_hash` formatting, and native object handles;
- inheritance after the single-class model is stable.

Exit criteria:

- Public static-name instance method calls execute through `phpc run` with
  scoped `$this`; broader dispatch remains named and tested as unsupported.
- Constructor and visibility behavior have fixtures and stable errors.
- Introspection builtins are updated to reflect implemented object semantics.
- Native object lowering remains explicit: implemented subset or precise
  rejection.

### 5. Program Structure

Add PHP program-structure features needed by real codebases.

Scope:

- include/require resolution and execution;
- namespaces, imports, aliases, and qualified name resolution;
- autoload boundary design before automatic class loading;
- source mapping for magic constants across files;
- clear fallback policy for highly dynamic cases.

Exit criteria:

- Multi-file programs can run through `phpc run` for a documented subset.
- Include paths, repeated includes, and failure modes have CLI coverage.
- Namespace-aware functions, constants, and classes are tested.
- Unsupported autoload and dynamic resolution cases are named.

### 6. Exceptions and Error Model

Replace project-specific fatal shortcuts with more PHP-shaped error behavior
where it matters.

Scope:

- `throw`, `try`, `catch`, and `finally`;
- `Throwable`, `Exception`, and core error object boundaries;
- partial stdout and output buffering behavior;
- warnings/notices where PHP continues execution;
- stack traces or an explicit documented substitute.

Exit criteria:

- Exception control flow has fixtures for normal, thrown, caught, and finally
  paths.
- Runtime diagnostics distinguish project compiler errors from PHP runtime
  errors.
- Tests pin stdout/stderr/exit behavior.

### 7. Standard Library and Extensions

Expand builtins according to real program needs and extension plans, not by
surface-area guessing.

Scope:

- prioritize core functions used by fixtures and target applications;
- implement `json`, `mbstring`, XML, `curl`, `openssl`, database, cache, and
  compression extension slices only behind focused docs in `docs/extensions/`;
- keep unsupported extension functions and constants documented.

Exit criteria:

- Every builtin or extension slice has signature docs, type/error behavior,
  fixture coverage, and unsupported edge cases.
- Extension support claims are scoped to executable code.
- Dynamic function-call availability is tested when a builtin is callable.

### 8. Application Compatibility Targets

Choose small real PHP programs as compatibility targets once the runtime is
ready enough.

Scope:

- start with self-contained scripts using arrays, functions, classes, includes,
  and selected core builtins;
- then move to framework-shaped examples;
- move to WordPress deliberately: first bootstrap/load targets, then selected
  admin/front-end/WP-CLI flows, then representative plugin and theme fixtures;
- keep database, filesystem, HTTP, cache, extension, and host-environment
  assumptions pinned per target.

Exit criteria:

- Each target has a committed fixture or sample app.
- `phpc run` behavior is compared against system PHP where feasible.
- Native compile behavior is either supported, partially supported with
  documented fallback, or rejected with exact diagnostics.
- WordPress targets record the WordPress version, PHP target version, required
  extensions, database/filesystem assumptions, and known unsupported behavior.

### 9. Compatibility Harnesses

Make compatibility measurable before claiming it.

Scope:

- add a PHP-version compatibility manifest that names target PHP branches,
  feature groups, fixtures, expected comparison mode, and known unsupported
  deltas;
- add a path for importing or mirroring small php-src-style behavioral tests
  where licensing and scope allow;
- add a pinned WordPress core compatibility harness that starts with parse/load
  inventory, then bootstrap fixtures, then request/CLI smoke tests;
- report compatibility status as data, not prose: pass, fail, skipped with a
  named unsupported reason, or not-yet-covered.

Exit criteria:

- A single command can run the committed PHP compatibility smoke suite.
- A single command can run the committed WordPress compatibility smoke suite
  without requiring undocumented host state.
- Failures distinguish compiler bugs from intentionally unsupported gaps.
- `docs/SUPPORT.md` and any generated manifest agree on support status.

### 10. Release-Quality Toolchain

Make the compiler usable outside the test suite.

Scope:

- stable CLI UX and exit codes;
- structured diagnostics with source spans;
- installable binary packaging;
- reproducible backend discovery;
- benchmark and smoke-test commands;
- contributor docs that explain how to add a PHP feature correctly.

Exit criteria:

- A user can install/build `phpc`, run examples, inspect support docs, and
  understand unsupported failures.
- CI or local automation runs the full test suite and representative CLI
  smoke tests.
- Release notes distinguish interpreter, IR, assembly, and native-executable
  support.

## Near-Term Queue

Start from `docs/NEXT_TASKS.md`; it is the source of truth for checked and
unchecked work. Current state:

- Milestone 850 is committed: a deterministic `mysqli_query()` placeholder
  boundary covers the reached WordPress charset setup statement and the real
  `wp-blog-header.php` front-controller probe exits `0` with no stdout under
  current placeholder database and CLI assumptions.
- Milestone 851 is committed: the passing front-controller probe has a
  committed deterministic smoke target that preserves the distinction between
  placeholder CLI/bootstrap success and full WordPress support.
- Milestone 852 is committed: the first deterministic empty `mysqli_result`
  lifecycle boundary for WordPress-style result consumption.
- Milestone 853 is committed: a synthetic `wpdb::query()`-style smoke consumes
  the placeholder empty result through WordPress-shaped object state.
- Milestone 854 is committed: remaining `SELECT`
  statements after the deterministic placeholder cases fail with a specific
  non-empty `mysqli` result-set diagnostic.
- Milestone 855 is committed: the exact deterministic
  query `SELECT ID, post_title FROM wp_posts WHERE ID = 1` returns a
  placeholder row-backed `mysqli_result` with two fields and one `stdClass`
  row, while all general SQL/database behavior remains unsupported.
- Milestone 856 is committed: a synthetic
  `wpdb::get_results()` smoke consumes that one-row placeholder through
  WordPress-shaped `last_result` and `num_rows` state without claiming real
  database support.
- Milestone 857 is committed: deterministic
  `mysqli_fetch_assoc()` returns the seed-post placeholder row as an
  associative PHP array and shares the placeholder result cursor with object
  fetching.
- Milestone 858 is committed: a synthetic
  `wpdb::get_results($query, ARRAY_A)` smoke consumes the associative row path
  through WordPress-shaped `last_result` and `num_rows` state without claiming
  real database support.
- Milestone 859 is the active checkpoint candidate: deterministic
  `mysqli_fetch_array($result, MYSQLI_ASSOC)` returns the seed-post
  placeholder row as an associative PHP array while numeric and mixed fetch
  modes remain named unsupported boundaries.
- The hard compatibility lanes remain open in parallel: PHP value semantics
  (`references` and copy-on-write), object semantics, error/exception behavior,
  standard-library and extension services, WordPress harness breadth, and
  native runtime integration.
- Older lane placeholders remain historical planning data; the live queue is
  the checked/unchecked work in `docs/NEXT_TASKS.md`, reconciled after each
  checkpoint.

Prefer depth in one documented subset over breadth across unimplemented PHP
features.

## Verification Commands

Use focused commands while developing:

```sh
cargo test -p php_runtime
cargo test -p phpc
cargo run -p phpc -- test
cargo run -p phpc -- test --compare-php tests/fixtures/milestoneN
cargo run -p phpc -- run tests/fixtures/milestoneN/example.php
cargo run -p phpc -- compile tests/fixtures/milestoneN/example.php --emit-ir
cargo run -p phpc -- compile tests/fixtures/milestoneN/example.php --emit-asm
```

Use the full gate before checkpointing or after an integration batch that
combines lane work:

```sh
tools/run-tests.sh
```

When running unattended loop automation, checkpoint only through:

```sh
tools/checkpoint.sh "short accurate message"
```

## Non-Goals For Now

- Unproven full PHP compatibility claims.
- Unproven WordPress compatibility claims.
- Zend extension loading.
- Regex parser rewrites.
- A custom x86-64 backend before the LLVM/runtime path is justified.
- Silent native compilation of constructs that only the interpreter supports.
- Broad standard-library stubs that return placeholder values.
