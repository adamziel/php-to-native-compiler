# PHP-to-Native Compiler

This project is an experimental PHP-to-native compiler implemented in stable
Rust. It is intentionally small and honest: implemented features are tested,
unsupported features are documented, and native code generation starts with LLVM
IR text.

The current project has two execution surfaces:

- `phpc run`, an interpreter/runtime path for the supported PHP subset.
- `phpc compile`, a narrower native-code path that emits LLVM IR or assembly for
  straight-line programs and rejects unsupported lowering with structured errors.

For exact support boundaries, read `docs/SUPPORT.md`. For design notes, read
`docs/ARCHITECTURE.md`. For the chronological proof log, read
`docs/PROGRESS.md`.

## Build

```sh
cargo build
```

## CLI

```sh
cargo run -p phpc -- run examples/hello.php
cargo run -p phpc -- compile examples/hello.php --emit-ir
cargo run -p phpc -- compile examples/hello.php --emit-asm
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
seeds: `PHPC_QUERY_STRING`, `PHPC_REQUEST_METHOD`, `PHPC_CONTENT_TYPE`,
`PHPC_REQUEST_BODY`, `PHPC_COOKIE`, and `PHPC_FILES`. These populate bounded
URL-encoded `$_GET`/`$_POST`/`$_REQUEST` data, including bracketed names,
repeated `[]` values, and top-level dotted/spaced request names normalized to
underscores. `PHPC_COOKIE` seeds `$_COOKIE` from a semicolon-delimited cookie
header string and exposes the raw value through `$_SERVER["HTTP_COOKIE"]`;
cookies are not merged into `$_REQUEST`. `PHPC_FILES` seeds explicit
`$_FILES` upload metadata from URL-encoded keys such as
`async-upload[name]=plugin.zip&async-upload[error]=0`; it does not parse
multipart bodies or create temporary upload files. `PHPC_REQUEST_BODY` also
seeds `php://input` for the interpreter only. `session_start()` now
materializes a bounded in-memory `$_SESSION` array for the current CLI request;
direct function-scope reads/writes route through that session root, including
covered nested reference aliases. Session persistence, locking, save handlers,
and cookie emission remain unsupported. `fopen()` can create bounded
interpreter-owned `php://memory`,
`php://temp`, `php://input`, and local UTF-8 file stream resources for simple
flows through `fwrite()`, `fread()`, `rewind()`, `stream_get_contents()`,
`feof()`, `ftell()`, `fseek()`, `fstat()`, `stream_get_meta_data()`, and
`fclose()`. `php://input` handles read the deterministic
`PHPC_REQUEST_BODY` seed and stay non-writable. Bounded
`stream_context_create()` resources store array options for
`stream_context_get_options()`, `stream_context_get_default()` returns a
request-local default context, and `stream_context_set_default()` plus
`stream_context_set_option()` persist string-keyed wrapper options on those
contexts. Context resources may be passed to the current
`file_get_contents()`/`fopen()` local and `php://input` paths without applying
wrapper-specific behavior. `opendir()`, `readdir()`, `rewinddir()`, and
`closedir()` cover bounded local UTF-8 directory handles. Unsupported wrappers,
filters, context option effects, context params, broader wrapper metadata, binary byte
fidelity, directory entry ordering fidelity, permissions/locking, stat-cache
behavior, warning recovery, temp-file spillover, and native stream resources
remain unsupported. Native lowering still rejects request/session/stream state
until a native runtime ABI exists.

### `phpc compile --emit-ir`

`phpc compile <input.php> --emit-ir` emits LLVM IR text for a smaller
straight-line subset. It currently supports scalar literals, direct scalar
variable assignment/readback, scalar `echo`/`print`, selected scalar operators,
selected folds, and a documented set of native builtin folds.

Anything outside that lowerable subset is rejected before misleading IR is
emitted. Arrays, objects, class-name constants, `instanceof` relationship
checks, ArrayAccess object-offset dispatch, clone expressions, include/require
expression return semantics, functions, general control flow,
try/catch/finally exception control, references, copy-on-write, and broad PHP
coercions remain interpreter-only or unsupported for native lowering. Try
blocks are rejected through a dedicated native diagnostic until catch matching,
catch variable binding, finally execution, and stack unwinding have native
semantics.
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
  warning/notice suppression
