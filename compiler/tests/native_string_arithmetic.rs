use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const ARITHMETIC_REJECTION: &str = "LLVM arithmetic lowering rejects unsupported binary arithmetic operators or operands until native PHP numeric coercion, division/modulo zero checks, modulo coercions, references/copy-on-write, and exact native error behavior exist; phpc run handles current arithmetic behavior";
const SCALAR_COERCION_ARITHMETIC_REJECTION: &str = "LLVM scalar-coercion arithmetic lowering rejects booleans, nulls, and strings in +, -, and * until native PHP numeric coercion, string numeric parsing, warnings/recovery behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current scalar-coercion arithmetic behavior";
const DIVISION_REJECTION: &str = "LLVM division lowering rejects / until native PHP division semantics, zero-divisor runtime checks, avoidance of misleading integer truncation, overflow/INF/NAN behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current division behavior";

#[test]
fn phpc_run_still_handles_numeric_string_arithmetic() {
    let execution = run_source(
        r#"<?php
echo "2" + 3, "\n";
echo 8 - "2.5", "\n";
echo "3e1" * 2, "\n";
echo 9 / "3", "\n";
echo "8" % 3;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "5\n5.5\n60\n3\n2");
}

#[test]
fn emit_ir_lowers_numeric_string_operands_in_static_primitive_arithmetic() {
    let ir = emit_ir_source(
        r#"<?php
echo "2" + 3, "\n";
echo 8 - "2.5", "\n";
echo "3e1" * 2;
"#,
    )
    .unwrap();

    assert!(ir.contains("@printf(ptr @.fmt_int, i64 5)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_float, double 5.5)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_float, double 60.0)"), "{ir}");

    let error = emit_ir_source("<?php\necho \"8\" % 3;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, ARITHMETIC_REJECTION);

    let error = emit_ir_source("<?php\necho 9 / \"3\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, DIVISION_REJECTION);
}

#[test]
fn emit_asm_rejects_non_numeric_string_arithmetic_before_backend_execution() {
    let error = emit_asm_source("<?php\necho \"two\" + 3;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, SCALAR_COERCION_ARITHMETIC_REJECTION);
}

#[test]
fn native_string_arithmetic_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone164/native_string_arithmetic.php");
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
        workspace_root.join("tests/fixtures/milestone164/native_string_arithmetic_emit_ir.cli"),
    )
    .expect("native string arithmetic CLI snapshot is readable");
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
