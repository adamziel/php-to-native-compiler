use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_EXCEPTION_REJECTION: &str = "LLVM exception lowering rejects throw statements and try/catch/finally blocks until native Throwable objects, stack unwinding, catch/finally dispatch, stack traces, and exact native error behavior exist; phpc run handles the current exception boundary";
const LLVM_OBJECT_CLASS_REJECTION: &str = "LLVM object/class lowering rejects class declarations, inheritance metadata, object instantiation, constructor dispatch, public property reads/writes, instance method calls, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";

#[test]
fn builtin_exception_metadata_supports_lookup_instantiation_and_inheritance() {
    let execution = run_source(
        r#"<?php
echo class_exists("Exception"), "\n";
$exception = new Exception();
echo get_class($exception), "\n";
echo is_a($exception, "Exception") ? "yes" : "no", "\n";

class CustomException extends Exception {}
$custom = new CustomException();
echo get_parent_class($custom), "\n";
echo is_subclass_of($custom, "Exception") ? "yes" : "no", "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1\nException\nyes\nException\nyes\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn user_code_cannot_redeclare_builtin_exception_class() {
    let error = run_source(
        r#"<?php
class Exception {}
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "class Exception is already defined");
}

#[test]
fn builtin_exception_constructor_arguments_remain_explicitly_unsupported() {
    let error = run_source(
        r#"<?php
$exception = new Exception("message");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 14);
    assert_eq!(
        error.message,
        "unsupported object instantiation for Exception: constructor arguments are not implemented"
    );
}

#[test]
fn throw_new_exception_keeps_existing_runtime_boundary_without_evaluating_operand() {
    let error = run_source(
        r#"<?php
throw new Exception("boom");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call throw: exception objects and stack unwinding are not implemented"
    );
}

#[test]
fn emit_ir_rejects_builtin_exception_instantiation_before_native_object_lowering() {
    let error = emit_ir_source("<?php\n$exception = new Exception();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_ir_rejects_builtin_exception_metadata_folds_before_native_class_tables() {
    for source in [
        "<?php\necho class_exists(\"Exception\") ? \"1\" : \"0\";\n",
        "<?php\n$name = \"Exception\";\necho class_exists($name, false) ? \"1\" : \"0\";\n",
        "<?php\necho method_exists(\"Exception\", \"getMessage\") ? \"1\" : \"0\";\n",
        "<?php\necho is_a(\"Exception\", \"Exception\", true) ? \"1\" : \"0\";\n",
        "<?php\necho is_subclass_of(\"CustomException\", \"Exception\") ? \"1\" : \"0\";\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_builtin_exception_metadata_folds_before_backend_execution() {
    let error =
        emit_asm_source("<?php\necho class_exists(\"Exception\") ? \"1\" : \"0\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_ir_rejects_throw_new_exception_before_native_exception_lowering() {
    let error = emit_ir_source("<?php\nthrow new Exception(\"boom\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_EXCEPTION_REJECTION);
}

#[test]
fn emit_asm_rejects_builtin_exception_instantiation_before_backend_execution() {
    let error = emit_asm_source("<?php\n$exception = new Exception();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}
