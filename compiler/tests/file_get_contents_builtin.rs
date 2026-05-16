use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn file_get_contents_reads_empty_php_input_placeholder() {
    let execution = run_source(
        r#"<?php
$raw = file_get_contents("php://input");
echo $raw === "" ? "empty" : "non-empty";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "empty");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_get_contents_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "file_get_contents";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("php://input") === "" ? "empty" : "non-empty";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|empty");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_get_contents_rejects_forms_outside_current_subset() {
    let non_string = run_source("<?php\nfile_get_contents(42);\n").unwrap_err();
    assert_eq!(non_string.phase, Phase::Runtime);
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 1);
    assert_eq!(
        non_string.message,
        "unsupported call file_get_contents(): path argument must be string in the current subset, got int"
    );

    let stream = run_source("<?php\nfile_get_contents('php://memory');\n").unwrap_err();
    assert_eq!(stream.phase, Phase::Runtime);
    assert_eq!(stream.line, 2);
    assert_eq!(stream.column, 1);
    assert_eq!(
        stream.message,
        "unsupported call file_get_contents(): only php://input is supported in the current stream-wrapper subset"
    );

    let local_file = run_source("<?php\nfile_get_contents(__FILE__);\n").unwrap_err();
    assert_eq!(local_file.phase, Phase::Runtime);
    assert_eq!(local_file.line, 2);
    assert_eq!(local_file.column, 1);
    assert_eq!(
        local_file.message,
        "unsupported call file_get_contents(): local filesystem reads are not implemented in the current subset"
    );

    let too_many = run_source("<?php\nfile_get_contents('php://input', false);\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for file_get_contents(): expected 1 argument(s), got 2"
    );
}

#[test]
fn emit_ir_folds_file_get_contents_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("file_get_contents") ? "1" : "0";
echo is_callable("file_get_contents") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nfile_get_contents('php://input');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
