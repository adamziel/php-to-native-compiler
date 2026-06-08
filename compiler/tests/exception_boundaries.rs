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
fn custom_exception_thrown_from_user_function_preserves_object_for_catch() {
    let execution = run_source(
        r#"<?php
class CustomException extends Exception {
    public $label = "kept";
}
function throw_custom() {
    throw new CustomException("boom");
}
try {
    throw_custom();
} catch (CustomException $e) {
    echo get_class($e), "|", $e->getMessage(), "|", $e->label;
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "CustomException|boom|kept");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn custom_exception_thrown_from_error_handler_preserves_object_for_catch() {
    let execution = run_source(
        r#"<?php
class WarningException extends Exception {
    public function __construct(public $errno, public $messageText) {}
}
set_error_handler(function($errno, $message) {
    throw new WarningException($errno, $message);
});
try {
    trigger_error("promoted", E_USER_WARNING);
} catch (WarningException $e) {
    echo $e->errno, "|", $e->messageText;
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "512|promoted");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn throwable_subclass_user_methods_are_dispatched_before_core_methods() {
    let execution = run_source(
        r#"<?php
class MyException extends Exception {
    public function __construct(public $error) {}
    public function getException() { return $this->error; }
}
$e = new MyException("kept");
echo $e->getException();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "kept");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn custom_assertion_exception_renders_as_uncaught_throwable() {
    let execution = run_source(
        r#"<?php
class CustomAssertionException extends Exception {}
assert(false, new CustomAssertionException("asserted"));
"#,
    )
    .unwrap();

    assert!(execution.stdout.starts_with(
        "Fatal error: Uncaught CustomAssertionException: asserted in Command line code:3"
    ));
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn invalid_throw_operands_raise_php_shaped_errors() {
    let non_object = run_source(
        r#"<?php
throw 1;
"#,
    )
    .unwrap();
    assert!(non_object
        .stdout
        .starts_with("Fatal error: Uncaught Error: Can only throw objects"));
    assert_eq!(non_object.exit_code, 255);

    let non_throwable = run_source(
        r#"<?php
class Box {}
throw new Box();
"#,
    )
    .unwrap();
    assert!(non_throwable.stdout.starts_with(
        "Fatal error: Uncaught Error: Cannot throw objects that do not implement Throwable"
    ));
    assert_eq!(non_throwable.exit_code, 255);
}

#[test]
fn throwable_constructor_message_type_errors_are_php_shaped() {
    let execution = run_source(
        r#"<?php
class CustomError extends Error {}
throw new CustomError(new stdClass());
"#,
    )
    .unwrap();

    assert!(execution.stdout.starts_with(
        "Fatal error: Uncaught TypeError: Error::__construct(): Argument #1 ($message) must be of type string, stdClass given"
    ));
    assert_eq!(execution.exit_code, 255);
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
fn exception_handlers_invoke_restore_reset_and_report_current_handler() {
    let execution = run_source(
        r#"<?php
function first($e) { echo "first:", get_class($e), "\n"; }
function second($e) { echo "second:", get_class($e), "\n"; }

echo get_exception_handler() === null ? "none\n" : "other\n";
$previous = set_exception_handler("first");
echo $previous === null ? "prev-null\n" : "prev-other\n";
$previous = set_exception_handler("second");
echo $previous === "first" ? "prev-first\n" : "prev-other\n";
restore_exception_handler();
echo get_exception_handler() === "first" ? "restored\n" : "missing\n";
throw new Exception();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "none\nprev-null\nprev-first\nrestored\nfirst:Exception\n"
    );
    assert_eq!(execution.exit_code, 0);

    let reset = run_source(
        r#"<?php
function first($e) { echo "first"; }
set_exception_handler("first");
$previous = set_exception_handler(null);
echo $previous === "first" ? "cleared\n" : "bad\n";
throw new Exception();
"#,
    )
    .unwrap();

    assert_eq!(reset.exit_code, 255);
    assert!(reset
        .stdout
        .starts_with("cleared\n\nFatal error: Uncaught Exception"));
}

#[test]
fn exception_handler_object_array_callback_receives_throwable() {
    let execution = run_source(
        r#"<?php
class Handler {
    public function handle($e) {
        echo "object:", get_class($e), "\n";
    }
}
$handler = new Handler();
set_exception_handler([$handler, "handle"]);
throw new Exception();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "object:Exception\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn set_exception_handler_rejects_invalid_callbacks_with_type_errors() {
    let execution = run_source(
        r#"<?php
try {
    set_exception_handler("fo");
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
try {
    set_exception_handler(["", ""]);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "set_exception_handler(): Argument #1 ($callback) must be a valid callback or null, function \"fo\" not found or invalid function name\nset_exception_handler(): Argument #1 ($callback) must be a valid callback or null, class \"\" not found\n"
    );
    assert_eq!(execution.exit_code, 0);
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
