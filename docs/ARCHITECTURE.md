# Architecture

## Pipeline

The intended compiler pipeline is:

```text
PHP source
-> lexer/parser
-> AST
-> semantic analysis
-> IR
-> lowering/type specialization
-> LLVM IR text
-> assembly/object/executable
-> linked runtime
```

Milestone 1 implements the lexer, parser, AST, a direct interpreter/runtime
execution path, and a narrow LLVM IR text emitter for simple straight-line code.
The interpreter runs top-level statements in a global symbol table and creates a
fresh local symbol table for each user-function call. Local scopes can import
direct root variables through `global $name, ...;`; imported names route direct
reads and writes through the shared root symbol table, while `unset($name)`
drops the local import without deleting the root value. If the same local name
was previously bound to a local direct array-offset alias, importing the global
name clears that stale local alias binding so later reads/writes use the root
symbol slot. Top-level `global`
declarations preserve existing root values and materialize missing listed names
as `null`, matching the reached WordPress bootstrap initialization shape.
Direct string-keyed `$GLOBALS['name']` reads and writes are a bounded
root-symbol-table route for the reached WordPress object-cache bootstrap
assignment. Direct string-keyed `$GLOBALS['name'] =& $value` may also join the
same bounded alias metadata used by covered direct array-offset references, so
a selected root global, the source variable group, and the covered array slot
observe the same value in the current top-level/root-symbol-table slice. This
same alias metadata also backs the bounded by-reference `foreach` path for
direct nested array-offset roots, request-bag roots such as
`$_REQUEST["payload"]`, and string-keyed nested `$GLOBALS` paths such as
`$GLOBALS["bag"]["child"]`. The same path now covers direct visible
object-property array roots such as `$object->items` and
`$object->items["child"]`, using the existing public/context property alias
root instead of a general PHP reference container. Direct free-function calls
declared as returning by reference can also serve as by-reference `foreach`
iterable roots when the function returns a direct variable backed by a caller
variable cell, such as a by-reference parameter; the interpreter binds a
private temporary root to that returned cell before applying the existing
foreach array-slot alias machinery. This is still a
materialized-symbol-table model, not PHP's full reference-backed alias,
recursive `$GLOBALS` array, copy-on-write, dynamic global-name, ArrayAccess
iteration, dynamic object-property iterable roots, or included-file scope
model.
`$_COOKIE`, `$_GET`, `$_POST`, `$_REQUEST`, and `$_FILES` are seeded in the
same root symbol table and route direct function-scope reads and writes through
that root storage. `$_SESSION` is materialized lazily into the same root symbol
table when the bounded CLI `session_start()` path succeeds and then routes
direct function-scope reads and writes through that same root storage,
including the current bounded direct nested array-offset alias metadata for
covered `$_SESSION["..."]` reference slots. The default request bags are empty
ordered arrays. For
explicit CLI request exercises, `PHPC_QUERY_STRING` seeds flat URL-encoded
and bracketed URL-encoded query pairs into `$_GET`, and
`PHPC_REQUEST_METHOD=POST` plus
`PHPC_CONTENT_TYPE=application/x-www-form-urlencoded` and `PHPC_REQUEST_BODY`
seeds flat and bracketed URL-encoded body pairs into `$_POST`. `PHPC_COOKIE`
is treated as an explicit semicolon-delimited cookie header seed for
`$_COOKIE` and `$_SERVER["HTTP_COOKIE"]`. `PHPC_FILES` is treated as an
explicit URL-encoded upload metadata seed for `$_FILES` using PHP-shaped keys
such as `async-upload[name]`, `async-upload[tmp_name]`,
`async-upload[error]`, and `async-upload[size]`; `error` and `size` leaf
values parse as decimal integers when possible. The same initial seed records
bounded upload provenance for `tmp_name` entries whose sibling `error` value
is `0`; `is_uploaded_file()` checks that request-local provenance and
`move_uploaded_file()` moves a registered local path once before clearing the
source provenance. Repeated scalar keys use
last-write-wins, `[]` bracket segments append in order, and keyed bracket
segments materialize nested arrays under the current ordered-array model. Dots
and spaces in top-level request names are normalized to underscores before
insertion, while bracket segment keys keep their current literal decoded text.
`$_REQUEST` starts as GET merged with POST at the top-level key. This is
deterministic request-state scaffolding only: it does not import browser
cookie headers from the host SAPI, merge cookies into `$_REQUEST`, handle
malformed or edge-case PHP request names with exact `parse_str()` behavior,
parse multipart uploads, create temporary upload files, validate host upload
state beyond the explicit seed, model failed-upload provenance, lock session
files, dispatch session save handlers, emit session cache headers, or model
`variables_order`/`request_order`. The only cross-process session persistence
path is the bounded `session.save_path`/`session_id()` file slice documented
below.

Array-offset references in the interpreter are currently represented as
symbol-table alias metadata, not general runtime reference containers. A direct
variable may route to one or more direct array-offset aliases; this lets the
current runtime mirror the bounded PHP behavior where copying a direct array
or declared public object-property array that contains a referenced direct slot
preserves that slot's reference identity across a copied direct array. Copying
a literal-key direct nested array path, including auto-global request paths
such as `$_REQUEST["payload"]`, also mirrors covered reference slots below the
copied path into the destination direct array variable. Variable, dynamic,
append, and side-effecting copied path keys remain outside this bounded mirror
and still require the future reference/COW value model. When an
array-offset or visible named object-property array-offset reference target is
bound from a direct variable that already shares a direct variable-to-variable
cell, the interpreter rewires every direct name in that small source cell
group to the selected slot alias. By-reference user-function, instance-method,
named static method, `self::` static method, and late-bound `static::` static
method parameters can also accept a direct visible named object-property
array-offset argument through a narrower output-parameter bridge: the caller
slot is materialized and copied into the callee parameter, then the final
parameter value is written back to the visible property slot after normal
return. Public slots use public alias metadata; private/protected slots use
context-aware alias metadata when reached from a valid method visibility
context. That bridge intentionally does not provide in-call
reference-container identity.
Direct object-variable clone assignments also mirror public object-property
alias metadata and context-aware non-public
object-property alias metadata from the cloned source variable to the target
variable, so the current bounded reference-slot model can keep covered property
slots shared across a fresh object handle for the covered `clone $object`
assignment shape.
Reference-returning `call_user_func_array()` sources use the same caller-cell
binding path as direct reference-returning function and method calls for the
current literal argument-array direct-variable and direct array-slot reference
element slice. Direct reference-returning free-function, visible
object-method, named-static-method, method-context `self::`/`parent::`/
`static::`, and dynamic-static-receiver assignment now also use the
array-offset writeback/alias result path when a reached by-reference parameter
is supplied by a direct array-offset or visible named object-property
array-offset argument and the function or method returns that parameter
directly. The same
assignment-only path can bind a returned direct array-offset expression such
as `return $param[$key];` or `return $param[$key][$subkey];` when `$param` is
one of those copied-in covered parent array-slot arguments or a direct caller
variable bound through a by-reference parameter. For copied-in slots, after
the local parent array is written back, the returned key suffix is appended to
the caller alias group. For direct caller variables, the returned explicit key
suffix is mapped to that caller variable's child slot. If the direct caller
variable is itself backed by the bounded array-offset alias metadata, the
callee parameter now receives the alias group instead of looking for a normal
symbol-table cell, so returned child-slot suffixes remain attached to the
underlying request/global/array/property slot. Literal callback argument
arrays can also bind direct, named property-held, and direct dynamic
property-held `ArrayAccess` elements,
including method-context `$this->{$name}` roots for visible private or
protected holder properties and nested offset elements, when public by-reference
`offsetGet($offset)` has the exact bounded `return $this->property[$offset];`
shape; the interpreter maps that to the backing property array alias root plus
any nested child-key suffix instead of creating a real reference container.
Append-offset `ArrayAccess` reference sources such as `$bag[]` and
`$holder->bag[]` use the same bridge and model PHP's `offsetGet(null)` call as
the backing property array's empty-string key for that exact body shape. It
does not make callback argument arrays, non-public object-property
array bridges, dynamic ArrayAccess roots beyond direct dynamic property-held
sources, non-direct holder expressions, mixed nested `ArrayAccess` chains,
magic-property references, arbitrary append ArrayAccess bodies,
or stored array-offset metadata into general runtime reference
containers. By-reference
`foreach` currently consumes direct free-function, direct visible
instance-method, direct named-static-method, method-context
`self::`/`parent::`/`static::`, dynamic static receiver, and bounded
`call_user_func_array()` reference-return iterable roots from this machinery,
including bounded direct caller-cell and direct static-local cell cases.
Property-return, array-offset-return beyond that assignment-only covered
parent-slot suffix shape, expression-return, magic `__callStatic`, and callback
forms outside the bounded `call_user_func_array()` slice remain outside the
executable foreach slice.
Whole-variable assignment to a direct array root and whole-property assignment
to a declared visible object-property root drop stale aliases for that root
before the replacement value is observed by future copies. Reassigning the
direct object variable also drops stale public and context-aware non-public
object-property roots for that object name. Direct dynamic-property assignment
from a reference array literal uses the evaluated property name as the public
or context-aware non-public object-property alias root, so later stored-array
callback use can reuse covered reference elements when the current method
context can see the property. Direct array-offset `unset(...)` paths and
direct visible object-property array-offset `unset(...)` paths remove covered
alias metadata for the removed slot, and for child aliases below a removed
parent slot, while storing the last observed alias value back into the
detached direct alias variable. This models the bounded PHP behavior where
unsetting a referenced container slot deletes the container entry without
deleting the remaining reference variable. Arbitrary nested copied reference
slots beyond the literal copied path slice, non-public property-offset or
magic clone alias mirroring, dynamic ArrayAccess references, arbitrary
ArrayAccess append bodies, reference array literals outside direct variable,
direct array-offset, direct visible object-property, and direct
dynamic-property assignment targets, alias cleanup outside covered unset slot
paths, exact alias destruction ordering, and native lowering still require the
future runtime reference/COW value model.

## Compiler Crate

`compiler/` contains:

- lexer and parser for the supported PHP subset
- AST definitions
- interpreter bridge for `phpc run`
- LLVM IR text emission for currently lowerable code
- CLI and fixture test runner

The `phpc compile` CLI validates its requested output mode before reading or
parsing the input file. That keeps usage mistakes such as `--emit-object`
reported as CLI diagnostics even when the input path is missing or the source
would otherwise fail earlier in the compiler pipeline.

The parser is handwritten recursive descent. This keeps the early grammar easy
to audit while avoiding regex-based parsing. Unsupported syntax boundaries use
stable diagnostics before AST construction when accepting the syntax would
imply runtime or native semantics that do not exist yet. Statement-form
`list($a, $b) = expr;` now has an AST/runtime path for direct variable targets
and skipped positional slots. Statement-form short `[$a, $b] = expr;`
destructuring reuses that same AST/runtime path for direct variable targets,
while expression-position `list(...)`, keyed/nested/reference targets,
`foreach` destructuring, and non-variable targets remain parser boundaries.
Named call arguments stop at a dedicated parse boundary before AST
construction because correct support needs parameter-name metadata, duplicate
and unknown-name diagnostics, positional/named ordering, by-reference binding,
variadic collection, unpacking interaction, and native lowering.

The lexer maintains both character and byte offsets as it advances. Prefix
checks for PHP tags, heredoc terminators, and other byte-slice comparisons use
the maintained byte offset so large real-world files do not repeatedly rescan
the already-consumed character prefix.
When the lexer sees `?>`, it emits the intervening inline HTML up to the next
PHP open tag as a token consumed by the parser into an echo statement. The
current slice consumes one immediate newline after `?>`, matching PHP's common
close-tag newline behavior for the covered fixtures. Short echo open tags
such as `<?= $value ?>` stop in lexing with a dedicated unsupported diagnostic
until the lexer/parser can expand them into echo statements with correct source
mapping, inline-HTML interaction, and native rejection behavior.
Backtick shell execution expressions such as `` `whoami` `` also stop in
lexing with a dedicated unsupported diagnostic until command-string
interpolation, process execution, stdout capture, platform error behavior,
references/copy-on-write, and native lowering have an explicit implementation.

Assignment targets are intentionally narrower than expression reads. Direct
variables, direct array offsets, direct append offsets, direct object
properties, selected static properties, direct-variable nested array offset
paths, direct-variable append-at-depth paths, and direct-object-property
nested/append-at-depth array paths have explicit AST targets. The nested array
targets keep a variable or object-property root plus evaluated index
expressions so the interpreter can materialize missing array containers under
the current no-reference/no-copy-on-write model. Direct object-property
array-offset compound assignment reuses the object-property root plus evaluated
index path for the current read-modify-write slice. Direct object-property
array-offset `isset(...)` reuses the same visible property plus evaluated index
path for the current presence-check slice. A visible direct object property
whose value is an `ArrayAccess` object can dispatch single-key read/write,
`isset`, `empty`, `??`, and `unset` operations through that contained object;
deeper mixed object/property/ArrayAccess chains and other nested
read-modify-write forms remain explicit boundaries.
Unset targets follow the same conservative pattern: direct variables,
direct/nested array offsets, selected static-property diagnostics, and
direct-object-property nested array offsets have explicit targets, while plain
object-property uninitialization and mixed target paths remain separate
boundaries.
PHP 8 nullsafe object access `?->` is tokenized so the parser can stop with a
specific unsupported syntax diagnostic before creating any AST node. Faithful
support still needs null-aware property/method chain evaluation,
short-circuiting, call argument ordering, assignment-target restrictions, and
native lowering.
PHP 8 `match` expressions remain a parser boundary rather than an AST node
until the expression model has strict arm matching, default/exhaustiveness
handling, throw-arm behavior, value evaluation ordering, reference/COW
interactions, and native lowering.
Parenthesized DNF-shaped type declarations such as `(A&B)|C` are also kept at
a parse boundary for parameters, return types, and typed properties until the
type metadata model can represent those shapes without implying runtime
enforcement.
Top-level trait declarations are parsed as metadata for empty traits and
simple public instance methods. A class body may use already-declared traits
with `use TraitName;`, repeated simple trait-use declarations, or one simple
comma-separated declaration such as `use TraitA, TraitB;`; the interpreter
composes those trait public instance methods onto the consuming class metadata
and stores the executable method bodies under the consuming class id, so
ordinary instance method dispatch works through `phpc run`. A narrow trait
method alias adaptation such as `use TraitName { method as alias; }` clones
the composed public instance method under the alias name while leaving the
original method available. The same alias path accepts an explicit `public`
marker with a same-use qualified trait target, such as
`use TraitA, TraitB { TraitA::method as public alias; }`, while still treating
the alias as an ordinary public method. Alias adaptations may also mark the new
alias `protected` or `private`; the original public trait method remains
available, and the non-public alias uses the existing method visibility checks
for dispatch, `method_exists()`, and `get_class_methods()`. Visibility-only
adaptations such as `use TraitName { method as protected; }` and
`use TraitName { method as private; }` change the original composed public
instance trait method visibility without creating an alias. A bounded trait
conflict adaptation such as
`use TraitA, TraitB { TraitA::method insteadof TraitB; }` is accepted for
public instance methods from traits in the same class-body `use`
declaration; composition registers the winner and skips the named loser method.
The same current slice accepts comma-separated loser lists such as
`TraitA::method insteadof TraitB, TraitC` when every loser trait appears in
the same class-body `use` declaration.
The current executable interaction slice also allows that selected winning
method to be exposed through a same-block explicit-public alias, such as
`use TraitA, TraitB { TraitA::method insteadof TraitB; TraitA::method as public alias; }`.
When a consuming class declares a public instance method with the same name as
a composed public trait method or alias, the class method takes precedence and
the trait method is skipped in the effective class method table. This lets a
concrete class override trait fallback methods while still satisfying current
interface method checks.
Public trait constants declared as `const NAME = ...` or
`public const NAME = ...` with the current class-constant expression subset are
composed into consuming classes and resolve through the existing
`ClassName::CONST`, `self::CONST`, and `static::CONST` paths. Trait
properties, non-public/typed/abstract/final/static trait constants,
multi-constant trait declarations, trait constant adaptations, conflicting
trait/class constants, static/abstract/final or non-public trait methods,
broad conflict resolution beyond class-method precedence and the current
bounded `insteadof` slice, unqualified visibility-only adaptations across
multiple used traits, unqualified `insteadof`,
qualified or multi-trait alias edge cases beyond the current winner-alias slice,
`__TRAIT__` context, references/copy-on-write, nested or conditional trait
declarations, and native trait lowering remain explicit boundaries.

Top-level interface declarations are parsed as metadata for public method
signatures and public constants. The current inheritance slice accepts one or
more user parent interfaces declared before or after the child interface, such
as `interface Child extends Parent, OtherParent`, and flattens those parent
names into concrete class `implements` relationship metadata. Class registration
enforces public method presence for child and parent interface methods,
including methods supplied by the current public trait composition and alias
subset. Interface registration also checks child-interface redeclarations of
inherited methods and simple multi-parent method conflicts with the current
bounded metadata rules: a redeclaration may not require more parameters than
the inherited method, may not add a parameter type where the parent method is
untyped, must keep compatible parameter types when both sides are typed, and
must keep compatible typed parent return declarations. Compatible typed
relationships include exact text case-insensitively plus simple declared
class/interface contravariant parameter and covariant return relationships when
both type names resolve through current metadata. Public interface constants declared
as `const NAME = ...` or
`public const NAME = ...` use the current class-constant expression subset and
resolve through interface names, parent-interface inheritance, and
implementing-class class-constant lookup. Missing or cyclic parent interface
inheritance remains a stable runtime boundary.
typed/static/non-public/abstract/final or multi-constant interface
declarations, full variance/signature enforcement, namespace-aware type-name
resolution, union/intersection canonicalization, built-in/internal interface inheritance catalogs, exact PHP
diagnostics, and native lowering remain explicit boundaries.

Double-quoted string interpolation is represented explicitly in the AST for
the current simple `$name`, `{$name}`, array-offset, object-property, and
chained access slices instead of being rewritten to ordinary string
concatenation. The runtime evaluates those parts left to right through the
active symbol table and PHP-shaped echo-string conversion. Native lowering
rejects ordinary interpolated strings with a dedicated codegen diagnostic until
native interpolation part evaluation, PHP-shaped string conversion,
array/object lookup, `__toString` dispatch, runtime string allocation,
references/copy-on-write, and exact native diagnostics exist. Interpolated
`defined("SODIUM_$constant")` names continue to use the global-constant native
boundary until native constant tables and runtime string lookup semantics
exist. `${...}`, dynamic properties, static properties, and arbitrary complex
expressions remain lexer boundaries.

PHP error-control syntax is represented as an explicit AST wrapper for
`@expr`. The interpreter currently evaluates the wrapped expression normally
and deliberately does not suppress diagnostics, because the runtime still
models undefined variables, unsupported calls, invalid arithmetic, and other
conditions as fatal project diagnostics rather than recoverable PHP
warnings/notices. Native lowering rejects the wrapper until generated code has
a real diagnostic severity and suppression model.

Loop-control statements record an explicit positive integer depth in the AST.
The interpreter represents `break N;` and `continue N;` as control-flow signals
whose depth is consumed one active `while`, `for`, `do ... while`, `foreach`,
or `switch` level at a time. Plain `continue;` that targets a switch remains a
runtime boundary, while `continue 2;` can pass through the switch to an outer
loop. Native lowering rejects all current structured control flow.
`for` headers store initializer, condition, and increment slots as ordered
lists. The interpreter executes initializer and increment actions left to
right; condition expressions also evaluate left to right and the final
expression's truthiness controls the loop.

Direct `exit()`/`die()` calls are treated as a bounded language-construct
termination signal rather than as ordinary callable builtins. The interpreter
preserves stdout accumulated so far and returns the current exit code for the
top-level CLI result. The construct is intentionally absent from
`function_exists()`/`is_callable()` lookup. Shutdown functions, destructors,
finally-during-exit ordering, output buffers, SAPI interaction, and native
termination lowering remain future runtime work. Native `--emit-ir` and
`--emit-asm` reject `exit()`/`die()` through a dedicated termination diagnostic
instead of the generic function-call boundary.

