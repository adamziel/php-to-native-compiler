use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source};

const LLVM_REQUIRE_REJECTION: &str = "LLVM include/require lowering rejects multi-file execution until native source loading, path resolution, declaration registration, stack/source mapping, and exact native error behavior exist; phpc run handles the current narrow include/require behavior";
const LLVM_REQUIRE_EXPRESSION_REJECTION: &str = "LLVM include/require lowering rejects multi-file execution for expression forms with include return values, _once de-duplication results, and caller-scope side effects until native source loading, path resolution, declaration registration, stack/source mapping, and exact native error behavior exist; phpc run handles current include/require expression behavior";

#[test]
fn emit_ir_rejects_statement_include_require_with_multifile_boundary() {
    for source in [
        "<?php\nrequire 'bootstrap.php';\n",
        "<?php\nrequire_once 'bootstrap.php';\n",
        "<?php\ninclude 'bootstrap.php';\n",
        "<?php\ninclude_once 'bootstrap.php';\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_REQUIRE_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_expression_include_require_with_return_value_boundary() {
    for source in [
        "<?php\n$ok = require 'bootstrap.php';\n",
        "<?php\n$ok = require_once 'bootstrap.php';\n",
        "<?php\n$ok = include 'bootstrap.php';\n",
        "<?php\n$ok = include_once 'bootstrap.php';\n",
        "<?php\necho include 'bootstrap.php';\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_REQUIRE_EXPRESSION_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_expression_include_require_before_backend_execution() {
    let error = emit_asm_source("<?php\n$ok = include 'bootstrap.php';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_REQUIRE_EXPRESSION_REJECTION);
}

#[test]
fn native_include_require_expression_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1108/native_include_expression_boundary.phpc-source");
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
            .join("tests/fixtures/milestone1108/native_include_expression_boundary_emit_ir.cli"),
    )
    .expect("native include expression CLI snapshot is readable");
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
