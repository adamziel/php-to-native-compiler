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
fn date_default_timezone_set_accepts_current_utc_slice() {
    let execution = run_source(
        r#"<?php
echo date_default_timezone_set("UTC") ? "utc" : "missing";
echo "|";
echo date_default_timezone_set("Nope/Zone") ? "valid" : "invalid";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "utc|invalid");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn date_default_timezone_set_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "date_default_timezone_set";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("UTC") ? "utc" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|utc");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn date_default_timezone_set_rejects_forms_outside_current_subset() {
    let arity = run_source(
        r#"<?php
date_default_timezone_set();
"#,
    )
    .unwrap();
    assert_eq!(
        arity.stdout,
        "Fatal error: Uncaught TypeError: Too few arguments to function date_default_timezone_set(), 0 passed in Command line code on line 2 and exactly 1 expected in Command line code:2\nStack trace:\n#0 {main}\n  thrown in Command line code on line 2"
    );
    assert_eq!(arity.stderr, "");
    assert_eq!(arity.exit_code, 255);

    let type_error = runtime_error(
        r#"<?php
date_default_timezone_set(42);
"#,
    );
    assert_eq!(type_error.line, 2);
    assert_eq!(type_error.column, 1);
    assert_eq!(
        type_error.message,
        "unsupported call date_default_timezone_set(): timezone identifier must be string in the current subset, got int"
    );
}

#[test]
fn emit_ir_rejects_date_default_timezone_set_until_native_time_state_exists() {
    let error = emit_ir_source(
        r#"<?php
date_default_timezone_set("UTC");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_includes_date_default_timezone_set_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("date_default_timezone_set") ? "1" : "0";
echo is_callable("date_default_timezone_set") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
