# PHP-to-Native Compiler

This project is an experimental PHP-to-native compiler implemented in stable
Rust. It is intentionally small and honest: implemented features are tested,
unsupported features are documented, and native code generation starts with LLVM
IR text.

The current project has two execution surfaces:

- `phpc run`, an interpreter/runtime path for the supported PHP subset.
- `phpc compile`, a narrower native-code path that emits LLVM IR or assembly for
  straight-line programs and rejects unsupported lowering with structured errors.

For the long-term compatibility target, read `GOAL.MD`. For exact support
boundaries, read `docs/SUPPORT.md`. For design notes, read
`docs/ARCHITECTURE.md`. For the chronological proof log, read
`docs/PROGRESS.md`. For the copy-on-write consolidation audit, read
`docs/COW_COVERAGE_MATRIX.md`.

## Build

```sh
cargo build
```

## CLI

```sh
cargo run -p phpc -- run examples/hello.php
cargo run -p phpc -- compile examples/hello.php --emit-ir
cargo run -p phpc -- compile examples/hello.php --emit-asm
cargo run -p phpc -- compile examples/hello.php --emit-exe /tmp/hello-phpc
cargo run -p phpc -- test
cargo run -p phpc -- test --list-fixtures
cargo run -p phpc -- test --list-fixtures-json
cargo run -p phpc -- test --compare-php-json
cargo run -p phpc -- test --php-versions-json
```

The installed binary name is `phpc`.

### `phpc run`

`phpc run <input.php>` parses the supported PHP subset and executes it through
the Rust runtime. This is the broadest implemented path today.

The runtime is PHP-shaped rather than Rust-shaped: values are boxed, arrays keep
PHP-style integer/string keys, supported object values use runtime class
metadata, and unsupported dynamic behavior fails with stable diagnostics instead
of silently pretending to work.

For bounded request/SAPI exercises, `phpc run` accepts explicit environment
seeds: `PHPC_REQUEST_TIME`, `PHPC_QUERY_STRING`, `PHPC_REQUEST_METHOD`,
`PHPC_CONTENT_TYPE`, `PHPC_REQUEST_BODY`, `PHPC_COOKIE`, and `PHPC_FILES`.
These populate bounded URL-encoded `$_GET`/`$_POST`/`$_REQUEST` data,
including bracketed names, repeated `[]` values, and top-level dotted/spaced
request names normalized to underscores. `PHPC_COOKIE` seeds `$_COOKIE` from
a semicolon-delimited cookie header string and exposes the raw value through
`$_SERVER["HTTP_COOKIE"]`;
cookies are not merged into `$_REQUEST`. `PHPC_FILES` seeds explicit
`$_FILES` upload metadata from URL-encoded keys such as
`async-upload[name]=plugin.zip&async-upload[error]=0`; it does not parse
multipart bodies or create temporary upload files. `is_uploaded_file()` and
`move_uploaded_file()` use only the initial `PHPC_FILES` `tmp_name` entries
with `error=0` as bounded local upload provenance. `PHPC_REQUEST_BODY` also
seeds `php://input` for the interpreter only. `session_start()` now
materializes a bounded in-memory `$_SESSION` array for the current CLI request;
direct function-scope reads/writes route through that session root, including
covered nested reference aliases. `session_start(["read_and_close" => true])`
materializes that root and immediately closes the bounded active status while
keeping `$_SESSION` visible. Starting a session after unbuffered output
returns `false` and emits a bounded `E_WARNING` through the current
`set_error_handler()` stack or stderr fallback. Starting an already active
session emits a bounded `E_NOTICE`, returns `true`, preserves the active
session and data, and ignores `read_and_close` for that restart attempt.
Successful fresh starts append a deterministic `Set-Cookie: PHPSESSID=<id>`
header to the same CLI header log exposed by `headers_list()`;
`session_start(["use_cookies" => false])` suppresses that bounded cookie
header for the start. Bounded session cookie attributes from
`cookie_lifetime`, `cookie_path`, `cookie_domain`, `cookie_secure`,
`cookie_httponly`, and `cookie_samesite` options are appended to that
deterministic header. Successful fresh starts also append the bounded default
no-cache session headers (`Expires`, `Cache-Control`, and `Pragma`) to the
CLI header log, including starts that suppress the cookie with
`use_cookies=false`.
`session_cache_limiter()` and `session_cache_expire()` expose bounded
request-local cache-header configuration before output or session start: the
default limiter is `nocache`, the default expiration is `180`, setting the
limiter to an empty string suppresses session cache headers, and restoring
`nocache` re-enables the deterministic no-cache trio. The `private`,
`private_no_expire`, and `public` limiters emit bounded deterministic
`Cache-Control`/`Expires`/`Last-Modified` variants using
`session_cache_expire()` minutes, the bounded request timestamp from
`PHPC_REQUEST_TIME` when supplied, and the main script modification time for
`Last-Modified` when the source file can be statted. Without an explicit
request-time seed, the CLI fallback remains a fixed synthetic epoch for
fixture stability.
`session_write_close()` stores a request-local snapshot for the current
session id, so a later `session_start()` reloads the last closed data instead
of preserving mutations made to visible `$_SESSION` while the bounded session
was closed. When `ini_set("session.save_path", $path)` supplies an explicit
local save path and `session_id($id)` supplies a bounded alphanumeric,
underscore, or hyphen id before start, `session_start()`/`session_write_close()`
also load and write PHP-compatible `sess_<id>` files for scalar and array
string-keyed session values across separate `phpc run` invocations. Malformed
or unsupported existing session files emit one bounded recoverable warning and
recover with an empty session array. Starting a session with an explicit id
outside that bounded file-safe subset returns `false` with a bounded warning
before headers or `$_SESSION` are changed.
Fresh `session_start()` cookie headers replace earlier deterministic
`PHPSESSID` cookie headers with the same normalized non-empty path and
ASCII-case-insensitive normalized non-empty domain identity in the CLI header
log, while same-name cookies for different path/domain identities are kept.
Locking, save handlers, garbage collection, broader PHP session-id policy,
integer top-level session keys, object/resource session values, option effects
beyond the documented session-start options, exact malformed-session recovery
parity, session-cookie encoding, expiration-date formatting beyond the bounded
`Max-Age` attribute, broader replacement policy, real SAPI `Date` header emission,
host-webserver request-time initialization, cache-header variants beyond empty
suppression, `nocache`, `private`, `private_no_expire`, and `public` remain
unsupported. `setcookie()`/`setrawcookie()` separately support a bounded
deterministic CLI header-log slice for encoded or raw string values,
expiration dates with host-clock-derived `Max-Age`, attributes, and
path/domain-aware replacement with ASCII-case-insensitive domain identity
matching; options-array calls match the documented option keys
case-insensitively, use the last inserted value for duplicate differently cased
documented keys, and reject numeric keys or unknown string option keys before
changing that header log. Exact `ValueError` objects/text, cookie name
validation/encoding, full domain policy, real SAPI emission, and native
lowering remain unsupported. `fopen()` can create bounded
interpreter-owned `php://memory`,
`php://temp`, `php://input`, local absolute `file://` URL with bounded UTF-8
percent-decoded path portions, and local UTF-8 file stream resources for simple
flows through `fwrite()`, `fscanf()`, `fread()`, `rewind()`, `stream_get_contents()`,
`feof()`, `ftell()`, `fseek()`, `fflush()`, `ftruncate()`, `fstat()`,
`stream_get_meta_data()`, and
`fclose()`. `php://input` handles read the deterministic
`PHPC_REQUEST_BODY` seed and stay non-writable. Bounded
`stream_context_create()` resources store array options and bounded params
(`notification` plus `options`), expose them through
`stream_context_get_options()` and `stream_context_get_params()`,
`stream_context_get_default()` returns a request-local default context, and
`stream_context_set_default()`, `stream_context_set_option()`, plus
`stream_context_set_params()` persist string-keyed wrapper options and bounded
params on those contexts. Context resources may be passed to the current
`file_get_contents()`/`fopen()` local, local absolute `file://`, and
`php://input` paths without applying wrapper-specific behavior, and
`file_get_contents()` accepts bounded integer offset plus optional
non-negative length reads over those UTF-8 payloads. Local absolute `file://`
URLs with an empty host or `localhost` are also accepted by the current
include/require resolver after the same bounded UTF-8 percent-decoding.
When `ini_set("open_basedir", $path)` configures a non-empty
request-local allow-list, local `file_get_contents()`/`fopen()` paths,
covered local metadata/mutation helpers, and local `file://` URLs are checked
against those bounded directories before opening or inspecting; denied
operations emit a bounded PHP-style display `E_WARNING`, return `false`, and
do not populate the realpath cache.
Missing local files and negative offsets before the start of those payloads
emit bounded PHP-style `E_WARNING` events, return `false`, and continue; the
current slice can route those warnings through the top registered string or
public array-callable `set_error_handler()` handler before the stderr fallback,
with `restore_error_handler()` restoring the previous bounded handler. Local
`fopen()` open failures, including missing read targets, use the same bounded
warning-plus-`false` recovery path and continue execution.
`opendir()`, `readdir()`, `rewinddir()`, and
`closedir()` cover bounded local UTF-8 directory handles. `clearstatcache()`
accepts the PHP-shaped zero-, one-, or two-argument forms and clears the
bounded request-local successful metadata cache used by `filesize()` and
`filemtime()`, either globally or for one local path; successful local-file
`fopen()` create/truncate, `fwrite()`, and `ftruncate()` mutations also clear
that bounded metadata cache for the affected path. `tempnam()` emits the
bounded PHP-shaped system-temporary-directory fallback notice before a fallback
`open_basedir` denial. Successful `realpath()` calls populate bounded
request-local `realpath_cache_get()` entries;
one-argument `clearstatcache(true)` clears those entries, while
`clearstatcache(true, $filename)` removes only a non-empty exact matching
cached resolved-path key. Successful local `file_get_contents()` reads, local
`fopen()` calls for paths that existed before opening, and successful local
include/require reads also populate bounded request-local realpath-cache
entries for the resolved target path.
`realpath_cache_size()` reports `0` for an empty bounded realpath cache and a
deterministic positive request-local size for cached resolved UTF-8 paths;
exact PHP memory-byte accounting remains unsupported. Bounded
`register_shutdown_function()` callbacks run supported string and public
array-callable callbacks with by-value extra arguments during normal shutdown
and after the bounded `exit()` path, before object destructors and final
output-buffer flushing; callbacks registered during shutdown are appended to
the same queue. Unsupported wrappers beyond the bounded local `file://` and
documented `php://` subset,
filters, malformed `file://` percent escapes, decoded NUL bytes, non-UTF-8
percent-decoded paths, context option effects, context param effects beyond
option merging, broader wrapper metadata,
binary byte fidelity, directory entry ordering fidelity, multipart upload
parsing, runtime temporary upload creation, host upload validation,
permissions/locking, realpath-cache ancestor entries and broader
`open_basedir` policy beyond covered local filesystem helpers,
stat-cache/realpath-cache state beyond those local read/mutation paths,
closure shutdown callback execution,
invokable-object shutdown callbacks, exact warning text and error-handler
integration beyond the documented `file_get_contents()` and local `fopen()`
open-failure recovery stack slices, temp-file spillover, and native stream
resources remain unsupported. Native lowering
still rejects request/session/stream state
until a native runtime ABI exists.

