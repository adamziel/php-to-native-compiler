use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_CONDITIONAL_REJECTION: &str = "LLVM conditional lowering rejects ternary and null coalescing expressions until native PHP truthiness, null-aware lookup, and branch side-effect ordering exist; phpc run handles current conditional expression behavior";

#[test]
fn phpc_run_still_handles_current_conditional_expressions() {
    let execution = run_source(
        r#"<?php
$missing = null;
echo true ? "yes" : fail(), "\n";
echo false ?: "fallback", "\n";
echo $missing ?? "coalesced", "\n";
$value = "present";
echo $value ?? fail();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes\nfallback\ncoalesced\npresent");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_ternary_and_null_coalescing_with_specific_boundary() {
    for source in [
        "<?php\necho true ? 1 : 2;\n",
        "<?php\necho false ?: 2;\n",
        "<?php\n$value = null;\necho $value ?? 2;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_CONDITIONAL_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_ternary_before_lowering_branch_operands() {
    let error = emit_ir_source("<?php\necho true ? [] : [];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONDITIONAL_REJECTION);
}

#[test]
fn emit_ir_rejects_null_coalescing_before_lowering_operands() {
    let error = emit_ir_source("<?php\necho [] ?? [];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONDITIONAL_REJECTION);
}

#[test]
fn emit_asm_rejects_conditional_expressions_before_backend_execution() {
    let error = emit_asm_source("<?php\necho true ? 1 : 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONDITIONAL_REJECTION);
}

#[test]
fn native_conditional_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone168/native_conditional_boundary.php");
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
        workspace_root.join("tests/fixtures/milestone168/native_conditional_boundary_emit_ir.cli"),
    )
    .expect("native conditional CLI snapshot is readable");
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
