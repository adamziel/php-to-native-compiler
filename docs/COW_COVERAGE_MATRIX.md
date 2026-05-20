# COW Coverage Matrix

This matrix is the regression lane's consolidation surface for PHP array
copy-on-write provenance. It classifies each COW dimension by the generalized
mechanism that should cover it, rather than by fixture milestone.

Generalized mechanism keys:

- `RPR`: runtime provenance resolver. `ArrayCopySource` roots resolve through
  runtime lvalue handles for local/static/global/object-property/runtime-cell
  sources before alias rehydration, mirror, promotion, import remapping, or
  writeback.
- `DMB`: dynamic mutation boundary. Writes, unsets, `??=`, compound writes,
  reference assignment, and property replacement invalidate or sync copied
  source metadata at the mutation boundary.
- `CCA`: callback call-frame adapter. Direct calls, closures, dynamic string
  callbacks, array-callables, `call_user_func()`, `call_user_func_array()`,
  by-value imports, by-reference bindings, and reference returns move copied
  source metadata into and out of callee frames.
- `GAP`: unsupported or intentionally narrower behavior. Add executable
  support only when a generalized mechanism can own it.

Literal array transforms are deliberately not a fourth mechanism. The current
implementation has many explicit helper branches around
`evaluate_array_transform_literal_call_for_assignment`; those branches are
accepted proof coverage only. Consolidation means extracting reusable
provenance remap primitives under `RPR`, then letting `CCA` consume the same
portable paths at call boundaries.

## Source Roots

| Cell | Current mechanism | Audit |
| --- | --- | --- |
| local | `RPR` plus `CCA` | Covered when the local carries copied-source metadata or aliases imported from a source-aware evaluation path. Plain untracked locals remain detached values. |
| global | `RPR` | Covered through global alias roots and `$GLOBALS`-derived paths where the runtime can form a lvalue handle. |
| static | `RPR` | Covered through static alias roots, including dirty static copied-source preservation. |
| alias | `RPR` plus `DMB` | Covered when alias metadata proves a real reference or materializable array-offset alias. Alias writes must sync or detach metadata, not rebuild provenance statically. |
| public object property | `RPR` plus `DMB` | Covered through public object-property alias roots and runtime-cell handle discovery for initialized public properties. |
| non-public property | `RPR` in context plus `DMB` | Covered only inside a visibility-valid method context where context object-property alias roots can be built. Cross-context recovery is a `GAP`. |
| runtime cell | `RPR` | Covered for observable reference cells resolved back to visible static/global/public-property handles. Unreachable cells without handles are a `GAP`. |
| literal transform | `RPR`, otherwise `GAP` | Covered only for documented assigned literal wrappers whose copied-source paths can be remapped into a reachable assignment/import root. The current implementation is branch-heavy and should be consolidated before adding adjacent wrappers. |

## Transform And Import

| Cell | Current mechanism | Audit |
| --- | --- | --- |
| array literal | `RPR` plus `CCA` | Expression-position array literals can carry per-entry copied-source paths and by-reference elements. `RPR` owns the portable path metadata; `CCA` owns call-frame import/export when the literal becomes an argument container. |
| `array_merge` | `RPR` | Covered for assigned literal operands, including integer reindexing and string overwrite. This is a representative rekey/overwrite remap, but today it is still one explicit branch. |
| `array_values` | `RPR` | Covered for assigned literal operands and direct literal element reads. This is a representative dense integer reindex remap, but today it is still one explicit branch. |
| selected nested path | `RPR` | Covered when the selected path is represented as source suffix keys and remains reachable through a runtime handle. |
| named argument remap | `CCA` | Covered for supported `call_user_func_array()` named-value setup, with documented gaps for variadic named forms and positional-after-named forms. |
| positional remap | `CCA` | Covered for direct arguments, literal/stored/general `call_user_func_array()` arrays, and variadic by-value target paths in the supported subset. |

Consolidation note: `array_reverse`, `array_slice`, `array_chunk`,
`array_filter` without callback, `array_map(null, array(...))`,
`array_intersect_key`, `array_diff_key`, `array_column`, `array_pad`,
`array_combine`, and `array_replace` are covered today as explicit
literal-transform branches. They prove useful remap classes, but further
same-shape fixtures should be rejected unless they expose a missing reusable
`RPR` remap primitive: dense reindex, preserve-key select, nested prefix,
truthy/key filter, value-derived key, duplicate-key overwrite, padding-source
import, or source invalidation.

## Call Boundaries

