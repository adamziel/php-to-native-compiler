use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::emit_asm_source;
use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_GETCWD_REJECTION: &str = "LLVM getcwd lowering rejects direct current-directory calls until native process/request cwd state, UTF-8/path policy, SAPI cwd behavior, chdir() interaction, failure false recovery, references/copy-on-write, and exact native getcwd diagnostics exist; phpc run handles current bounded getcwd behavior";

#[test]
fn getcwd_returns_current_utf8_working_directory() {
    let expected = std::env::current_dir()
        .expect("test process has a current working directory")
        .into_os_string()
        .into_string()
        .expect("test current working directory is valid UTF-8");

    let execution = run_source("<?php\necho getcwd();\n").unwrap();

    assert_eq!(execution.stdout, expected);
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn getcwd_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "getcwd";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo is_dir($call()) ? "dir" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|dir");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn getcwd_returns_false_after_current_directory_is_removed() {
    let saved_cwd = std::env::current_dir().expect("test process has a current directory");
    let execution = run_source(
        r#"<?php
$old = getcwd();
$dir = sys_get_temp_dir() . "/phpc-invalid-cwd-" . uniqid();
mkdir($dir);
chdir($dir);
rmdir($dir);
var_dump(getcwd());
var_dump(realpath(""));
var_dump(realpath("."));
var_dump(realpath("./"));
chdir($old);
"#,
    );
    std::env::set_current_dir(saved_cwd).expect("restore test process current directory");
    let execution = execution.unwrap();

    assert_eq!(
        execution.stdout,
        "bool(false)\nbool(false)\nstring(1) \".\"\nstring(1) \".\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn getcwd_rejects_arguments_in_current_subset() {
    let error = run_source("<?php\necho getcwd('/tmp');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "arity mismatch for getcwd(): expected 0 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_getcwd_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("getcwd") ? "1" : "0";
echo is_callable("getcwd") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho getcwd();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_GETCWD_REJECTION);
}

#[test]
fn emit_ir_rejects_getcwd_before_lowering_arguments() {
    let error = emit_ir_source("<?php\necho getcwd('/tmp');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_GETCWD_REJECTION);
}

#[test]
fn emit_asm_rejects_getcwd_before_backend_execution() {
    let error = emit_asm_source("<?php\necho getcwd();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_GETCWD_REJECTION);
}

#[test]
fn native_getcwd_emit_ir_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-ir",
        "tests/fixtures/milestone1208/native_getcwd_boundary_emit_ir.cli",
    );
}

#[test]
fn native_getcwd_emit_asm_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-asm",
        "tests/fixtures/milestone1208/native_getcwd_boundary_emit_asm.cli",
    );
}

fn assert_cli_snapshot_matches(mode: &str, snapshot_path: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone1208/native_getcwd_boundary.phpc-source");
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
        .expect("native getcwd CLI snapshot is readable");
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
