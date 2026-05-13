use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_ARITHMETIC_REJECTION: &str = "LLVM arithmetic lowering rejects binary arithmetic operators until native PHP numeric coercion, division/modulo zero checks, modulo coercions, references/copy-on-write, and exact native error behavior exist; phpc run handles current arithmetic behavior";

#[test]
fn phpc_run_still_handles_current_binary_arithmetic_subset() {
    let execution = run_source(
        r#"<?php
echo 1 + 2, "\n";
echo 8 - 2.5, "\n";
echo true * 6, "\n";
echo 9 / "3", "\n";
echo "8" % 3;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "3\n5.5\n6\n3\n2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_binary_arithmetic_with_specific_boundary() {
    for source in [
        "<?php\necho 1 + 2;\n",
        "<?php\necho 8 - 2;\n",
        "<?php\necho 3 * 4;\n",
        "<?php\necho 9 / 3;\n",
        "<?php\necho 8 % 3;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_ARITHMETIC_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_arithmetic_before_lowering_operands() {
    let error = emit_ir_source("<?php\necho [] + [];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARITHMETIC_REJECTION);
}

#[test]
fn emit_asm_rejects_arithmetic_before_backend_execution() {
    let error = emit_asm_source("<?php\necho 1 + 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARITHMETIC_REJECTION);
}

#[test]
fn native_arithmetic_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone178/native_arithmetic_boundary.php");
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
        workspace_root.join("tests/fixtures/milestone178/native_arithmetic_boundary_emit_ir.cli"),
    )
    .expect("native arithmetic CLI snapshot is readable");
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