### `phpc compile --emit-ir`

`phpc compile <input.php> --emit-ir` emits LLVM IR text for a smaller
straight-line subset. It currently supports scalar literals, direct scalar
variable assignment/readback, scalar `echo`/`print`, selected scalar operators,
selected folds, and a documented set of native builtin folds.
Native metadata/type-introspection builtin families share a backend-neutral
preflight for class/interface/trait/enum existence, property/method existence,
and relationship metadata calls, so call-result argument dependencies and arity
failures route through the same native call diagnostics in LLVM and generated C.

Anything outside that lowerable subset is rejected before misleading IR is
emitted. Arrays, objects, class-name constants, `instanceof` relationship
checks, ArrayAccess object-offset dispatch, clone expressions, include/require
expression return semantics, functions, general control flow,
try/catch/finally exception control, references, copy-on-write, and broad PHP
coercions remain interpreter-only or unsupported for native lowering. Request
superglobals remain rejected in native lowering even though the native runtime
ABI now pins a null-only request-state handle shape. Try blocks are rejected
through a dedicated native diagnostic until catch matching, catch variable
binding, finally execution, and stack unwinding have native semantics.
The compile mode flag is validated before the input file is read, so invalid
modes such as `--emit-object` report a stable CLI usage error instead of an
unrelated file, parse, or codegen diagnostic.

### `phpc compile --emit-asm`

`phpc compile <input.php> --emit-asm` first performs the same LLVM lowering as
`--emit-ir`. If lowering succeeds, assembly backend selection is:

1. `clang`
2. `llc`
3. `cc -S` over generated narrow-subset C, as a temporary bootstrap fallback

The C fallback keeps assembly emission usable on machines without LLVM tools. It
is not the long-term backend.

Backend behavior is covered by CLI fixtures, including backend discovery order,
missing tools, failed probes, selected-backend failures, empty or whitespace-only
assembly output, stderr handling on successful assembly, stdin handoff, and
argument validation. The tests normalize success output instead of snapshotting
platform-specific assembly text.

### `phpc compile --emit-exe`

`phpc compile <input.php> --emit-exe <output>` builds the first bounded linked
native executable path. It emits C for the current generated-native subset,
builds and links the Rust `php_runtime` static library with `cc`, and routes
direct output through runtime string/value stdout helpers. Bounded direct
`exit()`/`die()` calls now terminate generated-native executables for
materializable `null`, `int`, and `string` operands: strings are written to
stdout, integer operands become the process status, and owned native runtime
handles are cleaned before returning from `main`. Bounded `if`/`else`
statements can also lower when conditions use the existing native truthiness or
value-comparison boundaries and both branches leave persistent variable/cleanup
state unchanged. Generated-native casts route through the shared native
value-result carrier, so selected array-to-string casts produce `"Array"` and
report `Warning: Array to string conversion` through the same diagnostic path
as neighboring native value operations. Generated-native object dispatch also
shares one argument-handle materialization boundary across declared
constructors, declared methods, static calls, callable-array method branches,
invokable-object branches, and constructorless argument arrays. Dynamic
instance method-name matching uses a method-specific runtime helper that
normalizes supported scalar method-name operands instead of reusing the
string-only dynamic function-call matcher.

This is not broad native PHP support. Objects, functions, references,
request/session/stream/header state, exceptions, includes, shutdown callbacks,
destructors/finally ordering, output buffers, SAPI interaction, branch
environment merging, loops/switch/goto/break/continue, dynamic string-pointer
helper lowering, object `__toString()`/resource cast parity, and broad PHP
coercions remain unsupported or limited to the existing native diagnostics.
LLVM object/class dispatch parity, broader method-frame execution,
visibility/magic/typed-property policy, and constructor/destructor lifecycle
parity remain open.

## Current Status

Milestone 1 is in progress. The interpreter path is intentionally ahead of the
native path; native codegen must reject unsupported programs rather than emit
incorrect native code.

### Interpreter Path

`phpc run` currently supports the documented subset of:

- literals, variables, assignment, direct `unset`, `isset`, `empty`, and null
  coalescing forms, plus bounded inline HTML output between PHP close/open
  tags; short echo tags such as `<?= $value ?>` remain a lex boundary
