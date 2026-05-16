use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::emit_asm_source;
use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_STR_ENDS_WITH_REJECTION: &str = "LLVM str_ends_with lowering rejects direct string-suffix calls until native PHP string conversion, empty-needle handling, binary string byte semantics, argument diagnostics, references/copy-on-write, and exact native str_ends_with diagnostics exist; phpc run handles current bounded str_ends_with behavior";

#[test]
fn str_ends_with_executes_current_scalar_string_subset() {
    let execution = run_source(
        r#"<?php
echo str_ends_with("index.php", ".php") ? "yes" : "no";
echo "|";
echo str_ends_with("index.php", "php.cgi") ? "yes" : "no";
echo "|";
echo str_ends_with("index.php", "") ? "empty" : "no";
echo "|";
echo str_ends_with(42, "2") ? "coerced" : "no";
echo "|";
echo str_ends_with(null, "") ? "null-empty" : "no";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|no|empty|coerced|null-empty");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_ends_with_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "str_ends_with";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("index.php", ".php") ? "suffix" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|suffix");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_ends_with_rejects_forms_outside_current_subset() {
    let array_haystack = run_source("<?php\nstr_ends_with(['abc'], 'c');\n").unwrap_err();
    assert_eq!(array_haystack.phase, Phase::Runtime);
    assert_eq!(array_haystack.line, 2);
    assert_eq!(array_haystack.column, 1);
    assert_eq!(
        array_haystack.message,
        "unsupported call str_ends_with(): haystack argument arrays are not implemented in the current subset"
    );

    let array_needle = run_source("<?php\nstr_ends_with('abc', ['c']);\n").unwrap_err();
    assert_eq!(array_needle.phase, Phase::Runtime);
    assert_eq!(array_needle.line, 2);
    assert_eq!(array_needle.column, 1);
    assert_eq!(
        array_needle.message,
        "unsupported call str_ends_with(): needle argument arrays are not implemented in the current subset"
    );

    let too_few = run_source("<?php\nstr_ends_with('abc');\n").unwrap_err();
    assert_eq!(too_few.phase, Phase::Runtime);
    assert_eq!(too_few.line, 2);
    assert_eq!(too_few.column, 1);
    assert_eq!(
        too_few.message,
        "arity mismatch for str_ends_with(): expected 2 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_str_ends_with_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("str_ends_with") ? "1" : "0";
echo is_callable("str_ends_with") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nstr_ends_with('abc', 'c');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_STR_ENDS_WITH_REJECTION);
}

#[test]
fn emit_ir_rejects_str_ends_with_before_lowering_arguments() {
    let error = emit_ir_source("<?php\nstr_ends_with([], 'c');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_STR_ENDS_WITH_REJECTION);
}

#[test]
fn emit_asm_rejects_str_ends_with_before_backend_execution() {
    let error = emit_asm_source("<?php\nstr_ends_with('abc', 'c');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_STR_ENDS_WITH_REJECTION);
}

#[test]
fn native_str_ends_with_emit_ir_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-ir",
        "tests/fixtures/milestone1193/native_str_ends_with_boundary_emit_ir.cli",
    );
}

#[test]
fn native_str_ends_with_emit_asm_cli_snapshot_matches_committed_output() {
    assert_cli_snapshot_matches(
        "--emit-asm",
        "tests/fixtures/milestone1193/native_str_ends_with_boundary_emit_asm.cli",
    );
}

fn assert_cli_snapshot_matches(mode: &str, snapshot_path: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1193/native_str_ends_with_boundary.phpc-source");
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
        .expect("native str_ends_with CLI snapshot is readable");
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
