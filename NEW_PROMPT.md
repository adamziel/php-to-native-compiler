# Prompt: Build a Full-Compatibility PHP-to-Native Binary Compiler

You are implementing a real PHP-to-native binary compiler, not a demo,
transpiler, PHPT-row patcher, or compatibility toy.

The target is full PHP compatibility from day one, with only the most inherently
dynamic features allowed to remain interpreter/runtime fallback zones at first:
`eval`, variable variables, runtime-generated symbols, and similarly
self-modifying behavior. Everything else should be designed as part of one
generic compiler architecture, not as isolated exact-shape fixes.

## Mission

Build a compiler that accepts real PHP programs and emits native executables.
The compiler must be designed around PHP semantics first:

- PHP values are boxed, typed dynamically, and retain PHP conversion rules.
- Arrays are ordered maps with integer/string key behavior, references,
  copy-on-write, iteration semantics, and mutation visibility.
- Objects use PHP class metadata, visibility, inheritance, traits, interfaces,
  magic methods, late static binding, destructors, exceptions, and reflection.
- Calls support normal calls, dynamic calls, closures, methods, static methods,
  named arguments, variadics, by-reference parameters, default values, and
  internal functions.
- Errors, warnings, notices, fatal errors, exceptions, and stack traces must be
  represented generically, not hard-coded for individual tests.
- Native code generation must preserve PHP observable behavior, including
  evaluation order, destructor timing, reference identity, output, globals,
  superglobals, resources, streams, filesystem behavior, and extension APIs.

The implementation must prioritize a coherent generic runtime and compiler
pipeline over narrow PHPT-specific fixes.

## Non-Negotiables

- Do not implement compatibility by exact-shaping individual PHPT expected
  outputs.
- Do not special-case test filenames, line texts, or one-off output strings.
- Do not grow a pile of unrelated row-specific patches.
- Do not claim support for a feature unless it is implemented through generic
  semantics and proven by tests.
- Do not use regex parsing as a substitute for PHP parsing.
- Do not fake native compilation by only interpreting everything.
- Do not let a single failing edge case block broader architectural progress.
- Do not build an architecture that cannot naturally support full PHP later.

## Architecture

Design the system as these layers:

1. Parser and AST
   - Parse PHP source into a real syntax tree with source locations.
   - Preserve enough source metadata for diagnostics, stack traces, reflection,
     doc comments, attributes, and error reporting.

2. Semantic Model
   - Build symbol tables, class tables, function tables, namespace resolution,
     trait composition, inheritance graphs, interface checks, visibility rules,
     and extension metadata.
   - Model dynamic uncertainty explicitly instead of rejecting it prematurely.

3. PHP Runtime Model
   - Implement boxed `zval`-style values.
   - Implement arrays as PHP ordered maps.
   - Implement references and copy-on-write as first-class runtime concepts.
   - Implement object storage, property tables, class metadata, magic dispatch,
     reflection metadata, resources, exceptions, and error channels.
   - Keep runtime behavior generic and reusable from both interpreter and native
     code paths.

4. IR
   - Lower PHP AST into a PHP-aware intermediate representation.
   - IR must preserve evaluation order, reference behavior, dynamic dispatch,
     destructors, temporaries, and exception edges.
   - Unsupported dynamic behavior should lower to explicit runtime fallback
     calls, not disappear.

5. Optimization
   - Add optional specialization only after generic semantics are correct.
   - Optimizations must be guarded by runtime checks and deopt/fallback paths
     where PHP dynamism requires them.

6. Native Code Generation
   - Emit native binary code that calls the shared runtime for PHP semantics.
   - Start with conservative boxed operations, then specialize when safe.
   - The generated executable must not depend on source-level test knowledge.

7. Extension Compatibility
   - Implement internal functions and classes as generic extension modules.
   - Extension APIs should share common argument parsing, type coercion,
     diagnostics, resource management, and reflection metadata.

8. Dynamic Fallback
   - `eval`, variable variables, runtime symbol-table mutation, and similarly
     hard dynamic behavior may route to a runtime fallback.
   - Fallbacks must preserve program state and observable behavior.
   - Fallback boundaries must be explicit, documented, and tested.

## Testing Strategy

Use tests to prove semantics, not to shape implementation around expected text.

- Run PHP’s PHPT suite as the primary compatibility signal.
- Add unit tests for generic runtime semantics: values, arrays, references,
  copy-on-write, objects, calls, exceptions, resources, and diagnostics.
- Add differential tests against PHP for broad behavior families.
- Track pass counts, regressions, unsupported features, and fallback coverage.
- When a PHPT fails, diagnose the generic semantic gap first.
- Fix the architecture or runtime primitive that explains the failure cluster.
- Prefer one semantic fix that moves many rows over many one-row patches.

## Delivery Discipline

Every completed feature needs:

- Generic implementation code.
- Tests proving the general behavior.
- Native-code exercise path, not only interpreter execution.
- Documentation of supported behavior and remaining fallback zones.
- A clear list of unsupported dynamic edges.

Progress is measured by real compatibility and executable native behavior, not
by local patch inventory or narrow exact-output wins.

## Immediate Implementation Direction

Start by building the smallest complete vertical slice of the real architecture:

- Parse a useful PHP subset with real source metadata.
- Represent all runtime values through the same boxed PHP value model intended
  for full compatibility.
- Implement ordered arrays, object/class metadata, function calls, diagnostics,
  and exception flow as reusable runtime primitives.
- Lower supported PHP into PHP-aware IR.
- Generate native code that calls the runtime for generic PHP operations.
- Keep dynamic fallback hooks in the design from the start.

The first native binary does not need to support every PHP construct, but every
implemented construct must use the same architecture that will scale to full
compatibility. Avoid temporary designs that would need to be thrown away.

## Definition of Success

The project is on the right path when each new compatibility improvement:

- comes from a generic semantic capability;
- improves multiple realistic programs or PHPT clusters;
- is available to native binaries;
- does not require output-specific hacks;
- makes the compiler more capable of supporting arbitrary PHP code.

Build the compiler as if full PHP compatibility is the product, not as if the
test suite is the product.
