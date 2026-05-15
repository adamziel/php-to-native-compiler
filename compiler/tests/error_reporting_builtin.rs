use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn error_reporting_tracks_current_integer_mask_state() {
    let execution = run_source(
        r#"<?php
echo error_reporting();
echo "|";
echo error_reporting(0);
echo "|";
echo error_reporting();
echo "|";
echo error_reporting(E_ERROR | E_WARNING | E_PARSE);
echo "|";
echo error_reporting();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "32767|32767|0|0|7");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn error_reporting_exposes_current_php_error_constants() {
    let execution = run_source(
        r#"<?php
echo E_ERROR, "|", E_WARNING, "|", E_PARSE, "|", E_CORE_ERROR, "|";
echo E_CORE_WARNING, "|", E_COMPILE_ERROR, "|", E_USER_ERROR, "|";
echo E_USER_WARNING, "|", E_RECOVERABLE_ERROR, "|", E_ALL;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|2|4|16|32|64|256|512|4096|32767");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn error_reporting_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "error_reporting";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call(0);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|32767");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn error_reporting_rejects_forms_outside_current_subset() {
    let non_int = runtime_error(
        r#"<?php
error_reporting("0");
"#,
    );
    assert_eq!(non_int.line, 2);
    assert_eq!(non_int.column, 1);
    assert_eq!(
        non_int.message,
        "unsupported call error_reporting(): mask must be int in the current subset, got string"
    );

    let too_many = runtime_error(
        r#"<?php
error_reporting(0, 1);
"#,
    );
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for error_reporting(): expected 0 to 1 argument(s), got 2"
    );
}

#[test]
fn emit_ir_folds_error_reporting_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("error_reporting") ? "1" : "0";
echo is_callable("error_reporting") ? "1" : "0";
echo defined("E_ALL") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 3, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nerror_reporting(E_ALL);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
