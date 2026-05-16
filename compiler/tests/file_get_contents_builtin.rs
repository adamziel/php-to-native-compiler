use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::emit_asm_source;
use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::{run_source, run_source_with_source_file};

const LLVM_FILE_GET_CONTENTS_REJECTION: &str = "LLVM file_get_contents lowering rejects direct filesystem reads until native PHP stream wrapper handling, local file I/O, binary string byte fidelity, warning plus false recovery, stream contexts, offsets/lengths, include-path lookup, open_basedir/stat-cache behavior, references/copy-on-write, and exact native file_get_contents diagnostics exist; phpc run handles current bounded file_get_contents behavior";

fn fixture_source_file() -> String {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("compiler has a workspace root")
        .join("tests/fixtures/milestone1197/local_file_get_contents.php")
        .display()
        .to_string()
}

#[test]
fn file_get_contents_reads_empty_php_input_placeholder() {
    let execution = run_source(
        r#"<?php
$raw = file_get_contents("php://input");
echo $raw === "" ? "empty" : "non-empty";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "empty");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_get_contents_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "file_get_contents";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("php://input") === "" ? "empty" : "non-empty";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|empty");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_get_contents_reads_current_local_utf8_subset() {
    let execution = run_source_with_source_file(
        r#"<?php
$path = __DIR__ . "/local_read_payload.txt";
$contents = file_get_contents($path);
echo str_contains($contents, "ABSPATH") ? "wp-config" : "missing";
echo "|";
$call = "file_get_contents";
echo $call($path) === $contents ? "repeat" : "different";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(execution.stdout, "wp-config|repeat");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn file_get_contents_rejects_forms_outside_current_subset() {
    let non_string = run_source("<?php\nfile_get_contents(42);\n").unwrap_err();
    assert_eq!(non_string.phase, Phase::Runtime);
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 1);
    assert_eq!(
        non_string.message,
        "unsupported call file_get_contents(): path argument must be string in the current subset, got int"
    );

    let stream = run_source("<?php\nfile_get_contents('php://memory');\n").unwrap_err();
    assert_eq!(stream.phase, Phase::Runtime);
    assert_eq!(stream.line, 2);
    assert_eq!(stream.column, 1);
    assert_eq!(
        stream.message,
        "unsupported call file_get_contents(): only php://input is supported in the current stream-wrapper subset"
    );

    let missing_local_file =
        run_source("<?php\nfile_get_contents('tests/fixtures/missing-local-read.txt');\n")
            .unwrap_err();
    assert_eq!(missing_local_file.phase, Phase::Runtime);
    assert_eq!(missing_local_file.line, 2);
    assert_eq!(missing_local_file.column, 1);
    assert!(
        missing_local_file
            .message
            .starts_with("unsupported call file_get_contents(): local UTF-8 file read failed:"),
        "{}",
        missing_local_file.message
    );

    let too_many = run_source("<?php\nfile_get_contents('php://input', false);\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for file_get_contents(): expected 1 argument(s), got 2"
    );
}

#[test]
fn emit_ir_folds_file_get_contents_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("file_get_contents") ? "1" : "0";
echo is_callable("file_get_contents") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nfile_get_contents('php://input');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FILE_GET_CONTENTS_REJECTION);
}

#[test]
fn emit_ir_rejects_file_get_contents_before_lowering_arguments() {
    let error = emit_ir_source("<?php\nfile_get_contents(42);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FILE_GET_CONTENTS_REJECTION);
}

#[test]
fn emit_asm_rejects_file_get_contents_before_backend_execution() {
    let error = emit_asm_source("<?php\nfile_get_contents('php://input');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FILE_GET_CONTENTS_REJECTION);
}

#[test]
fn native_file_get_contents_emit_ir_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-ir",
        "tests/fixtures/milestone1203/native_file_get_contents_boundary_emit_ir.cli",
    );
}

#[test]
fn native_file_get_contents_emit_asm_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-asm",
        "tests/fixtures/milestone1203/native_file_get_contents_boundary_emit_asm.cli",
    );
}

fn assert_cli_snapshot_matches(mode: &str, snapshot_path: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1203/native_file_get_contents_boundary.phpc-source");
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
        .expect("native file_get_contents CLI snapshot is readable");
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