Cast expressions are represented as `Expr::Cast` with a small `CastKind`
covering `(string)`, `(int)/(integer)`, `(bool)/(boolean)`, and
`(float)/(double)`, and `(array)`. The interpreter owns PHP-shaped conversion
for the current scalar/null/array subset and keeps warning-producing or
object/resource-heavy cast behavior as runtime or parse boundaries. Native
lowering rejects all cast expressions with a dedicated diagnostic until
generated code has scalar conversion, array materialization, warning/recovery,
object/resource handling, and exact diagnostic behavior.

## Runtime Crate

`runtime/` contains the PHP-shaped boxed value model used by the interpreter and
future generated code helper calls.

Implemented now:

- `Null`
- `Bool`
- `Int`
- `Float`
- `String`
- ordered PHP arrays with integer/string keys, including a bounded numeric
  key-sort helper for the reached `ksort(..., SORT_NUMERIC)` path and ordered
  strict identity comparison over the current key/value model
- class metadata, object-shape descriptors, and minimal object values for
  `new ClassName(...)` over declared classes, process-local object handles,
  public instance properties including inherited public slots, inherited
  non-public instance slots with declaring-class ownership, single-parent
  metadata, execution-time registration for reached nested class declarations,
  metadata-only core `Exception`, `stdClass`, `PDO`, and `PDOStatement` class
  seeds,
  inherited method lookup, public/same-class private/protected same-class and
  child instance method dispatch, and public/inherited public instance
  `__construct` plus explicit parent/self method dispatch with scoped `$this`,
  parsed abstract/final class modifiers plus abstract/final method modifiers
  as metadata, with abstract class instantiation, final-parent inheritance,
  final method overrides, method visibility reductions, concrete classes with
  unimplemented abstract methods rejected at runtime, and inherited method
  static/non-static plus bounded non-constructor signature compatibility
  enforced at runtime, including required-parameter counts, parameter type
  metadata, and return type metadata,
  bounded `new self`/`new parent`/`new static` class-name resolution in active
  class contexts, and bounded direct-variable dynamic class-name instantiation
  for `new $class(...)`; missing named or direct-variable string class names
  invoke the current bounded autoload callbacks before the class table is
  rechecked; included class/interface declarations also use that callback path
  to load missing `extends` parents, direct `implements` interfaces, direct
  class-body trait uses, and parent interfaces before final registration
  validation,
  bounded dynamic property-name reads/writes for existing public slots,
  `stdClass` public dynamic slots, and the WordPress `wpdb` compatibility
  class's dynamic table-name slots, and bounded `clone` expressions that
  allocate fresh handles, shallow-copy current property slots, and dispatch
  visible non-static `__clone()` methods on cloned objects
- structured runtime error categories with stable diagnostic messages for the
  currently supported runtime failures
- PHP-ish echo conversion
- PHP-ish truthiness for the implemented value types
- basic arithmetic, comparison, and concatenation helpers
- key normalization for array strings that are valid decimal integers
- private `ArrayEntry` slot storage with value accessors (`value()`,
  `value_cloned()`, `value_mut()`, `set_value()`, and `into_value()`) that
  delegate through an explicit clone-by-value `ArraySlot`. `ArraySlot` owns a
  private `ArraySlotCell` through a shared handle, and normal slot cloning
  still allocates a fresh cell with a cloned value. Each cell has a stable
  internal `ArraySlotCellId`, but slot/cell equality remains value-based so
  current PHP array equality and clone behavior do not depend on storage
  identity. A private internal primitive can intentionally share a slot cell
  for future reference work, while current public by-value writes detach before
  mutation instead of exposing PHP-visible aliasing. This preserves the
  current eager array value semantics while isolating the storage object that
  future slot/reference-cell work must replace;
  `PhpArray::get_slot()` and `get_slot_mut()` expose normalized-key slot
  lookup without introducing aliasing

Planned runtime values and semantics:

- resources
- references
- copy-on-write containers

By-reference assignment has bounded execution slices for statement-form direct
variable sources. The interpreter symbol table stores direct variables as
mutable cells, so `$alias =& $value;` can bind both direct names to the same
cell in the current scope/global-routing model. Assignment through either name
updates the other, and `unset($alias)` or `unset($value)` removes only that
name binding while the cell remains alive through any remaining alias. Direct
array-offset reference sources now have narrow execution paths:
`$alias =& $array[$key];` and `$alias =& $array[$outer][$inner];` over a
direct array variable bind the target direct variable name to the selected
normalized-key array slot route. Missing keys and missing intermediate
containers on an existing array root are materialized before binding, and
undefined or `null` direct source roots are materialized as arrays containing
the selected `null` slot. Direct object-property array-offset reference sources
have a similarly narrow direct-variable target path:
`$alias =& $object->items[$key];` and
`$alias =& $object->items[$outer][$inner];` can bind the alias variable to an
explicit offset path inside a named visible property on a direct object
variable, including private `$this` and protected `$this`/peer-object roots
inside valid method visibility contexts. The route materializes a `null`
property as an array and missing selected slots as `null`. Writes through the
alias and through the direct array or supported object-property offset observe
the same value, and `unset($alias)` removes only the alias binding.
Append-at-depth reference sources have similarly bounded routes for direct
arrays and named visible object-property roots, including private `$this`,
protected `$this`, and protected peer-object roots when the source appears
inside a valid method visibility context. Dynamic public object-property
reference sources have similarly bounded routes for direct object variables.
Named visible object-property sources can bind through the current
public/private/protected method context. Missing direct object
properties can also dispatch to a visible non-static magic `__get()`
reference source when the method is declared by reference and returns a direct
variable through the existing reference-return method path; the alias target
binds to that returned variable cell rather than to a property slot. Dynamic
missing public-property names use the same magic route after the property
expression resolves to a string or integer. Non-array roots, non-direct object
expressions, inaccessible private/protected magic fallback fidelity, `__get()`
returns of properties/offsets/expressions, dynamic non-public magic-property
behavior, dynamic non-public append-source paths, `ArrayAccess` offsets,
object-property by-reference `foreach` iterables, exact broad by-reference
`foreach`, full reference containers, copy-on-write, and native lowering remain
future work. Direct variable sources
holding object values can also be assigned into direct array offsets under the
existing object-handle value model. Direct free-function
call sources have a narrow reference-return execution path when the function
declares `&`, returns a direct variable, and is used as a statement-form
reference-assignment source; the target name is bound to the returned variable
cell. Direct object method-call sources have the same narrow path for visible
non-static methods on object receivers. Direct named static method-call sources
have the same narrow path for visible static methods on direct class-name
receivers, and `self::` static method-call sources have the same path in active
class/method context. `parent::` static method-call sources have the same path
from active child class/method context when the resolved inherited method is
static. `static::` late-static method-call sources have the same path from
active class/method context when the called-class context resolves a visible
static method. Dynamic static receiver method-call sources have the same path
when the receiver evaluates to an object or class string and resolves a visible
static method. Missing static methods that would dispatch through magic
`__callStatic` are kept as an explicit reference-return boundary with a stable
unsupported-call diagnostic; magic reference-return dispatch is not modeled.
Direct array-offset reference targets also have narrow execution paths:
`$array[$key] =& $value;` works when the target root is a direct array
variable, the offset is explicit, and the source is a covered direct source
name: unaliased, part of a direct variable-to-variable alias group, or already
routed through covered array-offset alias metadata. `$array[] =& $value;`
works for the same direct root/source shape by appending through the runtime
array append cursor and routing the source group to the selected auto key.
`$array[$outer][$inner] =& $value;` works for explicit nested direct-array
targets by routing the source group to a normalized key path under the direct
array root. `$array[$outer][] =& $value;` uses the same key-path route after
appending through the runtime array append cursor at the selected nested
parent. Direct public object-property
array-offset targets have a similarly bounded route:
`$object->items[$key] =& $value;` can bind a covered direct source variable to
an explicit offset inside a declared public property on a direct object
variable. `$object->items[] =& $value;` and
`$object->groups[$key][] =& $value;` append through the same public-property
root route and bind the source name to the selected property-array auto key.
That route reads and writes the root through the existing public
object-property access path and materializes `null` property values as arrays
before selecting or appending the offset. The interpreter materializes missing
explicit keys, missing intermediate containers for the direct-array and
tested object-property target slices, and undefined or `null` direct-array
target roots through the direct-offset and nested-array materialization paths,
copies the current source value into the selected slot, then routes the source
name to that slot. Writes through either the source variable or the supported
direct array/object-property offset observe the same selected value, and
`unset($value)` detaches only the source name. When the covered public
object-property array is copied into a direct static variable, alias metadata
for referenced property slots is mirrored onto the copied static array so
writes through the source variable, the property slot, or the copied slot
synchronize for the covered key path. Undefined source variables start as
`null` before binding. PHP's deprecated false-root conversion, other non-array
roots, dynamic/magic/non-public property targets, non-direct sources,
non-static
`self::`/`parent::`/`static::`/dynamic-static sources, magic method reference
sources, full PHP reference containers, broader by-reference
`foreach` fidelity, mutation-ordering guarantees, alias rebinding, native
lowering, and copy-on-write remain future runtime work. ArrayAccess object
offset targets such as `$bag[$key] =& $value;` and property-held
`$holder->bag[$key] =& $value;` are an explicit runtime boundary with a stable
diagnostic, reflecting PHP's fatal behavior for assigning by reference to an
object array dimension while avoiding engine-specific notice/fatal text.
Direct array-offset reference sources mirror a subset of those same alias
routes for direct variable targets. `$alias =& $array[$key];` and
`$alias =& $array[$outer][$inner];` bind the target name to the selected
direct-array slot, materializing missing containers and selected slots as
needed. `$alias =& $array[];` and `$alias =& $array[$outer][];` append a
`null` slot through the runtime append cursor and bind the target name to that
selected slot. Direct public object-property sources such as
`$alias =& $object->property;` bind the target name to the selected property
root when that property is visible through the current public/private/protected
method context. Public property roots, private `$this->property` roots in the
declaring class, protected `$this->property` roots in visible class contexts,
and protected peer-object roots such as `$alias =& $other->property;` from a
valid child method context all reuse context-carrying object-property alias
metadata. Whole-property writes preserve those root aliases, while property
array-offset aliases detach to the previous property array when the whole
property is replaced. Dynamic public object-property sources such as
`$alias =& $object->$property;` reuse that same root route when the target
object is a direct variable, the property expression evaluates to a string or
integer name, and the selected property is public; allowed dynamic-property
objects such as `stdClass` materialize a missing selected property as `null`
before binding. Dynamic whole-property writes also detach narrower
property-array aliases before replacing the selected public property value.
Direct visible named object-property array-offset sources
follow the same bounded root route for named visible properties:
`$alias =& $object->items[$key];`,
`$alias =& $object->items[$outer][$inner];`,
`$alias =& $object->items[];`, and
`$alias =& $object->items[$outer][];`. Public properties use public alias
metadata, while private/protected properties use context-aware alias metadata
from valid method visibility contexts. String-keyed `$GLOBALS` append
reference sources such as `$alias =& $GLOBALS["bag"][];` and
`$alias =& $GLOBALS["bag"]["outer"][];` bind a direct alias variable to the
selected slot under the real global symbol table. `$GLOBALS[]` append
sources, non-string root keys, recursive `$GLOBALS` materialization,
dynamic-property sources on non-direct object expressions, dynamic
non-public property sources, magic property sources, non-variable reference
targets, full reference containers, copy-on-write, exact alias destruction
ordering, and native lowering remain future work. Direct `ArrayAccess`
reference sources now have a narrow root bridge for `$alias =& $bag[$key]`,
property-held roots such as `$alias =& $holder->bag[$key]`, direct dynamic
property-held roots such as `$alias =& $holder->{$name}[$key]`, and literal
callback elements such as `array(&$holder->{$name}[$key])` when the direct
object variable or visible selected property value implements `ArrayAccess`, public
`offsetGet($offset)` returns by reference, and the method body is exactly the
current `return $this->property[$offset];` shape. Property-held roots are
parked in a hidden object-handle symbol and then reuse the same backing
property array alias metadata, including private/protected properties through
the declaring method context. By-value `offsetGet()`, dynamic property-held
ArrayAccess sources on non-direct holder expressions or outside visible
property access, alias lifetime after replacing the containing property,
side-effecting or broader `offsetGet()` bodies, mixed nested ArrayAccess
chains, append sources, and real reference containers remain future work.
String-keyed `$GLOBALS` reference targets also have narrow routes:
`$GLOBALS["name"] =& $value;`, `$GLOBALS["bag"]["slot"] =& $value;`, and
`$GLOBALS["list"][] =& $value;` bind the selected root global symbol or
root-global array slot to a direct source variable cell, including from
function scope. The source may be unaliased, part of a direct
variable-to-variable alias group, or already routed through covered
array-offset alias metadata; for the array-offset source shape, the selected
`$GLOBALS` slot is merged into the same bounded alias group. Missing root
globals, `null` root globals, and missing intermediate containers materialize
as arrays for the nested/append forms, and append targets use the runtime
array append cursor under the selected root global array. Writes through the
source name, the direct global variable path, the supported string-keyed
`$GLOBALS` offset path, and other covered alias-group slots observe the same
value, and unsetting the source name detaches only that name. Nested by-value
writes through supported string-keyed `$GLOBALS` paths route through the root
global symbol table and sync covered aliases. Non-string root keys,
`$GLOBALS[] =& $value`, non-direct sources, recursive `$GLOBALS`
materialization, full reference containers, copy-on-write, exact mutation
ordering, and native lowering remain future work.
By-reference function and method return declarations are represented as
function metadata so declaration-contained code can register. Normal invocation
of reference-return functions and methods still reports stable runtime
boundaries before any by-value return is produced.
By-reference parameters are also metadata-first: omitted optional
by-reference parameters can use their defaults as ordinary local values, while
provided direct-variable by-reference arguments bind the callee parameter name
to the caller's variable cell for the duration of current user-function,
instance-method, constructor, named static method, `self::` static method, and
`parent::` instance/static method calls, and late-bound `static::` static
method calls. Writes through the parameter are visible before the call returns,
and `unset($param)` detaches only the callee's local name from the shared cell.
Direct array-offset arguments, including request-bag paths such as
`$_REQUEST["payload"]["slot"]`, and direct visible named object-property
array-offset arguments use a narrower copy-in/writeback bridge on the
user-function, instance-method, named static method, `self::` static method,
`parent::` instance/static method, and late-bound `static::` static method
dispatch paths.
The bridge keeps the local parameter's original cell so mutations before
`unset($param)` are written back at return, while later writes to a detached
local name are not. It still does not expose in-call PHP reference-container
identity.
`call_user_func_array()` reuses that same
direct cell binding for narrow string user-callbacks, public object-method
array callbacks, and public class-string static-method array callbacks where
the argument array is an unkeyed or integer-keyed literal containing
`&$directVariable` elements for reached by-reference parameters. The same
callback path can also copy in and write back direct array-offset elements,
including request/global bag paths such as `&$_REQUEST["payload"]["slot"]`
and `&$GLOBALS["bag"]["slot"]`, plus direct visible named object-property
array-offset elements such as `&$object->items[$group][$key]`, using the
bounded output-parameter bridge rather than in-call reference-container
identity. Visible named object-property callback roots use public alias
metadata for public slots and context-aware alias metadata for
private/protected slots reached from a valid method visibility context.
Literal reference elements whose direct variable is already backed
by covered array-offset alias metadata, such as
`array(&$payload, ...)` after `$payload =& $_REQUEST["payload"];`, reuse the
same alias group for copy-in/writeback instead of requiring a normal caller
variable cell. Statement-form reference assignment from a reference-returning
`call_user_func_array()` source can now return the same bounded direct
array-offset or visible named object-property array-offset route when the
callback returns that reached parameter, so the assigned alias is represented
in symbol-table alias metadata instead of as a general runtime reference
container. If the callback returns a child slot below that alias-backed
literal direct-variable parameter, the returned suffix is appended to the
underlying alias group. Direct stored argument
arrays can also satisfy reached by-reference
parameters when the selected slots were previously assigned by reference
through the covered direct array-offset target path. That path finds the
stored slot in the existing alias metadata, copies the current slot value into
the callee parameter, and writes the final parameter value back through the
same alias, which also syncs covered direct-variable, copied-array,
request-bag/global, public object-property, and context-aware non-public
object-property alias groups. For
statement-form reference assignment, a callback that returns that reached
parameter by reference now binds the assigned alias back to the same covered
alias group rather than only to the stored argument array slot. Normal
`call_user_func_array()` invocation can use the same copy-in/writeback bridge
when the callback function or public array-callable method is declared as
returning by reference, but the callback call itself still returns a value.
The stored argument array may be a direct variable or a direct visible named
object property, including private/protected properties reached from a valid
method visibility context. A stored argument-array variable may itself be
routed through covered array-offset alias metadata, including ordinary array
roots, request/global roots, and visible object-property array roots; slot
writes and stored-slot lookups compose the argument-array root alias with the
selected integer key before binding or writing back.
The stored callback path also has a narrow anonymous alias-group route for
reference assignments from covered append-offset sources into stored argument
slots, such as `$args[] =& $items[]` or
`$args["value"] =& $object->items[]`. The append source materializes a new
`null` slot, the stored argument target materializes its selected slot, and
both aliases are recorded together so later callback writeback and supported
reference-return binding sync the source slot and stored argument slot without
introducing a general runtime reference container.
For the bounded `call_user_func_array()` reference path, string-keyed
argument-array entries can also bind by declared parameter name for current
user-function callbacks and public object/static array-callable methods. The
literal path maps direct variable, direct array-offset, and visible named
object-property array-offset reference elements to the named parameter, while
the stored-array path looks up the existing alias metadata by the same string
key before copy-in/writeback or reference-return alias binding.
The by-value callback path now uses the same parameter-name mapping for
current user functions and public object/static array-callable methods when
the argument array contains string keys and no reached by-reference parameter
requires alias binding. That path is still an interpreter argument-mapping
layer over materialized values: duplicate names, unknown names, positional
entries after named entries, and variadic named arguments stop with stable
runtime diagnostics instead of pretending a general PHP named-argument model
exists.
This deliberately does not model full PHP reference containers, reference
array literals stored by value, stored arrays whose reached slots were not
assigned by reference, non-direct stored array expressions beyond direct
visible named object-property arrays, direct reference assignment between
object-property array offsets without an intermediate alias variable,
execution past unknown or duplicate string-keyed callback argument names,
positional arguments after a string-keyed named argument, variadic named
callback arguments, dynamic key expressions in the literal reference
named-argument path,
dynamic ArrayAccess reference roots, append-offset ArrayAccess source roots
outside the exact direct/property-held `offsetGet(null)` bridge, stored-array
ArrayAccess roots outside the current direct/property-held `offsetGet()`
reference bridge, non-public property roots outside the current valid
method-context named-property slice, dynamic static receiver callback
object-property array arguments, broader reference-return binding, exact
by-reference `foreach`, or copy-on-write.
By-reference `foreach` value syntax over a direct array variable has a bounded
interpreter path. Each iteration reads the active entry from the current
ordered array, writes the key variable by value, routes the value variable to
the active direct array slot for the body, and advances using the current array
order instead of an initial key snapshot. Appended elements and newly inserted
tail entries are visited by the same loop, and direct writes to the current
slot are visible through the loop variable. If the active direct array slot is
unset during the body, the value variable is detached onto the removed value;
a same-key reinsertion in that body does not retarget the value variable until
a later iteration reaches the reinserted tail entry. After loop completion, the
value variable remains routed to the last successfully iterated existing slot
until `unset($value)` detaches it. Empty array iteration creates no lingering
route. Temporary array-producing expressions such as array literals and direct
non-reference-returning function calls use the same direct-slot loop machinery
after the evaluated array is stored in an internal hidden array slot; this
preserves loop-body and post-loop value-variable aliasing to the temporary
without claiming mutation of a source lvalue. Direct string-keyed
`$GLOBALS["name"]` roots use the same alias metadata with a global-array root,
so the loop value can mutate and linger on slots under the real root symbol
table from top-level or function scope. This intentionally avoids claiming
full mutation-during-iteration fidelity, broad array reordering/replacement
semantics, nested lvalue iterable support such as `$items[0]` or
`$GLOBALS["name"]["child"]`, non-string-keyed `$GLOBALS` roots,
reference-returning call iterables, object/`Traversable` iteration,
destructuring targets,
array/object/`ArrayAccess` offset loop variables, nested-offset loop values,
full reference containers, or PHP copy-on-write.
- dynamic method/property names, broader visibility enforcement for
  non-public properties/constructors, static methods and broader static member
  semantics, magic methods beyond the current direct missing-property
  `__get`/`__isset`/`__set`/`__unset`, missing-method `__call`/`__callStatic`,
  direct object-to-string `__toString` including current interpolation,
  bounded core interface metadata, and direct/property-held `ArrayAccess`
  offset and compound-assignment/increment-decrement slices,
  typed/default property compatibility, broader
  `parent::`/`self::`/`static::`, broader
  inheritance and constructor semantics, exact nested declaration timing, and
  exact PHP object lifecycle behavior

