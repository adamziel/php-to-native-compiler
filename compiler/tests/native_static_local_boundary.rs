use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_STATIC_LOCAL_REJECTION: &str = "LLVM static-local lowering rejects static local declarations until native persistent per-function storage, initialization ordering, local scope interaction, references/copy-on-write, recursion, and exact native diagnostics exist; phpc run handles current bounded static local behavior";

#[test]
fn phpc_run_still_handles_current_static_local_behavior() {
    let execution = run_source(
        r#"<?php
function counter() {
    static $count = 0;
    $count = $count + 1;
    return $count;
}
echo counter(), ",", counter(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1,2\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_static_local_declarations_with_specific_boundary() {
    for source in [
        "<?php\nfunction counter() {\nstatic $count;\n}\n",
        "<?php\nfunction counter() {\nstatic $count = 0;\n}\n",
        "<?php\nfunction counter() {\nstatic $first = 1, $second;\n}\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.line, 3);
        assert_eq!(error.column, 1);
        assert_eq!(error.message, LLVM_STATIC_LOCAL_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_static_local_declarations_before_backend_execution() {
    let error =
        emit_asm_source("<?php\nfunction counter() {\nstatic $count = 0;\n}\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_STATIC_LOCAL_REJECTION);
}

#[test]
fn native_static_local_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1123/native_static_local_boundary.phpc-source");
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
            .join("tests/fixtures/milestone1123/native_static_local_boundary_emit_ir.cli"),
    )
    .expect("native static local CLI snapshot is readable");
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
