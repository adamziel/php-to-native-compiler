use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{codegen::emit_native_executable_c_source, emit_asm_source, emit_ir_source};
use php_compiler::{parse, run_source};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_DYNAMIC_FUNCTION_CALL_REJECTION: &str = "LLVM dynamic function-call lowering rejects variable-call expressions such as $name(...) until native callable expression evaluation, runtime function lookup, stack frames, arity/type diagnostics, callback dispatch, and exact native callable errors exist; phpc run handles current string-valued dynamic function calls";
const LLVM_FUNCTION_DECLARATION_REJECTION: &str = "LLVM user-function lowering rejects function declarations and return statements until native function symbol tables, stack-frame layout, default parameter binding, recursion guards, return-value flow, and exact native error behavior exist; phpc run handles current user-function declaration and return behavior";
const LLVM_METHOD_CALL_REJECTION: &str = "LLVM method-call lowering rejects instance, named static, object static-receiver, self::, parent::, and static:: method calls until native method lookup, receiver/static receiver resolution, $this and late-static-binding context, argument/arity diagnostics, visibility checks, references/copy-on-write, and exact native method-call errors exist; phpc run handles current bounded method-call behavior";
const LLVM_OBJECT_INSTANTIATION_REJECTION: &str = "LLVM object-instantiation lowering rejects new expressions and constructor dispatch until native object allocation, object handles, constructor calls, visibility checks, autoload/class lookup, references/copy-on-write, and exact native object-instantiation errors exist; phpc run handles current bounded new behavior";
const LLVM_REFERENCE_ASSIGNMENT_REJECTION: &str = "LLVM reference-assignment lowering rejects direct variable, array-offset, object-property, function-call, method-call, static-call, magic __get, and ArrayAccess reference sources or targets until native reference containers, alias-aware symbol tables, copy-on-write, object/property alias roots, and exact native error behavior exist; phpc run handles current bounded reference-assignment behavior";
const LLVM_ARRAY_ACCESS_REJECTION: &str = "LLVM ArrayAccess lowering rejects object offset reads/writes/isset/empty/unset/compound paths until native ArrayAccess dispatch for offsetGet(), offsetSet(), offsetExists(), and offsetUnset(), object handles, references/copy-on-write, and exact PHP diagnostics exist; phpc run handles current bounded ArrayAccess behavior";
const ASSEMBLY_FUNCTION_CALL_REJECTION: &str = "assembly function-call lowering rejects function calls outside the bounded generated-C user-function frame subset, including unknown user functions, callable builtins outside define()/constant()/defined(), arity-mismatched direct calls, unsupported by-reference argument binding, and unsupported dynamic string-valued calls, until full callable lookup, full arity/type diagnostics, callbacks, and cleanup handoff exist; generated-native C lowers supported by-value fixed/default/variadic direct, supported direct and compiler-known single-target by-reference frames, finite known-string dynamic, and runtime string-valued dynamic user-function frames";
const ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION: &str = "assembly dynamic function-call lowering rejects variable-call expressions outside the bounded generated-C finite known-string dispatch to registered user-function frames, supported native builtin families, or supported mixed callable target sets, runtime string-valued dispatch to registered user-function frames or supported native builtin families, and descriptor-backed closure values, including unknown callables, unsupported runtime callable builtin families, unsupported finite target sets, unsupported by-reference argument carriers, callbacks, methods, non-descriptor closures, and exact native callable errors; phpc run handles broader dynamic function calls";
const ASSEMBLY_CLOSURE_REJECTION: &str = "assembly closure lowering rejects closure shapes outside the bounded generated-C descriptor-backed closure frame subset, including by-reference closure captures that cannot be materialized through root symbol/reference handles or promoted frame locals, by-reference variadic closure parameters, by-reference closure returns, unsupported closure bodies, references/copy-on-write, and exact native callable errors; generated-native C lowers supported descriptor closures, supported static arrow closures, by-value captures, supported by-reference captures, implicit by-value arrow captures, non-static $this closure binding, typed/default/variadic by-value closure parameters, and untyped by-reference closure parameters through dynamic callable dispatch";
const ASSEMBLY_FUNCTION_DECLARATION_REJECTION: &str = "assembly user-function lowering rejects function declarations outside the bounded generated-C frame subset, including nested functions, unsupported typed/default/variadic by-reference parameters, malformed variadic declarations, unsupported parameter or return type metadata, static locals, and unsupported body cleanup, until full native function symbol tables, stack-frame layout, complete callable lookup, return-value flow, and exact native error behavior exist; generated-native C lowers supported by-value fixed/default/variadic direct, supported direct and compiler-known single-target by-reference frames, finite known-string dynamic, and runtime string-valued dynamic user-function frames with bounded scalar/array type enforcement";
const ASSEMBLY_CONDITIONAL_REJECTION: &str = "assembly conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior";
const ASSEMBLY_METHOD_CALL_REJECTION: &str = "assembly method-call lowering rejects method calls outside the bounded generated-C public declared instance/static method subset, including unsupported dynamic method-name dispatch, self::, parent::, static::, unsupported method declarations, unsupported receiver classes, visibility contexts, references/copy-on-write, and exact native method-call errors; generated-native C lowers supported public declared instance methods with $this frame binding, runtime string-valued dynamic public instance methods through declared-frame dispatch, supported named public static methods without $this, and supported object static-receiver calls through static source-call carriers";
const ASSEMBLY_OBJECT_INSTANTIATION_REJECTION: &str = "assembly object-instantiation lowering rejects new expressions outside the bounded generated-C declared-object constructor subset, including unsupported constructor declarations, non-public/static constructors, destructor-observable cleanup, visibility contexts, autoload/class lookup, references/copy-on-write, and exact native object-instantiation errors; generated-native C lowers supported named and runtime string-valued declared object allocation for destructor-free declared classes, constructorless argument evaluation, public constructors with $this frame binding, and explicit constructor value-return diagnostics";
const ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION: &str = "assembly reference-assignment lowering rejects direct variable, array-offset, object-property, function-call, method-call, static-call, magic __get, and ArrayAccess reference sources or targets until native reference containers, alias-aware symbol tables, copy-on-write, object/property alias roots, and exact native error behavior exist; phpc run handles current bounded reference-assignment behavior";
const ASSEMBLY_ARRAY_ACCESS_REJECTION: &str = "assembly ArrayAccess lowering rejects object offset reads/writes/isset/empty/unset/compound paths until native ArrayAccess dispatch for offsetGet(), offsetSet(), offsetExists(), and offsetUnset(), object handles, references/copy-on-write, and exact PHP diagnostics exist; phpc run handles current bounded ArrayAccess behavior";
const ASSEMBLY_TRY_BLOCK_REJECTION: &str = "assembly try/catch/finally lowering rejects try blocks outside the bounded generated-C normal-flow subset until native Throwable objects, stack unwinding, catch type matching, catch variable binding, finally execution during break/continue/return/exit/goto/throw control flow, stack traces, references/copy-on-write, and exact native try-block diagnostics exist; generated-native C executes try bodies, skips catches, and runs finally bodies only when no unwinding-capable transfer is present";

