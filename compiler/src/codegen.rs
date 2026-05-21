use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::process::{Command, Stdio};

use crate::ast::{
    ArrayItem, AssignTarget, BinaryOp, ClassMember, Expr, NewClassName, Program, Span, Stmt,
    UnaryOp, UnsetTarget,
};
use crate::error::{CompileResult, Diagnostic, Phase};

const MAX_KNOWN_INT_VALUES: usize = 4;
const MAX_KNOWN_FLOAT_VALUES: usize = 4;
const MAX_KNOWN_STRING_VALUES: usize = 4;
const LLVM_CONDITIONAL_REJECTION: &str = "LLVM conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior";
const ASSEMBLY_CONDITIONAL_REJECTION: &str = "assembly conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior";
const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const ASSEMBLY_FUNCTION_CALL_REJECTION: &str = "assembly function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_VALUE_DEBUG_OUTPUT_REJECTION: &str = "LLVM debug-output builtin lowering rejects var_dump() and print_r() until native value formatting, stdout side effects, return-output ownership, references/copy-on-write, and exact native diagnostics are wired through the LLVM backend; generated-native C routes lowerable direct-output forms through the shared runtime debug-output contract";
const ASSEMBLY_VALUE_DEBUG_OUTPUT_REJECTION: &str = "assembly debug-output builtin lowering rejects forms outside the reusable native value debug-output contract, including non-lowerable values and print_r() return-output ownership; lowerable var_dump() and direct-output print_r() values route through owned NativeValueHandle formatting, stdout, diagnostics, and cleanup";
const LLVM_STR_STARTS_WITH_REJECTION: &str = "LLVM str_starts_with lowering rejects direct string-prefix calls until native PHP string conversion, empty-needle handling, binary string byte semantics, argument diagnostics, references/copy-on-write, and exact native str_starts_with diagnostics exist; phpc run handles current bounded str_starts_with behavior";
const ASSEMBLY_STR_STARTS_WITH_REJECTION: &str = "assembly str_starts_with lowering rejects direct string-prefix calls until native PHP string conversion, empty-needle handling, binary string byte semantics, argument diagnostics, references/copy-on-write, and exact native str_starts_with diagnostics exist; phpc run handles current bounded str_starts_with behavior";
const LLVM_STR_ENDS_WITH_REJECTION: &str = "LLVM str_ends_with lowering rejects direct string-suffix calls until native PHP string conversion, empty-needle handling, binary string byte semantics, argument diagnostics, references/copy-on-write, and exact native str_ends_with diagnostics exist; phpc run handles current bounded str_ends_with behavior";
const ASSEMBLY_STR_ENDS_WITH_REJECTION: &str = "assembly str_ends_with lowering rejects direct string-suffix calls until native PHP string conversion, empty-needle handling, binary string byte semantics, argument diagnostics, references/copy-on-write, and exact native str_ends_with diagnostics exist; phpc run handles current bounded str_ends_with behavior";
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
const LLVM_DYNAMIC_FUNCTION_CALL_REJECTION: &str = "LLVM dynamic function-call lowering rejects variable-call expressions such as $name(...) until native callable expression evaluation, runtime function lookup, stack frames, arity/type diagnostics, callback dispatch, and exact native callable errors exist; phpc run handles current string-valued dynamic function calls";
const ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION: &str = "assembly dynamic function-call lowering rejects variable-call expressions such as $name(...) until native callable expression evaluation, runtime function lookup, stack frames, arity/type diagnostics, callback dispatch, and exact native callable errors exist; phpc run handles current string-valued dynamic function calls";
const LLVM_TERMINATION_REJECTION: &str = "LLVM termination lowering rejects exit()/die() until native termination control flow, exit status/stdout handoff, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die behavior";
const ASSEMBLY_TERMINATION_REJECTION: &str = "assembly termination lowering rejects exit()/die() until native termination control flow, exit status/stdout handoff, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die behavior";
const LLVM_TERMINATION_PARTIAL_BRANCH_REJECTION: &str = "LLVM termination control-flow lowering rejects exit()/die() in branches that may continue until native branch merge cleanup, live value-handle ownership across paths, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die branch behavior";
const ASSEMBLY_TERMINATION_PARTIAL_BRANCH_REJECTION: &str = "assembly termination control-flow lowering rejects exit()/die() in branches that may continue until native branch merge cleanup, live value-handle ownership across paths, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die branch behavior";
const LLVM_TERMINATION_LOOP_SCOPE_REJECTION: &str = "LLVM termination cleanup-stack lowering rejects exit()/die() across loop scopes until native loop unwind cleanup, branch/block local ownership, live value-handle ownership, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die loop behavior";
const ASSEMBLY_TERMINATION_LOOP_SCOPE_REJECTION: &str = "assembly termination cleanup-stack lowering rejects exit()/die() across loop scopes until native loop unwind cleanup, branch/block local ownership, live value-handle ownership, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die loop behavior";
const LLVM_TERMINATION_SWITCH_SCOPE_REJECTION: &str = "LLVM termination cleanup-stack lowering rejects exit()/die() across switch scopes until native switch unwind cleanup, case/fallthrough ownership, live value-handle ownership, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die switch behavior";
const ASSEMBLY_TERMINATION_SWITCH_SCOPE_REJECTION: &str = "assembly termination cleanup-stack lowering rejects exit()/die() across switch scopes until native switch unwind cleanup, case/fallthrough ownership, live value-handle ownership, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die switch behavior";
const LLVM_TERMINATION_GOTO_SCOPE_REJECTION: &str = "LLVM termination cleanup-stack lowering rejects exit()/die() across goto/label scopes until native goto reachability, label ownership, branch/block local cleanup, live value-handle ownership, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die goto behavior";
const ASSEMBLY_TERMINATION_GOTO_SCOPE_REJECTION: &str = "assembly termination cleanup-stack lowering rejects exit()/die() across goto/label scopes until native goto reachability, label ownership, branch/block local cleanup, live value-handle ownership, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die goto behavior";
const LLVM_TERMINATION_FUNCTION_FRAME_REJECTION: &str = "LLVM termination cleanup-stack lowering rejects exit()/die() across function frames until native stack-frame unwinding, local/value-handle ownership, return-value handoff, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die function behavior";
const ASSEMBLY_TERMINATION_FUNCTION_FRAME_REJECTION: &str = "assembly termination cleanup-stack lowering rejects exit()/die() across function frames until native stack-frame unwinding, local/value-handle ownership, return-value handoff, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die function behavior";
const LLVM_TERMINATION_RETURN_CONTEXT_REJECTION: &str = "LLVM termination control-flow lowering rejects exit()/die() in return expressions until native function return unwinding, stack-frame cleanup, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die return behavior";
const ASSEMBLY_TERMINATION_RETURN_CONTEXT_REJECTION: &str = "assembly termination control-flow lowering rejects exit()/die() in return expressions until native function return unwinding, stack-frame cleanup, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die return behavior";
const LLVM_TERMINATION_EXPRESSION_CONTEXT_REJECTION: &str = "LLVM termination control-flow lowering rejects exit()/die() in expression contexts until native expression-level termination propagation, temporary cleanup, live value-handle ownership, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded statement-level exit/die behavior";
const ASSEMBLY_TERMINATION_EXPRESSION_CONTEXT_REJECTION: &str = "assembly termination control-flow lowering rejects exit()/die() in expression contexts until native expression-level termination propagation, temporary cleanup, live value-handle ownership, shutdown functions, destructors/finally ordering, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded statement-level exit/die behavior";
const LLVM_TERMINATION_TRY_CONTEXT_REJECTION: &str = "LLVM termination control-flow lowering rejects exit()/die() in try/catch/finally contexts until native finally dispatch during termination, exception/termination unwinding, shutdown functions, destructors, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die try/finally behavior";
const ASSEMBLY_TERMINATION_TRY_CONTEXT_REJECTION: &str = "assembly termination control-flow lowering rejects exit()/die() in try/catch/finally contexts until native finally dispatch during termination, exception/termination unwinding, shutdown functions, destructors, output buffers, SAPI interaction, and exact native diagnostics exist; phpc run handles current bounded exit/die try/finally behavior";
const LLVM_TERMINATION_OUTPUT_BUFFER_REJECTION: &str = "LLVM termination hook lowering rejects exit()/die() with active or queried output buffers until native output-buffer stack flushing, discard/flush ordering, shutdown flushing, SAPI interaction, live value cleanup, and exact native diagnostics exist; phpc run handles current bounded output-buffer termination behavior";
const ASSEMBLY_TERMINATION_OUTPUT_BUFFER_REJECTION: &str = "assembly termination hook lowering rejects exit()/die() with active or queried output buffers until native output-buffer stack flushing, discard/flush ordering, shutdown flushing, SAPI interaction, live value cleanup, and exact native diagnostics exist; phpc run handles current bounded output-buffer termination behavior";
const LLVM_TERMINATION_SHUTDOWN_REJECTION: &str = "LLVM termination hook lowering rejects exit()/die() after shutdown-function registration until native shutdown queue storage, callback invocation ordering, output-buffer flushing, destructor/finally interaction, live value cleanup, and exact native diagnostics exist; phpc run handles current bounded shutdown behavior";
const ASSEMBLY_TERMINATION_SHUTDOWN_REJECTION: &str = "assembly termination hook lowering rejects exit()/die() after shutdown-function registration until native shutdown queue storage, callback invocation ordering, output-buffer flushing, destructor/finally interaction, live value cleanup, and exact native diagnostics exist; phpc run handles current bounded shutdown behavior";
const LLVM_TERMINATION_DESTRUCTOR_REJECTION: &str = "LLVM termination hook lowering rejects exit()/die() with pending object destructor semantics until native object lifetime tracking, destructor ordering, shutdown/finally interaction, output-buffer flushing, live value cleanup, and exact native diagnostics exist; phpc run handles current bounded destructor behavior";
const ASSEMBLY_TERMINATION_DESTRUCTOR_REJECTION: &str = "assembly termination hook lowering rejects exit()/die() with pending object destructor semantics until native object lifetime tracking, destructor ordering, shutdown/finally interaction, output-buffer flushing, live value cleanup, and exact native diagnostics exist; phpc run handles current bounded destructor behavior";
const LLVM_TERMINATION_EXCEPTION_REJECTION: &str = "LLVM termination hook lowering rejects exit()/die() across exception-control contexts until native exception unwinding, finally dispatch, shutdown/destructor interaction, output-buffer flushing, live value cleanup, and exact native diagnostics exist; phpc run handles current bounded exception behavior";
const ASSEMBLY_TERMINATION_EXCEPTION_REJECTION: &str = "assembly termination hook lowering rejects exit()/die() across exception-control contexts until native exception unwinding, finally dispatch, shutdown/destructor interaction, output-buffer flushing, live value cleanup, and exact native diagnostics exist; phpc run handles current bounded exception behavior";
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
const ASSEMBLY_ARRAY_REJECTION: &str = "assembly array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";
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
const LLVM_MUTATION_REJECTION: &str = "LLVM mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, assignment expressions, object property unset, static property unset, non-local unset operands, and mixed multiple-operand unset until native read-modify-write ordering, null-aware mutation, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";
const ASSEMBLY_MUTATION_REJECTION: &str = "assembly mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, assignment expressions, direct variable unset, object property unset, static property unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";
const LLVM_ISSET_REJECTION: &str = "LLVM isset lowering rejects array offset operands, object property operands, static property operands, complex operands, unsupported multiple operands, and unset/mutation interactions until native symbol-table storage, null-aware lookup, references/copy-on-write, and exact native error behavior exist; phpc run handles current isset behavior";
const ASSEMBLY_ISSET_REJECTION: &str = "assembly isset lowering rejects array offset operands, object property operands, complex operands, multiple operands, and unset/mutation interactions until native symbol-table storage, null-aware lookup, references/copy-on-write, and exact native error behavior exist; phpc run handles current isset behavior";
const LLVM_EMPTY_REJECTION: &str = "LLVM empty lowering rejects array offset operands, object property operands, static property operands, complex operands, arrays, unset/mutation interactions, and ambiguous truthiness until native symbol-table storage, PHP truthiness, references/copy-on-write, and exact native error behavior exist; phpc run handles current empty behavior";
const ASSEMBLY_EMPTY_REJECTION: &str = "assembly empty lowering rejects array offset operands, object property operands, complex operands, arrays, unset/mutation interactions, and ambiguous truthiness until native symbol-table storage, PHP truthiness, references/copy-on-write, and exact native error behavior exist; phpc run handles current empty behavior";
const LLVM_ERROR_CONTROL_REJECTION: &str = "LLVM error-control lowering rejects @expr until native diagnostic severity, warning/notice/deprecation suppression, error_reporting() mask interaction, recoverable expression values, and exact native diagnostics exist; phpc run handles current transparent error-control wrapper behavior";
const ASSEMBLY_ERROR_CONTROL_REJECTION: &str = "assembly error-control lowering rejects @expr until native diagnostic severity, warning/notice/deprecation suppression, error_reporting() mask interaction, recoverable expression values, and exact native diagnostics exist; phpc run handles current transparent error-control wrapper behavior";
const LLVM_CAST_REJECTION: &str = "LLVM cast lowering rejects (string), (int)/(integer), (bool)/(boolean), (float)/(double), and (array) casts until native PHP scalar conversion, array materialization, warning/recovery behavior, object/resource handling, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded cast behavior";
const ASSEMBLY_CAST_REJECTION: &str = "assembly cast lowering rejects (string), (int)/(integer), (bool)/(boolean), (float)/(double), and (array) casts until native PHP scalar conversion, array materialization, warning/recovery behavior, object/resource handling, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded cast behavior";
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

fn request_superglobal_expr_span(expr: &Expr) -> Option<Span> {
    match expr {
        Expr::Variable(name, span) if is_request_superglobal_name(name) => Some(*span),
        Expr::Index { target, .. } | Expr::AppendIndex { target, .. } => {
            request_superglobal_expr_span(target)
        }
        _ => None,
    }
}

fn request_superglobal_name_span(name: &str, span: Span) -> Option<Span> {
    is_request_superglobal_name(name).then_some(span)
}

fn request_superglobal_consumed_many_span<'a>(
    exprs: impl IntoIterator<Item = &'a Expr>,
) -> Option<Span> {
    exprs
        .into_iter()
        .find_map(request_superglobal_consumed_expr_span)
}

fn request_superglobal_consumed_expr_span(expr: &Expr) -> Option<Span> {
    if let Some(span) = request_superglobal_expr_span(expr) {
        return Some(span);
    }

    match expr {
        Expr::Array { items, .. } => items.iter().find_map(|item| {
            item.key
                .as_ref()
                .and_then(request_superglobal_consumed_expr_span)
                .or_else(|| request_superglobal_consumed_expr_span(&item.value))
        }),
        Expr::Index { index, .. } => request_superglobal_consumed_expr_span(index),
        Expr::Property { target, .. } => request_superglobal_consumed_expr_span(target),
        Expr::DynamicProperty {
            target, property, ..
        } => request_superglobal_consumed_expr_span(target)
            .or_else(|| request_superglobal_consumed_expr_span(property)),
        Expr::ObjectStaticProperty { target, .. } => request_superglobal_consumed_expr_span(target),
        Expr::MethodCall { target, args, .. } => request_superglobal_consumed_expr_span(target)
            .or_else(|| request_superglobal_consumed_args_span(args)),
        Expr::DynamicMethodCall {
            target,
            method,
            args,
            ..
        } => request_superglobal_consumed_expr_span(target)
            .or_else(|| request_superglobal_consumed_expr_span(method))
            .or_else(|| request_superglobal_consumed_args_span(args)),
        Expr::ParentMethodCall { args, .. }
        | Expr::StaticMethodCall { args, .. }
        | Expr::SelfMethodCall { args, .. }
        | Expr::LateStaticMethodCall { args, .. }
        | Expr::Call { args, .. } => request_superglobal_consumed_args_span(args),
        Expr::ObjectStaticMethodCall { target, args, .. } => {
            request_superglobal_consumed_expr_span(target)
                .or_else(|| request_superglobal_consumed_args_span(args))
        }
        Expr::DynamicCall { callee, args, .. } => request_superglobal_consumed_expr_span(callee)
            .or_else(|| request_superglobal_consumed_args_span(args)),
        Expr::InstanceOf { expr, .. }
        | Expr::Clone { expr, .. }
        | Expr::Unary { expr, .. }
        | Expr::ErrorControl { expr, .. }
        | Expr::Include { path: expr, .. }
        | Expr::Require { path: expr, .. }
        | Expr::Cast { expr, .. } => request_superglobal_consumed_expr_span(expr),
        Expr::New {
            class_name,
            args,
            span,
        } => request_superglobal_consumed_new_class_name_span(class_name, *span)
            .or_else(|| request_superglobal_consumed_args_span(args)),
        Expr::Binary { left, right, .. } => request_superglobal_consumed_expr_span(left)
            .or_else(|| request_superglobal_consumed_expr_span(right)),
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => request_superglobal_consumed_expr_span(condition)
            .or_else(|| request_superglobal_consumed_expr_span(if_true))
            .or_else(|| request_superglobal_consumed_expr_span(if_false)),
        Expr::ShortTernary {
            condition,
            if_false,
            ..
        } => request_superglobal_consumed_expr_span(condition)
            .or_else(|| request_superglobal_consumed_expr_span(if_false)),
        Expr::Assign { expr, .. }
        | Expr::CompoundAssign { expr, .. }
        | Expr::NullCoalesceAssign { expr, .. } => request_superglobal_consumed_expr_span(expr),
        Expr::IncrementDecrement { .. } => None,
        _ => None,
    }
}

fn request_superglobal_consumed_args_span(args: &[Expr]) -> Option<Span> {
    args.iter().find_map(request_superglobal_consumed_expr_span)
}

fn request_superglobal_consumed_new_class_name_span(
    class_name: &NewClassName,
    span: Span,
) -> Option<Span> {
    match class_name {
        NewClassName::DynamicVariable(name) => request_superglobal_name_span(name, span),
        NewClassName::Named(_)
        | NewClassName::SelfClass
        | NewClassName::ParentClass
        | NewClassName::StaticClass => None,
    }
}

