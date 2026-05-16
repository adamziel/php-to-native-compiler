use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_BASENAME_REJECTION: &str = "LLVM basename lowering rejects direct path basename calls until native PHP path string conversion, suffix handling, trailing-separator normalization, Windows/UNC and stream-wrapper path semantics, locale/codepage behavior, argument diagnostics, references/copy-on-write, and exact native basename diagnostics exist; phpc run handles current bounded basename behavior";
const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn basename_executes_current_unix_path_subset() {
    let execution = run_source(
        r#"<?php
echo basename("/tmp/wordpress/wp-includes/plugin.php"), "\n";
echo basename("/tmp/wordpress/wp-includes/"), "\n";
echo "[", basename("autoload.php"), "]\n";
echo "[", basename(""), "]\n";
echo "[", basename("/"), "]\n";
echo basename("/a/b/c.php", ".php"), "\n";
$call = "basename";
echo $call("/a/b//c.php");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "plugin.php\nwp-includes\n[autoload.php]\n[]\n[]\nc\nc.php"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn basename_reports_current_argument_boundaries() {
    let non_string_path = run_source("<?php\necho basename(42);\n").unwrap_err();
    assert_eq!(non_string_path.phase, Phase::Runtime);
    assert_eq!(non_string_path.line, 2);
    assert_eq!(non_string_path.column, 6);
    assert_eq!(
        non_string_path.message,
        "unsupported call basename(): path argument must be string in the current subset, got int"
    );

    let non_string_suffix = run_source("<?php\necho basename('/a/b.php', 42);\n").unwrap_err();
    assert_eq!(non_string_suffix.phase, Phase::Runtime);
    assert_eq!(non_string_suffix.line, 2);
    assert_eq!(non_string_suffix.column, 6);
    assert_eq!(
        non_string_suffix.message,
        "unsupported call basename(): suffix argument must be string in the current subset, got int"
    );

    let too_many = run_source("<?php\necho basename('/a/b.php', '.php', true);\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 6);
    assert_eq!(
        too_many.message,
        "arity mismatch for basename(): expected 1 to 2 argument(s), got 3"
    );
}

#[test]
fn basename_is_available_through_function_lookup() {
    let execution = run_source(
        r#"<?php
echo function_exists("basename") ? "exists" : "missing";
echo "\n";
echo is_callable("basename") ? "callable" : "not-callable";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "exists\ncallable");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_direct_basename_until_native_path_lowering_exists() {
    let error = emit_ir_source("<?php\necho basename('/a/b.php');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_BASENAME_REJECTION);
}

#[test]
fn emit_ir_rejects_basename_before_lowering_arguments() {
    let error = emit_ir_source("<?php\necho basename(42);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_BASENAME_REJECTION);
}

#[test]
fn emit_asm_rejects_basename_before_backend_execution() {
    let error = emit_asm_source("<?php\necho basename('/a/b.php');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_BASENAME_REJECTION);
}

#[test]
fn native_basename_emit_ir_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-ir",
        "tests/fixtures/milestone1198/native_basename_boundary_emit_ir.cli",
    );
}

#[test]
fn native_basename_emit_asm_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-asm",
        "tests/fixtures/milestone1198/native_basename_boundary_emit_asm.cli",
    );
}

#[test]
fn dirname_executes_current_unix_path_subset() {
    let execution = run_source(
        r#"<?php
echo dirname("/tmp/wordpress/wp-includes/sodium_compat/autoload.php"), "\n";
echo dirname("/tmp/wordpress/wp-includes/sodium_compat/"), "\n";
echo "[", dirname("autoload.php"), "]\n";
echo "[", dirname(""), "]\n";
echo dirname("/a/b/c.php", 2), "\n";
$call = "dirname";
echo $call("/a/b//c.php");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "/tmp/wordpress/wp-includes/sodium_compat\n/tmp/wordpress/wp-includes\n[.]\n[]\n/a\n/a/b"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dirname_reports_current_argument_boundaries() {
    let non_string_path = run_source("<?php\necho dirname(42);\n").unwrap_err();
    assert_eq!(non_string_path.phase, Phase::Runtime);
    assert_eq!(non_string_path.line, 2);
    assert_eq!(non_string_path.column, 6);
    assert_eq!(
        non_string_path.message,
        "unsupported call dirname(): path argument must be string in the current subset, got int"
    );

    let non_positive_levels = run_source("<?php\necho dirname('/a', 0);\n").unwrap_err();
    assert_eq!(non_positive_levels.phase, Phase::Runtime);
    assert_eq!(non_positive_levels.line, 2);
    assert_eq!(non_positive_levels.column, 6);
    assert_eq!(
        non_positive_levels.message,
        "unsupported call dirname(): levels argument must be greater than or equal to 1 in the current subset"
    );

    let non_int_levels = run_source("<?php\necho dirname('/a', '2');\n").unwrap_err();
    assert_eq!(non_int_levels.phase, Phase::Runtime);
    assert_eq!(non_int_levels.line, 2);
    assert_eq!(non_int_levels.column, 6);
    assert_eq!(
        non_int_levels.message,
        "unsupported call dirname(): levels argument must be int in the current subset, got string"
    );
}

#[test]
fn emit_ir_rejects_direct_dirname_until_native_path_lowering_exists() {
    let error = emit_ir_source("<?php\necho dirname('/a/b.php');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

fn assert_cli_snapshot_matches(mode: &str, snapshot_path: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone1198/native_basename_boundary.phpc-source");
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
        .expect("native basename CLI snapshot is readable");
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