The first native-runtime ABI prerequisite lives in
`docs/NATIVE_RUNTIME_ABI.md`. It exposes a C-compatible scalar handoff type for
`null`, booleans, integers, and floats, plus exported constructor symbols in
`php_runtime`. This is intentionally only an ABI seed for future generated-code
runtime helper calls. The compiler-side scalar echo helper probe renders
`usize`-shaped helper signatures from an explicit pointer-width target so the
ABI sketch can distinguish 32-bit and 64-bit targets. Linked native execution,
runtime helper calls from normal generated IR, strings, arrays, objects,
references, copy-on-write, stack frames, and diagnostics are still not
implemented.

## Native Codegen

The first backend emits LLVM IR text and shells out to `clang` for assembly.
This is deliberately less work than an x86-64 backend and lets the project focus
on PHP semantics first.

Tradeoff: Milestone 1 native lowering is smaller than interpreter support. The
backend must return a codegen error for unsupported constructs rather than
pretend to compile them.
Expression-form `include`, `include_once`, `require`, and `require_once` have
a dedicated native rejection boundary, separate from the statement-form
multi-file boundary. This keeps the extra expression semantics visible:
included-file return values, `_once` de-duplication result values,
caller-scope side effects, source loading, path resolution, declaration
registration, source mapping, and exact diagnostics all remain future native
work.
Statement-form reference assignment has a dedicated native rejection boundary,
separate from general mutation lowering. `Stmt::ReferenceAssign` is rejected
before lowering its source or target operands in both LLVM IR emission and the
C assembly fallback path, including direct variable, array-offset,
object-property, function-call, method-call, static-call, magic `__get`, and
`ArrayAccess`-shaped sources or targets. Real support depends on native
reference containers, alias-aware symbol tables, copy-on-write, object/property
alias roots, and exact diagnostic behavior.
Object-property `ArrayAccess` offset shapes also have a dedicated native
rejection boundary for reads, writes, `isset`, `empty`, `unset`, and compound
paths. This prevents property-held object offsets that `phpc run` can dispatch
through `offsetGet`, `offsetSet`, `offsetExists`, and `offsetUnset` from being
reported as only generic array or object lowering gaps. Native execution still
requires object handles, ArrayAccess method dispatch, reference/COW semantics,
and exact PHP diagnostics. Direct `$value[$key]` remains on the generic array
boundary at codegen time unless later analysis proves `$value` is an
`ArrayAccess` object.
`instanceof` expressions have a dedicated native rejection boundary for the
current class/interface relationship checks. Both LLVM IR emission and the C
assembly fallback path reject the `instanceof` AST node before lowering the
left operand, so missing native class metadata tables, object handles,
inheritance/interface registries, class-name resolution, autoload interaction,
references/copy-on-write, and exact native diagnostics stay visible instead of
collapsing into the broader object/class boundary.
Method-call expressions also have a dedicated native rejection boundary for
instance calls, named static calls, object/static-receiver calls, `self::`,
`parent::`, and late-static `static::` calls. Both LLVM IR emission and the C
assembly fallback path reject the method-call AST node before lowering the
receiver or arguments, so missing native method lookup, receiver/static
receiver resolution, `$this` and late-static-binding context, argument/arity
diagnostics, visibility checks, references/copy-on-write, and exact native
method-call errors stay visible instead of collapsing into the broader
object/class diagnostic.
Class-name constants have a dedicated native rejection boundary for
`ClassName::class`, `self::class`, `parent::class`, and `static::class`. LLVM
IR emission rejects those AST nodes before lowering class-name resolution, so
missing native class-name resolution, active class/parent and
late-static-binding context, namespace/import canonicalization, autoload-free
class lookup interaction, references/copy-on-write, and exact native
class-name constant diagnostics stay visible instead of collapsing into the
broader static-member diagnostic.
Static class members have a dedicated native rejection boundary for class
constants, static property reads/writes, and dynamic static-property
receivers. Both LLVM IR emission and the C assembly fallback path reject those
AST nodes before lowering class/member operands, so missing native class
constant tables, static property storage, class context, late-static-binding
resolution, visibility checks, autoload/class lookup, references/copy-on-write,
and exact native static-member errors stay visible instead of collapsing into
the broader object/class diagnostic.

Current assembly emission order:

1. Generate LLVM IR text.
2. Use `clang` if available.
3. Use `llc` if available.
4. If no LLVM assembly tool is available, generate equivalent C for the same
   narrow lowerable subset and ask `cc -S` for assembly.

The C fallback exists only to keep `phpc compile --emit-asm` executable on
machines without LLVM tools. It must not grow into the primary backend without a
documented architecture decision.
Assembly CLI coverage intentionally checks a normalized success summary for the
current straight-line scalar echo/assignment subset instead of committing exact
assembly text, because `clang`, `llc`, and the `cc -S` fallback produce
platform- and toolchain-specific output.
Assembly rejection CLI coverage also runs a representative unsupported array
program with backend tools removed from `PATH`; this pins that LLVM lowering
diagnostics are returned before backend discovery or invocation.
A separate backend-absence CLI snapshot runs a lowerable scalar program with
backend tools removed from `PATH`; this pins the missing-backend diagnostic
after LLVM lowering has succeeded.
The C fallback path has its own CLI snapshot that hides `clang` and `llc` while
exposing only `cc`; it checks normalized success properties instead of exact
assembly text.
A selected-backend failure snapshot exposes a deterministic fake `clang` that
passes discovery and then exits nonzero after accepting generated LLVM IR. That
pins the CLI diagnostic shape for backend failure without committing real
toolchain stderr.
A selected-`llc` snapshot exposes only a deterministic fake `llc` while hiding
`clang` and `cc`; it pins backend selection order with normalized assembly
properties instead of backend-specific assembly text.
A selected-`llc` failure snapshot exposes only a deterministic fake `llc` that
passes discovery and then exits nonzero after accepting generated LLVM IR. That
pins the CLI diagnostic shape for `llc` failure without committing real
toolchain stderr.
A C fallback failure snapshot exposes only a deterministic fake `cc` that passes
discovery and then exits nonzero after accepting generated C fallback source.
That pins the CLI diagnostic shape for `cc -S` fallback failure without
committing real toolchain stderr.
A discovery-exhaustion snapshot exposes deterministic fake `clang`, `llc`, and
`cc` commands whose `--version` probes all fail. That pins the missing-backend
diagnostic when candidate command names exist on `PATH` but no backend passes
discovery.
An empty-stderr backend-failure snapshot exposes a deterministic fake `clang`
that passes discovery and exits nonzero without stderr after accepting
generated LLVM IR. That pins the fallback diagnostic detail for selected
backend failures when a tool provides no error text.
An empty-stdout backend-success snapshot exposes a deterministic fake `clang`
that passes discovery and exits successfully after accepting generated LLVM IR
without producing assembly text. That pins the selected-backend diagnostic for
empty stdout instead of treating an empty assembly artifact as success.
A success-with-stderr backend snapshot exposes a deterministic fake `clang`
that passes discovery, emits nonempty assembly stdout, writes a diagnostic to
stderr, and exits successfully. That pins the current boundary: successful
backend stderr is intentionally ignored by `phpc`, while assembly is taken only
from stdout.
Success-with-stderr fallback snapshots expose deterministic fake `llc` and
`cc` tools with the same successful behavior, proving the shared success path
after LLVM fallback selection and after the `cc -S` C fallback selection.
Empty-stderr fallback failure snapshots expose deterministic fake `llc` and
`cc` tools that exit nonzero without diagnostics. That pins the shared
`backend exited without stderr` diagnostic detail after LLVM fallback selection
and after the `cc -S` C fallback selection.
Empty-stdout fallback success snapshots expose deterministic fake `llc` and
`cc` tools that exit successfully without assembly text. That pins the shared
empty-output diagnostic after LLVM fallback selection and after the `cc -S` C
fallback selection.
Whitespace-only fallback success snapshots expose deterministic fake `llc` and
`cc` tools that exit successfully with only whitespace on stdout. That pins the
shared whitespace-only-output diagnostic after LLVM fallback selection and
after the `cc -S` C fallback selection.
A selected-backend whitespace-only success snapshot exposes a deterministic
fake `clang` that exits successfully with only whitespace on stdout. That pins
the same shared whitespace-only-output diagnostic before fallback selection.
A selected-backend whitespace-with-stderr success snapshot exposes a
deterministic fake `clang` that exits successfully with only whitespace on
stdout while writing stderr diagnostics. That pins stdout validation as the
reported failure and keeps successful-backend stderr unsurfaced even when the
successful stdout artifact is invalid.
A selected-backend whitespace-with-stderr precedence snapshot exposes the same
invalid successful `clang` output while `llc` and `cc` are also available. That
pins the no-recovery boundary after invalid selected-backend output.
A selected-backend empty-stdout-with-stderr precedence snapshot exposes invalid
successful `clang` output with no assembly stdout and stderr diagnostics while
`llc` and `cc` are also available. That pins stdout validation as the reported
failure and keeps fallback recovery disabled for empty selected-backend
artifacts with stderr.
An `llc` whitespace-with-stderr precedence snapshot exposes invalid successful
`llc` output while the `cc -S` fallback is also available and `clang` is
unavailable. That pins the no-recovery boundary after invalid selected `llc`
output.
An `llc` empty-stdout precedence snapshot exposes invalid successful selected
`llc` output with no assembly stdout while the `cc -S` fallback is also
available. That pins the same no-recovery boundary for empty selected-`llc`
artifacts.
An `llc` empty-stdout-with-stderr precedence snapshot covers the same boundary
when selected `llc` writes stderr diagnostics but emits no assembly stdout
while `cc` is available.
Whitespace-with-stderr fallback snapshots expose deterministic fake `llc` and
`cc` tools with the same invalid successful-output behavior. That pins the same
stdout-validation precedence after LLVM fallback selection and after the
`cc -S` C fallback selection.
A selected-backend input-validation snapshot exposes a deterministic fake
`clang` that validates representative generated LLVM IR markers on stdin before
emitting normalized assembly. That pins the selected-backend stdin handoff for
the current lowerable scalar subset without treating the test double as
backend-specific IR validation.
Fallback input-validation snapshots expose deterministic fake `llc` and `cc`
tools that validate representative generated LLVM IR or generated C fallback
markers on stdin before emitting normalized assembly. That pins the same stdin
handoff after LLVM backend fallback selection and after the `cc -S` C fallback
selection without treating the test doubles as backend-specific IR/C
validation.
Backend argument-validation snapshots expose deterministic fake `clang`,
`llc`, and `cc` tools that validate the exact selected-backend and fallback
assembly emission argument vectors before accepting stdin and emitting
normalized assembly. That pins the current command-line contract without
treating it as full backend-specific CLI compatibility.
Backend discovery probe argument-validation snapshots expose deterministic fake
`clang`, `llc`, and `cc` tools that require an exact single-argument
`--version` probe before selected or fallback assembly emission. That pins the
current discovery probe contract without treating it as full backend-specific
discovery semantics.
Backend discovery probe output snapshots expose deterministic fake `clang`,
`llc`, and `cc` tools whose successful `--version` probes write stdout and
stderr diagnostics before selected or fallback assembly emission. That pins
the current boundary that probe output is ignored after a successful probe,
without treating that as full backend-specific discovery output semantics.
Failed backend discovery probe output snapshots expose deterministic fake
`clang`, `llc`, and `cc` tools whose failed `--version` probes write stdout and
stderr diagnostics before fallback selection or missing-backend reporting.
That pins the current boundary that failed-probe output is ignored and failed
probes still behave like unavailable tools, without treating that as full
backend-specific failed-probe output semantics.
Backend discovery probe start-failure snapshots expose deterministic fake
`clang`, `llc`, and `cc` command names that exist on `PATH` but cannot be
started for `--version`. That pins the current boundary that probe start
failures are treated as unavailable before fallback selection or
missing-backend reporting, without treating that as full backend-specific
discovery semantics.
Backend discovery probe permission-denied snapshots expose deterministic fake
`clang`, `llc`, and `cc` command names that exist on `PATH` but are not
executable for `--version`. That pins the current boundary that permission
denied probe starts are treated as unavailable before fallback selection or
missing-backend reporting, without treating that as full backend-specific
discovery semantics.
A selected-backend start-failure snapshot exposes a deterministic fake `clang`
that passes discovery and then rewrites itself to use a missing interpreter
before assembly emission. That pins the current diagnostic for a race-like
case where a discovered command cannot be started later, without treating that
as full backend race-condition recovery semantics.
A selected-backend permission-denied emission snapshot exposes a deterministic
fake `clang` that passes discovery and then removes its own execute permission
before assembly emission. That pins the same selected-backend start diagnostic
for permission-denied starts after discovery, without treating that as full
backend race-condition recovery semantics.
Fallback start-failure snapshots expose deterministic fake `llc` and `cc`
tools with the same race-like behavior. Those pin the current diagnostics for
previously discovered fallback commands that cannot be started later, without
treating that as full backend race-condition recovery semantics.
Fallback permission-denied emission snapshots expose deterministic fake `llc`
and `cc` tools that pass discovery and then remove their own execute
permission before assembly emission. Those pin the same fallback backend start
diagnostics for permission-denied starts after discovery, including the
current no-recovery boundary where a selected `llc` start failure is reported
without falling through to the `cc -S` C fallback.
A mixed scalar output snapshot uses deterministic fake `clang` coverage for a
lowerable straight-line program with both `echo` and `print`. A matching C
fallback snapshot hides LLVM assembly tools and uses a deterministic fake `cc`
that validates generated C fallback markers for that same static scalar output
boundary. These snapshots do not claim runtime-backed output conversion,
linking, execution, or broader native lowering.
A scalar reassignment assembly snapshot uses a deterministic fake `clang` that
validates generated LLVM IR for a straight-line program where later direct
static-variable assignments overwrite earlier lowerable scalar values before
output. This pins the current per-lowering-pass overwrite behavior without
claiming native symbol-table storage, references/copy-on-write behavior,
linking, execution, exact native PHP errors, or broader native lowering.
A matching C fallback reassignment snapshot hides LLVM assembly tools and uses
a deterministic fake `cc` that validates generated C fallback source for the
same straight-line overwrite boundary.
A matching `--emit-ir` reassignment snapshot commits the exact generated LLVM
IR and shows only the final overwritten scalar values are emitted.
A selected-`clang` type-introspection snapshot validates that the already
implemented native `is_callable(...)` false-folding IR is passed to the chosen
backend through stdin without broadening callable dispatch or native execution.
A selected-`clang` `function_exists($name)` snapshot validates the same stdin
handoff for already-implemented direct function-existence folding without
broadening runtime lookup, callable dispatch, or native execution.
A selected-`clang` `empty($name)` snapshot validates the same stdin handoff for
already-implemented direct-variable emptiness folding without broadening
symbol-table semantics, unset interactions, or native execution.
A selected-`clang` `isset($name)` snapshot validates the same stdin handoff for
already-implemented direct-variable existence folding without broadening
symbol-table semantics, unset interactions, or native execution.
A selected-`clang` `is_numeric($value)` snapshot validates the same stdin
handoff for already-implemented deterministic scalar/string numeric folding
without broadening runtime lookup, string coercion, or native execution.
A selected-`clang` `is_countable($value)`/`is_iterable($value)` snapshot
validates the same stdin handoff for already-implemented scalar/null
false-folding without broadening native array/object lowering or runtime-backed
type checks.
A selected-`clang` `is_object($value)`/`get_debug_type($value)` snapshot
validates the same stdin handoff for already-implemented scalar/null folding
without broadening native object lowering, object handles, or runtime-backed
type checks.
A selected-`clang` static metadata-exists snapshot validates the same stdin
handoff for already-implemented absent-class/interface/trait/enum false-folding
without adding native class metadata, autoloading, or object execution.
A selected-`clang` `strlen($value)` snapshot validates the same stdin handoff
for already-implemented known-string length folding without broadening string
coercion, dynamic calls, runtime lookup, or native execution.
A native `defined($name)` snapshot commits folded `--emit-ir` output and
normalized C fallback `--emit-asm` output for direct supported string names,
without adding native constant values, source-order definitions, runtime
constant tables, or native execution.
The Milestone 569 native `defined($name)` snapshot adds the current
`SORT_REGULAR` built-in constant to that static answer table without changing
the broader native constant-table or dynamic-call boundaries.
The Milestone 573 native `defined($name)` snapshot adds `SORT_NUMERIC` to the
same static answer table with folded `--emit-ir` and normalized C fallback
`--emit-asm` coverage while preserving those broader boundaries.
A selected-`clang` `defined("SORT_REGULAR")` snapshot validates that the same
folded LLVM IR reaches the chosen backend through stdin without changing
production lowering behavior.
A selected-`clang` `defined("SORT_NUMERIC")` snapshot validates the same
backend stdin handoff for the existing folded built-in constant answer without
changing production lowering behavior.
A selected-`clang` `defined("SORT_STRING")` snapshot validates the same
backend stdin handoff for the existing folded built-in constant answer without
changing production lowering behavior.
A selected-`clang` broader `defined($name)` constants snapshot validates the
same backend stdin handoff for the current exact built-in constant answer table
without changing production lowering behavior.
A backend-precedence snapshot exposes deterministic fake `clang`, `llc`, and
`cc` commands together. It pins the current selection order by proving
successful `clang` assembly emission is used before LLVM or C fallback tools
when all candidates are available, without treating that as full
backend-specific discovery semantics.
A fallback-precedence snapshot hides `clang` while exposing deterministic fake
`llc` and `cc` commands together. It pins the current fallback selection order
by proving successful `llc` assembly emission is used before the `cc -S` C
fallback when both fallback candidates are available, without treating that as
full backend-specific discovery semantics.
A selected-backend failure-precedence snapshot exposes deterministic fake
`clang`, `llc`, and `cc` commands together. It pins the current no-fallback
failure boundary by proving a selected `clang` emission failure is reported
directly instead of silently falling through to `llc` or `cc`, without treating
that as full backend recovery semantics.
A fallback failure-precedence snapshot hides `clang` while exposing
deterministic fake `llc` and `cc` commands together. It pins the same
no-recovery boundary after fallback selection by proving a selected `llc`
emission failure is reported directly instead of silently falling through to
the `cc -S` C fallback, without treating that as full backend recovery
semantics.
An empty-stderr fallback failure-precedence snapshot covers the same boundary
when selected `llc` exits nonzero without diagnostics while `cc` is also
available. It pins the stable empty-stderr `llc` diagnostic as final rather
than falling through to the `cc -S` C fallback.
An empty-stderr selected-backend failure-precedence snapshot covers the same
no-recovery boundary when selected `clang` exits nonzero without diagnostics
while `llc` and `cc` are also available. It pins the stable empty-stderr
`clang` diagnostic as final rather than falling through to fallback tools.
A selected-backend start-failure-precedence snapshot covers the same
no-recovery boundary when selected `clang` passes discovery but cannot be
started while `llc` and `cc` are also available. It pins the stable
selected-backend start diagnostic as final rather than falling through to
fallback tools.
A fallback start-failure-precedence snapshot covers the same no-recovery
boundary after fallback selection when `clang` is unavailable and selected
`llc` passes discovery but cannot be started while `cc` is also available. It
pins the stable `llc` start diagnostic as final rather than falling through to
the `cc -S` C fallback.

