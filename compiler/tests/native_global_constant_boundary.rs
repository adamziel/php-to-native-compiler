use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_GLOBAL_CONSTANT_REJECTION: &str = "LLVM global-constant lowering rejects built-in constants, runtime-defined constants, bare constant reads, top-level const declarations, and define()/constant()/defined() until native constant tables, source-order definitions, namespace-aware lookup, and exact native error behavior exist; phpc run handles current global constant behavior";

#[test]
fn phpc_run_still_handles_current_global_constant_subset() {
    let execution = run_source(
        r#"<?php
define("RUNTIME_BASE", 3);
const FROM_DEFINE = RUNTIME_BASE + 1;
const NAME = "compiler", MODE = ARRAY_FILTER_USE_KEY;
echo ARRAY_FILTER_USE_BOTH, "|", FROM_DEFINE, "|", NAME, "|", MODE, "\n";
echo constant("RUNTIME_BASE"), "|", defined("RUNTIME_BASE"), "|", defined("MISSING_CONST");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|4|compiler|2\n3|1|");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_bare_constant_reads_with_specific_boundary() {
    for source in [
        "<?php\necho ARRAY_FILTER_USE_KEY;\n",
        "<?php\ndefine(\"APP_NAME\", \"compiler\");\necho APP_NAME;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_GLOBAL_CONSTANT_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_top_level_const_declarations_before_lowering_values() {
    let error = emit_ir_source("<?php\nconst ITEMS = [1];\necho \"after\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_GLOBAL_CONSTANT_REJECTION);
}

#[test]
fn emit_ir_rejects_constant_table_builtins_before_lowering_arguments() {
    for source in [
        "<?php\ndefine(\"APP_NAME\", []);\n",
        "<?php\necho constant([]);\n",
        "<?php\necho defined([]);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_GLOBAL_CONSTANT_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_global_constants_before_backend_execution() {
    let error = emit_asm_source("<?php\necho ARRAY_FILTER_USE_KEY;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_GLOBAL_CONSTANT_REJECTION);
}

#[test]
fn native_global_constant_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone172/native_global_constant_boundary.php");
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
            .join("tests/fixtures/milestone172/native_global_constant_boundary_emit_ir.cli"),
    )
    .expect("native global-constant CLI snapshot is readable");
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
