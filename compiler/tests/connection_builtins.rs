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
fn ignore_user_abort_tracks_current_placeholder_state() {
    let execution = run_source(
        r#"<?php
echo ignore_user_abort();
echo "|";
echo ignore_user_abort(true);
echo "|";
echo ignore_user_abort();
echo "|";
echo ignore_user_abort(false);
echo "|";
echo ignore_user_abort(null);
echo "|";
echo ignore_user_abort("1");
echo "|";
echo ignore_user_abort("0");
echo "|";
echo ignore_user_abort();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|0|1|1|0|0|1|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ignore_user_abort_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "ignore_user_abort";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call(true);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn connection_state_reports_normal_cli_connection() {
    let execution = run_source(
        r#"<?php
echo CONNECTION_NORMAL, "|", CONNECTION_ABORTED, "|", CONNECTION_TIMEOUT, "\n";
var_dump(connection_status() === CONNECTION_NORMAL);
var_dump(connection_aborted());
foreach (["connection_status", "connection_aborted"] as $call) {
    echo function_exists($call) ? "1" : "0";
    echo is_callable($call) ? "1" : "0";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|1|2\nbool(true)\nint(0)\n1111");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ignore_user_abort_rejects_forms_outside_current_subset() {
    let unsupported = runtime_error(
        r#"<?php
ignore_user_abort([]);
"#,
    );
    assert_eq!(unsupported.line, 2);
    assert_eq!(unsupported.column, 1);
    assert_eq!(
        unsupported.message,
        "unsupported call ignore_user_abort(): setting argument must be null or scalar in the current subset, got array"
    );

    let too_many = runtime_error(
        r#"<?php
ignore_user_abort(true, false);
"#,
    );
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for ignore_user_abort(): expected 0 to 1 argument(s), got 2"
    );
}

#[test]
fn emit_ir_folds_ignore_user_abort_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("ignore_user_abort") ? "1" : "0";
echo is_callable("ignore_user_abort") ? "1" : "0";
echo function_exists("connection_status") ? "1" : "0";
echo is_callable("connection_aborted") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 4, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nignore_user_abort(true);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\nconnection_status();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