Current native lowering accepts same-type `null`, boolean, integer, finite
float, known ASCII nonnumeric NUL-free string loose/ordering comparisons, and
identical string-pointer self-comparisons only after both operands are
lowerable by the current straight-line scalar subset. Other
loose, ordering, mixed-type, array, object, and coercive comparison cases
still reject at the comparison boundary.
That keeps generated code from implying PHP comparison coercions, array/object
comparison behavior, untracked `NAN`/`INF` edge cases, or exact native error
objects that only the interpreter path currently handles or diagnoses.
The remaining lowerable native subset is intentionally tiny: literal scalar
values, direct static-variable assignments from those values, direct reads of
previously assigned static variables, later direct static-variable assignments
overwriting earlier lowerable scalar values in the same straight-line lowering
pass, lowerable same-type integer or same-type float `+`, `-`, and `*`
expressions, lowerable same-type `null`, boolean, integer, finite float, known
ASCII nonnumeric NUL-free string loose/ordering comparisons, identical string
pointer self-comparisons, empty-string concatenation identity for lowerable
string operands, direct supported-string-name `defined($name)` checks against
the current exact built-in constant-name set, and `echo`/`print` through
static `printf` calls.
It does not
model PHP zvals, symbol-table storage, PHP numeric coercion,
references/copy-on-write, dynamic string allocation, locale/version-specific
float formatting, integer overflow promotion, assembly linking/execution, or
native PHP error objects.
Finite same-type float arithmetic and finite float unary-minus results are
bounded and tracked only for later scalar folds such as strict identity when
all possible results are proven. Float overflow, `INF`, and `NAN`
result-tracking edges remain out of scope for this value-set tracker.
A dedicated mixed `echo`/`print` assembly CLI snapshot pins that boundary with
a deterministic fake backend and a lowerable scalar fixture; it is coverage of
the existing static output path, not broader native runtime output support.
Native reads of variables that have not been statically assigned earlier in
that same straight-line lowerer are rejected with a specific codegen
diagnostic until native symbol-table storage, undefined-variable diagnostics,
references/copy-on-write behavior, and exact native error behavior exist.
Native comparison lowering accepts same-type `null`, boolean, integer, finite
float, known ASCII nonnumeric NUL-free string loose/ordering comparisons, and
identical string-pointer self-comparisons for `==`, `!=`, `<`, `<=`, `>`, and
`>=`, plus strict identity `===` and `!==` when both operands are already
lowerable `null`, integers, booleans, floats, or strings in the straight-line
subset. Static
same-type scalar identity folds at compile time. Bounded integer, float,
string, and boolean identity fold when all
possible `===`/`!==` outcomes are proven identical. Identical lowerable
dynamic scalar operands fold for integers, booleans, already-lowerable string
pointers, and finite tracked floats, so `$x === $x` and `$x !== $x` avoid
runtime comparisons in those safe scalar cases. Identical lowerable integer
operands also fold for loose/ordering comparisons, including intentionally
untracked integer expressions such as overflow-sensitive shift results:
`$x == $x`, `$x <= $x`, and `$x >= $x` fold true, while `$x != $x`, `$x < $x`,
and `$x > $x` fold false. Dynamic boolean expression
operands compared with boolean literals fold for `$flag === true`, `true ===
$flag`, `$flag !== false`, and `false !== $flag` by reusing the original
native boolean expression, and inverse forms such as `$flag === false`, `false
=== $flag`, `$flag !== true`, and `true !== $flag` use the native boolean
inversion path. Dynamic boolean expression operands compared loosely with
boolean literals fold for `$flag == true`, `true == $flag`, `$flag != false`,
and `false != $flag` by reusing the native boolean expression, while inverse
forms such as `$flag == false`, `false == $flag`, `$flag != true`, and
`true != $flag` use the native boolean inversion path. Dynamic boolean
expression operands ordered against boolean literals also fold within boolean
semantics, reusing the expression, inverting it, or folding to a static boolean
for cases such as `$flag > false`, `$flag < true`, `$flag <= true`, and
`true >= $flag`. Same-type integer and finite-float loose/ordering
comparisons whose tracked possible operands prove one result fold to a static
boolean. Literal-only comparisons still fold, while ambiguous tracked
finite-float comparisons stay emitted as native comparisons.
Boolean expression comparisons whose tracked possible operands prove one
loose/ordering result also fold to that static boolean without emitting a
redundant native boolean comparison. Identical native boolean expression
operands also fold for loose/ordering comparisons, including ambiguous boolean
expressions: `$flag == $flag`, `$flag <= $flag`, and `$flag >= $flag` fold
true, while `$flag != $flag`, `$flag < $flag`, and `$flag > $flag` fold false.
Other ambiguous boolean expression comparisons stay emitted. Identical native
string pointer operands also fold for loose/ordering comparisons, including
untracked string pointer expressions whose possible value set exceeds the
current small tracker: `$text == $text`, `$text <= $text`, and `$text >=
$text` fold true, while `$text != $text`, `$text < $text`, and `$text >
$text` fold false. Non-identical unknown string comparisons stay rejected.
Statically known integer
strict-identity comparison results remain tracked for later boolean scalar
lowering even when the comparison itself stays emitted as `icmp`. Same-type
ambiguous dynamic integer and boolean identity lower to `icmp`, same-type
ambiguous dynamic float identity lowers to `fcmp`, and same-type ambiguous
already-lowerable string pointer identity lowers through `strcmp`. Those
dynamic scalar comparisons use PHP-shaped boolean echo output. Already
lowerable mixed scalar operands with different PHP scalar types fold without
runtime comparison calls, including dynamic integer, boolean, float, or string
pointer expression results when the opposite operand has a different static
scalar type. Known ASCII nonnumeric string loose/ordering comparisons lower
to a static boolean when every possible safe string outcome matches; ambiguous
safe string loose/ordering comparisons lower through `strcmp`. Statically
known boolean, integer, and finite-float loose/ordering comparison results
remain tracked for later boolean scalar lowering even when the comparison
itself stays emitted as `icmp`/`fcmp`; ambiguous bounded boolean,
finite-float, or string loose/ordering comparison results remain dynamic and
untracked.
It rejects ambiguous bounded integer, float, string, or boolean
identity, broader value-correlation proofs across related expressions such as
`$x` and `!$x`,
numeric-looking, non-identical unknown, non-ASCII, or NUL-containing string loose/ordering comparisons,
mixed null or other mixed-type comparisons, untracked or
non-finite float comparisons, dynamic null identity beyond static/type-only folds, PHP
truthiness conversion for loose logical operands, arrays, objects,
non-lowerable float sources, and dynamic string allocation beyond the static
straight-line subset until generated code has PHP comparison coercions,
non-scalar comparison behavior, references/copy-on-write side-effect behavior,
and exact native error behavior.
Native lowering accepts unary minus only when the operand is already a
lowerable integer or float, and logical not when the operand is already a
lowerable boolean or a native boolean expression result, when the operand is
`null`, or when a known integer, finite float, or string operand has one statically known PHP
truthiness result, in the straight-line subset. Dynamic boolean double
logical-not expressions such as `!!$flag` reuse the original native boolean
expression instead of emitting redundant inversions. Native lowering folds
double logical-not over known scalar operands such as integers, finite floats,
strings, and `null` through the same known-truthiness subset without emitting
boolean operations. Native lowering folds
logical not over single-result statically known native boolean expression
operands to the known boolean result in LLVM IR and in the C assembly fallback
when the C boolean expression has a tracked result. Known numeric logical-not
folds to a static boolean for zero and nonzero known integer/finite-float
operands when all possible values have the same truthiness. Known string
logical-not folds to a static boolean for `""`, `"0"`, and known-truthy string
operands when all possible string values have the same truthiness. Null
logical-not folds to `true` without claiming broader null truthiness beyond
the documented logical binary folding subset. Integer unary-minus results remain
statically tracked for later
checked integer arithmetic when all bounded possible negation results are
proven not to overflow; single-result statically known integer operands fold
to the known negated result without a redundant native unary-minus operation.
Finite float unary-minus results remain tracked for later strict-identity
folding when every possible negation result is proven; single-result
statically known nonzero finite float operands fold to the known negated
result without a redundant native unary-minus operation.
It rejects boolean, string, null, array, and object unary-minus operands until
generated code has PHP numeric coercion. It rejects ambiguous numeric or
string logical-not truthiness, untracked numeric/string logical-not
expressions, non-finite float logical-not truthiness, null truthiness outside
logical-not, other truthiness conversion, unary integer overflow behavior,
references/copy-on-write side-effect behavior, and exact native error
behavior.
Native logical lowering accepts `&&`, `||`, `and`, `or`, and `xor` only when
both operands are already lowerable booleans or native boolean expression
results, or when both already-lowerable scalar operands have one statically
known PHP truthiness result, in the straight-line subset. Static boolean pairs
fold, and static boolean identity and annihilator edges such as `true ||
$flag`, `false && $flag`, `$flag && true`, and `$flag xor false` preserve
proven boolean results for later scalar lowering. Identical native boolean
expression operands for `&&`/`and` and `||`/`or` reuse the existing expression
without a redundant native boolean operation, and identical native boolean
expression operands for `xor` fold to `false`. Native boolean expression
operations whose tracked possible operands prove one result fold to that
static boolean without a redundant native boolean operation. Known scalar
logical operands whose null, integer, finite-float, or string truthiness is
unambiguous fold to a static boolean result without emitting a native boolean
operation. Statically decisive known-left `&&`/`and` and `||`/`or`
short-circuit cases such as `false && rhs` and `true || rhs` lower without
lowering the skipped right-hand operand. Other dynamic boolean expressions
lower to native boolean operations with PHP-shaped boolean echo output. It
rejects general PHP truthiness conversion, dynamic short-circuiting, `xor`
right-hand skipping, selected/evaluated unsupported right-hand operands,
ambiguous scalar truthiness, untracked scalar logical operands, non-finite float
truthiness, null coalescing, arrays, objects, references/copy-on-write
side-effect behavior, and exact native error
behavior.
Native lowering accepts binary `+`, `-`, and `*` when both operands are
already same-type lowerable floats, or when both operands are lowerable
integers and the integer result is statically proven not to overflow, in the
straight-line subset. It also accepts integer `%` when the divisor is a
statically known positive integer, and statically known modulo results remain
tracked for later checked integer arithmetic. Tracked integer expression
operands and integer literal operands for `$x % 1` fold to zero, and bounded
tracked integer expression operands whose possible values all produce the same
remainder for a positive literal divisor fold to that remainder. Integer
modulo by one also folds after both operands lower when the dividend is
intentionally untracked, such as an overflow-sensitive shift result; other
modulo cases still require a statically known positive divisor and keep the
documented runtime-check boundary. Identical tracked integer
expression operands and identical integer literal operands for `-` fold to
zero without emitting a redundant native subtraction, and identical tracked
finite float expression operands and identical finite float literals for `-`
fold to `0.0` without emitting a redundant native subtraction. Identical integer
subtraction also folds after both operands lower when the value is
intentionally untracked, such as overflow-sensitive shift results; other
non-identity arithmetic with such values still rejects because exact overflow
tracking is unavailable. Tracked integer
expression operands and integer literal operands for `$x + 0`, `0 + $x`, and
`$x - 0` reuse the existing value, and tracked integer expression operands and
integer literal operands for `$x * 1` and `1 * $x` also reuse the existing
value. Tracked integer expression operands and integer literal operands for
`$x * 0` and `0 * $x` fold to zero. The `+ 0`, `- 0`, `* 1`, and `* 0`
identity or annihilator forms also fold after both operands lower when the
other integer operand is intentionally untracked, such as overflow-sensitive
shift results; non-identity arithmetic with such values still rejects because
exact overflow tracking is unavailable. Tracked single-result integer expression
arithmetic for `+`, `-`, and `*` folds to the known integer literal after
checked overflow analysis when tracked possible integer operands prove one
result. Literal-only integer arithmetic and ambiguous tracked-expression plus
tracked-expression integer arithmetic stay emitted. Tracked finite float
expression operands and finite float literals for nonzero `$x + 0.0`, `0.0 + $x`, and
`$x - 0.0`, and for `$x * 1.0` and `1.0 * $x`, reuse the existing expression.
Single-result statically known nonzero finite `0.0 - $x` folds to the known
negated float literal. Tracked finite positive float expression operands and
finite positive float literals for `$x * 0.0` and `0.0 * $x` fold to positive
`0.0`. Single-result statically known nonzero finite `$x * -1.0` and
`-1.0 * $x` fold to the known negated float literal. Tracked finite nonzero
float expression arithmetic for `+`, `-`, and `*` folds to the known float
literal when tracked possible finite-float operands prove one nonzero result.
Literal-only float arithmetic, zero-result arithmetic, possible signed-zero,
negative, and non-finite float identity/subtraction or multiplication-by-zero
cases, and signed-zero-sensitive multiplication by `-1.0`, stay emitted or
rejected rather than being folded.
It rejects mixed
int/float arithmetic, strings, booleans, nulls, arrays, objects, `/`,
overflow-sensitive or not-statically-proven integer arithmetic, dynamic or
non-positive modulo divisors, modulo results that are not statically known
enough for later checked arithmetic, and modulo cases that need PHP coercion
or runtime checks until generated code has PHP numeric coercion, dynamic
division/modulo zero checks, modulo coercions, PHP integer overflow promotion,
references/copy-on-write side-effect behavior, and exact native error
behavior. Mixed int/float `+`, `-`, and `*` operands use a
mixed-numeric-specific codegen rejection until
generated code has PHP numeric promotion and exact result typing. Boolean,
null, and string operands in `+`, `-`, and `*` use a scalar-coercion-specific
codegen rejection until generated code has PHP numeric coercion and string
numeric parsing. Overflow-sensitive or not-statically-proven integer `+`, `-`,
and `*` cases use an integer-overflow-specific codegen rejection until
generated code has PHP integer overflow promotion and runtime checks. Native
`/` uses a division-specific codegen rejection until generated code has PHP
division semantics, runtime zero checks, and no misleading integer truncation.
Dynamic, zero, or non-positive
integer modulo divisors use a modulo-specific codegen rejection so this runtime
check boundary stays distinct from generic unsupported arithmetic operands.
Native lowering accepts string concatenation only when both operands are
already lowerable strings in the straight-line subset, including ternary
operands that prove one static string result, folding the result into a
generated static string constant. Empty-string concatenation identity also
folds for already-lowerable string operands, including untracked string pointer
expressions: `$text . ""` and `"" . $text` reuse `$text` without runtime
string allocation. It rejects PHP scalar-to-string conversion, non-empty
ambiguous string expressions, arrays, objects, resources, runtime string
allocation, references/copy-on-write side-effect behavior, and exact native
error behavior. Double-quoted interpolation remains a separate native codegen
boundary rather than being lowered as string concatenation.
Native lowering accepts `&&`, `||`, `and`, `or`, and `xor` only when operands
are already lowerable booleans or native boolean expression results, or when
already-lowerable scalar operands have one statically known PHP truthiness
result, in the straight-line subset. Static boolean pairs fold at compile time,
known scalar truthiness folds to a static boolean, and dynamic boolean
expressions lower to native boolean operations plus PHP-shaped boolean echo
output. Statically decisive known-left `&&`/`and` and `||`/`or` cases such as
`false && rhs` and `true || rhs` lower without lowering the skipped right-hand
operand. The native lowerer still rejects cases that need general PHP
truthiness conversion, dynamic short-circuiting, `xor` right-hand skipping, or
selected/evaluated unsupported right-hand operands. Ambiguous scalar
truthiness, untracked scalar logical operands, non-finite float truthiness,
null coalescing, arrays, objects, references/copy-on-write side-effect
behavior, exact native error behavior, linking/execution, and broader native
lowering remain unsupported.
Native lowering accepts integer bitwise `&`, `|`, `^`, and unary `~` only when
operands are already lowerable integers in the straight-line subset. Bounded
statically known integer bitwise and unary bitwise-not results remain tracked
for later checked integer arithmetic. Single-result statically known integer
operands for unary `~` fold to the known bitwise-not result without a
redundant native bitwise-not operation. Double unary bitwise-not `~~$x` over
an already-lowerable integer operand reuses `$x`, including intentionally
untracked integer expressions such as overflow-sensitive shift results.
Identical tracked integer expression
operands and identical integer literal operands for `&` and `|` reuse the
existing value, and identical tracked integer expression operands and
identical integer literal operands for `^` fold to zero. Identical integer
operands also fold after both operands lower when the value is intentionally
untracked, such as overflow-sensitive shift results: `$x & $x` and `$x | $x`
reuse `$x`, while `$x ^ $x` folds to zero. Tracked integer
expression operands and integer literal operands for `$x & -1` and `-1 & $x`,
and for `$x | 0`, `0 | $x`, `$x ^ 0`, and `0 ^ $x`, reuse the existing value.
Tracked integer expression operands and integer literal operands for
`$x & 0` and `0 & $x` fold to zero. Tracked integer expression operands and
integer literal operands for `$x | -1` and `-1 | $x` fold to `-1` after both
operands lower. Single-known integer operands for `$x ^ -1` and `-1 ^ $x`
fold to the known bitwise-not result. The `& 0`, `& -1`, `| 0`, and `^ 0`
identity or annihilator forms also fold after both operands lower when the
other integer operand is intentionally untracked, such as overflow-sensitive
shift results. Tracked integer expression bitwise
operations for `&`, `|`, and `^` fold to the known integer literal when
tracked possible integer operands prove one result. Literal-only integer
bitwise operations and ambiguous tracked-expression plus tracked-expression
bitwise operations stay emitted. It accepts integer shifts
`<<` and `>>` only when the left operand is already a lowerable integer and the
right operand is a literal count or tracked integer expression count that
proves one value from 0 through 63; right shifts use arithmetic shift for
signed integer results. Tracked integer expression operands and integer
literal operands for `$x << 0` and `$x >> 0` reuse the existing value. Tracked
shift-by-zero identities also fold after both operands lower when the left
integer operand is intentionally untracked, such as an overflow-sensitive shift
result. Tracked single-result integer expression shifts with static safe
nonzero counts fold to the known integer literal. Literal-only shifts and
non-single tracked integer shifts stay emitted. Bounded statically known safe shift results remain
tracked for later checked integer arithmetic; overflow-sensitive left-shift
result sets remain unknown so later arithmetic rejects them instead of
implying PHP overflow semantics. It rejects ambiguous dynamic shift counts,
negative or large counts, PHP bytewise string bitwise operations, scalar-to-int coercion
for non-integer operands, arrays,
objects, references/copy-on-write side-effect behavior, exact native error
behavior, linking/execution, and broader native lowering.
Native lowering accepts full ternary `condition ? if_true : if_false` only
when the condition is already a lowerable boolean or native boolean expression
and both branch values are already lowerable integers, booleans, floats,
strings, or both branches are `null` in the same straight-line subset, or when
the condition is a statically known boolean and both branch values are already
lowerable scalar values, or when the condition and both branches are the same
direct variable whose current value is already lowerable. Dynamic mixed-type branch values are rejected until
native tagged values exist. Dynamic non-null ternaries emit LLVM `select` or
the corresponding C conditional expression, identical static string branches
fold to that string without a pointer select, identical boolean expression
branches fold to the reused expression without a redundant boolean select,
identical tracked integer expression branches and identical integer literal
branches fold to the reused value without a redundant integer select, and
identical integer branches also fold after both branches lower when the integer
value is intentionally untracked, such as an overflow-sensitive shift result.
Identical direct-variable full ternaries such as `$value ? $value : $value`
reuse the direct variable value without proving truthiness when all three
operands are the same already-lowerable direct variable, including untracked
integer, non-finite float-producing, string pointer, and boolean expressions,
plus null values.
Identical tracked float expression branches and identical float literal
branches fold to the reused value without a redundant float select, and
identical float branches also fold after both branches lower when the value is
intentionally untracked, such as a non-finite overflowing float multiplication.
Dynamic boolean literal branches fold without a boolean select for `$flag ? true : false`,
`$flag ? false : true`, `$flag ? true : true`, and
`$flag ? false : false`, dynamic `null`/`null` ternaries fold to `null`, and
static boolean ternaries fold to the selected branch value. Dynamic integer,
finite-float, and boolean ternaries whose possible branch values collapse to a
single known result fold to that scalar without a redundant select; ambiguous
same-type ternaries stay emitted. Full ternary conditions with null or with
single-known integer, finite-float, or known-string truthiness lower only the
selected already-lowerable branch; direct null-variable conditions select the
false branch without lowering unsupported true-branch calls. Dynamic boolean
full ternaries still require both branches to lower before selection.
Ambiguous integer, float, or string conditions, untracked string conditions,
non-finite float result tracking, and non-finite float conditions remain
rejected, and dynamic branch skipping for unsupported or side-effecting
branches remains unsupported. Dynamic integer ternaries and later checked integer
arithmetic track up to four statically known possible values; combinations
with more possible results remain unsupported. The lowerer still evaluates all
operands for dynamic branch selection,
so it rejects cases that need general PHP truthiness or dynamic lazy branch
evaluation to skip unsupported or side-effecting branches. Native short ternary `?:`
accepts lowerable boolean conditions in the same straight-line subset; dynamic
boolean forms require a lowerable boolean fallback, static-false forms return
any already-lowerable scalar fallback, and static-true forms fold to `true`
without lowering the fallback. Single-known integer conditions also fold
through integer truthiness: proven nonzero integer conditions reuse the integer
result, and proven zero integer conditions use the fallback. Single-known
finite float conditions fold through float truthiness the same way, with
proven nonzero finite floats reusing the float result and proven zero floats
using the fallback. Known string conditions fold through PHP string
truthiness when all possible values have the same truthiness: non-empty
strings except `"0"` reuse the string result, while `""` and `"0"` use the
fallback. Identical direct string-variable short ternaries such as `$text ?:
$text`, identical direct integer- or float-variable short ternaries such as
`$value ?: $value`, and identical direct boolean-variable short ternaries such
as `$flag ?: $flag` also reuse already-lowerable expressions without proving
broader truthiness, including untracked string pointer expressions, untracked
integer expressions, untracked non-finite float-producing expressions, and
boolean expressions. Null short ternaries use the fallback for `null ?:
fallback`, including direct null-variable fallback forms such as
`$value ?: $value`; broader null
truthiness in logical binaries or null coalescing remains unsupported.
Ambiguous string truthiness, non-identical untracked integer, float, or string expressions,
non-finite float truthiness, other non-boolean truthiness, null coalescing `??`,
null-aware variable/offset/property lookup, arrays, objects,
references/copy-on-write side-effect behavior, exact native error behavior,
linking/execution, and broader native lowering remain unsupported.
Native lowering statically folds direct `gettype`, `is_null`, `is_bool`,
`is_int`/`is_integer`/`is_long`, `is_float`/`is_double`, `is_string`,
`is_array`, `is_scalar`, and `is_numeric` calls when their single argument is
already in the straight-line native scalar/null subset. Native `is_numeric`
also folds literal and tracked string values only when the current
numeric-string grammar proves the result statically. Direct `is_countable` and
`is_iterable` calls fold to `false` for already-lowerable scalar/null/string
operands only, direct `is_object` calls fold to `false` for already-lowerable
scalar/null/string operands only, and direct scalar/null/string
`get_debug_type` calls fold to the current runtime type-name strings.
Interpreter `is_iterable()` also recognizes object metadata that records
`implements Iterator` or `implements IteratorAggregate` after concrete class
registration verifies the required public non-static methods with no required
parameters. Direct concrete `implements Traversable` remains a runtime
boundary until broader built-in engine interface inheritance semantics exist;
this is not native lowering or object iterator execution.
Direct `is_callable($value)` calls fold when `$value` is an already-lowerable
string value with a uniform known lookup result in the documented builtin
table, or when `$value` is an already-lowerable non-string scalar/null value,
which folds to false. Direct `is_callable($value, $syntax_only)` calls also
fold when `$value` is an already-lowerable string or non-string scalar/null
value and `$syntax_only` is an already-lowerable boolean; true syntax-only
flags return true for string values without name lookup, non-string
scalar/null values return false, while false flags use the documented builtin
lookup table for strings.
Direct `function_exists($name)` calls fold only when `$name` is an
already-lowerable string value with a uniform known answer in the documented
builtin table. Documented builtin names fold to true and missing names fold to
false; user-defined function tables, namespace/autoload-aware lookup,
extension-loaded functions outside the documented table, dynamic callees, and
runtime callable dispatch remain outside native lowering.
`dirname()` is currently an interpreter-only path builtin for lexical
Unix-style local paths. Native `function_exists("dirname")` and
`is_callable("dirname")` use the known-function table, but direct native
`dirname(...)` calls still reject under the function-call boundary until native
path policy and string-return lowering are proven.
`version_compare()` is likewise interpreter-only: `phpc run` supports a
bounded numeric-component version grammar for WordPress bootstrap guards, while
native `function_exists("version_compare")` can see the name through the known
function table and direct native calls still reject under the function-call
boundary.
`sprintf()` is an interpreter-only bounded string formatter for the current
WordPress bootstrap guard/message paths. It supports literal text, `%%`, `%s`,
and `%N$s` placeholders using runtime echo-string conversion for values. Native
`function_exists("sprintf")` and `is_callable("sprintf")` can see the name
through the known function table, but direct native calls still reject under
the function-call boundary until varargs/string formatting helpers are lowered.
`strtolower()` is an interpreter-only bounded case-mapping builtin for current
scalar/null string-convertible values. It applies ASCII lowercase mapping over
runtime UTF-8 strings so WordPress bootstrap can normalize simple option
suffixes, while locale-sensitive casing, full Unicode case folding, binary
string behavior beyond valid UTF-8, and native lowering remain out of scope.
`trim()` is an interpreter-only bounded string-normalization builtin for
current scalar/null string-convertible values. The first slice implements the
default PHP whitespace mask for represented runtime strings so WordPress can
parse shorthand INI values, while custom masks, binary/null-byte edge cases,
and native lowering remain out of scope.
`ltrim()` and `rtrim()` are interpreter-only bounded one-sided string
normalization builtins for the same scalar/null string-convertible values.
They support the default PHP whitespace mask and reached literal masks such as
`/`, while character-mask ranges, broad binary edge cases, object/resource
operands, exact diagnostics, and native lowering remain out of scope.
`str_contains()` is an interpreter-only bounded string-search builtin for
current scalar/null string-convertible haystack and needle values. It uses the
current UTF-8 runtime string representation, keeps PHP's empty-needle `true`
result, and leaves binary string edge cases and native lowering out of scope.
`str_starts_with()` is an interpreter-only bounded string-prefix builtin for
the same scalar/null string-convertible haystack and needle subset. It keeps
PHP's empty-needle `true` result for represented runtime strings, and leaves
binary string edge cases, object/resource coercions, exact diagnostics, and
native lowering out of scope. Direct native `str_starts_with(...)` calls stop
at a dedicated string-prefix codegen boundary before argument lowering or
backend selection, while native function-table introspection can still see the
known builtin name.
`str_ends_with()` is an interpreter-only bounded string-suffix builtin for the
same scalar/null string-convertible haystack and needle subset. It keeps PHP's
empty-needle `true` result for represented runtime strings, and leaves binary
string edge cases, object/resource coercions, exact diagnostics, and native
lowering out of scope. Direct native `str_ends_with(...)` calls stop at a
dedicated string-suffix codegen boundary before argument lowering or backend
selection, while native function-table introspection can still see the known
builtin name.
`basename()` is an interpreter-only bounded lexical path builtin for Unix-style
local path strings and an optional string suffix. It does not consult the
filesystem and leaves Windows drive/UNC paths, stream wrappers, null-byte
behavior, locale/codepage details, broad scalar coercions, exact diagnostics,
and native path lowering out of scope. Direct native `basename(...)` calls stop
at a dedicated path-basename codegen boundary before argument lowering or
backend selection, while native function-table introspection can still see the
known builtin name.
`substr()` is an interpreter-only bounded string-slicing builtin for current
scalar/null string-convertible inputs, integer offsets, and optional integer
lengths. It uses byte positions over represented runtime strings and rejects
slices that would produce invalid UTF-8, leaving PHP-exact binary string
behavior, broad offset/length coercions, object/resource operands, diagnostics,
and native lowering out of scope.
`min()` is an interpreter-only bounded integer helper for the current WordPress
memory-limit clamp. It accepts two or more integer arguments and returns the
smallest integer, while array-form calls, mixed-type comparison rules, and
native lowering remain out of scope.
`rand()` is an interpreter-only deterministic random boundary for the reached
WordPress `wpdb::placeholder_escape()` salt path. The current slice accepts no
arguments and returns a fixed integer so compatibility probes are reproducible;
PHP random-state compatibility, min/max forms, seeding, cryptographic
randomness, and native lowering remain out of scope.
`uniqid()` and `hash_hmac()` are interpreter-only deterministic hash
boundaries for the same placeholder-escape path. `uniqid()` returns a fixed
prefix-based ID, and `hash_hmac()` currently supports lowercase hex
HMAC-SHA256 through the `hmac` and `sha2` crates. Broader hash algorithms,
raw binary output, exact entropy/time behavior, and native lowering remain out
of scope.
`strcasecmp()` is an interpreter-only bounded string comparison builtin for
current scalar/null string-convertible values. It compares valid UTF-8 runtime
strings by bytes with ASCII case folding and returns only sign values. Native
function-table introspection recognizes the name, while direct native calls
still reject until string comparison helpers and diagnostics are lowered.
`str_replace()` is an interpreter-only bounded string replacement builtin for
scalar/null string-convertible search, replacement, and subject values. Native
function-table introspection recognizes the name, while direct native calls
still reject until string allocation, array forms, count-output references, and
diagnostics have a lowered runtime model.
`call_user_func()` is an interpreter-only bounded callable dispatcher for
string callbacks resolving to current user functions or documented callable
builtins. It reuses the value-based call path and does not implement array
callables, closure invocation, `__invoke`, `call_user_func_array`, references,
variadic unpacking, or native lowering.
`call_user_func_array()` is a separate interpreter-only callable dispatcher for
string callbacks, current public array-callable shapes, integer-keyed
positional argument arrays, and the bounded string-keyed named argument slice
for current user callbacks. String user-function callbacks, public
`[object, method]` instance callbacks, and public `["ClassName", "method"]`
static callbacks can bind reached by-reference parameters from unkeyed or
integer-keyed literal argument-array elements such as `array(&$value)` and
`array(10 => &$value)`, or from string-keyed elements whose keys match
declared parameter names, using the same direct caller cell as ordinary
by-reference user-function and method calls. Direct public object-property
array-offset reference elements use the current copy-in/writeback bridge for
that callback path. Direct stored argument arrays whose reached slots were
assigned by reference through the covered direct array-offset target path use
the existing alias metadata as a copy-in/writeback bridge, including for
normal callback invocations of reference-returning user functions or public
array-callable methods that mutate reached by-reference parameters.
Direct variable, direct array-offset, direct append-offset, nested
append-offset, direct visible object-property, and direct visible
object-property append-offset assignments from reference array literals now add
the selected literal slot to the same bounded alias metadata used by direct
reference assignment, so `$args = array(&$value)`,
`$registry["args"] = array(&$value)`,
`$store->args = array("value" => &$object->items[$key])`,
`$args[] = array(&$value)`, `$registry["groups"][] = array(&$value)`, and
`$store->groups[] = array(&$items["slot"])` can be used later as stored
`call_user_func_array()` argument arrays. Append targets first materialize the
array value at the runtime append key and then record literal reference slots
below that key. If the assigned direct variable is already backed by covered
array-offset alias metadata, such as `$args =& $registry["args"]`, the literal
reference slots are recorded below that aliased target path. This is still
keyed alias metadata over materialized array values, not a runtime reference
container. Reference array literals assigned into dynamic-property append
targets, `ArrayAccess` append targets, or other non-variable targets,
reference elements from arbitrary expressions, stored arrays without
covered reference-assigned slots,
execution past unknown or duplicate string-keyed argument names, variadic
named callback arguments, positional arguments after string-keyed named
arguments, closure invocation, `__invoke`, broader
named-argument semantics, exact warning behavior, and native lowering remain
unsupported.
`implode()` is an interpreter-only bounded array-to-string builtin for current
WordPress bootstrap message paths. It joins scalar/null array values in
insertion order with either an empty default separator or a string separator.
Native function-table introspection recognizes the name, while direct native
calls reject until array iteration, string allocation, and conversion
diagnostics have a lowered runtime model.
`ob_start()`, `ob_get_level()`, `ob_get_contents()`, `ob_get_length()`,
`ob_list_handlers()`, `ob_get_status()`, `ob_get_clean()`, `ob_get_flush()`,
`ob_clean()`, `ob_flush()`, `ob_end_clean()`, and `ob_end_flush()` are
interpreter-only output-buffer boundaries for the current WordPress
request/rendering path. The interpreter keeps a stack of string buffers;
PHP-visible output appends to the innermost active buffer, `ob_get_contents()`
peeks at that buffer without closing it, `ob_get_length()` reports its current
byte length, `ob_list_handlers()` reports one default handler name per active
buffer, `ob_get_status()` reports bounded default-handler status arrays for
the innermost buffer or for the full active stack, `ob_get_clean()` pops and
returns that buffer, `ob_get_flush()` closes the innermost buffer while
returning and flushing its contents outward, `ob_clean()` clears the innermost
buffer, `ob_flush()` moves its contents outward while keeping it active,
`ob_end_clean()` closes and discards it, `ob_end_flush()` closes it and
flushes its contents outward, and remaining
buffers flush outward to stdout when execution completes or the bounded
`exit()` path returns. Native function-table introspection recognizes the
names, while direct native calls reject until generated code has stdout capture
buffers, shutdown flushing, output-started/header interaction, SAPI
integration, and exact diagnostics.
`register_shutdown_function()` is modeled as an interpreter-only request/SAPI
shutdown queue. Registration evaluates and stores the callback plus extra
arguments in request-local interpreter state. Normal completion and the
bounded `exit()` path drain supported string user/builtin callbacks and public
object/static array callables in registration order before object destructors
and final output-buffer flushing; callbacks registered while the queue is
draining are appended and reached later in the same shutdown pass. Closure
callbacks remain a registration-only boundary because current closure values
store capture metadata but not executable closure bodies. The model does not
claim PHP's full fatal-error, finally, destructor, by-reference argument,
invokable-object, or native shutdown semantics.
`header()` is an interpreter-only web/SAPI boundary for the current WordPress
bootstrap/request path. It validates the current string/bool/int argument
shape, records the raw header line in deterministic in-process CLI request
state, and returns `null`. For ordinary colon-delimited header lines, default
replacement removes earlier lines with the same ASCII-case-insensitive field
name before appending the new line; `$replace = false` keeps duplicate lines.
Before output starts, it also updates request-local status state from an
explicit non-zero integer response-code argument, from bounded `HTTP/... NNN`
status lines, and from PHP-compatible `Location:` defaulting to `302` unless
the current status is `201` or already a redirect. A zero response-code
argument is treated as no explicit status. After output starts, late
`header()` calls leave the log/status unchanged and route a bounded
`E_WARNING` through the current error-handler stack or stderr fallback.
`http_response_code()` reads or writes that request-local status state and
preserves PHP's current previous-value return shape for the covered integer
argument slice.
`headers_list()` returns that header log as an ordered array of strings. Native
function-table introspection recognizes the names, while direct native calls
reject through a header-state boundary until response storage, diagnostics,
output-started tracking, status handling, and SAPI integration have a lowered
runtime model.
`header_remove()` mutates that deterministic CLI header log for the current
bounded subset: no arguments clear the log, and one string removes entries
whose raw header line has the same ASCII-case-insensitive field name before
the first colon. It leaves the log and response status unchanged after
unbuffered output has started and routes a bounded `E_WARNING` through the
current error-handler stack or stderr fallback. It still does not model
status-header removal, whitespace normalization, exact PHP warning text, or
full SAPI removal behavior.
`setcookie()` and `setrawcookie()` are interpreter-only header-state
boundaries. The current slice accepts a string cookie name plus optional string
value, bounded positional attributes, or the bounded options-array attribute
form. It formats nonzero expiration timestamps as GMT dates, appends a
deterministic `Set-Cookie:` line to the same CLI header log, and replaces
earlier deterministic `Set-Cookie` lines with the same cookie name.
`setcookie()` percent-encodes the value; `setrawcookie()` preserves the raw
string value. After unbuffered output starts these calls return `false`, leave
the header log unchanged, and route a bounded `E_WARNING` through the current
error-handler stack or stderr fallback; cookie-name validation/encoding,
`Max-Age`, path/domain-aware duplicate handling, exact warning text, SAPI
emission, and native lowering remain outside the model.
`session_start()` uses the same request-local output-started state for the
current bounded session lifecycle. Before unbuffered output it materializes the
in-memory `$_SESSION` root from a PHP-compatible `sess_<id>` file when
`session.save_path` was set through `ini_set()` and the current id is bounded
to ASCII letters, digits, underscores, or hyphens. When that file exists but
the current bounded scalar/array session decoder cannot parse it,
`session_start()` routes one recoverable warning and recovers with an empty
session array. If an explicit non-empty id
falls outside that bounded file-safe subset, the start returns `false`, routes a
bounded `E_WARNING`, and does not mark the session active, append headers, or
materialize `$_SESSION`. Without a file it falls back to the request-local
snapshot keyed by the current session id, or to an empty array when no snapshot
exists, and marks the session active. A fresh
successful start appends a deterministic
`Set-Cookie: PHPSESSID=<id>` line to the same request-local CLI header log used
by `header()`, `setcookie()`, and `headers_list()`; the bounded `use_cookies`
session option suppresses that line when it is falsey. The bounded
`cookie_lifetime`, `cookie_path`, `cookie_domain`, `cookie_secure`,
`cookie_httponly`, and `cookie_samesite` options only format deterministic
attributes on that in-memory header log; they do not model cookie encoding,
expiration-date formatting, replacement policy, or host SAPI emission.
`session_write_close()` stores the active `$_SESSION` array
back into that request-local snapshot map and, when the bounded save-path/id
file slice is active, writes string-keyed scalar and array values back to
`sess_<id>` before closing the status. A later `session_start()` for the same
id replaces the visible root from the session file or last closed snapshot, so
edits made while the session was closed are not promoted to active data. The recognized
`read_and_close` option reloads the file or snapshot
and immediately closes the bounded status back to `PHP_SESSION_NONE` while
keeping the in-memory session array visible. After unbuffered output it
returns `false`, leaves session status/data unchanged, and routes a bounded
`E_WARNING` through the current error-handler stack or stderr fallback before
applying options. Cache-header emission, cookie encoding, expiration-date
formatting, replacement policy, trans-sid behavior, locking, save handlers,
garbage collection, broader PHP session-id policy, integer top-level session
keys, object/resource session serialization, exact malformed-session recovery
parity, option effects beyond the documented session-start options, exact
warning text, reference aliases across `_SESSION` root replacement, and native
lowering remain outside the model.
`headers_sent()` is an interpreter-only web/SAPI boundary. The current
slice tracks the first non-empty write that reaches unbuffered stdout. Echo,
print, `exit("message")`, `var_dump()`, `print_r()`, and outermost
`ob_get_flush()`/`ob_flush()`/`ob_end_flush()` stamp that state with the current source filename
and source line; output held only inside active output buffers does not. Direct
variable, direct array-offset, direct object-property, and direct
object-property array-offset filename/line output arguments are written with
`""`/`0` before output starts and the stamped file/line after it starts.
Dynamic object-property output targets, callback-mediated output targets, exact
warning text, SAPI differences, shutdown-time buffer flushing visibility, and
native lowering are not modeled.
`php_sapi_name()` is an interpreter-only request/SAPI identity boundary that
returns the same deterministic `cli` string as `PHP_SAPI`. It intentionally
does not query the host PHP binary, web-server SAPI, CGI/FPM state, or native
runtime state.
Direct native reads, indexed reads, `isset(...)`, and `empty(...)` over the
current request superglobals `$_SERVER`, `$_COOKIE`, `$_GET`, `$_POST`,
`$_REQUEST`, and `$_FILES` reject through a request-state codegen boundary
until generated code has request storage, SAPI population, `variables_order`
policy, upload metadata, references/copy-on-write, and exact diagnostics. This
avoids folding unassigned request bags as ordinary missing native variables.
`abs()` is an interpreter-only bounded numeric builtin for current integer and
finite-float values. It is intentionally narrower than PHP coercion until
numeric string, bool/null coercion, overflow, NaN/infinity, and native runtime
numeric diagnostics are modeled.
`microtime(true)` is an interpreter-only host-clock boundary for bootstrap
timing checks. It returns a finite float seconds value from `SystemTime`, while
the string-return forms stay unsupported until time virtualization, formatting,
precision, monotonicity, INI/timezone policy, and native runtime calls are
designed.
`ini_get()` is an interpreter-only configuration boundary backed by a
deterministic compatibility registry rather than host php.ini. It is intended
to make bootstrap decisions reproducible while leaving mutable INI state,
`ini_get_all()`, SAPI policy, extension-owned option catalogs, and native
runtime integration explicit future work.
`ignore_user_abort()` is an interpreter-only SAPI/connection-state placeholder
for entry flows such as WordPress `wp-cron.php`. It stores one deterministic
boolean in the interpreter, returns the previous setting as PHP's `0`/`1`
integer shape, and deliberately does not observe host client disconnects,
web-server/FastCGI/LiteSpeed behavior, request finishing, cron spawning, or
native runtime state.
`mysqli_connect()` is a placeholder-handle boundary, not database support. The
runtime and native function tables expose the name so early application
extension guards can move to the next compatibility blocker, and direct or
dynamic connection attempts now return deterministic placeholder `mysqli`
objects after validating the current scalar/null argument subset. Host socket
connections, authentication, real database selection, query/result behavior,
errors, full escaping, charset state, and native database calls are still
future work.
The current placeholder MySQLi slice also supports deterministic
`mysqli_real_connect()`, `mysqli_get_server_info()`, `mysqli_query()` for the
reached SQL-mode probe, `mysqli_select_db()`, and
`mysqli_real_escape_string()` for scalar/null string-convertible values over
the placeholder handle. These calls are compatibility probes, not host DB
integration. The interpreter has one narrow `wp_options` state island for exact
WordPress-shaped option writes and reads, including direct option-value,
autoload, option-name/option-value, exact value/autoload updates, and bounded
full-row option-name/value/autoload result shapes with or without
deterministic placeholder option IDs, plus exact option-name equality and
option-name-list deletes for current option/transient cleanup probes, and
direct/prepared autoload equality reads for alloptions-shaped probes, plus
prepared autoload-only option-name equality reads for the current
`update_option()` autoload reevaluation probe, plus prepared option-value
equality reads with `LIMIT 1` for transient-shaped `wpdb::get_var()` probes,
plus one-shot `mysqli_execute_query()` prepared option insert/update/replace/delete
mutations and deterministic `mysqli_stmt_insert_id()` metadata for
prepared-statement option/transient-shaped state probes,
plus exact explicit full-row-with-id and
star-projection option-name equality reads with and without `LIMIT 1` for
object-row/result/column `wpdb` probes, plus
bounded direct
`option_name LIKE '<prefix>%'` and prepared `option_name LIKE ?` result scans
and deletes for transient-shaped option rows. The prefix scan result shapes
also accept the exact trailing `ORDER BY option_name` / backticked
`ORDER BY` suffix with optional `ASC`, while preserving the deterministic
ascending option-name order already used by the state island. A further
bounded transient-timeout scan accepts exact direct/prepared option-name
projections with `option_name LIKE '<prefix>%' AND option_value < timestamp`
or the prepared equivalent for rows whose recorded option value parses below a
decimal threshold; the matching exact direct/prepared transient-timeout delete
shape removes only those timeout rows and reports deterministic affected-row
metadata. The state island also accepts one exact WordPress-shaped
multi-table transient cleanup delete over `wp_options` aliases `a` and `b`,
where `a.option_name` matches a trailing-percent payload prefix, does not
match the matching timeout prefix, and `b.option_name` is the supported
`CONCAT( '<timeout_prefix>', SUBSTRING( a.option_name, <offset> ) )` timeout
row whose decimal value is below the direct or prepared threshold. That slice
deletes both reached payload and timeout rows and updates affected-row
metadata. It also exposes deterministic `SHOW TABLES LIKE 'wp_options'`,
`DESCRIBE`/`DESC wp_options`, and `SHOW [FULL] COLUMNS FROM wp_options`
result rows plus deterministic `SHOW INDEX`/`SHOW KEYS FROM wp_options` rows
for the current four-column option-table schema so bounded install/update
probes can inspect table existence, primary/unique key markers, fixed
primary/unique index rows, and autoload default/collation metadata. A separate
per-handle dynamic schema island records the current bounded
`CREATE TABLE`/`ALTER TABLE` WordPress/dbDelta shapes and answers deterministic
`SHOW TABLES LIKE`, `SHOW TABLE STATUS LIKE`, `SHOW TABLE STATUS WHERE Name`,
`DESCRIBE`/`DESC`, `SHOW [FULL] COLUMNS`, `SHOW CREATE TABLE`, and
`SHOW INDEX`/`SHOW KEYS` probes against that recorded shape, including bounded
column `DEFAULT`/`DEFAULT NULL`/`NOT NULL`/`auto_increment` metadata, inline
column primary/unique/non-unique key metadata, bounded `ASC`/`DESC`
index-part ordering metadata for `SHOW INDEX` collation values, and
deterministic `SHOW CREATE TABLE` text. Metadata `LIKE`
filters support exact patterns plus `%` wildcards, `_` single-character
wildcards, and backslash-escaped `%`, `_`, and `\` literals for table names,
table status rows, and column names. The same placeholder transaction and
savepoint helpers that snapshot the `wp_options` state island also snapshot
and restore this bounded dynamic schema-state island for recorded
`CREATE TABLE`/`ALTER TABLE` metadata. It does not model arbitrary
multi-table deletes, subqueries, schema DDL beyond the documented bounded
`CREATE TABLE`/`ALTER TABLE` shapes, dbDelta diffs,
charset/collation negotiation, locks, real index inspection
beyond recorded schema-state rows, expression indexes, opclass/parser metadata,
duplicate aliases, malformed `CONCAT`/`SUBSTRING`
forms, exact MySQL affected-row or insert-ID edge cases, real transactional
DDL/isolation/locking, or WordPress cleanup against tables outside the
deterministic `wp_options` state island; it is not a general SQL engine,
schema model, host database connection, PDO layer, or native database runtime.
`spl_autoload_register()` is currently an interpreter-only bounded
registration path: it accepts closure expressions, string user-function
callbacks, public `"ClassName::method"` static-method string callbacks,
public `[object, "method"]` instance-method array callbacks, and public
`["ClassName", "method"]` static-method array callbacks, plus object
callbacks with a public non-static `__invoke($name)` method, with optional
boolean flags and returns true. Supported non-closure callbacks are stored in
registration order, the current boolean `prepend` flag can place a callback at
the front, and truthy-autoload
`class_exists()`/`interface_exists()`/`trait_exists()` misses invoke those
callbacks with the requested class/interface/trait name before rechecking
metadata. That lets local include/require paths register autoloaded
class/interface/trait declarations through the existing included-file
declaration loader, including missing direct trait `use` names reached while
registering an included class declaration. Closure callbacks remain accepted
registration metadata only and report a stable unsupported autoload boundary if
lookup needs to invoke them. Non-public/static `__invoke`, invokable-object
dispatch outside autoloading, non-public methods, class-string non-static
methods, object static methods, `self::`/`parent::`/`static::` callback
strings, and exact callable validation remain unsupported. The same stored
callback vector backs `spl_autoload_functions()`, which reifies the current
bounded callback shapes as PHP values in dispatch order, and
`spl_autoload_unregister()`, which removes the first matching bounded callback
or returns false for valid unregistered callbacks. `spl_autoload_call($class)`
manually dispatches the same bounded callback vector for a string
class/interface/trait name and stops once any class-like metadata with that
name exists. `spl_autoload_extensions()` is a separate request-local string
slot initialized to PHP's `.inc,.php` default; string arguments replace the
slot, `null` reads it without mutation, and the value is exposed for parity
with autoload lifecycle introspection. The default `spl_autoload()` callback
uses that string as a comma-separated extension registry, lowercases the
requested class-like name, maps namespace separators to local path separators,
resolves candidate files through the same bounded local include resolver, and
includes the first existing file once. Registering `"spl_autoload"` stores it
as an ordinary function callback, but dispatch handles it as this builtin
probe instead of requiring a user function body. `class_alias()` uses that
same callback path when its source class or interface is missing and the
current bool-like autoload flag is truthy; successful class aliases insert an
additional case-insensitive name into the class table that points at the
original class metadata, so alias instantiation still creates objects whose
declared class is the source class. Runtime object relationship metadata
records active class-alias lookup names for the instantiated class and its
ancestors, which lets the current simple typed-property compatibility check
accept alias type names when the alias exists before object instantiation.
Successful interface aliases insert an
additional case-insensitive interface lookup entry that points at the original
interface declaration; relationship checks canonicalize that alias back to the
original interface name, instantiated object metadata records active aliases
for implemented declared interfaces for the same typed-property slice, and
`get_declared_interfaces()` still walks only real interface declarations.
Stream/URL paths, exact PHP warnings,
scalar-to-string extension coercions, recursive edge cases beyond the current
same-name guard, enum autoload lookup, trait aliasing, alias entries in
`get_declared_classes()`/`get_declared_interfaces()`, and native autoload
lowering remain outside this slice. Native function-table introspection
recognizes the names, while direct native calls reject under the function-call
boundary.
`assert()` is currently an interpreter-only assertion builtin for truthy
bootstrap guards. It evaluates one or two arguments normally, accepts scalar or
null descriptions as inert metadata, returns true for truthy assertions, and
keeps failing assertions behind a stable runtime boundary. Native
function-table introspection recognizes the name, while direct native calls
reject under the function-call boundary until generated code has assertion
policy, callbacks/exceptions, exact diagnostics, and unwinding behavior.
Runtime-backed constant behavior currently lives in the interpreter's
`ConstantTable`. Unbracketed namespace-scoped top-level `const NAME = value;`
declarations store canonical qualified names such as
`Sodium\CRYPTO_AUTH_BYTES`. String-name `defined(...)` and `constant(...)`
lookups accept qualified names with an optional leading global namespace
separator and answer from that deterministic table. Runtime string lookups for
declared class constants use the interpreter class metadata instead:
`ClassName::CONST` and `\ClassName::CONST` are split at runtime,
`defined(...)` reports true only for public declared or inherited constants,
and `constant(...)` resolves through the same class-constant visibility checks
as direct `ClassName::CONST`. This does not model bare namespace constant
fallback reads, autoload-triggered class discovery, broader `self`/`parent`/
`static` string names, host extension loading, full extension constant
inventories, or native lowering.
Direct `defined($name)` calls include the deterministic `PHP_VERSION_ID`,
`PHP_VERSION`, and 64-bit `PHP_INT_MAX` compatibility-target constants in the
built-in answer table. Bare global constant reads and `constant($name)` still
stay behind the native global-constant boundary until generated code has a real
constant table and version-policy model.
Direct `extension_loaded($name)` calls with already-lowerable string names fold
against the current deterministic bounded compatibility registry: `json` and
`hash` fold to true, while other names fold to false. Native code does not
query host PHP modules, `php.ini`, SAPI state, or dynamic extension loading.
`file_exists()` is currently an interpreter-only local filesystem metadata
builtin for the WordPress bootstrap drop-in check. It accepts one string local
path, rejects stream-wrapper paths, and returns a boolean for host filesystem
metadata existence. Relative paths check the process path first and then the
repository root for committed source-map fixture paths; this does not establish
include-path lookup, canonicalization, stream support, stat-cache semantics,
open_basedir, exact warnings, or native filesystem lowering. Native
function-table introspection recognizes the name, while direct native calls
reject under the function-call boundary.
`file_get_contents()` is also interpreter-only in the current runtime. It maps
`php://input` to the same explicit `PHPC_REQUEST_BODY` request seed used by
the request-bag scaffolding and otherwise keeps a bounded local UTF-8 text file
read using the same process-path-then-repo-root relative path policy as the
filesystem metadata builtins. The current bounded second argument accepts a
bool include-path flag; when true for relative local paths, lookup follows
the same include-path candidate order used by current `include`/`require`
resolution. The third argument accepts a bounded stream-context resource or
`null`, and the fourth/fifth arguments apply integer offset plus optional
non-negative max length over the current UTF-8 string payload. Missing local
file reads and negative offsets before the start of the current local or
`php://input` payload emit a bounded PHP-style `E_WARNING`, return `false`,
and continue. That warning can route through the current request-local
`set_error_handler()` stack when the top stored handler is a string
user-function callback or public object/static array callable whose mask
includes `E_WARNING`; `restore_error_handler()` pops that bounded stack so the
previous registration becomes active again. A `false` handler return falls
through to the stderr warning path when `error_reporting()` still includes
`E_WARNING`, while other return values suppress the fallback. This is
intentionally a local recovery
path, not the general PHP warning/error-handler system. It does not model PHP
binary strings, exact PHP warning or handler `errstr` text, stream context
effects, wrapper-specific context behavior, exact byte offsets through
non-UTF-8 data, warning recovery for other stream/resource paths,
handler stack mutation edge cases during active handler dispatch, `open_basedir`,
stat caching, host SAPI body streams, or native filesystem lowering. Direct
native `file_get_contents(...)` calls stop at a
dedicated filesystem-read codegen boundary before argument lowering or backend
selection, while native function-table introspection can still see the known
builtin name.
`filesize()` is interpreter-only for one string local path in the current
runtime. It uses the same process-path-then-repo-root relative path policy as
the other local metadata builtins, returns the host regular-file byte length
as an integer, and returns `false` for missing paths or non-file paths such as
directories. It rejects stream wrappers instead of modeling wrapper metadata.
Include-path lookup, PHP stat-cache semantics, `open_basedir`, exact warnings,
non-UTF-8 paths, oversized file handling beyond the current signed 64-bit
integer subset, and native filesystem lowering remain out of scope. Native
function-table introspection recognizes the name, while direct native calls
reject under the function-call boundary.
`filemtime()` is interpreter-only for one string local path in the current
runtime. It uses the same process-path-then-repo-root relative path policy as
the other local metadata builtins, returns the host filesystem modification
time as a Unix-timestamp integer for existing local entries, and returns
`false` for missing paths. It rejects stream wrappers instead of modeling
wrapper metadata. Include-path lookup, PHP stat-cache semantics,
`open_basedir`, exact warnings, non-UTF-8 paths, pre-Unix-epoch timestamps,
oversized timestamp handling beyond the current signed 64-bit integer subset,
and native filesystem lowering remain out of scope. Native function-table
introspection recognizes the name, while direct native calls reject under the
function-call boundary.
`realpath()` is interpreter-only for one string local path. It uses the same
process-path-then-repo-root relative path policy as the metadata builtins,
returns a UTF-8 resolved host path for existing local paths, and returns
`false` for unresolved local paths. Stream wrappers are rejected instead of
being modeled. Symlink policy differences, exact warning plus `false`
fidelity, include-path lookup, `open_basedir`, non-UTF-8 paths, stat-cache
behavior, and native filesystem lowering remain out of scope. Native
function-table introspection recognizes the name, while direct native
`realpath(...)` calls stop at a dedicated filesystem-canonicalization codegen
boundary before argument lowering or backend selection until generated code has
native filesystem canonicalization, symlink/path policy, warning/false
recovery, include_path/open_basedir/stat cache, non-UTF-8 path handling,
references/COW, and exact native diagnostics.
`getcwd()` is an interpreter-only request-state/filesystem builtin for the
current CLI process. It accepts no arguments and returns the process current
working directory as a UTF-8 string, while function-table introspection
recognizes the name. It does not model `chdir()` state mutation, failure
returning `false`, non-UTF-8 working-directory paths, SAPI-specific working
directory policy, include-path interaction, `open_basedir`, exact warnings, or
native filesystem lowering. Direct native `getcwd()` calls stop at a dedicated
current-directory codegen boundary before argument lowering or backend output
until generated code has process/request cwd state, UTF-8/path policy, SAPI cwd
behavior, references/copy-on-write, and exact native diagnostics.
`is_writable()` is interpreter-only for one string local path. It uses the same
process-path-then-repo-root relative path policy as the filesystem metadata
builtins, rejects stream-wrapper paths, returns `false` for missing local
paths, and treats existing readonly host metadata as not writable. Permission
portability, exact warning behavior, include_path lookup, `open_basedir`,
stream wrappers, symlink policy, stat-cache behavior, TOCTOU semantics,
non-UTF-8 paths, and native filesystem lowering remain out of scope. Native
function-table introspection recognizes the name, while direct native
`is_writable(...)` calls still reject under the generic function-call
boundary.
`is_link()` is interpreter-only for one string local path. It uses the same
process-path-then-repo-root relative path policy as the filesystem metadata
builtins, rejects stream-wrapper paths, returns `true` for host symbolic links
detected through local symlink metadata, and returns `false` for ordinary files
and missing local paths. Include-path lookup, `open_basedir`, stream wrappers,
exact warning behavior, stat-cache behavior, TOCTOU semantics,
broken-symlink policy fidelity, non-UTF-8 paths, and native filesystem lowering
remain out of scope. Native function-table introspection recognizes the name,
while direct native `is_link(...)` calls still reject under the generic
function-call boundary.
The table includes interpreter-only array builtins such as
`array_change_key_case`, `array_column`, `array_is_list`, `array_product`,
`array_reduce`, and `array_filter`; direct calls to those builtins still reject
under the array-lowering boundary.
Direct `strlen($value)` calls fold only when `$value` is an already-lowerable
known string operand, including tracked string expressions whose possible
values have one uniform byte length. A selected-`clang` assembly snapshot
validates that this existing folded LLVM IR is handed to the chosen backend
through stdin without changing production lowering behavior. Non-string
coercions, arrays, objects, resources, references/copy-on-write, dynamic
calls, and exact native PHP diagnostics remain outside this slice.
Direct `str_starts_with(...)` calls reject through a dedicated native
string-prefix boundary before argument lowering or backend selection. Native
function-table introspection still recognizes `str_starts_with`, but native
call execution still lacks PHP string conversion, empty-needle handling,
binary byte semantics, argument diagnostics, references/copy-on-write, and
exact native diagnostics.
Direct `file_get_contents(...)` calls reject through a dedicated native
filesystem-read boundary before argument lowering or backend selection. Native
function-table introspection still recognizes `file_get_contents`, but native
call execution still lacks stream-wrapper handling, local file I/O, binary
string byte fidelity, warning plus `false` recovery, stream contexts,
include-path lookup, `open_basedir` and stat-cache behavior,
references/copy-on-write, and exact native diagnostics.
The first stream-resource slices are interpreter-only. `fopen("php://memory",
$mode)` and `fopen("php://temp", $mode)` allocate request-local resource ids
backed by a Rust string buffer for simple `r`, `w`, `a`, or `c` modes with
optional `+`, `b`, or `t` flags. `fopen("php://input", $mode)` allocates a
read-only request-local stream over the deterministic `PHPC_REQUEST_BODY`
seed, reports PHP/Input/`rb` metadata, and reuses the current seek/read cursor
machinery without modeling a host SAPI input stream. `fopen($localPath, $mode,
$use_include_path = false, $context = null)` uses a host local file handle for
the same simple mode grammar, optionally resolves through the existing bounded
include-path candidate order, and accepts a bounded stream-context resource
without applying wrapper-specific context behavior. `stream_context_create()`
allocates a request-local context resource that stores array options and the
bounded params slice for `notification` plus `options`.
`stream_context_get_options()` returns those stored options, and
`stream_context_get_params()` returns the stored bounded params plus the
current `options` entry.
`stream_context_get_default()` lazily allocates and returns one request-local
default context resource, while `stream_context_set_default()` and
`stream_context_set_option()` merge string-keyed wrapper/option entries into
that same bounded context table. `stream_context_set_params()` updates the
stored bounded params and merges an `options` param into the same option table,
preserving the previous `notification` value when only options are supplied.
Unknown params, notification callback invocation, wrapper-specific effects, and
native lowering remain unsupported. `fwrite()`,
`fread()`, `rewind()`, `stream_get_contents()`, `feof()`, `ftell()`,
`fseek()`, `fstat()`, `stream_get_meta_data()`, and `fclose()` mutate, consume,
or inspect the resource cursor or bounded metadata; append-mode writes are
routed to EOF. A separate request-local directory resource table backs
`opendir()`, `readdir()`, `rewinddir()`, and `closedir()` for local UTF-8
directories, returning `.`, `..`, and sorted host entry names through the same
generic resource value shape. `fseek()` supports the built-in `SEEK_SET`,
`SEEK_CUR`, and `SEEK_END` constants for the current memory/temp/local-file
resource set, and `feof()` tracks the bounded EOF flag produced by exhaustive
reads. `fstat()` exposes buffer size for memory/temp/input handles and host
metadata for local files;
`stream_get_meta_data()` exposes deterministic wrapper/type/mode/URI,
seekable, unread-byte, and EOF metadata. Local file reads remain UTF-8 text
reads. This gives WordPress-style temporary request, cache-file stream, and
directory-scanning paths an executable path without claiming full PHP
resources: sockets, HTTP/FTP/phar wrappers, filters, context option effects
beyond persistence, broader wrapper/status metadata APIs, exact host
directory iteration order,
binary/non-UTF-8 byte strings, real SAPI body stream lifetime, writable
`php://input` edge behavior, `php://temp` spill-to-disk thresholds, permissions
policy, locking, stat-cache behavior, warning plus `false` recovery,
references/copy-on-write, and exact resource id/type behavior remain out of
scope. Native stream-resource calls reject before lowering under a
dedicated resource boundary, while function-table introspection recognizes the
names.
Direct `filesize(...)` calls reject through the native function-call boundary.
Native function-table introspection still recognizes `filesize`, but native
call execution still lacks filesystem metadata, warning plus `false` recovery,
include_path/open_basedir/stat-cache policy, stream-wrapper handling,
references/copy-on-write, and exact native diagnostics.
Direct `filemtime(...)` calls reject through the native function-call
boundary. Native function-table introspection still recognizes `filemtime`,
but native call execution still lacks filesystem modification-time metadata,
warning plus `false` recovery, include_path/open_basedir/stat-cache policy,
stream-wrapper handling, references/copy-on-write, and exact native
diagnostics.
Direct `realpath(...)` calls reject through a dedicated native
filesystem-canonicalization boundary before argument lowering or backend
selection. Native function-table introspection still recognizes `realpath`, but
native call execution still lacks filesystem canonicalization,
symlink/path policy, warning/false recovery, include_path/open_basedir/stat
cache, non-UTF-8 path handling, references/COW, and exact native diagnostics.
Native function-table introspection recognizes `is_writable`, but direct
native `is_writable(...)` calls still reject under the generic function-call
boundary until generated code has local writability checks, permission policy,
warnings, include_path and `open_basedir` handling, stream-wrapper
rejection/dispatch, symlink/stat-cache/TOCTOU behavior, non-UTF-8 path
handling, references/copy-on-write, and exact native diagnostics.
Native function-table introspection recognizes `is_link`, but direct native
`is_link(...)` calls still reject under the generic function-call boundary
until generated code has local symlink metadata lookup, include_path and
`open_basedir` handling, stream-wrapper rejection/dispatch, stat-cache/TOCTOU
behavior, broken-symlink policy fidelity, non-UTF-8 path handling,
references/copy-on-write, and exact native diagnostics.
Direct `defined($name)` calls fold only when `$name` is an already-lowerable
string value whose possible values are supported unqualified constant names
with a uniform answer against the current exact built-in constant-name set.
Exact `CASE_LOWER`, `CASE_UPPER`, `ARRAY_FILTER_USE_BOTH`,
`ARRAY_FILTER_USE_KEY`, `SORT_REGULAR`, `SORT_NUMERIC`, `SORT_STRING`,
`PHP_VERSION_ID`, `PHP_VERSION`, and `PHP_INT_MAX` names fold true, while other
supported unqualified names fold false.
Runtime-defined constants, source-order constant declarations, `define(...)`,
`constant(...)`, qualified names such as `\Sodium\CRYPTO_AUTH_BYTES`,
unsupported names, namespace-aware lookup, and exact native errors remain
outside this slice.
Direct `empty($name)` calls fold from the same straight-line static-variable
map used by direct `isset($name)`: missing variables and statically falsey
scalar/null values fold to true, while statically truthy scalar values fold to
false. Array offsets, object properties, arrays, complex operands,
unset/mutation interactions, ambiguous truthiness, references/copy-on-write,
and exact native error behavior remain outside this native slice.
Array/object operands remain rejected until native array/object lowering
exists. This is static folding, not runtime call dispatch. Dynamic calls,
wrong arity, non-string `function_exists` names, non-bool `is_callable`
syntax-only flags, callable-name output parameters, array/object/method
callables, direct `assert(...)`, callable builtin dispatch, runtime call lookup, stack-frame layout,
arity/type diagnostics, dynamic string-call dispatch, and exact native error
behavior remain unsupported. The
`define()`/`constant()` constant-table builtins and unsupported
`defined(...)` forms have a separate global-constant rejection boundary.
Native lowering also rejects user-function declarations and return statements,
including declarations whose parameter list uses the parser's optional
trailing-comma syntax, before traversing function bodies until generated code
has function symbol tables, stack-frame layout, default parameter binding,
recursion guards, return-value flow, and exact native error behavior.
Native lowering rejects executable magic constants `__LINE__`, `__FILE__`,
`__DIR__`, `__FUNCTION__`, and `__METHOD__` until generated code has source
mapping, path canonicalization, function/method-context tracking, eval/include
source interaction rules, and exact native error behavior.
Native lowering rejects built-in constants, runtime-defined constants, bare
constant reads, top-level `const` declarations, `define()`/`constant()`, and
unsupported `defined(...)` forms before operand/argument lowering until
generated code has native constant tables, source-order definitions,
namespace-aware lookup, and exact native error behavior.
Native lowering rejects class declarations, clone expressions, `instanceof`
expressions, and method-call expressions before body, operand, receiver, or
argument lowering. Object
metadata builtins have a dedicated native rejection before operand or argument
lowering, except for direct static
false-folding of
`class_exists`, `interface_exists`, `trait_exists`, and `enum_exists` when
their name argument is already lowerable as a string and the optional autoload
argument is already lowerable as a boolean. That metadata-exists native slice
does not expose native class metadata, autoloading, or callable dispatch.
Native lowering also folds direct `property_exists` and `method_exists` to
false when both arguments are already lowerable strings, because class
declarations still reject and there is no native class member table.
Direct string/string `is_a` and `is_subclass_of` calls with optional lowerable
boolean `allow_string` flags fold to false for the same no-native-class-table
and no-native-inheritance boundary.
Broader object/class lowering remains rejected until generated code has native
object layout, object handles, visibility checks, method dispatch, class
metadata tables, property/method tables, inheritance/interface/trait/enum
registries, property-slot cloning, `__clone` dispatch, reference-slot
metadata, inheritance, autoload interaction, and exact native error behavior.
Object instantiation and constructor dispatch have a separate native rejection
until generated code has native object allocation, object handles, constructor
calls, visibility checks, autoload/class lookup, references/copy-on-write, and
exact native object-instantiation errors.
Instance property reads/writes and dynamic property-name access have a
separate native object-property boundary until generated code has native
object layout, property tables/slots, visibility checks, magic property hooks,
dynamic property policy, references/copy-on-write, and exact native
object-property errors.
The method-call boundary is more specific: native method support still needs
method tables and lookup, instance and static receiver resolution, `$this` and
called-class/late-static-binding context, argument/arity diagnostics,
visibility enforcement, reference/COW parameter and return behavior, and exact
native method-call errors.
Native lowering rejects array literals, array indexing, array assignment,
`foreach` array iteration, array offset `unset`, and array builtin function
calls before body, operand, argument, or callback lowering until generated code
has native array storage layout, key normalization, copy-on-write containers,
references, callback dispatch, and exact native error behavior.
Native lowering rejects `if`/`elseif`/`else`, including alternate
colon/`endif` syntax, `while`, `for`, `do ... while`, `switch`, `break`, and
`continue` before condition, body, case, or loop-control lowering until
generated code has PHP truthiness, branch layout, loop control flow, switch
fallthrough, references/copy-on-write side-effect behavior, and exact native
error behavior.
Native lowering rejects compound assignment, null coalescing assignment,
increment/decrement, assignment expressions, direct variable `unset`, and
multiple-operand `unset` before operands or mutation targets are lowered until
generated code has read-modify-write ordering, null-aware mutation, unset
symbol-table effects, references/copy-on-write behavior, and exact native error
behavior.

