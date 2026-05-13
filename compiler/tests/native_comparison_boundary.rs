use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_COMPARISON_REJECTION: &str = "LLVM comparison lowering rejects comparison operators until native PHP comparison coercions exist; phpc run handles current scalar comparison diagnostics";

#[test]
fn phpc_run_still_handles_current_scalar_comparisons() {
    let execution = run_source(
        r#"<?php
echo 1 == "1", "\n";
echo 1 != 2, "\n";
echo 2 < 3, "\n";
echo 3 <= 3, "\n";
echo 4 > 3, "\n";
echo 4 >= 4, "\n";
echo 1 === 1, "\n";
echo 1 !== "1", "\n";
echo null == false, "\n";
echo "10" > 2;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1\n1\n1\n1\n1\n1\n1\n1\n1\n1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_all_comparison_operators_with_specific_boundary() {
    for source in [
        "<?php\necho 1 == 1;\n",
        "<?php\necho 1 != 2;\n",
        "<?php\necho 1 === 1;\n",
        "<?php\necho 1 !== \"1\";\n",
        "<?php\necho 1 < 2;\n",
        "<?php\necho 1 <= 1;\n",
        "<?php\necho 2 > 1;\n",
        "<?php\necho 2 >= 2;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_COMPARISON_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_comparison_before_lowering_operands() {
    let error = emit_ir_source("<?php\necho [] == [];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_COMPARISON_REJECTION);
}

#[test]
fn emit_asm_rejects_comparisons_before_backend_execution() {
    let error = emit_asm_source("<?php\necho 1 == 1;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_COMPARISON_REJECTION);
}

#[test]
fn native_comparison_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone165/native_comparison_boundary.php");
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
        workspace_root.join("tests/fixtures/milestone165/native_comparison_boundary_emit_ir.cli"),
    )
    .expect("native comparison CLI snapshot is readable");
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