- `if`, loops, `switch`, `break`/`continue` including positive integer literal
  loop-depth arguments, bounded `goto`/label execution,
  `foreach`, and user functions with local scopes, bounded function-local
  `static` variables, defaults, trailing variadic parameters, returns,
  dynamic string-valued calls,
  bounded function-scope `global $name, ...;` imports for direct variables,
  bounded direct string-keyed `$GLOBALS['name']` root-symbol reads/writes,
  bounded namespace-scoped function declarations and unqualified same-namespace
  calls with global fallback lookup,
  inert no-capture anonymous, static anonymous, and non-static arrow closure
  values,
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
  object-property array roots, plus direct free-function, direct visible
  instance-method, direct named-static-method, method-context
  `self::`/`parent::`/`static::`, dynamic static receiver, and bounded
  `call_user_func_array()` reference-return iterable roots when the returned
  direct variable is backed by a caller variable cell, plus bounded
  direct array-offset by-reference parameter writeback with `unset($param)`
  detachment for ordinary arrays and request bags, plus bounded
  direct free-function, visible object-method, and current static dispatch
  reference-return assignment that binds a returned by-reference parameter
  back to covered direct array-offset and public object-property array-offset
  arguments, including the narrow `return $param[$key]` child-slot shape when
  `$param` was supplied by a covered parent array/property slot,
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
  in caller scope, include return values, and `_once` de-duplication by
  resolved local file
- bounded deterministic `mysqli`/`wp_options` state-island behavior for
  WordPress bootstrap probes, including exact option insert/update/delete/read
  shapes and selected prepared option-value-only, option-name-only,
  option-name-list, full-row, star-projection, name/autoload-list, and
  autoload-list/equality result sets, plus bounded direct and prepared
  transient-shaped option-name prefix result scans and deletes;
  this is not real MySQL connectivity, arbitrary SQL, persistent object cache,
  full `wpdb`, or native database support
- a bounded namespace/class-name/function slice: one unbracketed named `namespace`
  declaration per file, simple top-level class `use` imports with optional
  `as` aliases, namespace-qualified class declarations, class imports for
  class-like references, `new`, `extends`, `instanceof`, static members, and
  `ClassName::class`, plus namespace-scoped function declarations and
  unqualified same-namespace calls
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
  presence checks, declared trait metadata for empty traits, public trait
  constants, and simple public instance trait methods, simple class-body
  `use TraitName;` and `use TraitA, TraitB;` composition for already-declared
  traits, plus simple public trait method alias adaptations such as
  `use TraitName { method as alias; }` and
  `use TraitA, TraitB { TraitA::method as public alias; }`, protected/private
  trait aliases such as `use TraitName { method as protected helper; }`,
  visibility-only trait adaptations such as
  `use TraitName { method as protected; }`, and
  bounded
  public instance conflict resolution such as
  `use TraitA, TraitB { TraitA::method insteadof TraitB; }`, including
  comma-separated loser lists in that same public instance method shape, the
  same-block winning-method public alias interaction, and class-declared public
  instance methods taking precedence over same-named composed trait methods or
  aliases,
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
include/require breadth beyond the current narrow local string-path and
include-path statement/expression slice, eval,
generators, closure invocation, explicit and implicit capture binding,
callback integration, named call arguments, call-time by-reference arguments,
type declaration enforcement, cast behavior outside the current `(string)`,
`(int)`, `(bool)`, and
`(float)`/`(double)` slices plus the null/scalar/array `(array)` slice,
actual PHP warning/notice suppression for `@expr`,
typed/non-public/abstract/final or multi-constant interface
declarations, full interface signature
enforcement, broad built-in/internal interface method enforcement/catalogs
beyond the current `Countable`, `Iterator`, and `IteratorAggregate` shape
checks, trait properties, non-public/typed/abstract/final/static trait
constants, multi-constant trait declarations, trait constant adaptations,
conflicting trait/class constants, static/abstract/final or non-public trait
methods, conflicting trait composition outside class-method precedence and the
bounded `insteadof` shape, aliases
beyond the current simple public, qualified public-alias, same-block
winner public-alias, and protected/private alias slices,
unqualified visibility-only adaptations across multiple used traits,
unqualified `insteadof`, `__TRAIT__`,
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
`spl_autoload_call()` loads; autoload lifecycle behavior beyond bounded
`spl_autoload_functions()`, `spl_autoload_unregister()`, and
`spl_autoload_call()`, including closure invocation, exact callable
validation, and enum autoloading,
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
visibility/staticness/signature-count/type-text, and class-constant slice, typed
property compatibility and DNF-shaped typed property declarations plus
property defaults beyond the current untyped
constant-expression instance property slice, readonly property metadata and
write-once enforcement, promoted constructor properties,
typed or multi-declarator class constants, dynamic method names, dynamic
property creation outside `stdClass`, non-public dynamic property access,
nullsafe object access `?->`, PHP 8 `match` expressions,
backtick shell execution operators,
magic methods beyond direct missing-property
`__get`/`__isset`/`__set`/`__unset`, missing-method `__call`/`__callStatic`,
direct object-to-string `__toString` including current interpolation, bounded
core interface metadata, and direct/property-held `ArrayAccess` offsets and
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
By-reference function and method return declarations also parse and have
bounded statement-form reference-assignment execution for direct variable
returns, covered array/property-slot by-reference arguments, and the narrow
`return $param[$key]` child-slot shape when `$param` was supplied by a covered
parent array/property slot. Normal by-value invocation of reference-return
functions and methods still reports a stable unsupported diagnostic.
Omitted optional by-reference parameters can use their defaults without alias
binding; direct-variable by-reference arguments use a bounded direct cell path
for output-parameter style calls. Direct array-offset arguments, including
request-bag paths, and direct public object-property array offset arguments now
have bounded output-parameter writeback for user functions,
instance methods, named static method calls, `self::` static method calls, and
`parent::` instance/static method calls, and late-bound `static::` static
method calls. Mutations before `unset($param)` are written back; later writes
to the detached local parameter are not.
`call_user_func_array()` also has a bounded string user-callback, public
object-method callback, and public class-string static-method callback slice
for unkeyed or integer-keyed literal argument arrays containing direct-variable
reference elements such as `array(&$value)` and `array(10 => &$value)`, plus
direct array-offset reference elements such as
`array(&$_REQUEST["payload"]["slot"])` and direct public object-property
array-offset elements such as `array(&$object->items[$key])` through
copy-in/writeback. Direct stored
argument arrays whose reached by-reference slots were assigned by reference,
such as `$args[0] =& $value; call_user_func_array($callback, $args);`, are
also covered for those same callback shapes, including normal callback
invocation of user functions and public array callables declared as returning
by reference when they mutate reached by-reference parameters. Reference array
literals stored by value, string-keyed named reference argument arrays,
non-public or dynamic callback object-property array arguments, dynamic static
receiver callback object-property array arguments, using
`call_user_func_array()` itself as a statement-form reference-return source,
broader aliasing, and full copy-on-write remain unsupported.
By-reference `foreach` over a direct array variable has a bounded copy-back
interpreter path for common array-walk code that unsets the loop variable after
the loop. It also supports direct array-offset paths, request-bag paths such as
`$_REQUEST["payload"]`, and string-keyed nested `$GLOBALS` paths such as
`$GLOBALS["bag"]["child"]`, with loop-body mutation, appended tail visitation,
and post-loop lingering aliases to selected array slots. It is not exact PHP
aliasing: broad mutation ordering, object-property/ArrayAccess iterables, array
slot cells, full reference containers, and copy-on-write remain unsupported.

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