## Dynamic Features

Dynamic PHP features will be implemented as runtime fallback zones:

- dynamic function calls use runtime lookup; the first implemented slice accepts
  string-valued callees that resolve to the documented callable builtin subset
  or user-defined functions
- variable variables use materialized symbol tables; current variable-variable
  syntax is rejected with an explicit diagnostic before execution
- dynamic includes will use runtime include resolution
- `eval` will parse and execute in the caller scope
- namespaces and imports use a bounded class-name plus same-namespace function
  declaration/call slice. Namespace-scoped function declarations register under
  their resolved names; unqualified direct calls inside a namespace first look
  for a same-namespace function and then fall back to global builtins/user
  functions. Function imports, qualified function calls, leading-backslash
  fully-qualified function calls, namespace-scoped constants, and dynamic
  string-name namespace expansion remain boundaries.
- global constants use a narrow interpreter constant table: exact uppercase
  `CASE_LOWER`, `CASE_UPPER`, `ARRAY_FILTER_USE_KEY`, and
  `ARRAY_FILTER_USE_BOTH` are available as bare built-in constants, while
  runtime-defined constants can be created with
  `define($name, $value)`, queried with `defined($name)`, and read with
  `constant($name)` or a bare unqualified constant name for the documented
  string-name and scalar/array value subset. Top-level single and grouped
  `const NAME = value;` declarations define unqualified constants at statement
  execution time over the current constant-expression and scalar/array value
  subset, including references to previously defined unqualified constants and
  the current built-in constant slice

