use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_ERROR_CONTROL_REJECTION: &str = "LLVM error-control lowering rejects @expr until native diagnostic severity, warning/notice/deprecation suppression, error_reporting() mask interaction, recoverable expression values, and exact native diagnostics exist; phpc run handles current bounded error-control diagnostic suppression";

#[test]
fn phpc_run_still_handles_current_error_control_wrapper_behavior() {
    let execution = run_source(
        r#"<?php
$value = 6;
echo @($value + 1), "\n";
echo @strlen("abc");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "7\n3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_error_control_with_specific_boundary() {
    for source in [
        "<?php\necho @5;\n",
        "<?php\n$value = 6;\necho @($value + 1);\n",
        "<?php\necho @strlen(\"abc\");\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_ERROR_CONTROL_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_error_control_before_lowering_operand() {
    let error = emit_ir_source("<?php\necho @[];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_ERROR_CONTROL_REJECTION);
}

#[test]
fn emit_asm_rejects_error_control_before_backend_execution() {
    let error = emit_asm_source("<?php\n$value = 6;\necho @($value + 1);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_ERROR_CONTROL_REJECTION);
}

#[test]
fn native_error_control_emit_ir_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-ir",
        "tests/fixtures/milestone1133/native_error_control_boundary_emit_ir.cli",
    );
}

#[test]
fn native_error_control_emit_asm_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-asm",
        "tests/fixtures/milestone1133/native_error_control_boundary_emit_asm.cli",
    );
}

fn assert_cli_snapshot_matches(mode: &str, snapshot_path: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1133/native_error_control_boundary.phpc-source");
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
        .expect("native error-control CLI snapshot is readable");
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