Native lowering rejects arrays, array destructuring, objects, `instanceof`
relationship checks, static class members, ArrayAccess object-offset dispatch,
clone expressions,
`exit()`/`die()` termination, user functions, closure values,
include/require, broad control flow, exception boundaries, scalar casts,
mutation forms that require symbol-table effects, double-quoted string
interpolation, dynamic calls, `assert()`, runtime constant tables,
direct request superglobal reads such as `$_GET`/`$_POST`/`$_COOKIE`/
`$_REQUEST`/`$_FILES`,
direct `str_starts_with(...)` string-prefix calls,
direct `str_ends_with(...)` string-suffix calls,
direct `basename(...)` lexical path calls,
direct `file_get_contents(...)` filesystem/stream reads,
direct `fopen()`/`stream_context_create()`/`stream_context_get_options()`/
`fwrite()`/`fread()`/`rewind()`/`stream_get_contents()`/`feof()`/`ftell()`/
`fseek()`/`fstat()`/`stream_get_meta_data()`/`fclose()`/`opendir()`/
`readdir()`/`rewinddir()`/`closedir()` stream-resource and directory-handle
calls,
direct `filesize(...)` local filesystem metadata calls,
direct `filemtime(...)` local filesystem metadata calls,
direct `getcwd()` current-directory calls,
direct `php_sapi_name()` SAPI identity calls,
direct `ob_start()`/`ob_get_level()`/`ob_get_contents()`/`ob_get_clean()`/
`ob_clean()`/`ob_flush()`/`ob_end_clean()`/`ob_end_flush()` output-buffer
calls,
direct `header()`/`header_remove()`/`headers_list()`/`headers_sent()`/
`http_response_code()`/`setcookie()` response header-state calls, including
interpreter-only `headers_sent()` output-started tracking, direct-variable
filename/line outputs, bounded ordinary header-name replacement in the
request-local CLI header log, and bounded request-local status-code state,
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
