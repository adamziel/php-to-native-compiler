use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source_with_source_file};

const LLVM_MAGIC_CONSTANT_REJECTION: &str = "LLVM magic-constant lowering rejects executable magic constants __LINE__, __FILE__, __DIR__, and __FUNCTION__ until native source mapping, path canonicalization, and function-context lowering exist; phpc run handles current magic constant behavior";

#[test]
fn phpc_run_still_handles_current_magic_constant_subset() {
    let execution = run_source_with_source_file(
        r#"<?php
echo "line:", __LINE__, "\n";
echo "file:", __FILE__, "\n";
echo "dir:", __DIR__, "\n";
echo "top:", __FUNCTION__, "\n";
function current_magic($default = __FUNCTION__) {
    echo "default:", $default, "\n";
    echo "body:", __FUNCTION__;
}
current_magic();
"#,
        "virtual/native_magic.php",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "line:2\nfile:virtual/native_magic.php\ndir:virtual\ntop:\ndefault:current_magic\nbody:current_magic"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_executable_magic_constants_with_specific_boundary() {
    for source in [
        "<?php\necho __LINE__;\n",
        "<?php\necho __FILE__;\n",
        "<?php\necho __DIR__;\n",
        "<?php\necho __FUNCTION__;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_MAGIC_CONSTANT_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_magic_constants_before_backend_execution() {
    let error = emit_asm_source("<?php\necho __LINE__;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_MAGIC_CONSTANT_REJECTION);
}

#[test]
fn native_magic_constant_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone171/native_magic_constant_boundary.php");
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
        workspace_root
            .join("tests/fixtures/milestone171/native_magic_constant_boundary_emit_ir.cli"),
    )
    .expect("native magic-constant CLI snapshot is readable");
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
