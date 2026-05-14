use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source};

const DIVISION_REJECTION: &str = "LLVM division lowering rejects / until native PHP division semantics, zero-divisor runtime checks, avoidance of misleading integer truncation, overflow/INF/NAN behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current division behavior";

#[test]
fn emit_ir_rejects_static_integer_zero_divisor() {
    let error = emit_ir_source("<?php\necho 10 / 0;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, DIVISION_REJECTION);
}

#[test]
fn emit_ir_rejects_static_float_zero_divisor() {
    let error = emit_ir_source("<?php\necho 10 / 0.0;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, DIVISION_REJECTION);
}

#[test]
fn emit_ir_rejects_static_null_and_false_zero_divisors() {
    let null_error = emit_ir_source("<?php\necho 10 / null;\n").unwrap_err();
    let false_error = emit_ir_source("<?php\n$zero = false;\necho 10 / $zero;\n").unwrap_err();

    assert_eq!(null_error.phase, Phase::Codegen);
    assert_eq!(false_error.phase, Phase::Codegen);
    assert_eq!(null_error.message, DIVISION_REJECTION);
    assert_eq!(false_error.message, DIVISION_REJECTION);
}

#[test]
fn emit_ir_rejects_nonzero_division_until_native_numeric_lowering_exists() {
    let error = emit_ir_source("<?php\necho 10 / 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, DIVISION_REJECTION);
}

#[test]
fn emit_ir_rejects_nonzero_literal_variable_divisor_until_native_numeric_lowering_exists() {
    let error = emit_ir_source("<?php\n$divisor = 2;\necho 10 / $divisor;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, DIVISION_REJECTION);
}

#[test]
fn emit_ir_rejects_dynamic_divisors_until_runtime_checks_exist() {
    let error = emit_ir_source("<?php\n$divisor = 4 - 2;\necho 10 / $divisor;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, DIVISION_REJECTION);
}

#[test]
fn emit_asm_rejects_static_zero_divisor_before_backend_execution() {
    let error = emit_asm_source("<?php\necho 10 / 0;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, DIVISION_REJECTION);
}

#[test]
fn native_division_zero_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone162/native_division_by_zero.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone162/native_division_by_zero_emit_ir.cli"),
    )
    .expect("native division safety CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_dynamic_division_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone163/native_dynamic_division.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone163/native_dynamic_division_emit_ir.cli"),
    )
    .expect("native dynamic division CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_division_boundary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone320/native_integer_division.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone320/native_integer_division_emit_ir.cli"),
    )
    .expect("native integer division boundary CLI snapshot is readable");
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
