use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_BITWISE_REJECTION: &str = "LLVM bitwise lowering rejects bitwise and shift operators until native PHP bitwise string semantics and shift diagnostics exist; phpc run handles current bitwise/shift behavior";

#[test]
fn phpc_run_still_handles_current_bitwise_and_shift_operators() {
    let execution = run_source(
        r#"<?php
var_dump(6 & 3);
var_dump(6 | 3);
var_dump(6 ^ 3);
var_dump(~5);
var_dump(8 << 1);
var_dump(8 >> 1);
var_dump("ab" & "AB");
var_dump("8" << true);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(2)\nint(7)\nint(5)\nint(-6)\nint(16)\nint(4)\nstring(2) \"AB\"\nint(16)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_all_bitwise_and_shift_operators_with_specific_boundary() {
    for source in [
        "<?php\necho 6 & 3;\n",
        "<?php\necho 6 | 3;\n",
        "<?php\necho 6 ^ 3;\n",
        "<?php\necho ~5;\n",
        "<?php\necho 8 << 1;\n",
        "<?php\necho 8 >> 1;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_BITWISE_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_bitwise_operators_before_lowering_operands() {
    let error = emit_ir_source("<?php\necho [] & [];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_BITWISE_REJECTION);
}

#[test]
fn emit_ir_rejects_unary_bitwise_not_before_lowering_operand() {
    let error = emit_ir_source("<?php\necho ~[];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_BITWISE_REJECTION);
}

#[test]
fn emit_asm_rejects_bitwise_operators_before_backend_execution() {
    let error = emit_asm_source("<?php\necho 6 & 3;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_BITWISE_REJECTION);
}

#[test]
fn native_bitwise_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone167/native_bitwise_boundary.php");
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
        workspace_root.join("tests/fixtures/milestone167/native_bitwise_boundary_emit_ir.cli"),
    )
    .expect("native bitwise CLI snapshot is readable");
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
