use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_CAST_REJECTION: &str = "LLVM cast lowering rejects (string), (int)/(integer), (bool)/(boolean), (float)/(double), (array), (object), and (void) casts plus strval(), boolval(), floatval(), and doubleval() until native PHP scalar conversion, array/object materialization, warning/recovery behavior, object/resource handling, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded cast behavior";

#[test]
fn phpc_run_still_handles_current_bounded_cast_behavior() {
    let execution = run_source(
        r#"<?php
echo (string) 42, "|";
echo (int) "15", "|";
echo (bool) "0" ? "true" : "false", "|";
echo (float) "2.5", "|";
echo count((array) null);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "42|15|false|2.5|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_casts_with_specific_boundary() {
    for source in [
        "<?php\necho (string) 42;\n",
        "<?php\necho (int) \"15\";\n",
        "<?php\necho (integer) \"15\";\n",
        "<?php\necho (bool) \"0\";\n",
        "<?php\necho (boolean) \"0\";\n",
        "<?php\necho (float) \"2.5\";\n",
        "<?php\necho (double) \"2.5\";\n",
        "<?php\necho (array) null;\n",
        "<?php\necho (object) 1;\n",
        "<?php\n(void) 1;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_CAST_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_casts_before_lowering_operands() {
    let error = emit_ir_source("<?php\necho (int) [];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_CAST_REJECTION);
}

#[test]
fn emit_asm_rejects_casts_before_backend_execution() {
    let error = emit_asm_source("<?php\n$value = \"15\";\necho (int) $value;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_CAST_REJECTION);
}

#[test]
fn native_cast_emit_ir_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-ir",
        "tests/fixtures/milestone1138/native_cast_boundary_emit_ir.cli",
    );
}

#[test]
fn native_cast_emit_asm_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-asm",
        "tests/fixtures/milestone1138/native_cast_boundary_emit_asm.cli",
    );
}

fn assert_cli_snapshot_matches(mode: &str, snapshot_path: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone1138/native_cast_boundary.phpc-source");
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
        .expect("native cast CLI snapshot is readable");
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
