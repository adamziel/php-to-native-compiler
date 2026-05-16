use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_GLOBAL_DECLARATION_REJECTION: &str = "LLVM global-declaration lowering rejects global declarations until native root symbol-table imports, local/global aliasing, $GLOBALS interactions, references/copy-on-write, included-file scope interactions, and exact native diagnostics exist; phpc run handles current bounded global declaration behavior";

#[test]
fn phpc_run_still_handles_current_global_declaration_behavior() {
    let execution = run_source(
        r#"<?php
$value = "root";
function read_global() {
    global $value;
    return $value;
}
echo read_global(), "\n";
global $missing;
echo $missing === null ? "missing-null" : "wrong";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "root\nmissing-null");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_global_declarations_with_specific_boundary() {
    for source in [
        "<?php\nglobal $value;\n",
        "<?php\nglobal $one, $two;\n",
        "<?php\n$value = \"root\";\nglobal $value;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_GLOBAL_DECLARATION_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_global_declarations_before_backend_execution() {
    let error = emit_asm_source("<?php\nglobal $value;\necho \"after\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_GLOBAL_DECLARATION_REJECTION);
}

#[test]
fn native_global_declaration_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1118/native_global_declaration_boundary.phpc-source");
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
            .join("tests/fixtures/milestone1118/native_global_declaration_boundary_emit_ir.cli"),
    )
    .expect("native global declaration CLI snapshot is readable");
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