fn request_superglobal_consumed_assign_target_span(target: &AssignTarget) -> Option<Span> {
    match target {
        AssignTarget::Variable { name, span } => request_superglobal_name_span(name, *span),
        AssignTarget::List { .. } => None,
        AssignTarget::ArrayIndex { name, index, span } => {
            request_superglobal_name_span(name, *span).or_else(|| {
                index
                    .as_ref()
                    .and_then(request_superglobal_consumed_expr_span)
            })
        }
        AssignTarget::NestedArrayIndex {
            name,
            indices,
            span,
        } => request_superglobal_name_span(name, *span)
            .or_else(|| request_superglobal_consumed_many_span(indices)),
        AssignTarget::NestedArrayAppend {
            name,
            indices,
            suffix_indices,
            span,
        } => request_superglobal_name_span(name, *span)
            .or_else(|| request_superglobal_consumed_many_span(indices))
            .or_else(|| request_superglobal_consumed_many_span(suffix_indices)),
        AssignTarget::Property { object, span, .. } => request_superglobal_name_span(object, *span),
        AssignTarget::DynamicProperty {
            object,
            property,
            span,
        } => request_superglobal_name_span(object, *span)
            .or_else(|| request_superglobal_consumed_expr_span(property)),
        AssignTarget::NonDirectProperty { holder, .. } => {
            request_superglobal_consumed_expr_span(holder)
        }
        AssignTarget::NonDirectDynamicProperty {
            holder, property, ..
        } => request_superglobal_consumed_expr_span(holder)
            .or_else(|| request_superglobal_consumed_expr_span(property)),
        AssignTarget::ObjectPropertyArrayIndex {
            object,
            indices,
            span,
            ..
        } => request_superglobal_name_span(object, *span)
            .or_else(|| request_superglobal_consumed_many_span(indices)),
        AssignTarget::ObjectPropertyArrayAppend {
            object,
            indices,
            suffix_indices,
            span,
            ..
        } => request_superglobal_name_span(object, *span)
            .or_else(|| request_superglobal_consumed_many_span(indices))
            .or_else(|| request_superglobal_consumed_many_span(suffix_indices)),
        AssignTarget::DynamicObjectPropertyArrayIndex {
            object,
            property,
            indices,
            span,
        } => request_superglobal_name_span(object, *span)
            .or_else(|| request_superglobal_consumed_expr_span(property))
            .or_else(|| request_superglobal_consumed_many_span(indices)),
        AssignTarget::DynamicObjectPropertyArrayAppend {
            object,
            property,
            indices,
            suffix_indices,
            span,
            ..
        } => request_superglobal_name_span(object, *span)
            .or_else(|| request_superglobal_consumed_expr_span(property))
            .or_else(|| request_superglobal_consumed_many_span(indices))
            .or_else(|| request_superglobal_consumed_many_span(suffix_indices)),
        AssignTarget::NonDirectObjectPropertyArrayIndex {
            holder, indices, ..
        } => request_superglobal_consumed_expr_span(holder)
            .or_else(|| request_superglobal_consumed_many_span(indices)),
        AssignTarget::NonDirectObjectPropertyArrayAppend {
            holder,
            indices,
            suffix_indices,
            ..
        } => request_superglobal_consumed_expr_span(holder)
            .or_else(|| request_superglobal_consumed_many_span(indices))
            .or_else(|| request_superglobal_consumed_many_span(suffix_indices)),
        AssignTarget::NonDirectDynamicObjectPropertyArrayIndex {
            holder,
            property,
            indices,
            ..
        } => request_superglobal_consumed_expr_span(holder)
            .or_else(|| request_superglobal_consumed_expr_span(property))
            .or_else(|| request_superglobal_consumed_many_span(indices)),
        AssignTarget::NonDirectDynamicObjectPropertyArrayAppend {
            holder,
            property,
            indices,
            suffix_indices,
            ..
        } => request_superglobal_consumed_expr_span(holder)
            .or_else(|| request_superglobal_consumed_expr_span(property))
            .or_else(|| request_superglobal_consumed_many_span(indices))
            .or_else(|| request_superglobal_consumed_many_span(suffix_indices)),
        AssignTarget::ObjectStaticProperty { target, .. } => {
            request_superglobal_consumed_expr_span(target)
        }
        AssignTarget::StaticProperty { .. }
        | AssignTarget::SelfStaticProperty { .. }
        | AssignTarget::ParentStaticProperty { .. }
        | AssignTarget::LateStaticProperty { .. } => None,
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

fn native_value_blocker_for_expr(expr: &Expr) -> Option<NativeValueBlocker> {
    if request_superglobal_expr_span(expr).is_some() {
        return Some(NativeValueBlocker::RequestState);
    }

    match expr {
        Expr::Array { .. } => Some(NativeValueBlocker::Array),
        Expr::Index { target, .. } | Expr::AppendIndex { target, .. } => {
            if is_object_offset_expr(target) {
                Some(NativeValueBlocker::ArrayAccess)
            } else {
                Some(NativeValueBlocker::Array)
            }
        }
        Expr::Property { .. } | Expr::DynamicProperty { .. } => {
            Some(NativeValueBlocker::ObjectProperty)
        }
        Expr::MethodCall { .. }
        | Expr::DynamicMethodCall { .. }
        | Expr::ParentMethodCall { .. }
        | Expr::StaticMethodCall { .. }
        | Expr::ObjectStaticMethodCall { .. }
        | Expr::SelfMethodCall { .. }
        | Expr::LateStaticMethodCall { .. } => Some(NativeValueBlocker::MethodCall),
        Expr::New { .. } => Some(NativeValueBlocker::ObjectInstantiation),
        Expr::Clone { .. } => Some(NativeValueBlocker::ObjectClone),
        Expr::InstanceOf { .. } => Some(NativeValueBlocker::InstanceOf),
        Expr::Call { name, .. } if is_stream_resource_builtin(name) => {
            Some(NativeValueBlocker::Resource)
        }
        Expr::DynamicCall { .. } => Some(NativeValueBlocker::DynamicCall),
        Expr::Assign { .. }
        | Expr::CompoundAssign { .. }
        | Expr::NullCoalesceAssign { .. }
        | Expr::IncrementDecrement { .. } => Some(NativeValueBlocker::CopyOnWrite),
        _ => None,
    }
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
        "%phpc.NativeStringConversionResult = type { %phpc.NativeByteBuffer, %phpc.NativeDiagnosticHandle }",
        "%phpc.NativeStringHandle = type { ptr }",
        "%phpc.NativeValueHandle = type { ptr }",
        "%phpc.NativeDiagnosticHandle = type { ptr }",
        "%phpc.NativeArrayHandle = type { ptr }",
        "%phpc.NativeArrayKeySnapshotHandle = type { ptr }",
        "%phpc.NativeArrayEntrySnapshotHandle = type { ptr }",
        &format!("%phpc.NativeArrayKeyMetadata = type {{ i8, [7 x i8], i64, {usize_type} }}"),
        "%phpc.NativeArrayKeyMaterializationResult = type { i8, [7 x i8], i64, %phpc.NativeByteBuffer, %phpc.NativeDiagnosticHandle }",
        "%phpc.NativeClassMetadataHandle = type { ptr }",
        "%phpc.NativeObjectHandle = type { ptr }",
        "%phpc.NativeResourceHandle = type { ptr }",
        "%phpc.NativeReferenceHandle = type { ptr }",
        "%phpc.NativeRequestStateHandle = type { ptr }",
        "%phpc.NativeSymbolTableHandle = type { ptr }",
        "@phpc.probe.bytes = private unnamed_addr constant [4 x i8] c\"heap\"",
        "@phpc.probe.string = private unnamed_addr constant [7 x i8] c\"php\\00abi\"",
        "@phpc.probe.invalid = private unnamed_addr constant [1 x i8] c\"\\FF\"",
        "@phpc.probe.binary = private unnamed_addr constant [4 x i8] c\"A\\00\\FF\\0A\"",
        "@phpc.probe.zero = private unnamed_addr constant [1 x i8] c\"0\"",
        "@phpc.probe.symbol = private unnamed_addr constant [5 x i8] c\"label\"",
        "@phpc.probe.text_membership_candidates = private unnamed_addr constant [1 x ptr] [ptr @phpc.probe.bytes]",
        &format!(
            "@phpc.probe.text_membership_candidate_lengths = private unnamed_addr constant [1 x {usize_type}] [{usize_type} 4]"
        ),
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
        &format!("declare i1 @phpc_native_string_truthy(ptr, {usize_type})"),
        "declare void @phpc_native_string_free(%phpc.NativeStringHandle)",
        "declare %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue)",
        "declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)",
        "declare %phpc.NativeValueHandle @phpc_native_value_from_string_with_diagnostic(%phpc.NativeStringHandle, ptr)",
        &format!(
            "declare %phpc.NativeValueHandle @phpc_native_binary_value_from_bytes(ptr, {usize_type})"
        ),
        "declare %phpc.NativeValueHandle @phpc_native_value_clone(%phpc.NativeValueHandle)",
        "declare i1 @phpc_native_value_truthy(%phpc.NativeValueHandle)",
        &format!("declare {usize_type} @phpc_native_value_string_len(%phpc.NativeValueHandle)"),
        &format!(
            "declare {usize_type} @phpc_native_value_string_len_with_diagnostic(%phpc.NativeValueHandle, ptr)"
        ),
        "declare %phpc.NativeByteBuffer @phpc_native_value_string_clone_bytes(%phpc.NativeValueHandle)",
        "declare %phpc.NativeByteBuffer @phpc_native_value_string_clone_bytes_with_diagnostic(%phpc.NativeValueHandle, ptr)",
        "declare %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle)",
        "declare %phpc.NativeByteBuffer @phpc_native_value_echo_bytes_with_diagnostic(%phpc.NativeValueHandle, ptr)",
        "declare %phpc.NativeByteBuffer @phpc_native_value_serialize_bytes_with_diagnostic(%phpc.NativeValueHandle, ptr)",
        "declare %phpc.NativeByteBuffer @phpc_native_value_var_dump_bytes_with_diagnostic(%phpc.NativeValueHandle, ptr)",
        "declare %phpc.NativeByteBuffer @phpc_native_value_print_r_bytes_with_diagnostic(%phpc.NativeValueHandle, ptr)",
        &format!("declare {usize_type} @phpc_native_value_echo_stdout(%phpc.NativeValueHandle)"),
        "declare void @phpc_native_value_free(%phpc.NativeValueHandle)",
        "declare %phpc.NativeStringConversionResult @phpc_native_value_to_string_bytes(%phpc.NativeValueHandle)",
        "declare %phpc.NativeStringConversionResult @phpc_native_value_text_bytes(%phpc.NativeValueHandle, i8)",
        &format!(
            "declare i1 @phpc_native_value_text_membership_with_diagnostic(%phpc.NativeValueHandle, i8, ptr, ptr, {usize_type}, i1, ptr)"
        ),
        "declare %phpc.NativeValueHandle @phpc_native_value_string_array_operation_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeValueHandle, i64, i8, i8, ptr)",
        "declare void @phpc_native_string_conversion_result_free(%phpc.NativeStringConversionResult)",
        &format!(
            "declare {usize_type} @phpc_native_diagnostic_message_len(%phpc.NativeDiagnosticHandle)"
        ),
        "declare %phpc.NativeByteBuffer @phpc_native_diagnostic_message_clone_bytes(%phpc.NativeDiagnosticHandle)",
        &format!(
            "declare {usize_type} @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)"
        ),
        "declare void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle)",
        "declare %phpc.NativeArrayHandle @phpc_native_array_null()",
        "declare %phpc.NativeArrayHandle @phpc_native_array_empty()",
        "declare i1 @phpc_native_array_is_null(%phpc.NativeArrayHandle)",
        &format!("declare {usize_type} @phpc_native_array_len(%phpc.NativeArrayHandle)"),
        &format!(
            "declare {usize_type} @phpc_native_value_array_len_with_diagnostic(%phpc.NativeValueHandle, ptr)"
        ),
        "declare i1 @phpc_native_array_append_scalar(%phpc.NativeArrayHandle, %phpc.NativeScalarValue)",
        "declare i1 @phpc_native_array_append_value(%phpc.NativeArrayHandle, %phpc.NativeValueHandle)",
        "declare i1 @phpc_native_array_write_int_scalar(%phpc.NativeArrayHandle, i64, %phpc.NativeScalarValue)",
        "declare i1 @phpc_native_array_write_int_value(%phpc.NativeArrayHandle, i64, %phpc.NativeValueHandle)",
        "declare i1 @phpc_native_array_write_string_scalar(%phpc.NativeArrayHandle, %phpc.NativeStringHandle, %phpc.NativeScalarValue)",
        "declare i1 @phpc_native_array_write_string_value(%phpc.NativeArrayHandle, %phpc.NativeStringHandle, %phpc.NativeValueHandle)",
        "declare %phpc.NativeValueHandle @phpc_native_array_read_int(%phpc.NativeArrayHandle, i64)",
        "declare %phpc.NativeValueHandle @phpc_native_array_read_string(%phpc.NativeArrayHandle, %phpc.NativeStringHandle)",
        "declare %phpc.NativeValueHandle @phpc_native_value_from_array_clone(%phpc.NativeArrayHandle)",
        "declare %phpc.NativeArrayKeySnapshotHandle @phpc_native_array_key_snapshot(%phpc.NativeArrayHandle)",
        "declare i1 @phpc_native_array_key_snapshot_is_null(%phpc.NativeArrayKeySnapshotHandle)",
        &format!(
            "declare {usize_type} @phpc_native_array_key_snapshot_len(%phpc.NativeArrayKeySnapshotHandle)"
        ),
        &format!(
            "declare %phpc.NativeArrayKeyMetadata @phpc_native_array_key_snapshot_key_at(%phpc.NativeArrayKeySnapshotHandle, {usize_type})"
        ),
        &format!(
            "declare %phpc.NativeStringHandle @phpc_native_array_key_snapshot_string_clone_at(%phpc.NativeArrayKeySnapshotHandle, {usize_type})"
        ),
        "declare void @phpc_native_array_key_snapshot_free(%phpc.NativeArrayKeySnapshotHandle)",
        "declare %phpc.NativeArrayKeyMaterializationResult @phpc_native_value_to_array_key(%phpc.NativeValueHandle)",
        "declare i1 @phpc_native_value_array_key_exists_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeValueHandle, ptr)",
        "declare %phpc.NativeValueHandle @phpc_native_value_array_operation_with_diagnostic(%phpc.NativeValueHandle, i8, ptr)",
        "declare %phpc.NativeValueHandle @phpc_native_value_array_query_operation_with_diagnostic(%phpc.NativeValueHandle, %phpc.NativeValueHandle, i8, i8, ptr)",
        "declare %phpc.NativeStringConversionResult @phpc_native_array_key_materialization_text_bytes(%phpc.NativeArrayKeyMaterializationResult, i8)",
        "declare %phpc.NativeValueHandle @phpc_native_array_key_materialization_to_value_with_diagnostic(%phpc.NativeArrayKeyMaterializationResult, ptr)",
        "declare void @phpc_native_array_key_materialization_result_free(%phpc.NativeArrayKeyMaterializationResult)",
        "declare %phpc.NativeArrayEntrySnapshotHandle @phpc_native_array_entry_snapshot(%phpc.NativeArrayHandle)",
        "declare i1 @phpc_native_array_entry_snapshot_is_null(%phpc.NativeArrayEntrySnapshotHandle)",
        &format!(
            "declare {usize_type} @phpc_native_array_entry_snapshot_len(%phpc.NativeArrayEntrySnapshotHandle)"
        ),
        &format!(
            "declare %phpc.NativeArrayKeyMetadata @phpc_native_array_entry_snapshot_key_at(%phpc.NativeArrayEntrySnapshotHandle, {usize_type})"
        ),
        &format!(
            "declare %phpc.NativeStringHandle @phpc_native_array_entry_snapshot_string_clone_at(%phpc.NativeArrayEntrySnapshotHandle, {usize_type})"
        ),
        &format!(
            "declare %phpc.NativeValueHandle @phpc_native_array_entry_snapshot_key_value_clone_at(%phpc.NativeArrayEntrySnapshotHandle, {usize_type})"
        ),
        &format!(
            "declare %phpc.NativeReferenceHandle @phpc_native_array_entry_snapshot_key_reference_clone_at(%phpc.NativeArrayEntrySnapshotHandle, {usize_type})"
        ),
        &format!(
            "declare %phpc.NativeValueHandle @phpc_native_array_entry_snapshot_value_clone_at(%phpc.NativeArrayEntrySnapshotHandle, {usize_type})"
        ),
        &format!(
            "declare %phpc.NativeReferenceHandle @phpc_native_array_entry_snapshot_value_reference_clone_at(%phpc.NativeArrayEntrySnapshotHandle, {usize_type})"
        ),
        "declare void @phpc_native_array_entry_snapshot_free(%phpc.NativeArrayEntrySnapshotHandle)",
        "declare void @phpc_native_array_free(%phpc.NativeArrayHandle)",
        "declare %phpc.NativeClassMetadataHandle @phpc_native_class_metadata_null()",
        "declare i1 @phpc_native_class_metadata_is_null(%phpc.NativeClassMetadataHandle)",
        &format!(
            "declare %phpc.NativeClassMetadataHandle @phpc_native_class_metadata_from_name(ptr, {usize_type})"
        ),
        "declare %phpc.NativeClassMetadataHandle @phpc_native_class_metadata_from_string(%phpc.NativeStringHandle)",
        &format!(
            "declare {usize_type} @phpc_native_class_metadata_name_len(%phpc.NativeClassMetadataHandle)"
        ),
        "declare %phpc.NativeByteBuffer @phpc_native_class_metadata_name_clone_bytes(%phpc.NativeClassMetadataHandle)",
        "declare void @phpc_native_class_metadata_free(%phpc.NativeClassMetadataHandle)",
        "declare %phpc.NativeObjectHandle @phpc_native_object_null()",
        "declare i1 @phpc_native_object_is_null(%phpc.NativeObjectHandle)",
        "declare %phpc.NativeObjectHandle @phpc_native_object_alloc(%phpc.NativeClassMetadataHandle)",
        &format!(
            "declare {usize_type} @phpc_native_object_class_name_len(%phpc.NativeObjectHandle)"
        ),
        "declare %phpc.NativeByteBuffer @phpc_native_object_class_name_clone_bytes(%phpc.NativeObjectHandle)",
        "declare %phpc.NativeClassMetadataHandle @phpc_native_object_class_metadata_clone(%phpc.NativeObjectHandle)",
        "declare void @phpc_native_object_free(%phpc.NativeObjectHandle)",
        "declare %phpc.NativeResourceHandle @phpc_native_resource_null()",
        "declare i1 @phpc_native_resource_is_null(%phpc.NativeResourceHandle)",
        "declare %phpc.NativeReferenceHandle @phpc_native_reference_null()",
        "declare i1 @phpc_native_reference_is_null(%phpc.NativeReferenceHandle)",
        "declare i1 @phpc_native_reference_is_empty(%phpc.NativeReferenceHandle)",
        "declare %phpc.NativeReferenceHandle @phpc_native_reference_from_scalar(%phpc.NativeScalarValue)",
        "declare %phpc.NativeReferenceHandle @phpc_native_reference_from_value(%phpc.NativeValueHandle)",
        "declare %phpc.NativeReferenceHandle @phpc_native_reference_clone(%phpc.NativeReferenceHandle)",
        "declare %phpc.NativeValueHandle @phpc_native_reference_read_value(%phpc.NativeReferenceHandle)",
        "declare i1 @phpc_native_reference_write_scalar(%phpc.NativeReferenceHandle, %phpc.NativeScalarValue)",
        "declare i1 @phpc_native_reference_write_value(%phpc.NativeReferenceHandle, %phpc.NativeValueHandle)",
        "declare i1 @phpc_native_reference_write_reference(%phpc.NativeReferenceHandle, %phpc.NativeReferenceHandle)",
        "declare %phpc.NativeStringConversionResult @phpc_native_reference_to_string_bytes(%phpc.NativeReferenceHandle)",
        "declare void @phpc_native_reference_free(%phpc.NativeReferenceHandle)",
        "declare %phpc.NativeRequestStateHandle @phpc_native_request_state_null()",
        "declare i1 @phpc_native_request_state_is_null(%phpc.NativeRequestStateHandle)",
        "declare %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_null()",
        "declare %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()",
        "declare i1 @phpc_native_symbol_table_is_null(%phpc.NativeSymbolTableHandle)",
        &format!(
            "declare i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle, ptr, {usize_type}, %phpc.NativeValueHandle)"
        ),
        &format!(
            "declare %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle, ptr, {usize_type})"
        ),
        "declare void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle)",
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
        &format!("define {usize_type} @phpc_probe_value_string_byte_diagnostics() {{"),
        "entry:",
        "  %len_diagnostic_slot = alloca %phpc.NativeDiagnosticHandle",
        "  %clone_diagnostic_slot = alloca %phpc.NativeDiagnosticHandle",
        &format!(
            "  %bytes = getelementptr inbounds [4 x i8], ptr @phpc.probe.binary, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %value = call %phpc.NativeValueHandle @phpc_native_binary_value_from_bytes(ptr %bytes, {usize_type} 4)"
        ),
        &format!(
            "  %len = call {usize_type} @phpc_native_value_string_len(%phpc.NativeValueHandle %value)"
        ),
        &format!(
            "  %len_with_diagnostic = call {usize_type} @phpc_native_value_string_len_with_diagnostic(%phpc.NativeValueHandle %value, ptr %len_diagnostic_slot)"
        ),
        "  %clone = call %phpc.NativeByteBuffer @phpc_native_value_string_clone_bytes(%phpc.NativeValueHandle %value)",
        &format!("  %clone_len = extractvalue %phpc.NativeByteBuffer %clone, 1"),
        "  %clone_with_diagnostic = call %phpc.NativeByteBuffer @phpc_native_value_string_clone_bytes_with_diagnostic(%phpc.NativeValueHandle %value, ptr %clone_diagnostic_slot)",
        &format!("  %clone_with_diagnostic_len = extractvalue %phpc.NativeByteBuffer %clone_with_diagnostic, 1"),
        "  %len_diagnostic = load %phpc.NativeDiagnosticHandle, ptr %len_diagnostic_slot",
        "  %clone_diagnostic = load %phpc.NativeDiagnosticHandle, ptr %clone_diagnostic_slot",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %clone_with_diagnostic)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %clone)",
        "  call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %clone_diagnostic)",
        "  call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %len_diagnostic)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %value)",
        &format!("  %string_sum0 = add {usize_type} %len, %len_with_diagnostic"),
        &format!("  %string_sum1 = add {usize_type} %string_sum0, %clone_len"),
        &format!("  %string_sum2 = add {usize_type} %string_sum1, %clone_with_diagnostic_len"),
        &format!("  ret {usize_type} %string_sum2"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_value_formatter_diagnostics() {{"),
        "entry:",
        "  %echo_diagnostic_slot = alloca %phpc.NativeDiagnosticHandle",
        "  %serialize_diagnostic_slot = alloca %phpc.NativeDiagnosticHandle",
        "  %dump_diagnostic_slot = alloca %phpc.NativeDiagnosticHandle",
        "  %print_diagnostic_slot = alloca %phpc.NativeDiagnosticHandle",
        &format!(
            "  %bytes = getelementptr inbounds [4 x i8], ptr @phpc.probe.binary, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %value = call %phpc.NativeValueHandle @phpc_native_binary_value_from_bytes(ptr %bytes, {usize_type} 4)"
        ),
        "  %echo = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes_with_diagnostic(%phpc.NativeValueHandle %value, ptr %echo_diagnostic_slot)",
        &format!("  %echo_len = extractvalue %phpc.NativeByteBuffer %echo, 1"),
        "  %serialized = call %phpc.NativeByteBuffer @phpc_native_value_serialize_bytes_with_diagnostic(%phpc.NativeValueHandle %value, ptr %serialize_diagnostic_slot)",
        &format!("  %serialized_len = extractvalue %phpc.NativeByteBuffer %serialized, 1"),
        "  %dump = call %phpc.NativeByteBuffer @phpc_native_value_var_dump_bytes_with_diagnostic(%phpc.NativeValueHandle %value, ptr %dump_diagnostic_slot)",
        &format!("  %dump_len = extractvalue %phpc.NativeByteBuffer %dump, 1"),
        "  %printed = call %phpc.NativeByteBuffer @phpc_native_value_print_r_bytes_with_diagnostic(%phpc.NativeValueHandle %value, ptr %print_diagnostic_slot)",
        &format!("  %printed_len = extractvalue %phpc.NativeByteBuffer %printed, 1"),
        "  %echo_diagnostic = load %phpc.NativeDiagnosticHandle, ptr %echo_diagnostic_slot",
        "  %serialize_diagnostic = load %phpc.NativeDiagnosticHandle, ptr %serialize_diagnostic_slot",
        "  %dump_diagnostic = load %phpc.NativeDiagnosticHandle, ptr %dump_diagnostic_slot",
        "  %print_diagnostic = load %phpc.NativeDiagnosticHandle, ptr %print_diagnostic_slot",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %printed)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %dump)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %serialized)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %echo)",
        "  call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %print_diagnostic)",
        "  call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %dump_diagnostic)",
        "  call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %serialize_diagnostic)",
        "  call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %echo_diagnostic)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %value)",
        &format!("  %format_sum0 = add {usize_type} %echo_len, %serialized_len"),
        &format!("  %format_sum1 = add {usize_type} %format_sum0, %dump_len"),
        &format!("  %format_sum2 = add {usize_type} %format_sum1, %printed_len"),
        &format!("  ret {usize_type} %format_sum2"),
        "}",
        "",
        "define i1 @phpc_probe_string_truthy_boundaries() {",
        "entry:",
        &format!(
            "  %zero = getelementptr inbounds [1 x i8], ptr @phpc.probe.zero, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %text = getelementptr inbounds [7 x i8], ptr @phpc.probe.string, {usize_type} 0, {usize_type} 0"
        ),
        &format!("  %zero_truthy = call i1 @phpc_native_string_truthy(ptr %zero, {usize_type} 1)"),
        &format!("  %text_truthy = call i1 @phpc_native_string_truthy(ptr %text, {usize_type} 7)"),
        &format!("  %empty_truthy = call i1 @phpc_native_string_truthy(ptr null, {usize_type} 0)"),
        "  %zero_false = xor i1 %zero_truthy, true",
        "  %empty_false = xor i1 %empty_truthy, true",
        "  %false_boundaries = and i1 %zero_false, %empty_false",
        "  %result = and i1 %false_boundaries, %text_truthy",
        "  ret i1 %result",
        "}",
        "",
        "define i1 @phpc_probe_native_value_truthy_clone() {",
        "entry:",
        "  %zero_tag = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %zero_scalar = insertvalue %phpc.NativeScalarValue %zero_tag, i64 0, 3",
        "  %zero_value = call %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue %zero_scalar)",
        "  %zero_truthy = call i1 @phpc_native_value_truthy(%phpc.NativeValueHandle %zero_value)",
        "  %zero_false = xor i1 %zero_truthy, true",
        &format!(
            "  %bytes = getelementptr inbounds [7 x i8], ptr @phpc.probe.string, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %string = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr %bytes, {usize_type} 7)"
        ),
        "  %string_value = call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle %string)",
        "  %string_clone = call %phpc.NativeValueHandle @phpc_native_value_clone(%phpc.NativeValueHandle %string_value)",
        "  %string_truthy = call i1 @phpc_native_value_truthy(%phpc.NativeValueHandle %string_value)",
        "  %clone_truthy = call i1 @phpc_native_value_truthy(%phpc.NativeValueHandle %string_clone)",
        "  %array = call %phpc.NativeArrayHandle @phpc_native_array_empty()",
        "  %empty_array = call %phpc.NativeValueHandle @phpc_native_value_from_array_clone(%phpc.NativeArrayHandle %array)",
        "  %empty_array_truthy = call i1 @phpc_native_value_truthy(%phpc.NativeValueHandle %empty_array)",
        "  %empty_array_false = xor i1 %empty_array_truthy, true",
        "  %wrote = call i1 @phpc_native_array_append_scalar(%phpc.NativeArrayHandle %array, %phpc.NativeScalarValue %zero_scalar)",
        "  %non_empty_array = call %phpc.NativeValueHandle @phpc_native_value_from_array_clone(%phpc.NativeArrayHandle %array)",
        "  %array_truthy = call i1 @phpc_native_value_truthy(%phpc.NativeValueHandle %non_empty_array)",
        "  %strings_truthy = and i1 %string_truthy, %clone_truthy",
        "  %false_values = and i1 %zero_false, %empty_array_false",
        "  %true_values = and i1 %strings_truthy, %array_truthy",
        "  %value_boundaries = and i1 %false_values, %true_values",
        "  %result = and i1 %value_boundaries, %wrote",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %non_empty_array)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %empty_array)",
        "  call void @phpc_native_array_free(%phpc.NativeArrayHandle %array)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %string_clone)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %string_value)",
        "  call void @phpc_native_string_free(%phpc.NativeStringHandle %string)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %zero_value)",
        "  ret i1 %result",
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
        "  %scalar_tag = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %scalar = insertvalue %phpc.NativeScalarValue %scalar_tag, i64 42, 3",
        "  %value = call %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue %scalar)",
        "  %conversion = call %phpc.NativeStringConversionResult @phpc_native_value_to_string_bytes(%phpc.NativeValueHandle %value)",
        "  %bytes = extractvalue %phpc.NativeStringConversionResult %conversion, 0",
        &format!("  %len = extractvalue %phpc.NativeByteBuffer %bytes, 1"),
        "  call void @phpc_native_string_conversion_result_free(%phpc.NativeStringConversionResult %conversion)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %value)",
        &format!("  ret {usize_type} %len"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_value_to_text_conversion_result() {{"),
        "entry:",
        "  %scalar_tag = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %scalar = insertvalue %phpc.NativeScalarValue %scalar_tag, i64 42, 3",
        "  %value = call %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue %scalar)",
        "  %conversion = call %phpc.NativeStringConversionResult @phpc_native_value_text_bytes(%phpc.NativeValueHandle %value, i8 4)",
        "  %bytes = extractvalue %phpc.NativeStringConversionResult %conversion, 0",
        &format!("  %len = extractvalue %phpc.NativeByteBuffer %bytes, 1"),
        "  call void @phpc_native_string_conversion_result_free(%phpc.NativeStringConversionResult %conversion)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %value)",
        &format!("  ret {usize_type} %len"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_value_text_membership() {{"),
        "entry:",
        &format!(
            "  %bytes = getelementptr inbounds [4 x i8], ptr @phpc.probe.bytes, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %value = call %phpc.NativeValueHandle @phpc_native_binary_value_from_bytes(ptr %bytes, {usize_type} 4)"
        ),
        "  %diagnostic_slot = alloca %phpc.NativeDiagnosticHandle",
        "  store %phpc.NativeDiagnosticHandle zeroinitializer, ptr %diagnostic_slot",
        &format!(
            "  %matched = call i1 @phpc_native_value_text_membership_with_diagnostic(%phpc.NativeValueHandle %value, i8 4, ptr @phpc.probe.text_membership_candidates, ptr @phpc.probe.text_membership_candidate_lengths, {usize_type} 1, i1 true, ptr %diagnostic_slot)"
        ),
        "  %diagnostic = load %phpc.NativeDiagnosticHandle, ptr %diagnostic_slot",
        "  call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %diagnostic)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %value)",
        &format!("  %result = zext i1 %matched to {usize_type}"),
        &format!("  ret {usize_type} %result"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_value_string_array_operation() {{"),
        "entry:",
        &format!(
            "  %bytes = getelementptr inbounds [4 x i8], ptr @phpc.probe.binary, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %separator_bytes = getelementptr inbounds [1 x i8], ptr @phpc.probe.invalid, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %subject = call %phpc.NativeValueHandle @phpc_native_binary_value_from_bytes(ptr %bytes, {usize_type} 4)"
        ),
        &format!(
            "  %separator = call %phpc.NativeValueHandle @phpc_native_binary_value_from_bytes(ptr %separator_bytes, {usize_type} 1)"
        ),
        "  %diagnostic_slot = alloca %phpc.NativeDiagnosticHandle",
        "  store %phpc.NativeDiagnosticHandle zeroinitializer, ptr %diagnostic_slot",
        "  %parts = call %phpc.NativeValueHandle @phpc_native_value_string_array_operation_with_diagnostic(%phpc.NativeValueHandle %subject, %phpc.NativeValueHandle %separator, i64 0, i8 0, i8 0, ptr %diagnostic_slot)",
        &format!(
            "  %parts_len = call {usize_type} @phpc_native_value_array_len_with_diagnostic(%phpc.NativeValueHandle %parts, ptr %diagnostic_slot)"
        ),
        "  %diagnostic = load %phpc.NativeDiagnosticHandle, ptr %diagnostic_slot",
        "  call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %diagnostic)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %parts)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %separator)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %subject)",
        &format!("  ret {usize_type} %parts_len"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_array_key_to_value_materialization() {{"),
        "entry:",
        &format!(
            "  %bytes = getelementptr inbounds [4 x i8], ptr @phpc.probe.binary, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %source = call %phpc.NativeValueHandle @phpc_native_binary_value_from_bytes(ptr %bytes, {usize_type} 4)"
        ),
        "  %key = call %phpc.NativeArrayKeyMaterializationResult @phpc_native_value_to_array_key(%phpc.NativeValueHandle %source)",
        "  %diagnostic_slot = alloca %phpc.NativeDiagnosticHandle",
        "  store %phpc.NativeDiagnosticHandle zeroinitializer, ptr %diagnostic_slot",
        "  %key_value = call %phpc.NativeValueHandle @phpc_native_array_key_materialization_to_value_with_diagnostic(%phpc.NativeArrayKeyMaterializationResult %key, ptr %diagnostic_slot)",
        "  %echo = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %key_value)",
        &format!("  %len = extractvalue %phpc.NativeByteBuffer %echo, 1"),
        "  %diagnostic = load %phpc.NativeDiagnosticHandle, ptr %diagnostic_slot",
        "  call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %diagnostic)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %echo)",
        "  call void @phpc_native_array_key_materialization_result_free(%phpc.NativeArrayKeyMaterializationResult %key)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %key_value)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %source)",
        &format!("  ret {usize_type} %len"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_value_array_key_exists() {{"),
        "entry:",
        "  %array = call %phpc.NativeArrayHandle @phpc_native_array_empty()",
        "  %key_tag = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %key_scalar = insertvalue %phpc.NativeScalarValue %key_tag, i64 0, 3",
        "  %stored = call i1 @phpc_native_array_write_int_scalar(%phpc.NativeArrayHandle %array, i64 0, %phpc.NativeScalarValue zeroinitializer)",
        "  %array_value = call %phpc.NativeValueHandle @phpc_native_value_from_array_clone(%phpc.NativeArrayHandle %array)",
        "  %key_value = call %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue %key_scalar)",
        "  %diagnostic_slot = alloca %phpc.NativeDiagnosticHandle",
        "  store %phpc.NativeDiagnosticHandle zeroinitializer, ptr %diagnostic_slot",
        "  %exists = call i1 @phpc_native_value_array_key_exists_with_diagnostic(%phpc.NativeValueHandle %array_value, %phpc.NativeValueHandle %key_value, ptr %diagnostic_slot)",
        "  %result_bool = and i1 %stored, %exists",
        "  %diagnostic = load %phpc.NativeDiagnosticHandle, ptr %diagnostic_slot",
        "  call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %diagnostic)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %key_value)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %array_value)",
        "  call void @phpc_native_array_free(%phpc.NativeArrayHandle %array)",
        &format!("  %result = zext i1 %result_bool to {usize_type}"),
        &format!("  ret {usize_type} %result"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_value_array_operation() {{"),
        "entry:",
        "  %array = call %phpc.NativeArrayHandle @phpc_native_array_empty()",
        "  %scalar_tag = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %scalar = insertvalue %phpc.NativeScalarValue %scalar_tag, i64 9, 3",
        "  %stored = call i1 @phpc_native_array_append_scalar(%phpc.NativeArrayHandle %array, %phpc.NativeScalarValue %scalar)",
        "  %array_value = call %phpc.NativeValueHandle @phpc_native_value_from_array_clone(%phpc.NativeArrayHandle %array)",
        "  %diagnostic_slot = alloca %phpc.NativeDiagnosticHandle",
        "  store %phpc.NativeDiagnosticHandle zeroinitializer, ptr %diagnostic_slot",
        "  %values = call %phpc.NativeValueHandle @phpc_native_value_array_operation_with_diagnostic(%phpc.NativeValueHandle %array_value, i8 0, ptr %diagnostic_slot)",
        "  %is_list = call %phpc.NativeValueHandle @phpc_native_value_array_operation_with_diagnostic(%phpc.NativeValueHandle %array_value, i8 5, ptr %diagnostic_slot)",
        &format!(
            "  %values_len = call {usize_type} @phpc_native_value_array_len_with_diagnostic(%phpc.NativeValueHandle %values, ptr %diagnostic_slot)"
        ),
        "  %diagnostic = load %phpc.NativeDiagnosticHandle, ptr %diagnostic_slot",
        "  call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %diagnostic)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %is_list)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %values)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %array_value)",
        "  call void @phpc_native_array_free(%phpc.NativeArrayHandle %array)",
        &format!("  %stored_value = zext i1 %stored to {usize_type}"),
        &format!("  %result = add {usize_type} %values_len, %stored_value"),
        &format!("  ret {usize_type} %result"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_value_array_query_operation() {{"),
        "entry:",
        "  %array = call %phpc.NativeArrayHandle @phpc_native_array_empty()",
        "  %scalar_tag = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %scalar = insertvalue %phpc.NativeScalarValue %scalar_tag, i64 9, 3",
        "  %stored = call i1 @phpc_native_array_append_scalar(%phpc.NativeArrayHandle %array, %phpc.NativeScalarValue %scalar)",
        "  %array_value = call %phpc.NativeValueHandle @phpc_native_value_from_array_clone(%phpc.NativeArrayHandle %array)",
        "  %needle = call %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue %scalar)",
        "  %diagnostic_slot = alloca %phpc.NativeDiagnosticHandle",
        "  store %phpc.NativeDiagnosticHandle zeroinitializer, ptr %diagnostic_slot",
        "  %contains = call %phpc.NativeValueHandle @phpc_native_value_array_query_operation_with_diagnostic(%phpc.NativeValueHandle %array_value, %phpc.NativeValueHandle %needle, i8 1, i8 1, ptr %diagnostic_slot)",
        "  %key = call %phpc.NativeValueHandle @phpc_native_value_array_query_operation_with_diagnostic(%phpc.NativeValueHandle %array_value, %phpc.NativeValueHandle %needle, i8 1, i8 2, ptr %diagnostic_slot)",
        "  %contains_buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %contains)",
        "  %key_buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %key)",
        "  %contains_len = extractvalue %phpc.NativeByteBuffer %contains_buffer, 1",
        "  %key_len = extractvalue %phpc.NativeByteBuffer %key_buffer, 1",
        "  %diagnostic = load %phpc.NativeDiagnosticHandle, ptr %diagnostic_slot",
        "  call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle %diagnostic)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %key_buffer)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %contains_buffer)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %key)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %contains)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %needle)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %array_value)",
        "  call void @phpc_native_array_free(%phpc.NativeArrayHandle %array)",
        &format!("  %stored_value = zext i1 %stored to {usize_type}"),
        &format!("  %partial = add {usize_type} %contains_len, %key_len"),
        &format!("  %result = add {usize_type} %partial, %stored_value"),
        &format!("  ret {usize_type} %result"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_array_key_text_view_diagnostic() {{"),
        "entry:",
        &format!(
            "  %bytes = getelementptr inbounds [4 x i8], ptr @phpc.probe.binary, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %source = call %phpc.NativeValueHandle @phpc_native_binary_value_from_bytes(ptr %bytes, {usize_type} 4)"
        ),
        "  %key = call %phpc.NativeArrayKeyMaterializationResult @phpc_native_value_to_array_key(%phpc.NativeValueHandle %source)",
        "  %text = call %phpc.NativeStringConversionResult @phpc_native_array_key_materialization_text_bytes(%phpc.NativeArrayKeyMaterializationResult %key, i8 1)",
        "  %diagnostic = extractvalue %phpc.NativeStringConversionResult %text, 1",
        &format!("  %len = call {usize_type} @phpc_native_diagnostic_message_len(%phpc.NativeDiagnosticHandle %diagnostic)"),
        "  call void @phpc_native_string_conversion_result_free(%phpc.NativeStringConversionResult %text)",
        "  call void @phpc_native_array_key_materialization_result_free(%phpc.NativeArrayKeyMaterializationResult %key)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %source)",
        &format!("  ret {usize_type} %len"),
        "}",
        "",
        "define i1 @phpc_probe_container_handle_null_shapes() {",
        "entry:",
        "  %array = call %phpc.NativeArrayHandle @phpc_native_array_null()",
        "  %array_is_null = call i1 @phpc_native_array_is_null(%phpc.NativeArrayHandle %array)",
        "  %class = call %phpc.NativeClassMetadataHandle @phpc_native_class_metadata_null()",
        "  %class_is_null = call i1 @phpc_native_class_metadata_is_null(%phpc.NativeClassMetadataHandle %class)",
        "  %object = call %phpc.NativeObjectHandle @phpc_native_object_null()",
        "  %object_is_null = call i1 @phpc_native_object_is_null(%phpc.NativeObjectHandle %object)",
        "  %resource = call %phpc.NativeResourceHandle @phpc_native_resource_null()",
        "  %resource_is_null = call i1 @phpc_native_resource_is_null(%phpc.NativeResourceHandle %resource)",
        "  %reference = call %phpc.NativeReferenceHandle @phpc_native_reference_null()",
        "  %reference_is_null = call i1 @phpc_native_reference_is_null(%phpc.NativeReferenceHandle %reference)",
        "  %left = and i1 %array_is_null, %class_is_null",
        "  %middle = and i1 %object_is_null, %resource_is_null",
        "  %right = and i1 %resource_is_null, %reference_is_null",
        "  %containers = and i1 %left, %middle",
        "  %all = and i1 %containers, %right",
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
        &format!("define {usize_type} @phpc_probe_array_key_snapshot_order() {{"),
        "entry:",
        "  %array = call %phpc.NativeArrayHandle @phpc_native_array_empty()",
        "  %int_tag = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %int_value = insertvalue %phpc.NativeScalarValue %int_tag, i64 7, 3",
        "  %wrote_int = call i1 @phpc_native_array_write_int_scalar(%phpc.NativeArrayHandle %array, i64 5, %phpc.NativeScalarValue %int_value)",
        &format!(
            "  %bytes = getelementptr inbounds [7 x i8], ptr @phpc.probe.string, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %key = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr %bytes, {usize_type} 7)"
        ),
        "  %wrote_string = call i1 @phpc_native_array_write_string_scalar(%phpc.NativeArrayHandle %array, %phpc.NativeStringHandle %key, %phpc.NativeScalarValue %int_value)",
        "  %snapshot = call %phpc.NativeArrayKeySnapshotHandle @phpc_native_array_key_snapshot(%phpc.NativeArrayHandle %array)",
        "  %snapshot_is_null = call i1 @phpc_native_array_key_snapshot_is_null(%phpc.NativeArrayKeySnapshotHandle %snapshot)",
        &format!(
            "  %snapshot_len = call {usize_type} @phpc_native_array_key_snapshot_len(%phpc.NativeArrayKeySnapshotHandle %snapshot)"
        ),
        &format!(
            "  %first_key = call %phpc.NativeArrayKeyMetadata @phpc_native_array_key_snapshot_key_at(%phpc.NativeArrayKeySnapshotHandle %snapshot, {usize_type} 0)"
        ),
        &format!(
            "  %second_key = call %phpc.NativeArrayKeyMetadata @phpc_native_array_key_snapshot_key_at(%phpc.NativeArrayKeySnapshotHandle %snapshot, {usize_type} 1)"
        ),
        "  %first_tag = extractvalue %phpc.NativeArrayKeyMetadata %first_key, 0",
        "  %second_string_len = extractvalue %phpc.NativeArrayKeyMetadata %second_key, 3",
        &format!(
            "  %key_clone = call %phpc.NativeStringHandle @phpc_native_array_key_snapshot_string_clone_at(%phpc.NativeArrayKeySnapshotHandle %snapshot, {usize_type} 1)"
        ),
        &format!(
            "  %key_clone_len = call {usize_type} @phpc_native_string_len(%phpc.NativeStringHandle %key_clone)"
        ),
        &format!("  %int_flag = zext i1 %wrote_int to {usize_type}"),
        &format!("  %string_flag = zext i1 %wrote_string to {usize_type}"),
        &format!("  %snapshot_flag = zext i1 %snapshot_is_null to {usize_type}"),
        &format!("  %tag_value = zext i8 %first_tag to {usize_type}"),
        &format!("  %write_flags = add {usize_type} %int_flag, %string_flag"),
        &format!("  %key_lens = add {usize_type} %second_string_len, %key_clone_len"),
        &format!("  %shape = add {usize_type} %snapshot_len, %tag_value"),
        &format!("  %flags_and_shape = add {usize_type} %write_flags, %shape"),
        &format!("  %with_keys = add {usize_type} %flags_and_shape, %key_lens"),
        &format!("  %result = sub {usize_type} %with_keys, %snapshot_flag"),
        "  call void @phpc_native_string_free(%phpc.NativeStringHandle %key_clone)",
        "  call void @phpc_native_array_key_snapshot_free(%phpc.NativeArrayKeySnapshotHandle %snapshot)",
        "  call void @phpc_native_string_free(%phpc.NativeStringHandle %key)",
        "  call void @phpc_native_array_free(%phpc.NativeArrayHandle %array)",
        &format!("  ret {usize_type} %result"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_array_entry_snapshot_value_routes() {{"),
        "entry:",
        "  %array = call %phpc.NativeArrayHandle @phpc_native_array_empty()",
        "  %int_tag = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %int_value = insertvalue %phpc.NativeScalarValue %int_tag, i64 11, 3",
        "  %wrote_int = call i1 @phpc_native_array_write_int_scalar(%phpc.NativeArrayHandle %array, i64 2, %phpc.NativeScalarValue %int_value)",
        &format!(
            "  %key_bytes = getelementptr inbounds [5 x i8], ptr @phpc.probe.symbol, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %key = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr %key_bytes, {usize_type} 5)"
        ),
        &format!(
            "  %value_bytes = getelementptr inbounds [7 x i8], ptr @phpc.probe.string, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %value_string = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr %value_bytes, {usize_type} 7)"
        ),
        "  %string_value = call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle %value_string)",
        "  %wrote_string = call i1 @phpc_native_array_write_string_value(%phpc.NativeArrayHandle %array, %phpc.NativeStringHandle %key, %phpc.NativeValueHandle %string_value)",
        "  %snapshot = call %phpc.NativeArrayEntrySnapshotHandle @phpc_native_array_entry_snapshot(%phpc.NativeArrayHandle %array)",
        "  %snapshot_is_null = call i1 @phpc_native_array_entry_snapshot_is_null(%phpc.NativeArrayEntrySnapshotHandle %snapshot)",
        &format!(
            "  %snapshot_len = call {usize_type} @phpc_native_array_entry_snapshot_len(%phpc.NativeArrayEntrySnapshotHandle %snapshot)"
        ),
        &format!(
            "  %first_key = call %phpc.NativeArrayKeyMetadata @phpc_native_array_entry_snapshot_key_at(%phpc.NativeArrayEntrySnapshotHandle %snapshot, {usize_type} 0)"
        ),
        &format!(
            "  %second_key = call %phpc.NativeArrayKeyMetadata @phpc_native_array_entry_snapshot_key_at(%phpc.NativeArrayEntrySnapshotHandle %snapshot, {usize_type} 1)"
        ),
        "  %first_tag = extractvalue %phpc.NativeArrayKeyMetadata %first_key, 0",
        "  %second_string_len = extractvalue %phpc.NativeArrayKeyMetadata %second_key, 3",
        &format!(
            "  %second_key_clone = call %phpc.NativeStringHandle @phpc_native_array_entry_snapshot_string_clone_at(%phpc.NativeArrayEntrySnapshotHandle %snapshot, {usize_type} 1)"
        ),
        &format!(
            "  %second_key_clone_len = call {usize_type} @phpc_native_string_len(%phpc.NativeStringHandle %second_key_clone)"
        ),
        &format!(
            "  %first_key_value = call %phpc.NativeValueHandle @phpc_native_array_entry_snapshot_key_value_clone_at(%phpc.NativeArrayEntrySnapshotHandle %snapshot, {usize_type} 0)"
        ),
        &format!(
            "  %second_key_value = call %phpc.NativeValueHandle @phpc_native_array_entry_snapshot_key_value_clone_at(%phpc.NativeArrayEntrySnapshotHandle %snapshot, {usize_type} 1)"
        ),
        &format!(
            "  %first_value = call %phpc.NativeValueHandle @phpc_native_array_entry_snapshot_value_clone_at(%phpc.NativeArrayEntrySnapshotHandle %snapshot, {usize_type} 0)"
        ),
        &format!(
            "  %second_value = call %phpc.NativeValueHandle @phpc_native_array_entry_snapshot_value_clone_at(%phpc.NativeArrayEntrySnapshotHandle %snapshot, {usize_type} 1)"
        ),
        &format!(
            "  %first_key_reference = call %phpc.NativeReferenceHandle @phpc_native_array_entry_snapshot_key_reference_clone_at(%phpc.NativeArrayEntrySnapshotHandle %snapshot, {usize_type} 0)"
        ),
        &format!(
            "  %second_value_reference = call %phpc.NativeReferenceHandle @phpc_native_array_entry_snapshot_value_reference_clone_at(%phpc.NativeArrayEntrySnapshotHandle %snapshot, {usize_type} 1)"
        ),
        "  %first_key_reference_value = call %phpc.NativeValueHandle @phpc_native_reference_read_value(%phpc.NativeReferenceHandle %first_key_reference)",
        "  %second_value_reference_value = call %phpc.NativeValueHandle @phpc_native_reference_read_value(%phpc.NativeReferenceHandle %second_value_reference)",
        "  %first_key_buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %first_key_value)",
        "  %second_key_buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %second_key_value)",
        "  %first_value_buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %first_value)",
        "  %second_value_buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %second_value)",
        "  %first_key_reference_buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %first_key_reference_value)",
        "  %second_value_reference_buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %second_value_reference_value)",
        "  %first_key_len = extractvalue %phpc.NativeByteBuffer %first_key_buffer, 1",
        "  %second_key_len = extractvalue %phpc.NativeByteBuffer %second_key_buffer, 1",
        "  %first_value_len = extractvalue %phpc.NativeByteBuffer %first_value_buffer, 1",
        "  %second_value_len = extractvalue %phpc.NativeByteBuffer %second_value_buffer, 1",
        "  %first_key_reference_len = extractvalue %phpc.NativeByteBuffer %first_key_reference_buffer, 1",
        "  %second_value_reference_len = extractvalue %phpc.NativeByteBuffer %second_value_reference_buffer, 1",
        &format!("  %int_flag = zext i1 %wrote_int to {usize_type}"),
        &format!("  %string_flag = zext i1 %wrote_string to {usize_type}"),
        &format!("  %snapshot_flag = zext i1 %snapshot_is_null to {usize_type}"),
        &format!("  %first_tag_value = zext i8 %first_tag to {usize_type}"),
        &format!("  %write_flags = add {usize_type} %int_flag, %string_flag"),
        &format!("  %key_shape = add {usize_type} %second_string_len, %second_key_clone_len"),
        &format!("  %key_values = add {usize_type} %first_key_len, %second_key_len"),
        &format!("  %entry_values = add {usize_type} %first_value_len, %second_value_len"),
        &format!(
            "  %reference_values = add {usize_type} %first_key_reference_len, %second_value_reference_len"
        ),
        &format!("  %value_routes = add {usize_type} %key_values, %entry_values"),
        &format!("  %clone_routes = add {usize_type} %value_routes, %reference_values"),
        &format!("  %shape = add {usize_type} %snapshot_len, %first_tag_value"),
        &format!("  %with_keys = add {usize_type} %shape, %key_shape"),
        &format!("  %with_flags = add {usize_type} %with_keys, %write_flags"),
        &format!("  %with_routes = add {usize_type} %with_flags, %clone_routes"),
        &format!("  %result = sub {usize_type} %with_routes, %snapshot_flag"),
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %first_key_buffer)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %second_key_buffer)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %first_value_buffer)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %second_value_buffer)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %first_key_reference_buffer)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %second_value_reference_buffer)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %first_key_reference_value)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %second_value_reference_value)",
        "  call void @phpc_native_reference_free(%phpc.NativeReferenceHandle %first_key_reference)",
        "  call void @phpc_native_reference_free(%phpc.NativeReferenceHandle %second_value_reference)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %first_key_value)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %second_key_value)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %first_value)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %second_value)",
        "  call void @phpc_native_string_free(%phpc.NativeStringHandle %second_key_clone)",
        "  call void @phpc_native_array_entry_snapshot_free(%phpc.NativeArrayEntrySnapshotHandle %snapshot)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %string_value)",
        "  call void @phpc_native_string_free(%phpc.NativeStringHandle %value_string)",
        "  call void @phpc_native_string_free(%phpc.NativeStringHandle %key)",
        "  call void @phpc_native_array_free(%phpc.NativeArrayHandle %array)",
        &format!("  ret {usize_type} %result"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_object_class_metadata_alloc_name() {{"),
        "entry:",
        &format!(
            "  %bytes = getelementptr inbounds [7 x i8], ptr @phpc.probe.string, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %class = call %phpc.NativeClassMetadataHandle @phpc_native_class_metadata_from_name(ptr %bytes, {usize_type} 7)"
        ),
        &format!(
            "  %class_len = call {usize_type} @phpc_native_class_metadata_name_len(%phpc.NativeClassMetadataHandle %class)"
        ),
        "  %class_name = call %phpc.NativeByteBuffer @phpc_native_class_metadata_name_clone_bytes(%phpc.NativeClassMetadataHandle %class)",
        "  %object = call %phpc.NativeObjectHandle @phpc_native_object_alloc(%phpc.NativeClassMetadataHandle %class)",
        &format!(
            "  %object_len = call {usize_type} @phpc_native_object_class_name_len(%phpc.NativeObjectHandle %object)"
        ),
        "  %object_name = call %phpc.NativeByteBuffer @phpc_native_object_class_name_clone_bytes(%phpc.NativeObjectHandle %object)",
        "  %cloned_class = call %phpc.NativeClassMetadataHandle @phpc_native_object_class_metadata_clone(%phpc.NativeObjectHandle %object)",
        &format!(
            "  %cloned_len = call {usize_type} @phpc_native_class_metadata_name_len(%phpc.NativeClassMetadataHandle %cloned_class)"
        ),
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %object_name)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %class_name)",
        "  call void @phpc_native_class_metadata_free(%phpc.NativeClassMetadataHandle %cloned_class)",
        "  call void @phpc_native_object_free(%phpc.NativeObjectHandle %object)",
        "  call void @phpc_native_class_metadata_free(%phpc.NativeClassMetadataHandle %class)",
        &format!("  %combined = add {usize_type} %object_len, %cloned_len"),
        &format!("  %result = sub {usize_type} %combined, %class_len"),
        &format!("  ret {usize_type} %result"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_reference_cell_roundtrip() {{"),
        "entry:",
        "  %int_tag = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %int_value = insertvalue %phpc.NativeScalarValue %int_tag, i64 42, 3",
        "  %reference = call %phpc.NativeReferenceHandle @phpc_native_reference_from_scalar(%phpc.NativeScalarValue %int_value)",
        "  %initial_is_null = call i1 @phpc_native_reference_is_null(%phpc.NativeReferenceHandle %reference)",
        "  %initial_is_empty = call i1 @phpc_native_reference_is_empty(%phpc.NativeReferenceHandle %reference)",
        "  %initial = call %phpc.NativeValueHandle @phpc_native_reference_read_value(%phpc.NativeReferenceHandle %reference)",
        "  %initial_buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %initial)",
        "  %initial_len = extractvalue %phpc.NativeByteBuffer %initial_buffer, 1",
        &format!(
            "  %bytes = getelementptr inbounds [7 x i8], ptr @phpc.probe.string, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %string = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr %bytes, {usize_type} 7)"
        ),
        "  %string_value = call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle %string)",
        "  %value_reference = call %phpc.NativeReferenceHandle @phpc_native_reference_from_value(%phpc.NativeValueHandle %string_value)",
        "  %cloned_reference = call %phpc.NativeReferenceHandle @phpc_native_reference_clone(%phpc.NativeReferenceHandle %value_reference)",
        "  %wrote_reference = call i1 @phpc_native_reference_write_reference(%phpc.NativeReferenceHandle %reference, %phpc.NativeReferenceHandle %cloned_reference)",
        "  %after_reference = call %phpc.NativeValueHandle @phpc_native_reference_read_value(%phpc.NativeReferenceHandle %reference)",
        "  %after_reference_buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %after_reference)",
        "  %after_reference_len = extractvalue %phpc.NativeByteBuffer %after_reference_buffer, 1",
        "  %wrote_string = call i1 @phpc_native_reference_write_value(%phpc.NativeReferenceHandle %reference, %phpc.NativeValueHandle %string_value)",
        "  %after_string_is_null = call i1 @phpc_native_reference_is_null(%phpc.NativeReferenceHandle %reference)",
        "  %after_string_is_empty = call i1 @phpc_native_reference_is_empty(%phpc.NativeReferenceHandle %reference)",
        "  %after_string = call %phpc.NativeValueHandle @phpc_native_reference_read_value(%phpc.NativeReferenceHandle %reference)",
        "  %after_string_buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %after_string)",
        "  %after_string_len = extractvalue %phpc.NativeByteBuffer %after_string_buffer, 1",
        "  %null_value = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 0, 0",
        "  %wrote_null = call i1 @phpc_native_reference_write_scalar(%phpc.NativeReferenceHandle %reference, %phpc.NativeScalarValue %null_value)",
        "  %after_null_is_null = call i1 @phpc_native_reference_is_null(%phpc.NativeReferenceHandle %reference)",
        "  %after_null_is_empty = call i1 @phpc_native_reference_is_empty(%phpc.NativeReferenceHandle %reference)",
        "  %after_null = call %phpc.NativeValueHandle @phpc_native_reference_read_value(%phpc.NativeReferenceHandle %reference)",
        "  %after_null_buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %after_null)",
        "  %after_null_len = extractvalue %phpc.NativeByteBuffer %after_null_buffer, 1",
        "  %cloned_read = call %phpc.NativeValueHandle @phpc_native_reference_read_value(%phpc.NativeReferenceHandle %cloned_reference)",
        "  %cloned_buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %cloned_read)",
        "  %cloned_len = extractvalue %phpc.NativeByteBuffer %cloned_buffer, 1",
        &format!("  %reference_flag = zext i1 %wrote_reference to {usize_type}"),
        &format!("  %string_flag = zext i1 %wrote_string to {usize_type}"),
        &format!("  %null_flag = zext i1 %wrote_null to {usize_type}"),
        &format!("  %initial_null_flag = zext i1 %initial_is_null to {usize_type}"),
        &format!("  %initial_empty_flag = zext i1 %initial_is_empty to {usize_type}"),
        &format!("  %after_string_null_flag = zext i1 %after_string_is_null to {usize_type}"),
        &format!("  %after_string_empty_flag = zext i1 %after_string_is_empty to {usize_type}"),
        &format!("  %after_null_null_flag = zext i1 %after_null_is_null to {usize_type}"),
        &format!("  %after_null_empty_flag = zext i1 %after_null_is_empty to {usize_type}"),
        &format!("  %first_sum = add {usize_type} %initial_len, %after_reference_len"),
        &format!("  %second_sum = add {usize_type} %after_string_len, %after_null_len"),
        &format!("  %third_sum = add {usize_type} %second_sum, %cloned_len"),
        &format!("  %byte_count = add {usize_type} %first_sum, %third_sum"),
        &format!("  %write_flags = add {usize_type} %reference_flag, %string_flag"),
        &format!("  %flag_count = add {usize_type} %write_flags, %null_flag"),
        &format!("  %initial_liveness = add {usize_type} %initial_null_flag, %initial_empty_flag"),
        &format!(
            "  %after_string_liveness = add {usize_type} %after_string_null_flag, %after_string_empty_flag"
        ),
        &format!("  %after_null_liveness = add {usize_type} %after_null_null_flag, %after_null_empty_flag"),
        &format!("  %early_liveness = add {usize_type} %initial_liveness, %after_string_liveness"),
        &format!("  %liveness_count = add {usize_type} %early_liveness, %after_null_liveness"),
        &format!("  %counts = add {usize_type} %byte_count, %flag_count"),
        &format!("  %result = add {usize_type} %counts, %liveness_count"),
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %initial_buffer)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %after_reference_buffer)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %after_string_buffer)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %after_null_buffer)",
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %cloned_buffer)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %initial)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %after_reference)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %after_string)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %after_null)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %cloned_read)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %string_value)",
        "  call void @phpc_native_string_free(%phpc.NativeStringHandle %string)",
        "  call void @phpc_native_reference_free(%phpc.NativeReferenceHandle %cloned_reference)",
        "  call void @phpc_native_reference_free(%phpc.NativeReferenceHandle %value_reference)",
        "  call void @phpc_native_reference_free(%phpc.NativeReferenceHandle %reference)",
        &format!("  ret {usize_type} %result"),
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_reference_string_conversion_diagnostic() {{"),
        "entry:",
        "  %scalar_tag = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %scalar = insertvalue %phpc.NativeScalarValue %scalar_tag, i64 7, 3",
        "  %reference = call %phpc.NativeReferenceHandle @phpc_native_reference_from_scalar(%phpc.NativeScalarValue %scalar)",
        "  %conversion = call %phpc.NativeStringConversionResult @phpc_native_reference_to_string_bytes(%phpc.NativeReferenceHandle %reference)",
        "  %diagnostic = extractvalue %phpc.NativeStringConversionResult %conversion, 1",
        &format!(
            "  %len = call {usize_type} @phpc_native_diagnostic_message_len(%phpc.NativeDiagnosticHandle %diagnostic)"
        ),
        "  call void @phpc_native_string_conversion_result_free(%phpc.NativeStringConversionResult %conversion)",
        "  call void @phpc_native_reference_free(%phpc.NativeReferenceHandle %reference)",
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
        "define i1 @phpc_probe_symbol_table_null_shape() {",
        "entry:",
        "  %symbols = call %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_null()",
        "  %symbols_is_null = call i1 @phpc_native_symbol_table_is_null(%phpc.NativeSymbolTableHandle %symbols)",
        "  ret i1 %symbols_is_null",
        "}",
        "",
        &format!("define {usize_type} @phpc_probe_symbol_table_write_read() {{"),
        "entry:",
        "  %symbols = call %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()",
        "  %symbols_is_null = call i1 @phpc_native_symbol_table_is_null(%phpc.NativeSymbolTableHandle %symbols)",
        "  %symbols_ready = xor i1 %symbols_is_null, true",
        &format!(
            "  %name = getelementptr inbounds [5 x i8], ptr @phpc.probe.symbol, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %bytes = getelementptr inbounds [7 x i8], ptr @phpc.probe.string, {usize_type} 0, {usize_type} 0"
        ),
        &format!(
            "  %string = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr %bytes, {usize_type} 7)"
        ),
        "  %value = call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle %string)",
        &format!(
            "  %written = call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle %symbols, ptr %name, {usize_type} 5, %phpc.NativeValueHandle %value)"
        ),
        &format!(
            "  %read = call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle %symbols, ptr %name, {usize_type} 5)"
        ),
        "  %buffer = call %phpc.NativeByteBuffer @phpc_native_value_echo_bytes(%phpc.NativeValueHandle %read)",
        "  %len = extractvalue %phpc.NativeByteBuffer %buffer, 1",
        &format!("  %ready_len = zext i1 %symbols_ready to {usize_type}"),
        &format!("  %written_len = zext i1 %written to {usize_type}"),
        &format!("  %partial = add {usize_type} %len, %ready_len"),
        &format!("  %result = add {usize_type} %partial, %written_len"),
        "  call void @phpc_native_byte_buffer_free(%phpc.NativeByteBuffer %buffer)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %read)",
        "  call void @phpc_native_value_free(%phpc.NativeValueHandle %value)",
        "  call void @phpc_native_string_free(%phpc.NativeStringHandle %string)",
        "  call void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle %symbols)",
        &format!("  ret {usize_type} %result"),
        "}",
        "",
    ]
    .join("\n")
}

#[derive(Default)]
struct LlvmGenerator {
    strings: Vec<(String, String)>,
    native_globals: Vec<String>,
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
    uses_native_value_from_string: bool,
    uses_native_value_from_scalar: bool,
    uses_native_value_text_membership: bool,
    uses_native_symbol_table_helpers: bool,
    emitted_native_symbol_table: bool,
}

#[derive(Debug, Clone)]
enum IrValue {
    Int(String),
    Float(String),
    String(String),
    StringPtr(String),
    NativeExpression {
        value: NativeExpressionValue,
        fallback: Box<IrValue>,
    },
    Bool(bool),
    BoolExpr(String),
    Null,
}

#[derive(Debug, Clone)]
enum NativeExpressionValue {
    DirectLocalSymbol { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeValueHandleOwnership {
    Owned,
    #[allow(dead_code)]
    Borrowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeValueBlocker {
    RequestState,
    Array,
    ArrayAccess,
    ObjectProperty,
    ObjectInstantiation,
    ObjectClone,
    MethodCall,
    InstanceOf,
    Resource,
    Reference,
    DynamicCall,
    CopyOnWrite,
    Unsupported,
}

impl NativeValueBlocker {
    fn rejection(self) -> &'static str {
        match self {
            Self::RequestState => LLVM_REQUEST_SUPERGLOBAL_REJECTION,
            Self::Array => LLVM_ARRAY_REJECTION,
            Self::ArrayAccess => LLVM_ARRAY_ACCESS_REJECTION,
            Self::ObjectProperty => LLVM_OBJECT_PROPERTY_REJECTION,
            Self::ObjectInstantiation => LLVM_OBJECT_INSTANTIATION_REJECTION,
            Self::ObjectClone => LLVM_CLONE_REJECTION,
            Self::MethodCall => LLVM_METHOD_CALL_REJECTION,
            Self::InstanceOf => LLVM_INSTANCEOF_REJECTION,
            Self::Resource => LLVM_STREAM_RESOURCE_REJECTION,
            Self::Reference => LLVM_REFERENCE_ASSIGNMENT_REJECTION,
            Self::DynamicCall => LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
            Self::CopyOnWrite => LLVM_MUTATION_REJECTION,
            Self::Unsupported => LLVM_FUNCTION_CALL_REJECTION,
        }
    }
}

#[derive(Debug)]
enum NativeValueHandleResult {
    Available(NativeValueMaterialization),
    Blocked(NativeValueBlocker),
}

impl NativeValueHandleResult {
    fn into_available(self) -> Option<NativeValueMaterialization> {
        match self {
            Self::Available(value) => Some(value),
            Self::Blocked(blocker) => {
                let _ = blocker.rejection();
                None
            }
        }
    }

    fn into_result(self) -> Result<NativeValueMaterialization, NativeValueBlocker> {
        match self {
            Self::Available(value) => Ok(value),
            Self::Blocked(blocker) => Err(blocker),
        }
    }
}

#[derive(Debug)]
struct NativeValueMaterialization {
    handle: String,
    ownership: NativeValueHandleOwnership,
    cleanup: Vec<String>,
}

#[derive(Debug)]
struct NativeCallArgumentMaterialization {
    handles: Vec<String>,
    cleanup: Vec<String>,
}

enum NativeSelectionBranch<'a> {
    Value(IrValue),
    Expr(&'a Expr),
}

#[derive(Debug, Clone, Copy)]
enum NativeTextSurface {
    FunctionName,
    ExtensionName,
}

impl NativeTextSurface {
    fn surface_tag(self) -> u8 {
        match self {
            Self::FunctionName => 4,
            Self::ExtensionName => 6,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeControlFlowEffect {
    Continues,
    Terminates,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeBranchEffectJoin {
    BothContinue,
    BothTerminate,
    ThenContinues,
    ElseContinues,
}

#[allow(dead_code)]
impl NativeBranchEffectJoin {
    fn from_flows(then_flow: NativeControlFlowEffect, else_flow: NativeControlFlowEffect) -> Self {
        match (then_flow, else_flow) {
            (NativeControlFlowEffect::Continues, NativeControlFlowEffect::Continues) => {
                Self::BothContinue
            }
            (NativeControlFlowEffect::Terminates, NativeControlFlowEffect::Terminates) => {
                Self::BothTerminate
            }
            (NativeControlFlowEffect::Continues, NativeControlFlowEffect::Terminates) => {
                Self::ThenContinues
            }
            (NativeControlFlowEffect::Terminates, NativeControlFlowEffect::Continues) => {
                Self::ElseContinues
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeBranchLocalCleanupPlan {
    control_join: NativeBranchEffectJoin,
    entry_live_locals: Vec<String>,
    then_live_locals: Vec<String>,
    else_live_locals: Vec<String>,
    stable_live_locals: Vec<String>,
    divergent_live_locals: Vec<String>,
    then_only_locals: Vec<String>,
    else_only_locals: Vec<String>,
}

#[allow(dead_code)]
impl NativeBranchLocalCleanupPlan {
    fn from_states<Value: PartialEq>(
        control_join: NativeBranchEffectJoin,
        entry: &HashMap<String, Value>,
        then_state: &HashMap<String, Value>,
        else_state: &HashMap<String, Value>,
    ) -> Self {
        let entry_live_locals = sorted_local_names(entry);
        let then_live_locals = sorted_local_names(then_state);
        let else_live_locals = sorted_local_names(else_state);

        let mut stable_live_locals = Vec::new();
        let mut divergent_live_locals = Vec::new();
        let mut then_only_locals = Vec::new();
        let mut else_only_locals = Vec::new();

        for name in local_name_union([entry, then_state, else_state]) {
            match (then_state.get(&name), else_state.get(&name)) {
                (Some(then_value), Some(else_value)) if then_value == else_value => {
                    stable_live_locals.push(name);
                }
                (Some(_), Some(_)) => divergent_live_locals.push(name),
                (Some(_), None) => {
                    then_only_locals.push(name.clone());
                    divergent_live_locals.push(name);
                }
                (None, Some(_)) => {
                    else_only_locals.push(name.clone());
                    divergent_live_locals.push(name);
                }
                (None, None) => {}
            }
        }

        Self {
            control_join,
            entry_live_locals,
            then_live_locals,
            else_live_locals,
            stable_live_locals,
            divergent_live_locals,
            then_only_locals,
            else_only_locals,
        }
    }

    fn control_join(&self) -> NativeBranchEffectJoin {
        self.control_join
    }

    fn continuing_arm(&self) -> Option<NativeContinuingBranchArm> {
        match self.control_join {
            NativeBranchEffectJoin::ThenContinues => Some(NativeContinuingBranchArm::Then),
            NativeBranchEffectJoin::ElseContinues => Some(NativeContinuingBranchArm::Else),
            NativeBranchEffectJoin::BothContinue | NativeBranchEffectJoin::BothTerminate => None,
        }
    }

    fn has_stable_local_merge(&self) -> bool {
        self.control_join == NativeBranchEffectJoin::BothContinue
            && self.divergent_live_locals.is_empty()
            && self.then_live_locals == self.stable_live_locals
            && self.else_live_locals == self.stable_live_locals
    }

    fn has_local_phi_merge_ownership(&self) -> bool {
        self.control_join == NativeBranchEffectJoin::BothContinue
            && self.then_only_locals.is_empty()
            && self.else_only_locals.is_empty()
    }

    fn locals_requiring_phi(&self) -> &[String] {
        &self.divergent_live_locals
    }
}

fn sorted_local_names<Value>(locals: &HashMap<String, Value>) -> Vec<String> {
    let mut names = locals.keys().cloned().collect::<Vec<_>>();
    names.sort();
    names
}

fn local_name_union<'a, Value: 'a>(
    locals: impl IntoIterator<Item = &'a HashMap<String, Value>>,
) -> Vec<String> {
    let mut names = Vec::new();
    for local_map in locals {
        names.extend(local_map.keys().cloned());
    }
    names.sort();
    names.dedup();
    names
}

fn sorted_unique_native_value_handles(handles: &[String]) -> Vec<String> {
    let mut handles = handles.to_vec();
    handles.sort();
    handles.dedup();
    handles
}

fn native_value_handle_set(handles: &[String]) -> HashSet<String> {
    handles.iter().cloned().collect()
}

fn native_value_handle_union<'a>(handles: impl IntoIterator<Item = &'a [String]>) -> Vec<String> {
    let mut names = Vec::new();
    for handle_set in handles {
        names.extend(handle_set.iter().cloned());
    }
    names.sort();
    names.dedup();
    names
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeBranchValueFactCleanupPlan {
    control_join: NativeBranchEffectJoin,
    entry_live_facts: Vec<String>,
    then_live_facts: Vec<String>,
    else_live_facts: Vec<String>,
    stable_live_facts: Vec<String>,
    divergent_live_facts: Vec<String>,
    then_only_facts: Vec<String>,
    else_only_facts: Vec<String>,
}

#[allow(dead_code)]
impl NativeBranchValueFactCleanupPlan {
    fn from_states(
        control_join: NativeBranchEffectJoin,
        entry: &HashMap<String, String>,
        then_state: &HashMap<String, String>,
        else_state: &HashMap<String, String>,
    ) -> Self {
        let entry_live_facts = sorted_local_names(entry);
        let then_live_facts = sorted_local_names(then_state);
        let else_live_facts = sorted_local_names(else_state);

        let mut stable_live_facts = Vec::new();
        let mut divergent_live_facts = Vec::new();
        let mut then_only_facts = Vec::new();
        let mut else_only_facts = Vec::new();

        for name in local_name_union([entry, then_state, else_state]) {
            match (then_state.get(&name), else_state.get(&name)) {
                (Some(then_value), Some(else_value)) if then_value == else_value => {
                    stable_live_facts.push(name);
                }
                (Some(_), Some(_)) => divergent_live_facts.push(name),
                (Some(_), None) => {
                    then_only_facts.push(name.clone());
                    divergent_live_facts.push(name);
                }
                (None, Some(_)) => {
                    else_only_facts.push(name.clone());
                    divergent_live_facts.push(name);
                }
                (None, None) => {}
            }
        }

        Self {
            control_join,
            entry_live_facts,
            then_live_facts,
            else_live_facts,
            stable_live_facts,
            divergent_live_facts,
            then_only_facts,
            else_only_facts,
        }
    }

    fn control_join(&self) -> NativeBranchEffectJoin {
        self.control_join
    }

    fn has_live_facts(&self) -> bool {
        !self.entry_live_facts.is_empty()
            || !self.then_live_facts.is_empty()
            || !self.else_live_facts.is_empty()
    }

    fn has_stable_value_fact_merge(&self) -> bool {
        self.control_join == NativeBranchEffectJoin::BothContinue
            && self.divergent_live_facts.is_empty()
            && self.then_live_facts == self.stable_live_facts
            && self.else_live_facts == self.stable_live_facts
    }

    fn merge_ownership(
        &self,
        local_cleanup_plan: &NativeBranchLocalCleanupPlan,
    ) -> NativeBranchValueFactOwnership {
        if self.has_stable_value_fact_merge() {
            return NativeBranchValueFactOwnership::Stable;
        }

        if self.control_join != NativeBranchEffectJoin::BothContinue
            || !self.then_only_facts.is_empty()
            || !self.else_only_facts.is_empty()
            || !local_cleanup_plan.has_local_phi_merge_ownership()
        {
            return NativeBranchValueFactOwnership::Blocked;
        }

        if self.divergent_live_facts.iter().all(|fact| {
            fact.split_once(':').is_some_and(|(local, _)| {
                local_cleanup_plan
                    .locals_requiring_phi()
                    .iter()
                    .any(|name| name == local)
            })
        }) {
            NativeBranchValueFactOwnership::LocalPhi
        } else {
            NativeBranchValueFactOwnership::Blocked
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeBranchLiveNativeValueCleanupPlan {
    control_join: NativeBranchEffectJoin,
    entry_live_handles: Vec<String>,
    then_live_handles: Vec<String>,
    else_live_handles: Vec<String>,
    stable_live_handles: Vec<String>,
    divergent_live_handles: Vec<String>,
    then_only_handles: Vec<String>,
    else_only_handles: Vec<String>,
}

#[allow(dead_code)]
impl NativeBranchLiveNativeValueCleanupPlan {
    fn from_handles(
        control_join: NativeBranchEffectJoin,
        entry: &[String],
        then_state: &[String],
        else_state: &[String],
    ) -> Self {
        let entry_live_handles = sorted_unique_native_value_handles(entry);
        let then_live_handles = sorted_unique_native_value_handles(then_state);
        let else_live_handles = sorted_unique_native_value_handles(else_state);
        let then_live = native_value_handle_set(then_state);
        let else_live = native_value_handle_set(else_state);

        let mut stable_live_handles = Vec::new();
        let mut divergent_live_handles = Vec::new();
        let mut then_only_handles = Vec::new();
        let mut else_only_handles = Vec::new();

        for handle in native_value_handle_union([entry, then_state, else_state]) {
            match (then_live.contains(&handle), else_live.contains(&handle)) {
                (true, true) => stable_live_handles.push(handle),
                (true, false) => {
                    then_only_handles.push(handle.clone());
                    divergent_live_handles.push(handle);
                }
                (false, true) => {
                    else_only_handles.push(handle.clone());
                    divergent_live_handles.push(handle);
                }
                (false, false) => {}
            }
        }

        Self {
            control_join,
            entry_live_handles,
            then_live_handles,
            else_live_handles,
            stable_live_handles,
            divergent_live_handles,
            then_only_handles,
            else_only_handles,
        }
    }

    fn control_join(&self) -> NativeBranchEffectJoin {
        self.control_join
    }

    fn has_live_handles(&self) -> bool {
        !self.entry_live_handles.is_empty()
            || !self.then_live_handles.is_empty()
            || !self.else_live_handles.is_empty()
    }

    fn has_stable_live_handle_merge(&self) -> bool {
        self.control_join == NativeBranchEffectJoin::BothContinue
            && self.divergent_live_handles.is_empty()
            && self.then_live_handles == self.entry_live_handles
            && self.else_live_handles == self.entry_live_handles
    }

    fn merge_ownership(&self) -> NativeBranchLiveNativeValueOwnership {
        if !self.has_live_handles() {
            NativeBranchLiveNativeValueOwnership::NoLiveHandles
        } else if self.has_stable_live_handle_merge() {
            NativeBranchLiveNativeValueOwnership::Stable
        } else {
            NativeBranchLiveNativeValueOwnership::Blocked
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeBranchValueFactOwnership {
    NoLiveFacts,
    Stable,
    LocalPhi,
    Blocked,
}

#[allow(dead_code)]
impl NativeBranchValueFactOwnership {
    fn can_merge_without_phi(self) -> bool {
        matches!(self, Self::NoLiveFacts | Self::Stable)
    }

    fn can_merge_with_local_phi(self) -> bool {
        matches!(self, Self::NoLiveFacts | Self::Stable | Self::LocalPhi)
    }
}

fn branch_value_fact_ownership(
    value_fact_plan: Option<&NativeBranchValueFactCleanupPlan>,
    local_cleanup_plan: &NativeBranchLocalCleanupPlan,
) -> NativeBranchValueFactOwnership {
    value_fact_plan.map_or(NativeBranchValueFactOwnership::NoLiveFacts, |plan| {
        plan.merge_ownership(local_cleanup_plan)
    })
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeBranchLiveNativeValueOwnership {
    NoLiveHandles,
    Stable,
    Blocked,
}

#[allow(dead_code)]
impl NativeBranchLiveNativeValueOwnership {
    fn can_merge_without_phi(self) -> bool {
        matches!(self, Self::NoLiveHandles | Self::Stable)
    }

    fn allows_non_joining_control_flow(self) -> bool {
        matches!(self, Self::NoLiveHandles)
    }
}

fn branch_live_native_value_ownership(
    live_native_value_plan: Option<&NativeBranchLiveNativeValueCleanupPlan>,
) -> NativeBranchLiveNativeValueOwnership {
    live_native_value_plan.map_or(
        NativeBranchLiveNativeValueOwnership::NoLiveHandles,
        NativeBranchLiveNativeValueCleanupPlan::merge_ownership,
    )
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum NativeTerminationCleanupAction<Handle> {
    LiveNativeValueHandle(Handle),
    BranchBlockLocals(NativeBranchLocalCleanupPlan),
    BranchLiveNativeValues(NativeBranchLiveNativeValueCleanupPlan),
    BranchValueFacts(NativeBranchValueFactCleanupPlan),
    DiscardedNativeTemporaries,
    GotoScope,
    LoopScope,
    SwitchScope,
    FunctionFrame,
    ReturnContext,
    FinallyDispatch,
    ExceptionUnwind,
    ShutdownQueue,
    DestructorQueue,
    OutputBufferStack,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeTerminationCleanupStack<Handle> {
    actions: Vec<NativeTerminationCleanupAction<Handle>>,
}

#[allow(dead_code)]
struct NativeTerminationCleanupExecutionPlan<'a, Handle> {
    live_native_value_handles: Vec<&'a Handle>,
}

#[allow(dead_code)]
impl<'a, Handle> NativeTerminationCleanupExecutionPlan<'a, Handle> {
    fn live_native_value_handles(&self) -> &[&'a Handle] {
        &self.live_native_value_handles
    }
}

#[allow(dead_code)]
enum NativeTerminationCleanupStackPlan<'a, Handle> {
    Executable(NativeTerminationCleanupExecutionPlan<'a, Handle>),
    Blocked(NativeTerminationCleanupBoundaryKind),
}

#[allow(dead_code)]
struct NativeTerminationReturnExecutionPlan<'a, Handle> {
    status_value: Option<&'a Handle>,
    cleanup_plan: NativeTerminationCleanupExecutionPlan<'a, Handle>,
}

#[allow(dead_code)]
impl<'a, Handle> NativeTerminationReturnExecutionPlan<'a, Handle> {
    fn status_value(&self) -> Option<&'a Handle> {
        self.status_value
    }

    fn live_native_value_handles(&self) -> &[&'a Handle] {
        self.cleanup_plan.live_native_value_handles()
    }
}

#[allow(dead_code)]
enum NativeTerminationReturnPlan<'a, Handle> {
    Executable(NativeTerminationReturnExecutionPlan<'a, Handle>),
    Blocked(NativeTerminationCleanupBoundaryKind),
}

#[allow(dead_code)]
impl<Handle> NativeTerminationCleanupStack<Handle> {
    fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    fn from_action(action: NativeTerminationCleanupAction<Handle>) -> Self {
        Self::from_actions(vec![action])
    }

    fn from_actions(actions: Vec<NativeTerminationCleanupAction<Handle>>) -> Self {
        Self { actions }
    }

    fn push_action(&mut self, action: NativeTerminationCleanupAction<Handle>) {
        self.actions.push(action);
    }

    fn append_stack(&mut self, mut cleanup_stack: NativeTerminationCleanupStack<Handle>) {
        self.actions.append(&mut cleanup_stack.actions);
    }

    fn prepend_stack(&mut self, mut cleanup_stack: NativeTerminationCleanupStack<Handle>) {
        cleanup_stack.actions.append(&mut self.actions);
        self.actions = cleanup_stack.actions;
    }

    fn from_branch_cleanup_plans(
        local_cleanup_plan: NativeBranchLocalCleanupPlan,
        value_fact_cleanup_plan: NativeBranchValueFactCleanupPlan,
    ) -> Self {
        let live_native_value_cleanup_plan = NativeBranchLiveNativeValueCleanupPlan::from_handles(
            local_cleanup_plan.control_join,
            &[],
            &[],
            &[],
        );
        Self::from_branch_cleanup_plans_with_live_native_values(
            local_cleanup_plan,
            live_native_value_cleanup_plan,
            value_fact_cleanup_plan,
        )
    }

    fn from_branch_cleanup_plans_with_live_native_values(
        local_cleanup_plan: NativeBranchLocalCleanupPlan,
        live_native_value_cleanup_plan: NativeBranchLiveNativeValueCleanupPlan,
        value_fact_cleanup_plan: NativeBranchValueFactCleanupPlan,
    ) -> Self {
        let mut actions = vec![
            NativeTerminationCleanupAction::DiscardedNativeTemporaries,
            NativeTerminationCleanupAction::BranchBlockLocals(local_cleanup_plan),
        ];
        if live_native_value_cleanup_plan.has_live_handles() {
            actions.push(NativeTerminationCleanupAction::BranchLiveNativeValues(
                live_native_value_cleanup_plan,
            ));
        }
        if value_fact_cleanup_plan.has_live_facts() {
            actions.push(NativeTerminationCleanupAction::BranchValueFacts(
                value_fact_cleanup_plan,
            ));
        }
        Self { actions }
    }

    fn push_live_native_value_handle(&mut self, handle: Handle) {
        self.push_action(NativeTerminationCleanupAction::LiveNativeValueHandle(
            handle,
        ));
    }

    fn actions(&self) -> &[NativeTerminationCleanupAction<Handle>] {
        &self.actions
    }

    fn branch_local_cleanup_plan(&self) -> Option<&NativeBranchLocalCleanupPlan> {
        self.actions.iter().find_map(|action| match action {
            NativeTerminationCleanupAction::BranchBlockLocals(cleanup_plan) => Some(cleanup_plan),
            _ => None,
        })
    }

    fn branch_value_fact_cleanup_plan(&self) -> Option<&NativeBranchValueFactCleanupPlan> {
        self.actions.iter().find_map(|action| match action {
            NativeTerminationCleanupAction::BranchValueFacts(cleanup_plan) => Some(cleanup_plan),
            _ => None,
        })
    }

    fn branch_live_native_value_cleanup_plan(
        &self,
    ) -> Option<&NativeBranchLiveNativeValueCleanupPlan> {
        self.actions.iter().find_map(|action| match action {
            NativeTerminationCleanupAction::BranchLiveNativeValues(cleanup_plan) => {
                Some(cleanup_plan)
            }
            _ => None,
        })
    }

    fn runtime_execution_plan(&self) -> NativeTerminationCleanupStackPlan<'_, Handle> {
        let mut live_native_value_handles = Vec::new();
        for (index, action) in self.actions.iter().enumerate() {
            match action {
                NativeTerminationCleanupAction::LiveNativeValueHandle(handle) => {
                    live_native_value_handles.push(handle);
                }
                NativeTerminationCleanupAction::BranchBlockLocals(_)
                | NativeTerminationCleanupAction::BranchLiveNativeValues(_)
                | NativeTerminationCleanupAction::BranchValueFacts(_) => {
                    return NativeTerminationCleanupStackPlan::Blocked(
                        NativeTerminationCleanupBoundaryKind::BranchMerge,
                    );
                }
                NativeTerminationCleanupAction::DiscardedNativeTemporaries => {
                    if self.actions[index + 1..].iter().any(|action| {
                        matches!(
                            action,
                            NativeTerminationCleanupAction::BranchBlockLocals(_)
                                | NativeTerminationCleanupAction::BranchLiveNativeValues(_)
                                | NativeTerminationCleanupAction::BranchValueFacts(_)
                        )
                    }) {
                        return NativeTerminationCleanupStackPlan::Blocked(
                            NativeTerminationCleanupBoundaryKind::BranchMerge,
                        );
                    }
                    return NativeTerminationCleanupStackPlan::Blocked(
                        NativeTerminationCleanupBoundaryKind::Hook(
                            NativeTerminationHookBoundary::ExpressionContext,
                        ),
                    );
                }
                NativeTerminationCleanupAction::GotoScope => {
                    return NativeTerminationCleanupStackPlan::Blocked(
                        NativeTerminationCleanupBoundaryKind::Hook(
                            NativeTerminationHookBoundary::GotoScope,
                        ),
                    );
                }
                NativeTerminationCleanupAction::LoopScope => {
                    return NativeTerminationCleanupStackPlan::Blocked(
                        NativeTerminationCleanupBoundaryKind::Hook(
                            NativeTerminationHookBoundary::LoopScope,
                        ),
                    );
                }
                NativeTerminationCleanupAction::SwitchScope => {
                    return NativeTerminationCleanupStackPlan::Blocked(
                        NativeTerminationCleanupBoundaryKind::Hook(
                            NativeTerminationHookBoundary::SwitchScope,
                        ),
                    );
                }
                NativeTerminationCleanupAction::FunctionFrame => {
                    return NativeTerminationCleanupStackPlan::Blocked(
                        NativeTerminationCleanupBoundaryKind::Hook(
                            NativeTerminationHookBoundary::FunctionFrame,
                        ),
                    );
                }
                NativeTerminationCleanupAction::ReturnContext => {
                    return NativeTerminationCleanupStackPlan::Blocked(
                        NativeTerminationCleanupBoundaryKind::Hook(
                            NativeTerminationHookBoundary::ReturnContext,
                        ),
                    );
                }
                NativeTerminationCleanupAction::FinallyDispatch => {
                    return NativeTerminationCleanupStackPlan::Blocked(
                        NativeTerminationCleanupBoundaryKind::Hook(
                            NativeTerminationHookBoundary::TryFinally,
                        ),
                    );
                }
                NativeTerminationCleanupAction::ExceptionUnwind => {
                    return NativeTerminationCleanupStackPlan::Blocked(
                        NativeTerminationCleanupBoundaryKind::Hook(
                            NativeTerminationHookBoundary::Exception,
                        ),
                    );
                }
                NativeTerminationCleanupAction::ShutdownQueue => {
                    return NativeTerminationCleanupStackPlan::Blocked(
                        NativeTerminationCleanupBoundaryKind::Hook(
                            NativeTerminationHookBoundary::Shutdown,
                        ),
                    );
                }
                NativeTerminationCleanupAction::DestructorQueue => {
                    return NativeTerminationCleanupStackPlan::Blocked(
                        NativeTerminationCleanupBoundaryKind::Hook(
                            NativeTerminationHookBoundary::Destructor,
                        ),
                    );
                }
                NativeTerminationCleanupAction::OutputBufferStack => {
                    return NativeTerminationCleanupStackPlan::Blocked(
                        NativeTerminationCleanupBoundaryKind::Hook(
                            NativeTerminationHookBoundary::OutputBuffer,
                        ),
                    );
                }
            }
        }
        NativeTerminationCleanupStackPlan::Executable(NativeTerminationCleanupExecutionPlan {
            live_native_value_handles,
        })
    }

    #[cfg(test)]
    fn unlowered_runtime_cleanup_boundary_kind(
        &self,
    ) -> Option<NativeTerminationCleanupBoundaryKind> {
        match self.runtime_execution_plan() {
            NativeTerminationCleanupStackPlan::Executable(_) => None,
            NativeTerminationCleanupStackPlan::Blocked(boundary) => Some(boundary),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeContinuingBranchArm {
    Then,
    Else,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeBranchTerminationEffect {
    join: NativeBranchEffectJoin,
    continuing_arm: Option<NativeContinuingBranchArm>,
    cleanup_stack: NativeTerminationCleanupStack<()>,
}

#[allow(dead_code)]
impl NativeBranchTerminationEffect {
    fn from_states<Value: PartialEq>(
        entry: &HashMap<String, Value>,
        then_state: &HashMap<String, Value>,
        else_state: &HashMap<String, Value>,
        then_flow: NativeControlFlowEffect,
        else_flow: NativeControlFlowEffect,
    ) -> Self {
        let empty_facts = HashMap::new();
        Self::from_merge_inputs(
            entry,
            then_state,
            else_state,
            &empty_facts,
            &empty_facts,
            &empty_facts,
            then_flow,
            else_flow,
        )
    }

    fn from_merge_inputs<Value: PartialEq>(
        entry: &HashMap<String, Value>,
        then_state: &HashMap<String, Value>,
        else_state: &HashMap<String, Value>,
        entry_value_facts: &HashMap<String, String>,
        then_value_facts: &HashMap<String, String>,
        else_value_facts: &HashMap<String, String>,
        then_flow: NativeControlFlowEffect,
        else_flow: NativeControlFlowEffect,
    ) -> Self {
        Self::from_merge_inputs_with_live_native_values(
            entry,
            then_state,
            else_state,
            entry_value_facts,
            then_value_facts,
            else_value_facts,
            &[],
            &[],
            &[],
            then_flow,
            else_flow,
        )
    }

    fn from_merge_inputs_with_live_native_values<Value: PartialEq>(
        entry: &HashMap<String, Value>,
        then_state: &HashMap<String, Value>,
        else_state: &HashMap<String, Value>,
        entry_value_facts: &HashMap<String, String>,
        then_value_facts: &HashMap<String, String>,
        else_value_facts: &HashMap<String, String>,
        entry_live_native_value_handles: &[String],
        then_live_native_value_handles: &[String],
        else_live_native_value_handles: &[String],
        then_flow: NativeControlFlowEffect,
        else_flow: NativeControlFlowEffect,
    ) -> Self {
        let join = NativeBranchEffectJoin::from_flows(then_flow, else_flow);
        let local_cleanup_plan =
            NativeBranchLocalCleanupPlan::from_states(join, entry, then_state, else_state);
        let cleanup_stack =
            NativeTerminationCleanupStack::from_branch_cleanup_plans_with_live_native_values(
                local_cleanup_plan.clone(),
                NativeBranchLiveNativeValueCleanupPlan::from_handles(
                    join,
                    entry_live_native_value_handles,
                    then_live_native_value_handles,
                    else_live_native_value_handles,
                ),
                NativeBranchValueFactCleanupPlan::from_states(
                    join,
                    entry_value_facts,
                    then_value_facts,
                    else_value_facts,
                ),
            );
        let continuing_arm = local_cleanup_plan.continuing_arm();

        Self {
            join,
            continuing_arm,
            cleanup_stack,
        }
    }

    fn join(&self) -> NativeBranchEffectJoin {
        self.join
    }

    fn has_stable_local_merge(&self) -> bool {
        self.cleanup_stack
            .branch_local_cleanup_plan()
            .is_some_and(NativeBranchLocalCleanupPlan::has_stable_local_merge)
    }

    fn has_stable_value_fact_merge(&self) -> bool {
        self.value_fact_ownership().can_merge_without_phi()
    }

    fn has_stable_live_native_value_merge(&self) -> bool {
        self.live_native_value_ownership().can_merge_without_phi()
    }

    fn has_stable_control_merge(&self) -> bool {
        self.has_stable_local_merge()
            && self.has_stable_value_fact_merge()
            && self.has_stable_live_native_value_merge()
    }

    fn has_local_phi_merge_ownership(&self) -> bool {
        self.cleanup_stack
            .branch_local_cleanup_plan()
            .is_some_and(NativeBranchLocalCleanupPlan::has_local_phi_merge_ownership)
    }

    fn has_value_fact_merge_for_local_phi(&self) -> bool {
        self.value_fact_ownership().can_merge_with_local_phi()
    }

    fn value_fact_ownership(&self) -> NativeBranchValueFactOwnership {
        let Some(local_plan) = self.cleanup_stack.branch_local_cleanup_plan() else {
            return NativeBranchValueFactOwnership::Blocked;
        };
        branch_value_fact_ownership(
            self.cleanup_stack.branch_value_fact_cleanup_plan(),
            local_plan,
        )
    }

    fn live_native_value_ownership(&self) -> NativeBranchLiveNativeValueOwnership {
        branch_live_native_value_ownership(
            self.cleanup_stack.branch_live_native_value_cleanup_plan(),
        )
    }

    fn continuing_arm(&self) -> Option<NativeContinuingBranchArm> {
        self.continuing_arm
    }

    fn cleanup_stack(&self) -> &NativeTerminationCleanupStack<()> {
        &self.cleanup_stack
    }

    fn into_cleanup_stack(self) -> NativeTerminationCleanupStack<()> {
        self.cleanup_stack
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeTerminationCleanupBlocker {
    span: Span,
    termination_effect: NativeTerminationEffect<()>,
    llvm_message: &'static str,
    assembly_message: &'static str,
}

#[allow(dead_code)]
impl NativeTerminationCleanupBlocker {
    fn from_cleanup_boundary(boundary: NativeTerminationCleanupBoundary) -> Self {
        let span = boundary.span();
        let llvm_message = boundary.llvm_message();
        let assembly_message = boundary.assembly_message();
        Self {
            span,
            termination_effect: boundary.into_termination_effect(),
            llvm_message,
            assembly_message,
        }
    }

    fn from_hook_boundary(span: Span, boundary: NativeTerminationHookBoundary) -> Self {
        Self::from_cleanup_boundary(NativeTerminationCleanupBoundary::from_hook_boundary(
            span, boundary,
        ))
    }

    fn branch_effect(span: Span, branch_effect: NativeBranchTerminationEffect) -> Self {
        Self::from_cleanup_boundary(NativeTerminationCleanupBoundary::from_branch_effect(
            span,
            branch_effect,
        ))
    }

    fn span(&self) -> Span {
        self.span
    }

    fn cleanup_stack(&self) -> &NativeTerminationCleanupStack<()> {
        self.termination_effect.cleanup_stack()
    }

    fn termination_effect(&self) -> &NativeTerminationEffect<()> {
        &self.termination_effect
    }

    fn llvm_message(&self) -> &'static str {
        self.llvm_message
    }

    fn assembly_message(&self) -> &'static str {
        self.assembly_message
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeBranchMergeKind {
    BothTerminate,
    ContinueWith(NativeContinuingBranchArm),
    BothContinueStable,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeBranchMergeResult {
    kind: NativeBranchMergeKind,
    termination_effect: NativeTerminationEffect<()>,
}

#[allow(dead_code)]
impl NativeBranchMergeResult {
    fn new(kind: NativeBranchMergeKind, cleanup_stack: NativeTerminationCleanupStack<()>) -> Self {
        Self {
            kind,
            termination_effect: NativeTerminationEffect::from_cleanup_stack(cleanup_stack),
        }
    }

    fn kind(&self) -> NativeBranchMergeKind {
        self.kind
    }

    fn cleanup_stack(&self) -> &NativeTerminationCleanupStack<()> {
        self.termination_effect.cleanup_stack()
    }

    fn termination_effect(&self) -> &NativeTerminationEffect<()> {
        &self.termination_effect
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum NativeBranchMergeOutcome {
    Merged(NativeBranchMergeResult),
    Blocked(NativeTerminationCleanupBlocker),
}

#[allow(dead_code)]
impl NativeBranchTerminationEffect {
    fn into_merge_outcome(
        self,
        span: Span,
        backend_states_can_merge_without_phi: bool,
    ) -> NativeBranchMergeOutcome {
        let live_native_value_ownership = self.live_native_value_ownership();
        match self.join() {
            NativeBranchEffectJoin::BothTerminate
                if live_native_value_ownership.allows_non_joining_control_flow() =>
            {
                NativeBranchMergeOutcome::Merged(NativeBranchMergeResult::new(
                    NativeBranchMergeKind::BothTerminate,
                    self.into_cleanup_stack(),
                ))
            }
            NativeBranchEffectJoin::ThenContinues
                if live_native_value_ownership.allows_non_joining_control_flow() =>
            {
                NativeBranchMergeOutcome::Merged(NativeBranchMergeResult::new(
                    NativeBranchMergeKind::ContinueWith(NativeContinuingBranchArm::Then),
                    self.into_cleanup_stack(),
                ))
            }
            NativeBranchEffectJoin::ElseContinues
                if live_native_value_ownership.allows_non_joining_control_flow() =>
            {
                NativeBranchMergeOutcome::Merged(NativeBranchMergeResult::new(
                    NativeBranchMergeKind::ContinueWith(NativeContinuingBranchArm::Else),
                    self.into_cleanup_stack(),
                ))
            }
            NativeBranchEffectJoin::BothContinue
                if self.has_stable_control_merge() && backend_states_can_merge_without_phi =>
            {
                NativeBranchMergeOutcome::Merged(NativeBranchMergeResult::new(
                    NativeBranchMergeKind::BothContinueStable,
                    self.into_cleanup_stack(),
                ))
            }
            _ => NativeBranchMergeOutcome::Blocked(NativeTerminationCleanupBlocker::branch_effect(
                span, self,
            )),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum NativeTerminationStatus<Handle> {
    NativeValueHandle(Handle),
    CleanupOnly,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeTerminationEffect<Handle> {
    status: NativeTerminationStatus<Handle>,
    cleanup_stack: NativeTerminationCleanupStack<Handle>,
}

#[allow(dead_code)]
impl<Handle> NativeTerminationEffect<Handle> {
    fn from_cleanup_stack(cleanup_stack: NativeTerminationCleanupStack<Handle>) -> Self {
        Self {
            status: NativeTerminationStatus::CleanupOnly,
            cleanup_stack,
        }
    }

    fn status_value(&self) -> Option<&Handle> {
        match &self.status {
            NativeTerminationStatus::NativeValueHandle(handle) => Some(handle),
            NativeTerminationStatus::CleanupOnly => None,
        }
    }

    fn cleanup_stack(&self) -> &NativeTerminationCleanupStack<Handle> {
        &self.cleanup_stack
    }

    fn append_cleanup_stack(&mut self, cleanup_stack: NativeTerminationCleanupStack<Handle>) {
        self.cleanup_stack.append_stack(cleanup_stack);
    }

    fn prepend_cleanup_stack(&mut self, cleanup_stack: NativeTerminationCleanupStack<Handle>) {
        self.cleanup_stack.prepend_stack(cleanup_stack);
    }

    fn into_cleanup_stack(self) -> NativeTerminationCleanupStack<Handle> {
        self.cleanup_stack
    }

    fn runtime_return_plan(&self) -> NativeTerminationReturnPlan<'_, Handle> {
        match self.cleanup_stack.runtime_execution_plan() {
            NativeTerminationCleanupStackPlan::Executable(cleanup_plan) => {
                NativeTerminationReturnPlan::Executable(NativeTerminationReturnExecutionPlan {
                    status_value: self.status_value(),
                    cleanup_plan,
                })
            }
            NativeTerminationCleanupStackPlan::Blocked(boundary) => {
                NativeTerminationReturnPlan::Blocked(boundary)
            }
        }
    }
}

#[allow(dead_code)]
impl<Handle: Clone> NativeTerminationEffect<Handle> {
    fn from_native_value_handle(handle: Handle) -> Self {
        let mut cleanup_stack = NativeTerminationCleanupStack::new();
        cleanup_stack.push_live_native_value_handle(handle.clone());
        Self {
            status: NativeTerminationStatus::NativeValueHandle(handle),
            cleanup_stack,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeTerminationHookBoundary {
    Destructor,
    Exception,
    ExpressionContext,
    FunctionFrame,
    GotoScope,
    LoopScope,
    OutputBuffer,
    ReturnContext,
    Shutdown,
    SwitchScope,
    TryFinally,
}

#[allow(dead_code)]
impl NativeTerminationHookBoundary {
    fn llvm_message(self) -> &'static str {
        match self {
            Self::Destructor => LLVM_TERMINATION_DESTRUCTOR_REJECTION,
            Self::Exception => LLVM_TERMINATION_EXCEPTION_REJECTION,
            Self::ExpressionContext => LLVM_TERMINATION_EXPRESSION_CONTEXT_REJECTION,
            Self::FunctionFrame => LLVM_TERMINATION_FUNCTION_FRAME_REJECTION,
            Self::GotoScope => LLVM_TERMINATION_GOTO_SCOPE_REJECTION,
            Self::LoopScope => LLVM_TERMINATION_LOOP_SCOPE_REJECTION,
            Self::OutputBuffer => LLVM_TERMINATION_OUTPUT_BUFFER_REJECTION,
            Self::ReturnContext => LLVM_TERMINATION_RETURN_CONTEXT_REJECTION,
            Self::Shutdown => LLVM_TERMINATION_SHUTDOWN_REJECTION,
            Self::SwitchScope => LLVM_TERMINATION_SWITCH_SCOPE_REJECTION,
            Self::TryFinally => LLVM_TERMINATION_TRY_CONTEXT_REJECTION,
        }
    }

    fn assembly_message(self) -> &'static str {
        match self {
            Self::Destructor => ASSEMBLY_TERMINATION_DESTRUCTOR_REJECTION,
            Self::Exception => ASSEMBLY_TERMINATION_EXCEPTION_REJECTION,
            Self::ExpressionContext => ASSEMBLY_TERMINATION_EXPRESSION_CONTEXT_REJECTION,
            Self::FunctionFrame => ASSEMBLY_TERMINATION_FUNCTION_FRAME_REJECTION,
            Self::GotoScope => ASSEMBLY_TERMINATION_GOTO_SCOPE_REJECTION,
            Self::LoopScope => ASSEMBLY_TERMINATION_LOOP_SCOPE_REJECTION,
            Self::OutputBuffer => ASSEMBLY_TERMINATION_OUTPUT_BUFFER_REJECTION,
            Self::ReturnContext => ASSEMBLY_TERMINATION_RETURN_CONTEXT_REJECTION,
            Self::Shutdown => ASSEMBLY_TERMINATION_SHUTDOWN_REJECTION,
            Self::SwitchScope => ASSEMBLY_TERMINATION_SWITCH_SCOPE_REJECTION,
            Self::TryFinally => ASSEMBLY_TERMINATION_TRY_CONTEXT_REJECTION,
        }
    }

    fn cleanup_action(self) -> NativeTerminationCleanupAction<()> {
        match self {
            Self::Destructor => NativeTerminationCleanupAction::DestructorQueue,
            Self::Exception => NativeTerminationCleanupAction::ExceptionUnwind,
            Self::ExpressionContext => NativeTerminationCleanupAction::DiscardedNativeTemporaries,
            Self::FunctionFrame => NativeTerminationCleanupAction::FunctionFrame,
            Self::ReturnContext => NativeTerminationCleanupAction::ReturnContext,
            Self::TryFinally => NativeTerminationCleanupAction::FinallyDispatch,
            Self::GotoScope => NativeTerminationCleanupAction::GotoScope,
            Self::LoopScope => NativeTerminationCleanupAction::LoopScope,
            Self::OutputBuffer => NativeTerminationCleanupAction::OutputBufferStack,
            Self::Shutdown => NativeTerminationCleanupAction::ShutdownQueue,
            Self::SwitchScope => NativeTerminationCleanupAction::SwitchScope,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeTerminationCleanupBoundaryKind {
    Hook(NativeTerminationHookBoundary),
    BranchMerge,
}

#[allow(dead_code)]
impl NativeTerminationCleanupBoundaryKind {
    fn llvm_message(self) -> &'static str {
        match self {
            Self::Hook(boundary) => boundary.llvm_message(),
            Self::BranchMerge => LLVM_TERMINATION_PARTIAL_BRANCH_REJECTION,
        }
    }

    fn assembly_message(self) -> &'static str {
        match self {
            Self::Hook(boundary) => boundary.assembly_message(),
            Self::BranchMerge => ASSEMBLY_TERMINATION_PARTIAL_BRANCH_REJECTION,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeTerminationCleanupBoundary {
    span: Span,
    diagnostic_boundary: NativeTerminationCleanupBoundaryKind,
    cleanup_stack: NativeTerminationCleanupStack<()>,
}

#[allow(dead_code)]
impl NativeTerminationCleanupBoundary {
    fn from_hook_boundary(span: Span, boundary: NativeTerminationHookBoundary) -> Self {
        Self {
            span,
            diagnostic_boundary: NativeTerminationCleanupBoundaryKind::Hook(boundary),
            cleanup_stack: NativeTerminationCleanupStack::from_action(boundary.cleanup_action()),
        }
    }

    fn from_branch_effect(span: Span, branch_effect: NativeBranchTerminationEffect) -> Self {
        Self {
            span,
            diagnostic_boundary: NativeTerminationCleanupBoundaryKind::BranchMerge,
            cleanup_stack: branch_effect.into_cleanup_stack(),
        }
    }

    fn span(&self) -> Span {
        self.span
    }

    fn llvm_message(&self) -> &'static str {
        self.diagnostic_boundary.llvm_message()
    }

    fn assembly_message(&self) -> &'static str {
        self.diagnostic_boundary.assembly_message()
    }

    fn append_cleanup_boundary(&mut self, boundary: NativeTerminationCleanupBoundary) {
        self.cleanup_stack.append_stack(boundary.cleanup_stack);
    }

    fn prepend_cleanup_boundary(&mut self, boundary: NativeTerminationCleanupBoundary) {
        self.cleanup_stack.prepend_stack(boundary.cleanup_stack);
    }

    fn with_outer_hook_boundary(
        mut self,
        span: Span,
        boundary: NativeTerminationHookBoundary,
    ) -> Self {
        self.span = span;
        self.diagnostic_boundary = NativeTerminationCleanupBoundaryKind::Hook(boundary);
        self.cleanup_stack
            .prepend_stack(NativeTerminationCleanupStack::from_action(
                boundary.cleanup_action(),
            ));
        self
    }

    fn into_termination_effect(self) -> NativeTerminationEffect<()> {
        NativeTerminationEffect::from_cleanup_stack(self.cleanup_stack)
    }
}

impl NativeCallArgumentMaterialization {
    fn from_values(values: Vec<NativeValueMaterialization>) -> Self {
        let handles = values.iter().map(|value| value.handle.clone()).collect();
        let cleanup = values
            .into_iter()
            .rev()
            .flat_map(NativeValueMaterialization::cleanup_after_use)
            .collect();

        Self { handles, cleanup }
    }

    fn cleanup_after_call(self) -> Vec<String> {
        self.cleanup
    }
}

impl NativeValueMaterialization {
    fn owned(handle: String, cleanup: Vec<String>) -> Self {
        Self {
            handle,
            ownership: NativeValueHandleOwnership::Owned,
            cleanup,
        }
    }

    #[allow(dead_code)]
    fn borrowed(handle: String, cleanup: Vec<String>) -> Self {
        Self {
            handle,
            ownership: NativeValueHandleOwnership::Borrowed,
            cleanup,
        }
    }

    fn cleanup_after_use(self) -> Vec<String> {
        let mut cleanup = Vec::new();
        if matches!(self.ownership, NativeValueHandleOwnership::Owned) {
            cleanup.push(format!(
                "call void @phpc_native_value_free(%phpc.NativeValueHandle {})",
                self.handle
            ));
        }
        cleanup.extend(self.cleanup);
        cleanup
    }

    fn into_selection_branch_parts(self) -> (String, NativeValueHandleOwnership, Vec<String>) {
        (self.handle, self.ownership, self.cleanup)
    }
}

impl IrValue {
    fn into_static_fallback(self) -> Self {
        match self {
            IrValue::NativeExpression { fallback, .. } => fallback.into_static_fallback(),
            value => value,
        }
    }
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
        if self.uses_native_value_from_scalar {
            output.push_str("%phpc.NativeScalarValue = type { i8, i8, [6 x i8], i64, double }\n");
        }
        if self.uses_native_value_echo_stdout
            || self.uses_native_value_from_string
            || self.uses_native_symbol_table_helpers
            || self.uses_native_value_text_membership
            || self.uses_native_value_from_scalar
        {
            output.push_str("%phpc.NativeStringHandle = type { ptr }\n");
            output.push_str("%phpc.NativeValueHandle = type { ptr }\n");
            output.push_str("%phpc.NativeDiagnosticHandle = type { ptr }\n");
        }
        if self.uses_native_symbol_table_helpers {
            output.push_str("%phpc.NativeSymbolTableHandle = type { ptr }\n");
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
        if self.uses_native_value_from_string && !self.uses_native_value_echo_stdout {
            let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
            output.push_str(&format!(
                "declare %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr, {usize_type})\n"
            ));
            output.push_str("declare void @phpc_native_string_free(%phpc.NativeStringHandle)\n");
        }
        if self.uses_native_value_from_string && !self.uses_native_symbol_table_helpers {
            output.push_str(
                "declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)\n",
            );
        }
        if self.uses_native_value_from_scalar && !self.uses_native_symbol_table_helpers {
            output.push_str(
                "declare %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue)\n",
            );
        }
        if self.uses_native_value_text_membership {
            let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
            output.push_str(&format!(
                "declare i1 @phpc_native_value_text_membership_with_diagnostic(%phpc.NativeValueHandle, i8, ptr, ptr, {usize_type}, i1, ptr)\n"
            ));
            if !self.uses_native_value_echo_stdout {
                output.push_str(&format!(
                    "declare {usize_type} @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle)\n"
                ));
                output.push_str(
                    "declare void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle)\n",
                );
            }
            if !self.uses_native_value_echo_stdout {
                output.push_str("declare void @phpc_native_value_free(%phpc.NativeValueHandle)\n");
            }
        }
        if self.uses_native_symbol_table_helpers {
            let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
            output.push_str(
                "declare %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle)\n",
            );
            if self.uses_native_value_from_scalar {
                output.push_str(
                    "declare %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue)\n",
                );
            }
            output.push_str(
                "declare %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()\n",
            );
            output.push_str(&format!(
                "declare i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle, ptr, {usize_type}, %phpc.NativeValueHandle)\n"
            ));
            output.push_str(&format!(
                "declare %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle, ptr, {usize_type})\n"
            ));
            output.push_str(&format!(
                "declare i1 @phpc_native_symbol_table_isset(%phpc.NativeSymbolTableHandle, ptr, {usize_type})\n"
            ));
            output.push_str(&format!(
                "declare i1 @phpc_native_symbol_table_empty(%phpc.NativeSymbolTableHandle, ptr, {usize_type})\n"
            ));
            output.push_str(&format!(
                "declare i1 @phpc_native_symbol_table_unset(%phpc.NativeSymbolTableHandle, ptr, {usize_type})\n"
            ));
            output.push_str(
                "declare void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle)\n",
            );
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
        for global in &self.native_globals {
            output.push_str(global);
            output.push('\n');
        }

        output.push_str("\ndefine i32 @main() {\nentry:\n");
        for line in &self.body {
            if !line.ends_with(':') {
                output.push_str("  ");
            }
            output.push_str(line);
            output.push('\n');
        }
        if self.uses_native_symbol_table_helpers {
            output.push_str(
                "  call void @phpc_native_symbol_table_free(%phpc.NativeSymbolTableHandle %phpc.symbols)\n",
            );
        }
        output.push_str("  ret i32 0\n}\n");
        Ok(output)
    }

    fn emit_statement(&mut self, stmt: &Stmt) -> CompileResult<()> {
        match stmt {
            Stmt::Namespace { span, .. } | Stmt::Use { span, .. } => {
                Err(self.unsupported(*span, LLVM_NAMESPACE_REJECTION))
            }
            Stmt::Echo { exprs, .. } => {
                for expr in exprs {
                    let value = self.emit_expr(expr)?;
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
            Stmt::ReferenceAssign { span, .. } => {
                Err(self.unsupported(*span, NativeValueBlocker::Reference.rejection()))
            }
            Stmt::CompoundAssign {
                target, expr, span, ..
            }
            | Stmt::NullCoalesceAssign {
                target, expr, span, ..
            } => {
                if let Some(superglobal_span) =
                    request_superglobal_consumed_assign_target_span(target)
                {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(expr) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, LLVM_ARRAY_ACCESS_REJECTION));
                }
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Stmt::IncrementDecrement { target, span, .. } => {
                if let Some(superglobal_span) =
                    request_superglobal_consumed_assign_target_span(target)
                {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, LLVM_ARRAY_ACCESS_REJECTION));
                }
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Stmt::Expr { expr, .. } => {
                self.emit_expr(expr)?;
                Ok(())
            }
            Stmt::Function(function) => {
                if let Some(span) = find_static_local_span(&function.body) {
                    return Err(self.unsupported(span, LLVM_STATIC_LOCAL_REJECTION));
                }
                Err(self.unsupported(function.span, LLVM_FUNCTION_DECLARATION_REJECTION))
            }
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
            Stmt::UnsetVariable { name, .. } => {
                self.emit_native_symbol_table_unset(name);
                self.variables.remove(name);
                Ok(())
            }
            Stmt::UnsetStaticProperty { span, .. }
            | Stmt::UnsetSelfStaticProperty { span, .. }
            | Stmt::UnsetParentStaticProperty { span, .. }
            | Stmt::UnsetLateStaticProperty { span, .. } => {
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Stmt::UnsetObjectProperty { span, .. }
            | Stmt::UnsetDynamicObjectProperty { span, .. } => {
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Stmt::UnsetArrayIndex { span, .. } | Stmt::UnsetNestedArrayIndex { span, .. } => {
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            Stmt::UnsetMany { targets, span } => {
                if targets
                    .iter()
                    .all(|target| matches!(target, UnsetTarget::Variable { .. }))
                {
                    for target in targets {
                        let UnsetTarget::Variable { name, .. } = target else {
                            unreachable!("all unset targets are direct variables");
                        };
                        self.emit_native_symbol_table_unset(name);
                        self.variables.remove(name);
                    }
                    return Ok(());
                }
                if targets
                    .iter()
                    .any(is_object_property_array_access_unset_target)
                {
                    return Err(self.unsupported(*span, LLVM_ARRAY_ACCESS_REJECTION));
                }
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Stmt::ConstDeclaration { declarations, span } => {
                if let Some(superglobal_span) = declarations.iter().find_map(|declaration| {
                    request_superglobal_consumed_expr_span(&declaration.value)
                }) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_GLOBAL_CONSTANT_REJECTION))
            }
            Stmt::Require { path, span, .. } | Stmt::Include { path, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(path) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_REQUIRE_REJECTION))
            }
            Stmt::Throw { expr, span } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(expr) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_EXCEPTION_REJECTION))
            }
            Stmt::Try { span, .. } => Err(self.unsupported(*span, LLVM_TRY_BLOCK_REJECTION)),
            Stmt::Return { value, span } => {
                if let Some(value) = value {
                    if let Some(superglobal_span) = request_superglobal_consumed_expr_span(value) {
                        return Err(
                            self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                        );
                    }
                }
                self.emit_return_boundary(value.as_ref(), *span)
            }
            Stmt::Global { names, span } => {
                if names.iter().any(|name| is_request_superglobal_name(name)) {
                    return Err(self.unsupported(*span, LLVM_REQUEST_SUPERGLOBAL_REJECTION));
                }
                Err(self.unsupported(*span, LLVM_GLOBAL_DECLARATION_REJECTION))
            }
            Stmt::StaticLocal { declarations, span } => {
                if let Some(superglobal_span) = declarations.iter().find_map(|declaration| {
                    declaration
                        .default
                        .as_ref()
                        .and_then(request_superglobal_consumed_expr_span)
                }) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
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
            Expr::ObjectStaticProperty { target, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(target) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_STATIC_MEMBER_REJECTION))
            }
            Expr::Array { items, span } => {
                if let Some(superglobal_span) = items.iter().find_map(|item| {
                    item.key
                        .as_ref()
                        .and_then(request_superglobal_consumed_expr_span)
                        .or_else(|| request_superglobal_consumed_expr_span(&item.value))
                }) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            Expr::Index { target, span, .. } => {
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
            Expr::AppendIndex { target, span } => {
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
            Expr::Property { target, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(target) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_OBJECT_PROPERTY_REJECTION))
            }
            Expr::DynamicProperty {
                target,
                property,
                span,
            } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(target)
                    .or_else(|| request_superglobal_consumed_expr_span(property))
                {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_OBJECT_PROPERTY_REJECTION))
            }
            Expr::MethodCall {
                target, args, span, ..
            } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(target)
                    .or_else(|| request_superglobal_consumed_args_span(args))
                {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_METHOD_CALL_REJECTION))
            }
            Expr::DynamicMethodCall {
                target,
                method,
                args,
                span,
            } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(target)
                    .or_else(|| request_superglobal_consumed_expr_span(method))
                    .or_else(|| request_superglobal_consumed_args_span(args))
                {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_METHOD_CALL_REJECTION))
            }
            Expr::ObjectStaticMethodCall {
                target, args, span, ..
            } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(target)
                    .or_else(|| request_superglobal_consumed_args_span(args))
                {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_METHOD_CALL_REJECTION))
            }
            Expr::ParentMethodCall { args, span, .. }
            | Expr::StaticMethodCall { args, span, .. }
            | Expr::SelfMethodCall { args, span, .. }
            | Expr::LateStaticMethodCall { args, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_METHOD_CALL_REJECTION))
            }
            Expr::Variable(name, span) => {
                if is_request_superglobal_name(name) {
                    return Err(self.unsupported(*span, LLVM_REQUEST_SUPERGLOBAL_REJECTION));
                }
                Ok(self.direct_local_native_expression_value(name))
            }
            Expr::Call { name, args, span } if is_exit_construct_name(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_TERMINATION_REJECTION))
            }
            Expr::Call { name, args, span } if is_value_debug_output_builtin(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_VALUE_DEBUG_OUTPUT_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("defined") => {
                self.emit_defined_call(args, *span)
            }
            Expr::Call { name, args, span } if is_global_constant_builtin(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_GLOBAL_CONSTANT_REJECTION))
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
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("str_starts_with") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_STR_STARTS_WITH_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("str_ends_with") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_STR_ENDS_WITH_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("basename") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_BASENAME_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("file_get_contents") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_FILE_GET_CONTENTS_REJECTION))
            }
            Expr::Call { name, args, span } if is_stream_resource_builtin(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_STREAM_RESOURCE_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("getcwd") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_GETCWD_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("realpath") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_REALPATH_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("is_writable") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_IS_WRITABLE_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("clearstatcache") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_CLEARSTATCACHE_REJECTION))
            }
            Expr::Call { name, args, span } if is_header_state_builtin(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_HEADER_STATE_REJECTION))
            }
            Expr::Call { name, args, span } if is_session_state_builtin(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_SESSION_STATE_REJECTION))
            }
            Expr::Call { name, args, span } if is_output_buffer_builtin(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_OUTPUT_BUFFER_REJECTION))
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
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_OBJECT_METADATA_REJECTION))
            }
            Expr::Call { name, args, span } if is_array_builtin(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            Expr::DynamicCall { callee, args, span } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(callee)
                    .or_else(|| request_superglobal_consumed_args_span(args))
                {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, NativeValueBlocker::DynamicCall.rejection()))
            }
            Expr::Call { args, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                self.emit_unsupported_call_boundary(args, *span)
            }
            Expr::InstanceOf { expr, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(expr) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_INSTANCEOF_REJECTION))
            }
            Expr::Closure { span, .. } => Err(self.unsupported(*span, LLVM_CLOSURE_REJECTION)),
            Expr::New {
                class_name,
                args,
                span,
            } => {
                if let Some(superglobal_span) =
                    request_superglobal_consumed_new_class_name_span(class_name, *span)
                        .or_else(|| request_superglobal_consumed_args_span(args))
                {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_OBJECT_INSTANTIATION_REJECTION))
            }
            Expr::Clone { expr, span } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(expr) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
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
                        let value = self.emit_expr(expr)?;
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
                        return match self.emit_expr(expr)? {
                            value @ IrValue::Int(_) => Ok(value),
                            _ => Err(self.unsupported(*span, LLVM_BITWISE_REJECTION)),
                        };
                    }
                }
                let value = self.emit_expr(expr)?;
                self.emit_unary(*op, value, *span)
            }
            Expr::ErrorControl { expr, span } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(expr) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_ERROR_CONTROL_REJECTION))
            }
            Expr::Include { path, span, .. } | Expr::Require { path, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(path) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_REQUIRE_EXPRESSION_REJECTION))
            }
            Expr::Cast { expr, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(expr) {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, LLVM_CAST_REJECTION))
            }
            Expr::Assign { target, expr, span } => {
                if let Some(superglobal_span) =
                    request_superglobal_consumed_assign_target_span(target)
                        .or_else(|| request_superglobal_consumed_expr_span(expr))
                {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, LLVM_ARRAY_ACCESS_REJECTION));
                }
                if is_static_member_assign_target(target) {
                    return Err(self.unsupported(*span, LLVM_STATIC_MEMBER_REJECTION));
                }
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Expr::CompoundAssign {
                target, expr, span, ..
            }
            | Expr::NullCoalesceAssign {
                target, expr, span, ..
            } => {
                if let Some(superglobal_span) =
                    request_superglobal_consumed_assign_target_span(target)
                        .or_else(|| request_superglobal_consumed_expr_span(expr))
                {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, LLVM_ARRAY_ACCESS_REJECTION));
                }
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Expr::IncrementDecrement { target, span, .. } => {
                if let Some(superglobal_span) =
                    request_superglobal_consumed_assign_target_span(target)
                {
                    return Err(
                        self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION)
                    );
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
                let left = self.emit_expr(left)?;
                let right = self.emit_expr(right)?;
                self.emit_binary(left, *op, right, *span)
            }
        }
    }

    fn emit_isset_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if args.is_empty() {
            return Err(self.unsupported(span, LLVM_ISSET_REJECTION));
        }

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.direct_local_native_expression_from_arg(arg, LLVM_ISSET_REJECTION)?);
        }

        let mut result = self.emit_native_expression_liveness_call("isset", &values[0]);
        for value in values.iter().skip(1) {
            let current = self.emit_native_expression_liveness_call("isset", value);
            let combined = self.next_temp();
            self.body
                .push(format!("{combined} = and i1 {result}, {current}"));
            result = combined;
        }

        Ok(IrValue::BoolExpr(result))
    }

    fn emit_empty_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        let [arg] = args else {
            return Err(self.unsupported(span, LLVM_EMPTY_REJECTION));
        };

        if let Some(superglobal_span) = request_superglobal_expr_span(arg) {
            return Err(self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION));
        }

        if is_array_access_offset_expr(arg) {
            return Err(self.unsupported(arg.span(), LLVM_ARRAY_ACCESS_REJECTION));
        }

        let value = self.direct_local_native_expression_from_arg(arg, LLVM_EMPTY_REJECTION)?;

        Ok(IrValue::BoolExpr(
            self.emit_native_expression_liveness_call("empty", &value),
        ))
    }

    fn emit_strlen_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if args.len() != 1 {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        self.strlen_result_for_value(&value)
            .map(|length| IrValue::Int(length.to_string()))
            .ok_or_else(|| self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION))
    }

    fn emit_function_exists_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if args.len() != 1 {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        if let Some(result) = self.function_exists_result_for_value(&value) {
            return Ok(IrValue::Bool(result));
        }

        self.emit_native_text_membership_bool(
            value,
            NativeTextSurface::FunctionName,
            NATIVE_KNOWN_FUNCTION_NAMES,
            true,
            span,
            LLVM_FUNCTION_CALL_REJECTION,
        )
    }

    fn emit_is_callable_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if !(1..=2).contains(&args.len()) {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        let syntax_only = if let Some(arg) = args.get(1) {
            match self.emit_expr(arg)? {
                IrValue::Bool(value) => value,
                _ => return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION)),
            }
        } else {
            false
        };

        if let Some(result) = self.is_callable_result_for_value(&value, syntax_only) {
            return Ok(IrValue::Bool(result));
        }
        if syntax_only {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        self.emit_native_text_membership_bool(
            value,
            NativeTextSurface::FunctionName,
            NATIVE_KNOWN_FUNCTION_NAMES,
            true,
            span,
            LLVM_FUNCTION_CALL_REJECTION,
        )
    }

    fn emit_defined_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if args.len() != 1 {
            return Err(self.unsupported(span, LLVM_GLOBAL_CONSTANT_REJECTION));
        }
        if matches!(args[0], Expr::InterpolatedString { .. }) {
            return Err(self.unsupported(span, LLVM_GLOBAL_CONSTANT_REJECTION));
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
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
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
                .ok_or_else(|| self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION)),
            "is_countable" | "is_iterable" => Ok(IrValue::Bool(false)),
            "extension_loaded" => {
                if let IrValue::String(name) = &value {
                    return Ok(IrValue::Bool(is_compat_loaded_extension_name(name)));
                }

                self.emit_native_text_membership_bool(
                    value,
                    NativeTextSurface::ExtensionName,
                    COMPAT_LOADED_EXTENSION_NAMES,
                    true,
                    span,
                    LLVM_FUNCTION_CALL_REJECTION,
                )
            }
            "is_object" => Ok(IrValue::Bool(false)),
            _ => Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION)),
        }
    }

    fn emit_native_metadata_exists_call(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<IrValue> {
        if !(1..=2).contains(&args.len()) {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        let name = self.emit_expr(&args[0])?;
        if !matches!(name, IrValue::String(_) | IrValue::StringPtr(_)) {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }
        if self.ir_value_mentions_builtin_class(&name) {
            return Err(self.unsupported(span, LLVM_OBJECT_METADATA_REJECTION));
        }

        if let Some(autoload) = args.get(1) {
            let autoload = self.emit_expr(autoload)?;
            if !matches!(autoload, IrValue::Bool(_) | IrValue::BoolExpr(_)) {
                return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
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
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        let object_or_class = self.emit_expr(&args[0])?;
        let member = self.emit_expr(&args[1])?;
        if !matches!(object_or_class, IrValue::String(_) | IrValue::StringPtr(_))
            || !matches!(member, IrValue::String(_) | IrValue::StringPtr(_))
        {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
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
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        let object_or_class = self.emit_expr(&args[0])?;
        let class_name = self.emit_expr(&args[1])?;
        if !matches!(object_or_class, IrValue::String(_) | IrValue::StringPtr(_))
            || !matches!(class_name, IrValue::String(_) | IrValue::StringPtr(_))
        {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }
        if self.ir_value_mentions_builtin_class(&object_or_class)
            || self.ir_value_mentions_builtin_class(&class_name)
        {
            return Err(self.unsupported(span, LLVM_OBJECT_METADATA_REJECTION));
        }

        if let Some(allow_string) = args.get(2) {
            let allow_string = self.emit_expr(allow_string)?;
            if !matches!(allow_string, IrValue::Bool(_) | IrValue::BoolExpr(_)) {
                return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
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

    fn emit_native_text_membership_bool(
        &mut self,
        value: IrValue,
        surface: NativeTextSurface,
        candidates: &[&str],
        case_insensitive: bool,
        span: Span,
        rejection: &'static str,
    ) -> CompileResult<IrValue> {
        let materialized = self
            .materialize_native_value_handle(&value)
            .into_result()
            .map_err(|blocker| {
                let message = if matches!(blocker, NativeValueBlocker::Unsupported) {
                    rejection
                } else {
                    blocker.rejection()
                };
                self.unsupported(span, message)
            })?;
        let (candidate_ptrs, candidate_lengths) =
            self.emit_native_text_membership_candidate_table(candidates);
        let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
        let diagnostic_slot = self.next_temp();
        let result = self.next_temp();
        let diagnostic = self.next_temp();

        self.uses_native_value_text_membership = true;
        self.body.push(format!(
            "{diagnostic_slot} = alloca %phpc.NativeDiagnosticHandle"
        ));
        self.body.push(format!(
            "store %phpc.NativeDiagnosticHandle zeroinitializer, ptr {diagnostic_slot}"
        ));
        self.body.push(format!(
            "{result} = call i1 @phpc_native_value_text_membership_with_diagnostic(%phpc.NativeValueHandle {}, i8 {}, ptr {candidate_ptrs}, ptr {candidate_lengths}, {usize_type} {}, i1 {}, ptr {diagnostic_slot})",
            materialized.handle,
            surface.surface_tag(),
            candidates.len(),
            if case_insensitive { "true" } else { "false" }
        ));
        self.body.push(format!(
            "{diagnostic} = load %phpc.NativeDiagnosticHandle, ptr {diagnostic_slot}"
        ));
        self.body.push(format!(
            "call {usize_type} @phpc_native_diagnostic_message_stderr(%phpc.NativeDiagnosticHandle {diagnostic})"
        ));
        self.body.push(format!(
            "call void @phpc_native_diagnostic_free(%phpc.NativeDiagnosticHandle {diagnostic})"
        ));
        self.body.extend(materialized.cleanup_after_use());

        Ok(IrValue::BoolExpr(result))
    }

    fn emit_native_text_membership_candidate_table(
        &mut self,
        candidates: &[&str],
    ) -> (String, String) {
        if candidates.is_empty() {
            return ("null".to_string(), "null".to_string());
        }

        let index = self.native_globals.len() / 2;
        let ptrs = format!("phpc_text_membership_candidates_{index}");
        let lengths = format!("phpc_text_membership_candidate_lengths_{index}");
        let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
        let candidate_ptrs = candidates
            .iter()
            .map(|candidate| format!("ptr @{}", self.add_string(candidate)))
            .collect::<Vec<_>>()
            .join(", ");
        let candidate_lengths = candidates
            .iter()
            .map(|candidate| format!("{usize_type} {}", candidate.len()))
            .collect::<Vec<_>>()
            .join(", ");

        self.native_globals.push(format!(
            "@{ptrs} = private unnamed_addr constant [{} x ptr] [{candidate_ptrs}]",
            candidates.len()
        ));
        self.native_globals.push(format!(
            "@{lengths} = private unnamed_addr constant [{} x {usize_type}] [{candidate_lengths}]",
            candidates.len()
        ));

        (format!("@{ptrs}"), format!("@{lengths}"))
    }

    fn is_numeric_result_for_value(&self, value: &IrValue) -> Option<bool> {
        match value {
            IrValue::Int(_) | IrValue::Float(_) => Some(true),
            IrValue::Null | IrValue::Bool(_) | IrValue::BoolExpr(_) => Some(false),
            IrValue::String(value) => Some(is_php_numeric_string_literal(value)),
            IrValue::StringPtr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_numeric_result(&values)
            }
            IrValue::NativeExpression { fallback, .. } => {
                self.is_numeric_result_for_value(fallback)
            }
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
            IrValue::NativeExpression { fallback, .. } => self.defined_result_for_value(fallback),
            _ => None,
        }
    }

    fn ensure_native_symbol_table(&mut self) -> &'static str {
        self.uses_native_symbol_table_helpers = true;
        if !self.emitted_native_symbol_table {
            self.body.push(
                "%phpc.symbols = call %phpc.NativeSymbolTableHandle @phpc_native_symbol_table_new()"
                    .to_string(),
            );
            self.emitted_native_symbol_table = true;
        }
        "%phpc.symbols"
    }

    fn emit_direct_local_assignment(&mut self, name: &str, expr: &Expr) -> CompileResult<()> {
        if let Some(blocker) = native_value_blocker_for_expr(expr) {
            return Err(self.unsupported(expr.span(), blocker.rejection()));
        }

        if matches!(expr, Expr::Ternary { .. } | Expr::ShortTernary { .. }) {
            self.ensure_native_symbol_table();
            let materialized = self.materialize_native_expr_value(expr, expr.span())?;
            let stored_value = self.emit_native_direct_local_assignment_materialized(
                name,
                materialized,
                IrValue::Null,
            );
            self.variables.insert(name.to_string(), stored_value);
            return Ok(());
        }

        let value = self.emit_expr(expr)?;
        let stored_value = self
            .emit_native_direct_local_assignment(name, &value)
            .unwrap_or(value);
        self.variables.insert(name.to_string(), stored_value);
        Ok(())
    }

    fn emit_return_boundary(&mut self, value: Option<&Expr>, span: Span) -> CompileResult<()> {
        let materialized = self.materialize_native_return_value(value, span)?;
        self.body.extend(materialized.cleanup_after_use());
        Err(self.unsupported(span, LLVM_FUNCTION_DECLARATION_REJECTION))
    }

    fn emit_unsupported_call_boundary(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<IrValue> {
        let materialized_args = self.materialize_native_call_arguments(args, span)?;
        let _argument_count = materialized_args.handles.len();
        self.body.extend(materialized_args.cleanup_after_call());
        Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION))
    }

    fn materialize_native_return_value(
        &mut self,
        value: Option<&Expr>,
        span: Span,
    ) -> CompileResult<NativeValueMaterialization> {
        match value {
            Some(value) => self.materialize_native_expr_value(value, span),
            None => self
                .materialize_native_value_handle(&IrValue::Null)
                .into_result()
                .map_err(|blocker| self.unsupported(span, blocker.rejection())),
        }
    }

    fn materialize_native_call_arguments(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<NativeCallArgumentMaterialization> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.materialize_native_expr_value(arg, span)?);
        }

        Ok(NativeCallArgumentMaterialization::from_values(values))
    }

    fn materialize_native_expr_value(
        &mut self,
        expr: &Expr,
        span: Span,
    ) -> CompileResult<NativeValueMaterialization> {
        match expr {
            Expr::Ternary {
                condition,
                if_true,
                if_false,
                span,
            } => {
                return self.materialize_native_ternary_value(condition, if_true, if_false, *span);
            }
            Expr::ShortTernary {
                condition,
                if_false,
                span,
            } => return self.materialize_native_short_ternary_value(condition, if_false, *span),
            _ => {}
        }

        if let Some(blocker) = native_value_blocker_for_expr(expr) {
            return Err(self.unsupported(expr.span(), blocker.rejection()));
        }

        let value = self.emit_expr(expr)?;
        self.materialize_native_value_handle(&value)
            .into_result()
            .map_err(|blocker| self.unsupported(span, blocker.rejection()))
    }

    fn materialize_native_ternary_value(
        &mut self,
        condition: &Expr,
        if_true: &Expr,
        if_false: &Expr,
        span: Span,
    ) -> CompileResult<NativeValueMaterialization> {
        if let Some(blocker) = native_value_blocker_for_expr(condition) {
            return Err(self.unsupported(condition.span(), blocker.rejection()));
        }

        let condition_value = self.emit_expr(condition)?;
        if let Some(truthy) = self.known_truthiness_for_value(&condition_value) {
            return if truthy {
                self.materialize_native_expr_value(if_true, span)
            } else {
                self.materialize_native_expr_value(if_false, span)
            };
        }

        let condition = self.native_selection_condition_operand(condition_value, span)?;
        self.materialize_native_value_selection(
            &condition,
            NativeSelectionBranch::Expr(if_true),
            NativeSelectionBranch::Expr(if_false),
            span,
        )
    }

    fn materialize_native_short_ternary_value(
        &mut self,
        condition: &Expr,
        if_false: &Expr,
        span: Span,
    ) -> CompileResult<NativeValueMaterialization> {
        if let Some(blocker) = native_value_blocker_for_expr(condition) {
            return Err(self.unsupported(condition.span(), blocker.rejection()));
        }

        let condition_value = self.emit_expr(condition)?;
        if let Some(truthy) = self.known_truthiness_for_value(&condition_value) {
            return if truthy {
                self.materialize_native_value_handle(&condition_value)
                    .into_result()
                    .map_err(|blocker| self.unsupported(span, blocker.rejection()))
            } else {
                self.materialize_native_expr_value(if_false, span)
            };
        }

        let condition_operand =
            self.native_selection_condition_operand(condition_value.clone(), span)?;
        self.materialize_native_value_selection(
            &condition_operand,
            NativeSelectionBranch::Value(condition_value),
            NativeSelectionBranch::Expr(if_false),
            span,
        )
    }

    fn native_selection_condition_operand(
        &mut self,
        condition: IrValue,
        span: Span,
    ) -> CompileResult<String> {
        match condition {
            IrValue::NativeExpression { value, .. } => {
                Ok(self.emit_native_expression_truthiness(&value))
            }
            condition => {
                let condition = condition.into_static_fallback();
                llvm_bool_operand(condition)
                    .ok_or_else(|| self.unsupported(span, LLVM_CONDITIONAL_REJECTION))
            }
        }
    }

    fn materialize_native_value_selection(
        &mut self,
        condition: &str,
        if_true: NativeSelectionBranch<'_>,
        if_false: NativeSelectionBranch<'_>,
        span: Span,
    ) -> CompileResult<NativeValueMaterialization> {
        let true_label = self.next_label("native.select.true");
        let false_label = self.next_label("native.select.false");
        let merge_label = self.next_label("native.select.merge");

        self.body.push(format!(
            "br i1 {condition}, label %{true_label}, label %{false_label}"
        ));

        self.body.push(format!("{true_label}:"));
        let true_value = self.materialize_native_selection_branch(if_true, span)?;
        let (true_handle, true_ownership, true_cleanup) = true_value.into_selection_branch_parts();
        self.body.extend(true_cleanup);
        self.body.push(format!("br label %{merge_label}"));

        self.body.push(format!("{false_label}:"));
        let false_value = self.materialize_native_selection_branch(if_false, span)?;
        let (false_handle, false_ownership, false_cleanup) =
            false_value.into_selection_branch_parts();
        self.body.extend(false_cleanup);
        self.body.push(format!("br label %{merge_label}"));

        let ownership = match (true_ownership, false_ownership) {
            (NativeValueHandleOwnership::Owned, NativeValueHandleOwnership::Owned) => {
                NativeValueHandleOwnership::Owned
            }
            (NativeValueHandleOwnership::Borrowed, NativeValueHandleOwnership::Borrowed) => {
                NativeValueHandleOwnership::Borrowed
            }
            _ => return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION)),
        };

        self.body.push(format!("{merge_label}:"));
        let selected = self.next_temp();
        self.body.push(format!(
            "{selected} = phi %phpc.NativeValueHandle [ {true_handle}, %{true_label} ], [ {false_handle}, %{false_label} ]"
        ));

        Ok(NativeValueMaterialization {
            handle: selected,
            ownership,
            cleanup: Vec::new(),
        })
    }

    fn materialize_native_selection_branch(
        &mut self,
        branch: NativeSelectionBranch<'_>,
        span: Span,
    ) -> CompileResult<NativeValueMaterialization> {
        match branch {
            NativeSelectionBranch::Value(value) => self
                .materialize_native_value_handle(&value)
                .into_result()
                .map_err(|blocker| self.unsupported(span, blocker.rejection())),
            NativeSelectionBranch::Expr(expr) => self.materialize_native_expr_value(expr, span),
        }
    }

    fn emit_native_direct_local_assignment(
        &mut self,
        name: &str,
        value: &IrValue,
    ) -> Option<IrValue> {
        self.ensure_native_symbol_table();
        let materialized = self
            .materialize_native_value_handle(value)
            .into_available()?;
        Some(self.emit_native_direct_local_assignment_materialized(
            name,
            materialized,
            value.clone(),
        ))
    }

    fn emit_native_direct_local_assignment_materialized(
        &mut self,
        name: &str,
        materialized: NativeValueMaterialization,
        fallback: IrValue,
    ) -> IrValue {
        let table = self.ensure_native_symbol_table();
        self.uses_native_value_echo_stdout = true;
        let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
        let name_global = self.add_string(name);
        let written = self.next_temp();

        self.body.push(format!(
            "{written} = call i1 @phpc_native_symbol_table_write(%phpc.NativeSymbolTableHandle {table}, ptr @{name_global}, {usize_type} {}, %phpc.NativeValueHandle {})",
            name.len(),
            materialized.handle
        ));
        self.body.extend(materialized.cleanup_after_use());

        IrValue::NativeExpression {
            value: self.direct_local_native_expression(name),
            fallback: Box::new(fallback),
        }
    }

    fn direct_local_native_expression(&self, name: &str) -> NativeExpressionValue {
        NativeExpressionValue::DirectLocalSymbol {
            name: name.to_string(),
        }
    }

    fn direct_local_native_expression_from_arg(
        &self,
        arg: &Expr,
        rejection: &'static str,
    ) -> CompileResult<NativeExpressionValue> {
        if let Some(superglobal_span) = request_superglobal_expr_span(arg) {
            return Err(self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION));
        }

        if is_array_access_offset_expr(arg) {
            return Err(self.unsupported(arg.span(), LLVM_ARRAY_ACCESS_REJECTION));
        }

        let Expr::Variable(name, _) = arg else {
            return Err(self.unsupported(arg.span(), rejection));
        };

        Ok(self.direct_local_native_expression(name))
    }

    fn direct_local_native_expression_value(&self, name: &str) -> IrValue {
        IrValue::NativeExpression {
            value: self.direct_local_native_expression(name),
            fallback: Box::new(self.variables.get(name).cloned().unwrap_or(IrValue::Null)),
        }
    }

    fn emit_native_expression_liveness_call(
        &mut self,
        helper: &str,
        value: &NativeExpressionValue,
    ) -> String {
        let table = self.ensure_native_symbol_table();
        let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
        let NativeExpressionValue::DirectLocalSymbol { name } = value;
        let name_global = self.add_string(name);
        let result = self.next_temp();

        self.body.push(format!(
            "{result} = call i1 @phpc_native_symbol_table_{helper}(%phpc.NativeSymbolTableHandle {table}, ptr @{name_global}, {usize_type} {})",
            name.len()
        ));

        result
    }

    fn emit_native_symbol_table_unset(&mut self, name: &str) {
        let value = self.direct_local_native_expression(name);
        let _ = self.emit_native_expression_liveness_call("unset", &value);
    }

    fn emit_native_expression_truthiness(&mut self, value: &NativeExpressionValue) -> String {
        let empty = self.emit_native_expression_liveness_call("empty", value);
        let truthy = self.next_temp();
        self.body.push(format!("{truthy} = xor i1 {empty}, true"));
        truthy
    }

    fn emit_native_expression_read_value(
        &mut self,
        value: &NativeExpressionValue,
    ) -> NativeValueMaterialization {
        let table = self.ensure_native_symbol_table();
        let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
        let NativeExpressionValue::DirectLocalSymbol { name } = value;
        let name_global = self.add_string(name);
        let runtime_value = self.next_temp();
        self.body.push(format!(
            "{runtime_value} = call %phpc.NativeValueHandle @phpc_native_symbol_table_read(%phpc.NativeSymbolTableHandle {table}, ptr @{name_global}, {usize_type} {})",
            name.len()
        ));

        NativeValueMaterialization::owned(runtime_value, Vec::new())
    }

    fn materialize_native_value_handle(&mut self, value: &IrValue) -> NativeValueHandleResult {
        match value {
            IrValue::NativeExpression { value, .. } => {
                NativeValueHandleResult::Available(self.emit_native_expression_read_value(value))
            }
            IrValue::String(_) | IrValue::StringPtr(_) => {
                let Some((value_ptr, value_len)) = self.native_string_pointer_and_len(value) else {
                    return NativeValueHandleResult::Blocked(NativeValueBlocker::Unsupported);
                };
                self.uses_native_value_from_string = true;
                let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
                let string = self.next_temp();
                let runtime_value = self.next_temp();

                self.body.push(format!(
                    "{string} = call %phpc.NativeStringHandle @phpc_native_string_from_bytes(ptr {value_ptr}, {usize_type} {value_len})"
                ));
                self.body.push(format!(
                    "{runtime_value} = call %phpc.NativeValueHandle @phpc_native_value_from_string(%phpc.NativeStringHandle {string})"
                ));

                NativeValueHandleResult::Available(NativeValueMaterialization::owned(
                    runtime_value,
                    vec![format!(
                        "call void @phpc_native_string_free(%phpc.NativeStringHandle {string})"
                    )],
                ))
            }
            IrValue::Int(value) => {
                NativeValueHandleResult::Available(NativeValueMaterialization::owned(
                    self.emit_native_scalar_value_handle(2, None, Some(value), None),
                    Vec::new(),
                ))
            }
            IrValue::Bool(value) => {
                let bool_value = if *value { "1" } else { "0" };
                NativeValueHandleResult::Available(NativeValueMaterialization::owned(
                    self.emit_native_scalar_value_handle(1, Some(bool_value), None, None),
                    Vec::new(),
                ))
            }
            IrValue::BoolExpr(value) => {
                let bool_value = self.next_temp();
                self.body
                    .push(format!("{bool_value} = zext i1 {value} to i8"));
                NativeValueHandleResult::Available(NativeValueMaterialization::owned(
                    self.emit_native_scalar_value_handle(1, Some(&bool_value), None, None),
                    Vec::new(),
                ))
            }
            IrValue::Float(value) => {
                NativeValueHandleResult::Available(NativeValueMaterialization::owned(
                    self.emit_native_scalar_value_handle(3, None, None, Some(value)),
                    Vec::new(),
                ))
            }
            IrValue::Null => NativeValueHandleResult::Available(NativeValueMaterialization::owned(
                self.emit_native_scalar_value_handle(0, None, None, None),
                Vec::new(),
            )),
        }
    }

    fn emit_native_scalar_value_handle(
        &mut self,
        tag: u8,
        bool_value: Option<&str>,
        int_value: Option<&str>,
        float_value: Option<&str>,
    ) -> String {
        self.uses_native_value_from_scalar = true;
        let tagged = self.next_temp();
        self.body.push(format!(
            "{tagged} = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 {tag}, 0"
        ));

        let scalar = if let Some(bool_value) = bool_value {
            let scalar = self.next_temp();
            self.body.push(format!(
                "{scalar} = insertvalue %phpc.NativeScalarValue {tagged}, i8 {bool_value}, 1"
            ));
            scalar
        } else if let Some(int_value) = int_value {
            let scalar = self.next_temp();
            self.body.push(format!(
                "{scalar} = insertvalue %phpc.NativeScalarValue {tagged}, i64 {int_value}, 3"
            ));
            scalar
        } else if let Some(float_value) = float_value {
            let scalar = self.next_temp();
            self.body.push(format!(
                "{scalar} = insertvalue %phpc.NativeScalarValue {tagged}, double {float_value}, 4"
            ));
            scalar
        } else {
            tagged
        };

        let runtime_value = self.next_temp();
        self.body.push(format!(
            "{runtime_value} = call %phpc.NativeValueHandle @phpc_native_value_from_scalar(%phpc.NativeScalarValue {scalar})"
        ));
        runtime_value
    }

    fn native_string_pointer_and_len(&mut self, value: &IrValue) -> Option<(String, String)> {
        match value {
            IrValue::String(value) => {
                let global = self.add_string(value);
                Some((format!("@{global}"), value.len().to_string()))
            }
            IrValue::StringPtr(value) => {
                let len = self.string_lengths.get(value)?.clone();
                Some((value.clone(), len))
            }
            IrValue::NativeExpression { fallback, .. } => {
                self.native_string_pointer_and_len(fallback)
            }
            _ => None,
        }
    }

    fn emit_assignment(&mut self, target: &AssignTarget, expr: &Expr) -> CompileResult<()> {
        if let Some(superglobal_span) = request_superglobal_consumed_assign_target_span(target)
            .or_else(|| request_superglobal_consumed_expr_span(expr))
        {
            return Err(self.unsupported(superglobal_span, LLVM_REQUEST_SUPERGLOBAL_REJECTION));
        }

        match target {
            AssignTarget::Variable { name, .. } => self.emit_direct_local_assignment(name, expr),
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
        let left = left.into_static_fallback();
        let right = right.into_static_fallback();
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
        let left = self
            .emit_expr(left)
            .map_err(|_| self.unsupported(span, llvm_comparison_rejection()))?;
        let right = self
            .emit_expr(right)
            .map_err(|_| self.unsupported(span, llvm_comparison_rejection()))?;
        self.emit_scalar_comparison(left, op, right, span)
    }

    fn emit_scalar_comparison(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        let left = left.into_static_fallback();
        let right = right.into_static_fallback();
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
            let right = self.emit_expr(right)?;
            return self.emit_empty_string_concat_identity(right, span);
        }
        if is_empty_string_literal(right) {
            let left = self.emit_expr(left)?;
            return self.emit_empty_string_concat_identity(left, span);
        }
        let left = self.emit_static_string_concat_operand(left, span)?;
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
            _ => Err(self.unsupported(span, LLVM_CONCAT_REJECTION)),
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
        let left = left.into_static_fallback();
        let right = right.into_static_fallback();
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
            value @ (IrValue::Bool(_) | IrValue::BoolExpr(_)) => Ok(value),
            _ => Err(self.unsupported(span, llvm_logical_rejection())),
        }
    }

    fn known_truthiness_for_value(&self, value: &IrValue) -> Option<bool> {
        match value {
            IrValue::Bool(value) => Some(*value),
            IrValue::BoolExpr(_) => None,
            IrValue::Int(value) => known_integer_truthiness(&self.known_integer_values(value)),
            IrValue::Float(value) => known_float_truthiness(&self.known_float_values(value)),
            IrValue::String(value) => Some(php_string_truthy(value)),
            IrValue::StringPtr(value) => self
                .known_string_values(value)
                .and_then(|values| known_string_truthiness(&values)),
            IrValue::NativeExpression { .. } => None,
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
            if !known_strings_are_safe_for_native_comparison(&left_values)
                || !known_strings_are_safe_for_native_comparison(&right_values)
            {
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
                if php_string_truthy(&value) {
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
            IrValue::NativeExpression { value, .. } => {
                let condition = self.emit_native_expression_truthiness(&value);
                self.emit_dynamic_ternary(IrValue::BoolExpr(condition), if_true, if_false, span)
            }
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
        if matches!(condition_value, IrValue::NativeExpression { .. }) {
            let if_true = self.emit_expr(if_true)?;
            let if_false = self.emit_expr(if_false)?;
            return self.emit_ternary(condition_value, if_true, if_false, span);
        }
        let condition_value = condition_value.into_static_fallback();
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
        let condition_value = if matches!(condition_value, IrValue::NativeExpression { .. }) {
            return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
        } else {
            condition_value.into_static_fallback()
        };
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
                if php_string_truthy(&value) {
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
            condition @ IrValue::BoolExpr(_) => {
                let if_false = self.emit_expr(if_false)?;
                if !matches!(if_false, IrValue::Bool(_) | IrValue::BoolExpr(_)) {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                }
                self.emit_ternary(condition, IrValue::Bool(true), if_false, span)
            }
            IrValue::NativeExpression { .. } => {
                Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION))
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

    fn emit_numeric_negate(&mut self, value: IrValue, span: Span) -> CompileResult<IrValue> {
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
            IrValue::String(value) => Ok(IrValue::Bool(!php_string_truthy(&value))),
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
            IrValue::NativeExpression { value, .. } => Ok(IrValue::BoolExpr(
                self.emit_native_expression_liveness_call("empty", &value),
            )),
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
            value @ IrValue::String(_) => {
                let _ = self.emit_native_value_result_stdout(&value);
            }
            IrValue::StringPtr(value) => {
                if let Some(len) = self.known_string_pointer_byte_len_operand(&value) {
                    let native_value = IrValue::StringPtr(value.clone());
                    if !self.emit_native_value_result_stdout(&native_value) {
                        self.emit_native_value_string_ptr_stdout(&value, &len);
                    }
                } else {
                    self.body.push(format!(
                        "call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr {value})"
                    ));
                }
            }
            IrValue::NativeExpression { value, .. } => {
                let materialized = self.emit_native_expression_read_value(&value);
                self.emit_native_value_handle_stdout(materialized);
            }
        }
    }

    fn emit_print(&mut self, value: IrValue) {
        self.emit_echo(value);
    }

    fn emit_native_value_result_stdout(&mut self, value: &IrValue) -> bool {
        let Some(materialized) = self.materialize_native_value_handle(value).into_available()
        else {
            return false;
        };

        self.emit_native_value_handle_stdout(materialized);
        true
    }

    fn emit_native_value_handle_stdout(&mut self, materialized: NativeValueMaterialization) {
        self.uses_native_value_echo_stdout = true;
        let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
        self.body.push(format!(
            "call {usize_type} @phpc_native_value_echo_stdout(%phpc.NativeValueHandle {})",
            materialized.handle
        ));
        self.body.extend(materialized.cleanup_after_use());
    }

    fn emit_native_value_string_ptr_stdout(&mut self, value: &str, len: &str) {
        let usize_type = NativeRuntimeIrTarget::host().usize_ir_type();
        self.emit_native_value_string_pointer_stdout(value, usize_type, len);
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

fn native_executable_symbol_table_helpers() -> &'static str {
    r#"typedef struct {
  int tag;
  int bool_value;
  long long int_value;
  phpc_NativeValueHandle runtime_value;
} phpc_NativeLinkedValue;

static phpc_NativeLinkedValue phpc_native_linked_value_from_bool(int value) {
  return (phpc_NativeLinkedValue){ 1, value ? 1 : 0, 0, {0} };
}

static phpc_NativeLinkedValue phpc_native_linked_value_from_int(long long value) {
  return (phpc_NativeLinkedValue){ 2, 0, value, {0} };
}

static phpc_NativeLinkedValue phpc_native_linked_value_from_runtime_value(
  phpc_NativeValueHandle value
) {
  return (phpc_NativeLinkedValue){ 3, 0, 0, value };
}

static void phpc_native_linked_value_echo_stdout(phpc_NativeLinkedValue value) {
  switch (value.tag) {
    case 1:
      if (value.bool_value) {
        printf("%s", "1");
      }
      return;
    case 2:
      printf("%lld", value.int_value);
      return;
    case 3:
      phpc_native_value_echo_stdout(value.runtime_value);
      return;
    default:
      return;
  }
}

typedef struct {
  const uint8_t *name;
  size_t name_len;
  phpc_NativeLinkedValue value;
} phpc_NativeSymbolEntry;

typedef struct {
  phpc_NativeSymbolEntry *entries;
  size_t len;
  size_t cap;
} phpc_NativeSymbolTable;

static phpc_NativeSymbolTableHandle phpc_native_symbol_table_new(void) {
  phpc_NativeSymbolTable *table = (phpc_NativeSymbolTable *)calloc(1, sizeof(*table));
  return (phpc_NativeSymbolTableHandle){ table };
}

static int phpc_native_symbol_name_matches(
  const phpc_NativeSymbolEntry *entry,
  const uint8_t *name,
  size_t name_len
) {
  return entry->name_len == name_len
    && (name_len == 0 || memcmp(entry->name, name, name_len) == 0);
}

static int phpc_native_symbol_table_write(
  phpc_NativeSymbolTableHandle handle,
  const uint8_t *name,
  size_t name_len,
  phpc_NativeLinkedValue value
) {
  phpc_NativeSymbolTable *table = (phpc_NativeSymbolTable *)handle.ptr;
  if (table == NULL || (name == NULL && name_len != 0)) {
    return 0;
  }

  for (size_t i = 0; i < table->len; i++) {
    if (phpc_native_symbol_name_matches(&table->entries[i], name, name_len)) {
      table->entries[i].value = value;
      return 1;
    }
  }

  if (table->len == table->cap) {
    size_t next_cap = table->cap == 0 ? 4 : table->cap * 2;
    phpc_NativeSymbolEntry *entries =
      (phpc_NativeSymbolEntry *)realloc(table->entries, next_cap * sizeof(*entries));
    if (entries == NULL) {
      return 0;
    }
    table->entries = entries;
    table->cap = next_cap;
  }

  table->entries[table->len++] = (phpc_NativeSymbolEntry){ name, name_len, value };
  return 1;
}

static phpc_NativeLinkedValue phpc_native_symbol_table_read(
  phpc_NativeSymbolTableHandle handle,
  const uint8_t *name,
  size_t name_len
) {
  phpc_NativeSymbolTable *table = (phpc_NativeSymbolTable *)handle.ptr;
  if (table == NULL || (name == NULL && name_len != 0)) {
    return (phpc_NativeLinkedValue){0};
  }

  for (size_t i = 0; i < table->len; i++) {
    if (phpc_native_symbol_name_matches(&table->entries[i], name, name_len)) {
      return table->entries[i].value;
    }
  }

  return (phpc_NativeLinkedValue){0};
}

static int phpc_native_symbol_table_isset(
  phpc_NativeSymbolTableHandle handle,
  const uint8_t *name,
  size_t name_len
) {
  phpc_NativeSymbolTable *table = (phpc_NativeSymbolTable *)handle.ptr;
  if (table == NULL || (name == NULL && name_len != 0)) {
    return 0;
  }

  for (size_t i = 0; i < table->len; i++) {
    if (phpc_native_symbol_name_matches(&table->entries[i], name, name_len)) {
      return table->entries[i].value.tag != 0;
    }
  }

  return 0;
}

static int phpc_native_symbol_table_unset(
  phpc_NativeSymbolTableHandle handle,
  const uint8_t *name,
  size_t name_len
) {
  phpc_NativeSymbolTable *table = (phpc_NativeSymbolTable *)handle.ptr;
  if (table == NULL || (name == NULL && name_len != 0)) {
    return 0;
  }

  for (size_t i = 0; i < table->len; i++) {
    if (phpc_native_symbol_name_matches(&table->entries[i], name, name_len)) {
      table->entries[i] = table->entries[table->len - 1];
      table->len--;
      return 1;
    }
  }

  return 1;
}

static void phpc_native_symbol_table_free(phpc_NativeSymbolTableHandle handle) {
  phpc_NativeSymbolTable *table = (phpc_NativeSymbolTable *)handle.ptr;
  if (table == NULL) {
    return;
  }
  free(table->entries);
  free(table);
}

"#
}

#[derive(Default)]
struct CGenerator {
    body: Vec<String>,
    static_data: Vec<String>,
    variables: HashMap<String, CValue>,
    array_variables: HashMap<String, String>,
    array_cleanup_handles: Vec<String>,
    known_ints: HashMap<String, KnownInt>,
    known_floats: HashMap<String, KnownFloat>,
    known_strings: HashMap<String, KnownString>,
    known_bools: HashMap<String, KnownBool>,
    symbol_table_variables: HashSet<String>,
    uses_strcmp: bool,
    uses_native_string_helpers: bool,
    uses_native_symbol_table_helpers: bool,
    emitted_native_symbol_table: bool,
    next_static_data: usize,
    next_native_temp: usize,
}

#[derive(Debug, Clone)]
enum CValue {
    Int(String),
    Float(String),
    String(String),
    StringExpr(String),
    Bool(bool),
    BoolExpr(String),
    Null,
}

#[derive(Debug, Clone, Copy)]
enum NativeValueDebugOutputOperation {
    VarDump,
    PrintR,
}

impl NativeValueDebugOutputOperation {
    fn operation_tag(self) -> u8 {
        match self {
            Self::VarDump => 0,
            Self::PrintR => 1,
        }
    }
}

struct CNativeValueMaterialization {
    handle: String,
    cleanup_after_use: Vec<String>,
}

struct CNativeArrayKeyMaterialization {
    result: String,
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
    fn emit_program(&mut self, program: &Program) -> CompileResult<String> {
        for stmt in &program.statements {
            self.emit_statement(stmt)?;
        }

        let mut output = String::new();
        if self.uses_native_string_helpers {
            output.push_str("/* generated by phpc native executable C link path */\n");
        } else {
            output.push_str("/* generated by phpc milestone 1 C assembly fallback */\n");
        }
        output.push_str("#include <stdio.h>\n\n");
        if self.uses_native_string_helpers {
            output.push_str("#include <stddef.h>\n");
            output.push_str("#include <stdint.h>\n\n");
            output.push_str("#include <stdbool.h>\n\n");
            output.push_str("typedef struct { uint8_t tag; uint8_t bool_value; int64_t int_value; double float_value; } phpc_NativeScalarValue;\n");
            output.push_str("typedef struct { void *ptr; } phpc_NativeStringHandle;\n");
            output.push_str("typedef struct { void *ptr; } phpc_NativeValueHandle;\n");
            output.push_str("typedef struct { void *ptr; } phpc_NativeDiagnosticHandle;\n\n");
            output.push_str(
                "typedef struct { uint8_t *ptr; size_t len; size_t cap; } phpc_NativeByteBuffer;\n",
            );
            output.push_str("typedef struct { void *ptr; } phpc_NativeArrayHandle;\n");
            output.push_str("typedef struct { uint8_t tag; int64_t int_value; phpc_NativeByteBuffer bytes; phpc_NativeDiagnosticHandle diagnostic; } phpc_NativeArrayKeyMaterializationResult;\n\n");
            output.push_str("extern phpc_NativeScalarValue phpc_native_null(void);\n");
            output.push_str("extern phpc_NativeScalarValue phpc_native_bool(bool value);\n");
            output.push_str("extern phpc_NativeScalarValue phpc_native_int(int64_t value);\n");
            output.push_str("extern phpc_NativeScalarValue phpc_native_float(double value);\n");
            output.push_str("extern phpc_NativeValueHandle phpc_native_value_from_scalar(phpc_NativeScalarValue value);\n");
            if self.uses_native_symbol_table_helpers {
                output.push_str("#include <stdlib.h>\n");
                output.push_str("#include <string.h>\n\n");
                output.push_str("typedef struct { void *ptr; } phpc_NativeSymbolTableHandle;\n\n");
            }
            output.push_str("extern phpc_NativeStringHandle phpc_native_string_from_bytes(const uint8_t *ptr, size_t len);\n");
            output.push_str("extern phpc_NativeValueHandle phpc_native_value_from_string_with_diagnostic(phpc_NativeStringHandle string, phpc_NativeDiagnosticHandle *diagnostic);\n");
            output.push_str("extern phpc_NativeValueHandle phpc_native_value_debug_output_with_diagnostic(phpc_NativeValueHandle value, uint8_t operation, bool return_output, phpc_NativeDiagnosticHandle *diagnostic);\n");
            output.push_str(
                "extern size_t phpc_native_value_echo_stdout(phpc_NativeValueHandle value);\n",
            );
            output.push_str("extern void phpc_native_value_free(phpc_NativeValueHandle value);\n");
            output.push_str("extern size_t phpc_native_diagnostic_message_stderr(phpc_NativeDiagnosticHandle diagnostic);\n");
            output.push_str("extern void phpc_native_diagnostic_free(phpc_NativeDiagnosticHandle diagnostic);\n");
            output.push_str(
                "extern void phpc_native_string_free(phpc_NativeStringHandle string);\n\n",
            );
            output.push_str("extern phpc_NativeArrayHandle phpc_native_array_empty(void);\n");
            output.push_str("extern bool phpc_native_array_append_value(phpc_NativeArrayHandle array, phpc_NativeValueHandle value);\n");
            output.push_str("extern phpc_NativeArrayKeyMaterializationResult phpc_native_value_to_array_key(phpc_NativeValueHandle value);\n");
            output.push_str("extern bool phpc_native_array_insert_key_value_with_diagnostic(phpc_NativeArrayHandle array, phpc_NativeArrayKeyMaterializationResult key, phpc_NativeValueHandle value, phpc_NativeDiagnosticHandle *diagnostic);\n");
            output.push_str("extern phpc_NativeValueHandle phpc_native_array_read_key_with_diagnostic(phpc_NativeArrayHandle array, phpc_NativeArrayKeyMaterializationResult key, phpc_NativeDiagnosticHandle *diagnostic);\n");
            output.push_str("extern void phpc_native_array_key_materialization_result_free(phpc_NativeArrayKeyMaterializationResult key);\n");
            output.push_str("extern phpc_NativeValueHandle phpc_native_value_from_array_clone(phpc_NativeArrayHandle array);\n");
            output
                .push_str("extern void phpc_native_array_free(phpc_NativeArrayHandle array);\n\n");
            if self.uses_native_symbol_table_helpers {
                output.push_str(native_executable_symbol_table_helpers());
            }
        }
        if self.uses_strcmp && !self.uses_native_symbol_table_helpers {
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
        for handle in self.array_cleanup_handles.iter().rev() {
            output.push_str("  phpc_native_array_free(");
            output.push_str(handle);
            output.push_str(");\n");
        }
        if self.emitted_native_symbol_table {
            output.push_str("  phpc_native_symbol_table_free(phpc_symbols);\n");
        }
        output.push_str("  return 0;\n");
        output.push_str("}\n");
        Ok(output)
    }

    fn emit_statement(&mut self, stmt: &Stmt) -> CompileResult<()> {
        match stmt {
            Stmt::Namespace { span, .. } | Stmt::Use { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_NAMESPACE_REJECTION))
            }
            Stmt::Echo { exprs, .. } => {
                for expr in exprs {
                    if self.try_emit_native_symbol_table_echo(expr)? {
                        continue;
                    }
                    if self.uses_native_string_helpers && self.emit_array_index_echo(expr)? {
                        continue;
                    }
                    let value = self.emit_expr(expr)?;
                    self.emit_echo(value)?;
                }
                Ok(())
            }
            Stmt::Print { expr, .. } => {
                if self.try_emit_native_symbol_table_echo(expr)? {
                    return Ok(());
                }
                if self.uses_native_string_helpers && self.emit_array_index_echo(expr)? {
                    return Ok(());
                }
                let value = self.emit_expr(expr)?;
                self.emit_echo(value)?;
                Ok(())
            }
            Stmt::Assign { target, expr, .. } => self.emit_assignment(target, expr),
            Stmt::ReferenceAssign { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION))
            }
            Stmt::CompoundAssign {
                target, expr, span, ..
            }
            | Stmt::NullCoalesceAssign {
                target, expr, span, ..
            } => {
                if let Some(superglobal_span) =
                    request_superglobal_consumed_assign_target_span(target)
                {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(expr) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Stmt::IncrementDecrement { target, span, .. } => {
                if let Some(superglobal_span) =
                    request_superglobal_consumed_assign_target_span(target)
                {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Stmt::Expr { expr, .. } => {
                self.emit_expr(expr)?;
                Ok(())
            }
            Stmt::Function(function) => {
                if let Some(span) = find_static_local_span(&function.body) {
                    return Err(self.unsupported(span, ASSEMBLY_STATIC_LOCAL_REJECTION));
                }
                Err(self.unsupported(function.span, ASSEMBLY_FUNCTION_DECLARATION_REJECTION))
            }
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
            Stmt::Foreach { span, .. } => Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION)),
            Stmt::UnsetVariable { name, span } => self.emit_unset_variable(name, *span),
            Stmt::UnsetStaticProperty { span, .. }
            | Stmt::UnsetSelfStaticProperty { span, .. }
            | Stmt::UnsetParentStaticProperty { span, .. }
            | Stmt::UnsetLateStaticProperty { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Stmt::UnsetObjectProperty { span, .. }
            | Stmt::UnsetDynamicObjectProperty { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Stmt::UnsetArrayIndex { span, .. } | Stmt::UnsetNestedArrayIndex { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            Stmt::UnsetMany { targets, span } => self.emit_unset_many(targets, *span),
            Stmt::ConstDeclaration { declarations, span } => {
                if let Some(superglobal_span) = declarations.iter().find_map(|declaration| {
                    request_superglobal_consumed_expr_span(&declaration.value)
                }) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION))
            }
            Stmt::Require { path, span, .. } | Stmt::Include { path, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(path) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_REQUIRE_REJECTION))
            }
            Stmt::Throw { expr, span } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(expr) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_EXCEPTION_REJECTION))
            }
            Stmt::Try { span, .. } => Err(self.unsupported(*span, ASSEMBLY_TRY_BLOCK_REJECTION)),
            Stmt::Return { value, span } => {
                if let Some(value) = value {
                    if let Some(superglobal_span) = request_superglobal_consumed_expr_span(value) {
                        return Err(self.unsupported(
                            superglobal_span,
                            ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION,
                        ));
                    }
                }
                Err(self.unsupported(*span, ASSEMBLY_FUNCTION_DECLARATION_REJECTION))
            }
            Stmt::Global { names, span } => {
                if names.iter().any(|name| is_request_superglobal_name(name)) {
                    return Err(self.unsupported(*span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION));
                }
                Err(self.unsupported(*span, ASSEMBLY_GLOBAL_DECLARATION_REJECTION))
            }
            Stmt::StaticLocal { declarations, span } => {
                if let Some(superglobal_span) = declarations.iter().find_map(|declaration| {
                    declaration
                        .default
                        .as_ref()
                        .and_then(request_superglobal_consumed_expr_span)
                }) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
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
            Expr::ObjectStaticProperty { target, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(target) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_STATIC_MEMBER_REJECTION))
            }
            Expr::Array { items, span } => {
                if let Some(superglobal_span) = items.iter().find_map(|item| {
                    item.key
                        .as_ref()
                        .and_then(request_superglobal_consumed_expr_span)
                        .or_else(|| request_superglobal_consumed_expr_span(&item.value))
                }) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            Expr::Index { target, span, .. } => {
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
            Expr::Property { target, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(target) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_OBJECT_PROPERTY_REJECTION))
            }
            Expr::DynamicProperty {
                target,
                property,
                span,
            } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(target)
                    .or_else(|| request_superglobal_consumed_expr_span(property))
                {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_OBJECT_PROPERTY_REJECTION))
            }
            Expr::MethodCall {
                target, args, span, ..
            } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(target)
                    .or_else(|| request_superglobal_consumed_args_span(args))
                {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_METHOD_CALL_REJECTION))
            }
            Expr::DynamicMethodCall {
                target,
                method,
                args,
                span,
            } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(target)
                    .or_else(|| request_superglobal_consumed_expr_span(method))
                    .or_else(|| request_superglobal_consumed_args_span(args))
                {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_METHOD_CALL_REJECTION))
            }
            Expr::ObjectStaticMethodCall {
                target, args, span, ..
            } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(target)
                    .or_else(|| request_superglobal_consumed_args_span(args))
                {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_METHOD_CALL_REJECTION))
            }
            Expr::ParentMethodCall { args, span, .. }
            | Expr::StaticMethodCall { args, span, .. }
            | Expr::SelfMethodCall { args, span, .. }
            | Expr::LateStaticMethodCall { args, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_METHOD_CALL_REJECTION))
            }
            Expr::Variable(name, span) => {
                if is_request_superglobal_name(name) {
                    return Err(self.unsupported(*span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION));
                }
                self.variables
                    .get(name)
                    .cloned()
                    .ok_or_else(|| self.unsupported(*span, ASSEMBLY_VARIABLE_READ_REJECTION))
            }
            Expr::Call { name, args, span } if is_exit_construct_name(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_TERMINATION_REJECTION))
            }
            Expr::Call { name, args, span } if is_value_debug_output_builtin(name) => {
                self.emit_value_debug_output_call(name, args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("defined") => {
                self.emit_defined_call(args, *span)
            }
            Expr::Call { name, args, span } if is_global_constant_builtin(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION))
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
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("str_starts_with") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_STR_STARTS_WITH_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("str_ends_with") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_STR_ENDS_WITH_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("basename") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_BASENAME_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("file_get_contents") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_FILE_GET_CONTENTS_REJECTION))
            }
            Expr::Call { name, args, span } if is_stream_resource_builtin(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_STREAM_RESOURCE_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("getcwd") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_GETCWD_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("realpath") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_REALPATH_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("is_writable") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_IS_WRITABLE_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("clearstatcache") => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_CLEARSTATCACHE_REJECTION))
            }
            Expr::Call { name, args, span } if is_header_state_builtin(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_HEADER_STATE_REJECTION))
            }
            Expr::Call { name, args, span } if is_session_state_builtin(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_SESSION_STATE_REJECTION))
            }
            Expr::Call { name, args, span } if is_output_buffer_builtin(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_OUTPUT_BUFFER_REJECTION))
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
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_OBJECT_METADATA_REJECTION))
            }
            Expr::Call { name, args, span } if is_array_builtin(name) => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            Expr::DynamicCall { callee, args, span } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(callee)
                    .or_else(|| request_superglobal_consumed_args_span(args))
                {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION))
            }
            Expr::Call { args, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_args_span(args) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_FUNCTION_CALL_REJECTION))
            }
            Expr::InstanceOf { expr, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(expr) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_INSTANCEOF_REJECTION))
            }
            Expr::Closure { span, .. } => Err(self.unsupported(*span, ASSEMBLY_CLOSURE_REJECTION)),
            Expr::New {
                class_name,
                args,
                span,
            } => {
                if let Some(superglobal_span) =
                    request_superglobal_consumed_new_class_name_span(class_name, *span)
                        .or_else(|| request_superglobal_consumed_args_span(args))
                {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_OBJECT_INSTANTIATION_REJECTION))
            }
            Expr::Clone { expr, span } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(expr) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
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
                        let value = self.emit_expr(expr)?;
                        if matches!(value, CValue::Bool(_) | CValue::BoolExpr(_)) {
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
                        return match self.emit_expr(expr)? {
                            value @ CValue::Int(_) => Ok(value),
                            _ => Err(self.unsupported(*span, ASSEMBLY_BITWISE_REJECTION)),
                        };
                    }
                }
                let value = self.emit_expr(expr)?;
                self.emit_unary(*op, value, *span)
            }
            Expr::ErrorControl { expr, span } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(expr) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_ERROR_CONTROL_REJECTION))
            }
            Expr::Include { path, span, .. } | Expr::Require { path, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(path) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_REQUIRE_EXPRESSION_REJECTION))
            }
            Expr::Cast { expr, span, .. } => {
                if let Some(superglobal_span) = request_superglobal_consumed_expr_span(expr) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                Err(self.unsupported(*span, ASSEMBLY_CAST_REJECTION))
            }
            Expr::Assign { target, expr, span } => {
                if let Some(superglobal_span) =
                    request_superglobal_consumed_assign_target_span(target)
                        .or_else(|| request_superglobal_consumed_expr_span(expr))
                {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
                if is_static_member_assign_target(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_STATIC_MEMBER_REJECTION));
                }
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Expr::CompoundAssign {
                target, expr, span, ..
            }
            | Expr::NullCoalesceAssign {
                target, expr, span, ..
            } => {
                if let Some(superglobal_span) =
                    request_superglobal_consumed_assign_target_span(target)
                        .or_else(|| request_superglobal_consumed_expr_span(expr))
                {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Expr::IncrementDecrement { target, span, .. } => {
                if let Some(superglobal_span) =
                    request_superglobal_consumed_assign_target_span(target)
                {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }
                if is_object_property_array_access_target(target) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
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
                let left = self.emit_expr(left)?;
                let right = self.emit_expr(right)?;
                self.emit_binary(left, *op, right, *span)
            }
        }
    }

    fn emit_isset_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        if args.is_empty() {
            return Err(self.unsupported(span, ASSEMBLY_ISSET_REJECTION));
        }

        let mut symbol_table_checks = Vec::new();
        for arg in args {
            if let Some(superglobal_span) = request_superglobal_expr_span(arg) {
                return Err(
                    self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                );
            }

            if is_array_access_offset_expr(arg) {
                return Err(self.unsupported(arg.span(), ASSEMBLY_ARRAY_ACCESS_REJECTION));
            }

            let Expr::Variable(name, _) = arg else {
                return Err(self.unsupported(arg.span(), ASSEMBLY_ISSET_REJECTION));
            };

            if self.uses_native_string_helpers
                && (self.symbol_table_variables.contains(name)
                    || !self.variables.contains_key(name))
            {
                symbol_table_checks.push(self.emit_native_symbol_table_isset_expr(name));
                continue;
            }

            if matches!(self.variables.get(name), None | Some(CValue::Null)) {
                return Ok(CValue::Bool(false));
            }
        }

        if symbol_table_checks.is_empty() {
            return Ok(CValue::Bool(true));
        }

        Ok(CValue::BoolExpr(
            symbol_table_checks
                .into_iter()
                .map(|check| format!("({check})"))
                .collect::<Vec<_>>()
                .join(" && "),
        ))
    }

    fn emit_empty_call(&self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        let [arg] = args else {
            return Err(self.unsupported(span, ASSEMBLY_EMPTY_REJECTION));
        };

        if let Some(superglobal_span) = request_superglobal_expr_span(arg) {
            return Err(self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION));
        }

        if is_array_access_offset_expr(arg) {
            return Err(self.unsupported(arg.span(), ASSEMBLY_ARRAY_ACCESS_REJECTION));
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

    fn emit_strlen_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        if args.len() != 1 {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        self.strlen_result_for_value(&value)
            .map(|length| CValue::Int(length.to_string()))
            .ok_or_else(|| self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION))
    }

    fn emit_function_exists_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        if args.len() != 1 {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        self.function_exists_result_for_value(&value)
            .map(CValue::Bool)
            .ok_or_else(|| self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION))
    }

    fn emit_is_callable_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        if !(1..=2).contains(&args.len()) {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        let syntax_only = if let Some(arg) = args.get(1) {
            match self.emit_expr(arg)? {
                CValue::Bool(value) => value,
                _ => return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION)),
            }
        } else {
            false
        };

        self.is_callable_result_for_value(&value, syntax_only)
            .map(CValue::Bool)
            .ok_or_else(|| self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION))
    }

    fn emit_value_debug_output_call(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<CValue> {
        if !self.uses_native_string_helpers {
            return Err(self.unsupported(span, ASSEMBLY_VALUE_DEBUG_OUTPUT_REJECTION));
        }

        if name.eq_ignore_ascii_case("var_dump") {
            if args.is_empty() {
                return Err(self.unsupported(span, ASSEMBLY_VALUE_DEBUG_OUTPUT_REJECTION));
            }

            for arg in args {
                let value = self.materialize_native_debug_output_value(arg)?;
                self.emit_native_value_debug_output(
                    value,
                    NativeValueDebugOutputOperation::VarDump,
                );
            }
            return Ok(CValue::Null);
        }

        if !name.eq_ignore_ascii_case("print_r") || !(1..=2).contains(&args.len()) {
            return Err(self.unsupported(span, ASSEMBLY_VALUE_DEBUG_OUTPUT_REJECTION));
        }

        let return_output = if let Some(arg) = args.get(1) {
            let value = self.emit_expr(arg)?;
            self.known_truthiness_for_value(&value).ok_or_else(|| {
                self.unsupported(arg.span(), ASSEMBLY_VALUE_DEBUG_OUTPUT_REJECTION)
            })?
        } else {
            false
        };

        if return_output {
            return Err(self.unsupported(span, ASSEMBLY_VALUE_DEBUG_OUTPUT_REJECTION));
        }

        let value = self.materialize_native_debug_output_value(&args[0])?;
        self.emit_native_value_debug_output(value, NativeValueDebugOutputOperation::PrintR);
        Ok(CValue::Bool(true))
    }

    fn materialize_native_debug_output_value(
        &mut self,
        expr: &Expr,
    ) -> CompileResult<CNativeValueMaterialization> {
        match expr {
            Expr::Array { items, span } => {
                if let Some(superglobal_span) = items.iter().find_map(|item| {
                    item.key
                        .as_ref()
                        .and_then(request_superglobal_consumed_expr_span)
                        .or_else(|| request_superglobal_consumed_expr_span(&item.value))
                }) {
                    return Err(
                        self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION)
                    );
                }

                let handle = self.emit_array_literal(items, *span)?;
                return Ok(self.materialize_native_array_value_clone(&handle));
            }
            Expr::Variable(name, span) => {
                if is_request_superglobal_name(name) {
                    return Err(self.unsupported(*span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION));
                }
                if let Some(handle) = self.array_variables.get(name).cloned() {
                    return Ok(self.materialize_native_array_value_clone(&handle));
                }
            }
            _ => {}
        }

        let value = self.emit_expr(expr)?;
        self.materialize_native_c_value_handle(&value, expr.span())
    }

    fn materialize_native_array_value_clone(
        &mut self,
        handle: &str,
    ) -> CNativeValueMaterialization {
        let value = self.next_native_name("debug_value");
        self.body.push(format!(
            "phpc_NativeValueHandle {value} = phpc_native_value_from_array_clone({handle});"
        ));
        CNativeValueMaterialization {
            handle: value.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({value});")],
        }
    }

    fn materialize_native_c_value_handle(
        &mut self,
        value: &CValue,
        span: Span,
    ) -> CompileResult<CNativeValueMaterialization> {
        self.materialize_native_c_value_handle_with_rejection(
            value,
            span,
            ASSEMBLY_VALUE_DEBUG_OUTPUT_REJECTION,
            "",
        )
    }

    fn materialize_native_array_c_value_handle(
        &mut self,
        value: &CValue,
        span: Span,
        failure_cleanup: &str,
    ) -> CompileResult<CNativeValueMaterialization> {
        self.materialize_native_c_value_handle_with_rejection(
            value,
            span,
            ASSEMBLY_ARRAY_REJECTION,
            failure_cleanup,
        )
    }

    fn materialize_native_c_value_handle_with_rejection(
        &mut self,
        value: &CValue,
        span: Span,
        rejection: &str,
        failure_cleanup: &str,
    ) -> CompileResult<CNativeValueMaterialization> {
        match value {
            CValue::Null => Ok(self.materialize_native_scalar_value_handle("phpc_native_null()")),
            CValue::Bool(value) => Ok(self.materialize_native_scalar_value_handle(&format!(
                "phpc_native_bool({})",
                if *value { "true" } else { "false" }
            ))),
            CValue::BoolExpr(value) => Ok(self.materialize_native_scalar_value_handle(&format!(
                "phpc_native_bool(({value}) ? true : false)"
            ))),
            CValue::Int(value) => Ok(self.materialize_native_scalar_value_handle(&format!(
                "phpc_native_int((int64_t)({value}))"
            ))),
            CValue::Float(value) => Ok(self.materialize_native_scalar_value_handle(&format!(
                "phpc_native_float((double)({value}))"
            ))),
            CValue::String(value) => {
                Ok(self.materialize_native_string_value_handle(value, failure_cleanup))
            }
            CValue::StringExpr(_) => Err(self.unsupported(span, rejection)),
        }
    }

    fn materialize_native_scalar_value_handle(
        &mut self,
        scalar: &str,
    ) -> CNativeValueMaterialization {
        let value = self.next_native_name("debug_value");
        self.body.push(format!(
            "phpc_NativeValueHandle {value} = phpc_native_value_from_scalar({scalar});"
        ));
        CNativeValueMaterialization {
            handle: value.clone(),
            cleanup_after_use: vec![format!("phpc_native_value_free({value});")],
        }
    }

    fn materialize_native_string_value_handle(
        &mut self,
        bytes: &str,
        failure_cleanup: &str,
    ) -> CNativeValueMaterialization {
        let string = self.emit_native_string_handle("debug_string", bytes);
        let diagnostic = self.next_native_name("debug_value_diagnostic");
        let value = self.next_native_name("debug_value");
        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {value} = phpc_native_value_from_string_with_diagnostic({string}, &{diagnostic});"
        ));
        let conversion_error_exit = self.native_error_exit(&format!(
            "phpc_native_diagnostic_message_stderr({diagnostic}); phpc_native_diagnostic_free({diagnostic}); phpc_native_string_free({string}); {failure_cleanup}"
        ));
        self.body.push(format!(
            "if ({value}.ptr == NULL) {{ {conversion_error_exit} }}"
        ));
        CNativeValueMaterialization {
            handle: value.clone(),
            cleanup_after_use: vec![
                format!("phpc_native_value_free({value});"),
                format!("phpc_native_string_free({string});"),
            ],
        }
    }

    fn emit_native_value_debug_output(
        &mut self,
        value: CNativeValueMaterialization,
        operation: NativeValueDebugOutputOperation,
    ) {
        let diagnostic = self.next_native_name("debug_output_diagnostic");
        let result = self.next_native_name("debug_output_result");
        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));
        self.body.push(format!(
            "phpc_NativeValueHandle {result} = phpc_native_value_debug_output_with_diagnostic({}, {}, false, &{diagnostic});",
            value.handle,
            operation.operation_tag()
        ));
        let local_cleanup = format!(
            "phpc_native_diagnostic_message_stderr({diagnostic}); phpc_native_diagnostic_free({diagnostic}); {}",
            value.cleanup_after_use.join(" ")
        );
        let error_exit = self.native_error_exit(&local_cleanup);
        self.body
            .push(format!("if ({result}.ptr == NULL) {{ {error_exit} }}"));
        self.body.push(format!("phpc_native_value_free({result});"));
        self.body.extend(value.cleanup_after_use);
    }

    fn emit_defined_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        if args.len() != 1 {
            return Err(self.unsupported(span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION));
        }
        if matches!(args[0], Expr::InterpolatedString { .. }) {
            return Err(self.unsupported(span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION));
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
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        match name.to_ascii_lowercase().as_str() {
            "gettype" => Ok(CValue::String(c_gettype_name(&value).to_string())),
            "get_debug_type" => Ok(CValue::String(c_debug_type_name(&value).to_string())),
            "is_null" => Ok(CValue::Bool(matches!(value, CValue::Null))),
            "is_bool" => Ok(CValue::Bool(matches!(
                value,
                CValue::Bool(_) | CValue::BoolExpr(_)
            ))),
            "is_int" | "is_integer" | "is_long" => {
                Ok(CValue::Bool(matches!(value, CValue::Int(_))))
            }
            "is_float" | "is_double" => Ok(CValue::Bool(matches!(value, CValue::Float(_)))),
            "is_string" => Ok(CValue::Bool(matches!(
                value,
                CValue::String(_) | CValue::StringExpr(_)
            ))),
            "is_array" => Ok(CValue::Bool(false)),
            "is_scalar" => Ok(CValue::Bool(matches!(
                value,
                CValue::Bool(_)
                    | CValue::BoolExpr(_)
                    | CValue::Int(_)
                    | CValue::Float(_)
                    | CValue::String(_)
                    | CValue::StringExpr(_)
            ))),
            "is_numeric" => self
                .is_numeric_result_for_value(&value)
                .map(CValue::Bool)
                .ok_or_else(|| self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION)),
            "is_countable" | "is_iterable" => Ok(CValue::Bool(false)),
            "extension_loaded" => match value {
                CValue::String(name) => Ok(CValue::Bool(is_compat_loaded_extension_name(&name))),
                CValue::StringExpr(_) => Ok(CValue::Bool(false)),
                _ => Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION)),
            },
            "is_object" => Ok(CValue::Bool(false)),
            _ => Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION)),
        }
    }

    fn emit_native_metadata_exists_call(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<CValue> {
        if !(1..=2).contains(&args.len()) {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        let name = self.emit_expr(&args[0])?;
        if !matches!(name, CValue::String(_) | CValue::StringExpr(_)) {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }
        if self.c_value_mentions_builtin_class(&name) {
            return Err(self.unsupported(span, ASSEMBLY_OBJECT_METADATA_REJECTION));
        }

        if let Some(autoload) = args.get(1) {
            let autoload = self.emit_expr(autoload)?;
            if !matches!(autoload, CValue::Bool(_) | CValue::BoolExpr(_)) {
                return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
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
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        let object_or_class = self.emit_expr(&args[0])?;
        let member = self.emit_expr(&args[1])?;
        if !matches!(object_or_class, CValue::String(_) | CValue::StringExpr(_))
            || !matches!(member, CValue::String(_) | CValue::StringExpr(_))
        {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
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
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        let object_or_class = self.emit_expr(&args[0])?;
        let class_name = self.emit_expr(&args[1])?;
        if !matches!(object_or_class, CValue::String(_) | CValue::StringExpr(_))
            || !matches!(class_name, CValue::String(_) | CValue::StringExpr(_))
        {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }
        if self.c_value_mentions_builtin_class(&object_or_class)
            || self.c_value_mentions_builtin_class(&class_name)
        {
            return Err(self.unsupported(span, ASSEMBLY_OBJECT_METADATA_REJECTION));
        }

        if let Some(allow_string) = args.get(2) {
            let allow_string = self.emit_expr(allow_string)?;
            if !matches!(allow_string, CValue::Bool(_) | CValue::BoolExpr(_)) {
                return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
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
            CValue::Null | CValue::Bool(_) | CValue::BoolExpr(_) => Some(false),
            CValue::String(value) => Some(is_php_numeric_string_literal(value)),
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
        if let Some(superglobal_span) = request_superglobal_consumed_assign_target_span(target)
            .or_else(|| request_superglobal_consumed_expr_span(expr))
        {
            return Err(self.unsupported(superglobal_span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION));
        }

        match target {
            AssignTarget::Variable { name, .. } => {
                if self.uses_native_string_helpers {
                    if let Expr::Array { items, span } = expr {
                        let handle = self.emit_array_literal(items, *span)?;
                        self.variables.remove(name);
                        self.symbol_table_variables.remove(name);
                        self.array_variables.insert(name.clone(), handle);
                        return Ok(());
                    }
                }
                let value = self.emit_expr(expr)?;
                self.array_variables.remove(name);
                if self.uses_native_string_helpers {
                    if self.emit_native_symbol_table_variable_copy(name, expr)? {
                        self.symbol_table_variables.insert(name.clone());
                    } else if self.emit_native_symbol_table_write(name, &value)? {
                        self.symbol_table_variables.insert(name.clone());
                    } else {
                        self.symbol_table_variables.remove(name);
                    }
                }
                self.variables.insert(name.clone(), value);
                Ok(())
            }
            AssignTarget::List { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_DESTRUCTURING_REJECTION))
            }
            AssignTarget::ArrayIndex { name, index, span } => {
                if self.uses_native_string_helpers {
                    if let Some(handle) = self.array_variables.get(name).cloned() {
                        if let Some(index) = index {
                            self.emit_array_write_key_value(&handle, index, expr)?;
                            return Ok(());
                        }

                        self.emit_array_append_value(&handle, expr)?;
                        return Ok(());
                    }
                }

                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            AssignTarget::NestedArrayIndex { span, .. }
            | AssignTarget::NestedArrayAppend { span, .. } => {
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

    fn emit_unset_variable(&mut self, name: &str, span: Span) -> CompileResult<()> {
        if !self.uses_native_string_helpers {
            return Err(self.unsupported(span, ASSEMBLY_MUTATION_REJECTION));
        }
        if is_request_superglobal_name(name) {
            return Err(self.unsupported(span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION));
        }

        if self.symbol_table_variables.contains(name) {
            self.emit_native_symbol_table_unset(name);
        }
        self.symbol_table_variables.remove(name);
        self.variables.remove(name);
        self.array_variables.remove(name);
        Ok(())
    }

    fn emit_unset_many(&mut self, targets: &[UnsetTarget], span: Span) -> CompileResult<()> {
        if targets
            .iter()
            .any(is_object_property_array_access_unset_target)
        {
            return Err(self.unsupported(span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
        }

        for target in targets {
            match target {
                UnsetTarget::Variable { name, span } => self.emit_unset_variable(name, *span)?,
                UnsetTarget::ArrayIndex { span, .. }
                | UnsetTarget::NestedArrayIndex { span, .. } => {
                    return Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION));
                }
                UnsetTarget::ObjectProperty { span, .. }
                | UnsetTarget::DynamicObjectProperty { span, .. }
                | UnsetTarget::NonDirectObjectProperty { span, .. }
                | UnsetTarget::NonDirectDynamicObjectProperty { span, .. }
                | UnsetTarget::StaticProperty { span, .. }
                | UnsetTarget::SelfStaticProperty { span, .. }
                | UnsetTarget::ParentStaticProperty { span, .. }
                | UnsetTarget::LateStaticProperty { span, .. } => {
                    return Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION));
                }
                UnsetTarget::ObjectPropertyArrayIndex { .. }
                | UnsetTarget::DynamicObjectPropertyArrayIndex { .. }
                | UnsetTarget::NonDirectObjectPropertyArrayIndex { .. }
                | UnsetTarget::NonDirectDynamicObjectPropertyArrayIndex { .. } => {
                    return Err(self.unsupported(span, ASSEMBLY_ARRAY_ACCESS_REJECTION));
                }
            }
        }

        Ok(())
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
                | CValue::String(_)
                | CValue::StringExpr(_),
                _,
            )
            | (
                _,
                CValue::Null
                | CValue::Bool(_)
                | CValue::BoolExpr(_)
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
        let left = self
            .emit_expr(left)
            .map_err(|_| self.unsupported(span, assembly_comparison_rejection()))?;
        let right = self
            .emit_expr(right)
            .map_err(|_| self.unsupported(span, assembly_comparison_rejection()))?;
        self.emit_scalar_comparison(left, op, right, span)
    }

    fn emit_scalar_comparison(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
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
            let right = self.emit_expr(right)?;
            return self.emit_empty_string_concat_identity(right, span);
        }
        if is_empty_string_literal(right) {
            let left = self.emit_expr(left)?;
            return self.emit_empty_string_concat_identity(left, span);
        }
        let left = self.emit_static_string_concat_operand(left, span)?;
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
            _ => Err(self.unsupported(span, ASSEMBLY_CONCAT_REJECTION)),
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
            CValue::Bool(_) | CValue::BoolExpr(_) => Ok(value),
            _ => Err(self.unsupported(span, assembly_logical_rejection())),
        }
    }

    fn known_truthiness_for_value(&self, value: &CValue) -> Option<bool> {
        match value {
            CValue::Bool(value) => Some(*value),
            CValue::BoolExpr(_) => None,
            CValue::Int(value) => known_integer_truthiness(&self.known_integer_values(value)),
            CValue::Float(value) => known_float_truthiness(&self.known_float_values(value)),
            CValue::String(value) => Some(php_string_truthy(value)),
            CValue::StringExpr(value) => self
                .known_string_values(value)
                .and_then(|values| known_string_truthiness(&values)),
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
            if !known_strings_are_safe_for_native_comparison(&left_values)
                || !known_strings_are_safe_for_native_comparison(&right_values)
            {
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
                if php_string_truthy(&value) {
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
        if !matches!(condition_value, CValue::BoolExpr(_)) {
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
                CValue::BoolExpr(_) | CValue::Int(_) | CValue::Float(_) | CValue::StringExpr(_)
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
                if php_string_truthy(&value) {
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
            condition @ CValue::BoolExpr(_) => {
                let if_false = self.emit_expr(if_false)?;
                if !matches!(if_false, CValue::Bool(_) | CValue::BoolExpr(_)) {
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
                    let if_true = c_string_operand(if_true);
                    let if_false = c_string_operand(if_false);
                    let expression = format!("(({condition}) ? ({if_true}) : ({if_false}))");
                    if let Some(result) = result {
                        self.known_strings.insert(expression.clone(), result);
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

    fn emit_numeric_negate(&mut self, value: CValue, span: Span) -> CompileResult<CValue> {
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
            CValue::String(value) => Ok(CValue::Bool(!php_string_truthy(&value))),
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

    fn emit_echo(&mut self, value: CValue) -> CompileResult<()> {
        match value {
            CValue::Null | CValue::Bool(false) => {}
            CValue::Bool(true) => self.body.push("printf(\"%s\", \"1\");".to_string()),
            CValue::BoolExpr(value) => self
                .body
                .push(format!("if ({value}) {{ printf(\"%s\", \"1\"); }}")),
            CValue::Int(value) => self.body.push(format!("printf(\"%lld\", {value});")),
            CValue::Float(value) => self.body.push(format!("printf(\"%g\", {value});")),
            CValue::String(value) => {
                if self.uses_native_string_helpers {
                    self.emit_native_string_helper_echo(&value);
                } else {
                    self.body
                        .push(format!("printf(\"%s\", \"{}\");", c_string(&value)));
                }
            }
            CValue::StringExpr(value) => {
                if self.uses_native_string_helpers {
                    return Err(Diagnostic::new(
                        Phase::Codegen,
                        0,
                        0,
                        format!("native executable string output only supports direct compile-time strings through runtime helpers; dynamic string pointer {value} is not linked yet"),
                    ));
                }
                self.body.push(format!("printf(\"%s\", {value});"));
            }
        }
        Ok(())
    }

    fn emit_array_literal(&mut self, items: &[ArrayItem], span: Span) -> CompileResult<String> {
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
        let value_span = value.span();
        let value = self.emit_expr(value)?;
        let value = self.materialize_native_array_c_value_handle(&value, value_span, "")?;
        let value_cleanup = c_cleanup_sequence(&value.cleanup_after_use);
        let key = self.materialize_native_array_key(key, &value_cleanup)?;
        let diagnostic = self.next_native_name("array_diagnostic");
        self.body
            .push(format!("phpc_NativeDiagnosticHandle {diagnostic} = {{0}};"));

        let local_cleanup = format!(
            "phpc_native_diagnostic_message_stderr({diagnostic}); phpc_native_diagnostic_free({diagnostic}); {}{}",
            c_cleanup_sequence(&key.cleanup_after_use),
            c_cleanup_sequence(&value.cleanup_after_use)
        );
        let write_error_exit = self.native_error_exit(&local_cleanup);
        self.body.push(format!(
            "if (!phpc_native_array_insert_key_value_with_diagnostic({handle}, {}, {}, &{diagnostic})) {{ {write_error_exit} }}",
            key.result, value.handle
        ));
        self.body.extend(key.cleanup_after_use);
        self.body.extend(value.cleanup_after_use);
        Ok(())
    }

    fn materialize_native_array_key(
        &mut self,
        key: &Expr,
        failure_cleanup: &str,
    ) -> CompileResult<CNativeArrayKeyMaterialization> {
        let key_value = self.emit_expr(key)?;
        let key_value =
            self.materialize_native_array_c_value_handle(&key_value, key.span(), failure_cleanup)?;
        let result = self.next_native_name("array_key");
        self.body.push(format!(
            "phpc_NativeArrayKeyMaterializationResult {result} = phpc_native_value_to_array_key({});",
            key_value.handle
        ));
        self.body.extend(key_value.cleanup_after_use);
        Ok(CNativeArrayKeyMaterialization {
            result: result.clone(),
            cleanup_after_use: vec![format!(
                "phpc_native_array_key_materialization_result_free({result});"
            )],
        })
    }

    fn emit_array_append_value(&mut self, handle: &str, value: &Expr) -> CompileResult<()> {
        let value_span = value.span();
        let value = self.emit_expr(value)?;
        let value = self.materialize_native_array_c_value_handle(&value, value_span, "")?;
        let local_cleanup = c_cleanup_sequence(&value.cleanup_after_use);
        let append_error_exit = self.native_error_exit(&local_cleanup);
        self.body.push(format!(
            "if (!phpc_native_array_append_value({handle}, {})) {{ {append_error_exit} }}",
            value.handle
        ));
        self.body.extend(value.cleanup_after_use);
        Ok(())
    }

    fn emit_array_index_echo(&mut self, expr: &Expr) -> CompileResult<bool> {
        let Expr::Index {
            target,
            index,
            span: _,
        } = expr
        else {
            return Ok(false);
        };

        let Expr::Variable(name, _) = target.as_ref() else {
            return Ok(false);
        };

        let Some(handle) = self.array_variables.get(name).cloned() else {
            return Ok(false);
        };

        let key = self.materialize_native_array_key(index.as_ref(), "")?;
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

    fn try_emit_native_symbol_table_echo(&mut self, expr: &Expr) -> CompileResult<bool> {
        if !self.uses_native_string_helpers {
            return Ok(false);
        }

        let Expr::Variable(name, span) = expr else {
            return Ok(false);
        };
        if is_request_superglobal_name(name) {
            return Err(self.unsupported(*span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION));
        }
        if self.symbol_table_variables.contains(name) {
            self.emit_native_symbol_table_value_stdout(name);
            return Ok(true);
        }
        if !self.variables.contains_key(name) {
            return Err(self.unsupported(*span, ASSEMBLY_VARIABLE_READ_REJECTION));
        }

        Ok(false)
    }

    fn ensure_native_symbol_table(&mut self) -> &'static str {
        self.uses_native_symbol_table_helpers = true;
        if !self.emitted_native_symbol_table {
            self.body.push(
                "phpc_NativeSymbolTableHandle phpc_symbols = phpc_native_symbol_table_new();"
                    .to_string(),
            );
            self.emitted_native_symbol_table = true;
        }
        "phpc_symbols"
    }

    fn emit_native_symbol_table_variable_copy(
        &mut self,
        target_name: &str,
        expr: &Expr,
    ) -> CompileResult<bool> {
        let Expr::Variable(source_name, span) = expr else {
            return Ok(false);
        };
        if is_request_superglobal_name(source_name) {
            return Err(self.unsupported(*span, ASSEMBLY_REQUEST_SUPERGLOBAL_REJECTION));
        }
        if !self.symbol_table_variables.contains(source_name) {
            return Ok(false);
        }

        let table = self.ensure_native_symbol_table();
        let index = self.next_native_temp;
        self.next_native_temp += 1;
        let source_data = self.emit_native_symbol_name_data(source_name);
        self.body.push(format!(
            "phpc_NativeLinkedValue value_{index} = phpc_native_symbol_table_read({table}, {source_data}, {});",
            source_name.len()
        ));

        let target_data = self.emit_native_symbol_name_data(target_name);
        self.body.push(format!(
            "phpc_native_symbol_table_write({table}, {target_data}, {}, value_{index});",
            target_name.len()
        ));

        Ok(true)
    }

    fn emit_native_symbol_table_write(
        &mut self,
        name: &str,
        value: &CValue,
    ) -> CompileResult<bool> {
        match value {
            CValue::Bool(value) => {
                self.emit_native_symbol_table_scalar_write(
                    name,
                    &format!(
                        "phpc_native_linked_value_from_bool({})",
                        if *value { 1 } else { 0 }
                    ),
                );
                return Ok(true);
            }
            CValue::BoolExpr(value) => {
                self.emit_native_symbol_table_scalar_write(
                    name,
                    &format!("phpc_native_linked_value_from_bool(({value}) ? 1 : 0)"),
                );
                return Ok(true);
            }
            CValue::Int(value) => {
                self.emit_native_symbol_table_scalar_write(
                    name,
                    &format!("phpc_native_linked_value_from_int((long long)({value}))"),
                );
                return Ok(true);
            }
            CValue::String(value) => self.emit_native_symbol_table_string_write(name, value),
            _ => Ok(false),
        }
    }

    fn emit_native_symbol_table_scalar_write(&mut self, name: &str, linked_value: &str) {
        let table = self.ensure_native_symbol_table();
        let name_data = self.emit_native_symbol_name_data(name);
        self.body.push(format!(
            "phpc_native_symbol_table_write({table}, {name_data}, {}, {linked_value});",
            name.len()
        ));
    }

    fn emit_native_symbol_table_string_write(
        &mut self,
        name: &str,
        value: &str,
    ) -> CompileResult<bool> {
        let table = self.ensure_native_symbol_table();
        let index = self.next_native_temp;
        self.next_native_temp += 1;
        let value_data = self.emit_static_bytes(value.as_bytes());
        let name_data = self.emit_native_symbol_name_data(name);

        self.body.push(format!(
            "phpc_NativeStringHandle string_{index} = phpc_native_string_from_bytes({value_data}, {});",
            value.len()
        ));
        self.body.push(format!(
            "phpc_NativeDiagnosticHandle diagnostic_{index} = {{0}};"
        ));
        self.body.push(format!(
            "phpc_NativeValueHandle value_{index} = phpc_native_value_from_string_with_diagnostic(string_{index}, &diagnostic_{index});"
        ));
        self.body.push(format!(
            "if (value_{index}.ptr == NULL) {{ phpc_native_diagnostic_message_stderr(diagnostic_{index}); phpc_native_diagnostic_free(diagnostic_{index}); }} else {{ phpc_native_symbol_table_write({table}, {name_data}, {}, phpc_native_linked_value_from_runtime_value(value_{index})); }}",
            name.len()
        ));
        self.body
            .push(format!("phpc_native_string_free(string_{index});"));

        Ok(true)
    }

    fn emit_native_symbol_table_value_stdout(&mut self, name: &str) {
        let table = self.ensure_native_symbol_table();
        let index = self.next_native_temp;
        self.next_native_temp += 1;
        let name_data = self.emit_native_symbol_name_data(name);

        self.body.push(format!(
            "phpc_NativeLinkedValue value_{index} = phpc_native_symbol_table_read({table}, {name_data}, {});",
            name.len()
        ));
        self.body.push(format!(
            "phpc_native_linked_value_echo_stdout(value_{index});"
        ));
    }

    fn emit_native_symbol_table_isset_expr(&mut self, name: &str) -> String {
        let table = self.ensure_native_symbol_table();
        let name_data = self.emit_native_symbol_name_data(name);
        format!(
            "phpc_native_symbol_table_isset({table}, {name_data}, {})",
            name.len()
        )
    }

    fn emit_native_symbol_table_unset(&mut self, name: &str) {
        let table = self.ensure_native_symbol_table();
        let name_data = self.emit_native_symbol_name_data(name);
        self.body.push(format!(
            "phpc_native_symbol_table_unset({table}, {name_data}, {});",
            name.len()
        ));
    }

    fn emit_native_symbol_name_data(&mut self, name: &str) -> String {
        let index = self.next_static_data;
        self.next_static_data += 1;
        let bytes = c_byte_array(name.as_bytes());
        let data = format!("phpc_native_symbol_name_{index}");
        self.static_data.push(format!(
            "static const uint8_t {data}[] = {{{bytes}}}; /* {} */",
            c_string(name)
        ));
        data
    }

    fn emit_static_bytes(&mut self, bytes: &[u8]) -> String {
        let index = self.next_static_data;
        self.next_static_data += 1;
        let data = format!("phpc_native_bytes_{index}");
        self.static_data.push(format!(
            "static const uint8_t {data}[] = {{{}}};",
            c_byte_array(bytes)
        ));
        data
    }

    fn emit_native_string_handle(&mut self, prefix: &str, value: &str) -> String {
        let handle = self.next_native_name(prefix);
        let data = self.emit_static_bytes(value.as_bytes());
        self.body.push(format!(
            "phpc_NativeStringHandle {handle} = phpc_native_string_from_bytes({data}, {});",
            value.len()
        ));
        handle
    }

    fn next_native_name(&mut self, prefix: &str) -> String {
        let index = self.next_native_temp;
        self.next_native_temp += 1;
        format!("{prefix}_{index}")
    }

    fn emit_native_string_helper_echo(&mut self, value: &str) {
        let index = self.next_native_temp;
        self.next_native_temp += 1;
        let data = self.emit_static_bytes(value.as_bytes());
        self.body.push(format!(
            "phpc_NativeStringHandle string_{index} = phpc_native_string_from_bytes({data}, {});",
            value.len()
        ));
        self.body.push(format!(
            "phpc_NativeDiagnosticHandle diagnostic_{index} = {{0}};"
        ));
        self.body.push(format!(
            "phpc_NativeValueHandle value_{index} = phpc_native_value_from_string_with_diagnostic(string_{index}, &diagnostic_{index});"
        ));
        self.body.push(format!(
            "if (value_{index}.ptr == NULL) {{ phpc_native_diagnostic_message_stderr(diagnostic_{index}); phpc_native_diagnostic_free(diagnostic_{index}); }} else {{ phpc_native_value_echo_stdout(value_{index}); }}"
        ));
        self.body
            .push(format!("phpc_native_value_free(value_{index});"));
        self.body
            .push(format!("phpc_native_string_free(string_{index});"));
    }

    fn native_error_exit(&self, local_cleanup: &str) -> String {
        format!("{local_cleanup}{}return 1;", self.native_program_cleanup())
    }

    fn native_program_cleanup(&self) -> String {
        let mut cleanup = String::new();
        for handle in self.array_cleanup_handles.iter().rev() {
            cleanup.push_str("phpc_native_array_free(");
            cleanup.push_str(handle);
            cleanup.push_str("); ");
        }
        if self.emitted_native_symbol_table {
            cleanup.push_str("phpc_native_symbol_table_free(phpc_symbols); ");
        }
        cleanup
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

fn known_strings_are_safe_for_native_comparison(values: &KnownString) -> bool {
    values.values().iter().all(|value| {
        value.bytes().all(|byte| byte.is_ascii() && byte != 0)
            && !string_looks_numeric_for_native_comparison(value)
    })
}

fn string_looks_numeric_for_native_comparison(value: &str) -> bool {
    let first = value.bytes().find(|byte| !byte.is_ascii_whitespace());
    matches!(first, Some(b'+' | b'-' | b'.' | b'0'..=b'9'))
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
    if !known_strings_are_safe_for_native_comparison(&left_values)
        || !known_strings_are_safe_for_native_comparison(&right_values)
    {
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
        _ => None,
    }
}

fn c_string_operand(value: CValue) -> String {
    match value {
        CValue::String(value) => format!("\"{}\"", c_string(&value)),
        CValue::StringExpr(value) => value,
        _ => unreachable!("string operands are prefiltered"),
    }
}

fn php_string_truthy(value: &str) -> bool {
    !value.is_empty() && value != "0"
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
    known_truthiness(values.values().iter().map(|value| php_string_truthy(value)))
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

fn is_value_debug_output_builtin(name: &str) -> bool {
    name.eq_ignore_ascii_case("var_dump") || name.eq_ignore_ascii_case("print_r")
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
            | "ksort"
            | "array_unshift"
            | "array_pop"
            | "next"
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

const NATIVE_KNOWN_FUNCTION_NAMES: &[&str] = &[
    "define",
    "strlen",
    "strtolower",
    "trim",
    "ltrim",
    "rtrim",
    "strcasecmp",
    "str_contains",
    "str_starts_with",
    "str_ends_with",
    "ctype_alnum",
    "ctype_alpha",
    "ctype_cntrl",
    "ctype_digit",
    "ctype_graph",
    "ctype_lower",
    "ctype_print",
    "ctype_punct",
    "ctype_space",
    "ctype_upper",
    "ctype_xdigit",
    "strpos",
    "substr",
    "substr_count",
    "str_replace",
    "preg_match",
    "preg_replace",
    "preg_split",
    "preg_replace_callback",
    "compact",
    "error_reporting",
    "ignore_user_abort",
    "php_sapi_name",
    "sprintf",
    "vsprintf",
    "call_user_func",
    "call_user_func_array",
    "implode",
    "basename",
    "dirname",
    "abs",
    "version_compare",
    "microtime",
    "date_default_timezone_set",
    "ini_get",
    "ini_set",
    "get_include_path",
    "set_include_path",
    "min",
    "rand",
    "uniqid",
    "hash_hmac",
    "count",
    "constant",
    "defined",
    "array_key_exists",
    "array_values",
    "array_key_first",
    "array_key_last",
    "current",
    "array_is_list",
    "array_keys",
    "array_reverse",
    "array_slice",
    "array_chunk",
    "array_pad",
    "array_merge",
    "array_replace",
    "array_flip",
    "array_change_key_case",
    "array_column",
    "array_fill_keys",
    "array_combine",
    "array_intersect_key",
    "array_diff_key",
    "array_diff",
    "array_intersect",
    "array_unique",
    "array_count_values",
    "array_sum",
    "array_product",
    "array_reduce",
    "array_filter",
    "array_map",
    "ksort",
    "array_unshift",
    "array_pop",
    "next",
    "in_array",
    "array_search",
    "gettype",
    "is_null",
    "is_bool",
    "is_int",
    "is_integer",
    "is_long",
    "is_float",
    "is_double",
    "is_string",
    "is_array",
    "is_scalar",
    "is_numeric",
    "is_countable",
    "is_iterable",
    "is_callable",
    "function_exists",
    "extension_loaded",
    "class_alias",
    "mysqli_connect",
    "mysqli_real_connect",
    "mysqli_get_server_info",
    "mysqli_get_server_version",
    "mysqli_get_host_info",
    "mysqli_get_client_info",
    "mysqli_get_client_version",
    "mysqli_get_proto_info",
    "mysqli_thread_id",
    "mysqli_kill",
    "mysqli_change_user",
    "mysqli_refresh",
    "mysqli_get_charset",
    "mysqli_character_set_name",
    "mysqli_field_count",
    "mysqli_close",
    "mysqli_options",
    "mysqli_set_opt",
    "mysqli_ssl_set",
    "mysqli_connect_errno",
    "mysqli_connect_error",
    "mysqli_error_list",
    "mysqli_get_connection_stats",
    "mysqli_get_links_stats",
    "mysqli_get_client_stats",
    "mysqli_thread_safe",
    "mysqli_stmt_init",
    "mysqli_prepare",
    "mysqli_stmt_prepare",
    "mysqli_stmt_param_count",
    "mysqli_stmt_get_warnings",
    "mysqli_stmt_error_list",
    "mysqli_stmt_bind_param",
    "mysqli_stmt_bind_result",
    "mysqli_stmt_execute",
    "mysqli_execute",
    "mysqli_stmt_get_result",
    "mysqli_stmt_close",
    "mysqli_stmt_errno",
    "mysqli_stmt_error",
    "mysqli_stmt_affected_rows",
    "mysqli_stmt_store_result",
    "mysqli_stmt_num_rows",
    "mysqli_stmt_fetch",
    "mysqli_stmt_result_metadata",
    "mysqli_stmt_field_count",
    "mysqli_stmt_free_result",
    "mysqli_stmt_data_seek",
    "mysqli_stmt_attr_get",
    "mysqli_stmt_attr_set",
    "mysqli_stmt_send_long_data",
    "mysqli_stmt_reset",
    "mysqli_stmt_more_results",
    "mysqli_stmt_next_result",
    "mysqli_stmt_sqlstate",
    "mysqli_stmt_warning_count",
    "mysqli_stmt_insert_id",
    "mysqli_execute_query",
    "mysqli_dump_debug_info",
    "mysqli_debug",
    "mysqli_stat",
    "mysqli_autocommit",
    "mysqli_begin_transaction",
    "mysqli_commit",
    "mysqli_rollback",
    "mysqli_savepoint",
    "mysqli_release_savepoint",
    "mysqli_set_charset",
    "mysqli_query",
    "mysqli_real_query",
    "mysqli_multi_query",
    "mysqli_errno",
    "mysqli_error",
    "mysqli_sqlstate",
    "mysqli_warning_count",
    "mysqli_info",
    "mysqli_get_warnings",
    "mysqli_affected_rows",
    "mysqli_insert_id",
    "mysqli_ping",
    "mysqli_select_db",
    "mysqli_real_escape_string",
    "mysqli_escape_string",
    "mysqli_fetch_object",
    "mysqli_fetch_assoc",
    "mysqli_fetch_row",
    "mysqli_fetch_array",
    "mysqli_fetch_all",
    "mysqli_fetch_column",
    "mysqli_fetch_field",
    "mysqli_fetch_fields",
    "mysqli_fetch_field_direct",
    "mysqli_num_fields",
    "mysqli_num_rows",
    "mysqli_fetch_lengths",
    "mysqli_data_seek",
    "mysqli_field_seek",
    "mysqli_field_tell",
    "mysqli_free_result",
    "mysqli_more_results",
    "mysqli_next_result",
    "mysqli_store_result",
    "mysqli_use_result",
    "mysqli_reap_async_query",
    "mysqli_poll",
    "mysqli_report",
    "mysqli_init",
    "is_uploaded_file",
    "move_uploaded_file",
    "file_exists",
    "file_get_contents",
    "fopen",
    "stream_context_create",
    "stream_context_get_options",
    "stream_context_get_params",
    "stream_context_get_default",
    "stream_context_set_default",
    "stream_context_set_option",
    "stream_context_set_params",
    "fwrite",
    "fread",
    "rewind",
    "stream_get_contents",
    "feof",
    "ftell",
    "fseek",
    "fstat",
    "stream_get_meta_data",
    "fclose",
    "opendir",
    "readdir",
    "rewinddir",
    "closedir",
    "filesize",
    "filemtime",
    "realpath",
    "realpath_cache_get",
    "realpath_cache_size",
    "getcwd",
    "is_dir",
    "is_file",
    "is_readable",
    "is_writable",
    "is_link",
    "clearstatcache",
    "register_shutdown_function",
    "set_error_handler",
    "restore_error_handler",
    "ob_start",
    "ob_get_level",
    "ob_get_contents",
    "ob_get_length",
    "ob_list_handlers",
    "ob_get_status",
    "ob_get_clean",
    "ob_get_flush",
    "ob_clean",
    "ob_flush",
    "ob_end_clean",
    "ob_end_flush",
    "header",
    "header_remove",
    "headers_list",
    "headers_sent",
    "http_response_code",
    "setcookie",
    "setrawcookie",
    "session_start",
    "session_status",
    "session_cache_limiter",
    "session_cache_expire",
    "session_id",
    "session_write_close",
    "assert",
    "get_class",
    "is_object",
    "get_debug_type",
    "class_exists",
    "interface_exists",
    "trait_exists",
    "enum_exists",
    "get_declared_classes",
    "get_declared_interfaces",
    "get_declared_traits",
    "class_implements",
    "class_uses",
    "class_parents",
    "get_called_class",
    "spl_object_id",
    "spl_object_hash",
    "spl_autoload",
    "spl_autoload_register",
    "spl_autoload_functions",
    "spl_autoload_extensions",
    "spl_autoload_unregister",
    "spl_autoload_call",
    "property_exists",
    "method_exists",
    "get_class_methods",
    "get_class_vars",
    "get_object_vars",
    "get_mangled_object_vars",
    "is_a",
    "is_subclass_of",
    "get_parent_class",
    "var_dump",
    "print_r",
];

fn is_native_known_function_name(name: &str) -> bool {
    NATIVE_KNOWN_FUNCTION_NAMES
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(name))
}

const COMPAT_LOADED_EXTENSION_NAMES: &[&str] = &["json", "hash", "pdo", "pdo_mysql"];

fn is_compat_loaded_extension_name(name: &str) -> bool {
    COMPAT_LOADED_EXTENSION_NAMES
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(name))
}

fn known_strings_have_uniform_numeric_result(values: &KnownString) -> Option<bool> {
    let mut result = None;
    for value in values.values() {
        let current = is_php_numeric_string_literal(value);
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

fn is_php_numeric_string_literal(value: &str) -> bool {
    let trimmed = value.trim_matches(|ch: char| ch.is_ascii_whitespace());
    !trimmed.is_empty() && is_well_formed_php_numeric_string(trimmed)
}

fn is_well_formed_php_numeric_string(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0;

    if matches!(bytes.get(index), Some(b'+' | b'-')) {
        index += 1;
    }

    let digits_before_decimal = consume_ascii_digits(bytes, &mut index);
    if matches!(bytes.get(index), Some(b'.')) {
        index += 1;
        let digits_after_decimal = consume_ascii_digits(bytes, &mut index);
        if digits_before_decimal == 0 && digits_after_decimal == 0 {
            return false;
        }
    } else if digits_before_decimal == 0 {
        return false;
    }

    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        if consume_ascii_digits(bytes, &mut index) == 0 {
            return false;
        }
    }

    index == bytes.len()
}

fn consume_ascii_digits(bytes: &[u8], index: &mut usize) -> usize {
    let start = *index;
    while matches!(bytes.get(*index), Some(b'0'..=b'9')) {
        *index += 1;
    }
    *index - start
}

fn llvm_gettype_name(value: &IrValue) -> &'static str {
    match value {
        IrValue::Null => "NULL",
        IrValue::Bool(_) | IrValue::BoolExpr(_) => "boolean",
        IrValue::Int(_) => "integer",
        IrValue::Float(_) => "double",
        IrValue::String(_) | IrValue::StringPtr(_) => "string",
        IrValue::NativeExpression { fallback, .. } => llvm_gettype_name(fallback),
    }
}

fn llvm_debug_type_name(value: &IrValue) -> &'static str {
    match value {
        IrValue::Null => "null",
        IrValue::Bool(_) | IrValue::BoolExpr(_) => "bool",
        IrValue::Int(_) => "int",
        IrValue::Float(_) => "float",
        IrValue::String(_) | IrValue::StringPtr(_) => "string",
        IrValue::NativeExpression { fallback, .. } => llvm_debug_type_name(fallback),
    }
}

fn c_gettype_name(value: &CValue) -> &'static str {
    match value {
        CValue::Null => "NULL",
        CValue::Bool(_) | CValue::BoolExpr(_) => "boolean",
        CValue::Int(_) => "integer",
        CValue::Float(_) => "double",
        CValue::String(_) | CValue::StringExpr(_) => "string",
    }
}

fn c_debug_type_name(value: &CValue) -> &'static str {
    match value {
        CValue::Null => "null",
        CValue::Bool(_) | CValue::BoolExpr(_) => "bool",
        CValue::Int(_) => "int",
        CValue::Float(_) => "float",
        CValue::String(_) | CValue::StringExpr(_) => "string",
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

    fn local_state(values: &[(&str, i32)]) -> HashMap<String, i32> {
        values
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect()
    }

    fn live_handles(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    fn span(line: usize, column: usize) -> Span {
        Span::new(line, column)
    }

    fn merged_branch_result(outcome: NativeBranchMergeOutcome) -> NativeBranchMergeResult {
        let NativeBranchMergeOutcome::Merged(result) = outcome else {
            panic!("expected successful branch merge");
        };
        result
    }

    fn empty_branch_local_cleanup_plan(
        control_join: NativeBranchEffectJoin,
    ) -> NativeBranchLocalCleanupPlan {
        NativeBranchLocalCleanupPlan {
            control_join,
            entry_live_locals: Vec::new(),
            then_live_locals: Vec::new(),
            else_live_locals: Vec::new(),
            stable_live_locals: Vec::new(),
            divergent_live_locals: Vec::new(),
            then_only_locals: Vec::new(),
            else_only_locals: Vec::new(),
        }
    }

    fn native_value_effect_with_cleanup_tail(
        handle: &str,
        tail_actions: Vec<NativeTerminationCleanupAction<String>>,
    ) -> NativeTerminationEffect<String> {
        let mut actions = vec![NativeTerminationCleanupAction::LiveNativeValueHandle(
            handle.to_string(),
        )];
        actions.extend(tail_actions);
        NativeTerminationEffect {
            status: NativeTerminationStatus::NativeValueHandle(handle.to_string()),
            cleanup_stack: NativeTerminationCleanupStack::from_actions(actions),
        }
    }

    #[test]
    fn native_branch_effect_join_classifies_all_flow_pairs() {
        assert_eq!(
            NativeBranchEffectJoin::from_flows(
                NativeControlFlowEffect::Continues,
                NativeControlFlowEffect::Continues,
            ),
            NativeBranchEffectJoin::BothContinue
        );
        assert_eq!(
            NativeBranchEffectJoin::from_flows(
                NativeControlFlowEffect::Terminates,
                NativeControlFlowEffect::Terminates,
            ),
            NativeBranchEffectJoin::BothTerminate
        );
        assert_eq!(
            NativeBranchEffectJoin::from_flows(
                NativeControlFlowEffect::Continues,
                NativeControlFlowEffect::Terminates,
            ),
            NativeBranchEffectJoin::ThenContinues
        );
        assert_eq!(
            NativeBranchEffectJoin::from_flows(
                NativeControlFlowEffect::Terminates,
                NativeControlFlowEffect::Continues,
            ),
            NativeBranchEffectJoin::ElseContinues
        );
    }

    #[test]
    fn native_branch_local_cleanup_plan_classifies_branch_state_changes() {
        let entry = local_state(&[("entry", 1), ("stable", 2), ("changed", 3)]);
        let then_state = local_state(&[
            ("stable", 2),
            ("changed", 4),
            ("then_only", 5),
            ("entry", 1),
        ]);
        let else_state = local_state(&[
            ("stable", 2),
            ("changed", 6),
            ("else_only", 7),
            ("entry", 1),
        ]);

        let plan = NativeBranchLocalCleanupPlan::from_states(
            NativeBranchEffectJoin::BothContinue,
            &entry,
            &then_state,
            &else_state,
        );

        assert_eq!(plan.control_join(), NativeBranchEffectJoin::BothContinue);
        assert_eq!(plan.entry_live_locals, ["changed", "entry", "stable"]);
        assert_eq!(
            plan.then_live_locals,
            ["changed", "entry", "stable", "then_only"]
        );
        assert_eq!(
            plan.else_live_locals,
            ["changed", "else_only", "entry", "stable"]
        );
        assert_eq!(plan.stable_live_locals, ["entry", "stable"]);
        assert_eq!(
            plan.divergent_live_locals,
            ["changed", "else_only", "then_only"]
        );
        assert_eq!(plan.then_only_locals, ["then_only"]);
        assert_eq!(plan.else_only_locals, ["else_only"]);
        assert!(!plan.has_stable_local_merge());
    }

    #[test]
    fn native_branch_local_cleanup_plan_accepts_stable_generic_state() {
        let entry = local_state(&[("left", 1), ("right", 2)]);
        let then_state = local_state(&[("left", 1), ("right", 2)]);
        let else_state = local_state(&[("right", 2), ("left", 1)]);

        let plan = NativeBranchLocalCleanupPlan::from_states(
            NativeBranchEffectJoin::BothContinue,
            &entry,
            &then_state,
            &else_state,
        );

        assert_eq!(plan.stable_live_locals, ["left", "right"]);
        assert!(plan.divergent_live_locals.is_empty());
        assert!(plan.then_only_locals.is_empty());
        assert!(plan.else_only_locals.is_empty());
        assert!(plan.has_stable_local_merge());
    }

    #[test]
    fn native_branch_value_fact_ownership_is_join_and_local_phi_driven() {
        let entry = local_state(&[("code", 1), ("stable", 10)]);
        let then_state = local_state(&[("code", 2), ("stable", 10)]);
        let else_state = local_state(&[("code", 3), ("stable", 10)]);
        let local_phi_plan = NativeBranchLocalCleanupPlan::from_states(
            NativeBranchEffectJoin::BothContinue,
            &entry,
            &then_state,
            &else_state,
        );
        assert_eq!(local_phi_plan.locals_requiring_phi(), &["code".to_string()]);
        assert!(local_phi_plan.has_local_phi_merge_ownership());
        assert!(!local_phi_plan.has_stable_local_merge());

        let stable_facts = HashMap::from([("stable:int".to_string(), "[10]".to_string())]);
        let stable_fact_plan = NativeBranchValueFactCleanupPlan::from_states(
            NativeBranchEffectJoin::BothContinue,
            &stable_facts,
            &stable_facts,
            &stable_facts,
        );
        assert_eq!(
            stable_fact_plan.merge_ownership(&local_phi_plan),
            NativeBranchValueFactOwnership::Stable
        );
        assert_eq!(
            branch_value_fact_ownership(None, &local_phi_plan),
            NativeBranchValueFactOwnership::NoLiveFacts
        );

        let entry_facts = HashMap::new();
        let then_code_fact = HashMap::from([("code:int".to_string(), "[2]".to_string())]);
        let else_code_fact = HashMap::from([("code:int".to_string(), "[3]".to_string())]);
        let local_phi_fact_plan = NativeBranchValueFactCleanupPlan::from_states(
            NativeBranchEffectJoin::BothContinue,
            &entry_facts,
            &then_code_fact,
            &else_code_fact,
        );
        assert_eq!(
            local_phi_fact_plan.merge_ownership(&local_phi_plan),
            NativeBranchValueFactOwnership::LocalPhi
        );
        assert!(!local_phi_fact_plan
            .merge_ownership(&local_phi_plan)
            .can_merge_without_phi());
        assert!(local_phi_fact_plan
            .merge_ownership(&local_phi_plan)
            .can_merge_with_local_phi());

        let then_other_fact = HashMap::from([("other:int".to_string(), "[2]".to_string())]);
        let else_other_fact = HashMap::from([("other:int".to_string(), "[3]".to_string())]);
        let unrelated_fact_plan = NativeBranchValueFactCleanupPlan::from_states(
            NativeBranchEffectJoin::BothContinue,
            &entry_facts,
            &then_other_fact,
            &else_other_fact,
        );
        assert_eq!(
            unrelated_fact_plan.merge_ownership(&local_phi_plan),
            NativeBranchValueFactOwnership::Blocked
        );

        let partial_plan = NativeBranchLocalCleanupPlan::from_states(
            NativeBranchEffectJoin::ThenContinues,
            &entry,
            &entry,
            &then_state,
        );
        assert_eq!(
            partial_plan.continuing_arm(),
            Some(NativeContinuingBranchArm::Then)
        );
        assert!(!partial_plan.has_stable_local_merge());
        assert!(!partial_plan.has_local_phi_merge_ownership());
        let partial_fact_plan = NativeBranchValueFactCleanupPlan::from_states(
            NativeBranchEffectJoin::ThenContinues,
            &stable_facts,
            &stable_facts,
            &stable_facts,
        );
        assert_eq!(
            partial_fact_plan.control_join(),
            NativeBranchEffectJoin::ThenContinues
        );
        assert_eq!(
            partial_fact_plan.merge_ownership(&partial_plan),
            NativeBranchValueFactOwnership::Blocked
        );
    }

    #[test]
    fn native_branch_live_native_value_cleanup_plan_tracks_branch_owned_handles() {
        let entry_handles = live_handles(&["%entry", "%shared", "%shared"]);
        let then_handles = live_handles(&["%branch_then", "%shared"]);
        let else_handles = live_handles(&["%branch_else", "%shared"]);

        let plan = NativeBranchLiveNativeValueCleanupPlan::from_handles(
            NativeBranchEffectJoin::BothContinue,
            &entry_handles,
            &then_handles,
            &else_handles,
        );

        assert_eq!(plan.control_join(), NativeBranchEffectJoin::BothContinue);
        assert_eq!(plan.entry_live_handles, ["%entry", "%shared"]);
        assert_eq!(plan.then_live_handles, ["%branch_then", "%shared"]);
        assert_eq!(plan.else_live_handles, ["%branch_else", "%shared"]);
        assert_eq!(plan.stable_live_handles, ["%shared"]);
        assert_eq!(
            plan.divergent_live_handles,
            ["%branch_else", "%branch_then"]
        );
        assert_eq!(plan.then_only_handles, ["%branch_then"]);
        assert_eq!(plan.else_only_handles, ["%branch_else"]);
        assert_eq!(
            plan.merge_ownership(),
            NativeBranchLiveNativeValueOwnership::Blocked
        );

        let stable_handles = live_handles(&["%stable_a", "%stable_b"]);
        let stable_plan = NativeBranchLiveNativeValueCleanupPlan::from_handles(
            NativeBranchEffectJoin::BothContinue,
            &stable_handles,
            &stable_handles,
            &stable_handles,
        );
        assert_eq!(
            stable_plan.merge_ownership(),
            NativeBranchLiveNativeValueOwnership::Stable
        );
        assert!(stable_plan.merge_ownership().can_merge_without_phi());

        let empty_plan = NativeBranchLiveNativeValueCleanupPlan::from_handles(
            NativeBranchEffectJoin::ThenContinues,
            &[],
            &[],
            &[],
        );
        assert_eq!(
            empty_plan.merge_ownership(),
            NativeBranchLiveNativeValueOwnership::NoLiveHandles
        );
        assert!(empty_plan
            .merge_ownership()
            .allows_non_joining_control_flow());
    }

    #[test]
    fn native_termination_cleanup_stack_composes_ordered_cleanup_actions() {
        let mut cleanup_stack: NativeTerminationCleanupStack<()> =
            NativeTerminationCleanupStack::from_action(
                NativeTerminationCleanupAction::OutputBufferStack,
            );

        cleanup_stack.push_action(NativeTerminationCleanupAction::ShutdownQueue);
        cleanup_stack.prepend_stack(NativeTerminationCleanupStack::from_actions(vec![
            NativeTerminationCleanupAction::FunctionFrame,
            NativeTerminationCleanupAction::GotoScope,
        ]));
        cleanup_stack.append_stack(NativeTerminationCleanupStack::from_action(
            NativeTerminationCleanupAction::DestructorQueue,
        ));

        assert_eq!(
            cleanup_stack.actions(),
            &[
                NativeTerminationCleanupAction::FunctionFrame,
                NativeTerminationCleanupAction::GotoScope,
                NativeTerminationCleanupAction::OutputBufferStack,
                NativeTerminationCleanupAction::ShutdownQueue,
                NativeTerminationCleanupAction::DestructorQueue,
            ]
        );
    }

    #[test]
    fn native_termination_cleanup_boundary_carries_outer_diagnostic_and_cleanup_stack() {
        let mut boundary = NativeTerminationCleanupBoundary::from_hook_boundary(
            span(4, 2),
            NativeTerminationHookBoundary::OutputBuffer,
        );
        boundary.append_cleanup_boundary(NativeTerminationCleanupBoundary::from_hook_boundary(
            span(5, 7),
            NativeTerminationHookBoundary::Shutdown,
        ));

        let boundary = boundary
            .with_outer_hook_boundary(span(2, 1), NativeTerminationHookBoundary::TryFinally);

        assert_eq!(boundary.span(), span(2, 1));
        assert!(boundary.llvm_message().starts_with(
            "LLVM termination control-flow lowering rejects exit()/die() in try/catch/finally contexts"
        ));
        assert!(boundary.assembly_message().starts_with(
            "assembly termination control-flow lowering rejects exit()/die() in try/catch/finally contexts"
        ));
        assert_eq!(
            boundary.cleanup_stack.actions(),
            &[
                NativeTerminationCleanupAction::FinallyDispatch,
                NativeTerminationCleanupAction::OutputBufferStack,
                NativeTerminationCleanupAction::ShutdownQueue,
            ]
        );
    }

    #[test]
    fn native_termination_cleanup_blocker_accepts_composed_boundaries() {
        let mut boundary = NativeTerminationCleanupBoundary::from_hook_boundary(
            span(10, 4),
            NativeTerminationHookBoundary::OutputBuffer,
        );
        boundary.prepend_cleanup_boundary(NativeTerminationCleanupBoundary::from_hook_boundary(
            span(9, 1),
            NativeTerminationHookBoundary::FunctionFrame,
        ));

        let blocker = NativeTerminationCleanupBlocker::from_cleanup_boundary(boundary);

        assert_eq!(blocker.span(), span(10, 4));
        assert!(blocker.llvm_message().starts_with(
            "LLVM termination hook lowering rejects exit()/die() with active or queried output buffers"
        ));
        assert!(blocker.assembly_message().starts_with(
            "assembly termination hook lowering rejects exit()/die() with active or queried output buffers"
        ));
        assert_eq!(blocker.termination_effect().status_value(), None);
        assert_eq!(
            blocker.cleanup_stack().actions(),
            &[
                NativeTerminationCleanupAction::FunctionFrame,
                NativeTerminationCleanupAction::OutputBufferStack,
            ]
        );
    }

    #[test]
    fn native_c_error_exit_centralizes_program_cleanup() {
        let mut generator = CGenerator::default();
        generator.array_cleanup_handles.push("array_0".to_string());
        generator.array_cleanup_handles.push("array_1".to_string());
        generator.emitted_native_symbol_table = true;

        assert_eq!(
            generator.native_error_exit("local_cleanup(); "),
            "local_cleanup(); phpc_native_array_free(array_1); phpc_native_array_free(array_0); phpc_native_symbol_table_free(phpc_symbols); return 1;"
        );
    }

    #[test]
    fn native_branch_merge_outcome_centralizes_cleanup_blockers() {
        let entry = local_state(&[("code", 1)]);
        let mut divergent_then = entry.clone();
        divergent_then.insert("message".to_string(), 2);
        let mut divergent_else = entry.clone();
        divergent_else.insert("message".to_string(), 3);

        let local_outcome = NativeBranchTerminationEffect::from_states(
            &entry,
            &divergent_then,
            &divergent_else,
            NativeControlFlowEffect::Continues,
            NativeControlFlowEffect::Continues,
        )
        .into_merge_outcome(span(20, 1), true);
        let NativeBranchMergeOutcome::Blocked(local_blocker) = local_outcome else {
            panic!("expected local cleanup blocker");
        };
        assert_eq!(local_blocker.span(), span(20, 1));
        assert!(local_blocker.llvm_message().starts_with(
            "LLVM termination control-flow lowering rejects exit()/die() in branches that may continue"
        ));
        assert_eq!(local_blocker.termination_effect().status_value(), None);
        assert!(matches!(
            local_blocker.cleanup_stack().actions().get(1),
            Some(NativeTerminationCleanupAction::BranchBlockLocals(cleanup_plan))
                if cleanup_plan.divergent_live_locals == ["message"]
        ));

        let empty_facts: HashMap<String, String> = HashMap::new();
        let mut then_facts = HashMap::new();
        then_facts.insert("code:int".to_string(), "[2]".to_string());
        let mut else_facts = HashMap::new();
        else_facts.insert("code:int".to_string(), "[3]".to_string());

        let value_outcome = NativeBranchTerminationEffect::from_merge_inputs(
            &entry,
            &entry,
            &entry,
            &empty_facts,
            &then_facts,
            &else_facts,
            NativeControlFlowEffect::Continues,
            NativeControlFlowEffect::Continues,
        )
        .into_merge_outcome(span(21, 1), true);
        let NativeBranchMergeOutcome::Blocked(value_blocker) = value_outcome else {
            panic!("expected value-fact cleanup blocker");
        };
        assert!(matches!(
            value_blocker.cleanup_stack().actions().get(2),
            Some(NativeTerminationCleanupAction::BranchValueFacts(cleanup_plan))
                if cleanup_plan.divergent_live_facts == ["code:int"]
        ));

        let backend_outcome = NativeBranchTerminationEffect::from_states(
            &entry,
            &entry,
            &entry,
            NativeControlFlowEffect::Continues,
            NativeControlFlowEffect::Continues,
        )
        .into_merge_outcome(span(22, 1), false);
        assert!(matches!(
            backend_outcome,
            NativeBranchMergeOutcome::Blocked(_)
        ));

        let stable_outcome = NativeBranchTerminationEffect::from_states(
            &entry,
            &entry,
            &entry,
            NativeControlFlowEffect::Continues,
            NativeControlFlowEffect::Continues,
        )
        .into_merge_outcome(span(23, 1), true);
        let stable_result = merged_branch_result(stable_outcome);
        assert_eq!(
            stable_result.kind(),
            NativeBranchMergeKind::BothContinueStable
        );
        assert_eq!(stable_result.termination_effect().status_value(), None);
        assert!(matches!(
            stable_result.cleanup_stack().actions().get(1),
            Some(NativeTerminationCleanupAction::BranchBlockLocals(cleanup_plan))
                if cleanup_plan.has_stable_local_merge()
        ));

        let partial_outcome = NativeBranchTerminationEffect::from_merge_inputs(
            &entry,
            &divergent_then,
            &divergent_else,
            &empty_facts,
            &then_facts,
            &else_facts,
            NativeControlFlowEffect::Terminates,
            NativeControlFlowEffect::Continues,
        )
        .into_merge_outcome(span(24, 1), false);
        let partial_result = merged_branch_result(partial_outcome);
        assert_eq!(
            partial_result.kind(),
            NativeBranchMergeKind::ContinueWith(NativeContinuingBranchArm::Else)
        );
        assert_eq!(partial_result.termination_effect().status_value(), None);
        assert!(matches!(
            partial_result.cleanup_stack().actions().get(1),
            Some(NativeTerminationCleanupAction::BranchBlockLocals(cleanup_plan))
                if cleanup_plan.divergent_live_locals == ["message"]
        ));
        assert!(matches!(
            partial_result.cleanup_stack().actions().get(2),
            Some(NativeTerminationCleanupAction::BranchValueFacts(cleanup_plan))
                if cleanup_plan.divergent_live_facts == ["code:int"]
        ));

        let terminated_outcome = NativeBranchTerminationEffect::from_states(
            &entry,
            &divergent_then,
            &divergent_else,
            NativeControlFlowEffect::Terminates,
            NativeControlFlowEffect::Terminates,
        )
        .into_merge_outcome(span(25, 1), false);
        let terminated_result = merged_branch_result(terminated_outcome);
        assert_eq!(
            terminated_result.kind(),
            NativeBranchMergeKind::BothTerminate
        );
        assert_eq!(terminated_result.termination_effect().status_value(), None);
        assert!(matches!(
            terminated_result.cleanup_stack().actions().get(1),
            Some(NativeTerminationCleanupAction::BranchBlockLocals(cleanup_plan))
                if cleanup_plan.divergent_live_locals == ["message"]
        ));
    }

    #[test]
    fn branch_live_native_values_block_unowned_branch_merges_and_transfers() {
        let entry = local_state(&[("code", 1)]);
        let empty_facts: HashMap<String, String> = HashMap::new();
        let stable_handles = live_handles(&["%stable"]);

        let stable_outcome =
            NativeBranchTerminationEffect::from_merge_inputs_with_live_native_values(
                &entry,
                &entry,
                &entry,
                &empty_facts,
                &empty_facts,
                &empty_facts,
                &stable_handles,
                &stable_handles,
                &stable_handles,
                NativeControlFlowEffect::Continues,
                NativeControlFlowEffect::Continues,
            )
            .into_merge_outcome(span(26, 1), true);
        let stable_result = merged_branch_result(stable_outcome);
        assert_eq!(
            stable_result.kind(),
            NativeBranchMergeKind::BothContinueStable
        );
        assert!(matches!(
            stable_result.cleanup_stack().actions().get(2),
            Some(NativeTerminationCleanupAction::BranchLiveNativeValues(cleanup_plan))
                if cleanup_plan.merge_ownership() == NativeBranchLiveNativeValueOwnership::Stable
        ));

        let then_handles = live_handles(&["%then_only"]);
        let divergent_outcome =
            NativeBranchTerminationEffect::from_merge_inputs_with_live_native_values(
                &entry,
                &entry,
                &entry,
                &empty_facts,
                &empty_facts,
                &empty_facts,
                &[],
                &then_handles,
                &[],
                NativeControlFlowEffect::Continues,
                NativeControlFlowEffect::Continues,
            )
            .into_merge_outcome(span(27, 1), true);
        let NativeBranchMergeOutcome::Blocked(divergent_blocker) = divergent_outcome else {
            panic!("expected branch-owned native value cleanup blocker");
        };
        assert!(matches!(
            divergent_blocker.cleanup_stack().actions().get(2),
            Some(NativeTerminationCleanupAction::BranchLiveNativeValues(cleanup_plan))
                if cleanup_plan.then_only_handles == ["%then_only"]
        ));

        let partial_outcome =
            NativeBranchTerminationEffect::from_merge_inputs_with_live_native_values(
                &entry,
                &entry,
                &entry,
                &empty_facts,
                &empty_facts,
                &empty_facts,
                &[],
                &then_handles,
                &[],
                NativeControlFlowEffect::Terminates,
                NativeControlFlowEffect::Continues,
            )
            .into_merge_outcome(span(28, 1), false);
        assert!(matches!(
            partial_outcome,
            NativeBranchMergeOutcome::Blocked(_)
        ));
    }

    #[test]
    fn native_termination_effect_stack_owns_live_native_value_handle() {
        let effect = NativeTerminationEffect::from_native_value_handle("%value".to_string());

        assert_eq!(effect.status_value().map(String::as_str), Some("%value"));
        assert_eq!(
            effect.cleanup_stack().actions(),
            &[NativeTerminationCleanupAction::LiveNativeValueHandle(
                "%value".to_string()
            )]
        );

        let mut cleanup_effect: NativeTerminationEffect<()> =
            NativeTerminationEffect::from_cleanup_stack(
                NativeTerminationCleanupStack::from_action(
                    NativeTerminationCleanupAction::LoopScope,
                ),
            );
        cleanup_effect.prepend_cleanup_stack(NativeTerminationCleanupStack::from_action(
            NativeTerminationCleanupAction::FunctionFrame,
        ));
        cleanup_effect.append_cleanup_stack(NativeTerminationCleanupStack::from_action(
            NativeTerminationCleanupAction::ShutdownQueue,
        ));

        assert_eq!(cleanup_effect.status_value(), None);
        let cleanup_stack = cleanup_effect.into_cleanup_stack();
        assert_eq!(
            cleanup_stack.actions(),
            &[
                NativeTerminationCleanupAction::FunctionFrame,
                NativeTerminationCleanupAction::LoopScope,
                NativeTerminationCleanupAction::ShutdownQueue,
            ]
        );
    }

    #[test]
    fn cleanup_stack_execution_plan_centralizes_live_handles_and_blockers() {
        let live_stack = NativeTerminationCleanupStack::from_actions(vec![
            NativeTerminationCleanupAction::LiveNativeValueHandle("first".to_string()),
            NativeTerminationCleanupAction::LiveNativeValueHandle("second".to_string()),
        ]);
        let NativeTerminationCleanupStackPlan::Executable(plan) =
            live_stack.runtime_execution_plan()
        else {
            panic!("live native handles should be executable cleanup");
        };
        let handles = plan
            .live_native_value_handles()
            .iter()
            .map(|handle| handle.as_str())
            .collect::<Vec<_>>();
        assert_eq!(handles, ["first", "second"]);
        assert_eq!(live_stack.unlowered_runtime_cleanup_boundary_kind(), None);

        let branch_stack: NativeTerminationCleanupStack<()> =
            NativeTerminationCleanupStack::from_branch_cleanup_plans(
                empty_branch_local_cleanup_plan(NativeBranchEffectJoin::BothContinue),
                NativeBranchValueFactCleanupPlan::from_states(
                    NativeBranchEffectJoin::BothContinue,
                    &HashMap::new(),
                    &HashMap::new(),
                    &HashMap::new(),
                ),
            );
        assert_eq!(
            branch_stack.unlowered_runtime_cleanup_boundary_kind(),
            Some(NativeTerminationCleanupBoundaryKind::BranchMerge)
        );

        let live_branch_stack: NativeTerminationCleanupStack<()> =
            NativeTerminationCleanupStack::from_branch_cleanup_plans_with_live_native_values(
                empty_branch_local_cleanup_plan(NativeBranchEffectJoin::BothContinue),
                NativeBranchLiveNativeValueCleanupPlan::from_handles(
                    NativeBranchEffectJoin::BothContinue,
                    &live_handles(&["%entry"]),
                    &live_handles(&["%entry", "%then"]),
                    &live_handles(&["%entry"]),
                ),
                NativeBranchValueFactCleanupPlan::from_states(
                    NativeBranchEffectJoin::BothContinue,
                    &HashMap::new(),
                    &HashMap::new(),
                    &HashMap::new(),
                ),
            );
        assert_eq!(
            live_branch_stack
                .branch_live_native_value_cleanup_plan()
                .map(NativeBranchLiveNativeValueCleanupPlan::merge_ownership),
            Some(NativeBranchLiveNativeValueOwnership::Blocked)
        );
        assert_eq!(
            live_branch_stack.unlowered_runtime_cleanup_boundary_kind(),
            Some(NativeTerminationCleanupBoundaryKind::BranchMerge)
        );

        let expression_branch_stack: NativeTerminationCleanupStack<()> =
            NativeTerminationCleanupStack::from_actions(vec![
                NativeTerminationCleanupAction::DiscardedNativeTemporaries,
                NativeTerminationCleanupAction::BranchBlockLocals(empty_branch_local_cleanup_plan(
                    NativeBranchEffectJoin::BothContinue,
                )),
            ]);
        assert_eq!(
            expression_branch_stack.unlowered_runtime_cleanup_boundary_kind(),
            Some(NativeTerminationCleanupBoundaryKind::BranchMerge)
        );

        let hook_cases: Vec<(
            NativeTerminationCleanupAction<()>,
            NativeTerminationCleanupBoundaryKind,
        )> = vec![
            (
                NativeTerminationCleanupAction::LoopScope,
                NativeTerminationCleanupBoundaryKind::Hook(
                    NativeTerminationHookBoundary::LoopScope,
                ),
            ),
            (
                NativeTerminationCleanupAction::SwitchScope,
                NativeTerminationCleanupBoundaryKind::Hook(
                    NativeTerminationHookBoundary::SwitchScope,
                ),
            ),
            (
                NativeTerminationCleanupAction::GotoScope,
                NativeTerminationCleanupBoundaryKind::Hook(
                    NativeTerminationHookBoundary::GotoScope,
                ),
            ),
            (
                NativeTerminationCleanupAction::FunctionFrame,
                NativeTerminationCleanupBoundaryKind::Hook(
                    NativeTerminationHookBoundary::FunctionFrame,
                ),
            ),
            (
                NativeTerminationCleanupAction::ReturnContext,
                NativeTerminationCleanupBoundaryKind::Hook(
                    NativeTerminationHookBoundary::ReturnContext,
                ),
            ),
            (
                NativeTerminationCleanupAction::FinallyDispatch,
                NativeTerminationCleanupBoundaryKind::Hook(
                    NativeTerminationHookBoundary::TryFinally,
                ),
            ),
            (
                NativeTerminationCleanupAction::ExceptionUnwind,
                NativeTerminationCleanupBoundaryKind::Hook(
                    NativeTerminationHookBoundary::Exception,
                ),
            ),
            (
                NativeTerminationCleanupAction::OutputBufferStack,
                NativeTerminationCleanupBoundaryKind::Hook(
                    NativeTerminationHookBoundary::OutputBuffer,
                ),
            ),
            (
                NativeTerminationCleanupAction::ShutdownQueue,
                NativeTerminationCleanupBoundaryKind::Hook(NativeTerminationHookBoundary::Shutdown),
            ),
            (
                NativeTerminationCleanupAction::DestructorQueue,
                NativeTerminationCleanupBoundaryKind::Hook(
                    NativeTerminationHookBoundary::Destructor,
                ),
            ),
            (
                NativeTerminationCleanupAction::DiscardedNativeTemporaries,
                NativeTerminationCleanupBoundaryKind::Hook(
                    NativeTerminationHookBoundary::ExpressionContext,
                ),
            ),
        ];
        for (action, expected_boundary) in hook_cases {
            let stack = NativeTerminationCleanupStack::from_action(action);
            assert_eq!(
                stack.unlowered_runtime_cleanup_boundary_kind(),
                Some(expected_boundary)
            );
        }

        let goto_after_live_stack = NativeTerminationCleanupStack::from_actions(vec![
            NativeTerminationCleanupAction::LiveNativeValueHandle("status".to_string()),
            NativeTerminationCleanupAction::GotoScope,
        ]);
        assert_eq!(
            goto_after_live_stack.unlowered_runtime_cleanup_boundary_kind(),
            Some(NativeTerminationCleanupBoundaryKind::Hook(
                NativeTerminationHookBoundary::GotoScope
            ))
        );
    }

    #[test]
    fn termination_return_plan_centralizes_status_handle_cleanup_and_blockers() {
        let executable = native_value_effect_with_cleanup_tail(
            "status",
            vec![NativeTerminationCleanupAction::LiveNativeValueHandle(
                "temporary".to_string(),
            )],
        );
        let NativeTerminationReturnPlan::Executable(plan) = executable.runtime_return_plan() else {
            panic!("live native handles should be executable termination return cleanup");
        };
        assert_eq!(plan.status_value().map(String::as_str), Some("status"));
        let handles = plan
            .live_native_value_handles()
            .iter()
            .map(|handle| handle.as_str())
            .collect::<Vec<_>>();
        assert_eq!(handles, ["status", "temporary"]);

        let return_cases: Vec<(
            Vec<NativeTerminationCleanupAction<String>>,
            NativeTerminationCleanupBoundaryKind,
        )> = vec![
            (
                vec![
                    NativeTerminationCleanupAction::DiscardedNativeTemporaries,
                    NativeTerminationCleanupAction::BranchBlockLocals(
                        empty_branch_local_cleanup_plan(NativeBranchEffectJoin::BothContinue),
                    ),
                ],
                NativeTerminationCleanupBoundaryKind::BranchMerge,
            ),
            (
                vec![NativeTerminationCleanupAction::LoopScope],
                NativeTerminationCleanupBoundaryKind::Hook(
                    NativeTerminationHookBoundary::LoopScope,
                ),
            ),
            (
                vec![NativeTerminationCleanupAction::ReturnContext],
                NativeTerminationCleanupBoundaryKind::Hook(
                    NativeTerminationHookBoundary::ReturnContext,
                ),
            ),
            (
                vec![NativeTerminationCleanupAction::ExceptionUnwind],
                NativeTerminationCleanupBoundaryKind::Hook(
                    NativeTerminationHookBoundary::Exception,
                ),
            ),
            (
                vec![
                    NativeTerminationCleanupAction::OutputBufferStack,
                    NativeTerminationCleanupAction::ShutdownQueue,
                    NativeTerminationCleanupAction::DestructorQueue,
                ],
                NativeTerminationCleanupBoundaryKind::Hook(
                    NativeTerminationHookBoundary::OutputBuffer,
                ),
            ),
        ];
        for (tail_actions, expected_boundary) in return_cases {
            let effect = native_value_effect_with_cleanup_tail("status", tail_actions);
            let NativeTerminationReturnPlan::Blocked(boundary) = effect.runtime_return_plan()
            else {
                panic!("cleanup blocker should be resolved before backend emission");
            };
            assert_eq!(boundary, expected_boundary);
        }
    }
}