String-valued dynamic function lookup and the narrow local `require path;` /
`require_once path;` / `include path;` / `include_once path;` statement slice
are executable today.
Variable-variable execution and `eval` remain design boundaries; direct
`eval(...)` syntax is reserved and rejected with a stable parse diagnostic.
Namespace declarations and top-level class `use` import declarations execute
as metadata/no-op statements in `phpc run`, but the native path still rejects
namespace declarations/imports before scalar folding or backend execution.
First-class callable syntax such as `strlen(...)` and `$callback(...)` also
stops at a stable parse diagnostic until Closure creation and callable object
semantics exist.
Call-site argument unpacking such as `handler(...$args)` stops at a dedicated
parse diagnostic until iterable expansion order, string-keyed named-argument
interaction, by-reference argument propagation, variadic collection, duplicate
argument diagnostics, and native lowering exist.
Call-time by-reference arguments such as `handler(&$value)` also stop at a
dedicated parse diagnostic because the current by-reference parameter execution
slice uses ordinary direct-variable call arguments and callee parameter
metadata; legacy call-site `&` syntax, alias setup, default handling,
variadic/unpacking interaction, references/copy-on-write, and native lowering
remain unsupported.
No-capture anonymous closure expressions, static anonymous closure expressions,
and non-static arrow function expressions allocate inert runtime closure values
in `phpc run`, which can be assigned, read, and truth-tested but not invoked.
Static closure binding semantics are not represented yet. Arrow values do not
bind implicit captures or execute their synthetic return bodies; invocation and
callable integration remain explicit runtime boundaries. Static arrow
functions stop at a dedicated parse boundary until no-`$this` binding,
implicit capture metadata, closure invocation, callback integration,
references/copy-on-write, and native lowering exist.
`phpc run` supports an opt-in execution-step budget via
`PHPC_MAX_EXECUTION_STEPS`; it is enforced at statement execution and loop
iteration boundaries to diagnose runtime loops, but it intentionally does not
count parser work, declaration registration, or native lowering.
`PHPC_TRACE_INCLUDES=1` writes include/require target paths to process stderr
before each target is parsed/executed so external timeout probes can retain the
last include frontier.
`PHPC_TRACE_PARSE=1` writes parser frontier lines for top-level statements,
class/interface/enum members, and block statements. It is an operational trace,
not PHP-visible output, and is intended for external timeout diagnosis.
Magic class names in `new` expressions, including `new self()`,
`new parent()`, and `new static()`, stop at a stable parse diagnostic until
class context tracking, parent resolution, and late static binding exist.
`is_callable($value, true)` has a narrow syntax-only array callable shape check
for two-element arrays keyed `0` and `1` whose first value is a string class
name or current object and whose second value is a string method name. This
does not resolve classes or methods and is not callable dispatch. Normal
`is_callable([$receiver, $method])` resolution checks the same shape against
current declared method metadata: object receivers are true for public declared
methods, and class-string receivers are true for public static declared
methods. Array/object callable dynamic invocation, private/protected
caller-context method callability, method calls, first-class callable syntax,
and namespace/autoload-aware callable resolution are still outside the
implemented dynamic-call subset. Constant names that are lexed as language keywords or
literals cannot be read bare, and case-insensitive legacy constants, extension
constants, namespace-qualified constants, nested or namespace-aware `const`
declarations, dynamic `const` values, class constants through
`constant(...)`/unsupported `defined(...)` forms, typed and multi-declarator
class constants, `static::CONST`, references/copy-on-write for constant values,
and broader constant lowering are still outside the implemented constant
subset. Namespace-qualified and leading-backslash fully-qualified constant
reads stop at dedicated parse diagnostics until namespace-aware constant-table
lookup, fallback behavior, imports, and native lowering exist. Direct
`ClassName::CONST`, `self::CONST`, and `parent::CONST`
execution use class metadata instead of the global constant table. Native
lowering currently folds only direct
supported-string-name `defined($name)` checks and otherwise rejects the
global-constant slice explicitly rather than emitting partial constant-table
code.

