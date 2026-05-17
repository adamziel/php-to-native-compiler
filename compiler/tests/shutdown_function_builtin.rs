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
fn register_shutdown_function_runs_supported_callbacks_at_shutdown() {
    let execution = run_source(
        r#"<?php
function string_shutdown($value) {
    echo "string:", $value, "\n";
}
register_shutdown_function("string_shutdown", "late");
echo "string";

class Handler {
    public function handle($value) {
        echo "array:", $value, "\n";
    }
}
$handler = new Handler();
echo "|";
register_shutdown_function([$handler, "handle"], "later");
echo "array";

$call = "register_shutdown_function";
echo "|";
$call(function () {
    echo "not-now";
});
echo "closure";
echo "|body";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string|array|closure|bodystring:late\narray:later\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn register_shutdown_function_runs_callbacks_registered_during_shutdown() {
    let execution = run_source(
        r#"<?php
function first_shutdown($value) {
    echo "first:", $value, "\n";
    register_shutdown_function("second_shutdown", "nested");
}
function second_shutdown($value) {
    echo "second:", $value;
}
register_shutdown_function("first_shutdown", "outer");
echo "body\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "body\nfirst:outer\nsecond:nested");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn register_shutdown_function_runs_all_callbacks_after_exit() {
    let execution = run_source(
        r#"<?php
function first_shutdown() {
    echo "first\n";
}
function second_shutdown() {
    echo "second";
}
register_shutdown_function("first_shutdown");
register_shutdown_function("second_shutdown");
exit("body\n");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "body\nfirst\nsecond");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn register_shutdown_function_rejects_forms_outside_current_subset() {
    let arity = runtime_error(
        r#"<?php
register_shutdown_function();
"#,
    );
    assert_eq!(arity.line, 2);
    assert_eq!(arity.column, 1);
    assert_eq!(
        arity.message,
        "arity mismatch for register_shutdown_function(): expected at least 1 argument(s), got 0"
    );

    let non_callable = runtime_error(
        r#"<?php
register_shutdown_function(42);
"#,
    );
    assert_eq!(non_callable.line, 2);
    assert_eq!(non_callable.column, 1);
    assert_eq!(
        non_callable.message,
        "unsupported call register_shutdown_function(): callback argument must be string, array callable, or closure in the current subset, got int"
    );

    let missing_string = runtime_error(
        r#"<?php
register_shutdown_function("missing_shutdown");
"#,
    );
    assert_eq!(missing_string.line, 2);
    assert_eq!(missing_string.column, 1);
    assert_eq!(
        missing_string.message,
        "unsupported call register_shutdown_function(): callback must be a valid callable in the current subset"
    );

    let invalid_array = runtime_error(
        r#"<?php
register_shutdown_function(["Missing", "handle"]);
"#,
    );
    assert_eq!(invalid_array.line, 2);
    assert_eq!(invalid_array.column, 1);
    assert_eq!(
        invalid_array.message,
        "unsupported call register_shutdown_function(): callback must be a valid callable in the current subset"
    );
}

#[test]
fn emit_ir_rejects_register_shutdown_function_until_native_sapi_shutdown_exists() {
    let error = emit_ir_source(
        r#"<?php
register_shutdown_function("strlen", "abc");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_includes_register_shutdown_function_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("register_shutdown_function") ? "1" : "0";
echo is_callable("register_shutdown_function") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
