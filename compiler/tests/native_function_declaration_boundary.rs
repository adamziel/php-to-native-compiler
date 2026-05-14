use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_FUNCTION_DECLARATION_REJECTION: &str = "LLVM user-function lowering rejects function declarations and return statements until native function symbol tables, stack-frame layout, default parameter binding, recursion guards, return-value flow, and exact native error behavior exist; phpc run handles current user-function declaration and return behavior";

#[test]
fn phpc_run_still_handles_current_user_function_declarations_and_returns() {
    let execution = run_source(
        r#"<?php
function join_label($prefix, $name = "Ada") {
    if ($name === "stop") {
        return $prefix . ":stopped";
    }
    return $prefix . ":" . $name;
}
echo join_label("hello"), "\n";
echo join_label("bye", "Grace");
return "finished";
echo "after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "hello:Ada\nbye:Grace");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_function_declarations_with_specific_boundary() {
    let error = emit_ir_source(
        r#"<?php
function label($value = "Ada") {
    return $value;
}
echo "after";
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_DECLARATION_REJECTION);
}

#[test]
fn emit_ir_rejects_function_declarations_with_trailing_parameter_commas() {
    let error = emit_ir_source(
        r#"<?php
function label($value = "Ada",) {
    return $value;
}
echo label();
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_DECLARATION_REJECTION);
}

#[test]
fn emit_ir_rejects_function_declarations_before_lowering_bodies() {
    let error = emit_ir_source(
        r#"<?php
function first() {
    return array_map("strlen", ["Ada"]);
}
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_DECLARATION_REJECTION);
}

#[test]
fn emit_ir_rejects_return_statements_with_specific_boundary() {
    let error = emit_ir_source("<?php\nreturn 1;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_DECLARATION_REJECTION);
}

#[test]
fn emit_asm_rejects_function_declarations_before_backend_execution() {
    let error = emit_asm_source("<?php\nfunction label() { return 1; }\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_DECLARATION_REJECTION);
}

#[test]
fn native_function_declaration_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone170/native_function_declaration_boundary.php");
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
            .join("tests/fixtures/milestone170/native_function_declaration_boundary_emit_ir.cli"),
    )
    .expect("native function declaration CLI snapshot is readable");
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