## Namespace/Import Boundary

The first executable namespace/import slice is class-name only. The parser
accepts one unbracketed named `namespace` declaration per file and simple
top-level class `use` imports with optional aliases. Class declarations and
class-like references in `extends`, `new`, `instanceof`, static members, and
`ClassName::class` are stored as canonical names without a leading slash and
resolved through the lexical namespace/import table. Namespace and `use`
statements are execution no-ops in `phpc run` because the parser has already
resolved the class-like names in the AST.
Class inheritance uses the same resolved class-like names. Parent classes must
already be present in the interpreter's class metadata table from the current
program or from an executed include/require path; class lookup does not invoke
autoload callbacks.

Unsupported namespace/import behavior remains: bracketed namespace blocks,
global namespace blocks, multiple namespaces in one file, namespace-scoped
functions/constants, namespace-qualified function calls, leading-backslash
fully-qualified function calls, namespace-qualified constant reads,
leading-backslash fully-qualified constant reads, grouped imports, function
imports, constant imports, string-name import expansion, trait `use`
execution, `__NAMESPACE__`, autoload interaction, exact PHP diagnostics,
partial-output behavior, and namespace-aware native lowering. The native path
rejects namespace declarations/imports before scalar folding or backend
execution until native symbol tables and namespace context exist.

## Object/Class Boundary

The current object/class step is a narrow object-execution boundary, not full
PHP object execution. `php_runtime` has a `PhpClassTable`, stable `ClassId`
handles, class metadata, property metadata, method metadata, visibility
markers, derived object shapes for instance-property layout, and minimal
object values. Class and
method lookup are case-insensitive, property lookup is case-sensitive, and
duplicate class/member metadata produces structured runtime errors.

`phpc run` parses top-level `class Name { ... }` declarations into that metadata
registry. The accepted member subset records public/protected/private
visibility, static flags, property names without defaults, and method names
whose parameters/bodies use the existing function parser subset. `new
ClassName(...)` can instantiate a declared class and execute a public or
inherited public `__construct` method with `$this` bound to the new object
handle. Classes without constructors still require no constructor arguments.
Objects whose class declares or inherits a public non-static no-argument
`__destruct` method are tracked after successful allocation and run during
normal shutdown in reverse allocation order; current shallow clones are tracked
as separate handles for that same destructor path. User-declared destructors
are validated when class members register, so non-public, static, or
parameterized `__destruct` declarations fail before object allocation instead
of being discovered by the shutdown queue.
The allocated object stores class identity and `null` instance-property slots
in inherited parent-to-child declaration order while skipping static
properties. Compatible public/protected redeclarations share the inherited
slot with the descendant visibility, while private parent redeclarations stay
separate child slots. Each slot carries the declaring class id/name used for
non-public access checks and private-property mangling.

