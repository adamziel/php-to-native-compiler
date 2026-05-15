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
fn exit_stops_execution_and_sets_current_status_subset() {
    let execution = run_source(
        r#"<?php
echo "before";
exit(7);
echo "after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "before");
    assert_eq!(execution.exit_code, 7);

    let message = run_source(
        r#"<?php
exit("message");
echo "after";
"#,
    )
    .unwrap();

    assert_eq!(message.stdout, "message");
    assert_eq!(message.exit_code, 0);
}

#[test]
fn die_alias_uses_same_current_exit_subset() {
    let execution = run_source(
        r#"<?php
echo "before";
die(3);
echo "after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "before");
    assert_eq!(execution.exit_code, 3);
}

#[test]
fn exit_is_not_reported_as_a_callable_function() {
    let execution = run_source(
        r#"<?php
echo function_exists("exit") ? "function" : "not-function";
echo "|";
echo is_callable("exit") ? "callable" : "not-callable";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "not-function|not-callable");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn exit_rejects_forms_outside_current_subset() {
    let arity = runtime_error(
        r#"<?php
exit(1, 2);
"#,
    );
    assert_eq!(arity.line, 2);
    assert_eq!(arity.column, 1);
    assert_eq!(
        arity.message,
        "arity mismatch for exit(): expected 0 to 1 argument(s), got 2"
    );

    let unsupported = runtime_error(
        r#"<?php
exit(true);
"#,
    );
    assert_eq!(unsupported.line, 2);
    assert_eq!(unsupported.column, 1);
    assert_eq!(
        unsupported.message,
        "unsupported call exit(): argument must be null, int, or string in the current subset, got bool"
    );
}

#[test]
fn emit_ir_rejects_exit_until_native_termination_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
exit(1);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