- scalar arithmetic, concatenation, comparisons, logical operators, bitwise
  operators, shifts, `(string)`, `(int)`, `(bool)`, `(float)`/`(double)`, and
  `(array)` casts over documented
  current value boundaries, ternaries, increments/decrements, and PHP
  error-control syntax `@expr` as a transparent runtime wrapper without
  warning/notice suppression. Runtime increment/decrement includes current
  string `++`/`--` behavior for numeric strings, PHP-shaped float promotion at
  signed 64-bit overflow, and terminal ASCII-alphanumeric string increments.
  PHP 8 `match` expressions execute in the
  runtime path for the current strict-comparison expression subset; native
  lowering rejects them until exact native match semantics are implemented.
- `if`, loops, `switch`, `break`/`continue` including positive integer literal
  loop-depth arguments, bounded `goto`/label execution,
  `foreach`, and user functions with local scopes, bounded function-local
  `static` variables, defaults, trailing variadic parameters, returns,
  dynamic string-valued calls,
  bounded function-scope `global $name, ...;` imports for direct variables,
  bounded direct string-keyed `$GLOBALS['name']` root-symbol reads/writes,
  bounded namespace-scoped function declarations and unqualified same-namespace
  calls with global fallback lookup,
  no-capture anonymous, static anonymous, and non-static arrow closure
  values with bounded direct/callback execution for ordinary closures,
  and recursion guarded by a fixed depth limit;
  parameter/return type syntax is accepted as metadata only, without runtime
  type enforcement, while parenthesized DNF-shaped type declarations and
  call-site argument unpacking such as `handler(...$args)` plus call-time
  by-reference arguments such as `handler(&$value)` remain parse boundaries
- top-level `global $name, ...;` declarations as no-op/import-compatible
  statements
- ordered arrays with integer/string keys, array literals, indexed reads/writes,
  append writes, nested direct-variable array-offset assignment expressions,
  append-at-depth assignment expressions, direct-object-property nested array
  assignment and append-at-depth expressions, direct/nested array offset
  removal, nested object-property array offset removal, array iteration
  including bounded by-reference iteration over direct array, nested array,
  superglobal/request-bag, string-keyed `$GLOBALS`, and visible
  named, direct dynamic, and bounded object-result non-direct named/dynamic
  object-property array roots, direct and visible property-held `ArrayAccess`
  offset-array roots, and bounded non-direct holder property-held
  `ArrayAccess` offset-array roots backed by the exact bounded by-reference
  `offsetGet()` bridge, plus direct
  free-function, direct visible
  instance-method, direct named-static-method, method-context
  `self::`/`parent::`/`static::`, dynamic static receiver, and bounded
  `call_user_func_array()` reference-return iterable roots when the returned
  direct variable is backed by a caller variable cell, plus the bounded
  multi-alias child-array shape where a reference-returning function returns
  `return $param[$key];` from a direct caller variable that shares a cell with
  another direct name, plus bounded
  direct array-offset by-reference parameter writeback with `unset($param)`
  detachment for ordinary arrays and request bags, plus bounded alias cleanup
  when direct array/object roots with covered child aliases are removed through
  `unset($name)`, plus bounded
  direct free-function, visible object-method, and current static dispatch
  reference-return assignment that binds a returned by-reference parameter
  back to covered direct array-offset and visible named object-property
  array-offset arguments, including public slots and private/protected slots
  reached from valid method visibility contexts in the current named-property
  array-offset slice, plus the narrow
  `return $param[$key]` and
  `return $param[$key][$subkey]` child-slot shapes when `$param` was supplied
  by a direct variable parent array, by a direct variable already backed by a
  covered array-offset alias, or by a covered parent array/property slot,
  plus bounded
  direct array-offset reference elements in literal `call_user_func_array()`
  argument arrays for request bags, `$GLOBALS`, and nested arrays,
  plus bounded
  preservation of covered reference elements when copying literal-key nested
  direct array paths such as `$_REQUEST["payload"]`,
  and
  positional statement-form
  `list($a, $b) = expr;` plus `[$a, $b] = expr;` assignment over numeric
  keys, including skipped slots
- top-level constants, namespace-scoped top-level `const` declarations in the
  current unbracketed namespace slice, selected built-in constants,
  runtime-defined constants with bounded qualified string names, simple
  interpolated runtime string names for `defined()`/`constant()`, bounded
  runtime string lookup of declared public class constants through
  `defined("ClassName::CONST")` and declared visible class constants through
  `constant("ClassName::CONST")`, and
  executable magic constants documented in the support matrix
- statement-form `throw expr;` as a bounded exception boundary: guarded throws
  can parse and be skipped, while reached throws report a stable runtime
  diagnostic without constructing exception objects or unwinding the stack
- statement-form `try`/`catch`/`finally` blocks as a bounded exception
  boundary: non-throwing try bodies execute, catch bodies are skipped without a
  thrown exception, finally bodies execute after normal try completion, and
  reached throws still report a stable runtime diagnostic before catch matching
  or unwinding exists
- narrow `require`, `require_once`, `include`, and `include_once` execution
  for local string paths in statement and expression position, including
  constant/string-concatenated paths resolved relative to the current source
  file or through the current bounded include path, included files executing
  in caller scope, include return values, missing local `include`/
  `include_once` warning-plus-`false` recovery, and `_once` de-duplication by
  resolved local file
- bounded deterministic `mysqli`/`wp_options` state-island behavior for
  WordPress bootstrap probes, including exact option insert/update/delete/read
  shapes and selected prepared option-value-only equality reads including
  `LIMIT 1`, option-name-only, option-name-list, full-row, explicit
  full-row-with-id equality with and without `LIMIT 1`, star-projection
  including exact option-name equality with and without `LIMIT 1`,
  name/autoload-list, autoload-only option-name equality for
  `update_option()`-shaped probes, and autoload-list/equality result sets,
  plus one-shot `mysqli_execute_query()` prepared option insert/update/replace/delete
  mutations and deterministic prepared-statement insert-ID metadata for
  option/transient-shaped state probes,
  plus bounded prepared-statement result binding where
  `mysqli_stmt_fetch()` can consume the deterministic executed placeholder row
  without `mysqli_stmt_store_result()` while `mysqli_stmt_num_rows()` remains
  buffered-only,
  plus bounded direct option-name `LIKE` result scans/deletes with `%`, `_`,
  backslash escapes, and single-character `ESCAPE` clauses, bounded prepared
  transient-shaped option-name prefix result scans with single-character
  `ESCAPE` clauses, SQL-mode-aware direct option-name equality reads and
  equality/`IN` deletes for the bounded `NO_BACKSLASH_ESCAPES` branch, and
  SQL-mode-aware prepared option-name `LIKE` scans/deletes for that same
  bounded branch, including exact
  `ORDER BY option_name` suffixes on those scans and a bounded
  expired-transient-timeout
  `option_name LIKE ... AND option_value < timestamp` option-name scan,
  timeout-row delete shape, and exact WordPress-shaped transient payload plus
  timeout pair delete shape, plus deterministic `SHOW TABLES LIKE
  'wp_options'`, `DESCRIBE`/`DESC wp_options`, and `SHOW [FULL] COLUMNS FROM
  wp_options`, and `SHOW INDEX`/`SHOW KEYS FROM wp_options` schema probe rows
  for the current option table, plus a bounded per-handle dynamic schema island
  for exact `CREATE TABLE`, `ALTER TABLE` add/change/modify/drop column and
  add/drop index probes with bounded column default/nullability/
  auto-increment metadata, inline column key metadata, `ASC`/`DESC`
  index-part ordering metadata, and bounded `FULLTEXT`/`SPATIAL` index type
  metadata, plus a bounded dbDelta-style repeated `CREATE TABLE` diff that
  upserts declared columns/indexes on an existing recorded table while
  preserving omitted recorded columns/indexes, and later `DESCRIBE`/`SHOW COLUMNS`/
  `SHOW INDEX`/`SHOW CREATE TABLE`/`SHOW TABLE STATUS` inspection, including
  bounded schema metadata `LIKE` wildcards and single-character `ESCAPE`
  clauses, literal `SHOW TABLE STATUS WHERE Name IN (...)` table-name lists,
  plus a bounded `NO_BACKSLASH_ESCAPES` branch for those schema
  metadata `LIKE` filters, and one-string-parameter prepared metadata filters
  for the documented `SHOW TABLES`, `SHOW TABLE STATUS`, `SHOW COLUMNS`, and
  `SHOW INDEX`/`SHOW KEYS` equality/`LIKE` forms, including exact
  `SHOW TABLE STATUS WHERE Name = ?` table-name probes and bounded prepared
  `SHOW TABLE STATUS WHERE Name IN (?, ...)` table-name lists, plus bounded
  table-identifier placeholders for `SHOW [FULL] COLUMNS FROM ?` and
  `SHOW INDEX`/`SHOW KEYS FROM ?` with optional documented field/key filters;
  this is not real MySQL connectivity, arbitrary SQL, broad mutable schema, real
  index inspection, expression indexes, fulltext parser clauses, index
  opclass/parser metadata, exact
  table-status counters or timestamps, full dbDelta diffing, real transactional DDL
  beyond bounded in-memory schema snapshots, persistent object cache, full
  `wpdb`, or native database support
