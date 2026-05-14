use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_EXCEPTION_REJECTION: &str = "LLVM exception lowering rejects throw statements and try/catch/finally blocks until native Throwable objects, stack unwinding, catch/finally dispatch, stack traces, and exact native error behavior exist; phpc run handles the current exception boundary";

#[test]
fn throw_statements_parse_and_execute_only_when_reached() {
    let execution = run_source(
        r#"<?php
if (false) {
    throw new Exception("not reached");
}
echo "after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "after");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn try_catch_finally_parse_and_execute_only_when_reached() {
    let execution = run_source(
        r#"<?php
function guarded() {
    try {
        echo "try";
    } catch (\Throwable|Exception $e) {
        echo "catch";
    } finally {
        echo "finally";
    }
}
echo "after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "after");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reached_try_statement_reports_current_runtime_boundary() {
    let error = run_source(
        r#"<?php
try {
    echo "try";
} catch (Exception $e) {
    echo "catch";
}
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call try: exception handling and stack unwinding are not implemented"
    );
}

#[test]
fn try_finally_without_catch_reports_current_runtime_boundary() {
    let error = run_source(
        r#"<?php
try {
    echo "try";
} finally {
    echo "finally";
}
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call try: exception handling and stack unwinding are not implemented"
    );
}

#[test]
fn reached_throw_statement_reports_current_runtime_boundary() {
    let error = run_source(
        r#"<?php
echo "before";
throw new Exception("boom");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call throw: exception objects and stack unwinding are not implemented"
    );
}

#[test]
fn reached_throw_statement_does_not_evaluate_operand_until_exceptions_exist() {
    let error = run_source(
        r#"<?php
throw MISSING_EXCEPTION_VALUE;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call throw: exception objects and stack unwinding are not implemented"
    );
}

#[test]
fn emit_ir_rejects_throw_statements_until_native_exceptions_exist() {
    let error = emit_ir_source("<?php\nthrow new Exception('boom');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_EXCEPTION_REJECTION);
}

#[test]
fn emit_asm_rejects_throw_statements_before_backend_execution() {
    let error = emit_asm_source("<?php\nthrow new Exception('boom');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_EXCEPTION_REJECTION);
}

#[test]
fn emit_ir_rejects_try_blocks_until_native_exceptions_exist() {
    let error = emit_ir_source(
        r#"<?php
try {
    echo "try";
} catch (Exception $e) {
    echo "catch";
}
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_EXCEPTION_REJECTION);
}