#[test]
fn phpc_run_still_handles_current_function_call_subset() {
    let execution = run_source(
        r#"<?php
echo strlen("abc"), "\n";
function label($value) {
    return $value . "!";
}
echo label("user"), "\n";
$call = "label";
echo $call("dynamic"), "\n";
$builtin = "strlen";
echo $builtin("callable");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "3\nuser!\ndynamic!\n8");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_direct_builtin_and_user_calls_with_specific_boundary() {
    for source in [
        "<?php\necho label(\"user\");\nfunction label($value) { return $value; }\n",
        "<?php\necho dirname(\"/a/b.php\");\n",
        "<?php\nassert(true);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn emit_ir_folds_direct_strlen_known_strings() {
    let ir = emit_ir_source(
        r#"<?php
$known = "native";
echo strlen("abc"), "\n";
echo strlen($known), "\n";
echo strlen(true ? "same" : "size"), "\n";
"#,
    )
    .unwrap();

    assert!(ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 3)"));
    assert!(ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 6)"));
    assert!(ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 4)"));
    assert!(!ir.contains("strlen"));
}

#[test]
fn emit_ir_rejects_direct_strlen_unsupported_operands() {
    for source in [
        "<?php\necho strlen(123);\n",
        "<?php\necho strlen(\"abc\", \"extra\");\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn emit_ir_routes_supported_direct_call_argument_results_through_call_boundary() {
    for (source, line, column) in [
        ("<?php\necho strlen(missing());\n", 2, 6),
        ("<?php\necho strrev(missing());\n", 2, 6),
        (
            "<?php\n$factory = \"strlen\";\necho function_exists($factory());\n",
            3,
            6,
        ),
        (
            "<?php\necho is_callable(function () { return \"x\"; });\n",
            2,
            6,
        ),
        ("<?php\necho class_exists(new Box());\n", 2, 6),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn native_executable_c_source_routes_supported_direct_call_argument_results_through_call_boundary()
{
    for (source, line, column) in [
        ("<?php\necho strlen(missing());\n", 2, 6),
        ("<?php\necho strrev(missing());\n", 2, 13),
        (
            "<?php\n$factory = \"strlen\";\necho function_exists($factory());\n",
            3,
            6,
        ),
        (
            "<?php\necho is_callable(function () { return \"x\"; });\n",
            2,
            6,
        ),
        ("<?php\necho class_exists(new Box());\n", 2, 6),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, ASSEMBLY_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn emit_ir_routes_unsupported_direct_call_argument_results_through_call_boundary() {
    for (source, line, column) in [
        ("<?php\necho str_starts_with(missing(), \"x\");\n", 2, 6),
        (
            "<?php\n$factory = \"make_path\";\necho basename($factory(), \".php\");\n",
            3,
            6,
        ),
        ("<?php\necho file_get_contents(new PathName());\n", 2, 6),
        ("<?php\necho fopen($box->path(), \"r\");\n", 2, 6),
        ("<?php\necho header(make_header());\n", 2, 6),
        ("<?php\necho ob_start(callback_factory());\n", 2, 6),
        ("<?php\necho array_map(callback_factory(), []);\n", 2, 6),
        ("<?php\necho get_object_vars(new Box());\n", 2, 6),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn native_executable_c_source_routes_unsupported_direct_call_argument_results_through_call_boundary(
) {
    for (source, line, column) in [
        ("<?php\necho str_starts_with(missing(), \"x\");\n", 2, 6),
        (
            "<?php\n$factory = \"make_path\";\necho basename($factory(), \".php\");\n",
            3,
            6,
        ),
        ("<?php\necho file_get_contents(new PathName());\n", 2, 6),
        ("<?php\necho fopen($box->path(), \"r\");\n", 2, 6),
        ("<?php\necho header(make_header());\n", 2, 6),
        ("<?php\necho ob_start(callback_factory());\n", 2, 6),
        ("<?php\necho array_map(callback_factory(), []);\n", 2, 16),
        ("<?php\necho get_object_vars(new Box());\n", 2, 6),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, ASSEMBLY_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn emit_ir_routes_constant_table_call_argument_results_through_call_boundary() {
    for (source, line, column) in [
        ("<?php\necho defined(strlen(\"abc\"));\n", 2, 6),
        (
            "<?php\n$resolver = \"constant_name\";\necho constant($resolver());\n",
            3,
            6,
        ),
        ("<?php\ndefine($box->name(), \"value\");\n", 2, 1),
        ("<?php\ndefine(\"APP\", new ValueName());\n", 2, 1),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn native_executable_c_source_routes_constant_table_call_argument_results_through_call_boundary() {
    for (source, line, column) in [
        ("<?php\necho defined(strlen(\"abc\"));\n", 2, 6),
        (
            "<?php\n$resolver = \"constant_name\";\necho constant($resolver());\n",
            3,
            6,
        ),
        ("<?php\ndefine($box->name(), \"value\");\n", 2, 1),
        ("<?php\ndefine(\"APP\", new ValueName());\n", 2, 1),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, ASSEMBLY_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn emit_ir_routes_direct_special_forms_through_call_boundary() {
    for (source, expected) in [
        ("<?php\n$value = defined();\n", LLVM_FUNCTION_CALL_REJECTION),
        (
            "<?php\n$value = defined(\"APP\", \"EXTRA\");\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        ("<?php\n$value = empty();\n", LLVM_FUNCTION_CALL_REJECTION),
        (
            "<?php\n$value = empty($value, $other);\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        ("<?php\n$value = isset();\n", LLVM_FUNCTION_CALL_REJECTION),
        (
            "<?php\n$value = isset($items[missing_key()]);\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing_key\";\n$value = empty($items[$call()]);\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = isset($items[$box->key()]);\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = empty(make_items()[0]);\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn native_executable_c_source_routes_direct_special_forms_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\n$value = defined();\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = defined(\"APP\", \"EXTRA\");\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = empty();\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = empty($value, $other);\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = isset();\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = isset($items[missing_key()]);\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = empty($items[(\"count\")([1])]);\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = isset($items[$box->key()]);\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$value = empty(make_items()[0]);\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn emit_ir_routes_assignment_and_unset_lvalue_operand_calls_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\n$items[key_name()] = 1;\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"key_name\";\n$items[$call()] = 1;\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$items[$box->key()] = 1;\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$items[new Key()] = 1;\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
        ),
        (
            "<?php\nunset($items[key_name()]);\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"key_name\";\nunset($items[$call()]);\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\nunset($items[$box->key()]);\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\nunset($items[new Key()]);\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn native_executable_c_source_routes_assignment_and_unset_lvalue_operand_calls_through_call_boundary(
) {
    for (source, expected) in [
        (
            "<?php\n$items[key_name()] = 1;\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"key_name\";\n$items[$call()] = 1;\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$items[$box->key()] = 1;\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$items[new Key()] = 1;\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
        (
            "<?php\nunset($items[key_name()]);\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"key_name\";\nunset($items[$call()]);\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\nunset($items[$box->key()]);\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\nunset($items[new Key()]);\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn emit_ir_routes_object_arrayaccess_write_operations_through_shared_boundary() {
    for source in [
        "<?php\n$box->items[0] = 1;\n",
        "<?php\n$name = \"items\";\n$box->$name[0] = 1;\n",
        "<?php\n$box->items[] = 1;\n",
        "<?php\n$box->items[0] += 2;\n",
        "<?php\n++$box->items[0];\n",
        "<?php\nunset($box->items[0]);\n",
        "<?php\nunset($local, $box->items[0]);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_ARRAY_ACCESS_REJECTION);
    }
}

#[test]
fn native_executable_c_source_routes_object_arrayaccess_write_operations_through_shared_boundary() {
    for source in [
        "<?php\n$box->items[0] = 1;\n",
        "<?php\n$name = \"items\";\n$box->$name[0] = 1;\n",
        "<?php\n$box->items[] = 1;\n",
        "<?php\n$box->items[0] += 2;\n",
        "<?php\n++$box->items[0];\n",
        "<?php\nunset($box->items[0]);\n",
        "<?php\nunset($local, $box->items[0]);\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, ASSEMBLY_ARRAY_ACCESS_REJECTION);
    }
}

#[test]
fn object_arrayaccess_write_lvalue_operands_still_route_before_shared_boundary() {
    for (source, llvm_expected, assembly_expected) in [
        (
            "<?php\n$box->items[key_name()] = 1;\n",
            LLVM_FUNCTION_CALL_REJECTION,
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"key_name\";\nunset($box->items[$call()]);\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$box->items[$receiver->key()] += 1;\n",
            LLVM_METHOD_CALL_REJECTION,
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
    ] {
        let llvm_error = emit_ir_source(source).unwrap_err();
        assert_eq!(llvm_error.phase, Phase::Codegen);
        assert_eq!(llvm_error.message, llvm_expected);

        let program = parse(source).unwrap();
        let assembly_error = emit_native_executable_c_source(&program).unwrap_err();
        assert_eq!(assembly_error.phase, Phase::Codegen);
        assert_eq!(assembly_error.message, assembly_expected);
    }
}

#[test]
fn emit_ir_routes_reference_source_lvalue_operand_calls_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\n$alias =& $items[key_name()];\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"key_name\";\n$alias =& $items[$call()];\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$alias =& $items[$box->key()];\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$alias =& $items[new Key()];\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
        ),
        (
            "<?php\n$alias =& identity(missing_call());\n",
            LLVM_REFERENCE_ASSIGNMENT_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn native_executable_c_source_routes_reference_source_lvalue_operand_calls_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\n$alias =& $items[key_name()];\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"key_name\";\n$alias =& $items[$call()];\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$alias =& $items[$box->key()];\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$alias =& $items[new Key()];\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
        (
            "<?php\n$alias =& identity(missing_call());\n",
            ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION,
        ),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn emit_ir_routes_reference_assignment_target_calls_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\n$items[key_name()] =& $value;\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"key_name\";\n$items[$call()] =& $value;\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$items[$box->key()] =& $value;\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$items[new Key()] =& $value;\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn native_executable_c_source_routes_reference_assignment_target_calls_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\n$items[key_name()] =& $value;\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"key_name\";\n$items[$call()] =& $value;\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$items[$box->key()] =& $value;\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$items[new Key()] =& $value;\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn emit_ir_routes_value_operand_call_results_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\necho [missing_value()];\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\n$value = [$call()];\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\nprint [$box->value()];\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\nprint [new Value()];\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
        ),
        (
            "<?php\necho (missing_value() == 1);\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\necho ($call() == 1);\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\necho ($box->value() == 1);\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\necho (new Value() == 1);\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
        ),
        (
            "<?php\necho (missing_value() . \"x\");\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\necho ($call() . \"x\");\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\necho ($box->value() . \"x\");\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\necho (new Value() . \"x\");\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn native_executable_c_source_routes_value_operand_call_results_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\necho [missing_value()];\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\n$value = [$call()];\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\nprint [$box->value()];\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\nprint [new Value()];\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
        (
            "<?php\necho (missing_value() == 1);\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\necho ($call() == 1);\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\necho ($box->value() == 1);\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\necho (new Value() == 1);\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
        (
            "<?php\necho (missing_value() . \"x\");\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\necho ($call() . \"x\");\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\necho ($box->value() . \"x\");\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\necho (new Value() . \"x\");\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn emit_ir_routes_exit_construct_arguments_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\nexit(missing_status());\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"status\";\ndie($call());\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\nexit($status->code());\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\ndie(new ExitStatus());\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn native_executable_c_source_routes_exit_construct_arguments_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\nexit(missing_status());\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"status\";\ndie($call());\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\nexit($status->code());\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\ndie(new ExitStatus());\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn emit_ir_routes_statement_operand_call_results_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\nif (missing()) { echo 1; }\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\nfor ($i = start(); $i < 3; $i++) { }\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"ready\";\nwhile ($call()) { break; }\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\nswitch ($box->kind()) { default: break; }\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\nforeach (new Items() as $item) { echo $item; }\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
        ),
        (
            "<?php\nrequire path_factory();\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\nthrow new Failure();\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn native_executable_c_source_routes_statement_operand_call_results_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\nif (missing()) { echo 1; }\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\nfor ($i = start(); $i < 3; $i++) { }\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"ready\";\nwhile ($call()) { break; }\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\nswitch ($box->kind()) { default: break; }\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\nforeach (new Items() as $item) { echo $item; }\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
        (
            "<?php\nrequire path_factory();\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\nthrow new Failure();\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn native_statement_preflight_routes_try_catch_finally_body_calls_through_call_boundary() {
    for (source, llvm_expected, assembly_expected) in [
        (
            "<?php\ntry { return missing(); } catch (Exception $e) {}\n",
            LLVM_FUNCTION_CALL_REJECTION,
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\ntry { if (true) { missing(); } } catch (Exception $e) {}\n",
            LLVM_FUNCTION_CALL_REJECTION,
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\ntry { echo \"ok\"; } catch (Exception $e) { $box->work(); }\n",
            LLVM_METHOD_CALL_REJECTION,
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\ntry { echo \"ok\"; } finally { throw new Failure(); }\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, llvm_expected);

        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, assembly_expected);
    }
}

#[test]
fn native_try_cleanup_unwind_requirements_route_control_transfers_through_shared_boundary() {
    for source in [
        "<?php\ntry { return 1; } catch (Exception $e) {}\n",
        "<?php\ntry { while (true) { break; } } catch (Exception $e) {}\n",
        "<?php\ntry { while (true) { continue; } } catch (Exception $e) {}\n",
        "<?php\ntry { goto done; } catch (Exception $e) {}\ndone:\n",
        "<?php\ntry { throw 1; } finally {}\n",
        "<?php\ntry { if (true) { exit(\"done\"); } } finally {}\n",
        "<?php\nclass Risk { public function __destruct() {} }\ntry { new Risk(); } finally {}\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, ASSEMBLY_TRY_BLOCK_REJECTION, "{source}");
    }
}

#[test]
fn native_executable_c_source_registers_allocatable_classes_only_without_destructor_cleanup_risk() {
    let source = r#"<?php
class CleanAllocation {}
class CleanInheritedAllocation extends CleanAllocation {}
class DirectDestructorAllocation {
    public function __destruct() { echo "direct"; }
}
class ParentDestructorAllocation {
    public function __destruct() { echo "parent"; }
}
class InheritedDestructorAllocation extends ParentDestructorAllocation {}
$name = isset($_GET["class"]) ? "CleanAllocation" : "CleanInheritedAllocation";
new $name();
"#;
    let generated = emit_native_executable_c_source(&parse(source).unwrap()).unwrap();
    let allocation_calls = generated
        .lines()
        .filter(|line| line.contains("= phpc_native_value_new_declared_class"))
        .count();

    assert_eq!(
        allocation_calls, 2,
        "only destructor-clean dynamic class candidates should be allocatable:\n{generated}"
    );
    assert!(
        generated.contains("phpc_native_value_dynamic_call_name_matches"),
        "dynamic allocation should still use the shared runtime class-name matcher:\n{generated}"
    );
    assert!(
        !generated.contains("object-instantiation lowering rejects")
            && !generated.contains("object/class lowering rejects"),
        "destructor-clean allocation candidates should not be blocked:\n{generated}"
    );
}

#[test]
fn destructor_observable_allocation_across_call_contexts() {
    for source in [
        r#"<?php
class DirectDestructor {
    public function __destruct() { echo "direct"; }
}
new DirectDestructor();
"#,
        r#"<?php
class ParentDestructor {
    public function __destruct() { echo "parent"; }
}
class ChildDestructor extends ParentDestructor {}
new ChildDestructor();
"#,
        r#"<?php
class KnownDynamicDestructor {
    public function __destruct() { echo "known"; }
}
$name = "KnownDynamicDestructor";
new $name();
"#,
        r#"<?php
class KnownCleanDynamic {}
class KnownDynamicChoiceDestructor {
    public function __destruct() { echo "known-choice"; }
}
$name = isset($_GET["class"]) ? "KnownCleanDynamic" : "KnownDynamicChoiceDestructor";
new $name();
"#,
        r#"<?php
class UnknownDynamicDestructor {
    public function __destruct() { echo "unknown"; }
}
new $name();
"#,
        r#"<?php
class NestedDestructor {
    public function __destruct() { echo "nested"; }
}
class ConstructorSink {
    public function __construct($value) {}
}
$name = "NestedDestructor";
new ConstructorSink(new $name());
"#,
        r#"<?php
class FunctionArgumentDestructor {
    public function __destruct() { echo "function-arg"; }
}
function sink($value) {}
sink(new FunctionArgumentDestructor());
"#,
        r#"<?php
class MethodArgumentDestructor {
    public function __destruct() { echo "method-arg"; }
}
class MethodSink {
    public function take($value) {}
}
$sink = new MethodSink();
$sink->take(new MethodArgumentDestructor());
"#,
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, ASSEMBLY_OBJECT_INSTANTIATION_REJECTION);
    }
}

#[test]
fn native_executable_c_source_keeps_destructor_free_dynamic_constructors_on_declared_path() {
    for source in [
        r#"<?php
class DirtyButUnselected {
    public function __destruct() { echo "dirty"; }
}
class CleanDynamicConstructor {
    public $value;
    public function __construct($value) { $this->value = $value; }
}
$name = "CleanDynamicConstructor";
$object = new $name("ok");
echo $object->value;
"#,
        r#"<?php
class DirtyFiniteUnselected {
    public function __destruct() { echo "dirty"; }
}
class FirstCleanDynamicConstructor {
    public function __construct($value) { echo $value; }
}
class SecondCleanDynamicConstructor {
    public function __construct($value) { echo $value; }
}
$name = isset($_GET["class"]) ? "FirstCleanDynamicConstructor" : "SecondCleanDynamicConstructor";
new $name("ok");
"#,
    ] {
        let generated = emit_native_executable_c_source(&parse(source).unwrap()).unwrap();

        assert!(
            generated.contains("phpc_native_value_dynamic_call_name_matches"),
            "destructor-free dynamic constructors should keep the runtime class-name dispatch path:\n{generated}"
        );
        assert!(
            !generated.contains("object-instantiation lowering rejects"),
            "destructor-free dynamic constructor should not be blocked:\n{generated}"
        );
    }
}

#[test]
fn native_executable_c_source_routes_symbol_environment_constructor_class_operands() {
    for (label, source) in [
        (
            "discard",
            r#"<?php
new $GLOBALS();
"#,
        ),
        (
            "assignment",
            r#"<?php
$value = new $_GET();
"#,
        ),
        (
            "echo",
            r#"<?php
echo new $_POST();
"#,
        ),
        (
            "array-value",
            r#"<?php
$items = [new $_REQUEST()];
"#,
        ),
    ] {
        let error = emit_native_executable_c_source(&parse(source).unwrap()).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen, "{label}");
        assert_eq!(
            error.message, ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
            "{label}"
        );
    }
}

#[test]
fn emit_ir_rejects_direct_calls_before_lowering_arguments() {
    for source in [
        "<?php\necho label([]);\nfunction label($value) { return $value; }\n",
        "<?php\nassert([]);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_dynamic_calls_before_lowering_callee_or_arguments() {
    for source in [
        "<?php\n$call = \"strlen\";\necho $call([]);\n",
        "<?php\n$call = \"assert\";\necho $call([]);\n",
        "<?php\n$call = \"strlen\";\necho $call(\"abc\");\n",
        "<?php\n$call = \"strlen\";\necho $call(\"abc\",);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_DYNAMIC_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn emit_ir_routes_call_operation_blockers_across_call_families() {
    for (source, expected) in [
        (
            "<?php\necho missing(1 + 2);\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\necho strlen(\"abc\", \"extra\");\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$flag = (1 + 2) === 3;\n$value = $flag ? \"1\" : \"nope\";\necho is_numeric($value);\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\necho missing(strlen(\"abc\"));\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = true ? \"strlen\" : \"count\";\necho $call(\"abc\");\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\necho $call(1 + 2, 3);\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\necho $call(strlen(\"abc\"));\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        ("<?php\nmissing()->value = 1;\n", LLVM_FUNCTION_CALL_REJECTION),
        (
            "<?php\n$call = \"missing\";\necho ($call()->value = 1);\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        ("<?php\necho missing()[0];\n", LLVM_FUNCTION_CALL_REJECTION),
        (
            "<?php\n$call = \"missing\";\necho $call()[1];\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        ("<?php\n$box->work(1 + 2);\n", LLVM_METHOD_CALL_REJECTION),
        (
            "<?php\n$method = true ? \"work\" : \"fallback\";\n$box->{$method}(1);\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"value\";\n$box->work($call());\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\necho $box->work()->value;\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\necho ($box->work()->value = 1);\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        ("<?php\nWorker::run(1, 2);\n", LLVM_METHOD_CALL_REJECTION),
        (
            "<?php\necho (new Box()->value = 1);\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
        ),
        (
            "<?php\nfunction identity($value) { return $value; }\n",
            LLVM_FUNCTION_DECLARATION_REJECTION,
        ),
        (
            "<?php\nfunction mutate(&$value) { return $value; }\n",
            LLVM_FUNCTION_DECLARATION_REJECTION,
        ),
        (
            "<?php\nfunction collect(callable ...$items) { return $items; }\n",
            LLVM_FUNCTION_DECLARATION_REJECTION,
        ),
        (
            "<?php\nfunction &borrow() { $value = 1; return $value; }\n",
            LLVM_FUNCTION_DECLARATION_REJECTION,
        ),
        (
            "<?php\nreturn strlen(\"abc\");\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$alias =& missing();\n",
            LLVM_REFERENCE_ASSIGNMENT_REJECTION,
        ),
        (
            "<?php\n$alias =& missing()[0];\n",
            LLVM_REFERENCE_ASSIGNMENT_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\n$alias =& $call();\n",
            LLVM_REFERENCE_ASSIGNMENT_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\n$alias =& $call()[1];\n",
            LLVM_REFERENCE_ASSIGNMENT_REJECTION,
        ),
        (
            "<?php\n$alias =& $box->work();\n",
            LLVM_REFERENCE_ASSIGNMENT_REJECTION,
        ),
        (
            "<?php\n$alias =& $box->work()[0];\n",
            LLVM_REFERENCE_ASSIGNMENT_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn native_executable_c_source_routes_call_operation_blockers_across_call_families() {
    for (source, expected) in [
        (
            "<?php\necho missing(1 + 2);\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\necho strlen(\"abc\", \"extra\");\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$flag = (1 + 2) === 3;\n$value = $flag ? \"1\" : \"nope\";\necho is_numeric($value);\n",
            ASSEMBLY_CONDITIONAL_REJECTION,
        ),
        (
            "<?php\necho missing(strlen(\"abc\"));\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$flag = isset($_GET[\"x\"]);\n$call = $flag ? \"strlen\" : \"count\";\necho $call(\"abc\");\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\necho $call(1 + 2, 3);\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\necho $call(strlen(\"abc\"));\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\nmissing()->value = 1;\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\necho ($call()->value = 1);\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\necho missing()[0];\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\necho $call()[1];\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$box->work(1 + 2);\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$method = true ? \"work\" : \"fallback\";\n$box->{$method}(1);\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"value\";\n$box->work($call());\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\necho $box->work()->value;\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\necho ($box->work()->value = 1);\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\nWorker::run(1, 2);\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\nfunction make_capture() { return function () use (&$missing) { return $missing; }; }\n$call = make_capture();\necho $call();\n",
            ASSEMBLY_CLOSURE_REJECTION,
        ),
        (
            "<?php\necho (new Box()->value = 1);\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
        (
            "<?php\nfunction mutate(int &$value) { return $value; }\n",
            ASSEMBLY_FUNCTION_DECLARATION_REJECTION,
        ),
        (
            "<?php\nfunction collect(callable ...$items) { return $items; }\n",
            ASSEMBLY_FUNCTION_DECLARATION_REJECTION,
        ),
        (
            "<?php\nfunction &borrow() { $value = 1; return $value; }\n",
            ASSEMBLY_FUNCTION_DECLARATION_REJECTION,
        ),
        (
            "<?php\n$alias =& missing();\n",
            ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION,
        ),
        (
            "<?php\n$alias =& missing()[0];\n",
            ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\n$alias =& $call();\n",
            ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\n$alias =& $call()[1];\n",
            ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION,
        ),
        (
            "<?php\n$alias =& $box->work();\n",
            ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION,
        ),
        (
            "<?php\n$alias =& $box->work()[0];\n",
            ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION,
        ),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn emit_ir_routes_unary_and_binary_operand_calls_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\n$value = -([] + missing());\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\n$value = -([] + $call());\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = -([] + $box->work());\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$value = -([] + new Box());\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
        ),
        (
            "<?php\n$value = ([] + missing()) + 1;\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\n$value = ([] + $call()) + 1;\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = ([] + $box->work()) + 1;\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$value = ([] + new Box()) + 1;\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn native_executable_c_source_routes_unary_and_binary_operand_calls_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\n$value = -([] + missing());\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\n$value = -([] + $call());\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = -([] + $box->work());\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$value = -([] + new Box());\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
        (
            "<?php\n$value = ([] + missing()) + 1;\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\n$value = ([] + $call()) + 1;\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = ([] + $box->work()) + 1;\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$value = ([] + new Box()) + 1;\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn emit_ir_routes_unemitted_binary_right_operand_calls_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\n$value = [] + missing();\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\n$value = [] == $call();\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = [] . $box->work();\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$value = [] . (new Box());\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn native_executable_c_source_routes_unemitted_binary_right_operand_calls_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\n$value = [] + missing();\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\n$value = [] == $call();\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$value = [] . $box->work();\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\n$value = [] . (new Box());\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn emit_ir_routes_unemitted_echo_operand_calls_through_call_boundary() {
    for (source, expected) in [
        ("<?php\necho [], missing();\n", LLVM_FUNCTION_CALL_REJECTION),
        (
            "<?php\n$call = \"missing\";\necho [], $call();\n",
            LLVM_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\necho [], $box->work();\n",
            LLVM_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\necho [], new Box();\n",
            LLVM_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn native_executable_c_source_routes_unemitted_echo_operand_calls_through_call_boundary() {
    for (source, expected) in [
        (
            "<?php\necho [], missing();\n",
            ASSEMBLY_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\n$call = \"missing\";\necho [], $call();\n",
            ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\necho [], $box->work();\n",
            ASSEMBLY_METHOD_CALL_REJECTION,
        ),
        (
            "<?php\necho [], new Box();\n",
            ASSEMBLY_OBJECT_INSTANTIATION_REJECTION,
        ),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn emit_asm_rejects_function_calls_before_backend_execution() {
    for source in [
        "<?php\necho label(\"abc\");\nfunction label($value) { return $value; }\n",
        "<?php\necho dirname(\"/a/b.php\");\n",
        "<?php\nassert(true);\n",
    ] {
        let error = emit_asm_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_dynamic_calls_before_backend_execution() {
    let error = emit_asm_source("<?php\n$call = \"strlen\";\necho $call(\"abc\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_DYNAMIC_FUNCTION_CALL_REJECTION);
}

#[test]
fn native_executable_c_source_executes_by_reference_closure_captures_across_consumers() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "$slot = \"old\";\n",
        "$get = function &() use (&$slot) { return $slot; };\n",
        "$alias =& $get();\n",
        "$alias = \"new\";\n",
        "echo $slot, \"|\", $get(), \"|\";\n",
        "function apply_closure($callback) { return $callback(); }\n",
        "echo apply_closure($get);\n",
    );
    let (source_path, output_path) =
        compile_native_function_call_fixture("by_reference_closure_captures", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run by-reference closure executable {}: {error}",
            output_path.display()
        )
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"new|new|new");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_routes_plain_reference_assignment_fallbacks_through_shared_reference_boundary(
) {
    for source in [
        "<?php\n$alias =& StaticSource::$slot;\n",
        "<?php\n$alias =& StaticSource::$slot[0];\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION);
    }
}

#[test]
fn native_executable_c_source_binds_object_property_reference_sources_across_reference_consumers() {
    if !has_cc() {
        return;
    }

    let source = r#"<?php
function replace(&$slot, $next) { $slot = $next; return $slot; }
function pick(&$slot) { return $slot; }
class PropertyReferenceSourceBox {
    public $direct;
    public $dynamic;
    public $items;
    public $target_source;
    public $appendItems;
}
$box = new PropertyReferenceSourceBox();
$direct =& $box->direct;
$direct = "D";
echo pick($box->direct), "|";
$name = "dynamic";
$dynamic =& $box->$name;
$dynamic = "Y";
echo pick($box->$name), "|";
$path =& $box->items["leaf"];
$path = "P";
echo pick($box->items["leaf"]), "|";
$targets = [];
$targets["alias"] =& $box->target_source;
replace($box->target_source, "T");
echo pick($targets["alias"]), "|";
$arrayAppend = [];
$arrayAlias =& $arrayAppend[];
$arrayAlias = "AA";
echo pick($arrayAppend[0]), "|";
$propertyAppend =& $box->appendItems[];
$propertyAppend = "PA";
echo pick($box->appendItems[0]);
"#;
    let generated = emit_native_executable_c_source(&parse(source).unwrap()).unwrap();
    assert!(
        generated.contains("phpc_native_value_public_property_reference_with_diagnostic_and_free")
    );

    let (source_path, output_path) =
        compile_native_function_call_fixture("object_property_reference_sources", source);
    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run object-property reference executable {}: {error}",
            output_path.display()
        )
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"D|Y|P|T|AA|PA");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_executes_descriptor_ready_closure_invocation() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "$prefix = \"P\";\n",
        "$join = function ($value) use ($prefix) { return $prefix . $value; };\n",
        "function apply_value($callback, $value) { return $callback($value); }\n",
        "echo $join(\"1\"), \"|\", apply_value($join, \"2\");\n",
    );
    let (source_path, output_path) =
        compile_native_function_call_fixture("descriptor_ready_closure_invocation", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run descriptor closure executable {}: {error}",
            output_path.display()
        )
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"P1|P2");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_direct_user_function_calls_use_runtime_callable_abi_across_arities() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "function zero() { return \"Z\"; }\n",
        "function fixed($left, $right) { return $left . $right; }\n",
        "function defaults($left, $right = \"D\") { return $left . $right; }\n",
        "function variadic($head, ...$tail) { return $head; }\n",
        "echo zero(), \"|\", fixed(\"F\", \"X\"), \"|\", defaults(\"A\"), \"|\", variadic(\"V\", \"1\", \"2\");\n",
    );

    let generated = emit_native_executable_c_source(&parse(source).unwrap()).unwrap();
    assert!(
        generated.contains("phpc_native_callable_lookup_invoke_value_with_diagnostic_and_free_arguments"),
        "direct user-function calls should consume the direct named lookup-plus-invoke source-call ABI:\n{generated}"
    );
    assert!(
        generated.contains("phpc_native_call_frame_read_value"),
        "registered user functions should be entered through runtime call frames:\n{generated}"
    );

    let (source_path, output_path) =
        compile_native_function_call_fixture("direct_user_function_callable_abi_arities", source);
    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run direct user-function callable ABI executable {}: {error}",
            output_path.display()
        )
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"Z|FX|AD|V");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_direct_user_function_calls_preserve_reference_arguments_through_runtime_callable_abi(
) {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "function append_marker(&$slot, $marker) { $slot = $slot . $marker; return $slot; }\n",
        "$value = \"A\";\n",
        "echo append_marker($value, \"B\"), \"|\", $value;\n",
    );

    let generated = emit_native_executable_c_source(&parse(source).unwrap()).unwrap();
    assert!(
        generated.contains("phpc_native_callable_lookup_invoke_value_with_diagnostic_and_free_arguments"),
        "by-reference direct calls should use the same direct named lookup-plus-invoke carrier:\n{generated}"
    );
    assert!(
        generated.contains("phpc_native_call_arguments_push_reference_and_free"),
        "by-reference direct arguments should be transported through runtime call arguments:\n{generated}"
    );
    assert!(
        generated.contains("phpc_native_call_frame_read_reference"),
        "by-reference function frames should read references from the runtime frame:\n{generated}"
    );

    let (source_path, output_path) = compile_native_function_call_fixture(
        "direct_user_function_callable_abi_references",
        source,
    );
    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run direct user-function callable ABI reference executable {}: {error}",
            output_path.display()
        )
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"AB|AB");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_direct_user_function_reference_returns_use_runtime_callable_frame_contract() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "function &borrow(&$slot) { return $slot; }\n",
        "function write_ref(&$slot, $value) { $slot = $value; return $slot; }\n",
        "$value = \"old\";\n",
        "$alias =& borrow($value);\n",
        "$alias = \"alias\";\n",
        "echo $value, \"|\";\n",
        "echo write_ref(borrow($value), \"consumer\"), \"|\", $value;\n",
    );

    let generated = emit_native_executable_c_source(&parse(source).unwrap()).unwrap();
    assert!(
        generated.contains("static phpc_NativeReferenceHandle phpc_user_function_"),
        "direct reference-return functions should lower to reference-returning generated-C frames:\n{generated}"
    );
    assert!(
        generated.contains("return phpc_native_call_result_from_reference(phpc_call_result);"),
        "direct reference-return callable wrappers should preserve result-slot ownership:\n{generated}"
    );
    assert!(
        generated.contains(
            "phpc_native_callable_lookup_invoke_reference_with_diagnostic_and_free_arguments"
        ),
        "reference consumers should use the source-call reference result carrier:\n{generated}"
    );
    assert!(
        generated.contains(
            "phpc_native_callable_lookup_invoke_value_with_diagnostic_and_free_arguments"
        ),
        "value consumers should continue to use the source-call value result carrier:\n{generated}"
    );
    assert!(
        generated.contains("phpc_native_call_arguments_push_reference_and_free"),
        "direct reference-return results should be reusable as by-reference consumer arguments:\n{generated}"
    );
    assert!(
        generated.contains("phpc_native_call_arguments_free(direct_callable_args"),
        "later argument failures should clean previously materialized direct source-call arguments:\n{generated}"
    );

    let (source_path, output_path) = compile_native_function_call_fixture(
        "direct_user_function_reference_return_callable_frame",
        source,
    );
    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run direct user-function reference-return executable {}: {error}",
            output_path.display()
        )
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"alias|consumer|consumer");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_c_source_direct_user_function_reference_returns_keep_alias_transfer_result_vectors() {
    let source = concat!(
        "<?php\n",
        "function &borrow(&$slot) { return $slot; }\n",
        "function make_value() { return \"made\"; }\n",
        "$alias =& borrow(make_value());\n",
    );

    let generated = emit_native_executable_c_source(&parse(source).unwrap()).unwrap();
    assert!(
        generated.contains("alias_transfer_arg_results"),
        "produced by-reference arguments should still be represented as call-result vectors:\n{generated}"
    );
    assert!(
        generated.contains(
            "phpc_native_call_frame_reference_parameter_alias_transfer_result_from_results_with_diagnostic"
        ),
        "produced by-reference arguments should use the runtime alias-transfer contract:\n{generated}"
    );
    assert!(
        generated.contains("phpc_native_call_result_take_reference_with_diagnostic_and_free"),
        "reference consumers of alias-transfer results should take a reference result explicitly:\n{generated}"
    );
}

#[test]
fn native_c_source_direct_user_function_reference_return_rejects_by_value_return_source() {
    let source = "<?php\nfunction &borrow($slot) { return $slot; }\n";
    let error = emit_native_executable_c_source(&parse(source).unwrap()).unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, ASSEMBLY_FUNCTION_DECLARATION_REJECTION);
}

#[test]
fn native_executable_c_source_persists_by_value_closure_captures_across_invocations() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "$prefix = \"A\";\n",
        "$join = function ($value) use ($prefix) { return $prefix . $value; };\n",
        "$prefix = \"B\";\n",
        "echo $join(\"1\"), \"|\", $join(\"2\");\n",
    );
    let (source_path, output_path) =
        compile_native_function_call_fixture("by_value_closure_capture_persistence", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run captured closure executable {}: {error}",
            output_path.display()
        )
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"A1|A2");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_captures_reference_backed_values_by_value_across_frame_families() {
    if !has_cc() {
        return;
    }

    let source = r#"<?php
function direct_capture(&$slot, $before, $after) {
    $slot = $before;
    $alias =& $slot;
    $copy = static function () use ($alias) { return $alias; };
    $slot = $after;
    return $copy;
}
class ReferenceBackedCaptureFactory {
    public static function static_capture(&$slot, $before, $after) {
        $slot = $before;
        $alias =& $slot;
        $copy = static function () use ($alias) { return $alias; };
        $slot = $after;
        return $copy;
    }
    public function method_capture(&$slot, $before, $after) {
        $slot = $before;
        $alias =& $slot;
        $copy = static function () use ($alias) { return $alias; };
        $slot = $after;
        return $copy;
    }
}
$closure_capture = function (&$slot, $before, $after) {
    $slot = $before;
    $alias =& $slot;
    $copy = static function () use ($alias) { return $alias; };
    $slot = $after;
    return $copy;
};
$direct = "old";
$direct_copy = direct_capture($direct, "D0", "D1");
echo $direct_copy(), ":", $direct, "|";
$static = "old";
$static_copy = ReferenceBackedCaptureFactory::static_capture($static, "S0", "S1");
echo $static_copy(), ":", $static, "|";
$method = "old";
$factory = new ReferenceBackedCaptureFactory();
$method_copy = $factory->method_capture($method, "M0", "M1");
echo $method_copy(), ":", $method, "|";
$closure = "old";
$closure_copy = $closure_capture($closure, "C0", "C1");
echo $closure_copy(), ":", $closure;
"#;
    let (source_path, output_path) =
        compile_native_function_call_fixture("reference_backed_by_value_closure_captures", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run reference-backed by-value closure capture executable {}: {error}",
            output_path.display()
        )
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"D0:D1|S0:S1|M0:M1|C0:C1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_source_closure_routes_value_and_reference_returns_through_shared_result_contract() {
    for source in [
        concat!(
            "<?php\n",
            "$prefix = \"P\";\n",
            "$join = function ($value) use ($prefix) { return $prefix . $value; };\n",
            "echo $join(\"1\");\n",
        ),
        concat!(
            "<?php\n",
            "$slot = \"old\";\n",
            "$get = function &() use (&$slot) { return $slot; };\n",
            "$alias =& $get();\n",
            "$alias = \"new\";\n",
            "echo $get();\n",
        ),
    ] {
        let program = parse(source).unwrap();
        let generated = emit_native_executable_c_source(&program).unwrap();

        assert!(
            generated.contains("phpc_NativeClosureInvocationResult")
                && generated.contains("phpc_native_closure_invoke_result")
                && generated.contains("phpc_native_closure_result_free"),
            "closure invocation should use the shared value/reference result contract:\n{generated}"
        );
    }
}

#[test]
fn native_function_call_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone169/native_function_call_boundary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone169/native_function_call_boundary_emit_ir.cli"),
    )
    .expect("native function-call CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_dynamic_function_call_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1128/native_dynamic_function_call_boundary.phpc-source");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone1128/native_dynamic_function_call_boundary_emit_ir.cli"),
    )
    .expect("native dynamic function-call IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_dynamic_function_call_emit_asm_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1128/native_dynamic_function_call_boundary.phpc-source");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone1128/native_dynamic_function_call_boundary_emit_asm.cli",
        ))
        .expect("native dynamic function-call assembly CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_strlen_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone562/native_strlen.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone562/native_strlen_emit_ir.cli"),
    )
    .expect("native strlen IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}

fn native_function_call_output_path(name: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "phpc-native-function-call-{name}-{}",
        std::process::id()
    ));
    path
}

fn compile_native_function_call_fixture(
    name: &str,
    source: &str,
) -> (std::path::PathBuf, std::path::PathBuf) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_function_call_output_path(name).with_extension("php");
    let output_path = native_function_call_output_path(name);
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(&source_path, source).expect("write native function-call fixture source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native function-call source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    (source_path, output_path)
}

fn has_cc() -> bool {
    Command::new("cc").arg("--version").output().is_ok()
}
