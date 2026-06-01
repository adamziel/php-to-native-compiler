use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_OBJECT_INSTANTIATION_REJECTION: &str = "LLVM object-instantiation lowering rejects new expressions and constructor dispatch until native object allocation, object handles, constructor calls, visibility checks, autoload/class lookup, references/copy-on-write, and exact native object-instantiation errors exist; phpc run handles current bounded new behavior";
const LLVM_OBJECT_METADATA_REJECTION: &str = "LLVM object-metadata lowering rejects object/class metadata builtins until native class metadata tables, object handles, inheritance/interface/trait/enum registries, property/method tables, autoload interaction, references/copy-on-write, and exact native object-metadata errors exist; phpc run handles current bounded object metadata behavior";

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
fn builtin_runtime_exception_catches_after_unresolved_multicatch_arm() {
    let execution = run_source(
        r#"<?php
try {
    throw new RuntimeException();
} catch (\FooEx | \RuntimeException $e) {
    echo get_class($e), "\n";
}
echo "after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "RuntimeException\nafter");
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
fn builtin_exception_constructor_arguments_initialize_message_state() {
    let execution = run_source(
        r#"<?php
$exception = new Exception("message");
echo $exception->getMessage();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "message");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn throw_new_exception_reports_uncaught_boundary_after_operand_evaluation() {
    let execution = run_source(
        r#"<?php
throw new Exception();
"#,
    )
    .unwrap();

    assert_eq!(execution.exit_code, 255);
    assert_eq!(
        execution.stdout,
        "Fatal error: Uncaught Exception in Command line code:2\nStack trace:\n#0 {main}\n  thrown in Command line code on line 2"
    );
}

#[test]
fn emit_ir_rejects_builtin_exception_instantiation_before_native_object_lowering() {
    let error = emit_ir_source("<?php\n$exception = new Exception();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_INSTANTIATION_REJECTION);
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
        assert_eq!(error.message, LLVM_OBJECT_METADATA_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_builtin_exception_metadata_folds_before_backend_execution() {
    let error =
        emit_asm_source("<?php\necho class_exists(\"Exception\") ? \"1\" : \"0\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_METADATA_REJECTION);
}

#[test]
fn emit_ir_rejects_throw_new_exception_before_native_object_lowering() {
    let error = emit_ir_source("<?php\nthrow new Exception(\"boom\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_INSTANTIATION_REJECTION);
}

#[test]
fn emit_asm_rejects_builtin_exception_instantiation_before_backend_execution() {
    let error = emit_asm_source("<?php\n$exception = new Exception();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_INSTANTIATION_REJECTION);
}
