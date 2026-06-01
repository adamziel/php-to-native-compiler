use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source, run_source_with_source_file};

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
fn reached_throw_statement_reports_uncaught_runtime_boundary() {
    let execution = run_source(
        r#"<?php
echo "before";
throw new Exception();
"#,
    )
    .unwrap();

    assert_eq!(execution.exit_code, 255);
    assert_eq!(
        execution.stdout,
        "before\nFatal error: Uncaught Exception in Command line code:3\nStack trace:\n#0 {main}\n  thrown in Command line code on line 3"
    );
}

#[test]
fn reached_throw_statement_evaluates_operand_before_throwing() {
    let error = run_source(
        r#"<?php
throw MISSING_EXCEPTION_VALUE;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 7);
    assert_eq!(error.message, "undefined constant MISSING_EXCEPTION_VALUE");
}

#[test]
fn throw_inside_try_binds_matching_catch_and_runs_finally() {
    let execution = run_source(
        r#"<?php
try {
    echo "try|";
    throw new Exception();
} catch (Exception $e) {
    echo "catch:", get_class($e), "|";
} finally {
    echo "finally|";
}
echo "after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "try|catch:Exception|finally|after");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn throwable_catch_type_matches_builtin_exception_objects() {
    let execution = run_source(
        r#"<?php
try {
    throw new Exception();
} catch (\Throwable $e) {
    echo "throwable:", get_class($e);
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "throwable:Exception");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unmatched_inner_catch_runs_finally_before_outer_catch() {
    let execution = run_source(
        r#"<?php
class CustomException extends Exception {}
try {
    try {
        echo "inner-try|";
        throw new CustomException();
    } catch (stdClass $e) {
        echo "wrong|";
    } finally {
        echo "inner-finally|";
    }
} catch (Exception $e) {
    echo "outer:", get_class($e), "|";
}
echo "after";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "inner-try|inner-finally|outer:CustomException|after"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn include_statement_throw_can_be_caught_by_caller_try() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is after Unix epoch")
        .as_nanos();
    let fixture_dir = std::env::temp_dir().join(format!(
        "phpc-exception-include-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&fixture_dir).expect("exception include fixture directory is created");
    fs::write(
        fixture_dir.join("thrower.php"),
        "<?php\nthrow new Exception();\n",
    )
    .expect("exception include fixture is written");
    let main = fixture_dir.join("main.php");

    let execution = run_source_with_source_file(
        r#"<?php
try {
    include 'thrower.php';
} catch (Exception $e) {
    echo "included:", get_class($e);
}
"#,
        main.display().to_string(),
    )
    .unwrap();

    assert_eq!(execution.stdout, "included:Exception");
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(fixture_dir.join("thrower.php"));
    let _ = fs::remove_dir(fixture_dir);
}

#[test]
fn emit_ir_rejects_throw_statements_until_native_exceptions_exist() {
    let error = emit_ir_source("<?php\n$exception = null;\nthrow $exception;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_EXCEPTION_REJECTION);
}

#[test]
fn emit_asm_rejects_throw_statements_before_backend_execution() {
    let error = emit_asm_source("<?php\n$exception = null;\nthrow $exception;\n").unwrap_err();

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
