# PHP Native Compiler Progress

Updated: 2026-05-27 23:17 CEST
Evaluation marker: `20260526T040843Z`
Strategy evaluator marker: `20260526T040843Z`

Accounting rule: progress counts only generalized, tested, committed, and
pushed primary work. Dirty WIP, lane-local claims, candidate artifacts,
review-only work, probe-only commits, docs-only substitutions, and broad tests
without focused proof do not increase capability bars.

Progress bars use 20 slots. One `#` is 5%. Percentages are intentionally
coarse; they do not move for narrow scaffolding unless the integrated behavior
changes the roadmap position.

## Executive Read

Overall integrated-roadmap progress: **93%** `[##################--]`

Selected executable PHP semantics: **99%** `[###################-]`

Latest accounted source capability: `c8cc77dc` rejects generated-C trait alias
collisions where an alias would silently overwrite another effective trait
method name. Trait composition now tracks composed method source identity, so
alias-to-existing-method and visibility-changing same-name alias cases stop at a
semantic blocker instead of publishing wrong native metadata or dispatch.
Broader array COW/reference edges, foreach by-reference lifetime, include
exception/diagnostic propagation, visibility-context rules, remaining
trait precedence parity, cleanup ordering, and non-C backend parity remain
blocked.

Recent source commit `c8cc77dc` advances trait conflict safety without adding
method-name production ladders, one-trait fixture lowering, or generated-C
substring gates. Trait composition now records each composed name's declaring
trait and original source method key, rejects aliases that collide with later
effective methods from the same trait, and rejects same-name aliases when they
would mutate visibility. Focused gates covered trait semantic unit tests,
native callable metadata blocker coverage, a runtime-style `phpc run` probe with
the expected collision diagnostic, adjacent trait alias/static metadata filters,
formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-trait-conflict-714973d8-20260527.gates.log`
sha256 `a61e9f779cc4c65ae1cf3c43775b924a238e0398217df10579163862358edb0a`.

Recent source commit `1186e46a` advances fatal cleanup/destructor parity without
adding destructor-name production ladders, source-shape recognizers, or
fixture-only lowering. Generated-C scope cleanup now calls
`phpc_native_request_destructor_finalizers_finalize_with_callable_table()` before
freeing request finalizers on fatal cleanup, then unwinds output buffers after
destructors have emitted observable output. Focused gates covered linked native
fatal cleanup output/error behavior, adjacent destructor finalization programs,
native output-buffer shutdown unwind, shutdown function builtin regressions,
formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-destructor-cleanup-8cb864fa-20260527.gates.log`
sha256 `974f1fef9124c482fd2069dc6e7edc5f03f4195a298b22181b45cbf5a761222b`.

Recent source commit `bbffaab4` advances direct array COW without adding
ArrayAccess overlap, object-property owner overlap, source-shape recognizers,
or fake reference writeback. Generated C now clones a direct RHS
`NativeArrayHandle` when storing another direct variable, so later nested
lvalue owner writes target a distinct array handle. Focused gates covered
generated-C source proof, linked executable direct-array copy/nested-write COW,
adjacent nested array lvalue writes, active symbol array lvalue reference-owner
paths, compile checking, formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-array-cow-2af4dcf7-20260527.gates.log`
sha256 `b1e4e8db9961ef970d2d12940212444b7986bd3a21a3d07f7ca23275c51190a4`.

Recent source commit `38997317` advances object-property reference/COW owner
coverage without adding ArrayAccess overlap, call-result owner shortcuts,
source-shape recognizers, fake reference cloning, or value-copy reference
writeback. The runtime now exposes
`phpc_native_value_public_property_bind_reference_with_diagnostic_and_free()`,
and generated C routes supported declared public property reference assignments
and owner commits through real property reference cells while keeping unsupported
typed/magic/non-public/static/non-direct cases on existing boundaries. Focused
gates covered the runtime bind ABI, generated-C source proof, linked executable
object-property reference assignment, nonlocal/dynamic property owner
regressions, typed-property boundaries, symbol reference assignment paths, a
nonzero property-held ArrayAccess owner boundary regression, compile checking,
formatting, and diff checks. One stale zero-test filter was corrected before
accounting. Primary gate log:
`state/logs/phpc-primary-object-property-ref-f5327931-20260527.gates.log`
sha256 `0dea01e422d2ef5beebf2375b9ce7f26d0580ea3d6620b1041ffdeb5ad1ba050`.

Recent source commit `be73c85b` advances request superglobal reference
arguments without adding request-name production ladders, fake array-offset
writeback, or callable-array carrier shortcuts. Generated C now materializes
root and path request-state references for by-reference call operands and lets
the existing runtime dynamic `call_user_func()` by-reference bridge admit those
direct request-superglobal expressions without forcing a globals symbol table.
Focused gates covered generated-C source proof, linked executable mutation of
root/key/path request superglobals, adjacent `call_user_func` by-reference
regressions, compile checking, formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-superglobal-ref-7f50db6c-20260527.gates.log`
sha256 `5ac37667f82be854a0246816a9cf7c6231d50d119147a3645610e9bdc8627eda`.

Recent source commit `0fe08856` advances compiled include control transfer
without adding include-path tables, source-shape recognizers, or fixture-only
lowering. Include-unit fatal/exit returns now carry
`PHPC_NATIVE_INCLUDE_RESULT_TERMINATE`, and generated callers test that tag
instead of using nonzero exit status as the control-transfer signal. Focused
gates covered generated-C source proof, linked executable proof that
`exit()` from an included compiled unit suppresses caller code after the
include, the full native include/require boundary suite, compile checking,
formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-include-control-flow-aaaa0cef-20260527.gates.log`
sha256 `12b40fa243e6754b3c1e371d13f1b2505ae6b1455ca848d85c6c7e4b57700da5`.

Recent source commit `2f6cbc0d` advances declared-class metadata visibility
without adding class/member-name ladders, source-shape recognizers, or
`method_exists()` behavior changes. Runtime user-class metadata lookup now
tracks the root class during hierarchy walks and suppresses private ancestor
properties only for `property_exists()` checks; generated C keeps using the
shared metadata ABI. Focused gates covered runtime metadata filtering, linked
native `property_exists()` proof with a `method_exists()` private-method
regression, adjacent user-class metadata registry execution, compile checking,
formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-property-visibility-c42deeb9-20260527.gates.log`
sha256 `c6586cd9ac64b5f8fba2ad2d26203ed139150681c3e212e48c3af36ec2745ffb`.

Recent source commit `6dc005ac` advances trait alias metadata without adding
method-name production ladders, raw generated-C member scans, one-trait fixture
lowering, or substring-only production gates. Interpreter class registration
now preserves `method.is_static` when publishing composed trait method metadata,
so public static aliases stay callable/static while protected aliases remain
hidden from public method lists. Focused gates covered object-model static
trait alias metadata, linked native static-alias execution, trait object-model
regressions, trait native-link regressions, compile checking, formatting, and
diff checks. Primary gate log:
`state/logs/phpc-primary-trait-alias-static-f9b829bc-20260527.gates.log`
sha256 `e2351729f3e08011599f35469155285fa179a45c7ec323035d6106bb87d009f5`.

Recent source commit `170e5766` advances reference/COW owner coverage without
adding object-name ladders, source-shape production recognizers, fake
reference cloning, or `offsetSet()` writeback. Generated C now records
reference-return source-call facts, materializes reference-call ArrayAccess
owners through the existing source-call reference path, clones only the native
reference value for reading, and commits replacements back through the same
reference handle. Focused gates covered reference-return ArrayAccess subject
source and linked execution, adjacent ArrayAccess reference owner regressions,
reference-return owner-stack regressions, source-call reference alias
regressions, method spread regressions, compile checking, formatting, and diff
checks. Primary gate log:
`state/logs/phpc-primary-ref-cow-owners-9c0d2c89-20260527.gates.log` sha256
`75a118265dc0e5bd851a3218def315be709176fb7bf32632a93ee63b71dcf6f0`.

Recent source commit `7837d68e` advances dynamic callable by-reference
lowering without adding callable-name ladders, source-shape production
recognizers, by-value fallbacks for reference slots, or fake writeback.
Generated C now preflights `call_user_func()` callback contracts with direct
argument shapes, reuses the existing source-call reference binding operands for
supported direct variables, and keeps unsupported owners blocked. Focused gates
covered `call_user_func` by-reference source and linked executable proof,
descriptor-closure regressions, closure-capture callable regressions, compile
checking, formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-callable-byref-args-fec83cfc-20260527.gates.log`
sha256 `51f55ecd03cd753326646da6c081619b8d030851c0632154bcfb6a9aaad0be47`.

Recent source commit `c9e97085` advances nested ArrayAccess reference owner
coverage without adding object-name ladders, generated-C source-shape
recognizers, or cloned reference writeback. Runtime
`phpc_native_reference_array_path_reference_with_diagnostic` materializes nested
array cells from an existing native reference and returns an aliasing reference
handle; generated C routes selected nested ArrayAccess reference consumers
through `offsetGet()` reference acquisition for the root segment and the shared
reference-array-path ABI for remaining keys. Focused gates covered runtime
reference-array-path mutation, generated-C source proof, linked executable
nested reference write-through, adjacent static-property ArrayAccess reference
sources, owner-stack reference-returning `offsetGet()` contracts, compile
checking, formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-arrayaccess-nested-ref-d4af514d-20260527.gates.log`
sha256 `5aa85e27ddb02b3bb9da356908e6850f66c29d8159a5742685e8091bac75b0c3`.

Recent source commit `182cb33a` scopes already-compiled literal include units
to the active generated user-function caller. Include units now receive a
reusable execution-state packet carrying the caller symbol scope, so included
top-level assignments, direct variable reads, and include return values are
visible inside function frames instead of being routed through the request root
symbol table. Runtime-only source activation, broader dynamic include
execution, exit/exception propagation parity, and non-C backend include
execution remain blocked.

Recent source commit `182cb33a` advances compiled include/require execution
without adding include-path tables as production shortcuts, source-name
recognizers, source-loader synthesis, or one-fixture lowering. Generated C now
splits root `$GLOBALS` handoff from function-local include scope requirements,
creates function-local symbol tables only for frames that execute includes, and
passes `phpc_NativeIncludeExecutionState` to compiled include units. Focused
gates covered generated-C source proof, linked executable function-scope
include execution, the full native include/require boundary suite, user-function
frame environment requirements, compile checking, formatting, and diff checks.
Primary gate log:
`state/logs/phpc-primary-include-return-once-d3f49f5e-20260527.gates.log`
sha256 `2cde7effd690e2dc87de88adaffd8d782a8b900bf1450771095aec3083b4d7c4`.

Recent source commit `f4ebc3f2` advances generated-C closure/callable
interoperability without adding closure-name ladders, one-fixture closure
structs, callable-dispatch bypasses, or fake by-reference capture mutation.
Direct `call_user_func()`/`call_user_func_array()` builtins now lower through
existing callable lookup and descriptor closure invocation paths when the
callback contract accepts ordinary value arguments, and unsupported builtin
spread families share the materialized-entry producer blocker. Focused gates
covered `call_user_func` source/link proof, descriptor closure regressions,
runtime builtin callable-string spread regressions, unsupported spread blocker
proof, compile checking, formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-closure-captures-5af0524a-20260527.gates.log`
sha256 `4d2cd67782613da4c8d83138ee6b4e3e728fabf72fe6e05858733c6ec1961c1b`.

Recent source commit `78a03ae7` publishes trait-composed class members through
the generated-C runtime metadata registry. Method/property introspection,
callable-spread contracts, declared method frames, constructor frames,
`instanceof` regressions, destructor-risk regressions, and class metadata
consumers now share the validated declared-class metadata boundary instead of
raw class member scans. Full trait conflict/precedence parity, visibility edge
cases, aliases beyond the selected metadata surface, and non-C backend parity
remain blocked.

Recent source commit `78a03ae7` advances trait-composed class metadata without
adding one-trait fixture lowering, method-name production ladders, or
generated-C substring gates as production logic. Generated-C class declaration
registration now consumes `CDeclaredClass` metadata for direct trait names,
effective method metadata, and composed property metadata, so shared runtime
metadata surfaces see trait-provided members consistently. Focused gates covered
trait-composed source proof, linked metadata/introspection execution,
trait-composed callable-spread execution, object-model trait composition,
declared callable frame metadata, adjacent declared-class metadata and source
call regressions, compile checking, formatting, and diff checks. Primary gate
log:
`state/logs/phpc-primary-trait-composition-ea6e7e87-20260527.gates.log`
sha256 `52895b767bdf76eb89ecbcb126694fc37f0d97eba26b93dfc53c9e01390e76ec`.

Recent source commit `b13c97cf` unwinds native output buffers at generated-C
shutdown and early-exit boundaries. Nested output-buffer handlers now drain in
final order through the runtime stack ABI, exit string operands write through
active buffers, and generated C calls the shared unwind helper before normal or
exit returns. Shutdown functions, destructor/fatal-exception parity,
callback-mutated buffer-stack edge cases, broader binary strings, and non-C
backend output-buffer parity remain blocked.

Recent source commit `b13c97cf` advances output-buffer shutdown/unwind without
adding exact output snapshots, one-handler constant lowering, or source-shape
recognizers. Runtime output writes can target a specific outer buffer, final
unwind drains nested handlers through the callable-table path, and generated C
routes normal main cleanup plus early `exit()` cleanup through
`phpc_native_output_buffer_unwind_stack_with_diagnostic`. Focused gates covered
runtime final-order unwind, exit-string buffering, linked shutdown/exit
programs, generated-C source proof, output-buffer builtin regressions, compile
checking, formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-output-buffer-unwind-3c9731a4-20260527.gates.log`
sha256 `c482bbf91eef7993d7acedeb80667af1f62f03b5fc708e656fc0f6bdf49fa96c`.

Recent source commit `b7ba8cfd` routes selected method/static
source-call spreads through the existing native materialized argument carriers.
Receiver, dynamic receiver, static, dynamic static, object-static, and
by-reference method/static spread calls can now reuse `NativeCallArgumentsHandle`
instead of falling back to spread blockers or generated dispatch ladders. Named
spread semantics, unknown runtime-only callable shapes, broader COW/reference
parity, magic dispatch, and LLVM/ASM callable parity remain blocked.

Recent source commit `b7ba8cfd` advances method and static source-call spread
lowering without adding one-callable ladders, fake argument flattening, or
source-shape production recognizers. Spread-bearing call sites now compute an
unknown static argument count, require parameter-name-compatible contracts where
needed, and feed proven receiver/static/callable source-call families through
the shared materialized argument ABI. Focused gates covered generated-C source
proof, linked executable receiver/static spread execution, interface-only
contract units, adjacent direct user-function spread lowering, named and
callable family regressions, runtime callable variable spread identity,
compile checking, formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-callable-spread-ba7ac923-20260527.gates.log`
sha256 `05cd93d6048f363acbf14d19fed533f44d3382fd266eacaa0d82a717bfe38530`.

Recent source commit `ba7ac923` resolves generated-C relative
`instanceof` targets from class context. `self` and `parent` targets use the
active declared-class metadata, `static` uses the generated called-scope string
handle, and all relative targets share the existing class-relationship ABI
instead of text-membership tables or fixture names. Parser support for
`instanceof static` is now present. LLVM/ASM relative targets, arbitrary
runtime source activation, and broader class-context parity remain blocked.
Primary gate log:
`state/logs/phpc-primary-relative-instanceof-9d342d7a-20260527.gates.log`
sha256 `705cc5379b3094075fcb08251c2c8dc0278bf1a059cb73106e3af08d3486dfb0`.

Recent source commit `9d342d7a` lowers LLVM/ASM dynamic right-hand
`instanceof` targets through
`phpc_native_value_dynamic_class_relationship_matches_with_diagnostic`.
LLVM now materializes dynamic variable and expression operands as native values,
frees them after the relationship call, and keeps relative targets at the
explicit LLVM boundary. Focused gates covered LLVM IR and assembly source
proof, generated-C dynamic relationship proof, runtime dynamic relationship
targets, named/relative/object `instanceof` regressions, syntax boundaries,
compile checking, formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-llvm-dynamic-instanceof-d2e48fd5-20260527.gates.log`
sha256 `c8cb8c6d1161197c3916feaa60fd1d3895dc3d6f2aeaa3b3145834cd28be2874`.

Recent source commit `dd391c36` routes reference-returning
`ArrayAccess::offsetGet()` through the native reference-owner path for selected
generated-C ArrayAccess subjects. Direct ArrayAccess index references and
property-held ArrayAccess owners can now feed by-reference consumers and alias
assignments through `phpc_native_value_arrayaccess_offset_get_reference...`
instead of falling back to `offsetSet()` or cloned values. Broader nested owner
shapes, unknown ArrayAccess facts, full COW/reference parity, and LLVM/ASM
parity remain blocked.

Recent source commit `dd391c36` advances ArrayAccess reference-source
write-through without adding fake reference cloning, object-name ladders,
fixture substring production, or `offsetSet()` substitution. Runtime tests
prove `offsetGet()` can return a write-through reference, generated-C source
tests prove direct and property-held ArrayAccess reference sources use the
shared reference ABI and object-property owner path, and linked executable
proof rejects accidental `offsetSet()` fallback by checking stdout. Focused
gates covered runtime write-through, generated-C source proof, linked
reference-source execution, existing ArrayAccess reference-owner and
property-held owner regressions, compile checking, formatting, and diff
checks. Primary gate log:
`state/logs/phpc-primary-arrayaccess-refget-e66480ba-20260527.gates.log`
sha256 `0da2d3eaf7b6d5fb4f93cea46fecad15f6f09d12efe9360d6ee1a4b41c80389d`.

Recent source commit `c89fedc4` routes dynamic `instanceof`
right-hand targets through generalized class-relationship metadata instead of
rejecting `$class`, parenthesized expressions, or object targets at parse time.
Parser and AST now represent `instanceof` operands as `NewClassName`,
interpreter execution resolves string/binary-string/object target values, and
generated C uses a runtime dynamic relationship ABI for class/interface checks.
Relative `self`/`parent`/`static` targets, arbitrary autoloaded runtime source
activation, and LLVM dynamic right-hand lowering remain blocked.

Recent source commit `c89fedc4` advances dynamic `instanceof` without adding
text-membership tables, fixture class-name ladders, literal source recognizers,
or generated-C substring-driven production logic. The integration resolved the
post-LLVM-parity rebase by keeping LLVM named-target lowering on the existing
relationship ABI while dynamic LLVM targets retain their explicit unsupported
boundary. Focused gates covered parser operand modeling, interpreter dynamic
string/object behavior, runtime dynamic relationship metadata, generated-C
source proof, linked declared-class/interface execution, unsupported snapshot
cleanup, compile checking, formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-dynamic-instanceof-c31ef8ff-20260527.gates.log`
sha256 `a50efcb2d43c7abfec62720a4ffba18b03a58bf4c0c0c43d26bcacfc363f148b`.

Recent source commit `02aaa312` tracks live native
output-buffer handler status flags instead of reporting bounded constants.
Runtime output buffers now store handler type and requested flags on each live
stack entry, mark started/processed state as writes, clean, flush, and chunk
thresholds pass through the shared buffer machinery, and generated C/LLVM/ASM
output-buffer builtins route through the native runtime ABI instead of the old
lowering rejection. Full PHP parity for shutdown/unwind order, SAPI
interaction, broader binary strings, exact diagnostics, and non-native backend
behavior remains blocked.

Recent source commit `02aaa312` advances native output-buffer status flags
without adding exact flag snapshots, fixture-shaped status arrays, or
generated-C constant lowering. Handler status now preserves requested
high/mutability bits while replacing the low type nibble from the runtime
handler kind, and runtime operations update `STARTED|PROCESSED` state from live
buffer transitions. Focused gates covered runtime flag/type relationships,
generated-C source proof that status calls use the shared runtime stack ABI,
linked executable flag/metadata programs, the full output-buffer builtin suite,
compile checking, formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-output-buffer-flags-3c1c8624-20260527.gates.log`
sha256 `c63849f3b4afd67c6ed7c052b662bc125384e7f5c06f0a8d42554408c21a32f3`.

Recent source commit `0b6cafd2` exposes the resolved
filesystem source path when runtime include/require registry misses hit an
existing PHP source. The runtime now returns `SOURCE_LOAD_REQUIRED` with an
owned `resolved_path` byte buffer, generated C frees that buffer on no-match
paths, and generated user-function frames receive include registry/source
context so SPL autoload callbacks that perform include/require reach the shared
source-loader boundary. Arbitrary runtime PHP parsing/execution, caller include
scope, declaration activation, once identity, return/exit propagation, and
full autoload/source diagnostics remain blocked.

Recent source commit `0b6cafd2` advances the runtime source-loader ABI without
adding a parser, Composer/PSR adapter, class-prefix map, generated source
table, exact path recognizer, or fake declaration activation. Runtime no-match
diagnostics now distinguish missing files from existing filesystem sources and
carry the concrete resolved path from include-path or source-relative search.
Generated C defines and frees the new no-match `resolved_path` field, and
autoload callback include tests prove existing source files reach the shared
loader-required diagnostic before the current callable-frame fatal boundary.
Focused gates covered runtime no-match diagnostics, runtime include-unit
registry lookup, generated-C source proof, autoload callback source proof,
existing include/require linked executable regressions, the full native
include/require boundary suite, compile checking, formatting, and diff checks.
Primary gate log:
`state/logs/phpc-primary-runtime-source-loader-2047aaab-20260527.gates.log`
sha256 `f0648d252ba70436709d4885778c255c9b0ad83949fc847d14933a86c4f2b718`.

Recent source commit `ba0dcfad` advances generated-C function namespace
fallback without adding exact source recognizers, one-namespace ladders,
fixture-name dispatch, or builtin-specific production branches. Parser tests
prove unqualified namespaced calls are marked before codegen fallback, runtime
tests preserve namespace-local shadowing, generated-C source tests prove
supported global builtins route through callable lookup/invoke only when no
local declaration is available, and linked executable proof covers local
shadowing plus builtin fallback together. Focused gates covered the new source
and linked namespace fallback tests, the full namespace-resolution suite,
global import snapshots, the functions/scopes regression suite, compile
checking, formatting, and diff checks. Primary gate log:
`state/logs/phpc-primary-function-namespace-fallback-2901b3de-20260527.gates.log`
sha256 `667620cfe6728df0b4e72a47ee3c2e72da010205944b2f826c25ba0126ab333e`.

Recent source commit `8ad19bc5` advances LLVM/ASM `instanceof` parity without
adding text-membership tables, class-name ladders, fixture recognizers, or
generated-C snapshot production lowering. Named non-relative `instanceof`
targets in LLVM now share the runtime relationship ABI used by generated C for
native values; tests keep `self`/`parent`/`static`, dynamic right-hand targets,
and LLVM object construction blocked at their existing generalized boundaries.
Focused gates covered current-head patch apply-check, LLVM IR source proof,
ASM source proof, relative/dynamic/object blocker regressions, syntax-boundary
regression, existing generated-C source and linked `instanceof` regressions,
runtime relationship metadata regression, compile checking, formatting, and
diff checks. Primary gate log:
`state/logs/phpc-primary-llvm-instanceof-94621061-20260527.gates.log` sha256
`8fa8f4d16de481034a03903fd3f4ab08b61a2ec828ed889021a553cb6f7c4d9d`.

Recent source commit `3cbaaf61` advances comparator sort callback writeback
without adding fixture-array production sorting, generated-C fake comparator
orders, callable-dispatch bypasses, or exact-shape ladders. `usort()`,
`uasort()`, and `uksort()` now participate in the shared runtime callable sort
family with `array, callback` signatures, generated-C source calls route
through `phpc_native_array_lvalue_owner_callable_sort_result`, and runtime
tests prove registered comparator invocation plus writeback for value,
associative, and key-sorting modes. The integration also removed a dead
array-sort fallback pattern exposed by the refresh. Focused gates covered
compile checking, runtime comparator writeback, runtime builtin signature
metadata, generated-C source proof, linked comparator callback execution,
existing runtime builtin sort writeback including `natsort()`/`natcasesort()`,
output-buffer status regression, formatting, and diff checks. Full
flag/diagnostic parity, object/ArrayAccess/resource owners, broader callable
object/array families, broad reference/COW parity, and LLVM/ASM parity remain
blocked. Primary rerun gate log:
`state/logs/phpc-primary-comparator-sort-f47469b1-20260527.rerun.gates.log`
sha256 `af147b7b8824c4ad2bd90bf493ec2baa4ce3b119263b732b952f67d5f283ad08`.

Recent source commit `4602d028` advances generated-C output-buffer status
metadata without adding production handler-name fixtures, generated-C snapshots,
or source-shape recognizers. Runtime output-buffer status now reads live stack
entries for handler names, stack levels, buffer byte counts, and configured
chunk sizes; callback handler names are derived from
`NativeCallableValueDispatch` metadata rather than fixture literals. Focused
gates covered compile checking, runtime status metadata, linked generated-C
status execution, generated-C source proof that status calls route through the
shared runtime stack ABI, existing runtime/generated-C output-buffer
regressions, SPL autoload function snapshot regression, formatting, and diff
checks. `type` and `flags` remain bounded constants; full PHP parity for
mutability flags, shutdown/unwind order, SAPI interaction, broader binary
strings, non-C backend output-buffer parity, and exact diagnostics remain
blocked. Primary gate log:
`state/logs/phpc-primary-output-buffer-status-d7c37ba1-20260527.gates.log`
sha256 `606b95ecaeadc49892b7b84bfdb93d269f4d7b4a4b68a400fd07e2257a519cac`.