- a bounded namespace/class-name/function slice: multiple unbracketed named
  `namespace` declarations per file, simple top-level class `use` imports with
  optional `as` aliases, including comma-separated class import lists and
  class-import prefix expansion for qualified function calls,
  namespace-qualified class declarations, class imports for class-like
  references, `new`, `extends`, `instanceof`, static members, and
  `ClassName::class`, plus namespace-scoped function declarations and
  unqualified, qualified, namespace-relative, fully-qualified, and imported
  direct function calls
- declared interface metadata: top-level `interface Name {}` declarations,
  parent forms such as `interface Child extends Parent, OtherParent`,
  including parent interfaces declared later in the same parsed program, and
  public method signatures parse, register class-like interface names, power
  `interface_exists()` and
  `get_declared_interfaces()`, participate in the current bounded autoload
  callback path for truthy-autoload `interface_exists()` misses, and
  require concrete classes that implement
  declared interfaces, including through inherited `implements` metadata and
  the current parent interface inheritance slice, to expose
  public methods with the required names and matching static/non-static shape
  at class registration time, to avoid requiring more parameters than those
  interface methods, to pass the current bounded interface parameter-type
  metadata check, and to pass the current bounded interface return-type
  metadata check, including simple declared class/interface contravariant
  parameter and covariant return relationships when both type names resolve
  through current metadata; child interfaces that redeclare inherited methods
  and simple multi-parent method conflicts are checked against the same bounded
  staticness, required-parameter, parameter-type, and return-type metadata
  rules; class `implements` clauses
  record comma-separated interface names and inherited parent interface names
  as relationship metadata for `is_a`, `is_subclass_of`, and `instanceof`,
  including unresolved built-in/internal interface names; public interface
  constants in the current untyped expression subset resolve through
  interface names, parent-interface inheritance, implementing classes, and
  string `defined()`/`constant()` lookups; missing or cyclic parent interface
  inheritance reports stable runtime boundaries;
  typed/non-public/abstract/final or
  multi-constant interface declarations, full variance/signature compatibility
  beyond the current bounded checks, namespace-aware type-name resolution,
  union/intersection canonicalization, class/interface type subtyping beyond
  declared simple names,
  broad built-in/internal interface
  inheritance catalogs, exact PHP diagnostics, and native lowering remain
  unsupported; the current internal-interface enforcement slice is limited to concrete
  `Countable` implementors exposing a public non-static `count()` method with
  no required parameters and concrete `Iterator`/`IteratorAggregate`
  implementors exposing their required public non-static methods with no
  required parameters
