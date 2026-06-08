use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_CONCAT_REJECTION: &str = "LLVM concatenation lowering rejects unsupported concatenation operands until native PHP scalar-to-string conversion, dynamic allocation, references/copy-on-write, and exact native error behavior exist; phpc run handles current concatenation behavior";
const LLVM_INTERPOLATED_STRING_REJECTION: &str = "LLVM interpolated-string lowering rejects double-quoted string interpolation until native interpolation part evaluation, PHP-shaped string conversion, array/object lookup, __toString dispatch, runtime string allocation, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded interpolation behavior";

#[test]
fn phpc_run_still_handles_current_concatenation_subset() {
    let execution = run_source(
        r#"<?php
echo "hello" . " " . "world", "\n";
echo "value=" . 7 . "\n";
echo null . false . true . "!";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "hello world\nvalue=7\n1!");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_concatenation_with_specific_boundary() {
    let error = emit_ir_source("<?php\necho \"a\" . 1;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONCAT_REJECTION);
}

#[test]
fn emit_ir_rejects_array_concatenation_before_lowering_operands() {
    let error = emit_ir_source("<?php\necho [] . [];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONCAT_REJECTION);
}

#[test]
fn emit_asm_rejects_concatenation_before_backend_execution() {
    let error = emit_asm_source("<?php\necho \"a\" . 1;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONCAT_REJECTION);
}

#[test]
fn emit_ir_rejects_interpolated_strings_with_specific_boundary() {
    for source in [
        r#"<?php
$constant = "RUNTIME";
echo "APP_$constant";
"#,
        r#"<?php
$constant = "RUNTIME";
echo "APP_{$constant}";
"#,
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.line, 3);
        assert_eq!(error.column, 6);
        assert_eq!(error.message, LLVM_INTERPOLATED_STRING_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_interpolated_strings_before_backend_execution() {
    let error = emit_asm_source(
        r#"<?php
$constant = "RUNTIME";
echo "APP_{$constant}";
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_INTERPOLATED_STRING_REJECTION);
}

#[test]
fn emit_ir_lowers_static_string_concatenation() {
    let ir = emit_ir_source(
        r#"<?php
$hello = "hello";
$space = " ";
$world = "world";
$message = $hello . $space . $world;
echo $message, "\n";
echo "say: " . $message;
"#,
    )
    .unwrap();

    assert!(ir.contains("c\"hello world\\00\""), "{ir}");
    assert!(ir.contains("c\"say: hello world\\00\""), "{ir}");
    assert!(
        !ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int"),
        "{ir}"
    );
}

#[test]
fn emit_ir_lowers_single_result_string_ternary_concatenation() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;

echo ($flag ? "he" : "he") . "llo", "\n";
echo "say " . ($flag ? "yes" : "yes");
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("c\"hello\\00\""), "{ir}");
    assert!(ir.contains("c\"say yes\\00\""), "{ir}");
    assert!(
        !ir.contains("select i1 %tmp1, ptr"),
        "single-result string ternary concat operand should fold to a static string:\n{ir}"
    );
}

#[test]
fn emit_ir_rejects_ambiguous_string_ternary_concatenation() {
    let error = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;

echo ($flag ? "a" : "b") . "!";
"#,
    )
    .expect_err("ambiguous string ternary concatenation still needs runtime allocation");

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONCAT_REJECTION);
}

#[test]
fn emit_ir_folds_empty_string_concat_with_untracked_string_expression() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$choice = $flag ? 3 : 4;
$ambiguous = $sum === $choice;
$left = $ambiguous ? "alpha" : "bravo";
$middle = $ambiguous ? "charlie" : "delta";
$wide = $ambiguous ? $left : "echo";
$text = $ambiguous ? $wide : $middle;

echo $text . "", "\n";
echo "" . $text;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 3, i64 4"), "{ir}");
    assert!(
        ir.contains("%tmp3 = icmp eq i64 %tmp0, %tmp2"),
        "source ambiguous condition should stay emitted:\n{ir}"
    );
    assert_eq!(
        ir.matches("select i1").count(),
        8,
        "untracked source string expression should stay emitted through pointer and length selects:\n{ir}"
    );
    assert_eq!(
        ir.matches("@phpc_native_value_from_string_bytes_with_diagnostic(ptr %tmp9, i64 %tmp10")
            .count(),
        2,
        "empty-string concatenation should reuse the string pointer and length expression:\n{ir}"
    );
    assert!(
        !ir.contains("c\"alpha\\00\\00\""),
        "identity concat must not invent a static string result for the dynamic expression:\n{ir}"
    );
}

#[test]
fn native_concat_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone179/native_concat_boundary.php");
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
        workspace_root.join("tests/fixtures/milestone179/native_concat_boundary_emit_ir.cli"),
    )
    .expect("native concat CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_interpolated_string_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1168/native_interpolated_string_boundary.phpc-source");
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
            .join("tests/fixtures/milestone1168/native_interpolated_string_boundary_emit_ir.cli"),
    )
    .expect("native interpolated-string IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_interpolated_string_emit_asm_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1168/native_interpolated_string_boundary.phpc-source");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone1168/native_interpolated_string_boundary_emit_asm.cli"),
    )
    .expect("native interpolated-string assembly CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_single_result_string_ternary_concat_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone481/native_single_result_string_ternary_concat.php");
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

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone481/native_single_result_string_ternary_concat_emit_ir.cli",
    ))
    .expect("native single-result string ternary concat IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_empty_string_concat_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone514/native_empty_string_concat_identity.php");
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
            .join("tests/fixtures/milestone514/native_empty_string_concat_identity_emit_ir.cli"),
    )
    .expect("native empty-string concat identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_static_string_concat_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone245/native_static_string_concat_emit_ir.php");
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
        workspace_root.join("tests/fixtures/milestone245/native_static_string_concat_emit_ir.cli"),
    )
    .expect("native static string concat IR CLI snapshot is readable");
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