Recent source commit `4982fccd` advances interface-aware `instanceof` without
adding one-interface, namespace, generated-text, or fixture-membership lowering.
`phpc_native_value_class_relationship_matches_with_diagnostic` centralizes the
runtime relationship check, while the old class-only wrapper remains as a
compatibility boundary. Generated C routes named `instanceof` and related
receiver class checks through that operation-tagged ABI, and the linked fixture
covers ordinary classes, direct interfaces, and parent-interface relationships.
Focused gates covered compile checking, runtime relationship metadata,
generated-C source proof, linked declared-class/interface execution, callable
object/array contract regression, SPL autoload function snapshot regression,
formatting, and diff checks. Dynamic `instanceof $class`, arbitrary runtime
source loading/autoloaded dynamic declarations, broader relationship operations,
and LLVM/ASM parity remain blocked. Primary gate log:
`state/logs/phpc-primary-instanceof-interface-ffac6f7a-20260527.gates.log`
sha256 `f362eac385a8ed11d64e6d600c9b5f117750dceb0f99c0cb3416b93dce3393cd`.

Recent source commit `f1386f82` advances callable-array/object source-call
contracts without adding fixture-name, one-class, generated-C substring, or
branch-shape production lowering. Concrete callable arrays such as
`[ClassName::class, "method"]` and `[$object, "method"]`, plus known public
non-static `__invoke` callable objects, now prefer identity-derived finite
contracts through `callable_value_source_call_contract_from_identities` before
falling back to the older object/array metadata helpers. Focused gates covered
compile checking, the central callable array/object identity unit, generated-C
source proof, linked mixed callable execution, heterogeneous metadata blocking,
callable object named/reference argument regression, callable array named
argument regression, SPL autoload function snapshot regression, formatting, and
diff checks. Interface-only callable contracts, runtime-only callable values,
unsupported external/core callables, comparator callbacks, broader magic
fallbacks, broad reference/COW parity, and LLVM/ASM parity remain blocked.
Primary gate log:
`state/logs/phpc-primary-callable-object-array-contracts-95d7f363-20260527.gates.log`
sha256 `0add93b5f36227c00df8a12929928d6302861cc6eb047aa9373a75837165de9c`.

Recent source commit `b0727428` advances generated-C
`spl_autoload_functions()` semantics without adding runtime PHP parsing or
fixture-shaped callback lists. The SPL autoload registry now retains original
callback values alongside normalized callable dispatch, exposes a snapshot ABI,
and generated C lowers `spl_autoload_functions()` through the request-local
registry so callback order, prepend behavior, unregister equality, and callback
forms are observable. Focused gates covered compile checking, generated-C
source lowering, linked executable registry snapshot proof, runtime snapshot
materialization/equality, runtime registry order/prepend/unregister regression,
class-alias autoload regression, SPL autoload register lowering regression,
formatting, and diff checks. Arbitrary runtime source loading, Composer/PSR
adapters, precise filesystem autoload parity, broader class-like source/alias
parity, and LLVM/ASM parity remain blocked. Primary gate log:
`state/logs/phpc-primary-spl-autoload-functions-214e60a6-20260527.gates.log`
sha256 `926bf03057cf4ed64717d0faa1657d6a5c5a1853a420cc233cae8d267693d945`.

Recent source commit `4138e55c` advances generated-C class alias autoload
semantics without adding fixture-specific alias loading. Runtime
`class_alias(..., true)` can now invoke callbacks from the existing SPL autoload
registry before alias lookup, generated C lowers default/true alias calls
through that registry-aware ABI, literal `false` stays on the no-registry path,
and dynamic autoload values select the correct ABI at runtime. Focused gates
covered compile checking, runtime alias-before-lookup proof, the generated-C
`class_alias` native-link suite, existing runtime class alias and SPL autoload
registry regressions, SPL autoload register lowering regression, heterogeneous
callable policy regression, formatting, and diff checks. `spl_autoload_functions()`,
arbitrary runtime source loading, Composer/PSR adapters, precise filesystem
autoload parity, broader class-like alias/source parity, and LLVM/ASM parity
remain blocked. Primary gate log:
`state/logs/phpc-primary-class-alias-autoload-a2c56579-20260527.gates.log`
sha256 `76fe8e2962ad5613db39301401bb342dd9294a655f090bb36da1bc74a864b9d4`.

Recent source commit `c0e81cfe` advances selected natural-sort callable
writeback without adding exact-name dynamic ladders, one-array expected-order
production lowering, or docs-only substitution. `NativeArraySortOperation` now
marks `natsort()` / `natcasesort()` as supported by the shared runtime callable
writeback family, runtime callable builtin signatures expose the natural-sort
arity/reference contract, and compiler callable-string metadata recognizes both
functions as by-reference runtime builtins with no optional flags parameter.
Focused gates covered compile checking, runtime builtin signature metadata,
runtime natural-sort writeback through callable references, compiler
runtime-builtin source-call metadata, generated-C source proof, linked known and
unknown callable-string natural-sort execution, heterogeneous callable policy
regression, SPL autoload lowering regression, formatting, and diff checks.
Comparator callback families (`usort`, `uasort`, `uksort`), full flag/diagnostic
parity outside the selected mode/natural subset, object/ArrayAccess/resource
owners, broad callable object/array families, broad reference/COW parity, and
LLVM/ASM parity remain blocked. Primary gate log:
`state/logs/phpc-primary-natural-sort-writeback-e9034447-20260527.gates.log`
sha256 `35652cfdd96563e3ddc8385c1c974f4a25a359213c42dbfc77160b27ce3c8470`.

Recent source commit `db8d36f6` advances heterogeneous callable-family policy
without adding generated-substring, one-callable, fixture-name, or per-branch
production logic. `CNativeCallableIdentity::DescriptorClosure` now participates
in `callable_value_source_call_contract_from_identities` by reading the copied
descriptor closure summary, checking arity, building the same frame argument
plan used by generated callable frames, and requiring the shared signature
compatibility rules before a finite mixed family can use compile-time
source-call finalization. Focused gates covered compile checking, the central
descriptor-closure mixed-family helper, generated-C source proof, linked
mixed generated/builtin callable execution, heterogeneous metadata regression,
SPL autoload lowering regression, formatting, and diff checks. Runtime-only
heterogeneous callable values, unsupported external/core callables, comparator
callbacks, broader callable object/array families, full by-reference/COW
parity, magic fallback callables, and LLVM/ASM parity remain blocked. Primary
gate log:
`state/logs/phpc-primary-heterogeneous-callable-policy-9828907a-20260527.gates.log`
sha256 `fbc5529af2f6a440fc7e97d6f8ee69bd0f3b613b72b6537811b6bedf6bbc204b`.

Recent source commit `0b7285a4` advances generated-C SPL autoload lowering
without adding runtime PHP parsing, source-file loading, docs-only
substitution, or fixture-shape callback dispatch. The compiler now emits a
request-local `phpc_user_spl_autoload_registry`, registers and unregisters
normalized callable values with prepend ordering, lowers `spl_autoload_call()`
to the runtime registry dispatcher, and sends autoload-enabled class, trait,
and interface metadata probes through
`phpc_native_value_class_metadata_exists_with_autoload_registry_and_diagnostic`.
The integration also tightened interface tests so `interface_exists(..., false)`
stays on the no-autoload metadata ABI while default/autoload-enabled interface
lookups use the SPL registry. Focused gates covered compile checking,
generated-C source proof, linked SPL autoload register/call/unregister
execution, class/trait/interface registry metadata lookups, include/require
metadata boundaries, runtime autoload registry regressions, output-buffer
chunk regression, formatting, and diff checks. `spl_autoload_functions()`,
arbitrary runtime source loading, Composer/PSR adapters, precise filesystem
autoload parity, and LLVM/ASM parity remain blocked. Primary rerun gate log:
`state/logs/phpc-primary-spl-autoload-lowering-a1ff9df4-20260527.rerun.gates.log`
sha256 `cac7ad09e00a4d280591cf31cfde407532a29ddc4430a757faa2075bbf6ea79c`.

Recent source commit `18fcb7f6` advances class-like interface metadata without
adding one-interface, namespace, fixture, or generated-text membership
lowering. `NativeClassMetadataOperation` now includes `InterfaceExists`,
runtime lookup normalizes interface names through the existing user-interface
metadata table, the autoload policy wrapper reports missing generated-native
interface autoload explicitly, and the registry-aware metadata ABI can recheck
interfaces after normalized callable callbacks run. Generated-C
`interface_exists($name, false)` now emits the same runtime metadata call
family as class/trait lookups, while autoload-enabled calls continue to route
through the centralized policy boundary until generated-C SPL registry state
lands. Focused gates covered compile checking, runtime class/trait/interface
autoload registry proof, generated-C source and linked `interface_exists`
tests, SPL autoload registry regression, output-buffer chunk regression,
formatting, and diff checks. Generated-C SPL autoload registry request state,
`spl_autoload_functions()`, arbitrary runtime source loading, Composer/PSR
adapters, and LLVM/ASM parity remain blocked. Primary rerun gate log:
`state/logs/phpc-primary-interface-metadata-f02b219b-20260527.rerun.gates.log`
sha256 `443ec3d4b3883801238cb8d56c0a394bef648992da6d93c6cdba99fbf9ee3e71`.

Recent source commit `abe650f4` advances selected callable key-sort writeback
without adding source-fixture, local-variable, literal-only, generated-C
substring-only, or backend special-case lowering. `NativeArraySortOperation`
now marks `ksort()` / `krsort()` as supported through the shared runtime
callable writeback family, runtime builtin lookup and names expose both
functions, and compiler callable-string metadata recognizes them with the same
by-reference `$array`, defaulted `$flags`, and source-call support as the
already integrated value-sort family. Focused gates covered compile checking,
runtime builtin signature metadata, direct runtime callable sort invocation,
native array lvalue sort regression, compiler builtin signature planning,
generated-C and linked known/unknown callable string sort writeback, output
buffer chunk regression, formatting, and diff checks. Comparator callback
families (`usort`, `uasort`, `uksort`), natural sort callable writeback, full
flag/diagnostic parity, object/ArrayAccess/resource owners, broad callable
object/array families, broad reference/COW parity, and LLVM/ASM parity remain
blocked. Primary gate log:
`state/logs/phpc-primary-key-sort-after-output-buffer-eb22c760-20260527.gates.log`
sha256 `63c90a741fe0512066c1672de962425cf55431ebd42837d8c80252779fd2a47f`.

Recent source commit `43975e79` advances selected output-buffer chunk-size
semantics without adding handler-name, output-string, fixture-shape, or
generated-C snapshot lowering. `NativeOutputBuffer` now stores the configured
chunk size, native stdout writes append through a shared target-buffer helper,
and buffers whose accumulated length reaches a positive threshold flush the
whole chunk to the next lower buffer or stdout. Automatic chunk flushing uses
the existing optional output-buffer callback dispatch path: the first chunk
uses `PHP_OUTPUT_HANDLER_START`, subsequent automatic chunks use the write
phase, and explicit final flush still uses the final phase. Focused gates
covered compile checking, runtime chunk tests over two chunk sizes with plain
and callback buffers, existing callback flush regression, linked generated-C
chunk execution, existing linked callback and ordinary output-buffer
regressions, SPL autoload registry regression, formatting, and diff checks.
Handler flags/mutability, shutdown/unwind ordering, precise diagnostics,
SAPI interaction, `ob_get_status()` metadata, binary string breadth, and
LLVM/ASM parity remain blocked. Primary gate log:
`state/logs/phpc-primary-output-buffer-chunk-7b543baf-20260527.gates.log`
sha256 `2910e004b281a22140c125122ca6c466d0640c326a18576d40037a0754793950`.

Recent source commit `f1b58b52` advances generalized SPL autoload runtime
semantics without adding generated-C lowering, runtime PHP parsing, or
framework/path-shape adapters. The runtime now has a request-local
`NativeSplAutoloadRegistryHandle`, stores callbacks as normalized
`NativeCallableValueDispatch` values, supports ordered register/prepend and
unregister by callable-value equality, invokes callbacks with the requested
class-like name through
`phpc_native_callable_value_invoke_discard_with_diagnostic_and_free`, and adds
`phpc_native_value_class_metadata_exists_with_autoload_registry_and_diagnostic`
as a class/trait metadata lookup sibling. The existing generated-native
autoload-policy blocker remains in place for current compiler callers until
compiler lowering deliberately migrates to the registry-aware ABI. Focused
gates covered compile checking, runtime registry ordering across callable
shapes, class and trait miss autoload, reentrant loading diagnostics,
callable-dispatch regression, existing autoload-policy blocker regression,
formatting, and diff checks. Generated-C `spl_autoload_register`,
`spl_autoload_unregister`, `spl_autoload_call`, `class_exists(..., true)` /
`trait_exists(..., true)` / `interface_exists(..., true)` lowering,
`spl_autoload_functions()`, arbitrary runtime source loading, Composer/PSR
adapters, and LLVM/ASM parity remain blocked. Primary gate log:
`state/logs/phpc-primary-spl-autoload-registry-d6047206-20260527.gates.log`
sha256 `114528df8e2bf463099dbd06e091c961afd67f9566c8f7847d5a83bc55e80023`.

Recent source commit `e74e365a` advances selected runtime callable sorting
writeback without adding callable-name or fixture-shape lowering. The runtime
sort operation metadata now exposes the same callable builtin signature and
writeback support for `asort()` / `arsort()` as for `sort()` / `rsort()`, and
compiler callable-string canonicalization plus native builtin source-call
metadata recognize the expanded family. Focused gates covered runtime callable
sort helpers, native array lvalue sorting, builtin signature metadata,
runtime callable array regressions, generated-C source proof, linked known and
unknown callable string execution, by-reference and variadic writeback
regressions, callable string spread regression, descriptor closure spread
regression, output-buffer callback regression, formatting, and diff checks.
Key-sort families, comparator callbacks, full flag/diagnostic parity,
object/ArrayAccess/resource operands, heterogeneous callable-family policy,
broader callable object/array/external families, broad COW/reference parity,
and LLVM/ASM parity remain blocked. Primary gate log:
`state/logs/phpc-primary-sort-after-closure-6f3157ea-20260527.gates.log`
sha256 `5b08c278181a9c25266258a39659e136a5c57e601feb3e7b29aa5dc566c58cca`.

Recent source commit `ebd29e69` advances selected descriptor closure callable
semantics without adding closure-literal or fixture-shape lowering.
`NativeClosureDescriptor` now carries copied source-call metadata for parameter
names, required/default flags, by-reference flags, default values, and variadic
metadata; `PhpClosure` stores that signature for descriptor-backed callable
values; and dynamic named/spread calls without a compile-time argument plan
route descriptor closures through
`phpc_native_callable_value_finalize_materialized_arguments_with_diagnostic`.
Generated closure descriptor emission reuses the same source-signature
registration helper consumed by generated functions and methods. Focused gates
covered runtime descriptor closure finalization, variadic descriptor closure
finalization, broader runtime callable-value regressions, linked unknown
descriptor closure execution, linked descriptor closure spread execution,
output-buffer callback regression, unknown-callable regression, formatting, and
diff checks. Heterogeneous callable-family signature compatibility,
unsupported external/core callables, broader callable object/array dispatch,
comparator callback signatures, full reference/COW parity, and LLVM/ASM parity
remain blocked. Primary gate log:
`state/logs/phpc-primary-closure-descriptor-after-output-d096ea47-20260527.gates.log`
sha256 `e2fed6e32f9b0a4f1faac954151af049d15f818ecefd6bb655079b4422d1bec2`.

Recent source commit `76497f73` advances selected output-buffer callback
dispatch without adding handler-name or fixture-shape lowering. Generated-C
`ob_start($handler, ...)` now routes callback operands through the native
callable table and `NativeCallableValueDispatch`; runtime output buffers store
the optional callable dispatch and invoke it through
`phpc_native_callable_value_invoke_value_with_diagnostic_and_free` at
flush/final boundaries. Ordinary output-buffer operations continue through the
non-callback runtime ABI. Focused gates covered runtime callback invocation,
ordinary output-buffer helper regressions, generated-C callback ABI routing,
linked callback execution, ordinary linked output-buffer regressions,
cross-backend native runtime ABI proof, unknown-callable regression coverage,
formatting, and diff checks. The primary gate also corrected a stale
generated-C source assertion to the source-signature callable-table registration
path introduced by `622f9479`. Full PHP output-buffer parity still needs
chunk-size-triggered flushing, handler flags and mutability, precise
warning/fatal ordering, shutdown/unwind flushing, binary-string breadth,
SAPI interactions, `ob_list_handlers()` / `ob_get_status()` metadata, broader
interpreter/LLVM/ASM callback parity, and broad reference/COW parity. Primary
rerun gate log:
`state/logs/phpc-primary-output-buffer-after-unknown-13eec3f0-20260527.rerun.gates.log`
sha256 `469536493d6ecfeea99671c2de3a59a614d46529bda4b16d32348ab51df137f7`.

Recent source commit `622f9479` advances selected unknown callable
named/spread semantics without adding callable-name or fixture-shape lowering.
Runtime callable descriptors now publish source-call signatures with parameter
names, required/default flags, by-reference flags, default values, and variadic
metadata; generated user functions and declared methods register those
signatures from PHP parameter metadata; and unknown callable source calls use
`phpc_native_callable_value_finalize_materialized_arguments_with_diagnostic`
instead of compile-time parameter arrays. Focused gates covered unknown
generated-C/link execution, runtime callable source-signature finalization,
materialized argument/reference helpers, sort/variadic/by-reference callable
regressions, leading-global and qualified-call regressions, keyword named
argument regressions, include_path registry regressions, formatting, and diff
checks. Heterogeneous callable-family signature policy, descriptor closure
source signatures, unsupported external/core callables, broader callable
object/array families, broader by-reference/writeback builtins, full
reference/COW parity, and LLVM/ASM parity remain blocked. Primary gate log:
`state/logs/phpc-primary-unknown-callable-6085869e-20260527.gates.log`
sha256 `a14912cfe23ba7e3922a7e78bd92a84d25fef1ac45f2b50f75146eee5f507dbd`.

Recent source commit `16f6737c` advances selected runtime include/require
registry fidelity without adding a runtime PHP parser or source loader. The
runtime ABI now searches generated registry aliases through ordered include_path
entries plus source-directory fallback, and generated-C include/require
dispatch passes those declared include_path entries before falling through to
centralized no-match diagnostics. Focused gates cover runtime registry lookup,
runtime no-match diagnostics, generated-C include_path include/require
boundary tests, the full native include/require boundary suite, formatting,
and diff checks. Fully arbitrary runtime source loading/parsing, dynamic
`set_include_path()` mutation, stream wrappers/file URLs, non-UTF-8 path
fidelity, symlink/stat-cache/open_basedir parity, exact warning/source-map
fidelity, and LLVM/ASM parity remain blocked.

Recent source commit `d6760f45` advances selected direct function-call
namespace semantics without adding one-name or one-namespace lowering. The
parser now normalizes direct call names into unqualified, namespace-relative,
leading-global, and explicit `namespace\...` categories; namespace resolution
applies imports only where PHP does, and generated-C user-function lookup
continues through the shared direct-call key path. Focused gates cover syntax
boundaries, namespace resolution, dynamic-feature lookup diagnostics,
generated-C source proof, linked qualified-call execution, imported and
leading-global regressions, keyword named-argument parser composition,
formatting, and diff checks. Multiple namespace declaration forms,
namespace-qualified function declarations, arbitrary dynamic runtime lookup,
namespace-qualified constants/static-member surfaces, broad callable/magic
fallback, and LLVM/ASM namespace lowering remain blocked.

Recent source commit `7533992f` advances selected generated-C by-reference
`foreach` writeback without adding source-shape lowering. Loop value references
now feed `AssignTarget` array-lvalue writeback for direct keyed writes, direct
appends, nested keyed writes, nested appends, and append-with-suffix nested
writes rooted at the loop value reference, preserving the existing owner
materialization and reference-slot invalidation boundaries. Focused gates cover
by-reference foreach source/link proof, native array-lvalue regressions, array
append call-value interaction, runtime foreach owner helpers, PHP array path
and reference-cell helpers, array reference literal regressions, formatting,
and diff checks. Temporary iterable owners, arbitrary side-effecting loop
bodies, branch/control-flow writeback, symbol-table/request iterable owners,
object/static/non-local iterable owners, broad reference/COW parity, and
LLVM/ASM parity remain blocked.

Recent source commit `9bf241cc` advances selected keyword named-argument
semantics through generalized parsing and call-argument normalization. The
parser now recognizes keyword tokens in contextual argument-label position, the
shared normalizer/finalizer treats those labels as ordinary parameter names,
and selected `print_r(value: ..., return: ...)` plus mixed spread forms route
through the materialized call-argument ABI and native value formatter return
mode. Focused gates cover parser keyword-label tests, shared call-argument
normalization/finalization, generated-C and linked selected `print_r`
named/spread execution, ordinary named-argument regressions, formatting, and
diff checks. Broader named-argument lowering, unsupported non-native
`print_r` operands, output/object formatting parity, arbitrary builtin-family
named/spread binding, and LLVM/ASM parity remain blocked.

Recent source commit `2c4268a9` advances selected append assignment call-value
semantics without adding source-shape lowering. Append targets that already own
native value-offset or array-lvalue writeback now share
`materialize_assignment_expression_replacement_value()`, cloning assignment
expression values for storage where needed and preserving expression results.
Focused gates cover generated-C source proof, linked direct and nested append
call-value execution, nested assignment/lvalue regressions, runtime nested
append materialization helpers, reference-backed append storage, formatting,
and diff checks. Keyed assignment RHS calls, object/ArrayAccess append owners,
unknown callable families, broad COW/reference parity, and LLVM/ASM parity
remain blocked.

Recent source commit `cb323d8e` integrates the dynamic magic method spread
surface through the shared materialized source-call forwarding carrier and
fixes magic `$args` mutation visibility at the runtime packer boundary.
Focused gates cover runtime magic argument COW/reference helpers, magic byref
spread, ordinary magic callable spread, dynamic magic method spread,
by-reference variadic unpack regressions, direct by-reference variadics,
formatting, and diff checks. Runtime-unknown dynamic method families outside
the selected declared magic fallback contract, broader callable object/array
families, full backend parity, and broad reference/COW parity remain blocked.

Recent source commit `5735906e` adds the next honest runtime include boundary
for generated-C registry no-matches. Runtime path values that miss the
generated include-unit registry now distinguish missing paths from existing
undeclared PHP source: missing `include` warns and returns false, missing
`require` warns then fatals with exit 255, and existing undeclared source
reports an explicit native source-loader/parser ABI blocker instead of
pretending the file is missing.

Recent source commit `5735906e` keeps arbitrary runtime PHP source loading
blocked while improving selected runtime include/require fidelity. Generated-C
runtime registry dispatch now calls a shared runtime no-match diagnostic ABI
when no declared include unit matches the runtime path. Focused gates cover the
full generated-C native include/require boundary suite, runtime native include
helpers, formatting, and diff checks. Runtime PHP source parsing/execution,
fully dynamic `include_path` mutation, stream wrappers, file URLs, non-UTF-8
path fidelity, Windows separators, open_basedir/stat-cache parity, source maps,
and LLVM/ASM parity remain blocked.

Recent source commit `792f9d3f` accepts leading-global fully-qualified direct
function calls such as `\strlen(...)`, `\array_push(...)`, and
`\App\Demo\local_value(...)` inside namespaces. The parser preserves the
leading global qualifier, namespace resolution keeps it out of
namespace-relative fallback, and generated-C execution routes selected global
builtins plus generated user functions through the shared exact callable
lookup/invocation paths instead of adding a one-builtin parser branch.

Recent source commit `792f9d3f` removes the parser boundary for selected
literal leading-global PHP function-call syntax. Focused gates cover parser
boundaries, namespace-resolution behavior, generated-C source proof, linked
execution over a generated user function and selected builtins, imported
builtin variadic writeback regressions, sort writeback regressions, formatting,
and diff checks. Namespace-relative `namespace\foo()` and namespace-qualified
`App\foo()` syntax without a leading global slash, broader unsupported
callable families, object/ArrayAccess/resource mutation operands, broad
COW/reference parity, and LLVM/ASM parity remain blocked.

Recent source commit `40b33bcd` routes selected runtime callable sorting
builtins `sort()` and `rsort()` through the shared materialized call-argument
and runtime callable builtin writeback path. Builtin signature metadata now
exposes a by-reference `$array` owner plus defaulted by-value `$flags`,
callable string canonicalization recognizes the selected sort family, and
runtime invocation mutates the caller-visible array through the existing native
array sort lvalue semantics.

Recent source commit `40b33bcd` advances selected sorting writeback without
adding source-shape lowering. Runtime callable strings and spread calls for
`sort()` / `rsort()` now share the same reference slot materialization,
defaulted argument finalization, flag coercion, and callable-value invocation
contracts used by other runtime builtin families. Focused gates cover runtime
callable sort helpers, native array lvalue sorting, builtin signature metadata,
generated-C source proof, linked executable writeback, by-reference builtin
writeback regressions, variadic writeback regressions, callable string spread
regressions, formatting, and diff checks. Key-preserving sort families,
comparator callbacks, full flag/diagnostic parity, object/ArrayAccess/resource
operands, unknown or heterogeneous callable families, magic fallback, broad
COW/reference parity, and LLVM/ASM parity remain blocked.

Recent source commit `22964e72` adds a generated include-unit registry plus
runtime lookup ABI for non-finite generated-C `include`/`require` path values
over declared generated include units. Runtime path operands can now be
converted to bytes, looked up against generated request/canonical aliases, and
dispatched through the same include unit, include_once/require_once, and result
propagation helpers as literal and finite dynamic include paths.

Recent source commit `22964e72` moves selected non-finite runtime
include/require operands past the old compile-time blocker without parsing
arbitrary PHP at runtime. Executable include discovery still discovers only
literal/finite declared units, but generated C now emits a compact registry of
request-path and canonical-path aliases and calls
`phpc_native_include_unit_registry_lookup()` to choose the declared include
unit at runtime. Focused gates cover generated-C source proof, linked
request-path/canonical-path include_once and require_once dispatch, include
path/missing/require regressions, runtime registry lookup, and diagnostic
severity regression. Fully arbitrary filesystem/source loading, dynamic
runtime `include_path` mutation, stream wrappers, file URLs, non-UTF-8 path
handling, open_basedir/stat-cache parity, exact warning/source-map fidelity,
and LLVM/ASM parity remain blocked.

