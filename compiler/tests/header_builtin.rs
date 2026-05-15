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
fn header_remove_accepts_current_noop_signature() {
    let execution = run_source(
        r#"<?php
header("Last-Modified: today");
$result = header_remove("Last-Modified");
echo $result === null ? "null" : "not-null";
header_remove();
echo "|after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "null|after");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn header_remove_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "header_remove";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$result = $call("Last-Modified");
echo $result === null ? "|null" : "|not-null";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|null");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn headers_sent_accepts_current_no_header_state_signature() {
    let execution = run_source(
        r#"<?php
echo headers_sent() ? "sent" : "open";
header("X-Test: one");
echo "|";
$call = "headers_sent";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call() ? "sent" : "open";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "open|yes|callable|open");
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
fn headers_sent_rejects_forms_outside_current_subset() {
    let output_arg = runtime_error(
        r#"<?php
$file = "";
echo headers_sent($file);
"#,
    );
    assert_eq!(output_arg.line, 3);
    assert_eq!(output_arg.column, 6);
    assert_eq!(
        output_arg.message,
        "unsupported call headers_sent(): filename and line output arguments are not implemented; call without arguments in the current subset"
    );

    let too_many = runtime_error(
        r#"<?php
echo headers_sent("", 0, "extra");
"#,
    );
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 6);
    assert_eq!(
        too_many.message,
        "arity mismatch for headers_sent(): expected 0 to 2 argument(s), got 3"
    );
}

#[test]
fn header_remove_rejects_forms_outside_current_subset() {
    let non_string = runtime_error(
        r#"<?php
echo header_remove(42);
"#,
    );
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 6);
    assert_eq!(
        non_string.message,
        "unsupported call header_remove(): header name argument must be string in the current subset, got int"
    );

    let too_many = runtime_error(
        r#"<?php
echo header_remove("A", "B");
"#,
    );
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 6);
    assert_eq!(
        too_many.message,
        "arity mismatch for header_remove(): expected 0 to 1 argument(s), got 2"
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
fn emit_ir_rejects_header_remove_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source("<?php\nheader_remove('Last-Modified');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_rejects_headers_sent_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho headers_sent();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_includes_header_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("header") ? "1" : "0";
echo is_callable("header") ? "1" : "0";
echo function_exists("header_remove") ? "1" : "0";
echo is_callable("header_remove") ? "1" : "0";
echo function_exists("headers_sent") ? "1" : "0";
echo is_callable("headers_sent") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 6, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
