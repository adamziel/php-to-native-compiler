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
const ASSEMBLY_FUNCTION_CALL_REJECTION: &str = "assembly function-call lowering rejects function calls outside the bounded generated-C user-function frame subset, including unknown user functions, callable builtins outside define()/constant()/defined(), arity-mismatched direct calls, unsupported by-reference argument binding, and unsupported dynamic string-valued calls, until full callable lookup, full arity/type diagnostics, callbacks, and cleanup handoff exist; generated-native C lowers supported by-value fixed/default/variadic direct, supported direct and compiler-known single-target by-reference frames, finite known-string dynamic, and runtime string-valued dynamic user-function frames";
const ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION: &str = "assembly dynamic function-call lowering rejects variable-call expressions outside the bounded generated-C finite known-string dispatch to registered user-function frames, supported native builtin families, or supported mixed callable target sets, runtime string-valued dispatch to registered user-function frames or supported native builtin families, and descriptor-backed closure values, including unknown callables, unsupported runtime callable builtin families, unsupported finite target sets, unsupported by-reference argument carriers, callbacks, methods, non-descriptor closures, and exact native callable errors; phpc run handles broader dynamic function calls";
const ASSEMBLY_CLOSURE_REJECTION: &str = "assembly closure lowering rejects closure shapes outside the bounded generated-C descriptor-backed closure frame subset, including arrow functions, by-reference closure captures, implicit arrow captures, by-reference/default/variadic closure parameters, by-reference closure returns, unsupported closure bodies, references/copy-on-write, and exact native callable errors; generated-native C lowers supported by-value descriptor closures and by-value captures through dynamic callable dispatch";
const ASSEMBLY_FUNCTION_DECLARATION_REJECTION: &str = "assembly user-function lowering rejects function declarations outside the bounded generated-C frame subset, including nested functions, unsupported typed/default/variadic by-reference parameters, malformed variadic declarations, unsupported parameter or return type metadata, static locals, and unsupported body cleanup, until full native function symbol tables, stack-frame layout, complete callable lookup, return-value flow, and exact native error behavior exist; generated-native C lowers supported by-value fixed/default/variadic direct, supported direct and compiler-known single-target by-reference frames, finite known-string dynamic, and runtime string-valued dynamic user-function frames with bounded scalar/array type enforcement";
const ASSEMBLY_CONDITIONAL_REJECTION: &str = "assembly conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior";
const ASSEMBLY_METHOD_CALL_REJECTION: &str = "assembly method-call lowering rejects method calls outside the bounded generated-C public declared instance/static method frame subset, including unsupported dynamic method-name dispatch, self::, parent::, static::, unsupported method declarations, unsupported receiver classes, visibility contexts, references/copy-on-write, and exact native method-call errors; generated-native C lowers supported public declared instance methods with $this frame binding, runtime string-valued dynamic public instance methods through declared-frame dispatch, supported named public static methods without $this, and supported object static-receiver calls through static frames";
const ASSEMBLY_OBJECT_INSTANTIATION_REJECTION: &str = "assembly object-instantiation lowering rejects new expressions outside the bounded generated-C declared-object constructor subset, including unsupported constructor declarations, non-public constructors, constructor returns, visibility contexts, autoload/class lookup, references/copy-on-write, and exact native object-instantiation errors; generated-native C lowers supported named and runtime string-valued declared object allocation, constructorless argument evaluation, and public constructors with $this frame binding";
const ASSEMBLY_REFERENCE_ASSIGNMENT_REJECTION: &str = "assembly reference-assignment lowering rejects direct variable, array-offset, object-property, function-call, method-call, static-call, magic __get, and ArrayAccess reference sources or targets until native reference containers, alias-aware symbol tables, copy-on-write, object/property alias roots, and exact native error behavior exist; phpc run handles current bounded reference-assignment behavior";

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
            "<?php\n$value = \"abc\";\n$call = function () use (&$value) { return $value; };\necho $call();\n",
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