- a minimal object/class slice: class metadata, `new ClassName(...)` with
  public and inherited public instance `__construct`, public instance
  property reads/writes, inherited instance property slots with
  declaring-class ownership, private same-declaring-class and protected
  same-class/child property reads/writes, `isset`/`empty`, read-modify-write,
  direct object-property array-offset `isset(...)`, and null-coalescing forms,
  compatible public/protected inherited property
  redeclarations sharing one runtime slot,
  braced nested class declarations that register only when execution reaches
  the `class` statement,
  parsed `abstract`/`final` class modifiers and `abstract`/`final` method
  modifiers as metadata, with abstract class instantiation rejected as a
  runtime boundary, final class inheritance, final method overrides, method
  visibility reductions, and inherited method static/non-static compatibility
  violations rejected as runtime boundaries, plus inherited non-constructor
  method required-parameter and type compatibility incompatibilities rejected
  as runtime boundaries, including simple declared class/interface
  contravariant parameter and covariant return relationships when both type
  names resolve through current metadata, concrete
  classes with unimplemented abstract methods rejected as runtime boundaries,
  and readonly class declarations kept at a parse boundary,
  bounded `new self`, `new parent`, and `new static` class-name instantiation
  in active class/method contexts, plus direct-variable dynamic class-name
  instantiation for `new $class(...)`; missing named or direct-variable
  string class names invoke currently registered string user-function,
  public object-method array-callable, public class-string static-method, and
  public invokable-object autoload callbacks before the class table is
  rechecked; class declarations loaded by executed include/require paths also
  invoke those callbacks for missing `extends` parent classes, direct
  `implements` interfaces, and direct class-body trait `use` names, while
  interface declarations loaded through that path invoke them for parent
  interfaces reached from autoloaded interface declarations before final
  registration validation; `spl_autoload_call($class)` manually invokes the
  current bounded callback list for class/interface/trait names,
  `spl_autoload_functions()` exposes the current bounded callback list, and
  `spl_autoload_unregister()` removes matching bounded callbacks,
  `spl_autoload_extensions()` reads and replaces the request-local extension
  string used by PHP's default SPL autoload surface, and bounded
  `spl_autoload($class)` probes local lowercased class/interface/trait file
  names through that extension registry and the current include resolver,
  `class_alias($class, $alias, $autoload = true)` can autoload a missing
  source class or interface and register a bounded metadata alias for class
  lookup, instantiation, interface lookup, and relationship checks,
  while parenthesized dynamic class-name expressions such as `new ($class)()`
  remain a dedicated parse boundary,
  metadata-only built-in `Exception` and `stdClass` class seeds, including
  no-argument instantiation and user subclasses for `Exception`,
  public and same-class private instance method calls, inherited public method
  calls, protected same-class/child method calls, explicit `parent::method()`
  and `parent::__construct()` calls in instance context, narrow
  `self::method()` calls in instance context, class-method default parameters
  using `self::CONST` from the declaring method class, narrow `ClassName::class`,
  `self::class`, and `parent::class` resolution, narrow class constants
  through `ClassName::CONST`, `self::CONST`, `parent::CONST`, and late-bound
  `static::CONST` in active called-class context,
  narrow static properties through `ClassName::$prop`, `self::$prop`,
  `parent::$prop`, and late-bound `static::$prop` in active called-class
  context with direct reads/writes, compound assignment, pre/post
  increment/decrement, `isset`/`empty`, `??`, `??=`, and stable `unset(...)`
  diagnostics for PHP-forbidden static-property unset,
  dynamic static method calls through `$object::method()` and
  `$className::method()` for visible static methods,
  dynamic property-name reads/writes for existing public slots and `stdClass`
  public dynamic slots when property-name values are strings or integers,
  `clone $object` for current object values with fresh object handles,
  shallow-copied property slots, bounded visible non-static `__clone()`
  dispatch on the cloned object, and
  bounded public-property plus context-aware non-public property reference-slot
  mirroring for direct-variable clone assignments, shutdown execution of
  public inherited or declared no-argument `__destruct` methods for allocated
  and cloned objects in reverse allocation order, with class-registration
  validation for non-public, static, or parameterized destructors, plus bounded direct
  object-property array-offset and append reference sources for named visible
  public, private `$this`, protected `$this`, and protected peer-object
  properties in valid method contexts,
  single-parent metadata including namespaced parent names when the parent is
  already declared, object `isset` and `empty`, and selected metadata builtins,
  including declared interface metadata with concrete-class public method
  presence checks and bounded `class_implements()` interface-name arrays for
  current object values or declared string class names, bounded
  `class_uses()` direct trait-name arrays for current object values or
  declared string class names, bounded `class_parents()` parent-chain arrays
  for current object values or declared string class names, bounded
  `ReflectionClass` metadata objects with `getName()`, `getShortName()`,
  `isInterface()`, `isTrait()`, `isInstantiable()`, `getParentClass()`,
  `getInterfaceNames()`, `getTraitNames()`, `getTraits()`,
  `hasMethod($name)`, `getMethod($name)`, `getMethods([$filter])`, class-like
  file/start/end/doc-comment source metadata, `hasProperty($name)`,
  `getProperty($name)`, and zero-argument `getProperties()`, bounded
  `ReflectionFunction` metadata objects for declared user functions and
  current closure values, with by-value `invoke()`/`invokeArgs()` for ordinary
  closures, plus a bounded internal slice for selected WordPress-relevant
  string, path, formatting, and metadata builtins, with name,
  file/start/end/doc-comment, parameter-list, return-type, and
  by-reference-return inspection plus by-value `invoke()`/`invokeArgs()`,
  including the current ASCII-only `str_increment`/`str_decrement` runtime
  subset,
  `ReflectionMethod`
  metadata objects with declaring-class, visibility, static, final, abstract,
  constructor, modifier-mask, class-method file/start/end/doc-comment source
  metadata, parameter-list, return-type inspection, and public non-static
  user-class by-value `invoke()`/`invokeArgs()`, plus static trait-method
  by-value invocation with bounded trait `__CLASS__`, `__METHOD__`,
  `self::class`, `static::class`, `get_called_class()`, and static
  `self::method()`/`static::method()` context, and stable diagnostics for
  abstract reflected method invocation before exact `ReflectionException`
  objects exist,
  bounded
  `ReflectionParameter` function/method-parameter metadata with name, position,
  declaring class/function, optional/default, by-reference, variadic, and
  type-presence predicates plus simple named and bounded compound
  `ReflectionType` metadata through `getType()`, `getReturnType()`,
  `allowsNull()`, `getTypes()`, `getName()`, and `isBuiltin()`, bounded
  `ReflectionProperty` metadata for declared
  user-class properties with declaring-class, visibility/static modifier,
  direct property doc-comment metadata, default-value, simple named typed-property inspection through
  `ReflectionNamedType`, bounded compound property type inspection through
  `ReflectionUnionType`/`ReflectionIntersectionType`, bounded public
  instance/static `getValue()`/`setValue()` mutation with current
  typed-property coercions, and bounded uninitialized typed-property slots for
  properties without explicit defaults, with runtime typed-property writes
  accepting inherited class-name objects and declared user-interface
  implementors in the current object metadata model, declared trait
  metadata for empty traits, public trait constants, supported trait
  properties, simple public instance trait methods, simple class-body
  `use TraitName;` and `use TraitA, TraitB;` composition for already-declared
  traits, simple trait-body `use` declarations
  that compose supported properties, public methods, and constants into classes consuming the outer
  trait, plus simple public trait method alias adaptations such as
  `use TraitName { method as alias; }` and
  `use TraitA, TraitB { TraitA::method as public alias; }`, protected/private
  trait aliases such as `use TraitName { method as protected helper; }`,
  visibility-only trait adaptations such as
  `use TraitName { method as protected; }`, and
  bounded
  public instance conflict resolution such as
  `use TraitA, TraitB { TraitA::method insteadof TraitB; }`, including
  comma-separated loser lists in that same public instance method shape, the
  same-block winning-method public alias interaction, the same bounded
  method-adaptation shapes inside trait-body `use` declarations, and class-declared public
  instance methods taking precedence over same-named composed trait methods or
  aliases. Unresolved same-name public methods from different composed traits
  stop with a stable `phpc run` trait-conflict diagnostic before class
  registration,
  declared unit-enum metadata, bounded `is_countable()`/`count()` for
  `Countable` implementors that pass the current method-shape check, and
  bounded `is_iterable()` metadata for `Iterator`/`IteratorAggregate`
  implementors that pass the current method-shape check
- a documented builtin subset for strings, arrays, constants, filesystem and
  request-state probes, output-buffer probes, type checks, callability checks,
  bounded truthy assertions, object/class metadata, and debug-style output