| Cell | Current mechanism | Audit |
| --- | --- | --- |
| direct call | `CCA` | Source-aware by-value argument evaluation imports copied-source bindings into the callee frame. |
| closure | `CCA` | Covered for supported closure invocation and callback dispatch where closure bodies run through the source-aware path. |
| string callback | `CCA` | Covered for supported user-function string callbacks through `call_user_func()` and `call_user_func_array()`. |
| object/static array-callable | `CCA` | Covered for supported public object/static array-callables, including method-body dynamic array-callables. |
| `call_user_func` | `CCA` | Covered for supported value and reference-return dispatch. Builtin/reference-param semantics remain bounded. |
| `call_user_func_array` | `CCA` | Covered for literal, stored, and supported general expression argument arrays where copied-source entries can be recovered. Unsupported named/variadic mixes stay `GAP`. |
| by-value | `CCA` plus `RPR` | Covered when argument evaluation returns `Value + ArrayCopySource`; plain arrays remain detached. |
| by-reference | `CCA` plus `DMB` | Covered when the argument can bind to a materialized caller cell or supported alias group. Unsupported by-reference targets remain `GAP`. |
| reference return | `CCA` plus `RPR` | Covered for direct/function/method/callback reference-return sources that resolve selected leaves through runtime handles. Unsupported source expressions remain `GAP`. |

## Mutations

| Cell | Current mechanism | Audit |
| --- | --- | --- |
| assignment | `DMB` plus `RPR` | Covered for supported direct/static/global/object-property/array-offset assignment surfaces, with copied-source mirror or invalidation. |
| compound assignment | `DMB` | Covered where dynamic lane routes the target through the same mutation boundary. Needs rejection if a patch adds special post-write provenance repair. |
| unset | `DMB` | Covered for supported direct and object-property/array paths by invalidating or detaching affected aliases. Unsupported unset operands stay `GAP`. |
| `??=` | `DMB` | Covered where the target is a supported mutation target; should share assignment invalidation, not a separate COW branch. |
| reference assignment | `DMB` plus `RPR` | Covered when a concrete alias group/reference cell can be promoted and synced. Object/ArrayAccess dimensions without concrete lvalues remain `GAP`. |
| alias write | `DMB` | Covered through alias sync, detached path tracking, and dirty copied-source metadata. |
| reference promotion | `RPR` plus `DMB` | Covered when copied source and alias metadata prove the same selected leaf and can materialize a `PhpReferenceCell`. |
| property replacement | `DMB` | Covered for tracked property roots by invalidating stale sources and preserving detached old cells where proven. Untracked containers are `GAP`. |

## Export And Cleanup

| Cell | Current mechanism | Audit |
| --- | --- | --- |
| normal return | `CCA` | Covered when return execution can carry an optional `ArrayCopySource` or literal copied-source paths. |
| reference return | `CCA` plus `RPR` | Covered for selected supported reference-return targets; rejects unsupported source expressions instead of fabricating native references. |
| by-reference writeback | `CCA` plus `DMB` | Covered through reference binding writeback, object alias transfer, detached alias writeback, and array-copy-source alias writeback. |
| dirty metadata sync | `DMB` | Covered for public/object/literal copied-source metadata that survives callee/method-body writes. |
| source invalidation | `DMB` | Covered when source root/path replacement is observable. Broad graph invalidation for untracked dynamic containers is `GAP`. |

## Current Gap List

- General transform engine: the assigned literal-transform path still grows one
  builtin branch per wrapper. Future work should extract reusable remap
  primitives before adding more wrappers from the same class.
- Callback-driven transforms: `array_map($callback, ...)` and
  callback-bearing `array_filter()` need a callback-aware transform adapter,
  not static literal remapping.
- Variadic null-callback zipping: multi-array `array_map(null, ...)` needs a
  generalized positional zip remapper.
- Value-comparison filters: `array_diff`, `array_intersect`, and
  `array_unique` should wait until array-valued comparison behavior is
  executable enough to prove parity.
- Padding-value propagation: `array_pad()` currently remaps original entries
  but does not propagate copied-source metadata from the padding value.
- Untracked containers and unreachable runtime cells remain explicit gaps.
- String COW and native lowering remain out of scope for the current
  interpreter COW lanes.

## Regression Lane Intake Rule

Reject a proposed duplicate fixture when it only repeats a covered matrix
cell. Accept it only if it proves one of these:

- a new source root can be resolved by `RPR`;
- a mutation shape reaches the shared `DMB` instead of a bespoke branch;
- a call boundary imports or exports copied-source metadata through `CCA`;
- a literal transform needs a new reusable `RPR` remap primitive;
- an unsupported `GAP` is now implemented with code, tests, CLI path,
  documentation, and named remaining edge cases.

When reviewing lane patches, record whether they reduce special cases. Patches
that only add another branch to `evaluate_array_transform_literal_call_for_assignment`
should be treated as regression proofs, not consolidation, unless they extract
or reuse a shared remap primitive.