Recent source commit `967181fd` removes the old imported/direct runtime
builtin split for represented function imports and aliases. Materialized call
arguments now have reusable reference-entry helpers, and source-call
materialization consults fixed parameter metadata to push ordinary or named
by-reference parameters as references before spread finalization. The focused
gates cover imported alias source/link execution, runtime callable builtin
writeback regressions, callable spread regressions, and by-reference variadic
regressions. Namespace-relative and namespace-qualified function syntax,
object/ArrayAccess/resource mutation operands, broader sorting families and
comparators/flags, unknown or heterogeneous callable builtin families, magic
fallback, broad COW parity, and LLVM/ASM parity remain blocked.

Recent source commit `e297477b` adds the selected runtime callable builtin
variadic writeback family for `array_push()` and `array_unshift()`. Builtin
signature metadata now exposes the shared by-reference `$array` parameter and
variadic `$values` tail, and runtime invocation mutates the referenced owner
through the existing array mutation semantics rather than a generated-C shape
path. Broader sorting families, object/ArrayAccess/resource mutation operands,
namespace-relative function syntax, unknown or heterogeneous callable families,
magic fallback, broad COW parity, and LLVM/ASM parity remain blocked.

Recent source commit `9a00643a` collects direct generated user-function and
declared-method by-reference variadic arguments through the normalized
call-argument plan. Reference-backed positional and named variadic entries now
flow into an array-valued frame slot and preserve caller-visible mutation,
while value-backed entries remain blocked where PHP requires a
reference/variable.

Recent source commit `9a00643a` routes direct function, static method, and
instance method by-reference variadic collection through shared normalized
call-argument and native array reference-slot helpers. The focused gates cover
direct source and linked execution, reference-backed and value-backed spread
behavior, runtime callable byref regressions, callable-family regressions, and
runtime materialized/reference helpers. Typed/default by-reference variadics,
descriptor-closure by-reference variadics, runtime builtins, magic fallback,
unknown-signature callables, broad COW parity, and LLVM/ASM parity remain
blocked.

Recent source commit `d4906e18` selects generated-C `print_r(<native value>,
$return)` return mode at runtime for generated native PHP values. The shared
runtime formatter ABI now evaluates the return-mode operand by PHP truthiness,
either returning an owned formatter string or writing formatter output and
returning PHP `true`, while named/spread arguments, unsupported non-native
operands, broad object formatting, output buffering, and LLVM/ASM parity
remain blocked.

Recent source commit `d4906e18` routes selected generated-C `print_r(<native
value>, $return)` expression values through the shared native value formatter
return-mode ABI. Generated metadata arrays from `get_declared_interfaces()`,
`class_implements(...)`, `get_declared_traits()`, and `class_uses(...)` can now
use runtime-selected truthy/falsey second operands without compile-time
literal folding or per-fixture array traversal. Arbitrary object `print_r`
parity, core/external metadata discovery, general output buffering,
named/spread arguments, unsupported non-native operands, and LLVM/ASM parity
remain blocked.

Recent source commit `e829222b` supports selected custom character masks for
the trim family across the shared runtime callable builtin and interpreter
paths. `trim`, `ltrim`, and `rtrim` now share byte-mask parsing for default
masks, empty masks, and incrementing `x..y` ranges over string and
binary-string values, while malformed/decrementing ranges and ambiguous dot
runs remain centralized blockers instead of fixture-specific behavior.

Recent source commit `e829222b` aligns generated-C runtime callable trim
handling and `phpc run` interpreter trim handling on the same selected byte-mask
semantics. The focused gates cover runtime trim helpers, direct trim builtin
tests, runtime callable string spread defaults, imported runtime builtin aliases,
and generated-C source proofs without adding source-shape lowering. Full PHP
charlist warning-and-continue behavior, ambiguous dot-run parsing, locale or
Unicode breadth, by-reference/writeback builtins, unknown callable families,
magic fallback, and LLVM/ASM parity remain blocked.

Recent source commit `3ae68adb` forwards selected magic fallback spread call
sources through the shared materialized call-argument carrier. Generated-C
programs that call missing methods with spread or named arguments can now clone
value/reference materialized entries into `NativeCallArgumentsHandle` in source
order, preserve source keys for magic `$args` packing, and dispatch through the
existing runtime magic method boundary. Runtime-unknown dynamic method-name
magic spread, by-reference unpack through magic `$args` as full PHP parity,
broad unknown signatures, broad COW parity, and LLVM/ASM parity remain blocked.

Recent source commit `3ae68adb` adds a reusable runtime helper that forwards
materialized value/reference call-argument entries into an existing
`NativeCallArgumentsHandle`. Magic fallback spread lowering now shares the
same materialized-entry producer as ordinary finalized calls, then forwards
source-order entries into the runtime magic carrier before invoking
`__call`/`__callStatic`. The focused source and linked proofs cover magic
spread dispatch while preserving callable-family, runtime callable, descriptor
closure, builtin callable, and by-reference spread regressions.

Recent source commit `2841e79a` folds callable-variable by-reference fact
preservation into the reusable symbol-table metadata helpers. Runtime-held
callable arrays and invokable objects assigned through direct variables keep
their finite callable identity sets after root-symbol-table writes, including
materialized native value writes and unset cleanup. The generated source and
linked executable proofs cover static method, instance method, and
`__invoke` callables mutating reference-backed spread slots through the shared
materialized argument finalizer.

Recent source commit `c64a0e21` routes selected trim-family runtime builtin
callables through the shared source-call and materialized-argument contracts.
Compiler signature metadata now marks `trim`, `ltrim`, and `rtrim` as
runtime-callable value builtins with defaulted `characters`, and the runtime
callable builtin table can look up and invoke those families over materialized
spread arguments. The accepted subset covers default `trim()` masks and simple
explicit ltrim/rtrim masks while preserving blockers for unsupported mask
ranges, by-reference/writeback builtin semantics, unknown callable families,
magic fallback, and LLVM/ASM parity.

Recent source commit `8a02fbbf` prints selected generated runtime metadata
arrays through the shared native value formatter. Discarded generated-C
`print_r(<native value array>)` now routes through the runtime formatter ABI,
recursively walks `PhpArray` entries, and covers
`print_r(get_declared_interfaces())` plus
`print_r(class_implements(...))` without per-fixture array traversal. String
return mode, non-discarded `print_r()` expression values, core/external
interface discovery, broad formatting parity, and LLVM/ASM parity remain
blocked.

Recent source commit `b4881053` registers selected generated trait metadata in
the shared runtime class-metadata registries. Generated-C `trait_exists()`,
`get_declared_traits()`, and direct `class_uses()` calls now reuse runtime
metadata declaration/value helpers, direct trait-to-class metadata edges, and
native value array consumers instead of generated per-fixture trait arrays.
Parent-recursive `class_uses()` helper patterns, autoload-discovered traits,
side-effecting include timing, core/external trait metadata, and LLVM/ASM
parity remain blocked.

Recent source commit `98e9f154` allows bounded literal `include_once` /
`require_once` cycles during generated-C executable include unit discovery and
lowering. The root file now has an include-once slot, active include units can
treat recursive `_once` edges as duplicate truth instead of fatal cyclic graph
blockers, and non-`_once` include cycles still reject before native link.
Dynamic paths, include_path search, stream wrappers, missing include
warning/false recovery, require fatal fidelity, and LLVM/ASM parity remain
blocked.

Recent source commit `98c8937b` lets selected generated native metadata arrays
participate in shared native value-array consumers. The native value array
query boundary now supports `count()` over any `NativeValueHandle` containing
an array, so generated-C `count(get_declared_interfaces())` and
`count(class_implements(...))` reuse the same runtime array/value path as
existing metadata queries rather than metadata-specific lowering. Printable
recursive native array traversal, core/external/autoload-only interface
discovery, and LLVM/ASM parity remain blocked.

Recent source commit `2d1144a4` preserves reference-backed unpacked slots
inside selected by-reference variadic calls. The materialized call-argument
finalizer can now build a finalized by-reference variadic array from unpacked
reference slots for generated user functions and method-backed callable arrays,
so mutations to `$values[0]` or named variadic entries are visible through the
caller's original cells. Value-backed unpack entries still reject
by-reference variadic parameters, and non-spread by-reference variadics,
runtime builtins, magic fallback, unknown signatures, broad COW parity, and
LLVM/ASM parity remain blocked.

Recent source commit `e7d22c9c` coerces selected typed variadic
descriptor-closure spread entries through a reusable variadic collection
call-type helper. Descriptor-closure spread still uses the shared materialized
call-argument finalizer and finalized variadic slot, but closure-frame binding
now applies closure parameter metadata to each finalized variadic collection
entry before exposing the typed variadic parameter. By-reference variadic
descriptor closures, by-reference unpack transfer, runtime callable strings,
magic fallback, unknown-signature callable spread, and LLVM/ASM parity remain
blocked.

Recent source commit `f038ae76` admits destructor-observable allocations inside
generated-C `try/finally` when the allocated object is already eligible for
request-end finalization. Request-finalizable destructors no longer trip the
try/finally cleanup preflight, finalizer setup forces the existing callable
table initializer for destructor-only programs, and linked native execution
preserves `try` body output, `finally` output, fallthrough output, then
request-end destructor output. Unsupported destructor shapes, shutdown
functions, broader exception/fatal unwinding, and exact cleanup-stack parity
remain blocked.

Recent source commit `aecbc310` routes selected runtime builtin callable-string
spread calls through reusable builtin signature metadata and the shared
materialized argument path. Runtime string-valued callables that can only name
supported homogeneous builtin families now build the same source-call argument
contract used by generated functions and methods, materialize/unpack spread
entries, finalize them through `NativeCallArgumentsHandle`, and invoke through
existing runtime builtin callable helpers. Builtins without runtime callable
families or default-value support, by-reference/writeback builtins, unknown or
heterogeneous runtime callables, magic fallback, and broader by-reference
unpack remain blocked.

Recent source commit `b8b29d93` routes bounded literal generated-C
include/require execution through reusable include-unit functions. `phpc
compile --emit-exe` now discovers literal included files as executable units,
emits one canonical include function per bounded path, executes ordered
top-level statements through the active root symbol table, returns tagged
include results for normal completion, explicit included-file `return`, and
duplicate `_once` truth, and tracks `_once` state by canonical path. Dynamic
paths, include_path search, stream wrappers, missing include warning/false
recovery, require fatal fidelity, cyclic include graphs, and LLVM/ASM parity
remain blocked instead of gaining exact-shape include lowering.

Recent source commit `0bb20ff7` routes selected descriptor-backed variadic
closure spread calls through a finalized variadic slot in the closure-frame
ABI. Materialized call-argument finalization now marks handles that contain a
packed variadic collection; descriptor closure invocation forwards that final
slot as the closure variadic argument instead of double-packing it, while
ordinary non-finalized closure calls keep their existing surplus-argument
packing path. Typed variadic descriptor closures remain blocked until
finalized variadic collection entry coercion exists generally, and
by-reference unpack, magic fallback, and unknown-signature callable spread
remain outside this descriptor-only surface.

Recent source commit `1d844d6d` registers generated declared
interface value metadata for selected generated-C `get_declared_interfaces()`
and `class_implements()` execution. Declared interface order and resolved
class-interface implementation metadata now feed compact runtime registries,
and the existing native class metadata value helper can return generated
interface arrays and class implementation maps without per-fixture arrays in
codegen. Core/external/autoload-only interfaces, broader consumers of returned
native arrays such as `print_r()`/`count()`, and LLVM/ASM parity remain
outside this bounded registry surface.

Recent source commit `beb73e34` finalizes selected
generated-native destructors through a request-owned finalizer registry.
Eligible public non-static parameterless `__destruct` methods on generated
declared objects, including trait-composed destructors, now register allocated
objects, de-duplicate by object id, drain in LIFO request-end order, and invoke
through existing declared method metadata and frame machinery. Destructor body
reads observe final object property state. Cleanup-order-sensitive allocations
inside `try/finally`, shutdown functions, and unsupported destructor shapes
remain blocked instead of gaining ad hoc destructor calls.

Recent source commit `b85b4585` routes selected runtime-held
callable array/object spread calls through finite homogeneous callable identity
sets. Expression-level ternary-selected callable arrays and invokable objects
can now preserve declared method parameter metadata, feed the existing
callable-value source-call contract, materialize and finalize spread/unpack
arguments, and invoke through the shared callable-value helpers. Heterogeneous
parameter metadata, unknown-signature runtime callables, magic fallback
callables, branch-join callable identity merges, by-reference callable-family
unpack, and LLVM/ASM parity remain blocked.

Recent source commit `1de45e2f` routes selected runtime string-valued callable
spread calls when the possible generated user functions or scoped static method
strings are known and homogeneous. The path derives a reusable callable-value
contract from `CNativeCallableIdentity` metadata, uses preserved function or
method parameter metadata, finalizes materialized unpack entries through
`NativeCallArgumentsHandle`, and avoids string-specific dispatch ladders.
Runtime builtins, magic fallback, unknown/heterogeneous callable strings, and
descriptor-closure variadics remain blocked.

Recent source commit `1c0b8707` preserves real reference-backed array slots
when materialized spread/unpack arguments are finalized for direct
user-function calls. Unpacked array entries whose source slot is a
`PhpReferenceCell` now flow into `NativeCallArgumentSlot::Reference` instead of
being cloned as values, so `...$args` can satisfy a by-reference parameter and
mutate the original cell through the existing native reference handle and
finalizer machinery. Value-backed unpack entries still reject by-reference
parameters, and callable-array/object by-reference unpack, descriptor-closure
by-reference unpack, by-reference variadic unpack, broad COW parity, and
LLVM/ASM parity remain blocked.

Recent source commit `15b8f029` composes selected trait properties and trait
constants through `trait_semantics` before declared class metadata emission.
Compatible duplicate trait instance properties, static properties, and public
parser-supported constants now become ordinary declared class metadata and
reuse object allocation defaults, static-property storage, class-constant
lookup, trait method frames, and shared conflict diagnostics, including nested
trait property conflicts. Unsupported trait member forms, unsupported
initializer expressions, autoload-discovered traits, dynamic include timing,
destructor execution, broad visibility/reference/COW parity, and LLVM/ASM
parity remain blocked instead of gaining trait-specific access ladders.

Recent source commit `aed8d3e7` routes selected
descriptor-backed closure spread calls through preserved closure parameter
contracts. Descriptor-closure-only callable values can now derive a reusable
source-call argument plan, materialize unpacked entries, finalize them through
`NativeCallArgumentsHandle`, and invoke via the existing closure argument
carrier without guessing implementation signatures. Variadic descriptor
closures remain explicitly blocked because the current closure-frame ABI would
double-pack finalized variadic slots; runtime callable strings, magic fallback,
unknown-signature spread, and by-reference unpack still require broader
signature/materialized-entry boundaries.

Recent source commit `b07e2407` routes selected generated-C
trait constructors through the existing trait composition and declared
constructor frame machinery. Trait-composed public non-static `__construct`
methods that pass constructor frame validation can fill the normal class
constructor metadata slot, so direct and included trait constructors reuse
constructor lookup/invoke, declared frame setup, `$this`, called scope,
named/default/variadic/by-reference argument binding, dynamic constructor
class-name dispatch, and callable-family regressions. Class constructors still
win; duplicate, conflicting, malformed, static, or non-public trait
constructors, destructor-observable cleanup, trait properties/constants,
autoload-discovered traits, broad reference/COW parity, and LLVM/ASM parity
remain blocked.

Recent source commit `b17f08f7` routes selected interface-only
callable-family spread/unpack calls through declared interface method
parameter metadata. Callable arrays and callable objects whose receiver facts
only expose an interface can now use that interface method contract to feed
the materialized argument finalizer and existing callable/receiver invoke
boundaries, while implementation-homogeneity and non-spread interface
receiver safeguards remain intact. Heterogeneous interface parameters,
runtime-dynamic callable values, descriptor closures, magic fallback callable
spread, and by-reference unpack transfer remain blocked behind reusable
signature/materialized-entry gaps.

Recent source commit `b14186bc` routes selected generated-C
`interface_exists()` checks through declared generated-native interface
metadata. Literal and dynamic interface names now reuse the shared native text
membership helper against `declared_interfaces` / `declared_interface_order`,
including case-insensitive canonical and leading-backslash spellings, while
classes, missing no-autoload names, core/external interfaces, and autoload-only
interfaces remain outside this bounded generated-declaration surface.
Generated-C `get_declared_interfaces()` and `class_implements()` still need
shared runtime value registries before they can be claimed.

Recent source commit `10b40ead` routes selected generated-C trait method
execution through declared method frame carriers. Trait-composed effective
methods that pass declared-method frame validation now become normal declared
method metadata, so trait methods, aliases, visibility adaptations, direct
class overrides, and included trait declarations reuse the shared callable
table, receiver/source-call invocation, declared frame setup, called scope,
`$this`, named-argument, cleanup, and by-reference boundaries. Trait
constructors, destructor execution, trait properties/constants, unsupported
method bodies, autoload-discovered traits, broad reference/COW parity, and
LLVM/ASM parity remain blocked instead of gaining trait-specific dispatch
ladders.

Recent source commit `3db04bc3` routes selected callable-family spread/unpack
calls through the shared materialized call-argument bridge. Known method-backed
callable arrays and callable objects now use declared method frame parameter
metadata to build a `NativeMaterializedCallArgumentsHandle` and finalize it
into the existing `NativeCallArgumentsHandle` before invoking through the
shared callable lookup/method helpers. Direct user-function spread,
callable-array, and callable-object regressions remain green. Interface-only
callable-family spread, runtime-dynamic/descriptor-closure/magic-fallback
callable spread, and by-reference unpack transfer remain blocked behind
explicit reusable-boundary gaps instead of per-shape lowering.

Recent source commit `1554707a` extends bounded literal include/require
declaration discovery from class-only files to declaration-only class,
interface, and trait files. Included interface and trait declarations now enter
the same expanded declaration stream and include metadata record as classes;
included traits can feed generated-C trait metadata registration for root
classes, while included interfaces compose with the integrated interface
metadata boundary. Dynamic paths, side-effecting includes, runtime include
timing, include return values, `_once` runtime de-duplication, include_path and
stream wrappers, autoload discovery, and LLVM/ASM parity remain blocked behind
explicit diagnostics.

Recent source correction `f72c212e` stabilizes native assembly callable
fallback generation. Generated C native-runtime headers now always declare
`phpc_NativeScalarValue` when native-runtime prototypes are emitted, fixing
cc-fallback paths whose unconditional scalar-value prototypes previously
depended on a conditional typedef. The callable native-assembly filter and
compact native callable lookup-table IR filter pass on primary. This also
refreshes focused callable lookup assembly summaries to account for valid
`printf` output from folded lookup-result fixtures and does not move the coarse
progress bars.

Recent source commit `a7dcd288` routes selected generated-C interface method
dispatch through shared metadata and source-call boundaries. Declared user
interfaces now register generated-C metadata, parent-interface expansion, and
case-insensitive interface keys; declared classes validate public non-static
implementation methods against interface method metadata before class metadata
is emitted. Supported interface-typed receiver calls now reuse the existing
receiver method lookup/invoke path, preserving declared method precedence,
visibility/access context, source-order named arguments, defaults, variadics,
by-reference diagnostics, receiver/value cleanup, malformed callable metadata
rejection, and class alias/case behavior. Missing, heterogeneous,
autoload-only, or malformed interface/implementation metadata remains blocked
instead of gaining interface-specific dispatch ladders.

Recent source commit `5cbb90af` routes selected direct generated user-function
spread/unpack calls through a reusable materialized call-argument entry and
finalizer bridge. Supported direct calls now preserve source-order evaluation,
positional unpack, string-key named unpack, ordinary named source arguments,
default slots, variadic collection slots, duplicate and unknown-name
diagnostics, positional-after-named diagnostics, required-argument diagnostics,
and cleanup of materialized/default/slot ownership. Unsupported callable-family
spread/unpack shapes and by-reference unpack transfer remain blocked at shared
boundaries instead of gaining per-callable or per-arity lowering.

Recent source correction `fbf1581b` restores LLVM known-string
`function_exists` folding before the runtime text-membership fallback. Known
function-name values now reuse the shared `function_exists_result_for_value`
proof instead of leaking runtime helper calls into callable lookup-table IR.
Focused `array_column`, direct-literal, known-string sibling, and compact
native-callable lookup-table gates passed on primary. This corrects an LLVM
lowering regression and does not move the coarse progress bars.

