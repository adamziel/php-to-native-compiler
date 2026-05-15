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
fn header_accepts_current_noop_signature() {
    let execution = run_source(
        r#"<?php
echo "before";
$result = header("HTTP/1.1 500 Internal Server Error", true, 500);
echo $result === null ? "|null" : "|not-null";
header("Content-Type: text/html; charset=utf-8");
header("X-No-Replace: one", false);
echo "|after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "before|null|after");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn header_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "header";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$result = $call("X-Test: dynamic", true, 204);
echo $result === null ? "|null" : "|not-null";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|null");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn header_rejects_forms_outside_current_subset() {
    let missing = runtime_error(
        r#"<?php
echo header();
"#,
    );
    assert_eq!(missing.line, 2);
    assert_eq!(missing.column, 6);
    assert_eq!(
        missing.message,
        "arity mismatch for header(): expected 1 to 3 argument(s), got 0"
    );

    let header = runtime_error(
        r#"<?php
echo header(42);
"#,
    );
    assert_eq!(header.line, 2);
    assert_eq!(header.column, 6);
    assert_eq!(
        header.message,
        "unsupported call header(): header argument must be string in the current subset, got int"
    );

    let replace = runtime_error(
        r#"<?php
echo header("X-Test: one", "yes");
"#,
    );
    assert_eq!(replace.line, 2);
    assert_eq!(replace.column, 6);
    assert_eq!(
        replace.message,
        "unsupported call header(): replace argument must be bool in the current subset, got string"
    );

    let response_code = runtime_error(
        r#"<?php
echo header("X-Test: one", true, "500");
"#,
    );
    assert_eq!(response_code.line, 2);
    assert_eq!(response_code.column, 6);
    assert_eq!(
        response_code.message,
        "unsupported call header(): response_code argument must be int in the current subset, got string"
    );
}

#[test]
fn emit_ir_rejects_header_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
header("Content-Type: text/html");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_includes_header_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("header") ? "1" : "0";
echo is_callable("header") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
