# COW Coverage Matrix

This matrix is the consolidation surface for PHP array copy-on-write
provenance. It classifies coverage by generalized mechanism, not by fixture
milestone. New COW work should extend one of these mechanisms instead of adding
another case-specific branch.

Mechanism keys:

- `RPR`: runtime provenance resolver. `ArrayCopySource` roots resolve through
  runtime lvalue handles for local, static, global, object-property, alias, and
  runtime-cell sources before alias rehydration, mirror, promotion, import
  remapping, or writeback.
- `DMB`: dynamic mutation boundary. Writes, unsets, `??=`, compound writes,
  reference assignment, alias writes, reference promotion, and property
  replacement invalidate or sync copied-source metadata at the mutation
  boundary.
- `CCA`: callback call-frame adapter. Direct calls, closures, dynamic string
  callbacks, array-callables, `call_user_func()`, `call_user_func_array()`,
  by-value imports, by-reference bindings, and reference returns move
  copied-source metadata into and out of callee frames.
- `GAP`: unsupported or intentionally narrower behavior. Add executable
  support only when a generalized mechanism can own it.

Literal array transforms are deliberately not a fourth mechanism. Explicit
branches around `evaluate_array_transform_literal_call_for_assignment` are
proof coverage only. Consolidation means extracting reusable provenance remap
primitives under `RPR`, then letting `CCA` consume the same portable paths at
call boundaries.

## Ownership

| Surface | Owner | Evidence | Intake rule |
| --- | --- | --- | --- |
| Reachable source roots: locals, globals, statics, aliases, public properties, visibility-valid non-public properties, runtime cells | `RPR` | `RuntimeAliasLvalueHandle`, `ResolvedArrayCopySource`, `ArrayCopySourceStorageIdentity`, `runtime_alias_lvalue_handles_for_array_copy_source()` | Accept changes that route root matching through resolver identities. |
| Source ancestry and transform/import remaps | `RPR` | `array_copy_source_relative_path_from_ancestor()`, record impact APIs, representative literal-transform fixtures | Accept reusable remap primitives; reject more one-off wrapper branches. |
| Assignment, compound assignment, unset, `??=`, reference assignment, alias write, reference promotion, property replacement | `DMB` | `HolderStorageMutationBoundary`, pre/post replacement hooks, mutation target impacts | Accept changes that capture the old storage identity before replacement and reuse shared invalidation/rehydration. |
| Dirty metadata sync, cleanup, and source invalidation | `DMB` | copied-source record enumeration/removal and dirty-source hooks | Accept shared record removal/impact paths; reject per-location cleanup duplication. |
| Direct calls, closures, dynamic calls, string callbacks, object/static array-callables | `CCA` | `CallFrameArgumentBindings`, `call_user_function_with_call_frame()` | Accept call shapes that build the shared frame carrier. |
| `call_user_func()` / `call_user_func_array()` argument containers and reference returns | `CCA` | `call_user_func_array_frame_from_value_sources()` and reference-return frame import/export | Accept changes that normalize imports/exports through the adapter. |
| Untracked containers, unreachable runtime cells, string COW, native COW lowering | `GAP` | Current support docs and native lowering rejections | Keep explicit unsupported gaps until a shared mechanism exists. |

## Representative Proof

Milestones 2294-2317 are proof inventory for dense reindexing,
preserve-key selection, nested prefixes, truthy/key filtering,
value-derived keys, duplicate-key overwrite, padding-source import, and
replacement. They are not precedent for adding every wrapper spelling.

Current representative fixture intake:

- Keep `milestone2294` as the narrow proof for `array_merge(array(...), ...)`
  integer reindexing and string overwrite remaps.
- Defer broad `2297`-`2317` fixture intake until transform remapping is
  extracted into a reusable `RPR` primitive.
- When accepting future fixtures, choose the smallest representative directory
  that proves a new `RPR`, `DMB`, or `CCA` primitive.

## Current Gaps

- Transform-remap extraction/shared engine.
- Callback-driven transforms.
- Multi-array `array_map(null, ...)` zipping.
- Value-comparison filters once executable.
- Untracked containers and unreachable runtime cells.
- String COW.
- Native reference/COW lowering.