Recent source correction `416f007a` aligns the shared trait composition
ordering boundary with PHP reflection behavior. Direct trait methods now
precede recursively imported trait methods, aliases are emitted at the target
method slot before the original method, and the corrected order is covered by
focused reflection, full `trait`, callable-object, and named-argument gates.
This corrects previously accounted trait metadata semantics and does not move
the coarse progress bars. Recent source commit `77cde10f` routes selected
generated-C
callable-array invocation through the shared callable/source-call machinery.
Known object-method callable arrays and class-string/static callable arrays
now classify as reusable callable values, preserve named argument source order
through `NativeCallArgumentsHandle`, and use the existing method/static
argument binding, visibility/access-context, receiver cleanup, malformed
metadata, by-reference diagnostic, and runtime lookup/invoke boundaries.
Malformed arrays, unknown methods, visibility failures, unsupported by-reference
literal shapes, spread/unpack callable-array calls, and callable-array
missing-method magic fallback remain blocked or diagnostic instead of becoming
fixture-specific array-literal lowering. Recent source commit `794af86c`
routes selected generated-C object property overloading through reusable
runtime property-magic helpers.
Missing or externally inaccessible instance property reads, writes, `isset`,
and `unset` now dispatch through public non-static `__get`, `__set`,
`__isset`, and `__unset` callable metadata using the shared lookup/invoke and
`NativeCallArgumentsHandle` boundaries, while visible declared public property
access still wins first. Private/static/malformed magic metadata rejects before
fallback, dynamic property names share the same mutation route, and
property-reference/COW shapes remain blocked instead of inventing fake `__get`
references. Recent source commit `bf2aa181` routes selected generated-C
dynamic constructor class-name expressions through a shared runtime
constructor-scope normalization boundary. Parenthesized `new (expr)(...)`
class names can now consume class strings or object receivers, normalize
generated-native class metadata with alias and case handling, verify that the
target class is allocatable, and then invoke constructors through the existing
constructor lookup/invoke and `NativeCallArgumentsHandle` path. Missing or
unknown classes and unsupported receiver values produce focused diagnostics;
autoload/general external classes, generic allocate-by-scope ABI, dynamic
constructor reference results, broad reference/COW parity, and destructor
cleanup ordering remain blocked. Recent source commit `7ea73c02` discovers
bounded literal same-repository include/require class metadata for
`phpc compile --emit-exe`.
Declaration-only included files reached through literal strings or `__DIR__`
concatenation are expanded before executable top-level statements, so generated
C can reuse the existing class, method, static-property, class-constant,
callable-table, class-alias/case, and runtime metadata boundaries for included
classes. Dynamic paths, missing/out-of-repository files, cyclic graphs,
side-effecting or autoload-registering include files, and late includes stay
behind explicit diagnostics instead of one-file fixture admission. Exact include
execution/return values, include-once runtime de-duplication, traits/interfaces
from included files, arbitrary autoload discovery, LLVM/ASM parity, and exact
PHP include timing remain blocked. Recent source commit `e9ed9af4` routes
selected generated-C computed static-property names through the shared
static-property lvalue and runtime property-name normalization boundary.
Supported `Class::${$name}`,
`$object::${$name}`, `self::${$name}`, `parent::${$name}`, and declared
method-frame `static::${$name}` paths now normalize dynamic names through
`phpc_native_static_property_name_from_value_with_diagnostic_and_free`, preserve
static-property storage identity, typed-property diagnostics, class alias/case
normalization, receiver cleanup, and read/write/compound/isset/unset behavior,
and keep unsupported dynamic/static reference-COW and top-level called-scope
shapes behind explicit blockers instead of generated literal-name ladders.
Dynamic receiver class-name constants, callable-object invocation, trait
effective-method metadata, direct receiver magic, static-property
storage/reference/offset/ArrayAccess references, constructor reference results,
reference-returning ArrayAccess owner stacks, descriptor closures, malformed
magic metadata, source-call references, exact imports, literal/relative class
constants, and class aliases remain green. Recent source commit `630f2b1f`
routes selected generated-C callable-object invocation through declared
`__invoke` metadata and the shared receiver-method source-call machinery.
Supported `$object(...)` calls now use access-context method lookup plus
`NativeCallArgumentsHandle`, preserving source-order named argument keys,
defaults, variadics, selected by-reference argument transfer, receiver/value
cleanup, and malformed metadata rejection through the runtime lookup/invoke
diagnostics. Static or private `__invoke`, unknown object metadata,
spread/unpack call sites, missing metadata, and unsupported by-reference
literal/non-cell shapes remain blocked instead of falling into fixture-specific
callable lowering. Recent source commit `f42f1f91` routes selected generated-C
dynamic receiver class-name constants through a reusable no-autoload receiver
normalization ABI. `$receiver::class` and `($receiver)::class` now consume
object or class-string receivers, normalize registered generated-native class
metadata and aliases without autoloading, preserve missing class-string
source spelling without lookup diagnostics, return owned native strings with
explicit cleanup, and keep unsupported receiver values behind a focused
dynamic class-name diagnostic. Literal `ClassName::class` remains the PHP
source-string form, while relative `self::class`, `parent::class`, and
method-frame `static::class` keep using the existing metadata-aware paths.
Recent source commit `b15a882c` adds a shared trait effective method metadata
composition boundary consumed by both interpreter metadata and generated-C
class metadata validation. Trait lookup, nested trait uses, aliases,
visibility adaptations, `insteadof` exclusions, direct class method overrides,
conflict diagnostics, recursion diagnostics, and case-insensitive trait/method
keys now flow through `trait_semantics` before generated-C class metadata is
validated. This does not claim trait method frame emission, dispatch, trait
properties/constants, destructor execution, interface dispatch, or
autoload-discovered traits. Recent source commit `ecef5dc3` routes selected
generated-C
static-property roots whose stored value is an `ArrayAccess` object into the
shared reference-owner `offsetGet()` boundary when the static-property offset
is used as a by-reference source. Literal class, object receiver,
class-string receiver, `self`, `parent`, and method-frame late-`static`
static-property roots preserve storage identity, visibility/type/missing
diagnostics, receiver-scope cleanup, and alias/write-through behavior instead
of reading the root by value or cloning a fake reference. Multi-hop
static-property ArrayAccess reference paths, computed/static/dynamic
reference-COW shapes, and new owner-stack/reference ABIs remain blocked.
Recent source commit `092288d3` routes selected
generated-C
dynamic class-constant receivers through the shared class/constant metadata
tables. `$class::CONST` and `$object::CONST` now normalize declared
class-string and object receivers, including class aliases, through the same
runtime class-constant lookup used by literal and relative receivers while
preserving inheritance, visibility, missing-class/missing-constant
diagnostics, receiver cleanup, and owned result cleanup. Named receiver/static
magic, constructor reference results, direct receiver magic, static-property
storage/reference/offset
references, reference-returning ArrayAccess owner stacks, descriptor closures,
malformed magic metadata, source-call references, exact imports, literal and
relative class constants, and class aliases remain green. Traits/interfaces,
include/autoload-discovered constants, unsupported initializer expressions,
broader diagnostic parity, references/COW, and LLVM/backend parity remain
blocked. Recent source commit `6672a448` routes selected generated-C
direct receiver calls with missing or externally inaccessible instance
methods through the shared receiver lookup-plus-invoke boundary when public
non-static `__call` metadata exists. Declared visible hits and class-context
private calls still bind through declared method metadata first, direct
fallback calls preserve source-order named `$args` keys through
`NativeCallArgumentsHandle`, and malformed public `__call` signatures reject
in the runtime callable metadata validator before magic `$args` packing.
Named receiver/static magic, constructor reference results, static-property
storage/reference/offset references, reference-returning ArrayAccess owner
stacks, descriptor closures, malformed magic metadata, source-call
references, exact imports, class constants, and class aliases remain green.
Unknown receiver identities, non-public/static `__call`, broader callable
object magic shapes, full `$args` reference/COW parity, traits/interfaces,
autoload breadth, spread/unpack, exact PHP diagnostics, and LLVM/backend
parity remain blocked. Recent source commit `ebabf95a` routes selected
generated-C
static-property array-offset reference sources through the shared
static-property storage and lvalue/reference boundary. Literal class,
object/class-string, `self`, `parent`, and declared method-frame `static`
receivers can now feed `Class::$prop[$key]`-style array paths into
by-reference arguments or direct reference assignment as storage-backed
reference carriers. Root array promotion writes back through the typed
static-property setter path, preserving storage identity, visibility/type
diagnostics, receiver-scope cleanup, and alias write-through instead of
cloning fake references. Static-property storage/reference/offset-mutation
paths, reference-returning ArrayAccess owner stacks, constructor reference
results, named receiver magic, descriptor closures, malformed magic,
source-call references, exact imports, class constants, and class aliases
remain green. Direct static-property offset reads outside reference transfer,
computed static-property names, top-level `static::$prop[...]` without
declared method-frame called scope, static-property ArrayAccess offset
references, magic/static-property overloading, traits/interfaces/effective
method tables, autoload breadth, broad references/COW, exact PHP diagnostics,
and LLVM/backend parity remain blocked. Recent source commit `4f0acdf2`
routes selected generated-C
nested ArrayAccess owner-stack descents through reference-returning
`offsetGet()` intermediates when declared facts prove the returned reference
is another ArrayAccess object. Direct-variable and visible property-held roots
can now assign or append through the returned reference using the shared
runtime reference dispatch ABI, preserving alias/write-through behavior and
avoiding fake parent `offsetSet()` commits. Runtime reference dispatch,
owner-frame cleanup, constructor/property facts, nested ArrayAccess
owner-stack paths, property-held roots, static-property offset mutation,
static-property references, constructor reference results, named receiver
magic, descriptor closures, malformed magic, source-call references, exact
imports, class constants, and class aliases remain green.
Reference-returning `offsetGet()` without proven object facts, arbitrary or
side-effecting `offsetGet()` bodies, unknown/mixed reference/COW facts,
static-property ArrayAccess roots, magic/static-property overloading,
spread/unpack, traits/interfaces/effective method tables, arbitrary
autoload/class discovery, broad references/COW, exact PHP diagnostics, and
LLVM/backend parity remain blocked. Recent source commit `d7b5bf98` routes
selected generated-C constructor reference results through the shared
constructor allocation/invoke and source-call argument carrier boundary.
Supported named declared constructors can now feed `new Class(...)` into
by-reference source-call arguments for direct function, dynamic function, and
constructor consumers by moving the allocated receiver into a real runtime
reference cell instead of copying a fake alias. Constructor allocation/invoke
diagnostics, value-return diagnostics, argument-handle ownership,
static-property offset mutation, static-property references, named receiver
magic, descriptor closures, nested ArrayAccess, malformed magic, source-call
references, exact imports, class constants, and class aliases remain green.
Dynamic constructor names, classes without constructors but with arguments,
constructor named arguments, spread/unpack, broader by-reference constructor
alias transfer, destructor-observable cleanup ordering,
traits/interfaces/effective method tables, arbitrary autoload/class
discovery, broad references/COW, exact PHP diagnostics, and LLVM/backend
parity remain blocked. Recent source commit
`9d2b3526` routes selected generated-C static-property array-offset mutation
through the shared static-property lvalue/storage boundary and ArrayAccess
owner-stack paths. Literal class, object/class-string, `self`, `parent`, and
declared method-frame `static` receivers now support selected static-property
offset assignment, append, compound/RMW updates, `??=`, and `unset()` without
materializing fake local owners. Recent source commit `f7facf37` preserves
source-order named argument keys for selected
generated-C receiver `__call` fallback calls through the shared
`NativeCallArgumentsHandle` metadata and runtime magic `$args` packing boundary.
Direct receiver calls with missing or inaccessible instance methods, plus
dynamic receiver calls with statically known method names, now carry named
`$args` keys when the compiler can prove the call site resolves to public
non-static receiver magic. Normal declared receiver hits still bind through
declared parameter metadata first, mixed declared-hit/magic-fallback facts stay
blocked, and malformed magic metadata still rejects before derived magic
argument packing. Recent source commit `ea6c7980` routes selected generated-C
static-property reference sources through shared runtime static-property
storage cells. Literal class, object/class-string, `self`, `parent`, and
declared method-frame `static` receivers can now materialize storage-backed
reference carriers for by-reference argument transfer and direct reference
assignment, preserving alias/write-through behavior without cloning values
into fake references. Reference writes use the diagnostic setter, so
typed-property constraints attached to static storage cells remain enforced,
and receiver-scope cleanup stays aligned with the existing static-property
lvalue boundary. Computed static-property names, top-level `static::$prop`,
static-property array-offset references, broader direct-variable
reference-target parity, magic/static-property overloading, traits/interfaces,
autoload breadth, broad reference/COW parity, and LLVM/backend parity remain
blocked. Recent source commit `3b3a740a` publishes magic-method
signature metadata through the shared generated-C callable table and rejects
malformed `__call`/`__callStatic` fallback targets before runtime magic
argument packing. Receiver `__call` and static `__callStatic` fallback lookup
now require valid two-argument, non-reference, string/array-compatible
signatures, while normal declared method hits still win and existing named
`__callStatic` source-order `$args` packing remains intact. Invalid metadata
reports through the shared runtime lookup-plus-invoke diagnostics instead of
falling into generated name ladders or exact-shape status branches. Full PHP
declaration-time warning/fatal timing, broader or mixed named dynamic receiver
`__call` key preservation, traits/interfaces/effective method tables, aliases/autoload,
callable-object magic shapes, full `$args` reference/COW parity, and
LLVM/backend parity remain blocked. Recent source commit `3149d1da` routes
selected generated-C
class constant reads through shared runtime class/constant metadata. Literal
`Class::CONST`, `self::CONST`, `parent::CONST`, and declared method-frame
`static::CONST` now share generated class/parent/constant tables, alias
canonicalization, inheritance lookup, visibility checks, owned-result
semantics, and runtime diagnostics instead of generated receiver-specific
string ladders. Relative `self::class`, `parent::class`, and `static::class`
are routed through the same class metadata context where lookup semantics are
needed, while literal `ClassName::class` remains the PHP no-lookup source
string form. Dynamic class-name constant receivers, unsupported initializer
expressions, traits/interfaces, include/autoload-discovered constants, broader
diagnostic parity, and LLVM/backend parity remain blocked. Recent source
commit `ae7b4535` preserves source-order named argument keys for selected
generated-C `__callStatic` fallback calls through the shared
`NativeCallArgumentsHandle` metadata and runtime magic `$args` packing
boundary. Missing or inaccessible static calls with public static
`__callStatic` now carry named `$args` keys for literal class,
object-static receiver, `self::`, and method-frame `static::` calls while
normal declared static method hits still bind through declared parameter
metadata first. Broader/mixed named dynamic receiver `__call`, spread/unpack, constructors,
unknown dynamic callables, full PHP declaration-time malformed-signature
parity, traits/interfaces, aliases/autoload, callable-object magic shapes,
full `$args` reference/COW parity, and LLVM/backend parity remain blocked.
Recent source commit
`14edee16` routes selected generated-C
static-property `unset()` through the shared static-property lvalue/storage
boundary for literal class, object/class-string, `self`, `parent`, and
declared method-frame `static` receivers. Runtime static-property storage now
has class and relative unset APIs, untyped storage resets to initialized
`null`, typed storage returns to the uninitialized state, and read/`isset()`/
`empty()` aftermath plus visibility, scope, and missing-property diagnostics
reuse the same storage boundary. Computed static-property names, top-level
`static::$prop`, static-property array-offset references, array-offset mutation,
magic/static overloading, traits/interfaces, autoload breadth, broad
references/COW, and LLVM/backend parity remain blocked. Recent source commit
`2e2625eb` routes selected generated-C
descriptor-closure reference returns into real reference carriers for proven
descriptor-backed closures. The supported path admits by-reference closure
returns from direct by-reference parameters or captures, invokes proven
descriptor closures through the runtime reference-return helper, and feeds the
owned reference into both by-reference argument transfer and direct reference
assignment materialization without copying values to fake aliases. Unknown or
mixed callables, callable arrays, invokable objects, method descriptors,
non-descriptor closures, unsupported closure reference return sources,
descriptor identity lost through symbol-table-only storage, broader
reference/COW ownership, and LLVM/backend parity remain blocked. Recent source
commit `16e7dec5` routes selected generated-C static-property null-coalescing
assignment through the shared static-property lvalue/storage boundary for
literal class, object/class-string, `self`, `parent`, and declared
method-frame `static` receivers. Recent source commit `3d32236a` routes
selected generated-C static-property `isset()` and `empty()` through the same
shared lvalue/storage boundary. Recent source commit `8c922fa2` routes
selected generated-C direct root ArrayAccess append-with-keyed-suffix
assignments such as `$bag[]["leaf"] = $value` through a generalized root
ArrayAccess owner boundary. The supported path materializes suffix keys before RHS evaluation,
wraps the RHS through the shared appended-slot value boundary, calls
`offsetSet(null, wrapped_value)` on the root ArrayAccess object, preserves
assignment-expression result ownership separately from the appended slot value,
and threads cleanup/diagnostics through the existing owner commit boundary.
Function-call suffix-key expressions, unknown root facts, arbitrary alias
roots, static-property roots, reference-returning `offsetGet()`, broad
references/COW, cleanup/unwind breadth, spread/unpack, and LLVM/backend parity
remain blocked. `7ff18dba` also routes selected generated-C constructor
allocation-plus-invoke through declared class metadata, typed-property-aware
allocation, shared `NativeCallArgumentsHandle` ownership, `$this` binding,
called scope, caller access context, allocated receiver cleanup, and dedicated
constructor value-return diagnostics. Recent source commits also route selected
generated-C nested ArrayAccess `unset()` and append-with-keyed-suffix
assignments through the generalized owner-stack path for direct-variable and
visible property-held roots. Supported unset paths materialize the root owner,
descend through by-value `offsetGet()` intermediates, execute leaf
`offsetUnset()`, reverse-write changed parents with `offsetSet()`, and commit
the original owner. Supported keyed append paths materialize suffix keys before
RHS evaluation, wrap the RHS as nested native arrays through the shared
appended-slot boundary, append at the owner-stack leaf with
`offsetSet(null, value)`, reverse-write parents, commit the original owner, and
keep assignment-expression result ownership separate from the appended slot
value. Reference-returning `offsetGet()`, arbitrary alias roots,
non-direct/unknown property holders, static-property roots, broad
references/COW, cleanup/unwind breadth, spread/unpack, and LLVM/backend parity
remain blocked. Recent source commits also route selected generated-C
static-property plain assignment, compound assignment, and pre/post
increment/decrement through a shared lvalue target for literal class receivers,
object/class-string receivers, and relative `self`, `parent`, and method-frame
`static` receivers. Supported paths read and write request-owned
static-property storage through the same runtime APIs, derive dynamic receiver
scope through the shared receiver-scope ABI, preserve expression-result
ownership for compound and pre/post operations, and keep parser
object-static-property lvalues flowing to codegen instead of falling behind
prefix/compound blockers. Computed static-property names, top-level
`static::$prop`, static-property array-offset references, array-offset
mutation, magic/static overloading, traits/interfaces, autoload
breadth, broad references/COW, and LLVM/backend parity remain blocked. Recent
source commits also route selected generated-C nested ArrayAccess
null-coalescing assignments through the generalized owner-stack path for
direct-variable and visible property-held roots. Supported paths materialize
the root owner, descend through by-value `offsetGet()` intermediates, probe the
leaf with `offsetExists()`, read present leaves with `offsetGet()`, lazily
evaluate the RHS only for missing/null leaves, write mutated leaves with
`offsetSet()`, reverse-write parents only on mutation branches, commit the
original owner, and preserve the `??=` expression result. Recent source
commits also route selected generated-C static
method source calls through the shared runtime `__callStatic` fallback
boundary. Normal declared static method hits still win, missing or inaccessible
static methods fall back to public static `__callStatic($name, $args)`, and
non-static methods called statically remain hard failures rather than falling
through to magic. The integrated paths cover literal `Class::method(...)`,
object-static receivers, `self::`, `parent::`, and bounded declared-frame
`static::` source calls without generated method-name ladders or fixture-shaped
dispatch. Full PHP declaration-time malformed signature parity, named dynamic
receiver magic fallback, traits/interfaces/effective method tables, aliases/autoload,
callable-object static magic shapes, full `$args` reference/COW parity, and
LLVM/direct assembly parity remain blocked. Recent source commits also
initialize generated-C declared static-property storage through the shared bulk
metadata/default ABI. Generated C now consumes declared-property metadata arrays
for names, visibility, type declarations, defaults, and static flags, registers
defaults only for static properties, and preserves non-static metadata so static
lookups still honor shadow boundaries. Typed static-property defaults and writes
now report through the runtime static-property type/visibility diagnostics
instead of using the old single-property generated-C registration path. Dynamic
class/property static shapes, static-property references, append/nested
static-property offset mutation, magic/static overloading, traits/interfaces,
autoload breadth, full type parity, and LLVM/backend parity remain blocked.
Recent source commits
also route
descriptor-backed closure calls through shared `NativeCallArgumentsHandle`
production and runtime closure-invoke call-result carriers. Proven
descriptor-closure callable facts now bypass the broad dynamic callable name
path, evaluate arguments through the same source-call argument machinery as
functions and methods, and consume result/value/reference/discard closure
invocation helpers without matching one fixture, arity, or local variable name.
Unknown or mixed callable identities, callable arrays, invokable objects,
non-descriptor closure handoff, descriptor/method/closure reference-return
breadth, spread/unpack, broader request-state handoff, and LLVM/backend parity
remain blocked. Recent source commits also route selected generated-C object
and declared class-string static-property receivers through the shared runtime
receiver-scope ABI before using request-owned static-property storage.
Supported paths cover direct reads and plain assignments such as `$object::$p`
and `$classString::$p`, preserve assignment expression results, free receiver
and scope handles on failure paths, and reuse the same static-storage read/write
ABI as literal and relative static-property producers. Top-level
`static::$prop`, dynamic property names, static-property references, compound
mutation/unset, magic/static overloading, traits/interfaces,
autoload breadth, LLVM parity, and reference/COW static-property breadth remain
blocked. Recent source commits
also route selected generated-C nested
ArrayAccess pre/post increment and decrement through the generalized nested
owner-stack write context for direct-variable and property-held roots.
Supported paths descend through by-value `offsetGet()`, read the leaf, compute
the native increment/decrement replacement, preserve pre/post expression-result
semantics, write the leaf with `offsetSet()`, perform reverse parent writeback,
and commit the root owner; newer append, null-coalescing assignment, unset, and
keyed-append-suffix paths reuse the same owner stack for direct and
property-held roots. Root keyed-suffix append without owner-stack descent,
reference-returning `offsetGet()`, arbitrary
alias roots, broad references/COW, cleanup/unwind breadth, spread/unpack, and
LLVM/backend parity remain blocked. Recent source
commits also add parser/AST source-ordered named call-argument nodes plus shared
call-argument normalization that binds source-order positional/named arguments
to parameter-order slots across required, optional/default, by-reference, and
variadic parameters. Generated C now lowers selected compiler-known direct
user-function calls and method/static/dynamic source-call carriers through that
normalizer before building shared `NativeCallArgumentsHandle` values. Named
builtins, constructors, unknown dynamic callables, magic `__call`/`__callStatic`
fallback named-argument parity, spread/unpack, malformed magic
signatures, broader runtime/inherited/trait/interface signatures, LLVM/direct
assembly parity, and backend-wide named-argument parity remain blocked. Recent
source commits also route generated-C dynamic receiver method calls on compiler-known object
receivers with declared instance `__call($name, $args)` through the shared
runtime lookup-plus-invoke dispatcher. Normal method hits still win, missing or
inaccessible receiver methods fall back to public non-static `__call`, and
runtime packs the original method name plus value snapshots of original
argument slots into the magic `$args` array while preserving class-context
private method dispatch. Named-argument `__callStatic` fallback, malformed
magic signature warning/fatal parity, traits/interfaces/effective method tables,
aliases/autoload,
callable-object fallbacks, full reference/COW alias behavior for `$args`, and
LLVM/direct assembly parity remain blocked. Recent source commits also lower exact
parser-resolved generated-C `use const` aliases for same-compilation-unit scalar
user constants and supported builtin constants through shared constant
metadata/import markers.
Exact imported misses reject without namespace/global fallback, and ordinary
bare constants, arrays and dynamic constant expressions, duplicate/builtin
collisions beyond explicit rejection, dynamic class-constant receivers,
broader class-constant initializer/diagnostic parity, include/autoload
discovery, dynamic alias roots, function-frame constant lookup, LLVM constant
lowering, and broad backend/global constant parity remain blocked. Recent
source commits also route generated-C `static::$prop` reads/writes in declared
method frames through the same shared relative static-property runtime storage
used by `self::$prop` and `parent::$prop`, threading the active method-frame
called scope into the runtime ABI so root and descendant static storage stay
distinct. Top-level `static::$prop`, dynamic class/property names,
static-property references, compound
mutation/unset/isset/empty, magic/static overloading, traits/interfaces,
autoload breadth, LLVM parity, and reference/COW static-property breadth remain
blocked (`bf8fd335`); route selected generated-C nested ArrayAccess compound
assignments through
the generalized nested owner-stack path for direct-variable and property-held
roots: each supported path descends through by-value `offsetGet()`, reads the
leaf, computes the native binary replacement value, writes the leaf with
`offsetSet()`, performs reverse parent writeback, and commits the root owner.
Nested `??=`, append, increment, decrement, reference-returning `offsetGet()`,
arbitrary alias roots, broader reference/COW semantics, cleanup/unwind breadth,
and LLVM parity remain blocked (`ba5769e2`); route generated-C `self::$prop`
and `parent::$prop` reads/writes from declared method frames through shared
relative static-property runtime storage, with one program-wide request storage
handle shared across top-level and method-frame static-property operations
(`cfc7c0ee`); route generated-C receiver
method calls with unknown runtime method-name expressions through the shared
lookup-plus-invoke source-call carrier when the receiver has compiler-known
object class facts and no declared `__call`. Runtime lookup now normalizes
scalar dynamic method-name values before access-context lookup, preserving
non-scalar diagnostics and static-through-object rejections. Declared
`__call`/broader `__callStatic`, magic argument packing, traits/interfaces, arbitrary
aliases/autoload, callable-object fallbacks, broader byte/encoding diagnostic
parity, and LLVM parity remain blocked (`7ae07fd7`); route
bounded generated-C
`static::method(...)` calls in declared method frames through runtime
called-scope handles and static source-call carriers, keeping lexical class
context only for visibility/access checks. Descendant-only targets,
override-visible late-static breadth, traits, interfaces, magic
broader `__callStatic`, dynamic static properties, `static::class`, `static::$prop`,
LLVM parity, and broad inheritance/interface resolution remain blocked. Recent
source commits also lower selected generated-C
declared typed instance properties by emitting per-property type/default
metadata, allocating objects through the runtime typed-property metadata ABI,
initializing default values, and routing known typed instance-property writes
through diagnostic mutation so type failures report instead of mutating
silently. Typed static properties, dynamic/magic properties, unsupported object
type declarations, broad reference/COW property semantics, and LLVM/backend
parity remain blocked (`f1816273`); preserve bounded generated-C
function and method return-through-finally cleanup by queuing finalizer output
cleanup operands into the terminal-kind return transfer before return-value
handoff, while keeping top-level try returns, throw/catch, exit/goto unwind,
finally-return replacement, source-call cleanup in return operands, destructor
ordering, and full exception/finally semantics blocked (`2ca9d0e3`); lower
selected generated-C nested ArrayAccess assignments through a
generalized owner stack for direct-variable roots and property-held roots: each
supported path descends through by-value `offsetGet()`, writes the leaf with
`offsetSet()`, performs reverse parent writeback, and commits the root owner
while keeping reference-returning `offsetGet()`, nested RMW, `??=`, append,
increment, decrement, and broader COW/reference forms blocked (`e8268187`);
they also lower exact parser-resolved generated-C `use function` aliases for
same-compilation-unit user functions and selected signature-backed runtime
callable builtins through the shared source-call argument and lookup/invoke
stack, preserving exact missing imported runtime names and unsupported imported
builtin lookup-before-argument guards while keeping dynamic alias roots,
include/require discovery, autoload, broad runtime builtin imports, native
const-import lowering, and LLVM/backend parity blocked (`5bacf1aa`); route
compiler-known generated-C dynamic receiver-method calls with known string names
through the shared source-call carrier stack, including class-context access for
declared method frames and runtime rejection of static descriptors used through
object dispatch, while keeping declared `__call`, broad magic dispatch,
traits/interfaces, unknown scalar method names, late-static behavior, and
unsupported receiver shapes blocked (`5a4f10a5`); route selected
generated-C property-held ArrayAccess append and pre/post increment/decrement
through real object-property reference owners, including conversion
diagnostics, expression-result ownership, owner writeback, and unsupported-owner
blockers while keeping nested offsets, unknown dynamic properties,
reference-returning `offsetGet()`, static-property owners, broad COW/reference
semantics, and backend parity blocked (`ad74d35b`); route generated-C receiver
and named static calls made from inside declared method frames through runtime
class-context access checks, so supported declared methods can call private
instance methods on `$this` and protected static methods on the declaring class
through shared source-call carriers while external receiver/static callers and
dynamic fallback ladders stay public-only (`99d23aa4`); route generated-C
object static-receiver method calls through shared
static source-call carriers by deriving an owned class scope from object or
known class-string receivers, building the shared
`NativeCallArgumentsHandle`, preserving default/variadic frame-compatible
static method calls, and keeping `static::`, magic, traits, interfaces, full
late-static binding, and unsupported receiver shapes blocked (`3eb101aa`);
lower generated-C literal declared `Class::$prop`
reads and writes through request-owned runtime static-property storage,
including default initialization, reset, type and visibility checks, canonical
class/alias lookup, expression-result ownership, and unsupported
dynamic/static-member shape blockers (`22b3074f`), route generated-C `self::`
and `parent::` static source calls through declared class-context lookup,
shared `NativeCallArgumentsHandle` binding, and source-call carriers, including
protected/private callable-table metadata for runtime access checks
(`b4503c2e`), publish source-call signature metadata for selected runtime
callable builtins (`strlen`, string case/value helpers, `gettype`,
`is_numeric`, and `str_contains`) while keeping unsupported runtime builtins
like `count` blocked before argument construction (`7e9a237b`), let
generated-C by-reference arguments consume proven reference-return source-call
results (`defad66a`), add generated-C class alias
metadata without fake autoload success (`bae8d3fe`), route explicit by-value
generated-C function and method returns through the terminal-kind cleanup
handoff (`01da56ce`), add request-scoped runtime storage for declared static
properties (`d6798eb8`), route generated-C constructor `return <expr>;`
through an explicit diagnostic with cleanup (`bc04035b`), route selected
generated-C non-local object-property
assignments through true public-property reference owners (`cabdcde6`), route
selected generated-C property-held ArrayAccess `unset()` through true
object-property reference owners (`96ad8464`), route direct generated-C user-function
reference-return frames through native reference-returning frame signatures
(`b3e2a724`), add parser/runtime `use const` exact lookup (`e81aa43e`), route
selected property-held ArrayAccess read/write/RMW/`??=` owners through real
object-property reference owners (`6757bb43`), broaden generated-C
receiver/static method source calls to default/variadic frame-compatible
arities (`8886d9e5`), add parser/runtime `use function` exact lookup
(`c3968d0a`), allow bare `return;` in supported constructor bodies
(`bd0eafd0`), add object-property owner/fact/commit prerequisites
(`b3f16040`), LLVM user-class metadata parity (`d96cc2bb`), a generated-C
namespace/import/class-name/autoload-policy boundary (`cb8457f1`), and
terminal-kind diagnostic-result ABI support (`ac5004a3`).

