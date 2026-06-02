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
fn set_error_handler_accepts_current_callable_shapes_without_invoking_them() {
    let execution = run_source(
        r#"<?php
function quiet_error_handler() {
}
$previous = set_error_handler("quiet_error_handler");
echo $previous === null ? "null" : "other";

class Handler {
    public function handle() {
    }
}
$handler = new Handler();
$previous = set_error_handler([$handler, "handle"], E_WARNING);
echo "|";
echo is_string($previous) ? $previous : "other";

$call = "set_error_handler";
$previous = $call(function () {
    echo "not-now";
});
echo "|";
echo is_array($previous) ? "array" : "other";
echo "|body";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "null|quiet_error_handler|array|body");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn restore_error_handler_clears_current_bounded_handler() {
    let execution = run_source(
        r#"<?php
function quiet_error_handler() {
}
set_error_handler("quiet_error_handler", E_WARNING);
echo restore_error_handler() ? "true" : "false";
$previous = set_error_handler("quiet_error_handler");
echo "|";
echo $previous === null ? "null" : "other";
$call = "restore_error_handler";
echo "|";
echo $call() ? "dynamic" : "false";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "true|null|dynamic");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn restore_error_handler_restores_previous_bounded_handler_registration() {
    let execution = run_source(
        r#"<?php
function first_handler() {
}
function second_handler() {
}
echo set_error_handler("first_handler", E_WARNING) === null ? "null" : "other";
$previous = set_error_handler("second_handler", E_WARNING);
echo "|";
echo is_string($previous) ? $previous : "other";
echo "|";
echo restore_error_handler() ? "restored" : "failed";
$previous = set_error_handler("second_handler", E_WARNING);
echo "|";
echo is_string($previous) ? $previous : "other";
restore_error_handler();
restore_error_handler();
$previous = set_error_handler("first_handler", E_WARNING);
echo "|";
echo $previous === null ? "null" : "other";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "null|first_handler|restored|first_handler|null"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn func_get_args_error_handler_reports_php_shaped_arity_error() {
    let execution = run_source(
        r#"<?php
set_error_handler('func_get_args');
function trigger_handler($value) {
    echo $missing;
}
try {
    trigger_handler(1);
} catch (\Error $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "func_get_args() expects exactly 0 arguments, 4 given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn set_error_handler_rejects_forms_outside_current_subset() {
    let arity = runtime_error(
        r#"<?php
set_error_handler();
"#,
    );
    assert_eq!(arity.line, 2);
    assert_eq!(arity.column, 1);
    assert_eq!(
        arity.message,
        "arity mismatch for set_error_handler(): expected 1 to 2 argument(s), got 0"
    );

    let non_callable = runtime_error(
        r#"<?php
set_error_handler(42);
"#,
    );
    assert_eq!(non_callable.line, 2);
    assert_eq!(non_callable.column, 1);
    assert_eq!(
        non_callable.message,
        "unsupported call set_error_handler(): callback argument must be string, array callable, or closure in the current subset, got int"
    );

    let missing_string = runtime_error(
        r#"<?php
set_error_handler("missing_error_handler");
"#,
    );
    assert_eq!(missing_string.line, 2);
    assert_eq!(missing_string.column, 1);
    assert_eq!(
        missing_string.message,
        "unsupported call set_error_handler(): callback must be a valid callable in the current subset"
    );

    let invalid_array = runtime_error(
        r#"<?php
set_error_handler(["Missing", "handle"]);
"#,
    );
    assert_eq!(invalid_array.line, 2);
    assert_eq!(invalid_array.column, 1);
    assert_eq!(
        invalid_array.message,
        "unsupported call set_error_handler(): callback must be a valid callable in the current subset"
    );

    let invalid_mask = runtime_error(
        r#"<?php
function quiet_error_handler() {
}
set_error_handler("quiet_error_handler", "warnings");
"#,
    );
    assert_eq!(invalid_mask.line, 4);
    assert_eq!(invalid_mask.column, 1);
    assert_eq!(
        invalid_mask.message,
        "unsupported call set_error_handler(): error levels argument must be int in the current subset, got string"
    );
}

#[test]
fn restore_error_handler_rejects_forms_outside_current_subset() {
    let arity = runtime_error(
        r#"<?php
restore_error_handler("extra");
"#,
    );
    assert_eq!(arity.line, 2);
    assert_eq!(arity.column, 1);
    assert_eq!(
        arity.message,
        "arity mismatch for restore_error_handler(): expected 0 argument(s), got 1"
    );
}

#[test]
fn emit_ir_rejects_set_error_handler_until_native_error_routing_exists() {
    let error = emit_ir_source(
        r#"<?php
set_error_handler("strlen", E_WARNING);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
restore_error_handler();
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_includes_error_handler_builtins_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("set_error_handler") ? "1" : "0";
echo is_callable("set_error_handler") ? "1" : "0";
echo function_exists("restore_error_handler") ? "1" : "0";
echo is_callable("restore_error_handler") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 4, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
