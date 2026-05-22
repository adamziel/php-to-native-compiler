use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::ast::{
    ArrayItem, AssignTarget, BinaryOp, CastKind, ClassMember, CompoundAssignOp, Expr, ForAction,
    FunctionDecl, FunctionParam, IncrementDecrementOp, IncrementDecrementPosition, Program,
    ReferenceSource, Span, Stmt, UnaryOp, UnsetTarget,
};
use crate::error::{CompileResult, Diagnostic, Phase};
use php_runtime::{
    classify_php_numeric_string, is_php_truthy_string, php_primitive_arithmetic_result,
    php_strings_use_numeric_comparison, NativeComparisonOp, NativeFilesystemPathOperation,
    NativeIntConversionOperation, NativeStringDistanceOperation, NativeStringIntOperation,
    NativeStringOffsetOperation, NativeStringPredicate, NativeStringResultOperation,
    PhpPrimitiveArithmeticError, PhpPrimitiveArithmeticOperation, PhpPrimitiveArithmeticValue,
    PhpPrimitiveValue,
};

const MAX_KNOWN_INT_VALUES: usize = 4;
const MAX_KNOWN_FLOAT_VALUES: usize = 4;
const MAX_KNOWN_STRING_VALUES: usize = 4;
const NATIVE_FILESYSTEM_PATH_HAS_BOOLEAN_OPTION: u8 = 1;
const NATIVE_FILESYSTEM_PATH_HAS_PATH: u8 = 8;
const LLVM_CONDITIONAL_REJECTION: &str = "LLVM conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior";
const ASSEMBLY_CONDITIONAL_REJECTION: &str = "assembly conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior";
const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const ASSEMBLY_FUNCTION_CALL_REJECTION: &str = "assembly function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_STRING_PREDICATE_REJECTION: &str = "LLVM string-predicate lowering rejects str_starts_with(), str_ends_with(), and str_contains() until native PHP string conversion, empty-needle handling, binary string byte semantics, argument diagnostics, references/copy-on-write, and exact native predicate diagnostics exist; generated-native C routes lowerable predicate operands through the shared runtime contract";
const ASSEMBLY_STRING_PREDICATE_REJECTION: &str = "assembly string-predicate lowering rejects forms outside the reusable native string predicate contract until operands can reach byte-preserving value conversion, diagnostics, and cleanup; generated-native C routes lowerable str_starts_with(), str_ends_with(), and str_contains() operands through the shared runtime contract";
const LLVM_STRING_INT_OPERATION_REJECTION: &str = "LLVM string-int builtin lowering rejects strcasecmp(), strcmp(), strncmp(), strncasecmp(), substr_count(), ord(), and crc32() forms outside the reusable native string-int operation contract, including unsupported arity, non-lowerable operands, nested call cleanup, references/copy-on-write, and exact native builtin diagnostics; lowerable LLVM and generated-native C string-int operands route through the shared runtime contract";
const ASSEMBLY_STRING_INT_OPERATION_REJECTION: &str = "assembly string-int builtin lowering rejects strcasecmp(), strcmp(), strncmp(), strncasecmp(), substr_count(), ord(), and crc32() forms outside the reusable native string-int operation contract until operands can reach byte-preserving value conversion, diagnostics, and cleanup";
const LLVM_STRING_DISTANCE_OPERATION_REJECTION: &str = "LLVM string-distance builtin lowering rejects levenshtein() and similar_text() until native PHP value-to-string byte conversion, optional cost conversion, references/copy-on-write, by-reference percent output, and exact native diagnostics exist; generated-native C routes lowerable string-distance operands through the shared runtime contract";
const ASSEMBLY_STRING_DISTANCE_OPERATION_REJECTION: &str = "assembly string-distance builtin lowering rejects levenshtein() and similar_text() forms outside the reusable native string-distance operation contract until operands can reach byte-preserving value conversion, diagnostics, and cleanup";
const LLVM_STRING_RESULT_OPERATION_REJECTION: &str = "LLVM string-result builtin lowering rejects strrev(), str_rot13(), bin2hex(), strtolower(), strtoupper(), ucfirst(), lcfirst(), escapeshellarg(), and escapeshellcmd() until native PHP string result ownership, byte-preserving conversion, diagnostics, references/copy-on-write, and exact native builtin diagnostics exist; generated-native C routes lowerable string-result operands through the shared runtime contract";
const ASSEMBLY_STRING_RESULT_OPERATION_REJECTION: &str = "assembly string-result builtin lowering rejects forms outside the reusable native string-result operation contract until operands can reach byte-preserving value conversion, diagnostics, result ownership, and cleanup";
const LLVM_BASENAME_REJECTION: &str = "LLVM basename lowering rejects direct path basename calls until native PHP path string conversion, suffix handling, trailing-separator normalization, Windows/UNC and stream-wrapper path semantics, locale/codepage behavior, argument diagnostics, references/copy-on-write, and exact native basename diagnostics exist; phpc run handles current bounded basename behavior";
const ASSEMBLY_BASENAME_REJECTION: &str = "assembly basename lowering rejects direct path basename calls until native PHP path string conversion, suffix handling, trailing-separator normalization, Windows/UNC and stream-wrapper path semantics, locale/codepage behavior, argument diagnostics, references/copy-on-write, and exact native basename diagnostics exist; phpc run handles current bounded basename behavior";
const LLVM_FILE_GET_CONTENTS_REJECTION: &str = "LLVM file_get_contents lowering rejects direct filesystem reads until native PHP stream wrapper handling, local file I/O, binary string byte fidelity, warning plus false recovery, stream contexts, include-path lookup, open_basedir/stat-cache behavior, references/copy-on-write, and exact native file_get_contents diagnostics exist; phpc run handles current bounded file_get_contents behavior including UTF-8 offset/length reads and selected warning-plus-false recovery";
const ASSEMBLY_FILE_GET_CONTENTS_REJECTION: &str = "assembly file_get_contents lowering rejects direct filesystem reads until native PHP stream wrapper handling, local file I/O, binary string byte fidelity, warning plus false recovery, stream contexts, include-path lookup, open_basedir/stat-cache behavior, references/copy-on-write, and exact native file_get_contents diagnostics exist; phpc run handles current bounded file_get_contents behavior including UTF-8 offset/length reads and selected warning-plus-false recovery";
const LLVM_STREAM_RESOURCE_REJECTION: &str = "LLVM stream-resource lowering rejects fopen(), stream_context_create(), stream_context_get_options(), stream_context_get_params(), stream_context_get_default(), stream_context_set_default(), stream_context_set_option(), stream_context_set_params(), fwrite(), fread(), rewind(), stream_get_contents(), feof(), ftell(), fseek(), fstat(), stream_get_meta_data(), fclose(), opendir(), readdir(), rewinddir(), closedir(), is_uploaded_file(), and move_uploaded_file() until native PHP resource handles, stream wrapper state, stream context state, directory handle state, upload provenance state, binary string byte fidelity, warning plus false recovery, references/copy-on-write, and exact native stream diagnostics exist; phpc run handles current bounded php://memory, php://temp, php://input, local file stream resources, stream context resources, local directory handles, and PHPC_FILES upload provenance";
const ASSEMBLY_STREAM_RESOURCE_REJECTION: &str = "assembly stream-resource lowering rejects fopen(), stream_context_create(), stream_context_get_options(), stream_context_get_params(), stream_context_get_default(), stream_context_set_default(), stream_context_set_option(), stream_context_set_params(), fwrite(), fread(), rewind(), stream_get_contents(), feof(), ftell(), fseek(), fstat(), stream_get_meta_data(), fclose(), opendir(), readdir(), rewinddir(), closedir(), is_uploaded_file(), and move_uploaded_file() until native PHP resource handles, stream wrapper state, stream context state, directory handle state, upload provenance state, binary string byte fidelity, warning plus false recovery, references/copy-on-write, and exact native stream diagnostics exist; phpc run handles current bounded php://memory, php://temp, php://input, local file stream resources, stream context resources, local directory handles, and PHPC_FILES upload provenance";
const LLVM_GETCWD_REJECTION: &str = "LLVM getcwd lowering rejects direct current-directory calls until native process/request cwd state, UTF-8/path policy, SAPI cwd behavior, chdir() interaction, failure false recovery, references/copy-on-write, and exact native getcwd diagnostics exist; phpc run handles current bounded getcwd behavior";
const ASSEMBLY_GETCWD_REJECTION: &str = "assembly getcwd lowering rejects direct current-directory calls until native process/request cwd state, UTF-8/path policy, SAPI cwd behavior, chdir() interaction, failure false recovery, references/copy-on-write, and exact native getcwd diagnostics exist; phpc run handles current bounded getcwd behavior";
const LLVM_REALPATH_REJECTION: &str = "LLVM realpath lowering rejects direct filesystem canonicalization calls until native filesystem canonicalization, symlink/path policy, warning/false recovery, include_path/open_basedir/stat cache, non-UTF-8 path handling, references/COW, and exact native realpath diagnostics exist; phpc run handles current bounded realpath behavior";
const ASSEMBLY_REALPATH_REJECTION: &str = "assembly realpath lowering rejects direct filesystem canonicalization calls until native filesystem canonicalization, symlink/path policy, warning/false recovery, include_path/open_basedir/stat cache, non-UTF-8 path handling, references/COW, and exact native realpath diagnostics exist; phpc run handles current bounded realpath behavior";
const LLVM_IS_WRITABLE_REJECTION: &str = "LLVM is_writable lowering rejects direct filesystem writability checks until native writability checks, permission policy, warnings, include_path/open_basedir, stream wrappers, symlink/stat-cache/TOCTOU behavior, non-UTF-8 paths, references/COW, and exact native is_writable diagnostics exist; phpc run handles current bounded is_writable behavior";
const ASSEMBLY_IS_WRITABLE_REJECTION: &str = "assembly is_writable lowering rejects direct filesystem writability checks until native writability checks, permission policy, warnings, include_path/open_basedir, stream wrappers, symlink/stat-cache/TOCTOU behavior, non-UTF-8 paths, references/COW, and exact native is_writable diagnostics exist; phpc run handles current bounded is_writable behavior";
const LLVM_CLEARSTATCACHE_REJECTION: &str = "LLVM clearstatcache lowering rejects stat-cache mutation until native filesystem metadata caches, realpath cache state, per-path invalidation, include_path/open_basedir policy, stream wrappers, request-local filesystem state, references/COW, and exact native diagnostics exist; phpc run handles current bounded stat-cache clearstatcache behavior";
const ASSEMBLY_CLEARSTATCACHE_REJECTION: &str = "assembly clearstatcache lowering rejects stat-cache mutation until native filesystem metadata caches, realpath cache state, per-path invalidation, include_path/open_basedir policy, stream wrappers, request-local filesystem state, references/COW, and exact native diagnostics exist; phpc run handles current bounded stat-cache clearstatcache behavior";
const LLVM_FILESYSTEM_PATH_OPERATION_REJECTION: &str = "LLVM filesystem-path builtin lowering rejects realpath_cache_get() and realpath_cache_size() until native filesystem realpath-cache ABI, request-local cache state, binary path byte fidelity, policy checks, warning-plus-false recovery, references/copy-on-write, and exact native diagnostics exist; generated-native C routes realpath-cache introspection through the shared runtime blocker";
const ASSEMBLY_FILESYSTEM_PATH_OPERATION_REJECTION: &str = "assembly filesystem-path builtin lowering rejects forms outside the reusable native filesystem path operation blocker, including unsupported arity, stream contexts, file_get_contents() offset/length forms, non-lowerable operands, filesystem policy, stat cache/current-directory state, realpath-cache introspection return ownership, references/copy-on-write, and exact native diagnostics; lowerable stream, canonicalization, stat-predicate, stat-value, current-directory, stat-cache, and realpath-cache operands route through byte-preserving value-to-string conversion, optional truthiness, diagnostics, and cleanup";
const LLVM_DYNAMIC_FUNCTION_CALL_REJECTION: &str = "LLVM dynamic function-call lowering rejects variable-call expressions such as $name(...) until native callable expression evaluation, runtime function lookup, stack frames, arity/type diagnostics, callback dispatch, and exact native callable errors exist; phpc run handles current string-valued dynamic function calls";
const ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION: &str = "assembly dynamic function-call lowering rejects variable-call expressions such as $name(...) until native callable expression evaluation, runtime function lookup, stack frames, arity/type diagnostics, callback dispatch, and exact native callable errors exist; phpc run handles current string-valued dynamic function calls";
const LLVM_TERMINATION_REJECTION: &str = "LLVM termination lowering rejects exit()/die() until native termination control flow, exit status/stdout handoff, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die behavior";
const ASSEMBLY_TERMINATION_REJECTION: &str = "assembly termination lowering rejects exit()/die() until native termination control flow, exit status/stdout handoff, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die behavior";
const LLVM_FUNCTION_DECLARATION_REJECTION: &str = "LLVM user-function lowering rejects function declarations and return statements until native function symbol tables, stack-frame layout, default parameter binding, recursion guards, return-value flow, and exact native error behavior exist; phpc run handles current user-function declaration and return behavior";
const ASSEMBLY_FUNCTION_DECLARATION_REJECTION: &str = "assembly user-function lowering rejects function declarations and return statements until native function symbol tables, stack-frame layout, default parameter binding, recursion guards, return-value flow, and exact native error behavior exist; phpc run handles current user-function declaration and return behavior";
const LLVM_STATIC_LOCAL_REJECTION: &str = "LLVM static-local lowering rejects static local declarations until native persistent per-function storage, initialization ordering, local scope interaction, references/copy-on-write, recursion, and exact native diagnostics exist; phpc run handles current bounded static local behavior";
const ASSEMBLY_STATIC_LOCAL_REJECTION: &str = "assembly static-local lowering rejects static local declarations until native persistent per-function storage, initialization ordering, local scope interaction, references/copy-on-write, recursion, and exact native diagnostics exist; phpc run handles current bounded static local behavior";
const LLVM_CLOSURE_REJECTION: &str = "LLVM closure lowering rejects anonymous closures, arrow functions, closure captures, implicit arrow captures, closure values and invocation, callback integration, references/copy-on-write, and exact native callable errors until native closure objects and call dispatch exist; phpc run handles current closure parse/runtime boundary";
const ASSEMBLY_CLOSURE_REJECTION: &str = "assembly closure lowering rejects anonymous closures, arrow functions, closure captures, implicit arrow captures, closure values and invocation, callback integration, references/copy-on-write, and exact native callable errors until native closure objects and call dispatch exist; phpc run handles current closure parse/runtime boundary";
const LLVM_REQUIRE_REJECTION: &str = "LLVM include/require lowering rejects multi-file execution until native source loading, path resolution, declaration registration, stack/source mapping, and exact native error behavior exist; phpc run handles the current narrow include/require behavior";
const ASSEMBLY_REQUIRE_REJECTION: &str = "assembly include/require lowering rejects multi-file execution until native source loading, path resolution, declaration registration, stack/source mapping, and exact native error behavior exist; phpc run handles the current narrow include/require behavior";
const LLVM_REQUIRE_EXPRESSION_REJECTION: &str = "LLVM include/require lowering rejects multi-file execution for expression forms with include return values, _once de-duplication results, and caller-scope side effects until native source loading, path resolution, declaration registration, stack/source mapping, and exact native error behavior exist; phpc run handles current include/require expression behavior";
const ASSEMBLY_REQUIRE_EXPRESSION_REJECTION: &str = "assembly include/require lowering rejects multi-file execution for expression forms with include return values, _once de-duplication results, and caller-scope side effects until native source loading, path resolution, declaration registration, stack/source mapping, and exact native error behavior exist; phpc run handles current include/require expression behavior";
const LLVM_MAGIC_CONSTANT_REJECTION: &str = "LLVM magic-constant lowering rejects executable magic constants __LINE__, __FILE__, __DIR__, __FUNCTION__, __CLASS__, and __METHOD__ until native source mapping, path canonicalization, and function/class/method-context lowering exist; phpc run handles current magic constant behavior";
const ASSEMBLY_MAGIC_CONSTANT_REJECTION: &str = "assembly magic-constant lowering rejects executable magic constants __LINE__, __FILE__, __DIR__, __FUNCTION__, __CLASS__, and __METHOD__ until native source mapping, path canonicalization, and function/class/method-context lowering exist; phpc run handles current magic constant behavior";
const LLVM_GLOBAL_CONSTANT_REJECTION: &str = "LLVM global-constant lowering rejects built-in constant values, runtime-defined constants, bare constant reads, top-level const declarations, define()/constant(), and unsupported defined() forms until native constant tables, source-order definitions, namespace-aware lookup, and exact native error behavior exist; phpc run handles current global constant behavior";
const ASSEMBLY_GLOBAL_CONSTANT_REJECTION: &str = "assembly global-constant lowering rejects built-in constant values, runtime-defined constants, bare constant reads, top-level const declarations, define()/constant(), and unsupported defined() forms until native constant tables, source-order definitions, namespace-aware lookup, and exact native error behavior exist; phpc run handles current global constant behavior";
const LLVM_GLOBAL_DECLARATION_REJECTION: &str = "LLVM global-declaration lowering rejects global declarations until native root symbol-table imports, local/global aliasing, $GLOBALS interactions, references/copy-on-write, included-file scope interactions, and exact native diagnostics exist; phpc run handles current bounded global declaration behavior";
const ASSEMBLY_GLOBAL_DECLARATION_REJECTION: &str = "assembly global-declaration lowering rejects global declarations until native root symbol-table imports, local/global aliasing, $GLOBALS interactions, references/copy-on-write, included-file scope interactions, and exact native diagnostics exist; phpc run handles current bounded global declaration behavior";
const LLVM_OBJECT_CLASS_REJECTION: &str = "LLVM object/class lowering rejects class declarations, inheritance metadata, object instantiation, constructor dispatch, public property reads/writes, instance method calls, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";
const ASSEMBLY_OBJECT_CLASS_REJECTION: &str = "assembly object/class lowering rejects class declarations, inheritance metadata, object instantiation, constructor dispatch, public property reads/writes, instance method calls, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";
const LLVM_OBJECT_INSTANTIATION_REJECTION: &str = "LLVM object-instantiation lowering rejects new expressions and constructor dispatch until native object allocation, object handles, constructor calls, visibility checks, autoload/class lookup, references/copy-on-write, and exact native object-instantiation errors exist; phpc run handles current bounded new behavior";
const ASSEMBLY_OBJECT_INSTANTIATION_REJECTION: &str = "assembly object-instantiation lowering rejects new expressions and constructor dispatch until native object allocation, object handles, constructor calls, visibility checks, autoload/class lookup, references/copy-on-write, and exact native object-instantiation errors exist; phpc run handles current bounded new behavior";
const LLVM_OBJECT_PROPERTY_REJECTION: &str = "LLVM object-property lowering rejects instance property reads/writes and dynamic property-name access until native object layout, property tables/slots, visibility checks, magic property hooks, dynamic property policy, references/copy-on-write, and exact native object-property errors exist; phpc run handles current bounded object-property behavior";
const ASSEMBLY_OBJECT_PROPERTY_REJECTION: &str = "assembly object-property lowering rejects instance property reads/writes and dynamic property-name access until native object layout, property tables/slots, visibility checks, magic property hooks, dynamic property policy, references/copy-on-write, and exact native object-property errors exist; phpc run handles current bounded object-property behavior";
const LLVM_OBJECT_METADATA_REJECTION: &str = "LLVM object-metadata lowering rejects object/class metadata builtins until native class metadata tables, object handles, inheritance/interface/trait/enum registries, property/method tables, autoload interaction, references/copy-on-write, and exact native object-metadata errors exist; phpc run handles current bounded object metadata behavior";
const ASSEMBLY_OBJECT_METADATA_REJECTION: &str = "assembly object-metadata lowering rejects object/class metadata builtins until native class metadata tables, object handles, inheritance/interface/trait/enum registries, property/method tables, autoload interaction, references/copy-on-write, and exact native object-metadata errors exist; phpc run handles current bounded object metadata behavior";
const LLVM_INSTANCEOF_REJECTION: &str = "LLVM instanceof lowering rejects class/interface relationship checks until native class metadata tables, object handles, inheritance/interface registries, class-name resolution, autoload interaction, references/copy-on-write, and exact native instanceof diagnostics exist; phpc run handles current bounded instanceof behavior";
const ASSEMBLY_INSTANCEOF_REJECTION: &str = "assembly instanceof lowering rejects class/interface relationship checks until native class metadata tables, object handles, inheritance/interface registries, class-name resolution, autoload interaction, references/copy-on-write, and exact native instanceof diagnostics exist; phpc run handles current bounded instanceof behavior";
const LLVM_CLASS_NAME_CONSTANT_REJECTION: &str = "LLVM class-name constant lowering rejects ClassName::class, self::class, parent::class, and static::class until native class-name resolution, active class/parent and late-static-binding context, namespace/import canonicalization, autoload-free class lookup interaction, references/copy-on-write, and exact native class-name constant diagnostics exist; phpc run handles current bounded class-name constant behavior";
const ASSEMBLY_CLASS_NAME_CONSTANT_REJECTION: &str = "assembly class-name constant lowering rejects ClassName::class, self::class, parent::class, and static::class until native class-name resolution, active class/parent and late-static-binding context, namespace/import canonicalization, autoload-free class lookup interaction, references/copy-on-write, and exact native class-name constant diagnostics exist; phpc run handles current bounded class-name constant behavior";
const LLVM_STATIC_MEMBER_REJECTION: &str = "LLVM static-member lowering rejects class constants, static property reads/writes, and dynamic static-property receivers until native class constant tables, static property storage, class context and late-static-binding resolution, visibility checks, autoload/class lookup, references/copy-on-write, and exact native static-member errors exist; phpc run handles current bounded static-member behavior";
const ASSEMBLY_STATIC_MEMBER_REJECTION: &str = "assembly static-member lowering rejects class constants, static property reads/writes, and dynamic static-property receivers until native class constant tables, static property storage, class context and late-static-binding resolution, visibility checks, autoload/class lookup, references/copy-on-write, and exact native static-member errors exist; phpc run handles current bounded static-member behavior";
const LLVM_METHOD_CALL_REJECTION: &str = "LLVM method-call lowering rejects instance, named static, object static-receiver, self::, parent::, and static:: method calls until native method lookup, receiver/static receiver resolution, $this and late-static-binding context, argument/arity diagnostics, visibility checks, references/copy-on-write, and exact native method-call errors exist; phpc run handles current bounded method-call behavior";
const ASSEMBLY_METHOD_CALL_REJECTION: &str = "assembly method-call lowering rejects instance, named static, object static-receiver, self::, parent::, and static:: method calls until native method lookup, receiver/static receiver resolution, $this and late-static-binding context, argument/arity diagnostics, visibility checks, references/copy-on-write, and exact native method-call errors exist; phpc run handles current bounded method-call behavior";
const LLVM_CLONE_REJECTION: &str = "LLVM clone lowering rejects clone expressions, including direct-variable clone assignments that mirror public and context-aware non-public property reference slots, until native object handles, property slot cloning, __clone dispatch, reference-slot metadata, references/copy-on-write, and exact native error behavior exist; phpc run handles current bounded clone behavior";
const ASSEMBLY_CLONE_REJECTION: &str = "assembly clone lowering rejects clone expressions, including direct-variable clone assignments that mirror public and context-aware non-public property reference slots, until native object handles, property slot cloning, __clone dispatch, reference-slot metadata, references/copy-on-write, and exact native error behavior exist; phpc run handles current bounded clone behavior";
const LLVM_INTERFACE_REJECTION: &str = "LLVM interface lowering rejects interface declarations until native class/interface tables, implementation checks, relationship queries, autoload interaction, and exact native error behavior exist; phpc run handles current interface metadata behavior";
const ASSEMBLY_INTERFACE_REJECTION: &str = "assembly interface lowering rejects interface declarations until native class/interface tables, implementation checks, relationship queries, autoload interaction, and exact native error behavior exist; phpc run handles current interface metadata behavior";
const LLVM_TRAIT_REJECTION: &str = "LLVM trait lowering rejects trait declarations until native trait tables, class trait-use composition, conflict resolution, aliasing, relationship metadata, autoload interaction, and exact native error behavior exist; phpc run handles current trait metadata behavior";
const ASSEMBLY_TRAIT_REJECTION: &str = "assembly trait lowering rejects trait declarations until native trait tables, class trait-use composition, conflict resolution, aliasing, relationship metadata, autoload interaction, and exact native error behavior exist; phpc run handles current trait metadata behavior";
const LLVM_ENUM_REJECTION: &str = "LLVM enum lowering rejects enum declarations until native class/enum tables, enum case objects, backed enum values, interface implementation, relationship queries, autoload interaction, and exact native error behavior exist; phpc run handles current enum metadata behavior";
const ASSEMBLY_ENUM_REJECTION: &str = "assembly enum lowering rejects enum declarations until native class/enum tables, enum case objects, backed enum values, interface implementation, relationship queries, autoload interaction, and exact native error behavior exist; phpc run handles current enum metadata behavior";
const LLVM_NAMESPACE_REJECTION: &str = "LLVM namespace lowering rejects namespace declarations, namespace-qualified names, namespace imports, and namespace-aware name resolution until native symbol tables, namespace context, aliases/imports, fallback function/constant lookup, class/autoload lookup, and exact native error behavior exist; phpc run handles current namespace behavior";
const ASSEMBLY_NAMESPACE_REJECTION: &str = "assembly namespace lowering rejects namespace declarations, namespace-qualified names, namespace imports, and namespace-aware name resolution until native symbol tables, namespace context, aliases/imports, fallback function/constant lookup, class/autoload lookup, and exact native error behavior exist; phpc run handles current namespace behavior";
const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";
const ASSEMBLY_ARRAY_REJECTION: &str = "assembly array lowering rejects unsupported arrays, unsupported array indexing forms, unsupported array assignment forms, unsupported foreach array iteration forms, unsupported array offset unset forms, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; generated-native C routes lowerable direct array offset writes, appends, unsets, and by-value foreach over tracked native array owners through shared native ABIs";
const ASSEMBLY_NATIVE_ARRAY_BY_REFERENCE_FOREACH_REJECTION: &str = "native executable by-reference foreach lowering rejects by-reference iteration until generated C has generalized reference-slot symbol storage, foreach cursor reference binding, owner/path value-reference acquisition through phpc_native_array_lvalue_owner_foreach_value_reference_result(), and loop-body cleanup ownership for array, nested-array, and value-root iterable owners; phpc run handles current by-reference foreach behavior";
const LLVM_ARRAY_ACCESS_REJECTION: &str = "LLVM ArrayAccess lowering rejects object offset reads/writes/isset/empty/unset/compound paths until native ArrayAccess dispatch for offsetGet(), offsetSet(), offsetExists(), and offsetUnset(), object handles, references/copy-on-write, and exact PHP diagnostics exist; phpc run handles current bounded ArrayAccess behavior";
const ASSEMBLY_ARRAY_ACCESS_REJECTION: &str = "assembly ArrayAccess lowering rejects object offset reads/writes/isset/empty/unset/compound paths until native ArrayAccess dispatch for offsetGet(), offsetSet(), offsetExists(), and offsetUnset(), object handles, references/copy-on-write, and exact PHP diagnostics exist; phpc run handles current bounded ArrayAccess behavior";
const LLVM_ARRAY_DESTRUCTURING_REJECTION: &str = "LLVM array destructuring lowering rejects list(...) and [...] assignment targets until native array storage layout, ordered key lookup, missing-key diagnostics, nested destructuring, references/copy-on-write, and exact native assignment ordering exist; phpc run handles current simple destructuring assignment behavior";
const ASSEMBLY_ARRAY_DESTRUCTURING_REJECTION: &str = "assembly array destructuring lowering rejects list(...) and [...] assignment targets until native array storage layout, ordered key lookup, missing-key diagnostics, nested destructuring, references/copy-on-write, and exact native assignment ordering exist; phpc run handles current simple destructuring assignment behavior";
const LLVM_CONTROL_FLOW_REJECTION: &str = "LLVM control-flow lowering rejects if/else and elseif chains, while loops, for loops, do-while loops, switch statements, goto labels, break, and continue until native PHP truthiness, branch layout, loop control flow, switch fallthrough, goto jumps, references/copy-on-write side effects, and exact native error behavior exist; phpc run handles current control-flow behavior";
const ASSEMBLY_CONTROL_FLOW_REJECTION: &str = "assembly control-flow lowering rejects if/else and elseif chains, while loops, for loops, do-while loops, switch statements, goto labels, break, and continue until native PHP truthiness, branch layout, loop control flow, switch fallthrough, goto jumps, references/copy-on-write side effects, and exact native error behavior exist; phpc run handles current control-flow behavior";
const LLVM_EXCEPTION_REJECTION: &str = "LLVM exception lowering rejects throw statements and try/catch/finally blocks until native Throwable objects, stack unwinding, catch/finally dispatch, stack traces, and exact native error behavior exist; phpc run handles the current exception boundary";
const ASSEMBLY_EXCEPTION_REJECTION: &str = "assembly exception lowering rejects throw statements and try/catch/finally blocks until native Throwable objects, stack unwinding, catch/finally dispatch, stack traces, and exact native error behavior exist; phpc run handles the current exception boundary";
const LLVM_TRY_BLOCK_REJECTION: &str = "LLVM try/catch/finally lowering rejects try blocks until native Throwable objects, stack unwinding, catch type matching, catch variable binding, finally execution during normal and exceptional control flow, stack traces, references/copy-on-write, and exact native try-block diagnostics exist; phpc run handles current bounded no-throw try/catch/finally behavior";
const ASSEMBLY_TRY_BLOCK_REJECTION: &str = "assembly try/catch/finally lowering rejects try blocks until native Throwable objects, stack unwinding, catch type matching, catch variable binding, finally execution during normal and exceptional control flow, stack traces, references/copy-on-write, and exact native try-block diagnostics exist; phpc run handles current bounded no-throw try/catch/finally behavior";
const LLVM_REFERENCE_ASSIGNMENT_REJECTION: &str = "LLVM reference-assignment lowering rejects direct variable, array-offset, object-property, function-call, method-call, static-call, magic __get, and ArrayAccess reference sources or targets until native reference containers, alias-aware symbol tables, copy-on-write, object/property alias roots, and exact native error behavior exist; phpc run handles current bounded reference-assignment behavior";
const ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION: &str = "assembly reference-assignment lowering rejects direct variable, array-offset, object-property, function-call, method-call, static-call, magic __get, and ArrayAccess reference sources or targets until native reference containers, alias-aware symbol tables, copy-on-write, object/property alias roots, and exact native error behavior exist; phpc run handles current bounded reference-assignment behavior";
const LLVM_MUTATION_REJECTION: &str = "LLVM mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, assignment expressions, direct variable unset, object property unset, static property unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";
const ASSEMBLY_MUTATION_REJECTION: &str = "assembly mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, assignment expressions outside lowerable direct and nested array offset write/append values, direct variable unset, object property unset, static property unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";
const LLVM_ISSET_REJECTION: &str = "LLVM isset lowering rejects array offset operands, object property operands, static property operands, complex operands, multiple operands, and unset/mutation interactions until native symbol-table storage, null-aware lookup, references/copy-on-write, and exact native error behavior exist; phpc run handles current isset behavior";
const ASSEMBLY_ISSET_REJECTION: &str = "assembly isset lowering rejects array offset operands, object property operands, complex operands, multiple operands, and unset/mutation interactions until native symbol-table storage, null-aware lookup, references/copy-on-write, and exact native error behavior exist; phpc run handles current isset behavior";
const LLVM_EMPTY_REJECTION: &str = "LLVM empty lowering rejects array offset operands, object property operands, static property operands, complex operands, arrays, unset/mutation interactions, and ambiguous truthiness until native symbol-table storage, PHP truthiness, references/copy-on-write, and exact native error behavior exist; phpc run handles current empty behavior";
const ASSEMBLY_EMPTY_REJECTION: &str = "assembly empty lowering rejects array offset operands, object property operands, complex operands, arrays, unset/mutation interactions, and ambiguous truthiness until native symbol-table storage, PHP truthiness, references/copy-on-write, and exact native error behavior exist; phpc run handles current empty behavior";
const LLVM_ERROR_CONTROL_REJECTION: &str = "LLVM error-control lowering rejects @expr until native diagnostic severity, warning/notice/deprecation suppression, error_reporting() mask interaction, recoverable expression values, and exact native diagnostics exist; phpc run handles current transparent error-control wrapper behavior";
const ASSEMBLY_ERROR_CONTROL_REJECTION: &str = "assembly error-control lowering rejects @expr until native diagnostic severity, warning/notice/deprecation suppression, error_reporting() mask interaction, recoverable expression values, and exact native diagnostics exist; phpc run handles current transparent error-control wrapper behavior";
const LLVM_CAST_REJECTION: &str = "LLVM cast lowering rejects (string), (int)/(integer), (bool)/(boolean), (float)/(double), and (array) casts plus strval(), boolval(), floatval(), and doubleval() until native PHP scalar conversion, array materialization, warning/recovery behavior, object/resource handling, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded cast behavior";
const ASSEMBLY_CAST_REJECTION: &str = "assembly cast lowering rejects (string), (int)/(integer), (bool)/(boolean), (float)/(double), and (array) casts plus strval(), boolval(), floatval(), and doubleval() until native PHP scalar conversion, array materialization, warning/recovery behavior, object/resource handling, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded cast behavior";
const LLVM_UNARY_REJECTION: &str = "LLVM unary lowering rejects unsupported unary operators, cast expressions, or operands until native PHP numeric coercion, truthiness conversion, scalar casts, overflow behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current unary and cast behavior";
const ASSEMBLY_UNARY_REJECTION: &str = "assembly unary lowering rejects unsupported unary operators, cast expressions, or operands until native PHP numeric coercion, truthiness conversion, scalar casts, overflow behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current unary and cast behavior";
const LLVM_ARITHMETIC_REJECTION: &str = "LLVM arithmetic lowering rejects unsupported binary arithmetic operators or operands until native PHP numeric coercion, division/modulo zero checks, modulo coercions, references/copy-on-write, and exact native error behavior exist; phpc run handles current arithmetic behavior";
const ASSEMBLY_ARITHMETIC_REJECTION: &str = "assembly arithmetic lowering rejects unsupported binary arithmetic operators or operands until native PHP numeric coercion, division/modulo zero checks, modulo coercions, references/copy-on-write, and exact native error behavior exist; phpc run handles current arithmetic behavior";
const LLVM_MIXED_NUMERIC_ARITHMETIC_REJECTION: &str = "LLVM mixed numeric arithmetic lowering rejects int/float operands until native PHP numeric promotion, result typing, overflow/INF/NAN behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current mixed numeric arithmetic behavior";
const ASSEMBLY_MIXED_NUMERIC_ARITHMETIC_REJECTION: &str = "assembly mixed numeric arithmetic lowering rejects int/float operands until native PHP numeric promotion, result typing, overflow/INF/NAN behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current mixed numeric arithmetic behavior";
const LLVM_SCALAR_COERCION_ARITHMETIC_REJECTION: &str = "LLVM scalar-coercion arithmetic lowering rejects booleans, nulls, and strings in +, -, and * until native PHP numeric coercion, string numeric parsing, warnings/recovery behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current scalar-coercion arithmetic behavior";
const ASSEMBLY_SCALAR_COERCION_ARITHMETIC_REJECTION: &str = "assembly scalar-coercion arithmetic lowering rejects booleans, nulls, and strings in +, -, and * until native PHP numeric coercion, string numeric parsing, warnings/recovery behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current scalar-coercion arithmetic behavior";
const LLVM_INTEGER_OVERFLOW_ARITHMETIC_REJECTION: &str = "LLVM integer arithmetic lowering rejects overflow-sensitive or not-statically-proven integer +, -, and * until native PHP integer overflow promotion, runtime checks, references/copy-on-write, and exact native error behavior exist; phpc run handles current integer overflow arithmetic behavior";
const ASSEMBLY_INTEGER_OVERFLOW_ARITHMETIC_REJECTION: &str = "assembly integer arithmetic lowering rejects overflow-sensitive or not-statically-proven integer +, -, and * until native PHP integer overflow promotion, runtime checks, references/copy-on-write, and exact native error behavior exist; phpc run handles current integer overflow arithmetic behavior";
const LLVM_DIVISION_REJECTION: &str = "LLVM division lowering rejects / until native PHP division semantics, zero-divisor runtime checks, avoidance of misleading integer truncation, overflow/INF/NAN behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current division behavior";
const ASSEMBLY_DIVISION_REJECTION: &str = "assembly division lowering rejects / until native PHP division semantics, zero-divisor runtime checks, avoidance of misleading integer truncation, overflow/INF/NAN behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current division behavior";
const LLVM_MODULO_RUNTIME_CHECK_REJECTION: &str = "LLVM modulo lowering rejects dynamic, zero, or non-positive integer divisors until native modulo runtime checks, PHP modulo diagnostics, negative-divisor/min-int edge behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current modulo behavior";
const ASSEMBLY_MODULO_RUNTIME_CHECK_REJECTION: &str = "assembly modulo lowering rejects dynamic, zero, or non-positive integer divisors until native modulo runtime checks, PHP modulo diagnostics, negative-divisor/min-int edge behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current modulo behavior";
const LLVM_CONCAT_REJECTION: &str = "LLVM concatenation lowering rejects unsupported concatenation operands until native PHP scalar-to-string conversion, dynamic allocation, references/copy-on-write, and exact native error behavior exist; phpc run handles current concatenation behavior";
const ASSEMBLY_CONCAT_REJECTION: &str = "assembly concatenation lowering rejects unsupported concatenation operands until native PHP scalar-to-string conversion, dynamic allocation, references/copy-on-write, and exact native error behavior exist; phpc run handles current concatenation behavior";
const LLVM_INTERPOLATED_STRING_REJECTION: &str = "LLVM interpolated-string lowering rejects double-quoted string interpolation until native interpolation part evaluation, PHP-shaped string conversion, array/object lookup, __toString dispatch, runtime string allocation, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded interpolation behavior";
const ASSEMBLY_INTERPOLATED_STRING_REJECTION: &str = "assembly interpolated-string lowering rejects double-quoted string interpolation until native interpolation part evaluation, PHP-shaped string conversion, array/object lookup, __toString dispatch, runtime string allocation, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded interpolation behavior";
const LLVM_BITWISE_REJECTION: &str = "LLVM bitwise lowering rejects unsupported bitwise or shift operators or operands until native PHP bitwise string semantics, scalar-to-int coercion, shift diagnostics, references/copy-on-write, and exact native error behavior exist; phpc run handles current bitwise/shift behavior";
const ASSEMBLY_BITWISE_REJECTION: &str = "assembly bitwise lowering rejects unsupported bitwise or shift operators or operands until native PHP bitwise string semantics, scalar-to-int coercion, shift diagnostics, references/copy-on-write, and exact native error behavior exist; phpc run handles current bitwise/shift behavior";
const LLVM_VARIABLE_READ_REJECTION: &str = "LLVM variable-read lowering rejects reads that are not statically assigned earlier in the same straight-line native subset until native symbol-table storage, undefined-variable diagnostics, references/copy-on-write, and exact native error behavior exist; phpc run handles current variable-read behavior";
const ASSEMBLY_VARIABLE_READ_REJECTION: &str = "assembly variable-read lowering rejects reads that are not statically assigned earlier in the same straight-line native subset until native symbol-table storage, undefined-variable diagnostics, references/copy-on-write, and exact native error behavior exist; phpc run handles current variable-read behavior";
const LLVM_REQUEST_SUPERGLOBAL_REJECTION: &str = "LLVM request-superglobal lowering rejects $_SERVER, $_COOKIE, $_GET, $_POST, $_REQUEST, $_FILES, and $_SESSION until native request-state storage, SAPI population, variables_order policy, upload metadata, session storage, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded request superglobal behavior";
const ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION: &str = "assembly request-superglobal lowering rejects $_SERVER, $_COOKIE, $_GET, $_POST, $_REQUEST, $_FILES, and $_SESSION until native request-state storage, SAPI population, variables_order policy, upload metadata, session storage, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded request superglobal behavior";
const LLVM_HEADER_STATE_REJECTION: &str = "LLVM header-state lowering rejects header(), header_remove(), headers_list(), headers_sent(), http_response_code(), setcookie(), and setrawcookie() until native response-header storage, output-started tracking, status-code handling, cookie formatting, SAPI emission, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded CLI header-state behavior";
const ASSEMBLY_HEADER_STATE_REJECTION: &str = "assembly header-state lowering rejects header(), header_remove(), headers_list(), headers_sent(), http_response_code(), setcookie(), and setrawcookie() until native response-header storage, output-started tracking, status-code handling, cookie formatting, SAPI emission, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded CLI header-state behavior";
const LLVM_SESSION_STATE_REJECTION: &str = "LLVM session-state lowering rejects $_SESSION and session_start(), session_status(), session_cache_limiter(), session_cache_expire(), session_id(), and session_write_close() until native request/session storage, session id persistence, cache limiter state, locking, cookie/header emission, save handlers, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded CLI session-state behavior";
const ASSEMBLY_SESSION_STATE_REJECTION: &str = "assembly session-state lowering rejects $_SESSION and session_start(), session_status(), session_cache_limiter(), session_cache_expire(), session_id(), and session_write_close() until native request/session storage, session id persistence, cache limiter state, locking, cookie/header emission, save handlers, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded CLI session-state behavior";
const LLVM_OUTPUT_BUFFER_REJECTION: &str = "LLVM output-buffer lowering rejects ob_start(), ob_get_level(), ob_get_contents(), ob_get_length(), ob_list_handlers(), ob_get_status(), ob_get_clean(), ob_get_flush(), ob_clean(), ob_flush(), ob_end_clean(), and ob_end_flush() until native stdout capture buffers, shutdown flushing, output-started tracking, SAPI interaction, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded output-buffer behavior";
const ASSEMBLY_OUTPUT_BUFFER_REJECTION: &str = "assembly output-buffer lowering rejects ob_start(), ob_get_level(), ob_get_contents(), ob_get_length(), ob_list_handlers(), ob_get_status(), ob_get_clean(), ob_get_flush(), ob_clean(), ob_flush(), ob_end_clean(), and ob_end_flush() until native stdout capture buffers, shutdown flushing, output-started tracking, SAPI interaction, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded output-buffer behavior";

const NATIVE_VALUE_OFFSET_MUTATION_WRITE: u8 = 0;
const NATIVE_VALUE_OFFSET_MUTATION_APPEND: u8 = 1;
const NATIVE_ARRAY_PATH_KEY_TAG: u8 = 0;
const NATIVE_ARRAY_PATH_APPEND_TAG: u8 = 1;
const NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE_TAG: u8 = 0;
const NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UNSET_TAG: u8 = 1;
const NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ_TAG: u8 = 2;
const NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UPDATE_TAG: u8 = 3;
const NATIVE_ARRAY_LVALUE_VALUE_OPERATION_ISSET_TAG: u8 = 4;
const NATIVE_ARRAY_LVALUE_VALUE_RESULT_INCREMENT_DECREMENT_TAG: u8 = 0;
const NATIVE_ARRAY_LVALUE_INCREMENT_TAG: u8 = 0;
const NATIVE_ARRAY_LVALUE_DECREMENT_TAG: u8 = 1;
const NATIVE_ARRAY_LVALUE_POSITION_PRE_TAG: u8 = 0;
const NATIVE_ARRAY_LVALUE_POSITION_POST_TAG: u8 = 1;
const NATIVE_ARRAY_LVALUE_POINTER_CURRENT_TAG: u8 = 0;
const NATIVE_ARRAY_LVALUE_POINTER_KEY_TAG: u8 = 1;
const NATIVE_ARRAY_LVALUE_POINTER_NEXT_TAG: u8 = 2;
const NATIVE_ARRAY_LVALUE_POINTER_PREV_TAG: u8 = 3;
const NATIVE_ARRAY_LVALUE_POINTER_RESET_TAG: u8 = 4;
const NATIVE_ARRAY_LVALUE_POINTER_END_TAG: u8 = 5;
const NATIVE_ARRAY_LVALUE_SORT_SORT_TAG: u8 = 0;
const NATIVE_ARRAY_LVALUE_SORT_RSORT_TAG: u8 = 1;
const NATIVE_ARRAY_LVALUE_SORT_ASORT_TAG: u8 = 2;
const NATIVE_ARRAY_LVALUE_SORT_ARSORT_TAG: u8 = 3;
const NATIVE_ARRAY_LVALUE_SORT_KSORT_TAG: u8 = 4;
const NATIVE_ARRAY_LVALUE_SORT_KRSORT_TAG: u8 = 5;
const NATIVE_ARRAY_LVALUE_SORT_NATSORT_TAG: u8 = 6;
const NATIVE_ARRAY_LVALUE_SORT_NATCASESORT_TAG: u8 = 7;
const NATIVE_ARRAY_LVALUE_SORT_USORT_TAG: u8 = 8;
const NATIVE_ARRAY_LVALUE_SORT_UASORT_TAG: u8 = 9;
const NATIVE_ARRAY_LVALUE_SORT_UKSORT_TAG: u8 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeCallBackend {
    Llvm,
    Assembly,
}

impl NativeCallBackend {
    fn function_call_rejection(self) -> &'static str {
        match self {
            Self::Llvm => LLVM_FUNCTION_CALL_REJECTION,
            Self::Assembly => ASSEMBLY_FUNCTION_CALL_REJECTION,
        }
    }

    fn dynamic_function_call_rejection(self) -> &'static str {
        match self {
            Self::Llvm => LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
            Self::Assembly => ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        }
    }

    fn function_declaration_rejection(self) -> &'static str {
        match self {
            Self::Llvm => LLVM_FUNCTION_DECLARATION_REJECTION,
            Self::Assembly => ASSEMBLY_FUNCTION_DECLARATION_REJECTION,
        }
    }

    fn closure_rejection(self) -> &'static str {
        match self {
            Self::Llvm => LLVM_CLOSURE_REJECTION,
            Self::Assembly => ASSEMBLY_CLOSURE_REJECTION,
        }
    }

    fn method_call_rejection(self) -> &'static str {
        match self {
            Self::Llvm => LLVM_METHOD_CALL_REJECTION,
            Self::Assembly => ASSEMBLY_METHOD_CALL_REJECTION,
        }
    }

    fn object_instantiation_rejection(self) -> &'static str {
        match self {
            Self::Llvm => LLVM_OBJECT_INSTANTIATION_REJECTION,
            Self::Assembly => ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        }
    }

    fn reference_assignment_rejection(self) -> &'static str {
        match self {
            Self::Llvm => LLVM_REFERENCE_ASSIGNMENT_REJECTION,
            Self::Assembly => ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeCallCallee {
    DirectNamed,
    DynamicExpression,
    MethodDispatch,
    ConstructorDispatch,
    FunctionFrame,
    ClosureFrame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeCallResult {
    Value,
    FrameHandoff,
    Reference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeCallBlocker {
    DynamicCallableEvaluation,
    ArgumentEvaluationCleanup,
    StatementOperandEvaluationCleanup,
    ValueOperandEvaluationCleanup,
    LvalueOperandEvaluationCleanup,
    ReturnValueOwnership,
    ByReferenceArgumentBinding,
    VariadicArgumentBinding,
    FunctionFrameHandoff,
    ClosureFrameHandoff,
    UnknownCalleeDiagnostics,
    MethodDispatch,
    ConstructorDispatch,
    ReferenceBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NativeCallOperation {
    span: Span,
    callee: NativeCallCallee,
    result: NativeCallResult,
    blocker: NativeCallBlocker,
}

impl NativeCallOperation {
    fn value_result(span: Span, callee: NativeCallCallee, blocker: NativeCallBlocker) -> Self {
        Self {
            span,
            callee,
            result: NativeCallResult::Value,
            blocker,
        }
    }

    fn direct_named_value(span: Span, blocker: NativeCallBlocker) -> Self {
        Self::value_result(span, NativeCallCallee::DirectNamed, blocker)
    }

    fn dynamic_value(span: Span) -> Self {
        Self::dynamic_value_with_blocker(span, NativeCallBlocker::DynamicCallableEvaluation)
    }

    fn dynamic_value_with_blocker(span: Span, blocker: NativeCallBlocker) -> Self {
        Self::value_result(span, NativeCallCallee::DynamicExpression, blocker)
    }

    fn method_value(span: Span) -> Self {
        Self::method_value_with_blocker(span, NativeCallBlocker::MethodDispatch)
    }

    fn method_value_with_blocker(span: Span, blocker: NativeCallBlocker) -> Self {
        Self::value_result(span, NativeCallCallee::MethodDispatch, blocker)
    }

    fn function_frame(span: Span, blocker: NativeCallBlocker) -> Self {
        Self {
            span,
            callee: NativeCallCallee::FunctionFrame,
            result: NativeCallResult::FrameHandoff,
            blocker,
        }
    }

    fn closure_frame(span: Span, blocker: NativeCallBlocker) -> Self {
        Self {
            span,
            callee: NativeCallCallee::ClosureFrame,
            result: NativeCallResult::FrameHandoff,
            blocker,
        }
    }

    fn constructor_value(span: Span, blocker: NativeCallBlocker) -> Self {
        Self::value_result(span, NativeCallCallee::ConstructorDispatch, blocker)
    }

    fn return_value(span: Span) -> Self {
        Self {
            span,
            callee: NativeCallCallee::FunctionFrame,
            result: NativeCallResult::Value,
            blocker: NativeCallBlocker::ReturnValueOwnership,
        }
    }

    fn reference_result(span: Span, callee: NativeCallCallee) -> Self {
        Self {
            span,
            callee,
            result: NativeCallResult::Reference,
            blocker: NativeCallBlocker::ReferenceBinding,
        }
    }

    fn dereferenced_value_result(span: Span, callee: NativeCallCallee) -> Self {
        Self {
            span,
            callee,
            result: NativeCallResult::Value,
            blocker: NativeCallBlocker::ReturnValueOwnership,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum NativeCallDiagnosticSubject<'a> {
    Operation(NativeCallOperation),
    CallRoot(&'a Expr),
    #[cfg(test)]
    FunctionFrame(&'a FunctionDecl),
    ReturnStatement(Span),
}

#[derive(Debug, Clone, Copy)]
struct NativeCallDiagnostics {
    backend: NativeCallBackend,
}

impl NativeCallDiagnosticSubject<'_> {
    fn operation(self) -> NativeCallOperation {
        match self {
            Self::Operation(operation) => operation,
            Self::CallRoot(expr) => native_call_root_operation(expr)
                .expect("native call diagnostic root must be a call expression"),
            #[cfg(test)]
            Self::FunctionFrame(function) => native_function_frame_call_operation(function),
            Self::ReturnStatement(span) => NativeCallOperation::return_value(span),
        }
    }
}

impl NativeCallDiagnostics {
    fn new(backend: NativeCallBackend) -> Self {
        Self { backend }
    }

    fn operation(self, operation: NativeCallOperation) -> Diagnostic {
        self.subject(NativeCallDiagnosticSubject::Operation(operation))
    }

    fn direct_call(self, span: Span, blocker: NativeCallBlocker) -> Diagnostic {
        self.operation(NativeCallOperation::direct_named_value(span, blocker))
    }

    fn call_root(self, expr: &Expr) -> Diagnostic {
        self.subject(NativeCallDiagnosticSubject::CallRoot(expr))
    }

    fn return_statement(self, span: Span) -> Diagnostic {
        self.subject(NativeCallDiagnosticSubject::ReturnStatement(span))
    }

    fn subject(self, subject: NativeCallDiagnosticSubject<'_>) -> Diagnostic {
        native_call_operation_diagnostic(self.backend, subject.operation())
    }
}

fn native_function_frame_blocker(function: &FunctionDecl) -> NativeCallBlocker {
    native_callable_frame_blocker(&function.params, function.returns_by_reference)
}

fn native_function_frame_call_operation(function: &FunctionDecl) -> NativeCallOperation {
    NativeCallOperation::function_frame(function.span, native_function_frame_blocker(function))
}

fn native_closure_frame_blocker(
    params: &[FunctionParam],
    returns_by_reference: bool,
) -> NativeCallBlocker {
    match native_callable_frame_blocker(params, returns_by_reference) {
        NativeCallBlocker::FunctionFrameHandoff => NativeCallBlocker::ClosureFrameHandoff,
        blocker => blocker,
    }
}

fn native_callable_frame_blocker(
    params: &[FunctionParam],
    returns_by_reference: bool,
) -> NativeCallBlocker {
    if params.iter().any(|param| param.by_reference) {
        NativeCallBlocker::ByReferenceArgumentBinding
    } else if params.iter().any(|param| param.is_variadic) {
        NativeCallBlocker::VariadicArgumentBinding
    } else if returns_by_reference {
        NativeCallBlocker::ReturnValueOwnership
    } else {
        NativeCallBlocker::FunctionFrameHandoff
    }
}

fn native_call_argument_list_blocker(args: &[Expr]) -> Option<NativeCallBlocker> {
    args.iter()
        .any(native_expr_contains_call_result)
        .then_some(NativeCallBlocker::ArgumentEvaluationCleanup)
}

fn native_direct_call_argument_result_operation(
    args: &[Expr],
    span: Span,
) -> Option<NativeCallOperation> {
    native_call_argument_list_blocker(args)
        .map(|blocker| NativeCallOperation::direct_named_value(span, blocker))
}

fn native_expr_call_result_operation(
    expr: &Expr,
    blocker: NativeCallBlocker,
) -> Option<NativeCallOperation> {
    match expr {
        Expr::Call { span, .. } => Some(NativeCallOperation::direct_named_value(*span, blocker)),
        Expr::DynamicCall { span, .. } => Some(NativeCallOperation::dynamic_value_with_blocker(
            *span, blocker,
        )),
        Expr::MethodCall { span, .. }
        | Expr::DynamicMethodCall { span, .. }
        | Expr::ParentMethodCall { span, .. }
        | Expr::StaticMethodCall { span, .. }
        | Expr::ObjectStaticMethodCall { span, .. }
        | Expr::SelfMethodCall { span, .. }
        | Expr::LateStaticMethodCall { span, .. } => Some(
            NativeCallOperation::method_value_with_blocker(*span, blocker),
        ),
        Expr::New { span, .. } => Some(NativeCallOperation::constructor_value(*span, blocker)),
        Expr::Closure {
            params,
            returns_by_reference,
            span,
            ..
        } => Some(NativeCallOperation::closure_frame(
            *span,
            native_closure_frame_blocker(params, *returns_by_reference),
        )),
        Expr::Array { items, .. } => items.iter().find_map(|item| {
            item.key
                .as_ref()
                .and_then(|key| native_expr_call_result_operation(key, blocker))
                .or_else(|| native_expr_call_result_operation(&item.value, blocker))
        }),
        Expr::Index { target, index, .. } => native_expr_call_result_operation(target, blocker)
            .or_else(|| native_expr_call_result_operation(index, blocker)),
        Expr::AppendIndex { target, .. }
        | Expr::Property { target, .. }
        | Expr::ObjectStaticProperty { target, .. }
        | Expr::InstanceOf { expr: target, .. } => {
            native_expr_call_result_operation(target, blocker)
        }
        Expr::DynamicProperty {
            target, property, ..
        } => native_expr_call_result_operation(target, blocker)
            .or_else(|| native_expr_call_result_operation(property, blocker)),
        Expr::Clone { expr, .. }
        | Expr::Unary { expr, .. }
        | Expr::ErrorControl { expr, .. }
        | Expr::Include { path: expr, .. }
        | Expr::Require { path: expr, .. }
        | Expr::Cast { expr, .. } => native_expr_call_result_operation(expr, blocker),
        Expr::Binary { left, right, .. } => native_expr_call_result_operation(left, blocker)
            .or_else(|| native_expr_call_result_operation(right, blocker)),
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => native_expr_call_result_operation(condition, blocker)
            .or_else(|| native_expr_call_result_operation(if_true, blocker))
            .or_else(|| native_expr_call_result_operation(if_false, blocker)),
        Expr::ShortTernary {
            condition,
            if_false,
            ..
        } => native_expr_call_result_operation(condition, blocker)
            .or_else(|| native_expr_call_result_operation(if_false, blocker)),
        Expr::Assign { target, expr, .. }
        | Expr::CompoundAssign { target, expr, .. }
        | Expr::NullCoalesceAssign { target, expr, .. } => {
            native_assignment_target_call_operation(target)
                .or_else(|| native_expr_call_result_operation(expr, blocker))
        }
        Expr::IncrementDecrement { target, .. } => native_assignment_target_call_operation(target),
        Expr::Null(_)
        | Expr::Bool(_, _)
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::String(_, _)
        | Expr::InterpolatedString { .. }
        | Expr::Variable(_, _)
        | Expr::MagicLine { .. }
        | Expr::MagicFile { .. }
        | Expr::MagicDir { .. }
        | Expr::MagicFunction { .. }
        | Expr::MagicClass { .. }
        | Expr::MagicMethod { .. }
        | Expr::GlobalConstant { .. }
        | Expr::ClassNameConstant { .. }
        | Expr::SelfClassNameConstant { .. }
        | Expr::ParentClassNameConstant { .. }
        | Expr::StaticClassNameConstant { .. }
        | Expr::ClassConstant { .. }
        | Expr::SelfClassConstant { .. }
        | Expr::ParentClassConstant { .. }
        | Expr::LateStaticClassConstant { .. }
        | Expr::StaticProperty { .. }
        | Expr::SelfStaticProperty { .. }
        | Expr::ParentStaticProperty { .. }
        | Expr::LateStaticProperty { .. } => None,
    }
}

fn native_lvalue_operand_call_result_operation(expr: &Expr) -> Option<NativeCallOperation> {
    native_value_result_expr_call_operation(expr, NativeCallBlocker::LvalueOperandEvaluationCleanup)
}

fn native_statement_operand_call_result_operation(expr: &Expr) -> Option<NativeCallOperation> {
    native_expr_call_result_operation(expr, NativeCallBlocker::StatementOperandEvaluationCleanup)
}

fn native_value_operand_call_result_operation(expr: &Expr) -> Option<NativeCallOperation> {
    native_expr_call_result_operation(expr, NativeCallBlocker::ValueOperandEvaluationCleanup)
}

fn native_failed_value_operand_call_result_operation(expr: &Expr) -> Option<NativeCallOperation> {
    if native_expr_is_call_operation_root(expr) {
        None
    } else {
        native_value_operand_call_result_operation(expr)
    }
}

fn native_unemitted_operand_call_operation(
    expr: &Expr,
    nested_blocker: NativeCallBlocker,
) -> Option<NativeCallOperation> {
    native_value_call_operation_for_expr(expr)
        .or_else(|| native_expr_call_result_operation(expr, nested_blocker))
}

fn native_unemitted_value_operand_call_operation(expr: &Expr) -> Option<NativeCallOperation> {
    native_unemitted_operand_call_operation(expr, NativeCallBlocker::ValueOperandEvaluationCleanup)
}

fn native_unemitted_value_operand_list_call_operation(
    exprs: &[&Expr],
) -> Option<NativeCallOperation> {
    exprs
        .iter()
        .find_map(|expr| native_unemitted_value_operand_call_operation(expr))
}

fn native_unemitted_statement_operand_call_operation(expr: &Expr) -> Option<NativeCallOperation> {
    native_unemitted_operand_call_operation(
        expr,
        NativeCallBlocker::StatementOperandEvaluationCleanup,
    )
}

fn native_unemitted_statement_operand_list_call_operation(
    exprs: &[Expr],
) -> Option<NativeCallOperation> {
    exprs
        .iter()
        .find_map(native_unemitted_statement_operand_call_operation)
}

fn native_expr_is_call_operation_root(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { .. }
            | Expr::DynamicCall { .. }
            | Expr::MethodCall { .. }
            | Expr::DynamicMethodCall { .. }
            | Expr::ParentMethodCall { .. }
            | Expr::StaticMethodCall { .. }
            | Expr::ObjectStaticMethodCall { .. }
            | Expr::SelfMethodCall { .. }
            | Expr::LateStaticMethodCall { .. }
            | Expr::New { .. }
            | Expr::Closure { .. }
    )
}

fn native_value_unary_op_tag(op: UnaryOp) -> Option<&'static str> {
    match op {
        UnaryOp::Negate => Some("PHPC_NATIVE_VALUE_UNARY_NEGATE"),
        UnaryOp::BitwiseNot => Some("PHPC_NATIVE_VALUE_UNARY_BITWISE_NOT"),
        UnaryOp::Not => None,
    }
}

fn native_value_binary_op_tag(op: BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("PHPC_NATIVE_VALUE_BINARY_ADD"),
        BinaryOp::Sub => Some("PHPC_NATIVE_VALUE_BINARY_SUB"),
        BinaryOp::Mul => Some("PHPC_NATIVE_VALUE_BINARY_MUL"),
        BinaryOp::Div => Some("PHPC_NATIVE_VALUE_BINARY_DIV"),
        BinaryOp::Mod => Some("PHPC_NATIVE_VALUE_BINARY_MOD"),
        BinaryOp::Concat => Some("PHPC_NATIVE_VALUE_BINARY_CONCAT"),
        BinaryOp::BitwiseAnd => Some("PHPC_NATIVE_VALUE_BINARY_BITWISE_AND"),
        BinaryOp::BitwiseOr => Some("PHPC_NATIVE_VALUE_BINARY_BITWISE_OR"),
        BinaryOp::BitwiseXor => Some("PHPC_NATIVE_VALUE_BINARY_BITWISE_XOR"),
        BinaryOp::ShiftLeft => Some("PHPC_NATIVE_VALUE_BINARY_SHIFT_LEFT"),
        BinaryOp::ShiftRight => Some("PHPC_NATIVE_VALUE_BINARY_SHIFT_RIGHT"),
        BinaryOp::LogicalAnd
        | BinaryOp::LogicalOr
        | BinaryOp::LogicalXor
        | BinaryOp::Eq
        | BinaryOp::Ne
        | BinaryOp::StrictEq
        | BinaryOp::StrictNe
        | BinaryOp::Lt
        | BinaryOp::Le
        | BinaryOp::Gt
        | BinaryOp::Ge
        | BinaryOp::NullCoalesce => None,
    }
}

fn native_value_bitwise_binary_op_tag(op: BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::BitwiseAnd => Some("PHPC_NATIVE_VALUE_BITWISE_AND"),
        BinaryOp::BitwiseOr => Some("PHPC_NATIVE_VALUE_BITWISE_OR"),
        BinaryOp::BitwiseXor => Some("PHPC_NATIVE_VALUE_BITWISE_XOR"),
        BinaryOp::ShiftLeft => Some("PHPC_NATIVE_VALUE_BITWISE_SHIFT_LEFT"),
        BinaryOp::ShiftRight => Some("PHPC_NATIVE_VALUE_BITWISE_SHIFT_RIGHT"),
        _ => None,
    }
}

fn native_value_bitwise_binary_result_prefix(op: BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::BitwiseAnd => Some("native_value_bitwise_and"),
        BinaryOp::BitwiseOr => Some("native_value_bitwise_or"),
        BinaryOp::BitwiseXor => Some("native_value_bitwise_xor"),
        BinaryOp::ShiftLeft => Some("native_value_shift_left"),
        BinaryOp::ShiftRight => Some("native_value_shift_right"),
        _ => None,
    }
}

fn native_array_lvalue_compound_binary_op_tag(op: CompoundAssignOp) -> &'static str {
    match op {
        CompoundAssignOp::Add => "PHPC_NATIVE_VALUE_BINARY_ADD",
        CompoundAssignOp::Sub => "PHPC_NATIVE_VALUE_BINARY_SUB",
        CompoundAssignOp::Mul => "PHPC_NATIVE_VALUE_BINARY_MUL",
        CompoundAssignOp::Div => "PHPC_NATIVE_VALUE_BINARY_DIV",
        CompoundAssignOp::Mod => "PHPC_NATIVE_VALUE_BINARY_MOD",
        CompoundAssignOp::Concat => "PHPC_NATIVE_VALUE_BINARY_CONCAT",
        CompoundAssignOp::BitwiseAnd => "PHPC_NATIVE_VALUE_BINARY_BITWISE_AND",
        CompoundAssignOp::BitwiseOr => "PHPC_NATIVE_VALUE_BINARY_BITWISE_OR",
        CompoundAssignOp::BitwiseXor => "PHPC_NATIVE_VALUE_BINARY_BITWISE_XOR",
        CompoundAssignOp::ShiftLeft => "PHPC_NATIVE_VALUE_BINARY_SHIFT_LEFT",
        CompoundAssignOp::ShiftRight => "PHPC_NATIVE_VALUE_BINARY_SHIFT_RIGHT",
    }
}

fn native_array_lvalue_increment_decrement_op_tag(op: IncrementDecrementOp) -> &'static str {
    match op {
        IncrementDecrementOp::Increment => "PHPC_NATIVE_ARRAY_LVALUE_INCREMENT",
        IncrementDecrementOp::Decrement => "PHPC_NATIVE_ARRAY_LVALUE_DECREMENT",
    }
}

fn native_array_lvalue_increment_decrement_position_tag(
    position: IncrementDecrementPosition,
) -> &'static str {
    match position {
        IncrementDecrementPosition::Pre => "PHPC_NATIVE_ARRAY_LVALUE_POSITION_PRE",
        IncrementDecrementPosition::Post => "PHPC_NATIVE_ARRAY_LVALUE_POSITION_POST",
    }
}

#[derive(Clone, Copy)]
enum NativeArrayPointerBuiltin {
    Current,
    Key,
    Next,
    Prev,
    Reset,
    End,
}

impl NativeArrayPointerBuiltin {
    fn operation_tag(self) -> &'static str {
        match self {
            Self::Current => "PHPC_NATIVE_ARRAY_LVALUE_POINTER_CURRENT",
            Self::Key => "PHPC_NATIVE_ARRAY_LVALUE_POINTER_KEY",
            Self::Next => "PHPC_NATIVE_ARRAY_LVALUE_POINTER_NEXT",
            Self::Prev => "PHPC_NATIVE_ARRAY_LVALUE_POINTER_PREV",
            Self::Reset => "PHPC_NATIVE_ARRAY_LVALUE_POINTER_RESET",
            Self::End => "PHPC_NATIVE_ARRAY_LVALUE_POINTER_END",
        }
    }
}

fn native_array_pointer_builtin(name: &str, args: &[Expr]) -> Option<NativeArrayPointerBuiltin> {
    match name.to_ascii_lowercase().as_str() {
        "current" if args.len() == 1 => Some(NativeArrayPointerBuiltin::Current),
        "key" if args.len() == 1 => Some(NativeArrayPointerBuiltin::Key),
        "next" if args.len() == 1 => Some(NativeArrayPointerBuiltin::Next),
        "prev" if args.len() == 1 => Some(NativeArrayPointerBuiltin::Prev),
        "reset" if args.len() == 1 => Some(NativeArrayPointerBuiltin::Reset),
        "end" if args.len() == 1 => Some(NativeArrayPointerBuiltin::End),
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum NativeArraySortBuiltin {
    Sort,
    Rsort,
    Asort,
    Arsort,
    Ksort,
    Krsort,
    Natsort,
    Natcasesort,
    Usort,
    Uasort,
    Uksort,
}

#[derive(Clone, Copy)]
enum NativeValueArrayCallbackBuiltin {
    Filter,
    Map,
    Reduce,
}

impl NativeArraySortBuiltin {
    fn operation_tag(self) -> &'static str {
        match self {
            Self::Sort => "PHPC_NATIVE_ARRAY_LVALUE_SORT_SORT",
            Self::Rsort => "PHPC_NATIVE_ARRAY_LVALUE_SORT_RSORT",
            Self::Asort => "PHPC_NATIVE_ARRAY_LVALUE_SORT_ASORT",
            Self::Arsort => "PHPC_NATIVE_ARRAY_LVALUE_SORT_ARSORT",
            Self::Ksort => "PHPC_NATIVE_ARRAY_LVALUE_SORT_KSORT",
            Self::Krsort => "PHPC_NATIVE_ARRAY_LVALUE_SORT_KRSORT",
            Self::Natsort => "PHPC_NATIVE_ARRAY_LVALUE_SORT_NATSORT",
            Self::Natcasesort => "PHPC_NATIVE_ARRAY_LVALUE_SORT_NATCASESORT",
            Self::Usort => "PHPC_NATIVE_ARRAY_LVALUE_SORT_USORT",
            Self::Uasort => "PHPC_NATIVE_ARRAY_LVALUE_SORT_UASORT",
            Self::Uksort => "PHPC_NATIVE_ARRAY_LVALUE_SORT_UKSORT",
        }
    }
}

impl NativeValueArrayCallbackBuiltin {
    fn operation_tag(self) -> &'static str {
        match self {
            Self::Filter => "PHPC_NATIVE_VALUE_ARRAY_CALLBACK_FILTER",
            Self::Map => "PHPC_NATIVE_VALUE_ARRAY_CALLBACK_MAP",
            Self::Reduce => "PHPC_NATIVE_VALUE_ARRAY_CALLBACK_REDUCE",
        }
    }
}

fn native_array_sort_builtin(name: &str, args: &[Expr]) -> Option<NativeArraySortBuiltin> {
    match name.to_ascii_lowercase().as_str() {
        "sort" if (1..=2).contains(&args.len()) => Some(NativeArraySortBuiltin::Sort),
        "rsort" if (1..=2).contains(&args.len()) => Some(NativeArraySortBuiltin::Rsort),
        "asort" if (1..=2).contains(&args.len()) => Some(NativeArraySortBuiltin::Asort),
        "arsort" if (1..=2).contains(&args.len()) => Some(NativeArraySortBuiltin::Arsort),
        "ksort" if (1..=2).contains(&args.len()) => Some(NativeArraySortBuiltin::Ksort),
        "krsort" if (1..=2).contains(&args.len()) => Some(NativeArraySortBuiltin::Krsort),
        "natsort" if args.len() == 1 => Some(NativeArraySortBuiltin::Natsort),
        "natcasesort" if args.len() == 1 => Some(NativeArraySortBuiltin::Natcasesort),
        "usort" if args.len() == 2 => Some(NativeArraySortBuiltin::Usort),
        "uasort" if args.len() == 2 => Some(NativeArraySortBuiltin::Uasort),
        "uksort" if args.len() == 2 => Some(NativeArraySortBuiltin::Uksort),
        _ => None,
    }
}

fn native_value_array_callback_builtin(
    name: &str,
    args: &[Expr],
) -> Option<NativeValueArrayCallbackBuiltin> {
    match name.to_ascii_lowercase().as_str() {
        "array_filter" if (2..=3).contains(&args.len()) => {
            Some(NativeValueArrayCallbackBuiltin::Filter)
        }
        "array_map" if args.len() >= 2 => Some(NativeValueArrayCallbackBuiltin::Map),
        "array_reduce" if (2..=3).contains(&args.len()) => {
            Some(NativeValueArrayCallbackBuiltin::Reduce)
        }
        _ => None,
    }
}

fn native_value_comparison_op_tag(op: BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Eq => Some("PHPC_NATIVE_VALUE_COMPARISON_EQ"),
        BinaryOp::Ne => Some("PHPC_NATIVE_VALUE_COMPARISON_NE"),
        BinaryOp::Lt => Some("PHPC_NATIVE_VALUE_COMPARISON_LT"),
        BinaryOp::Le => Some("PHPC_NATIVE_VALUE_COMPARISON_LE"),
        BinaryOp::Gt => Some("PHPC_NATIVE_VALUE_COMPARISON_GT"),
        BinaryOp::Ge => Some("PHPC_NATIVE_VALUE_COMPARISON_GE"),
        BinaryOp::Add
        | BinaryOp::Sub
        | BinaryOp::Mul
        | BinaryOp::Div
        | BinaryOp::Mod
        | BinaryOp::Concat
        | BinaryOp::StrictEq
        | BinaryOp::StrictNe
        | BinaryOp::NullCoalesce
        | BinaryOp::LogicalAnd
        | BinaryOp::LogicalOr
        | BinaryOp::LogicalXor
        | BinaryOp::BitwiseAnd
        | BinaryOp::BitwiseOr
        | BinaryOp::BitwiseXor
        | BinaryOp::ShiftLeft
        | BinaryOp::ShiftRight => None,
    }
}

fn native_value_result_output_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Index { .. } => true,
        Expr::CompoundAssign { .. } => true,
        Expr::NullCoalesceAssign { .. } => true,
        Expr::Unary { op, .. } => native_value_unary_op_tag(*op).is_some(),
        Expr::Binary { op, .. } => {
            native_value_binary_op_tag(*op).is_some() || matches!(op, BinaryOp::NullCoalesce)
        }
        Expr::Cast { .. } => true,
        Expr::Call { name, args, .. } => {
            native_value_cast_builtin_op_tag(name).is_some()
                || native_value_type_name_tag(name).is_some()
                || native_string_result_operation_for_name(name).is_some()
                || native_array_pointer_builtin(name, args).is_some()
                || native_value_array_callback_builtin(name, args).is_some()
        }
        _ => false,
    }
}

fn native_conditional_rhs_needs_cleanup_boundary(expr: &Expr) -> bool {
    match expr {
        Expr::Array { .. }
        | Expr::Assign { .. }
        | Expr::CompoundAssign { .. }
        | Expr::NullCoalesceAssign { .. }
        | Expr::IncrementDecrement { .. }
        | Expr::Ternary { .. }
        | Expr::ShortTernary { .. } => true,
        Expr::Index { target, index, .. } => {
            native_conditional_rhs_needs_cleanup_boundary(target)
                || native_conditional_rhs_needs_cleanup_boundary(index)
        }
        Expr::AppendIndex { target, .. }
        | Expr::Property { target, .. }
        | Expr::ObjectStaticProperty { target, .. }
        | Expr::InstanceOf { expr: target, .. }
        | Expr::Clone { expr: target, .. }
        | Expr::Unary { expr: target, .. }
        | Expr::ErrorControl { expr: target, .. }
        | Expr::Include { path: target, .. }
        | Expr::Require { path: target, .. }
        | Expr::Cast { expr: target, .. } => native_conditional_rhs_needs_cleanup_boundary(target),
        Expr::DynamicProperty {
            target, property, ..
        } => {
            native_conditional_rhs_needs_cleanup_boundary(target)
                || native_conditional_rhs_needs_cleanup_boundary(property)
        }
        Expr::Binary { left, right, .. } => {
            native_conditional_rhs_needs_cleanup_boundary(left)
                || native_conditional_rhs_needs_cleanup_boundary(right)
        }
        Expr::Call { args, .. } => args
            .iter()
            .any(native_conditional_rhs_needs_cleanup_boundary),
        Expr::DynamicCall { callee, args, .. } => {
            native_conditional_rhs_needs_cleanup_boundary(callee)
                || args
                    .iter()
                    .any(native_conditional_rhs_needs_cleanup_boundary)
        }
        Expr::New { args, .. } => args
            .iter()
            .any(native_conditional_rhs_needs_cleanup_boundary),
        Expr::Closure { .. } => true,
        Expr::Null(_)
        | Expr::Bool(_, _)
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::String(_, _)
        | Expr::InterpolatedString { .. }
        | Expr::Variable(_, _)
        | Expr::MagicLine { .. }
        | Expr::MagicFile { .. }
        | Expr::MagicDir { .. }
        | Expr::MagicFunction { .. }
        | Expr::MagicClass { .. }
        | Expr::MagicMethod { .. }
        | Expr::GlobalConstant { .. }
        | Expr::ClassNameConstant { .. }
        | Expr::SelfClassNameConstant { .. }
        | Expr::ParentClassNameConstant { .. }
        | Expr::StaticClassNameConstant { .. }
        | Expr::ClassConstant { .. }
        | Expr::SelfClassConstant { .. }
        | Expr::ParentClassConstant { .. }
        | Expr::LateStaticClassConstant { .. }
        | Expr::StaticProperty { .. }
        | Expr::SelfStaticProperty { .. }
        | Expr::ParentStaticProperty { .. }
        | Expr::LateStaticProperty { .. }
        | Expr::MethodCall { .. }
        | Expr::DynamicMethodCall { .. }
        | Expr::ParentMethodCall { .. }
        | Expr::StaticMethodCall { .. }
        | Expr::ObjectStaticMethodCall { .. }
        | Expr::SelfMethodCall { .. }
        | Expr::LateStaticMethodCall { .. } => false,
    }
}

fn native_value_cast_op_tag(kind: CastKind) -> &'static str {
    match kind {
        CastKind::String => "PHPC_NATIVE_VALUE_CAST_STRING",
        CastKind::Int => "PHPC_NATIVE_VALUE_CAST_INT",
        CastKind::Bool => "PHPC_NATIVE_VALUE_CAST_BOOL",
        CastKind::Float => "PHPC_NATIVE_VALUE_CAST_FLOAT",
        CastKind::Array => "PHPC_NATIVE_VALUE_CAST_ARRAY",
    }
}

fn native_value_cast_builtin_op_tag(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "strval" => Some("PHPC_NATIVE_VALUE_CAST_STRING"),
        "boolval" => Some("PHPC_NATIVE_VALUE_CAST_BOOL"),
        "floatval" | "doubleval" => Some("PHPC_NATIVE_VALUE_CAST_FLOAT"),
        _ => None,
    }
}

fn native_value_type_name_tag(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "gettype" => Some("PHPC_NATIVE_VALUE_TYPE_NAME_GETTYPE"),
        "get_debug_type" => Some("PHPC_NATIVE_VALUE_TYPE_NAME_DEBUG"),
        _ => None,
    }
}

fn native_value_type_predicate_tag(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "is_null" => Some("PHPC_NATIVE_VALUE_TYPE_IS_NULL"),
        "is_bool" => Some("PHPC_NATIVE_VALUE_TYPE_IS_BOOL"),
        "is_int" | "is_integer" | "is_long" => Some("PHPC_NATIVE_VALUE_TYPE_IS_INT"),
        "is_float" | "is_double" => Some("PHPC_NATIVE_VALUE_TYPE_IS_FLOAT"),
        "is_string" => Some("PHPC_NATIVE_VALUE_TYPE_IS_STRING"),
        "is_array" => Some("PHPC_NATIVE_VALUE_TYPE_IS_ARRAY"),
        "is_scalar" => Some("PHPC_NATIVE_VALUE_TYPE_IS_SCALAR"),
        "is_numeric" => Some("PHPC_NATIVE_VALUE_TYPE_IS_NUMERIC"),
        "is_countable" => Some("PHPC_NATIVE_VALUE_TYPE_IS_COUNTABLE"),
        "is_iterable" => Some("PHPC_NATIVE_VALUE_TYPE_IS_ITERABLE"),
        "is_object" => Some("PHPC_NATIVE_VALUE_TYPE_IS_OBJECT"),
        _ => None,
    }
}

fn native_value_result_expr_call_operation(
    expr: &Expr,
    blocker: NativeCallBlocker,
) -> Option<NativeCallOperation> {
    match expr {
        Expr::Unary { op, expr, .. } if native_value_unary_op_tag(*op).is_some() => {
            native_value_result_expr_call_operation(expr, blocker)
        }
        Expr::Binary {
            left, op, right, ..
        } if native_value_binary_op_tag(*op).is_some()
            || native_value_comparison_op_tag(*op).is_some() =>
        {
            native_value_result_expr_call_operation(left, blocker)
                .or_else(|| native_value_result_expr_call_operation(right, blocker))
        }
        Expr::Cast { expr, .. } => native_value_result_expr_call_operation(expr, blocker),
        Expr::Call { name, args, span } if native_value_cast_builtin_op_tag(name).is_some() => {
            let [arg] = args.as_slice() else {
                return Some(NativeCallOperation::direct_named_value(*span, blocker));
            };
            native_value_result_expr_call_operation(arg, blocker)
        }
        Expr::Call { name, args, span } if native_value_type_name_tag(name).is_some() => {
            let [arg] = args.as_slice() else {
                return Some(NativeCallOperation::direct_named_value(*span, blocker));
            };
            native_value_result_expr_call_operation(arg, blocker)
        }
        Expr::Call { name, args, span }
            if native_string_result_operation_for_name(name).is_some() =>
        {
            let [arg] = args.as_slice() else {
                return Some(NativeCallOperation::direct_named_value(*span, blocker));
            };
            native_value_result_expr_call_operation(arg, blocker)
        }
        Expr::Call { name, args, span } if native_array_pointer_builtin(name, args).is_some() => {
            let [arg] = args.as_slice() else {
                return Some(NativeCallOperation::direct_named_value(*span, blocker));
            };
            native_value_result_expr_call_operation(arg, blocker)
        }
        Expr::Call { name, args, .. } if native_array_sort_builtin(name, args).is_some() => args
            .iter()
            .find_map(|arg| native_value_result_expr_call_operation(arg, blocker)),
        Expr::Call { name, args, .. }
            if native_value_array_callback_builtin(name, args).is_some() =>
        {
            args.iter()
                .find_map(|arg| native_value_result_expr_call_operation(arg, blocker))
        }
        _ => native_expr_call_result_operation(expr, blocker),
    }
}

fn native_statement_assignment_rhs_call_operation(
    target: &AssignTarget,
    expr: &Expr,
) -> Option<NativeCallOperation> {
    match target {
        AssignTarget::Variable { .. } => None,
        _ => native_value_result_expr_call_operation(
            expr,
            NativeCallBlocker::StatementOperandEvaluationCleanup,
        ),
    }
}

fn native_statement_operand_call_operation(stmt: &Stmt) -> Option<NativeCallOperation> {
    match stmt {
        Stmt::If { condition, .. }
        | Stmt::While { condition, .. }
        | Stmt::DoWhile { condition, .. } => {
            native_statement_operand_call_result_operation(condition)
        }
        Stmt::For {
            initializers,
            conditions,
            increments,
            ..
        } => native_for_action_list_call_operation(initializers)
            .or_else(|| {
                conditions
                    .iter()
                    .find_map(native_statement_operand_call_result_operation)
            })
            .or_else(|| native_for_action_list_call_operation(increments)),
        Stmt::Switch { value, cases, .. } => native_statement_operand_call_result_operation(value)
            .or_else(|| {
                cases
                    .iter()
                    .filter_map(|case| case.condition.as_ref())
                    .find_map(native_statement_operand_call_result_operation)
            }),
        Stmt::Foreach { iterable, .. } => native_statement_operand_call_result_operation(iterable),
        Stmt::Assign { target, expr, .. } => native_assignment_target_call_operation(target)
            .or_else(|| native_statement_assignment_rhs_call_operation(target, expr)),
        Stmt::CompoundAssign { target, expr, .. }
        | Stmt::NullCoalesceAssign { target, expr, .. } => {
            native_assignment_target_call_operation(target)
                .or_else(|| native_statement_operand_call_result_operation(expr))
        }
        Stmt::IncrementDecrement { target, .. } => native_assignment_target_call_operation(target),
        Stmt::ConstDeclaration { declarations, .. } => {
            declarations.iter().find_map(|declaration| {
                native_statement_operand_call_result_operation(&declaration.value)
            })
        }
        Stmt::Require { path, .. } | Stmt::Include { path, .. } => {
            native_statement_operand_call_result_operation(path)
        }
        Stmt::Return {
            value: Some(value), ..
        } => native_statement_operand_call_result_operation(value),
        Stmt::Throw { expr, .. } => native_statement_operand_call_result_operation(expr),
        Stmt::StaticLocal { declarations, .. } => declarations
            .iter()
            .filter_map(|declaration| declaration.default.as_ref())
            .find_map(native_statement_operand_call_result_operation),
        Stmt::ReferenceAssign { target, source, .. } => {
            native_reference_assignment_call_operation(target, source)
        }
        Stmt::UnsetArrayIndex { index, .. } => native_lvalue_operand_call_result_operation(index),
        Stmt::UnsetNestedArrayIndex { indices, .. } => {
            native_expr_list_call_result_operation(indices)
        }
        Stmt::UnsetDynamicObjectProperty { property, .. } => {
            native_lvalue_operand_call_result_operation(property)
        }
        Stmt::UnsetMany { targets, .. } => {
            targets.iter().find_map(native_unset_target_call_operation)
        }
        Stmt::Namespace { .. }
        | Stmt::Use { .. }
        | Stmt::Echo { .. }
        | Stmt::Print { .. }
        | Stmt::Expr { .. }
        | Stmt::Goto { .. }
        | Stmt::Label { .. }
        | Stmt::Function(_)
        | Stmt::Interface(_)
        | Stmt::Trait(_)
        | Stmt::Enum(_)
        | Stmt::Class(_)
        | Stmt::UnsetVariable { .. }
        | Stmt::UnsetObjectProperty { .. }
        | Stmt::UnsetStaticProperty { .. }
        | Stmt::UnsetSelfStaticProperty { .. }
        | Stmt::UnsetParentStaticProperty { .. }
        | Stmt::UnsetLateStaticProperty { .. }
        | Stmt::Return { value: None, .. }
        | Stmt::Try { .. }
        | Stmt::Break { .. }
        | Stmt::Continue { .. }
        | Stmt::Global { .. } => None,
    }
}

fn native_for_action_list_call_operation(actions: &[ForAction]) -> Option<NativeCallOperation> {
    actions.iter().find_map(native_for_action_call_operation)
}

fn native_for_action_call_operation(action: &ForAction) -> Option<NativeCallOperation> {
    match action {
        ForAction::Assign { target, expr } | ForAction::CompoundAssign { target, expr, .. } => {
            native_assignment_target_call_operation(target)
                .or_else(|| native_statement_operand_call_result_operation(expr))
        }
        ForAction::IncrementDecrement { target, .. } => {
            native_assignment_target_call_operation(target)
        }
        ForAction::Expr { expr } => native_statement_operand_call_result_operation(expr),
    }
}

fn native_comparison_op_for_binary_op(op: BinaryOp) -> Option<NativeComparisonOp> {
    Some(match op {
        BinaryOp::Eq => NativeComparisonOp::LooseEq,
        BinaryOp::Ne => NativeComparisonOp::LooseNe,
        BinaryOp::Lt => NativeComparisonOp::LooseLt,
        BinaryOp::Le => NativeComparisonOp::LooseLe,
        BinaryOp::Gt => NativeComparisonOp::LooseGt,
        BinaryOp::Ge => NativeComparisonOp::LooseGe,
        BinaryOp::StrictEq => NativeComparisonOp::StrictEq,
        BinaryOp::StrictNe => NativeComparisonOp::StrictNe,
        _ => return None,
    })
}

fn native_comparison_c_uint8_argument(op: NativeComparisonOp) -> String {
    (op as u8).to_string()
}

fn native_expr_contains_call_result(expr: &Expr) -> bool {
    match expr {
        Expr::Call { .. }
        | Expr::DynamicCall { .. }
        | Expr::MethodCall { .. }
        | Expr::DynamicMethodCall { .. }
        | Expr::ParentMethodCall { .. }
        | Expr::StaticMethodCall { .. }
        | Expr::ObjectStaticMethodCall { .. }
        | Expr::SelfMethodCall { .. }
        | Expr::LateStaticMethodCall { .. }
        | Expr::New { .. }
        | Expr::Closure { .. } => true,
        Expr::Array { items, .. } => items.iter().any(|item| {
            item.key
                .as_ref()
                .is_some_and(native_expr_contains_call_result)
                || native_expr_contains_call_result(&item.value)
        }),
        Expr::Index { target, index, .. } => {
            native_expr_contains_call_result(target) || native_expr_contains_call_result(index)
        }
        Expr::AppendIndex { target, .. }
        | Expr::Property { target, .. }
        | Expr::ObjectStaticProperty { target, .. }
        | Expr::InstanceOf { expr: target, .. } => native_expr_contains_call_result(target),
        Expr::DynamicProperty {
            target, property, ..
        } => native_expr_contains_call_result(target) || native_expr_contains_call_result(property),
        Expr::Clone { expr, .. }
        | Expr::Unary { expr, .. }
        | Expr::ErrorControl { expr, .. }
        | Expr::Include { path: expr, .. }
        | Expr::Require { path: expr, .. }
        | Expr::Cast { expr, .. } => native_expr_contains_call_result(expr),
        Expr::Binary { left, right, .. } => {
            native_expr_contains_call_result(left) || native_expr_contains_call_result(right)
        }
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            native_expr_contains_call_result(condition)
                || native_expr_contains_call_result(if_true)
                || native_expr_contains_call_result(if_false)
        }
        Expr::ShortTernary {
            condition,
            if_false,
            ..
        } => {
            native_expr_contains_call_result(condition)
                || native_expr_contains_call_result(if_false)
        }
        Expr::Assign { target, expr, .. }
        | Expr::CompoundAssign { target, expr, .. }
        | Expr::NullCoalesceAssign { target, expr, .. } => {
            native_assign_target_contains_call_result(target)
                || native_expr_contains_call_result(expr)
        }
        Expr::IncrementDecrement { target, .. } => {
            native_assign_target_contains_call_result(target)
        }
        Expr::Null(_)
        | Expr::Bool(_, _)
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::String(_, _)
        | Expr::InterpolatedString { .. }
        | Expr::Variable(_, _)
        | Expr::MagicLine { .. }
        | Expr::MagicFile { .. }
        | Expr::MagicDir { .. }
        | Expr::MagicFunction { .. }
        | Expr::MagicClass { .. }
        | Expr::MagicMethod { .. }
        | Expr::GlobalConstant { .. }
        | Expr::ClassNameConstant { .. }
        | Expr::SelfClassNameConstant { .. }
        | Expr::ParentClassNameConstant { .. }
        | Expr::StaticClassNameConstant { .. }
        | Expr::ClassConstant { .. }
        | Expr::SelfClassConstant { .. }
        | Expr::ParentClassConstant { .. }
        | Expr::LateStaticClassConstant { .. }
        | Expr::StaticProperty { .. }
        | Expr::SelfStaticProperty { .. }
        | Expr::ParentStaticProperty { .. }
        | Expr::LateStaticProperty { .. } => false,
    }
}

fn native_assign_target_contains_call_result(target: &AssignTarget) -> bool {
    match target {
        AssignTarget::ArrayIndex { index, .. } => {
            index.as_ref().is_some_and(native_expr_contains_call_result)
        }
        AssignTarget::NestedArrayIndex { indices, .. }
        | AssignTarget::ObjectPropertyArrayIndex { indices, .. } => {
            indices.iter().any(native_expr_contains_call_result)
        }
        AssignTarget::NestedArrayAppend {
            indices,
            suffix_indices,
            ..
        }
        | AssignTarget::ObjectPropertyArrayAppend {
            indices,
            suffix_indices,
            ..
        } => indices
            .iter()
            .chain(suffix_indices.iter())
            .any(native_expr_contains_call_result),
        AssignTarget::NonDirectProperty { holder, .. } => native_expr_contains_call_result(holder),
        AssignTarget::NonDirectDynamicProperty {
            holder, property, ..
        } => native_expr_contains_call_result(holder) || native_expr_contains_call_result(property),
        AssignTarget::DynamicObjectPropertyArrayIndex {
            property, indices, ..
        } => {
            native_expr_contains_call_result(property)
                || indices.iter().any(native_expr_contains_call_result)
        }
        AssignTarget::NonDirectObjectPropertyArrayIndex {
            holder, indices, ..
        } => {
            native_expr_contains_call_result(holder)
                || indices.iter().any(native_expr_contains_call_result)
        }
        AssignTarget::NonDirectObjectPropertyArrayAppend {
            holder,
            indices,
            suffix_indices,
            ..
        } => {
            native_expr_contains_call_result(holder)
                || indices
                    .iter()
                    .chain(suffix_indices.iter())
                    .any(native_expr_contains_call_result)
        }
        AssignTarget::NonDirectDynamicObjectPropertyArrayIndex {
            holder,
            property,
            indices,
            ..
        } => {
            native_expr_contains_call_result(holder)
                || native_expr_contains_call_result(property)
                || indices.iter().any(native_expr_contains_call_result)
        }
        AssignTarget::NonDirectDynamicObjectPropertyArrayAppend {
            holder,
            property,
            indices,
            suffix_indices,
            ..
        } => {
            native_expr_contains_call_result(holder)
                || native_expr_contains_call_result(property)
                || indices
                    .iter()
                    .chain(suffix_indices.iter())
                    .any(native_expr_contains_call_result)
        }
        AssignTarget::DynamicObjectPropertyArrayAppend {
            property,
            indices,
            suffix_indices,
            ..
        } => {
            native_expr_contains_call_result(property)
                || indices
                    .iter()
                    .chain(suffix_indices.iter())
                    .any(native_expr_contains_call_result)
        }
        AssignTarget::DynamicProperty { property, .. }
        | AssignTarget::ObjectStaticProperty {
            target: property, ..
        } => native_expr_contains_call_result(property),
        AssignTarget::Variable { .. }
        | AssignTarget::List { .. }
        | AssignTarget::Property { .. }
        | AssignTarget::StaticProperty { .. }
        | AssignTarget::SelfStaticProperty { .. }
        | AssignTarget::ParentStaticProperty { .. }
        | AssignTarget::LateStaticProperty { .. } => false,
    }
}

fn native_reference_source_call_operation(source: &ReferenceSource) -> Option<NativeCallOperation> {
    native_reference_source_call_result_operation(source)
        .or_else(|| native_reference_source_lvalue_operand_call_operation(source))
}

fn native_reference_assignment_call_operation(
    target: &AssignTarget,
    source: &ReferenceSource,
) -> Option<NativeCallOperation> {
    native_assignment_target_call_operation(target)
        .or_else(|| native_reference_source_call_operation(source))
}

fn native_reference_source_call_result_operation(
    source: &ReferenceSource,
) -> Option<NativeCallOperation> {
    let (expr, span) = match source {
        ReferenceSource::MethodCall { expr, span }
        | ReferenceSource::Property { expr, span }
        | ReferenceSource::StaticProperty { expr, span }
        | ReferenceSource::StaticPropertyArrayIndex { expr, span, .. } => (expr, *span),
        ReferenceSource::ExpressionArrayIndex { target, span, .. }
        | ReferenceSource::ExpressionArrayAppend { target, span, .. } => (target, *span),
        ReferenceSource::NonDirectObjectPropertyArrayAppend { holder, span, .. }
        | ReferenceSource::NonDirectDynamicObjectPropertyArrayAppend { holder, span, .. }
        | ReferenceSource::NonDirectObjectPropertyNestedArrayIndex { holder, span, .. }
        | ReferenceSource::NonDirectDynamicObjectPropertyNestedArrayIndex {
            holder, span, ..
        } => (holder, *span),
        _ => return None,
    };

    native_reference_expr_call_callee(expr)
        .map(|callee| NativeCallOperation::reference_result(span, callee))
}

fn native_reference_source_lvalue_operand_call_operation(
    source: &ReferenceSource,
) -> Option<NativeCallOperation> {
    match source {
        ReferenceSource::ArrayIndex { index, .. }
        | ReferenceSource::ObjectPropertyArrayIndex { index, .. } => {
            native_lvalue_operand_call_result_operation(index)
        }
        ReferenceSource::ArrayAppend { indices, .. }
        | ReferenceSource::NestedArrayIndex { indices, .. }
        | ReferenceSource::ObjectPropertyArrayAppend { indices, .. }
        | ReferenceSource::ObjectPropertyNestedArrayIndex { indices, .. } => {
            native_expr_list_call_result_operation(indices)
        }
        ReferenceSource::DynamicObjectPropertyArrayIndex {
            property, index, ..
        } => native_lvalue_operand_call_result_operation(property)
            .or_else(|| native_lvalue_operand_call_result_operation(index)),
        ReferenceSource::DynamicObjectPropertyArrayAppend {
            property, indices, ..
        }
        | ReferenceSource::DynamicObjectPropertyNestedArrayIndex {
            property, indices, ..
        } => native_lvalue_operand_call_result_operation(property)
            .or_else(|| native_expr_list_call_result_operation(indices)),
        ReferenceSource::NonDirectObjectPropertyArrayAppend {
            holder, indices, ..
        }
        | ReferenceSource::NonDirectObjectPropertyNestedArrayIndex {
            holder, indices, ..
        } => native_lvalue_operand_call_result_operation(holder)
            .or_else(|| native_expr_list_call_result_operation(indices)),
        ReferenceSource::NonDirectDynamicObjectPropertyArrayAppend {
            holder,
            property,
            indices,
            ..
        }
        | ReferenceSource::NonDirectDynamicObjectPropertyNestedArrayIndex {
            holder,
            property,
            indices,
            ..
        } => native_lvalue_operand_call_result_operation(holder)
            .or_else(|| native_lvalue_operand_call_result_operation(property))
            .or_else(|| native_expr_list_call_result_operation(indices)),
        ReferenceSource::Property { expr, .. } | ReferenceSource::StaticProperty { expr, .. } => {
            native_lvalue_operand_call_result_operation(expr)
        }
        ReferenceSource::StaticPropertyArrayIndex { expr, indices, .. } => {
            native_lvalue_operand_call_result_operation(expr)
                .or_else(|| native_expr_list_call_result_operation(indices))
        }
        ReferenceSource::ExpressionArrayIndex {
            target, indices, ..
        }
        | ReferenceSource::ExpressionArrayAppend {
            target, indices, ..
        } => native_lvalue_operand_call_result_operation(target)
            .or_else(|| native_expr_list_call_result_operation(indices)),
        ReferenceSource::Variable { .. } | ReferenceSource::MethodCall { .. } => None,
    }
}

fn native_reference_expr_call_callee(expr: &Expr) -> Option<NativeCallCallee> {
    if let Some(callee) = native_call_callee_for_expr(expr) {
        return Some(callee);
    }

    match expr {
        Expr::Index { target, .. }
        | Expr::AppendIndex { target, .. }
        | Expr::Property { target, .. }
        | Expr::DynamicProperty { target, .. }
        | Expr::ObjectStaticProperty { target, .. } => native_reference_expr_call_callee(target),
        _ => None,
    }
}

fn native_dereferenced_call_result_operation(expr: &Expr) -> Option<NativeCallOperation> {
    let (target, span) = match expr {
        Expr::Index { target, span, .. }
        | Expr::AppendIndex { target, span }
        | Expr::Property { target, span, .. }
        | Expr::DynamicProperty { target, span, .. }
        | Expr::ObjectStaticProperty { target, span, .. } => (target.as_ref(), *span),
        _ => return None,
    };

    native_reference_expr_call_callee(target)
        .map(|callee| NativeCallOperation::dereferenced_value_result(span, callee))
}

fn native_assignment_target_call_result_operation(
    target: &AssignTarget,
) -> Option<NativeCallOperation> {
    native_assignment_target_call_result_callee(target)
        .map(|callee| NativeCallOperation::dereferenced_value_result(target.span(), callee))
}

fn native_assignment_target_call_result_callee(target: &AssignTarget) -> Option<NativeCallCallee> {
    match target {
        AssignTarget::NonDirectProperty { holder, .. }
        | AssignTarget::NonDirectObjectPropertyArrayIndex { holder, .. }
        | AssignTarget::NonDirectObjectPropertyArrayAppend { holder, .. }
        | AssignTarget::NonDirectDynamicProperty { holder, .. }
        | AssignTarget::NonDirectDynamicObjectPropertyArrayIndex { holder, .. }
        | AssignTarget::NonDirectDynamicObjectPropertyArrayAppend { holder, .. } => {
            native_reference_expr_call_callee(holder)
        }
        AssignTarget::ObjectStaticProperty { target, .. } => {
            native_reference_expr_call_callee(target)
        }
        AssignTarget::Variable { .. }
        | AssignTarget::List { .. }
        | AssignTarget::ArrayIndex { .. }
        | AssignTarget::NestedArrayIndex { .. }
        | AssignTarget::NestedArrayAppend { .. }
        | AssignTarget::Property { .. }
        | AssignTarget::ObjectPropertyArrayIndex { .. }
        | AssignTarget::DynamicObjectPropertyArrayIndex { .. }
        | AssignTarget::ObjectPropertyArrayAppend { .. }
        | AssignTarget::DynamicObjectPropertyArrayAppend { .. }
        | AssignTarget::DynamicProperty { .. }
        | AssignTarget::StaticProperty { .. }
        | AssignTarget::SelfStaticProperty { .. }
        | AssignTarget::ParentStaticProperty { .. }
        | AssignTarget::LateStaticProperty { .. } => None,
    }
}

fn native_assignment_target_call_operation(target: &AssignTarget) -> Option<NativeCallOperation> {
    native_assignment_target_call_result_operation(target)
        .or_else(|| native_assignment_target_lvalue_operand_call_operation(target))
}

fn native_assignment_target_lvalue_operand_call_operation(
    target: &AssignTarget,
) -> Option<NativeCallOperation> {
    match target {
        AssignTarget::ArrayIndex { index, .. } => index
            .as_ref()
            .and_then(native_lvalue_operand_call_result_operation),
        AssignTarget::NestedArrayIndex { indices, .. }
        | AssignTarget::ObjectPropertyArrayIndex { indices, .. } => {
            native_expr_list_call_result_operation(indices)
        }
        AssignTarget::NestedArrayAppend {
            indices,
            suffix_indices,
            ..
        }
        | AssignTarget::ObjectPropertyArrayAppend {
            indices,
            suffix_indices,
            ..
        } => native_expr_list_call_result_operation(indices)
            .or_else(|| native_expr_list_call_result_operation(suffix_indices)),
        AssignTarget::NonDirectProperty { holder, .. } => {
            native_lvalue_operand_call_result_operation(holder)
        }
        AssignTarget::NonDirectDynamicProperty {
            holder, property, ..
        } => native_lvalue_operand_call_result_operation(holder)
            .or_else(|| native_lvalue_operand_call_result_operation(property)),
        AssignTarget::DynamicObjectPropertyArrayIndex {
            property, indices, ..
        } => native_lvalue_operand_call_result_operation(property)
            .or_else(|| native_expr_list_call_result_operation(indices)),
        AssignTarget::NonDirectObjectPropertyArrayIndex {
            holder, indices, ..
        } => native_lvalue_operand_call_result_operation(holder)
            .or_else(|| native_expr_list_call_result_operation(indices)),
        AssignTarget::NonDirectObjectPropertyArrayAppend {
            holder,
            indices,
            suffix_indices,
            ..
        } => native_lvalue_operand_call_result_operation(holder)
            .or_else(|| native_expr_list_call_result_operation(indices))
            .or_else(|| native_expr_list_call_result_operation(suffix_indices)),
        AssignTarget::NonDirectDynamicObjectPropertyArrayIndex {
            holder,
            property,
            indices,
            ..
        } => native_lvalue_operand_call_result_operation(holder)
            .or_else(|| native_lvalue_operand_call_result_operation(property))
            .or_else(|| native_expr_list_call_result_operation(indices)),
        AssignTarget::NonDirectDynamicObjectPropertyArrayAppend {
            holder,
            property,
            indices,
            suffix_indices,
            ..
        } => native_lvalue_operand_call_result_operation(holder)
            .or_else(|| native_lvalue_operand_call_result_operation(property))
            .or_else(|| native_expr_list_call_result_operation(indices))
            .or_else(|| native_expr_list_call_result_operation(suffix_indices)),
        AssignTarget::DynamicObjectPropertyArrayAppend {
            property,
            indices,
            suffix_indices,
            ..
        } => native_lvalue_operand_call_result_operation(property)
            .or_else(|| native_expr_list_call_result_operation(indices))
            .or_else(|| native_expr_list_call_result_operation(suffix_indices)),
        AssignTarget::DynamicProperty { property, .. }
        | AssignTarget::ObjectStaticProperty {
            target: property, ..
        } => native_lvalue_operand_call_result_operation(property),
        AssignTarget::Variable { .. }
        | AssignTarget::List { .. }
        | AssignTarget::Property { .. }
        | AssignTarget::StaticProperty { .. }
        | AssignTarget::SelfStaticProperty { .. }
        | AssignTarget::ParentStaticProperty { .. }
        | AssignTarget::LateStaticProperty { .. } => None,
    }
}

fn native_unset_target_call_operation(target: &UnsetTarget) -> Option<NativeCallOperation> {
    match target {
        UnsetTarget::ArrayIndex { index, .. } => native_lvalue_operand_call_result_operation(index),
        UnsetTarget::NestedArrayIndex { indices, .. }
        | UnsetTarget::ObjectPropertyArrayIndex { indices, .. } => {
            native_expr_list_call_result_operation(indices)
        }
        UnsetTarget::NonDirectObjectPropertyArrayIndex {
            holder, indices, ..
        } => native_lvalue_operand_call_result_operation(holder)
            .or_else(|| native_expr_list_call_result_operation(indices)),
        UnsetTarget::DynamicObjectPropertyArrayIndex {
            property, indices, ..
        } => native_lvalue_operand_call_result_operation(property)
            .or_else(|| native_expr_list_call_result_operation(indices)),
        UnsetTarget::NonDirectDynamicObjectPropertyArrayIndex {
            holder,
            property,
            indices,
            ..
        } => native_lvalue_operand_call_result_operation(holder)
            .or_else(|| native_lvalue_operand_call_result_operation(property))
            .or_else(|| native_expr_list_call_result_operation(indices)),
        UnsetTarget::DynamicObjectProperty { property, .. } => {
            native_lvalue_operand_call_result_operation(property)
        }
        UnsetTarget::NonDirectObjectProperty { holder, .. } => {
            native_lvalue_operand_call_result_operation(holder)
        }
        UnsetTarget::NonDirectDynamicObjectProperty {
            holder, property, ..
        } => native_lvalue_operand_call_result_operation(holder)
            .or_else(|| native_lvalue_operand_call_result_operation(property)),
        UnsetTarget::Variable { .. }
        | UnsetTarget::ObjectProperty { .. }
        | UnsetTarget::StaticProperty { .. }
        | UnsetTarget::SelfStaticProperty { .. }
        | UnsetTarget::ParentStaticProperty { .. }
        | UnsetTarget::LateStaticProperty { .. } => None,
    }
}

fn native_expr_list_call_result_operation(exprs: &[Expr]) -> Option<NativeCallOperation> {
    exprs
        .iter()
        .find_map(native_lvalue_operand_call_result_operation)
}

fn native_value_call_operation_for_expr(expr: &Expr) -> Option<NativeCallOperation> {
    let (span, callee, default_blocker, args) = match expr {
        Expr::Call { args, span, .. } => (
            *span,
            NativeCallCallee::DirectNamed,
            NativeCallBlocker::UnknownCalleeDiagnostics,
            args.as_slice(),
        ),
        Expr::DynamicCall { args, span, .. } => (
            *span,
            NativeCallCallee::DynamicExpression,
            NativeCallBlocker::DynamicCallableEvaluation,
            args.as_slice(),
        ),
        Expr::MethodCall { span, .. }
        | Expr::DynamicMethodCall { span, .. }
        | Expr::ParentMethodCall { span, .. }
        | Expr::StaticMethodCall { span, .. }
        | Expr::ObjectStaticMethodCall { span, .. }
        | Expr::SelfMethodCall { span, .. }
        | Expr::LateStaticMethodCall { span, .. } => (
            *span,
            NativeCallCallee::MethodDispatch,
            NativeCallBlocker::MethodDispatch,
            native_call_args_for_expr(expr),
        ),
        Expr::New { args, span, .. } => (
            *span,
            NativeCallCallee::ConstructorDispatch,
            NativeCallBlocker::ConstructorDispatch,
            args.as_slice(),
        ),
        _ => return None,
    };
    let blocker = native_call_argument_list_blocker(args).unwrap_or(default_blocker);

    Some(match callee {
        NativeCallCallee::DirectNamed => NativeCallOperation::direct_named_value(span, blocker),
        NativeCallCallee::DynamicExpression
            if blocker == NativeCallBlocker::DynamicCallableEvaluation =>
        {
            NativeCallOperation::dynamic_value(span)
        }
        NativeCallCallee::DynamicExpression => {
            NativeCallOperation::dynamic_value_with_blocker(span, blocker)
        }
        NativeCallCallee::MethodDispatch if blocker == NativeCallBlocker::MethodDispatch => {
            NativeCallOperation::method_value(span)
        }
        NativeCallCallee::MethodDispatch => {
            NativeCallOperation::method_value_with_blocker(span, blocker)
        }
        NativeCallCallee::ConstructorDispatch => {
            NativeCallOperation::constructor_value(span, blocker)
        }
        NativeCallCallee::FunctionFrame => {
            unreachable!("value-call classifier cannot produce a function-frame callee")
        }
        NativeCallCallee::ClosureFrame => {
            unreachable!("value-call classifier cannot produce a closure-frame callee")
        }
    })
}

fn native_call_root_operation(expr: &Expr) -> Option<NativeCallOperation> {
    match expr {
        Expr::Closure {
            params,
            returns_by_reference,
            span,
            ..
        } => Some(NativeCallOperation::closure_frame(
            *span,
            native_closure_frame_blocker(params, *returns_by_reference),
        )),
        _ => native_value_call_operation_for_expr(expr),
    }
}

fn native_call_args_for_expr(expr: &Expr) -> &[Expr] {
    match expr {
        Expr::MethodCall { args, .. }
        | Expr::DynamicMethodCall { args, .. }
        | Expr::ParentMethodCall { args, .. }
        | Expr::StaticMethodCall { args, .. }
        | Expr::ObjectStaticMethodCall { args, .. }
        | Expr::SelfMethodCall { args, .. }
        | Expr::LateStaticMethodCall { args, .. } => args.as_slice(),
        _ => unreachable!("native call args requested for a non-method call expression"),
    }
}

fn native_call_callee_for_expr(expr: &Expr) -> Option<NativeCallCallee> {
    native_value_call_operation_for_expr(expr).map(|operation| operation.callee)
}

fn native_call_blocker_message(
    backend: NativeCallBackend,
    operation: NativeCallOperation,
) -> &'static str {
    match operation.callee {
        NativeCallCallee::DirectNamed => native_direct_call_blocker_message(backend, operation),
        NativeCallCallee::DynamicExpression => {
            native_dynamic_call_blocker_message(backend, operation)
        }
        NativeCallCallee::MethodDispatch => native_method_call_blocker_message(backend, operation),
        NativeCallCallee::ConstructorDispatch => {
            native_constructor_call_blocker_message(backend, operation)
        }
        NativeCallCallee::FunctionFrame => {
            native_function_frame_blocker_message(backend, operation)
        }
        NativeCallCallee::ClosureFrame => native_closure_frame_blocker_message(backend, operation),
    }
}

fn native_call_operation_diagnostic(
    backend: NativeCallBackend,
    operation: NativeCallOperation,
) -> Diagnostic {
    Diagnostic::new(
        Phase::Codegen,
        operation.span.line,
        operation.span.column,
        native_call_blocker_message(backend, operation),
    )
}

fn native_function_declaration_fallback_diagnostic(
    backend: NativeCallBackend,
    function: &FunctionDecl,
    static_local_rejection: &'static str,
) -> Diagnostic {
    if let Some(span) = find_static_local_span(&function.body) {
        return Diagnostic::new(
            Phase::Codegen,
            span.line,
            span.column,
            static_local_rejection,
        );
    }

    native_call_operation_diagnostic(backend, native_function_frame_call_operation(function))
}

fn native_direct_call_blocker_message(
    backend: NativeCallBackend,
    operation: NativeCallOperation,
) -> &'static str {
    match (operation.result, operation.blocker) {
        (
            NativeCallResult::Value,
            NativeCallBlocker::ArgumentEvaluationCleanup
            | NativeCallBlocker::StatementOperandEvaluationCleanup
            | NativeCallBlocker::ValueOperandEvaluationCleanup
            | NativeCallBlocker::LvalueOperandEvaluationCleanup
            | NativeCallBlocker::ReturnValueOwnership
            | NativeCallBlocker::UnknownCalleeDiagnostics,
        ) => backend.function_call_rejection(),
        (
            NativeCallResult::Reference,
            NativeCallBlocker::ReferenceBinding
            | NativeCallBlocker::ByReferenceArgumentBinding
            | NativeCallBlocker::ReturnValueOwnership,
        ) => backend.reference_assignment_rejection(),
        _ => unreachable!("invalid native direct-call blocker contract"),
    }
}

fn native_dynamic_call_blocker_message(
    backend: NativeCallBackend,
    operation: NativeCallOperation,
) -> &'static str {
    match (operation.result, operation.blocker) {
        (
            NativeCallResult::Value,
            NativeCallBlocker::DynamicCallableEvaluation
            | NativeCallBlocker::ArgumentEvaluationCleanup
            | NativeCallBlocker::StatementOperandEvaluationCleanup
            | NativeCallBlocker::ValueOperandEvaluationCleanup
            | NativeCallBlocker::LvalueOperandEvaluationCleanup
            | NativeCallBlocker::ReturnValueOwnership
            | NativeCallBlocker::UnknownCalleeDiagnostics,
        ) => backend.dynamic_function_call_rejection(),
        (
            NativeCallResult::Reference,
            NativeCallBlocker::ReferenceBinding
            | NativeCallBlocker::DynamicCallableEvaluation
            | NativeCallBlocker::ReturnValueOwnership,
        ) => backend.reference_assignment_rejection(),
        _ => unreachable!("invalid native dynamic-call blocker contract"),
    }
}

fn native_method_call_blocker_message(
    backend: NativeCallBackend,
    operation: NativeCallOperation,
) -> &'static str {
    match (operation.result, operation.blocker) {
        (
            NativeCallResult::Value,
            NativeCallBlocker::MethodDispatch
            | NativeCallBlocker::ArgumentEvaluationCleanup
            | NativeCallBlocker::StatementOperandEvaluationCleanup
            | NativeCallBlocker::ValueOperandEvaluationCleanup
            | NativeCallBlocker::LvalueOperandEvaluationCleanup
            | NativeCallBlocker::ReturnValueOwnership,
        ) => backend.method_call_rejection(),
        (
            NativeCallResult::Reference,
            NativeCallBlocker::ReferenceBinding
            | NativeCallBlocker::ByReferenceArgumentBinding
            | NativeCallBlocker::ReturnValueOwnership,
        ) => backend.reference_assignment_rejection(),
        _ => unreachable!("invalid native method-call blocker contract"),
    }
}

fn native_constructor_call_blocker_message(
    backend: NativeCallBackend,
    operation: NativeCallOperation,
) -> &'static str {
    match (operation.result, operation.blocker) {
        (
            NativeCallResult::Value,
            NativeCallBlocker::ConstructorDispatch
            | NativeCallBlocker::ArgumentEvaluationCleanup
            | NativeCallBlocker::StatementOperandEvaluationCleanup
            | NativeCallBlocker::ValueOperandEvaluationCleanup
            | NativeCallBlocker::LvalueOperandEvaluationCleanup
            | NativeCallBlocker::ReturnValueOwnership,
        ) => backend.object_instantiation_rejection(),
        (
            NativeCallResult::Reference,
            NativeCallBlocker::ReferenceBinding | NativeCallBlocker::ReturnValueOwnership,
        ) => backend.reference_assignment_rejection(),
        _ => unreachable!("invalid native constructor-call blocker contract"),
    }
}

fn native_function_frame_blocker_message(
    backend: NativeCallBackend,
    operation: NativeCallOperation,
) -> &'static str {
    match (operation.result, operation.blocker) {
        (
            NativeCallResult::FrameHandoff,
            NativeCallBlocker::FunctionFrameHandoff
            | NativeCallBlocker::ByReferenceArgumentBinding
            | NativeCallBlocker::VariadicArgumentBinding
            | NativeCallBlocker::ReturnValueOwnership,
        )
        | (NativeCallResult::Value, NativeCallBlocker::ReturnValueOwnership) => {
            backend.function_declaration_rejection()
        }
        _ => unreachable!("invalid native function-frame blocker contract"),
    }
}

fn native_closure_frame_blocker_message(
    backend: NativeCallBackend,
    operation: NativeCallOperation,
) -> &'static str {
    match (operation.result, operation.blocker) {
        (
            NativeCallResult::FrameHandoff,
            NativeCallBlocker::ClosureFrameHandoff
            | NativeCallBlocker::ByReferenceArgumentBinding
            | NativeCallBlocker::VariadicArgumentBinding
            | NativeCallBlocker::ReturnValueOwnership,
        ) => backend.closure_rejection(),
        _ => unreachable!("invalid native closure-frame blocker contract"),
    }
}

pub fn emit_llvm_ir(program: &Program) -> CompileResult<String> {
    let mut generator = LlvmGenerator::default();
    generator.emit_program(program)
}

pub fn emit_assembly(program: &Program) -> CompileResult<String> {
    let ir = emit_llvm_ir(program)?;
    if command_available("clang") {
        return clang_assembly_from_ir(&ir);
    }
    if command_available("llc") {
        return llc_assembly_from_ir(&ir);
    }
    if command_available("cc") {
        let c_source = emit_c_source_for_assembly(program)?;
        return cc_assembly_from_c(&c_source);
    }

    Err(Diagnostic::new(
        Phase::Codegen,
        0,
        0,
        "no assembly backend found; install clang, llc, or cc",
    ))
}

fn is_object_property_array_access_target(target: &AssignTarget) -> bool {
    matches!(
        target,
        AssignTarget::ObjectPropertyArrayIndex { .. }
            | AssignTarget::DynamicObjectPropertyArrayIndex { .. }
            | AssignTarget::NonDirectObjectPropertyArrayIndex { .. }
            | AssignTarget::NonDirectObjectPropertyArrayAppend { .. }
            | AssignTarget::NonDirectDynamicObjectPropertyArrayIndex { .. }
            | AssignTarget::NonDirectDynamicObjectPropertyArrayAppend { .. }
            | AssignTarget::ObjectPropertyArrayAppend { .. }
            | AssignTarget::DynamicObjectPropertyArrayAppend { .. }
    )
}

fn is_request_superglobal_name(name: &str) -> bool {
    matches!(
        name,
        "_SERVER" | "_COOKIE" | "_GET" | "_POST" | "_REQUEST" | "_FILES" | "_SESSION"
    )
}

fn request_superglobal_root_name(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Variable(name, _) if is_request_superglobal_name(name) => Some(name.as_str()),
        _ => None,
    }
}

fn request_superglobal_expr_span(expr: &Expr) -> Option<Span> {
    match expr {
        Expr::Variable(name, span) if is_request_superglobal_name(name) => Some(*span),
        Expr::Index { target, .. } | Expr::AppendIndex { target, .. } => {
            request_superglobal_expr_span(target)
        }
        _ => None,
    }
}

fn is_header_state_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "header"
            | "header_remove"
            | "headers_list"
            | "headers_sent"
            | "http_response_code"
            | "setcookie"
            | "setrawcookie"
    )
}

fn is_session_state_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "session_start"
            | "session_status"
            | "session_cache_limiter"
            | "session_cache_expire"
            | "session_id"
            | "session_write_close"
    )
}

fn is_output_buffer_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "ob_start"
            | "ob_get_level"
            | "ob_get_contents"
            | "ob_get_length"
            | "ob_list_handlers"
            | "ob_get_status"
            | "ob_get_clean"
            | "ob_get_flush"
            | "ob_clean"
            | "ob_flush"
            | "ob_end_clean"
            | "ob_end_flush"
    )
}

fn is_stream_resource_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "fopen"
            | "stream_context_create"
            | "stream_context_get_options"
            | "stream_context_get_params"
            | "stream_context_get_default"
            | "stream_context_set_default"
            | "stream_context_set_option"
            | "stream_context_set_params"
            | "fwrite"
            | "fread"
            | "rewind"
            | "stream_get_contents"
            | "feof"
            | "ftell"
            | "fseek"
            | "fstat"
            | "stream_get_meta_data"
            | "fclose"
            | "opendir"
            | "readdir"
            | "rewinddir"
            | "closedir"
            | "is_uploaded_file"
            | "move_uploaded_file"
    )
}

fn is_static_member_assign_target(target: &AssignTarget) -> bool {
    matches!(
        target,
        AssignTarget::StaticProperty { .. }
            | AssignTarget::ObjectStaticProperty { .. }
            | AssignTarget::SelfStaticProperty { .. }
            | AssignTarget::ParentStaticProperty { .. }
            | AssignTarget::LateStaticProperty { .. }
    )
}

fn is_object_property_array_access_unset_target(target: &UnsetTarget) -> bool {
    matches!(
        target,
        UnsetTarget::ObjectPropertyArrayIndex { .. }
            | UnsetTarget::DynamicObjectPropertyArrayIndex { .. }
            | UnsetTarget::NonDirectObjectPropertyArrayIndex { .. }
            | UnsetTarget::NonDirectDynamicObjectPropertyArrayIndex { .. }
    )
}

fn is_array_access_offset_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Index { target, .. } | Expr::AppendIndex { target, .. } => {
            is_object_offset_expr(target)
        }
        _ => false,
    }
}

fn is_object_offset_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Property { .. }
            | Expr::DynamicProperty { .. }
            | Expr::MethodCall { .. }
            | Expr::DynamicMethodCall { .. }
            | Expr::New { .. }
            | Expr::Clone { .. }
            | Expr::ObjectStaticProperty { .. }
            | Expr::StaticProperty { .. }
            | Expr::SelfStaticProperty { .. }
            | Expr::ParentStaticProperty { .. }
            | Expr::LateStaticProperty { .. }
    )
}

fn find_static_local_span(statements: &[Stmt]) -> Option<Span> {
    for statement in statements {
        match statement {
            Stmt::StaticLocal { span, .. } => return Some(*span),
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                if let Some(span) = find_static_local_span(then_branch) {
                    return Some(span);
                }
                if let Some(span) = find_static_local_span(else_branch) {
                    return Some(span);
                }
            }
            Stmt::While { body, .. }
            | Stmt::DoWhile { body, .. }
            | Stmt::For { body, .. }
            | Stmt::Foreach { body, .. } => {
                if let Some(span) = find_static_local_span(body) {
                    return Some(span);
                }
            }
            Stmt::Switch { cases, .. } => {
                for case in cases {
                    if let Some(span) = find_static_local_span(&case.body) {
                        return Some(span);
                    }
                }
            }
            Stmt::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                if let Some(span) = find_static_local_span(body) {
                    return Some(span);
                }
                for catch in catches {
                    if let Some(span) = find_static_local_span(&catch.body) {
                        return Some(span);
                    }
                }
                if let Some(finally_body) = finally_body {
                    if let Some(span) = find_static_local_span(finally_body) {
                        return Some(span);
                    }
                }
            }
            Stmt::Function(function) => {
                if let Some(span) = find_static_local_span(&function.body) {
                    return Some(span);
                }
            }
            Stmt::Class(class) => {
                for member in &class.members {
                    if let ClassMember::Method(method) = member {
                        if let Some(span) = find_static_local_span(&method.function.body) {
                            return Some(span);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeRuntimeIrTarget {
    Pointer32,
    Pointer64,
}

impl NativeRuntimeIrTarget {
    fn host() -> Self {
        if usize::BITS == 32 {
            Self::Pointer32
        } else {
            Self::Pointer64
        }
    }

    fn usize_ir_type(self) -> &'static str {
        match self {
            Self::Pointer32 => "i32",
            Self::Pointer64 => "i64",
        }
    }
}

pub fn native_runtime_scalar_echo_probe_ir() -> String {
    native_runtime_scalar_echo_probe_ir_for_target(NativeRuntimeIrTarget::host())
}

pub fn native_runtime_scalar_echo_probe_ir_for_target(target: NativeRuntimeIrTarget) -> String {
    let usize_type = target.usize_ir_type();
    [
        "; generated by phpc native runtime helper probe",
        "; this is a dependency sketch, not production lowering or linked execution",
        "%phpc.NativeScalarValue = type { i8, i8, [6 x i8], i64, double }",
        &format!("%phpc.NativeByteBuffer = type {{ ptr, {usize_type}, {usize_type} }}"),
        "%phpc.NativeStringHandle = type { ptr }",
        "%phpc.NativeValueHandle = type { ptr }",
        "%phpc.NativeDiagnosticHandle = type { ptr }",
        "%phpc.NativeStringConversionResult = type { %phpc.NativeByteBuffer, %phpc.NativeDiagnosticHandle }",
        "%phpc.NativeArrayHandle = type { ptr }",
        "%phpc.NativeObjectHandle = type { ptr }",
        "%phpc.NativeResourceHandle = type { ptr }",
        "%phpc.NativeReferenceHandle = type { ptr }",
        "%phpc.NativeRequestStateHandle = type { ptr }",
        "%phpc.NativeRequestStateOperationResult = type { %phpc.NativeValueHandle, %phpc.NativeArrayHandle, i8, i8, i8, i8 }",
        "%phpc.NativeRequestStateKeyResult = type { %phpc.NativeByteBuffer, i8 }",
        "@phpc.probe.bytes = private unnamed_addr constant [4 x i8] c\"heap\"",
        "@phpc.probe.string = private unnamed_addr constant [7 x i8] c\"php\\00abi\"",
        "@phpc.probe.invalid = private unnamed_addr constant [1 x i8] c\"\\FF\"",
        "@phpc.probe.request.bag = private unnamed_addr constant [4 x i8] c\"_GET\"",
        "",
        &format!("declare {usize_type} @phpc_native_scalar_echo_len(%phpc.NativeScalarValue)"),
        &format!(
            "declare {usize_type} @phpc_native_scalar_echo_write(%phpc.NativeScalarValue, ptr, {usize_type})"
        ),
        "declare %phpc.NativeByteBuffer @phpc_native_scalar_echo_bytes(%phpc.NativeScalarValue)",
        &format!(
            "declare %phpc.NativeByteBuffer @phpc_native_byte_buffer_from_bytes(ptr, {usize_type})"
        ),
        "declare void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer)",
        &format!(
            "declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, {usize_type})"
        ),
        &format!("declare {usize_type} @phpc_native_string_len(%phpc.NativeStringHandle)"),
        "declare ptr @phpc_native_string_bytes(%phpc.NativeStringHandle)",
        "declare %phpc.NativeByteBuffer @phpc_native_string_clone_bytes(%phpc.NativeStringHandle)",
        "declare void @phpc_native_string_free(%phpc.NativeStringHandle)",
        "declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)",
        "declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)",
        "declare %phpc.NativeStringConversionResult @phpc_native_value_to_string_bytes(%phpc.NativeValueHandle)",
        "declare %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle)",
        &format!("declare {usize_type} @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)"),
        "declare void @phpc_native_value_free(%phpc.NativeValueHandle)",
        "declare %phpc.NativeStringConversionResult @phpc_native_reference_to_string_bytes(%phpc.NativeReferenceHandle)",
        "declare void @phpc_native_string_conversion_result_free(%phpc.NativeStringConversionResult)",
        &format!(
            "declare {usize_type} @phpc_native_diagnostic_message_len(%phpc.NativeDiagnosticHandle)"
        ),
        "declare %phpc.NativeByteBuffer @phpc_native_diagnostic_message_clone_bytes(%phpc.NativeDiagnosticHandle)",
        &format!(
            "declare {usize_type} @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)"
        ),
        &format!("declare {usize_type} @phpc_native_diagnostic_count(%phpc.NativeDiagnosticHandle)"),
        &format!(
            "declare i8 @phpc_native_diagnostic_severity_at(%phpc.NativeDiagnosticHandle, {usize_type})"
        ),
        "declare i1 @phpc_native_diagnostic_contains_severity(%phpc.NativeDiagnosticHandle, i8)",
        "declare i1 @phpc_native_diagnostic_severity_is_known(i8)",
        "declare i32 @phpc_native_diagnostic_severity_mask(i8)",
        "declare i32 @phpc_native_diagnostic_error_control_suppression_mask()",
        "declare void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle)",
        "declare %phpc.NativeArrayHandle @phpc_native_array_null()",
        "declare %phpc.NativeArrayHandle @phpc_native_array_empty()",
        "declare i1 @phpc_native_array_is_null(%phpc.NativeArrayHandle)",
        &format!("declare {usize_type} @phpc_native_array_len(%phpc.NativeArrayHandle)"),
        "declare i1 @phpc_native_array_append_scalar(%phpc.NativeArrayHandle, %phpc.NativeScalarValue)",
        "declare i1 @phpc_native_array_append_value(%phpc.NativeArrayHandle, %phpc.NativeValueHandle)",
        "declare %phpc.NativeValueHandle @phpc_native_array_read_int(%phpc.NativeArrayHandle, i64)",
        "declare void @phpc_native_array_free(%phpc.NativeArrayHandle)",
        "declare %phpc.NativeObjectHandle @phpc_native_object_null()",
        "declare i1 @phpc_native_object_is_null(%phpc.NativeObjectHandle)",
        "declare %phpc.NativeResourceHandle @phpc_native_resource_null()",
        "declare i1 @phpc_native_resource_is_null(%phpc.NativeResourceHandle)",
        "declare %phpc.NativeReferenceHandle @phpc_native_reference_null()",
        "declare i1 @phpc_native_reference_is_null(%phpc.NativeReferenceHandle)",
        "declare %phpc.NativeRequestStateHandle @phpc_native_request_state_null()",
        "declare %phpc.NativeRequestStateHandle @phpc_native_request_state_empty()",
        "declare i1 @phpc_native_request_state_is_null(%phpc.NativeRequestStateHandle)",
        "declare %phpc.NativeRequestStateKeyResult @phpc_native_request_state_key_from_scalar(%phpc.NativeScalarValue)",
        &format!("declare %phpc.NativeRequestStateOperationResult @phpc_native_request_state_superglobal_operation(%phpc.NativeRequestStateHandle, i8, ptr, {usize_type}, ptr, {usize_type}, i8)"),
        "declare %phpc.NativeValueHandle @phpc_native_request_state_superglobal_snapshot_value(%phpc.NativeRequestStateHandle, %phpc.NativeStringHandle)",
        "declare i1 @phpc_native_request_state_rebuild_request_from_order(%phpc.NativeRequestStateHandle, %phpc.NativeStringHandle)",
        "declare i1 @phpc_native_request_state_rebuild_request_from_policy(%phpc.NativeRequestStateHandle, %phpc.NativeStringHandle, %phpc.NativeStringHandle)",
        "declare void @phpc_native_request_state_operation_result_free(%phpc.NativeRequestStateOperationResult)",
        "declare void @phpc_native_request_state_free(%phpc.NativeRequestStateHandle)",
        "",
        &format!("define {usize_type} @phpc_probe_scalar_echo_len() {{"),
        "entry:",
        "  %value = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %with_payload = insertvalue %phpc.NativeScalarValue %value, i64 42, 3",
        &format!(
            "  %len = call {usize_type} @phpc_native_scalar_echo_len(%phpc.NativeScalarValue %with_payload)"
        ),
        &format!("  ret {usize_type} %len"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_scalar_echo_owned_bytes() {{"),
        "entry:",
        "  %value = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %with_payload = insertvalue %phpc.NativeScalarValue %value, i64 -123, 3",
        "  %buffer = call %phpc.NativeByteBuffer @phpc_native_scalar_echo_bytes(%phpc.NativeScalarValue %with_payload)",
        "  %len = extractvalue %phpc.NativeByteBuffer %buffer, 1",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %buffer)",
        &format!("  ret {usize_type} %len"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_byte_buffer_from_bytes() {{"),
        "entry:",
        &format!(
            "  %bytes = getelementptr inbounds [4 x i8], ptr @phpc.probe.bytes, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %buffer = call %phpc.NativeByteBuffer @phpc_native_byte_buffer_from_bytes(ptr %bytes, {usize_type} 4)"
        ),
        "  %len = extractvalue %phpc.NativeByteBuffer %buffer, 1",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %buffer)",
        &format!("  ret {usize_type} %len"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_string_handle_roundtrip() {{"),
        "entry:",
        &format!(
            "  %bytes = getelementptr inbounds [7 x i8], ptr @phpc.probe.string, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %string = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr %bytes, {usize_type} 7)"
        ),
        &format!(
            "  %len = call {usize_type} @phpc_native_string_len(%phpc.NativeStringHandle %string)"
        ),
        "  %raw = call ptr @phpc_native_string_bytes(%phpc.NativeStringHandle %string)",
        "  %clone = call %phpc.NativeByteBuffer @phpc_native_string_clone_bytes(%phpc.NativeStringHandle %string)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %clone)",
        "  call void @phpc_native_string_free(%phpc.NativeStringHandle %string)",
        &format!("  ret {usize_type} %len"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_string_handle_to_value_echo() {{"),
        "entry:",
        &format!(
            "  %bytes = getelementptr inbounds [7 x i8], ptr @phpc.probe.string, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %string = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr %bytes, {usize_type} 7)"
        ),
        "  %value = call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle %string)",
        "  %buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %value)",
        "  %len = extractvalue %phpc.NativeByteBuffer %buffer, 1",
        &format!(
            "  %written = call {usize_type} @phpc_native_value_echo_stdout(%phpc.NativeValueHandle %value)"
        ),
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %buffer)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %value)",
        "  call void @phpc_native_string_free(%phpc.NativeStringHandle %string)",
        &format!("  ret {usize_type} %len"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_string_to_value_diagnostic() {{"),
        "entry:",
        "  %diagnostic_slot = alloca %phpc.NativeDiagnosticHandle",
        &format!(
            "  %bytes = getelementptr inbounds [1 x i8], ptr @phpc.probe.invalid, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %string = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr %bytes, {usize_type} 1)"
        ),
        "  %value = call %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle %string, ptr %diagnostic_slot)",
        "  %diagnostic = load %phpc.NativeDiagnosticHandle, ptr %diagnostic_slot",
        &format!(
            "  %len = call {usize_type} @phpc_native_diagnostic_message_len(%phpc.NativeDiagnosticHandle %diagnostic)"
        ),
        "  %message = call %phpc.NativeByteBuffer @phpc_native_diagnostic_message_clone_bytes(%phpc.NativeDiagnosticHandle %diagnostic)",
        &format!(
            "  %reported = call {usize_type} @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle %diagnostic)"
        ),
        &format!(
            "  %diagnostic_count = call {usize_type} @phpc_native_diagnostic_count(%phpc.NativeDiagnosticHandle %diagnostic)"
        ),
        &format!(
            "  %severity = call i8 @phpc_native_diagnostic_severity_at(%phpc.NativeDiagnosticHandle %diagnostic, {usize_type} 0)"
        ),
        "  %has_error = call i1 @phpc_native_diagnostic_contains_severity(%phpc.NativeDiagnosticHandle %diagnostic, i8 3)",
        "  %severity_known = call i1 @phpc_native_diagnostic_severity_is_known(i8 %severity)",
        "  %severity_mask = call i32 @phpc_native_diagnostic_severity_mask(i8 %severity)",
        "  %suppression_mask = call i32 @phpc_native_diagnostic_error_control_suppression_mask()",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %message)",
        "  call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %diagnostic)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %value)",
        "  call void @phpc_native_string_free(%phpc.NativeStringHandle %string)",
        &format!("  ret {usize_type} %len"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_string_to_value_diagnostic_branch() {{"),
        "entry:",
        "  %diagnostic_slot = alloca %phpc.NativeDiagnosticHandle",
        "  store %phpc.NativeDiagnosticHandle zeroinitializer, ptr %diagnostic_slot",
        &format!(
            "  %bytes = getelementptr inbounds [1 x i8], ptr @phpc.probe.invalid, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %string = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr %bytes, {usize_type} 1)"
        ),
        "  %value = call %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle %string, ptr %diagnostic_slot)",
        "  %value_ptr = extractvalue %phpc.NativeValueHandle %value, 0",
        "  %value_failed = icmp eq ptr %value_ptr, null",
        "  br i1 %value_failed, label %report_diagnostic, label %echo_value",
        "",
        "report_diagnostic:",
        "  %diagnostic = load %phpc.NativeDiagnosticHandle, ptr %diagnostic_slot",
        &format!(
            "  %diagnostic_len = call {usize_type} @phpc_native_diagnostic_message_len(%phpc.NativeDiagnosticHandle %diagnostic)"
        ),
        "  %message = call %phpc.NativeByteBuffer @phpc_native_diagnostic_message_clone_bytes(%phpc.NativeDiagnosticHandle %diagnostic)",
        &format!(
            "  %reported = call {usize_type} @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle %diagnostic)"
        ),
        &format!(
            "  %diagnostic_count = call {usize_type} @phpc_native_diagnostic_count(%phpc.NativeDiagnosticHandle %diagnostic)"
        ),
        &format!(
            "  %severity = call i8 @phpc_native_diagnostic_severity_at(%phpc.NativeDiagnosticHandle %diagnostic, {usize_type} 0)"
        ),
        "  %has_error = call i1 @phpc_native_diagnostic_contains_severity(%phpc.NativeDiagnosticHandle %diagnostic, i8 3)",
        "  %severity_known = call i1 @phpc_native_diagnostic_severity_is_known(i8 %severity)",
        "  %severity_mask = call i32 @phpc_native_diagnostic_severity_mask(i8 %severity)",
        "  %suppression_mask = call i32 @phpc_native_diagnostic_error_control_suppression_mask()",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %message)",
        "  call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %diagnostic)",
        "  br label %cleanup",
        "",
        "echo_value:",
        &format!(
            "  %written = call {usize_type} @phpc_native_value_echo_stdout(%phpc.NativeValueHandle %value)"
        ),
        "  br label %cleanup",
        "",
        "cleanup:",
        &format!(
            "  %result = phi {usize_type} [ %diagnostic_len, %report_diagnostic ], [ %written, %echo_value ]"
        ),
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %value)",
        "  call void @phpc_native_string_free(%phpc.NativeStringHandle %string)",
        &format!("  ret {usize_type} %result"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_value_to_string_conversion_result() {{"),
        "entry:",
        &format!(
            "  %bytes = getelementptr inbounds [7 x i8], ptr @phpc.probe.string, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %string = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr %bytes, {usize_type} 7)"
        ),
        "  %value = call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle %string)",
        "  %conversion = call %phpc.NativeStringConversionResult @phpc_native_value_to_string_bytes(%phpc.NativeValueHandle %value)",
        "  %converted_bytes = extractvalue %phpc.NativeStringConversionResult %conversion, 0",
        &format!("  %len = extractvalue %phpc.NativeByteBuffer %converted_bytes, 1"),
        "  call void @phpc_native_string_conversion_result_free(%phpc.NativeStringConversionResult %conversion)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %value)",
        "  call void @phpc_native_string_free(%phpc.NativeStringHandle %string)",
        &format!("  ret {usize_type} %len"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_reference_string_conversion_diagnostic() {{"),
        "entry:",
        "  %reference = call %phpc.NativeReferenceHandle @phpc_native_reference_null()",
        "  %conversion = call %phpc.NativeStringConversionResult @phpc_native_reference_to_string_bytes(%phpc.NativeReferenceHandle %reference)",
        "  %diagnostic = extractvalue %phpc.NativeStringConversionResult %conversion, 1",
        &format!(
            "  %len = call {usize_type} @phpc_native_diagnostic_message_len(%phpc.NativeDiagnosticHandle %diagnostic)"
        ),
        "  call void @phpc_native_string_conversion_result_free(%phpc.NativeStringConversionResult %conversion)",
        &format!("  ret {usize_type} %len"),
        "}",
        "",
        "define i1 @phpc_probe_container_handle_null_shapes() {",
        "entry:",
        "  %array = call %phpc.NativeArrayHandle @phpc_native_array_null()",
        "  %array_is_null = call i1 @phpc_native_array_is_null(%phpc.NativeArrayHandle %array)",
        "  %object = call %phpc.NativeObjectHandle @phpc_native_object_null()",
        "  %object_is_null = call i1 @phpc_native_object_is_null(%phpc.NativeObjectHandle %object)",
        "  %resource = call %phpc.NativeResourceHandle @phpc_native_resource_null()",
        "  %resource_is_null = call i1 @phpc_native_resource_is_null(%phpc.NativeResourceHandle %resource)",
        "  %reference = call %phpc.NativeReferenceHandle @phpc_native_reference_null()",
        "  %reference_is_null = call i1 @phpc_native_reference_is_null(%phpc.NativeReferenceHandle %reference)",
        "  %left = and i1 %array_is_null, %object_is_null",
        "  %right = and i1 %resource_is_null, %reference_is_null",
        "  %all = and i1 %left, %right",
        "  ret i1 %all",
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_array_handle_empty_len() {{"),
        "entry:",
        "  %array = call %phpc.NativeArrayHandle @phpc_native_array_empty()",
        "  %array_is_null = call i1 @phpc_native_array_is_null(%phpc.NativeArrayHandle %array)",
        &format!(
            "  %len = call {usize_type} @phpc_native_array_len(%phpc.NativeArrayHandle %array)"
        ),
        "  call void @phpc_native_array_free(%phpc.NativeArrayHandle %array)",
        &format!("  %nullable_len = zext i1 %array_is_null to {usize_type}"),
        &format!("  %result = add {usize_type} %len, %nullable_len"),
        &format!("  ret {usize_type} %result"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_array_handle_append_read() {{"),
        "entry:",
        "  %array = call %phpc.NativeArrayHandle @phpc_native_array_empty()",
        "  %scalar_tag = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %scalar = insertvalue %phpc.NativeScalarValue %scalar_tag, i64 7, 3",
        "  %appended_scalar = call i1 @phpc_native_array_append_scalar(%phpc.NativeArrayHandle %array, %phpc.NativeScalarValue %scalar)",
        &format!(
            "  %bytes = getelementptr inbounds [7 x i8], ptr @phpc.probe.string, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %string = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr %bytes, {usize_type} 7)"
        ),
        "  %value = call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle %string)",
        "  %appended_value = call i1 @phpc_native_array_append_value(%phpc.NativeArrayHandle %array, %phpc.NativeValueHandle %value)",
        "  %read = call %phpc.NativeValueHandle @phpc_native_array_read_int(%phpc.NativeArrayHandle %array, i64 1)",
        "  %buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %read)",
        "  %len = extractvalue %phpc.NativeByteBuffer %buffer, 1",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %buffer)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %read)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %value)",
        "  call void @phpc_native_string_free(%phpc.NativeStringHandle %string)",
        "  call void @phpc_native_array_free(%phpc.NativeArrayHandle %array)",
        &format!("  ret {usize_type} %len"),
        "}",
        "",
        "define i1 @phpc_probe_request_state_handle_null_shape() {",
        "entry:",
        "  %request_state = call %phpc.NativeRequestStateHandle @phpc_native_request_state_null()",
        "  %request_state_is_null = call i1 @phpc_native_request_state_is_null(%phpc.NativeRequestStateHandle %request_state)",
        "  ret i1 %request_state_is_null",
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_request_state_empty_missing_value() {{"),
        "entry:",
        "  %request_state = call %phpc.NativeRequestStateHandle @phpc_native_request_state_empty()",
        "  %key_tag = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %key_scalar = insertvalue %phpc.NativeScalarValue %key_tag, i64 0, 3",
        "  %key = call %phpc.NativeRequestStateKeyResult @phpc_native_request_state_key_from_scalar(%phpc.NativeScalarValue %key_scalar)",
        "  %key_buffer = extractvalue %phpc.NativeRequestStateKeyResult %key, 0",
        "  %key_status = extractvalue %phpc.NativeRequestStateKeyResult %key, 1",
        "  %key_ptr = extractvalue %phpc.NativeByteBuffer %key_buffer, 0",
        "  %key_len = extractvalue %phpc.NativeByteBuffer %key_buffer, 1",
        &format!(
            "  %bag = getelementptr inbounds [4 x i8], ptr @phpc.probe.request.bag, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %result = call %phpc.NativeRequestStateOperationResult @phpc_native_request_state_superglobal_operation(%phpc.NativeRequestStateHandle %request_state, i8 1, ptr %bag, {usize_type} 4, ptr %key_ptr, {usize_type} %key_len, i8 %key_status)"
        ),
        "  %is_set = extractvalue %phpc.NativeRequestStateOperationResult %result, 2",
        "  %status = extractvalue %phpc.NativeRequestStateOperationResult %result, 4",
        "  %exists = extractvalue %phpc.NativeRequestStateOperationResult %result, 5",
        &format!("  %is_set_len = zext i8 %is_set to {usize_type}"),
        &format!("  %status_len = zext i8 %status to {usize_type}"),
        &format!("  %exists_len = zext i8 %exists to {usize_type}"),
        &format!("  %partial = add {usize_type} %status_len, %exists_len"),
        &format!("  %total = add {usize_type} %partial, %is_set_len"),
        "  call void @phpc_native_request_state_operation_result_free(%phpc.NativeRequestStateOperationResult %result)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %key_buffer)",
        "  call void @phpc_native_request_state_free(%phpc.NativeRequestStateHandle %request_state)",
        &format!("  ret {usize_type} %total"),
        "}",
        "",
    ]
    .join("\n")
}

#[derive(Default)]
struct LlvmGenerator {
    strings: Vec<(String, String)>,
    body: Vec<String>,
    variables: HashMap<String, IrValue>,
    known_ints: HashMap<String, KnownInt>,
    known_floats: HashMap<String, KnownFloat>,
    known_strings: HashMap<String, KnownString>,
    string_lengths: HashMap<String, String>,
    known_bools: HashMap<String, KnownBool>,
    next_string: usize,
    next_temp: usize,
    next_label: usize,
    uses_strcmp: bool,
    uses_native_value_echo_stdout: bool,
    uses_native_string_int_operation: bool,
    uses_native_value_offset_operation: bool,
}

#[derive(Debug, Clone)]
enum IrValue {
    Int(String),
    Float(String),
    String(String),
    StringPtr(String),
    NativeValue(String),
    Bool(bool),
    BoolExpr(String),
    Null,
}

#[derive(Debug, Clone)]
struct KnownInt {
    values: Vec<i64>,
}

impl KnownInt {
    fn one(value: i64) -> Self {
        Self {
            values: vec![value],
        }
    }

    fn from_values(values: impl IntoIterator<Item = i64>) -> Option<Self> {
        let mut unique = Vec::new();
        for value in values {
            if unique.contains(&value) {
                continue;
            }
            unique.push(value);
            if unique.len() > MAX_KNOWN_INT_VALUES {
                return None;
            }
        }
        if unique.is_empty() {
            None
        } else {
            Some(Self { values: unique })
        }
    }

    fn values(&self) -> &[i64] {
        &self.values
    }

    fn is_single(&self) -> bool {
        self.values.len() == 1
    }

    fn is_single_value(&self, expected: i64) -> bool {
        matches!(self.values.as_slice(), [value] if *value == expected)
    }
}

#[derive(Debug, Clone)]
struct KnownFloat {
    values: Vec<f64>,
}

impl KnownFloat {
    fn one(value: f64) -> Self {
        Self {
            values: vec![value],
        }
    }

    fn from_values(values: impl IntoIterator<Item = f64>) -> Option<Self> {
        let mut unique = Vec::new();
        for value in values {
            if unique.iter().any(|existing| existing == &value) {
                continue;
            }
            unique.push(value);
            if unique.len() > MAX_KNOWN_FLOAT_VALUES {
                return None;
            }
        }
        if unique.is_empty() {
            None
        } else {
            Some(Self { values: unique })
        }
    }

    fn values(&self) -> &[f64] {
        &self.values
    }

    fn is_single(&self) -> bool {
        self.values.len() == 1
    }
}

#[derive(Debug, Clone)]
struct KnownString {
    values: Vec<String>,
}

impl KnownString {
    fn one(value: String) -> Self {
        Self {
            values: vec![value],
        }
    }

    fn from_values(values: impl IntoIterator<Item = String>) -> Option<Self> {
        let mut unique = Vec::new();
        for value in values {
            if unique.contains(&value) {
                continue;
            }
            unique.push(value);
            if unique.len() > MAX_KNOWN_STRING_VALUES {
                return None;
            }
        }
        if unique.is_empty() {
            None
        } else {
            Some(Self { values: unique })
        }
    }

    fn values(&self) -> &[String] {
        &self.values
    }

    fn is_single(&self) -> bool {
        self.values.len() == 1
    }
}

#[derive(Debug, Clone)]
struct KnownBool {
    values: Vec<bool>,
}

impl KnownBool {
    fn one(value: bool) -> Self {
        Self {
            values: vec![value],
        }
    }

    fn from_values(values: impl IntoIterator<Item = bool>) -> Option<Self> {
        let mut unique = Vec::new();
        for value in values {
            if unique.contains(&value) {
                continue;
            }
            unique.push(value);
        }
        if unique.is_empty() {
            None
        } else {
            Some(Self { values: unique })
        }
    }

    fn values(&self) -> &[bool] {
        &self.values
    }

    fn is_single(&self) -> bool {
        self.values.len() == 1
    }
}

impl LlvmGenerator {
    fn emit_program(&mut self, program: &Program) -> CompileResult<String> {
        for stmt in &program.statements {
            self.emit_statement(stmt)?;
        }

        let mut output = String::new();
        output.push_str("; generated by phpc milestone 1\n");
        if self.uses_native_value_echo_stdout {
            output.push_str("%phpc.NativeStringHandle = type { ptr }\n");
            output.push_str("%phpc.NativeValueHandle = type { ptr }\n");
            output.push_str("%phpc.NativeDiagnosticHandle = type { ptr }\n");
        }
        if self.uses_native_string_int_operation {
            if !self.uses_native_value_echo_stdout {
                output.push_str("%phpc.NativeValueHandle = type { ptr }\n");
                output.push_str("%phpc.NativeDiagnosticHandle = type { ptr }\n");
            }
            output.push_str("%phpc.NativeScalarValue = type { i8, i8, [6 x i8], i64, double }\n");
        }
        if self.uses_native_value_offset_operation {
            let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
            if !self.uses_native_value_echo_stdout && !self.uses_native_string_int_operation {
                output.push_str("%phpc.NativeValueHandle = type { ptr }\n");
                output.push_str("%phpc.NativeDiagnosticHandle = type { ptr }\n");
            }
            output.push_str(&format!(
                "%phpc.NativeByteBuffer = type {{ ptr, {usize_type}, {usize_type} }}\n"
            ));
            output.push_str("%phpc.NativeStringConversionResult = type { %phpc.NativeByteBuffer, %phpc.NativeDiagnosticHandle }\n");
        }
        output.push_str("declare i32 @printf(ptr, ...)\n");
        if self.uses_strcmp {
            output.push_str("declare i32 @strcmp(ptr, ptr)\n");
        }
        if self.uses_native_value_echo_stdout {
            let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
            output.push_str(&format!(
                "declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, {usize_type})\n"
            ));
            output.push_str("declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)\n");
            output.push_str(&format!(
                "declare {usize_type} @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)\n"
            ));
            output.push_str("declare void @phpc_native_value_free(%phpc.NativeValueHandle)\n");
            output.push_str(&format!(
                "declare {usize_type} @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)\n"
            ));
            output.push_str(
                "declare void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle)\n",
            );
            output.push_str("declare void @phpc_native_string_free(%phpc.NativeStringHandle)\n");
        }
        if self.uses_native_string_int_operation {
            let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
            output.push_str("declare %phpc.NativeScalarValue @phpc_native_null()\n");
            output.push_str("declare %phpc.NativeScalarValue @phpc_native_bool(i1)\n");
            output.push_str("declare %phpc.NativeScalarValue @phpc_native_int(i64)\n");
            output.push_str("declare %phpc.NativeScalarValue @phpc_native_float(double)\n");
            output.push_str("declare %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue)\n");
            output.push_str(&format!(
                "declare %phpc.NativeValueHandle @phpc_native_value_from_string_bytes_with_diagnostic(ptr, {usize_type}, ptr)\n"
            ));
            output.push_str("declare i64 @phpc_native_value_to_int64_with_diagnostic(%phpc.NativeValueHandle, i8, ptr)\n");
            output.push_str("declare i64 @phpc_native_value_string_int_operation_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeValueHandle, i64, i64, i8, i8, ptr)\n");
            if !self.uses_native_value_echo_stdout {
                output.push_str("declare void @phpc_native_value_free(%phpc.NativeValueHandle)\n");
                output.push_str(&format!(
                    "declare {usize_type} @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)\n"
                ));
                output.push_str(
                    "declare void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle)\n",
                );
            }
        }
        if self.uses_native_value_offset_operation {
            output.push_str("declare %phpc.NativeValueHandle @phpc_native_value_offset_operation_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeValueHandle, i8, ptr)\n");
            output.push_str("declare i1 @phpc_native_value_bool_with_diagnostic(%phpc.NativeValueHandle, ptr)\n");
            output.push_str("declare %phpc.NativeStringConversionResult @phpc_native_value_to_string_bytes(%phpc.NativeValueHandle)\n");
            output.push_str("declare void @phpc_native_string_conversion_result_free(%phpc.NativeStringConversionResult)\n");
        }
        output.push('\n');
        output.push_str("@.fmt_int = private unnamed_addr constant [5 x i8] c\"%lld\\00\"\n");
        output.push_str("@.fmt_float = private unnamed_addr constant [3 x i8] c\"%g\\00\"\n");
        output.push_str("@.fmt_str = private unnamed_addr constant [3 x i8] c\"%s\\00\"\n");

        for (name, text) in &self.strings {
            output.push_str(&format!(
                "@{name} = private unnamed_addr constant [{} x i8] c\"{}\"\n",
                text.as_bytes().len() + 1,
                llvm_c_string(text)
            ));
        }

        output.push_str("\ndefine i32 @main() {\nentry:\n");
        for line in &self.body {
            if !line.ends_with(':') {
                output.push_str("  ");
            }
            output.push_str(line);
            output.push('\n');
        }
        output.push_str("  ret i32 0\n}\n");
        Ok(output)
    }

    fn emit_statement(&mut self, stmt: &Stmt) -> CompileResult<()> {
        if let Some(operation) = native_statement_operand_call_operation(stmt) {
            return Err(self.unsupported_call_operation(operation));
        }

        match stmt {
            Stmt::Namespace { span, .. } | Stmt::Use { span, .. } => {
                Err(self.unsupported(*span, LLVM_NAMESPACE_REJECTION))
            }
            Stmt::Echo { exprs, .. } => {
                for (index, expr) in exprs.iter().enumerate() {
                    let value = match self.emit_expr(expr) {
                        Ok(value) => value,
                        Err(error) => {
                            return Err(self.unsupported_unemitted_statement_operands_or_original(
                                &exprs[index + 1..],
                                error,
                            ));
                        }
                    };
                    self.emit_echo(value);
                }
                Ok(())
            }
            Stmt::Print { expr, .. } => {
                let value = self.emit_expr(expr)?;
                self.emit_print(value);
                Ok(())
            }
            Stmt::Assign { target, expr, .. } => self.emit_assignment(target, expr),
            Stmt::ReferenceAssign {
                target,
                source,
                span,
            } => {
                if let Some(operation) = native_reference_assignment_call_operation(target, source)
                {
                    return Err(self.unsupported_call_operation(operation));
                }

                Err(self.unsupported(*span, LLVM_REFERENCE_ASSIGNMENT_REJECTION))
            }
            Stmt::CompoundAssign { target, span, .. }
            | Stmt::IncrementDecrement { target, span, .. }
            | Stmt::NullCoalesceAssign { target, span, .. } => {
                if let Some(operation) = native_assignment_target_call_operation(target) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, LLVM_ARRAY_ACCESS_REJECTION));
                }
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Stmt::Expr { expr, .. } => {
                let value = self.emit_expr(expr)?;
                if let IrValue::NativeValue(value) = value {
                    self.body.push(format!(
                        "call void @phpc_native_value_free(%phpc.NativeValueHandle {value})"
                    ));
                }
                Ok(())
            }
            Stmt::Function(function) => Err(native_function_declaration_fallback_diagnostic(
                NativeCallBackend::Llvm,
                function,
                LLVM_STATIC_LOCAL_REJECTION,
            )),
            Stmt::Interface(interface) => {
                Err(self.unsupported(interface.span, LLVM_INTERFACE_REJECTION))
            }
            Stmt::Trait(trait_decl) => Err(self.unsupported(trait_decl.span, LLVM_TRAIT_REJECTION)),
            Stmt::Enum(enum_decl) => Err(self.unsupported(enum_decl.span, LLVM_ENUM_REJECTION)),
            Stmt::Class(class) => {
                if let Some(span) = find_static_local_span(std::slice::from_ref(stmt)) {
                    return Err(self.unsupported(span, LLVM_STATIC_LOCAL_REJECTION));
                }
                Err(self.unsupported(class.span, LLVM_OBJECT_CLASS_REJECTION))
            }
            Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::DoWhile { span, .. }
            | Stmt::For { span, .. }
            | Stmt::Switch { span, .. }
            | Stmt::Goto { span, .. }
            | Stmt::Label { span, .. }
            | Stmt::Break { span, .. }
            | Stmt::Continue { span, .. } => {
                Err(self.unsupported(*span, LLVM_CONTROL_FLOW_REJECTION))
            }
            Stmt::Foreach { span, .. } => Err(self.unsupported(*span, LLVM_ARRAY_REJECTION)),
            Stmt::UnsetVariable { span, .. } => {
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Stmt::UnsetStaticProperty { span, .. }
            | Stmt::UnsetSelfStaticProperty { span, .. }
            | Stmt::UnsetParentStaticProperty { span, .. }
            | Stmt::UnsetLateStaticProperty { span, .. } => {
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Stmt::UnsetObjectProperty { span, .. } => {
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Stmt::UnsetDynamicObjectProperty { span, .. } => {
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Stmt::UnsetArrayIndex { span, .. } => {
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            Stmt::UnsetNestedArrayIndex { span, .. } => {
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            Stmt::UnsetMany { targets, span } => {
                if targets
                    .iter()
                    .any(is_object_property_array_access_unset_target)
                {
                    return Err(self.unsupported(*span, LLVM_ARRAY_ACCESS_REJECTION));
                }
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Stmt::ConstDeclaration { span, .. } => {
                Err(self.unsupported(*span, LLVM_GLOBAL_CONSTANT_REJECTION))
            }
            Stmt::Require { span, .. } | Stmt::Include { span, .. } => {
                Err(self.unsupported(*span, LLVM_REQUIRE_REJECTION))
            }
            Stmt::Throw { span, .. } => Err(self.unsupported(*span, LLVM_EXCEPTION_REJECTION)),
            Stmt::Try { span, .. } => Err(self.unsupported(*span, LLVM_TRY_BLOCK_REJECTION)),
            Stmt::Return { span, .. } => {
                Err(self.native_call_diagnostics().return_statement(*span))
            }
            Stmt::Global { span, .. } => {
                Err(self.unsupported(*span, LLVM_GLOBAL_DECLARATION_REJECTION))
            }
            Stmt::StaticLocal { span, .. } => {
                Err(self.unsupported(*span, LLVM_STATIC_LOCAL_REJECTION))
            }
        }
    }

    fn emit_expr(&mut self, expr: &Expr) -> CompileResult<IrValue> {
        match expr {
            Expr::Null(_) => Ok(IrValue::Null),
            Expr::Bool(value, _) => Ok(IrValue::Bool(*value)),
            Expr::Int(value, _) => Ok(IrValue::Int(value.to_string())),
            Expr::Float(value, _) => Ok(IrValue::Float(format_float_literal(*value))),
            Expr::String(value, _) => Ok(IrValue::String(value.clone())),
            Expr::InterpolatedString { span, .. } => {
                Err(self.unsupported(*span, LLVM_INTERPOLATED_STRING_REJECTION))
            }
            Expr::MagicLine { span }
            | Expr::MagicFile { span }
            | Expr::MagicDir { span }
            | Expr::MagicFunction { span }
            | Expr::MagicClass { span }
            | Expr::MagicMethod { span } => {
                Err(self.unsupported(*span, LLVM_MAGIC_CONSTANT_REJECTION))
            }
            Expr::GlobalConstant { span, .. } => {
                Err(self.unsupported(*span, LLVM_GLOBAL_CONSTANT_REJECTION))
            }
            Expr::ClassNameConstant { span, .. }
            | Expr::SelfClassNameConstant { span }
            | Expr::ParentClassNameConstant { span }
            | Expr::StaticClassNameConstant { span } => {
                Err(self.unsupported(*span, LLVM_CLASS_NAME_CONSTANT_REJECTION))
            }
            Expr::ClassConstant { span, .. }
            | Expr::SelfClassConstant { span, .. }
            | Expr::ParentClassConstant { span, .. }
            | Expr::LateStaticClassConstant { span, .. }
            | Expr::StaticProperty { span, .. }
            | Expr::SelfStaticProperty { span, .. }
            | Expr::ParentStaticProperty { span, .. }
            | Expr::LateStaticProperty { span, .. } => {
                Err(self.unsupported(*span, LLVM_STATIC_MEMBER_REJECTION))
            }
            Expr::Array { span, .. } => {
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            Expr::Index {
                target,
                index,
                span,
            } => {
                if let Some(operation) = native_dereferenced_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if let Some(superglobal_span) = request_superglobal_expr_span(target) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if is_object_offset_expr(target) {
                    return Err(self.unsupported(*span, LLVM_ARRAY_ACCESS_REJECTION));
                }
                if self.is_known_value_offset_target(target) {
                    let target = self.emit_value_operand_expr(target)?;
                    let index = self.emit_value_operand_expr(index)?;
                    return self.emit_native_value_offset_operation(
                        NativeStringOffsetOperation::Read,
                        target,
                        index,
                        *span,
                    );
                }
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            Expr::AppendIndex { target, span } => {
                if let Some(operation) = native_dereferenced_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if let Some(superglobal_span) = request_superglobal_expr_span(target) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if is_object_offset_expr(target) {
                    return Err(self.unsupported(*span, LLVM_ARRAY_ACCESS_REJECTION));
                }
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            Expr::Property { span, .. } | Expr::DynamicProperty { span, .. } => {
                if let Some(operation) = native_dereferenced_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, LLVM_OBJECT_PROPERTY_REJECTION))
            }
            Expr::ObjectStaticProperty { span, .. } => {
                if let Some(operation) = native_dereferenced_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, LLVM_STATIC_MEMBER_REJECTION))
            }
            Expr::MethodCall { .. }
            | Expr::DynamicMethodCall { .. }
            | Expr::ParentMethodCall { .. }
            | Expr::StaticMethodCall { .. }
            | Expr::ObjectStaticMethodCall { .. }
            | Expr::SelfMethodCall { .. }
            | Expr::LateStaticMethodCall { .. } => Err(self.unsupported_value_call(expr)),
            Expr::Variable(name, span) => {
                if is_request_superglobal_name(name) {
                    return Err(self.unsupported(*span, LLVM_REQUEST_SUPERGLOBAL_REJECTION));
                }
                self.variables
                    .get(name)
                    .cloned()
                    .ok_or_else(|| self.unsupported(*span, LLVM_VARIABLE_READ_REJECTION))
            }
            Expr::Call { name, args, span } if is_exit_construct_name(name) => {
                Err(self.unsupported_direct_named_call(args, *span, LLVM_TERMINATION_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("defined") => {
                self.emit_defined_call(args, *span)
            }
            Expr::Call { name, args, span } if is_global_constant_builtin(name) => {
                Err(self.unsupported_direct_named_call(args, *span, LLVM_GLOBAL_CONSTANT_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("isset") => {
                self.emit_isset_call(args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("empty") => {
                self.emit_empty_call(args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("strlen") => {
                self.emit_strlen_call(args, *span)
            }
            Expr::Call { name, args, span } if native_value_cast_builtin_op_tag(name).is_some() => {
                Err(self.unsupported_direct_named_call(args, *span, LLVM_CAST_REJECTION))
            }
            Expr::Call { name, args, span } if native_string_predicate_for_name(name).is_some() => {
                Err(self.unsupported_direct_named_call(
                    args,
                    *span,
                    LLVM_STRING_PREDICATE_REJECTION,
                ))
            }
            Expr::Call { name, args, span }
                if native_string_int_operation_for_name(name).is_some() =>
            {
                let operation = native_string_int_operation_for_name(name)
                    .expect("string-int operation checked above");
                self.emit_llvm_string_int_call(operation, args, *span)
            }
            Expr::Call { name, args, span }
                if native_string_distance_operation_for_name(name).is_some() =>
            {
                Err(self.unsupported_direct_named_call(
                    args,
                    *span,
                    LLVM_STRING_DISTANCE_OPERATION_REJECTION,
                ))
            }
            Expr::Call { name, args, span }
                if native_string_result_operation_for_name(name).is_some() =>
            {
                Err(self.unsupported_direct_named_call(
                    args,
                    *span,
                    LLVM_STRING_RESULT_OPERATION_REJECTION,
                ))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("basename") => {
                Err(self.unsupported_direct_named_call(args, *span, LLVM_BASENAME_REJECTION))
            }
            Expr::Call { name, args, span }
                if let Some(operation) = native_filesystem_path_operation_for_name(name) =>
            {
                Err(self.unsupported_direct_named_call(
                    args,
                    *span,
                    native_filesystem_path_operation_llvm_rejection(operation),
                ))
            }
            Expr::Call { name, args, span } if is_stream_resource_builtin(name) => {
                Err(self.unsupported_direct_named_call(args, *span, LLVM_STREAM_RESOURCE_REJECTION))
            }
            Expr::Call { name, args, span } if is_header_state_builtin(name) => {
                Err(self.unsupported_direct_named_call(args, *span, LLVM_HEADER_STATE_REJECTION))
            }
            Expr::Call { name, args, span } if is_session_state_builtin(name) => {
                Err(self.unsupported_direct_named_call(args, *span, LLVM_SESSION_STATE_REJECTION))
            }
            Expr::Call { name, args, span } if is_output_buffer_builtin(name) => {
                Err(self.unsupported_direct_named_call(args, *span, LLVM_OUTPUT_BUFFER_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("function_exists") => {
                self.emit_function_exists_call(args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("is_callable") => {
                self.emit_is_callable_call(args, *span)
            }
            Expr::Call { name, args, span } if is_native_type_introspection_builtin(name) => {
                self.emit_native_type_introspection_call(name, args, *span)
            }
            Expr::Call { name, args, span } if is_object_metadata_builtin(name) => {
                Err(self.unsupported_direct_named_call(args, *span, LLVM_OBJECT_METADATA_REJECTION))
            }
            Expr::Call { name, args, span } if is_array_builtin(name) => {
                Err(self.unsupported_direct_named_call(args, *span, LLVM_ARRAY_REJECTION))
            }
            Expr::DynamicCall { .. } => Err(self.unsupported_value_call(expr)),
            Expr::Call { .. } => Err(self.unsupported_value_call(expr)),
            Expr::InstanceOf { span, .. } => {
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, LLVM_INSTANCEOF_REJECTION))
            }
            Expr::Closure { .. } => Err(self.native_call_diagnostics().call_root(expr)),
            Expr::New { .. } => Err(self.unsupported_value_call(expr)),
            Expr::Clone { span, .. } => {
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, LLVM_CLONE_REJECTION))
            }
            Expr::Unary { op, expr, span } => {
                if matches!(op, UnaryOp::Not) {
                    if let Expr::Unary {
                        op: UnaryOp::Not,
                        expr,
                        ..
                    } = expr.as_ref()
                    {
                        let value = self.emit_value_operand_expr(expr)?;
                        if matches!(value, IrValue::Bool(_) | IrValue::BoolExpr(_)) {
                            return Ok(value);
                        }
                        let inverted = self.emit_bool_not(value, *span)?;
                        return self.emit_bool_not(inverted, *span);
                    }
                }
                if matches!(op, UnaryOp::BitwiseNot) {
                    if let Expr::Unary {
                        op: UnaryOp::BitwiseNot,
                        expr,
                        ..
                    } = expr.as_ref()
                    {
                        return match self.emit_value_operand_expr(expr)? {
                            value @ IrValue::Int(_) => Ok(value),
                            _ => Err(self.unsupported(*span, LLVM_BITWISE_REJECTION)),
                        };
                    }
                }
                let value = self.emit_value_operand_expr(expr)?;
                self.emit_unary(*op, value, *span)
            }
            Expr::ErrorControl { span, .. } => {
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, LLVM_ERROR_CONTROL_REJECTION))
            }
            Expr::Include { span, .. } | Expr::Require { span, .. } => {
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, LLVM_REQUIRE_EXPRESSION_REJECTION))
            }
            Expr::Cast { span, .. } => {
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, LLVM_CAST_REJECTION))
            }
            Expr::Assign { target, span, .. } => {
                if let Some(operation) = native_assignment_target_call_operation(target) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, LLVM_ARRAY_ACCESS_REJECTION));
                }
                if is_static_member_assign_target(target) {
                    return Err(self.unsupported(*span, LLVM_STATIC_MEMBER_REJECTION));
                }
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Expr::CompoundAssign { target, span, .. }
            | Expr::NullCoalesceAssign { target, span, .. }
            | Expr::IncrementDecrement { target, span, .. } => {
                if let Some(operation) = native_assignment_target_call_operation(target) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, LLVM_ARRAY_ACCESS_REJECTION));
                }
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Expr::Ternary {
                condition,
                if_true,
                if_false,
                span,
            } => self.emit_ternary_expr(condition, if_true, if_false, *span),
            Expr::ShortTernary {
                condition,
                if_false,
                span,
            } => self.emit_short_ternary(condition, if_false, *span),
            Expr::Binary {
                left,
                op,
                right,
                span,
            } => {
                if is_comparison_op(*op) && !matches!(op, BinaryOp::StrictEq | BinaryOp::StrictNe) {
                    return self.emit_scalar_comparison_expr(left, *op, right, *span);
                }
                if matches!(op, BinaryOp::NullCoalesce) {
                    if let Some(operation) = native_value_operand_call_result_operation(expr) {
                        return Err(self.unsupported_call_operation(operation));
                    }
                    return Err(self.unsupported(*span, LLVM_CONDITIONAL_REJECTION));
                }
                if matches!(op, BinaryOp::Concat) {
                    return self.emit_static_string_concat_expr(left, right, *span);
                }
                if matches!(
                    op,
                    BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::LogicalXor
                ) {
                    return self.emit_logical_expr(left, *op, right, *span);
                }
                let (left, right) = self.emit_binary_value_operand_exprs(left, right)?;
                self.emit_binary(left, *op, right, *span)
            }
        }
    }

    fn emit_isset_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if let Some(operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(operation));
        }

        let [arg] = args else {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        };

        if let Some(superglobal_span) = request_superglobal_expr_span(arg) {
            return Err(self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION));
        }

        if is_array_access_offset_expr(arg) {
            return Err(self.unsupported(arg.span(), LLVM_ARRAY_ACCESS_REJECTION));
        }

        if let Expr::Index {
            target,
            index,
            span,
        } = arg
        {
            if self.is_known_value_offset_target(target) {
                let target = self.emit_value_operand_expr(target)?;
                let index = self.emit_value_operand_expr(index)?;
                return self.emit_native_value_offset_bool_operation(
                    NativeStringOffsetOperation::Isset,
                    target,
                    index,
                    *span,
                );
            }
        }

        let Expr::Variable(name, _) = arg else {
            return Err(self.unsupported(arg.span(), LLVM_ISSET_REJECTION));
        };

        Ok(IrValue::Bool(!matches!(
            self.variables.get(name),
            None | Some(IrValue::Null)
        )))
    }

    fn emit_empty_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if let Some(operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(operation));
        }

        let [arg] = args else {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        };

        if let Some(superglobal_span) = request_superglobal_expr_span(arg) {
            return Err(self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION));
        }

        if is_array_access_offset_expr(arg) {
            return Err(self.unsupported(arg.span(), LLVM_ARRAY_ACCESS_REJECTION));
        }

        if let Expr::Index {
            target,
            index,
            span,
        } = arg
        {
            if self.is_known_value_offset_target(target) {
                let target = self.emit_value_operand_expr(target)?;
                let index = self.emit_value_operand_expr(index)?;
                return self.emit_native_value_offset_bool_operation(
                    NativeStringOffsetOperation::Empty,
                    target,
                    index,
                    *span,
                );
            }
        }

        let Expr::Variable(name, _) = arg else {
            return Err(self.unsupported(arg.span(), LLVM_EMPTY_REJECTION));
        };

        let Some(value) = self.variables.get(name) else {
            return Ok(IrValue::Bool(true));
        };

        self.known_truthiness_for_value(value)
            .map(|truthy| IrValue::Bool(!truthy))
            .ok_or_else(|| self.unsupported(arg.span(), LLVM_EMPTY_REJECTION))
    }

    fn emit_strlen_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if let Some(operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(operation));
        }

        if args.len() != 1 {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let value = self.emit_expr(&args[0])?;
        if let Some(length) = self.strlen_result_for_value(&value) {
            return Ok(IrValue::Int(length.to_string()));
        }
        if matches!(value, IrValue::NativeValue(_)) {
            return self.emit_native_value_string_len(value, span);
        }
        Err(self.unsupported_direct_call(span, NativeCallBlocker::ReturnValueOwnership))
    }

    fn emit_llvm_string_int_call(
        &mut self,
        operation: NativeStringIntOperation,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<IrValue> {
        if let Some(call_operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(call_operation));
        }

        match operation {
            NativeStringIntOperation::CaseCompare | NativeStringIntOperation::ByteCompare
                if args.len() == 2 =>
            {
                let subject = self.emit_value_operand_expr(&args[0])?;
                let operand = self.emit_value_operand_expr(&args[1])?;
                self.emit_native_string_int_operation(
                    operation,
                    subject,
                    Some(operand),
                    "0".to_string(),
                    "0".to_string(),
                    false,
                    span,
                )
            }
            NativeStringIntOperation::BytePrefixCompare
            | NativeStringIntOperation::CasePrefixCompare
                if args.len() == 3 =>
            {
                let subject = self.emit_value_operand_expr(&args[0])?;
                let operand = self.emit_value_operand_expr(&args[1])?;
                let length = self.emit_value_operand_expr(&args[2])?;
                let length = self.emit_native_int_for_ir_value(
                    length,
                    NativeIntConversionOperation::StringLength,
                    span,
                    LLVM_STRING_INT_OPERATION_REJECTION,
                )?;
                self.emit_native_string_int_operation(
                    operation,
                    subject,
                    Some(operand),
                    "0".to_string(),
                    length,
                    true,
                    span,
                )
            }
            NativeStringIntOperation::SubstrCount if (2..=4).contains(&args.len()) => {
                let subject = self.emit_value_operand_expr(&args[0])?;
                let operand = self.emit_value_operand_expr(&args[1])?;
                let offset = if let Some(offset) = args.get(2) {
                    let offset = self.emit_value_operand_expr(offset)?;
                    self.emit_native_int_for_ir_value(
                        offset,
                        NativeIntConversionOperation::StringOffset,
                        span,
                        LLVM_STRING_INT_OPERATION_REJECTION,
                    )?
                } else {
                    "0".to_string()
                };
                let (length, has_length) = if let Some(length) = args.get(3) {
                    let length = self.emit_value_operand_expr(length)?;
                    (
                        self.emit_native_int_for_ir_value(
                            length,
                            NativeIntConversionOperation::StringLength,
                            span,
                            LLVM_STRING_INT_OPERATION_REJECTION,
                        )?,
                        true,
                    )
                } else {
                    ("0".to_string(), false)
                };
                self.emit_native_string_int_operation(
                    operation,
                    subject,
                    Some(operand),
                    offset,
                    length,
                    has_length,
                    span,
                )
            }
            NativeStringIntOperation::Ordinal | NativeStringIntOperation::Crc32
                if args.len() == 1 =>
            {
                let subject = self.emit_value_operand_expr(&args[0])?;
                self.emit_native_string_int_operation(
                    operation,
                    subject,
                    None,
                    "0".to_string(),
                    "0".to_string(),
                    false,
                    span,
                )
            }
            _ => Err(self.unsupported_direct_named_call(
                args,
                span,
                LLVM_STRING_INT_OPERATION_REJECTION,
            )),
        }
    }

    fn emit_native_string_int_operation(
        &mut self,
        operation: NativeStringIntOperation,
        subject: IrValue,
        operand: Option<IrValue>,
        offset: String,
        length: String,
        has_length: bool,
        span: Span,
    ) -> CompileResult<IrValue> {
        let subject = self
            .emit_native_value_for_ir_value(subject, span)
            .map_err(|_| self.unsupported(span, LLVM_STRING_INT_OPERATION_REJECTION))?;
        let operand = match operand {
            Some(value) => Some(
                self.emit_native_value_for_ir_value(value, span)
                    .map_err(|_| self.unsupported(span, LLVM_STRING_INT_OPERATION_REJECTION))?,
            ),
            None => None,
        };
        let operand_value = operand.as_deref().unwrap_or("zeroinitializer").to_string();
        let flags = u8::from(has_length);
        let diagnostic_slot = self.next_temp();
        let result = self.next_temp();
        self.uses_native_string_int_operation = true;
        self.body.push(format!(
            "{diagnostic_slot} = alloca %phpc.NativeDiagnosticHandle"
        ));
        self.body.push(format!(
            "store %phpc.NativeDiagnosticHandle zeroinitializer, ptr {diagnostic_slot}"
        ));
        self.body.push(format!(
            "{result} = call i64 @phpc_native_value_string_int_operation_with_diagnostic(%phpc.NativeValueHandle {subject}, %phpc.NativeValueHandle {operand_value}, i64 {offset}, i64 {length}, i8 {flags}, i8 {}, ptr {diagnostic_slot})",
            operation as u8
        ));
        self.emit_report_native_diagnostic_slot(&diagnostic_slot);
        if let Some(operand) = operand {
            self.body.push(format!(
                "call void @phpc_native_value_free(%phpc.NativeValueHandle {operand})"
            ));
        }
        self.body.push(format!(
            "call void @phpc_native_value_free(%phpc.NativeValueHandle {subject})"
        ));
        Ok(IrValue::Int(result))
    }

    fn emit_native_value_offset_operation(
        &mut self,
        operation: NativeStringOffsetOperation,
        subject: IrValue,
        offset: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        let subject = self
            .emit_native_value_for_ir_value(subject, span)
            .map_err(|_| self.unsupported(span, LLVM_ARRAY_REJECTION))?;
        let offset = self
            .emit_native_value_for_ir_value(offset, span)
            .map_err(|_| self.unsupported(span, LLVM_ARRAY_REJECTION))?;
        let diagnostic_slot = self.next_temp();
        let result = self.next_temp();
        self.uses_native_value_offset_operation = true;
        self.body.push(format!(
            "{diagnostic_slot} = alloca %phpc.NativeDiagnosticHandle"
        ));
        self.body.push(format!(
            "store %phpc.NativeDiagnosticHandle zeroinitializer, ptr {diagnostic_slot}"
        ));
        self.body.push(format!(
            "{result} = call %phpc.NativeValueHandle @phpc_native_value_offset_operation_with_diagnostic(%phpc.NativeValueHandle {subject}, %phpc.NativeValueHandle {offset}, i8 {}, ptr {diagnostic_slot})",
            operation as u8
        ));
        self.emit_report_native_diagnostic_slot(&diagnostic_slot);
        self.body.push(format!(
            "call void @phpc_native_value_free(%phpc.NativeValueHandle {offset})"
        ));
        self.body.push(format!(
            "call void @phpc_native_value_free(%phpc.NativeValueHandle {subject})"
        ));
        Ok(IrValue::NativeValue(result))
    }

    fn emit_native_value_offset_bool_operation(
        &mut self,
        operation: NativeStringOffsetOperation,
        subject: IrValue,
        offset: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        let IrValue::NativeValue(value) =
            self.emit_native_value_offset_operation(operation, subject, offset, span)?
        else {
            unreachable!("value-offset operation returns a native value")
        };
        let diagnostic_slot = self.next_temp();
        let result = self.next_temp();
        self.uses_native_value_offset_operation = true;
        self.body.push(format!(
            "{diagnostic_slot} = alloca %phpc.NativeDiagnosticHandle"
        ));
        self.body.push(format!(
            "store %phpc.NativeDiagnosticHandle zeroinitializer, ptr {diagnostic_slot}"
        ));
        self.body.push(format!(
            "{result} = call i1 @phpc_native_value_bool_with_diagnostic(%phpc.NativeValueHandle {value}, ptr {diagnostic_slot})"
        ));
        self.emit_report_native_diagnostic_slot(&diagnostic_slot);
        self.body.push(format!(
            "call void @phpc_native_value_free(%phpc.NativeValueHandle {value})"
        ));
        Ok(IrValue::BoolExpr(result))
    }

    fn emit_function_exists_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if let Some(operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(operation));
        }

        if args.len() != 1 {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let value = self.emit_expr(&args[0])?;
        self.function_exists_result_for_value(&value)
            .map(IrValue::Bool)
            .ok_or_else(|| {
                self.unsupported_direct_call(span, NativeCallBlocker::UnknownCalleeDiagnostics)
            })
    }

    fn emit_is_callable_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if let Some(operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(operation));
        }

        if !(1..=2).contains(&args.len()) {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let value = self.emit_expr(&args[0])?;
        let syntax_only = if let Some(arg) = args.get(1) {
            match self.emit_expr(arg)? {
                IrValue::Bool(value) => value,
                _ => {
                    return Err(self.unsupported_direct_call(
                        span,
                        NativeCallBlocker::ArgumentEvaluationCleanup,
                    ));
                }
            }
        } else {
            false
        };

        self.is_callable_result_for_value(&value, syntax_only)
            .map(IrValue::Bool)
            .ok_or_else(|| {
                self.unsupported_direct_call(span, NativeCallBlocker::UnknownCalleeDiagnostics)
            })
    }

    fn emit_defined_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if let Some(operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(operation));
        }

        if args.len() != 1 {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let value = self.emit_expr(&args[0])?;
        self.defined_result_for_value(&value)
            .map(IrValue::Bool)
            .ok_or_else(|| self.unsupported(span, LLVM_GLOBAL_CONSTANT_REJECTION))
    }

    fn emit_native_type_introspection_call(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<IrValue> {
        if let Some(operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(operation));
        }

        if is_native_metadata_exists_builtin(name) {
            return self.emit_native_metadata_exists_call(args, span);
        }
        if is_native_member_metadata_exists_builtin(name) {
            return self.emit_native_member_metadata_exists_call(args, span);
        }
        if is_native_relationship_metadata_builtin(name) {
            return self.emit_native_relationship_metadata_call(args, span);
        }

        if args.len() != 1 {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let value = self.emit_expr(&args[0])?;
        if matches!(value, IrValue::NativeValue(_)) {
            return Err(self.unsupported_direct_call(span, NativeCallBlocker::ReturnValueOwnership));
        }
        match name.to_ascii_lowercase().as_str() {
            "gettype" => Ok(IrValue::String(llvm_gettype_name(&value).to_string())),
            "get_debug_type" => Ok(IrValue::String(llvm_debug_type_name(&value).to_string())),
            "is_null" => Ok(IrValue::Bool(matches!(value, IrValue::Null))),
            "is_bool" => Ok(IrValue::Bool(matches!(
                value,
                IrValue::Bool(_) | IrValue::BoolExpr(_)
            ))),
            "is_int" | "is_integer" | "is_long" => {
                Ok(IrValue::Bool(matches!(value, IrValue::Int(_))))
            }
            "is_float" | "is_double" => Ok(IrValue::Bool(matches!(value, IrValue::Float(_)))),
            "is_string" => Ok(IrValue::Bool(matches!(
                value,
                IrValue::String(_) | IrValue::StringPtr(_)
            ))),
            "is_array" => Ok(IrValue::Bool(false)),
            "is_scalar" => Ok(IrValue::Bool(matches!(
                value,
                IrValue::Bool(_)
                    | IrValue::BoolExpr(_)
                    | IrValue::Int(_)
                    | IrValue::Float(_)
                    | IrValue::String(_)
                    | IrValue::StringPtr(_)
            ))),
            "is_numeric" => self
                .is_numeric_result_for_value(&value)
                .map(IrValue::Bool)
                .ok_or_else(|| {
                    self.unsupported_direct_call(span, NativeCallBlocker::ReturnValueOwnership)
                }),
            "is_countable" | "is_iterable" => Ok(IrValue::Bool(false)),
            "extension_loaded" => match value {
                IrValue::String(name) => Ok(IrValue::Bool(is_compat_loaded_extension_name(&name))),
                IrValue::StringPtr(_) => Ok(IrValue::Bool(false)),
                _ => Err(self
                    .unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)),
            },
            "is_object" => Ok(IrValue::Bool(false)),
            _ => {
                Err(self.unsupported_direct_call(span, NativeCallBlocker::UnknownCalleeDiagnostics))
            }
        }
    }

    fn emit_native_metadata_exists_call(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<IrValue> {
        if !(1..=2).contains(&args.len()) {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let name = self.emit_expr(&args[0])?;
        if !matches!(name, IrValue::String(_) | IrValue::StringPtr(_)) {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }
        if self.ir_value_mentions_builtin_class(&name) {
            return Err(self.unsupported(span, LLVM_OBJECT_METADATA_REJECTION));
        }

        if let Some(autoload) = args.get(1) {
            let autoload = self.emit_expr(autoload)?;
            if !matches!(autoload, IrValue::Bool(_) | IrValue::BoolExpr(_)) {
                return Err(self
                    .unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup));
            }
        }

        Ok(IrValue::Bool(false))
    }

    fn emit_native_member_metadata_exists_call(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<IrValue> {
        if args.len() != 2 {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let object_or_class = self.emit_expr(&args[0])?;
        let member = self.emit_expr(&args[1])?;
        if !matches!(object_or_class, IrValue::String(_) | IrValue::StringPtr(_))
            || !matches!(member, IrValue::String(_) | IrValue::StringPtr(_))
        {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }
        if self.ir_value_mentions_builtin_class(&object_or_class) {
            return Err(self.unsupported(span, LLVM_OBJECT_METADATA_REJECTION));
        }

        Ok(IrValue::Bool(false))
    }

    fn emit_native_relationship_metadata_call(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<IrValue> {
        if !(2..=3).contains(&args.len()) {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let object_or_class = self.emit_expr(&args[0])?;
        let class_name = self.emit_expr(&args[1])?;
        if !matches!(object_or_class, IrValue::String(_) | IrValue::StringPtr(_))
            || !matches!(class_name, IrValue::String(_) | IrValue::StringPtr(_))
        {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }
        if self.ir_value_mentions_builtin_class(&object_or_class)
            || self.ir_value_mentions_builtin_class(&class_name)
        {
            return Err(self.unsupported(span, LLVM_OBJECT_METADATA_REJECTION));
        }

        if let Some(allow_string) = args.get(2) {
            let allow_string = self.emit_expr(allow_string)?;
            if !matches!(allow_string, IrValue::Bool(_) | IrValue::BoolExpr(_)) {
                return Err(self
                    .unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup));
            }
        }

        Ok(IrValue::Bool(false))
    }

    fn ir_value_mentions_builtin_class(&self, value: &IrValue) -> bool {
        self.known_string_values_for_value(value)
            .map(|values| {
                values
                    .values()
                    .iter()
                    .any(|value| is_builtin_class_name(value))
            })
            .unwrap_or(false)
    }

    fn is_numeric_result_for_value(&self, value: &IrValue) -> Option<bool> {
        match value {
            IrValue::Int(_) | IrValue::Float(_) => Some(true),
            IrValue::Null | IrValue::Bool(_) | IrValue::BoolExpr(_) => Some(false),
            IrValue::String(value) => Some(classify_php_numeric_string(value).is_numeric()),
            IrValue::StringPtr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_numeric_result(&values)
            }
            IrValue::NativeValue(_) => None,
        }
    }

    fn function_exists_result_for_value(&self, value: &IrValue) -> Option<bool> {
        match value {
            IrValue::String(value) => Some(is_native_known_function_name(value)),
            IrValue::StringPtr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_function_exists_result(&values)
            }
            _ => None,
        }
    }

    fn strlen_result_for_value(&self, value: &IrValue) -> Option<usize> {
        match value {
            IrValue::String(value) => Some(value.len()),
            IrValue::StringPtr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_byte_length(&values)
            }
            _ => None,
        }
    }

    fn is_known_value_offset_target(&self, target: &Expr) -> bool {
        self.is_known_string_offset_target(target)
    }

    fn is_known_string_offset_target(&self, target: &Expr) -> bool {
        match target {
            Expr::String(_, _) => true,
            Expr::Variable(name, _) => self
                .variables
                .get(name)
                .is_some_and(|value| self.is_string_offset_ir_value(value)),
            _ => false,
        }
    }

    fn is_string_offset_ir_value(&self, value: &IrValue) -> bool {
        matches!(
            value,
            IrValue::String(_) | IrValue::StringPtr(_) | IrValue::NativeValue(_)
        )
    }

    fn is_callable_result_for_value(&self, value: &IrValue, syntax_only: bool) -> Option<bool> {
        match value {
            IrValue::String(_) | IrValue::StringPtr(_) if syntax_only => Some(true),
            IrValue::Null
            | IrValue::Bool(_)
            | IrValue::BoolExpr(_)
            | IrValue::Int(_)
            | IrValue::Float(_) => Some(false),
            _ => self.function_exists_result_for_value(value),
        }
    }

    fn defined_result_for_value(&self, value: &IrValue) -> Option<bool> {
        match value {
            IrValue::String(value) => native_defined_result(value),
            IrValue::StringPtr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_defined_result(&values)
            }
            _ => None,
        }
    }

    fn emit_assignment(&mut self, target: &AssignTarget, expr: &Expr) -> CompileResult<()> {
        if let Some(operation) = native_assignment_target_call_operation(target) {
            return Err(self.unsupported_call_operation(operation));
        }

        match target {
            AssignTarget::Variable { name, .. } => {
                let value = self.emit_expr(expr)?;
                if matches!(value, IrValue::NativeValue(_)) {
                    return Err(self.unsupported_direct_call(
                        expr.span(),
                        NativeCallBlocker::ReturnValueOwnership,
                    ));
                }
                self.variables.insert(name.clone(), value);
                Ok(())
            }
            AssignTarget::List { span, .. } => {
                Err(self.unsupported(*span, LLVM_ARRAY_DESTRUCTURING_REJECTION))
            }
            AssignTarget::ArrayIndex { span, .. } => {
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            AssignTarget::NestedArrayIndex { span, .. }
            | AssignTarget::NestedArrayAppend { span, .. } => {
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            AssignTarget::ObjectPropertyArrayIndex { span, .. }
            | AssignTarget::DynamicObjectPropertyArrayIndex { span, .. }
            | AssignTarget::NonDirectObjectPropertyArrayIndex { span, .. }
            | AssignTarget::NonDirectObjectPropertyArrayAppend { span, .. }
            | AssignTarget::NonDirectDynamicObjectPropertyArrayIndex { span, .. }
            | AssignTarget::NonDirectDynamicObjectPropertyArrayAppend { span, .. }
            | AssignTarget::ObjectPropertyArrayAppend { span, .. }
            | AssignTarget::DynamicObjectPropertyArrayAppend { span, .. } => {
                Err(self.unsupported(*span, LLVM_ARRAY_ACCESS_REJECTION))
            }
            AssignTarget::Property { span, .. }
            | AssignTarget::NonDirectProperty { span, .. }
            | AssignTarget::NonDirectDynamicProperty { span, .. }
            | AssignTarget::DynamicProperty { span, .. } => {
                Err(self.unsupported(*span, LLVM_OBJECT_PROPERTY_REJECTION))
            }
            AssignTarget::StaticProperty { span, .. }
            | AssignTarget::ObjectStaticProperty { span, .. }
            | AssignTarget::SelfStaticProperty { span, .. }
            | AssignTarget::ParentStaticProperty { span, .. }
            | AssignTarget::LateStaticProperty { span, .. } => {
                Err(self.unsupported(*span, LLVM_STATIC_MEMBER_REJECTION))
            }
        }
    }

    fn emit_binary(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Mod => {
                self.emit_arithmetic_binary(left, op, right, span)
            }
            BinaryOp::Div => Err(self.unsupported(span, LLVM_DIVISION_REJECTION)),
            BinaryOp::Concat => Err(self.unsupported(span, LLVM_CONCAT_REJECTION)),
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => self.emit_scalar_comparison(left, op, right, span),
            BinaryOp::StrictEq | BinaryOp::StrictNe => {
                self.emit_static_strict_identity(left, op, right, span)
            }
            BinaryOp::NullCoalesce => Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION)),
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::LogicalXor => {
                self.emit_bool_binary(left, op, right, span)
            }
            BinaryOp::BitwiseAnd | BinaryOp::BitwiseOr | BinaryOp::BitwiseXor => {
                self.emit_integer_bitwise_binary(left, op, right, span)
            }
            BinaryOp::ShiftLeft | BinaryOp::ShiftRight => {
                self.emit_integer_shift_binary(left, op, right, span)
            }
        }
    }

    fn emit_arithmetic_binary(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        let direct_numeric_pair = matches!(
            (&left, &right),
            (IrValue::Int(_), IrValue::Int(_)) | (IrValue::Float(_), IrValue::Float(_))
        );
        if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) && !direct_numeric_pair {
            match self.checked_static_primitive_arithmetic_result_for_values(&left, op, &right) {
                Ok(Some(result)) => {
                    if let Some(value) = result.into_single_ir_value() {
                        return Ok(value);
                    }
                }
                Err(PhpPrimitiveArithmeticError::IntegerOverflow) => {
                    return Err(self.unsupported(span, LLVM_INTEGER_OVERFLOW_ARITHMETIC_REJECTION));
                }
                Err(PhpPrimitiveArithmeticError::NonFiniteFloat) => {
                    return Err(self.unsupported(span, LLVM_MIXED_NUMERIC_ARITHMETIC_REJECTION));
                }
                Ok(None)
                | Err(
                    PhpPrimitiveArithmeticError::MissingRightOperand
                    | PhpPrimitiveArithmeticError::Conversion(_),
                ) => {}
            }
        }

        match (left, right) {
            (IrValue::Int(left), IrValue::Int(right)) => match op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul => {
                    if matches!(op, BinaryOp::Add) {
                        if right == "0" {
                            return Ok(IrValue::Int(left));
                        }
                        if left == "0" {
                            return Ok(IrValue::Int(right));
                        }
                    }
                    if matches!(op, BinaryOp::Sub) && right == "0" {
                        return Ok(IrValue::Int(left));
                    }
                    if matches!(op, BinaryOp::Sub) && left == right {
                        return Ok(IrValue::Int("0".to_string()));
                    }
                    if matches!(op, BinaryOp::Mul) {
                        if right == "0" || left == "0" {
                            return Ok(IrValue::Int("0".to_string()));
                        }
                        if right == "1" {
                            return Ok(IrValue::Int(left));
                        }
                        if left == "1" {
                            return Ok(IrValue::Int(right));
                        }
                    }
                    let left_is_tracked = self.is_tracked_integer_value(&left);
                    let right_is_tracked = self.is_tracked_integer_value(&right);
                    let Some(result) = self.checked_static_integer_arithmetic(&left, op, &right)
                    else {
                        return Err(
                            self.unsupported(span, LLVM_INTEGER_OVERFLOW_ARITHMETIC_REJECTION)
                        );
                    };
                    if (left_is_tracked || right_is_tracked) && result.is_single() {
                        return Ok(IrValue::Int(result.values()[0].to_string()));
                    }
                    let instruction = match op {
                        BinaryOp::Add => "add",
                        BinaryOp::Sub => "sub",
                        BinaryOp::Mul => "mul",
                        _ => unreachable!("operator matched above"),
                    };
                    let temp = self.next_temp();
                    self.body
                        .push(format!("{temp} = {instruction} i64 {left}, {right}"));
                    self.known_ints.insert(temp.clone(), result);
                    return Ok(IrValue::Int(temp));
                }
                BinaryOp::Mod => {
                    let Ok(divisor) = right.parse::<i64>() else {
                        return Err(self.unsupported(span, LLVM_MODULO_RUNTIME_CHECK_REJECTION));
                    };
                    if divisor <= 0 {
                        return Err(self.unsupported(span, LLVM_MODULO_RUNTIME_CHECK_REJECTION));
                    }
                    if divisor == 1 {
                        return Ok(IrValue::Int("0".to_string()));
                    }
                    let modulo_result = self.static_integer_modulo(&left, divisor);
                    if let (Some(left_values), Some(result)) =
                        (self.known_integer_values(&left), modulo_result.as_ref())
                    {
                        if !left_values.is_single() && result.is_single() {
                            return Ok(IrValue::Int(result.values()[0].to_string()));
                        }
                    }
                    let temp = self.next_temp();
                    self.body.push(format!("{temp} = srem i64 {left}, {right}"));
                    if let Some(result) = modulo_result {
                        self.known_ints.insert(temp.clone(), result);
                    }
                    return Ok(IrValue::Int(temp));
                }
                _ => return Err(self.unsupported(span, LLVM_ARITHMETIC_REJECTION)),
            },
            (IrValue::Float(left), IrValue::Float(right)) => {
                if matches!(op, BinaryOp::Add) {
                    if right == "0.0" && self.known_finite_nonzero_float_values(&left) {
                        return Ok(IrValue::Float(left));
                    }
                    if left == "0.0" && self.known_finite_nonzero_float_values(&right) {
                        return Ok(IrValue::Float(right));
                    }
                }
                if matches!(op, BinaryOp::Sub)
                    && right == "0.0"
                    && self.known_finite_nonzero_float_values(&left)
                {
                    return Ok(IrValue::Float(left));
                }
                if matches!(op, BinaryOp::Sub) && left == "0.0" {
                    if let Some(result) = self.static_float_arithmetic(&left, op, &right) {
                        if result.is_single() && result.values()[0] != 0.0 {
                            return Ok(IrValue::Float(format_float_literal(result.values()[0])));
                        }
                    }
                }
                if matches!(op, BinaryOp::Mul) {
                    if (right == "0.0" && self.known_finite_positive_float_values(&left))
                        || (left == "0.0" && self.known_finite_positive_float_values(&right))
                    {
                        return Ok(IrValue::Float("0.0".to_string()));
                    }
                    if right == "-1.0" {
                        if let Some(result) = self.static_float_negate(&left) {
                            if result.is_single() && result.values()[0] != 0.0 {
                                return Ok(IrValue::Float(format_float_literal(
                                    result.values()[0],
                                )));
                            }
                        }
                    }
                    if left == "-1.0" {
                        if let Some(result) = self.static_float_negate(&right) {
                            if result.is_single() && result.values()[0] != 0.0 {
                                return Ok(IrValue::Float(format_float_literal(
                                    result.values()[0],
                                )));
                            }
                        }
                    }
                    if right == "1.0" && self.known_float_values(&left).is_some() {
                        return Ok(IrValue::Float(left));
                    }
                    if left == "1.0" && self.known_float_values(&right).is_some() {
                        return Ok(IrValue::Float(right));
                    }
                }
                if matches!(op, BinaryOp::Sub)
                    && left == right
                    && self
                        .known_float_values(&left)
                        .is_some_and(|values| values.values().iter().all(|value| value.is_finite()))
                {
                    return Ok(IrValue::Float("0.0".to_string()));
                }
                let left_is_tracked = self.is_tracked_float_value(&left);
                let right_is_tracked = self.is_tracked_float_value(&right);
                if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul)
                    && (left_is_tracked || right_is_tracked)
                {
                    if let Some(result) = self.static_float_arithmetic(&left, op, &right) {
                        if result.is_single() && result.values()[0] != 0.0 {
                            return Ok(IrValue::Float(format_float_literal(result.values()[0])));
                        }
                    }
                }
                let instruction = match op {
                    BinaryOp::Add => "fadd",
                    BinaryOp::Sub => "fsub",
                    BinaryOp::Mul => "fmul",
                    _ => return Err(self.unsupported(span, LLVM_ARITHMETIC_REJECTION)),
                };
                let temp = self.next_temp();
                self.body
                    .push(format!("{temp} = {instruction} double {left}, {right}"));
                if let Some(result) = self.static_float_arithmetic(&left, op, &right) {
                    self.known_floats.insert(temp.clone(), result);
                }
                Ok(IrValue::Float(temp))
            }
            (IrValue::Int(_), IrValue::Float(_)) | (IrValue::Float(_), IrValue::Int(_))
                if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) =>
            {
                Err(self.unsupported(span, LLVM_MIXED_NUMERIC_ARITHMETIC_REJECTION))
            }
            (
                IrValue::Null
                | IrValue::Bool(_)
                | IrValue::BoolExpr(_)
                | IrValue::String(_)
                | IrValue::StringPtr(_),
                _,
            )
            | (
                _,
                IrValue::Null
                | IrValue::Bool(_)
                | IrValue::BoolExpr(_)
                | IrValue::String(_)
                | IrValue::StringPtr(_),
            ) if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) => {
                Err(self.unsupported(span, LLVM_SCALAR_COERCION_ARITHMETIC_REJECTION))
            }
            _ => Err(self.unsupported(span, LLVM_ARITHMETIC_REJECTION)),
        }
    }

    fn emit_scalar_comparison_expr(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
        span: Span,
    ) -> CompileResult<IrValue> {
        let left = match self.emit_expr(left) {
            Ok(value) => value,
            Err(_) => {
                let fallback = self.unsupported_value_operand_or_fallback(
                    left,
                    span,
                    llvm_comparison_rejection(),
                );
                return Err(
                    self.unsupported_unemitted_value_operands_or_original(&[left, right], fallback)
                );
            }
        };
        let right = match self.emit_expr(right) {
            Ok(value) => value,
            Err(_) => {
                return Err(self.unsupported_value_operand_or_fallback(
                    right,
                    span,
                    llvm_comparison_rejection(),
                ));
            }
        };
        self.emit_scalar_comparison(left, op, right, span)
    }

    fn emit_scalar_comparison(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        match (left, right) {
            (IrValue::Null, IrValue::Null) => {
                let Some(result) = null_comparison_result(op) else {
                    return Err(self.unsupported(span, llvm_comparison_rejection()));
                };
                Ok(IrValue::Bool(result))
            }
            (IrValue::Bool(left), IrValue::Bool(right)) => {
                let Some(result) = bool_comparison_result(left, op, right) else {
                    return Err(self.unsupported(span, llvm_comparison_rejection()));
                };
                Ok(IrValue::Bool(result))
            }
            (IrValue::BoolExpr(left), IrValue::Bool(right)) => {
                let right = if right { "true" } else { "false" };
                self.emit_bool_scalar_comparison(left, op, right.to_string(), span)
            }
            (IrValue::Bool(left), IrValue::BoolExpr(right)) => {
                let left = if left { "true" } else { "false" };
                self.emit_bool_scalar_comparison(left.to_string(), op, right, span)
            }
            (IrValue::BoolExpr(left), IrValue::BoolExpr(right)) => {
                self.emit_bool_scalar_comparison(left, op, right, span)
            }
            (IrValue::String(left), IrValue::String(right)) => {
                let Some(result) = static_safe_string_comparison_result(
                    Some(KnownString::one(left)),
                    op,
                    Some(KnownString::one(right)),
                ) else {
                    return Err(self.unsupported(span, llvm_comparison_rejection()));
                };
                Ok(IrValue::Bool(result))
            }
            (IrValue::StringPtr(left), IrValue::StringPtr(right)) => {
                self.emit_string_comparison(left, op, right, span)
            }
            (IrValue::StringPtr(left), IrValue::String(right)) => {
                let right = self.string_pointer_operand(IrValue::String(right));
                self.emit_string_comparison(left, op, right, span)
            }
            (IrValue::String(left), IrValue::StringPtr(right)) => {
                let left = self.string_pointer_operand(IrValue::String(left));
                self.emit_string_comparison(left, op, right, span)
            }
            (IrValue::Int(left), IrValue::Int(right)) => {
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<i64>(), right.parse::<i64>())
                {
                    let Some(result) = integer_comparison_result(left_literal, op, right_literal)
                    else {
                        return Err(self.unsupported(span, llvm_comparison_rejection()));
                    };
                    return Ok(IrValue::Bool(result));
                }
                if left == right {
                    let Some(result) = integer_comparison_result(0, op, 0) else {
                        return Err(self.unsupported(span, llvm_comparison_rejection()));
                    };
                    return Ok(IrValue::Bool(result));
                }
                let left_is_tracked = self.is_tracked_integer_value(&left);
                let right_is_tracked = self.is_tracked_integer_value(&right);
                if left_is_tracked != right_is_tracked
                    && (left.parse::<i64>().is_ok() || right.parse::<i64>().is_ok())
                {
                    let tracked = if left_is_tracked { &left } else { &right };
                    if self
                        .known_integer_values(tracked)
                        .is_some_and(|values| values.is_single())
                    {
                        if let Some(result) =
                            self.static_integer_comparison_result(&left, op, &right)
                        {
                            return Ok(IrValue::Bool(result));
                        }
                    }
                }
                if let Some(result) = self.static_integer_comparison_result(&left, op, &right) {
                    return Ok(IrValue::Bool(result));
                }
                let predicate = match op {
                    BinaryOp::Eq => "eq",
                    BinaryOp::Ne => "ne",
                    BinaryOp::Lt => "slt",
                    BinaryOp::Le => "sle",
                    BinaryOp::Gt => "sgt",
                    BinaryOp::Ge => "sge",
                    _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
                };
                let temp = self.next_temp();
                self.body
                    .push(format!("{temp} = icmp {predicate} i64 {left}, {right}"));
                if let Some(result) = self.static_integer_comparison_result(&left, op, &right) {
                    self.known_bools
                        .insert(temp.clone(), KnownBool::one(result));
                }
                Ok(IrValue::BoolExpr(temp))
            }
            (IrValue::Float(left), IrValue::Float(right)) => {
                let Some(left_values) = self.known_float_values(&left) else {
                    return Err(self.unsupported(span, llvm_comparison_rejection()));
                };
                let Some(right_values) = self.known_float_values(&right) else {
                    return Err(self.unsupported(span, llvm_comparison_rejection()));
                };
                if !left_values.values().iter().all(|value| value.is_finite())
                    || !right_values.values().iter().all(|value| value.is_finite())
                {
                    return Err(self.unsupported(span, llvm_comparison_rejection()));
                }
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<f64>(), right.parse::<f64>())
                {
                    let Some(result) = float_comparison_result(left_literal, op, right_literal)
                    else {
                        return Err(self.unsupported(span, llvm_comparison_rejection()));
                    };
                    return Ok(IrValue::Bool(result));
                }
                let left_is_tracked = self.is_tracked_float_value(&left);
                let right_is_tracked = self.is_tracked_float_value(&right);
                if left_is_tracked != right_is_tracked
                    && (left.parse::<f64>().is_ok() || right.parse::<f64>().is_ok())
                {
                    let tracked = if left_is_tracked { &left } else { &right };
                    if self
                        .known_float_values(tracked)
                        .is_some_and(|values| values.is_single())
                    {
                        if let Some(result) = self.static_float_comparison_result(&left, op, &right)
                        {
                            return Ok(IrValue::Bool(result));
                        }
                    }
                }
                if let Some(result) = self.static_float_comparison_result(&left, op, &right) {
                    return Ok(IrValue::Bool(result));
                }
                let predicate = match op {
                    BinaryOp::Eq => "oeq",
                    BinaryOp::Ne => "une",
                    BinaryOp::Lt => "olt",
                    BinaryOp::Le => "ole",
                    BinaryOp::Gt => "ogt",
                    BinaryOp::Ge => "oge",
                    _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
                };
                let temp = self.next_temp();
                self.body
                    .push(format!("{temp} = fcmp {predicate} double {left}, {right}"));
                if let Some(result) = self.static_float_comparison_result(&left, op, &right) {
                    self.known_bools
                        .insert(temp.clone(), KnownBool::one(result));
                }
                Ok(IrValue::BoolExpr(temp))
            }
            _ => Err(self.unsupported(span, llvm_comparison_rejection())),
        }
    }

    fn emit_bool_scalar_comparison(
        &mut self,
        left: String,
        op: BinaryOp,
        right: String,
        span: Span,
    ) -> CompileResult<IrValue> {
        if let Some(fold) = bool_literal_comparison_fold(&left, op, &right, "true", "false") {
            return match fold {
                BoolLiteralComparisonFold::Static(value) => Ok(IrValue::Bool(value)),
                BoolLiteralComparisonFold::Reuse(value) => Ok(IrValue::BoolExpr(value)),
                BoolLiteralComparisonFold::Invert(value) => {
                    self.emit_bool_not(IrValue::BoolExpr(value), span)
                }
            };
        }
        if left == right {
            let Some(result) = bool_comparison_result(false, op, false) else {
                return Err(self.unsupported(span, llvm_comparison_rejection()));
            };
            return Ok(IrValue::Bool(result));
        }
        if let Some(result) = self.static_bool_comparison_result(&left, op, &right) {
            return Ok(IrValue::Bool(result));
        }
        let predicate = match op {
            BinaryOp::Eq => "eq",
            BinaryOp::Ne => "ne",
            BinaryOp::Lt => "ult",
            BinaryOp::Le => "ule",
            BinaryOp::Gt => "ugt",
            BinaryOp::Ge => "uge",
            _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
        };
        let temp = self.next_temp();
        self.body
            .push(format!("{temp} = icmp {predicate} i1 {left}, {right}"));
        if let Some(result) = self.static_bool_comparison_result(&left, op, &right) {
            self.known_bools
                .insert(temp.clone(), KnownBool::one(result));
        }
        Ok(IrValue::BoolExpr(temp))
    }

    fn checked_static_integer_arithmetic(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<KnownInt> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                let result = match op {
                    BinaryOp::Add => left.checked_add(*right),
                    BinaryOp::Sub => left.checked_sub(*right),
                    BinaryOp::Mul => left.checked_mul(*right),
                    _ => None,
                }?;
                results.push(result);
            }
        }
        KnownInt::from_values(results)
    }

    fn known_integer_values(&self, value: &str) -> Option<KnownInt> {
        value
            .parse::<i64>()
            .ok()
            .map(KnownInt::one)
            .or_else(|| self.known_ints.get(value).cloned())
    }

    fn is_tracked_integer_value(&self, value: &str) -> bool {
        self.known_ints.contains_key(value)
    }

    fn static_integer_modulo(&self, left: &str, divisor: i64) -> Option<KnownInt> {
        let left = self.known_integer_values(left)?;
        let values = left.values().iter().map(|value| value % divisor);
        KnownInt::from_values(values)
    }

    fn static_integer_comparison_result(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = integer_comparison_result(*left, op, *right)?;
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_float_comparison_result(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_float_values(left)?;
        let right_values = self.known_float_values(right)?;
        if !left_values.values().iter().all(|value| value.is_finite())
            || !right_values.values().iter().all(|value| value.is_finite())
        {
            return None;
        }
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = float_comparison_result(*left, op, *right)?;
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_bool_comparison_result(&self, left: &str, op: BinaryOp, right: &str) -> Option<bool> {
        let left_values = self.known_bool_values(left)?;
        let right_values = self.known_bool_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = bool_comparison_result(*left, op, *right)?;
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn emit_integer_shift_binary(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        let (IrValue::Int(left), IrValue::Int(right)) = (left, right) else {
            return Err(self.unsupported(span, LLVM_BITWISE_REJECTION));
        };
        let Some(count) = self.static_integer_shift_count(&right) else {
            return Err(self.unsupported(span, LLVM_BITWISE_REJECTION));
        };
        if count == 0 {
            return Ok(IrValue::Int(left));
        }
        if self.is_tracked_integer_value(&left) {
            if let Some(result) = self.static_integer_shift(&left, op, count) {
                if result.is_single() {
                    return Ok(IrValue::Int(result.values()[0].to_string()));
                }
            }
        }
        let instruction = match op {
            BinaryOp::ShiftLeft => "shl",
            BinaryOp::ShiftRight => "ashr",
            _ => return Err(self.unsupported(span, LLVM_BITWISE_REJECTION)),
        };
        let temp = self.next_temp();
        self.body
            .push(format!("{temp} = {instruction} i64 {left}, {count}"));
        if let Some(result) = self.static_integer_shift(&left, op, count) {
            self.known_ints.insert(temp.clone(), result);
        }
        Ok(IrValue::Int(temp))
    }

    fn static_integer_shift(&self, left: &str, op: BinaryOp, count: u32) -> Option<KnownInt> {
        let left = self.known_integer_values(left)?;
        let factor = if matches!(op, BinaryOp::ShiftLeft) {
            Some(1_i64.checked_shl(count)?)
        } else {
            None
        };
        let values = left.values().iter().map(|value| match op {
            BinaryOp::ShiftLeft => value.checked_mul(factor.expect("left shift has a factor")),
            BinaryOp::ShiftRight => Some(value >> count),
            _ => None,
        });
        let mut results = Vec::new();
        for value in values {
            results.push(value?);
        }
        KnownInt::from_values(results)
    }

    fn static_integer_shift_count(&self, right: &str) -> Option<u32> {
        if let Ok(count) = right.parse::<u32>() {
            return (count < 64).then_some(count);
        }
        let values = self.known_integer_values(right)?;
        if !values.is_single() {
            return None;
        }
        let count = u32::try_from(values.values()[0]).ok()?;
        (count < 64).then_some(count)
    }

    fn static_integer_bitwise(&self, left: &str, op: BinaryOp, right: &str) -> Option<KnownInt> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                results.push(match op {
                    BinaryOp::BitwiseAnd => left & right,
                    BinaryOp::BitwiseOr => left | right,
                    BinaryOp::BitwiseXor => left ^ right,
                    _ => return None,
                });
            }
        }
        KnownInt::from_values(results)
    }

    fn static_integer_negate(&self, value: &str) -> Option<KnownInt> {
        let value = self.known_integer_values(value)?;
        let mut results = Vec::new();
        for value in value.values() {
            results.push(value.checked_neg()?);
        }
        KnownInt::from_values(results)
    }

    fn static_integer_bitwise_not(&self, value: &str) -> Option<KnownInt> {
        let value = self.known_integer_values(value)?;
        KnownInt::from_values(value.values().iter().map(|value| !value))
    }

    fn emit_integer_bitwise_binary(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        let (IrValue::Int(left), IrValue::Int(right)) = (left, right) else {
            return Err(self.unsupported(span, LLVM_BITWISE_REJECTION));
        };
        if left == right {
            return Ok(match op {
                BinaryOp::BitwiseAnd | BinaryOp::BitwiseOr => IrValue::Int(left),
                BinaryOp::BitwiseXor => IrValue::Int("0".to_string()),
                _ => return Err(self.unsupported(span, LLVM_BITWISE_REJECTION)),
            });
        }
        if matches!(op, BinaryOp::BitwiseAnd) {
            if self
                .known_integer_values(&right)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(IrValue::Int("0".to_string()));
            }
            if self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(IrValue::Int("0".to_string()));
            }
            if self
                .known_integer_values(&right)
                .is_some_and(|values| values.is_single_value(-1))
            {
                return Ok(IrValue::Int(left));
            }
            if self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(-1))
            {
                return Ok(IrValue::Int(right));
            }
        }
        if matches!(op, BinaryOp::BitwiseOr | BinaryOp::BitwiseXor) {
            if self
                .known_integer_values(&right)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(IrValue::Int(left));
            }
            if self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(IrValue::Int(right));
            }
        }
        if matches!(op, BinaryOp::BitwiseOr)
            && (self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(-1))
                || self
                    .known_integer_values(&right)
                    .is_some_and(|values| values.is_single_value(-1)))
        {
            return Ok(IrValue::Int("-1".to_string()));
        }
        if matches!(op, BinaryOp::BitwiseXor)
            && ((self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(-1))
                && self
                    .known_integer_values(&right)
                    .is_some_and(|values| values.is_single()))
                || (self
                    .known_integer_values(&right)
                    .is_some_and(|values| values.is_single_value(-1))
                    && self
                        .known_integer_values(&left)
                        .is_some_and(|values| values.is_single())))
        {
            let result = self
                .static_integer_bitwise(&left, op, &right)
                .expect("single known integer XOR all-ones result is known");
            return Ok(IrValue::Int(result.values()[0].to_string()));
        }
        let left_is_tracked = self.is_tracked_integer_value(&left);
        let right_is_tracked = self.is_tracked_integer_value(&right);
        if left_is_tracked || right_is_tracked {
            if let Some(result) = self.static_integer_bitwise(&left, op, &right) {
                if result.is_single() {
                    return Ok(IrValue::Int(result.values()[0].to_string()));
                }
            }
        }
        let instruction = match op {
            BinaryOp::BitwiseAnd => "and",
            BinaryOp::BitwiseOr => "or",
            BinaryOp::BitwiseXor => "xor",
            _ => return Err(self.unsupported(span, LLVM_BITWISE_REJECTION)),
        };
        let temp = self.next_temp();
        self.body
            .push(format!("{temp} = {instruction} i64 {left}, {right}"));
        if let Some(result) = self.static_integer_bitwise(&left, op, &right) {
            self.known_ints.insert(temp.clone(), result);
        }
        Ok(IrValue::Int(temp))
    }

    fn emit_static_string_concat_expr(
        &mut self,
        left: &Expr,
        right: &Expr,
        span: Span,
    ) -> CompileResult<IrValue> {
        if is_empty_string_literal(left) {
            let right = match self.emit_expr(right) {
                Ok(value) => value,
                Err(error) => return Err(self.unsupported_value_operand_or_original(right, error)),
            };
            return self.emit_empty_string_concat_identity(right, span);
        }
        if is_empty_string_literal(right) {
            let left = match self.emit_expr(left) {
                Ok(value) => value,
                Err(error) => return Err(self.unsupported_value_operand_or_original(left, error)),
            };
            return self.emit_empty_string_concat_identity(left, span);
        }
        let left = match self.emit_static_string_concat_operand(left, span) {
            Ok(value) => value,
            Err(error) => {
                return Err(
                    self.unsupported_unemitted_value_operands_or_original(&[left, right], error)
                );
            }
        };
        let right = self.emit_static_string_concat_operand(right, span)?;
        Ok(IrValue::String(format!("{left}{right}")))
    }

    fn emit_empty_string_concat_identity(
        &self,
        value: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        match value {
            IrValue::String(_) | IrValue::StringPtr(_) => Ok(value),
            _ => Err(self.unsupported(span, LLVM_CONCAT_REJECTION)),
        }
    }

    fn emit_static_string_concat_operand(
        &mut self,
        expr: &Expr,
        span: Span,
    ) -> CompileResult<String> {
        match expr {
            Expr::String(value, _) => Ok(value.clone()),
            Expr::Variable(name, variable_span) => {
                if is_request_superglobal_name(name) {
                    return Err(
                        self.unsupported(*variable_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                match self.variables.get(name).cloned() {
                    Some(IrValue::String(value)) => Ok(value),
                    Some(_) => Err(self.unsupported(span, LLVM_CONCAT_REJECTION)),
                    None => Err(self.unsupported(*variable_span, LLVM_VARIABLE_READ_REJECTION)),
                }
            }
            Expr::Binary {
                left,
                op: BinaryOp::Concat,
                right,
                span: concat_span,
            } => match self.emit_static_string_concat_expr(left, right, *concat_span)? {
                IrValue::String(value) => Ok(value),
                _ => unreachable!("static string concatenation returns a string"),
            },
            Expr::Ternary { .. } => match self.emit_expr(expr)? {
                IrValue::String(value) => Ok(value),
                IrValue::StringPtr(value) => {
                    let values = self
                        .known_string_values(&value)
                        .ok_or_else(|| self.unsupported(span, LLVM_CONCAT_REJECTION))?;
                    if values.is_single() {
                        Ok(values.values()[0].clone())
                    } else {
                        Err(self.unsupported(span, LLVM_CONCAT_REJECTION))
                    }
                }
                _ => Err(self.unsupported(span, LLVM_CONCAT_REJECTION)),
            },
            _ => Err(self.unsupported_value_operand_or_fallback(expr, span, LLVM_CONCAT_REJECTION)),
        }
    }

    fn emit_static_strict_identity(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        let is_identical = match (left, right) {
            (IrValue::Null, IrValue::Null) => true,
            (IrValue::Bool(left), IrValue::Bool(right)) => left == right,
            (IrValue::BoolExpr(left), IrValue::Bool(right)) => {
                if let Some(result) = self.static_bool_strict_identity(
                    self.known_bool_values(&left),
                    Some(KnownBool::one(right)),
                    op,
                ) {
                    return Ok(IrValue::Bool(result));
                }
                if matches!(
                    (op, right),
                    (BinaryOp::StrictEq, true) | (BinaryOp::StrictNe, false)
                ) {
                    return Ok(IrValue::BoolExpr(left));
                }
                if matches!(
                    (op, right),
                    (BinaryOp::StrictEq, false) | (BinaryOp::StrictNe, true)
                ) {
                    return self.emit_bool_not(IrValue::BoolExpr(left), span);
                }
                let right = if right { "true" } else { "false" };
                return self.emit_bool_comparison(left, op, right.to_string(), span);
            }
            (IrValue::Bool(left), IrValue::BoolExpr(right)) => {
                if let Some(result) = self.static_bool_strict_identity(
                    Some(KnownBool::one(left)),
                    self.known_bool_values(&right),
                    op,
                ) {
                    return Ok(IrValue::Bool(result));
                }
                if matches!(
                    (op, left),
                    (BinaryOp::StrictEq, true) | (BinaryOp::StrictNe, false)
                ) {
                    return Ok(IrValue::BoolExpr(right));
                }
                if matches!(
                    (op, left),
                    (BinaryOp::StrictEq, false) | (BinaryOp::StrictNe, true)
                ) {
                    return self.emit_bool_not(IrValue::BoolExpr(right), span);
                }
                let left = if left { "true" } else { "false" };
                return self.emit_bool_comparison(left.to_string(), op, right, span);
            }
            (IrValue::BoolExpr(left), IrValue::BoolExpr(right)) => {
                if left == right {
                    return Ok(IrValue::Bool(static_strict_identity_result(true, op)));
                }
                if let Some(result) = self.static_bool_strict_identity(
                    self.known_bool_values(&left),
                    self.known_bool_values(&right),
                    op,
                ) {
                    return Ok(IrValue::Bool(result));
                }
                return self.emit_bool_comparison(left, op, right, span);
            }
            (IrValue::String(left), IrValue::String(right)) => left == right,
            (IrValue::StringPtr(left), IrValue::StringPtr(right)) => {
                if left == right {
                    return Ok(IrValue::Bool(static_strict_identity_result(true, op)));
                }
                if let Some(result) = self.static_string_strict_identity(
                    self.known_string_values(&left),
                    self.known_string_values(&right),
                    op,
                ) {
                    return Ok(IrValue::Bool(result));
                }
                return self.emit_string_comparison(left, op, right, span);
            }
            (IrValue::StringPtr(left), IrValue::String(right)) => {
                if let Some(result) = self.static_string_strict_identity(
                    self.known_string_values(&left),
                    Some(KnownString::one(right.clone())),
                    op,
                ) {
                    return Ok(IrValue::Bool(result));
                }
                let right = self.string_pointer_operand(IrValue::String(right));
                return self.emit_string_comparison(left, op, right, span);
            }
            (IrValue::String(left), IrValue::StringPtr(right)) => {
                if let Some(result) = self.static_string_strict_identity(
                    Some(KnownString::one(left.clone())),
                    self.known_string_values(&right),
                    op,
                ) {
                    return Ok(IrValue::Bool(result));
                }
                let left = self.string_pointer_operand(IrValue::String(left));
                return self.emit_string_comparison(left, op, right, span);
            }
            (IrValue::Float(left), IrValue::Float(right)) => {
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<f64>(), right.parse::<f64>())
                {
                    return Ok(IrValue::Bool(match op {
                        BinaryOp::StrictEq => left_literal == right_literal,
                        BinaryOp::StrictNe => left_literal != right_literal,
                        _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
                    }));
                }
                if left == right {
                    if let Some(result) = self.static_same_float_strict_identity(&left, op) {
                        return Ok(IrValue::Bool(result));
                    }
                }
                if let Some(result) = self.static_float_strict_identity(&left, op, &right) {
                    return Ok(IrValue::Bool(result));
                }
                let predicate = match op {
                    BinaryOp::StrictEq => "oeq",
                    BinaryOp::StrictNe => "une",
                    _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
                };
                let temp = self.next_temp();
                self.body
                    .push(format!("{temp} = fcmp {predicate} double {left}, {right}"));
                return Ok(IrValue::BoolExpr(temp));
            }
            (IrValue::Int(left), IrValue::Int(right)) => {
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<i64>(), right.parse::<i64>())
                {
                    return Ok(IrValue::Bool(match op {
                        BinaryOp::StrictEq => left_literal == right_literal,
                        BinaryOp::StrictNe => left_literal != right_literal,
                        _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
                    }));
                }
                if left == right {
                    return Ok(IrValue::Bool(static_strict_identity_result(true, op)));
                }
                if let Some(result) = self.static_integer_strict_identity(&left, op, &right) {
                    return Ok(IrValue::Bool(result));
                }
                let predicate = match op {
                    BinaryOp::StrictEq => "eq",
                    BinaryOp::StrictNe => "ne",
                    _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
                };
                let temp = self.next_temp();
                self.body
                    .push(format!("{temp} = icmp {predicate} i64 {left}, {right}"));
                if let Some(result) = self.static_integer_strict_identity_result(&left, op, &right)
                {
                    self.known_bools
                        .insert(temp.clone(), KnownBool::one(result));
                }
                return Ok(IrValue::BoolExpr(temp));
            }
            _ => false,
        };
        let result = match op {
            BinaryOp::StrictEq => is_identical,
            BinaryOp::StrictNe => !is_identical,
            _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
        };
        Ok(IrValue::Bool(result))
    }

    fn static_integer_strict_identity(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        if left_values.is_single() && right_values.is_single() {
            return None;
        }
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_integer_strict_identity_result(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn known_string_values(&self, value: &str) -> Option<KnownString> {
        self.known_strings.get(value).cloned()
    }

    fn known_bool_values(&self, value: &str) -> Option<KnownBool> {
        match value {
            "true" => Some(KnownBool::one(true)),
            "false" => Some(KnownBool::one(false)),
            _ => self.known_bools.get(value).cloned(),
        }
    }

    fn static_bool_strict_identity(
        &self,
        left_values: Option<KnownBool>,
        right_values: Option<KnownBool>,
        op: BinaryOp,
    ) -> Option<bool> {
        let left_values = left_values?;
        let right_values = right_values?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn known_float_values(&self, value: &str) -> Option<KnownFloat> {
        value
            .parse::<f64>()
            .ok()
            .map(KnownFloat::one)
            .or_else(|| self.known_floats.get(value).cloned())
    }

    fn is_tracked_float_value(&self, value: &str) -> bool {
        self.known_floats.contains_key(value)
    }

    fn known_finite_nonzero_float_values(&self, value: &str) -> bool {
        self.known_float_values(value).is_some_and(|values| {
            values
                .values()
                .iter()
                .all(|value| value.is_finite() && *value != 0.0)
        })
    }

    fn known_finite_positive_float_values(&self, value: &str) -> bool {
        self.known_float_values(value).is_some_and(|values| {
            values
                .values()
                .iter()
                .all(|value| value.is_finite() && *value > 0.0)
        })
    }

    fn static_float_strict_identity(&self, left: &str, op: BinaryOp, right: &str) -> Option<bool> {
        let left_values = self.known_float_values(left)?;
        let right_values = self.known_float_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_same_float_strict_identity(&self, value: &str, op: BinaryOp) -> Option<bool> {
        let values = self.known_float_values(value)?;
        if !values.values().iter().all(|value| value.is_finite()) {
            return None;
        }
        Some(static_strict_identity_result(true, op))
    }

    fn static_float_arithmetic(&self, left: &str, op: BinaryOp, right: &str) -> Option<KnownFloat> {
        let left_values = self.known_float_values(left)?;
        let right_values = self.known_float_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                let result = match op {
                    BinaryOp::Add => left + right,
                    BinaryOp::Sub => left - right,
                    BinaryOp::Mul => left * right,
                    _ => return None,
                };
                if !result.is_finite() {
                    return None;
                }
                results.push(result);
            }
        }
        KnownFloat::from_values(results)
    }

    fn known_string_values_for_value(&self, value: &IrValue) -> Option<KnownString> {
        match value {
            IrValue::String(value) => Some(KnownString::one(value.clone())),
            IrValue::StringPtr(value) => self.known_string_values(value),
            _ => None,
        }
    }

    fn static_string_strict_identity(
        &self,
        left_values: Option<KnownString>,
        right_values: Option<KnownString>,
        op: BinaryOp,
    ) -> Option<bool> {
        let left_values = left_values?;
        let right_values = right_values?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn emit_bool_binary(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        if let (Some(left), Some(right)) = (
            self.known_truthiness_for_value(&left),
            self.known_truthiness_for_value(&right),
        ) {
            return Ok(IrValue::Bool(logical_truthiness_result(left, op, right)?));
        }
        match (left, right) {
            (IrValue::Bool(left), IrValue::Bool(right)) => Ok(IrValue::Bool(match op {
                BinaryOp::LogicalAnd => left && right,
                BinaryOp::LogicalOr => left || right,
                BinaryOp::LogicalXor => left ^ right,
                _ => return Err(self.unsupported(span, llvm_logical_rejection())),
            })),
            (IrValue::Bool(left), right) => match op {
                BinaryOp::LogicalAnd if left => self.require_bool_value(right, span),
                BinaryOp::LogicalAnd => Ok(IrValue::Bool(false)),
                BinaryOp::LogicalOr if left => Ok(IrValue::Bool(true)),
                BinaryOp::LogicalOr => self.require_bool_value(right, span),
                BinaryOp::LogicalXor if left => {
                    let right = self.require_bool_value(right, span)?;
                    self.emit_bool_not(right, span)
                }
                BinaryOp::LogicalXor => self.require_bool_value(right, span),
                _ => Err(self.unsupported(span, llvm_logical_rejection())),
            },
            (left, IrValue::Bool(right)) => match op {
                BinaryOp::LogicalAnd if right => self.require_bool_value(left, span),
                BinaryOp::LogicalAnd => Ok(IrValue::Bool(false)),
                BinaryOp::LogicalOr if right => Ok(IrValue::Bool(true)),
                BinaryOp::LogicalOr => self.require_bool_value(left, span),
                BinaryOp::LogicalXor if right => {
                    let left = self.require_bool_value(left, span)?;
                    self.emit_bool_not(left, span)
                }
                BinaryOp::LogicalXor => self.require_bool_value(left, span),
                _ => Err(self.unsupported(span, llvm_logical_rejection())),
            },
            (left, right) => {
                let Some(left) = llvm_bool_operand(left) else {
                    return Err(self.unsupported(span, llvm_logical_rejection()));
                };
                let Some(right) = llvm_bool_operand(right) else {
                    return Err(self.unsupported(span, llvm_logical_rejection()));
                };
                if left == right && matches!(op, BinaryOp::LogicalAnd | BinaryOp::LogicalOr) {
                    return Ok(IrValue::BoolExpr(left));
                }
                if left == right && matches!(op, BinaryOp::LogicalXor) {
                    return Ok(IrValue::Bool(false));
                }
                let result = self.static_bool_binary(&left, op, &right);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(IrValue::Bool(result.values()[0]));
                    }
                }
                let instruction = match op {
                    BinaryOp::LogicalAnd => "and",
                    BinaryOp::LogicalOr => "or",
                    BinaryOp::LogicalXor => "xor",
                    _ => return Err(self.unsupported(span, llvm_logical_rejection())),
                };
                let temp = self.next_temp();
                self.body
                    .push(format!("{temp} = {instruction} i1 {left}, {right}"));
                if let Some(result) = result {
                    self.known_bools.insert(temp.clone(), result);
                }
                Ok(IrValue::BoolExpr(temp))
            }
        }
    }

    fn emit_logical_expr(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
        span: Span,
    ) -> CompileResult<IrValue> {
        let left = self.emit_expr(left)?;
        if let Some(left_truthy) = self.known_truthiness_for_value(&left) {
            match op {
                BinaryOp::LogicalAnd if !left_truthy => return Ok(IrValue::Bool(false)),
                BinaryOp::LogicalOr if left_truthy => return Ok(IrValue::Bool(true)),
                _ => {}
            }
        }
        let right = self.emit_expr(right)?;
        self.emit_bool_binary(left, op, right, span)
    }

    fn require_bool_value(&self, value: IrValue, span: Span) -> CompileResult<IrValue> {
        match value {
            IrValue::Bool(_) | IrValue::BoolExpr(_) => Ok(value),
            _ => Err(self.unsupported(span, llvm_logical_rejection())),
        }
    }

    fn known_truthiness_for_value(&self, value: &IrValue) -> Option<bool> {
        match value {
            IrValue::Bool(value) => Some(*value),
            IrValue::BoolExpr(_) => None,
            IrValue::Int(value) => known_integer_truthiness(&self.known_integer_values(value)),
            IrValue::Float(value) => known_float_truthiness(&self.known_float_values(value)),
            IrValue::String(value) => Some(is_php_truthy_string(value)),
            IrValue::StringPtr(value) => self
                .known_string_values(value)
                .and_then(|values| known_string_truthiness(&values)),
            IrValue::NativeValue(_) => None,
            IrValue::Null => Some(false),
        }
    }

    fn emit_bool_comparison(
        &mut self,
        left: String,
        op: BinaryOp,
        right: String,
        span: Span,
    ) -> CompileResult<IrValue> {
        let predicate = match op {
            BinaryOp::StrictEq => "eq",
            BinaryOp::StrictNe => "ne",
            _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
        };
        let temp = self.next_temp();
        self.body
            .push(format!("{temp} = icmp {predicate} i1 {left}, {right}"));
        Ok(IrValue::BoolExpr(temp))
    }

    fn static_bool_binary(&self, left: &str, op: BinaryOp, right: &str) -> Option<KnownBool> {
        let left_values = self.known_bool_values(left)?;
        let right_values = self.known_bool_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                results.push(match op {
                    BinaryOp::LogicalAnd => *left && *right,
                    BinaryOp::LogicalOr => *left || *right,
                    BinaryOp::LogicalXor => *left ^ *right,
                    _ => return None,
                });
            }
        }
        KnownBool::from_values(results)
    }

    fn emit_string_comparison(
        &mut self,
        left: String,
        op: BinaryOp,
        right: String,
        span: Span,
    ) -> CompileResult<IrValue> {
        let predicate = llvm_string_comparison_predicate(op)
            .ok_or_else(|| self.unsupported(span, llvm_comparison_rejection()))?;
        if left == right {
            let Some(result) = reflexive_string_comparison_result(op) else {
                return Err(self.unsupported(span, llvm_comparison_rejection()));
            };
            return Ok(IrValue::Bool(result));
        }
        let known_result = if matches!(op, BinaryOp::StrictEq | BinaryOp::StrictNe) {
            self.static_string_strict_identity(
                self.known_string_values(&left),
                self.known_string_values(&right),
                op,
            )
        } else {
            let left_values = self
                .known_string_values(&left)
                .ok_or_else(|| self.unsupported(span, llvm_comparison_rejection()))?;
            let right_values = self
                .known_string_values(&right)
                .ok_or_else(|| self.unsupported(span, llvm_comparison_rejection()))?;
            if !known_string_pairs_are_safe_for_native_comparison(&left_values, &right_values) {
                return Err(self.unsupported(span, llvm_comparison_rejection()));
            }
            string_comparison_result_for_known_values(&left_values, op, &right_values)
        };
        if let Some(known_result) = known_result {
            return Ok(IrValue::Bool(known_result));
        }
        self.uses_strcmp = true;
        let comparison = self.next_temp();
        self.body.push(format!(
            "{comparison} = call i32 @strcmp(ptr {left}, ptr {right})"
        ));
        let result = self.next_temp();
        self.body
            .push(format!("{result} = icmp {predicate} i32 {comparison}, 0"));
        if let Some(known_result) = known_result {
            self.known_bools
                .insert(result.clone(), KnownBool::one(known_result));
        }
        Ok(IrValue::BoolExpr(result))
    }

    fn emit_ternary(
        &mut self,
        condition: IrValue,
        if_true: IrValue,
        if_false: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        match condition {
            IrValue::Bool(true) => return Ok(if_true),
            IrValue::Bool(false) => return Ok(if_false),
            IrValue::Int(value) => {
                let Some(values) = self.known_integer_values(&value) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                if values.values().iter().all(|value| *value != 0) {
                    return Ok(if_true);
                }
                if values.is_single_value(0) {
                    return Ok(if_false);
                }
                Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION))
            }
            IrValue::Float(value) => {
                let Some(values) = self.known_float_values(&value) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                if !values.values().iter().all(|value| value.is_finite()) {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                }
                if values.values().iter().all(|value| *value != 0.0) {
                    return Ok(if_true);
                }
                if matches!(values.values(), [value] if *value == 0.0) {
                    return Ok(if_false);
                }
                Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION))
            }
            IrValue::String(value) => {
                if is_php_truthy_string(&value) {
                    Ok(if_true)
                } else {
                    Ok(if_false)
                }
            }
            IrValue::StringPtr(value) => {
                let Some(values) = self.known_string_values(&value) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                match known_string_truthiness(&values) {
                    Some(true) => Ok(if_true),
                    Some(false) => Ok(if_false),
                    None => Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION)),
                }
            }
            IrValue::Null => Ok(if_false),
            condition => self.emit_dynamic_ternary(condition, if_true, if_false, span),
        }
    }

    fn emit_ternary_expr(
        &mut self,
        condition: &Expr,
        if_true: &Expr,
        if_false: &Expr,
        span: Span,
    ) -> CompileResult<IrValue> {
        let condition_value = self.emit_expr(condition)?;
        if same_direct_variable_ternary_expr(condition, if_true, if_false) {
            return Ok(condition_value);
        }
        if let Some(truthy) = self.known_truthiness_for_value(&condition_value) {
            return if truthy {
                self.emit_expr(if_true)
            } else {
                self.emit_expr(if_false)
            };
        }
        if !matches!(condition_value, IrValue::BoolExpr(_)) {
            return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
        }
        let if_true = self.emit_expr(if_true)?;
        let if_false = self.emit_expr(if_false)?;
        self.emit_ternary(condition_value, if_true, if_false, span)
    }

    fn emit_short_ternary(
        &mut self,
        condition: &Expr,
        if_false: &Expr,
        span: Span,
    ) -> CompileResult<IrValue> {
        let condition_value = self.emit_expr(condition)?;
        if same_direct_variable_expr(condition, if_false) {
            if matches!(
                condition_value,
                IrValue::BoolExpr(_) | IrValue::Int(_) | IrValue::Float(_) | IrValue::StringPtr(_)
            ) {
                return Ok(condition_value);
            }
        }
        match condition_value {
            IrValue::Bool(true) => Ok(IrValue::Bool(true)),
            IrValue::Bool(false) => {
                let if_false = self.emit_expr(if_false)?;
                Ok(if_false)
            }
            IrValue::Int(value) => {
                let Some(values) = self.known_integer_values(&value) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                if values.values().iter().all(|value| *value != 0) {
                    Ok(IrValue::Int(value))
                } else if values.is_single_value(0) {
                    self.emit_expr(if_false)
                } else {
                    Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION))
                }
            }
            IrValue::Float(value) => {
                let Some(values) = self.known_float_values(&value) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                if !values.values().iter().all(|value| value.is_finite()) {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                }
                if values.values().iter().all(|value| *value != 0.0) {
                    Ok(IrValue::Float(value))
                } else if matches!(values.values(), [value] if *value == 0.0) {
                    self.emit_expr(if_false)
                } else {
                    Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION))
                }
            }
            IrValue::String(value) => {
                if is_php_truthy_string(&value) {
                    Ok(IrValue::String(value))
                } else {
                    self.emit_expr(if_false)
                }
            }
            IrValue::StringPtr(value) => {
                let Some(values) = self.known_string_values(&value) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                match known_string_truthiness(&values) {
                    Some(true) => Ok(IrValue::StringPtr(value)),
                    Some(false) => self.emit_expr(if_false),
                    None => Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION)),
                }
            }
            IrValue::Null => self.emit_expr(if_false),
            IrValue::NativeValue(_) => Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION)),
            condition @ IrValue::BoolExpr(_) => {
                let if_false = self.emit_expr(if_false)?;
                if !matches!(if_false, IrValue::Bool(_) | IrValue::BoolExpr(_)) {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                }
                self.emit_ternary(condition, IrValue::Bool(true), if_false, span)
            }
        }
    }

    fn emit_dynamic_ternary(
        &mut self,
        condition: IrValue,
        if_true: IrValue,
        if_false: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        let Some(condition) = llvm_bool_operand(condition) else {
            return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
        };
        match (if_true, if_false) {
            (IrValue::Null, IrValue::Null) => Ok(IrValue::Null),
            (IrValue::Int(if_true), IrValue::Int(if_false)) => {
                if if_true == if_false {
                    return Ok(IrValue::Int(if_true));
                }
                let result = self.static_integer_ternary(&if_true, &if_false);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(IrValue::Int(result.values()[0].to_string()));
                    }
                }
                let temp = self.next_temp();
                self.body.push(format!(
                    "{temp} = select i1 {condition}, i64 {if_true}, i64 {if_false}"
                ));
                if let Some(result) = result {
                    self.known_ints.insert(temp.clone(), result);
                }
                Ok(IrValue::Int(temp))
            }
            (IrValue::Float(if_true), IrValue::Float(if_false)) => {
                if if_true == if_false {
                    return Ok(IrValue::Float(if_true));
                }
                let result = self.static_float_ternary(&if_true, &if_false);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(IrValue::Float(format_float_literal(result.values()[0])));
                    }
                }
                let temp = self.next_temp();
                self.body.push(format!(
                    "{temp} = select i1 {condition}, double {if_true}, double {if_false}"
                ));
                if let Some(result) = result {
                    self.known_floats.insert(temp.clone(), result);
                }
                Ok(IrValue::Float(temp))
            }
            (if_true, if_false) => {
                if matches!(
                    (&if_true, &if_false),
                    (
                        IrValue::String(_) | IrValue::StringPtr(_),
                        IrValue::String(_) | IrValue::StringPtr(_)
                    )
                ) {
                    if let Some(result) = identical_string_ternary_branch(&if_true, &if_false) {
                        return Ok(result);
                    }
                    let result = self.static_string_ternary(&if_true, &if_false);
                    if let Some(result) = result.as_ref() {
                        if result.is_single() {
                            return Ok(IrValue::String(result.values()[0].clone()));
                        }
                    }
                    let if_true = self.string_pointer_operand(if_true);
                    let if_false = self.string_pointer_operand(if_false);
                    let if_true_len = self.string_pointer_byte_len_operand(&if_true);
                    let if_false_len = self.string_pointer_byte_len_operand(&if_false);
                    let temp = self.next_temp();
                    self.body.push(format!(
                        "{temp} = select i1 {condition}, ptr {if_true}, ptr {if_false}"
                    ));
                    match (if_true_len, if_false_len) {
                        (Some(if_true_len), Some(if_false_len)) if if_true_len == if_false_len => {
                            self.string_lengths.insert(temp.clone(), if_true_len);
                        }
                        (Some(if_true_len), Some(if_false_len)) => {
                            let len_temp = self.next_temp();
                            let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
                            self.body.push(format!(
                                "{len_temp} = select i1 {condition}, {usize_type} {if_true_len}, {usize_type} {if_false_len}"
                            ));
                            self.string_lengths.insert(temp.clone(), len_temp);
                        }
                        _ => {}
                    }
                    if let Some(result) = result {
                        self.known_strings.insert(temp.clone(), result);
                    }
                    return Ok(IrValue::StringPtr(temp));
                }
                if let Some(result) = identical_bool_expr_ternary_branch(&if_true, &if_false) {
                    return Ok(result);
                }
                if let Some(result) = bool_literal_ternary_branch(&condition, &if_true, &if_false) {
                    return match result {
                        BoolLiteralTernaryBranch::Static(value) => Ok(IrValue::Bool(value)),
                        BoolLiteralTernaryBranch::Reuse(value) => Ok(IrValue::BoolExpr(value)),
                        BoolLiteralTernaryBranch::Invert(value) => {
                            self.emit_bool_not(IrValue::BoolExpr(value), span)
                        }
                    };
                }
                let Some(if_true) = llvm_bool_operand(if_true) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                let Some(if_false) = llvm_bool_operand(if_false) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                let result = self.static_bool_ternary(&if_true, &if_false);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(IrValue::Bool(result.values()[0]));
                    }
                }
                let temp = self.next_temp();
                self.body.push(format!(
                    "{temp} = select i1 {condition}, i1 {if_true}, i1 {if_false}"
                ));
                if let Some(result) = result {
                    self.known_bools.insert(temp.clone(), result);
                }
                Ok(IrValue::BoolExpr(temp))
            }
        }
    }

    fn static_integer_ternary(&self, if_true: &str, if_false: &str) -> Option<KnownInt> {
        let if_true = self.known_integer_values(if_true)?;
        let if_false = self.known_integer_values(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values());
        KnownInt::from_values(values)
    }

    fn static_float_ternary(&self, if_true: &str, if_false: &str) -> Option<KnownFloat> {
        let if_true = self.known_float_values(if_true)?;
        let if_false = self.known_float_values(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values());
        KnownFloat::from_values(values)
    }

    fn static_string_ternary(&self, if_true: &IrValue, if_false: &IrValue) -> Option<KnownString> {
        let if_true = self.known_string_values_for_value(if_true)?;
        let if_false = self.known_string_values_for_value(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values().iter().cloned());
        KnownString::from_values(values)
    }

    fn static_bool_ternary(&self, if_true: &str, if_false: &str) -> Option<KnownBool> {
        let if_true = self.known_bool_values(if_true)?;
        let if_false = self.known_bool_values(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values());
        KnownBool::from_values(values)
    }

    fn emit_unary(&mut self, op: UnaryOp, value: IrValue, span: Span) -> CompileResult<IrValue> {
        match op {
            UnaryOp::Negate => self.emit_numeric_negate(value, span),
            UnaryOp::Not => self.emit_bool_not(value, span),
            UnaryOp::BitwiseNot => self.emit_integer_bitwise_not(value, span),
        }
    }

    fn primitive_source_for_value(&self, value: &IrValue) -> Option<BackendPrimitiveSource> {
        match value {
            IrValue::Null => Some(BackendPrimitiveSource::null()),
            IrValue::Bool(value) => Some(BackendPrimitiveSource::bool_value(*value)),
            IrValue::BoolExpr(value) => Some(
                self.known_bool_values(value)
                    .map(BackendPrimitiveSource::bool_values)
                    .unwrap_or_else(BackendPrimitiveSource::unknown_bool),
            ),
            IrValue::Int(value) => Some(
                self.known_integer_values(value)
                    .map(BackendPrimitiveSource::int_values)
                    .unwrap_or_else(BackendPrimitiveSource::unknown_int),
            ),
            IrValue::Float(value) => Some(
                self.known_float_values(value)
                    .map(BackendPrimitiveSource::float_values)
                    .unwrap_or_else(BackendPrimitiveSource::unknown_float),
            ),
            IrValue::String(value) => Some(BackendPrimitiveSource::string_value(value)),
            IrValue::StringPtr(value) => self
                .known_string_values(value)
                .map(BackendPrimitiveSource::string_values),
            IrValue::NativeValue(_) => None,
        }
    }

    fn checked_static_primitive_negate_result_for_value(
        &self,
        value: &IrValue,
    ) -> Result<Option<BackendArithmeticResult>, PhpPrimitiveArithmeticError> {
        self.primitive_source_for_value(value)
            .map(|source| source.single_arithmetic_result(PhpPrimitiveArithmeticOperation::Negate))
            .unwrap_or(Ok(None))
    }

    fn checked_static_primitive_arithmetic_result_for_values(
        &self,
        left: &IrValue,
        op: BinaryOp,
        right: &IrValue,
    ) -> Result<Option<BackendArithmeticResult>, PhpPrimitiveArithmeticError> {
        let Some(operation) = backend_binary_primitive_arithmetic_operation(op) else {
            return Ok(None);
        };
        let Some(left) = self.primitive_source_for_value(left) else {
            return Ok(None);
        };
        let Some(right) = self.primitive_source_for_value(right) else {
            return Ok(None);
        };
        left.pair_arithmetic_result(operation, &right)
    }

    fn emit_numeric_negate(&mut self, value: IrValue, span: Span) -> CompileResult<IrValue> {
        if !matches!(value, IrValue::Int(_) | IrValue::Float(_)) {
            match self.checked_static_primitive_negate_result_for_value(&value) {
                Ok(Some(result)) => {
                    if let Some(value) = result.into_single_ir_value() {
                        return Ok(value);
                    }
                }
                Ok(None)
                | Err(
                    PhpPrimitiveArithmeticError::MissingRightOperand
                    | PhpPrimitiveArithmeticError::Conversion(_)
                    | PhpPrimitiveArithmeticError::IntegerOverflow
                    | PhpPrimitiveArithmeticError::NonFiniteFloat,
                ) => {}
            }
        }

        match value {
            IrValue::Int(value) => {
                let Some(result) = self.static_integer_negate(&value) else {
                    return Err(self.unsupported(span, LLVM_UNARY_REJECTION));
                };
                if result.is_single() {
                    return Ok(IrValue::Int(result.values()[0].to_string()));
                }
                let temp = self.next_temp();
                self.body.push(format!("{temp} = sub i64 0, {value}"));
                self.known_ints.insert(temp.clone(), result);
                Ok(IrValue::Int(temp))
            }
            IrValue::Float(value) => {
                if let Some(result) = self.static_float_negate(&value) {
                    if result.is_single() && result.values()[0] != 0.0 {
                        return Ok(IrValue::Float(format_float_literal(result.values()[0])));
                    }
                }
                let temp = self.next_temp();
                self.body.push(format!("{temp} = fsub double 0.0, {value}"));
                if let Some(result) = self.static_float_negate(&value) {
                    self.known_floats.insert(temp.clone(), result);
                }
                Ok(IrValue::Float(temp))
            }
            _ => Err(self.unsupported(span, LLVM_UNARY_REJECTION)),
        }
    }

    fn emit_integer_bitwise_not(&mut self, value: IrValue, span: Span) -> CompileResult<IrValue> {
        let IrValue::Int(value) = value else {
            return Err(self.unsupported(span, LLVM_BITWISE_REJECTION));
        };
        if let Some(result) = self.static_integer_bitwise_not(&value) {
            if result.is_single() {
                return Ok(IrValue::Int(result.values()[0].to_string()));
            }
        }
        let temp = self.next_temp();
        self.body.push(format!("{temp} = xor i64 {value}, -1"));
        if let Some(result) = self.static_integer_bitwise_not(&value) {
            self.known_ints.insert(temp.clone(), result);
        }
        Ok(IrValue::Int(temp))
    }

    fn emit_bool_not(&mut self, value: IrValue, span: Span) -> CompileResult<IrValue> {
        match value {
            IrValue::Bool(value) => Ok(IrValue::Bool(!value)),
            IrValue::BoolExpr(value) => {
                if let Some(result) = self.static_bool_not(&value) {
                    if result.is_single() {
                        return Ok(IrValue::Bool(result.values()[0]));
                    }
                }
                let temp = self.next_temp();
                self.body.push(format!("{temp} = xor i1 {value}, true"));
                if let Some(result) = self.static_bool_not(&value) {
                    self.known_bools.insert(temp.clone(), result);
                }
                Ok(IrValue::BoolExpr(temp))
            }
            IrValue::Int(value) => {
                let Some(truthy) = known_integer_truthiness(&self.known_integer_values(&value))
                else {
                    return Err(self.unsupported(span, LLVM_UNARY_REJECTION));
                };
                Ok(IrValue::Bool(!truthy))
            }
            IrValue::Float(value) => {
                let Some(truthy) = known_float_truthiness(&self.known_float_values(&value)) else {
                    return Err(self.unsupported(span, LLVM_UNARY_REJECTION));
                };
                Ok(IrValue::Bool(!truthy))
            }
            IrValue::String(value) => Ok(IrValue::Bool(!is_php_truthy_string(&value))),
            IrValue::StringPtr(value) => {
                let Some(values) = self.known_string_values(&value) else {
                    return Err(self.unsupported(span, LLVM_UNARY_REJECTION));
                };
                match known_string_truthiness(&values) {
                    Some(value) => Ok(IrValue::Bool(!value)),
                    None => Err(self.unsupported(span, LLVM_UNARY_REJECTION)),
                }
            }
            IrValue::Null => Ok(IrValue::Bool(true)),
            IrValue::NativeValue(_) => Err(self.unsupported(span, LLVM_UNARY_REJECTION)),
        }
    }

    fn static_bool_not(&self, value: &str) -> Option<KnownBool> {
        let value = self.known_bool_values(value)?;
        KnownBool::from_values(value.values().iter().map(|value| !value))
    }

    fn static_float_negate(&self, value: &str) -> Option<KnownFloat> {
        let value = self.known_float_values(value)?;
        let mut results = Vec::new();
        for value in value.values() {
            let result = -value;
            if !result.is_finite() {
                return None;
            }
            results.push(result);
        }
        KnownFloat::from_values(results)
    }

    fn emit_echo(&mut self, value: IrValue) {
        match value {
            IrValue::Null | IrValue::Bool(false) => {}
            IrValue::Bool(true) => {
                let global = self.add_string("1");
                self.body.push(format!(
                    "call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr @{global})"
                ));
            }
            IrValue::BoolExpr(value) => {
                let true_global = self.add_string("1");
                let false_global = self.add_string("");
                let temp = self.next_temp();
                self.body.push(format!(
                    "{temp} = select i1 {value}, ptr @{true_global}, ptr @{false_global}"
                ));
                self.body.push(format!(
                    "call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr {temp})"
                ));
            }
            IrValue::Int(value) => {
                self.body.push(format!(
                    "call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 {value})"
                ));
            }
            IrValue::Float(value) => {
                self.body.push(format!(
                    "call i32 (ptr, ...) @printf(ptr @.fmt_float, double {value})"
                ));
            }
            IrValue::String(value) => self.emit_native_value_string_stdout(&value),
            IrValue::StringPtr(value) => {
                if let Some(len) = self.known_string_pointer_byte_len_operand(&value) {
                    self.emit_native_value_string_ptr_stdout(&value, &len);
                } else {
                    self.body.push(format!(
                        "call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr {value})"
                    ));
                }
            }
            IrValue::NativeValue(value) => self.emit_native_value_handle_stdout(&value),
        }
    }

    fn emit_print(&mut self, value: IrValue) {
        match value {
            IrValue::String(value) => self.emit_native_value_string_stdout(&value),
            value => self.emit_echo(value),
        }
    }

    fn emit_native_value_string_stdout(&mut self, value: &str) {
        let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
        let global = self.add_string(value);
        self.emit_native_value_string_pointer_stdout(
            &format!("@{global}"),
            usize_type,
            &value.len().to_string(),
        );
    }

    fn emit_native_value_string_ptr_stdout(&mut self, value: &str, len: &str) {
        let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
        self.emit_native_value_string_pointer_stdout(value, usize_type, len);
    }

    fn emit_native_value_handle_stdout(&mut self, value: &str) {
        let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
        self.uses_native_value_echo_stdout = true;
        self.body.push(format!(
            "call {usize_type} @phpc_native_value_echo_stdout(%phpc.NativeValueHandle {value})"
        ));
        self.body.push(format!(
            "call void @phpc_native_value_free(%phpc.NativeValueHandle {value})"
        ));
    }

    fn emit_native_value_string_pointer_stdout(
        &mut self,
        value: &str,
        usize_type: &str,
        len: &str,
    ) {
        let string = self.next_temp();
        let diagnostic_slot = self.next_temp();
        let runtime_value = self.next_temp();
        let runtime_value_ptr = self.next_temp();
        let value_failed = self.next_temp();
        let diagnostic = self.next_temp();
        let report_label = self.next_label("native_report_diagnostic");
        let echo_label = self.next_label("native_echo_value");
        let cleanup_label = self.next_label("native_cleanup_string_value");
        self.uses_native_value_echo_stdout = true;
        self.body.push(format!(
            "{diagnostic_slot} = alloca %phpc.NativeDiagnosticHandle"
        ));
        self.body.push(format!(
            "store %phpc.NativeDiagnosticHandle zeroinitializer, ptr {diagnostic_slot}"
        ));
        self.body.push(format!(
            "{string} = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr {value}, {usize_type} {len})"
        ));
        self.body.push(format!(
            "{runtime_value} = call %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle {string}, ptr {diagnostic_slot})"
        ));
        self.body.push(format!(
            "{runtime_value_ptr} = extractvalue %phpc.NativeValueHandle {runtime_value}, 0"
        ));
        self.body.push(format!(
            "{value_failed} = icmp eq ptr {runtime_value_ptr}, null"
        ));
        self.body.push(format!(
            "br i1 {value_failed}, label %{report_label}, label %{echo_label}"
        ));
        self.body.push(format!("{report_label}:"));
        self.body.push(format!(
            "{diagnostic} = load %phpc.NativeDiagnosticHandle, ptr {diagnostic_slot}"
        ));
        self.body.push(format!(
            "call {usize_type} @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle {diagnostic})"
        ));
        self.body.push(format!(
            "call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle {diagnostic})"
        ));
        self.body.push(format!("br label %{cleanup_label}"));
        self.body.push(format!("{echo_label}:"));
        self.body.push(format!(
            "call {usize_type} @phpc_native_value_echo_stdout(%phpc.NativeValueHandle {runtime_value})"
        ));
        self.body.push(format!("br label %{cleanup_label}"));
        self.body.push(format!("{cleanup_label}:"));
        self.body.push(format!(
            "call void @phpc_native_value_free(%phpc.NativeValueHandle {runtime_value})"
        ));
        self.body.push(format!(
            "call void @phpc_native_string_free(%phpc.NativeStringHandle {string})"
        ));
    }

    fn emit_native_int_for_ir_value(
        &mut self,
        value: IrValue,
        operation: NativeIntConversionOperation,
        span: Span,
        rejection: &'static str,
    ) -> CompileResult<String> {
        let value_handle = self
            .emit_native_value_for_ir_value(value, span)
            .map_err(|_| self.unsupported(span, rejection))?;
        let diagnostic_slot = self.next_temp();
        let result = self.next_temp();
        self.uses_native_string_int_operation = true;
        self.body.push(format!(
            "{diagnostic_slot} = alloca %phpc.NativeDiagnosticHandle"
        ));
        self.body.push(format!(
            "store %phpc.NativeDiagnosticHandle zeroinitializer, ptr {diagnostic_slot}"
        ));
        self.body.push(format!(
            "{result} = call i64 @phpc_native_value_to_int64_with_diagnostic(%phpc.NativeValueHandle {value_handle}, i8 {}, ptr {diagnostic_slot})",
            operation as u8
        ));
        self.emit_report_native_diagnostic_slot(&diagnostic_slot);
        self.body.push(format!(
            "call void @phpc_native_value_free(%phpc.NativeValueHandle {value_handle})"
        ));
        Ok(result)
    }

    fn emit_native_value_string_len(
        &mut self,
        value: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        let IrValue::NativeValue(value) = value else {
            return Err(self.unsupported_direct_call(span, NativeCallBlocker::ReturnValueOwnership));
        };
        let conversion = self.next_temp();
        let diagnostic = self.next_temp();
        let bytes = self.next_temp();
        let len = self.next_temp();
        self.uses_native_value_offset_operation = true;
        self.body.push(format!(
            "{conversion} = call %phpc.NativeStringConversionResult @phpc_native_value_to_string_bytes(%phpc.NativeValueHandle {value})"
        ));
        self.body.push(format!(
            "{diagnostic} = extractvalue %phpc.NativeStringConversionResult {conversion}, 1"
        ));
        self.emit_native_diagnostic_message_handle(&diagnostic);
        self.body.push(format!(
            "{bytes} = extractvalue %phpc.NativeStringConversionResult {conversion}, 0"
        ));
        let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
        self.body.push(format!(
            "{len} = extractvalue %phpc.NativeByteBuffer {bytes}, 1"
        ));
        self.body.push(format!(
            "call void @phpc_native_string_conversion_result_free(%phpc.NativeStringConversionResult {conversion})"
        ));
        self.body.push(format!(
            "call void @phpc_native_value_free(%phpc.NativeValueHandle {value})"
        ));
        if usize_type == "i64" {
            Ok(IrValue::Int(len))
        } else {
            let widened = self.next_temp();
            self.body
                .push(format!("{widened} = zext {usize_type} {len} to i64"));
            Ok(IrValue::Int(widened))
        }
    }

    fn emit_native_value_for_ir_value(
        &mut self,
        value: IrValue,
        span: Span,
    ) -> CompileResult<String> {
        self.uses_native_string_int_operation = true;
        match value {
            IrValue::Null => Ok(self.emit_native_value_from_scalar_call("@phpc_native_null()")),
            IrValue::Bool(value) => Ok(self.emit_native_value_from_scalar_call(&format!(
                "@phpc_native_bool(i1 {})",
                if value { "true" } else { "false" }
            ))),
            IrValue::BoolExpr(value) => {
                Ok(self
                    .emit_native_value_from_scalar_call(&format!("@phpc_native_bool(i1 {value})")))
            }
            IrValue::Int(value) => {
                Ok(self
                    .emit_native_value_from_scalar_call(&format!("@phpc_native_int(i64 {value})")))
            }
            IrValue::Float(value) => Ok(self.emit_native_value_from_scalar_call(&format!(
                "@phpc_native_float(double {value})"
            ))),
            IrValue::String(value) => {
                let global = self.add_string(&value);
                Ok(self.emit_native_value_from_string_bytes(
                    &format!("@{global}"),
                    &value.len().to_string(),
                ))
            }
            IrValue::StringPtr(value) => {
                let len = self
                    .string_pointer_byte_len_operand(&value)
                    .ok_or_else(|| self.unsupported(span, LLVM_STRING_INT_OPERATION_REJECTION))?;
                Ok(self.emit_native_value_from_string_bytes(&value, &len))
            }
            IrValue::NativeValue(value) => Ok(value),
        }
    }

    fn emit_native_value_from_scalar_call(&mut self, scalar_call: &str) -> String {
        let scalar = self.next_temp();
        let handle = self.next_temp();
        self.uses_native_string_int_operation = true;
        self.body.push(format!(
            "{scalar} = call %phpc.NativeScalarValue {scalar_call}"
        ));
        self.body.push(format!(
            "{handle} = call %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue {scalar})"
        ));
        handle
    }

    fn emit_native_value_from_string_bytes(&mut self, ptr: &str, len: &str) -> String {
        let diagnostic_slot = self.next_temp();
        let handle = self.next_temp();
        let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
        self.uses_native_string_int_operation = true;
        self.body.push(format!(
            "{diagnostic_slot} = alloca %phpc.NativeDiagnosticHandle"
        ));
        self.body.push(format!(
            "store %phpc.NativeDiagnosticHandle zeroinitializer, ptr {diagnostic_slot}"
        ));
        self.body.push(format!(
            "{handle} = call %phpc.NativeValueHandle @phpc_native_value_from_string_bytes_with_diagnostic(ptr {ptr}, {usize_type} {len}, ptr {diagnostic_slot})"
        ));
        self.emit_report_native_diagnostic_slot(&diagnostic_slot);
        handle
    }

    fn emit_report_native_diagnostic_slot(&mut self, diagnostic_slot: &str) {
        let diagnostic = self.next_temp();
        self.body.push(format!(
            "{diagnostic} = load %phpc.NativeDiagnosticHandle, ptr {diagnostic_slot}"
        ));
        self.emit_report_native_diagnostic_handle(&diagnostic);
    }

    fn emit_report_native_diagnostic_handle(&mut self, diagnostic: &str) {
        self.emit_native_diagnostic_message_handle(diagnostic);
        self.body.push(format!(
            "call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle {diagnostic})"
        ));
    }

    fn emit_native_diagnostic_message_handle(&mut self, diagnostic: &str) {
        let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
        self.body.push(format!(
            "call {usize_type} @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle {diagnostic})"
        ));
    }

    fn known_string_pointer_byte_len(&self, value: &str) -> Option<usize> {
        let values = self.known_string_values(value)?;
        let first = values.values().first()?.len();
        values
            .values()
            .iter()
            .all(|value| value.len() == first)
            .then_some(first)
    }

    fn known_string_pointer_byte_len_operand(&self, value: &str) -> Option<String> {
        self.known_string_pointer_byte_len(value)
            .map(|len| len.to_string())
            .or_else(|| self.string_lengths.get(value).cloned())
    }

    fn string_pointer_byte_len_operand(&self, value: &str) -> Option<String> {
        self.known_string_values(value)
            .and_then(|values| known_strings_have_uniform_byte_length(&values))
            .map(|len| len.to_string())
            .or_else(|| self.string_lengths.get(value).cloned())
    }

    fn string_pointer_operand(&mut self, value: IrValue) -> String {
        match value {
            IrValue::String(value) => {
                let name = format!("@{}", self.add_string(&value));
                self.known_strings
                    .insert(name.clone(), KnownString::one(value));
                if let Some(len) = self.known_string_pointer_byte_len(&name) {
                    self.string_lengths.insert(name.clone(), len.to_string());
                }
                name
            }
            IrValue::StringPtr(value) => value,
            _ => unreachable!("string pointer operands are prefiltered"),
        }
    }

    fn add_string(&mut self, value: &str) -> String {
        let name = format!(".str.{}", self.next_string);
        self.next_string += 1;
        self.strings.push((name.clone(), value.to_string()));
        name
    }

    fn next_temp(&mut self) -> String {
        let name = format!("%tmp{}", self.next_temp);
        self.next_temp += 1;
        name
    }

    fn next_label(&mut self, prefix: &str) -> String {
        let name = format!("{prefix}.{}", self.next_label);
        self.next_label += 1;
        name
    }

    fn native_call_diagnostics(&self) -> NativeCallDiagnostics {
        NativeCallDiagnostics::new(NativeCallBackend::Llvm)
    }

    fn unsupported_call_operation(&self, operation: NativeCallOperation) -> Diagnostic {
        self.native_call_diagnostics().operation(operation)
    }

    fn unsupported_direct_call(&self, span: Span, blocker: NativeCallBlocker) -> Diagnostic {
        self.native_call_diagnostics().direct_call(span, blocker)
    }

    fn unsupported_direct_named_call(
        &self,
        args: &[Expr],
        span: Span,
        fallback: &'static str,
    ) -> Diagnostic {
        native_direct_call_argument_result_operation(args, span)
            .map(|operation| self.unsupported_call_operation(operation))
            .unwrap_or_else(|| self.unsupported(span, fallback))
    }

    fn emit_binary_value_operand_exprs(
        &mut self,
        left: &Expr,
        right: &Expr,
    ) -> CompileResult<(IrValue, IrValue)> {
        let left_value = match self.emit_value_operand_expr(left) {
            Ok(value) => value,
            Err(error) => {
                return Err(
                    self.unsupported_unemitted_value_operands_or_original(&[left, right], error)
                );
            }
        };
        let right_value = self.emit_value_operand_expr(right)?;
        Ok((left_value, right_value))
    }

    fn emit_value_operand_expr(&mut self, expr: &Expr) -> CompileResult<IrValue> {
        match self.emit_expr(expr) {
            Ok(value) => Ok(value),
            Err(error) => Err(self.unsupported_value_operand_or_original(expr, error)),
        }
    }

    fn unsupported_value_operand_or_fallback(
        &self,
        expr: &Expr,
        span: Span,
        fallback: &'static str,
    ) -> Diagnostic {
        native_value_operand_call_result_operation(expr)
            .map(|operation| self.unsupported_call_operation(operation))
            .unwrap_or_else(|| self.unsupported(span, fallback))
    }

    fn unsupported_value_operand_or_original(
        &self,
        expr: &Expr,
        original: Diagnostic,
    ) -> Diagnostic {
        native_failed_value_operand_call_result_operation(expr)
            .map(|operation| self.unsupported_call_operation(operation))
            .unwrap_or(original)
    }

    fn unsupported_unemitted_value_operands_or_original(
        &self,
        exprs: &[&Expr],
        original: Diagnostic,
    ) -> Diagnostic {
        native_unemitted_value_operand_list_call_operation(exprs)
            .map(|operation| self.unsupported_call_operation(operation))
            .unwrap_or(original)
    }

    fn unsupported_unemitted_statement_operands_or_original(
        &self,
        exprs: &[Expr],
        original: Diagnostic,
    ) -> Diagnostic {
        native_unemitted_statement_operand_list_call_operation(exprs)
            .map(|operation| self.unsupported_call_operation(operation))
            .unwrap_or(original)
    }

    fn unsupported_value_call(&self, expr: &Expr) -> Diagnostic {
        self.native_call_diagnostics().call_root(expr)
    }

    fn unsupported(&self, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Phase::Codegen, span.line, span.column, message)
    }
}

fn clang_assembly_from_ir(ir: &str) -> CompileResult<String> {
    let mut child = Command::new("clang")
        .args(["-x", "ir", "-S", "-o", "-", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            Diagnostic::new(
                Phase::Codegen,
                0,
                0,
                format!("failed to start clang for assembly emission: {error}"),
            )
        })?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| Diagnostic::new(Phase::Codegen, 0, 0, "failed to open clang stdin"))?;
        stdin.write_all(ir.as_bytes()).map_err(|error| {
            Diagnostic::new(
                Phase::Codegen,
                0,
                0,
                format!("failed to write LLVM IR to clang: {error}"),
            )
        })?;
    }

    let output = child.wait_with_output().map_err(|error| {
        Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            format!("failed to wait for clang: {error}"),
        )
    })?;

    if !output.status.success() {
        return Err(Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            assembly_backend_failure_message("clang", &output.stderr),
        ));
    }

    assembly_backend_success_output("clang", &output)
}

fn llc_assembly_from_ir(ir: &str) -> CompileResult<String> {
    let mut child = Command::new("llc")
        .args(["-o", "-", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            Diagnostic::new(
                Phase::Codegen,
                0,
                0,
                format!("failed to start llc for assembly emission: {error}"),
            )
        })?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| Diagnostic::new(Phase::Codegen, 0, 0, "failed to open llc stdin"))?;
        stdin.write_all(ir.as_bytes()).map_err(|error| {
            Diagnostic::new(
                Phase::Codegen,
                0,
                0,
                format!("failed to write LLVM IR to llc: {error}"),
            )
        })?;
    }

    let output = child.wait_with_output().map_err(|error| {
        Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            format!("failed to wait for llc: {error}"),
        )
    })?;

    if !output.status.success() {
        return Err(Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            assembly_backend_failure_message("llc", &output.stderr),
        ));
    }

    assembly_backend_success_output("llc", &output)
}

fn cc_assembly_from_c(source: &str) -> CompileResult<String> {
    let mut child = Command::new("cc")
        .args(["-x", "c", "-S", "-o", "-", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            Diagnostic::new(
                Phase::Codegen,
                0,
                0,
                format!("failed to start cc for assembly emission: {error}"),
            )
        })?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| Diagnostic::new(Phase::Codegen, 0, 0, "failed to open cc stdin"))?;
        stdin.write_all(source.as_bytes()).map_err(|error| {
            Diagnostic::new(
                Phase::Codegen,
                0,
                0,
                format!("failed to write C fallback source to cc: {error}"),
            )
        })?;
    }

    let output = child.wait_with_output().map_err(|error| {
        Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            format!("failed to wait for cc: {error}"),
        )
    })?;

    if !output.status.success() {
        return Err(Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            assembly_backend_failure_message("cc", &output.stderr),
        ));
    }

    assembly_backend_success_output("cc", &output)
}

fn assembly_backend_failure_message(command: &str, stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    let detail = stderr.trim();
    if detail.is_empty() {
        format!("{command} failed to emit assembly: backend exited without stderr")
    } else {
        format!("{command} failed to emit assembly: {detail}")
    }
}

fn assembly_backend_success_output(
    command: &str,
    output: &std::process::Output,
) -> CompileResult<String> {
    if output.stdout.is_empty() {
        return Err(Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            format!("{command} emitted empty assembly output"),
        ));
    }

    if String::from_utf8_lossy(&output.stdout).trim().is_empty() {
        return Err(Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            format!("{command} emitted whitespace-only assembly output"),
        ));
    }

    // Successful backends may emit warnings or notes to stderr; assembly is
    // taken only from stdout and process stderr is not surfaced by phpc.
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn command_available(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn emit_c_source_for_assembly(program: &Program) -> CompileResult<String> {
    let mut generator = CGenerator::default();
    generator.emit_program(program)
}

pub fn emit_native_executable_c_source(program: &Program) -> CompileResult<String> {
    let mut generator = CGenerator {
        uses_native_string_helpers: true,
        ..CGenerator::default()
    };
    generator.emit_program(program)
}

#[derive(Default)]
struct CGenerator {
    body: Vec<String>,
    static_data: Vec<String>,
    variables: HashMap<String, CValue>,
    array_cleanup_handles: Vec<String>,
    owned_native_byte_buffers: Vec<String>,
    known_ints: HashMap<String, KnownInt>,
    known_floats: HashMap<String, KnownFloat>,
    known_strings: HashMap<String, KnownString>,
    known_string_lengths: HashMap<String, String>,
    known_bools: HashMap<String, KnownBool>,
    uses_strcmp: bool,
    uses_native_string_helpers: bool,
    uses_native_comparison_helpers: bool,
    uses_native_array_comparison_helpers: bool,
    uses_native_array_helpers: bool,
    uses_native_value_clone: bool,
    uses_native_value_string_clone_bytes: bool,
    uses_native_value_offset_mutation: bool,
    uses_native_value_offset_path_write: bool,
    uses_native_value_offset_path_append: bool,
    uses_native_value_offset_path_unset: bool,
    uses_native_array_lvalue_helpers: bool,
    uses_native_request_state_helpers: bool,
    next_static_data: usize,
    next_native_temp: usize,
    native_value_cleanup_handles: Vec<String>,
    native_request_state_handle: Option<String>,
}

#[derive(Debug, Clone)]
enum CValue {
    Int(String),
    Float(String),
    String(String),
    StringExpr(String),
    ArrayHandle(String),
    Bool(bool),
    BoolExpr(String),
    ComparisonDecision(String),
    NativeValueHandle(String),
    Null,
}

#[derive(Debug, Clone)]
enum BackendKnownPrimitive {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone)]
enum BackendPrimitiveSource {
    Null,
    Bool(Vec<bool>),
    IntValues(Vec<i64>),
    IntAny,
    FloatValues(Vec<f64>),
    FloatAny,
    StringValues(Vec<String>),
}

#[derive(Debug, Clone)]
enum BackendArithmeticResult {
    Int(KnownInt),
    Float(KnownFloat),
}

impl BackendKnownPrimitive {
    fn as_php_primitive(&self) -> PhpPrimitiveValue<'_> {
        match self {
            Self::Null => PhpPrimitiveValue::Null,
            Self::Bool(value) => PhpPrimitiveValue::Bool(*value),
            Self::Int(value) => PhpPrimitiveValue::Int(*value),
            Self::Float(value) => PhpPrimitiveValue::Float(*value),
            Self::String(value) => PhpPrimitiveValue::String(value),
        }
    }
}

impl BackendArithmeticResult {
    fn from_values(values: impl IntoIterator<Item = PhpPrimitiveArithmeticValue>) -> Option<Self> {
        let mut ints = Vec::new();
        let mut floats = Vec::new();

        for value in values {
            match value {
                PhpPrimitiveArithmeticValue::Int(value) => ints.push(value),
                PhpPrimitiveArithmeticValue::Float(value) if value.is_finite() => {
                    floats.push(value)
                }
                PhpPrimitiveArithmeticValue::Float(_) => return None,
            }
        }

        match (ints.is_empty(), floats.is_empty()) {
            (false, true) => KnownInt::from_values(ints).map(Self::Int),
            (true, false) => KnownFloat::from_values(floats).map(Self::Float),
            _ => None,
        }
    }

    fn into_single_ir_value(self) -> Option<IrValue> {
        match self {
            Self::Int(values) if values.is_single() => {
                Some(IrValue::Int(values.values()[0].to_string()))
            }
            Self::Float(values) if values.is_single() => {
                Some(IrValue::Float(format_float_literal(values.values()[0])))
            }
            Self::Int(_) | Self::Float(_) => None,
        }
    }

    fn into_single_c_value(self) -> Option<CValue> {
        match self {
            Self::Int(values) if values.is_single() => {
                Some(CValue::Int(values.values()[0].to_string()))
            }
            Self::Float(values) if values.is_single() => {
                Some(CValue::Float(format_float_literal(values.values()[0])))
            }
            Self::Int(_) | Self::Float(_) => None,
        }
    }
}

impl BackendPrimitiveSource {
    fn null() -> Self {
        Self::Null
    }

    fn bool_value(value: bool) -> Self {
        Self::Bool(vec![value])
    }

    fn unknown_bool() -> Self {
        Self::Bool(vec![false, true])
    }

    fn bool_values(values: KnownBool) -> Self {
        Self::Bool(values.values().to_vec())
    }

    fn int_values(values: KnownInt) -> Self {
        Self::IntValues(values.values().to_vec())
    }

    fn unknown_int() -> Self {
        Self::IntAny
    }

    fn float_values(values: KnownFloat) -> Self {
        Self::FloatValues(values.values().to_vec())
    }

    fn unknown_float() -> Self {
        Self::FloatAny
    }

    fn string_value(value: &str) -> Self {
        Self::StringValues(vec![value.to_string()])
    }

    fn string_values(values: KnownString) -> Self {
        Self::StringValues(values.values().to_vec())
    }

    fn known_primitives(&self) -> Option<Vec<BackendKnownPrimitive>> {
        match self {
            Self::Null => Some(vec![BackendKnownPrimitive::Null]),
            Self::Bool(values) => Some(
                values
                    .iter()
                    .map(|value| BackendKnownPrimitive::Bool(*value))
                    .collect(),
            ),
            Self::IntValues(values) => Some(
                values
                    .iter()
                    .map(|value| BackendKnownPrimitive::Int(*value))
                    .collect(),
            ),
            Self::IntAny => None,
            Self::FloatValues(values) if values.iter().all(|value| value.is_finite()) => Some(
                values
                    .iter()
                    .map(|value| BackendKnownPrimitive::Float(*value))
                    .collect(),
            ),
            Self::FloatValues(_) | Self::FloatAny => None,
            Self::StringValues(values) => Some(
                values
                    .iter()
                    .map(|value| BackendKnownPrimitive::String(value.clone()))
                    .collect(),
            ),
        }
    }

    fn single_arithmetic_result(
        &self,
        operation: PhpPrimitiveArithmeticOperation,
    ) -> Result<Option<BackendArithmeticResult>, PhpPrimitiveArithmeticError> {
        let Some(values) = self.known_primitives() else {
            return Ok(None);
        };
        let mut results = Vec::new();
        for value in &values {
            results.push(php_primitive_arithmetic_result(
                value.as_php_primitive(),
                operation,
                None,
            )?);
        }
        Ok(BackendArithmeticResult::from_values(results))
    }

    fn pair_arithmetic_result(
        &self,
        operation: PhpPrimitiveArithmeticOperation,
        right: &Self,
    ) -> Result<Option<BackendArithmeticResult>, PhpPrimitiveArithmeticError> {
        let Some(left_values) = self.known_primitives() else {
            return Ok(None);
        };
        let Some(right_values) = right.known_primitives() else {
            return Ok(None);
        };
        let mut results = Vec::new();
        for left in &left_values {
            for right in &right_values {
                results.push(php_primitive_arithmetic_result(
                    left.as_php_primitive(),
                    operation,
                    Some(right.as_php_primitive()),
                )?);
            }
        }
        Ok(BackendArithmeticResult::from_values(results))
    }

    fn from_arithmetic_result(result: BackendArithmeticResult) -> Self {
        match result {
            BackendArithmeticResult::Int(values) => Self::int_values(values),
            BackendArithmeticResult::Float(values) => Self::float_values(values),
        }
    }

    fn single_c_value(&self) -> Option<CValue> {
        match self {
            Self::Null => Some(CValue::Null),
            Self::Bool(values) if values.len() == 1 => Some(CValue::Bool(values[0])),
            Self::IntValues(values) if values.len() == 1 => {
                Some(CValue::Int(values[0].to_string()))
            }
            Self::FloatValues(values) if values.len() == 1 && values[0].is_finite() => {
                Some(CValue::Float(format_float_literal(values[0])))
            }
            Self::StringValues(values) if values.len() == 1 => {
                Some(CValue::String(values[0].clone()))
            }
            Self::Bool(_)
            | Self::IntValues(_)
            | Self::IntAny
            | Self::FloatValues(_)
            | Self::FloatAny
            | Self::StringValues(_) => None,
        }
    }
}

fn backend_binary_primitive_arithmetic_operation(
    op: BinaryOp,
) -> Option<PhpPrimitiveArithmeticOperation> {
    match op {
        BinaryOp::Add => Some(PhpPrimitiveArithmeticOperation::Add),
        BinaryOp::Sub => Some(PhpPrimitiveArithmeticOperation::Subtract),
        BinaryOp::Mul => Some(PhpPrimitiveArithmeticOperation::Multiply),
        BinaryOp::Div
        | BinaryOp::Mod
        | BinaryOp::Concat
        | BinaryOp::Eq
        | BinaryOp::Ne
        | BinaryOp::Lt
        | BinaryOp::Le
        | BinaryOp::Gt
        | BinaryOp::Ge
        | BinaryOp::StrictEq
        | BinaryOp::StrictNe
        | BinaryOp::NullCoalesce
        | BinaryOp::LogicalAnd
        | BinaryOp::LogicalOr
        | BinaryOp::LogicalXor
        | BinaryOp::BitwiseAnd
        | BinaryOp::BitwiseOr
        | BinaryOp::BitwiseXor
        | BinaryOp::ShiftLeft
        | BinaryOp::ShiftRight => None,
    }
}

struct NativeCComparisonOperand {
    operand: String,
}

struct CNativeArrayKeyMaterialization {
    result: String,
    cleanup_after_use: Vec<String>,
}

struct CNativeArrayLvaluePath {
    path: String,
    len: usize,
    cleanup_after_use: Vec<String>,
}

struct CNativeValueMaterialization {
    handle: String,
    cleanup_after_use: Vec<String>,
}

fn c_cleanup_sequence(cleanup: &[String]) -> String {
    if cleanup.is_empty() {
        String::new()
    } else {
        let mut sequence = cleanup.join(" ");
        sequence.push(' ');
        sequence
    }
}

impl CGenerator {
    fn next_native_name(&mut self, prefix: &str) -> String {
        let index = self.next_native_temp;
        self.next_native_temp += 1;
        format!("{prefix}_{index}")
    }

    fn uses_native_runtime_helpers(&self) -> bool {
        self.uses_native_string_helpers
            || self.uses_native_comparison_helpers
            || self.uses_native_array_helpers
            || self.uses_native_request_state_helpers
    }

    fn emit_program(&mut self, program: &Program) -> CompileResult<String> {
        for stmt in &program.statements {
            self.emit_statement(stmt)?;
        }

        let mut output = String::new();
        if self.uses_native_runtime_helpers() {
            output.push_str("/* generated by phpc native executable C link path */\n");
        } else {
            output.push_str("/* generated by phpc milestone 1 C assembly fallback */\n");
        }
        output.push_str("#include <stdio.h>\n\n");
        if self.uses_native_runtime_helpers() {
            output.push_str("#include <stddef.h>\n");
            output.push_str("#include <stdint.h>\n");
            if self.uses_native_comparison_helpers || self.uses_native_array_helpers {
                output.push_str("#include <stdbool.h>\n");
            }
            output.push('\n');
            if self.uses_native_string_helpers
                || self.uses_native_comparison_helpers
                || self.uses_native_array_helpers
            {
                output.push_str(
                    "typedef struct { uint8_t tag; uint8_t bool_value; int64_t int_value; double float_value; } phpc_NativeScalarValue;\n",
                );
            }
            output.push_str("typedef struct { void *ptr; } phpc_NativeStringHandle;\n");
            output.push_str("typedef struct { void *ptr; } phpc_NativeValueHandle;\n");
            output.push_str("typedef struct { void *ptr; } phpc_NativeDiagnosticHandle;\n");
            if self.uses_native_request_state_helpers {
                output.push_str("typedef struct { void *ptr; } phpc_NativeRequestStateHandle;\n");
            }
            if self.uses_native_string_helpers || self.uses_native_array_helpers {
                output.push_str(
                    "typedef struct { uint8_t *ptr; size_t len; size_t cap; } phpc_NativeByteBuffer;\n",
                );
            }
            if self.uses_native_string_helpers {
                output.push_str("typedef struct { phpc_NativeByteBuffer bytes; phpc_NativeDiagnosticHandle diagnostic; } phpc_NativeStringConversionResult;\n");
            }
            if self.uses_native_comparison_helpers {
                output.push_str("typedef struct { uint8_t opcode; uint8_t valid; } phpc_NativeComparisonOperation;\n");
                output.push_str("typedef struct { phpc_NativeValueHandle value; phpc_NativeDiagnosticHandle diagnostic; } phpc_NativeComparisonOperand;\n");
                output.push_str("typedef struct { uint8_t status; uint8_t relation; uint8_t family; phpc_NativeDiagnosticHandle diagnostic; } phpc_NativeComparisonRelationResult;\n");
                if self.uses_native_array_comparison_helpers {
                    output.push_str("typedef struct { uint8_t status; uint8_t value; size_t diagnostic_len; } phpc_NativeComparisonBranchResult;\n");
                }
                output.push_str("typedef struct { int exit_code; uint8_t value; } phpc_NativeComparisonBranchDecision;\n");
                output.push_str("#define PHPC_NATIVE_COMPARISON_STATUS_OK 0\n");
                output.push_str("#define PHPC_NATIVE_COMPARISON_STATUS_BLOCKED 1\n");
            }
            if self.uses_native_array_helpers {
                output.push_str("typedef struct { void *ptr; } phpc_NativeArrayHandle;\n");
                output.push_str("typedef struct { uint8_t tag; int64_t int_value; phpc_NativeByteBuffer bytes; phpc_NativeDiagnosticHandle diagnostic; } phpc_NativeArrayKeyMaterializationResult;\n");
                output.push_str("typedef struct { uint8_t tag; phpc_NativeValueHandle value; phpc_NativeDiagnosticHandle diagnostic; } phpc_NativeValueOperationResult;\n");
                if self.uses_native_array_lvalue_helpers {
                    output.push_str("typedef struct { void *ptr; } phpc_NativeReferenceHandle;\n");
                    output.push_str("typedef struct { uint8_t tag; phpc_NativeValueHandle key; } phpc_NativeArrayPathSegment;\n");
                    output.push_str("typedef struct { uint8_t tag; phpc_NativeArrayHandle array; phpc_NativeValueHandle value; phpc_NativeValueHandle *value_slot; phpc_NativeReferenceHandle reference; } phpc_NativeArrayLvalueOwner;\n");
                    output.push_str("typedef struct { uint8_t tag; phpc_NativeValueHandle value; phpc_NativeDiagnosticHandle diagnostic; } phpc_NativeArrayLvalueResult;\n");
                    output.push_str("#define PHPC_NATIVE_ARRAY_LVALUE_OK 0\n");
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_PATH_KEY {NATIVE_ARRAY_PATH_KEY_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_PATH_APPEND {NATIVE_ARRAY_PATH_APPEND_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE {NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UNSET {NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UNSET_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ {NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UPDATE {NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UPDATE_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_ISSET {NATIVE_ARRAY_LVALUE_VALUE_OPERATION_ISSET_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_VALUE_RESULT_INCREMENT_DECREMENT {NATIVE_ARRAY_LVALUE_VALUE_RESULT_INCREMENT_DECREMENT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_INCREMENT {NATIVE_ARRAY_LVALUE_INCREMENT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_DECREMENT {NATIVE_ARRAY_LVALUE_DECREMENT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_POSITION_PRE {NATIVE_ARRAY_LVALUE_POSITION_PRE_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_POSITION_POST {NATIVE_ARRAY_LVALUE_POSITION_POST_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_POINTER_CURRENT {NATIVE_ARRAY_LVALUE_POINTER_CURRENT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_POINTER_KEY {NATIVE_ARRAY_LVALUE_POINTER_KEY_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_POINTER_NEXT {NATIVE_ARRAY_LVALUE_POINTER_NEXT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_POINTER_PREV {NATIVE_ARRAY_LVALUE_POINTER_PREV_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_POINTER_RESET {NATIVE_ARRAY_LVALUE_POINTER_RESET_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_POINTER_END {NATIVE_ARRAY_LVALUE_POINTER_END_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_SORT_SORT {NATIVE_ARRAY_LVALUE_SORT_SORT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_SORT_RSORT {NATIVE_ARRAY_LVALUE_SORT_RSORT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_SORT_ASORT {NATIVE_ARRAY_LVALUE_SORT_ASORT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_SORT_ARSORT {NATIVE_ARRAY_LVALUE_SORT_ARSORT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_SORT_KSORT {NATIVE_ARRAY_LVALUE_SORT_KSORT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_SORT_KRSORT {NATIVE_ARRAY_LVALUE_SORT_KRSORT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_SORT_NATSORT {NATIVE_ARRAY_LVALUE_SORT_NATSORT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_SORT_NATCASESORT {NATIVE_ARRAY_LVALUE_SORT_NATCASESORT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_SORT_USORT {NATIVE_ARRAY_LVALUE_SORT_USORT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_SORT_UASORT {NATIVE_ARRAY_LVALUE_SORT_UASORT_TAG}\n"
                    ));
                    output.push_str(&format!(
                        "#define PHPC_NATIVE_ARRAY_LVALUE_SORT_UKSORT {NATIVE_ARRAY_LVALUE_SORT_UKSORT_TAG}\n"
                    ));
                    output.push_str("#define PHPC_NATIVE_VALUE_ARRAY_CALLBACK_FILTER 0\n");
                    output.push_str("#define PHPC_NATIVE_VALUE_ARRAY_CALLBACK_MAP 1\n");
                    output.push_str("#define PHPC_NATIVE_VALUE_ARRAY_CALLBACK_REDUCE 2\n");
                }
                output.push_str("#define PHPC_NATIVE_VALUE_OPERATION_OK 0\n");
                output.push_str("#define PHPC_NATIVE_VALUE_UNARY_NEGATE 0\n");
                output.push_str("#define PHPC_NATIVE_VALUE_UNARY_BITWISE_NOT 1\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BINARY_ADD 0\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BINARY_SUB 1\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BINARY_MUL 2\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BINARY_DIV 3\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BINARY_MOD 4\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BINARY_CONCAT 5\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BINARY_BITWISE_AND 6\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BINARY_BITWISE_OR 7\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BINARY_BITWISE_XOR 8\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BINARY_SHIFT_LEFT 9\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BINARY_SHIFT_RIGHT 10\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BITWISE_AND 0\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BITWISE_OR 1\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BITWISE_XOR 2\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BITWISE_NOT 3\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BITWISE_SHIFT_LEFT 4\n");
                output.push_str("#define PHPC_NATIVE_VALUE_BITWISE_SHIFT_RIGHT 5\n");
                output.push_str("#define PHPC_NATIVE_VALUE_COMPARISON_EQ 0\n");
                output.push_str("#define PHPC_NATIVE_VALUE_COMPARISON_NE 1\n");
                output.push_str("#define PHPC_NATIVE_VALUE_COMPARISON_LT 2\n");
                output.push_str("#define PHPC_NATIVE_VALUE_COMPARISON_LE 3\n");
                output.push_str("#define PHPC_NATIVE_VALUE_COMPARISON_GT 4\n");
                output.push_str("#define PHPC_NATIVE_VALUE_COMPARISON_GE 5\n");
                output.push_str("#define PHPC_NATIVE_VALUE_CAST_STRING 0\n");
                output.push_str("#define PHPC_NATIVE_VALUE_CAST_INT 1\n");
                output.push_str("#define PHPC_NATIVE_VALUE_CAST_BOOL 2\n");
                output.push_str("#define PHPC_NATIVE_VALUE_CAST_FLOAT 3\n");
                output.push_str("#define PHPC_NATIVE_VALUE_CAST_ARRAY 4\n");
                output.push_str("#define PHPC_NATIVE_VALUE_TYPE_IS_NULL 0\n");
                output.push_str("#define PHPC_NATIVE_VALUE_TYPE_IS_BOOL 1\n");
                output.push_str("#define PHPC_NATIVE_VALUE_TYPE_IS_INT 2\n");
                output.push_str("#define PHPC_NATIVE_VALUE_TYPE_IS_FLOAT 3\n");
                output.push_str("#define PHPC_NATIVE_VALUE_TYPE_IS_STRING 4\n");
                output.push_str("#define PHPC_NATIVE_VALUE_TYPE_IS_ARRAY 5\n");
                output.push_str("#define PHPC_NATIVE_VALUE_TYPE_IS_SCALAR 6\n");
                output.push_str("#define PHPC_NATIVE_VALUE_TYPE_IS_NUMERIC 7\n");
                output.push_str("#define PHPC_NATIVE_VALUE_TYPE_IS_COUNTABLE 8\n");
                output.push_str("#define PHPC_NATIVE_VALUE_TYPE_IS_ITERABLE 9\n");
                output.push_str("#define PHPC_NATIVE_VALUE_TYPE_IS_OBJECT 10\n");
                output.push_str("#define PHPC_NATIVE_VALUE_TYPE_NAME_GETTYPE 0\n");
                output.push_str("#define PHPC_NATIVE_VALUE_TYPE_NAME_DEBUG 1\n");
            }
            output.push('\n');
            output.push_str("extern phpc_NativeStringHandle phpc_native_string_from_bytes(const uint8_t *ptr, size_t len);\n");
            output.push_str("extern phpc_NativeValueHandle phpc_native_value_from_scalar(phpc_NativeScalarValue value);\n");
            output.push_str("extern phpc_NativeValueHandle phpc_native_value_from_string_with_diagnostic(phpc_NativeStringHandle string, phpc_NativeDiagnosticHandle *diagnostic);\n");
            output.push_str("extern phpc_NativeValueHandle phpc_native_value_from_string_bytes_with_diagnostic(const uint8_t *ptr, size_t len, phpc_NativeDiagnosticHandle *diagnostic);\n");
            if self.uses_native_request_state_helpers {
                output.push_str(
                    "extern phpc_NativeRequestStateHandle phpc_native_request_state_empty(void);\n",
                );
                output.push_str("extern phpc_NativeValueHandle phpc_native_request_state_superglobal_snapshot_value(phpc_NativeRequestStateHandle request_state, phpc_NativeStringHandle bag);\n");
                output.push_str(
                    "extern void phpc_native_request_state_free(phpc_NativeRequestStateHandle request_state);\n",
                );
            }
            if self.uses_native_string_helpers {
                output.push_str("extern phpc_NativeStringConversionResult phpc_native_value_to_string_bytes(phpc_NativeValueHandle value);\n");
                output.push_str("extern int64_t phpc_native_value_to_int64_with_diagnostic(phpc_NativeValueHandle value, uint8_t operation, phpc_NativeDiagnosticHandle *diagnostic);\n");
                output.push_str("extern _Bool phpc_native_value_string_predicate_with_diagnostic(phpc_NativeValueHandle haystack, phpc_NativeValueHandle needle, uint8_t predicate, phpc_NativeDiagnosticHandle *diagnostic);\n");
                output.push_str("extern int64_t phpc_native_value_string_int_operation_with_diagnostic(phpc_NativeValueHandle subject, phpc_NativeValueHandle operand, int64_t offset, int64_t length, uint8_t flags, uint8_t operation, phpc_NativeDiagnosticHandle *diagnostic);\n");
                output.push_str("extern int64_t phpc_native_value_string_distance_operation_with_diagnostic(phpc_NativeValueHandle subject, phpc_NativeValueHandle operand, int64_t insertion_cost, int64_t replacement_cost, int64_t deletion_cost, uint8_t operation, phpc_NativeDiagnosticHandle *diagnostic);\n");
                output.push_str("extern phpc_NativeValueHandle phpc_native_value_string_result_operation_with_diagnostic(phpc_NativeValueHandle subject, phpc_NativeValueHandle operand, phpc_NativeValueHandle replacement, int64_t offset, int64_t length, uint8_t flags, uint8_t operation, phpc_NativeDiagnosticHandle *diagnostic);\n");
                output.push_str("extern phpc_NativeValueHandle phpc_native_value_offset_operation_with_diagnostic(phpc_NativeValueHandle subject, phpc_NativeValueHandle offset, uint8_t operation, phpc_NativeDiagnosticHandle *diagnostic);\n");
                output.push_str("extern phpc_NativeValueHandle phpc_native_value_string_offset_operation_with_diagnostic(phpc_NativeValueHandle subject, phpc_NativeValueHandle offset, uint8_t operation, phpc_NativeDiagnosticHandle *diagnostic);\n");
                output.push_str("extern _Bool phpc_native_value_bool_with_diagnostic(phpc_NativeValueHandle value, phpc_NativeDiagnosticHandle *diagnostic);\n");
                if self.uses_native_value_offset_mutation {
                    output.push_str("extern phpc_NativeValueHandle phpc_native_value_offset_mutation_operation_with_diagnostic(phpc_NativeValueHandle subject, phpc_NativeValueHandle offset, phpc_NativeValueHandle replacement, uint8_t operation, phpc_NativeDiagnosticHandle *diagnostic);\n");
                    if self.uses_native_value_offset_path_write {
                        output.push_str("extern phpc_NativeValueHandle phpc_native_value_offset_path_write_with_diagnostic(phpc_NativeValueHandle subject, const phpc_NativeValueHandle *offsets, size_t offsets_len, phpc_NativeValueHandle replacement, phpc_NativeDiagnosticHandle *diagnostic);\n");
                    }
                    if self.uses_native_value_offset_path_append {
                        output.push_str("extern phpc_NativeValueHandle phpc_native_value_offset_path_append_with_diagnostic(phpc_NativeValueHandle subject, const phpc_NativeValueHandle *prefix_offsets, size_t prefix_offsets_len, const phpc_NativeValueHandle *suffix_offsets, size_t suffix_offsets_len, phpc_NativeValueHandle replacement, phpc_NativeDiagnosticHandle *diagnostic);\n");
                    }
                    if self.uses_native_value_offset_path_unset {
                        output.push_str("extern phpc_NativeValueHandle phpc_native_value_offset_path_unset_with_diagnostic(phpc_NativeValueHandle subject, const phpc_NativeValueHandle *offsets, size_t offsets_len, phpc_NativeDiagnosticHandle *diagnostic);\n");
                    }
                }
                if self.uses_native_value_string_clone_bytes {
                    output.push_str("extern phpc_NativeByteBuffer phpc_native_value_string_clone_bytes(phpc_NativeValueHandle value);\n");
                    output.push_str(
                        "extern void phpc_native_byte_buffer_free(phpc_NativeByteBuffer buffer);\n",
                    );
                }
                output.push_str("extern phpc_NativeValueHandle phpc_native_value_filesystem_path_operation_with_diagnostic(phpc_NativeValueHandle path, phpc_NativeValueHandle option, int64_t offset, int64_t length, uint8_t flags, uint8_t operation, phpc_NativeDiagnosticHandle *diagnostic);\n");
                output.push_str("extern void phpc_native_string_conversion_result_free(phpc_NativeStringConversionResult result);\n");
            }
            output.push_str(
                "extern size_t phpc_native_value_echo_stdout(phpc_NativeValueHandle value);\n",
            );
            if self.uses_native_value_clone {
                output.push_str(
                    "extern phpc_NativeValueHandle phpc_native_value_clone(phpc_NativeValueHandle value);\n",
                );
            }
            output.push_str("extern void phpc_native_value_free(phpc_NativeValueHandle value);\n");
            output.push_str("extern size_t phpc_native_diagnostic_message_stderr(phpc_NativeDiagnosticHandle diagnostic);\n");
            output.push_str("extern size_t phpc_native_diagnostic_report(phpc_NativeDiagnosticHandle diagnostic);\n");
            output.push_str("extern void phpc_native_diagnostic_free(phpc_NativeDiagnosticHandle diagnostic);\n");
            output.push_str(
                "extern void phpc_native_string_free(phpc_NativeStringHandle string);\n\n",
            );
            if self.uses_native_array_helpers {
                output.push_str("extern phpc_NativeArrayHandle phpc_native_array_empty(void);\n");
                output.push_str("extern phpc_NativeValueHandle phpc_native_value_from_array(phpc_NativeArrayHandle array);\n");
                output.push_str("extern phpc_NativeArrayHandle phpc_native_value_array_clone(phpc_NativeValueHandle value);\n");
                output.push_str("extern bool phpc_native_array_append_value(phpc_NativeArrayHandle array, phpc_NativeValueHandle value);\n");
                output.push_str("extern bool phpc_native_array_append_value_with_diagnostic(phpc_NativeArrayHandle array, phpc_NativeValueHandle value, phpc_NativeDiagnosticHandle *diagnostic);\n");
                output.push_str("extern phpc_NativeArrayKeyMaterializationResult phpc_native_value_to_array_key(phpc_NativeValueHandle value);\n");
                output.push_str("extern phpc_NativeValueOperationResult phpc_native_value_unary_result(phpc_NativeValueHandle value, uint8_t op);\n");
                output.push_str("extern phpc_NativeValueOperationResult phpc_native_value_binary_result(phpc_NativeValueHandle left, uint8_t op, phpc_NativeValueHandle right);\n");
                output.push_str("extern phpc_NativeValueOperationResult phpc_native_value_compare_result(phpc_NativeValueHandle left, uint8_t op, phpc_NativeValueHandle right);\n");
                output.push_str("extern phpc_NativeValueHandle phpc_native_value_bitwise_operation_with_diagnostic(phpc_NativeValueHandle subject, phpc_NativeValueHandle operand, uint8_t operation, phpc_NativeDiagnosticHandle *diagnostic);\n");
                output.push_str("extern phpc_NativeValueHandle phpc_native_value_cast_operation_with_diagnostic(phpc_NativeValueHandle value, uint8_t operation, phpc_NativeDiagnosticHandle *diagnostic);\n");
                output.push_str("extern phpc_NativeValueOperationResult phpc_native_value_type_name_result(phpc_NativeValueHandle value, uint8_t kind);\n");
                output.push_str("extern bool phpc_native_value_type_predicate(phpc_NativeValueHandle value, uint8_t predicate);\n");
                output.push_str("extern bool phpc_native_array_insert_key_value_with_diagnostic(phpc_NativeArrayHandle array, phpc_NativeArrayKeyMaterializationResult key, phpc_NativeValueHandle value, phpc_NativeDiagnosticHandle *diagnostic);\n");
                output.push_str("extern phpc_NativeValueHandle phpc_native_array_read_key_with_diagnostic(phpc_NativeArrayHandle array, phpc_NativeArrayKeyMaterializationResult key, phpc_NativeDiagnosticHandle *diagnostic);\n");
                output.push_str("extern void phpc_native_array_key_materialization_result_free(phpc_NativeArrayKeyMaterializationResult key);\n");
                output.push_str("extern void phpc_native_value_operation_result_free(phpc_NativeValueOperationResult result);\n");
                if self.uses_native_array_lvalue_helpers {
                    output.push_str("extern phpc_NativeArrayLvalueOwner phpc_native_array_lvalue_owner_array(phpc_NativeArrayHandle array);\n");
                    output.push_str("extern phpc_NativeArrayLvalueResult phpc_native_array_lvalue_owner_value_operation_result(phpc_NativeArrayLvalueOwner owner, const phpc_NativeArrayPathSegment *segments, size_t segment_count, uint8_t family, uint8_t operation, uint8_t op, uint8_t position, phpc_NativeValueHandle value);\n");
                    output.push_str("extern phpc_NativeArrayLvalueResult phpc_native_array_lvalue_owner_pointer_result(phpc_NativeArrayLvalueOwner owner, const phpc_NativeArrayPathSegment *segments, size_t segment_count, uint8_t operation);\n");
                    output.push_str("extern phpc_NativeArrayLvalueResult phpc_native_array_lvalue_owner_sort_result(phpc_NativeArrayLvalueOwner owner, const phpc_NativeArrayPathSegment *segments, size_t segment_count, uint8_t operation, const phpc_NativeValueHandle *operands, size_t operand_count);\n");
                    output.push_str("extern phpc_NativeArrayLvalueResult phpc_native_value_array_callback_result(uint8_t operation, const phpc_NativeValueHandle *values, size_t value_count);\n");
                    output.push_str("extern phpc_NativeArrayLvalueResult phpc_native_array_lvalue_owner_foreach_iterable_result(phpc_NativeArrayLvalueOwner owner, const phpc_NativeArrayPathSegment *segments, size_t segment_count);\n");
                    output.push_str("extern size_t phpc_native_array_foreach_iterable_len(phpc_NativeValueHandle iterable);\n");
                    output.push_str("extern phpc_NativeArrayLvalueResult phpc_native_array_foreach_iterable_key_result(phpc_NativeValueHandle iterable, size_t index);\n");
                    output.push_str("extern phpc_NativeArrayLvalueResult phpc_native_array_foreach_iterable_value_result(phpc_NativeValueHandle iterable, size_t index);\n");
                    output.push_str("extern void phpc_native_array_lvalue_result_free(phpc_NativeArrayLvalueResult result);\n");
                }
                output.push_str(
                    "extern void phpc_native_array_free(phpc_NativeArrayHandle array);\n\n",
                );
            }
        }
        if self.uses_native_comparison_helpers {
            output.push_str("extern phpc_NativeComparisonOperation phpc_native_comparison_operation_from_opcode(uint8_t opcode);\n");
            output.push_str("extern phpc_NativeComparisonOperand phpc_native_comparison_operand_from_scalar(phpc_NativeScalarValue value);\n");
            output.push_str("extern phpc_NativeComparisonOperand phpc_native_comparison_operand_from_string_and_free(phpc_NativeStringHandle string);\n");
            output.push_str("extern phpc_NativeComparisonRelationResult phpc_native_comparison_operand_compare_operation_relation_and_free(phpc_NativeComparisonOperand left, phpc_NativeComparisonOperation operation, phpc_NativeComparisonOperand right);\n");
            output.push_str("extern phpc_NativeComparisonBranchDecision phpc_native_comparison_relation_result_decision_or_report_stderr_and_free(phpc_NativeComparisonRelationResult result, phpc_NativeComparisonOperation operation);\n");
            output.push_str("extern phpc_NativeComparisonOperand phpc_native_comparison_branch_decision_result_operand(phpc_NativeComparisonBranchDecision decision);\n");
            if self.uses_native_array_comparison_helpers {
                output.push_str("extern phpc_NativeComparisonBranchResult phpc_native_array_compare_branch(phpc_NativeArrayHandle left, uint8_t op, phpc_NativeArrayHandle right);\n");
                output.push_str("extern phpc_NativeComparisonBranchDecision phpc_native_comparison_branch_decision_from_result(phpc_NativeComparisonBranchResult result);\n");
            }
            output.push_str("extern int phpc_native_comparison_branch_decision_abort_code(phpc_NativeComparisonBranchDecision decision);\n");
            output.push_str("extern bool phpc_native_comparison_branch_decision_is_true(phpc_NativeComparisonBranchDecision decision);\n\n");
        }
        if self.uses_strcmp {
            output.push_str("#include <string.h>\n\n");
        }
        for line in &self.static_data {
            output.push_str(line);
            output.push('\n');
        }
        if !self.static_data.is_empty() {
            output.push('\n');
        }
        output.push_str("int main(void) {\n");
        for line in &self.body {
            output.push_str("  ");
            output.push_str(line);
            output.push('\n');
        }
        for handle in self.native_value_cleanup_handles.iter().rev() {
            output.push_str("  ");
            output.push_str(&format!("phpc_native_value_free({handle});"));
            output.push('\n');
        }
        for buffer in self.owned_native_byte_buffers.iter().rev() {
            output.push_str("  ");
            output.push_str(&format!("phpc_native_byte_buffer_free({buffer});"));
            output.push('\n');
        }
        for handle in self.array_cleanup_handles.iter().rev() {
            output.push_str("  ");
            output.push_str(&format!("phpc_native_array_free({handle});"));
            output.push('\n');
        }
        if let Some(handle) = &self.native_request_state_handle {
            output.push_str("  ");
            output.push_str(&format!("phpc_native_request_state_free({handle});"));
            output.push('\n');
        }
        output.push_str("  return 0;\n");
        output.push_str("}\n");
        Ok(output)
    }

    fn retain_native_value_cleanup_handle(&mut self, handle: &str) {
        self.native_value_cleanup_handles.push(handle.to_string());
    }

    fn release_native_value_cleanup_handle(&mut self, handle: &str) {
        if let Some(index) = self
            .native_value_cleanup_handles
            .iter()
            .rposition(|owned| owned == handle)
        {
            self.native_value_cleanup_handles.remove(index);
        }
    }

    fn release_variable_native_value_handle(&mut self, name: &str) {
        let handle = self.variables.get(name).and_then(|value| match value {
            CValue::NativeValueHandle(handle) => Some(handle.clone()),
            _ => None,
        });
        if let Some(handle) = handle {
            self.release_native_value_cleanup_handle(&handle);
            self.body.push(format!("phpc_native_value_free({handle});"));
        }
    }

    fn clone_native_value_handle(&mut self, handle: &str) -> String {
        self.uses_native_string_helpers = true;
        self.uses_native_value_clone = true;
        let cloned = self.next_native_name("native_value_clone");
        self.body.push(format!(
            "phpc_NativeValueHandle {cloned} = phpc_native_value_clone({handle});"
        ));
        cloned
    }

    fn value_for_variable_storage(&mut self, value: CValue) -> CValue {
        match value {
            CValue::NativeValueHandle(handle) => {
                let cloned = self.clone_native_value_handle(&handle);
                self.retain_native_value_cleanup_handle(&cloned);
                CValue::NativeValueHandle(cloned)
            }
            value => value,
        }
    }

    fn store_variable_value(&mut self, name: &str, value: CValue) {
        let stored = self.value_for_variable_storage(value);
        self.release_variable_native_value_handle(name);
        self.variables.insert(name.to_string(), stored);
    }

    fn store_native_value_result_variable(
        &mut self,
        name: &str,
        value: CNativeValueMaterialization,
    ) {
        self.release_variable_native_value_handle(name);
        self.retain_native_value_cleanup_handle(&value.handle);
        self.variables
            .insert(name.to_string(), CValue::NativeValueHandle(value.handle));
    }

    fn ensure_native_request_state_handle(&mut self) -> String {
        self.uses_native_request_state_helpers = true;
        if let Some(handle) = &self.native_request_state_handle {
            return handle.clone();
        }

        let handle = self.next_native_name("request_state");
        self.body.push(format!(
            "phpc_NativeRequestStateHandle {handle} = phpc_native_request_state_empty();"
        ));
        self.native_request_state_handle = Some(handle.clone());
        handle
    }

    fn emit_request_superglobal_bag_handle(&mut self, name: &str) -> String {
        self.uses_native_string_helpers = true;
        self.uses_native_request_state_helpers = true;

        let index = self.next_static_data;
        self.next_static_data += 1;
        let bytes = c_byte_array(name.as_bytes());
        self.static_data.push(format!(
            "static const uint8_t request_superglobal_bag_bytes_{index}[] = {{{bytes}}};"
        ));

        let bag = self.next_native_name("request_superglobal_bag");
        self.body.push(format!(
            "phpc_NativeStringHandle {bag} = phpc_native_string_from_bytes(request_superglobal_bag_bytes_{index}, {});",
            name.len()
        ));
        bag
    }

    fn materialize_request_superglobal_snapshot_value(
        &mut self,
        name: &str,
        failure_cleanup: &str,
    ) -> CNativeValueMaterialization {
        let request_state = self.ensure_native_request_state_handle();
        let bag = self.emit_request_superglobal_bag_handle(name);
        let value = self.next_native_name("request_superglobal_snapshot");
        self.body.push(format!(
            "phpc_NativeValueHandle {value} = phpc_native_request_state_superglobal_snapshot_value({request_state}, {bag});"
        ));
        let error_exit = self.native_error_exit(&format!(
            "phpc_native_string_free({bag}); {failure_cleanup}"
        ));
        self.body
            .push(format!("if ({value}.ptr == NULL) {{ {error_exit} }}"));
        self.body.push(format!("phpc_native_string_free({bag});"));

        CNativeValueMaterialization {
            handle: value.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({value});")],
        }
    }

    fn emit_request_superglobal_isset_expr(&mut self, name: &str, failure_cleanup: &str) -> CValue {
        let value = self.materialize_request_superglobal_snapshot_value(name, failure_cleanup);
        let result = self.next_native_name("request_superglobal_isset");
        let handle = value.handle.clone();
        self.body
            .push(format!("_Bool {result} = {handle}.ptr != NULL;"));
        self.body.extend(value.cleanup_after_use);
        CValue::BoolExpr(result)
    }

    fn emit_request_superglobal_empty_expr(&mut self, name: &str, failure_cleanup: &str) -> CValue {
        let value = self.materialize_request_superglobal_snapshot_value(name, failure_cleanup);
        let bool_value = self.emit_native_value_cast_operation_result_handle(
            value,
            "PHPC_NATIVE_VALUE_CAST_BOOL",
            failure_cleanup,
        );
        let bool_handle = bool_value.handle.clone();
        let diagnostic = self.next_native_name("request_superglobal_empty_diagnostic");
        let truthy = self.next_native_name("request_superglobal_truthy");
        let result = self.next_native_name("request_superglobal_empty");
        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "_Bool {truthy} = phpc_native_value_bool_with_diagnostic({bool_handle}, &{diagnostic});"
        ));
        let bool_error_exit = self.native_error_exit(&format!(
            "phpc_native_diagnostic_report({diagnostic}); {}{failure_cleanup}",
            c_cleanup_sequence(&bool_value.cleanup_after_use)
        ));
        self.body.push(format!(
            "if ({diagnostic}.ptr != NULL) {{ {bool_error_exit} }}"
        ));
        self.body
            .push(format!("phpc_native_diagnostic_free({diagnostic});"));
        self.body.extend(bool_value.cleanup_after_use);
        self.body.push(format!("_Bool {result} = !{truthy};"));
        CValue::BoolExpr(result)
    }

    fn emit_statement(&mut self, stmt: &Stmt) -> CompileResult<()> {
        if let Some(operation) = native_statement_operand_call_operation(stmt) {
            return Err(self.unsupported_call_operation(operation));
        }

        match stmt {
            Stmt::Namespace { span, .. } | Stmt::Use { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_NAMESPACE_REJECTION))
            }
            Stmt::Echo { exprs, .. } => {
                for (index, expr) in exprs.iter().enumerate() {
                    if self.try_emit_native_value_result_output(expr)? {
                        continue;
                    }
                    if self.try_emit_array_index_output(expr)? {
                        continue;
                    }
                    let value = match self.emit_expr(expr) {
                        Ok(value) => value,
                        Err(error) => {
                            return Err(self.unsupported_unemitted_statement_operands_or_original(
                                &exprs[index + 1..],
                                error,
                            ));
                        }
                    };
                    if let Err(error) = self.emit_echo(value, expr.span()) {
                        return Err(self.unsupported_unemitted_statement_operands_or_original(
                            &exprs[index + 1..],
                            error,
                        ));
                    }
                }
                Ok(())
            }
            Stmt::Print { expr, .. } => {
                if self.try_emit_native_value_result_output(expr)? {
                    return Ok(());
                }
                if self.try_emit_array_index_output(expr)? {
                    return Ok(());
                }
                let value = self.emit_expr(expr)?;
                self.emit_echo(value, expr.span())?;
                Ok(())
            }
            Stmt::Assign { target, expr, .. } => self.emit_assignment(target, expr),
            Stmt::ReferenceAssign {
                target,
                source,
                span,
            } => {
                if let Some(operation) = native_reference_assignment_call_operation(target, source)
                {
                    return Err(self.unsupported_call_operation(operation));
                }

                Err(self.unsupported(*span, ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION))
            }
            Stmt::NullCoalesceAssign { target, expr, span } => {
                if let Some(operation) = native_assignment_target_call_operation(target) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
                if self.emit_array_offset_null_coalesce_assignment(target, expr, *span)? {
                    return Ok(());
                }
                if self.emit_array_lvalue_null_coalesce_assignment(target, expr, *span)? {
                    return Ok(());
                }
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Stmt::CompoundAssign {
                target,
                op,
                expr,
                span,
            } => {
                if let Some(operation) = native_assignment_target_call_operation(target) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
                if let Some(value) = self
                    .materialize_array_lvalue_compound_assignment_result_for_target(
                        target, *op, expr, *span, "",
                    )?
                {
                    self.body.extend(value.cleanup_after_use);
                    return Ok(());
                }
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Stmt::IncrementDecrement { target, op, span } => {
                if let Some(operation) = native_assignment_target_call_operation(target) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
                if let Some(value) = self
                    .materialize_array_lvalue_increment_decrement_result_for_target(
                        target,
                        *op,
                        IncrementDecrementPosition::Pre,
                        *span,
                        "",
                    )?
                {
                    self.body.extend(value.cleanup_after_use);
                    return Ok(());
                }
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Stmt::Expr { expr, .. } => {
                self.emit_expr(expr)?;
                Ok(())
            }
            Stmt::Function(function) => Err(native_function_declaration_fallback_diagnostic(
                NativeCallBackend::Assembly,
                function,
                ASSEMBLY_STATIC_LOCAL_REJECTION,
            )),
            Stmt::Interface(interface) => {
                Err(self.unsupported(interface.span, ASSEMBLY_INTERFACE_REJECTION))
            }
            Stmt::Trait(trait_decl) => {
                Err(self.unsupported(trait_decl.span, ASSEMBLY_TRAIT_REJECTION))
            }
            Stmt::Enum(enum_decl) => Err(self.unsupported(enum_decl.span, ASSEMBLY_ENUM_REJECTION)),
            Stmt::Class(class) => {
                if let Some(span) = find_static_local_span(std::slice::from_ref(stmt)) {
                    return Err(self.unsupported(span, ASSEMBLY_STATIC_LOCAL_REJECTION));
                }
                Err(self.unsupported(class.span, ASSEMBLY_OBJECT_CLASS_REJECTION))
            }
            Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::DoWhile { span, .. }
            | Stmt::For { span, .. }
            | Stmt::Switch { span, .. }
            | Stmt::Goto { span, .. }
            | Stmt::Label { span, .. }
            | Stmt::Break { span, .. }
            | Stmt::Continue { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_CONTROL_FLOW_REJECTION))
            }
            Stmt::Foreach {
                iterable,
                key,
                value,
                by_reference,
                body,
                span,
            } => self.emit_native_array_foreach_statement(
                iterable,
                key.as_deref(),
                value,
                *by_reference,
                body,
                *span,
            ),
            Stmt::UnsetVariable { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Stmt::UnsetStaticProperty { span, .. }
            | Stmt::UnsetSelfStaticProperty { span, .. }
            | Stmt::UnsetParentStaticProperty { span, .. }
            | Stmt::UnsetLateStaticProperty { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Stmt::UnsetObjectProperty { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Stmt::UnsetDynamicObjectProperty { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Stmt::UnsetArrayIndex { name, index, span } => {
                self.emit_unset_array_index(name, index, *span)
            }
            Stmt::UnsetNestedArrayIndex {
                name,
                indices,
                span,
            } => self.emit_unset_nested_array_index(name, indices, *span),
            Stmt::UnsetMany { targets, span } => {
                if targets
                    .iter()
                    .any(is_object_property_array_access_unset_target)
                {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
                self.emit_unset_many(targets, *span)
            }
            Stmt::ConstDeclaration { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION))
            }
            Stmt::Require { span, .. } | Stmt::Include { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_REQUIRE_REJECTION))
            }
            Stmt::Throw { span, .. } => Err(self.unsupported(*span, ASSEMBLY_EXCEPTION_REJECTION)),
            Stmt::Try { span, .. } => Err(self.unsupported(*span, ASSEMBLY_TRY_BLOCK_REJECTION)),
            Stmt::Return { span, .. } => {
                Err(self.native_call_diagnostics().return_statement(*span))
            }
            Stmt::Global { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_GLOBAL_DECLARATION_REJECTION))
            }
            Stmt::StaticLocal { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_STATIC_LOCAL_REJECTION))
            }
        }
    }

    fn emit_expr(&mut self, expr: &Expr) -> CompileResult<CValue> {
        match expr {
            Expr::Null(_) => Ok(CValue::Null),
            Expr::Bool(value, _) => Ok(CValue::Bool(*value)),
            Expr::Int(value, _) => Ok(CValue::Int(value.to_string())),
            Expr::Float(value, _) => Ok(CValue::Float(format_float_literal(*value))),
            Expr::String(value, _) => Ok(CValue::String(value.clone())),
            Expr::InterpolatedString { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_INTERPOLATED_STRING_REJECTION))
            }
            Expr::MagicLine { span }
            | Expr::MagicFile { span }
            | Expr::MagicDir { span }
            | Expr::MagicFunction { span }
            | Expr::MagicClass { span }
            | Expr::MagicMethod { span } => {
                Err(self.unsupported(*span, ASSEMBLY_MAGIC_CONSTANT_REJECTION))
            }
            Expr::GlobalConstant { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION))
            }
            Expr::ClassNameConstant { span, .. }
            | Expr::SelfClassNameConstant { span }
            | Expr::ParentClassNameConstant { span }
            | Expr::StaticClassNameConstant { span } => {
                Err(self.unsupported(*span, ASSEMBLY_CLASS_NAME_CONSTANT_REJECTION))
            }
            Expr::ClassConstant { span, .. }
            | Expr::SelfClassConstant { span, .. }
            | Expr::ParentClassConstant { span, .. }
            | Expr::LateStaticClassConstant { span, .. }
            | Expr::StaticProperty { span, .. }
            | Expr::SelfStaticProperty { span, .. }
            | Expr::ParentStaticProperty { span, .. }
            | Expr::LateStaticProperty { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_STATIC_MEMBER_REJECTION))
            }
            Expr::Array { items, span } => {
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if self.uses_native_string_helpers {
                    return self
                        .emit_array_literal(items, *span)
                        .map(CValue::ArrayHandle);
                }
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            Expr::Index { target, span, .. } => {
                if self.is_string_offset_subject_expr(target) {
                    return self.emit_string_offset_read_expr(expr);
                }
                if let Some(operation) = native_dereferenced_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if let Some(superglobal_span) = request_superglobal_expr_span(target) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if is_object_offset_expr(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            Expr::AppendIndex { target, span } => {
                if let Some(operation) = native_dereferenced_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if let Some(superglobal_span) = request_superglobal_expr_span(target) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if is_object_offset_expr(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            Expr::Property { span, .. } | Expr::DynamicProperty { span, .. } => {
                if let Some(operation) = native_dereferenced_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, ASSEMBLY_OBJECT_PROPERTY_REJECTION))
            }
            Expr::ObjectStaticProperty { span, .. } => {
                if let Some(operation) = native_dereferenced_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, ASSEMBLY_STATIC_MEMBER_REJECTION))
            }
            Expr::MethodCall { .. }
            | Expr::DynamicMethodCall { .. }
            | Expr::ParentMethodCall { .. }
            | Expr::StaticMethodCall { .. }
            | Expr::ObjectStaticMethodCall { .. }
            | Expr::SelfMethodCall { .. }
            | Expr::LateStaticMethodCall { .. } => Err(self.unsupported_value_call(expr)),
            Expr::Variable(name, span) => {
                if is_request_superglobal_name(name) {
                    let value = self.materialize_request_superglobal_snapshot_value(name, "");
                    self.retain_native_value_cleanup_handle(&value.handle);
                    return Ok(CValue::NativeValueHandle(value.handle));
                }
                self.variables
                    .get(name)
                    .cloned()
                    .ok_or_else(|| self.unsupported(*span, ASSEMBLY_VARIABLE_READ_REJECTION))
            }
            Expr::Call { name, args, span } if is_exit_construct_name(name) => {
                Err(self.unsupported_direct_named_call(args, *span, ASSEMBLY_TERMINATION_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("defined") => {
                self.emit_defined_call(args, *span)
            }
            Expr::Call { name, args, span } if is_global_constant_builtin(name) => Err(
                self.unsupported_direct_named_call(args, *span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION)
            ),
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("isset") => {
                self.emit_isset_call(args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("empty") => {
                self.emit_empty_call(args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("strlen") => {
                self.emit_strlen_call(args, *span)
            }
            Expr::Call { name, args, span } if native_value_cast_builtin_op_tag(name).is_some() => {
                Err(self.unsupported_direct_named_call(args, *span, ASSEMBLY_CAST_REJECTION))
            }
            Expr::Call { name, args, span } if native_string_predicate_for_name(name).is_some() => {
                let predicate = native_string_predicate_for_name(name)
                    .expect("string predicate guard should provide operation");
                self.emit_string_predicate_call(predicate, args, *span)
            }
            Expr::Call { name, args, span }
                if native_string_int_operation_for_name(name).is_some() =>
            {
                let operation = native_string_int_operation_for_name(name)
                    .expect("string-int guard should provide operation");
                self.emit_string_int_operation_call(operation, args, *span)
            }
            Expr::Call { name, args, span }
                if native_string_distance_operation_for_name(name).is_some() =>
            {
                let operation = native_string_distance_operation_for_name(name)
                    .expect("string-distance guard should provide operation");
                self.emit_string_distance_operation_call(operation, args, *span)
            }
            Expr::Call { name, args, span }
                if native_string_result_operation_for_name(name).is_some() =>
            {
                Err(self.unsupported_direct_named_call(
                    args,
                    *span,
                    ASSEMBLY_STRING_RESULT_OPERATION_REJECTION,
                ))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("basename") => {
                Err(self.unsupported_direct_named_call(args, *span, ASSEMBLY_BASENAME_REJECTION))
            }
            Expr::Call { name, args, span }
                if native_filesystem_path_operation_for_name(name).is_some() =>
            {
                let operation = native_filesystem_path_operation_for_name(name)
                    .expect("filesystem-path guard should provide operation");
                self.emit_filesystem_path_operation_call(operation, args, *span)
            }
            Expr::Call { name, args, span } if is_stream_resource_builtin(name) => Err(
                self.unsupported_direct_named_call(args, *span, ASSEMBLY_STREAM_RESOURCE_REJECTION)
            ),
            Expr::Call { name, args, span } if is_header_state_builtin(name) => Err(
                self.unsupported_direct_named_call(args, *span, ASSEMBLY_HEADER_STATE_REJECTION)
            ),
            Expr::Call { name, args, span } if is_session_state_builtin(name) => Err(
                self.unsupported_direct_named_call(args, *span, ASSEMBLY_SESSION_STATE_REJECTION)
            ),
            Expr::Call { name, args, span } if is_output_buffer_builtin(name) => Err(
                self.unsupported_direct_named_call(args, *span, ASSEMBLY_OUTPUT_BUFFER_REJECTION)
            ),
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("function_exists") => {
                self.emit_function_exists_call(args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("is_callable") => {
                self.emit_is_callable_call(args, *span)
            }
            Expr::Call { name, args, span } if is_native_type_introspection_builtin(name) => {
                self.emit_native_type_introspection_call(name, args, *span)
            }
            Expr::Call { name, args, span } if is_object_metadata_builtin(name) => Err(
                self.unsupported_direct_named_call(args, *span, ASSEMBLY_OBJECT_METADATA_REJECTION)
            ),
            Expr::Call { name, args, span } if native_array_sort_builtin(name, args).is_some() => {
                let builtin = native_array_sort_builtin(name, args)
                    .expect("array sort guard should provide operation");
                self.emit_native_array_sort_call(builtin, args, *span, "")
            }
            Expr::Call { name, args, span }
                if native_value_array_callback_builtin(name, args).is_some() =>
            {
                let builtin = native_value_array_callback_builtin(name, args)
                    .expect("array callback guard should provide operation");
                let value = self.emit_native_value_array_callback_call(builtin, args, *span, "")?;
                self.retain_native_value_cleanup_handle(&value.handle);
                Ok(CValue::NativeValueHandle(value.handle))
            }
            Expr::Call { name, args, span } if is_array_builtin(name) => {
                Err(self.unsupported_direct_named_call(args, *span, ASSEMBLY_ARRAY_REJECTION))
            }
            Expr::DynamicCall { .. } => Err(self.unsupported_value_call(expr)),
            Expr::Call { .. } => Err(self.unsupported_value_call(expr)),
            Expr::InstanceOf { span, .. } => {
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, ASSEMBLY_INSTANCEOF_REJECTION))
            }
            Expr::Closure { .. } => Err(self.native_call_diagnostics().call_root(expr)),
            Expr::New { .. } => Err(self.unsupported_value_call(expr)),
            Expr::Clone { span, .. } => {
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, ASSEMBLY_CLONE_REJECTION))
            }
            Expr::Unary { op, expr, span } => {
                if matches!(op, UnaryOp::Not) {
                    if let Expr::Unary {
                        op: UnaryOp::Not,
                        expr,
                        ..
                    } = expr.as_ref()
                    {
                        let value = self.emit_value_operand_expr(expr)?;
                        if matches!(
                            value,
                            CValue::Bool(_) | CValue::BoolExpr(_) | CValue::ComparisonDecision(_)
                        ) {
                            return Ok(value);
                        }
                        let inverted = self.emit_bool_not(value, *span)?;
                        return self.emit_bool_not(inverted, *span);
                    }
                }
                if matches!(op, UnaryOp::BitwiseNot) {
                    if let Expr::Unary {
                        op: UnaryOp::BitwiseNot,
                        expr,
                        ..
                    } = expr.as_ref()
                    {
                        return match self.emit_value_operand_expr(expr)? {
                            value @ CValue::Int(_) => Ok(value),
                            _ => Err(self.unsupported(*span, ASSEMBLY_BITWISE_REJECTION)),
                        };
                    }
                }
                let value = self.emit_value_operand_expr(expr)?;
                self.emit_unary(*op, value, *span)
            }
            Expr::ErrorControl { span, .. } => {
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, ASSEMBLY_ERROR_CONTROL_REJECTION))
            }
            Expr::Include { span, .. } | Expr::Require { span, .. } => {
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, ASSEMBLY_REQUIRE_EXPRESSION_REJECTION))
            }
            Expr::Cast { span, .. } => {
                if let Some(operation) = native_value_operand_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                Err(self.unsupported(*span, ASSEMBLY_CAST_REJECTION))
            }
            Expr::Assign { target, expr, span } => {
                if let Some(operation) = native_assignment_target_call_operation(target) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
                if is_static_member_assign_target(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_STATIC_MEMBER_REJECTION));
                }
                if let Some(value) =
                    self.emit_array_offset_mutation_assignment_expr(target, expr, *span)?
                {
                    return Ok(value);
                }
                if let Some(value) = self.emit_array_lvalue_assignment_expr(target, expr, *span)? {
                    return Ok(value);
                }
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Expr::NullCoalesceAssign { target, expr, span } => {
                if let Some(operation) = native_assignment_target_call_operation(target) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
                if let Some(value) = self.materialize_array_offset_null_coalesce_assignment_expr(
                    target, expr, *span, "",
                )? {
                    self.retain_native_value_cleanup_handle(&value.handle);
                    return Ok(CValue::NativeValueHandle(value.handle));
                }
                if let Some(value) = self.materialize_array_lvalue_null_coalesce_assignment_expr(
                    target, expr, *span, "",
                )? {
                    self.retain_native_value_cleanup_handle(&value.handle);
                    return Ok(CValue::NativeValueHandle(value.handle));
                }
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Expr::CompoundAssign {
                target,
                op,
                expr,
                span,
            } => {
                if let Some(operation) = native_assignment_target_call_operation(target) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
                if let Some(value) = self
                    .materialize_array_lvalue_compound_assignment_result_for_target(
                        target, *op, expr, *span, "",
                    )?
                {
                    self.retain_native_value_cleanup_handle(&value.handle);
                    return Ok(CValue::NativeValueHandle(value.handle));
                }
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Expr::IncrementDecrement {
                target,
                op,
                position,
                span,
            } => {
                if let Some(operation) = native_assignment_target_call_operation(target) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
                if let Some(value) = self
                    .materialize_array_lvalue_increment_decrement_result_for_target(
                        target, *op, *position, *span, "",
                    )?
                {
                    self.retain_native_value_cleanup_handle(&value.handle);
                    return Ok(CValue::NativeValueHandle(value.handle));
                }
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Expr::Ternary {
                condition,
                if_true,
                if_false,
                span,
            } => self.emit_ternary_expr(condition, if_true, if_false, *span),
            Expr::ShortTernary {
                condition,
                if_false,
                span,
            } => self.emit_short_ternary(condition, if_false, *span),
            Expr::Binary {
                left,
                op,
                right,
                span,
            } => {
                if is_comparison_op(*op) && !matches!(op, BinaryOp::StrictEq | BinaryOp::StrictNe) {
                    return self.emit_scalar_comparison_expr(left, *op, right, *span);
                }
                if matches!(op, BinaryOp::NullCoalesce) {
                    if let Some(operation) = native_value_operand_call_result_operation(expr) {
                        return Err(self.unsupported_call_operation(operation));
                    }
                    return Err(self.unsupported(*span, ASSEMBLY_CONDITIONAL_REJECTION));
                }
                if matches!(op, BinaryOp::Concat) {
                    return self.emit_static_string_concat_expr(left, right, *span);
                }
                if matches!(
                    op,
                    BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::LogicalXor
                ) {
                    return self.emit_logical_expr(left, *op, right, *span);
                }
                let (left, right) = self.emit_binary_value_operand_exprs(left, right)?;
                self.emit_binary(left, *op, right, *span)
            }
        }
    }

    fn emit_isset_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        if args.is_empty() {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let mut dynamic_checks = Vec::new();
        for arg in args {
            if let Some(name) = request_superglobal_root_name(arg) {
                match self.emit_request_superglobal_isset_expr(name, "") {
                    CValue::Bool(false) => return Ok(CValue::Bool(false)),
                    CValue::Bool(true) => continue,
                    CValue::BoolExpr(value) => dynamic_checks.push(value),
                    _ => unreachable!("request superglobal isset returns a bool C value"),
                }
                continue;
            }

            if let Some(superglobal_span) = request_superglobal_expr_span(arg) {
                return Err(
                    self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                );
            }

            if is_array_access_offset_expr(arg) {
                return Err(self.unsupported(arg.span(), ASSEMBLY_ARRAY_ACCESS_REJECTION));
            }

            if let Some(value) =
                self.emit_value_offset_bool_operation_expr(arg, NativeStringOffsetOperation::Isset)?
            {
                match value {
                    CValue::Bool(false) => return Ok(CValue::Bool(false)),
                    CValue::Bool(true) => continue,
                    CValue::BoolExpr(value) => dynamic_checks.push(value),
                    _ => unreachable!("value offset isset returns a bool C value"),
                }
                continue;
            }

            if let Some(operation) =
                native_direct_call_argument_result_operation(std::slice::from_ref(arg), span)
            {
                return Err(self.unsupported_call_operation(operation));
            }

            let Expr::Variable(name, _) = arg else {
                return Err(self.unsupported(arg.span(), ASSEMBLY_ISSET_REJECTION));
            };

            if matches!(self.variables.get(name), None | Some(CValue::Null)) {
                return Ok(CValue::Bool(false));
            }
        }

        Ok(match dynamic_checks.len() {
            0 => CValue::Bool(true),
            1 => CValue::BoolExpr(dynamic_checks.remove(0)),
            _ => CValue::BoolExpr(
                dynamic_checks
                    .into_iter()
                    .map(|value| format!("({value})"))
                    .collect::<Vec<_>>()
                    .join(" && "),
            ),
        })
    }

    fn emit_empty_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        let [arg] = args else {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        };

        if let Some(name) = request_superglobal_root_name(arg) {
            return Ok(self.emit_request_superglobal_empty_expr(name, ""));
        }

        if let Some(superglobal_span) = request_superglobal_expr_span(arg) {
            return Err(self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION));
        }

        if is_array_access_offset_expr(arg) {
            return Err(self.unsupported(arg.span(), ASSEMBLY_ARRAY_ACCESS_REJECTION));
        }

        if let Some(value) =
            self.emit_value_offset_bool_operation_expr(arg, NativeStringOffsetOperation::Empty)?
        {
            return Ok(value);
        }

        if let Some(operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(operation));
        }

        let Expr::Variable(name, _) = arg else {
            return Err(self.unsupported(arg.span(), ASSEMBLY_EMPTY_REJECTION));
        };

        let Some(value) = self.variables.get(name) else {
            return Ok(CValue::Bool(true));
        };

        self.known_truthiness_for_value(value)
            .map(|truthy| CValue::Bool(!truthy))
            .ok_or_else(|| self.unsupported(arg.span(), ASSEMBLY_EMPTY_REJECTION))
    }

    fn emit_value_offset_bool_operation_expr(
        &mut self,
        expr: &Expr,
        operation: NativeStringOffsetOperation,
    ) -> CompileResult<Option<CValue>> {
        let Expr::Index { target, index, .. } = expr else {
            return Ok(None);
        };

        let subject = self.materialize_native_value_result_operand(target, "")?;
        let offset_failure_cleanup = c_cleanup_sequence(&subject.cleanup_after_use);
        let offset =
            self.materialize_native_value_result_operand(index, &offset_failure_cleanup)?;
        let mut operand_cleanup = offset.cleanup_after_use;
        operand_cleanup.extend(subject.cleanup_after_use);

        self.uses_native_string_helpers = true;
        let result = self.next_native_name("value_offset_bool_value");
        let diagnostic = self.next_native_name("value_offset_bool_diagnostic");
        let bool_diagnostic = self.next_native_name("value_offset_bool_result_diagnostic");
        let bool_result = self.next_native_name("value_offset_bool_result");
        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {result} = phpc_native_value_offset_operation_with_diagnostic({}, {}, {}, &{diagnostic});",
            subject.handle,
            offset.handle,
            operation as u8
        ));
        let result_cleanup = format!(
            "phpc_native_diagnostic_report({diagnostic}); {}",
            c_cleanup_sequence(&operand_cleanup)
        );
        let result_error_exit = self.native_error_exit(&result_cleanup);
        self.body.push(format!(
            "if ({result}.ptr == NULL) {{ {result_error_exit} }}"
        ));
        self.body
            .push(format!("phpc_native_diagnostic_free({diagnostic});"));
        self.body.extend(operand_cleanup);
        self.body.push(format!(
            "phpc_NativeDiagnosticHandle {bool_diagnostic} = {{0}};"
        ));
        self.body.push(format!(
            "_Bool {bool_result} = phpc_native_value_bool_with_diagnostic({result}, &{bool_diagnostic});"
        ));
        let bool_error_exit = self.native_error_exit(&format!(
            "phpc_native_diagnostic_report({bool_diagnostic}); phpc_native_value_free({result}); "
        ));
        self.body.push(format!(
            "if ({bool_diagnostic}.ptr != NULL) {{ {bool_error_exit} }}"
        ));
        self.body
            .push(format!("phpc_native_diagnostic_free({bool_diagnostic});"));
        self.body.push(format!("phpc_native_value_free({result});"));

        Ok(Some(CValue::BoolExpr(bool_result)))
    }

    fn emit_string_offset_read_expr(&mut self, expr: &Expr) -> CompileResult<CValue> {
        let Expr::Index { target, index, .. } = expr else {
            unreachable!("string offset reads are emitted from index expressions")
        };

        let subject = self.materialize_native_value_result_operand(target, "")?;
        let offset_failure_cleanup = c_cleanup_sequence(&subject.cleanup_after_use);
        let offset =
            self.materialize_native_value_result_operand(index, &offset_failure_cleanup)?;
        let mut operand_cleanup = offset.cleanup_after_use;
        operand_cleanup.extend(subject.cleanup_after_use);

        self.uses_native_string_helpers = true;
        self.uses_native_value_string_clone_bytes = true;
        let read = self.next_native_name("string_offset_read_value");
        let diagnostic = self.next_native_name("string_offset_read_diagnostic");
        let buffer = self.next_native_name("string_offset_read_buffer");
        let bytes = self.next_native_name("string_offset_read_bytes");
        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {read} = phpc_native_value_string_offset_operation_with_diagnostic({}, {}, {}, &{diagnostic});",
            subject.handle,
            offset.handle,
            NativeStringOffsetOperation::Read as u8
        ));
        self.emit_report_native_diagnostic(&diagnostic);
        let read_error_exit = self.native_error_exit(&format!(
            "phpc_native_value_free({read}); {}",
            c_cleanup_sequence(&operand_cleanup)
        ));
        self.body
            .push(format!("if ({read}.ptr == NULL) {{ {read_error_exit} }}"));
        self.body.push(format!(
            "phpc_NativeByteBuffer {buffer} = phpc_native_value_string_clone_bytes({read});"
        ));
        self.body
            .push(format!("const uint8_t *{bytes} = {buffer}.ptr;"));
        self.body.push(format!("phpc_native_value_free({read});"));
        self.body.extend(operand_cleanup);
        self.known_string_lengths
            .insert(bytes.clone(), format!("{buffer}.len"));
        self.owned_native_byte_buffers.push(buffer);

        Ok(CValue::StringExpr(bytes))
    }

    fn is_string_offset_subject_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::String(_, _) => true,
            Expr::Variable(name, _) => matches!(
                self.variables.get(name),
                Some(CValue::String(_) | CValue::StringExpr(_))
            ),
            Expr::Call { name, args, .. } => {
                args.len() == 1 && native_string_result_operation_for_name(name).is_some()
            }
            _ => false,
        }
    }

    fn emit_string_offset_write_assignment(
        &mut self,
        name: &str,
        subject: CValue,
        index_expr: &Expr,
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<()> {
        let subject = self.materialize_native_array_c_value_handle(subject, span)?;
        let subject_cleanup = c_cleanup_sequence(&subject.cleanup_after_use);
        let offset = self.materialize_native_value_result_operand(index_expr, &subject_cleanup)?;
        let offset_cleanup = c_cleanup_sequence(&offset.cleanup_after_use);
        let replacement_failure_cleanup = format!("{offset_cleanup}{subject_cleanup}");
        let replacement = self.materialize_native_value_result_operand(
            replacement_expr,
            &replacement_failure_cleanup,
        )?;

        self.uses_native_string_helpers = true;
        self.uses_native_value_string_clone_bytes = true;
        self.uses_native_value_offset_mutation = true;

        let write = self.next_native_name("string_offset_write_value");
        let diagnostic = self.next_native_name("string_offset_write_diagnostic");
        let buffer = self.next_native_name("string_offset_write_buffer");
        let bytes = self.next_native_name("string_offset_write_bytes");

        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {write} = phpc_native_value_offset_mutation_operation_with_diagnostic({}, {}, {}, {NATIVE_VALUE_OFFSET_MUTATION_WRITE}, &{diagnostic});",
            subject.handle, offset.handle, replacement.handle
        ));
        self.body.push(format!(
            "if ({diagnostic}.ptr != NULL) {{ phpc_native_diagnostic_report({diagnostic}); }}"
        ));
        let write_failure_cleanup = format!(
            "{}{}{}",
            c_cleanup_sequence(&replacement.cleanup_after_use),
            c_cleanup_sequence(&offset.cleanup_after_use),
            c_cleanup_sequence(&subject.cleanup_after_use)
        );
        let write_error_exit = self.native_error_exit(&write_failure_cleanup);
        self.body
            .push(format!("if ({write}.ptr == NULL) {{ {write_error_exit} }}"));
        self.body.push(format!(
            "phpc_NativeByteBuffer {buffer} = phpc_native_value_string_clone_bytes({write});"
        ));
        self.body
            .push(format!("const uint8_t *{bytes} = {buffer}.ptr;"));
        self.body.push(format!("phpc_native_value_free({write});"));
        self.body.extend(replacement.cleanup_after_use);
        self.body.extend(offset.cleanup_after_use);
        self.body.extend(subject.cleanup_after_use);
        self.known_string_lengths
            .insert(bytes.clone(), format!("{buffer}.len"));
        self.owned_native_byte_buffers.push(buffer);
        self.store_variable_value(name, CValue::StringExpr(bytes));
        Ok(())
    }

    fn emit_array_offset_mutation_assignment(
        &mut self,
        name: &str,
        handle: &str,
        index_expr: &Expr,
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<()> {
        self.emit_array_offset_mutation(
            name,
            handle,
            Some(index_expr),
            Some(replacement_expr),
            NATIVE_VALUE_OFFSET_MUTATION_WRITE,
            "array_offset_write",
            span,
        )
    }

    fn emit_array_offset_append_assignment(
        &mut self,
        name: &str,
        handle: &str,
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<()> {
        self.emit_array_offset_mutation(
            name,
            handle,
            None,
            Some(replacement_expr),
            NATIVE_VALUE_OFFSET_MUTATION_APPEND,
            "array_offset_append",
            span,
        )
    }

    fn emit_value_offset_append_assignment(
        &mut self,
        name: &str,
        subject: CValue,
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<()> {
        let subject = self.materialize_native_array_c_value_handle(subject, span)?;
        let subject_cleanup = c_cleanup_sequence(&subject.cleanup_after_use);
        let replacement =
            self.materialize_native_value_result_operand(replacement_expr, &subject_cleanup)?;

        self.uses_native_string_helpers = true;
        self.uses_native_value_offset_mutation = true;
        self.uses_native_value_clone = true;

        let diagnostic = self.next_native_name("value_offset_append_diagnostic");
        let append_value = self.next_native_name("value_offset_append_value");
        let value_to_clone = self.next_native_name("value_offset_append_value_to_clone");
        let stored_value = self.next_native_name("value_offset_append_stored_value");

        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {append_value} = phpc_native_value_offset_mutation_operation_with_diagnostic({}, (phpc_NativeValueHandle){{0}}, {}, {NATIVE_VALUE_OFFSET_MUTATION_APPEND}, &{diagnostic});",
            subject.handle, replacement.handle
        ));
        self.emit_report_native_diagnostic(&diagnostic);
        self.body.push(format!(
            "phpc_NativeValueHandle {value_to_clone} = {};",
            subject.handle
        ));
        self.body.push(format!(
            "if ({append_value}.ptr != NULL) {{ {value_to_clone} = {append_value}; }}"
        ));
        self.body.push(format!(
            "phpc_NativeValueHandle {stored_value} = phpc_native_value_clone({value_to_clone});"
        ));
        self.body
            .push(format!("phpc_native_value_free({append_value});"));
        self.body.extend(replacement.cleanup_after_use);
        self.body.extend(subject.cleanup_after_use);
        self.store_native_value_result_variable(
            name,
            CNativeValueMaterialization {
                handle: stored_value,
                cleanup_after_use: Vec::new(),
            },
        );
        Ok(())
    }

    fn emit_value_offset_write_assignment(
        &mut self,
        name: &str,
        subject: CValue,
        index_expr: &Expr,
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<()> {
        let subject = self.materialize_native_array_c_value_handle(subject, span)?;
        let subject_cleanup = c_cleanup_sequence(&subject.cleanup_after_use);
        let offset = self.materialize_native_value_result_operand(index_expr, &subject_cleanup)?;
        let offset_cleanup = c_cleanup_sequence(&offset.cleanup_after_use);
        let replacement_failure_cleanup = format!("{offset_cleanup}{subject_cleanup}");
        let replacement = self.materialize_native_value_result_operand(
            replacement_expr,
            &replacement_failure_cleanup,
        )?;

        self.uses_native_string_helpers = true;
        self.uses_native_value_offset_mutation = true;
        self.uses_native_value_clone = true;

        let diagnostic = self.next_native_name("value_offset_write_diagnostic");
        let write_value = self.next_native_name("value_offset_write_value");
        let value_to_clone = self.next_native_name("value_offset_write_value_to_clone");
        let stored_value = self.next_native_name("value_offset_write_stored_value");

        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {write_value} = phpc_native_value_offset_mutation_operation_with_diagnostic({}, {}, {}, {NATIVE_VALUE_OFFSET_MUTATION_WRITE}, &{diagnostic});",
            subject.handle, offset.handle, replacement.handle
        ));
        self.emit_report_native_diagnostic(&diagnostic);
        self.body.push(format!(
            "phpc_NativeValueHandle {value_to_clone} = {};",
            subject.handle
        ));
        self.body.push(format!(
            "if ({write_value}.ptr != NULL) {{ {value_to_clone} = {write_value}; }}"
        ));
        self.body.push(format!(
            "phpc_NativeValueHandle {stored_value} = phpc_native_value_clone({value_to_clone});"
        ));
        self.body
            .push(format!("phpc_native_value_free({write_value});"));
        self.body.extend(replacement.cleanup_after_use);
        self.body.extend(offset.cleanup_after_use);
        self.body.extend(subject.cleanup_after_use);
        self.store_native_value_result_variable(
            name,
            CNativeValueMaterialization {
                handle: stored_value,
                cleanup_after_use: Vec::new(),
            },
        );
        Ok(())
    }

    fn emit_value_offset_path_write_assignment(
        &mut self,
        name: &str,
        subject: CValue,
        indices: &[Expr],
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<()> {
        let subject = self.materialize_native_array_c_value_handle(subject, span)?;
        let subject_cleanup = c_cleanup_sequence(&subject.cleanup_after_use);
        let (offsets_ptr, offsets_len, offset_values) =
            self.emit_native_value_offset_key_path(indices, &subject_cleanup)?;
        let offset_cleanup_steps = offset_values
            .iter()
            .flat_map(|offset| offset.cleanup_after_use.clone())
            .collect::<Vec<_>>();
        let offset_cleanup = c_cleanup_sequence(&offset_cleanup_steps);
        let replacement_failure_cleanup = format!("{offset_cleanup}{subject_cleanup}");
        let replacement = self.materialize_native_value_result_operand(
            replacement_expr,
            &replacement_failure_cleanup,
        )?;

        self.emit_value_offset_path_write_assignment_from_materialized(
            name,
            subject,
            offset_values,
            offsets_ptr,
            offsets_len,
            replacement,
        )
    }

    fn emit_value_offset_path_write_assignment_from_materialized(
        &mut self,
        name: &str,
        subject: CNativeValueMaterialization,
        offset_values: Vec<CNativeValueMaterialization>,
        offsets_ptr: String,
        offsets_len: usize,
        replacement: CNativeValueMaterialization,
    ) -> CompileResult<()> {
        self.uses_native_string_helpers = true;
        self.uses_native_value_offset_mutation = true;
        self.uses_native_value_offset_path_write = true;
        self.uses_native_value_clone = true;

        let diagnostic = self.next_native_name("value_offset_path_write_diagnostic");
        let path_value = self.next_native_name("value_offset_path_write_value");
        let value_to_clone = self.next_native_name("value_offset_path_write_value_to_clone");
        let stored_value = self.next_native_name("value_offset_path_write_stored_value");

        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {path_value} = phpc_native_value_offset_path_write_with_diagnostic({}, {offsets_ptr}, {offsets_len}, {}, &{diagnostic});",
            subject.handle, replacement.handle
        ));
        self.emit_report_native_diagnostic(&diagnostic);
        self.body.push(format!(
            "phpc_NativeValueHandle {value_to_clone} = {};",
            subject.handle
        ));
        self.body.push(format!(
            "if ({path_value}.ptr != NULL) {{ {value_to_clone} = {path_value}; }}"
        ));
        self.body.push(format!(
            "phpc_NativeValueHandle {stored_value} = phpc_native_value_clone({value_to_clone});"
        ));
        self.body
            .push(format!("phpc_native_value_free({path_value});"));
        self.body.extend(replacement.cleanup_after_use);
        for offset in offset_values {
            self.body.extend(offset.cleanup_after_use);
        }
        self.body.extend(subject.cleanup_after_use);
        self.store_native_value_result_variable(
            name,
            CNativeValueMaterialization {
                handle: stored_value,
                cleanup_after_use: Vec::new(),
            },
        );
        Ok(())
    }

    fn emit_value_offset_path_write_assignment_expr(
        &mut self,
        name: &str,
        subject: CValue,
        indices: &[Expr],
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<CValue> {
        let subject = self.materialize_native_array_c_value_handle(subject, span)?;
        let subject_cleanup = c_cleanup_sequence(&subject.cleanup_after_use);
        let (offsets_ptr, offsets_len, offset_values) =
            self.emit_native_value_offset_key_path(indices, &subject_cleanup)?;
        let offset_cleanup_steps = offset_values
            .iter()
            .flat_map(|offset| offset.cleanup_after_use.clone())
            .collect::<Vec<_>>();
        let replacement_failure_cleanup = format!(
            "{}{}",
            c_cleanup_sequence(&offset_cleanup_steps),
            subject_cleanup
        );
        let (replacement_value, replacement) = self
            .materialize_assignment_expression_replacement_value(
                replacement_expr,
                &replacement_failure_cleanup,
            )?;

        self.emit_value_offset_path_write_assignment_from_materialized(
            name,
            subject,
            offset_values,
            offsets_ptr,
            offsets_len,
            replacement,
        )?;
        Ok(replacement_value)
    }

    fn emit_value_offset_path_append_assignment(
        &mut self,
        name: &str,
        subject: CValue,
        indices: &[Expr],
        suffix_indices: &[Expr],
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<()> {
        let subject = self.materialize_native_array_c_value_handle(subject, span)?;
        let subject_cleanup = c_cleanup_sequence(&subject.cleanup_after_use);
        let (prefix_offsets_ptr, prefix_offsets_len, mut offset_values) =
            self.emit_native_value_offset_key_path(indices, &subject_cleanup)?;
        let prefix_cleanup_steps = offset_values
            .iter()
            .flat_map(|offset| offset.cleanup_after_use.clone())
            .collect::<Vec<_>>();
        let prefix_cleanup = c_cleanup_sequence(&prefix_cleanup_steps);
        let suffix_failure_cleanup = format!("{prefix_cleanup}{subject_cleanup}");
        let (suffix_offsets_ptr, suffix_offsets_len, suffix_offset_values) =
            self.emit_native_value_offset_key_path(suffix_indices, &suffix_failure_cleanup)?;
        offset_values.extend(suffix_offset_values);
        let offset_cleanup_steps = offset_values
            .iter()
            .flat_map(|offset| offset.cleanup_after_use.clone())
            .collect::<Vec<_>>();
        let offset_cleanup = c_cleanup_sequence(&offset_cleanup_steps);
        let replacement_failure_cleanup = format!("{offset_cleanup}{subject_cleanup}");
        let replacement = self.materialize_native_value_result_operand(
            replacement_expr,
            &replacement_failure_cleanup,
        )?;

        self.emit_value_offset_path_append_assignment_from_materialized(
            name,
            subject,
            offset_values,
            prefix_offsets_ptr,
            prefix_offsets_len,
            suffix_offsets_ptr,
            suffix_offsets_len,
            replacement,
        )
    }

    fn emit_value_offset_path_append_assignment_from_materialized(
        &mut self,
        name: &str,
        subject: CNativeValueMaterialization,
        offset_values: Vec<CNativeValueMaterialization>,
        prefix_offsets_ptr: String,
        prefix_offsets_len: usize,
        suffix_offsets_ptr: String,
        suffix_offsets_len: usize,
        replacement: CNativeValueMaterialization,
    ) -> CompileResult<()> {
        self.uses_native_string_helpers = true;
        self.uses_native_value_offset_mutation = true;
        self.uses_native_value_offset_path_append = true;
        self.uses_native_value_clone = true;

        let diagnostic = self.next_native_name("value_offset_path_append_diagnostic");
        let path_value = self.next_native_name("value_offset_path_append_value");
        let value_to_clone = self.next_native_name("value_offset_path_append_value_to_clone");
        let stored_value = self.next_native_name("value_offset_path_append_stored_value");

        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {path_value} = phpc_native_value_offset_path_append_with_diagnostic({}, {prefix_offsets_ptr}, {prefix_offsets_len}, {suffix_offsets_ptr}, {suffix_offsets_len}, {}, &{diagnostic});",
            subject.handle, replacement.handle
        ));
        self.emit_report_native_diagnostic(&diagnostic);
        self.body.push(format!(
            "phpc_NativeValueHandle {value_to_clone} = {};",
            subject.handle
        ));
        self.body.push(format!(
            "if ({path_value}.ptr != NULL) {{ {value_to_clone} = {path_value}; }}"
        ));
        self.body.push(format!(
            "phpc_NativeValueHandle {stored_value} = phpc_native_value_clone({value_to_clone});"
        ));
        self.body
            .push(format!("phpc_native_value_free({path_value});"));
        self.body.extend(replacement.cleanup_after_use);
        for offset in offset_values {
            self.body.extend(offset.cleanup_after_use);
        }
        self.body.extend(subject.cleanup_after_use);
        self.store_native_value_result_variable(
            name,
            CNativeValueMaterialization {
                handle: stored_value,
                cleanup_after_use: Vec::new(),
            },
        );
        Ok(())
    }

    fn emit_value_offset_path_append_assignment_expr(
        &mut self,
        name: &str,
        subject: CValue,
        indices: &[Expr],
        suffix_indices: &[Expr],
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<CValue> {
        let subject = self.materialize_native_array_c_value_handle(subject, span)?;
        let subject_cleanup = c_cleanup_sequence(&subject.cleanup_after_use);
        let (prefix_offsets_ptr, prefix_offsets_len, mut offset_values) =
            self.emit_native_value_offset_key_path(indices, &subject_cleanup)?;
        let prefix_cleanup_steps = offset_values
            .iter()
            .flat_map(|offset| offset.cleanup_after_use.clone())
            .collect::<Vec<_>>();
        let suffix_failure_cleanup = format!(
            "{}{}",
            c_cleanup_sequence(&prefix_cleanup_steps),
            subject_cleanup
        );
        let (suffix_offsets_ptr, suffix_offsets_len, suffix_offset_values) =
            self.emit_native_value_offset_key_path(suffix_indices, &suffix_failure_cleanup)?;
        offset_values.extend(suffix_offset_values);
        let offset_cleanup_steps = offset_values
            .iter()
            .flat_map(|offset| offset.cleanup_after_use.clone())
            .collect::<Vec<_>>();
        let replacement_failure_cleanup = format!(
            "{}{}",
            c_cleanup_sequence(&offset_cleanup_steps),
            subject_cleanup
        );
        let (replacement_value, replacement) = self
            .materialize_assignment_expression_replacement_value(
                replacement_expr,
                &replacement_failure_cleanup,
            )?;

        self.emit_value_offset_path_append_assignment_from_materialized(
            name,
            subject,
            offset_values,
            prefix_offsets_ptr,
            prefix_offsets_len,
            suffix_offsets_ptr,
            suffix_offsets_len,
            replacement,
        )?;
        Ok(replacement_value)
    }

    fn emit_value_offset_path_unset_statement(
        &mut self,
        name: &str,
        subject: CValue,
        indices: &[Expr],
        span: Span,
    ) -> CompileResult<()> {
        let subject = self.materialize_native_array_c_value_handle(subject, span)?;
        let subject_cleanup = c_cleanup_sequence(&subject.cleanup_after_use);
        let (offsets_ptr, offsets_len, offset_values) =
            self.emit_native_value_offset_key_path(indices, &subject_cleanup)?;

        self.uses_native_string_helpers = true;
        self.uses_native_value_offset_mutation = true;
        self.uses_native_value_offset_path_unset = true;
        self.uses_native_value_clone = true;

        let diagnostic = self.next_native_name("value_offset_path_unset_diagnostic");
        let path_value = self.next_native_name("value_offset_path_unset_value");
        let value_to_clone = self.next_native_name("value_offset_path_unset_value_to_clone");
        let stored_value = self.next_native_name("value_offset_path_unset_stored_value");

        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {path_value} = phpc_native_value_offset_path_unset_with_diagnostic({}, {offsets_ptr}, {offsets_len}, &{diagnostic});",
            subject.handle
        ));
        self.emit_report_native_diagnostic(&diagnostic);
        self.body.push(format!(
            "phpc_NativeValueHandle {value_to_clone} = {};",
            subject.handle
        ));
        self.body.push(format!(
            "if ({path_value}.ptr != NULL) {{ {value_to_clone} = {path_value}; }}"
        ));
        self.body.push(format!(
            "phpc_NativeValueHandle {stored_value} = phpc_native_value_clone({value_to_clone});"
        ));
        self.body
            .push(format!("phpc_native_value_free({path_value});"));
        for offset in offset_values {
            self.body.extend(offset.cleanup_after_use);
        }
        self.body.extend(subject.cleanup_after_use);
        self.store_native_value_result_variable(
            name,
            CNativeValueMaterialization {
                handle: stored_value,
                cleanup_after_use: Vec::new(),
            },
        );
        Ok(())
    }

    fn materialize_assignment_expression_replacement_value(
        &mut self,
        replacement_expr: &Expr,
        failure_cleanup: &str,
    ) -> CompileResult<(CValue, CNativeValueMaterialization)> {
        if let Some(value) =
            self.try_materialize_native_value_result_expr(replacement_expr, failure_cleanup)?
        {
            let result_handle = value.handle;
            self.retain_native_value_cleanup_handle(&result_handle);
            let replacement_handle = self.clone_native_value_handle(&result_handle);
            return Ok((
                CValue::NativeValueHandle(result_handle),
                CNativeValueMaterialization {
                    handle: replacement_handle.clone(),
                    cleanup_after_use: vec![format!(
                        "phpc_native_value_free({replacement_handle});"
                    )],
                },
            ));
        }

        let replacement_value = self.emit_expr(replacement_expr)?;
        let replacement = self.materialize_native_array_c_value_handle(
            replacement_value.clone(),
            replacement_expr.span(),
        )?;
        Ok((replacement_value, replacement))
    }

    fn emit_native_value_offset_key_path(
        &mut self,
        indices: &[Expr],
        failure_cleanup: &str,
    ) -> CompileResult<(String, usize, Vec<CNativeValueMaterialization>)> {
        if indices.is_empty() {
            return Ok(("NULL".to_string(), 0, Vec::new()));
        }

        let mut offset_values = Vec::with_capacity(indices.len());
        for index in indices {
            let prior_cleanup_steps = offset_values
                .iter()
                .flat_map(|offset: &CNativeValueMaterialization| offset.cleanup_after_use.clone())
                .collect::<Vec<_>>();
            let prior_cleanup = c_cleanup_sequence(&prior_cleanup_steps);
            let offset = self.materialize_native_value_result_operand(
                index,
                &format!("{prior_cleanup}{failure_cleanup}"),
            )?;
            offset_values.push(offset);
        }

        let offsets_array = self.next_native_name("value_offset_path_values");
        self.body.push(format!(
            "phpc_NativeValueHandle {offsets_array}[{}];",
            indices.len()
        ));
        for (index, offset) in offset_values.iter().enumerate() {
            self.body
                .push(format!("{offsets_array}[{index}] = {};", offset.handle));
        }
        Ok((offsets_array, indices.len(), offset_values))
    }

    fn emit_array_offset_null_coalesce_assignment(
        &mut self,
        target: &AssignTarget,
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<bool> {
        let AssignTarget::ArrayIndex {
            name,
            index: Some(index_expr),
            ..
        } = target
        else {
            return Ok(false);
        };

        let Some(CValue::ArrayHandle(handle)) = self.variables.get(name).cloned() else {
            return Ok(false);
        };

        self.emit_array_offset_null_coalesce_assignment_for_handle(
            name,
            &handle,
            index_expr,
            replacement_expr,
            span,
            None,
            "",
        )?;
        Ok(true)
    }

    fn materialize_array_offset_null_coalesce_assignment_expr(
        &mut self,
        target: &AssignTarget,
        replacement_expr: &Expr,
        span: Span,
        failure_cleanup: &str,
    ) -> CompileResult<Option<CNativeValueMaterialization>> {
        let AssignTarget::ArrayIndex {
            name,
            index: Some(index_expr),
            ..
        } = target
        else {
            return Ok(None);
        };

        let Some(CValue::ArrayHandle(handle)) = self.variables.get(name).cloned() else {
            return Ok(None);
        };

        self.emit_array_offset_null_coalesce_assignment_for_handle(
            name,
            &handle,
            index_expr,
            replacement_expr,
            span,
            Some("array_offset_null_coalesce_assign_result"),
            failure_cleanup,
        )
    }

    fn emit_array_offset_null_coalesce_assignment_for_handle(
        &mut self,
        name: &str,
        handle: &str,
        index_expr: &Expr,
        replacement_expr: &Expr,
        span: Span,
        result_prefix: Option<&str>,
        failure_cleanup: &str,
    ) -> CompileResult<Option<CNativeValueMaterialization>> {
        if native_conditional_rhs_needs_cleanup_boundary(replacement_expr) {
            return Err(self.unsupported(replacement_expr.span(), ASSEMBLY_CONDITIONAL_REJECTION));
        }

        self.uses_native_string_helpers = true;
        self.uses_native_array_helpers = true;
        self.uses_native_value_offset_mutation = true;

        let subject = self.materialize_native_array_c_value_handle(
            CValue::ArrayHandle(handle.to_string()),
            span,
        )?;
        let subject_cleanup = c_cleanup_sequence(&subject.cleanup_after_use);
        let offset_failure_cleanup = format!("{subject_cleanup}{failure_cleanup}");
        let offset =
            self.materialize_native_value_result_operand(index_expr, &offset_failure_cleanup)?;
        let mut operand_cleanup = offset.cleanup_after_use;
        operand_cleanup.extend(subject.cleanup_after_use);
        let operand_cleanup_sequence = c_cleanup_sequence(&operand_cleanup);

        let current_array = self.next_native_name("array_offset_null_coalesce_assign_array");
        let assigned_array =
            self.next_native_name("array_offset_null_coalesce_assign_written_array");
        self.body.push(format!(
            "phpc_NativeArrayHandle {current_array} = {handle};"
        ));
        self.body
            .push(format!("phpc_NativeArrayHandle {assigned_array} = {{0}};"));
        self.array_cleanup_handles.push(assigned_array.clone());

        let result = result_prefix.map(|prefix| self.next_native_name(prefix));
        if let Some(result) = &result {
            self.body
                .push(format!("phpc_NativeValueHandle {result} = {{0}};"));
        }

        let probe = self.next_native_name("array_offset_null_coalesce_assign_probe");
        let probe_diagnostic =
            self.next_native_name("array_offset_null_coalesce_assign_diagnostic");
        let bool_diagnostic =
            self.next_native_name("array_offset_null_coalesce_assign_bool_diagnostic");
        let present = self.next_native_name("array_offset_null_coalesce_assign_present");

        self.body.push(format!(
            "phpc_NativeDiagnosticHandle {probe_diagnostic} = {{0}};"
        ));
        self.body.push(format!(
            "phpc_NativeValueHandle {probe} = phpc_native_value_offset_operation_with_diagnostic({}, {}, {}, &{probe_diagnostic});",
            subject.handle,
            offset.handle,
            NativeStringOffsetOperation::Isset as u8
        ));
        let probe_error_exit = self.native_error_exit(&format!(
            "phpc_native_diagnostic_report({probe_diagnostic}); {operand_cleanup_sequence}{failure_cleanup}"
        ));
        self.body
            .push(format!("if ({probe}.ptr == NULL) {{ {probe_error_exit} }}"));
        self.body
            .push(format!("phpc_native_diagnostic_free({probe_diagnostic});"));
        self.body.push(format!(
            "phpc_NativeDiagnosticHandle {bool_diagnostic} = {{0}};"
        ));
        self.body.push(format!(
            "_Bool {present} = phpc_native_value_bool_with_diagnostic({probe}, &{bool_diagnostic});"
        ));
        let bool_error_exit = self.native_error_exit(&format!(
            "phpc_native_diagnostic_report({bool_diagnostic}); phpc_native_value_free({probe}); {operand_cleanup_sequence}{failure_cleanup}"
        ));
        self.body.push(format!(
            "if ({bool_diagnostic}.ptr != NULL) {{ {bool_error_exit} }}"
        ));
        self.body
            .push(format!("phpc_native_diagnostic_free({bool_diagnostic});"));
        self.body.push(format!("phpc_native_value_free({probe});"));

        if let Some(result) = &result {
            let read = self.next_native_name("array_offset_null_coalesce_assign_read");
            let read_diagnostic =
                self.next_native_name("array_offset_null_coalesce_assign_read_diagnostic");
            self.body.push(format!("if ({present}) {{"));
            self.body.push(format!(
                "phpc_NativeDiagnosticHandle {read_diagnostic} = {{0}};"
            ));
            self.body.push(format!(
                "phpc_NativeValueHandle {read} = phpc_native_value_offset_operation_with_diagnostic({}, {}, {}, &{read_diagnostic});",
                subject.handle,
                offset.handle,
                NativeStringOffsetOperation::Read as u8
            ));
            self.emit_report_native_diagnostic(&read_diagnostic);
            let read_error_exit = self.native_error_exit(&format!(
                "phpc_native_diagnostic_free({read_diagnostic}); {operand_cleanup_sequence}{failure_cleanup}"
            ));
            self.body
                .push(format!("if ({read}.ptr == NULL) {{ {read_error_exit} }}"));
            self.body
                .push(format!("phpc_native_diagnostic_free({read_diagnostic});"));
            self.body.push(format!("{result} = {read};"));
            self.body.push("} else {".to_string());
        } else {
            self.body.push(format!("if (!{present}) {{"));
        }

        let replacement_failure_cleanup = format!("{operand_cleanup_sequence}{failure_cleanup}");
        let replacement = self.materialize_native_value_result_operand(
            replacement_expr,
            &replacement_failure_cleanup,
        )?;
        let mutation = self.next_native_name("array_offset_null_coalesce_assign_value");
        let mutation_diagnostic =
            self.next_native_name("array_offset_null_coalesce_assign_write_diagnostic");
        self.body.push(format!(
            "phpc_NativeDiagnosticHandle {mutation_diagnostic} = {{0}};"
        ));
        self.body.push(format!(
            "phpc_NativeValueHandle {mutation} = phpc_native_value_offset_mutation_operation_with_diagnostic({}, {}, {}, {NATIVE_VALUE_OFFSET_MUTATION_WRITE}, &{mutation_diagnostic});",
            subject.handle,
            offset.handle,
            replacement.handle
        ));
        self.body.push(format!(
            "if ({mutation_diagnostic}.ptr != NULL) {{ phpc_native_diagnostic_report({mutation_diagnostic}); }}"
        ));
        let mutation_failure_cleanup = format!(
            "{}{operand_cleanup_sequence}{failure_cleanup}",
            c_cleanup_sequence(&replacement.cleanup_after_use)
        );
        let mutation_error_exit = self.native_error_exit(&mutation_failure_cleanup);
        self.body.push(format!(
            "if ({mutation}.ptr == NULL) {{ {mutation_error_exit} }}"
        ));
        self.body.push(format!(
            "{assigned_array} = phpc_native_value_array_clone({mutation});"
        ));
        let array_error_exit = self.native_error_exit(&format!(
            "phpc_native_value_free({mutation}); {mutation_failure_cleanup}"
        ));
        self.body.push(format!(
            "if ({assigned_array}.ptr == NULL) {{ {array_error_exit} }}"
        ));
        self.body
            .push(format!("phpc_native_value_free({mutation});"));
        if let Some(result) = &result {
            self.uses_native_value_clone = true;
            let assigned_result =
                self.next_native_name("array_offset_null_coalesce_assign_assigned_value");
            self.body.push(format!(
                "phpc_NativeValueHandle {assigned_result} = phpc_native_value_clone({});",
                replacement.handle
            ));
            self.body.push(format!("{result} = {assigned_result};"));
        }
        self.body
            .push(format!("{current_array} = {assigned_array};"));
        self.body.extend(replacement.cleanup_after_use);
        self.body.push("}".to_string());
        self.body.extend(operand_cleanup);
        self.variables
            .insert(name.to_string(), CValue::ArrayHandle(current_array));

        Ok(result.map(|handle| CNativeValueMaterialization {
            handle: handle.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({handle});")],
        }))
    }

    fn emit_array_lvalue_null_coalesce_assignment(
        &mut self,
        target: &AssignTarget,
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<bool> {
        let Some((handle, indices, target_span)) =
            self.native_array_lvalue_key_target_parts(target)
        else {
            return Ok(false);
        };

        let path = self.materialize_native_array_lvalue_key_path(&indices, target_span, "")?;
        self.emit_array_lvalue_null_coalesce_assignment_for_handle(
            &handle,
            path,
            replacement_expr,
            span,
            None,
            "",
        )?;
        Ok(true)
    }

    fn materialize_array_lvalue_null_coalesce_assignment_expr(
        &mut self,
        target: &AssignTarget,
        replacement_expr: &Expr,
        span: Span,
        failure_cleanup: &str,
    ) -> CompileResult<Option<CNativeValueMaterialization>> {
        let Some((handle, indices, target_span)) =
            self.native_array_lvalue_key_target_parts(target)
        else {
            return Ok(None);
        };

        let path =
            self.materialize_native_array_lvalue_key_path(&indices, target_span, failure_cleanup)?;
        self.emit_array_lvalue_null_coalesce_assignment_for_handle(
            &handle,
            path,
            replacement_expr,
            span,
            Some("array_lvalue_null_coalesce_assign_result"),
            failure_cleanup,
        )
        .map(Some)
    }

    fn emit_array_lvalue_null_coalesce_assignment_for_handle(
        &mut self,
        handle: &str,
        path: CNativeArrayLvaluePath,
        replacement_expr: &Expr,
        _span: Span,
        result_prefix: Option<&str>,
        failure_cleanup: &str,
    ) -> CompileResult<CNativeValueMaterialization> {
        if native_conditional_rhs_needs_cleanup_boundary(replacement_expr) {
            return Err(self.unsupported(replacement_expr.span(), ASSEMBLY_CONDITIONAL_REJECTION));
        }

        self.uses_native_string_helpers = true;
        self.uses_native_array_helpers = true;
        self.uses_native_array_lvalue_helpers = true;

        let path_cleanup = c_cleanup_sequence(&path.cleanup_after_use);
        let result = result_prefix.map(|prefix| self.next_native_name(prefix));
        if let Some(result) = &result {
            self.body
                .push(format!("phpc_NativeValueHandle {result} = {{0}};"));
        }

        let probe_owner = self.next_native_name("array_lvalue_null_coalesce_assign_probe_owner");
        let probe_result = self.next_native_name("array_lvalue_null_coalesce_assign_probe_result");
        let probe_value = self.next_native_name("array_lvalue_null_coalesce_assign_probe_value");
        let bool_diagnostic =
            self.next_native_name("array_lvalue_null_coalesce_assign_bool_diagnostic");
        let present = self.next_native_name("array_lvalue_null_coalesce_assign_present");

        self.body.push(format!(
            "phpc_NativeArrayLvalueOwner {probe_owner} = phpc_native_array_lvalue_owner_array({handle});"
        ));
        self.body.push(format!(
            "phpc_NativeArrayLvalueResult {probe_result} = phpc_native_array_lvalue_owner_value_operation_result({probe_owner}, {}, {}, PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_ISSET, 0, 0, 0, (phpc_NativeValueHandle){{0}});",
            path.path, path.len
        ));
        self.emit_native_array_lvalue_result_check(
            &probe_result,
            &format!("{path_cleanup}{failure_cleanup}"),
        );
        self.body.push(format!(
            "phpc_NativeValueHandle {probe_value} = {probe_result}.value;"
        ));
        self.body.push(format!(
            "{probe_result}.value = (phpc_NativeValueHandle){{0}};"
        ));
        self.body.push(format!(
            "phpc_native_array_lvalue_result_free({probe_result});"
        ));
        self.body.push(format!(
            "phpc_NativeDiagnosticHandle {bool_diagnostic} = {{0}};"
        ));
        self.body.push(format!(
            "_Bool {present} = phpc_native_value_bool_with_diagnostic({probe_value}, &{bool_diagnostic});"
        ));
        let bool_error_exit = self.native_error_exit(&format!(
            "phpc_native_diagnostic_report({bool_diagnostic}); phpc_native_value_free({probe_value}); {path_cleanup}{failure_cleanup}"
        ));
        self.body.push(format!(
            "if ({bool_diagnostic}.ptr != NULL) {{ {bool_error_exit} }}"
        ));
        self.body
            .push(format!("phpc_native_diagnostic_free({bool_diagnostic});"));
        self.body
            .push(format!("phpc_native_value_free({probe_value});"));

        if let Some(result) = &result {
            let read_owner = self.next_native_name("array_lvalue_null_coalesce_assign_read_owner");
            let read_result =
                self.next_native_name("array_lvalue_null_coalesce_assign_read_result");
            let read_value = self.next_native_name("array_lvalue_null_coalesce_assign_read_value");
            self.body.push(format!("if ({present}) {{"));
            self.body.push(format!(
                "phpc_NativeArrayLvalueOwner {read_owner} = phpc_native_array_lvalue_owner_array({handle});"
            ));
            self.body.push(format!(
                "phpc_NativeArrayLvalueResult {read_result} = phpc_native_array_lvalue_owner_value_operation_result({read_owner}, {}, {}, PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ, 0, 0, 0, (phpc_NativeValueHandle){{0}});",
                path.path, path.len
            ));
            self.emit_native_array_lvalue_result_check(
                &read_result,
                &format!("{path_cleanup}{failure_cleanup}"),
            );
            self.body.push(format!(
                "phpc_NativeValueHandle {read_value} = {read_result}.value;"
            ));
            self.body.push(format!(
                "{read_result}.value = (phpc_NativeValueHandle){{0}};"
            ));
            self.body.push(format!(
                "phpc_native_array_lvalue_result_free({read_result});"
            ));
            self.body.push(format!("{result} = {read_value};"));
            self.body.push("} else {".to_string());
        } else {
            self.body.push(format!("if (!{present}) {{"));
        }

        let replacement_failure_cleanup = format!("{path_cleanup}{failure_cleanup}");
        let replacement = self.materialize_native_value_result_operand(
            replacement_expr,
            &replacement_failure_cleanup,
        )?;
        let write_owner = self.next_native_name("array_lvalue_null_coalesce_assign_write_owner");
        let write_result = self.next_native_name("array_lvalue_null_coalesce_assign_write_result");
        self.body.push(format!(
            "phpc_NativeArrayLvalueOwner {write_owner} = phpc_native_array_lvalue_owner_array({handle});"
        ));
        self.body.push(format!(
            "phpc_NativeArrayLvalueResult {write_result} = phpc_native_array_lvalue_owner_value_operation_result({write_owner}, {}, {}, PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE, 0, 0, 0, {});",
            path.path, path.len, replacement.handle
        ));
        self.emit_native_array_lvalue_result_check(
            &write_result,
            &format!(
                "{}{path_cleanup}{failure_cleanup}",
                c_cleanup_sequence(&replacement.cleanup_after_use)
            ),
        );
        self.body.push(format!(
            "phpc_native_array_lvalue_result_free({write_result});"
        ));
        if let Some(result) = &result {
            self.uses_native_value_clone = true;
            let assigned_result =
                self.next_native_name("array_lvalue_null_coalesce_assign_assigned_value");
            self.body.push(format!(
                "phpc_NativeValueHandle {assigned_result} = phpc_native_value_clone({});",
                replacement.handle
            ));
            self.body.push(format!("{result} = {assigned_result};"));
        }
        self.body.extend(replacement.cleanup_after_use);
        self.body.push("}".to_string());
        self.body.extend(path.cleanup_after_use);

        Ok(match result {
            Some(handle) => CNativeValueMaterialization {
                handle: handle.clone(),
                cleanup_after_use: vec![format!("phpc_native_value_free({handle});")],
            },
            None => CNativeValueMaterialization {
                handle: "(phpc_NativeValueHandle){0}".to_string(),
                cleanup_after_use: Vec::new(),
            },
        })
    }

    fn materialize_native_array_lvalue_key_path(
        &mut self,
        indices: &[&Expr],
        span: Span,
        failure_cleanup: &str,
    ) -> CompileResult<CNativeArrayLvaluePath> {
        self.materialize_native_array_lvalue_path(indices, false, &[], span, failure_cleanup)
    }

    fn materialize_native_array_lvalue_append_path(
        &mut self,
        prefix_indices: &[&Expr],
        suffix_indices: &[&Expr],
        span: Span,
        failure_cleanup: &str,
    ) -> CompileResult<CNativeArrayLvaluePath> {
        self.materialize_native_array_lvalue_path(
            prefix_indices,
            true,
            suffix_indices,
            span,
            failure_cleanup,
        )
    }

    fn materialize_native_array_lvalue_path(
        &mut self,
        prefix_indices: &[&Expr],
        include_append: bool,
        suffix_indices: &[&Expr],
        span: Span,
        failure_cleanup: &str,
    ) -> CompileResult<CNativeArrayLvaluePath> {
        if prefix_indices.is_empty() && !include_append && suffix_indices.is_empty() {
            return Err(self.unsupported(span, ASSEMBLY_ARRAY_REJECTION));
        }

        let mut keys: Vec<CNativeValueMaterialization> = Vec::new();
        let mut initializers = Vec::new();

        for index in prefix_indices {
            let prior_cleanup = c_cleanup_sequence(
                &keys
                    .iter()
                    .flat_map(|key: &CNativeValueMaterialization| key.cleanup_after_use.clone())
                    .collect::<Vec<_>>(),
            );
            let key = self.materialize_native_value_result_operand(
                index,
                &format!("{prior_cleanup}{failure_cleanup}"),
            )?;
            initializers.push(format!("{{ PHPC_NATIVE_ARRAY_PATH_KEY, {} }}", key.handle));
            keys.push(key);
        }

        if include_append {
            initializers
                .push("{ PHPC_NATIVE_ARRAY_PATH_APPEND, (phpc_NativeValueHandle){0} }".to_string());
        }

        for index in suffix_indices {
            let prior_cleanup = c_cleanup_sequence(
                &keys
                    .iter()
                    .flat_map(|key| key.cleanup_after_use.clone())
                    .collect::<Vec<_>>(),
            );
            let key = self.materialize_native_value_result_operand(
                index,
                &format!("{prior_cleanup}{failure_cleanup}"),
            )?;
            initializers.push(format!("{{ PHPC_NATIVE_ARRAY_PATH_KEY, {} }}", key.handle));
            keys.push(key);
        }

        let path = self.next_native_name("array_lvalue_path");
        self.body.push(format!(
            "phpc_NativeArrayPathSegment {path}[{}] = {{",
            initializers.len()
        ));
        for initializer in initializers {
            self.body.push(format!("{initializer},"));
        }
        self.body.push("};".to_string());

        Ok(CNativeArrayLvaluePath {
            path,
            len: prefix_indices.len() + usize::from(include_append) + suffix_indices.len(),
            cleanup_after_use: keys
                .into_iter()
                .flat_map(|key| key.cleanup_after_use)
                .collect(),
        })
    }

    fn emit_array_lvalue_unset_for_handle(
        &mut self,
        handle: &str,
        indices: &[&Expr],
        span: Span,
    ) -> CompileResult<()> {
        self.uses_native_string_helpers = true;
        self.uses_native_array_helpers = true;
        self.uses_native_array_lvalue_helpers = true;

        let path = self.materialize_native_array_lvalue_key_path(indices, span, "")?;
        let owner = self.next_native_name("array_lvalue_owner");
        let result = self.next_native_name("array_lvalue_unset_result");
        self.body.push(format!(
            "phpc_NativeArrayLvalueOwner {owner} = phpc_native_array_lvalue_owner_array({handle});"
        ));
        self.body.push(format!(
            "phpc_NativeArrayLvalueResult {result} = phpc_native_array_lvalue_owner_value_operation_result({owner}, {}, {}, PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UNSET, 0, 0, 0, (phpc_NativeValueHandle){{0}});",
            path.path, path.len
        ));
        let cleanup = c_cleanup_sequence(&path.cleanup_after_use);
        self.emit_native_array_lvalue_result_check(&result, &cleanup);
        self.body
            .push(format!("phpc_native_array_lvalue_result_free({result});"));
        self.body.extend(path.cleanup_after_use);
        Ok(())
    }

    fn emit_native_array_foreach_statement(
        &mut self,
        iterable: &Expr,
        key: Option<&str>,
        value: &str,
        by_reference: bool,
        body: &[Stmt],
        span: Span,
    ) -> CompileResult<()> {
        if by_reference {
            return Err(
                self.unsupported(span, ASSEMBLY_NATIVE_ARRAY_BY_REFERENCE_FOREACH_REJECTION)
            );
        }
        if key == Some(value) {
            return Err(self.unsupported(span, ASSEMBLY_ARRAY_REJECTION));
        }
        if key
            .map(|name| self.native_foreach_symbol_target_has_prior_storage(name))
            .unwrap_or(false)
            || self.native_foreach_symbol_target_has_prior_storage(value)
        {
            return Err(self.unsupported(span, ASSEMBLY_ARRAY_REJECTION));
        }
        if native_foreach_body_may_mutate_storage(body) {
            return Err(self.unsupported(span, ASSEMBLY_MUTATION_REJECTION));
        }

        let foreach_iterable = self.materialize_native_array_foreach_iterable(iterable, span)?;
        let iterable_handle = foreach_iterable.handle.clone();
        let len = self.next_native_name("array_foreach_len");
        let index = self.next_native_name("array_foreach_index");

        self.retain_native_value_cleanup_handle(&iterable_handle);
        self.body.push(format!(
            "size_t {len} = phpc_native_array_foreach_iterable_len({iterable_handle});"
        ));
        self.body.push(format!(
            "for (size_t {index} = 0; {index} < {len}; ++{index}) {{"
        ));

        let key_handle = if let Some(key) = key {
            let key_handle =
                self.emit_native_array_foreach_cursor_value(&iterable_handle, &index, "key");
            self.retain_native_value_cleanup_handle(&key_handle);
            self.variables.insert(
                key.to_string(),
                CValue::NativeValueHandle(key_handle.clone()),
            );
            Some((key.to_string(), key_handle))
        } else {
            None
        };
        let value_handle =
            self.emit_native_array_foreach_cursor_value(&iterable_handle, &index, "value");
        self.retain_native_value_cleanup_handle(&value_handle);
        self.variables.insert(
            value.to_string(),
            CValue::NativeValueHandle(value_handle.clone()),
        );

        for statement in body {
            self.emit_statement(statement)?;
        }

        self.variables.remove(value);
        self.release_native_value_cleanup_handle(&value_handle);
        self.body
            .push(format!("phpc_native_value_free({value_handle});"));
        if let Some((key, key_handle)) = key_handle {
            self.variables.remove(&key);
            self.release_native_value_cleanup_handle(&key_handle);
            self.body
                .push(format!("phpc_native_value_free({key_handle});"));
        }
        self.body.push("}".to_string());
        self.release_native_value_cleanup_handle(&iterable_handle);
        self.body.extend(foreach_iterable.cleanup_after_use);
        Ok(())
    }

    fn materialize_native_array_foreach_iterable(
        &mut self,
        iterable: &Expr,
        span: Span,
    ) -> CompileResult<CNativeValueMaterialization> {
        self.uses_native_string_helpers = true;
        self.uses_native_array_helpers = true;
        self.uses_native_array_lvalue_helpers = true;

        if let Some((handle, indices, lvalue_span)) =
            self.native_array_foreach_lvalue_parts(iterable)
        {
            return self.materialize_native_array_foreach_iterable_for_handle(
                &handle,
                &indices,
                lvalue_span,
                Vec::new(),
            );
        }

        if let Expr::Array { items, span } = iterable {
            let handle = self.emit_array_literal(items, *span)?;
            self.release_native_array_cleanup_handle(&handle);
            return self.materialize_native_array_foreach_iterable_for_handle(
                &handle,
                &[],
                *span,
                vec![format!("phpc_native_array_free({handle});")],
            );
        }

        Err(self.unsupported(span, ASSEMBLY_ARRAY_REJECTION))
    }

    fn materialize_native_array_foreach_iterable_for_handle(
        &mut self,
        handle: &str,
        indices: &[&Expr],
        span: Span,
        owner_cleanup_after_use: Vec<String>,
    ) -> CompileResult<CNativeValueMaterialization> {
        let (path_arg, path_len, mut path_cleanup) = if indices.is_empty() {
            ("NULL".to_string(), 0, Vec::new())
        } else {
            let path = self.materialize_native_array_lvalue_key_path(
                indices,
                span,
                &c_cleanup_sequence(&owner_cleanup_after_use),
            )?;
            (path.path, path.len, path.cleanup_after_use)
        };

        let owner = self.next_native_name("array_foreach_owner");
        let result = self.next_native_name("array_foreach_iterable_result");
        let value = self.next_native_name("array_foreach_iterable");
        self.body.push(format!(
            "phpc_NativeArrayLvalueOwner {owner} = phpc_native_array_lvalue_owner_array({handle});"
        ));
        self.body.push(format!(
            "phpc_NativeArrayLvalueResult {result} = phpc_native_array_lvalue_owner_foreach_iterable_result({owner}, {path_arg}, {path_len});"
        ));
        let cleanup = format!(
            "{}{}",
            c_cleanup_sequence(&path_cleanup),
            c_cleanup_sequence(&owner_cleanup_after_use)
        );
        self.emit_native_array_lvalue_result_check(&result, &cleanup);
        self.body
            .push(format!("phpc_NativeValueHandle {value} = {result}.value;"));
        self.body
            .push(format!("{result}.value = (phpc_NativeValueHandle){{0}};"));
        self.body
            .push(format!("phpc_native_array_lvalue_result_free({result});"));
        self.body.append(&mut path_cleanup);
        self.body.extend(owner_cleanup_after_use);

        Ok(CNativeValueMaterialization {
            handle: value.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({value});")],
        })
    }

    fn native_array_foreach_lvalue_parts<'a>(
        &self,
        iterable: &'a Expr,
    ) -> Option<(String, Vec<&'a Expr>, Span)> {
        match iterable {
            Expr::Variable(name, span) => match self.variables.get(name) {
                Some(CValue::ArrayHandle(handle)) => Some((handle.clone(), Vec::new(), *span)),
                _ => None,
            },
            Expr::Index {
                target,
                index,
                span,
            } => {
                let (handle, mut indices, _) = self.native_array_foreach_lvalue_parts(target)?;
                indices.push(index);
                Some((handle, indices, *span))
            }
            _ => None,
        }
    }

    fn materialize_native_array_pointer_call(
        &mut self,
        builtin: NativeArrayPointerBuiltin,
        args: &[Expr],
        span: Span,
        failure_cleanup: &str,
    ) -> CompileResult<CNativeValueMaterialization> {
        self.uses_native_string_helpers = true;
        self.uses_native_array_helpers = true;
        self.uses_native_array_lvalue_helpers = true;

        let [argument] = args else {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        };
        let Some((handle, indices, lvalue_span)) = self.native_array_foreach_lvalue_parts(argument)
        else {
            return Err(self.unsupported(argument.span(), ASSEMBLY_ARRAY_REJECTION));
        };

        let (path_arg, path_len, mut path_cleanup) = if indices.is_empty() {
            ("NULL".to_string(), 0, Vec::new())
        } else {
            let path = self.materialize_native_array_lvalue_key_path(
                &indices,
                lvalue_span,
                failure_cleanup,
            )?;
            (path.path, path.len, path.cleanup_after_use)
        };

        let owner = self.next_native_name("array_pointer_owner");
        let result = self.next_native_name("array_pointer_result");
        let value = self.next_native_name("array_pointer_value");
        let operation = builtin.operation_tag();
        self.body.push(format!(
            "phpc_NativeArrayLvalueOwner {owner} = phpc_native_array_lvalue_owner_array({handle});"
        ));
        self.body.push(format!(
            "phpc_NativeArrayLvalueResult {result} = phpc_native_array_lvalue_owner_pointer_result({owner}, {path_arg}, {path_len}, {operation});"
        ));
        let cleanup = format!("{}{}", c_cleanup_sequence(&path_cleanup), failure_cleanup);
        self.emit_native_array_lvalue_result_check(&result, &cleanup);
        self.body
            .push(format!("phpc_NativeValueHandle {value} = {result}.value;"));
        self.body
            .push(format!("{result}.value = (phpc_NativeValueHandle){{0}};"));
        self.body
            .push(format!("phpc_native_array_lvalue_result_free({result});"));
        self.body.append(&mut path_cleanup);

        Ok(CNativeValueMaterialization {
            handle: value.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({value});")],
        })
    }

    fn emit_native_array_sort_call(
        &mut self,
        builtin: NativeArraySortBuiltin,
        args: &[Expr],
        span: Span,
        failure_cleanup: &str,
    ) -> CompileResult<CValue> {
        self.uses_native_string_helpers = true;
        self.uses_native_array_helpers = true;
        self.uses_native_array_lvalue_helpers = true;

        let Some((argument, operand_exprs)) = args.split_first() else {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        };
        let Some((handle, indices, lvalue_span)) = self.native_array_foreach_lvalue_parts(argument)
        else {
            return Err(self.unsupported(argument.span(), ASSEMBLY_ARRAY_REJECTION));
        };

        let (path_arg, path_len, path_cleanup) = if indices.is_empty() {
            ("NULL".to_string(), 0, Vec::new())
        } else {
            let path = self.materialize_native_array_lvalue_key_path(
                &indices,
                lvalue_span,
                failure_cleanup,
            )?;
            (path.path, path.len, path.cleanup_after_use)
        };

        let mut operands = Vec::new();
        let mut operand_cleanup = Vec::new();
        for operand in operand_exprs {
            let operand_failure_cleanup = format!(
                "{}{}{}",
                c_cleanup_sequence(&operand_cleanup),
                c_cleanup_sequence(&path_cleanup),
                failure_cleanup
            );
            let materialized =
                self.materialize_native_value_result_operand(operand, &operand_failure_cleanup)?;
            operand_cleanup.extend(materialized.cleanup_after_use.clone());
            operands.push(materialized);
        }

        let (operands_arg, operand_count) = if operands.is_empty() {
            ("NULL".to_string(), 0)
        } else {
            let operand_array = self.next_native_name("array_sort_operands");
            let handles = operands
                .iter()
                .map(|operand| operand.handle.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            self.body.push(format!(
                "phpc_NativeValueHandle {operand_array}[] = {{ {handles} }};"
            ));
            (operand_array, operands.len())
        };

        let owner = self.next_native_name("array_sort_owner");
        let result = self.next_native_name("array_sort_result");
        let operation = builtin.operation_tag();
        self.body.push(format!(
            "phpc_NativeArrayLvalueOwner {owner} = phpc_native_array_lvalue_owner_array({handle});"
        ));
        self.body.push(format!(
            "phpc_NativeArrayLvalueResult {result} = phpc_native_array_lvalue_owner_sort_result({owner}, {path_arg}, {path_len}, {operation}, {operands_arg}, {operand_count});"
        ));
        let cleanup = format!(
            "{}{}{}",
            c_cleanup_sequence(&operand_cleanup),
            c_cleanup_sequence(&path_cleanup),
            failure_cleanup
        );
        self.emit_native_array_lvalue_result_check(&result, &cleanup);
        self.body
            .push(format!("phpc_native_array_lvalue_result_free({result});"));
        self.body.extend(operand_cleanup);
        self.body.extend(path_cleanup);

        Ok(CValue::Bool(true))
    }

    fn emit_native_value_array_callback_call(
        &mut self,
        builtin: NativeValueArrayCallbackBuiltin,
        args: &[Expr],
        span: Span,
        failure_cleanup: &str,
    ) -> CompileResult<CNativeValueMaterialization> {
        self.uses_native_string_helpers = true;
        self.uses_native_array_helpers = true;
        self.uses_native_array_lvalue_helpers = true;

        if args.is_empty() {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let mut operands = Vec::new();
        let mut operand_cleanup = Vec::new();
        for operand in args {
            let operand_failure_cleanup = format!(
                "{}{}",
                c_cleanup_sequence(&operand_cleanup),
                failure_cleanup
            );
            let materialized =
                self.materialize_native_value_result_operand(operand, &operand_failure_cleanup)?;
            operand_cleanup.extend(materialized.cleanup_after_use.clone());
            operands.push(materialized);
        }

        let operand_array = self.next_native_name("native_value_array_callback_args");
        let handles = operands
            .iter()
            .map(|operand| operand.handle.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        self.body.push(format!(
            "phpc_NativeValueHandle {operand_array}[] = {{ {handles} }};"
        ));

        let result = self.next_native_name("native_value_array_callback_result");
        let value = self.next_native_name("native_value_array_callback");
        let operation = builtin.operation_tag();
        self.body.push(format!(
            "phpc_NativeArrayLvalueResult {result} = phpc_native_value_array_callback_result({operation}, {operand_array}, {});",
            operands.len()
        ));
        let cleanup = format!(
            "{}{}",
            c_cleanup_sequence(&operand_cleanup),
            failure_cleanup
        );
        self.emit_native_array_lvalue_result_check(&result, &cleanup);
        self.body
            .push(format!("phpc_NativeValueHandle {value} = {result}.value;"));
        self.body
            .push(format!("{result}.value = (phpc_NativeValueHandle){{0}};"));
        self.body
            .push(format!("phpc_native_array_lvalue_result_free({result});"));
        self.body.extend(operand_cleanup);

        Ok(CNativeValueMaterialization {
            handle: value.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({value});")],
        })
    }

    fn emit_native_array_foreach_cursor_value(
        &mut self,
        iterable: &str,
        index: &str,
        cursor: &str,
    ) -> String {
        let helper = match cursor {
            "key" => "phpc_native_array_foreach_iterable_key_result",
            _ => "phpc_native_array_foreach_iterable_value_result",
        };
        let result = self.next_native_name(&format!("array_foreach_{cursor}_result"));
        let cursor_value = self.next_native_name(&format!("array_foreach_{cursor}_value"));
        self.body.push(format!(
            "phpc_NativeArrayLvalueResult {result} = {helper}({iterable}, {index});"
        ));
        self.emit_native_array_lvalue_result_check(&result, "");
        self.body.push(format!(
            "phpc_NativeValueHandle {cursor_value} = {result}.value;"
        ));
        self.body
            .push(format!("{result}.value = (phpc_NativeValueHandle){{0}};"));
        self.body
            .push(format!("phpc_native_array_lvalue_result_free({result});"));
        cursor_value
    }

    fn native_foreach_symbol_target_has_prior_storage(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    fn emit_array_lvalue_write_for_handle(
        &mut self,
        handle: &str,
        indices: &[&Expr],
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<()> {
        self.uses_native_string_helpers = true;
        self.uses_native_array_helpers = true;
        self.uses_native_array_lvalue_helpers = true;

        let path = self.materialize_native_array_lvalue_key_path(indices, span, "")?;
        let path_cleanup = c_cleanup_sequence(&path.cleanup_after_use);
        let replacement =
            self.materialize_native_value_result_operand(replacement_expr, &path_cleanup)?;
        self.emit_array_lvalue_write_materialized_for_handle(
            handle,
            path,
            replacement,
            "array_lvalue_write_result",
        )
    }

    fn emit_array_lvalue_write_materialized_for_handle(
        &mut self,
        handle: &str,
        path: CNativeArrayLvaluePath,
        replacement: CNativeValueMaterialization,
        result_prefix: &str,
    ) -> CompileResult<()> {
        self.uses_native_string_helpers = true;
        self.uses_native_array_helpers = true;
        self.uses_native_array_lvalue_helpers = true;

        let owner = self.next_native_name("array_lvalue_owner");
        let result = self.next_native_name(result_prefix);
        self.body.push(format!(
            "phpc_NativeArrayLvalueOwner {owner} = phpc_native_array_lvalue_owner_array({handle});"
        ));
        self.body.push(format!(
            "phpc_NativeArrayLvalueResult {result} = phpc_native_array_lvalue_owner_value_operation_result({owner}, {}, {}, PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE, 0, 0, 0, {});",
            path.path, path.len, replacement.handle
        ));
        let path_cleanup = c_cleanup_sequence(&path.cleanup_after_use);
        let cleanup = format!(
            "{}{}",
            c_cleanup_sequence(&replacement.cleanup_after_use),
            path_cleanup
        );
        self.emit_native_array_lvalue_result_check(&result, &cleanup);
        self.body
            .push(format!("phpc_native_array_lvalue_result_free({result});"));
        self.body.extend(replacement.cleanup_after_use);
        self.body.extend(path.cleanup_after_use);
        Ok(())
    }

    fn emit_array_lvalue_read_materialized_for_handle(
        &mut self,
        handle: &str,
        path: CNativeArrayLvaluePath,
        result_prefix: &str,
        failure_cleanup: &str,
    ) -> CNativeValueMaterialization {
        self.uses_native_string_helpers = true;
        self.uses_native_array_helpers = true;
        self.uses_native_array_lvalue_helpers = true;

        let owner = self.next_native_name("array_lvalue_owner");
        let result = self.next_native_name(result_prefix);
        let value = self.next_native_name("array_lvalue_read_value");
        self.body.push(format!(
            "phpc_NativeArrayLvalueOwner {owner} = phpc_native_array_lvalue_owner_array({handle});"
        ));
        self.body.push(format!(
            "phpc_NativeArrayLvalueResult {result} = phpc_native_array_lvalue_owner_value_operation_result({owner}, {}, {}, PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ, 0, 0, 0, (phpc_NativeValueHandle){{0}});",
            path.path, path.len
        ));
        let path_cleanup = c_cleanup_sequence(&path.cleanup_after_use);
        let cleanup = format!("{path_cleanup}{failure_cleanup}");
        self.emit_native_array_lvalue_result_check(&result, &cleanup);
        self.body
            .push(format!("phpc_NativeValueHandle {value} = {result}.value;"));
        self.body
            .push(format!("{result}.value = (phpc_NativeValueHandle){{0}};"));
        self.body
            .push(format!("phpc_native_array_lvalue_result_free({result});"));
        self.body.extend(path.cleanup_after_use);

        CNativeValueMaterialization {
            handle: value.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({value});")],
        }
    }

    fn emit_array_lvalue_append_write_for_handle(
        &mut self,
        handle: &str,
        prefix_indices: &[&Expr],
        suffix_indices: &[&Expr],
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<()> {
        self.uses_native_string_helpers = true;
        self.uses_native_array_helpers = true;
        self.uses_native_array_lvalue_helpers = true;

        let path = self.materialize_native_array_lvalue_append_path(
            prefix_indices,
            suffix_indices,
            span,
            "",
        )?;
        let path_cleanup = c_cleanup_sequence(&path.cleanup_after_use);
        let replacement =
            self.materialize_native_value_result_operand(replacement_expr, &path_cleanup)?;
        self.emit_array_lvalue_write_materialized_for_handle(
            handle,
            path,
            replacement,
            "array_lvalue_append_write_result",
        )
    }

    fn emit_unset_array_index(
        &mut self,
        name: &str,
        index_expr: &Expr,
        span: Span,
    ) -> CompileResult<()> {
        if !self.uses_native_string_helpers {
            return Err(self.unsupported(span, ASSEMBLY_ARRAY_REJECTION));
        }

        match self.variables.get(name).cloned() {
            Some(CValue::ArrayHandle(handle)) => {
                self.emit_array_lvalue_unset_for_handle(&handle, &[index_expr], span)
            }
            Some(CValue::String(_) | CValue::StringExpr(_)) => {
                Err(self.unsupported(span, ASSEMBLY_ARRAY_REJECTION))
            }
            Some(subject) => self.emit_value_offset_path_unset_statement(
                name,
                subject,
                std::slice::from_ref(index_expr),
                span,
            ),
            None => Err(self.unsupported(span, ASSEMBLY_ARRAY_REJECTION)),
        }
    }

    fn emit_unset_nested_array_index(
        &mut self,
        name: &str,
        indices: &[Expr],
        span: Span,
    ) -> CompileResult<()> {
        if !self.uses_native_string_helpers {
            return Err(self.unsupported(span, ASSEMBLY_ARRAY_REJECTION));
        }

        match self.variables.get(name).cloned() {
            Some(CValue::ArrayHandle(handle)) => {
                let indices = indices.iter().collect::<Vec<_>>();
                self.emit_array_lvalue_unset_for_handle(&handle, &indices, span)
            }
            Some(CValue::String(_) | CValue::StringExpr(_)) => {
                Err(self.unsupported(span, ASSEMBLY_ARRAY_REJECTION))
            }
            Some(subject) => {
                self.emit_value_offset_path_unset_statement(name, subject, indices, span)
            }
            None => Err(self.unsupported(span, ASSEMBLY_ARRAY_REJECTION)),
        }
    }

    fn emit_unset_many(&mut self, targets: &[UnsetTarget], span: Span) -> CompileResult<()> {
        if targets.is_empty() {
            return Err(self.unsupported(span, ASSEMBLY_MUTATION_REJECTION));
        }

        for target in targets {
            match target {
                UnsetTarget::ArrayIndex { name, index, span } => {
                    self.emit_unset_array_index(name, index, *span)?;
                }
                UnsetTarget::NestedArrayIndex {
                    name,
                    indices,
                    span,
                } => {
                    self.emit_unset_nested_array_index(name, indices, *span)?;
                }
                _ => return Err(self.unsupported(span, ASSEMBLY_MUTATION_REJECTION)),
            }
        }

        Ok(())
    }

    fn emit_array_offset_mutation(
        &mut self,
        name: &str,
        handle: &str,
        index_expr: Option<&Expr>,
        replacement_expr: Option<&Expr>,
        operation: u8,
        temp_prefix: &str,
        span: Span,
    ) -> CompileResult<()> {
        let subject = self.materialize_native_array_c_value_handle(
            CValue::ArrayHandle(handle.to_string()),
            span,
        )?;
        let subject_cleanup = c_cleanup_sequence(&subject.cleanup_after_use);
        let offset = if let Some(index_expr) = index_expr {
            self.materialize_native_value_result_operand(index_expr, &subject_cleanup)?
        } else {
            CNativeValueMaterialization {
                handle: "(phpc_NativeValueHandle){0}".to_string(),
                cleanup_after_use: Vec::new(),
            }
        };
        let offset_cleanup = c_cleanup_sequence(&offset.cleanup_after_use);
        let replacement_failure_cleanup = format!("{offset_cleanup}{subject_cleanup}");
        let replacement = if let Some(replacement_expr) = replacement_expr {
            self.materialize_native_value_result_operand(
                replacement_expr,
                &replacement_failure_cleanup,
            )?
        } else {
            CNativeValueMaterialization {
                handle: "(phpc_NativeValueHandle){0}".to_string(),
                cleanup_after_use: Vec::new(),
            }
        };

        self.emit_array_offset_mutation_from_materialized(
            name,
            subject,
            offset,
            replacement,
            operation,
            temp_prefix,
        )
    }

    fn emit_array_offset_mutation_assignment_expr(
        &mut self,
        target: &AssignTarget,
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<Option<CValue>> {
        if !self.uses_native_string_helpers {
            return Ok(None);
        }

        let AssignTarget::ArrayIndex { name, index, .. } = target else {
            return Ok(None);
        };

        let Some(CValue::ArrayHandle(handle)) = self.variables.get(name).cloned() else {
            return Ok(None);
        };

        let subject =
            self.materialize_native_array_c_value_handle(CValue::ArrayHandle(handle), span)?;
        let subject_cleanup = c_cleanup_sequence(&subject.cleanup_after_use);
        let offset = if let Some(index_expr) = index.as_ref() {
            self.materialize_native_value_result_operand(index_expr, &subject_cleanup)?
        } else {
            CNativeValueMaterialization {
                handle: "(phpc_NativeValueHandle){0}".to_string(),
                cleanup_after_use: Vec::new(),
            }
        };
        let replacement_value = self.emit_expr(replacement_expr)?;
        let replacement = self.materialize_native_array_c_value_handle(
            replacement_value.clone(),
            replacement_expr.span(),
        )?;

        let (operation, temp_prefix) = if index.is_some() {
            (
                NATIVE_VALUE_OFFSET_MUTATION_WRITE,
                "array_offset_assign_expr",
            )
        } else {
            (
                NATIVE_VALUE_OFFSET_MUTATION_APPEND,
                "array_offset_append_assign_expr",
            )
        };

        self.emit_array_offset_mutation_from_materialized(
            name,
            subject,
            offset,
            replacement,
            operation,
            temp_prefix,
        )?;
        Ok(Some(replacement_value))
    }

    fn emit_array_lvalue_assignment_expr(
        &mut self,
        target: &AssignTarget,
        replacement_expr: &Expr,
        span: Span,
    ) -> CompileResult<Option<CValue>> {
        if !self.uses_native_string_helpers {
            return Ok(None);
        }

        let (handle, path) = match target {
            AssignTarget::NestedArrayIndex { name, indices, .. } => {
                match self.variables.get(name).cloned() {
                    Some(CValue::ArrayHandle(handle)) => {
                        let indices = indices.iter().collect::<Vec<_>>();
                        let path =
                            self.materialize_native_array_lvalue_key_path(&indices, span, "")?;
                        (handle, path)
                    }
                    Some(subject)
                        if !matches!(subject, CValue::String(_) | CValue::StringExpr(_)) =>
                    {
                        return self
                            .emit_value_offset_path_write_assignment_expr(
                                name,
                                subject,
                                indices,
                                replacement_expr,
                                span,
                            )
                            .map(Some);
                    }
                    _ => return Ok(None),
                }
            }
            AssignTarget::NestedArrayAppend {
                name,
                indices,
                suffix_indices,
                ..
            } => match self.variables.get(name).cloned() {
                Some(CValue::ArrayHandle(handle)) => {
                    let prefix_indices = indices.iter().collect::<Vec<_>>();
                    let suffix_indices = suffix_indices.iter().collect::<Vec<_>>();
                    let path = self.materialize_native_array_lvalue_append_path(
                        &prefix_indices,
                        &suffix_indices,
                        span,
                        "",
                    )?;
                    (handle, path)
                }
                Some(subject) if !matches!(subject, CValue::String(_) | CValue::StringExpr(_)) => {
                    return self
                        .emit_value_offset_path_append_assignment_expr(
                            name,
                            subject,
                            indices,
                            suffix_indices,
                            replacement_expr,
                            span,
                        )
                        .map(Some);
                }
                _ => return Ok(None),
            },
            _ => return Ok(None),
        };

        let replacement_value = self.emit_expr(replacement_expr)?;
        let replacement = self.materialize_native_array_c_value_handle(
            replacement_value.clone(),
            replacement_expr.span(),
        )?;

        self.emit_array_lvalue_write_materialized_for_handle(
            &handle,
            path,
            replacement,
            "array_lvalue_assign_expr_result",
        )?;
        Ok(Some(replacement_value))
    }

    fn native_array_lvalue_key_target_parts<'a>(
        &self,
        target: &'a AssignTarget,
    ) -> Option<(String, Vec<&'a Expr>, Span)> {
        match target {
            AssignTarget::ArrayIndex {
                name,
                index: Some(index),
                span,
            } => match self.variables.get(name) {
                Some(CValue::ArrayHandle(handle)) => Some((handle.clone(), vec![index], *span)),
                _ => None,
            },
            AssignTarget::NestedArrayIndex {
                name,
                indices,
                span,
            } => match self.variables.get(name) {
                Some(CValue::ArrayHandle(handle)) => {
                    Some((handle.clone(), indices.iter().collect(), *span))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn materialize_array_lvalue_compound_assignment_result_for_target(
        &mut self,
        target: &AssignTarget,
        op: CompoundAssignOp,
        rhs_expr: &Expr,
        _span: Span,
        failure_cleanup: &str,
    ) -> CompileResult<Option<CNativeValueMaterialization>> {
        let Some((handle, indices, span)) = self.native_array_lvalue_key_target_parts(target)
        else {
            return Ok(None);
        };

        self.materialize_array_lvalue_compound_assignment_result_for_handle(
            &handle,
            &indices,
            span,
            op,
            rhs_expr,
            failure_cleanup,
        )
        .map(Some)
    }

    fn materialize_array_lvalue_compound_assignment_result_for_handle(
        &mut self,
        handle: &str,
        indices: &[&Expr],
        span: Span,
        op: CompoundAssignOp,
        rhs_expr: &Expr,
        failure_cleanup: &str,
    ) -> CompileResult<CNativeValueMaterialization> {
        self.uses_native_string_helpers = true;
        self.uses_native_array_helpers = true;
        self.uses_native_array_lvalue_helpers = true;

        let path = self.materialize_native_array_lvalue_key_path(indices, span, failure_cleanup)?;
        let path_cleanup = c_cleanup_sequence(&path.cleanup_after_use);
        let read_owner = self.next_native_name("array_lvalue_compound_read_owner");
        let read_result = self.next_native_name("array_lvalue_compound_read_result");
        let current = self.next_native_name("array_lvalue_compound_current");
        self.body.push(format!(
            "phpc_NativeArrayLvalueOwner {read_owner} = phpc_native_array_lvalue_owner_array({handle});"
        ));
        self.body.push(format!(
            "phpc_NativeArrayLvalueResult {read_result} = phpc_native_array_lvalue_owner_value_operation_result({read_owner}, {}, {}, PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ, 0, 0, 0, (phpc_NativeValueHandle){{0}});",
            path.path, path.len
        ));
        self.emit_native_array_lvalue_result_check(
            &read_result,
            &format!("{path_cleanup}{failure_cleanup}"),
        );
        self.body.push(format!(
            "phpc_NativeValueHandle {current} = {read_result}.value;"
        ));
        self.body.push(format!(
            "{read_result}.value = (phpc_NativeValueHandle){{0}};"
        ));
        self.body.push(format!(
            "phpc_native_array_lvalue_result_free({read_result});"
        ));

        let current_value = CNativeValueMaterialization {
            handle: current.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({current});")],
        };
        let rhs_failure_cleanup = format!(
            "{}{path_cleanup}{failure_cleanup}",
            c_cleanup_sequence(&current_value.cleanup_after_use)
        );
        let rhs_value =
            self.materialize_native_value_result_operand(rhs_expr, &rhs_failure_cleanup)?;
        let value = self.emit_native_value_binary_result_handle(
            current_value,
            native_array_lvalue_compound_binary_op_tag(op),
            rhs_value,
            &format!("{path_cleanup}{failure_cleanup}"),
        );

        let write_owner = self.next_native_name("array_lvalue_compound_write_owner");
        let write_result = self.next_native_name("array_lvalue_compound_write_result");
        self.body.push(format!(
            "phpc_NativeArrayLvalueOwner {write_owner} = phpc_native_array_lvalue_owner_array({handle});"
        ));
        self.body.push(format!(
            "phpc_NativeArrayLvalueResult {write_result} = phpc_native_array_lvalue_owner_value_operation_result({write_owner}, {}, {}, PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE, 0, 0, 0, {});",
            path.path, path.len, value.handle
        ));
        self.emit_native_array_lvalue_result_check(
            &write_result,
            &format!(
                "{}{path_cleanup}{failure_cleanup}",
                c_cleanup_sequence(&value.cleanup_after_use)
            ),
        );
        self.body.push(format!(
            "phpc_native_array_lvalue_result_free({write_result});"
        ));
        self.body.extend(path.cleanup_after_use);

        Ok(value)
    }

    fn materialize_array_lvalue_increment_decrement_result_for_target(
        &mut self,
        target: &AssignTarget,
        op: IncrementDecrementOp,
        position: IncrementDecrementPosition,
        _span: Span,
        failure_cleanup: &str,
    ) -> CompileResult<Option<CNativeValueMaterialization>> {
        let Some((handle, path)) =
            self.materialize_array_lvalue_increment_decrement_target_path(target, failure_cleanup)?
        else {
            return Ok(None);
        };

        self.materialize_array_lvalue_increment_decrement_result_for_handle(
            &handle,
            path,
            op,
            position,
            failure_cleanup,
        )
        .map(Some)
    }

    fn materialize_array_lvalue_increment_decrement_target_path(
        &mut self,
        target: &AssignTarget,
        failure_cleanup: &str,
    ) -> CompileResult<Option<(String, CNativeArrayLvaluePath)>> {
        match target {
            AssignTarget::ArrayIndex { name, index, span } => {
                let Some(CValue::ArrayHandle(handle)) = self.variables.get(name).cloned() else {
                    return Ok(None);
                };
                let path = if let Some(index) = index.as_ref() {
                    self.materialize_native_array_lvalue_key_path(&[index], *span, failure_cleanup)?
                } else {
                    self.materialize_native_array_lvalue_append_path(
                        &[],
                        &[],
                        *span,
                        failure_cleanup,
                    )?
                };
                Ok(Some((handle, path)))
            }
            AssignTarget::NestedArrayIndex {
                name,
                indices,
                span,
            } => {
                let Some(CValue::ArrayHandle(handle)) = self.variables.get(name).cloned() else {
                    return Ok(None);
                };
                let indices = indices.iter().collect::<Vec<_>>();
                let path = self.materialize_native_array_lvalue_key_path(
                    &indices,
                    *span,
                    failure_cleanup,
                )?;
                Ok(Some((handle, path)))
            }
            AssignTarget::NestedArrayAppend {
                name,
                indices,
                suffix_indices,
                span,
            } if suffix_indices.is_empty() => {
                let Some(CValue::ArrayHandle(handle)) = self.variables.get(name).cloned() else {
                    return Ok(None);
                };
                let prefix_indices = indices.iter().collect::<Vec<_>>();
                let path = self.materialize_native_array_lvalue_append_path(
                    &prefix_indices,
                    &[],
                    *span,
                    failure_cleanup,
                )?;
                Ok(Some((handle, path)))
            }
            _ => Ok(None),
        }
    }

    fn materialize_array_lvalue_increment_decrement_result_for_handle(
        &mut self,
        handle: &str,
        path: CNativeArrayLvaluePath,
        op: IncrementDecrementOp,
        position: IncrementDecrementPosition,
        failure_cleanup: &str,
    ) -> CompileResult<CNativeValueMaterialization> {
        self.uses_native_string_helpers = true;
        self.uses_native_array_helpers = true;
        self.uses_native_array_lvalue_helpers = true;

        let path_cleanup = c_cleanup_sequence(&path.cleanup_after_use);
        let owner = self.next_native_name("array_lvalue_increment_owner");
        let result = self.next_native_name("array_lvalue_increment_result");
        let value = self.next_native_name("array_lvalue_increment_value");
        let op_tag = native_array_lvalue_increment_decrement_op_tag(op);
        let position_tag = native_array_lvalue_increment_decrement_position_tag(position);
        self.body.push(format!(
            "phpc_NativeArrayLvalueOwner {owner} = phpc_native_array_lvalue_owner_array({handle});"
        ));
        self.body.push(format!(
            "phpc_NativeArrayLvalueResult {result} = phpc_native_array_lvalue_owner_value_operation_result({owner}, {}, {}, PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UPDATE, PHPC_NATIVE_ARRAY_LVALUE_VALUE_RESULT_INCREMENT_DECREMENT, {op_tag}, {position_tag}, (phpc_NativeValueHandle){{0}});",
            path.path, path.len
        ));
        self.emit_native_array_lvalue_result_check(
            &result,
            &format!("{path_cleanup}{failure_cleanup}"),
        );
        self.body
            .push(format!("phpc_NativeValueHandle {value} = {result}.value;"));
        self.body
            .push(format!("{result}.value = (phpc_NativeValueHandle){{0}};"));
        self.body
            .push(format!("phpc_native_array_lvalue_result_free({result});"));
        self.body.extend(path.cleanup_after_use);

        Ok(CNativeValueMaterialization {
            handle: value.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({value});")],
        })
    }

    fn emit_array_offset_mutation_from_materialized(
        &mut self,
        name: &str,
        subject: CNativeValueMaterialization,
        offset: CNativeValueMaterialization,
        replacement: CNativeValueMaterialization,
        operation: u8,
        temp_prefix: &str,
    ) -> CompileResult<()> {
        self.uses_native_string_helpers = true;
        self.uses_native_array_helpers = true;
        self.uses_native_value_offset_mutation = true;

        let mutation = self.next_native_name(&format!("{temp_prefix}_value"));
        let diagnostic = self.next_native_name(&format!("{temp_prefix}_diagnostic"));
        let array = self.next_native_name(&format!("{temp_prefix}_array"));

        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {mutation} = phpc_native_value_offset_mutation_operation_with_diagnostic({}, {}, {}, {operation}, &{diagnostic});",
            subject.handle, offset.handle, replacement.handle
        ));
        self.body.push(format!(
            "if ({diagnostic}.ptr != NULL) {{ phpc_native_diagnostic_report({diagnostic}); }}"
        ));
        let mutation_failure_cleanup = format!(
            "{}{}{}",
            c_cleanup_sequence(&replacement.cleanup_after_use),
            c_cleanup_sequence(&offset.cleanup_after_use),
            c_cleanup_sequence(&subject.cleanup_after_use)
        );
        let mutation_error_exit = self.native_error_exit(&mutation_failure_cleanup);
        self.body.push(format!(
            "if ({mutation}.ptr == NULL) {{ {mutation_error_exit} }}"
        ));
        self.body.push(format!(
            "phpc_NativeArrayHandle {array} = phpc_native_value_array_clone({mutation});"
        ));
        let array_error_exit = self.native_error_exit(&format!(
            "phpc_native_value_free({mutation}); {mutation_failure_cleanup}"
        ));
        self.body
            .push(format!("if ({array}.ptr == NULL) {{ {array_error_exit} }}"));
        self.body
            .push(format!("phpc_native_value_free({mutation});"));
        self.body.extend(replacement.cleanup_after_use);
        self.body.extend(offset.cleanup_after_use);
        self.body.extend(subject.cleanup_after_use);
        self.array_cleanup_handles.push(array.clone());
        self.store_variable_value(name, CValue::ArrayHandle(array));
        Ok(())
    }

    fn emit_strlen_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        if let Some(operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(operation));
        }

        if args.len() != 1 {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let value = self.emit_expr(&args[0])?;
        if self.uses_native_string_helpers {
            return self.emit_native_value_string_byte_len(value, span);
        }

        self.strlen_result_for_value(&value)
            .map(|length| CValue::Int(length.to_string()))
            .ok_or_else(|| {
                self.unsupported_direct_call(span, NativeCallBlocker::ReturnValueOwnership)
            })
    }

    fn emit_string_predicate_call(
        &mut self,
        predicate: NativeStringPredicate,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<CValue> {
        if let Some(operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(operation));
        }

        if args.len() != 2 || !self.uses_native_string_helpers {
            return Err(self.unsupported_direct_named_call(
                args,
                span,
                ASSEMBLY_STRING_PREDICATE_REJECTION,
            ));
        }

        let haystack = self.emit_value_operand_expr(&args[0])?;
        let needle = self.emit_value_operand_expr(&args[1])?;
        let haystack = self.emit_native_value_for_cvalue(haystack, span)?;
        let needle = self.emit_native_value_for_cvalue(needle, span)?;
        let result = format!("string_predicate_result_{}", self.next_native_temp);
        self.next_native_temp += 1;
        let diagnostic = format!("string_predicate_diagnostic_{}", self.next_native_temp);
        self.next_native_temp += 1;

        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "_Bool {result} = phpc_native_value_string_predicate_with_diagnostic({haystack}, {needle}, {}, &{diagnostic});",
            predicate as u8
        ));
        self.emit_report_native_diagnostic(&diagnostic);
        self.body.push(format!("phpc_native_value_free({needle});"));
        self.body
            .push(format!("phpc_native_value_free({haystack});"));

        Ok(CValue::BoolExpr(result))
    }

    fn emit_string_int_operation_call(
        &mut self,
        operation: NativeStringIntOperation,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<CValue> {
        if let Some(call_operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(call_operation));
        }

        if !self.uses_native_string_helpers {
            return Err(self.unsupported_direct_named_call(
                args,
                span,
                ASSEMBLY_STRING_INT_OPERATION_REJECTION,
            ));
        }

        let (subject_arg, operand_arg, offset_arg, length_arg, has_length) = match operation {
            NativeStringIntOperation::CaseCompare if args.len() == 2 => {
                (&args[0], Some(&args[1]), None, None, false)
            }
            NativeStringIntOperation::ByteCompare if args.len() == 2 => {
                (&args[0], Some(&args[1]), None, None, false)
            }
            NativeStringIntOperation::BytePrefixCompare
            | NativeStringIntOperation::CasePrefixCompare
                if args.len() == 3 =>
            {
                (&args[0], Some(&args[1]), None, Some(&args[2]), true)
            }
            NativeStringIntOperation::SubstrCount if (2..=4).contains(&args.len()) => (
                &args[0],
                Some(&args[1]),
                args.get(2),
                args.get(3),
                args.get(3).is_some(),
            ),
            NativeStringIntOperation::Ordinal | NativeStringIntOperation::Crc32
                if args.len() == 1 =>
            {
                (&args[0], None, None, None, false)
            }
            _ => {
                return Err(self.unsupported_direct_named_call(
                    args,
                    span,
                    ASSEMBLY_STRING_INT_OPERATION_REJECTION,
                ))
            }
        };

        let subject = self.emit_value_operand_expr(subject_arg)?;
        let subject = self
            .emit_native_value_for_cvalue(subject, span)
            .map_err(|_| self.unsupported(span, ASSEMBLY_STRING_INT_OPERATION_REJECTION))?;
        let operand = if let Some(expr) = operand_arg {
            let value = self.emit_value_operand_expr(expr)?;
            self.emit_native_value_for_cvalue(value, span)
                .map_err(|_| self.unsupported(span, ASSEMBLY_STRING_INT_OPERATION_REJECTION))?
        } else {
            "(phpc_NativeValueHandle){0}".to_string()
        };
        let offset = self.emit_optional_native_int_argument(
            offset_arg,
            span,
            "0",
            NativeIntConversionOperation::StringOffset,
            ASSEMBLY_STRING_INT_OPERATION_REJECTION,
        )?;
        let length = self.emit_optional_native_int_argument(
            length_arg,
            span,
            "0",
            NativeIntConversionOperation::StringLength,
            ASSEMBLY_STRING_INT_OPERATION_REJECTION,
        )?;
        let result = format!("string_int_result_{}", self.next_native_temp);
        self.next_native_temp += 1;
        let diagnostic = format!("string_int_diagnostic_{}", self.next_native_temp);
        self.next_native_temp += 1;
        let flags = u8::from(has_length);

        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "long long {result} = (long long)phpc_native_value_string_int_operation_with_diagnostic({subject}, {operand}, (int64_t)({offset}), (int64_t)({length}), {flags}, {}, &{diagnostic});",
            operation as u8
        ));
        self.emit_report_native_diagnostic(&diagnostic);
        if operand_arg.is_some() {
            self.body
                .push(format!("phpc_native_value_free({operand});"));
        }
        self.body
            .push(format!("phpc_native_value_free({subject});"));

        Ok(CValue::Int(result))
    }

    fn emit_optional_native_int_argument(
        &mut self,
        expr: Option<&Expr>,
        span: Span,
        default: &str,
        operation: NativeIntConversionOperation,
        rejection: &'static str,
    ) -> CompileResult<String> {
        let Some(expr) = expr else {
            return Ok(default.to_string());
        };

        let value = self.emit_value_operand_expr(expr)?;
        let value_handle = self
            .emit_native_value_for_cvalue(value, span)
            .map_err(|_| self.unsupported(span, rejection))?;
        let result = self.next_native_name("int_conversion_result");
        let diagnostic = self.next_native_name("int_conversion_diagnostic");

        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "long long {result} = (long long)phpc_native_value_to_int64_with_diagnostic({value_handle}, {}, &{diagnostic});",
            operation as u8
        ));
        self.emit_report_native_diagnostic(&diagnostic);
        self.body
            .push(format!("phpc_native_value_free({value_handle});"));

        Ok(result)
    }

    fn emit_string_distance_operation_call(
        &mut self,
        operation: NativeStringDistanceOperation,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<CValue> {
        if let Some(call_operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(call_operation));
        }

        if !self.uses_native_string_helpers {
            return Err(self.unsupported_direct_named_call(
                args,
                span,
                ASSEMBLY_STRING_DISTANCE_OPERATION_REJECTION,
            ));
        }

        let (subject_arg, operand_arg, insertion_cost_arg, replacement_cost_arg, deletion_cost_arg) =
            match operation {
                NativeStringDistanceOperation::Levenshtein if (2..=5).contains(&args.len()) => {
                    (&args[0], &args[1], args.get(2), args.get(3), args.get(4))
                }
                NativeStringDistanceOperation::SimilarText if args.len() == 2 => {
                    (&args[0], &args[1], None, None, None)
                }
                _ => {
                    return Err(self.unsupported_direct_named_call(
                        args,
                        span,
                        ASSEMBLY_STRING_DISTANCE_OPERATION_REJECTION,
                    ))
                }
            };

        let subject = self.emit_value_operand_expr(subject_arg)?;
        let operand = self.emit_value_operand_expr(operand_arg)?;
        let subject = self
            .emit_native_value_for_cvalue(subject, span)
            .map_err(|_| self.unsupported(span, ASSEMBLY_STRING_DISTANCE_OPERATION_REJECTION))?;
        let operand = self
            .emit_native_value_for_cvalue(operand, span)
            .map_err(|_| self.unsupported(span, ASSEMBLY_STRING_DISTANCE_OPERATION_REJECTION))?;
        let insertion_cost = self.emit_optional_native_int_argument(
            insertion_cost_arg,
            span,
            "1",
            NativeIntConversionOperation::StringDistanceCost,
            ASSEMBLY_STRING_DISTANCE_OPERATION_REJECTION,
        )?;
        let replacement_cost = self.emit_optional_native_int_argument(
            replacement_cost_arg,
            span,
            "1",
            NativeIntConversionOperation::StringDistanceCost,
            ASSEMBLY_STRING_DISTANCE_OPERATION_REJECTION,
        )?;
        let deletion_cost = self.emit_optional_native_int_argument(
            deletion_cost_arg,
            span,
            "1",
            NativeIntConversionOperation::StringDistanceCost,
            ASSEMBLY_STRING_DISTANCE_OPERATION_REJECTION,
        )?;
        let result = self.next_native_name(match operation {
            NativeStringDistanceOperation::Levenshtein => "levenshtein_result",
            NativeStringDistanceOperation::SimilarText => "similar_text_result",
        });
        let diagnostic = self.next_native_name("string_distance_diagnostic");

        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "long long {result} = (long long)phpc_native_value_string_distance_operation_with_diagnostic({subject}, {operand}, (int64_t)({insertion_cost}), (int64_t)({replacement_cost}), (int64_t)({deletion_cost}), {}, &{diagnostic});",
            operation as u8
        ));
        self.emit_report_native_diagnostic(&diagnostic);
        self.body
            .push(format!("phpc_native_value_free({operand});"));
        self.body
            .push(format!("phpc_native_value_free({subject});"));

        Ok(CValue::Int(result))
    }

    fn emit_filesystem_path_operation_call(
        &mut self,
        operation: NativeFilesystemPathOperation,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<CValue> {
        if let Some(call_operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(call_operation));
        }

        if !self.uses_native_string_helpers {
            return Err(self.unsupported_direct_named_call(
                args,
                span,
                native_filesystem_path_operation_assembly_rejection(operation),
            ));
        }

        match operation {
            NativeFilesystemPathOperation::FileGetContents => {
                if args.is_empty() || args.len() > 2 {
                    return Err(self.unsupported_direct_named_call(
                        args,
                        span,
                        ASSEMBLY_FILESYSTEM_PATH_OPERATION_REJECTION,
                    ));
                }
                let path = self.emit_value_operand_expr(&args[0])?;
                let (option, flags) = if let Some(use_include_path) = args.get(1) {
                    (
                        Some(self.emit_value_operand_expr(use_include_path)?),
                        NATIVE_FILESYSTEM_PATH_HAS_BOOLEAN_OPTION.to_string(),
                    )
                } else {
                    (None, "0".to_string())
                };
                self.emit_native_filesystem_path_operation(
                    operation,
                    Some(path),
                    option,
                    "0".to_string(),
                    "0".to_string(),
                    flags,
                    span,
                )
            }
            NativeFilesystemPathOperation::Realpath
            | NativeFilesystemPathOperation::FileExists
            | NativeFilesystemPathOperation::IsDir
            | NativeFilesystemPathOperation::IsFile
            | NativeFilesystemPathOperation::IsReadable
            | NativeFilesystemPathOperation::IsWritable
            | NativeFilesystemPathOperation::IsLink
            | NativeFilesystemPathOperation::FileSize
            | NativeFilesystemPathOperation::FileMTime => {
                if args.len() != 1 {
                    return Err(self.unsupported_direct_named_call(
                        args,
                        span,
                        ASSEMBLY_FILESYSTEM_PATH_OPERATION_REJECTION,
                    ));
                }
                let path = self.emit_value_operand_expr(&args[0])?;
                self.emit_native_filesystem_path_operation(
                    operation,
                    Some(path),
                    None,
                    "0".to_string(),
                    "0".to_string(),
                    "0".to_string(),
                    span,
                )
            }
            NativeFilesystemPathOperation::GetCwd => {
                if !args.is_empty() {
                    return Err(self.unsupported_direct_named_call(
                        args,
                        span,
                        ASSEMBLY_FILESYSTEM_PATH_OPERATION_REJECTION,
                    ));
                }
                self.emit_native_filesystem_path_operation(
                    operation,
                    None,
                    None,
                    "0".to_string(),
                    "0".to_string(),
                    "0".to_string(),
                    span,
                )
            }
            NativeFilesystemPathOperation::RealpathCacheGet
            | NativeFilesystemPathOperation::RealpathCacheSize => {
                if !args.is_empty() {
                    return Err(self.unsupported_direct_named_call(
                        args,
                        span,
                        ASSEMBLY_FILESYSTEM_PATH_OPERATION_REJECTION,
                    ));
                }
                self.emit_native_filesystem_path_operation(
                    operation,
                    None,
                    None,
                    "0".to_string(),
                    "0".to_string(),
                    "0".to_string(),
                    span,
                )
            }
            NativeFilesystemPathOperation::ClearStatCache => {
                if args.len() > 2 {
                    return Err(self.unsupported_direct_named_call(
                        args,
                        span,
                        ASSEMBLY_FILESYSTEM_PATH_OPERATION_REJECTION,
                    ));
                }
                let (option, mut flags) = if let Some(clear_realpath_cache) = args.first() {
                    (
                        Some(self.emit_value_operand_expr(clear_realpath_cache)?),
                        NATIVE_FILESYSTEM_PATH_HAS_BOOLEAN_OPTION.to_string(),
                    )
                } else {
                    (None, "0".to_string())
                };
                let path = if let Some(path) = args.get(1) {
                    flags = format!("({flags} | {NATIVE_FILESYSTEM_PATH_HAS_PATH})");
                    Some(self.emit_value_operand_expr(path)?)
                } else {
                    None
                };
                self.emit_native_filesystem_path_operation(
                    operation,
                    path,
                    option,
                    "0".to_string(),
                    "0".to_string(),
                    flags,
                    span,
                )
            }
        }
    }

    fn emit_native_filesystem_path_operation(
        &mut self,
        operation: NativeFilesystemPathOperation,
        path: Option<CValue>,
        option: Option<CValue>,
        offset: String,
        length: String,
        flags: String,
        span: Span,
    ) -> CompileResult<CValue> {
        let owns_path = path.is_some();
        let path = match path {
            Some(value) => self
                .emit_native_value_for_cvalue(value, span)
                .map_err(|_| {
                    self.unsupported(span, ASSEMBLY_FILESYSTEM_PATH_OPERATION_REJECTION)
                })?,
            None => "(phpc_NativeValueHandle){0}".to_string(),
        };
        let owns_option = option.is_some();
        let option = match option {
            Some(value) => self
                .emit_native_value_for_cvalue(value, span)
                .map_err(|_| {
                    self.unsupported(span, ASSEMBLY_FILESYSTEM_PATH_OPERATION_REJECTION)
                })?,
            None => "(phpc_NativeValueHandle){0}".to_string(),
        };
        let result =
            self.next_native_name(native_filesystem_path_operation_result_prefix(operation));
        let diagnostic = self.next_native_name("filesystem_path_operation_diagnostic");

        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {result} = phpc_native_value_filesystem_path_operation_with_diagnostic({path}, {option}, (int64_t)({offset}), (int64_t)({length}), {flags}, {}, &{diagnostic});",
            operation as u8
        ));
        self.emit_report_native_diagnostic(&diagnostic);
        self.body.push(format!("phpc_native_value_free({result});"));
        if owns_option {
            self.body.push(format!("phpc_native_value_free({option});"));
        }
        if owns_path {
            self.body.push(format!("phpc_native_value_free({path});"));
        }
        Ok(CValue::Null)
    }

    fn emit_function_exists_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        if let Some(operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(operation));
        }

        if args.len() != 1 {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let value = self.emit_expr(&args[0])?;
        self.function_exists_result_for_value(&value)
            .map(CValue::Bool)
            .ok_or_else(|| {
                self.unsupported_direct_call(span, NativeCallBlocker::UnknownCalleeDiagnostics)
            })
    }

    fn emit_is_callable_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        if let Some(operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(operation));
        }

        if !(1..=2).contains(&args.len()) {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let value = self.emit_expr(&args[0])?;
        let syntax_only = if let Some(arg) = args.get(1) {
            match self.emit_expr(arg)? {
                CValue::Bool(value) => value,
                _ => {
                    return Err(self.unsupported_direct_call(
                        span,
                        NativeCallBlocker::ArgumentEvaluationCleanup,
                    ));
                }
            }
        } else {
            false
        };

        self.is_callable_result_for_value(&value, syntax_only)
            .map(CValue::Bool)
            .ok_or_else(|| {
                self.unsupported_direct_call(span, NativeCallBlocker::UnknownCalleeDiagnostics)
            })
    }

    fn emit_defined_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        if let Some(operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(operation));
        }

        if args.len() != 1 {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let value = self.emit_expr(&args[0])?;
        self.defined_result_for_value(&value)
            .map(CValue::Bool)
            .ok_or_else(|| self.unsupported(span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION))
    }

    fn emit_native_type_introspection_call(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<CValue> {
        if args.len() == 1 {
            if let Some(predicate_tag) = native_value_type_predicate_tag(name) {
                if let Some(value) = self.try_materialize_native_value_result_expr(&args[0], "")? {
                    return Ok(self.emit_native_value_type_predicate(value, predicate_tag));
                }
            }
        }

        if let Some(operation) = native_direct_call_argument_result_operation(args, span) {
            return Err(self.unsupported_call_operation(operation));
        }

        if is_native_metadata_exists_builtin(name) {
            return self.emit_native_metadata_exists_call(args, span);
        }
        if is_native_member_metadata_exists_builtin(name) {
            return self.emit_native_member_metadata_exists_call(args, span);
        }
        if is_native_relationship_metadata_builtin(name) {
            return self.emit_native_relationship_metadata_call(args, span);
        }

        if args.len() != 1 {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let value = self.emit_expr(&args[0])?;
        if matches!(value, CValue::NativeValueHandle(_)) {
            return Err(self.unsupported_direct_call(span, NativeCallBlocker::ReturnValueOwnership));
        }
        match name.to_ascii_lowercase().as_str() {
            "gettype" => Ok(CValue::String(c_gettype_name(&value).to_string())),
            "get_debug_type" => Ok(CValue::String(c_debug_type_name(&value).to_string())),
            "is_null" => Ok(CValue::Bool(matches!(value, CValue::Null))),
            "is_bool" => Ok(CValue::Bool(matches!(
                value,
                CValue::Bool(_) | CValue::BoolExpr(_) | CValue::ComparisonDecision(_)
            ))),
            "is_int" | "is_integer" | "is_long" => {
                Ok(CValue::Bool(matches!(value, CValue::Int(_))))
            }
            "is_float" | "is_double" => Ok(CValue::Bool(matches!(value, CValue::Float(_)))),
            "is_string" => Ok(CValue::Bool(matches!(
                value,
                CValue::String(_) | CValue::StringExpr(_)
            ))),
            "is_array" => Ok(CValue::Bool(matches!(value, CValue::ArrayHandle(_)))),
            "is_scalar" => Ok(CValue::Bool(matches!(
                value,
                CValue::Bool(_)
                    | CValue::BoolExpr(_)
                    | CValue::ComparisonDecision(_)
                    | CValue::Int(_)
                    | CValue::Float(_)
                    | CValue::String(_)
                    | CValue::StringExpr(_)
            ))),
            "is_numeric" => self
                .is_numeric_result_for_value(&value)
                .map(CValue::Bool)
                .ok_or_else(|| {
                    self.unsupported_direct_call(span, NativeCallBlocker::ReturnValueOwnership)
                }),
            "is_countable" | "is_iterable" => Ok(CValue::Bool(false)),
            "extension_loaded" => match value {
                CValue::String(name) => Ok(CValue::Bool(is_compat_loaded_extension_name(&name))),
                CValue::StringExpr(_) => Ok(CValue::Bool(false)),
                _ => Err(self
                    .unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)),
            },
            "is_object" => Ok(CValue::Bool(false)),
            _ => {
                Err(self.unsupported_direct_call(span, NativeCallBlocker::UnknownCalleeDiagnostics))
            }
        }
    }

    fn emit_native_metadata_exists_call(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<CValue> {
        if !(1..=2).contains(&args.len()) {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let name = self.emit_expr(&args[0])?;
        if !matches!(name, CValue::String(_) | CValue::StringExpr(_)) {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }
        if self.c_value_mentions_builtin_class(&name) {
            return Err(self.unsupported(span, ASSEMBLY_OBJECT_METADATA_REJECTION));
        }

        if let Some(autoload) = args.get(1) {
            let autoload = self.emit_expr(autoload)?;
            if !matches!(autoload, CValue::Bool(_) | CValue::BoolExpr(_)) {
                return Err(self
                    .unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup));
            }
        }

        Ok(CValue::Bool(false))
    }

    fn emit_native_member_metadata_exists_call(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<CValue> {
        if args.len() != 2 {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let object_or_class = self.emit_expr(&args[0])?;
        let member = self.emit_expr(&args[1])?;
        if !matches!(object_or_class, CValue::String(_) | CValue::StringExpr(_))
            || !matches!(member, CValue::String(_) | CValue::StringExpr(_))
        {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }
        if self.c_value_mentions_builtin_class(&object_or_class) {
            return Err(self.unsupported(span, ASSEMBLY_OBJECT_METADATA_REJECTION));
        }

        Ok(CValue::Bool(false))
    }

    fn emit_native_relationship_metadata_call(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<CValue> {
        if !(2..=3).contains(&args.len()) {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }

        let object_or_class = self.emit_expr(&args[0])?;
        let class_name = self.emit_expr(&args[1])?;
        if !matches!(object_or_class, CValue::String(_) | CValue::StringExpr(_))
            || !matches!(class_name, CValue::String(_) | CValue::StringExpr(_))
        {
            return Err(
                self.unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup)
            );
        }
        if self.c_value_mentions_builtin_class(&object_or_class)
            || self.c_value_mentions_builtin_class(&class_name)
        {
            return Err(self.unsupported(span, ASSEMBLY_OBJECT_METADATA_REJECTION));
        }

        if let Some(allow_string) = args.get(2) {
            let allow_string = self.emit_expr(allow_string)?;
            if !matches!(allow_string, CValue::Bool(_) | CValue::BoolExpr(_)) {
                return Err(self
                    .unsupported_direct_call(span, NativeCallBlocker::ArgumentEvaluationCleanup));
            }
        }

        Ok(CValue::Bool(false))
    }

    fn c_value_mentions_builtin_class(&self, value: &CValue) -> bool {
        self.known_string_values_for_value(value)
            .map(|values| {
                values
                    .values()
                    .iter()
                    .any(|value| is_builtin_class_name(value))
            })
            .unwrap_or(false)
    }

    fn is_numeric_result_for_value(&self, value: &CValue) -> Option<bool> {
        match value {
            CValue::Int(_) | CValue::Float(_) => Some(true),
            CValue::Null
            | CValue::Bool(_)
            | CValue::BoolExpr(_)
            | CValue::ComparisonDecision(_)
            | CValue::ArrayHandle(_) => Some(false),
            CValue::NativeValueHandle(_) => None,
            CValue::String(value) => Some(classify_php_numeric_string(value).is_numeric()),
            CValue::StringExpr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_numeric_result(&values)
            }
        }
    }

    fn function_exists_result_for_value(&self, value: &CValue) -> Option<bool> {
        match value {
            CValue::String(value) => Some(is_native_known_function_name(value)),
            CValue::StringExpr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_function_exists_result(&values)
            }
            _ => None,
        }
    }

    fn strlen_result_for_value(&self, value: &CValue) -> Option<usize> {
        match value {
            CValue::String(value) => Some(value.len()),
            CValue::StringExpr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_byte_length(&values)
            }
            _ => None,
        }
    }

    fn is_callable_result_for_value(&self, value: &CValue, syntax_only: bool) -> Option<bool> {
        match value {
            CValue::String(_) | CValue::StringExpr(_) if syntax_only => Some(true),
            CValue::Null
            | CValue::Bool(_)
            | CValue::BoolExpr(_)
            | CValue::ComparisonDecision(_)
            | CValue::Int(_)
            | CValue::Float(_) => Some(false),
            _ => self.function_exists_result_for_value(value),
        }
    }

    fn defined_result_for_value(&self, value: &CValue) -> Option<bool> {
        match value {
            CValue::String(value) => native_defined_result(value),
            CValue::StringExpr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_defined_result(&values)
            }
            _ => None,
        }
    }

    fn emit_assignment(&mut self, target: &AssignTarget, expr: &Expr) -> CompileResult<()> {
        if let Some(operation) = native_assignment_target_call_operation(target) {
            return Err(self.unsupported_call_operation(operation));
        }

        match target {
            AssignTarget::Variable { name, .. } => {
                if self.uses_native_string_helpers {
                    if let Some(value) = self.try_materialize_native_value_result_expr(expr, "")? {
                        self.store_native_value_result_variable(name, value);
                        return Ok(());
                    }
                }
                let value = self.emit_expr(expr)?;
                self.store_variable_value(name, value);
                Ok(())
            }
            AssignTarget::List { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_DESTRUCTURING_REJECTION))
            }
            AssignTarget::ArrayIndex { name, index, span } => {
                if self.uses_native_string_helpers {
                    return match self.variables.get(name).cloned() {
                        Some(CValue::ArrayHandle(handle)) => {
                            if let Some(index) = index.as_ref() {
                                self.emit_array_offset_mutation_assignment(
                                    name, &handle, index, expr, *span,
                                )
                            } else {
                                self.emit_array_offset_append_assignment(name, &handle, expr, *span)
                            }
                        }
                        Some(subject @ (CValue::String(_) | CValue::StringExpr(_))) => {
                            let Some(index) = index.as_ref() else {
                                return Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION));
                            };
                            self.emit_string_offset_write_assignment(
                                name, subject, index, expr, *span,
                            )
                        }
                        Some(subject) if index.is_none() => {
                            self.emit_value_offset_append_assignment(name, subject, expr, *span)
                        }
                        Some(subject) => {
                            let Some(index) = index.as_ref() else {
                                return Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION));
                            };
                            self.emit_value_offset_write_assignment(
                                name, subject, index, expr, *span,
                            )
                        }
                        None => {
                            let Some(index) = index.as_ref() else {
                                return Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION));
                            };
                            self.emit_value_offset_write_assignment(
                                name,
                                CValue::Null,
                                index,
                                expr,
                                *span,
                            )
                        }
                    };
                }
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            AssignTarget::NestedArrayIndex {
                name,
                indices,
                span,
            } => {
                if self.uses_native_string_helpers {
                    if let Some(CValue::ArrayHandle(handle)) = self.variables.get(name).cloned() {
                        let indices = indices.iter().collect::<Vec<_>>();
                        return self
                            .emit_array_lvalue_write_for_handle(&handle, &indices, expr, *span);
                    }
                    if let Some(subject) = self.variables.get(name).cloned() {
                        if matches!(subject, CValue::String(_) | CValue::StringExpr(_)) {
                            return Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION));
                        }
                        return self.emit_value_offset_path_write_assignment(
                            name, subject, indices, expr, *span,
                        );
                    }
                }
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            AssignTarget::NestedArrayAppend {
                name,
                indices,
                suffix_indices,
                span,
            } => {
                if self.uses_native_string_helpers {
                    if let Some(CValue::ArrayHandle(handle)) = self.variables.get(name).cloned() {
                        let prefix_indices = indices.iter().collect::<Vec<_>>();
                        let suffix_indices = suffix_indices.iter().collect::<Vec<_>>();
                        return self.emit_array_lvalue_append_write_for_handle(
                            &handle,
                            &prefix_indices,
                            &suffix_indices,
                            expr,
                            *span,
                        );
                    }
                    if let Some(subject) = self.variables.get(name).cloned() {
                        if matches!(subject, CValue::String(_) | CValue::StringExpr(_)) {
                            return Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION));
                        }
                        return self.emit_value_offset_path_append_assignment(
                            name,
                            subject,
                            indices,
                            suffix_indices,
                            expr,
                            *span,
                        );
                    }
                }
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            AssignTarget::ObjectPropertyArrayIndex { span, .. }
            | AssignTarget::DynamicObjectPropertyArrayIndex { span, .. }
            | AssignTarget::NonDirectObjectPropertyArrayIndex { span, .. }
            | AssignTarget::NonDirectObjectPropertyArrayAppend { span, .. }
            | AssignTarget::NonDirectDynamicObjectPropertyArrayIndex { span, .. }
            | AssignTarget::NonDirectDynamicObjectPropertyArrayAppend { span, .. }
            | AssignTarget::ObjectPropertyArrayAppend { span, .. }
            | AssignTarget::DynamicObjectPropertyArrayAppend { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION))
            }
            AssignTarget::Property { span, .. }
            | AssignTarget::NonDirectProperty { span, .. }
            | AssignTarget::NonDirectDynamicProperty { span, .. }
            | AssignTarget::DynamicProperty { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_OBJECT_PROPERTY_REJECTION))
            }
            AssignTarget::StaticProperty { span, .. }
            | AssignTarget::ObjectStaticProperty { span, .. }
            | AssignTarget::SelfStaticProperty { span, .. }
            | AssignTarget::ParentStaticProperty { span, .. }
            | AssignTarget::LateStaticProperty { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_STATIC_MEMBER_REJECTION))
            }
        }
    }

    fn emit_binary(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Mod => {
                self.emit_arithmetic_binary(left, op, right, span)
            }
            BinaryOp::Div => Err(self.unsupported(span, ASSEMBLY_DIVISION_REJECTION)),
            BinaryOp::Concat => Err(self.unsupported(span, ASSEMBLY_CONCAT_REJECTION)),
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => self.emit_scalar_comparison(left, op, right, span),
            BinaryOp::StrictEq | BinaryOp::StrictNe => {
                self.emit_static_strict_identity(left, op, right, span)
            }
            BinaryOp::NullCoalesce => Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION)),
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::LogicalXor => {
                self.emit_bool_binary(left, op, right, span)
            }
            BinaryOp::BitwiseAnd | BinaryOp::BitwiseOr | BinaryOp::BitwiseXor => {
                self.emit_integer_bitwise_binary(left, op, right, span)
            }
            BinaryOp::ShiftLeft | BinaryOp::ShiftRight => {
                self.emit_integer_shift_binary(left, op, right, span)
            }
        }
    }

    fn emit_arithmetic_binary(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        let operator = match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Mod => "%",
            _ => return Err(self.unsupported(span, ASSEMBLY_ARITHMETIC_REJECTION)),
        };
        let direct_numeric_pair = matches!(
            (&left, &right),
            (CValue::Int(_), CValue::Int(_)) | (CValue::Float(_), CValue::Float(_))
        );
        if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) && !direct_numeric_pair {
            match self.checked_static_primitive_arithmetic_result_for_values(&left, op, &right) {
                Ok(Some(result)) => {
                    if let Some(value) = result.into_single_c_value() {
                        return Ok(value);
                    }
                }
                Err(PhpPrimitiveArithmeticError::IntegerOverflow) => {
                    return Err(
                        self.unsupported(span, ASSEMBLY_INTEGER_OVERFLOW_ARITHMETIC_REJECTION)
                    );
                }
                Err(PhpPrimitiveArithmeticError::NonFiniteFloat) => {
                    return Err(self.unsupported(span, ASSEMBLY_MIXED_NUMERIC_ARITHMETIC_REJECTION));
                }
                Ok(None)
                | Err(
                    PhpPrimitiveArithmeticError::MissingRightOperand
                    | PhpPrimitiveArithmeticError::Conversion(_),
                ) => {}
            }
        }

        match (left, right) {
            (CValue::Int(left), CValue::Int(right)) => {
                if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) {
                    if matches!(op, BinaryOp::Add) {
                        if right == "0" {
                            return Ok(CValue::Int(left));
                        }
                        if left == "0" {
                            return Ok(CValue::Int(right));
                        }
                    }
                    if matches!(op, BinaryOp::Sub) && right == "0" {
                        return Ok(CValue::Int(left));
                    }
                    if matches!(op, BinaryOp::Sub) && left == right {
                        return Ok(CValue::Int("0".to_string()));
                    }
                    if matches!(op, BinaryOp::Mul) {
                        if right == "0" || left == "0" {
                            return Ok(CValue::Int("0".to_string()));
                        }
                        if right == "1" {
                            return Ok(CValue::Int(left));
                        }
                        if left == "1" {
                            return Ok(CValue::Int(right));
                        }
                    }
                    let left_is_tracked = self.is_tracked_integer_value(&left);
                    let right_is_tracked = self.is_tracked_integer_value(&right);
                    let Some(result) = self.checked_static_integer_arithmetic(&left, op, &right)
                    else {
                        return Err(
                            self.unsupported(span, ASSEMBLY_INTEGER_OVERFLOW_ARITHMETIC_REJECTION)
                        );
                    };
                    if (left_is_tracked || right_is_tracked) && result.is_single() {
                        return Ok(CValue::Int(result.values()[0].to_string()));
                    }
                    let expression = format!("({left} {operator} {right})");
                    self.known_ints.insert(expression.clone(), result);
                    return Ok(CValue::Int(expression));
                }
                if matches!(op, BinaryOp::Mod) {
                    let Ok(divisor) = right.parse::<i64>() else {
                        return Err(self.unsupported(span, ASSEMBLY_MODULO_RUNTIME_CHECK_REJECTION));
                    };
                    if divisor <= 0 {
                        return Err(self.unsupported(span, ASSEMBLY_MODULO_RUNTIME_CHECK_REJECTION));
                    }
                    if divisor == 1 {
                        return Ok(CValue::Int("0".to_string()));
                    }
                    let modulo_result = self.static_integer_modulo(&left, divisor);
                    if let (Some(left_values), Some(result)) =
                        (self.known_integer_values(&left), modulo_result.as_ref())
                    {
                        if !left_values.is_single() && result.is_single() {
                            return Ok(CValue::Int(result.values()[0].to_string()));
                        }
                    }
                    let expression = format!("({left} {operator} {right})");
                    if let Some(result) = modulo_result {
                        self.known_ints.insert(expression.clone(), result);
                    }
                    return Ok(CValue::Int(expression));
                }
                Ok(CValue::Int(format!("({left} {operator} {right})")))
            }
            (CValue::Float(left), CValue::Float(right)) => {
                if matches!(op, BinaryOp::Mod) {
                    return Err(self.unsupported(span, ASSEMBLY_ARITHMETIC_REJECTION));
                }
                if matches!(op, BinaryOp::Add) {
                    if right == "0.0" && self.known_finite_nonzero_float_values(&left) {
                        return Ok(CValue::Float(left));
                    }
                    if left == "0.0" && self.known_finite_nonzero_float_values(&right) {
                        return Ok(CValue::Float(right));
                    }
                }
                if matches!(op, BinaryOp::Sub)
                    && right == "0.0"
                    && self.known_finite_nonzero_float_values(&left)
                {
                    return Ok(CValue::Float(left));
                }
                if matches!(op, BinaryOp::Sub) && left == "0.0" {
                    if let Some(result) = self.static_float_arithmetic(&left, op, &right) {
                        if result.is_single() && result.values()[0] != 0.0 {
                            return Ok(CValue::Float(format_float_literal(result.values()[0])));
                        }
                    }
                }
                if matches!(op, BinaryOp::Mul) {
                    if (right == "0.0" && self.known_finite_positive_float_values(&left))
                        || (left == "0.0" && self.known_finite_positive_float_values(&right))
                    {
                        return Ok(CValue::Float("0.0".to_string()));
                    }
                    if right == "-1.0" {
                        if let Some(result) = self.static_float_negate(&left) {
                            if result.is_single() && result.values()[0] != 0.0 {
                                return Ok(CValue::Float(format_float_literal(result.values()[0])));
                            }
                        }
                    }
                    if left == "-1.0" {
                        if let Some(result) = self.static_float_negate(&right) {
                            if result.is_single() && result.values()[0] != 0.0 {
                                return Ok(CValue::Float(format_float_literal(result.values()[0])));
                            }
                        }
                    }
                    if right == "1.0" && self.known_float_values(&left).is_some() {
                        return Ok(CValue::Float(left));
                    }
                    if left == "1.0" && self.known_float_values(&right).is_some() {
                        return Ok(CValue::Float(right));
                    }
                }
                if matches!(op, BinaryOp::Sub)
                    && left == right
                    && self
                        .known_float_values(&left)
                        .is_some_and(|values| values.values().iter().all(|value| value.is_finite()))
                {
                    return Ok(CValue::Float("0.0".to_string()));
                }
                let left_is_tracked = self.is_tracked_float_value(&left);
                let right_is_tracked = self.is_tracked_float_value(&right);
                if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul)
                    && (left_is_tracked || right_is_tracked)
                {
                    if let Some(result) = self.static_float_arithmetic(&left, op, &right) {
                        if result.is_single() && result.values()[0] != 0.0 {
                            return Ok(CValue::Float(format_float_literal(result.values()[0])));
                        }
                    }
                }
                let expression = format!("({left} {operator} {right})");
                if let Some(result) = self.static_float_arithmetic(&left, op, &right) {
                    self.known_floats.insert(expression.clone(), result);
                }
                Ok(CValue::Float(expression))
            }
            (CValue::Int(_), CValue::Float(_)) | (CValue::Float(_), CValue::Int(_))
                if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) =>
            {
                Err(self.unsupported(span, ASSEMBLY_MIXED_NUMERIC_ARITHMETIC_REJECTION))
            }
            (
                CValue::Null
                | CValue::Bool(_)
                | CValue::BoolExpr(_)
                | CValue::ComparisonDecision(_)
                | CValue::String(_)
                | CValue::StringExpr(_),
                _,
            )
            | (
                _,
                CValue::Null
                | CValue::Bool(_)
                | CValue::BoolExpr(_)
                | CValue::ComparisonDecision(_)
                | CValue::String(_)
                | CValue::StringExpr(_),
            ) if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) => {
                Err(self.unsupported(span, ASSEMBLY_SCALAR_COERCION_ARITHMETIC_REJECTION))
            }
            _ => Err(self.unsupported(span, ASSEMBLY_ARITHMETIC_REJECTION)),
        }
    }

    fn emit_scalar_comparison_expr(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
        span: Span,
    ) -> CompileResult<CValue> {
        let left = match self.emit_expr(left) {
            Ok(value) => value,
            Err(_) => {
                let fallback = self.unsupported_value_operand_or_fallback(
                    left,
                    span,
                    assembly_comparison_rejection(),
                );
                return Err(
                    self.unsupported_unemitted_value_operands_or_original(&[left, right], fallback)
                );
            }
        };
        let right = match self.emit_expr(right) {
            Ok(value) => value,
            Err(_) => {
                return Err(self.unsupported_value_operand_or_fallback(
                    right,
                    span,
                    assembly_comparison_rejection(),
                ));
            }
        };
        self.emit_scalar_comparison(left, op, right, span)
    }

    fn emit_scalar_comparison(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        let compares_array_handles = matches!(
            (&left, &right),
            (CValue::ArrayHandle(_), CValue::ArrayHandle(_))
        );
        if self.uses_native_string_helpers || compares_array_handles {
            return self.emit_native_runtime_comparison(left, op, right, span);
        }

        match (left, right) {
            (CValue::Null, CValue::Null) => {
                let Some(result) = null_comparison_result(op) else {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                };
                Ok(CValue::Bool(result))
            }
            (CValue::Bool(left), CValue::Bool(right)) => {
                let Some(result) = bool_comparison_result(left, op, right) else {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                };
                Ok(CValue::Bool(result))
            }
            (CValue::BoolExpr(left), CValue::Bool(right)) => {
                let right = if right { "1" } else { "0" };
                self.emit_bool_scalar_comparison(left, op, right.to_string(), span)
            }
            (CValue::Bool(left), CValue::BoolExpr(right)) => {
                let left = if left { "1" } else { "0" };
                self.emit_bool_scalar_comparison(left.to_string(), op, right, span)
            }
            (CValue::BoolExpr(left), CValue::BoolExpr(right)) => {
                self.emit_bool_scalar_comparison(left, op, right, span)
            }
            (CValue::String(left), CValue::String(right)) => {
                let Some(result) = static_safe_string_comparison_result(
                    Some(KnownString::one(left)),
                    op,
                    Some(KnownString::one(right)),
                ) else {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                };
                Ok(CValue::Bool(result))
            }
            (CValue::StringExpr(left), CValue::StringExpr(right)) => {
                self.emit_string_comparison(left, op, right, span)
            }
            (CValue::StringExpr(left), CValue::String(right)) => {
                let right_operand = c_string_operand(CValue::String(right.clone()));
                self.known_strings
                    .insert(right_operand.clone(), KnownString::one(right));
                self.emit_string_comparison(left, op, right_operand, span)
            }
            (CValue::String(left), CValue::StringExpr(right)) => {
                let left_operand = c_string_operand(CValue::String(left.clone()));
                self.known_strings
                    .insert(left_operand.clone(), KnownString::one(left));
                self.emit_string_comparison(left_operand, op, right, span)
            }
            (CValue::Int(left), CValue::Int(right)) => {
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<i64>(), right.parse::<i64>())
                {
                    let Some(result) = integer_comparison_result(left_literal, op, right_literal)
                    else {
                        return Err(self.unsupported(span, assembly_comparison_rejection()));
                    };
                    return Ok(CValue::Bool(result));
                }
                if left == right {
                    let Some(result) = integer_comparison_result(0, op, 0) else {
                        return Err(self.unsupported(span, assembly_comparison_rejection()));
                    };
                    return Ok(CValue::Bool(result));
                }
                let left_is_tracked = self.is_tracked_integer_value(&left);
                let right_is_tracked = self.is_tracked_integer_value(&right);
                if left_is_tracked != right_is_tracked
                    && (left.parse::<i64>().is_ok() || right.parse::<i64>().is_ok())
                {
                    let tracked = if left_is_tracked { &left } else { &right };
                    if self
                        .known_integer_values(tracked)
                        .is_some_and(|values| values.is_single())
                    {
                        if let Some(result) =
                            self.static_integer_comparison_result(&left, op, &right)
                        {
                            return Ok(CValue::Bool(result));
                        }
                    }
                }
                if let Some(result) = self.static_integer_comparison_result(&left, op, &right) {
                    return Ok(CValue::Bool(result));
                }
                let operator = c_comparison_operator(op)
                    .ok_or_else(|| self.unsupported(span, assembly_comparison_rejection()))?;
                let expression = format!("(({left}) {operator} ({right}))");
                if let Some(result) = self.static_integer_comparison_result(&left, op, &right) {
                    self.known_bools
                        .insert(expression.clone(), KnownBool::one(result));
                }
                Ok(CValue::BoolExpr(expression))
            }
            (CValue::Float(left), CValue::Float(right)) => {
                let Some(left_values) = self.known_float_values(&left) else {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                };
                let Some(right_values) = self.known_float_values(&right) else {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                };
                if !left_values.values().iter().all(|value| value.is_finite())
                    || !right_values.values().iter().all(|value| value.is_finite())
                {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                }
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<f64>(), right.parse::<f64>())
                {
                    let Some(result) = float_comparison_result(left_literal, op, right_literal)
                    else {
                        return Err(self.unsupported(span, assembly_comparison_rejection()));
                    };
                    return Ok(CValue::Bool(result));
                }
                let left_is_tracked = self.is_tracked_float_value(&left);
                let right_is_tracked = self.is_tracked_float_value(&right);
                if left_is_tracked != right_is_tracked
                    && (left.parse::<f64>().is_ok() || right.parse::<f64>().is_ok())
                {
                    let tracked = if left_is_tracked { &left } else { &right };
                    if self
                        .known_float_values(tracked)
                        .is_some_and(|values| values.is_single())
                    {
                        if let Some(result) = self.static_float_comparison_result(&left, op, &right)
                        {
                            return Ok(CValue::Bool(result));
                        }
                    }
                }
                if let Some(result) = self.static_float_comparison_result(&left, op, &right) {
                    return Ok(CValue::Bool(result));
                }
                let operator = c_comparison_operator(op)
                    .ok_or_else(|| self.unsupported(span, assembly_comparison_rejection()))?;
                let expression = format!("(({left}) {operator} ({right}))");
                if let Some(result) = self.static_float_comparison_result(&left, op, &right) {
                    self.known_bools
                        .insert(expression.clone(), KnownBool::one(result));
                }
                Ok(CValue::BoolExpr(expression))
            }
            _ => Err(self.unsupported(span, assembly_comparison_rejection())),
        }
    }

    fn emit_bool_scalar_comparison(
        &mut self,
        left: String,
        op: BinaryOp,
        right: String,
        span: Span,
    ) -> CompileResult<CValue> {
        if let Some(fold) = bool_literal_comparison_fold(&left, op, &right, "1", "0") {
            return match fold {
                BoolLiteralComparisonFold::Static(value) => Ok(CValue::Bool(value)),
                BoolLiteralComparisonFold::Reuse(value) => Ok(CValue::BoolExpr(value)),
                BoolLiteralComparisonFold::Invert(value) => {
                    self.emit_bool_not(CValue::BoolExpr(value), span)
                }
            };
        }
        if left == right {
            let Some(result) = bool_comparison_result(false, op, false) else {
                return Err(self.unsupported(span, assembly_comparison_rejection()));
            };
            return Ok(CValue::Bool(result));
        }
        if let Some(result) = self.static_bool_comparison_result(&left, op, &right) {
            return Ok(CValue::Bool(result));
        }
        let operator = c_comparison_operator(op)
            .ok_or_else(|| self.unsupported(span, assembly_comparison_rejection()))?;
        let expression = format!("(({left}) {operator} ({right}))");
        if let Some(result) = self.static_bool_comparison_result(&left, op, &right) {
            self.known_bools
                .insert(expression.clone(), KnownBool::one(result));
        }
        Ok(CValue::BoolExpr(expression))
    }

    fn emit_native_runtime_comparison(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        let Some(op) = native_comparison_op_for_binary_op(op) else {
            return Err(self.unsupported(span, assembly_comparison_rejection()));
        };

        match (left, right) {
            (CValue::ArrayHandle(left), CValue::ArrayHandle(right)) => {
                Ok(self.emit_native_array_handle_comparison(left, op, right))
            }
            (CValue::ArrayHandle(_), _) | (_, CValue::ArrayHandle(_)) => {
                Err(self.unsupported(span, assembly_comparison_rejection()))
            }
            (left, right) => {
                self.uses_native_comparison_helpers = true;
                let comparison_index = self.next_native_temp;
                self.next_native_temp += 1;
                let left = self.emit_native_comparison_operand(left, span)?;
                let right = self.emit_native_comparison_operand(right, span)?;
                let comparison_operation = format!("comparison_operation_{comparison_index}");
                let comparison_relation = format!("comparison_relation_{comparison_index}");
                let comparison_decision = format!("comparison_decision_{comparison_index}");

                self.body.push(format!(
                    "phpc_NativeComparisonOperation {comparison_operation} = phpc_native_comparison_operation_from_opcode({});",
                    native_comparison_c_uint8_argument(op)
                ));
                self.body.push(format!(
                    "phpc_NativeComparisonRelationResult {comparison_relation} = phpc_native_comparison_operand_compare_operation_relation_and_free({}, {comparison_operation}, {});",
                    left.operand, right.operand
                ));
                self.body.push(format!(
                    "phpc_NativeComparisonBranchDecision {comparison_decision} = phpc_native_comparison_relation_result_decision_or_report_stderr_and_free({comparison_relation}, {comparison_operation});"
                ));
                let comparison_abort_code = self.emit_native_comparison_decision_abort_code(
                    comparison_index,
                    &comparison_decision,
                );
                self.body
                    .push(format!("if ({comparison_abort_code} != 0) {{"));
                self.body.push(format!("  return {comparison_abort_code};"));
                self.body.push("}".to_string());

                Ok(CValue::ComparisonDecision(comparison_decision))
            }
        }
    }

    fn emit_native_comparison_branch_decision(
        &mut self,
        comparison_index: usize,
        comparison_branch: &str,
    ) -> (String, String) {
        let comparison_decision = format!("comparison_decision_{comparison_index}");
        self.body.push(format!(
            "phpc_NativeComparisonBranchDecision {comparison_decision} = phpc_native_comparison_branch_decision_from_result({comparison_branch});"
        ));
        let comparison_abort_code =
            self.emit_native_comparison_decision_abort_code(comparison_index, &comparison_decision);
        (comparison_decision, comparison_abort_code)
    }

    fn emit_native_comparison_decision_abort_code(
        &mut self,
        comparison_index: usize,
        comparison_decision: &str,
    ) -> String {
        let comparison_abort_code = format!("comparison_abort_code_{comparison_index}");
        self.body.push(format!(
            "int {comparison_abort_code} = phpc_native_comparison_branch_decision_abort_code({comparison_decision});"
        ));
        comparison_abort_code
    }

    fn emit_native_array_handle_comparison(
        &mut self,
        left: String,
        op: NativeComparisonOp,
        right: String,
    ) -> CValue {
        self.uses_native_array_helpers = true;
        self.uses_native_comparison_helpers = true;
        self.uses_native_array_comparison_helpers = true;
        let comparison_index = self.next_native_temp;
        self.next_native_temp += 1;
        let comparison_branch = format!("array_comparison_branch_{comparison_index}");

        self.body.push(format!(
            "phpc_NativeComparisonBranchResult {comparison_branch} = phpc_native_array_compare_branch({left}, {}, {right});",
            native_comparison_c_uint8_argument(op)
        ));
        let (comparison_decision, comparison_abort_code) =
            self.emit_native_comparison_branch_decision(comparison_index, &comparison_branch);
        self.body
            .push(format!("if ({comparison_abort_code} != 0) {{"));
        self.body.push(format!(
            "  {}",
            self.native_error_exit_with_code("", &comparison_abort_code)
        ));
        self.body.push("}".to_string());

        CValue::ComparisonDecision(comparison_decision)
    }

    fn emit_native_comparison_operand(
        &mut self,
        value: CValue,
        span: Span,
    ) -> CompileResult<NativeCComparisonOperand> {
        let index = self.next_native_temp;
        self.next_native_temp += 1;
        let operand = format!("comparison_operand_{index}");

        match value {
            CValue::Null => {
                self.body.push(format!(
                    "phpc_NativeComparisonOperand {operand} = phpc_native_comparison_operand_from_scalar((phpc_NativeScalarValue){{0}});"
                ));
                Ok(NativeCComparisonOperand { operand })
            }
            CValue::Bool(value) => {
                let bool_value = u8::from(value);
                self.body.push(format!(
                    "phpc_NativeComparisonOperand {operand} = phpc_native_comparison_operand_from_scalar((phpc_NativeScalarValue){{1, {bool_value}, 0, 0.0}});"
                ));
                Ok(NativeCComparisonOperand { operand })
            }
            CValue::BoolExpr(value) => {
                self.body.push(format!(
                    "phpc_NativeComparisonOperand {operand} = phpc_native_comparison_operand_from_scalar((phpc_NativeScalarValue){{1, (uint8_t)(({value}) != 0), 0, 0.0}});"
                ));
                Ok(NativeCComparisonOperand { operand })
            }
            CValue::ComparisonDecision(decision) => {
                self.body.push(format!(
                    "phpc_NativeComparisonOperand {operand} = phpc_native_comparison_branch_decision_result_operand({decision});"
                ));
                Ok(NativeCComparisonOperand { operand })
            }
            CValue::Int(value) => {
                self.body.push(format!(
                    "phpc_NativeComparisonOperand {operand} = phpc_native_comparison_operand_from_scalar((phpc_NativeScalarValue){{2, 0, (int64_t)({value}), 0.0}});"
                ));
                Ok(NativeCComparisonOperand { operand })
            }
            CValue::Float(value) => {
                self.body.push(format!(
                    "phpc_NativeComparisonOperand {operand} = phpc_native_comparison_operand_from_scalar((phpc_NativeScalarValue){{3, 0, 0, (double)({value})}});"
                ));
                Ok(NativeCComparisonOperand { operand })
            }
            CValue::String(value) => {
                let string = format!("comparison_string_{index}");
                let (bytes, byte_len) = if value.is_empty() {
                    ("NULL".to_string(), "0".to_string())
                } else {
                    let bytes = c_byte_array(value.as_bytes());
                    let data = format!("phpc_native_comparison_bytes_{index}");
                    self.static_data
                        .push(format!("static const uint8_t {data}[] = {{{bytes}}};"));
                    (data, value.len().to_string())
                };
                self.body.push(format!(
                    "phpc_NativeStringHandle {string} = phpc_native_string_from_bytes({bytes}, {byte_len});"
                ));
                self.body.push(format!(
                    "phpc_NativeComparisonOperand {operand} = phpc_native_comparison_operand_from_string_and_free({string});"
                ));
                Ok(NativeCComparisonOperand { operand })
            }
            CValue::StringExpr(value) => {
                let byte_len = self.native_comparison_string_expr_len_operand(&value, span)?;
                let string = format!("comparison_string_{index}");
                self.body.push(format!(
                    "phpc_NativeStringHandle {string} = phpc_native_string_from_bytes((const uint8_t *)({value}), {byte_len});"
                ));
                self.body.push(format!(
                    "phpc_NativeComparisonOperand {operand} = phpc_native_comparison_operand_from_string_and_free({string});"
                ));
                Ok(NativeCComparisonOperand { operand })
            }
            CValue::ArrayHandle(_) | CValue::NativeValueHandle(_) => {
                Err(self.unsupported(span, assembly_comparison_rejection()))
            }
        }
    }

    fn emit_native_value_for_cvalue(&mut self, value: CValue, span: Span) -> CompileResult<String> {
        let index = self.next_native_temp;
        self.next_native_temp += 1;
        let value_handle = format!("native_value_handle_{index}");

        match value {
            CValue::Null => {
                self.body.push(format!(
                    "phpc_NativeValueHandle {value_handle} = phpc_native_value_from_scalar((phpc_NativeScalarValue){{0}});"
                ));
            }
            CValue::Bool(value) => {
                let bool_value = u8::from(value);
                self.body.push(format!(
                    "phpc_NativeValueHandle {value_handle} = phpc_native_value_from_scalar((phpc_NativeScalarValue){{1, {bool_value}, 0, 0.0}});"
                ));
            }
            CValue::BoolExpr(value) => {
                self.body.push(format!(
                    "phpc_NativeValueHandle {value_handle} = phpc_native_value_from_scalar((phpc_NativeScalarValue){{1, (uint8_t)(({value}) != 0), 0, 0.0}});"
                ));
            }
            CValue::ComparisonDecision(decision) => {
                self.body.push(format!(
                    "phpc_NativeValueHandle {value_handle} = phpc_native_value_from_scalar((phpc_NativeScalarValue){{1, (uint8_t)(phpc_native_comparison_branch_decision_is_true({decision}) != 0), 0, 0.0}});"
                ));
            }
            CValue::Int(value) => {
                self.body.push(format!(
                    "phpc_NativeValueHandle {value_handle} = phpc_native_value_from_scalar((phpc_NativeScalarValue){{2, 0, (int64_t)({value}), 0.0}});"
                ));
            }
            CValue::Float(value) => {
                self.body.push(format!(
                    "phpc_NativeValueHandle {value_handle} = phpc_native_value_from_scalar((phpc_NativeScalarValue){{3, 0, 0, (double)({value})}});"
                ));
            }
            CValue::String(value) => {
                let diagnostic_handle = format!("native_value_diagnostic_{index}");
                let (bytes, byte_len) = if value.is_empty() {
                    ("NULL".to_string(), "0".to_string())
                } else {
                    let bytes = c_byte_array(value.as_bytes());
                    let data = format!("phpc_native_value_bytes_{index}");
                    self.static_data
                        .push(format!("static const uint8_t {data}[] = {{{bytes}}};"));
                    (data, value.len().to_string())
                };
                self.body.push(format!(
                    "phpc_NativeDiagnosticHandle {diagnostic_handle} = {{0}};"
                ));
                self.body.push(format!(
                    "phpc_NativeValueHandle {value_handle} = phpc_native_value_from_string_bytes_with_diagnostic({bytes}, {byte_len}, &{diagnostic_handle});"
                ));
                self.emit_report_native_diagnostic(&diagnostic_handle);
            }
            CValue::StringExpr(value) => {
                let Some(byte_len) = self.c_string_expr_byte_len_operand(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
                };
                let diagnostic_handle = format!("native_value_diagnostic_{index}");
                self.body.push(format!(
                    "phpc_NativeDiagnosticHandle {diagnostic_handle} = {{0}};"
                ));
                self.body.push(format!(
                    "phpc_NativeValueHandle {value_handle} = phpc_native_value_from_string_bytes_with_diagnostic((const uint8_t *)({value}), {byte_len}, &{diagnostic_handle});"
                ));
                self.emit_report_native_diagnostic(&diagnostic_handle);
            }
            CValue::ArrayHandle(handle) => {
                self.uses_native_array_helpers = true;
                self.body.push(format!(
                    "phpc_NativeValueHandle {value_handle} = phpc_native_value_from_array({handle});"
                ));
            }
            CValue::NativeValueHandle(handle) => {
                self.uses_native_value_clone = true;
                self.body.push(format!(
                    "phpc_NativeValueHandle {value_handle} = phpc_native_value_clone({handle});"
                ));
            }
        }

        Ok(value_handle)
    }

    fn emit_native_value_string_byte_len(
        &mut self,
        value: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        let value_handle = self.emit_native_value_for_cvalue(value, span)?;
        let conversion = format!("string_conversion_{}", self.next_native_temp);
        self.next_native_temp += 1;
        let byte_count = format!("byte_count_{}", self.next_native_temp);
        self.next_native_temp += 1;

        self.body.push(format!(
            "phpc_NativeStringConversionResult {conversion} = phpc_native_value_to_string_bytes({value_handle});"
        ));
        self.body.push(format!(
            "if ({conversion}.diagnostic.ptr != NULL) {{ phpc_native_diagnostic_message_stderr({conversion}.diagnostic); }}"
        ));
        self.body.push(format!(
            "long long {byte_count} = (long long){conversion}.bytes.len;"
        ));
        self.body.push(format!(
            "phpc_native_string_conversion_result_free({conversion});"
        ));
        self.body
            .push(format!("phpc_native_value_free({value_handle});"));

        Ok(CValue::Int(byte_count))
    }

    fn emit_report_native_diagnostic(&mut self, diagnostic: &str) {
        self.body.push(format!(
            "if ({diagnostic}.ptr != NULL) {{ phpc_native_diagnostic_report({diagnostic}); {diagnostic}.ptr = NULL; }}"
        ));
    }

    fn native_comparison_string_expr_len_operand(
        &mut self,
        value: &str,
        span: Span,
    ) -> CompileResult<String> {
        if let Some(length) = self.known_string_lengths.get(value) {
            return Ok(length.clone());
        }

        let Some(values) = self.known_string_values(value) else {
            return Err(self.unsupported(span, assembly_comparison_rejection()));
        };

        if let Some(byte_len) = known_strings_have_uniform_byte_length(&values) {
            return Ok(byte_len.to_string());
        }

        if known_strings_are_nul_free(&values) {
            self.uses_strcmp = true;
            return Ok(format!("strlen((const char *)({value}))"));
        }

        Err(self.unsupported(span, assembly_comparison_rejection()))
    }

    fn checked_static_integer_arithmetic(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<KnownInt> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                let result = match op {
                    BinaryOp::Add => left.checked_add(*right),
                    BinaryOp::Sub => left.checked_sub(*right),
                    BinaryOp::Mul => left.checked_mul(*right),
                    _ => None,
                }?;
                results.push(result);
            }
        }
        KnownInt::from_values(results)
    }

    fn known_integer_values(&self, value: &str) -> Option<KnownInt> {
        value
            .parse::<i64>()
            .ok()
            .map(KnownInt::one)
            .or_else(|| self.known_ints.get(value).cloned())
    }

    fn is_tracked_integer_value(&self, value: &str) -> bool {
        self.known_ints.contains_key(value)
    }

    fn static_integer_modulo(&self, left: &str, divisor: i64) -> Option<KnownInt> {
        let left = self.known_integer_values(left)?;
        let values = left.values().iter().map(|value| value % divisor);
        KnownInt::from_values(values)
    }

    fn static_integer_comparison_result(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = integer_comparison_result(*left, op, *right)?;
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_float_comparison_result(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_float_values(left)?;
        let right_values = self.known_float_values(right)?;
        if !left_values.values().iter().all(|value| value.is_finite())
            || !right_values.values().iter().all(|value| value.is_finite())
        {
            return None;
        }
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = float_comparison_result(*left, op, *right)?;
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_bool_comparison_result(&self, left: &str, op: BinaryOp, right: &str) -> Option<bool> {
        let left_values = self.known_bool_values(left)?;
        let right_values = self.known_bool_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = bool_comparison_result(*left, op, *right)?;
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn emit_integer_shift_binary(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        let (CValue::Int(left), CValue::Int(right)) = (left, right) else {
            return Err(self.unsupported(span, ASSEMBLY_BITWISE_REJECTION));
        };
        let Some(count) = self.static_integer_shift_count(&right) else {
            return Err(self.unsupported(span, ASSEMBLY_BITWISE_REJECTION));
        };
        if count == 0 {
            return Ok(CValue::Int(left));
        }
        if self.is_tracked_integer_value(&left) {
            if let Some(result) = self.static_integer_shift(&left, op, count) {
                if result.is_single() {
                    return Ok(CValue::Int(result.values()[0].to_string()));
                }
            }
        }
        let operator = match op {
            BinaryOp::ShiftLeft => "<<",
            BinaryOp::ShiftRight => ">>",
            _ => return Err(self.unsupported(span, ASSEMBLY_BITWISE_REJECTION)),
        };
        let expression = format!("({left} {operator} {count})");
        if let Some(result) = self.static_integer_shift(&left, op, count) {
            self.known_ints.insert(expression.clone(), result);
        }
        Ok(CValue::Int(expression))
    }

    fn static_integer_shift(&self, left: &str, op: BinaryOp, count: u32) -> Option<KnownInt> {
        let left = self.known_integer_values(left)?;
        let factor = if matches!(op, BinaryOp::ShiftLeft) {
            Some(1_i64.checked_shl(count)?)
        } else {
            None
        };
        let values = left.values().iter().map(|value| match op {
            BinaryOp::ShiftLeft => value.checked_mul(factor.expect("left shift has a factor")),
            BinaryOp::ShiftRight => Some(value >> count),
            _ => None,
        });
        let mut results = Vec::new();
        for value in values {
            results.push(value?);
        }
        KnownInt::from_values(results)
    }

    fn static_integer_shift_count(&self, right: &str) -> Option<u32> {
        if let Ok(count) = right.parse::<u32>() {
            return (count < 64).then_some(count);
        }
        let values = self.known_integer_values(right)?;
        if !values.is_single() {
            return None;
        }
        let count = u32::try_from(values.values()[0]).ok()?;
        (count < 64).then_some(count)
    }

    fn emit_integer_bitwise_binary(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        let (CValue::Int(left), CValue::Int(right)) = (left, right) else {
            return Err(self.unsupported(span, ASSEMBLY_BITWISE_REJECTION));
        };
        if left == right {
            return Ok(match op {
                BinaryOp::BitwiseAnd | BinaryOp::BitwiseOr => CValue::Int(left),
                BinaryOp::BitwiseXor => CValue::Int("0".to_string()),
                _ => return Err(self.unsupported(span, ASSEMBLY_BITWISE_REJECTION)),
            });
        }
        if matches!(op, BinaryOp::BitwiseAnd) {
            if self
                .known_integer_values(&right)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(CValue::Int("0".to_string()));
            }
            if self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(CValue::Int("0".to_string()));
            }
            if self
                .known_integer_values(&right)
                .is_some_and(|values| values.is_single_value(-1))
            {
                return Ok(CValue::Int(left));
            }
            if self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(-1))
            {
                return Ok(CValue::Int(right));
            }
        }
        if matches!(op, BinaryOp::BitwiseOr | BinaryOp::BitwiseXor) {
            if self
                .known_integer_values(&right)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(CValue::Int(left));
            }
            if self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(CValue::Int(right));
            }
        }
        if matches!(op, BinaryOp::BitwiseOr)
            && (self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(-1))
                || self
                    .known_integer_values(&right)
                    .is_some_and(|values| values.is_single_value(-1)))
        {
            return Ok(CValue::Int("-1".to_string()));
        }
        if matches!(op, BinaryOp::BitwiseXor)
            && ((self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(-1))
                && self
                    .known_integer_values(&right)
                    .is_some_and(|values| values.is_single()))
                || (self
                    .known_integer_values(&right)
                    .is_some_and(|values| values.is_single_value(-1))
                    && self
                        .known_integer_values(&left)
                        .is_some_and(|values| values.is_single())))
        {
            let result = self
                .static_integer_bitwise(&left, op, &right)
                .expect("single known integer XOR all-ones result is known");
            return Ok(CValue::Int(result.values()[0].to_string()));
        }
        let left_is_tracked = self.is_tracked_integer_value(&left);
        let right_is_tracked = self.is_tracked_integer_value(&right);
        if left_is_tracked || right_is_tracked {
            if let Some(result) = self.static_integer_bitwise(&left, op, &right) {
                if result.is_single() {
                    return Ok(CValue::Int(result.values()[0].to_string()));
                }
            }
        }
        let operator = match op {
            BinaryOp::BitwiseAnd => "&",
            BinaryOp::BitwiseOr => "|",
            BinaryOp::BitwiseXor => "^",
            _ => return Err(self.unsupported(span, ASSEMBLY_BITWISE_REJECTION)),
        };
        let expression = format!("({left} {operator} {right})");
        if let Some(result) = self.static_integer_bitwise(&left, op, &right) {
            self.known_ints.insert(expression.clone(), result);
        }
        Ok(CValue::Int(expression))
    }

    fn static_integer_bitwise(&self, left: &str, op: BinaryOp, right: &str) -> Option<KnownInt> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                results.push(match op {
                    BinaryOp::BitwiseAnd => left & right,
                    BinaryOp::BitwiseOr => left | right,
                    BinaryOp::BitwiseXor => left ^ right,
                    _ => return None,
                });
            }
        }
        KnownInt::from_values(results)
    }

    fn static_integer_negate(&self, value: &str) -> Option<KnownInt> {
        let value = self.known_integer_values(value)?;
        let mut results = Vec::new();
        for value in value.values() {
            results.push(value.checked_neg()?);
        }
        KnownInt::from_values(results)
    }

    fn static_integer_bitwise_not(&self, value: &str) -> Option<KnownInt> {
        let value = self.known_integer_values(value)?;
        KnownInt::from_values(value.values().iter().map(|value| !value))
    }

    fn emit_static_string_concat_expr(
        &mut self,
        left: &Expr,
        right: &Expr,
        span: Span,
    ) -> CompileResult<CValue> {
        if is_empty_string_literal(left) {
            let right = match self.emit_expr(right) {
                Ok(value) => value,
                Err(error) => return Err(self.unsupported_value_operand_or_original(right, error)),
            };
            return self.emit_empty_string_concat_identity(right, span);
        }
        if is_empty_string_literal(right) {
            let left = match self.emit_expr(left) {
                Ok(value) => value,
                Err(error) => return Err(self.unsupported_value_operand_or_original(left, error)),
            };
            return self.emit_empty_string_concat_identity(left, span);
        }
        let left = match self.emit_static_string_concat_operand(left, span) {
            Ok(value) => value,
            Err(error) => {
                return Err(
                    self.unsupported_unemitted_value_operands_or_original(&[left, right], error)
                );
            }
        };
        let right = self.emit_static_string_concat_operand(right, span)?;
        Ok(CValue::String(format!("{left}{right}")))
    }

    fn emit_empty_string_concat_identity(
        &self,
        value: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        match value {
            CValue::String(_) | CValue::StringExpr(_) => Ok(value),
            _ => Err(self.unsupported(span, ASSEMBLY_CONCAT_REJECTION)),
        }
    }

    fn emit_static_string_concat_operand(
        &mut self,
        expr: &Expr,
        span: Span,
    ) -> CompileResult<String> {
        match expr {
            Expr::String(value, _) => Ok(value.clone()),
            Expr::Variable(name, variable_span) => {
                if is_request_superglobal_name(name) {
                    return Err(
                        self.unsupported(*variable_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                match self.variables.get(name).cloned() {
                    Some(CValue::String(value)) => Ok(value),
                    Some(_) => Err(self.unsupported(span, ASSEMBLY_CONCAT_REJECTION)),
                    None => Err(self.unsupported(*variable_span, ASSEMBLY_VARIABLE_READ_REJECTION)),
                }
            }
            Expr::Binary {
                left,
                op: BinaryOp::Concat,
                right,
                span: concat_span,
            } => match self.emit_static_string_concat_expr(left, right, *concat_span)? {
                CValue::String(value) => Ok(value),
                _ => unreachable!("static string concatenation returns a string"),
            },
            Expr::Ternary { .. } => match self.emit_expr(expr)? {
                CValue::String(value) => Ok(value),
                CValue::StringExpr(value) => {
                    let values = self
                        .known_string_values(&value)
                        .ok_or_else(|| self.unsupported(span, ASSEMBLY_CONCAT_REJECTION))?;
                    if values.is_single() {
                        Ok(values.values()[0].clone())
                    } else {
                        Err(self.unsupported(span, ASSEMBLY_CONCAT_REJECTION))
                    }
                }
                _ => Err(self.unsupported(span, ASSEMBLY_CONCAT_REJECTION)),
            },
            _ => Err(self.unsupported_value_operand_or_fallback(
                expr,
                span,
                ASSEMBLY_CONCAT_REJECTION,
            )),
        }
    }

    fn emit_static_strict_identity(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        let is_identical = match (left, right) {
            (CValue::Null, CValue::Null) => true,
            (CValue::Bool(left), CValue::Bool(right)) => left == right,
            (CValue::BoolExpr(left), CValue::Bool(right)) => {
                if let Some(result) = self.static_bool_strict_identity(
                    self.known_bool_values(&left),
                    Some(KnownBool::one(right)),
                    op,
                ) {
                    return Ok(CValue::Bool(result));
                }
                if matches!(
                    (op, right),
                    (BinaryOp::StrictEq, true) | (BinaryOp::StrictNe, false)
                ) {
                    return Ok(CValue::BoolExpr(left));
                }
                if matches!(
                    (op, right),
                    (BinaryOp::StrictEq, false) | (BinaryOp::StrictNe, true)
                ) {
                    return self.emit_bool_not(CValue::BoolExpr(left), span);
                }
                let right = if right { "1" } else { "0" };
                return self.emit_bool_comparison(left, op, right.to_string(), span);
            }
            (CValue::Bool(left), CValue::BoolExpr(right)) => {
                if let Some(result) = self.static_bool_strict_identity(
                    Some(KnownBool::one(left)),
                    self.known_bool_values(&right),
                    op,
                ) {
                    return Ok(CValue::Bool(result));
                }
                if matches!(
                    (op, left),
                    (BinaryOp::StrictEq, true) | (BinaryOp::StrictNe, false)
                ) {
                    return Ok(CValue::BoolExpr(right));
                }
                if matches!(
                    (op, left),
                    (BinaryOp::StrictEq, false) | (BinaryOp::StrictNe, true)
                ) {
                    return self.emit_bool_not(CValue::BoolExpr(right), span);
                }
                let left = if left { "1" } else { "0" };
                return self.emit_bool_comparison(left.to_string(), op, right, span);
            }
            (CValue::BoolExpr(left), CValue::BoolExpr(right)) => {
                if left == right {
                    return Ok(CValue::Bool(static_strict_identity_result(true, op)));
                }
                if let Some(result) = self.static_bool_strict_identity(
                    self.known_bool_values(&left),
                    self.known_bool_values(&right),
                    op,
                ) {
                    return Ok(CValue::Bool(result));
                }
                return self.emit_bool_comparison(left, op, right, span);
            }
            (CValue::ComparisonDecision(left), right) => {
                let Some(right) = c_bool_operand(right) else {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                };
                return self.emit_bool_comparison(
                    c_comparison_decision_bool_expr(&left),
                    op,
                    right,
                    span,
                );
            }
            (left, CValue::ComparisonDecision(right)) => {
                let Some(left) = c_bool_operand(left) else {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                };
                return self.emit_bool_comparison(
                    left,
                    op,
                    c_comparison_decision_bool_expr(&right),
                    span,
                );
            }
            (CValue::String(left), CValue::String(right)) => left == right,
            (CValue::StringExpr(left), CValue::StringExpr(right)) => {
                if left == right {
                    return Ok(CValue::Bool(static_strict_identity_result(true, op)));
                }
                if let Some(result) = self.static_string_strict_identity(
                    self.known_string_values(&left),
                    self.known_string_values(&right),
                    op,
                ) {
                    return Ok(CValue::Bool(result));
                }
                return self.emit_string_comparison(left, op, right, span);
            }
            (CValue::StringExpr(left), CValue::String(right)) => {
                if let Some(result) = self.static_string_strict_identity(
                    self.known_string_values(&left),
                    Some(KnownString::one(right.clone())),
                    op,
                ) {
                    return Ok(CValue::Bool(result));
                }
                let right = c_string_operand(CValue::String(right));
                return self.emit_string_comparison(left, op, right, span);
            }
            (CValue::String(left), CValue::StringExpr(right)) => {
                if let Some(result) = self.static_string_strict_identity(
                    Some(KnownString::one(left.clone())),
                    self.known_string_values(&right),
                    op,
                ) {
                    return Ok(CValue::Bool(result));
                }
                let left = c_string_operand(CValue::String(left));
                return self.emit_string_comparison(left, op, right, span);
            }
            (CValue::ArrayHandle(left), CValue::ArrayHandle(right)) => {
                let Some(op) = native_comparison_op_for_binary_op(op) else {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                };
                return Ok(self.emit_native_array_handle_comparison(left, op, right));
            }
            (CValue::Float(left), CValue::Float(right)) => {
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<f64>(), right.parse::<f64>())
                {
                    return Ok(CValue::Bool(match op {
                        BinaryOp::StrictEq => left_literal == right_literal,
                        BinaryOp::StrictNe => left_literal != right_literal,
                        _ => return Err(self.unsupported(span, assembly_comparison_rejection())),
                    }));
                }
                if left == right {
                    if let Some(result) = self.static_same_float_strict_identity(&left, op) {
                        return Ok(CValue::Bool(result));
                    }
                }
                if let Some(result) = self.static_float_strict_identity(&left, op, &right) {
                    return Ok(CValue::Bool(result));
                }
                let operator = match op {
                    BinaryOp::StrictEq => "==",
                    BinaryOp::StrictNe => "!=",
                    _ => return Err(self.unsupported(span, assembly_comparison_rejection())),
                };
                let expression = format!("(({left}) {operator} ({right}))");
                if let Some(result) = self.static_integer_strict_identity_result(&left, op, &right)
                {
                    self.known_bools
                        .insert(expression.clone(), KnownBool::one(result));
                }
                return Ok(CValue::BoolExpr(expression));
            }
            (CValue::Int(left), CValue::Int(right)) => {
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<i64>(), right.parse::<i64>())
                {
                    return Ok(CValue::Bool(match op {
                        BinaryOp::StrictEq => left_literal == right_literal,
                        BinaryOp::StrictNe => left_literal != right_literal,
                        _ => return Err(self.unsupported(span, assembly_comparison_rejection())),
                    }));
                }
                if left == right {
                    return Ok(CValue::Bool(static_strict_identity_result(true, op)));
                }
                if let Some(result) = self.static_integer_strict_identity(&left, op, &right) {
                    return Ok(CValue::Bool(result));
                }
                let operator = match op {
                    BinaryOp::StrictEq => "==",
                    BinaryOp::StrictNe => "!=",
                    _ => return Err(self.unsupported(span, assembly_comparison_rejection())),
                };
                let expression = format!("(({left}) {operator} ({right}))");
                if let Some(result) = self.static_integer_strict_identity_result(&left, op, &right)
                {
                    self.known_bools
                        .insert(expression.clone(), KnownBool::one(result));
                }
                return Ok(CValue::BoolExpr(expression));
            }
            _ => false,
        };
        let result = match op {
            BinaryOp::StrictEq => is_identical,
            BinaryOp::StrictNe => !is_identical,
            _ => return Err(self.unsupported(span, assembly_comparison_rejection())),
        };
        Ok(CValue::Bool(result))
    }

    fn static_integer_strict_identity(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        if left_values.is_single() && right_values.is_single() {
            return None;
        }
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_integer_strict_identity_result(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn known_string_values(&self, value: &str) -> Option<KnownString> {
        self.known_strings.get(value).cloned()
    }

    fn known_bool_values(&self, value: &str) -> Option<KnownBool> {
        match value {
            "1" => Some(KnownBool::one(true)),
            "0" => Some(KnownBool::one(false)),
            _ => self.known_bools.get(value).cloned(),
        }
    }

    fn static_bool_strict_identity(
        &self,
        left_values: Option<KnownBool>,
        right_values: Option<KnownBool>,
        op: BinaryOp,
    ) -> Option<bool> {
        let left_values = left_values?;
        let right_values = right_values?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn known_float_values(&self, value: &str) -> Option<KnownFloat> {
        value
            .parse::<f64>()
            .ok()
            .map(KnownFloat::one)
            .or_else(|| self.known_floats.get(value).cloned())
    }

    fn is_tracked_float_value(&self, value: &str) -> bool {
        self.known_floats.contains_key(value)
    }

    fn known_finite_nonzero_float_values(&self, value: &str) -> bool {
        self.known_float_values(value).is_some_and(|values| {
            values
                .values()
                .iter()
                .all(|value| value.is_finite() && *value != 0.0)
        })
    }

    fn known_finite_positive_float_values(&self, value: &str) -> bool {
        self.known_float_values(value).is_some_and(|values| {
            values
                .values()
                .iter()
                .all(|value| value.is_finite() && *value > 0.0)
        })
    }

    fn static_float_strict_identity(&self, left: &str, op: BinaryOp, right: &str) -> Option<bool> {
        let left_values = self.known_float_values(left)?;
        let right_values = self.known_float_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_same_float_strict_identity(&self, value: &str, op: BinaryOp) -> Option<bool> {
        let values = self.known_float_values(value)?;
        if !values.values().iter().all(|value| value.is_finite()) {
            return None;
        }
        Some(static_strict_identity_result(true, op))
    }

    fn static_float_arithmetic(&self, left: &str, op: BinaryOp, right: &str) -> Option<KnownFloat> {
        let left_values = self.known_float_values(left)?;
        let right_values = self.known_float_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                let result = match op {
                    BinaryOp::Add => left + right,
                    BinaryOp::Sub => left - right,
                    BinaryOp::Mul => left * right,
                    _ => return None,
                };
                if !result.is_finite() {
                    return None;
                }
                results.push(result);
            }
        }
        KnownFloat::from_values(results)
    }

    fn known_string_values_for_value(&self, value: &CValue) -> Option<KnownString> {
        match value {
            CValue::String(value) => Some(KnownString::one(value.clone())),
            CValue::StringExpr(value) => self.known_string_values(value),
            _ => None,
        }
    }

    fn c_string_value_byte_len_operand(&mut self, value: &CValue) -> Option<String> {
        match value {
            CValue::String(value) => Some(value.len().to_string()),
            CValue::StringExpr(value) => self.c_string_expr_byte_len_operand(value),
            _ => None,
        }
    }

    fn c_string_expr_byte_len_operand(&mut self, value: &str) -> Option<String> {
        if let Some(length) = self.known_string_lengths.get(value) {
            return Some(length.clone());
        }

        let values = self.known_string_values(value)?;
        if let Some(byte_len) = known_strings_have_uniform_byte_length(&values) {
            return Some(byte_len.to_string());
        }

        if known_strings_are_nul_free(&values) {
            self.uses_strcmp = true;
            return Some(format!("strlen((const char *)({value}))"));
        }

        None
    }

    fn static_string_strict_identity(
        &self,
        left_values: Option<KnownString>,
        right_values: Option<KnownString>,
        op: BinaryOp,
    ) -> Option<bool> {
        let left_values = left_values?;
        let right_values = right_values?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn emit_bool_binary(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        if let (Some(left), Some(right)) = (
            self.known_truthiness_for_value(&left),
            self.known_truthiness_for_value(&right),
        ) {
            return Ok(CValue::Bool(logical_truthiness_result(left, op, right)?));
        }
        match (left, right) {
            (CValue::Bool(left), CValue::Bool(right)) => Ok(CValue::Bool(match op {
                BinaryOp::LogicalAnd => left && right,
                BinaryOp::LogicalOr => left || right,
                BinaryOp::LogicalXor => left ^ right,
                _ => return Err(self.unsupported(span, assembly_logical_rejection())),
            })),
            (CValue::Bool(left), right) => match op {
                BinaryOp::LogicalAnd if left => self.require_bool_value(right, span),
                BinaryOp::LogicalAnd => Ok(CValue::Bool(false)),
                BinaryOp::LogicalOr if left => Ok(CValue::Bool(true)),
                BinaryOp::LogicalOr => self.require_bool_value(right, span),
                BinaryOp::LogicalXor if left => {
                    let right = self.require_bool_value(right, span)?;
                    self.emit_bool_not(right, span)
                }
                BinaryOp::LogicalXor => self.require_bool_value(right, span),
                _ => Err(self.unsupported(span, assembly_logical_rejection())),
            },
            (left, CValue::Bool(right)) => match op {
                BinaryOp::LogicalAnd if right => self.require_bool_value(left, span),
                BinaryOp::LogicalAnd => Ok(CValue::Bool(false)),
                BinaryOp::LogicalOr if right => Ok(CValue::Bool(true)),
                BinaryOp::LogicalOr => self.require_bool_value(left, span),
                BinaryOp::LogicalXor if right => {
                    let left = self.require_bool_value(left, span)?;
                    self.emit_bool_not(left, span)
                }
                BinaryOp::LogicalXor => self.require_bool_value(left, span),
                _ => Err(self.unsupported(span, assembly_logical_rejection())),
            },
            (left, right) => {
                let Some(left) = c_bool_operand(left) else {
                    return Err(self.unsupported(span, assembly_logical_rejection()));
                };
                let Some(right) = c_bool_operand(right) else {
                    return Err(self.unsupported(span, assembly_logical_rejection()));
                };
                if left == right && matches!(op, BinaryOp::LogicalAnd | BinaryOp::LogicalOr) {
                    return Ok(CValue::BoolExpr(left));
                }
                if left == right && matches!(op, BinaryOp::LogicalXor) {
                    return Ok(CValue::Bool(false));
                }
                let result = self.static_bool_binary(&left, op, &right);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(CValue::Bool(result.values()[0]));
                    }
                }
                let operator = match op {
                    BinaryOp::LogicalAnd => "&&",
                    BinaryOp::LogicalOr => "||",
                    BinaryOp::LogicalXor => "!=",
                    _ => return Err(self.unsupported(span, assembly_logical_rejection())),
                };
                let expression = format!("(({left}) {operator} ({right}))");
                if let Some(result) = result {
                    self.known_bools.insert(expression.clone(), result);
                }
                Ok(CValue::BoolExpr(expression))
            }
        }
    }

    fn emit_logical_expr(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
        span: Span,
    ) -> CompileResult<CValue> {
        let left = self.emit_expr(left)?;
        if let Some(left_truthy) = self.known_truthiness_for_value(&left) {
            match op {
                BinaryOp::LogicalAnd if !left_truthy => return Ok(CValue::Bool(false)),
                BinaryOp::LogicalOr if left_truthy => return Ok(CValue::Bool(true)),
                _ => {}
            }
        }
        let right = self.emit_expr(right)?;
        self.emit_bool_binary(left, op, right, span)
    }

    fn require_bool_value(&self, value: CValue, span: Span) -> CompileResult<CValue> {
        match value {
            CValue::Bool(_) | CValue::BoolExpr(_) | CValue::ComparisonDecision(_) => Ok(value),
            _ => Err(self.unsupported(span, assembly_logical_rejection())),
        }
    }

    fn known_truthiness_for_value(&self, value: &CValue) -> Option<bool> {
        match value {
            CValue::Bool(value) => Some(*value),
            CValue::BoolExpr(_) => None,
            CValue::ComparisonDecision(_) => None,
            CValue::Int(value) => known_integer_truthiness(&self.known_integer_values(value)),
            CValue::Float(value) => known_float_truthiness(&self.known_float_values(value)),
            CValue::String(value) => Some(is_php_truthy_string(value)),
            CValue::StringExpr(value) => self
                .known_string_values(value)
                .and_then(|values| known_string_truthiness(&values)),
            CValue::ArrayHandle(_) | CValue::NativeValueHandle(_) => None,
            CValue::Null => Some(false),
        }
    }

    fn emit_bool_comparison(
        &self,
        left: String,
        op: BinaryOp,
        right: String,
        span: Span,
    ) -> CompileResult<CValue> {
        let operator = match op {
            BinaryOp::StrictEq => "==",
            BinaryOp::StrictNe => "!=",
            _ => return Err(self.unsupported(span, assembly_comparison_rejection())),
        };
        Ok(CValue::BoolExpr(format!("(({left}) {operator} ({right}))")))
    }

    fn static_bool_binary(&self, left: &str, op: BinaryOp, right: &str) -> Option<KnownBool> {
        let left_values = self.known_bool_values(left)?;
        let right_values = self.known_bool_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                results.push(match op {
                    BinaryOp::LogicalAnd => *left && *right,
                    BinaryOp::LogicalOr => *left || *right,
                    BinaryOp::LogicalXor => *left ^ *right,
                    _ => return None,
                });
            }
        }
        KnownBool::from_values(results)
    }

    fn emit_string_comparison(
        &mut self,
        left: String,
        op: BinaryOp,
        right: String,
        span: Span,
    ) -> CompileResult<CValue> {
        let operator = c_string_comparison_operator(op)
            .ok_or_else(|| self.unsupported(span, assembly_comparison_rejection()))?;
        if left == right {
            let Some(result) = reflexive_string_comparison_result(op) else {
                return Err(self.unsupported(span, assembly_comparison_rejection()));
            };
            return Ok(CValue::Bool(result));
        }
        let known_result = if matches!(op, BinaryOp::StrictEq | BinaryOp::StrictNe) {
            self.static_string_strict_identity(
                self.known_string_values(&left),
                self.known_string_values(&right),
                op,
            )
        } else {
            let left_values = self
                .known_string_values(&left)
                .ok_or_else(|| self.unsupported(span, assembly_comparison_rejection()))?;
            let right_values = self
                .known_string_values(&right)
                .ok_or_else(|| self.unsupported(span, assembly_comparison_rejection()))?;
            if !known_string_pairs_are_safe_for_native_comparison(&left_values, &right_values) {
                return Err(self.unsupported(span, assembly_comparison_rejection()));
            }
            string_comparison_result_for_known_values(&left_values, op, &right_values)
        };
        if let Some(known_result) = known_result {
            return Ok(CValue::Bool(known_result));
        }
        self.uses_strcmp = true;
        let expression = format!("(strcmp({left}, {right}) {operator} 0)");
        if let Some(known_result) = known_result {
            self.known_bools
                .insert(expression.clone(), KnownBool::one(known_result));
        }
        Ok(CValue::BoolExpr(expression))
    }

    fn emit_ternary(
        &mut self,
        condition: CValue,
        if_true: CValue,
        if_false: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        match condition {
            CValue::Bool(true) => return Ok(if_true),
            CValue::Bool(false) => return Ok(if_false),
            CValue::Int(value) => {
                let Some(values) = self.known_integer_values(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                if values.values().iter().all(|value| *value != 0) {
                    return Ok(if_true);
                }
                if values.is_single_value(0) {
                    return Ok(if_false);
                }
                Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION))
            }
            CValue::Float(value) => {
                let Some(values) = self.known_float_values(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                if !values.values().iter().all(|value| value.is_finite()) {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                }
                if values.values().iter().all(|value| *value != 0.0) {
                    return Ok(if_true);
                }
                if matches!(values.values(), [value] if *value == 0.0) {
                    return Ok(if_false);
                }
                Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION))
            }
            CValue::String(value) => {
                if is_php_truthy_string(&value) {
                    Ok(if_true)
                } else {
                    Ok(if_false)
                }
            }
            CValue::StringExpr(value) => {
                let Some(values) = self.known_string_values(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                match known_string_truthiness(&values) {
                    Some(true) => Ok(if_true),
                    Some(false) => Ok(if_false),
                    None => Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION)),
                }
            }
            CValue::Null => Ok(if_false),
            condition => self.emit_dynamic_ternary(condition, if_true, if_false, span),
        }
    }

    fn emit_ternary_expr(
        &mut self,
        condition: &Expr,
        if_true: &Expr,
        if_false: &Expr,
        span: Span,
    ) -> CompileResult<CValue> {
        let condition_value = self.emit_expr(condition)?;
        if same_direct_variable_ternary_expr(condition, if_true, if_false) {
            return Ok(condition_value);
        }
        if let Some(truthy) = self.known_truthiness_for_value(&condition_value) {
            return if truthy {
                self.emit_expr(if_true)
            } else {
                self.emit_expr(if_false)
            };
        }
        if !matches!(
            condition_value,
            CValue::BoolExpr(_) | CValue::ComparisonDecision(_)
        ) {
            return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
        }
        let if_true = self.emit_expr(if_true)?;
        let if_false = self.emit_expr(if_false)?;
        self.emit_ternary(condition_value, if_true, if_false, span)
    }

    fn emit_short_ternary(
        &mut self,
        condition: &Expr,
        if_false: &Expr,
        span: Span,
    ) -> CompileResult<CValue> {
        let condition_value = self.emit_expr(condition)?;
        if same_direct_variable_expr(condition, if_false) {
            if matches!(
                condition_value,
                CValue::BoolExpr(_)
                    | CValue::ComparisonDecision(_)
                    | CValue::Int(_)
                    | CValue::Float(_)
                    | CValue::StringExpr(_)
            ) {
                return Ok(condition_value);
            }
        }
        match condition_value {
            CValue::Bool(true) => Ok(CValue::Bool(true)),
            CValue::Bool(false) => {
                let if_false = self.emit_expr(if_false)?;
                Ok(if_false)
            }
            CValue::Int(value) => {
                let Some(values) = self.known_integer_values(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                if values.values().iter().all(|value| *value != 0) {
                    Ok(CValue::Int(value))
                } else if values.is_single_value(0) {
                    self.emit_expr(if_false)
                } else {
                    Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION))
                }
            }
            CValue::Float(value) => {
                let Some(values) = self.known_float_values(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                if !values.values().iter().all(|value| value.is_finite()) {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                }
                if values.values().iter().all(|value| *value != 0.0) {
                    Ok(CValue::Float(value))
                } else if matches!(values.values(), [value] if *value == 0.0) {
                    self.emit_expr(if_false)
                } else {
                    Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION))
                }
            }
            CValue::String(value) => {
                if is_php_truthy_string(&value) {
                    Ok(CValue::String(value))
                } else {
                    self.emit_expr(if_false)
                }
            }
            CValue::StringExpr(value) => {
                let Some(values) = self.known_string_values(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                match known_string_truthiness(&values) {
                    Some(true) => Ok(CValue::StringExpr(value)),
                    Some(false) => self.emit_expr(if_false),
                    None => Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION)),
                }
            }
            CValue::Null => self.emit_expr(if_false),
            CValue::ArrayHandle(_) | CValue::NativeValueHandle(_) => {
                Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION))
            }
            condition @ (CValue::BoolExpr(_) | CValue::ComparisonDecision(_)) => {
                let if_false = self.emit_expr(if_false)?;
                if !matches!(
                    if_false,
                    CValue::Bool(_) | CValue::BoolExpr(_) | CValue::ComparisonDecision(_)
                ) {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                }
                self.emit_ternary(condition, CValue::Bool(true), if_false, span)
            }
        }
    }

    fn emit_dynamic_ternary(
        &mut self,
        condition: CValue,
        if_true: CValue,
        if_false: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        let Some(condition) = c_bool_operand(condition) else {
            return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
        };
        match (if_true, if_false) {
            (CValue::Null, CValue::Null) => Ok(CValue::Null),
            (CValue::Int(if_true), CValue::Int(if_false)) => {
                if if_true == if_false {
                    return Ok(CValue::Int(if_true));
                }
                let result = self.static_integer_ternary(&if_true, &if_false);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(CValue::Int(result.values()[0].to_string()));
                    }
                }
                let expression = format!("(({condition}) ? ({if_true}) : ({if_false}))");
                if let Some(result) = result {
                    self.known_ints.insert(expression.clone(), result);
                }
                Ok(CValue::Int(expression))
            }
            (CValue::Float(if_true), CValue::Float(if_false)) => {
                if if_true == if_false {
                    return Ok(CValue::Float(if_true));
                }
                let result = self.static_float_ternary(&if_true, &if_false);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(CValue::Float(format_float_literal(result.values()[0])));
                    }
                }
                let expression = format!("(({condition}) ? ({if_true}) : ({if_false}))");
                if let Some(result) = result {
                    self.known_floats.insert(expression.clone(), result);
                }
                Ok(CValue::Float(expression))
            }
            (if_true, if_false) => {
                if matches!(
                    (&if_true, &if_false),
                    (
                        CValue::String(_) | CValue::StringExpr(_),
                        CValue::String(_) | CValue::StringExpr(_)
                    )
                ) {
                    if let Some(result) = identical_c_string_ternary_branch(&if_true, &if_false) {
                        return Ok(result);
                    }
                    let result = self.static_string_ternary(&if_true, &if_false);
                    if let Some(result) = result.as_ref() {
                        if result.is_single() {
                            return Ok(CValue::String(result.values()[0].clone()));
                        }
                    }
                    let if_true_len = self.c_string_value_byte_len_operand(&if_true);
                    let if_false_len = self.c_string_value_byte_len_operand(&if_false);
                    let if_true = c_string_operand(if_true);
                    let if_false = c_string_operand(if_false);
                    let expression = format!("(({condition}) ? ({if_true}) : ({if_false}))");
                    if let Some(result) = result {
                        self.known_strings.insert(expression.clone(), result);
                    }
                    if let (Some(if_true_len), Some(if_false_len)) = (if_true_len, if_false_len) {
                        let length = if if_true_len == if_false_len {
                            if_true_len
                        } else {
                            format!("(({condition}) ? ({if_true_len}) : ({if_false_len}))")
                        };
                        self.known_string_lengths.insert(expression.clone(), length);
                    }
                    return Ok(CValue::StringExpr(expression));
                }
                if let Some(result) = identical_c_bool_expr_ternary_branch(&if_true, &if_false) {
                    return Ok(result);
                }
                if let Some(result) = c_bool_literal_ternary_branch(&condition, &if_true, &if_false)
                {
                    return match result {
                        BoolLiteralTernaryBranch::Static(value) => Ok(CValue::Bool(value)),
                        BoolLiteralTernaryBranch::Reuse(value) => Ok(CValue::BoolExpr(value)),
                        BoolLiteralTernaryBranch::Invert(value) => {
                            self.emit_bool_not(CValue::BoolExpr(value), span)
                        }
                    };
                }
                let Some(if_true) = c_bool_operand(if_true) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                let Some(if_false) = c_bool_operand(if_false) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                let result = self.static_bool_ternary(&if_true, &if_false);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(CValue::Bool(result.values()[0]));
                    }
                }
                let expression = format!("(({condition}) ? ({if_true}) : ({if_false}))");
                if let Some(result) = result {
                    self.known_bools.insert(expression.clone(), result);
                }
                Ok(CValue::BoolExpr(expression))
            }
        }
    }

    fn static_integer_ternary(&self, if_true: &str, if_false: &str) -> Option<KnownInt> {
        let if_true = self.known_integer_values(if_true)?;
        let if_false = self.known_integer_values(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values());
        KnownInt::from_values(values)
    }

    fn static_float_ternary(&self, if_true: &str, if_false: &str) -> Option<KnownFloat> {
        let if_true = self.known_float_values(if_true)?;
        let if_false = self.known_float_values(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values());
        KnownFloat::from_values(values)
    }

    fn static_string_ternary(&self, if_true: &CValue, if_false: &CValue) -> Option<KnownString> {
        let if_true = self.known_string_values_for_value(if_true)?;
        let if_false = self.known_string_values_for_value(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values().iter().cloned());
        KnownString::from_values(values)
    }

    fn static_bool_ternary(&self, if_true: &str, if_false: &str) -> Option<KnownBool> {
        let if_true = self.known_bool_values(if_true)?;
        let if_false = self.known_bool_values(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values());
        KnownBool::from_values(values)
    }

    fn emit_unary(&mut self, op: UnaryOp, value: CValue, span: Span) -> CompileResult<CValue> {
        match op {
            UnaryOp::Negate => self.emit_numeric_negate(value, span),
            UnaryOp::Not => self.emit_bool_not(value, span),
            UnaryOp::BitwiseNot => self.emit_integer_bitwise_not(value, span),
        }
    }

    fn primitive_source_for_value(&self, value: &CValue) -> Option<BackendPrimitiveSource> {
        match value {
            CValue::Null => Some(BackendPrimitiveSource::null()),
            CValue::Bool(value) => Some(BackendPrimitiveSource::bool_value(*value)),
            CValue::BoolExpr(value) => Some(
                self.known_bool_values(value)
                    .map(BackendPrimitiveSource::bool_values)
                    .unwrap_or_else(BackendPrimitiveSource::unknown_bool),
            ),
            CValue::Int(value) => Some(
                self.known_integer_values(value)
                    .map(BackendPrimitiveSource::int_values)
                    .unwrap_or_else(BackendPrimitiveSource::unknown_int),
            ),
            CValue::Float(value) => Some(
                self.known_float_values(value)
                    .map(BackendPrimitiveSource::float_values)
                    .unwrap_or_else(BackendPrimitiveSource::unknown_float),
            ),
            CValue::String(value) => Some(BackendPrimitiveSource::string_value(value)),
            CValue::StringExpr(value) => self
                .known_string_values(value)
                .map(BackendPrimitiveSource::string_values),
            CValue::ArrayHandle(_)
            | CValue::ComparisonDecision(_)
            | CValue::NativeValueHandle(_) => None,
        }
    }

    fn checked_static_primitive_negate_result_for_value(
        &self,
        value: &CValue,
    ) -> Result<Option<BackendArithmeticResult>, PhpPrimitiveArithmeticError> {
        self.primitive_source_for_value(value)
            .map(|source| source.single_arithmetic_result(PhpPrimitiveArithmeticOperation::Negate))
            .unwrap_or(Ok(None))
    }

    fn checked_static_primitive_arithmetic_result_for_values(
        &self,
        left: &CValue,
        op: BinaryOp,
        right: &CValue,
    ) -> Result<Option<BackendArithmeticResult>, PhpPrimitiveArithmeticError> {
        let Some(operation) = backend_binary_primitive_arithmetic_operation(op) else {
            return Ok(None);
        };
        let Some(left) = self.primitive_source_for_value(left) else {
            return Ok(None);
        };
        let Some(right) = self.primitive_source_for_value(right) else {
            return Ok(None);
        };
        left.pair_arithmetic_result(operation, &right)
    }

    fn static_primitive_source_for_expr(
        &self,
        expr: &Expr,
    ) -> Result<Option<BackendPrimitiveSource>, PhpPrimitiveArithmeticError> {
        match expr {
            Expr::Null(_) => Ok(Some(BackendPrimitiveSource::null())),
            Expr::Bool(value, _) => Ok(Some(BackendPrimitiveSource::bool_value(*value))),
            Expr::Int(value, _) => Ok(Some(BackendPrimitiveSource::int_values(KnownInt::one(
                *value,
            )))),
            Expr::Float(value, _) => Ok(Some(BackendPrimitiveSource::float_values(
                KnownFloat::one(*value),
            ))),
            Expr::String(value, _) => Ok(Some(BackendPrimitiveSource::string_value(value))),
            Expr::Variable(name, _) => Ok(self
                .variables
                .get(name)
                .and_then(|value| self.primitive_source_for_value(value))),
            Expr::Unary {
                op: UnaryOp::Negate,
                expr,
                ..
            } => {
                let Some(source) = self.static_primitive_source_for_expr(expr)? else {
                    return Ok(None);
                };
                Ok(source
                    .single_arithmetic_result(PhpPrimitiveArithmeticOperation::Negate)?
                    .map(BackendPrimitiveSource::from_arithmetic_result))
            }
            Expr::Binary {
                left, op, right, ..
            } => {
                let Some(operation) = backend_binary_primitive_arithmetic_operation(*op) else {
                    return Ok(None);
                };
                let Some(left) = self.static_primitive_source_for_expr(left)? else {
                    return Ok(None);
                };
                let Some(right) = self.static_primitive_source_for_expr(right)? else {
                    return Ok(None);
                };
                Ok(left
                    .pair_arithmetic_result(operation, &right)?
                    .map(BackendPrimitiveSource::from_arithmetic_result))
            }
            _ => Ok(None),
        }
    }

    fn try_emit_static_primitive_arithmetic_output(&mut self, expr: &Expr) -> CompileResult<bool> {
        if !matches!(
            expr,
            Expr::Unary {
                op: UnaryOp::Negate,
                ..
            } | Expr::Binary {
                op: BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul,
                ..
            }
        ) {
            return Ok(false);
        }

        let Ok(Some(source)) = self.static_primitive_source_for_expr(expr) else {
            return Ok(false);
        };
        let Some(value) = source.single_c_value() else {
            return Ok(false);
        };
        self.emit_echo(value, expr.span())?;
        Ok(true)
    }

    fn emit_numeric_negate(&mut self, value: CValue, span: Span) -> CompileResult<CValue> {
        if !matches!(value, CValue::Int(_) | CValue::Float(_)) {
            match self.checked_static_primitive_negate_result_for_value(&value) {
                Ok(Some(result)) => {
                    if let Some(value) = result.into_single_c_value() {
                        return Ok(value);
                    }
                }
                Ok(None)
                | Err(
                    PhpPrimitiveArithmeticError::MissingRightOperand
                    | PhpPrimitiveArithmeticError::Conversion(_)
                    | PhpPrimitiveArithmeticError::IntegerOverflow
                    | PhpPrimitiveArithmeticError::NonFiniteFloat,
                ) => {}
            }
        }

        match value {
            CValue::Int(value) => {
                let Some(result) = self.static_integer_negate(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_UNARY_REJECTION));
                };
                if result.is_single() {
                    return Ok(CValue::Int(result.values()[0].to_string()));
                }
                let expression = format!("(-{value})");
                self.known_ints.insert(expression.clone(), result);
                Ok(CValue::Int(expression))
            }
            CValue::Float(value) => {
                if let Some(result) = self.static_float_negate(&value) {
                    if result.is_single() && result.values()[0] != 0.0 {
                        return Ok(CValue::Float(format_float_literal(result.values()[0])));
                    }
                }
                let expression = format!("(-{value})");
                if let Some(result) = self.static_float_negate(&value) {
                    self.known_floats.insert(expression.clone(), result);
                }
                Ok(CValue::Float(expression))
            }
            _ => Err(self.unsupported(span, ASSEMBLY_UNARY_REJECTION)),
        }
    }

    fn emit_integer_bitwise_not(&mut self, value: CValue, span: Span) -> CompileResult<CValue> {
        let CValue::Int(value) = value else {
            return Err(self.unsupported(span, ASSEMBLY_BITWISE_REJECTION));
        };
        if let Some(result) = self.static_integer_bitwise_not(&value) {
            if result.is_single() {
                return Ok(CValue::Int(result.values()[0].to_string()));
            }
        }
        let expression = format!("(~{value})");
        if let Some(result) = self.static_integer_bitwise_not(&value) {
            self.known_ints.insert(expression.clone(), result);
        }
        Ok(CValue::Int(expression))
    }

    fn emit_bool_not(&mut self, value: CValue, span: Span) -> CompileResult<CValue> {
        match value {
            CValue::Bool(value) => Ok(CValue::Bool(!value)),
            CValue::ComparisonDecision(decision) => Ok(CValue::BoolExpr(format!(
                "!({})",
                c_comparison_decision_bool_expr(&decision)
            ))),
            CValue::BoolExpr(value) => {
                if let Some(result) = self.static_bool_not(&value) {
                    if result.is_single() {
                        return Ok(CValue::Bool(result.values()[0]));
                    }
                }
                let expression = format!("!({value})");
                if let Some(result) = self.static_bool_not(&value) {
                    self.known_bools.insert(expression.clone(), result);
                }
                Ok(CValue::BoolExpr(expression))
            }
            CValue::Int(value) => {
                let Some(truthy) = known_integer_truthiness(&self.known_integer_values(&value))
                else {
                    return Err(self.unsupported(span, ASSEMBLY_UNARY_REJECTION));
                };
                Ok(CValue::Bool(!truthy))
            }
            CValue::Float(value) => {
                let Some(truthy) = known_float_truthiness(&self.known_float_values(&value)) else {
                    return Err(self.unsupported(span, ASSEMBLY_UNARY_REJECTION));
                };
                Ok(CValue::Bool(!truthy))
            }
            CValue::String(value) => Ok(CValue::Bool(!is_php_truthy_string(&value))),
            CValue::StringExpr(value) => {
                let Some(values) = self.known_string_values(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_UNARY_REJECTION));
                };
                match known_string_truthiness(&values) {
                    Some(value) => Ok(CValue::Bool(!value)),
                    None => Err(self.unsupported(span, ASSEMBLY_UNARY_REJECTION)),
                }
            }
            CValue::Null => Ok(CValue::Bool(true)),
            CValue::ArrayHandle(_) | CValue::NativeValueHandle(_) => {
                Err(self.unsupported(span, ASSEMBLY_UNARY_REJECTION))
            }
        }
    }

    fn static_bool_not(&self, value: &str) -> Option<KnownBool> {
        let value = self.known_bool_values(value)?;
        KnownBool::from_values(value.values().iter().map(|value| !value))
    }

    fn static_float_negate(&self, value: &str) -> Option<KnownFloat> {
        let value = self.known_float_values(value)?;
        let mut results = Vec::new();
        for value in value.values() {
            let result = -value;
            if !result.is_finite() {
                return None;
            }
            results.push(result);
        }
        KnownFloat::from_values(results)
    }

    fn emit_array_literal(&mut self, items: &[ArrayItem], span: Span) -> CompileResult<String> {
        self.uses_native_array_helpers = true;
        let handle = self.next_native_name("array");
        self.body.push(format!(
            "phpc_NativeArrayHandle {handle} = phpc_native_array_empty();"
        ));
        self.array_cleanup_handles.push(handle.clone());

        for item in items {
            if item.by_reference {
                return Err(self.unsupported(span, ASSEMBLY_ARRAY_REJECTION));
            }
            if let Some(key) = &item.key {
                self.emit_array_write_key_value(&handle, key, &item.value)?;
            } else {
                self.emit_array_append_value(&handle, &item.value)?;
            }
        }

        Ok(handle)
    }

    fn emit_array_write_key_value(
        &mut self,
        handle: &str,
        key: &Expr,
        value: &Expr,
    ) -> CompileResult<()> {
        let value = self.materialize_native_array_expr_value_handle(value, "")?;
        let value_cleanup = value.cleanup_after_use.clone();
        let key = self.materialize_native_array_key(key, &value_cleanup)?;
        let diagnostic = self.next_native_name("array_diagnostic");
        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));

        let local_cleanup = format!(
            "phpc_native_diagnostic_message_stderr({diagnostic}); phpc_native_diagnostic_free({diagnostic}); {}{}",
            c_cleanup_sequence(&key.cleanup_after_use),
            c_cleanup_sequence(&value_cleanup)
        );
        let write_error_exit = self.native_error_exit(&local_cleanup);
        self.body.push(format!(
            "if (!phpc_native_array_insert_key_value_with_diagnostic({handle}, {}, {}, &{diagnostic})) {{ {write_error_exit} }}",
            key.result,
            value.handle
        ));
        self.body.extend(key.cleanup_after_use);
        self.body.extend(value_cleanup);
        Ok(())
    }

    fn emit_array_append_value(&mut self, handle: &str, value: &Expr) -> CompileResult<()> {
        let value = self.materialize_native_array_expr_value_handle(value, "")?;
        let value_cleanup = value.cleanup_after_use;
        let diagnostic = self.next_native_name("array_append_diagnostic");
        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        let append_failure_cleanup = format!(
            "if ({diagnostic}.ptr != NULL) {{ phpc_native_diagnostic_message_stderr({diagnostic}); phpc_native_diagnostic_free({diagnostic}); }} {}",
            c_cleanup_sequence(&value_cleanup)
        );
        let append_error_exit = self.native_error_exit(&append_failure_cleanup);
        self.body.push(format!(
            "if (!phpc_native_array_append_value_with_diagnostic({handle}, {}, &{diagnostic})) {{ {append_error_exit} }}",
            value.handle
        ));
        self.emit_report_native_diagnostic(&diagnostic);
        self.body.extend(value_cleanup);
        Ok(())
    }

    fn materialize_native_array_expr_value_handle(
        &mut self,
        expr: &Expr,
        failure_cleanup: &str,
    ) -> CompileResult<CNativeValueMaterialization> {
        if let Expr::Array { items, span } = expr {
            return self.materialize_native_array_literal_value_handle(items, *span);
        }

        if let Some(value) = self.try_materialize_native_value_result_expr(expr, failure_cleanup)? {
            return Ok(value);
        }

        let value = self.emit_expr(expr)?;
        self.materialize_native_array_c_value_handle(value, expr.span())
    }

    fn materialize_native_array_c_value_handle(
        &mut self,
        value: CValue,
        span: Span,
    ) -> CompileResult<CNativeValueMaterialization> {
        let handle = self.emit_native_value_for_cvalue(value, span)?;
        Ok(CNativeValueMaterialization {
            handle: handle.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({handle});")],
        })
    }

    fn materialize_native_array_literal_value_handle(
        &mut self,
        items: &[ArrayItem],
        span: Span,
    ) -> CompileResult<CNativeValueMaterialization> {
        let array = self.emit_array_literal(items, span)?;
        let handle = self.next_native_name("array_value");
        self.body.push(format!(
            "phpc_NativeValueHandle {handle} = phpc_native_value_from_array({array});"
        ));
        self.release_native_array_cleanup_handle(&array);
        self.body.push(format!("phpc_native_array_free({array});"));
        Ok(CNativeValueMaterialization {
            handle: handle.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({handle});")],
        })
    }

    fn release_native_array_cleanup_handle(&mut self, handle: &str) {
        if let Some(index) = self
            .array_cleanup_handles
            .iter()
            .rposition(|cleanup_handle| cleanup_handle == handle)
        {
            self.array_cleanup_handles.remove(index);
        }
    }

    fn materialize_native_value_result_operand(
        &mut self,
        expr: &Expr,
        failure_cleanup: &str,
    ) -> CompileResult<CNativeValueMaterialization> {
        if let Expr::Array { items, span } = expr {
            return self.materialize_native_array_literal_value_handle(items, *span);
        }

        if let Some(value) = self.try_materialize_native_value_result_expr(expr, failure_cleanup)? {
            return Ok(value);
        }

        let value = self.emit_expr(expr)?;
        self.materialize_native_array_c_value_handle(value, expr.span())
    }

    fn try_materialize_native_value_result_expr(
        &mut self,
        expr: &Expr,
        failure_cleanup: &str,
    ) -> CompileResult<Option<CNativeValueMaterialization>> {
        if let Some(name) = request_superglobal_root_name(expr) {
            return Ok(Some(self.materialize_request_superglobal_snapshot_value(
                name,
                failure_cleanup,
            )));
        }

        if let Some(value) =
            self.try_materialize_nested_array_lvalue_read_expr(expr, failure_cleanup)?
        {
            return Ok(Some(value));
        }

        match expr {
            Expr::Unary { op, expr, .. } => {
                if matches!(op, UnaryOp::BitwiseNot) {
                    let value =
                        self.materialize_native_value_result_operand(expr, failure_cleanup)?;
                    return Ok(Some(
                        self.emit_native_value_bitwise_operation_result_handle(
                            value,
                            None,
                            "PHPC_NATIVE_VALUE_BITWISE_NOT",
                            "native_value_bitwise_not",
                            failure_cleanup,
                        ),
                    ));
                }

                let Some(op_tag) = native_value_unary_op_tag(*op) else {
                    return Ok(None);
                };
                let value = self.materialize_native_value_result_operand(expr, failure_cleanup)?;
                Ok(Some(self.emit_native_value_unary_result_handle(
                    value,
                    op_tag,
                    failure_cleanup,
                )))
            }
            Expr::Binary {
                left, op, right, ..
            } => {
                if matches!(op, BinaryOp::NullCoalesce) {
                    return self.try_materialize_native_value_offset_null_coalesce_expr(
                        left,
                        right,
                        failure_cleanup,
                    );
                }

                if let (Some(op_tag), Some(result_prefix)) = (
                    native_value_bitwise_binary_op_tag(*op),
                    native_value_bitwise_binary_result_prefix(*op),
                ) {
                    let left_value =
                        self.materialize_native_value_result_operand(left, failure_cleanup)?;
                    let right_failure_cleanup = format!(
                        "{}{}",
                        c_cleanup_sequence(&left_value.cleanup_after_use),
                        failure_cleanup
                    );
                    let right_value = self
                        .materialize_native_value_result_operand(right, &right_failure_cleanup)?;
                    return Ok(Some(
                        self.emit_native_value_bitwise_operation_result_handle(
                            left_value,
                            Some(right_value),
                            op_tag,
                            result_prefix,
                            failure_cleanup,
                        ),
                    ));
                }

                if let Some(op_tag) = native_value_binary_op_tag(*op) {
                    let left_value =
                        self.materialize_native_value_result_operand(left, failure_cleanup)?;
                    let right_failure_cleanup = format!(
                        "{}{}",
                        c_cleanup_sequence(&left_value.cleanup_after_use),
                        failure_cleanup
                    );
                    let right_value = self
                        .materialize_native_value_result_operand(right, &right_failure_cleanup)?;
                    return Ok(Some(self.emit_native_value_binary_result_handle(
                        left_value,
                        op_tag,
                        right_value,
                        failure_cleanup,
                    )));
                }
                let Some(op_tag) = native_value_comparison_op_tag(*op) else {
                    return Ok(None);
                };
                let left_value =
                    self.materialize_native_value_result_operand(left, failure_cleanup)?;
                let right_failure_cleanup = format!(
                    "{}{}",
                    c_cleanup_sequence(&left_value.cleanup_after_use),
                    failure_cleanup
                );
                let right_value =
                    self.materialize_native_value_result_operand(right, &right_failure_cleanup)?;
                Ok(Some(self.emit_native_value_compare_result_handle(
                    left_value,
                    op_tag,
                    right_value,
                    failure_cleanup,
                )))
            }
            Expr::Cast { kind, expr, .. } => {
                let value = self.materialize_native_value_result_operand(expr, failure_cleanup)?;
                Ok(Some(self.emit_native_value_cast_operation_result_handle(
                    value,
                    native_value_cast_op_tag(*kind),
                    failure_cleanup,
                )))
            }
            Expr::CompoundAssign {
                target,
                op,
                expr,
                span,
            } => self.materialize_array_lvalue_compound_assignment_result_for_target(
                target,
                *op,
                expr,
                *span,
                failure_cleanup,
            ),
            Expr::IncrementDecrement {
                target,
                op,
                position,
                span,
            } => self.materialize_array_lvalue_increment_decrement_result_for_target(
                target,
                *op,
                *position,
                *span,
                failure_cleanup,
            ),
            Expr::Call { name, args, span } => {
                if let Some(op_tag) = native_value_cast_builtin_op_tag(name) {
                    let [arg] = args.as_slice() else {
                        return Err(self.unsupported(*span, ASSEMBLY_CAST_REJECTION));
                    };
                    let value =
                        self.materialize_native_value_result_operand(arg, failure_cleanup)?;
                    return Ok(Some(self.emit_native_value_cast_operation_result_handle(
                        value,
                        op_tag,
                        failure_cleanup,
                    )));
                }
                if let Some(type_name_tag) = native_value_type_name_tag(name) {
                    let [arg] = args.as_slice() else {
                        return Err(self.unsupported(*span, ASSEMBLY_FUNCTION_CALL_REJECTION));
                    };
                    let value =
                        self.materialize_native_value_result_operand(arg, failure_cleanup)?;
                    return Ok(Some(self.emit_native_value_type_name_result_handle(
                        value,
                        type_name_tag,
                        failure_cleanup,
                    )));
                }
                if let Some(operation) = native_string_result_operation_for_name(name) {
                    let [arg] = args.as_slice() else {
                        return Err(
                            self.unsupported(*span, ASSEMBLY_STRING_RESULT_OPERATION_REJECTION)
                        );
                    };
                    let value =
                        self.materialize_native_value_result_operand(arg, failure_cleanup)?;
                    return Ok(Some(self.emit_native_string_result_operation_handle(
                        value,
                        operation,
                        failure_cleanup,
                    )));
                }
                if let Some(builtin) = native_array_pointer_builtin(name, args) {
                    return self
                        .materialize_native_array_pointer_call(
                            builtin,
                            args,
                            *span,
                            failure_cleanup,
                        )
                        .map(Some);
                }
                if let Some(builtin) = native_array_sort_builtin(name, args) {
                    let value =
                        self.emit_native_array_sort_call(builtin, args, *span, failure_cleanup)?;
                    return self
                        .materialize_native_array_c_value_handle(value, *span)
                        .map(Some);
                }
                if let Some(builtin) = native_value_array_callback_builtin(name, args) {
                    return self
                        .emit_native_value_array_callback_call(
                            builtin,
                            args,
                            *span,
                            failure_cleanup,
                        )
                        .map(Some);
                }
                Ok(None)
            }
            Expr::NullCoalesceAssign { target, expr, span } => self
                .materialize_array_offset_null_coalesce_assignment_expr(
                    target,
                    expr,
                    *span,
                    failure_cleanup,
                )?
                .map_or_else(
                    || {
                        self.materialize_array_lvalue_null_coalesce_assignment_expr(
                            target,
                            expr,
                            *span,
                            failure_cleanup,
                        )
                    },
                    |value| Ok(Some(value)),
                ),
            Expr::Index {
                target,
                index,
                span,
            } => {
                if !self.uses_native_string_helpers || !self.is_array_offset_subject_expr(target) {
                    return Ok(None);
                }
                if let Some(operation) = native_dereferenced_call_result_operation(expr) {
                    return Err(self.unsupported_call_operation(operation));
                }
                if let Some(superglobal_span) = request_superglobal_expr_span(target) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if is_object_offset_expr(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }

                let subject =
                    self.materialize_native_value_result_operand(target, failure_cleanup)?;
                let offset_failure_cleanup = format!(
                    "{}{}",
                    c_cleanup_sequence(&subject.cleanup_after_use),
                    failure_cleanup
                );
                let offset =
                    self.materialize_native_value_result_operand(index, &offset_failure_cleanup)?;
                Ok(Some(self.emit_native_value_offset_read_result_handle(
                    subject,
                    offset,
                    failure_cleanup,
                )))
            }
            _ => Ok(None),
        }
    }

    fn try_materialize_nested_array_lvalue_read_expr(
        &mut self,
        expr: &Expr,
        failure_cleanup: &str,
    ) -> CompileResult<Option<CNativeValueMaterialization>> {
        let Some((root, indices, span)) = array_index_expr_path(expr) else {
            return Ok(None);
        };
        if indices.len() < 2 {
            return Ok(None);
        }
        if let Some(superglobal_span) = request_superglobal_expr_span(root) {
            return Err(self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION));
        }
        if is_object_offset_expr(root) {
            return Err(self.unsupported(root.span(), ASSEMBLY_ARRAY_ACCESS_REJECTION));
        }

        let Expr::Variable(name, _) = root else {
            return Ok(None);
        };
        let Some(CValue::ArrayHandle(handle)) = self.variables.get(name).cloned() else {
            return Ok(None);
        };

        let path =
            self.materialize_native_array_lvalue_key_path(&indices, span, failure_cleanup)?;
        Ok(Some(self.emit_array_lvalue_read_materialized_for_handle(
            &handle,
            path,
            "array_lvalue_read_result",
            failure_cleanup,
        )))
    }

    fn try_materialize_native_value_offset_null_coalesce_expr(
        &mut self,
        left: &Expr,
        right: &Expr,
        failure_cleanup: &str,
    ) -> CompileResult<Option<CNativeValueMaterialization>> {
        let Expr::Index {
            target,
            index,
            span,
        } = left
        else {
            return Ok(None);
        };

        if !self.is_array_offset_subject_expr(target) && !self.is_string_offset_subject_expr(target)
        {
            return Ok(None);
        }
        if let Some(operation) = native_dereferenced_call_result_operation(left) {
            return Err(self.unsupported_call_operation(operation));
        }
        if let Some(superglobal_span) = request_superglobal_expr_span(target) {
            return Err(self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION));
        }
        if is_object_offset_expr(target) {
            return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
        }

        let subject = self.materialize_native_value_result_operand(target, failure_cleanup)?;
        let offset_failure_cleanup = format!(
            "{}{}",
            c_cleanup_sequence(&subject.cleanup_after_use),
            failure_cleanup
        );
        let offset =
            self.materialize_native_value_result_operand(index, &offset_failure_cleanup)?;

        Ok(Some(
            self.emit_native_value_offset_null_coalesce_result_handle(
                subject,
                offset,
                right,
                failure_cleanup,
            )?,
        ))
    }

    fn is_array_offset_subject_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Array { .. } => true,
            Expr::Variable(name, _) => {
                matches!(
                    self.variables.get(name),
                    Some(CValue::ArrayHandle(_) | CValue::NativeValueHandle(_))
                )
            }
            _ => false,
        }
    }

    fn emit_native_value_offset_read_result_handle(
        &mut self,
        subject: CNativeValueMaterialization,
        offset: CNativeValueMaterialization,
        failure_cleanup: &str,
    ) -> CNativeValueMaterialization {
        self.uses_native_string_helpers = true;

        let subject_handle = subject.handle.clone();
        let offset_handle = offset.handle.clone();
        let mut operand_cleanup = offset.cleanup_after_use;
        operand_cleanup.extend(subject.cleanup_after_use);

        let diagnostic = self.next_native_name("value_offset_read_diagnostic");
        let result = self.next_native_name("value_offset_read");
        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {result} = phpc_native_value_offset_operation_with_diagnostic({subject_handle}, {offset_handle}, {}, &{diagnostic});",
            NativeStringOffsetOperation::Read as u8
        ));
        self.emit_report_native_diagnostic(&diagnostic);
        self.body
            .push(format!("phpc_native_diagnostic_free({diagnostic});"));
        let cleanup = format!(
            "{}{}",
            c_cleanup_sequence(&operand_cleanup),
            failure_cleanup
        );
        let error_exit = self.native_error_exit(&cleanup);
        self.body
            .push(format!("if ({result}.ptr == NULL) {{ {error_exit} }}"));
        self.body.extend(operand_cleanup);

        CNativeValueMaterialization {
            handle: result.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({result});")],
        }
    }

    fn emit_native_value_offset_null_coalesce_result_handle(
        &mut self,
        subject: CNativeValueMaterialization,
        offset: CNativeValueMaterialization,
        right: &Expr,
        failure_cleanup: &str,
    ) -> CompileResult<CNativeValueMaterialization> {
        self.uses_native_string_helpers = true;

        let subject_handle = subject.handle.clone();
        let offset_handle = offset.handle.clone();
        let mut operand_cleanup = offset.cleanup_after_use;
        operand_cleanup.extend(subject.cleanup_after_use);
        let operand_cleanup_sequence = c_cleanup_sequence(&operand_cleanup);

        let result = self.next_native_name("value_offset_null_coalesce");
        let probe = self.next_native_name("value_offset_null_coalesce_probe");
        let probe_diagnostic = self.next_native_name("value_offset_null_coalesce_diagnostic");
        let bool_diagnostic = self.next_native_name("value_offset_null_coalesce_bool_diagnostic");
        let present = self.next_native_name("value_offset_null_coalesce_present");

        self.body
            .push(format!("phpc_NativeValueHandle {result} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeDiagnosticHandle {probe_diagnostic} = {{0}};"
        ));
        self.body.push(format!(
            "phpc_NativeValueHandle {probe} = phpc_native_value_offset_operation_with_diagnostic({subject_handle}, {offset_handle}, {}, &{probe_diagnostic});",
            NativeStringOffsetOperation::Isset as u8
        ));
        let probe_error_exit = self.native_error_exit(&format!(
            "phpc_native_diagnostic_report({probe_diagnostic}); {operand_cleanup_sequence}{failure_cleanup}"
        ));
        self.body
            .push(format!("if ({probe}.ptr == NULL) {{ {probe_error_exit} }}"));
        self.body
            .push(format!("phpc_native_diagnostic_free({probe_diagnostic});"));
        self.body.push(format!(
            "phpc_NativeDiagnosticHandle {bool_diagnostic} = {{0}};"
        ));
        self.body.push(format!(
            "_Bool {present} = phpc_native_value_bool_with_diagnostic({probe}, &{bool_diagnostic});"
        ));
        let bool_error_exit = self.native_error_exit(&format!(
            "phpc_native_diagnostic_report({bool_diagnostic}); phpc_native_value_free({probe}); {operand_cleanup_sequence}{failure_cleanup}"
        ));
        self.body.push(format!(
            "if ({bool_diagnostic}.ptr != NULL) {{ {bool_error_exit} }}"
        ));
        self.body
            .push(format!("phpc_native_diagnostic_free({bool_diagnostic});"));
        self.body.push(format!("phpc_native_value_free({probe});"));

        let read = self.next_native_name("value_offset_null_coalesce_read");
        let read_diagnostic = self.next_native_name("value_offset_null_coalesce_read_diagnostic");
        self.body.push(format!("if ({present}) {{"));
        self.body.push(format!(
            "phpc_NativeDiagnosticHandle {read_diagnostic} = {{0}};"
        ));
        self.body.push(format!(
            "phpc_NativeValueHandle {read} = phpc_native_value_offset_operation_with_diagnostic({subject_handle}, {offset_handle}, {}, &{read_diagnostic});",
            NativeStringOffsetOperation::Read as u8
        ));
        self.emit_report_native_diagnostic(&read_diagnostic);
        let read_error_exit =
            self.native_error_exit(&format!("{operand_cleanup_sequence}{failure_cleanup}"));
        self.body
            .push(format!("if ({read}.ptr == NULL) {{ {read_error_exit} }}"));
        self.body
            .push(format!("phpc_native_diagnostic_free({read_diagnostic});"));
        self.body.push(format!("{result} = {read};"));
        self.body.push("} else {".to_string());

        let right_failure_cleanup = format!("{operand_cleanup_sequence}{failure_cleanup}");
        let right_value =
            self.materialize_native_value_result_operand(right, &right_failure_cleanup)?;
        self.body
            .push(format!("{result} = {};", right_value.handle));
        self.body.push("}".to_string());
        self.body.extend(operand_cleanup);

        Ok(CNativeValueMaterialization {
            handle: result.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({result});")],
        })
    }

    fn emit_native_value_unary_result_handle(
        &mut self,
        value: CNativeValueMaterialization,
        op_tag: &str,
        failure_cleanup: &str,
    ) -> CNativeValueMaterialization {
        let value_handle = value.handle.clone();
        self.emit_native_value_result_handle(
            "native_value_unary_result",
            "native_value_unary",
            value.cleanup_after_use,
            failure_cleanup,
            |this, result| {
                this.body.push(format!(
                    "phpc_NativeValueOperationResult {result} = phpc_native_value_unary_result({value_handle}, {op_tag});"
                ));
            },
        )
    }

    fn emit_native_value_binary_result_handle(
        &mut self,
        left: CNativeValueMaterialization,
        op_tag: &str,
        right: CNativeValueMaterialization,
        failure_cleanup: &str,
    ) -> CNativeValueMaterialization {
        let left_handle = left.handle.clone();
        let right_handle = right.handle.clone();
        let mut cleanup_after_use = right.cleanup_after_use;
        cleanup_after_use.extend(left.cleanup_after_use);
        self.emit_native_value_result_handle(
            "native_value_binary_result",
            "native_value_binary",
            cleanup_after_use,
            failure_cleanup,
            |this, result| {
                this.body.push(format!(
                    "phpc_NativeValueOperationResult {result} = phpc_native_value_binary_result({left_handle}, {op_tag}, {right_handle});"
                ));
            },
        )
    }

    fn emit_native_value_compare_result_handle(
        &mut self,
        left: CNativeValueMaterialization,
        op_tag: &str,
        right: CNativeValueMaterialization,
        failure_cleanup: &str,
    ) -> CNativeValueMaterialization {
        let left_handle = left.handle.clone();
        let right_handle = right.handle.clone();
        let mut cleanup_after_use = right.cleanup_after_use;
        cleanup_after_use.extend(left.cleanup_after_use);
        self.emit_native_value_result_handle(
            "native_value_compare_result",
            "native_value_compare",
            cleanup_after_use,
            failure_cleanup,
            |this, result| {
                this.body.push(format!(
                    "phpc_NativeValueOperationResult {result} = phpc_native_value_compare_result({left_handle}, {op_tag}, {right_handle});"
                ));
            },
        )
    }

    fn emit_native_value_bitwise_operation_result_handle(
        &mut self,
        subject: CNativeValueMaterialization,
        operand: Option<CNativeValueMaterialization>,
        op_tag: &str,
        result_prefix: &str,
        failure_cleanup: &str,
    ) -> CNativeValueMaterialization {
        self.uses_native_array_helpers = true;

        let subject_handle = subject.handle.clone();
        let (operand_handle, operand_cleanup) = match operand {
            Some(operand) => (operand.handle.clone(), operand.cleanup_after_use),
            None => ("(phpc_NativeValueHandle){0}".to_string(), Vec::new()),
        };
        let mut cleanup_after_use = operand_cleanup;
        cleanup_after_use.extend(subject.cleanup_after_use);

        let diagnostic = self.next_native_name("value_bitwise_diagnostic");
        let result = self.next_native_name(result_prefix);
        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {result} = phpc_native_value_bitwise_operation_with_diagnostic({subject_handle}, {operand_handle}, {op_tag}, &{diagnostic});"
        ));
        let cleanup = format!(
            "phpc_native_diagnostic_message_stderr({diagnostic}); phpc_native_diagnostic_free({diagnostic}); {}{}",
            c_cleanup_sequence(&cleanup_after_use),
            failure_cleanup
        );
        let error_exit = self.native_error_exit(&cleanup);
        self.body
            .push(format!("if ({result}.ptr == NULL) {{ {error_exit} }}"));
        self.body
            .push(format!("phpc_native_diagnostic_free({diagnostic});"));
        self.body.extend(cleanup_after_use);

        CNativeValueMaterialization {
            handle: result.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({result});")],
        }
    }

    fn emit_native_value_cast_operation_result_handle(
        &mut self,
        value: CNativeValueMaterialization,
        op_tag: &str,
        failure_cleanup: &str,
    ) -> CNativeValueMaterialization {
        self.uses_native_array_helpers = true;

        let value_handle = value.handle.clone();
        let diagnostic = self.next_native_name("value_cast_diagnostic");
        let result = self.next_native_name("native_value_cast");
        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {result} = phpc_native_value_cast_operation_with_diagnostic({value_handle}, {op_tag}, &{diagnostic});"
        ));
        let cleanup = format!(
            "phpc_native_diagnostic_message_stderr({diagnostic}); phpc_native_diagnostic_free({diagnostic}); {}{}",
            c_cleanup_sequence(&value.cleanup_after_use),
            failure_cleanup
        );
        let error_exit = self.native_error_exit(&cleanup);
        self.body
            .push(format!("if ({result}.ptr == NULL) {{ {error_exit} }}"));
        self.body
            .push(format!("phpc_native_diagnostic_free({diagnostic});"));
        self.body.extend(value.cleanup_after_use);

        CNativeValueMaterialization {
            handle: result.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({result});")],
        }
    }

    fn emit_native_string_result_operation_handle(
        &mut self,
        subject: CNativeValueMaterialization,
        operation: NativeStringResultOperation,
        failure_cleanup: &str,
    ) -> CNativeValueMaterialization {
        self.uses_native_string_helpers = true;

        let subject_handle = subject.handle.clone();
        let diagnostic = self.next_native_name("string_result_diagnostic");
        let result = self.next_native_name(native_string_result_operation_prefix(operation));
        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {result} = phpc_native_value_string_result_operation_with_diagnostic({subject_handle}, (phpc_NativeValueHandle){{0}}, (phpc_NativeValueHandle){{0}}, 0, 0, 0, {}, &{diagnostic});",
            operation as u8
        ));
        let cleanup = format!(
            "phpc_native_diagnostic_message_stderr({diagnostic}); phpc_native_diagnostic_free({diagnostic}); {}{}",
            c_cleanup_sequence(&subject.cleanup_after_use),
            failure_cleanup
        );
        let error_exit = self.native_error_exit(&cleanup);
        self.body
            .push(format!("if ({result}.ptr == NULL) {{ {error_exit} }}"));
        self.body
            .push(format!("phpc_native_diagnostic_free({diagnostic});"));
        self.body.extend(subject.cleanup_after_use);

        CNativeValueMaterialization {
            handle: result.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({result});")],
        }
    }

    fn emit_native_value_type_name_result_handle(
        &mut self,
        value: CNativeValueMaterialization,
        type_name_tag: &str,
        failure_cleanup: &str,
    ) -> CNativeValueMaterialization {
        let value_handle = value.handle.clone();
        self.emit_native_value_result_handle(
            "native_value_type_name_result",
            "native_value_type_name",
            value.cleanup_after_use,
            failure_cleanup,
            |this, result| {
                this.body.push(format!(
                    "phpc_NativeValueOperationResult {result} = phpc_native_value_type_name_result({value_handle}, {type_name_tag});"
                ));
            },
        )
    }

    fn emit_native_value_type_predicate(
        &mut self,
        value: CNativeValueMaterialization,
        predicate_tag: &str,
    ) -> CValue {
        self.uses_native_array_helpers = true;
        let value_handle = value.handle.clone();
        let result = self.next_native_name("native_value_type_predicate");
        self.body.push(format!(
            "bool {result} = phpc_native_value_type_predicate({value_handle}, {predicate_tag});"
        ));
        self.body.extend(value.cleanup_after_use);
        CValue::BoolExpr(result)
    }

    fn emit_native_value_result_handle(
        &mut self,
        result_prefix: &str,
        value_prefix: &str,
        operand_cleanup: Vec<String>,
        failure_cleanup: &str,
        emit_call: impl FnOnce(&mut Self, &str),
    ) -> CNativeValueMaterialization {
        self.uses_native_array_helpers = true;
        let result = self.next_native_name(result_prefix);
        let value_result = self.next_native_name(value_prefix);
        self.body
            .push(format!("phpc_NativeValueHandle {value_result} = {{0}};"));
        emit_call(self, &result);
        let cleanup = format!(
            "{}{}",
            c_cleanup_sequence(&operand_cleanup),
            failure_cleanup
        );
        self.emit_native_value_operation_result_check(&result, &cleanup);
        self.body.push(format!("{value_result} = {result}.value;"));
        self.body.push(format!("{result}.value.ptr = NULL;"));
        self.body.push(format!(
            "phpc_native_value_operation_result_free({result});"
        ));
        self.body.extend(operand_cleanup);

        CNativeValueMaterialization {
            handle: value_result.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({value_result});")],
        }
    }

    fn emit_native_value_operation_result_check(&mut self, result: &str, cleanup: &str) {
        let result_error_exit = self.native_error_exit(&format!(
            "{cleanup}phpc_native_value_operation_result_free({result}); "
        ));
        self.body.push(format!(
            "if ({result}.tag != PHPC_NATIVE_VALUE_OPERATION_OK) {{ if ({result}.diagnostic.ptr != NULL) {{ phpc_native_diagnostic_message_stderr({result}.diagnostic); }} {result_error_exit} }}"
        ));
    }

    fn emit_native_array_lvalue_result_check(&mut self, result: &str, cleanup: &str) {
        let result_error_exit = self.native_error_exit(&format!(
            "{cleanup}phpc_native_array_lvalue_result_free({result}); "
        ));
        self.body.push(format!(
            "if ({result}.diagnostic.ptr != NULL) {{ phpc_native_diagnostic_report({result}.diagnostic); {result}.diagnostic.ptr = NULL; }}"
        ));
        self.body.push(format!(
            "if ({result}.tag != PHPC_NATIVE_ARRAY_LVALUE_OK) {{ {result_error_exit} }}"
        ));
    }

    fn materialize_native_array_key(
        &mut self,
        key: &Expr,
        failure_cleanup: &[String],
    ) -> CompileResult<CNativeArrayKeyMaterialization> {
        let key_failure_cleanup = c_cleanup_sequence(failure_cleanup);
        let key_value =
            self.materialize_native_array_expr_value_handle(key, &key_failure_cleanup)?;
        let result = self.next_native_name("array_key");
        self.body.push(format!(
            "phpc_NativeArrayKeyMaterializationResult {result} = phpc_native_value_to_array_key({});",
            key_value.handle
        ));
        self.body.extend(key_value.cleanup_after_use);
        if !failure_cleanup.is_empty() {
            let key_error_exit = self.native_error_exit(&format!(
                "phpc_native_array_key_materialization_result_free({result}); {}",
                c_cleanup_sequence(failure_cleanup)
            ));
            self.body.push(format!(
                "if ({result}.tag == 2) {{ phpc_native_diagnostic_message_stderr({result}.diagnostic); {key_error_exit} }}"
            ));
        }
        Ok(CNativeArrayKeyMaterialization {
            result: result.clone(),
            cleanup_after_use: vec![format!(
                "phpc_native_array_key_materialization_result_free({result});"
            )],
        })
    }

    fn try_emit_native_value_result_output(&mut self, expr: &Expr) -> CompileResult<bool> {
        if self.try_emit_static_primitive_arithmetic_output(expr)? {
            return Ok(true);
        }

        if !native_value_result_output_expr(expr) {
            return Ok(false);
        }

        let value = self.try_materialize_native_value_result_expr(expr, "")?;
        let Some(value) = value else {
            return Ok(false);
        };

        self.body
            .push(format!("phpc_native_value_echo_stdout({});", value.handle));
        self.body.extend(value.cleanup_after_use);
        Ok(true)
    }

    fn try_emit_array_index_output(&mut self, expr: &Expr) -> CompileResult<bool> {
        let Expr::Index { target, index, .. } = expr else {
            return Ok(false);
        };
        if self.is_string_offset_subject_expr(target) {
            return Ok(false);
        }
        if !self.uses_native_string_helpers {
            return Ok(false);
        }

        let target = self.emit_expr(target)?;
        let CValue::ArrayHandle(handle) = target else {
            return Err(self.unsupported(expr.span(), ASSEMBLY_ARRAY_REJECTION));
        };

        let key = self.materialize_native_array_key(index, &[])?;
        let diagnostic = self.next_native_name("array_read_diagnostic");
        let read = self.next_native_name("array_read");
        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {read} = phpc_native_array_read_key_with_diagnostic({handle}, {}, &{diagnostic});",
            key.result
        ));
        let read_error_exit = self.native_error_exit(&format!(
            "phpc_native_diagnostic_message_stderr({diagnostic}); phpc_native_diagnostic_free({diagnostic}); {}",
            c_cleanup_sequence(&key.cleanup_after_use)
        ));
        self.body.push(format!(
            "if ({diagnostic}.ptr != NULL) {{ {read_error_exit} }}"
        ));
        self.body.extend(key.cleanup_after_use);
        self.body
            .push(format!("phpc_native_value_echo_stdout({read});"));
        self.body.push(format!("phpc_native_value_free({read});"));
        Ok(true)
    }

    fn native_error_exit(&self, local_cleanup: &str) -> String {
        self.native_error_exit_with_code(local_cleanup, "1")
    }

    fn native_error_exit_with_code(&self, local_cleanup: &str, exit_code: &str) -> String {
        let mut cleanup = String::new();
        cleanup.push_str(local_cleanup);
        for handle in self.native_value_cleanup_handles.iter().rev() {
            cleanup.push_str(&format!(" phpc_native_value_free({handle});"));
        }
        for buffer in self.owned_native_byte_buffers.iter().rev() {
            cleanup.push_str(&format!(" phpc_native_byte_buffer_free({buffer});"));
        }
        for handle in self.array_cleanup_handles.iter().rev() {
            cleanup.push_str(&format!(" phpc_native_array_free({handle});"));
        }
        if let Some(handle) = &self.native_request_state_handle {
            cleanup.push_str(&format!(" phpc_native_request_state_free({handle});"));
        }
        cleanup.push_str(&format!(" return {exit_code};"));
        cleanup
    }

    fn emit_echo(&mut self, value: CValue, span: Span) -> CompileResult<()> {
        match value {
            CValue::Null if self.uses_native_string_helpers => {
                self.emit_native_scalar_helper_echo(0, None)
            }
            CValue::Bool(value) if self.uses_native_string_helpers => self
                .emit_native_scalar_helper_echo(1, Some(("bool_value", (value as u8).to_string()))),
            CValue::Null | CValue::Bool(false) => {}
            CValue::Bool(true) => self.emit_c_stdout_printf("printf(\"%s\", \"1\");"),
            CValue::BoolExpr(value) if self.uses_native_string_helpers => self
                .emit_native_scalar_helper_echo(
                    1,
                    Some(("bool_value", format!("(({value}) ? 1 : 0)"))),
                ),
            CValue::BoolExpr(value) => {
                self.emit_c_stdout_printf(format!("if ({value}) {{ printf(\"%s\", \"1\"); }}"))
            }
            CValue::ComparisonDecision(decision) => self.emit_c_stdout_printf(format!(
                "if ({}) {{ printf(\"%s\", \"1\"); }}",
                c_comparison_decision_bool_expr(&decision)
            )),
            CValue::Int(value) if self.uses_native_string_helpers => {
                self.emit_native_scalar_helper_echo(2, Some(("int_value", value)))
            }
            CValue::Int(value) => self.emit_c_stdout_printf(format!("printf(\"%lld\", {value});")),
            CValue::Float(value) if self.uses_native_string_helpers => {
                self.emit_native_scalar_helper_echo(3, Some(("float_value", value)))
            }
            CValue::Float(value) => self.emit_c_stdout_printf(format!("printf(\"%g\", {value});")),
            CValue::String(value) => {
                if self.uses_native_string_helpers {
                    self.emit_native_string_helper_echo(&value);
                } else {
                    self.emit_c_stdout_printf(format!("printf(\"%s\", \"{}\");", c_string(&value)));
                }
            }
            CValue::StringExpr(value) => {
                if self.uses_native_string_helpers {
                    let Some(len) = self.c_string_expr_byte_len_operand(&value) else {
                        return Err(Diagnostic::new(
                            Phase::Codegen,
                            0,
                            0,
                            format!("native executable string output requires a byte length before routing dynamic string pointer {value} through runtime helpers"),
                        ));
                    };
                    self.emit_native_string_pointer_helper_echo(&value, &len);
                } else {
                    self.emit_c_stdout_printf(format!("printf(\"%s\", {value});"));
                }
            }
            CValue::ArrayHandle(_) => return Err(self.unsupported(span, ASSEMBLY_ARRAY_REJECTION)),
            CValue::NativeValueHandle(handle) => {
                self.body
                    .push(format!("phpc_native_value_echo_stdout({handle});"));
            }
        }
        Ok(())
    }

    fn emit_native_scalar_helper_echo(&mut self, tag: u8, payload: Option<(&str, String)>) {
        let index = self.next_static_data;
        self.next_static_data += 1;
        self.body
            .push(format!("phpc_NativeScalarValue scalar_{index} = {{0}};"));
        self.body.push(format!("scalar_{index}.tag = {tag};"));
        if let Some((field, value)) = payload {
            self.body.push(format!("scalar_{index}.{field} = {value};"));
        }
        self.body.push(format!(
            "phpc_NativeValueHandle value_{index} = phpc_native_value_from_scalar(scalar_{index});"
        ));
        self.body
            .push(format!("phpc_native_value_echo_stdout(value_{index});"));
        self.body
            .push(format!("phpc_native_value_free(value_{index});"));
    }

    fn emit_c_stdout_printf(&mut self, line: impl Into<String>) {
        self.body.push(line.into());
        if self.uses_native_runtime_helpers() {
            self.body.push("fflush(stdout);".to_string());
        }
    }

    fn emit_native_string_helper_echo(&mut self, value: &str) {
        let index = self.next_static_data;
        self.next_static_data += 1;
        let bytes = c_byte_array(value.as_bytes());
        let data = format!("phpc_native_bytes_{index}");
        self.static_data
            .push(format!("static const uint8_t {data}[] = {{{bytes}}};"));
        self.emit_native_string_pointer_helper_echo(&data, &value.len().to_string());
    }

    fn emit_native_string_pointer_helper_echo(&mut self, value: &str, len: &str) {
        let index = self.next_static_data;
        self.next_static_data += 1;
        let data = format!("(const uint8_t *)({value})");
        self.body.push(format!(
            "phpc_NativeStringHandle string_{index} = phpc_native_string_from_bytes({data}, {len});"
        ));
        self.body.push(format!(
            "phpc_NativeDiagnosticHandle diagnostic_{index} = {{0}};"
        ));
        self.body.push(format!(
            "phpc_NativeValueHandle value_{index} = phpc_native_value_from_string_with_diagnostic(string_{index}, &diagnostic_{index});"
        ));
        self.body.push(format!(
            "if (value_{index}.ptr == NULL) {{ phpc_native_diagnostic_report(diagnostic_{index}); }} else {{ phpc_native_value_echo_stdout(value_{index}); }}"
        ));
        self.body
            .push(format!("phpc_native_value_free(value_{index});"));
        self.body
            .push(format!("phpc_native_string_free(string_{index});"));
    }

    fn native_call_diagnostics(&self) -> NativeCallDiagnostics {
        NativeCallDiagnostics::new(NativeCallBackend::Assembly)
    }

    fn unsupported_call_operation(&self, operation: NativeCallOperation) -> Diagnostic {
        self.native_call_diagnostics().operation(operation)
    }

    fn unsupported_direct_call(&self, span: Span, blocker: NativeCallBlocker) -> Diagnostic {
        self.native_call_diagnostics().direct_call(span, blocker)
    }

    fn unsupported_direct_named_call(
        &self,
        args: &[Expr],
        span: Span,
        fallback: &'static str,
    ) -> Diagnostic {
        native_direct_call_argument_result_operation(args, span)
            .map(|operation| self.unsupported_call_operation(operation))
            .unwrap_or_else(|| self.unsupported(span, fallback))
    }

    fn emit_binary_value_operand_exprs(
        &mut self,
        left: &Expr,
        right: &Expr,
    ) -> CompileResult<(CValue, CValue)> {
        let left_value = match self.emit_value_operand_expr(left) {
            Ok(value) => value,
            Err(error) => {
                return Err(
                    self.unsupported_unemitted_value_operands_or_original(&[left, right], error)
                );
            }
        };
        let right_value = self.emit_value_operand_expr(right)?;
        Ok((left_value, right_value))
    }

    fn emit_value_operand_expr(&mut self, expr: &Expr) -> CompileResult<CValue> {
        match self.emit_expr(expr) {
            Ok(value) => Ok(value),
            Err(error) => Err(self.unsupported_value_operand_or_original(expr, error)),
        }
    }

    fn unsupported_value_operand_or_fallback(
        &self,
        expr: &Expr,
        span: Span,
        fallback: &'static str,
    ) -> Diagnostic {
        native_value_operand_call_result_operation(expr)
            .map(|operation| self.unsupported_call_operation(operation))
            .unwrap_or_else(|| self.unsupported(span, fallback))
    }

    fn unsupported_value_operand_or_original(
        &self,
        expr: &Expr,
        original: Diagnostic,
    ) -> Diagnostic {
        native_failed_value_operand_call_result_operation(expr)
            .map(|operation| self.unsupported_call_operation(operation))
            .unwrap_or(original)
    }

    fn unsupported_unemitted_value_operands_or_original(
        &self,
        exprs: &[&Expr],
        original: Diagnostic,
    ) -> Diagnostic {
        native_unemitted_value_operand_list_call_operation(exprs)
            .map(|operation| self.unsupported_call_operation(operation))
            .unwrap_or(original)
    }

    fn unsupported_unemitted_statement_operands_or_original(
        &self,
        exprs: &[Expr],
        original: Diagnostic,
    ) -> Diagnostic {
        native_unemitted_statement_operand_list_call_operation(exprs)
            .map(|operation| self.unsupported_call_operation(operation))
            .unwrap_or(original)
    }

    fn unsupported_value_call(&self, expr: &Expr) -> Diagnostic {
        self.native_call_diagnostics().call_root(expr)
    }

    fn unsupported(&self, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Phase::Codegen, span.line, span.column, message)
    }
}

fn llvm_c_string(value: &str) -> String {
    let mut escaped = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'\\' => escaped.push_str("\\5C"),
            b'"' => escaped.push_str("\\22"),
            b'\n' => escaped.push_str("\\0A"),
            b'\r' => escaped.push_str("\\0D"),
            b'\t' => escaped.push_str("\\09"),
            0x20..=0x7e => escaped.push(*byte as char),
            other => escaped.push_str(&format!("\\{other:02X}")),
        }
    }
    escaped.push_str("\\00");
    escaped
}

fn c_string(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_ascii_graphic() || ch == ' ' => escaped.push(ch),
            ch => escaped.push_str(&format!("\\x{:02X}", ch as u32)),
        }
    }
    escaped
}

fn c_byte_array(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn array_index_expr_path(expr: &Expr) -> Option<(&Expr, Vec<&Expr>, Span)> {
    let Expr::Index {
        target,
        index,
        span,
    } = expr
    else {
        return None;
    };

    match target.as_ref() {
        nested @ Expr::Index { .. } => {
            let (root, mut indices, _) = array_index_expr_path(nested)?;
            indices.push(index.as_ref());
            Some((root, indices, *span))
        }
        root => Some((root, vec![index.as_ref()], *span)),
    }
}

fn native_foreach_body_may_mutate_storage(body: &[Stmt]) -> bool {
    body.iter().any(native_foreach_stmt_may_mutate_storage)
}

fn native_foreach_stmt_may_mutate_storage(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Echo { exprs, .. } => exprs.iter().any(native_foreach_expr_may_mutate_storage),
        Stmt::Print { expr, .. } | Stmt::Expr { expr, .. } => {
            native_foreach_expr_may_mutate_storage(expr)
        }
        _ => true,
    }
}

fn native_foreach_expr_may_mutate_storage(expr: &Expr) -> bool {
    match expr {
        Expr::Assign { .. }
        | Expr::CompoundAssign { .. }
        | Expr::NullCoalesceAssign { .. }
        | Expr::IncrementDecrement { .. }
        | Expr::Closure { .. } => true,
        Expr::Array { items, .. } => items.iter().any(|item| {
            item.key
                .as_ref()
                .is_some_and(native_foreach_expr_may_mutate_storage)
                || native_foreach_expr_may_mutate_storage(&item.value)
        }),
        Expr::Index { target, index, .. } => {
            native_foreach_expr_may_mutate_storage(target)
                || native_foreach_expr_may_mutate_storage(index)
        }
        Expr::AppendIndex { target, .. }
        | Expr::Property { target, .. }
        | Expr::ObjectStaticProperty { target, .. }
        | Expr::InstanceOf { expr: target, .. }
        | Expr::Clone { expr: target, .. }
        | Expr::Unary { expr: target, .. }
        | Expr::ErrorControl { expr: target, .. }
        | Expr::Include { path: target, .. }
        | Expr::Require { path: target, .. }
        | Expr::Cast { expr: target, .. } => native_foreach_expr_may_mutate_storage(target),
        Expr::DynamicProperty {
            target, property, ..
        } => {
            native_foreach_expr_may_mutate_storage(target)
                || native_foreach_expr_may_mutate_storage(property)
        }
        Expr::MethodCall { target, args, .. }
        | Expr::DynamicMethodCall { target, args, .. }
        | Expr::ObjectStaticMethodCall { target, args, .. } => {
            native_foreach_expr_may_mutate_storage(target)
                || args.iter().any(native_foreach_expr_may_mutate_storage)
        }
        Expr::DynamicCall { callee, args, .. } => {
            native_foreach_expr_may_mutate_storage(callee)
                || args.iter().any(native_foreach_expr_may_mutate_storage)
        }
        Expr::Call { args, .. }
        | Expr::ParentMethodCall { args, .. }
        | Expr::StaticMethodCall { args, .. }
        | Expr::SelfMethodCall { args, .. }
        | Expr::LateStaticMethodCall { args, .. }
        | Expr::New { args, .. } => args.iter().any(native_foreach_expr_may_mutate_storage),
        Expr::Binary { left, right, .. } => {
            native_foreach_expr_may_mutate_storage(left)
                || native_foreach_expr_may_mutate_storage(right)
        }
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            native_foreach_expr_may_mutate_storage(condition)
                || native_foreach_expr_may_mutate_storage(if_true)
                || native_foreach_expr_may_mutate_storage(if_false)
        }
        Expr::ShortTernary {
            condition,
            if_false,
            ..
        } => {
            native_foreach_expr_may_mutate_storage(condition)
                || native_foreach_expr_may_mutate_storage(if_false)
        }
        Expr::Null(_)
        | Expr::Bool(_, _)
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::String(_, _)
        | Expr::InterpolatedString { .. }
        | Expr::Variable(_, _)
        | Expr::MagicLine { .. }
        | Expr::MagicFile { .. }
        | Expr::MagicDir { .. }
        | Expr::MagicFunction { .. }
        | Expr::MagicClass { .. }
        | Expr::MagicMethod { .. }
        | Expr::GlobalConstant { .. }
        | Expr::ClassNameConstant { .. }
        | Expr::SelfClassNameConstant { .. }
        | Expr::ParentClassNameConstant { .. }
        | Expr::StaticClassNameConstant { .. }
        | Expr::ClassConstant { .. }
        | Expr::SelfClassConstant { .. }
        | Expr::ParentClassConstant { .. }
        | Expr::LateStaticClassConstant { .. }
        | Expr::StaticProperty { .. }
        | Expr::SelfStaticProperty { .. }
        | Expr::ParentStaticProperty { .. }
        | Expr::LateStaticProperty { .. } => false,
    }
}

fn is_comparison_op(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::StrictEq
            | BinaryOp::StrictNe
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge
    )
}

fn is_empty_string_literal(expr: &Expr) -> bool {
    matches!(expr, Expr::String(value, _) if value.is_empty())
}

fn same_direct_variable_expr(left: &Expr, right: &Expr) -> bool {
    matches!((left, right), (Expr::Variable(left, _), Expr::Variable(right, _)) if left == right)
}

fn same_direct_variable_ternary_expr(condition: &Expr, if_true: &Expr, if_false: &Expr) -> bool {
    same_direct_variable_expr(condition, if_true) && same_direct_variable_expr(condition, if_false)
}

fn llvm_comparison_rejection() -> &'static str {
    "LLVM comparison lowering rejects unsupported comparison operands until native PHP comparison coercions exist; same-type null, boolean, integer, finite float, known ASCII nonnumeric string comparisons, and identical string-pointer self-comparisons are lowered for the current native subset; phpc run handles current scalar comparison diagnostics"
}

fn assembly_comparison_rejection() -> &'static str {
    "assembly comparison lowering rejects unsupported comparison operands until native PHP comparison coercions exist; same-type null, boolean, integer, finite float, known ASCII nonnumeric string comparisons, and identical string-pointer self-comparisons are lowered for the current native subset; phpc run handles current scalar comparison diagnostics"
}

fn c_comparison_operator(op: BinaryOp) -> Option<&'static str> {
    Some(match op {
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        _ => return None,
    })
}

fn identical_string_ternary_branch(if_true: &IrValue, if_false: &IrValue) -> Option<IrValue> {
    match (if_true, if_false) {
        (IrValue::String(left), IrValue::String(right)) if left == right => {
            Some(IrValue::String(left.clone()))
        }
        (IrValue::StringPtr(left), IrValue::StringPtr(right)) if left == right => {
            Some(IrValue::StringPtr(left.clone()))
        }
        _ => None,
    }
}

fn identical_bool_expr_ternary_branch(if_true: &IrValue, if_false: &IrValue) -> Option<IrValue> {
    match (if_true, if_false) {
        (IrValue::BoolExpr(left), IrValue::BoolExpr(right)) if left == right => {
            Some(IrValue::BoolExpr(left.clone()))
        }
        _ => None,
    }
}

fn identical_c_string_ternary_branch(if_true: &CValue, if_false: &CValue) -> Option<CValue> {
    match (if_true, if_false) {
        (CValue::String(left), CValue::String(right)) if left == right => {
            Some(CValue::String(left.clone()))
        }
        (CValue::StringExpr(left), CValue::StringExpr(right)) if left == right => {
            Some(CValue::StringExpr(left.clone()))
        }
        _ => None,
    }
}

fn identical_c_bool_expr_ternary_branch(if_true: &CValue, if_false: &CValue) -> Option<CValue> {
    match (if_true, if_false) {
        (CValue::BoolExpr(left), CValue::BoolExpr(right)) if left == right => {
            Some(CValue::BoolExpr(left.clone()))
        }
        _ => None,
    }
}

enum BoolLiteralTernaryBranch {
    Static(bool),
    Reuse(String),
    Invert(String),
}

fn bool_literal_ternary_branch(
    condition: &str,
    if_true: &IrValue,
    if_false: &IrValue,
) -> Option<BoolLiteralTernaryBranch> {
    match (if_true, if_false) {
        (IrValue::Bool(true), IrValue::Bool(true)) => Some(BoolLiteralTernaryBranch::Static(true)),
        (IrValue::Bool(false), IrValue::Bool(false)) => {
            Some(BoolLiteralTernaryBranch::Static(false))
        }
        (IrValue::Bool(true), IrValue::Bool(false)) => {
            Some(BoolLiteralTernaryBranch::Reuse(condition.to_string()))
        }
        (IrValue::Bool(false), IrValue::Bool(true)) => {
            Some(BoolLiteralTernaryBranch::Invert(condition.to_string()))
        }
        _ => None,
    }
}

fn c_bool_literal_ternary_branch(
    condition: &str,
    if_true: &CValue,
    if_false: &CValue,
) -> Option<BoolLiteralTernaryBranch> {
    match (if_true, if_false) {
        (CValue::Bool(true), CValue::Bool(true)) => Some(BoolLiteralTernaryBranch::Static(true)),
        (CValue::Bool(false), CValue::Bool(false)) => Some(BoolLiteralTernaryBranch::Static(false)),
        (CValue::Bool(true), CValue::Bool(false)) => {
            Some(BoolLiteralTernaryBranch::Reuse(condition.to_string()))
        }
        (CValue::Bool(false), CValue::Bool(true)) => {
            Some(BoolLiteralTernaryBranch::Invert(condition.to_string()))
        }
        _ => None,
    }
}

fn llvm_string_comparison_predicate(op: BinaryOp) -> Option<&'static str> {
    Some(match op {
        BinaryOp::Eq | BinaryOp::StrictEq => "eq",
        BinaryOp::Ne | BinaryOp::StrictNe => "ne",
        BinaryOp::Lt => "slt",
        BinaryOp::Le => "sle",
        BinaryOp::Gt => "sgt",
        BinaryOp::Ge => "sge",
        _ => return None,
    })
}

fn c_string_comparison_operator(op: BinaryOp) -> Option<&'static str> {
    Some(match op {
        BinaryOp::Eq | BinaryOp::StrictEq => "==",
        BinaryOp::Ne | BinaryOp::StrictNe => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        _ => return None,
    })
}

fn known_string_pairs_are_safe_for_native_comparison(
    left_values: &KnownString,
    right_values: &KnownString,
) -> bool {
    known_string_values_have_native_comparison_safe_bytes(left_values)
        && known_string_values_have_native_comparison_safe_bytes(right_values)
        && known_string_pairs_use_binary_comparison(left_values, right_values)
}

fn known_string_values_have_native_comparison_safe_bytes(values: &KnownString) -> bool {
    values
        .values()
        .iter()
        .all(|value| value.bytes().all(|byte| byte.is_ascii() && byte != 0))
}

fn known_string_pairs_use_binary_comparison(
    left_values: &KnownString,
    right_values: &KnownString,
) -> bool {
    left_values.values().iter().all(|left| {
        right_values
            .values()
            .iter()
            .all(|right| !php_strings_use_numeric_comparison(left, right))
    })
}

fn string_comparison_result_for_known_values(
    left_values: &KnownString,
    op: BinaryOp,
    right_values: &KnownString,
) -> Option<bool> {
    let mut result = None;
    for left in left_values.values() {
        for right in right_values.values() {
            let ordering = left.cmp(right);
            let current = match op {
                BinaryOp::Eq => ordering.is_eq(),
                BinaryOp::Ne => !ordering.is_eq(),
                BinaryOp::Lt => ordering.is_lt(),
                BinaryOp::Le => ordering.is_lt() || ordering.is_eq(),
                BinaryOp::Gt => ordering.is_gt(),
                BinaryOp::Ge => ordering.is_gt() || ordering.is_eq(),
                _ => return None,
            };
            if result.is_some_and(|result| result != current) {
                return None;
            }
            result = Some(current);
        }
    }
    result
}

fn static_safe_string_comparison_result(
    left_values: Option<KnownString>,
    op: BinaryOp,
    right_values: Option<KnownString>,
) -> Option<bool> {
    let left_values = left_values?;
    let right_values = right_values?;
    if !known_string_pairs_are_safe_for_native_comparison(&left_values, &right_values) {
        return None;
    }
    string_comparison_result_for_known_values(&left_values, op, &right_values)
}

fn llvm_logical_rejection() -> &'static str {
    "LLVM logical lowering rejects unsupported logical operands until native PHP truthiness and short-circuit semantics exist; phpc run handles current logical operator behavior"
}

fn assembly_logical_rejection() -> &'static str {
    "assembly logical lowering rejects unsupported logical operands until native PHP truthiness and short-circuit semantics exist; phpc run handles current logical operator behavior"
}

fn static_strict_identity_result(is_identical: bool, op: BinaryOp) -> bool {
    match op {
        BinaryOp::StrictEq => is_identical,
        BinaryOp::StrictNe => !is_identical,
        _ => unreachable!("strict identity helper only accepts strict identity operators"),
    }
}

fn reflexive_string_comparison_result(op: BinaryOp) -> Option<bool> {
    Some(match op {
        BinaryOp::Eq | BinaryOp::Le | BinaryOp::Ge | BinaryOp::StrictEq => true,
        BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::StrictNe => false,
        _ => return None,
    })
}

enum BoolLiteralComparisonFold {
    Static(bool),
    Reuse(String),
    Invert(String),
}

fn bool_literal_comparison_fold(
    left: &str,
    op: BinaryOp,
    right: &str,
    true_literal: &str,
    false_literal: &str,
) -> Option<BoolLiteralComparisonFold> {
    let left_literal = bool_literal_value(left, true_literal, false_literal);
    let right_literal = bool_literal_value(right, true_literal, false_literal);
    match (left_literal, right_literal) {
        (Some(_), Some(_)) | (None, None) => return None,
        (None, Some(literal)) => bool_comparison_with_right_literal_fold(left, op, literal),
        (Some(literal), None) => bool_comparison_with_left_literal_fold(literal, op, right),
    }
}

fn bool_comparison_with_right_literal_fold(
    dynamic: &str,
    op: BinaryOp,
    literal: bool,
) -> Option<BoolLiteralComparisonFold> {
    match (op, literal) {
        (BinaryOp::Eq, true)
        | (BinaryOp::Ne, false)
        | (BinaryOp::Gt, false)
        | (BinaryOp::Ge, true) => Some(BoolLiteralComparisonFold::Reuse(dynamic.to_string())),
        (BinaryOp::Eq, false)
        | (BinaryOp::Ne, true)
        | (BinaryOp::Lt, true)
        | (BinaryOp::Le, false) => Some(BoolLiteralComparisonFold::Invert(dynamic.to_string())),
        (BinaryOp::Le, true) | (BinaryOp::Ge, false) => {
            Some(BoolLiteralComparisonFold::Static(true))
        }
        (BinaryOp::Lt, false) | (BinaryOp::Gt, true) => {
            Some(BoolLiteralComparisonFold::Static(false))
        }
        _ => None,
    }
}

fn bool_comparison_with_left_literal_fold(
    literal: bool,
    op: BinaryOp,
    dynamic: &str,
) -> Option<BoolLiteralComparisonFold> {
    match (op, literal) {
        (BinaryOp::Eq, true)
        | (BinaryOp::Ne, false)
        | (BinaryOp::Lt, false)
        | (BinaryOp::Le, true) => Some(BoolLiteralComparisonFold::Reuse(dynamic.to_string())),
        (BinaryOp::Eq, false)
        | (BinaryOp::Ne, true)
        | (BinaryOp::Gt, true)
        | (BinaryOp::Ge, false) => Some(BoolLiteralComparisonFold::Invert(dynamic.to_string())),
        (BinaryOp::Le, false) | (BinaryOp::Ge, true) => {
            Some(BoolLiteralComparisonFold::Static(true))
        }
        (BinaryOp::Lt, true) | (BinaryOp::Gt, false) => {
            Some(BoolLiteralComparisonFold::Static(false))
        }
        _ => None,
    }
}

fn bool_literal_value(value: &str, true_literal: &str, false_literal: &str) -> Option<bool> {
    if value == true_literal {
        Some(true)
    } else if value == false_literal {
        Some(false)
    } else {
        None
    }
}

fn null_comparison_result(op: BinaryOp) -> Option<bool> {
    Some(match op {
        BinaryOp::Eq | BinaryOp::Le | BinaryOp::Ge => true,
        BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Gt => false,
        _ => return None,
    })
}

fn bool_comparison_result(left: bool, op: BinaryOp, right: bool) -> Option<bool> {
    let left = u8::from(left);
    let right = u8::from(right);
    Some(match op {
        BinaryOp::Eq => left == right,
        BinaryOp::Ne => left != right,
        BinaryOp::Lt => left < right,
        BinaryOp::Le => left <= right,
        BinaryOp::Gt => left > right,
        BinaryOp::Ge => left >= right,
        _ => return None,
    })
}

fn integer_comparison_result(left: i64, op: BinaryOp, right: i64) -> Option<bool> {
    Some(match op {
        BinaryOp::Eq => left == right,
        BinaryOp::Ne => left != right,
        BinaryOp::Lt => left < right,
        BinaryOp::Le => left <= right,
        BinaryOp::Gt => left > right,
        BinaryOp::Ge => left >= right,
        _ => return None,
    })
}

fn float_comparison_result(left: f64, op: BinaryOp, right: f64) -> Option<bool> {
    if !left.is_finite() || !right.is_finite() {
        return None;
    }
    Some(match op {
        BinaryOp::Eq => left == right,
        BinaryOp::Ne => left != right,
        BinaryOp::Lt => left < right,
        BinaryOp::Le => left <= right,
        BinaryOp::Gt => left > right,
        BinaryOp::Ge => left >= right,
        _ => return None,
    })
}

fn llvm_bool_operand(value: IrValue) -> Option<String> {
    match value {
        IrValue::Bool(true) => Some("true".to_string()),
        IrValue::Bool(false) => Some("false".to_string()),
        IrValue::BoolExpr(value) => Some(value),
        _ => None,
    }
}

fn c_bool_operand(value: CValue) -> Option<String> {
    match value {
        CValue::Bool(true) => Some("1".to_string()),
        CValue::Bool(false) => Some("0".to_string()),
        CValue::BoolExpr(value) => Some(value),
        CValue::ComparisonDecision(decision) => Some(c_comparison_decision_bool_expr(&decision)),
        _ => None,
    }
}

fn c_comparison_decision_bool_expr(decision: &str) -> String {
    format!("phpc_native_comparison_branch_decision_is_true({decision})")
}

fn c_string_operand(value: CValue) -> String {
    match value {
        CValue::String(value) => format!("\"{}\"", c_string(&value)),
        CValue::StringExpr(value) => value,
        _ => unreachable!("string operands are prefiltered"),
    }
}

fn logical_truthiness_result(left: bool, op: BinaryOp, right: bool) -> CompileResult<bool> {
    Ok(match op {
        BinaryOp::LogicalAnd => left && right,
        BinaryOp::LogicalOr => left || right,
        BinaryOp::LogicalXor => left ^ right,
        _ => unreachable!("logical operands are prefiltered"),
    })
}

fn known_integer_truthiness(values: &Option<KnownInt>) -> Option<bool> {
    let values = values.as_ref()?;
    known_truthiness(values.values().iter().map(|value| *value != 0))
}

fn known_float_truthiness(values: &Option<KnownFloat>) -> Option<bool> {
    let values = values.as_ref()?;
    if !values.values().iter().all(|value| value.is_finite()) {
        return None;
    }
    known_truthiness(values.values().iter().map(|value| *value != 0.0))
}

fn known_string_truthiness(values: &KnownString) -> Option<bool> {
    known_truthiness(
        values
            .values()
            .iter()
            .map(|value| is_php_truthy_string(value)),
    )
}

fn known_truthiness(values: impl IntoIterator<Item = bool>) -> Option<bool> {
    let mut result = None;
    for current in values {
        if result.is_some_and(|result| result != current) {
            return None;
        }
        result = Some(current);
    }
    result
}

fn is_global_constant_builtin(name: &str) -> bool {
    name.eq_ignore_ascii_case("define")
        || name.eq_ignore_ascii_case("constant")
        || name.eq_ignore_ascii_case("defined")
}

fn is_object_metadata_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "get_class"
            | "is_object"
            | "get_debug_type"
            | "class_exists"
            | "interface_exists"
            | "trait_exists"
            | "enum_exists"
            | "get_declared_classes"
            | "get_declared_interfaces"
            | "get_declared_traits"
            | "class_implements"
            | "class_uses"
            | "class_parents"
            | "get_called_class"
            | "spl_object_id"
            | "spl_object_hash"
            | "property_exists"
            | "method_exists"
            | "get_class_methods"
            | "get_class_vars"
            | "get_object_vars"
            | "get_mangled_object_vars"
            | "is_a"
            | "is_subclass_of"
            | "get_parent_class"
    )
}

fn is_array_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "count"
            | "array_key_exists"
            | "array_values"
            | "array_key_first"
            | "array_key_last"
            | "current"
            | "key"
            | "next"
            | "prev"
            | "reset"
            | "end"
            | "array_is_list"
            | "array_keys"
            | "array_reverse"
            | "array_slice"
            | "array_chunk"
            | "array_pad"
            | "array_merge"
            | "array_replace"
            | "array_flip"
            | "array_change_key_case"
            | "array_column"
            | "array_fill_keys"
            | "array_combine"
            | "array_intersect_key"
            | "array_diff_key"
            | "array_diff"
            | "array_intersect"
            | "array_unique"
            | "array_count_values"
            | "array_sum"
            | "array_product"
            | "array_reduce"
            | "array_filter"
            | "array_map"
            | "sort"
            | "rsort"
            | "asort"
            | "arsort"
            | "ksort"
            | "krsort"
            | "natsort"
            | "natcasesort"
            | "usort"
            | "uasort"
            | "uksort"
            | "array_unshift"
            | "array_pop"
            | "in_array"
            | "array_search"
    )
}

fn is_native_type_introspection_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "gettype"
            | "is_null"
            | "is_bool"
            | "is_int"
            | "is_integer"
            | "is_long"
            | "is_float"
            | "is_double"
            | "is_string"
            | "is_array"
            | "is_scalar"
            | "is_numeric"
            | "is_countable"
            | "is_iterable"
            | "extension_loaded"
            | "is_object"
            | "get_debug_type"
            | "class_exists"
            | "interface_exists"
            | "trait_exists"
            | "enum_exists"
            | "property_exists"
            | "method_exists"
            | "is_a"
            | "is_subclass_of"
    )
}

fn is_exit_construct_name(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(), "exit" | "die")
}

fn is_native_metadata_exists_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "class_exists" | "interface_exists" | "trait_exists" | "enum_exists"
    )
}

fn is_native_member_metadata_exists_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "property_exists" | "method_exists"
    )
}

fn is_native_relationship_metadata_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "is_a" | "is_subclass_of"
    )
}

fn is_builtin_class_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "exception" | "pdo" | "pdostatement" | "reflectionclass" | "reflectionmethod"
    )
}

fn known_strings_have_uniform_function_exists_result(values: &KnownString) -> Option<bool> {
    let mut result = None;
    for value in values.values() {
        let current = is_native_known_function_name(value);
        if let Some(previous) = result {
            if previous != current {
                return None;
            }
        } else {
            result = Some(current);
        }
    }
    result
}

fn native_string_predicate_for_name(name: &str) -> Option<NativeStringPredicate> {
    match name.to_ascii_lowercase().as_str() {
        "str_starts_with" => Some(NativeStringPredicate::StartsWith),
        "str_ends_with" => Some(NativeStringPredicate::EndsWith),
        "str_contains" => Some(NativeStringPredicate::Contains),
        _ => None,
    }
}

fn native_string_int_operation_for_name(name: &str) -> Option<NativeStringIntOperation> {
    match name.to_ascii_lowercase().as_str() {
        "strcasecmp" => Some(NativeStringIntOperation::CaseCompare),
        "strcmp" => Some(NativeStringIntOperation::ByteCompare),
        "strncmp" => Some(NativeStringIntOperation::BytePrefixCompare),
        "strncasecmp" => Some(NativeStringIntOperation::CasePrefixCompare),
        "substr_count" => Some(NativeStringIntOperation::SubstrCount),
        "ord" => Some(NativeStringIntOperation::Ordinal),
        "crc32" => Some(NativeStringIntOperation::Crc32),
        _ => None,
    }
}

fn native_string_distance_operation_for_name(name: &str) -> Option<NativeStringDistanceOperation> {
    match name.to_ascii_lowercase().as_str() {
        "levenshtein" => Some(NativeStringDistanceOperation::Levenshtein),
        "similar_text" => Some(NativeStringDistanceOperation::SimilarText),
        _ => None,
    }
}

fn native_string_result_operation_for_name(name: &str) -> Option<NativeStringResultOperation> {
    match name.to_ascii_lowercase().as_str() {
        "strrev" => Some(NativeStringResultOperation::Reverse),
        "bin2hex" => Some(NativeStringResultOperation::BinToHex),
        "str_rot13" => Some(NativeStringResultOperation::Rot13),
        "strtolower" => Some(NativeStringResultOperation::AsciiLower),
        "strtoupper" => Some(NativeStringResultOperation::AsciiUpper),
        "ucfirst" => Some(NativeStringResultOperation::AsciiFirstUpper),
        "lcfirst" => Some(NativeStringResultOperation::AsciiFirstLower),
        "escapeshellarg" => Some(NativeStringResultOperation::ShellArgEscape),
        "escapeshellcmd" => Some(NativeStringResultOperation::ShellCommandEscape),
        _ => None,
    }
}

fn native_string_result_operation_prefix(operation: NativeStringResultOperation) -> &'static str {
    match operation {
        NativeStringResultOperation::Reverse => "strrev_result",
        NativeStringResultOperation::BinToHex => "bin2hex_result",
        NativeStringResultOperation::Rot13 => "str_rot13_result",
        NativeStringResultOperation::AsciiLower => "strtolower_result",
        NativeStringResultOperation::AsciiUpper => "strtoupper_result",
        NativeStringResultOperation::AsciiFirstUpper => "ucfirst_result",
        NativeStringResultOperation::AsciiFirstLower => "lcfirst_result",
        NativeStringResultOperation::ShellArgEscape => "escapeshellarg_result",
        NativeStringResultOperation::ShellCommandEscape => "escapeshellcmd_result",
    }
}

fn native_filesystem_path_operation_for_name(name: &str) -> Option<NativeFilesystemPathOperation> {
    match name.to_ascii_lowercase().as_str() {
        "file_get_contents" => Some(NativeFilesystemPathOperation::FileGetContents),
        "realpath" => Some(NativeFilesystemPathOperation::Realpath),
        "file_exists" => Some(NativeFilesystemPathOperation::FileExists),
        "is_dir" => Some(NativeFilesystemPathOperation::IsDir),
        "is_file" => Some(NativeFilesystemPathOperation::IsFile),
        "is_readable" => Some(NativeFilesystemPathOperation::IsReadable),
        "is_writable" => Some(NativeFilesystemPathOperation::IsWritable),
        "is_link" => Some(NativeFilesystemPathOperation::IsLink),
        "filesize" => Some(NativeFilesystemPathOperation::FileSize),
        "filemtime" => Some(NativeFilesystemPathOperation::FileMTime),
        "getcwd" => Some(NativeFilesystemPathOperation::GetCwd),
        "clearstatcache" => Some(NativeFilesystemPathOperation::ClearStatCache),
        "realpath_cache_get" => Some(NativeFilesystemPathOperation::RealpathCacheGet),
        "realpath_cache_size" => Some(NativeFilesystemPathOperation::RealpathCacheSize),
        _ => None,
    }
}

fn native_filesystem_path_operation_result_prefix(
    operation: NativeFilesystemPathOperation,
) -> &'static str {
    match operation {
        NativeFilesystemPathOperation::FileGetContents => "file_get_contents_value",
        NativeFilesystemPathOperation::Realpath => "realpath_value",
        NativeFilesystemPathOperation::FileExists => "file_exists_value",
        NativeFilesystemPathOperation::IsDir => "is_dir_value",
        NativeFilesystemPathOperation::IsFile => "is_file_value",
        NativeFilesystemPathOperation::IsReadable => "is_readable_value",
        NativeFilesystemPathOperation::IsWritable => "is_writable_value",
        NativeFilesystemPathOperation::IsLink => "is_link_value",
        NativeFilesystemPathOperation::FileSize => "filesize_value",
        NativeFilesystemPathOperation::FileMTime => "filemtime_value",
        NativeFilesystemPathOperation::GetCwd => "getcwd_value",
        NativeFilesystemPathOperation::ClearStatCache => "clearstatcache_value",
        NativeFilesystemPathOperation::RealpathCacheGet => "realpath_cache_get_value",
        NativeFilesystemPathOperation::RealpathCacheSize => "realpath_cache_size_value",
    }
}

fn native_filesystem_path_operation_llvm_rejection(
    operation: NativeFilesystemPathOperation,
) -> &'static str {
    match operation {
        NativeFilesystemPathOperation::FileGetContents => LLVM_FILE_GET_CONTENTS_REJECTION,
        NativeFilesystemPathOperation::Realpath => LLVM_REALPATH_REJECTION,
        NativeFilesystemPathOperation::IsWritable => LLVM_IS_WRITABLE_REJECTION,
        NativeFilesystemPathOperation::GetCwd => LLVM_GETCWD_REJECTION,
        NativeFilesystemPathOperation::ClearStatCache => LLVM_CLEARSTATCACHE_REJECTION,
        NativeFilesystemPathOperation::RealpathCacheGet
        | NativeFilesystemPathOperation::RealpathCacheSize => {
            LLVM_FILESYSTEM_PATH_OPERATION_REJECTION
        }
        NativeFilesystemPathOperation::FileExists
        | NativeFilesystemPathOperation::IsDir
        | NativeFilesystemPathOperation::IsFile
        | NativeFilesystemPathOperation::IsReadable
        | NativeFilesystemPathOperation::IsLink
        | NativeFilesystemPathOperation::FileSize
        | NativeFilesystemPathOperation::FileMTime => LLVM_FUNCTION_CALL_REJECTION,
    }
}

fn native_filesystem_path_operation_assembly_rejection(
    operation: NativeFilesystemPathOperation,
) -> &'static str {
    match operation {
        NativeFilesystemPathOperation::FileGetContents => ASSEMBLY_FILE_GET_CONTENTS_REJECTION,
        NativeFilesystemPathOperation::Realpath => ASSEMBLY_REALPATH_REJECTION,
        NativeFilesystemPathOperation::IsWritable => ASSEMBLY_IS_WRITABLE_REJECTION,
        NativeFilesystemPathOperation::GetCwd => ASSEMBLY_GETCWD_REJECTION,
        NativeFilesystemPathOperation::ClearStatCache => ASSEMBLY_CLEARSTATCACHE_REJECTION,
        NativeFilesystemPathOperation::RealpathCacheGet
        | NativeFilesystemPathOperation::RealpathCacheSize => {
            ASSEMBLY_FILESYSTEM_PATH_OPERATION_REJECTION
        }
        NativeFilesystemPathOperation::FileExists
        | NativeFilesystemPathOperation::IsDir
        | NativeFilesystemPathOperation::IsFile
        | NativeFilesystemPathOperation::IsReadable
        | NativeFilesystemPathOperation::IsLink
        | NativeFilesystemPathOperation::FileSize
        | NativeFilesystemPathOperation::FileMTime => ASSEMBLY_FILESYSTEM_PATH_OPERATION_REJECTION,
    }
}

fn known_strings_have_uniform_byte_length(values: &KnownString) -> Option<usize> {
    let mut result = None;
    for value in values.values() {
        let current = value.len();
        if let Some(previous) = result {
            if previous != current {
                return None;
            }
        } else {
            result = Some(current);
        }
    }
    result
}

fn known_strings_are_nul_free(values: &KnownString) -> bool {
    values.values().iter().all(|value| !value.contains('\0'))
}

fn known_strings_have_uniform_defined_result(values: &KnownString) -> Option<bool> {
    let mut result = None;
    for value in values.values() {
        let current = native_defined_result(value)?;
        if let Some(previous) = result {
            if previous != current {
                return None;
            }
        } else {
            result = Some(current);
        }
    }
    result
}

fn native_defined_result(name: &str) -> Option<bool> {
    if !is_supported_native_constant_name(name) {
        return None;
    }

    Some(builtin_global_constant_is_defined(name))
}

fn builtin_global_constant_is_defined(name: &str) -> bool {
    match name {
        "PHP_VERSION"
        | "PHP_VERSION_ID"
        | "PHP_INT_MAX"
        | "PHP_SAPI"
        | "PATH_SEPARATOR"
        | "PHP_SESSION_DISABLED"
        | "PHP_SESSION_NONE"
        | "PHP_SESSION_ACTIVE"
        | "E_ERROR"
        | "E_WARNING"
        | "E_PARSE"
        | "E_NOTICE"
        | "E_CORE_ERROR"
        | "E_CORE_WARNING"
        | "E_COMPILE_ERROR"
        | "E_COMPILE_WARNING"
        | "E_USER_ERROR"
        | "E_USER_WARNING"
        | "E_USER_NOTICE"
        | "E_STRICT"
        | "E_RECOVERABLE_ERROR"
        | "E_DEPRECATED"
        | "E_USER_DEPRECATED"
        | "E_ALL"
        | "CASE_LOWER"
        | "CASE_UPPER"
        | "ARRAY_FILTER_USE_BOTH"
        | "ARRAY_FILTER_USE_KEY"
        | "PREG_SPLIT_DELIM_CAPTURE"
        | "SORT_REGULAR"
        | "SORT_NUMERIC"
        | "SORT_STRING"
        | "SEEK_SET"
        | "SEEK_CUR"
        | "SEEK_END"
        | "MYSQLI_REPORT_OFF"
        | "MYSQLI_REPORT_ERROR"
        | "MYSQLI_REPORT_STRICT"
        | "MYSQLI_ASSOC"
        | "MYSQLI_NUM"
        | "MYSQLI_BOTH"
        | "MYSQLI_ASYNC"
        | "MYSQLI_CLIENT_SSL"
        | "MYSQLI_CLIENT_COMPRESS"
        | "MYSQLI_CLIENT_INTERACTIVE"
        | "MYSQLI_CLIENT_IGNORE_SPACE"
        | "MYSQLI_CLIENT_NO_SCHEMA"
        | "MYSQLI_CLIENT_FOUND_ROWS"
        | "MYSQLI_CLIENT_SSL_VERIFY_SERVER_CERT"
        | "MYSQLI_CLIENT_SSL_DONT_VERIFY_SERVER_CERT"
        | "MYSQLI_CLIENT_CAN_HANDLE_EXPIRED_PASSWORDS"
        | "MYSQLI_OPT_CONNECT_TIMEOUT"
        | "MYSQLI_OPT_LOCAL_INFILE"
        | "MYSQLI_OPT_LOAD_DATA_LOCAL_DIR"
        | "MYSQLI_INIT_COMMAND"
        | "MYSQLI_OPT_READ_TIMEOUT"
        | "MYSQLI_OPT_NET_CMD_BUFFER_SIZE"
        | "MYSQLI_OPT_NET_READ_BUFFER_SIZE"
        | "MYSQLI_OPT_INT_AND_FLOAT_NATIVE"
        | "MYSQLI_OPT_SSL_VERIFY_SERVER_CERT"
        | "MYSQLI_OPT_CAN_HANDLE_EXPIRED_PASSWORDS"
        | "MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH"
        | "MYSQLI_STMT_ATTR_CURSOR_TYPE"
        | "MYSQLI_STMT_ATTR_PREFETCH_ROWS"
        | "MYSQLI_CURSOR_TYPE_NO_CURSOR"
        | "MYSQLI_CURSOR_TYPE_READ_ONLY"
        | "MYSQLI_CURSOR_TYPE_FOR_UPDATE"
        | "MYSQLI_CURSOR_TYPE_SCROLLABLE"
        | "MYSQLI_REFRESH_GRANT"
        | "MYSQLI_REFRESH_LOG"
        | "MYSQLI_REFRESH_TABLES"
        | "MYSQLI_REFRESH_HOSTS"
        | "MYSQLI_REFRESH_STATUS"
        | "MYSQLI_REFRESH_THREADS"
        | "MYSQLI_REFRESH_SLAVE"
        | "MYSQLI_REFRESH_REPLICA"
        | "MYSQLI_REFRESH_MASTER"
        | "MYSQLI_REFRESH_BACKUP_LOG" => true,
        _ => false,
    }
}

fn is_supported_native_constant_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn is_native_known_function_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "define"
            | "strlen"
            | "strtolower"
            | "trim"
            | "ltrim"
            | "rtrim"
            | "strcasecmp"
            | "strcmp"
            | "strncmp"
            | "strncasecmp"
            | "str_contains"
            | "str_starts_with"
            | "str_ends_with"
            | "strpos"
            | "substr"
            | "substr_count"
            | "str_replace"
            | "levenshtein"
            | "similar_text"
            | "preg_match"
            | "preg_replace"
            | "preg_split"
            | "preg_replace_callback"
            | "compact"
            | "error_reporting"
            | "ignore_user_abort"
            | "php_sapi_name"
            | "sprintf"
            | "vsprintf"
            | "call_user_func"
            | "call_user_func_array"
            | "implode"
            | "basename"
            | "dirname"
            | "abs"
            | "version_compare"
            | "microtime"
            | "date_default_timezone_set"
            | "ini_get"
            | "ini_set"
            | "get_include_path"
            | "set_include_path"
            | "min"
            | "rand"
            | "uniqid"
            | "hash_hmac"
            | "count"
            | "constant"
            | "defined"
            | "array_key_exists"
            | "array_values"
            | "array_key_first"
            | "array_key_last"
            | "current"
            | "key"
            | "next"
            | "prev"
            | "reset"
            | "end"
            | "array_is_list"
            | "array_keys"
            | "array_reverse"
            | "array_slice"
            | "array_chunk"
            | "array_pad"
            | "array_merge"
            | "array_replace"
            | "array_flip"
            | "array_change_key_case"
            | "array_column"
            | "array_fill_keys"
            | "array_combine"
            | "array_intersect_key"
            | "array_diff_key"
            | "array_diff"
            | "array_intersect"
            | "array_unique"
            | "array_count_values"
            | "array_sum"
            | "array_product"
            | "array_reduce"
            | "array_filter"
            | "array_map"
            | "sort"
            | "rsort"
            | "asort"
            | "arsort"
            | "ksort"
            | "krsort"
            | "natsort"
            | "natcasesort"
            | "usort"
            | "uasort"
            | "uksort"
            | "array_unshift"
            | "array_pop"
            | "in_array"
            | "array_search"
            | "gettype"
            | "is_null"
            | "is_bool"
            | "is_int"
            | "is_integer"
            | "is_long"
            | "is_float"
            | "is_double"
            | "is_string"
            | "is_array"
            | "is_scalar"
            | "is_numeric"
            | "is_countable"
            | "is_iterable"
            | "is_callable"
            | "function_exists"
            | "extension_loaded"
            | "class_alias"
            | "mysqli_connect"
            | "mysqli_real_connect"
            | "mysqli_get_server_info"
            | "mysqli_get_server_version"
            | "mysqli_get_host_info"
            | "mysqli_get_client_info"
            | "mysqli_get_client_version"
            | "mysqli_get_proto_info"
            | "mysqli_thread_id"
            | "mysqli_kill"
            | "mysqli_change_user"
            | "mysqli_refresh"
            | "mysqli_get_charset"
            | "mysqli_character_set_name"
            | "mysqli_field_count"
            | "mysqli_close"
            | "mysqli_options"
            | "mysqli_set_opt"
            | "mysqli_ssl_set"
            | "mysqli_connect_errno"
            | "mysqli_connect_error"
            | "mysqli_error_list"
            | "mysqli_get_connection_stats"
            | "mysqli_get_links_stats"
            | "mysqli_get_client_stats"
            | "mysqli_thread_safe"
            | "mysqli_stmt_init"
            | "mysqli_prepare"
            | "mysqli_stmt_prepare"
            | "mysqli_stmt_param_count"
            | "mysqli_stmt_get_warnings"
            | "mysqli_stmt_error_list"
            | "mysqli_stmt_bind_param"
            | "mysqli_stmt_bind_result"
            | "mysqli_stmt_execute"
            | "mysqli_execute"
            | "mysqli_stmt_get_result"
            | "mysqli_stmt_close"
            | "mysqli_stmt_errno"
            | "mysqli_stmt_error"
            | "mysqli_stmt_affected_rows"
            | "mysqli_stmt_store_result"
            | "mysqli_stmt_num_rows"
            | "mysqli_stmt_fetch"
            | "mysqli_stmt_result_metadata"
            | "mysqli_stmt_field_count"
            | "mysqli_stmt_free_result"
            | "mysqli_stmt_data_seek"
            | "mysqli_stmt_attr_get"
            | "mysqli_stmt_attr_set"
            | "mysqli_stmt_send_long_data"
            | "mysqli_stmt_reset"
            | "mysqli_stmt_more_results"
            | "mysqli_stmt_next_result"
            | "mysqli_stmt_sqlstate"
            | "mysqli_stmt_warning_count"
            | "mysqli_stmt_insert_id"
            | "mysqli_execute_query"
            | "mysqli_dump_debug_info"
            | "mysqli_debug"
            | "mysqli_stat"
            | "mysqli_autocommit"
            | "mysqli_begin_transaction"
            | "mysqli_commit"
            | "mysqli_rollback"
            | "mysqli_savepoint"
            | "mysqli_release_savepoint"
            | "mysqli_set_charset"
            | "mysqli_query"
            | "mysqli_real_query"
            | "mysqli_multi_query"
            | "mysqli_errno"
            | "mysqli_error"
            | "mysqli_sqlstate"
            | "mysqli_warning_count"
            | "mysqli_info"
            | "mysqli_get_warnings"
            | "mysqli_affected_rows"
            | "mysqli_insert_id"
            | "mysqli_ping"
            | "mysqli_select_db"
            | "mysqli_real_escape_string"
            | "mysqli_escape_string"
            | "mysqli_fetch_object"
            | "mysqli_fetch_assoc"
            | "mysqli_fetch_row"
            | "mysqli_fetch_array"
            | "mysqli_fetch_all"
            | "mysqli_fetch_column"
            | "mysqli_fetch_field"
            | "mysqli_fetch_fields"
            | "mysqli_fetch_field_direct"
            | "mysqli_num_fields"
            | "mysqli_num_rows"
            | "mysqli_fetch_lengths"
            | "mysqli_data_seek"
            | "mysqli_field_seek"
            | "mysqli_field_tell"
            | "mysqli_free_result"
            | "mysqli_more_results"
            | "mysqli_next_result"
            | "mysqli_store_result"
            | "mysqli_use_result"
            | "mysqli_reap_async_query"
            | "mysqli_poll"
            | "mysqli_report"
            | "mysqli_init"
            | "is_uploaded_file"
            | "move_uploaded_file"
            | "file_exists"
            | "file_get_contents"
            | "fopen"
            | "stream_context_create"
            | "stream_context_get_options"
            | "stream_context_get_params"
            | "stream_context_get_default"
            | "stream_context_set_default"
            | "stream_context_set_option"
            | "stream_context_set_params"
            | "fwrite"
            | "fread"
            | "rewind"
            | "stream_get_contents"
            | "feof"
            | "ftell"
            | "fseek"
            | "fstat"
            | "stream_get_meta_data"
            | "fclose"
            | "opendir"
            | "readdir"
            | "rewinddir"
            | "closedir"
            | "filesize"
            | "filemtime"
            | "realpath"
            | "realpath_cache_get"
            | "realpath_cache_size"
            | "getcwd"
            | "is_dir"
            | "is_file"
            | "is_readable"
            | "is_writable"
            | "is_link"
            | "clearstatcache"
            | "register_shutdown_function"
            | "set_error_handler"
            | "restore_error_handler"
            | "ob_start"
            | "ob_get_level"
            | "ob_get_contents"
            | "ob_get_length"
            | "ob_list_handlers"
            | "ob_get_status"
            | "ob_get_clean"
            | "ob_get_flush"
            | "ob_clean"
            | "ob_flush"
            | "ob_end_clean"
            | "ob_end_flush"
            | "header"
            | "header_remove"
            | "headers_list"
            | "headers_sent"
            | "http_response_code"
            | "setcookie"
            | "setrawcookie"
            | "session_start"
            | "session_status"
            | "session_cache_limiter"
            | "session_cache_expire"
            | "session_id"
            | "session_write_close"
            | "assert"
            | "get_class"
            | "is_object"
            | "get_debug_type"
            | "class_exists"
            | "interface_exists"
            | "trait_exists"
            | "enum_exists"
            | "get_declared_classes"
            | "get_declared_interfaces"
            | "get_declared_traits"
            | "class_implements"
            | "class_uses"
            | "class_parents"
            | "get_called_class"
            | "spl_object_id"
            | "spl_object_hash"
            | "spl_autoload"
            | "spl_autoload_register"
            | "spl_autoload_functions"
            | "spl_autoload_extensions"
            | "spl_autoload_unregister"
            | "spl_autoload_call"
            | "property_exists"
            | "method_exists"
            | "get_class_methods"
            | "get_class_vars"
            | "get_object_vars"
            | "get_mangled_object_vars"
            | "is_a"
            | "is_subclass_of"
            | "get_parent_class"
            | "var_dump"
            | "print_r"
    )
}

fn is_compat_loaded_extension_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "json" | "hash" | "pdo" | "pdo_mysql"
    )
}

fn known_strings_have_uniform_numeric_result(values: &KnownString) -> Option<bool> {
    let mut result = None;
    for value in values.values() {
        let current = classify_php_numeric_string(value).is_numeric();
        if let Some(previous) = result {
            if previous != current {
                return None;
            }
        } else {
            result = Some(current);
        }
    }
    result
}

fn llvm_gettype_name(value: &IrValue) -> &'static str {
    match value {
        IrValue::Null => "NULL",
        IrValue::Bool(_) | IrValue::BoolExpr(_) => "boolean",
        IrValue::Int(_) => "integer",
        IrValue::Float(_) => "double",
        IrValue::String(_) | IrValue::StringPtr(_) => "string",
        IrValue::NativeValue(_) => "unknown",
    }
}

fn llvm_debug_type_name(value: &IrValue) -> &'static str {
    match value {
        IrValue::Null => "null",
        IrValue::Bool(_) | IrValue::BoolExpr(_) => "bool",
        IrValue::Int(_) => "int",
        IrValue::Float(_) => "float",
        IrValue::String(_) | IrValue::StringPtr(_) => "string",
        IrValue::NativeValue(_) => "unknown",
    }
}

fn c_gettype_name(value: &CValue) -> &'static str {
    match value {
        CValue::Null => "NULL",
        CValue::Bool(_) | CValue::BoolExpr(_) | CValue::ComparisonDecision(_) => "boolean",
        CValue::Int(_) => "integer",
        CValue::Float(_) => "double",
        CValue::String(_) | CValue::StringExpr(_) => "string",
        CValue::ArrayHandle(_) => "array",
        CValue::NativeValueHandle(_) => "unknown",
    }
}

fn c_debug_type_name(value: &CValue) -> &'static str {
    match value {
        CValue::Null => "null",
        CValue::Bool(_) | CValue::BoolExpr(_) | CValue::ComparisonDecision(_) => "bool",
        CValue::Int(_) => "int",
        CValue::Float(_) => "float",
        CValue::String(_) | CValue::StringExpr(_) => "string",
        CValue::ArrayHandle(_) => "array",
        CValue::NativeValueHandle(_) => "unknown",
    }
}

fn format_float_literal(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ArrayItem, FunctionParam, StaticLocalDeclarator};

    #[test]
    fn native_comparison_abi_opcodes_follow_runtime_contract() {
        for (binary_op, runtime_op) in [
            (BinaryOp::Eq, NativeComparisonOp::LooseEq),
            (BinaryOp::Ne, NativeComparisonOp::LooseNe),
            (BinaryOp::Lt, NativeComparisonOp::LooseLt),
            (BinaryOp::Le, NativeComparisonOp::LooseLe),
            (BinaryOp::Gt, NativeComparisonOp::LooseGt),
            (BinaryOp::Ge, NativeComparisonOp::LooseGe),
            (BinaryOp::StrictEq, NativeComparisonOp::StrictEq),
            (BinaryOp::StrictNe, NativeComparisonOp::StrictNe),
        ] {
            let codegen_op = native_comparison_op_for_binary_op(binary_op)
                .expect("comparison operator should map to native ABI opcode");
            assert_eq!(codegen_op, runtime_op);
            assert_eq!(
                native_comparison_c_uint8_argument(codegen_op),
                (runtime_op as u8).to_string()
            );
        }

        assert!(native_comparison_op_for_binary_op(BinaryOp::Add).is_none());
    }

    fn test_span() -> Span {
        Span::new(1, 1)
    }

    #[test]
    fn known_string_truthiness_reuses_runtime_semantics_across_value_families() {
        for value in ["", "0", "00", "0.0", " ", "false"] {
            let known = KnownString::one(value.to_string());
            assert_eq!(
                known_string_truthiness(&known),
                Some(is_php_truthy_string(value)),
                "single known string truthiness for {value:?}",
            );
        }

        let falsey_values =
            KnownString::from_values(["".to_string(), "0".to_string()]).expect("known strings");
        assert_eq!(known_string_truthiness(&falsey_values), Some(false));

        let truthy_values = KnownString::from_values([
            "00".to_string(),
            "0.0".to_string(),
            " ".to_string(),
            "false".to_string(),
        ])
        .expect("known strings");
        assert_eq!(known_string_truthiness(&truthy_values), Some(true));

        let mixed_values =
            KnownString::from_values(["0".to_string(), "00".to_string()]).expect("known strings");
        assert_eq!(known_string_truthiness(&mixed_values), None);
    }

    #[test]
    fn generated_c_string_comparison_safety_uses_shared_pair_classifier() {
        let binary_program = crate::parse(
            r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$left = $flag ? "10" : "8foo";
$right = $flag ? "zeta" : "+foo";

echo $left < $right, "\n";
echo ".name" != "-word", "\n";
echo " 10" < "zeta";
"#,
        )
        .expect("parse binary string comparison source");
        let c_source = emit_c_source_for_assembly(&binary_program)
            .expect("known binary string comparison pairs should lower through C fallback");

        assert!(
            c_source.contains("strcmp("),
            "numeric-vs-nonnumeric, leading-numeric, and sign/dot-prefixed nonnumeric pairs should lower through generated-C binary string comparison:\n{c_source}"
        );

        for source in [
            "<?php\necho \"10\" < \"2\";\n",
            "<?php\necho \" 10\" < \"2\";\n",
            "<?php\necho \"-2\" < \".5\";\n",
            "<?php\necho \".5\" < \"5.\";\n",
        ] {
            let program = crate::parse(source).expect("parse numeric string comparison source");
            let error = emit_c_source_for_assembly(&program).unwrap_err();

            assert_eq!(error.phase, Phase::Codegen);
            assert_eq!(error.message, assembly_comparison_rejection());
        }
    }

    fn test_param(by_reference: bool, is_variadic: bool) -> FunctionParam {
        FunctionParam {
            name: "value".to_string(),
            type_decl: None,
            by_reference,
            is_variadic,
            default: None,
            span: test_span(),
        }
    }

    fn test_function(params: Vec<FunctionParam>, returns_by_reference: bool) -> FunctionDecl {
        FunctionDecl {
            name: "sample".to_string(),
            params,
            return_type: None,
            returns_by_reference,
            body: Vec::new(),
            is_nested: false,
            end_line: 1,
            doc_comment: None,
            span: test_span(),
        }
    }

    fn test_call_reference_source(expr: Expr) -> ReferenceSource {
        ReferenceSource::MethodCall {
            expr,
            span: test_span(),
        }
    }

    fn test_variable_expr(name: &str) -> Expr {
        Expr::Variable(name.to_string(), test_span())
    }

    fn test_closure_expr(params: Vec<FunctionParam>, returns_by_reference: bool) -> Expr {
        Expr::Closure {
            params,
            captures: Vec::new(),
            return_type: None,
            returns_by_reference,
            body: Vec::new(),
            is_static: false,
            is_arrow: false,
            span: test_span(),
        }
    }

    #[test]
    fn native_function_frame_blocker_classifies_parameter_and_return_families() {
        assert!(matches!(
            native_function_frame_blocker(&test_function(Vec::new(), false)),
            NativeCallBlocker::FunctionFrameHandoff
        ));
        assert!(matches!(
            native_function_frame_blocker(&test_function(vec![test_param(true, false)], false)),
            NativeCallBlocker::ByReferenceArgumentBinding
        ));
        assert!(matches!(
            native_function_frame_blocker(&test_function(vec![test_param(false, true)], false)),
            NativeCallBlocker::VariadicArgumentBinding
        ));
        assert!(matches!(
            native_function_frame_blocker(&test_function(Vec::new(), true)),
            NativeCallBlocker::ReturnValueOwnership
        ));
        assert!(matches!(
            native_closure_frame_blocker(&[], false),
            NativeCallBlocker::ClosureFrameHandoff
        ));
        assert!(matches!(
            native_closure_frame_blocker(&[test_param(true, false)], false),
            NativeCallBlocker::ByReferenceArgumentBinding
        ));
        assert!(matches!(
            native_closure_frame_blocker(&[test_param(false, true)], false),
            NativeCallBlocker::VariadicArgumentBinding
        ));
        assert!(matches!(
            native_closure_frame_blocker(&[], true),
            NativeCallBlocker::ReturnValueOwnership
        ));
    }

    #[test]
    fn native_function_declaration_fallback_diagnostic_reuses_frame_contract_for_frame_families() {
        let span = test_span();

        for (function, operation) in [
            (
                test_function(Vec::new(), false),
                NativeCallOperation::function_frame(span, NativeCallBlocker::FunctionFrameHandoff),
            ),
            (
                test_function(vec![test_param(true, false)], false),
                NativeCallOperation::function_frame(
                    span,
                    NativeCallBlocker::ByReferenceArgumentBinding,
                ),
            ),
            (
                test_function(vec![test_param(false, true)], false),
                NativeCallOperation::function_frame(
                    span,
                    NativeCallBlocker::VariadicArgumentBinding,
                ),
            ),
            (
                test_function(Vec::new(), true),
                NativeCallOperation::function_frame(span, NativeCallBlocker::ReturnValueOwnership),
            ),
        ] {
            for (backend, static_local_rejection) in [
                (NativeCallBackend::Llvm, LLVM_STATIC_LOCAL_REJECTION),
                (NativeCallBackend::Assembly, ASSEMBLY_STATIC_LOCAL_REJECTION),
            ] {
                let diagnostic = native_function_declaration_fallback_diagnostic(
                    backend,
                    &function,
                    static_local_rejection,
                );

                assert_eq!(
                    diagnostic.message,
                    native_call_operation_diagnostic(backend, operation).message
                );
                assert_eq!(diagnostic.message, backend.function_declaration_rejection());
            }
        }

        let static_span = Span::new(7, 5);
        let mut static_local_function = test_function(Vec::new(), false);
        static_local_function.body = vec![Stmt::StaticLocal {
            declarations: vec![StaticLocalDeclarator {
                name: "seen".to_string(),
                default: None,
                span: static_span,
            }],
            span: static_span,
        }];

        let diagnostic = native_function_declaration_fallback_diagnostic(
            NativeCallBackend::Assembly,
            &static_local_function,
            ASSEMBLY_STATIC_LOCAL_REJECTION,
        );

        assert_eq!(diagnostic.phase, Phase::Codegen);
        assert_eq!(diagnostic.line, static_span.line);
        assert_eq!(diagnostic.column, static_span.column);
        assert_eq!(diagnostic.message, ASSEMBLY_STATIC_LOCAL_REJECTION);
    }

    #[test]
    fn native_reference_source_call_operation_preserves_call_family_for_reference_results() {
        let span = test_span();

        for (source, callee) in [
            (
                test_call_reference_source(Expr::Call {
                    name: "borrow".to_string(),
                    args: vec![Expr::Int(1, span)],
                    span,
                }),
                NativeCallCallee::DirectNamed,
            ),
            (
                test_call_reference_source(Expr::DynamicCall {
                    callee: Box::new(test_variable_expr("callback")),
                    args: vec![Expr::String("value".to_string(), span)],
                    span,
                }),
                NativeCallCallee::DynamicExpression,
            ),
            (
                test_call_reference_source(Expr::DynamicMethodCall {
                    target: Box::new(test_variable_expr("box")),
                    method: Box::new(test_variable_expr("method")),
                    args: vec![Expr::Bool(true, span)],
                    span,
                }),
                NativeCallCallee::MethodDispatch,
            ),
            (
                test_call_reference_source(Expr::StaticMethodCall {
                    class_name: "Box".to_string(),
                    method: "borrow".to_string(),
                    args: vec![Expr::Int(2, span)],
                    span,
                }),
                NativeCallCallee::MethodDispatch,
            ),
            (
                ReferenceSource::ExpressionArrayIndex {
                    target: Expr::Call {
                        name: "borrow".to_string(),
                        args: vec![Expr::Int(3, span)],
                        span,
                    },
                    indices: vec![Expr::Int(0, span)],
                    span,
                },
                NativeCallCallee::DirectNamed,
            ),
            (
                ReferenceSource::ExpressionArrayAppend {
                    target: Expr::DynamicCall {
                        callee: Box::new(test_variable_expr("callback")),
                        args: vec![Expr::String("value".to_string(), span)],
                        span,
                    },
                    indices: vec![Expr::Int(0, span)],
                    span,
                },
                NativeCallCallee::DynamicExpression,
            ),
            (
                ReferenceSource::Property {
                    expr: Expr::Property {
                        target: Box::new(Expr::MethodCall {
                            target: Box::new(test_variable_expr("box")),
                            method: "borrow".to_string(),
                            args: vec![Expr::Bool(false, span)],
                            span,
                        }),
                        property: "value".to_string(),
                        span,
                    },
                    span,
                },
                NativeCallCallee::MethodDispatch,
            ),
            (
                ReferenceSource::NonDirectObjectPropertyNestedArrayIndex {
                    holder: Expr::DynamicCall {
                        callee: Box::new(test_variable_expr("callback")),
                        args: vec![Expr::String("holder".to_string(), span)],
                        span,
                    },
                    property: "items".to_string(),
                    indices: vec![Expr::Int(1, span)],
                    span,
                },
                NativeCallCallee::DynamicExpression,
            ),
            (
                ReferenceSource::ExpressionArrayIndex {
                    target: Expr::New {
                        class_name: crate::ast::NewClassName::Named("Box".to_string()),
                        args: vec![Expr::Int(3, span)],
                        span,
                    },
                    indices: vec![Expr::Int(0, span)],
                    span,
                },
                NativeCallCallee::ConstructorDispatch,
            ),
        ] {
            assert_eq!(
                native_reference_source_call_operation(&source),
                Some(NativeCallOperation::reference_result(span, callee))
            );
        }

        assert_eq!(
            native_reference_source_call_operation(&ReferenceSource::Variable {
                name: "value".to_string(),
                span,
            }),
            None
        );
    }

    #[test]
    fn native_reference_source_call_operation_classifies_lvalue_operand_cleanup() {
        let span = test_span();

        for (source, operation) in [
            (
                ReferenceSource::ArrayIndex {
                    name: "items".to_string(),
                    index: Expr::Call {
                        name: "key_name".to_string(),
                        args: vec![Expr::String("id".to_string(), span)],
                        span,
                    },
                    span,
                },
                NativeCallOperation::direct_named_value(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                ReferenceSource::DynamicObjectPropertyArrayIndex {
                    object: "box".to_string(),
                    property: Expr::DynamicCall {
                        callee: Box::new(test_variable_expr("property_name")),
                        args: vec![Expr::Bool(true, span)],
                        span,
                    },
                    index: Expr::Int(0, span),
                    span,
                },
                NativeCallOperation::dynamic_value_with_blocker(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                ReferenceSource::NonDirectDynamicObjectPropertyNestedArrayIndex {
                    holder: test_variable_expr("box"),
                    property: Expr::MethodCall {
                        target: Box::new(test_variable_expr("names")),
                        method: "current".to_string(),
                        args: vec![Expr::Int(2, span)],
                        span,
                    },
                    indices: vec![Expr::Int(0, span)],
                    span,
                },
                NativeCallOperation::method_value_with_blocker(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                ReferenceSource::NestedArrayIndex {
                    name: "items".to_string(),
                    indices: vec![Expr::New {
                        class_name: crate::ast::NewClassName::Named("Key".to_string()),
                        args: vec![Expr::Int(3, span)],
                        span,
                    }],
                    span,
                },
                NativeCallOperation::constructor_value(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
        ] {
            assert_eq!(
                native_reference_source_call_operation(&source),
                Some(operation)
            );
        }

        assert_eq!(
            native_reference_source_call_operation(&ReferenceSource::ArrayIndex {
                name: "items".to_string(),
                index: Expr::String("plain".to_string(), span),
                span,
            }),
            None
        );
    }

    #[test]
    fn native_reference_assignment_call_operation_classifies_target_lvalue_cleanup() {
        let span = test_span();
        let source = ReferenceSource::Variable {
            name: "value".to_string(),
            span,
        };

        for (target, operation) in [
            (
                AssignTarget::ArrayIndex {
                    name: "items".to_string(),
                    index: Some(Expr::Call {
                        name: "key_name".to_string(),
                        args: vec![Expr::String("id".to_string(), span)],
                        span,
                    }),
                    span,
                },
                NativeCallOperation::direct_named_value(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                AssignTarget::DynamicObjectPropertyArrayIndex {
                    object: "box".to_string(),
                    property: Expr::DynamicCall {
                        callee: Box::new(test_variable_expr("property_name")),
                        args: vec![Expr::Bool(true, span)],
                        span,
                    },
                    indices: vec![Expr::Int(0, span)],
                    span,
                },
                NativeCallOperation::dynamic_value_with_blocker(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                AssignTarget::DynamicProperty {
                    object: "box".to_string(),
                    property: Expr::MethodCall {
                        target: Box::new(test_variable_expr("names")),
                        method: "current".to_string(),
                        args: vec![Expr::Int(2, span)],
                        span,
                    },
                    span,
                },
                NativeCallOperation::method_value_with_blocker(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                AssignTarget::NestedArrayIndex {
                    name: "items".to_string(),
                    indices: vec![Expr::New {
                        class_name: crate::ast::NewClassName::Named("Key".to_string()),
                        args: vec![Expr::Int(3, span)],
                        span,
                    }],
                    span,
                },
                NativeCallOperation::constructor_value(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
        ] {
            let stmt = Stmt::ReferenceAssign {
                target: target.clone(),
                source: source.clone(),
                span,
            };

            assert_eq!(
                native_reference_assignment_call_operation(&target, &source),
                Some(operation)
            );
            assert_eq!(
                native_statement_operand_call_operation(&stmt),
                Some(operation)
            );
        }

        let source = ReferenceSource::ArrayIndex {
            name: "items".to_string(),
            index: Expr::Call {
                name: "source_key".to_string(),
                args: vec![Expr::Int(4, span)],
                span,
            },
            span,
        };
        let target = AssignTarget::Variable {
            name: "alias".to_string(),
            span,
        };

        assert_eq!(
            native_reference_assignment_call_operation(&target, &source),
            Some(NativeCallOperation::direct_named_value(
                span,
                NativeCallBlocker::LvalueOperandEvaluationCleanup,
            ))
        );
    }

    #[test]
    fn native_dereferenced_call_result_operation_preserves_call_family_for_value_results() {
        let span = test_span();

        for (expr, callee) in [
            (
                Expr::Index {
                    target: Box::new(Expr::Call {
                        name: "result".to_string(),
                        args: vec![Expr::Int(1, span)],
                        span,
                    }),
                    index: Box::new(Expr::Int(0, span)),
                    span,
                },
                NativeCallCallee::DirectNamed,
            ),
            (
                Expr::AppendIndex {
                    target: Box::new(Expr::DynamicCall {
                        callee: Box::new(test_variable_expr("callback")),
                        args: vec![Expr::String("value".to_string(), span)],
                        span,
                    }),
                    span,
                },
                NativeCallCallee::DynamicExpression,
            ),
            (
                Expr::Property {
                    target: Box::new(Expr::MethodCall {
                        target: Box::new(test_variable_expr("box")),
                        method: "result".to_string(),
                        args: vec![Expr::Bool(true, span)],
                        span,
                    }),
                    property: "value".to_string(),
                    span,
                },
                NativeCallCallee::MethodDispatch,
            ),
            (
                Expr::DynamicProperty {
                    target: Box::new(Expr::Index {
                        target: Box::new(Expr::Call {
                            name: "result".to_string(),
                            args: vec![Expr::Int(2, span)],
                            span,
                        }),
                        index: Box::new(Expr::Int(0, span)),
                        span,
                    }),
                    property: Box::new(test_variable_expr("property")),
                    span,
                },
                NativeCallCallee::DirectNamed,
            ),
            (
                Expr::Property {
                    target: Box::new(Expr::New {
                        class_name: crate::ast::NewClassName::Named("Box".to_string()),
                        args: vec![Expr::Int(3, span)],
                        span,
                    }),
                    property: "value".to_string(),
                    span,
                },
                NativeCallCallee::ConstructorDispatch,
            ),
        ] {
            assert_eq!(
                native_dereferenced_call_result_operation(&expr),
                Some(NativeCallOperation::dereferenced_value_result(span, callee))
            );
        }

        assert_eq!(
            native_dereferenced_call_result_operation(&Expr::Index {
                target: Box::new(test_variable_expr("items")),
                index: Box::new(Expr::Int(0, span)),
                span,
            }),
            None
        );
    }

    #[test]
    fn native_assignment_target_call_result_operation_preserves_call_family_for_lvalue_roots() {
        let span = test_span();

        for (target, callee) in [
            (
                AssignTarget::NonDirectProperty {
                    holder: Expr::Call {
                        name: "result".to_string(),
                        args: vec![Expr::Int(1, span)],
                        span,
                    },
                    property: "value".to_string(),
                    span,
                },
                NativeCallCallee::DirectNamed,
            ),
            (
                AssignTarget::NonDirectDynamicProperty {
                    holder: Expr::DynamicCall {
                        callee: Box::new(test_variable_expr("callback")),
                        args: vec![Expr::String("value".to_string(), span)],
                        span,
                    },
                    property: test_variable_expr("property"),
                    span,
                },
                NativeCallCallee::DynamicExpression,
            ),
            (
                AssignTarget::NonDirectObjectPropertyArrayIndex {
                    holder: Expr::MethodCall {
                        target: Box::new(test_variable_expr("box")),
                        method: "result".to_string(),
                        args: vec![Expr::Bool(true, span)],
                        span,
                    },
                    property: "items".to_string(),
                    indices: vec![Expr::Int(0, span)],
                    span,
                },
                NativeCallCallee::MethodDispatch,
            ),
            (
                AssignTarget::ObjectStaticProperty {
                    target: Expr::New {
                        class_name: crate::ast::NewClassName::Named("Box".to_string()),
                        args: vec![Expr::Int(3, span)],
                        span,
                    },
                    property: "value".to_string(),
                    span,
                },
                NativeCallCallee::ConstructorDispatch,
            ),
        ] {
            assert_eq!(
                native_assignment_target_call_result_operation(&target),
                Some(NativeCallOperation::dereferenced_value_result(span, callee))
            );
        }

        assert_eq!(
            native_assignment_target_call_result_operation(&AssignTarget::Property {
                object: "box".to_string(),
                property: "value".to_string(),
                span,
            }),
            None
        );
    }

    #[test]
    fn native_assignment_target_call_operation_classifies_lvalue_operand_cleanup() {
        let span = test_span();

        for (target, operation) in [
            (
                AssignTarget::ArrayIndex {
                    name: "items".to_string(),
                    index: Some(Expr::Call {
                        name: "key_name".to_string(),
                        args: vec![Expr::Int(1, span)],
                        span,
                    }),
                    span,
                },
                NativeCallOperation::direct_named_value(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                AssignTarget::NestedArrayIndex {
                    name: "items".to_string(),
                    indices: vec![Expr::DynamicCall {
                        callee: Box::new(test_variable_expr("key_factory")),
                        args: vec![Expr::String("slot".to_string(), span)],
                        span,
                    }],
                    span,
                },
                NativeCallOperation::dynamic_value_with_blocker(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                AssignTarget::NonDirectDynamicProperty {
                    holder: test_variable_expr("box"),
                    property: Expr::MethodCall {
                        target: Box::new(test_variable_expr("namer")),
                        method: "property".to_string(),
                        args: vec![Expr::Bool(true, span)],
                        span,
                    },
                    span,
                },
                NativeCallOperation::method_value_with_blocker(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                AssignTarget::DynamicObjectPropertyArrayIndex {
                    object: "box".to_string(),
                    property: Expr::New {
                        class_name: crate::ast::NewClassName::Named("Key".to_string()),
                        args: vec![Expr::Int(2, span)],
                        span,
                    },
                    indices: vec![Expr::Int(0, span)],
                    span,
                },
                NativeCallOperation::constructor_value(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
        ] {
            assert_eq!(
                native_assignment_target_call_operation(&target),
                Some(operation)
            );
        }

        assert_eq!(
            native_assignment_target_call_operation(&AssignTarget::ArrayIndex {
                name: "items".to_string(),
                index: Some(Expr::String("plain".to_string(), span)),
                span,
            }),
            None
        );
        assert_eq!(
            native_assignment_target_call_operation(&AssignTarget::ArrayIndex {
                name: "items".to_string(),
                index: Some(Expr::Call {
                    name: "strval".to_string(),
                    args: vec![Expr::String("plain".to_string(), span)],
                    span,
                }),
                span,
            }),
            None
        );
        assert_eq!(
            native_assignment_target_call_operation(&AssignTarget::ArrayIndex {
                name: "items".to_string(),
                index: Some(Expr::Call {
                    name: "strval".to_string(),
                    args: vec![Expr::Call {
                        name: "key_name".to_string(),
                        args: Vec::new(),
                        span,
                    }],
                    span,
                }),
                span,
            }),
            Some(NativeCallOperation::direct_named_value(
                span,
                NativeCallBlocker::LvalueOperandEvaluationCleanup,
            ))
        );
    }

    #[test]
    fn native_unset_target_call_operation_classifies_lvalue_operand_cleanup() {
        let span = test_span();

        for (target, operation) in [
            (
                UnsetTarget::ArrayIndex {
                    name: "items".to_string(),
                    index: Expr::Call {
                        name: "key_name".to_string(),
                        args: vec![Expr::Int(1, span)],
                        span,
                    },
                    span,
                },
                NativeCallOperation::direct_named_value(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                UnsetTarget::NestedArrayIndex {
                    name: "items".to_string(),
                    indices: vec![Expr::DynamicCall {
                        callee: Box::new(test_variable_expr("key_factory")),
                        args: vec![Expr::String("slot".to_string(), span)],
                        span,
                    }],
                    span,
                },
                NativeCallOperation::dynamic_value_with_blocker(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                UnsetTarget::NonDirectDynamicObjectProperty {
                    holder: test_variable_expr("box"),
                    property: Expr::MethodCall {
                        target: Box::new(test_variable_expr("namer")),
                        method: "property".to_string(),
                        args: vec![Expr::Bool(true, span)],
                        span,
                    },
                    span,
                },
                NativeCallOperation::method_value_with_blocker(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                UnsetTarget::DynamicObjectPropertyArrayIndex {
                    object: "box".to_string(),
                    property: Expr::New {
                        class_name: crate::ast::NewClassName::Named("Key".to_string()),
                        args: vec![Expr::Int(2, span)],
                        span,
                    },
                    indices: vec![Expr::Int(0, span)],
                    span,
                },
                NativeCallOperation::constructor_value(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
        ] {
            assert_eq!(native_unset_target_call_operation(&target), Some(operation));
        }

        assert_eq!(
            native_unset_target_call_operation(&UnsetTarget::ArrayIndex {
                name: "items".to_string(),
                index: Expr::String("plain".to_string(), span),
                span,
            }),
            None
        );
    }

    #[test]
    fn native_statement_operand_call_operation_classifies_unset_lvalue_cleanup() {
        let span = test_span();

        for (stmt, operation) in [
            (
                Stmt::UnsetArrayIndex {
                    name: "items".to_string(),
                    index: Expr::Call {
                        name: "key_name".to_string(),
                        args: vec![Expr::Int(1, span)],
                        span,
                    },
                    span,
                },
                NativeCallOperation::direct_named_value(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                Stmt::UnsetNestedArrayIndex {
                    name: "items".to_string(),
                    indices: vec![Expr::DynamicCall {
                        callee: Box::new(test_variable_expr("key_factory")),
                        args: vec![Expr::String("slot".to_string(), span)],
                        span,
                    }],
                    span,
                },
                NativeCallOperation::dynamic_value_with_blocker(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                Stmt::UnsetDynamicObjectProperty {
                    object: "box".to_string(),
                    property: Expr::MethodCall {
                        target: Box::new(test_variable_expr("namer")),
                        method: "property".to_string(),
                        args: vec![Expr::Bool(true, span)],
                        span,
                    },
                    span,
                },
                NativeCallOperation::method_value_with_blocker(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
            (
                Stmt::UnsetMany {
                    targets: vec![UnsetTarget::DynamicObjectPropertyArrayIndex {
                        object: "box".to_string(),
                        property: Expr::New {
                            class_name: crate::ast::NewClassName::Named("Key".to_string()),
                            args: vec![Expr::Int(2, span)],
                            span,
                        },
                        indices: vec![Expr::Int(0, span)],
                        span,
                    }],
                    span,
                },
                NativeCallOperation::constructor_value(
                    span,
                    NativeCallBlocker::LvalueOperandEvaluationCleanup,
                ),
            ),
        ] {
            assert_eq!(
                native_statement_operand_call_operation(&stmt),
                Some(operation)
            );
        }

        assert_eq!(
            native_statement_operand_call_operation(&Stmt::UnsetArrayIndex {
                name: "items".to_string(),
                index: Expr::String("plain".to_string(), span),
                span,
            }),
            None
        );
    }

    #[test]
    fn native_value_operand_call_result_operation_classifies_value_operand_cleanup() {
        let span = test_span();

        for (expr, operation) in [
            (
                Expr::Array {
                    items: vec![ArrayItem {
                        key: None,
                        value: Expr::Call {
                            name: "produce".to_string(),
                            args: vec![Expr::Int(1, span)],
                            span,
                        },
                        by_reference: false,
                    }],
                    span,
                },
                NativeCallOperation::direct_named_value(
                    span,
                    NativeCallBlocker::ValueOperandEvaluationCleanup,
                ),
            ),
            (
                Expr::Index {
                    target: Box::new(test_variable_expr("items")),
                    index: Box::new(Expr::DynamicCall {
                        callee: Box::new(test_variable_expr("key_factory")),
                        args: vec![Expr::String("key".to_string(), span)],
                        span,
                    }),
                    span,
                },
                NativeCallOperation::dynamic_value_with_blocker(
                    span,
                    NativeCallBlocker::ValueOperandEvaluationCleanup,
                ),
            ),
            (
                Expr::DynamicProperty {
                    target: Box::new(test_variable_expr("box")),
                    property: Box::new(Expr::MethodCall {
                        target: Box::new(test_variable_expr("name_source")),
                        method: "name".to_string(),
                        args: vec![Expr::Bool(true, span)],
                        span,
                    }),
                    span,
                },
                NativeCallOperation::method_value_with_blocker(
                    span,
                    NativeCallBlocker::ValueOperandEvaluationCleanup,
                ),
            ),
            (
                Expr::Cast {
                    kind: crate::ast::CastKind::String,
                    expr: Box::new(Expr::New {
                        class_name: crate::ast::NewClassName::Named("Value".to_string()),
                        args: vec![Expr::Int(2, span)],
                        span,
                    }),
                    span,
                },
                NativeCallOperation::constructor_value(
                    span,
                    NativeCallBlocker::ValueOperandEvaluationCleanup,
                ),
            ),
        ] {
            assert_eq!(
                native_value_operand_call_result_operation(&expr),
                Some(operation)
            );
        }

        assert_eq!(
            native_value_operand_call_result_operation(&Expr::String("plain".to_string(), span)),
            None
        );
    }

    #[test]
    fn native_value_result_expr_call_operation_preserves_owned_result_families() {
        let span = test_span();
        let nested_call_span = Span::new(2, 3);

        let type_name_result = Expr::Call {
            name: "get_debug_type".to_string(),
            args: vec![Expr::Cast {
                kind: crate::ast::CastKind::String,
                expr: Box::new(Expr::Int(123, span)),
                span,
            }],
            span,
        };
        assert_eq!(
            native_value_result_expr_call_operation(
                &type_name_result,
                NativeCallBlocker::StatementOperandEvaluationCleanup,
            ),
            None
        );

        let compare_cast_result = Expr::Cast {
            kind: crate::ast::CastKind::String,
            expr: Box::new(Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Int(2, span)),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::Int(3, span)),
                    span,
                }),
                op: BinaryOp::Gt,
                right: Box::new(Expr::Int(4, span)),
                span,
            }),
            span,
        };
        assert_eq!(
            native_value_result_expr_call_operation(
                &compare_cast_result,
                NativeCallBlocker::StatementOperandEvaluationCleanup,
            ),
            None
        );

        let type_name_with_nested_call = Expr::Call {
            name: "gettype".to_string(),
            args: vec![Expr::Call {
                name: "produce".to_string(),
                args: Vec::new(),
                span: nested_call_span,
            }],
            span,
        };
        assert_eq!(
            native_value_result_expr_call_operation(
                &type_name_with_nested_call,
                NativeCallBlocker::StatementOperandEvaluationCleanup,
            ),
            Some(NativeCallOperation::direct_named_value(
                nested_call_span,
                NativeCallBlocker::StatementOperandEvaluationCleanup,
            ))
        );

        let scalar_cast_builtin_with_nested_call = Expr::Call {
            name: "strval".to_string(),
            args: vec![Expr::Call {
                name: "produce".to_string(),
                args: Vec::new(),
                span: nested_call_span,
            }],
            span,
        };
        assert_eq!(
            native_value_result_expr_call_operation(
                &scalar_cast_builtin_with_nested_call,
                NativeCallBlocker::StatementOperandEvaluationCleanup,
            ),
            Some(NativeCallOperation::direct_named_value(
                nested_call_span,
                NativeCallBlocker::StatementOperandEvaluationCleanup,
            ))
        );

        let bad_type_name_arity = Expr::Call {
            name: "gettype".to_string(),
            args: vec![Expr::Int(1, span), Expr::Int(2, span)],
            span,
        };
        assert_eq!(
            native_value_result_expr_call_operation(
                &bad_type_name_arity,
                NativeCallBlocker::StatementOperandEvaluationCleanup,
            ),
            Some(NativeCallOperation::direct_named_value(
                span,
                NativeCallBlocker::StatementOperandEvaluationCleanup,
            ))
        );
    }

    #[test]
    fn native_value_call_operation_classifies_named_dynamic_method_and_constructor_families() {
        let span = test_span();

        for (expr, operation) in [
            (
                Expr::Call {
                    name: "missing".to_string(),
                    args: vec![Expr::Int(1, span)],
                    span,
                },
                NativeCallOperation::direct_named_value(
                    span,
                    NativeCallBlocker::UnknownCalleeDiagnostics,
                ),
            ),
            (
                Expr::DynamicCall {
                    callee: Box::new(test_variable_expr("callback")),
                    args: vec![Expr::String("value".to_string(), span)],
                    span,
                },
                NativeCallOperation::dynamic_value(span),
            ),
            (
                Expr::MethodCall {
                    target: Box::new(test_variable_expr("box")),
                    method: "work".to_string(),
                    args: vec![Expr::Bool(true, span)],
                    span,
                },
                NativeCallOperation::method_value(span),
            ),
            (
                Expr::LateStaticMethodCall {
                    method: "work".to_string(),
                    args: vec![Expr::Int(2, span)],
                    span,
                },
                NativeCallOperation::method_value(span),
            ),
            (
                Expr::New {
                    class_name: crate::ast::NewClassName::Named("Box".to_string()),
                    args: vec![Expr::Bool(false, span)],
                    span,
                },
                NativeCallOperation::constructor_value(
                    span,
                    NativeCallBlocker::ConstructorDispatch,
                ),
            ),
        ] {
            assert_eq!(native_value_call_operation_for_expr(&expr), Some(operation));
        }

        assert_eq!(
            native_value_call_operation_for_expr(&Expr::Variable("value".to_string(), span)),
            None
        );
    }

    #[test]
    fn native_value_call_operation_classifies_argument_cleanup_across_call_families() {
        let span = test_span();

        for (expr, callee) in [
            (
                Expr::Call {
                    name: "consume".to_string(),
                    args: vec![Expr::Binary {
                        left: Box::new(Expr::Int(1, span)),
                        op: BinaryOp::Add,
                        right: Box::new(Expr::Call {
                            name: "produce".to_string(),
                            args: vec![Expr::String("value".to_string(), span)],
                            span,
                        }),
                        span,
                    }],
                    span,
                },
                NativeCallCallee::DirectNamed,
            ),
            (
                Expr::DynamicCall {
                    callee: Box::new(test_variable_expr("consumer")),
                    args: vec![Expr::Array {
                        items: vec![ArrayItem {
                            key: Some(Expr::Int(0, span)),
                            value: Expr::DynamicCall {
                                callee: Box::new(test_variable_expr("producer")),
                                args: vec![Expr::Bool(true, span)],
                                span,
                            },
                            by_reference: false,
                        }],
                        span,
                    }],
                    span,
                },
                NativeCallCallee::DynamicExpression,
            ),
            (
                Expr::StaticMethodCall {
                    class_name: "Consumer".to_string(),
                    method: "take".to_string(),
                    args: vec![Expr::Ternary {
                        condition: Box::new(Expr::Bool(true, span)),
                        if_true: Box::new(Expr::MethodCall {
                            target: Box::new(test_variable_expr("producer")),
                            method: "value".to_string(),
                            args: vec![Expr::Int(2, span)],
                            span,
                        }),
                        if_false: Box::new(Expr::String("fallback".to_string(), span)),
                        span,
                    }],
                    span,
                },
                NativeCallCallee::MethodDispatch,
            ),
            (
                Expr::New {
                    class_name: crate::ast::NewClassName::Named("Consumer".to_string()),
                    args: vec![Expr::ShortTernary {
                        condition: Box::new(Expr::Call {
                            name: "produce".to_string(),
                            args: vec![Expr::Int(4, span)],
                            span,
                        }),
                        if_false: Box::new(Expr::String("fallback".to_string(), span)),
                        span,
                    }],
                    span,
                },
                NativeCallCallee::ConstructorDispatch,
            ),
        ] {
            assert_eq!(
                native_value_call_operation_for_expr(&expr),
                Some(NativeCallOperation::value_result(
                    span,
                    callee,
                    NativeCallBlocker::ArgumentEvaluationCleanup,
                ))
            );
        }

        assert_eq!(
            native_call_argument_list_blocker(&[
                Expr::Int(1, span),
                Expr::String("plain".to_string(), span),
            ]),
            None
        );
        assert_eq!(
            native_call_argument_list_blocker(&[test_closure_expr(Vec::new(), false)]),
            Some(NativeCallBlocker::ArgumentEvaluationCleanup)
        );
    }

    #[test]
    fn native_direct_call_argument_result_operation_reuses_argument_cleanup_boundary() {
        let span = test_span();

        for args in [
            vec![Expr::Call {
                name: "produce".to_string(),
                args: vec![Expr::String("value".to_string(), span)],
                span,
            }],
            vec![Expr::Array {
                items: vec![ArrayItem {
                    key: Some(Expr::Int(0, span)),
                    value: Expr::DynamicCall {
                        callee: Box::new(test_variable_expr("producer")),
                        args: vec![Expr::Bool(true, span)],
                        span,
                    },
                    by_reference: false,
                }],
                span,
            }],
            vec![test_closure_expr(vec![test_param(false, true)], false)],
            vec![Expr::New {
                class_name: crate::ast::NewClassName::DynamicVariable("class".to_string()),
                args: vec![Expr::Int(1, span)],
                span,
            }],
        ] {
            assert_eq!(
                native_direct_call_argument_result_operation(&args, span),
                Some(NativeCallOperation::direct_named_value(
                    span,
                    NativeCallBlocker::ArgumentEvaluationCleanup,
                ))
            );
        }

        assert_eq!(
            native_direct_call_argument_result_operation(
                &[Expr::Int(1, span), Expr::String("plain".to_string(), span)],
                span,
            ),
            None
        );
    }

    #[test]
    fn native_unemitted_statement_operand_call_operation_preserves_call_contracts() {
        let span = test_span();

        for (expr, operation) in [
            (
                Expr::DynamicCall {
                    callee: Box::new(test_variable_expr("callback")),
                    args: Vec::new(),
                    span,
                },
                NativeCallOperation::dynamic_value(span),
            ),
            (
                Expr::MethodCall {
                    target: Box::new(test_variable_expr("receiver")),
                    method: "work".to_string(),
                    args: Vec::new(),
                    span,
                },
                NativeCallOperation::method_value(span),
            ),
            (
                Expr::New {
                    class_name: crate::ast::NewClassName::Named("Consumer".to_string()),
                    args: vec![Expr::DynamicCall {
                        callee: Box::new(test_variable_expr("produce")),
                        args: Vec::new(),
                        span,
                    }],
                    span,
                },
                NativeCallOperation::constructor_value(
                    span,
                    NativeCallBlocker::ArgumentEvaluationCleanup,
                ),
            ),
            (
                Expr::Binary {
                    left: Box::new(Expr::Int(1, span)),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::Call {
                        name: "produce".to_string(),
                        args: Vec::new(),
                        span,
                    }),
                    span,
                },
                NativeCallOperation::direct_named_value(
                    span,
                    NativeCallBlocker::StatementOperandEvaluationCleanup,
                ),
            ),
        ] {
            assert_eq!(
                native_unemitted_statement_operand_call_operation(&expr),
                Some(operation)
            );
        }

        let unsupported_first = Expr::Array {
            items: Vec::new(),
            span,
        };
        let later_call_operand = Expr::Binary {
            left: Box::new(Expr::String("left".to_string(), span)),
            op: BinaryOp::Concat,
            right: Box::new(Expr::MethodCall {
                target: Box::new(test_variable_expr("box")),
                method: "label".to_string(),
                args: Vec::new(),
                span,
            }),
            span,
        };

        assert_eq!(
            native_unemitted_statement_operand_list_call_operation(&[
                unsupported_first,
                later_call_operand,
            ]),
            Some(NativeCallOperation::method_value_with_blocker(
                span,
                NativeCallBlocker::StatementOperandEvaluationCleanup,
            ))
        );
    }

    #[test]
    fn native_call_diagnostics_centralizes_backend_recovery_across_call_families() {
        let span = test_span();
        let function_frame = test_function(Vec::new(), false);
        let by_ref_function_frame = test_function(vec![test_param(true, false)], false);
        let closure_expr = test_closure_expr(Vec::new(), false);

        for (diagnostics, expr, expected) in [
            (
                NativeCallDiagnostics::new(NativeCallBackend::Llvm),
                Expr::Call {
                    name: "produce".to_string(),
                    args: Vec::new(),
                    span,
                },
                LLVM_FUNCTION_CALL_REJECTION,
            ),
            (
                NativeCallDiagnostics::new(NativeCallBackend::Assembly),
                Expr::DynamicCall {
                    callee: Box::new(test_variable_expr("produce")),
                    args: vec![Expr::String("value".to_string(), span)],
                    span,
                },
                ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
            ),
            (
                NativeCallDiagnostics::new(NativeCallBackend::Llvm),
                Expr::StaticMethodCall {
                    class_name: "Box".to_string(),
                    method: "produce".to_string(),
                    args: vec![Expr::Bool(true, span)],
                    span,
                },
                LLVM_METHOD_CALL_REJECTION,
            ),
            (
                NativeCallDiagnostics::new(NativeCallBackend::Assembly),
                Expr::New {
                    class_name: crate::ast::NewClassName::Named("Value".to_string()),
                    args: vec![Expr::Int(1, span)],
                    span,
                },
                ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
            ),
            (
                NativeCallDiagnostics::new(NativeCallBackend::Llvm),
                closure_expr.clone(),
                LLVM_CLOSURE_REJECTION,
            ),
        ] {
            let diagnostic = diagnostics.call_root(&expr);

            assert_eq!(diagnostic.phase, Phase::Codegen);
            assert_eq!(diagnostic.line, span.line);
            assert_eq!(diagnostic.column, span.column);
            assert_eq!(diagnostic.message, expected);
        }

        for (diagnostics, subject, expected) in [
            (
                NativeCallDiagnostics::new(NativeCallBackend::Llvm),
                NativeCallDiagnosticSubject::FunctionFrame(&function_frame),
                LLVM_FUNCTION_DECLARATION_REJECTION,
            ),
            (
                NativeCallDiagnostics::new(NativeCallBackend::Assembly),
                NativeCallDiagnosticSubject::FunctionFrame(&by_ref_function_frame),
                ASSEMBLY_FUNCTION_DECLARATION_REJECTION,
            ),
            (
                NativeCallDiagnostics::new(NativeCallBackend::Llvm),
                NativeCallDiagnosticSubject::ReturnStatement(span),
                LLVM_FUNCTION_DECLARATION_REJECTION,
            ),
            (
                NativeCallDiagnostics::new(NativeCallBackend::Assembly),
                NativeCallDiagnosticSubject::Operation(NativeCallOperation::reference_result(
                    span,
                    NativeCallCallee::MethodDispatch,
                )),
                ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION,
            ),
            (
                NativeCallDiagnostics::new(NativeCallBackend::Llvm),
                NativeCallDiagnosticSubject::CallRoot(&closure_expr),
                LLVM_CLOSURE_REJECTION,
            ),
        ] {
            let diagnostic = diagnostics.subject(subject);

            assert_eq!(diagnostic.phase, Phase::Codegen);
            assert_eq!(diagnostic.line, span.line);
            assert_eq!(diagnostic.column, span.column);
            assert_eq!(diagnostic.message, expected);
        }

        assert_eq!(
            NativeCallDiagnostics::new(NativeCallBackend::Assembly)
                .direct_call(span, NativeCallBlocker::UnknownCalleeDiagnostics)
                .message,
            ASSEMBLY_FUNCTION_CALL_REJECTION
        );
    }

    #[test]
    fn native_call_blocker_message_routes_shared_contract_by_backend_and_call_family() {
        let span = test_span();

        for (backend, direct, dynamic, method, constructor, frame, closure, reference) in [
            (
                NativeCallBackend::Llvm,
                LLVM_FUNCTION_CALL_REJECTION,
                LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
                LLVM_METHOD_CALL_REJECTION,
                LLVM_OBJECT_INSTANTIATION_REJECTION,
                LLVM_FUNCTION_DECLARATION_REJECTION,
                LLVM_CLOSURE_REJECTION,
                LLVM_REFERENCE_ASSIGNMENT_REJECTION,
            ),
            (
                NativeCallBackend::Assembly,
                ASSEMBLY_FUNCTION_CALL_REJECTION,
                ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
                ASSEMBLY_METHOD_CALL_REJECTION,
                ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
                ASSEMBLY_FUNCTION_DECLARATION_REJECTION,
                ASSEMBLY_CLOSURE_REJECTION,
                ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION,
            ),
        ] {
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::direct_named_value(
                        span,
                        NativeCallBlocker::ArgumentEvaluationCleanup,
                    ),
                ),
                direct
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::direct_named_value(
                        span,
                        NativeCallBlocker::ValueOperandEvaluationCleanup,
                    ),
                ),
                direct
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::direct_named_value(
                        span,
                        NativeCallBlocker::LvalueOperandEvaluationCleanup,
                    ),
                ),
                direct
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::direct_named_value(
                        span,
                        NativeCallBlocker::UnknownCalleeDiagnostics,
                    ),
                ),
                direct
            );
            assert_eq!(
                native_call_blocker_message(backend, NativeCallOperation::dynamic_value(span)),
                dynamic
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::dynamic_value_with_blocker(
                        span,
                        NativeCallBlocker::ArgumentEvaluationCleanup,
                    ),
                ),
                dynamic
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::dynamic_value_with_blocker(
                        span,
                        NativeCallBlocker::ValueOperandEvaluationCleanup,
                    ),
                ),
                dynamic
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::dynamic_value_with_blocker(
                        span,
                        NativeCallBlocker::LvalueOperandEvaluationCleanup,
                    ),
                ),
                dynamic
            );
            assert_eq!(
                native_call_blocker_message(backend, NativeCallOperation::method_value(span)),
                method
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::method_value_with_blocker(
                        span,
                        NativeCallBlocker::ArgumentEvaluationCleanup,
                    ),
                ),
                method
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::method_value_with_blocker(
                        span,
                        NativeCallBlocker::ValueOperandEvaluationCleanup,
                    ),
                ),
                method
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::method_value_with_blocker(
                        span,
                        NativeCallBlocker::LvalueOperandEvaluationCleanup,
                    ),
                ),
                method
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::constructor_value(
                        span,
                        NativeCallBlocker::ConstructorDispatch,
                    ),
                ),
                constructor
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::constructor_value(
                        span,
                        NativeCallBlocker::ArgumentEvaluationCleanup,
                    ),
                ),
                constructor
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::constructor_value(
                        span,
                        NativeCallBlocker::ValueOperandEvaluationCleanup,
                    ),
                ),
                constructor
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::constructor_value(
                        span,
                        NativeCallBlocker::LvalueOperandEvaluationCleanup,
                    ),
                ),
                constructor
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::function_frame(
                        span,
                        NativeCallBlocker::ByReferenceArgumentBinding,
                    ),
                ),
                frame
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::function_frame(
                        span,
                        NativeCallBlocker::VariadicArgumentBinding,
                    ),
                ),
                frame
            );
            assert_eq!(
                native_call_blocker_message(backend, NativeCallOperation::return_value(span)),
                frame
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::closure_frame(
                        span,
                        NativeCallBlocker::ClosureFrameHandoff,
                    ),
                ),
                closure
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::closure_frame(
                        span,
                        NativeCallBlocker::ByReferenceArgumentBinding,
                    ),
                ),
                closure
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::reference_result(span, NativeCallCallee::DirectNamed),
                ),
                reference
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::reference_result(
                        span,
                        NativeCallCallee::DynamicExpression,
                    ),
                ),
                reference
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::reference_result(span, NativeCallCallee::MethodDispatch),
                ),
                reference
            );
            assert_eq!(
                native_call_blocker_message(
                    backend,
                    NativeCallOperation::reference_result(
                        span,
                        NativeCallCallee::ConstructorDispatch,
                    ),
                ),
                reference
            );
        }
    }
}