Why the headline bars moved: the new source removes primary blockers
across object/class execution and backend parity: source-ordered named
arguments now parse into explicit AST nodes and selected generated-C direct
user-function plus method/static/dynamic source-call carriers bind them through
shared parameter-order normalization instead of treating them as unsupported
syntax or exact-shape call lowering,
selected nested ArrayAccess pre/post increment and decrement now reuse the
owner-stack descent, leaf mutation, reverse writeback, and root-commit boundary
instead of staying behind the broad non-assignment mutation blocker,
generated method frames can
write `$this` properties, selected constructors with supported bodies and bare
early returns run, constructor value returns now fail through an explicit
generated-C runtime diagnostic with cleanup, method/static source calls handle
default and variadic frame-compatible arities, `self::` and `parent::`
static source calls now run through class-context source-call carriers,
declared method-frame calls can now invoke supported private/protected
receiver/static methods through runtime class-context lookup-plus-invoke
carriers,
compiler-known dynamic receiver-method calls with known string names now reuse
source-call carriers instead of generated name-comparison ladders, including
class-context private/protected access from declared method frames,
runtime-produced dynamic receiver-method names now also route through runtime
lookup-plus-invoke carriers for compiler-known object receiver facts,
declared instance `__call` receiver fallbacks now share that runtime dispatch
boundary and pack PHP magic `$name`/`$args` values instead of reviving generated
comparison ladders,
object static-receiver calls now derive runtime receiver scope and run through
shared static source-call carriers for exact/default/variadic
frame-compatible declared static methods,
bounded declared-frame `static::method(...)` calls now use runtime called scope
instead of lexical `self::` dispatch while preserving lexical access context,
literal declared `Class::$prop` reads and writes now execute through
request-owned runtime static-property storage instead of being rejected,
declared-frame `self::$prop` and `parent::$prop` now share that request storage
through relative static-property receivers,
declared-frame `static::$prop` now shares it with the active called-scope
receiver so inherited static-property methods can read and write descendant
storage without collapsing to lexical `self`,
object and declared class-string static-property receivers now derive runtime
receiver scope and reuse request-owned static-property storage for direct reads
and plain assignments,
declared typed instance properties now allocate with runtime type/default
metadata, initialize defaults, and enforce typed writes through diagnostic
mutation in generated C,
generated-C exact `use function` aliases now execute for same-unit user
functions and selected runtime callable builtins through shared source-call
arguments, while unsupported imported runtime builtins fail before argument
side effects,
generated-C exact `use const` aliases now execute for same-unit scalar user
constants and supported builtin constants through parser/import metadata while
ordinary bare and broader constant lookup shapes remain blocked,
descriptor-backed closure calls now use shared call-argument handles and
closure invoke result carriers instead of falling through the broad dynamic
callable name path,
selected nested ArrayAccess assignments now execute through owner-stack
descent, leaf write, reverse parent writeback, and root commit for direct and
property-held roots,
selected nested ArrayAccess compound assignments now reuse that owner-stack
boundary with leaf reads, native binary replacement values, reverse parent
writeback, and direct/property root commits,
bounded generated-C function and method returns through active `finally` bodies
now preserve finalizer stdout/diagnostics as terminal-transfer cleanup operands
before returning the original value,
interpreter function imports resolve exactly,
interpreter const imports resolve exactly before namespace/global fallback,
direct generated-C user-function frames can return references through shared
source-call carriers and by-reference consumer paths, proven reference-return
direct/dynamic/receiver/static source calls can feed by-reference arguments
through shared carriers, selected runtime callable builtin signatures now feed
dynamic source-call argument binding while unsupported builtins are rejected
before side-effecting arguments are built, selected property-held
ArrayAccess owners now cover append and pre/post increment/decrement in
addition to `unset()` through the same reference-owner commit path as
reads/writes/RMW, and selected external object-property assignments reuse true
public-property reference owners instead of temporary mutation
shortcuts,
explicit generated-C by-value function/method returns now pass through the
terminal-kind cleanup-transfer ABI before returning values to existing frame
callers,
runtime static-property storage now has request-scoped default initialization,
type/visibility enforcement, reference identity preservation, and request reset
coverage for later backend producers,
generated C handles parser-resolved namespace/import class policy without fake
autoload success and now records generated-C class aliases through normalized
metadata/canonical lookup boundaries, while missing-source autoload remains
blocked,
LLVM can declare and query user-class metadata through the
shared runtime ABI, terminal transfer now carries return/throw/exit kind, and
object-property owner facts now drive selected property-held and nested
ArrayAccess production. The bars remain far from 100% because root keyed-suffix
append without owner-stack descent, reference-returning ArrayAccess owners, static magic named-argument
and signature breadth,
malformed magic signature parity, remaining override-visible late-static and
broader magic method shapes,
spread arguments and unsupported named builtin/constructor/fallback call
families, broader const discovery/lookup and function-import
coverage, broader
constructor execution, late-static binding,
arbitrary alias roots, LLVM and remaining generated-C static-property producers,
broader static-property reference/`??=`/unset/isset/empty and dynamic/magic property breadth,
exceptions/cleanup, full SPL
autoload, visibility/magic breadth, and backend parity are still open.

Current critical path to 100%:

1. Extend expression-owned `NativeCallResultHandle` carriers and production
   source-call lowering over the lookup-plus-invoke ownership helpers,
   including remaining dynamic/magic method shapes and late-static override
   breadth,
   constructor allocation, non-descriptor closure handoff, broader by-reference alias
   transfer, and spread ownership.
2. Continue migrating expression, statement, terminal, cleanup, lvalue,
   reference, and call-argument lowering onto produced
   `NativeDiagnosticResult` operands.
3. Route value/reference/return/deferred-cleanup consumers through the shared
   diagnostic-result and call-result carrier stack.
4. Broaden the integrated object-property owner/fact/commit boundary from
   selected non-local assignments, property-held ArrayAccess
   read/write/RMW/unset/append/increment, and selected nested
   assignment/RMW/increment/decrement/append/`??=`/unset/keyed-append suffix
   into root keyed-suffix append, reference-returning forms,
   references/COW, arbitrary alias-root writeback, and broader
   object/static/dynamic/magic property storage and broader static-property
   mutation/reference surfaces.
5. Implement real exception/Throwable propagation, catch/finally/destructor/
   shutdown cleanup, source-ordered diagnostics, and custom handler behavior.
6. Broaden namespace/const import production breadth, function-import
   discovery/fallback breadth, namespace fallback, autoload, broader aliases,
   visibility/magic, constructors, spread arguments, unsupported named
   builtin/constructor/fallback call families,
   descriptor/method/closure return references, and backend parity.

## Roadmap Bars

