# PHPT Attribute Blocker Map - 2026-06-13

## Scope

Broad 1k tier slice:
`.runtime/phpt-baseline/20260613T144540Z/attributes-rows-1000-slice.txt`

Source filter:
`Zend/tests/attributes/**` rows selected by
`.runtime/phpt-baseline/20260613T144540Z/phpt-baseline-1000.txt`.

## Classification Before This Change

Classifier-only sweep on `origin/master` before the attribute syntax rule:

| Category | Rows |
| --- | ---: |
| Total selected | 204 |
| Runnable | 137 |
| Unsupported language | 59 |
| Unsupported extension | 8 |

The 137 runnable rows all require PHP attribute syntax and related parser,
metadata, and Reflection behavior. PTN does not currently model attributes in
the parser, AST, semantic metadata, class/function/property/parameter tables, or
Reflection APIs, so running these rows as ordinary executable semantic coverage
produces parser/runtime noise rather than a useful compatibility signal.

## Classification After This Change

Attribute syntax is now classified generically when the PHPT `--FILE--` section
contains PHP attribute syntax (`#[...]`). This is intentionally syntax based,
not path based, so future attribute rows outside `Zend/tests/attributes` are
classified by the same rule.

Focused PHPT manifest result after this change:

| Category | Rows |
| --- | ---: |
| Total selected | 204 |
| Runnable | 9 |
| Unsupported language | 187 |
| Unsupported extension | 8 |

This moves 128 broad-tier rows from runnable failures into explicit
unsupported-language classification. The 9 remaining runnable rows do not
contain attribute syntax in `--FILE--`; they exercise attribute-adjacent APIs
and built-ins such as `ReflectionAttribute`, `Reflection*::getAttributes()`,
`Attribute::TARGET_*`, `Deprecated`, and `NoDiscard`.

## Real Implementation Blockers

Full attribute support is a multi-surface feature, not a one-row parser tweak:

- Lexer/parser support for `T_ATTRIBUTE` groups on declarations and parameters.
- AST storage for attribute names, arguments, nesting, and source locations.
- Semantic metadata attached to classes, functions, methods, properties,
  constants, and parameters.
- Constant-expression evaluation for attribute arguments.
- Runtime and Reflection APIs for `ReflectionAttribute` and attribute reads.
- Validation of attribute targets, repeatability, and declaration classes.

Until those surfaces exist, the broad PHPT baseline should count attribute rows
as a known unsupported language cluster instead of runnable failures.
