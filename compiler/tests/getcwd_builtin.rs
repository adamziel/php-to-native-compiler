use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn getcwd_returns_current_utf8_working_directory() {
    let expected = std::env::current_dir()
        .expect("test process has a current working directory")
        .into_os_string()
        .into_string()
        .expect("test current working directory is valid UTF-8");

    let execution = run_source("<?php\necho getcwd();\n").unwrap();

    assert_eq!(execution.stdout, expected);
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn getcwd_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "getcwd";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo is_dir($call()) ? "dir" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|dir");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn getcwd_rejects_arguments_in_current_subset() {
    let error = run_source("<?php\necho getcwd('/tmp');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "arity mismatch for getcwd(): expected 0 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_getcwd_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("getcwd") ? "1" : "0";
echo is_callable("getcwd") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho getcwd();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