| Workstream | Integrated | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **90%** | `[##################--]` | Strong selected-path value, byte-string, array, reference, symbol, callable, call-frame/result, diagnostic-result, terminal-kind, request-state, lvalue, ArrayAccess, class metadata value, class constant metadata/lookup tables, descriptor-closure call-result helpers, request-scoped static-property storage plus bulk metadata/default registration, object/class-string receiver-scope resolution, and static-property unset storage APIs, typed instance-property metadata/allocation/write diagnostics, selected receiver `__call` and static `__callStatic` dispatch, generated-C class alias metadata, function/const-import exact lookup, and autoload-policy boundary surfaces. Remaining gaps include arbitrary alias transfer, full autoload, namespace fallback, malformed magic signature parity, broader closure frame handoff, cleanup/unwind parity, and broader lookup parity. |
| Compiler/backend consumers | **85%** | `[#################---]` | Generated C has the freshest consumers for calls, callable facts, selected descriptor-closure calls, selected object/class metadata, selected literal/relative/dynamic class constant metadata consumers, namespace/import class policy, exact function-import aliases for same-unit user functions and selected runtime builtins, exact const-import aliases for same-unit scalar user constants and supported builtins, typed declared instance-property allocation/write metadata, selected bulk typed static-property metadata/default registration, selected literal, object/class-string receiver, and relative `self`/`parent`/`static` static-property read/write/observation/mutation/`??=`/unset storage, selected ArrayAccess/lvalue paths, selected non-local object-property assignment owner commits, value-result casts, explicit by-value return terminal handoff, diagnostic-result family consumers, discarded statement-expression diagnostic operands, echo/print output diagnostic operands, and control-transfer cleanup report bridging. It still rejects ordinary/broader constant lookup, dynamic receiver `::class`, and unsupported function-import shapes at explicit production boundaries. LLVM shares user-class metadata declaration/exists routing plus the discarded-expression, output operand, and cleanup report bridge paths, while direct assembly still lags newer object-offset/lvalue/static-property/runtime ABIs and most semantic result operands remain unmigrated. |
| Executable PHP semantics | **85%** | `[#################---]` | Many executable islands exist, including bounded method/static/dynamic-receiver and descriptor-closure source-call production, generated method-frame `$this` property assignment, selected non-local object-property assignment commits, selected declared literal, object/class-string receiver, and `self`/`parent`/`static` static-property reads/writes plus selected static-property observation/compound/increment/decrement/`??=`/unset producers, selected typed static-property metadata/default registration and writes, selected declared typed instance-property defaults and typed writes, selected constructor bodies with bare early returns, selected property-held ArrayAccess read/write/RMW/`??=`/unset/append/increment owners, selected nested ArrayAccess assignment, compound-assignment, pre/post increment/decrement, append, null-coalescing assignment, unset, and keyed-append-suffix owner stacks for direct and property-held roots, selected direct-root ArrayAccess keyed-suffix append, namespace/import class policy, interpreter function/const-import exact lookup, generated-C exact function-import aliases for same-unit user functions and selected runtime builtins, generated-C exact const-import aliases for same-unit scalar user constants and supported builtins, selected literal/relative/dynamic class constant reads, and value-returning class metadata consumers, but broad assignment/RMW/writeback, references/COW, reference-returning nested ArrayAccess breadth, unknown dynamic/static-property shapes, dynamic receiver `::class`, broader static-property reference/array-offset and dynamic/magic property breadth, cleanup/unwind/finally/destructors, exact diagnostics, broader const discovery/lookup, broader function-import coverage, and backend parity remain open. |
| Strings and byte semantics | **60%** | `[############--------]` | Byte-backed values and selected byte-preserving string-array slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **85%** | `[#################---]` | Selected lvalue/reference-source extraction, ReferenceSlot owner facts, object-property owner/fact/commit prerequisites, selected non-local object-property assignment commits, reference-cell predicates, membership helpers, RMW array-lvalue owner/writeback, selected direct/generated-object ArrayAccess RMW/`??=` paths, selected direct-root ArrayAccess keyed-suffix append, selected property-held ArrayAccess read/write/RMW/`??=`/unset/append/increment owners, and selected nested ArrayAccess assignment, compound-assignment, pre/post increment/decrement, append, null-coalescing assignment, unset, and keyed-append-suffix owner stacks are integrated. Reference-returning `offsetGet()`, arbitrary alias roots, foreach breadth, broader writeback, and full COW remain incomplete. |
| Symbols, globals, request state | **70%** | `[##############------]` | Selected globals, root-symbol consumers, active symbol-table consumers, request-key blockers, append-shaped symbol reference-source materialization, direct generated-C request-state frame handoff, and dynamic user-function handoff proof exist. `$GLOBALS` self-cells, closure request-state handoff, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **90%** | `[##################--]` | Runtime callable table/value dispatch, selected runtime builtin source-call signatures/blockers, call arguments/frame/result ABI, source-order/parameter-order named call-argument normalization, conditional handoff, generated-C direct/dynamic callable consumers, declared-method registration/wrapper frames, callable return facts, by-reference argument transport, descriptor closures, descriptor-closure invocation through shared argument/result carriers, closure returns, request-state frame handoff, access-context lookup ABI, lookup-plus-invoke exactly-once argument ownership helpers, source-call result carrier selectors, selected production source-call carrier emission, direct generated user-function lookup-plus-invoke production, direct user-function reference-return frames, exact generated-C `use function` aliases for same-unit user functions and selected runtime callable builtins, explicit by-value return terminal handoff, method/static source-call target operands, method/static source-call binding operands, method/static signature fallback selection, selected direct/dynamic/receiver/static/self/parent reference-return source-call alias transfer into by-reference arguments, executable receiver/static/self/parent/object-static/late-static method source-call production for exact, default, variadic, and selected named-argument frame-compatible arities where class context or receiver/called scope is known, selected generated-C dynamic receiver-method source-call production for known string and runtime-produced method names including declared receiver `__call` fallback, selected generated-C static method `__callStatic` fallback including named magic `$args` keys, explicit generated-C constructor value-return diagnostics, and interpreter/generated-C exact function/const import islands are integrated. Unknown runtime callables, named dynamic receiver magic fallback, malformed magic signature parity, broader late-static override resolution, broader builtin/native/inherited/trait/interface signature metadata, broader by-reference alias transfer, broader const-import discovery/fallback and function-import discovery/fallback, spread and unsupported named builtin/constructor/fallback breadth, descriptor/method/closure and broader return references, broader constructor allocation/execution, cleanup/unwind, and backend parity remain open. |
| Objects, properties, methods | **85%** | `[#################---]` | Selected object metadata, value-returning class metadata consumers, LLVM/generated-C user-class metadata consumers, generated-C namespace/import class policy, generated-C class alias metadata and canonical class/member lookup, selected declared class constant metadata/lookup for literal and relative receivers, public property reference-source extraction, method-frame `$this` property assignment, selected non-local object-property assignment commits, object-property owner/fact/commit prerequisites, object-property reference-slot mutation, selected property-held ArrayAccess read/write/RMW/`??=`/unset/append/increment owners, selected direct-root ArrayAccess keyed-suffix append, selected nested ArrayAccess assignment, compound-assignment, pre/post increment/decrement, append, null-coalescing assignment, unset, and keyed-append-suffix owner stacks, request-scoped runtime static-property storage plus bulk declared-property metadata/default arrays and generated-C declared literal, object/class-string receiver, and relative `self`/`parent`/`static` static-property read/write/observation/mutation/`??=`/unset producers, generated-C declared typed instance-property allocation/default/write diagnostics, generated-C ArrayAccess consumers for compiler-known generated objects, dynamic generated class-name producers, object-call argument handles, declared-method callable-table publication, bounded executable receiver/static/self/parent/object-static/dynamic/late-static method production through access-context source-call carriers, selected declared receiver `__call` and static `__callStatic` fallback dispatch including named magic static `$args`, selected constructor bodies with bare early returns and explicit value-return diagnostics, allocatable class metadata, user-class metadata registry consumers, and access-context preflights exist. Reference-returning nested ArrayAccess breadth, named dynamic receiver magic fallback/malformed-magic/mixed-runtime-dynamic-call/clone/static-property breadth, full late-static override/interface/trait binding, broader dynamic/static-property/class-constant breadth, broader class-alias/autoload parity, broader visibility parity, broader static-property reference/array-offset and dynamic/magic property breadth, destructors, interfaces/traits execution, references/COW, broader constructor allocation/execution, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **70%** | `[##############------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostic blockers, owned diagnostic-result list contracts, consumer contracts, backend family consumers, deferred-cleanup blockers, control-transfer cleanup result consumers, terminal cleanup transfer ABI, terminal-kind ABI, explicit by-value return terminal handoff, bounded function/method return-through-finally cleanup operands, cleanup-frame producers/source metadata/report bridges, cleanup-frame stack aggregation, cleanup-frame enqueue validation, try-body call-boundary preflight, report sinks, continuation helpers, discarded statement-expression operands, and echo/print output operands exist. Broad unwind/finally/destructor/shutdown execution, cleanup result production from arbitrary control flow, executable reference binding, remaining semantic diagnostic-result producer migration, and source-ordered diagnostics remain open. |
| Broad integrated verification | **75%** | `[###############-----]` | Focused gates around recent source work are strong, with several primary integration gates now covering linked generated-C class/method/constructor programs, LLVM class metadata routing, terminal-kind ABI behavior, and owner-boundary regressions. Broad verification is still constrained by lane extraction cost, stale candidate expectations, heavy formatter/log pressure, and backend parity gaps. |

## Recently Accounted Source Work

| Commit | Capability | Proof shape |
| --- | --- | --- |
| `b15a882c` | Generated C now validates class metadata with a shared trait effective-method composition helper also used by interpreter metadata/reflection. The helper covers trait lookup, nested trait uses, aliases, visibility adaptations, `insteadof` exclusions, direct class overrides, conflict/recursion diagnostics, and case-insensitive keys while keeping trait method execution, trait properties/constants, destructor execution, interface dispatch, and autoload blocked. | Primary integration gates passed with `SUMMARY passes=19 failures=0`, covering fmt, diff check, trait semantics unit proof, native function-call-boundary trait metadata proof, `cargo check -p phpc`, dynamic class constants, static-property ArrayAccess references, class constants, class aliases, magic dynamic/static, malformed magic, static-property offset references, reference-returning ArrayAccess, constructors, descriptor closures, source-call references, exact imports, and `named_` with the clean-baseline stale parse-phase test skipped and recorded. Gate log: `state/workers/logs/phpc-primary-trait-effective-method-metadata-r2-integration-20260527.gates.log` sha256 `8751bb08d2971e6baa60f8117953f38fb832b5520b956e2aa01a7d75959500f8`. |
| `ecef5dc3` | Generated C now routes selected static-property `ArrayAccess` by-reference sources through real static-property storage identity and the shared reference-returning `offsetGet()` owner boundary. Literal class, object receiver, class-string receiver, `self`, `parent`, and method-frame late-`static` static-property roots can feed by-reference calls and direct alias assignment without fake by-value reference cloning. Multi-hop static-property ArrayAccess reference paths, computed/static/dynamic reference-COW shapes, and new owner-stack/reference ABIs remain blocked. | Primary integration gates passed with `SUMMARY passes=23 failures=0`, covering fmt, diff check, `cargo check -p phpc`, unit owner-selection proof, generated-C and linked executable static-property ArrayAccess reference proofs, unsupported static-property dynamic-shape blockers, static-property reference/offset-reference/offset-mutation regressions, reference-returning ArrayAccess, nested/property-held ArrayAccess, source-call references, descriptor closures, constructors, named/magic dynamic/static calls, malformed magic, dynamic class constants, exact imports, class constants, and class aliases. Gate log: `state/workers/logs/phpc-primary-static-property-arrayaccess-reference-r1-integration-20260527.gates.log` sha256 `2e6f8470963fa3acd4850fa746e5e7c9c9ed8badee9d602d3a5af27e67d1cd15`. |
| `092288d3` | Generated C now routes selected dynamic class-constant receivers through shared class/constant metadata. Declared class-string and object receivers for `$receiver::CONST` normalize through the runtime class-constant scope helper, honor alias canonicalization, inheritance, visibility, missing-class/missing-constant diagnostics, and explicit cleanup, while dynamic receiver `::class` remains blocked at the no-autoload ABI boundary. | Primary integration gates passed with `SUMMARY passes=19 failures=0`, covering fmt, diff check, `cargo check -p phpc`, runtime class-constant metadata, generated-C and linked executable dynamic class-constant proofs, dynamic `::class` blocker proof, class constants, class aliases, exact imports, named/magic dynamic/static calls, malformed magic, static-property reference/offset-reference, reference-returning ArrayAccess, constructors, descriptor closures, and source-call references. Gate log: `state/workers/logs/phpc-primary-dynamic-class-constant-r1-integration-20260527.gates.log` sha256 `c8160ce3a108a02118b6d7731d96095a9e579108906148984a12269041bf6f00`. |
| `d7b5bf98` | Generated C now routes selected constructor reference results through the shared constructor allocation/invoke result carrier and call-argument ownership boundary. Supported named declared constructors consumed by by-reference source-call arguments move the allocated receiver into a real runtime reference cell for direct function, dynamic function, and constructor consumers, preserving alias/write-through behavior without cloning fake references. Dynamic constructor names, classes without constructors but with arguments, constructor named arguments, spread/unpack, destructor-observable cleanup ordering, broader by-reference constructor alias transfer, traits/interfaces, autoload breadth, broad references/COW, exact PHP diagnostics, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=24 failures=0`, covering fmt, diff check, `cargo check -p phpc`, invoke/source-call carrier units, runtime constructor reference ownership, generated-C and linked executable constructor reference proofs, constructor, source-call reference, descriptor closure, named, magic dynamic/static, static-property offset mutation, static-property storage/reference, runtime static-property, nested/property ArrayAccess, malformed magic, exact imports, class constants, and class aliases. Gate log: `state/workers/logs/phpc-primary-constructor-reference-result-r7-integration-20260527.gates.log` sha256 `f46ca8d8676735da381931d87c8c3fd79c48b318a3472bb09ede049afb74dde4`. |
| `9d2b3526` | Generated C now routes selected static-property array-offset mutation through the shared static-property lvalue/storage boundary and ArrayAccess owner-stack paths. Literal class, object/class-string, `self`, `parent`, and declared method-frame `static` receivers can perform selected offset assignment, append, compound/RMW updates, `??=`, and `unset()` while preserving expression-result ownership, typed/visibility diagnostics, and static storage identity. Computed names, top-level `static::$prop`, static-property array-offset references, arbitrary alias roots, magic/static overloading, broad references/COW, cleanup/unwind breadth, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=22 failures=0`, covering fmt, diff check, `cargo check -p phpc`, nested ArrayAccess owner-stack and direct-root append guards, runtime static-property tests, generated-C linked static-property offset mutation proofs, static-property reference/storage regressions, named receiver magic regressions, nested/property ArrayAccess, descriptor closures, magic dynamic/static filters, malformed magic signatures, source-call references, exact imports, class constants, and class aliases. Gate log: `state/workers/logs/phpc-primary-static-property-offset-mutation-r3-integration-20260527.gates.log` sha256 `41488602ec21cdf8af469c72d323980d1f3c1a21886f93cc61c385436936c00a`. |
| `f7facf37` | Generated C now preserves source-order named argument keys for selected receiver `__call` fallback calls through shared `NativeCallArgumentsHandle` source-name metadata and runtime magic `$args` packing. Direct receiver calls with missing or inaccessible instance methods and dynamic receiver calls with statically known method names carry named `$args` keys when every known possibility resolves to public non-static receiver magic. Declared receiver hits keep declared parameter binding first, mixed declared-hit/magic-fallback facts remain blocked, and malformed magic signatures still reject before derived magic argument packing. Unknown runtime dynamic receiver method names, constructor named arguments, spread/unpack, unknown dynamic callables, traits/interfaces, aliases/autoload, callable-object magic shapes, full `$args` reference/COW parity, static-property array-offset references, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=27 failures=0`, covering fmt, diff check, call-argument metadata, runtime named magic `$args`, generated-C and linked executable named receiver magic proofs, unknown named-dynamic blockers, named static magic regression, malformed magic metadata, magic dynamic/static filters, named arguments, descriptor closures, static-property storage/reference behavior, class constants, constructors, nested ArrayAccess, source-call references, exact imports, namespace imports, class aliases, and `cargo check -p phpc`. Gate log: `state/workers/logs/phpc-primary-named-dynamic-magic-r4-integration-20260527.gates.log` sha256 `491e631a289ca1c448f84517aed3069c053906f007212b649a8c08e590a0fac3`. |
| `3149d1da` | Generated C now routes selected class constant reads through shared class/constant metadata instead of receiver-specific generated ladders. Literal `Class::CONST`, relative `self::CONST`/`parent::CONST`/method-frame `static::CONST`, alias canonicalization, inherited constants, visibility checks, and owned-result diagnostics share one runtime table. Relative `self::class`, `parent::class`, and `static::class` also use the declared/called class metadata context, while literal `ClassName::class` remains no-lookup source-string lowering. Dynamic class-name constant receivers, unsupported initializer expressions, traits/interfaces, include/autoload discovery, fuller diagnostics, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=19 failures=0`, covering fmt, diff check, `cargo check -p phpc`, runtime class-constant ABI proof, generated-C and linked executable class-constant proofs, class aliases, exact imports, static property, descriptor closure, magic static, named arguments, constructors, nested ArrayAccess, and source-call reference filters. Gate log: `state/workers/logs/phpc-primary-class-constants-r2-integration-20260527.gates.log` sha256 `181f391429b3972d20c73c11bf94b889b8186afa11776490b1fd3761b588c656`. |
| `ae7b4535` | Generated C now preserves source-order named argument keys for selected static `__callStatic` fallback calls through shared `NativeCallArgumentsHandle` source-name metadata and runtime magic `$args` packing. Missing or inaccessible static method calls with public static `__callStatic` carry named `$args` keys for literal class, object-static receiver, `self::`, and method-frame `static::` calls, while normal declared static method hits keep declared parameter binding ahead of magic fallback. Named dynamic receiver `__call`, spread/unpack, constructors, unknown dynamic callables, malformed magic signatures, traits/interfaces, aliases/autoload, callable-object magic shapes, full `$args` reference/COW parity, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=19 failures=0`, covering fmt, diff checks, `cargo check -p phpc`, magic argument normalizer unit proof, generated-C and linked executable named magic static proofs, named dynamic receiver blocker, magic static, named, descriptor closure, static-property, static-property mutation, nested and property-held ArrayAccess, constructor, source-call reference, exact import, class alias, and namespace import filters. Gate log: `state/workers/logs/phpc-primary-magic-static-namedargs-r2-integration-20260527.gates.log` sha256 `27be264a2d3b11163e84ae6d105d492b1de79dbce85c2b9b1e5d2518e15fb84a`. |
| `14edee16` | Generated C now routes selected static-property `unset()` through the shared static-property lvalue/storage boundary for literal class, object/class-string, `self`, `parent`, and declared method-frame `static` receivers. Runtime storage supports class and relative unset APIs, untyped storage resets to initialized `null`, typed storage returns to uninitialized, and read/`isset()`/`empty()` aftermath plus visibility/scope/missing-property diagnostics reuse the same storage boundary. Computed names, top-level `static::$prop`, references, static-property array-offset mutation, magic/static overloading, broader references/COW, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=15 failures=0`, covering fmt, diff checks, `cargo check -p phpc`, runtime static-property tests, static-property and static-property mutation native-link filters, object static-property receivers, nested and property-held ArrayAccess, constructors, descriptor closures, magic static, source-call references, exact imports, and class aliases. Gate log: `state/workers/logs/phpc-primary-static-property-unset-r1-integration-20260527.gates.log` sha256 `d341a10f1c480b925d899ca9a2b4078add06ca215ddd7dff45c6ada3f8f921ea`. |
| `2e2625eb` | Generated C now routes selected descriptor-closure reference returns through a real runtime reference carrier for proven descriptor-backed closures. By-reference closure returns are admitted only when the return source is a direct by-reference parameter or capture, and the resulting reference can feed both by-reference argument transfer and direct reference assignment materialization with alias/write-through behavior. Unknown/mixed callables, callable arrays, invokable objects, method descriptors, non-descriptor closures, unsupported by-ref return sources, identity loss through symbol-table-only closure storage, broader reference/COW ownership, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=16 failures=0`, covering fmt, diff checks, `cargo check -p phpc`, descriptor-closure reference carrier unit proof, generated-C and linked executable reference-return consumer proofs, descriptor closure and source-call reference filters, magic static, named, static property, nested and property-held ArrayAccess, constructor, exact import, and class-alias filters. Gate log: `state/workers/logs/phpc-primary-descriptor-closure-reference-return-r2-integration-20260527.gates.log` sha256 `a31b72882552c114b2db8f69b5a12bc95c82d00e0b96ea7ffd11aad647695cb1`. |
| `16e7dec5` | Generated C now routes selected static-property `??=` through the shared static-property lvalue target for literal class, object/class-string, `self`, `parent`, and method-frame `static` receivers. Supported paths use read-for-`isset` probes, skip RHS evaluation for present non-null storage, lazily write missing/null/uninitialized storage through typed/visibility static-property APIs, and preserve expression-result ownership. Computed names, top-level `static::$prop`, references, array-offset mutation, magic/static overloading, broader references/COW, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=15 failures=0`, covering fmt, diff checks, `cargo check -p phpc`, runtime static-property tests, static-property and static-property mutation native-link filters, object static-property receivers, descriptor closures, magic static, nested and property-held ArrayAccess, constructors, source-call references, exact imports, and class aliases. Gate log: `state/workers/logs/phpc-primary-static-property-nullcoalesce-r2-integration-20260527.gates.log` sha256 `ae3fba1e97863bcf7068e4298d0103d9fe5bafb35a6a9c6aa6205470f201a6f8`. |
| `ba46f5e2` | Generated C now routes selected nested ArrayAccess `unset()` and append-with-keyed-suffix assignments through the generalized owner stack for direct-variable and visible property-held roots. Supported unset paths perform by-value `offsetGet()` descent, leaf `offsetUnset()`, reverse parent writeback, and root commit. Supported keyed append paths materialize suffix keys, wrap the RHS through the shared appended-slot value boundary, append at the leaf with `offsetSet(null, value)`, reverse-write parents, commit the root owner, and preserve assignment expression results separately from the appended value. Root keyed-suffix append without owner-stack descent, reference-returning `offsetGet()`, arbitrary alias/static roots, broader references/COW, cleanup/unwind breadth, spread/unpack, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=18 failures=0`, covering fmt, diff checks, nested owner-stack units, exact generated-C and linked executable proofs for nested unset and keyed-append suffix, nested ArrayAccess regressions, property-held ArrayAccess owners, descriptor closures, magic static calls, static-property and static-property mutation regressions, object static-property receivers, source-call references, exact imports, class aliases, and `cargo check -q -p phpc`. |
| `1202542e` | Generated C now routes selected static-property plain assignment, compound assignment, and pre/post increment/decrement through a shared lvalue target for literal class, object/class-string, `self`, `parent`, and method-frame `static` receivers. The path reuses runtime static-property storage read/write APIs, derives object/class-string scope through the shared receiver-scope ABI, preserves compound and pre/post expression-result ownership, and lets parser object-static-property lvalues reach codegen for prefix/compound mutations. Computed names, top-level `static::$prop`, static-property references, array-offset mutation, magic/static overloading, broader references/COW, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=14 failures=0`, covering fmt, diff checks, runtime static-property tests, static-property native-link proofs, static-property mutation executable proof, object static-property receivers, descriptor closures, magic static calls, source-call references, nested ArrayAccess, property-held ArrayAccess, class aliases, and `cargo check -p php_runtime -p phpc`. |
| `8a8d85ed` | Generated C now routes selected nested ArrayAccess null-coalescing assignments through the generalized owner stack for direct-variable and property-held roots. Supported paths probe the leaf with `offsetExists()`, read present leaves, lazily evaluate the RHS only for missing/null leaves, write mutated leaves with `offsetSet()`, reverse-write parents only on mutation branches, commit the root owner, and preserve the `??=` expression result. Nested unset, append-with-keyed-suffix, reference-returning `offsetGet()`, arbitrary alias/static roots, broader references/COW, cleanup/unwind breadth, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=14 failures=0`, covering fmt, diff checks, nested owner-stack units, exact generated-C and linked executable nested `??=` proofs, nested ArrayAccess regressions, property-held ArrayAccess owners, descriptor closures, magic static calls, static properties, source-call references, class aliases, and `cargo check -p phpc`. |
| `5feb8cfe` | Generated C routes selected nested ArrayAccess append assignments through the generalized owner stack for direct-variable and property-held roots. Supported paths materialize the root owner, descend through by-value `offsetGet()` intermediates, append at the leaf with `offsetSet(null, value)`, reverse-write parents, commit the original owner, and preserve assignment expression result ownership. Append-with-keyed-suffix, nested unset, reference-returning `offsetGet()`, arbitrary alias/static roots, broader references/COW, cleanup/unwind breadth, spread/unpack, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=14 failures=0`, covering fmt, diff checks, nested owner-stack units, exact generated-C and linked executable nested append proofs, nested ArrayAccess regressions, property-held ArrayAccess owners, descriptor closures, magic static calls, static properties, source-call references, class aliases, and `cargo check -p phpc`. |
| `f65be9b1` | Generated C now routes selected static method source calls through runtime `__callStatic` fallback. Declared static method hits still win, missing or inaccessible static methods fall back to public static `__callStatic($name, $args)`, non-static methods called statically stay hard failures, and literal class, object-static receiver, `self::`, `parent::`, and bounded declared-frame `static::` calls share the runtime lookup-plus-invoke boundary. Malformed magic signatures, traits/interfaces/effective method tables, aliases/autoload, callable-object static magic shapes, `$args` reference/COW parity, and LLVM/direct assembly parity remain blocked. | Primary integration gates passed with `SUMMARY passes=25 failures=0`, covering fmt, diff check, runtime `__callStatic` lookup-plus-invoke, runtime method lookup helpers, native source-call signature fallback contracts, generated-C source and linked executable magic-static proofs, named static magic fallback blocker, descriptor closures, object static-property receivers, named arguments, magic dynamic/static calls, object-static/class-context/self-parent/late-static/static-property paths, typed properties, exact imports, source-call references, nested ArrayAccess, class aliases, and `cargo check -p php_runtime -p phpc`. |
| `c2ffab4b` | Generated C now initializes declared static-property storage through the bulk metadata/default ABI, consuming property name, visibility, type, default, and static-flag arrays from class metadata. Static defaults are registered only for static properties while non-static metadata remains visible to static lookup shadowing, and typed static-property defaults/writes now use the runtime diagnostic path. Dynamic class/property names, static-property references, unset/isset/empty, compound mutation, increment/decrement, `??=`, append/nested static-property offset mutation, magic/static overloading, traits/interfaces, full type parity, and LLVM/backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=16 failures=0`, covering fmt, runtime static-property tests, generated-C static-property native-link proofs, object static-property receivers, descriptor closure calls, named arguments, magic/runtime dynamic method calls, exact imports, late static, typed declared instance properties, source-call references, class aliases, nested ArrayAccess, diff check, and `cargo check -p php_runtime -p phpc`. |
| `55aef1ee` | Generated C now routes proven descriptor-backed closure calls through shared `NativeCallArgumentsHandle` production and descriptor closure invoke helpers for result, value, reference, and discard consumers. The compiler selects this path from callable identity facts, not from a source spelling, fixture, arity, local variable name, or generated-C substring. Unknown/mixed callables, callable arrays, invokable objects, non-descriptor closure handoff, descriptor/method/closure reference-return breadth, spread/unpack, broader request-state handoff, and LLVM/backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=28 failures=0`, covering runtime closure invoke helpers, generated-C source proof, linked descriptor-closure executable proof, named arguments, exact imports, magic/runtime/dynamic method source calls, late/static properties, object static-property receivers, object-static calls, typed properties, try/finally, ArrayAccess reference slots, property-held and nested ArrayAccess, source-call references, class aliases, fmt, diff check, and `cargo check -p php_runtime -p phpc`. |
| `147ad3b5` | Generated C now routes selected object and declared class-string static-property receivers through a shared runtime receiver-scope helper before using request-owned static-property storage for direct reads and plain assignments. Object instance receivers and class-string values share the same runtime scope derivation and static read/write ABI; unsupported receiver scopes still report diagnostics and clean up owned handles. Top-level `static::$prop`, dynamic property names, static-property references, compound mutation/unset/isset/empty, magic/static overloading, traits/interfaces, autoload breadth, LLVM parity, and reference/COW static-property breadth remain blocked. | Primary integration gates passed with `SUMMARY passes=20 failures=0`, covering runtime receiver-scope helpers, generated-C object/class-string static-property source and linked executable proof, named argument, magic receiver, exact import, declared static-property, late-static, runtime dynamic method, typed property, nested/property-held ArrayAccess, reference-source, constructor value-return, non-local property, source-call reference alias, class-alias, fmt, diff checks, and `cargo check -p php_runtime -p phpc`. |
| `28ce0faa` | Generated C now routes selected nested ArrayAccess pre/post increment and decrement through the generalized owner-stack write context for direct-variable and property-held roots, preserving pre/post expression results while doing by-value `offsetGet()` descent, leaf read, native increment/decrement replacement, leaf `offsetSet()`, reverse parent writeback, and root commit. Nested `??=`, append, unset, reference-returning `offsetGet()`, arbitrary alias roots, broad references/COW, cleanup/unwind breadth, spread/unpack, and LLVM/backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=47 failures=0`, covering nested owner-stack units, generated-C source and linked executable proof for direct/property-root nested increment/decrement, nested assignment and compound-assignment regressions, reference-returning and unsupported nested mutation blockers, property-held ArrayAccess owner regressions, named call arguments, magic receiver calls, exact function/const imports, late/static properties, runtime dynamic methods, typed properties, source-call byref, constructor value-return diagnostics, class aliases, fmt, diff checks, and `cargo check -p phpc`. |
| `425f1ef5` | Generated C now lowers selected named call arguments for compiler-known direct user functions and method/static/dynamic source-call carriers through shared source-order/parameter-order normalization. The normalizer preserves source-order evaluation, binds named values to required/optional/default/by-reference/variadic parameters, rejects duplicate/unknown/positional-after-named/unpack shapes at explicit boundaries, and then feeds parameter-order `NativeCallArgumentsHandle` construction. Named builtins, constructors, unsupported dynamic callables, magic fallback named-argument parity, spread/unpack, broader signature metadata, and LLVM/direct-assembly parity remain blocked. | Primary integration gates passed with `SUMMARY passes=38 failures=0`, covering parser/AST named-argument boundaries, shared normalizer unit proof, generated-C source proof for direct user functions and method/static/dynamic source-call carriers, linked named function/method/dynamic-method executable proof, explicit unsupported builtin/dynamic fallback/unpack blockers, magic dynamic method regressions, exact function/const imports, late/static properties, typed properties, nested and property-held ArrayAccess, source-call references, class aliases, runtime magic lookup, fmt, diff check, cargo check, and cached diff check. |
| `eedd0879` | Generated-C dynamic receiver method calls on compiler-known receivers with declared instance `__call($name, $args)` now use the shared runtime lookup-plus-invoke dispatcher. Normal method hits still win, missing or inaccessible methods fall back to public non-static `__call`, and magic arguments pack the original method name plus value snapshots of the original call arguments. `__callStatic`, malformed magic signature parity, traits/interfaces/effective tables, aliases/autoload, callable-object fallbacks, full `$args` reference/COW alias behavior, and LLVM/direct-assembly parity remain blocked. | Primary integration gates passed with `SUMMARY passes=18 failures=0`, covering runtime magic `__call` lookup-plus-invoke, method lookup filtering, generated-C magic dynamic source proof, linked magic dynamic executable proof, dynamic/class-context/object-static/late-static/static-property/typed-property/nested ArrayAccess/exact const/source-call reference/class-alias regressions, fmt, diff check, and cached diff check. |
| `a4d151b8` | Generated C now lowers exact parser-resolved `use const` aliases for same-compilation-unit scalar user constants and supported builtin constants through shared constant metadata/import markers. Exact imported misses reject without namespace/global fallback; ordinary bare constants, arrays/dynamic constant expressions, class constants, include/autoload discovery, dynamic alias roots, function-frame constant lookup, LLVM constant lowering, and broad backend/global constant parity remain blocked. | Primary integration gates passed with `SUMMARY passes=23 failures=0`, covering const-import generated-C source proof, exact-miss and declaration-shape blockers, parser/runtime exact lookup rules, namespace/import focused module, linked exact imported-const executable proof, LLVM constant blockers, function-import regressions, runtime dynamic methods, late-static properties, nested ArrayAccess RMW, typed properties, class aliases, try/finally cleanup, fmt, diff check, and cached diff check. |
| `bf8fd335` | Generated C now routes declared-frame `static::$prop` reads/writes through shared relative static-property runtime storage using the active called-scope receiver, so inherited static-property methods update descendant storage instead of lexical `self`. Top-level `static::$prop`, object static-property receivers, dynamic class/property names, static-property references, compound mutation/unset/isset/empty, magic/static overloading, traits/interfaces, autoload breadth, LLVM parity, and reference/COW static-property breadth remain blocked. | Primary integration gates passed with `SUMMARY passes=15 failures=0`, covering generated-C source proof, linked late-static property executable proof, static-property native-link and runtime filters, runtime dynamic method and late-static method filters, typed declared instance-property and object-static regressions, nested ArrayAccess RMW source and executable regressions, runtime dynamic method linked executable proof, typed-property failure proof, fmt, diff check, and cached diff check. |
| `ba5769e2` | Generated C now routes selected nested ArrayAccess compound assignments through the generalized owner stack for direct-variable and property-held roots, including by-value `offsetGet()` descent, leaf read, native binary replacement, leaf `offsetSet()`, reverse parent writeback, and root commit. Nested `??=`, append, increment, decrement, reference-returning `offsetGet()`, arbitrary alias roots, broader COW/reference forms, cleanup/unwind breadth, and backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=24 failures=0`, covering nested owner-stack unit proof, generated-C direct/property-root RMW source proof, linked nested ArrayAccess RMW executable proof, existing nested assignment source and executable regressions, reference-returning `offsetGet()` and non-assignment mutation blockers, property-held and direct ArrayAccess owner regressions, relative static properties, runtime dynamic methods, late-static calls, typed properties, fmt, diff check, and cached diff check. |
| `cfc7c0ee` | Generated C now routes declared-frame `self::$prop` and `parent::$prop` reads/writes through shared relative static-property runtime storage, using one program-wide request storage handle across top-level and method-frame static-property operations. `static::$prop`, object-static property receivers, dynamic class/property names, static-property references, compound mutation/unset/isset/empty, magic/static overloading, traits/interfaces, autoload breadth, LLVM parity, and reference/COW static-property breadth remain blocked. | Primary integration gates passed with `SUMMARY passes=20 failures=0`, covering generated-C source proof, linked self/parent static-property execution, declared static-property storage, unsupported static-property blockers, self/parent static calls, late-static, dynamic methods, object-static carriers, runtime static-property storage, typed instance properties, constructor value diagnostics, class aliases, try/finally cleanup, property-held and nested ArrayAccess, exact/unsupported imports, fmt, and diff check. |
| `7ae07fd7` | Generated-C receiver dynamic method calls with unknown runtime method-name expressions now route through the shared lookup-plus-invoke source-call carrier when receiver facts are compiler-known and the class has no declared `__call`; runtime lookup normalizes scalar method-name values before access-context lookup. Declared `__call`/`__callStatic`, magic argument packing, traits/interfaces, arbitrary aliases/autoload, callable-object fallbacks, broader byte/encoding diagnostic parity, and LLVM parity remain blocked. | Primary integration gates passed with `SUMMARY passes=20 failures=0`, covering runtime method-name normalization, access-context preflights, signature fallback contracts, generated-C source proof, linked runtime dynamic method executable proof, magic dynamic method blockers, dynamic/class-context/object-static carriers, late-static, property-held ArrayAccess, source-call byref, runtime dynamic builtins, declared static properties, typed instance properties, static-property runtime storage, fmt, and diff check. |
| `441de993` | Generated C now routes bounded declared-frame `static::method(...)` calls through runtime called-scope handles and static source-call carriers, while preserving lexical class context only for protected/private access checks. Descendant-only targets, override-visible late-static breadth, traits, interfaces, magic `__callStatic`, dynamic static properties, `static::class`, `static::$prop`, LLVM parity, and broad inheritance/interface resolution remain blocked. | Primary integration gates passed with `SUMMARY passes=21 failures=0`, covering late-static source and linked executable proof, runtime called-scope dispatch, typed instance-property regressions, try/finally, imports, dynamic/object/static/self-parent/class-context carriers, property-held and nested ArrayAccess, declared static properties, source-call byref, constructor value diagnostics, non-local object-property owners, class aliases, object call dispatch handles, fmt, and diff check. |
| `f1816273` | Generated C now lowers selected declared typed instance properties by emitting per-property type/default metadata, allocating declared objects through a typed-property metadata runtime ABI, initializing defaults, and routing known typed instance-property writes through diagnostic mutation. Typed static properties, dynamic/magic properties, unsupported object type declarations, broad reference/COW property semantics, and LLVM/backend parity remain blocked. | Primary integration gates passed with `SUMMARY passes=24 failures=0`, covering typed instance-property generated source and linked executable success/failure proof, runtime declared-class allocation metadata, unsupported declared-class feature blockers, finally return cleanup, exact imports, late-static/object-static/class-context carriers, static-property storage, non-local property owners, property-held and nested ArrayAccess, source-call byref, constructor value-return diagnostics, class aliases, declared object-property regressions, fmt, and diff check. |
| `2ca9d0e3` | Generated C now routes bounded function and method returns through active `finally` bodies by reporting finalizer output, enqueueing cleanup-surface operands, transferring return terminal kind with a non-empty cleanup list, and then taking the original return value. Top-level try returns, throw/catch, exit/goto unwind, finally-return replacement, destructor/shutdown ordering, return operands with source-call cleanup, and full exception/finally semantics remain blocked. | Primary integration gates passed with `SUMMARY passes=36 failures=0`, covering native diagnostic ABI, source-call carriers, nested ArrayAccess smoke, terminal-kind ABI, try/finally source and linked executable proof, top-level try-return blocker, try-unwind blockers, exact function-import regressions, dynamic method/object-static/class-context carriers, property-held ArrayAccess owners, declared static-property storage, byref/source-call alias transfer, constructor value-return diagnostics, non-local owner commits, class aliases, native function destructor boundary, shutdown functions, fmt, and diff check. |
| `e8268187` | Generated C now lowers selected nested ArrayAccess assignments through a generalized owner stack for direct-variable and property-held roots, including by-value `offsetGet()` descent, leaf `offsetSet()`, reverse parent writeback, and root commit. Reference-returning `offsetGet()`, nested RMW/`??=`/append/increment/decrement, broader COW/reference forms, and backend parity remain blocked. | Nested owner-stack unit proof, generated-C direct/property-root source proof, linked nested ArrayAccess executable proof, reference-returning `offsetGet()` blocker, non-assignment nested mutation blockers, direct nested append guard, property-held ArrayAccess owner regressions, declared static-property regressions, object-static/class-context/source-call byref regressions, constructor value-return and non-local property owner regressions, runtime builtin and class-alias regressions, fmt, diff check. |
| `5bacf1aa` | Generated C now lowers exact parser-resolved `use function` aliases for same-compilation-unit user functions and selected signature-backed runtime callable builtins through shared source-call arguments and lookup/invoke carriers, preserves exact missing imported runtime names, and keeps unsupported imported runtime builtins such as `count()` behind lookup-before-argument side-effect guards. Dynamic alias roots, include/require discovery, autoload, broad runtime builtin imports, native const-import lowering, and backend parity remain blocked. | Function-import namespace proof, generated-C imported runtime-builtin source proof, linked imported user-function executable proof, linked imported runtime-builtin executable proof, unsupported imported builtin side-effect guard, dynamic builtin regressions, dynamic method regressions, object-static regressions, property-held ArrayAccess owner regressions, declared static-property regressions, terminal-return regression, runtime callable builtin regressions, fmt, diff check. |
| `5a4f10a5` | Generated C now routes compiler-known dynamic receiver-method calls with known string method names through shared source-call carriers, including class-context access for declared method frames and runtime rejection of static descriptors used through object dispatch, while keeping declared `__call`, broad magic dispatch, traits/interfaces, unknown scalar method names, late-static behavior, and unsupported receiver shapes blocked. | Runtime access-context/static-through-object diagnostic proof, generated-C dynamic receiver source proof, linked dynamic receiver executable proofs, class-context dynamic private-call proof, magic boundary proof, object-static/class-context/self-parent/static-method carrier regressions, property-held ArrayAccess append/increment regressions, declared static-property storage/production regressions, source-call byref and direct reference-return regressions, runtime builtin regressions, fmt, diff check. |
| `ad74d35b` | Generated C now lowers selected property-held ArrayAccess append and pre/post increment/decrement through existing object-property reference owners for literal and single-known dynamic properties, commits mutated holders through reference writeback, preserves expression-result ownership and conversion diagnostics, and keeps nested offsets, unknown dynamic properties, reference-returning `offsetGet()`, static-property owners, broad COW/reference semantics, and backend parity blocked. | Property-held ArrayAccess source and linked executable append/increment proofs, conversion-diagnostic executable proof, unsupported-owner blocker proof, adjacent ArrayAccess RMW/`??=`/unset owner regressions, non-local object-property owner regressions, static-property production/storage regressions, object-static source-call regressions, source-call byref and self/parent regressions, class-context method-call regressions, fmt, diff check. |
| `99d23aa4` | Generated C now carries declared-class caller scope into receiver-method and named static source-call wrappers emitted from declared method frames, allowing supported private `$this->method()` and protected `Class::method()` calls through runtime class-context access checks while keeping external receiver/static callers, dynamic fallback ladders, callable-object/array shortcuts, magic calls, late-static binding, traits/interfaces, and direct private/protected frame dispatch blocked. | Runtime access-context lookup diagnostics, generated-C source proof for class-context receiver/static carriers, linked executable private/protected class-context program, object-static source-call regressions, method/static/default/variadic/self/parent carrier regressions, runtime builtin signatures, static-property storage/production regressions, source-call byref regressions, constructor value-return diagnostics, non-local object-property regressions, fmt, diff check. |
| `3eb101aa` | Generated C now routes object static-receiver calls such as `$obj::method()` and known class-string receiver forms through shared static source-call carriers, deriving an owned runtime class scope from object/class-string receivers, building `NativeCallArgumentsHandle` once, supporting exact/default/variadic frame-compatible declared public static methods, and keeping non-static methods, `static::`, magic, traits/interfaces, full late-static binding, and unsupported receivers blocked. | Object-static signature contract proofs, runtime receiver-scope helper proof, generated-C source proofs for object-static exact/default/variadic carriers, linked object-static executable proofs, adjacent receiver/static/default/variadic/self/parent source-call regressions, runtime builtin signature/dynamic builtin regressions, static-property production/storage regressions, source-call byref regressions, constructor value-return diagnostics, non-local object-property and class-alias regressions, fmt, diff check. |
| `22b3074f` | Generated C now lowers declared literal `Class::$prop` reads and writes for compiler-known classes through request-owned runtime static-property storage, registering defaults/reset handles, preserving assignment expression results, routing type/visibility/canonical class lookup through the runtime ABI, and keeping dynamic class/property, `self::`/`parent::`/`static::`, object-static, magic, compound/unset/reference, LLVM, and assembly static-member forms blocked. | Runtime static-property suite, generated-C static-property source proof, linked executable static-property program, unsupported dynamic-shape blocker proof, self/parent static regressions, builtin-signature/runtime dynamic builtin regressions, class-alias regressions, constructor value-return diagnostics, non-local object-property owner regressions, fmt, diff check. |
| `b4503c2e` | Generated-C `self::` and `parent::` static source calls now carry active declared class and parent-class context, publish protected/private static method metadata for runtime access checks, and invoke through class-context static source-call carriers while keeping direct static ladders public-only and `static::`/late-static binding blocked. | Self/parent source-call unit proof, generated-C source proof, linked self/parent executable proof, builtin-signature regressions, source-call byref regressions, constructor value-return diagnostics, non-local object-property owner regressions, static-property runtime regressions, fmt, diff check. |
| `7e9a237b` | Selected runtime callable builtins now expose arity, by-reference, return, and source-call support metadata for dynamic source-call argument binding; unsupported runtime builtins such as `count` remain blocked at lookup time before argument construction, without fake `count()` semantics. | Builtin signature metadata unit proof, runtime callable signature/dispatch/boundary proof, generated-C known/runtime dynamic builtin source proof, linked runtime dynamic builtin executable proof, unsupported builtin side-effect guard, source-call byref regressions, terminal-return, class-alias, non-local object-property, and static-property regressions, fmt, diff check. |
| `defad66a` | Generated-C by-reference arguments can now materialize proven reference-return source-call results from direct user functions, dynamic callable identities, receiver methods, and named static methods through shared source-call carriers while keeping by-value produced calls on the alias-transfer blocker. | Source-call byref generated-source proof, linked byref alias executable and cleanup-failure proof, by-value produced-call blocker proof, direct reference-return regressions, terminal-return regressions, class-alias regressions, constructor value-return regressions, non-local object-property and static-property runtime regressions, fmt, diff check. |
| `bae8d3fe` | Generated C can register class aliases for already-declared metadata targets, resolve canonical class/member metadata through aliases, preserve alias conflicts and missing-source autoload boundaries, and keep LLVM direct `class_alias()` lowering rejected. | Runtime alias metadata tests, generated-C source and linked executable class-alias metadata proof, missing autoload boundary proof, namespace alias policy regressions, LLVM rejection/interpreter regressions, terminal-return regressions, static-property runtime regression, constructor value-return regressions, non-local object-property and property-held ArrayAccess owner regressions, fmt, diff check. |
| `01da56ce` | Generated-C explicit by-value function and method returns now produce terminal-surface `NativeDiagnosticResult` values, transfer return terminals through cleanup handling, and extract owned return values back into existing native-value and closure-value frame contracts while preserving reference-return and constructor value-return paths. | Terminal-return generated-source proof, runtime return-handoff ABI proof, linked executable function/method return proof, constructor value-return regressions, direct reference-return regressions, property-held ArrayAccess owner/unset regressions, non-local object-property owner regressions, static-property runtime storage regression, fmt, diff check. |
| `d6798eb8` | Runtime declared static properties now have request-scoped default initialization and reset, per-class/per-property visibility shadowing, assignment type checks, reference identity preservation, and late-static receiver resolution through storage helpers. Generated-C/LLVM static-property production lowering remains blocked. | Runtime static-property storage tests, existing non-local assignment/unset owner boundary regressions, property-held ArrayAccess owner/unset executable regressions, constructor value-return diagnostic regression, fmt, diff check. |
| `bc04035b` | Generated-C constructor `return <expr>;` no longer pre-rejects during constructor validation; generated constructor frames return status `2`, free the returned value and receiver at the call site, and report the explicit `constructor value returns are not implemented` diagnostic through the existing failure cleanup path while preserving public/non-static and reference/typed/global-import blockers. | Constructor value-return generated-source proof, linked executable diagnostic proof, constructor dispatch/blocker regressions, direct reference-return regression, property-held ArrayAccess owner/unset regressions, non-local object-property owner regressions, fmt, diff check. |
| `cabdcde6` | Generated-C non-local object-property assignments now materialize literal and single-known dynamic public-property owners through reference slots, clone the assigned value for expression result semantics, commit replacements through reference writeback, update owner facts, and keep unknown dynamic, nested, and static property assignment shapes blocked. | Generated-C source proof for owner/reference commit shape, linked executable proof across literal, replacement, and dynamic property holders, reference-backed dynamic property/value proof, unsupported-shape rejection proof, property-held ArrayAccess owner/unset regressions, fmt, diff check. |
| `96ad8464` | Generated-C property-held ArrayAccess `unset()` now materializes literal and single-known dynamic object-property holders through public-property reference owners, invokes the ArrayAccess write/unset ABI, commits the mutated holder through reference writeback, and keeps nested/non-direct/unknown dynamic owners blocked. | Generated-C source proof for write/unset ABI and owner commit shape, linked executable proof across literal and dynamic property holders, unsupported owner-shape rejection proof, fmt, diff check. |
| `b3e2a724` | Generated-C direct user-function reference-return frames now accept by-reference parameter return sources, use native reference-returning frame signatures, preserve callable-wrapper reference ownership, route reference consumers through source-call reference carriers, and keep by-value return sources rejected. | Focused reference-return frame contract tests, alias-transfer result-vector source proof, by-value source rejection proof, source-call carrier selector regressions, direct user-function frame source/link regressions, fmt, diff check. |
| `e81aa43e` | Parser/runtime `use const` imports carry import-kind metadata, resolve arbitrary aliases/default aliases exactly before namespace/global fallback, preserve non-imported namespace-then-global fallback, reject aliases that conflict with existing imports or same-namespace constant declarations in either order, and keep generated C/LLVM at explicit production rejection boundaries. | Namespace-resolution proof across const aliases, default aliases, exact-missing no-fallback, non-import fallback, declaration/import conflict guards, class/function import regressions, generated-C and LLVM rejection, unsupported const-use CLI snapshot, focused global-constant rejection, fmt, diff check. |
| `6757bb43` | Generated-C property-held ArrayAccess owners materialize literal and single-known dynamic object-property values through public-property reference owners, use constructor `$this` property fact summaries for direct `new` assignments, commit writes/RMW/`??=` through reference writeback, and keep unknown/nested/append/increment owner shapes blocked. | Codegen owner/fact unit proof, generated-C source proof for property-held read/write/RMW/`??=`, linked executable proof across literal and dynamic property holders, unsupported-owner regression, existing ArrayAccess RMW/`??=` regressions, `$this` assignment and constructor regressions, method/static default-variadic regression, fmt, diff check. |
| `8886d9e5` | Generated-C receiver-method and named static-method source-call production now accepts frame-compatible default and variadic arities by synthesizing omitted defaults and variadic packs into the shared `NativeCallArgumentsHandle` before invoking existing source-call carriers. | Contract unit proof for exact/forward versus default/variadic frame plans, generated-C source proof for receiver/static carrier paths and variadic packing, linked executable proof across receiver/static default and variadic calls, existing exact-arity method/static and constructor regressions, fmt, diff check. |
| `c3968d0a` | Parser/runtime `use function` imports now carry import-kind metadata, resolve arbitrary aliases/default aliases exactly before namespace fallback, reject alias conflicts with same-namespace function declarations/imports, and keep generated-C at an explicit production rejection boundary. | Runtime namespace-resolution proof for aliases, default aliases, non-imported fallback, exact-missing no-global-fallback, alias-conflict guards, generated-C rejection, class-import regressions, linked namespace alias/class policy regressions, constructor regression, fmt, diff check. |
| `bd0eafd0` | Generated-C declared constructor validation allows bare `return;` early exits while keeping `return <value>` blocked, using the existing method-frame constructor dispatch and declared allocation paths. | Linked constructor executable covering default args, required args, `$this` assignments, guarded bare returns, dynamic constructor allocation/dispatch regressions, unsupported constructor value-return guard, method-frame `$this` assignment regressions, object-model dynamic class-name proof, fmt, diff check. |
| `b3f16040` | Structured object-property owner/fact/commit prerequisite boundary tracks literal and dynamic property writes without stringified paths, materializes dormant object-property owners through public-property reference slots, commits replacements through reference writeback, and preserves ArrayAccess owner cleanup ownership. | Codegen unit proof for owner materialization, fact invalidation, and commit cleanup; focused generated-C ArrayAccess owner-boundary/rejection/`??=` regressions; fmt, diff check. |
| `d96cc2bb` | LLVM user-class metadata parity declares top-level user classes, parents, methods, and properties through the shared runtime registry and routes LLVM `class_exists()`, `method_exists()`, and `property_exists()` through the metadata-exists ABI. | LLVM IR declaration/call proof, LLVM assembly acceptance, object-model metadata ABI tests, class-boundary expectation updates, generated-C metadata regressions, fmt, diff check. |
| `cb8457f1` | Generated-C accepts parser-resolved namespace/import class policy, named class constants, leading-backslash/case-normalized dynamic class matching, and `class_exists()` autoload policy without pretending autoload succeeded. | Runtime class-name/autoload-policy tests, generated-C source proof, linked namespace/import executable, exact namespaced `class_exists` precedence, missing default-autoload terminal boundary, metadata registry regression, fmt, diff check. |
| `ac5004a3` | Runtime/compiler diagnostic results can carry return/throw/exit terminal kinds through cleanup transfer, report sinks, terminal-list inspection, and backend declaration surfaces without enabling production return/throw/exit lowering. | Runtime ABI tests across all terminal kinds, terminal cleanup masking, invalid/missing ownership blockers, report sinks, backend surface tests, existing terminal transfer/report regressions, fmt, diff check. |
| `04b18506` | Generated non-static method and constructor frames can assign literal or dynamic `$this` properties through the shared object-property mutation ABI while preserving receiver/replacement ownership, diagnostics, cleanup, and assignment-expression value semantics. | Generated-C source proof updated for source-call carrier callers, linked `$this` assignment success/failure programs, declared method and constructor executable regressions, object-property mutation ABI regression, fmt, diff check. |
| `c8aeb771` | Bounded generated-C receiver-method and named static-method source-call production runs through shared target operands, binding operands, signature fallback contracts, source-call carriers, and runtime diagnostics when exact frame-compatible signatures are known. | Generated-C carrier-shape proof, linked executable method/static proof, signature fallback carrier regression, native source-call carrier regressions, declared static dispatch guard, fmt, diff check. |
| `4e5e2709` | Runtime and generated-C value-returning class metadata consumers for `get_parent_class()`, `class_parents()`, `get_declared_classes()`, `get_class_methods()`, and `get_class_vars()` over registered user classes and selected core class metadata. | Runtime ABI test across parent chain, declared classes, method/property visibility, and ownership; generated-C source proof; linked executable proof including user and core metadata; direct-call diagnostics regression; fmt, diff check. |
| `69526631` | Generated-C dynamic source-call reference results can be materialized as by-reference arguments when the callee is proven reference-returning through scoped callable-string metadata or native callable identity summaries, while unsupported produced calls still hit the existing blockers. | Generated-C source proof across direct and dynamic consumers, multiple reference-return methods, multiple symbols, and multiple by-reference positions; linked executable success and cleanup-failure proofs; existing adjacent alias-transfer/runtime ABI regressions; fmt, diff check. |
| `2cd4f628` | Runtime user-class metadata registry plus generated-C declaration and class/member metadata-exists consumers for declared user classes, inherited methods/properties, and runtime value operands. | Runtime registry test across declared class, parent, inherited method/property, and diagnostics; generated-C source proof; linked executable proof for class/member/property existence; package checks; fmt, diff check. |
| `8d5f3715` | Method/static signature fallback contract classifies declared receiver/static method metadata as known scoped callable-string signatures or runtime fallback, and feeds the selection through shared source-call binding operands and carriers without adding executable method/static production lowering. | Contract classification test across known, missing, arity-mismatch, heterogeneous, and runtime-dynamic metadata; binding/carrier test across receiver, static, and runtime fallback selections; existing source-call emitter/carrier regression; fmt, diff check. |
| `53bef000` | Source-call binding operands compose method/static target preinvoke cleanup, owner cleanup, signature-driven by-reference argument binding, and the shared `NativeCallArgumentsHandle` path across receiver-method and static-method carrier families. | Source-call emitter carrier test across direct, callable-value, materialized callable, receiver-method, and static-method targets with by-reference signature handoff; carrier selector tests; fmt, diff check. |
| `a7054ed1` | Runtime terminal cleanup transfer consumes a pending terminal result plus aggregated cleanup results, preserves terminal values across non-terminal cleanup, releases them after terminal cleanup, and exposes the ABI to LLVM/generated C without enabling production lowering. | Runtime ABI tests across int/string/array terminal values, mixed cleanup lists, terminal cleanup, and ownership blockers; backend declaration regression; existing cleanup sequencing regression; fmt, diff check. |
| `d67c7f14` | Receiver-method and static-method source-call target operands now compose access-context lookup-plus-invoke helpers with `NativeSourceCallResultCarrier` and the shared exactly-once `NativeCallArgumentsHandle` path, while separating pre-invocation failure cleanup from post-invocation auxiliary cleanup. | Source-call emitter carrier test across direct, callable-value, materialized callable, receiver-method, and static-method targets; carrier selector tests; lookup-plus-invoke declaration test; fmt, diff check. |
| `4d6f0f1e` | Cleanup-frame enqueueing validates producer surfaces and rejects terminal-source value ownership before LLVM/generated C backend emission while preserving the existing cleanup-frame stack/report path. | Cross-backend cleanup-frame surface and terminal-value blocker test, stack aggregation regression, cleanup report bridge regression, fmt, diff check. |
| `8ebeae19` | Cleanup frames can aggregate nested frame stacks in innermost-first unwind order while preserving `NativeDiagnosticCleanupFrameSource` metadata and feeding the existing cleanup report bridge across LLVM and generated C. | New stack aggregation test across LLVM/generated C, cleanup-frame operand/source regression, cleanup report bridge regression, runtime cleanup sequencing, fmt, diff check. |
| `964e3e2b` | Direct generated-C user-function calls route through direct named lookup-plus-invoke source-call carriers and reusable target operands while preserving the shared call-arguments ownership path. | Carrier/emitter and carrier-selector unit tests, generated-C link/run proof across zero/fixed/default/variadic arities and by-reference argument transport, fmt, diff check. |
| `0430efcc` | Cleanup frames carry terminal/control-transfer/deferred-cleanup source metadata and report helpers now consume frames instead of raw result slices. | Compiler cleanup-frame source tests across accepted/rejected operands and both backends, existing report regression, runtime cleanup sequencing, fmt, diff check. |
| `75f20f3f` | Selected generated-C production source-call paths build call arguments once and invoke dynamic callable values, scoped callable-string reference assignments, and materialized direct user-function callables through source-call carriers. | Carrier/emitter unit tests, source-call selector tests, generated-C link/run proof for dynamic callable values and direct user-function frames, fmt, diff check. |
| `d26c64f7` | LLVM/generated-C cleanup frames queue cleanup-surface diagnostic-result operands and reject non-cleanup surfaces before reporting through the control-transfer cleanup bridge. | Compiler cleanup-frame test across value, diagnostic, null, rejected non-cleanup surfaces, both backends, existing bridge regression, runtime cleanup sequencing, fmt, diff check. |
| `50d19f99` | LLVM/generated-C cleanup report bridge consumes already-produced cleanup diagnostic-result operands through the control-transfer cleanup consumer and diagnostics-only report sink. | Compiler bridge test across value, diagnostic, null, non-empty, and empty cleanup lists; fmt, diff check. |
| `7891fcf3` | Runtime converter and compiler selectors compose source-call target helpers with owned-result, value, reference, discard, and diagnostic-result consumers. | Runtime value/reference/failure/null conversion test, compiler carrier selector/declaration tests, fmt, diff check. |
| `8b53ed25` | Control-transfer cleanup `NativeDiagnosticResult` lists consume already-produced cleanup operands in source order, preserve diagnostics, free owned values, and stop after terminal diagnostics. | Runtime ABI shape tests across value, warning, terminal, null-entry, null-list, and empty-list inputs; compiler backend family consumer tests, fmt, diff check. |
| `b7e6f117` | Direct callable, receiver-method, and static-method lookup-plus-invoke helpers consume `NativeCallArgumentsHandle` exactly once and expose compiler helper selection/declarations. | Runtime ownership tests across lookup/invoke success and failure, callable access-context regressions, compiler selector/declaration tests, fmt, diff check. |
| `a991cf34` | Echo operands and statement-form `print` lower into owned `NativeDiagnosticResult` output operands and report/free through the shared echo sink. | Compiler output-operand tests, runtime echo-sink diagnostic test, executable generated-C link/run proof, fmt, diff check. |
| `dcdd330f` | Discarded expression statements lower into owned `NativeDiagnosticResult` operands and report/free through diagnostics-only sinks in LLVM and generated C. | Compiler result-operand tests, `native_runtime_abi` sink tests, executable generated-C link/run proof, fmt, diff check. |
| `5902369c` | Shared callable access-context lookup, allocatable class metadata, diagnostic-result continuation helpers, and stderr/echo report sinks. | Runtime focused tests, compiler ABI declaration tests, `native_runtime_abi` tests, fmt, diff check. |
| `950a17fe` | LLVM/generated-C diagnostic-result family consumers over already-produced operand lists. | Family selector, backend emission, empty-list, missing-runtime-ABI, fmt, diff check. |
| `81c60f38` | Runtime diagnostic-result consumer contracts for value-required and cleanup families. | Result-list ownership, terminal preservation, null/empty list behavior. |
| `099b76fc` | Owned `NativeDiagnosticResult` value/diagnostic/null result contract. | Value, diagnostic, null, list cleanup, adjacent blocker behavior. |
| `08d00fe1` | Conditional call-frame result handoff for short-ternary/null-coalescing families. | Success/diagnostic preservation and cleanup-sensitive blockers. |
| `7fb9db15` | Shared object metadata/type-introspection builtin preflight. | `class_exists`, `property_exists`, `is_a`, direct-call cleanup diagnostics. |
| `a3826e2f` | Generated-C dynamic instance method name normalization through native value helper. | Runtime lookup normalization and generated-C helper selection. |
| `3ac78d8b` | Shared generated-C object-call argument handles. | Constructor, method, static, callable-array, invokable-object argument families. |
| `73195f96` | Native value-result cast diagnostics. | Array-to-string warnings over direct and compound value-result paths. |
| `0bebd2e9` | By-reference alias-transfer result boundary for produced call results. | Direct generated user-function call consumers for echo/print/discard. |
| `b3d90dbc` | Runtime/compiler reference-cell predicate and membership boundaries. | `isset`, `empty`, truthiness, `array_key_exists` over value/reference subjects. |
| `05214fd4` | Compiler-known declared-method callable identities and return summaries. | Public/static/object receiver policy, callable identities, return-summary resolution. |

## Active Roadmap Items

Primary-integrated capability and candidate/lane-local work are separated.

| Item | Primary Integrated | Candidate Readiness | Toward Full Feature | Status |
| --- | ---: | ---: | ---: | --- |
| Diagnostic-result carrier stack | **100%** `[####################]` | **100%** `[####################]` | **60%** `[############--------]` | Runtime/result contracts, family consumers, continuation helpers, report sinks, discarded statement-expression operands, echo/print output operands, source-call result conversion, control-transfer cleanup result consumers/report bridges, and cleanup-frame producers are integrated. Terminal producers, semantic cleanup result production from real control flow, lvalue, reference, RMW, and call-argument operands still need exact ownership and ordering migrations. |
| Callable access context and class metadata | **100%** `[####################]` | **100%** `[####################]` | **65%** `[#############-------]` | Shared runtime access-context policy, lookup-plus-invoke argument ownership, source-call result carrier selectors, selected production source-call carrier emission, direct generated user-function lookup-plus-invoke production, direct user-function reference-return frame consumers, bounded generated-C method/static source-call production including default, variadic, and selected named-argument frame-compatible arities, method/static source-call target and binding operands, method/static signature fallback selection, allocatable-class metadata, generated-C user-class metadata-exists consumers, and value-returning class metadata consumers are integrated for selected function/method/static/constructor/class lookup preflights. Constructor execution, dynamic method names, spread and unsupported named call families, runtime/builtin/inherited/trait/interface signature metadata, non-descriptor closure argument-handle ownership, namespace/function/const fallback, autoload, magic, and full visibility parity remain open. |
| ArrayAccess compiler consumers | **100%** `[####################]` | **100%** `[####################]` | **75%** `[###############-----]` | Generated-C direct-object/direct-variable read, `isset`, `empty`, `??`, write, append, keyed-suffix append, unset, compound assignment, `??=`, selected property-held literal/single-known dynamic object-property read/write/RMW/`??=`/unset owners, and selected nested assignment/RMW/increment/decrement/append/`??=`/unset/keyed-append-suffix owner stacks are integrated for compiler-known generated declared `ArrayAccess` objects. Reference-returning `offsetGet`, arbitrary alias roots, references/COW, cleanup/unwind, and backend parity remain open. |
| ReferenceSlot owner facts | **100%** `[####################]` | **100%** `[####################]` | **50%** `[##########----------]` | Compiler-visible native reference handles can recover facts, source owners, and commit writeback for selected variable, non-local object-property assignment, and property-held ArrayAccess paths. Arbitrary alias roots, request/superglobal path facts, broader property-held reference binding, closure callback fact transport, references/COW, and backend parity remain open. |
| Callable identity return summaries | **100%** `[####################]` | **100%** `[####################]` | **60%** `[############--------]` | Generated functions, declared methods/static methods, descriptor closures, known strings, definite `__invoke` objects, compiler-known callable arrays, and selected direct user-function reference-return frames can publish or consume selected return facts. Unknown runtime callables, builtins, non-descriptor closures, recursive summaries, descriptor/method/closure reference returns, property/magic producers, references/COW, and backend parity remain open. |
| Dynamic object/interface fact carrier | **100%** `[####################]` | **100%** `[####################]` | **65%** `[#############-------]` | Generated declared objects, known dynamic class-name `new`, copies, gotos, branches, generated-callable returns, descriptor closures, known string/invokable/callable-array summaries, compiler-visible reference slots, and selected declared static-property producers feed existing object/interface consumers. Broader properties, clones, dynamic/static property shapes, arbitrary symbols, unknown runtime callables/arrays, and non-descriptor closures remain open. |
| Cleanup/unwind execution | **30%** `[######--------------]` | **30%** `[######--------------]` | **30%** `[######--------------]` | Requirement/preflight boundaries, cleanup result consumers, cleanup report bridges, terminal cleanup transfer ABI, cleanup-frame producer queues, cleanup-frame source metadata, nested cleanup-frame stack aggregation, and cleanup-frame enqueue validation are integrated. Actual exception propagation, catch/finally/destructor/shutdown execution, production cleanup operand enqueueing from real control flow, terminal-kind lowering, and object lifetime cleanup are still not implemented. |
| Broad dirty lane extraction backlog | **0%** `[--------------------]` | **35%** `[#######-------------]` | **35%** `[#######-------------]` | Dirty call, diagnostic, object, control-flow, symbol, byte/string, and array lanes remain evidence pools until split into fresh current-head candidates with focused proof. |

## Done

- Runtime callable table plus call arguments/frame/result ABI.
- Parser/AST named call-argument nodes plus shared call-argument normalization
  bind source-order positional/named arguments to parameter-order required,
  optional/default, by-reference, and variadic slots for selected generated-C
  direct user-function and method/static/dynamic source-call carriers.
- Runtime callable-value dispatch for selected function names, callable arrays,
  descriptor closures, inherited methods, bound receivers, and object
  `__invoke`.
- Runtime lookup-plus-invoke helpers for direct callable, receiver-method, and
  static-method families consume `NativeCallArgumentsHandle` exactly once
  across lookup failure, invoke failure, result handoff, and discard/value/
  reference consumers.
- Runtime receiver-method lookup-plus-invoke can fall back to selected declared
  instance `__call($name, $args)` for missing or inaccessible object receiver
  methods, packing the original method name and value snapshots of the original
  argument slots while preserving normal class-context method hits.
- Runtime static-method lookup-plus-invoke can fall back to selected public
  static `__callStatic($name, $args)` for missing or inaccessible static method
  calls while preserving normal declared static hits and non-static static-call
  failures.
- Source-call result carrier selectors compose direct named, receiver-method,
  static-method, materialized-callable, and callable-value targets with owned
  result, value, reference, discard, and diagnostic-result consumers.
- Generated-C receiver-method and static-method source-call target operands
  compose access-context lookup-plus-invoke helpers with shared source-call
  carriers and distinguish pre-invocation failure cleanup from post-invocation
  auxiliary cleanup.
- Source-call binding operands compose method/static targets with signature-
  driven by-reference argument binding over the shared
  `NativeCallArgumentsHandle` emitter.
- Selected runtime callable builtin signatures feed dynamic source-call
  argument binding and return facts; unsupported runtime builtins such as
  `count` stay blocked before argument construction.
- Method/static signature fallback selection classifies declared receiver and
  static method metadata as known scoped callable-string signatures or runtime
  fallback, then feeds that selection through shared source-call binding
  operands and carriers.
- Bounded generated-C receiver-method and named static-method calls execute
  through shared source-call target operands, binding operands, result
  carriers, and runtime diagnostics when exact frame-compatible signatures are
  known.
- Generated-C receiver-method and named static-method calls with supported
  default or variadic declared method parameters synthesize frame-shaped call
  arguments through the shared `NativeCallArgumentsHandle` and source-call
  carriers.
- Generated-C selected static method source calls route missing or inaccessible
  static targets through shared runtime `__callStatic` lookup-plus-invoke
  fallback without generated method-name ladders.
- Selected generated-C production source-call paths build
  `NativeCallArgumentsHandle` once and invoke dynamic callable values,
  scoped callable-string reference assignments, and direct generated
  user-function lookup-plus-invoke calls through source-call carriers.
- Selected generated-C by-reference call arguments can materialize proven
  reference-return direct, dynamic, receiver-method, and named static-method
  source-call results through the shared source-call reference carrier and
  `NativeCallArgumentsHandle` push-reference path.
- Direct generated-C user-function calls and dynamic generated-C callee
  expressions through shared runtime callable lookup/invocation.
- Direct generated-C user-function reference-return frames can return
  by-reference parameter sources through native reference handles and feed
  reference consumers through source-call carriers.
- Explicit by-value generated-C function and method returns route through the
  terminal-kind `NativeDiagnosticResult` cleanup handoff before returning
  values to existing native-value and closure-value frame contracts.
- Generated declared-method callable-table registration and wrapper frames.
- Generated-C user-class declarations register class, parent, method, and
  property metadata, and generated-C `class_exists()`, `method_exists()`, and
  `property_exists()` consume that registry through runtime value operands and
  shared diagnostics.
- Generated-C `get_parent_class()`, `class_parents()`,
  `get_declared_classes()`, `get_class_methods()`, and `get_class_vars()`
  consume shared runtime class metadata value handles for selected registered
  user classes and core class metadata.
- LLVM top-level user-class declarations and LLVM `class_exists()`,
  `method_exists()`, and `property_exists()` route through the shared class
  metadata registry ABI.
- Generated-C parser-resolved namespace/import class policy, named
  `ClassName::class`, leading-backslash/case-normalized dynamic class-name
  matching, and explicit `class_exists()` autoload policy boundary.
- Generated-C `class_alias()` registers normalized aliases for already-declared
  metadata targets and routes class/member metadata consumers through canonical
  alias lookup without fake autoload success.
- Parser/runtime `use function` imports with import-kind metadata, arbitrary
  aliases/default aliases, exact imported lookup without global suffix
  fallback, and alias-conflict guards. Generated C rejects function imports at
  an explicit production boundary until native imported-call lowering exists.
- Parser/runtime `use const` imports with import-kind metadata, arbitrary
  aliases/default aliases, exact imported lookup before namespace/global
  fallback, declaration/import conflict guards, and explicit generated-C/LLVM
  production rejection boundaries.
- Generated non-static method/constructor frames can assign literal and
  dynamic `$this` properties through the shared object-property mutation ABI.
- Generated-C selected non-local object-property assignments route literal and
  single-known dynamic public-property owners through reference-slot commits.
- Generated-C declared constructors with supported bodies can use bare
  `return;` early exits, and constructor `return <value>` now reports an
  explicit generated-C diagnostic with cleanup instead of pre-rejecting.
- Runtime declared static-property storage initializes defaults per request,
  preserves reference identity, enforces visibility shadowing and type checks,
  resolves late-static receivers, and resets storage between request states.
- Generated-C declared literal `Class::$prop` reads and writes route through
  request-owned runtime static-property storage with default/reset
  registration, type/visibility checks, canonical alias lookup, assignment
  result ownership, and explicit unsupported-shape blockers.
- Generated-C declared static-property storage initialization consumes bulk
  declared-property metadata/default arrays for names, visibility, types,
  defaults, and static flags, including selected typed static-property
  defaults/writes through runtime diagnostics.
- Generated-C selected object and declared class-string static-property
  receiver reads and plain assignments route through runtime receiver-scope
  derivation and request-owned static-property storage.
- Generated-C proven descriptor-closure calls route through shared call
  argument handles and runtime closure-invoke result carriers.
- Runtime terminal-kind diagnostic results can preserve return/throw/exit kind
  through cleanup transfer and report sinks.
- Structured generated-C object-property owner/fact/commit prerequisite
  boundary over public-property reference slots, including conservative fact
  invalidation and cleanup ownership for future property-held ArrayAccess.
- Receiver-free static `Class::method` string callable lookup through the
  runtime callable-value ABI.
- Shared diagnostic operation/operand-list blocker boundary.
- Owned diagnostic-result contracts, family consumers, continuation helpers,
  and report sinks for selected diagnostic-result paths.
- Discarded expression statements in LLVM and generated C lower through owned
  `NativeDiagnosticResult` statement operands and diagnostics-only report
  sinks.
- Echo operands and statement-form `print` in LLVM and generated C lower
  through owned `NativeDiagnosticResult` output operands and the shared echo
  report sink, including array-to-string conversion diagnostics.
- Control-transfer cleanup result lists have a runtime/backend consumer that
  consumes already-produced cleanup operands in source order, frees owned
  values, preserves diagnostics, and stops after terminal diagnostics.
- Runtime terminal cleanup transfer can preserve a pending terminal value
  through non-terminal cleanup result sequencing and release it when cleanup
  becomes terminal.
- LLVM and generated C have a shared control-transfer cleanup report bridge
  over already-produced cleanup diagnostic-result operands.
- LLVM and generated C have cleanup-frame producer queues that accept only
  cleanup-surface `NativeDiagnosticResult` operands before feeding the
  control-transfer cleanup report bridge.
- Cleanup frames carry terminal/control-transfer/deferred-cleanup source
  metadata and control-transfer cleanup reports consume frames instead of raw
  result slices.
- Cleanup-frame stacks aggregate nested frames innermost-first while preserving
  source metadata before feeding the shared cleanup report bridge.
- Cleanup-frame enqueueing validates cleanup surfaces and rejects
  terminal-source value ownership across LLVM and generated C before backend
  cleanup-frame emission.
- Reference-binding, assignment-lvalue, and RMW-lvalue operand-list blockers.
- Generated-C selected RMW array-lvalue owner/writeback for local native arrays
  and active-symbol/global-import reference-slot owners.
- Cleanup/unwind requirement diagnostics/preflight.
- Runtime ArrayAccess read/exists and write/append/unset dispatch ABIs.
- Generated-C ArrayAccess read/isset/write/append/unset/empty/null-coalesce/
  RMW/`??=` consumers for compiler-known generated declared `ArrayAccess`
  objects.
- Generated-C selected property-held ArrayAccess read/isset/empty/`??`, write,
  unset, compound RMW, and `??=` owners for literal and single-known dynamic
  object properties proven by constructor `$this` property facts.
- Generated-C selected nested ArrayAccess assignment, compound-assignment,
  pre/post increment/decrement, append, and `??=` owner stacks for
  direct-variable and property-held roots, including by-value descent, leaf
  mutation, reverse parent writeback, lazy null-coalescing RHS branches, and
  root commit.
- Shared generated-C native value/object facts for selected generated declared
  object producers and callable return summaries.
- Shared generated-C ReferenceSlot value owner source/commit and reference-cell
  fact ledger for compiler-visible native reference handles.
- Declared-class allocation cleanup-risk metadata and allocatable-class lookup
  metadata.
- Selected reference-source/lvalue extraction, reference-backed closure capture
  materialization, descriptor closure returns, byte-backed PHP string values,
  and byte-preserving selected string-array slots.

## Not Done

- Dynamic ArrayAccess producers beyond known generated declared-class `new` and
  direct generated-callable return summaries.
- Nested ArrayAccess root keyed-suffix append/reference-returning production,
  non-direct property-held ArrayAccess forms, unknown dynamic property owners,
  dynamic class-name holders without definite facts, and reference-returning
  ArrayAccess semantics on top of the integrated object-property owner
  boundary.
- LLVM static-property producers, generated-C dynamic and broader
  object/static-property shapes, broader static-property reference/`??=`,
  unset/isset/empty lowering, dynamic/magic property lowering, and full
  method/object model execution beyond the selected generated `$this`
  assignment and declared static-property subsets.
- Full reference/COW identity and arbitrary alias-root writeback.
- Actual exception/Throwable propagation, catch matching/binding, `finally`,
  destructors, shutdown cleanup, and object lifetime cleanup.
- Full SPL autoload, broader class-alias parity, broader const-import
  discovery/fallback, broader function-import discovery/fallback,
  broader namespace/function/const fallback, broader visibility,
  named-argument `__callStatic` fallback, malformed magic signature parity,
  broader magic-call coverage, broader constructor allocation/execution,
  spread arguments,
  unsupported named builtin/constructor/fallback call families, and return
  references.
- Broader source-call production lowering over expression-owned
  `NativeCallResultHandle` carriers, including remaining dynamic/magic method
  shapes, late-static binding, constructor allocation,
  non-descriptor closure invocation ownership, direct function/method/static produced-call
  by-reference alias transfer, unknown runtime callable reference returns,
  runtime/builtin/inherited/trait/interface signature metadata, unsupported
  named argument consumers, and spread ownership.
- Remaining semantic diagnostic-result operand migration for throw/exit/default
  return terminals, cleanup frame/result production from real control flow,
  lvalue, reference, RMW, and call-argument families; exact PHP diagnostics,
  source ordering, suppression/custom handlers, and backend parity across
  generated C, LLVM, and direct assembly.
- Pending diagnostic production from real control-flow cleanup, remaining
  terminal-kind lowering over the terminal transfer ABI, and exact
  `finally`/destructor/shutdown sequencing.

## Latest Focused Verification

For `1202542e`:

- `cargo fmt --all -- --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test -p php_runtime static_property -- --nocapture`
- `cargo test -p phpc --test native_link static_property -- --nocapture`
- `cargo test -p phpc --test native_link static_property_mutation -- --nocapture`
- `cargo test -p phpc --test native_link object_static_property_receiver -- --nocapture`
- Focused adjacent regressions for descriptor closures, magic static calls,
  source-call references, nested ArrayAccess, property-held ArrayAccess, and
  class aliases, plus `cargo check -q -p php_runtime -p phpc`. Primary
  integration log:
  `/home/claude/supervised-php-compiler/state/workers/logs/phpc-primary-static-property-mutation-r4-integration-20260527.gates.log`
  (`SUMMARY passes=14 failures=0`, sha256
  `f74ebb2b211e2a3b08f6e47879acc6e62fefbf99e3d61b5a279072211066835e`).

For `8a8d85ed`:

- `cargo fmt --all -- --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test -p phpc --lib native_nested_arrayaccess_owner_stack -- --test-threads=1`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_nested_arrayaccess_null_coalesce_owner_stack_for_direct_and_property_roots -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_nested_arrayaccess_null_coalesce_owner_stack_program -- --exact --test-threads=1`
- Focused adjacent regressions for nested ArrayAccess, property-held
  ArrayAccess, descriptor closures, magic static calls, static properties,
  source-call references, and class aliases, plus `cargo check -q -p phpc`.
  Primary integration log:
  `/home/claude/supervised-php-compiler/state/workers/logs/phpc-primary-nested-arrayaccess-nullcoalesce-r3-integration-20260527.gates.log`
  (`SUMMARY passes=14 failures=0`, sha256
  `eccbf159904d5c93cbb338668436db77b726d6d08174b100069ab96fca53318a`).

For `f65be9b1`:

- `cargo fmt --all -- --check`
- `git diff --check`
- `cargo test -p php_runtime call_static -- --nocapture`
- `cargo test -p php_runtime native_method_lookup -- --nocapture`
- `cargo test -p php_runtime native_lookup_plus_invoke -- --nocapture`
- `cargo test -p phpc --lib native_method_static_signature_fallback_contract -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_magic_static_methods_through_runtime_dispatch_boundary -- --exact --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_magic_static_method_source_call_program -- --exact --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_blocks_named_static_magic_fallback_without_shared_contract -- --exact --nocapture`
- Focused adjacent regressions for descriptor closures, object static-property
  receivers, named arguments, magic dynamic/static calls, object-static,
  class-context, self/parent/static, late static, static properties, typed
  instance properties, exact imports, source-call references, nested
  ArrayAccess, and class aliases.
- `cargo check -p php_runtime -p phpc`
- Primary integration log:
  `/tmp/phpc-primary-callstatic-magic-r2-integration-20260527.gates.log`
  (`SUMMARY passes=25 failures=0`).

For `c2ffab4b`:

- `cargo fmt --all -- --check`
- `cargo test -p php_runtime static_property -- --nocapture`
- `cargo test -p phpc --test native_link static_property -- --nocapture`
- `cargo test -p phpc --test native_link object_static_property_receiver -- --nocapture`
- `cargo test -p phpc --test native_link descriptor_closure -- --nocapture`
- Focused adjacent regressions for named arguments, magic/runtime dynamic
  methods, exact imports, late static, typed declared instance properties,
  source-call references, class aliases, and nested ArrayAccess.
- `git diff --check`
- `cargo check -p php_runtime -p phpc`
- Primary integration log:
  `/tmp/phpc-primary-typed-static-property-r11-integration-20260527.gates.log`
  (`SUMMARY passes=16 failures=0`).

For `55aef1ee`:

- `cargo fmt --all -- --check`
- `cargo test -p php_runtime native_closure_invoke_helpers_bridge_call_arguments_to_call_results -- --nocapture`
- `cargo test -p phpc generated_c_descriptor_closure_calls_use_shared_call_arguments_and_results -- --nocapture`
- `cargo test -p phpc --test native_link descriptor_closure -- --nocapture`
- Focused adjacent regressions for named arguments, exact imports,
  magic/runtime/dynamic method source calls, late/static properties,
  object static-property receivers, object-static calls, typed properties,
  try/finally, ArrayAccess reference slots, property-held and nested
  ArrayAccess, source-call references, and class aliases. Primary integration
  log:
  `/tmp/phpc-primary-closure-call-production-r8-integration-20260527.gates.log`
  (`SUMMARY passes=28 failures=0`).

For `147ad3b5`:

- `cargo fmt --all -- --check`
- `cargo test -p php_runtime static_property -- --nocapture`
- `cargo test -p phpc --test native_link object_static_property_receiver -- --nocapture`
- Focused adjacent regressions for named arguments, magic receiver calls, exact
  imports, declared/late static properties, runtime dynamic methods, typed
  properties, nested/property-held ArrayAccess, reference-slot owners,
  constructor value-return diagnostics, non-local object properties,
  source-call reference aliases, and class aliases. Primary integration log:
  `/tmp/phpc-primary-static-property-object-receiver-r4-integration-20260527.gates.log`
  (`SUMMARY passes=20 failures=0`).

For `28ce0faa`:

- `cargo fmt -q -p phpc -- --check`
- `cargo test -q -p phpc --lib native_nested_arrayaccess_owner_stack -- --test-threads=1`
- `cargo test -q -p phpc --test native_link native_executable_c_source_routes_nested_arrayaccess_increment_decrement_owner_stack_for_direct_and_property_roots -- --exact --test-threads=1`
- `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_nested_arrayaccess_increment_decrement_owner_stack_program -- --exact --test-threads=1`
- Focused adjacent regressions for nested assignment/RMW, unsupported
  reference-returning/non-assignment nested mutations, property-held ArrayAccess
  owners, named call arguments, magic receiver calls, exact function/const
  imports, late/static properties, runtime dynamic methods, typed properties,
  source-call by-reference arguments, constructor value-return diagnostics, and
  class aliases. Primary integration log:
  `/tmp/phpc-primary-nested-arrayaccess-mutation-r3-integration-20260527.gates.log`
  (`SUMMARY passes=47 failures=0`).

For `425f1ef5`:

- `cargo fmt -- --check`
- `git diff --check`
- `cargo test -q -p phpc --lib call_arguments::tests:: --`
- `cargo test -q -p phpc --test syntax_boundaries named_arguments_parse_as_source_ordered_call_argument_nodes -- --exact --nocapture`
- `cargo test -q -p phpc --test syntax_boundaries emit_ir_rejects_named_builtin_arguments_at_codegen_boundary -- --exact --nocapture`
- `cargo test -q -p phpc --test native_link native_executable_c_source_lowers_named_user_function_arguments_through_shared_normalization -- --exact --nocapture`
- `cargo test -q -p phpc --test native_link native_executable_c_source_lowers_named_method_source_call_arguments_through_carriers -- --exact --nocapture`
- `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_named_dynamic_method_source_call_argument_program -- --exact --nocapture`
- Focused adjacent regressions for magic dynamic methods, exact function/const
  imports, late/static properties, typed properties, nested/property-held
  ArrayAccess, source-call references, class aliases, runtime magic lookup, and
  native source-call carrier contracts. Primary integration log:
  `/tmp/phpc-primary-named-callargs-r15-integration-20260527.gates.log`
  (`SUMMARY passes=38 failures=0`).

For `eedd0879`:

- `cargo fmt -- --check`
- `git diff --check`
- `cargo test -q -p php_runtime --lib tests::native_method_lookup_plus_invoke_dispatches_missing_and_inaccessible_methods_to_magic_call -- --exact --nocapture`
- `cargo test -q -p php_runtime --lib native_method_lookup -- --nocapture`
- `cargo test -q -p php_runtime --lib native_lookup_plus_invoke -- --nocapture`
- `cargo test -q -p phpc --test native_link native_executable_c_source_routes_magic_dynamic_methods_through_runtime_dispatch_boundary -- --exact --nocapture`
- `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_magic_dynamic_method_source_call_program -- --exact --nocapture`
- Focused adjacent regressions for dynamic method source calls, class-context
  method calls, object-static calls, late-static/static-property paths, typed
  properties, nested ArrayAccess, source-call reference aliases, class aliases,
  and exact const imports. Primary integration log:
  `/tmp/phpc-primary-magic-call-r2-integration-20260527.gates.log`
  (`SUMMARY passes=18 failures=0`).

For `e8268187`:

- `cargo fmt -q -p phpc -- --check`
- `cargo test -q -p phpc --lib native_nested_arrayaccess_owner_stack -- --test-threads=1`
- `cargo test -q -p phpc --test native_link native_executable_c_source_routes_nested_arrayaccess_owner_stack_for_direct_and_property_roots -- --exact --test-threads=1`
- `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_nested_arrayaccess_owner_stack_program -- --exact --test-threads=1`
- `cargo test -q -p phpc --test native_link native_executable_c_source_rejects_nested_arrayaccess_reference_returning_offsetget -- --exact --test-threads=1`
- `cargo test -q -p phpc --test native_link native_executable_c_source_rejects_nested_arrayaccess_non_assignment_mutations -- --exact --test-threads=1`
- Focused adjacent regressions for property-held ArrayAccess owners, declared
  static-property storage, object-static and class-context source-call carriers,
  source-call by-reference aliasing, constructor value-return diagnostics,
  non-local property owner commits, runtime dynamic builtins, and class-alias
  metadata. Primary integration log:
  `/tmp/phpc-primary-nested-arrayaccess-r9-integration-20260527.gates.log`
  (`SUMMARY passes=41 failures=0`).

For `5bacf1aa`:

- `cargo fmt --all -- --check`
- `git diff --check`
- `cargo test -q -p phpc --test namespace_resolution function_imports_resolve_aliases_and_keep_non_imported_fallback -- --exact`
- `cargo test -q -p phpc --test namespace_resolution function_imports_use_exact_lookup_without_global_suffix_fallback -- --exact`
- `cargo test -q -p phpc --test namespace_resolution missing_imported_function_and_non_imported_namespaced_calls_report_distinct_runtime_names -- --exact`
- `cargo test -q -p phpc --test namespace_resolution generated_c_lowers_imported_runtime_builtin_function_boundary -- --exact`
- `cargo test -q -p phpc --test namespace_resolution generated_c_rejects_qualified_imported_type_builtin_without_exact_user_function -- --exact`
- `cargo test -q -p phpc --test native_link native_executable_c_source_lowers_exact_imported_user_function_aliases -- --exact`
- `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_exact_imported_user_function_alias_program -- --exact`
- `cargo test -q -p phpc --test native_link native_executable_c_source_lowers_exact_imported_runtime_builtin_aliases -- --exact`
- `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_exact_imported_runtime_builtin_alias_program -- --exact`
- `cargo test -q -p phpc --test native_link native_executable_c_source_preserves_unsupported_imported_builtin_lookup_boundary -- --exact`
- `cargo test -q -p phpc --test native_link emit_exe_reports_unsupported_imported_builtin_before_arguments -- --exact`
- Focused adjacent regressions for dynamic builtins, dynamic methods,
  object-static source calls, property-held ArrayAccess owners, declared
  static-property storage, terminal returns, and runtime callable builtin
  signatures. Primary integration log:
  `/tmp/phpc-primary-function-import-r12-integration-20260527.gates.log`
  (`SUMMARY passes=26 failures=0`).

For `e81aa43e`:

- `cargo fmt --all --check`
- `git diff --check`
- `cargo test -p phpc --test namespace_resolution -- --test-threads=1`
- `cargo test -p phpc --test unsupported_dynamic_features_cli cli_unsupported_dynamic_feature_snapshots_match_committed_outputs -- --exact --test-threads=1`
- `cargo test -p phpc --test native_global_constant_boundary emit_ir_rejects_bare_constant_reads_with_specific_boundary -- --test-threads=1`

For `6757bb43`:

- `cargo fmt --all --check`
- `git diff --check`
- `cargo test -p phpc --lib native_object_property_owner -- --test-threads=1`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_property_held_arrayaccess_through_object_property_owner_boundary -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_property_held_arrayaccess_owner_program -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link native_executable_c_source_rejects_arrayaccess_rmw_unsupported_owner_shapes -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_arrayaccess_rmw_nullcoalesce_assignment_through_owner_boundary -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_arrayaccess_rmw_nullcoalesce_assignment_runtime_consumer_program -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link this_property_assignment -- --test-threads=1`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_declared_class_constructor_program -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_declared_method_static_default_variadic_calls_through_source_call_carriers -- --exact --test-threads=1`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_declared_method_static_default_variadic_source_call_program -- --exact --test-threads=1`

For `b3f16040`:

- `cargo test -p phpc native_object_property_owner -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_arrayaccess_reference_slot_owners_through_value_owner_boundary -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_rejects_arrayaccess_rmw_unsupported_owner_shapes -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_arrayaccess_rmw_nullcoalesce_assignment_through_owner_boundary -- --nocapture`
- `cargo fmt --check`
- `git diff --check`

For `d96cc2bb`:

- `cargo test -p phpc --test native_runtime_abi native_user_class_metadata_registry_emit_ir_routes_llvm_through_runtime_abis -- --exact`
- `cargo test -p phpc --test native_runtime_abi native_user_class_metadata_registry_emit_asm_accepts_llvm_runtime_abi_routing -- --exact`
- `cargo test -p phpc --test object_model emit_ir_routes_class_exists_through_native_metadata_abi_and_folds_other_absent_metadata_calls -- --exact`
- `cargo test -p phpc --test object_model emit_ir_routes_absent_native_property_and_method_exists_calls_through_metadata_abi -- --exact`
- `cargo test -p phpc --test native_object_class_boundary metadata_registry`
- `cargo test -p phpc --test object_model until_native_object_lowering_exists`
- `cargo test -p phpc --lib native_object_metadata_call_preflight_reuses_direct_call_diagnostics`
- `cargo fmt --check`
- `git diff --check`

For `cb8457f1`, `ac5004a3`, and `04b18506`:

- `cargo test -p phpc --test native_link emit_exe_links_and_runs_namespace_alias_class_policy_program -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_namespaced_class_exists_user_function_takes_exact_precedence -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_class_exists_missing_default_reports_autoload_boundary -- --nocapture`
- `cargo test -p phpc --test native_runtime_abi terminal_kind -- --nocapture`
- `cargo test -p phpc --lib native_diagnostic_result_terminal_kind -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_this_property_assignment_in_generated_method_frames -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_this_property_assignment_reports_mutation_failure_from_method_frame -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_declared_class_constructor_program -- --nocapture`
- `cargo fmt --check`
- `git diff --check`

For `c8aeb771`:

- `cargo test -p phpc --test native_link native_executable_c_source_routes_declared_method_static_calls_through_source_call_carriers -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_declared_method_static_source_call_program -- --nocapture`
- `cargo test -p phpc --lib native_method_static_signature_fallback_contract_feeds_shared_binding_and_carriers -- --nocapture`
- `cargo test -p phpc --lib native_source_call -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_declared_static_methods_through_frame_dispatch -- --nocapture`
- `cargo fmt --check`
- `git diff --check`

For `4e5e2709`:

- `cargo test -p php_runtime native_user_class_metadata_registry_feeds_value_metadata_surfaces -- --nocapture`
- `cargo test -p phpc --test native_link user_class_value_metadata_registry -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_value_metadata_consumers_through_runtime_registry -- --nocapture`
- `cargo test -p phpc --lib native_object_metadata_call_preflight_reuses_direct_call_diagnostics -- --nocapture`
- `cargo fmt --check`
- `git diff --check`

For `69526631`:

- `cargo test -p phpc --test native_link native_executable_c_source_transfers_reference_return_source_calls_into_byref_arguments -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_source_call_reference_alias_byref_argument_program -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_reports_source_call_reference_alias_argument_cleanup_failure -- --nocapture`
- `cargo test -p phpc --test native_runtime_abi native_c_byref_produced_argument_consumers_use_alias_transfer_result_vector -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_scoped_callable_string_signature_program -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_plans_scoped_callable_string_signature_arguments -- --nocapture`
- `cargo test -p phpc --test native_runtime_abi native_call_frame_byref_alias_transfer_result_vector_preserves_targets_and_families -- --nocapture`
- `cargo test -p phpc --test native_runtime_abi native_c_adjacent_argument_binding_avoids_alias_transfer_without_produced_byref_args -- --nocapture`
- `cargo fmt --check`
- `git diff --check`

For `8d5f3715`, `53bef000`, `a7054ed1`, `d67c7f14`, `4d6f0f1e`, `8ebeae19`, `964e3e2b`,
`0430efcc`, `75f20f3f`, `d26c64f7`, `50d19f99`, and `7891fcf3`:

- `cargo test -p phpc --lib native_method_static_signature_fallback_contract_classifies_declared_and_runtime_metadata -- --nocapture`
- `cargo test -p phpc --lib native_method_static_signature_fallback_contract_feeds_shared_binding_and_carriers -- --nocapture`
- `cargo test -p phpc --lib native_diagnostic_cleanup_frame_stack_aggregates_nested_pending_diagnostics_for_unwind -- --nocapture`
- `cargo test -p phpc --lib native_diagnostic_cleanup_frames_enforce_cleanup_surface_and_terminal_value_ownership -- --nocapture`
- `cargo test -p phpc --lib native_diagnostic_result_control_transfer_cleanup_reports_reusable_cleanup_operands -- --nocapture`
- `cargo test -p phpc --test native_runtime_abi native_diagnostic_result_control_transfer_cleanup_sequences_result_shapes -- --nocapture`
- `cargo test -p phpc --test native_runtime_abi terminal_value_transfer -- --nocapture`
- `cargo test -p phpc --lib native_source_call_emitter_builds_arguments_once_and_routes_carriers -- --nocapture`
- `cargo test -p phpc --lib native_source_call -- --nocapture`
- `cargo test -p phpc --lib native_invoke_result_helper_selector_routes_lookup_plus_invoke_families -- --nocapture`
- `cargo test -p phpc --lib native_callable_runtime_boundary_declares_lookup_plus_invoke_helpers -- --nocapture`
- `cargo test -p phpc --test native_function_call_boundary native_executable_direct_user_function_calls_use_runtime_callable_abi_across_arities -- --nocapture`
- `cargo test -p phpc --test native_function_call_boundary native_executable_direct_user_function_calls_preserve_reference_arguments_through_runtime_callable_abi -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_dynamic_string_callable_value_program -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_direct_user_function_frame_program -- --nocapture`
- `cargo test -p phpc --test native_link native_executable_c_source_routes_dynamic_callable_values_through_runtime_abi -- --nocapture`
- `cargo test -p php_runtime native_diagnostic_result_from_call_result_consumes_value_reference_failure_and_null -- --nocapture`
- `cargo test -p phpc --lib native_source_call_result_carrier_routes_targets_and_consumers_through_owned_results -- --nocapture`
- `cargo test -p phpc --lib native_callable_runtime_boundary_declares_call_result_diagnostic_converter -- --nocapture`
- `cargo fmt --check -p phpc -p php_runtime`
- `git diff --check`
