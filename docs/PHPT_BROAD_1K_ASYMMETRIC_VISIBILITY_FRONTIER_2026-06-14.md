# PHPT Broad 1k Asymmetric Visibility Frontier: 2026-06-14

Issue: `ptn-316b`

This slice refreshes the PHP 8.4 asymmetric property visibility frontier on
current `origin/master`. PTN now has bounded support for simple instance and
static `private(set)` / `protected(set)` metadata, so the old all-excluded map
is stale. The current broad 1k slice is a mixed frontier: four rows pass,
eighteen rows still fail, and seventeen rows remain classified for broader
class-metadata or assertion-mode blockers.

This is a blocker map, not a support claim. The remaining rows do not form one
credible 25-row implementation patch because they cross parser, constructor
promotion, property validation, references, unset, inheritance metadata, magic
hooks, property hooks, and typed/uninitialized property semantics.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Generated manifest:
`.runtime/phpt-baseline/20260614T035907Z/phpt-baseline-1000.txt`

Classification artifact:
`.runtime/phpt-progress/classification-20260614T035907Z.tsv`

PTN commit: `c40f0e24ed98`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 429 | 571 |

Top current exclusion buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 281 |
| `unsupported-class-metadata` | 144 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 18 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |

## Focused Frontier

Committed manifest:
`tools/phpt-asymmetric-visibility-frontier-manifest.txt`

Selection from the broad 1k manifest:

```sh
rg '^Zend/tests/asymmetric_visibility/' \
  .runtime/phpt-baseline/20260614T035907Z/phpt-baseline-1000.txt
```

Focused run:

```sh
tools/run-phpt-manifest.sh tools/phpt-asymmetric-visibility-frontier-manifest.txt
```

Result at `.runtime/phpt-progress/run-20260614T041251Z.log`:

| Selected | Runnable | Excluded | Passed | Failed | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 39 | 22 | 17 | 4 | 18 | 0 |

Classifier split for the 17 excluded rows:

| Rows | Reason |
| ---: | --- |
| 9 | typed property metadata outside PTN's modeled property declarations |
| 4 | magic method dispatch/reflection metadata |
| 3 | indirect readonly property mutation diagnostics |
| 1 | configurable `assert.exception` assertion mode |

## Runnable Outcomes

Rows that currently pass:

```text
Zend/tests/asymmetric_visibility/decrease_scope_private_private.phpt
Zend/tests/asymmetric_visibility/decrease_scope_protected_protected.phpt
Zend/tests/asymmetric_visibility/private.phpt
Zend/tests/asymmetric_visibility/protected.phpt
```

The eighteen failures split across independent implementation surfaces:

| Rows | Blocker | Representative evidence |
| ---: | --- | --- |
| 5 | Constructor-promoted properties with asymmetric visibility are not parsed/lowered. | `bug004.phpt`, `cpp_no_type.phpt`, `cpp_private.phpt`, `cpp_protected.phpt`, and `cpp_wider_set_scope.phpt` fail with `expected function parameter variable` before reaching property metadata. |
| 5 | Property declaration validation and override diagnostics are still bounded. | `decrease_scope_private_protected.phpt`, `duplicate_modifier*.phpt`, `no_type.phpt`, and `override_protected_private.phpt` either emit PTN-specific diagnostics or miss required fatal validation. |
| 5 | By-reference, indirect-write, and unset guards do not yet distinguish PHP's asymmetric property access modes. | `bug003.phpt`, `object_reference.phpt`, `reference*.phpt`, and `unset.phpt` differ on `Cannot unset` versus `Cannot modify`, indirect reference diagnostics, and copy-versus-reference behavior. |
| 1 | Inherited typed property slots and r/w cache behavior need uninitialized metadata parity. | `unshared_rw_cache_slot.phpt` prints `NULL` and write-guard errors where PHP expects uninitialized typed-property diagnostics and an empty object dump. |
| 2 | Property-hook syntax is not parsed before asymmetric visibility validation. | `virtual_get_only.phpt` and `virtual_set_only.phpt` fail with `expected semicolon` instead of PHP's get-only/set-only virtual property errors. |

## Implementation Boundary

Current native coverage proves only the bounded subset:

- `compile_asymmetric_instance_property_visibility_to_native_binary`
- `compile_asymmetric_static_property_visibility_to_native_binary`
- parser metadata tests for simple typed asymmetric properties

Moving the focused PHPT frontier requires several generic changes:

1. Parse constructor-promoted property modifiers with `private(set)`,
   `protected(set)`, and typed/no-type validation in parameter lists.
2. Normalize declaration validation and fatal diagnostic messages for
   duplicate access modifiers, no-type asymmetric properties, and invalid
   read/set visibility combinations.
3. Extend property lvalue access with separate write, unset, by-reference, and
   indirect-modification modes so the runtime can report PHP's exact access
   category while preserving COW/reference behavior.
4. Carry inherited typed-property initialization state through class metadata,
   dumps, and r/w cache paths.
5. Model property hooks and magic `__set`/`__unset` interactions before
   reopening the excluded magic/virtual rows.

The largest coherent first implementation split is constructor-promoted
properties: it owns five focused failures and would also create parser/AST
structure needed by typed-property and readonly constructor-promotion rows.
The reference/unset group should be a separate runtime-lvalue patch after that.

## Verification

```sh
cargo fmt --check
cargo test --test compile_native asymmetric
tools/run-phpt-baseline.sh --tier 1000 --classify-only
tools/run-phpt-manifest.sh tools/phpt-asymmetric-visibility-frontier-manifest.txt
```
