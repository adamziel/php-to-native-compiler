use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_LOGICAL_REJECTION: &str = "LLVM logical lowering rejects logical operators until native PHP truthiness and short-circuit semantics exist; phpc run handles current logical operator behavior";

#[test]
fn phpc_run_still_handles_current_logical_operators() {
    let execution = run_source(
        r#"<?php
echo true && false, "\n";
echo false || true, "\n";
echo (true and false) ? "1" : "0", "\n";
echo (false or true) ? "1" : "0", "\n";
echo (true xor false) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "\n1\n0\n1\n1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_all_logical_operators_with_specific_boundary() {
    for source in [
        "<?php\necho true && false;\n",
        "<?php\necho false || true;\n",
        "<?php\necho true and false;\n",
        "<?php\necho true xor false;\n",
        "<?php\necho false or true;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_LOGICAL_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_logical_operators_before_lowering_operands() {
    let error = emit_ir_source("<?php\necho [] && [];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_LOGICAL_REJECTION);
}

#[test]
fn emit_asm_rejects_logical_operators_before_backend_execution() {
    let error = emit_asm_source("<?php\necho true && false;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_LOGICAL_REJECTION);
}

#[test]
fn native_logical_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone166/native_logical_boundary.php");
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
        workspace_root.join("tests/fixtures/milestone166/native_logical_boundary_emit_ir.cli"),
    )
    .expect("native logical CLI snapshot is readable");
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