The runtime still names unsupported zones explicitly. Examples include
references beyond the current direct variable-to-variable assignment cell
slice, copy-on-write, namespace forms beyond the current class-name/import,
same-namespace function, and namespace-scoped top-level constant slices,
including leading-backslash fully-qualified function calls such as `\strlen()`,
leading-backslash fully-qualified constant reads such as `\PHP_VERSION`,
include/require breadth beyond the current narrow local string-path,
include-path, missing-include recovery, and bounded missing-require fatal
statement/expression slice, eval, generators, closure behavior beyond the
current direct `$closure(...)` by-value and direct-variable/direct-array-offset
reference-parameter slice, `call_user_func()` string/closure by-value dispatch
with bounded PHP-matching by-reference-parameter warnings,
`call_user_func_array()` positional by-value, bounded
`call_user_func_array()` closure reference-parameter support, and bounded
`ReflectionFunction::invoke()` slices, explicit and implicit capture binding,
named call arguments, call-time by-reference arguments,
type declaration enforcement, cast behavior outside the current `(string)`,
`(int)`, `(bool)`, and
`(float)`/`(double)` slices plus the null/scalar/array `(array)` slice,
actual PHP warning/notice suppression for `@expr`,
typed/non-public/abstract/final or multi-constant interface
declarations, full interface signature
enforcement, broad built-in/internal interface method enforcement/catalogs
beyond the current `Countable`, `Iterator`, and `IteratorAggregate` shape
checks, non-public/typed/abstract/final/static trait
constants, multi-constant trait declarations, trait constant adaptations,
conflicting trait/class constants, static/abstract/final or non-public trait
methods, executing conflicting trait composition outside class-method
precedence and the bounded `insteadof` shape, exact PHP fatal-error text for
unresolved trait conflicts, aliases
beyond the current simple public, qualified public-alias, same-block
winner public-alias, and protected/private alias slices,
unqualified visibility-only adaptations across multiple used traits,
unqualified `insteadof`, trait property or constant adaptations, `__TRAIT__`,
conditional/nested trait registration, enum case objects/backed
values/methods/interfaces,
catch matching and exception unwinding, exception objects and stack unwinding,
autoload-triggered class discovery beyond string user-function callbacks,
public `"ClassName::method"` static-method strings, public object-method array
callables, public class-string static-method array callables, and public
invokable-object callbacks registered through `spl_autoload_register()` for
`class_exists()`, `interface_exists()`, `trait_exists()`, missing `new` class
instantiation, and included
class/interface/trait declaration dependencies and manual
`spl_autoload_call()` loads; bounded default `spl_autoload()` local file
probing through the current extension registry; autoload lifecycle behavior beyond bounded
`spl_autoload_functions()`, `spl_autoload_unregister()`, and
`spl_autoload_call()`/`spl_autoload()` behavior, including closure invocation,
exact callable validation, scalar-to-string coercions for SPL autoload
extension arguments, warning parity, recursive loader edge cases, and enum
autoloading,
array destructuring beyond positional statement-form `list(...)`/`[...]` with
skipped slots,
constructor behavior beyond public/inherited public instance `__construct`
and explicit parent calls, broader `self::`/`static::` execution beyond the
current method, dynamic static method, class-name, class-constant, and
static-property slices,
exact PHP nested class declaration timing and fatal behavior, real
`Exception` constructor state/methods, `Throwable`, stack traces, exception
throw/catch execution,
bare namespace constant fallback reads, namespace-qualified constant reads,
class-constant lookup through
`defined()`/`constant()` beyond the current declared-class/public-visibility
string-name slice, full extension constant catalogs,
complex double-quoted string interpolation such as array offsets or object
properties, heredoc/nowdoc,
full method signature compatibility beyond the current inherited/interface
required-parameter and same-text type metadata checks, visibility enforcement
beyond the current public and same-declaring-class private-property, protected-property,
protected-method, constructor, method inheritance
visibility/staticness/signature-count/type-text, and class-constant slice,
typed property compatibility beyond exact same-text inherited metadata, weak
scalar coercions, inherited class-name typed-property write checks, declared
user-interface typed-property write checks, current or newly registered
class/interface alias typed-property write checks, and bounded union/pure
intersection property type checks, parenthesized DNF-shaped typed property
declarations, exact PHP union scalar coercion preference rules, readonly
property metadata and write-once enforcement, promoted
constructor properties,
typed or multi-declarator class constants, dynamic method names, dynamic
property creation outside `stdClass` and `wpdb`, non-public dynamic property
access outside valid method visibility contexts,
nullsafe object access `?->`, PHP 8 `match` expressions,
backtick shell execution operators,
magic methods beyond direct missing-property
`__get`/`__isset`/`__set`/`__unset`, missing-method `__call`/`__callStatic`,
direct object-to-string `__toString` including current interpolation, bounded
core interface metadata, broad reflection metadata and exact engine ordering
beyond the current `class_implements()`/`class_uses()`/`class_parents()` and
bounded `ReflectionClass`/`ReflectionFunction`/`ReflectionMethod`/`ReflectionParameter`/
`ReflectionNamedType`/`ReflectionProperty` metadata table slices, interface
and trait method source-file persistence, exact `ReflectionClass::getMethod()`
and `getMethods()` exception objects/text and broad trait-order parity,
reflection invocation beyond the current by-value user function/user-class
method, static trait-method, named bounded internal function, and abstract-method
diagnostic slices, non-public or dynamic
`ReflectionProperty` value mutation, adapted recursive trait metadata edge
cases, and
direct/property-held `ArrayAccess` offsets and
compound assignment/increment/decrement, plus bounded `Countable`
`is_countable()`/`count()` object protocol dispatch with concrete implementor
method-shape checks,
resources, and
clone visibility/destructor behavior beyond the current bounded clone-method
dispatch slice, resources, and native
extension integration.