`phpc run` can read and write public instance properties by static property
name, for example `$box->name` and `$box->name = "Ada"`. Writes mutate the
current object value stored in that variable. Direct `isset($box->name)` checks
the current public slot without treating a missing name as a property read.
Direct `empty($box->name)` checks the current public slot truthiness and treats
null slots, missing names, undefined target variables, and non-object target
variables as empty for the current direct-object-variable subset.
`get_class($object)` returns the declared class name stored in the current
minimal object value and is also available through string-valued dynamic
function calls. `get_debug_type($value)` reports current scalar/array type
names and the declared class name for current object values.
`class_exists($name[, $autoload])` checks the interpreter's already-registered
class metadata table by string class name, using the same case-insensitive
class lookup as instantiation. The autoload flag accepts current bool-like
scalar values; when truthy and the initial lookup misses, currently registered
string user-function callbacks, public `"ClassName::method"` static-method
string callbacks, public `[object, "method"]` instance-method array callables,
and public `["ClassName", "method"]` static-method array callables run before
the metadata check returns.
Missing named or direct-variable string class names in `new` expressions use
the same bounded autoload callback path before reporting the current
undefined-class diagnostic.
`null`, arrays, objects, references, and exact PHP
deprecation/`TypeError` behavior remain unsupported for that flag.
`interface_exists($name[, $autoload])`, `trait_exists($name[, $autoload])`,
and `enum_exists($name[, $autoload])` accept the same string-name and
bool-like scalar autoload boundary. `interface_exists()` shares the bounded
autoload callback path with `class_exists()`, and `trait_exists()` uses that
same path for declared top-level trait metadata. Enum lookups still check only
already-declared metadata. Invokable objects, closures at invocation time,
non-public methods, class-string non-static methods, object static methods,
arbitrary callable arrays, exact PHP warning/throw behavior, and native
autoload lowering remain unsupported. `class_exists()` also reports true for
declared enums in the current class-like metadata slice.
`property_exists($object_or_class, $property)` checks the same declared and
inherited property metadata for current object values or string class names,
with case-sensitive property names and no autoload side effects.
`get_class_vars($class_name)` accepts declared string class names and returns
public declared and inherited property names in child-to-parent declaration
order with supported constant-expression default values or `null` for
properties without defaults.
`get_object_vars($object)` accepts current object values and returns public
exact and inherited instance property names with their current slot values in
parent-to-child slot order.
`get_mangled_object_vars($object)` accepts current object values and returns
public, protected, and private instance slots in declaration order with
PHP-style property keys: public names as-is, protected names as `\0*\0name`,
and private names as `\0ClassName\0name` using the declaring class name.
Dynamic properties, readonly property metadata/enforcement, non-constant or
typed property defaults, promoted constructor properties, trait/interface
properties, and non-public property visibility-context behavior beyond
same-declaring-class private access and class/ancestor protected access remain
outside the current object model.
`is_a($object_or_class, $class_name[, $allow_string])` checks exact-class,
single-parent ancestor, and recorded `implements` metadata relationships
against the current metadata table.
`is_subclass_of(...)` shares the same argument boundary and walks the current
single-parent chain plus inherited `implements` metadata while keeping
exact-class and missing-class cases false.
`$value instanceof Name` uses the same current object class metadata,
single-parent chain, and recorded `implements` metadata for object values.
Non-object values return false. `implements` names are recorded as metadata so
internal names can participate in relationships without being declared. For
interfaces declared in the current parsed program, runtime class registration
checks concrete classes, including concrete children of abstract implementors,
for public methods with the required interface method names and required static
or non-static method shape. Parent interfaces declared before or after the
child are flattened into both relationship metadata and method-presence checks.
Implementations may omit an interface parameter type or repeat the same type
text case-insensitively, but may not add a parameter type to an untyped
interface parameter or substitute a different type for a typed interface
parameter. Implementations may add a return type to an untyped interface
method, but a typed interface method requires the implementation to declare the
same return type text case-insensitively. Public static methods satisfy only
static interface method requirements and do not satisfy non-static interface
method requirements. This is a bounded compatibility check only; full
parameter variance, broader return type covariance/contravariance,
type subtyping, alias/import resolution, union/intersection canonicalization,
cyclic parent-interface inheritance beyond stable rejection, broad
built-in/internal interface method enforcement beyond the current
`Countable`, `Iterator`, and `IteratorAggregate` shape checks, exact PHP error
objects, autoload behavior, and native lowering remain separate work. A bounded
core interface catalog seeds `interface_exists()` and `get_declared_interfaces()` for
`Traversable`, `IteratorAggregate`, `Iterator`, `Serializable`, `ArrayAccess`,
`Countable`, and `Stringable`. Protocol execution is added one narrow slice at
a time: current `ArrayAccess` paths dispatch selected offset methods, current
`Stringable`/`__toString` paths cover string conversion, and current
`Countable` paths require concrete implementors to expose a public non-static
`count()` method with no required parameters, let `is_countable($object)`
recognize recorded `implements Countable` metadata, and let `count($object)`
dispatch that method with an integer result. Current `Iterator` and
`IteratorAggregate` paths require concrete implementors to expose their
required public non-static methods with no required parameters so
`is_iterable($object)` can observe those metadata relationships. Full internal
interface signature enforcement, tentative return-type notices, object
`foreach`, iterator method execution, `IteratorAggregate::getIterator()`
dispatch, broad protocol composition, references/copy-on-write, exact
diagnostics, and native lowering remain
separate runtime work.
`get_parent_class($object_or_class)` accepts current object values or declared
string class names and returns the immediate parent class name when one is
recorded, otherwise false.
`class_implements($object_or_class[, $autoload])` is a bounded
reflection-style metadata builtin over the current class/interface table. It
accepts object values or string class names, optionally invokes the existing
class autoload path for string misses, and returns an associative
interface-name array. The collection follows system PHP for the covered
single-parent/user-interface cases, including parent-class interfaces before
child-class interfaces.
`class_uses($object_or_class[, $autoload])` is the matching bounded
class/trait metadata builtin. It accepts object values or string class names,
optionally invokes the existing class autoload path for string misses, and
returns an associative array of the resolved class's direct trait names. It
does not recurse into parent classes or synthesize broader Reflection metadata.
`class_parents($object_or_class[, $autoload])` walks the same class metadata
chain from immediate parent to root and returns a PHP-shaped associative array
of parent class names. This keeps `class_uses()` non-recursive while enabling
covered userland recursive trait helpers that explicitly combine parent names
with direct trait metadata.
`ReflectionClass` is a bounded runtime metadata object for declared
class-like values. `new ReflectionClass($object_or_class)` accepts object
values and string names for the current declared class, interface, and trait
tables, including the existing autoload callback path for string misses. The
object stores request-local reflection state and dispatches the current
`getName()`, `getShortName()`, `isInterface()`, `isTrait()`,
`isInstantiable()`, `getParentClass()`, `getInterfaceNames()`, and
`hasMethod($name)` methods directly through the interpreter. The same
request-local reflection path now covers declared user-class properties:
`hasProperty($name)`, `getProperty($name)`, and zero-argument
`getProperties()` resolve current class metadata, include inherited public and
protected properties, and exclude inherited private properties when reflecting
a child class. This is metadata only.
`ReflectionMethod` uses the same core placeholder class plus request-local
state pattern for declared user class, interface, and trait methods. The
constructor accepts an object or class-like string plus a string method name,
resolves inherited class methods through the existing method metadata chain,
and exposes the bounded source metadata methods, modifier predicates,
`getDeclaringClass()`, parameter counts, `getParameters()`,
`hasReturnType()`, and `getReturnType()` through interpreter dispatch.
Class-method source paths are tracked in the same method-signature table as
parameter and return metadata when declarations are loaded from a known
CLI/fixture or include path; start/end lines and directly preceding
`/** ... */` doc-comments come from the parsed method declaration. Interface
and trait methods currently retain parsed line/doc-comment metadata but do not
persist declaration source-file paths. Return type objects reuse the same request-local
`ReflectionNamedType`, `ReflectionUnionType`, and `ReflectionIntersectionType`
state as the property type metadata slice.
`ReflectionFunction` follows that same core placeholder plus request-local
state pattern for declared user functions named by string. It stores parsed
user-function metadata for `getName()`, source file, start/end lines, direct
docblock text, parameter counts, `getParameters()`, `hasReturnType()`,
`getReturnType()`, and `returnsReference()`. Source paths are tracked beside
registered function declarations because the AST remains source-text scoped;
included files record the include source path at declaration-registration time.
The lexer preserves bounded `/** ... */` doc-comment tokens so the parser can
attach a directly preceding docblock to a function declaration. The function
path intentionally rejects internal functions and closure targets until those
metadata sources exist, and doc-comment association does not yet model
attributes or every PHP trivia edge case.
`ReflectionParameter` follows the same request-local state pattern for
parameters reached from `ReflectionMethod::getParameters()`,
`ReflectionFunction::getParameters()`,
`new ReflectionParameter([$object_or_class, $method], $parameter)`, or
`new ReflectionParameter($function, $parameter)` for declared user function
strings. Parameter objects store copied function or method metadata plus the selected parsed parameter
metadata, so the interpreter can answer names, positions, declaring
class/function, optional/default availability and values, by-reference flags,
variadic flags, type-presence checks, nullability checks, and simple named
type metadata. `ReflectionParameter::getType()` now materializes a
request-local `ReflectionNamedType` object for a single parsed named type, or
a request-local `ReflectionUnionType`/`ReflectionIntersectionType` object for
bounded union and pure intersection parameter types, and stores the copied
type names, nullable flags, and builtin flags in the interpreter. Untyped
parameters return `null`. It does not expose
attributes, files, line numbers, doc comments, extension/internal metadata,
closure parameter targets, method/function invocation, exact exception
objects, DNF type objects, runtime argument/return type enforcement, or native
lowering.
`ReflectionProperty` uses a core placeholder class plus request-local state for
the selected declaring class id/name, property name, visibility, static flag,
and optional property type metadata. The interpreter answers name, declaring
class, modifier-mask, visibility/static predicates, default-value availability
and value, and `hasType()`/`getType()`. Simple named property types materialize
the existing request-local `ReflectionNamedType` object. Bounded union and pure
intersection property types materialize request-local `ReflectionUnionType` or
`ReflectionIntersectionType` objects whose `getTypes()` entries are
`ReflectionNamedType` objects. Runtime typed-property enforcement reuses the
current scalar coercion and class/interface relationship checks for those
bounded property type shapes. Parenthesized DNF property types, exact PHP union
scalar coercion preference rules, reference/COW interactions, and native
lowering remain unsupported.
`get_declared_classes()` lists classes and unit enums declared in the current
parsed program;
`get_declared_interfaces()` lists interfaces declared in the current parsed
program; `get_declared_traits()` lists top-level traits declared in the current
parsed program, including traits that contain currently supported public
instance methods.
`get_called_class()` is a zero-argument runtime builtin that reads the
interpreter's called-class context in current instance and static method calls;
outside method or static class context it fails with a stable unsupported-call
diagnostic.
`spl_object_id($object)` accepts current object values and returns a
process-local handle id; non-object inputs fail with the current stable
type-boundary diagnostic.
`spl_object_hash($object)` accepts current object values and returns a stable
current-subset hash derived from the handle id; exact system PHP hash formatting
is not claimed yet, and non-object inputs fail with the current stable
type-boundary diagnostic.
Missing properties, non-object targets, and non-public properties outside the
current private/protected method context still produce stable runtime
diagnostics for normal reads/writes. Public, same-class private, and protected
same-class/child instance methods can execute through `phpc run` with `$this`
bound to the receiver object handle, and inherited public constructors execute
during child instantiation. Protected constructors can execute from same-class
or child-class method context. Explicit
`parent::method(...)` and `parent::__construct(...)` calls execute from active
instance method/constructor context against the current class's parent chain
with the current `$this` object. `self::method(...)` calls execute from active
instance method/constructor context against the current class and inherited
method chain with the current `$this` object. Objects do not enforce
non-public property visibility beyond same-declaring-class private access and
class/ancestor protected access for plain read/write, `isset`/`empty`,
read-modify-write, and null-coalescing forms, compatible public/protected
property redeclarations sharing one runtime slot, or full constructor
visibility.
Objects do not expose reflection, implement
dynamic method/property names, broader `parent::`/`self::`/`static::`,
typed/default property compatibility, broader inheritance/constructor
semantics, `__clone` dispatch, destructor behavior beyond the current normal
shutdown public/no-argument slice, handle reuse behavior, or exact PHP
lifecycle behavior.
Named static method syntax through `ClassName::method(...)`,
`$object::method(...)`, `$className::method(...)`, `self::method(...)`,
`parent::method(...)`, and `static::method(...)` executes declared or inherited
visible static methods in the interpreter without binding `$this`. Dynamic
receivers through `$object::method(...)` and `$className::method(...)` use the
receiver class as the called-class context; `static::method(...)` resolves
through the active called class and forwards that called-class context into
nested calls.
`ClassName::class` returns the source-spelled class string, `self::class` and
`parent::class` resolve from the active declaring class context, and
`static::class` resolves from the active called-class context in current
instance and static method execution. Direct `ClassName::CONST`, `self::CONST`,
`parent::CONST`, and late-bound `static::CONST` resolve declared or inherited
class constants through runtime class metadata in the interpreter; typed
constants, multiple constants in one declaration, and broader dynamic
class-constant string lookup beyond loaded `ClassName::CONST` names remain
unsupported. Direct `ClassName::$prop`, `self::$prop`, `parent::$prop`,
and late-bound `static::$prop` resolve untyped static properties through
interpreter-owned class-level storage, initialize from the current
constant-expression default subset or `null`, and support direct reads/writes,
compound assignment, pre/post increment/decrement, `isset`, `empty`, `??`,
`??=`, and stable runtime diagnostics for PHP-forbidden static-property
`unset(...)`; typed static properties, dynamic names, storage-removing
static-property unset, and top-level `static::$prop` execution remain
unsupported.
Native lowering
rejects class declarations, inheritance metadata, class-name constants, class
constants, static properties, object instantiation, and object metadata
builtins with a specific object/class codegen diagnostic. Instance
object-property reads/writes, including dynamic property-name and
magic-property forms, are rejected with a separate object-property diagnostic
that names missing native object layout, property tables/slots, visibility,
magic property hooks, dynamic property policy, references/copy-on-write, and
exact native object-property errors. The narrow direct string-name
metadata-exists false-folding slice,
string/string `property_exists`/`method_exists` false-folding slice, and
string/string `is_a`/`is_subclass_of` false-folding slice remain the only
native object/class metadata exceptions. Native method-call lowering
separately rejects instance, named static, object/static-receiver, `self::`,
`parent::`, and late-static `static::` calls until generated code has native
method lookup, receiver resolution, `$this` and late-static-binding context,
argument/arity diagnostics, visibility, references/copy-on-write, and exact
native method-call errors. See `docs/OBJECT_MODEL.md` for the named
unsupported edge cases.

## Include/Require Resolution Design

The first executable include/require slice is now a narrow `require path;`,
`require_once path;`, `include path;`, and `include_once path;` subset for
local files in statement and expression position. It uses these rules:

- the interpreter carries the current file path in runtime execution context
- only paths that evaluate to PHP strings are accepted
- absolute paths resolve directly
- relative paths resolve against the directory of the file containing the
  `require` statement
- if that source-relative path is not present, relative paths fall back through
  the current `include_path` string managed by `get_include_path()` and
  `set_include_path()`; the default is `"."`, path-list entries use
  `PATH_SEPARATOR`, and empty entries are treated as `.`
- included files are parsed as PHP files with `<?php`, register top-level
  function/class declarations into the active interpreter, and execute in the
  caller scope
- `require_once` and `include_once` de-duplicate by resolved local file,
  including files first loaded through non-once `require` or `include`
- top-level `return` in an included file returns to the including file
- statement forms ignore the include return value
- expression forms return the included file's top-level return value, `1` for
  normal completion, or `true` when a `_once` construct skips an already loaded
  file
- native lowering rejects include/require until file loading, scope effects,
  and return-value behavior have explicit lowering support

Unsupported include/require behavior remains: missing-file warning/recovery for
executed `include` statements, exact PHP include-path search ordering,
process-current-working-directory behavior beyond the default `"."` include
path entry and fallback used when no source file is available, stream
wrappers, `phar://`, URL includes, autoload interaction, opcache behavior,
declaration-order dependencies such as a required file declaring `class Child
extends Base` only after requiring the base class, exact source mapping for
declarations after include, and PHP's warning-vs-fatal recovery details.

## Eval Fallback Design

`eval` is reserved by the lexer/parser today and rejected with a stable parse
diagnostic before execution. The first executable `eval` slice should use these
rules:

- parse direct `eval(<expr>)` as a special language construct, not as an
  ordinary function or dynamic callable
- require exactly one argument and evaluate that argument in the caller scope
- accept only string-valued code for the first slice
- parse the evaluated string with a dedicated eval-fragment parser entry point
  that reads a statement list without requiring a `<?php` opening tag
- execute the resulting statements against the caller's current symbol table, so
  assignments affect the same local or top-level scope that called `eval`
- let `return` inside the evaluated fragment produce the `eval(...)` expression
  value; falling off the end should produce `null`
- keep native lowering rejecting `eval` until parser re-entry, source mapping,
  caller-scope effects, and return behavior have explicit lowering support

Initial unsupported eval behavior remains: non-string eval arguments, exact
`ParseError` object semantics, source mapping for diagnostics inside evaluated
strings, functions/classes declared from evaluated code, nested eval,
include/require inside eval, references/copy-on-write interactions,
`GLOBALS`/superglobal behavior, namespaces/use declarations, opcache behavior,
and PHP's exact warning/fatal recovery details.

## Attribute Boundary

PHP attributes are currently syntax-only metadata for simple no-argument
`#[Name]` blocks; the lexer skips them before the parser sees the surrounding
declaration. Attribute blocks with constructor-style arguments such as
`#[Route('/wp-json/demo')]` stop at a dedicated lex diagnostic because
attribute argument evaluation, reflection data, target validation,
namespace-aware attribute name resolution, repeated-attribute rules,
references/copy-on-write behavior, and native lowering do not exist yet.
Ordinary `#` comments remain comments, including `# [` with whitespace before
the bracket.

## Cast Boundary

Cast expressions are AST-backed, but execution is intentionally limited to the
current scalar/null/array runtime value model. `(string)` handles scalar/null
values through the runtime echo-string conversion boundary. `(int)`/`(integer)`
handles scalar/null values through a narrow integer-cast policy for WordPress
bootstrap parsing and focused fixtures, including bounded leading-numeric
string prefixes. `(array)` handles `null`, scalars, and already-array values
only. Object-to-array property materialization, resources, exact PHP
warning/recovery behavior for leading-numeric strings, numeric grammar outside
the current prefix scanner, non-finite or out-of-range float behavior, and
native cast lowering remain explicit boundaries.

## Exception Boundary

Exception syntax is reserved by the lexer/parser today, but the boundary is no
longer a single parse-only category. The runtime seeds a metadata-only
`Exception` class so class lookup, no-argument instantiation, and user classes
extending `Exception` use the same object metadata table as declared classes.
It does not model `Throwable`, constructor state, exception methods, stack
traces, or unwinding.

Statement-form `throw expr;` and `try`/`catch`/`finally` blocks build AST nodes
so guarded WordPress compatibility code can parse and be skipped by normal
control flow. If execution reaches a throw statement, the interpreter reports a
stable unsupported runtime boundary without evaluating the throw operand. If
execution reaches a try block, the interpreter executes the normal no-throw
body path, skips catch bodies when no exception is thrown, and runs finally
bodies after normal try completion. Reached throws and other runtime errors do
not unwind into catch/finally handling in this slice.

Throw expressions, malformed try blocks, and standalone `catch`/`finally`
remain parse boundaries. Native lowering rejects `throw` statements before
emitting LLVM IR or assembly until `Throwable`/`Exception` objects, stack
unwinding, catch/finally dispatch, stack traces, and exact native error behavior
have explicit runtime and IR semantics. Native lowering rejects
`try`/`catch`/`finally` blocks through a dedicated boundary until catch type
matching, catch variable binding, finally execution during normal and
exceptional control flow, references/copy-on-write, stack traces, and exact
native try-block diagnostics exist.

## Match Expression Boundary

PHP 8 `match` expressions are reserved by the lexer/parser today and rejected
with a stable parse diagnostic before execution. They do not build AST nodes yet
because expression-form branching needs explicit semantics for strict arm
matching, default arms, exhaustiveness errors, thrown expressions inside arms,
value evaluation order, and exact native error objects. Native lowering must
continue rejecting `match` until those runtime and IR semantics exist.

## Goto Boundary

`goto target;` statements and `target:` labels build AST nodes for the current
interpreter path. Execution resolves labels in the active statement list and
lets nested statements propagate jumps outward, which covers the WordPress
UTF-8 scanner's forward error-path labels without pretending to implement every
PHP jump rule.

Exact PHP compile-time validation, duplicate label diagnostics, jumps into
nested blocks, cross-function jumps, included-file label boundaries,
interaction with future `finally` execution, and native control-flow lowering
remain explicit unsupported zones.

## Heredoc/Nowdoc Boundary

Heredoc and nowdoc syntax is tokenized directly in the lexer for the current
unindented identifier-label subset. Heredoc reuses the existing double-quoted
interpolation parts, while nowdoc emits a literal string. The lexer trims the
line ending immediately before the terminator to match PHP's runtime value.
Indentation stripping, broader quoted-label forms, malformed-label recovery,
exact diagnostics, and native lowering remain explicit boundaries.

## Fixture Tests

Fixture tests are stored as `.php` files with sibling `.stdout`, `.stderr`, and
`.exit` files. The runner strips one final editor newline from `.stdout` and
`.stderr` fixtures. A fixture that needs to assert an actual trailing newline
should include a blank final line.

When `phpc` intentionally differs from system PHP, a sibling `.phpc-only` marker
keeps the fixture in the normal runner while skipping optional system PHP
comparison.

## Extension Model

Zend extension loading is not an early target. Selected extensions will be
implemented as runtime modules with documented dependencies and semantic gaps.
Until that exists, `extension_loaded()` uses a bounded compiler/runtime
compatibility registry. It is intentionally just enough for current WordPress
bootstrap requirement checks and does not claim host extension support,
extension functions/constants, extension versions, or dynamic loading.
The current `mysqli_connect()` path is a placeholder-handle boundary used to
get past WordPress-shaped procedural connection code; it does not mark the
`mysqli` extension loaded and does not provide executable host database
behavior. PDO is currently a visibility/metadata boundary only:
`pdo`/`pdo_mysql` extension names, `PDO`/`PDOStatement` class metadata, and a
bounded public integer `PDO` constant catalog for common error-mode,
fetch-mode, and MySQL init-command checks are exposed, while `new PDO(...)`
remains an explicit unsupported host-database boundary.
