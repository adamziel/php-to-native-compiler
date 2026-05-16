use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_EXCEPTION_REJECTION: &str = "LLVM exception lowering rejects throw statements and try/catch/finally blocks until native Throwable objects, stack unwinding, catch/finally dispatch, stack traces, and exact native error behavior exist; phpc run handles the current exception boundary";
const LLVM_TRY_BLOCK_REJECTION: &str = "LLVM try/catch/finally lowering rejects try blocks until native Throwable objects, stack unwinding, catch type matching, catch variable binding, finally execution during normal and exceptional control flow, stack traces, references/copy-on-write, and exact native try-block diagnostics exist; phpc run handles current bounded no-throw try/catch/finally behavior";

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
fn reached_try_statement_executes_body_and_skips_catch_without_throw() {
    let execution = run_source(
        r#"<?php
try {
    echo "try";
} catch (Exception $e) {
    echo "catch";
}
echo "after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "tryafter");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn try_finally_without_catch_executes_body_and_finally_without_throw() {
    let execution = run_source(
        r#"<?php
try {
    echo "try";
} finally {
    echo "finally";
}
echo "after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "tryfinallyafter");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn try_catch_finally_skips_catch_and_runs_finally_without_throw() {
    let execution = run_source(
        r#"<?php
try {
    echo "try";
} catch (Throwable|Exception $e) {
    echo "catch";
} finally {
    echo "finally";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "tryfinally");
    assert_eq!(execution.exit_code, 0);
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
fn throw_inside_try_still_reports_throw_boundary_before_catch_matching() {
    let error = run_source(
        r#"<?php
try {
    throw MISSING_EXCEPTION_VALUE;
} catch (Exception $e) {
    echo "catch";
} finally {
    echo "finally";
}
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 5);
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
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_TRY_BLOCK_REJECTION);
}

#[test]
fn emit_asm_rejects_try_blocks_before_backend_execution() {
    let error = emit_asm_source(
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
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_TRY_BLOCK_REJECTION);
}

#[test]
fn native_try_block_emit_ir_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-ir",
        "tests/fixtures/milestone1173/native_try_block_boundary_emit_ir.cli",
    );
}

#[test]
fn native_try_block_emit_asm_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-asm",
        "tests/fixtures/milestone1173/native_try_block_boundary_emit_asm.cli",
    );
}

fn assert_cli_snapshot_matches(mode: &str, snapshot_path: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone1173/native_try_block_boundary.phpc-source");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, mode])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(snapshot_path))
        .expect("native try-block CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}