By-reference assignment syntax has bounded value-model slices for direct
variables, direct array offsets, object-property array offsets, and
string-keyed `$GLOBALS` targets. Covered `$GLOBALS` targets can now join a
source variable that is already routed through covered array-offset alias
metadata, so writes through the global slot and the original slot observe the
same value. These paths are still symbol-table alias metadata rather than full
PHP reference containers; broader alias rebinding, exact mutation ordering, and
copy-on-write remain unsupported.
Covered root-variable `unset($name)` now detaches remaining aliases below a
removed direct array or object variable with their last observed values. Plain
object-property `unset(...)` cleanup now covers visible public properties and
method-context private/protected properties for aliases below the removed
property. Magic-property references, exact alias destruction ordering, and
broad array/object copy-on-write remain unsupported.
By-reference function and method return declarations also parse and have
bounded statement-form reference-assignment execution for direct variable
returns, covered array/property-slot by-reference arguments, and the narrow
`return $param[$key]` and `return $param[$key][$subkey]` child-slot shapes
when `$param` was supplied by a direct variable parent array or a covered
parent array/property slot. Normal by-value invocation of direct free-function,
direct visible object-method, direct named static method, `self::`,
`parent::`, `static::`, and dynamic static receiver reference-return calls can
execute the same direct-variable return shape and the same bounded
`return $param[$key]` / `return $param[$key][$subkey]` child-slot shape as a
by-value read after covered by-reference array-offset argument writeback.
Direct named static method and dynamic static receiver reference-return calls
also use the bounded magic-property array-offset bridge when visible public
`__get()` returns a direct variable by reference, including missing or
inaccessible declared properties reached by that bridge. Arbitrary
reference-return expressions, mixed nested `ArrayAccess` chains, normal
property-read magic fallback breadth, and general magic-property reference
containers remain unsupported.
Omitted optional by-reference parameters can use their defaults without alias
binding; direct-variable by-reference arguments use a bounded direct cell path
for output-parameter style calls. Direct array-offset arguments, including
request-bag paths, and direct visible named object-property array offset
arguments now have bounded output-parameter writeback for user functions,
instance methods, named static method calls, `self::` static method calls, and
`parent::` instance/static method calls, and late-bound `static::` static
method calls. The object-property slice includes public properties and
private/protected properties reached from valid method visibility contexts.
Direct user-function calls additionally accept non-direct holder plain
object-property array-offset arguments such as
`handler($holders["bag"]->items["outer"]["slot"])`, including dynamic selected
visible properties, by evaluating the holder once and writing callee mutations
back through the selected property array slot.
Direct user-function calls also accept direct missing or inaccessible declared
named and dynamic object-property arguments such as
`handler($object->missing)`, `handler($object->private)`, and
`handler($object->{$name})` when public `__get($name)` returns a direct
variable by reference; the parameter binds to that returned cell. Array offsets
below magic properties use the same bounded copy-in/writeback bridge for direct
user-function by-reference parameters and for normal direct free-function or
direct visible object-method reference-return calls that discard the returned
reference. General magic-property reference containers, normal property-read
magic fallback breadth, and arbitrary reference expressions remain unsupported.
Mutations before `unset($param)` are written back; later writes to the
detached local parameter are not.
Direct `call_user_func()` invocation of string user-function callbacks and
ordinary closure callbacks follows PHP's by-reference-parameter behavior for
the reached non-variadic subset: arguments are passed by value, a bounded
`E_WARNING` is emitted through the current error-handler stack or stderr
fallback for reached by-reference parameters, callee writes do not mutate the
caller argument, and closure by-reference direct-variable captures still share
their captured cell. Array-callable `call_user_func()` callbacks, `__invoke`,
exact warning object/text behavior, broader references/copy-on-write, and
native lowering remain unsupported.
`call_user_func_array()` also has a bounded string user-callback, public
object-method callback, and public class-string static-method callback slice
for unkeyed, integer-keyed, or supported string-keyed by-value argument arrays
whose string keys name declared non-variadic parameters. The same callback
forms support string-keyed literal argument arrays containing direct-variable
reference elements such as `array(&$value)`,
`array(10 => &$value)`, and
`array("suffix" => "cache", "value" => &$value)`, plus
direct array-offset reference elements such as
`array(&$_REQUEST["payload"]["slot"])` and direct visible named
object-property array-offset elements such as `array(&$object->items[$key])`
through
copy-in/writeback. Direct stored
argument arrays whose reached by-reference slots were assigned by reference,
such as `$args[0] =& $value; call_user_func_array($callback, $args);`, are
also covered for those same callback shapes, including normal callback
invocation of user functions and public array callables declared as returning
by reference when they mutate reached by-reference parameters. The stored
argument array may be a direct variable or a direct visible named object
property, including private/protected properties reached from a valid method
visibility context, and a direct stored argument-array variable may itself be
backed by covered array-offset alias metadata such as
`$args =& $_REQUEST["callback_args"]` or `$args =& $object->store["args"]`.
Stored callback argument slots may also be assigned by reference from covered
append-offset sources such as `$args[] =& $items[]` and
`$args["value"] =& $object->items[]`; the appended source slot and stored
argument slot share the same bounded alias group for callback writeback and
supported reference-return binding.
Literal reference elements may also use a
direct variable already backed by covered array-offset alias metadata, such as
`array(&$payload)` after `$payload =& $_REQUEST["payload"];`, and
reference-returning callbacks can bind the returned parameter or returned
child slot back to that alias group.
Direct variable, direct array-offset, direct append-offset, nested
append-offset, direct visible object-property, direct visible
object-property append-offset, direct public dynamic-property, and
non-public dynamic-property targets reached from a valid method visibility
context, when assigned from reference array literals such as
`$args = array(&$value)`,
`$registry["args"] = array(&$value)`, and
`$store->args = array("value" => &$object->items[$key])`, plus
`$args[] = array(&$value)`, `$registry["groups"][] = array(&$value)`, and
`$store->groups[] = array(&$items["slot"])`, preserve covered direct
variable, direct array-offset, and direct visible object-property array-offset
reference elements for later stored-array callback invocation and
reference-return alias binding. Append targets record the literal slots below
the actual appended integer key. The same covered reference elements are
preserved when the assigned direct variable is already backed by covered
array-offset alias metadata, such as
`$args =& $registry["args"]; $args = array(&$value)`. Private/protected
dynamic property names use the same context-aware alias root as named
`$this->property` access when the current method context can see that
property. Reference array literals assigned into dynamic properties outside
that documented direct property assignment shape, dynamic-property append
targets, `ArrayAccess`
append targets, or other non-variable targets,
reference elements from arbitrary expressions, executing
unknown or duplicate string-keyed argument names beyond the stable diagnostic
path, positional arguments after string-keyed named arguments, variadic named
callback arguments, stored arrays whose reached slots were not assigned by
reference or by a covered reference array literal, non-direct stored array
expressions beyond direct array offsets and direct
visible named object-property arrays, direct reference assignment between
object-property array offsets without an intermediate alias variable, dynamic callback
object-property array arguments, dynamic static receiver callback
object-property array arguments, dynamic property-held ArrayAccess reference
roots beyond direct visible holder-property sources, ArrayAccess append
reference sources outside the exact `offsetGet(null)` bridge, broader
aliasing, and full copy-on-write remain unsupported.
The current interpreter does include a narrow direct and property-held
ArrayAccess reference
root bridge for `$alias =& $bag[$key]`, nested direct sources such as
`$alias =& $bag["outer"]["slot"]`, property-held sources such as
`$alias =& $holder->bag["outer"]["slot"]`, and literal callback elements such
as `array(&$holder->bag["outer"]["slot"])`. It also covers direct dynamic
property-held sources such as `$alias =& $holder->{$name}["outer"]["slot"]`,
`array(&$holder->{$name}["outer"]["slot"])`, and stored callback slots
assigned from `$holder->{$name}["created"]["leaf"]`. Method-context forms
such as `$alias =& $this->{$name}["slot"]`,
`array(&$this->{$name}["outer"]["slot"])`, and stored callback slots assigned
from `$this->{$name}["created"]["leaf"]` are covered when the selected
private or protected holder property is visible and holds the bounded
`ArrayAccess` object. In these cases public by-reference
`offsetGet($offset)` is exactly `return $this->property[$offset];`. Direct and
property-held append reference sources such as `$alias =& $bag[]` and
`$args[0] =& $holder->bag[]` are covered only for that same exact body shape,
where PHP's `offsetGet(null)` maps to the backing array's empty-string key.
By-reference `foreach` can also consume direct and visible property-held
ArrayAccess offset-array roots such as `foreach ($bag["outer"] as &$value)`
and `foreach ($holder->{$name}["outer"] as &$value)` through that same exact
by-reference `offsetGet()` bridge when the selected backing slot is an array.
Bounded non-direct holder property-held roots such as
`foreach ($holders["bag"]->store["outer"] as &$value)` and
`foreach ($holders["bag"]->{$name}["outer"] as &$value)` are also covered when
the holder expression evaluates once to an object, the selected property is
visible, and that property holds the same bounded `ArrayAccess` object shape.
Real reference containers, magic-property references, arbitrary expressions,
invisible selected properties, mixed nested `ArrayAccess` chains, alias cleanup
outside covered unset root/property/slot paths, broad copy-on-write, exact
alias destruction ordering, broader by-reference `foreach` expansion, native
lowering, and alias lifetime after replacing the containing property remain
outside that bounded slice.
By-reference `foreach` has a bounded copy-back interpreter path for common
array-walk code that unsets the loop variable after the loop. It supports
direct array-offset paths, request-bag paths such as `$_REQUEST["payload"]`,
string-keyed nested `$GLOBALS` paths such as `$GLOBALS["bag"]["child"]`,
visible object-property array roots, and direct/property-held `ArrayAccess`
offset-array roots through the exact `offsetGet()` bridge, with loop-body
mutation, appended tail visitation, and post-loop lingering aliases to
selected array slots. It is not exact PHP aliasing: broad mutation ordering,
object/Traversable iteration, ArrayAccess roots outside the documented bridge,
runtime-backed lvalue-slot handles across all roots, full reference
containers, and broad copy-on-write remain unsupported.

### Native Path

`phpc compile --emit-ir` and `--emit-asm` are intentionally narrower than
`phpc run`.

The current native path is focused on straight-line scalar lowering:

- scalar/null literals, direct scalar assignments, direct reads, `echo`, and
  `print`
- selected `isset` and `empty` folds over the current static variable map
- selected scalar arithmetic, bitwise, comparison, logical, unary, ternary, and
  string-concatenation forms when operands are already lowerable and the result
  semantics are proven
- selected direct builtin folds such as scalar type checks, `strlen`, selected
  callability/function-existence checks, selected metadata-existence checks, and
  selected constant-existence checks

The native runtime ABI has an early helper surface for scalar echo conversion,
owned byte buffers, opaque copied PHP string handles, and a bounded valid-UTF-8
string-handle-to-runtime-value bridge with diagnostic handles for that
conversion's null-handle and non-UTF-8 failure cases. It also exposes the first
nullable native array handle slice: null array handles, allocated empty array
handles, length reads, and handle free. Object, resource, and reference handles
remain null-only opaque shapes for future native storage work. The deterministic
probe includes string-to-value diagnostic message ownership/reporting and the
empty-array handle length/free path. Normal generated LLVM now uses the
string/value ABI for a narrow output path: statement-form `echo` and `print` of
a direct compile-time string value or documented selected string pointer call
runtime string/value helpers, branch on nullable value-handle conversion
failure, report that helper diagnostic to stderr, and otherwise call
`phpc_native_value_echo_stdout`. Dynamic string-pointer expression output beyond
the documented selected-string slices, linked native execution, binary PHP
string value handles, production array lowering beyond this empty-handle ABI
probe, object/resource/reference storage semantics, general diagnostics, and
broad production runtime string-helper lowering are not implemented.

The linked native executable C path handles bounded direct `exit()`/`die()`
termination for materializable no-argument, `null`, `int`, and `string`
operands, plus scoped `if`/`else` branches that do not require persistent
environment merging. It also handles selected known scoped `Class::method`
callable strings for declared public static methods, including by-reference
argument planning through the shared native call-arguments carrier and selected
by-reference method returns. LLVM IR and assembly lowering still reject
`exit()`/`die()`, structured statement control flow, and scoped callable-string
execution.

