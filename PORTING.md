# PHP-to-Native Porting Guide

This file captures working rules distilled from full-corpus PHPT work. Keep it
in context for php-to-native/compiler work and use it to steer fixes toward
runtime root causes instead of isolated row fixes.

## Operating Rules

- Use the active full-corpus partial dashboard as the scoreboard. Historical
  full/published summaries, focused packs, and old dedup buffers are useful
  only when clearly labeled.
- Use full-corpus failures to choose clusters, then use focused manifests to
  verify specific fixes. Do not spend long proving tiny packs green unless the
  pack represents a root-cause cluster.
- Treat exclusions as visible debt, not success. A classified exclusion is not
  compiler compatibility.
- Prefer fixes that leave behind a reusable helper, subsystem rule, focused
  manifest, or dashboard classification.
- Verify on current HEAD before claiming progress. A stale corpus run can guide
  cluster selection, but it cannot prove a current fix.
- Keep PHAR/INI/process classifiers as side lanes. Main-lane work should target
  broad parser/runtime failure clusters: core arrays, objects, strings,
  references/COW, include/path behavior, fatal classes, and registry gaps.

## Common Time Wasters

- Mixing non-comparable numbers: stale summaries, live partials, focused runs,
  and old buffers.
- Treating exact-output PHPT failures as cosmetic. Output order, warning text,
  namespace order, object dumps, exception formatting, and whitespace are part
  of compatibility.
- Fixing a row without extracting the general rule that caused it.
- Checking a visible `PtnValue` directly when PHP's effective value is hidden
  behind references, wrappers, extension objects, or encoded containers.
- Repeating one-off conditionals instead of centralizing extension-specific
  semantics in helpers.
- Running expensive or duplicate verification in every lane. Workers should
  prove the affected cluster; integration should run merge-risk smoke plus the
  affected manifest unless shared runtime surfaces require broader checks.
- Printing huge diagnostics. Use exact paths, bounded `sed`, bounded `tail`,
  summary files, status TSVs, and purpose-built dashboard scripts.

## General Porting Principles

- Find the PHP semantic boundary before patching: parser, value conversion,
  object model, extension shim, stream/process runtime, or output formatting.
- Patch at the semantic boundary, not at a test-shaped branch.
- Normalize effective values before decisions. Add helpers such as
  `*_value_resolves_to_null`, `*_effective_scalar`, or `*_unwrap_param` instead
  of repeating direct type checks.
- Separate escaping contexts. XML text, XML attributes, raw XML, HTML text,
  INI, URL/query strings, and shell/process strings have different rules.
- Model php-src behavior, including historical quirks, instead of idealized PHP.
- Centralize extension quirks. SOAP, streams, reflection, generators, SPL, PDO,
  PHAR, XML, and sessions each need local helper layers.
- Assume PHPT exactness is intentional until proven otherwise.

## C-to-Rust Runtime Patterns

- PHP values are not just tagged values. References, copy-on-write, ordered
  arrays, dynamic properties, magic methods, resources, and internal handlers
  all affect visible behavior.
- Arrays are ordered maps. Preserve insertion order, integer-string key
  coercion, next-auto-index behavior, packed/mixed distinctions, and mutation
  behavior during iteration.
- Null, false, empty string, and zero are distinct until php-src explicitly
  coerces them. Match the conversion point.
- String conversion can warn, throw, call `__toString`, inspect enum backing
  values, or reject arrays/resources depending on context.
- Object identity and handlers matter. Internal classes are not plain structs:
  constructor state, native data, dynamic properties, inheritance checks,
  cloning, destructors, and method dispatch are compatibility surface.
- Warnings, notices, exceptions, and fatals are observable. Match class, message,
  timing, side effects, and line behavior.
- Iteration mutation rules matter for arrays, generators, iterators, destructors,
  and `finally` blocks.
- Resource lifetime is observable: stream close, process handles, generator
  cleanup, shutdown order, object dtor/free order, and file descriptor state.
- Serialization is exact behavior. XML/SOAP, `var_dump`, `serialize`, JSON,
  Reflection strings, traces, and INI output require byte-level compatibility.
- When uncertain, read the php-src C path and extract the smallest semantic rule
  into a runtime abstraction.

## SOAP Lessons From Current Work

- XML text escaping is not XML attribute escaping. In SOAP text nodes, escape
  `&`, `<`, and `>`, but do not escape `"` as `&quot;`.
- Null handling must unwrap SOAP containers. `NULL`, `SoapParam(NULL, ...)`,
  and `SoapVar(NULL, XSD_*)` must resolve to nil where php-src emits
  `xsi:nil="true"`.
- Exact namespace order can matter in PHPT. For nil SOAP requests, expected
  output may require `xmlns:xsi` before `xmlns:xsd`.
- Encoded nil SOAP responses should emit nil as php-src does; do not add
  semantically redundant `xsi:type` if expected output omits it.

## Cluster Workflow

1. Read the active full-corpus partial dashboard and identify top failing
   clusters by root cause and subsystem.
2. Pick a cluster that plausibly unlocks many rows or removes a recurring
   runtime semantic mismatch.
3. Inspect only bounded diffs/log snippets for representative rows.
4. Patch a subsystem helper or semantic boundary.
5. Verify the focused representative manifest on current HEAD.
6. Run a small broader regression manifest for the same subsystem when the
   touched surface is shared.
7. Commit only the verified change and record which cluster it addresses.
