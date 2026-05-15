use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn mysqli_connect_is_visible_but_connections_are_an_explicit_boundary() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_connect";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable");
    assert_eq!(execution.exit_code, 0);

    let error = run_source(
        r#"<?php
mysqli_connect("localhost", "user", "password", "database");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_connect(): mysqli/database connections are not implemented in the current subset"
    );
}

#[test]
fn mysqli_report_accepts_current_wordpress_startup_modes() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_report";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo defined("MYSQLI_REPORT_OFF") ? MYSQLI_REPORT_OFF : "missing";
echo "|";
echo mysqli_report(MYSQLI_REPORT_OFF) ? "off" : "fail";
echo "|";
echo $call(MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT) ? "strict" : "fail";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|0|off|strict");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_report_rejects_modes_outside_current_boundary() {
    let error = run_source(
        r#"<?php
mysqli_report(4);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_report(): only MYSQLI_REPORT_OFF and MYSQLI_REPORT_ERROR|MYSQLI_REPORT_STRICT are supported in the current subset"
    );

    let error = run_source(
        r#"<?php
mysqli_report("off");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_report(): report mode must be int in the current subset, got string"
    );
}

#[test]
fn dynamic_mysqli_connect_calls_use_the_same_database_boundary() {
    let error = run_source(
        r#"<?php
$call = "mysqli_connect";
$call("localhost");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_connect(): mysqli/database connections are not implemented in the current subset"
    );
}

#[test]
fn emit_ir_folds_mysqli_connect_metadata_but_rejects_direct_connection_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("mysqli_connect") ? "1" : "0";
echo is_callable("mysqli_connect") ? "1" : "0";
echo function_exists("mysqli_report") ? "1" : "0";
echo is_callable("mysqli_report") ? "1" : "0";
echo defined("MYSQLI_REPORT_OFF") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 5, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
    assert!(!ir.contains("MYSQLI_REPORT_OFF"), "{ir}");

    let error = emit_ir_source(
        r#"<?php
mysqli_report(MYSQLI_REPORT_OFF);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_connect("localhost");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
