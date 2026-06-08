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
        "<?php\necho (string) [];\n",
        "<?php\necho (int) \"42tail\";\n",
        "<?php\necho (float) \"42tail\";\n",
        "<?php\necho (array) null;\n",
        "<?php\necho (object) 1;\n",
        "<?php\n(void) 1;\n",
        "<?php\necho strval(\"value\");\n",
        "<?php\necho boolval(\"value\");\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_CAST_REJECTION);
    }
}

#[test]
fn emit_ir_lowers_static_scalar_casts_for_current_subset() {
    let ir = emit_ir_source(
        r#"<?php
$value = "15";
$truthy = (1 + 2) === 3;
echo (string) true, "|", (string) 42, "|", (string) "ok", "\n";
echo (string) 2.5, "\n";
echo (int) null, "|", (int) false, "|", (int) true, "|", (int) $value, "|", (int) "2.9", "|", (int) "word", "\n";
echo (bool) null ? "T" : "F";
echo (bool) "0" ? "T" : "F";
echo (bool) "value" ? "T" : "F";
echo (int) $truthy, "\n";
echo (float) null, "|", (float) true, "|", (float) 42, "|", (float) "2.5", "|", (float) "word";
"#,
    )
    .expect("static scalar/null casts should lower in the LLVM output subset");

    assert!(ir.contains("c\"1\\00\""), "{ir}");
    assert!(ir.contains("c\"42\\00\""), "{ir}");
    assert!(ir.contains("c\"ok\\00\""), "{ir}");
    assert!(ir.contains("c\"2.5\\00\""), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 15)"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 2)"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 0)"), "{ir}");
    assert!(ir.contains("sitofp i64 42 to double"), "{ir}");
    assert!(ir.contains("@phpc_native_float(double 2.5)"), "{ir}");
    assert!(ir.contains("@phpc_native_float(double 0.0)"), "{ir}");
    assert!(
        !ir.contains("LLVM cast lowering rejects"),
        "supported scalar casts should not hit the cast blocker:\n{ir}"
    );
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
fn emit_asm_lowers_static_scalar_casts_after_ir_preflight() {
    let asm = emit_asm_source("<?php\n$value = \"15\";\necho (int) $value;\n").unwrap();

    assert!(asm.contains("main"), "{asm}");
}

#[test]
fn native_cast_emit_ir_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-ir",
        "tests/fixtures/milestone1138/native_cast_boundary_emit_ir.cli",
    );
}

#[test]
fn native_scalar_cast_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone2304/native_scalar_cast_emit_ir.php");
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
        workspace_root.join("tests/fixtures/milestone2304/native_scalar_cast_emit_ir.cli"),
    )
    .expect("native scalar cast IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
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
