use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_UNARY_REJECTION: &str = "LLVM unary lowering rejects unary minus and logical not until native PHP numeric coercion, truthiness conversion, references/copy-on-write, and exact native error behavior exist; phpc run handles current unary behavior";

#[test]
fn phpc_run_still_handles_current_unary_subset() {
    let execution = run_source(
        r#"<?php
echo -5, "\n";
echo -2.5, "\n";
echo -true, "\n";
echo !false, "\n";
echo !true, "empty\n";
echo !"0", "\n";
echo !"php", "empty";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "-5\n-2.5\n-1\n1\nempty\n1\nempty");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_unary_minus_and_logical_not_with_specific_boundary() {
    for source in ["<?php\necho -5;\n", "<?php\necho !false;\n"] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_UNARY_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_unary_forms_before_lowering_operands() {
    for source in ["<?php\necho -[];\n", "<?php\necho ![];\n"] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_UNARY_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_unary_forms_before_backend_execution() {
    let error = emit_asm_source("<?php\necho -5;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_UNARY_REJECTION);
}

#[test]
fn native_unary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone177/native_unary_boundary.php");
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
        workspace_root.join("tests/fixtures/milestone177/native_unary_boundary_emit_ir.cli"),
    )
    .expect("native unary CLI snapshot is readable");
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