Native lowering rejects arrays, array destructuring, objects, `instanceof`
relationship checks, static class members, ArrayAccess object-offset dispatch,
clone expressions, user functions, closure values,
include/require, broad control flow, branch environment merging, exception
boundaries, scalar casts, mutation forms that require symbol-table effects,
double-quoted string interpolation, dynamic calls outside the documented
generated-C known string/callable surfaces, `assert()`, runtime constant tables,
direct request superglobal reads such as `$_GET`/`$_POST`/`$_COOKIE`/
`$_REQUEST`/`$_FILES`,
direct `str_starts_with(...)` string-prefix calls,
direct `str_ends_with(...)` string-suffix calls,
direct `basename(...)` lexical path calls,
direct `file_get_contents(...)` filesystem/stream reads,
direct `fopen()`/`stream_context_create()`/`stream_context_get_options()`/
`stream_context_get_params()`/`stream_context_set_params()`/`fwrite()`/
`fscanf()`/`fread()`/`rewind()`/`stream_get_contents()`/`feof()`/`ftell()`/
`fseek()`/`fflush()`/`ftruncate()`/`fstat()`/`stream_get_meta_data()`/`fclose()`/`opendir()`/
`readdir()`/`rewinddir()`/`closedir()` stream-resource and directory-handle
calls,
direct `filesize(...)` local filesystem metadata calls,
direct `filemtime(...)` local filesystem metadata calls,
direct `clearstatcache(...)` stat-cache mutation calls,
direct `realpath_cache_get()`/`realpath_cache_size()` realpath-cache
introspection calls,
direct `getcwd()` current-directory calls,
direct `php_sapi_name()` SAPI identity calls,
direct `ob_start()`/`ob_get_level()`/`ob_get_contents()`/`ob_get_length()`/
`ob_list_handlers()`/`ob_get_status()`/`ob_get_clean()`/`ob_get_flush()`/
`ob_clean()`/`ob_flush()`/`ob_end_clean()`/`ob_end_flush()` output-buffer calls,
direct `header()`/`header_remove()`/`headers_list()`/`headers_sent()`/
`http_response_code()`/`setcookie()`/`setrawcookie()` response header-state calls, including
interpreter-only `headers_sent()` output-started tracking, direct variable,
array-offset, object-property, and object-property array-offset filename/line
outputs including direct variables backed by the current bounded array-offset
reference-alias metadata, bounded ordinary header-name replacement in the
request-local CLI header log, bounded path/domain-aware cookie replacement,
bounded cookie `Max-Age` emission for nonzero expirations, bounded
cookie name validation and options-array key validation,
request-local status-code state, and
bounded post-output `E_WARNING` routing for `header()`, `header_remove()`,
and `setcookie()`/`setrawcookie()`,
direct `realpath(...)` filesystem canonicalization calls,
direct `is_writable(...)` filesystem writability metadata calls,
direct `is_link(...)` filesystem symlink metadata calls,
PHP-wide coercions,
references, copy-on-write, linking, and execution until those semantics exist
in generated code. Statement-form reference assignment has its own native
rejection boundary for direct variable, array-offset, object-property,
function/method/static-call, magic `__get`, and `ArrayAccess` source or target
shapes.

### Tests And Fixtures

Fixture tests live under `tests/fixtures`. The test runner strips one final
newline from `.stdout` and `.stderr` fixtures so expected-output files remain
editor-friendly; use a blank final line when expected program output should
include a trailing newline.

Fixtures with a sibling `.phpc-only` marker are still tested by `phpc`, but are
skipped by optional system PHP comparison when the project intentionally reports
different diagnostics.
When `--compare-php` is used, the summary reports compared fixtures and skipped
fixtures, with skipped fixtures split into missing-`php` and `.phpc-only`
counts.
Use `phpc test --compare-php-json [fixture-dir]` for the same comparison path
as deterministic JSON with `contract_version` 1. It reports aggregate fixture
pass/fail counts plus compared, skipped, missing-system-`php`, and
`.phpc-only` comparison counts. It does not add PHP support, normalize
PHP-version-specific diagnostics, or replace committed fixture expectations.
Use `phpc test --list-fixtures [fixture-dir]` to print a deterministic fixture
manifest without parsing or executing fixtures. The manifest lists each fixture,
its committed expectation files, aggregate expectation/comparison counts, and
whether it is eligible for system PHP comparison. `.phpc-only` fixture entries
also include their marker text as `phpc-only-reason=<reason>`, and the text
manifest reports deterministic source and recognized sidecar byte counts,
including `.cli` snapshot exercise files, for fixtures, summaries, orphan
sidecars, and compatibility targets. Text fixture rows also include SHA-256
digests for fixture sources and present recognized fixture sidecars in
deterministic `source`, `stdout`, `stderr`, `exit`, `cli`, `phpc-only` order,
using `-` for absent sidecars; this is a text-only contract refinement, so the
JSON `contract_version` remains unchanged. The text manifest reports CLI exercise
gap counts for fixtures without `.cli` snapshot sidecars and `.phpc-only`
reason gap counts for markers whose text is empty or whitespace-only. It also
reports aggregate, per-target, and per-fixture missing recognized expectation
sidecars for the `.stdout`, `.stderr`, `.exit`, and `.cli` fixture contract
files without requiring or creating those files. Text orphan and unrecognized
sidecar rows include SHA-256 digests. Text manifests also report unrecognized
sidecar-like siblings whose extension is not part of the fixture contract but
whose corresponding `.php` fixture exists. Compatibility target entries also
report `source-pin.md` path, byte count, and SHA-256 when a target pin file is
present, plus deterministic
`compat/<target>/**/*.expected` probe expectation artifacts with path, byte
count, and SHA-256.
Use `phpc test --list-fixtures-json [fixture-dir]` for the same audit-only
manifest as deterministic JSON with `contract_version` 13. The JSON records
sibling `.phpc-only` marker text as `phpc_only_reason`, source/recognized
sidecar byte counts, recognized orphan sidecar byte counts, CLI exercise gap
counts, missing recognized expectation sidecar metadata for `.stdout`,
`.stderr`, `.exit`, and `.cli`, `.phpc-only` reason gap counts, unrecognized
sidecar counts and byte totals for files with matching `.php` fixtures, and
SHA-256 digests for fixture sources, recognized sidecars, recognized orphan
sidecars, and those
unrecognized sidecar-like siblings so comparison opt-outs and committed
expectation and `.cli` exercise payloads are visible without executing fixtures
or CLI snapshots. When the fixture root contains `compat/<target>` directories,
the JSON also includes per-target compatibility counts, per-target CLI exercise
gap counts, per-target missing recognized expectation sidecar counts,
per-target `.phpc-only` reason gap counts, per-target unrecognized sidecar
counts and byte totals, optional `source-pin.md` audit metadata, and
`.expected` probe expectation artifact metadata, including targets with no
executable `.php` fixtures yet.

Use these commands while developing:

```sh
cargo test --workspace
cargo run -p phpc -- test
cargo run -p phpc -- test --compare-php
```

For the exhaustive support matrix, see `docs/SUPPORT.md`.

## Operations

Operational automation lives in `docs/OPERATIONS.md`.

- `tools/run-tests.sh` runs the full project test suite.
- `tools/checkpoint.sh "message"` runs the suite and commits all current changes
  only if tests pass.
- `tools/codex-loop.sh` runs a bounded Codex supervisor loop when
  `CODEX_RUNNER` is set.
- `tools/codex-yolo-forever.sh` runs an infinite unattended yolo loop with
  durable memory in `docs/LOOP_MEMORY.md`.
