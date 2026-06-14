# PHPT Broad 1k Class Declaration Frontier: 2026-06-14

Issue: `ptn-zzr2`

This slice maps the broad 1k PHPT rows blocked on class declaration surfaces
that PTN does not yet model generically: interfaces, interface implementation
checks, traits, trait composition, and anonymous classes.

This is a blocker map, not a support claim. Reopening these rows requires
parser, AST, class-table, inheritance, interface, trait, anonymous class,
autoload, reflection, and runtime dispatch work. That is not credible as a
narrow PHPT-row patch.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-zzr2-baseline-current
```

Generated manifest:
`.runtime/ptn-zzr2-baseline-current/20260614T023205Z/phpt-baseline-1000.txt`

Classification artifact:
`.runtime/phpt-progress/classification-20260614T023205Z.tsv`

PTN commit: `0e868d3b731a`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 431 | 569 |

Top blocker counts:

| Bucket | Rows |
| --- | ---: |
| PHP attributes | 141 |
| magic method dispatch/reflection metadata | 69 |
| call-site/array unpacking | 34 |
| trait declarations | 25 |
| interface declarations | 23 |
| non-public property visibility metadata | 19 |
| configurable `assert.exception` assertion mode | 17 |
| interface implementation checks | 15 |
| anonymous class syntax | 15 |
| `memory_limit` parsing/enforcement | 15 |

## Focused Frontier

Committed manifest:
`tools/phpt-class-declaration-frontier-manifest.txt`

Selection from `classification-20260614T023205Z.tsv`:

```sh
awk -F'\t' '$3 ~ /interface|trait declarations|anonymous class/ {print $1}'
```

Focused classifier result after `ptn-0pys`:

Result at `.runtime/phpt-progress/run-20260614T040446Z-manifest.log`:

| Selected | Runnable | Passing | Excluded |
| ---: | ---: | ---: | ---: |
| 78 | 1 | 1 | 77 |

Current classified blockers:

| Class declaration blocker | Rows |
| --- | ---: |
| unsupported language / class-like runtime surfaces | 73 |
| unsupported class/reflection metadata | 4 |

Runnable row:

```text
Zend/tests/bug32427.phpt
```

Adjacent assigned `ptn-0pys` raw PHPT cluster:

| Selected | Runnable | Passing |
| ---: | ---: | ---: |
| 6 | 6 | 6 |

Passing rows:

```text
Zend/tests/class_exists_003.phpt
Zend/tests/constants/class_constants_004.phpt
Zend/tests/objects/objects_012.phpt
Zend/tests/objects/objects_013.phpt
Zend/tests/objects/objects_014.phpt
Zend/tests/objects/objects_018.phpt
```

Path concentration:

| Path group | Rows |
| --- | ---: |
| `Zend/tests/attributes` | 34 |
| `Zend/tests/anon` | 20 |
| `Zend/tests/ArrayAccess` | 10 |
| `Zend/tests/backtrace` | 4 |
| other Zend rows | 8 |
| `ext/standard/tests` | 1 |
| `tests/basic` | 1 |

## Why This Is A Blocker

The remaining rows share a semantic root: PTN now has bounded class-like
metadata for interfaces, traits, abstract/final classes, interface constants,
and duplicate/non-interface implementation diagnostics, but not the full
declaration graph needed for complete class metadata. Generic support still
needs:

- trait composition with aliases and precedence rules, and anonymous class
  expressions;
- validation for full interface contracts, method signature compatibility,
  trait conflict resolution, and trait adaptation;
- anonymous class naming, source metadata, constructor dispatch, inheritance,
  closure binding, and reflection exposure;
- runtime checks for `instanceof`, `is_subclass_of()`, `class_implements()`,
  `class_uses()`, `ArrayAccess`, `Iterator`, `Traversable`, and related
  built-in interface behavior;
- integration with autoload, attributes, override validation, backtraces, and
  object string/magic method metadata.

Treating these rows as runnable today would turn absent class-graph semantics
into noisy parser/runtime failures. Keeping this cluster explicitly mapped
makes the broad baseline more actionable until the generic class model lands.

## Representative Rows

```text
Zend/tests/ArrayAccess/ArrayAccess_indirect_append.phpt
Zend/tests/ArrayAccess/bug69955.phpt
Zend/tests/anon/001.phpt
Zend/tests/anon/014.phpt
Zend/tests/anon/gh16067.phpt
Zend/tests/attributes/override/001.phpt
Zend/tests/attributes/override/020.phpt
Zend/tests/autoload/bug49908.phpt
Zend/tests/backtrace/bug69180-backtrace.phpt
ext/standard/tests/array/array_fill_object.phpt
tests/basic/bug73969.phpt
```

## Verification

```sh
cargo fmt --check
cargo test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-zzr2-baseline-current
tools/run-bounded-phpt.sh tools/phpt-class-declaration-frontier-manifest.txt
```
